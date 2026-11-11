//! Pure-Rust WebRTC UDP Selective Forwarding Unit (SFU) + Binary UDP DataChannel Engine
//!
//! Multiplexes ALL browser clients' Opus voice audio (`DTLS-SRTP`) AND real-time 3D game state
//! (`SCTP/DTLS` DataChannels `ella-udp-state` + `ella-reliable-events`) over ONE single UDP port
//! (`ELLA_WEBRTC_UDP_PORT`, default `44444/udp`) via `UDPMuxDefault`.

use crate::udp_native::handle_client_event_command;
use crate::udp_wire::{decode_c2s_state, encode_pong, OP_C2S_PING, OP_C2S_STATE};
use crate::world::{BridgedOpusFrame, UdpWorldEvent, WorldEngine};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::{APIBuilder, API};
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice::network_type::NetworkType;
use webrtc::ice::udp_mux::{UDPMuxDefault, UDPMuxParams};
use webrtc::ice::udp_network::UDPNetwork;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::{
    RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType,
};
use webrtc::rtp_transceiver::rtp_sender::RTCRtpSender;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};

struct PeerRtcSession {
    self_id: String,
    username: String,
    pc: Arc<RTCPeerConnection>,
    events_dc: RwLock<Option<Arc<RTCDataChannel>>>,
    renegotiate_lock: Mutex<()>,
    subscribed_senders: Mutex<HashMap<String, Arc<RTCRtpSender>>>,
}

pub struct WebRtcSfu {
    api: API,
    udp_port: u16,
    world: Arc<WorldEngine>,
    sessions: RwLock<HashMap<String, Arc<PeerRtcSession>>>,
    voice_tracks: RwLock<HashMap<String, Arc<TrackLocalStaticRTP>>>,
    active_count: AtomicUsize,
}

impl WebRtcSfu {
    pub async fn bind_and_start(world: Arc<WorldEngine>) -> anyhow::Result<Arc<Self>> {
        let bind_addr: SocketAddr = format!(
            "{}:{}",
            world.config.webrtc_udp_bind_ip, world.config.webrtc_udp_port
        )
        .parse()?;

        let udp_socket = UdpSocket::bind(bind_addr).await?;
        let actual_port = udp_socket.local_addr()?.port();
        info!(
            "🎙️ [WebRTC SFU] Single-port multiplexed UDP Voice + State listening on {}:{}/udp",
            world.config.webrtc_udp_bind_ip, actual_port
        );

        let udp_mux = UDPMuxDefault::new(UDPMuxParams::new(udp_socket));

        let mut media_engine = MediaEngine::default();
        media_engine.register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: MIME_TYPE_OPUS.to_owned(),
                    clock_rate: 48_000,
                    channels: 2,
                    sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                    rtcp_feedback: vec![],
                },
                payload_type: 111,
                ..Default::default()
            },
            RTPCodecType::Audio,
        )?;

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)?;

        let mut setting_engine = SettingEngine::default();
        setting_engine.set_network_types(vec![NetworkType::Udp4]);
        setting_engine.set_udp_network(UDPNetwork::Muxed(udp_mux));

        // Exclude container virtual bridge interfaces (`docker0`, `br-*`, `veth*`) so browsers
        // only receive genuine physical LAN & WAN UDP candidates.
        setting_engine.set_interface_filter(Box::new(|iface_name: &str| {
            !iface_name.starts_with("docker")
                && !iface_name.starts_with("br-")
                && !iface_name.starts_with("veth")
                && !iface_name.starts_with("virbr")
        }));

        if world.config.webrtc_udp_bind_ip == "127.0.0.1" || world.config.public_ip.is_none() {
            setting_engine.set_include_loopback_candidate(true);
        }

        if let Some(pub_ip) = &world.config.public_ip {
            info!(
                "🌐 [WebRTC SFU] Advertising dual LAN Host + WAN Srflx UDP candidate {}:{}",
                pub_ip, actual_port
            );
        }

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .with_setting_engine(setting_engine)
            .build();

        let sfu = Arc::new(Self {
            api,
            udp_port: actual_port,
            world: Arc::clone(&world),
            sessions: RwLock::new(HashMap::new()),
            voice_tracks: RwLock::new(HashMap::new()),
            active_count: AtomicUsize::new(0),
        });

        // Bridge incoming Native Raw UDP Opus packets into WebRTC SFU tracks so browser players
        // also hear native UDP voice clients!
        let bridge_sfu = Arc::clone(&sfu);
        let mut opus_rx = world.opus_bridge_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(frame) = opus_rx.recv().await {
                if !frame.from_native_udp {
                    continue;
                }
                let track = bridge_sfu.ensure_or_get_voice_track(&frame.from_id).await;
                let rtp_pkt = webrtc::rtp::packet::Packet {
                    header: webrtc::rtp::header::Header {
                        version: 2,
                        payload_type: 111,
                        sequence_number: frame.seq,
                        timestamp: frame.timestamp,
                        ssrc: 0xE11A_0001,
                        ..Default::default()
                    },
                    payload: frame.payload,
                };
                let _ = track.write_rtp(&rtp_pkt).await;
            }
        });

        Ok(sfu)
    }

    pub fn udp_port(&self) -> u16 {
        self.udp_port
    }

    pub fn active_session_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }

    async fn ensure_or_get_voice_track(
        self: &Arc<Self>,
        from_id: &str,
    ) -> Arc<TrackLocalStaticRTP> {
        if let Some(existing) = self.voice_tracks.read().await.get(from_id).cloned() {
            return existing;
        }
        let local_track = Arc::new(TrackLocalStaticRTP::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: 48_000,
                channels: 2,
                sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                rtcp_feedback: vec![],
            },
            format!("voice_{from_id}"),
            from_id.to_string(),
        ));
        self.voice_tracks
            .write()
            .await
            .insert(from_id.to_string(), Arc::clone(&local_track));

        // Attach to all other currently connected WebRTC sessions
        let peers: Vec<Arc<PeerRtcSession>> = self
            .sessions
            .read()
            .await
            .values()
            .filter(|s| s.self_id != from_id)
            .cloned()
            .collect();

        for peer_sess in peers {
            let sfu_clone = Arc::clone(self);
            let from_key = from_id.to_string();
            let trk = Arc::clone(&local_track);
            tokio::spawn(async move {
                sfu_clone
                    .attach_remote_voice_to_session(&peer_sess, &from_key, trk)
                    .await;
            });
        }

        local_track
    }

    async fn attach_remote_voice_to_session(
        &self,
        target_sess: &Arc<PeerRtcSession>,
        source_peer_id: &str,
        track: Arc<TrackLocalStaticRTP>,
    ) {
        let _guard = target_sess.renegotiate_lock.lock().await;
        {
            let mut subs = target_sess.subscribed_senders.lock().await;
            if subs.contains_key(source_peer_id) {
                return;
            }
            match target_sess
                .pc
                .add_track(Arc::clone(&track) as Arc<dyn TrackLocal + Send + Sync>)
                .await
            {
                Ok(rtp_sender) => {
                    // Drain RTCP receiver reports so sender buffer never stalls
                    let rtcp_reader = Arc::clone(&rtp_sender);
                    tokio::spawn(async move {
                        let mut rtcp_buf = vec![0u8; 1500];
                        while rtcp_reader.read(&mut rtcp_buf).await.is_ok() {}
                    });
                    subs.insert(source_peer_id.to_string(), rtp_sender);
                }
                Err(err) => {
                    warn!(
                        "Failed to add SFU voice track {} -> {}: {}",
                        source_peer_id, target_sess.self_id, err
                    );
                    return;
                }
            }
        }

        self.send_renegotiate_offer_locked(target_sess).await;
    }

    async fn send_renegotiate_offer_locked(&self, target_sess: &Arc<PeerRtcSession>) {
        let Some(dc) = target_sess.events_dc.read().await.clone() else {
            return;
        };
        let mut gather_done = target_sess.pc.gathering_complete_promise().await;
        let Ok(offer) = target_sess.pc.create_offer(None).await else {
            return;
        };
        if target_sess.pc.set_local_description(offer).await.is_err() {
            return;
        }
        let _ =
            tokio::time::timeout(std::time::Duration::from_millis(800), gather_done.recv()).await;
        let Some(local_desc) = target_sess.pc.local_description().await else {
            return;
        };
        let sdp = self.inject_public_srflx_candidate(&local_desc.sdp);
        let msg = serde_json::json!({
            "type": "rtc_renegotiate",
            "sdp": sdp
        });
        if let Ok(txt) = serde_json::to_string(&msg) {
            let _ = dc.send_text(txt).await;
        }
    }

    /// Because `webrtc-ice` skips `ServerReflexive` candidate gathering when `UDPNetwork::Muxed`
    /// is active (and `CandidateType::Host` 1:1 NAT mapping would overwrite the LAN IP needed for
    /// hairpin-free local Wi-Fi play), we append an RFC 8445 `typ srflx` UDP candidate on our
    /// multiplexed UDP port whenever `ELLA_PUBLIC_IP` is configured.
    fn inject_public_srflx_candidate(&self, sdp: &str) -> String {
        let Some(pub_ip) = &self.world.config.public_ip else {
            return sdp.to_string();
        };
        if sdp.contains(pub_ip) {
            return sdp.to_string();
        }
        let port = self.udp_port;
        let srflx_c1 = format!(
            "a=candidate:999999001 1 udp 1694498815 {pub_ip} {port} typ srflx raddr 0.0.0.0 rport {port}"
        );
        let mut out = String::with_capacity(sdp.len() + 256);
        for line in sdp.split_inclusive('\n') {
            out.push_str(line);
            let trimmed = line.trim_end_matches(['\r', '\n']);
            // Inject right after the first component-1 host candidate in each media section
            if trimmed.starts_with("a=candidate:")
                && trimmed.contains(" 1 udp ")
                && trimmed.ends_with("typ host")
            {
                let eol = if line.ends_with("\r\n") { "\r\n" } else { "\n" };
                out.push_str(&srflx_c1);
                out.push_str(eol);
            }
        }
        out
    }

    pub async fn cleanup_session(self: &Arc<Self>, self_id: &str) {
        let removed_sess = {
            let mut map = self.sessions.write().await;
            let s = map.remove(self_id);
            self.active_count.store(map.len(), Ordering::Relaxed);
            s
        };
        if let Some(s) = removed_sess {
            let _ = s.pc.close().await;
        }

        let removed_track = self.voice_tracks.write().await.remove(self_id).is_some();
        if removed_track {
            let peers: Vec<Arc<PeerRtcSession>> =
                self.sessions.read().await.values().cloned().collect();
            for peer_sess in peers {
                let _guard = peer_sess.renegotiate_lock.lock().await;
                let maybe_sender = peer_sess.subscribed_senders.lock().await.remove(self_id);
                if let Some(sender) = maybe_sender {
                    let _ = peer_sess.pc.remove_track(&sender).await;
                    self.send_renegotiate_offer_locked(&peer_sess).await;
                }
            }
        }
    }

    /// Accepts a browser's SDP Offer (`POST /api/ella-world/rtc/offer`), sets up:
    /// - Inbound Opus RTP voice track -> forwarded over UDP (`44444/udp`) to all peers
    /// - `ella-udp-state` unreliable binary DataChannel (`20 Hz` UDP movement/emotes/RTT)
    /// - `ella-reliable-events` reliable UDP DataChannel (blocks, chat, outfits, runway & SFU renegotiation)
    ///
    /// Returns the complete gathered SDP Answer.
    pub async fn accept_browser_offer(
        self: &Arc<Self>,
        self_id: String,
        username: String,
        offer_sdp: String,
    ) -> anyhow::Result<String> {
        // Clean up any prior stale PC for the exact same self_id
        self.cleanup_session(&self_id).await;

        let pc = Arc::new(
            self.api
                .new_peer_connection(RTCConfiguration::default())
                .await?,
        );

        let session = Arc::new(PeerRtcSession {
            self_id: self_id.clone(),
            username: username.clone(),
            pc: Arc::clone(&pc),
            events_dc: RwLock::new(None),
            renegotiate_lock: Mutex::new(()),
            subscribed_senders: Mutex::new(HashMap::new()),
        });

        {
            let mut map = self.sessions.write().await;
            map.insert(self_id.clone(), Arc::clone(&session));
            self.active_count.store(map.len(), Ordering::Relaxed);
        }

        // 1. Lifecycle state monitor
        let state_sfu = Arc::clone(self);
        let state_world = Arc::clone(&self.world);
        let state_self_id = self_id.clone();
        pc.on_peer_connection_state_change(Box::new(move |st: RTCPeerConnectionState| {
            let sfu_c = Arc::clone(&state_sfu);
            let world_c = Arc::clone(&state_world);
            let sid = state_self_id.clone();
            Box::pin(async move {
                if matches!(
                    st,
                    RTCPeerConnectionState::Failed
                        | RTCPeerConnectionState::Closed
                        | RTCPeerConnectionState::Disconnected
                ) {
                    sfu_c.cleanup_session(&sid).await;
                    if matches!(
                        st,
                        RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed
                    ) {
                        world_c.remove_player_immediately(&sid).await;
                    }
                }
            })
        }));

        // 2. Incoming Opus voice RTP stream over UDP -> fan out via SFU TrackLocalStaticRTP
        let track_sfu = Arc::clone(self);
        let track_world = Arc::clone(&self.world);
        let track_self_id = self_id.clone();
        pc.on_track(Box::new(move |remote_track, _receiver, _transceiver| {
            let sfu_c = Arc::clone(&track_sfu);
            let world_c = Arc::clone(&track_world);
            let sid = track_self_id.clone();
            Box::pin(async move {
                if remote_track.kind() != RTPCodecType::Audio {
                    return;
                }
                info!(
                    "🎤 [WebRTC SFU] Receiving UDP Opus voice RTP stream from {}",
                    sid
                );
                let local_track = sfu_c.ensure_or_get_voice_track(&sid).await;

                while let Ok((rtp_pkt, _attrs)) = remote_track.read_rtp().await {
                    let _ = world_c.opus_bridge_tx.send(BridgedOpusFrame {
                        from_id: sid.clone(),
                        from_native_udp: false,
                        seq: rtp_pkt.header.sequence_number,
                        timestamp: rtp_pkt.header.timestamp,
                        payload: rtp_pkt.payload.clone(),
                    });
                    if local_track.write_rtp(&rtp_pkt).await.is_err() {
                        break;
                    }
                }
            })
        }));

        // 3. Incoming DataChannels over UDP (`ella-udp-state` & `ella-reliable-events`)
        let dc_sfu = Arc::clone(self);
        let dc_world = Arc::clone(&self.world);
        let dc_sess = Arc::clone(&session);
        pc.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
            let label = dc.label().to_string();
            let sfu_c = Arc::clone(&dc_sfu);
            let world_c = Arc::clone(&dc_world);
            let sess_c = Arc::clone(&dc_sess);

            Box::pin(async move {
                if label == "ella-udp-state" {
                    Self::wire_unreliable_state_channel(dc, world_c, sess_c).await;
                } else if label == "ella-reliable-events" {
                    Self::wire_reliable_events_channel(sfu_c, dc, world_c, sess_c).await;
                }
            })
        }));

        let offer = RTCSessionDescription::offer(offer_sdp)?;
        pc.set_remote_description(offer).await?;

        let mut gather_complete = pc.gathering_complete_promise().await;
        let answer = pc.create_answer(None).await?;
        pc.set_local_description(answer).await?;
        let _ =
            tokio::time::timeout(std::time::Duration::from_secs(2), gather_complete.recv()).await;

        let local_desc = pc
            .local_description()
            .await
            .ok_or_else(|| anyhow::anyhow!("Missing local SDP description"))?;

        Ok(self.inject_public_srflx_candidate(&local_desc.sdp))
    }

    async fn wire_unreliable_state_channel(
        dc: Arc<RTCDataChannel>,
        world: Arc<WorldEngine>,
        sess: Arc<PeerRtcSession>,
    ) {
        // Inbound 28-byte binary C2S_STATE (0x10) & C2S_PING (0x12) datagrams
        let in_world = Arc::clone(&world);
        let in_dc = Arc::clone(&dc);
        let in_self_id = sess.self_id.clone();
        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            let w = Arc::clone(&in_world);
            let d = Arc::clone(&in_dc);
            let sid = in_self_id.clone();
            Box::pin(async move {
                let raw = msg.data.as_ref();
                if raw.is_empty() {
                    return;
                }
                match raw[0] {
                    OP_C2S_STATE => {
                        if let Some(st) = decode_c2s_state(raw) {
                            w.apply_c2s_state(&sid, st).await;
                        }
                    }
                    OP_C2S_PING if raw.len() >= 5 => {
                        let client_ts = u32::from_le_bytes([raw[1], raw[2], raw[3], raw[4]]);
                        w.touch_player(&sid).await;
                        let pong = encode_pong(client_ts, w.current_tick());
                        let _ = d.send(&pong).await;
                    }
                    _ => {}
                }
            })
        }));

        // Outbound 20Hz S2C_WORLD_STATE (0x11) binary datagram pump while channel is open
        let out_dc = Arc::clone(&dc);
        let out_world = Arc::clone(&world);
        let out_self_id = sess.self_id.clone();
        dc.on_open(Box::new(move || {
            let d = Arc::clone(&out_dc);
            let w = Arc::clone(&out_world);
            let sid = out_self_id.clone();
            Box::pin(async move {
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
                    loop {
                        interval.tick().await;
                        let snap = w.encode_world_snapshot_for(&sid).await;
                        if d.send(&snap).await.is_err() {
                            break;
                        }
                    }
                });
            })
        }));
    }

    async fn wire_reliable_events_channel(
        sfu: Arc<Self>,
        dc: Arc<RTCDataChannel>,
        world: Arc<WorldEngine>,
        sess: Arc<PeerRtcSession>,
    ) {
        // Inbound reliable JSON commands + `rtc_answer` for SFU voice renegotiation
        let msg_world = Arc::clone(&world);
        let msg_sess = Arc::clone(&sess);
        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            let w = Arc::clone(&msg_world);
            let s = Arc::clone(&msg_sess);
            Box::pin(async move {
                let Ok(txt) = std::str::from_utf8(&msg.data) else {
                    return;
                };
                let Ok(cmd) = serde_json::from_str::<serde_json::Value>(txt) else {
                    return;
                };
                if cmd.get("type").and_then(|v| v.as_str()) == Some("rtc_answer") {
                    if let Some(ans_sdp) = cmd.get("sdp").and_then(|v| v.as_str()) {
                        if let Ok(ans) = RTCSessionDescription::answer(ans_sdp.to_string()) {
                            let _ = s.pc.set_remote_description(ans).await;
                        }
                    }
                    return;
                }
                handle_client_event_command(&w, &s.self_id, &s.username, &cmd).await;
            })
        }));

        // When reliable channel opens: store handle, send initial Roster snapshot,
        // attach any existing SFU voice tracks, and forward live UdpWorldEvents!
        let open_sfu = Arc::clone(&sfu);
        let open_dc = Arc::clone(&dc);
        let open_world = Arc::clone(&world);
        let open_sess = Arc::clone(&sess);
        dc.on_open(Box::new(move || {
            let sfu_c = Arc::clone(&open_sfu);
            let d = Arc::clone(&open_dc);
            let w = Arc::clone(&open_world);
            let s = Arc::clone(&open_sess);
            Box::pin(async move {
                *s.events_dc.write().await = Some(Arc::clone(&d));

                // 1. Send initial Roster + blocksVersion over UDP
                let others: Vec<_> = w
                    .active_players_list()
                    .await
                    .into_iter()
                    .filter(|p| p.id != s.self_id)
                    .collect();
                let bver = w.blocks_data.read().await.version;
                let roster_ev = UdpWorldEvent::Roster {
                    players: others,
                    blocks_version: bver,
                };
                if let Ok(json_txt) = serde_json::to_string(&roster_ev) {
                    let _ = d.send_text(json_txt).await;
                }

                // 2. Attach all currently active SFU Opus voice tracks from other players in the room
                let existing_tracks: Vec<(String, Arc<TrackLocalStaticRTP>)> = sfu_c
                    .voice_tracks
                    .read()
                    .await
                    .iter()
                    .filter(|(peer_id, _)| *peer_id != &s.self_id)
                    .map(|(k, v)| (k.clone(), Arc::clone(v)))
                    .collect();

                for (source_id, trk) in existing_tracks {
                    sfu_c
                        .attach_remote_voice_to_session(&s, &source_id, trk)
                        .await;
                }

                // 3. Forward all discrete world events over UDP until channel closes
                let mut ev_rx = w.udp_event_tx.subscribe();
                let pump_dc = Arc::clone(&d);
                let self_id_filter = s.self_id.clone();
                tokio::spawn(async move {
                    while let Ok(ev) = ev_rx.recv().await {
                        if let UdpWorldEvent::PlayerJoin { player } = &ev {
                            if player.id == self_id_filter {
                                continue;
                            }
                        }
                        if let Ok(json_txt) = serde_json::to_string(&ev) {
                            if pump_dc.send_text(json_txt).await.is_err() {
                                break;
                            }
                        }
                    }
                });
            })
        }));
    }
}

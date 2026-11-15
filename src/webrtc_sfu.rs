//! Pure-Rust Zero-Renegotiation WebRTC UDP Selective Forwarding Unit (SFU) + Fast-Lane State Engine
//!
//! Multiplexes ALL browser clients' Opus voice audio (`DTLS-SRTP`) AND 20 Hz unreliable 3D game state
//! (`SCTP/DTLS` DataChannel `ella-udp-state`) over ONE single UDP port (`44444/udp`) via `UDPMuxDefault`.
//!
//! ### Why Zero Renegotiation?
//! Instead of calling `add_track` / `remove_track` / `rtc_renegotiate` mid-game (which resets ICE
//! candidates and triggers 8.8s disconnect loops across browsers), the client offers up to
//! `MAX_VOICE_SLOTS` (`8`) pre-allocated `recvonly` Opus transceivers (`ella_slot_0`..`ella_slot_7`)
//! during the initial HTTP SDP handshake. Speakers are dynamically routed into pre-bound slots in
//! `O(1)` memory with monotonic per-slot RTP sequence numbers and lightweight `voice_slot_map` events.

use crate::models::now_ms;
use crate::udp_native::handle_client_event_command;
use crate::udp_wire::{decode_c2s_state, encode_pong, OP_C2S_PING, OP_C2S_STATE};
use crate::world::{BridgedOpusFrame, UdpWorldEvent, WorldEngine};
use bytes::Bytes;
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
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
use webrtc::rtp_transceiver::rtp_transceiver_direction::RTCRtpTransceiverDirection;
use webrtc::rtp_transceiver::RTCRtpTransceiverInit;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};

pub const MAX_VOICE_SLOTS: usize = 8;

struct VoiceSlot {
    slot_id: String,
    track: Arc<TrackLocalStaticRTP>,
    seq: AtomicU16,
    last_audio_ms: AtomicU64,
    assigned_peer: RwLock<Option<String>>,
}

struct PeerRtcSession {
    session_nonce: u64,
    self_id: String,
    username: String,
    device_id: Option<String>,
    revoked: AtomicBool,
    pc: Arc<RTCPeerConnection>,
    events_dc: RwLock<Option<Arc<RTCDataChannel>>>,
    slots: Vec<Arc<VoiceSlot>>,
}

impl PeerRtcSession {
    async fn snapshot_slot_map(&self) -> HashMap<String, Option<String>> {
        let mut out = HashMap::with_capacity(self.slots.len());
        for s in &self.slots {
            let assigned = s.assigned_peer.read().await.clone();
            out.insert(s.slot_id.clone(), assigned);
        }
        out
    }

    /// Finds the existing slot for `speaker_id`, or claims an empty / least-recently-active slot.
    /// Returns `(slot, mapping_changed)`.
    async fn resolve_or_assign_slot(&self, speaker_id: &str) -> Option<(Arc<VoiceSlot>, bool)> {
        if self.slots.is_empty() {
            return None;
        }
        // Fast path: already assigned
        for s in &self.slots {
            if s.assigned_peer.read().await.as_deref() == Some(speaker_id) {
                return Some((Arc::clone(s), false));
            }
        }
        // Assign first empty slot
        for s in &self.slots {
            let mut guard = s.assigned_peer.write().await;
            if guard.is_none() {
                *guard = Some(speaker_id.to_string());
                s.last_audio_ms.store(now_ms(), Ordering::Relaxed);
                return Some((Arc::clone(s), true));
            }
        }
        // All slots occupied (>8 speakers): reuse least-recently-speaking slot
        let oldest = self
            .slots
            .iter()
            .min_by_key(|s| s.last_audio_ms.load(Ordering::Relaxed))
            .cloned()?;
        {
            let mut guard = oldest.assigned_peer.write().await;
            *guard = Some(speaker_id.to_string());
        }
        oldest.last_audio_ms.store(now_ms(), Ordering::Relaxed);
        Some((oldest, true))
    }

    async fn release_speaker_slot(&self, departed_peer_id: &str) -> bool {
        let mut changed = false;
        for s in &self.slots {
            let mut guard = s.assigned_peer.write().await;
            if guard.as_deref() == Some(departed_peer_id) {
                *guard = None;
                changed = true;
            }
        }
        changed
    }
}

pub struct WebRtcSfu {
    api: API,
    udp_port: u16,
    world: Arc<WorldEngine>,
    sessions: RwLock<HashMap<String, Arc<PeerRtcSession>>>,
    active_count: AtomicUsize,
    next_nonce: AtomicU64,
}

impl WebRtcSfu {
    pub async fn bind_and_start(world: Arc<WorldEngine>) -> anyhow::Result<Arc<Self>> {
        let bind_addr: SocketAddr = format!(
            "{}:{}",
            world.config.webrtc_udp_bind_ip, world.config.webrtc_udp_port
        )
        .parse()?;

        let sock = socket2::Socket::new(
            if bind_addr.is_ipv4() {
                socket2::Domain::IPV4
            } else {
                socket2::Domain::IPV6
            },
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )?;
        sock.set_reuse_address(true)?;
        let _ = sock.set_recv_buffer_size(4 * 1024 * 1024);
        let _ = sock.set_send_buffer_size(4 * 1024 * 1024);
        sock.set_nonblocking(true)?;
        sock.bind(&bind_addr.into())?;
        let std_udp: std::net::UdpSocket = sock.into();
        let udp_socket = UdpSocket::from_std(std_udp)?;
        let actual_port = udp_socket.local_addr()?.port();
        info!(
            "🎙️ [WebRTC SFU] Single-port zero-renegotiation UDP Voice + State listening on {}:{}/udp",
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

        // Exclude container virtual bridge interfaces (`docker0`, `br-*`, `veth*`)
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
            active_count: AtomicUsize::new(0),
            next_nonce: AtomicU64::new(1),
        });

        // Bridge incoming Native Raw UDP Opus packets into browser pre-allocated voice slots!
        let bridge_sfu = Arc::clone(&sfu);
        let bridge_world = Arc::clone(&world);
        let mut opus_rx = world.opus_bridge_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(frame) = opus_rx.recv().await {
                if !frame.from_native_udp {
                    continue;
                }
                if !bridge_world.is_voice_allowed_for(&frame.from_id).await {
                    continue;
                }
                bridge_sfu
                    .fanout_opus_payload(&frame.from_id, frame.timestamp, &frame.payload)
                    .await;
            }
        });

        // Release voice slots cleanly whenever a player leaves the world
        let leave_sfu = Arc::clone(&sfu);
        let mut ev_rx = world.udp_event_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(ev) = ev_rx.recv().await {
                if let UdpWorldEvent::PlayerLeave { id, .. } = ev {
                    leave_sfu.release_departed_speaker(&id).await;
                }
            }
        });

        // Immediately close WebRTC UDP PeerConnections on session revocation / password rollover
        let revoke_sfu = Arc::clone(&sfu);
        let mut revoke_rx = world.session_revoke_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(sig) = revoke_rx.recv().await {
                let matching: Vec<Arc<PeerRtcSession>> = {
                    let map = revoke_sfu.sessions.read().await;
                    map.values()
                        .filter(|s| sig.matches(&s.self_id, &s.username, s.device_id.as_deref()))
                        .cloned()
                        .collect()
                };
                let notice = json!({
                    "type": "session_revoked",
                    "reason": sig.reason()
                })
                .to_string();
                for sess in matching {
                    sess.revoked.store(true, Ordering::Release);
                    if let Some(dc) = sess.events_dc.read().await.clone() {
                        let _ = dc.send_text(notice.clone()).await;
                    }
                    revoke_sfu
                        .cleanup_session_matching(&sess.self_id, Some(sess.session_nonce))
                        .await;
                }
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

    async fn emit_slot_map_for(&self, sess: &Arc<PeerRtcSession>) {
        let slots = sess.snapshot_slot_map().await;
        let ev = UdpWorldEvent::VoiceSlotMap {
            target_self_id: sess.self_id.clone(),
            slots,
        };
        if let Ok(txt) = serde_json::to_string(&ev) {
            self.world
                .send_peer_targeted_json(&sess.self_id, txt.clone())
                .await;
            if let Some(dc) = sess.events_dc.read().await.clone() {
                let _ = dc.send_text(txt).await;
            }
        }
    }

    async fn release_departed_speaker(&self, departed_id: &str) {
        let peers: Vec<Arc<PeerRtcSession>> =
            self.sessions.read().await.values().cloned().collect();
        for sess in peers {
            if sess.release_speaker_slot(departed_id).await {
                self.emit_slot_map_for(&sess).await;
            }
        }
    }

    async fn fanout_opus_payload(&self, from_id: &str, timestamp: u32, payload: &Bytes) {
        let listeners: Vec<Arc<PeerRtcSession>> = self
            .sessions
            .read()
            .await
            .values()
            .filter(|s| s.self_id != from_id)
            .cloned()
            .collect();

        let now = now_ms();
        for listener in listeners {
            let Some((slot, changed)) = listener.resolve_or_assign_slot(from_id).await else {
                continue;
            };
            if changed {
                self.emit_slot_map_for(&listener).await;
            }
            slot.last_audio_ms.store(now, Ordering::Relaxed);
            let next_seq = slot.seq.fetch_add(1, Ordering::Relaxed);
            let rtp_pkt = webrtc::rtp::packet::Packet {
                header: webrtc::rtp::header::Header {
                    version: 2,
                    payload_type: 111,
                    sequence_number: next_seq,
                    timestamp,
                    ssrc: 0xE11A_1000,
                    ..Default::default()
                },
                payload: payload.clone(),
            };
            let _ = slot.track.write_rtp(&rtp_pkt).await;
        }
    }

    /// Counts how many audio media sections in the browser's SDP offer accept downlink audio
    /// (`a=recvonly` or `a=sendrecv`) so we pre-bind that exact count of `Sendonly` slot tracks
    /// (up to `MAX_VOICE_SLOTS = 8`) with zero unmatched transceivers.
    fn count_offered_downlink_audio_slots(offer_sdp: &str) -> usize {
        let mut in_audio = false;
        let mut current_dir = "sendrecv";
        let mut recv_slots = 0usize;

        for raw_line in offer_sdp.lines() {
            let line = raw_line.trim();
            if line.starts_with("m=") {
                if in_audio && (current_dir == "recvonly" || current_dir == "sendrecv") {
                    recv_slots += 1;
                }
                in_audio = line.starts_with("m=audio ");
                current_dir = "sendrecv";
            } else if in_audio {
                if let Some(dir) = line.strip_prefix("a=") {
                    if matches!(dir, "sendonly" | "recvonly" | "sendrecv" | "inactive") {
                        current_dir = dir;
                    }
                }
            }
        }
        if in_audio && (current_dir == "recvonly" || current_dir == "sendrecv") {
            recv_slots += 1;
        }

        recv_slots.min(MAX_VOICE_SLOTS)
    }

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

    /// Cleans up a specific `PeerRtcSession` (matching `expected_nonce` if `Some` so an older
    /// closing PeerConnection can never tear down a newly replaced session!).
    /// Never evicts the player from `WorldEngine::players`—presence belongs to the WebSocket Spine!
    pub async fn cleanup_session_matching(&self, self_id: &str, expected_nonce: Option<u64>) {
        let removed_sess = {
            let mut map = self.sessions.write().await;
            let should_remove = match (map.get(self_id), expected_nonce) {
                (Some(existing), Some(nonce)) => existing.session_nonce == nonce,
                (Some(_), None) => true,
                (None, _) => false,
            };
            let s = if should_remove {
                map.remove(self_id)
            } else {
                None
            };
            self.active_count.store(map.len(), Ordering::Relaxed);
            s
        };

        if let Some(s) = removed_sess {
            s.revoked.store(true, Ordering::Release);
            self.world.set_udp_fastlane_active(self_id, false).await;
            let _ = s.pc.close().await;
        }
    }

    pub async fn cleanup_session(&self, self_id: &str) {
        self.cleanup_session_matching(self_id, None).await;
    }

    pub async fn cleanup_sessions_for_username(&self, username: &str) -> usize {
        let target = username.trim().to_lowercase();
        let matching: Vec<(String, u64)> = {
            let map = self.sessions.read().await;
            map.values()
                .filter(|s| s.username.eq_ignore_ascii_case(&target))
                .map(|s| {
                    s.revoked.store(true, Ordering::Release);
                    (s.self_id.clone(), s.session_nonce)
                })
                .collect()
        };
        let count = matching.len();
        for (sid, nonce) in matching {
            self.cleanup_session_matching(&sid, Some(nonce)).await;
        }
        count
    }

    pub async fn cleanup_all_sessions(&self) -> usize {
        let all: Vec<(String, u64)> = {
            let map = self.sessions.read().await;
            map.values()
                .map(|s| {
                    s.revoked.store(true, Ordering::Release);
                    (s.self_id.clone(), s.session_nonce)
                })
                .collect()
        };
        let count = all.len();
        for (sid, nonce) in all {
            self.cleanup_session_matching(&sid, Some(nonce)).await;
        }
        count
    }

    /// Accepts a browser's single-shot SDP Offer (`POST /api/ella-world/rtc/offer`), binding:
    /// - 1 inbound Opus RTP voice track -> fanned out over UDP (`44444/udp`) to all listeners' pre-allocated slots
    /// - Up to 8 pre-allocated outbound `Sendonly` Opus voice slots (`ella_slot_0`..`ella_slot_7`) -> ZERO renegotiation!
    /// - `ella-udp-state` unreliable binary DataChannel (`20 Hz` UDP movement/emotes/RTT fast-lane)
    /// - `ella-reliable-events` reliable UDP DataChannel
    pub async fn accept_browser_offer(
        self: &Arc<Self>,
        self_id: String,
        username: String,
        device_id: Option<String>,
        offer_sdp: String,
    ) -> anyhow::Result<String> {
        let _ = self.world.ensure_player_online(&self_id, &username).await;
        self.cleanup_session_matching(&self_id, None).await;

        let pc = Arc::new(
            self.api
                .new_peer_connection(RTCConfiguration::default())
                .await?,
        );

        let slot_count = Self::count_offered_downlink_audio_slots(&offer_sdp);
        let mut slots = Vec::with_capacity(slot_count);

        for idx in 0..slot_count {
            let slot_id = format!("ella_slot_{idx}");
            let local_track = Arc::new(TrackLocalStaticRTP::new(
                RTCRtpCodecCapability {
                    mime_type: MIME_TYPE_OPUS.to_owned(),
                    clock_rate: 48_000,
                    channels: 2,
                    sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                    rtcp_feedback: vec![],
                },
                slot_id.clone(),
                slot_id.clone(),
            ));

            let transceiver = pc
                .add_transceiver_from_track(
                    Arc::clone(&local_track) as Arc<dyn TrackLocal + Send + Sync>,
                    Some(RTCRtpTransceiverInit {
                        direction: RTCRtpTransceiverDirection::Sendonly,
                        send_encodings: Vec::new(),
                    }),
                )
                .await?;

            // Drain RTCP receiver reports so sender buffers never stall
            let rtcp_sender = transceiver.sender().await;
            tokio::spawn(async move {
                let mut rtcp_buf = vec![0u8; 1500];
                while rtcp_sender.read(&mut rtcp_buf).await.is_ok() {}
            });

            slots.push(Arc::new(VoiceSlot {
                slot_id,
                track: local_track,
                seq: AtomicU16::new(1),
                last_audio_ms: AtomicU64::new(0),
                assigned_peer: RwLock::new(None),
            }));
        }

        // Pre-assign slots for currently online peers so mapping is ready before first syllable!
        let existing_peer_ids: Vec<String> = self
            .world
            .active_player_ids()
            .await
            .into_iter()
            .filter(|pid| pid != &self_id)
            .take(slot_count)
            .collect();
        for (slot, pid) in slots.iter().zip(existing_peer_ids) {
            *slot.assigned_peer.write().await = Some(pid);
            slot.last_audio_ms.store(now_ms(), Ordering::Relaxed);
        }

        let session_nonce = self.next_nonce.fetch_add(1, Ordering::Relaxed);
        let session = Arc::new(PeerRtcSession {
            session_nonce,
            self_id: self_id.clone(),
            username: username.clone(),
            device_id,
            revoked: AtomicBool::new(false),
            pc: Arc::clone(&pc),
            events_dc: RwLock::new(None),
            slots,
        });

        {
            let mut map = self.sessions.write().await;
            map.insert(self_id.clone(), Arc::clone(&session));
            self.active_count.store(map.len(), Ordering::Relaxed);
        }

        // Also pre-assign this new joiner into an open voice slot on already-connected peers
        let other_sessions: Vec<Arc<PeerRtcSession>> = self
            .sessions
            .read()
            .await
            .values()
            .filter(|s| s.self_id != self_id)
            .cloned()
            .collect();
        for other in other_sessions {
            if let Some((_, true)) = other.resolve_or_assign_slot(&self_id).await {
                self.emit_slot_map_for(&other).await;
            }
        }

        // 1. Lifecycle monitor — ONLY toggles `udp_fastlane_active`; NEVER kicks player from world!
        let state_sfu = Arc::clone(self);
        let state_self_id = self_id.clone();
        pc.on_peer_connection_state_change(Box::new(move |st: RTCPeerConnectionState| {
            let sfu_c = Arc::clone(&state_sfu);
            let sid = state_self_id.clone();
            Box::pin(async move {
                if matches!(
                    st,
                    RTCPeerConnectionState::Failed | RTCPeerConnectionState::Closed
                ) {
                    sfu_c
                        .cleanup_session_matching(&sid, Some(session_nonce))
                        .await;
                } else if st == RTCPeerConnectionState::Disconnected {
                    sfu_c.world.set_udp_fastlane_active(&sid, false).await;
                }
            })
        }));

        // 2. Incoming Opus voice RTP stream over UDP -> fan out into listeners' pre-allocated slots!
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
                    "🎤 [WebRTC SFU] Receiving zero-renegotiation UDP Opus stream from {}",
                    sid
                );
                while let Ok((rtp_pkt, _attrs)) = remote_track.read_rtp().await {
                    if !world_c.is_voice_allowed_for(&sid).await {
                        continue;
                    }
                    let _ = world_c.opus_bridge_tx.send(BridgedOpusFrame {
                        from_id: sid.clone(),
                        from_native_udp: false,
                        seq: rtp_pkt.header.sequence_number,
                        timestamp: rtp_pkt.header.timestamp,
                        payload: rtp_pkt.payload.clone(),
                    });
                    sfu_c
                        .fanout_opus_payload(&sid, rtp_pkt.header.timestamp, &rtp_pkt.payload)
                        .await;
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
        if let Err(err) = pc.set_remote_description(offer).await {
            warn!("Rejected invalid SDP offer from {}: {}", self_id, err);
            self.cleanup_session_matching(&self_id, Some(session_nonce))
                .await;
            return Err(err.into());
        }

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
        let in_world = Arc::clone(&world);
        let in_dc = Arc::clone(&dc);
        let in_sess = Arc::clone(&sess);
        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            let w = Arc::clone(&in_world);
            let d = Arc::clone(&in_dc);
            let s = Arc::clone(&in_sess);
            Box::pin(async move {
                if s.revoked.load(Ordering::Acquire) {
                    return;
                }
                let raw = msg.data.as_ref();
                if raw.is_empty() {
                    return;
                }
                match raw[0] {
                    OP_C2S_STATE => {
                        if let Some(st) = decode_c2s_state(raw) {
                            w.apply_c2s_state_ext(&s.self_id, Some(&s.username), true, st)
                                .await;
                        }
                    }
                    OP_C2S_PING if raw.len() >= 5 => {
                        let client_ts = u32::from_le_bytes([raw[1], raw[2], raw[3], raw[4]]);
                        w.touch_player_udp(&s.self_id).await;
                        let pong = encode_pong(client_ts, w.current_tick());
                        let _ = d.send(&pong).await;
                    }
                    _ => {}
                }
            })
        }));

        // Outbound 20Hz S2C_WORLD_STATE (0x11) binary datagram pump over UDP
        let out_dc = Arc::clone(&dc);
        let out_world = Arc::clone(&world);
        let out_sess = Arc::clone(&sess);
        dc.on_open(Box::new(move || {
            let d = Arc::clone(&out_dc);
            let w = Arc::clone(&out_world);
            let s = Arc::clone(&out_sess);
            Box::pin(async move {
                if s.revoked.load(Ordering::Acquire) {
                    return;
                }
                w.set_udp_fastlane_active(&s.self_id, true).await;
                tokio::spawn(async move {
                    let mut tick_rx = w.pre_encoded_tick_tx.subscribe();
                    while tick_rx.changed().await.is_ok() {
                        if s.revoked.load(Ordering::Acquire) {
                            w.set_udp_fastlane_active(&s.self_id, false).await;
                            break;
                        }
                        let pre = tick_rx.borrow_and_update().clone();
                        let snap = crate::world::WorldEngine::encode_pre_tick_scheduled_chunk_for(
                            &pre, &s.self_id,
                        );
                        if d.send(&snap).await.is_err() {
                            w.set_udp_fastlane_active(&s.self_id, false).await;
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
        let msg_world = Arc::clone(&world);
        let msg_sess = Arc::clone(&sess);
        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            let w = Arc::clone(&msg_world);
            let s = Arc::clone(&msg_sess);
            Box::pin(async move {
                if s.revoked.load(Ordering::Acquire) {
                    return;
                }
                let Ok(txt) = std::str::from_utf8(&msg.data) else {
                    return;
                };
                let Ok(cmd) = serde_json::from_str::<serde_json::Value>(txt) else {
                    return;
                };
                handle_client_event_command(&w, &s.self_id, &s.username, &cmd).await;
            })
        }));

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

                // 1. Send initial Roster + current VoiceSlotMap immediately
                let others: Vec<_> = w
                    .active_players_list()
                    .await
                    .into_iter()
                    .filter(|p| p.id != s.self_id)
                    .collect();
                let bver = w.blocks_data.read().await.version;
                let roster_ev = UdpWorldEvent::Roster {
                    state_seq: w.current_state_seq(),
                    server_time_ms: now_ms(),
                    players: others,
                    blocks_version: bver,
                };
                if let Ok(json_txt) = serde_json::to_string(&roster_ev) {
                    let _ = d.send_text(json_txt).await;
                }
                sfu_c.emit_slot_map_for(&s).await;

                // 2. Forward world events if the client has no active WebSocket spine (e.g., UDP-only test peer)
                let mut ev_rx = w.udp_event_tx.subscribe();
                let pump_dc = Arc::clone(&d);
                let pump_world = Arc::clone(&w);
                let self_id_filter = s.self_id.clone();
                tokio::spawn(async move {
                    while let Ok(ev) = ev_rx.recv().await {
                        match &ev {
                            UdpWorldEvent::VoiceSlotMap { target_self_id, .. } => {
                                if target_self_id != &self_id_filter {
                                    continue;
                                }
                            }
                            UdpWorldEvent::PlayerJoin { player, .. } => {
                                if player.id == self_id_filter {
                                    continue;
                                }
                                // Avoid duplicate JSON event push when WS spine is already connected
                                let ws_up = pump_world
                                    .players
                                    .read()
                                    .await
                                    .get(&self_id_filter)
                                    .map(|p| p.ws_connected)
                                    .unwrap_or(false);
                                if ws_up {
                                    continue;
                                }
                            }
                            UdpWorldEvent::DungeonMobs { .. }
                            | UdpWorldEvent::DungeonSync { .. } => {
                                let (ws_up, zone) = {
                                    let map = pump_world.players.read().await;
                                    map.get(&self_id_filter)
                                        .map(|p| (p.ws_connected, p.zone))
                                        .unwrap_or((false, crate::udp_wire::ZONE_PLAZA))
                                };
                                if ws_up || zone != crate::udp_wire::ZONE_DUNGEON {
                                    continue;
                                }
                            }
                            UdpWorldEvent::PrisonSync { .. } => {
                                let (ws_up, zone) = {
                                    let map = pump_world.players.read().await;
                                    map.get(&self_id_filter)
                                        .map(|p| (p.ws_connected, p.zone))
                                        .unwrap_or((false, crate::udp_wire::ZONE_PLAZA))
                                };
                                if ws_up || zone != crate::udp_wire::ZONE_PRISON {
                                    continue;
                                }
                            }
                            UdpWorldEvent::RayGunBeam {
                                zone: beam_zone, ..
                            } => {
                                let (ws_up, zone) = {
                                    let map = pump_world.players.read().await;
                                    map.get(&self_id_filter)
                                        .map(|p| (p.ws_connected, p.zone))
                                        .unwrap_or((false, crate::udp_wire::ZONE_PLAZA))
                                };
                                if ws_up || zone != *beam_zone {
                                    continue;
                                }
                            }
                            _ => {
                                let ws_up = pump_world
                                    .players
                                    .read()
                                    .await
                                    .get(&self_id_filter)
                                    .map(|p| p.ws_connected)
                                    .unwrap_or(false);
                                if ws_up {
                                    continue;
                                }
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

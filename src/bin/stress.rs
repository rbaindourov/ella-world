//! Multi-Stage Classroom Load & Stress Simulator (`ella-stress`)
//!
//! Spawns cumulative virtual student bots (`10 -> 25 -> 50 -> 100`) speaking exact production
//! ELLA World protocols:
//!   1. `POST /api/ella-world/login` (over HTTPS/TLS or HTTP)
//!   2. Persistent `wss://.../api/ella-world/ws` spine (`OP_C2S_STATE`, `udp_fastlane`, chats/blocks)
//!   3. WebRTC Single-Port UDP (`4444/udp`) SFU session (`POST /api/ella-world/rtc/offer` +
//!      `ella-udp-state` 20 Hz unreliable DataChannel + 8 downlink voice slots + active Opus senders)

use anyhow::{bail, Context};
use bytes::Bytes;
use ella_world::udp_wire::{
    decode_s2c_world_state, encode_c2s_state, DecodedC2SState, OP_S2C_PONG, OP_S2C_WORLD_STATE,
};
use futures_util::{SinkExt, StreamExt};
use http_body_util::{BodyExt, Full};
use hyper::client::conn::http1;
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio_rustls::rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use tokio_rustls::rustls::crypto::ring;
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use tokio_rustls::rustls::{
    ClientConfig, DigitallySignedStruct, Error as RustlsError, SignatureScheme,
};
use tokio_rustls::TlsConnector;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::{APIBuilder, API};
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice::network_type::NetworkType;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::{
    RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType,
};
use webrtc::rtp_transceiver::rtp_transceiver_direction::RTCRtpTransceiverDirection;
use webrtc::rtp_transceiver::RTCRtpTransceiverInit;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};

#[derive(Clone, Debug)]
struct StressConfig {
    base_url: String,
    insecure_tls: bool,
    stages: Vec<usize>,
    hold_secs: u64,
    ramp_window_ms: u64,
    mode: String, // "hybrid" | "ws"
    active_talkers: usize,
    username_prefix: String,
    password: String,
    shared_username: Option<String>,
    json_report: Option<String>,
}

impl StressConfig {
    fn from_args() -> Self {
        let mut cfg = Self {
            base_url: "http://127.0.0.1:8097".into(),
            insecure_tls: false,
            stages: vec![10, 25, 50, 100],
            hold_secs: 20,
            ramp_window_ms: 3000,
            mode: "hybrid".into(),
            active_talkers: 3,
            username_prefix: "stress_".into(),
            password: "stress-classroom-2026".into(),
            shared_username: None,
            json_report: None,
        };

        let args: Vec<String> = env::args().collect();
        let mut i = 1;
        while i < args.len() {
            let k = args[i].as_str();
            let next = args.get(i + 1).cloned();
            match (k, next) {
                ("--base-url", Some(v)) => {
                    cfg.base_url = v.trim_end_matches('/').to_string();
                    i += 2;
                }
                ("--insecure-tls", _) => {
                    cfg.insecure_tls = true;
                    i += 1;
                }
                ("--stages", Some(v)) => {
                    let parsed: Vec<usize> = v
                        .split(',')
                        .filter_map(|s| s.trim().parse::<usize>().ok())
                        .filter(|&n| n > 0)
                        .collect();
                    if !parsed.is_empty() {
                        cfg.stages = parsed;
                    }
                    i += 2;
                }
                ("--hold-secs", Some(v)) => {
                    if let Ok(n) = v.parse() {
                        cfg.hold_secs = n;
                    }
                    i += 2;
                }
                ("--ramp-window-ms", Some(v)) => {
                    if let Ok(n) = v.parse() {
                        cfg.ramp_window_ms = n;
                    }
                    i += 2;
                }
                ("--mode", Some(v)) => {
                    cfg.mode = v;
                    i += 2;
                }
                ("--active-talkers", Some(v)) => {
                    if let Ok(n) = v.parse() {
                        cfg.active_talkers = n;
                    }
                    i += 2;
                }
                ("--username-prefix", Some(v)) => {
                    cfg.username_prefix = v;
                    i += 2;
                }
                ("--password", Some(v)) => {
                    cfg.password = v;
                    i += 2;
                }
                ("--shared-username", Some(v)) => {
                    cfg.shared_username = Some(v);
                    i += 2;
                }
                ("--json-report", Some(v)) => {
                    cfg.json_report = Some(v);
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }
        cfg
    }
}

#[derive(Debug)]
struct NoCertVerifier;

impl ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn build_tls_client_config(insecure: bool) -> Arc<ClientConfig> {
    let provider = Arc::new(ring::default_provider());
    if insecure {
        Arc::new(
            ClientConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .expect("tls versions")
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
                .with_no_client_auth(),
        )
    } else {
        let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        Arc::new(
            ClientConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .expect("tls versions")
                .with_root_certificates(root_store)
                .with_no_client_auth(),
        )
    }
}

#[derive(Clone)]
struct HttpTarget {
    is_tls: bool,
    host: String,
    port: u16,
    host_header: String,
    tls_connector: TlsConnector,
}

impl HttpTarget {
    fn parse(base_url: &str, tls_cfg: Arc<ClientConfig>) -> anyhow::Result<Self> {
        let (is_tls, rest) = if let Some(r) = base_url.strip_prefix("https://") {
            (true, r)
        } else if let Some(r) = base_url.strip_prefix("http://") {
            (false, r)
        } else {
            bail!("Unsupported base_url scheme: {base_url}");
        };
        let authority = rest.split('/').next().unwrap_or(rest);
        let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
            (h.to_string(), p.parse::<u16>()?)
        } else {
            (authority.to_string(), if is_tls { 443 } else { 80 })
        };
        Ok(Self {
            is_tls,
            host: host.clone(),
            port,
            host_header: authority.to_string(),
            tls_connector: TlsConnector::from(tls_cfg),
        })
    }

    async fn json_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> anyhow::Result<(u16, Value)> {
        let tcp = TcpStream::connect((self.host.as_str(), self.port)).await?;
        let _ = tcp.set_nodelay(true);
        let payload = body
            .map(serde_json::to_vec)
            .transpose()?
            .unwrap_or_default();

        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("Host", &self.host_header)
            .header("Accept-Encoding", "identity")
            .header("Connection", "close");
        if !payload.is_empty() {
            builder = builder
                .header("Content-Type", "application/json")
                .header("Content-Length", payload.len().to_string());
        }
        let req = builder.body(Full::new(Bytes::from(payload)))?;

        if self.is_tls {
            let server_name = ServerName::try_from(self.host.clone())?;
            let tls_stream = self.tls_connector.connect(server_name, tcp).await?;
            let (mut sender, conn) = http1::handshake(TokioIo::new(tls_stream)).await?;
            tokio::spawn(async move {
                let _ = conn.await;
            });
            let resp = sender.send_request(req).await?;
            let status = resp.status().as_u16();
            let bytes = resp.into_body().collect().await?.to_bytes();
            let json_val = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
            Ok((status, json_val))
        } else {
            let (mut sender, conn) = http1::handshake(TokioIo::new(tcp)).await?;
            tokio::spawn(async move {
                let _ = conn.await;
            });
            let resp = sender.send_request(req).await?;
            let status = resp.status().as_u16();
            let bytes = resp.into_body().collect().await?.to_bytes();
            let json_val = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
            Ok((status, json_val))
        }
    }

    fn ws_url(&self, token: &str) -> String {
        let scheme = if self.is_tls { "wss" } else { "ws" };
        format!(
            "{scheme}://{}/api/ella-world/ws?token={token}",
            self.host_header
        )
    }
}

struct MetricsCollector {
    recording: AtomicBool,
    login_latencies_ms: Mutex<Vec<f64>>,
    rtc_offer_latencies_ms: Mutex<Vec<f64>>,
    state_rtt_ms: Mutex<Vec<f64>>,
    login_ok: AtomicUsize,
    login_err: AtomicUsize,
    ws_connected: AtomicUsize,
    ws_errors: AtomicUsize,
    ws_disconnects: AtomicUsize,
    udp_dc_open: AtomicUsize,
    rtc_errors: AtomicUsize,
    snapshots_via_udp: AtomicU64,
    snapshots_via_ws: AtomicU64,
    audio_rtp_rx: AtomicU64,
    chats_sent: AtomicU64,
    blocks_sent: AtomicU64,
    seen_peers_by_bot: RwLock<HashMap<usize, HashSet<String>>>,
}

impl MetricsCollector {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            recording: AtomicBool::new(false),
            login_latencies_ms: Mutex::new(Vec::new()),
            rtc_offer_latencies_ms: Mutex::new(Vec::new()),
            state_rtt_ms: Mutex::new(Vec::new()),
            login_ok: AtomicUsize::new(0),
            login_err: AtomicUsize::new(0),
            ws_connected: AtomicUsize::new(0),
            ws_errors: AtomicUsize::new(0),
            ws_disconnects: AtomicUsize::new(0),
            udp_dc_open: AtomicUsize::new(0),
            rtc_errors: AtomicUsize::new(0),
            snapshots_via_udp: AtomicU64::new(0),
            snapshots_via_ws: AtomicU64::new(0),
            audio_rtp_rx: AtomicU64::new(0),
            chats_sent: AtomicU64::new(0),
            blocks_sent: AtomicU64::new(0),
            seen_peers_by_bot: RwLock::new(HashMap::new()),
        })
    }

    async fn reset_steady_window(&self) {
        self.recording.store(false, Ordering::SeqCst);
        self.state_rtt_ms.lock().await.clear();
        self.snapshots_via_udp.store(0, Ordering::Relaxed);
        self.snapshots_via_ws.store(0, Ordering::Relaxed);
        self.audio_rtp_rx.store(0, Ordering::Relaxed);
        self.chats_sent.store(0, Ordering::Relaxed);
        self.blocks_sent.store(0, Ordering::Relaxed);
        self.ws_disconnects.store(0, Ordering::Relaxed);
        self.seen_peers_by_bot.write().await.clear();
        self.recording.store(true, Ordering::SeqCst);
    }

    async fn record_rtt(&self, bot_idx: usize, rtt_ms: f64, via_udp: bool, frame_ids: &[String]) {
        if !self.recording.load(Ordering::Relaxed) {
            return;
        }
        if via_udp {
            self.snapshots_via_udp.fetch_add(1, Ordering::Relaxed);
        } else {
            self.snapshots_via_ws.fetch_add(1, Ordering::Relaxed);
        }
        if rtt_ms > 0.0 && rtt_ms < 5000.0 {
            let mut vec = self.state_rtt_ms.lock().await;
            if vec.len() < 100_000 {
                vec.push(rtt_ms);
            }
        }
        if !frame_ids.is_empty() {
            let need_insert = {
                let map = self.seen_peers_by_bot.read().await;
                match map.get(&bot_idx) {
                    Some(set) => frame_ids.iter().any(|id| !set.contains(id)),
                    None => true,
                }
            };
            if need_insert {
                let mut map = self.seen_peers_by_bot.write().await;
                let set = map.entry(bot_idx).or_default();
                for id in frame_ids {
                    set.insert(id.clone());
                }
            }
        }
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StageReport {
    target_players: usize,
    login_ok: usize,
    login_err: usize,
    ws_connected: usize,
    ws_disconnects_in_hold: usize,
    udp_dc_open: usize,
    rtc_errors: usize,
    login_p95_ms: f64,
    rtc_offer_p95_ms: f64,
    snapshot_hz_per_bot: f64,
    udp_share_pct: f64,
    state_rtt_p50_ms: f64,
    state_rtt_p95_ms: f64,
    state_rtt_max_ms: f64,
    avg_visible_peers: f64,
    max_visible_peers: usize,
    visibility_pct: f64,
    audio_rtp_packets_rx: u64,
    server_tick_hz: f64,
    server_active_players: u64,
    server_active_ws: u64,
    server_active_webrtc: u64,
    duration_secs: f64,
}

fn build_client_webrtc_api() -> anyhow::Result<API> {
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
    setting_engine.set_interface_filter(Box::new(|iface_name: &str| {
        !iface_name.starts_with("docker")
            && !iface_name.starts_with("br-")
            && !iface_name.starts_with("veth")
            && !iface_name.starts_with("virbr")
    }));

    Ok(APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .with_setting_engine(setting_engine)
        .build())
}

#[allow(clippy::too_many_arguments)]
async fn run_virtual_student(
    bot_idx: usize,
    cfg: StressConfig,
    http: HttpTarget,
    ws_tls_cfg: Arc<ClientConfig>,
    webrtc_api: Arc<API>,
    metrics: Arc<MetricsCollector>,
    stop_notify: Arc<Notify>,
    ready_signal: Arc<tokio::sync::Barrier>,
) {
    let username = cfg
        .shared_username
        .clone()
        .unwrap_or_else(|| format!("{}{:03}", cfg.username_prefix, bot_idx + 1));
    let client_id = format!("bot{:03}", bot_idx + 1);

    // 1. Login
    let t_login = Instant::now();
    let login_res = http
        .json_request(
            "POST",
            "/api/ella-world/login",
            Some(&json!({
                "username": username,
                "password": cfg.password,
                "clientId": client_id
            })),
        )
        .await;

    let (token, self_id) = match login_res {
        Ok((200, val)) => {
            let tok = val["token"].as_str().unwrap_or("").to_string();
            let sid = val["selfId"].as_str().unwrap_or("").to_string();
            if tok.is_empty() || sid.is_empty() {
                metrics.login_err.fetch_add(1, Ordering::Relaxed);
                let _ = ready_signal.wait().await;
                return;
            }
            metrics.login_ok.fetch_add(1, Ordering::Relaxed);
            metrics
                .login_latencies_ms
                .lock()
                .await
                .push(t_login.elapsed().as_secs_f64() * 1000.0);
            (tok, sid)
        }
        Err(err) => {
            eprintln!("[bot {bot_idx}] login error: {err}");
            metrics.login_err.fetch_add(1, Ordering::Relaxed);
            let _ = ready_signal.wait().await;
            return;
        }
        Ok((st, val)) => {
            eprintln!("[bot {bot_idx}] login HTTP {st}: {val}");
            metrics.login_err.fetch_add(1, Ordering::Relaxed);
            let _ = ready_signal.wait().await;
            return;
        }
    };

    // 2. Connect persistent TLS WebSocket Spine
    let ws_url = http.ws_url(&token);
    let ws_stream = match connect_async_tls_with_config(
        &ws_url,
        None,
        false,
        Some(Connector::Rustls(ws_tls_cfg)),
    )
    .await
    {
        Ok((ws, _)) => {
            metrics.ws_connected.fetch_add(1, Ordering::Relaxed);
            ws
        }
        Err(err) => {
            if bot_idx < 3 {
                eprintln!("[bot {bot_idx}] WS connect error: {err}");
            }
            metrics.ws_errors.fetch_add(1, Ordering::Relaxed);
            let _ = ready_signal.wait().await;
            return;
        }
    };

    let (ws_sink, mut ws_rx) = ws_stream.split();
    let ws_sink = Arc::new(Mutex::new(ws_sink));
    let clock_start = Instant::now();
    let udp_healthy = Arc::new(AtomicBool::new(false));
    let last_udp_rx_ms = Arc::new(AtomicU64::new(0));

    // WS reader task
    let ws_metrics = Arc::clone(&metrics);
    let ws_self_id = self_id.clone();
    let ws_clock = clock_start;
    let ws_rx_handle = tokio::spawn(async move {
        while let Some(item) = ws_rx.next().await {
            match item {
                Ok(WsMessage::Binary(raw)) if !raw.is_empty() => {
                    if raw[0] == OP_S2C_WORLD_STATE {
                        if let Some(snap) = decode_s2c_world_state(&raw) {
                            let now_ms_rel = ws_clock.elapsed().as_millis() as u32;
                            let rtt = if snap.echo_client_time_ms > 0
                                && now_ms_rel >= snap.echo_client_time_ms
                            {
                                f64::from(now_ms_rel - snap.echo_client_time_ms)
                            } else {
                                0.0
                            };
                            let ids: Vec<String> = snap
                                .players
                                .iter()
                                .filter(|p| p.id != ws_self_id)
                                .map(|p| p.id.clone())
                                .collect();
                            ws_metrics.record_rtt(bot_idx, rtt, false, &ids).await;
                        }
                    } else if raw[0] == OP_S2C_PONG {
                        // handled
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        ws_metrics.ws_disconnects.fetch_add(1, Ordering::Relaxed);
    });

    // 3. Optional WebRTC SFU UDP Connection (`hybrid` mode)
    let mut state_dc_holder = None;
    let mut mic_track_holder = None;
    let mut pc_holder = None;

    if cfg.mode == "hybrid" {
        match webrtc_api
            .new_peer_connection(RTCConfiguration::default())
            .await
        {
            Ok(pc_raw) => {
                let pc = Arc::new(pc_raw);
                let mic_track = Arc::new(TrackLocalStaticRTP::new(
                    RTCRtpCodecCapability {
                        mime_type: MIME_TYPE_OPUS.to_owned(),
                        clock_rate: 48_000,
                        channels: 2,
                        sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                        rtcp_feedback: vec![],
                    },
                    format!("mic_{bot_idx}"),
                    format!("stream_{bot_idx}"),
                ));
                let _ = pc
                    .add_transceiver_from_track(
                        Arc::clone(&mic_track) as Arc<dyn TrackLocal + Send + Sync>,
                        Some(RTCRtpTransceiverInit {
                            direction: RTCRtpTransceiverDirection::Sendonly,
                            send_encodings: Vec::new(),
                        }),
                    )
                    .await;

                for _slot in 0..8 {
                    let _ = pc
                        .add_transceiver_from_kind(
                            RTPCodecType::Audio,
                            Some(RTCRtpTransceiverInit {
                                direction: RTCRtpTransceiverDirection::Recvonly,
                                send_encodings: Vec::new(),
                            }),
                        )
                        .await;
                }

                let audio_m = Arc::clone(&metrics);
                pc.on_track(Box::new(move |track, _rx, _tr| {
                    let m = Arc::clone(&audio_m);
                    Box::pin(async move {
                        while let Ok((_pkt, _)) = track.read_rtp().await {
                            if m.recording.load(Ordering::Relaxed) {
                                m.audio_rtp_rx.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    })
                }));

                if let Ok(dc) = pc
                    .create_data_channel(
                        "ella-udp-state",
                        Some(RTCDataChannelInit {
                            ordered: Some(false),
                            max_retransmits: Some(0),
                            ..Default::default()
                        }),
                    )
                    .await
                {
                    let _ = pc.create_data_channel("ella-reliable-events", None).await;
                    let open_flag = Arc::clone(&udp_healthy);
                    let open_m = Arc::clone(&metrics);
                    let ws_notify_sink = Arc::clone(&ws_sink);
                    dc.on_open(Box::new(move || {
                        let f = Arc::clone(&open_flag);
                        let m = Arc::clone(&open_m);
                        let sink = Arc::clone(&ws_notify_sink);
                        Box::pin(async move {
                            f.store(true, Ordering::Relaxed);
                            m.udp_dc_open.fetch_add(1, Ordering::Relaxed);
                            let mut s = sink.lock().await;
                            let _ = s
                                .send(WsMessage::Text(
                                    json!({ "type": "udp_fastlane", "active": true }).to_string(),
                                ))
                                .await;
                        })
                    }));

                    let rx_m = Arc::clone(&metrics);
                    let rx_flag = Arc::clone(&last_udp_rx_ms);
                    let rx_sid = self_id.clone();
                    let rx_clock = clock_start;
                    dc.on_message(Box::new(move |msg: DataChannelMessage| {
                        let m = Arc::clone(&rx_m);
                        let rf = Arc::clone(&rx_flag);
                        let sid = rx_sid.clone();
                        let clk = rx_clock;
                        Box::pin(async move {
                            let raw = msg.data.as_ref();
                            if !raw.is_empty() && raw[0] == OP_S2C_WORLD_STATE {
                                if let Some(snap) = decode_s2c_world_state(raw) {
                                    let now_rel = clk.elapsed().as_millis() as u64;
                                    rf.store(now_rel, Ordering::Relaxed);
                                    let rtt = if snap.echo_client_time_ms > 0
                                        && (now_rel as u32) >= snap.echo_client_time_ms
                                    {
                                        f64::from((now_rel as u32) - snap.echo_client_time_ms)
                                    } else {
                                        0.0
                                    };
                                    let ids: Vec<String> = snap
                                        .players
                                        .iter()
                                        .filter(|p| p.id != sid)
                                        .map(|p| p.id.clone())
                                        .collect();
                                    m.record_rtt(bot_idx, rtt, true, &ids).await;
                                }
                            }
                        })
                    }));

                    let mut gather = pc.gathering_complete_promise().await;
                    if let Ok(offer) = pc.create_offer(None).await {
                        let _ = pc.set_local_description(offer).await;
                        let _ =
                            tokio::time::timeout(Duration::from_millis(1200), gather.recv()).await;
                        if let Some(local_sdp) = pc.local_description().await {
                            let t_rtc = Instant::now();
                            match http
                                .json_request(
                                    "POST",
                                    "/api/ella-world/rtc/offer",
                                    Some(&json!({ "token": token, "sdp": local_sdp.sdp })),
                                )
                                .await
                            {
                                Ok((200, ans)) => {
                                    if let Some(sdp_str) = ans["sdp"].as_str() {
                                        if let Ok(desc) =
                                            RTCSessionDescription::answer(sdp_str.into())
                                        {
                                            if pc.set_remote_description(desc).await.is_ok() {
                                                metrics
                                                    .rtc_offer_latencies_ms
                                                    .lock()
                                                    .await
                                                    .push(t_rtc.elapsed().as_secs_f64() * 1000.0);
                                            } else {
                                                metrics.rtc_errors.fetch_add(1, Ordering::Relaxed);
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    metrics.rtc_errors.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                    state_dc_holder = Some(dc);
                    mic_track_holder = Some(mic_track);
                    pc_holder = Some(pc);
                }
            }
            Err(_) => {
                metrics.rtc_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    // Signal orchestrator this bot finished joining
    let _ = ready_signal.wait().await;

    // 4. Optional active Opus speaker loop (20 ms synthetic Opus DTX packet)
    let voice_handle = if bot_idx < cfg.active_talkers {
        mic_track_holder.map(|track| {
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(20));
                let mut seq = 1u16;
                loop {
                    interval.tick().await;
                    let pkt = webrtc::rtp::packet::Packet {
                        header: webrtc::rtp::header::Header {
                            version: 2,
                            payload_type: 111,
                            sequence_number: seq,
                            timestamp: u32::from(seq) * 960,
                            ssrc: 0xEE00_0000 + bot_idx as u32,
                            ..Default::default()
                        },
                        payload: Bytes::from_static(&[0xF8, 0xFF, 0xFE, 0x01, 0x02, 0x03]),
                    };
                    if track.write_rtp(&pkt).await.is_err() {
                        break;
                    }
                    seq = seq.wrapping_add(1);
                }
            })
        })
    } else {
        None
    };

    // 5. Main 20 Hz movement + periodic chat loop
    let mut move_tick = tokio::time::interval(Duration::from_millis(50));
    move_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let seq_counter = AtomicU32::new(1);
    let angle_offset = (bot_idx as f32) * 0.43;
    let is_speaking = bot_idx < cfg.active_talkers;
    let mut tick_count = 0u64;

    loop {
        tokio::select! {
            _ = stop_notify.notified() => break,
            _ = move_tick.tick() => {
                tick_count += 1;
                let t = clock_start.elapsed().as_secs_f32() + angle_offset;
                let seq = seq_counter.fetch_add(1, Ordering::Relaxed);
                let client_ms = clock_start.elapsed().as_millis() as u32;
                let frame = encode_c2s_state(&DecodedC2SState {
                    seq,
                    client_time_ms: client_ms,
                    x: (t * 0.7).cos() * 14.0,
                    y: if tick_count.is_multiple_of(40) { 1.2 } else { 0.0 },
                    z: (t * 0.7).sin() * 14.0,
                    ry: t,
                    anim: "run".into(),
                    emote: None,
                    muted: false,
                    speaking: is_speaking,
                    zone: 0,
                });

                let mut sent_udp = false;
                if udp_healthy.load(Ordering::Relaxed) {
                    if let Some(dc) = &state_dc_holder {
                        if dc.send(&frame).await.is_ok() {
                            sent_udp = true;
                        }
                    }
                }
                if !sent_udp {
                    let mut sink = ws_sink.lock().await;
                    if sink.send(WsMessage::Binary(frame.to_vec())).await.is_err() {
                        break;
                    }
                }

                // Stagger realistic classroom chat / block edits across bots
                if tick_count % 300 == ((bot_idx as u64 * 17) % 300) && metrics.recording.load(Ordering::Relaxed) {
                    let http_c = http.clone();
                    let tok_c = token.clone();
                    let m_c = Arc::clone(&metrics);
                    tokio::spawn(async move {
                        let _ = http_c
                            .json_request(
                                "POST",
                                "/api/ella-world/chat",
                                Some(&json!({ "token": tok_c, "text": "great runway look bestie!" })),
                            )
                            .await;
                        m_c.chats_sent.fetch_add(1, Ordering::Relaxed);
                    });
                }
            }
        }
    }

    if let Some(vh) = voice_handle {
        vh.abort();
    }
    ws_rx_handle.abort();
    if let Some(pc) = pc_holder {
        let _ = pc.close().await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = StressConfig::from_args();
    let tls_cfg = build_tls_client_config(cfg.insecure_tls);
    let http =
        HttpTarget::parse(&cfg.base_url, Arc::clone(&tls_cfg)).context("Parse --base-url")?;
    let webrtc_api = Arc::new(build_client_webrtc_api()?);
    let metrics = MetricsCollector::new();
    let stop_notify = Arc::new(Notify::new());

    println!(
        "🚀 ELLA World Classroom Stress Runner\n   Target: {} | Mode: {} | Stages: {:?} | Hold: {}s | Active Talkers: {}",
        cfg.base_url, cfg.mode, cfg.stages, cfg.hold_secs, cfg.active_talkers
    );

    let (st_health, init_health) = http
        .json_request("GET", "/api/ella-world/health", None)
        .await
        .context("Initial /api/ella-world/health check failed")?;
    if st_health != 200 {
        bail!("Health check returned HTTP {st_health}: {init_health}");
    }
    println!(
        "   Server v{} | Initial blocksVersion={} | webrtcUdpPort={}",
        init_health["version"].as_str().unwrap_or("?"),
        init_health["blocksVersion"],
        init_health["webrtcUdpPort"]
    );

    let mut current_bots = 0usize;
    let mut handles = Vec::new();
    let mut reports = Vec::new();

    for &target_n in &cfg.stages {
        if target_n < current_bots {
            continue;
        }
        let delta = target_n - current_bots;
        println!(
            "\n=== Stage: Ramping {} -> {} concurrent students (+{}) ===",
            current_bots, target_n, delta
        );

        let barrier = Arc::new(tokio::sync::Barrier::new(delta + 1));
        let delay_per_bot = if delta > 1 {
            cfg.ramp_window_ms / (delta as u64)
        } else {
            0
        };

        for offset in 0..delta {
            let bot_idx = current_bots + offset;
            let cfg_c = cfg.clone();
            let http_c = http.clone();
            let tls_c = Arc::clone(&tls_cfg);
            let api_c = Arc::clone(&webrtc_api);
            let m_c = Arc::clone(&metrics);
            let stop_c = Arc::clone(&stop_notify);
            let bar_c = Arc::clone(&barrier);

            let h = tokio::spawn(async move {
                if delay_per_bot > 0 {
                    tokio::time::sleep(Duration::from_millis(delay_per_bot * offset as u64)).await;
                }
                run_virtual_student(bot_idx, cfg_c, http_c, tls_c, api_c, m_c, stop_c, bar_c).await;
            });
            handles.push(h);
        }

        // Wait for stage bots to finish handshake (or time out gracefully at 15s)
        let _ = tokio::time::timeout(Duration::from_secs(15), barrier.wait()).await;
        // Let ICE/DataChannels finish settling for 2.5s before opening steady-state window
        tokio::time::sleep(Duration::from_millis(2500)).await;
        current_bots = target_n;

        let (_, h_start) = http
            .json_request("GET", "/api/ella-world/health", None)
            .await
            .unwrap_or((500, json!({})));
        let tick_start = h_start["serverTick"].as_u64().unwrap_or(0);
        let t_hold = Instant::now();
        metrics.reset_steady_window().await;

        tokio::time::sleep(Duration::from_secs(cfg.hold_secs)).await;
        metrics.recording.store(false, Ordering::SeqCst);
        let elapsed_s = t_hold.elapsed().as_secs_f64();

        let (_, h_end) = http
            .json_request("GET", "/api/ella-world/health", None)
            .await
            .unwrap_or((500, json!({})));
        let tick_end = h_end["serverTick"].as_u64().unwrap_or(tick_start);
        let tick_hz = (tick_end.saturating_sub(tick_start) as f64) / elapsed_s.max(0.1);

        let mut logins = metrics.login_latencies_ms.lock().await.clone();
        logins.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut rtcs = metrics.rtc_offer_latencies_ms.lock().await.clone();
        rtcs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut rtts = metrics.state_rtt_ms.lock().await.clone();
        rtts.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let udp_snaps = metrics.snapshots_via_udp.load(Ordering::Relaxed);
        let ws_snaps = metrics.snapshots_via_ws.load(Ordering::Relaxed);
        let total_snaps = udp_snaps + ws_snaps;
        let ws_conn = metrics.ws_connected.load(Ordering::Relaxed);
        let snaps_per_bot_hz = if ws_conn > 0 {
            (total_snaps as f64) / (ws_conn as f64 * elapsed_s.max(0.1))
        } else {
            0.0
        };
        let udp_pct = if total_snaps > 0 {
            (udp_snaps as f64 / total_snaps as f64) * 100.0
        } else {
            0.0
        };

        let seen_map = metrics.seen_peers_by_bot.read().await;
        let counts: Vec<usize> = seen_map.values().map(|s| s.len()).collect();
        let avg_vis = if counts.is_empty() {
            0.0
        } else {
            counts.iter().sum::<usize>() as f64 / counts.len() as f64
        };
        let max_vis = counts.iter().copied().max().unwrap_or(0);
        let expected_others = (target_n.saturating_sub(1)).max(1) as f64;
        let vis_pct = (avg_vis / expected_others) * 100.0;

        let report = StageReport {
            target_players: target_n,
            login_ok: metrics.login_ok.load(Ordering::Relaxed),
            login_err: metrics.login_err.load(Ordering::Relaxed),
            ws_connected: ws_conn,
            ws_disconnects_in_hold: metrics.ws_disconnects.load(Ordering::Relaxed),
            udp_dc_open: metrics.udp_dc_open.load(Ordering::Relaxed),
            rtc_errors: metrics.rtc_errors.load(Ordering::Relaxed),
            login_p95_ms: (percentile(&logins, 0.95) * 10.0).round() / 10.0,
            rtc_offer_p95_ms: (percentile(&rtcs, 0.95) * 10.0).round() / 10.0,
            snapshot_hz_per_bot: (snaps_per_bot_hz * 10.0).round() / 10.0,
            udp_share_pct: (udp_pct * 10.0).round() / 10.0,
            state_rtt_p50_ms: (percentile(&rtts, 0.50) * 10.0).round() / 10.0,
            state_rtt_p95_ms: (percentile(&rtts, 0.95) * 10.0).round() / 10.0,
            state_rtt_max_ms: (rtts.last().copied().unwrap_or(0.0) * 10.0).round() / 10.0,
            avg_visible_peers: (avg_vis * 10.0).round() / 10.0,
            max_visible_peers: max_vis,
            visibility_pct: (vis_pct * 10.0).round() / 10.0,
            audio_rtp_packets_rx: metrics.audio_rtp_rx.load(Ordering::Relaxed),
            server_tick_hz: (tick_hz * 100.0).round() / 100.0,
            server_active_players: h_end["activePlayers"].as_u64().unwrap_or(0),
            server_active_ws: h_end["activeWsSessions"].as_u64().unwrap_or(0),
            server_active_webrtc: h_end["activeWebrtcSessions"].as_u64().unwrap_or(0),
            duration_secs: (elapsed_s * 10.0).round() / 10.0,
        };

        println!(
            "📊 [N={:>3}] Login: {}/{} (p95 {:.1}ms) | WS: {} (drop:{}) | WebRTC DC: {} (err:{}, offer p95 {:.1}ms)",
            report.target_players,
            report.login_ok,
            report.login_ok + report.login_err,
            report.login_p95_ms,
            report.ws_connected,
            report.ws_disconnects_in_hold,
            report.udp_dc_open,
            report.rtc_errors,
            report.rtc_offer_p95_ms,
        );
        println!(
            "           SnapHz: {:.1}/s ({:.1}% UDP) | RTT p50/p95/max: {:.1}/{:.1}/{:.1}ms | Peers Seen: {:.1}/{} ({:.1}%) | SrvTick: {:.2}Hz | AudioRTP: {}",
            report.snapshot_hz_per_bot,
            report.udp_share_pct,
            report.state_rtt_p50_ms,
            report.state_rtt_p95_ms,
            report.state_rtt_max_ms,
            report.avg_visible_peers,
            target_n.saturating_sub(1),
            report.visibility_pct,
            report.server_tick_hz,
            report.audio_rtp_packets_rx
        );
        reports.push(report);
    }

    stop_notify.notify_waiters();
    tokio::time::sleep(Duration::from_millis(500)).await;
    for h in handles {
        h.abort();
    }

    if let Some(report_path) = cfg.json_report {
        if let Some(parent) = std::path::Path::new(&report_path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let body = serde_json::to_string_pretty(&json!({
            "target": cfg.base_url,
            "mode": cfg.mode,
            "stages": reports
        }))?;
        fs::write(&report_path, body)?;
        println!("\n✅ Saved JSON report to {report_path}");
    }

    Ok(())
}

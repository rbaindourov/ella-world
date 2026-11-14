use bytes::{BufMut, BytesMut};
use ella_world::config::ServerConfig;
use ella_world::start_all_servers;
use ella_world::udp_wire::{
    decode_s2c_world_state, encode_c2s_state, encode_native_hello, unwrap_native_packet,
    wrap_native_packet, DecodedC2SState, OP_C2S_STATE, OP_EVENT_ACK, OP_EVENT_JSON,
    OP_S2C_WORLD_STATE, OP_VOICE_OPUS, OP_WELCOME,
};
use ella_world::world::WorldEngine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as TungsMessage;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
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

fn make_test_config(test_name: &str) -> ServerConfig {
    let unique = format!(
        "ella-world-it-{}-{}-{}",
        test_name,
        std::process::id(),
        rand::random::<u32>()
    );
    let data_dir = std::env::temp_dir().join(unique);
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ServerConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        webrtc_udp_bind_ip: "127.0.0.1".into(),
        webrtc_udp_port: 0,
        native_udp_addr: "127.0.0.1:0".parse().unwrap(),
        public_ip: None,
        data_dir,
        public_dir: manifest_dir.join("public"),
        seed_users_file: manifest_dir.join("config/users.example.json"),
        hmac_secret: "integration-test-hmac-secret-2026".into(),
        tls_cert_path: None,
        tls_key_path: None,
    }
}

async fn http_json_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).await.expect("connect HTTP");
    let body_bytes = body
        .map(|v| serde_json::to_vec(&v).unwrap())
        .unwrap_or_default();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nAccept-Encoding: identity\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
        body_bytes.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(&body_bytes).await.unwrap();
    }
    let mut raw_resp = Vec::new();
    stream.read_to_end(&mut raw_resp).await.unwrap();
    let resp_str = String::from_utf8_lossy(&raw_resp);
    let (head, body_part) = resp_str
        .split_once("\r\n\r\n")
        .expect("valid HTTP response headers");
    let status: u16 = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);
    let parsed: Value = serde_json::from_str(body_part.trim()).unwrap_or_else(|_| json!({}));
    (status, parsed)
}

#[tokio::test]
async fn http_persistence_and_zero_sse_contract() {
    let cfg = make_test_config("http_persist");
    let data_dir = cfg.data_dir.clone();
    let seed_file = cfg.seed_users_file.clone();

    let running = start_all_servers(cfg).await.expect("start servers");

    // 1. Verify /api/ella-world/health confirms Hybrid WS Spine + Zero-Renegotiation SFU + Zero SSE
    let (status, health) =
        http_json_request(running.http_addr, "GET", "/api/ella-world/health", None).await;
    assert_eq!(status, 200);
    assert_eq!(health["engine"], "ella-world-rust-udp");
    assert_eq!(health["version"], "0.3.0");
    assert_eq!(health["wsSpineEnabled"], true);
    assert_eq!(health["zeroRenegotiationSfu"], true);
    assert_eq!(health["sseEnabled"], false);
    assert_eq!(health["httpPollingSyncEnabled"], false);
    assert_eq!(
        health["webrtcUdpPort"].as_u64().unwrap() as u16,
        running.webrtc_udp_port
    );

    // 2. Login as Ella and mutate both outfit & voxel world blocks
    let (l_status, login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "ella",
            "password": "changeme-starlight",
            "clientId": "tabA"
        })),
    )
    .await;
    assert_eq!(l_status, 200);
    let token = login["token"].as_str().unwrap().to_string();

    let (av_status, av_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": token,
            "avatar": {
                "topStyle": "puff_blouse",
                "bottomStyle": "cargo_pants",
                "hat": "beret",
                "fabricFinish": "satin"
            }
        })),
    )
    .await;
    assert_eq!(av_status, 200);
    assert_eq!(av_res["user"]["avatar"]["topStyle"], "puff_blouse");
    assert_eq!(av_res["user"]["avatar"]["hat"], "beret");

    let (blk_status, blk_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "place",
            "x": 7,
            "y": 3,
            "z": -9,
            "color": "#34d399"
        })),
    )
    .await;
    assert_eq!(blk_status, 200);
    assert!(blk_res["blocksVersion"].as_u64().unwrap() >= 2);

    running.http_task.abort();

    // 3. Boot a fresh WorldEngine on the same data_dir and verify state survived restart!
    let reload_cfg = ServerConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        webrtc_udp_bind_ip: "127.0.0.1".into(),
        webrtc_udp_port: 0,
        native_udp_addr: "127.0.0.1:0".parse().unwrap(),
        public_ip: None,
        data_dir,
        public_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public"),
        seed_users_file: seed_file,
        hmac_secret: "integration-test-hmac-secret-2026".into(),
        tls_cert_path: None,
        tls_key_path: None,
    };
    let reloaded = WorldEngine::new(reload_cfg).expect("reload world");
    let ella = reloaded
        .users_cfg
        .read()
        .await
        .users
        .iter()
        .find(|u| u.username == "ella")
        .cloned()
        .expect("ella user");
    assert_eq!(ella.avatar.top_style, "puff_blouse");
    assert_eq!(ella.avatar.bottom_style, "cargo_pants");
    assert_eq!(ella.avatar.hat, "beret");

    let persisted_blocks = reloaded.blocks_data.read().await.blocks.clone();
    assert!(
        persisted_blocks
            .iter()
            .any(|b| b.x == 7 && b.y == 3 && b.z == -9 && b.color == "#34d399"),
        "Placed block must persist across restarts"
    );
}

#[tokio::test]
async fn native_udp_voice_and_state_sync_end_to_end() {
    let cfg = make_test_config("native_udp");
    let running = start_all_servers(cfg).await.expect("start servers");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "u1" })),
    )
    .await;
    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "u2" })),
    )
    .await;

    let ella_token = ella_login["token"].as_str().unwrap();
    let maya_token = maya_login["token"].as_str().unwrap();

    let sock_ella = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sock_maya = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    sock_ella.connect(running.native_udp_addr).await.unwrap();
    sock_maya.connect(running.native_udp_addr).await.unwrap();

    // 1. Both peers authenticate over Native Raw UDP (`OP_HELLO` -> `OP_WELCOME`)
    sock_ella
        .send(&encode_native_hello(ella_token))
        .await
        .unwrap();
    sock_maya
        .send(&encode_native_hello(maya_token))
        .await
        .unwrap();

    let mut buf = [0u8; 2048];
    let n1 = tokio::time::timeout(Duration::from_secs(2), sock_ella.recv(&mut buf))
        .await
        .unwrap()
        .unwrap();
    let (op1, _) = unwrap_native_packet(&buf[..n1]).expect("EWLD packet");
    assert_eq!(op1, OP_WELCOME);

    let n2 = tokio::time::timeout(Duration::from_secs(2), sock_maya.recv(&mut buf))
        .await
        .unwrap()
        .unwrap();
    let (op2, _) = unwrap_native_packet(&buf[..n2]).expect("EWLD packet");
    assert_eq!(op2, OP_WELCOME);

    // 2. Ella streams 28-byte binary C2S_STATE over UDP; Maya receives S2C_WORLD_STATE over UDP
    let move_pkt = encode_c2s_state(&DecodedC2SState {
        seq: 10,
        client_time_ms: 55_000,
        x: 14.5,
        y: 2.25,
        z: -11.0,
        ry: 1.25,
        anim: "run".into(),
        emote: Some("dance".into()),
        muted: false,
        speaking: true,
        zone: 0,
    });
    let wrapped_move = wrap_native_packet(OP_C2S_STATE, &move_pkt);
    sock_ella.send(&wrapped_move).await.unwrap();

    let mut received_ella_move = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline {
        let len = tokio::time::timeout(Duration::from_millis(500), sock_maya.recv(&mut buf))
            .await
            .unwrap()
            .unwrap();
        if let Some((OP_S2C_WORLD_STATE, body)) = unwrap_native_packet(&buf[..len]) {
            let snap = decode_s2c_world_state(body).expect("decode S2C");
            if let Some(p) = snap.players.iter().find(|p| p.id == "peer_ella_u1") {
                if (p.x - 14.5).abs() < 0.01 && p.emote.as_deref() == Some("dance") && p.speaking {
                    received_ella_move = true;
                    break;
                }
            }
        }
    }
    assert!(
        received_ella_move,
        "Maya must receive Ella's 3D movement & speaking state over UDP"
    );

    // 3. Ella transmits an Opus voice packet over UDP (`OP_VOICE_OPUS`); Maya receives it over UDP
    let mut voice_body = BytesMut::new();
    voice_body.put_u16_le(77);
    voice_body.put_u32_le(960 * 77);
    voice_body.put_slice(b"OPUS_UDP_VOICE_SAMPLE_FRAME");
    let voice_pkt = wrap_native_packet(OP_VOICE_OPUS, &voice_body);
    sock_ella.send(&voice_pkt).await.unwrap();

    let mut received_voice = false;
    let voice_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < voice_deadline {
        let len = tokio::time::timeout(Duration::from_millis(500), sock_maya.recv(&mut buf))
            .await
            .unwrap()
            .unwrap();
        if let Some((OP_VOICE_OPUS, body)) = unwrap_native_packet(&buf[..len]) {
            let seq = u16::from_le_bytes([body[0], body[1]]);
            let id_len = body[6] as usize;
            let from_id = std::str::from_utf8(&body[7..7 + id_len]).unwrap();
            let opus_payload = &body[7 + id_len..];
            if seq == 77
                && from_id == "peer_ella_u1"
                && opus_payload == b"OPUS_UDP_VOICE_SAMPLE_FRAME"
            {
                received_voice = true;
                break;
            }
        }
    }
    assert!(
        received_voice,
        "Maya must receive Ella's Opus voice frame over UDP"
    );

    // 4. Maya places a voxel block over UDP (`OP_EVENT_JSON`) and gets `OP_EVENT_ACK`
    let cmd_bytes = serde_json::to_vec(&json!({
        "type": "block",
        "action": "place",
        "x": -5,
        "y": 4,
        "z": 12,
        "color": "#fbbf24"
    }))
    .unwrap();
    let mut ev_body = BytesMut::new();
    ev_body.put_u32_le(901);
    ev_body.put_slice(&cmd_bytes);
    sock_maya
        .send(&wrap_native_packet(OP_EVENT_JSON, &ev_body))
        .await
        .unwrap();

    let mut got_ack = false;
    let ack_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < ack_deadline {
        let len = tokio::time::timeout(Duration::from_millis(500), sock_maya.recv(&mut buf))
            .await
            .unwrap()
            .unwrap();
        if let Some((OP_EVENT_ACK, body)) = unwrap_native_packet(&buf[..len]) {
            let ack_seq = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
            if ack_seq == 901 {
                got_ack = true;
                break;
            }
        }
    }
    assert!(got_ack, "Maya must receive OP_EVENT_ACK over UDP");

    running.http_task.abort();
}

fn build_webrtc_test_client_api() -> webrtc::api::API {
    let mut media_engine = MediaEngine::default();
    media_engine
        .register_codec(
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
        )
        .unwrap();
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine).unwrap();
    let mut setting_engine = webrtc::api::setting_engine::SettingEngine::default();
    setting_engine.set_network_types(vec![webrtc::ice::network_type::NetworkType::Udp4]);
    setting_engine.set_include_loopback_candidate(true);
    APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .with_setting_engine(setting_engine)
        .build()
}

#[tokio::test]
async fn webrtc_sfu_udp_state_and_opus_voice_forwarding_end_to_end() {
    let cfg = make_test_config("webrtc_sfu");
    let running = start_all_servers(cfg).await.expect("start servers");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "w1" })),
    )
    .await;
    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "w2" })),
    )
    .await;

    let ella_token = ella_login["token"].as_str().unwrap().to_string();
    let maya_token = maya_login["token"].as_str().unwrap().to_string();

    let client_api = build_webrtc_test_client_api();

    // --- Client 1: Ella (sends Opus RTP voice + 28-byte binary C2S state over UDP) ---
    let pc_ella = Arc::new(
        client_api
            .new_peer_connection(RTCConfiguration::default())
            .await
            .unwrap(),
    );
    let ella_mic_track = Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_OPUS.to_owned(),
            clock_rate: 48_000,
            channels: 2,
            sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
            rtcp_feedback: vec![],
        },
        "mic_ella".into(),
        "stream_ella".into(),
    ));
    let ella_tx = pc_ella
        .add_transceiver_from_track(
            Arc::clone(&ella_mic_track) as Arc<dyn TrackLocal + Send + Sync>,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Sendonly,
                send_encodings: vec![],
            }),
        )
        .await
        .unwrap();
    let ella_sender = ella_tx.sender().await;
    tokio::spawn(async move {
        let mut rtcp = vec![0u8; 1500];
        while ella_sender.read(&mut rtcp).await.is_ok() {}
    });

    let (ella_open_tx, mut ella_open_rx) = mpsc::channel::<()>(2);
    let ella_state_dc = pc_ella
        .create_data_channel(
            "ella-udp-state",
            Some(RTCDataChannelInit {
                ordered: Some(false),
                max_retransmits: Some(0),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
    ella_state_dc.on_open(Box::new(move || {
        let tx = ella_open_tx.clone();
        Box::pin(async move {
            let _ = tx.send(()).await;
        })
    }));
    let _ella_events_dc = pc_ella
        .create_data_channel("ella-reliable-events", None)
        .await
        .unwrap();

    let mut gather_ella = pc_ella.gathering_complete_promise().await;
    let offer_ella = pc_ella.create_offer(None).await.unwrap();
    pc_ella.set_local_description(offer_ella).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(2), gather_ella.recv()).await;
    let sdp_ella = pc_ella.local_description().await.unwrap().sdp;

    let (st1, ans1) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/rtc/offer",
        Some(json!({ "token": ella_token, "sdp": sdp_ella })),
    )
    .await;
    assert_eq!(st1, 200);
    pc_ella
        .set_remote_description(
            RTCSessionDescription::answer(ans1["sdp"].as_str().unwrap().into()).unwrap(),
        )
        .await
        .unwrap();

    // --- Client 2: Maya (pre-allocates 8 `Recvonly` Opus speaker slots — ZERO renegotiation!) ---
    let pc_maya = Arc::new(
        client_api
            .new_peer_connection(RTCConfiguration::default())
            .await
            .unwrap(),
    );
    for _ in 0..8 {
        pc_maya
            .add_transceiver_from_kind(
                RTPCodecType::Audio,
                Some(RTCRtpTransceiverInit {
                    direction: RTCRtpTransceiverDirection::Recvonly,
                    send_encodings: vec![],
                }),
            )
            .await
            .unwrap();
    }

    let (maya_voice_tx, mut maya_voice_rx) = mpsc::channel::<(String, Vec<u8>)>(16);
    pc_maya.on_track(Box::new(move |track, _rx, _tx| {
        let vtx = maya_voice_tx.clone();
        let slot_stream_id = track.stream_id().to_owned();
        Box::pin(async move {
            while let Ok((pkt, _)) = track.read_rtp().await {
                let _ = vtx
                    .send((slot_stream_id.clone(), pkt.payload.to_vec()))
                    .await;
            }
        })
    }));

    let (maya_state_tx, mut maya_state_rx) = mpsc::channel::<Vec<u8>>(32);
    let maya_state_dc = pc_maya
        .create_data_channel(
            "ella-udp-state",
            Some(RTCDataChannelInit {
                ordered: Some(false),
                max_retransmits: Some(0),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
    maya_state_dc.on_message(Box::new(move |msg: DataChannelMessage| {
        let stx = maya_state_tx.clone();
        Box::pin(async move {
            let _ = stx.send(msg.data.to_vec()).await;
        })
    }));

    let reneg_count = Arc::new(AtomicUsize::new(0));
    let reneg_count_cb = Arc::clone(&reneg_count);
    let (slot_map_tx, mut slot_map_rx) = mpsc::channel::<Value>(8);
    let maya_events_dc = pc_maya
        .create_data_channel("ella-reliable-events", None)
        .await
        .unwrap();
    maya_events_dc.on_message(Box::new(move |msg: DataChannelMessage| {
        let r_cnt = Arc::clone(&reneg_count_cb);
        let sm_tx = slot_map_tx.clone();
        Box::pin(async move {
            if let Ok(val) = serde_json::from_slice::<Value>(&msg.data) {
                if val.get("type").and_then(|v| v.as_str()) == Some("rtc_renegotiate") {
                    r_cnt.fetch_add(1, Ordering::Relaxed);
                } else if val.get("type").and_then(|v| v.as_str()) == Some("voice_slot_map") {
                    let _ = sm_tx.send(val).await;
                }
            }
        })
    }));

    let mut gather_maya = pc_maya.gathering_complete_promise().await;
    let offer_maya = pc_maya.create_offer(None).await.unwrap();
    pc_maya.set_local_description(offer_maya).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(2), gather_maya.recv()).await;
    let sdp_maya = pc_maya.local_description().await.unwrap().sdp;

    let (st2, ans2) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/rtc/offer",
        Some(json!({ "token": maya_token, "sdp": sdp_maya })),
    )
    .await;
    assert_eq!(st2, 200);
    pc_maya
        .set_remote_description(
            RTCSessionDescription::answer(ans2["sdp"].as_str().unwrap().into()).unwrap(),
        )
        .await
        .unwrap();

    // Wait for Ella's UDP DataChannel to open
    tokio::time::timeout(Duration::from_secs(5), ella_open_rx.recv())
        .await
        .expect("Ella UDP DataChannel open timeout");

    // 1. Verify 28-byte binary game state sync over WebRTC UDP DataChannel (`ella-udp-state`)
    let c2s_bytes = encode_c2s_state(&DecodedC2SState {
        seq: 1,
        client_time_ms: 42_000,
        x: -22.5,
        y: 1.5,
        z: 6.0,
        ry: 0.75,
        anim: "walk".into(),
        emote: Some("pose".into()),
        muted: false,
        speaking: true,
        zone: 0,
    });
    ella_state_dc.send(&c2s_bytes).await.unwrap();

    let mut saw_ella_pose = false;
    let state_deadline = tokio::time::Instant::now() + Duration::from_secs(4);
    while tokio::time::Instant::now() < state_deadline {
        if let Ok(Some(raw)) =
            tokio::time::timeout(Duration::from_millis(500), maya_state_rx.recv()).await
        {
            if let Some(snap) = decode_s2c_world_state(&raw) {
                if let Some(p) = snap.players.iter().find(|p| p.id == "peer_ella_w1") {
                    if (p.x - (-22.5)).abs() < 0.01 && p.emote.as_deref() == Some("pose") {
                        saw_ella_pose = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(
        saw_ella_pose,
        "Maya's WebRTC UDP DataChannel must receive Ella's binary 3D pose snapshot"
    );

    // 2. Verify Opus RTP voice audio forwarded over pre-allocated `ella_slot_0` with ZERO renegotiation!
    let voice_task_track = Arc::clone(&ella_mic_track);
    let pump = tokio::spawn(async move {
        let mut seq = 1u16;
        loop {
            let pkt = webrtc::rtp::packet::Packet {
                header: webrtc::rtp::header::Header {
                    version: 2,
                    payload_type: 111,
                    sequence_number: seq,
                    timestamp: u32::from(seq) * 960,
                    ssrc: 0xABCD_1234,
                    ..Default::default()
                },
                payload: bytes::Bytes::from_static(b"ELLA_SFU_OPUS_VOICE"),
            };
            let _ = voice_task_track.write_rtp(&pkt).await;
            seq = seq.wrapping_add(1);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    let (slot_stream_id, forwarded_payload) =
        tokio::time::timeout(Duration::from_secs(5), maya_voice_rx.recv())
            .await
            .expect("Maya SFU UDP voice receive timeout")
            .expect("Maya voice channel open");
    assert_eq!(slot_stream_id, "ella_slot_0");
    assert_eq!(forwarded_payload, b"ELLA_SFU_OPUS_VOICE");

    let slot_map_ev = tokio::time::timeout(Duration::from_secs(2), slot_map_rx.recv())
        .await
        .expect("voice_slot_map event")
        .unwrap();
    assert_eq!(slot_map_ev["slots"]["ella_slot_0"], "peer_ella_w1");
    assert_eq!(
        reneg_count.load(Ordering::Relaxed),
        0,
        "Zero-renegotiation SFU must NEVER send rtc_renegotiate"
    );

    pump.abort();
    let _ = pc_ella.close().await;
    let _ = pc_maya.close().await;
    running.http_task.abort();
}

#[tokio::test]
async fn ws_spine_presence_grace_window_and_binary_fallback_end_to_end() {
    let cfg = make_test_config("ws_spine_grace");
    let running = start_all_servers(cfg).await.expect("start servers");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "ws1" })),
    )
    .await;
    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "ws2" })),
    )
    .await;

    let ella_token = ella_login["token"].as_str().unwrap();
    let maya_token = maya_login["token"].as_str().unwrap();

    let url_ella = format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, ella_token
    );
    let url_maya = format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, maya_token
    );

    let (mut ws_ella, _) = tokio_tungstenite::connect_async(&url_ella).await.unwrap();
    let (mut ws_maya, _) = tokio_tungstenite::connect_async(&url_maya).await.unwrap();

    // 1. Verify immediate `welcome_snapshot` on WebSocket connect
    let first_ella = tokio::time::timeout(Duration::from_secs(2), ws_ella.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let welcome_json: Value = serde_json::from_str(first_ella.to_text().unwrap()).unwrap();
    assert_eq!(welcome_json["type"], "welcome_snapshot");
    assert_eq!(welcome_json["selfId"], "peer_ella_ws1");
    assert!(welcome_json["serverTimeMs"].as_u64().unwrap() > 0);

    // 2. Verify 20 Hz binary `OP_C2S_STATE` -> `OP_S2C_WORLD_STATE` fallback over WebSocket
    let move_bytes = encode_c2s_state(&DecodedC2SState {
        seq: 1,
        client_time_ms: 9001,
        x: 19.25,
        y: 3.5,
        z: -7.75,
        ry: 0.5,
        anim: "run".into(),
        emote: Some("peace".into()),
        muted: false,
        speaking: false,
        zone: 0,
    });
    ws_ella
        .send(TungsMessage::Binary(move_bytes.to_vec()))
        .await
        .unwrap();

    let mut maya_saw_ella_ws_binary = false;
    let bin_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < bin_deadline {
        if let Ok(Some(Ok(TungsMessage::Binary(raw)))) =
            tokio::time::timeout(Duration::from_millis(400), ws_maya.next()).await
        {
            if let Some(snap) = decode_s2c_world_state(&raw) {
                if let Some(p) = snap.players.iter().find(|p| p.id == "peer_ella_ws1") {
                    if (p.x - 19.25).abs() < 0.01 && p.emote.as_deref() == Some("peace") {
                        maya_saw_ella_ws_binary = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(
        maya_saw_ella_ws_binary,
        "Maya must receive Ella's 20 Hz binary state over WebSocket when UDP fast-lane is inactive"
    );

    // 3. Simulate brief Wi-Fi drop on Ella's WebSocket -> verify 12s grace window does NOT drop Ella!
    let _ = ws_ella.close(None).await;
    drop(ws_ella);
    tokio::time::sleep(Duration::from_millis(350)).await;

    let (_, health_mid) =
        http_json_request(running.http_addr, "GET", "/api/ella-world/health", None).await;
    assert_eq!(
        health_mid["activePlayers"], 2,
        "12-second grace window must keep Ella online during transient socket drop"
    );

    // Ella reconnects within grace window with restarted sequence counter (seq=1 again)
    let (mut ws_ella_reconn, _) = tokio_tungstenite::connect_async(&url_ella).await.unwrap();
    let reconn_move = encode_c2s_state(&DecodedC2SState {
        seq: 1,
        client_time_ms: 9500,
        x: -11.0,
        y: 1.0,
        z: 15.0,
        ry: 1.0,
        anim: "walk".into(),
        emote: Some("spin".into()),
        muted: false,
        speaking: false,
        zone: 0,
    });
    ws_ella_reconn
        .send(TungsMessage::Binary(reconn_move.to_vec()))
        .await
        .unwrap();

    // Verify Maya receives both Ella's post-reconnect movement AND periodic `world_sync_digest` without any `player_leave`!
    let mut saw_post_reconn_spin = false;
    let mut saw_sync_digest = false;
    let mut saw_spurious_leave = false;
    let digest_deadline = tokio::time::Instant::now() + Duration::from_millis(3800);
    while tokio::time::Instant::now() < digest_deadline
        && (!saw_post_reconn_spin || !saw_sync_digest)
    {
        if let Ok(Some(Ok(msg))) =
            tokio::time::timeout(Duration::from_millis(500), ws_maya.next()).await
        {
            match msg {
                TungsMessage::Binary(raw) => {
                    if let Some(snap) = decode_s2c_world_state(&raw) {
                        if let Some(p) = snap.players.iter().find(|p| p.id == "peer_ella_ws1") {
                            if (p.x - (-11.0)).abs() < 0.01 && p.emote.as_deref() == Some("spin") {
                                saw_post_reconn_spin = true;
                            }
                        }
                    }
                }
                TungsMessage::Text(txt) => {
                    if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                        if val["type"] == "player_leave" && val["id"] == "peer_ella_ws1" {
                            saw_spurious_leave = true;
                        } else if val["type"] == "world_sync_digest" {
                            let has_ella = val["players"]
                                .as_array()
                                .map(|arr| {
                                    arr.iter().any(|p| {
                                        p["id"] == "peer_ella_ws1"
                                            && p["avatarHash"]
                                                .as_str()
                                                .map(|h| h.len() == 16)
                                                .unwrap_or(false)
                                    })
                                })
                                .unwrap_or(false);
                            if has_ella {
                                saw_sync_digest = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    assert!(
        !saw_spurious_leave,
        "Grace window reconnect must never emit a spurious player_leave"
    );
    assert!(
        saw_post_reconn_spin,
        "Reset sequence numbers after reconnect must be accepted immediately"
    );
    assert!(
        saw_sync_digest,
        "3-second world_sync_digest must broadcast authoritative player roster + avatarHash"
    );

    running.http_task.abort();
}

#[tokio::test]
async fn hybrid_ws_plus_zero_renegotiation_sfu_failover_stability() {
    let cfg = make_test_config("hybrid_failover");
    let running = start_all_servers(cfg).await.expect("start servers");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "hf1" })),
    )
    .await;
    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "hf2" })),
    )
    .await;

    let ella_token = ella_login["token"].as_str().unwrap().to_string();
    let maya_token = maya_login["token"].as_str().unwrap().to_string();

    // Both connect WebSocket spines first
    let (mut ws_ella, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, ella_token
    ))
    .await
    .unwrap();
    let (mut ws_maya, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, maya_token
    ))
    .await
    .unwrap();

    // Maya also connects WebRTC UDP fast-lane and signals `udp_fastlane: true`
    let client_api = build_webrtc_test_client_api();
    let pc_maya = Arc::new(
        client_api
            .new_peer_connection(RTCConfiguration::default())
            .await
            .unwrap(),
    );
    let (maya_open_tx, mut maya_open_rx) = mpsc::channel::<()>(2);
    let (maya_udp_rx_tx, mut maya_udp_rx) = mpsc::channel::<Vec<u8>>(32);
    let maya_state_dc = pc_maya
        .create_data_channel(
            "ella-udp-state",
            Some(RTCDataChannelInit {
                ordered: Some(false),
                max_retransmits: Some(0),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
    maya_state_dc.on_open(Box::new(move || {
        let tx = maya_open_tx.clone();
        Box::pin(async move {
            let _ = tx.send(()).await;
        })
    }));
    maya_state_dc.on_message(Box::new(move |msg: DataChannelMessage| {
        let tx = maya_udp_rx_tx.clone();
        Box::pin(async move {
            let _ = tx.send(msg.data.to_vec()).await;
        })
    }));

    let mut gather_maya = pc_maya.gathering_complete_promise().await;
    let offer_maya = pc_maya.create_offer(None).await.unwrap();
    pc_maya.set_local_description(offer_maya).await.unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(2), gather_maya.recv()).await;
    let sdp_maya = pc_maya.local_description().await.unwrap().sdp;

    let (st_m, ans_m) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/rtc/offer",
        Some(json!({ "token": maya_token, "sdp": sdp_maya })),
    )
    .await;
    assert_eq!(st_m, 200);
    pc_maya
        .set_remote_description(
            RTCSessionDescription::answer(ans_m["sdp"].as_str().unwrap().into()).unwrap(),
        )
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), maya_open_rx.recv())
        .await
        .expect("Maya UDP DataChannel open");

    // Maya marks UDP fast-lane active and sends a heartbeat over UDP so `last_udp_seen_ms` is fresh
    let maya_hb = encode_c2s_state(&DecodedC2SState {
        seq: 1,
        client_time_ms: 1000,
        x: 0.0,
        y: 0.5,
        z: 0.0,
        ry: 0.0,
        anim: "idle".into(),
        emote: None,
        muted: false,
        speaking: false,
        zone: 0,
    });
    maya_state_dc.send(&maya_hb).await.unwrap();
    ws_maya
        .send(TungsMessage::Text(
            json!({ "type": "udp_fastlane", "active": true }).to_string(),
        ))
        .await
        .unwrap();

    // Ella moves; Maya receives over UDP fast-lane (`44444/udp`)
    let ella_m1 = encode_c2s_state(&DecodedC2SState {
        seq: 2,
        client_time_ms: 2000,
        x: 8.5,
        y: 1.0,
        z: -4.0,
        ry: 0.25,
        anim: "walk".into(),
        emote: None,
        muted: false,
        speaking: false,
        zone: 0,
    });
    ws_ella
        .send(TungsMessage::Binary(ella_m1.to_vec()))
        .await
        .unwrap();

    let mut saw_over_udp = false;
    let d1 = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < d1 {
        if let Ok(Some(raw)) =
            tokio::time::timeout(Duration::from_millis(400), maya_udp_rx.recv()).await
        {
            if let Some(snap) = decode_s2c_world_state(&raw) {
                if snap
                    .players
                    .iter()
                    .any(|p| p.id == "peer_ella_hf1" && (p.x - 8.5).abs() < 0.01)
                {
                    saw_over_udp = true;
                    break;
                }
            }
        }
    }
    assert!(saw_over_udp, "Maya must receive state over UDP fast-lane");

    // Now simulate Maya's UDP connection failing completely (`pc_maya.close()`)!
    // Verify:
    //   1) Maya is NOT evicted from `WorldEngine::players` (`onlinePlayers == 2`)
    //   2) Maya's WebSocket immediately resumes 20 Hz binary `0x11` state delivery!
    let _ = pc_maya.close().await;
    ws_maya
        .send(TungsMessage::Text(
            json!({ "type": "udp_fastlane", "active": false }).to_string(),
        ))
        .await
        .unwrap();

    let (_, health_after_udp_drop) =
        http_json_request(running.http_addr, "GET", "/api/ella-world/health", None).await;
    assert_eq!(
        health_after_udp_drop["activePlayers"], 2,
        "Closing WebRTC UDP must NEVER evict a player whose WebSocket spine is connected!"
    );

    let ella_m2 = encode_c2s_state(&DecodedC2SState {
        seq: 3,
        client_time_ms: 2500,
        x: 31.25,
        y: 4.0,
        z: 12.0,
        ry: 1.5,
        anim: "jump".into(),
        emote: Some("cheer".into()),
        muted: false,
        speaking: false,
        zone: 0,
    });
    ws_ella
        .send(TungsMessage::Binary(ella_m2.to_vec()))
        .await
        .unwrap();

    let mut saw_ws_failover = false;
    let d2 = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < d2 {
        if let Ok(Some(Ok(TungsMessage::Binary(raw)))) =
            tokio::time::timeout(Duration::from_millis(400), ws_maya.next()).await
        {
            if let Some(snap) = decode_s2c_world_state(&raw) {
                if snap.players.iter().any(|p| {
                    p.id == "peer_ella_hf1"
                        && (p.x - 31.25).abs() < 0.01
                        && p.emote.as_deref() == Some("cheer")
                }) {
                    saw_ws_failover = true;
                    break;
                }
            }
        }
    }
    assert!(
        saw_ws_failover,
        "WebSocket spine must instantaneously take over 20 Hz binary state delivery when UDP drops"
    );

    running.http_task.abort();
}

async fn http_json_auth_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    bearer: &str,
    body: Option<Value>,
) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).await.expect("connect HTTP");
    let body_bytes = body
        .map(|v| serde_json::to_vec(&v).unwrap())
        .unwrap_or_default();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {bearer}\r\nContent-Type: application/json\r\nAccept-Encoding: identity\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
        body_bytes.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();
    if !body_bytes.is_empty() {
        stream.write_all(&body_bytes).await.unwrap();
    }
    let mut raw_resp = Vec::new();
    stream.read_to_end(&mut raw_resp).await.unwrap();
    let resp_str = String::from_utf8_lossy(&raw_resp);
    let (head, body_part) = resp_str
        .split_once("\r\n\r\n")
        .expect("valid HTTP response headers");
    let status: u16 = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);
    let parsed: Value = serde_json::from_str(body_part.trim()).unwrap_or_else(|_| json!({}));
    (status, parsed)
}

#[tokio::test]
async fn chat_persistence_word_filter_and_admin_chat_viewer_work_across_restart() {
    let cfg = make_test_config("chat-safety");
    let data_dir = cfg.data_dir.clone();
    let seed_file = cfg.seed_users_file.clone();
    let running = start_all_servers(cfg).await.expect("start server");

    // 1. Login as ella
    let (st_login, login_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight" })),
    )
    .await;
    assert_eq!(st_login, 200);
    let token = login_res["token"].as_str().unwrap().to_string();

    // 2. Clean friendly messages succeed (and legitimate words like "classic" / "butterfly" pass)
    let (st_ok1, ok1) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": token, "text": "hello bestie! classic butterfly runway look!" })),
    )
    .await;
    assert_eq!(st_ok1, 200);
    assert_eq!(ok1["ok"], true);

    // 3. Profanity / masked profanity is rejected with 400 and logged to world_chat_blocked.jsonl
    let (st_bad1, bad1) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": token, "text": "what the fuuuck" })),
    )
    .await;
    assert_eq!(st_bad1, 400);
    assert_eq!(bad1["ok"], false);

    let (st_bad2, _bad2) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": token, "text": "stop being a b1tch" })),
    )
    .await;
    assert_eq!(st_bad2, 400);

    // 4. Parent Admin login -> inspect /admin/chats?includeBlocked=true and customize blockedWords
    let (st_adm, adm_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(st_adm, 200);
    let admin_token = adm_res["adminToken"].as_str().unwrap().to_string();
    assert_eq!(adm_res["dashboard"]["chatCount"], 1);
    assert_eq!(adm_res["dashboard"]["blockedChatCount"], 2);

    let (st_chats, chats_res) = http_json_auth_request(
        running.http_addr,
        "GET",
        "/api/ella-world/admin/chats?includeBlocked=true&limit=50",
        &admin_token,
        None,
    )
    .await;
    assert_eq!(st_chats, 200);
    assert_eq!(chats_res["total"], 1);
    assert_eq!(chats_res["blockedTotal"], 2);
    assert_eq!(
        chats_res["chats"][0]["text"],
        "hello bestie! classic butterfly runway look!"
    );

    // Add a custom blocked word via /admin/settings
    let (st_set, _set_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/settings",
        &admin_token,
        Some(json!({
            "chatFilterEnabled": true,
            "blockedWords": ["fuck", "meanyhead"]
        })),
    )
    .await;
    assert_eq!(st_set, 200);

    let (st_custom_blk, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": token, "text": "you are a meanyhead" })),
    )
    .await;
    assert_eq!(st_custom_blk, 400);

    running.http_task.abort();

    // 5. Boot a fresh WorldEngine on the same data_dir -> chats must hydrate from world_chat.jsonl!
    let reload_cfg = ServerConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        webrtc_udp_bind_ip: "127.0.0.1".into(),
        webrtc_udp_port: 0,
        native_udp_addr: "127.0.0.1:0".parse().unwrap(),
        public_ip: None,
        data_dir,
        public_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public"),
        seed_users_file: seed_file,
        hmac_secret: "integration-test-hmac-secret-2026".into(),
        tls_cert_path: None,
        tls_key_path: None,
    };
    let reloaded = WorldEngine::new(reload_cfg).expect("reload world");
    let chats_after_restart = reloaded.chats.read().await;
    assert_eq!(chats_after_restart.len(), 1);
    assert_eq!(
        chats_after_restart[0].text,
        "hello bestie! classic butterfly runway look!"
    );
    assert_eq!(
        reloaded.users_cfg.read().await.blocked_words,
        vec!["fuck", "meanyhead"]
    );
}

#[tokio::test]
async fn scaling_pre_encoded_chunked_snapshots_and_async_persistence() {
    let cfg = make_test_config("scaling_100");
    let world = WorldEngine::new(cfg.clone()).expect("world engine");

    for i in 0..100usize {
        let sid = format!("peer_scale_{i:03}");
        let res = world
            .ensure_player_online(&sid, if i % 2 == 0 { "ella" } else { "maya" })
            .await;
        assert!(res.is_some());
        world
            .apply_c2s_state(
                &sid,
                DecodedC2SState {
                    seq: 1,
                    client_time_ms: 1000 + i as u32,
                    x: (i as f32) * 0.25,
                    y: 0.0,
                    z: -3.0,
                    ry: 0.5,
                    anim: "run".into(),
                    emote: None,
                    muted: false,
                    speaking: i < 3,
                    zone: 0,
                },
            )
            .await;
    }

    world.rebuild_pre_encoded_tick(77).await;
    let chunks = world
        .encode_world_snapshot_chunks_for("peer_scale_000")
        .await;
    assert!(
        chunks.len() >= 2,
        "100 peers must chunk into >= 2 MTU-safe packets (got {})",
        chunks.len()
    );
    let mut seen_ids = std::collections::HashSet::new();
    for c in &chunks {
        assert!(
            c.len() <= 1240,
            "every chunk must stay within MAX_SNAPSHOT_MTU_BYTES (got {})",
            c.len()
        );
        let decoded = decode_s2c_world_state(c).expect("decode chunk");
        assert_eq!(decoded.server_tick, 77);
        assert_eq!(decoded.echo_client_time_ms, 1000);
        for p in decoded.players {
            seen_ids.insert(p.id);
        }
    }
    assert_eq!(
        seen_ids.len(),
        99,
        "all 99 other peers must be delivered without any 64-player cap"
    );
    assert!(!seen_ids.contains("peer_scale_000"));

    // Verify round-robin scheduled chunk selector covers all 99 peers across consecutive ticks
    let mut sched_ids = std::collections::HashSet::new();
    for tick in 100u32..106u32 {
        world.rebuild_pre_encoded_tick(tick).await;
        let pre = world.pre_encoded_tick.read().await.clone();
        let pkt = WorldEngine::encode_pre_tick_scheduled_chunk_for(&pre, "peer_scale_000");
        assert!(pkt.len() <= 1240);
        let decoded = decode_s2c_world_state(&pkt).expect("decode scheduled chunk");
        for p in decoded.players {
            sched_ids.insert(p.id);
        }
    }
    assert_eq!(
        sched_ids.len(),
        99,
        "scheduled chunk rotation must cover all 99 other peers cleanly within <= 6 ticks"
    );

    // Verify async block persistence + cached sync digest work seamlessly
    let (ver, blocks) = world
        .mutate_block(
            "peer_scale_000",
            "ella",
            "place",
            (4, 2, 4),
            Some("#38bdf8"),
        )
        .await
        .expect("mutate_block");
    assert!(ver >= 2);
    assert!(blocks.iter().any(|b| b.x == 4 && b.y == 2 && b.z == 4));

    let digest_json = world.get_cached_sync_digest_json().await;
    assert!(digest_json.contains("world_sync_digest"));
}

#[tokio::test]
async fn dungeon_portal_farming_and_live_p2p_trade_end_to_end() {
    let cfg = make_test_config("dungeon_and_p2p_trade");
    let data_dir = cfg.data_dir.clone();
    let seed_file = cfg.seed_users_file.clone();
    let running = start_all_servers(cfg).await.expect("start servers");

    // 1. Verify item catalog endpoint exposes all 12 items
    let (st_cat, cat_res) = http_json_request(
        running.http_addr,
        "GET",
        "/api/ella-world/economy/catalog",
        None,
    )
    .await;
    assert_eq!(st_cat, 200);
    assert_eq!(cat_res["itemCatalog"].as_array().unwrap().len(), 12);

    // 2. Login Ella & Maya and connect WebSocket spines
    let (st_e, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "ec1" })),
    )
    .await;
    assert_eq!(st_e, 200);
    let ella_token = ella_login["token"].as_str().unwrap().to_string();
    assert_eq!(ella_login["self"]["zone"], 0);
    assert_eq!(ella_login["economy"]["coins"], 0);

    let (st_m, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "ec2" })),
    )
    .await;
    assert_eq!(st_m, 200);
    let maya_token = maya_login["token"].as_str().unwrap().to_string();

    let (mut ws_ella, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, ella_token
    ))
    .await
    .unwrap();
    let (_ws_maya, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, maya_token
    ))
    .await
    .unwrap();

    // 3. Attempting to equip unowned dungeon-exclusive Obsidian Crown falls back to 'crown'
    let (st_av_locked, av_locked_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": ella_token,
            "avatar": { "hat": "obsidian_crown" }
        })),
    )
    .await;
    assert_eq!(st_av_locked, 200);
    assert_eq!(
        av_locked_res["user"]["avatar"]["hat"], "crown",
        "Unowned dungeon-exclusive cosmetic must not equip"
    );

    // 4. Transition Ella into Crystal Cavern Dungeon (zone = 1)
    let (st_portal, portal_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({ "token": ella_token, "targetZone": 1 })),
    )
    .await;
    assert_eq!(st_portal, 200);
    assert_eq!(portal_res["zone"], 1);

    // Verify 20 Hz binary snapshot zone isolation: Maya (zone 0) should NOT see Ella (zone 1)
    running.world.rebuild_pre_encoded_tick(500).await;
    let maya_plaza_chunks = running
        .world
        .encode_world_snapshot_chunks_for("peer_maya_ec2")
        .await;
    let maya_decoded = decode_s2c_world_state(&maya_plaza_chunks[0]).unwrap();
    assert!(
        maya_decoded.players.is_empty(),
        "Plaza snapshot must exclude peers currently inside Crystal Cavern Dungeon"
    );

    // 5. Ella walks to crystal_1 (-14, 0, 12) in zone 1, harvests it, and picks up drop
    let move_to_crystal = encode_c2s_state(&DecodedC2SState {
        seq: 1,
        client_time_ms: 1000,
        x: -14.0,
        y: 0.0,
        z: 12.0,
        ry: 0.0,
        anim: "walk".into(),
        emote: None,
        muted: false,
        speaking: false,
        zone: 1,
    });
    ws_ella
        .send(TungsMessage::Binary(move_to_crystal.to_vec()))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(60)).await;

    let (st_harv, harv_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/interact",
        Some(json!({ "token": ella_token, "nodeId": "crystal_1" })),
    )
    .await;
    assert_eq!(st_harv, 200);
    let crystal_drop_id = harv_res["drop"]["id"].as_str().unwrap().to_string();

    let (st_pick1, pick1_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/pickup",
        Some(json!({ "token": ella_token, "dropId": crystal_drop_id })),
    )
    .await;
    assert_eq!(st_pick1, 200);
    assert!(pick1_res["economy"]["coins"].as_u64().unwrap() >= 10);

    // 6. Ella walks to slime_1 (0, 0, 8), Jump-Stomps it (2 dmg = instant defeat), and collects drop
    let move_to_slime = encode_c2s_state(&DecodedC2SState {
        seq: 2,
        client_time_ms: 1100,
        x: 0.0,
        y: 0.0,
        z: 8.0,
        ry: 0.0,
        anim: "jump".into(),
        emote: None,
        muted: false,
        speaking: false,
        zone: 1,
    });
    ws_ella
        .send(TungsMessage::Binary(move_to_slime.to_vec()))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(60)).await;

    let (st_atk, atk_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": ella_token, "mobId": "slime_1", "mode": "stomp" })),
    )
    .await;
    assert_eq!(st_atk, 200);
    assert_eq!(atk_res["defeated"], true);
    let slime_drop_id = atk_res["drop"]["id"].as_str().unwrap().to_string();

    let (st_pick2, pick2_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/pickup",
        Some(json!({ "token": ella_token, "dropId": slime_drop_id })),
    )
    .await;
    assert_eq!(st_pick2, 200);
    let ella_farmed_coins = pick2_res["economy"]["coins"].as_u64().unwrap();
    assert!(ella_farmed_coins >= 22);

    // 7. Grant deterministic Obsidian Crown + Speed Potion to Ella & 60 coins to Maya for P2P trade test
    {
        let mut cfg_guard = running.world.users_cfg.write().await;
        let ella_u = cfg_guard
            .users
            .iter_mut()
            .find(|u| u.username == "ella")
            .unwrap();
        ella_u
            .inventory
            .insert("cosmetic_hat_obsidian_crown".into(), 1);
        ella_u.inventory.insert("speed_potion".into(), 1);
        let maya_u = cfg_guard
            .users
            .iter_mut()
            .find(|u| u.username == "maya")
            .unwrap();
        maya_u.coins = 60;
    }

    // Drink Speed Potion -> grants speedBoostUntilMs and decrements stack
    let (st_use, use_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/economy/use",
        Some(json!({ "token": ella_token, "itemId": "speed_potion" })),
    )
    .await;
    assert_eq!(st_use, 200);
    assert!(use_res["speedBoostUntilMs"].as_u64().unwrap() > 0);

    // Equip Obsidian Crown on Ella now that she owns 1x cosmetic_hat_obsidian_crown
    let (st_av_ok, av_ok_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": ella_token,
            "avatar": { "hat": "obsidian_crown" }
        })),
    )
    .await;
    assert_eq!(st_av_ok, 200);
    assert_eq!(av_ok_res["user"]["avatar"]["hat"], "obsidian_crown");

    // 8. Execute Live Two-Sided Scam-Proof P2P Trade between Ella and Maya
    let (st_inv, inv_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/invite",
        Some(json!({ "token": ella_token, "targetPeerId": "peer_maya_ec2" })),
    )
    .await;
    assert_eq!(st_inv, 200);
    let trade_id = inv_res["trade"]["tradeId"].as_str().unwrap().to_string();

    let (st_resp, resp_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/respond",
        Some(json!({ "token": maya_token, "tradeId": trade_id, "accept": true })),
    )
    .await;
    assert_eq!(st_resp, 200);
    assert_eq!(resp_res["trade"]["status"], "active");

    // Ella offers 1x Obsidian Crown + 10 coins; Maya offers 45 coins
    let (st_of_e, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/offer",
        Some(json!({
            "token": ella_token,
            "tradeId": trade_id,
            "coins": 10,
            "items": { "cosmetic_hat_obsidian_crown": 1 }
        })),
    )
    .await;
    assert_eq!(st_of_e, 200);

    let (st_of_m, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/offer",
        Some(json!({
            "token": maya_token,
            "tradeId": trade_id,
            "coins": 45,
            "items": {}
        })),
    )
    .await;
    assert_eq!(st_of_m, 200);

    // Both lock offers
    let (st_l1, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/lock",
        Some(json!({ "token": ella_token, "tradeId": trade_id, "locked": true })),
    )
    .await;
    assert_eq!(st_l1, 200);
    let (st_l2, l2_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/lock",
        Some(json!({ "token": maya_token, "tradeId": trade_id, "locked": true })),
    )
    .await;
    assert_eq!(st_l2, 200);
    assert_eq!(l2_res["trade"]["initiatorOffer"]["locked"], true);
    assert_eq!(l2_res["trade"]["partnerOffer"]["locked"], true);

    // Anti-bait check: Maya edits coin offer from 45 -> 35, resetting BOTH locks!
    let (st_edit, edit_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/offer",
        Some(json!({
            "token": maya_token,
            "tradeId": trade_id,
            "coins": 35,
            "items": {}
        })),
    )
    .await;
    assert_eq!(st_edit, 200);
    assert_eq!(edit_res["trade"]["initiatorOffer"]["locked"], false);
    assert_eq!(edit_res["trade"]["partnerOffer"]["locked"], false);

    // Confirming while unlocked MUST fail with 400
    let (st_premature_cf, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/confirm",
        Some(json!({ "token": ella_token, "tradeId": trade_id })),
    )
    .await;
    assert_eq!(st_premature_cf, 400);

    // Re-lock both sides and confirm both sides
    http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/lock",
        Some(json!({ "token": ella_token, "tradeId": trade_id, "locked": true })),
    )
    .await;
    http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/lock",
        Some(json!({ "token": maya_token, "tradeId": trade_id, "locked": true })),
    )
    .await;

    let (st_cf1, cf1_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/confirm",
        Some(json!({ "token": ella_token, "tradeId": trade_id })),
    )
    .await;
    assert_eq!(st_cf1, 200);
    assert_eq!(cf1_res["completed"], false);

    let (st_cf2, cf2_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/confirm",
        Some(json!({ "token": maya_token, "tradeId": trade_id })),
    )
    .await;
    assert_eq!(st_cf2, 200);
    assert_eq!(cf2_res["completed"], true);

    // Allow background spawn_blocking persistence write to settle
    tokio::time::sleep(Duration::from_millis(120)).await;
    running.http_task.abort();

    // 9. Reload WorldEngine from disk and verify atomic conservation + automatic cosmetic unequip!
    let reload_cfg = ServerConfig {
        http_addr: "127.0.0.1:0".parse().unwrap(),
        webrtc_udp_bind_ip: "127.0.0.1".into(),
        webrtc_udp_port: 0,
        native_udp_addr: "127.0.0.1:0".parse().unwrap(),
        public_ip: None,
        data_dir,
        public_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public"),
        seed_users_file: seed_file,
        hmac_secret: "integration-test-hmac-secret-2026".into(),
        tls_cert_path: None,
        tls_key_path: None,
    };
    let reloaded = WorldEngine::new(reload_cfg).expect("reload world");
    let cfg_after = reloaded.users_cfg.read().await;
    let ella_after = cfg_after
        .users
        .iter()
        .find(|u| u.username == "ella")
        .unwrap();
    let maya_after = cfg_after
        .users
        .iter()
        .find(|u| u.username == "maya")
        .unwrap();

    // Ella gave 10 coins and received 35 coins (net +25); Maya had 60, gave 35, received 10 (35 left)
    assert_eq!(ella_after.coins, ella_farmed_coins + 25);
    assert_eq!(maya_after.coins, 35);
    assert_eq!(
        ella_after
            .inventory
            .get("cosmetic_hat_obsidian_crown")
            .copied()
            .unwrap_or(0),
        0
    );
    assert_eq!(
        maya_after
            .inventory
            .get("cosmetic_hat_obsidian_crown")
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(
        ella_after.avatar.hat, "crown",
        "Trading away equipped Obsidian Crown must automatically unequip back to 'crown'"
    );
}

#[tokio::test]
async fn male_oblivion_jack_tech49_avatar_api_and_resume_persistence() {
    let cfg = make_test_config("male_oblivion_avatar");
    let running = start_all_servers(cfg).await.expect("start servers");

    let (st_login, login_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "m1" })),
    )
    .await;
    assert_eq!(st_login, 200);
    let token = login_res["token"].as_str().unwrap().to_string();
    assert_eq!(login_res["self"]["user"]["avatar"]["bodyType"], "female");

    // Equip Jack Tech-49 Oblivion flight suit preset via HTTP avatar endpoint
    let (st_av, av_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": token,
            "avatar": {
                "avatarMode": "stylized",
                "bodyType": "male",
                "characterPreset": "jack_tech49",
                "skinColor": "#eab692",
                "eyeColor": "#38bdf8",
                "hairStyle": "tech_crop",
                "hairColor": "#3b2516",
                "topStyle": "tech_jacket",
                "topColor": "#e2e8f0",
                "bottomStyle": "tech_pants",
                "bottomColor": "#64748b",
                "shoeStyle": "flight_boots",
                "shoeColor": "#334155",
                "hat": "recon_cap",
                "hatColor": "#334155",
                "accessory": "aviators",
                "accessoryColor": "#38bdf8",
                "fabricFinish": "matte",
                "face": "hero"
            }
        })),
    )
    .await;
    assert_eq!(st_av, 200);
    assert_eq!(av_res["user"]["avatar"]["bodyType"], "male");
    assert_eq!(av_res["user"]["avatar"]["characterPreset"], "jack_tech49");
    assert_eq!(av_res["user"]["avatar"]["hairStyle"], "tech_crop");
    assert_eq!(av_res["user"]["avatar"]["topStyle"], "tech_jacket");
    assert_eq!(av_res["user"]["avatar"]["bottomStyle"], "tech_pants");
    assert_eq!(av_res["user"]["avatar"]["shoeStyle"], "flight_boots");
    assert_eq!(av_res["user"]["avatar"]["hat"], "recon_cap");
    assert_eq!(av_res["user"]["avatar"]["accessory"], "aviators");
    assert_eq!(av_res["user"]["avatar"]["face"], "hero");

    // Verify /api/ella-world/resume restores the persisted male build & Oblivion wardrobe cleanly
    let (st_res, resume_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/resume",
        Some(json!({ "token": token, "clientId": "m1" })),
    )
    .await;
    assert_eq!(st_res, 200);
    assert_eq!(resume_res["self"]["user"]["avatar"]["bodyType"], "male");
    assert_eq!(
        resume_res["self"]["user"]["avatar"]["topStyle"],
        "tech_jacket"
    );
    assert_eq!(
        resume_res["self"]["user"]["avatar"]["accessory"],
        "aviators"
    );
}

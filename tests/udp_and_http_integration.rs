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
    // Note: each profanity violation also broadcasts a public Judge Starlight court notice (isSystem=true),
    // so 1 clean message + 2 Judge Starlight notices = 3 persisted chat messages and 2 blocked audit entries.
    tokio::time::sleep(Duration::from_millis(60)).await;
    let (st_adm, adm_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(st_adm, 200);
    let admin_token = adm_res["adminToken"].as_str().unwrap().to_string();
    assert_eq!(adm_res["dashboard"]["chatCount"], 3);
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
    assert_eq!(chats_res["total"], 3);
    assert_eq!(chats_res["blockedTotal"], 2);
    let chat_texts: Vec<&str> = chats_res["chats"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["text"].as_str())
        .collect();
    assert!(
        chat_texts.contains(&"hello bestie! classic butterfly runway look!"),
        "Clean chat message must be present in persisted chat history"
    );
    assert!(
        chat_texts.iter().any(|t| t.contains("Debtor's Prison")),
        "Judge Starlight court enforcement notice must be present in chat history"
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
    tokio::time::sleep(Duration::from_millis(60)).await;

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
    assert_eq!(chats_after_restart.len(), 4);
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

    // 1. Verify item catalog endpoint exposes all items (18 world items + 3 parent-exclusive items)
    let (st_cat, cat_res) = http_json_request(
        running.http_addr,
        "GET",
        "/api/ella-world/economy/catalog",
        None,
    )
    .await;
    assert_eq!(st_cat, 200);
    assert_eq!(
        cat_res["itemCatalog"].as_array().unwrap().len(),
        ella_world::economy::ITEM_CATALOG.len()
    );

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
        Some(json!({ "token": ella_token, "zone": 1, "targetZone": 1 })),
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
    let ella_farmed_coins = pick2_res["economy"]["coins"].as_i64().unwrap();
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

async fn http_json_request_with_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    bearer_token: Option<&str>,
    body: Option<Value>,
) -> (u16, String, Value) {
    let mut stream = TcpStream::connect(addr).await.expect("connect HTTP");
    let body_bytes = body
        .map(|v| serde_json::to_vec(&v).unwrap())
        .unwrap_or_default();
    let auth_header = bearer_token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n{auth_header}Accept-Encoding: identity\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
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
    (status, head.to_ascii_lowercase(), parsed)
}

#[tokio::test]
async fn password_rollover_and_global_session_revocation_expire_tokens_and_kick_live_transports() {
    use ella_world::auth::{sign_token, TokenClaims};

    let cfg = make_test_config("session_revoke_and_pw_roll");
    let hmac_secret = cfg.hmac_secret.clone();
    let running = start_all_servers(cfg).await.expect("start servers");

    // 1. Verify legacy pre-epoch tokens (missing exp / sessionEpoch / userTokenEpoch / pwdSig) are rejected
    let legacy_token = sign_token(
        &TokenClaims {
            self_id: Some("peer_ella_leg".into()),
            username: Some("ella".into()),
            role: Some("host".into()),
            iat: 1_700_000_000_000,
            exp: None,
            session_epoch: None,
            user_token_epoch: None,
            pwd_sig: None,
            device_id: None,
        },
        &hmac_secret,
    );
    let (leg_status, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/resume",
        Some(json!({ "token": legacy_token, "clientId": "leg" })),
    )
    .await;
    assert_eq!(
        leg_status, 401,
        "Legacy tokens without epoch & password fingerprint must be rejected"
    );

    let leg_ws_res = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, legacy_token
    ))
    .await;
    assert!(
        leg_ws_res.is_err(),
        "Legacy token must fail WebSocket upgrade"
    );

    // 2. Ella and Maya log in and connect live WebSocket spines (+ Ella connects Native UDP)
    let (st_e, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "rev1" })),
    )
    .await;
    assert_eq!(st_e, 200);
    let ella_old_token = ella_login["token"].as_str().unwrap().to_string();

    let (st_m, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "rev2" })),
    )
    .await;
    assert_eq!(st_m, 200);
    let maya_old_token = maya_login["token"].as_str().unwrap().to_string();

    let (mut ws_ella, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, ella_old_token
    ))
    .await
    .expect("Ella WS connect");
    let (mut ws_maya, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, maya_old_token
    ))
    .await
    .expect("Maya WS connect");

    // Drain initial welcome_snapshot frames
    let _ = tokio::time::timeout(Duration::from_secs(2), ws_ella.next())
        .await
        .unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(2), ws_maya.next())
        .await
        .unwrap();

    // Connect Native UDP client for Ella as well
    let udp_sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let hello_pkt = encode_native_hello(&ella_old_token);
    udp_sock
        .send_to(&hello_pkt, running.native_udp_addr)
        .await
        .unwrap();
    let mut udp_buf = [0u8; 2048];
    let (n_udp, _) = tokio::time::timeout(Duration::from_secs(2), udp_sock.recv_from(&mut udp_buf))
        .await
        .expect("UDP welcome timeout")
        .unwrap();
    let (op_udp, _) = unwrap_native_packet(&udp_buf[..n_udp]).unwrap();
    assert_eq!(op_udp, OP_WELCOME);

    let (_, health_before) =
        http_json_request(running.http_addr, "GET", "/api/ella-world/health", None).await;
    assert_eq!(health_before["activePlayers"], 2);
    assert_eq!(health_before["activeNativeUdpSessions"], 1);

    // 3. Parent logs into Admin API and rotates Ella's password
    let (adm_st, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(adm_st, 200);
    let admin_token_1 = adm_login["adminToken"].as_str().unwrap().to_string();

    let (rot_st, _, rot_res) = http_json_request_with_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/users",
        Some(&admin_token_1),
        Some(json!({
            "username": "ella",
            "displayName": "Ella",
            "password": "rolled-super-secret-2026"
        })),
    )
    .await;
    assert_eq!(rot_st, 200);
    assert_eq!(rot_res["passwordRotated"], true);
    assert_eq!(rot_res["kickedSessions"], 1);

    // Verify Ella's open WebSocket receives `session_revoked` and closes immediately!
    let mut saw_ella_revoked_frame = false;
    let revoke_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < revoke_deadline {
        match tokio::time::timeout(Duration::from_millis(500), ws_ella.next()).await {
            Ok(Some(Ok(TungsMessage::Text(txt)))) => {
                if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                    if v["type"] == "session_revoked" {
                        saw_ella_revoked_frame = true;
                    }
                }
            }
            Ok(Some(Ok(TungsMessage::Close(_)))) | Ok(None) | Ok(Some(Err(_))) => break,
            _ => {}
        }
    }
    assert!(
        saw_ella_revoked_frame,
        "Open WebSocket must receive `session_revoked` frame immediately when password is rolled"
    );

    tokio::time::sleep(Duration::from_millis(120)).await;
    let (_, health_after_roll) =
        http_json_request(running.http_addr, "GET", "/api/ella-world/health", None).await;
    assert_eq!(
        health_after_roll["activePlayers"], 1,
        "Kicked player must be removed immediately without 12s grace window while Maya remains online"
    );
    assert_eq!(
        health_after_roll["activeNativeUdpSessions"], 0,
        "Native UDP session must also be evicted on password rollover"
    );

    // Verify Ella's old token fails 401 across /resume, /block, /chat, and /ws
    let (res_old_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/resume",
        Some(json!({ "token": ella_old_token, "clientId": "rev1" })),
    )
    .await;
    assert_eq!(res_old_st, 401);

    let (blk_old_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({ "token": ella_old_token, "action": "place", "x": 2, "y": 0, "z": 2, "color": "#ec4899" })),
    )
    .await;
    assert_eq!(blk_old_st, 401);

    assert!(
        tokio_tungstenite::connect_async(format!(
            "ws://{}/api/ella-world/ws?token={}",
            running.http_addr, ella_old_token
        ))
        .await
        .is_err(),
        "Rolled-out token must fail WebSocket reconnect"
    );

    // Old password login fails; new password login succeeds!
    let (bad_pw_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "rev1" })),
    )
    .await;
    assert_eq!(bad_pw_st, 401);

    let (new_pw_st, ella_relogin) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "rolled-super-secret-2026", "clientId": "rev1" })),
    )
    .await;
    assert_eq!(new_pw_st, 200);
    let ella_new_token = ella_relogin["token"].as_str().unwrap().to_string();

    let (mut ws_ella_2, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, ella_new_token
    ))
    .await
    .expect("Ella WS reconnect with new password token");
    let _ = tokio::time::timeout(Duration::from_secs(2), ws_ella_2.next())
        .await
        .unwrap();

    // 4. Parent triggers Global Session Revocation (`POST /api/ella-world/admin/sessions/revoke` with `{}`)
    let (glob_st, _, glob_res) = http_json_request_with_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/sessions/revoke",
        Some(&admin_token_1),
        Some(json!({})),
    )
    .await;
    assert_eq!(glob_st, 200);
    assert_eq!(glob_res["scope"], "all");
    assert_eq!(glob_res["sessionEpoch"], 2);
    assert_eq!(glob_res["kickedCount"], 2);
    let admin_token_2 = glob_res["adminToken"].as_str().unwrap().to_string();

    // Both Ella's new WS and Maya's WS receive `session_revoked`!
    let mut maya_revoked = false;
    let glob_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < glob_deadline {
        match tokio::time::timeout(Duration::from_millis(500), ws_maya.next()).await {
            Ok(Some(Ok(TungsMessage::Text(txt)))) => {
                if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                    if v["type"] == "session_revoked" {
                        maya_revoked = true;
                    }
                }
            }
            Ok(Some(Ok(TungsMessage::Close(_)))) | Ok(None) | Ok(Some(Err(_))) => break,
            _ => {}
        }
    }
    assert!(
        maya_revoked,
        "Global revoke must notify Maya's open WebSocket"
    );
    let _ = ws_ella_2.close(None).await;

    tokio::time::sleep(Duration::from_millis(120)).await;
    let (_, health_after_glob) =
        http_json_request(running.http_addr, "GET", "/api/ella-world/health", None).await;
    assert_eq!(health_after_glob["activePlayers"], 0);

    // Both player tokens AND the pre-epoch admin token are now 401; fresh admin_token_2 works!
    let (e_res_after, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/resume",
        Some(json!({ "token": ella_new_token, "clientId": "rev1" })),
    )
    .await;
    assert_eq!(e_res_after, 401);

    let (m_res_after, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/resume",
        Some(json!({ "token": maya_old_token, "clientId": "rev2" })),
    )
    .await;
    assert_eq!(m_res_after, 401);

    let (old_adm_st, _, _) = http_json_request_with_headers(
        running.http_addr,
        "GET",
        "/api/ella-world/admin/state",
        Some(&admin_token_1),
        None,
    )
    .await;
    assert_eq!(
        old_adm_st, 401,
        "Pre-revoke admin token must also be invalidated when sessionEpoch bumps"
    );

    let (new_adm_st, _, new_adm_state) = http_json_request_with_headers(
        running.http_addr,
        "GET",
        "/api/ella-world/admin/state",
        Some(&admin_token_2),
        None,
    )
    .await;
    assert_eq!(new_adm_st, 200);
    assert_eq!(new_adm_state["dashboard"]["sessionEpoch"], 2);

    // Verify /logout succeeds and preserves machine seals (never emits Clear-Site-Data)
    let (logout_st, logout_headers, _) = http_json_request_with_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/logout",
        None,
        Some(json!({ "token": ella_new_token })),
    )
    .await;
    assert_eq!(logout_st, 200);
    assert!(
        !logout_headers.contains("clear-site-data"),
        "Logout endpoint must preserve cryptographic machine seals across sign-outs"
    );

    running.http_task.abort();
}

#[tokio::test]
async fn parental_controls_escalating_fines_and_debtors_prison_end_to_end() {
    let cfg = make_test_config("parental_and_debtors_prison");
    let running = start_all_servers(cfg).await.expect("start servers");

    // 1. Login Ella, Maya, and Parent Admin
    let (st_e, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "pris1" })),
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
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "pris2" })),
    )
    .await;
    assert_eq!(st_m, 200);
    let maya_token = maya_login["token"].as_str().unwrap().to_string();

    let (st_adm, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(st_adm, 200);
    let admin_token = adm_login["adminToken"].as_str().unwrap().to_string();

    let (mut ws_ella, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, ella_token
    ))
    .await
    .unwrap();

    // 2. Scunthorpe & numeric clean messages MUST pass with 0 strikes and 0 fine
    for clean_msg in [
        "hello class! let's pass the grass!",
        "i saved 455 coins in my classic scrapbook!",
    ] {
        let (st_ok, _) = http_json_request(
            running.http_addr,
            "POST",
            "/api/ella-world/chat",
            Some(json!({ "token": ella_token, "text": clean_msg })),
        )
        .await;
        assert_eq!(
            st_ok, 200,
            "Clean text should never be blocked: {clean_msg}"
        );
    }

    // 3. Evasion profanity triggers escalating negative-balance fines & auto-sentences to ZONE_PRISON (2)
    // Strike #1: spaced single-letter evasion ("f u c k") -> -15 coins (balance = -15, still Plaza)
    let (st_s1, res_s1) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": ella_token, "text": "what the f u c k is that" })),
    )
    .await;
    assert_eq!(st_s1, 400);
    assert!(res_s1["error"].as_str().unwrap().contains("-15"));

    // Strike #2: punctuation-split evasion ("s.h.i.t") -> -35 coins (balance = -50 <= -50 threshold -> Debtor's Prison!)
    let (st_s2, res_s2) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": ella_token, "text": "this is s.h.i.t right now" })),
    )
    .await;
    assert_eq!(st_s2, 400);
    assert!(
        res_s2["error"]
            .as_str()
            .unwrap()
            .contains("Debtor's Prison"),
        "Dropping to -50 coins must sentence player to Debtor's Prison"
    );

    // Multi-word leetspeak ("k1ll y0urself") is also caught while inside prison
    let (st_s3, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": ella_token, "text": "go k1ll y0urself" })),
    )
    .await;
    assert_eq!(st_s3, 400);

    tokio::time::sleep(Duration::from_millis(60)).await;
    let (_, chats_audit) = http_json_auth_request(
        running.http_addr,
        "GET",
        "/api/ella-world/admin/chats?includeBlocked=true&limit=20",
        &admin_token,
        None,
    )
    .await;
    let blocked_arr = chats_audit["blockedChats"].as_array().unwrap();
    assert_eq!(blocked_arr.len(), 3);
    assert!(
        blocked_arr
            .iter()
            .any(|b| b["sentencedToPrison"] == true && b["balanceAfter"] == -50),
        "Blocked chat audit must record strike fine, negative balanceAfter, and sentencedToPrison"
    );

    // Verify Ella's live runtime zone is ZONE_PRISON (2) and spoofed C2S coords clamp inside [-13.2, 13.2]
    let spoof_escape = encode_c2s_state(&DecodedC2SState {
        seq: 1,
        client_time_ms: 1000,
        x: 60.0,
        y: 0.0,
        z: 60.0,
        ry: 0.0,
        anim: "run".into(),
        emote: None,
        muted: false,
        speaking: false,
        zone: 0, // attempting to spoof zone 0 while imprisoned
    });
    ws_ella
        .send(TungsMessage::Binary(spoof_escape.to_vec()))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    {
        let p_map = running.world.players.read().await;
        let ella_rt = p_map.get("peer_ella_pris1").expect("ella runtime");
        assert_eq!(
            ella_rt.zone, 2,
            "Server must force imprisoned peer to stay in ZONE_PRISON (2)"
        );
        assert!(
            ella_rt.x <= 13.25 && ella_rt.z <= 13.25,
            "Prison bounds must clamp movement"
        );
    }

    // 4. Verify Prisoner Containment: portal exit, block placement, item use, and trading are all rejected!
    let (st_esc, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({ "token": ella_token, "targetZone": 0 })),
    )
    .await;
    assert_eq!(
        st_esc, 400,
        "Prisoner with negative debt & chores must not exit portal"
    );

    let (st_blk, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({ "token": ella_token, "action": "place", "x": 1, "y": 1, "z": 1, "color": "#f472b6" })),
    )
    .await;
    assert_eq!(st_blk, 400, "Prisoner cannot place blocks");

    let (st_trd, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/trade/invite",
        Some(json!({ "token": ella_token, "targetPeerId": "peer_maya_pris2" })),
    )
    .await;
    assert_eq!(st_trd, 400, "Prisoner cannot initiate P2P trade");

    // 5. Test Community Service Work Stations (+5 toward clearing negative debt, strictly capped at 0!)
    // Set Ella to -3 coins and 1 chore remaining so we verify the strict 0-coin ceiling (no positive farming!)
    {
        let mut cfg_w = running.world.users_cfg.write().await;
        let ella_u = cfg_w
            .users
            .iter_mut()
            .find(|u| u.username == "ella")
            .unwrap();
        ella_u.coins = -3;
        ella_u.prison_tasks_remaining = 1;
    }
    // Walk Ella next to chore_sweep_1 (-7.5, 0.0, -4.5)
    let move_to_chore = encode_c2s_state(&DecodedC2SState {
        seq: 2,
        client_time_ms: 1200,
        x: -7.5,
        y: 0.0,
        z: -4.5,
        ry: 0.0,
        anim: "walk".into(),
        emote: None,
        muted: false,
        speaking: false,
        zone: 2,
    });
    ws_ella
        .send(TungsMessage::Binary(move_to_chore.to_vec()))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let (st_work, work_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/prison/work",
        Some(json!({ "token": ella_token, "nodeId": "prison_rock_1" })),
    )
    .await;
    assert_eq!(st_work, 200);
    assert_eq!(
        work_res["earnedTowardDebt"], 3,
        "Work reward must strictly cap at 0 coins with zero positive coin farming"
    );
    assert_eq!(work_res["economy"]["coins"], 0);
    assert_eq!(work_res["moderation"]["prisonTasksRemaining"], 0);
    assert_eq!(work_res["readyForRelease"], true);

    // Now Ella can step through the Release Gate back to Starlight Plaza (zone 0)!
    let (st_rel, rel_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({ "token": ella_token, "targetZone": 0 })),
    )
    .await;
    assert_eq!(st_rel, 200);
    assert_eq!(rel_res["zone"], 0);
    assert_eq!(rel_res["moderation"]["inDebtorsPrison"], false);

    // 6. Parental Controls: Parent Sentence & Parent Pardon on Maya
    let (st_sent_m, sent_m_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({
            "username": "maya",
            "action": "sentence_prison",
            "prisonTasks": 5,
            "coinDelta": -40,
            "reason": "Testing Parent Judge sentence"
        })),
    )
    .await;
    assert_eq!(st_sent_m, 200);
    assert_eq!(sent_m_res["economy"]["coins"], -40);
    assert_eq!(sent_m_res["moderation"]["inDebtorsPrison"], true);
    assert_eq!(sent_m_res["moderation"]["prisonTasksRemaining"], 5);

    let (st_pard_m, pard_m_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({
            "username": "maya",
            "action": "pardon_prison"
        })),
    )
    .await;
    assert_eq!(st_pard_m, 200);
    assert_eq!(
        pard_m_res["economy"]["coins"], 0,
        "Parent Pardon must forgive negative debt back to 0"
    );
    assert_eq!(pard_m_res["moderation"]["inDebtorsPrison"], false);
    assert_eq!(pard_m_res["moderation"]["prisonTasksRemaining"], 0);

    // 7. Safe Quick-Chat-Only Mode & Server-Enforced Voice Mute over Native UDP
    let (st_qc, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({
            "username": "ella",
            "action": "update_controls",
            "chatMode": "quick_only",
            "voiceMutedByParent": true
        })),
    )
    .await;
    assert_eq!(st_qc, 200);

    // Approved quick-chat phrase succeeds, arbitrary free-text is rejected
    let (st_qp_ok, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": ella_token, "text": "⭐⭐⭐⭐⭐ 5 STARS! Slay!!" })),
    )
    .await;
    assert_eq!(st_qp_ok, 200);

    let (st_qp_bad, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({ "token": ella_token, "text": "custom free text message" })),
    )
    .await;
    assert_eq!(st_qp_bad, 400);

    // Verify server drops OP_VOICE_OPUS packets when voiceMutedByParent == true
    let sock_ella = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sock_maya = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    sock_ella.connect(running.native_udp_addr).await.unwrap();
    sock_maya.connect(running.native_udp_addr).await.unwrap();
    sock_ella
        .send(&encode_native_hello(&ella_token))
        .await
        .unwrap();
    sock_maya
        .send(&encode_native_hello(&maya_token))
        .await
        .unwrap();

    let mut udp_buf = [0u8; 2048];
    let _ = tokio::time::timeout(Duration::from_secs(1), sock_ella.recv(&mut udp_buf)).await;
    let _ = tokio::time::timeout(Duration::from_secs(1), sock_maya.recv(&mut udp_buf)).await;

    let mut voice_body = BytesMut::new();
    voice_body.put_u16_le(1);
    voice_body.put_u32_le(960);
    voice_body.extend_from_slice(&[0xF8, 0xFF, 0xFE, 0x11, 0x22]);
    let voice_pkt = wrap_native_packet(OP_VOICE_OPUS, &voice_body);
    sock_ella.send(&voice_pkt).await.unwrap();

    // Drain Maya's UDP socket for 180ms and assert NO OP_VOICE_OPUS packet arrives while Ella is parent-muted
    let deadline = tokio::time::Instant::now() + Duration::from_millis(180);
    while tokio::time::Instant::now() < deadline {
        if let Ok(Ok(n)) =
            tokio::time::timeout(Duration::from_millis(40), sock_maya.recv(&mut udp_buf)).await
        {
            if let Some((pkt_type, _)) = unwrap_native_packet(&udp_buf[..n]) {
                assert_ne!(
                    pkt_type, OP_VOICE_OPUS,
                    "Server must drop OP_VOICE_OPUS frames from parent-muted account"
                );
            }
        }
    }

    // 8. Account Pause (`suspended: true`) & Global World Lock (`worldLocked: true`) return 403 Forbidden
    let (st_susp, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({
            "username": "maya",
            "action": "update_controls",
            "suspended": true
        })),
    )
    .await;
    assert_eq!(st_susp, 200);

    let (st_maya_blocked, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "pris3" })),
    )
    .await;
    assert_eq!(
        st_maya_blocked, 403,
        "Paused child account must receive 403 Forbidden on login"
    );

    let (st_wlock, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/settings",
        &admin_token,
        Some(json!({ "worldLocked": true })),
    )
    .await;
    assert_eq!(st_wlock, 200);

    let (st_ella_wlocked, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "pris4" })),
    )
    .await;
    assert_eq!(
        st_ella_wlocked, 403,
        "World lock must block logins with 403 Forbidden"
    );

    running.http_task.abort();
}

#[tokio::test]
async fn legacy_world_blocks_compatibility_and_doors_windows_interiors_persist() {
    let cfg = make_test_config("doors_interiors");
    let data_dir = cfg.data_dir.clone();
    let seed_file = cfg.seed_users_file.clone();
    std::fs::create_dir_all(&data_dir).unwrap();

    // Seed a legacy pre-v0.4 `world_blocks.json` without `kind`, `rot`, `open`, or `lootItemId`
    std::fs::write(
        data_dir.join("world_blocks.json"),
        serde_json::to_vec_pretty(&json!({
            "version": 4,
            "blocks": [
                {
                    "x": 5,
                    "y": 1,
                    "z": 5,
                    "color": "#f472b6",
                    "author": "ella"
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let running = start_all_servers(cfg).await.expect("start server");

    let (st_login, login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "house1" })),
    )
    .await;
    assert_eq!(st_login, 200);
    let ella_token = login["token"].as_str().unwrap().to_string();

    // Verify legacy block hydrated with backward-compatible serde defaults alongside Prebuilt Village v1 seeding
    let village_count = ella_world::models::prebuilt_village_blocks().len();
    let initial_blocks = login["blocks"].as_array().unwrap();
    assert_eq!(initial_blocks.len(), 1 + village_count);
    let legacy_block = initial_blocks
        .iter()
        .find(|b| b["x"] == 5 && b["y"] == 1 && b["z"] == 5)
        .expect("legacy block at (5,1,5)");
    assert_eq!(legacy_block["kind"], "block");
    assert_eq!(legacy_block["rot"], 0);
    assert_eq!(legacy_block["open"], false);

    // Place 2-block-tall hinged Door at (6, 1, 5), rot=2, Window at (8, 2, 5), Stairs at (4, 1, 5), and Kitchen/Bath/Bedroom furniture
    let (st_door, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "place",
            "x": 6,
            "y": 1,
            "z": 5,
            "kind": "door",
            "rot": 2,
            "color": "#b45309"
        })),
    )
    .await;
    assert_eq!(st_door, 200);

    let (st_win, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "place",
            "x": 8,
            "y": 2,
            "z": 5,
            "kind": "window",
            "rot": 1,
            "color": "#f8fafc"
        })),
    )
    .await;
    assert_eq!(st_win, 200);

    for (x, kind, rot) in [
        (4, "stairs", 1),
        (7, "kitchen_stove", 3),
        (9, "bath_tub", 0),
        (10, "bed", 2),
    ] {
        let (st_prop, _) = http_json_request(
            running.http_addr,
            "POST",
            "/api/ella-world/block",
            Some(json!({
                "token": ella_token,
                "action": "place",
                "x": x,
                "y": 1,
                "z": 7,
                "kind": kind,
                "rot": rot,
                "color": "#38bdf8"
            })),
        )
        .await;
        assert_eq!(st_prop, 200, "Failed placing furniture prop {kind}");
    }

    // Toggle the 2-tall door by targeting its UPPER cell (6, 2, 5) — must resolve to (6, 1, 5) and flip `open: true`!
    let (st_tog_door, tog_door_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "toggle",
            "x": 6,
            "y": 2,
            "z": 5
        })),
    )
    .await;
    assert_eq!(st_tog_door, 200);
    let door_entry = tog_door_res["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["x"] == 6 && b["y"] == 1 && b["z"] == 5)
        .expect("door at (6,1,5)");
    assert_eq!(door_entry["open"], true);

    // Toggle the window at (8, 2, 5) -> `open: true`
    let (st_tog_win, tog_win_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "toggle",
            "x": 8,
            "y": 2,
            "z": 5
        })),
    )
    .await;
    assert_eq!(st_tog_win, 200);
    let win_entry = tog_win_res["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["x"] == 8 && b["y"] == 2 && b["z"] == 5)
        .expect("window at (8,2,5)");
    assert_eq!(win_entry["open"], true);

    tokio::time::sleep(Duration::from_millis(80)).await;
    running.http_task.abort();

    // Reload WorldEngine from disk and verify doors, windows, rotation, and furniture kinds persisted
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
    let bd = reloaded.blocks_data.read().await;
    assert!(bd.prebuilt_village_v1_seeded);
    assert_eq!(bd.blocks.len(), 7 + village_count);
    let saved_door = bd
        .blocks
        .iter()
        .find(|b| b.x == 6 && b.y == 1 && b.z == 5)
        .unwrap();
    assert_eq!(saved_door.kind, "door");
    assert_eq!(saved_door.rot, 2);
    assert!(saved_door.open);
    let saved_win = bd
        .blocks
        .iter()
        .find(|b| b.x == 8 && b.y == 2 && b.z == 5)
        .unwrap();
    assert_eq!(saved_win.kind, "window");
    assert!(saved_win.open);
}

#[tokio::test]
async fn backpack_loot_house_decorating_anti_duplication_ownership_and_reset_refund() {
    let cfg = make_test_config("loot_decor_house");
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "ld1" })),
    )
    .await;
    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "ld2" })),
    )
    .await;
    let ella_token = ella_login["token"].as_str().unwrap().to_string();
    let maya_token = maya_login["token"].as_str().unwrap().to_string();

    // Grant Ella 1x legendary_ray_gun and 1x geode_lamp
    {
        let mut guard = running.world.users_cfg.write().await;
        let ella_u = guard
            .users
            .iter_mut()
            .find(|u| u.username == "ella")
            .unwrap();
        ella_u.inventory.insert("legendary_ray_gun".into(), 1);
        ella_u.inventory.insert("geode_lamp".into(), 1);
    }

    // Equip the Legendary Ray Gun on Ella's avatar
    let (st_eq, eq_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": ella_token,
            "avatar": { "accessory": "ray_gun" }
        })),
    )
    .await;
    assert_eq!(st_eq, 200);
    assert_eq!(eq_res["user"]["avatar"]["accessory"], "ray_gun");

    // Ella places her 1x legendary_ray_gun on a house display pedestal at (10, 1, 10)
    let (st_place, place_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "place",
            "x": 10,
            "y": 1,
            "z": 10,
            "kind": "loot_decor",
            "rot": 1,
            "color": "#38bdf8",
            "lootItemId": "legendary_ray_gun"
        })),
    )
    .await;
    assert_eq!(st_place, 200);
    assert_eq!(
        place_res["economy"]["inventory"]
            .get("legendary_ray_gun")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        0,
        "Placing a loot trophy must atomically deduct 1 from Backpack inventory"
    );

    // Because her last Ray Gun is now displayed on a pedestal, avatar accessory must auto-reconcile to "none"
    {
        let guard = running.world.users_cfg.read().await;
        let ella_u = guard.users.iter().find(|u| u.username == "ella").unwrap();
        assert_eq!(ella_u.avatar.accessory, "none");
    }

    // Attempting to place a second Ray Gun without owning another copy MUST fail with 400
    let (st_dup, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "place",
            "x": 11,
            "y": 1,
            "z": 10,
            "kind": "loot_decor",
            "lootItemId": "legendary_ray_gun"
        })),
    )
    .await;
    assert_eq!(st_dup, 400);

    // Maya cannot overwrite or remove Ella's displayed trophy!
    let (st_maya_overwrite, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": maya_token,
            "action": "place",
            "x": 10,
            "y": 1,
            "z": 10,
            "kind": "block",
            "color": "#ef4444"
        })),
    )
    .await;
    assert_eq!(st_maya_overwrite, 400);

    let (st_maya_steal, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": maya_token,
            "action": "remove",
            "x": 10,
            "y": 1,
            "z": 10
        })),
    )
    .await;
    assert_eq!(st_maya_steal, 400);

    // Ella picks up her Ray Gun from (10, 1, 10) -> +1 refunded to Backpack!
    let (st_pickup, pickup_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "remove",
            "x": 10,
            "y": 1,
            "z": 10
        })),
    )
    .await;
    assert_eq!(st_pickup, 200);
    assert_eq!(pickup_res["economy"]["inventory"]["legendary_ray_gun"], 1);

    // Ella places geode_lamp at (12, 1, 10), then Parent Admin resets world blocks -> geode_lamp must be refunded!
    let (st_geode, geode_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "place",
            "x": 12,
            "y": 1,
            "z": 10,
            "kind": "loot_decor",
            "lootItemId": "geode_lamp"
        })),
    )
    .await;
    assert_eq!(st_geode, 200);
    assert_eq!(
        geode_res["economy"]["inventory"]
            .get("geode_lamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        0
    );

    let (_, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    let admin_token = adm_login["adminToken"].as_str().unwrap().to_string();

    let (st_rst, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/world/reset",
        &admin_token,
        Some(json!({})),
    )
    .await;
    assert_eq!(st_rst, 200);

    {
        let guard = running.world.users_cfg.read().await;
        let ella_u = guard.users.iter().find(|u| u.username == "ella").unwrap();
        assert_eq!(
            ella_u.inventory.get("geode_lamp").copied().unwrap_or(0),
            1,
            "Admin world reset must refund displayed Backpack trophies back to their owners"
        );
    }

    running.http_task.abort();
}

#[tokio::test]
async fn legendary_ray_gun_ranged_dungeon_combat_and_plaza_light_show() {
    let cfg = make_test_config("legendary_ray_gun");
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "rg1" })),
    )
    .await;
    let ella_token = ella_login["token"].as_str().unwrap().to_string();

    // Verify catalog includes legendary_ray_gun with rarity == "legendary"
    let catalog = ella_login["itemCatalog"].as_array().unwrap();
    let rg_def = catalog
        .iter()
        .find(|i| i["id"] == "legendary_ray_gun")
        .expect("legendary_ray_gun in itemCatalog");
    assert_eq!(rg_def["rarity"], "legendary");

    // Without owning legendary_ray_gun, equipping accessory="ray_gun" is blocked and Plaza /raygun fails
    let (st_unowned_av, unowned_av) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": ella_token,
            "avatar": { "accessory": "ray_gun" }
        })),
    )
    .await;
    assert_eq!(st_unowned_av, 200);
    assert_ne!(unowned_av["user"]["avatar"]["accessory"], "ray_gun");

    let (st_unowned_zap, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/raygun",
        Some(json!({ "token": ella_token, "x": 4.0, "y": 1.2, "z": 6.0 })),
    )
    .await;
    assert_eq!(st_unowned_zap, 400);

    // Connect WS spine, warp into Crystal Cavern (zone 1), and stand 12.0m away from slime_1 (0, 0, 8) at (0, 0, 20)
    let (mut ws_ella, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, ella_token
    ))
    .await
    .unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(2), ws_ella.next()).await;

    let (st_portal, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({ "token": ella_token, "zone": 1, "targetZone": 1 })),
    )
    .await;
    assert_eq!(st_portal, 200);

    let stand_at_12m = encode_c2s_state(&DecodedC2SState {
        seq: 2,
        client_time_ms: 1200,
        x: 0.0,
        y: 0.0,
        z: 20.0,
        ry: std::f32::consts::PI,
        anim: "idle".into(),
        emote: None,
        muted: false,
        speaking: false,
        zone: 1,
    });
    ws_ella
        .send(TungsMessage::Binary(stand_at_12m.to_vec()))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(60)).await;

    // Normal blast from 12m fails (out of 3.9m range), and raygun mode without owning item fails
    let (st_blast_far, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": ella_token, "mobId": "slime_1", "mode": "blast" })),
    )
    .await;
    assert_eq!(st_blast_far, 400);

    let (st_rg_unowned, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": ella_token, "mobId": "slime_1", "mode": "raygun" })),
    )
    .await;
    assert_eq!(st_rg_unowned, 400);

    // Grant 1x legendary_ray_gun -> ranged 12m Ray Gun beam deals 3 HP damage and defeats slime_1!
    {
        let mut guard = running.world.users_cfg.write().await;
        let ella_u = guard
            .users
            .iter_mut()
            .find(|u| u.username == "ella")
            .unwrap();
        ella_u.inventory.insert("legendary_ray_gun".into(), 1);
    }

    let (st_rg_hit, rg_hit) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": ella_token, "mobId": "slime_1", "mode": "raygun" })),
    )
    .await;
    assert_eq!(st_rg_hit, 200);
    assert_eq!(rg_hit["damage"], 3);
    assert_eq!(rg_hit["defeated"], true);

    // Return to Starlight Plaza (zone 0), equip Ray Gun, and fire harmless Plaza laser light show
    let (st_back, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({ "token": ella_token, "targetZone": 0 })),
    )
    .await;
    assert_eq!(st_back, 200);

    let (st_zap_ok, zap_ok) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/raygun",
        Some(json!({ "token": ella_token, "x": 6.0, "y": 1.5, "z": 6.0 })),
    )
    .await;
    assert_eq!(st_zap_ok, 200);
    assert_eq!(zap_ok["ok"], true);

    running.http_task.abort();
}

async fn http_json_request_custom_headers(
    addr: SocketAddr,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: Option<Value>,
) -> (u16, String, Value) {
    let mut stream = TcpStream::connect(addr).await.expect("connect HTTP");
    let body_bytes = body
        .map(|v| serde_json::to_vec(&v).unwrap())
        .unwrap_or_default();
    let mut header_block = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nAccept-Encoding: identity\r\nConnection: close\r\nContent-Length: {}\r\n",
        body_bytes.len()
    );
    for (k, v) in extra_headers {
        header_block.push_str(&format!("{k}: {v}\r\n"));
    }
    header_block.push_str("\r\n");

    stream.write_all(header_block.as_bytes()).await.unwrap();
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
    (status, head.to_string(), parsed)
}

#[tokio::test]
async fn machine_digital_signature_blocks_alt_account_evasion_and_evicts_live_tabs() {
    let cfg = make_test_config("machine_sig_anti_evasion");
    let running = start_all_servers(cfg).await.expect("start servers");

    let mac_m2_chrome_tel = json!({
        "gpuVendor": "Google Inc. (Apple)",
        "gpuRenderer": "ANGLE (Apple, ANGLE Metal Renderer: Apple M2, Unspecified Version)",
        "glCaps": "16384|16384|16384|16|4096|1024|32|16384x16384",
        "canvasHash": "c4f199a0b21177ee",
        "audioHash": "a9810cc4512bb009",
        "screenW": 1512,
        "screenH": 982,
        "colorDepth": 30,
        "pixelRatioX100": 200,
        "cpuCores": 8,
        "deviceMemoryGb": 8,
        "maxTouchPoints": 0,
        "osFamily": "macOS",
        "timezone": "America/Los_Angeles",
        "language": "en-US"
    });

    // Sibling B registers an independent identical-model MacBook BEFORE any penalty exists
    let (st_sib_reg, _, sib_reg) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.56"),
            (
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/537.36 Chrome/126.0",
            ),
        ],
        Some(json!({
            "username": "chloe",
            "password": "changeme-rainbow",
            "clientId": "sib_mac",
            "deviceTelemetry": mac_m2_chrome_tel
        })),
    )
    .await;
    assert_eq!(st_sib_reg, 200);
    let chloe_sibling_seal = sib_reg["machineSeal"]
        .as_str()
        .expect("chloe sibling machineSeal")
        .to_string();
    let chloe_sibling_sig = sib_reg["machineShortSig"].as_str().unwrap().to_string();

    // 1. Ella signs in from Machine A (`192.168.11.55`) -> receives HttpOnly `ewld_machine_seal` cookie + JSON seal
    let (st_ella, ella_headers, ella_login) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            (
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/537.36 Chrome/126.0",
            ),
        ],
        Some(json!({
            "username": "ella",
            "password": "changeme-starlight",
            "clientId": "ella_tab",
            "deviceTelemetry": {
                "gpuVendor": "Google Inc. (Apple)",
                "gpuRenderer": "ANGLE (Apple, ANGLE Metal Renderer: Apple M2, Unspecified Version)",
                "glCaps": "16384|16384|16384|16|4096|1024|32|16384x16384",
                "canvasHash": "e11a88aa00112233",
                "audioHash": "e11a77bb44556677",
                "screenW": 1512,
                "screenH": 982,
                "colorDepth": 30,
                "pixelRatioX100": 200,
                "cpuCores": 8,
                "deviceMemoryGb": 8,
                "maxTouchPoints": 0,
                "osFamily": "macOS",
                "timezone": "America/Los_Angeles",
                "language": "en-US"
            }
        })),
    )
    .await;
    assert_eq!(st_ella, 200);
    assert!(
        ella_headers
            .to_ascii_lowercase()
            .contains("set-cookie: ewld_machine_seal="),
        "Login must set HttpOnly ewld_machine_seal cookie: {ella_headers}"
    );
    assert!(ella_headers.to_ascii_lowercase().contains("httponly"));
    let ella_seal = ella_login["machineSeal"]
        .as_str()
        .expect("ella machineSeal")
        .to_string();
    let ella_short_sig = ella_login["machineShortSig"].as_str().unwrap().to_string();
    assert_ne!(ella_short_sig, chloe_sibling_sig);
    let ella_token = ella_login["token"].as_str().unwrap().to_string();

    // Ella commits 3 profanity violations -> sentenced to Debtor's Prison!
    for idx in 0..3 {
        let (st_curse, _, curse_out) = http_json_request_custom_headers(
            running.http_addr,
            "POST",
            "/api/ella-world/chat",
            &[("X-Forwarded-For", "192.168.11.55")],
            Some(json!({
                "token": ella_token,
                "text": format!("what the fuck #{idx}")
            })),
        )
        .await;
        assert_eq!(st_curse, 400);
        if idx == 2 {
            assert!(
                curse_out["error"]
                    .as_str()
                    .unwrap_or("")
                    .contains("Debtor's Prison"),
                "Expected Debtor's Prison sentence notice: {curse_out:?}"
            );
        }
    }

    // Ella clicks Sign Out -> `/logout` MUST NOT emit `Clear-Site-Data` (preserving seal cookie & storage)
    let (st_logout, logout_headers, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/logout",
        &[("Cookie", &format!("ewld_machine_seal={ella_seal}"))],
        Some(json!({ "token": ella_token })),
    )
    .await;
    assert_eq!(st_logout, 200);
    assert!(
        !logout_headers
            .to_ascii_lowercase()
            .contains("clear-site-data"),
        "Logout must never send Clear-Site-Data or cheaters could wipe machine seals by signing out!"
    );

    // 2. Scenario A: Ella tries signing into friend `maya`'s account on the same browser (seal cookie present) -> 403!
    let cookie_hdr = format!("ewld_machine_seal={ella_seal}");
    let (st_swap_seal, _, swap_seal_err) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            ("Cookie", &cookie_hdr),
            (
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/537.36 Chrome/126.0",
            ),
        ],
        Some(json!({
            "username": "maya",
            "password": "changeme-puppy",
            "clientId": "evade_1"
        })),
    )
    .await;
    assert_eq!(st_swap_seal, 403);
    assert!(
        swap_seal_err["error"]
            .as_str()
            .unwrap_or("")
            .contains("@ella"),
        "403 lock message must name penalized owner @ella: {:?}",
        swap_seal_err
    );

    // Verify Maya's account was NOT penalized or contaminated by Ella's evasion attempt!
    {
        let guard = running.world.users_cfg.read().await;
        let maya_u = guard.users.iter().find(|u| u.username == "maya").unwrap();
        assert!(!maya_u.in_debtors_prison);
        assert_eq!(maya_u.violation_strikes, 0);
        assert!(maya_u.coins >= 0);
    }

    // 3. Scenario B: Incognito Window (NO cookie, NO machineSeal in body, same exact hardware telemetry) -> 403!
    let (st_incog, _, incog_err) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            (
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/537.36 Chrome/126.0",
            ),
        ],
        Some(json!({
            "username": "maya",
            "password": "changeme-puppy",
            "clientId": "incog_tab",
            "deviceTelemetry": {
                "gpuVendor": "Google Inc. (Apple)",
                "gpuRenderer": "ANGLE (Apple, ANGLE Metal Renderer: Apple M2, Unspecified Version)",
                "glCaps": "16384|16384|16384|16|4096|1024|32|16384x16384",
                "canvasHash": "e11a88aa00112233",
                "audioHash": "e11a77bb44556677",
                "screenW": 1512,
                "screenH": 982,
                "colorDepth": 30,
                "pixelRatioX100": 200,
                "cpuCores": 8,
                "deviceMemoryGb": 8,
                "maxTouchPoints": 0,
                "osFamily": "macOS",
                "timezone": "America/Los_Angeles",
                "language": "en-US"
            }
        })),
    )
    .await;
    assert_eq!(
        st_incog, 403,
        "Incognito alt-account attempt must be blocked: {incog_err:?}"
    );

    // 4. Scenario C: Alternate Browser on Same Machine (Chrome -> Safari: different canvas/audio/UA, same Apple M2 + screen + subnet) -> 403!
    let (st_safari, _, safari_err) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 Version/17.5 Safari/605.1.15"),
        ],
        Some(json!({
            "username": "maya",
            "password": "changeme-puppy",
            "clientId": "safari_tab",
            "deviceTelemetry": {
                "gpuVendor": "Apple Inc.",
                "gpuRenderer": "Apple M2",
                "glCaps": "16384|16384|16384|16|1024|1024|16|16384x16384",
                "canvasHash": "safari_different_canvas_99",
                "audioHash": "safari_different_audio_88",
                "screenW": 1512,
                "screenH": 982,
                "colorDepth": 24,
                "pixelRatioX100": 200,
                "cpuCores": 8,
                "deviceMemoryGb": 0,
                "maxTouchPoints": 0,
                "osFamily": "MacIntel",
                "timezone": "America/Los_Angeles",
                "language": "en-US"
            }
        })),
    )
    .await;
    assert_eq!(
        st_safari, 403,
        "Cross-browser Chrome->Safari evasion on same physical machine must be blocked: {safari_err:?}"
    );

    // Meanwhile, Ella CAN still log back into her own account (`@ella`) on Machine A to work off her prison chores!
    let (st_ella_return, _, ella_return) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            ("Cookie", &cookie_hdr),
        ],
        Some(json!({
            "username": "ella",
            "password": "changeme-starlight",
            "clientId": "ella_serve_sentence"
        })),
    )
    .await;
    assert_eq!(st_ella_return, 200);
    assert_eq!(ella_return["self"]["zone"], 2);

    // 5. Scenario D: Genuine Separate Sibling Devices on Same Home LAN Allowed (`200 OK`)!
    // 5a: Distinct Windows laptop (`192.168.11.88`) for Maya -> 200 OK
    let (st_maya_win, _, maya_win) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.88"),
            (
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/126.0",
            ),
        ],
        Some(json!({
            "username": "maya",
            "password": "changeme-puppy",
            "clientId": "maya_own_pc",
            "deviceTelemetry": {
                "gpuVendor": "Google Inc. (Intel)",
                "gpuRenderer": "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics)",
                "glCaps": "16384|16384|16384|16|4096|1024|32|16384x16384",
                "canvasHash": "win_intel_canvas_01",
                "audioHash": "win_intel_audio_01",
                "screenW": 1920,
                "screenH": 1080,
                "colorDepth": 24,
                "pixelRatioX100": 125,
                "cpuCores": 12,
                "deviceMemoryGb": 16,
                "maxTouchPoints": 0,
                "osFamily": "Windows",
                "timezone": "America/Los_Angeles",
                "language": "en-US"
            }
        })),
    )
    .await;
    assert_eq!(
        st_maya_win, 200,
        "Friend on distinct PC must log in cleanly: {maya_win:?}"
    );

    // 5b: Sibling Chloe on her own identical-model M2 MacBook holding her distinct `ewld_machine_seal` -> 200 OK!
    let chloe_cookie = format!("ewld_machine_seal={chloe_sibling_seal}");
    let (st_chloe_sib, _, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.56"),
            ("Cookie", &chloe_cookie),
            (
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/537.36 Chrome/126.0",
            ),
        ],
        Some(json!({
            "username": "chloe",
            "password": "changeme-rainbow",
            "clientId": "chloe_sib_ok",
            "deviceTelemetry": mac_m2_chrome_tel
        })),
    )
    .await;
    assert_eq!(
        st_chloe_sib, 200,
        "Sibling with distinct valid ewld_machine_seal must never false-collide with Ella's locked MacBook"
    );

    // 6. Authenticate Parent Admin -> verify evasion audit log & devices dashboard
    let (st_adm, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(st_adm, 200);
    let admin_token = adm_login["adminToken"].as_str().unwrap().to_string();
    let admin_auth = format!("Bearer {admin_token}");

    let evasions = adm_login["dashboard"]["recentDeviceEvasions"]
        .as_array()
        .expect("recentDeviceEvasions array");
    assert!(
        evasions.len() >= 3,
        "Expected >= 3 logged evasion attempts (seal, hw_exact, hw_xbrowser), got {}",
        evasions.len()
    );
    let methods: Vec<&str> = evasions
        .iter()
        .filter_map(|e| e["matchMethod"].as_str())
        .collect();
    assert!(
        methods.contains(&"machine_seal_cookie"),
        "methods: {methods:?}"
    );
    assert!(
        methods.contains(&"hardware_exact_sig"),
        "methods: {methods:?}"
    );
    assert!(
        methods.contains(&"hardware_cross_browser_sig"),
        "methods: {methods:?}"
    );

    let ella_device_id = adm_login["dashboard"]["devices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["shortSig"] == ella_short_sig)
        .and_then(|d| d["deviceId"].as_str())
        .expect("Ella device record in admin dashboard")
        .to_string();

    // 7. Scenario E: Pardon Ella -> Machine A unlocks -> Maya logs in & opens WS tab on Machine A ->
    // Ella gets paused by Parent -> Maya's live WS tab on Machine A is IMMEDIATELY evicted!
    let (st_pardon, _, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &[("Authorization", &admin_auth)],
        Some(json!({ "username": "ella", "action": "pardon_prison" })),
    )
    .await;
    assert_eq!(st_pardon, 200);

    let (st_maya_m1, _, maya_m1) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            ("Cookie", &cookie_hdr),
        ],
        Some(json!({
            "username": "maya",
            "password": "changeme-puppy",
            "clientId": "maya_on_ella_mac"
        })),
    )
    .await;
    assert_eq!(
        st_maya_m1, 200,
        "After Ella is pardoned, Machine A is unlocked"
    );
    let maya_m1_token = maya_m1["token"].as_str().unwrap().to_string();

    let (mut ws_maya_m1, _) = tokio_tungstenite::connect_async(format!(
        "ws://{}/api/ella-world/ws?token={}",
        running.http_addr, maya_m1_token
    ))
    .await
    .expect("connect Maya WS on Machine A");
    // Drain initial welcome_snapshot
    let _ = tokio::time::timeout(Duration::from_secs(2), ws_maya_m1.next()).await;

    // Now Parent sentences Ella back to Debtor's Prison -> Maya's live WS session on Machine A MUST be revoked!
    let (st_re_sentence, _, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &[("Authorization", &admin_auth)],
        Some(json!({
            "username": "ella",
            "action": "sentence_prison",
            "prisonTasks": 3,
            "reason": "Habitual violation test"
        })),
    )
    .await;
    assert_eq!(st_re_sentence, 200);

    let mut saw_revoked = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(400), ws_maya_m1.next()).await {
            Ok(Some(Ok(TungsMessage::Text(txt)))) => {
                if txt.contains("session_revoked") {
                    saw_revoked = true;
                    break;
                }
            }
            Ok(Some(Ok(TungsMessage::Close(_)))) | Ok(None) => {
                saw_revoked = true;
                break;
            }
            _ => {}
        }
    }
    assert!(
        saw_revoked,
        "Maya's live WebSocket tab on Ella's penalized machine must be immediately evicted!"
    );

    // Maya's saved token also cannot `/resume` on Machine A (`403 Forbidden`)
    let (st_maya_resume_blocked, _, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/resume",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            ("Cookie", &cookie_hdr),
        ],
        Some(json!({
            "token": maya_m1_token,
            "clientId": "maya_on_ella_mac"
        })),
    )
    .await;
    assert_eq!(st_maya_resume_blocked, 403);

    // 8. Scenario F: Parent toggles `trustedShared: true` on Machine A -> Maya CAN sign in on shared family PC!
    let (st_trust, _, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/devices",
        &[("Authorization", &admin_auth)],
        Some(json!({
            "deviceId": ella_device_id,
            "action": "update",
            "trustedShared": true
        })),
    )
    .await;
    assert_eq!(st_trust, 200);

    let (st_maya_shared_ok, _, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            ("Cookie", &cookie_hdr),
        ],
        Some(json!({
            "username": "maya",
            "password": "changeme-puppy",
            "clientId": "maya_shared_pc"
        })),
    )
    .await;
    assert_eq!(
        st_maya_shared_ok, 200,
        "Trusted Shared Family PC must allow sibling login even while Ella is in Debtor's Prison"
    );

    // 9. Hard Machine Ban (`banned: true`) blocks all logins & evicts sessions on Machine A
    let (st_ban, _, ban_out) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/devices",
        &[("Authorization", &admin_auth)],
        Some(json!({
            "deviceId": ella_device_id,
            "action": "update",
            "banned": true
        })),
    )
    .await;
    assert_eq!(st_ban, 200);
    assert!(ban_out["kickedCount"].as_u64().unwrap_or(0) >= 1);

    let (st_banned_login, _, _) = http_json_request_custom_headers(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        &[
            ("X-Forwarded-For", "192.168.11.55"),
            ("Cookie", &cookie_hdr),
        ],
        Some(json!({
            "username": "ella",
            "password": "changeme-starlight",
            "clientId": "ella_banned_pc"
        })),
    )
    .await;
    assert_eq!(st_banned_login, 403);

    running.http_task.abort();
}

async fn http_get_text(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.expect("connect HTTP");
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).await.unwrap();
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
    (status, body_part.to_string())
}

#[tokio::test]
async fn glam_studio_modal_stage_and_wardrobe_persistence_contract() {
    let cfg = make_test_config("glam_studio_modal");
    let running = start_all_servers(cfg).await.expect("start server");

    // 1. Verify Top-Layer Modal Glam Studio HTML markup & accessibility contracts
    let (st_html, html) = http_get_text(running.http_addr, "/ella-world/index.html").await;
    assert_eq!(st_html, 200);
    assert!(
        html.contains(r#"<dialog id="avatar-dialog" class="glam-studio-modal" aria-labelledby="avatar-dialog-title">"#),
        "#avatar-dialog must be a true modal <dialog> without popover=\"manual\""
    );
    assert!(
        !html.contains(r#"id="avatar-dialog" class="glam-salon-popover""#),
        "Legacy non-modal glam-salon-popover must not remain on #avatar-dialog"
    );
    assert!(html.contains(r#"id="glam-stage-pane""#));
    assert!(html.contains(r#"id="glam-stage-canvas-wrap""#));
    assert!(html.contains(r#"id="glam-stage-toolbar""#));
    assert!(html.contains(r#"id="glam-pose-toolbar""#));
    assert!(html.contains(r#"id="glam-surprise-btn""#));
    assert!(html.contains(r#"id="glam-compare-btn""#));
    assert!(html.contains(r#"id="glam-runway-theme-pill""#));
    assert!(
        !html.contains(r#"id="glam-tab-looks" aria-controls="#),
        "Glam tab buttons must omit deprecated aria-controls per modern-web-guidance"
    );

    // 2. Verify CSS top-layer modal backdrop & responsive stage split rules
    let (st_css, css) = http_get_text(running.http_addr, "/ella-world/css/styles.css").await;
    assert_eq!(st_css, 200);
    assert!(css.contains("dialog.glam-studio-modal::backdrop"));
    assert!(css.contains(".glam-studio-layout"));
    assert!(css.contains(".glam-stage-pane"));

    // 3. Verify WebGL GlamSalonStage3D isolation & showModal() lifecycle in JS modules
    let (st_w3d, w3d_js) = http_get_text(running.http_addr, "/ella-world/js/world3d.js").await;
    assert_eq!(st_w3d, 200);
    assert!(w3d_js.contains("export class GlamSalonStage3D"));
    assert!(w3d_js.contains("cloneGeometries"));
    assert!(w3d_js.contains("studioMode"));

    let (st_app, app_js) = http_get_text(running.http_addr, "/ella-world/js/app.js").await;
    assert_eq!(st_app, 200);
    assert!(app_js.contains("avatarDialog.showModal()"));
    assert!(app_js.contains("avatarDialog.returnValue = ''"));
    assert!(app_js.contains("randomizeGlamOutfit"));
    assert!(app_js.contains("toggleGlamCompare"));

    // 4. Verify end-to-end outfit customization, male/female build switching, revert & /resume persistence
    let (st_login, login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "ella",
            "password": "changeme-starlight",
            "clientId": "glam_it_1"
        })),
    )
    .await;
    assert_eq!(st_login, 200);
    let token = login["token"].as_str().unwrap().to_string();
    let original_avatar = login["user"]["avatar"].clone();

    // Equip custom Lavender Coquette look
    let (st_custom, custom_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": token,
            "avatar": {
                "avatarMode": "stylized",
                "bodyType": "female",
                "characterPreset": "custom",
                "hairStyle": "amy_waves",
                "hairColor": "#78350f",
                "topStyle": "puff_blouse",
                "topColor": "#c084fc",
                "bottomStyle": "pleated_skirt",
                "bottomColor": "#f8fafc",
                "shoeStyle": "platform_boots",
                "shoeColor": "#a855f7",
                "hat": "beret",
                "hatColor": "#f472b6",
                "accessory": "star_purse",
                "accessoryColor": "#fde047",
                "fabricFinish": "glitter"
            }
        })),
    )
    .await;
    assert_eq!(st_custom, 200);
    assert_eq!(custom_res["user"]["avatar"]["topStyle"], "puff_blouse");
    assert_eq!(custom_res["user"]["avatar"]["bottomStyle"], "pleated_skirt");
    assert_eq!(custom_res["user"]["avatar"]["fabricFinish"], "glitter");

    // Verify /resume restores the saved custom outfit
    let (st_resume, resume_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/resume",
        Some(json!({ "token": token, "clientId": "glam_it_1" })),
    )
    .await;
    assert_eq!(st_resume, 200);
    assert_eq!(resume_res["user"]["avatar"]["topStyle"], "puff_blouse");
    assert_eq!(resume_res["user"]["avatar"]["hat"], "beret");

    // Verify Undo Changes revert back to salonOriginalAvatar persists cleanly
    let (st_revert, revert_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": token,
            "avatar": original_avatar
        })),
    )
    .await;
    assert_eq!(st_revert, 200);
    assert_eq!(
        revert_res["user"]["avatar"]["topStyle"],
        original_avatar["topStyle"]
    );

    running.http_task.abort();
}

#[tokio::test]
async fn subadmin_role_can_moderate_friends_only_and_cannot_bypass_parent_locks() {
    let cfg = make_test_config("subadmin_role");
    let running = start_all_servers(cfg).await.expect("start server");

    // 1. Parent admin promotes Ella to role="subadmin"
    let (st_adm, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(st_adm, 200);
    let admin_token = adm_login["adminToken"].as_str().unwrap().to_string();

    let (st_promo, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({
            "username": "ella",
            "action": "update_controls",
            "role": "subadmin"
        })),
    )
    .await;
    assert_eq!(st_promo, 200);

    // 2. Ella & Maya log in; Ella sees role="subadmin"
    let (st_e, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(
            json!({ "username": "ella", "password": "changeme-starlight", "clientId": "sub_ella" }),
        ),
    )
    .await;
    assert_eq!(st_e, 200);
    assert_eq!(ella_login["self"]["user"]["role"], "subadmin");
    let ella_token = ella_login["token"].as_str().unwrap().to_string();

    let (st_m, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "sub_maya" })),
    )
    .await;
    assert_eq!(st_m, 200);
    let maya_token = maya_login["token"].as_str().unwrap().to_string();

    // 3. Regular friend (Maya) is forbidden from /subadmin/state
    let (st_maya_sub, _) = http_json_auth_request(
        running.http_addr,
        "GET",
        "/api/ella-world/subadmin/state",
        &maya_token,
        None,
    )
    .await;
    assert_eq!(st_maya_sub, 403);

    // 4. Sub-Admin Ella can fetch friend list & mute Maya's mic + set quick_only chat + lock build
    let (st_state, sub_state) = http_json_auth_request(
        running.http_addr,
        "GET",
        "/api/ella-world/subadmin/state",
        &ella_token,
        None,
    )
    .await;
    assert_eq!(st_state, 200);
    let friends = sub_state["friends"].as_array().unwrap();
    assert!(friends.iter().any(|f| f["username"] == "maya"));

    let (st_mod, mod_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/subadmin/moderate",
        &ella_token,
        Some(json!({
            "username": "maya",
            "action": "update_controls",
            "voiceMutedByParent": true,
            "chatMode": "quick_only",
            "canBuild": false,
            "canTrade": false
        })),
    )
    .await;
    assert_eq!(st_mod, 200);
    assert_eq!(mod_res["moderation"]["voiceMutedByParent"], true);
    assert_eq!(mod_res["moderation"]["chatMode"], "quick_only");
    assert_eq!(mod_res["moderation"]["canBuild"], false);
    assert_eq!(mod_res["moderation"]["canTrade"], false);

    // 5. Sub-Admin Ella CANNOT self-moderate or access parent /admin/state
    let (st_self, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/subadmin/moderate",
        &ella_token,
        Some(json!({
            "username": "ella",
            "action": "update_controls",
            "voiceMutedByParent": false
        })),
    )
    .await;
    assert_eq!(st_self, 403);

    let (st_parent_api, _) = http_json_auth_request(
        running.http_addr,
        "GET",
        "/api/ella-world/admin/state",
        &ella_token,
        None,
    )
    .await;
    assert_eq!(st_parent_api, 401);

    // 5b. Sub-Admin Ella can manage Maya's gold, inventory items (gift/clone/remove), displayName & avatar
    let (st_sub_cust, sub_cust_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/subadmin/moderate",
        &ella_token,
        Some(json!({
            "username": "maya",
            "action": "update_controls",
            "setCoins": 250,
            "displayName": "Maya VIP",
            "avatar": { "characterPreset": "katey", "topColor": "#c084fc" }
        })),
    )
    .await;
    assert_eq!(st_sub_cust, 200);
    assert_eq!(sub_cust_res["economy"]["coins"], 250);

    let (st_sub_gift, sub_gift_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/subadmin/moderate",
        &ella_token,
        Some(json!({
            "username": "maya",
            "action": "gift_item",
            "itemId": "parent_golden_trophy",
            "qty": 2,
            "allowClone": true
        })),
    )
    .await;
    assert_eq!(st_sub_gift, 200);
    assert_eq!(
        sub_gift_res["economy"]["inventory"]["parent_golden_trophy"],
        2
    );

    let (st_sub_rm, sub_rm_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/subadmin/moderate",
        &ella_token,
        Some(json!({
            "username": "maya",
            "action": "gift_item",
            "itemId": "parent_golden_trophy",
            "setQty": 0
        })),
    )
    .await;
    assert_eq!(st_sub_rm, 200);
    assert!(sub_rm_res["economy"]["inventory"]["parent_golden_trophy"].is_null());

    // 6. Sub-Admin Ella can kick Maya's live session (invalidating Maya's active session)
    let (st_kick, kick_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/subadmin/moderate",
        &ella_token,
        Some(json!({
            "username": "maya",
            "action": "kick_session"
        })),
    )
    .await;
    assert_eq!(st_kick, 200);
    assert_eq!(kick_res["kickedPeers"].as_array().unwrap().len(), 1);

    // 7. If Ella is sentenced to Debtor's Prison, her Sub-Admin tools lock immediately (403)
    let (st_pris, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({
            "username": "ella",
            "action": "sentence_prison",
            "prisonTasks": 2
        })),
    )
    .await;
    assert_eq!(st_pris, 200);

    let (st_locked, _) = http_json_auth_request(
        running.http_addr,
        "GET",
        "/api/ella-world/subadmin/state",
        &ella_token,
        None,
    )
    .await;
    assert_eq!(st_locked, 403);

    running.http_task.abort();
}

#[tokio::test]
async fn parent_exclusive_item_gift_and_authorized_cloning_end_to_end() {
    let cfg = make_test_config("parent_gift_clone");
    let running = start_all_servers(cfg).await.expect("start server");

    let (st_adm, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(st_adm, 200);
    let admin_token = adm_login["adminToken"].as_str().unwrap().to_string();

    let (st_e, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "gift_ella" })),
    )
    .await;
    assert_eq!(st_e, 200);
    let ella_token = ella_login["token"].as_str().unwrap().to_string();

    // 1. Cannot clone before parent grants item + allowClone permission
    let (st_pre, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/economy/clone",
        Some(json!({ "token": ella_token, "itemId": "parent_golden_trophy" })),
    )
    .await;
    assert_eq!(st_pre, 400);

    // 2. Parent gifts 1x Parent-Exclusive Golden Star Trophy with allowClone: true
    let (st_gift, gift_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/gift-item",
        &admin_token,
        Some(json!({
            "username": "ella",
            "itemId": "parent_golden_trophy",
            "qty": 1,
            "allowClone": true
        })),
    )
    .await;
    assert_eq!(st_gift, 200);
    assert_eq!(gift_res["qtyGranted"], 1);
    assert_eq!(gift_res["allowClone"], true);
    assert_eq!(gift_res["economy"]["inventory"]["parent_golden_trophy"], 1);

    // 3. Ella clones her gifted trophy -> quantity becomes 2
    let (st_clone, clone_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/economy/clone",
        Some(json!({ "token": ella_token, "itemId": "parent_golden_trophy" })),
    )
    .await;
    assert_eq!(st_clone, 200);
    assert_eq!(clone_res["newQty"], 2);
    assert_eq!(clone_res["economy"]["inventory"]["parent_golden_trophy"], 2);

    // 4. Parent revokes clone permission -> further clone attempts return 403 Forbidden
    let (st_rev, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/gift-item",
        &admin_token,
        Some(json!({
            "username": "ella",
            "itemId": "parent_golden_trophy",
            "qty": 0,
            "allowClone": false
        })),
    )
    .await;
    assert_eq!(st_rev, 200);

    let (st_blocked, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/economy/clone",
        Some(json!({ "token": ella_token, "itemId": "parent_golden_trophy" })),
    )
    .await;
    assert_eq!(st_blocked, 403);

    running.http_task.abort();
}

#[tokio::test]
async fn expanded_plaza_double_doors_and_pro_obby_gauntlet_end_to_end() {
    let cfg = make_test_config("expanded_plaza_doors_gauntlet");
    let running = start_all_servers(cfg).await.expect("start server");

    let (st_login, login_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "ella",
            "password": "changeme-starlight",
            "clientId": "peer_ella_map210"
        })),
    )
    .await;
    assert_eq!(st_login, 200);
    let token = login_res["token"].as_str().unwrap().to_string();
    let initial_coins = login_res["economy"]["coins"].as_i64().unwrap_or(0);

    // 1. Expanded 210x210 Plaza build bounds (-100..=100, y 1..=80)
    let (st_outer_ok, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "place",
            "x": 92,
            "y": 1,
            "z": -88,
            "color": "#f472b6",
            "kind": "block"
        })),
    )
    .await;
    assert_eq!(st_outer_ok, 200, "Outer 210x210 plot coords must succeed");

    let (st_tall_ok, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "place",
            "x": 70,
            "y": 72,
            "z": 70,
            "color": "#38bdf8",
            "kind": "block"
        })),
    )
    .await;
    assert_eq!(st_tall_ok, 200, "Expanded y=72 height must succeed");

    let (st_oob_x, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "place",
            "x": 101,
            "y": 1,
            "z": 0,
            "color": "#38bdf8",
            "kind": "block"
        })),
    )
    .await;
    assert_eq!(st_oob_x, 400, "x=101 must be rejected outside -100..=100");

    let (st_oob_y, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "place",
            "x": 0,
            "y": 81,
            "z": 0,
            "color": "#38bdf8",
            "kind": "block"
        })),
    )
    .await;
    assert_eq!(st_oob_y, 400, "y=81 must be rejected outside 1..=80");

    // 2. Walkable 2x2 Double Door footprint, partner-cell toggle, edge-bounds guard, and partner-cell remove
    let (st_dd_edge, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "place",
            "x": 100,
            "y": 1,
            "z": 0,
            "color": "#fbbf24",
            "kind": "double_door",
            "rot": 0
        })),
    )
    .await;
    assert_eq!(
        st_dd_edge, 400,
        "double_door whose partner cell (101,1,0) crosses map edge must be rejected"
    );

    let (st_dd_place, dd_place_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "place",
            "x": 52,
            "y": 1,
            "z": -26,
            "color": "#ec4899",
            "kind": "double_door",
            "rot": 0
        })),
    )
    .await;
    assert_eq!(st_dd_place, 200);
    assert!(
        dd_place_res["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["x"] == 52
                && b["y"] == 1
                && b["z"] == -26
                && b["kind"] == "double_door"
                && b["open"] == false),
        "Placed double_door should start closed"
    );

    // Toggle via partner upper cell (53, 2, -26)
    let (st_dd_toggle, dd_toggle_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "toggle",
            "x": 53,
            "y": 2,
            "z": -26
        })),
    )
    .await;
    assert_eq!(
        st_dd_toggle, 200,
        "Toggling double_door via partner upper cell must succeed"
    );
    assert!(
        dd_toggle_res["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["x"] == 52
                && b["y"] == 1
                && b["z"] == -26
                && b["kind"] == "double_door"
                && b["open"] == true),
        "double_door must swing open when toggled from partner cell"
    );

    // Remove via partner lower cell (53, 1, -26)
    let (st_dd_rem, dd_rem_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "remove",
            "x": 53,
            "y": 1,
            "z": -26
        })),
    )
    .await;
    assert_eq!(st_dd_rem, 200);
    assert!(
        !dd_rem_res["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["x"] == 52 && b["y"] == 1 && b["z"] == -26 && b["kind"] == "double_door"),
        "Removing via partner cell must remove the entire double_door"
    );

    // 3. Starlight Sky Gauntlet (Pro Obby) +25 coin win reward & 60s anti-farm cooldown
    let (st_win1, win1_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({
            "token": token,
            "isTrophyWin": true,
            "trophyTier": "hard"
        })),
    )
    .await;
    assert_eq!(st_win1, 200);
    assert_eq!(win1_res["coinsAwarded"], 25);
    assert!(
        win1_res["message"]["text"]
            .as_str()
            .unwrap_or("")
            .contains("Starlight Sky Gauntlet"),
        "Hard obby win must announce Starlight Sky Gauntlet"
    );

    let (st_win2, win2_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/chat",
        Some(json!({
            "token": token,
            "isTrophyWin": true,
            "trophyTier": "hard"
        })),
    )
    .await;
    assert_eq!(st_win2, 200);
    assert_eq!(
        win2_res["coinsAwarded"], 0,
        "Immediate repeat hard obby win must not double-award coins during 60s cooldown"
    );

    let final_coins = {
        let cfg = running.world.users_cfg.read().await;
        cfg.users
            .iter()
            .find(|u| u.username == "ella")
            .unwrap()
            .coins
    };
    assert_eq!(
        final_coins,
        initial_coins + 25,
        "Persistent balance must increase by exactly +25 coins"
    );

    running.http_task.abort();
}

#[tokio::test]
async fn prebuilt_village_homes_and_shadow_rift_mortal_arena_respawn_end_to_end() {
    use ella_world::udp_wire::ZONE_ARENA;

    // 1. Verify legacy world_blocks.json without `prebuiltVillageV1Seeded` preserves player blocks AND seeds all 3 homes
    let cfg = make_test_config("prebuilt_village_and_arena");
    std::fs::create_dir_all(&cfg.data_dir).expect("create test data_dir");
    let custom_block_json = json!({
        "version": 7,
        "blocks": [
            {
                "x": 52,
                "y": 1,
                "z": -26,
                "color": "#a855f7",
                "author": "ella",
                "kind": "block",
                "rot": 0,
                "open": false
            }
        ]
    });
    std::fs::write(
        cfg.data_dir.join("world_blocks.json"),
        serde_json::to_vec_pretty(&custom_block_json).unwrap(),
    )
    .expect("seed legacy world_blocks.json");

    let running = start_all_servers(cfg).await.expect("start server");

    let (st_login, login_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "ella",
            "password": "changeme-starlight",
            "clientId": "peer_ella_arena"
        })),
    )
    .await;
    assert_eq!(st_login, 200);
    let token = login_res["token"].as_str().unwrap().to_string();
    let self_id = login_res["selfId"].as_str().unwrap().to_string();
    let blocks = login_res["blocks"].as_array().expect("blocks array");

    // Existing custom block on Plot #1 (52, 1, -26) must be preserved
    assert!(
        blocks
            .iter()
            .any(|b| b["x"] == 52 && b["y"] == 1 && b["z"] == -26 && b["color"] == "#a855f7"),
        "Existing player block must be preserved during prebuilt village seeding"
    );
    // All 3 Prebuilt Homes must be seeded: Rosewater Villa (81, 1, -22 double_door), Starlight Cottage (52, 1, 17 door), Woodland Chalet (-65, 1, -13 double_door)
    assert!(
        blocks
            .iter()
            .any(|b| b["x"] == 81 && b["y"] == 1 && b["z"] == -22 && b["kind"] == "double_door"),
        "Rosewater Grand Villa double_door must be seeded at Plot #2"
    );
    assert!(
        blocks
            .iter()
            .any(|b| b["x"] == 52 && b["y"] == 1 && b["z"] == 17 && b["kind"] == "door"),
        "Starlight Beach Cottage door must be seeded at Plot #3"
    );
    assert!(
        blocks
            .iter()
            .any(|b| b["x"] == -65 && b["y"] == 1 && b["z"] == -13 && b["kind"] == "double_door"),
        "Whispering Pine Woodland Chalet double_door must be seeded at Plot #5"
    );

    // Toggle Rosewater Villa front double door open via partner cell (82, 1, -22)
    let (st_toggle, toggle_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": token,
            "action": "toggle",
            "x": 82,
            "y": 1,
            "z": -22
        })),
    )
    .await;
    assert_eq!(st_toggle, 200);
    assert!(
        toggle_res["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["x"] == 81 && b["y"] == 1 && b["z"] == -22 && b["open"] == true),
        "Prebuilt Rosewater Villa front double_door must swing open"
    );

    // 2. Enter Shadow Rift Mortal Arena (`ZONE_ARENA = 3`)
    let (st_portal, portal_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({
            "token": token,
            "zone": ZONE_ARENA
        })),
    )
    .await;
    assert_eq!(st_portal, 200);
    assert_eq!(portal_res["zone"], ZONE_ARENA);
    assert_eq!(portal_res["arenaStatus"]["hp"], 100);
    assert_eq!(portal_res["arenaStatus"]["alive"], true);

    let arena_mobs = portal_res["arena"]["mobs"].as_array().expect("arena mobs");
    assert!(arena_mobs.iter().any(|m| m["kind"] == "rift_spider"));
    assert!(arena_mobs.iter().any(|m| m["kind"] == "void_wraith"));
    assert!(arena_mobs.iter().any(|m| m["kind"] == "lava_golem"));

    // Move Ella near `spider_1` (-9.5, 0.0, 10.5) over binary C2S state and defeat it
    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 1,
                client_time_ms: 100,
                x: -9.5,
                y: 0.0,
                z: 11.2,
                ry: 0.0,
                anim: "run".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: ZONE_ARENA,
            },
        )
        .await;

    let (st_hit1, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": token, "mobId": "spider_1", "mode": "stomp" })),
    )
    .await;
    assert_eq!(st_hit1, 200);
    tokio::time::sleep(Duration::from_millis(180)).await;

    let (st_hit2, hit2_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": token, "mobId": "spider_1", "mode": "stomp" })),
    )
    .await;
    assert_eq!(st_hit2, 200);
    assert_eq!(hit2_res["defeated"], true);
    let drop_id = hit2_res["drop"]["id"].as_str().unwrap().to_string();

    // 3. Expire spawn shield & plunge into the Molten Abyss (`y = -3.0`) -> Ella dies (`hp == 0`, `alive == false`)!
    {
        let mut ar = running.world.arena.write().await;
        if let Some(st) = ar.player_statuses.get_mut(&self_id) {
            st.invuln_until_ms = 0;
        }
    }
    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 2,
                client_time_ms: 350,
                x: -9.5,
                y: -3.0,
                z: -5.0,
                ry: 0.0,
                anim: "jump".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: ZONE_ARENA,
            },
        )
        .await;
    running.world.tick_arena_if_occupied().await;

    let dead_status = running.world.arena.read().await.status_for(&self_id);
    assert_eq!(dead_status.hp, 0);
    assert!(!dead_status.alive, "Lethal lava fall must mark player dead");
    assert_eq!(dead_status.defeated_by.as_deref(), Some("the Molten Abyss"));

    // Attacking or picking up loot while dead must fail with 400 until Respawn!
    let (st_dead_atk, dead_atk_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": token, "mobId": "spider_2", "mode": "blast" })),
    )
    .await;
    assert_eq!(st_dead_atk, 400);
    assert!(
        dead_atk_res["error"]
            .as_str()
            .unwrap_or("")
            .contains("Respawn"),
        "Dead player must be required to Respawn before attacking"
    );

    let (st_dead_pick, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/pickup",
        Some(json!({ "token": token, "dropId": drop_id })),
    )
    .await;
    assert_eq!(st_dead_pick, 400);

    // 4. Respawn at Sanctuary (`POST /api/ella-world/arena/respawn`) -> restores 100 HP at (0, 0, 25)!
    let (st_respawn, respawn_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/arena/respawn",
        Some(json!({ "token": token })),
    )
    .await;
    assert_eq!(st_respawn, 200);
    assert_eq!(respawn_res["arenaStatus"]["hp"], 100);
    assert_eq!(respawn_res["arenaStatus"]["alive"], true);
    assert_eq!(respawn_res["z"], 25.0);

    // Walk back to `drop_id` and collect the Arena reward!
    let coins_before = {
        let cfg = running.world.users_cfg.read().await;
        cfg.users
            .iter()
            .find(|u| u.username == "ella")
            .unwrap()
            .coins
    };
    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 3,
                client_time_ms: 600,
                x: -9.5,
                y: 0.0,
                z: 10.5,
                ry: 0.0,
                anim: "run".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: ZONE_ARENA,
            },
        )
        .await;
    let (st_pick_ok, pick_ok_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/pickup",
        Some(json!({ "token": token, "dropId": drop_id })),
    )
    .await;
    assert_eq!(st_pick_ok, 200);
    assert!(pick_ok_res["economy"]["coins"].as_i64().unwrap() >= coins_before + 22);

    running.http_task.abort();
}

// ============================================================================
// Expansion Pack End-to-End Integration Tests (All 6 Upgrades + Edge Cases)
// ============================================================================

#[tokio::test]
async fn expansion_adoptable_pets_hoverboards_and_egg_hatching_end_to_end() {
    let cfg = make_test_config("exp_pets_mounts");
    let running = start_all_servers(cfg).await.expect("start server");

    let (st_adm, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(st_adm, 200);
    let admin_token = adm_login["adminToken"].as_str().unwrap().to_string();

    let (st_login, login_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "maya",
            "password": "changeme-puppy",
            "clientId": "maya_pets_1"
        })),
    )
    .await;
    assert_eq!(st_login, 200);
    let token = login_res["token"].as_str().unwrap().to_string();
    let self_id = login_res["selfId"].as_str().unwrap().to_string();

    // 1. Starter Cloud Puppy + Starlight Hoverboard unlocked out-of-the-box for everyone!
    let owned_pets = login_res["companions"]["ownedPets"].as_array().unwrap();
    let owned_mounts = login_res["companions"]["ownedMounts"].as_array().unwrap();
    assert!(owned_pets.iter().any(|v| v.as_str() == Some("cloud_puppy")));
    assert!(owned_mounts
        .iter()
        .any(|v| v.as_str() == Some("starlight_board")));
    assert_eq!(login_res["petCatalog"].as_array().unwrap().len(), 4);
    assert_eq!(login_res["mountCatalog"].as_array().unwrap().len(), 3);

    // 2. Equip starter pet & verify profanity filter blocks unkind pet nickname
    let (st_eq, eq_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": token, "action": "equip_pet", "petId": "cloud_puppy" })),
    )
    .await;
    assert_eq!(st_eq, 200);
    assert_eq!(eq_res["user"]["equippedPet"], "cloud_puppy");

    let (st_bad_nick, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": token, "action": "rename_pet", "nickname": "sh1t dog" })),
    )
    .await;
    assert_eq!(st_bad_nick, 400);

    let (st_good_nick, nick_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": token, "action": "rename_pet", "nickname": "Fluffy Cloud" })),
    )
    .await;
    assert_eq!(st_good_nick, 200);
    assert_eq!(nick_res["user"]["petNickname"], "Fluffy Cloud");

    // 3. Cannot adopt rare Starlight Kitsune without 180 coins
    let (st_poor, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": token, "action": "adopt_pet", "petId": "starlight_kitsune" })),
    )
    .await;
    assert_eq!(st_poor, 400);

    // Grant coins + 1x pet_egg via Parent Admin, then adopt Kitsune, buy Comet Surfer & hatch egg!
    let (st_grant, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({ "username": "maya", "action": "adjust_coins", "coinDelta": 500 })),
    )
    .await;
    assert_eq!(st_grant, 200);

    let (st_adopt, adopt_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": token, "action": "adopt_pet", "petId": "starlight_kitsune" })),
    )
    .await;
    assert_eq!(st_adopt, 200);
    assert_eq!(adopt_res["user"]["equippedPet"], "starlight_kitsune");
    assert_eq!(adopt_res["economy"]["coins"], 340);

    let (st_mount, mount_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": token, "action": "buy_mount", "mountId": "comet_surfer" })),
    )
    .await;
    assert_eq!(st_mount, 200);
    assert_eq!(mount_res["user"]["equippedMount"], "comet_surfer");
    assert_eq!(mount_res["economy"]["coins"], 20);

    // Gift & hatch a Starlight Companion Egg via `/economy/use`
    let (st_egg_gift, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/gift-item",
        &admin_token,
        Some(json!({ "username": "maya", "itemId": "pet_egg", "qty": 1, "allowClone": false })),
    )
    .await;
    assert_eq!(st_egg_gift, 200);

    let (st_hatch, hatch_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/economy/use",
        Some(json!({ "token": token, "itemId": "pet_egg" })),
    )
    .await;
    assert_eq!(st_hatch, 200);
    assert!(hatch_res["hatchedPetId"].as_str().is_some());
    assert!(hatch_res["economy"]["inventory"]
        .get("pet_egg")
        .and_then(Value::as_u64)
        .is_none());

    // 4. Verify UDP binary wire roundtrip for new `hover` anim (4) and `sleep` emote (9)
    let pkt = encode_c2s_state(&DecodedC2SState {
        seq: 7,
        client_time_ms: 900,
        x: 10.0,
        y: 0.5,
        z: -4.0,
        ry: 1.25,
        anim: "hover".into(),
        emote: Some("sleep".into()),
        muted: false,
        speaking: false,
        zone: 0,
    });
    let decoded = ella_world::udp_wire::decode_c2s_state(&pkt).expect("decode hover/sleep packet");
    assert_eq!(decoded.anim, "hover");
    assert_eq!(decoded.emote.as_deref(), Some("sleep"));
    running.world.apply_c2s_state(&self_id, decoded).await;

    running.http_task.abort();
}

#[tokio::test]
async fn expansion_claimable_homes_mailboxes_locks_cobuilders_and_doorbells_end_to_end() {
    let cfg = make_test_config("exp_homes_doorbells");
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "ella_h" })),
    )
    .await;
    let ella_token = ella_login["token"].as_str().unwrap().to_string();

    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "maya_h" })),
    )
    .await;
    let maya_token = maya_login["token"].as_str().unwrap().to_string();

    let (_, chloe_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "chloe", "password": "changeme-rainbow", "clientId": "chloe_h" })),
    )
    .await;
    let chloe_token = chloe_login["token"].as_str().unwrap().to_string();

    // 1. Maya claims Plot #2 (Rosewater Villa), then switches to Plot #3 -> Plot #2 auto-releases!
    let (st_c2, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({ "token": maya_token, "action": "claim", "plotId": "plot_2" })),
    )
    .await;
    assert_eq!(st_c2, 200);

    let (st_c3, c3_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({ "token": maya_token, "action": "claim", "plotId": "plot_3" })),
    )
    .await;
    assert_eq!(st_c3, 200);
    let homes = c3_res["homes"].as_array().unwrap();
    let p2 = homes.iter().find(|p| p["plotId"] == "plot_2").unwrap();
    let p3 = homes.iter().find(|p| p["plotId"] == "plot_3").unwrap();
    assert!(
        p2["ownerUsername"].is_null(),
        "1-plot-per-player rule must release plot_2"
    );
    assert_eq!(p3["ownerUsername"], "maya");

    // 2. Profanity in customTitle is rejected; clean title + doorLocked: true succeeds
    let (st_bad_title, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({
            "token": maya_token,
            "action": "update_settings",
            "plotId": "plot_3",
            "customTitle": "f u c k house"
        })),
    )
    .await;
    assert_eq!(st_bad_title, 400);

    let (st_lock, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({
            "token": maya_token,
            "action": "update_settings",
            "plotId": "plot_3",
            "customTitle": "Maya's Starlight Villa",
            "doorLocked": true
        })),
    )
    .await;
    assert_eq!(st_lock, 200);

    // 3. Chloe (non-owner) cannot place blocks OR open locked front door on Plot #3 (43..=61, 11..=29)
    let (st_chloe_place, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": chloe_token,
            "action": "place",
            "x": 45,
            "y": 1,
            "z": 13,
            "kind": "sofa",
            "color": "#f472b6",
            "rot": 0
        })),
    )
    .await;
    assert_eq!(st_chloe_place, 400);

    // Find an actual door block on Plot #3 (43..=61, 11..=29)
    let plot3_door = {
        let bd = running.world.blocks_data.read().await;
        bd.blocks
            .iter()
            .find(|b| {
                matches!(b.kind.as_str(), "door" | "double_door")
                    && b.x >= 43
                    && b.x <= 61
                    && b.z >= 11
                    && b.z <= 29
            })
            .cloned()
            .expect("Plot #3 prebuilt door")
    };
    let (st_chloe_door, door_err) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": chloe_token,
            "action": "toggle",
            "x": plot3_door.x,
            "y": plot3_door.y,
            "z": plot3_door.z
        })),
    )
    .await;
    assert_eq!(st_chloe_door, 400);
    assert!(door_err["error"]
        .as_str()
        .unwrap_or("")
        .contains("Doorbell"));

    // 4. Chloe rings the Doorbell (200 OK), and immediate spam ring is rate-limited (400)
    let (st_bell, bell_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({ "token": chloe_token, "action": "ring_doorbell", "plotId": "plot_3" })),
    )
    .await;
    assert_eq!(st_bell, 200);
    assert_eq!(bell_res["ringerName"], "Chloe");

    let (st_bell_spam, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({ "token": chloe_token, "action": "ring_doorbell", "plotId": "plot_3" })),
    )
    .await;
    assert_eq!(st_bell_spam, 400);

    // 5. Maya invites Chloe as coBuilder -> Chloe can now place `crystal_lamp` & toggle its light switch!
    let (st_cobuild, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({
            "token": maya_token,
            "action": "update_settings",
            "plotId": "plot_3",
            "coBuilders": ["chloe"]
        })),
    )
    .await;
    assert_eq!(st_cobuild, 200);

    let (st_lamp, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": chloe_token,
            "action": "place",
            "x": 45,
            "y": 1,
            "z": 13,
            "kind": "crystal_lamp",
            "color": "#a855f7",
            "rot": 0
        })),
    )
    .await;
    assert_eq!(st_lamp, 200);

    let (st_lamp_toggle, lamp_tog_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": chloe_token,
            "action": "toggle",
            "x": 45,
            "y": 1,
            "z": 13
        })),
    )
    .await;
    assert_eq!(st_lamp_toggle, 200);
    let lamp_block = lamp_tog_res["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["x"] == 45 && b["y"] == 1 && b["z"] == 13)
        .unwrap();
    assert_eq!(lamp_block["open"], true);

    // 6. Host Ella (`role == "host"`) retains moderation override even on Maya's claimed plot
    let (st_ella_override, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": ella_token,
            "action": "place",
            "x": 46,
            "y": 1,
            "z": 13,
            "kind": "sofa",
            "color": "#38bdf8",
            "rot": 2
        })),
    )
    .await;
    assert_eq!(st_ella_override, 200);

    running.http_task.abort();
}

#[tokio::test]
async fn expansion_crafting_bench_weapons_aegis_charm_and_potions_end_to_end() {
    use ella_world::udp_wire::ZONE_ARENA;

    let cfg = make_test_config("exp_crafting_gear");
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    let admin_token = adm_login["adminToken"].as_str().unwrap().to_string();

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "ella_craft" })),
    )
    .await;
    let token = ella_login["token"].as_str().unwrap().to_string();
    let self_id = ella_login["selfId"].as_str().unwrap().to_string();

    // 1. Crafting without materials fails cleanly with 400
    let (st_fail, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/craft",
        Some(json!({ "token": token, "recipeId": "craft_crystal_blade" })),
    )
    .await;
    assert_eq!(st_fail, 400);

    // 2. Grant coins + dungeon materials, then craft Crystal Blade, Starlight Wand, Aegis Charm & Potion!
    http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/moderate",
        &admin_token,
        Some(json!({ "username": "ella", "action": "adjust_coins", "coinDelta": 600 })),
    )
    .await;
    for (item_id, qty) in [
        ("star_shard", 25),
        ("slime_gel", 15),
        ("moon_opal", 12),
        ("crown_relic", 4),
    ] {
        let (st_g, _) = http_json_auth_request(
            running.http_addr,
            "POST",
            "/api/ella-world/admin/gift-item",
            &admin_token,
            Some(json!({ "username": "ella", "itemId": item_id, "qty": qty, "allowClone": false })),
        )
        .await;
        assert_eq!(st_g, 200);
    }

    for recipe_id in [
        "craft_crystal_blade",
        "craft_starlight_wand",
        "craft_aegis_charm",
        "craft_starlight_potion",
    ] {
        let (st_c, c_res) = http_json_request(
            running.http_addr,
            "POST",
            "/api/ella-world/craft",
            Some(json!({ "token": token, "recipeId": recipe_id })),
        )
        .await;
        assert_eq!(st_c, 200, "recipe {recipe_id} failed: {c_res}");
    }

    // 3. Equip Astral Starlight Wand in Glam Closet & enter Shadow Rift -> passive Aegis Charm gives 150 Max HP!
    let (st_av, av_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({ "token": token, "avatar": { "accessory": "starlight_wand" } })),
    )
    .await;
    assert_eq!(st_av, 200);
    assert_eq!(av_res["user"]["avatar"]["accessory"], "starlight_wand");

    let (st_portal, portal_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({ "token": token, "targetZone": ZONE_ARENA })),
    )
    .await;
    assert_eq!(st_portal, 200);
    assert_eq!(portal_res["arenaStatus"]["maxHp"], 150);
    assert_eq!(portal_res["arenaStatus"]["hp"], 150);

    // 4. Damage Ella down to 60 HP, fire ranged Starlight Wand at `spider_1` (12m away -> 2 DMG + +8 HP self-mend!), then drink Starlight Potion (+75 HP)!
    {
        let mut ar = running.world.arena.write().await;
        if let Some(st) = ar.player_statuses.get_mut(&self_id) {
            st.hp = 60;
        }
    }
    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 1,
                client_time_ms: 100,
                x: -9.5,
                y: 0.0,
                z: 22.0, // ~11.5m from spider_1 (-9.5, 0.0, 10.5) -> within 16.5m wand range!
                ry: 0.0,
                anim: "idle".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: ZONE_ARENA,
            },
        )
        .await;

    let (st_wand, wand_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": token, "mobId": "spider_1", "mode": "blast" })),
    )
    .await;
    assert_eq!(st_wand, 200);
    assert_eq!(wand_res["damage"], 2);
    let hp_after_wand = running.world.arena.read().await.status_for(&self_id).hp;
    assert_eq!(hp_after_wand, 68, "Starlight Wand must mend +8 HP per hit");

    let (st_pot, pot_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/economy/use",
        Some(json!({ "token": token, "itemId": "starlight_potion" })),
    )
    .await;
    assert_eq!(st_pot, 200);
    assert_eq!(pot_res["arenaStatus"]["hp"], 143); // 68 + 75 = 143 / 150

    running.http_task.abort();
}

#[tokio::test]
async fn expansion_timed_fashion_show_ai_judges_and_podium_prizes_end_to_end() {
    let cfg = make_test_config("exp_fashion_show");
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(
            json!({ "username": "ella", "password": "changeme-starlight", "clientId": "fs_ella" }),
        ),
    )
    .await;
    let ella_token = ella_login["token"].as_str().unwrap().to_string();

    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "fs_maya" })),
    )
    .await;
    let maya_self_id = maya_login["selfId"].as_str().unwrap().to_string();

    let (_, chloe_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(
            json!({ "username": "chloe", "password": "changeme-rainbow", "clientId": "fs_chloe" }),
        ),
    )
    .await;
    let chloe_token = chloe_login["token"].as_str().unwrap().to_string();

    // Place Maya far away building on Plot #1 (52, 0, -26) -> starting show MUST NOT teleport Maya!
    running
        .world
        .apply_c2s_state(
            &maya_self_id,
            DecodedC2SState {
                seq: 1,
                client_time_ms: 100,
                x: 52.0,
                y: 0.0,
                z: -26.0,
                ry: 0.0,
                anim: "idle".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;

    // 1. Ella starts Timed Fashion Show -> phase == "dress_up", Chloe opts in via `join_show`
    let (st_start, start_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/runway",
        Some(json!({ "token": ella_token, "action": "start_show" })),
    )
    .await;
    assert_eq!(st_start, 200);
    assert_eq!(start_res["runway"]["show"]["phase"], "dress_up");

    let maya_pos = {
        let map = running.world.players.read().await;
        let p = map.get(&maya_self_id).unwrap();
        (p.x, p.z)
    };
    assert_eq!(
        maya_pos,
        (52.0, -26.0),
        "Non-participating builders must NEVER be force-teleported"
    );

    let (st_join, join_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/runway",
        Some(json!({ "token": chloe_token, "action": "join_show" })),
    )
    .await;
    assert_eq!(st_join, 200);
    assert_eq!(
        join_res["runway"]["show"]["contestants"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // 2. Self-voting during active show is rejected; Ella votes 5★ for Chloe!
    let (st_self_vote, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/runway",
        Some(
            json!({ "token": ella_token, "action": "vote", "targetUsername": "ella", "stars": 5 }),
        ),
    )
    .await;
    assert_eq!(st_self_vote, 400);

    let (st_vote, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/runway",
        Some(
            json!({ "token": ella_token, "action": "vote", "targetUsername": "chloe", "stars": 5 }),
        ),
    )
    .await;
    assert_eq!(st_vote, 200);

    // 3. Advance `dress_up` -> `catwalk` (contestant 0 -> contestant 1) -> `podium`!
    {
        let mut rw = running.world.runway.write().await;
        rw.show.phase_ends_at = 1;
    }
    running.world.tick_fashion_show().await; // -> catwalk idx 0
    {
        let mut rw = running.world.runway.write().await;
        assert_eq!(rw.show.phase, "catwalk");
        assert_eq!(rw.show.spotlight_index, 0);
        rw.show.phase_ends_at = 1;
    }
    running.world.tick_fashion_show().await; // -> catwalk idx 1
    {
        let mut rw = running.world.runway.write().await;
        assert_eq!(rw.show.phase, "catwalk");
        assert_eq!(rw.show.spotlight_index, 1);
        rw.show.phase_ends_at = 1;
    }
    running.world.tick_fashion_show().await; // -> podium + AI Judges + coin prizes!

    let summary = running.world.runway.read().await.summary();
    assert_eq!(summary.show.phase, "podium");
    assert_eq!(summary.show.podium_winners.len(), 2);
    assert!(
        summary
            .show
            .podium_winners
            .iter()
            .all(|p| p.ai_judged && p.avg_stars >= 4.0),
        "AI Judges must supply friendly 4★-5★ ratings when < 3 human voters participated"
    );

    let (ella_coins, chloe_coins) = {
        let cfg = running.world.users_cfg.read().await;
        let ec = cfg
            .users
            .iter()
            .find(|u| u.username == "ella")
            .unwrap()
            .coins;
        let cc = cfg
            .users
            .iter()
            .find(|u| u.username == "chloe")
            .unwrap()
            .coins;
        (ec, cc)
    };
    assert_eq!(
        ella_coins + chloe_coins,
        100,
        "1st (+60) and 2nd (+40) Podium prizes must total 100 Starlight Coins"
    );

    running.http_task.abort();
}

#[tokio::test]
async fn expansion_obby_speedrun_gates_hoverboard_dismount_and_persistent_leaderboards() {
    let cfg = make_test_config("exp_obby_speedrun");
    let cfg_restart = cfg.clone();
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "obby_ella" })),
    )
    .await;
    let token = ella_login["token"].as_str().unwrap().to_string();
    let self_id = ella_login["selfId"].as_str().unwrap().to_string();

    // 1. Equip Starlight Hoverboard, stand at Sky Obby Start Gate `(0, 0.8, -13.5)`, and start run -> auto-dismounts hoverboard!
    let (st_mnt, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": token, "action": "equip_mount", "mountId": "starlight_board" })),
    )
    .await;
    assert_eq!(st_mnt, 200);

    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 1,
                client_time_ms: 100,
                x: 0.0,
                y: 0.8,
                z: -13.5,
                ry: 0.0,
                anim: "hover".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;

    let (st_start1, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/obby",
        Some(json!({ "token": token, "action": "start", "course": "easy" })),
    )
    .await;
    assert_eq!(st_start1, 200);
    let mount_after_start = {
        let cfg = running.world.users_cfg.read().await;
        cfg.users
            .iter()
            .find(|u| u.username == "ella")
            .unwrap()
            .equipped_mount
            .clone()
    };
    assert_eq!(
        mount_after_start, None,
        "Starting an Obby speedrun must auto-dismount hoverboards"
    );

    // 2. Cancelling timer (checkpoint warp `R`) invalidates finishing
    let (st_cancel, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/obby",
        Some(json!({ "token": token, "action": "cancel" })),
    )
    .await;
    assert_eq!(st_cancel, 200);

    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 2,
                client_time_ms: 200,
                x: 6.5,
                y: 17.0,
                z: -20.0,
                ry: 0.0,
                anim: "idle".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;
    let (st_fin_invalid, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/obby",
        Some(json!({ "token": token, "action": "finish", "course": "easy" })),
    )
    .await;
    assert_eq!(st_fin_invalid, 400);

    // 3. Valid timed run from Start Gate -> Finish Gate records Top 5 & persists to `world_leaderboards.json`!
    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 3,
                client_time_ms: 300,
                x: 0.0,
                y: 0.8,
                z: -13.5,
                ry: 0.0,
                anim: "run".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;
    let (st_start2, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/obby",
        Some(json!({ "token": token, "action": "start", "course": "easy" })),
    )
    .await;
    assert_eq!(st_start2, 200);

    // Backdate `started_at_ms` by 9.42s so we test realistic speedrun timing without sleeping
    {
        let mut runs = running.world.active_obby_runs.write().await;
        if let Some(r) = runs.get_mut(&self_id) {
            r.started_at_ms = r.started_at_ms.saturating_sub(9_420);
        }
    }
    running
        .world
        .apply_c2s_state(
            &self_id,
            DecodedC2SState {
                seq: 4,
                client_time_ms: 400,
                x: 6.5,
                y: 17.0,
                z: -20.0,
                ry: 0.0,
                anim: "jump".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;

    let (st_fin, fin_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/obby",
        Some(json!({ "token": token, "action": "finish", "course": "easy" })),
    )
    .await;
    assert_eq!(st_fin, 200);
    assert_eq!(fin_res["rank"], 1);
    assert_eq!(fin_res["improved"], true);
    assert_eq!(fin_res["awardedCoins"], 37); // 12 base + 25 #1 record bonus
    assert!(fin_res["timeFormatted"].as_str().unwrap().ends_with('s'));

    tokio::time::sleep(Duration::from_millis(60)).await;
    running.http_task.abort();

    // Reload WorldEngine from disk and verify `world_leaderboards.json` persisted Ella's #1 time!
    let reloaded = WorldEngine::new(cfg_restart).expect("reload WorldEngine");
    let lb = reloaded.obby_leaderboards.read().await.clone();
    assert_eq!(lb.easy.len(), 1);
    assert_eq!(lb.easy[0].username, "ella");
}

#[tokio::test]
async fn expansion_sky_meteor_shower_and_coop_obsidian_dragon_boss_end_to_end() {
    use ella_world::economy::BossShockwave;
    use ella_world::udp_wire::ZONE_ARENA;

    let cfg = make_test_config("exp_sky_and_boss");
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "boss_ella" })),
    )
    .await;
    let ella_token = ella_login["token"].as_str().unwrap().to_string();
    let ella_id = ella_login["selfId"].as_str().unwrap().to_string();

    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "boss_maya" })),
    )
    .await;
    let maya_token = maya_login["token"].as_str().unwrap().to_string();
    let maya_id = maya_login["selfId"].as_str().unwrap().to_string();

    // 1. Non-host Maya cannot control Sky/Meteor Shower (400); Host Ella CAN set Aurora Night & launch Meteor Shower!
    let (st_maya_sky, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/sky",
        Some(json!({ "token": maya_token, "action": "start_meteor_shower" })),
    )
    .await;
    assert_eq!(st_maya_sky, 400);

    let (st_ella_sky, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/sky",
        Some(json!({ "token": ella_token, "action": "set_sky", "mode": "aurora_night" })),
    )
    .await;
    assert_eq!(st_ella_sky, 200);

    let (st_meteor, meteor_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/sky",
        Some(json!({ "token": ella_token, "action": "start_meteor_shower" })),
    )
    .await;
    assert_eq!(st_meteor, 200);
    let stars = meteor_res["sky"]["fallingStars"].as_array().unwrap();
    assert!(!stars.is_empty());
    let first_star = &stars[0];
    let star_id = first_star["id"].as_str().unwrap().to_string();
    let sx = first_star["x"].as_f64().unwrap() as f32;
    let sz = first_star["z"].as_f64().unwrap() as f32;

    running
        .world
        .apply_c2s_state(
            &maya_id,
            DecodedC2SState {
                seq: 1,
                client_time_ms: 100,
                x: sx,
                y: 0.0,
                z: sz,
                ry: 0.0,
                anim: "run".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;
    let (st_catch, catch_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/sky",
        Some(json!({ "token": maya_token, "action": "catch_star", "starId": star_id })),
    )
    .await;
    assert_eq!(st_catch, 200);
    assert!(catch_res["coinsAwarded"].as_u64().unwrap() >= 16);

    // 2. Both Ella and Maya enter Shadow Rift (`ZONE_ARENA`) and approach North Island Altar `(0, 0, -24.5)`!
    for tok in [&ella_token, &maya_token] {
        let (st_p, _) = http_json_request(
            running.http_addr,
            "POST",
            "/api/ella-world/dungeon/portal",
            Some(json!({ "token": tok, "targetZone": ZONE_ARENA })),
        )
        .await;
        assert_eq!(st_p, 200);
    }

    // Expire Sanctuary spawn shields and place Ella grounded (`y = 0.0`) & Maya jumping (`y = 1.2`) at `z = -20.5`
    {
        let mut ar = running.world.arena.write().await;
        for pid in [&ella_id, &maya_id] {
            if let Some(st) = ar.player_statuses.get_mut(pid) {
                st.invuln_until_ms = 0;
            }
        }
    }
    running
        .world
        .apply_c2s_state(
            &ella_id,
            DecodedC2SState {
                seq: 2,
                client_time_ms: 200,
                x: 0.0,
                y: 0.0,
                z: -20.5,
                ry: 0.0,
                anim: "idle".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: ZONE_ARENA,
            },
        )
        .await;
    running
        .world
        .apply_c2s_state(
            &maya_id,
            DecodedC2SState {
                seq: 2,
                client_time_ms: 200,
                x: 0.0,
                y: 1.2, // Jumping cleanly over the ground shockwave ring (`y >= 0.65`)!
                z: -20.5,
                ry: 0.0,
                anim: "jump".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: ZONE_ARENA,
            },
        )
        .await;

    // Summon Co-Op Obsidian Dragon Boss -> 2 players in Arena scales HP to `18 + 12 = 30 HP`!
    let (st_boss, boss_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/arena/boss",
        Some(json!({ "token": ella_token, "useBeacon": false })),
    )
    .await;
    assert_eq!(st_boss, 200);
    assert_eq!(boss_res["boss"]["active"], true);
    assert_eq!(boss_res["boss"]["maxHp"], 30);

    // 3. Inject an expanding Shockwave Ring passing across `dist = 4.0` from `(0, -24.5)`:
    // Grounded Ella (`y = 0.0`) takes 18 DMG -> 82 HP; Jumping Maya (`y = 1.2`) dodges -> 100 HP!
    {
        let mut ar = running.world.arena.write().await;
        ar.boss.shockwaves.push(BossShockwave {
            id: 777,
            x: 0.0,
            z: -24.5,
            radius: 3.6,
            max_radius: 18.0,
            spawned_at_ms: 0,
        });
    }
    running.world.tick_arena_if_occupied().await;
    {
        let ar = running.world.arena.read().await;
        assert_eq!(
            ar.status_for(&ella_id).hp,
            82,
            "Grounded player must take 18 DMG from Obsidian Dragon Shockwave"
        );
        assert_eq!(
            ar.status_for(&maya_id).hp,
            100,
            "Jumping player (y >= 0.65) must dodge Obsidian Dragon Shockwave"
        );
    }

    // 4. Both Ella and Maya contribute hits and defeat the Boss -> BOTH get personal coins, rare loot & first-kill `pet_egg`!
    {
        let mut ar = running.world.arena.write().await;
        ar.boss.x = 0.0;
        ar.boss.y = 0.5;
        ar.boss.z = -23.0;
        ar.boss.hp = 3; // 2 hits to finish (`stomp` = 2 DMG, `blast` = 1 DMG)
    }
    let (st_hit_maya, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": maya_token, "mobId": "obsidian_dragon", "mode": "blast" })),
    )
    .await;
    assert_eq!(st_hit_maya, 200);

    let (st_kill_ella, kill_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/attack",
        Some(json!({ "token": ella_token, "mobId": "obsidian_dragon", "mode": "stomp" })),
    )
    .await;
    assert_eq!(st_kill_ella, 200);
    assert_eq!(kill_res["defeated"], true);
    let rewards = kill_res["bossRewards"].as_array().unwrap();
    assert_eq!(
        rewards.len(),
        2,
        "Every contributing player must receive fair personal loot without loot-stealing"
    );
    assert!(rewards.iter().all(|r| r["petEgg"] == true));

    let cfg_snap = running.world.users_cfg.read().await;
    for uname in ["ella", "maya"] {
        let u = cfg_snap.users.iter().find(|u| u.username == uname).unwrap();
        assert_eq!(u.inventory.get("pet_egg").copied(), Some(1));
        assert!(u.coins >= 45);
    }

    running.http_task.abort();
}

#[tokio::test]
async fn expansion_reviewer_edge_cases_shop_egg_mount_toggle_plot_boundary_and_aegis_sync() {
    use ella_world::udp_wire::ZONE_ARENA;

    let cfg = make_test_config("exp_reviewer_edge_cases");
    let running = start_all_servers(cfg).await.expect("start server");

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "rev_e" })),
    )
    .await;
    let ella_token = ella_login["token"].as_str().unwrap().to_string();
    let ella_id = ella_login["selfId"].as_str().unwrap().to_string();
    assert_eq!(ella_login["petEggShopPrice"], 120);

    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "rev_m" })),
    )
    .await;
    let maya_token = maya_login["token"].as_str().unwrap().to_string();
    let maya_id = maya_login["selfId"].as_str().unwrap().to_string();

    let (_, chloe_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "chloe", "password": "changeme-rainbow", "clientId": "rev_c" })),
    )
    .await;
    let chloe_token = chloe_login["token"].as_str().unwrap().to_string();

    // Seed Maya with 500 Starlight Coins
    {
        let mut cfg_w = running.world.users_cfg.write().await;
        let m = cfg_w
            .users
            .iter_mut()
            .find(|u| u.username == "maya")
            .unwrap();
        m.coins = 500;
    }

    // 1. Shop Pet Egg purchase (`buy_egg` = 120 coins -> +1 `pet_egg`) + hatch!
    let (st_buy_egg, buy_egg_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": maya_token, "action": "buy_egg" })),
    )
    .await;
    assert_eq!(st_buy_egg, 200);
    assert_eq!(buy_egg_res["economy"]["coins"], 380);
    assert_eq!(buy_egg_res["economy"]["inventory"]["pet_egg"], 1);

    let (st_hatch, hatch_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": maya_token, "action": "hatch_egg" })),
    )
    .await;
    assert_eq!(st_hatch, 200);
    assert!(hatch_res["hatchedPetId"].as_str().is_some());

    // 2. Unlocking `comet_surfer` (320 coins) and quick-toggling (`mountId: "toggle"`) preserves `comet_surfer`!
    let (st_buy_mount, buy_mount_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": maya_token, "action": "buy_mount", "mountId": "comet_surfer" })),
    )
    .await;
    assert_eq!(st_buy_mount, 200);
    assert_eq!(buy_mount_res["companions"]["equippedMount"], "comet_surfer");
    assert_eq!(
        buy_mount_res["companions"]["preferredMount"],
        "comet_surfer"
    );

    let (_, tog_off) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": maya_token, "action": "equip_mount", "mountId": "toggle" })),
    )
    .await;
    assert!(tog_off["companions"]["equippedMount"].is_null());

    let (_, tog_on) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/companions",
        Some(json!({ "token": maya_token, "action": "equip_mount", "mountId": "toggle" })),
    )
    .await;
    assert_eq!(
        tog_on["companions"]["equippedMount"], "comet_surfer",
        "Quick-toggle after dismounting must re-equip preferred mount instead of resetting to starlight_board"
    );

    // 3. Plot Ownership Switch Reset + Multi-Cell `double_door` Boundary Grief Protection!
    let (st_claim1, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({ "token": ella_token, "action": "claim", "plotId": "plot_1" })),
    )
    .await;
    assert_eq!(st_claim1, 200);
    let (st_upd1, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({
            "token": ella_token,
            "action": "update_settings",
            "plotId": "plot_1",
            "customTitle": "Ella Secret Loft",
            "coBuilders": ["maya"],
            "doorLocked": true
        })),
    )
    .await;
    assert_eq!(st_upd1, 200);

    // Ella switches to `plot_4` -> `plot_1` becomes unclaimed; Chloe claims `plot_1` -> must NOT inherit Maya as co-builder!
    let (st_switch, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({ "token": ella_token, "action": "claim", "plotId": "plot_4" })),
    )
    .await;
    assert_eq!(st_switch, 200);

    let (st_chloe_claim, chloe_claim_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/homes",
        Some(json!({ "token": chloe_token, "action": "claim", "plotId": "plot_1" })),
    )
    .await;
    assert_eq!(st_chloe_claim, 200);
    let p1 = chloe_claim_res["homes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["plotId"] == "plot_1")
        .unwrap();
    assert_eq!(p1["ownerUsername"], "chloe");
    assert_eq!(p1["coBuilders"].as_array().unwrap().len(), 0);
    assert_eq!(p1["doorLocked"], false);

    // Chloe places a 2-wide `double_door` at `(42, 1, -26, rot=0)` straddling `(42, -26)` outside and `(43, -26)` inside `plot_1`
    let (st_place_dd, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": chloe_token,
            "action": "place",
            "x": 42,
            "y": 1,
            "z": -26,
            "kind": "double_door",
            "rot": 0,
            "color": "#f472b6"
        })),
    )
    .await;
    assert_eq!(st_place_dd, 200);

    // Maya tries removing Chloe's `double_door` by clicking the OUTSIDE cell `(42, 1, -26)` -> MUST be rejected (400)!
    let (st_grief_rm, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": maya_token,
            "action": "remove",
            "x": 42,
            "y": 1,
            "z": -26
        })),
    )
    .await;
    assert_eq!(
        st_grief_rm, 400,
        "Stranger must not delete a multi-cell block straddling a claimed plot boundary via its outside cell"
    );

    // 4. Obby Anti-Teleport `min_elapsed_ms` Guard (`< 2_800ms` rejected on Sky Obby)
    running
        .world
        .apply_c2s_state(
            &ella_id,
            DecodedC2SState {
                seq: 10,
                client_time_ms: 100,
                x: 0.0,
                y: 0.8,
                z: -13.5,
                ry: 0.0,
                anim: "run".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;
    let (st_ob_s, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/obby",
        Some(json!({ "token": ella_token, "action": "start", "course": "easy" })),
    )
    .await;
    assert_eq!(st_ob_s, 200);
    {
        let mut runs = running.world.active_obby_runs.write().await;
        if let Some(r) = runs.get_mut(&ella_id) {
            r.started_at_ms = r.started_at_ms.saturating_sub(150); // Impossible 0.15s teleport!
        }
    }
    running
        .world
        .apply_c2s_state(
            &ella_id,
            DecodedC2SState {
                seq: 11,
                client_time_ms: 250,
                x: 6.5,
                y: 17.0,
                z: -20.0,
                ry: 0.0,
                anim: "jump".into(),
                emote: None,
                muted: false,
                speaking: false,
                zone: 0,
            },
        )
        .await;
    let (st_ob_fast, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/obby",
        Some(json!({ "token": ella_token, "action": "finish", "course": "easy" })),
    )
    .await;
    assert_eq!(
        st_ob_fast, 400,
        "Spoofed 150ms Sky Obby finish must be rejected by min_elapsed_ms (2_800ms)"
    );

    // 5. Acquiring `aegis_heart_charm` while already inside `ZONE_ARENA` dynamically upgrades `max_hp` to `150` on tick!
    let (st_arena_in, arena_in_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/dungeon/portal",
        Some(json!({ "token": maya_token, "targetZone": ZONE_ARENA })),
    )
    .await;
    assert_eq!(st_arena_in, 200);
    assert_eq!(arena_in_res["arenaStatus"]["maxHp"], 100);
    assert_eq!(arena_in_res["arenaStatus"]["hp"], 100);

    {
        let mut cfg_w = running.world.users_cfg.write().await;
        let m = cfg_w
            .users
            .iter_mut()
            .find(|u| u.username == "maya")
            .unwrap();
        m.inventory.insert("aegis_heart_charm".to_string(), 1);
    }
    running.world.tick_arena_if_occupied().await;
    {
        let ar = running.world.arena.read().await;
        let st = ar.status_for(&maya_id);
        assert_eq!(st.max_hp, 150);
        assert_eq!(st.hp, 150);
    }

    running.http_task.abort();
}

#[tokio::test]
async fn invite_only_multi_playground_and_cross_world_avatar_isolation_end_to_end() {
    let cfg = make_test_config("multi_playground_invites");
    let running = start_all_servers(cfg).await.expect("start servers");

    // 1. Uninvited open registration and bogus invite codes are strictly rejected (403)
    let (reg_st, reg_err) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/register",
        Some(json!({ "username": "stranger", "password": "pw1234" })),
    )
    .await;
    assert_eq!(reg_st, 403, "uninvited /register must be 403: {reg_err}");

    let (bad_inv_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/redeem",
        Some(json!({
            "code": "STAR-FAKE-0000",
            "username": "stranger",
            "password": "pw1234"
        })),
    )
    .await;
    assert_eq!(bad_inv_st, 403, "invalid STAR invite must be 403");

    // 2. Non-host friend (`maya`) cannot draft host invites (403), while host (`ella`) can!
    let (_, maya_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "maya", "password": "changeme-puppy", "clientId": "m1" })),
    )
    .await;
    let maya_tok = maya_login["token"].as_str().unwrap();
    let (m_draft_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/host-draft",
        Some(json!({ "token": maya_tok, "labelNote": "Unauthorized draft" })),
    )
    .await;
    assert_eq!(m_draft_st, 403);

    let (_, ella_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({ "username": "ella", "password": "changeme-starlight", "clientId": "e1" })),
    )
    .await;
    assert_eq!(ella_login["worldSlug"], "ella");
    let ella_tok = ella_login["token"].as_str().unwrap();

    let (draft_st, draft_res) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/host-draft",
        Some(json!({ "token": ella_tok, "labelNote": "For Sophie from dance class" })),
    )
    .await;
    assert_eq!(draft_st, 200);
    let draft_code = draft_res["invite"]["code"].as_str().unwrap().to_string();
    assert!(draft_code.starts_with("STAR-"));
    assert_eq!(draft_res["invite"]["status"], "pending_parent_approval");

    // 3. Pending host draft cannot be redeemed before Parent Admin 1-click approval!
    let (insp_st, insp_pending) = http_json_request(
        running.http_addr,
        "GET",
        &format!("/api/ella-world/invites/inspect?code={draft_code}"),
        None,
    )
    .await;
    assert_eq!(insp_st, 200);
    assert_eq!(insp_pending["redeemable"], false);
    assert_eq!(insp_pending["status"], "pending_parent_approval");

    let (premature_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/redeem",
        Some(json!({
            "code": draft_code,
            "username": "sophie",
            "displayName": "Sophie",
            "password": "sophie-secret"
        })),
    )
    .await;
    assert_eq!(
        premature_st, 403,
        "PendingParentApproval invite must not be redeemable before parent approval"
    );

    // 4. Parent Admin of `"ella"` logs in, sees pending draft, and approves with safety presets
    let (adm_st, adm_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(adm_st, 200);
    assert_eq!(adm_login["worldSlug"], "ella");
    let ella_admin_tok = adm_login["adminToken"].as_str().unwrap().to_string();
    assert!(adm_login["dashboard"]["invites"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["code"] == draft_code && i["status"] == "pending_parent_approval"));

    let (app_st, app_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/invites",
        &ella_admin_tok,
        Some(json!({
            "action": "approve",
            "code": draft_code,
            "presetChatMode": "quick_only",
            "starterCoins": 175
        })),
    )
    .await;
    assert_eq!(app_st, 200);
    assert_eq!(app_res["invite"]["status"], "active");

    // 5. Sophie redeems approved STAR-... code -> receives starterCoins & quick_only preset!
    let (red_st, sophie_sess) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/redeem",
        Some(json!({
            "code": draft_code,
            "username": "sophie",
            "displayName": "Sophie",
            "password": "sophie-secret",
            "clientId": "s1"
        })),
    )
    .await;
    assert_eq!(red_st, 200);
    assert_eq!(sophie_sess["worldSlug"], "ella");
    assert_eq!(sophie_sess["economy"]["coins"], 175);
    assert_eq!(sophie_sess["moderation"]["chatMode"], "quick_only");
    let sophie_tok = sophie_sess["token"].as_str().unwrap().to_string();

    // Single-use invite is now exhausted
    let (reuse_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/redeem",
        Some(json!({
            "code": draft_code,
            "username": "sophie2",
            "password": "sophie-secret"
        })),
    )
    .await;
    assert_eq!(reuse_st, 403, "Exhausted invite must reject second use");

    // 6. Super-Admin issues a FAMILY-... Founder Invite -> Parent creates isolated `"leo-world"`
    let (sa_st, sa_login) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/superadmin/login",
        Some(json!({ "password": "changeme-platform-superadmin" })),
    )
    .await;
    assert_eq!(sa_st, 200);
    let super_tok = sa_login["superadminToken"].as_str().unwrap().to_string();

    let (fam_inv_st, fam_inv_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/superadmin/invites",
        &super_tok,
        Some(json!({
            "action": "create",
            "labelNote": "Leo Family Playground",
            "maxUses": 1
        })),
    )
    .await;
    assert_eq!(fam_inv_st, 200);
    let family_code = fam_inv_res["invite"]["code"].as_str().unwrap().to_string();
    assert!(family_code.starts_with("FAMILY-"));

    let (fam_red_st, leo_family_sess) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/redeem-family",
        Some(json!({
            "code": family_code,
            "worldName": "Leo's Cosmic World",
            "worldSlug": "leo-world",
            "adminPassword": "leo-parent-secret-99",
            "hostUsername": "leo",
            "hostDisplayName": "Leo",
            "hostPassword": "leo-star-pw",
            "clientId": "l1"
        })),
    )
    .await;
    assert_eq!(fam_red_st, 200, "redeem-family failed: {leo_family_sess}");
    assert_eq!(leo_family_sess["worldSlug"], "leo-world");
    assert_eq!(leo_family_sess["user"]["role"], "host");
    assert_eq!(leo_family_sess["economy"]["coins"], 250);
    let leo_world_tok = leo_family_sess["token"].as_str().unwrap().to_string();
    let leo_admin_tok = leo_family_sess["adminToken"].as_str().unwrap().to_string();

    // Verify per-playground Parent Admin isolation: Ella's admin password cannot unlock leo-world!
    let (cross_adm_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/login",
        Some(json!({ "worldSlug": "leo-world", "password": "changeme-parent-admin" })),
    )
    .await;
    assert_eq!(cross_adm_st, 401);

    let (leo_adm_st, leo_adm_state) = http_json_auth_request(
        running.http_addr,
        "GET",
        "/api/ella-world/admin/state",
        &leo_admin_tok,
        None,
    )
    .await;
    assert_eq!(leo_adm_st, 200);
    assert_eq!(leo_adm_state["worldSlug"], "leo-world");
    assert_eq!(
        leo_adm_state["dashboard"]["worldName"],
        "Leo's Cosmic World"
    );

    // 7. Ella's Parent Admin creates an active STAR-... invite so Leo can ALSO visit `"ella"`!
    let (star2_st, star2_res) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/invites",
        &ella_admin_tok,
        Some(json!({
            "action": "create",
            "labelNote": "Invite Leo to Ella's World",
            "starterCoins": 90
        })),
    )
    .await;
    assert_eq!(star2_st, 200);
    let star2_code = star2_res["invite"]["code"].as_str().unwrap().to_string();

    // Before redeeming STAR code, Leo cannot switch to `"ella"` (403)
    let (unauth_sw_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/playgrounds/switch",
        Some(json!({ "token": leo_world_tok, "worldSlug": "ella", "clientId": "l1" })),
    )
    .await;
    assert_eq!(unauth_sw_st, 403);

    // Leo redeems STAR-... via `/invites/join-world` -> enters `"ella"` with 90 coins!
    let (join_st, leo_in_ella) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/join-world",
        Some(json!({ "token": leo_world_tok, "code": star2_code, "clientId": "l1" })),
    )
    .await;
    assert_eq!(join_st, 200);
    assert_eq!(leo_in_ella["worldSlug"], "ella");
    assert_eq!(leo_in_ella["economy"]["coins"], 90);
    assert_eq!(leo_in_ella["myPlaygrounds"].as_array().unwrap().len(), 2);
    let leo_ella_tok = leo_in_ella["token"].as_str().unwrap().to_string();

    // 8. Leo customizes his 3D Avatar & places a block in `"ella"`, then switches back to `"leo-world"`
    let (av_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/avatar",
        Some(json!({
            "token": leo_ella_tok,
            "avatar": {
                "bodyType": "male",
                "hairStyle": "maverick_quiff",
                "topStyle": "tech_jacket",
                "topColor": "#38bdf8",
                "bottomStyle": "tech_pants",
                "shoeStyle": "flight_boots"
            }
        })),
    )
    .await;
    assert_eq!(av_st, 200);

    let (blk_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/block",
        Some(json!({
            "token": leo_ella_tok,
            "action": "place",
            "x": 11,
            "y": 4,
            "z": -11,
            "color": "#fb7185"
        })),
    )
    .await;
    assert_eq!(blk_st, 200);

    let (sw_back_st, leo_back_home) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/playgrounds/switch",
        Some(json!({ "token": leo_ella_tok, "worldSlug": "leo-world", "clientId": "l1" })),
    )
    .await;
    assert_eq!(sw_back_st, 200);
    assert_eq!(leo_back_home["worldSlug"], "leo-world");
    // Avatar look carried across playgrounds!
    assert_eq!(
        leo_back_home["user"]["avatar"]["hairStyle"],
        "maverick_quiff"
    );
    assert_eq!(leo_back_home["user"]["avatar"]["topStyle"], "tech_jacket");
    assert_eq!(leo_back_home["user"]["avatar"]["topColor"], "#38bdf8");
    // Coins and placed blocks remained 100% isolated per Playground!
    assert_eq!(leo_back_home["economy"]["coins"], 250);
    let leo_blocks = leo_back_home["blocks"].as_array().unwrap();
    assert!(
        !leo_blocks
            .iter()
            .any(|b| b["x"] == 11 && b["y"] == 4 && b["z"] == -11),
        "Blocks placed in 'ella' must NOT leak into 'leo-world'"
    );
    let leo_home_tok = leo_back_home["token"].as_str().unwrap().to_string();

    // 9. Native Raw UDP (`:44445/udp` EWLD v1) multiplexes both Playgrounds on ONE port with zero leakage!
    // Create a second friend in `leo-world` so we verify positive intra-world UDP state AND zero cross-world leakage
    let (_, leo_friend_inv) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/invites",
        &leo_admin_tok,
        Some(json!({ "action": "create", "labelNote": "Max" })),
    )
    .await;
    let (_, max_sess) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/invites/redeem",
        Some(json!({
            "code": leo_friend_inv["invite"]["code"],
            "username": "max",
            "displayName": "Max",
            "password": "max-secret",
            "clientId": "mx1"
        })),
    )
    .await;
    let max_leo_tok = max_sess["token"].as_str().unwrap().to_string();

    let sock_sophie_ella = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sock_leo_world = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sock_max_leo_world = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    sock_sophie_ella
        .connect(running.native_udp_addr)
        .await
        .unwrap();
    sock_leo_world
        .connect(running.native_udp_addr)
        .await
        .unwrap();
    sock_max_leo_world
        .connect(running.native_udp_addr)
        .await
        .unwrap();

    sock_sophie_ella
        .send(&encode_native_hello(&sophie_tok))
        .await
        .unwrap();
    sock_leo_world
        .send(&encode_native_hello(&leo_home_tok))
        .await
        .unwrap();
    sock_max_leo_world
        .send(&encode_native_hello(&max_leo_tok))
        .await
        .unwrap();

    let mut ubuf = [0u8; 2048];
    for s in [&sock_sophie_ella, &sock_leo_world, &sock_max_leo_world] {
        let n = tokio::time::timeout(Duration::from_secs(2), s.recv(&mut ubuf))
            .await
            .unwrap()
            .unwrap();
        let (op, _) = unwrap_native_packet(&ubuf[..n]).unwrap();
        assert_eq!(op, OP_WELCOME);
    }

    let leo_state_pkt = wrap_native_packet(
        OP_C2S_STATE,
        &encode_c2s_state(&DecodedC2SState {
            seq: 7,
            client_time_ms: 500,
            x: 24.0,
            y: 1.5,
            z: -18.0,
            ry: 0.75,
            anim: "run".into(),
            emote: None,
            muted: false,
            speaking: false,
            zone: 0,
        }),
    );
    sock_leo_world.send(&leo_state_pkt).await.unwrap();

    // Max (in `leo-world`) MUST receive Leo's 20 Hz state, while Sophie (in `ella`) MUST NOT!
    let mut max_saw_leo = false;
    for _ in 0..12 {
        let Ok(Ok(n)) = tokio::time::timeout(
            Duration::from_millis(200),
            sock_max_leo_world.recv(&mut ubuf),
        )
        .await
        else {
            continue;
        };
        if let Some((OP_S2C_WORLD_STATE, body)) = unwrap_native_packet(&ubuf[..n]) {
            if let Some(decoded) = decode_s2c_world_state(body) {
                if decoded.players.iter().any(|p| p.id.contains("peer_leo")) {
                    max_saw_leo = true;
                    break;
                }
            }
        }
    }
    assert!(
        max_saw_leo,
        "Peer in same family playground ('leo-world') must receive Leo's EWLD UDP state"
    );

    for _ in 0..6 {
        if let Ok(Ok(n)) =
            tokio::time::timeout(Duration::from_millis(120), sock_sophie_ella.recv(&mut ubuf)).await
        {
            if let Some((OP_S2C_WORLD_STATE, body)) = unwrap_native_packet(&ubuf[..n]) {
                if let Some(decoded) = decode_s2c_world_state(body) {
                    assert!(
                        !decoded
                            .players
                            .iter()
                            .any(|p| p.id.contains("peer_leo") || p.id.contains("peer_max")),
                        "EWLD UDP snapshot in 'ella' must NEVER leak players from 'leo-world'!"
                    );
                }
            }
        }
    }

    // 10. Android release metadata endpoint contract
    let (and_st, and_meta) = http_json_request(
        running.http_addr,
        "GET",
        "/api/ella-world/android/release",
        None,
    )
    .await;
    assert_eq!(and_st, 200);
    assert_eq!(and_meta["applicationId"], "com.robertscreations.ellaworld");
    assert_eq!(and_meta["aabConfigured"], true);

    // 11. Cross-Playground Password Rollover & Foreign Admin Isolation:
    // Updating guest `leo`'s password inside `"ella"` must NEVER hijack `"leo-world"` (where `leo` is Host)!
    let (up_guest_st, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/users",
        &ella_admin_tok,
        Some(json!({
            "username": "leo",
            "displayName": "Leo Guest",
            "password": "ella-guest-override"
        })),
    )
    .await;
    assert_eq!(up_guest_st, 200);

    let (hijack_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "leo",
            "password": "ella-guest-override",
            "worldSlug": "leo-world",
            "clientId": "hijack_probe"
        })),
    )
    .await;
    assert_eq!(
        hijack_st, 401,
        "Foreign playground admin password edit must NEVER grant login to Leo's home playground!"
    );

    let (old_ella_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "leo",
            "password": "leo-star-pw",
            "worldSlug": "ella",
            "clientId": "stale_ella_probe"
        })),
    )
    .await;
    assert_eq!(
        old_ella_st, 401,
        "Rotated per-playground password must not be bypassed by global password!"
    );

    // Rotating `leo`'s password in his own `"leo-world"` immediately invalidates `"leo-star-pw"` there too!
    let (up_home_st, _) = http_json_auth_request(
        running.http_addr,
        "POST",
        "/api/ella-world/admin/users",
        &leo_admin_tok,
        Some(json!({
            "username": "leo",
            "displayName": "Leo",
            "password": "leo-cosmic-rotated-pw",
            "role": "host"
        })),
    )
    .await;
    assert_eq!(up_home_st, 200);

    let (old_home_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "leo",
            "password": "leo-star-pw",
            "worldSlug": "leo-world",
            "clientId": "stale_home_probe"
        })),
    )
    .await;
    assert_eq!(
        old_home_st, 401,
        "Old password must be rejected after parent rotates host password in family playground!"
    );

    let (new_home_st, _) = http_json_request(
        running.http_addr,
        "POST",
        "/api/ella-world/login",
        Some(json!({
            "username": "leo",
            "password": "leo-cosmic-rotated-pw",
            "worldSlug": "leo-world",
            "clientId": "valid_home_probe"
        })),
    )
    .await;
    assert_eq!(new_home_st, 200);

    running.http_task.abort();
}

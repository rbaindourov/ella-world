use bytes::{BufMut, BytesMut};
use ella_world::config::ServerConfig;
use ella_world::start_all_servers;
use ella_world::udp_wire::{
    decode_s2c_world_state, encode_c2s_state, encode_native_hello, unwrap_native_packet,
    wrap_native_packet, DecodedC2SState, OP_C2S_STATE, OP_EVENT_ACK, OP_EVENT_JSON,
    OP_S2C_WORLD_STATE, OP_VOICE_OPUS, OP_WELCOME,
};
use ella_world::world::WorldEngine;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::mpsc;
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

    // 1. Verify /api/ella-world/health confirms SSE & HTTP sync polling are permanently disabled
    let (status, health) =
        http_json_request(running.http_addr, "GET", "/api/ella-world/health", None).await;
    assert_eq!(status, 200);
    assert_eq!(health["engine"], "ella-world-rust-udp");
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
    let ella_sender = pc_ella
        .add_track(Arc::clone(&ella_mic_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await
        .unwrap();
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

    // --- Client 2: Maya (receives Ella's 20Hz binary state + renegotiated SFU Opus voice over UDP) ---
    let pc_maya = Arc::new(
        client_api
            .new_peer_connection(RTCConfiguration::default())
            .await
            .unwrap(),
    );
    pc_maya
        .add_transceiver_from_kind(RTPCodecType::Audio, None)
        .await
        .unwrap();

    let (maya_voice_tx, mut maya_voice_rx) = mpsc::channel::<Vec<u8>>(16);
    pc_maya.on_track(Box::new(move |track, _rx, _tx| {
        let vtx = maya_voice_tx.clone();
        Box::pin(async move {
            while let Ok((pkt, _)) = track.read_rtp().await {
                let _ = vtx.send(pkt.payload.to_vec()).await;
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

    let maya_events_dc = pc_maya
        .create_data_channel("ella-reliable-events", None)
        .await
        .unwrap();
    let pc_maya_reneg = Arc::clone(&pc_maya);
    let dc_maya_reneg = Arc::clone(&maya_events_dc);
    maya_events_dc.on_message(Box::new(move |msg: DataChannelMessage| {
        let pc_c = Arc::clone(&pc_maya_reneg);
        let dc_c = Arc::clone(&dc_maya_reneg);
        Box::pin(async move {
            if let Ok(val) = serde_json::from_slice::<Value>(&msg.data) {
                if val.get("type").and_then(|v| v.as_str()) == Some("rtc_renegotiate") {
                    if let Some(offer_sdp) = val.get("sdp").and_then(|v| v.as_str()) {
                        let offer = RTCSessionDescription::offer(offer_sdp.to_string()).unwrap();
                        pc_c.set_remote_description(offer).await.unwrap();
                        let ans = pc_c.create_answer(None).await.unwrap();
                        pc_c.set_local_description(ans.clone()).await.unwrap();
                        let reply = json!({ "type": "rtc_answer", "sdp": ans.sdp });
                        let _ = dc_c.send_text(reply.to_string()).await;
                    }
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

    // 2. Verify Opus RTP voice audio forwarded over single-port UDP SFU (`44444/udp` mux)
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

    let forwarded_payload = tokio::time::timeout(Duration::from_secs(5), maya_voice_rx.recv())
        .await
        .expect("Maya SFU UDP voice receive timeout")
        .expect("Maya voice channel open");
    assert_eq!(forwarded_payload, b"ELLA_SFU_OPUS_VOICE");

    pump.abort();
    let _ = pc_ella.close().await;
    let _ = pc_maya.close().await;
    running.http_task.abort();
}

//! Native Raw UDP Server (`tokio::net::UdpSocket` on `ELLA_NATIVE_UDP_PORT`, default `:44445`)
//!
//! Speaks the `EWLD` v1 binary UDP protocol directly over raw UDP datagrams for:
//! - Player authentication (`OP_HELLO` -> `OP_WELCOME`)
//! - 20 Hz unreliable player movement, animations, emotes & speaking state (`OP_C2S_STATE` / `OP_S2C_WORLD_STATE`)
//! - Raw UDP Opus voice frames (`OP_VOICE_OPUS`) relayed to all native UDP peers + bridged to WebRTC SFU
//! - Reliable acknowledged discrete events (`OP_EVENT_JSON` + `OP_EVENT_ACK`) for blocks, chat, outfits & runway

use crate::models::now_ms;
use crate::udp_wire::{
    decode_c2s_state, encode_native_voice_opus, encode_native_welcome, encode_pong,
    unwrap_native_packet, wrap_native_packet, OP_C2S_PING, OP_C2S_STATE, OP_EVENT_ACK,
    OP_EVENT_JSON, OP_HELLO, OP_S2C_PONG, OP_S2C_WORLD_STATE, OP_VOICE_OPUS,
};
use crate::world::{BridgedOpusFrame, WorldEngine};
use bytes::{BufMut, Bytes, BytesMut};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone, Debug)]
struct NativePeerSession {
    self_id: String,
    username: String,
    addr: SocketAddr,
    last_seen_ms: u64,
}

pub struct NativeUdpServer {
    socket: Arc<UdpSocket>,
    local_addr: SocketAddr,
    sessions_by_addr: RwLock<HashMap<SocketAddr, NativePeerSession>>,
    active_count: AtomicUsize,
}

impl NativeUdpServer {
    pub async fn bind_and_start(world: Arc<WorldEngine>) -> anyhow::Result<Arc<Self>> {
        let socket = Arc::new(UdpSocket::bind(world.config.native_udp_addr).await?);
        let local_addr = socket.local_addr()?;
        info!(
            "🚀 [Native UDP] Listening for binary EWLD voice + game-state datagrams on {}/udp",
            local_addr
        );

        let srv = Arc::new(Self {
            socket,
            local_addr,
            sessions_by_addr: RwLock::new(HashMap::new()),
            active_count: AtomicUsize::new(0),
        });

        // 1. Inbound UDP datagram receive loop
        let rx_srv = Arc::clone(&srv);
        let rx_world = Arc::clone(&world);
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                let Ok((len, peer_addr)) = rx_srv.socket.recv_from(&mut buf).await else {
                    continue;
                };
                rx_srv
                    .handle_datagram(&rx_world, peer_addr, &buf[..len])
                    .await;
            }
        });

        // 2. 20Hz Outbound S2C_WORLD_STATE UDP broadcast tick + stale session reaper
        let tx_srv = Arc::clone(&srv);
        let tx_world = Arc::clone(&world);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
            let mut tick_idx = 0u32;
            loop {
                interval.tick().await;
                tick_idx = tick_idx.wrapping_add(1);

                let peers: Vec<NativePeerSession> = {
                    let map = tx_srv.sessions_by_addr.read().await;
                    map.values().cloned().collect()
                };

                for sess in &peers {
                    let chunks = tx_world.encode_world_snapshot_chunks_for(&sess.self_id).await;
                    for snap in chunks {
                        let framed = wrap_native_packet(OP_S2C_WORLD_STATE, &snap);
                        let _ = tx_srv.socket.send_to(&framed, sess.addr).await;
                    }
                }

                if tick_idx.is_multiple_of(100) {
                    let cutoff = now_ms().saturating_sub(35_000);
                    let mut map = tx_srv.sessions_by_addr.write().await;
                    map.retain(|_, s| s.last_seen_ms >= cutoff);
                    tx_srv.active_count.store(map.len(), Ordering::Relaxed);
                }
            }
        });

        // 3. Discrete World Event Push over UDP (`OP_EVENT_JSON`)
        let ev_srv = Arc::clone(&srv);
        let mut ev_rx = world.udp_event_tx.subscribe();
        let mut out_event_seq = 1u32;
        tokio::spawn(async move {
            while let Ok(ev) = ev_rx.recv().await {
                if matches!(ev, crate::world::UdpWorldEvent::VoiceSlotMap { .. }) {
                    continue;
                }
                let Ok(json_bytes) = serde_json::to_vec(&ev) else {
                    continue;
                };
                out_event_seq = out_event_seq.wrapping_add(1);
                let mut body = BytesMut::with_capacity(4 + json_bytes.len());
                body.put_u32_le(out_event_seq);
                body.put_slice(&json_bytes);
                let pkt = wrap_native_packet(OP_EVENT_JSON, &body);

                let addrs: Vec<SocketAddr> = {
                    let map = ev_srv.sessions_by_addr.read().await;
                    map.keys().copied().collect()
                };
                for addr in addrs {
                    let _ = ev_srv.socket.send_to(&pkt, addr).await;
                }
            }
        });

        // 4. Forward WebRTC SFU Opus frames down to Native UDP listeners (`OP_VOICE_OPUS`)
        let voice_srv = Arc::clone(&srv);
        let mut opus_rx = world.opus_bridge_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(frame) = opus_rx.recv().await {
                // Native->Native already fanned out directly; here we also deliver WebRTC->Native!
                if frame.from_native_udp {
                    continue;
                }
                let pkt = encode_native_voice_opus(
                    frame.seq,
                    frame.timestamp,
                    &frame.from_id,
                    &frame.payload,
                );
                let targets: Vec<SocketAddr> = {
                    let map = voice_srv.sessions_by_addr.read().await;
                    map.values()
                        .filter(|s| s.self_id != frame.from_id)
                        .map(|s| s.addr)
                        .collect()
                };
                for addr in targets {
                    let _ = voice_srv.socket.send_to(&pkt, addr).await;
                }
            }
        });

        Ok(srv)
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn active_session_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }

    async fn handle_datagram(&self, world: &WorldEngine, peer_addr: SocketAddr, raw: &[u8]) {
        let Some((pkt_type, body)) = unwrap_native_packet(raw) else {
            return;
        };

        if pkt_type == OP_HELLO {
            if body.len() < 2 {
                return;
            }
            let tok_len = u16::from_le_bytes([body[0], body[1]]) as usize;
            if body.len() < 2 + tok_len {
                return;
            }
            let Ok(token_str) = std::str::from_utf8(&body[2..2 + tok_len]) else {
                return;
            };
            let Some(claims) = world.verify_session_token(token_str) else {
                warn!("Rejected invalid Native UDP HELLO token from {}", peer_addr);
                return;
            };
            let (Some(self_id), Some(username)) = (claims.self_id, claims.username) else {
                return;
            };

            {
                let mut map = self.sessions_by_addr.write().await;
                map.insert(
                    peer_addr,
                    NativePeerSession {
                        self_id: self_id.clone(),
                        username: username.clone(),
                        addr: peer_addr,
                        last_seen_ms: now_ms(),
                    },
                );
                self.active_count.store(map.len(), Ordering::Relaxed);
            }
            let _ = world.ensure_player_online(&self_id, &username).await;
            world.touch_player_udp(&self_id).await;
            let blocks_ver = world.blocks_data.read().await.version;
            let welcome = encode_native_welcome(blocks_ver, &self_id);
            let _ = self.socket.send_to(&welcome, peer_addr).await;
            return;
        }

        // All subsequent packet types require an authenticated session
        let session = {
            let mut map = self.sessions_by_addr.write().await;
            let Some(s) = map.get_mut(&peer_addr) else {
                return;
            };
            s.last_seen_ms = now_ms();
            s.clone()
        };

        match pkt_type {
            OP_C2S_STATE => {
                // `body` may either start with `OP_C2S_STATE` (28 bytes) or be the 27-byte body
                let decoded = if body.first() == Some(&OP_C2S_STATE) {
                    decode_c2s_state(body)
                } else {
                    let mut full = Vec::with_capacity(1 + body.len());
                    full.push(OP_C2S_STATE);
                    full.extend_from_slice(body);
                    decode_c2s_state(&full)
                };
                if let Some(st) = decoded {
                    world
                        .apply_c2s_state_ext(&session.self_id, Some(&session.username), true, st)
                        .await;
                }
            }
            OP_C2S_PING => {
                let client_ts = if body.len() >= 4 {
                    u32::from_le_bytes([body[0], body[1], body[2], body[3]])
                } else {
                    0
                };
                world.touch_player_udp(&session.self_id).await;
                let pong = encode_pong(client_ts, world.current_tick());
                let framed = wrap_native_packet(OP_S2C_PONG, &pong);
                let _ = self.socket.send_to(&framed, peer_addr).await;
            }
            OP_VOICE_OPUS => {
                // Incoming Native UDP Voice: [seq: u16 LE][timestamp: u32 LE][opus_bytes...]
                if body.len() <= 6 {
                    return;
                }
                let seq = u16::from_le_bytes([body[0], body[1]]);
                let timestamp = u32::from_le_bytes([body[2], body[3], body[4], body[5]]);
                let opus_slice = &body[6..];

                // 1. Relay over UDP directly to all other Native UDP peers
                let out_pkt =
                    encode_native_voice_opus(seq, timestamp, &session.self_id, opus_slice);
                let other_addrs: Vec<SocketAddr> = {
                    let map = self.sessions_by_addr.read().await;
                    map.values()
                        .filter(|s| s.self_id != session.self_id)
                        .map(|s| s.addr)
                        .collect()
                };
                for addr in other_addrs {
                    let _ = self.socket.send_to(&out_pkt, addr).await;
                }

                // 2. Publish onto Opus bridge so WebRTC SFU browser peers also hear Native UDP voice!
                let _ = world.opus_bridge_tx.send(BridgedOpusFrame {
                    from_id: session.self_id,
                    from_native_udp: true,
                    seq,
                    timestamp,
                    payload: Bytes::copy_from_slice(opus_slice),
                });
            }
            OP_EVENT_JSON => {
                if body.len() < 5 {
                    return;
                }
                let event_seq = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
                if let Ok(cmd) = serde_json::from_slice::<serde_json::Value>(&body[4..]) {
                    handle_client_event_command(world, &session.self_id, &session.username, &cmd)
                        .await;
                }
                let ack = wrap_native_packet(OP_EVENT_ACK, &event_seq.to_le_bytes());
                let _ = self.socket.send_to(&ack, peer_addr).await;
            }
            _ => {}
        }
    }
}

pub async fn handle_client_event_command(
    world: &WorldEngine,
    self_id: &str,
    username: &str,
    cmd: &serde_json::Value,
) {
    let Some(cmd_type) = cmd.get("type").and_then(|v| v.as_str()) else {
        return;
    };
    match cmd_type {
        "block" => {
            let action = cmd
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("place");
            let x = cmd.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = cmd.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let z = cmd.get("z").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let color = cmd.get("color").and_then(|v| v.as_str());
            let _ = world
                .mutate_block(self_id, username, action, (x, y, z), color)
                .await;
        }
        "chat" => {
            let text = cmd.get("text").and_then(|v| v.as_str());
            let trophy = cmd
                .get("isTrophyWin")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let _ = world.post_chat(self_id, username, text, trophy).await;
        }
        "avatar" => {
            if let Some(av) = cmd.get("avatar") {
                let _ = world.update_avatar(self_id, username, av).await;
            }
        }
        "runway" => {
            let action = cmd.get("action").and_then(|v| v.as_str()).unwrap_or("vote");
            let theme = cmd.get("theme").and_then(|v| v.as_str());
            let target = cmd.get("targetUsername").and_then(|v| v.as_str());
            let stars = cmd.get("stars").and_then(|v| v.as_u64()).map(|s| s as u8);
            let _ = world
                .mutate_runway(self_id, username, action, theme, target, stars)
                .await;
        }
        _ => {}
    }
}

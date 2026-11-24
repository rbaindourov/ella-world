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
    OP_EVENT_JSON, OP_HELLO, OP_S2C_PONG, OP_S2C_WORLD_STATE, OP_VOICE_OPUS, ZONE_ARENA,
    ZONE_DUNGEON, ZONE_PRISON,
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
    device_id: Option<String>,
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
                    let chunks = tx_world
                        .encode_world_snapshot_chunks_for(&sess.self_id)
                        .await;
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
        let ev_world = Arc::clone(&world);
        let mut ev_rx = world.udp_event_tx.subscribe();
        let mut out_event_seq = 1u32;
        tokio::spawn(async move {
            while let Ok(ev) = ev_rx.recv().await {
                if matches!(ev, crate::world::UdpWorldEvent::VoiceSlotMap { .. }) {
                    continue;
                }
                let dungeon_only = matches!(
                    ev,
                    crate::world::UdpWorldEvent::DungeonMobs { .. }
                        | crate::world::UdpWorldEvent::DungeonSync { .. }
                );
                let arena_only = matches!(
                    ev,
                    crate::world::UdpWorldEvent::ArenaMobs { .. }
                        | crate::world::UdpWorldEvent::ArenaSync { .. }
                );
                let prison_only = matches!(ev, crate::world::UdpWorldEvent::PrisonSync { .. });
                let raygun_zone = match &ev {
                    crate::world::UdpWorldEvent::RayGunBeam { zone, .. } => Some(*zone),
                    _ => None,
                };
                let Ok(json_bytes) = serde_json::to_vec(&ev) else {
                    continue;
                };
                out_event_seq = out_event_seq.wrapping_add(1);
                let mut body = BytesMut::with_capacity(4 + json_bytes.len());
                body.put_u32_le(out_event_seq);
                body.put_slice(&json_bytes);
                let pkt = wrap_native_packet(OP_EVENT_JSON, &body);

                let peers: Vec<NativePeerSession> = {
                    let map = ev_srv.sessions_by_addr.read().await;
                    map.values().cloned().collect()
                };
                for sess in peers {
                    if dungeon_only && ev_world.get_peer_zone(&sess.self_id).await != ZONE_DUNGEON {
                        continue;
                    }
                    if arena_only && ev_world.get_peer_zone(&sess.self_id).await != ZONE_ARENA {
                        continue;
                    }
                    if prison_only && ev_world.get_peer_zone(&sess.self_id).await != ZONE_PRISON {
                        continue;
                    }
                    if let Some(rz) = raygun_zone {
                        if ev_world.get_peer_zone(&sess.self_id).await != rz {
                            continue;
                        }
                    }
                    let _ = ev_srv.socket.send_to(&pkt, sess.addr).await;
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

        // 5. Evict Native UDP sessions immediately on password rollover or session revocation
        let revoke_srv = Arc::clone(&srv);
        let mut revoke_rx = world.session_revoke_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(sig) = revoke_rx.recv().await {
                let mut map = revoke_srv.sessions_by_addr.write().await;
                map.retain(|_, s| !sig.matches(&s.self_id, &s.username, s.device_id.as_deref()));
                revoke_srv.active_count.store(map.len(), Ordering::Relaxed);
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

    pub async fn evict_sessions_for_peer(&self, self_id: &str) -> usize {
        let mut map = self.sessions_by_addr.write().await;
        let before = map.len();
        map.retain(|_, s| s.self_id != self_id);
        let after = map.len();
        self.active_count.store(after, Ordering::Relaxed);
        before.saturating_sub(after)
    }

    pub async fn evict_sessions_for_username(&self, username: &str) -> usize {
        let target = username.trim().to_lowercase();
        let mut map = self.sessions_by_addr.write().await;
        let before = map.len();
        map.retain(|_, s| !s.username.eq_ignore_ascii_case(&target));
        let after = map.len();
        self.active_count.store(after, Ordering::Relaxed);
        before.saturating_sub(after)
    }

    pub async fn evict_all_sessions(&self) -> usize {
        let mut map = self.sessions_by_addr.write().await;
        let count = map.len();
        map.clear();
        self.active_count.store(0, Ordering::Relaxed);
        count
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
            let Some((self_id, username, _, _, _, device_id)) =
                world.verify_active_player_token(token_str).await
            else {
                warn!(
                    "Rejected invalid or expired Native UDP HELLO token from {}",
                    peer_addr
                );
                return;
            };

            {
                let mut map = self.sessions_by_addr.write().await;
                map.insert(
                    peer_addr,
                    NativePeerSession {
                        self_id: self_id.clone(),
                        username: username.clone(),
                        device_id,
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
                if !world.is_voice_allowed_for(&session.self_id).await {
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
            let kind = cmd.get("kind").and_then(|v| v.as_str());
            let rot = cmd.get("rot").and_then(|v| v.as_u64()).map(|r| r as u8);
            let loot_item_id = cmd
                .get("lootItemId")
                .or_else(|| cmd.get("loot_item_id"))
                .and_then(|v| v.as_str());
            if let Err(err) = world
                .mutate_block_ext(
                    self_id,
                    username,
                    crate::world::BlockMutationInput {
                        action,
                        coords: (x, y, z),
                        color,
                        kind,
                        rot,
                        loot_item_id,
                    },
                )
                .await
            {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "raygun_zap" => {
            let tx = cmd.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let ty = cmd.get("y").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            let tz = cmd.get("z").and_then(|v| v.as_f64()).unwrap_or(8.0) as f32;
            if let Err(err) = world
                .fire_plaza_raygun(self_id, username, [tx, ty, tz])
                .await
            {
                if !err.contains("recharging") {
                    emit_action_error(world, self_id, cmd_type, &err).await;
                }
            }
        }
        "chat" => {
            let text = cmd.get("text").and_then(|v| v.as_str());
            let trophy = cmd
                .get("isTrophyWin")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let trophy_tier = cmd.get("trophyTier").and_then(|v| v.as_str());
            if let Err(err) = world
                .post_chat(self_id, username, text, trophy, trophy_tier)
                .await
            {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "prison_work" => {
            let node_id = cmd.get("nodeId").and_then(|v| v.as_str()).unwrap_or("");
            if let Err(err) = world.prison_perform_work(self_id, username, node_id).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
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
            if let Err(err) = world
                .mutate_runway(self_id, username, action, theme, target, stars)
                .await
            {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "zone_portal" => {
            let zone = cmd.get("zone").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            if let Err(err) = world.transition_player_zone(self_id, username, zone).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "arena_respawn" => {
            if let Err(err) = world.respawn_in_arena(self_id, username).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "dungeon_interact" => {
            let node_id = cmd.get("nodeId").and_then(|v| v.as_str()).unwrap_or("");
            if let Err(err) = world
                .dungeon_interact_node(self_id, username, node_id)
                .await
            {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "dungeon_attack" => {
            let mob_id = cmd.get("mobId").and_then(|v| v.as_str()).unwrap_or("");
            let mode = cmd.get("mode").and_then(|v| v.as_str()).unwrap_or("blast");
            if let Err(err) = world.dungeon_attack_mob(self_id, mob_id, mode).await {
                // Suppress rapid cooldown error toasts on fast swings
                if !err.contains("cooling down") {
                    emit_action_error(world, self_id, cmd_type, &err).await;
                }
            }
        }
        "dungeon_pickup" => {
            let drop_id = cmd.get("dropId").and_then(|v| v.as_str()).unwrap_or("");
            let _ = world.dungeon_pickup_drop(self_id, username, drop_id).await;
        }
        "use_item" => {
            let item_id = cmd.get("itemId").and_then(|v| v.as_str()).unwrap_or("");
            if let Err(err) = world.use_inventory_item(self_id, username, item_id).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "trade_invite" => {
            let target_id = cmd
                .get("targetPeerId")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if let Err(err) = world.trade_invite(self_id, username, target_id).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "trade_respond" => {
            let trade_id = cmd.get("tradeId").and_then(|v| v.as_str()).unwrap_or("");
            let accept = cmd.get("accept").and_then(|v| v.as_bool()).unwrap_or(false);
            if let Err(err) = world.trade_respond(self_id, trade_id, accept).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "trade_offer" => {
            let trade_id = cmd.get("tradeId").and_then(|v| v.as_str()).unwrap_or("");
            let coins = cmd.get("coins").and_then(|v| v.as_u64()).unwrap_or(0);
            let mut items = HashMap::new();
            if let Some(obj) = cmd.get("items").and_then(|v| v.as_object()) {
                for (k, v) in obj {
                    if let Some(q) = v.as_u64() {
                        items.insert(k.clone(), q.min(u32::MAX as u64) as u32);
                    }
                }
            }
            if let Err(err) = world
                .trade_update_offer(self_id, username, trade_id, coins, &items)
                .await
            {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "trade_lock" => {
            let trade_id = cmd.get("tradeId").and_then(|v| v.as_str()).unwrap_or("");
            let locked = cmd.get("locked").and_then(|v| v.as_bool()).unwrap_or(true);
            if let Err(err) = world.trade_set_lock(self_id, trade_id, locked).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "trade_confirm" => {
            let trade_id = cmd.get("tradeId").and_then(|v| v.as_str()).unwrap_or("");
            if let Err(err) = world.trade_confirm(self_id, trade_id).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "trade_cancel" => {
            let trade_id = cmd.get("tradeId").and_then(|v| v.as_str());
            if let Err(err) = world.trade_cancel(self_id, trade_id).await {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "companions" => {
            let action = cmd.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let pet_id = cmd.get("petId").and_then(|v| v.as_str());
            let mount_id = cmd.get("mountId").and_then(|v| v.as_str());
            let nickname = cmd.get("nickname").and_then(|v| v.as_str());
            if let Err(err) = world
                .mutate_companions(self_id, username, action, pet_id, mount_id, nickname)
                .await
            {
                emit_action_error(world, self_id, cmd_type, &err).await;
            }
        }
        "homes" => {
            let action = cmd.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let plot_id = cmd.get("plotId").and_then(|v| v.as_str());
            let custom_title = cmd.get("customTitle").and_then(|v| v.as_str());
            let door_locked = cmd.get("doorLocked").and_then(|v| v.as_bool());
            let co_builders = cmd.get("coBuilders").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            });
            match world
                .mutate_home(
                    self_id,
                    username,
                    false,
                    action,
                    plot_id,
                    custom_title,
                    door_locked,
                    co_builders,
                )
                .await
            {
                Ok(res) => {
                    let _ = world
                        .send_peer_targeted_json(self_id, res.to_string())
                        .await;
                }
                Err(err) => {
                    emit_action_error(world, self_id, cmd_type, &err).await;
                }
            }
        }
        "craft" => {
            let recipe_id = cmd.get("recipeId").and_then(|v| v.as_str()).unwrap_or("");
            match world.craft_recipe(self_id, username, recipe_id).await {
                Ok(res) => {
                    let _ = world
                        .send_peer_targeted_json(self_id, res.to_string())
                        .await;
                }
                Err(err) => {
                    emit_action_error(world, self_id, cmd_type, &err).await;
                }
            }
        }
        "obby" => {
            let action = cmd.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let course = cmd.get("course").and_then(|v| v.as_str());
            match world
                .obby_speedrun_action(self_id, username, action, course)
                .await
            {
                Ok(res) => {
                    let _ = world
                        .send_peer_targeted_json(self_id, res.to_string())
                        .await;
                }
                Err(err) => {
                    emit_action_error(world, self_id, cmd_type, &err).await;
                }
            }
        }
        "sky" => {
            let action = cmd.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let mode = cmd.get("mode").and_then(|v| v.as_str());
            let star_id = cmd.get("starId").and_then(|v| v.as_str());
            match world
                .mutate_sky(self_id, Some(username), false, action, mode, star_id)
                .await
            {
                Ok(res) => {
                    let _ = world
                        .send_peer_targeted_json(self_id, res.to_string())
                        .await;
                }
                Err(err) => {
                    emit_action_error(world, self_id, cmd_type, &err).await;
                }
            }
        }
        "arena_boss" => {
            let use_beacon = cmd
                .get("useBeacon")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match world.summon_arena_boss(self_id, username, use_beacon).await {
                Ok(res) => {
                    let _ = world
                        .send_peer_targeted_json(self_id, res.to_string())
                        .await;
                }
                Err(err) => {
                    emit_action_error(world, self_id, cmd_type, &err).await;
                }
            }
        }
        _ => {}
    }
}

async fn emit_action_error(world: &WorldEngine, self_id: &str, action: &str, error: &str) {
    let payload = serde_json::json!({
        "type": "action_error",
        "action": action,
        "error": error,
    })
    .to_string();
    let _ = world.send_peer_targeted_json(self_id, payload).await;
}

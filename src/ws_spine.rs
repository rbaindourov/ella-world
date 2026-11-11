//! Authoritative Persistent Binary + JSON WebSocket Spine (`GET /api/ella-world/ws?token=...`)
//!
//! Owns:
//! 1. Player session presence with automatic token re-hydration & 12-second reconnect grace window
//! 2. Reliable discrete world events (`PlayerJoin`, `PlayerLeave`, `AvatarUpdated`, `Block`, `Chat`, `Runway`, `VoiceSlotMap`)
//! 3. Self-healing 3-second `world_sync_digest` (`stateSeq`, `serverTimeMs`, `blocksVersion`, `avatarHash`, `players`)
//! 4. Seamless automatic 20 Hz 28-byte binary `OP_C2S_STATE` / `OP_S2C_WORLD_STATE` fallback whenever
//!    the `44444/udp` WebRTC DataChannel fast-lane is handshaking, reconnecting, or blocked by a firewall.

use crate::models::now_ms;
use crate::udp_native::handle_client_event_command;
use crate::udp_wire::{decode_c2s_state, encode_pong, OP_C2S_PING, OP_C2S_STATE};
use crate::world::{UdpWorldEvent, WorldEngine};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;

use crate::http_api::AppState;

#[derive(Default)]
pub struct WsSpineHub {
    active_count: AtomicUsize,
}

impl WsSpineHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            active_count: AtomicUsize::new(0),
        })
    }

    pub fn active_session_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }
}

#[derive(Deserialize)]
pub struct WsAuthQuery {
    pub token: String,
}

pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State(st): State<AppState>,
) -> Response {
    let Some(claims) = st.world.verify_session_token(query.token.trim()) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "Invalid session token for WebSocket spine." })),
        )
            .into_response();
    };
    let (Some(self_id), Some(username)) = (claims.self_id, claims.username) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "Missing player identity in token." })),
        )
            .into_response();
    };

    let world = Arc::clone(&st.world);
    let hub = Arc::clone(&st.ws_hub);

    ws.max_message_size(64 * 1024)
        .on_upgrade(move |socket| run_ws_spine_session(socket, world, hub, self_id, username))
}

async fn run_ws_spine_session(
    mut socket: WebSocket,
    world: Arc<WorldEngine>,
    hub: Arc<WsSpineHub>,
    self_id: String,
    username: String,
) {
    let Some((epoch, _player)) = world.mark_ws_connected(&self_id, &username).await else {
        let _ = socket.close().await;
        return;
    };

    hub.active_count.fetch_add(1, Ordering::Relaxed);
    let mut ev_rx = world.udp_event_tx.subscribe();

    // 1. Push full authoritative welcome snapshot immediately on connect/reconnect
    if let Some(welcome) = world.build_welcome_snapshot(&self_id, &username).await {
        if let Ok(txt) = serde_json::to_string(&welcome) {
            if socket.send(Message::Text(txt)).await.is_err() {
                hub.active_count.fetch_sub(1, Ordering::Relaxed);
                world.mark_ws_disconnected(&self_id, epoch).await;
                return;
            }
        }
    }

    let mut binary_fallback_tick = tokio::time::interval(Duration::from_millis(50));
    binary_fallback_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut digest_tick = tokio::time::interval(Duration::from_millis(3_000));
    digest_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // Consume immediate initial tick since `welcome_snapshot` was just sent
    digest_tick.tick().await;

    let mut keepalive_tick = tokio::time::interval(Duration::from_secs(15));
    keepalive_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    keepalive_tick.tick().await;

    loop {
        tokio::select! {
            inbound = socket.recv() => {
                let Some(Ok(msg)) = inbound else {
                    break;
                };
                match msg {
                    Message::Binary(raw) => {
                        if raw.is_empty() {
                            continue;
                        }
                        match raw[0] {
                            OP_C2S_STATE => {
                                if let Some(st) = decode_c2s_state(&raw) {
                                    world
                                        .apply_c2s_state_ext(&self_id, Some(&username), false, st)
                                        .await;
                                }
                            }
                            OP_C2S_PING if raw.len() >= 5 => {
                                let client_ts =
                                    u32::from_le_bytes([raw[1], raw[2], raw[3], raw[4]]);
                                world.touch_player(&self_id).await;
                                let pong = encode_pong(client_ts, world.current_tick());
                                if socket.send(Message::Binary(pong.to_vec())).await.is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    Message::Text(txt) => {
                        let Ok(cmd) = serde_json::from_str::<Value>(&txt) else {
                            continue;
                        };
                        let Some(cmd_type) = cmd.get("type").and_then(|v| v.as_str()) else {
                            continue;
                        };
                        match cmd_type {
                            "udp_fastlane" => {
                                let active = cmd
                                    .get("active")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                world.set_udp_fastlane_active(&self_id, active).await;
                            }
                            "request_resync" => {
                                world.touch_player(&self_id).await;
                                if let Some(snap) =
                                    world.build_welcome_snapshot(&self_id, &username).await
                                {
                                    if let Ok(snap_txt) = serde_json::to_string(&snap) {
                                        if socket.send(Message::Text(snap_txt)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            "ping" => {
                                world.touch_player(&self_id).await;
                                let reply = json!({
                                    "type": "pong",
                                    "clientTimeMs": cmd.get("clientTimeMs").cloned().unwrap_or(Value::Null),
                                    "serverTimeMs": now_ms(),
                                    "stateSeq": world.current_state_seq()
                                });
                                if socket
                                    .send(Message::Text(reply.to_string()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            _ => {
                                world.touch_player(&self_id).await;
                                handle_client_event_command(&world, &self_id, &username, &cmd)
                                    .await;
                                if let Some(req_id) = cmd.get("reqId") {
                                    let ack = json!({
                                        "type": "cmd_ack",
                                        "reqId": req_id,
                                        "stateSeq": world.current_state_seq(),
                                        "serverTimeMs": now_ms()
                                    });
                                    if socket
                                        .send(Message::Text(ack.to_string()))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Message::Ping(payload) => {
                        world.touch_player(&self_id).await;
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Message::Pong(_) => {
                        world.touch_player(&self_id).await;
                    }
                    Message::Close(_) => {
                        break;
                    }
                }
            }

            ev_res = ev_rx.recv() => {
                match ev_res {
                    Ok(ev) => {
                        match &ev {
                            UdpWorldEvent::VoiceSlotMap { target_self_id, .. }
                                if target_self_id != &self_id =>
                            {
                                continue;
                            }
                            UdpWorldEvent::PlayerJoin { player, .. } if player.id == self_id => {
                                continue;
                            }
                            _ => {}
                        }
                        if let Ok(json_txt) = serde_json::to_string(&ev) {
                            if socket.send(Message::Text(json_txt)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(RecvError::Lagged(_)) => {
                        // Broadcast ring lagged; immediately self-heal client with full welcome_snapshot!
                        if let Some(snap) = world.build_welcome_snapshot(&self_id, &username).await {
                            if let Ok(snap_txt) = serde_json::to_string(&snap) {
                                if socket.send(Message::Text(snap_txt)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(RecvError::Closed) => break,
                }
            }

            _ = binary_fallback_tick.tick() => {
                // Automatic 20Hz Binary Fallback: stream compact `OP_S2C_WORLD_STATE` (`0x11`)
                // over the WebSocket spine whenever the UDP fast-lane (`44444/udp`) is not active.
                if !world.is_udp_fastlane_healthy(&self_id).await {
                    let snap = world.encode_world_snapshot_for(&self_id).await;
                    if socket.send(Message::Binary(snap.to_vec())).await.is_err() {
                        break;
                    }
                }
            }

            _ = digest_tick.tick() => {
                let digest = world.build_sync_digest().await;
                if let Ok(json_txt) = serde_json::to_string(&digest) {
                    if socket.send(Message::Text(json_txt)).await.is_err() {
                        break;
                    }
                }
            }

            _ = keepalive_tick.tick() => {
                if socket.send(Message::Ping(vec![b'E', b'W'])).await.is_err() {
                    break;
                }
            }
        }
    }

    hub.active_count.fetch_sub(1, Ordering::Relaxed);
    world.mark_ws_disconnected(&self_id, epoch).await;
}

use crate::auth::{sign_token, verify_token, TokenClaims};
use crate::config::ServerConfig;
use crate::economy::{
    add_inventory_item, execute_atomic_trade_swap, find_item_def, reconcile_equipped_cosmetics,
    remove_inventory_item, roll_crystal_loot, roll_golden_chest_loot, roll_mob_loot,
    roll_wooden_chest_loot, sanitize_and_validate_offer, DungeonMob, DungeonSnapshot, DungeonState,
    TradeSession, TradeSideOffer, DUNGEON_RETURN_PORTAL_POS, DUNGEON_SPAWN_POS, ITEM_CATALOG,
    PLAZA_PORTAL_POS, PLAZA_RETURN_SPAWN_POS, TRADE_ACTIVE_IDLE_TTL_MS, TRADE_INVITE_TTL_MS,
};
use crate::models::{
    default_starter_blocks, is_valid_hex_color, now_ms, BlockedChatEntry, BlocksDataFile,
    ChatMessage, PlayerEconomySnapshot, PlayerRuntimeState, RunwayPublicSummary, RunwayState,
    SanitizedUser, SavedPosition, UserAccount, UsersConfigFile, VoxelBlock, RUNWAY_THEMES,
};
use crate::persistence::PersistenceStore;
use crate::udp_wire::{
    assemble_s2c_world_state_chunks, assemble_s2c_world_state_scheduled_chunk,
    encode_player_binary_entry, DecodedC2SState, MAX_SNAPSHOT_MTU_BYTES, ZONE_DUNGEON, ZONE_PLAZA,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, watch, RwLock};

pub const DEFAULT_DISCONNECT_GRACE_MS: u64 = 12_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UdpWorldEvent {
    Roster {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        #[serde(rename = "serverTimeMs")]
        server_time_ms: u64,
        players: Vec<PlayerRuntimeState>,
        #[serde(rename = "blocksVersion")]
        blocks_version: u64,
    },
    WorldSyncDigest {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        #[serde(rename = "serverTimeMs")]
        server_time_ms: u64,
        #[serde(rename = "blocksVersion")]
        blocks_version: u64,
        runway: RunwayPublicSummary,
        players: Vec<PlayerRuntimeState>,
    },
    PlayerJoin {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        player: PlayerRuntimeState,
    },
    PlayerLeave {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        id: String,
    },
    AvatarUpdated {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        #[serde(rename = "selfId")]
        self_id: String,
        #[serde(rename = "avatarHash")]
        avatar_hash: String,
        user: SanitizedUser,
    },
    Chat {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        chat: ChatMessage,
    },
    Block {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        #[serde(rename = "selfId")]
        self_id: String,
        #[serde(rename = "blocksVersion")]
        blocks_version: u64,
        blocks: Vec<VoxelBlock>,
    },
    Runway {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        event: String,
        #[serde(rename = "actorName", skip_serializing_if = "Option::is_none")]
        actor_name: Option<String>,
        #[serde(rename = "voterId", skip_serializing_if = "Option::is_none")]
        voter_id: Option<String>,
        #[serde(rename = "voterName", skip_serializing_if = "Option::is_none")]
        voter_name: Option<String>,
        #[serde(rename = "targetUsername", skip_serializing_if = "Option::is_none")]
        target_username: Option<String>,
        stars: Option<u8>,
        runway: RunwayPublicSummary,
    },
    VoiceSlotMap {
        #[serde(rename = "targetSelfId")]
        target_self_id: String,
        slots: HashMap<String, Option<String>>,
    },
    PlayerZone {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        id: String,
        zone: u8,
        x: f32,
        y: f32,
        z: f32,
    },
    DungeonMobs {
        #[serde(rename = "serverTimeMs")]
        server_time_ms: u64,
        mobs: Vec<DungeonMob>,
    },
    DungeonSync {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        event: Option<String>,
        #[serde(rename = "actorId", skip_serializing_if = "Option::is_none")]
        actor_id: Option<String>,
        dungeon: DungeonSnapshot,
    },
}

#[derive(Clone, Debug)]
pub struct BridgedOpusFrame {
    pub from_id: String,
    pub from_native_udp: bool,
    pub seq: u16,
    pub timestamp: u32,
    pub payload: Bytes,
}

#[derive(Clone, Default)]
pub struct PreEncodedWorldTick {
    pub server_tick: u32,
    pub entries: Vec<(String, Bytes)>,
    pub entries_by_zone: HashMap<u8, Vec<(String, Bytes)>>,
    pub zone_by_id: HashMap<String, u8>,
    pub echo_ts_by_id: HashMap<String, u32>,
}

pub struct WorldEngine {
    pub config: ServerConfig,
    pub store: Arc<PersistenceStore>,
    pub users_cfg: RwLock<UsersConfigFile>,
    pub blocks_data: RwLock<BlocksDataFile>,
    pub players: RwLock<HashMap<String, PlayerRuntimeState>>,
    pub chats: RwLock<VecDeque<ChatMessage>>,
    pub chat_cooldowns: RwLock<HashMap<String, VecDeque<u64>>>,
    pub runway: RwLock<RunwayState>,
    pub dungeon: RwLock<DungeonState>,
    pub trades: RwLock<HashMap<String, TradeSession>>,
    pub server_tick: AtomicU32,
    pub state_seq: AtomicU64,
    pub disconnect_grace_ms: AtomicU64,
    pub users_dirty: AtomicBool,
    pub cached_chat_count: AtomicUsize,
    pub cached_blocked_chat_count: AtomicUsize,
    pub pre_encoded_tick: RwLock<Arc<PreEncodedWorldTick>>,
    pub pre_encoded_tick_tx: watch::Sender<Arc<PreEncodedWorldTick>>,
    pub cached_digest: RwLock<(u64, Arc<String>)>,
    pub peer_json_txs: RwLock<HashMap<String, mpsc::UnboundedSender<String>>>,
    pub udp_event_tx: broadcast::Sender<UdpWorldEvent>,
    pub opus_bridge_tx: broadcast::Sender<BridgedOpusFrame>,
}

impl WorldEngine {
    pub fn new(config: ServerConfig) -> anyhow::Result<Arc<Self>> {
        let store = Arc::new(PersistenceStore::new(
            &config.data_dir,
            &config.seed_users_file,
        )?);
        let users_cfg = store.load_or_init_users()?;
        let blocks_data = store.load_or_init_blocks()?;
        let initial_chat_count = store.count_chats();
        let initial_blocked_count = store.count_blocked_chats();
        let recent_chats = store.load_recent_chats(60);
        let mut chat_deque = VecDeque::with_capacity(64);
        for c in recent_chats {
            chat_deque.push_back(c);
        }
        let (udp_event_tx, _) = broadcast::channel(4096);
        let (opus_bridge_tx, _) = broadcast::channel(2048);
        let initial_tick_arc = Arc::new(PreEncodedWorldTick::default());
        let (pre_encoded_tick_tx, _) = watch::channel(Arc::clone(&initial_tick_arc));

        let engine = Arc::new(Self {
            config,
            store,
            users_cfg: RwLock::new(users_cfg),
            blocks_data: RwLock::new(blocks_data),
            players: RwLock::new(HashMap::new()),
            chats: RwLock::new(chat_deque),
            chat_cooldowns: RwLock::new(HashMap::new()),
            runway: RwLock::new(RunwayState::default()),
            dungeon: RwLock::new(DungeonState::new()),
            trades: RwLock::new(HashMap::new()),
            server_tick: AtomicU32::new(1),
            state_seq: AtomicU64::new(1),
            disconnect_grace_ms: AtomicU64::new(DEFAULT_DISCONNECT_GRACE_MS),
            users_dirty: AtomicBool::new(false),
            cached_chat_count: AtomicUsize::new(initial_chat_count),
            cached_blocked_chat_count: AtomicUsize::new(initial_blocked_count),
            pre_encoded_tick: RwLock::new(initial_tick_arc),
            pre_encoded_tick_tx,
            cached_digest: RwLock::new((0, Arc::new(String::new()))),
            peer_json_txs: RwLock::new(HashMap::new()),
            udp_event_tx,
            opus_bridge_tx,
        });

        let bg = Arc::clone(&engine);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut tick_counter = 0u32;
            loop {
                interval.tick().await;
                let tick = bg.server_tick.fetch_add(1, Ordering::Relaxed) + 1;
                tick_counter = tick_counter.wrapping_add(1);
                if tick_counter.is_multiple_of(10) {
                    bg.prune_stale_players().await;
                    bg.prune_expired_trades().await;
                }
                if tick_counter.is_multiple_of(5) {
                    bg.tick_dungeon_if_occupied().await;
                }
                bg.rebuild_pre_encoded_tick(tick).await;
                if tick_counter.is_multiple_of(100) {
                    bg.flush_player_positions_async().await;
                } else if tick_counter.is_multiple_of(60)
                    && bg.users_dirty.swap(false, Ordering::AcqRel)
                {
                    let snapshot = bg.users_cfg.read().await.clone();
                    let store = Arc::clone(&bg.store);
                    tokio::task::spawn_blocking(move || {
                        let _ = store.save_users(&snapshot);
                    });
                }
            }
        });

        Ok(engine)
    }

    pub fn current_tick(&self) -> u32 {
        self.server_tick.load(Ordering::Relaxed)
    }

    pub fn current_state_seq(&self) -> u64 {
        self.state_seq.load(Ordering::Relaxed)
    }

    pub fn next_state_seq(&self) -> u64 {
        self.state_seq.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn set_disconnect_grace_ms(&self, ms: u64) {
        self.disconnect_grace_ms.store(ms, Ordering::Relaxed);
    }

    pub fn verify_session_token(&self, token: &str) -> Option<TokenClaims> {
        verify_token(token, &self.config.hmac_secret)
    }

    pub fn sign_session_token(&self, claims: &TokenClaims) -> String {
        sign_token(claims, &self.config.hmac_secret)
    }

    pub async fn prune_stale_players(&self) {
        let now = now_ms();
        let grace_ms = self.disconnect_grace_ms.load(Ordering::Relaxed);
        let idle_no_ws_cutoff_ms = grace_ms.max(25_000);
        let mut removed = Vec::new();
        {
            let mut map = self.players.write().await;
            map.retain(|id, p| {
                // Refresh UDP fast-lane liveness automatically
                if p.udp_fastlane_active && now.saturating_sub(p.last_udp_seen_ms) > 2_200 {
                    p.udp_fastlane_active = false;
                }

                let should_evict = if p.ws_connected {
                    now.saturating_sub(p.last_seen) > 45_000
                } else if let Some(disc_at) = p.disconnected_at_ms {
                    // Only evict if BOTH grace period elapsed AND no fresh UDP packets arrived since disconnect
                    now.saturating_sub(disc_at) >= grace_ms
                        && now.saturating_sub(p.last_seen) >= grace_ms
                } else {
                    now.saturating_sub(p.last_seen) > idle_no_ws_cutoff_ms
                };

                if should_evict {
                    removed.push(id.clone());
                    false
                } else {
                    true
                }
            });
        }
        for id in removed {
            self.cancel_trades_for_peer(&id, "Friend disconnected from the world.")
                .await;
            let seq = self.next_state_seq();
            let _ = self
                .udp_event_tx
                .send(UdpWorldEvent::PlayerLeave { state_seq: seq, id });
        }
    }

    pub async fn remove_player_immediately(&self, self_id: &str) {
        let existed = self.players.write().await.remove(self_id).is_some();
        if existed {
            self.cancel_trades_for_peer(self_id, "Friend left the world.")
                .await;
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerLeave {
                state_seq: seq,
                id: self_id.to_string(),
            });
        }
    }

    pub async fn active_players_list(&self) -> Vec<PlayerRuntimeState> {
        let map = self.players.read().await;
        map.values().cloned().collect()
    }

    fn spawn_coords_for_user(
        active_count: usize,
        found_user: &UserAccount,
    ) -> (f32, f32, f32, f32) {
        let angle = (active_count as f32 * 1.35) % std::f32::consts::TAU;
        let default_x = (angle.cos() * 2.5 * 10.0).round() / 10.0;
        let default_z = (angle.sin() * 2.5 * 10.0).round() / 10.0;

        if let Some(pos) = &found_user.last_pos {
            (
                pos.x.clamp(-75.0, 75.0),
                pos.y.clamp(0.0, 75.0),
                pos.z.clamp(-75.0, 75.0),
                pos.ry,
            )
        } else {
            (default_x, 0.0, default_z, 0.0)
        }
    }

    /// Ensures `self_id` is present in `players` (re-hydrating seamlessly from `users_cfg` if the
    /// player reconnected after a timeout or server binary restart).
    pub async fn ensure_player_online(
        &self,
        self_id: &str,
        username: &str,
    ) -> Option<PlayerRuntimeState> {
        let found_user = {
            let cfg = self.users_cfg.read().await;
            cfg.users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
                .cloned()?
        };
        let sanitized = found_user.sanitized();
        let avatar_hash = sanitized.avatar.short_hash();
        let now = now_ms();

        let mut map = self.players.write().await;
        if let Some(existing) = map.get_mut(self_id) {
            existing.user = sanitized;
            existing.avatar_hash = avatar_hash;
            existing.last_seen = now;
            existing.disconnected_at_ms = None;
            return Some(existing.clone());
        }

        let (spawn_x, spawn_y, spawn_z, spawn_ry) =
            Self::spawn_coords_for_user(map.len(), &found_user);
        let player_doc = PlayerRuntimeState {
            id: self_id.to_string(),
            user: sanitized,
            avatar_hash,
            zone: ZONE_PLAZA,
            x: spawn_x,
            y: spawn_y,
            z: spawn_z,
            ry: spawn_ry,
            anim: "idle".into(),
            emote: None,
            muted: false,
            speaking: false,
            last_seen: now,
            last_seq: 0,
            last_client_time_ms: 0,
            conn_epoch: 1,
            ws_connected: false,
            udp_fastlane_active: false,
            last_udp_seen_ms: 0,
            disconnected_at_ms: None,
            speed_boost_until_ms: 0,
            jump_boost_until_ms: 0,
        };
        map.insert(self_id.to_string(), player_doc.clone());
        drop(map);

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerJoin {
            state_seq: seq,
            player: player_doc.clone(),
        });

        Some(player_doc)
    }

    pub async fn mark_ws_connected(
        &self,
        self_id: &str,
        username: &str,
    ) -> Option<(u32, PlayerRuntimeState)> {
        let _ = self.ensure_player_online(self_id, username).await?;
        let mut map = self.players.write().await;
        let p = map.get_mut(self_id)?;
        p.conn_epoch = p.conn_epoch.wrapping_add(1).max(1);
        p.ws_connected = true;
        p.disconnected_at_ms = None;
        p.last_seen = now_ms();
        p.last_seq = 0;
        Some((p.conn_epoch, p.clone()))
    }

    pub async fn mark_ws_disconnected(&self, self_id: &str, epoch: u32) {
        let mut map = self.players.write().await;
        if let Some(p) = map.get_mut(self_id) {
            if p.conn_epoch == epoch {
                p.ws_connected = false;
                p.disconnected_at_ms = Some(now_ms());
            }
        }
    }

    pub async fn set_udp_fastlane_active(&self, self_id: &str, active: bool) {
        let now = now_ms();
        let mut map = self.players.write().await;
        if let Some(p) = map.get_mut(self_id) {
            if active && !p.udp_fastlane_active {
                p.last_seq = 0;
            }
            p.udp_fastlane_active = active;
            if active {
                p.last_udp_seen_ms = now;
                p.last_seen = now;
            }
        }
    }

    pub async fn is_udp_fastlane_healthy(&self, self_id: &str) -> bool {
        let now = now_ms();
        let map = self.players.read().await;
        map.get(self_id)
            .map(|p| p.udp_fastlane_active && now.saturating_sub(p.last_udp_seen_ms) <= 2_200)
            .unwrap_or(false)
    }

    pub async fn create_or_resume_session(
        &self,
        found_user: &UserAccount,
        raw_client_id: Option<&str>,
    ) -> serde_json::Value {
        let clean_user = found_user.username.trim().to_lowercase();
        let clean_cid: String = raw_client_id
            .unwrap_or("")
            .trim()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .take(24)
            .collect();
        let self_id = if clean_cid.is_empty() {
            format!("peer_{clean_user}")
        } else {
            format!("peer_{clean_user}_{clean_cid}")
        };

        let token = self.sign_session_token(&TokenClaims {
            self_id: Some(self_id.clone()),
            username: Some(found_user.username.clone()),
            role: Some(found_user.role.clone()),
            iat: now_ms(),
        });

        let sanitized = found_user.sanitized();
        let avatar_hash = sanitized.avatar.short_hash();
        let now = now_ms();

        let (player_doc, is_brand_new) = {
            let mut map = self.players.write().await;
            if let Some(existing) = map.get_mut(&self_id) {
                existing.user = sanitized.clone();
                existing.avatar_hash = avatar_hash;
                existing.last_seen = now;
                existing.last_seq = 0;
                existing.disconnected_at_ms = None;
                (existing.clone(), false)
            } else {
                let (spawn_x, spawn_y, spawn_z, spawn_ry) =
                    Self::spawn_coords_for_user(map.len(), found_user);
                let doc = PlayerRuntimeState {
                    id: self_id.clone(),
                    user: sanitized.clone(),
                    avatar_hash,
                    zone: ZONE_PLAZA,
                    x: spawn_x,
                    y: spawn_y,
                    z: spawn_z,
                    ry: spawn_ry,
                    anim: "idle".into(),
                    emote: None,
                    muted: false,
                    speaking: false,
                    last_seen: now,
                    last_seq: 0,
                    last_client_time_ms: 0,
                    conn_epoch: 1,
                    ws_connected: false,
                    udp_fastlane_active: false,
                    last_udp_seen_ms: 0,
                    disconnected_at_ms: None,
                    speed_boost_until_ms: 0,
                    jump_boost_until_ms: 0,
                };
                map.insert(self_id.clone(), doc.clone());
                (doc, true)
            }
        };

        if is_brand_new {
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerJoin {
                state_seq: seq,
                player: player_doc.clone(),
            });
        }

        let other_players: Vec<PlayerRuntimeState> = self
            .players
            .read()
            .await
            .values()
            .filter(|p| p.id != self_id)
            .cloned()
            .collect();

        let blocks_snapshot = self.blocks_data.read().await.clone();
        let chat_history: Vec<ChatMessage> = self
            .chats
            .read()
            .await
            .iter()
            .rev()
            .take(25)
            .rev()
            .cloned()
            .collect();
        let runway_summary = self.runway.read().await.summary();
        let dungeon_snapshot = self.dungeon.read().await.snapshot();
        let economy_snapshot = found_user.economy_snapshot();
        let cfg = self.users_cfg.read().await;

        serde_json::json!({
            "token": token,
            "selfId": self_id,
            "self": player_doc,
            "user": sanitized,
            "economy": economy_snapshot,
            "itemCatalog": ITEM_CATALOG,
            "dungeon": dungeon_snapshot,
            "players": other_players,
            "blocks": blocks_snapshot.blocks,
            "blocksVersion": blocks_snapshot.version,
            "chatHistory": chat_history,
            "runway": runway_summary,
            "stateSeq": self.current_state_seq(),
            "serverTimeMs": now,
            "worldMeta": {
                "worldName": cfg.world_name,
                "voiceDefaultMode": cfg.voice_default_mode,
                "iceServers": cfg.ice_servers,
                "transport": "hybrid-ws-spine-plus-udp-sfu",
                "webrtcUdpPort": self.config.webrtc_udp_port,
                "nativeUdpPort": self.config.native_udp_addr.port(),
                "wsUrl": "/api/ella-world/ws",
                "rtcOfferUrl": "/api/ella-world/rtc/offer"
            }
        })
    }

    pub async fn build_welcome_snapshot(
        &self,
        self_id: &str,
        username: &str,
    ) -> Option<serde_json::Value> {
        let me = self.ensure_player_online(self_id, username).await?;
        let others: Vec<PlayerRuntimeState> = self
            .players
            .read()
            .await
            .values()
            .filter(|p| p.id != self_id)
            .cloned()
            .collect();
        let blocks_snapshot = self.blocks_data.read().await.clone();
        let chat_history: Vec<ChatMessage> = self
            .chats
            .read()
            .await
            .iter()
            .rev()
            .take(25)
            .rev()
            .cloned()
            .collect();
        let runway_summary = self.runway.read().await.summary();
        let dungeon_snapshot = self.dungeon.read().await.snapshot();
        let economy_snapshot = self.get_player_economy(username).await.unwrap_or_default();

        Some(serde_json::json!({
            "type": "welcome_snapshot",
            "stateSeq": self.current_state_seq(),
            "serverTimeMs": now_ms(),
            "selfId": self_id,
            "self": me,
            "economy": economy_snapshot,
            "itemCatalog": ITEM_CATALOG,
            "dungeon": dungeon_snapshot,
            "players": others,
            "blocks": blocks_snapshot.blocks,
            "blocksVersion": blocks_snapshot.version,
            "chatHistory": chat_history,
            "runway": runway_summary
        }))
    }

    pub async fn build_sync_digest(&self) -> UdpWorldEvent {
        let players = self.active_players_list().await;
        let blocks_version = self.blocks_data.read().await.version;
        let runway = self.runway.read().await.summary();
        UdpWorldEvent::WorldSyncDigest {
            state_seq: self.current_state_seq(),
            server_time_ms: now_ms(),
            blocks_version,
            runway,
            players,
        }
    }

    pub async fn get_cached_sync_digest_json(&self) -> Arc<String> {
        let now = now_ms();
        {
            let guard = self.cached_digest.read().await;
            if now.saturating_sub(guard.0) < 2_000 && !guard.1.is_empty() {
                return Arc::clone(&guard.1);
            }
        }
        let digest = self.build_sync_digest().await;
        let serialized = Arc::new(serde_json::to_string(&digest).unwrap_or_default());
        let mut guard = self.cached_digest.write().await;
        *guard = (now, Arc::clone(&serialized));
        serialized
    }

    pub async fn register_peer_json_channel(
        &self,
        self_id: &str,
        tx: mpsc::UnboundedSender<String>,
    ) {
        self.peer_json_txs
            .write()
            .await
            .insert(self_id.to_string(), tx);
    }

    pub async fn unregister_peer_json_channel(&self, self_id: &str) {
        self.peer_json_txs.write().await.remove(self_id);
    }

    pub async fn send_peer_targeted_json(&self, self_id: &str, json_text: String) -> bool {
        let tx = self.peer_json_txs.read().await.get(self_id).cloned();
        if let Some(sender) = tx {
            sender.send(json_text).is_ok()
        } else {
            false
        }
    }

    pub async fn persist_users_async(&self) {
        self.users_dirty.store(false, Ordering::Release);
        let snapshot = self.users_cfg.read().await.clone();
        let store = Arc::clone(&self.store);
        let _ = tokio::task::spawn_blocking(move || {
            let _ = store.save_users(&snapshot);
        })
        .await;
    }

    pub async fn persist_blocks_async(&self, snapshot: BlocksDataFile) {
        let store = Arc::clone(&self.store);
        let _ = tokio::task::spawn_blocking(move || {
            let _ = store.save_blocks(&snapshot);
        })
        .await;
    }

    pub async fn apply_c2s_state(&self, self_id: &str, decoded: DecodedC2SState) {
        self.apply_c2s_state_ext(self_id, None, false, decoded)
            .await;
    }

    pub async fn active_player_ids(&self) -> Vec<String> {
        self.players.read().await.keys().cloned().collect()
    }

    pub async fn apply_c2s_state_ext(
        &self,
        self_id: &str,
        username_hint: Option<&str>,
        from_udp: bool,
        decoded: DecodedC2SState,
    ) {
        let now = now_ms();
        {
            let mut map = self.players.write().await;
            if let Some(p) = map.get_mut(self_id) {
                let idle_gap = now.saturating_sub(p.last_seen) > 1_500;
                // Only drop genuine short-range out-of-order UDP datagrams on an active burst;
                // never lock out sequence resets after a reconnect, tab reload, or transport switch!
                if !idle_gap
                    && decoded.seq != 0
                    && p.last_seq != 0
                    && decoded.seq < p.last_seq
                    && (p.last_seq - decoded.seq) < 2_000
                {
                    return;
                }
                p.last_seq = decoded.seq;
                p.last_client_time_ms = decoded.client_time_ms;
                if decoded.zone == p.zone {
                    p.x = decoded.x;
                    p.y = decoded.y;
                    p.z = decoded.z;
                    p.ry = decoded.ry;
                    p.anim = decoded.anim;
                    p.emote = decoded.emote;
                }
                p.muted = decoded.muted;
                p.speaking = decoded.speaking;
                p.last_seen = now;
                p.disconnected_at_ms = None;
                if from_udp {
                    p.udp_fastlane_active = true;
                    p.last_udp_seen_ms = now;
                }
                return;
            }
        }

        // Player not currently in map -> lazily re-hydrate then apply once
        if let Some(uname) = username_hint {
            if self.ensure_player_online(self_id, uname).await.is_some() {
                let mut map = self.players.write().await;
                if let Some(p) = map.get_mut(self_id) {
                    p.last_seq = decoded.seq;
                    p.last_client_time_ms = decoded.client_time_ms;
                    if decoded.zone == p.zone {
                        p.x = decoded.x;
                        p.y = decoded.y;
                        p.z = decoded.z;
                        p.ry = decoded.ry;
                        p.anim = decoded.anim;
                        p.emote = decoded.emote;
                    }
                    p.muted = decoded.muted;
                    p.speaking = decoded.speaking;
                    p.last_seen = now;
                    p.disconnected_at_ms = None;
                    if from_udp {
                        p.udp_fastlane_active = true;
                        p.last_udp_seen_ms = now;
                    }
                }
            }
        }
    }

    pub async fn flush_player_positions_async(&self) {
        let updates: Vec<(String, SavedPosition)> = {
            let map = self.players.read().await;
            map.values()
                .filter(|p| p.zone == ZONE_PLAZA && p.y >= 0.0)
                .map(|p| {
                    (
                        p.user.username.to_lowercase(),
                        SavedPosition {
                            x: (p.x * 10.0).round() / 10.0,
                            y: (p.y * 10.0).round() / 10.0,
                            z: (p.z * 10.0).round() / 10.0,
                            ry: (p.ry * 100.0).round() / 100.0,
                        },
                    )
                })
                .collect()
        };
        let dirty_flag = self.users_dirty.swap(false, Ordering::AcqRel);
        if updates.is_empty() && !dirty_flag {
            return;
        }
        let now = now_ms();
        let mut by_user: HashMap<String, SavedPosition> = HashMap::with_capacity(updates.len());
        for (u, pos) in updates {
            by_user.insert(u, pos);
        }
        let mut changed = dirty_flag;
        let snapshot_to_save = {
            let mut cfg = self.users_cfg.write().await;
            for u in cfg.users.iter_mut() {
                if let Some(next_pos) = by_user.get(&u.username.to_lowercase()) {
                    let elapsed = now.saturating_sub(u.last_pos_saved_at.unwrap_or(0));
                    let dist = u
                        .last_pos
                        .as_ref()
                        .map(|prev| {
                            ((prev.x - next_pos.x).powi(2) + (prev.z - next_pos.z).powi(2)).sqrt()
                        })
                        .unwrap_or(999.0);
                    if elapsed > 10_000 && dist > 2.0 {
                        u.last_pos = Some(next_pos.clone());
                        u.last_pos_saved_at = Some(now);
                        changed = true;
                    }
                }
            }
            if changed {
                Some(cfg.clone())
            } else {
                None
            }
        };
        if let Some(snapshot) = snapshot_to_save {
            let store = Arc::clone(&self.store);
            tokio::task::spawn_blocking(move || {
                let _ = store.save_users(&snapshot);
            });
        }
    }

    pub async fn rebuild_pre_encoded_tick(&self, server_tick: u32) {
        let map = self.players.read().await;
        let snap = if map.is_empty() {
            drop(map);
            Arc::new(PreEncodedWorldTick {
                server_tick,
                entries: Vec::new(),
                entries_by_zone: HashMap::new(),
                zone_by_id: HashMap::new(),
                echo_ts_by_id: HashMap::new(),
            })
        } else {
            let mut entries = Vec::with_capacity(map.len());
            let mut entries_by_zone: HashMap<u8, Vec<(String, Bytes)>> = HashMap::new();
            let mut zone_by_id = HashMap::with_capacity(map.len());
            let mut echo_ts_by_id = HashMap::with_capacity(map.len());
            for (id, p) in map.iter() {
                let encoded = encode_player_binary_entry(p);
                entries.push((id.clone(), encoded.clone()));
                entries_by_zone
                    .entry(p.zone)
                    .or_default()
                    .push((id.clone(), encoded));
                zone_by_id.insert(id.clone(), p.zone);
                echo_ts_by_id.insert(id.clone(), p.last_client_time_ms);
            }
            drop(map);
            Arc::new(PreEncodedWorldTick {
                server_tick,
                entries,
                entries_by_zone,
                zone_by_id,
                echo_ts_by_id,
            })
        };
        *self.pre_encoded_tick.write().await = Arc::clone(&snap);
        let _ = self.pre_encoded_tick_tx.send(snap);
    }

    pub fn encode_pre_tick_chunks_for(pre: &PreEncodedWorldTick, self_id: &str) -> Vec<Bytes> {
        let echo_ts = pre.echo_ts_by_id.get(self_id).copied().unwrap_or(0);
        let my_zone = pre.zone_by_id.get(self_id).copied().unwrap_or(ZONE_PLAZA);
        let zone_entries = pre
            .entries_by_zone
            .get(&my_zone)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        assemble_s2c_world_state_chunks(
            pre.server_tick,
            echo_ts,
            zone_entries,
            self_id,
            MAX_SNAPSHOT_MTU_BYTES,
        )
    }

    pub fn encode_pre_tick_scheduled_chunk_for(pre: &PreEncodedWorldTick, self_id: &str) -> Bytes {
        let echo_ts = pre.echo_ts_by_id.get(self_id).copied().unwrap_or(0);
        let my_zone = pre.zone_by_id.get(self_id).copied().unwrap_or(ZONE_PLAZA);
        let zone_entries = pre
            .entries_by_zone
            .get(&my_zone)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        assemble_s2c_world_state_scheduled_chunk(
            pre.server_tick,
            echo_ts,
            zone_entries,
            self_id,
            MAX_SNAPSHOT_MTU_BYTES,
        )
    }

    pub async fn touch_player(&self, self_id: &str) {
        let now = now_ms();
        if let Some(p) = self.players.write().await.get_mut(self_id) {
            p.last_seen = now;
            p.disconnected_at_ms = None;
        }
    }

    pub async fn touch_player_udp(&self, self_id: &str) {
        let now = now_ms();
        if let Some(p) = self.players.write().await.get_mut(self_id) {
            p.last_seen = now;
            p.last_udp_seen_ms = now;
            p.udp_fastlane_active = true;
            p.disconnected_at_ms = None;
        }
    }

    pub async fn encode_world_snapshot_chunks_for(&self, self_id: &str) -> Vec<Bytes> {
        let pre = self.pre_encoded_tick.read().await.clone();
        let current_len = self.players.read().await.len();
        if pre.entries.len() == current_len && (!pre.entries.is_empty() || current_len == 0) {
            return Self::encode_pre_tick_chunks_for(&pre, self_id);
        }

        let map = self.players.read().await;
        let echo_ts = map.get(self_id).map(|p| p.last_client_time_ms).unwrap_or(0);
        let my_zone = map.get(self_id).map(|p| p.zone).unwrap_or(ZONE_PLAZA);
        let entries: Vec<(String, Bytes)> = map
            .iter()
            .filter(|(_, p)| p.zone == my_zone)
            .map(|(id, p)| (id.clone(), encode_player_binary_entry(p)))
            .collect();
        drop(map);
        assemble_s2c_world_state_chunks(
            self.current_tick(),
            echo_ts,
            &entries,
            self_id,
            MAX_SNAPSHOT_MTU_BYTES,
        )
    }

    pub async fn encode_world_snapshot_for(&self, self_id: &str) -> Bytes {
        self.encode_world_snapshot_chunks_for(self_id)
            .await
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    pub async fn mutate_block(
        &self,
        self_id: &str,
        username: &str,
        action: &str,
        coords: (i32, i32, i32),
        color: Option<&str>,
    ) -> Result<(u64, Vec<VoxelBlock>), &'static str> {
        let (x, y, z) = coords;
        if !(-45..=45).contains(&x) || !(1..=60).contains(&y) || !(-45..=45).contains(&z) {
            return Err("Invalid block coordinates");
        }

        let (next_version, snapshot, blocks_file) = {
            let mut bd = self.blocks_data.write().await;
            bd.blocks.retain(|b| !(b.x == x && b.y == y && b.z == z));
            if action != "remove" {
                let valid_color = color
                    .filter(|c| is_valid_hex_color(c))
                    .unwrap_or("#38bdf8")
                    .to_string();
                if bd.blocks.len() >= 5000 {
                    bd.blocks.remove(0);
                }
                bd.blocks.push(VoxelBlock {
                    x,
                    y,
                    z,
                    color: valid_color,
                    author: username.to_string(),
                    placed_at: Some(now_ms()),
                });
            }
            bd.version += 1;
            (bd.version, bd.blocks.clone(), bd.clone())
        };
        self.persist_blocks_async(blocks_file).await;

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::Block {
            state_seq: seq,
            self_id: self_id.to_string(),
            blocks_version: next_version,
            blocks: snapshot.clone(),
        });

        Ok((next_version, snapshot))
    }

    pub async fn update_avatar(
        &self,
        self_id: &str,
        username: &str,
        patch: &serde_json::Value,
    ) -> Option<SanitizedUser> {
        let sanitized = {
            let mut cfg = self.users_cfg.write().await;
            let user = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(username))?;
            user.avatar.apply_patch(patch);
            reconcile_equipped_cosmetics(&mut user.avatar, &user.inventory);
            user.sanitized()
        };
        self.persist_users_async().await;
        let avatar_hash = sanitized.avatar.short_hash();

        {
            let mut map = self.players.write().await;
            for p in map.values_mut() {
                if p.user.username.eq_ignore_ascii_case(username) {
                    p.user = sanitized.clone();
                    p.avatar_hash = avatar_hash.clone();
                }
            }
        }

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::AvatarUpdated {
            state_seq: seq,
            self_id: self_id.to_string(),
            avatar_hash,
            user: sanitized.clone(),
        });

        Some(sanitized)
    }

    pub async fn post_chat(
        &self,
        self_id: &str,
        username: &str,
        raw_text: Option<&str>,
        is_trophy_win: bool,
    ) -> Result<ChatMessage, String> {
        let (disp_name, role, filter_enabled, blocked_words) = {
            let cfg = self.users_cfg.read().await;
            let Some(u) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account is no longer active.".into());
            };
            let s = u.sanitized();
            (
                s.display_name,
                s.role,
                cfg.chat_filter_enabled,
                cfg.blocked_words.clone(),
            )
        };

        let ts = now_ms();
        let clean_text = if is_trophy_win {
            format!("🏆 {disp_name} conquered the Sky Obby and reached the Golden Trophy! 🎉")
        } else {
            raw_text
                .unwrap_or("")
                .chars()
                .map(|c| {
                    if c == '\r' || c == '\n' || c == '\t' {
                        ' '
                    } else {
                        c
                    }
                })
                .collect::<String>()
                .trim()
                .chars()
                .take(180)
                .collect::<String>()
        };

        if clean_text.is_empty() {
            return Err("Chat message cannot be empty.".into());
        }

        if !is_trophy_win {
            if filter_enabled {
                if let Some(matched) = find_blocked_word(&clean_text, &blocked_words) {
                    let blocked = BlockedChatEntry {
                        id: format!("{ts}-{:06x}", rand::random::<u32>() & 0xffffff),
                        sender_id: self_id.to_string(),
                        username: username.to_string(),
                        display_name: disp_name,
                        text: clean_text,
                        reason: "blocked_word".into(),
                        matched_word: Some(matched),
                        ts,
                    };
                    let store = Arc::clone(&self.store);
                    tokio::task::spawn_blocking(move || {
                        let _ = store.append_blocked_chat(&blocked);
                    });
                    self.cached_blocked_chat_count
                        .fetch_add(1, Ordering::Relaxed);
                    return Err("Keep chat friendly! Please avoid inappropriate words.".into());
                }
            }

            let mut cooldowns = self.chat_cooldowns.write().await;
            let q = cooldowns.entry(username.to_lowercase()).or_default();
            while let Some(&oldest) = q.front() {
                if ts.saturating_sub(oldest) > 8_000 {
                    q.pop_front();
                } else {
                    break;
                }
            }
            if q.len() >= 6 {
                let blocked = BlockedChatEntry {
                    id: format!("{ts}-{:06x}", rand::random::<u32>() & 0xffffff),
                    sender_id: self_id.to_string(),
                    username: username.to_string(),
                    display_name: disp_name,
                    text: clean_text,
                    reason: "rate_limit".into(),
                    matched_word: None,
                    ts,
                };
                let store = Arc::clone(&self.store);
                tokio::task::spawn_blocking(move || {
                    let _ = store.append_blocked_chat(&blocked);
                });
                self.cached_blocked_chat_count
                    .fetch_add(1, Ordering::Relaxed);
                return Err(
                    "Slow down a tiny bit! Wait a couple seconds before chatting again.".into(),
                );
            }
            q.push_back(ts);
        }

        let entry = ChatMessage {
            id: format!("{ts}-{:06x}", rand::random::<u32>() & 0xffffff),
            sender_id: self_id.to_string(),
            username: username.to_string(),
            display_name: disp_name,
            role,
            text: clean_text,
            is_system: is_trophy_win,
            is_trophy_win,
            ts,
        };

        {
            let mut chats = self.chats.write().await;
            if chats.len() >= 60 {
                chats.pop_front();
            }
            chats.push_back(entry.clone());
        }
        self.cached_chat_count.fetch_add(1, Ordering::Relaxed);
        let store = Arc::clone(&self.store);
        let entry_for_disk = entry.clone();
        tokio::task::spawn_blocking(move || {
            let _ = store.append_chat(&entry_for_disk);
        });

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::Chat {
            state_seq: seq,
            chat: entry.clone(),
        });

        Ok(entry)
    }

    pub async fn mutate_runway(
        &self,
        self_id: &str,
        username: &str,
        action: &str,
        theme: Option<&str>,
        target_username: Option<&str>,
        stars: Option<u8>,
    ) -> RunwayPublicSummary {
        let mut rw = self.runway.write().await;
        if action == "theme" {
            let next_theme = theme
                .filter(|t| RUNWAY_THEMES.contains(t))
                .map(str::to_string)
                .unwrap_or_else(|| {
                    let idx = RUNWAY_THEMES
                        .iter()
                        .position(|t| *t == rw.theme)
                        .unwrap_or(0);
                    RUNWAY_THEMES[(idx + 1) % RUNWAY_THEMES.len()].to_string()
                });
            rw.theme = next_theme;
            rw.updated_at = now_ms();
            rw.votes_by_target.clear();
            let summary = rw.summary();
            drop(rw);
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::Runway {
                state_seq: seq,
                event: "theme".into(),
                actor_name: Some(username.to_string()),
                voter_id: None,
                voter_name: None,
                target_username: None,
                stars: None,
                runway: summary.clone(),
            });
            return summary;
        }

        if action == "vote" {
            let clean_stars = stars.unwrap_or(5).clamp(1, 5);
            let clean_target = target_username
                .unwrap_or(username)
                .trim()
                .to_lowercase()
                .chars()
                .take(64)
                .collect::<String>();
            let voter_key = username.trim().to_lowercase();
            rw.votes_by_target
                .entry(clean_target.clone())
                .or_default()
                .insert(voter_key, clean_stars);
            rw.updated_at = now_ms();
            let summary = rw.summary();
            drop(rw);
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::Runway {
                state_seq: seq,
                event: "vote".into(),
                actor_name: None,
                voter_id: Some(self_id.to_string()),
                voter_name: Some(username.to_string()),
                target_username: Some(clean_target),
                stars: Some(clean_stars),
                runway: summary.clone(),
            });
            return summary;
        }

        rw.summary()
    }

    pub async fn reset_world_blocks(&self, clear_all: bool) -> Vec<VoxelBlock> {
        let (next_ver, next_blocks, blocks_file) = {
            let mut bd = self.blocks_data.write().await;
            bd.blocks = if clear_all {
                Vec::new()
            } else {
                default_starter_blocks()
            };
            bd.version += 1;
            (bd.version, bd.blocks.clone(), bd.clone())
        };
        self.persist_blocks_async(blocks_file).await;
        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::Block {
            state_seq: seq,
            self_id: "admin".into(),
            blocks_version: next_ver,
            blocks: next_blocks.clone(),
        });
        next_blocks
    }

    // ========================================================================
    // Crystal Cavern Dungeon, Persistent Economy & Live P2P Trading
    // ========================================================================

    pub async fn get_player_economy(&self, username: &str) -> Option<PlayerEconomySnapshot> {
        let cfg = self.users_cfg.read().await;
        cfg.users
            .iter()
            .find(|u| u.username.eq_ignore_ascii_case(username))
            .map(UserAccount::economy_snapshot)
    }

    pub async fn get_peer_zone(&self, self_id: &str) -> u8 {
        self.players
            .read()
            .await
            .get(self_id)
            .map(|p| p.zone)
            .unwrap_or(ZONE_PLAZA)
    }

    pub async fn send_economy_update(&self, self_id: &str, username: &str, reason: &str) {
        let Some(econ) = self.get_player_economy(username).await else {
            return;
        };
        let peer_ids: Vec<String> = {
            let map = self.players.read().await;
            let mut ids: Vec<String> = map
                .values()
                .filter(|p| p.user.username.eq_ignore_ascii_case(username))
                .map(|p| p.id.clone())
                .collect();
            if !ids.iter().any(|id| id == self_id) {
                ids.push(self_id.to_string());
            }
            ids
        };
        let payload = serde_json::json!({
            "type": "economy_update",
            "reason": reason,
            "economy": econ,
        })
        .to_string();
        for pid in peer_ids {
            let _ = self.send_peer_targeted_json(&pid, payload.clone()).await;
        }
    }

    pub async fn tick_dungeon_if_occupied(&self) {
        let any_in_dungeon = {
            let map = self.players.read().await;
            map.values().any(|p| p.zone == ZONE_DUNGEON)
        };
        if !any_in_dungeon {
            return;
        }
        let now = now_ms();
        let (world_changed, mobs, maybe_snapshot) = {
            let mut dg = self.dungeon.write().await;
            let (changed, mobs) = dg.tick(now);
            let snap = if changed { Some(dg.snapshot()) } else { None };
            (changed, mobs, snap)
        };
        let _ = self.udp_event_tx.send(UdpWorldEvent::DungeonMobs {
            server_time_ms: now,
            mobs,
        });
        if world_changed {
            if let Some(dungeon) = maybe_snapshot {
                let seq = self.next_state_seq();
                let _ = self.udp_event_tx.send(UdpWorldEvent::DungeonSync {
                    state_seq: seq,
                    event: Some("tick".into()),
                    actor_id: None,
                    dungeon,
                });
            }
        }
    }

    pub async fn transition_player_zone(
        &self,
        self_id: &str,
        username: &str,
        target_zone: u8,
    ) -> Result<serde_json::Value, String> {
        if target_zone != ZONE_PLAZA && target_zone != ZONE_DUNGEON {
            return Err("Unknown world zone.".into());
        }
        let _ = self
            .ensure_player_online(self_id, username)
            .await
            .ok_or_else(|| "Player session not active.".to_string())?;

        let (nx, ny, nz) = if target_zone == ZONE_DUNGEON {
            DUNGEON_SPAWN_POS
        } else {
            PLAZA_RETURN_SPAWN_POS
        };

        {
            let mut map = self.players.write().await;
            let Some(p) = map.get_mut(self_id) else {
                return Err("Player session not found.".into());
            };
            p.zone = target_zone;
            p.x = nx;
            p.y = ny;
            p.z = nz;
            p.anim = "idle".into();
            p.emote = None;
            p.last_seen = now_ms();
        }

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerZone {
            state_seq: seq,
            id: self_id.to_string(),
            zone: target_zone,
            x: nx,
            y: ny,
            z: nz,
        });

        let dungeon_snapshot = self.dungeon.read().await.snapshot();
        let response = serde_json::json!({
            "ok": true,
            "type": "zone_transitioned",
            "stateSeq": seq,
            "zone": target_zone,
            "x": nx,
            "y": ny,
            "z": nz,
            "plazaPortalPos": [PLAZA_PORTAL_POS.0, PLAZA_PORTAL_POS.1, PLAZA_PORTAL_POS.2],
            "returnPortalPos": [
                DUNGEON_RETURN_PORTAL_POS.0,
                DUNGEON_RETURN_PORTAL_POS.1,
                DUNGEON_RETURN_PORTAL_POS.2
            ],
            "dungeon": dungeon_snapshot,
        });
        let _ = self
            .send_peer_targeted_json(self_id, response.to_string())
            .await;
        Ok(response)
    }

    pub async fn dungeon_interact_node(
        &self,
        self_id: &str,
        username: &str,
        node_id: &str,
    ) -> Result<serde_json::Value, String> {
        let (p_zone, px, py, pz) = {
            let map = self.players.read().await;
            let Some(p) = map.get(self_id) else {
                return Err("Player session not found.".into());
            };
            (p.zone, p.x, p.y, p.z)
        };
        if p_zone != ZONE_DUNGEON {
            return Err("Enter the Crystal Cavern Portal first!".into());
        }

        let (node_kind, nx, ny, nz, node_available) = {
            let dg = self.dungeon.read().await;
            let Some(n) = dg.nodes.iter().find(|n| n.id == node_id) else {
                return Err("Crystal vein or chest not found.".into());
            };
            (n.kind.clone(), n.x, n.y, n.z, n.available)
        };

        if !node_available {
            return Err("That crystal or chest is still recharging!".into());
        }
        let dist = ((px - nx).powi(2) + (py - ny).powi(2) + (pz - nz).powi(2)).sqrt();
        if dist > 6.5 {
            return Err("Walk a little closer to harvest or open it!".into());
        }

        let consumed_key = if node_kind == "golden_chest" {
            let mut cfg = self.users_cfg.write().await;
            let Some(user) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account not found.".into());
            };
            if !remove_inventory_item(&mut user.inventory, "dungeon_key", 1) {
                return Err(
                    "Locked Golden Vault! Farm or trade for a Skeleton Vault Key 🗝️ first.".into(),
                );
            }
            true
        } else {
            false
        };

        let now = now_ms();
        let outcome = {
            let mut dg = self.dungeon.write().await;
            let Some(idx) = dg.nodes.iter().position(|n| n.id == node_id) else {
                return Err("Node disappeared.".into());
            };
            if !dg.nodes[idx].available {
                None
            } else {
                let cd = dg.nodes[idx].cooldown_ms;
                let kind = dg.nodes[idx].kind.clone();
                let drop_x = dg.nodes[idx].x;
                let drop_y = dg.nodes[idx].y + 0.35;
                let drop_z = dg.nodes[idx].z + 0.85;
                dg.nodes[idx].available = false;
                dg.nodes[idx].ready_at_ms = now + cd;

                let (coins, item_opt, qty) = match kind.as_str() {
                    "golden_chest" => roll_golden_chest_loot(),
                    "wooden_chest" => roll_wooden_chest_loot(),
                    _ => roll_crystal_loot(),
                };
                let drop = dg.spawn_drop(drop_x, drop_y, drop_z, coins, item_opt, qty, now);
                let snap = dg.snapshot();
                Some((drop, snap))
            }
        };

        let Some((spawned_drop, snapshot)) = outcome else {
            if consumed_key {
                let mut cfg = self.users_cfg.write().await;
                if let Some(user) = cfg
                    .users
                    .iter_mut()
                    .find(|u| u.username.eq_ignore_ascii_case(username))
                {
                    add_inventory_item(&mut user.inventory, "dungeon_key", 1);
                }
            }
            return Err("Another adventurer just opened that chest!".into());
        };

        if consumed_key {
            self.persist_users_async().await;
            self.send_economy_update(self_id, username, "used_dungeon_key")
                .await;
        }

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::DungeonSync {
            state_seq: seq,
            event: Some(format!("node:{node_id}")),
            actor_id: Some(self_id.to_string()),
            dungeon: snapshot.clone(),
        });

        Ok(serde_json::json!({
            "ok": true,
            "drop": spawned_drop,
            "dungeon": snapshot,
        }))
    }

    pub async fn dungeon_attack_mob(
        &self,
        self_id: &str,
        mob_id: &str,
        mode: &str,
    ) -> Result<serde_json::Value, String> {
        let (p_zone, px, py, pz) = {
            let map = self.players.read().await;
            let Some(p) = map.get(self_id) else {
                return Err("Player session not found.".into());
            };
            (p.zone, p.x, p.y, p.z)
        };
        if p_zone != ZONE_DUNGEON {
            return Err("Enter the Crystal Cavern Portal first!".into());
        }

        let now = now_ms();
        let (defeated, spawned_drop, snapshot) = {
            let mut dg = self.dungeon.write().await;
            let Some(idx) = dg.mobs.iter().position(|m| m.id == mob_id) else {
                return Err("Dungeon creature not found.".into());
            };
            if !dg.mobs[idx].alive {
                return Err("That creature was already defeated!".into());
            }
            if now.saturating_sub(dg.mobs[idx].last_hit_at_ms) < 160 {
                return Err("Attack cooling down...".into());
            }
            let mx = dg.mobs[idx].x;
            let my = dg.mobs[idx].y;
            let mz = dg.mobs[idx].z;
            let dist = ((px - mx).powi(2) + (py - my).powi(2) + (pz - mz).powi(2)).sqrt();
            if dist > 6.2 {
                return Err("Move closer to stomp or Sparkle-Blast that creature!".into());
            }

            let dmg: u8 = if mode == "stomp" { 2 } else { 1 };
            dg.mobs[idx].last_hit_at_ms = now;
            dg.mobs[idx].hp = dg.mobs[idx].hp.saturating_sub(dmg);

            let mut maybe_drop = None;
            let defeated = dg.mobs[idx].hp == 0;
            if defeated {
                let kind = dg.mobs[idx].kind.clone();
                dg.mobs[idx].alive = false;
                dg.mobs[idx].respawn_at_ms = now + 14_000;
                let (coins, item_opt, qty) = roll_mob_loot(&kind);
                maybe_drop = Some(dg.spawn_drop(mx, my.max(0.25), mz, coins, item_opt, qty, now));
            }
            let snap = dg.snapshot();
            (defeated, maybe_drop, snap)
        };

        let seq = self.next_state_seq();
        let event_tag = if defeated {
            format!("mob_defeated:{mob_id}")
        } else {
            format!("mob_hit:{mob_id}")
        };
        let _ = self.udp_event_tx.send(UdpWorldEvent::DungeonSync {
            state_seq: seq,
            event: Some(event_tag),
            actor_id: Some(self_id.to_string()),
            dungeon: snapshot.clone(),
        });

        Ok(serde_json::json!({
            "ok": true,
            "defeated": defeated,
            "drop": spawned_drop,
            "dungeon": snapshot,
        }))
    }

    pub async fn dungeon_pickup_drop(
        &self,
        self_id: &str,
        username: &str,
        drop_id: &str,
    ) -> Result<serde_json::Value, String> {
        let (p_zone, px, py, pz) = {
            let map = self.players.read().await;
            let Some(p) = map.get(self_id) else {
                return Err("Player session not found.".into());
            };
            (p.zone, p.x, p.y, p.z)
        };
        if p_zone != ZONE_DUNGEON {
            return Err("You must be inside the Crystal Cavern to pick up dungeon drops.".into());
        }

        let (picked, snapshot) = {
            let mut dg = self.dungeon.write().await;
            let Some(idx) = dg.drops.iter().position(|d| d.id == drop_id) else {
                return Err("Loot orb already collected!".into());
            };
            let d = &dg.drops[idx];
            let dist = ((px - d.x).powi(2) + (py - d.y).powi(2) + (pz - d.z).powi(2)).sqrt();
            if dist > 6.0 {
                return Err("Walk closer to scoop up that loot!".into());
            }
            let removed = dg.drops.remove(idx);
            let snap = dg.snapshot();
            (removed, snap)
        };

        let econ = {
            let mut cfg = self.users_cfg.write().await;
            let Some(user) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account not found.".into());
            };
            user.coins = user.coins.saturating_add(picked.coins);
            if let Some(ref item_id) = picked.item_id {
                add_inventory_item(&mut user.inventory, item_id, picked.qty.max(1));
            }
            user.economy_snapshot()
        };

        if picked.item_id.is_some() {
            self.persist_users_async().await;
        } else {
            self.users_dirty.store(true, Ordering::Release);
        }

        self.send_economy_update(self_id, username, &format!("pickup:{}", picked.label))
            .await;

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::DungeonSync {
            state_seq: seq,
            event: Some(format!("pickup:{drop_id}")),
            actor_id: Some(self_id.to_string()),
            dungeon: snapshot.clone(),
        });

        Ok(serde_json::json!({
            "ok": true,
            "picked": picked,
            "economy": econ,
            "dungeon": snapshot,
        }))
    }

    pub async fn use_inventory_item(
        &self,
        self_id: &str,
        username: &str,
        item_id: &str,
    ) -> Result<serde_json::Value, String> {
        let Some(def) = find_item_def(item_id) else {
            return Err("Unknown item.".into());
        };
        if def.category == "cosmetic" {
            return Err("Open Glam Closet (👗) to wear your unlocked dungeon cosmetics!".into());
        }
        if def.category != "consumable" {
            return Err(format!(
                "{} is a {} — use Vault Keys on Golden Chests or trade materials with friends!",
                def.name, def.category
            ));
        }

        let (p_zone, px, py, pz) = {
            let map = self.players.read().await;
            let Some(p) = map.get(self_id) else {
                return Err("Player session not found.".into());
            };
            (p.zone, p.x, p.y, p.z)
        };

        if def.id == "prism_block" && p_zone != ZONE_PLAZA {
            return Err(
                "Return through the portal to Starlight Plaza to place Prism Sparkle Blocks!"
                    .into(),
            );
        }

        let econ = {
            let mut cfg = self.users_cfg.write().await;
            let Some(user) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account not found.".into());
            };
            if !remove_inventory_item(&mut user.inventory, def.id, 1) {
                return Err(format!("You don't have any {} left!", def.name));
            }
            user.economy_snapshot()
        };

        let now = now_ms();
        let mut speed_until = 0u64;
        let mut jump_until = 0u64;
        match def.id {
            "speed_potion" => {
                let mut map = self.players.write().await;
                if let Some(p) = map.get_mut(self_id) {
                    p.speed_boost_until_ms = now + 20_000;
                    speed_until = p.speed_boost_until_ms;
                    jump_until = p.jump_boost_until_ms;
                }
            }
            "jump_potion" => {
                let mut map = self.players.write().await;
                if let Some(p) = map.get_mut(self_id) {
                    p.jump_boost_until_ms = now + 20_000;
                    speed_until = p.speed_boost_until_ms;
                    jump_until = p.jump_boost_until_ms;
                }
            }
            "prism_block" => {
                let bx = (px.round() as i32).clamp(-45, 45);
                let by = (py.floor() as i32 + 1).clamp(1, 60);
                let bz = (pz.round() as i32 + 1).clamp(-45, 45);
                if let Err(e) = self
                    .mutate_block(self_id, username, "place", (bx, by, bz), Some("#e879f9"))
                    .await
                {
                    let mut cfg = self.users_cfg.write().await;
                    if let Some(user) = cfg
                        .users
                        .iter_mut()
                        .find(|u| u.username.eq_ignore_ascii_case(username))
                    {
                        add_inventory_item(&mut user.inventory, def.id, 1);
                    }
                    return Err(e.to_string());
                }
            }
            _ => {}
        }

        self.persist_users_async().await;
        let res = serde_json::json!({
            "ok": true,
            "type": "item_used",
            "itemId": def.id,
            "itemName": def.name,
            "speedBoostUntilMs": speed_until,
            "jumpBoostUntilMs": jump_until,
            "economy": econ,
        });
        let _ = self.send_peer_targeted_json(self_id, res.to_string()).await;
        self.send_economy_update(self_id, username, &format!("used:{}", def.id))
            .await;
        Ok(res)
    }

    async fn notify_trade_peers(
        &self,
        sess: &TradeSession,
        event_type: &str,
        reason: Option<&str>,
    ) {
        let mut msg = serde_json::json!({
            "type": event_type,
            "tradeId": sess.trade_id,
            "trade": sess,
        });
        if let Some(r) = reason {
            msg["reason"] = serde_json::Value::String(r.to_string());
        }
        let raw = msg.to_string();
        let _ = self
            .send_peer_targeted_json(&sess.initiator_id, raw.clone())
            .await;
        let _ = self.send_peer_targeted_json(&sess.partner_id, raw).await;
    }

    pub async fn prune_expired_trades(&self) {
        let now = now_ms();
        let expired: Vec<TradeSession> = {
            let mut trades = self.trades.write().await;
            let mut out = Vec::new();
            trades.retain(|_, s| {
                let ttl = if s.status == "pending" {
                    TRADE_INVITE_TTL_MS
                } else {
                    TRADE_ACTIVE_IDLE_TTL_MS
                };
                if now.saturating_sub(s.updated_at_ms) > ttl {
                    out.push(s.clone());
                    false
                } else {
                    true
                }
            });
            out
        };
        for sess in expired {
            self.notify_trade_peers(&sess, "trade_cancelled", Some("Trade expired."))
                .await;
        }
    }

    pub async fn cancel_trades_for_peer(&self, peer_id: &str, reason: &str) {
        let cancelled: Vec<TradeSession> = {
            let mut trades = self.trades.write().await;
            let mut out = Vec::new();
            trades.retain(|_, s| {
                if s.involves_peer(peer_id) {
                    out.push(s.clone());
                    false
                } else {
                    true
                }
            });
            out
        };
        for sess in cancelled {
            self.notify_trade_peers(&sess, "trade_cancelled", Some(reason))
                .await;
        }
    }

    pub async fn trade_invite(
        &self,
        self_id: &str,
        username: &str,
        target_peer_id: &str,
    ) -> Result<TradeSession, String> {
        let target_peer_id = target_peer_id.trim();
        if target_peer_id.is_empty() || target_peer_id == self_id {
            return Err("Choose an online friend to trade with!".into());
        }

        let (init_disp, part_user, part_disp) = {
            let map = self.players.read().await;
            let Some(me) = map.get(self_id) else {
                return Err("Your session is not online.".into());
            };
            let Some(them) = map.get(target_peer_id) else {
                return Err("That friend is no longer online.".into());
            };
            if me.user.username.eq_ignore_ascii_case(&them.user.username) {
                return Err("You cannot trade between two tabs of the same account.".into());
            }
            (
                me.user.display_name.clone(),
                them.user.username.clone(),
                them.user.display_name.clone(),
            )
        };

        let now = now_ms();
        let mut stale_pending = Vec::new();
        let sess = {
            let mut trades = self.trades.write().await;
            for existing in trades.values() {
                if existing.status == "active" && existing.involves_peer(self_id) {
                    return Err("Finish or cancel your current trade first.".into());
                }
                if existing.status == "active" && existing.involves_peer(target_peer_id) {
                    return Err(format!("{part_disp} is already in another live trade!"));
                }
            }
            trades.retain(|_, existing| {
                if existing.status == "pending" && existing.initiator_id == self_id {
                    stale_pending.push(existing.clone());
                    false
                } else {
                    true
                }
            });

            let trade_id = format!("trd_{now}_{:06x}", rand::random::<u32>() & 0xffffff);
            let created = TradeSession {
                trade_id: trade_id.clone(),
                status: "pending".into(),
                initiator_id: self_id.to_string(),
                initiator_username: username.to_string(),
                initiator_display_name: init_disp,
                partner_id: target_peer_id.to_string(),
                partner_username: part_user,
                partner_display_name: part_disp,
                initiator_offer: TradeSideOffer::default(),
                partner_offer: TradeSideOffer::default(),
                offer_revision: 0,
                status_note: Some("Waiting for friend to accept trade invite...".into()),
                created_at_ms: now,
                updated_at_ms: now,
            };
            trades.insert(trade_id, created.clone());
            created
        };

        for old in stale_pending {
            self.notify_trade_peers(&old, "trade_cancelled", Some("Replaced by a new invite."))
                .await;
        }

        let invite_msg = serde_json::json!({
            "type": "trade_invite_received",
            "tradeId": sess.trade_id,
            "trade": sess,
        })
        .to_string();
        let _ = self
            .send_peer_targeted_json(&sess.partner_id, invite_msg)
            .await;

        let sent_msg = serde_json::json!({
            "type": "trade_invite_sent",
            "tradeId": sess.trade_id,
            "trade": sess,
        })
        .to_string();
        let _ = self
            .send_peer_targeted_json(&sess.initiator_id, sent_msg)
            .await;

        Ok(sess)
    }

    pub async fn trade_respond(
        &self,
        self_id: &str,
        trade_id: &str,
        accept: bool,
    ) -> Result<Option<TradeSession>, String> {
        let now = now_ms();
        let mut other_cancelled = Vec::new();

        if !accept {
            let removed = {
                let mut trades = self.trades.write().await;
                let Some(s) = trades.get(trade_id) else {
                    return Err("Trade invite expired or was already closed.".into());
                };
                if !s.involves_peer(self_id) {
                    return Err("You are not part of this trade.".into());
                }
                trades.remove(trade_id)
            };
            if let Some(sess) = removed {
                self.notify_trade_peers(&sess, "trade_cancelled", Some("Trade invite declined."))
                    .await;
            }
            return Ok(None);
        }

        let accepted_sess = {
            let mut trades = self.trades.write().await;
            let (init_id, part_id) = {
                let Some(s) = trades.get(trade_id) else {
                    return Err("Trade invite expired or was already closed.".into());
                };
                if s.partner_id != self_id {
                    return Err("Only the invited friend can accept this trade.".into());
                }
                if s.status != "pending" {
                    return Err("Trade was already accepted.".into());
                }
                (s.initiator_id.clone(), s.partner_id.clone())
            };

            let busy = trades.values().any(|other| {
                other.trade_id != trade_id
                    && other.status == "active"
                    && (other.involves_peer(&init_id) || other.involves_peer(&part_id))
            });
            if busy {
                return Err("One of the players is already in another active trade.".into());
            }

            trades.retain(|id, other| {
                if id != trade_id
                    && other.status == "pending"
                    && (other.involves_peer(&init_id) || other.involves_peer(&part_id))
                {
                    other_cancelled.push(other.clone());
                    false
                } else {
                    true
                }
            });

            let s = trades.get_mut(trade_id).expect("verified trade exists");
            s.status = "active".into();
            s.updated_at_ms = now;
            s.status_note = Some("Add items or coins, then click Lock Offer!".into());
            s.clone()
        };

        for c in other_cancelled {
            self.notify_trade_peers(&c, "trade_cancelled", Some("Player started another trade."))
                .await;
        }

        self.notify_trade_peers(&accepted_sess, "trade_updated", None)
            .await;
        Ok(Some(accepted_sess))
    }

    pub async fn trade_update_offer(
        &self,
        self_id: &str,
        username: &str,
        trade_id: &str,
        coins: u64,
        items: &HashMap<String, u32>,
    ) -> Result<TradeSession, String> {
        let (owned_coins, owned_inv) = {
            let cfg = self.users_cfg.read().await;
            let Some(user) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account not found.".into());
            };
            (user.coins, user.inventory.clone())
        };

        let (clean_coins, clean_items) =
            sanitize_and_validate_offer(coins, items, owned_coins, &owned_inv)?;

        let now = now_ms();
        let updated = {
            let mut trades = self.trades.write().await;
            let Some(sess) = trades.get_mut(trade_id) else {
                return Err("Trade is no longer open.".into());
            };
            sess.apply_validated_offer(self_id, clean_coins, clean_items, now)?;
            sess.clone()
        };

        self.notify_trade_peers(&updated, "trade_updated", None)
            .await;
        Ok(updated)
    }

    pub async fn trade_set_lock(
        &self,
        self_id: &str,
        trade_id: &str,
        locked: bool,
    ) -> Result<TradeSession, String> {
        let now = now_ms();
        let updated = {
            let mut trades = self.trades.write().await;
            let Some(sess) = trades.get_mut(trade_id) else {
                return Err("Trade is no longer open.".into());
            };
            sess.set_locked(self_id, locked, now)?;
            sess.clone()
        };

        self.notify_trade_peers(&updated, "trade_updated", None)
            .await;
        Ok(updated)
    }

    pub async fn trade_confirm(
        &self,
        self_id: &str,
        trade_id: &str,
    ) -> Result<serde_json::Value, String> {
        let now = now_ms();
        let (both_confirmed, target_sess) = {
            let mut trades = self.trades.write().await;
            let Some(sess) = trades.get_mut(trade_id) else {
                return Err("Trade is no longer open.".into());
            };
            let both = sess.mark_confirmed(self_id, now)?;
            if both {
                let removed = trades.remove(trade_id).expect("verified trade exists");
                (true, removed)
            } else {
                (false, sess.clone())
            }
        };

        if !both_confirmed {
            self.notify_trade_peers(&target_sess, "trade_updated", None)
                .await;
            return Ok(serde_json::json!({
                "ok": true,
                "completed": false,
                "trade": target_sess,
            }));
        }

        // Execute atomic swap under a SINGLE write guard on `users_cfg`
        let swap_outcome = {
            let mut cfg = self.users_cfg.write().await;
            let idx_a = cfg.users.iter().position(|u| {
                u.username
                    .eq_ignore_ascii_case(&target_sess.initiator_username)
            });
            let idx_b = cfg.users.iter().position(|u| {
                u.username
                    .eq_ignore_ascii_case(&target_sess.partner_username)
            });

            match (idx_a, idx_b) {
                (Some(ia), Some(ib)) if ia != ib => {
                    let (user_a, user_b) = if ia < ib {
                        let (left, right) = cfg.users.split_at_mut(ib);
                        (&mut left[ia], &mut right[0])
                    } else {
                        let (left, right) = cfg.users.split_at_mut(ia);
                        (&mut right[0], &mut left[ib])
                    };
                    execute_atomic_trade_swap(
                        user_a,
                        &target_sess.initiator_offer,
                        user_b,
                        &target_sess.partner_offer,
                    )
                    .map(|(av_a, av_b)| {
                        (
                            av_a,
                            av_b,
                            user_a.sanitized(),
                            user_b.sanitized(),
                            user_a.economy_snapshot(),
                            user_b.economy_snapshot(),
                        )
                    })
                }
                _ => Err("One of the trading accounts could not be found.".into()),
            }
        };

        let (av_a_changed, av_b_changed, san_a, san_b, econ_a, econ_b) = match swap_outcome {
            Ok(vals) => vals,
            Err(err_msg) => {
                let mut reopened = target_sess;
                reopened.initiator_offer.locked = false;
                reopened.initiator_offer.confirmed = false;
                reopened.partner_offer.locked = false;
                reopened.partner_offer.confirmed = false;
                reopened.status_note = Some(err_msg.clone());
                reopened.updated_at_ms = now_ms();
                {
                    let mut trades = self.trades.write().await;
                    let busy = trades.values().any(|other| {
                        other.status == "active"
                            && (other.involves_peer(&reopened.initiator_id)
                                || other.involves_peer(&reopened.partner_id))
                    });
                    if !busy {
                        trades.insert(reopened.trade_id.clone(), reopened.clone());
                    }
                }
                self.notify_trade_peers(&reopened, "trade_updated", Some(&err_msg))
                    .await;
                return Err(err_msg);
            }
        };

        self.persist_users_async().await;

        // Broadcast immediate avatar updates if either player traded away an equipped dungeon item
        if av_a_changed {
            let hash_a = san_a.avatar.short_hash();
            {
                let mut map = self.players.write().await;
                for p in map.values_mut() {
                    if p.user.username.eq_ignore_ascii_case(&san_a.username) {
                        p.user = san_a.clone();
                        p.avatar_hash = hash_a.clone();
                    }
                }
            }
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::AvatarUpdated {
                state_seq: seq,
                self_id: target_sess.initiator_id.clone(),
                avatar_hash: hash_a,
                user: san_a,
            });
        }
        if av_b_changed {
            let hash_b = san_b.avatar.short_hash();
            {
                let mut map = self.players.write().await;
                for p in map.values_mut() {
                    if p.user.username.eq_ignore_ascii_case(&san_b.username) {
                        p.user = san_b.clone();
                        p.avatar_hash = hash_b.clone();
                    }
                }
            }
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::AvatarUpdated {
                state_seq: seq,
                self_id: target_sess.partner_id.clone(),
                avatar_hash: hash_b,
                user: san_b,
            });
        }

        self.send_economy_update(
            &target_sess.initiator_id,
            &target_sess.initiator_username,
            "trade_completed",
        )
        .await;
        self.send_economy_update(
            &target_sess.partner_id,
            &target_sess.partner_username,
            "trade_completed",
        )
        .await;

        let done_a = serde_json::json!({
            "type": "trade_completed",
            "tradeId": target_sess.trade_id,
            "partnerName": target_sess.partner_display_name,
            "economy": econ_a,
        })
        .to_string();
        let done_b = serde_json::json!({
            "type": "trade_completed",
            "tradeId": target_sess.trade_id,
            "partnerName": target_sess.initiator_display_name,
            "economy": econ_b,
        })
        .to_string();
        let _ = self
            .send_peer_targeted_json(&target_sess.initiator_id, done_a)
            .await;
        let _ = self
            .send_peer_targeted_json(&target_sess.partner_id, done_b)
            .await;

        Ok(serde_json::json!({
            "ok": true,
            "completed": true,
            "trade": target_sess,
            "initiatorEconomy": econ_a,
            "partnerEconomy": econ_b,
        }))
    }

    pub async fn trade_cancel(&self, self_id: &str, trade_id: Option<&str>) -> Result<(), String> {
        if let Some(tid) = trade_id.filter(|s| !s.trim().is_empty()) {
            let removed = {
                let mut trades = self.trades.write().await;
                let Some(s) = trades.get(tid) else {
                    return Ok(());
                };
                if !s.involves_peer(self_id) {
                    return Err("You are not part of this trade.".into());
                }
                trades.remove(tid)
            };
            if let Some(sess) = removed {
                self.notify_trade_peers(&sess, "trade_cancelled", Some("Trade cancelled."))
                    .await;
            }
        } else {
            self.cancel_trades_for_peer(self_id, "Trade cancelled.")
                .await;
        }
        Ok(())
    }
}

fn normalize_leet_char(c: char) -> Option<char> {
    let lc = c.to_ascii_lowercase();
    match lc {
        'a'..='z' => Some(lc),
        '0' => Some('o'),
        '1' | '!' => Some('i'),
        '3' => Some('e'),
        '4' | '@' => Some('a'),
        '5' | '$' => Some('s'),
        '7' => Some('t'),
        _ => None,
    }
}

fn collapse_repeats(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev: Option<char> = None;
    for c in s.chars() {
        if prev != Some(c) {
            out.push(c);
            prev = Some(c);
        }
    }
    out
}

pub fn find_blocked_word(text: &str, blocked_words: &[String]) -> Option<String> {
    if blocked_words.is_empty() {
        return None;
    }

    let normalized_blocked: Vec<String> = blocked_words
        .iter()
        .map(|w| {
            w.trim()
                .chars()
                .filter_map(normalize_leet_char)
                .collect::<String>()
        })
        .filter(|w| !w.is_empty())
        .collect();

    const SAFE_SUBSTRING_ROOTS: &[&str] = &[
        "fuck", "shit", "bitch", "asshole", "nigger", "nigga", "faggot", "cunt", "whore", "slut",
    ];

    for raw_token in text.split_whitespace() {
        let alnum_token: String = raw_token.chars().filter_map(normalize_leet_char).collect();
        if alnum_token.is_empty() {
            continue;
        }
        let collapsed = collapse_repeats(&alnum_token);

        for bw in &normalized_blocked {
            let collapsed_bw = collapse_repeats(bw);
            if alnum_token == *bw || collapsed == collapsed_bw {
                return Some(bw.clone());
            }
            if bw.len() >= 4
                && SAFE_SUBSTRING_ROOTS.iter().any(|root| bw.contains(root))
                && (alnum_token.contains(bw) || collapsed.contains(&collapsed_bw))
            {
                return Some(bw.clone());
            }
        }

        // Also check strict alphanumeric sub-segments split on non-alnum (e.g. "hi,bitch!")
        for sub in raw_token.split(|c: char| !c.is_ascii_alphanumeric()) {
            let sub_norm: String = sub.chars().filter_map(normalize_leet_char).collect();
            if sub_norm.is_empty() {
                continue;
            }
            let sub_col = collapse_repeats(&sub_norm);
            for bw in &normalized_blocked {
                if sub_norm == *bw || sub_col == collapse_repeats(bw) {
                    return Some(bw.clone());
                }
            }
        }
    }

    None
}

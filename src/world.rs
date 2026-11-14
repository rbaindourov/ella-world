use crate::auth::{sign_token, verify_token, TokenClaims};
use crate::config::ServerConfig;
use crate::models::{
    default_starter_blocks, is_valid_hex_color, now_ms, BlockedChatEntry, BlocksDataFile,
    ChatMessage, PlayerRuntimeState, RunwayPublicSummary, RunwayState, SanitizedUser,
    SavedPosition, UserAccount, UsersConfigFile, VoxelBlock, RUNWAY_THEMES,
};
use crate::persistence::PersistenceStore;
use crate::udp_wire::{encode_s2c_world_state, DecodedC2SState};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

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
}

#[derive(Clone, Debug)]
pub struct BridgedOpusFrame {
    pub from_id: String,
    pub from_native_udp: bool,
    pub seq: u16,
    pub timestamp: u32,
    pub payload: Bytes,
}

pub struct WorldEngine {
    pub config: ServerConfig,
    pub store: PersistenceStore,
    pub users_cfg: RwLock<UsersConfigFile>,
    pub blocks_data: RwLock<BlocksDataFile>,
    pub players: RwLock<HashMap<String, PlayerRuntimeState>>,
    pub chats: RwLock<VecDeque<ChatMessage>>,
    pub chat_cooldowns: RwLock<HashMap<String, VecDeque<u64>>>,
    pub runway: RwLock<RunwayState>,
    pub server_tick: AtomicU32,
    pub state_seq: AtomicU64,
    pub disconnect_grace_ms: AtomicU64,
    pub udp_event_tx: broadcast::Sender<UdpWorldEvent>,
    pub opus_bridge_tx: broadcast::Sender<BridgedOpusFrame>,
}

impl WorldEngine {
    pub fn new(config: ServerConfig) -> anyhow::Result<Arc<Self>> {
        let store = PersistenceStore::new(&config.data_dir, &config.seed_users_file)?;
        let users_cfg = store.load_or_init_users()?;
        let blocks_data = store.load_or_init_blocks()?;
        let recent_chats = store.load_recent_chats(60);
        let mut chat_deque = VecDeque::with_capacity(64);
        for c in recent_chats {
            chat_deque.push_back(c);
        }
        let (udp_event_tx, _) = broadcast::channel(1024);
        let (opus_bridge_tx, _) = broadcast::channel(1024);

        let engine = Arc::new(Self {
            config,
            store,
            users_cfg: RwLock::new(users_cfg),
            blocks_data: RwLock::new(blocks_data),
            players: RwLock::new(HashMap::new()),
            chats: RwLock::new(chat_deque),
            chat_cooldowns: RwLock::new(HashMap::new()),
            runway: RwLock::new(RunwayState::default()),
            server_tick: AtomicU32::new(1),
            state_seq: AtomicU64::new(1),
            disconnect_grace_ms: AtomicU64::new(DEFAULT_DISCONNECT_GRACE_MS),
            udp_event_tx,
            opus_bridge_tx,
        });

        let bg = Arc::clone(&engine);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
            let mut tick_counter = 0u32;
            loop {
                interval.tick().await;
                bg.server_tick.fetch_add(1, Ordering::Relaxed);
                tick_counter = tick_counter.wrapping_add(1);
                if tick_counter.is_multiple_of(10) {
                    bg.prune_stale_players().await;
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
            let seq = self.next_state_seq();
            let _ = self
                .udp_event_tx
                .send(UdpWorldEvent::PlayerLeave { state_seq: seq, id });
        }
    }

    pub async fn remove_player_immediately(&self, self_id: &str) {
        let existed = self.players.write().await.remove(self_id).is_some();
        if existed {
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
        let cfg = self.users_cfg.read().await;

        serde_json::json!({
            "token": token,
            "selfId": self_id,
            "self": player_doc,
            "user": sanitized,
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

        Some(serde_json::json!({
            "type": "welcome_snapshot",
            "stateSeq": self.current_state_seq(),
            "serverTimeMs": now_ms(),
            "selfId": self_id,
            "self": me,
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

    pub async fn apply_c2s_state(&self, self_id: &str, decoded: DecodedC2SState) {
        self.apply_c2s_state_ext(self_id, None, false, decoded)
            .await;
    }

    pub async fn apply_c2s_state_ext(
        &self,
        self_id: &str,
        username_hint: Option<&str>,
        from_udp: bool,
        decoded: DecodedC2SState,
    ) {
        let missing = !self.players.read().await.contains_key(self_id);
        if missing {
            if let Some(uname) = username_hint {
                let _ = self.ensure_player_online(self_id, uname).await;
            }
        }

        let now = now_ms();
        let mut maybe_save_pos: Option<(String, SavedPosition)> = None;

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
                p.x = decoded.x;
                p.y = decoded.y;
                p.z = decoded.z;
                p.ry = decoded.ry;
                p.anim = decoded.anim;
                p.emote = decoded.emote;
                p.muted = decoded.muted;
                p.speaking = decoded.speaking;
                p.last_seen = now;
                p.disconnected_at_ms = None;
                if from_udp {
                    p.udp_fastlane_active = true;
                    p.last_udp_seen_ms = now;
                }

                if p.y >= 0.0 {
                    maybe_save_pos = Some((
                        p.user.username.clone(),
                        SavedPosition {
                            x: (p.x * 10.0).round() / 10.0,
                            y: (p.y * 10.0).round() / 10.0,
                            z: (p.z * 10.0).round() / 10.0,
                            ry: (p.ry * 100.0).round() / 100.0,
                        },
                    ));
                }
            }
        }

        // Throttled persistence of builder lastPos (at most once every 10s when moved > 2.0m)
        if let Some((username, next_pos)) = maybe_save_pos {
            let mut should_write = false;
            {
                let mut cfg = self.users_cfg.write().await;
                if let Some(u) = cfg
                    .users
                    .iter_mut()
                    .find(|u| u.username.eq_ignore_ascii_case(&username))
                {
                    let elapsed = now.saturating_sub(u.last_pos_saved_at.unwrap_or(0));
                    let dist = u
                        .last_pos
                        .as_ref()
                        .map(|prev| {
                            ((prev.x - next_pos.x).powi(2) + (prev.z - next_pos.z).powi(2)).sqrt()
                        })
                        .unwrap_or(999.0);
                    if elapsed > 10_000 && dist > 2.0 {
                        u.last_pos = Some(next_pos);
                        u.last_pos_saved_at = Some(now);
                        should_write = true;
                    }
                }
                if should_write {
                    let _ = self.store.save_users(&cfg);
                }
            }
        }
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

    pub async fn encode_world_snapshot_for(&self, self_id: &str) -> Bytes {
        let map = self.players.read().await;
        let echo_ts = map.get(self_id).map(|p| p.last_client_time_ms).unwrap_or(0);
        let list: Vec<PlayerRuntimeState> = map.values().cloned().collect();
        drop(map);
        encode_s2c_world_state(self.current_tick(), echo_ts, &list, self_id)
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

        let (next_version, snapshot) = {
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
            let _ = self.store.save_blocks(&bd);
            (bd.version, bd.blocks.clone())
        };

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
            let s = user.sanitized();
            let _ = self.store.save_users(&cfg);
            s
        };
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
                    let _ = self.store.append_blocked_chat(&blocked);
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
                let _ = self.store.append_blocked_chat(&blocked);
                return Err("Slow down a tiny bit! Wait a couple seconds before chatting again.".into());
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
        let _ = self.store.append_chat(&entry);

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
        let (next_ver, next_blocks) = {
            let mut bd = self.blocks_data.write().await;
            bd.blocks = if clear_all {
                Vec::new()
            } else {
                default_starter_blocks()
            };
            bd.version += 1;
            let _ = self.store.save_blocks(&bd);
            (bd.version, bd.blocks.clone())
        };
        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::Block {
            state_seq: seq,
            self_id: "admin".into(),
            blocks_version: next_ver,
            blocks: next_blocks.clone(),
        });
        next_blocks
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

    const SAFE_SUBSTRING_ROOTS: &[&str] = &["fuck", "shit", "bitch", "asshole", "nigger", "nigga", "faggot", "cunt", "whore", "slut"];

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

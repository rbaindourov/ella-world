use crate::auth::{
    constant_time_str_eq, password_token_sig, sign_token, verify_token, TokenClaims,
};
use crate::config::ServerConfig;
use crate::device_sig::{
    active_penalties_for_user, compute_hw_exact_sig, compute_hw_xbrowser_sig,
    compute_net_os_fallback_sig, match_existing_device, mint_deterministic_or_fresh_device_id,
    short_sig_from_device_id, sign_machine_seal, verify_machine_seal, ClientDeviceTelemetry,
    DeviceSealClaims,
};
use crate::economy::{
    add_inventory_item, calculate_escalating_fine, execute_atomic_trade_swap, find_item_def,
    player_owns_item, reconcile_equipped_cosmetics, remove_inventory_item, roll_crystal_loot,
    roll_golden_chest_loot, roll_mob_loot, roll_wooden_chest_loot, sanitize_and_validate_offer,
    DungeonMob, DungeonSnapshot, DungeonState, PrisonSnapshot, PrisonState, TradeSession,
    TradeSideOffer, DUNGEON_RETURN_PORTAL_POS, DUNGEON_SPAWN_POS, ITEM_CATALOG, PLAZA_PORTAL_POS,
    PLAZA_RETURN_SPAWN_POS, PRISON_RELEASE_PORTAL_POS, PRISON_SPAWN_POS, PRISON_WORK_REWARD_COINS,
    TRADE_ACTIVE_IDLE_TTL_MS, TRADE_INVITE_TTL_MS,
};
use crate::models::{
    default_block_kind, default_starter_blocks, is_toggleable_block_kind, is_two_tall_block_kind,
    is_valid_block_kind, is_valid_hex_color, normalize_chat_mode, now_ms, BlockedChatEntry,
    BlocksDataFile, ChatMessage, DeviceEvasionEntry, MachineDeviceRecord, PlayerEconomySnapshot,
    PlayerModerationSnapshot, PlayerRuntimeState, RunwayPublicSummary, RunwayState, SanitizedUser,
    SavedPosition, UserAccount, UsersConfigFile, VoxelBlock, RUNWAY_THEMES,
    SAFE_QUICK_CHAT_PHRASES,
};
use crate::persistence::PersistenceStore;
use crate::udp_wire::{
    assemble_s2c_world_state_chunks, assemble_s2c_world_state_scheduled_chunk,
    encode_player_binary_entry, DecodedC2SState, MAX_SNAPSHOT_MTU_BYTES, ZONE_DUNGEON, ZONE_PLAZA,
    ZONE_PRISON,
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
    PrisonSync {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        event: Option<String>,
        #[serde(rename = "actorId", skip_serializing_if = "Option::is_none")]
        actor_id: Option<String>,
        prison: PrisonSnapshot,
    },
    RayGunBeam {
        #[serde(rename = "stateSeq")]
        state_seq: u64,
        #[serde(rename = "actorId")]
        actor_id: String,
        zone: u8,
        from: [f32; 3],
        to: [f32; 3],
        #[serde(rename = "hitMobId", skip_serializing_if = "Option::is_none")]
        hit_mob_id: Option<String>,
    },
}

#[derive(Clone, Debug, Default)]
pub struct BlockMutationInput<'a> {
    pub action: &'a str,
    pub coords: (i32, i32, i32),
    pub color: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub rot: Option<u8>,
    pub loot_item_id: Option<&'a str>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminModerateUserInput {
    pub username: String,
    pub action: String, // "update_controls" | "sentence_prison" | "pardon_prison" | "adjust_coins" | "reset_strikes"
    pub voice_muted_by_parent: Option<bool>,
    pub chat_mode: Option<String>,
    pub can_build: Option<bool>,
    pub can_trade: Option<bool>,
    pub suspended: Option<bool>,
    pub coin_delta: Option<i64>,
    pub prison_tasks: Option<u32>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BridgedOpusFrame {
    pub from_id: String,
    pub from_native_udp: bool,
    pub seq: u16,
    pub timestamp: u32,
    pub payload: Bytes,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminManageDeviceInput {
    pub device_id: String,
    pub action: String, // "set_flags" | "unlink_users"
    pub trusted_shared: Option<bool>,
    pub banned: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct ResolvedMachineSession {
    pub device_id: String,
    pub short_sig: String,
    pub seal_token: String,
}

#[derive(Clone, Debug)]
pub enum SessionRevokeSignal {
    All {
        reason: String,
    },
    Username {
        username_lower: String,
        reason: String,
    },
    PeerId {
        self_id: String,
        reason: String,
    },
    DeviceOtherUsers {
        device_id: String,
        allowed_username_lower: String,
        reason: String,
    },
}

impl SessionRevokeSignal {
    pub fn matches(&self, self_id: &str, username: &str, device_id: Option<&str>) -> bool {
        match self {
            Self::All { .. } => true,
            Self::Username { username_lower, .. } => {
                username.trim().eq_ignore_ascii_case(username_lower)
            }
            Self::PeerId {
                self_id: target_id, ..
            } => self_id == target_id,
            Self::DeviceOtherUsers {
                device_id: target_dev,
                allowed_username_lower,
                ..
            } => {
                device_id == Some(target_dev.as_str())
                    && !username.trim().eq_ignore_ascii_case(allowed_username_lower)
            }
        }
    }

    pub fn reason(&self) -> &str {
        match self {
            Self::All { reason }
            | Self::Username { reason, .. }
            | Self::PeerId { reason, .. }
            | Self::DeviceOtherUsers { reason, .. } => reason,
        }
    }
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
    pub prison: RwLock<PrisonState>,
    pub prison_work_cooldowns: RwLock<HashMap<String, u64>>,
    pub raygun_cooldowns: RwLock<HashMap<String, u64>>,
    pub trades: RwLock<HashMap<String, TradeSession>>,
    pub global_voice_enabled: AtomicBool,
    pub server_tick: AtomicU32,
    pub state_seq: AtomicU64,
    pub disconnect_grace_ms: AtomicU64,
    pub users_dirty: AtomicBool,
    pub cached_chat_count: AtomicUsize,
    pub cached_blocked_chat_count: AtomicUsize,
    pub cached_device_evasion_count: AtomicUsize,
    pub pre_encoded_tick: RwLock<Arc<PreEncodedWorldTick>>,
    pub pre_encoded_tick_tx: watch::Sender<Arc<PreEncodedWorldTick>>,
    pub cached_digest: RwLock<(u64, Arc<String>)>,
    pub peer_json_txs: RwLock<HashMap<String, mpsc::UnboundedSender<String>>>,
    pub udp_event_tx: broadcast::Sender<UdpWorldEvent>,
    pub opus_bridge_tx: broadcast::Sender<BridgedOpusFrame>,
    pub session_revoke_tx: broadcast::Sender<SessionRevokeSignal>,
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
        let initial_evasion_count = store.count_device_evasions();
        let recent_chats = store.load_recent_chats(60);
        let mut chat_deque = VecDeque::with_capacity(64);
        for c in recent_chats {
            chat_deque.push_back(c);
        }
        let (udp_event_tx, _) = broadcast::channel(4096);
        let (opus_bridge_tx, _) = broadcast::channel(2048);
        let (session_revoke_tx, _) = broadcast::channel(256);
        let initial_tick_arc = Arc::new(PreEncodedWorldTick::default());
        let (pre_encoded_tick_tx, _) = watch::channel(Arc::clone(&initial_tick_arc));

        let initial_global_voice = users_cfg.global_voice_enabled;
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
            prison: RwLock::new(PrisonState::new()),
            prison_work_cooldowns: RwLock::new(HashMap::new()),
            raygun_cooldowns: RwLock::new(HashMap::new()),
            trades: RwLock::new(HashMap::new()),
            global_voice_enabled: AtomicBool::new(initial_global_voice),
            server_tick: AtomicU32::new(1),
            state_seq: AtomicU64::new(1),
            disconnect_grace_ms: AtomicU64::new(DEFAULT_DISCONNECT_GRACE_MS),
            users_dirty: AtomicBool::new(false),
            cached_chat_count: AtomicUsize::new(initial_chat_count),
            cached_blocked_chat_count: AtomicUsize::new(initial_blocked_count),
            cached_device_evasion_count: AtomicUsize::new(initial_evasion_count),
            pre_encoded_tick: RwLock::new(initial_tick_arc),
            pre_encoded_tick_tx,
            cached_digest: RwLock::new((0, Arc::new(String::new()))),
            peer_json_txs: RwLock::new(HashMap::new()),
            udp_event_tx,
            opus_bridge_tx,
            session_revoke_tx,
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
                    bg.tick_prison_if_occupied().await;
                }
                if tick_counter.is_multiple_of(40) {
                    bg.enforce_live_world_access().await;
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

    pub fn player_password_sig(&self, user: &UserAccount) -> String {
        password_token_sig(&user.username, &user.password, &self.config.hmac_secret)
    }

    pub fn admin_password_sig(&self, admin_password: &str) -> String {
        password_token_sig("__admin__", admin_password, &self.config.hmac_secret)
    }

    pub fn mint_player_token(
        &self,
        self_id: &str,
        user: &UserAccount,
        session_epoch: u64,
        device_id: Option<&str>,
    ) -> String {
        let now = now_ms();
        let exp = now.saturating_add(self.config.token_ttl_ms());
        let pwd_sig = self.player_password_sig(user);
        self.sign_session_token(&TokenClaims {
            self_id: Some(self_id.to_string()),
            username: Some(user.username.clone()),
            role: Some(user.role.clone()),
            iat: now,
            exp: Some(exp),
            session_epoch: Some(session_epoch),
            user_token_epoch: Some(user.token_epoch),
            pwd_sig: Some(pwd_sig),
            device_id: device_id.map(str::to_string),
        })
    }

    pub async fn mint_admin_token(&self) -> String {
        let (session_epoch, admin_pw) = {
            let cfg = self.users_cfg.read().await;
            (cfg.session_epoch, cfg.admin_password.clone())
        };
        let now = now_ms();
        let exp = now.saturating_add(self.config.token_ttl_ms());
        let pwd_sig = self.admin_password_sig(&admin_pw);
        self.sign_session_token(&TokenClaims {
            self_id: None,
            username: Some("admin".into()),
            role: Some("admin".into()),
            iat: now,
            exp: Some(exp),
            session_epoch: Some(session_epoch),
            user_token_epoch: Some(0),
            pwd_sig: Some(pwd_sig),
            device_id: None,
        })
    }

    pub async fn verify_player_token_claims_only(
        &self,
        token: &str,
    ) -> Option<(String, String, UserAccount, u64, String, Option<String>)> {
        let claims = self.verify_session_token(token.trim())?;
        let exp = claims.exp?;
        if now_ms() > exp {
            return None;
        }
        let self_id = claims.self_id?.trim().to_string();
        let username = claims.username?.trim().to_string();
        if self_id.is_empty() || username.is_empty() {
            return None;
        }
        let claim_sess_epoch = claims.session_epoch?;
        let claim_user_epoch = claims.user_token_epoch?;
        let claim_pwd_sig = claims.pwd_sig?;
        let claim_device_id = claims.device_id.filter(|d| !d.trim().is_empty());

        let cfg = self.users_cfg.read().await;
        if claim_sess_epoch != cfg.session_epoch {
            return None;
        }
        let found = cfg
            .users
            .iter()
            .find(|u| u.username.eq_ignore_ascii_case(&username))?
            .clone();
        drop(cfg);

        if claim_user_epoch != found.token_epoch {
            return None;
        }
        let expected_sig = self.player_password_sig(&found);
        if !constant_time_str_eq(&claim_pwd_sig, &expected_sig) {
            return None;
        }

        Some((
            self_id,
            found.username.clone(),
            found,
            claim_sess_epoch,
            expected_sig,
            claim_device_id,
        ))
    }

    /// Authoritatively verifies a player session token against:
    /// 1. HMAC-SHA256 signature (`ELLA_HMAC_SECRET`)
    /// 2. Expiration window (`exp`)
    /// 3. Global server `session_epoch` (invalidates all tokens on global revoke)
    /// 4. Account existence + per-user `token_epoch`
    /// 5. Current password HMAC fingerprint (`pwd_sig`) — so rolling a password immediately expires old tokens!
    /// 6. Parental controls (`world_locked`, `curfew_enabled`, and per-child `suspended`)
    /// 7. Machine digital signature lock (`device_id` must not be banned or locked by another penalized account)
    pub async fn verify_active_player_token(
        &self,
        token: &str,
    ) -> Option<(String, String, UserAccount, u64, String, Option<String>)> {
        let verified = self.verify_player_token_claims_only(token).await?;
        let cfg = self.users_cfg.read().await;
        if check_world_access_for_cfg(&cfg, &verified.2).is_err() {
            return None;
        }
        if check_machine_access_for_cfg(&cfg, verified.5.as_deref(), &verified.1).is_err() {
            return None;
        }
        drop(cfg);
        Some(verified)
    }

    pub async fn check_world_access_for_user(&self, user: &UserAccount) -> Result<(), String> {
        let cfg = self.users_cfg.read().await;
        check_world_access_for_cfg(&cfg, user)
    }

    pub async fn is_voice_allowed_for(&self, self_id: &str) -> bool {
        if !self.global_voice_enabled.load(Ordering::Relaxed) {
            return false;
        }
        self.players
            .read()
            .await
            .get(self_id)
            .map(|p| !p.server_voice_muted)
            .unwrap_or(false)
    }

    pub async fn enforce_live_world_access(&self) {
        if self.players.read().await.is_empty() {
            return;
        }
        let (global_block_reason, suspended_users) = {
            let cfg = self.users_cfg.read().await;
            let global_reason = if cfg.world_locked {
                Some(
                    "⏸️ ELLA World was paused by a parent for a break! Check back soon."
                        .to_string(),
                )
            } else if cfg.curfew_enabled
                && is_within_curfew_window(
                    local_minutes_since_midnight(),
                    &cfg.curfew_start,
                    &cfg.curfew_end,
                )
            {
                Some(format!(
                    "🌙 Bedtime curfew is active ({} – {})! Sweet dreams — see you tomorrow!",
                    cfg.curfew_start, cfg.curfew_end
                ))
            } else {
                None
            };
            let suspended: Vec<String> = cfg
                .users
                .iter()
                .filter(|u| u.suspended)
                .map(|u| u.username.clone())
                .collect();
            (global_reason, suspended)
        };

        if let Some(reason) = global_block_reason {
            let _ = self.kick_and_remove_all_sessions(&reason).await;
            return;
        }

        for uname in suspended_users {
            let is_online = self
                .players
                .read()
                .await
                .values()
                .any(|p| p.user.username.eq_ignore_ascii_case(&uname));
            if is_online {
                let _ = self
                    .kick_and_remove_user_sessions(&uname, "Your account was paused by a parent.")
                    .await;
            }
        }
    }

    pub async fn verify_active_admin_token(&self, token: &str) -> bool {
        let Some(claims) = self.verify_session_token(token.trim()) else {
            return false;
        };
        if claims.role.as_deref() != Some("admin") {
            return false;
        }
        let Some(exp) = claims.exp else {
            return false;
        };
        if now_ms() > exp {
            return false;
        }
        let (Some(claim_sess_epoch), Some(claim_pwd_sig)) = (claims.session_epoch, claims.pwd_sig)
        else {
            return false;
        };
        let (sess_epoch, admin_pw) = {
            let cfg = self.users_cfg.read().await;
            (cfg.session_epoch, cfg.admin_password.clone())
        };
        if claim_sess_epoch != sess_epoch {
            return false;
        }
        let expected_sig = self.admin_password_sig(&admin_pw);
        constant_time_str_eq(&claim_pwd_sig, &expected_sig)
    }

    pub async fn is_session_still_authorized(
        &self,
        username: &str,
        session_epoch: u64,
        user_token_epoch: u64,
        pwd_sig: &str,
        device_id: Option<&str>,
    ) -> bool {
        let cfg = self.users_cfg.read().await;
        if cfg.session_epoch != session_epoch {
            return false;
        }
        let Some(user) = cfg
            .users
            .iter()
            .find(|u| u.username.eq_ignore_ascii_case(username))
        else {
            return false;
        };
        if user.token_epoch != user_token_epoch {
            return false;
        }
        if check_world_access_for_cfg(&cfg, user).is_err() {
            return false;
        }
        if check_machine_access_for_cfg(&cfg, device_id, username).is_err() {
            return false;
        }
        let expected_sig = self.player_password_sig(user);
        constant_time_str_eq(pwd_sig, &expected_sig)
    }

    /// Resolves a client's cryptographic machine digital signature on `/login` or `/resume`,
    /// rejects banned machines or alt-account penalty evasion attempts with `403` error text
    /// (while logging forensic evidence to `world_device_evasion.jsonl`), and returns the
    /// resolved `(device_id, short_sig, seal_token)` when machine tracking applies.
    #[allow(clippy::too_many_arguments)]
    pub async fn resolve_and_authorize_machine_login(
        &self,
        username: &str,
        cookie_seal: Option<&str>,
        body_seal: Option<&str>,
        token_claim_device_id: Option<&str>,
        telemetry: Option<&ClientDeviceTelemetry>,
        user_agent: Option<&str>,
        peer_ip: Option<std::net::IpAddr>,
    ) -> Result<Option<ResolvedMachineSession>, String> {
        let secret = &self.config.hmac_secret;
        let verified_cookie = cookie_seal.and_then(|t| verify_machine_seal(t, secret));
        let verified_body = body_seal.and_then(|t| verify_machine_seal(t, secret));
        let from_cookie = verified_cookie.is_some();
        let mut verified_seal = verified_cookie.or(verified_body);

        // If resuming a token that already carried a signed `device_id`, synthesize seal lookup if needed
        if verified_seal.is_none() {
            if let Some(cid) = token_claim_device_id.filter(|s| s.starts_with("dev_")) {
                verified_seal = Some(DeviceSealClaims {
                    device_id: cid.to_string(),
                    hw_core_hash: String::new(),
                    iat: now_ms(),
                });
            }
        }

        let tel_ref = telemetry.filter(|t| t.has_hardware_signals());
        let is_headless_loopback_test = verified_seal.is_none()
            && tel_ref.is_none()
            && user_agent.map(str::trim).unwrap_or("").is_empty()
            && peer_ip.map(|ip| ip.is_loopback()).unwrap_or(true);

        if is_headless_loopback_test {
            return Ok(None);
        }

        let hw_exact = tel_ref.and_then(|t| compute_hw_exact_sig(t, user_agent, secret));
        let hw_xb = tel_ref.and_then(|t| compute_hw_xbrowser_sig(t, user_agent, peer_ip, secret));
        let net_os = compute_net_os_fallback_sig(peer_ip, user_agent, secret);

        let now = now_ms();
        let ip_str = peer_ip.map(|ip| ip.to_string()).unwrap_or_default();
        let clean_uname = username.trim().to_lowercase();

        let mut evasion_to_log: Option<DeviceEvasionEntry> = None;

        let outcome = {
            let mut cfg = self.users_cfg.write().await;
            let matched = match_existing_device(
                &cfg.devices,
                &cfg.users,
                verified_seal.as_ref(),
                hw_exact.as_deref(),
                hw_xb.as_deref(),
                net_os.as_deref(),
                from_cookie,
            );

            let default_tel = ClientDeviceTelemetry::default();
            let (label, os_fam, gpu_sum, scr_sum) =
                tel_ref.unwrap_or(&default_tel).summaries(user_agent);

            let dev_idx = if let Some(idx) = matched.index {
                idx
            } else {
                let device_id = if let Some(ref s) = verified_seal {
                    s.device_id.clone()
                } else {
                    let seed = hw_exact
                        .as_deref()
                        .or(hw_xb.as_deref())
                        .or(net_os.as_deref())
                        .map(str::to_string)
                        .unwrap_or_else(|| format!("ephem-{now}-{:08x}", rand::random::<u32>()));
                    let candidate = mint_deterministic_or_fresh_device_id(&seed, secret);
                    if cfg.devices.iter().any(|d| d.device_id == candidate) {
                        mint_deterministic_or_fresh_device_id(
                            &format!("{seed}-{now}-{:06x}", rand::random::<u32>() & 0xffffff),
                            secret,
                        )
                    } else {
                        candidate
                    }
                };
                let short_sig = short_sig_from_device_id(&device_id);
                cfg.devices.push(MachineDeviceRecord {
                    device_id,
                    short_sig,
                    label: label.clone(),
                    os_family: os_fam.clone(),
                    gpu_summary: gpu_sum.clone(),
                    screen_summary: scr_sum.clone(),
                    hw_exact_sig: hw_exact.clone().unwrap_or_default(),
                    hw_xbrowser_sig: hw_xb.clone().unwrap_or_default(),
                    net_os_sig: net_os.clone().unwrap_or_default(),
                    last_ip: ip_str.clone(),
                    associated_users: Vec::new(),
                    last_username: String::new(),
                    first_seen_ms: now,
                    last_seen_ms: now,
                    trusted_shared: false,
                    banned: false,
                });
                cfg.devices.len() - 1
            };

            if cfg.devices[dev_idx].banned {
                Err(format!(
                    "🚫 Machine #{} is locked by a parent. Ask a parent to unlock this device in the Parent Admin Panel!",
                    cfg.devices[dev_idx].short_sig
                ))
            } else {
                let enforce =
                    cfg.enforce_machine_signatures && !cfg.devices[dev_idx].trusted_shared;
                let mut blocking_hit: Option<(String, String, Vec<String>)> = None;
                if enforce {
                    for assoc_u in &cfg.devices[dev_idx].associated_users {
                        if assoc_u.eq_ignore_ascii_case(&clean_uname) {
                            continue;
                        }
                        if let Some(other_acc) = cfg
                            .users
                            .iter()
                            .find(|u| u.username.eq_ignore_ascii_case(assoc_u))
                        {
                            let pens = active_penalties_for_user(other_acc);
                            if !pens.is_empty() {
                                let disp = if other_acc.display_name.is_empty() {
                                    other_acc.username.clone()
                                } else {
                                    other_acc.display_name.clone()
                                };
                                blocking_hit = Some((other_acc.username.clone(), disp, pens));
                                break;
                            }
                        }
                    }
                }

                if let Some((blk_user, blk_disp, pens)) = blocking_hit {
                    let d = &mut cfg.devices[dev_idx];
                    d.last_seen_ms = now;
                    if !ip_str.is_empty() {
                        d.last_ip = ip_str.clone();
                    }
                    evasion_to_log = Some(DeviceEvasionEntry {
                        id: format!("ev_{now}_{:06x}", rand::random::<u32>() & 0xffffff),
                        device_id: d.device_id.clone(),
                        short_sig: d.short_sig.clone(),
                        device_label: d.label.clone(),
                        attempted_username: clean_uname.clone(),
                        blocking_username: blk_user.clone(),
                        active_penalties: pens.clone(),
                        match_method: matched.match_method.clone(),
                        peer_ip: ip_str,
                        ts: now,
                    });
                    Err(format!(
                        "⚖️ Machine Signature Lock (#{}): {} (@{}) has an active penalty on this device ({}). Sign in as @{} to complete your chores, or ask a parent for a Pardon or Shared-PC Override!",
                        d.short_sig,
                        blk_disp,
                        blk_user,
                        pens.join(", "),
                        blk_user
                    ))
                } else {
                    let d = &mut cfg.devices[dev_idx];
                    if tel_ref.is_some() {
                        d.label = label;
                        d.os_family = os_fam;
                        d.gpu_summary = gpu_sum;
                        d.screen_summary = scr_sum;
                        if let Some(ex) = hw_exact {
                            d.hw_exact_sig = ex;
                        }
                        if let Some(xb) = hw_xb.clone() {
                            d.hw_xbrowser_sig = xb;
                        }
                    }
                    if let Some(no) = net_os {
                        d.net_os_sig = no;
                    }
                    if !ip_str.is_empty() {
                        d.last_ip = ip_str;
                    }
                    if !d
                        .associated_users
                        .iter()
                        .any(|u| u.eq_ignore_ascii_case(&clean_uname))
                    {
                        d.associated_users.push(clean_uname.clone());
                    }
                    d.last_username = clean_uname;
                    d.last_seen_ms = now;

                    let core_hash = if !d.hw_xbrowser_sig.is_empty() {
                        d.hw_xbrowser_sig.chars().take(16).collect()
                    } else {
                        d.short_sig.clone()
                    };
                    let seal = sign_machine_seal(
                        &DeviceSealClaims {
                            device_id: d.device_id.clone(),
                            hw_core_hash: core_hash,
                            iat: now,
                        },
                        secret,
                    );
                    Ok(Some(ResolvedMachineSession {
                        device_id: d.device_id.clone(),
                        short_sig: d.short_sig.clone(),
                        seal_token: seal,
                    }))
                }
            }
        };

        self.persist_users_async().await;

        if let Some(entry) = evasion_to_log {
            self.cached_device_evasion_count
                .fetch_add(1, Ordering::Relaxed);
            let store = Arc::clone(&self.store);
            tokio::task::spawn_blocking(move || {
                let _ = store.append_device_evasion(&entry);
            });
        }

        outcome
    }

    /// Immediately evicts any OTHER user accounts that are concurrently logged in on the same
    /// non-exempt physical machine(s) as `penalized_username` when `penalized_username` incurs
    /// an active penalty, closing the "pre-open a friend's account in Tab 2" loophole.
    pub async fn evict_other_accounts_on_user_machines(
        &self,
        penalized_username: &str,
    ) -> Vec<String> {
        let clean_pen = penalized_username.trim().to_lowercase();
        let live_dev_ids_for_pen: Vec<String> = {
            let map = self.players.read().await;
            map.values()
                .filter(|p| p.user.username.eq_ignore_ascii_case(&clean_pen))
                .filter_map(|p| p.device_id.clone())
                .collect()
        };

        let (pen_summary, locked_devices) = {
            let cfg = self.users_cfg.read().await;
            if !cfg.enforce_machine_signatures {
                return Vec::new();
            }
            let Some(pen_user) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(&clean_pen))
            else {
                return Vec::new();
            };
            let pens = active_penalties_for_user(pen_user);
            if pens.is_empty() {
                return Vec::new();
            }
            let mut devs: Vec<MachineDeviceRecord> = cfg
                .devices
                .iter()
                .filter(|d| {
                    !d.trusted_shared
                        && (d
                            .associated_users
                            .iter()
                            .any(|u| u.eq_ignore_ascii_case(&clean_pen))
                            || live_dev_ids_for_pen.contains(&d.device_id))
                })
                .cloned()
                .collect();
            for lid in &live_dev_ids_for_pen {
                if !devs.iter().any(|d| &d.device_id == lid) {
                    devs.push(MachineDeviceRecord {
                        device_id: lid.clone(),
                        short_sig: short_sig_from_device_id(lid),
                        label: "Active Machine".into(),
                        os_family: String::new(),
                        gpu_summary: String::new(),
                        screen_summary: String::new(),
                        hw_exact_sig: String::new(),
                        hw_xbrowser_sig: String::new(),
                        net_os_sig: String::new(),
                        last_ip: String::new(),
                        associated_users: vec![clean_pen.clone()],
                        last_username: clean_pen.clone(),
                        first_seen_ms: now_ms(),
                        last_seen_ms: now_ms(),
                        trusted_shared: false,
                        banned: false,
                    });
                }
            }
            (pens, devs)
        };

        if locked_devices.is_empty() {
            return Vec::new();
        }

        let mut evicted_peer_ids = Vec::new();
        let mut evasions = Vec::new();
        let now = now_ms();

        for dev in &locked_devices {
            let reason = format!(
                "⚖️ Machine Signature Lock (#{}): @{} received an active penalty ({}) on this device. Alternate accounts on the same machine were signed out!",
                dev.short_sig,
                clean_pen,
                pen_summary.join(", ")
            );
            let _ = self
                .session_revoke_tx
                .send(SessionRevokeSignal::DeviceOtherUsers {
                    device_id: dev.device_id.clone(),
                    allowed_username_lower: clean_pen.clone(),
                    reason,
                });

            let victims: Vec<(String, String)> = {
                let map = self.players.read().await;
                map.values()
                    .filter(|p| {
                        p.device_id.as_deref() == Some(dev.device_id.as_str())
                            && !p.user.username.eq_ignore_ascii_case(&clean_pen)
                    })
                    .map(|p| (p.id.clone(), p.user.username.clone()))
                    .collect()
            };

            for (pid, victim_uname) in victims {
                evicted_peer_ids.push(pid.clone());
                evasions.push(DeviceEvasionEntry {
                    id: format!("ev_{now}_{:06x}", rand::random::<u32>() & 0xffffff),
                    device_id: dev.device_id.clone(),
                    short_sig: dev.short_sig.clone(),
                    device_label: dev.label.clone(),
                    attempted_username: victim_uname,
                    blocking_username: clean_pen.clone(),
                    active_penalties: pen_summary.clone(),
                    match_method: "live_tab_eviction".into(),
                    peer_ip: dev.last_ip.clone(),
                    ts: now,
                });
            }
        }

        for pid in &evicted_peer_ids {
            self.remove_player_immediately(pid).await;
        }

        if !evasions.is_empty() {
            self.cached_device_evasion_count
                .fetch_add(evasions.len(), Ordering::Relaxed);
            let store = Arc::clone(&self.store);
            tokio::task::spawn_blocking(move || {
                for ev in evasions {
                    let _ = store.append_device_evasion(&ev);
                }
            });
        }

        evicted_peer_ids
    }

    pub async fn admin_manage_device(
        &self,
        input: AdminManageDeviceInput,
    ) -> Result<(MachineDeviceRecord, Vec<String>), String> {
        let dev_id = input.device_id.trim();
        if dev_id.is_empty() {
            return Err("deviceId is required.".into());
        }
        let action = input.action.trim().to_lowercase();

        let (updated_rec, kick_banned) = {
            let mut cfg = self.users_cfg.write().await;
            let Some(dev) = cfg.devices.iter_mut().find(|d| d.device_id == dev_id) else {
                return Err(format!("Machine {dev_id} not found."));
            };
            match action.as_str() {
                "set_flags" | "update" => {
                    if let Some(ts) = input.trusted_shared {
                        dev.trusted_shared = ts;
                    }
                    if let Some(b) = input.banned {
                        dev.banned = b;
                    }
                }
                "unlink_users" => {
                    dev.associated_users.clear();
                }
                _ => return Err("Unknown device management action.".into()),
            }
            (dev.clone(), dev.banned)
        };

        self.persist_users_async().await;

        let mut kicked_peers = Vec::new();
        if kick_banned {
            let _ = self
                .session_revoke_tx
                .send(SessionRevokeSignal::DeviceOtherUsers {
                    device_id: updated_rec.device_id.clone(),
                    allowed_username_lower: String::new(),
                    reason: format!(
                        "🚫 Machine #{} was locked by a parent.",
                        updated_rec.short_sig
                    ),
                });
            let banned_peers: Vec<String> = {
                let map = self.players.read().await;
                map.values()
                    .filter(|p| p.device_id.as_deref() == Some(updated_rec.device_id.as_str()))
                    .map(|p| p.id.clone())
                    .collect()
            };
            for pid in banned_peers {
                self.remove_player_immediately(&pid).await;
                kicked_peers.push(pid);
            }
        } else if !updated_rec.trusted_shared {
            // Re-enforcing anti-cheat on this machine: if any linked user is penalized, evict others
            for uname in &updated_rec.associated_users {
                let evicted = self.evict_other_accounts_on_user_machines(uname).await;
                kicked_peers.extend(evicted);
            }
        }

        Ok((updated_rec, kicked_peers))
    }

    pub async fn kick_and_remove_peer(&self, self_id: &str, reason: &str) {
        let _ = self.session_revoke_tx.send(SessionRevokeSignal::PeerId {
            self_id: self_id.to_string(),
            reason: reason.to_string(),
        });
        self.remove_player_immediately(self_id).await;
    }

    pub async fn kick_and_remove_user_sessions(&self, username: &str, reason: &str) -> Vec<String> {
        let clean = username.trim().to_lowercase();
        let _ = self.session_revoke_tx.send(SessionRevokeSignal::Username {
            username_lower: clean.clone(),
            reason: reason.to_string(),
        });
        let peer_ids: Vec<String> = {
            let map = self.players.read().await;
            map.values()
                .filter(|p| p.user.username.eq_ignore_ascii_case(&clean))
                .map(|p| p.id.clone())
                .collect()
        };
        for pid in &peer_ids {
            self.remove_player_immediately(pid).await;
        }
        peer_ids
    }

    pub async fn kick_and_remove_all_sessions(&self, reason: &str) -> Vec<String> {
        let _ = self.session_revoke_tx.send(SessionRevokeSignal::All {
            reason: reason.to_string(),
        });
        let peer_ids: Vec<String> = {
            let map = self.players.read().await;
            map.keys().cloned().collect()
        };
        for pid in &peer_ids {
            self.remove_player_immediately(pid).await;
        }
        peer_ids
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
            existing.server_voice_muted = found_user.voice_muted_by_parent;
            if found_user.in_debtors_prison && existing.zone != ZONE_PRISON {
                existing.zone = ZONE_PRISON;
                existing.x = PRISON_SPAWN_POS.0;
                existing.y = PRISON_SPAWN_POS.1;
                existing.z = PRISON_SPAWN_POS.2;
            }
            existing.last_seen = now;
            existing.disconnected_at_ms = None;
            return Some(existing.clone());
        }

        let (initial_zone, spawn_x, spawn_y, spawn_z, spawn_ry) = if found_user.in_debtors_prison {
            (
                ZONE_PRISON,
                PRISON_SPAWN_POS.0,
                PRISON_SPAWN_POS.1,
                PRISON_SPAWN_POS.2,
                0.0,
            )
        } else {
            let (sx, sy, sz, sry) = Self::spawn_coords_for_user(map.len(), &found_user);
            (ZONE_PLAZA, sx, sy, sz, sry)
        };
        let player_doc = PlayerRuntimeState {
            id: self_id.to_string(),
            user: sanitized,
            avatar_hash,
            zone: initial_zone,
            x: spawn_x,
            y: spawn_y,
            z: spawn_z,
            ry: spawn_ry,
            anim: "idle".into(),
            emote: None,
            muted: found_user.voice_muted_by_parent,
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
            server_voice_muted: found_user.voice_muted_by_parent,
            device_id: None,
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
        device_id: Option<&str>,
    ) -> Option<(u32, PlayerRuntimeState)> {
        let _ = self.ensure_player_online(self_id, username).await?;
        let mut map = self.players.write().await;
        let p = map.get_mut(self_id)?;
        p.conn_epoch = p.conn_epoch.wrapping_add(1).max(1);
        p.ws_connected = true;
        p.disconnected_at_ms = None;
        p.last_seen = now_ms();
        p.last_seq = 0;
        if let Some(did) = device_id.filter(|s| !s.is_empty()) {
            p.device_id = Some(did.to_string());
        }
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
        machine: Option<&ResolvedMachineSession>,
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

        let dev_id_opt = machine.map(|m| m.device_id.as_str());
        let session_epoch = self.users_cfg.read().await.session_epoch;
        let token = self.mint_player_token(&self_id, found_user, session_epoch, dev_id_opt);

        let sanitized = found_user.sanitized();
        let avatar_hash = sanitized.avatar.short_hash();
        let now = now_ms();

        let (player_doc, is_brand_new) = {
            let mut map = self.players.write().await;
            if let Some(existing) = map.get_mut(&self_id) {
                existing.user = sanitized.clone();
                existing.avatar_hash = avatar_hash;
                existing.server_voice_muted = found_user.voice_muted_by_parent;
                if let Some(did) = dev_id_opt {
                    existing.device_id = Some(did.to_string());
                }
                if found_user.in_debtors_prison && existing.zone != ZONE_PRISON {
                    existing.zone = ZONE_PRISON;
                    existing.x = PRISON_SPAWN_POS.0;
                    existing.y = PRISON_SPAWN_POS.1;
                    existing.z = PRISON_SPAWN_POS.2;
                }
                existing.last_seen = now;
                existing.last_seq = 0;
                existing.disconnected_at_ms = None;
                (existing.clone(), false)
            } else {
                let (initial_zone, spawn_x, spawn_y, spawn_z, spawn_ry) =
                    if found_user.in_debtors_prison {
                        (
                            ZONE_PRISON,
                            PRISON_SPAWN_POS.0,
                            PRISON_SPAWN_POS.1,
                            PRISON_SPAWN_POS.2,
                            0.0,
                        )
                    } else {
                        let (sx, sy, sz, sry) = Self::spawn_coords_for_user(map.len(), found_user);
                        (ZONE_PLAZA, sx, sy, sz, sry)
                    };
                let doc = PlayerRuntimeState {
                    id: self_id.clone(),
                    user: sanitized.clone(),
                    avatar_hash,
                    zone: initial_zone,
                    x: spawn_x,
                    y: spawn_y,
                    z: spawn_z,
                    ry: spawn_ry,
                    anim: "idle".into(),
                    emote: None,
                    muted: found_user.voice_muted_by_parent,
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
                    server_voice_muted: found_user.voice_muted_by_parent,
                    device_id: dev_id_opt.map(str::to_string),
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
        let prison_snapshot = self.prison.read().await.snapshot();
        let economy_snapshot = found_user.economy_snapshot();
        let moderation_snapshot = found_user.moderation_snapshot();
        let cfg = self.users_cfg.read().await;

        let mut resp_json = serde_json::json!({
            "token": token,
            "selfId": self_id,
            "self": player_doc,
            "user": sanitized,
            "economy": economy_snapshot,
            "moderation": moderation_snapshot,
            "itemCatalog": ITEM_CATALOG,
            "dungeon": dungeon_snapshot,
            "prison": prison_snapshot,
            "players": other_players,
            "blocks": blocks_snapshot.blocks,
            "blocksVersion": blocks_snapshot.version,
            "chatHistory": chat_history,
            "runway": runway_summary,
            "stateSeq": self.current_state_seq(),
            "serverTimeMs": now,
            "parentalWorld": {
                "globalVoiceEnabled": cfg.global_voice_enabled,
                "globalChatMode": normalize_chat_mode(&cfg.global_chat_mode),
                "safeQuickChatPhrases": SAFE_QUICK_CHAT_PHRASES,
                "profanityFineBase": cfg.profanity_fine_base,
                "debtPrisonThreshold": cfg.debt_prison_threshold,
                "prisonStrikeThreshold": cfg.prison_strike_threshold,
                "enforceMachineSignatures": cfg.enforce_machine_signatures,
            },
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
        });
        if let Some(m) = machine {
            if let Some(obj) = resp_json.as_object_mut() {
                obj.insert("machineSeal".into(), serde_json::json!(m.seal_token));
                obj.insert("machineShortSig".into(), serde_json::json!(m.short_sig));
            }
        }
        resp_json
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
        let prison_snapshot = self.prison.read().await.snapshot();
        let (economy_snapshot, moderation_snapshot, parental_world) = {
            let cfg = self.users_cfg.read().await;
            let (econ, mod_snap) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
                .map(|u| (u.economy_snapshot(), u.moderation_snapshot()))
                .unwrap_or_default();
            let pw = serde_json::json!({
                "globalVoiceEnabled": cfg.global_voice_enabled,
                "globalChatMode": normalize_chat_mode(&cfg.global_chat_mode),
                "safeQuickChatPhrases": SAFE_QUICK_CHAT_PHRASES,
                "profanityFineBase": cfg.profanity_fine_base,
                "debtPrisonThreshold": cfg.debt_prison_threshold,
                "prisonStrikeThreshold": cfg.prison_strike_threshold,
            });
            (econ, mod_snap, pw)
        };

        Some(serde_json::json!({
            "type": "welcome_snapshot",
            "stateSeq": self.current_state_seq(),
            "serverTimeMs": now_ms(),
            "selfId": self_id,
            "self": me,
            "economy": economy_snapshot,
            "moderation": moderation_snapshot,
            "parentalWorld": parental_world,
            "itemCatalog": ITEM_CATALOG,
            "dungeon": dungeon_snapshot,
            "prison": prison_snapshot,
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
        let global_voice_ok = self.global_voice_enabled.load(Ordering::Relaxed);
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
                    if p.zone == ZONE_PRISON {
                        p.x = decoded.x.clamp(-13.2, 13.2);
                        p.y = decoded.y.clamp(0.0, 12.0);
                        p.z = decoded.z.clamp(-13.2, 13.2);
                    } else {
                        p.x = decoded.x;
                        p.y = decoded.y;
                        p.z = decoded.z;
                    }
                    p.ry = decoded.ry;
                    p.anim = decoded.anim;
                    p.emote = decoded.emote;
                }
                if !global_voice_ok || p.server_voice_muted {
                    p.muted = true;
                    p.speaking = false;
                } else {
                    p.muted = decoded.muted;
                    p.speaking = decoded.speaking;
                }
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
                        if p.zone == ZONE_PRISON {
                            p.x = decoded.x.clamp(-13.2, 13.2);
                            p.y = decoded.y.clamp(0.0, 12.0);
                            p.z = decoded.z.clamp(-13.2, 13.2);
                        } else {
                            p.x = decoded.x;
                            p.y = decoded.y;
                            p.z = decoded.z;
                        }
                        p.ry = decoded.ry;
                        p.anim = decoded.anim;
                        p.emote = decoded.emote;
                    }
                    if !global_voice_ok || p.server_voice_muted {
                        p.muted = true;
                        p.speaking = false;
                    } else {
                        p.muted = decoded.muted;
                        p.speaking = decoded.speaking;
                    }
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
    ) -> Result<(u64, Vec<VoxelBlock>), String> {
        self.mutate_block_ext(
            self_id,
            username,
            BlockMutationInput {
                action,
                coords,
                color,
                kind: None,
                rot: None,
                loot_item_id: None,
            },
        )
        .await
    }

    pub async fn mutate_block_ext(
        &self,
        self_id: &str,
        username: &str,
        input: BlockMutationInput<'_>,
    ) -> Result<(u64, Vec<VoxelBlock>), String> {
        let (x, y, z) = input.coords;
        if !(-45..=45).contains(&x) || !(1..=60).contains(&y) || !(-45..=45).contains(&z) {
            return Err("Invalid block coordinates".into());
        }

        let action = input.action.trim();
        let (in_prison, can_build) = {
            let cfg = self.users_cfg.read().await;
            cfg.users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
                .map(|u| (u.in_debtors_prison, u.can_build))
                .unwrap_or((false, true))
        };

        if in_prison {
            return Err("Block building and doors are disabled while in Debtor's Prison!".into());
        }

        // 1. Interactive Door / Window hinge toggle (`action == "toggle"`)
        if action == "toggle" {
            let (next_version, snapshot, blocks_file) = {
                let mut bd = self.blocks_data.write().await;
                let target_idx = bd.blocks.iter().position(|b| {
                    b.x == x
                        && b.z == z
                        && ((b.y == y && is_toggleable_block_kind(&b.kind))
                            || (b.y == y - 1 && b.kind == "door"))
                });
                let Some(idx) = target_idx else {
                    return Err("No door or window at that spot to open or close.".into());
                };
                bd.blocks[idx].open = !bd.blocks[idx].open;
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
            return Ok((next_version, snapshot));
        }

        if !can_build {
            return Err("Block building is currently locked for your account by a parent.".into());
        }

        // 2. Remove block / furniture / displayed loot trophy (`action == "remove"`)
        if action == "remove" {
            let (next_version, snapshot, blocks_file, refunded_loot) = {
                let mut bd = self.blocks_data.write().await;
                let hit_idx = bd.blocks.iter().position(|b| {
                    b.x == x
                        && b.z == z
                        && (b.y == y || (b.y == y - 1 && is_two_tall_block_kind(&b.kind)))
                });
                let mut refund: Option<(String, String)> = None;
                if let Some(idx) = hit_idx {
                    if let Some(ref item_id) = bd.blocks[idx].loot_item_id {
                        if !bd.blocks[idx].author.eq_ignore_ascii_case(username) {
                            return Err(format!(
                                "Only {} can pick up their displayed trophy!",
                                bd.blocks[idx].author
                            ));
                        }
                        refund = Some((bd.blocks[idx].author.clone(), item_id.clone()));
                    }
                    bd.blocks.remove(idx);
                }
                bd.version += 1;
                (bd.version, bd.blocks.clone(), bd.clone(), refund)
            };

            self.persist_blocks_async(blocks_file).await;

            if let Some((owner_user, loot_id)) = refunded_loot {
                {
                    let mut cfg = self.users_cfg.write().await;
                    if let Some(u) = cfg
                        .users
                        .iter_mut()
                        .find(|u| u.username.eq_ignore_ascii_case(&owner_user))
                    {
                        add_inventory_item(&mut u.inventory, &loot_id, 1);
                    }
                }
                self.persist_users_async().await;
                self.send_economy_update(
                    self_id,
                    &owner_user,
                    &format!("picked_up_decor:{loot_id}"),
                )
                .await;
            }

            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::Block {
                state_seq: seq,
                self_id: self_id.to_string(),
                blocks_version: next_version,
                blocks: snapshot.clone(),
            });
            return Ok((next_version, snapshot));
        }

        // 3. Place block / stairs / door / window / furniture / Backpack loot trophy (`action == "place"`)
        let kind = input
            .kind
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("block");
        if !is_valid_block_kind(kind) {
            return Err("Unknown building block or furniture kind.".into());
        }
        let two_tall = is_two_tall_block_kind(kind);
        if two_tall && y >= 60 {
            return Err("Too high to place a 2-block-tall door, fridge, or wardrobe here!".into());
        }
        let rot = input.rot.unwrap_or(0) % 4;
        let valid_color = input
            .color
            .filter(|c| is_valid_hex_color(c))
            .unwrap_or("#38bdf8")
            .to_string();

        let mut deducted_loot_id: Option<&'static str> = None;
        let mut prev_avatar = None;
        let mut updated_sanitized_after_deduct: Option<SanitizedUser> = None;

        if kind == "loot_decor" {
            let raw_item = input
                .loot_item_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "Select a Backpack item to place on a display stand!".to_string())?;
            let Some(def) = find_item_def(raw_item) else {
                return Err("Unknown Backpack item for home decoration.".into());
            };
            {
                let mut cfg = self.users_cfg.write().await;
                let Some(u) = cfg
                    .users
                    .iter_mut()
                    .find(|u| u.username.eq_ignore_ascii_case(username))
                else {
                    return Err("Account not found.".into());
                };
                if !remove_inventory_item(&mut u.inventory, def.id, 1) {
                    return Err(format!(
                        "You don't have any {} left in your Backpack to place!",
                        def.name
                    ));
                }
                let saved_av = u.avatar.clone();
                if reconcile_equipped_cosmetics(&mut u.avatar, &u.inventory) {
                    prev_avatar = Some(saved_av);
                    updated_sanitized_after_deduct = Some(u.sanitized());
                }
            }
            deducted_loot_id = Some(def.id);
        }

        let place_res = {
            let mut bd = self.blocks_data.write().await;
            let overlaps = |b: &VoxelBlock| -> bool {
                if b.x != x || b.z != z {
                    return false;
                }
                if b.y == y {
                    return true;
                }
                if two_tall && b.y == y + 1 {
                    return true;
                }
                if b.y == y - 1 && is_two_tall_block_kind(&b.kind) {
                    return true;
                }
                if two_tall && b.y == y + 1 && is_two_tall_block_kind(&b.kind) {
                    return true;
                }
                false
            };

            if bd
                .blocks
                .iter()
                .any(|b| overlaps(b) && b.loot_item_id.is_some())
            {
                Err("Pick up the displayed loot trophy there before building over it!".to_string())
            } else {
                bd.blocks.retain(|b| !overlaps(b));
                if bd.blocks.len() >= 5000 {
                    if let Some(evict_idx) = bd.blocks.iter().position(|b| b.loot_item_id.is_none())
                    {
                        bd.blocks.remove(evict_idx);
                    } else {
                        bd.blocks.remove(0);
                    }
                }
                bd.blocks.push(VoxelBlock {
                    x,
                    y,
                    z,
                    color: valid_color,
                    author: username.to_string(),
                    placed_at: Some(now_ms()),
                    kind: if kind.is_empty() {
                        default_block_kind()
                    } else {
                        kind.to_string()
                    },
                    rot,
                    open: false,
                    loot_item_id: deducted_loot_id.map(str::to_string),
                });
                bd.version += 1;
                Ok((bd.version, bd.blocks.clone(), bd.clone()))
            }
        };

        let (next_version, snapshot, blocks_file) = match place_res {
            Ok(tuple) => tuple,
            Err(err_msg) => {
                if let Some(lid) = deducted_loot_id {
                    let mut cfg = self.users_cfg.write().await;
                    if let Some(u) = cfg
                        .users
                        .iter_mut()
                        .find(|u| u.username.eq_ignore_ascii_case(username))
                    {
                        add_inventory_item(&mut u.inventory, lid, 1);
                        if let Some(old_av) = prev_avatar {
                            u.avatar = old_av;
                        }
                    }
                }
                return Err(err_msg);
            }
        };

        self.persist_blocks_async(blocks_file).await;

        if let Some(lid) = deducted_loot_id {
            self.persist_users_async().await;
            if let Some(sanitized) = updated_sanitized_after_deduct {
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
                let av_seq = self.next_state_seq();
                let _ = self.udp_event_tx.send(UdpWorldEvent::AvatarUpdated {
                    state_seq: av_seq,
                    self_id: self_id.to_string(),
                    avatar_hash,
                    user: sanitized,
                });
            }
            self.send_economy_update(self_id, username, &format!("placed_decor:{lid}"))
                .await;
        }

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

    pub async fn broadcast_system_chat_notice(&self, text: String) -> ChatMessage {
        let ts = now_ms();
        let entry = ChatMessage {
            id: format!("{ts}-{:06x}", rand::random::<u32>() & 0xffffff),
            sender_id: "judge_starlight".into(),
            username: "judge_starlight".into(),
            display_name: "⚖️ Judge Starlight".into(),
            role: "system".into(),
            text,
            is_system: true,
            is_trophy_win: false,
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
        entry
    }

    pub async fn post_chat(
        &self,
        self_id: &str,
        username: &str,
        raw_text: Option<&str>,
        is_trophy_win: bool,
    ) -> Result<ChatMessage, String> {
        let (
            disp_name,
            role,
            filter_enabled,
            blocked_words,
            effective_chat_mode,
            in_debtors_prison,
        ) = {
            let cfg = self.users_cfg.read().await;
            let Some(u) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account is no longer active.".into());
            };
            check_world_access_for_cfg(&cfg, u)?;
            let global_mode = normalize_chat_mode(&cfg.global_chat_mode);
            let user_mode = normalize_chat_mode(&u.chat_mode);
            let eff_mode = if global_mode == "muted" || user_mode == "muted" {
                "muted".to_string()
            } else if global_mode == "quick_only" || user_mode == "quick_only" {
                "quick_only".to_string()
            } else {
                "normal".to_string()
            };
            let s = u.sanitized();
            (
                s.display_name,
                s.role,
                cfg.chat_filter_enabled,
                cfg.blocked_words.clone(),
                eff_mode,
                u.in_debtors_prison,
            )
        };

        if is_trophy_win && in_debtors_prison {
            return Err("Trophy celebrations are disabled inside Debtor's Prison.".into());
        }

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
            if effective_chat_mode == "muted" {
                let blocked = BlockedChatEntry {
                    id: format!("{ts}-{:06x}", rand::random::<u32>() & 0xffffff),
                    sender_id: self_id.to_string(),
                    username: username.to_string(),
                    display_name: disp_name,
                    text: clean_text,
                    reason: "chat_muted".into(),
                    matched_word: None,
                    strike_number: None,
                    fine_coins: None,
                    balance_after: None,
                    sentenced_to_prison: false,
                    ts,
                };
                let store = Arc::clone(&self.store);
                tokio::task::spawn_blocking(move || {
                    let _ = store.append_blocked_chat(&blocked);
                });
                self.cached_blocked_chat_count
                    .fetch_add(1, Ordering::Relaxed);
                return Err(
                    "🔇 Text chat is currently muted by a parent. Use Avatar Emotes (1–8) to wave!"
                        .into(),
                );
            }

            if effective_chat_mode == "quick_only"
                && !SAFE_QUICK_CHAT_PHRASES.contains(&clean_text.as_str())
            {
                let blocked = BlockedChatEntry {
                    id: format!("{ts}-{:06x}", rand::random::<u32>() & 0xffffff),
                    sender_id: self_id.to_string(),
                    username: username.to_string(),
                    display_name: disp_name,
                    text: clean_text,
                    reason: "quick_chat_only".into(),
                    matched_word: None,
                    strike_number: None,
                    fine_coins: None,
                    balance_after: None,
                    sentenced_to_prison: false,
                    ts,
                };
                let store = Arc::clone(&self.store);
                tokio::task::spawn_blocking(move || {
                    let _ = store.append_blocked_chat(&blocked);
                });
                self.cached_blocked_chat_count
                    .fetch_add(1, Ordering::Relaxed);
                return Err(
                    "🛡️ Safe Quick-Chat Mode is active! Please use the quick reaction buttons below the chat box."
                        .into(),
                );
            }

            if filter_enabled {
                if let Some(matched) = find_blocked_word(&clean_text, &blocked_words) {
                    let (
                        strike_num,
                        fine_u64,
                        balance_after,
                        newly_sentenced,
                        now_in_prison,
                        sanitized_after,
                    ) = {
                        let mut cfg = self.users_cfg.write().await;
                        let base_fine = cfg.profanity_fine_base;
                        let debt_thresh = cfg.debt_prison_threshold;
                        let strike_thresh = cfg.prison_strike_threshold;
                        let Some(u) = cfg
                            .users
                            .iter_mut()
                            .find(|u| u.username.eq_ignore_ascii_case(username))
                        else {
                            return Err("Account is no longer active.".into());
                        };
                        u.violation_strikes = u.violation_strikes.saturating_add(1);
                        let strike = u.violation_strikes;
                        let fine = calculate_escalating_fine(base_fine, strike);
                        let fine_i64 = i64::try_from(fine).unwrap_or(500);
                        u.coins = u.coins.saturating_sub(fine_i64);
                        let bal = u.coins;

                        let debt_triggered = bal <= debt_thresh;
                        let strike_triggered = strike_thresh > 0 && strike >= strike_thresh;
                        let newly = !u.in_debtors_prison && (debt_triggered || strike_triggered);
                        if newly || u.in_debtors_prison {
                            u.in_debtors_prison = true;
                            u.prison_tasks_remaining = u.prison_tasks_remaining.max(3);
                            if newly {
                                u.prison_reason = Some(if debt_triggered {
                                    format!("Unpaid chat fines ({bal} 🪙 debt, Strike #{strike})")
                                } else {
                                    format!("Habitual chat violations (Strike #{strike})")
                                });
                            }
                        }
                        (strike, fine, bal, newly, u.in_debtors_prison, u.sanitized())
                    };

                    self.persist_users_async().await;
                    self.sync_runtime_user_state(username, &sanitized_after)
                        .await;
                    self.send_economy_update(self_id, username, "profanity_fine")
                        .await;
                    self.send_moderation_update(username, "profanity_fine")
                        .await;

                    if now_in_prison {
                        self.warp_user_sessions_to_zone(
                            username,
                            ZONE_PRISON,
                            PRISON_SPAWN_POS,
                            "Sentenced to Debtor's Prison!",
                        )
                        .await;
                    }
                    if now_in_prison || balance_after < 0 {
                        let _ = self.evict_other_accounts_on_user_machines(username).await;
                    }

                    let blocked = BlockedChatEntry {
                        id: format!("{ts}-{:06x}", rand::random::<u32>() & 0xffffff),
                        sender_id: self_id.to_string(),
                        username: username.to_string(),
                        display_name: disp_name.clone(),
                        text: clean_text,
                        reason: "blocked_word".into(),
                        matched_word: Some(matched),
                        strike_number: Some(strike_num),
                        fine_coins: Some(fine_u64),
                        balance_after: Some(balance_after),
                        sentenced_to_prison: newly_sentenced,
                        ts,
                    };
                    let store = Arc::clone(&self.store);
                    tokio::task::spawn_blocking(move || {
                        let _ = store.append_blocked_chat(&blocked);
                    });
                    self.cached_blocked_chat_count
                        .fetch_add(1, Ordering::Relaxed);

                    let judge_msg = if newly_sentenced {
                        format!(
                            "⚖️ {disp_name} was fined -{fine_u64} 🪙 (Strike #{strike_num}, balance: {balance_after} 🪙) and sentenced to Debtor's Prison! Complete community service to clear your debt."
                        )
                    } else if now_in_prison {
                        format!(
                            "⚖️ {disp_name} received another -{fine_u64} 🪙 fine inside Debtor's Prison (Strike #{strike_num}, balance: {balance_after} 🪙)!"
                        )
                    } else {
                        format!(
                            "⚖️ {disp_name} was fined -{fine_u64} 🪙 for unkind language (Strike #{strike_num}, balance: {balance_after} 🪙). Repeat violations lead to Debtor's Prison!"
                        )
                    };
                    let _ = self.broadcast_system_chat_notice(judge_msg).await;

                    return Err(if newly_sentenced || now_in_prison {
                        format!(
                            "⚖️ Unkind word blocked! Fined -{fine_u64} 🪙 (Strike #{strike_num}, balance: {balance_after} 🪙). Sentenced to Debtor's Prison — complete community service stations to clear your debt!"
                        )
                    } else {
                        format!(
                            "⚖️ Unkind word blocked! Fined -{fine_u64} 🪙 (Strike #{strike_num}, balance: {balance_after} 🪙). Habitual violations lead to Debtor's Prison!"
                        )
                    });
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
                    strike_number: None,
                    fine_coins: None,
                    balance_after: None,
                    sentenced_to_prison: false,
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
        let imprisoned = {
            let cfg = self.users_cfg.read().await;
            cfg.users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
                .map(|u| u.in_debtors_prison)
                .unwrap_or(false)
        };
        if imprisoned {
            return self.runway.read().await.summary();
        }
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
        let (next_ver, next_blocks, blocks_file, refunded_trophies) = {
            let mut bd = self.blocks_data.write().await;
            let mut refunds: Vec<(String, String)> = Vec::new();
            for b in &bd.blocks {
                if let Some(ref item_id) = b.loot_item_id {
                    refunds.push((b.author.clone(), item_id.clone()));
                }
            }
            bd.blocks = if clear_all {
                Vec::new()
            } else {
                default_starter_blocks()
            };
            bd.version += 1;
            (bd.version, bd.blocks.clone(), bd.clone(), refunds)
        };
        self.persist_blocks_async(blocks_file).await;

        if !refunded_trophies.is_empty() {
            let mut refunded_users = Vec::new();
            {
                let mut cfg = self.users_cfg.write().await;
                for (owner_name, item_id) in refunded_trophies {
                    if let Some(u) = cfg
                        .users
                        .iter_mut()
                        .find(|u| u.username.eq_ignore_ascii_case(&owner_name))
                    {
                        add_inventory_item(&mut u.inventory, &item_id, 1);
                        if !refunded_users
                            .iter()
                            .any(|n: &String| n.eq_ignore_ascii_case(&owner_name))
                        {
                            refunded_users.push(owner_name);
                        }
                    }
                }
            }
            self.persist_users_async().await;
            for uname in refunded_users {
                self.send_economy_update("admin", &uname, "world_reset_trophy_refund")
                    .await;
            }
        }

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

    pub async fn get_player_moderation(&self, username: &str) -> Option<PlayerModerationSnapshot> {
        let cfg = self.users_cfg.read().await;
        cfg.users
            .iter()
            .find(|u| u.username.eq_ignore_ascii_case(username))
            .map(UserAccount::moderation_snapshot)
    }

    pub async fn sync_runtime_user_state(&self, username: &str, sanitized: &SanitizedUser) {
        let avatar_hash = sanitized.avatar.short_hash();
        let voice_muted = sanitized.moderation.voice_muted_by_parent;
        let global_voice_ok = self.global_voice_enabled.load(Ordering::Relaxed);
        let mut map = self.players.write().await;
        for p in map.values_mut() {
            if p.user.username.eq_ignore_ascii_case(username) {
                p.user = sanitized.clone();
                p.avatar_hash = avatar_hash.clone();
                p.server_voice_muted = voice_muted;
                if !global_voice_ok || voice_muted {
                    p.muted = true;
                    p.speaking = false;
                }
            }
        }
    }

    pub async fn send_moderation_update(&self, username: &str, reason: &str) {
        let (Some(moderation), Some(economy)) = (
            self.get_player_moderation(username).await,
            self.get_player_economy(username).await,
        ) else {
            return;
        };
        let peer_ids: Vec<String> = {
            let map = self.players.read().await;
            map.values()
                .filter(|p| p.user.username.eq_ignore_ascii_case(username))
                .map(|p| p.id.clone())
                .collect()
        };
        let payload = serde_json::json!({
            "type": "moderation_update",
            "reason": reason,
            "moderation": moderation,
            "economy": economy,
        })
        .to_string();
        for pid in peer_ids {
            let _ = self.send_peer_targeted_json(&pid, payload.clone()).await;
        }
    }

    pub async fn broadcast_parental_world_settings(&self) {
        let (global_voice, global_chat, fine_base, debt_thresh, strike_thresh) = {
            let cfg = self.users_cfg.read().await;
            (
                cfg.global_voice_enabled,
                normalize_chat_mode(&cfg.global_chat_mode),
                cfg.profanity_fine_base,
                cfg.debt_prison_threshold,
                cfg.prison_strike_threshold,
            )
        };
        self.global_voice_enabled
            .store(global_voice, Ordering::Relaxed);

        let peer_ids: Vec<String> = {
            let mut map = self.players.write().await;
            if !global_voice {
                for p in map.values_mut() {
                    p.muted = true;
                    p.speaking = false;
                }
            }
            map.keys().cloned().collect()
        };

        let payload = serde_json::json!({
            "type": "parental_settings_update",
            "parentalWorld": {
                "globalVoiceEnabled": global_voice,
                "globalChatMode": global_chat,
                "safeQuickChatPhrases": SAFE_QUICK_CHAT_PHRASES,
                "profanityFineBase": fine_base,
                "debtPrisonThreshold": debt_thresh,
                "prisonStrikeThreshold": strike_thresh,
            }
        })
        .to_string();
        for pid in peer_ids {
            let _ = self.send_peer_targeted_json(&pid, payload.clone()).await;
        }
    }

    pub async fn warp_user_sessions_to_zone(
        &self,
        username: &str,
        target_zone: u8,
        coords: (f32, f32, f32),
        cancel_trade_reason: &str,
    ) {
        let affected_ids: Vec<String> = {
            let mut map = self.players.write().await;
            let mut ids = Vec::new();
            let now = now_ms();
            for p in map.values_mut() {
                if p.user.username.eq_ignore_ascii_case(username) {
                    p.zone = target_zone;
                    p.x = coords.0;
                    p.y = coords.1;
                    p.z = coords.2;
                    p.anim = "idle".into();
                    p.emote = None;
                    p.last_seen = now;
                    ids.push(p.id.clone());
                }
            }
            ids
        };

        if affected_ids.is_empty() {
            return;
        }

        let dungeon_snapshot = self.dungeon.read().await.snapshot();
        let prison_snapshot = self.prison.read().await.snapshot();

        for pid in affected_ids {
            self.cancel_trades_for_peer(&pid, cancel_trade_reason).await;
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerZone {
                state_seq: seq,
                id: pid.clone(),
                zone: target_zone,
                x: coords.0,
                y: coords.1,
                z: coords.2,
            });
            let response = serde_json::json!({
                "ok": true,
                "type": "zone_transitioned",
                "stateSeq": seq,
                "zone": target_zone,
                "x": coords.0,
                "y": coords.1,
                "z": coords.2,
                "plazaPortalPos": [PLAZA_PORTAL_POS.0, PLAZA_PORTAL_POS.1, PLAZA_PORTAL_POS.2],
                "returnPortalPos": [
                    DUNGEON_RETURN_PORTAL_POS.0,
                    DUNGEON_RETURN_PORTAL_POS.1,
                    DUNGEON_RETURN_PORTAL_POS.2
                ],
                "prisonReleasePortalPos": [
                    PRISON_RELEASE_PORTAL_POS.0,
                    PRISON_RELEASE_PORTAL_POS.1,
                    PRISON_RELEASE_PORTAL_POS.2
                ],
                "dungeon": dungeon_snapshot,
                "prison": prison_snapshot,
            });
            let _ = self
                .send_peer_targeted_json(&pid, response.to_string())
                .await;
        }
    }

    pub async fn tick_prison_if_occupied(&self) {
        let any_in_prison = {
            let map = self.players.read().await;
            map.values().any(|p| p.zone == ZONE_PRISON)
        };
        if !any_in_prison {
            return;
        }
        let now = now_ms();
        let maybe_snapshot = {
            let mut pr = self.prison.write().await;
            if pr.tick(now) {
                Some(pr.snapshot())
            } else {
                None
            }
        };
        if let Some(prison) = maybe_snapshot {
            let seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::PrisonSync {
                state_seq: seq,
                event: Some("tick".into()),
                actor_id: None,
                prison,
            });
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
        let current_player = self
            .ensure_player_online(self_id, username)
            .await
            .ok_or_else(|| "Player session not active.".to_string())?;

        // Enforce Debtor's Prison containment & release eligibility
        let mut released_disp_name: Option<String> = None;
        let mut released_sanitized: Option<SanitizedUser> = None;
        let mod_snapshot = {
            let mut cfg = self.users_cfg.write().await;
            if let Some(user) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            {
                if user.in_debtors_prison || current_player.zone == ZONE_PRISON {
                    if target_zone != ZONE_PLAZA {
                        return Err(
                            "Finish your Debtor's Prison sentence and return to Starlight Plaza first!"
                                .into(),
                        );
                    }
                    if user.coins < 0 || user.prison_tasks_remaining > 0 {
                        let rem_debt = (-user.coins).max(0);
                        let rem_tasks = user.prison_tasks_remaining;
                        return Err(format!(
                            "🔒 Release Gate locked! Work off your remaining debt ({rem_debt} 🪙 left) and community service ({rem_tasks} chores left) — or ask a parent for a pardon."
                        ));
                    }
                    if user.in_debtors_prison {
                        user.in_debtors_prison = false;
                        user.prison_reason = None;
                        user.prison_tasks_remaining = 0;
                        released_disp_name = Some(user.display_name.clone());
                        released_sanitized = Some(user.sanitized());
                    }
                }
                user.moderation_snapshot()
            } else {
                PlayerModerationSnapshot::default()
            }
        };

        if let Some(ref san) = released_sanitized {
            self.persist_users_async().await;
            self.sync_runtime_user_state(username, san).await;
            self.send_moderation_update(username, "served_prison_sentence")
                .await;
            if let Some(disp) = released_disp_name {
                let _ = self
                    .broadcast_system_chat_notice(format!(
                        "⚖️ {disp} completed community service, cleared all negative debt, and has been released from Debtor's Prison! Welcome back to Starlight Plaza!"
                    ))
                    .await;
            }
        }

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
        let prison_snapshot = self.prison.read().await.snapshot();
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
            "prisonReleasePortalPos": [
                PRISON_RELEASE_PORTAL_POS.0,
                PRISON_RELEASE_PORTAL_POS.1,
                PRISON_RELEASE_PORTAL_POS.2
            ],
            "dungeon": dungeon_snapshot,
            "prison": prison_snapshot,
            "moderation": mod_snapshot,
        });
        let _ = self
            .send_peer_targeted_json(self_id, response.to_string())
            .await;
        Ok(response)
    }

    pub async fn prison_perform_work(
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
        if p_zone != ZONE_PRISON {
            return Err("You are not inside the Debtor's Prison courtyard.".into());
        }

        let now = now_ms();
        {
            let cooldowns = self.prison_work_cooldowns.read().await;
            if let Some(&last_ts) = cooldowns.get(&username.to_lowercase()) {
                if now.saturating_sub(last_ts) < 1_500 {
                    return Err(
                        "Catch your breath for a second before starting the next chore!".into(),
                    );
                }
            }
        }

        let (station_label, snapshot) = {
            let mut pr = self.prison.write().await;
            let Some(idx) = pr.nodes.iter().position(|n| n.id == node_id) else {
                return Err("Community service station not found.".into());
            };
            let nx = pr.nodes[idx].x;
            let ny = pr.nodes[idx].y;
            let nz = pr.nodes[idx].z;
            let dist = ((px - nx).powi(2) + (py - ny).powi(2) + (pz - nz).powi(2)).sqrt();
            if dist > 6.2 {
                return Err("Walk a little closer to work at that station!".into());
            }
            if !pr.nodes[idx].available {
                return Err(
                    "That station was just tidied! Walk over to another glowing station.".into(),
                );
            }
            let label = format!("{} {}", pr.nodes[idx].icon, pr.nodes[idx].name);
            pr.nodes[idx].available = false;
            pr.nodes[idx].ready_at_ms = now + 6_000;
            (label, pr.snapshot())
        };

        self.prison_work_cooldowns
            .write()
            .await
            .insert(username.to_lowercase(), now);

        let (earned_toward_debt, ready_for_release, sanitized, econ, mod_snap) = {
            let mut cfg = self.users_cfg.write().await;
            let Some(user) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account not found.".into());
            };
            let before = user.coins;
            if user.coins < 0 {
                user.coins = (user.coins + PRISON_WORK_REWARD_COINS).min(0);
            }
            let earned = (user.coins - before).max(0);
            user.prison_tasks_remaining = user.prison_tasks_remaining.saturating_sub(1);
            let ready = user.coins >= 0 && user.prison_tasks_remaining == 0;
            (
                earned,
                ready,
                user.sanitized(),
                user.economy_snapshot(),
                user.moderation_snapshot(),
            )
        };

        self.persist_users_async().await;
        self.sync_runtime_user_state(username, &sanitized).await;

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::PrisonSync {
            state_seq: seq,
            event: Some(format!("work:{node_id}")),
            actor_id: Some(self_id.to_string()),
            prison: snapshot.clone(),
        });

        self.send_economy_update(self_id, username, "prison_work")
            .await;
        self.send_moderation_update(username, "prison_work").await;

        Ok(serde_json::json!({
            "ok": true,
            "stationLabel": station_label,
            "earnedTowardDebt": earned_toward_debt,
            "readyForRelease": ready_for_release,
            "economy": econ,
            "moderation": mod_snap,
            "prison": snapshot,
        }))
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
        let (p_zone, px, py, pz, username) = {
            let map = self.players.read().await;
            let Some(p) = map.get(self_id) else {
                return Err("Player session not found.".into());
            };
            (p.zone, p.x, p.y, p.z, p.user.username.clone())
        };
        if p_zone != ZONE_DUNGEON {
            return Err("Enter the Crystal Cavern Portal first!".into());
        }

        let is_raygun = mode == "raygun";
        if is_raygun {
            let owns_raygun = {
                let cfg = self.users_cfg.read().await;
                cfg.users
                    .iter()
                    .find(|u| u.username.eq_ignore_ascii_case(&username))
                    .map(|u| player_owns_item(&u.inventory, "legendary_ray_gun"))
                    .unwrap_or(false)
            };
            if !owns_raygun {
                return Err(
                    "You need the Legendary Starlight Plasma Ray Gun 🔫 to fire plasma beams!"
                        .into(),
                );
            }
        }

        let now = now_ms();
        let (dmg, defeated, spawned_drop, snapshot, mob_center) = {
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
            let max_reach = if is_raygun { 14.5 } else { 6.2 };
            if dist > max_reach {
                return Err(if is_raygun {
                    "Step within 14m so your Plasma Ray Gun can lock onto that creature!".into()
                } else {
                    "Move closer to stomp or Sparkle-Blast that creature!".into()
                });
            }

            let dmg: u8 = if is_raygun {
                3
            } else if mode == "stomp" {
                2
            } else {
                1
            };
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
            (dmg, defeated, maybe_drop, snap, [mx, my + 0.55, mz])
        };

        if is_raygun {
            let beam_seq = self.next_state_seq();
            let _ = self.udp_event_tx.send(UdpWorldEvent::RayGunBeam {
                state_seq: beam_seq,
                actor_id: self_id.to_string(),
                zone: ZONE_DUNGEON,
                from: [px, py + 1.05, pz],
                to: mob_center,
                hit_mob_id: Some(mob_id.to_string()),
            });
        }

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
            "mode": mode,
            "damage": dmg,
            "defeated": defeated,
            "drop": spawned_drop,
            "dungeon": snapshot,
        }))
    }

    pub async fn fire_plaza_raygun(
        &self,
        self_id: &str,
        username: &str,
        target: [f32; 3],
    ) -> Result<serde_json::Value, String> {
        let (p_zone, px, py, pz) = {
            let map = self.players.read().await;
            let Some(p) = map.get(self_id) else {
                return Err("Player session not found.".into());
            };
            (p.zone, p.x, p.y, p.z)
        };
        if p_zone != ZONE_PLAZA {
            return Err(
                "Plaza Laser Light-Show beams can only be fired in Starlight Plaza!".into(),
            );
        }

        {
            let cfg = self.users_cfg.read().await;
            let Some(u) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            else {
                return Err("Account not found.".into());
            };
            if u.in_debtors_prison {
                return Err("Ray Gun beams are disabled while in Debtor's Prison!".into());
            }
            if !player_owns_item(&u.inventory, "legendary_ray_gun") {
                return Err(
                    "Unlock the Legendary Starlight Plasma Ray Gun 🔫 in Crystal Cavern first!"
                        .into(),
                );
            }
        }

        let now = now_ms();
        {
            let mut cds = self.raygun_cooldowns.write().await;
            let key = username.to_lowercase();
            if let Some(&last) = cds.get(&key) {
                if now.saturating_sub(last) < 200 {
                    return Err("Ray Gun plasma capacitor recharging...".into());
                }
            }
            cds.insert(key, now);
        }

        let from = [px, py + 1.05, pz];
        let mut dx = if target[0].is_finite() {
            target[0] - from[0]
        } else {
            0.0
        };
        let mut dy = if target[1].is_finite() {
            target[1] - from[1]
        } else {
            0.0
        };
        let mut dz = if target[2].is_finite() {
            target[2] - from[2]
        } else {
            1.0
        };
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        if len < 0.05 {
            dz = 8.0;
        } else if len > 14.5 {
            let scale = 14.5 / len;
            dx *= scale;
            dy *= scale;
            dz *= scale;
        }
        let to = [
            ((from[0] + dx) * 100.0).round() / 100.0,
            ((from[1] + dy).clamp(0.15, 62.0) * 100.0).round() / 100.0,
            ((from[2] + dz) * 100.0).round() / 100.0,
        ];

        let seq = self.next_state_seq();
        let _ = self.udp_event_tx.send(UdpWorldEvent::RayGunBeam {
            state_seq: seq,
            actor_id: self_id.to_string(),
            zone: ZONE_PLAZA,
            from,
            to,
            hit_mob_id: None,
        });

        Ok(serde_json::json!({
            "ok": true,
            "stateSeq": seq,
            "from": from,
            "to": to,
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
            user.coins = user
                .coins
                .saturating_add(i64::try_from(picked.coins).unwrap_or(0));
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

        if p_zone == ZONE_PRISON {
            return Err("Inventory items cannot be used while inside Debtor's Prison!".into());
        }

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
            if user.in_debtors_prison {
                return Err("Inventory items cannot be used while inside Debtor's Prison!".into());
            }
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
            if me.zone == ZONE_PRISON {
                return Err("Trading is locked while serving a Debtor's Prison sentence!".into());
            }
            if them.zone == ZONE_PRISON {
                return Err(format!(
                    "{} is currently in Debtor's Prison and cannot trade.",
                    them.user.display_name
                ));
            }
            (
                me.user.display_name.clone(),
                them.user.username.clone(),
                them.user.display_name.clone(),
            )
        };

        {
            let cfg = self.users_cfg.read().await;
            if let Some(me_u) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))
            {
                if me_u.in_debtors_prison {
                    return Err(
                        "Trading is locked while serving a Debtor's Prison sentence!".into(),
                    );
                }
                if !me_u.can_trade {
                    return Err("Trading is currently disabled on your account by a parent.".into());
                }
            }
            if let Some(them_u) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(&part_user))
            {
                if them_u.in_debtors_prison {
                    return Err(format!(
                        "{part_disp} is currently in Debtor's Prison and cannot trade."
                    ));
                }
                if !them_u.can_trade {
                    return Err(format!(
                        "Trading is currently disabled by a parent for {part_disp}."
                    ));
                }
            }
        }

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

        let (init_uname_check, part_uname_check) = {
            let trades = self.trades.read().await;
            let Some(s) = trades.get(trade_id) else {
                return Err("Trade invite expired or was already closed.".into());
            };
            (s.initiator_username.clone(), s.partner_username.clone())
        };
        {
            let cfg = self.users_cfg.read().await;
            for uname_chk in [&init_uname_check, &part_uname_check] {
                if let Some(u) = cfg
                    .users
                    .iter()
                    .find(|u| u.username.eq_ignore_ascii_case(uname_chk))
                {
                    if u.in_debtors_prison {
                        return Err(
                            "Trading is locked while serving a Debtor's Prison sentence!".into(),
                        );
                    }
                    if !u.can_trade {
                        return Err(
                            "Trading is disabled by a parent for one of the players.".into()
                        );
                    }
                }
            }
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
                    if user_a.in_debtors_prison || user_b.in_debtors_prison {
                        Err(
                            "Trade cancelled: players in Debtor's Prison cannot complete trades."
                                .into(),
                        )
                    } else if !user_a.can_trade || !user_b.can_trade {
                        Err("Trade cancelled: trading is disabled by a parent.".into())
                    } else {
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

    pub async fn admin_moderate_user(
        &self,
        input: AdminModerateUserInput,
    ) -> Result<serde_json::Value, String> {
        let uname = input.username.trim().to_lowercase();
        if uname.is_empty() {
            return Err("Username is required.".into());
        }
        let action = input.action.trim().to_lowercase();

        let mut warp_to_prison = false;
        let mut warp_to_plaza = false;
        let mut cancel_trades = false;
        let mut system_notice: Option<String> = None;

        let (sanitized, econ, mod_snap, now_suspended) = {
            let mut cfg = self.users_cfg.write().await;
            let debt_thresh = cfg.debt_prison_threshold;
            let Some(user) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(&uname))
            else {
                return Err(format!("Account @{uname} not found."));
            };

            match action.as_str() {
                "update_controls" => {
                    if let Some(vm) = input.voice_muted_by_parent {
                        user.voice_muted_by_parent = vm;
                    }
                    if let Some(ref cm) = input.chat_mode {
                        user.chat_mode = normalize_chat_mode(cm);
                    }
                    if let Some(cb) = input.can_build {
                        user.can_build = cb;
                    }
                    if let Some(ct) = input.can_trade {
                        user.can_trade = ct;
                        if !ct {
                            cancel_trades = true;
                        }
                    }
                    if let Some(susp) = input.suspended {
                        user.suspended = susp;
                    }
                }
                "sentence_prison" => {
                    if let Some(delta) = input.coin_delta {
                        user.coins = user
                            .coins
                            .saturating_add(delta)
                            .clamp(-1_000_000, 1_000_000);
                    }
                    let tasks = input.prison_tasks.unwrap_or(4).clamp(1, 25);
                    let reason_text: String = input
                        .reason
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .unwrap_or("Sentenced to Debtor's Prison by Parent Judge.")
                        .chars()
                        .take(140)
                        .collect();
                    user.in_debtors_prison = true;
                    user.prison_tasks_remaining = tasks;
                    user.prison_reason = Some(reason_text.clone());
                    warp_to_prison = true;
                    cancel_trades = true;
                    system_notice = Some(format!(
                        "⚖️ {} was sentenced to Debtor's Prison ({tasks} community-service chores, balance: {} 🪙): {reason_text}",
                        user.display_name, user.coins
                    ));
                }
                "pardon_prison" => {
                    let was_imprisoned = user.in_debtors_prison;
                    let had_debt = user.coins < 0;
                    user.in_debtors_prison = false;
                    user.prison_reason = None;
                    user.prison_tasks_remaining = 0;
                    if user.coins < 0 {
                        user.coins = 0;
                    }
                    warp_to_plaza = was_imprisoned;
                    if was_imprisoned || had_debt {
                        system_notice = Some(format!(
                            "⚖️ {} received a Parent Pardon! Negative debt cleared and welcomed back to Starlight Plaza.",
                            user.display_name
                        ));
                    }
                }
                "adjust_coins" => {
                    let delta = input.coin_delta.unwrap_or(0);
                    if delta == 0 {
                        return Err("Specify a non-zero coin adjustment.".into());
                    }
                    user.coins = user
                        .coins
                        .saturating_add(delta)
                        .clamp(-1_000_000, 1_000_000);
                    if delta < 0 && user.coins <= debt_thresh && !user.in_debtors_prison {
                        user.in_debtors_prison = true;
                        user.prison_tasks_remaining = user.prison_tasks_remaining.max(3);
                        user.prison_reason = Some(format!(
                            "Account balance dropped to {} 🪙 (below {} 🪙 debt threshold).",
                            user.coins, debt_thresh
                        ));
                        warp_to_prison = true;
                        cancel_trades = true;
                        system_notice = Some(format!(
                            "⚖️ {}'s balance reached {} 🪙 and was sentenced to Debtor's Prison!",
                            user.display_name, user.coins
                        ));
                    }
                }
                "reset_strikes" => {
                    user.violation_strikes = 0;
                }
                _ => return Err("Unknown moderation action.".into()),
            }

            (
                user.sanitized(),
                user.economy_snapshot(),
                user.moderation_snapshot(),
                user.suspended,
            )
        };

        self.persist_users_async().await;
        self.sync_runtime_user_state(&uname, &sanitized).await;

        if cancel_trades {
            let peer_ids: Vec<String> = {
                let map = self.players.read().await;
                map.values()
                    .filter(|p| p.user.username.eq_ignore_ascii_case(&uname))
                    .map(|p| p.id.clone())
                    .collect()
            };
            for pid in peer_ids {
                self.cancel_trades_for_peer(&pid, "Trading restricted by parental controls.")
                    .await;
            }
        }

        if warp_to_prison {
            self.warp_user_sessions_to_zone(
                &uname,
                ZONE_PRISON,
                PRISON_SPAWN_POS,
                "Sentenced to Debtor's Prison!",
            )
            .await;
        } else if warp_to_plaza {
            let any_in_prison = {
                let map = self.players.read().await;
                map.values()
                    .any(|p| p.user.username.eq_ignore_ascii_case(&uname) && p.zone == ZONE_PRISON)
            };
            if any_in_prison {
                self.warp_user_sessions_to_zone(
                    &uname,
                    ZONE_PLAZA,
                    PLAZA_RETURN_SPAWN_POS,
                    "Pardoned from Debtor's Prison.",
                )
                .await;
            }
        }

        let online_peer = {
            let map = self.players.read().await;
            map.values()
                .find(|p| p.user.username.eq_ignore_ascii_case(&uname))
                .map(|p| p.id.clone())
                .unwrap_or_default()
        };
        if !online_peer.is_empty() {
            self.send_economy_update(&online_peer, &uname, &format!("admin:{action}"))
                .await;
        }
        self.send_moderation_update(&uname, &format!("admin:{action}"))
            .await;

        if let Some(notice) = system_notice {
            let _ = self.broadcast_system_chat_notice(notice).await;
        }

        let evicted_alts = self.evict_other_accounts_on_user_machines(&uname).await;

        Ok(serde_json::json!({
            "ok": true,
            "action": action,
            "username": uname,
            "suspended": now_suspended,
            "evictedAltSessions": evicted_alts.len(),
            "evictedAltPeers": evicted_alts,
            "economy": econ,
            "moderation": mod_snap,
        }))
    }
}

pub fn check_machine_access_for_cfg(
    cfg: &UsersConfigFile,
    device_id: Option<&str>,
    username: &str,
) -> Result<(), String> {
    let Some(did) = device_id.filter(|s| !s.trim().is_empty()) else {
        return Ok(());
    };
    let Some(dev) = cfg.devices.iter().find(|d| d.device_id == did) else {
        return Ok(());
    };
    if dev.banned {
        return Err(format!(
            "🚫 Machine #{} is locked by a parent.",
            dev.short_sig
        ));
    }
    if !cfg.enforce_machine_signatures || dev.trusted_shared {
        return Ok(());
    }
    let clean_u = username.trim().to_lowercase();
    for assoc_u in &dev.associated_users {
        if assoc_u.eq_ignore_ascii_case(&clean_u) {
            continue;
        }
        if let Some(other_acc) = cfg
            .users
            .iter()
            .find(|u| u.username.eq_ignore_ascii_case(assoc_u))
        {
            let pens = active_penalties_for_user(other_acc);
            if !pens.is_empty() {
                return Err(format!(
                    "⚖️ Machine Signature Lock (#{}): @{} has an active penalty on this device ({}).",
                    dev.short_sig,
                    other_acc.username,
                    pens.join(", ")
                ));
            }
        }
    }
    Ok(())
}

pub fn check_world_access_for_cfg(cfg: &UsersConfigFile, user: &UserAccount) -> Result<(), String> {
    if cfg.world_locked {
        return Err(
            "⏸️ ELLA World is currently paused by a parent! Ask a parent to unlock the world."
                .into(),
        );
    }
    if cfg.curfew_enabled
        && is_within_curfew_window(
            local_minutes_since_midnight(),
            &cfg.curfew_start,
            &cfg.curfew_end,
        )
    {
        return Err(format!(
            "🌙 Bedtime curfew is active ({} – {})! Sweet dreams — see you tomorrow!",
            cfg.curfew_start, cfg.curfew_end
        ));
    }
    if user.suspended {
        return Err(
            "🚫 Your account is currently paused by a parent. Ask a parent to re-enable access!"
                .into(),
        );
    }
    Ok(())
}

pub fn parse_hhmm_minutes(s: &str) -> Option<u16> {
    let trimmed = s.trim();
    let (h_str, m_str) = trimmed.split_once(':')?;
    let h: u16 = h_str.trim().parse().ok()?;
    let m: u16 = m_str.trim().parse().ok()?;
    if h >= 24 || m >= 60 {
        return None;
    }
    Some(h * 60 + m)
}

pub fn is_within_curfew_window(now_mins: u16, start_hhmm: &str, end_hhmm: &str) -> bool {
    let (Some(start), Some(end)) = (parse_hhmm_minutes(start_hhmm), parse_hhmm_minutes(end_hhmm))
    else {
        return false;
    };
    if start == end {
        return false;
    }
    if start < end {
        now_mins >= start && now_mins < end
    } else {
        // Overnight window (e.g. 21:00 -> 07:00)
        now_mins >= start || now_mins < end
    }
}

fn local_utc_offset_seconds(unix_secs: i64) -> i64 {
    let Ok(bytes) = std::fs::read("/etc/localtime") else {
        return 0;
    };
    if bytes.len() < 44 || &bytes[0..4] != b"TZif" {
        return 0;
    }
    let timecnt = u32::from_be_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]) as usize;
    let typecnt = u32::from_be_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]) as usize;
    if typecnt == 0 {
        return 0;
    }
    let trans_start = 44usize;
    let types_start = trans_start.saturating_add(timecnt.saturating_mul(4));
    let ttinfo_start = types_start.saturating_add(timecnt);
    let ttinfo_end = ttinfo_start.saturating_add(typecnt.saturating_mul(6));
    if bytes.len() < ttinfo_end {
        return 0;
    }

    let mut chosen_type_idx: usize = 0;
    let clamped_ts = unix_secs.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    for i in 0..timecnt {
        let off = trans_start + i * 4;
        let t = i32::from_be_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]);
        if t <= clamped_ts {
            chosen_type_idx = bytes[types_start + i] as usize;
        } else {
            break;
        }
    }
    if chosen_type_idx >= typecnt {
        chosen_type_idx = 0;
    }
    let rec = ttinfo_start + chosen_type_idx * 6;
    i32::from_be_bytes([bytes[rec], bytes[rec + 1], bytes[rec + 2], bytes[rec + 3]]) as i64
}

pub fn local_minutes_since_midnight() -> u16 {
    let unix_secs = (now_ms() / 1_000) as i64;
    let local_secs = unix_secs.saturating_add(local_utc_offset_seconds(unix_secs));
    let secs_of_day = local_secs.rem_euclid(86_400);
    (secs_of_day / 60) as u16
}

fn normalize_leet_char(c: char) -> Option<char> {
    let lc = c.to_ascii_lowercase();
    match lc {
        'a'..='z' => Some(lc),
        '0' => Some('o'),
        '1' | '!' | '|' => Some('i'),
        '3' => Some('e'),
        '4' | '@' => Some('a'),
        '5' | '$' => Some('s'),
        '7' => Some('t'),
        _ => None,
    }
}

fn has_non_numeric_signal(raw: &str) -> bool {
    raw.chars()
        .any(|c| c.is_ascii_alphabetic() || matches!(c, '@' | '$' | '!'))
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

fn matches_single_blocked_token(
    alnum_token: &str,
    collapsed: &str,
    single_blocked: &[(String, String)],
    safe_substring_roots: &[&str],
) -> Option<String> {
    for (bw, collapsed_bw) in single_blocked {
        if alnum_token == bw || collapsed == collapsed_bw {
            return Some(bw.clone());
        }
        if bw.len() >= 4
            && safe_substring_roots.iter().any(|root| bw.contains(root))
            && (alnum_token.contains(bw.as_str()) || collapsed.contains(collapsed_bw.as_str()))
        {
            return Some(bw.clone());
        }
    }
    None
}

pub fn find_blocked_word(text: &str, blocked_words: &[String]) -> Option<String> {
    if blocked_words.is_empty() {
        return None;
    }

    const SAFE_SUBSTRING_ROOTS: &[&str] = &[
        "fuck", "shit", "bitch", "asshole", "nigger", "nigga", "faggot", "cunt", "whore", "slut",
    ];

    let mut single_blocked: Vec<(String, String)> = Vec::new();
    let mut multi_word_blocked: Vec<(String, Vec<(String, String)>)> = Vec::new();

    for raw_bw in blocked_words {
        let trimmed = raw_bw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<(String, String)> = trimmed
            .split_whitespace()
            .map(|p| {
                let norm: String = p.chars().filter_map(normalize_leet_char).collect();
                let col = collapse_repeats(&norm);
                (norm, col)
            })
            .filter(|(n, _)| !n.is_empty())
            .collect();
        if parts.is_empty() {
            continue;
        }
        let joined: String = parts.iter().map(|(n, _)| n.as_str()).collect();
        let joined_col = collapse_repeats(&joined);
        single_blocked.push((joined.clone(), joined_col));
        if parts.len() >= 2 {
            let label = parts
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            multi_word_blocked.push((label, parts));
        }
    }

    let mut msg_tokens: Vec<(String, String, bool)> = Vec::new();

    for raw_token in text.split_whitespace() {
        if !has_non_numeric_signal(raw_token) {
            continue;
        }
        let alnum_token: String = raw_token.chars().filter_map(normalize_leet_char).collect();
        if alnum_token.is_empty() {
            continue;
        }
        let collapsed = collapse_repeats(&alnum_token);

        if let Some(hit) = matches_single_blocked_token(
            &alnum_token,
            &collapsed,
            &single_blocked,
            SAFE_SUBSTRING_ROOTS,
        ) {
            return Some(hit);
        }

        // Also check strict alphanumeric sub-segments split on non-alnum (e.g. "hi,bitch!")
        for sub in raw_token.split(|c: char| !c.is_ascii_alphanumeric()) {
            if !has_non_numeric_signal(sub) {
                continue;
            }
            let sub_norm: String = sub.chars().filter_map(normalize_leet_char).collect();
            if sub_norm.is_empty() {
                continue;
            }
            let sub_col = collapse_repeats(&sub_norm);
            for (bw, collapsed_bw) in &single_blocked {
                if sub_norm == *bw || sub_col == *collapsed_bw {
                    return Some(bw.clone());
                }
            }
        }

        let is_single_letter_frag = collapsed.chars().count() == 1;
        msg_tokens.push((alnum_token, collapsed, is_single_letter_frag));
    }

    // Detect multi-word blocked phrases (e.g. "kill yourself", "k1ll y0urself")
    for (label, phrase_parts) in &multi_word_blocked {
        let plen = phrase_parts.len();
        if msg_tokens.len() >= plen {
            for win in msg_tokens.windows(plen) {
                let all_match = win
                    .iter()
                    .zip(phrase_parts.iter())
                    .all(|((tok, col, _), (bw_tok, bw_col))| tok == bw_tok || col == bw_col);
                if all_match {
                    return Some(label.clone());
                }
            }
        }
    }

    // Detect spaced letter-by-letter evasion runs (e.g. "f u c k", "s h i t", "f uuu c k")
    let mut idx = 0usize;
    while idx < msg_tokens.len() {
        if !msg_tokens[idx].2 {
            idx += 1;
            continue;
        }
        let start = idx;
        while idx < msg_tokens.len() && msg_tokens[idx].2 {
            idx += 1;
        }
        let run_len = idx - start;
        if run_len >= 2 {
            // Check every contiguous sub-slice of length >= 3 (or full run) so "u f u c k" catches "fuck"
            for sub_start in start..idx {
                for sub_end in (sub_start + 2)..=idx {
                    let joined_raw: String = msg_tokens[sub_start..sub_end]
                        .iter()
                        .map(|(t, _, _)| t.as_str())
                        .collect();
                    let joined_col = collapse_repeats(&joined_raw);
                    if joined_col.len() < 3 {
                        continue;
                    }
                    for (bw, collapsed_bw) in &single_blocked {
                        if joined_raw == *bw || joined_col == *collapsed_bw {
                            return Some(bw.clone());
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::default_blocked_words;

    #[test]
    fn profanity_filter_catches_spaced_leet_and_phrases_without_scunthorpe_false_positives() {
        let words = default_blocked_words();

        // Spaced single-letter & repeated evasion
        assert!(find_blocked_word("what the f u c k", &words).is_some());
        assert!(find_blocked_word("you s h i t head", &words).is_some());
        assert!(find_blocked_word("b i t c h", &words).is_some());
        assert!(find_blocked_word("f uuu c k", &words).is_some());

        // Punctuation & leetspeak evasion
        assert!(find_blocked_word("s.h.i.t", &words).is_some());
        assert!(find_blocked_word("f_u_c_k", &words).is_some());
        assert!(find_blocked_word("sh1t!", &words).is_some());
        assert!(find_blocked_word("b!tch", &words).is_some());
        assert!(find_blocked_word("@sshole", &words).is_some());

        // Multi-word phrases
        assert!(find_blocked_word("please kill yourself now", &words).is_some());
        assert!(find_blocked_word("k1ll y0urself", &words).is_some());

        // Scunthorpe & numeric false-positive safety
        assert_eq!(
            find_blocked_word("Meet me in class after we pass the grass!", &words),
            None
        );
        assert_eq!(
            find_blocked_word("Hello friends! Look at that tall skyscraper!", &words),
            None
        );
        assert_eq!(
            find_blocked_word("I farmed 455 coins from scrap crystals!", &words),
            None
        );
    }

    #[test]
    fn curfew_window_handles_overnight_and_daytime_ranges() {
        // Overnight 21:00 (1260) -> 07:00 (420)
        assert!(is_within_curfew_window(21 * 60, "21:00", "07:00"));
        assert!(is_within_curfew_window(23 * 60 + 30, "21:00", "07:00"));
        assert!(is_within_curfew_window(6 * 60 + 59, "21:00", "07:00"));
        assert!(!is_within_curfew_window(7 * 60, "21:00", "07:00"));
        assert!(!is_within_curfew_window(15 * 60, "21:00", "07:00"));

        // Daytime 13:00 -> 15:00
        assert!(!is_within_curfew_window(12 * 60 + 59, "13:00", "15:00"));
        assert!(is_within_curfew_window(13 * 60, "13:00", "15:00"));
        assert!(is_within_curfew_window(14 * 60 + 30, "13:00", "15:00"));
        assert!(!is_within_curfew_window(15 * 60, "13:00", "15:00"));
    }
}

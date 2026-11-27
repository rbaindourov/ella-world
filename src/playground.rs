//! Multi-Playground Registry, Global Kid Identity & Two-Tier Invite-Only Governance
//!
//! Multiplexes Ella's canonical `"ella"` playground (`data/`) alongside isolated Family Playgrounds
//! (`data/playgrounds/<slug>/`) over the single `:4444` (HTTPS/WSS/WebRTC) and `:44445/udp` (`EWLD`)
//! ports while synchronizing 3D Avatar appearances across playgrounds and strictly isolating Home Plots,
//! Coins, Inventories, Blocks, Curfews, Chat Logs, and Per-Playground Parent Admin Passwords.

use crate::auth::{
    constant_time_str_eq, password_token_sig, sign_token, verify_token, verify_world_token,
    TokenClaims, DEFAULT_WORLD_SLUG,
};
use crate::config::ServerConfig;
use crate::economy::reconcile_equipped_cosmetics;
use crate::models::{
    default_blocked_words, default_chat_mode, default_curfew_end, default_curfew_start,
    default_debt_prison_threshold, default_ice_servers, default_prison_strike_threshold,
    default_profanity_fine_base, default_session_epoch, normalize_chat_mode, now_ms,
    AvatarAppearance, UserAccount, UsersConfigFile,
};
use crate::persistence::write_atomic_json;
use crate::world::{UdpWorldEvent, WorldEngine};
use anyhow::Context;
use axum::http::StatusCode;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

const INVITE_ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const RESERVED_WORLD_SLUGS: &[&str] = &[
    "api",
    "admin",
    "superadmin",
    "playgrounds",
    "download",
    "health",
    "ella-world",
    "invites",
    "ws",
    "rtc",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalUserIdentity {
    pub username: String,
    pub display_name: String,
    pub password: String,
    #[serde(default)]
    pub avatar: AvatarAppearance,
    #[serde(default)]
    pub member_worlds: BTreeSet<String>,
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invited_by_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InviteKind {
    FriendPlaydate {
        #[serde(rename = "worldSlug")]
        world_slug: String,
    },
    FamilyFounder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InviteStatus {
    PendingParentApproval,
    Active,
    Revoked,
    Exhausted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteTicket {
    pub code: String,
    pub kind: InviteKind,
    pub status: InviteStatus,
    pub created_by: String,
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_note: Option<String>,
    pub max_uses: u32,
    pub uses: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    #[serde(default = "default_chat_mode")]
    pub preset_chat_mode: String,
    #[serde(default)]
    pub preset_voice_muted: bool,
    #[serde(default = "default_true")]
    pub preset_can_build: bool,
    #[serde(default = "default_true")]
    pub preset_can_trade: bool,
    #[serde(default = "default_starter_coins")]
    pub starter_coins: i64,
    #[serde(default)]
    pub redeemed_by: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_starter_coins() -> i64 {
    100
}

impl InviteTicket {
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at_ms.is_some_and(|exp| now > exp)
    }

    pub fn effective_status(&self, now: u64) -> &'static str {
        match self.status {
            InviteStatus::PendingParentApproval => "pending_parent_approval",
            InviteStatus::Revoked => "revoked",
            InviteStatus::Exhausted => "exhausted",
            InviteStatus::Active => {
                if self.uses >= self.max_uses {
                    "exhausted"
                } else if self.is_expired(now) {
                    "expired"
                } else {
                    "active"
                }
            }
        }
    }

    pub fn is_redeemable(&self, now: u64) -> bool {
        self.status == InviteStatus::Active && self.uses < self.max_uses && !self.is_expired(now)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaygroundMetaRecord {
    pub slug: String,
    pub world_name: String,
    pub host_username: String,
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_via_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformRegistryFile {
    #[serde(default = "default_registry_version")]
    pub version: u32,
    #[serde(default = "default_true")]
    pub invite_only_registration: bool,
    #[serde(default)]
    pub playgrounds: BTreeMap<String, PlaygroundMetaRecord>,
    #[serde(default)]
    pub users: BTreeMap<String, GlobalUserIdentity>,
    #[serde(default)]
    pub invites: BTreeMap<String, InviteTicket>,
}

fn default_registry_version() -> u32 {
    1
}

impl Default for PlatformRegistryFile {
    fn default() -> Self {
        Self {
            version: 1,
            invite_only_registration: true,
            playgrounds: BTreeMap::new(),
            users: BTreeMap::new(),
            invites: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminInviteActionInput {
    pub action: String, // "create" | "approve" | "reject" | "revoke"
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub label_note: Option<String>,
    #[serde(default)]
    pub max_uses: Option<u32>,
    #[serde(default)]
    pub expires_in_hours: Option<u64>,
    #[serde(default)]
    pub preset_chat_mode: Option<String>,
    #[serde(default)]
    pub preset_voice_muted: Option<bool>,
    #[serde(default)]
    pub preset_can_build: Option<bool>,
    #[serde(default)]
    pub preset_can_trade: Option<bool>,
    #[serde(default)]
    pub starter_coins: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuperAdminInviteActionInput {
    pub action: String, // "create" | "revoke"
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub label_note: Option<String>,
    #[serde(default)]
    pub max_uses: Option<u32>,
    #[serde(default)]
    pub expires_in_hours: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeemFriendInviteInput {
    pub code: String,
    pub username: String,
    #[serde(default)]
    pub display_name: String,
    pub password: String,
    #[serde(default)]
    pub avatar: Option<Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeemFamilyInviteInput {
    pub code: String,
    pub world_name: String,
    #[serde(default)]
    pub world_slug: Option<String>,
    pub admin_password: String,
    pub host_username: String,
    #[serde(default)]
    pub host_display_name: String,
    pub host_password: String,
    #[serde(default)]
    pub avatar: Option<Value>,
}

pub struct PlaygroundRegistry {
    pub base_config: ServerConfig,
    registry_path: PathBuf,
    playgrounds_root: PathBuf,
    default_world: Arc<WorldEngine>,
    worlds: RwLock<HashMap<String, Arc<WorldEngine>>>,
    data: RwLock<PlatformRegistryFile>,
    avatar_sync_tx: mpsc::UnboundedSender<(String, String, AvatarAppearance)>,
}

impl PlaygroundRegistry {
    pub async fn new(default_world: Arc<WorldEngine>) -> anyhow::Result<Arc<Self>> {
        let base_config = default_world.config.clone();
        let registry_path = base_config.data_dir.join("platform_registry.json");
        let playgrounds_root = base_config.data_dir.join("playgrounds");
        fs::create_dir_all(&playgrounds_root).with_context(|| {
            format!(
                "Failed to create playgrounds directory {}",
                playgrounds_root.display()
            )
        })?;

        let mut loaded_data = if registry_path.exists() {
            let raw = fs::read_to_string(&registry_path)?;
            serde_json::from_str::<PlatformRegistryFile>(&raw)
                .with_context(|| format!("Invalid JSON in {}", registry_path.display()))?
        } else {
            PlatformRegistryFile::default()
        };
        loaded_data.invite_only_registration = true;

        let (avatar_sync_tx, mut avatar_sync_rx) =
            mpsc::unbounded_channel::<(String, String, AvatarAppearance)>();

        let mut world_map = HashMap::new();
        *default_world.avatar_sync_tx.write().await = Some(avatar_sync_tx.clone());
        world_map.insert(DEFAULT_WORLD_SLUG.to_string(), Arc::clone(&default_world));

        // Discover any persisted playgrounds under `data/playgrounds/<slug>/`
        let mut discovered_slugs: BTreeSet<String> =
            loaded_data.playgrounds.keys().cloned().collect();
        if let Ok(entries) = fs::read_dir(&playgrounds_root) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            if let Some(clean_slug) = normalize_world_slug(name) {
                                if clean_slug != DEFAULT_WORLD_SLUG
                                    && entry.path().join("world_users.json").exists()
                                {
                                    discovered_slugs.insert(clean_slug);
                                }
                            }
                        }
                    }
                }
            }
        }

        for slug in discovered_slugs {
            let pg_dir = playgrounds_root.join(&slug);
            if !pg_dir.join("world_users.json").exists() {
                continue;
            }
            let mut pg_cfg = base_config.clone();
            pg_cfg.data_dir = pg_dir.clone();
            pg_cfg.seed_users_file = pg_dir.join("world_users.json");
            let engine = WorldEngine::new_with_slug(pg_cfg, slug.clone())?;
            *engine.avatar_sync_tx.write().await = Some(avatar_sync_tx.clone());
            world_map.insert(slug, engine);
        }

        let reg = Arc::new(Self {
            base_config,
            registry_path,
            playgrounds_root,
            default_world,
            worlds: RwLock::new(world_map),
            data: RwLock::new(loaded_data),
            avatar_sync_tx,
        });

        reg.sync_all_worlds_into_registry().await?;

        let bg_reg = Arc::clone(&reg);
        tokio::spawn(async move {
            while let Some((source_slug, username, avatar)) = avatar_sync_rx.recv().await {
                bg_reg
                    .sync_avatar_across_worlds(&username, &avatar, Some(&source_slug))
                    .await;
            }
        });

        Ok(reg)
    }

    pub fn default_world(&self) -> Arc<WorldEngine> {
        Arc::clone(&self.default_world)
    }

    pub async fn resolve_world(&self, slug: &str) -> Option<Arc<WorldEngine>> {
        let key = slug.trim().to_ascii_lowercase();
        let effective = if key.is_empty() {
            DEFAULT_WORLD_SLUG
        } else {
            key.as_str()
        };
        self.worlds.read().await.get(effective).cloned()
    }

    pub async fn all_worlds(&self) -> Vec<(String, Arc<WorldEngine>)> {
        let map = self.worlds.read().await;
        let mut out: Vec<(String, Arc<WorldEngine>)> = map
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        out.sort_by(|a, b| {
            if a.0 == DEFAULT_WORLD_SLUG {
                std::cmp::Ordering::Less
            } else if b.0 == DEFAULT_WORLD_SLUG {
                std::cmp::Ordering::Greater
            } else {
                a.0.cmp(&b.0)
            }
        });
        out
    }

    pub async fn total_active_players(&self) -> usize {
        let worlds = self.all_worlds().await;
        let mut total = 0usize;
        for (_, w) in worlds {
            total += w.players.read().await.len();
        }
        total
    }

    pub fn verify_token_claims(&self, token: &str) -> Option<TokenClaims> {
        verify_token(token.trim(), &self.base_config.hmac_secret)
    }

    pub async fn resolve_world_for_token(&self, token: &str) -> Option<(String, Arc<WorldEngine>)> {
        let (_claims, slug) = verify_world_token(token.trim(), &self.base_config.hmac_secret)?;
        let world = self.resolve_world(&slug).await?;
        Some((slug, world))
    }

    pub fn verify_superadmin_password(&self, candidate: &str) -> bool {
        let expected = self.base_config.superadmin_password();
        constant_time_str_eq(&expected, candidate)
    }

    pub fn mint_superadmin_token(&self) -> String {
        let now = now_ms();
        let exp = now.saturating_add(self.base_config.token_ttl_ms());
        let super_pw = self.base_config.superadmin_password();
        let pwd_sig =
            password_token_sig("__superadmin__", &super_pw, &self.base_config.hmac_secret);
        sign_token(
            &TokenClaims {
                self_id: None,
                username: Some("superadmin".into()),
                role: Some("superadmin".into()),
                iat: now,
                exp: Some(exp),
                session_epoch: Some(1),
                user_token_epoch: Some(0),
                pwd_sig: Some(pwd_sig),
                device_id: None,
            },
            &self.base_config.hmac_secret,
        )
    }

    pub fn verify_superadmin_token(&self, token: &str) -> bool {
        let Some(claims) = self.verify_token_claims(token) else {
            return false;
        };
        if claims.role.as_deref() != Some("superadmin") {
            return false;
        }
        let Some(exp) = claims.exp else {
            return false;
        };
        if now_ms() > exp {
            return false;
        }
        let Some(claim_sig) = claims.pwd_sig else {
            return false;
        };
        let super_pw = self.base_config.superadmin_password();
        let expected_sig =
            password_token_sig("__superadmin__", &super_pw, &self.base_config.hmac_secret);
        constant_time_str_eq(&claim_sig, &expected_sig)
    }

    async fn save_registry(&self) -> anyhow::Result<()> {
        let snap = self.data.read().await.clone();
        write_atomic_json(&self.registry_path, &snap)
    }

    /// Synchronizes any pre-existing or CLI-edited accounts from all loaded `WorldEngine`s
    /// into `platform_registry.json` with zero data loss or migration breakage.
    pub async fn sync_all_worlds_into_registry(&self) -> anyhow::Result<()> {
        let worlds = self.all_worlds().await;
        let now = now_ms();
        {
            let mut reg = self.data.write().await;
            for (slug, world) in worlds {
                let cfg = world.users_cfg.read().await;
                let host_uname = cfg
                    .users
                    .iter()
                    .find(|u| u.is_host())
                    .or_else(|| cfg.users.first())
                    .map(|u| u.username.to_lowercase())
                    .unwrap_or_else(|| "ella".to_string());

                reg.playgrounds
                    .entry(slug.clone())
                    .and_modify(|m| {
                        m.world_name = cfg.world_name.clone();
                    })
                    .or_insert(PlaygroundMetaRecord {
                        slug: slug.clone(),
                        world_name: cfg.world_name.clone(),
                        host_username: host_uname,
                        created_at_ms: now,
                        created_via_code: None,
                    });

                for u in &cfg.users {
                    let uname_lc = u.username.trim().to_lowercase();
                    if uname_lc.is_empty() {
                        continue;
                    }
                    let disp = if u.display_name.trim().is_empty() {
                        u.username.clone()
                    } else {
                        u.display_name.clone()
                    };
                    reg.users
                        .entry(uname_lc.clone())
                        .and_modify(|gu| {
                            gu.member_worlds.insert(slug.clone());
                            if slug == DEFAULT_WORLD_SLUG || gu.member_worlds.len() <= 1 {
                                gu.password = u.password.clone();
                                gu.display_name = disp.clone();
                            }
                        })
                        .or_insert_with(|| {
                            let mut set = BTreeSet::new();
                            set.insert(slug.clone());
                            GlobalUserIdentity {
                                username: uname_lc,
                                display_name: disp,
                                password: u.password.clone(),
                                avatar: u.avatar.clone(),
                                member_worlds: set,
                                created_at_ms: now,
                                invited_by_code: None,
                            }
                        });
                }
            }
        }
        self.save_registry().await
    }

    pub async fn sync_admin_upserted_user(
        &self,
        world_slug: &str,
        username: &str,
        display_name: &str,
        password: &str,
        avatar: &AvatarAppearance,
    ) {
        let uname_lc = username.trim().to_lowercase();
        if uname_lc.is_empty() {
            return;
        }
        {
            let mut reg = self.data.write().await;
            reg.users
                .entry(uname_lc.clone())
                .and_modify(|gu| {
                    gu.member_worlds.insert(world_slug.to_string());
                    if gu.member_worlds.len() <= 1 || world_slug == DEFAULT_WORLD_SLUG {
                        gu.password = password.to_string();
                        gu.display_name = display_name.to_string();
                    }
                })
                .or_insert_with(|| {
                    let mut set = BTreeSet::new();
                    set.insert(world_slug.to_string());
                    GlobalUserIdentity {
                        username: uname_lc,
                        display_name: display_name.to_string(),
                        password: password.to_string(),
                        avatar: avatar.clone(),
                        member_worlds: set,
                        created_at_ms: now_ms(),
                        invited_by_code: None,
                    }
                });
        }
        let _ = self.save_registry().await;
    }

    pub async fn sync_admin_deleted_user(&self, world_slug: &str, username: &str) {
        let uname_lc = username.trim().to_lowercase();
        {
            let mut reg = self.data.write().await;
            let mut remove_global = false;
            if let Some(gu) = reg.users.get_mut(&uname_lc) {
                gu.member_worlds.remove(world_slug);
                remove_global = gu.member_worlds.is_empty();
            }
            if remove_global {
                reg.users.remove(&uname_lc);
            }
        }
        let _ = self.save_registry().await;
    }

    /// Propagates a player's latest 3D Avatar appearance to `GlobalUserIdentity` and across all
    /// other Playgrounds they belong to (reconciling exclusive loot per Playground inventory!).
    pub async fn sync_avatar_across_worlds(
        &self,
        username: &str,
        new_avatar: &AvatarAppearance,
        skip_world_slug: Option<&str>,
    ) {
        let uname_lc = username.trim().to_lowercase();
        if uname_lc.is_empty() {
            return;
        }
        let member_worlds: Vec<String> = {
            let mut reg = self.data.write().await;
            if let Some(gu) = reg.users.get_mut(&uname_lc) {
                gu.avatar = new_avatar.clone();
                gu.member_worlds.iter().cloned().collect()
            } else {
                Vec::new()
            }
        };
        let _ = self.save_registry().await;

        for slug in member_worlds {
            if skip_world_slug == Some(slug.as_str()) {
                continue;
            }
            let Some(world) = self.resolve_world(&slug).await else {
                continue;
            };
            let maybe_updated = {
                let mut cfg = world.users_cfg.write().await;
                if let Some(u) = cfg
                    .users
                    .iter_mut()
                    .find(|u| u.username.eq_ignore_ascii_case(&uname_lc))
                {
                    let mut candidate = new_avatar.clone();
                    reconcile_equipped_cosmetics(&mut candidate, &u.inventory);
                    u.avatar = candidate;
                    Some(u.sanitized())
                } else {
                    None
                }
            };
            if let Some(sanitized) = maybe_updated {
                world.persist_users_async().await;
                let avatar_hash = sanitized.visual_hash();
                let mut live_self_id: Option<String> = None;
                {
                    let mut players = world.players.write().await;
                    for p in players.values_mut() {
                        if p.user.username.eq_ignore_ascii_case(&uname_lc) {
                            p.user = sanitized.clone();
                            p.avatar_hash = avatar_hash.clone();
                            live_self_id = Some(p.id.clone());
                        }
                    }
                }
                if let Some(sid) = live_self_id {
                    let seq = world.next_state_seq();
                    let _ = world.udp_event_tx.send(UdpWorldEvent::AvatarUpdated {
                        state_seq: seq,
                        self_id: sid,
                        avatar_hash,
                        user: sanitized,
                    });
                }
            }
        }
    }

    pub async fn list_public_playgrounds(&self) -> Vec<Value> {
        let worlds = self.all_worlds().await;
        let mut list = Vec::with_capacity(worlds.len());
        for (slug, w) in worlds {
            let cfg = w.users_cfg.read().await;
            let online = w.players.read().await.len();
            list.push(json!({
                "slug": slug,
                "worldName": cfg.world_name,
                "onlineCount": online,
                "worldLocked": cfg.world_locked,
                "isDefault": slug == DEFAULT_WORLD_SLUG,
            }));
        }
        list
    }

    pub async fn my_playgrounds_for_user(&self, username: &str) -> Vec<Value> {
        let uname_lc = username.trim().to_lowercase();
        let mut slugs: BTreeSet<String> = {
            let reg = self.data.read().await;
            reg.users
                .get(&uname_lc)
                .map(|u| u.member_worlds.clone())
                .unwrap_or_default()
        };

        let worlds = self.all_worlds().await;
        for (slug, w) in &worlds {
            let cfg = w.users_cfg.read().await;
            if cfg
                .users
                .iter()
                .any(|u| u.username.eq_ignore_ascii_case(&uname_lc))
            {
                slugs.insert(slug.clone());
            }
        }

        let mut ordered: Vec<String> = slugs.into_iter().collect();
        ordered.sort_by(|a, b| {
            if a == DEFAULT_WORLD_SLUG {
                std::cmp::Ordering::Less
            } else if b == DEFAULT_WORLD_SLUG {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        let mut out = Vec::with_capacity(ordered.len());
        for slug in ordered {
            let Some(w) = self.resolve_world(&slug).await else {
                continue;
            };
            let cfg = w.users_cfg.read().await;
            let online = w.players.read().await.len();
            let Some(u_acc) = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(&uname_lc))
            else {
                continue;
            };
            out.push(json!({
                "slug": slug,
                "worldName": cfg.world_name,
                "role": u_acc.role,
                "coins": u_acc.coins,
                "onlineCount": online,
                "worldLocked": cfg.world_locked,
                "isDefault": slug == DEFAULT_WORLD_SLUG,
            }));
        }
        out
    }

    pub async fn resolve_login_world_and_user(
        &self,
        raw_username: &str,
        password: &str,
        requested_slug: Option<&str>,
    ) -> Result<(String, Arc<WorldEngine>, UserAccount), (StatusCode, String)> {
        let uname = raw_username.trim().to_lowercase();
        if uname.is_empty() || password.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Please enter your username and password.".into(),
            ));
        }

        // Self-heal in case `world_users.json` was updated on disk or via admin endpoint
        let (global_hit, member_worlds) = {
            let reg = self.data.read().await;
            if let Some(gu) = reg.users.get(&uname) {
                (
                    Some((gu.password.clone(), gu.avatar.clone())),
                    gu.member_worlds.clone(),
                )
            } else {
                (None, BTreeSet::new())
            }
        };

        let target_slug = if let Some(req) = requested_slug
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase)
        {
            req
        } else if member_worlds.contains(DEFAULT_WORLD_SLUG) || member_worlds.is_empty() {
            DEFAULT_WORLD_SLUG.to_string()
        } else {
            member_worlds
                .iter()
                .next()
                .cloned()
                .unwrap_or_else(|| DEFAULT_WORLD_SLUG.to_string())
        };

        let Some(world) = self.resolve_world(&target_slug).await else {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Playground '{target_slug}' does not exist."),
            ));
        };

        let local_user = {
            let cfg = world.users_cfg.read().await;
            cfg.users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(&uname))
                .cloned()
        };

        // Verify password against local world account OR global identity (when switching/logging into a secondary world)
        let password_ok = match (&local_user, &global_hit) {
            (Some(lu), _) if constant_time_str_eq(&lu.password, password) => true,
            (Some(_), Some((gpw, _))) if constant_time_str_eq(gpw, password) => true,
            _ => false,
        };

        if !password_ok {
            // Check if the user actually belongs to another playground when no requested_slug was passed
            if requested_slug.is_none() {
                for (other_slug, other_w) in self.all_worlds().await {
                    if other_slug == target_slug {
                        continue;
                    }
                    let maybe_u = {
                        let cfg = other_w.users_cfg.read().await;
                        cfg.users
                            .iter()
                            .find(|u| {
                                u.username.eq_ignore_ascii_case(&uname)
                                    && constant_time_str_eq(&u.password, password)
                            })
                            .cloned()
                    };
                    if let Some(mut found_u) = maybe_u {
                        if let Some((_, ref global_av)) = global_hit {
                            let mut synced = global_av.clone();
                            reconcile_equipped_cosmetics(&mut synced, &found_u.inventory);
                            found_u.avatar = synced;
                        }
                        let _ = self.sync_all_worlds_into_registry().await;
                        return Ok((other_slug, other_w, found_u));
                    }
                }
            }
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid username or password. Ella's World is invite-only — redeem a STAR-... code or ask a parent to check the Admin Panel!".into(),
            ));
        }

        let Some(mut user) = local_user else {
            return Err((
                StatusCode::FORBIDDEN,
                format!("You haven't joined playground '{target_slug}' yet! Redeem a STAR-... Friend Invite code to unlock it."),
            ));
        };

        // Ensure global avatar appearance carries into this world on login
        if let Some((_, global_av)) = global_hit {
            let mut candidate = global_av;
            reconcile_equipped_cosmetics(&mut candidate, &user.inventory);
            if user.avatar != candidate {
                user.avatar = candidate.clone();
                {
                    let mut cfg = world.users_cfg.write().await;
                    if let Some(u_mut) = cfg
                        .users
                        .iter_mut()
                        .find(|u| u.username.eq_ignore_ascii_case(&uname))
                    {
                        u_mut.avatar = candidate;
                    }
                }
                world.persist_users_async().await;
            }
        } else {
            let _ = self.sync_all_worlds_into_registry().await;
        }

        Ok((target_slug, world, user))
    }

    pub async fn prepare_world_switch(
        &self,
        username: &str,
        target_raw_slug: &str,
    ) -> Result<(String, Arc<WorldEngine>, UserAccount), (StatusCode, String)> {
        let uname = username.trim().to_lowercase();
        let Some(target_slug) = normalize_world_slug(target_raw_slug) else {
            return Err((StatusCode::BAD_REQUEST, "Invalid playground slug.".into()));
        };
        let Some(target_world) = self.resolve_world(&target_slug).await else {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Playground '{target_slug}' was not found."),
            ));
        };

        let global_avatar = {
            let reg = self.data.read().await;
            if let Some(gu) = reg.users.get(&uname) {
                if !gu.member_worlds.contains(&target_slug) {
                    // Check if directly listed in target_world users_cfg
                    let in_world = target_world
                        .users_cfg
                        .read()
                        .await
                        .users
                        .iter()
                        .any(|u| u.username.eq_ignore_ascii_case(&uname));
                    if !in_world {
                        return Err((
                            StatusCode::FORBIDDEN,
                            "You have not been invited to that Playground yet! Enter a STAR-... Friend Invite code first.".into(),
                        ));
                    }
                }
                Some(gu.avatar.clone())
            } else {
                None
            }
        };

        let maybe_user = {
            let mut cfg = target_world.users_cfg.write().await;
            if let Some(u) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(&uname))
            {
                if let Some(av) = global_avatar {
                    let mut synced = av;
                    reconcile_equipped_cosmetics(&mut synced, &u.inventory);
                    u.avatar = synced;
                }
                Some(u.clone())
            } else {
                None
            }
        };

        let Some(user) = maybe_user else {
            return Err((
                StatusCode::FORBIDDEN,
                "Your account is not registered in that Playground.".into(),
            ));
        };
        target_world.persist_users_async().await;
        Ok((target_slug, target_world, user))
    }

    pub async fn inspect_invite(&self, raw_code: &str) -> Result<Value, (StatusCode, String)> {
        let code = normalize_invite_code(raw_code);
        if code.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Please enter an invite code.".into(),
            ));
        }
        let now = now_ms();
        let ticket = {
            let reg = self.data.read().await;
            reg.invites.get(&code).cloned()
        };
        let Some(t) = ticket else {
            return Err((
                StatusCode::NOT_FOUND,
                "Invite code not found. Double-check the letters and numbers!".into(),
            ));
        };

        let (kind_str, world_slug, world_name) = match &t.kind {
            InviteKind::FriendPlaydate { world_slug } => {
                let w_name = if let Some(w) = self.resolve_world(world_slug).await {
                    w.users_cfg.read().await.world_name.clone()
                } else {
                    world_slug.clone()
                };
                ("friend_playdate", Some(world_slug.clone()), Some(w_name))
            }
            InviteKind::FamilyFounder => ("family_founder", None, None),
        };

        Ok(json!({
            "ok": true,
            "code": t.code,
            "inviteType": kind_str,
            "status": t.effective_status(now),
            "redeemable": t.is_redeemable(now),
            "worldSlug": world_slug,
            "worldName": world_name,
            "labelNote": t.label_note,
            "maxUses": t.max_uses,
            "uses": t.uses,
            "remainingUses": t.max_uses.saturating_sub(t.uses),
            "expiresAtMs": t.expires_at_ms,
            "presetChatMode": t.preset_chat_mode,
            "presetVoiceMuted": t.preset_voice_muted,
            "presetCanBuild": t.preset_can_build,
            "presetCanTrade": t.preset_can_trade,
            "starterCoins": t.starter_coins,
        }))
    }

    pub async fn host_draft_friend_invite(
        &self,
        world_slug: &str,
        actor_username: &str,
        label_note: Option<&str>,
    ) -> Result<InviteTicket, (StatusCode, String)> {
        let Some(world) = self.resolve_world(world_slug).await else {
            return Err((StatusCode::NOT_FOUND, "Playground not found.".into()));
        };
        let is_host = {
            let cfg = world.users_cfg.read().await;
            cfg.users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(actor_username))
                .map(|u| u.is_host())
                .unwrap_or(false)
        };
        if !is_host {
            return Err((
                StatusCode::FORBIDDEN,
                "Only the Playground Host can draft friend invites for Parent approval!".into(),
            ));
        }

        let clean_note = label_note
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.chars().take(64).collect::<String>());

        let now = now_ms();
        let ticket = {
            let mut reg = self.data.write().await;
            let pending_count = reg
                .invites
                .values()
                .filter(|t| {
                    t.status == InviteStatus::PendingParentApproval
                        && matches!(&t.kind, InviteKind::FriendPlaydate { world_slug: ws } if ws == world_slug)
                })
                .count();
            if pending_count >= 25 {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    "You already have 25 pending friend invites! Ask a parent to review them in the Admin Panel.".into(),
                ));
            }
            let code = generate_unique_code("STAR", &reg.invites);
            let t = InviteTicket {
                code: code.clone(),
                kind: InviteKind::FriendPlaydate {
                    world_slug: world_slug.to_string(),
                },
                status: InviteStatus::PendingParentApproval,
                created_by: actor_username.trim().to_lowercase(),
                created_at_ms: now,
                label_note: clean_note,
                max_uses: 1,
                uses: 0,
                expires_at_ms: Some(now.saturating_add(7 * 24 * 3600 * 1_000)),
                preset_chat_mode: default_chat_mode(),
                preset_voice_muted: false,
                preset_can_build: true,
                preset_can_trade: true,
                starter_coins: 100,
                redeemed_by: Vec::new(),
            };
            reg.invites.insert(code, t.clone());
            t
        };

        let _ = self.save_registry().await;
        Ok(ticket)
    }

    pub async fn admin_list_invites(&self, world_slug: &str) -> Vec<InviteTicket> {
        let reg = self.data.read().await;
        let mut list: Vec<InviteTicket> = reg
            .invites
            .values()
            .filter(|t| {
                matches!(
                    &t.kind,
                    InviteKind::FriendPlaydate { world_slug: ws } if ws == world_slug
                )
            })
            .cloned()
            .collect();
        list.sort_by(|a, b| {
            let pa = a.status == InviteStatus::PendingParentApproval;
            let pb = b.status == InviteStatus::PendingParentApproval;
            pb.cmp(&pa)
                .then_with(|| b.created_at_ms.cmp(&a.created_at_ms))
        });
        list
    }

    pub async fn admin_manage_invite(
        &self,
        world_slug: &str,
        input: AdminInviteActionInput,
    ) -> Result<InviteTicket, (StatusCode, String)> {
        let action = input.action.trim().to_ascii_lowercase();
        let now = now_ms();

        let ticket =
            {
                let mut reg = self.data.write().await;
                match action.as_str() {
                    "create" => {
                        let code = generate_unique_code("STAR", &reg.invites);
                        let max_uses = input.max_uses.unwrap_or(1).clamp(1, 50);
                        let expires_at_ms = input
                            .expires_in_hours
                            .filter(|h| *h > 0)
                            .map(|h| now.saturating_add(h.min(24 * 365) * 3600 * 1_000));
                        let clean_note = input
                            .label_note
                            .as_deref()
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.chars().take(64).collect::<String>());
                        let t = InviteTicket {
                            code: code.clone(),
                            kind: InviteKind::FriendPlaydate {
                                world_slug: world_slug.to_string(),
                            },
                            status: InviteStatus::Active,
                            created_by: "parent_admin".into(),
                            created_at_ms: now,
                            label_note: clean_note,
                            max_uses,
                            uses: 0,
                            expires_at_ms,
                            preset_chat_mode: input
                                .preset_chat_mode
                                .as_deref()
                                .map(normalize_chat_mode)
                                .unwrap_or_else(default_chat_mode),
                            preset_voice_muted: input.preset_voice_muted.unwrap_or(false),
                            preset_can_build: input.preset_can_build.unwrap_or(true),
                            preset_can_trade: input.preset_can_trade.unwrap_or(true),
                            starter_coins: input.starter_coins.unwrap_or(100).clamp(0, 5_000),
                            redeemed_by: Vec::new(),
                        };
                        reg.invites.insert(code, t.clone());
                        t
                    }
                    "approve" | "reject" | "revoke" => {
                        let code = normalize_invite_code(input.code.as_deref().unwrap_or(""));
                        let Some(t) = reg.invites.get_mut(&code) else {
                            return Err((StatusCode::NOT_FOUND, "Invite code not found.".into()));
                        };
                        let belongs = matches!(
                            &t.kind,
                            InviteKind::FriendPlaydate { world_slug: ws } if ws == world_slug
                        );
                        if !belongs {
                            return Err((
                                StatusCode::FORBIDDEN,
                                "That invite code belongs to a different Playground.".into(),
                            ));
                        }
                        if action == "approve" {
                            t.status = InviteStatus::Active;
                            if let Some(mu) = input.max_uses {
                                t.max_uses = mu.clamp(1, 50);
                            }
                            if let Some(hours) = input.expires_in_hours.filter(|h| *h > 0) {
                                t.expires_at_ms =
                                    Some(now.saturating_add(hours.min(24 * 365) * 3600 * 1_000));
                            }
                            if let Some(cm) = &input.preset_chat_mode {
                                t.preset_chat_mode = normalize_chat_mode(cm);
                            }
                            if let Some(vm) = input.preset_voice_muted {
                                t.preset_voice_muted = vm;
                            }
                            if let Some(cb) = input.preset_can_build {
                                t.preset_can_build = cb;
                            }
                            if let Some(ct) = input.preset_can_trade {
                                t.preset_can_trade = ct;
                            }
                            if let Some(sc) = input.starter_coins {
                                t.starter_coins = sc.clamp(0, 5_000);
                            }
                        } else {
                            t.status = InviteStatus::Revoked;
                        }
                        t.clone()
                    }
                    _ => return Err((
                        StatusCode::BAD_REQUEST,
                        "Unsupported invite action (expected create, approve, reject, or revoke)."
                            .into(),
                    )),
                }
            };

        let _ = self.save_registry().await;
        Ok(ticket)
    }

    pub async fn superadmin_state(&self) -> Value {
        let worlds = self.all_worlds().await;
        let mut pg_views = Vec::with_capacity(worlds.len());
        for (slug, w) in worlds {
            let cfg = w.users_cfg.read().await;
            let online = w.players.read().await.len();
            let blocks = w.blocks_data.read().await.blocks.len();
            let host = cfg
                .users
                .iter()
                .find(|u| u.is_host())
                .map(|u| u.username.clone())
                .unwrap_or_default();
            pg_views.push(json!({
                "slug": slug,
                "worldName": cfg.world_name,
                "hostUsername": host,
                "userCount": cfg.users.len(),
                "onlineCount": online,
                "blockCount": blocks,
                "worldLocked": cfg.world_locked,
                "isDefault": slug == DEFAULT_WORLD_SLUG,
            }));
        }

        let reg = self.data.read().await;
        let mut family_invites: Vec<InviteTicket> = reg
            .invites
            .values()
            .filter(|t| matches!(t.kind, InviteKind::FamilyFounder))
            .cloned()
            .collect();
        family_invites.sort_by_key(|a| std::cmp::Reverse(a.created_at_ms));

        json!({
            "ok": true,
            "inviteOnlyRegistration": reg.invite_only_registration,
            "globalUserCount": reg.users.len(),
            "playgrounds": pg_views,
            "familyInvites": family_invites,
        })
    }

    pub async fn superadmin_manage_invite(
        &self,
        input: SuperAdminInviteActionInput,
    ) -> Result<InviteTicket, (StatusCode, String)> {
        let action = input.action.trim().to_ascii_lowercase();
        let now = now_ms();
        let ticket = {
            let mut reg = self.data.write().await;
            match action.as_str() {
                "create" => {
                    let code = generate_unique_code("FAMILY", &reg.invites);
                    let max_uses = input.max_uses.unwrap_or(1).clamp(1, 25);
                    let expires_at_ms = input
                        .expires_in_hours
                        .filter(|h| *h > 0)
                        .map(|h| now.saturating_add(h.min(24 * 365) * 3600 * 1_000));
                    let clean_note = input
                        .label_note
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.chars().take(64).collect::<String>());
                    let t = InviteTicket {
                        code: code.clone(),
                        kind: InviteKind::FamilyFounder,
                        status: InviteStatus::Active,
                        created_by: "superadmin".into(),
                        created_at_ms: now,
                        label_note: clean_note,
                        max_uses,
                        uses: 0,
                        expires_at_ms,
                        preset_chat_mode: default_chat_mode(),
                        preset_voice_muted: false,
                        preset_can_build: true,
                        preset_can_trade: true,
                        starter_coins: 250,
                        redeemed_by: Vec::new(),
                    };
                    reg.invites.insert(code, t.clone());
                    t
                }
                "revoke" => {
                    let code = normalize_invite_code(input.code.as_deref().unwrap_or(""));
                    let Some(t) = reg.invites.get_mut(&code) else {
                        return Err((StatusCode::NOT_FOUND, "Invite code not found.".into()));
                    };
                    t.status = InviteStatus::Revoked;
                    t.clone()
                }
                _ => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Unsupported superadmin invite action (expected create or revoke).".into(),
                    ))
                }
            }
        };
        let _ = self.save_registry().await;
        Ok(ticket)
    }

    /// Redeems an active `STAR-...` Friend Invite to create a new kid account (or attach an
    /// existing global kid account with matching password) into the target Playground.
    pub async fn redeem_friend_invite(
        &self,
        input: RedeemFriendInviteInput,
    ) -> Result<(String, Arc<WorldEngine>, UserAccount, InviteTicket), (StatusCode, String)> {
        let code = normalize_invite_code(&input.code);
        if code.is_empty() {
            return Err((
                StatusCode::FORBIDDEN,
                "Ella's World is invite-only! Please enter a valid STAR-... Friend Invite code."
                    .into(),
            ));
        }

        let clean_username = sanitize_username(&input.username);
        let clean_pw = input.password.trim().to_string();
        if clean_username.len() < 2 || clean_pw.len() < 4 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Username (2+ letters/numbers) and password (4+ characters) are required.".into(),
            ));
        }
        let clean_display = if input.display_name.trim().is_empty() {
            clean_username.clone()
        } else {
            input.display_name.trim().chars().take(32).collect()
        };

        let now = now_ms();
        let (target_slug, ticket_preview) = {
            let reg = self.data.read().await;
            let Some(t) = reg.invites.get(&code) else {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Invalid invite code! Ask your friend or parent for an active STAR-... invite."
                        .into(),
                ));
            };
            let world_slug = match &t.kind {
                InviteKind::FriendPlaydate { world_slug } => world_slug.clone(),
                InviteKind::FamilyFounder => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "That is a FAMILY-... Parent Founder code! Use 'Create Family Playground' to set up your own world.".into(),
                    ));
                }
            };
            validate_ticket_redeemable(t, now)?;
            (world_slug, t.clone())
        };

        let Some(world) = self.resolve_world(&target_slug).await else {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Target Playground '{target_slug}' does not exist."),
            ));
        };

        // Check global user identity & world membership atomically
        let resolved_avatar = {
            let mut reg = self.data.write().await;
            let Some(t_check) = reg.invites.get(&code) else {
                return Err((StatusCode::FORBIDDEN, "Invite code not found.".into()));
            };
            validate_ticket_redeemable(t_check, now)?;

            let av = if let Some(existing_gu) = reg.users.get_mut(&clean_username) {
                if !constant_time_str_eq(&existing_gu.password, &clean_pw) {
                    return Err((
                        StatusCode::CONFLICT,
                        format!(
                            "Username @{clean_username} is already taken! If that's you, enter your existing password to link this Playground, or pick another username."
                        ),
                    ));
                }
                if existing_gu.member_worlds.contains(&target_slug) {
                    return Err((
                        StatusCode::CONFLICT,
                        format!(
                            "@{clean_username} is already a member of this Playground! Switch to 'Sign In' to join."
                        ),
                    ));
                }
                if let Some(patch) = &input.avatar {
                    existing_gu.avatar.apply_patch(patch);
                }
                existing_gu.member_worlds.insert(target_slug.clone());
                existing_gu.avatar.clone()
            } else {
                let mut av = AvatarAppearance::default();
                if let Some(patch) = &input.avatar {
                    av.apply_patch(patch);
                }
                let mut set = BTreeSet::new();
                set.insert(target_slug.clone());
                reg.users.insert(
                    clean_username.clone(),
                    GlobalUserIdentity {
                        username: clean_username.clone(),
                        display_name: clean_display.clone(),
                        password: clean_pw.clone(),
                        avatar: av.clone(),
                        member_worlds: set,
                        created_at_ms: now,
                        invited_by_code: Some(code.clone()),
                    },
                );
                av
            };

            let t_mut = reg.invites.get_mut(&code).expect("verified invite");
            t_mut.uses = t_mut.uses.saturating_add(1);
            if t_mut.uses >= t_mut.max_uses {
                t_mut.status = InviteStatus::Exhausted;
            }
            t_mut.redeemed_by.push(clean_username.clone());
            av
        };

        let created_user = {
            let mut cfg = world.users_cfg.write().await;
            if let Some(existing) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(&clean_username))
            {
                existing.avatar = resolved_avatar;
                existing.clone()
            } else {
                let mut nu = UserAccount::new_basic(
                    clean_username.clone(),
                    clean_display,
                    clean_pw,
                    "friend",
                );
                nu.avatar = resolved_avatar;
                nu.coins = ticket_preview.starter_coins;
                nu.chat_mode = normalize_chat_mode(&ticket_preview.preset_chat_mode);
                nu.voice_muted_by_parent = ticket_preview.preset_voice_muted;
                nu.can_build = ticket_preview.preset_can_build;
                nu.can_trade = ticket_preview.preset_can_trade;
                cfg.users.push(nu.clone());
                nu
            }
        };
        world.persist_users_async().await;
        let _ = self.save_registry().await;

        let final_ticket = self
            .data
            .read()
            .await
            .invites
            .get(&code)
            .cloned()
            .unwrap_or(ticket_preview);

        Ok((target_slug, world, created_user, final_ticket))
    }

    /// Allows an already signed-in kid to redeem a `STAR-...` Friend Invite from the
    /// `"My Playgrounds"` Switcher to unlock another friend's Playground.
    pub async fn join_world_with_friend_invite(
        &self,
        username: &str,
        raw_code: &str,
    ) -> Result<(String, Arc<WorldEngine>, UserAccount, InviteTicket), (StatusCode, String)> {
        let uname_lc = username.trim().to_lowercase();
        let code = normalize_invite_code(raw_code);
        if code.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Please enter a STAR-... Friend Invite code.".into(),
            ));
        }

        let now = now_ms();
        let (target_slug, ticket_preview) = {
            let reg = self.data.read().await;
            let Some(t) = reg.invites.get(&code) else {
                return Err((StatusCode::FORBIDDEN, "Invalid Friend Invite code.".into()));
            };
            let world_slug = match &t.kind {
                InviteKind::FriendPlaydate { world_slug } => world_slug.clone(),
                InviteKind::FamilyFounder => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "That is a FAMILY-... Parent Founder code, not a Friend Invite!".into(),
                    ));
                }
            };
            validate_ticket_redeemable(t, now)?;
            (world_slug, t.clone())
        };

        let Some(world) = self.resolve_world(&target_slug).await else {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Playground '{target_slug}' not found."),
            ));
        };

        let (disp_name, pw, avatar) = {
            let mut reg = self.data.write().await;
            let Some(t_check) = reg.invites.get(&code) else {
                return Err((StatusCode::FORBIDDEN, "Invite code not found.".into()));
            };
            validate_ticket_redeemable(t_check, now)?;

            let Some(gu) = reg.users.get_mut(&uname_lc) else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "Global user profile not found.".into(),
                ));
            };
            if gu.member_worlds.contains(&target_slug) {
                return Err((
                    StatusCode::CONFLICT,
                    "You have already joined that Playground! Select it in My Playgrounds to switch.".into(),
                ));
            }
            gu.member_worlds.insert(target_slug.clone());
            let info = (
                gu.display_name.clone(),
                gu.password.clone(),
                gu.avatar.clone(),
            );

            let t_mut = reg.invites.get_mut(&code).expect("verified invite");
            t_mut.uses = t_mut.uses.saturating_add(1);
            if t_mut.uses >= t_mut.max_uses {
                t_mut.status = InviteStatus::Exhausted;
            }
            t_mut.redeemed_by.push(uname_lc.clone());
            info
        };

        let user_acc = {
            let mut cfg = world.users_cfg.write().await;
            if let Some(existing) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(&uname_lc))
            {
                let mut synced = avatar;
                reconcile_equipped_cosmetics(&mut synced, &existing.inventory);
                existing.avatar = synced;
                existing.clone()
            } else {
                let mut nu = UserAccount::new_basic(uname_lc.clone(), disp_name, pw, "friend");
                let mut synced = avatar;
                reconcile_equipped_cosmetics(&mut synced, &nu.inventory);
                nu.avatar = synced;
                nu.coins = ticket_preview.starter_coins;
                nu.chat_mode = normalize_chat_mode(&ticket_preview.preset_chat_mode);
                nu.voice_muted_by_parent = ticket_preview.preset_voice_muted;
                nu.can_build = ticket_preview.preset_can_build;
                nu.can_trade = ticket_preview.preset_can_trade;
                cfg.users.push(nu.clone());
                nu
            }
        };
        world.persist_users_async().await;
        let _ = self.save_registry().await;

        let final_ticket = self
            .data
            .read()
            .await
            .invites
            .get(&code)
            .cloned()
            .unwrap_or(ticket_preview);

        Ok((target_slug, world, user_acc, final_ticket))
    }

    /// Redeems an active `FAMILY-...` Parent Founder code to provision an isolated
    /// `<Kid>'s World` Playground (`data/playgrounds/<slug>/`) with its own Parent Admin Password
    /// and initial Kid Host account.
    pub async fn redeem_family_founder_invite(
        &self,
        input: RedeemFamilyInviteInput,
    ) -> Result<(String, Arc<WorldEngine>, UserAccount, InviteTicket), (StatusCode, String)> {
        let code = normalize_invite_code(&input.code);
        if code.is_empty() {
            return Err((
                StatusCode::FORBIDDEN,
                "A valid FAMILY-... Parent Founder Invite Code is required to create a Playground."
                    .into(),
            ));
        }

        let world_name: String = input.world_name.trim().chars().take(64).collect();
        if world_name.len() < 3 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Playground Title must be at least 3 characters (e.g. \"Maya's Starlight World\")."
                    .into(),
            ));
        }
        let admin_pw = input.admin_password.trim().to_string();
        if admin_pw.len() < 4 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Parent Admin Password must be at least 4 characters.".into(),
            ));
        }

        let host_uname = sanitize_username(&input.host_username);
        let host_pw = input.host_password.trim().to_string();
        if host_uname.len() < 2 || host_pw.len() < 4 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Kid Host username (2+ chars) and password (4+ chars) are required.".into(),
            ));
        }
        let host_disp = if input.host_display_name.trim().is_empty() {
            host_uname.clone()
        } else {
            input.host_display_name.trim().chars().take(32).collect()
        };

        let raw_slug_candidate = input
            .world_slug
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("{host_uname}-world"));
        let Some(slug) = normalize_world_slug(&raw_slug_candidate) else {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid Playground URL slug (use 2–32 letters, numbers, or dashes).".into(),
            ));
        };
        if slug == DEFAULT_WORLD_SLUG || self.worlds.read().await.contains_key(&slug) {
            return Err((
                StatusCode::CONFLICT,
                format!("Playground slug '{slug}' is already taken! Choose another short name."),
            ));
        }

        let now = now_ms();
        let (host_avatar, final_ticket) = {
            let mut reg = self.data.write().await;
            if reg.playgrounds.contains_key(&slug) {
                return Err((
                    StatusCode::CONFLICT,
                    format!("Playground '{slug}' already exists."),
                ));
            }
            let Some(t_check) = reg.invites.get(&code) else {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Invalid FAMILY-... Parent Founder Invite Code.".into(),
                ));
            };
            if !matches!(t_check.kind, InviteKind::FamilyFounder) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "That code is a Friend Invite (STAR-...), not a FAMILY-... Founder Code."
                        .into(),
                ));
            }
            validate_ticket_redeemable(t_check, now)?;

            let starter_coins = t_check.starter_coins.max(250);

            let av = if let Some(existing_gu) = reg.users.get_mut(&host_uname) {
                if !constant_time_str_eq(&existing_gu.password, &host_pw) {
                    return Err((
                        StatusCode::CONFLICT,
                        format!(
                            "Username @{host_uname} already exists! Enter @{host_uname}'s existing password to link them as Host, or choose a new username."
                        ),
                    ));
                }
                if let Some(patch) = &input.avatar {
                    existing_gu.avatar.apply_patch(patch);
                }
                existing_gu.member_worlds.insert(slug.clone());
                existing_gu.avatar.clone()
            } else {
                let mut av = AvatarAppearance::default();
                if let Some(patch) = &input.avatar {
                    av.apply_patch(patch);
                }
                let mut set = BTreeSet::new();
                set.insert(slug.clone());
                reg.users.insert(
                    host_uname.clone(),
                    GlobalUserIdentity {
                        username: host_uname.clone(),
                        display_name: host_disp.clone(),
                        password: host_pw.clone(),
                        avatar: av.clone(),
                        member_worlds: set,
                        created_at_ms: now,
                        invited_by_code: Some(code.clone()),
                    },
                );
                av
            };

            let pg_dir = self.playgrounds_root.join(&slug);
            let mut host_account =
                UserAccount::new_basic(host_uname.clone(), host_disp, host_pw, "host");
            host_account.avatar = av.clone();
            host_account.coins = starter_coins;

            let initial_users_cfg = UsersConfigFile {
                world_name: world_name.clone(),
                admin_password: admin_pw,
                session_epoch: default_session_epoch(),
                voice_default_mode: "party".into(),
                ice_servers: default_ice_servers(),
                chat_filter_enabled: true,
                blocked_words: default_blocked_words(),
                global_voice_enabled: true,
                global_chat_mode: default_chat_mode(),
                world_locked: false,
                curfew_enabled: false,
                curfew_start: default_curfew_start(),
                curfew_end: default_curfew_end(),
                profanity_fine_base: default_profanity_fine_base(),
                debt_prison_threshold: default_debt_prison_threshold(),
                prison_strike_threshold: default_prison_strike_threshold(),
                enforce_machine_signatures: true,
                devices: Vec::new(),
                users: vec![host_account],
            };

            write_atomic_json(&pg_dir.join("world_users.json"), &initial_users_cfg)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            reg.playgrounds.insert(
                slug.clone(),
                PlaygroundMetaRecord {
                    slug: slug.clone(),
                    world_name,
                    host_username: host_uname.clone(),
                    created_at_ms: now,
                    created_via_code: Some(code.clone()),
                },
            );

            let t_mut = reg.invites.get_mut(&code).expect("verified family invite");
            t_mut.uses = t_mut.uses.saturating_add(1);
            if t_mut.uses >= t_mut.max_uses {
                t_mut.status = InviteStatus::Exhausted;
            }
            t_mut.redeemed_by.push(format!("{slug} (@{host_uname})"));
            (av, t_mut.clone())
        };

        let pg_dir = self.playgrounds_root.join(&slug);
        let mut pg_cfg = self.base_config.clone();
        pg_cfg.data_dir = pg_dir.clone();
        pg_cfg.seed_users_file = pg_dir.join("world_users.json");

        let engine = WorldEngine::new_with_slug(pg_cfg, slug.clone())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        *engine.avatar_sync_tx.write().await = Some(self.avatar_sync_tx.clone());

        let host_user = {
            let cfg = engine.users_cfg.read().await;
            cfg.users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(&host_uname))
                .cloned()
                .unwrap_or_else(|| {
                    let mut fallback = UserAccount::new_basic(&host_uname, &host_uname, "", "host");
                    fallback.avatar = host_avatar;
                    fallback
                })
        };

        self.worlds
            .write()
            .await
            .insert(slug.clone(), Arc::clone(&engine));
        let _ = self.save_registry().await;

        Ok((slug, engine, host_user, final_ticket))
    }
}

fn validate_ticket_redeemable(t: &InviteTicket, now: u64) -> Result<(), (StatusCode, String)> {
    match t.status {
        InviteStatus::PendingParentApproval => Err((
            StatusCode::FORBIDDEN,
            "⏳ This friend invite is waiting for Parent approval! Ask a parent to approve it in the Parent Admin Panel.".into(),
        )),
        InviteStatus::Revoked => Err((
            StatusCode::FORBIDDEN,
            "🚫 This invite code was revoked by a parent.".into(),
        )),
        InviteStatus::Exhausted => Err((
            StatusCode::FORBIDDEN,
            "🎟️ This invite code has already reached its maximum uses.".into(),
        )),
        InviteStatus::Active => {
            if t.uses >= t.max_uses {
                Err((
                    StatusCode::FORBIDDEN,
                    "🎟️ This invite code has already reached its maximum uses.".into(),
                ))
            } else if t.is_expired(now) {
                Err((
                    StatusCode::FORBIDDEN,
                    "⏰ This invite code has expired. Ask for a fresh code!".into(),
                ))
            } else {
                Ok(())
            }
        }
    }
}

pub fn sanitize_username(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(24)
        .collect()
}

pub fn normalize_world_slug(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in raw.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if (ch == '-' || ch == '_' || ch == ' ') && !out.is_empty() && !prev_dash {
            out.push('-');
            prev_dash = true;
        }
        if out.len() >= 32 {
            break;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.len() < 2 || RESERVED_WORLD_SLUGS.contains(&out.as_str()) {
        return None;
    }
    Some(out)
}

pub fn normalize_invite_code(raw: &str) -> String {
    let compact: String = raw
        .trim()
        .to_ascii_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if let Some(rest) = compact.strip_prefix("STAR") {
        if rest.len() == 8 {
            return format!("STAR-{}-{}", &rest[..4], &rest[4..8]);
        }
    }
    if let Some(rest) = compact.strip_prefix("FAMILY") {
        if rest.len() == 8 {
            return format!("FAMILY-{}-{}", &rest[..4], &rest[4..8]);
        }
    }
    raw.trim().to_ascii_uppercase().replace([' ', '_'], "-")
}

fn generate_unique_code(prefix: &str, existing: &BTreeMap<String, InviteTicket>) -> String {
    let mut rng = rand::thread_rng();
    loop {
        let mut part_a = String::with_capacity(4);
        let mut part_b = String::with_capacity(4);
        for _ in 0..4 {
            part_a.push(INVITE_ALPHABET[rng.gen_range(0..INVITE_ALPHABET.len())] as char);
            part_b.push(INVITE_ALPHABET[rng.gen_range(0..INVITE_ALPHABET.len())] as char);
        }
        let candidate = format!("{prefix}-{part_a}-{part_b}");
        if !existing.contains_key(&candidate) {
            return candidate;
        }
    }
}

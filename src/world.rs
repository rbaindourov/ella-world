use crate::auth::{sign_token, verify_token, TokenClaims};
use crate::config::ServerConfig;
use crate::models::{
    default_starter_blocks, is_valid_hex_color, now_ms, BlocksDataFile, ChatMessage,
    PlayerRuntimeState, RunwayPublicSummary, RunwayState, SanitizedUser, SavedPosition,
    UserAccount, UsersConfigFile, VoxelBlock, RUNWAY_THEMES,
};
use crate::persistence::PersistenceStore;
use crate::udp_wire::{encode_s2c_world_state, DecodedC2SState};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UdpWorldEvent {
    Roster {
        players: Vec<PlayerRuntimeState>,
        #[serde(rename = "blocksVersion")]
        blocks_version: u64,
    },
    PlayerJoin {
        player: PlayerRuntimeState,
    },
    PlayerLeave {
        id: String,
    },
    AvatarUpdated {
        #[serde(rename = "selfId")]
        self_id: String,
        user: SanitizedUser,
    },
    Chat {
        chat: ChatMessage,
    },
    Block {
        #[serde(rename = "selfId")]
        self_id: String,
        #[serde(rename = "blocksVersion")]
        blocks_version: u64,
        blocks: Vec<VoxelBlock>,
    },
    Runway {
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
    pub runway: RwLock<RunwayState>,
    pub server_tick: AtomicU32,
    pub udp_event_tx: broadcast::Sender<UdpWorldEvent>,
    pub opus_bridge_tx: broadcast::Sender<BridgedOpusFrame>,
}

impl WorldEngine {
    pub fn new(config: ServerConfig) -> anyhow::Result<Arc<Self>> {
        let store = PersistenceStore::new(&config.data_dir, &config.seed_users_file)?;
        let users_cfg = store.load_or_init_users()?;
        let blocks_data = store.load_or_init_blocks()?;
        let (udp_event_tx, _) = broadcast::channel(512);
        let (opus_bridge_tx, _) = broadcast::channel(1024);

        let engine = Arc::new(Self {
            config,
            store,
            users_cfg: RwLock::new(users_cfg),
            blocks_data: RwLock::new(blocks_data),
            players: RwLock::new(HashMap::new()),
            chats: RwLock::new(VecDeque::with_capacity(64)),
            runway: RwLock::new(RunwayState::default()),
            server_tick: AtomicU32::new(1),
            udp_event_tx,
            opus_bridge_tx,
        });

        let bg = Arc::clone(&engine);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
            let mut prune_counter = 0u32;
            loop {
                interval.tick().await;
                bg.server_tick.fetch_add(1, Ordering::Relaxed);
                prune_counter = prune_counter.wrapping_add(1);
                if prune_counter.is_multiple_of(60) {
                    bg.prune_stale_players().await;
                }
            }
        });

        Ok(engine)
    }

    pub fn current_tick(&self) -> u32 {
        self.server_tick.load(Ordering::Relaxed)
    }

    pub fn verify_session_token(&self, token: &str) -> Option<TokenClaims> {
        verify_token(token, &self.config.hmac_secret)
    }

    pub fn sign_session_token(&self, claims: &TokenClaims) -> String {
        sign_token(claims, &self.config.hmac_secret)
    }

    pub async fn prune_stale_players(&self) {
        let cutoff = now_ms().saturating_sub(35_000);
        let mut removed = Vec::new();
        {
            let mut map = self.players.write().await;
            map.retain(|id, p| {
                if p.last_seen < cutoff {
                    removed.push(id.clone());
                    false
                } else {
                    true
                }
            });
        }
        for id in removed {
            let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerLeave { id });
        }
    }

    pub async fn remove_player_immediately(&self, self_id: &str) {
        let existed = self.players.write().await.remove(self_id).is_some();
        if existed {
            let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerLeave {
                id: self_id.to_string(),
            });
        }
    }

    pub async fn active_players_list(&self) -> Vec<PlayerRuntimeState> {
        let map = self.players.read().await;
        map.values().cloned().collect()
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

        let active_count = self.players.read().await.len();
        let angle = (active_count as f32 * 1.35) % std::f32::consts::TAU;
        let default_x = (angle.cos() * 2.5 * 10.0).round() / 10.0;
        let default_z = (angle.sin() * 2.5 * 10.0).round() / 10.0;

        let (spawn_x, spawn_y, spawn_z, spawn_ry) = if let Some(pos) = &found_user.last_pos {
            (
                pos.x.clamp(-75.0, 75.0),
                pos.y.clamp(0.0, 75.0),
                pos.z.clamp(-75.0, 75.0),
                pos.ry,
            )
        } else {
            (default_x, 0.0, default_z, 0.0)
        };

        let sanitized = found_user.sanitized();
        let player_doc = PlayerRuntimeState {
            id: self_id.clone(),
            user: sanitized.clone(),
            x: spawn_x,
            y: spawn_y,
            z: spawn_z,
            ry: spawn_ry,
            anim: "idle".into(),
            emote: None,
            muted: false,
            speaking: false,
            last_seen: now_ms(),
            last_seq: 0,
            last_client_time_ms: 0,
        };

        {
            let mut map = self.players.write().await;
            map.insert(self_id.clone(), player_doc.clone());
        }

        let _ = self.udp_event_tx.send(UdpWorldEvent::PlayerJoin {
            player: player_doc.clone(),
        });

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
            "worldMeta": {
                "worldName": cfg.world_name,
                "voiceDefaultMode": cfg.voice_default_mode,
                "iceServers": cfg.ice_servers,
                "transport": "udp-webrtc-sfu",
                "webrtcUdpPort": self.config.webrtc_udp_port,
                "nativeUdpPort": self.config.native_udp_addr.port(),
                "rtcOfferUrl": "/api/ella-world/rtc/offer"
            }
        })
    }

    pub async fn apply_c2s_state(&self, self_id: &str, decoded: DecodedC2SState) {
        let now = now_ms();
        let mut maybe_save_pos: Option<(String, SavedPosition)> = None;

        {
            let mut map = self.players.write().await;
            if let Some(p) = map.get_mut(self_id) {
                // Drop stale out-of-order UDP datagrams (with wrap-around tolerance)
                if decoded.seq != 0
                    && p.last_seq != 0
                    && decoded.seq < p.last_seq
                    && (p.last_seq - decoded.seq) < 100_000
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
        if let Some(p) = self.players.write().await.get_mut(self_id) {
            p.last_seen = now_ms();
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

        let _ = self.udp_event_tx.send(UdpWorldEvent::Block {
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

        {
            let mut map = self.players.write().await;
            for p in map.values_mut() {
                if p.user.username.eq_ignore_ascii_case(username) {
                    p.user = sanitized.clone();
                }
            }
        }

        let _ = self.udp_event_tx.send(UdpWorldEvent::AvatarUpdated {
            self_id: self_id.to_string(),
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
    ) -> Option<ChatMessage> {
        let (disp_name, role) = {
            let cfg = self.users_cfg.read().await;
            let u = cfg
                .users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(username))?;
            let s = u.sanitized();
            (s.display_name, s.role)
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
            return None;
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

        let _ = self.udp_event_tx.send(UdpWorldEvent::Chat {
            chat: entry.clone(),
        });

        Some(entry)
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
            let _ = self.udp_event_tx.send(UdpWorldEvent::Runway {
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
            let _ = self.udp_event_tx.send(UdpWorldEvent::Runway {
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
        let _ = self.udp_event_tx.send(UdpWorldEvent::Block {
            self_id: "admin".into(),
            blocks_version: next_ver,
            blocks: next_blocks.clone(),
        });
        next_blocks
    }
}

use crate::models::{
    default_blocked_words, default_chat_mode, default_curfew_end, default_curfew_start,
    default_debt_prison_threshold, default_ice_servers, default_prison_strike_threshold,
    default_profanity_fine_base, default_session_epoch, default_starter_blocks, default_world_name,
    BlockedChatEntry, BlocksDataFile, ChatMessage, DeviceEvasionEntry, HomePlotsFile,
    ObbyLeaderboardsFile, UserAccount, UsersConfigFile,
};
use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct PersistenceStore {
    users_path: PathBuf,
    blocks_path: PathBuf,
    homes_path: PathBuf,
    leaderboards_path: PathBuf,
    seed_users_path: PathBuf,
    chat_log_path: PathBuf,
    blocked_chat_log_path: PathBuf,
    device_evasion_log_path: PathBuf,
}

impl PersistenceStore {
    pub fn new(data_dir: &Path, seed_users_path: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(data_dir)
            .with_context(|| format!("Failed to create data_dir {}", data_dir.display()))?;

        // Support both world_users.json and legacy ella_world_users.json seamlessly
        let legacy_users = data_dir.join("ella_world_users.json");
        let canonical_users = data_dir.join("world_users.json");
        let users_path = if !canonical_users.exists() && legacy_users.exists() {
            legacy_users
        } else {
            canonical_users
        };

        let legacy_blocks = data_dir.join("ella_world_blocks.json");
        let canonical_blocks = data_dir.join("world_blocks.json");
        let blocks_path = if !canonical_blocks.exists() && legacy_blocks.exists() {
            legacy_blocks
        } else {
            canonical_blocks
        };

        let homes_path = data_dir.join("world_homes.json");
        let leaderboards_path = data_dir.join("world_leaderboards.json");
        let chat_log_path = data_dir.join("world_chat.jsonl");
        let blocked_chat_log_path = data_dir.join("world_chat_blocked.jsonl");
        let device_evasion_log_path = data_dir.join("world_device_evasion.jsonl");

        Ok(Self {
            users_path,
            blocks_path,
            homes_path,
            leaderboards_path,
            seed_users_path: seed_users_path.to_path_buf(),
            chat_log_path,
            blocked_chat_log_path,
            device_evasion_log_path,
        })
    }

    pub fn users_file_path(&self) -> &Path {
        &self.users_path
    }

    pub fn load_or_init_users(&self) -> anyhow::Result<UsersConfigFile> {
        if self.users_path.exists() {
            let raw = fs::read_to_string(&self.users_path)?;
            let mut parsed: UsersConfigFile = serde_json::from_str(&raw)
                .with_context(|| format!("Invalid JSON in {}", self.users_path.display()))?;
            for u in &mut parsed.users {
                u.ensure_starter_companions();
            }
            return Ok(parsed);
        }

        let mut seeded = if self.seed_users_path.exists() {
            let raw = fs::read_to_string(&self.seed_users_path)?;
            serde_json::from_str::<UsersConfigFile>(&raw)?
        } else {
            UsersConfigFile {
                world_name: default_world_name(),
                admin_password: "changeme-parent-admin".into(),
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
                users: vec![UserAccount::new_basic(
                    "ella",
                    "Ella",
                    "changeme-starlight",
                    "host",
                )],
            }
        };
        for u in &mut seeded.users {
            u.ensure_starter_companions();
        }

        self.save_users(&seeded)?;
        Ok(seeded)
    }

    pub fn save_users(&self, config: &UsersConfigFile) -> anyhow::Result<()> {
        write_atomic_json(&self.users_path, config)
    }

    pub fn load_or_init_blocks(&self) -> anyhow::Result<BlocksDataFile> {
        if self.blocks_path.exists() {
            let raw = fs::read_to_string(&self.blocks_path)?;
            let parsed: BlocksDataFile = serde_json::from_str(&raw)
                .with_context(|| format!("Invalid JSON in {}", self.blocks_path.display()))?;
            return Ok(parsed);
        }

        let initial = BlocksDataFile {
            version: 1,
            blocks: default_starter_blocks(),
            prebuilt_village_v1_seeded: true,
        };
        self.save_blocks(&initial)?;
        Ok(initial)
    }

    pub fn save_blocks(&self, blocks: &BlocksDataFile) -> anyhow::Result<()> {
        write_atomic_json(&self.blocks_path, blocks)
    }

    pub fn load_or_init_homes(&self) -> anyhow::Result<HomePlotsFile> {
        if self.homes_path.exists() {
            let raw = fs::read_to_string(&self.homes_path)?;
            let mut parsed: HomePlotsFile = serde_json::from_str(&raw)
                .with_context(|| format!("Invalid JSON in {}", self.homes_path.display()))?;
            parsed.normalize();
            return Ok(parsed);
        }
        let initial = HomePlotsFile::default();
        self.save_homes(&initial)?;
        Ok(initial)
    }

    pub fn save_homes(&self, homes: &HomePlotsFile) -> anyhow::Result<()> {
        write_atomic_json(&self.homes_path, homes)
    }

    pub fn load_or_init_leaderboards(&self) -> anyhow::Result<ObbyLeaderboardsFile> {
        if self.leaderboards_path.exists() {
            let raw = fs::read_to_string(&self.leaderboards_path)?;
            let parsed: ObbyLeaderboardsFile = serde_json::from_str(&raw)
                .with_context(|| format!("Invalid JSON in {}", self.leaderboards_path.display()))?;
            return Ok(parsed);
        }
        let initial = ObbyLeaderboardsFile::default();
        self.save_leaderboards(&initial)?;
        Ok(initial)
    }

    pub fn save_leaderboards(&self, boards: &ObbyLeaderboardsFile) -> anyhow::Result<()> {
        write_atomic_json(&self.leaderboards_path, boards)
    }

    pub fn append_chat(&self, msg: &ChatMessage) -> anyhow::Result<()> {
        append_jsonl_line(&self.chat_log_path, msg)
    }

    pub fn append_blocked_chat(&self, entry: &BlockedChatEntry) -> anyhow::Result<()> {
        append_jsonl_line(&self.blocked_chat_log_path, entry)
    }

    pub fn load_recent_chats(&self, limit: usize) -> Vec<ChatMessage> {
        let mut all = read_jsonl_records::<ChatMessage>(&self.chat_log_path);
        if all.len() > limit {
            all.drain(0..(all.len() - limit));
        }
        all
    }

    pub fn query_chats(
        &self,
        limit: usize,
        user_filter: Option<&str>,
        search: Option<&str>,
    ) -> (usize, Vec<ChatMessage>) {
        query_jsonl_filtered(
            &self.chat_log_path,
            limit,
            user_filter,
            search,
            |c: &ChatMessage| (&c.username, &c.display_name, &c.text),
        )
    }

    pub fn query_blocked_chats(
        &self,
        limit: usize,
        user_filter: Option<&str>,
        search: Option<&str>,
    ) -> (usize, Vec<BlockedChatEntry>) {
        query_jsonl_filtered(
            &self.blocked_chat_log_path,
            limit,
            user_filter,
            search,
            |c: &BlockedChatEntry| (&c.username, &c.display_name, &c.text),
        )
    }

    pub fn count_chats(&self) -> usize {
        read_jsonl_records::<serde_json::Value>(&self.chat_log_path).len()
    }

    pub fn count_blocked_chats(&self) -> usize {
        read_jsonl_records::<serde_json::Value>(&self.blocked_chat_log_path).len()
    }

    pub fn append_device_evasion(&self, entry: &DeviceEvasionEntry) -> anyhow::Result<()> {
        append_jsonl_line(&self.device_evasion_log_path, entry)
    }

    pub fn load_recent_device_evasions(&self, limit: usize) -> Vec<DeviceEvasionEntry> {
        let mut all = read_jsonl_records::<DeviceEvasionEntry>(&self.device_evasion_log_path);
        if all.len() > limit {
            all.drain(0..(all.len() - limit));
        }
        all.reverse();
        all
    }

    pub fn count_device_evasions(&self) -> usize {
        read_jsonl_records::<serde_json::Value>(&self.device_evasion_log_path).len()
    }
}

fn query_jsonl_filtered<T, F>(
    path: &Path,
    limit: usize,
    user_filter: Option<&str>,
    search: Option<&str>,
    fields_fn: F,
) -> (usize, Vec<T>)
where
    T: DeserializeOwned,
    F: Fn(&T) -> (&str, &str, &str),
{
    let user_lc = user_filter
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());
    let search_lc = search
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());

    let all = read_jsonl_records::<T>(path);
    let filtered: Vec<T> = all
        .into_iter()
        .filter(|item| {
            let (username, display_name, text) = fields_fn(item);
            if let Some(u) = &user_lc {
                if !username.to_lowercase().contains(u) && !display_name.to_lowercase().contains(u)
                {
                    return false;
                }
            }
            if let Some(q) = &search_lc {
                if !text.to_lowercase().contains(q)
                    && !username.to_lowercase().contains(q)
                    && !display_name.to_lowercase().contains(q)
                {
                    return false;
                }
            }
            true
        })
        .collect();
    let total = filtered.len();
    let mut recent: Vec<T> = filtered.into_iter().rev().take(limit).collect();
    recent.reverse();
    (total, recent)
}

fn append_jsonl_line<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut line = serde_json::to_vec(value)?;
    line.push(b'\n');
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(&line)?;
    Ok(())
}

fn read_jsonl_records<T: DeserializeOwned>(path: &Path) -> Vec<T> {
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(item) = serde_json::from_str::<T>(trimmed) {
            out.push(item);
        }
    }
    out
}

fn write_atomic_json<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_name = format!(
        ".{}.tmp.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("state.json"),
        rand::random::<u32>()
    );
    let tmp_path = path.with_file_name(tmp_name);

    let mut payload = serde_json::to_vec_pretty(value)?;
    payload.push(b'\n');

    {
        let mut f = File::create(&tmp_path)?;
        f.write_all(&payload)?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}

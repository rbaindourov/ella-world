use crate::models::{
    default_ice_servers, default_starter_blocks, default_world_name, AvatarAppearance,
    BlocksDataFile, UserAccount, UsersConfigFile,
};
use anyhow::Context;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct PersistenceStore {
    users_path: PathBuf,
    blocks_path: PathBuf,
    seed_users_path: PathBuf,
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

        Ok(Self {
            users_path,
            blocks_path,
            seed_users_path: seed_users_path.to_path_buf(),
        })
    }

    pub fn users_file_path(&self) -> &Path {
        &self.users_path
    }

    pub fn load_or_init_users(&self) -> anyhow::Result<UsersConfigFile> {
        if self.users_path.exists() {
            let raw = fs::read_to_string(&self.users_path)?;
            let parsed: UsersConfigFile = serde_json::from_str(&raw)
                .with_context(|| format!("Invalid JSON in {}", self.users_path.display()))?;
            return Ok(parsed);
        }

        let seeded = if self.seed_users_path.exists() {
            let raw = fs::read_to_string(&self.seed_users_path)?;
            serde_json::from_str::<UsersConfigFile>(&raw)?
        } else {
            UsersConfigFile {
                world_name: default_world_name(),
                admin_password: "changeme-parent-admin".into(),
                voice_default_mode: "party".into(),
                ice_servers: default_ice_servers(),
                users: vec![UserAccount {
                    username: "ella".into(),
                    display_name: "Ella".into(),
                    password: "changeme-starlight".into(),
                    role: "host".into(),
                    avatar: AvatarAppearance::default(),
                    last_pos: None,
                    last_pos_saved_at: None,
                }],
            }
        };

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
        };
        self.save_blocks(&initial)?;
        Ok(initial)
    }

    pub fn save_blocks(&self, blocks: &BlocksDataFile) -> anyhow::Result<()> {
        write_atomic_json(&self.blocks_path, blocks)
    }
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

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const RUNWAY_THEMES: &[&str] = &[
    "✨ Starlight Pop Idol",
    "🔍 Mystery Detective (Forensics!)",
    "🧚‍♀️ Crystal Fairy Royalty",
    "🎀 Y2K Skater Chic",
    "👑 Red Carpet Ballgown Gala",
    "🚀 Cosmic Galaxy Explorer",
    "🐱 Kawaii Pastel Runway",
    "🎨 Parisian Fashion Week",
    "🌺 Sunset Beach Glam",
    "🌈 Rainbow Slumber Party",
    "🛸 Sky-Tower Tech Recon",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AvatarAppearance {
    #[serde(default = "default_avatar_mode")]
    pub avatar_mode: String,
    #[serde(default = "default_body_type")]
    pub body_type: String,
    #[serde(default = "default_preset")]
    pub character_preset: String,
    #[serde(default = "default_skin")]
    pub skin_color: String,
    #[serde(default = "default_eye")]
    pub eye_color: String,
    #[serde(default = "default_hair_style")]
    pub hair_style: String,
    #[serde(default = "default_hair_color")]
    pub hair_color: String,
    #[serde(default = "default_top_style")]
    pub top_style: String,
    #[serde(default = "default_top_color")]
    pub top_color: String,
    #[serde(default = "default_top_color")]
    pub torso_color: String,
    #[serde(default = "default_bottom_style")]
    pub bottom_style: String,
    #[serde(default = "default_bottom_color")]
    pub bottom_color: String,
    #[serde(default = "default_bottom_color")]
    pub legs_color: String,
    #[serde(default = "default_shoe_style")]
    pub shoe_style: String,
    #[serde(default = "default_shoe_color")]
    pub shoe_color: String,
    #[serde(default = "default_hat")]
    pub hat: String,
    #[serde(default = "default_shoe_color")]
    pub hat_color: String,
    #[serde(default = "default_accessory")]
    pub accessory: String,
    #[serde(default = "default_acc_color")]
    pub accessory_color: String,
    #[serde(default = "default_finish")]
    pub fabric_finish: String,
    #[serde(default = "default_face")]
    pub face: String,
}

fn default_avatar_mode() -> String {
    "stylized".into()
}
fn default_body_type() -> String {
    "female".into()
}
fn default_preset() -> String {
    "ella_idol".into()
}
fn default_skin() -> String {
    "#ffdfc4".into()
}
fn default_eye() -> String {
    "#60a5fa".into()
}
fn default_hair_style() -> String {
    "twin_pigtails".into()
}
fn default_hair_color() -> String {
    "#fde047".into()
}
fn default_top_style() -> String {
    "glam_gown".into()
}
fn default_top_color() -> String {
    "#f472b6".into()
}
fn default_bottom_style() -> String {
    "pleated_skirt".into()
}
fn default_bottom_color() -> String {
    "#6366f1".into()
}
fn default_shoe_style() -> String {
    "runway_heels".into()
}
fn default_shoe_color() -> String {
    "#fbbf24".into()
}
fn default_hat() -> String {
    "crown".into()
}
fn default_accessory() -> String {
    "star_purse".into()
}
fn default_acc_color() -> String {
    "#ec4899".into()
}
fn default_finish() -> String {
    "glitter".into()
}
fn default_face() -> String {
    "star".into()
}

impl Default for AvatarAppearance {
    fn default() -> Self {
        Self {
            avatar_mode: default_avatar_mode(),
            body_type: default_body_type(),
            character_preset: default_preset(),
            skin_color: default_skin(),
            eye_color: default_eye(),
            hair_style: default_hair_style(),
            hair_color: default_hair_color(),
            top_style: default_top_style(),
            top_color: default_top_color(),
            torso_color: default_top_color(),
            bottom_style: default_bottom_style(),
            bottom_color: default_bottom_color(),
            legs_color: default_bottom_color(),
            shoe_style: default_shoe_style(),
            shoe_color: default_shoe_color(),
            hat: default_hat(),
            hat_color: default_shoe_color(),
            accessory: default_accessory(),
            accessory_color: default_acc_color(),
            fabric_finish: default_finish(),
            face: default_face(),
        }
    }
}

pub fn is_valid_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

impl AvatarAppearance {
    pub fn apply_patch(&mut self, patch: &serde_json::Value) {
        let Some(obj) = patch.as_object() else {
            return;
        };
        let get_str = |k: &str| obj.get(k).and_then(|v| v.as_str());

        if let Some(v) = get_str("avatarMode") {
            if ["stylized", "blocky"].contains(&v) {
                self.avatar_mode = v.into();
            }
        }
        if let Some(v) = get_str("bodyType") {
            if ["female", "male"].contains(&v) {
                self.body_type = v.into();
            }
        }
        if let Some(v) = get_str("characterPreset") {
            if [
                "ella_idol",
                "katey",
                "lizzy",
                "amy",
                "mom",
                "fairy_queen",
                "y2k_skater",
                "runway_gala",
                "jack_tech49",
                "maverick_cabin",
                "custom",
            ]
            .contains(&v)
            {
                self.character_preset = v.into();
                if get_str("bodyType").is_none() {
                    if ["jack_tech49", "maverick_cabin"].contains(&v) {
                        self.body_type = "male".into();
                    } else if v != "custom" {
                        self.body_type = "female".into();
                    }
                }
            }
        }
        if let Some(v) = get_str("skinColor").filter(|c| is_valid_hex_color(c)) {
            self.skin_color = v.into();
        }
        if let Some(v) = get_str("eyeColor").filter(|c| is_valid_hex_color(c)) {
            self.eye_color = v.into();
        }
        if let Some(v) = get_str("hairStyle") {
            if [
                "katey_long",
                "amy_waves",
                "lizzy_bob",
                "mom_bun",
                "twin_pigtails",
                "glam_ponytail",
                "tech_crop",
                "maverick_quiff",
            ]
            .contains(&v)
            {
                self.hair_style = v.into();
            }
        }
        if let Some(v) = get_str("hairColor").filter(|c| is_valid_hex_color(c)) {
            self.hair_color = v.into();
        }
        if let Some(v) = get_str("topStyle") {
            if [
                "sweater",
                "star_tee",
                "crop_top",
                "crew_tee",
                "puff_blouse",
                "glam_gown",
                "tech_jacket",
                "hero_henley",
            ]
            .contains(&v)
            {
                self.top_style = v.into();
            }
        }
        if let Some(v) = get_str("topColor").filter(|c| is_valid_hex_color(c)) {
            self.top_color = v.into();
            self.torso_color = v.into();
        } else if let Some(v) = get_str("torsoColor").filter(|c| is_valid_hex_color(c)) {
            self.torso_color = v.into();
            self.top_color = v.into();
        }
        if let Some(v) = get_str("bottomStyle") {
            if [
                "pleated_skirt",
                "ruffle_mini",
                "jeans",
                "cargo_pants",
                "overalls",
                "tech_pants",
            ]
            .contains(&v)
            {
                self.bottom_style = v.into();
            }
        }
        if let Some(v) = get_str("bottomColor").filter(|c| is_valid_hex_color(c)) {
            self.bottom_color = v.into();
            self.legs_color = v.into();
        } else if let Some(v) = get_str("legsColor").filter(|c| is_valid_hex_color(c)) {
            self.legs_color = v.into();
            self.bottom_color = v.into();
        }
        if let Some(v) = get_str("shoeStyle") {
            if ["sneakers", "platform_boots", "runway_heels", "flight_boots"].contains(&v) {
                self.shoe_style = v.into();
            }
        }
        if let Some(v) = get_str("shoeColor").filter(|c| is_valid_hex_color(c)) {
            self.shoe_color = v.into();
        }
        if let Some(v) = get_str("hat") {
            if [
                "none",
                "crown",
                "catears",
                "hair_bow",
                "beret",
                "wizard",
                "astronaut",
                "wings",
                "tophat",
                "recon_cap",
                "ninja",
                "obsidian_crown",
            ]
            .contains(&v)
            {
                self.hat = v.into();
            }
        }
        if let Some(v) = get_str("hatColor").filter(|c| is_valid_hex_color(c)) {
            self.hat_color = v.into();
        }
        if let Some(v) = get_str("accessory") {
            if [
                "none",
                "star_purse",
                "sunglasses",
                "aviators",
                "choker",
                "pearls",
                "wings",
                "crystal_wings",
                "star_lantern",
                "ray_gun",
            ]
            .contains(&v)
            {
                self.accessory = v.into();
            }
        }
        if let Some(v) = get_str("accessoryColor").filter(|c| is_valid_hex_color(c)) {
            self.accessory_color = v.into();
        }
        if let Some(v) = get_str("fabricFinish") {
            if ["matte", "satin", "glitter"].contains(&v) {
                self.fabric_finish = v.into();
            }
        }
        if let Some(v) = get_str("face") {
            if ["happy", "star", "cat", "cool", "wink", "silly", "hero"].contains(&v) {
                self.face = v.into();
            }
        }
    }

    /// Deterministic 16-hex-character digest of all outfit fields (immune to JS `Number` 53-bit precision limits).
    pub fn short_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        for field in [
            &self.avatar_mode,
            &self.body_type,
            &self.character_preset,
            &self.skin_color,
            &self.eye_color,
            &self.hair_style,
            &self.hair_color,
            &self.top_style,
            &self.top_color,
            &self.bottom_style,
            &self.bottom_color,
            &self.shoe_style,
            &self.shoe_color,
            &self.hat,
            &self.hat_color,
            &self.accessory,
            &self.accessory_color,
            &self.fabric_finish,
            &self.face,
        ] {
            h.update(field.as_bytes());
            h.update(b"|");
        }
        let digest = h.finalize();
        let mut hex = String::with_capacity(16);
        for b in &digest[..8] {
            use std::fmt::Write;
            let _ = write!(&mut hex, "{b:02x}");
        }
        hex
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ry: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IceServerConfig {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

pub const SAFE_QUICK_CHAT_PHRASES: &[&str] = &[
    "⭐⭐⭐⭐⭐ 5 STARS! Slay!!",
    "💃 Meet me at the Fashion Runway!",
    "🎉 Let's race the Sky Obby!",
    "👗 Check out my new Glam Closet look!",
    "👋 Hi besties!!",
    "💎 Let's explore Crystal Cavern!",
    "🙏 Sorry! Working hard!",
    "👍 Good game!",
];

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerEconomySnapshot {
    pub coins: i64,
    pub inventory: HashMap<String, u32>,
    #[serde(default)]
    pub cloneable_items: HashSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerModerationSnapshot {
    #[serde(default)]
    pub violation_strikes: u32,
    #[serde(default)]
    pub in_debtors_prison: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prison_reason: Option<String>,
    #[serde(default)]
    pub prison_tasks_remaining: u32,
    #[serde(default)]
    pub voice_muted_by_parent: bool,
    #[serde(default = "default_chat_mode")]
    pub chat_mode: String,
    #[serde(default)]
    pub suspended: bool,
    #[serde(default = "default_true")]
    pub can_build: bool,
    #[serde(default = "default_true")]
    pub can_trade: bool,
}

impl Default for PlayerModerationSnapshot {
    fn default() -> Self {
        Self {
            violation_strikes: 0,
            in_debtors_prison: false,
            prison_reason: None,
            prison_tasks_remaining: 0,
            voice_muted_by_parent: false,
            chat_mode: default_chat_mode(),
            suspended: false,
            can_build: true,
            can_trade: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccount {
    pub username: String,
    #[serde(default)]
    pub display_name: String,
    pub password: String,
    #[serde(default = "default_role")]
    pub role: String,
    #[serde(default)]
    pub token_epoch: u64,
    #[serde(default)]
    pub avatar: AvatarAppearance,
    #[serde(default)]
    pub coins: i64,
    #[serde(default)]
    pub inventory: HashMap<String, u32>,
    #[serde(default)]
    pub cloneable_items: HashSet<String>,
    #[serde(default)]
    pub violation_strikes: u32,
    #[serde(default)]
    pub in_debtors_prison: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prison_reason: Option<String>,
    #[serde(default)]
    pub prison_tasks_remaining: u32,
    #[serde(default)]
    pub voice_muted_by_parent: bool,
    #[serde(default = "default_chat_mode")]
    pub chat_mode: String,
    #[serde(default)]
    pub suspended: bool,
    #[serde(default = "default_true")]
    pub can_build: bool,
    #[serde(default = "default_true")]
    pub can_trade: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_pos: Option<SavedPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_pos_saved_at: Option<u64>,
}

fn default_role() -> String {
    "friend".into()
}

pub fn default_chat_mode() -> String {
    "normal".into()
}

pub fn normalize_chat_mode(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "quick_only" | "quick" | "safe" => "quick_only".into(),
        "muted" | "off" | "disabled" => "muted".into(),
        _ => "normal".into(),
    }
}

impl UserAccount {
    pub fn is_subadmin(&self) -> bool {
        self.role.eq_ignore_ascii_case("subadmin")
    }

    pub fn sanitized(&self) -> SanitizedUser {
        SanitizedUser {
            username: self.username.clone(),
            display_name: if self.display_name.is_empty() {
                self.username.clone()
            } else {
                self.display_name.clone()
            },
            role: if self.role.is_empty() {
                "friend".into()
            } else {
                self.role.clone()
            },
            avatar: self.avatar.clone(),
            moderation: self.moderation_snapshot(),
        }
    }

    pub fn economy_snapshot(&self) -> PlayerEconomySnapshot {
        PlayerEconomySnapshot {
            coins: self.coins,
            inventory: self.inventory.clone(),
            cloneable_items: self.cloneable_items.clone(),
        }
    }

    pub fn moderation_snapshot(&self) -> PlayerModerationSnapshot {
        PlayerModerationSnapshot {
            violation_strikes: self.violation_strikes,
            in_debtors_prison: self.in_debtors_prison,
            prison_reason: self.prison_reason.clone(),
            prison_tasks_remaining: self.prison_tasks_remaining,
            voice_muted_by_parent: self.voice_muted_by_parent,
            chat_mode: normalize_chat_mode(&self.chat_mode),
            suspended: self.suspended,
            can_build: self.can_build,
            can_trade: self.can_trade,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanitizedUser {
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub avatar: AvatarAppearance,
    #[serde(default)]
    pub moderation: PlayerModerationSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersConfigFile {
    #[serde(default = "default_world_name")]
    pub world_name: String,
    #[serde(default = "default_admin_password")]
    pub admin_password: String,
    #[serde(default = "default_session_epoch")]
    pub session_epoch: u64,
    #[serde(default = "default_voice_mode")]
    pub voice_default_mode: String,
    #[serde(default = "default_ice_servers")]
    pub ice_servers: Vec<IceServerConfig>,
    #[serde(default = "default_true")]
    pub chat_filter_enabled: bool,
    #[serde(default = "default_blocked_words")]
    pub blocked_words: Vec<String>,
    #[serde(default = "default_true")]
    pub global_voice_enabled: bool,
    #[serde(default = "default_chat_mode")]
    pub global_chat_mode: String,
    #[serde(default)]
    pub world_locked: bool,
    #[serde(default)]
    pub curfew_enabled: bool,
    #[serde(default = "default_curfew_start")]
    pub curfew_start: String,
    #[serde(default = "default_curfew_end")]
    pub curfew_end: String,
    #[serde(default = "default_profanity_fine_base")]
    pub profanity_fine_base: u64,
    #[serde(default = "default_debt_prison_threshold")]
    pub debt_prison_threshold: i64,
    #[serde(default = "default_prison_strike_threshold")]
    pub prison_strike_threshold: u32,
    #[serde(default = "default_true")]
    pub enforce_machine_signatures: bool,
    #[serde(default)]
    pub devices: Vec<MachineDeviceRecord>,
    #[serde(default)]
    pub users: Vec<UserAccount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineDeviceRecord {
    pub device_id: String,
    pub short_sig: String,
    pub label: String,
    #[serde(default)]
    pub os_family: String,
    #[serde(default)]
    pub gpu_summary: String,
    #[serde(default)]
    pub screen_summary: String,
    #[serde(default)]
    pub hw_exact_sig: String,
    #[serde(default)]
    pub hw_xbrowser_sig: String,
    #[serde(default)]
    pub net_os_sig: String,
    #[serde(default)]
    pub last_ip: String,
    #[serde(default)]
    pub associated_users: Vec<String>,
    #[serde(default)]
    pub last_username: String,
    pub first_seen_ms: u64,
    pub last_seen_ms: u64,
    #[serde(default)]
    pub trusted_shared: bool,
    #[serde(default)]
    pub banned: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEvasionEntry {
    pub id: String,
    pub device_id: String,
    pub short_sig: String,
    pub device_label: String,
    pub attempted_username: String,
    pub blocking_username: String,
    #[serde(default)]
    pub active_penalties: Vec<String>,
    pub match_method: String,
    #[serde(default)]
    pub peer_ip: String,
    pub ts: u64,
}

pub fn default_session_epoch() -> u64 {
    1
}

fn default_true() -> bool {
    true
}

pub fn default_curfew_start() -> String {
    "21:00".into()
}

pub fn default_curfew_end() -> String {
    "07:00".into()
}

pub fn default_profanity_fine_base() -> u64 {
    15
}

pub fn default_debt_prison_threshold() -> i64 {
    -50
}

pub fn default_prison_strike_threshold() -> u32 {
    3
}

pub fn default_blocked_words() -> Vec<String> {
    vec![
        "fuck",
        "fucking",
        "fucker",
        "motherfucker",
        "shit",
        "bullshit",
        "dipshit",
        "bitch",
        "asshole",
        "jackass",
        "dumbass",
        "bastard",
        "cunt",
        "dick",
        "cock",
        "pussy",
        "whore",
        "slut",
        "twat",
        "prick",
        "wanker",
        "nigger",
        "nigga",
        "faggot",
        "fag",
        "retard",
        "kys",
        "stfu",
        "gtfo",
        "kill yourself",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub fn default_world_name() -> String {
    "Ella's Starlight Glam Plaza".into()
}
fn default_admin_password() -> String {
    "changeme-parent-admin".into()
}
fn default_voice_mode() -> String {
    "party".into()
}
pub fn default_ice_servers() -> Vec<IceServerConfig> {
    vec![IceServerConfig {
        urls: vec![
            "stun:stun.l.google.com:19302".into(),
            "stun:stun1.l.google.com:19302".into(),
        ],
        username: None,
        credential: None,
    }]
}

pub const ALLOWED_BLOCK_KINDS: &[&str] = &[
    "block",
    "stairs",
    "door",
    "double_door",
    "window",
    "kitchen_counter",
    "kitchen_sink",
    "kitchen_stove",
    "kitchen_fridge",
    "dining_table",
    "chair",
    "bath_tub",
    "bath_vanity",
    "bath_toilet",
    "bed",
    "nightstand_lamp",
    "wardrobe",
    "rug",
    "loot_decor",
];

pub fn default_block_kind() -> String {
    "block".into()
}

pub fn is_valid_block_kind(kind: &str) -> bool {
    ALLOWED_BLOCK_KINDS.contains(&kind)
}

pub fn is_two_tall_block_kind(kind: &str) -> bool {
    matches!(kind, "door" | "double_door" | "kitchen_fridge" | "wardrobe")
}

pub fn is_toggleable_block_kind(kind: &str) -> bool {
    matches!(kind, "door" | "double_door" | "window")
}

/// Deterministic horizontal partner cell `(dx, dz)` for a 2-block-wide `double_door`
/// based on its 90° rotation (`rot % 4`).
pub fn double_door_partner_offset(rot: u8) -> (i32, i32) {
    match rot % 4 {
        0 => (1, 0),
        1 => (0, -1),
        2 => (-1, 0),
        _ => (0, 1),
    }
}

/// All integer voxel cells `(x, y, z)` occupied by a placed block, furniture prop, or door.
pub fn block_occupied_cells(x: i32, y: i32, z: i32, kind: &str, rot: u8) -> Vec<(i32, i32, i32)> {
    let mut cells = vec![(x, y, z)];
    if is_two_tall_block_kind(kind) {
        cells.push((x, y + 1, z));
    }
    if kind == "double_door" {
        let (dx, dz) = double_door_partner_offset(rot);
        cells.push((x + dx, y, z + dz));
        cells.push((x + dx, y + 1, z + dz));
    }
    cells
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VoxelBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub color: String,
    #[serde(default = "default_role")]
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placed_at: Option<u64>,
    #[serde(default = "default_block_kind")]
    pub kind: String,
    #[serde(default)]
    pub rot: u8,
    #[serde(default)]
    pub open: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loot_item_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlocksDataFile {
    pub version: u64,
    pub blocks: Vec<VoxelBlock>,
}

pub fn default_starter_blocks() -> Vec<VoxelBlock> {
    let mk = |x: i32, y: i32, z: i32, color: &str| VoxelBlock {
        x,
        y,
        z,
        color: color.into(),
        author: "ella".into(),
        placed_at: None,
        kind: default_block_kind(),
        rot: 0,
        open: false,
        loot_item_id: None,
    };
    vec![
        mk(16, 1, -6, "#f472b6"),
        mk(16, 2, -6, "#fbbf24"),
        mk(16, 3, -6, "#34d399"),
        mk(16, 4, -6, "#38bdf8"),
        mk(17, 4, -6, "#a78bfa"),
        mk(18, 4, -6, "#f472b6"),
        mk(19, 4, -6, "#fbbf24"),
        mk(20, 4, -6, "#38bdf8"),
        mk(20, 3, -6, "#34d399"),
        mk(20, 2, -6, "#fbbf24"),
        mk(20, 1, -6, "#f472b6"),
    ]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRuntimeState {
    pub id: String,
    pub user: SanitizedUser,
    #[serde(default)]
    pub avatar_hash: String,
    #[serde(default)]
    pub zone: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ry: f32,
    pub anim: String,
    pub emote: Option<String>,
    pub muted: bool,
    pub speaking: bool,
    pub last_seen: u64,
    #[serde(skip)]
    pub last_seq: u32,
    #[serde(skip)]
    pub last_client_time_ms: u32,
    #[serde(skip)]
    pub conn_epoch: u32,
    #[serde(skip)]
    pub ws_connected: bool,
    #[serde(skip)]
    pub udp_fastlane_active: bool,
    #[serde(skip)]
    pub last_udp_seen_ms: u64,
    #[serde(skip)]
    pub disconnected_at_ms: Option<u64>,
    #[serde(skip)]
    pub speed_boost_until_ms: u64,
    #[serde(skip)]
    pub jump_boost_until_ms: u64,
    #[serde(skip)]
    pub server_voice_muted: bool,
    #[serde(skip)]
    pub device_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub text: String,
    pub is_system: bool,
    pub is_trophy_win: bool,
    pub ts: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedChatEntry {
    pub id: String,
    pub sender_id: String,
    pub username: String,
    pub display_name: String,
    pub text: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_word: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strike_number: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fine_coins: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance_after: Option<i64>,
    #[serde(default)]
    pub sentenced_to_prison: bool,
    pub ts: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunwayScoreEntry {
    pub total: u32,
    pub count: usize,
    pub avg: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunwayLeaderEntry {
    pub username: String,
    pub votes: usize,
    pub total_stars: u32,
    pub avg_stars: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunwayPublicSummary {
    pub theme: String,
    pub themes: Vec<String>,
    pub updated_at: u64,
    pub scores: HashMap<String, RunwayScoreEntry>,
    pub leaderboard: Vec<RunwayLeaderEntry>,
}

#[derive(Clone, Debug)]
pub struct RunwayState {
    pub theme: String,
    pub updated_at: u64,
    pub votes_by_target: HashMap<String, HashMap<String, u8>>,
}

impl Default for RunwayState {
    fn default() -> Self {
        Self {
            theme: RUNWAY_THEMES[0].to_string(),
            updated_at: now_ms(),
            votes_by_target: HashMap::new(),
        }
    }
}

impl RunwayState {
    pub fn summary(&self) -> RunwayPublicSummary {
        let mut scores = HashMap::new();
        let mut leaderboard = Vec::new();
        for (target, voter_map) in &self.votes_by_target {
            if voter_map.is_empty() {
                continue;
            }
            let total: u32 = voter_map.values().map(|&s| u32::from(s)).sum();
            let count = voter_map.len();
            let avg = ((f64::from(total) / count as f64) * 100.0).round() / 100.0;
            scores.insert(target.clone(), RunwayScoreEntry { total, count, avg });
            leaderboard.push(RunwayLeaderEntry {
                username: target.clone(),
                votes: count,
                total_stars: total,
                avg_stars: avg,
            });
        }
        leaderboard.sort_by(|a, b| {
            b.avg_stars
                .partial_cmp(&a.avg_stars)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.votes.cmp(&a.votes))
        });
        RunwayPublicSummary {
            theme: self.theme.clone(),
            themes: RUNWAY_THEMES.iter().map(|s| (*s).to_string()).collect(),
            updated_at: self.updated_at,
            scores,
            leaderboard,
        }
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn male_oblivion_jack_tech49_avatar_patch_and_legacy_defaults() {
        // Legacy saved user JSON without bodyType must deserialize cleanly as "female"
        let legacy: AvatarAppearance = serde_json::from_value(json!({
            "characterPreset": "katey",
            "hairStyle": "katey_long"
        }))
        .expect("legacy avatar json");
        assert_eq!(legacy.body_type, "female");

        let mut jack = AvatarAppearance::default();
        let base_hash = jack.short_hash();
        jack.apply_patch(&json!({
            "characterPreset": "jack_tech49",
            "skinColor": "#e5b887",
            "eyeColor": "#0ea5e9",
            "hairStyle": "tech_crop",
            "hairColor": "#3b2314",
            "topStyle": "tech_jacket",
            "topColor": "#e2e8f0",
            "bottomStyle": "tech_pants",
            "bottomColor": "#64748b",
            "shoeStyle": "flight_boots",
            "shoeColor": "#334155",
            "hat": "recon_cap",
            "hatColor": "#334155",
            "accessory": "aviators",
            "accessoryColor": "#38bdf8",
            "fabricFinish": "matte",
            "face": "hero"
        }));

        assert_eq!(jack.body_type, "male");
        assert_eq!(jack.character_preset, "jack_tech49");
        assert_eq!(jack.hair_style, "tech_crop");
        assert_eq!(jack.top_style, "tech_jacket");
        assert_eq!(jack.bottom_style, "tech_pants");
        assert_eq!(jack.shoe_style, "flight_boots");
        assert_eq!(jack.hat, "recon_cap");
        assert_eq!(jack.accessory, "aviators");
        assert_eq!(jack.face, "hero");
        assert_ne!(jack.short_hash(), base_hash);

        // Switching only bodyType must alter short_hash so remote 20Hz peers resync immediately
        let male_hash = jack.short_hash();
        jack.apply_patch(&json!({ "bodyType": "female" }));
        assert_eq!(jack.body_type, "female");
        assert_ne!(jack.short_hash(), male_hash);
    }
}

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
                "crystal_blade",
                "starlight_wand",
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
pub struct PlayerCompanionsSnapshot {
    pub owned_pets: Vec<String>,
    pub equipped_pet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pet_nickname: Option<String>,
    pub owned_mounts: Vec<String>,
    pub equipped_mount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_mount: Option<String>,
    pub first_boss_defeated: bool,
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

pub fn default_owned_pets() -> HashSet<String> {
    HashSet::from(["cloud_puppy".to_string()])
}

pub fn default_equipped_pet() -> Option<String> {
    Some("cloud_puppy".to_string())
}

pub fn default_owned_mounts() -> HashSet<String> {
    HashSet::from(["starlight_board".to_string()])
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
    #[serde(default = "default_owned_pets")]
    pub owned_pets: HashSet<String>,
    #[serde(default = "default_equipped_pet")]
    pub equipped_pet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pet_nickname: Option<String>,
    #[serde(default = "default_owned_mounts")]
    pub owned_mounts: HashSet<String>,
    #[serde(default)]
    pub equipped_mount: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_mount: Option<String>,
    #[serde(default)]
    pub first_boss_defeated: bool,
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
    pub fn new_basic(
        username: impl Into<String>,
        display_name: impl Into<String>,
        password: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            display_name: display_name.into(),
            password: password.into(),
            role: role.into(),
            token_epoch: 0,
            avatar: AvatarAppearance::default(),
            coins: 0,
            inventory: HashMap::new(),
            cloneable_items: HashSet::new(),
            owned_pets: default_owned_pets(),
            equipped_pet: default_equipped_pet(),
            pet_nickname: None,
            owned_mounts: default_owned_mounts(),
            equipped_mount: None,
            preferred_mount: None,
            first_boss_defeated: false,
            violation_strikes: 0,
            in_debtors_prison: false,
            prison_reason: None,
            prison_tasks_remaining: 0,
            voice_muted_by_parent: false,
            chat_mode: default_chat_mode(),
            suspended: false,
            can_build: true,
            can_trade: true,
            last_pos: None,
            last_pos_saved_at: None,
        }
    }

    pub fn ensure_starter_companions(&mut self) {
        self.owned_pets.insert("cloud_puppy".to_string());
        self.owned_mounts.insert("starlight_board".to_string());
    }

    pub fn best_owned_mount(&self) -> String {
        if let Some(ref pref) = self.preferred_mount {
            if pref == "starlight_board" || self.owned_mounts.contains(pref) {
                return pref.clone();
            }
        }
        if self.owned_mounts.contains("comet_surfer") {
            "comet_surfer".to_string()
        } else if self.owned_mounts.contains("sunset_skates") {
            "sunset_skates".to_string()
        } else {
            "starlight_board".to_string()
        }
    }

    pub fn is_subadmin(&self) -> bool {
        self.role.eq_ignore_ascii_case("subadmin")
    }

    pub fn is_host(&self) -> bool {
        self.role.eq_ignore_ascii_case("host")
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
            equipped_pet: self.equipped_pet.clone(),
            pet_nickname: self.pet_nickname.clone(),
            equipped_mount: self.equipped_mount.clone(),
        }
    }

    pub fn companions_snapshot(&self) -> PlayerCompanionsSnapshot {
        let mut pets: Vec<String> = self.owned_pets.iter().cloned().collect();
        if !pets.iter().any(|p| p == "cloud_puppy") {
            pets.push("cloud_puppy".to_string());
        }
        pets.sort();
        let mut mounts: Vec<String> = self.owned_mounts.iter().cloned().collect();
        if !mounts.iter().any(|m| m == "starlight_board") {
            mounts.push("starlight_board".to_string());
        }
        mounts.sort();
        PlayerCompanionsSnapshot {
            owned_pets: pets,
            equipped_pet: self.equipped_pet.clone(),
            pet_nickname: self.pet_nickname.clone(),
            owned_mounts: mounts,
            equipped_mount: self.equipped_mount.clone(),
            preferred_mount: Some(self.best_owned_mount()),
            first_boss_defeated: self.first_boss_defeated,
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
    #[serde(default)]
    pub equipped_pet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pet_nickname: Option<String>,
    #[serde(default)]
    pub equipped_mount: Option<String>,
}

impl SanitizedUser {
    /// Deterministic 16-hex-character digest covering avatar outfit + 3D companion/mount state.
    pub fn visual_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(self.avatar.short_hash().as_bytes());
        h.update(b"|");
        if let Some(p) = &self.equipped_pet {
            h.update(p.as_bytes());
        }
        h.update(b"|");
        if let Some(n) = &self.pet_nickname {
            h.update(n.as_bytes());
        }
        h.update(b"|");
        if let Some(m) = &self.equipped_mount {
            h.update(m.as_bytes());
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
    "sofa",
    "bath_tub",
    "bath_vanity",
    "bath_toilet",
    "bed",
    "nightstand_lamp",
    "crystal_lamp",
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
    matches!(
        kind,
        "door" | "double_door" | "window" | "nightstand_lamp" | "crystal_lamp"
    )
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
    #[serde(default, rename = "prebuiltVillageV1Seeded")]
    pub prebuilt_village_v1_seeded: bool,
}

/// Three fully walkable, furnished Prebuilt Homes seeded on Plots #2, #3, and #5
/// (leaving Plots #1, #4, and #6 completely blank for player custom builds!):
///   1. 🌸 Rosewater Grand Villa at Sunset Plot #2 (`x = 82, z = -26`) — 2-story `9×9` villa
///      with `double_door` entrance, walkable `stairs` to 2nd floor + front balcony `door`,
///      full Kitchen, Bathroom, and upstairs Master Suite!
///   2. 🌊 Starlight Beach Cottage at Starlight Plot #3 (`x = 52, z = 20`) — airy `7×7` cottage
///      with front & garden walkable `door`s, `window`s, cozy bedroom, dining nook & rug!
///   3. 🌲 Whispering Pine Woodland Chalet at West Villa Plot #5 (`x = -64, z = -16`) — warm
///      `9×7` chalet with `double_door`, kitchen nook, dining table, bed & wardrobe!
pub fn prebuilt_village_blocks() -> Vec<VoxelBlock> {
    let mut out: Vec<VoxelBlock> = Vec::with_capacity(420);
    let mut occupied: HashSet<(i32, i32, i32)> = HashSet::with_capacity(500);

    let mut put = |x: i32, y: i32, z: i32, kind: &str, rot: u8, color: &str, open: bool| {
        let r = rot % 4;
        let cells = block_occupied_cells(x, y, z, kind, r);
        if cells.iter().any(|c| occupied.contains(c)) {
            return;
        }
        for c in cells {
            occupied.insert(c);
        }
        out.push(VoxelBlock {
            x,
            y,
            z,
            color: color.to_string(),
            author: "ella".to_string(),
            placed_at: None,
            kind: kind.to_string(),
            rot: r,
            open,
            loot_item_id: None,
        });
    };

    // ========================================================================
    // 1. 🌸 Rosewater Grand Villa — Sunset Plot #2 (center x=82, z=-26; 9x9)
    // ========================================================================
    let villa_wall = "#fdf2f8";
    let villa_trim = "#f472b6";
    let villa_floor = "#fbcfe8";

    // Doors & Windows placed FIRST so wall loops automatically skip their cells!
    // Front 2x2 French Double Door at South face (z = -22, x = 81..=82, y = 1..=2)
    put(81, 1, -22, "double_door", 0, villa_trim, false);
    // 1st-floor Windows
    put(79, 2, -22, "window", 0, "#ffffff", false);
    put(85, 2, -22, "window", 0, "#ffffff", false);
    put(78, 2, -26, "window", 1, "#ffffff", false);
    put(86, 2, -26, "window", 1, "#ffffff", false);
    put(82, 2, -30, "window", 0, "#ffffff", false);

    // Walkable 2nd-story staircase along East interior wall (x = 85, climbing -Z)
    put(85, 1, -24, "stairs", 0, villa_trim, false);
    put(85, 2, -25, "stairs", 0, villa_trim, false);
    put(85, 3, -26, "stairs", 0, villa_trim, false);
    put(85, 4, -27, "stairs", 0, villa_trim, false);

    // 1st-floor exterior walls (x in 78..=86, z in -30..=-22, y in 1..=3)
    for y in 1..=3 {
        let col = if y == 1 { villa_trim } else { villa_wall };
        for x in 78..=86 {
            put(x, y, -22, "block", 0, col, false);
            put(x, y, -30, "block", 0, col, false);
        }
        for z in -29..=-23 {
            put(78, y, z, "block", 0, col, false);
            put(86, y, z, "block", 0, col, false);
        }
    }

    // 2nd-story floor slab (y = 4) with stairwell opening at x=85, z=-26..=-24
    for x in 78..=86 {
        for z in -30..=-22 {
            if x == 85 && (-26..=-24).contains(&z) {
                continue;
            }
            let is_edge = x == 78 || x == 86 || z == -30 || z == -22;
            put(
                x,
                4,
                z,
                "block",
                0,
                if is_edge { villa_trim } else { villa_floor },
                false,
            );
        }
    }
    // 2nd-story front sun balcony (z = -21..=-20, x = 80..=84) + low railing (y = 5)
    for x in 80..=84 {
        put(x, 4, -21, "block", 0, villa_floor, false);
        put(x, 4, -20, "block", 0, villa_trim, false);
        put(x, 5, -20, "block", 0, "#ffffff", false);
    }
    put(80, 5, -21, "block", 0, "#ffffff", false);
    put(84, 5, -21, "block", 0, "#ffffff", false);

    // Upstairs Balcony Door & 2nd-floor Windows
    put(82, 5, -22, "door", 0, villa_trim, false);
    put(80, 6, -22, "window", 0, "#ffffff", false);
    put(84, 6, -22, "window", 0, "#ffffff", false);
    put(78, 6, -26, "window", 1, "#ffffff", false);
    put(86, 6, -26, "window", 1, "#ffffff", false);
    put(82, 6, -30, "window", 0, "#ffffff", false);

    // 2nd-floor exterior walls (y in 5..=7) & flat crown roof (y = 8)
    for y in 5..=7 {
        let col = if y == 7 { villa_trim } else { villa_wall };
        for x in 78..=86 {
            put(x, y, -22, "block", 0, col, false);
            put(x, y, -30, "block", 0, col, false);
        }
        for z in -29..=-23 {
            put(78, y, z, "block", 0, col, false);
            put(86, y, z, "block", 0, col, false);
        }
    }
    for x in 78..=86 {
        for z in -30..=-22 {
            let is_edge = x == 78 || x == 86 || z == -30 || z == -22;
            put(
                x,
                8,
                z,
                "block",
                0,
                if is_edge { villa_trim } else { "#fce7f3" },
                false,
            );
        }
    }

    // Villa 1st-Floor Interior Furnishings (Kitchen, Dining, Lounge, Bath & Welcome Rug)
    put(82, 1, -25, "rug", 0, "#f472b6", false);
    put(83, 1, -24, "sofa", 3, villa_trim, false);
    put(84, 1, -23, "crystal_lamp", 0, "#f472b6", false);
    put(79, 1, -29, "kitchen_fridge", 1, "#ffffff", false);
    put(79, 1, -28, "kitchen_counter", 1, "#fbcfe8", false);
    put(79, 1, -27, "kitchen_sink", 1, "#bae6fd", false);
    put(79, 1, -26, "kitchen_stove", 1, "#e2e8f0", false);
    put(81, 1, -27, "dining_table", 0, "#ffffff", false);
    put(81, 1, -26, "chair", 2, villa_trim, false);
    put(84, 1, -29, "bath_tub", 0, "#bae6fd", false);
    put(83, 1, -29, "bath_vanity", 0, "#fbcfe8", false);

    // Villa 2nd-Floor Master Suite Furnishings (y = 5 sits flush on y = 4 floor!)
    put(82, 5, -26, "rug", 0, "#c084fc", false);
    put(80, 5, -28, "bed", 0, villa_trim, false);
    put(79, 5, -29, "nightstand_lamp", 0, "#fde047", false);
    put(79, 5, -24, "wardrobe", 1, "#fbcfe8", false);

    // ========================================================================
    // 2. 🌊 Starlight Beach Cottage — Starlight Plot #3 (center x=52, z=20; 7x7)
    // ========================================================================
    let beach_wall = "#f0f9ff";
    let beach_trim = "#38bdf8";

    // Front Door (North face z = 17) & Garden Patio Door (South face z = 23)
    put(52, 1, 17, "door", 0, beach_trim, false);
    put(52, 1, 23, "door", 0, beach_trim, false);
    put(50, 2, 17, "window", 0, "#ffffff", false);
    put(54, 2, 17, "window", 0, "#ffffff", false);
    put(49, 2, 20, "window", 1, "#ffffff", false);
    put(55, 2, 20, "window", 1, "#ffffff", false);

    for y in 1..=3 {
        let col = if y == 1 { "#bae6fd" } else { beach_wall };
        for x in 49..=55 {
            put(x, y, 17, "block", 0, col, false);
            put(x, y, 23, "block", 0, col, false);
        }
        for z in 18..=22 {
            put(49, y, z, "block", 0, col, false);
            put(55, y, z, "block", 0, col, false);
        }
    }
    // Coastal pitched roof (y = 4 overhang + y = 5 sky-blue ridge)
    for x in 49..=55 {
        for z in 17..=23 {
            let is_edge = x == 49 || x == 55 || z == 17 || z == 23;
            put(
                x,
                4,
                z,
                "block",
                0,
                if is_edge { beach_trim } else { "#7dd3fc" },
                false,
            );
        }
    }
    for x in 50..=54 {
        for z in 18..=22 {
            put(x, 5, z, "block", 0, beach_trim, false);
        }
    }

    // Cottage Cozy Interior
    put(52, 1, 20, "rug", 0, "#38bdf8", false);
    put(50, 1, 19, "sofa", 1, "#7dd3fc", false);
    put(54, 1, 18, "crystal_lamp", 0, "#38bdf8", false);
    put(50, 1, 21, "bed", 1, "#38bdf8", false);
    put(50, 1, 22, "nightstand_lamp", 1, "#fde047", false);
    put(54, 1, 19, "dining_table", 0, "#ffffff", false);
    put(54, 1, 20, "chair", 2, "#38bdf8", false);
    put(54, 1, 22, "kitchen_counter", 3, "#bae6fd", false);

    // ========================================================================
    // 3. 🌲 Whispering Pine Woodland Chalet — West Plot #5 (center x=-64, z=-16; 9x7)
    // ========================================================================
    let chalet_wall = "#fef3c7";
    let chalet_wood = "#d97706";
    let chalet_roof = "#10b981";

    // Grand Double Door on South face (z = -13, x = -65..=-64)
    put(-65, 1, -13, "double_door", 0, chalet_wood, false);
    put(-67, 2, -13, "window", 0, chalet_wood, false);
    put(-61, 2, -13, "window", 0, chalet_wood, false);
    put(-68, 2, -16, "window", 1, chalet_wood, false);
    put(-60, 2, -16, "window", 1, chalet_wood, false);
    put(-64, 2, -19, "window", 0, chalet_wood, false);

    for y in 1..=3 {
        let col = if y == 1 { chalet_wood } else { chalet_wall };
        for x in -68..=-60 {
            put(x, y, -13, "block", 0, col, false);
            put(x, y, -19, "block", 0, col, false);
        }
        for z in -18..=-14 {
            put(-68, y, z, "block", 0, col, false);
            put(-60, y, z, "block", 0, col, false);
        }
    }
    for x in -68..=-60 {
        for z in -19..=-13 {
            let is_edge = x == -68 || x == -60 || z == -19 || z == -13;
            put(
                x,
                4,
                z,
                "block",
                0,
                if is_edge { chalet_wood } else { chalet_roof },
                false,
            );
        }
    }
    for x in -67..=-61 {
        for z in -18..=-14 {
            put(x, 5, z, "block", 0, "#34d399", false);
        }
    }

    // Chalet Interior Furnishings
    put(-64, 1, -16, "rug", 0, "#fbbf24", false);
    put(-63, 1, -15, "sofa", 2, "#34d399", false);
    put(-67, 1, -14, "crystal_lamp", 0, "#fbbf24", false);
    put(-67, 1, -18, "kitchen_fridge", 0, "#f8fafc", false);
    put(-66, 1, -18, "kitchen_stove", 0, "#cbd5e1", false);
    put(-65, 1, -18, "kitchen_sink", 0, "#bae6fd", false);
    put(-66, 1, -15, "dining_table", 0, chalet_wood, false);
    put(-65, 1, -15, "chair", 3, "#34d399", false);
    put(-61, 1, -17, "bed", 3, "#34d399", false);
    put(-61, 1, -18, "nightstand_lamp", 3, "#fde047", false);
    put(-61, 1, -14, "wardrobe", 2, chalet_wood, false);
    put(-63, 1, -18, "bath_toilet", 0, "#ffffff", false);

    out
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
    let mut blocks = vec![
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
    ];
    blocks.extend(prebuilt_village_blocks());
    blocks
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
pub struct FashionShowContestant {
    pub username: String,
    pub display_name: String,
    pub ready: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FashionPodiumEntry {
    pub place: u8,
    pub username: String,
    pub display_name: String,
    pub avg_stars: f64,
    pub votes: usize,
    pub prize_coins: u64,
    pub ai_judged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FashionShowPublicSummary {
    pub show_id: u64,
    pub phase: String, // "idle" | "dress_up" | "catwalk" | "podium"
    pub theme: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_by: Option<String>,
    pub phase_started_at: u64,
    pub phase_ends_at: u64,
    pub contestants: Vec<FashionShowContestant>,
    pub spotlight_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spotlight_username: Option<String>,
    pub podium_winners: Vec<FashionPodiumEntry>,
}

#[derive(Clone, Debug)]
pub struct FashionShowState {
    pub show_id: u64,
    pub phase: String,
    pub started_by: Option<String>,
    pub phase_started_at: u64,
    pub phase_ends_at: u64,
    pub contestants: Vec<FashionShowContestant>,
    pub spotlight_index: usize,
    pub podium_winners: Vec<FashionPodiumEntry>,
}

impl Default for FashionShowState {
    fn default() -> Self {
        Self {
            show_id: 0,
            phase: "idle".to_string(),
            started_by: None,
            phase_started_at: 0,
            phase_ends_at: 0,
            contestants: Vec::new(),
            spotlight_index: 0,
            podium_winners: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunwayPublicSummary {
    pub theme: String,
    pub themes: Vec<String>,
    pub updated_at: u64,
    pub scores: HashMap<String, RunwayScoreEntry>,
    pub leaderboard: Vec<RunwayLeaderEntry>,
    pub show: FashionShowPublicSummary,
}

#[derive(Clone, Debug)]
pub struct RunwayState {
    pub theme: String,
    pub updated_at: u64,
    pub votes_by_target: HashMap<String, HashMap<String, u8>>,
    pub show: FashionShowState,
}

impl Default for RunwayState {
    fn default() -> Self {
        Self {
            theme: RUNWAY_THEMES[0].to_string(),
            updated_at: now_ms(),
            votes_by_target: HashMap::new(),
            show: FashionShowState::default(),
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
        let spotlight_username = if self.show.phase == "catwalk" {
            self.show
                .contestants
                .get(self.show.spotlight_index)
                .map(|c| c.username.clone())
        } else {
            None
        };
        RunwayPublicSummary {
            theme: self.theme.clone(),
            themes: RUNWAY_THEMES.iter().map(|s| (*s).to_string()).collect(),
            updated_at: self.updated_at,
            scores,
            leaderboard,
            show: FashionShowPublicSummary {
                show_id: self.show.show_id,
                phase: self.show.phase.clone(),
                theme: self.theme.clone(),
                started_by: self.show.started_by.clone(),
                phase_started_at: self.show.phase_started_at,
                phase_ends_at: self.show.phase_ends_at,
                contestants: self.show.contestants.clone(),
                spotlight_index: self.show.spotlight_index,
                spotlight_username,
                podium_winners: self.show.podium_winners.clone(),
            },
        }
    }
}

// ============================================================================
// Claimable Neighborhood Homes, Mailboxes & Doorbells
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct HomePlotDef {
    pub id: &'static str,
    pub default_name: &'static str,
    pub prebuilt: bool,
    pub center_x: i32,
    pub center_z: i32,
    pub half_extent: i32,
    pub mailbox_pos: [f32; 3],
    pub mailbox_rot_y: f32,
}

pub const HOME_PLOT_DEFS: &[HomePlotDef] = &[
    HomePlotDef {
        id: "plot_1",
        default_name: "🏡 Sunset Plot #1",
        prebuilt: false,
        center_x: 52,
        center_z: -26,
        half_extent: 9,
        mailbox_pos: [52.0, 0.0, -16.2],
        mailbox_rot_y: 0.0,
    },
    HomePlotDef {
        id: "plot_2",
        default_name: "🌸 Rosewater Villa",
        prebuilt: true,
        center_x: 82,
        center_z: -26,
        half_extent: 9,
        mailbox_pos: [79.0, 0.0, -16.2],
        mailbox_rot_y: 0.0,
    },
    HomePlotDef {
        id: "plot_3",
        default_name: "🌊 Starlight Cottage",
        prebuilt: true,
        center_x: 52,
        center_z: 20,
        half_extent: 9,
        mailbox_pos: [49.5, 0.0, 10.2],
        mailbox_rot_y: std::f32::consts::PI,
    },
    HomePlotDef {
        id: "plot_4",
        default_name: "🏡 Starlight Plot #4",
        prebuilt: false,
        center_x: 82,
        center_z: 20,
        half_extent: 9,
        mailbox_pos: [82.0, 0.0, 10.2],
        mailbox_rot_y: std::f32::consts::PI,
    },
    HomePlotDef {
        id: "plot_5",
        default_name: "🌲 Woodland Chalet",
        prebuilt: true,
        center_x: -64,
        center_z: -16,
        half_extent: 10,
        mailbox_pos: [-61.5, 0.0, -5.2],
        mailbox_rot_y: 0.0,
    },
    HomePlotDef {
        id: "plot_6",
        default_name: "🏡 West Villa Plot #6",
        prebuilt: false,
        center_x: -64,
        center_z: 32,
        half_extent: 10,
        mailbox_pos: [-64.0, 0.0, 21.2],
        mailbox_rot_y: std::f32::consts::PI,
    },
];

pub fn find_home_plot_def(plot_id: &str) -> Option<&'static HomePlotDef> {
    HOME_PLOT_DEFS.iter().find(|p| p.id == plot_id)
}

pub fn find_home_plot_for_cell(x: i32, z: i32) -> Option<&'static HomePlotDef> {
    HOME_PLOT_DEFS.iter().find(|p| {
        x >= p.center_x - p.half_extent
            && x <= p.center_x + p.half_extent
            && z >= p.center_z - p.half_extent
            && z <= p.center_z + p.half_extent
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomePlotClaim {
    pub plot_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    #[serde(default)]
    pub co_builders: Vec<String>,
    #[serde(default)]
    pub door_locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_at_ms: Option<u64>,
}

impl HomePlotClaim {
    pub fn empty(plot_id: &str) -> Self {
        Self {
            plot_id: plot_id.to_string(),
            owner_username: None,
            owner_display_name: None,
            custom_title: None,
            co_builders: Vec::new(),
            door_locked: false,
            claimed_at_ms: None,
        }
    }

    pub fn is_authorized_builder(&self, username: &str, role: &str) -> bool {
        if role.eq_ignore_ascii_case("host") || role.eq_ignore_ascii_case("admin") {
            return true;
        }
        let Some(owner) = &self.owner_username else {
            return true;
        };
        if owner.eq_ignore_ascii_case(username) {
            return true;
        }
        self.co_builders
            .iter()
            .any(|u| u.eq_ignore_ascii_case(username))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomePlotsFile {
    #[serde(default = "default_session_epoch")]
    pub version: u64,
    #[serde(default)]
    pub plots: Vec<HomePlotClaim>,
}

impl Default for HomePlotsFile {
    fn default() -> Self {
        Self {
            version: 1,
            plots: HOME_PLOT_DEFS
                .iter()
                .map(|d| HomePlotClaim::empty(d.id))
                .collect(),
        }
    }
}

impl HomePlotsFile {
    pub fn normalize(&mut self) {
        for def in HOME_PLOT_DEFS {
            if !self.plots.iter().any(|p| p.plot_id == def.id) {
                self.plots.push(HomePlotClaim::empty(def.id));
            }
        }
        self.plots
            .retain(|p| find_home_plot_def(&p.plot_id).is_some());
    }

    pub fn public_views(&self) -> Vec<HomePlotPublicView> {
        HOME_PLOT_DEFS
            .iter()
            .map(|def| {
                let claim = self
                    .plots
                    .iter()
                    .find(|p| p.plot_id == def.id)
                    .cloned()
                    .unwrap_or_else(|| HomePlotClaim::empty(def.id));
                HomePlotPublicView {
                    plot_id: def.id.to_string(),
                    default_name: def.default_name.to_string(),
                    prebuilt: def.prebuilt,
                    center_x: def.center_x,
                    center_z: def.center_z,
                    half_extent: def.half_extent,
                    mailbox_pos: def.mailbox_pos,
                    mailbox_rot_y: def.mailbox_rot_y,
                    owner_username: claim.owner_username,
                    owner_display_name: claim.owner_display_name,
                    custom_title: claim.custom_title,
                    co_builders: claim.co_builders,
                    door_locked: claim.door_locked,
                    claimed_at_ms: claim.claimed_at_ms,
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomePlotPublicView {
    pub plot_id: String,
    pub default_name: String,
    pub prebuilt: bool,
    pub center_x: i32,
    pub center_z: i32,
    pub half_extent: i32,
    pub mailbox_pos: [f32; 3],
    pub mailbox_rot_y: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    pub co_builders: Vec<String>,
    pub door_locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed_at_ms: Option<u64>,
}

// ============================================================================
// Obby Speedrun Gates & Persistent Top-5 Best-Time Boards
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct ObbyCourseSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub start_pos: [f32; 3],
    pub start_radius: f32,
    pub finish_pos: [f32; 3],
    pub finish_radius: f32,
    pub min_elapsed_ms: u64,
    pub completion_coins: u64,
    pub record_bonus_coins: u64,
}

pub const OBBY_COURSES: &[ObbyCourseSpec] = &[
    ObbyCourseSpec {
        id: "easy",
        name: "🌈 Sky Obby",
        start_pos: [0.0, 0.4, -12.0],
        start_radius: 8.5,
        finish_pos: [6.5, 16.5, -20.0],
        finish_radius: 9.0,
        min_elapsed_ms: 2_800,
        completion_coins: 12,
        record_bonus_coins: 25,
    },
    ObbyCourseSpec {
        id: "hard",
        name: "🔥 Pro Obby Gauntlet",
        start_pos: [52.0, 0.4, 52.0],
        start_radius: 9.0,
        finish_pos: [72.6, 40.2, 69.2],
        finish_radius: 9.5,
        min_elapsed_ms: 5_000,
        completion_coins: 35,
        record_bonus_coins: 55,
    },
];

pub fn find_obby_course(course_id: &str) -> Option<&'static ObbyCourseSpec> {
    OBBY_COURSES
        .iter()
        .find(|c| c.id.eq_ignore_ascii_case(course_id.trim()))
}

pub fn format_obby_time_ms(elapsed_ms: u64) -> String {
    format!("{:.2}s", (elapsed_ms as f64) / 1000.0)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ObbyRecord {
    pub username: String,
    pub display_name: String,
    pub elapsed_ms: u64,
    pub time_formatted: String,
    pub recorded_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObbyLeaderboardsFile {
    #[serde(default = "default_session_epoch")]
    pub version: u64,
    #[serde(default)]
    pub easy: Vec<ObbyRecord>,
    #[serde(default)]
    pub hard: Vec<ObbyRecord>,
}

impl Default for ObbyLeaderboardsFile {
    fn default() -> Self {
        Self {
            version: 1,
            easy: Vec::new(),
            hard: Vec::new(),
        }
    }
}

impl ObbyLeaderboardsFile {
    /// Updates the Top-5 leaderboard for `course_id` (`"easy"` or `"hard"`), keeping each player's personal best.
    /// Returns `(rank_1_to_5_or_0, improved_board)`.
    pub fn record_finish(&mut self, course_id: &str, record: ObbyRecord) -> (usize, bool) {
        let board = if course_id == "hard" {
            &mut self.hard
        } else {
            &mut self.easy
        };
        let mut improved = false;
        if let Some(existing) = board
            .iter_mut()
            .find(|r| r.username.eq_ignore_ascii_case(&record.username))
        {
            if record.elapsed_ms < existing.elapsed_ms {
                *existing = record.clone();
                improved = true;
            } else {
                existing.display_name = record.display_name.clone();
            }
        } else if board.len() < 5
            || board
                .last()
                .map(|w| record.elapsed_ms < w.elapsed_ms)
                .unwrap_or(true)
        {
            board.push(record.clone());
            improved = true;
        }
        board.sort_by(|a, b| {
            a.elapsed_ms
                .cmp(&b.elapsed_ms)
                .then_with(|| a.recorded_at_ms.cmp(&b.recorded_at_ms))
        });
        if board.len() > 5 {
            board.truncate(5);
        }
        let rank = board
            .iter()
            .position(|r| r.username.eq_ignore_ascii_case(&record.username))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        if improved {
            self.version = self.version.saturating_add(1);
        }
        (rank, improved)
    }
}

// ============================================================================
// Dynamic 12-Minute Sky / Sunset / Aurora Cycle & Meteor Shower
// ============================================================================

pub const SKY_CYCLE_DURATION_MS: u64 = 720_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FallingStarNode {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub coins: u64,
    pub bonus_shard: bool,
    pub spawned_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyWeatherPublicState {
    pub mode: String,  // "auto" | "day" | "sunset" | "aurora_night"
    pub phase: String, // "day" | "sunset" | "aurora_night"
    pub progress: f32, // 0.0 .. 1.0 across the 12-min cycle
    pub cycle_duration_ms: u64,
    pub server_time_ms: u64,
    pub cycle_anchor_ms: u64,
    pub meteor_shower_active: bool,
    pub meteor_shower_until_ms: u64,
    pub falling_stars: Vec<FallingStarNode>,
}

#[derive(Clone, Debug)]
pub struct SkyWeatherState {
    pub mode: String,
    pub cycle_anchor_ms: u64,
    pub meteor_shower_until_ms: u64,
    pub falling_stars: Vec<FallingStarNode>,
    pub next_star_seq: u64,
    pub last_auto_meteor_cycle: Option<u64>,
}

impl Default for SkyWeatherState {
    fn default() -> Self {
        Self {
            mode: "auto".to_string(),
            cycle_anchor_ms: now_ms(),
            meteor_shower_until_ms: 0,
            falling_stars: Vec::new(),
            next_star_seq: 1,
            last_auto_meteor_cycle: None,
        }
    }
}

impl SkyWeatherState {
    pub fn compute_phase_and_progress(&self, now: u64) -> (&'static str, f32) {
        match self.mode.as_str() {
            "day" => ("day", 0.18),
            "sunset" => ("sunset", 0.52),
            "aurora_night" => ("aurora_night", 0.80),
            _ => {
                let elapsed = now.saturating_sub(self.cycle_anchor_ms);
                let prog =
                    ((elapsed % SKY_CYCLE_DURATION_MS) as f32) / (SKY_CYCLE_DURATION_MS as f32);
                let phase = if prog < 0.42 {
                    "day"
                } else if prog < 0.64 {
                    "sunset"
                } else {
                    "aurora_night"
                };
                (phase, (prog * 10_000.0).round() / 10_000.0)
            }
        }
    }

    pub fn snapshot(&self, now: u64) -> SkyWeatherPublicState {
        let (phase, progress) = self.compute_phase_and_progress(now);
        let active = now < self.meteor_shower_until_ms;
        SkyWeatherPublicState {
            mode: self.mode.clone(),
            phase: phase.to_string(),
            progress,
            cycle_duration_ms: SKY_CYCLE_DURATION_MS,
            server_time_ms: now,
            cycle_anchor_ms: self.cycle_anchor_ms,
            meteor_shower_active: active,
            meteor_shower_until_ms: if active {
                self.meteor_shower_until_ms
            } else {
                0
            },
            falling_stars: self.falling_stars.clone(),
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

    #[test]
    fn prebuilt_village_homes_have_valid_kinds_walkable_doors_and_zero_overlaps() {
        let homes = prebuilt_village_blocks();
        assert!(
            homes.len() >= 250 && homes.len() <= 800,
            "Expected 3 furnished prebuilt homes (~300-600 blocks), got {}",
            homes.len()
        );
        let mut seen = HashSet::new();
        let mut door_count = 0usize;
        let mut double_door_count = 0usize;
        let mut stairs_count = 0usize;
        for b in &homes {
            assert!(is_valid_block_kind(&b.kind), "Invalid kind: {}", b.kind);
            assert!(is_valid_hex_color(&b.color), "Invalid color: {}", b.color);
            assert!(
                (-100..=100).contains(&b.x)
                    && (1..=80).contains(&b.y)
                    && (-100..=100).contains(&b.z)
            );
            if b.kind == "door" {
                door_count += 1;
            } else if b.kind == "double_door" {
                double_door_count += 1;
            } else if b.kind == "stairs" {
                stairs_count += 1;
            }
            for cell in block_occupied_cells(b.x, b.y, b.z, &b.kind, b.rot) {
                assert!(
                    seen.insert(cell),
                    "Duplicate occupied voxel cell in prebuilt village: {cell:?}"
                );
            }
        }
        assert!(door_count >= 3, "Expected >=3 walkable single doors");
        assert!(double_door_count >= 2, "Expected >=2 walkable double doors");
        assert_eq!(
            stairs_count, 4,
            "Expected 4-step 2nd-story staircase in Rosewater Villa"
        );
    }
}

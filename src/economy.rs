//! Crystal Cavern Dungeon Entities, Authoritative Item Catalog & Scam-Proof P2P Trade Engine

use crate::models::{now_ms, AvatarAppearance, UserAccount};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MAX_ITEM_STACK: u32 = 999;
pub const MAX_TRADE_COINS: u64 = 1_000_000_000;
pub const TRADE_INVITE_TTL_MS: u64 = 60_000;
pub const TRADE_ACTIVE_IDLE_TTL_MS: u64 = 300_000;

pub const PLAZA_PORTAL_POS: (f32, f32, f32) = (18.0, 0.0, 16.0);
pub const PLAZA_RETURN_SPAWN_POS: (f32, f32, f32) = (14.2, 0.0, 13.2);
pub const DUNGEON_SPAWN_POS: (f32, f32, f32) = (0.0, 0.0, 22.0);
pub const DUNGEON_RETURN_PORTAL_POS: (f32, f32, f32) = (0.0, 0.0, 28.0);
pub const PRISON_SPAWN_POS: (f32, f32, f32) = (0.0, 0.0, 3.5);
pub const PRISON_RELEASE_PORTAL_POS: (f32, f32, f32) = (0.0, 0.0, 11.5);
pub const PRISON_WORK_REWARD_COINS: i64 = 5;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub rarity: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub cosmetic_slot: Option<&'static str>,
    pub cosmetic_value: Option<&'static str>,
    pub suggested_coins: u64,
}

pub const ITEM_CATALOG: &[ItemDef] = &[
    ItemDef {
        id: "star_shard",
        name: "Starlight Shard",
        category: "material",
        rarity: "common",
        icon: "✨",
        description: "Glowing crystal shard mined from Crystal Cavern veins.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 12,
    },
    ItemDef {
        id: "slime_gel",
        name: "Glowing Slime Gel",
        category: "material",
        rarity: "common",
        icon: "🧪",
        description: "Bouncy bioluminescent gel dropped by Shadow Slimes.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 15,
    },
    ItemDef {
        id: "moon_opal",
        name: "Moonstone Opal",
        category: "material",
        rarity: "uncommon",
        icon: "🔮",
        description: "Iridescent cavern gemstone prized by collectors.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 35,
    },
    ItemDef {
        id: "dungeon_key",
        name: "Skeleton Vault Key",
        category: "key",
        rarity: "rare",
        icon: "🗝️",
        description: "Unlocks Locked Golden Treasure Chests deep in the Crystal Cavern.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 65,
    },
    ItemDef {
        id: "crown_relic",
        name: "Ancient Crown Relic",
        category: "material",
        rarity: "epic",
        icon: "🏺",
        description: "Rare royal artifact unearthed from Golden Vault Chests.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 140,
    },
    ItemDef {
        id: "speed_potion",
        name: "Swiftness Fizz",
        category: "consumable",
        rarity: "uncommon",
        icon: "⚡",
        description: "Drink for +35% walk & sprint speed for 20 seconds!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 25,
    },
    ItemDef {
        id: "jump_potion",
        name: "Cloud Leap Elixir",
        category: "consumable",
        rarity: "uncommon",
        icon: "🪽",
        description: "Drink for +30% higher jumps for 20 seconds!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 25,
    },
    ItemDef {
        id: "prism_block",
        name: "Prism Sparkle Block",
        category: "consumable",
        rarity: "rare",
        icon: "💠",
        description: "Places a shimmering amethyst-prism block beneath you in Starlight Plaza!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 40,
    },
    ItemDef {
        id: "cosmetic_hat_shadow_hood",
        name: "Shadow Ninja Hood",
        category: "cosmetic",
        rarity: "rare",
        icon: "🥷",
        description: "Unlocks the Shadow Ninja Headband & Mask in Glam Closet while owned!",
        cosmetic_slot: Some("hat"),
        cosmetic_value: Some("ninja"),
        suggested_coins: 180,
    },
    ItemDef {
        id: "cosmetic_acc_star_lantern",
        name: "Starlight Cavern Lantern",
        category: "cosmetic",
        rarity: "rare",
        icon: "🏮",
        description: "Unlocks the glowing Starlight Cavern Lantern accessory while owned!",
        cosmetic_slot: Some("accessory"),
        cosmetic_value: Some("star_lantern"),
        suggested_coins: 190,
    },
    ItemDef {
        id: "cosmetic_hat_obsidian_crown",
        name: "Obsidian Gem Crown",
        category: "cosmetic",
        rarity: "epic",
        icon: "👑",
        description: "Unlocks the royal Obsidian & Amethyst Gem Crown in Glam Closet while owned!",
        cosmetic_slot: Some("hat"),
        cosmetic_value: Some("obsidian_crown"),
        suggested_coins: 320,
    },
    ItemDef {
        id: "cosmetic_acc_crystal_wings",
        name: "Crystal Prism Wings",
        category: "cosmetic",
        rarity: "epic",
        icon: "🦋",
        description: "Unlocks shimmering Crystal Prism Wings in Glam Closet while owned!",
        cosmetic_slot: Some("accessory"),
        cosmetic_value: Some("crystal_wings"),
        suggested_coins: 350,
    },
    ItemDef {
        id: "geode_lamp",
        name: "Amethyst Geode Lamp",
        category: "decor",
        rarity: "uncommon",
        icon: "🔮",
        description: "Glowing split amethyst geode lamp! Place it inside your house from Backpack.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 55,
    },
    ItemDef {
        id: "slime_lava_lamp",
        name: "Glow-Slime Terrarium",
        category: "decor",
        rarity: "uncommon",
        icon: "🧪",
        description: "Cute bioluminescent cavern slime in a brass-trimmed terrarium jar for your room!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 60,
    },
    ItemDef {
        id: "bat_plushie",
        name: "Crystal Bat Plushie",
        category: "decor",
        rarity: "rare",
        icon: "🦇",
        description: "Ultra-cuddly lavender Crystal Bat plushie dropped in the cavern — perfect for bedrooms!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 110,
    },
    ItemDef {
        id: "royal_chalice",
        name: "Sanctum Golden Chalice",
        category: "decor",
        rarity: "rare",
        icon: "🏆",
        description: "Ruby-and-starlight jeweled goblet for kitchen tables or trophy pedestals!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 145,
    },
    ItemDef {
        id: "dragon_egg_relic",
        name: "Prismatic Dragon Egg",
        category: "decor",
        rarity: "epic",
        icon: "🥚",
        description: "Shimmering dragon egg resting on a crimson velvet pillow from Golden Vault Chests!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 280,
    },
    ItemDef {
        id: "legendary_ray_gun",
        name: "Starlight Plasma Ray Gun",
        category: "weapon",
        rarity: "legendary",
        icon: "🔫",
        description: "FIRST LEGENDARY! Equip in right hand to zap Cavern monsters from afar (3 HP!) or display on a Home Pedestal!",
        cosmetic_slot: Some("accessory"),
        cosmetic_value: Some("ray_gun"),
        suggested_coins: 950,
    },
];

pub fn find_item_def(item_id: &str) -> Option<&'static ItemDef> {
    ITEM_CATALOG.iter().find(|it| it.id == item_id)
}

/// Returns the required inventory `item_id` if a cosmetic `slot` + `value` is dungeon-exclusive.
pub fn required_item_for_cosmetic(slot: &str, value: &str) -> Option<&'static str> {
    for it in ITEM_CATALOG {
        if it.cosmetic_slot == Some(slot) && it.cosmetic_value == Some(value) {
            return Some(it.id);
        }
    }
    None
}

pub fn player_owns_item(inventory: &HashMap<String, u32>, item_id: &str) -> bool {
    inventory.get(item_id).copied().unwrap_or(0) > 0
}

/// Ensures `avatar` does not wear any dungeon-exclusive cosmetic that `inventory` lacks.
/// Returns `true` if one or more slots were reset.
pub fn reconcile_equipped_cosmetics(
    avatar: &mut AvatarAppearance,
    inventory: &HashMap<String, u32>,
) -> bool {
    let mut changed = false;
    if let Some(req_hat_item) = required_item_for_cosmetic("hat", &avatar.hat) {
        if !player_owns_item(inventory, req_hat_item) {
            avatar.hat = "crown".into();
            changed = true;
        }
    }
    if let Some(req_acc_item) = required_item_for_cosmetic("accessory", &avatar.accessory) {
        if !player_owns_item(inventory, req_acc_item) {
            avatar.accessory = "none".into();
            changed = true;
        }
    }
    changed
}

pub fn add_inventory_item(inventory: &mut HashMap<String, u32>, item_id: &str, qty: u32) -> u32 {
    if qty == 0 || find_item_def(item_id).is_none() {
        return inventory.get(item_id).copied().unwrap_or(0);
    }
    let entry = inventory.entry(item_id.to_string()).or_insert(0);
    *entry = entry.saturating_add(qty).min(MAX_ITEM_STACK);
    *entry
}

pub fn remove_inventory_item(
    inventory: &mut HashMap<String, u32>,
    item_id: &str,
    qty: u32,
) -> bool {
    if qty == 0 {
        return true;
    }
    let current = inventory.get(item_id).copied().unwrap_or(0);
    if current < qty {
        return false;
    }
    let next = current - qty;
    if next == 0 {
        inventory.remove(item_id);
    } else {
        inventory.insert(item_id.to_string(), next);
    }
    true
}

// ============================================================================
// Crystal Cavern Dungeon Entities & Simulation
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DungeonNode {
    pub id: String,
    pub kind: String, // "crystal" | "wooden_chest" | "golden_chest"
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub available: bool,
    #[serde(skip)]
    pub cooldown_ms: u64,
    #[serde(skip)]
    pub ready_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DungeonMob {
    pub id: String,
    pub kind: String, // "shadow_slime" | "crystal_bat"
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ry: f32,
    pub hp: u8,
    pub max_hp: u8,
    pub alive: bool,
    #[serde(skip)]
    pub home_x: f32,
    #[serde(skip)]
    pub home_y: f32,
    #[serde(skip)]
    pub home_z: f32,
    #[serde(skip)]
    pub patrol_radius: f32,
    #[serde(skip)]
    pub phase: f32,
    #[serde(skip)]
    pub speed: f32,
    #[serde(skip)]
    pub respawn_at_ms: u64,
    #[serde(skip)]
    pub last_hit_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DungeonDrop {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub coins: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    pub qty: u32,
    pub label: String,
    pub rarity: String,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DungeonSnapshot {
    pub nodes: Vec<DungeonNode>,
    pub mobs: Vec<DungeonMob>,
    pub drops: Vec<DungeonDrop>,
    pub portal_pos: [f32; 3],
    pub return_portal_pos: [f32; 3],
}

pub struct DungeonState {
    pub nodes: Vec<DungeonNode>,
    pub mobs: Vec<DungeonMob>,
    pub drops: Vec<DungeonDrop>,
    next_drop_seq: u64,
}

impl Default for DungeonState {
    fn default() -> Self {
        Self::new()
    }
}

impl DungeonState {
    pub fn new() -> Self {
        let nodes = vec![
            DungeonNode {
                id: "crystal_1".into(),
                kind: "crystal".into(),
                label: "Starlight Crystal Vein".into(),
                x: -14.0,
                y: 0.0,
                z: 12.0,
                available: true,
                cooldown_ms: 16_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "crystal_2".into(),
                kind: "crystal".into(),
                label: "Amethyst Geode".into(),
                x: -22.0,
                y: 0.0,
                z: 2.0,
                available: true,
                cooldown_ms: 16_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "crystal_3".into(),
                kind: "crystal".into(),
                label: "Starlight Crystal Vein".into(),
                x: 14.0,
                y: 0.0,
                z: 12.0,
                available: true,
                cooldown_ms: 16_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "crystal_4".into(),
                kind: "crystal".into(),
                label: "Moonstone Cluster".into(),
                x: 22.0,
                y: 0.0,
                z: 2.0,
                available: true,
                cooldown_ms: 18_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "crystal_5".into(),
                kind: "crystal".into(),
                label: "Deep Prism Spire".into(),
                x: -16.0,
                y: 1.8,
                z: -16.0,
                available: true,
                cooldown_ms: 18_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "crystal_6".into(),
                kind: "crystal".into(),
                label: "Deep Prism Spire".into(),
                x: 16.0,
                y: 1.8,
                z: -16.0,
                available: true,
                cooldown_ms: 18_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "chest_wood_1".into(),
                kind: "wooden_chest".into(),
                label: "Explorer's Supply Chest".into(),
                x: -10.0,
                y: 0.0,
                z: -4.0,
                available: true,
                cooldown_ms: 24_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "chest_wood_2".into(),
                kind: "wooden_chest".into(),
                label: "Miner's Treasure Chest".into(),
                x: 10.0,
                y: 0.0,
                z: -4.0,
                available: true,
                cooldown_ms: 24_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "chest_gold_1".into(),
                kind: "golden_chest".into(),
                label: "Locked Golden Vault (Needs Key 🗝️)".into(),
                x: -18.0,
                y: 3.2,
                z: -26.0,
                available: true,
                cooldown_ms: 38_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "chest_gold_2".into(),
                kind: "golden_chest".into(),
                label: "Royal Sanctum Chest (Needs Key 🗝️)".into(),
                x: 0.0,
                y: 4.5,
                z: -30.0,
                available: true,
                cooldown_ms: 38_000,
                ready_at_ms: 0,
            },
        ];

        let mobs = vec![
            Self::make_mob(
                "slime_1",
                "shadow_slime",
                "Shadow Slime",
                0.0,
                0.0,
                8.0,
                4.2,
                0.0,
                0.9,
                2,
            ),
            Self::make_mob(
                "slime_2",
                "shadow_slime",
                "Shadow Slime",
                -12.0,
                0.0,
                4.0,
                3.8,
                1.3,
                1.0,
                2,
            ),
            Self::make_mob(
                "slime_3",
                "shadow_slime",
                "Shadow Slime",
                12.0,
                0.0,
                4.0,
                3.8,
                2.6,
                1.0,
                2,
            ),
            Self::make_mob(
                "slime_4",
                "shadow_slime",
                "Glowberry Slime",
                -8.0,
                0.0,
                -14.0,
                4.5,
                3.9,
                1.1,
                2,
            ),
            Self::make_mob(
                "slime_5",
                "shadow_slime",
                "Glowberry Slime",
                8.0,
                0.0,
                -14.0,
                4.5,
                5.1,
                1.1,
                2,
            ),
            Self::make_mob(
                "bat_1",
                "crystal_bat",
                "Crystal Bat",
                0.0,
                1.35,
                -8.0,
                5.2,
                0.7,
                1.35,
                3,
            ),
            Self::make_mob(
                "bat_2",
                "crystal_bat",
                "Crystal Bat",
                -14.0,
                1.35,
                -20.0,
                4.4,
                2.2,
                1.4,
                3,
            ),
            Self::make_mob(
                "bat_3",
                "crystal_bat",
                "Crystal Bat",
                14.0,
                1.35,
                -20.0,
                4.4,
                4.1,
                1.4,
                3,
            ),
        ];

        let mut st = Self {
            nodes,
            mobs,
            drops: Vec::new(),
            next_drop_seq: 1,
        };
        st.seed_ambient_coins(now_ms());
        st
    }

    #[allow(clippy::too_many_arguments)]
    fn make_mob(
        id: &str,
        kind: &str,
        name: &str,
        hx: f32,
        hy: f32,
        hz: f32,
        radius: f32,
        phase: f32,
        speed: f32,
        max_hp: u8,
    ) -> DungeonMob {
        DungeonMob {
            id: id.into(),
            kind: kind.into(),
            name: name.into(),
            x: hx,
            y: hy,
            z: hz,
            ry: 0.0,
            hp: max_hp,
            max_hp,
            alive: true,
            home_x: hx,
            home_y: hy,
            home_z: hz,
            patrol_radius: radius,
            phase,
            speed,
            respawn_at_ms: 0,
            last_hit_at_ms: 0,
        }
    }

    pub fn snapshot(&self) -> DungeonSnapshot {
        DungeonSnapshot {
            nodes: self.nodes.clone(),
            mobs: self.mobs.clone(),
            drops: self.drops.clone(),
            portal_pos: [PLAZA_PORTAL_POS.0, PLAZA_PORTAL_POS.1, PLAZA_PORTAL_POS.2],
            return_portal_pos: [
                DUNGEON_RETURN_PORTAL_POS.0,
                DUNGEON_RETURN_PORTAL_POS.1,
                DUNGEON_RETURN_PORTAL_POS.2,
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn spawn_drop(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        coins: u64,
        item_id: Option<&str>,
        qty: u32,
        now: u64,
    ) -> DungeonDrop {
        let seq = self.next_drop_seq;
        self.next_drop_seq = self.next_drop_seq.wrapping_add(1).max(1);
        let (label, rarity, valid_item) = if let Some(iid) = item_id.and_then(find_item_def) {
            let lbl = if coins > 0 {
                format!("{} {} x{} + {} 🪙", iid.icon, iid.name, qty.max(1), coins)
            } else {
                format!("{} {} x{}", iid.icon, iid.name, qty.max(1))
            };
            (lbl, iid.rarity.to_string(), Some(iid.id.to_string()))
        } else {
            (format!("🪙 {coins} Coins"), "common".to_string(), None)
        };

        let drop_entry = DungeonDrop {
            id: format!("drop_{seq}"),
            x: (x * 100.0).round() / 100.0,
            y: y.max(0.0),
            z: (z * 100.0).round() / 100.0,
            coins,
            item_id: valid_item,
            qty: if item_id.is_some() { qty.max(1) } else { 0 },
            label,
            rarity,
            expires_at_ms: now + 60_000,
        };
        if self.drops.len() >= 80 {
            self.drops.remove(0);
        }
        self.drops.push(drop_entry.clone());
        drop_entry
    }

    fn seed_ambient_coins(&mut self, now: u64) {
        let spots = [
            (-5.0, 0.0, 15.0, 8),
            (5.0, 0.0, 15.0, 8),
            (-16.0, 0.0, 7.0, 10),
            (16.0, 0.0, 7.0, 10),
            (0.0, 0.0, -2.0, 12),
            (0.0, 2.2, -18.0, 15),
        ];
        for (x, y, z, coins) in spots {
            let mut d = self.spawn_drop(x, y, z, coins, None, 0, now);
            d.expires_at_ms = now + 180_000;
            if let Some(last) = self.drops.last_mut() {
                last.expires_at_ms = d.expires_at_ms;
            }
        }
    }

    /// Advances dungeon patrol paths, respawns nodes/mobs, and refreshes ambient coins.
    /// Returns `(nodes_or_drops_changed, mobs_snapshot)`.
    pub fn tick(&mut self, now: u64) -> (bool, Vec<DungeonMob>) {
        let mut world_changed = false;

        for node in &mut self.nodes {
            if !node.available && now >= node.ready_at_ms {
                node.available = true;
                node.ready_at_ms = 0;
                world_changed = true;
            }
        }

        let before_drops = self.drops.len();
        self.drops.retain(|d| now < d.expires_at_ms);
        if self.drops.len() != before_drops {
            world_changed = true;
        }
        let ambient_coin_count = self.drops.iter().filter(|d| d.item_id.is_none()).count();
        if ambient_coin_count < 3 {
            let mut rng = rand::thread_rng();
            let rx = rng.gen_range(-18.0f32..18.0f32);
            let rz = rng.gen_range(-14.0f32..16.0f32);
            let c = rng.gen_range(6u64..=14u64);
            let _ = self.spawn_drop(rx, 0.0, rz, c, None, 0, now);
            if let Some(last) = self.drops.last_mut() {
                last.expires_at_ms = now + 180_000;
            }
            world_changed = true;
        }

        let t_sec = (now % 3_600_000) as f32 * 0.001;
        for mob in &mut self.mobs {
            if !mob.alive {
                if now >= mob.respawn_at_ms {
                    mob.alive = true;
                    mob.hp = mob.max_hp;
                    mob.x = mob.home_x;
                    mob.y = mob.home_y;
                    mob.z = mob.home_z;
                    world_changed = true;
                }
                continue;
            }
            let angle = t_sec * mob.speed + mob.phase;
            let nx = mob.home_x + angle.cos() * mob.patrol_radius;
            let nz = mob.home_z + (angle * 0.85).sin() * (mob.patrol_radius * 0.78);
            let dx = nx - mob.x;
            let dz = nz - mob.z;
            if dx.abs() + dz.abs() > 0.01 {
                mob.ry = dx.atan2(dz);
            }
            mob.x = (nx * 100.0).round() / 100.0;
            mob.z = (nz * 100.0).round() / 100.0;
            if mob.kind == "crystal_bat" {
                mob.y = ((mob.home_y + (angle * 2.2).sin() * 0.32) * 100.0).round() / 100.0;
            }
        }

        (world_changed, self.mobs.clone())
    }
}

pub fn roll_crystal_loot() -> (u64, Option<&'static str>, u32) {
    let mut rng = rand::thread_rng();
    let coins = rng.gen_range(10u64..=20u64);
    let roll: u32 = rng.gen_range(0..100);
    if roll < 42 {
        (coins, Some("star_shard"), rng.gen_range(1..=2))
    } else if roll < 66 {
        (coins, Some("moon_opal"), 1)
    } else if roll < 80 {
        (coins, Some("geode_lamp"), 1)
    } else if roll < 92 {
        (coins, Some("prism_block"), 1)
    } else {
        (coins, Some("dungeon_key"), 1)
    }
}

pub fn roll_mob_loot(kind: &str) -> (u64, Option<&'static str>, u32) {
    let mut rng = rand::thread_rng();
    let base_coins = if kind == "crystal_bat" {
        rng.gen_range(18u64..=32u64)
    } else {
        rng.gen_range(12u64..=24u64)
    };
    let roll: u32 = rng.gen_range(0..100);
    if roll < 26 {
        (base_coins, Some("slime_gel"), rng.gen_range(1..=2))
    } else if roll < 40 {
        (base_coins, Some("speed_potion"), 1)
    } else if roll < 52 {
        (base_coins, Some("jump_potion"), 1)
    } else if roll < 66 {
        (base_coins, Some("dungeon_key"), 1)
    } else if roll < 78 {
        if kind == "crystal_bat" {
            (base_coins, Some("bat_plushie"), 1)
        } else {
            (base_coins, Some("slime_lava_lamp"), 1)
        }
    } else if roll < 86 {
        (base_coins, Some("geode_lamp"), 1)
    } else if roll < 92 {
        (base_coins, Some("cosmetic_hat_shadow_hood"), 1)
    } else if roll < 96 {
        (base_coins, Some("cosmetic_acc_star_lantern"), 1)
    } else if roll < 98 {
        (base_coins, Some("royal_chalice"), 1)
    } else {
        (base_coins + 30, Some("legendary_ray_gun"), 1)
    }
}

pub fn roll_wooden_chest_loot() -> (u64, Option<&'static str>, u32) {
    let mut rng = rand::thread_rng();
    let coins = rng.gen_range(22u64..=40u64);
    let roll: u32 = rng.gen_range(0..100);
    if roll < 24 {
        (coins, Some("moon_opal"), rng.gen_range(1..=2))
    } else if roll < 46 {
        (coins, Some("dungeon_key"), 1)
    } else if roll < 60 {
        (coins, Some("speed_potion"), 2)
    } else if roll < 72 {
        (coins, Some("prism_block"), 2)
    } else if roll < 82 {
        (coins, Some("geode_lamp"), 1)
    } else if roll < 90 {
        (coins, Some("slime_lava_lamp"), 1)
    } else if roll < 96 {
        (coins, Some("bat_plushie"), 1)
    } else {
        (coins, Some("cosmetic_acc_star_lantern"), 1)
    }
}

pub fn roll_golden_chest_loot() -> (u64, Option<&'static str>, u32) {
    let mut rng = rand::thread_rng();
    let coins = rng.gen_range(55u64..=95u64);
    let roll: u32 = rng.gen_range(0..100);
    if roll < 18 {
        (coins, Some("crown_relic"), 1)
    } else if roll < 34 {
        (coins, Some("royal_chalice"), 1)
    } else if roll < 50 {
        (coins, Some("dragon_egg_relic"), 1)
    } else if roll < 66 {
        (coins, Some("cosmetic_hat_obsidian_crown"), 1)
    } else if roll < 82 {
        (coins, Some("cosmetic_acc_crystal_wings"), 1)
    } else if roll < 90 {
        (coins, Some("bat_plushie"), 1)
    } else {
        (coins + 50, Some("legendary_ray_gun"), 1)
    }
}

// ============================================================================
// Escalating Profanity Fines & Debtor's Prison Community Service Yard
// ============================================================================

/// Computes an escalating fine for repeat chat violations:
/// Strike 1 -> `base` (default 15 🪙)
/// Strike 2 -> `(base * 7 / 3).max(base + 5)` (default 35 🪙)
/// Strike 3+ -> `base * 5 * 2^(strike - 3)` capped at 500 🪙 (default 75 -> 150 -> 300 -> 500 🪙)
pub fn calculate_escalating_fine(base_fine: u64, strike_number: u32) -> u64 {
    if base_fine == 0 {
        return 0;
    }
    let strike = strike_number.max(1);
    let raw = match strike {
        1 => base_fine,
        2 => (base_fine.saturating_mul(7) / 3).max(base_fine.saturating_add(5)),
        s => {
            let exp = (s - 3).min(6);
            base_fine.saturating_mul(5).saturating_mul(1u64 << exp)
        }
    };
    raw.min(500)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrisonWorkNode {
    pub id: String,
    pub kind: String, // "boulder" | "sweep" | "lantern"
    pub name: String,
    pub icon: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub available: bool,
    pub ready_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrisonSnapshot {
    pub nodes: Vec<PrisonWorkNode>,
    pub release_portal_pos: [f32; 3],
    pub reward_coins: i64,
}

#[derive(Clone, Debug)]
pub struct PrisonState {
    pub nodes: Vec<PrisonWorkNode>,
}

impl Default for PrisonState {
    fn default() -> Self {
        Self::new()
    }
}

impl PrisonState {
    pub fn new() -> Self {
        let nodes = vec![
            Self::make_node(
                "prison_rock_1",
                "boulder",
                "West Quarry Boulder",
                "🪨",
                -7.5,
                0.0,
                -4.5,
            ),
            Self::make_node(
                "prison_rock_2",
                "boulder",
                "East Quarry Boulder",
                "🪨",
                7.5,
                0.0,
                -4.5,
            ),
            Self::make_node(
                "prison_sweep_1",
                "sweep",
                "North Courtyard Dust Pile",
                "🧹",
                -5.0,
                0.0,
                -9.0,
            ),
            Self::make_node(
                "prison_sweep_2",
                "sweep",
                "South Courtyard Dust Pile",
                "🧹",
                5.0,
                0.0,
                -9.0,
            ),
            Self::make_node(
                "prison_lantern_1",
                "lantern",
                "West Tarnished Lantern",
                "🧽",
                -8.2,
                0.0,
                3.5,
            ),
            Self::make_node(
                "prison_lantern_2",
                "lantern",
                "East Tarnished Lantern",
                "🧽",
                8.2,
                0.0,
                3.5,
            ),
        ];
        Self { nodes }
    }

    fn make_node(
        id: &str,
        kind: &str,
        name: &str,
        icon: &str,
        x: f32,
        y: f32,
        z: f32,
    ) -> PrisonWorkNode {
        PrisonWorkNode {
            id: id.into(),
            kind: kind.into(),
            name: name.into(),
            icon: icon.into(),
            x,
            y,
            z,
            available: true,
            ready_at_ms: 0,
        }
    }

    pub fn snapshot(&self) -> PrisonSnapshot {
        PrisonSnapshot {
            nodes: self.nodes.clone(),
            release_portal_pos: [
                PRISON_RELEASE_PORTAL_POS.0,
                PRISON_RELEASE_PORTAL_POS.1,
                PRISON_RELEASE_PORTAL_POS.2,
            ],
            reward_coins: PRISON_WORK_REWARD_COINS,
        }
    }

    /// Re-enables completed community-service stations after their 6-second recharge timer.
    pub fn tick(&mut self, now: u64) -> bool {
        let mut changed = false;
        for node in &mut self.nodes {
            if !node.available && now >= node.ready_at_ms {
                node.available = true;
                node.ready_at_ms = 0;
                changed = true;
            }
        }
        changed
    }
}

// ============================================================================
// Live Two-Sided Player-to-Player (P2P) Trade Engine
// ============================================================================

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TradeSideOffer {
    pub coins: u64,
    pub items: HashMap<String, u32>,
    pub locked: bool,
    pub confirmed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSession {
    pub trade_id: String,
    pub status: String, // "pending" | "active"
    pub initiator_id: String,
    pub initiator_username: String,
    pub initiator_display_name: String,
    pub partner_id: String,
    pub partner_username: String,
    pub partner_display_name: String,
    pub initiator_offer: TradeSideOffer,
    pub partner_offer: TradeSideOffer,
    pub offer_revision: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_note: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

pub fn sanitize_and_validate_offer(
    coins: u64,
    raw_items: &HashMap<String, u32>,
    owned_coins: i64,
    owned_inv: &HashMap<String, u32>,
) -> Result<(u64, HashMap<String, u32>), String> {
    if coins > MAX_TRADE_COINS {
        return Err("Offered coin amount exceeds maximum trade limit.".into());
    }
    if coins > 0 && owned_coins <= 0 {
        return Err(format!(
            "You cannot offer coins while your balance is {owned_coins} 🪙 (in debt)!"
        ));
    }
    if coins > 0 && (coins as i64) > owned_coins {
        return Err(format!(
            "Not enough coins! You offered {coins} 🪙 but only have {owned_coins} 🪙."
        ));
    }
    if raw_items.len() > ITEM_CATALOG.len() {
        return Err("Too many distinct item stacks in trade offer.".into());
    }

    let mut clean_items = HashMap::new();
    for (raw_id, &qty) in raw_items {
        if qty == 0 {
            continue;
        }
        let id = raw_id.trim();
        let Some(def) = find_item_def(id) else {
            return Err(format!("Unknown item '{id}'."));
        };
        if qty > MAX_ITEM_STACK {
            return Err(format!(
                "Cannot offer more than {MAX_ITEM_STACK} of {}.",
                def.name
            ));
        }
        let owned = owned_inv.get(def.id).copied().unwrap_or(0);
        if qty > owned {
            return Err(format!(
                "You only have {owned}x {} in your backpack.",
                def.name
            ));
        }
        clean_items.insert(def.id.to_string(), qty);
    }

    Ok((coins, clean_items))
}

impl TradeSession {
    pub fn involves_peer(&self, peer_id: &str) -> bool {
        self.initiator_id == peer_id || self.partner_id == peer_id
    }

    pub fn other_peer_id<'a>(&'a self, peer_id: &str) -> Option<&'a str> {
        if self.initiator_id == peer_id {
            Some(&self.partner_id)
        } else if self.partner_id == peer_id {
            Some(&self.initiator_id)
        } else {
            None
        }
    }

    pub fn apply_validated_offer(
        &mut self,
        self_id: &str,
        coins: u64,
        items: HashMap<String, u32>,
        now: u64,
    ) -> Result<bool, String> {
        if self.status != "active" {
            return Err("Trade has not been accepted yet.".into());
        }
        let was_locked_or_confirmed = self.initiator_offer.locked
            || self.partner_offer.locked
            || self.initiator_offer.confirmed
            || self.partner_offer.confirmed;

        let side = if self.initiator_id == self_id {
            &mut self.initiator_offer
        } else if self.partner_id == self_id {
            &mut self.partner_offer
        } else {
            return Err("You are not part of this trade.".into());
        };

        let changed = side.coins != coins || side.items != items;
        side.coins = coins;
        side.items = items;

        if changed {
            // Anti-bait-and-switch invariant: any modification resets BOTH sides' lock & confirm!
            self.initiator_offer.locked = false;
            self.initiator_offer.confirmed = false;
            self.partner_offer.locked = false;
            self.partner_offer.confirmed = false;
            self.offer_revision = self.offer_revision.wrapping_add(1);
            self.status_note = if was_locked_or_confirmed {
                Some("Offer changed — locks were reset so both friends can review safely!".into())
            } else {
                None
            };
        }
        self.updated_at_ms = now;
        Ok(changed)
    }

    pub fn set_locked(&mut self, self_id: &str, locked: bool, now: u64) -> Result<(), String> {
        if self.status != "active" {
            return Err("Trade has not been accepted yet.".into());
        }
        if self.initiator_id == self_id {
            self.initiator_offer.locked = locked;
        } else if self.partner_id == self_id {
            self.partner_offer.locked = locked;
        } else {
            return Err("You are not part of this trade.".into());
        }

        if !locked {
            self.initiator_offer.confirmed = false;
            self.partner_offer.confirmed = false;
            self.status_note = Some("Offer unlocked for editing.".into());
        } else if self.initiator_offer.locked && self.partner_offer.locked {
            self.status_note =
                Some("Both offers locked! Review carefully and click Confirm Trade.".into());
        } else {
            self.status_note = Some("Waiting for both friends to lock their offers...".into());
        }
        self.updated_at_ms = now;
        Ok(())
    }

    /// Marks `self_id` as confirmed (only valid when both sides are locked).
    /// Returns `Ok(true)` if BOTH sides have now confirmed and the swap should execute!
    pub fn mark_confirmed(&mut self, self_id: &str, now: u64) -> Result<bool, String> {
        if self.status != "active" {
            return Err("Trade is not active.".into());
        }
        if !self.initiator_offer.locked || !self.partner_offer.locked {
            return Err("Both friends must Lock their offers before confirming!".into());
        }
        if self.initiator_id == self_id {
            self.initiator_offer.confirmed = true;
        } else if self.partner_id == self_id {
            self.partner_offer.confirmed = true;
        } else {
            return Err("You are not part of this trade.".into());
        }
        self.updated_at_ms = now;
        let both = self.initiator_offer.confirmed && self.partner_offer.confirmed;
        if !both {
            self.status_note =
                Some("Waiting for the other friend to click Confirm Trade...".into());
        }
        Ok(both)
    }
}

/// Atomically validates balances & stack capacities and swaps coins + items between `user_a` and `user_b`.
/// Returns `(avatar_a_unequipped, avatar_b_unequipped)` if either player traded away an equipped cosmetic.
pub fn execute_atomic_trade_swap(
    user_a: &mut UserAccount,
    offer_a: &TradeSideOffer,
    user_b: &mut UserAccount,
    offer_b: &TradeSideOffer,
) -> Result<(bool, bool), String> {
    if user_a.username.eq_ignore_ascii_case(&user_b.username) {
        return Err("Cannot trade between identical accounts.".into());
    }

    // 1. Validate sufficient balances on both sides before mutating anything
    let (_, clean_a_items) = sanitize_and_validate_offer(
        offer_a.coins,
        &offer_a.items,
        user_a.coins,
        &user_a.inventory,
    )?;
    let (_, clean_b_items) = sanitize_and_validate_offer(
        offer_b.coins,
        &offer_b.items,
        user_b.coins,
        &user_b.inventory,
    )?;

    // 2. Check coin & item stack overflow after deducting outgoing and adding incoming
    let offer_a_i64 = i64::try_from(offer_a.coins)
        .map_err(|_| "Offered coin amount exceeds maximum trade limit.".to_string())?;
    let offer_b_i64 = i64::try_from(offer_b.coins)
        .map_err(|_| "Offered coin amount exceeds maximum trade limit.".to_string())?;
    let next_a_coins = user_a
        .coins
        .checked_sub(offer_a_i64)
        .and_then(|v| v.checked_add(offer_b_i64))
        .ok_or_else(|| "Coin balance overflow for initiator.".to_string())?;
    let next_b_coins = user_b
        .coins
        .checked_sub(offer_b_i64)
        .and_then(|v| v.checked_add(offer_a_i64))
        .ok_or_else(|| "Coin balance overflow for partner.".to_string())?;

    let mut next_a_inv = user_a.inventory.clone();
    let mut next_b_inv = user_b.inventory.clone();

    for (item_id, &qty) in &clean_a_items {
        if !remove_inventory_item(&mut next_a_inv, item_id, qty) {
            return Err(format!("Initiator no longer has {qty}x {item_id}."));
        }
    }
    for (item_id, &qty) in &clean_b_items {
        if !remove_inventory_item(&mut next_b_inv, item_id, qty) {
            return Err(format!("Partner no longer has {qty}x {item_id}."));
        }
    }

    for (item_id, &qty) in &clean_b_items {
        let cur = next_a_inv.get(item_id).copied().unwrap_or(0);
        if cur.saturating_add(qty) > MAX_ITEM_STACK {
            return Err(format!(
                "{}'s backpack stack for {item_id} is full (max {MAX_ITEM_STACK}).",
                user_a.display_name
            ));
        }
        add_inventory_item(&mut next_a_inv, item_id, qty);
    }
    for (item_id, &qty) in &clean_a_items {
        let cur = next_b_inv.get(item_id).copied().unwrap_or(0);
        if cur.saturating_add(qty) > MAX_ITEM_STACK {
            return Err(format!(
                "{}'s backpack stack for {item_id} is full (max {MAX_ITEM_STACK}).",
                user_b.display_name
            ));
        }
        add_inventory_item(&mut next_b_inv, item_id, qty);
    }

    // 3. Commit mutations atomically
    user_a.coins = next_a_coins;
    user_a.inventory = next_a_inv;
    user_b.coins = next_b_coins;
    user_b.inventory = next_b_inv;

    let av_a_changed = reconcile_equipped_cosmetics(&mut user_a.avatar, &user_a.inventory);
    let av_b_changed = reconcile_equipped_cosmetics(&mut user_b.avatar, &user_b.inventory);

    Ok((av_a_changed, av_b_changed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_user(username: &str, coins: i64) -> UserAccount {
        UserAccount {
            username: username.into(),
            display_name: username.into(),
            password: "pw".into(),
            role: "friend".into(),
            token_epoch: 0,
            avatar: AvatarAppearance::default(),
            coins,
            inventory: HashMap::new(),
            violation_strikes: 0,
            in_debtors_prison: false,
            prison_reason: None,
            prison_tasks_remaining: 0,
            voice_muted_by_parent: false,
            chat_mode: "normal".into(),
            suspended: false,
            can_build: true,
            can_trade: true,
            last_pos: None,
            last_pos_saved_at: None,
        }
    }

    #[test]
    fn escalating_fines_and_negative_balance_trade_guard() {
        assert_eq!(calculate_escalating_fine(15, 1), 15);
        assert_eq!(calculate_escalating_fine(15, 2), 35);
        assert_eq!(calculate_escalating_fine(15, 3), 75);
        assert_eq!(calculate_escalating_fine(15, 4), 150);
        assert_eq!(calculate_escalating_fine(15, 5), 300);
        assert_eq!(calculate_escalating_fine(15, 6), 500);

        let inv = HashMap::from([("star_shard".to_string(), 2)]);
        // In-debt player cannot offer positive coins
        assert!(sanitize_and_validate_offer(5, &inv, -35, &inv).is_err());
        // In-debt player CAN offer 0 coins if trading items (when outside prison & permitted)
        assert!(sanitize_and_validate_offer(0, &inv, -35, &inv).is_ok());
    }

    #[test]
    fn trade_offer_modification_resets_locks_and_confirmations() {
        let mut sess = TradeSession {
            trade_id: "trd_1".into(),
            status: "active".into(),
            initiator_id: "peer_ella".into(),
            initiator_username: "ella".into(),
            initiator_display_name: "Ella".into(),
            partner_id: "peer_maya".into(),
            partner_username: "maya".into(),
            partner_display_name: "Maya".into(),
            initiator_offer: TradeSideOffer {
                coins: 50,
                items: HashMap::new(),
                locked: true,
                confirmed: true,
            },
            partner_offer: TradeSideOffer {
                coins: 0,
                items: HashMap::from([("star_shard".to_string(), 2)]),
                locked: true,
                confirmed: false,
            },
            offer_revision: 1,
            status_note: None,
            created_at_ms: 100,
            updated_at_ms: 100,
        };

        // Ella tries to lower coins from 50 -> 5 after locking!
        let changed = sess
            .apply_validated_offer("peer_ella", 5, HashMap::new(), 200)
            .expect("valid edit");
        assert!(changed);
        assert!(!sess.initiator_offer.locked);
        assert!(!sess.initiator_offer.confirmed);
        assert!(!sess.partner_offer.locked);
        assert!(!sess.partner_offer.confirmed);
        assert_eq!(sess.offer_revision, 2);

        // Confirming before re-locking must fail
        assert!(sess.mark_confirmed("peer_maya", 210).is_err());
    }

    #[test]
    fn atomic_swap_transfers_assets_and_unequips_traded_exclusive_cosmetics() {
        let mut ella = dummy_user("ella", 100);
        add_inventory_item(&mut ella.inventory, "cosmetic_hat_obsidian_crown", 1);
        ella.avatar.hat = "obsidian_crown".into();

        let mut maya = dummy_user("maya", 40);
        add_inventory_item(&mut maya.inventory, "dungeon_key", 2);

        let offer_ella = TradeSideOffer {
            coins: 25,
            items: HashMap::from([("cosmetic_hat_obsidian_crown".to_string(), 1)]),
            locked: true,
            confirmed: true,
        };
        let offer_maya = TradeSideOffer {
            coins: 10,
            items: HashMap::from([("dungeon_key".to_string(), 1)]),
            locked: true,
            confirmed: true,
        };

        let (av_ella_changed, av_maya_changed) =
            execute_atomic_trade_swap(&mut ella, &offer_ella, &mut maya, &offer_maya)
                .expect("atomic swap succeeds");

        assert!(
            av_ella_changed,
            "Ella traded away her last Obsidian Crown so hat resets"
        );
        assert!(!av_maya_changed);
        assert_eq!(ella.avatar.hat, "crown");
        assert_eq!(ella.coins, 85); // 100 - 25 + 10
        assert_eq!(maya.coins, 55); // 40 - 10 + 25
        assert_eq!(ella.inventory.get("dungeon_key").copied(), Some(1));
        assert_eq!(ella.inventory.get("cosmetic_hat_obsidian_crown"), None);
        assert_eq!(
            maya.inventory.get("cosmetic_hat_obsidian_crown").copied(),
            Some(1)
        );
        assert_eq!(maya.inventory.get("dungeon_key").copied(), Some(1));
    }
}

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
pub const PLAZA_ARENA_PORTAL_POS: (f32, f32, f32) = (-22.0, 0.0, 28.0);
pub const PLAZA_ARENA_RETURN_SPAWN_POS: (f32, f32, f32) = (-18.0, 0.0, 25.0);
pub const ARENA_SPAWN_POS: (f32, f32, f32) = (0.0, 0.0, 25.0);
pub const ARENA_RETURN_PORTAL_POS: (f32, f32, f32) = (0.0, 0.0, 30.0);
pub const ARENA_MAX_HP: u8 = 100;
pub const ARENA_AEGIS_MAX_HP: u8 = 150;
pub const ARENA_BOSS_ALTAR_POS: (f32, f32, f32) = (0.0, 0.0, -24.5);
pub const CRAFTING_BENCH_POS: (f32, f32, f32) = (12.5, 0.0, 6.5);
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
    pub parent_exclusive: bool,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
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
        parent_exclusive: false,
    },
    ItemDef {
        id: "pet_egg",
        name: "Starlight Companion Egg",
        category: "egg",
        rarity: "rare",
        icon: "🥚",
        description: "Hatch in Pets & Boards to adopt a 3D Starlight Kitsune, Shadow Kitty, or Baby Crystal Dragon!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 120,
        parent_exclusive: false,
    },
    ItemDef {
        id: "crystal_blade",
        name: "Starlight Crystal Blade",
        category: "weapon",
        rarity: "epic",
        icon: "⚔️",
        description: "Crafted melee sword! Equip in right hand for 2 HP strikes + glowing crystal swing arcs!",
        cosmetic_slot: Some("accessory"),
        cosmetic_value: Some("crystal_blade"),
        suggested_coins: 260,
        parent_exclusive: false,
    },
    ItemDef {
        id: "starlight_wand",
        name: "Astral Starlight Wand",
        category: "weapon",
        rarity: "epic",
        icon: "🪄",
        description: "Crafted magic wand! Casts ranged star-bolts (2 HP, 16m range) + mends +8 HP per hit in Shadow Rift!",
        cosmetic_slot: Some("accessory"),
        cosmetic_value: Some("starlight_wand"),
        suggested_coins: 290,
        parent_exclusive: false,
    },
    ItemDef {
        id: "aegis_heart_charm",
        name: "Aegis Heart Charm",
        category: "charm",
        rarity: "epic",
        icon: "💖",
        description: "Passive Guardian Relic! Boosts your Max HP from 100 to 150 inside Shadow Rift while in your Backpack!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 340,
        parent_exclusive: false,
    },
    ItemDef {
        id: "starlight_potion",
        name: "Starlight Restoration Elixir",
        category: "consumable",
        rarity: "rare",
        icon: "🧪",
        description: "Drink to restore +75 HP in Shadow Rift AND gain +35% Speed & +30% Jump for 30 seconds!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 55,
        parent_exclusive: false,
    },
    ItemDef {
        id: "rift_beacon",
        name: "Obsidian Rift Beacon",
        category: "consumable",
        rarity: "epic",
        icon: "🔮",
        description: "Summons the Obsidian Dragon Boss at the North Island Altar immediately (bypasses cooldown + boosts Pet Egg luck)!",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 175,
        parent_exclusive: false,
    },
    ItemDef {
        id: "aurora_crystal_lamp",
        name: "Aurora Prism Lamp",
        category: "decor",
        rarity: "epic",
        icon: "🌟",
        description: "Rare crafted floating Aurora orb lamp! Place it inside your home from Backpack.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 210,
        parent_exclusive: false,
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
        parent_exclusive: false,
    },
    ItemDef {
        id: "parent_golden_trophy",
        name: "Parent's Golden Star Trophy",
        category: "decor",
        rarity: "mythic",
        icon: "🏅",
        description: "Parent-Exclusive Award! Never spawns in the world — gifted only by a Parent in Admin.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 0,
        parent_exclusive: true,
    },
    ItemDef {
        id: "parent_guardian_crown",
        name: "Guardian's Ruby Crown",
        category: "decor",
        rarity: "mythic",
        icon: "👸",
        description: "Parent-Exclusive Heirloom Trophy! Never spawns in-game — gifted only by a Parent.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 0,
        parent_exclusive: true,
    },
    ItemDef {
        id: "parent_starlight_heirloom",
        name: "Royal Starlight Heirloom",
        category: "material",
        rarity: "mythic",
        icon: "💠",
        description: "Parent-Exclusive Collector Gem! Never spawns in-game — gifted only by a Parent.",
        cosmetic_slot: None,
        cosmetic_value: None,
        suggested_coins: 0,
        parent_exclusive: true,
    },
];

// ============================================================================
// Adoptable 3D Pets, Plaza Hoverboards / Mounts & Starlight Crafting Recipes
// ============================================================================

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetDef {
    pub id: &'static str,
    pub name: &'static str,
    pub rarity: &'static str,
    pub icon: &'static str,
    pub glb_node: &'static str,
    pub perk_label: &'static str,
    pub coin_price: u64,
    pub egg_weight: u32,
    pub magnet_bonus_m: f32,
    pub coin_bonus_pct: u32,
    pub bonus_combat_dmg: u8,
}

pub const PET_CATALOG: &[PetDef] = &[
    PetDef {
        id: "cloud_puppy",
        name: "Cloud Puppy",
        rarity: "common",
        icon: "🐶",
        glb_node: "Env_Pet_CloudPuppy",
        perk_label: "Free Starter Bestie • +2.0m Coin Magnet & +5% Coins",
        coin_price: 0,
        egg_weight: 0,
        magnet_bonus_m: 2.0,
        coin_bonus_pct: 5,
        bonus_combat_dmg: 0,
    },
    PetDef {
        id: "starlight_kitsune",
        name: "Starlight Kitsune",
        rarity: "rare",
        icon: "🦊",
        glb_node: "Env_Pet_StarlightKitsune",
        perk_label: "Triple-Tail Luck • +20% Bonus Coins & +1.5m Magnet",
        coin_price: 160,
        egg_weight: 46,
        magnet_bonus_m: 1.5,
        coin_bonus_pct: 20,
        bonus_combat_dmg: 0,
    },
    PetDef {
        id: "shadow_kitty",
        name: "Shadow Rift Kitty",
        rarity: "epic",
        icon: "🐱",
        glb_node: "Env_Pet_ShadowKitty",
        perk_label: "Night Prowler • +12% Sprint Agility & +10% Coins",
        coin_price: 220,
        egg_weight: 34,
        magnet_bonus_m: 1.5,
        coin_bonus_pct: 10,
        bonus_combat_dmg: 0,
    },
    PetDef {
        id: "baby_crystal_dragon",
        name: "Baby Crystal Dragon",
        rarity: "legendary",
        icon: "🐉",
        glb_node: "Env_Pet_BabyDragon_Body",
        perk_label: "Rift Guardian • +1 Combat Damage, +15% Coins & +2.5m Magnet",
        coin_price: 420,
        egg_weight: 20,
        magnet_bonus_m: 2.5,
        coin_bonus_pct: 15,
        bonus_combat_dmg: 1,
    },
];

pub fn find_pet_def(pet_id: &str) -> Option<&'static PetDef> {
    PET_CATALOG.iter().find(|p| p.id == pet_id)
}

pub fn roll_hatched_pet(owned_pets: &std::collections::HashSet<String>) -> &'static PetDef {
    let mut rng = rand::thread_rng();
    // Strongly favor unowned pets first so hatching eggs feels rewarding and fair!
    let unowned: Vec<&'static PetDef> = PET_CATALOG
        .iter()
        .filter(|p| p.egg_weight > 0 && !owned_pets.contains(p.id))
        .collect();
    let pool: Vec<&'static PetDef> = if !unowned.is_empty() {
        unowned
    } else {
        PET_CATALOG.iter().filter(|p| p.egg_weight > 0).collect()
    };
    let total_weight: u32 = pool.iter().map(|p| p.egg_weight).sum();
    if total_weight == 0 {
        return &PET_CATALOG[1];
    }
    let mut pick = rng.gen_range(0..total_weight);
    for p in &pool {
        if pick < p.egg_weight {
            return p;
        }
        pick -= p.egg_weight;
    }
    pool[0]
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MountDef {
    pub id: &'static str,
    pub name: &'static str,
    pub rarity: &'static str,
    pub icon: &'static str,
    pub glb_node: &'static str,
    pub speed_mult: f32,
    pub coin_price: u64,
    pub description: &'static str,
}

pub const MOUNT_CATALOG: &[MountDef] = &[
    MountDef {
        id: "starlight_board",
        name: "Starlight Hoverboard",
        rarity: "common",
        icon: "🛹",
        glb_node: "Env_Mount_Hoverboard",
        speed_mult: 1.58,
        coin_price: 0,
        description: "Free starter hoverboard! Press X in Starlight Plaza to cruise fast!",
    },
    MountDef {
        id: "sunset_skates",
        name: "Cotton-Candy Starlight Skates",
        rarity: "rare",
        icon: "🛼",
        glb_node: "Env_Mount_StarlightSkates",
        speed_mult: 1.66,
        coin_price: 150,
        description: "Twin winged hover-skates with pastel ribbon trails across the Plaza!",
    },
    MountDef {
        id: "comet_surfer",
        name: "Golden Comet Surfer",
        rarity: "epic",
        icon: "☄️",
        glb_node: "Env_Mount_CometSurfer",
        speed_mult: 1.75,
        coin_price: 320,
        description: "Championship gold-and-cyan surfboard glider with starlight wake!",
    },
];

pub fn find_mount_def(mount_id: &str) -> Option<&'static MountDef> {
    MOUNT_CATALOG.iter().find(|m| m.id == mount_id)
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingIngredient {
    pub item_id: &'static str,
    pub qty: u32,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CraftingRecipeDef {
    pub id: &'static str,
    pub output_item_id: &'static str,
    pub output_qty: u32,
    pub coin_cost: u64,
    pub ingredients: &'static [CraftingIngredient],
    pub summary: &'static str,
}

pub const CRAFTING_RECIPES: &[CraftingRecipeDef] = &[
    CraftingRecipeDef {
        id: "craft_crystal_blade",
        output_item_id: "crystal_blade",
        output_qty: 1,
        coin_cost: 40,
        ingredients: &[
            CraftingIngredient {
                item_id: "star_shard",
                qty: 3,
            },
            CraftingIngredient {
                item_id: "slime_gel",
                qty: 2,
            },
        ],
        summary: "Forge a 2-DMG right-hand Starlight Crystal Blade with 3D arc swing trails!",
    },
    CraftingRecipeDef {
        id: "craft_starlight_wand",
        output_item_id: "starlight_wand",
        output_qty: 1,
        coin_cost: 50,
        ingredients: &[
            CraftingIngredient {
                item_id: "star_shard",
                qty: 3,
            },
            CraftingIngredient {
                item_id: "moon_opal",
                qty: 2,
            },
        ],
        summary:
            "Channel ranged 16m star-bolts (2 DMG) that also mend +8 HP per hit in Shadow Rift!",
    },
    CraftingRecipeDef {
        id: "craft_aegis_charm",
        output_item_id: "aegis_heart_charm",
        output_qty: 1,
        coin_cost: 80,
        ingredients: &[
            CraftingIngredient {
                item_id: "moon_opal",
                qty: 2,
            },
            CraftingIngredient {
                item_id: "crown_relic",
                qty: 1,
            },
        ],
        summary: "Raises your Shadow Rift Max HP from 100 -> 150 while carried in your Backpack!",
    },
    CraftingRecipeDef {
        id: "craft_starlight_potion",
        output_item_id: "starlight_potion",
        output_qty: 2,
        coin_cost: 15,
        ingredients: &[
            CraftingIngredient {
                item_id: "star_shard",
                qty: 1,
            },
            CraftingIngredient {
                item_id: "slime_gel",
                qty: 1,
            },
        ],
        summary:
            "Brews 2x Starlight Elixirs (+75 HP heal in Shadow Rift + 30s Speed & Jump boost)!",
    },
    CraftingRecipeDef {
        id: "craft_rift_beacon",
        output_item_id: "rift_beacon",
        output_qty: 1,
        coin_cost: 45,
        ingredients: &[
            CraftingIngredient {
                item_id: "star_shard",
                qty: 2,
            },
            CraftingIngredient {
                item_id: "moon_opal",
                qty: 1,
            },
        ],
        summary: "Instantly summons the Co-Op Obsidian Dragon Boss with boosted Pet Egg luck!",
    },
    CraftingRecipeDef {
        id: "craft_aurora_lamp",
        output_item_id: "aurora_crystal_lamp",
        output_qty: 1,
        coin_cost: 35,
        ingredients: &[
            CraftingIngredient {
                item_id: "star_shard",
                qty: 2,
            },
            CraftingIngredient {
                item_id: "moon_opal",
                qty: 1,
            },
        ],
        summary: "Crafts a floating luminous Aurora Prism Lamp for your home interior!",
    },
    CraftingRecipeDef {
        id: "craft_pet_egg",
        output_item_id: "pet_egg",
        output_qty: 1,
        coin_cost: 65,
        ingredients: &[
            CraftingIngredient {
                item_id: "slime_gel",
                qty: 2,
            },
            CraftingIngredient {
                item_id: "moon_opal",
                qty: 1,
            },
        ],
        summary: "Incubate a Hatchable Starlight Companion Egg to adopt a new 3D Pet!",
    },
];

pub fn find_crafting_recipe(recipe_id: &str) -> Option<&'static CraftingRecipeDef> {
    CRAFTING_RECIPES
        .iter()
        .find(|r| r.id == recipe_id || r.output_item_id == recipe_id)
}

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
    pub fn make_mob(
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
    if kind == "lava_golem" {
        let coins = rng.gen_range(48u64..=80u64);
        let roll: u32 = rng.gen_range(0..100);
        return if roll < 16 {
            (coins, Some("crown_relic"), 1)
        } else if roll < 30 {
            (coins, Some("dragon_egg_relic"), 1)
        } else if roll < 44 {
            (coins, Some("pet_egg"), 1)
        } else if roll < 58 {
            (coins, Some("royal_chalice"), 1)
        } else if roll < 72 {
            (coins, Some("cosmetic_hat_obsidian_crown"), 1)
        } else if roll < 86 {
            (coins, Some("cosmetic_acc_crystal_wings"), 1)
        } else {
            (coins + 45, Some("legendary_ray_gun"), 1)
        };
    }
    if kind == "void_wraith" {
        let coins = rng.gen_range(30u64..=50u64);
        let roll: u32 = rng.gen_range(0..100);
        return if roll < 22 {
            (coins, Some("dungeon_key"), 1)
        } else if roll < 40 {
            (coins, Some("moon_opal"), 2)
        } else if roll < 54 {
            (coins, Some("cosmetic_hat_shadow_hood"), 1)
        } else if roll < 68 {
            (coins, Some("cosmetic_acc_star_lantern"), 1)
        } else if roll < 80 {
            (coins, Some("pet_egg"), 1)
        } else if roll < 89 {
            (coins, Some("royal_chalice"), 1)
        } else if roll < 95 {
            (coins, Some("dragon_egg_relic"), 1)
        } else {
            (coins + 35, Some("legendary_ray_gun"), 1)
        };
    }
    if kind == "rift_spider" {
        let coins = rng.gen_range(22u64..=36u64);
        let roll: u32 = rng.gen_range(0..100);
        return if roll < 24 {
            (coins, Some("speed_potion"), 1)
        } else if roll < 46 {
            (coins, Some("jump_potion"), 1)
        } else if roll < 68 {
            (coins, Some("dungeon_key"), 1)
        } else if roll < 84 {
            (coins, Some("geode_lamp"), 1)
        } else if roll < 95 {
            (coins, Some("cosmetic_hat_shadow_hood"), 1)
        } else {
            (coins + 30, Some("legendary_ray_gun"), 1)
        };
    }
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
    if roll < 16 {
        (coins, Some("crown_relic"), 1)
    } else if roll < 30 {
        (coins, Some("pet_egg"), 1)
    } else if roll < 44 {
        (coins, Some("royal_chalice"), 1)
    } else if roll < 58 {
        (coins, Some("dragon_egg_relic"), 1)
    } else if roll < 72 {
        (coins, Some("cosmetic_hat_obsidian_crown"), 1)
    } else if roll < 86 {
        (coins, Some("cosmetic_acc_crystal_wings"), 1)
    } else if roll < 92 {
        (coins, Some("bat_plushie"), 1)
    } else {
        (coins + 50, Some("legendary_ray_gun"), 1)
    }
}

/// Rolls personal boss loot for each contributing player upon defeating the Obsidian Dragon Boss.
pub fn roll_boss_personal_loot() -> (u64, &'static str, u32) {
    let mut rng = rand::thread_rng();
    let coins = rng.gen_range(95u64..=140u64);
    let roll: u32 = rng.gen_range(0..100);
    if roll < 24 {
        (coins, "crown_relic", 1)
    } else if roll < 44 {
        (coins, "dragon_egg_relic", 1)
    } else if roll < 60 {
        (coins, "cosmetic_hat_obsidian_crown", 1)
    } else if roll < 76 {
        (coins, "cosmetic_acc_crystal_wings", 1)
    } else if roll < 88 {
        (coins, "aurora_crystal_lamp", 1)
    } else {
        (coins + 50, "legendary_ray_gun", 1)
    }
}

// ============================================================================
// Shadow Rift Mortal Arena (`ZONE_ARENA = 3`) — Chasing Mobs, Lava, Boss & Respawn
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArenaPlayerStatus {
    pub hp: u8,
    pub max_hp: u8,
    pub alive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defeated_by: Option<String>,
    pub respawn_ready_at_ms: u64,
    pub invuln_until_ms: u64,
}

impl ArenaPlayerStatus {
    pub fn fresh(now: u64) -> Self {
        Self::fresh_with_max_hp(now, ARENA_MAX_HP)
    }

    pub fn fresh_with_max_hp(now: u64, max_hp: u8) -> Self {
        let capped = max_hp.max(ARENA_MAX_HP);
        Self {
            hp: capped,
            max_hp: capped,
            alive: true,
            defeated_by: None,
            respawn_ready_at_ms: 0,
            invuln_until_ms: now + 2_500,
        }
    }
}

pub fn max_hp_for_inventory(inventory: &HashMap<String, u32>) -> u8 {
    if player_owns_item(inventory, "aegis_heart_charm") {
        ARENA_AEGIS_MAX_HP
    } else {
        ARENA_MAX_HP
    }
}

#[derive(Clone, Debug)]
pub struct ArenaDamageNotice {
    pub self_id: String,
    pub display_name: String,
    pub status: ArenaPlayerStatus,
    pub damage: u8,
    pub source_label: String,
    pub just_died: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BossShockwave {
    pub id: u64,
    pub x: f32,
    pub z: f32,
    pub radius: f32,
    pub max_radius: f32,
    pub spawned_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaBossSnapshot {
    pub active: bool,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ry: f32,
    pub hp: u16,
    pub max_hp: u16,
    pub cooldown_until_ms: u64,
    pub beacon_boosted: bool,
    pub altar_pos: [f32; 3],
    pub shockwaves: Vec<BossShockwave>,
    pub contributors_count: usize,
}

pub type BossAttackOutcome = (ArenaBossSnapshot, Option<(Vec<String>, bool)>);

#[derive(Clone, Debug)]
pub struct ArenaBossState {
    pub active: bool,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ry: f32,
    pub hp: u16,
    pub max_hp: u16,
    pub cooldown_until_ms: u64,
    pub beacon_boosted: bool,
    pub shockwaves: Vec<BossShockwave>,
    pub contributors: std::collections::HashSet<String>,
    pub scaled_players_count: usize,
    last_shockwave_ms: u64,
    next_shockwave_id: u64,
    shockwave_hits: std::collections::HashSet<(u64, String)>,
}

impl Default for ArenaBossState {
    fn default() -> Self {
        Self {
            active: false,
            name: "Obsidian Rift Dragon".to_string(),
            x: ARENA_BOSS_ALTAR_POS.0,
            y: 0.4,
            z: ARENA_BOSS_ALTAR_POS.2,
            ry: 0.0,
            hp: 0,
            max_hp: 18,
            cooldown_until_ms: 0,
            beacon_boosted: false,
            shockwaves: Vec::new(),
            contributors: std::collections::HashSet::new(),
            scaled_players_count: 1,
            last_shockwave_ms: 0,
            next_shockwave_id: 1,
            shockwave_hits: std::collections::HashSet::new(),
        }
    }
}

impl ArenaBossState {
    pub fn snapshot(&self) -> ArenaBossSnapshot {
        ArenaBossSnapshot {
            active: self.active,
            name: self.name.clone(),
            x: self.x,
            y: self.y,
            z: self.z,
            ry: self.ry,
            hp: self.hp,
            max_hp: self.max_hp,
            cooldown_until_ms: self.cooldown_until_ms,
            beacon_boosted: self.beacon_boosted,
            altar_pos: [
                ARENA_BOSS_ALTAR_POS.0,
                ARENA_BOSS_ALTAR_POS.1,
                ARENA_BOSS_ALTAR_POS.2,
            ],
            shockwaves: self.shockwaves.clone(),
            contributors_count: self.contributors.len(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaSnapshot {
    pub nodes: Vec<DungeonNode>,
    pub mobs: Vec<DungeonMob>,
    pub drops: Vec<DungeonDrop>,
    pub portal_pos: [f32; 3],
    pub return_portal_pos: [f32; 3],
    pub boss: ArenaBossSnapshot,
}

pub fn is_in_arena_sanctuary(x: f32, z: f32) -> bool {
    z >= 19.5 && x.abs() <= 9.0
}

/// Returns `Some((damage, cause))` if `(x, y, z)` is touching the Molten Lava River
/// (`z in (-6.8, -3.2)` off the 3 basalt bridges at `x = 0, ±16.0`) or Outer Lava Moat.
pub fn check_arena_lava_hazard(x: f32, y: f32, z: f32) -> Option<(u8, &'static str)> {
    if y < -2.2 {
        return Some((u8::MAX, "the Molten Abyss"));
    }
    if is_in_arena_sanctuary(x, z) {
        return None;
    }
    if (x.abs() > 30.4 || z.abs() > 33.4) && y <= 0.28 {
        return Some((35, "the Outer Lava Moat"));
    }
    let on_bridge = x.abs() <= 2.75 || (13.25..=18.75).contains(&x.abs());
    if z > -6.8 && z < -3.2 && !on_bridge && y <= 0.14 {
        return Some((34, "the Molten Lava River"));
    }
    None
}

pub fn arena_mob_melee_damage(kind: &str) -> u8 {
    match kind {
        "lava_golem" => 32,
        "void_wraith" => 22,
        "rift_spider" => 15,
        _ => 12,
    }
}

pub struct ArenaState {
    pub nodes: Vec<DungeonNode>,
    pub mobs: Vec<DungeonMob>,
    pub drops: Vec<DungeonDrop>,
    pub boss: ArenaBossState,
    pub player_statuses: HashMap<String, ArenaPlayerStatus>,
    last_player_hit_ms: HashMap<String, u64>,
    last_lava_tick_ms: HashMap<String, u64>,
    last_tick_ms: u64,
    next_drop_seq: u64,
}

impl Default for ArenaState {
    fn default() -> Self {
        Self::new()
    }
}

impl ArenaState {
    pub fn new() -> Self {
        let nodes = vec![
            DungeonNode {
                id: "arena_crystal_1".into(),
                kind: "crystal".into(),
                label: "Bloodstone Rift Spire".into(),
                x: -16.0,
                y: 0.0,
                z: 10.0,
                available: true,
                cooldown_ms: 15_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "arena_crystal_2".into(),
                kind: "crystal".into(),
                label: "Bloodstone Rift Spire".into(),
                x: 16.0,
                y: 0.0,
                z: 10.0,
                available: true,
                cooldown_ms: 15_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "arena_crystal_3".into(),
                kind: "crystal".into(),
                label: "Void-Shard Cluster".into(),
                x: -22.0,
                y: 0.0,
                z: -14.0,
                available: true,
                cooldown_ms: 16_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "arena_crystal_4".into(),
                kind: "crystal".into(),
                label: "Void-Shard Cluster".into(),
                x: 22.0,
                y: 0.0,
                z: -14.0,
                available: true,
                cooldown_ms: 16_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "arena_chest_1".into(),
                kind: "wooden_chest".into(),
                label: "Obsidian War Chest".into(),
                x: -8.0,
                y: 0.0,
                z: 2.0,
                available: true,
                cooldown_ms: 22_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "arena_chest_2".into(),
                kind: "wooden_chest".into(),
                label: "Obsidian War Chest".into(),
                x: 8.0,
                y: 0.0,
                z: 2.0,
                available: true,
                cooldown_ms: 22_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "arena_vault_1".into(),
                kind: "golden_chest".into(),
                label: "Magma Citadel Vault (Needs Key 🗝️)".into(),
                x: -14.0,
                y: 0.0,
                z: -27.0,
                available: true,
                cooldown_ms: 34_000,
                ready_at_ms: 0,
            },
            DungeonNode {
                id: "arena_vault_2".into(),
                kind: "golden_chest".into(),
                label: "Shadow Throne Vault (Needs Key 🗝️)".into(),
                x: 14.0,
                y: 0.0,
                z: -27.0,
                available: true,
                cooldown_ms: 34_000,
                ready_at_ms: 0,
            },
        ];

        let mobs = vec![
            DungeonState::make_mob(
                "spider_1",
                "rift_spider",
                "Rift Spider",
                -9.5,
                0.0,
                10.5,
                4.2,
                0.0,
                3.7,
                3,
            ),
            DungeonState::make_mob(
                "spider_2",
                "rift_spider",
                "Rift Spider",
                9.5,
                0.0,
                10.5,
                4.2,
                1.9,
                3.7,
                3,
            ),
            DungeonState::make_mob(
                "spider_3",
                "rift_spider",
                "Venomfang Spider",
                0.0,
                0.0,
                3.5,
                4.6,
                3.5,
                3.9,
                3,
            ),
            DungeonState::make_mob(
                "wraith_1",
                "void_wraith",
                "Void Wraith",
                -15.5,
                0.25,
                -11.5,
                4.5,
                0.8,
                4.1,
                4,
            ),
            DungeonState::make_mob(
                "wraith_2",
                "void_wraith",
                "Void Wraith",
                15.5,
                0.25,
                -11.5,
                4.5,
                2.7,
                4.1,
                4,
            ),
            DungeonState::make_mob(
                "wraith_3",
                "void_wraith",
                "Phantom Archon",
                0.0,
                0.25,
                -14.0,
                5.0,
                4.4,
                4.3,
                4,
            ),
            DungeonState::make_mob(
                "golem_1",
                "lava_golem",
                "Magma Colossus",
                -9.5,
                0.0,
                -22.5,
                4.0,
                1.2,
                2.9,
                6,
            ),
            DungeonState::make_mob(
                "golem_2",
                "lava_golem",
                "Obsidian Titan",
                9.5,
                0.0,
                -22.5,
                4.0,
                3.9,
                2.9,
                6,
            ),
        ];

        Self {
            nodes,
            mobs,
            drops: Vec::new(),
            boss: ArenaBossState::default(),
            player_statuses: HashMap::new(),
            last_player_hit_ms: HashMap::new(),
            last_lava_tick_ms: HashMap::new(),
            last_tick_ms: now_ms(),
            next_drop_seq: 5_001,
        }
    }

    pub fn snapshot(&self) -> ArenaSnapshot {
        ArenaSnapshot {
            nodes: self.nodes.clone(),
            mobs: self.mobs.clone(),
            drops: self.drops.clone(),
            portal_pos: [
                PLAZA_ARENA_PORTAL_POS.0,
                PLAZA_ARENA_PORTAL_POS.1,
                PLAZA_ARENA_PORTAL_POS.2,
            ],
            return_portal_pos: [
                ARENA_RETURN_PORTAL_POS.0,
                ARENA_RETURN_PORTAL_POS.1,
                ARENA_RETURN_PORTAL_POS.2,
            ],
            boss: self.boss.snapshot(),
        }
    }

    pub fn enter_or_reset_player_with_max_hp(
        &mut self,
        self_id: &str,
        max_hp: u8,
        now: u64,
    ) -> ArenaPlayerStatus {
        let fresh = ArenaPlayerStatus::fresh_with_max_hp(now, max_hp);
        self.player_statuses
            .insert(self_id.to_string(), fresh.clone());
        self.last_player_hit_ms.remove(self_id);
        self.last_lava_tick_ms.remove(self_id);
        fresh
    }

    pub fn enter_or_reset_player(&mut self, self_id: &str, now: u64) -> ArenaPlayerStatus {
        self.enter_or_reset_player_with_max_hp(self_id, ARENA_MAX_HP, now)
    }

    pub fn sync_player_max_hp(
        &mut self,
        self_id: &str,
        desired_max_hp: u8,
    ) -> Option<ArenaPlayerStatus> {
        let st = self.player_statuses.get_mut(self_id)?;
        let capped = desired_max_hp.max(ARENA_MAX_HP);
        if st.max_hp == capped {
            return None;
        }
        if capped > st.max_hp && st.alive {
            st.hp = st.hp.saturating_add(capped - st.max_hp).min(capped);
        } else if st.hp > capped {
            st.hp = capped;
        }
        st.max_hp = capped;
        Some(st.clone())
    }

    pub fn status_for(&self, self_id: &str) -> ArenaPlayerStatus {
        self.player_statuses
            .get(self_id)
            .cloned()
            .unwrap_or_else(|| ArenaPlayerStatus::fresh(0))
    }

    pub fn remove_player(&mut self, self_id: &str) {
        self.player_statuses.remove(self_id);
        self.last_player_hit_ms.remove(self_id);
        self.last_lava_tick_ms.remove(self_id);
    }

    pub fn heal_player(&mut self, self_id: &str, amount: u8) -> Option<ArenaPlayerStatus> {
        let st = self.player_statuses.get_mut(self_id)?;
        if !st.alive {
            return None;
        }
        st.hp = st.hp.saturating_add(amount).min(st.max_hp);
        Some(st.clone())
    }

    pub fn summon_boss(
        &mut self,
        summoner_username: &str,
        active_arena_players: usize,
        use_beacon: bool,
        now: u64,
    ) -> Result<ArenaBossSnapshot, String> {
        if self.boss.active {
            return Err("🐉 The Obsidian Rift Dragon is already awake at the North Altar!".into());
        }
        if !use_beacon && now < self.boss.cooldown_until_ms {
            let secs = ((self.boss.cooldown_until_ms - now) as f64 / 1000.0).ceil() as u64;
            return Err(format!(
                "⏳ Altar recharging ({secs}s left) — or use an Obsidian Rift Beacon 🔮 to summon instantly!"
            ));
        }
        let players_n = active_arena_players.max(1);
        let max_hp = (18u16 + (players_n.saturating_sub(1) as u16) * 12).min(120);
        self.boss.active = true;
        self.boss.x = ARENA_BOSS_ALTAR_POS.0;
        self.boss.y = 0.55;
        self.boss.z = ARENA_BOSS_ALTAR_POS.2;
        self.boss.ry = 0.0;
        self.boss.hp = max_hp;
        self.boss.max_hp = max_hp;
        self.boss.beacon_boosted = use_beacon;
        self.boss.shockwaves.clear();
        self.boss.contributors.clear();
        self.boss
            .contributors
            .insert(summoner_username.to_ascii_lowercase());
        self.boss.scaled_players_count = players_n;
        self.boss.last_shockwave_ms = now;
        self.boss.shockwave_hits.clear();
        Ok(self.boss.snapshot())
    }

    /// Applies `damage` to the Obsidian Dragon Boss and records `attacker_username` as a contributor.
    /// Returns `(boss_snapshot, defeated_contributors_and_beacon_flag)`.
    pub fn attack_boss(
        &mut self,
        attacker_username: &str,
        damage: u16,
        now: u64,
    ) -> Result<BossAttackOutcome, String> {
        if !self.boss.active {
            return Err("No active Boss at the North Altar right now.".into());
        }
        let uname_lc = attacker_username.to_ascii_lowercase();
        let is_new_contributor = self.boss.contributors.insert(uname_lc);
        if is_new_contributor && self.boss.contributors.len() > self.boss.scaled_players_count {
            self.boss.scaled_players_count = self.boss.contributors.len();
            self.boss.hp = self.boss.hp.saturating_add(10).min(140);
            self.boss.max_hp = self.boss.max_hp.saturating_add(10).min(140);
        }
        let dealt = damage.max(1);
        self.boss.hp = self.boss.hp.saturating_sub(dealt);
        if self.boss.hp == 0 {
            self.boss.active = false;
            self.boss.cooldown_until_ms = now + 25_000;
            self.boss.shockwaves.clear();
            let mut contributors: Vec<String> = self.boss.contributors.iter().cloned().collect();
            contributors.sort();
            let boosted = self.boss.beacon_boosted;
            return Ok((self.boss.snapshot(), Some((contributors, boosted))));
        }
        Ok((self.boss.snapshot(), None))
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
        self.next_drop_seq = self.next_drop_seq.wrapping_add(1).max(5_001);
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
            id: format!("arena_drop_{seq}"),
            x: (x * 100.0).round() / 100.0,
            y: y.max(0.0),
            z: (z * 100.0).round() / 100.0,
            coins,
            item_id: valid_item,
            qty: if item_id.is_some() { qty.max(1) } else { 0 },
            label,
            rarity,
            expires_at_ms: now + 75_000,
        };
        if self.drops.len() >= 80 {
            self.drops.remove(0);
        }
        self.drops.push(drop_entry.clone());
        drop_entry
    }

    fn apply_damage_internal(
        &mut self,
        self_id: &str,
        display_name: &str,
        raw_damage: u8,
        source_label: &str,
        now: u64,
    ) -> Option<ArenaDamageNotice> {
        let st = self
            .player_statuses
            .entry(self_id.to_string())
            .or_insert_with(|| ArenaPlayerStatus::fresh(0));
        if !st.alive || now < st.invuln_until_ms || raw_damage == 0 {
            return None;
        }
        st.hp = st.hp.saturating_sub(raw_damage);
        let mut just_died = false;
        if st.hp == 0 {
            st.alive = false;
            st.defeated_by = Some(source_label.to_string());
            st.respawn_ready_at_ms = now + 1_200;
            just_died = true;
        }
        Some(ArenaDamageNotice {
            self_id: self_id.to_string(),
            display_name: display_name.to_string(),
            status: st.clone(),
            damage: raw_damage,
            source_label: source_label.to_string(),
            just_died,
        })
    }

    /// Advances Shadow Rift Arena chasing AI, melee strikes, Boss Shockwave Rings, lava burn hazards, node/mob respawns, and drops.
    /// `arena_players` entries: `(self_id, display_name, x, y, z)`.
    pub fn tick(
        &mut self,
        now: u64,
        arena_players: &[(String, String, f32, f32, f32)],
    ) -> (bool, Vec<DungeonMob>, Vec<ArenaDamageNotice>) {
        let dt = ((now.saturating_sub(self.last_tick_ms)) as f32 * 0.001).clamp(0.05, 0.40);
        self.last_tick_ms = now;

        let mut world_changed = false;
        let mut damage_notices = Vec::new();

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

        // Ensure every player in ZONE_ARENA has an ArenaPlayerStatus and check lava hazards
        for (pid, disp, px, py, pz) in arena_players {
            if !self.player_statuses.contains_key(pid) {
                self.player_statuses
                    .insert(pid.clone(), ArenaPlayerStatus::fresh(now));
            }
            let is_alive = self
                .player_statuses
                .get(pid)
                .map(|s| s.alive)
                .unwrap_or(false);
            if !is_alive {
                continue;
            }
            if let Some((lava_dmg, lava_cause)) = check_arena_lava_hazard(*px, *py, *pz) {
                let last_lava = self.last_lava_tick_ms.get(pid).copied().unwrap_or(0);
                if lava_dmg >= ARENA_MAX_HP || now.saturating_sub(last_lava) >= 650 {
                    self.last_lava_tick_ms.insert(pid.clone(), now);
                    if let Some(notice) =
                        self.apply_damage_internal(pid, disp, lava_dmg, lava_cause, now)
                    {
                        damage_notices.push(notice);
                    }
                }
            }
        }

        // Collect living, non-Sanctuary player targets for mob & boss aggro
        let valid_targets: Vec<(String, String, f32, f32, f32)> = arena_players
            .iter()
            .filter(|(pid, _, px, _, pz)| {
                !is_in_arena_sanctuary(*px, *pz)
                    && self
                        .player_statuses
                        .get(pid)
                        .map(|s| s.alive)
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        let t_sec = (now % 3_600_000) as f32 * 0.001;
        let mut pending_melee_hits: Vec<(String, String, u8, String)> = Vec::new();

        // Advance Obsidian Rift Dragon Boss + Expanding Jump-Over Shockwave Rings
        if self.boss.active {
            let orbit_ang = t_sec * 0.65;
            self.boss.x =
                ((ARENA_BOSS_ALTAR_POS.0 + orbit_ang.cos() * 2.4) * 100.0).round() / 100.0;
            self.boss.z =
                ((ARENA_BOSS_ALTAR_POS.2 + orbit_ang.sin() * 1.8) * 100.0).round() / 100.0;
            self.boss.y = ((0.65 + (t_sec * 2.4).sin() * 0.28) * 100.0).round() / 100.0;

            let north_targets: Vec<&(String, String, f32, f32, f32)> = valid_targets
                .iter()
                .filter(|(_, _, _, _, pz)| *pz < -6.5)
                .collect();
            if let Some((_, _, tx, _, tz)) = north_targets.first() {
                let dx = *tx - self.boss.x;
                let dz = *tz - self.boss.z;
                if dx.abs() + dz.abs() > 0.05 {
                    self.boss.ry = dx.atan2(dz);
                }
            }

            if !north_targets.is_empty() && now.saturating_sub(self.boss.last_shockwave_ms) >= 3_600
            {
                let sw_id = self.boss.next_shockwave_id;
                self.boss.next_shockwave_id = self.boss.next_shockwave_id.wrapping_add(1).max(1);
                self.boss.last_shockwave_ms = now;
                self.boss.shockwaves.push(BossShockwave {
                    id: sw_id,
                    x: self.boss.x,
                    z: self.boss.z,
                    radius: 0.8,
                    max_radius: 18.0,
                    spawned_at_ms: now,
                });
                world_changed = true;
            }

            let mut active_sw = Vec::with_capacity(self.boss.shockwaves.len());
            for mut sw in self.boss.shockwaves.drain(..) {
                let prev_r = sw.radius;
                sw.radius = ((sw.radius + 7.5 * dt) * 100.0).round() / 100.0;
                for (t_id, t_disp, tx, ty, tz) in &north_targets {
                    let dist = ((*tx - sw.x).powi(2) + (*tz - sw.z).powi(2)).sqrt();
                    let band_min = prev_r - 0.85;
                    let band_max = sw.radius + 0.85;
                    // Players who jump (`ty >= 0.65`) vault cleanly over the ground shockwave ring!
                    if dist >= band_min
                        && dist <= band_max
                        && *ty < 0.65
                        && self.boss.shockwave_hits.insert((sw.id, t_id.clone()))
                    {
                        pending_melee_hits.push((
                            t_id.clone(),
                            t_disp.clone(),
                            18,
                            "Obsidian Dragon Shockwave".to_string(),
                        ));
                    }
                }
                if sw.radius < sw.max_radius {
                    active_sw.push(sw);
                } else {
                    self.boss.shockwave_hits.retain(|(id, _)| *id != sw.id);
                    world_changed = true;
                }
            }
            self.boss.shockwaves = active_sw;
        }

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

            let mob_south_bank = mob.home_z > -5.0;
            // Find closest living player on the same side of the lava river (within 14m aggro range)
            let mut best_target: Option<&(String, String, f32, f32, f32)> = None;
            let mut best_dist = 14.0f32;
            for tgt in &valid_targets {
                let tgt_south_bank = tgt.4 > -5.0;
                if tgt_south_bank != mob_south_bank {
                    continue;
                }
                let d = ((tgt.2 - mob.x).powi(2) + (tgt.4 - mob.z).powi(2)).sqrt();
                if d < best_dist {
                    best_dist = d;
                    best_target = Some(tgt);
                }
            }

            if let Some((t_id, t_disp, tx, ty, tz)) = best_target {
                let dx = *tx - mob.x;
                let dz = *tz - mob.z;
                let horiz = (dx * dx + dz * dz).sqrt();
                if horiz > 0.05 {
                    mob.ry = dx.atan2(dz);
                }
                if horiz > 1.35 {
                    let step = (mob.speed * dt).min(horiz - 1.25);
                    let mut nx = mob.x + (dx / horiz) * step;
                    let mut nz = mob.z + (dz / horiz) * step;
                    nx = nx.clamp(-29.0, 29.0);
                    if mob_south_bank {
                        nz = nz.clamp(-2.3, 18.8);
                    } else {
                        nz = nz.clamp(-31.5, -7.7);
                    }
                    mob.x = (nx * 100.0).round() / 100.0;
                    mob.z = (nz * 100.0).round() / 100.0;
                }

                // Melee strike check (`<= 2.05m` horizontal and reasonable vertical reach)
                let post_dist = ((*tx - mob.x).powi(2) + (*tz - mob.z).powi(2)).sqrt();
                if post_dist <= 2.05 && (*ty - mob.y).abs() <= 2.2 {
                    let last_hit = self.last_player_hit_ms.get(t_id).copied().unwrap_or(0);
                    if now.saturating_sub(last_hit) >= 1_150 {
                        self.last_player_hit_ms.insert(t_id.clone(), now);
                        pending_melee_hits.push((
                            t_id.clone(),
                            t_disp.clone(),
                            arena_mob_melee_damage(&mob.kind),
                            mob.name.clone(),
                        ));
                    }
                }
            } else {
                // Smooth patrol orbit around home
                let angle = t_sec * (mob.speed * 0.34) + mob.phase;
                let nx = mob.home_x + angle.cos() * mob.patrol_radius;
                let nz = mob.home_z + (angle * 0.85).sin() * (mob.patrol_radius * 0.78);
                let dx = nx - mob.x;
                let dz = nz - mob.z;
                if dx.abs() + dz.abs() > 0.01 {
                    mob.ry = dx.atan2(dz);
                }
                let blend = (dt * 2.8).min(1.0);
                mob.x = ((mob.x + dx * blend) * 100.0).round() / 100.0;
                mob.z = ((mob.z + dz * blend) * 100.0).round() / 100.0;
            }

            if mob.kind == "void_wraith" {
                let hover = (t_sec * 3.1 + mob.phase).sin() * 0.22;
                mob.y = ((mob.home_y + hover) * 100.0).round() / 100.0;
            }
        }

        for (t_id, t_disp, dmg, mob_name) in pending_melee_hits {
            if let Some(notice) = self.apply_damage_internal(&t_id, &t_disp, dmg, &mob_name, now) {
                damage_notices.push(notice);
            }
        }

        (world_changed, self.mobs.clone(), damage_notices)
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
        let mut u = UserAccount::new_basic(username, username, "pw", "friend");
        u.coins = coins;
        u
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

    #[test]
    fn parent_exclusive_items_never_spawn_in_dungeon_loot_tables() {
        for _ in 0..250 {
            for kind in ["shadow_slime", "crystal_bat"] {
                let (_, item_opt, _) = roll_mob_loot(kind);
                if let Some(id) = item_opt {
                    let def = find_item_def(id).expect("valid item def");
                    assert!(
                        !def.parent_exclusive,
                        "Parent-exclusive item {id} must never spawn from mob loot"
                    );
                }
            }
            for (_, item_opt, _) in [
                roll_crystal_loot(),
                roll_wooden_chest_loot(),
                roll_golden_chest_loot(),
            ] {
                if let Some(id) = item_opt {
                    let def = find_item_def(id).expect("valid item def");
                    assert!(
                        !def.parent_exclusive,
                        "Parent-exclusive item {id} must never spawn from crystal/chest loot"
                    );
                }
            }
        }
    }
}

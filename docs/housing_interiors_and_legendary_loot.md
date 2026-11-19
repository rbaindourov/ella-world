# Solid 2-Story House Collisions, Doors & Windows, 3D Home Interiors, Backpack Loot Decorating & Legendary Ray Gun

## 1. Overview

To support players building multi-story houses with enclosed rooms, kitchens, bathrooms, bedrooms, and trophy displays, ELLA World adds:

1. **Solid Cylinder-vs-AABB Wall, Stair, Ceiling & Indoor Camera Physics** (`public/js/world3d.js`) so avatars cannot walk through house walls, clip through 2nd-story floor slabs, or lose camera visibility indoors.
2. **Interactive Hinged Doors & Windows** (`kind: "door"` & `kind: "window"`) synchronized live across all connected peers (`action: "toggle"` via `E` or click).
3. **Headless Blender 5.2 Home Interiors & Legendary Loot Pack** (`scripts/build_props_and_loot_pack.py` → `assets/blender/ella_world_props_loot.blend` + `public/models/ella_world_props_loot.glb`, 25 meshes).
4. **Categorized Home Builder HUD (`#build-toolbar`) + 90° Rotation (`Q`)** (`🧱 Structure`, `🍳 Kitchen`, `🛁 Bathroom`, `🛏️ Bedroom`, `🏆 My Loot`) with accessible `focusgroup` toolbar/tablist semantics and scroll-driven edge fades.
5. **Anti-Duplication Backpack Loot House Decorating** (`kind: "loot_decor"`, `lootItemId`) with atomic `-1` placement deduction, owner-only pickup protection, and `+1` Backpack refund on pickup or parent world reset.
6. **5 New 3D Dungeon Relics + First Legendary Item: `legendary_ray_gun` (`🔫 Starlight Plasma Ray Gun`)** with `"legendary"` rarity, right-hand 3D weapon mounting, `14.5m` `3 HP` ranged Crystal Cavern plasma beams, and harmless Starlight Plaza laser light shows (`POST /api/ella-world/raygun`).

---

## 2. Solid 3D House Collision, Auto-Step Stairs & Indoor Camera (`public/js/world3d.js`)

### 2.1 Horizontal Circle-vs-AABB Wall Sweep + `0.55m` Stair Auto-Step
- **Player Capsule Envelope**: `playerRadius = 0.36`, `playerHeight = 1.75`.
- **Two-Step Stairs (`kind: "stairs"`)**: Each 1×1×1 stair cell registers two non-overlapping step colliders (`0.0..0.5m` front tread and `0.5..1.0m` rear tread oriented along `rot = 0..3`).
- **Auto-Stepping Without Wall-Climbing**: When horizontal motion intersects a solid collider (`!c.passable`), `_animate(now)` checks whether `stepUp = c.topY - pos.y` falls within `(0, 0.55]` (`stepHeight = 0.55`) and verifies `_hasHeadroomAt(pos.x, c.topY, pos.z)` (`1.75m` vertical clearance). Players smoothly walk up `0.5m` stair treads to reach 2nd-story floors (`y = 2.0m+`), while `1.0m` wall blocks (`stepUp = 1.0 > 0.55`) cleanly stop horizontal movement and allow smooth wall-sliding along the tangent axis.
- **Unstuck Guard**: If a friend builds a solid block directly around a standing avatar, the shortest radial push-out vector ejects the avatar horizontally without trapping them inside walls.

### 2.2 Vertical Ceiling Bonk & Highest-Surface Landing
- Jumping indoors (`this.velocity.y > 0`) checks head clearance (`nextY + playerHeight`) against overhead floor slabs (`c.bottomY`), clamping `pos.y = c.bottomY - playerHeight` and zeroing upward velocity on impact.
- Downward landing (`this.velocity.y <= 0`) selects the **highest** valid `c.topY` beneath the player's horizontal footprint rather than breaking on the first collider in array order, ensuring rock-solid footing on 2nd-story floors above 1st-floor furniture.

### 2.3 Indoor 3rd-Person Camera Anti-Clip Pullback
- `targetLookAt` sits at `pos.y + 1.62` (safely inside `2.0m` 1st-story ceiling height rather than inside the 2nd-floor slab).
- `this.cameraRaycaster` casts from `targetLookAt` toward the desired orbit camera position against `this.cameraOccluders` (opaque house walls, ceilings, closed doors, and static architecture, excluding glass windows and small interior props), smoothly pulling `this.effectiveCameraDist` inward (`hitDist - 0.28`, `minDist = 1.05`) inside rooms and easing back out outdoors.

---

## 3. Backward-Compatible `VoxelBlock` Schema & Interactive Doors / Windows

### 3.1 `VoxelBlock` Persistence Schema (`src/models.rs`)
Existing `data/world_blocks.json` files hydrate with zero migration steps via `#[serde(default)]`:

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `x`, `y`, `z` | `i32` | *(required)* | Plaza grid coordinates (`[-45..45]`, `y` in `[1..60]`) |
| `color` | `String` | *(required)* | Validated `#RRGGBB` paint or pedestal tint |
| `author` | `String` | *(required)* | Builder / trophy owner username |
| `placedAt` | `Option<u64>` | `None` | Unix timestamp (ms) |
| `kind` | `String` | `"block"` | One of `ALLOWED_BLOCK_KINDS` (`block`, `stairs`, `door`, `window`, kitchen/bath/bedroom props, `loot_decor`) |
| `rot` | `u8` | `0` | Quarter-turn orientation (`0 = 0°`, `1 = 90°`, `2 = 180°`, `3 = 270°`) |
| `open` | `bool` | `false` | Hinge state for `door` and `window` |
| `lootItemId` | `Option<String>` | `None` | Catalog item ID displayed when `kind == "loot_decor"` |

- **2-Block-Tall Props (`is_two_tall_block_kind`)**: `"door"`, `"kitchen_fridge"`, and `"wardrobe"` occupy both `(x, y, z)` and `(x, y + 1, z)` for overlap checks, raycast removal, and door toggling.
- **Interactive Hinges (`action: "toggle"`)**: Pressing `E` within `3.6m` or clicking a door/window sends `POST /api/ella-world/block` (`{ action: "toggle", x, y, z }`) or WS/UDP `"block"` action `"toggle"`. All connected peers animate the door leaf (`Prop_Door_Leaf`) or window sash (`Prop_Window_Sash`) around its hinge pivot and toggle `collider.passable = block.open`.

---

## 4. Blender 5.2 Asset Pack (`scripts/build_props_and_loot_pack.py`)

Generated non-destructively via headless Blender 5.2 (`blender -b --factory-startup -P scripts/build_props_and_loot_pack.py`):
- **Source `.blend`**: `assets/blender/ella_world_props_loot.blend`
- **Runtime `.glb`**: `public/models/ella_world_props_loot.glb` (25 meshes, `PaintTint` material slot dynamically recolored per placed block's `color` swatch while preserving metallic chrome, glass, porcelain, Cushion fabric, and emissive accents):
  - **Architecture (5)**: `Prop_Stairs`, `Prop_Door_Frame`, `Prop_Door_Leaf`, `Prop_Window_Frame`, `Prop_Window_Sash`.
  - **Kitchen (6)**: `Prop_Kitchen_Counter`, `Prop_Kitchen_Sink`, `Prop_Kitchen_Stove`, `Prop_Kitchen_Fridge`, `Prop_Dining_Table`, `Prop_Chair` (sourced & normalized from CC0 PolyHaven `DiningChair_01`).
  - **Bathroom (3)**: `Prop_Bath_Tub`, `Prop_Bath_Vanity`, `Prop_Bath_Toilet`.
  - **Bedroom (4)**: `Prop_Bed` (normalized from CC0 PolyHaven `GothicBed_01`), `Prop_Nightstand_Lamp` (normalized from CC0 PolyHaven `ClassicNightstand_01`), `Prop_Wardrobe`, `Prop_Rug`.
  - **Trophy Pedestal & 3D Loot Relics (7)**: `Loot_Display_Pedestal`, `Loot_Ray_Gun`, `Loot_Geode_Lamp`, `Loot_Slime_Lamp`, `Loot_Bat_Plush`, `Loot_Royal_Chalice` (normalized from CC0 PolyHaven `brass_goblets`), and `Loot_Dragon_Egg`.

---

## 5. Anti-Duplication Backpack Loot Decorating (`kind: "loot_decor"`)

Players can display any owned item from their Backpack (`🏆 My Loot` tab in Build Mode or `🏠 Decorate Home` inside Backpack `I`):
1. **Atomic `-1` Deduction**: Placing `kind: "loot_decor"` with `lootItemId` atomically decrements `1` from `UserAccount::inventory` inside `WorldEngine::mutate_block_ext` and runs `reconcile_equipped_cosmetics` (automatically holstering an equipped cosmetic/weapon if its last copy is placed on a pedestal).
2. **Owner-Only Protection**: Friends cannot overwrite a `loot_decor` pedestal with another block or pick up another player's displayed trophy (`Only <author> can pick up their displayed trophy!`).
3. **Guaranteed `+1` Refund**: Removing a `loot_decor` pedestal (`Right-Click` or `⛏️ Remove`) refunds `+1` of `lootItemId` to `block.author`. Parent `POST /api/ella-world/admin/world/reset` also refunds all displayed `loot_decor` items back to their owners before clearing `world_blocks.json`, and the `12,000`-block FIFO cap (expanded for the `210 × 210` map across `x/z ∈ [-100, 100], y ∈ [1, 80]`) evicts plain `"block"` entries before `loot_decor`.
4. **Walkable Single (`door`) & Grand `2×2` Double Doors (`double_door`)**: Both doors open/close manually via `E` (even while Build Mode is active) or clicking either leaf. Open doorways register in `openDoorPortals` for snag-free passage (`0.26` corridor radius + gentle lateral centering), and `double_door` enforces an authoritative `2×2` voxel footprint (`block_occupied_cells`) on placement, overlap, toggling, and removal.

---

## 6. New Dungeon Drops & First Legendary Item (`legendary_ray_gun`)

Added 6 new items to `ITEM_CATALOG` (`src/economy.rs`):
- `geode_lamp` (*uncommon*, Crystal Veins & Wooden Chests) — **🔮 Amethyst Geode Lamp**
- `slime_lava_lamp` (*rare*, Shadow Slimes) — **🧪 Neon Slime Lava Lamp**
- `bat_plushie` (*rare*, Crystal Bats) — **🦇 Plushie Crystal Bat**
- `royal_chalice` (*epic*, Wooden & Golden Vault Chests) — **🏆 Golden Starlight Chalice**
- `dragon_egg_relic` (*epic*, Golden Vault Chests) — **🥚 Starlight Dragon Egg**
- `legendary_ray_gun` (*legendary*, `10%` Golden Vault Chest jackpot + rare `2%` Crystal Bat drop, `950 🪙` trade value) — **🔫 Starlight Plasma Ray Gun**:
  - **Right-Hand 3D Weapon Attachment**: Equippable via Backpack (`🔫 Equip Ray Gun`) or Glam Salon (`accessory: "ray_gun"`), mounting `Loot_Ray_Gun` onto `Hand.R` (`StylizedCharacterRig`) and `rightArmPivot` (`ClassicBlockyRig`).
  - **Ranged Crystal Cavern Plasma Beam**: Fires a `14.5m` cyan-magenta plasma beam (`POST /api/ella-world/dungeon/attack` with `mode: "raygun"` or `WS/UDP` `"dungeon_attack"`), dealing **`3 HP` instant-defeat damage** to Shadow Slimes and Crystal Bats and broadcasting `UdpWorldEvent::RayGunBeam` to all cavern peers.
  - **Starlight Plaza Laser Light Show**: Clicking in Starlight Plaza with the Ray Gun equipped fires harmless neon laser beams (`POST /api/ella-world/raygun` / `"raygun_zap"`, `200ms` per-player cooldown) visible to everyone in the Plaza.

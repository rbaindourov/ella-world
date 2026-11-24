# ELLA World — Blender 5.2 3D Asset & Optimization Inventory

Canonical inventory of all Blender `.blend`/`.glb` asset packs and remaining procedural Three.js primitives in **ELLA World** (`public/js/world3d.js`), ordered by visual impact, draw-call/polygon savings, and CC0 open-source sourcing strategy. Machine-readable manifest: [`docs/blender_asset_inventory.csv`](blender_asset_inventory.csv).

---

## 1. Current Shipped Blender GLB Packs (Measured Telemetry)

| Pack | Native `.blend` | Shipped `.glb` | Meshes | Total Polys | Total Verts | Optimization Status |
| :--- | :--- | :--- | ---: | ---: | ---: | :--- |
| **`ella_world_environment`** *(Trees, DTI Stage, Stargates, Camp, 5 Mobs, 4 Pets, 3 Mounts, Workshop, Mailboxes, Obby Gates/Boards & Obsidian Dragon Boss)* | `assets/blender/ella_world_environment.blend` (`25.2 MB`) | `public/models/ella_world_environment.glb` (`3.91 MB` / `3,914.1 KB`, WebP 78%) | 47 | 42,336 | 41,231 | **P0 + Full Expansion Pack Shipped** — 47 PBR meshes covering botanical trees, DTI stage, Stargates, camp, mobs, pets, hoverboards, workshop, mailboxes, speedrun gates/boards & Dragon Boss |
| **`ella_world_props_loot`** *(Home Interiors, Sofa/Lamps, Doors, Crafted Weapons & Relics)* | `assets/blender/ella_world_props_loot.blend` (`664.2 KB`) | `public/models/ella_world_props_loot.glb` (`1,072.4 KB`) | 33 | 26,211 | 24,640 | **Expansion Upgraded (`25 -> 33` meshes)** — added `Prop_Sofa`, `Prop_Crystal_Lamp`, `Loot_Crystal_Blade`, `Loot_Starlight_Wand`, `Loot_Aegis_Charm`, `Loot_Starlight_Potion`, `Loot_Rift_Beacon` & `Loot_Pet_Egg` |
| **`ella_dti_wardrobe`** *(Rigged Characters & DTI Closet)* | `assets/blender/ella_dti_wardrobe.blend` (`7.32 MB`) | `public/models/ella_dti_wardrobe.glb` (`6.85 MB`) | 47 | 167,499 | 105,772 | **P1 Decimation Target** — heaviest meshes can drop ~60% with Draco/Meshopt |

---

## 2. Completed P0 & Expansion Pack Upgrades (`public/models/ella_world_environment.glb`)

Built via headless Blender 5.2 LTS (`scripts/build_environment_pack.py`), sourcing real **PolyHaven CC0** botanical & architectural scans (`scratch/cc0_env_sources/`) alongside custom couture & fantasy bmesh assemblies, with `512×512` WebP-packed textures:

| Asset Node | Source | Polys | Verts | Dimensions (m) | World Usage |
| :--- | :--- | ---: | ---: | :--- | :--- |
| `Env_Tree_Blossom` | PolyHaven CC0 `jacaranda_tree` (pink tint) | 5,119 | 4,474 | `8.15 × 6.85 × 6.80` | 12 Cherry Blossom trees across Plaza & Park |
| `Env_Tree_Emerald` | PolyHaven CC0 `tree_small_02` | 3,770 | 5,093 | `5.16 × 5.39 × 6.40` | 12 Emerald deciduous park trees |
| `Env_Tree_Lilac` | PolyHaven CC0 `jacaranda_tree` (lavender tint) | 4,795 | 4,265 | `7.49 × 6.99 × 6.50` | 8 Starlight Lilac park & perimeter trees |
| `Env_DTI_Arch` | Blender 5.2 Couture bmesh | 950 | 1,252 | `7.65 × 0.95 × 7.22` | 3 fluted marble & rose-gold DTI Runway arches |
| `Env_DTI_Backdrop` | Blender 5.2 Couture bmesh | 534 | 712 | `11.62 × 0.90 × 7.20` | Pleated royal-violet velvet backstage wall |
| `Env_DTI_StarCrest` | Blender 5.2 Couture bmesh | 436 | 604 | `3.13 × 0.72 × 3.13` | Spinning faceted 8-point Diamond Star Crest |
| `Env_DTI_Chandelier` | PolyHaven CC0 `Chandelier_01` | 1,919 | 1,340 | `1.56 × 1.66 × 1.55` | 3 hanging crystal chandeliers under Runway arches |
| `Env_DTI_Spotlight` | PolyHaven CC0 `flood_lights` | 2,189 | 1,454 | `1.11 × 1.74 × 2.35` | 4 standing studio tripod spotlights flanking catwalk |
| `Env_DTI_Stanchion` | Blender 5.2 Couture bmesh | 282 | 390 | `0.44 × 2.21 × 1.10` | 14 gold stanchions with draped velvet ropes |
| `Env_DTI_Bench` | PolyHaven CC0 `painted_wooden_bench` | 374 | 315 | `2.25 × 0.96 × 1.72` | 4 VIP front-row runway audience benches |
| `Env_DTI_Lamp` | PolyHaven CC0 `street_lamp_01` | 2,699 | 1,482 | `0.77 × 0.42 × 4.20` | 2 ornate entrance promenade streetlamps |
| `Env_Campfire_Pit` | PolyHaven CC0 `stone_fire_pit` | 853 | 470 | `2.30 × 2.28 × 0.61` | Central Plaza campfire stone ring |
| `Env_Stargate_Frame` | Blender 5.2 Stargate Architecture | 1,158 | 1,440 | `6.51 × 2.60 × 6.48` | Stepped naquadah outer ring, 9 glowing chevrons & boarding ramp (both portals) |
| `Env_Stargate_InnerRing` | Blender 5.2 Stargate Architecture | 648 | 864 | `5.26 × 0.44 × 5.26` | Continuously spinning constellation glyph ring inside Stargate aperture |
| `Env_Camp_Tent` | Blender 5.2 Glamping Tent | 246 | 338 | `4.45 × 4.18 × 2.58` | Striped A-frame canvas glamping tent with cedar poles, sleeping mat & lantern |
| `Env_Camp_LogBench` | Blender 5.2 Cedar Log Bench | 102 | 156 | `4.09 × 0.64 × 0.77` | 2 sculpted natural cedar log benches around campfire |
| `Env_Camp_Lantern` | PolyHaven CC0 `Lantern_01` | 1,865 | 926 | `0.22 × 0.17 × 0.52` | Glowing brass camp lantern atop stump side-table |
| `Env_Camp_Stool` | PolyHaven CC0 `folding_wooden_stool` | 1,344 | 841 | `0.69 × 0.72 × 0.58` | Campfire wooden folding stool |
| `Env_Camp_Stump` | PolyHaven CC0 `tree_stump_01` | 1,231 | 637 | `1.78 × 1.98 × 0.72` | Rustic stump side-table beside campfire |
| `Env_Camp_Pot` | PolyHaven CC0 `pot_enamel_01` | 1,288 | 642 | `0.42 × 0.29 × 0.29` | Enamel camp kettle/pot beside fire pit |
| `Env_Camp_Boulder` | PolyHaven CC0 `rock_07` | 612 | 312 | `1.65 × 3.12 × 1.39` | 2 woodland clearing boulders framing the campsite |
| `Env_Mob_ShadowSlime` | Blender 5.2 Sculpted Monster | 1,268 | 1,180 | `1.47 × 1.48 × 1.23` | Crowned translucent jelly Shadow Slime in Crystal Cavern |
| `Env_Mob_CrystalBat_*` *(Body + WingL/R)* | Blender 5.2 Articulated Monster | 553 | 582 | `2.18 × 1.09 × 1.08` | Articulated flapping Crystal Bat in Crystal Cavern |
| `Env_Mob_RiftSpider` | Blender 5.2 Sculpted Monster | 502 | 520 | `1.97 × 1.72 × 1.08` | Chasing obsidian & crimson arachnid in Shadow Rift Arena |
| `Env_Mob_VoidWraith` | Blender 5.2 Sculpted Monster | 438 | 498 | `1.80 × 1.54 × 2.40` | Hovering cyan-eyed phantom wraith with rune halo |
| `Env_Mob_LavaGolem_*` *(Body + ArmL/R)* | Blender 5.2 Articulated Boss | 480 | 548 | `2.31 × 0.97 × 2.71` | Magma Colossus with swinging stone gauntlets in Shadow Rift |
| `Env_Pet_CloudPuppy` | Blender 5.2 Companion Pet | 1,224 | 1,152 | `0.51 × 0.96 × 0.77` | Starter Cloud Puppy following players across all zones |
| `Env_Pet_StarlightKitsune` | Blender 5.2 Companion Pet | 968 | 920 | `0.44 × 1.17 × 0.86` | Twin-tailed pastel Starlight Kitsune companion |
| `Env_Pet_ShadowKitty` | Blender 5.2 Companion Pet | 650 | 636 | `0.41 × 0.83 × 0.80` | Star-bell velvet Shadow Kitty companion |
| `Env_Pet_BabyDragon_*` *(Body + WingL/R)* | Blender 5.2 Articulated Pet | 704 | 668 | `1.19 × 1.05 × 0.78` | Articulated flapping Baby Crystal Dragon hatched from `pet_egg` |
| `Env_Mount_Hoverboard` | Blender 5.2 Plaza Mount | 120 | 160 | `0.50 × 1.24 × 0.13` | Starter Starlight Hoverboard (`1.62×` Plaza speed) |
| `Env_Mount_StarlightSkates` | Blender 5.2 Plaza Mount | 128 | 176 | `0.69 × 0.44 × 0.26` | Twin Sunset Neon Skates (`1.70×` Plaza speed) |
| `Env_Mount_CometSurfer` | Blender 5.2 Plaza Mount | 314 | 306 | `1.12 × 1.51 × 0.22` | Winged Astral Comet Surfer (`1.78×` Plaza speed) |
| `Env_Crafting_Bench` | Blender 5.2 Landmark | 390 | 430 | `2.10 × 1.00 × 1.56` | Starlight Workshop bench with anvil, floating crystal & elixirs at `(12.5, 0, 6.5)` |
| `Env_Plot_Mailbox` | Blender 5.2 Landmark | 220 | 248 | `0.39 × 0.59 × 1.52` | 6 Neighborhood Claim Mailboxes with parcel flag & brass doorbell |
| `Env_Obby_Gate` | Blender 5.2 Speedrun Arch | 412 | 472 | `4.28 × 0.72 × 3.95` | 4 Start/Finish Laser Gates across Sky Obby & Pro Obby Gauntlet |
| `Env_Leaderboard_Board` | Blender 5.2 Hall of Fame | 474 | 480 | `3.54 × 0.48 × 3.68` | 2 standing Top-5 Speedrun Leaderboard frames |
| `Env_Boss_Altar` | Blender 5.2 Arena Landmark | 238 | 264 | `3.70 × 3.70 × 2.18` | North Island 4-horn Obsidian Dragon Summoning Altar `(0, 0, -24.5)` |
| `Env_Boss_ObsidianDragon_*` *(Body + WingL/R)* | Blender 5.2 Articulated Co-Op Boss | 839 | 906 | `6.49 × 4.50 × 3.72` | Co-Op Obsidian Rift Dragon Boss with Shockwave Ring attacks |

### Obby PBR Texture Pack (`public/textures/obby/`, `450.6 KB` total `512×512` progressive JPEG)
- **`red_brick`** (PolyHaven CC0 `red_brick` diff/nor/rough) — Red & warm floating obby blocks on both Sky Obby & Pro Obby Gauntlet.
- **`hex_scifi`** (PolyHaven CC0 `hexagonal_concrete_paving` diff/nor/rough) — Sci-fi hexagonal floating tiles & blinking star tiles with animated anti-grav plasma thruster rings & perimeter holo-bars.
- **`metal_plate`** (PolyHaven CC0 `blue_metal_plate` diff/nor/rough) — Moving platforms & jump pads with counter-rotating hover thrusters.
- **`marble_tiles`** (PolyHaven CC0 `marble_tiles` diff/nor/rough) — Checkpoints, trophy pads & pastel stepping tiles.

---

## 3. Queued Optimization Backlog (P1 & P2)

### Priority P1 — High-Impact Visual Upgrades & Download-Size Reduction
1. **Avatar Wardrobe Decimation & Compression (`ella_dti_wardrobe.glb`: `6.85 MB` → `~2.2 MB`)**:
   - Largest bottlenecks measured in Blender 5.2:
     - `Body_Stylized`: `19,131` polys → target `~6,500` polys (preserve rig weights & facial topology).
     - `Body_Male`: `19,131` polys → target `~6,500` polys.
     - `Bottom_overalls` (`8,995`), `Top_tech_jacket` (`8,618`), `Bottom_tech_pants` (`7,912`), `Top_sweater` (`7,798`), `Bottom_cargo_pants` (`7,740`), `Bottom_jeans` (`7,313`), `Top_puff_blouse` (`6,143`), `Top_crop_top` (`5,759`) → decimate garments to `2,500–3,500` polys each while preserving silhouette & `SkinnedMesh` weights.
   - Enable `EXT_meshopt_compression` or `KHR_draco_mesh_compression` (`Three.js` already ships `DRACOLoader` / `MeshoptDecoder` in `addons/`) to cut initial mobile/Wi-Fi load time by >65%.
2. **Instanced Tree Rendering (`THREE.InstancedMesh` for 32+ Plaza Trees)**:
   - Currently `cloneEnvTemplate` creates 32 cloned `Group` nodes (`64` draw calls across trunk + leaf primitives). Promoting `Env_Tree_Blossom`, `Env_Tree_Emerald`, and `Env_Tree_Lilac` to `THREE.InstancedMesh` reduces all 32 trees to **6 draw calls total** (~10× CPU draw-call reduction on low-end laptops/tablets).
3. **`Prop_Double_Door` (`2×2` French Double Door in `ella_world_props_loot.glb`)**:
   - Upgrade procedural `double_door` boxes in `world3d.js` to beveled molded Blender French doors matching `Prop_Door_Frame` / `Prop_Door_Leaf`.
4. **Blossom Park Crystal Fountain (`Env_Park_Fountain`) & Park Benches / Wildflowers**:
   - Sculpt tiered baroque fountain basin + water spouts; reuse `Env_DTI_Bench` for the 4 Blossom Park benches; add PolyHaven CC0 `flower_gazania` & `shrub_01` foliage clusters around the pond and boulevards.
5. **Neighborhood Streetlamps (`24` plot corners) & Carved Directional Signposts (`10` instances)**:
   - Replace cylinder+sphere corner markers with a lightweight `~650`-poly CC0 lantern post (`PolyHaven Lantern_01`) and upgrade signposts to carved wooden frames.
6. **Obby Monuments & Portal Archway**:
   - Replace Sky Obby cylinder cup with sculpted CC0 `brass_goblets`-derived Golden Trophy (`Env_Obby_Trophy`), replace Pro Obby Diamond Crown primitives with a faceted jeweled crown monument (`Env_Gauntlet_Crown`), and sculpt runic stone arches for the Crystal Cavern Portals (`Env_Portal_Arch`).

### Priority P2 — Dungeon, Prison & Fallback Polish
7. **Crystal Cavern Pack (`ella_world_dungeon.glb`)**:
   - Faceted crystal spires & ore veins (`PolyHaven rock_face_01`), hinged wooden & golden treasure chests, and stylized rigged Shadow Slime + Crystal Bat models.
8. **Debtor's Prison Courthouse & Iron Gate**:
   - Molded stone columns, wrought-iron gate bars with ornate transom, and courtyard chore props.
9. **Classic Blocky Avatar Fallback Hats**:
   - Reuse existing `Hat_*` meshes from `ella_dti_wardrobe.glb` where appropriate to retire remaining procedural hat boxes.

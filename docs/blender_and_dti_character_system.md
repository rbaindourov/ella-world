# 👑 ELLA World 3D Stylized Blender Characters & *Dress to Impress* Wardrobe Architecture

## 1. Overview
**ELLA World** (`/ella-world`, served across `https://robertscreations.com/ella-world` and `https://winwinhost.com/ella-world` via `src/routes/ellaWorldRoutes.ts`) upgrades its player avatars from procedural Three.js cubes (`ClassicBlockyRig`) to **rigged, skeletal-animated 3D stylized characters** (`StylizedCharacterRig`) derived from the four **Ella Forensics** Blender characters (`katey_3d.glb`, `lizzy_3d.glb`, `amy_3d.glb`, `mom_3d.glb`) and expanded into a full ***Dress to Impress* (DTI)** modular glam wardrobe, live-mirror Salon UI, and West Plaza Fashion Runway with 1–5 Star Voting.

Players spawn as stylized 3D characters by default while retaining a one-click **Classic Retro Blocky** toggle (`avatarMode: 'blocky'`) in the Glam Salon Closet.

---

## 2. Headless Blender 5.2 LTS Pipeline (`ella_dti_wardrobe.glb`)

### 2.1 Source Consolidation & Payload Compression
The four raw Forensics `.glb` files totaled **30.2 MB** (`~394,000` vertices) with baked dark vertex-color (`COLOR_0`) attributes and static studio poses. The reproducible headless Blender 5.2 build script (`scratch/build_ella_dti_avatar_pack.py`) consolidates all four characters + newly sculpted DTI garments and 12 NLA skeletal animation clips into a single **4.74 MB** WebGL-ready asset:
- **Source Master Blend**: `data/blender/ella-world/ella_dti_wardrobe.blend`
- **Production Runtime Pack**: `public/ella-world/models/ella_dti_wardrobe.glb` (`4.74 MB`, `70,605` total vertices across 37 modular meshes; an active character renders only `~18k–25k` vertices).
- **Studio Preview Verifier**: `scratch/render_ella_dti_previews.py` renders 4 headless EEVEE showcase cards (`scratch/dti_studio_renders/01_ella_starlight_gown.png` – `04_lizzy_y2k_streetwear.png`) to verify skin weights, silhouette clearance, and PBR tinting.

### 2.2 Shared 25-Bone Armature (`Ella_DTI_Armature`)
All garments and body parts bind to a unified 25-bone rig (`root`, `pelvis`, `spine`, `chest`, `clavicle.L/R`, `upper_arm.L/R`, `forearm.L/R`, `hand.L/R`, `fingers.L/R`, `thumb.L/R`, `peace_fingers.R`, `neck`, `head`, `thigh.L/R`, `shin.L/R`, `foot.L/R`):
- **Coordinate Alignment**: Exported with `export_yup=True` (`+Z` forward, feet planted at `Y = 0.0`), scaled uniformly by `1.55` in Three.js (`~2.81` world units tall) to match `BlockyWorld3D` platform colliders, voice rings, and nametag sprites.
- **Weight-Bleed Isolation**: Lofted skirts and ballgowns transfer weights from an armless torso/pelvis/thigh reference mesh (`Temp_TorsoLegs_Ref`) and prune arm/finger vertex groups (`strip_vgroups_except`), preventing skirt hem spikes when arms raise during waves, dances, or peace-sign poses.

### 2.3 Modular Submesh Inventory (37 Meshes)
| Slot Prefix | Exported Submesh Names | Material & Tinting Behavior |
| :--- | :--- | :--- |
| **Base Body** | `Body_Stylized`, `Face_Brows`, `Face_SmileTeeth`, `Eye_L`, `Eye_R` | Normalized `COLOR_0` preserves dark eyelashes & rosy blush while multiplying cleanly by `skinColor`; `Eye_L/R` split sclera/pupil (`Mat_Avatar_EyeBase`) from tintable `eyeColor` iris (`Mat_Avatar_EyeIris`). |
| **Hair (`Hair_*`)** | `Hair_twin_pigtails`, `Hair_glam_ponytail`, `Hair_katey_long`, `Hair_amy_waves`, `Hair_lizzy_bob`, `Hair_mom_bun` | Grayscale-normalized highlights multiplied by `hairColor`; scrunchies/cuffs tinted by `accessoryColor`. |
| **Tops & Gowns (`Top_*`)** | `Top_glam_gown`, `Top_puff_blouse`, `Top_star_tee`, `Top_sweater`, `Top_crop_top`, `Top_crew_tee` | Tinted by `topColor` with dynamic `fabricFinish` (`matte`, `satin`, `glitter`) + gold 3D star/sash accents (`Mat_Avatar_Gold`). Selecting `Top_glam_gown` automatically hides `Bottom_*` meshes. |
| **Bottoms (`Bottom_*`)** | `Bottom_pleated_skirt`, `Bottom_ruffle_mini`, `Bottom_jeans`, `Bottom_cargo_pants`, `Bottom_overalls` | Tinted by `bottomColor` + `fabricFinish`; `Bottom_overalls` retains metallic silver buckles (`Mat_Avatar_Silver`). |
| **Footwear (`Shoe_*`)** | `Shoe_runway_heels`, `Shoe_platform_boots`, `Shoe_sneakers` | Upper/straps tinted by `shoeColor` (`Mat_Avatar_Shoe`) with crisp white soles/cuffs (`Mat_Avatar_ShoeSole`) or gold stiletto posts. |
| **Headwear (`Hat_*`)** | `Hat_crown`, `Hat_hair_bow`, `Hat_catears`, `Hat_beret`, `Hat_wizard`, `Hat_astronaut`, `Hat_tophat` | Rigidly parented to `head` bone; tinted by `hatColor` with emissive pink crystal gems (`Mat_Avatar_Gem`). |
| **Accessories (`Acc_*`)** | `Acc_star_purse`, `Acc_wings`, `Acc_sunglasses`, `Acc_choker`, `Acc_pearls` | `Acc_star_purse` parented to `hand.L`, `Acc_sunglasses` to `head`, `Acc_wings` to `chest` (with live sinusoidal wing-flutter in WebGL). |

### 2.4 Baked NLA Skeletal Animation Clips (12 Clips)
`Idle_Glam`, `Walk_Runway`, `Run_Obby` (`Shift` sprint), `Jump_Air`, `Emote_Pose` (`1`), `Emote_Peace` (`2`, articulating `peace_fingers.R`), `Emote_Dance` (`3`), `Emote_Spin` (`4`), `Emote_Wave` (`5`), `Emote_Cheer` (`6`), `Emote_Flip` (`7`), and `Emote_Sit` (`8`), blended via `THREE.AnimationMixer` with `0.18s` crossfades.

---

## 3. WebGL Client, Glam Salon UI & Fashion Runway (`public/ella-world/`)

1. **Self-Hosted Vendor Addons & Importmap (`public/ella-world/index.html`)**:
   - Maps `"three"` to `/ella-world/vendor/three.module.js` and `"three/addons/"` to `/ella-world/vendor/addons/` (`GLTFLoader.js`, `SkeletonUtils.js`, `BufferGeometryUtils.js`) with zero external CDN dependencies.
2. **Non-Modal Glam Salon Closet & Live 3D Mirror (`modern-web-guidance: dialog` + `tabs`)**:
   - Uses `<dialog id="avatar-dialog" class="glam-salon-popover" popover="manual">` docked to the right edge while `world3d.setSalonMirrorMode(true)` frames the player's 3D character in the left viewport so players can drag-orbit 360° and press keys `1–8` to preview runway poses before saving.
   - Uses a single-row horizontally scrolling tablist (`focusgroup="tablist nomemory"`, `@property --fade-start` / `--fade-end` scroll-driven mask fades + `IntersectionObserver` fallback, forced-colors-safe `::after` border underline, and `hidden="until-found"` stacked tabpanels with inner `.glam-panel-inner` wrappers).
3. **West Plaza *Dress to Impress* Fashion Runway & Star Voting (`POST /api/ella-world/runway`)**:
   - Located off West Plaza (`x = -24, z = -12..+13`, warp shortcut `F`) featuring an illuminated pink-and-gold catwalk, chasing LED edge bulbs, 3 glowing spotlight arches, a velvet backstage photo wall with spinning 3D Star Crest, theme spinner (`10` DTI themes), and live 1–5 `⭐` Star Voting synced via SSE (`event: runway`) and cluster heartbeats.

---

## 4. Multi-Primitive glTF Slot Indexing, Salon Hit-Testing & Persistent World State

### 4.1 Three.js `GLTFLoader` Multi-Primitive `Group` Slot Indexing (`public/ella-world/js/world3d.js`)
- In Three.js `GLTFLoader`, any glTF mesh node with `primitives.length > 1` (20 of the 37 exported garments/features, including `Top_glam_gown`, `Top_puff_blouse`, `Bottom_pleated_skirt`, `Bottom_overalls`, `Shoe_sneakers`, `Shoe_runway_heels`, `Hair_twin_pigtails`, `Hat_crown`, `Acc_star_purse`, and `Eye_L/R`) imports as a **`THREE.Group`** carrying the canonical glTF node name (`"Top_glam_gown"`), while its child `THREE.SkinnedMesh` primitives carry Blender datablock names (`"Top_glam_gown_Mesh_1"`, `"Mesh_Studio_Sneakers_004_1"`).
- `StylizedCharacterRig` indexes canonical wardrobe slots in `this.slotsByName` (`Map<slotName, { node, meshes }>`), registering both single-primitive `Mesh`/`SkinnedMesh` nodes and multi-primitive `Group` containers while skipping child primitive meshes whose ancestor `Group` is already registered. `applyAppearance()` toggles both `slot.node.visible` and every child primitive's `mesh.visible`, preventing multi-material garments from stacking permanently.

### 4.2 Stacked Tabpanel Hit-Testing with `hidden="until-found"` (`public/ella-world/css/styles.css`)
- All 5 Glam Salon `[role="tabpanel"]` panels share `grid-area: 1 / 1` so switching tabs never causes layout height jumps.
- Because HTML `hidden="until-found"` applies `content-visibility: hidden` (rather than `display: none`), inactive panels later in DOM order (`#glam-panel-acc`) still occupy the `grid-area: 1 / 1` hit-test box unless pointer events are disabled.
- Per `modern-web-guidance` (`search-hidden-content`), `display: none` and `visibility: hidden` are never applied to `[hidden="until-found"]`; instead, `.glam-panels-stack > [role="tabpanel"][hidden] { pointer-events: none; user-select: none; z-index: 0; }` ensures active outfit cards receive 100% of clicks while keeping browser find-in-page (`beforematch`) functional.
- Clicking any preset, garment, finish, or color swatch immediately previews on the 3D character, auto-unblocks conflicting full-body slots (e.g., switching `glam_gown` → `puff_blouse`/`crew_tee` when clicking a skirt/pants/overalls), and auto-saves via `POST /api/ella-world/avatar` (`240ms` debounce), while `↩️ Undo Changes` restores `salonOriginalAvatar`.

### 4.3 MongoDB + Disk Dual-Write World Persistence (`src/routes/ellaWorldRoutes.ts`)
- **Persistent Sandbox Builds (`ella_world_state` → `_id: 'world_blocks'`)**:
  - `POST /api/ella-world/block` persists up to `5,000` blocks (`x/z ∈ [-45, 45]`, `y ∈ [1, 60]`) synchronously to MongoDB collection `ella_world_state` (`_id: 'world_blocks'`) AND mirrors to `data/ella_world_blocks.json` on disk, with live `#build-save-status` HUD feedback (`✅ Saved World (N blocks)`).
  - Raycasting in `BlockyWorld3D._updateBuildGhost()` includes `this.colliders` so players can build directly atop plaza stages, runway decks, and sky obby platforms as well as ground and existing blocks.
- **Persistent Outfits, Accounts & Builder Position (`ella_world_state` → `_id: 'world_users'` + `POST /api/ella-world/resume`)**:
  - Saved outfits, parent settings, friend accounts, and throttled `lastPos` coordinates dual-write to MongoDB (`_id: 'world_users'`) and `data/ella_world_users.json`.
  - Browser reloads automatically call `POST /api/ella-world/resume` using `localStorage.getItem('ella_world_token')`, restoring the player's session, saved 3D outfit, `lastPos`, and world blocks without re-typing credentials.
  - `scripts/deploy-zero-downtime.sh` excludes `data/ella_world_blocks.json` and `data/ella_world_users.json` from `rsync` (seeding `ella_world_users.json` only with `--ignore-existing`), guaranteeing zero state loss across deployments and Blue/Green restarts.

---

## 5. Standalone Rust UDP Extraction (`ella-world` v0.2.0)

In November 2026, ELLA World was extracted out of the `multipleDomainCMS` Node.js webhosting runtime into its own standalone, MIT-licensed public Rust repository (`/home/ella/projects/ella-world`, binary `ella-world-server`):

1. **Complete Decoupling from Node.js / Express / SSE**:
   - Nginx on `192.168.11.32` routes `/ella-world/` and `/api/ella-world/` directly to the native `ella-world.service` systemd daemon (`127.0.0.1:8097`), completely bypassing the `multipleDomainCMS` Express Blue/Green cluster.
   - Server-Sent Events (`EventSource`) and HTTP `/sync` polling are permanently eliminated (`sseEnabled: false`, `httpPollingSyncEnabled: false`).
2. **100% UDP Voice + 20 Hz Binary Game State**:
   - **`44444/udp` (`WebRtcSfu` via `UDPMuxDefault`)**: Single-port multiplexed UDP socket carrying both 48 kHz Opus voice (`DTLS-SRTP`) and 20 Hz 28-byte binary 3D movement/pose datagrams (`SCTP/DTLS` unreliable unordered `ella-udp-state`, opcodes `0x10`/`0x11`), plus reliable discrete world events (`ella-reliable-events`).
   - **`44445/udp` (`NativeUdpServer`)**: Raw `tokio::net::UdpSocket` speaking the `EWLD` v1 binary UDP protocol with bidirectional Opus voice bridging to WebRTC browser peers.
3. **Production Deployment & Atomic State**:
   - Compiled as a static zero-glibc `x86_64-unknown-linux-musl` Rust binary at `/opt/ella-world/ella-world-server` with persistent world state stored in `/var/lib/ella-world/world_users.json` and `/var/lib/ella-world/world_blocks.json`.

---

## 6. Male Character Rig & *Oblivion* "Tech-49" Wardrobe Expansion (`v0.3.0`)

To support boy characters alongside the Ella Forensics female cast without duplicating armatures or animation tracks, [`scripts/build_male_oblivion_wardrobe.py`](../scripts/build_male_oblivion_wardrobe.py) extends `assets/blender/ella_dti_wardrobe.blend` and exports a unified **47-mesh** `public/models/ella_dti_wardrobe.glb` (`~6.91 MB`, 23 PBR materials, 12 shared NLA skeletal animations across 25 bones):

### 6.1 Shared-Skeleton Male Base Mesh (`Body_Male` & `Face_Brows_Male`)
- **`Body_Male`**: Sculpted from `Body_Stylized` in rest pose while preserving exact 25-bone skin weights and joint pivots (`1.55x` WebGL world height):
  - Flattens the female bust and smooths chest/waist topology into an athletic masculine V-taper torso, broader lats/shoulders, stronger neck, defined jaw/chin/nasal bridge, and neutralized vertex-color blush.
- **`Face_Brows_Male`**: Thicker, straighter, lower-set masculine eyebrows parented to the `head` bone (`Mat_Avatar_Brow`).
- **Runtime Toggling (`StylizedCharacterRig.applyAppearance`)**:
  - Evaluates `isMale = av.bodyType === 'male' || av.characterPreset === 'jack_tech49' || av.characterPreset === 'maverick_cabin'`.
  - Shows `Body_Male` + `Face_Brows_Male` when `isMale` (or `Body_Stylized` + `Face_Brows` when `!isMale`) and tints `Mat_Avatar_Lips` to a natural muted skin-terracotta shade on male builds.

### 6.2 *Oblivion* ("Jack Tech-49") & Casual Boy Modular Slots (10 New Meshes)
| Slot Name | Description | Materials & Binding |
| :--- | :--- | :--- |
| `Hair_tech_crop` | Short textured side-part crew cut inspired by Jack Harper (*Oblivion*); BVHTree cranium-raycast watertight quad dome + 6 feathered swept locks + Catmull-Clark `SUBSURF` | `Mat_Avatar_HairSolid`, parented to `head` |
| `Hair_maverick_quiff` | Wind-swept tousled hero quiff with full temple & sideburn coverage | `Mat_Avatar_HairSolid`, parented to `head` |
| `Top_tech_jacket` | Light-grey/white *Oblivion* Tech-49 tactical flight jacket with stand-up Mandarin collar, 3D charcoal shoulder yokes, diagonal chest harness seams, full-wrap wrist gauntlets, glowing cyan Tech-49 telemetry chest badge, sleeve recon patch, and 3D utility belt + silver buckle + hip pouches | `Mat_Avatar_Top`, `Mat_Avatar_TechDark`, `Mat_Avatar_Accent`, `Mat_Avatar_Silver` (25-bone `DataTransfer`) |
| `Top_hero_henley` | Athletic short-sleeve Henley crew tee (`bmesh.ops.bisect_plane` bicep cuffs, 3D collar/placket, full-coverage hip hem, and slim dark waist belt) | `Mat_Avatar_Top`, `Mat_Avatar_TechDark` (25-bone `DataTransfer`) |
| `Bottom_tech_pants` | Straight-cut tactical flight trousers with 3D beveled charcoal knee armor pads, cargo flaps/silver clasps, and boot-tucked ankles | `Mat_Avatar_Bottom`, `Mat_Avatar_TechDark`, `Mat_Avatar_Silver` (25-bone `DataTransfer`) |
| `Shoe_flight_boots` | Closed-toe rounded 3D tactical flight boots with chunky charcoal lug soles, front shin armor plates, and dual brushed-silver shin buckles | `Mat_Avatar_Shoe`, `Mat_Avatar_TechDark`, `Mat_Avatar_Silver` (25-bone `DataTransfer`) |
| `Acc_aviators` | Brushed-silver double-bridge teardrop Aviator sunglasses with dark mirror lenses | `Mat_Avatar_Silver`, `Mat_Avatar_AviatorLens`, parented to `head` |
| `Hat_recon_cap` | Curved-brim *Oblivion* Recon baseball cap with forward visor and cyan front mission patch | `Mat_Avatar_Hat`, `Mat_Avatar_TechDark`, `Mat_Avatar_Accent`, parented to `head` |

### 6.3 Reproducible Headless Blender Build & Verification
```bash
/home/ella/.local/bin/blender -b /home/ella/projects/ella-world/assets/blender/ella_dti_wardrobe.blend \
  -P /home/ella/projects/ella-world/scripts/build_male_oblivion_wardrobe.py
```
Automatically rebuilds all 10 male character/wardrobe meshes cleanly (idempotent cleanup on re-run), saves `assets/blender/ella_dti_wardrobe.blend`, exports `public/models/ella_dti_wardrobe.glb`, and renders 5 EEVEE studio verification cards (`jack_tech49_front.png`, `jack_tech49_three_quarter.png`, `jack_face_closeup.png`, `jack_aviators_closeup.png`, `maverick_cabin_look.png`).

### 6.4 Server Schema & Accessible Glam Salon Integration
- **`src/models.rs`**: Adds `#[serde(default = "default_body_type")] pub body_type: String` (defaulting to `"female"` so existing `world_users.json` profiles deserialize seamlessly), whitelists all new boy presets/slots/faces (`hero`), includes `body_type` in `AvatarAppearance::short_hash()` for instant 20 Hz binary digest propagation, and adds `"🛸 Sky-Tower Tech Recon"` to `RUNWAY_THEMES`.
- **Glam Salon Closet (`public/index.html`, `public/js/app.js`, `public/css/styles.css`)**: Adds **Jack Tech-49** (`jack_tech49`) and **Maverick Recon** (`maverick_cabin`) signature preset cards, an accessible `👧 Girl Build` / `👦 Boy Build` `role="group"` segmented toggle with live `aria-pressed` synchronization and `:focus-visible` outlines, and full Classic Blocky fallback parity (`ClassicBlockyRig`).
- **Admin Player Provisioning CLI ([`scripts/ella_admin_user_cli.mjs`](../scripts/ella_admin_user_cli.mjs))**: Parameterized CLI helper (`--username`, `--display-name`, `--password`, `--preset jack_tech49|maverick_cabin`) that authenticates against `/api/ella-world/admin/login`, upserts player accounts via `/api/ella-world/admin/users`, and verifies login/logout round-trip without storing credentials in source control.

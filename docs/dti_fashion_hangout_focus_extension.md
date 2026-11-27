# DTI Dress-Up, Runway & Plaza Hangout Focus Extension (Option #3 Spotlight Mode)

## 1. Purpose & Design Rationale

While the Ella's World **Godot 4 Android App (`com.robertscreations.ellaworld`)** ships with **100% cross-play parity** across all four server zones (Starlight Plaza, Sky Obby + Pro Obby Gauntlet, Crystal Cavern, Debtor's Yard, and Shadow Rift Mortal Arena + Obsidian Dragon Boss), **Option #3 — "DTI Dress-Up, Runway & Plaza Hangout Focus"** represents a dedicated, high-engagement mobile experience spotlighting the social creative heart of Ella's World:

1. **Instant Mobile Dress-Up Flow**: On phones and tablets, kids spend a large share of play sessions styling outfits, coordinating duo themes, walking the catwalk, rating friends `1★–5★`, and hanging out around the Starlight Plaza campfire, disco floor, and neighborhood villas.
2. **Cross-Playground Wardrobe Continuity**: Because `PlaygroundRegistry` (`src/playground.rs`) synchronizes each player's `AvatarAppearance` across `data/platform_registry.json` while reconciling Playground-exclusive trophies locally, outfits styled in a dedicated Glam Focus session travel seamlessly whenever a player hops between Playgrounds (`"My Playgrounds"` Switcher).
3. **Low-Cognitive-Load Touch Ergonomics**: A dedicated Fashion & Hangout spotlight mode strips away combat HUD elements (HP bars, weapon targeting, dungeon teleporters) so younger players and casual hangout sessions enjoy a calm, expressive studio-and-catwalk UI.

---

## 2. Core Capabilities of the DTI Fashion & Hangout Spotlight Mode

### 2.1 Portrait + Landscape Adaptive One-Handed Glam Closet
- **Dynamic Orientation Unlock (`DisplayServer.screen_set_orientation`)**:
  - Standard 3D Obby/Arena exploration locks to **Sensor Landscape** (`window/handheld/orientation = 4`) for dual-thumb joystick + camera + jump ergonomics.
  - Entering **Glam Studio Spotlight** optionally unlocks **Sensor Portrait / Split-Stage** layout: top 52% of the screen renders the illuminated 360° `AvatarRig` turntable pedestal (`ella_dti_wardrobe.glb` with pinch-zoom to inspect `Head`, `Torso`, or `Shoes`), while the bottom 48% presents thumb-scrollable category pills (`Hair`, `Top`, `Bottom`, `Shoes`, `Hat`, `Accessory`, `Fabric Finish: Matte / Satin / Glitter`, and `Color Swatches`).

### 2.2 Saved Lookbook Slots & Shareable Outfit Codes (`LOOK-XXXX-XXXX`)
- **Personal Lookbook Closet (`12` Named Slots)**:
  - Save complete `AvatarAppearance` snapshots locally (`user://lookbook.json`) and server-side per `GlobalUserIdentity` (e.g., *"Cotton-Candy Runway"*, *"Aurora Gala"*, *"Tech-49 Pilot"*), enabling 1-tap mid-party outfit changes during the `45s` Fashion Show countdown.
- **Friend Outfit Sharing & Twin/Duo Styling**:
  - Export an outfit into a compact deterministic preset code (`LOOK-...`) or tap **"👯 Twin With Bestie"** (with opt-in friend permission) so two friends can walk the catwalk in matching or complementary colorways.

### 2.3 Duo Catwalk Struts, Animated Judge Stickers & Podium Photo Booth
- **Duo Runway Mode (`action: "start_duo_show"`)**:
  - Extends the 4-state server runway state machine (`idle -> dress_up (45s) -> catwalk (12s/entry) -> podium (18s)`) to allow paired friend entries with synchronized side-by-side catwalk waypoints (`x = -25.1` and `x = -22.9`, `z: -8.8 -> 11.0`) and coordinated finale poses (`pose` + `peace`, `spin` + `cheer`).
- **Expressive Non-Verbal Judge Stickers**:
  - Alongside `1★–5★` star ratings, audience members tap Kid-Safe holographic reaction stickers (`💖 Slay!`, `✨ Sparkle!`, `👑 Iconic!`, `🦋 Dreamy!`, `🌈 Creative!`) that float upward from the runway stanchions over `OP_EVENT_JSON` / WSS without cluttering text chat.
- **Golden Podium Selfie & Postcard Frame**:
  - During the `18s` `podium` finale (and at the Blossom Fountain Park / Campfire photo spots in Starlight Plaza), a clean **`📸 Pose & Frame`** camera mode hides all HUD chrome, frames the 1st/2nd/3rd winners with theme banners (`RUNWAY_THEMES`), and captures local device gallery snapshots—never uploading photos to third-party servers.

---

## 3. Architecture Mapping onto Existing Ella's World Primitives

| Extension Feature | Shared Asset / Server Module | Wire / Storage Mechanism |
| :--- | :--- | :--- |
| **3D Rig, 47 Modular Slots & Finishes** | `public/models/ella_dti_wardrobe.glb` + `godot_client/scripts/AvatarRig.gd` | `POST /api/ella-world/avatar` + `UdpWorldEvent::AvatarUpdated` + cross-world `GlobalUserIdentity.avatar` |
| **Timed Runway State Machine** | `src/world.rs` (`mutate_runway`, `RunwayState`) | `POST /api/ella-world/runway` (`start_show`, `ready_show`, `vote`) + `UdpWorldEvent::Runway` |
| **Stage Architecture & Lighting** | `Env_DTI_Arch`, `Env_DTI_Backdrop`, `Env_DTI_Chandelier`, `Env_DTI_Spotlight` (`ella_world_environment.glb`) | Phase-locked via `serverTimeMs` across WebGL & Godot 4 Mobile Vulkan |
| **Safe Hangout Social Controls** | `src/playground.rs` (`InviteTicket` safety presets) + `src/world.rs` (`SAFE_QUICK_CHAT_PHRASES`) | Per-Playground `chatMode` (`normal` / `quick_only` / `muted`) & `voiceMutedByParent` |

---

## 4. Rollout Path as an In-App Mode Toggle

Because `MainApp.gd` and `WorldController.gd` already share the full `AvatarRig` and `NetworkClient` stack:
1. Add a **`👠 Runway Hangout Mode`** toggle pill on the Godot 4 Android home/top bar that hides combat/builder shortcuts, pins the **Glam Closet**, **Emote Wheel**, **Judge Stickers**, and **Runway Timer** into single-tap studio pills, and warps the player directly to the Starlight Plaza Runway Lounge (`(-19.5, 0.3, 1.5)`).
2. Add `lookbook: Vec<SavedOutfitSlot>` onto `GlobalUserIdentity` in `src/playground.rs` so saved outfits roam across every Playground alongside the active `AvatarAppearance`.

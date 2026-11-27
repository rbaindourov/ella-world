# ELLA World — Hybrid WebSocket Spine + Zero-Renegotiation UDP SFU Architecture (`v0.3.0`)

## 1. System Overview

ELLA World is architected as a self-contained Rust server (`ella-world-server`) with zero runtime dependencies on Node.js, Express, Server-Sent Events (SSE), HTTP `/sync` polling, or external databases.

In `v0.3.0`, ELLA World pairs a **Persistent Binary + JSON WebSocket Spine (`443/tcp` `TCP_NODELAY`)** with a **Zero-Renegotiation Single-Port Multiplexed UDP Fast-Lane + 8-Slot Opus SFU (`44444/udp`)**:

```mermaid
flowchart LR
  subgraph Browser["Browser Client (Three.js + WebAudio)"]
    UI["3D World + Glam Salon\nServer-Synced Clock"]
    WS["WorldSpineClient (wss:// :443)\nPresence + Digest + Binary Fallback"]
    VC["VoiceChatManager (:44444/udp)\n1 Mic + 8 Pre-Allocated Voice Slots"]
  end

  subgraph Native["Native / Bot UDP Client"]
    NC["tokio::net::UdpSocket (EWLD v1)"]
  end

  subgraph RustServer["ella-world-server (Standalone Rust v0.3.0)"]
    AX["Axum HTTP + WS Spine (:8097)\nTCP_NODELAY + /ws + /rtc/offer"]
    SFU["WebRtcSfu (:44444/udp)\nUDPMuxDefault\n8-Slot Opus SFU + 20Hz Fast-Lane"]
    RAW["NativeUdpServer (:44445/udp)\nEWLD v1 Binary UDP"]
    ENG["WorldEngine (20 Hz Tick + 3s Digest)\n12s Reconnect Grace + Atomic JSON Store"]
  end

  WS <== "1. wss:// :443 Presence, 3s Digest,\nReliable Events & Auto 20Hz Fallback" ==> AX
  VC <== "2. 44444/udp: Zero-Renegotiation Opus RTP\n+ 28B Unreliable Fast-Lane (0x10/0x11)" ==> SFU
  NC <== "44445/udp: EWLD v1 Binary\nState + Opus Voice" ==> RAW
  AX <--> ENG
  SFU <--> ENG
  RAW <--> ENG
  SFU <== "Opus Voice Cross-Bridge" ==> RAW
```

---

## 2. Hybrid Dual-Plane Reliability Design

### Plane A — Authoritative Persistent WebSocket Spine (`src/ws_spine.rs`, `wss://<host>/api/ella-world/ws`)
1. **Exclusive Presence Ownership & 12-Second Reconnect Grace**: WebRTC ICE hiccups never evict a player from `WorldEngine::players`. Only an explicit `/logout` or a WebSocket disconnect exceeding the **12-second grace window** broadcasts `player_leave`. Reconnecting within 12 seconds cancels eviction seamlessly with zero "left/joined" flicker.
2. **Self-Healing 3-Second `world_sync_digest`**: Every 3 seconds, the server broadcasts a compact authoritative digest (`stateSeq`, `serverTimeMs`, `blocksVersion`, `players` with 16-hex SHA-256 `avatarHash`, and `runway` summary). Any client that missed an outfit change, player join, or voxel block build automatically detects the version/hash delta and heals within ≤3 seconds.
3. **Instant Automatic 20 Hz Binary Fallback**: Whenever a client's UDP `44444` fast-lane is handshaking, reconnecting, or blocked by strict carrier/guest-Wi-Fi firewalls, the exact same 28-byte `OP_C2S_STATE` (`0x10`) and `OP_S2C_WORLD_STATE` (`0x11`) binary frames flow over the `TCP_NODELAY` WebSocket at 20 Hz—promoting automatically to `44444/udp` the instant live UDP pongs confirm reachability.
4. **Server-Synchronized World Clock (`serverTimeMs`)**: Clients lock `serverClockOffsetMs` via RTT-compensated `ping`/`pong` frames so moving Sky Obby platforms, disco floor tiles, and runway strip lights are phase-locked across every device.

### Plane B — Zero-Renegotiation Single-Port UDP Fast-Lane + 8-Slot Opus SFU (`src/webrtc_sfu.rs`, `44444/udp`)
1. **Single-Port `UDPMuxDefault` (`44444/udp`)**: All browser ICE connectivity checks, `DTLS-SRTP` Opus audio packets, and `SCTP/DTLS` binary game-state datagrams multiplex over **one UDP port (`44444/udp`)** with dual LAN `typ host` + WAN `typ srflx` ICE candidates.
2. **Pre-Allocated 8-Slot Speaker Pool (Zero SDP Renegotiation)**: During the initial `POST /api/ella-world/rtc/offer` handshake, the browser offers 1 `sendonly` Opus mic transceiver plus **8 `recvonly` Opus speaker slot transceivers**, which `WebRtcSfu` binds immediately to `ella_slot_0`..`ella_slot_7` (`TrackLocalStaticRTP`). When friends join, speak, or leave, the Rust SFU dynamically maps active speakers to free slots in user-space RTP memory and emits a lightweight `voice_slot_map` JSON event—**zero mid-game `setRemoteDescription` renegotiations, zero ICE restarts, and zero transceiver envelope errors**.
3. **Zero Head-of-Line Blocking (`ella-udp-state`)**: Movement, animations, runway poses, and speaking flags travel over `ordered: false, maxRetransmits: 0` SCTP/UDP datagrams when healthy.

---

## 3. Binary State Codec (`src/udp_wire.rs`)

Shared identically across WebRTC `ella-udp-state` (`44444/udp`), WebSocket Binary Fallback (`443/tcp`), and Native `EWLD` (`44445/udp`):

### Client-to-Server `OP_C2S_STATE` (`0x10` — Fixed 28 Bytes)

| Byte Offset | Type | Field | Description |
| :--- | :--- | :--- | :--- |
| `0` | `u8` | `opcode` | `0x10` (`OP_C2S_STATE`) |
| `1..5` | `u32 LE` | `seq` | Shared monotonic sequence number across WS & UDP |
| `5..9` | `u32 LE` | `client_time_ms` | Client monotonic ms timestamp echoed back for live RTT calculation |
| `9..13` | `f32 LE` | `x` | World X coordinate (`[-80.0, 80.0]`) |
| `13..17` | `f32 LE` | `y` | World Y elevation (`[-20.0, 80.0]`) |
| `17..21` | `f32 LE` | `z` | World Z coordinate (`[-80.0, 80.0]`) |
| `21..25` | `f32 LE` | `ry` | Character yaw rotation in radians |
| `25` | `u8` | `anim` | `0=idle`, `1=walk`, `2=run`, `3=jump` |
| `26` | `u8` | `emote` | `0=none`, `1=pose`, `2=peace`, `3=dance`, `4=spin`, `5=wave`, `6=cheer`, `7=flip`, `8=sit` |
| `27` | `u8` | `flags` | Bit `0`: `muted`, Bit `1`: `speaking`, Bits `2..7`: `zone` (`0 = ZONE_PLAZA`, `1 = ZONE_DUNGEON`, `2 = ZONE_PRISON`) |

### Server-to-Client `OP_S2C_WORLD_STATE` (`0x11` — Zone-Partitioned Compact Snapshot)

- Header (`11 bytes`): `[0x11][server_tick: u32 LE][echo_client_time_ms: u32 LE][player_count: u16 LE]`
- Per Remote Player (`1 + id_len + 19 bytes`): `[id_len: u8][id_utf8][x: f32 LE][y: f32 LE][z: f32 LE][ry: f32 LE][anim: u8][emote: u8][flags: u8]`
- **Zero-Byte-Overhead Zone Partitioning**: `WorldEngine::rebuild_pre_encoded_tick` buckets pre-encoded player frames into `entries_by_zone` (`ZONE_PLAZA = 0`, `ZONE_DUNGEON = 1`, `ZONE_PRISON = 2`). Peers only receive 20 Hz movement frames for remote players sharing their active zone, while `playersMap` and 8-slot Opus Party Voice remain connected across zones (unless muted by parental controls).

### Native Raw UDP Envelope (`44445/udp`)

Raw UDP datagrams on `44445/udp` prefix a 6-byte magic header `[b"EWLD"][version = 1][pkt_type]` supporting `OP_HELLO` (`0x01`), `OP_WELCOME` (`0x02`), `OP_C2S_STATE` (`0x10`), `OP_S2C_WORLD_STATE` (`0x11`), `OP_C2S_PING` (`0x12`), `OP_S2C_PONG` (`0x13`), `OP_VOICE_OPUS` (`0x20` cross-bridged into the 8-slot WebRTC SFU), and `OP_EVENT_JSON` / `OP_EVENT_ACK` (`0x30`/`0x31`).

---

## 4. Crystal Cavern Dungeon, Persistent Economy & Scam-Proof P2P Trading (`src/economy.rs`)

### 4.1 Multi-Zone Portal & Active-Only 4 Hz Dungeon Simulation
- **Portals**: Players transition between **Starlight Plaza (`ZONE_PLAZA = 0`)** via the East Plaza portal at `(18, 0, 16)` (or HUD shortcut `G`) and the **Crystal Cavern (`ZONE_DUNGEON = 1`)** return portal at `(0, 0, 28)`. Only Plaza coordinates persist to `UserAccount::last_pos`.
- **Dungeon Entities (< 128 KiB Server RAM)**:
  - **6 Crystal Veins** (`crystal_1`..`crystal_6`, 14s respawn) + **2 Wooden Chests** (28s respawn) + **2 Key-Locked Golden Vault Chests** (consume `1x dungeon_key`, 65s respawn).
  - **8 Patrolling Mobs**: 5 Shadow Slimes (`2 HP`) and 3 Crystal Bats (`3 HP`) defeated via **Jump-Stomp** (`2 dmg` + bounce) or **Sparkle Blast** (`E`, `1 dmg`).
  - **4 Hz Active-Only Tick (`tick_dungeon_if_occupied`)**: Runs every 250ms (`tick_seq % 5 == 0`) **only** when `>= 1` player occupies `ZONE_DUNGEON`, broadcasting compact `DungeonMobs` / `DungeonSync` updates exclusively to dungeon occupants.

### 4.2 18-Item Catalog, Legendary Ray Gun, Home Interiors & Exclusive Glam Ownership Reconciliation
- **Materials, Keys & 3D Home Trophy Relics**: `star_shard`, `slime_gel`, `moon_opal`, `dungeon_key`, `crown_relic`, `geode_lamp`, `slime_lava_lamp`, `bat_plushie`, `royal_chalice`, and `dragon_egg_relic`.
- **Consumables (`POST /api/ella-world/economy/use`)**: `speed_potion` (`+35%` sprint for 45s), `jump_potion` (`+30%` jump boost for 45s), and `prism_block` (places a glowing `#67e8f9` voxel block at the player's Plaza feet).
- **Dungeon-Exclusive 3D Glam Relics & First Legendary Weapon**:
  - `cosmetic_hat_shadow_hood` → `hat = "ninja"`
  - `cosmetic_acc_star_lantern` → `accessory = "star_lantern"`
  - `cosmetic_hat_obsidian_crown` → `hat = "obsidian_crown"`
  - `cosmetic_acc_crystal_wings` → `accessory = "crystal_wings"`
  - `legendary_ray_gun` (*legendary*, `10%` Golden Vault Chest jackpot) → `accessory = "ray_gun"`, granting a `14.5m` `3 HP` ranged plasma beam in Crystal Cavern (`POST /api/ella-world/dungeon/attack` with `mode: "raygun"`) plus harmless Starlight Plaza laser light-show beams (`POST /api/ella-world/raygun` / `"raygun_zap"`).
- **2-Story House Collisions, Hinged Doors/Windows & Anti-Duplication Loot Decorating (`POST /api/ella-world/block`)**:
  - `VoxelBlock` supports backward-compatible `kind`, `rot` (`0..3`), `open` (`action: "toggle"` for `door` & `window`), and `lootItemId` (`kind: "loot_decor"` with atomic `-1` placement deduction, owner-only removal, and `+1` refund on pickup or `POST /api/ella-world/admin/world/reset`). See [`docs/housing_interiors_and_legendary_loot.md`](housing_interiors_and_legendary_loot.md).
- **Server-Side Ownership Enforcement (`reconcile_equipped_cosmetics`)**: Verified on `POST /api/ella-world/avatar`, `loot_decor` pedestal placement, and P2P trade completion. Trading away or displaying the last copy of an equipped dungeon relic or Legendary Ray Gun automatically unequips it back to `"crown"` / `"none"` and broadcasts `avatar_updated`.

### 4.3 Scam-Proof Two-Sided Atomic Live P2P Trading
- **No NPC Shop / Pure Player Economy**: Friends exchange farmed Coins, Keys, Potions, Relics, and Exclusive Cosmetics directly via live P2P trade invites (`/api/ella-world/trade/invite`, `/respond`, `/offer`, `/lock`, `/confirm`, `/cancel`).
- **Anti-Bait Lock Reset**: Modifying coins or item stacks on either side increments `offer_revision` and resets `locked = false` and `confirmed = false` on **both** sides so neither player can confirm an altered offer without re-inspecting and re-locking.
- **Single-Guard Atomic Swap**: When the second party confirms, `WorldEngine::trade_confirm` acquires a single `users_cfg.write().await` guard, locates both accounts via disjoint mutable slice splits (`split_at_mut`), re-verifies exact coin balances, item stack quantities, and `MAX_ITEM_STACK` (`999`) bounds, transfers assets with zero duplication or loss, reconciles equipped cosmetics, and persists atomically to `world_users.json`.

---

## 5. Atomic Disk Persistence (`src/persistence.rs`)

All persistent state lives in `ELLA_DATA_DIR` (e.g., `/var/lib/ella-world/`):
- `world_users.json`: Whitelisted friend accounts, outfits, persistent `coins` & `inventory` stacks, master admin password, chat filter settings (`chatFilterEnabled`, `blockedWords`), and throttled Plaza `lastPos` coordinates (`chmod 0600` on Unix).
- `world_blocks.json`: Monotonically versioned co-op voxel blocks (`blocksVersion` + `blocks[]`).
- `world_chat.jsonl`: Append-only newline-delimited JSON history of delivered in-world chat messages; the most recent 60 messages hydrate on startup so chat survives restarts and feeds the Parent Admin Chat Viewer (`GET /api/ella-world/admin/chats`).
- `world_chat_blocked.jsonl`: Append-only audit trail of rejected chat attempts (profanity/blocked-word matches and anti-flood rate blocks) inspectable in the Parent Admin Dashboard.

JSON state writes serialize to `.tmp.<pid>` in the same directory and execute an atomic POSIX `rename()` so power loss or process restarts never corrupt world files. Chat `.jsonl` logs append line-by-line via `OpenOptions::append(true)`.

---

## 6. Native Rust TLS (`ELLA_TLS_CERT` / `ELLA_TLS_KEY`)

When `ELLA_TLS_CERT` and `ELLA_TLS_KEY` are configured (e.g., for dedicated deployment on `192.168.11.24:4444`), `start_all_servers` wraps the TCP accept loop in `tokio_rustls::TlsAcceptor` (`ring` crypto provider, `http/1.1` ALPN + `TCP_NODELAY`) and `hyper::server::conn::http1` with `.with_upgrades()` so HTTPS static assets, REST APIs, and `wss://<host>:4444/api/ella-world/ws` run natively without Nginx or port `443`.

---

## 7. Cryptographic Session Token Binding & Live Session Revocation (`src/auth.rs`, `src/world.rs`)

To ensure parent password rotations and emergency sign-outs take effect **instantaneously** across persisted `localStorage` tokens (`ella_world_token`) as well as already-open WebSocket, WebRTC UDP SFU, and Native UDP connections:

1. **Epoch & Password-Bound `TokenClaims` (`src/auth.rs`)**:
   - Every HMAC-SHA256 session token binds `exp` (configurable via `ELLA_TOKEN_TTL_SECS`, default `43200s` / `12h`), global `sessionEpoch` (`UsersConfigFile::session_epoch`), per-account `userTokenEpoch` (`UserAccount::token_epoch`), and `pwdSig` (a 64-bit truncated HMAC-SHA256 over `"ewld-pwd-v1|<subject_lower>|<current_password>"` signed with `ELLA_HMAC_SECRET`).
   - Legacy tokens missing `exp`, `sessionEpoch`, `userTokenEpoch`, or `pwdSig` are unconditionally rejected (`401 Unauthorized`) across `/api/ella-world/resume`, `/api/ella-world/ws`, `/api/ella-world/rtc/offer`, Native UDP `OP_HELLO`, and all authenticated HTTP action endpoints.
2. **Automatic Revocation on Password Rollover (`POST /api/ella-world/admin/users`)**:
   - Whenever an existing user's password changes, the server increments `UserAccount::token_epoch`, persists `world_users.json` atomically, tears down active WebRTC SFU PeerConnections (`WebRtcSfu::cleanup_sessions_for_username`) and Native UDP endpoints (`NativeUdpServer::evict_sessions_for_username`), broadcasts `SessionRevokeSignal::Username`, sends `{"type":"session_revoked","reason":...}` + WebSocket close, and evicts the player immediately with **zero 12-second grace window**.
3. **One-Click Global & Per-User Token Expiration (`POST /api/ella-world/admin/sessions/revoke`)**:
   - **Global Revoke (`{}`)**: Increments `UsersConfigFile::session_epoch` and every user's `token_epoch`, disconnects all WebSocket, WebRTC SFU, and Native UDP sessions immediately, and returns a freshly minted `adminToken` for the parent dashboard.
   - **Per-User Revoke (`{ "username": "<id>" }`)**: Increments that account's `token_epoch` and kicks only that player's active transports.
   - **CLI & UI**: Accessible both from the Parent Admin Panel (**"🔒 Expire All Logged-In Tokens"** and per-row **"Expire"** buttons) and via `node scripts/ella_admin_user_cli.mjs --revoke-all` / `--revoke-user <username>`.
   - **Emergency HMAC Signing Key Rotation & Production Verifier**: `node scripts/rotate_prod_hmac_secret.mjs` losslessly archives `/etc/systemd/system/ella-world.service.d/override.conf` to `/var/lib/ella-world/`, rotates `ELLA_HMAC_SECRET` to a fresh 256-bit key, restarts `ella-world.service`, and confirms `HTTP 401` on prior tokens; `node scripts/verify_prod_http_and_auth.mjs` verifies live HTML, static CSS/ES-module asset integrity (`device_sig.js`, `ws_spine.js`, `voice.js`, `world3d.js`), stale-token `401` rejection, and `/logout` machine-seal preservation.
   - **Sign-Out Token Eviction + Machine-Seal Preservation**: `POST /api/ella-world/logout` tears down server-side WebRTC SFU, Native UDP, and WebSocket sessions while intentionally omitting `Clear-Site-Data` so `__Host-ewld_machine_seal` / `ewld_machine_seal` and client `IndexedDB`/`localStorage` vaults survive sign-outs; browser clients purge only `localStorage['ella_world_token']` on sign-out or `401`/`403` revocation.

---

## 8. Evasion-Resistant Chat Sanitization, Negative-Balance Fines, 3D Debtor's Prison & Parental Controls (`src/world.rs`, `src/economy.rs`)

### 8.1 Evasion-Resistant Cursing Sanitization (`find_blocked_word`)
- Catches spaced single-letter evasion (`"f u c k"`), punctuation-split evasion (`"s.h.i.t"`), multi-word toxic phrases (`"kill yourself"`, `"k1ll y0urself"`), repeated-letter elongation (`"fuuuck"`), and extended leetspeak (`1=i`, `3=e`, `4=a`, `5=s`, `0=o`, `7=t`, `8=b`, `@=a`, `$=s`, `!=i`) while enforcing word-boundary guards on short roots (`<= 4` chars) and skipping pure numeric tokens so clean words (`"class"`, `"pass"`, `"grass"`, `"hello"`, `"scrap"`, `"455 coins"`) never false-positive.

### 8.2 Signed Debt Economy (`i64`) & Escalating Habitual Fines
- `UserAccount::coins` and `PlayerEconomySnapshot::coins` use signed `i64` balances.
- Each blocked profanity attempt increments `UserAccount::violation_strikes` and deducts an escalating Starlight Coin fine (`calculate_escalating_fine(base, strike)`: `-15`, `-35`, `-75`, `-150`, `-300`, `-500` 🪙 for `profanityFineBase = 15`), allowing habitual violators to drop into negative debt.
- Every violation appends full judicial telemetry (`strikeNumber`, `fineCoins`, `balanceAfter`, `sentencedToPrison`) to `world_chat_blocked.jsonl` and broadcasts a public **Judge Starlight** court notice in world chat.

### 8.3 3D Debtor's Prison (`ZONE_PRISON = 2`) & Community Service (`POST /api/ella-world/prison/work`)
- Players whose balance drops `<= debtPrisonThreshold` (default `-50 🪙`) **or** whose strikes reach `>= prisonStrikeThreshold` (default `3`)—or who are manually sentenced by a parent—are immediately warped into **Debtor's Prison (`ZONE_PRISON = 2`)**, an enclosed `28m × 28m` 3D courtyard (`[-13.2, 13.2]` server-clamped bounds).
- While imprisoned (`in_debtors_prison == true`), the server blocks portal warps to Plaza/Dungeon, voxel block building (`/block`), item consumables (`/economy/use`), runway theme/vote mutations, and P2P trading (`/trade/*`).
- Prisoners clear their sentence either by completing **6 glowing 3D community-service stations** (`prison_rock_1`, `prison_rock_2`, `prison_sweep_1`, `prison_sweep_2`, `prison_lantern_1`, `prison_lantern_2`, 6s recharge; each awards `+5 🪙` strictly capped at `0 🪙` with zero positive-coin farming exploit and decrements `prison_tasks_remaining`) and walking through the unlocked Release Gate Arch at `(0, 0, 11.5)` once `coins >= 0 && prison_tasks_remaining == 0`, **or** by receiving an instant **🕊️ Parent Pardon** from the Parent Admin Panel / CLI.

### 8.4 Comprehensive Parental Controls (`POST /api/ella-world/admin/moderate` & `/admin/settings`)
- **Per-Child Controls**: Server-enforced Voice Mic Mute (`voiceMutedByParent` drops Opus RTP/UDP frames inside both `WebRtcSfu` and `NativeUdpServer`), Text Chat Policy (`normal`, `quick_only` allowing only `SAFE_QUICK_CHAT_PHRASES`, or `muted`), Build Lock (`canBuild`), Trade Lock (`canTrade`), Account Pause/Grounding (`suspended` returning `403 Forbidden` + instant live session kick), `+ / -` Coin Adjust, Send to Prison, Parent Pardon, and Reset Strikes.
- **Global Safety Switches**: World Pause Lock (`worldLocked`), Automatic Bedtime Curfew Window (`curfewEnabled`, `curfewStart`..`curfewEnd` in 24h local server time), Global Voice Toggle (`globalVoiceEnabled`), Global Chat Policy (`globalChatMode`), and configurable `profanityFineBase` / `debtPrisonThreshold` / `prisonStrikeThreshold`.

### 8.5 Sub-Admin Friend Moderator Role (`role: "subadmin"`, `GET /subadmin/state`, `POST /subadmin/moderate`)
- Parents can promote or demote any account between `friend`, `subadmin`, and `host` in the Parent Admin Panel or via `node scripts/ella_admin_user_cli.mjs --username <u> --set-role subadmin`.
- A `subadmin` account unlocks the **🛡️ Mod Friends** modal (`#subadmin-dialog`) in the HUD to manage **`friend`** accounts' Gold (`coinDelta` / `setCoins`), Inventory (`gift_item` with `qty`, `setQty`, and `allowClone`), Customizable Features (`displayName` and `avatar` look presets), and Moderation Controls (Mic Mute, Text Chat Mode, Build Lock, Trade Lock, and live `kick_session`).
- **Strict Parent-Only Boundaries**: Sub-Admins cannot self-moderate, cannot moderate `host` or other `subadmin` accounts, cannot change passwords/curfews/machine-signature overrides or pardon Debtor's Prison, and automatically lose Sub-Admin access (`403 Forbidden`) if their own account is paused or in Debtor's Prison.

### 8.6 Parent-Exclusive Item Gifting & Authorized Cloning (`POST /admin/gift-item`, `POST /economy/clone`)
- **Parent-Exclusive Catalog (`parent_exclusive: true`)**: Mythic items (`parent_golden_trophy`, `parent_guardian_crown`, `parent_starlight_heirloom`) never spawn in Crystal Cavern mobs, chests, or veins (`roll_mob_loot` / `roll_chest_loot`) and can only be gifted by a Parent in **Parent Admin** or via `--gift-item <itemId> --qty <n> --allow-clone true|false` in `scripts/ella_admin_user_cli.mjs`.
- **Authorized Item Cloning (`UserAccount::cloneable_items`)**: When Parents enable **🧬 Allow Clone** on a gifted item, the player's **🎒 Backpack** renders a **`🧬 Clone (+1)`** button (`POST /api/ella-world/economy/clone`), which verifies ownership + clone permission + non-prison/non-suspended status before minting `+1` copy.

---

## 9. Cryptographic Machine Digital Signatures & Anti-Alt-Evasion (`src/device_sig.rs`, `public/js/device_sig.js`)

To prevent penalized players from signing out, opening an Incognito window, or launching an alternate browser on the same computer to log into a friend's unpunished account:

### 9.1 Three-Layer Machine Identification
1. **Layer 1 — Server-Signed Cryptographic Machine Seal (`__Host-ewld_machine_seal` / `ewld_machine_seal`)**:
   - Signed with constant-time `HMAC-SHA256` (`subtle::ConstantTimeEq`) using `ELLA_HMAC_SECRET` (`purpose: "machine_seal_v1"`).
   - Delivered in an `HttpOnly; SameSite=Lax; Path=/; Max-Age=31536000` cookie (`Secure` + `__Host-` prefix when TLS is active) AND stored redundantly in client `localStorage` + `IndexedDB` (`ella_world_device_vault`), automatically self-healing if a user deletes one client store in DevTools.
2. **Layer 2 — Exact Hardware Digital Signature (`hw_exact_sig`)**:
   - Keyed `HMAC-SHA256` digest of canonical WebGL GPU renderer/capabilities, 2D Canvas raster digest, time-bounded `OfflineAudioContext` DSP digest, orientation-invariant screen bounds, pixel ratio, CPU core count, touch points, OS family, and timezone. Catches Incognito/Private windows and cookie wipes where no seal is presented.
3. **Layer 3 — Cross-Browser Physical Machine + Subnet Signature (`hw_xbrowser_sig` & `net_os_sig`)**:
   - Excludes browser-engine-specific Canvas/Audio/colorDepth perturbations and canonicalizes ANGLE vs. native WebGL GPU strings (`"ANGLE (Apple, Apple M2...)"` -> `"apple m2"`), combined with exact RFC1918 LAN host IP (`/32`) or public WAN `/24` (IPv4) / `/64` (IPv6) subnet (`X-Forwarded-For` trusted **only** when the TCP peer is loopback `127.0.0.1` / `::1`). Catches Chrome -> Safari/Firefox/Edge switches on a penalized computer while keeping unpenalized sibling devices distinct.
   - **Sibling Zero-Collision Guarantee**: When a request carries a valid cryptographic seal for an already-registered sibling device (`dev_B`), Layer 1 resolves `dev_B` directly and never falls through to fuzzy hardware/subnet matching—preventing false positives between two identical-model family iPads or MacBooks on the same home Wi-Fi.

### 9.2 Cross-Account Penalty Lock, Live Tab Eviction & Parent Shared-PC Override
- **Triggering Penalties (`active_penalties_for_user`)**: Any active penalty on a user linked to a non-exempt machine (`inDebtorsPrison`, `suspended`, `coins < 0`, `voiceMutedByParent`, `chatMode != "normal"`, `!canBuild`, or `!canTrade`) locks that machine exclusively to the penalized account.
- **403 Login/Resume Block + Audit Trail**: Attempting to log in or `/resume` as any other friend account on that locked machine returns `403 Forbidden`, leaves the friend's account completely unpenalized, and appends a forensic record to `world_device_evasion.jsonl`.
- **Instant Multi-Tab Eviction (`SessionRevokeSignal::DeviceOtherUsers`)**: Whenever an account incurs a penalty (via profanity fine/Debtor's Prison sentence in `submit_chat` or parent action in `admin_moderate_user`), `evict_other_accounts_on_user_machines` immediately revokes and disconnects any other accounts' open WebSocket, WebRTC SFU, and Native UDP sessions on that machine (`TokenClaims::device_id` + `PlayerRuntimeState::device_id`).
- **Parent Controls (`POST /api/ella-world/admin/devices` & CLI)**: Parents can mark a shared household computer as **🏠 Shared PC** (`trustedShared: true`) so siblings can play independently, **🚫 Ban PC** (`banned: true`), or **🧹 Unlink** historical accounts in the Parent Admin Panel or via `node scripts/ella_admin_user_cli.mjs --devices` / `--trust-device` / `--ban-device` / `--unlink-device`.

---

## 10. Top-Layer Modal Glam Studio & Dedicated 3D Fitting Stage (`public/index.html`, `public/js/world3d.js`, `public/js/app.js`)

- **True Modal Input Isolation (`avatarDialog.showModal()`)**: Opening the Glam Salon (`👗 Glam` or shortcut `C`) promotes `<dialog id="avatar-dialog" class="glam-studio-modal">` into the browser Top Layer above `dialog::backdrop`, natively marking `#canvas-container` and `#game-hud` `inert` while `BlockyWorld3D.setSalonMirrorMode(true)` clears active movement/drag states.
- **Character Rendered In Front of Modal Screen (`GlamSalonStage3D`)**: A dedicated transparent-alpha Three.js `WebGLRenderer` inside `#glam-stage-pane` renders the player's `WorldAvatar` (`studioMode: true`, `cloneGeometries: true`) on an illuminated Starlight Fitting Pedestal directly in front of the dimmed/blurred backdrop (left/right split on `> 900px`, top-stage + bottom-sheet on `<= 900px`), pausing its `requestAnimationFrame` loop whenever closed.
- **Interactive Dress-Up Tooling**: Includes 360° drag turntable rotation, wheel/two-finger pinch zoom, accessible `focusgroup="toolbar wrap"` camera framing (`Full Fit`, `Head`, `Outfit`, `Shoes`), in-modal `0–8` pose/dance auditioning, build-aware `🎲 Surprise Me!` randomization, `👁️ Compare` before/after toggle, live Runway Theme pill, and `🎒 Bag` → `👗 Equip in Glam` relic framing. See [`docs/blender_and_dti_character_system.md`](blender_and_dti_character_system.md#7-top-layer-modal-glam-studio--glamsalonstage3d-publicjsworld3djs-publicjsappjs).

---

## 11. Clean Fullscreen HUD Mode, Panel Toggles & Miniaturized Voting Pill (`public/index.html`, `public/css/styles.css`, `public/js/app.js`)

- **Default Clean Fullscreen HUD (`#game-hud.is-fullscreen-hud`)**: On world entry (`enterGameWorld`), `#game-hud` defaults to Clean Fullscreen HUD mode so the 3D viewport is unobstructed—displaying only the translucent bottom-right **Friends Text Chat** (`.hud-chat-panel` with a single-row horizontally scrollable `.quick-reactions` strip) and a minimal top-right corner pill (`.hud-mode-corner`) with **`🎛️ Controls`** (`#toggle-hud-mode-btn`, shortcut `\`) and **`⛶`** (`#toggle-browser-fullscreen-btn` for native browser `requestFullscreen()`).
- **Controls Mode & Per-Panel Toggles**: Clicking **`🎛️ Controls`** (or pressing `\`) expands `#hud-top-bar` with a **`Panels:`** toggle strip (`aria-pressed`) so players can independently show or hide:
  - **`🎤 Friends`** (`#toggle-friends-panel-btn` → `.hud-left-panel`, default off)
  - **`👗 Voting`** (`#toggle-runway-bar-btn` → `#runway-hud-bar`, default off)
  - **`💃 Emotes`** (`#toggle-emote-bar-btn` → `.emote-bar`, default off)
  - **`🎮 Touch Pad`** (`#toggle-touch-controls-btn` → `.touch-controls`, auto-enabled on `(pointer: coarse)` touchscreens, default off on desktop)
  - **`💬 Chat`** (`#toggle-chat-panel-btn` → `.hud-chat-panel`, default on)
- **Miniaturized Runway Voting Pill (`#runway-hud-bar`)**: Stacked cleanly below `#hud-top-bar` inside `.hud-top-stack` (preventing vertical overlap) as a compact single-row pill featuring the truncated theme name, dice button (`🎲`), friend selector, compact `1★`–`5★` rating buttons, and compact leader badge (`🏆 <name> (<avg>★)`).

---

## 12. 2.5× Larger Starlight Plaza, Walkable Single/Double Doors & Starlight Sky Gauntlet (`src/udp_wire.rs`, `src/models.rs`, `src/world.rs`, `public/js/world3d.js`)

- **2.5× Expanded Starlight Plaza (`210 × 210`)**:
  - Ground collider expanded from `84 × 84` (`[-42, 42]`) to `210 × 210` (`[-105, 105]`), preserving all original central landmarks & saved blocks intact.
  - Backend bounds expanded: binary UDP state clamps widened to `[-120, 120]` (`src/udp_wire.rs`), saved `last_pos` widened to `[-110, 110]`, voxel building unlocked across `x/z ∈ [-100, 100], y ∈ [1, 80]` with a `12,000`-block cap (`src/world.rs`).
  - Outer districts added via paved Starlight Boulevards & Central Plaza 3D signposts: 6 labeled neighborhood build plots (**Sunset Build Meadow** East & **Westside Villa Estates** West), **Blossom Fountain Park** (South glowing koi pond, tiered crystal fountain & benches), and player-following directional shadow frustum for crisp shadows across the entire map.
- **Walkable Single & 2×2 Double Doors (`door`, `double_door`)**:
  - Open doors register in `openDoorPortals` so walking through an open doorway uses a narrowed `0.26` effective horizontal radius + gentle lateral centering assist—eliminating adjacent-wall snagging on `1×2` doors while keeping closed doors and walls solid.
  - Pressing `E` toggles nearby doors/double-doors/windows even while **Build Mode** is active.
  - Added `double_door` (`2×2` counter-swinging French double door) using `block_occupied_cells` (`src/models.rs`) so placement, overlap, toggling, and removal work seamlessly from either half.
- **Second, More Difficult Obby — Starlight Sky Gauntlet (Pro Obby)**:
  - Located in the expanded Southeast sector (`x ∈ [48, 92], z ∈ [48, 92], y ∈ [0, 44]`) with 5 checkpoints and a spinning **Diamond Crown** summit: narrow zig-zag steps, faster split-axis movers, server-synced blinking/disappearing tiles (`type: 'blinking'`), chained high-arc bounce pads, and jump-over spinning sweeper beams (`sweeperHazards`).
  - Independent checkpoint tracks for Sky Obby (`R` / `🏃‍♀️ Obby`) and Pro Obby (`T` / `🔥 Pro Obby`), plus a server-authoritative **+25 🪙** completion reward (`POST /api/ella-world/chat` with `isTrophyWin: true, trophyTier: "hard"`, 60-second anti-farm cooldown).

---

## 13. Sculpted 3D Enemies, Prebuilt Village Homes & Shadow Rift Mortal Arena (`scripts/build_environment_pack.py`, `src/persistence.rs`, `src/economy.rs`, `src/world.rs`, `public/js/world3d.js`, `public/js/app.js`)

- **Sculpted & Articulated Blender 5.2 3D Monsters (`ella_world_environment.glb`)**:
  - Replaced primitive geometric dungeon mobs with 5 sculpted/articulated meshes in `assets/blender/ella_world_environment.blend` (`30` meshes, `35,655` polys total):
    - `Env_Mob_ShadowSlime` (`shadow_slime`): translucent purple slime with suspended crystal core & glowing eyes (squish/hop animation).
    - `Env_Mob_CrystalBat_Body` + hinged `WingL`/`WingR` (`crystal_bat`): fanged cavern bat with crystal crest & articulated wing-flapping (`±0.34, 0.68, 0`).
    - `Env_Mob_RiftSpider` (`rift_spider`): 8-legged obsidian arachnid with glowing crimson eyes & segmented abdomen.
    - `Env_Mob_VoidWraith` (`void_wraith`): tattered floating cloak, horned crown & hovering amethyst runes.
    - `Env_Mob_LavaGolem_Body` + hinged `ArmL`/`ArmR` (`lava_golem`): cracked volcanic basalt juggernaut with molten magma core & swinging stone gauntlets (`±0.84, 1.72, 0`).
- **3 Furnished, Walkable Prebuilt Village Homes (`prebuilt_village_blocks()`)**:
  - Seeded idempotently via `BlocksDataFile.prebuilt_village_v1_seeded` (`prebuiltVillageV1Seeded` in `data/blocks.json`) without overwriting any existing player-built voxels, leaving 3 neighborhood build plots (`Plot #1 (52, -26)`, `Plot #4 (82, 20)`, `Plot #6 (-64, 32)`) completely open:
    1. **🌸 Rosewater Grand Villa** (`Plot #2` around `(82, -26)`): 2-story pastel villa with walkable `2×2` `double_door` entrance, `window`s, furnished living room/kitchen/bath, `stairs` to 2nd-floor bedroom, and balcony `door`.
    2. **🌊 Starlight Beach Cottage** (`Plot #3` around `(52, 20)`): cozy wraparound-porch cottage with walkable `door` entrance, `fence` porch rails, kitchen/dining nook, bedroom, and garden lantern arch.
    3. **🌲 Whispering Pine Woodland Chalet** (`Plot #5` around `(-64, -16)`): rustic brick & timber chalet with walkable `2×2` `double_door`, fireplace hearth, loft `stairs`, and furnished kitchen/bath/bedroom.
- **Second Stargate Portal & Mortal Combat Zone — Shadow Rift Arena (`ZONE_ARENA = 3`)**:
  - **Southwest Starlight Plaza Bloodmoon Stargate** (`(-22, 0, 28)`, crimson/obsidian GLSL wormhole shader) + HUD warp button (`⚔️ Shadow Rift`, shortcut `K`).
  - **Server-Authoritative `ArenaState` (`src/economy.rs`, `src/world.rs`)**:
    - Tracks `100` max HP (`ARENA_MAX_HP`), `alive: bool`, and `spawn_shield_until` (`2.5s` invulnerability on entry/respawn).
    - **Safe Sanctuary Circle** around `(0, 0, 25)` (`z >= 19.5 && |x| <= 9.0`) with Return Stargate at `(0, 0, 30)` where mobs cannot enter or deal damage.
    - **Chasing Aggro AI & Volcanic Lava Hazards**: 8 fierce arena monsters (`rift_spider`, `void_wraith`, `lava_golem`) patrol South (`z ∈ [-2.3, 18.8]`) and North (`z ∈ [-31.5, -7.7]`) basalt islands, chasing nearby players and striking on melee contact (`14`–`28` HP damage). Stepping into the central **Molten Lava River** (`z ∈ (-6.8, -3.2)` outside the 3 stone bridges at `x = -16, 0, +16`) or outer lava moat burns HP (`25` HP/tick), and falling into the abyss (`y < -2.2`) triggers instant KO (`100` HP).
    - **Zero-Grief Death & Mandatory Respawn**: Reaching `0 HP` sets `alive = false` (locking movement, melee swings, resource harvesting, and loot pickups while keeping 100% of Starlight Coins and Backpack items safe), updates the native `<progress id="arena-hp-bar">` HUD bar, and opens `#arena-death-overlay` (**💀 YOU FELL IN THE SHADOW RIFT!**). Calling `POST /api/ella-world/arena/respawn` (or WS/UDP `"arena_respawn"`) restores full HP (`100` or `150` with `aegis_heart_charm`) at Sanctuary `(0, 0, 25)` with a fresh `2.5s` shield; drinking a potion heals `+35 HP` (`+75 HP` for `starlight_potion`) in the arena.

---

## 14. Full Ella's World Expansion Pack — All 6 Upgrades (`src/models.rs`, `src/economy.rs`, `src/persistence.rs`, `src/world.rs`, `src/http_api.rs`, `public/js/world3d.js`, `public/js/app.js`)

### 14.1 Adoptable 3D Pets & Plaza Hoverboards / Mounts (`POST /api/ella-world/companions`)
- **Zero-Friction Starter Kit (`ensure_starter_companions`)**: Every existing and newly created account automatically owns `cloud_puppy` (equipped by default) and `starlight_board`.
- **4 Sculpted 3D Pets (`PET_CATALOG`) & Shop/Dungeon Companion Eggs (`PET_EGG_SHOP_PRICE = 120`)**:
  - `cloud_puppy` (*common*, `0 🪙` starter): `+2.0m` loot magnet radius + `+5%` bonus Starlight Coins.
  - `starlight_kitsune` (*rare*, `160 🪙` or `pet_egg`): `+1.5m` magnet + `+20%` bonus Starlight Coins.
  - `shadow_kitty` (*epic*, `220 🪙` or `pet_egg`): `+1.5m` magnet + `+10%` bonus coins + `+12%` sprint agility.
  - `baby_crystal_dragon` (*legendary*, `420 🪙` or hatched from `pet_egg` with articulated flapping `WingL`/`WingR`): `+2.5m` magnet + `+15%` bonus coins + `+1` bonus combat damage.
  - Players can earn `pet_egg` from Golden Vault Chests, Obsidian Dragon Boss kills, Starlight Workshop crafting, or purchase one directly via `action: "buy_egg"` (`120 🪙`).
- **3 Plaza Hoverboards & Skates (`MOUNT_CATALOG`)**:
  - `starlight_board` (*common*, `0 🪙` starter, `1.58×` Plaza speed), `sunset_skates` (*rare*, `150 🪙`, `1.66×` Plaza speed), and `comet_surfer` (*epic*, `320 🪙`, `1.75×` Plaza speed).
  - `UserAccount::preferred_mount` remembers the player's chosen hoverboard/skates so quick-toggling (`X` / `mountId: "toggle"`) re-equips their unlocked mount instead of resetting to `starlight_board`.
  - Binary `OP_C2S_STATE` / `OP_S2C_WORLD_STATE` extended with `anim = 4 ("hover")` and `emote = 9 ("sleep")` (`src/udp_wire.rs`).
  - **World-Space Trailing Pet Physics (`WorldAvatar.petHolder`)**: Companion pets detach into world-space coordinate smoothing so spinning or flipping the owner avatar never whips the pet around in a rigid circle.

### 14.2 Claimable Neighborhood Homes, 3D Nameplate Mailboxes, Co-Builders & Doorbells (`GET/POST /api/ella-world/homes`, `world_homes.json`)
- **6 Neighborhood Plots (`plot_1`..`plot_6`)**: Persisted atomically in `data/world_homes.json`. Enforces **1 claimed plot per player** (claiming a new plot cleanly releases the player's previous plot and resets any previous stranger's `coBuilders`/`doorLocked`/`customTitle`).
- **Plot Permissions (`HomePlotClaim::is_authorized_builder`)**:
  - Unclaimed plots remain open for collaborative building.
  - On a claimed plot, voxel placement and multi-cell `block_occupied_cells` removal inside `[centerX ± halfExtent, centerZ ± halfExtent]` are restricted to `ownerUsername`, invited `coBuilders`, Host Ella (`role == "host"`), and Parent Admin (`role == "admin"`).
  - When `doorLocked == true`, visitors cannot toggle doors/windows inside the plot (`🔒 Home is locked — press E on the 3D Mailbox to ring the Doorbell!`), while Host Ella & Parent Admin retain moderation override (`action: "release"`).
- **Live Doorbell Chime & Top-Layer Popover**: `action: "ring_doorbell"` broadcasts `home_doorbell`, triggering 3D mailbox sparkles, a synthesized two-tone WebAudio Ding-Dong chime, and a non-modal `popover="manual"` notification (`#doorbell-popover`) with a one-click **`🔓 Open Home`** button for the owner.
- **Interactive Home Furniture**: Sitting on `chair` / `sofa` (`emote: "sit"`), lying down on `bed` (`emote: "sleep"`), and toggling `nightstand_lamp` / `crystal_lamp` `PointLight` illumination via `action: "toggle"`.

### 14.3 Starlight Workshop Crafting Bench & Gear Progression (`GET/POST /api/ella-world/craft`)
- **3D Plaza Workshop (`Env_Crafting_Bench` at `(12.5, 0, 6.5)`)**: Combines real `ITEM_CATALOG` Dungeon & Shadow Rift drops (`star_shard`, `slime_gel`, `moon_opal`, `crown_relic`) + Starlight Coins across 7 recipes (`CRAFTING_RECIPES`):
  1. `crystal_blade` (`⚔️ Starlight Crystal Blade`, `accessory: "crystal_blade"`, `6.2m` `2 DMG` melee cleave + 3D glowing crystal crescent slash arc).
  2. `starlight_wand` (`🪄 Starlight Astral Wand`, `accessory: "starlight_wand"`, `16.2m` `2 DMG` astral bolt + `+8 HP` self-mend per hit in Shadow Rift).
  3. `aegis_heart_charm` (`🛡️ Aegis Heart Charm`, passive `100 -> 150 Max HP` in Shadow Rift whenever held in Backpack, reconciled live on Arena ticks, trades, and crafts).
  4. `starlight_potion` (`✨ Starlight Elixir`, `+75 HP` heal + `30s` Swiftness & Cloud Leap).
  5. `rift_beacon` (`🔮 Obsidian Rift Beacon`, bypasses Dragon Boss altar cooldown + `80%` `pet_egg` luck).
  6. `aurora_crystal_lamp` (`💎 Aurora Crystal Lamp`, glowing northern-lights home decor relic).
  7. `pet_egg` (`🥚 Starlight Companion Egg`, hatches 3D pets in `#companions-dialog`).

### 14.4 Timed "Start Fashion Show!" Mode on the DTI Runway (`POST /api/ella-world/runway`)
- **Opt-In Non-Modal Invite (`#fashion-invite-popover`)**: `action: "start_show"` broadcasts `show_started` without force-teleporting builders or dungeon fighters.
- **4-State Server Machine (`idle -> dress_up (45s) -> catwalk (12s/model) -> podium (18s) -> idle`)**:
  - Contestants locking outfits early (`action: "ready_show"`) advance immediately to `catwalk`; when `dress_up` transitions to `catwalk`, any contestant still inside Glam Studio automatically saves & exits onto the Runway stage.
  - During `catwalk`, 3D stage spotlight `runwaySpotlightGroup` tracks `spotlightUsername`, spotlighted contestants automatically strut from backstage (`z = -8.8`) to the front podium disc (`z = 11.0`) and strike a pose (yielding immediately if manual WASD keys are pressed), and self-voting is blocked (`HTTP 400`).
  - **Solo / Small-Party AI Runway Judges**: Whenever `< 3` human voters rate a contestant, friendly AI Judges inject `4★–5★` scores so solo and 2-friend shows always feel celebratory, awarding `+60 / +40 / +25 / +15 🪙` on the Podium!

### 14.5 Obby Speedrun Gates & Persistent Top-5 Leaderboards (`GET/POST /api/ella-world/obby`, `world_leaderboards.json`)
- **Start & Summit Finish Laser Gates (`Env_Obby_Gate`) + 3D Hall of Fame Scoreboards (`Env_Leaderboard_Board`)**: Built for both **Classic Sky Obby (`easy`)** (`(0, 0, -13.5)` -> `(6.5, 17.0, -20.0)`) and **Pro Obby Gauntlet (`hard`)** (`(54.2, 0, 52.0)` -> `(64.0, 39.0, 74.0)`).
- **Anti-Cheese Fairness**: Crossing a Start Gate flushes an immediate `C2S_STATE` frame and automatically dismounts any active hoverboard (`companions_update` `"auto_dismount_obby"`), enforces server-authoritative minimum possible times (`>= 2,800 ms` easy, `>= 5,000 ms` hard), and cancels active runs if a player warps (`H`/`F`/`R`/`T`/Mailbox/Zone) or mounts mid-run. Personal bests & Top-5 all-time records (`0.01s` precision) persist atomically in `data/world_leaderboards.json`.

### 14.6 Dynamic 12-Minute Sky / Sunset / Aurora Night Cycle, Meteor Shower & Co-Op Obsidian Dragon Boss (`GET/POST /api/ella-world/sky`, `POST /api/ella-world/arena/boss`)
- **Synchronized Plaza Sky Cycle (`720,000 ms`)**: Transitions smoothly across **Sunny Day**, **Cotton-Candy Sunset**, and **Aurora Night** (GLSL Northern Lights ribbons + 260 twinkling stars + **warm golden cottage windows, campfires, and street/plot lanterns that brighten automatically at Sunset & Night**). Host Ella (`role == "host"`) and Parent Admin can override the phase (`auto` / `day` / `sunset` / `aurora_night`) or launch a **🌠 Starlight Meteor Shower** spawning catchable **Falling Stars** (`action: "catch_star"`, `+16..32 🪙` + pet bonus + `40%` `star_shard`).
- **Co-Op Obsidian Rift Dragon Boss (`Env_Boss_ObsidianDragon_*` at `(0, 0, -24.5)`)**:
  - Summoned at the North Island Altar (`POST /api/ella-world/arena/boss`, `25s` cooldown or instant with `rift_beacon`).
  - Scales HP dynamically by active Arena player count (`18 + (players - 1) * 12`) and pulses expanding purple **Shockwave Rings** (`18 DMG` on ground contact if `y < 0.65`, cleanly dodged by jumping `y >= 0.65`).
  - **Zero Loot-Stealing**: Every contributing attacker receives personal Starlight Coins (`95..=140 🪙`), a personal rare/epic/legendary relic (`roll_boss_personal_loot`), and a **guaranteed `pet_egg` on their first Obsidian Dragon defeat** (`first_boss_defeated`, plus `40%` base / `80%` `rift_beacon` `pet_egg` rolls on repeat kills).

---

## 15. Offloaded Rust Build & Verification Pipeline on `water` (`192.168.11.7`) / `solidstate` (`192.168.11.25`)

To keep the local workstation (`darter`) cool and responsive, heavy `cargo` compilations, Clippy checks, Android NDK `gdext` builds, headless Godot 4 imports/tests, and APK/AAB exports offload over LAN SSH to **`water` (`robert@192.168.11.7:~/builds/ella-world`)** with automatic fallback to **`solidstate` (`robert@192.168.11.25:~/builds/ella-world`)**:
- **`.cargo/config.toml`**: Caps any accidental local `cargo` invocation to `jobs = 2`.
- **`scripts/remote_cargo_solidstate.sh`**: Auto-selects `water` (`192.168.11.7`) or `solidstate` (`192.168.11.25`), syncs tracked sources via `rsync`, executes `cargo "$@"` against the warm remote `target/` cache, and pulls `target/release/ella-world-server` back to local `target/release/` on `--release` builds.
- **`scripts/verify_native.sh`**: Runs browser ES module & CLI `node --check` syntax checks + zero-SSE audit locally, then runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and all **40 unit + integration tests** remotely.

---

## 16. Invite-Only Multi-Playground Platform & Native Godot 4 Android App (`src/playground.rs`, `godot_client/`)

### 16.1 Platform Registry & Zero-Migration Multi-Playground Multiplexing (`src/playground.rs`)
- **Zero-Breakage Default Playground (`"ella"`)**: Ella's primary world remains stored directly in `ELLA_DATA_DIR` (`data/world_users.json`, `data/world_blocks.json`, `data/world_homes.json`, `data/world_leaderboards.json`, `data/world_chat.jsonl`).
- **Isolated Additional Family Playgrounds (`data/playgrounds/<slug>/`)**: Each additional Playground spins up its own `Arc<WorldEngine>` backed by `data/playgrounds/<slug>/`, isolating:
  - Per-Playground **Parent Master Password** (`adminPassword` — single-password Admin Panel workflow identical to Ella's World)
  - Voxel blocks & neighborhood Home Plot claims (`world_blocks.json`, `world_homes.json`)
  - Starlight Coin balances, 28-item Backpacks, Debtor's Prison state, Curfews & Chat logs (`world_users.json`, `world_chat.jsonl`)
- **Single-Port Transport Multiplexing**:
  - `src/auth.rs` embeds `worldSlug` inside signed HMAC-SHA256 `WireTokenPayload` tokens (`sign_world_token` / `verify_world_token`) while keeping `TokenClaims` backward-compatible (`default = "ella"`).
  - `:4444` HTTPS/WSS (`src/http_api.rs`, `src/ws_spine.rs`), `:4444/udp` WebRTC SFU (`src/webrtc_sfu.rs`), and `:44445/udp` Native `EWLD` v1 (`src/udp_native.rs`) all route player state, discrete events, and Opus voice strictly within the session's `world_slug` (`attach_world`).

### 16.2 Two-Tier Invite Governance (`data/platform_registry.json`)
1. **Uninvited Signup Rejection**: Public account registration without a valid invite code is rejected (`403 Forbidden`).
2. **Tier 1 — Friend Invites (`STAR-XXXX-XXXX`)**:
   - **Parent Admin (`POST /api/ella-world/admin/invites`)**: Creates immediately `active` `STAR-...` codes with configurable `maxUses`, `expiresInHours`, `starterCoins`, and safety presets (`presetChatMode`, `presetVoiceMuted`, `presetCanBuild`, `presetCanTrade`), plus scannable SVG QR codes (`public/js/qr_svg.js`).
   - **Kid Host Drafts (`POST /api/ella-world/invites/host-draft`)**: A Playground's Kid Host (`role == "host"`, e.g. Ella) can draft a Friend Invite in-game (`status: "pending_parent_approval"`). Attempting to redeem a pending code returns `403 Forbidden` until a Parent clicks **Approve** in the Admin Panel (`action: "approve"`).
3. **Tier 2 — Parent Playground Founder Invites (`FAMILY-XXXX-XXXX`)**:
   - Issued by the Platform Super-Admin (`ELLA_SUPERADMIN_PASSWORD`, `POST /api/ella-world/superadmin/invites`, `#superadmin-dialog`, or `scripts/ella_admin_user_cli.mjs --create-family-invite`).
   - Redeemed via `POST /api/ella-world/invites/redeem-family` (aliased at `/playgrounds/create`) to provision a brand-new `<Kid>'s World` Playground (`data/playgrounds/<slug>/`), register its independent Parent Master Password + initial Kid Host account, and attach live UDP/SFU loops without restarting `ella-world.service`.

### 16.3 One Login + `"My Playgrounds"` Switcher & Cross-World Avatar Continuity
- **Single Global Kid Identity (`GlobalUserIdentity`)**: Kids sign in once (`POST /api/ella-world/login`), inspect all invited Playgrounds (`GET /api/ella-world/playgrounds/mine`), redeem additional `STAR-...` invites (`POST /api/ella-world/invites/join-world`), and hop across Playgrounds in 1 click/tap (`POST /api/ella-world/playgrounds/switch`).
- **Roaming 3D `AvatarAppearance` with Local Exclusive Reconciliation**: Updating an outfit in any Playground (`WorldEngine::avatar_sync_tx`) updates `GlobalUserIdentity.avatar` and propagates to every Playground the player belongs to—automatically stripping Playground-exclusive dungeon/boss cosmetics in worlds where that item has not yet been earned (`reconcile_equipped_cosmetics`).

### 16.4 Native Godot 4.3 Android App (`godot_client/`, `com.robertscreations.ellaworld`)
- **Shared 3D Assets (`scripts/sync_godot_assets.sh`)**: Reuses the exact Blender 5.2 `.glb` packs (`ella_dti_wardrobe.glb`, `ella_world_environment.glb`, `ella_world_props_loot.glb`) and PBR Obby textures (`public/textures/obby/*`) for 1:1 visual and coordinate cross-play with browser players.
- **Rust GDExtension (`godot_client/rust_opus`, `EllaNativeNet`)**: Built with `godot = "0.5"` (`api-4-3`) and pure-Rust `opus-pure = "0.2.5"` (RFC 6716 SILK/CELT/Hybrid 48 kHz stereo Opus encoder & decoder) for `aarch64-linux-android`, `x86_64-linux-android`, and `x86_64-unknown-linux-gnu`—speaking raw `EWLD` v1 UDP (`:44445/udp`) for 20 Hz movement + Push-to-Talk cross-play voice bridged directly into browser WebRTC SFU peers, paired with `WebSocketPeer` (`/api/ella-world/ws`) spine + binary fallback.
- **Landscape Multi-Touch UX & Dual Packaging (`scripts/remote_build_android.sh`)**:
  - Touch-index-isolated 360° left virtual joystick (`MultiTouchController.gd`), right-side 1-finger swipe-to-look + 2-finger pinch-zoom, and thumb-arc `Jump` / `Action` / `Board` / `Hold Mic` buttons.
  - Exports both a **Signed Family Release APK** (`https://robertscreations.com:4444/download/ella-world.apk` + `/api/ella-world/android/release` + Web QR install modal) and a **Google Play Store Android App Bundle (`ella-world.aab`)** for Closed Testing.
  - Highlighted fashion-spotlight extension documented in [`docs/dti_fashion_hangout_focus_extension.md`](dti_fashion_hangout_focus_extension.md).




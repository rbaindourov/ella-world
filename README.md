# 👑 ELLA World (`ella-world` `v0.3.0`)

**ELLA World** is a standalone, zero-Node.js 3D multiplayer hangout, **Dress to Impress** fashion runway, sky obby, co-op voxel builder, and spatial voice server written in **100% Rust** (`tokio` + `webrtc-rs` + `axum`) with a Three.js WebGL frontend.

**`v0.3.0` Hybrid Sync + Zero-Renegotiation UDP SFU Architecture**:

- **Persistent Binary + JSON WebSocket Spine (`wss://<host>/api/ella-world/ws` over `:443/tcp`, `TCP_NODELAY`)**:
  - **Rock-Solid Presence (`12s` Reconnect Grace)**: Brief Wi-Fi blips, tab switches, or WebRTC ICE transitions never drop players or reset avatars.
  - **Self-Healing 3-Second `world_sync_digest`**: Continuously reconciles active players, SHA-256 `avatarHash` outfit signatures, runway leaderboards, and `blocksVersion` so game state never drifts out of sync.
  - **Automatic 20 Hz Binary Fallback**: Seamlessly carries 28-byte `OP_C2S_STATE` / `OP_S2C_WORLD_STATE` frames over `:443` while UDP handshakes or whenever strict NAT/firewalls block UDP, promoting automatically to `44444/udp` when healthy.
  - **Server-Synchronized World Clock**: Phase-locks moving Sky Obby platforms, disco floor tiles, and runway lights across all connected devices.
- **Zero-Renegotiation Single-Port Browser UDP (`44444/udp` via `UDPMuxDefault`)**:
  - **8-Slot Pre-Allocated Opus Voice SFU (`DTLS-SRTP` over UDP)**: Negotiates 1 upstream mic track + 8 downlink speaker slots (`ella_slot_0`..`ella_slot_7`) **once** at login—eliminating mid-game SDP renegotiation and ICE drops when friends join or leave.
  - **20 Hz Unreliable UDP Fast-Lane (`SCTP/DTLS` `ella-udp-state`)**: `ordered: false, maxRetransmits: 0` binary 28-byte state datagrams eliminate TCP head-of-line blocking on live LAN/WAN UDP links.
- **Native Raw UDP Server (`44445/udp` via `tokio::net::UdpSocket`)**:
  - Speaks the compact binary `EWLD` v1 UDP wire protocol (`OP_HELLO`, `OP_WELCOME`, `OP_C2S_STATE`, `OP_S2C_WORLD_STATE`, `OP_VOICE_OPUS`, `OP_EVENT_JSON` + `OP_EVENT_ACK`) and cross-bridges Opus voice packets bidirectionally with WebRTC browser peers.

---

## ✨ Features

- **📱 Native Godot 4 Android App (`com.robertscreations.ellaworld`) + Instant Family QR Install**: Full 3D cross-play parity with WebGL browsers over `:44445/udp` `EWLD` v1 + pure-Rust RFC 6716 Opus voice (`EllaNativeNet` GDExtension) and `:4444` `WSS` spine fallback, reusing the exact Blender 5.2 `.glb` character, environment, monster, and furniture packs with landscape-first 360° thumbstick + swipe-look/pinch-zoom + thumb-arc controls. Hosted directly at `/download/ella-world.apk` (with scannable SVG QR codes on the Login & Invite screens) alongside a Google Play Closed Testing `.aab` bundle (`godot_client/build/ella-world.aab`). See [`docs/dti_fashion_hangout_focus_extension.md`](docs/dti_fashion_hangout_focus_extension.md) for the DTI Fashion & Hangout Spotlight extension.
- **🎟️ Invite-Only Onboarding & Multi-Playground Family Hosting (`STAR-...` & `FAMILY-...`)**:
  - **Friend Invites (`STAR-XXXX-XXXX`)**: Uninvited public signups are blocked. Parent Admins generate active `STAR-...` links/QR codes with preset chat/voice safety rules & starter coins, while Kid Hosts (e.g. Ella) can draft `"Pending Parent Approval"` invites that activate on 1-click Parent approval.
  - **Parent Founder Invites (`FAMILY-XXXX-XXXX`)**: Platform Super-Admin issues `FAMILY-...` codes so trusted families can create isolated **`"<Kid>'s World"` Playgrounds** (`data/playgrounds/<slug>/`), each governed by a single Per-Playground Parent Master Password.
  - **One Login + `"My Playgrounds"` Switcher**: Kids sign in once, switch between invited Playgrounds in 1 tap/click, and carry their customized 3D Avatar across every Playground while Home Plots, Coins/Inventory, Curfews, and Chat Logs stay strictly isolated per Playground.
- **🐾 Adoptable 3D Pets & Plaza Hoverboards / Mounts (`P`, `X`)**: Every player receives **Cloud Puppy** + **Starlight Hoverboard** free out of the box! Adopt or hatch **Starlight Kitsune**, **Shadow Rift Kitty** (`+12%` sprint agility), and **Baby Crystal Dragon** (`+1` combat damage) from **🥚 Starlight Companion Eggs** (dropped in dungeons/bosses, crafted, or bought in `#companions-dialog` for `120 🪙`), or cruise on **Cotton-Candy Starlight Skates** and **Golden Comet Surfer** (`1.58x–1.75x` Plaza speed; `X` remembers your preferred mount and Obby Start Gates auto-dismount).
- **🏡 Claimable Neighborhood Homes, 3D Nameplate Mailboxes, Co-Builders & Doorbells**: Claim 1 of 6 neighborhood plots (`plot_1`..`plot_6` — 3 furnished starter villas + 3 open architect lots) persisted in `data/world_homes.json` (switchable/releasable anytime, with Host Ella & Parent Admin moderation override), customize your 3D Mailbox Nameplate & Co-Builders, toggle **🔓 Open** vs **🔒 Locked + 🔔 Doorbell** (with WebAudio Ding-Dong chime + quick-unlock popover), sit on chairs/sofas, sleep (`9`) on beds, and click lamps to switch room lights on/off.
- **⚒️ Starlight Workshop Crafting Bench (`U`) & Gear Progression**: Visit the 3D Workshop at `(12.5, 0, 6.5)` in Starlight Plaza to combine `star_shard`, `slime_gel`, `moon_opal`, and `crown_relic` with Starlight Coins into **⚔️ Starlight Crystal Blade** (`2 DMG` melee cleave + 3D crystal slash arc), **🪄 Starlight Astral Wand** (`16m` `2 DMG` star-bolt + `+8 HP` Arena self-mend), **🛡️ Aegis Heart Charm** (passive `100 -> 150 Max HP` in Shadow Rift), **✨ Starlight Elixir** (`+75 HP` + 30s speed & jump boost), **🔮 Obsidian Rift Beacon**, **💎 Aurora Crystal Lamp**, and **🥚 Companion Eggs**.
- **👗 Timed "Start Fashion Show!" Mode on the DTI Runway**: Opt-in non-modal `popover="manual"` invite banner (never teleports busy builders/fighters), `45s` Dress-Up countdown visible inside Glam Studio & Runway HUD, automatic `12s`-per-model spotlight catwalk strut & front-of-stage pose, `1★–5★` voting (no self-voting during active show), friendly **AI Runway Judges** (`4★–5★` when `< 3` human voters rate), and 1st/2nd/3rd Podium + Starlight Coin prizes (`+60 / +40 / +25 / +15 🪙`).
- **⏱️ Obby Speedrun Laser Gates & Persistent Top-5 Leaderboards**: Glowing Start & Summit Finish laser gates + 3D Hall of Fame scoreboards (`data/world_leaderboards.json`, `0.01s` precision) for both **Classic Sky Obby** (`easy`) and **Pro Obby Gauntlet** (`hard`), with one-click warp to Start Gate and automatic cancellation on checkpoint warps (`R`/`T`) or hoverboard mounts.
- **🌅 Dynamic 12-Minute Sky / Cotton-Candy Sunset / Aurora Night Cycle + Meteor Shower + Co-Op Obsidian Dragon Boss**: Server-synchronized `720s` Plaza sky cycle where **house windows, campfires, and lanterns warm up with golden light at Sunset & Aurora Night**, Host Ella & Parent Admin Sky/Meteor Shower controls spawning catchable **🌠 Falling Stars** (`16–32 🪙` + bonus `star_shard`), and the summonable **🐉 Co-Op Obsidian Rift Dragon Boss** at Shadow Rift North Altar `(0, 0, -24.5)` with jump-over purple Shockwave Rings (`18 DMG` if `y < 0.65`, dodged when `y >= 0.65`), dynamic player HP scaling, and fair personal loot + guaranteed first-kill `pet_egg` for every contributor.
- **Crystal Cavern (`G`) & Shadow Rift Mortal Arena (`K`) + Scam-Proof P2P Trading (`I`)**: 28-item persistent economy (`data/world_users.json`), 5 sculpted dungeon/arena monster types (`ella_world_environment.glb`, 47 meshes), anti-duplication house trophy pedestals (`🏆 My Loot`, `ella_props_and_loot.glb`, 33 meshes), and two-sided Lock + Confirm P2P trading.
- **Stylized 3D Girl & Boy Characters + Dress to Impress Glam Salon (`C`)**: Rigged 25-bone skeletal characters (`ella_dti_wardrobe.glb`, 47 modular meshes across `Body_Stylized` & `Body_Male`), right-hand **Plasma Ray Gun / Crystal Blade / Starlight Wand** attachments, 9 poses/emotes (`1`–`9`), and dedicated `GlamSalonStage3D` mirror preview.
- **Parent-Controlled Whitelist, Escalating Fines, 3D Debtor's Prison & Parental Safety Suite**:
  - **Evasion-Resistant Cursing Sanitization & Negative-Balance Fines**: Catches spaced/punctuation/leetspeak profanity evasion while preserving clean words (`"class"`, `"pass"`, `"grass"`, `"455 coins"`), deducting escalating Starlight Coin fines (`-15`, `-35`, `-75`, `-150`, ...) into signed negative `i64` debt balances and logging full strike/fine telemetry in `world_chat_blocked.jsonl`.
  - **3D Debtor's Prison (`ZONE_PRISON = 2`) & Community Service**: Habitual violators who drop below the debt threshold (`-50 🪙`) or reach the strike limit (`3`) are automatically sentenced into an enclosed 3D Debtor's Prison courtyard where building, trading, and dungeon warps are locked until they complete glowing 3D community-service chores (`E`, `+5 🪙` strictly capped at `0 🪙`) or receive a **🕊️ Parent Pardon**.
  - **Granular Per-Child & Global Parental Controls**: Per-account server-enforced Mic Mute (drops Opus voice frames on the SFU & native UDP servers), Safe Preset Quick-Chat-Only mode, Build/Trade locks, Account Grounding, World Pause Lock, and automatic 24h Bedtime Curfew windows.

---

## 🚀 Quick Start (Native Rust Binary & Remote Build Offload on `water` / `solidstate`)

```bash
# 1. Verify JS/ES modules locally + offload Rust fmt, Clippy & 40/40 tests to water (192.168.11.7) / solidstate (192.168.11.25)
./scripts/verify_native.sh

# 2. Build release ella-world-server + GDExtension NDK + Godot 4 Headless Tests + Signed APK & Play Store AAB
./scripts/remote_build_android.sh
```

Then open **`http://localhost:8097`** in two browser tabs (or install `public/download/ella-world.apk` on Android) and sign in or redeem a `STAR-...` Friend Invite!

### Static Portable Linux Build (`musl`)

To build a zero-glibc static Linux binary that runs on any x86_64 Linux server regardless of distribution age:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
# Output: target/x86_64-unknown-linux-musl/release/ella-world-server
```

---

## ⚙️ Environment Variables

| Variable | Default | Description |
| :--- | :--- | :--- |
| `ELLA_HTTP_ADDR` | `0.0.0.0:8097` | Bind address for static WebGL assets, `/ws` spine & `/rtc/offer` (`0.0.0.0:4444` for standalone TLS) |
| `ELLA_TLS_CERT` | *(unset)* | Optional PEM certificate chain (`fullchain.pem`) to serve native Rust HTTPS + `wss://` directly without Nginx |
| `ELLA_TLS_KEY` | *(unset)* | Optional PEM private key (`privkey.pem`) paired with `ELLA_TLS_CERT` |
| `ELLA_WEBRTC_BIND_IP` | `0.0.0.0` | Bind IP for the single-port multiplexed WebRTC UDP socket |
| `ELLA_WEBRTC_UDP_PORT` | `44444` | **UDP port** carrying browser Opus voice + 20 Hz binary game state (`4444` in standalone `:4444` mode) |
| `ELLA_NATIVE_UDP_PORT` | `44445` | **UDP port** carrying raw `EWLD` v1 native client voice + state (used by Godot 4 Android app) |
| `ELLA_PUBLIC_IP` | *(unset)* | Optional public WAN IPv4 advertised as an additional `srflx` UDP ICE candidate alongside LAN `host` candidates |
| `ELLA_DATA_DIR` | `./data` | Persistent storage directory for `platform_registry.json`, default `world_*.json`, & isolated `playgrounds/<slug>/` |
| `ELLA_PUBLIC_DIR` | `./public` | Path to static Three.js client & `/download/ella-world.apk` directory |
| `ELLA_SEED_USERS_FILE` | `./config/users.example.json` | Initial seed template copied into `ELLA_DATA_DIR` on first boot |
| `ELLA_SUPERADMIN_PASSWORD` | *(falls back to primary admin pw)* | Platform Super-Admin password for issuing `FAMILY-XXXX-XXXX` Parent Founder invites |
| `ELLA_HMAC_SECRET` | *(dev default)* | Secret key used to sign constant-time HMAC-SHA256 session tokens & password fingerprints |
| `ELLA_TOKEN_TTL_SECS` | `43200` (`12h`) | Maximum lifetime (seconds, clamped `60..2592000`) for player & admin session tokens |

### 🔒 Parent Admin, Invite Governance & Multi-Playground CLI

Rolling any friend's password in the Parent Admin Panel or CLI automatically invalidates saved browser/Android tokens (`pwdSig` + `tokenEpoch`) and kicks live WebSocket / WebRTC / UDP sessions immediately:

```bash
# Friend Invites (STAR-XXXX-XXXX) & 1-click Parent Approvals (scoped via --world <slug>, default ella):
node scripts/ella_admin_user_cli.mjs --invites
node scripts/ella_admin_user_cli.mjs --create-friend-invite --invite-label "For Leo" --starter-coins 150
node scripts/ella_admin_user_cli.mjs --approve-invite STAR-ABCD-1234
node scripts/ella_admin_user_cli.mjs --revoke-invite STAR-ABCD-1234

# Platform Super-Admin & Family Founder Invites (FAMILY-XXXX-XXXX):
node scripts/ella_admin_user_cli.mjs --playgrounds
node scripts/ella_admin_user_cli.mjs --create-family-invite --invite-label "Maya Family Playground"

# Immediately expire ALL logged-in tokens globally & kick active sessions:
node scripts/ella_admin_user_cli.mjs [--world ella] --revoke-all

# Expire tokens & sign out a single player account:
node scripts/ella_admin_user_cli.mjs [--world ella] --revoke-user ella

# Rotate production ELLA_HMAC_SECRET key in systemd & verify stale-token 401 rejection:
node scripts/rotate_prod_hmac_secret.mjs
node scripts/verify_prod_http_and_auth.mjs

# Inspect current sessionEpoch, online status, coins/debt, strikes, prison status & parental locks:
node scripts/ella_admin_user_cli.mjs [--world ella] --list

# Parental controls, coin fines/bonuses, Debtor's Prison sentencing & Parent Pardons:
node scripts/ella_admin_user_cli.mjs --username ella --coins -50
node scripts/ella_admin_user_cli.mjs --username ella --sentence 4 --reason "Habitual unkind chat"
node scripts/ella_admin_user_cli.mjs --username ella --pardon
node scripts/ella_admin_user_cli.mjs --username ella --reset-strikes
node scripts/ella_admin_user_cli.mjs --username ella --mute-voice true --chat-mode quick_only

# Inspect Machine Digital Signatures, blocked alt-account evasions & Shared Family PC overrides:
node scripts/ella_admin_user_cli.mjs --devices
node scripts/ella_admin_user_cli.mjs --trust-device SIG-AB12CD34 true
node scripts/ella_admin_user_cli.mjs --ban-device SIG-AB12CD34 true
node scripts/ella_admin_user_cli.mjs --unlink-device SIG-AB12CD34
```

---

## 🔥 Firewall & Router Port-Forwarding Summary

1. **Standalone HTTPS + WebSocket Spine (`TCP 4444`, zero port `443`)**: With `ELLA_TLS_CERT` / `ELLA_TLS_KEY` set and `ELLA_HTTP_ADDR=0.0.0.0:4444`, Rust serves HTTPS WebGL assets, `/download/ella-world.apk`, auth, `wss://<host>:4444/api/ella-world/ws`, and 20 Hz fallback natively without Nginx.
2. **Browser Opus Voice + UDP Fast-Lane (`UDP 4444`)**: Open/forward **`4444/udp`** on your host firewall and NAT router to your dedicated game server's LAN IP (`192.168.11.24`) for sub-millisecond UDP movement and Opus voice.
3. **Godot 4 Android & Native Raw UDP Protocol (`UDP 44445`)**: Open/forward **`44445/udp`** so Godot 4 Android devices and native `EWLD` UDP clients enjoy direct 20 Hz UDP movement + cross-bridged 48 kHz Opus voice over Wi-Fi and LTE/5G (automatically falling back to `:4444` WSS if blocked).

See [`docs/architecture.md`](docs/architecture.md), [`docs/dti_fashion_hangout_focus_extension.md`](docs/dti_fashion_hangout_focus_extension.md), [`docs/housing_interiors_and_legendary_loot.md`](docs/housing_interiors_and_legendary_loot.md), [`docs/firewall_and_network_setup.md`](docs/firewall_and_network_setup.md), and [`docs/stress_test_and_scaling.md`](docs/stress_test_and_scaling.md) for complete protocol frame layouts, 3D housing/loot mechanics, stress-test benchmarks, and deployment instructions.

---

## 📊 Classroom Stress Testing & 100-Player Scaling

Verified on the 2-vCPU production server (`192.168.11.24`) up to **100 concurrent students + 3 active Opus speakers** using the built-in Rust [`ella-stress`](src/bin/stress.rs) harness (`10 -> 25 -> 50 -> 100` stages, 100% peer visibility, 100% UDP fast-lane share, 100% DataChannels open, `74.9 ms` login p95, `~51.8%` CPU, `~95 MiB` RSS):

```bash
cargo build --release --bin ella-stress
./target/release/ella-stress \
  --base-url https://192.168.11.24:14444 \
  --insecure-tls \
  --stages 10,25,50,100 \
  --hold-secs 15 \
  --active-talkers 3
```

Full Before/After telemetry and bottleneck analysis: [`docs/stress_test_and_scaling.md`](docs/stress_test_and_scaling.md).

---

## 📄 License

Released under the [MIT License](LICENSE).

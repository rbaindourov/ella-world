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

- **Crystal Cavern Dungeon Portal (`G`) & Legendary Loot Farming**: Step through the glowing East Plaza portal into a dark-amethyst subterranean cavern to mine 6 Crystal Veins, unlock Wooden & Golden Vault Chests with Dungeon Keys, and Jump-Stomp, Sparkle-Blast (`E`), or zap with the **🔫 Starlight Plasma Ray Gun** (`14.5m` ranged beam, `3 HP` instant defeat) against 8 patrolling Shadow Slimes & Crystal Bats for Starlight Coins, Potions, 3D Home Relics, and Exclusive Glam Cosmetics.
- **Persistent Backpack Economy (`I`), House Trophy Decorating & Scam-Proof Live P2P Trading**: 18-item catalog persisted in `data/world_users.json` (Gems, Vault Keys, Swiftness & Cloud Leap Potions, Prism Blocks, 5 3D Home Trophy Relics: *Amethyst Geode Lamp*, *Neon Slime Lava Lamp*, *Plushie Crystal Bat*, *Golden Starlight Chalice*, *Starlight Dragon Egg*, 4 Dungeon-Exclusive Glam Relics, and the first **Legendary** weapon `legendary_ray_gun`). Owned items can be displayed on 3D house pedestals (`🏆 My Loot`, atomic `-1`/`+1` anti-duplication conservation) or traded live via a two-sided Lock + Confirm window.
- **Stylized 3D Girl & Boy Characters + Dress to Impress Glam Salon**: Rigged 25-bone skeletal characters (`ella_dti_wardrobe.glb`, 47 modular meshes across `Body_Stylized` & `Body_Male`), featuring both the Ella Forensics glam cast and **Jack Tech-49 / Maverick Recon** boy builds (*Oblivion*-inspired Tech-49 Flight Jacket, Tactical Flight Pants, Armored Flight Boots, Recon Cap, Aviator Shades, and Hero Henley Tee), mix-and-match hairstyles, satin/glitter/matte gowns & tops, crowns, fairy wings, 3D star bags, right-hand **Starlight Plasma Ray Gun**, and dungeon-exclusive relics with live mirror preview.
- **Illuminated Fashion Runway**: Live rotating runway themes, 8 runway poses & dances (`1`–`8`), and 1–5 ⭐ live star voting with celebration confetti.
- **Solid 2-Story House Builder, Interactive Doors/Windows & 3D Home Interiors**: Solid cylinder-vs-AABB wall collision, `0.55m` stair auto-stepping, indoor ceiling bonk & camera pullback, interactive swinging **Doors & Windows** (`E` or click), 90° prop rotation (`Q`), and 25-mesh Blender 5.2 **Kitchen, Bathroom, Bedroom & Trophy Pedestal** interiors (`ella_world_props_loot.glb`) persisted atomically to `data/world_blocks.json`.
- **Synchronized Sky Obby Course**: Multi-stage floating parkour course with server-phase-locked moving platforms, checkpoints (`R`), and a Golden Trophy summit celebration.
- **Parent-Controlled Whitelist, Escalating Fines, 3D Debtor's Prison & Parental Safety Suite**:
  - **Evasion-Resistant Cursing Sanitization & Negative-Balance Fines**: Catches spaced/punctuation/leetspeak profanity evasion while preserving clean words (`"class"`, `"pass"`, `"grass"`, `"455 coins"`), deducting escalating Starlight Coin fines (`-15`, `-35`, `-75`, `-150`, ...) into signed negative `i64` debt balances and logging full strike/fine telemetry in `world_chat_blocked.jsonl`.
  - **3D Debtor's Prison (`ZONE_PRISON = 2`) & Community Service**: Habitual violators who drop below the debt threshold (`-50 🪙`) or reach the strike limit (`3`) are automatically sentenced into an enclosed 3D Debtor's Prison courtyard where building, trading, and dungeon warps are locked until they complete glowing 3D community-service chores (`E`, `+5 🪙` strictly capped at `0 🪙`) or receive a **🕊️ Parent Pardon**.
  - **Granular Per-Child & Global Parental Controls**: Per-account server-enforced Mic Mute (drops Opus voice frames on the SFU & native UDP servers), Safe Preset Quick-Chat-Only mode, Build/Trade locks, Account Grounding, World Pause Lock, and automatic 24h Bedtime Curfew windows.

---

## 🚀 Quick Start (Native Rust Binary)

```bash
# 1. Build & run tests natively (zero Docker, zero cloud CI required)
./scripts/verify_native.sh

# 2. Start the standalone server
cargo run --release
```

Then open **`http://localhost:8097`** in two browser tabs and sign in with the starter accounts from `config/users.example.json` (`ella` / `changeme-starlight` and `maya` / `changeme-puppy`).

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
| `ELLA_NATIVE_UDP_PORT` | `44445` | **UDP port** carrying raw `EWLD` v1 native client voice + state |
| `ELLA_PUBLIC_IP` | *(unset)* | Optional public WAN IPv4 advertised as an additional `srflx` UDP ICE candidate alongside LAN `host` candidates |
| `ELLA_DATA_DIR` | `./data` | Persistent storage directory for `world_users.json`, `world_blocks.json`, `world_chat.jsonl`, & `world_chat_blocked.jsonl` |
| `ELLA_PUBLIC_DIR` | `./public` | Path to static Three.js client directory |
| `ELLA_SEED_USERS_FILE` | `./config/users.example.json` | Initial seed template copied into `ELLA_DATA_DIR` on first boot |
| `ELLA_HMAC_SECRET` | *(dev default)* | Secret key used to sign constant-time HMAC-SHA256 session tokens & password fingerprints |
| `ELLA_TOKEN_TTL_SECS` | `43200` (`12h`) | Maximum lifetime (seconds, clamped `60..2592000`) for player & admin session tokens |

### 🔒 Parent Session Revocation & Password Rollover CLI

Rolling any friend's password in the Parent Admin Panel or CLI automatically invalidates saved browser tokens (`pwdSig` + `tokenEpoch`) and kicks live WebSocket / WebRTC / UDP sessions immediately. Parents can also expire all logged-in tokens at any time:

```bash
# Immediately expire ALL logged-in tokens globally & kick active sessions:
node scripts/ella_admin_user_cli.mjs --revoke-all

# Expire tokens & sign out a single player account:
node scripts/ella_admin_user_cli.mjs --revoke-user ella

# Rotate production ELLA_HMAC_SECRET key in systemd & verify stale-token 401 rejection:
node scripts/rotate_prod_hmac_secret.mjs
node scripts/verify_prod_http_and_auth.mjs

# Inspect current sessionEpoch, online status, coins/debt, strikes, prison status & parental locks:
node scripts/ella_admin_user_cli.mjs --list

# Parental controls, coin fines/bonuses, Debtor's Prison sentencing & Parent Pardons:
node scripts/ella_admin_user_cli.mjs --username ella --coins -50
node scripts/ella_admin_user_cli.mjs --username ella --sentence 4 --reason "Habitual unkind chat"
node scripts/ella_admin_user_cli.mjs --username ella --pardon
node scripts/ella_admin_user_cli.mjs --username ella --reset-strikes
node scripts/ella_admin_user_cli.mjs --username ella --mute-voice true --chat-mode quick_only
```

---

## 🔥 Firewall & Router Port-Forwarding Summary

1. **Standalone HTTPS + WebSocket Spine (`TCP 4444`, zero port `443`)**: With `ELLA_TLS_CERT` / `ELLA_TLS_KEY` set and `ELLA_HTTP_ADDR=0.0.0.0:4444`, Rust serves HTTPS WebGL assets, auth, `wss://<host>:4444/api/ella-world/ws`, and 20 Hz fallback natively without Nginx.
2. **Browser Opus Voice + UDP Fast-Lane (`UDP 4444`)**: Open/forward **`4444/udp`** on your host firewall and NAT router to your dedicated game server's LAN IP (`192.168.11.24`) for sub-millisecond UDP movement and Opus voice.
3. **Native Raw UDP Protocol (`UDP 44445`, optional)**: Open/forward **`44445/udp`** only if connecting native desktop/CLI `EWLD` UDP clients from outside your LAN.

See [`docs/architecture.md`](docs/architecture.md), [`docs/housing_interiors_and_legendary_loot.md`](docs/housing_interiors_and_legendary_loot.md), [`docs/firewall_and_network_setup.md`](docs/firewall_and_network_setup.md), and [`docs/stress_test_and_scaling.md`](docs/stress_test_and_scaling.md) for complete protocol frame layouts, 3D housing/loot mechanics, stress-test benchmarks, and deployment instructions.

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

# 👑 ELLA World (`ella-world`)

**ELLA World** is a standalone, zero-Node.js 3D multiplayer hangout, **Dress to Impress** fashion runway, sky obby, co-op voxel builder, and spatial voice server written in **100% Rust** (`tokio` + `webrtc-rs` + `axum`) with a Three.js WebGL frontend.

Unlike traditional web games that tunnel state over HTTP polling or Server-Sent Events (SSE), **ELLA World runs both real-time voice audio AND 20 Hz 3D game state synchronization natively over UDP**:

- **Single-Port Browser UDP (`44444/udp` via `UDPMuxDefault`)**:
  - **Voice Audio (`DTLS-SRTP` over UDP)**: Pure-Rust WebRTC Selective Forwarding Unit (SFU) relays 48 kHz Opus voice streams with 3D proximity panning, party mode, per-friend volume sliders, push-to-talk, and live green speaking rings.
  - **20 Hz 3D Game State (`SCTP/DTLS` Unreliable UDP DataChannel `ella-udp-state`)**: `ordered: false, maxRetransmits: 0` binary 28-byte `OP_C2S_STATE` (`0x10`) and `OP_S2C_WORLD_STATE` (`0x11`) datagrams eliminate TCP head-of-line blocking and measure live round-trip UDP latency (`ms`).
  - **Discrete World Events (`ella-reliable-events` over UDP)**: Co-op voxel block builds, Glam Salon outfit updates, speech-bubble chat, 1–5★ runway votes, and dynamic SFU track renegotiation.
- **Native Raw UDP Server (`44445/udp` via `tokio::net::UdpSocket`)**:
  - Speaks the compact binary `EWLD` v1 UDP wire protocol (`OP_HELLO`, `OP_WELCOME`, `OP_C2S_STATE`, `OP_S2C_WORLD_STATE`, `OP_VOICE_OPUS`, `OP_EVENT_JSON` + `OP_EVENT_ACK`) and cross-bridges Opus voice packets bidirectionally with WebRTC browser peers.

---

## ✨ Features

- **Stylized 3D Characters & Dress to Impress Glam Salon**: Rigged 20-bone skeletal characters (`ella_dti_wardrobe.glb`), mix-and-match hairstyles, satin/glitter/matte tops & gowns, skirts/jeans/cargos/overalls, heels/platform boots/sneakers, crowns, fairy wings, and 3D star bags with live mirror preview.
- **Illuminated Fashion Runway**: Live rotating runway themes, 8 runway poses & dances (`1`–`8`), and 1–5 ⭐ live star voting with celebration confetti.
- **Persistent Co-Op Voxel Builder**: Place and remove colorful 3D blocks together in real time; creations and character outfits persist atomically to `data/world_blocks.json` and `data/world_users.json`.
- **Sky Obby Course**: Multi-stage floating parkour course with checkpoints (`R`) and a Golden Trophy summit celebration.
- **Parent-Controlled Whitelist & Admin Panel**: Password-protected friend accounts and master parent dashboard — zero public sign-ups or unmoderated strangers.

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
| `ELLA_HTTP_ADDR` | `0.0.0.0:8097` | Bind address for static WebGL assets & WebRTC SDP handshake |
| `ELLA_WEBRTC_BIND_IP` | `0.0.0.0` | Bind IP for the single-port multiplexed WebRTC UDP socket |
| `ELLA_WEBRTC_UDP_PORT` | `44444` | **UDP port** carrying browser Opus voice + 20 Hz binary game state |
| `ELLA_NATIVE_UDP_PORT` | `44445` | **UDP port** carrying raw `EWLD` v1 native client voice + state |
| `ELLA_PUBLIC_IP` | *(unset)* | Optional public WAN IPv4 advertised as an additional `srflx` UDP ICE candidate alongside LAN `host` candidates |
| `ELLA_DATA_DIR` | `./data` | Persistent storage directory for `world_users.json` & `world_blocks.json` |
| `ELLA_PUBLIC_DIR` | `./public` | Path to static Three.js client directory |
| `ELLA_SEED_USERS_FILE` | `./config/users.example.json` | Initial seed template copied into `ELLA_DATA_DIR` on first boot |
| `ELLA_HMAC_SECRET` | *(dev default)* | Secret key used to sign constant-time HMAC-SHA256 session tokens |

---

## 🔥 Firewall & Router Port-Forwarding Summary

Because all browser WebRTC voice (`DTLS-SRTP`) and 20 Hz game state (`SCTP/DTLS`) traffic is multiplexed onto **one single UDP port** via `UDPMuxDefault`:

1. **HTTPS / Reverse Proxy (`TCP 443`)**: Required so browsers unlock `navigator.mediaDevices.getUserMedia` microphone access over a secure context (`deploy/nginx-ella-world.conf`).
2. **Browser Voice + Game State (`UDP 44444`)**: Open/forward **`44444/udp`** on your host firewall and NAT router to your server's LAN IP.
3. **Native Raw UDP Protocol (`UDP 44445`, optional)**: Open/forward **`44445/udp`** only if connecting native desktop/CLI `EWLD` UDP clients from outside your LAN.

See [`docs/architecture.md`](docs/architecture.md) and [`docs/firewall_and_network_setup.md`](docs/firewall_and_network_setup.md) for complete protocol frame layouts and deployment instructions.

---

## 📄 License

Released under the [MIT License](LICENSE).

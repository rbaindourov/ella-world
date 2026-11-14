# Firewall, NAT Port-Forwarding & Standalone TLS Setup (`4444` TCP + UDP)

ELLA World (`v0.3.0`) runs natively as a **standalone TLS Rust Game Server** directly on port **`4444`** (`4444/tcp` HTTPS + WebSocket Spine & `4444/udp` Multiplexed WebRTC Opus Voice + 20 Hz State)—with **zero dependency on port `443` or Nginx**.

*(If you instead choose to terminate TLS behind Nginx on port `443`, see `deploy/nginx-ella-world.conf`.)*

---

## 1. Required Ports Matrix (Dedicated Standalone Game Box — `192.168.11.24`)

| Port | Protocol | Direction | Required For | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **`4444`** | **TCP** | Inbound (WAN & LAN) | **Required** | Native Rust TLS (`tokio-rustls`) serving HTTPS WebGL assets, `/api/ella-world/ws` persistent WebSocket spine (`wss://<host>:4444/api/ella-world/ws`), 3s digest, and 20 Hz binary fallback |
| **`4444`** | **UDP** | Inbound / Bi-directional | **Required for WAN Voice & UDP Fast-Lane** | Single-port multiplexed WebRTC UDP socket carrying **both** 8-slot Opus Voice (`DTLS-SRTP`) and 20 Hz Unreliable Binary Game State (`SCTP/DTLS`) |
| `44445` | UDP | Inbound / Bi-directional | *Optional* | Native raw `EWLD` v1 UDP socket for non-browser desktop/CLI clients |

> **Why TCP + UDP share port `4444` cleanly**: Linux separates TCP and UDP socket namespaces completely. `ella-world-server` binds `0.0.0.0:4444` (TCP HTTPS/WSS) and `0.0.0.0:4444` (UDP `UDPMuxDefault`) simultaneously inside one standalone Rust process.

---

## 2. Home / Edge Router NAT Port-Forwarding (Internet Play)

For players on the **same local LAN / Wi-Fi**, zero router changes are required as long as the server's OS firewall allows `4444/tcp` and `4444/udp`.

For friends connecting from **outside your home/office network over the Internet**:

1. Log into your edge router's **Port Forwarding / NAT** page.
2. Add **TCP + UDP port-forwarding on port `4444`** to `192.168.11.24`:
   - **Service Name**: `ELLA World Standalone Rust Server`
   - **Protocol**: **`TCP & UDP`**
   - **External / WAN Port**: **`4444`**
   - **Internal / LAN IP**: **`192.168.11.24`**
   - **Internal Port**: **`4444`**
3. *(Optional)* Add **`44445/udp`** -> `192.168.11.24:44445` only if connecting native non-browser `EWLD` UDP clients from outside your LAN.
4. Set `ELLA_PUBLIC_IP=<YOUR_WAN_PUBLIC_IPV4>` in `/etc/systemd/system/ella-world.service.d/override.conf` so `ella-world-server` embeds both your LAN IP (`host`) and WAN IP (`srflx`) UDP ICE candidates in every SDP answer.
5. Set `ELLA_TLS_CERT=/etc/ella-world/tls/fullchain.pem` and `ELLA_TLS_KEY=/etc/ella-world/tls/privkey.pem` so browsers trust HTTPS (`https://robertscreations.com:4444`) and grant WebRTC microphone permissions without using port `443`.

---

## 3. Linux Host Firewall Rules (`ufw` / `iptables`)

If `ufw` is active on your Linux server:

```bash
sudo ufw allow 4444/tcp comment "ELLA World HTTPS + WSS Spine"
sudo ufw allow 4444/udp comment "ELLA World WebRTC Voice + UDP Game State"
# Optional native raw UDP client port:
sudo ufw allow 44445/udp comment "ELLA World Native EWLD UDP"
```

Or with raw `iptables`:

```bash
sudo iptables -I INPUT -p tcp --dport 4444 -j ACCEPT
sudo iptables -I INPUT -p udp --dport 4444 -j ACCEPT
sudo iptables -I INPUT -p udp --dport 44445 -j ACCEPT
```

---

## 4. Verifying TLS + UDP Reachability

Check that `ella-world-server` is bound to both TCP and UDP on `:4444`:

```bash
ss -tlnpu | grep 4444
```

And check the HTTPS health endpoint directly:

```bash
curl -s https://robertscreations.com:4444/api/ella-world/health | jq .
```

Expected fields:
- `"engine": "ella-world-rust-udp"`
- `"version": "0.3.0"`
- `"wsSpineEnabled": true`
- `"zeroRenegotiationSfu": true`
- `"sseEnabled": false`
- `"httpPollingSyncEnabled": false`
- `"webrtcUdpPort": 4444`
- `"nativeUdpPort": 44445`

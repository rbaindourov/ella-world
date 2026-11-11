# Firewall, NAT Port-Forwarding & Reverse Proxy Guide

ELLA World uses **HTTPS (`TCP 443`)** once on page load (so modern browsers permit microphone capture via `navigator.mediaDevices.getUserMedia`) and **UDP (`44444/udp`)** for all continuous voice audio and 20 Hz 3D game state synchronization.

---

## 1. Required Ports Matrix

| Port | Protocol | Direction | Required For | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| **`443`** | **TCP** | Inbound (WAN & LAN) | **Required** | HTTPS static WebGL client, login, and initial WebRTC SDP offer/answer via Nginx reverse proxy to `127.0.0.1:8097` |
| **`44444`** | **UDP** | Inbound / Bi-directional | **Required for WAN Friends** | Single-port multiplexed WebRTC UDP socket carrying **both** Opus Voice (`DTLS-SRTP`) and 20 Hz Binary Game State (`SCTP/DTLS`) |
| **`44445`** | **UDP** | Inbound / Bi-directional | *Optional* | Native raw `EWLD` v1 UDP socket for non-browser desktop/CLI bots |
| `8097` | TCP | Loopback (`127.0.0.1`) | Internal only | Local HTTP upstream between Nginx and `ella-world-server` (do **not** expose `8097/tcp` to WAN) |

---

## 2. Home / Edge Router NAT Port-Forwarding (Internet Play)

For players on the **same local LAN / Wi-Fi**, zero router changes are required as long as the server's OS firewall allows `44444/udp` — browsers automatically select the server's LAN `host` UDP candidate (`<1 ms` latency).

For friends connecting from **outside your home/office network over the Internet**:

1. Log into your edge router's **Port Forwarding / NAT** page.
2. Add **one UDP port-forwarding rule**:
   - **Service Name**: `ELLA World UDP Voice & State`
   - **Protocol**: **`UDP`** *(not TCP only)*
   - **External / WAN Port**: **`44444`**
   - **Internal / LAN IP**: `<YOUR_SERVER_LAN_IP>`
   - **Internal Port**: **`44444`**
3. *(Optional)* Add a second UDP rule for **`44445/udp`** -> `<YOUR_SERVER_LAN_IP>:44445` only if you plan to connect native non-browser UDP clients from outside your LAN.
4. Set `ELLA_PUBLIC_IP=<YOUR_WAN_PUBLIC_IPV4>` in `/etc/systemd/system/ella-world.service` so `ella-world-server` embeds both your LAN IP (`host`) and WAN IP (`srflx`) UDP ICE candidates in every SDP answer.

---

## 3. Linux Host Firewall Rules (`ufw` / `iptables`)

If `ufw` is active on your Linux server:

```bash
sudo ufw allow 44444/udp comment "ELLA World WebRTC Voice + UDP Game State"
# Optional native raw UDP client port:
sudo ufw allow 44445/udp comment "ELLA World Native EWLD UDP"
```

Or with raw `iptables`:

```bash
sudo iptables -I INPUT -p udp --dport 44444 -j ACCEPT
sudo iptables -I INPUT -p udp --dport 44445 -j ACCEPT
```

---

## 4. Verifying UDP Reachability

Check that `ella-world-server` is bound to both UDP sockets on the server:

```bash
ss -ulnp | grep -E '44444|44445'
```

And check the health endpoint:

```bash
curl -s http://127.0.0.1:8097/api/ella-world/health | jq .
```

Expected fields:
- `"engine": "ella-world-rust-udp"`
- `"sseEnabled": false`
- `"httpPollingSyncEnabled": false`
- `"webrtcUdpPort": 44444`
- `"nativeUdpPort": 44445`

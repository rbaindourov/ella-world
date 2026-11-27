//! Native `EWLD` v1 Binary UDP + Pure-Rust RFC 6716 Opus Voice GDExtension (`EllaNativeNet`)
//!
//! Targets:
//! - `aarch64-linux-android` (Android phones & tablets)
//! - `x86_64-linux-android`  (Android emulators)
//! - `x86_64-unknown-linux-gnu` (Headless Godot 4 CI & desktop verification)

use godot::prelude::*;
use opus_pure::{Application, MAX_PACKET_BYTES, MAX_PACKET_SAMPLES, OpusDecoder, OpusEncoder};
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

const NATIVE_MAGIC: [u8; 4] = *b"EWLD";
const PROTOCOL_VERSION: u8 = 1;

const OP_HELLO: u8 = 0x01;
const OP_WELCOME: u8 = 0x02;
const OP_C2S_STATE: u8 = 0x10;
const OP_S2C_WORLD_STATE: u8 = 0x11;
const OP_C2S_PING: u8 = 0x12;
const OP_S2C_PONG: u8 = 0x13;
const OP_VOICE_OPUS: u8 = 0x20;
const OP_EVENT_JSON: u8 = 0x30;
const OP_EVENT_ACK: u8 = 0x31;

const VOICE_FLAG_MUTED: u8 = 0b0000_0001;
const VOICE_FLAG_SPEAKING: u8 = 0b0000_0010;
const ZONE_FLAG_SHIFT: u8 = 2;
const ZONE_FLAG_MASK: u8 = 0b1111_1100;

const OPUS_SAMPLE_RATE: i32 = 48_000;
const OPUS_CHANNELS: usize = 2;
const OPUS_FRAME_SAMPLES: usize = 960; // 20 ms @ 48 kHz

struct EllaNativeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EllaNativeExtension {}

#[derive(Clone)]
struct PendingEventPacket {
    framed: Vec<u8>,
    sent_at_ms: u64,
    retries: u8,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn anim_to_code(anim: &str) -> u8 {
    match anim {
        "walk" => 1,
        "run" => 2,
        "jump" => 3,
        "hover" => 4,
        _ => 0,
    }
}

fn code_to_anim(code: u8) -> &'static str {
    match code {
        1 => "walk",
        2 => "run",
        3 => "jump",
        4 => "hover",
        _ => "idle",
    }
}

fn emote_to_code(emote: &str) -> u8 {
    match emote {
        "pose" => 1,
        "peace" => 2,
        "dance" => 3,
        "spin" => 4,
        "wave" => 5,
        "cheer" => 6,
        "flip" => 7,
        "sit" => 8,
        "sleep" => 9,
        _ => 0,
    }
}

fn code_to_emote(code: u8) -> &'static str {
    match code {
        1 => "pose",
        2 => "peace",
        3 => "dance",
        4 => "spin",
        5 => "wave",
        6 => "cheer",
        7 => "flip",
        8 => "sit",
        9 => "sleep",
        _ => "",
    }
}

fn pack_state_flags(muted: bool, speaking: bool, zone: u8) -> u8 {
    let mut flags = (zone << ZONE_FLAG_SHIFT) & ZONE_FLAG_MASK;
    if muted {
        flags |= VOICE_FLAG_MUTED;
    }
    if speaking {
        flags |= VOICE_FLAG_SPEAKING;
    }
    flags
}

fn wrap_native_packet(pkt_type: u8, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(6 + body.len());
    out.extend_from_slice(&NATIVE_MAGIC);
    out.push(PROTOCOL_VERSION);
    out.push(pkt_type);
    out.extend_from_slice(body);
    out
}

fn unwrap_native_packet(raw: &[u8]) -> Option<(u8, &[u8])> {
    if raw.len() < 6 || raw[0..4] != NATIVE_MAGIC || raw[4] != PROTOCOL_VERSION {
        return None;
    }
    Some((raw[5], &raw[6..]))
}

fn build_hello_datagram(token: &str) -> Vec<u8> {
    let tb = token.as_bytes();
    let len = tb.len().min(u16::MAX as usize);
    let mut body = Vec::with_capacity(2 + len);
    body.extend_from_slice(&(len as u16).to_le_bytes());
    body.extend_from_slice(&tb[..len]);
    wrap_native_packet(OP_HELLO, &body)
}

#[allow(clippy::too_many_arguments)]
fn build_c2s_state_body(
    seq: u32,
    client_time_ms: u32,
    x: f32,
    y: f32,
    z: f32,
    ry: f32,
    anim: &str,
    emote: &str,
    muted: bool,
    speaking: bool,
    zone: u8,
) -> [u8; 28] {
    let mut buf = [0u8; 28];
    buf[0] = OP_C2S_STATE;
    buf[1..5].copy_from_slice(&seq.to_le_bytes());
    buf[5..9].copy_from_slice(&client_time_ms.to_le_bytes());
    buf[9..13].copy_from_slice(&x.to_le_bytes());
    buf[13..17].copy_from_slice(&y.to_le_bytes());
    buf[17..21].copy_from_slice(&z.to_le_bytes());
    buf[21..25].copy_from_slice(&ry.to_le_bytes());
    buf[25] = anim_to_code(anim);
    buf[26] = emote_to_code(emote);
    buf[27] = pack_state_flags(muted, speaking, zone);
    buf
}

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct EllaNativeNet {
    base: Base<RefCounted>,
    socket: Option<UdpSocket>,
    remote_addr: Option<SocketAddr>,
    token: String,
    welcomed: bool,
    self_id: String,
    blocks_version: u64,
    next_event_seq: u32,
    pending_events: HashMap<u32, PendingEventPacket>,
    voice_seq: u16,
    voice_timestamp: u32,
    mic_interleaved_buf: Vec<f32>,
    opus_encoder: Option<OpusEncoder>,
    opus_decoders: HashMap<String, OpusDecoder>,
    last_rtt_ms: f32,
}

#[godot_api]
impl IRefCounted for EllaNativeNet {
    fn init(base: Base<RefCounted>) -> Self {
        let opus_encoder = OpusEncoder::new(OPUS_SAMPLE_RATE, OPUS_CHANNELS, Application::Voip)
            .ok()
            .map(|mut enc| {
                enc.bitrate_bps = 32_000;
                enc
            });
        Self {
            base,
            socket: None,
            remote_addr: None,
            token: String::new(),
            welcomed: false,
            self_id: String::new(),
            blocks_version: 0,
            next_event_seq: 1,
            pending_events: HashMap::new(),
            voice_seq: 0,
            voice_timestamp: 0,
            mic_interleaved_buf: Vec::with_capacity(OPUS_FRAME_SAMPLES * OPUS_CHANNELS * 4),
            opus_encoder,
            opus_decoders: HashMap::new(),
            last_rtt_ms: 0.0,
        }
    }
}

#[godot_api]
impl EllaNativeNet {
    /// Connects non-blocking native UDP socket to `<host>:<port>` (default `:44445/udp`)
    /// and dispatches the initial `EWLD` v1 `OP_HELLO` authentication datagram.
    #[func]
    pub fn connect_udp(&mut self, host: GString, port: i64, token: GString) -> bool {
        self.disconnect_udp();
        let host_str = host.to_string();
        let token_str = token.to_string();
        if host_str.trim().is_empty() || token_str.trim().is_empty() || !(1..=65535).contains(&port) {
            return false;
        }
        let target_spec = format!("{}:{}", host_str.trim(), port);
        let Ok(mut addrs) = target_spec.to_socket_addrs() else {
            return false;
        };
        let resolved: Vec<SocketAddr> = addrs.by_ref().collect();
        let target = resolved
            .iter()
            .find(|a| a.is_ipv4())
            .copied()
            .or_else(|| resolved.first().copied());
        let Some(remote) = target else {
            return false;
        };
        let bind_addr = if remote.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let Ok(sock) = UdpSocket::bind(bind_addr) else {
            return false;
        };
        if sock.set_nonblocking(true).is_err() {
            return false;
        }
        let _ = sock.connect(remote);
        let hello = build_hello_datagram(&token_str);
        let _ = sock.send(&hello);

        self.socket = Some(sock);
        self.remote_addr = Some(remote);
        self.token = token_str;
        self.welcomed = false;
        self.pending_events.clear();
        self.mic_interleaved_buf.clear();
        true
    }

    #[func]
    pub fn disconnect_udp(&mut self) {
        self.socket = None;
        self.remote_addr = None;
        self.token.clear();
        self.welcomed = false;
        self.self_id.clear();
        self.pending_events.clear();
        self.mic_interleaved_buf.clear();
        self.opus_decoders.clear();
    }

    #[func]
    pub fn is_welcomed(&self) -> bool {
        self.welcomed
    }

    #[func]
    pub fn get_rtt_ms(&self) -> f64 {
        self.last_rtt_ms as f64
    }

    #[func]
    pub fn resend_hello(&self) -> bool {
        let Some(sock) = &self.socket else {
            return false;
        };
        if self.token.is_empty() {
            return false;
        }
        let hello = build_hello_datagram(&self.token);
        sock.send(&hello).is_ok()
    }

    /// Encodes and sends a 34-byte `EWLD` v1 `OP_C2S_STATE` (`0x10`) movement/anim/emote/voice-flag frame.
    #[func]
    #[allow(clippy::too_many_arguments)]
    pub fn send_c2s_state(
        &self,
        seq: i64,
        client_time_ms: i64,
        x: f64,
        y: f64,
        z: f64,
        ry: f64,
        anim: GString,
        emote: GString,
        muted: bool,
        speaking: bool,
        zone: i64,
    ) -> bool {
        let Some(sock) = &self.socket else {
            return false;
        };
        let body = build_c2s_state_body(
            seq as u32,
            client_time_ms as u32,
            x as f32,
            y as f32,
            z as f32,
            ry as f32,
            &anim.to_string(),
            &emote.to_string(),
            muted,
            speaking,
            (zone.clamp(0, 15)) as u8,
        );
        let framed = wrap_native_packet(OP_C2S_STATE, &body);
        sock.send(&framed).is_ok()
    }

    #[func]
    pub fn send_ping(&self, client_time_ms: i64) -> bool {
        let Some(sock) = &self.socket else {
            return false;
        };
        let ts = (client_time_ms as u32).to_le_bytes();
        let framed = wrap_native_packet(OP_C2S_PING, &ts);
        sock.send(&framed).is_ok()
    }

    /// Sends a reliable acknowledged discrete JSON command (`OP_EVENT_JSON` `0x30`).
    #[func]
    pub fn send_event_json(&mut self, json_payload: GString) -> i64 {
        let Some(sock) = &self.socket else {
            return 0;
        };
        let bytes = json_payload.to_string().into_bytes();
        if bytes.is_empty() || bytes.len() > 8192 {
            return 0;
        }
        let seq = self.next_event_seq;
        self.next_event_seq = self.next_event_seq.wrapping_add(1).max(1);
        let mut body = Vec::with_capacity(4 + bytes.len());
        body.extend_from_slice(&seq.to_le_bytes());
        body.extend_from_slice(&bytes);
        let framed = wrap_native_packet(OP_EVENT_JSON, &body);
        let _ = sock.send(&framed);
        self.pending_events.insert(
            seq,
            PendingEventPacket {
                framed,
                sent_at_ms: now_ms(),
                retries: 0,
            },
        );
        seq as i64
    }

    /// Accepts 48 kHz stereo microphone frames from Godot `AudioEffectCapture`, encodes 20 ms
    /// RFC 6716 Opus packets via `opus-pure`, and transmits `OP_VOICE_OPUS` (`0x20`) over `:44445/udp`.
    #[func]
    pub fn push_mic_stereo_frames(&mut self, frames: PackedVector2Array) -> i64 {
        let Some(sock) = &self.socket else {
            return 0;
        };
        if frames.is_empty() {
            return 0;
        }
        for v in frames.as_slice() {
            self.mic_interleaved_buf.push(v.x.clamp(-1.0, 1.0));
            self.mic_interleaved_buf.push(v.y.clamp(-1.0, 1.0));
        }

        let Some(enc) = &mut self.opus_encoder else {
            self.mic_interleaved_buf.clear();
            return 0;
        };

        let chunk_floats = OPUS_FRAME_SAMPLES * OPUS_CHANNELS;
        let mut opus_out = [0u8; MAX_PACKET_BYTES];
        let mut packets_sent = 0i64;

        while self.mic_interleaved_buf.len() >= chunk_floats {
            let block: Vec<f32> = self.mic_interleaved_buf.drain(..chunk_floats).collect();
            if let Ok(encoded_len) = enc.encode(&block, OPUS_FRAME_SAMPLES, &mut opus_out) {
                if encoded_len > 0 {
                    self.voice_seq = self.voice_seq.wrapping_add(1);
                    self.voice_timestamp =
                        self.voice_timestamp.wrapping_add(OPUS_FRAME_SAMPLES as u32);
                    let mut body = Vec::with_capacity(6 + encoded_len);
                    body.extend_from_slice(&self.voice_seq.to_le_bytes());
                    body.extend_from_slice(&self.voice_timestamp.to_le_bytes());
                    body.extend_from_slice(&opus_out[..encoded_len]);
                    let framed = wrap_native_packet(OP_VOICE_OPUS, &body);
                    if sock.send(&framed).is_ok() {
                        packets_sent += 1;
                    }
                }
            }
        }

        packets_sent
    }

    /// Drains all inbound UDP datagrams, retransmits unacknowledged `OP_EVENT_JSON` frames,
    /// decodes inbound RFC 6716 Opus voice packets to 48 kHz stereo `PackedVector2Array`,
    /// and returns an array of event dictionaries for `NetworkClient.gd`.
    #[func]
    pub fn poll_packets(&mut self, current_client_time_ms: i64) -> VarArray {
        let mut events = VarArray::new();
        let mut recv_buf = [0u8; 4096];

        // 1. Drain inbound UDP datagrams first without holding immutable borrow across `&mut self`
        let mut raw_packets: Vec<Vec<u8>> = Vec::new();
        if let Some(sock) = &self.socket {
            while let Ok(len) = sock.recv(&mut recv_buf) {
                if len >= 6 {
                    raw_packets.push(recv_buf[..len].to_vec());
                }
                if raw_packets.len() >= 96 {
                    break;
                }
            }
        } else {
            return events;
        }

        for raw in raw_packets {
            let Some((pkt_type, body)) = unwrap_native_packet(&raw) else {
                continue;
            };
            match pkt_type {
                OP_WELCOME => {
                    if body.len() >= 9 {
                        let mut ver_bytes = [0u8; 8];
                        ver_bytes.copy_from_slice(&body[0..8]);
                        let blocks_ver = u64::from_le_bytes(ver_bytes);
                        let id_len = body[8] as usize;
                        if body.len() >= 9 + id_len {
                            if let Ok(self_id) = std::str::from_utf8(&body[9..9 + id_len]) {
                                self.welcomed = true;
                                self.self_id = self_id.to_string();
                                self.blocks_version = blocks_ver;
                                let mut d = VarDictionary::new();
                                d.set("kind", "welcome");
                                d.set("selfId", self_id);
                                d.set("blocksVersion", blocks_ver as i64);
                                events.push(&d.to_variant());
                            }
                        }
                    }
                }
                OP_S2C_WORLD_STATE => {
                    if let Some(d) =
                        Self::decode_world_state_dict(body, current_client_time_ms as u32)
                    {
                        if let Some(rtt_var) = d.get("rttMs") {
                            if let Ok(rtt) = rtt_var.try_to::<f64>() {
                                if rtt > 0.0 && rtt < 5_000.0 {
                                    self.last_rtt_ms = rtt as f32;
                                }
                            }
                        }
                        events.push(&d.to_variant());
                    }
                }
                OP_S2C_PONG => {
                    let payload = if body.first() == Some(&OP_S2C_PONG) {
                        &body[1..]
                    } else {
                        body
                    };
                    if payload.len() >= 8 {
                        let echo_ts =
                            u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                        let rtt = (current_client_time_ms as u32).wrapping_sub(echo_ts) as f64;
                        if (0.0..5_000.0).contains(&rtt) {
                            self.last_rtt_ms = rtt as f32;
                        }
                        let mut d = VarDictionary::new();
                        d.set("kind", "pong");
                        d.set("rttMs", rtt);
                        events.push(&d.to_variant());
                    }
                }
                OP_EVENT_ACK => {
                    if body.len() >= 4 {
                        let ack_seq = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
                        self.pending_events.remove(&ack_seq);
                    }
                }
                OP_EVENT_JSON => {
                    if body.len() > 4 {
                        if let Ok(json_str) = std::str::from_utf8(&body[4..]) {
                            let mut d = VarDictionary::new();
                            d.set("kind", "event_json");
                            d.set("json", json_str);
                            events.push(&d.to_variant());
                        }
                    }
                }
                OP_VOICE_OPUS => {
                    if body.len() > 7 {
                        let id_len = body[6] as usize;
                        if body.len() > 7 + id_len {
                            if let Ok(from_id) = std::str::from_utf8(&body[7..7 + id_len]) {
                                let opus_slice = &body[7 + id_len..];
                                let dec_opt = self
                                    .opus_decoders
                                    .entry(from_id.to_string())
                                    .or_insert_with(|| {
                                        OpusDecoder::new(OPUS_SAMPLE_RATE, OPUS_CHANNELS)
                                            .expect("valid 48kHz stereo OpusDecoder")
                                    });
                                let mut pcm_out = vec![0.0f32; MAX_PACKET_SAMPLES * OPUS_CHANNELS];
                                if let Ok(samples_per_ch) =
                                    dec_opt.decode(opus_slice, MAX_PACKET_SAMPLES, &mut pcm_out)
                                {
                                    if samples_per_ch > 0 {
                                        let mut frames = PackedVector2Array::new();
                                        frames.resize(samples_per_ch);
                                        let slice_mut = frames.as_mut_slice();
                                        for i in 0..samples_per_ch {
                                            slice_mut[i] = Vector2::new(
                                                pcm_out[i * 2].clamp(-1.0, 1.0),
                                                pcm_out[i * 2 + 1].clamp(-1.0, 1.0),
                                            );
                                        }
                                        let mut d = VarDictionary::new();
                                        d.set("kind", "voice_pcm");
                                        d.set("fromId", from_id);
                                        d.set("frames", &frames);
                                        events.push(&d.to_variant());
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // 2. Retransmit unacked `OP_EVENT_JSON` frames after 260 ms (up to 4 retries)
        let now = now_ms();
        if let Some(sock) = &self.socket {
            let mut expired = Vec::new();
            for (seq, item) in &mut self.pending_events {
                if now.saturating_sub(item.sent_at_ms) >= 260 {
                    if item.retries >= 4 {
                        expired.push(*seq);
                    } else {
                        let _ = sock.send(&item.framed);
                        item.sent_at_ms = now;
                        item.retries += 1;
                    }
                }
            }
            for seq in expired {
                self.pending_events.remove(&seq);
            }
        }

        events
    }

    /// Codec helper exposed for headless Godot unit tests (`run_godot_tests.gd`).
    #[func]
    #[allow(clippy::too_many_arguments)]
    pub fn encode_c2s_state_packet(
        &self,
        seq: i64,
        client_time_ms: i64,
        x: f64,
        y: f64,
        z: f64,
        ry: f64,
        anim: GString,
        emote: GString,
        muted: bool,
        speaking: bool,
        zone: i64,
    ) -> PackedByteArray {
        let body = build_c2s_state_body(
            seq as u32,
            client_time_ms as u32,
            x as f32,
            y as f32,
            z as f32,
            ry as f32,
            &anim.to_string(),
            &emote.to_string(),
            muted,
            speaking,
            zone.clamp(0, 15) as u8,
        );
        let framed = wrap_native_packet(OP_C2S_STATE, &body);
        PackedByteArray::from(framed.as_slice())
    }

    fn decode_world_state_dict(raw: &[u8], now_client_ms: u32) -> Option<VarDictionary> {
        let body = if raw.first() == Some(&OP_S2C_WORLD_STATE) {
            raw
        } else {
            return None;
        };
        if body.len() < 11 {
            return None;
        }
        let server_tick = u32::from_le_bytes([body[1], body[2], body[3], body[4]]);
        let echo_client_ms = u32::from_le_bytes([body[5], body[6], body[7], body[8]]);
        let count = u16::from_le_bytes([body[9], body[10]]) as usize;
        let mut offset = 11usize;
        let mut players = VarArray::new();

        for _ in 0..count {
            if offset >= body.len() {
                return None;
            }
            let id_len = body[offset] as usize;
            offset += 1;
            if offset + id_len + 19 > body.len() {
                return None;
            }
            let id = std::str::from_utf8(&body[offset..offset + id_len]).ok()?;
            offset += id_len;

            let x = f32::from_le_bytes([
                body[offset],
                body[offset + 1],
                body[offset + 2],
                body[offset + 3],
            ]);
            let y = f32::from_le_bytes([
                body[offset + 4],
                body[offset + 5],
                body[offset + 6],
                body[offset + 7],
            ]);
            let z = f32::from_le_bytes([
                body[offset + 8],
                body[offset + 9],
                body[offset + 10],
                body[offset + 11],
            ]);
            let ry = f32::from_le_bytes([
                body[offset + 12],
                body[offset + 13],
                body[offset + 14],
                body[offset + 15],
            ]);
            let anim = code_to_anim(body[offset + 16]);
            let emote = code_to_emote(body[offset + 17]);
            let flags = body[offset + 18];
            offset += 19;

            let mut p = VarDictionary::new();
            p.set("id", id);
            p.set("x", x as f64);
            p.set("y", y as f64);
            p.set("z", z as f64);
            p.set("ry", ry as f64);
            p.set("anim", anim);
            p.set("emote", emote);
            p.set("muted", (flags & VOICE_FLAG_MUTED) != 0);
            p.set("speaking", (flags & VOICE_FLAG_SPEAKING) != 0);
            p.set("zone", ((flags & ZONE_FLAG_MASK) >> ZONE_FLAG_SHIFT) as i64);
            players.push(&p.to_variant());
        }

        let mut out = VarDictionary::new();
        out.set("kind", "world_state");
        out.set("serverTick", server_tick as i64);
        if echo_client_ms > 0 {
            let rtt = now_client_ms.wrapping_sub(echo_client_ms) as f64;
            if (0.0..5_000.0).contains(&rtt) {
                out.set("rttMs", rtt);
            }
        }
        out.set("players", &players);
        Some(out)
    }
}

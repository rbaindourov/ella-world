//! Binary UDP Wire Protocol (`EWLD` + Compact Binary State Codec)
//!
//! Shared between:
//! 1. Browser WebRTC `RTCDataChannel` (`ella-udp-state`, `ordered: false, maxRetransmits: 0`)
//!    over `44444/udp`
//! 2. Native Raw UDP Socket clients (`tokio::net::UdpSocket`) over `44445/udp`

use crate::models::PlayerRuntimeState;
use bytes::{BufMut, Bytes, BytesMut};

pub const NATIVE_MAGIC: [u8; 4] = *b"EWLD";
pub const PROTOCOL_VERSION: u8 = 1;

// Opcodes shared across WebRTC UDP DataChannel & Native UDP
pub const OP_HELLO: u8 = 0x01;
pub const OP_WELCOME: u8 = 0x02;
pub const OP_C2S_STATE: u8 = 0x10;
pub const OP_S2C_WORLD_STATE: u8 = 0x11;
pub const OP_C2S_PING: u8 = 0x12;
pub const OP_S2C_PONG: u8 = 0x13;
pub const OP_VOICE_OPUS: u8 = 0x20;
pub const OP_EVENT_JSON: u8 = 0x30;
pub const OP_EVENT_ACK: u8 = 0x31;

pub const VOICE_FLAG_MUTED: u8 = 0b0000_0001;
pub const VOICE_FLAG_SPEAKING: u8 = 0b0000_0010;

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedC2SState {
    pub seq: u32,
    pub client_time_ms: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ry: f32,
    pub anim: String,
    pub emote: Option<String>,
    pub muted: bool,
    pub speaking: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedPlayerFrame {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub ry: f32,
    pub anim: String,
    pub emote: Option<String>,
    pub muted: bool,
    pub speaking: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedS2CWorldState {
    pub server_tick: u32,
    pub echo_client_time_ms: u32,
    pub players: Vec<DecodedPlayerFrame>,
}

pub fn anim_to_code(anim: &str) -> u8 {
    match anim {
        "walk" => 1,
        "run" => 2,
        "jump" => 3,
        _ => 0,
    }
}

pub fn code_to_anim(code: u8) -> &'static str {
    match code {
        1 => "walk",
        2 => "run",
        3 => "jump",
        _ => "idle",
    }
}

pub fn emote_to_code(emote: Option<&str>) -> u8 {
    match emote {
        Some("pose") => 1,
        Some("peace") => 2,
        Some("dance") => 3,
        Some("spin") => 4,
        Some("wave") => 5,
        Some("cheer") => 6,
        Some("flip") => 7,
        Some("sit") => 8,
        _ => 0,
    }
}

pub fn code_to_emote(code: u8) -> Option<&'static str> {
    match code {
        1 => Some("pose"),
        2 => Some("peace"),
        3 => Some("dance"),
        4 => Some("spin"),
        5 => Some("wave"),
        6 => Some("cheer"),
        7 => Some("flip"),
        8 => Some("sit"),
        _ => None,
    }
}

/// Encodes a 28-byte `OP_C2S_STATE` binary datagram (used by tests & native clients).
pub fn encode_c2s_state(state: &DecodedC2SState) -> Bytes {
    let mut buf = BytesMut::with_capacity(28);
    buf.put_u8(OP_C2S_STATE);
    buf.put_u32_le(state.seq);
    buf.put_u32_le(state.client_time_ms);
    buf.put_f32_le(state.x);
    buf.put_f32_le(state.y);
    buf.put_f32_le(state.z);
    buf.put_f32_le(state.ry);
    buf.put_u8(anim_to_code(&state.anim));
    buf.put_u8(emote_to_code(state.emote.as_deref()));
    let mut flags = 0u8;
    if state.muted {
        flags |= VOICE_FLAG_MUTED;
    }
    if state.speaking {
        flags |= VOICE_FLAG_SPEAKING;
    }
    buf.put_u8(flags);
    buf.freeze()
}

/// Decodes a 28-byte `OP_C2S_STATE` datagram (`[0] == 0x10`).
pub fn decode_c2s_state(raw: &[u8]) -> Option<DecodedC2SState> {
    if raw.len() < 28 || raw[0] != OP_C2S_STATE {
        return None;
    }
    let seq = u32::from_le_bytes(raw[1..5].try_into().ok()?);
    let client_time_ms = u32::from_le_bytes(raw[5..9].try_into().ok()?);
    let x = f32::from_le_bytes(raw[9..13].try_into().ok()?);
    let y = f32::from_le_bytes(raw[13..17].try_into().ok()?);
    let z = f32::from_le_bytes(raw[17..21].try_into().ok()?);
    let ry = f32::from_le_bytes(raw[21..25].try_into().ok()?);
    if !x.is_finite() || !y.is_finite() || !z.is_finite() || !ry.is_finite() {
        return None;
    }
    let anim = code_to_anim(raw[25]).to_string();
    let emote = code_to_emote(raw[26]).map(str::to_string);
    let flags = raw[27];
    Some(DecodedC2SState {
        seq,
        client_time_ms,
        x: x.clamp(-80.0, 80.0),
        y: y.clamp(-20.0, 80.0),
        z: z.clamp(-80.0, 80.0),
        ry,
        anim,
        emote,
        muted: (flags & VOICE_FLAG_MUTED) != 0,
        speaking: (flags & VOICE_FLAG_SPEAKING) != 0,
    })
}

/// Encodes `OP_S2C_WORLD_STATE` (`0x11`) binary snapshot for a specific recipient
/// (excluding `exclude_self_id` and echoing their `echo_client_time_ms` for live RTT measurement).
pub fn encode_s2c_world_state(
    server_tick: u32,
    echo_client_time_ms: u32,
    players: &[PlayerRuntimeState],
    exclude_self_id: &str,
) -> Bytes {
    let others: Vec<&PlayerRuntimeState> = players
        .iter()
        .filter(|p| p.id != exclude_self_id)
        .take(64)
        .collect();

    let mut buf = BytesMut::with_capacity(11 + others.len() * 36);
    buf.put_u8(OP_S2C_WORLD_STATE);
    buf.put_u32_le(server_tick);
    buf.put_u32_le(echo_client_time_ms);
    buf.put_u16_le(others.len() as u16);

    for p in others {
        let id_bytes = p.id.as_bytes();
        let id_len = id_bytes.len().min(64);
        buf.put_u8(id_len as u8);
        buf.put_slice(&id_bytes[..id_len]);
        buf.put_f32_le(p.x);
        buf.put_f32_le(p.y);
        buf.put_f32_le(p.z);
        buf.put_f32_le(p.ry);
        buf.put_u8(anim_to_code(&p.anim));
        buf.put_u8(emote_to_code(p.emote.as_deref()));
        let mut flags = 0u8;
        if p.muted {
            flags |= VOICE_FLAG_MUTED;
        }
        if p.speaking {
            flags |= VOICE_FLAG_SPEAKING;
        }
        buf.put_u8(flags);
    }

    buf.freeze()
}

pub fn decode_s2c_world_state(raw: &[u8]) -> Option<DecodedS2CWorldState> {
    if raw.len() < 11 || raw[0] != OP_S2C_WORLD_STATE {
        return None;
    }
    let server_tick = u32::from_le_bytes(raw[1..5].try_into().ok()?);
    let echo_client_time_ms = u32::from_le_bytes(raw[5..9].try_into().ok()?);
    let count = u16::from_le_bytes(raw[9..11].try_into().ok()?) as usize;

    let mut offset = 11usize;
    let mut players = Vec::with_capacity(count.min(64));
    for _ in 0..count {
        if offset >= raw.len() {
            return None;
        }
        let id_len = raw[offset] as usize;
        offset += 1;
        if offset + id_len + 19 > raw.len() {
            return None;
        }
        let id = std::str::from_utf8(&raw[offset..offset + id_len])
            .ok()?
            .to_string();
        offset += id_len;

        let x = f32::from_le_bytes(raw[offset..offset + 4].try_into().ok()?);
        let y = f32::from_le_bytes(raw[offset + 4..offset + 8].try_into().ok()?);
        let z = f32::from_le_bytes(raw[offset + 8..offset + 12].try_into().ok()?);
        let ry = f32::from_le_bytes(raw[offset + 12..offset + 16].try_into().ok()?);
        let anim = code_to_anim(raw[offset + 16]).to_string();
        let emote = code_to_emote(raw[offset + 17]).map(str::to_string);
        let flags = raw[offset + 18];
        offset += 19;

        players.push(DecodedPlayerFrame {
            id,
            x,
            y,
            z,
            ry,
            anim,
            emote,
            muted: (flags & VOICE_FLAG_MUTED) != 0,
            speaking: (flags & VOICE_FLAG_SPEAKING) != 0,
        });
    }

    Some(DecodedS2CWorldState {
        server_tick,
        echo_client_time_ms,
        players,
    })
}

pub fn encode_pong(client_time_ms: u32, server_tick: u32) -> Bytes {
    let mut buf = BytesMut::with_capacity(9);
    buf.put_u8(OP_S2C_PONG);
    buf.put_u32_le(client_time_ms);
    buf.put_u32_le(server_tick);
    buf.freeze()
}

/// Wraps a payload with the 6-byte Native Raw UDP `EWLD` header (`[b"EWLD"][version][pkt_type][body]`).
pub fn wrap_native_packet(pkt_type: u8, body: &[u8]) -> Bytes {
    let mut buf = BytesMut::with_capacity(6 + body.len());
    buf.put_slice(&NATIVE_MAGIC);
    buf.put_u8(PROTOCOL_VERSION);
    buf.put_u8(pkt_type);
    buf.put_slice(body);
    buf.freeze()
}

/// Unwraps a 6-byte Native Raw UDP `EWLD` packet, returning `(pkt_type, body_slice)`.
pub fn unwrap_native_packet(raw: &[u8]) -> Option<(u8, &[u8])> {
    if raw.len() < 6 || raw[0..4] != NATIVE_MAGIC || raw[4] != PROTOCOL_VERSION {
        return None;
    }
    Some((raw[5], &raw[6..]))
}

pub fn encode_native_hello(token: &str) -> Bytes {
    let tb = token.as_bytes();
    let mut body = BytesMut::with_capacity(2 + tb.len());
    body.put_u16_le(tb.len() as u16);
    body.put_slice(tb);
    wrap_native_packet(OP_HELLO, &body)
}

pub fn encode_native_welcome(blocks_version: u64, self_id: &str) -> Bytes {
    let idb = self_id.as_bytes();
    let len = idb.len().min(64);
    let mut body = BytesMut::with_capacity(9 + len);
    body.put_u64_le(blocks_version);
    body.put_u8(len as u8);
    body.put_slice(&idb[..len]);
    wrap_native_packet(OP_WELCOME, &body)
}

pub fn encode_native_voice_opus(
    seq: u16,
    timestamp: u32,
    from_id: &str,
    opus_payload: &[u8],
) -> Bytes {
    let idb = from_id.as_bytes();
    let len = idb.len().min(64);
    let mut body = BytesMut::with_capacity(7 + len + opus_payload.len());
    body.put_u16_le(seq);
    body.put_u32_le(timestamp);
    body.put_u8(len as u8);
    body.put_slice(&idb[..len]);
    body.put_slice(opus_payload);
    wrap_native_packet(OP_VOICE_OPUS, &body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AvatarAppearance, SanitizedUser};

    #[test]
    fn roundtrip_c2s_and_s2c_binary_udp_state() {
        let input = DecodedC2SState {
            seq: 42,
            client_time_ms: 123_456,
            x: -14.5,
            y: 3.25,
            z: 22.0,
            ry: 1.57,
            anim: "run".into(),
            emote: Some("peace".into()),
            muted: false,
            speaking: true,
        };
        let encoded = encode_c2s_state(&input);
        assert_eq!(encoded.len(), 28);
        let decoded = decode_c2s_state(&encoded).expect("valid C2S state");
        assert_eq!(decoded, input);

        let players = vec![
            PlayerRuntimeState {
                id: "peer_ella_tab1".into(),
                user: SanitizedUser {
                    username: "ella".into(),
                    display_name: "Ella".into(),
                    role: "host".into(),
                    avatar: AvatarAppearance::default(),
                },
                avatar_hash: AvatarAppearance::default().short_hash(),
                x: decoded.x,
                y: decoded.y,
                z: decoded.z,
                ry: decoded.ry,
                anim: decoded.anim.clone(),
                emote: decoded.emote.clone(),
                muted: decoded.muted,
                speaking: decoded.speaking,
                last_seen: 1000,
                last_seq: 42,
                last_client_time_ms: 123_456,
                conn_epoch: 1,
                ws_connected: true,
                udp_fastlane_active: true,
                last_udp_seen_ms: 1000,
                disconnected_at_ms: None,
            },
            PlayerRuntimeState {
                id: "peer_maya_tab1".into(),
                user: SanitizedUser {
                    username: "maya".into(),
                    display_name: "Maya".into(),
                    role: "friend".into(),
                    avatar: AvatarAppearance::default(),
                },
                avatar_hash: AvatarAppearance::default().short_hash(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
                ry: 0.0,
                anim: "idle".into(),
                emote: None,
                muted: true,
                speaking: false,
                last_seen: 1000,
                last_seq: 1,
                last_client_time_ms: 999,
                conn_epoch: 1,
                ws_connected: true,
                udp_fastlane_active: true,
                last_udp_seen_ms: 1000,
                disconnected_at_ms: None,
            },
        ];

        let snap = encode_s2c_world_state(900, 999, &players, "peer_maya_tab1");
        let parsed = decode_s2c_world_state(&snap).expect("valid S2C snapshot");
        assert_eq!(parsed.server_tick, 900);
        assert_eq!(parsed.echo_client_time_ms, 999);
        assert_eq!(parsed.players.len(), 1);
        assert_eq!(parsed.players[0].id, "peer_ella_tab1");
        assert_eq!(parsed.players[0].anim, "run");
        assert_eq!(parsed.players[0].emote.as_deref(), Some("peace"));
        assert!(parsed.players[0].speaking);
        assert!(!parsed.players[0].muted);
    }
}

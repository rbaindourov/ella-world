//! Cryptographic Machine Digital Signatures & Anti-Alt-Evasion Engine (`src/device_sig.rs`).
//!
//! Combines three defense-in-depth layers to identify machines across logouts, storage wipes,
//! Incognito/Private windows, and alternate browsers on the same physical computer:
//! 1. **HMAC-SHA256 Server-Signed Machine Seal (`__Host-ewld_machine_seal` / `ewld_machine_seal`)**:
//!    Stored in an `HttpOnly; SameSite=Lax` cookie (`Secure` + `__Host-` prefix under TLS) plus
//!    redundant `localStorage` and `IndexedDB` vaults so JS storage clearing cannot forge or erase it.
//! 2. **Hardware + GPU + Audio + Display Digital Signature (`hw_exact_sig`)**:
//!    Keyed HMAC digest of WebGL GPU renderer/caps, Offscreen Canvas 2D hash, `OfflineAudioContext`
//!    DSP hash, orientation-invariant screen bounds, logical CPU cores, touch topology, OS, and timezone.
//! 3. **Cross-Browser Physical Machine + Subnet Signature (`hw_xbrowser_sig` & `net_os_sig`)**:
//!    Browser-engine-independent hardware & IP `/24` (`/64` IPv6) signature catching Chrome -> Safari/Firefox
//!    switches on the same computer while preserving zero false-positive collisions between distinct
//!    sibling devices that already hold separate valid `ewld_machine_seal` tokens.

use crate::models::{normalize_chat_mode, MachineDeviceRecord, UserAccount};
use axum::http::HeaderMap;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::net::{IpAddr, SocketAddr};
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

pub const MACHINE_SEAL_COOKIE_HOST: &str = "__Host-ewld_machine_seal";
pub const MACHINE_SEAL_COOKIE_PLAIN: &str = "ewld_machine_seal";

/// Request extension injected by the TCP / Native-TLS accept loop carrying the peer's real `SocketAddr`.
#[derive(Clone, Copy, Debug)]
pub struct ClientPeerAddr(pub SocketAddr);

/// Client-collected hardware, display, GPU, canvas, and audio telemetry (minimized & pre-hashed).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDeviceTelemetry {
    #[serde(default)]
    pub gpu_vendor: String,
    #[serde(default)]
    pub gpu_renderer: String,
    #[serde(default)]
    pub gl_caps: String,
    #[serde(default)]
    pub canvas_hash: String,
    #[serde(default)]
    pub audio_hash: String,
    #[serde(default)]
    pub screen_w: u32,
    #[serde(default)]
    pub screen_h: u32,
    #[serde(default)]
    pub color_depth: u32,
    #[serde(default)]
    pub pixel_ratio_x100: u32,
    #[serde(default)]
    pub cpu_cores: u32,
    #[serde(default)]
    pub device_memory_gb: u32,
    #[serde(default)]
    pub max_touch_points: u32,
    #[serde(default)]
    pub os_family: String,
    #[serde(default)]
    pub timezone: String,
    #[serde(default)]
    pub language: String,
}

impl ClientDeviceTelemetry {
    pub fn has_hardware_signals(&self) -> bool {
        (self.screen_w > 0 && self.screen_h > 0)
            || !self.gpu_renderer.trim().is_empty()
            || !self.canvas_hash.trim().is_empty()
            || !self.audio_hash.trim().is_empty()
            || self.cpu_cores > 0
    }

    pub fn normalized_screen(&self) -> String {
        if self.screen_w == 0 || self.screen_h == 0 {
            return "unknown".into();
        }
        let hi = self.screen_w.max(self.screen_h);
        let lo = self.screen_w.min(self.screen_h);
        format!("{hi}x{lo}")
    }

    pub fn canonical_os(&self, user_agent: Option<&str>) -> String {
        let raw = if !self.os_family.trim().is_empty() {
            self.os_family.trim().to_ascii_lowercase()
        } else {
            user_agent.unwrap_or("").to_ascii_lowercase()
        };
        canonicalize_os_str(&raw)
    }

    /// Browser-independent GPU silicon vendor so Chrome ANGLE (`ANGLE (Apple, Apple M2)`),
    /// Firefox (`Apple M2`), and Safari (`Apple GPU`) normalize to the same architecture family.
    pub fn gpu_arch_family(&self) -> String {
        let combined = format!(
            "{} {}",
            self.gpu_vendor.to_ascii_lowercase(),
            self.gpu_renderer.to_ascii_lowercase()
        );
        for (needle, tag) in [
            ("apple", "apple"),
            ("nvidia", "nvidia"),
            ("geforce", "nvidia"),
            ("rtx", "nvidia"),
            ("gtx", "nvidia"),
            ("radeon", "amd"),
            ("amd", "amd"),
            ("intel", "intel"),
            ("iris", "intel"),
            ("uhd graphics", "intel"),
            ("adreno", "qualcomm"),
            ("qualcomm", "qualcomm"),
            ("mali", "arm"),
            ("samsung", "samsung"),
            ("llvmpipe", "mesa_sw"),
            ("swiftshader", "swiftshader"),
        ] {
            if combined.contains(needle) {
                return tag.into();
            }
        }
        "generic".into()
    }

    pub fn summaries(&self, user_agent: Option<&str>) -> (String, String, String, String) {
        let os_fam = self.canonical_os(user_agent);
        let os_pretty = match os_fam.as_str() {
            "macos" => "macOS",
            "windows" => "Windows",
            "linux" => "Linux",
            "ios" => "iOS / iPadOS",
            "android" => "Android",
            "chromeos" => "ChromeOS",
            _ => "Device",
        };
        let gpu_clean = clean_gpu_renderer(&self.gpu_renderer, &self.gpu_vendor);
        let screen_str = if self.screen_w > 0 && self.screen_h > 0 {
            format!("{}×{}", self.screen_w, self.screen_h)
        } else {
            "—".into()
        };
        let label = if !gpu_clean.is_empty() && screen_str != "—" {
            format!("{os_pretty} • {gpu_clean} • {screen_str}")
        } else if !gpu_clean.is_empty() {
            format!("{os_pretty} • {gpu_clean}")
        } else if screen_str != "—" {
            format!("{os_pretty} • {screen_str}")
        } else {
            format!("{os_pretty} Client")
        };
        (label, os_fam, gpu_clean, screen_str)
    }
}

fn clean_gpu_renderer(renderer: &str, vendor: &str) -> String {
    let trimmed = renderer
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(96)
        .collect::<String>();
    if !trimmed.is_empty() {
        // Strip verbose `ANGLE (...)` wrapper if present while preserving actual GPU chip model
        if let Some(inner) = trimmed
            .strip_prefix("ANGLE (")
            .and_then(|s| s.strip_suffix(')'))
        {
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
            if parts.len() >= 2 {
                let model = parts[1]
                    .strip_prefix("ANGLE Metal Renderer: ")
                    .unwrap_or(parts[1]);
                return model.chars().take(42).collect();
            }
        }
        return trimmed.chars().take(42).collect();
    }
    vendor
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(32)
        .collect()
}

pub fn canonicalize_os_str(raw_lc: &str) -> String {
    if raw_lc.contains("ipad") || raw_lc.contains("iphone") || raw_lc.contains("ios") {
        "ios".into()
    } else if raw_lc.contains("android") {
        "android".into()
    } else if raw_lc.contains("cros") || raw_lc.contains("chromeos") {
        "chromeos".into()
    } else if raw_lc.contains("mac") || raw_lc.contains("darwin") {
        "macos".into()
    } else if raw_lc.contains("win") {
        "windows".into()
    } else if raw_lc.contains("linux") || raw_lc.contains("x11") {
        "linux".into()
    } else {
        "unknown".into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSealClaims {
    pub device_id: String,
    pub hw_core_hash: String,
    pub iat: u64,
}

pub fn sign_machine_seal(claims: &DeviceSealClaims, secret: &str) -> String {
    let json_bytes =
        serde_json::to_vec(claims).expect("DeviceSealClaims serialization cannot fail");
    let encoded_payload = URL_SAFE_NO_PAD.encode(json_bytes);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any size");
    mac.update(b"ewld-seal-v1.");
    mac.update(encoded_payload.as_bytes());
    let encoded_sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    format!("{encoded_payload}.{encoded_sig}")
}

pub fn verify_machine_seal(token: &str, secret: &str) -> Option<DeviceSealClaims> {
    let (data, sig) = token.trim().split_once('.')?;
    if data.is_empty() || sig.is_empty() {
        return None;
    }
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(b"ewld-seal-v1.");
    mac.update(data.as_bytes());
    let expected_sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    if sig.len() != expected_sig.len() {
        return None;
    }
    if sig.as_bytes().ct_eq(expected_sig.as_bytes()).unwrap_u8() != 1 {
        return None;
    }
    let decoded = URL_SAFE_NO_PAD.decode(data).ok()?;
    let claims: DeviceSealClaims = serde_json::from_slice(&decoded).ok()?;
    if !claims.device_id.starts_with("dev_") || claims.device_id.len() < 12 {
        return None;
    }
    Some(claims)
}

pub fn extract_machine_seal_cookie(headers: &HeaderMap) -> Option<String> {
    let mut fallback_plain: Option<String> = None;
    for val in headers.get_all(axum::http::header::COOKIE) {
        let Ok(raw) = val.to_str() else {
            continue;
        };
        for pair in raw.split(';') {
            let Some((k, v)) = pair.trim().split_once('=') else {
                continue;
            };
            let key = k.trim();
            let value = v.trim();
            if value.is_empty() {
                continue;
            }
            if key == MACHINE_SEAL_COOKIE_HOST {
                return Some(value.to_string());
            }
            if key == MACHINE_SEAL_COOKIE_PLAIN && fallback_plain.is_none() {
                fallback_plain = Some(value.to_string());
            }
        }
    }
    fallback_plain
}

pub fn build_machine_seal_set_cookie(seal_token: &str, tls_enabled: bool) -> String {
    if tls_enabled {
        format!(
            "{MACHINE_SEAL_COOKIE_HOST}={seal_token}; SameSite=Lax; HttpOnly; Secure; Path=/; Max-Age=31536000"
        )
    } else {
        format!(
            "{MACHINE_SEAL_COOKIE_PLAIN}={seal_token}; SameSite=Lax; HttpOnly; Path=/; Max-Age=31536000"
        )
    }
}

/// Extracts the effective client IP. Honors `X-Forwarded-For` ONLY when the direct TCP socket peer
/// is loopback (`127.0.0.1` / `::1`) so remote clients can never spoof IP headers.
pub fn extract_effective_client_ip(
    headers: &HeaderMap,
    peer_addr: Option<SocketAddr>,
) -> Option<IpAddr> {
    let direct_ip = peer_addr.map(|s| canonical_ip(s.ip()));
    let allow_forwarded = direct_ip.map(|ip| ip.is_loopback()).unwrap_or(true);
    if allow_forwarded {
        if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(first) = xff.split(',').next() {
                if let Ok(parsed) = first.trim().parse::<IpAddr>() {
                    return Some(canonical_ip(parsed));
                }
            }
        }
    }
    direct_ip
}

fn canonical_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                IpAddr::V4(v4)
            } else {
                IpAddr::V6(v6)
            }
        }
        IpAddr::V4(v4) => IpAddr::V4(v4),
    }
}

pub fn subnet_bucket(ip: Option<IpAddr>) -> String {
    let Some(ip) = ip.map(canonical_ip) else {
        return "none".into();
    };
    match ip {
        IpAddr::V4(v4) => {
            if v4.is_private() || v4.is_link_local() {
                format!("v4:{v4}/32")
            } else {
                let o = v4.octets();
                format!("v4:{}.{}.{}.0/24", o[0], o[1], o[2])
            }
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("v6:{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

fn hmac_hex(secret: &str, domain: &[u8], parts: &[&str], byte_len: usize) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any size");
    mac.update(domain);
    for p in parts {
        mac.update(b"|");
        mac.update(p.trim().to_ascii_lowercase().as_bytes());
    }
    let out = mac.finalize().into_bytes();
    let take = byte_len.min(out.len());
    let mut hex = String::with_capacity(take * 2);
    for b in &out[..take] {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{b:02x}");
    }
    hex
}

pub fn compute_hw_exact_sig(
    tel: &ClientDeviceTelemetry,
    user_agent: Option<&str>,
    secret: &str,
) -> Option<String> {
    if !tel.has_hardware_signals() {
        return None;
    }
    let os = tel.canonical_os(user_agent);
    let scr = tel.normalized_screen();
    let cd = tel.color_depth.to_string();
    let dpr = tel.pixel_ratio_x100.to_string();
    let cores = tel.cpu_cores.to_string();
    let touch = tel.max_touch_points.to_string();
    Some(hmac_hex(
        secret,
        b"ewld-hw-exact-v1",
        &[
            &os,
            &scr,
            &cd,
            &dpr,
            &cores,
            &touch,
            &tel.timezone,
            &tel.gpu_vendor,
            &tel.gpu_renderer,
            &tel.gl_caps,
            &tel.canvas_hash,
            &tel.audio_hash,
        ],
        16,
    ))
}

pub fn compute_hw_xbrowser_sig(
    tel: &ClientDeviceTelemetry,
    user_agent: Option<&str>,
    peer_ip: Option<IpAddr>,
    secret: &str,
) -> Option<String> {
    if !tel.has_hardware_signals() {
        return None;
    }
    let sub = subnet_bucket(peer_ip);
    let os = tel.canonical_os(user_agent);
    let scr = tel.normalized_screen();
    let cores = tel.cpu_cores.to_string();
    let touch = tel.max_touch_points.to_string();
    let gpu_arch = tel.gpu_arch_family();
    Some(hmac_hex(
        secret,
        b"ewld-hw-xbrowser-v1",
        &[&sub, &os, &scr, &cores, &touch, &tel.timezone, &gpu_arch],
        16,
    ))
}

/// Network + OS fallback signature for non-loopback requests where a client stripped JS telemetry.
pub fn compute_net_os_fallback_sig(
    peer_ip: Option<IpAddr>,
    user_agent: Option<&str>,
    secret: &str,
) -> Option<String> {
    let ip = peer_ip.map(canonical_ip)?;
    if ip.is_loopback() {
        return None;
    }
    let ua_raw = user_agent?.trim();
    if ua_raw.is_empty() {
        return None;
    }
    let os = canonicalize_os_str(&ua_raw.to_ascii_lowercase());
    let ip_str = ip.to_string();
    Some(hmac_hex(secret, b"ewld-net-os-v1", &[&ip_str, &os], 16))
}

pub fn mint_deterministic_or_fresh_device_id(seed_material: &str, secret: &str) -> String {
    let hex = hmac_hex(secret, b"ewld-dev-id-v1", &[seed_material], 8);
    format!("dev_{hex}")
}

pub fn short_sig_from_device_id(device_id: &str) -> String {
    let raw = device_id.strip_prefix("dev_").unwrap_or(device_id);
    let take: String = raw.chars().take(8).collect();
    take.to_ascii_uppercase()
}

/// Returns all active judicial or parental restrictions on `user` that lock their machine against
/// switching to a friend's account until cleared or exempted by a parent.
pub fn active_penalties_for_user(user: &UserAccount) -> Vec<String> {
    let mut out = Vec::new();
    if user.in_debtors_prison {
        out.push("Debtor's Prison".into());
    }
    if user.suspended {
        out.push("Account Paused".into());
    }
    if user.coins < 0 {
        out.push(format!("Negative Debt ({} 🪙)", user.coins));
    }
    if user.voice_muted_by_parent {
        out.push("Parent Mic Mute".into());
    }
    let chat_mode = normalize_chat_mode(&user.chat_mode);
    if chat_mode != "normal" {
        out.push(format!("Chat Restricted ({chat_mode})"));
    }
    if !user.can_build {
        out.push("Build Locked".into());
    }
    if !user.can_trade {
        out.push("Trade Locked".into());
    }
    out
}

#[derive(Clone, Debug)]
pub struct ResolvedMachineMatch {
    pub index: Option<usize>,
    pub match_method: String,
    pub seal_claims_valid: bool,
}

/// Resolves an incoming login/resume request against the persisted `devices` registry.
///
/// **Sibling Zero-Collision Guarantee**:
/// - If the client presents a valid HMAC-verified `ewld_machine_seal` belonging to an existing
///   `MachineDeviceRecord` (`dev_B`), it resolves directly to `dev_B` and NEVER merges `dev_B`
///   into another sibling's `dev_A` even if both siblings own identical iPads on the same Wi-Fi!
/// - Only when NO valid `ewld_machine_seal` is presented (e.g., Incognito mode, cleared cookies/IndexedDB,
///   or switching to a fresh browser like Firefox/Safari on the penalized computer) does the server
///   cluster-match `hw_exact_sig`, `hw_xbrowser_sig`, or `net_os_sig` against existing machines
///   (prioritizing any machine that currently carries a penalized or banned lock).
pub fn match_existing_device(
    devices: &[MachineDeviceRecord],
    users: &[UserAccount],
    verified_seal: Option<&DeviceSealClaims>,
    hw_exact_sig: Option<&str>,
    hw_xbrowser_sig: Option<&str>,
    net_os_sig: Option<&str>,
    from_cookie: bool,
) -> ResolvedMachineMatch {
    if let Some(seal) = verified_seal {
        if let Some(idx) = devices.iter().position(|d| d.device_id == seal.device_id) {
            return ResolvedMachineMatch {
                index: Some(idx),
                match_method: if from_cookie {
                    "machine_seal_cookie".into()
                } else {
                    "machine_seal_vault".into()
                },
                seal_claims_valid: true,
            };
        }
        return ResolvedMachineMatch {
            index: None,
            match_method: if from_cookie {
                "machine_seal_cookie".into()
            } else {
                "machine_seal_vault".into()
            },
            seal_claims_valid: true,
        };
    }

    let device_has_lock = |d: &MachineDeviceRecord| -> bool {
        if d.banned {
            return true;
        }
        if d.trusted_shared {
            return false;
        }
        d.associated_users.iter().any(|uname| {
            users
                .iter()
                .find(|u| u.username.eq_ignore_ascii_case(uname))
                .map(|u| !active_penalties_for_user(u).is_empty())
                .unwrap_or(false)
        })
    };

    if let Some(exact) = hw_exact_sig.filter(|s| !s.is_empty()) {
        if let Some(idx) = devices
            .iter()
            .position(|d| d.hw_exact_sig == exact && device_has_lock(d))
            .or_else(|| devices.iter().position(|d| d.hw_exact_sig == exact))
        {
            return ResolvedMachineMatch {
                index: Some(idx),
                match_method: "hardware_exact_sig".into(),
                seal_claims_valid: false,
            };
        }
    }

    if let Some(xb) = hw_xbrowser_sig.filter(|s| !s.is_empty()) {
        if let Some(idx) = devices
            .iter()
            .position(|d| d.hw_xbrowser_sig == xb && device_has_lock(d))
        {
            return ResolvedMachineMatch {
                index: Some(idx),
                match_method: "hardware_cross_browser_sig".into(),
                seal_claims_valid: false,
            };
        }
    }

    if let Some(net) = net_os_sig.filter(|s| !s.is_empty()) {
        if let Some(idx) = devices
            .iter()
            .position(|d| d.net_os_sig == net && device_has_lock(d))
        {
            return ResolvedMachineMatch {
                index: Some(idx),
                match_method: "network_ip_fallback".into(),
                seal_claims_valid: false,
            };
        }
    }

    ResolvedMachineMatch {
        index: None,
        match_method: "new_machine".into(),
        seal_claims_valid: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AvatarAppearance;
    use std::collections::HashMap;

    fn sample_user(username: &str) -> UserAccount {
        UserAccount {
            username: username.into(),
            display_name: username.into(),
            password: "pw".into(),
            role: "friend".into(),
            token_epoch: 0,
            avatar: AvatarAppearance::default(),
            coins: 20,
            inventory: HashMap::new(),
            cloneable_items: Default::default(),
            violation_strikes: 0,
            in_debtors_prison: false,
            prison_reason: None,
            prison_tasks_remaining: 0,
            voice_muted_by_parent: false,
            chat_mode: "normal".into(),
            suspended: false,
            can_build: true,
            can_trade: true,
            last_pos: None,
            last_pos_saved_at: None,
        }
    }

    #[test]
    fn machine_seal_roundtrip_and_tamper_rejection() {
        let secret = "unit-test-secret";
        let claims = DeviceSealClaims {
            device_id: "dev_0123456789abcdef".into(),
            hw_core_hash: "aabbccddeeff0011".into(),
            iat: 1_700_000_000_000,
        };
        let tok = sign_machine_seal(&claims, secret);
        let parsed = verify_machine_seal(&tok, secret).expect("valid machine seal");
        assert_eq!(parsed.device_id, claims.device_id);
        assert_eq!(parsed.hw_core_hash, claims.hw_core_hash);

        // Tampered signature or wrong secret must fail
        assert!(verify_machine_seal(&format!("{tok}x"), secret).is_none());
        assert!(verify_machine_seal(&tok, "wrong-secret").is_none());
    }

    #[test]
    fn cross_browser_and_incognito_matching_without_false_sibling_collision() {
        let secret = "unit-test-secret";
        let ip: IpAddr = "192.168.11.55".parse().unwrap();

        // Chrome on Maya's MacBook M2
        let chrome_tel = ClientDeviceTelemetry {
            gpu_vendor: "Google Inc. (Apple)".into(),
            gpu_renderer: "ANGLE (Apple, ANGLE Metal Renderer: Apple M2, Unspecified Version)"
                .into(),
            gl_caps: "16384|16384|16".into(),
            canvas_hash: "c_chrome_111".into(),
            audio_hash: "a_coreaudio_222".into(),
            screen_w: 1512,
            screen_h: 982,
            color_depth: 30,
            pixel_ratio_x100: 200,
            cpu_cores: 8,
            device_memory_gb: 8,
            max_touch_points: 0,
            os_family: "macOS".into(),
            timezone: "America/Los_Angeles".into(),
            language: "en-US".into(),
        };

        // Safari on the EXACT SAME MacBook M2 (different canvasHash, masked "Apple GPU", no deviceMemory)
        let safari_tel = ClientDeviceTelemetry {
            gpu_vendor: "Apple Inc.".into(),
            gpu_renderer: "Apple GPU".into(),
            gl_caps: "16384|16384|16".into(),
            canvas_hash: "c_safari_999".into(),
            audio_hash: "a_safari_888".into(),
            screen_w: 1512,
            screen_h: 982,
            color_depth: 30,
            pixel_ratio_x100: 200,
            cpu_cores: 8,
            device_memory_gb: 0,
            max_touch_points: 0,
            os_family: "MacIntel".into(),
            timezone: "America/Los_Angeles".into(),
            language: "en-US".into(),
        };

        let chrome_exact = compute_hw_exact_sig(&chrome_tel, None, secret).unwrap();
        let chrome_xb = compute_hw_xbrowser_sig(&chrome_tel, None, Some(ip), secret).unwrap();
        let safari_xb = compute_hw_xbrowser_sig(&safari_tel, None, Some(ip), secret).unwrap();

        assert_eq!(
            chrome_xb, safari_xb,
            "Chrome and Safari on the same MacBook + subnet must produce the exact same hw_xbrowser_sig"
        );

        let mut maya = sample_user("maya");
        maya.in_debtors_prison = true;
        let chloe = sample_user("chloe");
        let users = vec![maya, chloe];

        let dev_a = MachineDeviceRecord {
            device_id: "dev_aaaaaaaaaaaaaaaa".into(),
            short_sig: "AAAAAAAA".into(),
            label: "macOS • Apple M2 • 1512×982".into(),
            os_family: "macos".into(),
            gpu_summary: "Apple M2".into(),
            screen_summary: "1512×982".into(),
            hw_exact_sig: chrome_exact.clone(),
            hw_xbrowser_sig: chrome_xb.clone(),
            net_os_sig: String::new(),
            last_ip: ip.to_string(),
            associated_users: vec!["maya".into()],
            last_username: "maya".into(),
            first_seen_ms: 1,
            last_seen_ms: 1,
            trusted_shared: false,
            banned: false,
        };
        let dev_b = MachineDeviceRecord {
            device_id: "dev_bbbbbbbbbbbbbbbb".into(),
            short_sig: "BBBBBBBB".into(),
            ..dev_a.clone()
        };
        let mut dev_b_clean = dev_b;
        dev_b_clean.associated_users = vec!["chloe".into()];
        dev_b_clean.last_username = "chloe".into();

        let devices = vec![dev_a, dev_b_clean];

        // 1. Safari opened without cookie/seal on Maya's MacBook matches penalized dev_a!
        let m_safari = match_existing_device(
            &devices,
            &users,
            None,
            compute_hw_exact_sig(&safari_tel, None, secret).as_deref(),
            Some(&safari_xb),
            None,
            false,
        );
        assert_eq!(m_safari.index, Some(0));
        assert_eq!(m_safari.match_method, "hardware_cross_browser_sig");

        // 2. Sibling Chloe on her own identical MacBook WITH her own valid seal (`dev_bbbb...`)
        // resolves cleanly to index 1 (dev_b) with ZERO false collision against Maya's dev_a!
        let chloe_seal = DeviceSealClaims {
            device_id: "dev_bbbbbbbbbbbbbbbb".into(),
            hw_core_hash: safari_xb,
            iat: 2,
        };
        let m_sibling =
            match_existing_device(&devices, &users, Some(&chloe_seal), None, None, None, true);
        assert_eq!(m_sibling.index, Some(1));
        assert_eq!(m_sibling.match_method, "machine_seal_cookie");
    }
}

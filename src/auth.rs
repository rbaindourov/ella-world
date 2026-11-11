use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenClaims {
    #[serde(default)]
    pub self_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    pub iat: u64,
}

pub fn sign_token(claims: &TokenClaims, secret: &str) -> String {
    let json_bytes = serde_json::to_vec(claims).expect("TokenClaims serialization cannot fail");
    let encoded_payload = URL_SAFE_NO_PAD.encode(json_bytes);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any size");
    mac.update(encoded_payload.as_bytes());
    let sig_bytes = mac.finalize().into_bytes();
    let encoded_sig = URL_SAFE_NO_PAD.encode(sig_bytes);
    format!("{encoded_payload}.{encoded_sig}")
}

pub fn verify_token(token: &str, secret: &str) -> Option<TokenClaims> {
    let (data, sig) = token.split_once('.')?;
    if data.is_empty() || sig.is_empty() {
        return None;
    }

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(data.as_bytes());
    let expected_sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    if sig.len() != expected_sig.len() {
        return None;
    }
    if sig.as_bytes().ct_eq(expected_sig.as_bytes()).unwrap_u8() != 1 {
        return None;
    }

    let decoded = URL_SAFE_NO_PAD.decode(data).ok()?;
    serde_json::from_slice(&decoded).ok()
}

pub fn constant_time_str_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes().ct_eq(b.as_bytes()).unwrap_u8() == 1
}

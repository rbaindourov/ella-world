use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub http_addr: SocketAddr,
    pub webrtc_udp_bind_ip: String,
    pub webrtc_udp_port: u16,
    pub native_udp_addr: SocketAddr,
    pub public_ip: Option<String>,
    pub data_dir: PathBuf,
    pub public_dir: PathBuf,
    pub seed_users_file: PathBuf,
    pub hmac_secret: String,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
}

pub const DEFAULT_TOKEN_TTL_SECS: u64 = 12 * 3600;

impl ServerConfig {
    pub fn token_ttl_ms(&self) -> u64 {
        let secs = env::var("ELLA_TOKEN_TTL_SECS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_TOKEN_TTL_SECS)
            .clamp(60, 30 * 24 * 3600);
        secs.saturating_mul(1_000)
    }

    pub fn from_env() -> Self {
        let http_addr: SocketAddr = env::var("ELLA_HTTP_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:8097".to_string())
            .parse()
            .expect("Invalid ELLA_HTTP_ADDR socket address");

        let webrtc_udp_bind_ip =
            env::var("ELLA_WEBRTC_BIND_IP").unwrap_or_else(|_| "0.0.0.0".to_string());

        let webrtc_udp_port: u16 = env::var("ELLA_WEBRTC_UDP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(44444);

        let native_udp_port: u16 = env::var("ELLA_NATIVE_UDP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(44445);

        let native_udp_addr: SocketAddr = env::var("ELLA_NATIVE_UDP_ADDR")
            .unwrap_or_else(|_| format!("0.0.0.0:{native_udp_port}"))
            .parse()
            .expect("Invalid ELLA_NATIVE_UDP_ADDR socket address");

        let public_ip = env::var("ELLA_PUBLIC_IP")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let data_dir = PathBuf::from(env::var("ELLA_DATA_DIR").unwrap_or_else(|_| "./data".into()));
        let public_dir =
            PathBuf::from(env::var("ELLA_PUBLIC_DIR").unwrap_or_else(|_| "./public".into()));
        let seed_users_file = PathBuf::from(
            env::var("ELLA_SEED_USERS_FILE")
                .unwrap_or_else(|_| "./config/users.example.json".into()),
        );

        let hmac_secret = env::var("ELLA_HMAC_SECRET")
            .unwrap_or_else(|_| "ella-world-rust-udp-dev-hmac-change-in-prod".to_string());

        let tls_cert_path = env::var("ELLA_TLS_CERT")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        let tls_key_path = env::var("ELLA_TLS_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        Self {
            http_addr,
            webrtc_udp_bind_ip,
            webrtc_udp_port,
            native_udp_addr,
            public_ip,
            data_dir,
            public_dir,
            seed_users_file,
            hmac_secret,
            tls_cert_path,
            tls_key_path,
        }
    }
}

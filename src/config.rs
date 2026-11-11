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
}

impl ServerConfig {
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
        }
    }
}

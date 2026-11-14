use ella_world::config::ServerConfig;
use ella_world::start_all_servers;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,webrtc_sctp=error,webrtc_ice=warn")),
        )
        .init();

    let config = ServerConfig::from_env();
    let running = start_all_servers(config).await?;

    let proto = if running.tls_enabled {
        "HTTPS+WSS"
    } else {
        "HTTP+WS"
    };
    info!(
        "✨ ELLA World Standalone Rust UDP Server online! {}={} | WebRTC_SFU={}/udp | Native_EWLD={}/udp",
        proto, running.http_addr, running.webrtc_udp_port, running.native_udp_addr
    );

    tokio::select! {
        res = running.http_task => {
            res??;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down ELLA World server cleanly...");
        }
    }

    Ok(())
}

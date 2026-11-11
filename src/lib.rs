pub mod auth;
pub mod config;
pub mod http_api;
pub mod models;
pub mod persistence;
pub mod udp_native;
pub mod udp_wire;
pub mod webrtc_sfu;
pub mod world;

use crate::config::ServerConfig;
use crate::http_api::{build_router, AppState};
use crate::udp_native::NativeUdpServer;
use crate::webrtc_sfu::WebRtcSfu;
use crate::world::WorldEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct RunningEllaWorld {
    pub http_addr: SocketAddr,
    pub webrtc_udp_port: u16,
    pub native_udp_addr: SocketAddr,
    pub world: Arc<WorldEngine>,
    pub sfu: Arc<WebRtcSfu>,
    pub native_udp: Arc<NativeUdpServer>,
    pub http_task: JoinHandle<anyhow::Result<()>>,
}

/// Starts the complete standalone ELLA World stack:
/// 1. `WorldEngine` (persistent users + blocks + 20 Hz simulation tick)
/// 2. `WebRtcSfu` (single-port multiplexed UDP Voice RTP + Binary UDP DataChannels on `webrtc_udp_port`)
/// 3. `NativeUdpServer` (raw `tokio::net::UdpSocket` `EWLD` v1 voice + state server on `native_udp_addr`)
/// 4. `axum` HTTP static asset + WebRTC SDP signaling server on `http_addr`
pub async fn start_all_servers(mut config: ServerConfig) -> anyhow::Result<RunningEllaWorld> {
    let pre_world = WorldEngine::new(config.clone())?;
    let sfu = WebRtcSfu::bind_and_start(Arc::clone(&pre_world)).await?;
    let native_udp = NativeUdpServer::bind_and_start(Arc::clone(&pre_world)).await?;

    // Reflect ephemeral port bindings if `0` was passed in integration tests
    config.webrtc_udp_port = sfu.udp_port();
    config.native_udp_addr = native_udp.local_addr();

    let tcp_listener = TcpListener::bind(config.http_addr).await?;
    let actual_http_addr = tcp_listener.local_addr()?;

    let router = build_router(AppState {
        world: Arc::clone(&pre_world),
        sfu: Arc::clone(&sfu),
        native_udp: Arc::clone(&native_udp),
    });

    let http_task = tokio::spawn(async move {
        axum::serve(tcp_listener, router).await?;
        Ok(())
    });

    Ok(RunningEllaWorld {
        http_addr: actual_http_addr,
        webrtc_udp_port: sfu.udp_port(),
        native_udp_addr: native_udp.local_addr(),
        world: pre_world,
        sfu,
        native_udp,
        http_task,
    })
}

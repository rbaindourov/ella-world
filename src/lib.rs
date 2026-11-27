pub mod auth;
pub mod config;
pub mod device_sig;
pub mod economy;
pub mod http_api;
pub mod models;
pub mod persistence;
pub mod playground;
pub mod udp_native;
pub mod udp_wire;
pub mod webrtc_sfu;
pub mod world;
pub mod ws_spine;

use crate::config::ServerConfig;
use crate::device_sig::ClientPeerAddr;
use crate::http_api::{build_router, AppState};
use crate::playground::PlaygroundRegistry;
use crate::udp_native::NativeUdpServer;
use crate::webrtc_sfu::WebRtcSfu;
use crate::world::WorldEngine;
use crate::ws_spine::WsSpineHub;
use anyhow::Context;
use axum::Router;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::rustls::crypto::ring;
use tokio_rustls::rustls::pki_types::CertificateDer;
use tokio_rustls::rustls::ServerConfig as RustlsServerConfig;
use tokio_rustls::TlsAcceptor;

pub struct RunningEllaWorld {
    pub http_addr: SocketAddr,
    pub tls_enabled: bool,
    pub webrtc_udp_port: u16,
    pub native_udp_addr: SocketAddr,
    pub world: Arc<WorldEngine>,
    pub registry: Arc<PlaygroundRegistry>,
    pub sfu: Arc<WebRtcSfu>,
    pub native_udp: Arc<NativeUdpServer>,
    pub ws_hub: Arc<WsSpineHub>,
    pub http_task: JoinHandle<anyhow::Result<()>>,
}

/// Starts the complete standalone ELLA World Multi-Playground Hybrid stack:
/// 1. `WorldEngine` + `PlaygroundRegistry` (default `"ella"` + isolated `data/playgrounds/<slug>/` worlds + Invite Ledger)
/// 2. `WebRtcSfu` (single-port multiplexed 8-slot zero-renegotiation UDP Voice + State fast-lane on `webrtc_udp_port`)
/// 3. `NativeUdpServer` (raw `tokio::net::UdpSocket` `EWLD` v1 voice + state server on `native_udp_addr`)
/// 4. `axum` HTTP or Native Rust TLS HTTPS + `TCP_NODELAY` Persistent Binary/JSON WebSocket Spine (`/api/ella-world/ws`) on `http_addr`
pub async fn start_all_servers(mut config: ServerConfig) -> anyhow::Result<RunningEllaWorld> {
    let pre_world = WorldEngine::new(config.clone())?;
    let registry = PlaygroundRegistry::new(Arc::clone(&pre_world)).await?;
    let sfu = WebRtcSfu::bind_and_start(Arc::clone(&registry)).await?;
    let native_udp = NativeUdpServer::bind_and_start(Arc::clone(&registry)).await?;
    let ws_hub = WsSpineHub::new();

    // Reflect ephemeral port bindings if `0` was passed in integration tests
    config.webrtc_udp_port = sfu.udp_port();
    config.native_udp_addr = native_udp.local_addr();

    let tls_acceptor = match (&config.tls_cert_path, &config.tls_key_path) {
        (Some(cert), Some(key)) => Some(build_tls_acceptor(cert, key)?),
        (Some(_), None) | (None, Some(_)) => {
            anyhow::bail!("Both ELLA_TLS_CERT and ELLA_TLS_KEY must be set together");
        }
        (None, None) => None,
    };
    let tls_enabled = tls_acceptor.is_some();

    let tcp_listener = TcpListener::bind(config.http_addr).await?;
    let actual_http_addr = tcp_listener.local_addr()?;

    let router = build_router(AppState {
        world: Arc::clone(&pre_world),
        registry: Arc::clone(&registry),
        sfu: Arc::clone(&sfu),
        native_udp: Arc::clone(&native_udp),
        ws_hub: Arc::clone(&ws_hub),
        tls_enabled,
    });

    let http_task = if let Some(acceptor) = tls_acceptor {
        tokio::spawn(async move { serve_https_loop(tcp_listener, acceptor, router).await })
    } else {
        tokio::spawn(async move { serve_http_loop(tcp_listener, router).await })
    };

    Ok(RunningEllaWorld {
        http_addr: actual_http_addr,
        tls_enabled,
        webrtc_udp_port: sfu.udp_port(),
        native_udp_addr: native_udp.local_addr(),
        world: pre_world,
        registry,
        sfu,
        native_udp,
        ws_hub,
        http_task,
    })
}

pub fn build_tls_acceptor(cert_path: &Path, key_path: &Path) -> anyhow::Result<TlsAcceptor> {
    let cert_file = File::open(cert_path)
        .with_context(|| format!("Failed to open TLS cert {}", cert_path.display()))?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("Failed to parse TLS certs from {}", cert_path.display()))?;
    if certs.is_empty() {
        anyhow::bail!("No certificates found in {}", cert_path.display());
    }

    let key_file = File::open(key_path)
        .with_context(|| format!("Failed to open TLS key {}", key_path.display()))?;
    let mut key_reader = BufReader::new(key_file);
    let key = rustls_pemfile::private_key(&mut key_reader)
        .with_context(|| format!("Failed to read TLS private key {}", key_path.display()))?
        .with_context(|| format!("No private key found in {}", key_path.display()))?;

    let mut tls_cfg = RustlsServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
        .with_safe_default_protocol_versions()
        .context("Failed to initialize TLS protocol versions")?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("Invalid TLS certificate/key pair")?;

    // Force HTTP/1.1 ALPN so RFC 6455 `wss://` WebSocket upgrades work natively over TLS
    tls_cfg.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(TlsAcceptor::from(Arc::new(tls_cfg)))
}

async fn serve_http_loop(listener: TcpListener, router: Router) -> anyhow::Result<()> {
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let _ = stream.set_nodelay(true);
        let router_for_conn = router
            .clone()
            .layer(axum::Extension(ClientPeerAddr(remote_addr)));

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = TowerToHyperService::new(router_for_conn);
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                tracing::debug!(?remote_addr, "HTTP connection ended: {err}");
            }
        });
    }
}

async fn serve_https_loop(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    router: Router,
) -> anyhow::Result<()> {
    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let _ = stream.set_nodelay(true);
        let acceptor = acceptor.clone();
        let router_for_conn = router
            .clone()
            .layer(axum::Extension(ClientPeerAddr(remote_addr)));

        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(s) => s,
                Err(err) => {
                    tracing::debug!(?remote_addr, "TLS handshake failed: {err}");
                    return;
                }
            };
            let io = TokioIo::new(tls_stream);
            let service = TowerToHyperService::new(router_for_conn);
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                tracing::debug!(?remote_addr, "HTTPS connection ended: {err}");
            }
        });
    }
}

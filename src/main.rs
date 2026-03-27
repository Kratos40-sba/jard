use dashmap::DashMap;
use jard::api::{router, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub mod api;
mod assets;
pub mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Jard — The Zero-Hardware Barcode Bridge...");

    // 2. Initialize State & Security Token
    use rand::distributions::{Alphanumeric, DistString};
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 12);

    let state = Arc::new(AppState {
        scans: DashMap::new(),
        access_token: token.clone(),
        rate_limiter: DashMap::new(),
    });

    // 3. Identify Local IP
    let my_local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    info!("--------------------------------------------------");
    info!(" PC Dashboard:   http://localhost:8080?token={}", token);
    info!(
        " Mobile Scanner: http://{}:8080/scanner?token={}",
        my_local_ip, token
    );
    info!("--------------------------------------------------");

    // 4. Auto-Open Browser
    let dashboard_url = format!("http://localhost:8080?token={}", token);
    if let Err(e) = open::that(dashboard_url) {
        tracing::warn!("Failed to open browser automatically: {}", e);
    }

    // 5. Run Server with Graceful Shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(|e| anyhow::anyhow!("Server runtime error: {}", e))?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { info!("Ctrl+C received, shutting down..."); },
        _ = terminate => { info!("SIGTERM received, shutting down..."); },
    }
}

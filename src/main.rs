use dashmap::DashMap;
use raf::api::{router, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting RAF (رف) — The Order Fulfillment Engine...");

    // 2. Initialize mDNS Discovery (jard.local)
    use mdns_sd::{ServiceDaemon, ServiceInfo};
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let service_type = "_http._tcp.local.";
    let instance_name = "raf";
    let host_name = "raf.local.";
    let port = 8080;

    // Identify local IP for mDNS
    let my_local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

    let service_info = ServiceInfo::new(
        service_type,
        instance_name,
        host_name,
        my_local_ip.to_string(),
        port,
        None,
    )
    .expect("Failed to create mDNS service info");

    mdns.register(service_info)
        .expect("Failed to register mDNS service");
    info!("mDNS: Registered service as raf.local");

    // 3. Initialize Database
    let db = Arc::new(raf::infra::db::Database::new("raf.db").expect("Failed to initialize database"));

    // 4. Generate Security Token
    use rand::distributions::{Alphanumeric, DistString};
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 12);
    
    // Load existing data
    let scans_map = DashMap::new();
    if let Ok(loaded_scans) = db.load_scans() {
        for (k, v) in loaded_scans {
            scans_map.insert(k, v);
        }
    }

    let orders_map = DashMap::new();
    if let Ok(loaded_orders) = db.load_orders() {
        for o in loaded_orders {
            orders_map.insert(o.id.clone(), o);
        }
    }

    let state = Arc::new(AppState {
        scans: scans_map,
        orders: orders_map,
        product_lookup: DashMap::new(),
        db: db.clone(),
        access_token: token.clone(),
        rate_limiter: DashMap::new(),
    });

    // 3. Identify Local IP
    let my_local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    // 5. Zero-Config HTTPS (Self-Signed)
    use axum_server::tls_rustls::RustlsConfig;
    use rcgen::generate_simple_self_signed;

    let cert_subject = vec![
        "jard.local".to_string(),
        "localhost".to_string(),
        my_local_ip.to_string(),
    ];
    let cert = generate_simple_self_signed(cert_subject)
        .map_err(|e| anyhow::anyhow!("Failed to generate cert: {}", e))?;
    let cert_der = cert.cert.der().to_vec();
    let key_der = cert.key_pair.serialize_der();

    let config = RustlsConfig::from_der(vec![cert_der], key_der)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create Rustls config: {}", e))?;

    info!("--------------------------------------------------");
    info!(" PC Dashboard:   https://localhost:8080?token={}", token);
    info!(
        " Mobile Picker:  https://{}:8080/scanner?token={}",
        my_local_ip, token
    );
    info!(
        " Zero-Conf:      https://raf.local:8080/scanner?token={}",
        token
    );
    info!("--------------------------------------------------");

    // 6. Auto-Open Browser
    let dashboard_url = format!("https://localhost:8080?token={}", token);
    if let Err(e) = open::that(dashboard_url) {
        tracing::warn!("Failed to open browser automatically: {}", e);
    }

    // 7. Run HTTPS Server with Graceful Shutdown
    let handle = axum_server::Handle::new();

    // Spawn a task to listen for shutdown signal
    let shutdown_handle = handle.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
    });

    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(router(state).into_make_service_with_connect_info::<SocketAddr>())
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

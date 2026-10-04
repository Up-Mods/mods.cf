use mods_cf::web;
use rootcause::prelude::*;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{Level, info};

const PORT: u16 = 3000;

#[dotenvy::load(required = false)]
#[tokio::main]
async fn main() -> rootcause::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let (app, shutdown) = web::init_router(true).await?;
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), PORT))
        .await
        .context_with(|| format!("Unable to create listener on port {PORT}"))?;

    info!("Listening on http://localhost:{PORT}");
    axum::serve(listener, app)
        .with_graceful_shutdown(wait_for_shutdown_signal())
        .await?;

    shutdown().await
}

async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        biased;
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating shutdown");
        }
    }
}

use anyhow::Context;
use mods_cf::web;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::signal;

const PORT: u16 = 3000;

#[dotenvy::load(required = false)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app = web::init_router(true).await?;
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), PORT))
        .await
        .with_context(|| format!("Unable to create listener on port {PORT}"))?;

    log::info!("Listening on http://localhost:{PORT}");
    axum::serve(listener, app)
        .with_graceful_shutdown(wait_for_shutdown_signal())
        .await?;

    // TODO handle shutdown

    Ok(())
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
            log::info!("Received Ctrl+C, initiating shutdown");
        }
        _ = terminate => {
            log::info!("Received SIGTERM, initiating shutdown");
        }
    }
}

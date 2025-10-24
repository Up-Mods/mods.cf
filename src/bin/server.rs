use anyhow::Context;
use mods_cf::web;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;

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
    axum::serve(listener, app).await?;
    Ok(())
}

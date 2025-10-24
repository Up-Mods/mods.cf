use anyhow::Context;
use mods_cf::web;
use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;

const PORT: u16 = 3000;

#[dotenvy::load(required = false)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let eternal_api_token = env::var("CURSEFORGE_ETERNAL_API_TOKEN")
        .expect("Please specify CURSEFORGE_ETERNAL_API_TOKEN for Curseforge Eternal API!");

    let app = web::init_router(eternal_api_token.as_str()).await?;
    let listener = TcpListener::bind(SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), PORT))
        .await
        .with_context(|| format!("Unable to create listener on port {PORT}"))?;

    log::info!("Listening on http://localhost:{PORT}");
    axum::serve(listener, app).await?;
    Ok(())
}

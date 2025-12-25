mod error;
mod routes;
mod types;

use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize a global tracing subscriber (logging).
    //
    // Behavior:
    // - Reads log level from RUST_LOG env var if set (e.g. RUST_LOG=info,billiard_api=debug).
    // - Falls back to "info" if RUST_LOG is not set.
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/simulate", post(routes::simulate));

    // Bind and serve
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

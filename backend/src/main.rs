use backend::app::{create_app, shutdown, shutdown_signal};
use backend::config;
use backend::telemetry;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::info;

#[tokio::main]
async fn main() {
    telemetry::init_tracing();

    let config = config::Config::from_env().expect("Failed to load config");

    let (app, state) = create_app(config);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080);
    info!(%addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    // Not `with_graceful_shutdown`: it waits for open WebSockets, which never close.
    tokio::select! {
        res = axum::serve(listener, app) => res.expect("Server error"),
        () = shutdown_signal() => {},
    }
    shutdown(&state).await;
}

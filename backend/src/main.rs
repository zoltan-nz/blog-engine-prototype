use backend::app::create_app;
use backend::config;
use backend::telemetry;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tracing::info;

#[tokio::main]
async fn main() {
    telemetry::init_tracing();

    let config = config::Config::from_env().expect("Failed to load config");

    let (app, _state) = create_app(config);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080);
    info!(%addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Server error");
}

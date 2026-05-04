use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace,axum=trace,backend=trace,tower_http=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .try_init();
}

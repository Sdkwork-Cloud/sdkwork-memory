use sdkwork_memory_api_server::{build_router, run_database_migrate_only};
use tokio::signal;

fn init_tracing() {
    let environment = std::env::var("SDKWORK_MEMORY_ENVIRONMENT").unwrap_or_else(|_| "development".to_owned());
    let use_json = environment.eq_ignore_ascii_case("production")
        || std::env::var("SDKWORK_MEMORY_LOG_FORMAT")
            .map(|value| value.eq_ignore_ascii_case("json"))
            .unwrap_or(false);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if use_json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .with_current_span(true)
            .with_span_list(true)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }
}

#[tokio::main]
async fn main() {
    init_tracing();

    if matches!(std::env::args().nth(1).as_deref(), Some("db-migrate")) {
        run_database_migrate_only()
            .await
            .expect("memory database migration failed");
        return;
    }

    let bind_address = std::env::var("SDKWORK_MEMORY_APPLICATION_PUBLIC_INGRESS_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let app = build_router()
        .await
        .expect("memory api-server bootstrap failed");
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("bind memory api-server listener failed");
    tracing::info!("sdkwork-memory-api-server listening on {bind_address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve memory api-server failed");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("sdkwork-memory-api-server shutdown signal received");
}

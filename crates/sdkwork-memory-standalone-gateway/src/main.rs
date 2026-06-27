use sdkwork_memory_standalone_gateway::{build_router, init_tracing, run_database_migrate_only};
use tokio::signal;
use tokio::time::Duration;

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
        .expect("memory standalone-gateway bootstrap failed");
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("bind memory standalone-gateway listener failed");
    tracing::info!("sdkwork-memory-standalone-gateway listening on {bind_address}");

    let worker_shutdown_tx = app.worker_shutdown_tx;
    axum::serve(listener, app.router)
        .with_graceful_shutdown(shutdown_signal(worker_shutdown_tx))
        .await
        .expect("serve memory standalone-gateway failed");
}

async fn shutdown_signal(worker_shutdown_tx: tokio::sync::watch::Sender<bool>) {
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

    tracing::info!("sdkwork-memory-standalone-gateway shutdown signal received");

    // Trigger graceful shutdown of all background workers.
    let _ = worker_shutdown_tx.send(true);

    // Give workers a bounded grace period to drain in-flight work.
    // Workers log their own shutdown confirmation; we only wait
    // a short time to avoid hanging on unresponsive workers.
    tokio::time::sleep(Duration::from_secs(3)).await;
    tracing::info!("sdkwork-memory-standalone-gateway background workers shutdown complete");
}

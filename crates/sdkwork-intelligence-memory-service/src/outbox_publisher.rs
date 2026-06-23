use std::sync::Arc;
use std::time::Duration;

use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;
use sdkwork_memory_spi::MemoryScopeContext;

pub fn spawn_outbox_publisher(store: Arc<NativeSqlMemoryStore>) {
    tokio::spawn(async move {
        let poll_interval = std::env::var("SDKWORK_MEMORY_OUTBOX_POLL_INTERVAL_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(2);
        loop {
            match store.list_global_pending_outbox_events(64).await {
                Ok(rows) => {
                    for row in rows {
                        let scope = MemoryScopeContext {
                            tenant_id: row.tenant_id,
                            space_id: 0,
                            organization_id: None,
                            user_id: None,
                        };
                        match store
                            .mark_outbox_published(&scope, &row.outbox.outbox_id)
                            .await
                        {
                            Ok(Some(_)) => tracing::info!(
                                tenant_id = row.tenant_id,
                                outbox_id = %row.outbox.outbox_id,
                                event_type = %row.outbox.event_type,
                                aggregate_id = %row.outbox.aggregate_id,
                                "memory domain outbox event published"
                            ),
                            Ok(None) => {}
                            Err(error) => tracing::warn!(
                                tenant_id = row.tenant_id,
                                outbox_id = %row.outbox.outbox_id,
                                error = %error,
                                "memory outbox publish failed"
                            ),
                        }
                    }
                }
                Err(error) => tracing::error!(error = %error, "memory outbox publisher poll failed"),
            }
            tokio::time::sleep(Duration::from_secs(poll_interval)).await;
        }
    });
}

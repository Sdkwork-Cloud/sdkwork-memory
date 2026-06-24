use std::sync::Arc;
use std::time::Duration;

use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

use crate::outbox_delivery::{deliver_outbox_event, OutboxDeliveryConfig};

pub fn spawn_outbox_publisher(store: Arc<NativeSqlMemoryStore>) {
    tokio::spawn(async move {
        let config = OutboxDeliveryConfig::from_env();
        let poll_interval = std::env::var("SDKWORK_MEMORY_OUTBOX_POLL_INTERVAL_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(2);
        let stale_processing_seconds =
            std::env::var("SDKWORK_MEMORY_OUTBOX_STALE_PROCESSING_SECS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(300);
        loop {
            if let Err(error) = store
                .requeue_stale_processing_outbox_events(stale_processing_seconds)
                .await
            {
                tracing::warn!(error = %error, "memory outbox stale processing requeue failed");
            }

            match store.claim_global_pending_outbox_events(64).await {
                Ok(rows) => {
                    for row in rows {
                        let outbox_id = row.outbox.outbox_id.clone();
                        let tenant_id = row.tenant_id;
                        match deliver_outbox_event(&row, &config).await {
                            Ok(()) => match store
                                .ack_outbox_delivery_success(tenant_id, &outbox_id)
                                .await
                            {
                                Ok(Some(_)) => {
                                    crate::domain_metrics::memory_domain_metrics()
                                        .record_outbox_published();
                                }
                                Ok(None) => {
                                    crate::domain_metrics::memory_domain_metrics()
                                        .record_outbox_delivery_failed();
                                    tracing::warn!(
                                        tenant_id,
                                        outbox_id = %outbox_id,
                                        "memory outbox ack skipped because row is no longer processing"
                                    );
                                }
                                Err(error) => {
                                    crate::domain_metrics::memory_domain_metrics()
                                        .record_outbox_delivery_failed();
                                    tracing::error!(
                                        tenant_id,
                                        outbox_id = %outbox_id,
                                        error = %error,
                                        "memory outbox publish ack failed"
                                    );
                                }
                            },
                            Err(error) => {
                                crate::domain_metrics::memory_domain_metrics()
                                    .record_outbox_delivery_failed();
                                match store
                                    .record_outbox_delivery_failure(
                                        tenant_id,
                                        &outbox_id,
                                        config.max_retries,
                                    )
                                    .await
                                {
                                    Ok(Some(updated)) => tracing::warn!(
                                        tenant_id,
                                        outbox_id = %outbox_id,
                                        publish_state = %updated.publish_state,
                                        retry_count = updated.retry_count,
                                        error = %error,
                                        "memory outbox delivery failed"
                                    ),
                                    Ok(None) => tracing::warn!(
                                        tenant_id,
                                        outbox_id = %outbox_id,
                                        error = %error,
                                        "memory outbox delivery failure could not be recorded"
                                    ),
                                    Err(store_error) => tracing::error!(
                                        tenant_id,
                                        outbox_id = %outbox_id,
                                        error = %store_error,
                                        delivery_error = %error,
                                        "memory outbox delivery failure persistence failed"
                                    ),
                                }
                            }
                        }
                    }
                }
                Err(error) => {
                    crate::domain_metrics::memory_domain_metrics().record_outbox_publish_failed();
                    tracing::error!(error = %error, "memory outbox publisher claim failed");
                }
            }
            tokio::time::sleep(Duration::from_secs(poll_interval)).await;
        }
    });
}

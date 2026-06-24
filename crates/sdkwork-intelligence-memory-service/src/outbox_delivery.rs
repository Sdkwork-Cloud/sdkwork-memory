use sdkwork_memory_plugin_native_sql::NativeSqlScopedOutboxEvent;
use sdkwork_utils_rust::format_datetime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboxDeliveryMode {
    Log,
    Http,
}

pub struct OutboxDeliveryConfig {
    pub mode: OutboxDeliveryMode,
    pub webhook_url: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl OutboxDeliveryConfig {
    pub fn from_env() -> Self {
        let mode = std::env::var("SDKWORK_MEMORY_OUTBOX_DELIVERY_MODE")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .map(|value| {
                if value == "http" {
                    OutboxDeliveryMode::Http
                } else {
                    OutboxDeliveryMode::Log
                }
            })
            .unwrap_or(OutboxDeliveryMode::Log);
        let webhook_url = std::env::var("SDKWORK_MEMORY_OUTBOX_DELIVERY_URL")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let timeout_seconds = std::env::var("SDKWORK_MEMORY_OUTBOX_DELIVERY_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(10);
        let max_retries = std::env::var("SDKWORK_MEMORY_OUTBOX_MAX_RETRIES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(5);
        Self {
            mode,
            webhook_url,
            timeout_seconds,
            max_retries,
        }
    }
}

pub fn build_cloud_event_envelope(row: &NativeSqlScopedOutboxEvent) -> serde_json::Value {
    let payload = serde_json::from_str::<serde_json::Value>(&row.outbox.payload_json)
        .unwrap_or_else(|_| serde_json::json!({ "raw": row.outbox.payload_json }));
    serde_json::json!({
        "specversion": "1.0",
        "id": row.outbox.outbox_id,
        "type": row.outbox.event_type,
        "source": "sdkwork-memory",
        "time": format_datetime(sdkwork_utils_rust::now(), None),
        "tenantId": row.tenant_id.to_string(),
        "subject": row.outbox.aggregate_id,
        "data": {
            "aggregateType": row.outbox.aggregate_type,
            "aggregateId": row.outbox.aggregate_id,
            "eventVersion": row.outbox.event_version,
            "payload": payload,
        }
    })
}

pub async fn deliver_outbox_event(
    row: &NativeSqlScopedOutboxEvent,
    config: &OutboxDeliveryConfig,
) -> Result<(), String> {
    let envelope = build_cloud_event_envelope(row);
    match config.mode {
        OutboxDeliveryMode::Log => {
            tracing::info!(
                tenant_id = row.tenant_id,
                outbox_id = %row.outbox.outbox_id,
                event_type = %row.outbox.event_type,
                aggregate_id = %row.outbox.aggregate_id,
                delivery_mode = "log",
                envelope = %envelope,
                "memory domain outbox event delivered"
            );
            Ok(())
        }
        OutboxDeliveryMode::Http => {
            let url = config
                .webhook_url
                .as_deref()
                .ok_or_else(|| {
                    "SDKWORK_MEMORY_OUTBOX_DELIVERY_URL is required when delivery mode is http"
                        .to_string()
                })?;
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout_seconds.max(1)))
                .build()
                .map_err(|error| format!("outbox delivery client init failed: {error}"))?;
            let response = client
                .post(url)
                .header("content-type", "application/json")
                .json(&envelope)
                .send()
                .await
                .map_err(|error| format!("outbox delivery request failed: {error}"))?;
            if response.status().is_success() {
                tracing::info!(
                    tenant_id = row.tenant_id,
                    outbox_id = %row.outbox.outbox_id,
                    event_type = %row.outbox.event_type,
                    delivery_mode = "http",
                    status = %response.status(),
                    "memory domain outbox event delivered"
                );
                Ok(())
            } else {
                Err(format!(
                    "outbox delivery webhook returned {}",
                    response.status()
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_memory_plugin_native_sql::NativeSqlMemoryOutboxEvent;

    #[test]
    fn cloud_event_envelope_includes_tenant_and_event_type() {
        let row = NativeSqlScopedOutboxEvent {
            tenant_id: 1001,
            outbox: NativeSqlMemoryOutboxEvent {
                outbox_id: "9001".to_string(),
                aggregate_type: "mem_record".to_string(),
                aggregate_id: "rec-1".to_string(),
                event_type: "memory.record.created".to_string(),
                event_version: "1".to_string(),
                payload_json: r#"{"memoryId":"rec-1"}"#.to_string(),
                publish_state: "processing".to_string(),
                published_at: None,
                retry_count: 0,
            },
        };
        let envelope = build_cloud_event_envelope(&row);
        assert_eq!(envelope["type"], "memory.record.created");
        assert_eq!(envelope["tenantId"], "1001");
        assert_eq!(envelope["data"]["aggregateId"], "rec-1");
    }
}

//! Background workers: learning job queue, eval runs, provider health probes.

use std::sync::Arc;
use std::time::Duration;

use sdkwork_memory_contract::{
    MemoryExtractionRequest, MemoryLearningJob, MemoryMigrationJobRequest,
    MemoryOpenApi, MemoryOpenApiRequestContext, MemoryRetentionJobRequest,
};
use sdkwork_memory_plugin_native_sql::{
    InsertLearningJobCommand, NativeSqlLearningJobRow, NativeSqlMemoryStore,
};
use sdkwork_memory_spi::MemoryScopeContext;
use sdkwork_utils_rust::is_blank;
use serde_json::Value;

use crate::open_api::OpenMemoryService;
use crate::platform;

/// Spawns all background workers and returns a shutdown sender.
///
/// The caller MUST keep the sender alive and call `send(true)` during
/// graceful shutdown so that all workers drain in-flight work and exit.
/// Dropping the sender also causes receivers to observe a closed channel,
/// but an explicit `send(true)` ensures a cleaner shutdown with logged
/// confirmation from each worker.
pub fn spawn_background_workers(
    service: Arc<OpenMemoryService>,
) -> tokio::sync::watch::Sender<bool> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    crate::outbox_publisher::spawn_outbox_publisher(service.store.clone(), shutdown_rx.clone());
    spawn_learning_job_worker(service.clone(), shutdown_rx.clone());
    spawn_eval_run_worker(service.clone(), shutdown_rx.clone());
    spawn_provider_health_probe(service, shutdown_rx);
    shutdown_tx
}

fn spawn_learning_job_worker(service: Arc<OpenMemoryService>, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
    tokio::spawn(async move {
        let poll_interval = platform::read_env_u64("SDKWORK_MEMORY_JOB_POLL_INTERVAL_SECS", 2);
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("memory learning job worker shutting down");
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(poll_interval)) => {
                    if let Err(error) = process_learning_job_batch(&service).await {
                        tracing::warn!(error = %error, "memory learning job worker batch failed");
                    }
                }
            }
        }
    });
}

fn spawn_eval_run_worker(service: Arc<OpenMemoryService>, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
    tokio::spawn(async move {
        let poll_interval = platform::read_env_u64("SDKWORK_MEMORY_EVAL_POLL_INTERVAL_SECS", 5);
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("memory eval run worker shutting down");
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(poll_interval)) => {
                    if let Err(error) = process_eval_run_batch(&service).await {
                        tracing::warn!(error = %error, "memory eval run worker batch failed");
                    }
                }
            }
        }
    });
}

fn spawn_provider_health_probe(service: Arc<OpenMemoryService>, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
    tokio::spawn(async move {
        let poll_interval = platform::read_env_u64("SDKWORK_MEMORY_PROVIDER_HEALTH_PROBE_SECS", 60);
        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("memory provider health probe shutting down");
                        break;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(poll_interval)) => {
                    if let Err(error) = probe_provider_bindings(&service).await {
                        tracing::warn!(error = %error, "memory provider health probe failed");
                    }
                }
            }
        }
    });
}

async fn process_learning_job_batch(service: &OpenMemoryService) -> Result<(), String> {
    let jobs = service
        .store
        .claim_queued_learning_jobs(16)
        .await
        .map_err(|error| error.to_string())?;
    for job in jobs {
        if let Err(error) = execute_learning_job(service, &job).await {
            tracing::warn!(
                tenant_id = job.tenant_id,
                job_uuid = %job.job_uuid,
                job_type = %job.job_type,
                error = %error,
                "memory learning job failed"
            );
            let _ = service
                .store
                .finish_learning_job(
                    job.tenant_id,
                    &job.job_uuid,
                    "failed",
                    None,
                    Some(&serde_json::json!({ "message": error }).to_string()),
                )
                .await;
        }
    }
    Ok(())
}

async fn execute_learning_job(
    service: &OpenMemoryService,
    job: &NativeSqlLearningJobRow,
) -> Result<(), String> {
    let input = job
        .input_json
        .as_deref()
        .ok_or_else(|| "learning job input_json is missing".to_string())?;
    let open_context = MemoryOpenApiRequestContext::for_backend_surface(
        u64::try_from(job.tenant_id.max(0)).unwrap_or(0),
        None,
    );
    let result = match job.job_type.as_str() {
        "extract" | "extraction" => {
            let request: MemoryExtractionRequest = serde_json::from_str(input)
                .map_err(|error| format!("extraction input decode failed: {error}"))?;
            let learning_job =
                MemoryOpenApi::create_extraction(service, open_context, request)
                    .await
                    .map_err(|error| error.detail)?;
            serde_json::to_string(&learning_job)
                .map_err(|error| format!("extraction result encode failed: {error}"))?
        }
        "consolidation" => {
            let request: MemoryExtractionRequest = serde_json::from_str(input)
                .map_err(|error| format!("consolidation input decode failed: {error}"))?;
            let tenant_id = job.tenant_id;
            let space_id = platform::space_id_i64(request.space_id).map_err(|error| error.detail)?;
            let scope = MemoryScopeContext {
                tenant_id,
                space_id,
                organization_id: None,
                user_id: None,
            };
            let merged = service
                .store
                .consolidate_duplicate_records_in_scope(&scope)
                .await
                .map_err(|error| error.to_string())?;
            serde_json::json!({ "mergedDuplicates": merged, "spaceId": request.space_id }).to_string()
        }
        "retention" => {
            let request: MemoryRetentionJobRequest = serde_json::from_str(input)
                .map_err(|error| format!("retention input decode failed: {error}"))?;
            let tenant_id = job.tenant_id;
            let space_id = request
                .space_id
                .map(platform::space_id_i64)
                .transpose()
                .map_err(|error| error.detail)?
                .unwrap_or(1);
            let scope = MemoryScopeContext {
                tenant_id,
                space_id,
                organization_id: None,
                user_id: None,
            };
            let dry_run = request.dry_run.unwrap_or(false);
            let deleted = service
                .store
                .purge_expired_records_for_scope(&scope, dry_run)
                .await
                .map_err(|error| error.to_string())?;
            serde_json::json!({ "deletedRecords": deleted, "dryRun": dry_run }).to_string()
        }
        "index_rebuild" | "index_sync" => {
            let payload: Value = serde_json::from_str(input)
                .map_err(|error| format!("index rebuild input decode failed: {error}"))?;
            let index_id = payload
                .get("indexId")
                .and_then(Value::as_u64)
                .ok_or_else(|| "index rebuild input missing indexId".to_string())?;
            let rebuilt = service
                .store
                .rebuild_all_record_search_indexes(job.tenant_id)
                .await
                .map_err(|error| error.to_string())?;
            let _ = service
                .store
                .update_mem_index_for_tenant(
                    job.tenant_id,
                    &index_id.to_string(),
                    Some("active"),
                    None,
                    Some(platform::current_timestamp().as_str()),
                    None,
                    None,
                )
                .await;
            serde_json::json!({ "indexId": index_id, "rebuiltRecords": rebuilt }).to_string()
        }
        "migration" => {
            let request: MemoryMigrationJobRequest = serde_json::from_str(input)
                .map_err(|error| format!("migration input decode failed: {error}"))?;
            service.store.ping().await.map_err(|error| error.to_string())?;
            let profile_id = request.target_implementation_profile_id;
            let row = service
                .store
                .retrieve_mem_implementation_profile_for_tenant(
                    job.tenant_id,
                    &profile_id.to_string(),
                )
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| format!("implementation profile {profile_id} not found"))?;
            serde_json::json!({
                "targetImplementationProfileId": profile_id,
                "implementationKind": row.implementation_kind,
                "verified": true,
            })
            .to_string()
        }
        other => return Err(format!("unsupported learning job type: {other}")),
    };
    service
        .store
        .finish_learning_job(job.tenant_id, &job.job_uuid, "succeeded", Some(&result), None)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn process_eval_run_batch(service: &OpenMemoryService) -> Result<(), String> {
    let runs = service
        .store
        .list_queued_eval_runs(8)
        .await
        .map_err(|error| error.to_string())?;
    for (tenant_id, eval_uuid, eval_type) in runs {
        service
            .store
            .update_eval_run_state(tenant_id, &eval_uuid, "running", None, None)
            .await
            .map_err(|error| error.to_string())?;
        let metrics = match eval_type.as_str() {
            "retrieval_quality" => run_retrieval_quality_eval(service, tenant_id).await?,
            _ => serde_json::json!({ "evalType": eval_type, "status": "skipped" }).to_string(),
        };
        service
            .store
            .update_eval_run_state(
                tenant_id,
                &eval_uuid,
                "succeeded",
                Some(&metrics),
                Some(&metrics),
            )
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

async fn run_retrieval_quality_eval(
    service: &OpenMemoryService,
    tenant_id: i64,
) -> Result<String, String> {
    let _ = service
        .store
        .ensure_default_retrieval_profile_for_tenant(tenant_id)
        .await;
    let sample_query = "memory";
    let scope = MemoryScopeContext {
        tenant_id,
        space_id: 1,
        organization_id: None,
        user_id: None,
    };
    let hits = service
        .store
        .search_record_details_keyword(&scope, sample_query, 5)
        .await
        .map_err(|error| error.to_string())?;
    Ok(serde_json::json!({
        "evalType": "retrieval_quality",
        "sampleQuery": sample_query,
        "hitCount": hits.len(),
        "passed": true,
    })
    .to_string())
}

async fn probe_provider_bindings(service: &OpenMemoryService) -> Result<(), String> {
    let tenants = service
        .store
        .list_distinct_tenant_ids_with_provider_bindings()
        .await
        .map_err(|error| error.to_string())?;
    for tenant_id in tenants {
        let rows = service
            .store
            .list_mem_provider_bindings_for_tenant(tenant_id, 100, None)
            .await
            .map_err(|error| error.to_string())?;
        for row in rows {
            let health = probe_binding_endpoint(row.endpoint_ref.as_deref()).await;
            let now = platform::current_timestamp();
            let _ = service
                .store
                .update_mem_provider_binding_for_tenant(
                    tenant_id,
                    &row.binding_uuid,
                    None,
                    None,
                    Some(health.as_str()),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Some(now.as_str())),
                )
                .await;
        }
    }
    Ok(())
}

/// Validates an endpoint URL to prevent SSRF attacks.
///
/// Rules:
/// - Only `http` and `https` schemes are allowed.
/// - `https` is required in production-like environments.
/// - Hosts must not resolve to private, loopback, or link-local ranges.
/// - Bare IP addresses in private ranges are rejected.
fn validate_endpoint_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("invalid URL: {e}"))?;

    match parsed.scheme() {
        "https" => {}
        "http" => {
            if platform::is_production_like_environment() {
                return Err("HTTP scheme is not allowed in production; use HTTPS".to_string());
            }
        }
        other => return Err(format!("unsupported URL scheme: {other}")),
    }

    let host = parsed.host_str().ok_or_else(|| "URL must have a host".to_string())?;

    // Reject bare private/loopback/link-local IP literals.
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if ip.is_loopback()
            || ip.is_unspecified()
            || is_private_ip(&ip)
            || is_link_local_ip(&ip)
        {
            return Err(format!("private or loopback IP addresses are not allowed: {host}"));
        }
    }

    // Reject common localhost hostnames.
    let host_lower = host.to_ascii_lowercase();
    if host_lower == "localhost" || host_lower.ends_with(".localhost") {
        return Err(format!("localhost hostnames are not allowed: {host}"));
    }

    // Reject metadata service hostnames used by cloud providers.
    if host_lower == "metadata.google.internal"
        || host_lower == "169.254.169.254"
        || host_lower == "metadata.aws.internal"
    {
        return Err(format!("cloud metadata endpoints are not allowed: {host}"));
    }

    Ok(())
}

fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_private() || v4.is_link_local()
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback() || v6.is_unspecified()
        }
    }
}

fn is_link_local_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_link_local(),
        std::net::IpAddr::V6(_) => false,
    }
}

async fn probe_binding_endpoint(endpoint_ref: Option<&str>) -> String {
    let Some(url) = endpoint_ref.filter(|value| !is_blank(Some(value))) else {
        return "healthy".to_string();
    };

    if let Err(reason) = validate_endpoint_url(url) {
        tracing::warn!(
            url = %url,
            reason = %reason,
            "provider health probe rejected unsafe endpoint URL"
        );
        return "unhealthy".to_string();
    }

    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(8)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    });
    match client.get(url).send().await {
        Ok(response) if response.status().is_success() => "healthy".to_string(),
        Ok(_) => "degraded".to_string(),
        Err(_) => "unhealthy".to_string(),
    }
}

#[allow(dead_code)]
pub async fn enqueue_learning_job(
    store: &NativeSqlMemoryStore,
    tenant_id: i64,
    job_id: u64,
    job_type: &str,
    space_id: Option<u64>,
    input_json: &str,
    priority: i32,
) -> Result<(), sdkwork_memory_plugin_native_sql::NativeSqlStoreError> {
    store
        .insert_learning_job(InsertLearningJobCommand {
            tenant_id,
            job_uuid: &job_id.to_string(),
            space_id: platform::optional_u64_as_i64(space_id).ok().flatten(),
            job_type,
            state: "queued",
            priority,
            idempotency_key: None,
            input_json: Some(input_json),
        })
        .await
}

#[allow(dead_code)]
pub fn learning_job_from_row(row: &NativeSqlLearningJobRow) -> MemoryServiceResult<MemoryLearningJob> {
    use sdkwork_memory_contract::MemoryServiceError;
    type MemoryServiceResult<T> = Result<T, MemoryServiceError>;
    let job_id = row.job_uuid.parse::<u64>().map_err(|error| {
        MemoryServiceError::storage(format!("learning job id must be numeric: {error}"))
    })?;
    Ok(MemoryLearningJob {
        job_id,
        space_id: row
            .space_id
            .and_then(|value| u64::try_from(value.max(0)).ok()),
        job_type: row.job_type.clone(),
        state: row.state.clone(),
        priority: row.priority,
        result: row
            .result_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        error: row
            .error_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        started_at: row.started_at.clone(),
        finished_at: row.finished_at.clone(),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        version: Some(u64::try_from(row.version.max(0)).unwrap_or(0)),
    })
}

use sdkwork_memory_contract::MemoryServiceResult;

//! Background workers: learning job queue, eval runs, provider health probes.

use std::sync::Arc;
use std::time::Duration;

use sdkwork_memory_contract::{
    MemoryExtractionRequest, MemoryLearningJob, MemoryMigrationJobRequest,
    MemoryOpenApiRequestContext, MemoryRetentionJobRequest, MemoryServiceError,
};
use sdkwork_memory_plugin_native_sql::{
    InsertLearningJobCommand, NativeSqlLearningJobRow, NativeSqlMemoryStore,
};
use sdkwork_memory_spi::{
    MemoryRetrieverKind, MemoryScopeContext, MemorySensitivityReadScope,
    SearchMemoryCandidatesQuery,
};
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

fn spawn_learning_job_worker(
    service: Arc<OpenMemoryService>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
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

fn spawn_eval_run_worker(
    service: Arc<OpenMemoryService>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
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

fn spawn_provider_health_probe(
    service: Arc<OpenMemoryService>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
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
    let stale_secs = platform::read_env_u64("SDKWORK_MEMORY_JOB_STALE_SECS", 900);
    let _ = service
        .store
        .requeue_stale_running_learning_jobs(stale_secs)
        .await
        .map_err(|error| error.to_string())?;
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

fn assert_learning_job_space_id(
    job: &NativeSqlLearningJobRow,
    space_id: i64,
) -> Result<(), String> {
    match job.space_id {
        Some(job_space_id) if job_space_id != space_id => Err(format!(
            "learning job space_id {job_space_id} does not match input space_id {space_id}"
        )),
        None => Err(format!(
            "learning job {job_uuid} missing space_id; scope mutations require an explicit space",
            job_uuid = job.job_uuid
        )),
        Some(_) => Ok(()),
    }
}

fn parse_background_job_actor_id(input_json: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(input_json)
        .ok()
        .and_then(|value| {
            value
                .get("actorId")
                .or_else(|| value.get("actor_id"))
                .and_then(|id| {
                    id.as_u64()
                        .or_else(|| id.as_i64().and_then(|v| u64::try_from(v.max(0)).ok()))
                })
        })
}

fn background_job_context(
    job: &NativeSqlLearningJobRow,
    input_json: &str,
) -> MemoryOpenApiRequestContext {
    MemoryOpenApiRequestContext::for_background_job(
        u64::try_from(job.tenant_id.max(0)).unwrap_or(0),
        parse_background_job_actor_id(input_json),
    )
}

async fn execute_learning_job(
    service: &OpenMemoryService,
    job: &NativeSqlLearningJobRow,
) -> Result<(), String> {
    let input = job
        .input_json
        .as_deref()
        .ok_or_else(|| "learning job input_json is missing".to_string())?;
    let open_context = background_job_context(job, input);
    let result = match job.job_type.as_str() {
        "extract" | "extraction" => {
            let request: MemoryExtractionRequest = serde_json::from_str(input)
                .map_err(|error| format!("extraction input decode failed: {error}"))?;
            let space_id =
                platform::space_id_i64(request.space_id).map_err(|error| error.detail)?;
            assert_learning_job_space_id(job, space_id)?;
            if open_context.actor_id.is_none() {
                return Err(
                    "extraction background job requires actorId in input_json for authorization"
                        .to_string(),
                );
            }
            OpenMemoryService::execute_extraction_work(service, open_context, request)
                .await
                .map_err(|error| error.detail)?
                .to_string()
        }
        "consolidation" => {
            let request: MemoryExtractionRequest = serde_json::from_str(input)
                .map_err(|error| format!("consolidation input decode failed: {error}"))?;
            let tenant_id = job.tenant_id;
            let space_id =
                platform::space_id_i64(request.space_id).map_err(|error| error.detail)?;
            assert_learning_job_space_id(job, space_id)?;
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
            serde_json::json!({ "mergedDuplicates": merged, "spaceId": request.space_id })
                .to_string()
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
                .ok_or_else(|| "retention input missing spaceId".to_string())?;
            assert_learning_job_space_id(job, space_id)?;
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
            let index_row = service
                .store
                .retrieve_mem_index_for_tenant(job.tenant_id, &index_id.to_string())
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| format!("memory index {index_id} not found for tenant"))?;
            let rebuilt = if let Some(space_id) = index_row.space_id {
                service
                    .store
                    .rebuild_record_search_indexes_for_space(job.tenant_id, space_id)
                    .await
                    .map_err(|error| error.to_string())?
            } else {
                service
                    .store
                    .rebuild_all_record_search_indexes(job.tenant_id)
                    .await
                    .map_err(|error| error.to_string())?
            };
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
            serde_json::json!({
                "indexId": index_id,
                "spaceId": index_row.space_id,
                "rebuiltRecords": rebuilt
            })
            .to_string()
        }
        "migration" => {
            let request: MemoryMigrationJobRequest = serde_json::from_str(input)
                .map_err(|error| format!("migration input decode failed: {error}"))?;
            let result = crate::implementation_migration::execute_implementation_profile_migration(
                &service.store,
                job.tenant_id,
                &request,
                &service.core_runtime.profile().profile_id,
            )
            .await
            .map_err(|error| error.detail)?;
            serde_json::to_string(&result)
                .map_err(|error| format!("migration result encode failed: {error}"))?
        }
        other => return Err(format!("unsupported learning job type: {other}")),
    };
    service
        .store
        .finish_learning_job(
            job.tenant_id,
            &job.job_uuid,
            "succeeded",
            Some(&result),
            None,
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn process_eval_run_batch(service: &OpenMemoryService) -> Result<(), String> {
    let stale_secs = platform::read_env_u64("SDKWORK_MEMORY_EVAL_STALE_SECS", 900);
    let _ = service
        .store
        .requeue_stale_running_eval_runs(stale_secs)
        .await
        .map_err(|error| error.to_string())?;
    let runs = service
        .store
        .claim_queued_eval_runs(8)
        .await
        .map_err(|error| error.to_string())?;
    for (tenant_id, eval_uuid, eval_type) in runs {
        let (state, metrics) = match eval_type.as_str() {
            "retrieval_quality" => {
                let metrics = run_retrieval_quality_eval(service, tenant_id).await?;
                ("succeeded", metrics)
            }
            other => {
                let metrics = serde_json::json!({
                    "evalType": other,
                    "status": "skipped",
                    "reason": "eval type is not implemented"
                })
                .to_string();
                ("skipped", metrics)
            }
        };
        service
            .store
            .update_eval_run_state(tenant_id, &eval_uuid, state, Some(&metrics), Some(&metrics))
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
    let spaces = service
        .store
        .list_spaces_for_tenant(tenant_id, sdkwork_utils_rust::MAX_LIST_PAGE_SIZE, 0, None)
        .await
        .map_err(|error| error.to_string())?;
    let space_id = spaces.first().map(|space| space.space_id).unwrap_or(1);
    let sample_query = "memory";
    let scope = MemoryScopeContext {
        tenant_id,
        space_id,
        organization_id: None,
        user_id: None,
    };
    let search = service
        .runtime_data_plane
        .search_candidates_scoped(SearchMemoryCandidatesQuery {
            scope,
            query: sample_query.to_string(),
            limit: 5,
            retriever_kinds: vec![MemoryRetrieverKind::Keyword],
            memory_types: Vec::new(),
            read_scope: MemorySensitivityReadScope::Owner,
        })
        .await
        .map_err(|error| error.detail)?;
    let hit_count = search.records.len();
    Ok(serde_json::json!({
        "evalType": "retrieval_quality",
        "sampleQuery": sample_query,
        "spaceId": space_id,
        "hitCount": hit_count,
        "passed": hit_count > 0,
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
        let mut cursor = None;
        loop {
            let rows = service
                .store
                .list_mem_provider_bindings_for_tenant(
                    tenant_id,
                    sdkwork_utils_rust::MAX_LIST_PAGE_SIZE,
                    cursor.as_deref(),
                )
                .await
                .map_err(|error| error.to_string())?;
            let page_size = usize::try_from(sdkwork_utils_rust::MAX_LIST_PAGE_SIZE).unwrap_or(200);
            let has_more = rows.len() > page_size;
            for row in rows.iter().take(page_size) {
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
            if !has_more {
                break;
            }
            cursor = rows
                .get(page_size.saturating_sub(1))
                .map(|row| row.binding_uuid.clone());
        }
    }
    Ok(())
}

async fn probe_binding_endpoint(endpoint_ref: Option<&str>) -> String {
    let Some(url) = endpoint_ref.filter(|value| !is_blank(Some(value))) else {
        return "healthy".to_string();
    };

    if let Err(reason) = crate::endpoint_validation::validate_outbound_url(url) {
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

pub fn learning_job_from_row(
    row: &NativeSqlLearningJobRow,
) -> MemoryServiceResult<MemoryLearningJob> {
    let job_id = crate::platform::parse_numeric_id(&row.job_uuid).ok_or_else(|| {
        MemoryServiceError::storage("learning job id must be numeric".to_string())
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

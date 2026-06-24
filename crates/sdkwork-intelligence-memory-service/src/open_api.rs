use std::sync::Arc;

use async_trait::async_trait;
use sdkwork_memory_contract::{
    ListCandidatesQuery, ListMemoriesQuery, MemoryAppRequestContext, MemoryBackendRequestContext,
    MemoryCandidate, MemoryCandidateList, MemoryCapabilities, MemoryContextPack,
    MemoryContextPackRequest, MemoryEvent, MemoryEventRequest, MemoryExtractionRequest,
    MemoryFeedback, MemoryFeedbackRequest, MemoryImplementationKind, MemoryLearningJob,
    MemoryOpenApi, MemoryOpenApiRequestContext, MemoryPageInfo, MemoryProviderHealth,
    MemoryProviderHealthStatus, MemoryProviderInterface, MemoryRecord, MemoryRecordList,
    MemoryRecordPatch, MemoryRecordRequest, MemoryRetrievalHit, MemoryRetrievalRequest,
    MemoryRetrievalResult, MemoryRetrievalTrace, MemoryRetrieverKind, MemoryServiceError,
    MemoryServiceResult, MemoryType,
};
use sdkwork_memory_core::{
    build_context_pack_from_hits, fuse_retrieval_candidates, orchestrate_retrieval_candidates,
    RetrievalCandidate, RetrievalEventInput, RetrievalRecordInput,
};
use sdkwork_memory_plugin_native_sql::{
    NativeSqlAppendOutboxEventCommand, NativeSqlMemoryRecordDetail, NativeSqlMemoryStore,
    NativeSqlOpenApiEventRow,
};
use sdkwork_memory_spi::{
    AppendMemoryRetrievalTraceCommand, CreateMemoryCandidateCommand, MemoryRetrievalHitDraft,
    MemoryScopeContext,
};

use tracing::info;

use crate::access;
use crate::platform;
use crate::store_error::map_native_sql_store_error;

pub struct OpenMemoryService {
    pub(crate) store: Arc<NativeSqlMemoryStore>,
    pub(crate) profile_id: String,
    pub(crate) primary_plugin_id: String,
}

impl OpenMemoryService {
    pub fn new(store: NativeSqlMemoryStore) -> Self {
        Self::with_runtime_profile(
            store,
            "native-sql-phase1",
            sdkwork_memory_plugin_native_sql::NATIVE_SQL_PLUGIN_ID,
        )
    }

    pub fn with_runtime_profile(
        store: NativeSqlMemoryStore,
        profile_id: impl Into<String>,
        primary_plugin_id: impl Into<String>,
    ) -> Self {
        Self {
            store: Arc::new(store),
            profile_id: profile_id.into(),
            primary_plugin_id: primary_plugin_id.into(),
        }
    }

    pub fn from_phase1_runtime(
        phase1: sdkwork_memory_plugin_native_sql::NativeSqlPhase1Runtime,
        profile_id: impl Into<String>,
        primary_plugin_id: impl Into<String>,
    ) -> Self {
        Self {
            store: phase1.into_arc_store(),
            profile_id: profile_id.into(),
            primary_plugin_id: primary_plugin_id.into(),
        }
    }

    pub async fn ready_check(&self) -> MemoryServiceResult<()> {
        self.store
            .ping()
            .await
            .map_err(Self::map_store_error)?;
        tracing::debug!(
            profile_id = %self.profile_id,
            primary_plugin_id = %self.primary_plugin_id,
            "memory store ready"
        );
        Ok(())
    }

    pub fn runtime_profile_label(&self) -> &'static str {
        match self.store.dialect() {
            sdkwork_memory_plugin_native_sql::MemorySqlDialect::Postgres => "postgresql",
            sdkwork_memory_plugin_native_sql::MemorySqlDialect::Sqlite => "sqlite",
        }
    }

    pub fn spawn_background_workers(service: &Arc<Self>) {
        crate::outbox_publisher::spawn_outbox_publisher(service.store.clone());
    }

    pub(crate) fn to_open_context(app: &MemoryAppRequestContext) -> MemoryOpenApiRequestContext {
        MemoryOpenApiRequestContext {
            api_key_id: app
                .session_id
                .clone()
                .unwrap_or_else(|| format!("app-{}", app.actor_id.unwrap_or(0))),
            tenant_id: app.tenant_id,
            actor_id: app.actor_id,
            elevated_tenant_access: false,
        }
    }

    pub(crate) fn to_open_context_backend(
        backend: &MemoryBackendRequestContext,
    ) -> MemoryOpenApiRequestContext {
        MemoryOpenApiRequestContext::for_backend_surface(backend.tenant_id, backend.operator_id)
    }

    pub(crate) fn next_id(&self) -> MemoryServiceResult<u64> {
        platform::next_numeric_id()
    }

    pub(crate) fn scope(
        context: &MemoryOpenApiRequestContext,
        space_id: u64,
    ) -> MemoryServiceResult<MemoryScopeContext> {
        Ok(MemoryScopeContext {
            tenant_id: platform::tenant_id_i64(context.tenant_id)?,
            space_id: platform::space_id_i64(space_id)?,
            organization_id: None,
            user_id: context.actor_id.map(|value| value as i64),
        })
    }

    pub(crate) fn map_store_error(
        error: sdkwork_memory_plugin_native_sql::NativeSqlStoreError,
    ) -> MemoryServiceError {
        map_native_sql_store_error(error)
    }

    pub(crate) async fn publish_domain_event(
        &self,
        scope: &MemoryScopeContext,
        event_type: &str,
        aggregate_type: &str,
        aggregate_id: &str,
        payload: serde_json::Value,
    ) -> MemoryServiceResult<()> {
        let outbox_id = self.next_id()?.to_string();
        let payload_json = serde_json::to_string(&payload).map_err(|error| {
            MemoryServiceError::storage_internal(format!("domain event payload encode failed: {error}"))
        })?;
        self.store
            .append_outbox_event(NativeSqlAppendOutboxEventCommand {
                scope,
                outbox_id: &outbox_id,
                aggregate_type,
                aggregate_id,
                event_type,
                event_version: "1.0",
                payload_json: &payload_json,
            })
            .await
            .map_err(Self::map_store_error)?;
        Ok(())
    }

    async fn load_scoped_record(
        &self,
        context: &MemoryOpenApiRequestContext,
        space_id: u64,
        memory_id: u64,
    ) -> MemoryServiceResult<MemoryRecord> {
        access::assert_actor_can_access_space(&self.store, context, space_id).await?;
        let scope = Self::scope(context, space_id)?;
        match self
            .store
            .retrieve_record_detail(&scope, &memory_id.to_string())
            .await
            .map_err(Self::map_store_error)?
        {
            Some(row) => Self::map_record(row),
            None => Err(MemoryServiceError::not_found("memory not found")),
        }
    }

    async fn load_scoped_event(
        &self,
        context: &MemoryOpenApiRequestContext,
        space_id: u64,
        event_id: u64,
    ) -> MemoryServiceResult<MemoryEvent> {
        access::assert_actor_can_access_space(&self.store, context, space_id).await?;
        let scope = Self::scope(context, space_id)?;
        match self
            .store
            .retrieve_open_api_event(&scope, &event_id.to_string())
            .await
            .map_err(Self::map_store_error)?
        {
            Some(row) => Self::map_event(row),
            None => Err(MemoryServiceError::not_found("event not found")),
        }
    }

    pub(crate) fn parse_id(value: &str) -> Option<u64> {
        platform::parse_numeric_id(value)
    }

    pub(crate) fn memory_type_to_db(value: MemoryType) -> &'static str {
        match value {
            MemoryType::Working => "working",
            MemoryType::Session => "session",
            MemoryType::Semantic => "semantic",
            MemoryType::Episodic => "episodic",
            MemoryType::Procedural => "procedural",
            MemoryType::Habit => "habit",
            MemoryType::Relationship => "relationship",
            MemoryType::DomainKnowledge => "domain_knowledge",
        }
    }

    pub(crate) fn memory_type_from_db(value: &str) -> MemoryType {
        match value {
            "working" => MemoryType::Working,
            "session" => MemoryType::Session,
            "episodic" => MemoryType::Episodic,
            "procedural" => MemoryType::Procedural,
            "habit" => MemoryType::Habit,
            "relationship" => MemoryType::Relationship,
            "domain_knowledge" => MemoryType::DomainKnowledge,
            _ => MemoryType::Semantic,
        }
    }

    pub(crate) fn map_record(detail: NativeSqlMemoryRecordDetail) -> MemoryServiceResult<MemoryRecord> {
        let memory_id = Self::parse_id(&detail.memory_id)
            .ok_or_else(|| MemoryServiceError::storage("memory id must be numeric"))?;
        let space_id = u64::try_from(detail.space_id)
            .map_err(|_| MemoryServiceError::storage("space id must be non-negative"))?;
        let version = u64::try_from(detail.version.max(0))
            .map_err(|_| MemoryServiceError::storage("version must be non-negative"))?;
        let supersedes_memory_id = detail
            .supersedes_memory_id
            .as_deref()
            .and_then(Self::parse_id);
        let superseded_by_memory_id = detail
            .superseded_by_memory_id
            .as_deref()
            .and_then(Self::parse_id);

        Ok(MemoryRecord {
            memory_id,
            uuid: Some(detail.memory_id),
            space_id,
            user_id: None,
            scope: detail.scope,
            memory_type: Self::memory_type_from_db(&detail.memory_type),
            subject: detail.subject,
            predicate: detail.predicate,
            object_text: Some(detail.object_text),
            canonical_text: detail.canonical_text,
            summary_text: None,
            confidence: detail.confidence,
            evidence_count: Some(1),
            contradiction_count: Some(0),
            status: detail.status,
            sensitivity_level: detail.sensitivity_level,
            supersedes_memory_id,
            superseded_by_memory_id,
            created_at: detail.created_at,
            updated_at: detail.updated_at,
            version,
        })
    }

    pub(crate) fn map_event(row: NativeSqlOpenApiEventRow) -> MemoryServiceResult<MemoryEvent> {
        let event_id = Self::parse_id(&row.event_id)
            .ok_or_else(|| MemoryServiceError::storage("event id must be numeric"))?;
        let space_id = u64::try_from(row.space_id)
            .map_err(|_| MemoryServiceError::storage("space id must be non-negative"))?;

        Ok(MemoryEvent {
            event_id,
            uuid: Some(row.event_id),
            space_id,
            user_id: None,
            actor_type: None,
            actor_id: None,
            event_type: row.event_type,
            source_type: row.source_type,
            event_time: row.event_time,
            payload: Some(row.payload),
            payload_hash: row.payload_hash,
            sensitivity_level: None,
            ingestion_status: row.ingestion_status,
            created_at: row.created_at,
        })
    }

    pub(crate) fn normalize_sensitivity_level(value: Option<&str>) -> MemoryServiceResult<&'static str> {
        match value.unwrap_or("internal") {
            "public" => Ok("public"),
            "internal" => Ok("internal"),
            "private" => Ok("private"),
            "sensitive" => Ok("sensitive"),
            "restricted" => Ok("restricted"),
            other => Err(MemoryServiceError::validation(format!(
                "sensitivityLevel must be one of public, internal, private, sensitive, restricted; got {other}"
            ))),
        }
    }

    fn default_retriever_profile() -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "keyword": { "weight": 1.0 },
            "dictionary": { "weight": 0.85 },
            "time": { "weight": 0.5 },
            "event": { "weight": 0.6 },
            "sql": { "weight": 0.75 }
        }))
    }
}

#[async_trait]
impl MemoryOpenApi for OpenMemoryService {
    async fn retrieve_capabilities(
        &self,
        _context: MemoryOpenApiRequestContext,
    ) -> MemoryServiceResult<MemoryCapabilities> {
        Ok(MemoryCapabilities {
            embedding_optional: true,
            retrievers: vec![
                MemoryRetrieverKind::Keyword,
                MemoryRetrieverKind::Dictionary,
                MemoryRetrieverKind::Time,
                MemoryRetrieverKind::Event,
                MemoryRetrieverKind::Sql,
            ],
            provider_interfaces: vec![
                MemoryProviderInterface::Memory,
                MemoryProviderInterface::Search,
            ],
            implementation_kinds: vec![MemoryImplementationKind::NativeSql],
            open_api_prefix: "/mem/v3/api".to_string(),
            sdk_family: "sdkwork-memory-sdk".to_string(),
            checked_at: platform::current_timestamp(),
            metadata: None,
        })
    }

    async fn create_event(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryEventRequest,
    ) -> MemoryServiceResult<MemoryEvent> {
        access::assert_actor_can_access_space_for_write(&self.store, &context, request.space_id).await?;
        let scope = Self::scope(&context, request.space_id)?;
        let event_id = self.next_id()?.to_string();
        let sensitivity =
            Self::normalize_sensitivity_level(request.sensitivity_level.as_deref())?;
        self.store
            .append_open_api_event(
                &scope,
                &event_id,
                &request.event_type,
                &request.source_type,
                &request.event_time,
                &request.payload,
                sensitivity,
            )
            .await
            .map_err(Self::map_store_error)?;

        self.store
            .retrieve_open_api_event(&scope, &event_id)
            .await
            .map_err(Self::map_store_error)?
            .map(Self::map_event)
            .transpose()?
            .ok_or_else(|| MemoryServiceError::storage("created event could not be loaded"))
    }

    async fn retrieve_event(
        &self,
        context: MemoryOpenApiRequestContext,
        event_id: u64,
        space_id: u64,
    ) -> MemoryServiceResult<MemoryEvent> {
        self.load_scoped_event(&context, space_id, event_id).await
    }

    async fn list_memories(
        &self,
        context: MemoryOpenApiRequestContext,
        query: ListMemoriesQuery,
    ) -> MemoryServiceResult<MemoryRecordList> {
        let space_id = access::require_list_space_id(query.space_id)?;
        access::assert_actor_can_access_space(&self.store, &context, space_id).await?;
        let scope = Self::scope(&context, space_id)?;
        let page_size = query.page_size.unwrap_or(20);
        let rows = self
            .store
            .list_record_details(
                &scope,
                query.q.as_deref(),
                page_size,
                query.cursor.as_deref(),
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() > page_size as usize;
        let items = rows
            .into_iter()
            .take(page_size as usize)
            .map(Self::map_record)
            .collect::<Result<Vec<_>, _>>()?;
        let next_cursor = items.last().map(|record| record.memory_id.to_string());

        Ok(MemoryRecordList {
            items,
            page_info: MemoryPageInfo {
                next_cursor: if has_more { next_cursor } else { None },
                has_more,
                page_size: Some(page_size),
            },
        })
    }

    async fn create_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryRecordRequest,
    ) -> MemoryServiceResult<MemoryRecord> {
        access::assert_actor_can_access_space_for_write(&self.store, &context, request.space_id).await?;
        let scope = Self::scope(&context, request.space_id)?;
        crate::tenant_quota::assert_space_record_quota(
            &self.store,
            &scope,
            crate::tenant_quota::MemoryQuotaLimits::from_env(),
        )
        .await?;
        let memory_id = self.next_id()?.to_string();
        let object_text = request
            .object_text
            .unwrap_or_else(|| request.canonical_text.clone());
        let sensitivity = Self::normalize_sensitivity_level(request.sensitivity_level.as_deref())?;

        self.store
            .create_record_open_api(
                &scope,
                &memory_id,
                &request.scope,
                Self::memory_type_to_db(request.memory_type),
                request.subject.as_deref(),
                request.predicate.as_deref(),
                &object_text,
                &request.canonical_text,
                sensitivity,
            )
            .await
            .map_err(Self::map_store_error)?;

        let record = self
            .store
            .retrieve_record_detail(&scope, &memory_id)
            .await
            .map_err(Self::map_store_error)?
            .map(Self::map_record)
            .transpose()?
            .ok_or_else(|| MemoryServiceError::storage("created memory could not be loaded"))?;

        self.publish_domain_event(
            &scope,
            "memory.record.created",
            "memory_record",
            &memory_id,
            serde_json::json!({
                "memoryId": memory_id,
                "spaceId": request.space_id,
                "memoryType": request.memory_type,
            }),
        )
        .await?;

        Ok(record)
    }

    async fn retrieve_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        memory_id: u64,
        space_id: u64,
    ) -> MemoryServiceResult<MemoryRecord> {
        self.load_scoped_record(&context, space_id, memory_id)
            .await
    }

    async fn update_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        memory_id: u64,
        space_id: u64,
        patch: MemoryRecordPatch,
    ) -> MemoryServiceResult<MemoryRecord> {
        access::assert_actor_can_access_space_for_write(&self.store, &context, space_id).await?;
        let scope = Self::scope(&context, space_id)?;
        let _existing = self.load_scoped_record(&context, space_id, memory_id).await?;

        match self
            .store
            .update_record_open_api(
                &scope,
                &memory_id.to_string(),
                patch.canonical_text.as_deref(),
                patch.subject.as_deref(),
            )
            .await
            .map_err(Self::map_store_error)?
        {
            Some(row) => {
                let record = Self::map_record(row)?;
                self.publish_domain_event(
                    &scope,
                    "memory.record.updated",
                    "memory_record",
                    &memory_id.to_string(),
                    serde_json::json!({
                        "memoryId": memory_id,
                        "spaceId": space_id,
                    }),
                )
                .await?;
                Ok(record)
            }
            None => Err(MemoryServiceError::not_found("memory not found")),
        }
    }

    async fn delete_memory(
        &self,
        context: MemoryOpenApiRequestContext,
        memory_id: u64,
        space_id: u64,
    ) -> MemoryServiceResult<()> {
        access::assert_actor_can_access_space_for_write(&self.store, &context, space_id).await?;
        let scope = Self::scope(&context, space_id)?;
        let _existing = self.load_scoped_record(&context, space_id, memory_id).await?;

        self.store
            .mark_record_deleted(&scope, &memory_id.to_string())
            .await
            .map_err(Self::map_store_error)?;
        self.publish_domain_event(
            &scope,
            "memory.record.deleted",
            "memory_record",
            &memory_id.to_string(),
            serde_json::json!({
                "memoryId": memory_id,
                "spaceId": space_id,
            }),
        )
        .await?;
        Ok(())
    }

    async fn create_retrieval(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryRetrievalRequest,
    ) -> MemoryServiceResult<MemoryRetrievalResult> {
        if request.space_ids.is_empty() {
            return Err(MemoryServiceError::validation("spaceIds must not be empty"));
        }

        access::assert_actor_can_access_spaces(&self.store, &context, &request.space_ids).await?;

        let started = std::time::Instant::now();
        let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
        self.store
            .ensure_default_retrieval_profile_for_tenant(tenant_id)
            .await
            .map_err(Self::map_store_error)?;

        let (effective_top_k, applied_profile_id, profile_retrievers) =
            if let Some(profile_id) = request.retrieval_profile_id {
                if let Some(row) = self
                    .store
                    .retrieve_mem_retrieval_profile_for_tenant(
                        tenant_id,
                        &profile_id.to_string(),
                    )
                    .await
                    .map_err(Self::map_store_error)?
                {
                    let retrievers = serde_json::from_str(&row.retrievers_json).ok();
                    (row.top_k.min(request.top_k), Some(profile_id), retrievers)
                } else {
                    (request.top_k, None, Self::default_retriever_profile())
                }
            } else {
                (request.top_k, None, Self::default_retriever_profile())
            };

        let memory_type_filter = request.memory_types.as_ref().map(|types| {
            types
                .iter()
                .map(|value| Self::memory_type_to_db(*value))
                .collect::<Vec<_>>()
        });

        let mut candidates = Vec::new();
        let mut space_owner_cache = std::collections::HashMap::new();
        for space_id in &request.space_ids {
            let scope = Self::scope(&context, *space_id)?;
            let actor_is_owner = if let Some(cached) = space_owner_cache.get(space_id) {
                *cached
            } else {
                let is_owner = access::actor_is_space_owner(&self.store, &context, *space_id).await?;
                space_owner_cache.insert(*space_id, is_owner);
                is_owner
            };
            let rows = self
                .store
                .search_record_details_keyword(&scope, &request.query, effective_top_k)
                .await
                .map_err(Self::map_store_error)?;

            let event_rows = self
                .store
                .search_open_api_events_keyword(&scope, &request.query, effective_top_k)
                .await
                .map_err(Self::map_store_error)?;

            let record_inputs: Vec<RetrievalRecordInput> = rows
                .iter()
                .map(|row| RetrievalRecordInput {
                    memory_id: row.memory_id.clone(),
                    subject: row.subject.clone(),
                    predicate: row.predicate.clone(),
                    object_text: row.object_text.clone(),
                    canonical_text: row.canonical_text.clone(),
                    created_at: row.created_at.clone(),
                })
                .collect();
            let event_inputs: Vec<RetrievalEventInput> = event_rows
                .iter()
                .map(|row| RetrievalEventInput {
                    event_id: row.event_id.clone(),
                    payload_text: row.payload.to_string(),
                    created_at: row.created_at.clone(),
                })
                .collect();

            let orchestrated = orchestrate_retrieval_candidates(
                &request.query,
                &record_inputs,
                &event_inputs,
                profile_retrievers.as_ref(),
                effective_top_k as usize,
            );

            let rows_by_memory_id: std::collections::HashMap<&str, &_> = rows
                .iter()
                .map(|row| (row.memory_id.as_str(), row))
                .collect();

            for candidate in orchestrated {
                let memory = if candidate.record.memory_id.starts_with("event:") {
                    continue;
                } else if let Some(row) = rows_by_memory_id.get(candidate.record.memory_id.as_str()) {
                    Self::map_record((*row).clone())?
                } else {
                    continue;
                };
                if !access::actor_may_read_sensitivity(
                    &context,
                    &memory.sensitivity_level,
                    actor_is_owner,
                ) {
                    continue;
                }
                if let Some(filters) = &memory_type_filter {
                    let memory_type = Self::memory_type_to_db(memory.memory_type);
                    if !filters.iter().any(|filter| filter == &memory_type) {
                        continue;
                    }
                }
                if candidate.raw_score > 0.0 {
                    candidates.push(RetrievalCandidate {
                        memory,
                        retriever_name: candidate.retriever_name,
                        raw_score: candidate.raw_score,
                        rank: 0,
                    });
                }
            }
        }

        let fused = fuse_retrieval_candidates(candidates, effective_top_k as usize);
        let retrieval_id = self.next_id()?;
        let trace_id = retrieval_id.to_string();
        let primary_scope = Self::scope(&context, request.space_ids[0])?;
        let latency_ms = platform::elapsed_millis_i64(started);
        let query_hash = platform::stable_query_hash(&request.query);
        let retrievers_json = profile_retrievers
            .as_ref()
            .map(|value| value.to_string())
            .unwrap_or_else(|| r#"{"keyword":{"weight":1.0}}"#.to_string());
        let hits: Vec<MemoryRetrievalHit> = fused
            .iter()
            .enumerate()
            .map(|(_index, hit)| {
                Ok(MemoryRetrievalHit {
                    hit_id: self.next_id()?,
                    memory: Some(hit.memory.clone()),
                    memory_id: Some(hit.memory.memory_id),
                    retriever_name: hit.retriever_name.clone(),
                    result_rank: hit.rank,
                    raw_score: Some(hit.raw_score),
                    fused_score: Some(hit.fused_score),
                    explanation: None,
                    status: "accepted".to_string(),
                })
            })
            .collect::<MemoryServiceResult<Vec<_>>>()?;

        let trace_hits: Vec<MemoryRetrievalHitDraft> = hits
            .iter()
            .map(|hit| MemoryRetrievalHitDraft {
                hit_id: hit.hit_id.to_string(),
                memory_id: hit.memory_id.map(|value| value.to_string()),
                retriever_name: hit.retriever_name.clone(),
                result_rank: i64::from(hit.result_rank),
                raw_score: hit.raw_score,
                fused_score: hit.fused_score,
                explanation_json: None,
                status: hit.status.clone(),
            })
            .collect();

        let _ = self
            .store
            .append_retrieval_trace(&AppendMemoryRetrievalTraceCommand {
                scope: primary_scope,
                trace_id: trace_id.clone(),
                actor_id: request.actor_id.clone(),
                query_text: Some(request.query.clone()),
                query_hash: query_hash.clone(),
                retrievers_json: Some(retrievers_json.clone()),
                latency_ms: Some(latency_ms),
                degraded: false,
                metadata_json: request.filters.as_ref().map(|filters| {
                    serde_json::json!({ "filters": filters }).to_string()
                }),
                hits: trace_hits,
                context_pack: None,
            })
            .await
            .map_err(Self::map_store_error)?;

        let trace = if request.include_trace.unwrap_or(false) {
            Some(MemoryRetrievalTrace {
                trace_id: retrieval_id,
                space_id: Some(request.space_ids[0]),
                retrieval_profile_id: applied_profile_id,
                actor_id: request.actor_id,
                query_text: Some(request.query),
                query_hash,
                result_count: hits.len() as i32,
                degraded: false,
                created_at: platform::current_timestamp(),
            })
        } else {
            None
        };

        info!(
            tenant_id,
            retrieval_id,
            hit_count = hits.len(),
            latency_ms,
            space_count = request.space_ids.len(),
            "memory retrieval completed"
        );
        crate::domain_metrics::memory_domain_metrics().record_retrieval_completed();

        Ok(MemoryRetrievalResult {
            retrieval_id,
            trace,
            hits,
            degraded: false,
        })
    }

    async fn retrieve_retrieval(
        &self,
        context: MemoryOpenApiRequestContext,
        retrieval_id: u64,
    ) -> MemoryServiceResult<MemoryRetrievalResult> {
        let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
        let lookup = self
            .store
            .retrieve_retrieval_trace_lookup_for_tenant(tenant_id, &retrieval_id.to_string())
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("retrieval not found"))?;
        access::assert_actor_can_access_space_i64(&self.store, &context, lookup.space_id).await?;
        let trace = lookup.trace;
        let trace_space_id = u64::try_from(lookup.space_id.max(0))
            .map_err(|_| MemoryServiceError::storage("space id must be non-negative"))?;
        let scope = Self::scope(&context, trace_space_id)?;

        let mut hits = Vec::new();
        for (index, hit) in trace.hits.iter().enumerate() {
            let memory = if let Some(memory_id) = hit.memory_id.as_deref() {
                self.store
                    .retrieve_record_detail(&scope, memory_id)
                    .await
                    .map_err(Self::map_store_error)?
                    .map(Self::map_record)
                    .transpose()?
            } else {
                None
            };

            hits.push(MemoryRetrievalHit {
                hit_id: hit
                    .hit_id
                    .parse()
                    .ok()
                    .or_else(|| Self::parse_id(&hit.hit_id))
                    .unwrap_or_else(|| retrieval_id.saturating_add(index as u64 + 1)),
                memory,
                memory_id: hit.memory_id.as_deref().and_then(Self::parse_id),
                retriever_name: hit.retriever_name.clone(),
                result_rank: i32::try_from(hit.result_rank).unwrap_or(1),
                raw_score: hit.raw_score,
                fused_score: hit.fused_score,
                explanation: hit
                    .explanation_json
                    .as_deref()
                    .and_then(|value| serde_json::from_str(value).ok()),
                status: hit.status.clone(),
            });
        }

        Ok(MemoryRetrievalResult {
            retrieval_id,
            trace: Some(MemoryRetrievalTrace {
                trace_id: retrieval_id,
                space_id: Some(trace_space_id),
                retrieval_profile_id: None,
                actor_id: trace.actor_id,
                query_text: trace.query_text,
                query_hash: trace.query_hash,
                result_count: trace.result_count as i32,
                degraded: trace.degraded,
                created_at: lookup.created_at.clone(),
            }),
            hits,
            degraded: trace.degraded,
        })
    }

    async fn retrieve_provider_health(
        &self,
        context: MemoryOpenApiRequestContext,
    ) -> MemoryServiceResult<MemoryProviderHealth> {
        let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
        let rows = self
            .store
            .list_mem_provider_bindings_for_tenant(tenant_id, 100, None)
            .await
            .map_err(Self::map_store_error)?;
        let providers = rows
            .iter()
            .map(Self::map_provider_binding_public)
            .collect::<Result<Vec<_>, _>>()?;
        let status = if providers.is_empty() {
            MemoryProviderHealthStatus::Healthy
        } else if providers
            .iter()
            .all(|provider| provider.health_state == "healthy")
        {
            MemoryProviderHealthStatus::Healthy
        } else if providers
            .iter()
            .any(|provider| provider.health_state == "unhealthy")
        {
            MemoryProviderHealthStatus::Unhealthy
        } else {
            MemoryProviderHealthStatus::Degraded
        };
        Ok(MemoryProviderHealth {
            status,
            checked_at: platform::current_timestamp(),
            providers,
        })
    }

    async fn create_context_pack(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryContextPackRequest,
    ) -> MemoryServiceResult<MemoryContextPack> {
        if request.space_ids.is_empty() {
            return Err(MemoryServiceError::validation("spaceIds must not be empty"));
        }

        access::assert_actor_can_access_spaces(&self.store, &context, &request.space_ids).await?;

        let top_k = if request.context_budget_tokens > 0 {
            (request.context_budget_tokens / 200).clamp(1, 50)
        } else {
            10
        };

        let retrieval = self
            .create_retrieval(
                context.clone(),
                MemoryRetrievalRequest {
                    query: request.query.clone(),
                    space_ids: request.space_ids.clone(),
                    actor_id: request.actor_id.clone(),
                    retrieval_profile_id: request.retrieval_profile_id,
                    memory_types: None,
                    filters: request.filters.clone(),
                    top_k,
                    context_budget_tokens: request.context_budget_tokens,
                    include_trace: Some(false),
                },
            )
            .await?;

        let (pack, estimated_tokens, truncated) =
            build_context_pack_from_hits(&retrieval.hits, request.context_budget_tokens);
        let context_pack_id = self.next_id()?;
        let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
        let primary_space = request.space_ids[0] as i64;

        self.store
            .insert_context_pack_open_api(
                tenant_id,
                primary_space,
                &context_pack_id.to_string(),
                Some(&retrieval.retrieval_id.to_string()),
                request.actor_id.as_deref(),
                Some(&request.query),
                &pack.to_string(),
                i64::from(estimated_tokens),
                truncated,
            )
            .await
            .map_err(Self::map_store_error)?;

        Ok(MemoryContextPack {
            context_pack_id,
            retrieval_id: Some(retrieval.retrieval_id),
            query: Some(request.query),
            pack,
            estimated_tokens,
            truncated,
            created_at: platform::current_timestamp(),
        })
    }

    async fn retrieve_context_pack(
        &self,
        context: MemoryOpenApiRequestContext,
        context_pack_id: u64,
    ) -> MemoryServiceResult<MemoryContextPack> {
        let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
        let row = self
            .store
            .retrieve_context_pack_for_tenant(tenant_id, &context_pack_id.to_string())
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("context pack not found"))?;

        if let Some(space_id) = row.space_id {
            access::assert_actor_can_access_space_i64(&self.store, &context, space_id).await?;
        } else {
            return Err(MemoryServiceError::forbidden(
                "context pack is not linked to an authorized memory space",
            ));
        }

        let pack = serde_json::from_str(&row.pack_json).map_err(|error| {
            MemoryServiceError::storage(format!("context pack payload is corrupt: {error}"))
        })?;

        Ok(MemoryContextPack {
            context_pack_id,
            retrieval_id: None,
            query: row.query_text,
            pack,
            estimated_tokens: row.estimated_tokens as i32,
            truncated: row.truncated,
            created_at: row.created_at,
        })
    }

    async fn create_feedback(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryFeedbackRequest,
    ) -> MemoryServiceResult<MemoryFeedback> {
        let feedback_id = self.next_id()?;
        let space_id = if request.target_type == "memory" {
            let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
            let memory = self
                .store
                .retrieve_record_detail_for_tenant(tenant_id, &request.target_id.to_string())
                .await
                .map_err(Self::map_store_error)?
                .ok_or_else(|| MemoryServiceError::not_found("memory not found"))?;
            u64::try_from(memory.space_id)
                .map_err(|_| MemoryServiceError::storage("space id must be non-negative"))?
        } else if request.target_type == "retrieval" {
            let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
            let lookup = self
                .store
                .retrieve_retrieval_trace_lookup_for_tenant(
                    tenant_id,
                    &request.target_id.to_string(),
                )
                .await
                .map_err(Self::map_store_error)?
                .ok_or_else(|| MemoryServiceError::not_found("retrieval not found"))?;
            u64::try_from(lookup.space_id.max(0))
                .map_err(|_| MemoryServiceError::storage("space id must be non-negative"))?
        } else if request.target_type == "candidate" {
            let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
            let row = self
                .store
                .retrieve_candidate_for_tenant(tenant_id, &request.target_id.to_string())
                .await
                .map_err(Self::map_store_error)?
                .ok_or_else(|| MemoryServiceError::not_found("candidate not found"))?;
            u64::try_from(row.space_id.max(0))
                .map_err(|_| MemoryServiceError::storage("space id must be non-negative"))?
        } else {
            return Err(MemoryServiceError::validation(
                "feedback targetType must be memory, retrieval, or candidate",
            ));
        };
        let scope = Self::scope(&context, space_id)?;
        access::assert_actor_can_access_space_for_write(&self.store, &context, space_id).await?;
        self.store
            .append_audit(
                &scope,
                &feedback_id.to_string(),
                "feedback.create",
                &request.target_type,
                &request.target_id.to_string(),
                "accepted",
            )
            .await
            .map_err(Self::map_store_error)?;

        Ok(MemoryFeedback {
            feedback_id,
            target_type: request.target_type,
            target_id: request.target_id,
            feedback_type: request.feedback_type,
            created_at: platform::current_timestamp(),
        })
    }

    async fn create_extraction(
        &self,
        context: MemoryOpenApiRequestContext,
        request: MemoryExtractionRequest,
    ) -> MemoryServiceResult<MemoryLearningJob> {
        access::assert_actor_can_access_space_for_write(&self.store, &context, request.space_id).await?;
        let job_id = self.next_id()?;
        let scope = Self::scope(&context, request.space_id)?;
        let mut created_candidates = 0_u32;

        for event_id in &request.input_events {
            if let Some(payload) = self
                .store
                .retrieve_event_payload(&scope, &event_id.to_string())
                .await
                .map_err(Self::map_store_error)?
            {
                let proposed = payload
                    .get("content")
                    .and_then(|value| value.as_str())
                    .unwrap_or("extracted memory candidate")
                    .to_string();
                let candidate_id = self.next_id()?.to_string();
                self.store
                    .create_candidate(&CreateMemoryCandidateCommand {
                        scope: scope.clone(),
                        candidate_id,
                        candidate_type: "extraction".to_string(),
                        memory_type: "semantic".to_string(),
                        proposed_text: proposed,
                        proposed_payload_json: Some(payload.to_string()),
                        evidence_json: Some(format!(r#"["event:{event_id}"]"#)),
                        confidence: 0.7,
                    })
                    .await
                    .map_err(Self::map_store_error)?;
                created_candidates += 1;
            }
        }

        Ok(MemoryLearningJob {
            job_id,
            space_id: Some(request.space_id),
            job_type: "extraction".to_string(),
            state: if created_candidates > 0 {
                "completed".to_string()
            } else {
                "failed".to_string()
            },
            priority: 0,
            result: Some(serde_json::json!({
                "candidateCount": created_candidates,
                "extractionMode": request.extraction_mode.unwrap_or_else(|| "deterministic".to_string()),
            })),
            error: None,
            started_at: None,
            finished_at: None,
            created_at: platform::current_timestamp(),
            updated_at: platform::current_timestamp(),
            version: None,
        })
    }

    async fn list_candidates(
        &self,
        context: MemoryOpenApiRequestContext,
        query: ListCandidatesQuery,
    ) -> MemoryServiceResult<MemoryCandidateList> {
        let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
        let space_id = access::require_list_space_id(query.space_id)?;
        access::assert_actor_can_access_space(&self.store, &context, space_id).await?;
        let page_size = query.page_size.unwrap_or(20);
        let rows = self
            .store
            .list_candidates_for_tenant(
                tenant_id,
                Some(space_id as i64),
                page_size,
                query.cursor.as_deref(),
            )
            .await
            .map_err(Self::map_store_error)?;
        let has_more = rows.len() > page_size as usize;
        let items = rows
            .into_iter()
            .take(page_size as usize)
            .map(|row| {
                Ok(MemoryCandidate {
                    candidate_id: row.candidate_id.parse().unwrap_or(0),
                    space_id: u64::try_from(row.space_id.max(0)).unwrap_or(0),
                    candidate_type: row.candidate_type,
                    memory_type: Self::memory_type_from_db(&row.memory_type),
                    proposed_text: row.proposed_text,
                    confidence: row.confidence,
                    decision_state: row.decision_state,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
            })
            .collect::<MemoryServiceResult<Vec<_>>>()?;
        let next_cursor = items
            .last()
            .map(|candidate| candidate.candidate_id.to_string());

        Ok(MemoryCandidateList {
            items,
            page_info: MemoryPageInfo {
                next_cursor: if has_more { next_cursor } else { None },
                has_more,
                page_size: Some(page_size),
            },
        })
    }

    async fn retrieve_candidate(
        &self,
        context: MemoryOpenApiRequestContext,
        candidate_id: u64,
    ) -> MemoryServiceResult<MemoryCandidate> {
        let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
        match self
            .store
            .retrieve_candidate_for_tenant(tenant_id, &candidate_id.to_string())
            .await
            .map_err(Self::map_store_error)?
        {
            Some(row) => {
                access::assert_actor_can_access_space_i64(&self.store, &context, row.space_id)
                    .await?;
                Self::map_candidate(row)
            }
            None => Err(MemoryServiceError::not_found("candidate not found")),
        }
    }
}

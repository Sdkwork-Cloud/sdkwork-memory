use sdkwork_memory_contract::{
    MemoryBackendRequestContext, MemoryExtractionRequest, MemoryOpenApi, MemoryRecordPatch,
    MemoryServiceError, MemoryServiceResult,
};
use sdkwork_memory_plugin_native_sql::{
    NativeSqlEvalRunRow, NativeSqlImplementationProfileRow, NativeSqlMemoryIndexRow,
    NativeSqlProviderBindingRow, NativeSqlRetrievalProfileRow,
};

use crate::open_api::OpenMemoryService;
use crate::platform;

const RT_EXTRACTION_JOB: &str = "extraction_job";
const RT_CONSOLIDATION_JOB: &str = "consolidation_job";
const RT_RETENTION_JOB: &str = "retention_job";
const RT_MIGRATION_JOB: &str = "migration_job";

impl OpenMemoryService {
    fn admin_page_info(page_size: i32) -> serde_json::Value {
        serde_json::json!({
            "nextCursor": null,
            "hasMore": false,
            "pageSize": page_size,
        })
    }

    fn page_size_from_query(query: &serde_json::Value) -> i32 {
        query
            .get("page_size")
            .or_else(|| query.get("pageSize"))
            .and_then(|value| value.as_i64())
            .unwrap_or(20) as i32
    }

    fn optional_space_id(request: &serde_json::Value) -> Option<i64> {
        request
            .get("spaceId")
            .and_then(|value| value.as_str())
            .and_then(|value| value.parse().ok())
    }

    fn optional_u64_json(value: Option<i64>) -> serde_json::Value {
        value.map_or(serde_json::Value::Null, |id| {
            serde_json::Value::String(id.to_string())
        })
    }

    fn memory_index_json(row: &NativeSqlMemoryIndexRow) -> MemoryServiceResult<serde_json::Value> {
        let config = row
            .config_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok())
            .unwrap_or(serde_json::Value::Null);
        Ok(serde_json::json!({
            "indexId": row.index_uuid,
            "spaceId": Self::optional_u64_json(row.space_id),
            "indexKind": row.index_kind,
            "schemaVersion": row.schema_version,
            "status": row.status,
            "config": config,
            "lastRebuiltAt": row.last_rebuilt_at,
            "createdAt": row.created_at,
            "updatedAt": row.updated_at,
            "version": row.version.to_string(),
        }))
    }

    fn retrieval_profile_json(
        row: &NativeSqlRetrievalProfileRow,
    ) -> MemoryServiceResult<serde_json::Value> {
        let retrievers: serde_json::Value =
            serde_json::from_str(&row.retrievers_json).map_err(|error| {
                MemoryServiceError::storage(format!(
                    "retrieval profile retrievers decode failed: {error}"
                ))
            })?;
        Ok(serde_json::json!({
            "profileId": row.profile_uuid,
            "spaceId": Self::optional_u64_json(row.space_id),
            "name": row.name,
            "strategy": row.strategy,
            "retrievers": retrievers,
            "topK": row.top_k,
            "contextBudgetTokens": row.context_budget_tokens,
            "status": row.status,
            "createdAt": row.created_at,
            "updatedAt": row.updated_at,
            "version": row.version.to_string(),
        }))
    }

    fn implementation_profile_json(
        row: &NativeSqlImplementationProfileRow,
    ) -> MemoryServiceResult<serde_json::Value> {
        let capabilities: serde_json::Value =
            serde_json::from_str(&row.capability_json).map_err(|error| {
                MemoryServiceError::storage(format!(
                    "implementation profile capabilities decode failed: {error}"
                ))
            })?;
        Ok(serde_json::json!({
            "profileId": row.profile_uuid,
            "name": row.name,
            "implementationKind": row.implementation_kind,
            "role": row.role,
            "status": row.status,
            "capabilities": capabilities,
            "createdAt": row.created_at,
            "updatedAt": row.updated_at,
            "version": row.version.to_string(),
        }))
    }

    fn provider_binding_json(row: &NativeSqlProviderBindingRow) -> serde_json::Value {
        serde_json::json!({
            "bindingId": row.binding_uuid,
            "providerKind": row.provider_kind,
            "providerCode": row.provider_code,
            "displayName": row.display_name,
            "healthState": row.health_state,
            "createdAt": row.created_at,
            "updatedAt": row.updated_at,
            "version": row.version.to_string(),
        })
    }

    fn eval_run_json(row: &NativeSqlEvalRunRow) -> MemoryServiceResult<serde_json::Value> {
        let metrics = row
            .metrics_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok())
            .unwrap_or(serde_json::Value::Null);
        Ok(serde_json::json!({
            "evalRunId": row.eval_run_uuid,
            "evalType": row.eval_type,
            "state": row.state,
            "metrics": metrics,
            "createdAt": row.created_at,
            "updatedAt": row.updated_at,
        }))
    }

    async fn save_governance_entity(
        &self,
        tenant_id: i64,
        resource_type: &str,
        entity_id: &str,
        entity: &serde_json::Value,
    ) -> MemoryServiceResult<()> {
        let metadata = serde_json::to_string(entity).map_err(|error| {
            MemoryServiceError::storage(format!("governance entity encode failed: {error}"))
        })?;
        self.store
            .save_admin_config_entity(tenant_id, resource_type, entity_id, &metadata)
            .await
            .map_err(OpenMemoryService::map_store_error)
    }

    async fn load_governance_entity(
        &self,
        tenant_id: i64,
        resource_type: &str,
        entity_id: &str,
    ) -> MemoryServiceResult<serde_json::Value> {
        let metadata = self
            .store
            .retrieve_admin_config_entity(tenant_id, resource_type, entity_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("governance entity not found"))?;
        serde_json::from_str(&metadata).map_err(|error| {
            MemoryServiceError::storage(format!("governance entity decode failed: {error}"))
        })
    }

    pub(crate) async fn backend_list_indexes(
        &self,
        context: MemoryBackendRequestContext,
        query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let page_size = Self::page_size_from_query(&query);
        self.store
            .ensure_default_keyword_index_for_tenant(tenant_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let rows = self
            .store
            .list_mem_indexes_for_tenant(tenant_id, page_size)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let items = rows
            .iter()
            .map(Self::memory_index_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(serde_json::json!({
            "items": items,
            "pageInfo": Self::admin_page_info(page_size),
        }))
    }

    pub(crate) async fn backend_create_index(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let index_id = self.next_id()?.to_string();
        let config_json = request
            .get("config")
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("index config encode failed: {error}"))
            })?;
        self.store
            .insert_mem_index(
                tenant_id,
                &index_id,
                Self::optional_space_id(&request),
                request
                    .get("indexKind")
                    .and_then(|value| value.as_str())
                    .unwrap_or("keyword"),
                request
                    .get("schemaVersion")
                    .and_then(|value| value.as_str())
                    .unwrap_or("2026-06-10"),
                request
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("active"),
                config_json.as_deref(),
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_index_for_tenant(tenant_id, &index_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("created index could not be loaded"))?;
        Self::memory_index_json(&row)
    }

    pub(crate) async fn backend_retrieve_index(
        &self,
        context: MemoryBackendRequestContext,
        index_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        self.store
            .ensure_default_keyword_index_for_tenant(tenant_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_index_for_tenant(tenant_id, &index_id.to_string())
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("memory index not found"))?;
        Self::memory_index_json(&row)
    }

    pub(crate) async fn backend_update_index(
        &self,
        context: MemoryBackendRequestContext,
        index_id: u64,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let config_json = request
            .get("config")
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("index config encode failed: {error}"))
            })?;
        let row = self
            .store
            .update_mem_index_for_tenant(
                tenant_id,
                &index_id.to_string(),
                request.get("status").and_then(|value| value.as_str()),
                config_json.as_deref(),
                None,
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("memory index not found"))?;
        Self::memory_index_json(&row)
    }

    pub(crate) async fn backend_rebuild_index(
        &self,
        context: MemoryBackendRequestContext,
        index_id: u64,
        _request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let rebuilt_at = platform::current_timestamp();
        let row = self
            .store
            .update_mem_index_for_tenant(
                tenant_id,
                &index_id.to_string(),
                Some("active"),
                None,
                Some(rebuilt_at.as_str()),
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("memory index not found"))?;
        Self::memory_index_json(&row)
    }

    pub(crate) async fn backend_list_retrieval_profiles(
        &self,
        context: MemoryBackendRequestContext,
        query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let page_size = Self::page_size_from_query(&query);
        self.store
            .ensure_default_retrieval_profile_for_tenant(tenant_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let rows = self
            .store
            .list_mem_retrieval_profiles_for_tenant(tenant_id, page_size)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let items = rows
            .iter()
            .map(Self::retrieval_profile_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(serde_json::json!({
            "items": items,
            "pageInfo": Self::admin_page_info(page_size),
        }))
    }

    pub(crate) async fn backend_create_retrieval_profile(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let profile_id = self.next_id()?.to_string();
        let retrievers_json = request
            .get("retrievers")
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("retrievers encode failed: {error}"))
            })?
            .unwrap_or_else(|| r#"[{"name":"keyword","weight":1.0}]"#.to_string());
        self.store
            .insert_mem_retrieval_profile(
                tenant_id,
                &profile_id,
                Self::optional_space_id(&request),
                request
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("custom-profile"),
                request
                    .get("strategy")
                    .and_then(|value| value.as_str())
                    .unwrap_or("deterministic"),
                &retrievers_json,
                request
                    .get("topK")
                    .and_then(|value| value.as_i64())
                    .unwrap_or(10) as i32,
                request
                    .get("contextBudgetTokens")
                    .and_then(|value| value.as_i64())
                    .unwrap_or(2048) as i32,
                request
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("active"),
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_retrieval_profile_for_tenant(tenant_id, &profile_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("created profile could not be loaded"))?;
        Self::retrieval_profile_json(&row)
    }

    pub(crate) async fn backend_retrieve_retrieval_profile(
        &self,
        context: MemoryBackendRequestContext,
        profile_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        self.store
            .ensure_default_retrieval_profile_for_tenant(tenant_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_retrieval_profile_for_tenant(tenant_id, &profile_id.to_string())
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("retrieval profile not found"))?;
        Self::retrieval_profile_json(&row)
    }

    pub(crate) async fn backend_update_retrieval_profile(
        &self,
        context: MemoryBackendRequestContext,
        profile_id: u64,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let retrievers_json = request
            .get("retrievers")
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("retrievers encode failed: {error}"))
            })?;
        let row = self
            .store
            .update_mem_retrieval_profile_for_tenant(
                tenant_id,
                &profile_id.to_string(),
                request.get("name").and_then(|value| value.as_str()),
                request.get("strategy").and_then(|value| value.as_str()),
                retrievers_json.as_deref(),
                request
                    .get("topK")
                    .and_then(|value| value.as_i64())
                    .map(|value| value as i32),
                request
                    .get("contextBudgetTokens")
                    .and_then(|value| value.as_i64())
                    .map(|value| value as i32),
                request.get("status").and_then(|value| value.as_str()),
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("retrieval profile not found"))?;
        Self::retrieval_profile_json(&row)
    }

    pub(crate) async fn backend_list_implementation_profiles(
        &self,
        context: MemoryBackendRequestContext,
        query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let page_size = Self::page_size_from_query(&query);
        self.store
            .ensure_default_implementation_profile_for_tenant(tenant_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let rows = self
            .store
            .list_mem_implementation_profiles_for_tenant(tenant_id, page_size)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let items = rows
            .iter()
            .map(Self::implementation_profile_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(serde_json::json!({
            "items": items,
            "pageInfo": Self::admin_page_info(page_size),
        }))
    }

    pub(crate) async fn backend_create_implementation_profile(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let profile_id = self.next_id()?.to_string();
        let capability_json = request
            .get("capabilities")
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("capabilities encode failed: {error}"))
            })?
            .unwrap_or_else(|| r#"{"keyword":true,"embedding":false}"#.to_string());
        self.store
            .insert_mem_implementation_profile(
                tenant_id,
                &profile_id,
                request
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("custom-implementation"),
                request
                    .get("implementationKind")
                    .and_then(|value| value.as_str())
                    .unwrap_or("native_sql"),
                request
                    .get("role")
                    .and_then(|value| value.as_str())
                    .unwrap_or("primary"),
                request
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("active"),
                &capability_json,
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_implementation_profile_for_tenant(tenant_id, &profile_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("created profile could not be loaded"))?;
        Self::implementation_profile_json(&row)
    }

    pub(crate) async fn backend_retrieve_implementation_profile(
        &self,
        context: MemoryBackendRequestContext,
        profile_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        self.store
            .ensure_default_implementation_profile_for_tenant(tenant_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_implementation_profile_for_tenant(tenant_id, &profile_id.to_string())
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("implementation profile not found"))?;
        Self::implementation_profile_json(&row)
    }

    pub(crate) async fn backend_update_implementation_profile(
        &self,
        context: MemoryBackendRequestContext,
        profile_id: u64,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let capability_json = request
            .get("capabilities")
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("capabilities encode failed: {error}"))
            })?;
        let row = self
            .store
            .update_mem_implementation_profile_for_tenant(
                tenant_id,
                &profile_id.to_string(),
                request.get("name").and_then(|value| value.as_str()),
                request
                    .get("implementationKind")
                    .and_then(|value| value.as_str()),
                request.get("role").and_then(|value| value.as_str()),
                request.get("status").and_then(|value| value.as_str()),
                capability_json.as_deref(),
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("implementation profile not found"))?;
        Self::implementation_profile_json(&row)
    }

    pub(crate) async fn backend_list_provider_bindings(
        &self,
        context: MemoryBackendRequestContext,
        query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let page_size = Self::page_size_from_query(&query);
        let rows = self
            .store
            .list_mem_provider_bindings_for_tenant(tenant_id, page_size)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        Ok(serde_json::json!({
            "items": rows.iter().map(Self::provider_binding_json).collect::<Vec<_>>(),
            "pageInfo": Self::admin_page_info(page_size),
        }))
    }

    pub(crate) async fn backend_create_provider_binding(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let binding_id = self.next_id()?.to_string();
        self.store
            .insert_mem_provider_binding(
                tenant_id,
                &binding_id,
                request
                    .get("providerKind")
                    .and_then(|value| value.as_str())
                    .unwrap_or("memory"),
                request
                    .get("providerCode")
                    .and_then(|value| value.as_str())
                    .unwrap_or("native_sql"),
                request
                    .get("displayName")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Native SQL"),
                "healthy",
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_provider_binding_for_tenant(tenant_id, &binding_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("created binding could not be loaded"))?;
        Ok(Self::provider_binding_json(&row))
    }

    pub(crate) async fn backend_update_provider_binding(
        &self,
        context: MemoryBackendRequestContext,
        binding_id: u64,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let row = self
            .store
            .update_mem_provider_binding_for_tenant(
                tenant_id,
                &binding_id.to_string(),
                request.get("displayName").and_then(|value| value.as_str()),
                request.get("providerCode").and_then(|value| value.as_str()),
                request.get("healthState").and_then(|value| value.as_str()),
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("provider binding not found"))?;
        Ok(Self::provider_binding_json(&row))
    }

    pub(crate) async fn backend_list_eval_runs(
        &self,
        context: MemoryBackendRequestContext,
        query: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let page_size = Self::page_size_from_query(&query);
        let rows = self
            .store
            .list_mem_eval_runs_for_tenant(tenant_id, page_size)
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let items = rows
            .iter()
            .map(Self::eval_run_json)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(serde_json::json!({
            "items": items,
            "pageInfo": Self::admin_page_info(page_size),
        }))
    }

    pub(crate) async fn backend_create_eval_run(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let eval_run_id = self.next_id()?.to_string();
        let metrics_json = request
            .get("metrics")
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("eval metrics encode failed: {error}"))
            })?;
        self.store
            .insert_mem_eval_run(
                tenant_id,
                &eval_run_id,
                request
                    .get("evalType")
                    .and_then(|value| value.as_str())
                    .unwrap_or("retrieval"),
                "completed",
                metrics_json.as_deref(),
            )
            .await
            .map_err(OpenMemoryService::map_store_error)?;
        let row = self
            .store
            .retrieve_mem_eval_run_for_tenant(tenant_id, &eval_run_id)
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("created eval run could not be loaded"))?;
        Self::eval_run_json(&row)
    }

    pub(crate) async fn backend_retrieve_eval_run(
        &self,
        context: MemoryBackendRequestContext,
        eval_run_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let row = self
            .store
            .retrieve_mem_eval_run_for_tenant(tenant_id, &eval_run_id.to_string())
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("eval run not found"))?;
        Self::eval_run_json(&row)
    }

    pub(crate) async fn backend_create_extraction_job(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let space_id = request
            .get("spaceId")
            .and_then(|value| value.as_str())
            .and_then(|value| value.parse().ok())
            .unwrap_or(1_u64);
        let input_events = request
            .get("inputEvents")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|value| {
                        value
                            .as_str()
                            .and_then(|text| text.parse().ok())
                            .or_else(|| value.as_u64())
                    })
                    .collect::<Vec<u64>>()
            })
            .unwrap_or_default();
        let job = MemoryOpenApi::create_extraction(
            self,
            Self::to_open_context_backend(&context),
            MemoryExtractionRequest {
                space_id,
                input_events,
                extraction_mode: request
                    .get("extractionMode")
                    .and_then(|value| value.as_str())
                    .map(str::to_string),
            },
        )
        .await?;
        let entity = serde_json::to_value(&job).map_err(|error| {
            MemoryServiceError::storage(format!("extraction job encode failed: {error}"))
        })?;
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        self.save_governance_entity(
            tenant_id,
            RT_EXTRACTION_JOB,
            &job.job_id.to_string(),
            &entity,
        )
        .await?;
        Ok(entity)
    }

    pub(crate) async fn backend_retrieve_extraction_job(
        &self,
        context: MemoryBackendRequestContext,
        job_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        self.load_governance_entity(tenant_id, RT_EXTRACTION_JOB, &job_id.to_string())
            .await
    }

    pub(crate) async fn backend_create_consolidation_job(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let job_id = self.next_id()?;
        let timestamp = platform::current_timestamp();
        let entity = serde_json::json!({
            "jobId": job_id.to_string(),
            "jobType": "consolidation",
            "state": "completed",
            "request": request,
            "createdAt": timestamp,
            "updatedAt": timestamp,
        });
        self.persist_governance_job(
            tenant_id,
            job_id,
            RT_CONSOLIDATION_JOB,
            "consolidationJobs.create",
            &entity,
        )
        .await?;
        Ok(entity)
    }

    pub(crate) async fn backend_create_retention_job(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let job_id = self.next_id()?;
        let timestamp = platform::current_timestamp();
        let entity = serde_json::json!({
            "jobId": job_id.to_string(),
            "jobType": "retention",
            "state": "accepted",
            "request": request,
            "createdAt": timestamp,
            "updatedAt": timestamp,
        });
        self.persist_governance_job(
            tenant_id,
            job_id,
            RT_RETENTION_JOB,
            "retentionJobs.create",
            &entity,
        )
        .await?;
        Ok(entity)
    }

    pub(crate) async fn backend_create_migration_job(
        &self,
        context: MemoryBackendRequestContext,
        request: serde_json::Value,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let job_id = self.next_id()?;
        let timestamp = platform::current_timestamp();
        let entity = serde_json::json!({
            "jobId": job_id.to_string(),
            "jobType": "migration",
            "state": "accepted",
            "request": request,
            "createdAt": timestamp,
            "updatedAt": timestamp,
        });
        self.persist_governance_job(
            tenant_id,
            job_id,
            RT_MIGRATION_JOB,
            "migrationJobs.create",
            &entity,
        )
        .await?;
        Ok(entity)
    }

    pub(crate) async fn backend_retrieve_migration_job(
        &self,
        context: MemoryBackendRequestContext,
        job_id: u64,
    ) -> MemoryServiceResult<serde_json::Value> {
        let tenant_id = i64::try_from(context.tenant_id).unwrap_or(i64::MAX);
        let entity: serde_json::Value = self
            .load_governance_job(tenant_id, job_id, RT_MIGRATION_JOB)
            .await?;
        Ok(entity)
    }

    pub(crate) async fn backend_supersede_memory(
        &self,
        context: MemoryBackendRequestContext,
        memory_id: u64,
        request: serde_json::Value,
    ) -> MemoryServiceResult<sdkwork_memory_contract::MemoryRecord> {
        let canonical_text = request
            .get("canonicalText")
            .and_then(|value| value.as_str())
            .ok_or_else(|| MemoryServiceError::validation("canonicalText is required"))?;
        MemoryOpenApi::update_memory(
            self,
            Self::to_open_context_backend(&context),
            memory_id,
            MemoryRecordPatch {
                canonical_text: Some(canonical_text.to_string()),
                subject: request
                    .get("subject")
                    .and_then(|value| value.as_str())
                    .map(str::to_string),
                summary_text: None,
                metadata: None,
            },
        )
        .await
    }
}

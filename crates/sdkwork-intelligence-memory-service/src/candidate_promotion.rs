use sdkwork_memory_contract::{MemoryCandidate, MemoryServiceError, MemoryServiceResult};
use sdkwork_memory_plugin_native_sql::NativeSqlCandidateDetailRow;
use sdkwork_memory_spi::{ApproveMemoryCandidateCommand, MemoryScopeContext};
use sdkwork_utils_rust::is_blank;
use serde_json::Value;

use crate::open_api::OpenMemoryService;

pub(crate) fn parse_evidence_event_ids(evidence_json: Option<&str>) -> Vec<String> {
    let Some(raw) = evidence_json.filter(|value| !is_blank(Some(value))) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };

    if let Some(items) = value.as_array() {
        return items
            .iter()
            .filter_map(|item| {
                item.as_str().and_then(|entry| {
                    entry
                        .strip_prefix("event:")
                        .map(|event_id| event_id.to_string())
                })
            })
            .collect();
    }

    if let Some(event_id) = value
        .get("eventId")
        .or_else(|| value.get("event_id"))
        .and_then(|item| item.as_str())
    {
        return vec![event_id.to_string()];
    }

    Vec::new()
}

impl OpenMemoryService {
    pub(crate) async fn approve_candidate_with_promotion(
        &self,
        tenant_id: i64,
        scope: MemoryScopeContext,
        candidate_id: u64,
        decided_by: Option<u64>,
    ) -> MemoryServiceResult<MemoryCandidate> {
        let detail = self
            .store
            .retrieve_candidate_detail_for_tenant(tenant_id, &candidate_id.to_string())
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("candidate not found"))?;

        if detail.decision_state == "approved" {
            return Self::map_candidate_detail(detail);
        }

        let memory_uuid = if let Some(memory_uuid) = detail.target_memory_uuid.clone() {
            memory_uuid
        } else {
            let memory_uuid = self.next_id()?.to_string();
            self.store
                .create_record_open_api(
                    &scope,
                    &memory_uuid,
                    "user",
                    &detail.memory_type,
                    None,
                    None,
                    &detail.proposed_text,
                    &detail.proposed_text,
                )
                .await
                .map_err(OpenMemoryService::map_store_error)?;

            for event_id in parse_evidence_event_ids(detail.evidence_json.as_deref()) {
                let source_id = self.next_id()?.to_string();
                let _ = self
                    .store
                    .append_record_source_for_tenant(
                        tenant_id,
                        &source_id,
                        &memory_uuid,
                        &event_id,
                        "evidence",
                        Some(detail.confidence),
                    )
                    .await;
            }

            self.store
                .set_candidate_target_memory_for_tenant(
                    tenant_id,
                    &candidate_id.to_string(),
                    &memory_uuid,
                )
                .await
                .map_err(OpenMemoryService::map_store_error)?;

            memory_uuid
        };

        self.store
            .approve_candidate(&ApproveMemoryCandidateCommand {
                scope,
                candidate_id: candidate_id.to_string(),
                decision_reason: None,
                decided_by: decided_by.map(|value| value as i64),
            })
            .await
            .map_err(OpenMemoryService::map_store_error)?;

        let refreshed = self
            .store
            .retrieve_candidate_detail_for_tenant(tenant_id, &candidate_id.to_string())
            .await
            .map_err(OpenMemoryService::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("candidate not found"))?;

        if refreshed.target_memory_uuid.as_deref() != Some(memory_uuid.as_str()) {
            return Err(MemoryServiceError::storage(
                "approved candidate did not retain promoted memory",
            ));
        }

        Self::map_candidate_detail(refreshed)
    }

    fn map_candidate_detail(
        row: NativeSqlCandidateDetailRow,
    ) -> MemoryServiceResult<MemoryCandidate> {
        Ok(MemoryCandidate {
            candidate_id: row.candidate_id.parse().unwrap_or(0),
            space_id: u64::try_from(row.space_id.max(0)).unwrap_or(0),
            candidate_type: row.candidate_type,
            memory_type: OpenMemoryService::memory_type_from_db(&row.memory_type),
            proposed_text: row.proposed_text,
            confidence: row.confidence,
            decision_state: row.decision_state,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

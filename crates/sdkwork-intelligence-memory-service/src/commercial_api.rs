//! Commercial memory management service layer.
//!
//! Implements subject, binding, and capability management with ID generation,
//! validation, and access control. All operations require backend-level
//! access (elevated tenant access).

use sdkwork_memory_contract::{
    BindingKind, CapabilityMode, CapabilityTargetType, CreateBindingCommand,
    CreateCapabilityBindingCommand, CreateEdgeCommand, CreateEntityCommand,
    CreatePolicyAssignmentCommand, CreatePolicyCommand, CreateSubjectCommand,
    ListBindingsQuery, ListCapabilityBindingsQuery, ListEdgesQuery, ListEntitiesQuery,
    ListPoliciesQuery, ListPolicyAssignmentsQuery, ListSubjectsQuery, MemoryBinding,
    MemoryBindingList, MemoryCapabilityBinding, MemoryCapabilityBindingList,
    MemoryCommercialReadiness, MemoryEdge, MemoryEdgeList, MemoryEntity, MemoryEntityList,
    MemoryPolicy, MemoryPolicyAssignment, MemoryPolicyAssignmentList, MemoryPolicyList,
    MemoryServiceError, MemoryServiceResult, MemorySubject, MemorySubjectList,
    PolicyAssignmentTargetType, PolicyInheritanceMode, RebuildCommercialReadinessCommand,
    ResolvedCapability, SubjectType, UpdateEdgeCommand, UpdateEntityCommand,
    UpdatePolicyAssignmentCommand, UpdatePolicyCommand, UpdateSubjectCommand,
};
use sdkwork_memory_plugin_native_sql::{
    InsertEdgeCommand as StoreInsertEdgeCommand, InsertEntityCommand as StoreInsertEntityCommand,
    InsertPolicyAssignmentCommand as StoreInsertPolicyAssignmentCommand,
    InsertPolicyCommand as StoreInsertPolicyCommand,
    InsertCommercialReadinessCommand, InsertSubjectCommand, NativeSqlBindingRow,
    NativeSqlCapabilityBindingRow, NativeSqlCommercialReadinessRow, NativeSqlEdgeRow,
    NativeSqlEntityRow, NativeSqlPolicyAssignmentRow, NativeSqlPolicyRow, NativeSqlSubjectRow,
    UpdateEdgeCommand as StoreUpdateEdgeCommand, UpdateEntityCommand as StoreUpdateEntityCommand,
    UpdatePolicyAssignmentCommand as StoreUpdatePolicyAssignmentCommand,
    UpdatePolicyCommand as StoreUpdatePolicyCommand,
    UpdateSubjectCommand as StoreUpdateSubjectCommand,
};

use crate::platform;

impl super::open_api::OpenMemoryService {
    // -----------------------------------------------------------------------
    // Subject management
    // -----------------------------------------------------------------------

    pub async fn create_subject(
        &self,
        cmd: CreateSubjectCommand,
    ) -> MemoryServiceResult<MemorySubject> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();
        let subject_type = subject_type_str(cmd.subject_type);
        let metadata_json = cmd
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| MemoryServiceError::storage(format!("metadata serialization failed: {error}")))?;

        self.store
            .insert_subject(InsertSubjectCommand {
                id: id as i64,
                uuid: &uuid,
                tenant_id,
                organization_id: cmd.organization_id.map(|v| v as i64),
                subject_type,
                subject_ref: &cmd.subject_ref,
                display_name: &cmd.display_name,
                default_space_id: cmd.default_space_id.map(|v| v as i64),
                metadata_json: metadata_json.as_deref(),
            })
            .await
            .map_err(Self::map_store_error)?;

        let row = self
            .store
            .retrieve_subject(tenant_id, &uuid)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("subject not found after insert"))?;

        Ok(map_subject_row_to_dto(row))
    }

    pub async fn retrieve_subject(
        &self,
        tenant_id: u64,
        subject_id: &str,
    ) -> MemoryServiceResult<MemorySubject> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_subject(tenant_id_i64, subject_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("subject not found"))?;
        Ok(map_subject_row_to_dto(row))
    }

    pub async fn list_subjects(
        &self,
        query: ListSubjectsQuery,
    ) -> MemoryServiceResult<MemorySubjectList> {
        let tenant_id = platform::tenant_id_i64(query.tenant_id)?;
        let page_size = platform::clamp_page_size(query.page_size);
        let subject_type = query.subject_type.map(subject_type_str);
        let rows = self
            .store
            .list_subjects(
                tenant_id,
                subject_type,
                query.status.as_deref(),
                query.cursor.as_deref(),
                page_size,
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() as i32 > page_size;
        let items: Vec<_> = rows
            .into_iter()
            .take(page_size as usize)
            .map(map_subject_row_to_dto)
            .collect();
        let next_cursor = if has_more {
            items.last().map(|item| item.subject_id.clone())
        } else {
            None
        };

        Ok(MemorySubjectList {
            items,
            page_info: platform::memory_cursor_page_info(page_size, has_more, next_cursor),
        })
    }

    pub async fn update_subject(
        &self,
        tenant_id: u64,
        subject_id: &str,
        cmd: UpdateSubjectCommand,
    ) -> MemoryServiceResult<MemorySubject> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let metadata_json = cmd
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| MemoryServiceError::storage(format!("metadata serialization failed: {error}")))?;

        let updated = self
            .store
            .update_subject(
                tenant_id_i64,
                subject_id,
                StoreUpdateSubjectCommand {
                    display_name: cmd.display_name.as_deref(),
                    default_space_id: Some(cmd.default_space_id.map(|v| v as i64)),
                    status: cmd.status.as_deref(),
                    metadata_json: metadata_json.as_deref(),
                },
            )
            .await
            .map_err(Self::map_store_error)?;

        if !updated {
            return Err(MemoryServiceError::not_found("subject not found"));
        }

        let row = self
            .store
            .retrieve_subject(tenant_id_i64, subject_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("subject not found after update"))?;

        Ok(map_subject_row_to_dto(row))
    }

    pub async fn delete_subject(
        &self,
        tenant_id: u64,
        subject_id: &str,
    ) -> MemoryServiceResult<()> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let deleted = self
            .store
            .delete_subject(tenant_id_i64, subject_id)
            .await
            .map_err(Self::map_store_error)?;

        if !deleted {
            return Err(MemoryServiceError::not_found("subject not found"));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Binding management
    // -----------------------------------------------------------------------

    pub async fn create_binding(
        &self,
        cmd: CreateBindingCommand,
    ) -> MemoryServiceResult<MemoryBinding> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();
        let binding_kind = binding_kind_str(cmd.binding_kind);
        let capability_codes_json = cmd
            .capability_codes
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| MemoryServiceError::storage(format!("capability_codes serialization failed: {error}")))?;
        let metadata_json = cmd
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| MemoryServiceError::storage(format!("metadata serialization failed: {error}")))?;

        self.store
            .insert_binding(
                id as i64,
                &uuid,
                tenant_id,
                cmd.space_id.map(|v| v as i64),
                binding_kind,
                &cmd.binding_role,
                cmd.source_subject_id.map(|v| v as i64),
                cmd.target_subject_id.map(|v| v as i64),
                cmd.target_space_id.map(|v| v as i64),
                capability_codes_json.as_deref(),
                cmd.valid_from.as_deref(),
                cmd.valid_to.as_deref(),
                metadata_json.as_deref(),
            )
            .await
            .map_err(Self::map_store_error)?;

        let row = self
            .store
            .retrieve_binding(tenant_id, &uuid)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::storage("binding not found after insert"))?;

        Ok(map_binding_row_to_dto(row))
    }

    pub async fn retrieve_binding(
        &self,
        tenant_id: u64,
        binding_id: &str,
    ) -> MemoryServiceResult<MemoryBinding> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_binding(tenant_id_i64, binding_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("binding not found"))?;
        Ok(map_binding_row_to_dto(row))
    }

    pub async fn list_bindings(
        &self,
        query: ListBindingsQuery,
    ) -> MemoryServiceResult<MemoryBindingList> {
        let tenant_id = platform::tenant_id_i64(query.tenant_id)?;
        let page_size = platform::clamp_page_size(query.page_size);
        let binding_kind = query.binding_kind.map(binding_kind_str);
        let rows = self
            .store
            .list_bindings(
                tenant_id,
                query.source_subject_id.map(|v| v as i64),
                query.target_subject_id.map(|v| v as i64),
                query.target_space_id.map(|v| v as i64),
                binding_kind,
                query.status.as_deref(),
                query.cursor.as_deref(),
                page_size,
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() as i32 > page_size;
        let items: Vec<_> = rows
            .into_iter()
            .take(page_size as usize)
            .map(map_binding_row_to_dto)
            .collect();
        let next_cursor = if has_more {
            items.last().map(|item| item.binding_id.clone())
        } else {
            None
        };

        Ok(MemoryBindingList {
            items,
            page_info: platform::memory_cursor_page_info(page_size, has_more, next_cursor),
        })
    }

    pub async fn delete_binding(
        &self,
        tenant_id: u64,
        binding_id: &str,
    ) -> MemoryServiceResult<()> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let deleted = self
            .store
            .delete_binding(tenant_id_i64, binding_id)
            .await
            .map_err(Self::map_store_error)?;

        if !deleted {
            return Err(MemoryServiceError::not_found("binding not found"));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Capability binding management
    // -----------------------------------------------------------------------

    pub async fn create_capability_binding(
        &self,
        cmd: CreateCapabilityBindingCommand,
    ) -> MemoryServiceResult<MemoryCapabilityBinding> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();
        let target_type = target_type_str(cmd.target_type);
        let mode = mode_str(cmd.mode);
        let metadata_json = cmd
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| MemoryServiceError::storage(format!("metadata serialization failed: {error}")))?;

        self.store
            .insert_capability_binding(
                id as i64,
                &uuid,
                tenant_id,
                &cmd.capability_code,
                target_type,
                cmd.target_id as i64,
                mode,
                cmd.priority,
                cmd.valid_from.as_deref(),
                cmd.valid_to.as_deref(),
                metadata_json.as_deref(),
            )
            .await
            .map_err(Self::map_store_error)?;

        let row = self
            .store
            .retrieve_capability_binding(tenant_id, &uuid)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| {
                MemoryServiceError::storage("capability binding not found after insert")
            })?;

        Ok(map_capability_binding_row_to_dto(row))
    }

    pub async fn retrieve_capability_binding(
        &self,
        tenant_id: u64,
        cap_id: &str,
    ) -> MemoryServiceResult<MemoryCapabilityBinding> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_capability_binding(tenant_id_i64, cap_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("capability binding not found"))?;
        Ok(map_capability_binding_row_to_dto(row))
    }

    pub async fn list_capability_bindings(
        &self,
        query: ListCapabilityBindingsQuery,
    ) -> MemoryServiceResult<MemoryCapabilityBindingList> {
        let tenant_id = platform::tenant_id_i64(query.tenant_id)?;
        let page_size = platform::clamp_page_size(query.page_size);
        let target_type = query.target_type.map(target_type_str);
        let rows = self
            .store
            .list_capability_bindings(
                tenant_id,
                query.capability_code.as_deref(),
                target_type,
                query.target_id.map(|v| v as i64),
                query.status.as_deref(),
                query.cursor.as_deref(),
                page_size,
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() as i32 > page_size;
        let items: Vec<_> = rows
            .into_iter()
            .take(page_size as usize)
            .map(map_capability_binding_row_to_dto)
            .collect();
        let next_cursor = if has_more {
            items.last().map(|item| item.capability_binding_id.clone())
        } else {
            None
        };

        Ok(MemoryCapabilityBindingList {
            items,
            page_info: platform::memory_cursor_page_info(page_size, has_more, next_cursor),
        })
    }

    pub async fn delete_capability_binding(
        &self,
        tenant_id: u64,
        cap_id: &str,
    ) -> MemoryServiceResult<()> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let deleted = self
            .store
            .delete_capability_binding(tenant_id_i64, cap_id)
            .await
            .map_err(Self::map_store_error)?;

        if !deleted {
            return Err(MemoryServiceError::not_found("capability binding not found"));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Capability resolution
    // -----------------------------------------------------------------------

    pub async fn resolve_capabilities(
        &self,
        tenant_id: u64,
        target_type: CapabilityTargetType,
        target_id: u64,
    ) -> MemoryServiceResult<Vec<ResolvedCapability>> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let target_type_str = target_type_str(target_type);
        let rows = self
            .store
            .resolve_capabilities_for_target(tenant_id_i64, target_type_str, target_id as i64)
            .await
            .map_err(Self::map_store_error)?;

        Ok(rows
            .into_iter()
            .map(|row| ResolvedCapability {
                capability_code: row.capability_code,
                mode: parse_mode(&row.mode),
                priority: row.priority,
                source: row.uuid,
            })
            .collect())
    }

    // -----------------------------------------------------------------------
    // Entity management
    // -----------------------------------------------------------------------

    pub async fn create_entity(
        &self,
        cmd: CreateEntityCommand,
    ) -> MemoryServiceResult<MemoryEntity> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        let space_id = platform::tenant_id_i64(cmd.space_id)?;
        if cmd.entity_type.trim().is_empty() || cmd.canonical_name.trim().is_empty() {
            return Err(MemoryServiceError::validation("entityType and canonicalName are required"));
        }
        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();
        let aliases_json = optional_json_array(cmd.aliases)?;
        let attributes_json = optional_json_value(cmd.attributes)?;

        self.store
            .insert_entity(StoreInsertEntityCommand {
                id: id as i64,
                uuid: &uuid,
                tenant_id,
                space_id,
                entity_type: &cmd.entity_type,
                canonical_name: &cmd.canonical_name,
                aliases_json: aliases_json.as_deref(),
                attributes_json: attributes_json.as_deref(),
                sensitivity_level: &cmd.sensitivity_level,
            })
            .await
            .map_err(Self::map_store_error)?;

        self.retrieve_entity(tenant_id as u64, &uuid).await
    }

    pub async fn retrieve_entity(
        &self,
        tenant_id: u64,
        entity_id: &str,
    ) -> MemoryServiceResult<MemoryEntity> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_entity(tenant_id_i64, entity_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("entity not found"))?;
        Ok(map_entity_row_to_dto(row))
    }

    pub async fn list_entities(
        &self,
        query: ListEntitiesQuery,
    ) -> MemoryServiceResult<MemoryEntityList> {
        let tenant_id = platform::tenant_id_i64(query.tenant_id)?;
        let page_size = platform::clamp_page_size(query.page_size);
        let rows = self
            .store
            .list_entities(
                tenant_id,
                query.space_id.map(|value| value as i64),
                query.entity_type.as_deref(),
                query.status.as_deref(),
                query.cursor.as_deref(),
                page_size,
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() as i32 > page_size;
        let items: Vec<_> = rows
            .into_iter()
            .take(page_size as usize)
            .map(map_entity_row_to_dto)
            .collect();
        let next_cursor = if has_more {
            items.last().map(|item| item.entity_id.clone())
        } else {
            None
        };

        Ok(MemoryEntityList {
            items,
            page_info: platform::memory_cursor_page_info(page_size, has_more, next_cursor),
        })
    }

    pub async fn update_entity(
        &self,
        tenant_id: u64,
        entity_id: &str,
        cmd: UpdateEntityCommand,
    ) -> MemoryServiceResult<MemoryEntity> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let aliases_json = optional_json_array(cmd.aliases)?;
        let attributes_json = optional_json_value(cmd.attributes)?;

        let updated = self
            .store
            .update_entity(
                tenant_id_i64,
                entity_id,
                StoreUpdateEntityCommand {
                    canonical_name: cmd.canonical_name.as_deref(),
                    aliases_json: aliases_json.as_deref(),
                    attributes_json: attributes_json.as_deref(),
                    sensitivity_level: cmd.sensitivity_level.as_deref(),
                    status: cmd.status.as_deref(),
                },
            )
            .await
            .map_err(Self::map_store_error)?;

        if !updated {
            return Err(MemoryServiceError::not_found("entity not found"));
        }

        self.retrieve_entity(tenant_id, entity_id).await
    }

    // -----------------------------------------------------------------------
    // Edge management
    // -----------------------------------------------------------------------

    pub async fn create_edge(
        &self,
        cmd: CreateEdgeCommand,
    ) -> MemoryServiceResult<MemoryEdge> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        let space_id = platform::tenant_id_i64(cmd.space_id)?;
        if cmd.relation_type.trim().is_empty() {
            return Err(MemoryServiceError::validation("relationType is required"));
        }

        let source_entity_id = self
            .store
            .resolve_entity_internal_id(tenant_id, &cmd.source_entity_id)
            .await
            .map_err(Self::map_store_error)?;
        let target_entity_id = self
            .store
            .resolve_entity_internal_id(tenant_id, &cmd.target_entity_id)
            .await
            .map_err(Self::map_store_error)?;

        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();
        let metadata_json = optional_json_value(cmd.metadata)?;

        self.store
            .insert_edge(StoreInsertEdgeCommand {
                id: id as i64,
                uuid: &uuid,
                tenant_id,
                space_id,
                source_entity_id,
                target_entity_id,
                relation_type: &cmd.relation_type,
                weight: cmd.weight,
                valid_from: cmd.valid_from.as_deref(),
                valid_to: cmd.valid_to.as_deref(),
                metadata_json: metadata_json.as_deref(),
            })
            .await
            .map_err(Self::map_store_error)?;

        self.retrieve_edge(tenant_id as u64, &uuid).await
    }

    pub async fn retrieve_edge(
        &self,
        tenant_id: u64,
        edge_id: &str,
    ) -> MemoryServiceResult<MemoryEdge> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_edge(tenant_id_i64, edge_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("edge not found"))?;
        Ok(map_edge_row_to_dto(row))
    }

    pub async fn list_edges(
        &self,
        query: ListEdgesQuery,
    ) -> MemoryServiceResult<MemoryEdgeList> {
        let tenant_id = platform::tenant_id_i64(query.tenant_id)?;
        let page_size = platform::clamp_page_size(query.page_size);
        let rows = self
            .store
            .list_edges(
                tenant_id,
                query.space_id.map(|value| value as i64),
                query.relation_type.as_deref(),
                query.source_entity_id.as_deref(),
                query.cursor.as_deref(),
                page_size,
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() as i32 > page_size;
        let items: Vec<_> = rows
            .into_iter()
            .take(page_size as usize)
            .map(map_edge_row_to_dto)
            .collect();
        let next_cursor = if has_more {
            items.last().map(|item| item.edge_id.clone())
        } else {
            None
        };

        Ok(MemoryEdgeList {
            items,
            page_info: platform::memory_cursor_page_info(page_size, has_more, next_cursor),
        })
    }

    pub async fn update_edge(
        &self,
        tenant_id: u64,
        edge_id: &str,
        cmd: UpdateEdgeCommand,
    ) -> MemoryServiceResult<MemoryEdge> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let metadata_json = optional_json_value(cmd.metadata)?;

        let updated = self
            .store
            .update_edge(
                tenant_id_i64,
                edge_id,
                StoreUpdateEdgeCommand {
                    relation_type: cmd.relation_type.as_deref(),
                    weight: cmd.weight,
                    status: cmd.status.as_deref(),
                    valid_from: cmd.valid_from.as_deref(),
                    valid_to: cmd.valid_to.as_deref(),
                    metadata_json: metadata_json.as_deref(),
                },
            )
            .await
            .map_err(Self::map_store_error)?;

        if !updated {
            return Err(MemoryServiceError::not_found("edge not found"));
        }

        self.retrieve_edge(tenant_id, edge_id).await
    }

    pub async fn delete_edge(&self, tenant_id: u64, edge_id: &str) -> MemoryServiceResult<()> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let deleted = self
            .store
            .delete_edge(tenant_id_i64, edge_id)
            .await
            .map_err(Self::map_store_error)?;
        if !deleted {
            return Err(MemoryServiceError::not_found("edge not found"));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Policy management (backend)
    // -----------------------------------------------------------------------

    pub async fn create_policy(
        &self,
        cmd: CreatePolicyCommand,
    ) -> MemoryServiceResult<MemoryPolicy> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        if cmd.policy_type.trim().is_empty() || cmd.scope.trim().is_empty() {
            return Err(MemoryServiceError::validation("policyType and scope are required"));
        }
        let policy_json = serde_json::to_string(&cmd.policy).map_err(|error| {
            MemoryServiceError::storage(format!("policy serialization failed: {error}"))
        })?;
        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();

        self.store
            .insert_policy(StoreInsertPolicyCommand {
                id: id as i64,
                uuid: &uuid,
                tenant_id,
                policy_type: &cmd.policy_type,
                scope: &cmd.scope,
                scope_ref: cmd.scope_ref.as_deref(),
                policy_json: &policy_json,
            })
            .await
            .map_err(Self::map_store_error)?;

        self.retrieve_policy(tenant_id as u64, &uuid).await
    }

    pub async fn retrieve_policy(
        &self,
        tenant_id: u64,
        policy_id: &str,
    ) -> MemoryServiceResult<MemoryPolicy> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_policy(tenant_id_i64, policy_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("policy not found"))?;
        Ok(map_policy_row_to_dto(row))
    }

    pub async fn list_policies(
        &self,
        query: ListPoliciesQuery,
    ) -> MemoryServiceResult<MemoryPolicyList> {
        let tenant_id = platform::tenant_id_i64(query.tenant_id)?;
        let page_size = platform::clamp_page_size(query.page_size);
        let rows = self
            .store
            .list_policies(
                tenant_id,
                query.policy_type.as_deref(),
                query.scope.as_deref(),
                query.cursor.as_deref(),
                page_size,
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() as i32 > page_size;
        let items: Vec<_> = rows
            .into_iter()
            .take(page_size as usize)
            .map(map_policy_row_to_dto)
            .collect();
        let next_cursor = if has_more {
            items.last().map(|item| item.policy_id.clone())
        } else {
            None
        };

        Ok(MemoryPolicyList {
            items,
            page_info: platform::memory_cursor_page_info(page_size, has_more, next_cursor),
        })
    }

    pub async fn update_policy(
        &self,
        tenant_id: u64,
        policy_id: &str,
        cmd: UpdatePolicyCommand,
    ) -> MemoryServiceResult<MemoryPolicy> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let policy_json = cmd
            .policy
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| {
                MemoryServiceError::storage(format!("policy serialization failed: {error}"))
            })?;

        let updated = self
            .store
            .update_policy(
                tenant_id_i64,
                policy_id,
                StoreUpdatePolicyCommand {
                    policy_type: cmd.policy_type.as_deref(),
                    scope: cmd.scope.as_deref(),
                    scope_ref: cmd.scope_ref.as_deref(),
                    policy_json: policy_json.as_deref(),
                    status: cmd.status.as_deref(),
                },
            )
            .await
            .map_err(Self::map_store_error)?;

        if !updated {
            return Err(MemoryServiceError::not_found("policy not found"));
        }

        self.retrieve_policy(tenant_id, policy_id).await
    }

    pub async fn delete_policy(&self, tenant_id: u64, policy_id: &str) -> MemoryServiceResult<()> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let deleted = self
            .store
            .delete_policy(tenant_id_i64, policy_id)
            .await
            .map_err(Self::map_store_error)?;
        if !deleted {
            return Err(MemoryServiceError::not_found("policy not found"));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Policy assignment management
    // -----------------------------------------------------------------------

    pub async fn create_policy_assignment(
        &self,
        cmd: CreatePolicyAssignmentCommand,
    ) -> MemoryServiceResult<MemoryPolicyAssignment> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        let policy_id = self
            .store
            .resolve_policy_internal_id(tenant_id, &cmd.policy_id)
            .await
            .map_err(Self::map_store_error)?;
        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();

        self.store
            .insert_policy_assignment(StoreInsertPolicyAssignmentCommand {
                id: id as i64,
                uuid: &uuid,
                tenant_id,
                policy_id,
                target_type: policy_target_type_str(cmd.target_type),
                target_id: cmd.target_id as i64,
                priority: cmd.priority,
                inheritance_mode: policy_inheritance_mode_str(cmd.inheritance_mode),
                valid_from: cmd.valid_from.as_deref(),
                valid_to: cmd.valid_to.as_deref(),
            })
            .await
            .map_err(Self::map_store_error)?;

        self.retrieve_policy_assignment(tenant_id as u64, &uuid)
            .await
    }

    pub async fn retrieve_policy_assignment(
        &self,
        tenant_id: u64,
        assignment_id: &str,
    ) -> MemoryServiceResult<MemoryPolicyAssignment> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_policy_assignment(tenant_id_i64, assignment_id)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("policy assignment not found"))?;
        Ok(map_policy_assignment_row_to_dto(row))
    }

    pub async fn list_policy_assignments(
        &self,
        query: ListPolicyAssignmentsQuery,
    ) -> MemoryServiceResult<MemoryPolicyAssignmentList> {
        let tenant_id = platform::tenant_id_i64(query.tenant_id)?;
        let page_size = platform::clamp_page_size(query.page_size);
        let target_type = query.target_type.map(policy_target_type_str);
        let rows = self
            .store
            .list_policy_assignments(
                tenant_id,
                target_type,
                query.target_id.map(|value| value as i64),
                query.policy_id.as_deref(),
                query.cursor.as_deref(),
                page_size,
            )
            .await
            .map_err(Self::map_store_error)?;

        let has_more = rows.len() as i32 > page_size;
        let items: Vec<_> = rows
            .into_iter()
            .take(page_size as usize)
            .map(map_policy_assignment_row_to_dto)
            .collect();
        let next_cursor = if has_more {
            items
                .last()
                .map(|item| item.policy_assignment_id.clone())
        } else {
            None
        };

        Ok(MemoryPolicyAssignmentList {
            items,
            page_info: platform::memory_cursor_page_info(page_size, has_more, next_cursor),
        })
    }

    pub async fn update_policy_assignment(
        &self,
        tenant_id: u64,
        assignment_id: &str,
        cmd: UpdatePolicyAssignmentCommand,
    ) -> MemoryServiceResult<MemoryPolicyAssignment> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let updated = self
            .store
            .update_policy_assignment(
                tenant_id_i64,
                assignment_id,
                StoreUpdatePolicyAssignmentCommand {
                    priority: cmd.priority,
                    inheritance_mode: cmd
                        .inheritance_mode
                        .map(policy_inheritance_mode_str),
                    status: cmd.status.as_deref(),
                    valid_from: cmd.valid_from.as_deref(),
                    valid_to: cmd.valid_to.as_deref(),
                },
            )
            .await
            .map_err(Self::map_store_error)?;

        if !updated {
            return Err(MemoryServiceError::not_found("policy assignment not found"));
        }

        self.retrieve_policy_assignment(tenant_id, assignment_id)
            .await
    }

    pub async fn delete_policy_assignment(
        &self,
        tenant_id: u64,
        assignment_id: &str,
    ) -> MemoryServiceResult<()> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let deleted = self
            .store
            .delete_policy_assignment(tenant_id_i64, assignment_id)
            .await
            .map_err(Self::map_store_error)?;
        if !deleted {
            return Err(MemoryServiceError::not_found("policy assignment not found"));
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Commercial readiness
    // -----------------------------------------------------------------------

    pub async fn retrieve_commercial_readiness(
        &self,
        tenant_id: u64,
    ) -> MemoryServiceResult<MemoryCommercialReadiness> {
        let tenant_id_i64 = platform::tenant_id_i64(tenant_id)?;
        let row = self
            .store
            .retrieve_latest_commercial_readiness(tenant_id_i64)
            .await
            .map_err(Self::map_store_error)?
            .ok_or_else(|| MemoryServiceError::not_found("commercial readiness not found"))?;
        Ok(map_readiness_row_to_dto(row))
    }

    pub async fn rebuild_commercial_readiness(
        &self,
        cmd: RebuildCommercialReadinessCommand,
    ) -> MemoryServiceResult<MemoryCommercialReadiness> {
        let tenant_id = platform::tenant_id_i64(cmd.tenant_id)?;
        let subject_count = self
            .store
            .count_subjects_for_tenant(tenant_id)
            .await
            .map_err(Self::map_store_error)?;
        let binding_count = self
            .store
            .count_bindings_for_tenant(tenant_id)
            .await
            .map_err(Self::map_store_error)?;
        let entity_count = self
            .store
            .count_entities_for_tenant(tenant_id)
            .await
            .map_err(Self::map_store_error)?;
        let edge_count = self
            .store
            .count_edges_for_tenant(tenant_id)
            .await
            .map_err(Self::map_store_error)?;
        let policy_count = self
            .store
            .count_policies_for_tenant(tenant_id)
            .await
            .map_err(Self::map_store_error)?;
        let assignment_count = self
            .store
            .count_policy_assignments_for_tenant(tenant_id)
            .await
            .map_err(Self::map_store_error)?;

        let management_coverage = serde_json::json!({
            "subjects": subject_count,
            "bindings": binding_count,
            "entities": entity_count,
            "edges": edge_count,
            "policies": policy_count,
            "policyAssignments": assignment_count,
        });
        let contract_coverage = serde_json::json!({
            "subjects": true,
            "bindings": true,
            "entities": true,
            "edges": true,
            "policies": true,
            "policyAssignments": true,
            "commercialReadiness": true,
        });

        let mut blocking_findings = Vec::new();
        let mut warning_findings = Vec::new();
        if subject_count == 0 {
            blocking_findings.push("no_subjects".to_owned());
        }
        if binding_count == 0 {
            warning_findings.push("no_bindings".to_owned());
        }

        let populated_layers = [subject_count, binding_count, entity_count, edge_count]
            .iter()
            .filter(|&&count| count > 0)
            .count() as f64;
        let data_score = populated_layers / 4.0;
        let contract_score = 1.0;
        let score = ((contract_score * 0.6) + (data_score * 0.4)).min(1.0);
        let state = if !blocking_findings.is_empty() {
            "blocked"
        } else if score >= 0.75 {
            "ready"
        } else {
            "warning"
        };

        let blocking_json = if blocking_findings.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&blocking_findings).map_err(|error| {
                MemoryServiceError::storage(format!("blocking findings serialization failed: {error}"))
            })?)
        };
        let warning_json = if warning_findings.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&warning_findings).map_err(|error| {
                MemoryServiceError::storage(format!("warning findings serialization failed: {error}"))
            })?)
        };
        let management_json = serde_json::to_string(&management_coverage).map_err(|error| {
            MemoryServiceError::storage(format!("management coverage serialization failed: {error}"))
        })?;
        let contract_json = serde_json::to_string(&contract_coverage).map_err(|error| {
            MemoryServiceError::storage(format!("contract coverage serialization failed: {error}"))
        })?;

        let id = platform::next_numeric_id()?;
        let uuid = id.to_string();
        self.store
            .delete_commercial_readiness_for_profile(
                tenant_id,
                cmd.implementation_profile_id.map(|value| value as i64),
            )
            .await
            .map_err(Self::map_store_error)?;
        self.store
            .insert_commercial_readiness_snapshot(InsertCommercialReadinessCommand {
                id: id as i64,
                uuid: &uuid,
                tenant_id,
                implementation_profile_id: cmd.implementation_profile_id.map(|value| value as i64),
                score,
                state,
                contract_coverage_json: Some(&contract_json),
                management_coverage_json: Some(&management_json),
                runtime_conformance_json: None,
                privacy_coverage_json: None,
                audit_coverage_json: None,
                sdk_coverage_json: None,
                evaluation_coverage_json: None,
                observability_coverage_json: None,
                migration_coverage_json: None,
                blocking_findings_json: blocking_json.as_deref(),
                warning_findings_json: warning_json.as_deref(),
            })
            .await
            .map_err(Self::map_store_error)?;

        self.retrieve_commercial_readiness(cmd.tenant_id).await
    }
}

// ---------------------------------------------------------------------------
// Mappers
// ---------------------------------------------------------------------------

fn map_subject_row_to_dto(row: NativeSqlSubjectRow) -> MemorySubject {
    MemorySubject {
        subject_id: row.uuid,
        tenant_id: row.tenant_id as u64,
        organization_id: row.organization_id.map(|v| v as u64),
        subject_type: parse_subject_type(&row.subject_type),
        subject_ref: row.subject_ref,
        display_name: row.display_name,
        default_space_id: row.default_space_id.map(|v| v as u64),
        status: row.status,
        metadata: row
            .metadata_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok()),
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version as u64,
    }
}

fn map_binding_row_to_dto(row: NativeSqlBindingRow) -> MemoryBinding {
    MemoryBinding {
        binding_id: row.uuid,
        tenant_id: row.tenant_id as u64,
        space_id: row.space_id.map(|v| v as u64),
        binding_kind: parse_binding_kind(&row.binding_kind),
        binding_role: row.binding_role,
        source_subject_id: row.source_subject_id.map(|v| v as u64),
        target_subject_id: row.target_subject_id.map(|v| v as u64),
        target_space_id: row.target_space_id.map(|v| v as u64),
        capability_codes: row
            .capability_codes_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok()),
        status: row.status,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        metadata: row
            .metadata_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok()),
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version as u64,
    }
}

fn map_capability_binding_row_to_dto(row: NativeSqlCapabilityBindingRow) -> MemoryCapabilityBinding {
    MemoryCapabilityBinding {
        capability_binding_id: row.uuid,
        tenant_id: row.tenant_id as u64,
        capability_code: row.capability_code,
        target_type: parse_target_type(&row.target_type),
        target_id: row.target_id as u64,
        mode: parse_mode(&row.mode),
        priority: row.priority,
        status: row.status,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        metadata: row
            .metadata_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok()),
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version as u64,
    }
}

fn optional_json_value(value: Option<serde_json::Value>) -> MemoryServiceResult<Option<String>> {
    match value {
        Some(json) => serde_json::to_string(&json)
            .map(Some)
            .map_err(|error| MemoryServiceError::storage(format!("json serialization failed: {error}"))),
        None => Ok(None),
    }
}

fn optional_json_array(values: Option<Vec<String>>) -> MemoryServiceResult<Option<String>> {
    match values {
        Some(items) => serde_json::to_string(&items)
            .map(Some)
            .map_err(|error| MemoryServiceError::storage(format!("json serialization failed: {error}"))),
        None => Ok(None),
    }
}

fn map_entity_row_to_dto(row: NativeSqlEntityRow) -> MemoryEntity {
    MemoryEntity {
        entity_id: row.uuid,
        space_id: row.space_id as u64,
        entity_type: row.entity_type,
        canonical_name: row.canonical_name,
        aliases: row
            .aliases_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        attributes: row
            .attributes_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        sensitivity_level: row.sensitivity_level,
        status: row.status,
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version as u64,
    }
}

fn map_edge_row_to_dto(row: NativeSqlEdgeRow) -> MemoryEdge {
    MemoryEdge {
        edge_id: row.uuid,
        space_id: row.space_id as u64,
        source_entity_id: row.source_entity_uuid,
        target_entity_id: row.target_entity_uuid,
        relation_type: row.relation_type,
        weight: row.weight,
        status: row.status,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        metadata: row
            .metadata_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version as u64,
    }
}

fn map_policy_row_to_dto(row: NativeSqlPolicyRow) -> MemoryPolicy {
    MemoryPolicy {
        policy_id: row.uuid,
        tenant_id: row.tenant_id as u64,
        policy_type: row.policy_type,
        scope: row.scope,
        scope_ref: row.scope_ref,
        status: row.status,
        policy: serde_json::from_str(&row.policy_json).unwrap_or(serde_json::Value::Null),
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version as u64,
    }
}

fn map_policy_assignment_row_to_dto(row: NativeSqlPolicyAssignmentRow) -> MemoryPolicyAssignment {
    MemoryPolicyAssignment {
        policy_assignment_id: row.uuid,
        tenant_id: row.tenant_id as u64,
        policy_id: row.policy_uuid,
        target_type: parse_policy_target_type(&row.target_type),
        target_id: row.target_id as u64,
        priority: row.priority,
        inheritance_mode: parse_policy_inheritance_mode(&row.inheritance_mode),
        status: row.status,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        created_at: row.created_at,
        updated_at: row.updated_at,
        version: row.version as u64,
    }
}

fn map_readiness_row_to_dto(row: NativeSqlCommercialReadinessRow) -> MemoryCommercialReadiness {
    MemoryCommercialReadiness {
        readiness_id: row.uuid,
        tenant_id: row.tenant_id as u64,
        implementation_profile_id: row.implementation_profile_id.map(|value| value as u64),
        score: row.score,
        state: row.state,
        contract_coverage: row
            .contract_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        management_coverage: row
            .management_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        runtime_conformance: row
            .runtime_conformance_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        privacy_coverage: row
            .privacy_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        audit_coverage: row
            .audit_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        sdk_coverage: row
            .sdk_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        evaluation_coverage: row
            .evaluation_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        observability_coverage: row
            .observability_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        migration_coverage: row
            .migration_coverage_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        blocking_findings: row
            .blocking_findings_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        warning_findings: row
            .warning_findings_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        created_at: row.created_at,
    }
}

// ---------------------------------------------------------------------------
// Enum string conversions
// ---------------------------------------------------------------------------

fn subject_type_str(t: SubjectType) -> &'static str {
    match t {
        SubjectType::Tenant => "tenant",
        SubjectType::Organization => "organization",
        SubjectType::User => "user",
        SubjectType::Application => "application",
        SubjectType::Service => "service",
    }
}

fn parse_subject_type(s: &str) -> SubjectType {
    match s {
        "tenant" => SubjectType::Tenant,
        "organization" => SubjectType::Organization,
        "user" => SubjectType::User,
        "application" => SubjectType::Application,
        "service" => SubjectType::Service,
        _ => SubjectType::User,
    }
}

fn binding_kind_str(k: BindingKind) -> &'static str {
    match k {
        BindingKind::Ownership => "ownership",
        BindingKind::Access => "access",
        BindingKind::Share => "share",
        BindingKind::Reference => "reference",
        BindingKind::Provision => "provision",
    }
}

fn parse_binding_kind(s: &str) -> BindingKind {
    match s {
        "ownership" => BindingKind::Ownership,
        "access" => BindingKind::Access,
        "share" => BindingKind::Share,
        "reference" => BindingKind::Reference,
        "provision" => BindingKind::Provision,
        _ => BindingKind::Access,
    }
}

fn target_type_str(t: CapabilityTargetType) -> &'static str {
    match t {
        CapabilityTargetType::Subject => "subject",
        CapabilityTargetType::Space => "space",
        CapabilityTargetType::Binding => "binding",
        CapabilityTargetType::Memory => "memory",
    }
}

fn parse_target_type(s: &str) -> CapabilityTargetType {
    match s {
        "subject" => CapabilityTargetType::Subject,
        "space" => CapabilityTargetType::Space,
        "binding" => CapabilityTargetType::Binding,
        "memory" => CapabilityTargetType::Memory,
        _ => CapabilityTargetType::Subject,
    }
}

fn mode_str(m: CapabilityMode) -> &'static str {
    match m {
        CapabilityMode::Allow => "allow",
        CapabilityMode::Deny => "deny",
        CapabilityMode::Conditional => "conditional",
    }
}

fn parse_mode(s: &str) -> CapabilityMode {
    match s {
        "allow" => CapabilityMode::Allow,
        "deny" => CapabilityMode::Deny,
        "conditional" => CapabilityMode::Conditional,
        _ => CapabilityMode::Allow,
    }
}

fn policy_target_type_str(value: PolicyAssignmentTargetType) -> &'static str {
    match value {
        PolicyAssignmentTargetType::Subject => "subject",
        PolicyAssignmentTargetType::Space => "space",
        PolicyAssignmentTargetType::Entity => "entity",
        PolicyAssignmentTargetType::Binding => "binding",
        PolicyAssignmentTargetType::CapabilityBinding => "capability_binding",
        PolicyAssignmentTargetType::ImplementationProfile => "implementation_profile",
    }
}

fn parse_policy_target_type(value: &str) -> PolicyAssignmentTargetType {
    match value {
        "subject" => PolicyAssignmentTargetType::Subject,
        "space" => PolicyAssignmentTargetType::Space,
        "entity" => PolicyAssignmentTargetType::Entity,
        "binding" => PolicyAssignmentTargetType::Binding,
        "capability_binding" => PolicyAssignmentTargetType::CapabilityBinding,
        "implementation_profile" => PolicyAssignmentTargetType::ImplementationProfile,
        _ => PolicyAssignmentTargetType::Subject,
    }
}

fn policy_inheritance_mode_str(value: PolicyInheritanceMode) -> &'static str {
    match value {
        PolicyInheritanceMode::Inherit => "inherit",
        PolicyInheritanceMode::Override => "override",
        PolicyInheritanceMode::Deny => "deny",
        PolicyInheritanceMode::Shadow => "shadow",
    }
}

fn parse_policy_inheritance_mode(value: &str) -> PolicyInheritanceMode {
    match value {
        "inherit" => PolicyInheritanceMode::Inherit,
        "override" => PolicyInheritanceMode::Override,
        "deny" => PolicyInheritanceMode::Deny,
        "shadow" => PolicyInheritanceMode::Shadow,
        _ => PolicyInheritanceMode::Inherit,
    }
}

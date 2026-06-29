//! Commercial memory management service layer.
//!
//! Implements subject, binding, and capability management with ID generation,
//! validation, and access control. All operations require backend-level
//! access (elevated tenant access).

use sdkwork_memory_contract::{
    BindingKind, CapabilityMode, CapabilityTargetType, CreateBindingCommand,
    CreateCapabilityBindingCommand, CreateSubjectCommand, ListBindingsQuery, ListCapabilityBindingsQuery,
    ListSubjectsQuery, MemoryBinding, MemoryBindingList, MemoryCapabilityBinding,
    MemoryCapabilityBindingList, MemoryPageInfo, MemoryServiceError, MemoryServiceResult,
    MemorySubject, MemorySubjectList, ResolvedCapability, SubjectType, UpdateSubjectCommand,
};
use sdkwork_memory_plugin_native_sql::{
    InsertSubjectCommand, NativeSqlBindingRow, NativeSqlCapabilityBindingRow, NativeSqlSubjectRow,
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
            page_info: MemoryPageInfo {
                next_cursor,
                has_more,
                page_size: Some(page_size),
            },
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
            page_info: MemoryPageInfo {
                next_cursor,
                has_more,
                page_size: Some(page_size),
            },
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
            page_info: MemoryPageInfo {
                next_cursor,
                has_more,
                page_size: Some(page_size),
            },
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

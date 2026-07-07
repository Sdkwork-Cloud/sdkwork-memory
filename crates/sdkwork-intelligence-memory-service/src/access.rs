use sdkwork_memory_contract::{MemoryOpenApiRequestContext, MemoryServiceError, MemoryServiceResult};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

use crate::platform;
use crate::store_error::map_native_sql_store_error;

// ---------------------------------------------------------------------------
// Commercial capability codes
// ---------------------------------------------------------------------------

/// Capability code controlling memory retrieval operations.
pub const CAPABILITY_MEMORY_RETRIEVE: &str = "memory.retrieve";

/// Capability code controlling memory write operations.
pub const CAPABILITY_MEMORY_WRITE: &str = "memory.write";

/// Check whether a capability binding is currently within its validity period.
fn is_capability_valid(valid_from: &Option<String>, valid_to: &Option<String>, now: &str) -> bool {
    if let Some(from) = valid_from {
        if !sdkwork_utils_rust::is_blank(Some(from.as_str())) && now < from.as_str() {
            return false;
        }
    }
    if let Some(to) = valid_to {
        if !sdkwork_utils_rust::is_blank(Some(to.as_str())) && now > to.as_str() {
            return false;
        }
    }
    true
}

/// Resolve capabilities for a space and determine whether the given capability
/// code is explicitly denied. Deny wins over allow per commercial management
/// design §8.2. Backend operators with elevated tenant access bypass this check.
async fn is_capability_denied(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
    capability_code: &str,
) -> MemoryServiceResult<bool> {
    if context.elevated_tenant_access {
        return Ok(false);
    }

    let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
    let space_id_i64 = platform::space_id_i64(space_id)?;
    let mut cursor = None;
    loop {
        let capabilities = store
            .resolve_capabilities_for_target(
                tenant_id,
                "space",
                space_id_i64,
                sdkwork_utils_rust::MAX_LIST_PAGE_SIZE,
                cursor.as_deref(),
            )
            .await
            .map_err(map_native_sql_store_error)?;

        let page_size = usize::try_from(sdkwork_utils_rust::MAX_LIST_PAGE_SIZE).unwrap_or(200);
        let has_more = capabilities.len() > page_size;
        let now = platform::current_timestamp();
        for cap in capabilities.iter().take(page_size) {
            if cap.capability_code == capability_code
                && cap.mode == "deny"
                && is_capability_valid(&cap.valid_from, &cap.valid_to, &now)
            {
                return Ok(true);
            }
        }
        if !has_more {
            break;
        }
        cursor = capabilities
            .get(page_size.saturating_sub(1))
            .map(|cap| cap.uuid.clone());
    }
    Ok(false)
}

/// Assert that retrieval is not explicitly denied for a space via capability
/// bindings. This implements the capability resolver integration described in
/// the commercial memory management design (§8.3 Retrieval Integration).
pub async fn assert_retrieval_capability_allowed(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
) -> MemoryServiceResult<()> {
    if is_capability_denied(store, context, space_id, CAPABILITY_MEMORY_RETRIEVE).await? {
        tracing::warn!(
            tenant_id = context.tenant_id,
            space_id,
            capability_code = CAPABILITY_MEMORY_RETRIEVE,
            "retrieval denied by capability binding"
        );
        return forbidden("retrieval denied for this memory space by capability policy");
    }
    Ok(())
}

/// Assert that writing is not explicitly denied for a space via capability
/// bindings. This implements the capability resolver integration described in
/// the commercial memory management design (§8.4 Write Integration).
pub async fn assert_write_capability_allowed(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
) -> MemoryServiceResult<()> {
    if is_capability_denied(store, context, space_id, CAPABILITY_MEMORY_WRITE).await? {
        tracing::warn!(
            tenant_id = context.tenant_id,
            space_id,
            capability_code = CAPABILITY_MEMORY_WRITE,
            "write denied by capability binding"
        );
        return forbidden("write denied for this memory space by capability policy");
    }
    Ok(())
}

fn forbidden(detail: impl Into<String>) -> MemoryServiceResult<()> {
    crate::domain_metrics::memory_domain_metrics().record_authz_denied();
    Err(MemoryServiceError::forbidden(detail))
}

pub async fn assert_actor_can_access_space(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
) -> MemoryServiceResult<()> {
    assert_actor_can_access_existing_space(store, context, space_id, false).await
}

pub async fn assert_actor_can_access_space_for_write(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
) -> MemoryServiceResult<()> {
    let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
    let space_id_i64 = platform::space_id_i64(space_id)?;
    let space = store
        .retrieve_space_for_tenant(tenant_id, space_id_i64)
        .await
        .map_err(map_native_sql_store_error)?;

    if space.is_none() {
        if context.elevated_tenant_access {
            tracing::warn!(
                tenant_id = context.tenant_id,
                space_id,
                actor_id = ?context.actor_id,
                "elevated_tenant_access bypass: write to non-existent space"
            );
            return Ok(());
        }
        return forbidden(
            "memory space must be created before write operations are allowed",
        );
    }

    assert_actor_can_access_existing_space(store, context, space_id, true).await?;

    // Commercial capability check: deny write if explicitly denied for this space.
    assert_write_capability_allowed(store, context, space_id).await
}

async fn assert_actor_can_access_existing_space(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
    require_write: bool,
) -> MemoryServiceResult<()> {
    let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
    let space_id_i64 = platform::space_id_i64(space_id)?;
    let Some(space) = store
        .retrieve_space_for_tenant(tenant_id, space_id_i64)
        .await
        .map_err(map_native_sql_store_error)?
    else {
        return Err(MemoryServiceError::not_found("memory space not found"));
    };

    if space.lifecycle_status == "deleted" {
        return Err(MemoryServiceError::not_found("memory space not found"));
    }

    if context.elevated_tenant_access {
        tracing::warn!(
            tenant_id = context.tenant_id,
            space_id,
            actor_id = ?context.actor_id,
            "elevated_tenant_access bypass: space access check skipped"
        );
        return Ok(());
    }

    let Some(actor_id) = context.actor_id else {
        return forbidden("authenticated actor is required for memory space access");
    };

    let actor = actor_id.to_string();
    if space.owner_subject_type == "user" {
        if space.owner_subject_id == actor {
            return Ok(());
        }
        if store
            .actor_has_active_space_binding(tenant_id, space_id_i64, &actor, require_write)
            .await
            .map_err(map_native_sql_store_error)?
        {
            return Ok(());
        }
        return forbidden("actor is not authorized for this memory space");
    }

    if space.owner_subject_type == "agent" && space.owner_subject_id == actor {
        return Ok(());
    }

    if store
        .actor_has_active_space_binding(tenant_id, space_id_i64, &actor, require_write)
        .await
        .map_err(map_native_sql_store_error)?
    {
        return Ok(());
    }

    forbidden("actor is not authorized for this memory space")
}

pub fn actor_may_read_sensitivity(
    context: &MemoryOpenApiRequestContext,
    sensitivity_level: &str,
    actual_actor_is_space_owner: bool,
) -> bool {
    match sensitivity_level {
        "public" | "internal" => true,
        "private" | "sensitive" => {
            if context.elevated_tenant_access {
                tracing::warn!(
                    tenant_id = context.tenant_id,
                    sensitivity_level,
                    actor_id = ?context.actor_id,
                    "elevated_tenant_access: backend operator accessing private/sensitive memory"
                );
                return true;
            }
            actual_actor_is_space_owner
        }
        "restricted" => actual_actor_is_space_owner,
        _ => actual_actor_is_space_owner,
    }
}

/// Enforces read-path sensitivity policy for a single memory record.
///
/// Returns `not_found` when the actor lacks read access so existence of
/// restricted records is not leaked to unauthorized callers.
pub async fn assert_actor_may_read_record_sensitivity(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
    sensitivity_level: &str,
) -> MemoryServiceResult<()> {
    let actual_owner = actual_actor_is_space_owner(store, context, space_id).await?;
    if actor_may_read_sensitivity(context, sensitivity_level, actual_owner) {
        Ok(())
    } else {
        Err(MemoryServiceError::not_found("memory not found"))
    }
}

pub async fn actual_actor_is_space_owner(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
) -> MemoryServiceResult<bool> {
    let tenant_id = platform::tenant_id_i64(context.tenant_id)?;
    let space_id_i64 = platform::space_id_i64(space_id)?;
    let Some(space) = store
        .retrieve_space_for_tenant(tenant_id, space_id_i64)
        .await
        .map_err(map_native_sql_store_error)?
    else {
        return Ok(false);
    };
    let Some(actor_id) = context.actor_id.as_ref() else {
        return Ok(false);
    };
    Ok(space.owner_subject_id == actor_id.to_string())
}

pub fn sensitivity_read_scope(
    context: &MemoryOpenApiRequestContext,
    actual_actor_is_space_owner: bool,
) -> i32 {
    use sdkwork_memory_plugin_native_sql::{
        SENSITIVITY_READ_ELEVATED, SENSITIVITY_READ_OWNER, SENSITIVITY_READ_PUBLIC,
    };
    if actual_actor_is_space_owner {
        SENSITIVITY_READ_OWNER
    } else if context.elevated_tenant_access {
        SENSITIVITY_READ_ELEVATED
    } else {
        SENSITIVITY_READ_PUBLIC
    }
}

pub async fn assert_actor_can_access_spaces(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_ids: &[u64],
) -> MemoryServiceResult<()> {
    for space_id in space_ids {
        assert_actor_can_access_space(store, context, *space_id).await?;
    }
    Ok(())
}

pub fn require_list_space_id(space_id: Option<u64>) -> MemoryServiceResult<u64> {
    space_id.ok_or_else(|| {
        MemoryServiceError::validation("spaceId query parameter is required")
    })
}

pub fn require_commercial_list_space_id(
    context: &MemoryOpenApiRequestContext,
    space_id: Option<u64>,
) -> MemoryServiceResult<Option<u64>> {
    if context.elevated_tenant_access {
        Ok(space_id)
    } else {
        Ok(Some(require_list_space_id(space_id)?))
    }
}

pub async fn assert_actor_may_read_entity_sensitivity(
    context: &MemoryOpenApiRequestContext,
    sensitivity_level: &str,
    actual_actor_is_space_owner: bool,
) -> MemoryServiceResult<()> {
    if actor_may_read_sensitivity(context, sensitivity_level, actual_actor_is_space_owner) {
        Ok(())
    } else {
        Err(MemoryServiceError::not_found("entity not found"))
    }
}

pub async fn assert_actor_can_access_space_i64(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: i64,
) -> MemoryServiceResult<()> {
    if space_id < 0 {
        return Err(MemoryServiceError::validation("spaceId must be non-negative"));
    }
    assert_actor_can_access_space(
        store,
        context,
        u64::try_from(space_id).map_err(|_| {
            MemoryServiceError::validation("spaceId must fit in an unsigned 64-bit integer")
        })?,
    )
    .await
}

pub fn validate_user_space_owner(
    context: &MemoryOpenApiRequestContext,
    owner_subject_type: &str,
    owner_subject_id: &str,
) -> MemoryServiceResult<()> {
    if context.elevated_tenant_access {
        tracing::warn!(
            tenant_id = context.tenant_id,
            owner_subject_type,
            "elevated_tenant_access bypass: space owner validation skipped"
        );
        return Ok(());
    }
    if owner_subject_type != "user" {
        return forbidden(
            "only backend operators may create non-user-owned memory spaces",
        );
    }
    let Some(actor_id) = context.actor_id else {
        return forbidden(
            "authenticated actor is required to create a user-owned memory space",
        );
    };
    if owner_subject_id != actor_id.to_string() {
        return forbidden(
            "ownerSubjectId must match the authenticated actor for user-owned spaces",
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_memory_plugin_native_sql::NativeSqlCreateSpaceCommand;

    async fn seed_user_space(
        store: &NativeSqlMemoryStore,
        tenant_id: i64,
        space_id: i64,
        owner_id: &str,
    ) {
        store
            .create_space_record(
                tenant_id,
                space_id,
                &NativeSqlCreateSpaceCommand {
                    organization_id: None,
                    owner_subject_type: "user".to_string(),
                    owner_subject_id: owner_id.to_string(),
                    space_type: "personal".to_string(),
                    display_name: "test space".to_string(),
                    default_scope: "user".to_string(),
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn write_to_missing_space_is_denied() {
        let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        let error = assert_actor_can_access_space_for_write(&store, &context, 99)
            .await
            .expect_err("missing space write must be forbidden");
        assert_eq!(error.code, "forbidden");
    }

    #[tokio::test]
    async fn actor_can_access_owned_user_space() {
        let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
        seed_user_space(&store, 100_001, 1, "2001").await;
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        assert_actor_can_access_space(&store, &context, 1)
            .await
            .expect("owned space should be accessible");
    }

    #[tokio::test]
    async fn actor_cannot_access_foreign_user_space() {
        let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
        seed_user_space(&store, 100_001, 2, "3002").await;
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        let error = assert_actor_can_access_space(&store, &context, 2)
            .await
            .expect_err("foreign space must be forbidden");
        assert_eq!(error.code, "forbidden");
    }

    #[tokio::test]
    async fn missing_actor_is_denied_for_user_space() {
        let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
        seed_user_space(&store, 100_001, 1, "2001").await;
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, None);
        let error = assert_actor_can_access_space(&store, &context, 1)
            .await
            .expect_err("missing actor must fail closed");
        assert_eq!(error.code, "forbidden");
    }

    #[tokio::test]
    async fn backend_operator_can_access_tenant_spaces() {
        let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
        seed_user_space(&store, 100_001, 2, "3002").await;
        let context = MemoryOpenApiRequestContext::for_backend_surface(100_001, Some(9001));
        assert_actor_can_access_space(&store, &context, 2)
            .await
            .expect("backend operator should have tenant-wide access");
    }

    #[tokio::test]
    async fn actor_can_access_space_via_active_binding() {
        use sdkwork_memory_plugin_native_sql::InsertSubjectCommand;

        let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
        seed_user_space(&store, 100_001, 2, "3002").await;
        store
            .insert_subject(InsertSubjectCommand {
                id: 501,
                uuid: "501",
                tenant_id: 100_001,
                organization_id: None,
                subject_type: "user",
                subject_ref: "2001",
                display_name: "bound actor",
                default_space_id: None,
                metadata_json: None,
            })
            .await
            .unwrap();
        store
            .insert_binding(
                601,
                "601",
                100_001,
                None,
                "access",
                "viewer",
                Some(501),
                None,
                Some(2),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        assert_actor_can_access_space(&store, &context, 2)
            .await
            .expect("binding grant should allow read access");
    }

    #[tokio::test]
    async fn viewer_binding_allows_read_but_denies_write() {
        use sdkwork_memory_plugin_native_sql::InsertSubjectCommand;

        let store = NativeSqlMemoryStore::new_in_memory_sqlite().await.unwrap();
        seed_user_space(&store, 100_001, 2, "3002").await;
        store
            .insert_subject(InsertSubjectCommand {
                id: 502,
                uuid: "502",
                tenant_id: 100_001,
                organization_id: None,
                subject_type: "user",
                subject_ref: "2001",
                display_name: "viewer actor",
                default_space_id: None,
                metadata_json: None,
            })
            .await
            .unwrap();
        store
            .insert_binding(
                602,
                "602",
                100_001,
                None,
                "access",
                "viewer",
                Some(502),
                None,
                Some(2),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        assert_actor_can_access_space(&store, &context, 2)
            .await
            .expect("viewer binding should allow read");
        let error = assert_actor_can_access_space_for_write(&store, &context, 2)
            .await
            .expect_err("viewer binding must not allow write");
        assert_eq!(error.code, "forbidden");
    }

    #[test]
    fn actor_may_read_public_and_internal_without_ownership() {
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        assert!(actor_may_read_sensitivity(&context, "public", false));
        assert!(actor_may_read_sensitivity(&context, "internal", false));
    }

    #[test]
    fn actor_may_not_read_restricted_sensitivity_without_ownership() {
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        assert!(!actor_may_read_sensitivity(&context, "private", false));
        assert!(!actor_may_read_sensitivity(&context, "sensitive", false));
        assert!(!actor_may_read_sensitivity(&context, "restricted", false));
    }

    #[test]
    fn space_owner_may_read_restricted_sensitivity() {
        let context = MemoryOpenApiRequestContext::for_open_surface("key-1", 100_001, Some(2001));
        assert!(actor_may_read_sensitivity(&context, "restricted", true));
    }

    #[test]
    fn elevated_backend_can_read_private_and_sensitive() {
        let context = MemoryOpenApiRequestContext::for_backend_surface(100_001, Some(9001));
        assert!(actor_may_read_sensitivity(&context, "private", false));
        assert!(actor_may_read_sensitivity(&context, "sensitive", false));
    }

    #[test]
    fn elevated_backend_cannot_read_restricted_without_ownership() {
        let context = MemoryOpenApiRequestContext::for_backend_surface(100_001, Some(9001));
        assert!(!actor_may_read_sensitivity(&context, "restricted", false));
    }

    #[test]
    fn elevated_backend_can_read_restricted_as_space_owner() {
        let context = MemoryOpenApiRequestContext::for_backend_surface(100_001, Some(9001));
        assert!(actor_may_read_sensitivity(&context, "restricted", true));
    }
}

use sdkwork_memory_contract::{MemoryOpenApiRequestContext, MemoryServiceError, MemoryServiceResult};
use sdkwork_memory_plugin_native_sql::NativeSqlMemoryStore;

use crate::platform;
use crate::store_error::map_native_sql_store_error;

pub async fn assert_actor_can_access_space(
    store: &NativeSqlMemoryStore,
    context: &MemoryOpenApiRequestContext,
    space_id: u64,
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

    let Some(actor_id) = context.actor_id else {
        return Ok(());
    };

    if space.owner_subject_type != "user" {
        return Ok(());
    }

    let actor = actor_id.to_string();
    if space.owner_subject_id == actor {
        return Ok(());
    }

    if space.owner_subject_id.starts_with("tenant-") {
        return Ok(());
    }

    Err(MemoryServiceError::forbidden(
        "actor is not authorized for this memory space",
    ))
}

pub fn require_list_space_id(space_id: Option<u64>) -> MemoryServiceResult<u64> {
    space_id.ok_or_else(|| {
        MemoryServiceError::validation("spaceId query parameter is required")
    })
}

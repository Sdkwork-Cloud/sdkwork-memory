use std::sync::OnceLock;

use sdkwork_id_core::SnowflakeIdGenerator;
use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceResult};

pub fn tenant_id_i64(tenant_id: u64) -> MemoryServiceResult<i64> {
    i64::try_from(tenant_id)
        .map_err(|_| MemoryServiceError::validation("tenantId is out of storage range"))
}

pub fn space_id_i64(space_id: u64) -> MemoryServiceResult<i64> {
    i64::try_from(space_id)
        .map_err(|_| MemoryServiceError::validation("spaceId is out of storage range"))
}

pub fn optional_u64_as_i64(value: Option<u64>) -> MemoryServiceResult<Option<i64>> {
    value.map(space_id_i64).transpose()
}

pub fn optional_i64_as_u64(value: Option<i64>) -> Option<u64> {
    value.and_then(|id| u64::try_from(id.max(0)).ok())
}

static ID_GENERATOR: OnceLock<SnowflakeIdGenerator> = OnceLock::new();

fn resolve_snowflake_node_id() -> u16 {
    if let Ok(value) = std::env::var("SDKWORK_MEMORY_SNOWFLAKE_NODE_ID") {
        if let Ok(parsed) = value.parse::<u16>() {
            return parsed;
        }
    }

    for key in ["POD_NAME", "HOSTNAME", "COMPUTERNAME"] {
        if let Ok(name) = std::env::var(key) {
            let mut hash: u32 = 0;
            for byte in name.as_bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(u32::from(*byte));
            }
            return (hash % 1024) as u16;
        }
    }

    1
}

fn id_generator() -> &'static SnowflakeIdGenerator {
    ID_GENERATOR.get_or_init(|| {
        let node_id = resolve_snowflake_node_id();
        SnowflakeIdGenerator::new(node_id).expect("memory snowflake generator must initialize")
    })
}

pub fn next_numeric_id() -> MemoryServiceResult<u64> {
    let id = id_generator()
        .generate()
        .map_err(|error| MemoryServiceError::storage(format!("id generation failed: {error}")))?;
    u64::try_from(id)
        .map_err(|_| MemoryServiceError::storage(format!("id out of u64 range: {id}")))
}

pub fn current_timestamp() -> String {
    sdkwork_utils_rust::format_datetime(sdkwork_utils_rust::now(), None)
}

pub fn elapsed_millis_i64(started: std::time::Instant) -> i64 {
    i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX).max(0)
}

pub fn parse_numeric_id(value: &str) -> Option<u64> {
    value.parse().ok()
}

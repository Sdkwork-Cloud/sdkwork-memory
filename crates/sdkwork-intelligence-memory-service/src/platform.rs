use std::sync::OnceLock;

use rand::Rng;
use sdkwork_database_id::{NodeLease, SnowflakeIdGenerator};
use sdkwork_id_core::max_snowflake_node_id;
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

// ---------------------------------------------------------------------------
// Snowflake ID generator — database-backed with env fallback
// ---------------------------------------------------------------------------

/// Holds the initialized generator and an optional database lease.
///
/// The lease keeps the database heartbeat alive while the generator is in use.
/// When the process exits, the heartbeat stops and the lease expires after its
/// TTL, allowing another process to reclaim the node_id.
struct IdGeneratorHolder {
    generator: SnowflakeIdGenerator,
    _lease: Option<NodeLease>,
}

static ID_GENERATOR: OnceLock<IdGeneratorHolder> = OnceLock::new();

/// Initialize the global ID generator from a database-allocated node_id.
///
/// This is the recommended initialization path for production. Call this
/// during application bootstrap after the database pool is available.
///
/// The `lease` must be kept alive for as long as the process generates IDs.
/// It is stored in the global holder and dropped when the process exits.
pub fn init_id_generator(generator: SnowflakeIdGenerator, lease: Option<NodeLease>) {
    let _ = ID_GENERATOR.set(IdGeneratorHolder {
        generator,
        _lease: lease,
    });
}

/// Fallback: resolve a node_id from env var or a random value.
///
/// Used only when database-backed allocation is not available (e.g. dev/test).
/// Uses `SDKWORK_MEMORY_SNOWFLAKE_NODE_ID` if set, otherwise a random u16
/// in `0..1024` to avoid collisions between processes on the same host.
fn resolve_snowflake_node_id() -> u16 {
    if let Ok(value) = std::env::var("SDKWORK_MEMORY_SNOWFLAKE_NODE_ID") {
        if let Ok(parsed) = value.parse::<u16>() {
            return parsed;
        }
    }

    // Random node_id to avoid collisions between processes on the same host.
    rand::thread_rng().gen::<u16>() % max_snowflake_node_id()
}

fn id_generator() -> &'static SnowflakeIdGenerator {
    let holder = ID_GENERATOR.get_or_init(|| {
        let node_id = resolve_snowflake_node_id();
        tracing::warn!(
            node_id,
            "memory snowflake generator using fallback (env/hostname hash) — \
             database-backed allocation was not initialized"
        );
        IdGeneratorHolder {
            generator: SnowflakeIdGenerator::new(node_id)
                .unwrap_or_else(|error| {
                    panic!("memory snowflake generator must initialize: {error}")
                }),
            _lease: None,
        }
    });
    &holder.generator
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

/// Returns true when the runtime is configured for a production-like environment.
pub fn is_production_like_environment() -> bool {
    sdkwork_memory_contract::memory_is_production_like_environment()
}

pub fn elapsed_millis_i64(started: std::time::Instant) -> i64 {
    i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX).max(0)
}

pub fn stable_query_hash(query: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let normalized = query.trim().to_lowercase();
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("query:{:016x}", hasher.finish())
}

pub fn parse_numeric_id(value: &str) -> Option<u64> {
    value.parse().ok()
}

/// Maximum allowed page size for list operations.
pub const MAX_PAGE_SIZE: i32 = 100;
/// Default page size for list operations.
pub const DEFAULT_PAGE_SIZE: i32 = 20;

/// Clamps a page size to a safe range \[1, MAX_PAGE_SIZE\], defaulting to
/// `DEFAULT_PAGE_SIZE` when `None`.
pub fn clamp_page_size(page_size: Option<i32>) -> i32 {
    page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE)
}

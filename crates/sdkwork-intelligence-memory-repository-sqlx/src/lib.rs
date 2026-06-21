//! SQL storage support for SDKWork Memory.

mod bootstrap;
pub mod db;

// DATABASE_SPEC.md section 34: repository-sqlx anchors the shared repository crate.
use sdkwork_database_repository as _;

pub use bootstrap::{
    bootstrap_memory_database, bootstrap_memory_database_from_env,
    bootstrap_memory_data_plane_from_env, connect_and_bootstrap_memory_database_from_env,
    MemoryDataPlane,
};
pub use db::{
    connect_memory_pool_from_env, install_sqlite_schema, open_native_sql_store_from_pool,
    MemoryDatabasePool,
};

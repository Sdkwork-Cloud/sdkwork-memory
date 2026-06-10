use sdkwork_memory_spi::MemoryPluginManifest;

pub const NATIVE_SQL_PLUGIN_ID: &str = "sdkwork-memory-plugin-native-sql";

pub fn native_sql_manifest() -> MemoryPluginManifest {
    MemoryPluginManifest::native_sql_baseline()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSqlPortBuilder {
    pub port_name: &'static str,
    pub builder_name: &'static str,
    pub ready: bool,
}

pub fn build_native_sql_record_store() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryRecordStorePort",
        builder_name: "build_native_sql_record_store",
        ready: true,
    }
}

pub fn build_native_sql_event_store() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryEventStorePort",
        builder_name: "build_native_sql_event_store",
        ready: true,
    }
}

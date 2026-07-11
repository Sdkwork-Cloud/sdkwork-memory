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

pub fn build_native_sql_audit_store() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryAuditStorePort",
        builder_name: "build_native_sql_audit_store",
        ready: true,
    }
}

pub fn build_native_sql_outbox_store() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryOutboxStorePort",
        builder_name: "build_native_sql_outbox_store",
        ready: true,
    }
}

pub fn build_native_sql_candidate_store() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryCandidateStorePort",
        builder_name: "build_native_sql_candidate_store",
        ready: true,
    }
}

pub fn build_native_sql_habit_store() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryHabitStorePort",
        builder_name: "build_native_sql_habit_store",
        ready: true,
    }
}

pub fn build_native_sql_retrieval_trace_store() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryRetrievalTraceStorePort",
        builder_name: "build_native_sql_retrieval_trace_store",
        ready: true,
    }
}

pub fn build_native_sql_retriever() -> NativeSqlPortBuilder {
    NativeSqlPortBuilder {
        port_name: "MemoryRetrieverPort",
        builder_name: "build_native_sql_retriever",
        ready: true,
    }
}

pub fn native_sql_phase1_port_builders() -> [NativeSqlPortBuilder; 8] {
    [
        build_native_sql_record_store(),
        build_native_sql_event_store(),
        build_native_sql_audit_store(),
        build_native_sql_outbox_store(),
        build_native_sql_candidate_store(),
        build_native_sql_habit_store(),
        build_native_sql_retrieval_trace_store(),
        build_native_sql_retriever(),
    ]
}

/// Manifest `build_ports` preflight: every declared builder must be executable.
pub fn validate_native_sql_port_builders(manifest: &MemoryPluginManifest) -> Result<(), String> {
    for builder in native_sql_phase1_port_builders() {
        if !builder.ready {
            return Err(format!(
                "native sql port builder {} is not ready for {}",
                builder.builder_name,
                builder.port_name
            ));
        }
        let declared = manifest
            .port_exports
            .iter()
            .any(|export| export.port == builder.port_name && export.builder == builder.builder_name);
        if !declared {
            return Err(format!(
                "native sql manifest must declare {} via {}",
                builder.port_name,
                builder.builder_name
            ));
        }
    }
    Ok(())
}

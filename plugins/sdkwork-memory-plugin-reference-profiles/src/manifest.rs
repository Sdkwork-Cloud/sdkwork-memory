use sdkwork_memory_spi::MemoryPluginManifest;

pub const REFERENCE_PROFILES_PLUGIN_ID: &str = "sdkwork-memory-plugin-reference-profiles";

pub fn reference_profiles_manifest() -> MemoryPluginManifest {
    MemoryPluginManifest::reference_profiles_baseline()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceProfilePortBuilder {
    pub port_name: &'static str,
    pub builder_name: &'static str,
    pub ready: bool,
    pub fail_closed: bool,
}

pub fn build_reference_record_store() -> ReferenceProfilePortBuilder {
    ready_builder("MemoryRecordStorePort", "build_reference_record_store")
}

pub fn build_reference_event_store() -> ReferenceProfilePortBuilder {
    ready_builder("MemoryEventStorePort", "build_reference_event_store")
}

pub fn build_reference_audit_store() -> ReferenceProfilePortBuilder {
    ready_builder("MemoryAuditStorePort", "build_reference_audit_store")
}

pub fn build_reference_outbox_store() -> ReferenceProfilePortBuilder {
    ready_builder("MemoryOutboxStorePort", "build_reference_outbox_store")
}

pub fn build_reference_candidate_store() -> ReferenceProfilePortBuilder {
    ready_builder(
        "MemoryCandidateStorePort",
        "build_reference_candidate_store",
    )
}

pub fn build_reference_habit_store() -> ReferenceProfilePortBuilder {
    ready_builder("MemoryHabitStorePort", "build_reference_habit_store")
}

pub fn build_reference_retrieval_trace_store() -> ReferenceProfilePortBuilder {
    ready_builder(
        "MemoryRetrievalTraceStorePort",
        "build_reference_retrieval_trace_store",
    )
}

pub fn build_reference_governance_access() -> ReferenceProfilePortBuilder {
    ready_builder(
        "MemoryGovernanceAccessPort",
        "build_reference_governance_access",
    )
}

pub fn build_reference_space_store() -> ReferenceProfilePortBuilder {
    ready_builder("MemorySpaceStorePort", "build_reference_space_store")
}

pub fn build_reference_retriever() -> ReferenceProfilePortBuilder {
    ready_builder("MemoryRetrieverPort", "build_reference_retriever")
}

pub fn build_reference_index() -> ReferenceProfilePortBuilder {
    fail_closed_builder("MemoryIndexPort", "build_reference_index")
}

pub fn build_reference_external_bridge() -> ReferenceProfilePortBuilder {
    fail_closed_builder(
        "ExternalMemoryBridgePort",
        "build_reference_external_bridge",
    )
}

pub fn build_reference_context_assembler() -> ReferenceProfilePortBuilder {
    ready_builder(
        "MemoryContextAssemblerPort",
        "build_reference_context_assembler",
    )
}

pub fn build_reference_evaluation() -> ReferenceProfilePortBuilder {
    fail_closed_builder("MemoryEvaluationPort", "build_reference_evaluation")
}

fn ready_builder(
    port_name: &'static str,
    builder_name: &'static str,
) -> ReferenceProfilePortBuilder {
    ReferenceProfilePortBuilder {
        port_name,
        builder_name,
        ready: true,
        fail_closed: false,
    }
}

fn fail_closed_builder(
    port_name: &'static str,
    builder_name: &'static str,
) -> ReferenceProfilePortBuilder {
    ReferenceProfilePortBuilder {
        port_name,
        builder_name,
        ready: false,
        fail_closed: true,
    }
}

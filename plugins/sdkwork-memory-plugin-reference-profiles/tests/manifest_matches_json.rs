use std::fs;

use sdkwork_memory_plugin_reference_profiles::{
    build_reference_audit_store, build_reference_candidate_store,
    build_reference_context_assembler, build_reference_evaluation, build_reference_event_store,
    build_reference_external_bridge, build_reference_habit_store, build_reference_index,
    build_reference_outbox_store, build_reference_record_store,
    build_reference_retrieval_trace_store, build_reference_retriever, reference_profiles_manifest,
};
use sdkwork_memory_spi::{MemoryImplementationKind, MemoryPluginManifest};

#[test]
fn rust_manifest_matches_source_controlled_json_manifest() {
    let json_path = format!("{}/sdkwork.memory.plugin.json", env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(json_path).unwrap();
    let json_manifest: MemoryPluginManifest = serde_json::from_str(&json).unwrap();
    let rust_manifest = reference_profiles_manifest();

    assert_eq!(
        serde_json::to_value(&rust_manifest).unwrap(),
        serde_json::to_value(&json_manifest).unwrap(),
        "Rust and source-controlled reference profile manifests must match in full"
    );
    assert!(rust_manifest.capabilities.candidate_lifecycle);
    assert!(rust_manifest.capabilities.habit_learning);
    assert!(rust_manifest.capabilities.retrieval_trace);
    assert!(rust_manifest.validate().is_ok());

    for implementation_kind in [
        MemoryImplementationKind::EventSourced,
        MemoryImplementationKind::SearchFirst,
        MemoryImplementationKind::GraphTemporal,
        MemoryImplementationKind::ExternalProviderBridge,
        MemoryImplementationKind::HybridPlatform,
    ] {
        assert!(rust_manifest
            .implementation_kinds
            .contains(&implementation_kind));
    }
}

#[test]
fn manifest_builders_are_exported_by_plugin_crate() {
    let manifest = reference_profiles_manifest();
    let builders = [
        build_reference_record_store(),
        build_reference_event_store(),
        build_reference_audit_store(),
        build_reference_outbox_store(),
        build_reference_candidate_store(),
        build_reference_habit_store(),
        build_reference_retrieval_trace_store(),
        build_reference_retriever(),
        build_reference_index(),
        build_reference_external_bridge(),
        build_reference_context_assembler(),
        build_reference_evaluation(),
    ];

    for builder in builders {
        assert!(manifest
            .port_exports
            .iter()
            .any(|export| export.builder == builder.builder_name));
        assert!(builder.ready);
    }

    assert!(build_reference_external_bridge().fail_closed);
}

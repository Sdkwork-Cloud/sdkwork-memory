use std::fs;

use sdkwork_memory_plugin_reference_profiles::{
    build_reference_audit_store, build_reference_candidate_store,
    build_reference_context_assembler, build_reference_evaluation, build_reference_event_store,
    build_reference_external_bridge, build_reference_governance_access,
    build_reference_habit_store, build_reference_index, build_reference_outbox_store,
    build_reference_record_store, build_reference_retrieval_trace_store, build_reference_retriever,
    build_reference_space_store, reference_profiles_manifest,
};
use sdkwork_memory_spi::{
    MemoryImplementationKind, MemoryPluginManifest, MemoryPluginRole, MemoryRetrieverKind,
};

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

    assert_eq!(
        rust_manifest.implementation_kinds,
        vec![MemoryImplementationKind::SearchFirst]
    );
    assert!(rust_manifest.provider_kinds.is_empty());
    assert!(rust_manifest.index_kinds.is_empty());
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
        build_reference_governance_access(),
        build_reference_space_store(),
        build_reference_retriever(),
        build_reference_context_assembler(),
    ];

    for builder in builders {
        assert!(manifest
            .port_exports
            .iter()
            .any(|export| export.builder == builder.builder_name));
        assert!(builder.ready);
    }

    for builder in [
        build_reference_index(),
        build_reference_external_bridge(),
        build_reference_evaluation(),
    ] {
        assert!(!builder.ready);
        assert!(builder.fail_closed);
        assert!(!manifest
            .port_exports
            .iter()
            .any(|export| export.builder == builder.builder_name));
    }
}

#[test]
fn manifest_declares_reference_retriever_contract() {
    let manifest = reference_profiles_manifest();
    let builder = build_reference_retriever();
    let export = manifest
        .port_exports
        .iter()
        .find(|export| export.port == builder.port_name)
        .expect("reference manifest must export MemoryRetrieverPort");

    assert_eq!(builder.port_name, "MemoryRetrieverPort");
    assert_eq!(export.builder, builder.builder_name);
    assert!(builder.ready);
    assert!(manifest.plugin_roles.contains(&MemoryPluginRole::Retriever));
    assert_eq!(manifest.retriever_kinds, vec![MemoryRetrieverKind::Keyword]);
}

use std::fs;

use sdkwork_memory_plugin_native_sql::{
    build_native_sql_audit_store, build_native_sql_candidate_store, build_native_sql_event_store,
    build_native_sql_habit_store, build_native_sql_outbox_store, build_native_sql_record_store,
    build_native_sql_retrieval_trace_store, native_sql_manifest, validate_native_sql_port_builders,
};
use sdkwork_memory_spi::{MemoryImplementationKind, MemoryPluginManifest};

#[test]
fn rust_manifest_matches_source_controlled_json_manifest() {
    let json_path = format!("{}/sdkwork.memory.plugin.json", env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(json_path).unwrap();
    let json_manifest: MemoryPluginManifest = serde_json::from_str(&json).unwrap();
    let rust_manifest = native_sql_manifest();

    assert_eq!(
        serde_json::to_value(&rust_manifest).unwrap(),
        serde_json::to_value(&json_manifest).unwrap(),
        "Rust and source-controlled native SQL manifests must match in full"
    );
    assert!(rust_manifest.validate().is_ok());
    assert!(rust_manifest
        .implementation_kinds
        .contains(&MemoryImplementationKind::NativeSql));
    assert!(rust_manifest
        .implementation_kinds
        .contains(&MemoryImplementationKind::LocalEmbedded));
    assert!(!rust_manifest.capabilities.embedding_required);
}

#[test]
fn manifest_record_store_builder_is_exported_by_plugin_crate() {
    let manifest = native_sql_manifest();
    let builder = build_native_sql_record_store();

    assert!(manifest
        .port_exports
        .iter()
        .any(|export| export.builder == builder.builder_name));
    assert_eq!(builder.port_name, "MemoryRecordStorePort");
    assert!(builder.ready);
}

#[test]
fn manifest_event_store_builder_is_exported_by_plugin_crate() {
    let manifest = native_sql_manifest();
    let builder = build_native_sql_event_store();

    assert!(manifest
        .port_exports
        .iter()
        .any(|export| export.builder == builder.builder_name));
    assert_eq!(builder.port_name, "MemoryEventStorePort");
    assert!(builder.ready);
}

#[test]
fn manifest_audit_store_builder_is_exported_by_plugin_crate() {
    let manifest = native_sql_manifest();
    let builder = build_native_sql_audit_store();

    assert!(manifest
        .port_exports
        .iter()
        .any(|export| export.builder == builder.builder_name));
    assert_eq!(builder.port_name, "MemoryAuditStorePort");
    assert!(builder.ready);
}

#[test]
fn manifest_outbox_store_builder_is_exported_by_plugin_crate() {
    let manifest = native_sql_manifest();
    let builder = build_native_sql_outbox_store();

    assert!(manifest
        .port_exports
        .iter()
        .any(|export| export.builder == builder.builder_name));
    assert_eq!(builder.port_name, "MemoryOutboxStorePort");
    assert!(builder.ready);
}

#[test]
fn manifest_candidate_store_builder_is_exported_by_plugin_crate() {
    let manifest = native_sql_manifest();
    let builder = build_native_sql_candidate_store();

    assert!(manifest
        .port_exports
        .iter()
        .any(|export| export.builder == builder.builder_name));
    assert_eq!(builder.port_name, "MemoryCandidateStorePort");
    assert!(builder.ready);
}

#[test]
fn manifest_habit_store_builder_is_exported_by_plugin_crate() {
    let manifest = native_sql_manifest();
    let builder = build_native_sql_habit_store();

    assert!(manifest
        .port_exports
        .iter()
        .any(|export| export.builder == builder.builder_name));
    assert_eq!(builder.port_name, "MemoryHabitStorePort");
    assert!(builder.ready);
}

#[test]
fn manifest_retrieval_trace_store_builder_is_exported_by_plugin_crate() {
    let manifest = native_sql_manifest();
    let builder = build_native_sql_retrieval_trace_store();

    assert!(manifest
        .port_exports
        .iter()
        .any(|export| export.builder == builder.builder_name));
    assert_eq!(builder.port_name, "MemoryRetrievalTraceStorePort");
    assert!(builder.ready);
}

#[test]
fn manifest_declares_native_sql_store_ports() {
    let manifest = native_sql_manifest();
    let ports = manifest
        .port_exports
        .iter()
        .map(|export| export.port.as_str())
        .collect::<Vec<_>>();

    assert!(ports.contains(&"MemoryRecordStorePort"));
    assert!(ports.contains(&"MemoryEventStorePort"));
    assert!(ports.contains(&"MemoryAuditStorePort"));
    assert!(ports.contains(&"MemoryOutboxStorePort"));
    assert!(ports.contains(&"MemoryCandidateStorePort"));
    assert!(ports.contains(&"MemoryHabitStorePort"));
    assert!(ports.contains(&"MemoryRetrievalTraceStorePort"));
}

#[test]
fn validate_native_sql_port_builders_matches_manifest_exports() {
    let manifest = native_sql_manifest();
    validate_native_sql_port_builders(&manifest).expect("phase1 port builders must be ready");
}

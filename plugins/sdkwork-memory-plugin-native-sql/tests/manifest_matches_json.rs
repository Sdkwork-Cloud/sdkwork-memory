use std::fs;

use sdkwork_memory_plugin_native_sql::{
    build_native_sql_event_store, build_native_sql_record_store, native_sql_manifest,
};
use sdkwork_memory_spi::{MemoryImplementationKind, MemoryPluginManifest};

#[test]
fn rust_manifest_matches_source_controlled_json_manifest() {
    let json_path = format!("{}/sdkwork.memory.plugin.json", env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(json_path).unwrap();
    let json_manifest: MemoryPluginManifest = serde_json::from_str(&json).unwrap();
    let rust_manifest = native_sql_manifest();

    assert_eq!(rust_manifest.plugin_id, json_manifest.plugin_id);
    assert_eq!(rust_manifest.package_name, json_manifest.package_name);
    assert_eq!(rust_manifest.version, json_manifest.version);
    assert_eq!(rust_manifest.capabilities, json_manifest.capabilities);
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
fn manifest_declares_record_and_event_store_ports() {
    let manifest = native_sql_manifest();
    let ports = manifest
        .port_exports
        .iter()
        .map(|export| export.port.as_str())
        .collect::<Vec<_>>();

    assert!(ports.contains(&"MemoryRecordStorePort"));
    assert!(ports.contains(&"MemoryEventStorePort"));
}

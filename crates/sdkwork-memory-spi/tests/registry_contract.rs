use sdkwork_memory_spi::{
    MemoryImplementationKind, MemoryPluginManifest, MemoryPluginRegistry, MemorySpiError,
};

#[test]
fn registry_registers_plugins_and_finds_implementation_kinds() {
    let mut registry = MemoryPluginRegistry::default();
    let manifest = MemoryPluginManifest::native_sql_for_test();

    registry.register(manifest).unwrap();

    let native_sql = registry.plugins_for_implementation(MemoryImplementationKind::NativeSql);
    assert_eq!(native_sql.len(), 1);
    assert_eq!(native_sql[0].plugin_id, "sdkwork-memory-plugin-native-sql");
}

#[test]
fn registry_registers_phase1_baseline_plugins_for_every_implementation_kind() {
    let mut registry = MemoryPluginRegistry::default();
    for manifest in MemoryPluginManifest::phase1_baseline_manifests_for_test() {
        registry.register(manifest).unwrap();
    }

    for implementation_kind in [
        MemoryImplementationKind::NativeSql,
        MemoryImplementationKind::LocalEmbedded,
        MemoryImplementationKind::EventSourced,
        MemoryImplementationKind::SearchFirst,
        MemoryImplementationKind::GraphTemporal,
        MemoryImplementationKind::ExternalProviderBridge,
        MemoryImplementationKind::HybridPlatform,
    ] {
        assert_eq!(
            registry
                .plugins_for_implementation(implementation_kind.clone())
                .len(),
            1,
            "expected exactly one phase1 baseline plugin for {implementation_kind:?}"
        );
    }
}

#[test]
fn registry_rejects_duplicate_plugin_ids() {
    let mut registry = MemoryPluginRegistry::default();
    let manifest = MemoryPluginManifest::native_sql_for_test();

    registry.register(manifest.clone()).unwrap();
    let err = registry.register(manifest).unwrap_err();

    assert!(matches!(err, MemorySpiError::DuplicatePluginId(_)));
}

#[test]
fn registry_validates_required_port_exports_before_runtime_serves() {
    let mut registry = MemoryPluginRegistry::default();
    registry
        .register(MemoryPluginManifest::native_sql_for_test())
        .unwrap();

    registry
        .validate_required_ports(
            "sdkwork-memory-plugin-native-sql",
            &[
                "MemoryRecordStorePort",
                "MemoryEventStorePort",
                "MemoryAuditStorePort",
                "MemoryOutboxStorePort",
                "MemoryCandidateStorePort",
                "MemoryHabitStorePort",
                "MemoryRetrievalTraceStorePort",
            ],
        )
        .unwrap();

    let err = registry
        .validate_required_ports(
            "sdkwork-memory-plugin-native-sql",
            &[
                "MemoryRecordStorePort",
                "MemoryEventStorePort",
                "MemoryAuditStorePort",
                "MemoryOutboxStorePort",
                "MemoryCandidateStorePort",
                "MemoryHabitStorePort",
                "MemoryRetrievalTraceStorePort",
                "MemoryPolicyStorePort",
            ],
        )
        .unwrap_err();
    assert!(matches!(err, MemorySpiError::RequiredPortMissing { .. }));
}

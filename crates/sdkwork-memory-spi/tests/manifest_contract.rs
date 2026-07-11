use sdkwork_memory_spi::{
    MemoryDeploymentMode, MemoryImplementationKind, MemoryPluginManifest, MemoryPluginRole,
};

#[test]
fn native_sql_manifest_deserializes_and_declares_no_embedding_baseline() {
    let manifest: MemoryPluginManifest = serde_json::from_str(
        r#"{
          "schemaVersion": 1,
          "kind": "sdkwork.memory.plugin",
          "pluginId": "sdkwork-memory-plugin-native-sql",
          "packageName": "sdkwork-memory-plugin-native-sql",
          "displayName": "SDKWork Memory Native SQL Plugin",
          "version": "0.1.0",
          "owner": "sdkwork-memory",
          "implementationKinds": ["native_sql", "local_embedded"],
          "pluginRoles": ["implementation", "store"],
          "deploymentModes": ["server", "container", "private", "local", "test"],
          "portExports": [
            {"port": "MemoryRecordStorePort", "builder": "build_native_sql_record_store"},
            {"port": "MemoryEventStorePort", "builder": "build_native_sql_event_store"},
            {"port": "MemoryAuditStorePort", "builder": "build_native_sql_audit_store"},
            {"port": "MemoryOutboxStorePort", "builder": "build_native_sql_outbox_store"},
            {"port": "MemoryCandidateStorePort", "builder": "build_native_sql_candidate_store"},
            {"port": "MemoryHabitStorePort", "builder": "build_native_sql_habit_store"},
            {"port": "MemoryRetrievalTraceStorePort", "builder": "build_native_sql_retrieval_trace_store"}
          ],
          "providerKinds": [],
          "retrieverKinds": [],
          "indexKinds": [],
          "requiredCoreVersion": "0.1.0",
          "secretRefs": [],
          "dataClasses": ["tenant", "personal"],
          "capabilities": {
            "canonicalStore": true,
            "eventLog": true,
            "candidateLifecycle": true,
            "habitLearning": true,
            "retrievalTrace": true,
            "deletionPropagation": true,
            "auditLog": true,
            "outboxLog": true,
            "embeddingRequired": false
          },
          "degradation": {"mode": "fail_required_degrade_optional", "returnsStaleHits": false},
          "migration": {"exportSupported": true, "importSupported": true, "dualWriteSupported": false, "shadowReadSupported": true},
          "observability": {"metricsPrefix": "sdkwork_memory_native_sql", "redactsPayloads": true},
          "conformance": {"suite": "sdkwork-memory-plugin-conformance", "suiteVersion": "0.1.0"}
        }"#,
    )
    .unwrap();

    assert_eq!(manifest.schema_version, 1);
    assert!(manifest
        .implementation_kinds
        .contains(&MemoryImplementationKind::NativeSql));
    assert!(manifest
        .implementation_kinds
        .contains(&MemoryImplementationKind::LocalEmbedded));
    assert!(manifest
        .plugin_roles
        .contains(&MemoryPluginRole::Implementation));
    assert!(!manifest.capabilities.embedding_required);
    assert!(manifest.validate().is_ok());
}

#[test]
fn phase1_baseline_manifests_cover_all_implementation_families() {
    let manifests = MemoryPluginManifest::phase1_baseline_manifests_for_test();
    let covered_kinds = manifests
        .iter()
        .flat_map(|manifest| manifest.implementation_kinds.iter())
        .collect::<Vec<_>>();

    for implementation_kind in [
        MemoryImplementationKind::NativeSql,
        MemoryImplementationKind::LocalEmbedded,
        MemoryImplementationKind::EventSourced,
        MemoryImplementationKind::SearchFirst,
        MemoryImplementationKind::GraphTemporal,
        MemoryImplementationKind::ExternalProviderBridge,
        MemoryImplementationKind::HybridPlatform,
    ] {
        assert!(
            covered_kinds.contains(&&implementation_kind),
            "phase1 baseline manifests must cover {implementation_kind:?}"
        );
    }

    for manifest in manifests {
        assert!(manifest.validate().is_ok());
    }
}

#[test]
fn manifest_rejects_secret_values_and_agent_plugin_paths() {
    let mut manifest = MemoryPluginManifest::native_sql_for_test();
    manifest
        .secret_refs
        .push("literal-token-secret".to_string());
    assert!(manifest.validate().is_err());

    let mut manifest = MemoryPluginManifest::native_sql_for_test();
    manifest.package_name = ".sdkwork/plugins/sdkwork-memory-plugin-native-sql".to_string();
    assert!(manifest.validate().is_err());
}

#[test]
fn manifest_rejects_enabled_capabilities_without_required_ports() {
    let mut manifest = MemoryPluginManifest::native_sql_for_test();
    manifest
        .port_exports
        .retain(|export| export.port != "MemoryAuditStorePort");

    assert!(manifest.validate().is_err());

    let mut manifest = MemoryPluginManifest::native_sql_for_test();
    manifest
        .port_exports
        .retain(|export| export.port != "MemoryOutboxStorePort");

    assert!(manifest.validate().is_err());
}

#[test]
fn reference_profiles_are_explicitly_evaluation_only() {
    let manifest = MemoryPluginManifest::reference_profiles_for_test();

    assert_eq!(
        manifest.deployment_modes,
        vec![MemoryDeploymentMode::Test, MemoryDeploymentMode::EvalOnly]
    );
    assert!(!manifest
        .deployment_modes
        .contains(&MemoryDeploymentMode::Server));
    assert!(!manifest
        .deployment_modes
        .contains(&MemoryDeploymentMode::Container));
}

#[test]
fn manifest_rejects_missing_or_duplicate_executable_port_builders() {
    let mut manifest = MemoryPluginManifest::native_sql_for_test();
    manifest.port_exports[0].builder.clear();
    assert!(manifest.validate().is_err());

    let mut manifest = MemoryPluginManifest::native_sql_for_test();
    manifest.port_exports[1].port = manifest.port_exports[0].port.clone();
    assert!(manifest.validate().is_err());
}

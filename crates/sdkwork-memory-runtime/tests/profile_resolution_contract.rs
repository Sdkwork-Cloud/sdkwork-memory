use sdkwork_memory_runtime::{
    MemoryImplementationProfileDraft, MemoryRuntimeError, MemoryRuntimeProfileResolver,
};
use sdkwork_memory_spi::{MemoryImplementationKind, MemoryPluginManifest, MemoryPluginRegistry};

fn registry_with_native_sql() -> MemoryPluginRegistry {
    let mut registry = MemoryPluginRegistry::default();
    registry
        .register(MemoryPluginManifest::native_sql_for_test())
        .unwrap();
    registry
}

fn registry_with_phase1_baselines() -> MemoryPluginRegistry {
    let mut registry = MemoryPluginRegistry::default();
    for manifest in MemoryPluginManifest::phase1_baseline_manifests_for_test() {
        registry.register(manifest).unwrap();
    }
    registry
}

#[test]
fn native_sql_profile_resolves_when_plugin_is_registered() {
    let registry = registry_with_native_sql();
    let profile = MemoryImplementationProfileDraft::native_sql_phase1();

    let resolved = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap();

    assert_eq!(resolved.profile_id, "native-sql-phase1");
    assert_eq!(
        resolved.primary_plugin_id,
        "sdkwork-memory-plugin-native-sql"
    );
}

#[test]
fn native_sql_and_local_embedded_profiles_require_learning_and_trace_ports() {
    for profile in [
        MemoryImplementationProfileDraft::native_sql_phase1(),
        MemoryImplementationProfileDraft::local_embedded_phase1(),
    ] {
        for required_port in [
            "MemoryRecordStorePort",
            "MemoryEventStorePort",
            "MemoryAuditStorePort",
            "MemoryOutboxStorePort",
            "MemoryCandidateStorePort",
            "MemoryHabitStorePort",
            "MemoryRetrievalTraceStorePort",
        ] {
            assert!(
                profile.required_ports.contains(&required_port.to_string()),
                "{} must require {required_port}",
                profile.profile_id
            );
        }
    }
}

#[test]
fn reference_family_profiles_require_learning_and_trace_ports() {
    for profile in MemoryImplementationProfileDraft::phase1_family_baselines()
        .into_iter()
        .filter(|profile| profile.primary_plugin_id == "sdkwork-memory-plugin-reference-profiles")
    {
        for required_port in [
            "MemoryCandidateStorePort",
            "MemoryHabitStorePort",
            "MemoryRetrievalTraceStorePort",
        ] {
            assert!(
                profile.required_ports.contains(&required_port.to_string()),
                "{} must require {required_port}",
                profile.profile_id
            );
        }
    }
}

#[test]
fn phase1_family_profiles_resolve_when_baseline_plugins_are_registered() {
    let registry = registry_with_phase1_baselines();

    let resolved = MemoryImplementationProfileDraft::phase1_family_baselines()
        .into_iter()
        .map(|profile| {
            MemoryRuntimeProfileResolver::new(&registry)
                .resolve(profile)
                .unwrap()
        })
        .collect::<Vec<_>>();

    let implementation_kinds = resolved
        .iter()
        .map(|profile| profile.implementation_kind.clone())
        .collect::<Vec<_>>();

    assert_eq!(resolved.len(), 7);
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
            implementation_kinds.contains(&implementation_kind),
            "phase1 family profiles must include {implementation_kind:?}"
        );
    }
}

#[test]
fn profile_fails_before_serving_when_primary_plugin_is_missing() {
    let registry = MemoryPluginRegistry::default();
    let profile = MemoryImplementationProfileDraft::native_sql_phase1();

    let err = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap_err();

    assert!(matches!(err, MemoryRuntimeError::PrimaryPluginMissing(_)));
}

#[test]
fn profile_fails_before_serving_when_required_port_is_missing() {
    let registry = registry_with_native_sql();
    let mut profile = MemoryImplementationProfileDraft::native_sql_phase1();
    profile
        .required_ports
        .push("MemoryPolicyStorePort".to_string());

    let err = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap_err();

    assert!(matches!(
        err,
        MemoryRuntimeError::RequiredPortMissing { .. }
    ));
}

#[test]
fn profile_rejects_safe_config_that_contains_literal_secret_values() {
    let registry = registry_with_native_sql();
    let mut profile = MemoryImplementationProfileDraft::native_sql_phase1();
    profile.safe_config_json = serde_json::json!({
        "token": "literal-token-secret"
    });

    let err = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap_err();

    assert!(matches!(err, MemoryRuntimeError::UnsafeConfigSecret(_)));
}

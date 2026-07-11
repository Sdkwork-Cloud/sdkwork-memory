use sdkwork_memory_profile_resolver::{
    MemoryImplementationProfileDraft, MemoryRuntimeError, MemoryRuntimeProfileResolver,
};
use sdkwork_memory_spi::{
    MemoryDeploymentMode, MemoryImplementationKind, MemoryPluginManifest, MemoryPluginRegistry,
};

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
    assert_eq!(resolved.deployment_mode, MemoryDeploymentMode::Server);
}

#[test]
fn local_embedded_profile_resolves_only_for_local_deployment() {
    let registry = registry_with_native_sql();
    let profile = MemoryImplementationProfileDraft::local_embedded_phase1();

    let resolved = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap();

    assert_eq!(resolved.profile_id, "local-embedded-phase1");
    assert_eq!(resolved.deployment_mode, MemoryDeploymentMode::Local);
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

    for profile in resolved
        .iter()
        .filter(|profile| profile.primary_plugin_id == "sdkwork-memory-plugin-reference-profiles")
    {
        assert_eq!(profile.deployment_mode, MemoryDeploymentMode::EvalOnly);
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

#[test]
fn reference_profile_cannot_be_promoted_to_server_without_production_plugin_support() {
    let registry = registry_with_phase1_baselines();
    let mut profile = MemoryImplementationProfileDraft::search_first_phase1();
    profile.deployment_mode = MemoryDeploymentMode::Server;

    let err = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap_err();

    assert!(matches!(
        err,
        MemoryRuntimeError::DeploymentModeUnsupported {
            deployment_mode: MemoryDeploymentMode::Server,
            ..
        }
    ));
}

#[test]
fn hybrid_profile_can_compose_required_ports_across_plugins() {
    let registry = registry_with_phase1_baselines();
    let native_sql = "sdkwork-memory-plugin-native-sql";
    let mut profile = MemoryImplementationProfileDraft::hybrid_platform_phase1();
    profile.deployment_mode = MemoryDeploymentMode::Test;
    let profile = profile
        .with_port_binding("MemoryRecordStorePort", native_sql)
        .with_port_binding("MemoryEventStorePort", native_sql)
        .with_port_binding("MemoryAuditStorePort", native_sql)
        .with_port_binding("MemoryOutboxStorePort", native_sql)
        .with_port_binding("MemoryCandidateStorePort", native_sql)
        .with_port_binding("MemoryHabitStorePort", native_sql)
        .with_port_binding("MemoryRetrievalTraceStorePort", native_sql);

    let resolved = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .expect("hybrid profile should compose ports from both baseline plugins");

    assert_eq!(resolved.deployment_mode, MemoryDeploymentMode::Test);
    assert!(resolved
        .port_bindings
        .iter()
        .filter(|binding| binding.plugin_id == native_sql)
        .count()
        >= 7);
    assert!(resolved.port_bindings.iter().any(|binding| {
        binding.port == "MemoryRetrieverPort"
            && binding.plugin_id == "sdkwork-memory-plugin-reference-profiles"
    }));
}

#[test]
fn profile_rejects_unknown_or_duplicate_port_bindings() {
    let registry = registry_with_phase1_baselines();

    let profile = MemoryImplementationProfileDraft::native_sql_phase1()
        .with_port_binding("MemoryPolicyStorePort", "sdkwork-memory-plugin-native-sql");
    let err = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap_err();
    assert!(matches!(
        err,
        MemoryRuntimeError::PortBindingNotRequired { .. }
    ));

    let profile = MemoryImplementationProfileDraft::native_sql_phase1()
        .with_port_binding(
            "MemoryRecordStorePort",
            "sdkwork-memory-plugin-native-sql",
        )
        .with_port_binding(
            "MemoryRecordStorePort",
            "sdkwork-memory-plugin-native-sql",
        );
    let err = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap_err();
    assert!(matches!(err, MemoryRuntimeError::DuplicatePortBinding(_)));
}

#[test]
fn profile_rejects_bound_plugin_that_cannot_serve_the_selected_mode() {
    let registry = registry_with_phase1_baselines();
    let profile = MemoryImplementationProfileDraft::native_sql_phase1().with_port_binding(
        "MemoryRecordStorePort",
        "sdkwork-memory-plugin-reference-profiles",
    );

    let err = MemoryRuntimeProfileResolver::new(&registry)
        .resolve(profile)
        .unwrap_err();
    assert!(matches!(
        err,
        MemoryRuntimeError::DeploymentModeUnsupported {
            plugin_id,
            deployment_mode: MemoryDeploymentMode::Server,
        } if plugin_id == "sdkwork-memory-plugin-reference-profiles"
    ));
}

use sdkwork_memory_runtime::{
    MemoryImplementationProfileDraft, MemoryRuntimeError, MemoryRuntimeProfileResolver,
};
use sdkwork_memory_spi::{MemoryPluginManifest, MemoryPluginRegistry};

fn registry_with_native_sql() -> MemoryPluginRegistry {
    let mut registry = MemoryPluginRegistry::default();
    registry
        .register(MemoryPluginManifest::native_sql_for_test())
        .unwrap();
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

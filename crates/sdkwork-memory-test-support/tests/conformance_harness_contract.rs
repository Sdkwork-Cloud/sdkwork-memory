use sdkwork_memory_spi::MemoryPluginManifest;
use sdkwork_memory_test_support::{ConformanceCheckStatus, MemoryPluginConformanceHarness};

#[test]
fn conformance_harness_reports_native_sql_baseline_checks_explicitly() {
    let manifest = MemoryPluginManifest::native_sql_for_test();
    let report = MemoryPluginConformanceHarness.verify_manifest_skeleton(&manifest);

    assert!(report.has_status("manifest_validation", ConformanceCheckStatus::Passed));
    assert!(report.has_status("required_port_declarations", ConformanceCheckStatus::Passed));
    assert!(report.has_status("no_embedding_profile", ConformanceCheckStatus::Passed));
    assert!(report.has_status("secret_redaction", ConformanceCheckStatus::Passed));
    assert!(report.has_status(
        "runtime_plugin_path_separation",
        ConformanceCheckStatus::Passed
    ));
    assert!(report.has_status("store_crud", ConformanceCheckStatus::Passed));
    assert!(report.has_status("tenant_isolation", ConformanceCheckStatus::Passed));
    assert!(report.has_status("deletion_propagation", ConformanceCheckStatus::Passed));
    assert!(report.has_status("retriever", ConformanceCheckStatus::Passed));
    assert!(report.has_status("index", ConformanceCheckStatus::Pending));
    assert!(report.has_status("retrieval_trace", ConformanceCheckStatus::Passed));
    assert!(report.has_status("audit_and_outbox", ConformanceCheckStatus::Passed));
    assert!(report.has_status("candidate_lifecycle", ConformanceCheckStatus::Passed));
    assert!(report.has_status("habit_learning", ConformanceCheckStatus::Passed));
}

#[test]
fn conformance_harness_reports_reference_profile_baseline_checks_explicitly() {
    let manifest = MemoryPluginManifest::reference_profiles_for_test();
    let report = MemoryPluginConformanceHarness.verify_manifest_skeleton(&manifest);

    assert!(report.has_status("manifest_validation", ConformanceCheckStatus::Passed));
    assert!(report.has_status("required_port_declarations", ConformanceCheckStatus::Passed));
    assert!(report.has_status("no_embedding_profile", ConformanceCheckStatus::Passed));
    assert!(report.has_status("store_crud", ConformanceCheckStatus::Passed));
    assert!(report.has_status("tenant_isolation", ConformanceCheckStatus::Passed));
    assert!(report.has_status("deletion_propagation", ConformanceCheckStatus::Passed));
    assert!(report.has_status("retriever", ConformanceCheckStatus::Passed));
    assert!(report.has_status("index", ConformanceCheckStatus::Pending));
    assert!(report.has_status("retrieval_trace", ConformanceCheckStatus::Passed));
    assert!(report.has_status("audit_and_outbox", ConformanceCheckStatus::Passed));
    assert!(report.has_status("candidate_lifecycle", ConformanceCheckStatus::Passed));
    assert!(report.has_status("habit_learning", ConformanceCheckStatus::Passed));
    assert!(report.has_status("external_bridge", ConformanceCheckStatus::Pending));
}

use sdkwork_memory_spi::MemoryPluginManifest;
use sdkwork_memory_test_support::{ConformanceCheckStatus, MemoryPluginConformanceHarness};

#[test]
fn conformance_harness_reports_passed_and_pending_checks_explicitly() {
    let manifest = MemoryPluginManifest::native_sql_for_test();
    let report = MemoryPluginConformanceHarness::default().verify_manifest_skeleton(&manifest);

    assert!(report.has_status("manifest_validation", ConformanceCheckStatus::Passed));
    assert!(report.has_status("required_port_declarations", ConformanceCheckStatus::Passed));
    assert!(report.has_status("no_embedding_profile", ConformanceCheckStatus::Passed));
    assert!(report.has_status("secret_redaction", ConformanceCheckStatus::Passed));
    assert!(report.has_status(
        "runtime_plugin_path_separation",
        ConformanceCheckStatus::Passed
    ));
    assert!(report.has_status("store_crud", ConformanceCheckStatus::Pending));
    assert!(report.has_status("tenant_isolation", ConformanceCheckStatus::Pending));
    assert!(report.has_status("deletion_propagation", ConformanceCheckStatus::Pending));
    assert!(report.has_status("audit_and_outbox", ConformanceCheckStatus::Pending));
}

use std::sync::Arc;

use sdkwork_memory_plugin_reference_profiles::{
    build_reference_executable_runtime, reference_profiles_manifest, ReferenceMemoryRuntime,
};

#[test]
fn reference_executable_runtime_materializes_every_declared_port() {
    let runtime = build_reference_executable_runtime(Arc::new(ReferenceMemoryRuntime::new()));

    for export in reference_profiles_manifest().port_exports {
        assert!(
            runtime.has_port(&export.port),
            "missing executable reference port {}",
            export.port
        );
    }
}

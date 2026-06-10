//! SDKWork Memory test support and conformance helpers.

use sdkwork_memory_spi::MemoryPluginManifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConformanceCheckStatus {
    Passed,
    Failed,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceCheck {
    pub name: String,
    pub status: ConformanceCheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryPluginConformanceReport {
    pub plugin_id: String,
    pub checks: Vec<ConformanceCheck>,
}

impl MemoryPluginConformanceReport {
    pub fn has_status(&self, name: &str, status: ConformanceCheckStatus) -> bool {
        self.checks
            .iter()
            .any(|check| check.name == name && check.status == status)
    }
}

#[derive(Debug, Default)]
pub struct MemoryPluginConformanceHarness;

impl MemoryPluginConformanceHarness {
    pub fn verify_manifest_skeleton(
        &self,
        manifest: &MemoryPluginManifest,
    ) -> MemoryPluginConformanceReport {
        let mut checks = Vec::new();

        checks.push(check(
            "manifest_validation",
            if manifest.validate().is_ok() {
                ConformanceCheckStatus::Passed
            } else {
                ConformanceCheckStatus::Failed
            },
            "Manifest must satisfy SDKWork Memory plugin manifest rules.",
        ));
        checks.push(check(
            "required_port_declarations",
            if manifest.port_exports.is_empty() {
                ConformanceCheckStatus::Failed
            } else {
                ConformanceCheckStatus::Passed
            },
            "Plugin must declare executable port exports.",
        ));
        checks.push(check(
            "no_embedding_profile",
            if manifest.capabilities.embedding_required {
                ConformanceCheckStatus::Failed
            } else {
                ConformanceCheckStatus::Passed
            },
            "First native SQL profile must not require embeddings.",
        ));
        checks.push(check(
            "secret_redaction",
            if manifest.secret_refs.is_empty() {
                ConformanceCheckStatus::Passed
            } else if manifest.validate().is_ok() {
                ConformanceCheckStatus::Passed
            } else {
                ConformanceCheckStatus::Failed
            },
            "Manifest may contain secret references but no secret values.",
        ));
        checks.push(check(
            "runtime_plugin_path_separation",
            if manifest.package_name.contains(".sdkwork/plugins")
                || manifest.package_name.contains(".sdkwork\\plugins")
            {
                ConformanceCheckStatus::Failed
            } else {
                ConformanceCheckStatus::Passed
            },
            "Runtime plugins must live under plugins/, not .sdkwork/plugins/.",
        ));

        for pending in [
            "store_crud",
            "tenant_isolation",
            "deletion_propagation",
            "retrieval_trace",
            "audit_and_outbox",
        ] {
            checks.push(check(
                pending,
                ConformanceCheckStatus::Pending,
                "Runtime behavior check is pending until native SQL stores land.",
            ));
        }

        MemoryPluginConformanceReport {
            plugin_id: manifest.plugin_id.clone(),
            checks,
        }
    }
}

fn check(
    name: impl Into<String>,
    status: ConformanceCheckStatus,
    message: impl Into<String>,
) -> ConformanceCheck {
    ConformanceCheck {
        name: name.into(),
        status,
        message: message.into(),
    }
}

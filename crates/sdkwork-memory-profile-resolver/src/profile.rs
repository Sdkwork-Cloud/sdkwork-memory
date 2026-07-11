use sdkwork_memory_spi::{
    MemoryCoreRuntime, MemoryDeploymentMode, MemoryImplementationKind, MemoryPluginRegistry,
    MemoryRuntimeProfileMetadata, MemorySpiError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProfilePortBinding {
    pub port: String,
    pub plugin_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryImplementationProfileDraft {
    pub profile_id: String,
    pub implementation_kind: MemoryImplementationKind,
    pub primary_plugin_id: String,
    pub deployment_mode: MemoryDeploymentMode,
    #[serde(default)]
    pub port_bindings: Vec<MemoryProfilePortBinding>,
    pub required_ports: Vec<String>,
    pub safe_config_json: Value,
}

impl MemoryImplementationProfileDraft {
    pub fn native_sql_phase1() -> Self {
        Self {
            profile_id: "native-sql-phase1".to_string(),
            implementation_kind: MemoryImplementationKind::NativeSql,
            primary_plugin_id: "sdkwork-memory-plugin-native-sql".to_string(),
            deployment_mode: MemoryDeploymentMode::Server,
            port_bindings: Vec::new(),
            required_ports: native_sql_store_ports(),
            safe_config_json: Value::Object(Default::default()),
        }
    }

    pub fn local_embedded_phase1() -> Self {
        Self {
            profile_id: "local-embedded-phase1".to_string(),
            implementation_kind: MemoryImplementationKind::LocalEmbedded,
            primary_plugin_id: "sdkwork-memory-plugin-native-sql".to_string(),
            deployment_mode: MemoryDeploymentMode::Local,
            port_bindings: Vec::new(),
            required_ports: native_sql_store_ports(),
            safe_config_json: Value::Object(Default::default()),
        }
    }

    pub fn event_sourced_phase1() -> Self {
        reference_profile(
            "event-sourced-phase1",
            MemoryImplementationKind::EventSourced,
            vec![
                "MemoryEventStorePort",
                "MemoryRecordStorePort",
                "MemoryAuditStorePort",
                "MemoryOutboxStorePort",
            ],
        )
    }

    pub fn search_first_phase1() -> Self {
        reference_profile(
            "search-first-phase1",
            MemoryImplementationKind::SearchFirst,
            vec![
                "MemoryRecordStorePort",
                "MemoryRetrieverPort",
                "MemoryIndexPort",
                "MemoryAuditStorePort",
            ],
        )
    }

    pub fn graph_temporal_phase1() -> Self {
        reference_profile(
            "graph-temporal-phase1",
            MemoryImplementationKind::GraphTemporal,
            vec![
                "MemoryRecordStorePort",
                "MemoryRetrieverPort",
                "MemoryIndexPort",
                "MemoryContextAssemblerPort",
            ],
        )
    }

    pub fn external_provider_bridge_eval() -> Self {
        reference_profile(
            "external-provider-bridge-eval",
            MemoryImplementationKind::ExternalProviderBridge,
            vec![
                "ExternalMemoryBridgePort",
                "MemoryAuditStorePort",
                "MemoryOutboxStorePort",
            ],
        )
    }

    pub fn hybrid_platform_phase1() -> Self {
        reference_profile(
            "hybrid-platform-phase1",
            MemoryImplementationKind::HybridPlatform,
            vec![
                "MemoryRecordStorePort",
                "MemoryEventStorePort",
                "MemoryAuditStorePort",
                "MemoryOutboxStorePort",
                "MemoryRetrieverPort",
                "MemoryIndexPort",
                "ExternalMemoryBridgePort",
                "MemoryContextAssemblerPort",
                "MemoryEvaluationPort",
            ],
        )
    }

    pub fn phase1_family_baselines() -> Vec<Self> {
        vec![
            Self::native_sql_phase1(),
            Self::local_embedded_phase1(),
            Self::event_sourced_phase1(),
            Self::search_first_phase1(),
            Self::graph_temporal_phase1(),
            Self::external_provider_bridge_eval(),
            Self::hybrid_platform_phase1(),
        ]
    }
}

fn reference_profile(
    profile_id: &str,
    implementation_kind: MemoryImplementationKind,
    required_ports: Vec<&str>,
) -> MemoryImplementationProfileDraft {
    let mut required_ports = required_ports
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    for required_port in learning_and_trace_ports() {
        if !required_ports.contains(&required_port.to_string()) {
            required_ports.push(required_port.to_string());
        }
    }

    MemoryImplementationProfileDraft {
        profile_id: profile_id.to_string(),
        implementation_kind,
        primary_plugin_id: "sdkwork-memory-plugin-reference-profiles".to_string(),
        deployment_mode: MemoryDeploymentMode::EvalOnly,
        port_bindings: Vec::new(),
        required_ports,
        safe_config_json: Value::Object(Default::default()),
    }
}

fn native_sql_store_ports() -> Vec<String> {
    [
        "MemoryRecordStorePort".to_string(),
        "MemoryEventStorePort".to_string(),
        "MemoryAuditStorePort".to_string(),
        "MemoryOutboxStorePort".to_string(),
        "MemoryCandidateStorePort".to_string(),
        "MemoryHabitStorePort".to_string(),
        "MemoryRetrievalTraceStorePort".to_string(),
    ]
    .to_vec()
}

fn learning_and_trace_ports() -> [&'static str; 3] {
    [
        "MemoryCandidateStorePort",
        "MemoryHabitStorePort",
        "MemoryRetrievalTraceStorePort",
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMemoryImplementationProfile {
    pub profile_id: String,
    pub implementation_kind: MemoryImplementationKind,
    pub primary_plugin_id: String,
    pub deployment_mode: MemoryDeploymentMode,
    pub port_bindings: Vec<MemoryProfilePortBinding>,
}

impl MemoryImplementationProfileDraft {
    pub fn with_port_binding(
        mut self,
        port: impl Into<String>,
        plugin_id: impl Into<String>,
    ) -> Self {
        self.port_bindings.push(MemoryProfilePortBinding {
            port: port.into(),
            plugin_id: plugin_id.into(),
        });
        self
    }
}

#[derive(Debug)]
pub struct MemoryRuntimeProfileResolver<'a> {
    registry: &'a MemoryPluginRegistry,
}

impl<'a> MemoryRuntimeProfileResolver<'a> {
    pub fn new(registry: &'a MemoryPluginRegistry) -> Self {
        Self { registry }
    }

    pub fn resolve(
        &self,
        profile: MemoryImplementationProfileDraft,
    ) -> Result<ResolvedMemoryImplementationProfile, MemoryRuntimeError> {
        let manifest = self
            .registry
            .get(&profile.primary_plugin_id)
            .ok_or_else(|| {
                MemoryRuntimeError::PrimaryPluginMissing(profile.primary_plugin_id.clone())
            })?;

        if !manifest
            .implementation_kinds
            .contains(&profile.implementation_kind)
        {
            return Err(MemoryRuntimeError::ImplementationKindUnsupported {
                plugin_id: profile.primary_plugin_id,
                implementation_kind: format!("{:?}", profile.implementation_kind),
            });
        }

        if !manifest.deployment_modes.contains(&profile.deployment_mode) {
            return Err(MemoryRuntimeError::DeploymentModeUnsupported {
                plugin_id: profile.primary_plugin_id,
                deployment_mode: profile.deployment_mode,
            });
        }

        reject_literal_secrets(&profile.safe_config_json)?;

        let mut explicit_bindings = std::collections::HashMap::new();
        for binding in &profile.port_bindings {
            if !profile
                .required_ports
                .iter()
                .any(|port| port == &binding.port)
            {
                return Err(MemoryRuntimeError::PortBindingNotRequired {
                    port: binding.port.clone(),
                });
            }
            if explicit_bindings
                .insert(binding.port.clone(), binding.plugin_id.clone())
                .is_some()
            {
                return Err(MemoryRuntimeError::DuplicatePortBinding(
                    binding.port.clone(),
                ));
            }
        }

        let mut resolved_bindings = Vec::with_capacity(profile.required_ports.len());
        for required_port in &profile.required_ports {
            let plugin_id = explicit_bindings
                .get(required_port)
                .cloned()
                .unwrap_or_else(|| profile.primary_plugin_id.clone());
            let manifest = self.registry.get(&plugin_id).ok_or_else(|| {
                if plugin_id == profile.primary_plugin_id {
                    MemoryRuntimeError::PrimaryPluginMissing(plugin_id.clone())
                } else {
                    MemoryRuntimeError::PortBindingPluginMissing {
                        port: required_port.clone(),
                        plugin_id: plugin_id.clone(),
                    }
                }
            })?;

            if !manifest.deployment_modes.contains(&profile.deployment_mode) {
                return Err(MemoryRuntimeError::DeploymentModeUnsupported {
                    plugin_id,
                    deployment_mode: profile.deployment_mode.clone(),
                });
            }
            if !manifest
                .port_exports
                .iter()
                .any(|export| export.port == *required_port)
            {
                return Err(MemoryRuntimeError::RequiredPortMissing {
                    plugin_id,
                    port: required_port.clone(),
                });
            }

            resolved_bindings.push(MemoryProfilePortBinding {
                port: required_port.clone(),
                plugin_id,
            });
        }

        Ok(ResolvedMemoryImplementationProfile {
            profile_id: profile.profile_id,
            implementation_kind: profile.implementation_kind,
            primary_plugin_id: profile.primary_plugin_id,
            deployment_mode: profile.deployment_mode,
            port_bindings: resolved_bindings,
        })
    }

    pub fn assemble(
        &self,
        profile: &ResolvedMemoryImplementationProfile,
    ) -> Result<MemoryCoreRuntime, MemoryRuntimeError> {
        let mut runtime = MemoryCoreRuntime::new(MemoryRuntimeProfileMetadata {
            profile_id: profile.profile_id.clone(),
            implementation_kind: profile.implementation_kind.clone(),
            primary_plugin_id: profile.primary_plugin_id.clone(),
            deployment_mode: profile.deployment_mode.clone(),
        });

        for binding in &profile.port_bindings {
            let executable = self
                .registry
                .require_executable_runtime(&binding.plugin_id)
                .map_err(MemoryRuntimeError::from)?;
            runtime
                .bind_port(binding.plugin_id.clone(), &binding.port, executable)
                .map_err(MemoryRuntimeError::from)?;
        }

        Ok(runtime)
    }

    pub fn resolve_executable(
        &self,
        profile: MemoryImplementationProfileDraft,
    ) -> Result<MemoryCoreRuntime, MemoryRuntimeError> {
        let resolved = self.resolve(profile)?;
        self.assemble(&resolved)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemoryRuntimeError {
    #[error("primary memory plugin is missing: {0}")]
    PrimaryPluginMissing(String),
    #[error(
        "memory plugin {plugin_id} does not support implementation kind {implementation_kind}"
    )]
    ImplementationKindUnsupported {
        plugin_id: String,
        implementation_kind: String,
    },
    #[error("memory plugin {plugin_id} does not support deployment mode {deployment_mode:?}")]
    DeploymentModeUnsupported {
        plugin_id: String,
        deployment_mode: MemoryDeploymentMode,
    },
    #[error("memory profile binds non-required port {port}")]
    PortBindingNotRequired { port: String },
    #[error("memory profile port {port} is bound to missing plugin {plugin_id}")]
    PortBindingPluginMissing { port: String, plugin_id: String },
    #[error("memory profile declares duplicate binding for port {0}")]
    DuplicatePortBinding(String),
    #[error("memory runtime profile is missing required port {port} on plugin {plugin_id}")]
    RequiredPortMissing { plugin_id: String, port: String },
    #[error("memory plugin has no executable runtime registered: {0}")]
    ExecutablePluginMissing(String),
    #[error("memory plugin {plugin_id} has no executable port {port}")]
    ExecutablePortMissing { plugin_id: String, port: String },
    #[error("memory runtime safe config contains a literal secret-like value at {0}")]
    UnsafeConfigSecret(String),
    #[error("memory runtime SPI error: {0}")]
    Spi(String),
}

impl From<MemorySpiError> for MemoryRuntimeError {
    fn from(value: MemorySpiError) -> Self {
        match value {
            MemorySpiError::RequiredPortMissing { plugin_id, port } => {
                MemoryRuntimeError::RequiredPortMissing { plugin_id, port }
            }
            MemorySpiError::ExecutableRuntimeMissing { plugin_id } => {
                MemoryRuntimeError::ExecutablePluginMissing(plugin_id)
            }
            MemorySpiError::ExecutablePortMissing { plugin_id, port } => {
                MemoryRuntimeError::ExecutablePortMissing { plugin_id, port }
            }
            other => MemoryRuntimeError::Spi(other.to_string()),
        }
    }
}

fn reject_literal_secrets(value: &Value) -> Result<(), MemoryRuntimeError> {
    reject_literal_secrets_at("$", value)
}

fn reject_literal_secrets_at(path: &str, value: &Value) -> Result<(), MemoryRuntimeError> {
    match value {
        Value::String(text) if looks_like_secret_value(text) => {
            Err(MemoryRuntimeError::UnsafeConfigSecret(path.to_string()))
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_literal_secrets_at(&format!("{path}[{index}]"), item)?;
            }
            Ok(())
        }
        Value::Object(map) => {
            for (key, item) in map {
                if looks_like_secret_key(key) && item.is_string() {
                    return Err(MemoryRuntimeError::UnsafeConfigSecret(format!(
                        "{path}.{key}"
                    )));
                }
                reject_literal_secrets_at(&format!("{path}.{key}"), item)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn looks_like_secret_key(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("token")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("private_key")
        || lower.contains("secret")
}

fn looks_like_secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("literal")
        || lower.contains("password")
        || lower.contains("api_key")
        || lower.contains("private_key")
        || lower.contains("access_token")
        || lower.contains("refresh_token")
        || lower.contains("bearer ")
        || lower.contains("sk-")
        || lower.contains("token-secret")
}

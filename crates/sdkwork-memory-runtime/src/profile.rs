use sdkwork_memory_spi::{MemoryImplementationKind, MemoryPluginRegistry, MemorySpiError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryImplementationProfileDraft {
    pub profile_id: String,
    pub implementation_kind: MemoryImplementationKind,
    pub primary_plugin_id: String,
    pub required_ports: Vec<String>,
    pub safe_config_json: Value,
}

impl MemoryImplementationProfileDraft {
    pub fn native_sql_phase1() -> Self {
        Self {
            profile_id: "native-sql-phase1".to_string(),
            implementation_kind: MemoryImplementationKind::NativeSql,
            primary_plugin_id: "sdkwork-memory-plugin-native-sql".to_string(),
            required_ports: vec![
                "MemoryRecordStorePort".to_string(),
                "MemoryEventStorePort".to_string(),
                "MemoryAuditStorePort".to_string(),
                "MemoryOutboxStorePort".to_string(),
            ],
            safe_config_json: Value::Object(Default::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMemoryImplementationProfile {
    pub profile_id: String,
    pub implementation_kind: MemoryImplementationKind,
    pub primary_plugin_id: String,
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

        reject_literal_secrets(&profile.safe_config_json)?;

        let required_ports: Vec<&str> = profile.required_ports.iter().map(String::as_str).collect();
        self.registry
            .validate_required_ports(&profile.primary_plugin_id, &required_ports)
            .map_err(MemoryRuntimeError::from)?;

        Ok(ResolvedMemoryImplementationProfile {
            profile_id: profile.profile_id,
            implementation_kind: profile.implementation_kind,
            primary_plugin_id: profile.primary_plugin_id,
        })
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
    #[error("memory runtime profile is missing required port {port} on plugin {plugin_id}")]
    RequiredPortMissing { plugin_id: String, port: String },
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

use thiserror::Error;

pub type MemorySpiResult<T> = Result<T, MemorySpiError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemorySpiError {
    #[error("memory plugin manifest is invalid: {0}")]
    ManifestInvalid(String),
    #[error("memory plugin id is already registered: {0}")]
    DuplicatePluginId(String),
    #[error("memory plugin was not found: {0}")]
    PluginNotFound(String),
    #[error("memory plugin {plugin_id} is missing required port {port}")]
    RequiredPortMissing { plugin_id: String, port: String },
}

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
    #[error("memory idempotency conflict for key {idempotency_key}")]
    IdempotencyConflict { idempotency_key: String },
    #[error("memory plugin port {port} operation failed: {message}")]
    PortOperationFailed { port: String, message: String },
    #[error("memory plugin {plugin_id} has no executable runtime registered")]
    ExecutableRuntimeMissing { plugin_id: String },
    #[error("memory plugin executable runtime is already registered: {0}")]
    DuplicateExecutableRuntime(String),
    #[error("memory plugin {plugin_id} has no executable port {port}")]
    ExecutablePortMissing { plugin_id: String, port: String },
    #[error("memory runtime port is already bound: {0}")]
    ExecutablePortAlreadyBound(String),
    #[error("memory runtime port is unsupported: {0}")]
    UnsupportedRuntimePort(String),
    #[error("memory plugin {plugin_id} executable runtime exports undeclared port {port}")]
    ExecutablePortUndeclared { plugin_id: String, port: String },
}

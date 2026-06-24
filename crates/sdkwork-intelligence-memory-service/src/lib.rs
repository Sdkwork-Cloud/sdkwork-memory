//! Business service boundary for SDKWork Memory HTTP runtime.

mod access;
mod app_backend_api;
mod backend_admin_api;
mod candidate_promotion;
mod domain_metrics;
mod open_api;
mod outbox_delivery;
mod outbox_publisher;
mod platform;
mod store_error;
mod tenant_quota;

pub use open_api::OpenMemoryService;
pub use outbox_publisher::spawn_outbox_publisher;
pub use domain_metrics::{
    memory_domain_metrics, render_memory_domain_prometheus,
};

pub type MemoryProductService = OpenMemoryService;

use sdkwork_memory_spi::error::MemorySpiError;

#[derive(Debug, Default)]
pub struct MemoryService;

impl MemoryService {
    pub fn new() -> Self {
        Self
    }

    pub fn health_check(&self) -> Result<&'static str, MemorySpiError> {
        Ok("ok")
    }
}

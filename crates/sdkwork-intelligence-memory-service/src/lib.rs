//! Business service boundary for SDKWork Memory HTTP runtime.

mod access;
mod app_backend_api;
mod backend_admin_api;
mod candidate_promotion;
mod open_api;
mod outbox_publisher;
mod platform;
mod store_error;

pub use open_api::OpenMemoryService;
pub use outbox_publisher::spawn_outbox_publisher;

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

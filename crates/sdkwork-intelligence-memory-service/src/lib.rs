//! Business service boundary for SDKWork Memory HTTP runtime.

mod app_backend_api;
mod backend_admin_api;
mod candidate_promotion;
mod open_api;

pub use open_api::OpenMemoryService;

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

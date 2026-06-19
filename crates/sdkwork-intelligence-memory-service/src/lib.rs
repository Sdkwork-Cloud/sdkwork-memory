//! Business service boundary for SDKWork Memory HTTP runtime.

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

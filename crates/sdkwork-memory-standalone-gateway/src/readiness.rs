use std::sync::Arc;

use sdkwork_intelligence_memory_service::{memory_domain_metrics, OpenMemoryService};
use sdkwork_routes_memory_support::memory_dependency_ready_check;
use sdkwork_web_bootstrap::{ReadinessCheck, ReadinessFuture};

pub struct MemoryReadinessCheck {
    service: Arc<OpenMemoryService>,
}

impl MemoryReadinessCheck {
    pub fn new(service: Arc<OpenMemoryService>) -> Self {
        Self { service }
    }
}

impl ReadinessCheck for MemoryReadinessCheck {
    fn check(&self) -> ReadinessFuture<'_> {
        let service = self.service.clone();
        Box::pin(async move {
            if service.ready_check().await.is_err() {
                memory_domain_metrics().set_serving(false);
                return Err("memory store not ready".to_owned());
            }
            if !memory_dependency_ready_check().await {
                memory_domain_metrics().set_serving(false);
                return Err("memory dependencies not ready".to_owned());
            }
            memory_domain_metrics().set_serving(true);
            Ok(())
        })
    }
}

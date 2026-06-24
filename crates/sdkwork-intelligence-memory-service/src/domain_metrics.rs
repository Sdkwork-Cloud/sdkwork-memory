use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

static DOMAIN_METRICS: OnceLock<MemoryDomainMetrics> = OnceLock::new();

pub struct MemoryDomainMetrics {
    retrieval_total: AtomicU64,
    authz_denied_total: AtomicU64,
    quota_exceeded_total: AtomicU64,
    outbox_published_total: AtomicU64,
    outbox_publish_failed_total: AtomicU64,
    outbox_delivery_failed_total: AtomicU64,
    serving: AtomicU64,
}

impl MemoryDomainMetrics {
    fn new() -> Self {
        Self {
            retrieval_total: AtomicU64::new(0),
            authz_denied_total: AtomicU64::new(0),
            quota_exceeded_total: AtomicU64::new(0),
            outbox_published_total: AtomicU64::new(0),
            outbox_publish_failed_total: AtomicU64::new(0),
            outbox_delivery_failed_total: AtomicU64::new(0),
            serving: AtomicU64::new(1),
        }
    }

    pub fn record_retrieval_completed(&self) {
        self.retrieval_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_authz_denied(&self) {
        self.authz_denied_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_quota_exceeded(&self) {
        self.quota_exceeded_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_outbox_published(&self) {
        self.outbox_published_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_outbox_publish_failed(&self) {
        self.outbox_publish_failed_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_outbox_delivery_failed(&self) {
        self.outbox_delivery_failed_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_serving(&self, serving: bool) {
        self.serving.store(if serving { 1 } else { 0 }, Ordering::Relaxed);
    }

    pub fn render_prometheus(
        &self,
        service: &str,
        environment: &str,
        deployment_profile: &str,
        runtime_target: &str,
        runtime_profile: &str,
    ) -> String {
        let labels = format!(
            "service=\"{service}\",environment=\"{environment}\",deployment_profile=\"{deployment_profile}\",runtime_target=\"{runtime_target}\",runtime_profile=\"{runtime_profile}\""
        );
        format!(
            "# HELP memory_retrieval_completed_total Memory retrieval operations completed.\n\
             # TYPE memory_retrieval_completed_total counter\n\
             memory_retrieval_completed_total{{{labels}}} {}\n\
             # HELP memory_authz_denied_total Memory authorization denials at service layer.\n\
             # TYPE memory_authz_denied_total counter\n\
             memory_authz_denied_total{{{labels}}} {}\n\
             # HELP memory_quota_exceeded_total Memory tenant or space quota rejections.\n\
             # TYPE memory_quota_exceeded_total counter\n\
             memory_quota_exceeded_total{{{labels}}} {}\n\
             # HELP memory_outbox_published_total Memory domain outbox events published.\n\
             # TYPE memory_outbox_published_total counter\n\
             memory_outbox_published_total{{{labels}}} {}\n\
             # HELP memory_outbox_publish_failed_total Memory domain outbox claim failures.\n\
             # TYPE memory_outbox_publish_failed_total counter\n\
             memory_outbox_publish_failed_total{{{labels}}} {}\n\
             # HELP memory_outbox_delivery_failed_total Memory domain outbox delivery or ack failures.\n\
             # TYPE memory_outbox_delivery_failed_total counter\n\
             memory_outbox_delivery_failed_total{{{labels}}} {}\n\
             # HELP memory_health_status Memory service health (1=serving, 0=not serving).\n\
             # TYPE memory_health_status gauge\n\
             memory_health_status{{{labels}}} {}\n",
            self.retrieval_total.load(Ordering::Relaxed),
            self.authz_denied_total.load(Ordering::Relaxed),
            self.quota_exceeded_total.load(Ordering::Relaxed),
            self.outbox_published_total.load(Ordering::Relaxed),
            self.outbox_publish_failed_total.load(Ordering::Relaxed),
            self.outbox_delivery_failed_total.load(Ordering::Relaxed),
            self.serving.load(Ordering::Relaxed),
        )
    }
}

pub fn memory_domain_metrics() -> &'static MemoryDomainMetrics {
    DOMAIN_METRICS.get_or_init(MemoryDomainMetrics::new)
}

pub fn render_memory_domain_prometheus(
    service: &str,
    environment: &str,
    deployment_profile: &str,
    runtime_target: &str,
    runtime_profile: &str,
) -> String {
    memory_domain_metrics().render_prometheus(
        service,
        environment,
        deployment_profile,
        runtime_target,
        runtime_profile,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prometheus_render_includes_quota_counter() {
        memory_domain_metrics().record_quota_exceeded();
        let rendered = render_memory_domain_prometheus(
            "sdkwork-memory-api-server",
            "test",
            "standalone",
            "server",
            "sqlite",
        );
        assert!(rendered.contains("memory_quota_exceeded_total"));
        assert!(rendered.contains("# TYPE memory_quota_exceeded_total counter"));
    }
}

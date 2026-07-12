//! Phase-1 native SQL runtime bundle: one store instance backing all required SPI ports.

use std::sync::Arc;

use sdkwork_database_config::DatabaseConfig;
use sdkwork_memory_spi::{
    MemoryAuditStorePort, MemoryCandidateStorePort, MemoryEventStorePort,
    MemoryExecutablePluginRuntime, MemoryGovernanceAccessPort, MemoryHabitStorePort,
    MemoryOutboxStorePort, MemoryPluginPorts, MemoryRecordStorePort, MemoryRetrievalTraceStorePort,
    MemoryRetrieverPort, MemorySpaceStorePort,
};

use crate::store::{NativeSqlMemoryStore, NativeSqlStoreError};

/// Executable phase-1 runtime for the native SQL plugin profile.
pub struct NativeSqlPhase1Runtime {
    store: Arc<NativeSqlMemoryStore>,
}

impl NativeSqlPhase1Runtime {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, NativeSqlStoreError> {
        Self::open(config, true).await
    }

    /// Use when `sdkwork-memory-database-host` already applied postgres lifecycle migrations.
    pub async fn connect_without_migration(
        config: &DatabaseConfig,
    ) -> Result<Self, NativeSqlStoreError> {
        Self::open(config, false).await
    }

    async fn open(
        config: &DatabaseConfig,
        apply_migration: bool,
    ) -> Result<Self, NativeSqlStoreError> {
        let store = NativeSqlMemoryStore::open_pool(config, apply_migration).await?;
        Ok(Self::from_store(store))
    }

    pub fn from_store(store: NativeSqlMemoryStore) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    pub fn store(&self) -> &NativeSqlMemoryStore {
        &self.store
    }

    pub fn into_arc_store(self) -> Arc<NativeSqlMemoryStore> {
        self.store
    }

    pub fn into_store(self) -> NativeSqlMemoryStore {
        Arc::try_unwrap(self.store).unwrap_or_else(|arc| (*arc).clone())
    }

    pub fn executable_plugin_runtime(&self) -> MemoryExecutablePluginRuntime {
        build_native_sql_executable_runtime(self.store.clone())
    }
}

pub fn build_native_sql_executable_runtime(
    store: Arc<NativeSqlMemoryStore>,
) -> MemoryExecutablePluginRuntime {
    MemoryExecutablePluginRuntime::new(
        MemoryPluginPorts::new()
            .with_record_store(store.clone())
            .with_event_store(store.clone())
            .with_audit_store(store.clone())
            .with_outbox_store(store.clone())
            .with_candidate_store(store.clone())
            .with_habit_store(store.clone())
            .with_retrieval_trace_store(store.clone())
            .with_governance_access(store.clone())
            .with_space_store(store.clone())
            .with_retriever(store),
    )
}

/// Runtime proof that the store exposes every phase-1 required SPI port plus DB readiness.
pub async fn validate_native_sql_phase1_ports(
    store: &NativeSqlMemoryStore,
) -> Result<(), NativeSqlStoreError> {
    let record: &dyn MemoryRecordStorePort = store;
    let event: &dyn MemoryEventStorePort = store;
    let audit: &dyn MemoryAuditStorePort = store;
    let outbox: &dyn MemoryOutboxStorePort = store;
    let candidate: &dyn MemoryCandidateStorePort = store;
    let habit: &dyn MemoryHabitStorePort = store;
    let trace: &dyn MemoryRetrievalTraceStorePort = store;
    let governance: &dyn MemoryGovernanceAccessPort = store;
    let space: &dyn MemorySpaceStorePort = store;
    let retriever: &dyn MemoryRetrieverPort = store;

    let _ = (
        record, event, audit, outbox, candidate, habit, trace, governance, space, retriever,
    );

    store.ping().await
}

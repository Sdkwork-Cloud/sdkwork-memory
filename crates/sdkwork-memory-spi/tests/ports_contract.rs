use async_trait::async_trait;
use sdkwork_memory_spi::{
    AppendMemoryAuditCommand, AssembleMemoryContextCommand, EmbeddingCommand, EmbeddingModelPort,
    ExternalMemoryBridgePort, ExternalMemoryDeleteCommand, ExternalMemoryDeleteReceipt,
    ExternalMemoryExportCommand, ExternalMemoryExportResult, ExternalMemoryImportCommand,
    ExternalMemoryImportResult, ExternalMemoryShadowReadCommand, ExternalMemoryShadowReadResult,
    LanguageModelCommand, LanguageModelPort, MemoryAuditRecord, MemoryAuditStorePort,
    MemoryContextAssemblerPort, MemoryContextPackDraft, MemoryEvalRunResult, MemoryEvaluationPort,
    MemoryEvent, MemoryEventStorePort, MemoryIndexPort, MemoryIndexReceipt, MemoryPolicy,
    MemoryPolicyStorePort, MemoryRecord, MemoryRecordStorePort, MemoryRetrieverPort,
    MemoryRetrieverResult, RerankMemoryHitsCommand, RerankMemoryHitsResult, RerankModelPort,
    RetrieveMemoryCandidatesCommand, RunMemoryEvalCommand,
};

struct FakePorts;

#[async_trait]
impl MemoryRecordStorePort for FakePorts {
    async fn create(
        &self,
        command: sdkwork_memory_spi::CreateMemoryRecordCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryRecord> {
        Ok(MemoryRecord {
            memory_id: command.memory_id,
            content: command.content,
        })
    }

    async fn retrieve(
        &self,
        query: sdkwork_memory_spi::RetrieveMemoryRecordQuery,
    ) -> sdkwork_memory_spi::MemorySpiResult<Option<MemoryRecord>> {
        Ok(Some(MemoryRecord {
            memory_id: query.memory_id,
            content: "redacted memory".to_string(),
        }))
    }
}

#[async_trait]
impl MemoryEventStorePort for FakePorts {
    async fn append(
        &self,
        command: sdkwork_memory_spi::AppendMemoryEventCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryEvent> {
        Ok(MemoryEvent {
            event_id: command.event_id,
            content: command.content,
        })
    }

    async fn retrieve(
        &self,
        query: sdkwork_memory_spi::RetrieveMemoryEventQuery,
    ) -> sdkwork_memory_spi::MemorySpiResult<Option<MemoryEvent>> {
        Ok(Some(MemoryEvent {
            event_id: query.event_id,
            content: "redacted event".to_string(),
        }))
    }
}

#[async_trait]
impl MemoryAuditStorePort for FakePorts {
    async fn append(
        &self,
        command: AppendMemoryAuditCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryAuditRecord> {
        Ok(MemoryAuditRecord {
            audit_id: command.audit_id,
            action: command.action,
            resource_type: command.resource_type,
            resource_id: command.resource_id,
            result: command.result,
        })
    }

    async fn retrieve(
        &self,
        query: sdkwork_memory_spi::RetrieveMemoryAuditQuery,
    ) -> sdkwork_memory_spi::MemorySpiResult<Option<MemoryAuditRecord>> {
        Ok(Some(MemoryAuditRecord {
            audit_id: query.audit_id,
            action: "memory.audit.checked".to_string(),
            resource_type: "mem_audit_log".to_string(),
            resource_id: "audit".to_string(),
            result: "success".to_string(),
        }))
    }
}

#[async_trait]
impl MemoryPolicyStorePort for FakePorts {
    async fn resolve_policy(
        &self,
        policy_code: String,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryPolicy> {
        Ok(MemoryPolicy { policy_code })
    }
}

#[async_trait]
impl MemoryRetrieverPort for FakePorts {
    fn retriever_code(&self) -> &str {
        "fake-retriever"
    }

    async fn retrieve(
        &self,
        _command: RetrieveMemoryCandidatesCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryRetrieverResult> {
        Ok(MemoryRetrieverResult { memory_ids: vec![] })
    }
}

#[async_trait]
impl MemoryIndexPort for FakePorts {
    fn index_kind(&self) -> &str {
        "sql"
    }

    async fn index(
        &self,
        memory_id: String,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryIndexReceipt> {
        Ok(MemoryIndexReceipt { memory_id })
    }
}

#[async_trait]
impl LanguageModelPort for FakePorts {
    fn provider_code(&self) -> &str {
        "fake-language"
    }

    async fn generate(
        &self,
        command: LanguageModelCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<String> {
        Ok(command.prompt)
    }
}

#[async_trait]
impl EmbeddingModelPort for FakePorts {
    fn provider_code(&self) -> &str {
        "fake-embedding"
    }

    fn dimensions(&self) -> usize {
        384
    }

    async fn embed(
        &self,
        command: EmbeddingCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<Vec<f32>> {
        Ok(vec![command.input.len() as f32])
    }
}

#[async_trait]
impl RerankModelPort for FakePorts {
    fn provider_code(&self) -> &str {
        "fake-rerank"
    }

    async fn rerank(
        &self,
        _command: RerankMemoryHitsCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<RerankMemoryHitsResult> {
        Ok(RerankMemoryHitsResult { memory_ids: vec![] })
    }
}

#[async_trait]
impl ExternalMemoryBridgePort for FakePorts {
    fn provider_code(&self) -> &str {
        "fake-external"
    }

    async fn import(
        &self,
        _command: ExternalMemoryImportCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<ExternalMemoryImportResult> {
        Ok(ExternalMemoryImportResult { imported_count: 0 })
    }

    async fn export(
        &self,
        _command: ExternalMemoryExportCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<ExternalMemoryExportResult> {
        Ok(ExternalMemoryExportResult { exported_count: 0 })
    }

    async fn delete(
        &self,
        _command: ExternalMemoryDeleteCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<ExternalMemoryDeleteReceipt> {
        Ok(ExternalMemoryDeleteReceipt { verified: true })
    }

    async fn shadow_read(
        &self,
        _command: ExternalMemoryShadowReadCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<ExternalMemoryShadowReadResult> {
        Ok(ExternalMemoryShadowReadResult { comparable: true })
    }
}

#[async_trait]
impl MemoryContextAssemblerPort for FakePorts {
    async fn assemble(
        &self,
        command: AssembleMemoryContextCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryContextPackDraft> {
        Ok(MemoryContextPackDraft {
            memory_ids: command.memory_ids,
            context_text: "redacted context".to_string(),
        })
    }
}

#[async_trait]
impl MemoryEvaluationPort for FakePorts {
    async fn run(
        &self,
        command: RunMemoryEvalCommand,
    ) -> sdkwork_memory_spi::MemorySpiResult<MemoryEvalRunResult> {
        Ok(MemoryEvalRunResult {
            eval_type: command.eval_type,
        })
    }
}

#[test]
fn spi_ports_are_provider_neutral_and_implementation_friendly() {
    fn assert_ports<T>()
    where
        T: MemoryRecordStorePort
            + MemoryEventStorePort
            + MemoryAuditStorePort
            + MemoryPolicyStorePort
            + MemoryRetrieverPort
            + MemoryIndexPort
            + LanguageModelPort
            + EmbeddingModelPort
            + RerankModelPort
            + ExternalMemoryBridgePort
            + MemoryContextAssemblerPort
            + MemoryEvaluationPort,
    {
    }

    assert_ports::<FakePorts>();
    assert_eq!(FakePorts.retriever_code(), "fake-retriever");
    assert_eq!(
        <FakePorts as EmbeddingModelPort>::dimensions(&FakePorts),
        384
    );
}

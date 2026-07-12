use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::{
    EmbeddingModelPort, ExternalMemoryBridgePort, LanguageModelPort, MemoryAuditStorePort,
    MemoryCandidateStorePort, MemoryContextAssemblerPort, MemoryDeploymentMode,
    MemoryEvaluationPort, MemoryEventStorePort, MemoryGovernanceAccessPort, MemoryHabitStorePort,
    MemoryImplementationKind, MemoryIndexPort, MemoryOutboxStorePort, MemoryPolicyStorePort,
    MemoryRecordStorePort, MemoryRetrievalTraceStorePort, MemoryRetrieverPort, MemoryRuntimePlugin,
    MemorySpaceStorePort, MemorySpiError, MemorySpiResult, RerankModelPort,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRuntimeProfileMetadata {
    pub profile_id: String,
    pub implementation_kind: MemoryImplementationKind,
    pub primary_plugin_id: String,
    pub deployment_mode: MemoryDeploymentMode,
}

macro_rules! define_memory_plugin_ports {
    (
        $(
            $field:ident,
            $with_method:ident,
            $getter:ident,
            $port_trait:ident,
            $port_name:literal;
        )+
    ) => {
        #[derive(Clone, Default)]
        pub struct MemoryPluginPorts {
            $(
                $field: Option<Arc<dyn $port_trait>>,
            )+
        }

        impl MemoryPluginPorts {
            pub fn new() -> Self {
                Self::default()
            }

            $(
                pub fn $with_method(mut self, port: Arc<dyn $port_trait>) -> Self {
                    self.$field = Some(port);
                    self
                }

                pub fn $getter(&self) -> Option<Arc<dyn $port_trait>> {
                    self.$field.clone()
                }
            )+

            pub fn has_port(&self, port: &str) -> bool {
                match port {
                    $(
                        $port_name => self.$field.is_some(),
                    )+
                    _ => false,
                }
            }

            pub fn port_names(&self) -> Vec<&'static str> {
                let mut ports = Vec::new();
                $(
                    if self.$field.is_some() {
                        ports.push($port_name);
                    }
                )+
                ports
            }

            pub fn supports_port_name(port: &str) -> bool {
                matches!(
                    port,
                    $(
                        $port_name
                    )|+
                )
            }

            fn bind_port_to(
                &self,
                plugin_id: &str,
                port: &str,
                target: &mut MemoryPluginPorts,
            ) -> MemorySpiResult<()> {
                match port {
                    $(
                        $port_name => {
                            let executable = self.$field.clone().ok_or_else(|| {
                                MemorySpiError::ExecutablePortMissing {
                                    plugin_id: plugin_id.to_string(),
                                    port: port.to_string(),
                                }
                            })?;
                            if target.$field.replace(executable).is_some() {
                                return Err(MemorySpiError::ExecutablePortAlreadyBound(
                                    port.to_string(),
                                ));
                            }
                            Ok(())
                        }
                    )+
                    _ => Err(MemorySpiError::UnsupportedRuntimePort(port.to_string())),
                }
            }
        }

        impl fmt::Debug for MemoryPluginPorts {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct("MemoryPluginPorts")
                    .field("port_names", &self.port_names())
                    .finish()
            }
        }
    };
}

define_memory_plugin_ports! {
    record_store,
    with_record_store,
    record_store,
    MemoryRecordStorePort,
    "MemoryRecordStorePort";
    event_store,
    with_event_store,
    event_store,
    MemoryEventStorePort,
    "MemoryEventStorePort";
    audit_store,
    with_audit_store,
    audit_store,
    MemoryAuditStorePort,
    "MemoryAuditStorePort";
    outbox_store,
    with_outbox_store,
    outbox_store,
    MemoryOutboxStorePort,
    "MemoryOutboxStorePort";
    candidate_store,
    with_candidate_store,
    candidate_store,
    MemoryCandidateStorePort,
    "MemoryCandidateStorePort";
    habit_store,
    with_habit_store,
    habit_store,
    MemoryHabitStorePort,
    "MemoryHabitStorePort";
    retrieval_trace_store,
    with_retrieval_trace_store,
    retrieval_trace_store,
    MemoryRetrievalTraceStorePort,
    "MemoryRetrievalTraceStorePort";
    policy_store,
    with_policy_store,
    policy_store,
    MemoryPolicyStorePort,
    "MemoryPolicyStorePort";
    governance_access,
    with_governance_access,
    governance_access,
    MemoryGovernanceAccessPort,
    "MemoryGovernanceAccessPort";
    space_store,
    with_space_store,
    space_store,
    MemorySpaceStorePort,
    "MemorySpaceStorePort";
    retriever,
    with_retriever,
    retriever,
    MemoryRetrieverPort,
    "MemoryRetrieverPort";
    index,
    with_index,
    index,
    MemoryIndexPort,
    "MemoryIndexPort";
    language_model,
    with_language_model,
    language_model,
    LanguageModelPort,
    "LanguageModelPort";
    embedding_model,
    with_embedding_model,
    embedding_model,
    EmbeddingModelPort,
    "EmbeddingModelPort";
    rerank_model,
    with_rerank_model,
    rerank_model,
    RerankModelPort,
    "RerankModelPort";
    external_memory_bridge,
    with_external_memory_bridge,
    external_memory_bridge,
    ExternalMemoryBridgePort,
    "ExternalMemoryBridgePort";
    context_assembler,
    with_context_assembler,
    context_assembler,
    MemoryContextAssemblerPort,
    "MemoryContextAssemblerPort";
    evaluation,
    with_evaluation,
    evaluation,
    MemoryEvaluationPort,
    "MemoryEvaluationPort";
}

#[derive(Debug, Clone, Default)]
pub struct MemoryExecutablePluginRuntime {
    ports: MemoryPluginPorts,
}

impl MemoryRuntimePlugin for MemoryExecutablePluginRuntime {}

impl MemoryExecutablePluginRuntime {
    pub fn new(ports: MemoryPluginPorts) -> Self {
        Self { ports }
    }

    pub fn ports(&self) -> &MemoryPluginPorts {
        &self.ports
    }

    pub fn has_port(&self, port: &str) -> bool {
        self.ports.has_port(port)
    }
}

#[derive(Debug, Clone)]
pub struct MemoryCoreRuntime {
    profile: MemoryRuntimeProfileMetadata,
    port_owners: BTreeMap<String, String>,
    ports: MemoryPluginPorts,
}

impl MemoryCoreRuntime {
    pub fn new(profile: MemoryRuntimeProfileMetadata) -> Self {
        Self {
            profile,
            port_owners: BTreeMap::new(),
            ports: MemoryPluginPorts::new(),
        }
    }

    pub fn profile(&self) -> &MemoryRuntimeProfileMetadata {
        &self.profile
    }

    pub fn port_owners(&self) -> &BTreeMap<String, String> {
        &self.port_owners
    }

    pub fn port_owner(&self, port: &str) -> Option<&str> {
        self.port_owners.get(port).map(String::as_str)
    }

    pub fn has_port(&self, port: &str) -> bool {
        self.ports.has_port(port)
    }

    pub fn port_names(&self) -> Vec<&'static str> {
        self.ports.port_names()
    }

    pub fn bind_port(
        &mut self,
        plugin_id: impl Into<String>,
        port: &str,
        executable: &MemoryExecutablePluginRuntime,
    ) -> MemorySpiResult<()> {
        if self.port_owners.contains_key(port) {
            return Err(MemorySpiError::ExecutablePortAlreadyBound(port.to_string()));
        }

        let plugin_id = plugin_id.into();
        executable
            .ports
            .bind_port_to(&plugin_id, port, &mut self.ports)?;
        self.port_owners.insert(port.to_string(), plugin_id);
        Ok(())
    }

    pub fn record_store(&self) -> Option<Arc<dyn MemoryRecordStorePort>> {
        self.ports.record_store()
    }

    pub fn event_store(&self) -> Option<Arc<dyn MemoryEventStorePort>> {
        self.ports.event_store()
    }

    pub fn audit_store(&self) -> Option<Arc<dyn MemoryAuditStorePort>> {
        self.ports.audit_store()
    }

    pub fn outbox_store(&self) -> Option<Arc<dyn MemoryOutboxStorePort>> {
        self.ports.outbox_store()
    }

    pub fn candidate_store(&self) -> Option<Arc<dyn MemoryCandidateStorePort>> {
        self.ports.candidate_store()
    }

    pub fn habit_store(&self) -> Option<Arc<dyn MemoryHabitStorePort>> {
        self.ports.habit_store()
    }

    pub fn retrieval_trace_store(&self) -> Option<Arc<dyn MemoryRetrievalTraceStorePort>> {
        self.ports.retrieval_trace_store()
    }

    pub fn policy_store(&self) -> Option<Arc<dyn MemoryPolicyStorePort>> {
        self.ports.policy_store()
    }

    pub fn governance_access(&self) -> Option<Arc<dyn MemoryGovernanceAccessPort>> {
        self.ports.governance_access()
    }

    pub fn space_store(&self) -> Option<Arc<dyn MemorySpaceStorePort>> {
        self.ports.space_store()
    }

    pub fn retriever(&self) -> Option<Arc<dyn MemoryRetrieverPort>> {
        self.ports.retriever()
    }

    pub fn index(&self) -> Option<Arc<dyn MemoryIndexPort>> {
        self.ports.index()
    }

    pub fn language_model(&self) -> Option<Arc<dyn LanguageModelPort>> {
        self.ports.language_model()
    }

    pub fn embedding_model(&self) -> Option<Arc<dyn EmbeddingModelPort>> {
        self.ports.embedding_model()
    }

    pub fn rerank_model(&self) -> Option<Arc<dyn RerankModelPort>> {
        self.ports.rerank_model()
    }

    pub fn external_memory_bridge(&self) -> Option<Arc<dyn ExternalMemoryBridgePort>> {
        self.ports.external_memory_bridge()
    }

    pub fn context_assembler(&self) -> Option<Arc<dyn MemoryContextAssemblerPort>> {
        self.ports.context_assembler()
    }

    pub fn evaluation(&self) -> Option<Arc<dyn MemoryEvaluationPort>> {
        self.ports.evaluation()
    }
}

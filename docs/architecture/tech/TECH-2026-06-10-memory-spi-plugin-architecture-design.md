> Migrated from `docs/superpowers/specs/2026-06-10-memory-spi-plugin-architecture-design.md` on 2026-06-24.
> Owner: SDKWork maintainers

## 1. Purpose

This document upgrades SDKWork Memory from a provider-switchable contract skeleton into a runtime architecture that can support multiple implementations without changing public Open API, App API, or Backend API contracts.

The design separates stable Memory behavior from variable implementation choices. Stable behavior stays in the Memory core: tenant and scope enforcement, canonical records, evidence events, policy, audit, retrieval orchestration, context assembly, evaluation, migration gates, and contract verification. Variable behavior moves behind SPI ports and runtime plugins: SQL storage, event sourcing, keyword search, vector search, graph traversal, file search, model extraction, rerank, external memory bridges, and deployment-specific storage adapters.

The goal is a memory system that supports a practical no-embedding native SQL baseline while still allowing advanced industry patterns such as selective memory extraction, temporal knowledge graph retrieval, archival agent memory, vector store abstractions, GraphRAG-style graph indexes, and external memory providers.

## 2. Current Repository Review

The current repository is a contract and standards skeleton. It has no runtime `src/`, Rust workspace, service crates, route crates, or generated language SDK output. The authoritative artifacts are:

- `sdkwork.app.config.json`
- `specs/component.spec.json`
- `docs/superpowers/specs/2026-06-10-ai-memory-architecture-design.md`
- `docs/schema-registry/tables/*.yaml`
- `sdks/sdkwork-memory-sdk/openapi/memory-open-api.openapi.json`
- `sdks/sdkwork-memory-app-sdk/openapi/memory-app-api.openapi.json`
- `sdks/sdkwork-memory-backend-sdk/openapi/memory-backend-api.openapi.json`
- `tools/materialize_phase1_contracts.mjs`
- `tools/verify_phase1.ps1`

The current contract already contains the right raw materials:

- `mem_record` and `mem_event` are canonical source-of-truth tables.
- `mem_index`, `mem_index_entry`, `mem_retrieval_profile`, `mem_retrieval_trace`, and `mem_context_pack` represent derived retrieval and context state.
- `mem_implementation_profile`, `mem_provider_binding`, and `mem_policy` represent runtime selection and governance.
- Backend API exposes implementation profiles, provider bindings, indexes, retrieval profiles, provider health, migrations, eval runs, and audit logs.

The missing layer is an executable SPI and plugin contract that makes these profiles enforceable in runtime code and testable across implementations.

## 3. Industry Pattern Review

The current industry direction is not one implementation model. The best systems combine several patterns behind a stable memory interface:

- Selective extracted memory: systems such as Mem0 emphasize extracting, updating, and retrieving compact memories instead of replaying all chat history.
- Temporal knowledge graph memory: systems such as Zep/Graphiti treat entities, facts, relationships, and time as first-class retrieval state.
- Agent state memory: systems such as Letta keep memory blocks in active context and archival memory outside the context window.
- Framework stores: LangGraph and LangMem separate short-term thread state from long-term namespace stores and model semantic, episodic, and procedural memory explicitly.
- Vector store abstraction: Semantic Kernel and similar frameworks isolate vector storage and search behind provider-neutral abstractions.
- GraphRAG-style indexing: graph extraction and graph-local/global retrieval become a derived index, not the canonical memory store.

SDKWork Memory should not clone one of these systems. It should provide a stable platform core that can host these patterns as interchangeable plugins and evaluate them through the same contract suite.

### 3.1 Industry References

The first SPI version is based on the following industry evidence:

- LangGraph documents semantic, episodic, and procedural memory as separate long-term memory forms, and separates memory storage from agent logic through namespace/key stores.
- Mem0 positions memory as a persistent, self-improving layer for LLM applications and separates working, session, and long-term memory concerns.
- Zep and Graphiti use temporal knowledge graphs for agent memory, with entities, relationships, facts, time, and hybrid search as first-class retrieval state.
- Letta separates always-in-context memory blocks from archival memory that is queried on demand, and supports background memory management through sleep-time agents.
- MemGPT and MemoryOS use operating-system-inspired memory tiers and update/retrieval/generation flows to overcome fixed context windows.
- A-MEM shows that agentic memory benefits from structured notes, tags, keywords, links, and evolving memory networks instead of fixed record-only retrieval.

The implementation consequence is explicit: SDKWork Memory must not hard-code one memory engine. The core owns governed memory state transitions. Plugin ports host the interchangeable storage, retrieval, graph, vector, archival, background-learning, and external bridge patterns.

## 4. Design Principles

1. Canonical truth is stable.
   `mem_record`, `mem_event`, source links, audit logs, and deletion state are the durable authority. Derived indexes may be rebuilt or switched.

2. Embeddings are optional.
   Vector search is one retriever/index plugin. It must not be required for write, list, retrieve, delete, context assembly, or no-embedding retrieval.

3. Plugins are implementation details.
   Public OpenAPI schemas, SDK families, app services, and backend-admin consumers must not expose provider DTOs, raw provider ids, transport errors, or provider-specific credentials.

4. Core owns governance.
   Plugins may propose, retrieve, rank, index, or bridge. Core enforces tenant scope, organization scope, user scope, policy, sensitivity, retention, deletion, audit, and request context.

5. Profiles configure composition.
   `mem_implementation_profile`, `mem_provider_binding`, and typed runtime config select plugins and provider bindings. Code forks must not select behavior implicitly.

6. Conformance beats trust.
   Every plugin that claims a port must pass the same Conformance suite for that port, including negative tests for tenant isolation, deletion, degraded mode, and secret redaction.

7. External bridges are shadowed.
   External memory engines may participate, but SDKWork keeps enough local shadow state for source tracing, export, deletion, audit, and migration.

## 5. Stable Core And Plugin Boundaries

The stable core is the Memory product runtime. It owns business invariants and cross-plugin orchestration. Plugins are replaceable runtime capabilities.

Stable core responsibilities:

- Resolve `MemoryScopeContext` from typed appbase request context.
- Authorize every read, write, review, retrieval, export, and admin operation.
- Validate OpenAPI DTOs and map errors to Problem Details.
- Own canonical memory lifecycle: event append, candidate creation, record creation, update, supersession, deletion, and source linking.
- Enforce policy: sensitivity, retention, learning, review, retrieval, provider, export, and deletion.
- Orchestrate retrieval: select retrievers, apply pre-filters, call plugins, rehydrate canonical records, recheck policy, fuse scores, assemble context, write trace/hits.
- Own deletion propagation state and fail closed when a plugin cannot prove deletion or suppression.
- Emit audit records and outbox events.
- Expose health and capability summaries.
- Run migration, shadow read, dual write, and eval gates.

Plugin responsibilities:

- Provide one or more SPI ports declared in a manifest.
- Validate plugin-local config.
- Accept only typed SDKWork commands, never raw HTTP requests or raw credential headers.
- Return provider-neutral results and typed plugin errors.
- Declare degraded behavior and unsupported capability paths.
- Redact secrets and sensitive payloads from logs, traces, health checks, and errors.
- Prove conformance through tests.

Forbidden plugin behavior:

- Bypassing canonical scope, policy, deletion, or audit checks.
- Replacing appbase request context with plugin-local credential parsing.
- Mutating public OpenAPI or generated SDK output.
- Storing live API keys, tokens, passwords, private keys, or provider secrets in plugin manifests or `config_json`.
- Treating external provider state as sufficient for SDKWork export, deletion, or audit requirements without local shadow records.
- Returning deleted, unauthorized, or policy-suppressed records because a derived index is stale.

## 6. Runtime Composition Model

Runtime composition starts with an active implementation profile and a plugin registry.

```text
typed runtime config
  -> plugin discovery
  -> manifest validation
  -> provider binding resolution
  -> implementation profile resolution
  -> MemoryCoreRuntime builder
  -> port conformance preflight
  -> health and readiness
  -> request handlers call product services
```

`MemoryCoreRuntime` is the stable composition object for one process/runtime profile. It wires the core services to plugin-provided ports.

```ts
export interface MemoryRuntimePlugin {
  manifest(): MemoryPluginManifest;
  validateConfig(command: ValidateMemoryPluginConfigCommand): Promise<MemoryPluginConfigValidation>;
  buildPorts(command: BuildMemoryPluginPortsCommand): Promise<MemoryPluginPorts>;
  preflight(command: MemoryPluginPreflightCommand): Promise<MemoryPluginPreflightResult>;
  health(command: MemoryPluginHealthCommand): Promise<MemoryPluginHealth>;
  start(command: StartMemoryPluginCommand): Promise<void>;
  drain(command: DrainMemoryPluginCommand): Promise<void>;
  stop(command: StopMemoryPluginCommand): Promise<void>;
}

export interface MemoryCoreRuntime {
  profile: ResolvedMemoryImplementationProfile;
  registry: MemoryPluginRegistry;
  stores: MemoryStorePorts;
  retrievers: MemoryRetrieverRegistry;
  indexes: MemoryIndexRegistry;
  providers: MemoryProviderRegistry;
  policy: MemoryPolicyEngine;
  learning: MemoryLearningEngine;
  context: MemoryContextAssembler;
  evaluation: MemoryEvaluationRuntime;
  health(): Promise<MemoryRuntimeHealth>;
  shutdown(): Promise<void>;
}
```

Runtime profile resolution must fail before serving traffic when:

- The primary implementation plugin is missing.
- A required port is missing.
- A provider binding references an unknown plugin or unsupported capability.
- A same-process plugin lacks executable integration entrypoints.
- A plugin config contains secrets instead of secret references.
- A migration or external bridge requires shadow/audit support but the plugin manifest does not provide it.

## 7. SPI Port Families

The SPI is split into small port families. A plugin can implement one port, several ports, or a full implementation bundle.

### 7.1 Required Core Ports

Required ports are present in every runnable profile.

```ts
export interface MemoryRecordStorePort {
  create(command: CreateMemoryRecordCommand): Promise<MemoryRecord>;
  retrieve(query: RetrieveMemoryRecordQuery): Promise<MemoryRecord | null>;
  list(query: ListMemoryRecordsQuery): Promise<Page<MemoryRecord>>;
  update(command: UpdateMemoryRecordCommand): Promise<MemoryRecord>;
  supersede(command: SupersedeMemoryRecordCommand): Promise<MemoryRecord>;
  markDeleted(command: DeleteMemoryRecordCommand): Promise<MemoryDeletionReceipt>;
}

export interface MemoryEventStorePort {
  append(command: AppendMemoryEventCommand): Promise<MemoryEvent>;
  retrieve(query: RetrieveMemoryEventQuery): Promise<MemoryEvent | null>;
  list(query: ListMemoryEventsQuery): Promise<Page<MemoryEvent>>;
}

export interface MemoryAuditStorePort {
  append(command: AppendMemoryAuditCommand): Promise<MemoryAuditRecord>;
  list(query: ListMemoryAuditQuery): Promise<Page<MemoryAuditRecord>>;
}

export interface MemoryOutboxStorePort {
  append(command: AppendMemoryOutboxCommand): Promise<MemoryOutboxEvent>;
  retrieve(query: RetrieveMemoryOutboxQuery): Promise<MemoryOutboxEvent | null>;
  listPending(query: ListPendingMemoryOutboxQuery): Promise<Page<MemoryOutboxEvent>>;
  markPublished(command: MarkMemoryOutboxPublishedCommand): Promise<MemoryOutboxEvent>;
  markFailed(command: MarkMemoryOutboxFailedCommand): Promise<MemoryOutboxEvent>;
}

export interface MemoryPolicyStorePort {
  resolvePolicy(query: ResolveMemoryPolicyQuery): Promise<ResolvedMemoryPolicy>;
  upsertPolicy(command: UpsertMemoryPolicyCommand): Promise<MemoryPolicy>;
}
```

These ports must be implemented by a trusted local plugin for L3 deployments. External-only implementations are not sufficient because Memory must enforce deletion, retention, audit, and tenant isolation locally.

### 7.2 Learning Ports

Learning ports generate reviewable candidates and habit signals. They do not directly commit stable memory facts.

```ts
export interface MemoryExtractionPort {
  extract(command: ExtractMemoryCandidatesCommand): Promise<MemoryCandidateDraft[]>;
}

export interface MemoryCandidateStorePort {
  create(command: CreateMemoryCandidateCommand): Promise<MemoryCandidate>;
  decide(command: DecideMemoryCandidateCommand): Promise<MemoryDecisionResult>;
  list(query: ListMemoryCandidatesQuery): Promise<Page<MemoryCandidate>>;
}

export interface MemoryHabitLearnerPort {
  observe(command: ObserveHabitSignalCommand): Promise<MemoryHabitSignalResult>;
  score(command: ScoreHabitCommand): Promise<MemoryHabitScore>;
  promote(command: PromoteHabitCommand): Promise<MemoryHabitPromotionResult>;
  decay(command: DecayHabitCommand): Promise<MemoryHabitDecayResult>;
}
```

Rules:

- Extraction can use deterministic rules, LLMs, imported provider memories, or hybrid logic.
- Candidate approval, rejection, supersession, and deletion remain core-governed.
- Habit activation must pass current policy and current request context, not only historical habit strength.

### 7.3 Retrieval Ports

Retrieval ports return candidate hits. Core rehydrates and validates canonical records before context assembly.

```ts
export interface MemoryRetrieverPort {
  retrieverCode(): string;
  retrieverKind(): MemoryRetrieverKind;
  capabilities(): MemoryRetrieverCapabilities;
  retrieve(command: RetrieveMemoryCandidatesCommand): Promise<MemoryRetrieverResult>;
}

export type MemoryRetrieverKind =
  | "sql"
  | "keyword"
  | "dictionary"
  | "time"
  | "event"
  | "vector"
  | "graph"
  | "grep_file"
  | "external"
  | "custom";
```

Core retrieval flow:

1. Resolve profile and policy.
2. Select retrievers from `mem_retrieval_profile`.
3. Apply scope and sensitivity pre-filters.
4. Call retriever plugins with bounded request objects.
5. Rehydrate canonical records.
6. Recheck authorization, deletion, sensitivity, and retention.
7. Fuse, rerank, explain, and assemble context.
8. Persist trace and hits.

Retrievers must tolerate stale derived indexes but must not treat stale index entries as canonical truth.

### 7.4 Index Ports

Index ports build, rebuild, repair, and delete derived index state.

```ts
export interface MemoryIndexPort {
  indexKind(): MemoryIndexKind;
  index(command: IndexMemoryCommand): Promise<MemoryIndexReceipt>;
  remove(command: RemoveMemoryIndexCommand): Promise<MemoryIndexReceipt>;
  rebuild(command: RebuildMemoryIndexCommand): AsyncIterable<MemoryIndexRebuildProgress>;
  health(command: MemoryIndexHealthCommand): Promise<MemoryIndexHealth>;
}
```

Vector, graph, search, and file indexes are optional. A profile may run without any optional indexes.

### 7.5 Provider Ports

Provider ports isolate LLMs, embeddings, rerankers, graph engines, search engines, file engines, and external memory engines.

```ts
export interface LanguageModelPort {
  providerCode(): string;
  capabilities(): LanguageModelCapabilities;
  generate(command: LanguageModelCommand): Promise<LanguageModelResult>;
}

export interface EmbeddingModelPort {
  providerCode(): string;
  embeddingSpace(): MemoryEmbeddingSpace;
  embed(command: EmbeddingCommand): Promise<EmbeddingResult>;
}

export interface RerankModelPort {
  providerCode(): string;
  rerank(command: RerankMemoryHitsCommand): Promise<RerankMemoryHitsResult>;
}

export interface ExternalMemoryBridgePort {
  providerCode(): string;
  import(command: ExternalMemoryImportCommand): Promise<ExternalMemoryImportResult>;
  export(command: ExternalMemoryExportCommand): Promise<ExternalMemoryExportResult>;
  delete(command: ExternalMemoryDeleteCommand): Promise<ExternalMemoryDeletionReceipt>;
  shadowRead(command: ExternalMemoryShadowReadCommand): Promise<ExternalMemoryShadowReadResult>;
}
```

Provider-specific SDKs, HTTP clients, credentials, retries, and rate limits live inside provider adapters. Shared Memory services see only these ports.

### 7.6 Context And Evaluation Ports

Context assembly and evaluation can be customized, but core controls final policy and audit.

```ts
export interface MemoryContextAssemblerPort {
  assemble(command: AssembleMemoryContextCommand): Promise<MemoryContextPackDraft>;
}

export interface MemoryEvaluationPort {
  run(command: RunMemoryEvalCommand): Promise<MemoryEvalRunResult>;
}
```

Default context assembly must work without LLMs. Optional LLM-based summarization is a plugin capability that cannot hide citations, confidence, or policy decisions.

## 8. Runtime Plugin Manifest

Every runtime plugin declares a `MemoryPluginManifest`. This manifest is source-controlled with the plugin package and can also be exposed in backend-admin health/capability summaries.

```ts
export interface MemoryPluginManifest {
  schemaVersion: 1;
  kind: "sdkwork.memory.plugin";
  pluginId: string;
  packageName: string;
  displayName: string;
  version: string;
  owner: string;
  implementationKinds: MemoryImplementationKind[];
  pluginRoles: MemoryPluginRole[];
  deploymentModes: MemoryDeploymentMode[];
  portExports: MemoryPluginPortExport[];
  providerKinds: MemoryProviderKind[];
  retrieverKinds: MemoryRetrieverKind[];
  indexKinds: MemoryIndexKind[];
  requiredCoreVersion: string;
  configSchemaRef?: string;
  secretRefs: MemoryPluginSecretRef[];
  dataClasses: MemoryPluginDataClass[];
  capabilities: MemoryPluginCapabilities;
  degradation: MemoryPluginDegradationPolicy;
  migration: MemoryPluginMigrationCapabilities;
  observability: MemoryPluginObservabilityContract;
  conformance: MemoryPluginConformanceContract;
}
```

Required manifest rules:

- `pluginId` uses lowercase kebab-case and starts with `sdkwork-memory-plugin-` for SDKWork-owned plugins.
- `kind` is exactly `sdkwork.memory.plugin`.
- `secretRefs` may name required secret references but must never contain secret values.
- `portExports` must name executable Rust package-root builders or approved service builders.
- Provider DTO schemas must not be referenced by public OpenAPI schemas.
- `conformance` must declare the required test suite version and supported test fixtures.

Example:

```json
{
  "schemaVersion": 1,
  "kind": "sdkwork.memory.plugin",
  "pluginId": "sdkwork-memory-plugin-native-sql",
  "packageName": "sdkwork-memory-plugin-native-sql",
  "displayName": "SDKWork Memory Native SQL Plugin",
  "version": "0.1.0",
  "owner": "sdkwork-memory",
  "implementationKinds": ["native_sql", "local_embedded"],
  "pluginRoles": ["implementation", "store", "retriever", "index"],
  "deploymentModes": ["server", "container", "private", "local", "test"],
  "portExports": [
    {
      "port": "MemoryRecordStorePort",
      "builder": "build_native_sql_record_store"
    },
    {
      "port": "MemoryEventStorePort",
      "builder": "build_native_sql_event_store"
    },
    {
      "port": "MemoryAuditStorePort",
      "builder": "build_native_sql_audit_store"
    },
    {
      "port": "MemoryOutboxStorePort",
      "builder": "build_native_sql_outbox_store"
    }
  ],
  "providerKinds": [],
  "retrieverKinds": ["sql", "keyword", "dictionary", "time", "event"],
  "indexKinds": ["sql", "keyword", "dictionary", "time", "event"],
  "requiredCoreVersion": "0.1.0",
  "secretRefs": [],
  "capabilities": {
    "canonicalStore": true,
    "eventLog": true,
    "candidateLifecycle": true,
    "habitLearning": true,
    "deletionPropagation": true,
    "auditLog": true,
    "outboxLog": true,
    "embeddingRequired": false
  },
  "degradation": {
    "mode": "fail_required_degrade_optional",
    "returnsStaleHits": false
  },
  "migration": {
    "exportSupported": true,
    "importSupported": true,
    "dualWriteSupported": false,
    "shadowReadSupported": true
  },
  "observability": {
    "metricsPrefix": "sdkwork_memory_native_sql",
    "redactsPayloads": true
  },
  "conformance": {
    "suite": "sdkwork-memory-plugin-conformance",
    "suiteVersion": "0.1.0"
  }
}
```

## 9. Plugin Lifecycle

Plugins must support a predictable lifecycle.

```text
discover
validate_manifest
load_config_schema
resolve_provider_bindings
validate_config
build_ports
run_preflight
start
serve
health_check
drain
stop
```

Migration-capable plugins add:

```text
export_snapshot
import_snapshot
dual_write
shadow_read
compare
cut_over
rollback
```

Lifecycle rules:

- Discovery must be deterministic and based on package manifests, compiled feature lists, or configured plugin directories. It must not scan arbitrary user-writable directories in production.
- Runtime loading must fail closed when a required plugin is missing.
- Dynamic loading is optional. Static compile-time registration is acceptable for the first Rust implementation and safer for L3 deployments.
- A plugin can be disabled only when it does not own required canonical ports for the active primary profile.
- Health checks must report stable status and safe error classes, not raw provider errors or secrets.
- Shutdown must drain in-flight indexing, migration, and provider calls when possible.

## 10. Built-In Plugin Families

### 10.1 `native_sql`

Purpose:

- Default MVP and production baseline.
- PostgreSQL server/container profile.
- SQLite local/private/test profile where feasible.

Ports:

- Required core stores.
- SQL, keyword, dictionary, time, and event retrievers.
- Basic index maintenance.
- Audit and outbox stores.

Rules:

- Must pass full no-embedding conformance.
- Must not require vector extensions.
- Must be the first complete landing target.

### 10.2 `event_sourced`

Purpose:

- Strong replay, audit, rebuild, and historical projection support.

Ports:

- Event store as the primary write authority.
- Projection rebuild ports.
- Record projections and retrieval traces.

Rules:

- Must prove projection rebuild from `mem_event`.
- May use `mem_record` as projection or canonical snapshot, but event evidence remains required.

### 10.3 `search_first`

Purpose:

- High-volume text memory, logs, and document-like retrieval.

Ports:

- Search index port.
- Keyword/BM25 retriever.
- Optional rerank bridge.

Rules:

- Search index is derived.
- Results must rehydrate canonical records before returning.
- OpenSearch, Elasticsearch, Tantivy, Lucene, or PostgreSQL FTS are adapters behind the same port.

### 10.4 `graph_temporal`

Purpose:

- Relationship-heavy memory and temporal facts.

Ports:

- Entity and edge projection index.
- Graph retriever.
- Optional temporal knowledge graph provider.

Rules:

- Relational `mem_entity` and `mem_edge` are the first portable graph model.
- External graph databases are optional providers.
- Graph output must be explainable through memory ids, entity ids, edge ids, and valid time ranges.

### 10.5 `local_embedded`

Purpose:

- Desktop, local, private, and test deployments.

Ports:

- SQLite store.
- SQLite FTS, dictionary, time, event, and grep/file retrievers.
- Local runtime config and user-private paths.

Rules:

- Must not require SaaS provider calls.
- Must keep secrets and local state outside source-controlled `.sdkwork/`.
- Must preserve API parity with server/container profiles where those APIs are served.

### 10.6 `external_provider_bridge`

Purpose:

- Connect to external memory engines such as selective memory services, temporal graph memory services, or enterprise memory products.

Ports:

- External import/export/delete/shadow-read bridge.
- Optional retriever.
- Optional extraction provider.

Rules:

- Must maintain SDKWork local shadow state for records, events, audit, deletion, and export.
- Must pass deletion propagation tests.
- Must not allow provider-specific DTOs into public SDKWork API schemas.
- Must support fail-closed behavior when deletion propagation, export, or policy proof is missing.

### 10.7 `hybrid_platform`

Purpose:

- Final platform profile combining SQL canonical store, optional event sourcing, search, vector, graph, file, and provider bridges.

Ports:

- All required core ports.
- Multiple optional retrieval/index/provider ports.
- Eval and migration orchestration.

Rules:

- Must use profile-driven selection, not hard-coded plugin order.
- Must run shadow eval before switching retrieval profiles or provider routes.

## 11. Plugin Package Layout

Runtime plugins are not Codex agent plugins. Do not place runtime Memory plugins under `.sdkwork/plugins/`.

Recommended first implementation layout:

```text
crates/
  sdkwork-memory-contract/
  sdkwork-memory-spi/
  sdkwork-memory-runtime/
  sdkwork-memory-core/
  sdkwork-memory-test-support/
plugins/
  sdkwork-memory-plugin-native-sql/
    Cargo.toml
    sdkwork.memory.plugin.json
    src/lib.rs
    src/manifest.rs
    src/stores.rs
    src/retrievers/
    tests/conformance_native_sql.rs
  sdkwork-memory-plugin-local-embedded/
  sdkwork-memory-plugin-search-opensearch/
  sdkwork-memory-plugin-graph-relational/
  sdkwork-memory-plugin-external-bridge/
services/
  sdkwork-memory-product/
  sdkwork-memory-service/
packages/native-rust/routes/
  open-api/sdkwork-routes-memory-open-api/
  app-api/sdkwork-routes-memory-app-api/
  backend-api/sdkwork-routes-memory-backend-api/
```

The repository can start with static Rust feature registration:

```rust
pub fn register_builtin_memory_plugins(registry: &mut MemoryPluginRegistry) {
    sdkwork_memory_plugin_native_sql::register(registry);
}
```

Dynamic plugin loading can be introduced later only after security, signing, compatibility, and deployment policy are defined.

## 12. Profiles, Provider Bindings, And Config

The database-backed profile tables remain the control-plane model.

- `mem_implementation_profile` selects the active implementation kind, role, status, capabilities, rollout, and plugin config references.
- `mem_provider_binding` selects provider kind, provider code, endpoint ref, secret ref, model ref, capabilities, health, and safe non-secret config.
- `mem_retrieval_profile` selects retriever list, fusion policy, rerank policy, top K, token budget, and degraded-mode behavior.
- `mem_policy` selects retention, privacy, review, learning, retrieval, provider, export, and deletion behavior.

Runtime config may include plugin enablement and safe paths:

```ts
export interface MemoryPluginRuntimeConfig {
  enabledPluginIds: string[];
  builtinOnly: boolean;
  pluginManifestPaths?: string[];
  defaultImplementationProfileId?: string;
  testMode?: boolean;
}
```

Rules:

- Runtime config must not contain live tokens, API keys, passwords, private keys, provider secrets, or raw credential DTOs.
- `secret_ref` points to secret manager or secure runtime storage.
- Browser-visible config must not include plugin internals or provider secrets.
- Tenant-specific profile selection is data/config, not code.

## 13. Error Handling And Degraded Mode

Plugin errors map to stable core error classes:

```text
plugin_unavailable
plugin_unsupported_capability
plugin_config_invalid
plugin_health_degraded
plugin_timeout
plugin_rate_limited
plugin_policy_denied
plugin_delete_unverified
plugin_external_consistency_unknown
plugin_internal_error
```

Rules:

- Required core store failure returns an API error.
- Optional retriever failure follows retrieval profile policy: fail, degrade with explanation, or skip.
- Degraded retrieval must be recorded in `mem_retrieval_trace`.
- External provider deletion uncertainty must block cutover and surface as an operator-visible migration/eval failure.
- Raw provider errors, SQL, stack traces, payloads, object keys, signed URLs, and secrets must not appear in API responses or normal logs.

## 14. Conformance And Verification

The Conformance suite is the executable contract for plugins.

Conformance groups:

- Manifest validation.
- Required port presence.
- Store CRUD and optimistic concurrency.
- Tenant, organization, user, owner, and data-scope isolation.
- Event append idempotency.
- Candidate lifecycle and decision state transitions.
- Habit signal, promotion, rejection, and decay behavior.
- No-embedding retrieval.
- Derived index rebuild and stale index suppression.
- Retrieval trace and hit explainability.
- Context assembly with citations and token budget.
- Deletion and suppression propagation.
- Provider health and degraded mode.
- Audit and outbox event creation.
- Migration export/import/shadow-read where claimed.
- Secret redaction and safe observability labels.

First required verification commands:

```powershell
node tests/contracts/spi_design_contract_test.mjs
node tools/materialize_phase1_contracts.mjs
powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1
```

Future runtime verification commands:

```powershell
cargo test -p sdkwork-memory-spi
cargo test -p sdkwork-memory-runtime
cargo test -p sdkwork-memory-plugin-native-sql --test conformance_native_sql
cargo test --workspace
```

Every plugin must declare the conformance suite version it passed. A plugin that has not passed the required suite can be used only in explicit `test` or `eval_only` profiles.

## 15. API And Schema Impact

The first SPI landing phase should not change Open API, App API, or Backend API paths.

Existing backend-admin resources are enough to configure early SPI behavior:

- `implementationProfiles.*`
- `providerBindings.*`
- `retrievalProfiles.*`
- `indexes.*`
- `providerHealth.retrieve`
- `migrationJobs.*`
- `evalRuns.*`
- `auditLogs.list`

Potential future additive backend-admin APIs:

- `GET /backend/v3/api/memory/plugins`
- `GET /backend/v3/api/memory/plugins/{pluginId}`
- `POST /backend/v3/api/memory/plugins/{pluginId}/validate`
- `POST /backend/v3/api/memory/plugins/{pluginId}/conformance_runs`

Do not add these until runtime plugin manifests exist and the backend-admin API has a concrete operator workflow. For the first landing phase, plugin manifests can be validated by repository tests and surfaced through provider health metadata.

## 16. Implementation Roadmap

### Phase A: SPI Authority

Deliver:

- This design spec.
- Materializer reference from `specs/README.md`.
- Contract test proving the SPI design is present and materialized.

### Phase B: Rust SPI And Runtime Skeleton

Deliver:

- `crates/sdkwork-memory-contract`
- `crates/sdkwork-memory-spi`
- `crates/sdkwork-memory-runtime`
- `crates/sdkwork-memory-test-support`
- Manifest structs.
- Port traits.
- Plugin registry.
- Runtime profile resolver.
- Conformance test harness skeleton.

### Phase C: Native SQL Plugin

Deliver:

- `plugins/sdkwork-memory-plugin-native-sql`
- PostgreSQL and SQLite migrations from schema registry.
- Record, event, candidate, habit, policy, audit, and outbox stores.
- SQL, keyword, dictionary, time, and event retrievers.
- No-embedding conformance tests.

### Phase D: Product Services And Routes

Deliver:

- `services/sdkwork-memory-product`
- `services/sdkwork-memory-service`
- Open API, App API, and Backend API Rust route crates.
- Typed request-context consumption.
- Smoke tests for API key and dual-token contexts using approved test resolvers.

### Phase E: Optional Retrieval Plugins

Deliver:

- Search plugin.
- Vector plugin.
- Rerank plugin.
- Graph relational plugin.
- Grep/file plugin.
- Retrieval profile eval gates.

### Phase F: External Bridge Plugins

Deliver:

- External provider bridge SPI.
- One approved bridge adapter in `eval_only` or `shadow` mode first.
- Import/export/delete/shadow-read conformance tests.
- Privacy and deletion propagation review.

## 17. 0.1.0 Implementation Decisions

These decisions are fixed for the first runtime landing so implementation can start without further product ambiguity.

1. Static Rust registration is the only 0.1.0 plugin loading mode.
   Dynamic shared-library loading, marketplace installation, remote plugin loading, and user-writable plugin directories are out of scope until signing, supply-chain, compatibility, and deployment policy are defined.

2. `local_embedded` starts as a capability/profile inside `sdkwork-memory-plugin-native-sql`.
   A separate `sdkwork-memory-plugin-local-embedded` package may be split later only when desktop/local filesystem ownership, package release, and runtime directory policy require it.

3. Plugin manifests use JSON manifest plus Rust constant.
   Each runtime plugin keeps a source-controlled `sdkwork.memory.plugin.json` for tooling and exposes the same manifest through a Rust constant/function for static registration tests. The Rust constant must be generated from or verified against the JSON manifest before release.

4. The first external bridge is generic and `eval_only`.
   No Mem0, Zep/Graphiti, Letta, or other provider becomes a first-class production bridge in 0.1.0. The external bridge SPI is defined now, but production activation waits for native SQL conformance, deletion propagation tests, privacy review, and provider-specific SDK/security review.

5. Plugin conformance results are stored in `mem_eval_run` for 0.1.0.
   A dedicated `mem_plugin_conformance_run` table is deferred until conformance history needs lifecycle, retention, or operator workflow fields that do not fit eval runs.

6. Backend plugin-list APIs wait until native SQL runtime manifests exist.
   Phase B validates manifests through repository and Rust tests. Backend-admin plugin listing is an additive API after runtime manifests, health summaries, and operator workflows are concrete.

7. Native SQL is the first complete implementation target.
   `native_sql` must pass no-embedding conformance before optional vector, graph, search-engine, file, or external bridge plugins can become production profiles.

8. The SPI crate must be provider-neutral and framework-neutral.
   Port traits may use async Rust, typed DTOs, and typed errors, but must not depend on HTTP framework types, generated SDKs for the same authority, raw request headers, or provider-specific SDK DTOs.

## 18. Acceptance Criteria

1. Core API contracts remain stable while implementation profile changes.
2. No-embedding native SQL profile can run without LLM, embedding, vector, graph, search, or external memory providers.
3. Optional retrievers can be added or removed without changing canonical memory records.
4. Every plugin exposes a manifest and passes its declared conformance suite.
5. External bridge plugins keep local shadow records sufficient for deletion, export, audit, and source tracing.
6. Provider secrets are referenced through secure secret refs and never stored in manifests, app config, OpenAPI, generated SDKs, logs, or `.sdkwork/`.
7. Retrieval always rehydrates canonical records and rechecks policy before context assembly.
8. Migrations and provider switches produce evidence before cutover.
9. Static scans can distinguish runtime Memory plugins under `plugins/` from agent workspace plugins under `.sdkwork/plugins/`.
10. SDKWork Phase 1 verification continues to pass after the SPI design is added.


# SDKWork Memory PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-memory
Updated: 2026-06-26
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Add `PRD-<topic>.md` shards in this directory when the PRD grows beyond one reviewable screen.

## 1. Background And Problem

Modern AI assistants suffer from session-scoped amnesia: every conversation starts cold,
re-asking context the user already provided. Persistent memory frameworks exist but force
embedding-vector databases, cloud-only deployment, or vendor-locked retrieval pipelines.

SDKWork Memory solves this by providing an **embedding-optional, provider-switchable,
tenant-isolated** memory service that:

- Stores episodic events, canonical records, entities, and edges in a relational database.
- Supports pluggable retrieval providers (native SQL FTS, Drive-backed, external embedding
  services) via the SDKWork SPI.
- Ships as a standalone Rust service deployable on Kubernetes, Docker, or bare metal.
- Exposes three API surfaces (open / app / backend) with dual-token authentication.

The service is the memory backbone for the SDKWork application family and is designed for
commercial multi-tenant deployment.

## 2. Target Users

| User | Description | Primary Need |
|---|---|---|
| SDKWork application developers | Teams building AI assistants on the SDKWork platform | Reliable, tenant-scoped memory API with SDK clients |
| Platform operators | DevOps/SRE teams running SDKWork cloud or on-prem | Production-grade deployment, observability, and tenant isolation |
| Integrators | Third-party vendors embedding memory into their products | Open API with stable contract, SDK generation, and provider extension |
| End users | Users of AI assistants powered by SDKWork Memory | Privacy controls: view, export, delete, opt-out of learning |

## 3. Goals And Non-Goals

### Goals

- **G1**: Provide a multi-tenant memory service with space-isolated access and audit trails.
- **G2**: Support embedding-optional retrieval — keyword FTS works without a vector backend.
- **G3**: Enable provider switching (native SQL, Drive, external embedding) via SPI plugins
  without recompiling the service.
- **G4**: Deliver three API surfaces (open / app / backend) with generated SDK clients.
- **G5**: Achieve production-grade security: dual-token auth, PII detection, hard delete,
  tenant quotas, and fail-closed access control.
- **G6**: Ship Kubernetes deployment assets (Deployment, HPA, PDB, migration Job,
  ServiceMonitor, Ingress with TLS).
- **G7**: Commercial memory management: subjects, bindings, capabilities, policy assignments,
  and commercial readiness evaluation.

### Non-Goals

- **NG1**: Not building an embedding/vector database — embeddings are delegated to providers.
- **NG2**: Not providing mobile or third-party embedded UI; the repository-owned PC application provides the supported Console and Admin experiences.
- **NG3**: Not replacing IAM — authentication and tenant identity are delegated to SDKWork IAM.
- **NG4**: Not supporting multi-region active-active in Phase 1 — single-region HA only.

## 4. Scope

### In Scope (Phase 1 — Production Ready)

- Memory core: spaces, events, records, record sources, entities, edges.
- Retrieval: keyword FTS (PostgreSQL tsvector + SQLite FTS5), context pack fusion.
- Learning: candidate extraction, habit signals, async learning job queue.
- Governance: audit log, outbox events, tenant preferences, tenant quotas.
- Provider binding: native SQL plugin, reference profiles plugin, SPI registry.
- API: open (17 ops), app (33 ops), backend (55 ops) with generated TypeScript SDK.
- PC application: customer-facing Console and internal Admin surfaces with strict App SDK and Backend SDK isolation.
- Deployment: Docker and Kubernetes through the standard `standalone` and `cloud` profiles.
- Observability: Prometheus metrics, tracing, readiness/liveness probes, correlation IDs.
- Privacy: soft delete, hard delete, forget requests, export jobs, sensitivity classification.

### Out of Scope (Phase 2 — Commercial Extension)

- Commercial memory extensions beyond Phase 2a: policy assignments, entity/edge CRUD APIs,
  relation rebuild jobs, and commercial readiness snapshots remain planned for a later release.
  Phase 2a (subjects, bindings, capability bindings, capability resolution) is implemented
  and exposed on the backend API with IAM route-manifest enforcement.
- Plugin hot-loading and version negotiation.
- Multi-region active-active deployment.
- Embedding-backed semantic retrieval (provider contract defined, no reference implementation).

## 5. User Scenarios

### Scenario 1: Developer creates a memory and retrieves it

1. Developer obtains an auth token and access token from SDKWork IAM.
2. POST `/app/v3/api/memory/spaces.create` to create a space.
3. POST `/app/v3/api/memory/memories.create` with event text to store a memory.
4. POST `/app/v3/api/memory/retrievals.create` with a query to retrieve relevant memories.
5. GET `/app/v3/api/memory/memories.list` to paginate through stored memories.

### Scenario 2: End user exercises privacy controls

1. User authenticates via the consuming app (which holds the auth token).
2. POST `/app/v3/api/memory/forget-requests.create` to request deletion of their memories.
3. POST `/app/v3/api/memory/export-jobs.create` to request an export of their data.
4. The backend processes the forget request (hard delete + derivative cleanup) and writes
   an audit event.

### Scenario 3: Operator deploys to Kubernetes

1. Operator runs the migration Job: `kubectl apply -f deployments/kubernetes/migration-job.yaml`.
2. Operator applies the Deployment, Service, HPA, PDB, Ingress, ServiceMonitor.
3. The startup probe waits for `/readyz` to return 200 (database + IAM readiness).
4. The HPA scales based on CPU (70%) and memory (80%) utilization.
5. Prometheus scrapes `/metrics` via the ServiceMonitor.

### Scenario 4: Integrator extends with a custom retrieval provider

1. Integrator implements the `MemoryRetrievalProviderPort` trait in a custom plugin crate.
2. The plugin declares `sdkwork.memory.plugin.json` with `portExports` and `dataClasses`.
3. The plugin is registered in the SPI registry at bootstrap.
4. The service routes retrieval requests to the plugin based on provider binding configuration.

## 6. Success Metrics

| Metric | Target | Measurement |
|---|---|---|
| API availability | 99.9% monthly | Kubernetes liveness probe + uptime monitoring |
| p99 read latency | < 200ms | Prometheus histogram on retrieval operations |
| p99 write latency | < 500ms | Prometheus histogram on create operations |
| Tenant isolation | 0 cross-tenant access events | Access control fail-closed + audit log review |
| Migration rollback | 100% of migrations have paired down.sql | CI layout validation |
| SDK contract drift | 0 breaking changes without SemVer bump | CI OpenAPI diff check |
| Privacy SLA | Forget requests completed within 72h | Audit log timestamp delta |

## 7. Phases

| Phase | Name | Status | Scope |
|---|---|---|---|
| Phase 1 | Production Ready | Active | Memory core, FTS retrieval, governance, deployment, observability, privacy |
| Phase 2 | Commercial Memory | Designed | Subjects, bindings, capabilities, policy assignments, commercial readiness |
| Phase 3 | Semantic Retrieval | Planned | Embedding-backed retrieval via provider plugin, multi-region HA |

## 8. Linked Requirements

- REQ-MEM-001: Multi-tenant memory storage with space isolation
- REQ-MEM-002: Embedding-optional keyword retrieval (PostgreSQL + SQLite)
- REQ-MEM-003: SPI plugin architecture for provider switching
- REQ-MEM-004: Three API surfaces with generated SDK (open / app / backend)
- REQ-MEM-005: Dual-token authentication and fail-closed access control
- REQ-MEM-006: Privacy controls (view, export, delete, opt-out)
- REQ-MEM-007: Kubernetes deployment with HPA, PDB, migration Job
- REQ-MEM-008: Prometheus metrics and structured tracing
- REQ-MEM-009: Commercial memory management (Phase 2)
- REQ-MEM-010: Hard delete and derivative cleanup for GDPR compliance

> REQ-* records will be created in `docs/product/requirements/` as they are formalized.

## 9. Open Questions

- **OQ-1**: Should the Phase 2 commercial memory tables use the `ai_` prefix (consistent with
  Phase 1) or introduce a `mem_commercial_` prefix? Current decision: use `ai_` prefix for
  consistency.
- **OQ-2**: Should plugin hot-loading be supported in Phase 2, or deferred to Phase 3?
  Current decision: deferred — static registry only in Phase 1/2.
- **OQ-3**: Should the outbox publisher use a lease-lock column for multi-instance
  deduplication? Current decision: yes — add `locked_at`/`locked_by` columns.
- **OQ-4**: Should the service support SQLite for production single-node deployments?
  Current decision: no — production MUST use PostgreSQL per ENVIRONMENT_SPEC §7.1.

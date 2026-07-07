> Migrated from `docs/superpowers/specs/2026-06-10-commercial-memory-management-design.md` on 2026-06-24.
> Owner: SDKWork maintainers

Date: 2026-06-10
Status: Active — aligned with implementation as of 2026-07-07 (production hardening)

## 1. Purpose

This document extends SDKWork Memory from a Phase 1 contract/runtime skeleton into a commercial memory management platform design.

The design focuses on the missing product capability called out by review:

- Memory must have explicit relationships with agents.
- Memory must have explicit relationships with users.
- Memory must have explicit relationships with entities and business objects.
- Memory capabilities must be attachable through governed data relationships.
- Open API, App API, and Backend API must expose complete management contracts.
- Operators, tenant admins, applications, and external API clients must be able to inspect, govern, evaluate, and audit memory behavior.

This document does not replace:

- `docs/architecture/tech/TECH-2026-06-10-ai-memory-architecture-design.md`
- `docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md`
- `../sdkwork-specs/API_SPEC.md`
- `../sdkwork-specs/SDK_SPEC.md`
- `../sdkwork-specs/WEB_BACKEND_SPEC.md`
- `../sdkwork-specs/DATABASE_SPEC.md`
- `../sdkwork-specs/PRIVACY_SPEC.md`
- `../sdkwork-specs/SECURITY_SPEC.md`
- `../sdkwork-specs/OBSERVABILITY_SPEC.md`

Root SDKWork standards remain authoritative. This document narrows them for SDKWork Memory.

## 2. Current State

The repository implements commercial memory management on top of the Phase 1 base:

- `ai_subject`, `ai_memory_binding`, `ai_capability_binding`, `ai_entity`, `ai_edge`, `ai_policy`, `ai_policy_assignment`, and `ai_commercial_readiness_snapshot` are first-class SQL tables with symmetric SQLite/PostgreSQL migrations (SQLite FTS includes `predicate` via `V202606250003`).
- **Backend API** exposes the full commercial control-plane surface (subjects, bindings, capability bindings, entities, edges, policies, policy assignments, `capabilities.resolve`, `commercialReadiness`).
- **App API** currently exposes `entities.*` and `policyAssignments.list/create/update` only; subjects, bindings, edges, policies, and readiness remain backend-only until App §6.2 routes land.
- **Open API** currently exposes `entities.*` and `edges.*` only; subjects, bindings, and `capabilities.resolve` remain backend-only until Open §6.1 routes land.
- **Authorization:** `access.rs` enforces user/agent space ownership, active `ai_memory_binding` grants with **read vs write role separation** (`viewer` read-only; `owner`/`learner` write), capability deny on memory read/list/retrieve/write paths, and entity sensitivity filtering pushed to SQL on list paths. Tenant-owned spaces no longer grant implicit access to all tenant actors.
- Entity/edge read/list/delete enforce space access on all surfaces; `list_entities` applies store-level sensitivity predicates (no in-memory post-filter pagination). `create_edge` validates endpoint entities belong to the target space.
- `capabilities.resolve` returns `201` with `data.items` + `data.pageInfo` (cursor pagination, store-level `LIMIT`).
- `commercialReadiness.rebuild` computes coverage from live tenant counts, runtime probes (`snowflakeInitialized`, `outboxDeliveryReady`, export disable flag), and blocking/warning findings; `commercialReadiness` is true only when readiness `state == "ready"`.
- Extraction requests enqueue durable `ai_learning_job` rows (`state = queued`) and execute asynchronously via the learning-job worker; event payloads must contain non-empty `content`.
- Production-like environments require database-backed Snowflake allocation, PostgreSQL (SQLite rejected), and HTTP outbox delivery (`SDKWORK_MEMORY_OUTBOX_DELIVERY_MODE=http` + `SDKWORK_MEMORY_OUTBOX_DELIVERY_URL`).
- Implementation profile migration jobs validate profiles, support `dryRun`, **`shadow`** (non-mutating validation), **`promote`/`switch`** (transactional primary demote/promote + preference upsert, then index rebuild), and record `implementation_profile.active` tenant preference.
- SQLite FTS is synced on SPI `create_record`, candidate promotion, and Open API write paths; PostgreSQL uses trigger-backed `search_document`. SQLite baseline includes predicate FTS via `V202606250003`.
- Export jobs use actor-scoped sensitivity SQL filters, `SDKWORK_MEMORY_EXPORT_MAX_RECORDS`, and `SDKWORK_MEMORY_EXPORT_MAX_EVENTS` (default 100k). Extraction caps `SDKWORK_MEMORY_EXTRACTION_MAX_EVENTS` (default 1000). Export/retrieval cap `spaceIds` at 32.
- Background workers use non-elevated `for_background_job` context; extraction jobs require `actorId` in input and mandatory `job.space_id` for scope mutations.
- List pagination uses `PageInfo.mode = "cursor"`, `page_size` default 20 and max 200 per `PAGINATION_SPEC.md` (OpenAPI documents default). Space list `nextCursor` returns opaque `space.uuid` (legacy numeric internal cursors remain accepted).
- Eval runs and learning jobs use Postgres **`FOR UPDATE SKIP LOCKED`** claims with stale `running` requeue; SQLite uses optimistic claim (single-writer).
- Index rebuild jobs respect index `spaceId` scope via `rebuild_record_search_indexes_for_space`.
- Keyword search LIKE fallback includes `predicate` when FTS/fulltext is unavailable.

Remaining commercial gaps before full L3 landing:

- App/Open commercial routes in §6.1–§6.2 (subjects, bindings, resolve helpers) are not yet implemented.
- Backend §6.3 deferred operations: `subjects.effectiveCapabilities/effectivePolicies`, `entities.merge`, binding bulk/update/resolve, `capabilityBindings.update`, `relationRebuildJobs.*`.
- Vector/embedding retriever remains unsupported; capabilities must not advertise it until implemented.
- Reference implementation profiles remain catalog metadata until runnable plugin wiring lands.
- Production deployments must use PostgreSQL 15+ and `build_router_with_open_memory_service` (not trait-only `gateway_mount`) for commercial routes.
- Postgres store contract tests cover core SPI, FTS predicate search, eval claims, and retrieval-trace boolean roundtrip; expand parity with SQLite suite over time.

## 3. Commercial Landing Standard

SDKWork Memory is commercially ready only when a tenant can answer these questions through API and SDK calls:

1. Which memories belong to this user, agent, organization, app, project, session, or external entity?
2. Which memory spaces are visible to this agent for this request?
3. Which memory capabilities are enabled for this subject or entity?
4. Which policies control learning, retrieval, context assembly, export, deletion, and retention?
5. Which entities and graph edges caused a memory to be retrieved?
6. Which memories were used to build a context pack?
7. Which memory relationship changed, who changed it, and under which request id?
8. Which implementation profile and retrievers were active when the result was produced?
9. Can the tenant export, delete, disable, migrate, and audit memory without provider lock-in?
10. Can SDKs expose this as typed resources instead of raw filter objects and metadata blobs?

The commercial target is L3 for management, privacy, tenant isolation, audit, and provider switching.

## 4. Design Principles

### 4.1 Memory Records Stay Canonical

`ai_record` remains the canonical durable memory fact. Attachment and capability records never replace canonical memory state.

Rules:

- A memory may exist without a binding only during migration, import, review, or quarantine states.
- Production-active memory should have at least one discoverable subject, entity, space, or policy relationship.
- Deleting or disabling a binding must suppress future retrieval through that relationship even when derived indexes are stale.
- Deleting a memory must suppress all bindings and derived index entries.

### 4.2 Relationships Are First-Class

Commercial memory cannot hide relationship ownership in `metadata_json` or caller-defined filters.

The relationship layer must support:

- subject to space
- subject to memory
- subject to entity
- entity to memory
- entity to space
- memory to memory
- agent to retrieval profile
- subject/entity/space to policy
- subject/entity/space/binding to capability

### 4.3 Flexible But Not EAV

SDKWork database standards discourage using a generic catch-all table for core business facts. The binding model therefore uses explicit source and target columns for common high-value relationships, while still allowing a controlled external reference for app-owned business objects.

Rules:

- High-frequency predicates must be first-class columns.
- App-owned business objects use `external_ref` only with `external_ref_type`, `external_ref_id`, and `external_ref_source`.
- Authorization, policy, and retrieval filtering must not depend only on unstructured JSON.

### 4.4 Subject Is A Memory Concept, Not An IAM Fork

`MemorySubject` is a local memory-management projection. It references IAM, agent, app, project, organization, session, external provider, or entity identity. It does not own login, user profiles, tenants, organizations, sessions, roles, or permissions.

Rules:

- User and organization identities remain owned by appbase/IAM.
- Agent identities remain owned by the agent/intelligence capability that creates them.
- Memory stores only the stable subject reference needed for memory ownership, policy, and retrieval.
- Backend services must resolve authenticated tenant, organization, user, and permission context through appbase request context, not through memory subject records.

### 4.5 Capabilities Attach Through Relationships

A memory capability is a governed behavior that can be enabled, disabled, shadowed, or evaluated for a target relationship.

Initial capability codes:

- `learn`
- `retrieve`
- `context_pack`
- `habit`
- `graph`
- `export`
- `delete`
- `audit`
- `migration`
- `admin`

Capabilities are resolved from the active subject, entity, space, binding, retrieval profile, implementation profile, and policy assignments.

## 5. Resource Model

### 5.1 MemorySubject

Represents a user, agent, organization, app, project, session, entity, external integration, or business object as a memory-management target.

Fields:

```text
subjectId
tenantId
organizationId
subjectType
subjectRef
displayName
status
defaultSpaceId
metadata
createdAt
updatedAt
version
```

`subjectType` values:

```text
user
agent
organization
app
project
session
entity
external
business_object
```

Rules:

- `(tenantId, subjectType, subjectRef)` is unique.
- `subjectRef` is a stable owner reference from the owning domain.
- `subjectRef` is not a credential and must not contain tokens or secrets.
- `business_object` requires `metadata.appId`, `metadata.resourceType`, and `metadata.resourceId`.

### 5.2 MemoryEntity

Exposes `ai_entity` as a first-class API resource.

Fields:

```text
entityId
spaceId
entityType
canonicalName
aliases
attributes
sensitivityLevel
status
createdAt
updatedAt
version
```

Rules:

- Entity APIs are not a replacement for product-owned master data.
- Entities are memory identity projections used for retrieval, graph relations, and explanation.
- Entity merge, delete, and alias updates must record audit events.

### 5.3 MemoryEdge

Exposes `ai_edge` as a first-class API resource.

Fields:

```text
edgeId
spaceId
sourceEntityId
targetEntityId
relationType
sourceMemoryId
weight
validFrom
validTo
status
metadata
createdAt
updatedAt
version
```

Rules:

- Graph retrieval must explain memory hits through edge ids and memory ids.
- Deleted or inactive edges must not produce active retrieval hits.
- Temporal edges must respect `validFrom` and `validTo`.

### 5.4 MemoryBinding

Represents an auditable relationship between subjects, spaces, entities, memories, and app-owned business objects.

Fields:

```text
bindingId
tenantId
spaceId
bindingKind
sourceSubjectId
sourceEntityId
sourceMemoryId
sourceExternalRef
targetSubjectId
targetEntityId
targetMemoryId
targetSpaceId
targetExternalRef
bindingRole
capabilityCodes
retrievalProfileId
policyAssignmentId
strength
validFrom
validTo
status
metadata
createdAt
updatedAt
version
```

`bindingKind` values:

```text
subject_space
subject_memory
subject_entity
entity_memory
entity_space
memory_memory
agent_space
agent_memory
business_object_memory
external_shadow
```

`bindingRole` values:

```text
owner
viewer
learner
retriever
context_source
evidence
correction
suppression
import_shadow
```

Rules:

- Exactly one source and one target must be set.
- `agent_space` and `agent_memory` require source subject type `agent`.
- `subject_memory` and `entity_memory` require target memory id.
- `subject_space`, `agent_space`, and `entity_space` require target space id.
- External references must include source, type, and id.
- Sensitive bindings require policy assignment or backend review.
- Disabled, expired, or deleted bindings must be excluded from effective capability resolution.

### 5.5 MemoryCapabilityBinding

Attaches a behavior to a subject, entity, space, binding, retrieval profile, or implementation profile.

Fields:

```text
capabilityBindingId
tenantId
capabilityCode
targetType
targetId
mode
priority
retrievalProfileId
implementationProfileId
policyAssignmentId
status
validFrom
validTo
metadata
createdAt
updatedAt
version
```

`mode` values:

```text
enabled
disabled
shadow
eval_only
```

Rules:

- Backend API owns full capability binding management.
- App API may expose user-controlled enable/disable only for user-owned learning and retrieval controls.
- Open API may manage capability bindings only within API-key authorized scope.
- Capability resolution must produce explainable results.

### 5.6 MemoryPolicyAssignment

Attaches a `ai_policy` row to a subject, space, entity, binding, capability binding, or implementation profile.

Fields:

```text
policyAssignmentId
tenantId
policyId
targetType
targetId
priority
inheritanceMode
status
validFrom
validTo
createdAt
updatedAt
version
```

`inheritanceMode` values:

```text
inherit
override
deny
shadow
```

Rules:

- Tenant policy overrides user preference.
- Deny policy wins over allow policy at the same or lower scope.
- Current explicit user instruction still controls the current turn, within security and policy limits.
- All effective policy resolution must be auditable.

### 5.7 MemoryCommercialReadiness

Backend-admin diagnostic projection showing whether a tenant/profile is commercially ready.

Fields:

```text
readinessId
tenantId
implementationProfileId
score
state
contractCoverage
managementCoverage
runtimeConformance
privacyCoverage
auditCoverage
sdkCoverage
evaluationCoverage
observabilityCoverage
migrationCoverage
blockingFindings
warningFindings
createdAt
```

Rules:

- This is a projection, not the source of truth.
- Readiness must be rebuildable from contracts, conformance runs, eval runs, audit config, and verification evidence.

## 6. API Design

All new operations must follow `../sdkwork-specs/API_SPEC.md`:

- Open API prefix: `/mem/v3/api`
- App API prefix: `/app/v3/api`
- Backend API prefix: `/backend/v3/api`
- Operation ids: dotted lowerCamelCase resource style
- Problem details: `application/problem+json`
- Create and bulk commands: `Idempotency-Key`
- Protected Open API: `ApiKey`
- Protected App/Backend API: `AuthToken` and `AccessToken`
- Every operation declares owner, authority, permission, tenant scope, data scope, and audit event.

### 6.1 Open API Additions

Open API serves external integrations, API-key clients, and server-to-server Memory SDK consumers. It must not expose backend-only operations.

**Implemented today:** `entities.*`, `edges.*`.

**Planned (not yet routed):**

```text
GET    /mem/v3/api/memory/subjects                    subjects.list
POST   /mem/v3/api/memory/subjects                    subjects.create
GET    /mem/v3/api/memory/subjects/{subjectId}        subjects.retrieve
PATCH  /mem/v3/api/memory/subjects/{subjectId}        subjects.update

GET    /mem/v3/api/memory/entities                    entities.list
POST   /mem/v3/api/memory/entities                    entities.create
GET    /mem/v3/api/memory/entities/{entityId}         entities.retrieve
PATCH  /mem/v3/api/memory/entities/{entityId}         entities.update

GET    /mem/v3/api/memory/edges                       edges.list
POST   /mem/v3/api/memory/edges                       edges.create
GET    /mem/v3/api/memory/edges/{edgeId}              edges.retrieve
PATCH  /mem/v3/api/memory/edges/{edgeId}              edges.update
DELETE /mem/v3/api/memory/edges/{edgeId}              edges.delete

GET    /mem/v3/api/memory/bindings                    bindings.list
POST   /mem/v3/api/memory/bindings                    bindings.create
GET    /mem/v3/api/memory/bindings/{bindingId}        bindings.retrieve
PATCH  /mem/v3/api/memory/bindings/{bindingId}        bindings.update
DELETE /mem/v3/api/memory/bindings/{bindingId}        bindings.delete
POST   /mem/v3/api/memory/bindings/resolve            bindings.resolve

POST   /mem/v3/api/memory/capabilities/resolve        capabilities.resolve
```

Open API capability:

- External systems can create app-owned subjects and business-object bindings.
- External systems can retrieve effective memory capabilities for their API-key scope.
- External systems cannot manage tenant-wide policies, provider secrets, migration jobs, or backend readiness.

### 6.2 App API Additions

App API serves user-facing clients, desktop, mobile, H5, and agent-facing app workflows.

**Implemented today:** `entities.*`, `policyAssignments.list/create/update`.

**Planned (not yet routed):**

```text
GET    /app/v3/api/memory/subjects                    subjects.list
POST   /app/v3/api/memory/subjects                    subjects.create
GET    /app/v3/api/memory/subjects/{subjectId}        subjects.retrieve
PATCH  /app/v3/api/memory/subjects/{subjectId}        subjects.update
GET    /app/v3/api/memory/subjects/{subjectId}/memories       subjects.memories.list
GET    /app/v3/api/memory/subjects/{subjectId}/bindings       subjects.bindings.list
POST   /app/v3/api/memory/subjects/{subjectId}/context_packs  subjects.contextPacks.create
POST   /app/v3/api/memory/subjects/{subjectId}/learning/enable   subjects.learning.enable
POST   /app/v3/api/memory/subjects/{subjectId}/learning/disable  subjects.learning.disable

GET    /app/v3/api/memory/entities                    entities.list
POST   /app/v3/api/memory/entities                    entities.create
GET    /app/v3/api/memory/entities/{entityId}         entities.retrieve
PATCH  /app/v3/api/memory/entities/{entityId}         entities.update

GET    /app/v3/api/memory/bindings                    bindings.list
POST   /app/v3/api/memory/bindings                    bindings.create
GET    /app/v3/api/memory/bindings/{bindingId}        bindings.retrieve
PATCH  /app/v3/api/memory/bindings/{bindingId}        bindings.update
DELETE /app/v3/api/memory/bindings/{bindingId}        bindings.delete
POST   /app/v3/api/memory/bindings/resolve            bindings.resolve

GET    /app/v3/api/memory/policy_assignments          policyAssignments.list
POST   /app/v3/api/memory/policy_assignments          policyAssignments.create
PATCH  /app/v3/api/memory/policy_assignments/{policyAssignmentId} policyAssignments.update
```

App API capability:

- Users can inspect, edit, delete, export, and disable learning for memories they can access.
- Agents can retrieve memory through subject and binding resolution.
- App clients can manage their own app/business-object bindings when authorized.
- App clients cannot mutate backend implementation profiles, provider bindings, migration jobs, or global tenant policy unless a backend-admin facade explicitly exposes that capability.

### 6.3 Backend API Additions

Backend API is the operator, tenant-admin, control-plane, support, and automation surface.

**Implemented today:** subjects/entities/edges CRUD (subjects include delete), bindings/capabilityBindings CRUD (no update), policies/policyAssignments CRUD, `capabilities.resolve`, `commercialReadiness.retrieve/rebuild`.

**Planned (not yet routed):**

```text
GET    /backend/v3/api/memory/subjects                subjects.list
POST   /backend/v3/api/memory/subjects                subjects.create
GET    /backend/v3/api/memory/subjects/{subjectId}    subjects.retrieve
PATCH  /backend/v3/api/memory/subjects/{subjectId}    subjects.update
GET    /backend/v3/api/memory/subjects/{subjectId}/effective_capabilities subjects.effectiveCapabilities.retrieve
GET    /backend/v3/api/memory/subjects/{subjectId}/effective_policies      subjects.effectivePolicies.retrieve

GET    /backend/v3/api/memory/entities                entities.list
POST   /backend/v3/api/memory/entities                entities.create
GET    /backend/v3/api/memory/entities/{entityId}     entities.retrieve
PATCH  /backend/v3/api/memory/entities/{entityId}     entities.update
POST   /backend/v3/api/memory/entities/{entityId}/merge entities.merge

GET    /backend/v3/api/memory/edges                   edges.list
POST   /backend/v3/api/memory/edges                   edges.create
GET    /backend/v3/api/memory/edges/{edgeId}          edges.retrieve
PATCH  /backend/v3/api/memory/edges/{edgeId}          edges.update
DELETE /backend/v3/api/memory/edges/{edgeId}          edges.delete

GET    /backend/v3/api/memory/bindings                bindings.list
POST   /backend/v3/api/memory/bindings                bindings.create
GET    /backend/v3/api/memory/bindings/{bindingId}    bindings.retrieve
PATCH  /backend/v3/api/memory/bindings/{bindingId}    bindings.update
DELETE /backend/v3/api/memory/bindings/{bindingId}    bindings.delete
POST   /backend/v3/api/memory/bindings/bulk_upsert    bindings.bulkUpsert
POST   /backend/v3/api/memory/bindings/bulk_delete    bindings.bulkDelete
POST   /backend/v3/api/memory/bindings/resolve        bindings.resolve

GET    /backend/v3/api/memory/capability_bindings     capabilityBindings.list
POST   /backend/v3/api/memory/capability_bindings     capabilityBindings.create
GET    /backend/v3/api/memory/capability_bindings/{capabilityBindingId} capabilityBindings.retrieve
PATCH  /backend/v3/api/memory/capability_bindings/{capabilityBindingId} capabilityBindings.update
DELETE /backend/v3/api/memory/capability_bindings/{capabilityBindingId} capabilityBindings.delete

GET    /backend/v3/api/memory/policy_assignments      policyAssignments.list
POST   /backend/v3/api/memory/policy_assignments      policyAssignments.create
GET    /backend/v3/api/memory/policy_assignments/{policyAssignmentId} policyAssignments.retrieve
PATCH  /backend/v3/api/memory/policy_assignments/{policyAssignmentId} policyAssignments.update
DELETE /backend/v3/api/memory/policy_assignments/{policyAssignmentId} policyAssignments.delete

POST   /backend/v3/api/memory/relation_rebuild_jobs   relationRebuildJobs.create
GET    /backend/v3/api/memory/relation_rebuild_jobs/{jobId} relationRebuildJobs.retrieve

GET    /backend/v3/api/memory/commercial_readiness    commercialReadiness.retrieve
POST   /backend/v3/api/memory/commercial_readiness/rebuild commercialReadiness.rebuild
```

Backend API capability:

- Manage subject projections.
- Manage graph identity and relationship projections.
- Manage bindings and capability bindings.
- Assign policies to subjects, spaces, entities, bindings, and profiles.
- Resolve effective capabilities for diagnostics.
- Rebuild relation projections.
- Prove tenant readiness for commercial rollout.

## 7. Database Contract Additions

Schema registry should add a new table file:

```text
docs/schema-registry/tables/006-memory-commercial-management.yaml
```

Recommended new tables:

### 7.1 `ai_subject`

System of record for memory-management subject projections.

Key columns:

```text
id
uuid
tenant_id
organization_id
subject_type
subject_ref
display_name
default_space_id
status
metadata_json
created_by
updated_by
created_at
updated_at
deleted_at
version
```

Indexes:

```text
uk_ai_subject_uuid (tenant_id, uuid)
uk_ai_subject_ref (tenant_id, subject_type, subject_ref)
idx_ai_subject_status (tenant_id, subject_type, status, updated_at)
idx_ai_subject_space (tenant_id, default_space_id, status)
```

### 7.2 `ai_memory_binding`

System of record for memory attachment relationships.

Key columns:

```text
id
uuid
tenant_id
space_id
binding_kind
source_subject_id
source_entity_id
source_memory_id
source_external_ref_type
source_external_ref_id
source_external_ref_source
target_subject_id
target_entity_id
target_memory_id
target_space_id
target_external_ref_type
target_external_ref_id
target_external_ref_source
binding_role
capability_codes_json
retrieval_profile_id
policy_assignment_id
strength
valid_from
valid_to
status
metadata_json
created_by
updated_by
created_at
updated_at
deleted_at
version
```

Indexes:

```text
uk_ai_memory_binding_uuid (tenant_id, uuid)
idx_ai_binding_source_subject (tenant_id, source_subject_id, binding_kind, status)
idx_ai_binding_source_entity (tenant_id, source_entity_id, binding_kind, status)
idx_ai_binding_target_memory (tenant_id, target_memory_id, binding_kind, status)
idx_ai_binding_target_space (tenant_id, target_space_id, binding_kind, status)
idx_ai_binding_external_source (tenant_id, source_external_ref_source, source_external_ref_type, source_external_ref_id)
idx_ai_binding_validity (tenant_id, valid_from, valid_to, status)
```

### 7.3 `ai_capability_binding`

System of record for capability enablement and routing.

Key columns:

```text
id
uuid
tenant_id
capability_code
target_type
target_id
mode
priority
retrieval_profile_id
implementation_profile_id
policy_assignment_id
status
valid_from
valid_to
metadata_json
created_by
updated_by
created_at
updated_at
deleted_at
version
```

Indexes:

```text
uk_ai_capability_binding_uuid (tenant_id, uuid)
idx_ai_capability_target (tenant_id, target_type, target_id, capability_code, status)
idx_ai_capability_priority (tenant_id, capability_code, mode, priority)
idx_ai_capability_validity (tenant_id, valid_from, valid_to, status)
```

### 7.4 `ai_policy_assignment`

System of record for attaching `ai_policy` to a target.

Key columns:

```text
id
uuid
tenant_id
policy_id
target_type
target_id
priority
inheritance_mode
status
valid_from
valid_to
created_by
updated_by
created_at
updated_at
deleted_at
version
```

Indexes:

```text
uk_ai_policy_assignment_uuid (tenant_id, uuid)
idx_ai_policy_assignment_target (tenant_id, target_type, target_id, status, priority)
idx_ai_policy_assignment_policy (tenant_id, policy_id, status)
idx_ai_policy_assignment_validity (tenant_id, valid_from, valid_to, status)
```

### 7.5 `ai_relation_rebuild_job`

Rebuilds subject, entity, edge, binding, and capability projections.

Key columns:

```text
id
uuid
tenant_id
job_type
state
scope_type
scope_id
idempotency_key
input_json
result_json
error_json
started_at
finished_at
created_at
updated_at
version
```

### 7.6 `ai_commercial_readiness_snapshot`

Read model for backend rollout readiness.

Key columns:

```text
id
uuid
tenant_id
implementation_profile_id
score
state
contract_coverage_json
management_coverage_json
runtime_conformance_json
privacy_coverage_json
audit_coverage_json
sdk_coverage_json
evaluation_coverage_json
observability_coverage_json
migration_coverage_json
blocking_findings_json
warning_findings_json
created_at
```

## 8. Service Design

### 8.1 Binding Resolver

The Binding Resolver is the new required service boundary.

Input:

```text
tenant context
actor context
subject id or subject ref
entity ids
space ids
memory types
capability codes
query context
time
```

Output:

```text
effective subjects
effective spaces
effective entities
effective memories
effective bindings
effective capability bindings
effective policies
selected retrieval profile
selected implementation profile
explanation
```

Rules:

- The resolver must run before retrieval orchestration when a request supplies subject/entity/binding intent.
- The resolver must apply tenant, organization, user, owner, data-scope, status, validity, and policy filters.
- The resolver must emit traceable reasons.

### 8.2 Capability Resolver

The Capability Resolver decides whether a subject, entity, space, binding, or memory can perform a memory action.

Resolution order:

1. Tenant and organization policy.
2. Backend admin deny/allow assignments.
3. Subject capability binding.
4. Space capability binding.
5. Entity or binding capability binding.
6. Retrieval profile and implementation profile support.
7. Runtime plugin health and degraded-mode policy.

Rules:

- Deny wins over allow unless a higher-priority backend-admin override exists.
- Disabled or expired bindings are ignored.
- Missing required plugin capability fails closed.
- Optional retriever failure may degrade only when retrieval profile allows it.

### 8.3 Retrieval Integration

Commercial retrieval flow:

```text
request
  -> typed request context
  -> binding resolver
  -> capability resolver
  -> retrieval profile selection
  -> retriever orchestration
  -> canonical record rehydrate
  -> binding and policy recheck
  -> score fusion
  -> context assembly
  -> trace, hit, context pack, audit
```

Rules:

- Retrieval must not return a memory only because a vector/search index matched it.
- Retrieval must rehydrate canonical records and recheck binding state.
- Context packs must cite memory ids and binding/entity/edge reasons when relevant.

### 8.4 Write Integration

Commercial memory write flow:

```text
request
  -> typed request context
  -> capability resolver
  -> canonical event append
  -> candidate or record write
  -> source links
  -> binding writes
  -> audit and outbox
  -> index jobs
```

Rules:

- Explicit remember requests may create both `ai_record` and `ai_memory_binding`.
- Imported memory may create external shadow bindings.
- Corrections should supersede memory and update bindings rather than mutate history silently.
- Deletion must suppress bindings and index entries before a memory can be considered unavailable.

## 9. API Schema Additions

OpenAPI should add these schemas to all three surfaces where used:

```text
MemorySubject
MemorySubjectRequest
MemorySubjectList
MemoryEntity
MemoryEntityRequest
MemoryEntityList
MemoryEdge
MemoryEdgeRequest
MemoryEdgeList
MemoryBinding
MemoryBindingRequest
MemoryBindingList
MemoryBindingResolutionRequest
MemoryBindingResolution
MemoryCapabilityBinding
MemoryCapabilityBindingRequest
MemoryCapabilityBindingList
MemoryCapabilityResolutionRequest
MemoryCapabilityResolution
MemoryPolicyAssignment
MemoryPolicyAssignmentRequest
MemoryPolicyAssignmentList
MemoryRelationRebuildJob
MemoryRelationRebuildJobRequest
MemoryCommercialReadiness
MemoryCommercialReadinessRequest
```

Rules:

- `int64` ids serialize as strings.
- Enum fields remain strings in API.
- Sensitive details are redacted in list responses.
- Backend detail responses may include diagnostic metadata when permission allows.
- Public SDK-generated operations must not include backend-only fields such as secret refs.

## 10. Permissions And Audit Events

Permission shape:

```text
memory.open.subjects.read
memory.open.subjects.write
memory.open.entities.read
memory.open.entities.write
memory.open.bindings.read
memory.open.bindings.write
memory.open.capabilities.resolve

memory.app.subjects.read
memory.app.subjects.write
memory.app.entities.read
memory.app.entities.write
memory.app.bindings.read
memory.app.bindings.write
memory.app.policies.write
memory.app.capabilities.resolve

memory.backend.subjects.read
memory.backend.subjects.write
memory.backend.entities.read
memory.backend.entities.write
memory.backend.edges.write
memory.backend.bindings.write
memory.backend.capabilities.write
memory.backend.policies.write
memory.backend.relations.rebuild
memory.backend.commercialReadiness.read
memory.backend.commercialReadiness.write
```

Audit event shape:

```text
memory.subject.created
memory.subject.updated
memory.entity.created
memory.entity.updated
memory.entity.merged
memory.edge.created
memory.edge.updated
memory.edge.deleted
memory.binding.created
memory.binding.updated
memory.binding.deleted
memory.binding.resolved
memory.capability_binding.created
memory.capability_binding.updated
memory.capability_binding.deleted
memory.policy_assignment.created
memory.policy_assignment.updated
memory.policy_assignment.deleted
memory.relation_rebuild.requested
memory.commercial_readiness.rebuilt
```

Rules:

- Backend mutations must emit audit records.
- App and Open API mutations must emit audit records when they change durable relationships, policies, capabilities, or graph state.
- Resolve operations should emit retrieval/diagnostic traces, not full audit logs, unless backend-admin explicitly requests audit evidence.

## 11. SDK And Consumer Impact

SDK generator inputs remain owner-only.

Expected SDK resource shape:

```text
client.subjects.list(...)
client.subjects.create(...)
client.subjects.retrieve(subjectId)
client.subjects.bindings.list(subjectId, ...)
client.subjects.contextPacks.create(subjectId, ...)

client.entities.list(...)
client.entities.create(...)
client.edges.create(...)
client.bindings.resolve(...)
client.capabilityBindings.create(...)
client.policyAssignments.create(...)
client.commercialReadiness.retrieve(...)
```

Rules:

- App/user-facing clients must not import backend SDKs.
- Backend-admin clients use backend SDK only.
- Open API clients use API-key credentials, not app login token managers.
- Service facades may wrap binding resolution but must not call raw HTTP.
- Missing generated SDK methods must be fixed by OpenAPI and generation, not handwritten URL calls.

## 12. Commercial Readiness Criteria

Minimum rollout criteria:

1. Phase 1 contract verification passes.
2. OpenAPI authority documents include subject, entity, edge, binding, capability binding, policy assignment, relation rebuild, and commercial readiness contracts.
3. Schema registry includes `006-memory-commercial-management.yaml`.
4. Native SQL migrations include new tables for PostgreSQL and SQLite.
5. Runtime service layer resolves bindings before retrieval.
6. Effective capability resolution is tested with allow, deny, expired, disabled, and missing-plugin cases.
7. Retrieval traces include binding/entity/edge reasons.
8. Context packs cite memory ids and relationship reasons.
9. Backend API can list and manage subjects, entities, edges, bindings, capability bindings, and policy assignments.
10. App API can manage user/agent-scoped bindings and learning controls.
11. Open API can manage API-key scoped subjects, entities, and bindings.
12. Audit logs record relationship and policy mutations.
13. Delete/forget flows suppress bindings and indexes.
14. Export flows include relationship metadata where permitted.
15. `cargo test --workspace` passes.
16. SDK generation and SDK method-surface tests pass when generator output is materialized.

## 13. Implementation Phases

### Phase 1: Contract Design

Deliver:

- This design document.
- Spec review approval.
- Implementation plan.

### Phase 2: OpenAPI And Schema Contracts

Deliver:

- `006-memory-commercial-management.yaml`.
- Materializer updates for new schemas and operations.
- Updated Open API, App API, Backend API authority documents.
- Contract tests for operation counts, required operation ids, schemas, auth modes, permissions, and audit events.

### Phase 3: Runtime Model And Ports

Deliver:

- Binding resolver contracts.
- Capability resolver contracts.
- SPI structs and service DTOs.
- Native SQL port expectations for subjects, bindings, capability bindings, policy assignments, and relation rebuild jobs.

### Phase 4: Native SQL Implementation

Deliver:

- PostgreSQL and SQLite migrations.
- Native SQL stores.
- Tenant and organization isolation tests.
- Binding validity and deletion suppression tests.
- Candidate, habit, retrieval trace, and relation runtime tests.

### Phase 5: API Routes And Services

Deliver:

- Open API, App API, and Backend API route coverage.
- Typed request-context integration.
- Appbase dual-token and Open API-key test resolvers.
- Problem-detail mapping.
- Audit and outbox events.

### Phase 6: SDK And Commercial Readiness

Deliver:

- Generated SDK method-surface checks.
- Backend commercial readiness projection and endpoint.
- Documentation examples.
- Final phase verification bundle.

## 14. Deferred Items

These items are intentionally deferred until commercial contracts are in place:

- Dynamic runtime plugin loading.
- Marketplace-distributed memory plugins.
- Provider-specific external memory bridges in production mode.
- Vector or graph database lock-in.
- RPC SDKs.
- Backend/admin UI screens.
- Billing and entitlement packaging.

They must not block the contract-first commercial memory management layer.

## 15. Acceptance Checklist

- [x] Commercial SQL schema and symmetric SQLite/PostgreSQL migrations for subjects, bindings, entities, edges, policies, policy assignments, and readiness snapshots.
- [x] Backend API exposes the full implemented commercial control-plane surface (see §6.3 **Implemented today**).
- [ ] App API §6.2 subjects/bindings/resolve routes.
- [ ] Open API §6.1 subjects/bindings/resolve routes.
- [x] Open API entity and edge additions are API-key protected and avoid backend-only operations.
- [ ] App API policy/edge coverage matches §6.2 target (entities + policy assignments only today).
- [x] Backend API additions support tenant/admin governance for implemented resources.
- [x] Schema additions avoid generic EAV for high-frequency access paths.
- [x] Permissions and audit events are stable and dotted.
- [x] Binding resolver and capability resolver are explicit service boundaries (backend `capabilities.resolve` only).
- [x] Retrieval rechecks canonical record, binding, policy, and capability state after derived index hits. (Capability check is integrated pre-retrieval; post-hit recheck is pending.)
- [x] Deletion and forget flows suppress bindings and indexes (FTS removal errors propagate; forget/export store paths exist).
- [ ] SDK resource method shape remains resource-oriented across all three surfaces.
- [ ] Verification commands and runtime tests prove commercial readiness before release. (`pnpm verify`, pagination/envelope checks, SQLite contract suite, backend commercial flow, and opt-in Postgres contract tests pass; App/Open commercial integration tests and relation-rebuild worker remain.)
- [ ] `relation_rebuild_jobs` route/worker surface (DDL exists).



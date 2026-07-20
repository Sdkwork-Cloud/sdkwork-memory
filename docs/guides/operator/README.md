# Operator Guide

Deployment, monitoring, and incident response entrypoints.

## Memory Scheme Selection

Production PostgreSQL deployments use `native_sql`; local/private SQLite uses
`local_embedded`. The default `auto` selection enforces that mapping.

```powershell
$env:SDKWORK_MEMORY_IMPLEMENTATION_PROFILE = "native_sql"
$env:SDKWORK_MEMORY_RETRIEVAL_STRATEGY = "balanced"
```

Supported retrieval strategies are:

| Strategy | Use case |
| --- | --- |
| `balanced` | General assistant memory across facts, recency, and linked events |
| `search_first` | Text-heavy memory where lexical precision and predictable latency dominate |
| `event_aware` | Conversation and activity memory where linked event evidence should rank first |

Unknown strategy names and storage-incompatible implementation selections stop
startup. Scheme changes require a process restart. Verify the active selection
through the open API capabilities metadata and the bounded runtime profile
metric label before routing production traffic.

Evaluation-only implementation kinds (`event_sourced`, standalone
`search_first`, `graph_temporal`, `external_provider_bridge`, and
`hybrid_platform`) must not be promoted to primary.

See `DOCUMENTATION_SPEC.md` section 2.

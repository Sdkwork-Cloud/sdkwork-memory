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

## Retrieval Quality Evaluation

`retrieval_quality` requires bounded inline golden cases. `datasetRef` is
persisted for dataset identity and audit, but it is not resolved from a file or
remote registry; a request containing only `datasetRef` is rejected. This is a
deliberate fail-closed boundary until a reviewed dataset provider exists.

```json
{
  "evalType": "retrieval_quality",
  "datasetRef": "support-preferences-v3",
  "profileRef": "42",
  "config": {
    "contextBudgetTokens": 4096,
    "cases": [
      {
        "spaceId": "7",
        "query": "Which editor keybindings does the user prefer?",
        "expectedMemoryIds": ["10021"],
        "topK": 10
      }
    ],
    "thresholds": {
      "minRecallAtK": 0.9,
      "minHitRateAtK": 0.95,
      "minMeanReciprocalRank": 0.8,
      "maxDegradedRate": 0.01
    }
  }
}
```

The worker runs the selected profile through the production retrieval path and
stores macro Recall@K, Hit Rate@K, MRR, degraded rate, per-case hashed query
identity, and `qualityGatePassed`. A false gate means the evaluation completed
but the profile is not qualified; it is not rewritten as a successful quality
claim. A malformed dataset or missing profile moves the run to `failed` with a
reason and no fabricated metrics.

See `DOCUMENTATION_SPEC.md` section 2.

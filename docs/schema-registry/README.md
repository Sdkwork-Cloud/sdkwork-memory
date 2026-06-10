# SDKWork Memory Schema Registry

Memory database contracts are defined here before migrations or ORM entities are created.

Rules:

- Physical table names use the `mem_` prefix.
- `mem_record` and `mem_event` are canonical source-of-truth tables.
- Index, retrieval, vector, graph, grep/file, and provider states are derived or governed tables and must be rebuildable from canonical records and events when possible.
- PostgreSQL is the production/server target; SQLite is allowed for local/private/test parity where feasible.
- 64-bit identifiers are serialized as strings in API/SDK contracts.

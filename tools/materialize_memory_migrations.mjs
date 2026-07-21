#!/usr/bin/env node

// Compatibility entrypoint retained for existing verification commands.
// Canonical migration folding is owned by the application-root materializer.
await import("../scripts/materialize-memory-database-baseline.mjs");

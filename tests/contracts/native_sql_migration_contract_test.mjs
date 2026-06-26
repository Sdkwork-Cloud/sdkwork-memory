import assert from "node:assert/strict";
import fs from "node:fs";

const migrationPaths = [
  "database/migrations/sqlite/0001_memory_phase1.up.sql",
  "database/migrations/postgres/0001_memory_phase1.up.sql",
  "plugins/sdkwork-memory-plugin-native-sql/migrations/sqlite/V202606100001__memory_phase1.sql",
  "plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100001__memory_phase1.sql",
];

const requiredTables = [
  "ai_space",
  "ai_event",
  "ai_record",
  "ai_record_source",
  "ai_candidate",
  "ai_habit",
  "ai_retrieval_trace",
  "ai_retrieval_hit",
  "ai_context_pack",
  "ai_index",
  "ai_retrieval_profile",
  "ai_implementation_profile",
  "ai_provider_binding",
  "ai_eval_run",
  "ai_audit_log",
  "ai_outbox_event",
];

for (const migrationPath of migrationPaths) {
  assert.ok(fs.existsSync(migrationPath), `${migrationPath} must exist`);
  const sql = fs.readFileSync(migrationPath, "utf8").toLowerCase();

  for (const table of requiredTables) {
    assert.match(
      sql,
      new RegExp(`create\\s+table\\s+(if\\s+not\\s+exists\\s+)?${table}\\b`),
      `${migrationPath} must create ${table}`,
    );
  }

  assert.doesNotMatch(
    sql,
    /\b(vector|embedding|embeddings|pgvector)\b/,
    `${migrationPath} must not require vector or embedding storage in native_sql phase1`,
  );
}

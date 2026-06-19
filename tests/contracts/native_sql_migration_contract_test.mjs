import assert from "node:assert/strict";
import fs from "node:fs";

const migrationPaths = [
  "plugins/sdkwork-memory-plugin-native-sql/migrations/sqlite/V202606100001__memory_phase1.sql",
  "plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100001__memory_phase1.sql",
];

const requiredTables = [
  "mem_space",
  "mem_event",
  "mem_record",
  "mem_record_source",
  "mem_candidate",
  "mem_habit",
  "mem_retrieval_trace",
  "mem_retrieval_hit",
  "mem_context_pack",
  "mem_index",
  "mem_retrieval_profile",
  "mem_implementation_profile",
  "mem_provider_binding",
  "mem_eval_run",
  "mem_audit_log",
  "mem_outbox_event",
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

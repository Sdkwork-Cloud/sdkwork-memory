import assert from "node:assert/strict";
import fs from "node:fs";

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
];

for (const file of [
  "plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100001__memory_phase1.sql",
  "plugins/sdkwork-memory-plugin-native-sql/migrations/sqlite/V202606100001__memory_phase1.sql",
]) {
  const sql = fs.readFileSync(file, "utf8").toLowerCase();
  for (const table of requiredTables) {
    assert.match(
      sql,
      new RegExp(`create\\s+table\\s+(if\\s+not\\s+exists\\s+)?${table}\\b`),
      `${file} missing ${table}`,
    );
  }
  assert.doesNotMatch(
    sql,
    /vector|embedding\(/,
    `${file} must not require vector or embedding storage in Phase 1`,
  );
}

console.log("Schema registry phase1 contract test passed");

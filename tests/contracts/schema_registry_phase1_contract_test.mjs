import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const schemaRegistryDir = "docs/schema-registry/tables";
const phase1Tables = new Set([
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
]);

const migrationPaths = [
  "plugins/sdkwork-memory-plugin-native-sql/migrations/sqlite/V202606100001__memory_phase1.sql",
  "plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100001__memory_phase1.sql",
];

function loadSchemaRegistryIndexes() {
  const indexes = [];
  for (const file of fs.readdirSync(schemaRegistryDir)) {
    if (!file.endsWith(".yaml")) {
      continue;
    }
    const text = fs.readFileSync(path.join(schemaRegistryDir, file), "utf8");
    let currentTable = null;
    let inIndexesSection = false;
    for (const line of text.split("\n")) {
      const tableMatch = line.match(/^\s+- table:\s+(\w+)/);
      if (tableMatch) {
        currentTable = tableMatch[1];
        inIndexesSection = false;
        continue;
      }
      if (/^\s+indexes:/.test(line)) {
        inIndexesSection = true;
        continue;
      }
      if (/^\s+columns:/.test(line)) {
        inIndexesSection = false;
        continue;
      }
      if (/^\s+- table:/.test(line) || /^\s+serialization:/.test(line)) {
        inIndexesSection = false;
      }
      const indexMatch = line.match(/^\s+- \{ name: (\w+),/);
      if (
        indexMatch
        && currentTable
        && phase1Tables.has(currentTable)
        && inIndexesSection
      ) {
        indexes.push(indexMatch[1]);
      }
    }
  }
  return [...new Set(indexes)].sort();
}

const requiredIndexes = loadSchemaRegistryIndexes();
assert.ok(requiredIndexes.length > 0, "schema registry must declare phase1 indexes");

const migrationGroups = [
  [
    "plugins/sdkwork-memory-plugin-native-sql/migrations/sqlite/V202606100001__memory_phase1.sql",
    "plugins/sdkwork-memory-plugin-native-sql/migrations/sqlite/V202606100002__memory_phase1_indexes.sql",
  ],
  [
    "plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100001__memory_phase1.sql",
    "plugins/sdkwork-memory-plugin-native-sql/migrations/postgres/V202606100002__memory_phase1_indexes.sql",
  ],
];

for (const group of migrationGroups) {
  const combinedSql = group
    .map((migrationPath) => {
      assert.ok(fs.existsSync(migrationPath), `${migrationPath} must exist`);
      return fs.readFileSync(migrationPath, "utf8");
    })
    .join("\n")
    .toLowerCase();

  for (const indexName of requiredIndexes) {
    assert.match(
      combinedSql,
      new RegExp(`\\b${indexName.toLowerCase()}\\b`),
      `${group.join(" + ")} must materialize schema-registry index ${indexName}`,
    );
  }
}

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

for (const migrationPath of migrationPaths) {
  const sql = fs.readFileSync(migrationPath, "utf8").toLowerCase();
  for (const table of requiredTables) {
    assert.match(
      sql,
      new RegExp(`create\\s+table\\s+(if\\s+not\\s+exists\\s+)?${table}\\b`),
      `${migrationPath} missing ${table}`,
    );
  }
  assert.doesNotMatch(
    sql,
    /vector|embedding\(/,
    `${migrationPath} must not require vector or embedding storage in Phase 1`,
  );
}

const storeSource = fs.readFileSync(
  "plugins/sdkwork-memory-plugin-native-sql/src/store.rs",
  "utf8",
);
assert.ok(
  storeSource.includes("V202606100002__memory_phase1_indexes.sql"),
  "native-sql store must apply phase1 index migration",
);

console.log(
  `Schema registry phase1 contract test passed (${requiredIndexes.length} indexes)`,
);

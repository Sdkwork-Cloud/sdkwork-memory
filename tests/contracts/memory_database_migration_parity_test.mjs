#!/usr/bin/env node
/**
 * Ensures canonical database/ddl/baseline stays aligned with native-sql plugin authority.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const engines = ["postgres", "sqlite"];

function normalizeSql(sql) {
  return sql.replace(/\r\n/g, "\n");
}

const pluginMigrationOrder = [
  "V202606100001__memory_phase1.sql",
  "V202606100002__memory_phase1_indexes.sql",
  "V202606230001__mem_tenant_preference.sql",
  "V202606240001__ai_learning_job.sql",
  "V202606240002__ai_record_fulltext_search.sql",
  "V202606250001__ai_eval_run_extend.sql",
  "V202606250002__memory_commercial_management.sql",
];

for (const engine of engines) {
  const baselinePath = path.join(
    root,
    "database/ddl/baseline",
    engine,
    "0001_memory_baseline.sql",
  );
  assert.ok(fs.existsSync(baselinePath), `${baselinePath} must exist`);
  const baselineSql = normalizeSql(fs.readFileSync(baselinePath, "utf8"));

  for (const pluginName of pluginMigrationOrder) {
    const pluginPath = path.join(
      root,
      "plugins/sdkwork-memory-plugin-native-sql/migrations",
      engine,
      pluginName,
    );
    assert.ok(fs.existsSync(pluginPath), `${pluginPath} must exist`);
    const pluginSql = normalizeSql(fs.readFileSync(pluginPath, "utf8"));
    assert.ok(
      baselineSql.includes(pluginSql),
      `${baselinePath} must include plugin authority ${pluginPath}`,
    );
  }

  const migrationDir = path.join(root, "database/migrations", engine);
  const migrationSqlFiles = fs
    .readdirSync(migrationDir)
    .filter((name) => name.endsWith(".sql"));
  assert.equal(
    migrationSqlFiles.length,
    0,
    `${migrationDir} must stay empty during initialization (found ${migrationSqlFiles.join(", ")})`,
  );
}

const requiredPhase1Tables = ["ai_space", "ai_event", "ai_record"];

for (const engine of engines) {
  const baseline = fs
    .readFileSync(
      path.join(root, "database/ddl/baseline", engine, "0001_memory_baseline.sql"),
      "utf8",
    )
    .toLowerCase();
  for (const table of requiredPhase1Tables) {
    assert.match(
      baseline,
      new RegExp(`create\\s+table\\s+(if\\s+not\\s+exists\\s+)?${table}\\b`),
      `baseline for ${engine} must create ${table}`,
    );
  }

  assert.match(
    baseline,
    /create\s+table\s+(if\s+not\s+exists\s+)?ai_tenant_preference\b/,
    `baseline for ${engine} must create ai_tenant_preference`,
  );
  assert.match(
    baseline,
    /create\s+table\s+(if\s+not\s+exists\s+)?ai_subject\b/,
    `baseline for ${engine} must create ai_subject`,
  );
}

#!/usr/bin/env node
/**
 * Ensures canonical database/migrations stay aligned with native-sql plugin authority.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const engines = ["postgres", "sqlite"];
const mappings = [
  ["V202606100001__memory_phase1.sql", "0001_memory_phase1.up.sql"],
  ["V202606100002__memory_phase1_indexes.sql", "0002_memory_phase1_indexes.up.sql"],
  ["V202606230001__mem_tenant_preference.sql", "0003_mem_tenant_preference.up.sql"],
];

for (const engine of engines) {
  for (const [pluginName, canonicalName] of mappings) {
    const pluginPath = path.join(
      root,
      "plugins/sdkwork-memory-plugin-native-sql/migrations",
      engine,
      pluginName,
    );
    const canonicalPath = path.join(root, "database/migrations", engine, canonicalName);
    assert.ok(fs.existsSync(pluginPath), `${pluginPath} must exist`);
    assert.ok(fs.existsSync(canonicalPath), `${canonicalPath} must exist`);
    const pluginSql = fs.readFileSync(pluginPath, "utf8");
    const canonicalSql = fs.readFileSync(canonicalPath, "utf8");
    assert.equal(
      canonicalSql,
      pluginSql,
      `${canonicalPath} must match plugin authority ${pluginPath}`,
    );
  }
}

const requiredPhase1Tables = ["ai_space", "ai_event", "ai_record"];

for (const engine of engines) {
  const phase1 = fs.readFileSync(
    path.join(root, "database/migrations", engine, "0001_memory_phase1.up.sql"),
    "utf8",
  ).toLowerCase();
  for (const table of requiredPhase1Tables) {
    assert.match(
      phase1,
      new RegExp(`create\\s+table\\s+(if\\s+not\\s+exists\\s+)?${table}\\b`),
      `0001 migration for ${engine} must create ${table}`,
    );
  }

  const tenantPreference = fs.readFileSync(
    path.join(root, "database/migrations", engine, "0003_mem_tenant_preference.up.sql"),
    "utf8",
  ).toLowerCase();
  assert.match(
    tenantPreference,
    /create\s+table\s+(if\s+not\s+exists\s+)?ai_tenant_preference\b/,
    `0003 migration for ${engine} must create ai_tenant_preference`,
  );
}

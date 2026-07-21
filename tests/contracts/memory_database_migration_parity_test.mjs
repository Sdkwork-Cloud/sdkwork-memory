#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const requiredTables = ["ai_space", "ai_event", "ai_record", "ai_tenant_preference", "ai_subject"];

for (const engine of ["postgres", "sqlite"]) {
  const migrationRoot = path.join(root, "database", "migrations", engine);
  const upFiles = fs.readdirSync(migrationRoot)
    .filter((name) => name.endsWith(".up.sql"))
    .sort();
  assert.ok(upFiles.length > 0, `${engine} must own canonical application-root migrations`);

  for (const upFile of upFiles) {
    const downFile = upFile.replace(/\.up\.sql$/u, ".down.sql");
    assert.ok(
      fs.existsSync(path.join(migrationRoot, downFile)),
      `${engine}/${upFile} must have paired ${downFile}`,
    );
  }

  const migrationSql = upFiles
    .map((name) => fs.readFileSync(path.join(migrationRoot, name), "utf8"))
    .join("\n")
    .toLowerCase();
  const baseline = fs.readFileSync(
    path.join(root, "database", "ddl", "baseline", engine, "0001_memory_baseline.sql"),
    "utf8",
  ).toLowerCase();

  for (const table of requiredTables) {
    const pattern = new RegExp(`create\\s+table\\s+(if\\s+not\\s+exists\\s+)?${table}\\b`);
    assert.match(migrationSql, pattern, `${engine} migrations must create ${table}`);
    assert.match(baseline, pattern, `${engine} baseline must create ${table}`);
  }

  if (engine === "postgres") {
    assert.doesNotMatch(migrationSql, /\bbigserial\b|\bserial\b/u, "PostgreSQL IDs must be application generated");
    assert.doesNotMatch(
      migrationSql,
      /\b(?:numeric|decimal)\s*\(/u,
      "native SQL Any profile floating-point scores must use DOUBLE PRECISION",
    );
    assert.doesNotMatch(
      migrationSql,
      /\bjsonb\b|\btimestamptz\b/u,
      "PostgreSQL native-sql storage must retain the cross-engine TEXT physical profile used by SQLx Any",
    );
  } else {
    assert.doesNotMatch(migrationSql, /\bid\s+integer\s+primary\s+key\b/u, "SQLite business IDs must not use rowid allocation");
  }
}

for (const engine of ["postgres", "sqlite"]) {
  const pluginMigrationRoot = path.join(
    root,
    "plugins",
    "sdkwork-memory-plugin-native-sql",
    "migrations",
    engine,
  );
  const legacySql = fs.readdirSync(pluginMigrationRoot).filter((name) => name.endsWith(".sql"));
  assert.deepEqual(legacySql, [], `${pluginMigrationRoot} must not remain a second migration authority`);
}

const nativeSqlRoot = path.join(root, "plugins", "sdkwork-memory-plugin-native-sql");
for (const sourceDirectory of ["src", "tests"]) {
  const sourceRoot = path.join(nativeSqlRoot, sourceDirectory);
  for (const sourceName of fs.readdirSync(sourceRoot).filter((name) => name.endsWith(".rs"))) {
    const source = fs.readFileSync(path.join(sourceRoot, sourceName), "utf8");
    for (const match of source.matchAll(/insert\s+into\s+(ai_[a-z_]+)\s*\(([^)]+)\)/giu)) {
      const table = match[1].toLowerCase();
      if (table.endsWith("_fts") || table === "ai_schema_migration") continue;
      const columns = match[2].split(",").map((column) => column.trim().toLowerCase());
      assert.ok(
        columns.includes("id"),
        `${sourceDirectory}/${sourceName} insert into ${table} must bind an approved application-generated id`,
      );
    }
  }
}

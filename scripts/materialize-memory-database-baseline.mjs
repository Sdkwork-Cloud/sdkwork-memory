#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, relative, resolve, sep } from "node:path";

const root = resolve(import.meta.dirname, "..");
const checkOnly = process.argv.includes("--check");

for (const engine of ["postgres", "sqlite"]) {
  const migrationRoot = join(root, "database", "migrations", engine);
  const migrationFiles = readdirSync(migrationRoot)
    .filter((name) => /^\d+_[a-z0-9_]+\.up\.sql$/u.test(name))
    .sort();
  if (migrationFiles.length === 0) {
    throw new Error(`no canonical ${engine} migrations found`);
  }
  for (const upFile of migrationFiles) {
    const downFile = upFile.replace(/\.up\.sql$/u, ".down.sql");
    if (!existsSync(join(migrationRoot, downFile))) {
      throw new Error(`canonical migration ${engine}/${upFile} is missing ${downFile}`);
    }
  }

  const generated = [
    "-- Generated from canonical application-root migrations.",
    "-- Do not edit this folded baseline directly; run `pnpm db:materialize:baseline`.",
    "",
    ...migrationFiles.flatMap((name) => {
      const path = join(migrationRoot, name);
      return [
        `-- source: ${portable(relative(root, path))}`,
        readFileSync(path, "utf8").replace(/\r\n/gu, "\n").trimEnd(),
        "",
      ];
    }),
  ].join("\n");
  const outputPath = join(
    root,
    "database",
    "ddl",
    "baseline",
    engine,
    "0001_memory_baseline.sql",
  );
  if (checkOnly) {
    const current = readFileSync(outputPath, "utf8").replace(/\r\n/gu, "\n");
    if (current !== generated) {
      throw new Error(`${portable(relative(root, outputPath))} is stale; run pnpm db:materialize:baseline`);
    }
  } else {
    writeFileSync(outputPath, generated, "utf8");
  }
}

console.log(`Memory database baselines ${checkOnly ? "are current" : "materialized"}`);

function portable(value) {
  return value.split(sep).join("/");
}

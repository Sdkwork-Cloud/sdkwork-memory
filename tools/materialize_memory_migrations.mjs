#!/usr/bin/env node
/**
 * Sync canonical database migrations from the native-sql plugin authority.
 * Source: plugins/sdkwork-memory-plugin-native-sql/migrations/
 * Target: database/migrations/{postgres,sqlite}/
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import crypto from "node:crypto";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const mappings = [
  {
    source: "V202606100001__memory_phase1.sql",
    target: "0001_memory_phase1.up.sql",
  },
  {
    source: "V202606100002__memory_phase1_indexes.sql",
    target: "0002_memory_phase1_indexes.up.sql",
  },
  {
    source: "V202606230001__mem_tenant_preference.sql",
    target: "0003_ai_tenant_preference.up.sql",
  },
  {
    source: "V202606240001__ai_learning_job.sql",
    target: "0004_ai_learning_job.up.sql",
  },
  {
    source: "V202606240002__ai_record_fulltext_search.sql",
    target: "0005_ai_record_fulltext_search.up.sql",
  },
  {
    source: "V202606250001__ai_eval_run_extend.sql",
    target: "0006_ai_eval_run_extend.up.sql",
  },
  {
    source: "V202606250002__memory_commercial_management.sql",
    target: "0007_memory_commercial_management.up.sql",
  },
];

for (const engine of ["postgres", "sqlite"]) {
  for (const { source, target } of mappings) {
    const from = path.join(
      root,
      "plugins/sdkwork-memory-plugin-native-sql/migrations",
      engine,
      source,
    );
    const to = path.join(root, "database/migrations", engine, target);
    if (!fs.existsSync(from)) {
      throw new Error(`missing plugin migration source: ${from}`);
    }
    const sql = fs.readFileSync(from, "utf8");
    fs.mkdirSync(path.dirname(to), { recursive: true });
    fs.writeFileSync(to, sql);
    const digest = crypto.createHash("sha256").update(sql).digest("hex");
    process.stdout.write(
      `materialized ${path.relative(root, to)} (sha256:${digest.slice(0, 12)})\n`,
    );
  }
}

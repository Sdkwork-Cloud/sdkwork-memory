import assert from "node:assert/strict";
import fs from "node:fs";

const spiDesignPath =
  "docs/architecture/tech/TECH-2026-06-10-memory-spi-plugin-architecture-design.md";
const legacySpiDesignStub =
  "docs/superpowers/specs/2026-06-10-memory-spi-plugin-architecture-design.md";
const materializerPath = "tools/materialize_phase1_contracts.mjs";
const specsReadmePath = "specs/README.md";

assert.ok(fs.existsSync(spiDesignPath), `${spiDesignPath} must exist`);
assert.ok(fs.existsSync(legacySpiDesignStub), `${legacySpiDesignStub} redirect stub must exist`);

const spiDesign = fs.readFileSync(spiDesignPath, "utf8");
for (const requiredText of [
  "## 1. Purpose",
  "## 5. Stable Core And Plugin Boundaries",
  "## 7. SPI Port Families",
  "## 8. Runtime Plugin Manifest",
  "## 10. Built-In Plugin Families",
  "## 14. Conformance And Verification",
]) {
  assert.ok(spiDesign.includes(requiredText), `SPI design must include ${requiredText}`);
}

for (const requiredText of [
  "MemoryPluginManifest",
  "MemoryRuntimePlugin",
  "MemoryCoreRuntime",
  "native_sql",
  "external_provider_bridge",
  "Conformance suite",
  "0.1.0 Implementation Decisions",
  "Static Rust registration",
  "JSON manifest plus Rust constant",
  "Runtime plugins are not Codex agent plugins",
  "Do not place runtime Memory plugins under `.sdkwork/plugins/`",
  "Industry References",
]) {
  assert.ok(spiDesign.includes(requiredText), `SPI design must define ${requiredText}`);
}

assert.ok(
  !spiDesign.includes("## 17. Open Decisions"),
  "SPI design must resolve first-landing open decisions before runtime implementation starts",
);

const legacyStub = fs.readFileSync(legacySpiDesignStub, "utf8");
assert.ok(legacyStub.includes("Migrated"), `${legacySpiDesignStub} must remain a redirect stub`);
assert.ok(
  legacyStub.includes(spiDesignPath),
  `${legacySpiDesignStub} must point to canonical SPI design`,
);

const materializer = fs.readFileSync(materializerPath, "utf8");
assert.ok(
  materializer.includes(spiDesignPath),
  `${materializerPath} must materialize README references to the SPI design`,
);

const specsReadme = fs.readFileSync(specsReadmePath, "utf8");
assert.ok(
  specsReadme.includes(spiDesignPath),
  `${specsReadmePath} must list the SPI design as local design authority`,
);

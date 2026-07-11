import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

function collectFiles(root, predicate) {
  if (!fs.existsSync(root)) {
    return [];
  }

  const result = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const fullPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      result.push(...collectFiles(fullPath, predicate));
    } else if (predicate(fullPath)) {
      result.push(fullPath.replaceAll(path.sep, "/"));
    }
  }
  return result;
}

const runtimePluginManifests = collectFiles(
  "plugins",
  (file) => path.basename(file) === "sdkwork.memory.plugin.json",
);
const agentPluginRuntimeManifests = collectFiles(
  ".sdkwork/plugins",
  (file) => path.basename(file) === "sdkwork.memory.plugin.json",
);

const expectedPluginIds = new Set([
  "sdkwork-memory-plugin-native-sql",
  "sdkwork-memory-plugin-reference-profiles",
]);

const requiredCanonicalSpecs = [
  "SOUL.md",
  "COMPONENT_SPEC.md",
  "CODE_STYLE_SPEC.md",
  "NAMING_SPEC.md",
  "MODULE_SPEC.md",
  "RUST_CODE_SPEC.md",
  "CONFIG_SPEC.md",
  "DEPLOYMENT_SPEC.md",
  "SECURITY_SPEC.md",
  "PRIVACY_SPEC.md",
  "TEST_SPEC.md",
];

function sorted(values) {
  return [...values].sort();
}

assert.ok(
  runtimePluginManifests.includes(
    "plugins/sdkwork-memory-plugin-native-sql/sdkwork.memory.plugin.json",
  ),
  "native SQL runtime plugin manifest must live under plugins/",
);
assert.ok(
  runtimePluginManifests.includes(
    "plugins/sdkwork-memory-plugin-reference-profiles/sdkwork.memory.plugin.json",
  ),
  "reference implementation profile runtime plugin manifest must live under plugins/",
);
assert.deepEqual(
  agentPluginRuntimeManifests,
  [],
  "runtime Memory plugin manifests must not live under .sdkwork/plugins/",
);

for (const manifestPath of runtimePluginManifests) {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  const pluginRoot = path.dirname(manifestPath);
  const componentSpecPath = path.join(pluginRoot, "specs", "component.spec.json");
  const componentReadmePath = path.join(pluginRoot, "specs", "README.md");
  const pluginReadmePath = path.join(pluginRoot, "README.md");

  assert.equal(manifest.kind, "sdkwork.memory.plugin");
  assert.ok(
    manifest.pluginId.startsWith("sdkwork-memory-plugin-"),
    `${manifestPath} pluginId must use SDKWork Memory runtime plugin naming`,
  );

  for (const requiredPath of [
    "Cargo.toml",
    "src/lib.rs",
    "src/manifest.rs",
    "tests",
    "sdkwork.memory.plugin.json",
    "README.md",
    "specs/README.md",
    "specs/component.spec.json",
  ]) {
    assert.ok(
      fs.existsSync(path.join(pluginRoot, requiredPath)),
      `${manifestPath} plugin module must contain ${requiredPath}`,
    );
  }

  const componentSpec = JSON.parse(fs.readFileSync(componentSpecPath, "utf8"));
  const component = componentSpec.component;
  const contracts = componentSpec.contracts;
  assert.equal(componentSpec.kind, "sdkwork.component.spec");
  assert.equal(component.name, manifest.pluginId);
  assert.equal(component.version, manifest.version);
  assert.equal(component.type, "rust-crate");
  assert.equal(component.root, `sdkwork-memory/${pluginRoot}`);
  assert.equal(component.domain, "intelligence");
  assert.equal(component.capability, "memory");
  assert.equal(component.surface, "runtime-plugin");
  assert.deepEqual(component.languages, ["rust"]);
  assert.equal(component.generated, false);
  assert.deepEqual(
    sorted(component.manifests),
    ["Cargo.toml", "sdkwork.memory.plugin.json", "specs/component.spec.json"],
  );

  const canonicalSpecs = componentSpec.canonicalSpecs;
  const canonicalFiles = new Set(canonicalSpecs.map((entry) => entry.file));
  for (const requiredSpec of requiredCanonicalSpecs) {
    assert.ok(
      canonicalFiles.has(requiredSpec),
      `${manifestPath} component spec must link ${requiredSpec}`,
    );
  }
  for (const entry of canonicalSpecs) {
    assert.ok(
      fs.existsSync(path.join(pluginRoot, entry.path)),
      `${manifestPath} canonical spec path ${entry.path} must resolve from the plugin root`,
    );
  }

  assert.equal(contracts.layerRole, "backend-provider");
  assert.deepEqual(contracts.publicExports, ["."]);
  assert.deepEqual(
    sorted(contracts.providedPorts.map((port) => port.name)),
    sorted(manifest.portExports.map((port) => port.port)),
    `${manifestPath} component provided ports must project the runtime manifest`,
  );
  for (const port of contracts.providedPorts) {
    assert.equal(port.export, ".");
  }
  assert.ok(
    contracts.requiredPorts.some(
      (port) => port.name === "MemoryPluginSpi" && port.dependency === "sdkwork-memory-spi",
    ),
    `${manifestPath} must declare the Memory SPI dependency`,
  );
  for (const port of manifest.portExports) {
    assert.ok(
      contracts.runtimeEntrypoints.some((entry) => entry.endsWith(`::${port.builder}`)),
      `${manifestPath} must expose executable entrypoint ${port.builder}`,
    );
  }
  assert.deepEqual(contracts.sdkClients, []);
  assert.deepEqual(contracts.sdkDependencies, []);
  assert.equal(contracts.runtimePlugin.manifest, "sdkwork.memory.plugin.json");
  assert.equal(contracts.runtimePlugin.manifestKind, "sdkwork.memory.plugin");
  assert.equal(typeof contracts.runtimePlugin.deploymentQualification, "object");
  assert.equal(typeof contracts.runtimePlugin.deploymentQualification.state, "string");
  assert.ok(Array.isArray(contracts.runtimePlugin.deploymentQualification.eligibleDeploymentProfiles));
  assert.ok(Array.isArray(contracts.runtimePlugin.deploymentQualification.eligibleRuntimeTargets));
  assert.ok(
    fs.readFileSync(pluginReadmePath, "utf8").includes("sdkwork.memory.plugin.json"),
    `${manifestPath} README must identify the runtime manifest`,
  );
  assert.ok(
    fs.readFileSync(componentReadmePath, "utf8").includes("component.spec.json"),
    `${manifestPath} specs README must identify the component contract`,
  );

  if (manifest.pluginId === "sdkwork-memory-plugin-native-sql") {
    assert.equal(component.status, "active");
    assert.deepEqual(
      contracts.runtimePlugin.deploymentQualification,
      {
        state: "production-baseline",
        eligibleDeploymentProfiles: ["standalone", "cloud"],
        eligibleRuntimeTargets: ["server", "container"],
        testOnlyModes: ["test"],
      },
    );
    assert.ok(manifest.deploymentModes.includes("server"));
    assert.ok(manifest.deploymentModes.includes("container"));
  }

  if (manifest.pluginId === "sdkwork-memory-plugin-reference-profiles") {
    assert.equal(component.status, "evaluation-only");
    assert.deepEqual(
      contracts.runtimePlugin.deploymentQualification,
      {
        state: "evaluation-only",
        eligibleDeploymentProfiles: [],
        eligibleRuntimeTargets: ["test-runner"],
        allowedPluginModes: ["test", "eval_only"],
        productionSelection: "forbidden",
      },
    );
    assert.deepEqual(sorted(manifest.deploymentModes), ["eval_only", "test"]);
    assert.ok(
      fs.readFileSync(pluginReadmePath, "utf8").includes("evaluation-only"),
      `${manifestPath} README must state evaluation-only qualification`,
    );
  }
}

for (const expectedPluginId of expectedPluginIds) {
  assert.ok(
    runtimePluginManifests.some((manifestPath) => {
      const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
      return manifest.pluginId === expectedPluginId;
    }),
    `${expectedPluginId} must remain registered as a phase-1 runtime plugin`,
  );
}

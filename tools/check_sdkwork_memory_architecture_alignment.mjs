#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];
const warnings = [];

function readText(relativePath) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(absolutePath)) {
    failures.push(`${relativePath} must exist`);
    return '';
  }
  return fs.readFileSync(absolutePath, 'utf8');
}

function readJson(relativePath) {
  return JSON.parse(readText(relativePath));
}

function assert(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function assertDirectory(relativePath) {
  assert(fs.existsSync(path.join(repoRoot, relativePath)), `${relativePath}/ must exist`);
}

function assertCargoDependsOnWebFramework(relativeCrateToml) {
  const text = readText(relativeCrateToml);
  assert(
    text.includes('sdkwork-web-axum.workspace = true')
      || text.includes('sdkwork-web-axum = {'),
    `${relativeCrateToml} must depend on sdkwork-web-axum per WEB_FRAMEWORK_SPEC.md`,
  );
}

const requiredDirectories = [
  'apis',
  'apps',
  'crates',
  'sdks',
  'deployments',
  'configs',
  'scripts',
  'docs',
  'tests',
  '.sdkwork',
  'specs',
];

for (const directory of requiredDirectories) {
  assertDirectory(directory);
}

assert(fs.existsSync(path.join(repoRoot, 'sdkwork.app.config.json')), 'sdkwork.app.config.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'sdkwork.workflow.json')), 'sdkwork.workflow.json must exist');
assert(fs.existsSync(path.join(repoRoot, 'package.json')), 'package.json must exist per PNPM_SCRIPT_SPEC.md');
assert(
  fs.existsSync(path.join(repoRoot, '.github/workflows/package.yml')),
  '.github/workflows/package.yml must exist per GITHUB_WORKFLOW_SPEC.md',
);

const packageJson = readJson('package.json');
for (const script of ['dev', 'build', 'test', 'check', 'verify', 'clean']) {
  assert(packageJson.scripts?.[script], `package.json must expose pnpm ${script}`);
}

const cargoToml = readText('Cargo.toml');
assert(cargoToml.includes('sdkwork-web-core'), 'Cargo.toml must declare sdkwork-web-core');
assert(cargoToml.includes('sdkwork-web-axum'), 'Cargo.toml must declare sdkwork-web-axum');
assert(cargoToml.includes('sdkwork-iam-web-adapter'), 'Cargo.toml must declare sdkwork-iam-web-adapter');
assert(cargoToml.includes('sdkwork-database-config'), 'Cargo.toml must declare sdkwork-database-config');
assert(cargoToml.includes('sdkwork-database-sqlx'), 'Cargo.toml must declare sdkwork-database-sqlx');
assert(cargoToml.includes('sdkwork-database-repository'), 'Cargo.toml must declare sdkwork-database-repository');
assert(cargoToml.includes('sdkwork-memory-api-server'), 'Cargo.toml must include sdkwork-memory-api-server');
assert(cargoToml.includes('sdkwork-intelligence-memory-repository-sqlx'), 'Cargo.toml must include repository-sqlx crate');
assert(!cargoToml.includes('sdkwork-discovery'), 'sdkwork-discovery is not required until RPC services exist');

const workflow = readJson('sdkwork.workflow.json');
const dependencyIds = new Set((workflow.dependencies || []).map((dependency) => dependency.id));
for (const dependencyId of [
  'sdkwork-appbase',
  'sdkwork-database',
  'sdkwork-web-framework',
  'sdkwork-sdk-generator',
]) {
  assert(dependencyIds.has(dependencyId), `sdkwork.workflow.json must declare ${dependencyId}`);
}
assert(!dependencyIds.has('sdkwork-discovery'), 'sdkwork.workflow.json must not declare sdkwork-discovery until RPC exists');

const routerCrates = [
  'crates/sdkwork-router-memory-open-api/Cargo.toml',
  'crates/sdkwork-router-memory-app-api/Cargo.toml',
  'crates/sdkwork-router-memory-backend-api/Cargo.toml',
];

for (const routerCrate of routerCrates) {
  assertCargoDependsOnWebFramework(routerCrate);
  const crateName = path.basename(path.dirname(routerCrate));
  assert(
    fs.existsSync(path.join(repoRoot, `crates/${crateName}/src/web_bootstrap.rs`)),
    `${crateName} must provide web_bootstrap.rs`,
  );
}

const repositorySqlxToml = readText('crates/sdkwork-intelligence-memory-repository-sqlx/Cargo.toml');
assert(
  repositorySqlxToml.includes('sdkwork-database-sqlx'),
  'repository-sqlx crate must depend on sdkwork-database-sqlx',
);
assert(
  repositorySqlxToml.includes('sdkwork-database-repository'),
  'repository-sqlx crate must depend on sdkwork-database-repository per DATABASE_SPEC.md',
);
assert(
  repositorySqlxToml.includes('migrate'),
  'repository-sqlx sqlx dependency must enable migrate feature',
);

const componentSpec = readJson('specs/component.spec.json');
const sdkDependencyIds = new Set((componentSpec.contracts?.sdkDependencies ?? []).map((item) => item.workspace));
for (const workspace of [
  'sdkwork-web-framework',
  'sdkwork-database',
  'sdkwork-appbase',
  'sdkwork-id',
  'sdkwork-sdk-generator',
]) {
  assert(
    sdkDependencyIds.has(workspace),
    `specs/component.spec.json must declare sdkDependencies workspace ${workspace}`,
  );
}

const routeManifestPaths = [
  'sdks/_route-manifests/open-api/sdkwork-router-memory-open-api.route-manifest.json',
  'sdks/_route-manifests/app-api/sdkwork-router-memory-app-api.route-manifest.json',
  'sdks/_route-manifests/backend-api/sdkwork-router-memory-backend-api.route-manifest.json',
];

for (const relativePath of routeManifestPaths) {
  const manifest = readJson(relativePath);
  for (const route of manifest.routes ?? []) {
    assert(
      route.requestContext === 'WebRequestContext',
      `${relativePath} route ${route.method} ${route.path} must declare WebRequestContext`,
    );
    assert(
      ['open-api', 'app-api', 'backend-api'].includes(route.apiSurface),
      `${relativePath} route ${route.method} ${route.path} must declare canonical apiSurface`,
    );
  }
}

assert(componentSpec.component.type === 'web-backend-service', 'component type must be web-backend-service');
assert(componentSpec.component.domain === 'intelligence', 'component domain must be intelligence');
assert(componentSpec.component.capability === 'memory', 'component capability must be memory');

const canonicalSpecs = (componentSpec.canonicalSpecs || []).map((entry) => entry.file);
for (const specFile of ['WEB_FRAMEWORK_SPEC.md', 'WEB_BACKEND_SPEC.md', 'DATABASE_SPEC.md', 'DEPLOYMENT_SPEC.md']) {
  assert(canonicalSpecs.includes(specFile), `specs/component.spec.json must reference ${specFile}`);
}

const openapiPaths = [
  'sdks/sdkwork-memory-sdk/openapi/memory-open-api.openapi.json',
  'sdks/sdkwork-memory-app-sdk/openapi/memory-app-api.openapi.json',
  'sdks/sdkwork-memory-backend-sdk/openapi/memory-backend-api.openapi.json',
];

for (const relativePath of openapiPaths) {
  const openapi = readJson(relativePath);
  let hasSurface = false;
  for (const pathItem of Object.values(openapi.paths ?? {})) {
    for (const operation of Object.values(pathItem ?? {})) {
      if (operation && typeof operation === 'object' && operation.operationId) {
        assert(
          operation['x-sdkwork-request-context'] === 'WebRequestContext',
          `${relativePath} operation ${operation.operationId} must declare WebRequestContext`,
        );
        assert(
          ['open-api', 'app-api', 'backend-api'].includes(operation['x-sdkwork-api-surface']),
          `${relativePath} operation ${operation.operationId} must declare canonical x-sdkwork-api-surface`,
        );
        hasSurface = true;
      }
    }
  }
  if (!hasSurface) {
    assert(false, `${relativePath} must declare x-sdkwork-api-surface on operations`);
  }
}

const requiredSkeletonPaths = [
  'apis/README.md',
  'apis/authority-manifest.json',
  'apis/open-api/intelligence/memory/README.md',
  'apis/app-api/intelligence/memory/README.md',
  'apis/backend-api/intelligence/memory/README.md',
  'apis/rpc/README.md',
  'deployments/docker/README.md',
  'deployments/kubernetes/README.md',
  'deployments/runbooks/README.md',
  'configs/README.md',
  'scripts/README.md',
  'apps/README.md',
  'specs/topology.spec.json',
];

for (const relativePath of requiredSkeletonPaths) {
  assert(
    fs.existsSync(path.join(repoRoot, relativePath)),
    `${relativePath} must exist per SDKWORK_WORKSPACE_SPEC.md skeleton`,
  );
}

if (failures.length > 0) {
  process.stderr.write(
    `Architecture alignment failed:\n${failures.map((failure) => `- ${failure}`).join('\n')}\n`,
  );
  if (warnings.length > 0) {
    process.stderr.write(
      `Warnings:\n${warnings.map((warning) => `- ${warning}`).join('\n')}\n`,
    );
  }
  process.exit(1);
}

if (warnings.length > 0) {
  process.stdout.write(
    `Architecture alignment passed with warnings:\n${warnings.map((warning) => `- ${warning}`).join('\n')}\n`,
  );
} else {
  process.stdout.write('Architecture alignment passed\n');
}

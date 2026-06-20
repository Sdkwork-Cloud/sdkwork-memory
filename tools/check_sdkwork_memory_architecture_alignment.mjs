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
  'database',
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
assert(
  packageJson.scripts?.['check:architecture-alignment'],
  'package.json must expose pnpm check:architecture-alignment',
);
assert(packageJson.scripts?.['topology:validate'], 'package.json must expose pnpm topology:validate');
assert(
  packageJson.scripts?.['check:pnpm-script-standard'],
  'package.json must expose pnpm check:pnpm-script-standard',
);
assert(
  packageJson.scripts?.['check:agent-workflow-standard'],
  'package.json must expose pnpm check:agent-workflow-standard',
);
assert(packageJson.dependencies?.['@sdkwork/app-topology'], 'package.json must declare @sdkwork/app-topology');

const cargoToml = readText('Cargo.toml');
assert(cargoToml.includes('sdkwork-web-core'), 'Cargo.toml must declare sdkwork-web-core');
assert(cargoToml.includes('sdkwork-web-axum'), 'Cargo.toml must declare sdkwork-web-axum');
assert(cargoToml.includes('sdkwork-iam-web-adapter'), 'Cargo.toml must declare sdkwork-iam-web-adapter');
assert(cargoToml.includes('sdkwork-database-config'), 'Cargo.toml must declare sdkwork-database-config');
assert(cargoToml.includes('sdkwork-database-sqlx'), 'Cargo.toml must declare sdkwork-database-sqlx');
assert(cargoToml.includes('sdkwork-database-repository'), 'Cargo.toml must declare sdkwork-database-repository');
assert(cargoToml.includes('sdkwork-utils-rust'), 'Cargo.toml must declare sdkwork-utils-rust');
assert(cargoToml.includes('sdkwork-id-core'), 'Cargo.toml must declare sdkwork-id-core');
assert(cargoToml.includes('sdkwork-memory-api-server'), 'Cargo.toml must include sdkwork-memory-api-server');
assert(cargoToml.includes('sdkwork-intelligence-memory-repository-sqlx'), 'Cargo.toml must include repository-sqlx crate');
assert(!cargoToml.includes('sdkwork-discovery'), 'sdkwork-discovery is not required until RPC services exist');

const workflow = readJson('sdkwork.workflow.json');
const dependencyIds = new Set((workflow.dependencies || []).map((dependency) => dependency.id));
for (const dependencyId of [
  'sdkwork-appbase',
  'sdkwork-database',
  'sdkwork-web-framework',
  'sdkwork-utils',
  'sdkwork-id',
  'sdkwork-sdk-generator',
  'sdkwork-app-topology',
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
  assert(
    fs.existsSync(path.join(repoRoot, `crates/${crateName}/src/manifest.rs`)),
    `${crateName} must provide manifest.rs`,
  );
  assert(
    fs.existsSync(path.join(repoRoot, `crates/${crateName}/README.md`)),
    `${crateName} must provide README.md`,
  );
  assert(
    fs.existsSync(path.join(repoRoot, `crates/${crateName}/specs/component.spec.json`)),
    `${crateName} must provide specs/component.spec.json`,
  );
}

for (const routeTest of [
  'crates/sdkwork-router-memory-open-api/tests/open_api_routes.rs',
  'crates/sdkwork-router-memory-open-api/tests/open_web_framework_routes.rs',
  'crates/sdkwork-router-memory-open-api/tests/open_openapi_routes.rs',
  'crates/sdkwork-router-memory-app-api/tests/app_api_routes.rs',
  'crates/sdkwork-router-memory-app-api/tests/app_web_framework_routes.rs',
  'crates/sdkwork-router-memory-app-api/tests/app_openapi_routes.rs',
  'crates/sdkwork-router-memory-backend-api/tests/backend_api_routes.rs',
  'crates/sdkwork-router-memory-backend-api/tests/backend_web_framework_routes.rs',
  'crates/sdkwork-router-memory-backend-api/tests/backend_openapi_routes.rs',
]) {
  assert(fs.existsSync(path.join(repoRoot, routeTest)), `${routeTest} must exist`);
}

assert(
  fs.existsSync(path.join(repoRoot, 'deployments/docker/Dockerfile')),
  'deployments/docker/Dockerfile must exist per DEPLOYMENT_SPEC.md',
);
assert(
  fs.existsSync(path.join(repoRoot, 'scripts/memory-dev.mjs')),
  'scripts/memory-dev.mjs must exist',
);

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
  repositorySqlxToml.includes('sdkwork-utils-rust'),
  'repository-sqlx crate must depend on sdkwork-utils-rust',
);
assert(
  repositorySqlxToml.includes('migrate'),
  'repository-sqlx sqlx dependency must enable migrate feature',
);

const serviceToml = readText('crates/sdkwork-intelligence-memory-service/Cargo.toml');
assert(
  serviceToml.includes('sdkwork-utils-rust'),
  'service crate must depend on sdkwork-utils-rust for shared utility helpers',
);
assert(
  serviceToml.includes('sdkwork-id-core'),
  'service crate must depend on sdkwork-id-core for snowflake id generation',
);

const componentSpec = readJson('specs/component.spec.json');
const sdkDependencyIds = new Set((componentSpec.contracts?.sdkDependencies ?? []).map((item) => item.workspace));
for (const workspace of [
  'sdkwork-web-framework',
  'sdkwork-database',
  'sdkwork-utils',
  'sdkwork-appbase',
  'sdkwork-id',
  'sdkwork-sdk-generator',
]) {
  assert(
    sdkDependencyIds.has(workspace),
    `specs/component.spec.json must declare sdkDependencies workspace ${workspace}`,
  );
}

assert(!sdkDependencyIds.has('sdkwork-discovery'), 'component spec must not declare sdkwork-discovery until RPC exists');

assert(fs.existsSync(path.join(repoRoot, '.env.example')), '.env.example must exist');
assert(fs.existsSync(path.join(repoRoot, '.sdkwork/.gitignore')), '.sdkwork/.gitignore must exist');
assert(fs.existsSync(path.join(repoRoot, 'docs/topology-standard.md')), 'docs/topology-standard.md must exist');
assert(
  fs.existsSync(path.join(repoRoot, 'scripts/lib/memory-topology.mjs')),
  'scripts/lib/memory-topology.mjs must exist',
);
assert(
  fs.existsSync(path.join(repoRoot, 'scripts/generate-memory-sdk.mjs')),
  'scripts/generate-memory-sdk.mjs must exist',
);
assert(
  fs.existsSync(path.join(repoRoot, 'sdks/standardize-memory-sdk-family.mjs')),
  'sdks/standardize-memory-sdk-family.mjs must exist',
);

const topologySpec = readJson('specs/topology.spec.json');
assert(topologySpec.schemaVersion === 2, 'specs/topology.spec.json schemaVersion must be 2');
assert(topologySpec.archetype === 'application-http-gateway', 'topology archetype must be application-http-gateway');
for (const profileId of [
  topologySpec.defaults?.developmentProfileId,
  topologySpec.defaults?.productionProfileId,
]) {
  assert(profileId, 'topology defaults must declare development and production profile ids');
  assert(
    topologySpec.profileFiles?.[profileId],
    `specs/topology.spec.json must declare profileFiles.${profileId}`,
  );
  assert(
    fs.existsSync(path.join(repoRoot, topologySpec.profileFiles[profileId])),
    `${topologySpec.profileFiles[profileId]} must exist`,
  );
}
assert(
  fs.existsSync(path.join(repoRoot, 'configs/topology/standalone.unified-process.production.env')),
  'configs/topology/standalone.unified-process.production.env must exist',
);
assert(
  fs.existsSync(path.join(repoRoot, 'sdks/test/verify-sdk-ownership-boundaries.test.mjs')),
  'sdks/test/verify-sdk-ownership-boundaries.test.mjs must exist',
);
assert(
  fs.existsSync(path.join(repoRoot, 'tools/verify_sdkwork_structure.ps1')),
  'tools/verify_sdkwork_structure.ps1 must exist',
);
assert(
  fs.existsSync(path.join(repoRoot, 'tools/verify_openapi_operation_ids.ps1')),
  'tools/verify_openapi_operation_ids.ps1 must exist',
);
assert(
  fs.existsSync(path.join(repoRoot, 'deployments/kubernetes/deployment.yaml')),
  'deployments/kubernetes/deployment.yaml must exist per DEPLOYMENT_SPEC.md',
);
assert(
  fs.existsSync(path.join(repoRoot, 'deployments/kubernetes/service.yaml')),
  'deployments/kubernetes/service.yaml must exist per DEPLOYMENT_SPEC.md',
);
assert(
  fs.existsSync(path.join(repoRoot, 'tests/contract/database-framework.contract.test.mjs')),
  'tests/contract/database-framework.contract.test.mjs must exist',
);
assert(
  fs.existsSync(path.join(repoRoot, '.github/workflows/verify.yml')),
  '.github/workflows/verify.yml must exist per GITHUB_WORKFLOW_SPEC.md',
);

const authorityManifest = readJson('apis/authority-manifest.json');
for (const surface of authorityManifest.surfaces ?? []) {
  assert(surface.authorityPath, 'authority manifest surface must declare authorityPath');
  assert(surface.sdkPath, 'authority manifest surface must declare sdkPath');
  assert(
    fs.existsSync(path.join(repoRoot, surface.authorityPath)),
    `${surface.authorityPath} must exist`,
  );
  assert(fs.existsSync(path.join(repoRoot, surface.sdkPath)), `${surface.sdkPath} must exist`);
}

const sdkFamilyRoots = [
  'sdks/sdkwork-memory-sdk',
  'sdks/sdkwork-memory-app-sdk',
  'sdks/sdkwork-memory-backend-sdk',
];
for (const familyRoot of sdkFamilyRoots) {
  const manifest = readJson(path.join(familyRoot, 'sdk-manifest.json'));
  assert(manifest.standardProfile === 'sdkwork-v3', `${familyRoot} must declare standardProfile sdkwork-v3`);
  assert(manifest.generatedOutput, `${familyRoot} must declare generatedOutput`);
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
for (const specFile of [
  'WEB_FRAMEWORK_SPEC.md',
  'WEB_BACKEND_SPEC.md',
  'DATABASE_SPEC.md',
  'DEPLOYMENT_SPEC.md',
  'SDK_SPEC.md',
  'SDK_WORKSPACE_GENERATION_SPEC.md',
  'TEST_SPEC.md',
]) {
  assert(canonicalSpecs.includes(specFile), `specs/component.spec.json must reference ${specFile}`);
}

const crateComponentSpecs = [
  'crates/sdkwork-memory-contract/specs/component.spec.json',
  'crates/sdkwork-intelligence-memory-service/specs/component.spec.json',
  'crates/sdkwork-intelligence-memory-repository-sqlx/specs/component.spec.json',
  'crates/sdkwork-memory-database-host/specs/component.spec.json',
  'crates/sdkwork-memory-api-server/specs/component.spec.json',
];
for (const relativePath of crateComponentSpecs) {
  assert(fs.existsSync(path.join(repoRoot, relativePath)), `${relativePath} must exist`);
}

const requiredGeneratedSdkRoots = [
  'sdks/sdkwork-memory-sdk/sdkwork-memory-sdk-typescript/generated/server-openapi',
  'sdks/sdkwork-memory-app-sdk/sdkwork-memory-app-sdk-typescript/generated/server-openapi',
  'sdks/sdkwork-memory-backend-sdk/sdkwork-memory-backend-sdk-typescript/generated/server-openapi',
];
for (const relativePath of requiredGeneratedSdkRoots) {
  assert(fs.existsSync(path.join(repoRoot, relativePath)), `${relativePath} must exist`);
  for (const requiredFile of ['sdkwork-sdk.json', 'package.json', 'src/index.ts']) {
    assert(
      fs.existsSync(path.join(repoRoot, relativePath, requiredFile)),
      `${relativePath}/${requiredFile} must exist`,
    );
  }
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

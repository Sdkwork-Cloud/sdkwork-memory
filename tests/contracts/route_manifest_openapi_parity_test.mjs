import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, relativePath), 'utf8'));
}

function listOpenApiRoutes(openapiPath) {
  const openapi = readJson(openapiPath);
  const routes = [];
  for (const [path, methods] of Object.entries(openapi.paths ?? {})) {
    for (const [method, operation] of Object.entries(methods ?? {})) {
      if (['get', 'post', 'put', 'patch', 'delete'].includes(method)) {
        routes.push({
          method: method.toUpperCase(),
          path,
          operationId: operation.operationId,
        });
      }
    }
  }
  return routes;
}

function routeKey(route) {
  return `${route.method} ${route.path} ${route.operationId}`;
}

const surfaces = [
  {
    openapiPath: 'sdks/sdkwork-memory-sdk/openapi/memory-open-api.openapi.json',
    routeManifestPath: 'sdks/_route-manifests/open-api/sdkwork-routes-memory-open-api.route-manifest.json',
    httpRouteManifestPath: 'crates/sdkwork-routes-memory-open-api/src/http_route_manifest.rs',
  },
  {
    openapiPath: 'sdks/sdkwork-memory-app-sdk/openapi/memory-app-api.openapi.json',
    routeManifestPath: 'sdks/_route-manifests/app-api/sdkwork-routes-memory-app-api.route-manifest.json',
    httpRouteManifestPath: 'crates/sdkwork-routes-memory-app-api/src/http_route_manifest.rs',
  },
  {
    openapiPath: 'sdks/sdkwork-memory-backend-sdk/openapi/memory-backend-api.openapi.json',
    routeManifestPath: 'sdks/_route-manifests/backend-api/sdkwork-routes-memory-backend-api.route-manifest.json',
    httpRouteManifestPath: 'crates/sdkwork-routes-memory-backend-api/src/http_route_manifest.rs',
  },
];

for (const surface of surfaces) {
  test(`${surface.openapiPath} route manifest parity`, () => {
    const openapiRoutes = listOpenApiRoutes(surface.openapiPath);
    const routeManifest = readJson(surface.routeManifestPath);
    const expectedRoutes = openapiRoutes.map(routeKey).sort();
    const actualRoutes = routeManifest.routes.map(routeKey).sort();
    assert.equal(new Set(expectedRoutes).size, expectedRoutes.length);
    assert.equal(new Set(actualRoutes).size, actualRoutes.length);
    assert.deepEqual(actualRoutes, expectedRoutes);
    const rustManifest = fs.readFileSync(path.join(repoRoot, surface.httpRouteManifestPath), 'utf8');
    const rustRouteCount = (rustManifest.match(/HttpRoute::/g) ?? []).length;
    assert.equal(rustRouteCount, openapiRoutes.length);
  });
}

test('apis authority manifest mirrors sdk openapi paths', () => {
  const authorityManifest = readJson('apis/authority-manifest.json');
  for (const surface of authorityManifest.surfaces ?? []) {
    assert.ok(fs.existsSync(path.join(repoRoot, surface.authorityPath)));
    assert.ok(fs.existsSync(path.join(repoRoot, surface.sdkPath)));
  }
});

test('apis authority openapi content matches sdk openapi authority copies', () => {
  const authorityManifest = readJson('apis/authority-manifest.json');
  for (const surface of authorityManifest.surfaces ?? []) {
    const authority = fs.readFileSync(path.join(repoRoot, surface.authorityPath), 'utf8');
    const sdkCopy = fs.readFileSync(path.join(repoRoot, surface.sdkPath), 'utf8');
    assert.equal(
      authority,
      sdkCopy,
      `${surface.authorityPath} must match ${surface.sdkPath}`,
    );
  }
});

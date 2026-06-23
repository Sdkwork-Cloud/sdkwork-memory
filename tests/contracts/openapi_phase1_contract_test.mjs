import assert from "node:assert/strict";
import fs from "node:fs";

const open = JSON.parse(
  fs.readFileSync(
    "sdks/sdkwork-memory-sdk/openapi/memory-open-api.openapi.json",
    "utf8",
  ),
);

const operations = Object.values(open.paths).flatMap((pathItem) =>
  ["get", "post", "patch", "delete"]
    .filter((method) => pathItem[method])
    .map((method) => pathItem[method]),
);

assert.equal(operations.length, 17);
assert.ok(open.components.securitySchemes.ApiKey);
assert.ok(!open.components.securitySchemes.AuthToken);
assert.ok(!open.components.securitySchemes.AccessToken);
for (const operation of operations) {
  assert.deepEqual(operation.security, [{ ApiKey: [] }]);
  assert.equal(operation["x-sdkwork-auth-mode"], "api-key");
  assert.equal(operation["x-sdkwork-api-authority"], "sdkwork-memory-open-api");
}

for (const [pathKey, pathItem] of Object.entries(open.paths)) {
  for (const method of ["post", "patch", "delete"]) {
    const operation = pathItem[method];
    if (!operation) {
      continue;
    }
    const expectedTier =
      operation.operationId === "memories.delete" ? "authCritical" : "openApiDefault";
    assert.equal(
      operation["x-sdkwork-rate-limit-tier"],
      expectedTier,
      `${method.toUpperCase()} ${pathKey} must declare ${expectedTier} rate limit tier`,
    );
  }
}

const deleteOp = operations.find((operation) => operation.operationId === "memories.delete");
assert.ok(deleteOp);
assert.equal(deleteOp["x-sdkwork-rate-limit-tier"], "authCritical");

console.log("OpenAPI phase1 contract test passed");

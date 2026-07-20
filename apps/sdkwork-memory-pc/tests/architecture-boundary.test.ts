import { readFileSync, readdirSync, statSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const appRoot = resolve(import.meta.dirname, "..");
const packagesRoot = resolve(appRoot, "packages");

describe("Memory PC architecture boundaries", () => {
  it("keeps backend SDK imports out of console packages", () => {
    const source = readPackageFamily("sdkwork-memory-pc-console-");
    expect(source).not.toContain("@sdkwork/memory-backend-sdk");
    expect(source).not.toContain("/backend/v3/api");
  });

  it("keeps app SDK imports out of backend-admin packages", () => {
    const source = readPackageFamily("sdkwork-memory-pc-admin-");
    expect(source).not.toContain("@sdkwork/memory-app-sdk");
    expect(source).not.toContain("/app/v3/api");
  });

  it("does not use raw business HTTP or manual auth headers in packages", () => {
    const source = readSources(packagesRoot);
    expect(source).not.toMatch(/\bfetch\s*\(/);
    expect(source).not.toMatch(/\baxios\b/);
    expect(source).not.toMatch(/["'](?:Authorization|Access-Token|X-API-Key)["']/);
  });

  it("does not paginate downloaded collections in memory", () => {
    const source = readSources(packagesRoot);
    expect(source).not.toMatch(/\.slice\s*\([^)]*(?:page|pageSize|offset)/);
    expect(source).not.toContain("listAll");
  });
});

function readPackageFamily(prefix: string): string {
  return readdirSync(packagesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name.startsWith(prefix))
    .map((entry) => readSources(resolve(packagesRoot, entry.name, "src")))
    .join("\n");
}

function readSources(root: string): string {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) return readSources(path);
    if (statSync(path).isFile() && /\.(?:ts|tsx)$/.test(path)) return readFileSync(path, "utf8");
    return [];
  }).join("\n");
}

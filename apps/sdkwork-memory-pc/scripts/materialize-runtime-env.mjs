import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const argumentIndex = process.argv.indexOf("--environment");
const environment = argumentIndex >= 0 ? process.argv[argumentIndex + 1] : undefined;
const allowed = new Set(["development", "test", "staging", "production"]);

if (!environment || !allowed.has(environment)) {
  throw new Error("--environment must be development, test, staging, or production");
}

const source = resolve(appRoot, "etc", "browser", `runtime-env.${environment}.example.json`);
const target = resolve(appRoot, "public", "runtime-env.json");
const config = JSON.parse(readFileSync(source, "utf8"));

for (const key of ["appApiBaseUrl", "backendApiBaseUrl", "appbaseAppApiBaseUrl"]) {
  const value = new URL(config[key]);
  if (environment === "production" && ["localhost", "127.0.0.1", "::1"].includes(value.hostname)) {
    throw new Error(`Production ${key} must not use a loopback host`);
  }
}

mkdirSync(dirname(target), { recursive: true });
copyFileSync(source, target);

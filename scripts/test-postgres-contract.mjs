#!/usr/bin/env node

import { randomBytes } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const containerName = `sdkwork-memory-postgres-${process.pid}-${Date.now()}`;
const image = process.env.SDKWORK_POSTGRES_TEST_IMAGE ?? "postgres:16-alpine";
const password = randomBytes(24).toString("hex");
const dockerEnv = { ...process.env, POSTGRES_PASSWORD: password };

try {
  exec("docker", [
    "run",
    "--rm",
    "--detach",
    "--name",
    containerName,
    "--env",
    "POSTGRES_PASSWORD",
    "--env",
    "POSTGRES_DB=memory_lifecycle",
    "--publish",
    "127.0.0.1::5432",
    image,
  ], dockerEnv);
  waitUntilReady(dockerEnv);
  exec("docker", [
    "exec",
    containerName,
    "createdb",
    "--username",
    "postgres",
    "memory_plugin",
  ], dockerEnv);

  const portOutput = execFileSync("docker", ["port", containerName, "5432/tcp"], {
    cwd: root,
    env: dockerEnv,
    encoding: "utf8",
  }).trim();
  const port = portOutput.match(/:(\d+)$/u)?.[1];
  if (!port) throw new Error("failed to resolve ephemeral PostgreSQL host port");

  exec("cargo", [
    "test",
    "-p",
    "sdkwork-memory-database-host",
    "--test",
    "postgres_lifecycle",
    "--",
    "--ignored",
    "--nocapture",
  ], {
    ...process.env,
    SDKWORK_MEMORY_POSTGRES_LIFECYCLE_TEST_URL:
      `postgres://postgres:${password}@127.0.0.1:${port}/memory_lifecycle`,
  });
  exec("cargo", [
    "test",
    "-p",
    "sdkwork-memory-plugin-native-sql",
    "--test",
    "postgres_store_contract",
    "--",
    "--nocapture",
  ], {
    ...process.env,
    SDKWORK_MEMORY_POSTGRES_TEST_URL:
      `postgres://postgres:${password}@127.0.0.1:${port}/memory_plugin`,
  });
} finally {
  spawnSync("docker", ["stop", "--time", "1", containerName], {
    cwd: root,
    env: dockerEnv,
    stdio: "ignore",
  });
}

function waitUntilReady(env) {
  const deadline = Date.now() + 30_000;
  let consecutiveReadyChecks = 0;
  while (Date.now() < deadline) {
    const running = spawnSync(
      "docker",
      ["inspect", "--format", "{{.State.Running}}", containerName],
      { cwd: root, env, encoding: "utf8" },
    );
    if (running.status !== 0 || running.stdout.trim() !== "true") {
      throw new Error("ephemeral PostgreSQL container exited during initialization");
    }
    const result = spawnSync(
      "docker",
      ["exec", containerName, "pg_isready", "--username", "postgres", "--dbname", "memory_lifecycle"],
      { cwd: root, env, stdio: "ignore" },
    );
    consecutiveReadyChecks = result.status === 0 ? consecutiveReadyChecks + 1 : 0;
    // The official image briefly exposes its bootstrap server before restarting.
    // Requiring stable readiness prevents createdb from racing that shutdown.
    if (consecutiveReadyChecks >= 4) return;
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 250);
  }
  throw new Error("ephemeral PostgreSQL did not become ready within 30 seconds");
}

function exec(command, args, env) {
  const result = spawnSync(command, args, { cwd: root, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} failed with exit code ${result.status ?? 1}`);
}

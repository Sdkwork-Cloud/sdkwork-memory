#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import {
  closeSync,
  cpSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readSync,
  renameSync,
  rmSync,
  statSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const workspaceRoot = resolve(root, "..");
const version = required(process.env.SDKWORK_PACKAGE_VERSION, "SDKWORK_PACKAGE_VERSION");
const sourceRevision = gitValue(["rev-parse", "HEAD"]);
const builderName = process.env.SDKWORK_CONTAINER_BUILDER ?? "sdkwork-memory-release";
const maxContextBytes = 256 * 1024 * 1024;
const maxContextFiles = 50_000;
const maxImageLayerBytes = 512 * 1024 * 1024;
const maxRuntimeBinaryBytes = 256 * 1024 * 1024;
const loadImage = process.env.SDKWORK_CONTAINER_LOAD_IMAGE === "true";
const embedSbom = process.env.SDKWORK_CONTAINER_SBOM_ATTESTATION !== "false";
if (!embedSbom && process.env.SDKWORK_RELEASE_VALIDATION === "strict") {
  throw new Error("strict release validation forbids disabling OCI SBOM attestation");
}
const artifactPath = resolve(
  root,
  process.env.SDKWORK_PACKAGE_ARTIFACT_PATH
    ?? "deployments/artifacts/release/sdkwork-memory-container-x64-cloud.oci.tar",
);
const stagedArtifactPath = `${artifactPath}.partial-${process.pid}`;
const imageReference = `registry.sdkwork.com/apps/sdkwork-memory:${version}`;

mkdirSync(dirname(artifactPath), { recursive: true });
rmSync(stagedArtifactPath, { force: true });
const contextRoot = mkdtempSync(join(tmpdir(), "sdkwork-memory-oci-"));

try {
  const context = materializeBuildContext(contextRoot);
  ensureAttestationBuilder(builderName);

  const commonBuildArguments = [
    "buildx",
    "build",
    "--builder",
    builderName,
    "--file",
    "sdkwork-memory/deployments/docker/Dockerfile",
    "--platform",
    "linux/amd64",
    "--build-arg",
    `SDKWORK_SOURCE_REVISION=${sourceRevision}`,
    "--build-arg",
    `SDKWORK_RELEASE_VERSION=${version}`,
    "--tag",
    imageReference,
  ];
  const buildArguments = [
    ...commonBuildArguments,
    "--provenance=mode=max",
    ...(embedSbom ? ["--sbom=true"] : []),
    "--output",
    `type=oci,dest=${stagedArtifactPath}`,
    contextRoot,
  ];

  execFileSync("docker", buildArguments, { cwd: workspaceRoot, stdio: "inherit" });

  const sizeBytes = verifyOciArchive(stagedArtifactPath);
  if (loadImage) {
    execFileSync("docker", [
      ...commonBuildArguments,
      "--provenance=false",
      "--load",
      contextRoot,
    ], { cwd: workspaceRoot, stdio: "inherit" });
  }
  rmSync(artifactPath, { force: true });
  renameSync(stagedArtifactPath, artifactPath);
  console.log(JSON.stringify({
    artifactPath,
    contextBytes: context.bytes,
    contextFiles: context.files,
    imageReference: loadImage ? imageReference : null,
    sizeBytes,
    sourceRevision,
    version,
  }, null, 2));
} finally {
  rmSync(stagedArtifactPath, { force: true });
  rmSync(contextRoot, { force: true, recursive: true, maxRetries: 3, retryDelay: 250 });
}

function required(value, label) {
  const text = String(value ?? "").trim();
  if (!text) throw new Error(`${label} is required`);
  return text;
}

function gitValue(args) {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
}

function materializeBuildContext(contextRoot) {
  let bytes = 0;
  let files = 0;
  const sources = [
    [resolve(root, "Cargo.toml"), "sdkwork-memory/Cargo.toml"],
    [resolve(root, "Cargo.lock"), "sdkwork-memory/Cargo.lock"],
    [resolve(root, "crates"), "sdkwork-memory/crates"],
    [resolve(root, "plugins"), "sdkwork-memory/plugins"],
    [resolve(root, "database"), "sdkwork-memory/database"],
    [resolve(workspaceRoot, "sdkwork-database/Cargo.toml"), "sdkwork-database/Cargo.toml"],
    [resolve(workspaceRoot, "sdkwork-database/crates"), "sdkwork-database/crates"],
    [resolve(workspaceRoot, "sdkwork-drive/Cargo.toml"), "sdkwork-drive/Cargo.toml"],
    [resolve(workspaceRoot, "sdkwork-drive/crates"), "sdkwork-drive/crates"],
    [resolve(workspaceRoot, "sdkwork-drive/database"), "sdkwork-drive/database"],
    [resolve(workspaceRoot, "sdkwork-iam/Cargo.toml"), "sdkwork-iam/Cargo.toml"],
    [resolve(workspaceRoot, "sdkwork-iam/crates"), "sdkwork-iam/crates"],
    [resolve(workspaceRoot, "sdkwork-iam/database"), "sdkwork-iam/database"],
    [resolve(workspaceRoot, "sdkwork-id/Cargo.toml"), "sdkwork-id/Cargo.toml"],
    [resolve(workspaceRoot, "sdkwork-id/crates"), "sdkwork-id/crates"],
    [resolve(workspaceRoot, "sdkwork-utils/Cargo.toml"), "sdkwork-utils/Cargo.toml"],
    [
      resolve(workspaceRoot, "sdkwork-utils/packages/sdkwork-utils-rust"),
      "sdkwork-utils/packages/sdkwork-utils-rust",
    ],
    [
      resolve(workspaceRoot, "sdkwork-web-framework/Cargo.toml"),
      "sdkwork-web-framework/Cargo.toml",
    ],
    [resolve(workspaceRoot, "sdkwork-web-framework/crates"), "sdkwork-web-framework/crates"],
    [
      resolve(workspaceRoot, "sdkwork-web-framework/tests/architecture"),
      "sdkwork-web-framework/tests/architecture",
    ],
  ];

  for (const [source, relativeDestination] of sources) {
    const destination = resolve(contextRoot, relativeDestination);
    mkdirSync(dirname(destination), { recursive: true });
    cpSync(source, destination, {
      errorOnExist: true,
      force: false,
      recursive: true,
      filter(candidate) {
        const metadata = lstatSync(candidate);
        if (metadata.isSymbolicLink()) {
          throw new Error(`OCI build context cannot contain symbolic links: ${candidate}`);
        }
        if (metadata.isDirectory() && isBuildOutputDirectory(candidate)) return false;
        if (metadata.isFile()) {
          files += 1;
          bytes += metadata.size;
          if (files > maxContextFiles || bytes > maxContextBytes) {
            throw new Error(
              `OCI build context exceeds its bound (${files} files, ${bytes} bytes)`,
            );
          }
        }
        return true;
      },
    });
  }
  return { bytes, files };
}

function isBuildOutputDirectory(path) {
  const name = basename(path).toLowerCase();
  return name === ".cache"
    || name === ".git"
    || name === "coverage"
    || name === "dist"
    || name === "node_modules"
    || name === "target"
    || name.startsWith("target-");
}

function verifyOciArchive(path) {
  const sizeBytes = statSync(path).size;
  if (sizeBytes === 0) throw new Error(`OCI archive is empty: ${path}`);

  execFileSync("tar", ["-tf", path, "oci-layout", "index.json"], {
    cwd: root,
    stdio: "ignore",
  });
  const index = readOciJson(path, "index.json");
  if (index.schemaVersion !== 2 || !Array.isArray(index.manifests) || index.manifests.length === 0) {
    throw new Error("OCI archive index contains no image manifests");
  }
  const manifest = resolveImageManifest(path, index.manifests);
  const contentDescriptors = [manifest.config, ...(manifest.layers ?? [])];
  if (contentDescriptors.length < 2) {
    throw new Error("OCI image manifest is missing its config or layers");
  }
  for (const descriptor of contentDescriptors) {
    const contentPath = descriptorPath(descriptor, "image content");
    execFileSync("tar", ["-tf", path, contentPath], { cwd: root, stdio: "ignore" });
  }
  verifyImageConfig(readOciJson(path, descriptorPath(manifest.config, "image config")));
  verifyRuntimeBinary(path, manifest.layers);
  return sizeBytes;
}

function verifyImageConfig(config) {
  if (config?.architecture !== "amd64" || config?.os !== "linux") {
    throw new Error(`OCI image platform must be linux/amd64, received ${config?.os}/${config?.architecture}`);
  }
  if (config?.config?.User !== "10001:10001") {
    throw new Error(`OCI image must run as 10001:10001, received ${config?.config?.User}`);
  }
  const command = config?.config?.Cmd;
  if (!Array.isArray(command) || command.at(-1) !== "sdkwork-api-memory-standalone-gateway") {
    throw new Error("OCI image command does not start the standalone gateway");
  }
}

function verifyRuntimeBinary(archivePath, layers) {
  if (!Array.isArray(layers) || layers.length === 0) {
    throw new Error("OCI image manifest contains no runtime layers");
  }
  const inspectionRoot = mkdtempSync(join(tmpdir(), "sdkwork-memory-oci-verify-"));
  const entryPath = "usr/local/bin/sdkwork-api-memory-standalone-gateway";
  try {
    for (let index = layers.length - 1; index >= 0; index -= 1) {
      const layer = layers[index];
      if (!Number.isSafeInteger(layer?.size) || layer.size <= 0 || layer.size > maxImageLayerBytes) {
        throw new Error(`OCI image layer size is outside the allowed bound: ${layer?.size}`);
      }
      const layerEntry = descriptorPath(layer, "runtime layer");
      execFileSync("tar", ["-xf", archivePath, "-C", inspectionRoot, layerEntry], {
        cwd: root,
        stdio: "ignore",
      });
      const layerPath = resolve(inspectionRoot, layerEntry);
      const entryRoot = resolve(inspectionRoot, `layer-${index}`);
      mkdirSync(entryRoot, { recursive: true });
      try {
        execFileSync("tar", ["-xf", layerPath, "-C", entryRoot, entryPath], {
          cwd: root,
          stdio: "ignore",
        });
      } catch {
        rmSync(entryRoot, { force: true, recursive: true });
        rmSync(layerPath, { force: true });
        continue;
      }
      const binaryPath = resolve(entryRoot, entryPath);
      const metadata = lstatSync(binaryPath);
      if (!metadata.isFile() || metadata.size < 4 || metadata.size > maxRuntimeBinaryBytes) {
        throw new Error(`OCI runtime binary size is outside the allowed bound: ${metadata.size}`);
      }
      const descriptor = openSync(binaryPath, "r");
      const magic = Buffer.alloc(4);
      try {
        if (readSync(descriptor, magic, 0, magic.length, 0) !== magic.length) {
          throw new Error("OCI runtime binary is truncated");
        }
      } finally {
        closeSync(descriptor);
      }
      if (!magic.equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46]))) {
        throw new Error("OCI runtime binary is not an ELF executable");
      }
      return;
    }
  } finally {
    rmSync(inspectionRoot, { force: true, recursive: true, maxRetries: 3, retryDelay: 250 });
  }
  throw new Error(`OCI image does not contain ${entryPath}`);
}

function resolveImageManifest(archivePath, descriptors, depth = 0) {
  if (depth > 4 || !Array.isArray(descriptors) || descriptors.length === 0) {
    throw new Error("OCI archive does not resolve to a bounded image manifest");
  }
  const descriptor = descriptors.find((candidate) =>
    candidate?.platform?.architecture === "amd64" && candidate?.platform?.os === "linux"
  ) ?? descriptors.find((candidate) =>
    candidate?.mediaType === "application/vnd.oci.image.manifest.v1+json"
  ) ?? descriptors[0];
  const document = readOciJson(archivePath, descriptorPath(descriptor, "image manifest"));
  if (document.schemaVersion !== 2) {
    throw new Error("OCI image descriptor has an unsupported schema version");
  }
  if (document.config && Array.isArray(document.layers)) return document;
  return resolveImageManifest(archivePath, document.manifests, depth + 1);
}

function readOciJson(archivePath, entryPath) {
  const content = execFileSync("tar", ["-xOf", archivePath, entryPath], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  return JSON.parse(content);
}

function descriptorPath(descriptor, label) {
  const digest = String(descriptor?.digest ?? "");
  if (!/^sha256:[a-f0-9]{64}$/u.test(digest)) {
    throw new Error(`OCI ${label} has an invalid digest: ${digest}`);
  }
  return `blobs/sha256/${digest.slice("sha256:".length)}`;
}

function ensureAttestationBuilder(name) {
  try {
    execFileSync("docker", ["buildx", "inspect", name], {
      cwd: root,
      stdio: "ignore",
    });
  } catch {
    execFileSync(
      "docker",
      ["buildx", "create", "--name", name, "--driver", "docker-container"],
      { cwd: root, stdio: "inherit" },
    );
  }
  execFileSync("docker", ["buildx", "inspect", name, "--bootstrap"], {
    cwd: root,
    stdio: "inherit",
  });
}

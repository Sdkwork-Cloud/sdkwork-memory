# SDKWork Memory PC

SDKWork Memory PC is the product application for customer memory management and internal memory operations. It is one React application root with two deliberately isolated surfaces.

## Surface Contract

| Route | Audience | SDK boundary | Permission namespace |
| --- | --- | --- | --- |
| `/console/*` | Customers, tenant owners, end users | `@sdkwork/memory-app-sdk` only | `memory.*` App permissions |
| `/admin/*` | Internal operations, support, security, audit | `@sdkwork/memory-backend-sdk` only | `memory.backend.*` and control-plane permissions |

The shared commons package owns rendering, server pagination, type-aware command inputs, guarded destructive actions, safe `ProblemDetail` output, and localization. It does not own a network client. Console/Admin core packages own their respective composed SDK client and resource registry.

## Package Ownership

| Package | Responsibility |
| --- | --- |
| `sdkwork-memory-pc-core` | Runtime config, host composition, auth/session wiring |
| `sdkwork-memory-pc-commons` | Shared workspace and resource controls |
| `sdkwork-memory-pc-console-core` | App SDK context and Console data/action registry |
| `sdkwork-memory-pc-console-*` | Console feature route, permission, resources, and i18n |
| `sdkwork-memory-pc-console-shell` | Console navigation shell |
| `sdkwork-memory-pc-admin-core` | Backend SDK context and Admin data/action registry |
| `sdkwork-memory-pc-admin-*` | Admin feature route, permission, resources, and i18n |
| `sdkwork-memory-pc-admin-shell` | Admin navigation shell |

Feature packages do not create raw HTTP clients, auth headers, or generated-transport deep imports.

## Runtime Configuration

`etc/browser/runtime-env.<environment>.example.json` is the public runtime template. `scripts/materialize-runtime-env.mjs` writes `public/runtime-env.json` before development/build. Credentials, tokens, and provider secrets are forbidden in these files.

Production builds reject loopback App, Backend, and Appbase API URLs. Console and Admin share the host origin but resolve distinct SDK Base URLs and session permissions.

## Commands

```powershell
pnpm --dir apps/sdkwork-memory-pc dev
pnpm --dir apps/sdkwork-memory-pc check
pnpm --dir apps/sdkwork-memory-pc build:browser:cloud
```

`build:browser:cloud` creates the deterministic browser ZIP under `deployments/artifacts/pc/`, generates SPDX SBOM and provenance evidence, computes SHA-256, and synchronizes the candidate evidence into the app manifest.

## Release State

The manifest remains `DRAFT` until the GitHub release workflow provides an OIDC artifact attestation, uploads the immutable ZIP to the reserved release URL, and records smoke/rollback evidence. A local checksum is real candidate evidence but is not a substitute for trusted signing or publication.

Operational procedures are in `docs/runbooks/RUNBOOK-memory-pc-operations.md`; release and rollback gates are in `docs/releases/README.md`.

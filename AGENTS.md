# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../sdkwork-specs/SOUL.md` before executing tasks in this root. Follow specs before memory, dictionary before context, stop on ambiguity, and evidence before completion.

## SDKWORK Standards

Canonical SDKWORK specs path from this root:

- `../sdkwork-specs/README.md`
- `../sdkwork-specs/SOUL.md`
- `../sdkwork-specs/AGENTS_SPEC.md`
- `../sdkwork-specs/PNPM_SCRIPT_SPEC.md`
- `../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`
- `../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../sdkwork-specs/NAMING_SPEC.md`

Do not copy root standard text into this repository. If these relative paths do not resolve, stop and report the broken workspace layout.

## Application Identity

Read `sdkwork.app.config.json` only when the task touches Memory application behavior, runtime config, SDK wiring, release metadata, app-owned capabilities, packaging, or deployment. For unrelated documentation or tooling work, do not expand into the full app manifest unless evidence requires it.

## Local Dictionary Structure

- `AGENTS.md`: repository agent entrypoint and relative SDKWork spec index.
- `CLAUDE.md`, `GEMINI.md`, `CODEX.md`: compatibility shims that point to `AGENTS.md` and must not duplicate rules.
- `sdkwork.app.config.json`: Memory application identity, runtime, release, and capability metadata.
- `sdkwork.workflow.json`: GitHub packaging/release workflow manifest governed by `GITHUB_WORKFLOW_SPEC.md`.
- `.github/workflows/package.yml`: thin reusable workflow call only.
- `.sdkwork/`: repository/application AI workspace metadata, local skills, local plugins, and manifests.
- `specs/`: local application/component contracts and narrowing rules.
- `apis/`: Memory-owned API contract sources and materialized OpenAPI inputs.
- `apps/`: reserved for future client application roots.
- `crates/`: reusable Rust service, repository, route, and API server crates.
- `sdks/`: SDK families, SDK generation manifests, route manifests, and generated SDK artifacts.
- `database/`: database contract, baseline DDL, migrations, seeds, and drift policy.
- `etc/`, `deployments/`, `scripts/`, `tools/`, `docs/`, `tests/`: source-owned config templates, deployment descriptors, thin command entrypoints, validators, documentation, and verification assets.
- `package.json`, `Cargo.toml`: language/build manifests.

## Spec Resolution Order

Use dynamic progressive loading:

1. Read this `AGENTS.md` and any nearer component-level `AGENTS.md`.
2. Read `sdkwork.app.config.json` only when app behavior, runtime config, SDK wiring, release, packaging, or app-owned capabilities are touched.
3. Read local `specs/README.md` and `specs/component.spec.json` only when the task touches that local contract.
4. Read local `.sdkwork/README.md`, `.sdkwork/skills/`, and `.sdkwork/plugins/` only when local agent extensions are relevant.
5. Read `../sdkwork-specs/README.md`, then only the task-specific root specs.
6. Inspect implementation files after the dictionary and relevant specs are clear.

Do not load the whole repository or every root spec before identifying the task surface.

## Required Specs By Task Type

- Agent/workflow changes: `../sdkwork-specs/SOUL.md`, `../sdkwork-specs/AGENTS_SPEC.md`, `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`, and `../sdkwork-specs/TEST_SPEC.md`.
- Package script changes: `../sdkwork-specs/PNPM_SCRIPT_SPEC.md`, `../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`, `../sdkwork-specs/CONFIG_SPEC.md`, and `../sdkwork-specs/TEST_SPEC.md`.
- Any code change: `../sdkwork-specs/CODE_STYLE_SPEC.md`, `../sdkwork-specs/NAMING_SPEC.md`, plus only the touched language/framework spec.
- Rust code: `../sdkwork-specs/RUST_CODE_SPEC.md`; add `../sdkwork-specs/RUST_RPC_SPEC.md` when RPC is touched.
- API/SDK changes: `../sdkwork-specs/API_SPEC.md`, `../sdkwork-specs/WEB_FRAMEWORK_SPEC.md`, `../sdkwork-specs/WEB_BACKEND_SPEC.md`, `../sdkwork-specs/SDK_SPEC.md`, `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`, and `../sdkwork-specs/TEST_SPEC.md`.
- Database changes: `../sdkwork-specs/DATABASE_SPEC.md`, `../sdkwork-specs/DATABASE_FRAMEWORK_SPEC.md`, `../sdkwork-specs/PRIVACY_SPEC.md`, and `../sdkwork-specs/TEST_SPEC.md`.
- Runtime/deployment/release changes: `../sdkwork-specs/CONFIG_SPEC.md`, `../sdkwork-specs/ENVIRONMENT_SPEC.md`, `../sdkwork-specs/DEPLOYMENT_SPEC.md`, `../sdkwork-specs/RELEASE_SPEC.md`, and `../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`.
- Provider/integration changes: `../sdkwork-specs/INTEGRATION_SPEC.md`, `../sdkwork-specs/SECURITY_SPEC.md`, and `../sdkwork-specs/PRIVACY_SPEC.md`.

Language-specific specs are on-demand; do not load Rust, Java, TypeScript, and frontend specs for unrelated tasks.

## Code Style Rules

Read `../sdkwork-specs/CODE_STYLE_SPEC.md` and `../sdkwork-specs/NAMING_SPEC.md` before code changes. Generated SDK output under `generated/server-openapi` must not be hand-edited. Fix OpenAPI, route manifests, generator input, or approved composed facades, then regenerate. Use `sdkwork-utils-rust` and `sdkwork-id-core` for shared helpers instead of duplicating utility logic locally.

## Build, Test, and Verification

Use canonical root package scripts from `PNPM_SCRIPT_SPEC.md`:

```powershell
pnpm verify
pnpm check
pnpm topology:validate
pnpm db:validate
```

## Agent Execution Rules

Do not rely on memory when a relevant SDKWork spec exists. Do not replace generated SDK calls with raw HTTP. Stop when the relative specs path, app identity, component spec, API authority, SDK family, table prefix, or provider ownership is ambiguous.

## List And Search Pagination

All L2+ list/search APIs and their backing services, repositories, SDK consumers, and interactive frontend lists `MUST` follow `PAGINATION_SPEC.md`:

- **Input:** standard `SdkWorkListQuery` or query params (`page`/`page_size` or `cursor`/`page_size` per `API_SPEC.md` §14.1); default `page_size` `20`; max `200` unless a documented exception exists.
- **Output:** `SdkWorkApiResponse.data.items` + `data.pageInfo` with `PageInfo.mode` (`offset` or `cursor`) per `API_SPEC.md` §16.
- **Store-level pagination:** push filtering, sorting, and page selection to SQL `LIMIT`/keyset or incrementally maintained indexes — never unbounded collect then `skip`/`take`/`slice` in process memory (`PAGINATION_SPEC.md` §2).
- **SDK and frontend:** interactive lists request one page at a time from the server; no default `listAll*` on P0/P1 paths; no client-side `slice` pagination over full downloads.

Before completing list/search API, repository, SDK list helper, projection read model, or paginated UI work, run:

```bash
node <sdkwork-specs>/tools/check-pagination.mjs --workspace <workspace-root>
```

Authority: `PAGINATION_SPEC.md`, `API_SPEC.md` §14.1/§16, `DATABASE_SPEC.md` §20.5, `WEB_BACKEND_SPEC.md` §12, `SDK_SPEC.md` §4.2/§6, `FRONTEND_SPEC.md`, `APP_SDK_INTEGRATION_SPEC.md` §9.

## App SDK Consumer Imports

Application, feature, shell, and service packages `MUST` consume HTTP SDKs through scoped composed consumer packages, not generator transport package names.

- App API clients: `@sdkwork/<application-code>-app-sdk`
- Backend API clients (`backend-admin` only): `@sdkwork/<application-code>-backend-sdk`
- Open/domain API clients: `@sdkwork/<domain>-sdk`

Forbidden in application code: generator transport package names, deep imports into `generated/server-openapi/src/*` from consumers when a composed facade exists.

Before completing SDK integration work, run:

```bash
node <sdkwork-specs>/tools/check-app-sdk-consumer-imports.mjs --workspace <workspace-root>
```

Authority: `APP_SDK_INTEGRATION_SPEC.md` section 9, `SDK_SPEC.md`, `SDK_WORKSPACE_GENERATION_SPEC.md`.

## HTTP API Response Envelope

All L2+ SDKWork-owned custom HTTP contracts, including `app-api`, `backend-api`, and SDKWork-owned business `open-api`, `MUST` follow `API_SPEC.md` sections 4.5 and 14-16:

- Omitted `x-sdkwork-wire-protocol` means the SDKWork `sdkwork-v3` protocol. Only operations that declare both `x-sdkwork-wire-protocol: external` and `x-sdkwork-external-protocol-id` may use a third-party compatibility wire format.
- Inputs use typed request bodies and the standard list/search/command shapes. Free-text search uses `q`.
- Success responses use `SdkWorkApiResponse` with numeric `code: 0`, typed `data`, and server-generated `traceId`.
- Errors use HTTP 4xx/5xx `application/problem+json` with numeric non-zero `code` and `traceId`.
- Single-resource data uses `data.item`; lists use `data.items` and `data.pageInfo`; commands use `data.accepted`; async accepts use `data.operationId` and `data.status`.
- Create uses HTTP 201, delete uses HTTP 204 with no JSON body, and PUT/PATCH use SDK action `update`.

SDKWork-owned business APIs must not opt out of the standard envelope. Business failures must not use HTTP 2xx with a non-zero code, a string wire code, `success`, or a human `message` field. Generated SDKs unwrap `data` by default; consumers use `.raw` only when the full envelope is required.

Before completing API contract, SDK generation, or frontend service work, run:

```bash
node <sdkwork-specs>/tools/check-api-operation-patterns.mjs --workspace <workspace-root>
node <sdkwork-specs>/tools/check-api-response-envelope.mjs --workspace <workspace-root>
```

Authority: `API_SPEC.md` sections 4.5 and 14-16, `SDK_SPEC.md` section 4.2, `FRONTEND_SPEC.md`, and `MIGRATION_SPEC.md` section 4.2.

## Human Review Rules

Human review is required for breaking public API changes, schema migrations, privacy/security exceptions, generated SDK ownership changes, provider lock-in decisions, and destructive filesystem or data operations.

# Repository Guidelines

## SDKWORK Soul

Read `../sdkwork-specs/SOUL.md` before executing repository tasks. It defines the SDKWork rules for specs-before-memory, dictionary-before-context, stop-on-ambiguity, and evidence-before-completion.

## SDKWORK Standards

The canonical standards entrypoint is `../sdkwork-specs/README.md`. This repository must reference root standards by relative path and must not copy standard text into local specs.

## Application Identity

Read `sdkwork.app.config.json`, `specs/README.md`, and `specs/component.spec.json` before changing Memory behavior, runtime config, SDK wiring, database schema, API contracts, provider abstractions, release metadata, or app-owned capabilities.

## Local Dictionary Structure

- `AGENTS.md`: repository agent execution rules.
- `CODEX.md`, `CLAUDE.md`, `GEMINI.md`: tool compatibility shims that point back to `AGENTS.md`.
- `sdkwork.app.config.json`: SDKWork Memory app/service identity.
- `.sdkwork/`: local skills, plugins, and workspace metadata.
- `specs/`: local component contract entrypoint.
- `docs/superpowers/specs/`: product and architecture design specs.
- `docs/schema-registry/tables/`: Memory table contracts using the `mem_` prefix.
- `sdks/`: SDK families, OpenAPI authority files, SDK assembly manifests, and generated SDK outputs when materialized.
- `tools/`: materialization and verification commands.

## Spec Resolution Order

1. Read this `AGENTS.md`.
2. Read `sdkwork.app.config.json` when present.
3. Read `specs/README.md` and `specs/component.spec.json`.
4. Read local `.sdkwork/README.md` and relevant local skills/plugins when present.
5. Resolve `../sdkwork-specs/README.md`.
6. Read task-specific root specs.
7. Inspect implementation files.

## Required Specs By Task Type

- Agent/workflow changes: `../sdkwork-specs/SOUL.md`, `../sdkwork-specs/AGENTS_SPEC.md`, `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.
- Code changes: `../sdkwork-specs/CODE_STYLE_SPEC.md`, `../sdkwork-specs/NAMING_SPEC.md`, and only the touched language/framework spec.
- API changes: `../sdkwork-specs/API_SPEC.md`, `../sdkwork-specs/WEB_BACKEND_SPEC.md`, `../sdkwork-specs/SDK_SPEC.md`, `../sdkwork-specs/TEST_SPEC.md`.
- SDK generation or consumption: `../sdkwork-specs/SDK_SPEC.md`, `../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`, `../sdkwork-specs/API_SPEC.md`, `../sdkwork-specs/TEST_SPEC.md`.
- Database and storage changes: `../sdkwork-specs/DATABASE_SPEC.md`, `../sdkwork-specs/PRIVACY_SPEC.md`, `../sdkwork-specs/TEST_SPEC.md`.
- Events, jobs, and outbox changes: `../sdkwork-specs/EVENT_SPEC.md`, `../sdkwork-specs/OBSERVABILITY_SPEC.md`.
- Provider/integration changes: `../sdkwork-specs/INTEGRATION_SPEC.md`, `../sdkwork-specs/SECURITY_SPEC.md`, `../sdkwork-specs/PRIVACY_SPEC.md`.
- App identity/release changes: `../sdkwork-specs/APP_MANIFEST_SPEC.md`, `../sdkwork-specs/CONFIG_SPEC.md`, `../sdkwork-specs/DEPLOYMENT_SPEC.md`.

## Code Style Rules

Use the repository's existing patterns first. Generated SDK output under `generated/server-openapi` must not be hand-edited. Fix OpenAPI, route manifests, generator input, or approved composed facades, then regenerate.

## Build, Test, and Verification

Run commands from this repository root. Phase 1 contract verification:

```powershell
node tools/materialize_phase1_contracts.mjs
powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1
```

## Agent Execution Rules

Do not rely on memory when a relevant SDKWork spec exists. Do not replace generated SDK calls with raw HTTP. Stop when the relative specs path, app identity, component spec, API authority, SDK family, table prefix, or provider ownership is ambiguous.

## Human Review Rules

Human review is required for breaking public API changes, schema migrations, privacy/security exceptions, generated SDK ownership changes, provider lock-in decisions, and destructive filesystem or data operations.

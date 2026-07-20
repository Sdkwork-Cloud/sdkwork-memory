# SDKWork Memory PC Application

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing tasks in this root.

## SDKWORK Standards

- `../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`
- `../../../sdkwork-specs/APP_PC_REACT_UI_SPEC.md`
- `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`
- `../../../sdkwork-specs/BACKEND_UI_SPEC.md`
- `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md`
- `../../../sdkwork-specs/I18N_SPEC.md`
- `../../../sdkwork-specs/CONFIG_SPEC.md`
- `../../../sdkwork-specs/SOURCE_CONFIG_SPEC.md`
- `../../../sdkwork-specs/TEST_SPEC.md`

Do not copy global spec bodies into this application root. If these relative paths do not resolve, stop and report the broken workspace layout.

## Application Identity

- Application: `sdkwork-memory-pc`
- Application code: `memory`
- Architecture: `pc-react`
- Surfaces: `app-console`, `backend-admin`

## Surface Boundary

- `pc-console-*` packages consume `@sdkwork/memory-app-sdk` only through `pc-console-core`.
- `pc-admin-*` packages consume `@sdkwork/memory-backend-sdk` only through `pc-admin-core`.
- Console and admin packages must not import each other's business packages or SDK exports.
- Generated SDK output under `generated/server-openapi` must not be hand-edited.

## Verification

```powershell
pnpm --dir apps/sdkwork-memory-pc check
node ../sdkwork-specs/tools/check-app-sdk-consumer-imports.mjs --workspace .
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
```

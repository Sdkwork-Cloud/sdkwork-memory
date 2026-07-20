# SDKWork Memory PC

SDKWork Memory PC provides the user-facing Memory Console and the internal `backend-admin` operations surface from one PC React application root.

## Surfaces

| Route | Audience | SDK authority |
| --- | --- | --- |
| `/console/*` | Customers, tenant owners, and users managing their own memory | `@sdkwork/memory-app-sdk` |
| `/admin/*` | Internal operators, support, security, and auditors | `@sdkwork/memory-backend-sdk` |

## Commands

```powershell
pnpm --dir apps/sdkwork-memory-pc dev
pnpm --dir apps/sdkwork-memory-pc check
```

Runtime URL templates live in `config/browser/`. They contain public endpoints only and must never contain credentials.

# Docker Deployment

Build and run the SDKWork Memory API server container from the repository root:

```powershell
docker build -f deployments/docker/Dockerfile -t sdkwork-memory:local .
docker run --rm -p 8080:8080 `
  -e SDKWORK_MEMORY_ENVIRONMENT=development `
  -e SDKWORK_MEMORY_DEV_AUTH_BYPASS=true `
  -e SDKWORK_MEMORY_DATABASE_URL=sqlite::memory: `
  sdkwork-memory:local
```

The image exposes `SDKWORK_MEMORY_APPLICATION_PUBLIC_INGRESS_BIND` on `0.0.0.0:8080`, ships `/app/database` lifecycle assets, and defaults `SDKWORK_MEMORY_ENVIRONMENT=production` when no overrides are supplied.

For local development without Docker, use `pnpm dev`, which loads `configs/topology/standalone.unified-process.development.env` through `scripts/memory-dev.mjs`.

# Docker Deployment

Build and run the SDKWork Memory API server container from the repository root:

```powershell
docker build -f deployments/docker/Dockerfile -t sdkwork-memory:local .
docker run --rm -p 8080:8080 `
  -e SDKWORK_MEMORY_DATABASE_URL=sqlite::memory: `
  sdkwork-memory:local
```

The image exposes `SDKWORK_MEMORY_APPLICATION_PUBLIC_INGRESS_BIND` on `0.0.0.0:8080` and ships the release binary built from `sdkwork-memory-api-server`.

For local development without Docker, use `pnpm dev`, which loads `configs/topology/standalone.unified-process.development.env` through `scripts/memory-dev.mjs`.

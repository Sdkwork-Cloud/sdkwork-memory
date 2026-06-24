# Kubernetes deployment

Owner: sdkwork-memory

Unified-process Memory API server manifests for cloud-hosted deployment.

## Files

- `deployment.yaml` — `sdkwork-memory-api-server` Deployment (`deploymentProfile=cloud`, 2 replicas, pod anti-affinity, startup/readiness/liveness probes, graceful shutdown, securityContext)
- `migration-job.yaml` — one-shot database migration Job (`db-migrate` subcommand, `SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE=true`)
- `service.yaml` — ClusterIP service exposing port 8080 (Prometheus scrape annotations on `/metrics`)
- `servicemonitor.yaml` — Prometheus Operator scrape config for `/metrics` (optional when operator is installed)
- `prometheus-rules.yaml` — Prometheus Operator alert rules (health, authz, quota, outbox)
- `hpa.yaml` — CPU autoscaler (min 2, max 6)
- `pdb.yaml` — Pod disruption budget (`minAvailable: 1`)
- `ingress.yaml` — Public ingress for `/apps/sdkwork-memory`

## Prerequisites

- Container image built from `deployments/docker/Dockerfile` (ships `/app/database` lifecycle assets)
- Secret `sdkwork-memory-database` with key `database-url` for Memory PostgreSQL runtime
- Secret `sdkwork-memory-iam-database` with key `database-url` for IAM PostgreSQL auth resolution

## Apply

```bash
kubectl apply -f deployments/kubernetes/migration-job.yaml
kubectl wait --for=condition=complete job/sdkwork-memory-db-migrate --timeout=300s
kubectl apply -f deployments/kubernetes/
```

Runtime pods set `SDKWORK_MEMORY_DATABASE_AUTO_MIGRATE=false`; run the migration Job before rolling out new schema versions.

When an OpenTelemetry collector is available, set `OTEL_EXPORTER_OTLP_ENDPOINT` on the Deployment (release image is built with the `otel` feature). Apply `prometheus-rules.yaml` when Prometheus Operator is installed.

`/readyz` checks Memory database connectivity and IAM PostgreSQL when `SDKWORK_MEMORY_ENVIRONMENT=production`. Outbox publishing uses `FOR UPDATE SKIP LOCKED` on PostgreSQL so multiple replicas do not double-publish events.

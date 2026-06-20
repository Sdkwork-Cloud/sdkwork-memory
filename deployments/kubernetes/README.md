# Kubernetes deployment

Owner: sdkwork-memory

Unified-process Memory API server manifests for cloud-hosted deployment.

## Files

- `deployment.yaml` — `sdkwork-memory-api-server` Deployment with health probes on `/mem/v3/api/capabilities`
- `service.yaml` — ClusterIP service exposing port 8080

## Prerequisites

- Container image built from `deployments/docker/Dockerfile`
- Secret `sdkwork-memory-database` with key `database-url` when using external database

## Apply

```bash
kubectl apply -f deployments/kubernetes/
```

## Notes

Memory Phase 1 runs open, app, and backend API surfaces in a single unified process. Split-service manifests are deferred until topology profile `cloud.split-services` is materialized.

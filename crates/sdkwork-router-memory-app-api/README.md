# sdkwork-router-memory-app-api

Domain: intelligence
Capability: memory
Package type: rust-route-crate
Surface: app-api

This crate owns the SDKWork Memory app-api route adapter for `/app/v3/api`.

## Responsibilities

- Mount authenticated Memory app-api routes.
- Expose deterministic route manifest metadata and `HttpRouteManifest` (`RouteAuth::DualToken`).
- Wire `sdkwork-web-framework` through `IamDatabaseWebRequestContextResolver` with route-manifest auth enforcement.
- Decode HTTP requests, consume typed IAM context, call injected service traits, and map responses to API contracts.

## Boundaries

- Does not own business rules, SQLx queries, or provider clients.
- Does not expose open-api or backend-api routes.
- Does not import generated SDK output for the same authority.

## Verification

- `cargo test -p sdkwork-router-memory-app-api`
- `pnpm check:architecture-alignment`

# sdkwork-routes-memory-backend-api

Domain: intelligence
Capability: memory
Package type: rust-route-crate
Surface: backend-api

This crate owns the SDKWork Memory backend-api route adapter for `/backend/v3/api`.

## Responsibilities

- Mount authenticated Memory backend-api routes.
- Expose deterministic route manifest metadata and `HttpRouteManifest` (`RouteAuth::DualToken`).
- Wire `sdkwork-web-framework` through `IamWebRequestContextResolver` with route-manifest auth enforcement.
- Decode HTTP requests, consume typed IAM context, call injected service traits, and map responses to API contracts.

## Boundaries

- Does not own business rules, SQLx queries, or provider clients.
- Does not expose open-api or app-api routes.
- Does not import generated SDK output for the same authority.

## Verification

- `cargo test -p sdkwork-routes-memory-backend-api`
- `pnpm check:architecture-alignment`

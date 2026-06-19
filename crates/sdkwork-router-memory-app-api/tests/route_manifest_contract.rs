use sdkwork_router_memory_app_api::{app_route_manifest, memory_app_api_public_path_prefixes};

#[test]
fn app_route_manifest_resolves_contract_routes() {
    let manifest = app_route_manifest();
    let route = manifest
        .match_route("GET", "/app/v3/api/memory/spaces")
        .expect("spaces list route");
    assert_eq!(route.operation_id, "spaces.list");
    manifest
        .validate_public_path_prefixes(&memory_app_api_public_path_prefixes())
        .expect("public prefixes must not cover protected routes");
}

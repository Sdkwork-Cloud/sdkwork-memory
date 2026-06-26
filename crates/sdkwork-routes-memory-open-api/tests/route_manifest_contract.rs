use sdkwork_routes_memory_open_api::{memory_open_api_public_path_prefixes, open_route_manifest};

#[test]
fn open_route_manifest_resolves_contract_routes() {
    let manifest = open_route_manifest();
    let route = manifest
        .match_route("GET", "/mem/v3/api/memory/capabilities")
        .expect("capabilities route");
    assert_eq!(route.operation_id, "capabilities.retrieve");
    manifest
        .validate_public_path_prefixes(&memory_open_api_public_path_prefixes())
        .expect("public prefixes must not cover protected routes");
}

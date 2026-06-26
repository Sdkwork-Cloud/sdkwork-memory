use sdkwork_routes_memory_backend_api::{
    backend_route_manifest, memory_backend_api_public_path_prefixes,
};

#[test]
fn backend_route_manifest_resolves_contract_routes() {
    let manifest = backend_route_manifest();
    let route = manifest
        .match_route("GET", "/backend/v3/api/memory/spaces")
        .expect("spaces list route");
    assert_eq!(route.operation_id, "spaces.list");
    manifest
        .validate_public_path_prefixes(&memory_backend_api_public_path_prefixes())
        .expect("public prefixes must not cover protected routes");
}

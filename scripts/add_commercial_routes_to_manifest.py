#!/usr/bin/env python3
"""Add commercial management routes to the backend route manifest."""
import json

MANIFEST_PATH = "sdks/_route-manifests/backend-api/sdkwork-routes-memory-backend-api.route-manifest.json"

with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
    manifest = json.load(f)

existing_paths = {(r["method"], r["path"]) for r in manifest["routes"]}

def make_route(method, path, operation_id, rate_limit_tier=None):
    key = (method, path)
    if key in existing_paths:
        return None
    route = {
        "method": method,
        "path": path,
        "operationId": operation_id,
        "tags": ["memory"],
        "auth": {"mode": "dual-token", "required": True},
        "handler": {"module": "crate::commercial_routes", "name": None},
        "ownership": {
            "owner": "sdkwork-memory",
            "apiAuthority": "sdkwork-memory.backend",
        },
        "requestContext": "WebRequestContext",
        "apiSurface": "backend-api",
    }
    if rate_limit_tier:
        route["rateLimitTier"] = rate_limit_tier
    return route

P = "/backend/v3/api/memory"
new_routes = [
    # Subjects
    make_route("GET", f"{P}/subjects", "subjects.list"),
    make_route("POST", f"{P}/subjects", "subjects.create"),
    make_route("GET", f"{P}/subjects/{{subjectId}}", "subjects.retrieve"),
    make_route("PATCH", f"{P}/subjects/{{subjectId}}", "subjects.update"),
    make_route("DELETE", f"{P}/subjects/{{subjectId}}", "subjects.delete"),
    make_route("GET", f"{P}/subjects/{{subjectId}}/effective_capabilities", "subjects.effectiveCapabilities.retrieve"),
    make_route("GET", f"{P}/subjects/{{subjectId}}/effective_policies", "subjects.effectivePolicies.retrieve"),
    # Bindings
    make_route("GET", f"{P}/bindings", "bindings.list"),
    make_route("POST", f"{P}/bindings", "bindings.create"),
    make_route("GET", f"{P}/bindings/{{bindingId}}", "bindings.retrieve"),
    make_route("PATCH", f"{P}/bindings/{{bindingId}}", "bindings.update"),
    make_route("DELETE", f"{P}/bindings/{{bindingId}}", "bindings.delete"),
    make_route("POST", f"{P}/bindings/bulk_delete", "bindings.bulkDelete"),
    make_route("POST", f"{P}/bindings/bulk_upsert", "bindings.bulkUpsert"),
    make_route("POST", f"{P}/bindings/resolve", "bindings.resolve"),
    # Capability bindings
    make_route("GET", f"{P}/capability_bindings", "capabilityBindings.list"),
    make_route("POST", f"{P}/capability_bindings", "capabilityBindings.create"),
    make_route("GET", f"{P}/capability_bindings/{{capabilityBindingId}}", "capabilityBindings.retrieve"),
    make_route("PATCH", f"{P}/capability_bindings/{{capabilityBindingId}}", "capabilityBindings.update"),
    make_route("DELETE", f"{P}/capability_bindings/{{capabilityBindingId}}", "capabilityBindings.delete"),
    # Capabilities resolve
    make_route("POST", f"{P}/capabilities/resolve", "capabilities.resolve"),
    # Commercial readiness
    make_route("GET", f"{P}/commercial_readiness", "commercialReadiness.retrieve"),
    make_route("POST", f"{P}/commercial_readiness/rebuild", "commercialReadiness.rebuild", "authCritical"),
    # Entities
    make_route("GET", f"{P}/entities", "entities.list"),
    make_route("POST", f"{P}/entities", "entities.create"),
    make_route("GET", f"{P}/entities/{{entityId}}", "entities.retrieve"),
    make_route("PATCH", f"{P}/entities/{{entityId}}", "entities.update"),
    make_route("POST", f"{P}/entities/{{entityId}}/merge", "entities.merge"),
    # Edges
    make_route("GET", f"{P}/edges", "edges.list"),
    make_route("POST", f"{P}/edges", "edges.create"),
    make_route("GET", f"{P}/edges/{{edgeId}}", "edges.retrieve"),
    make_route("PATCH", f"{P}/edges/{{edgeId}}", "edges.update"),
    make_route("DELETE", f"{P}/edges/{{edgeId}}", "edges.delete"),
    # Policy assignments
    make_route("GET", f"{P}/policy_assignments", "policyAssignments.list"),
    make_route("POST", f"{P}/policy_assignments", "policyAssignments.create"),
    make_route("GET", f"{P}/policy_assignments/{{policyAssignmentId}}", "policyAssignments.retrieve"),
    make_route("PATCH", f"{P}/policy_assignments/{{policyAssignmentId}}", "policyAssignments.update"),
    make_route("DELETE", f"{P}/policy_assignments/{{policyAssignmentId}}", "policyAssignments.delete"),
    # Relation rebuild jobs
    make_route("POST", f"{P}/relation_rebuild_jobs", "relationRebuildJobs.create", "authCritical"),
    make_route("GET", f"{P}/relation_rebuild_jobs/{{jobId}}", "relationRebuildJobs.retrieve"),
]

added = 0
for route in new_routes:
    if route is not None:
        manifest["routes"].append(route)
        added += 1

with open(MANIFEST_PATH, "w", encoding="utf-8") as f:
    json.dump(manifest, f, indent=2, ensure_ascii=False)
    f.write("\n")

print(f"Added {added} commercial routes to manifest (total: {len(manifest['routes'])})")

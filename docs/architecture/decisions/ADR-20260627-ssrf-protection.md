# ADR-20260627: SSRF Protection for Provider Health Probes

- Status: Accepted
- Date: 2026-06-27
- Deciders: Memory Platform Team

## Context

The `probe_binding_endpoint` function in `job_worker.rs` made HTTP GET requests to arbitrary URLs stored in provider binding records. There was no URL validation, allowing Server-Side Request Forgery (SSRF) attacks where a malicious operator could configure a binding with an internal URL (e.g., `http://169.254.169.254/latest/meta-data/`) to access cloud metadata services or internal network resources.

## Decision

Add a `validate_endpoint_url` function that checks:

1. **Scheme**: Only `http` and `https` are allowed. `https` is required in production-like environments.
2. **IP literals**: Reject private (RFC 1918), loopback, unspecified, and link-local IP addresses.
3. **Hostnames**: Reject `localhost` and `.localhost` suffixes.
4. **Cloud metadata endpoints**: Explicitly block `169.254.169.254`, `metadata.google.internal`, and `metadata.aws.internal`.
5. **Redirect policy**: Set `reqwest::redirect::Policy::none()` to prevent redirects to internal resources.

## Consequences

- **Positive**: Eliminates the SSRF attack surface for provider health probes.
- **Positive**: HTTPS enforcement in production ensures probe traffic is encrypted.
- **Positive**: Redirect blocking prevents bypass via redirect chains.
- **Negative**: Hostname-based validation (e.g., `internal.corp`) cannot be blocked without DNS resolution, which introduces latency and complexity.
- **Mitigation**: The allowlist approach (only allowing pre-approved endpoint hosts) should be considered for high-security deployments.

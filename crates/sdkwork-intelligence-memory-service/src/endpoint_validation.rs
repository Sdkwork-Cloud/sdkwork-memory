//! Outbound URL validation for webhook and provider integrations.
//!
//! Prevents SSRF against private networks, loopback, and cloud metadata endpoints.

use crate::platform;

/// Validates an outbound HTTP(S) URL before the service posts to it.
pub fn validate_outbound_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|error| format!("invalid URL: {error}"))?;

    match parsed.scheme() {
        "https" => {}
        "http" => {
            if platform::is_production_like_environment() {
                return Err("HTTP scheme is not allowed in production; use HTTPS".to_string());
            }
        }
        other => return Err(format!("unsupported URL scheme: {other}")),
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL must have a host".to_string())?;

    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if ip.is_loopback() || ip.is_unspecified() || is_private_ip(&ip) || is_link_local_ip(&ip) {
            return Err(format!(
                "private or loopback IP addresses are not allowed: {host}"
            ));
        }
    }

    let host_lower = host.to_ascii_lowercase();
    if host_lower == "localhost" || host_lower.ends_with(".localhost") {
        return Err(format!("localhost hostnames are not allowed: {host}"));
    }

    if host_lower == "metadata.google.internal"
        || host_lower == "169.254.169.254"
        || host_lower == "metadata.aws.internal"
    {
        return Err(format!("cloud metadata endpoints are not allowed: {host}"));
    }

    Ok(())
}

fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_private() || v4.is_link_local(),
        std::net::IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

fn is_link_local_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_link_local(),
        std::net::IpAddr::V6(v6) => {
            let segments = v6.segments();
            segments[0] == 0xfe80
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_localhost_hostname() {
        assert!(validate_outbound_url("https://localhost/hook").is_err());
    }

    #[test]
    fn rejects_private_ip_literal() {
        assert!(validate_outbound_url("https://10.0.0.1/hook").is_err());
    }

    #[test]
    fn accepts_public_https_endpoint() {
        assert!(validate_outbound_url("https://hooks.example.com/memory").is_ok());
    }
}

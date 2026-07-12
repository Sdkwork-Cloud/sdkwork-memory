//! Reject memory payloads that resemble secrets or credentials (PRIVACY_SPEC / SECURITY_SPEC).

use regex::Regex;
use sdkwork_utils_rust::is_blank;
use std::sync::OnceLock;

use sdkwork_memory_contract::MemoryServiceError;

fn sensitive_patterns() -> &'static [Regex] {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            // Generic API key / secret key / token assignments.
            r"(?i)\b(api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token)\s*[:=]\s*\S{8,}",
            // Password assignments.
            r"(?i)\b(password|passwd|pwd)\s*[:=]\s*\S+",
            // Bearer tokens.
            r"(?i)\b(bearer\s+[a-z0-9\-_\.~\+/]+=*)",
            // OpenAI / Slack platform tokens.
            r"(?i)\b(sk-[a-z0-9]{20,}|xox[baprs]-[a-z0-9-]{10,})",
            // PEM private key blocks.
            r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
            // AWS key assignments.
            r"(?i)\baws[_-]?(?:secret|access)[_-]?key\s*[:=]\s*\S+",
            // JWT tokens (eyJ header prefix).
            r"\beyJ[a-zA-Z0-9_-]{10,}\.eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}\b",
            // GitHub personal access / OAuth / app tokens.
            r"\b(ghp_[a-zA-Z0-9]{36}|gho_[a-zA-Z0-9]{36}|ghu_[a-zA-Z0-9]{36}|ghs_[a-zA-Z0-9]{36}|ghr_[a-zA-Z0-9]{76})\b",
            // Google API key prefix.
            r"\bAIza[a-zA-Z0-9_-]{35}\b",
            // Connection strings with embedded credentials.
            r"(?i)\b(?:postgres|mysql|mongodb|redis|amqp)://[^:\s]+:[^@\s]+@",
            // Azure storage account keys.
            r"(?i)\b(account[_-]?key|storage[_-]?key)\s*[:=]\s*[A-Za-z0-9+/=]{60,}",
        ]
        .into_iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
    })
}

pub fn assert_memory_text_is_safe(fields: &[(&str, &str)]) -> Result<(), MemoryServiceError> {
    for (label, value) in fields {
        if is_blank(Some(*value)) {
            continue;
        }
        for pattern in sensitive_patterns() {
            if pattern.is_match(value) {
                return Err(MemoryServiceError::validation(format!(
                    "memory {label} rejected: sensitive credential-like content is not allowed"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_api_key_like_content() {
        let err = assert_memory_text_is_safe(&[(
            "canonicalText",
            "user api_key=sk-live-abcdefghijklmnopqrstuvwxyz",
        )])
        .unwrap_err();
        assert_eq!(err.code, "validation_error");
    }

    #[test]
    fn allows_normal_memory_text() {
        assert_memory_text_is_safe(&[("canonicalText", "User prefers dark mode in the IDE.")])
            .expect("normal text should be accepted");
    }

    #[test]
    fn rejects_jwt_token() {
        let err = assert_memory_text_is_safe(&[(
            "canonicalText",
            "token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
        )])
        .unwrap_err();
        assert_eq!(err.code, "validation_error");
    }

    #[test]
    fn rejects_github_token() {
        // GitHub personal access token: ghp_ prefix followed by exactly 36 alphanumeric chars.
        let err = assert_memory_text_is_safe(&[(
            "canonicalText",
            "ghp_0123456789abcdef0123456789abcdef0123",
        )])
        .unwrap_err();
        assert_eq!(err.code, "validation_error");
    }

    #[test]
    fn rejects_connection_string_with_credentials() {
        let err = assert_memory_text_is_safe(&[(
            "canonicalText",
            "postgres://admin:secretpass@db.example.com:5432/mydb",
        )])
        .unwrap_err();
        assert_eq!(err.code, "validation_error");
    }

    #[test]
    fn allows_email_and_phone_in_memory() {
        // PII like email and phone are legitimate memory content;
        // only credential/secret patterns should be rejected.
        assert_memory_text_is_safe(&[(
            "canonicalText",
            "User contact email is user@example.com, phone is 13800138000.",
        )])
        .expect("email and phone should be accepted");
    }
}

//! Memory runtime environment helpers shared by routers and the API server.

use sdkwork_utils_rust::parse_bool;

static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Serializes env-mutating tests across crates that share process environment state.
#[doc(hidden)]
pub fn env_test_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Canonical `SDKWORK_MEMORY_ENVIRONMENT` / `SDKWORK_MEMORY_CONFIG_PROFILE` value.
pub fn memory_environment_name() -> String {
    std::env::var("SDKWORK_MEMORY_ENVIRONMENT")
        .or_else(|_| std::env::var("SDKWORK_MEMORY_CONFIG_PROFILE"))
        .unwrap_or_else(|_| "development".to_string())
        .to_ascii_lowercase()
}

/// Returns true for production-like lifecycle profiles that must never use dev inline auth.
pub fn memory_is_production_like_environment() -> bool {
    matches!(
        memory_environment_name().as_str(),
        "production" | "prod" | "staging" | "stage" | "test"
    )
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| parse_bool(&value))
        .unwrap_or(false)
}

/// `SDKWORK_MEMORY_DEV_AUTH_BYPASS` enables inline dev credentials only in non-production profiles.
pub fn memory_dev_auth_bypass_enabled() -> bool {
    env_truthy("SDKWORK_MEMORY_DEV_AUTH_BYPASS")
}

/// Whether HTTP surfaces may use `DefaultWebRequestContextResolver` with inline dev credentials.
pub fn memory_use_dev_inline_auth_resolver() -> bool {
    !memory_is_production_like_environment() && memory_dev_auth_bypass_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_env(key: &str, value: Option<&str>, test: impl FnOnce()) {
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        test();
        match previous {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn production_never_uses_dev_inline_auth_even_when_bypass_flag_is_set() {
        let _guard = env_test_lock();
        with_env("SDKWORK_MEMORY_ENVIRONMENT", Some("production"), || {
            with_env("SDKWORK_MEMORY_DEV_AUTH_BYPASS", Some("true"), || {
                assert!(!memory_use_dev_inline_auth_resolver());
            });
        });
    }

    #[test]
    fn development_requires_explicit_dev_auth_bypass_flag() {
        let _guard = env_test_lock();
        with_env("SDKWORK_MEMORY_ENVIRONMENT", Some("development"), || {
            with_env("SDKWORK_MEMORY_DEV_AUTH_BYPASS", None, || {
                assert!(!memory_use_dev_inline_auth_resolver());
            });
            with_env("SDKWORK_MEMORY_DEV_AUTH_BYPASS", Some("true"), || {
                assert!(memory_use_dev_inline_auth_resolver());
            });
        });
    }

    #[test]
    fn dev_auth_bypass_rejects_non_boolean_env_values() {
        let _guard = env_test_lock();
        with_env("SDKWORK_MEMORY_ENVIRONMENT", Some("development"), || {
            with_env("SDKWORK_MEMORY_DEV_AUTH_BYPASS", Some("maybe"), || {
                assert!(!memory_dev_auth_bypass_enabled());
            });
            with_env("SDKWORK_MEMORY_DEV_AUTH_BYPASS", Some("false"), || {
                assert!(!memory_dev_auth_bypass_enabled());
            });
        });
    }
}

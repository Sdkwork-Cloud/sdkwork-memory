//! IAM principal identifier parsing shared across Memory HTTP surfaces.

/// Parse a numeric tenant, user, or organization id from IAM principal wire values.
pub fn parse_principal_u64(value: &str) -> Option<u64> {
    sdkwork_utils_rust::parse_int(value).and_then(|parsed| u64::try_from(parsed).ok())
}

/// Parse an optional organization id when the principal carries one.
pub fn parse_principal_optional_u64(value: Option<&str>) -> Option<u64> {
    value.and_then(parse_principal_u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_principal_u64_accepts_numeric_wire_values() {
        assert_eq!(parse_principal_u64("100001"), Some(100001));
        assert_eq!(parse_principal_u64(" 2001 "), Some(2001));
        assert_eq!(parse_principal_u64("not-a-number"), None);
    }

    #[test]
    fn parse_principal_optional_u64_skips_blank_values() {
        assert_eq!(parse_principal_optional_u64(Some("42")), Some(42));
        assert_eq!(parse_principal_optional_u64(Some("   ")), None);
        assert_eq!(parse_principal_optional_u64(None), None);
    }
}

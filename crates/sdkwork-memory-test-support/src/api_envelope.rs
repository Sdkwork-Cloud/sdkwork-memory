//! Helpers for asserting SdkWorkApiResponse envelopes in HTTP integration tests.

use serde_json::Value;

pub fn item(envelope: &Value) -> &Value {
    envelope
        .pointer("/data/item")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.item, got {envelope}"))
}

pub fn items(envelope: &Value) -> &Value {
    envelope
        .pointer("/data/items")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.items, got {envelope}"))
}

pub fn page_info(envelope: &Value) -> &Value {
    envelope
        .pointer("/data/pageInfo")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.pageInfo, got {envelope}"))
}

pub fn assert_cursor_page_info(envelope: &Value) {
    let page_info = page_info(envelope);
    assert_eq!(
        page_info.get("mode").and_then(|value| value.as_str()),
        Some("cursor"),
        "pageInfo.mode must be cursor, got {page_info}"
    );
    assert!(
        page_info.get("hasMore").is_some(),
        "pageInfo.hasMore must be present, got {page_info}"
    );
}

//! Helpers for asserting SdkWorkApiResponse envelopes in HTTP integration tests.

use serde_json::Value;

pub fn item<'a>(envelope: &'a Value) -> &'a Value {
    envelope
        .pointer("/data/item")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.item, got {envelope}"))
}

pub fn items<'a>(envelope: &'a Value) -> &'a Value {
    envelope
        .pointer("/data/items")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.items, got {envelope}"))
}

pub fn page_info<'a>(envelope: &'a Value) -> &'a Value {
    envelope
        .pointer("/data/pageInfo")
        .unwrap_or_else(|| panic!("expected SdkWorkApiResponse data.pageInfo, got {envelope}"))
}

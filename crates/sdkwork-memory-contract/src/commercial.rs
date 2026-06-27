//! Commercial memory management contract types.
//!
//! These types model the commercial layer that binds memory spaces to
//! subjects (tenants, organizations, users, applications), assigns
//! capabilities, and enforces policies for retrieval and access control.

use crate::serde_int64::{
    deserialize_option_u64_from_string_or_number, deserialize_u64_from_string_or_number,
    serialize_option_u64_as_string, serialize_u64_as_string,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Subject
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectType {
    Tenant,
    Organization,
    User,
    Application,
    Service,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySubject {
    pub subject_id: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub organization_id: Option<u64>,
    pub subject_type: SubjectType,
    pub subject_ref: String,
    pub display_name: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub default_space_id: Option<u64>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubjectCommand {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub organization_id: Option<u64>,
    pub subject_type: SubjectType,
    pub subject_ref: String,
    pub display_name: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub default_space_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubjectCommand {
    pub display_name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub default_space_id: Option<u64>,
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSubjectsQuery {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_type: Option<SubjectType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySubjectList {
    pub items: Vec<MemorySubject>,
    pub page_info: crate::dto::MemoryPageInfo,
}

// ---------------------------------------------------------------------------
// Memory Binding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindingKind {
    Ownership,
    Access,
    Share,
    Reference,
    Provision,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryBinding {
    pub binding_id: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub space_id: Option<u64>,
    pub binding_kind: BindingKind,
    pub binding_role: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub source_subject_id: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub target_subject_id: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub target_space_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_codes: Option<Vec<String>>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBindingCommand {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub space_id: Option<u64>,
    pub binding_kind: BindingKind,
    pub binding_role: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub source_subject_id: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub target_subject_id: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub target_space_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_codes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBindingsQuery {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub source_subject_id: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub target_subject_id: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub target_space_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_kind: Option<BindingKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryBindingList {
    pub items: Vec<MemoryBinding>,
    pub page_info: crate::dto::MemoryPageInfo,
}

// ---------------------------------------------------------------------------
// Capability Binding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityMode {
    Allow,
    Deny,
    Conditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityTargetType {
    Subject,
    Space,
    Binding,
    Memory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCapabilityBinding {
    pub capability_binding_id: String,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    pub capability_code: String,
    pub target_type: CapabilityTargetType,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub target_id: u64,
    pub mode: CapabilityMode,
    pub priority: i32,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCapabilityBindingCommand {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    pub capability_code: String,
    pub target_type: CapabilityTargetType,
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub target_id: u64,
    pub mode: CapabilityMode,
    #[serde(default)]
    pub priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCapabilityBindingsQuery {
    #[serde(
        serialize_with = "serialize_u64_as_string",
        deserialize_with = "deserialize_u64_from_string_or_number"
    )]
    pub tenant_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_type: Option<CapabilityTargetType>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_u64_as_string",
        deserialize_with = "deserialize_option_u64_from_string_or_number"
    )]
    pub target_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCapabilityBindingList {
    pub items: Vec<MemoryCapabilityBinding>,
    pub page_info: crate::dto::MemoryPageInfo,
}

// ---------------------------------------------------------------------------
// Resolved capability (result of binding resolution)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCapability {
    pub capability_code: String,
    pub mode: CapabilityMode,
    pub priority: i32,
    pub source: String,
}

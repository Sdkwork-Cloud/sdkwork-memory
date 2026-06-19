use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub r#type: String,
    pub title: String,
    pub status: u16,
    pub detail: Option<String>,
    pub instance: Option<String>,
    pub code: Option<String>,
}

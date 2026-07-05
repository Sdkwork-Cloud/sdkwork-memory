use serde_json::Value;

use sdkwork_memory_contract::{MemoryServiceError, MemoryServiceResult};

const SUPPORTED_RETRIEVERS: &[&str] = &["keyword", "dictionary", "sql", "time", "event"];

pub fn validate_retrieval_retrievers(retrievers: &Value) -> MemoryServiceResult<()> {
    let Some(object) = retrievers.as_object() else {
        return Err(MemoryServiceError::validation(
            "retrievers must be a JSON object mapping retriever names to weight config",
        ));
    };
    if object.is_empty() {
        return Err(MemoryServiceError::validation("retrievers must not be empty"));
    }
    for key in object.keys() {
        if !SUPPORTED_RETRIEVERS.contains(&key.as_str()) {
            return Err(MemoryServiceError::validation(format!(
                "unsupported retriever '{key}': supported values are {}",
                SUPPORTED_RETRIEVERS.join(", ")
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_unsupported_vector_retriever() {
        let err = validate_retrieval_retrievers(&json!({ "vector": { "weight": 1.0 } }))
            .expect_err("vector retriever must be rejected");
        assert!(format!("{err:?}").contains("unsupported retriever 'vector'"));
    }

    #[test]
    fn accepts_supported_retriever_mix() {
        validate_retrieval_retrievers(&json!({
            "keyword": { "weight": 0.6 },
            "dictionary": { "weight": 0.4 }
        }))
        .expect("supported retrievers must pass validation");
    }
}

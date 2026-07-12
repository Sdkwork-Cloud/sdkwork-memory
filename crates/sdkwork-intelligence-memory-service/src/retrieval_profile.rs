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
        return Err(MemoryServiceError::validation(
            "retrievers must not be empty",
        ));
    }
    for (key, config) in object {
        if !SUPPORTED_RETRIEVERS.contains(&key.as_str()) {
            return Err(MemoryServiceError::validation(format!(
                "unsupported retriever '{key}': supported values are {}",
                SUPPORTED_RETRIEVERS.join(", ")
            )));
        }
        config
            .as_object()
            .and_then(|config| config.get("weight"))
            .and_then(Value::as_f64)
            .filter(|weight| weight.is_finite() && *weight > 0.0 && *weight <= 10.0)
            .ok_or_else(|| {
                MemoryServiceError::validation(format!(
                    "retriever '{key}' weight must be a finite number greater than 0 and at most 10"
                ))
            })?;
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

    #[test]
    fn rejects_missing_zero_or_excessive_weights() {
        for retrievers in [
            json!({ "keyword": {} }),
            json!({ "keyword": { "weight": 0.0 } }),
            json!({ "keyword": { "weight": 10.1 } }),
        ] {
            assert!(validate_retrieval_retrievers(&retrievers).is_err());
        }
    }
}

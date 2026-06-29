use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_intelligence_memory_service::OpenMemoryService;
use sdkwork_memory_contract::MemoryOpenApiRequestContext;
use sdkwork_memory_test_support::api_envelope;
use sdkwork_routes_memory_open_api::build_router_with_open_api;
use serde_json::json;
use tower::util::ServiceExt;

fn open_context() -> MemoryOpenApiRequestContext {
    MemoryOpenApiRequestContext::for_open_surface("api-key-001", 100_001, Some(2001))
}

#[tokio::test]
async fn open_api_mvp_flow_memory_retrieval_and_context_pack_without_embeddings() {
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = build_router_with_open_api(OpenMemoryService::new(store));

    let create_memory = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/memories")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    json!({
                        "spaceId": "2",
                        "scope": "user",
                        "memoryType": "semantic",
                        "canonicalText": "User prefers concise answers"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_memory.status(), StatusCode::CREATED);

    let retrieval = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/retrievals")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    json!({
                        "query": "concise answers",
                        "spaceIds": ["2"],
                        "topK": 5,
                        "contextBudgetTokens": 512
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieval.status(), StatusCode::CREATED);
    let retrieval_body = to_bytes(retrieval.into_body(), usize::MAX).await.unwrap();
    let retrieval_json: serde_json::Value = serde_json::from_slice(&retrieval_body).unwrap();
    assert!(api_envelope::item(&retrieval_json)["hits"]
        .as_array()
        .unwrap()
        .iter()
        .any(|hit| hit["retrieverName"] == "keyword"));

    let context_pack = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/context_packs")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    json!({
                        "query": "concise answers",
                        "spaceIds": ["2"],
                        "contextBudgetTokens": 512
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(context_pack.status(), StatusCode::CREATED);
    let pack_body = to_bytes(context_pack.into_body(), usize::MAX)
        .await
        .unwrap();
    let pack_json: serde_json::Value = serde_json::from_slice(&pack_body).unwrap();
    let pack_item = api_envelope::item(&pack_json);
    assert!(pack_item["pack"]["embeddingOptional"]
        .as_bool()
        .unwrap_or(false));
    assert!(pack_item["pack"]["fragments"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn open_api_mvp_flow_event_extraction_candidates_and_feedback() {
    let store = sdkwork_memory_test_support::space_fixtures::new_seeded_in_memory_store().await;
    let app = build_router_with_open_api(OpenMemoryService::new(store));

    let create_event = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/events")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    json!({
                        "spaceId": "2",
                        "eventType": "conversation.turn",
                        "sourceType": "chat",
                        "eventTime": "2026-06-10T12:00:00Z",
                        "payload": { "content": "User asked for bullet-point summaries" }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_event.status(), StatusCode::CREATED);
    let event_body = to_bytes(create_event.into_body(), usize::MAX)
        .await
        .unwrap();
    let event_json: serde_json::Value = serde_json::from_slice(&event_body).unwrap();
    let event_id = api_envelope::item(&event_json)["eventId"].as_str().unwrap();

    let extraction = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/extractions")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    json!({
                        "spaceId": "2",
                        "inputEvents": [event_id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(extraction.status(), StatusCode::CREATED);
    let extraction_body = to_bytes(extraction.into_body(), usize::MAX).await.unwrap();
    let extraction_json: serde_json::Value = serde_json::from_slice(&extraction_body).unwrap();
    let extraction_item = api_envelope::item(&extraction_json);
    assert_eq!(extraction_item["jobType"], "extraction");
    assert_eq!(extraction_item["state"], "completed");

    let candidates = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mem/v3/api/memory/candidates?spaceId=2")
                .extension(open_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(candidates.status(), StatusCode::OK);
    let candidates_body = to_bytes(candidates.into_body(), usize::MAX).await.unwrap();
    let candidates_json: serde_json::Value = serde_json::from_slice(&candidates_body).unwrap();
    let candidate_id = api_envelope::items(&candidates_json)[0]["candidateId"]
        .as_str()
        .unwrap()
        .to_string();

    let candidate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/mem/v3/api/memory/candidates/{candidate_id}"))
                .extension(open_context())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(candidate.status(), StatusCode::OK);

    let feedback = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mem/v3/api/memory/feedback")
                .header("content-type", "application/json")
                .extension(open_context())
                .body(Body::from(
                    json!({
                        "targetType": "candidate",
                        "targetId": candidate_id,
                        "feedbackType": "helpful"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feedback.status(), StatusCode::CREATED);
}

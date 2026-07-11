use std::collections::HashMap;

use serde_json::Value;

use sdkwork_memory_contract::MemoryRecord;
use sdkwork_utils_rust::is_blank;

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalCandidate {
    pub memory: MemoryRecord,
    pub retriever_name: String,
    pub raw_score: f64,
    pub rank: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FusedRetrievalHit {
    pub memory: MemoryRecord,
    pub retriever_name: String,
    pub raw_score: f64,
    pub fused_score: f64,
    pub rank: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalRecordInput {
    pub memory_id: String,
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object_text: String,
    pub canonical_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalEventInput {
    pub memory_id: Option<String>,
    pub event_id: String,
    pub payload_text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrchestratedCandidate {
    pub record: RetrievalRecordInput,
    pub retriever_name: String,
    pub raw_score: f64,
}

// ---------------------------------------------------------------------------
// Tokenisation helpers — CJK-aware
// ---------------------------------------------------------------------------

/// True when `ch` is a CJK Unified Ideograph (U+4E00–U+9FFF), CJK
/// Extension A (U+3400–U+4DBF), or a full-width punctuation/compatibility
/// ideograph.
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{3400}'..='\u{4DBF}'    // CJK Unified Extension A
        | '\u{4E00}'..='\u{9FFF}'  // CJK Unified Ideographs
        | '\u{F900}'..='\u{FAFF}'  // CJK Compatibility Ideographs
        | '\u{2F800}'..='\u{2FA1F}' // CJK Compatibility Supplement (supplementary plane)
    )
}

/// Tokenise a trimmed, lowercased query into searchable tokens.
///
/// For CJK text, each character is treated as a separate token (character-level
/// unigram).  For non-CJK text, whitespace delimiters are used.  Mixed text
/// (e.g. "hello世界") is handled correctly because we look at each character
/// individually.
fn tokenise_query(text: &str) -> Vec<String> {
    let text = text.trim().to_lowercase();
    if text.is_empty() {
        return vec![];
    }

    let chars: Vec<char> = text.chars().collect();

    // If *any* character in the query is CJK, use character-level tokenisation
    // for the entire query so that a CJK query string finds CJK content and
    // ASCII tokens still get found by substring.
    let has_cjk = chars.iter().any(|c| is_cjk(*c));

    if has_cjk {
        // Character-level unigrams — deduplicate while preserving order.
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        for ch in chars {
            if ch.is_alphanumeric() && seen.insert(ch) {
                tokens.push(ch.to_string());
            }
        }
        tokens
    } else {
        // Whitespace-delimited word-level tokens.
        chars
            .split(|c| c.is_ascii_whitespace())
            .filter(|w| !w.is_empty())
            .map(|w| w.iter().collect::<String>())
            .collect()
    }
}

pub fn fuse_retrieval_candidates(
    candidates: Vec<RetrievalCandidate>,
    top_k: usize,
) -> Vec<FusedRetrievalHit> {
    let mut sorted = candidates;
    sorted.sort_by(|left, right| {
        right
            .raw_score
            .partial_cmp(&left.raw_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.memory.memory_id.cmp(&right.memory.memory_id))
    });

    sorted
        .into_iter()
        .take(top_k)
        .enumerate()
        .map(|(index, candidate)| FusedRetrievalHit {
            memory: candidate.memory,
            retriever_name: candidate.retriever_name,
            raw_score: candidate.raw_score,
            fused_score: candidate.raw_score,
            rank: (index as i32) + 1,
        })
        .collect()
}

pub fn keyword_match_score(query: &str, canonical_text: &str) -> f64 {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return 0.0;
    }

    let haystack = canonical_text.to_lowercase();
    if haystack == query {
        return 1.0;
    }
    if haystack.contains(&query) {
        return 0.85;
    }

    token_overlap_score(&query, &haystack, &tokenise_query(&query))
}

pub fn dictionary_match_score(
    query: &str,
    subject: Option<&str>,
    predicate: Option<&str>,
    object_text: &str,
) -> f64 {
    let mut corpus = String::new();
    if let Some(subject) = subject {
        corpus.push_str(subject);
        corpus.push(' ');
    }
    if let Some(predicate) = predicate {
        corpus.push_str(predicate);
        corpus.push(' ');
    }
    corpus.push_str(object_text);
    let tokens = tokenise_query(&query.trim().to_lowercase());
    token_overlap_score(&query.trim().to_lowercase(), &corpus.to_lowercase(), &tokens)
}

pub fn sql_structured_match_score(
    query: &str,
    subject: Option<&str>,
    predicate: Option<&str>,
) -> f64 {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return 0.0;
    }

    let mut score: f64 = 0.0;
    let tokens = tokenise_query(&query);
    if subject
        .map(|value| value.to_lowercase() == query)
        .unwrap_or(false)
    {
        score = score.max(1.0);
    }
    if predicate
        .map(|value| value.to_lowercase() == query)
        .unwrap_or(false)
    {
        score = score.max(0.95);
    }

    // CJK-aware partial match
    for token in &tokens {
        if subject
            .map(|value| value.to_lowercase().contains(token.as_str()))
            .unwrap_or(false)
        {
            score = score.max(0.8);
            break;
        }
    }
    score
}

pub fn time_recency_score(created_at: &str) -> f64 {
    let parsed = sdkwork_utils_rust::parse_datetime(created_at, None);
    let Some(timestamp) = parsed else {
        return 0.35;
    };
    let age_hours = (sdkwork_utils_rust::now() - timestamp).num_hours().max(0) as f64;
    (1.0 / (1.0 + age_hours / 24.0)).clamp(0.1, 1.0)
}

pub fn event_match_score(query: &str, payload_text: &str) -> f64 {
    keyword_match_score(query, payload_text)
}

fn token_overlap_score(_query: &str, haystack: &str, tokens: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }

    // For CJK text, a character-level overlap heuristic is more meaningful.
    if tokens.iter().any(|t| is_cjk(t.chars().next().unwrap_or_default())) {
        let matched = tokens
            .iter()
            .filter(|token| haystack.contains(token.as_str()))
            .count();
        return matched as f64 / tokens.len() as f64;
    }

    let matched = tokens
        .iter()
        .filter(|token| haystack.contains(token.as_str()))
        .count();
    matched as f64 / tokens.len() as f64
}

fn retriever_weight(profile: Option<&Value>, retriever: &str) -> f64 {
    profile
        .and_then(|value| value.get(retriever))
        .and_then(|entry| entry.get("weight"))
        .and_then(|weight| weight.as_f64())
        .filter(|weight| *weight > 0.0)
        .unwrap_or(0.0)
}

fn retriever_enabled(profile: Option<&Value>, retriever: &str, default_enabled: bool) -> bool {
    if let Some(profile) = profile {
        retriever_weight(Some(profile), retriever) > 0.0
    } else {
        default_enabled
    }
}

pub fn orchestrate_retrieval_candidates(
    query: &str,
    records: &[RetrievalRecordInput],
    events: &[RetrievalEventInput],
    profile: Option<&Value>,
    top_k: usize,
) -> Vec<OrchestratedCandidate> {
    let mut weighted: HashMap<String, (RetrievalRecordInput, f64, String)> = HashMap::new();

    let push_score = |map: &mut HashMap<String, (RetrievalRecordInput, f64, String)>,
                      record: RetrievalRecordInput,
                      retriever: &str,
                      raw_score: f64,
                      weight: f64| {
        if raw_score <= 0.0 || weight <= 0.0 {
            return;
        }
        let fused = raw_score * weight;
        let key = record.memory_id.clone();
        map.entry(key)
            .and_modify(|existing| {
                if fused > existing.1 {
                    *existing = (record.clone(), fused, retriever.to_string());
                }
            })
            .or_insert((record, fused, retriever.to_string()));
    };

    if retriever_enabled(profile, "keyword", true) {
        let weight = retriever_weight(profile, "keyword").max(1.0);
        for record in records.iter().take(top_k.saturating_mul(2)) {
            let score = keyword_match_score(query, &record.canonical_text);
            push_score(&mut weighted, record.clone(), "keyword", score, weight);
        }
    }

    if retriever_enabled(profile, "dictionary", true) {
        let weight = retriever_weight(profile, "dictionary").max(0.85);
        for record in records.iter().take(top_k.saturating_mul(2)) {
            let score = dictionary_match_score(
                query,
                record.subject.as_deref(),
                record.predicate.as_deref(),
                &record.object_text,
            );
            push_score(&mut weighted, record.clone(), "dictionary", score, weight);
        }
    }

    if retriever_enabled(profile, "sql", true) {
        let weight = retriever_weight(profile, "sql").max(0.75);
        for record in records.iter().take(top_k.saturating_mul(2)) {
            let score = sql_structured_match_score(
                query,
                record.subject.as_deref(),
                record.predicate.as_deref(),
            );
            push_score(&mut weighted, record.clone(), "sql", score, weight);
        }
    }

    if retriever_enabled(profile, "time", true) {
        let weight = retriever_weight(profile, "time").max(0.5);
        for record in records.iter().take(top_k.saturating_mul(2)) {
            let score = time_recency_score(&record.created_at);
            if keyword_match_score(query, &record.canonical_text) > 0.0 || is_blank(Some(query)) {
                push_score(&mut weighted, record.clone(), "time", score, weight);
            }
        }
    }

    if retriever_enabled(profile, "event", true) {
        let weight = retriever_weight(profile, "event").max(0.6);
        for event in events.iter().take(top_k.saturating_mul(2)) {
            let score = event_match_score(query, &event.payload_text);
            if score <= 0.0 {
                continue;
            }
            let synthetic = RetrievalRecordInput {
                memory_id: event
                    .memory_id
                    .clone()
                    .unwrap_or_else(|| format!("event:{}", event.event_id)),
                subject: Some("event".to_string()),
                predicate: Some("mentions".to_string()),
                object_text: event.payload_text.clone(),
                canonical_text: event.payload_text.clone(),
                created_at: event.created_at.clone(),
            };
            push_score(&mut weighted, synthetic, "event", score, weight);
        }
    }

    let mut results: Vec<OrchestratedCandidate> = weighted
        .into_values()
        .map(|(record, fused, retriever_name)| OrchestratedCandidate {
            record,
            retriever_name,
            raw_score: fused,
        })
        .collect();

    results.sort_by(|left, right| {
        right
            .raw_score
            .partial_cmp(&left.raw_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.record.memory_id.cmp(&right.record.memory_id))
    });
    results.truncate(top_k);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orchestrator_combines_keyword_and_dictionary_signals() {
        let records = vec![RetrievalRecordInput {
            memory_id: "1".to_string(),
            subject: Some("preference".to_string()),
            predicate: Some("is".to_string()),
            object_text: "concise answers".to_string(),
            canonical_text: "User prefers concise answers".to_string(),
            created_at: sdkwork_utils_rust::format_datetime(sdkwork_utils_rust::now(), None),
        }];
        let hits = orchestrate_retrieval_candidates(
            "concise answers",
            &records,
            &[],
            Some(&serde_json::json!({
                "keyword": { "weight": 1.0 },
                "dictionary": { "weight": 0.85 }
            })),
            5,
        );
        assert!(!hits.is_empty());
        assert!(hits[0].raw_score > 0.0);
    }
}

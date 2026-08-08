use makosh_events_api::NewEventEnvelope;
use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Transaction};

use makosh_events_postgres::store::EventStore;

use super::constants::AI_EMBEDDING_DIMENSION;
use super::errors::AiError;
use super::semantic::models::SemanticSearchResult;

pub(super) fn merge_retrieval_results(
    vector_results: Vec<SemanticSearchResult>,
    text_results: Vec<SemanticSearchResult>,
) -> Vec<SemanticSearchResult> {
    let mut merged: HashMap<(String, String), SemanticSearchResult> = HashMap::new();
    for mut result in vector_results {
        result.score *= 0.75;
        merged.insert(
            (result.source_kind.clone(), result.source_id.clone()),
            result,
        );
    }
    for mut result in text_results {
        result.score += 0.75;
        let key = (result.source_kind.clone(), result.source_id.clone());
        merged
            .entry(key)
            .and_modify(|existing| existing.score += result.score)
            .or_insert(result);
    }

    let mut results = merged.into_values().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.source_kind.cmp(&right.source_kind))
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    results
}

pub(super) fn halfvec_literal(embedding: &[f32]) -> Result<String, AiError> {
    if embedding.len() != AI_EMBEDDING_DIMENSION {
        return Err(AiError::InvalidEmbeddingDimension {
            expected: AI_EMBEDDING_DIMENSION,
            actual: embedding.len(),
        });
    }

    let mut literal = String::with_capacity(embedding.len() * 10);
    literal.push('[');
    for (index, value) in embedding.iter().enumerate() {
        if !value.is_finite() {
            return Err(AiError::InvalidRequest("embedding values must be finite"));
        }
        if index > 0 {
            literal.push(',');
        }
        literal.push_str(&value.to_string());
    }
    literal.push(']');
    Ok(literal)
}

pub(super) fn content_hash(value: &str) -> String {
    format!("sha256:{}", sha256_hex(value.as_bytes()))
}

pub(super) fn semantic_embedding_id(
    source_kind: &str,
    source_id: &str,
    embedding_model: &str,
) -> String {
    format!(
        "semantic_embedding:v3:{}:{}",
        source_kind,
        sha256_hex(format!("{source_id}\n{embedding_model}").as_bytes())
    )
}

pub(crate) fn run_id_from_command(workflow: &str, command_id: &str) -> String {
    format!("ai_run:v3:{workflow}:{}", sha256_hex(command_id.as_bytes()))
}

pub(crate) fn event_id_from_command(event_type: &str, command_id: &str) -> String {
    format!("{event_type}:{}", sha256_hex(command_id.as_bytes()))
}

pub(super) fn ai_task_candidate_id(source_kind: &str, source_id: &str, title: &str) -> String {
    format!(
        "task_candidate:v3:ai:{}",
        sha256_hex(format!("{source_kind}\n{source_id}\n{title}").as_bytes())
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

pub(super) fn recipients_text(value: Value) -> String {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

pub(super) fn validate_non_empty(field_name: &'static str, value: &str) -> Result<String, AiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AiError::InvalidRequest(field_name));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn validate_limit(limit: i64) -> Result<i64, AiError> {
    if !(1..=100).contains(&limit) {
        return Err(AiError::InvalidRequest("limit must be between 1 and 100"));
    }
    Ok(limit)
}

pub(crate) fn text_preview(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    let mut preview = String::new();
    for character in trimmed.chars().take(max_chars) {
        preview.push(character);
    }
    if trimmed.chars().count() > max_chars {
        preview.push_str("...");
    }
    preview
}

pub(super) fn elapsed_ms(started_at: Instant) -> i64 {
    i64::try_from(started_at.elapsed().as_millis()).unwrap_or(i64::MAX)
}

#[allow(dead_code)]
async fn _append_ai_event_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    event: &NewEventEnvelope,
) -> Result<i64, AiError> {
    Ok(EventStore::append_in_transaction(transaction, event).await?)
}

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::review::models::{
    NewReviewItem, NewReviewItemEvidence, ReviewItem, ReviewItemKind, ReviewItemStatus,
    ReviewPromotionTarget,
};
use crate::domains::review::service::ReviewInboxService;
use crate::domains::review::store::{ReviewInboxStore, ReviewItemEvidenceRecord};
use crate::engines::attention::{
    engine::AttentionEngine,
    models::{
        AttentionCandidate, AttentionCard, AttentionEvidenceRef, AttentionRelatedEntity,
        AttentionSuggestedAction,
    },
};
use crate::engines::context_packs::{
    models::ContextPack,
    review::{
        ReviewContextPackEvidence, ReviewContextPackInput, ReviewContextPackItem,
        build_review_context_pack,
    },
    store::ContextPackStore,
};
use crate::workflows::review_promotion::ReviewPromotionService;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

const DEFAULT_REVIEW_LIMIT: i64 = 50;
const MIN_REVIEW_LIMIT: i64 = 1;
const MAX_REVIEW_LIMIT: i64 = 100;
const REVIEW_STATUS_ACTIVE: &str = "active";
const REVIEW_STATUS_ALL: &str = "all";

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewItemsQuery {
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReviewItemsResponse {
    items: Vec<ReviewItem>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReviewAttentionCardsResponse {
    cards: Vec<AttentionCard>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateReviewItemRequest {
    item_kind: String,
    title: String,
    summary: String,
    confidence: f64,
    metadata: Option<Value>,
    evidence: Vec<CreateReviewItemEvidenceRequest>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateReviewItemEvidenceRequest {
    observation_id: String,
    evidence_role: Option<String>,
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PromoteReviewItemRequest {
    target_domain: String,
    target_entity_kind: String,
    target_entity_id: String,
}

pub(crate) async fn get_v1_review_items(
    State(state): State<AppState>,
    Query(query): Query<ReviewItemsQuery>,
) -> Result<Json<ReviewItemsResponse>, ApiError> {
    let items = list_review_items_for_query(&state, query).await?;
    Ok(Json(ReviewItemsResponse { items }))
}

pub(crate) async fn get_v1_review_attention_cards(
    State(state): State<AppState>,
    Query(query): Query<ReviewItemsQuery>,
) -> Result<Json<ReviewAttentionCardsResponse>, ApiError> {
    let store = review_store(&state)?;
    let items = list_review_items_for_query(&state, query).await?;
    let mut candidates = Vec::with_capacity(items.len());

    for item in items {
        let evidence = store.list_evidence(&item.review_item_id).await?;
        let trace_id = review_item_trace_id(&state, &item.review_item_id).await?;
        candidates.push(attention_candidate_from_review_item(
            &item, evidence, trace_id,
        ));
    }

    let cards = AttentionEngine::build_cards(&candidates).map_err(|error| {
        tracing::error!(error = %error, "review attention card generation failed");
        ApiError::InvalidReviewQuery("review attention card generation failed")
    })?;

    Ok(Json(ReviewAttentionCardsResponse { cards }))
}

pub(crate) async fn get_v1_review_item_context_pack(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ContextPack>, ApiError> {
    let store = review_store(&state)?;
    let item = store.get(&review_item_id).await?;
    let evidence = store.list_evidence(&review_item_id).await?;
    let evidence = enrich_review_pack_evidence_with_observations(&state, evidence).await?;
    let trace_id = review_item_trace_id(&state, &item.review_item_id).await?;
    let input = review_context_pack_input(item, evidence, trace_id);
    let result = build_review_context_pack(input).map_err(|error| {
        tracing::error!(
            error = %error,
            review_item_id = %review_item_id,
            "review context pack generation failed"
        );
        ApiError::InvalidReviewQuery("review context pack generation failed")
    })?;
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    let stored = ContextPackStore::new(pool.clone())
        .upsert_with_sources(&result.pack, &result.sources)
        .await
        .map_err(|error| {
            tracing::error!(
                error = %error,
                review_item_id = %review_item_id,
                "review context pack persistence failed"
            );
            ApiError::InvalidReviewQuery("review context pack generation failed")
        })?;

    Ok(Json(stored))
}

pub(crate) async fn post_v1_review_items(
    State(state): State<AppState>,
    Json(request): Json<CreateReviewItemRequest>,
) -> Result<Json<ReviewItem>, ApiError> {
    let mut item = NewReviewItem::new(
        parse_item_kind(&request.item_kind)?,
        request.title,
        request.summary,
        request.confidence,
    );
    if let Some(metadata) = request.metadata {
        item = item.metadata(metadata);
    }

    let evidence: Vec<NewReviewItemEvidence> = request
        .evidence
        .into_iter()
        .map(|item| {
            let mut evidence = NewReviewItemEvidence::new(item.observation_id);
            if let Some(role) = item.evidence_role {
                evidence = evidence.role(role);
            }
            if let Some(metadata) = item.metadata {
                evidence = evidence.metadata(metadata);
            }
            evidence
        })
        .collect();

    let item = review_store(&state)?
        .create_with_evidence(&item, &evidence)
        .await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_approve(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Approved).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_dismiss(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Dismissed).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_archive(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Archived).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_take(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::InReview).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_promote(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
    Json(request): Json<PromoteReviewItemRequest>,
) -> Result<Json<ReviewItem>, ApiError> {
    let target = ReviewPromotionTarget::new(
        request.target_domain,
        request.target_entity_kind,
        request.target_entity_id,
    );
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let observation =
        crate::app::api_support::stores::domain_stores::app_store::<ObservationStore>(pool.clone())
            .capture(
                &NewObservation::new(
                    "REVIEW_TRANSITION",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "review_item_id": review_item_id,
                        "operation": "review_item_promote",
                        "target_domain": target.target_domain,
                        "target_entity_kind": target.target_entity_kind,
                        "target_entity_id": target.target_entity_id,
                    }),
                    format!("review-item://{review_item_id}/promote"),
                )
                .provenance(json!({
                    "captured_by": "review_api.post_v1_review_item_promote",
                    "endpoint": "post_v1_review_item_promote",
                })),
            )
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "review item promote observation capture failed");
                ApiError::InvalidReviewQuery("review item promote observation capture failed")
            })?;

    let item = ReviewPromotionService::new(pool.clone())
        .promote_with_observation(
            &review_item_id,
            target,
            Some(&observation.observation_id),
            Some(json!({
                "captured_by": "review_api.post_v1_review_item_promote",
                "endpoint": "post_v1_review_item_promote",
            })),
            None,
            None,
        )
        .await?;
    Ok(Json(item))
}

async fn transition_review_item_status(
    state: &AppState,
    review_item_id: &str,
    status: ReviewItemStatus,
) -> Result<ReviewItem, ApiError> {
    let item = review_service(state)?
        .transition_status_from_manual(
            review_item_id,
            status,
            "review_api.transition_review_item_status",
            "transition_review_item_status",
        )
        .await?;
    Ok(item)
}

fn review_service(state: &AppState) -> Result<ReviewInboxService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ReviewInboxService::new(pool.clone()))
}

fn review_store(state: &AppState) -> Result<ReviewInboxStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::stores::domain_stores::app_store::<
        ReviewInboxStore,
    >(pool.clone()))
}

async fn list_review_items_for_query(
    state: &AppState,
    query: ReviewItemsQuery,
) -> Result<Vec<ReviewItem>, ApiError> {
    let status = parse_status_filter(query.status.as_deref())?;
    let limit = validate_limit(query.limit)?;
    let store = review_store(state)?;
    let items = match status {
        ReviewItemsStatusFilter::Single(status) => store.list_by_status(status, limit).await?,
        ReviewItemsStatusFilter::Active => store.list_open(limit).await?,
        ReviewItemsStatusFilter::All => store.list_all(limit).await?,
    };
    Ok(items)
}

async fn review_item_trace_id(state: &AppState, review_item_id: &str) -> Result<String, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    let trace_id = makosh_events_postgres::store::EventStore::new(pool.clone())
        .first_trace_id_for_subject("review_item_id", review_item_id)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "review attention trace lookup failed");
            ApiError::InvalidReviewQuery("review attention trace lookup failed")
        })?;

    Ok(trace_id.unwrap_or_else(|| review_item_id.to_owned()))
}

fn attention_candidate_from_review_item(
    item: &ReviewItem,
    evidence: Vec<ReviewItemEvidenceRecord>,
    trace_id: String,
) -> AttentionCandidate {
    let evidence_count = evidence.len();
    let mut candidate = AttentionCandidate::new(
        item.review_item_id.clone(),
        item.item_kind.as_str(),
        item.title.clone(),
        item.summary.clone(),
        trace_id,
    )
    .status(item.status.as_str())
    .confidence(item.confidence)
    .evidence(attention_evidence_refs(evidence))
    .related_entities(attention_related_entities(item))
    .source_summary(format!(
        "Review item has {evidence_count} canonical observation evidence reference(s)."
    ))
    .suggested_actions(attention_suggested_actions(item.status));

    if let Some(group_key) = attention_group_key(&item.metadata) {
        candidate = candidate.group_key(group_key);
    }

    candidate
}

fn attention_evidence_refs(evidence: Vec<ReviewItemEvidenceRecord>) -> Vec<AttentionEvidenceRef> {
    evidence
        .into_iter()
        .map(|record| AttentionEvidenceRef::new(record.observation_id).role(record.evidence_role))
        .collect()
}

fn attention_related_entities(item: &ReviewItem) -> Vec<AttentionRelatedEntity> {
    let Some(entity_kind) = item.target_entity_kind.as_ref() else {
        return Vec::new();
    };
    let Some(entity_id) = item.target_entity_id.as_ref() else {
        return Vec::new();
    };

    let mut entity = AttentionRelatedEntity::new(entity_kind, entity_id);
    if let Some(target_domain) = item.target_domain.as_ref() {
        entity = entity.label(target_domain);
    }
    vec![entity]
}

fn attention_suggested_actions(status: ReviewItemStatus) -> Vec<AttentionSuggestedAction> {
    match status {
        ReviewItemStatus::New => vec![
            AttentionSuggestedAction::new("take_review", "Take review"),
            AttentionSuggestedAction::new("approve", "Approve"),
            AttentionSuggestedAction::new("dismiss", "Dismiss"),
        ],
        ReviewItemStatus::InReview => vec![
            AttentionSuggestedAction::new("approve", "Approve"),
            AttentionSuggestedAction::new("dismiss", "Dismiss"),
        ],
        ReviewItemStatus::Approved => vec![
            AttentionSuggestedAction::new("promote", "Promote"),
            AttentionSuggestedAction::new("dismiss", "Dismiss"),
        ],
        ReviewItemStatus::Promoted | ReviewItemStatus::Dismissed | ReviewItemStatus::Archived => {
            vec![AttentionSuggestedAction::new("archive", "Archive")]
        }
    }
}

fn attention_group_key(metadata: &Value) -> Option<String> {
    metadata
        .get("attention_group_key")
        .or_else(|| metadata.get("group_key"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn review_context_pack_input(
    item: ReviewItem,
    evidence: Vec<ReviewItemEvidenceRecord>,
    trace_id: String,
) -> ReviewContextPackInput {
    ReviewContextPackInput {
        review_item: ReviewContextPackItem {
            review_item_id: item.review_item_id,
            item_kind: item.item_kind.as_str().to_owned(),
            title: item.title,
            summary: item.summary,
            status: item.status.as_str().to_owned(),
            target_domain: item.target_domain,
            target_entity_kind: item.target_entity_kind,
            target_entity_id: item.target_entity_id,
            confidence: item.confidence,
            metadata: item.metadata,
            created_at: item.created_at,
            updated_at: item.updated_at,
        },
        evidence: evidence
            .into_iter()
            .map(|record| ReviewContextPackEvidence {
                observation_id: record.observation_id,
                evidence_role: record.evidence_role,
                metadata: record.metadata,
            })
            .collect(),
        trace_id,
    }
}

async fn enrich_review_pack_evidence_with_observations(
    state: &AppState,
    evidence: Vec<ReviewItemEvidenceRecord>,
) -> Result<Vec<ReviewItemEvidenceRecord>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    let observation_store =
        crate::app::api_support::stores::domain_stores::app_store::<ObservationStore>(pool.clone());

    let mut enriched = Vec::with_capacity(evidence.len());
    for mut record in evidence {
        match observation_store.get(&record.observation_id).await {
            Ok(Some(observation)) => {
                record.metadata = merge_observation_payload_into_evidence_metadata(
                    &record.metadata,
                    &observation.payload,
                );
            }
            Ok(None) => {
                // Keep legacy/embedded metadata when observation no longer exists in store.
                // This is non-fatal because observation data is optional for pack generation.
            }
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    review_item_observation_id = %record.observation_id,
                    "failed to load observation for review context pack",
                );
            }
        }

        enriched.push(record);
    }

    Ok(enriched)
}

fn merge_observation_payload_into_evidence_metadata(metadata: &Value, payload: &Value) -> Value {
    let mut merged = metadata.clone();
    let Value::Object(merged_map) = &mut merged else {
        return metadata.clone();
    };

    if let Value::Object(payload_map) = payload {
        let legacy_persona_id_key = ["person", "id"].join("_");
        if !merged_map.contains_key("persona_id")
            && let Some(value) = payload_map
                .get("persona_id")
                .or_else(|| payload_map.get(&legacy_persona_id_key))
        {
            merged_map.insert("persona_id".to_owned(), value.clone());
        }

        for key in [
            "title",
            "summary",
            "body",
            "text",
            "message_id",
            "subject",
            "document_id",
            "document_title",
            "person_name",
            "organization_id",
            "organization_name",
            "name",
        ] {
            if !merged_map.contains_key(key)
                && let Some(value) = payload_map.get(key)
            {
                merged_map.insert(key.to_owned(), value.clone());
            }
        }
    }

    Value::Object(merged_map.clone())
}

fn parse_item_kind(value: &str) -> Result<ReviewItemKind, ApiError> {
    ReviewItemKind::parse(value).map_err(ApiError::from)
}

enum ReviewItemsStatusFilter {
    Single(ReviewItemStatus),
    Active,
    All,
}

fn parse_status_filter(value: Option<&str>) -> Result<ReviewItemsStatusFilter, ApiError> {
    match value {
        None => Ok(ReviewItemsStatusFilter::Active),
        Some(value) => match value {
            REVIEW_STATUS_ACTIVE => Ok(ReviewItemsStatusFilter::Active),
            REVIEW_STATUS_ALL => Ok(ReviewItemsStatusFilter::All),
            unknown => ReviewItemStatus::parse(unknown)
                .map(ReviewItemsStatusFilter::Single)
                .map_err(ApiError::from),
        },
    }
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_REVIEW_LIMIT);
    if !(MIN_REVIEW_LIMIT..=MAX_REVIEW_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidReviewQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}

#[cfg(test)]
mod tests {
    use serde_json::Map;

    use super::*;

    #[test]
    fn observation_payload_persona_identity_is_merged_under_the_canonical_key() {
        let legacy_persona_id_key = ["person", "id"].join("_");
        let mut payload = Map::new();
        payload.insert(
            legacy_persona_id_key.clone(),
            Value::String("persona:legacy".to_owned()),
        );

        let merged =
            merge_observation_payload_into_evidence_metadata(&json!({}), &Value::Object(payload));

        assert_eq!(merged["persona_id"], json!("persona:legacy"));
        assert!(merged.get(&legacy_persona_id_key).is_none());

        let mut payload = Map::new();
        payload.insert(
            legacy_persona_id_key,
            Value::String("persona:legacy".to_owned()),
        );
        payload.insert(
            "persona_id".to_owned(),
            Value::String("persona:canonical".to_owned()),
        );

        let merged =
            merge_observation_payload_into_evidence_metadata(&json!({}), &Value::Object(payload));

        assert_eq!(merged["persona_id"], json!("persona:canonical"));
    }
}

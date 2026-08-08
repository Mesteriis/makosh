use serde_json::{Value, json};
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use makosh_observations_api::models::NewObservationLink;

use crate::errors::ObservationStoreError;
use crate::store::ObservationStore;

#[allow(clippy::too_many_arguments)]
pub async fn materialize_review_transition_link(
    pool: &PgPool,
    observation_id: Option<&str>,
    domain: &str,
    entity_kind: &str,
    entity_id: &str,
    state_field: &str,
    state_value: &str,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let Some(link) = build_review_transition_link(
        observation_id,
        domain,
        entity_kind,
        entity_id,
        state_field,
        state_value,
        metadata,
    ) else {
        return Ok(());
    };

    ObservationStore::new(pool.clone())
        .upsert_link(&link)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn materialize_review_transition_link_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    domain: &str,
    entity_kind: &str,
    entity_id: &str,
    state_field: &str,
    state_value: &str,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let Some(link) = build_review_transition_link(
        observation_id,
        domain,
        entity_kind,
        entity_id,
        state_field,
        state_value,
        metadata,
    ) else {
        return Ok(());
    };

    ObservationStore::upsert_link_in_transaction(transaction, &link).await?;
    Ok(())
}

fn build_review_transition_link(
    observation_id: Option<&str>,
    domain: &str,
    entity_kind: &str,
    entity_id: &str,
    state_field: &str,
    state_value: &str,
    metadata: Option<Value>,
) -> Option<NewObservationLink> {
    let observation_id = observation_id.filter(|value| !value.is_empty())?;

    let mut link = NewObservationLink::new(
        observation_id.to_owned(),
        domain,
        entity_kind,
        entity_id.to_owned(),
    )
    .relationship_kind("review_transition")
    .metadata(json!({
        state_field: state_value,
    }));

    if let Some(extra) = metadata {
        if let (Some(base), Some(extra)) = (link.metadata.as_object_mut(), extra.as_object()) {
            for (key, value) in extra {
                base.insert(key.clone(), value.clone());
            }
        } else {
            link = link.metadata(extra);
        }
    }

    Some(link)
}

#[allow(clippy::too_many_arguments)]
pub async fn link_domain_entity(
    pool: &PgPool,
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let link = build_domain_entity_link(
        observation_id,
        domain,
        entity_kind,
        entity_id.into(),
        relationship_kind,
        confidence,
        metadata,
    )?;
    ObservationStore::new(pool.clone())
        .upsert_link(&link)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn link_domain_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let link = build_domain_entity_link(
        observation_id,
        domain,
        entity_kind,
        entity_id.into(),
        relationship_kind,
        confidence,
        metadata,
    )?;
    ObservationStore::upsert_link_in_transaction(transaction, &link).await?;
    Ok(())
}

fn build_domain_entity_link(
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: String,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<NewObservationLink, ObservationStoreError> {
    let observation_id = observation_id.trim();
    if observation_id.is_empty() {
        return Err(ObservationStoreError::EmptyField("observation_id"));
    }

    let mut link =
        NewObservationLink::new(observation_id.to_owned(), domain, entity_kind, entity_id);

    if let Some(relationship_kind) = relationship_kind.filter(|value| !value.is_empty()) {
        link = link.relationship_kind(relationship_kind);
    }
    if let Some(confidence) = confidence {
        link = link.confidence(confidence);
    }
    if let Some(metadata) = metadata {
        link = link.metadata(metadata);
    }

    Ok(link)
}

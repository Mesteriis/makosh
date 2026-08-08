use serde_json::{Value, json};
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;

use super::errors::OrgCoreError;

pub(crate) async fn link_organization_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    organization_id: &str,
    action: &str,
    metadata: Option<Value>,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization",
        organization_id.to_owned(),
        None,
        None,
        Some(merge_metadata(
            json!({
                "action": action,
            }),
            metadata,
        )),
    )
    .await?;
    Ok(())
}

pub(crate) async fn link_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: &str,
    metadata: Value,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        entity_kind,
        entity_id.to_owned(),
        None,
        None,
        Some(metadata),
    )
    .await?;
    Ok(())
}

pub(crate) async fn link_review_transition_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: &str,
    metadata: Value,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        entity_kind,
        entity_id.to_owned(),
        Some("review_transition"),
        None,
        Some(metadata),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn link_email_domain_projection_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    organization_id: &str,
    organization_inserted: bool,
    organization_domain_id: &str,
    domain: &str,
    domain_inserted: bool,
    organization_identity_id: &str,
) -> Result<(), OrgCoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization",
        organization_id.to_owned(),
        Some("email_sync_projection"),
        None,
        Some(json!({
            "projection": "organization",
            "domain": domain,
            "inserted": organization_inserted,
        })),
    )
    .await?;

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization_domain",
        organization_domain_id.to_owned(),
        Some("email_sync_projection"),
        None,
        Some(json!({
            "projection": "organization_domain",
            "organization_id": organization_id,
            "domain": domain,
            "inserted": domain_inserted,
        })),
    )
    .await?;

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "organizations",
        "organization_identity",
        organization_identity_id.to_owned(),
        Some("email_sync_projection"),
        None,
        Some(json!({
            "projection": "organization_identity",
            "organization_id": organization_id,
            "identity_type": "email_domain",
            "identity_value": domain,
        })),
    )
    .await?;

    Ok(())
}

fn merge_metadata(base: Value, extra: Option<Value>) -> Value {
    match extra {
        Some(extra) if base.is_object() && extra.is_object() => {
            let mut merged = base;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base,
    }
}

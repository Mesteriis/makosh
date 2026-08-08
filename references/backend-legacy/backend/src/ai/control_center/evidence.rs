use chrono::Utc;
use serde_json::json;
use sqlx::{Postgres, Transaction};

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::link_domain_entity_in_transaction;
use makosh_observations_postgres::store::ObservationStore;

use super::errors::AiControlCenterError;
use super::models::{
    AiModelCatalogItem, AiModelRoute, AiPromptEvalRun, AiPromptTemplate, AiPromptVersion,
    AiProviderAccount,
};

pub(super) async fn capture_provider_account_observation(
    transaction: &mut Transaction<'_, Postgres>,
    provider: &AiProviderAccount,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    capture_provider_account_observation_with_origin(
        transaction,
        provider,
        relationship_kind,
        actor,
        ObservationOriginKind::LocalRuntime,
    )
    .await
}

pub(super) async fn capture_provider_account_observation_with_origin(
    transaction: &mut Transaction<'_, Postgres>,
    provider: &AiProviderAccount,
    relationship_kind: &str,
    actor: &str,
    origin_kind: ObservationOriginKind,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROVIDER_ACCOUNT",
            origin_kind,
            Utc::now(),
            json!({
                "provider_id": provider.provider_id,
                "provider_kind": provider.provider_kind,
                "provider_key": provider.provider_key,
                "display_name": provider.display_name,
                "status": provider.status,
                "consent_state": provider.consent_state,
                "consented_at": provider.consented_at,
                "config": provider.config,
                "capabilities": provider.capabilities,
                "action": relationship_kind,
            }),
            format!("ai-provider://{}", provider.provider_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_account",
        provider.provider_id.clone(),
        relationship_kind,
        json!({
            "provider_kind": provider.provider_kind,
            "provider_key": provider.provider_key,
            "status": provider.status,
            "consent_state": provider.consent_state,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_provider_secret_binding_observation(
    transaction: &mut Transaction<'_, Postgres>,
    provider_id: &str,
    secret_purpose: &str,
    secret_ref: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    capture_provider_secret_binding_observation_with_origin(
        transaction,
        provider_id,
        secret_purpose,
        secret_ref,
        actor,
        ObservationOriginKind::LocalRuntime,
    )
    .await
}

pub(super) async fn capture_provider_secret_binding_observation_with_origin(
    transaction: &mut Transaction<'_, Postgres>,
    provider_id: &str,
    secret_purpose: &str,
    secret_ref: &str,
    actor: &str,
    origin_kind: ObservationOriginKind,
) -> Result<(), AiControlCenterError> {
    let binding_id = format!("{provider_id}:{secret_purpose}");
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROVIDER_SECRET_BINDING",
            origin_kind,
            Utc::now(),
            json!({
                "provider_id": provider_id,
                "secret_purpose": secret_purpose,
                "secret_ref": secret_ref,
                "action": "bind",
            }),
            format!("ai-provider://{provider_id}/secret-binding/{secret_purpose}"),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": "bind",
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_secret_binding",
        binding_id,
        "bind",
        json!({
            "provider_id": provider_id,
            "secret_purpose": secret_purpose,
            "secret_ref": secret_ref,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_model_route_observation(
    transaction: &mut Transaction<'_, Postgres>,
    route: &AiModelRoute,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_MODEL_ROUTE",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "capability_slot": route.capability_slot,
                "provider_id": route.provider_id,
                "model_key": route.model_key,
                "action": "put",
            }),
            format!("ai-model-route://{}", route.capability_slot),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": "put",
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "model_route",
        route.capability_slot.clone(),
        "put",
        json!({
            "provider_id": route.provider_id,
            "model_key": route.model_key,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_prompt_template_observation(
    transaction: &mut Transaction<'_, Postgres>,
    prompt: &AiPromptTemplate,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROMPT_TEMPLATE",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "prompt_id": prompt.prompt_id,
                "name": prompt.name,
                "entity_scope": prompt.entity_scope,
                "capability_slot": prompt.capability_slot,
                "description": prompt.description,
                "is_system": prompt.is_system,
                "active_version_id": prompt.active_version_id,
                "metadata": prompt.metadata,
                "action": relationship_kind,
            }),
            format!("ai-prompt://{}", prompt.prompt_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "prompt_template",
        prompt.prompt_id.clone(),
        relationship_kind,
        json!({
            "entity_scope": prompt.entity_scope,
            "capability_slot": prompt.capability_slot,
            "active_version_id": prompt.active_version_id,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_prompt_version_observation(
    transaction: &mut Transaction<'_, Postgres>,
    version: &AiPromptVersion,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROMPT_TEMPLATE_VERSION",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "prompt_version_id": version.prompt_version_id,
                "prompt_id": version.prompt_id,
                "version_label": version.version_label,
                "body_template": version.body_template,
                "variables": version.variables,
                "status": version.status,
                "created_by_actor_id": version.created_by_actor_id,
                "action": relationship_kind,
            }),
            format!("ai-prompt-version://{}", version.prompt_version_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "prompt_template_version",
        version.prompt_version_id.clone(),
        relationship_kind,
        json!({
            "prompt_id": version.prompt_id,
            "version_label": version.version_label,
            "status": version.status,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_prompt_eval_run_observation(
    transaction: &mut Transaction<'_, Postgres>,
    eval_run: &AiPromptEvalRun,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROMPT_EVAL_RUN",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "eval_run_id": eval_run.eval_run_id,
                "prompt_id": eval_run.prompt_id,
                "prompt_version_id": eval_run.prompt_version_id,
                "provider_id": eval_run.provider_id,
                "model_key": eval_run.model_key,
                "source_refs": eval_run.source_refs,
                "variables": eval_run.variables,
                "output_text": eval_run.output_text,
                "score": eval_run.score,
                "notes": eval_run.notes,
                "actor_id": eval_run.actor_id,
                "action": "test",
            }),
            format!("ai-prompt-eval://{}", eval_run.eval_run_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": "test",
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "prompt_eval_run",
        eval_run.eval_run_id.clone(),
        "test",
        json!({
            "prompt_id": eval_run.prompt_id,
            "prompt_version_id": eval_run.prompt_version_id,
            "provider_id": eval_run.provider_id,
            "model_key": eval_run.model_key,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_model_catalog_item_observation(
    transaction: &mut Transaction<'_, Postgres>,
    model: &AiModelCatalogItem,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let entity_id = format!("{}:{}", model.provider_id, model.model_key);
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_MODEL_CATALOG_ITEM",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "provider_id": model.provider_id,
                "model_key": model.model_key,
                "display_name": model.display_name,
                "category": model.category,
                "privacy": model.privacy,
                "capabilities": model.capabilities,
                "context_window": model.context_window,
                "embedding_dimension": model.embedding_dimension,
                "is_available": model.is_available,
                "metadata": model.metadata,
                "action": relationship_kind,
            }),
            format!("ai-model-catalog://{entity_id}"),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "model_catalog_item",
        entity_id,
        relationship_kind,
        json!({
            "provider_id": model.provider_id,
            "model_key": model.model_key,
            "category": model.category,
            "privacy": model.privacy,
        }),
    )
    .await?;
    Ok(())
}

async fn link_ai_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    metadata: serde_json::Value,
) -> Result<(), makosh_observations_postgres::errors::ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "ai",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}

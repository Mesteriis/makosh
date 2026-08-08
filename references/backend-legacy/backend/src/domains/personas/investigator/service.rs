use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::assembly;
use super::errors::InvestigatorError;
use super::meeting_prep;
use super::models::{DossierReviewState, DossierSnapshot, MeetingPrep, PersonaDossier};
use super::snapshots;
use crate::domains::personas::core::evidence::link_persona_entity;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::review_links::materialize_review_transition_link;
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct PersonaInvestigator {
    pool: PgPool,
}

impl PersonaInvestigator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn assemble_dossier(
        &self,
        persona_id: &str,
    ) -> Result<PersonaDossier, InvestigatorError> {
        assembly::assemble_dossier(&self.pool, persona_id).await
    }

    pub async fn assemble_and_cache_dossier(
        &self,
        persona_id: &str,
    ) -> Result<(PersonaDossier, DossierSnapshot), InvestigatorError> {
        let dossier = self.assemble_dossier(persona_id).await?;
        let snapshot = self.cache_dossier_snapshot(&dossier).await?;
        Ok((dossier, snapshot))
    }

    pub async fn assemble_cache_and_record_refresh(
        &self,
        persona_id: &str,
        operation: &str,
        captured_by: &str,
        endpoint: &str,
        source_ref: String,
    ) -> Result<(PersonaDossier, DossierSnapshot), InvestigatorError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "PERSONA_MUTATION",
                    ObservationOriginKind::Manual,
                    chrono::Utc::now(),
                    json!({
                        "persona_id": persona_id,
                        "operation": operation,
                    }),
                    source_ref,
                )
                .provenance(json!({
                    "captured_by": captured_by,
                    "endpoint": endpoint,
                })),
            )
            .await?;
        let (dossier, snapshot) = self.assemble_and_cache_dossier(persona_id).await?;
        link_persona_entity(
            &self.pool,
            &observation.observation_id,
            "dossier_snapshot",
            snapshot.dossier_snapshot_id.clone(),
            Some("dossier_refresh"),
            Some(json!({
                "persona_id": persona_id,
                "trigger": endpoint,
            })),
        )
        .await?;
        Ok((dossier, snapshot))
    }

    pub async fn cache_dossier_snapshot(
        &self,
        dossier: &PersonaDossier,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        snapshots::cache_dossier_snapshot(&self.pool, dossier).await
    }

    pub async fn review_dossier_snapshot(
        &self,
        persona_id: &str,
        review_state: DossierReviewState,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        self.review_dossier_snapshot_with_observation(persona_id, review_state, None, None)
            .await
    }

    pub async fn review_dossier_snapshot_with_observation(
        &self,
        persona_id: &str,
        review_state: DossierReviewState,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<DossierSnapshot, InvestigatorError> {
        let snapshot =
            snapshots::review_dossier_snapshot(&self.pool, persona_id, review_state).await?;
        materialize_review_transition_link(
            &self.pool,
            observation_id,
            "personas",
            "dossier_snapshot",
            &snapshot.dossier_snapshot_id,
            "review_state",
            snapshot.review_state.as_str(),
            metadata
                .map(|extra| {
                    json!({
                        "persona_id": persona_id,
                        "context": extra,
                    })
                })
                .or_else(|| {
                    Some(json!({
                        "persona_id": persona_id,
                    }))
                }),
        )
        .await?;
        Ok(snapshot)
    }

    pub async fn meeting_prep(&self, persona_id: &str) -> Result<MeetingPrep, InvestigatorError> {
        meeting_prep::meeting_prep(&self.pool, persona_id).await
    }
}

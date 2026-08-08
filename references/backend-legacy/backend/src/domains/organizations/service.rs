use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::errors::ObservationStoreError;
use makosh_observations_postgres::store::ObservationStore;

use super::api::{Organization, OrganizationError, OrganizationStore, OrganizationUpdate};
use super::core::aliases::{OrgAliasStore, OrganizationAlias};
use super::core::departments::{OrgDepartment, OrgDepartmentStore};
use super::core::errors::OrgCoreError;
use super::core::identity::{OrgIdentityStore, OrganizationIdentity};
use super::core::persona_links::{OrgPersonaLink, OrgPersonaLinkStore};
use super::enrichment::{OrgEnrichmentError, OrgEnrichmentStore};
use super::health::{OrgHealthError, OrgHealthStore};

#[derive(Clone)]
pub struct OrganizationCommandService {
    pool: PgPool,
}

impl OrganizationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_organization_manual(
        &self,
        display_name: &str,
        org_type: Option<&str>,
    ) -> Result<Organization, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "display_name": display_name,
                    "org_type": org_type,
                    "action": "create",
                }),
                format!("organizations://create/{display_name}"),
                json!({
                    "captured_by": "organizations_service.create_organization_manual",
                    "operation": "create_organization_manual",
                }),
            )
            .await?;

        Ok(OrganizationStore::new(self.pool.clone())
            .create_with_observation(display_name, org_type, &observation.observation_id)
            .await?)
    }

    pub async fn update_organization_manual(
        &self,
        organization_id: &str,
        update: &OrganizationUpdate,
    ) -> Result<Organization, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "update": serde_json::to_value(update).unwrap_or(Value::Null),
                    "action": "update",
                }),
                format!("organization://{organization_id}/update"),
                json!({
                    "captured_by": "organizations_service.update_organization_manual",
                    "operation": "update_organization_manual",
                }),
            )
            .await?;

        Ok(OrganizationStore::new(self.pool.clone())
            .update_with_observation(organization_id, update, &observation.observation_id)
            .await?)
    }

    pub async fn archive_organization_manual(
        &self,
        organization_id: &str,
    ) -> Result<(), OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "action": "archive",
                }),
                format!("organization://{organization_id}/archive"),
                json!({
                    "captured_by": "organizations_service.archive_organization_manual",
                    "operation": "archive_organization_manual",
                }),
            )
            .await?;

        OrganizationStore::new(self.pool.clone())
            .archive_with_observation(organization_id, &observation.observation_id)
            .await?;
        Ok(())
    }

    pub async fn add_identity_manual(
        &self,
        organization_id: &str,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<OrganizationIdentity, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("organization://{organization_id}/identities/{identity_type}"),
                json!({
                    "captured_by": "organizations_service.add_identity_manual",
                    "operation": "add_identity_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(OrgIdentityStore::new(self.pool.clone())
            .upsert_with_observation(
                organization_id,
                identity_type,
                identity_value,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn add_alias_manual(
        &self,
        organization_id: &str,
        name: &str,
        alias_type: &str,
        requested_source: &str,
    ) -> Result<OrganizationAlias, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "name": name,
                    "alias_type": alias_type,
                    "source": requested_source,
                }),
                format!("organization://{organization_id}/aliases/{alias_type}"),
                json!({
                    "captured_by": "organizations_service.add_alias_manual",
                    "operation": "add_alias_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(OrgAliasStore::new(self.pool.clone())
            .add_with_observation(
                organization_id,
                name,
                alias_type,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn add_department_manual(
        &self,
        organization_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<OrgDepartment, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "name": name,
                    "description": description,
                    "parent_department_id": parent_id,
                }),
                format!("organization://{organization_id}/departments/{name}"),
                json!({
                    "captured_by": "organizations_service.add_department_manual",
                    "operation": "add_department_manual",
                }),
            )
            .await?;

        Ok(OrgDepartmentStore::new(self.pool.clone())
            .add_with_observation(
                organization_id,
                name,
                description,
                parent_id,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn link_persona_manual(
        &self,
        organization_id: &str,
        persona_id: &str,
        role: Option<&str>,
        department: Option<&str>,
        requested_source: &str,
    ) -> Result<OrgPersonaLink, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "persona_id": persona_id,
                    "role": role,
                    "department": department,
                    "source": requested_source,
                }),
                format!("organization://{organization_id}/persona-links/{persona_id}"),
                json!({
                    "captured_by": "organizations_service.link_persona_manual",
                    "operation": "link_persona_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(OrgPersonaLinkStore::new(self.pool.clone())
            .link_with_observation(
                organization_id,
                persona_id,
                role,
                department,
                Some(&format!("observation:{}", observation.observation_id)),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn apply_enrichment_manual(
        &self,
        organization_id: &str,
        result_id: &str,
    ) -> Result<(), OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "REVIEW_TRANSITION",
                json!({
                    "organization_id": organization_id,
                    "result_id": result_id,
                    "operation": "organization_enrichment_apply",
                }),
                format!("organization://{organization_id}/enrichment/{result_id}/apply"),
                json!({
                    "captured_by": "organizations_service.apply_enrichment_manual",
                    "operation": "apply_enrichment_manual",
                }),
            )
            .await?;

        OrgEnrichmentStore::new(self.pool.clone())
            .apply_with_observation(result_id, &observation.observation_id)
            .await?;
        Ok(())
    }

    pub async fn toggle_watchlist_manual(
        &self,
        organization_id: &str,
    ) -> Result<bool, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "action": "toggle_watchlist",
                }),
                format!("organization://{organization_id}/watchlist"),
                json!({
                    "captured_by": "organizations_service.toggle_watchlist_manual",
                    "operation": "toggle_watchlist_manual",
                }),
            )
            .await?;

        Ok(OrgHealthStore::new(self.pool.clone())
            .toggle_watchlist_with_observation(
                organization_id,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    async fn capture_manual(
        &self,
        kind: &str,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<makosh_observations_api::models::Observation, OrganizationCommandServiceError> {
        Ok(ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum OrganizationCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    Core(#[from] OrgCoreError),

    #[error(transparent)]
    Enrichment(#[from] OrgEnrichmentError),

    #[error(transparent)]
    Health(#[from] OrgHealthError),
}

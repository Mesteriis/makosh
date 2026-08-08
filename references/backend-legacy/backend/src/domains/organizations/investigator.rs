use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::core::errors::OrgCoreError;
use makosh_observations_postgres::errors::ObservationStoreError;
use serde::Serialize;
use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct OrgDossier {
    pub organization: Value,
    pub identities: Vec<Value>,
    pub domains: Vec<Value>,
    pub linked_personas: Vec<Value>,
    pub facts: Vec<Value>,
    pub memory_cards: Vec<Value>,
    pub timeline: Vec<Value>,
    pub contracts: Vec<Value>,
    pub risks: Vec<Value>,
    pub portals: Vec<Value>,
    pub procedures: Vec<Value>,
    pub enrichment: Vec<Value>,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgBrief {
    pub organization_id: String,
    pub display_name: String,
    pub org_type: Option<String>,
    pub last_interaction_days: Option<i64>,
    pub open_risks: i64,
    pub active_contracts: i64,
    pub primary_persona: Option<String>,
    pub language: Option<String>,
    pub next_deadline: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrgContextPack {
    pub brief: OrgBrief,
    pub recent_events: Vec<Value>,
    pub key_personas: Vec<Value>,
    pub active_contracts: Vec<Value>,
    pub open_risks: Vec<Value>,
    pub portals: Vec<Value>,
    pub procedures: Vec<Value>,
}

#[derive(Clone)]
pub struct OrganizationInvestigator {
    pool: PgPool,
}

impl OrganizationInvestigator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn dossier(&self, org_id: &str) -> Result<OrgDossier, InvestigatorError> {
        use crate::domains::organizations::api::OrganizationStore;
        let org = OrganizationStore::new(self.pool.clone());
        let org_data = org.get(org_id).await?.ok_or(InvestigatorError::NotFound)?;
        let org_json = serde_json::to_value(&org_data).unwrap_or_default();

        let mut parts = Vec::new();
        if let Some(t) = &org_data.org_type {
            parts.push(format!("Type: {t}"));
        }
        if org_data.interaction_count > 0 {
            parts.push(format!("{} interactions", org_data.interaction_count));
        }

        Ok(OrgDossier {
            organization: org_json,
            identities: vec![],
            domains: vec![],
            linked_personas: vec![],
            facts: vec![],
            memory_cards: vec![],
            timeline: vec![],
            contracts: vec![],
            risks: vec![],
            portals: vec![],
            procedures: vec![],
            enrichment: vec![],
            summary: parts.join(" | "),
        })
    }

    pub async fn brief(&self, org_id: &str) -> Result<OrgBrief, InvestigatorError> {
        use crate::domains::organizations::api::OrganizationStore;
        let org = OrganizationStore::new(self.pool.clone());
        let org_data = org.get(org_id).await?.ok_or(InvestigatorError::NotFound)?;
        let last_days = org_data
            .last_interaction_at
            .map(|dt| (chrono::Utc::now() - dt).num_days());
        Ok(OrgBrief {
            organization_id: org_data.organization_id,
            display_name: org_data.display_name,
            org_type: org_data.org_type,
            last_interaction_days: last_days,
            open_risks: 0,
            active_contracts: 0,
            primary_persona: None,
            language: org_data.primary_language,
            next_deadline: None,
        })
    }

    pub async fn context_pack(&self, org_id: &str) -> Result<OrgContextPack, InvestigatorError> {
        let brief = self.brief(org_id).await?;
        Ok(OrgContextPack {
            brief,
            recent_events: vec![],
            key_personas: vec![],
            active_contracts: vec![],
            open_risks: vec![],
            portals: vec![],
            procedures: vec![],
        })
    }
}

impl From<OrganizationError> for InvestigatorError {
    fn from(e: OrganizationError) -> Self {
        match e {
            OrganizationError::NotFound => InvestigatorError::NotFound,
            OrganizationError::Validation(message) => InvestigatorError::Validation(message),
            OrganizationError::Sqlx(e) => InvestigatorError::Sqlx(e),
            OrganizationError::Core(e) => InvestigatorError::Core(e),
            OrganizationError::Observation(e) => InvestigatorError::Observation(e),
        }
    }
}

#[derive(Debug, Error)]
pub enum InvestigatorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Core(#[from] OrgCoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("{0}")]
    Validation(String),
    #[error("organization not found")]
    NotFound,
}

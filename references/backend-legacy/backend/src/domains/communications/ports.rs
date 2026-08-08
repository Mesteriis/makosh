use sqlx::PgPool;

use super::ai_state::{
    CommunicationAiStateError, CommunicationAiStateRecord, CommunicationAiStateStore,
    CommunicationAiStateTransitionRequest,
};
use super::messages::models::ProjectedMessage;
use super::spam_reputation::{
    SenderReputationDecision, SenderReputationError, SenderReputationStore,
};
use makosh_communications_api::evidence::{
    NewRawCommunicationRecord, StoredRawCommunicationRecord,
};
use makosh_communications_postgres::errors::CommunicationIngestionError;
use makosh_communications_postgres::store::CommunicationIngestionStore;

#[derive(Clone)]
pub struct CommunicationRawEvidencePort(CommunicationIngestionStore);

impl CommunicationRawEvidencePort {
    pub fn new(pool: PgPool) -> Self {
        Self(CommunicationIngestionStore::new(pool))
    }

    pub async fn record_raw_source(
        &self,
        record: &NewRawCommunicationRecord,
    ) -> Result<StoredRawCommunicationRecord, CommunicationIngestionError> {
        self.0.record_raw_source(record).await
    }
}

#[derive(Clone)]
pub struct CommunicationAiStatePort(CommunicationAiStateStore);

impl CommunicationAiStatePort {
    pub fn new(pool: PgPool) -> Self {
        Self(CommunicationAiStateStore::new(pool))
    }

    pub async fn recover_expired_mail_processing(
        &self,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize, CommunicationAiStateError> {
        self.0.recover_expired_mail_processing(now).await
    }

    pub async fn claim_due_mail_messages(
        &self,
        limit: i64,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<String>, CommunicationAiStateError> {
        self.0.claim_due_mail_messages(limit, now).await
    }

    pub async fn record_mail_processing_failure(
        &self,
        message_id: &str,
        error: &str,
        retryable: bool,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        self.0
            .record_mail_processing_failure(message_id, error, retryable, now)
            .await
    }

    pub async fn transition(
        &self,
        message_id: &str,
        request: CommunicationAiStateTransitionRequest,
    ) -> Result<Option<CommunicationAiStateRecord>, CommunicationAiStateError> {
        self.0.transition(message_id, request).await
    }
}

#[derive(Clone)]
pub struct SenderReputationPort(SenderReputationStore);

impl SenderReputationPort {
    pub fn new(pool: PgPool) -> Self {
        Self(SenderReputationStore::new(pool))
    }

    pub async fn evaluate_message(
        &self,
        message: &ProjectedMessage,
    ) -> Result<SenderReputationDecision, SenderReputationError> {
        self.0.evaluate_message(message).await
    }

    pub async fn record_suppressed_message(
        &self,
        message: &ProjectedMessage,
        reason: &str,
    ) -> Result<(), SenderReputationError> {
        self.0.record_suppressed_message(message, reason).await
    }

    pub async fn record_analysis(
        &self,
        message: &ProjectedMessage,
        classification: super::spam_reputation::SenderReputationClassification,
        reason: &str,
    ) -> Result<super::spam_reputation::SenderReputationRecord, SenderReputationError> {
        self.0
            .record_analysis(message, classification, reason)
            .await
    }
}

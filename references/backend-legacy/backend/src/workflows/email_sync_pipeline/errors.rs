use thiserror::Error;

use crate::application::relationship_graph::RelationshipGraphCoordinatorError;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::provider_observation_projection::CommunicationSignalProjectionError;
use crate::domains::communications::storage::errors::{
    AttachmentSafetyScanError, CommunicationStorageError,
};
use crate::domains::decisions::ports::DecisionReviewPortError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::organizations::core::errors::OrgCoreError;
use crate::domains::personas::memory::errors::PersonaMemoryError;
use crate::domains::signal_hub::store::SignalHubError;
use crate::domains::tasks::candidates::errors::TaskCandidateError;
use crate::workflows::review_inbox::ReviewInboxWorkflowError;
use makosh_communications_api::evidence::CommunicationEvidencePortError;

#[derive(Debug, Error)]
pub enum EmailSyncRecordError {
    #[error("email sync record field must not be empty: {0}")]
    EmptyField(&'static str),

    #[error(transparent)]
    CommunicationEvidence(#[from] CommunicationEvidencePortError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error("email sync payload must be a JSON object before raw blob projection")]
    InvalidRawPayloadObject,

    #[error("email sync payload missing provider raw field: {field}")]
    MissingRawPayloadField { field: &'static str },

    #[error("email sync payload field {field} is invalid base64: {source}")]
    InvalidRawPayloadBase64 {
        field: &'static str,
        #[source]
        source: base64::DecodeError,
    },

    #[error("email sync does not support provider kind: {0}")]
    UnsupportedProviderKind(String),
}

#[derive(Debug, Error)]
pub enum EmailSyncPipelineError {
    #[error(transparent)]
    Sync(#[from] EmailSyncRecordError),

    #[error(transparent)]
    Message(#[from] MessageProjectionError),

    #[error(transparent)]
    SignalHub(#[from] SignalHubError),

    #[error(transparent)]
    SignalProjection(#[from] CommunicationSignalProjectionError),

    #[error(transparent)]
    PersonaMemory(#[from] PersonaMemoryError),

    #[error(transparent)]
    CommunicationStorage(#[from] CommunicationStorageError),

    #[error(transparent)]
    AttachmentScan(#[from] AttachmentSafetyScanError),

    #[error(transparent)]
    Decision(#[from] DecisionReviewPortError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    OrganizationCore(#[from] OrgCoreError),

    #[error(transparent)]
    RelationshipGraph(#[from] RelationshipGraphCoordinatorError),

    #[error(transparent)]
    TaskCandidate(#[from] TaskCandidateError),

    #[error(transparent)]
    ReviewInboxWorkflow(#[from] ReviewInboxWorkflowError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid email participant address: {0}")]
    InvalidParticipantEmail(String),
}

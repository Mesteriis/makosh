use makosh_events_api::EventEnvelopeError;
use std::io;

use crate::ai::control_center::errors::AiControlCenterError;
use crate::ai::core::errors::AiError;
use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::storage::errors::CommunicationStorageError;
use crate::domains::decisions::errors::DecisionStoreError;
use crate::domains::documents::processing::errors::DocumentProcessingError;
use crate::domains::obligations::errors::ObligationStoreError;
use crate::domains::personas::api::errors::PersonaProjectionError;
use crate::domains::personas::identity::errors::PersonaIdentityError;
use crate::domains::projects::core::errors::ProjectStoreError;
use crate::domains::projects::link_reviews::errors::ProjectLinkReviewError;
use crate::domains::relationships::errors::RelationshipStoreError;
use crate::domains::review::errors::ReviewInboxError;
use crate::domains::signal_hub::store::SignalHubError;
use crate::domains::tasks::candidates::errors::TaskCandidateError;
use crate::engines::automation::errors::AutomationError;
use crate::engines::consistency::errors::ConsistencyError;
use crate::integrations::mail::accounts::errors::EmailAccountSetupError;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;
use crate::integrations::zoom::client::errors::ZoomError;
use crate::platform::audit::errors::ApiAuditError;
use crate::platform::calls::errors::CallError;
use crate::platform::settings::errors::SettingsError;
use crate::platform::storage::errors::StorageError;
use crate::vault::errors::HostVaultError;
use crate::workflows::review_promotion::ReviewPromotionError;
use makosh_communications_postgres::errors::CommunicationIngestionError;
use makosh_events_postgres::errors::EventStoreError;

#[derive(Debug)]
pub enum ApiError {
    DatabaseNotConfigured,
    InvalidEnvelope(EventEnvelopeError),
    Audit(ApiAuditError),
    Store(EventStoreError),
    Graph(crate::domains::graph::core::errors::GraphStoreError),
    InvalidGraphQuery(&'static str),
    InvalidPersonaQuery(&'static str),
    Projects(ProjectStoreError),
    InvalidProjectQuery(&'static str),
    InvalidProjectLinkReview(&'static str),
    InvalidTaskCandidateQuery(&'static str),
    InvalidTaskCandidateReview(&'static str),
    InvalidTaskQuery(&'static str),
    InvalidObligationQuery(&'static str),
    InvalidObligationReview(&'static str),
    InvalidDecisionQuery(&'static str),
    InvalidDecisionReview(&'static str),
    InvalidRelationshipQuery(&'static str),
    InvalidRelationshipReview(&'static str),
    InvalidContradictionQuery(&'static str),
    InvalidContradictionReview(&'static str),
    InvalidReviewQuery(&'static str),
    InvalidReviewItem(&'static str),
    FailedPrecondition(String),
    InvalidPersonaIdentityReview(&'static str),
    InvalidDocumentProcessingQuery(&'static str),
    Settings(SettingsError),
    SignalHub(SignalHubError),
    SettingNotFound,
    DocumentProcessing(DocumentProcessingError),
    TaskCandidateNotFound,
    TaskCandidate(TaskCandidateError),
    ObligationNotFound,
    Obligation(ObligationStoreError),
    DecisionNotFound,
    Decision(DecisionStoreError),
    RelationshipNotFound,
    Relationship(RelationshipStoreError),
    ContradictionObservationNotFound,
    ReviewItemNotFound,
    ReviewInbox(ReviewInboxError),
    ReviewPromotion(ReviewPromotionError),
    Consistency(ConsistencyError),
    AiRunNotFound,
    Ai(AiError),
    AiControlCenter(AiControlCenterError),
    Telegram(TelegramError),
    WhatsappWeb(WhatsappWebError),
    Zoom(ZoomError),
    YandexTelemost(YandexTelemostError),
    Automation(AutomationError),
    Call(CallError),
    ProjectLinkTargetNotFound,
    ProjectLinkReview(ProjectLinkReviewError),
    PersonaIdentityNotFound,
    PersonaProjection(PersonaProjectionError),
    PersonaIdentity(PersonaIdentityError),
    Messages(MessageProjectionError),
    CommunicationIngestion(CommunicationIngestionError),
    CommunicationStorage(CommunicationStorageError),
    InvalidCommunicationQuery(&'static str),
    ProviderWriteConfirmationRequired,
    CommunicationMessageNotFound,
    SecretVaultNotConfigured,
    HostVault(HostVaultError),
    AccountSetup(EmailAccountSetupError),
    AccountSetupState,
    AccountSetupPendingGrantNotFound,
    AccountSetupStateMismatch,
    GraphNotFound,
    ProjectNotFound,
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Io(#[from] io::Error),
}

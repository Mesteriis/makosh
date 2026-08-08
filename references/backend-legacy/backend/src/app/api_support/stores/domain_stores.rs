use super::super::*;
use super::database::database_pool;
use crate::domains::communications::storage::blob_store::LocalCommunicationBlobStore;
use crate::platform::audit::store::ApiAuditLog;
use crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT;
use sqlx::PgPool;

pub(crate) trait AppStoreFactory: Sized {
    fn from_pool(pool: PgPool) -> Self;
}

pub(crate) fn app_store<S: AppStoreFactory>(pool: PgPool) -> S {
    S::from_pool(pool)
}

macro_rules! impl_app_store_factory {
    ($($store:path),+ $(,)?) => {
        $(
            impl AppStoreFactory for $store {
                fn from_pool(pool: PgPool) -> Self {
                    <$store>::new(pool)
                }
            }
        )+
    };
}

impl_app_store_factory!(
    crate::domains::calendar::core::agendas::EventAgendaStore,
    crate::domains::calendar::core::checklists::EventChecklistStore,
    crate::domains::calendar::core::context_packs::EventContextPackStore,
    crate::domains::calendar::core::participants::EventParticipantStore,
    crate::domains::calendar::core::relations::EventRelationStore,
    crate::domains::calendar::events::account_store::CalendarAccountStore,
    crate::domains::calendar::events::event_store::CalendarEventStore,
    crate::domains::calendar::events::source_store::CalendarSourceStore,
    crate::domains::calendar::meetings::recordings::EventRecordingStore,
    crate::domains::calendar::meetings::transcripts::EventTranscriptStore,
    crate::domains::calendar::meetings::notes::MeetingNoteStore,
    crate::domains::calendar::meetings::outcomes::MeetingOutcomeStore,
    crate::domains::calendar::reminders::CalendarReminderStore,
    crate::domains::calendar::rules::CalendarRuleStore,
    crate::domains::calendar::scheduling::DeadlineStore,
    crate::domains::calendar::scheduling::FocusBlockStore,
    crate::domains::communications::ai_state::CommunicationAiStateStore,
    crate::domains::communications::analytics::EmailAnalyticsStore,
    crate::domains::communications::attachment_dedup::AttachmentDedupStore,
    crate::domains::communications::attachment_search::AttachmentSearchStore,
    crate::domains::communications::bulk_actions::BulkMessageActionStore,
    makosh_communications_postgres::provider_store::CommunicationProviderAccountStore,
    makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore,
    crate::domains::communications::delivery_notifications::CommunicationDeliveryNotificationStore,
    crate::domains::communications::drafts::CommunicationDraftStore,
    crate::domains::communications::finance::CommunicationFinanceStore,
    crate::domains::communications::folders::CommunicationFolderStore,
    crate::domains::communications::legal::LegalDocumentStore,
    crate::domains::communications::messages::store::MessageProjectionStore,
    crate::domains::communications::outbox::CommunicationOutboxStore,
    crate::domains::communications::personas::CommunicationPersonaStore,
    crate::domains::communications::provider_resources::MailProviderResourceStore,
    crate::domains::communications::read_receipts::CommunicationReadReceiptStore,
    crate::domains::communications::saved_searches::CommunicationSavedSearchStore,
    crate::domains::communications::signatures::store::CertificateStore,
    crate::domains::communications::subscriptions::SubscriptionStore,
    crate::domains::communications::templates::CommunicationTemplateStore,
    crate::domains::communications::threads::CommunicationThreadStore,
    crate::domains::documents::processing::store::DocumentProcessingStore,
    crate::domains::organizations::api::OrganizationStore,
    crate::domains::organizations::core::aliases::OrgAliasStore,
    crate::domains::organizations::core::persona_links::OrgPersonaLinkStore,
    crate::domains::organizations::core::departments::OrgDepartmentStore,
    crate::domains::organizations::core::domains::OrgDomainStore,
    crate::domains::organizations::core::identity::OrgIdentityStore,
    crate::domains::organizations::core::related::RelatedOrgStore,
    crate::domains::organizations::enrichment::OrgEnrichmentStore,
    crate::domains::organizations::finance::OrgComplianceStore,
    crate::domains::organizations::finance::OrgContractStore,
    crate::domains::organizations::finance::OrgFinancialStore,
    crate::domains::organizations::finance::OrgProductStore,
    crate::domains::organizations::finance::OrgServiceStore,
    crate::domains::organizations::health::OrgHealthStore,
    crate::domains::organizations::health::OrgRiskStore,
    crate::domains::organizations::workflows::playbooks::OrgPlaybookStore,
    crate::domains::organizations::workflows::portals::OrgPortalStore,
    crate::domains::organizations::workflows::procedures::OrgProcedureStore,
    crate::domains::organizations::workflows::templates::OrgTemplateStore,
    crate::domains::organizations::workflows::timeline::OrgTimelineStore,
    crate::domains::personas::api::store::PersonaProjectionStore,
    crate::domains::personas::core::interaction_contexts::PersonaInteractionContextStore,
    crate::domains::personas::core::roles::PersonaRoleStore,
    crate::domains::personas::core::identities::PersonaIdentityStore,
    crate::domains::personas::enrichment::store::PersonaEnrichmentStore,
    crate::domains::personas::enrichment_engine::EnrichmentResultStore,
    crate::domains::personas::expertise::PersonaExpertiseStore,
    crate::domains::personas::health::PersonaHealthStore,
    crate::domains::personas::memory::facts::PersonaFactStore,
    crate::domains::personas::memory::cards::PersonaMemoryCardStore,
    crate::domains::personas::memory::preferences::PersonaPreferenceStore,
    crate::domains::personas::memory::snapshots::PersonaSnapshotStore,
    crate::domains::personas::memory::relationship_events::RelationshipEventStore,
    crate::domains::personas::trust::promises::PersonaPromiseStore,
    crate::domains::personas::trust::risks::PersonaRiskStore,
    crate::domains::review::store::ReviewInboxStore,
    crate::domains::tasks::api::TaskStore,
    crate::domains::tasks::core::external_identities::ExternalTaskIdentityStore,
    crate::domains::tasks::core::checklists::TaskChecklistStore,
    crate::domains::tasks::core::context_packs::TaskContextPackStore,
    crate::domains::tasks::core::evidence::TaskEvidenceStore,
    crate::domains::tasks::core::provider_store::TaskProviderStore,
    crate::domains::tasks::core::relations::TaskRelationStore,
    crate::domains::tasks::core::subtasks::TaskSubtaskStore,
    crate::domains::tasks::rules::TaskRuleStore,
    crate::domains::tasks::rules::TaskTemplateStore,
    crate::engines::consistency::store::ContradictionObservationStore,
    makosh_events_postgres::store::EventStore,
    makosh_observations_postgres::store::ObservationStore,
    crate::workflows::mail_background_sync::store::MailSyncStore,
);

pub(crate) fn communication_blob_store() -> LocalCommunicationBlobStore {
    LocalCommunicationBlobStore::new(DEFAULT_MAIL_SYNC_BLOB_ROOT)
}

pub(crate) fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    Ok(EventStore::new(database_pool(state)?))
}

pub(crate) fn message_store(state: &AppState) -> Result<MessageProjectionStore, ApiError> {
    Ok(MessageProjectionStore::new(database_pool(state)?))
}

pub(crate) fn communication_storage_store(
    state: &AppState,
) -> Result<CommunicationStorageStore, ApiError> {
    Ok(CommunicationStorageStore::new(database_pool(state)?))
}

pub(crate) fn communication_ingestion_store(
    state: &AppState,
) -> Result<CommunicationIngestionStore, ApiError> {
    Ok(CommunicationIngestionStore::new(database_pool(state)?))
}

pub(crate) fn communication_provider_account_store(
    state: &AppState,
) -> Result<
    makosh_communications_postgres::provider_store::CommunicationProviderAccountStore,
    ApiError,
> {
    Ok(
        makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            database_pool(state)?,
        ),
    )
}

pub(crate) fn communication_provider_secret_binding_store(
    state: &AppState,
) -> Result<
    makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore,
    ApiError,
> {
    Ok(
        makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
            database_pool(state)?,
        ),
    )
}

pub(crate) fn project_link_review_store(
    state: &AppState,
) -> Result<ProjectLinkReviewStore, ApiError> {
    Ok(ProjectLinkReviewStore::new(database_pool(state)?))
}

pub(crate) fn task_candidate_store(state: &AppState) -> Result<TaskCandidateStore, ApiError> {
    Ok(TaskCandidateStore::new(database_pool(state)?))
}

pub(crate) fn document_processing_store(
    state: &AppState,
) -> Result<DocumentProcessingStore, ApiError> {
    Ok(DocumentProcessingStore::new(database_pool(state)?))
}

pub(crate) fn persona_identity_review_store(
    state: &AppState,
) -> Result<PersonaIdentityReviewStore, ApiError> {
    Ok(PersonaIdentityReviewStore::new(database_pool(state)?))
}

pub(crate) fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    Ok(ApiAuditLog::new(database_pool(state)?))
}

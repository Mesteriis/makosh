use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_events_api::NewEventEnvelope;
use makosh_events_postgres::store::EventStore;
use makosh_hub_backend::application::review_transitions::{
    TaskCandidateReviewApplicationError, TaskCandidateReviewApplicationService,
};
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::documents::core::models::NewDocumentImport;
use makosh_hub_backend::domains::documents::core::store::DocumentImportStore;
use makosh_hub_backend::domains::tasks::candidates::models::{
    TaskCandidateReviewCommand, TaskCandidateReviewCommandResult, TaskCandidateReviewState,
};
use makosh_hub_backend::domains::tasks::candidates::store::TaskCandidateStore;

use makosh_hub_backend::platform::storage::database::Database;
use serde_json::json;
use sqlx::postgres::PgPool;

const TASK_CANDIDATE_REVIEW_EVENT_TYPE: &str = "task_candidate.review_state_changed";

pub(crate) struct TaskCandidateTestContext {
    pub(crate) pool: PgPool,
    pub(crate) store: TaskCandidateStore,
    pub(crate) event_store: EventStore,
}

impl TaskCandidateTestContext {
    pub(crate) async fn review(
        &self,
        command: &TaskCandidateReviewCommand,
    ) -> Result<TaskCandidateReviewCommandResult, TaskCandidateReviewApplicationError> {
        TaskCandidateReviewApplicationService::new(self.pool.clone())
            .review_manual(command)
            .await
    }
}

pub(crate) async fn live_task_candidate_context() -> Option<TaskCandidateTestContext> {
    let test_context = Box::leak(Box::new(TestContext::new().await));
    let database_url = test_context.connection_string();
    task_candidate_context(&database_url).await
}

async fn task_candidate_context(database_url: &str) -> Option<TaskCandidateTestContext> {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(TaskCandidateTestContext {
        store: TaskCandidateStore::new(pool.clone()),
        event_store: EventStore::new(pool.clone()),
        pool,
    })
}

pub(crate) async fn seed_message(
    context: &TaskCandidateTestContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_task_candidate_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(context.pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Task Candidate Gmail",
            format!("task-candidate-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_task_candidate_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:task-candidate:{suffix}:{provider_record_id}"),
                format!("batch-task-candidate-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"task_candidate_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(context.pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

pub(crate) async fn seed_document(
    pool: &PgPool,
    document_id: &str,
    title: &str,
    body: &str,
) -> String {
    let import = NewDocumentImport::markdown(document_id, title, body);
    DocumentImportStore::new(pool.clone())
        .import_document(&import)
        .await
        .expect("document import");
    document_id.to_owned()
}

pub(crate) fn build_review_event(
    task_candidate_id: &str,
    review_state: TaskCandidateReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        format!("task_candidate_review:{command_id}"),
        TASK_CANDIDATE_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "task_candidate_review",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({"kind": "task_candidate_review"}),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "task_candidate_id": task_candidate_id,
        "review_state": review_state.as_str(),
    }))
    .build()
    .expect("review event")
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

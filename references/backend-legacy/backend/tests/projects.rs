use makosh_backend_testkit::context::TestContext;
use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::evidence::NewRawCommunicationRecord;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::messages::projection::project_raw_email_message;
use makosh_hub_backend::domains::communications::messages::store::MessageProjectionStore;
use makosh_hub_backend::domains::documents::core::{
    models::NewDocumentImport, store::DocumentImportStore,
};
use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::projects::core::{models::NewProject, store::ProjectStore};
use makosh_hub_backend::domains::projects::link_reviews::{
    models::{ProjectLinkReviewCommand, ProjectLinkReviewState, ProjectLinkTargetKind},
    store::ProjectLinkReviewStore,
};

use makosh_hub_backend::platform::storage::database::Database;

#[tokio::test]
async fn project_detail_links_keyword_messages_documents_and_personas_against_postgres() {
    let Some(context) = live_project_context("project detail").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectMemory{suffix}");
    let project_id = format!("project:v1:test:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Memory {suffix}"),
                "Product Development",
                "Source-backed project memory test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(42),
        )
        .await
        .expect("upsert project");
    context
        .person_store
        .upsert_email_persona(&format!("owner-{suffix}@example.com"))
        .await
        .expect("upsert owner person");

    seed_message(
        &context,
        suffix,
        &format!("owner-{suffix}@example.com"),
        &[format!("reviewer-{suffix}@example.com")],
        &format!("provider-project-memory-{suffix}"),
        &format!("{keyword} planning thread"),
        "Project body",
    )
    .await;
    seed_message(
        &context,
        suffix,
        &format!("other-{suffix}@example.com"),
        &[format!("noise-{suffix}@example.com")],
        &format!("provider-project-memory-noise-{suffix}"),
        "Unrelated thread",
        "No matching project keyword",
    )
    .await;
    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_project_memory_{suffix}"),
            format!("{keyword} architecture.md"),
            "# Project Architecture\n\nSource-backed document.",
        ))
        .await
        .expect("import project document");

    let detail = context
        .project_store
        .project_detail(&project_id)
        .await
        .expect("project detail")
        .expect("project exists");

    assert_eq!(detail.project.project_id, project_id);
    assert_eq!(detail.project.progress_percent, 42);
    assert_eq!(detail.stats.message_count, 1);
    assert_eq!(detail.stats.document_count, 1);
    assert_eq!(detail.stats.persona_count, 2);
    #[allow(deprecated)]
    {
        assert_eq!(detail.stats.people_count, 2);
    }
    assert_eq!(detail.recent_messages.len(), 1);
    assert_eq!(
        detail.recent_messages[0].subject,
        format!("{keyword} planning thread")
    );
    assert_eq!(detail.documents.len(), 1);
    assert_eq!(
        detail.documents[0].title,
        format!("{keyword} architecture.md")
    );
    assert_eq!(detail.timeline.len(), 2);
    assert!(
        detail
            .key_personas
            .iter()
            .any(|person| person.email_address == format!("owner-{suffix}@example.com"))
    );
    #[allow(deprecated)]
    {
        assert_eq!(detail.key_people, detail.key_personas);
    }

    cleanup_project(&context.pool, &project_id).await;
}

#[tokio::test]
async fn project_detail_excludes_rejected_keyword_message_against_postgres() {
    let Some(context) = live_project_context("project detail excludes rejected keyword").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectRejected{suffix}");
    let project_id = format!("project:v1:reject:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Rejected Project {suffix}"),
                "Product Development",
                "Reject keyword matches",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(33),
        )
        .await
        .expect("upsert rejected project");

    let message_id = seed_message(
        &context,
        suffix,
        &format!("owner-reject-{suffix}@example.com"),
        &[format!("reviewer-reject-{suffix}@example.com")],
        &format!("provider-project-reject-{suffix}"),
        &format!("{keyword} kickoff"),
        "This keyword message should be excluded",
    )
    .await;

    context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("project-reject-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id,
            review_state: ProjectLinkReviewState::UserRejected,
            actor_id: "project-reviewer".to_owned(),
        })
        .await
        .expect("set rejected review");

    let detail = context
        .project_store
        .project_detail(&project_id)
        .await
        .expect("project detail");
    assert!(detail.is_some(), "project exists");
    let detail = detail.expect("project detail");

    assert_eq!(detail.stats.message_count, 0);
    assert_eq!(detail.recent_messages.len(), 0);
    assert_eq!(detail.timeline.len(), 0);

    cleanup_project(&context.pool, &project_id).await;
}

#[tokio::test]
async fn project_detail_includes_confirmed_non_keyword_message_against_postgres() {
    let Some(context) = live_project_context("project detail includes confirmed non keyword").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectConfirmKeyword{suffix}");
    let non_keyword_subject = format!("Non keyword subject {suffix}");
    let project_id = format!("project:v1:confirm:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Confirmed Project {suffix}"),
                "Product Development",
                "Confirm non-matching message",
                "Alex Morgan",
                vec![keyword],
            )
            .progress(44),
        )
        .await
        .expect("upsert confirmed project");

    let message_id = seed_message(
        &context,
        suffix,
        &format!("owner-confirm-{suffix}@example.com"),
        &[format!("reviewer-confirm-{suffix}@example.com")],
        &format!("provider-project-confirm-{suffix}"),
        &non_keyword_subject,
        "This message does not contain the project keyword",
    )
    .await;

    context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("project-confirm-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id,
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: "project-reviewer".to_owned(),
        })
        .await
        .expect("set confirmed review");

    let detail = context
        .project_store
        .project_detail(&project_id)
        .await
        .expect("project detail");
    assert!(detail.is_some(), "project exists");
    let detail = detail.expect("project detail");

    assert_eq!(detail.stats.message_count, 1);
    assert_eq!(detail.recent_messages.len(), 1);
    assert_eq!(detail.recent_messages[0].subject, non_keyword_subject);

    cleanup_project(&context.pool, &project_id).await;
}

struct LiveProjectContext {
    pool: PgPool,
    person_store: PersonaProjectionStore,
    communication_store: CommunicationIngestionStore,
    document_store: DocumentImportStore,
    message_store: MessageProjectionStore,
    project_store: ProjectStore,
    review_store: ProjectLinkReviewStore,
}

async fn live_project_context(_test_name: &str) -> Option<LiveProjectContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(LiveProjectContext {
        pool: pool.clone(),
        person_store: PersonaProjectionStore::new(pool.clone()),
        communication_store: CommunicationIngestionStore::new(pool.clone()),
        document_store: DocumentImportStore::new(pool.clone()),
        message_store: MessageProjectionStore::new(pool.clone()),
        project_store: ProjectStore::new(pool.clone()),
        review_store: ProjectLinkReviewStore::new(pool.clone()),
    })
}

async fn cleanup_project(pool: &PgPool, project_id: &str) {
    sqlx::query("DELETE FROM projects WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .expect("cleanup project test project");
}

async fn seed_message(
    context: &LiveProjectContext,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_project_memory_{suffix}");
    context
        .communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Project Memory Gmail",
            format!("project-memory-{suffix}@example.com"),
        ))
        .await
        .expect("store project provider account");

    let raw_record_id = format!("raw_project_memory_{suffix}_{provider_record_id}");
    let raw = context
        .communication_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:project-memory:{suffix}:{provider_record_id}"),
                format!("batch_project_memory_{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"projects_test"})),
        )
        .await
        .expect("record project raw message");

    let message = project_raw_email_message(&context.message_store, &raw)
        .await
        .expect("project raw message");

    message.message_id
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

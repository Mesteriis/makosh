# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `087-test-backend-part-010`
- Group / Группа: `backend`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/backend-tests.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/tests/project_link_reviews.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/project_link_reviews.rs`
- Size bytes / Размер в байтах: `23852`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use serde_json::json;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::decisions::{
    DecisionEntityKind, DecisionReviewState, DecisionStore,
};
use makosh_hub_backend::domains::projects::core::{NewProject, ProjectStore};
use makosh_hub_backend::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewCommandResult, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use makosh_hub_backend::domains::relationships::{
    RelationshipEntityKind, RelationshipReviewState, RelationshipStore,
};
use makosh_hub_backend::platform::events::{
    EventConsumerConfig, EventConsumerRunner, EventStore, NewEventEnvelope,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::project_link_review_effects::{
    PROJECT_LINK_REVIEW_EFFECTS_CONSUMER, project_link_review_effect_event,
};

const PROJECT_LINK_REVIEW_EVENT_TYPE: &str = "project.link_review_state_changed";

#[tokio::test]
async fn project_link_review_command_appends_event_and_updates_review_against_postgres() {
    let Some(context) = live_review_context("project link review command").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectLinkReview{suffix}");
    let project_id = format!("project:v1:review:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Link Review {suffix}"),
                "Product Development",
                "Project link review event test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(63),
        )
        .await
        .expect("upsert review project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-{suffix}"),
        &format!("{keyword} kickoff"),
        "Review review candidate",
    )
    .await;
    let command_id = format!("link-review-confirm-{suffix}");
    let command = ProjectLinkReviewCommand {
        command_id: command_id.clone(),
        project_id: project_id.clone(),
        target_kind: ProjectLinkTargetKind::Message,
        target_id: message_id.clone(),
        review_state: ProjectLinkReviewState::UserConfirmed,
        actor_id: "reviewer".to_owned(),
    };
    let result = context
        .review_store
        .set_review_state(&command)
        .await
        .expect("set review state");

    assert_eq!(
        result,
        ProjectLinkReviewCommandResult {
            project_id,
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id,
            review_state: ProjectLinkReviewState::UserConfirmed,
            event_id: format!("project_link_review:{command_id}"),
        }
    );

    let review = context
        .review_store
        .explicit_review(&result.project_id, result.target_kind, &result.target_id)
        .await
        .expect("load review row")
        .expect("review exists");
    assert_eq!(review.review_state, ProjectLinkReviewState::UserConfirmed);
    assert_eq!(review.event_id, result.event_id);
}

#[tokio::test]
async fn project_link_review_confirm_materializes_user_confirmed_decision_against_postgres() {
    let Some(context) = live_review_context("project link review decision adapter").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectDecisionAdapter{suffix}");
    let project_id = format!("project:v1:review-decision:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Review Decision {suffix}"),
                "Product Development",
                "Project link review decision adapter test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(64),
        )
        .await
        .expect("upsert review decision project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("decision-reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-decision-{suffix}"),
        &format!("{keyword} proposal"),
        "Review decision adapter body",
    )
    .await;
    let command_id = format!("link-review-decision-confirm-{suffix}");
    let result = context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: command_id.clone(),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("confirm project link review");
    run_project_link_review_effects(&context).await;

    let decisions = context
        .decision_store
        .list_for_entity(DecisionEntityKind::Project, &project_id, 20)
        .await
        .expect("project decisions");
    let decision = decisions
        .iter()
        .find(|item| item.metadata["project_link_review_event_id"] == json!(result.event_id))
        .expect("confirmed project link review should create a durable Decision");

    assert_eq!(decision.review_state, DecisionReviewState::UserConfirmed);
    assert_eq!(
        decision.rationale,
        "User confirmed a message link candidate for this project."
    );
    assert_eq!(decision.metadata["project_id"], json!(project_id));
    assert_eq!(decision.metadata["target_kind"], json!("message"));
    assert_eq!(decision.metadata["target_id"], json!(message_id));

    let impacted_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM decision_impacted_entities
        WHERE decision_id = $1
          AND (
            (entity_kind = 'project' AND entity_id = $2)
            OR (entity_kind = 'communication' AND entity_id = $3)
          )
        "#,
    )
    .bind(&decision.decision_id)
    .bind(&project_id)
    .bind(&message_id)
    .fetch_one(&context.pool)
    .await
    .expect("decision impacted entities");
    assert_eq!(impacted_count, 2);

    let evidence: (String, String, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, quote FROM decision_evidence WHERE decision_id = $1",
    )
    .bind(&decision.decision_id)
    .fetch_one(&context.pool)
    .await
    .expect("decision evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(
        evidence.2.as_deref(),
        Some("User confirmed message link to project.")
    );

    let observation_kind: String = sqlx::query_scalar(
        "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
    )
    .bind(&evidence.1)
    .fetch_one(&context.pool)
    .await
    .expect("project link review decision observation kind");
    assert_eq!(observation_kind, "PROJECT_LINK_REVIEW");
}

#[tokio::test]
async fn project_link_review_confirm_materializes_relationship_against_postgres() {
    let Some(context) = live_review_context("project link review relationship adapter").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("ProjectRelationshipAdapter{suffix}");
    let project_id = format!("project:v1:review-relationship:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Project Review Relationship {suffix}"),
                "Product Development",
                "Project link review relationship adapter test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(64),
        )
        .await
        .expect("upsert review relationship project");
    let message_id = seed_message(
        &context,
        suffix,
        &format!("relationship-reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-relationship-{suffix}"),
        &format!("{keyword} proposal"),
        "Review relationship adapter body",
    )
    .await;
    let command_id = format!("link-review-relationship-confirm-{suffix}");
    let _result = context
        .review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: command_id.clone(),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: message_id.clone(),
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: "reviewer".to_owned(),
        })
        .await
        .expect("confirm project link review");
    run_project_link_review_effects(&context).await;

    let relationships = context
        .relationship_store
        .list_for_entity(RelationshipEntityKind::Project, &project_id, 20)
        .await
        .expect("project relationships");
    let relationship = relationships
        .iter()
        .find(|item| {
            item.source_entity_kind == RelationshipEntityKind::Project
                && item.source_entity_id == project_id
                && item.target_entity_kind == RelationshipEntityKind::Communication
                && item.target_entity_id == message_id
                && item.relationship_type == "project_has_message"
        })
        .expect("confirmed project link review should create a durable Relationship");

    assert_eq!(
        relationship.review_state,
        RelationshipReviewState::UserConfirmed
    );
    assert_eq!(relationship.confidence, 1.0);
    assert_eq!(
        relationship.metadata["compatibility_table"],
        json!("project_link_reviews")
    );
    assert_eq!(relationship.metadata["project_id"], json!(project_id));
    assert_eq!(relationship.metadata["target_kind"], json!("message"));
    assert_eq!(relationship.metadata["target_id"], json!(message_id));

    let evidence: (String, String, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, excerpt FROM relationship_evidence WHERE relationship_id = $1",
    )
    .bind(&relationship.relationship_id)
    .fetch_one(&context.pool)
    .await
    .expect("relationship evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(
        evidence.2.as_deref(),
        Some("User confirmed message link to project.")
    );

    let observation_kind: String = sqlx::query_scalar(
        "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
    )
    .bind(&evidence.1)
    .fetch_one(&context.pool)
    .await
    .expect("project link review relationship observation kind");
    assert_eq!(observation_kind, "PROJECT_LINK_REVIEW");
}

#[tokio::test]
async fn project_link_review_reset_clears_review_and_demotes_relationship_against_postgres() {
    let Some(context) = live_review_context("project link review reset").await else {
        return;
    };
    let suffix = unique
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/projection_runner.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/projection_runner.rs`
- Size bytes / Размер в байтах: `5289`
- Included characters / Включено символов: `5289`
- Truncated / Обрезано: `no`

```rust
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use serde_json::json;

use makosh_hub_backend::platform::events::{EventStore, NewEventEnvelope, ProjectionCursorStore};
use makosh_hub_backend::platform::projections::{
    ProjectionBatchOutcome, ProjectionHandlerError, run_projection_batch,
};
use makosh_hub_backend::platform::storage::Database;

#[tokio::test]
async fn projection_runner_processes_batch_and_advances_cursor_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let events = EventStore::new(pool.clone());
    let cursors = ProjectionCursorStore::new(pool.clone());

    let suffix = unique_suffix();
    let projection_name = format!("projection_runner_success_{suffix}");
    cursors
        .save_position(&projection_name, latest_event_position(&pool).await)
        .await
        .expect("initialize cursor");
    let first_position = append_projection_test_event(&events, &suffix, "first").await;
    let second_position = append_projection_test_event(&events, &suffix, "second").await;
    let handled_event_ids = Arc::new(Mutex::new(Vec::new()));

    let outcome = run_projection_batch(&events, &cursors, &projection_name, 10, {
        let handled_event_ids = Arc::clone(&handled_event_ids);
        move |event| {
            let handled_event_ids = Arc::clone(&handled_event_ids);
            async move {
                handled_event_ids
                    .lock()
                    .expect("handled ids lock")
                    .push(event.event.event_id);
                Ok(())
            }
        }
    })
    .await
    .expect("projection run");

    assert_eq!(
        outcome,
        ProjectionBatchOutcome {
            processed_count: 2,
            last_processed_position: second_position,
        }
    );
    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("cursor"),
        second_position
    );
    assert_eq!(handled_event_ids.lock().expect("handled ids lock").len(), 2);
    assert!(first_position < second_position);
}

#[tokio::test]
async fn projection_runner_stops_on_handler_error_without_advancing_failed_event_against_postgres()
{
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let events = EventStore::new(pool.clone());
    let cursors = ProjectionCursorStore::new(pool.clone());

    let suffix = unique_suffix();
    let projection_name = format!("projection_runner_failure_{suffix}");
    cursors
        .save_position(&projection_name, latest_event_position(&pool).await)
        .await
        .expect("initialize cursor");
    let first_position = append_projection_test_event(&events, &suffix, "first").await;
    let second_position = append_projection_test_event(&events, &suffix, "second").await;

    let result = run_projection_batch(
        &events,
        &cursors,
        &projection_name,
        10,
        |event| async move {
            if event.position == second_position {
                return Err(ProjectionHandlerError::new("handler failed"));
            }

            Ok(())
        },
    )
    .await;

    assert!(result.is_err(), "handler failure must fail the batch");
    assert_eq!(
        cursors
            .last_processed_position(&projection_name)
            .await
            .expect("cursor after failure"),
        first_position
    );

    let retry = run_projection_batch(&events, &cursors, &projection_name, 10, |_| async {
        Ok(())
    })
    .await
    .expect("retry projection run");

    assert_eq!(
        retry,
        ProjectionBatchOutcome {
            processed_count: 1,
            last_processed_position: second_position,
        }
    );
}

async fn append_projection_test_event(
    events: &EventStore,
    suffix: &str,
    logical_name: &str,
) -> i64 {
    let event_id = format!("evt_projection_runner_{logical_name}_{suffix}");
    let event = NewEventEnvelope::builder(
        &event_id,
        "system_projection_runner_test_event",
        Utc::now(),
        json!({
            "kind": "test",
            "provider": "integration",
            "source_id": event_id,
        }),
        json!({"kind": "system", "entity_id": "backend"}),
    )
    .build()
    .expect("valid event");

    events.append(&event).await.expect("append event")
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
        .to_string()
}

async fn latest_event_position(pool: &sqlx::PgPool) -> i64 {
    sqlx::query_scalar::<_, Option<i64>>("SELECT max(position) FROM event_log")
        .fetch_one(pool)
        .await
        .expect("latest event position")
        .unwrap_or(0)
}
```

### `backend/tests/projects.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/projects.rs`
- Size bytes / Размер в байтах: `11122`
- Included characters / Включено символов: `11122`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::projects::core::{NewProject, ProjectStore};
use makosh_hub_backend::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewState, ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use makosh_hub_backend::platform::storage::Database;

#[tokio::test]
async fn project_detail_links_keyword_messages_documents_and_people_against_postgres() {
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
        .upsert_email_person(&format!("owner-{suffix}@example.com"))
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
    assert_eq!(detail.stats.people_count, 2);
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
            .key_people
            .iter()
            .any(|person| person.email_address == format!("owner-{suffix}@example.com"))
    );

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
    person_store: PersonProjectionStore,
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
        person_store: PersonProjectionStore::new(pool.clone()),
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
            EmailProviderKind::Gmail,
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
```

### `backend/tests/projects_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/projects_api.rs`
- Size bytes / Размер в байтах: `17713`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount, NewRawCommunicationRecord,
};
use makosh_hub_backend::domains::communications::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::projects::core::{NewProject, ProjectStore};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "projects-api-test-token";

#[tokio::test]
async fn projects_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/projects"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn project_detail_returns_live_project_payload() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let project_id = format!("project:v1:api:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("API Project {suffix}"),
                "Product Development",
                "API project detail test",
                "Alex Morgan",
                vec![format!("ApiProject{suffix}")],
            )
            .progress(64),
        )
        .await
        .expect("upsert API project");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/projects/{}",
                urlencoding_percent_encode(&project_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["project"]["project_id"], json!(project_id));
    assert_eq!(body["project"]["progress_percent"], json!(64));
    assert_eq!(body["stats"]["message_count"], json!(0));
    assert!(body["timeline"].as_array().expect("timeline").is_empty());

    sqlx::query("DELETE FROM projects WHERE project_id = $1")
        .bind(&project_id)
        .execute(&pool)
        .await
        .expect("cleanup API project");
}

#[tokio::test]
async fn project_link_candidates_rejects_missing_local_api_secret() {
    let app = build_router(config_with_api_token());
    let response = app
        .oneshot(get_request(
            "/api/v1/projects/project%3Alink-review-placeholder/link-candidates",
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "error": "invalid_api_secret",
            "message": "missing or invalid x-makosh-secret header"
        })
    );
}

#[tokio::test]
async fn project_link_candidates_return_safe_message_and_document_candidates() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let keyword = format!("LinkKeyword{suffix}");
    let project_id = format!("project:v1:link-candidates:{suffix}");

    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Link Candidates Project {suffix}"),
                "Product Development",
                "Project for link candidates API test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(37),
        )
        .await
        .expect("upsert link candidates project");

    let message_id = seed_message(
        &pool,
        suffix,
        &format!("reviewer-link-{suffix}@example.com"),
        &[format!("owner-link-{suffix}@example.com")],
        &format!("provider-link-candidates-message-{suffix}"),
        &format!("{keyword} message subject"),
        "Message body",
    )
    .await;
    let document_id = seed_document(
        &pool,
        format!("doc_link_candidates_{suffix}"),
        &format!("{keyword} architecture.md"),
        "# Architecture\n\nProject body.",
    )
    .await;

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/projects/{}/link-candidates",
                urlencoding_percent_encode(&project_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert_eq!(items.len(), 2);

    let message_candidate = items
        .iter()
        .find(|item| item["target_kind"] == json!("message"))
        .expect("message candidate");
    let document_candidate = items
        .iter()
        .find(|item| item["target_kind"] == json!("document"))
        .expect("document candidate");

    assert_eq!(message_candidate["review_state"], json!("suggested"));
    assert_eq!(message_candidate["target_id"], json!(message_id));
    assert_eq!(message_candidate["project_id"], json!(project_id));
    assert_eq!(document_candidate["review_state"], json!("suggested"));
    assert_eq!(document_candidate["target_id"], json!(document_id));
    assert_eq!(document_candidate["project_id"], json!(project_id));
    assert!(message_candidate["evidence_excerpt"].is_string());
    assert!(document_candidate["evidence_excerpt"].is_string());

    assert_eq!(
        message_candidate["evidence_excerpt"],
        json!(format!("reviewer-link-{suffix}@example.com"))
    );

    let review_items: Vec<(String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT
            review_item_id,
            item_kind,
            metadata->>'mirrored_from',
            metadata->>'target_id'
        FROM review_items
        WHERE metadata->>'project_id' = $1
          AND item_kind = 'project_link_candidate'
        ORDER BY created_at ASC
        "#,
    )
    .bind(&project_id)
    .fetch_all(&pool)
    .await
    .expect("project link candidate review items");
    assert_eq!(review_items.len(), 2);
    assert!(
        review_items
            .iter()
            .all(|item| item.1 == "project_link_candidate")
    );
    assert!(
        review_items
            .iter()
            .all(|item| item.2 == "project_link_candidates")
    );
    assert!(review_items.iter().any(|item| item.3 == message_id));
    assert!(review_items.iter().any(|item| item.3 == document_id));
}

#[tokio::test]
async fn put_project_link_review_updates_review_state() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let project_id = format!("project:v1:link-review-api:{suffix}");
    ProjectStore::new(pool.clone())
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Link Review API {suffix}"),
                "Product Development",
                "Project for link review API test",
                "Alex Morgan",
                vec![format!("LinkReview{suffix}")],
            )
            .progress(66),
        )
        .await
        .expect("upsert link review project");

    let message_id = seed_message(
        &pool,
        suffix,
        &format!("reviewer-{suffix}@example.com"),
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-link-review-api-{suffix}"),
        &format!("LinkReview{suffix} Message"),
        "Link review body",
    )
    .await;

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let command_id = format!("link-review-confirm-{suffix}");
    let response = app
        .oneshot(json_put_request_with_token(
            &format!(
                "/api/v1/projects/{}/link-reviews",
                urlencoding_percent_encode(&project_id)
            ),
            json!({
                "command_id": command_id,
                "target_kind": "message",
                "target_id": message_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;

    assert_eq!(
        body,
        json!({
            "project_id": project_id,
            "target_kind": "message",
            "target_id": message_id,
            "review_state": "user_confirmed",
            "event_id": format!("project_link_review:{command_id}"),
        })
    );

    let persisted_state: String = sqlx::query_scalar(
        "SELECT review_state FROM project_link_reviews WHERE project_id = $1 AND target_kind = 'message' AND target_id = $2",
    )
    .bind(&project_id)
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("review state");
    assert_eq!(persisted_state, "user_confirmed");

    let review_transition_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'projects'
           AND entity_kind = 'project_link_review'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(format!("project_link_review:{command_id}"))
    .fetch_one(&pool)
    .await
    .expect("project link review observation link count");
    assert_eq!(review_transition_link_count, 1);

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'project_id' = $1
          AND metadata->>'target_kind' = 'message'
          AND metadata->>'target_id' = $2
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&project_id)
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("project link review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "project_link_candidate");
    assert_eq!(review_item.2, format!("{project_id}:message:{message_id}"));
}

#[tokio::test]
async fn put_project_link_review_rejects_missing_target() {
    let test_context = TestContext::n
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/relationships.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/relationships.rs`
- Size bytes / Размер в байтах: `25941`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use makosh_hub_backend::application::OrganizationContactLinkApplicationService;
use makosh_hub_backend::domains::graph::core::{GraphNodeKind, node_id};
use makosh_hub_backend::domains::organizations::api::OrganizationStore;
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewState, RelationshipStore,
    RelationshipStoreError,
};
use makosh_hub_backend::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

#[tokio::test]
async fn relationship_store_upserts_persona_relationship_with_evidence_against_postgres() {
    let Some((pool, person_store, relationship_store)) =
        live_relationship_context("persona relationship upsert").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source = person_store
        .upsert_email_person(&format!("relationship-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_person(&format!("relationship-target-{suffix}@example.com"))
        .await
        .expect("target persona");

    let relationship = NewRelationship::between_personas(
        &source.person_id,
        &target.person_id,
        "collaborates_with",
        0.82,
        0.64,
        0.91,
        RelationshipReviewState::UserConfirmed,
    )
    .metadata(json!({"project": "relationship-store-test"}));
    let evidence_source_id = format!("message:{suffix}");
    let first_evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .excerpt("We agreed to collaborate on the Макошь relationship model.")
    .metadata(json!({"channel": "email", "revision": 1}));
    let second_evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .excerpt("Updated relationship evidence.")
    .metadata(json!({"channel": "email", "revision": 2}));

    let first = relationship_store
        .upsert_with_evidence(&relationship, std::slice::from_ref(&first_evidence))
        .await
        .expect("first relationship upsert");
    let second = relationship_store
        .upsert_with_evidence(&relationship, &[second_evidence])
        .await
        .expect("second relationship upsert");

    assert_eq!(first.relationship_id, second.relationship_id);
    assert_eq!(first.source_entity_kind, RelationshipEntityKind::Persona);
    assert_eq!(first.source_entity_id, source.person_id);
    assert_eq!(first.target_entity_kind, RelationshipEntityKind::Persona);
    assert_eq!(first.target_entity_id, target.person_id);
    assert_eq!(first.relationship_type, "collaborates_with");
    assert_eq!(first.trust_score, 0.82);
    assert_eq!(first.strength_score, 0.64);
    assert_eq!(first.confidence, 0.91);
    assert_eq!(first.review_state, RelationshipReviewState::UserConfirmed);

    let evidence_row = sqlx::query(
        r#"
        SELECT excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.relationship_id)
    .bind(RelationshipEvidenceSourceKind::Communication.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored relationship evidence");
    let excerpt: Option<String> = evidence_row.try_get("excerpt").expect("evidence excerpt");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(excerpt.as_deref(), Some("Updated relationship evidence."));
    assert_eq!(metadata, json!({"channel": "email", "revision": 2}));

    let source_relationships = relationship_store
        .list_for_entity(RelationshipEntityKind::Persona, &source.person_id, 10)
        .await
        .expect("source relationships");
    let target_relationships = relationship_store
        .list_for_entity(RelationshipEntityKind::Persona, &target.person_id, 10)
        .await
        .expect("target relationships");

    assert!(
        source_relationships
            .iter()
            .any(|item| item.relationship_id == first.relationship_id)
    );
    assert!(
        target_relationships
            .iter()
            .any(|item| item.relationship_id == first.relationship_id)
    );
}

#[tokio::test]
async fn relationship_store_projects_persona_relationship_into_graph_against_postgres() {
    let Some((pool, person_store, relationship_store)) =
        live_relationship_context("persona relationship graph projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source = person_store
        .upsert_email_person(&format!("graph-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_person(&format!("graph-target-{suffix}@example.com"))
        .await
        .expect("target persona");

    let relationship = NewRelationship::between_personas(
        &source.person_id,
        &target.person_id,
        "knows",
        0.77,
        0.58,
        0.83,
        RelationshipReviewState::Suggested,
    );
    let stored = relationship_store
        .upsert_with_evidence(
            &relationship,
            &[NewRelationshipEvidence::new(
                RelationshipEvidenceSourceKind::Communication,
                format!("message:graph-projection:{suffix}"),
            )
            .excerpt("Introduced during a project discussion.")],
        )
        .await
        .expect("relationship upsert with graph projection");

    let edge_row = sqlx::query(
        r#"
        SELECT edge.edge_id, edge.confidence::float8 AS confidence, edge.review_state, edge.properties
        FROM graph_edges edge
        WHERE edge.source_node_id = $1
          AND edge.target_node_id = $2
          AND edge.relationship_type = 'entity_relationship'
          AND edge.valid_to IS NULL
        "#,
    )
    .bind(node_id(GraphNodeKind::Person, &source.person_id))
    .bind(node_id(GraphNodeKind::Person, &target.person_id))
    .fetch_one(&pool)
    .await
    .expect("relationship graph edge");

    let confidence: f64 = edge_row.try_get("confidence").expect("graph confidence");
    let review_state: String = edge_row
        .try_get("review_state")
        .expect("graph review state");
    let properties: Value = edge_row.try_get("properties").expect("graph properties");
    assert_eq!(confidence, 0.83);
    assert_eq!(review_state, "suggested");
    assert_eq!(properties["relationship_id"], json!(stored.relationship_id));
    assert_eq!(properties["relationship_type"], json!("knows"));
    assert_eq!(properties["trust_score"], json!(0.77));
    assert_eq!(properties["strength_score"], json!(0.58));

    let edge_id: String = edge_row.try_get("edge_id").expect("graph edge id");
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE edge_id = $1
          AND source_kind = 'relationship'
          AND source_id = $2
        "#,
    )
    .bind(edge_id)
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship graph evidence count");

    assert_eq!(evidence_count, 1);
}

#[tokio::test]
async fn relationship_store_projects_supported_cross_domain_relationship_into_graph_against_postgres()
 {
    let Some((pool, _person_store, relationship_store)) =
        live_relationship_context("cross-domain relationship graph projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let decision_id = format!("decision:v1:relationship-graph:{suffix}");
    let project_id = format!("project:v1:relationship-graph:{suffix}");
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Decision,
        source_entity_id: decision_id.clone(),
        target_entity_kind: RelationshipEntityKind::Project,
        target_entity_id: project_id.clone(),
        relationship_type: "sets_direction_for".to_owned(),
        trust_score: 0.7,
        strength_score: 0.62,
        confidence: 0.86,
        review_state: RelationshipReviewState::Suggested,
        valid_from: None,
        valid_to: None,
        metadata: json!({"source": "relationships_cross_domain_test"}),
    };
    let stored = relationship_store
        .upsert_with_evidence(
            &relationship,
            &[NewRelationshipEvidence::new(
                RelationshipEvidenceSourceKind::Decision,
                decision_id.clone(),
            )
            .excerpt("This decision sets direction for the project.")
            .metadata(json!({"source": "relationships_cross_domain_test"}))],
        )
        .await
        .expect("cross-domain relationship upsert with graph projection");

    let decision_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'decision' AND stable_key = $1",
    )
    .bind(&decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision relationship graph node");
    let project_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'project' AND stable_key = $1",
    )
    .bind(&project_id)
    .fetch_one(&pool)
    .await
    .expect("project relationship graph node");
    let graph_edge_row = sqlx::query(
        r#"
        SELECT edge_id, confidence::float8 AS confidence, review_state, properties
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND valid_to IS NULL
        "#,
    )
    .bind(&decision_node_id)
    .bind(&project_node_id)
    .fetch_one(&pool)
    .await
    .expect("cross-domain relationship graph edge");

    let confidence: f64 = graph_edge_row
        .try_get("confidence")
        .expect("graph confidence");
    let review_state: String = graph_edge_row
        .try_get("review_state")
        .expect("graph review state");
    let properties: Value = graph_edge_row
        .try_get("properties")
        .expect("graph properties");

    assert_eq!(confidence, 0.86);
    assert_eq!(review_state, "suggested");
    assert_eq!(properties["relationship_id"], json!(stored.relationship_id));
    assert_eq!(properties["relationship_type"], json!("sets_direction_for"));
    assert_eq!(properties["source_entity_kind"], json!("decision"));
    assert_eq!(properties["target_entity_kind"], json!("project"));

    let edge_id: String = graph_edge_row.try_get("edge_id").expect("graph edge id");
    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_evidence
        WHERE edge_id = $1
          AND source_kind = 'relationship'
          AND source_id = $2
        "#,
    )
    .bind(edge_id)
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship graph evidence count");

    assert_eq!(evidence_count, 1);
}

#[tokio::test]
async fn relationship_store_projects_organization_task_relationship_into_graph_against_postgres() {
    let Some((pool, _person_store, relationship_store)) =
        live_relationship_context("organization task relationship graph projection").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let organization_id = format!("organization:v1:relationship-graph:{suffix}");
    let task_id = format!("task:v1:relationship-graph:{suffix}");
    let relationship = NewRelationship {
        sour
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/relationships_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/relationships_api.rs`
- Size bytes / Размер в байтах: `10820`
- Included characters / Включено символов: `10820`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::graph::core::{GraphNodeKind, node_id};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, Relationship, RelationshipEvidenceSourceKind,
    RelationshipReviewState, RelationshipStore,
};
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "relationships-api-test-token";

#[tokio::test]
async fn relationships_list_returns_entity_scoped_relationships() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_persona_relationship(&pool, suffix).await;
    let source_entity_id = &stored.source_entity_id;

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/relationships?entity_kind=persona&entity_id={source_entity_id}&limit=10"
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let item = items
        .iter()
        .find(|item| item["relationship_id"] == json!(stored.relationship_id))
        .expect("seeded relationship");

    assert_eq!(item["source_entity_kind"], "persona");
    assert_eq!(item["source_entity_id"], stored.source_entity_id);
    assert_eq!(item["target_entity_kind"], "persona");
    assert_eq!(item["target_entity_id"], stored.target_entity_id);
    assert_eq!(item["relationship_type"], "collaborates_with");
    assert_eq!(item["review_state"], "suggested");
}

#[tokio::test]
async fn relationships_list_returns_global_suggested_review_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let suggested = seed_persona_relationship_with_state(
        &pool,
        suffix,
        "global_review_suggested",
        RelationshipReviewState::Suggested,
    )
    .await;
    let confirmed = seed_persona_relationship_with_state(
        &pool,
        suffix + 1,
        "global_review_confirmed",
        RelationshipReviewState::UserConfirmed,
    )
    .await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/relationships?review_state=suggested&limit=10",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(
        items
            .iter()
            .any(|item| item["relationship_id"] == json!(suggested.relationship_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["relationship_id"] != json!(confirmed.relationship_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["review_state"] == json!("suggested"))
    );
}

#[tokio::test]
async fn put_relationship_review_updates_relationship_and_graph_projection() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let stored = seed_persona_relationship(&pool, suffix).await;
    let relationship_id = path_segment(&stored.relationship_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/relationships/{relationship_id}/review"),
            json!({
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["relationship_id"], stored.relationship_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert_eq!(body["trust_score"], json!(0.72));
    assert_eq!(body["strength_score"], json!(0.66));

    let stored_review_state: String =
        sqlx::query_scalar("SELECT review_state FROM relationships WHERE relationship_id = $1")
            .bind(&stored.relationship_id)
            .fetch_one(&pool)
            .await
            .expect("relationship review state");
    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'relationships'
           AND entity_kind = 'relationship'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    let graph_row = sqlx::query(
        r#"
        SELECT review_state, properties
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND valid_to IS NULL
        "#,
    )
    .bind(node_id(GraphNodeKind::Person, &stored.source_entity_id))
    .bind(node_id(GraphNodeKind::Person, &stored.target_entity_id))
    .fetch_one(&pool)
    .await
    .expect("relationship graph edge");
    let graph_review_state: String = graph_row.try_get("review_state").expect("graph review");
    let graph_properties: Value = graph_row.try_get("properties").expect("graph properties");

    assert_eq!(stored_review_state, "user_confirmed");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(graph_review_state, "user_confirmed");
    assert_eq!(
        graph_properties["relationship_id"],
        json!(stored.relationship_id)
    );

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("relationship observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(payload["relationship_id"], json!(stored.relationship_id));
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'relationship_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.relationship_id)
    .fetch_one(&pool)
    .await
    .expect("relationship review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "relationship");
    assert_eq!(review_item.2, stored.relationship_id);
}

async fn app_and_pool(database_url: &str) -> (axum::Router, PgPool) {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url),
        database,
    );

    (app, pool)
}

async fn seed_persona_relationship(pool: &PgPool, suffix: u128) -> Relationship {
    seed_persona_relationship_with_state(
        pool,
        suffix,
        "collaborates_with",
        RelationshipReviewState::Suggested,
    )
    .await
}

async fn seed_persona_relationship_with_state(
    pool: &PgPool,
    suffix: u128,
    relationship_type: &str,
    review_state: RelationshipReviewState,
) -> Relationship {
    let person_store = PersonProjectionStore::new(pool.clone());
    let source = person_store
        .upsert_email_person(&format!("relationship-api-source-{suffix}@example.com"))
        .await
        .expect("source persona");
    let target = person_store
        .upsert_email_person(&format!("relationship-api-target-{suffix}@example.com"))
        .await
        .expect("target persona");
    let relationship = NewRelationship::between_personas(
        &source.person_id,
        &target.person_id,
        relationship_type,
        0.72,
        0.66,
        0.88,
        review_state,
    )
    .metadata(json!({"source": "relationships_api_test"}));
    let evidence = NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Communication,
        format!("message:relationship-api:{suffix}"),
    )
    .excerpt("They agreed to collaborate on the relationship API.")
    .metadata(json!({"source": "relationships_api_test"}));

    RelationshipStore::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await
        .expect("seed relationship")
}

fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

fn json_put_request(uri: &str, value: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(value.to_string()))
        .expect("request")
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
```

### `backend/tests/review_inbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/review_inbox.rs`
- Size bytes / Размер в байтах: `77558`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use chrono::{TimeZone, Utc};
use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::decisions::DecisionStore;
use makosh_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use makosh_hub_backend::domains::obligations::ObligationStore;
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::identity::PersonIdentityStore;
use makosh_hub_backend::domains::projects::core::ProjectStore;
use makosh_hub_backend::domains::relationships::RelationshipStore;
use makosh_hub_backend::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxStore, ReviewItemKind, ReviewItemStatus,
    ReviewPromotionTarget,
};
use makosh_hub_backend::domains::tasks::api::TaskStore;
use makosh_hub_backend::platform::events::EventStore;
use makosh_hub_backend::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::review_inbox::project_person_identity_review_event;
use makosh_hub_backend::workflows::review_inbox::sync_decisions_to_review_for_observations;
use makosh_hub_backend::workflows::review_inbox::sync_obligations_to_review_for_observations;
use makosh_hub_backend::workflows::review_inbox::sync_relationships_to_review_for_observations;
use makosh_hub_backend::workflows::review_inbox::sync_task_candidates_to_review_for_observations;
use makosh_hub_backend::workflows::review_promotion::ReviewPromotionService;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

#[tokio::test]
async fn review_inbox_creates_evidence_backed_item_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("evidence-backed review item").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                "Send the Friday report",
                "Email evidence suggests a possible deadline-backed task.",
                0.84,
            )
            .metadata(json!({"detector": "contract-test"})),
            &[NewReviewItemEvidence::new(observation_id.clone()).role("primary")],
        )
        .await
        .expect("create review item");

    assert_eq!(item.item_kind, ReviewItemKind::PotentialTask);
    assert_eq!(item.status, ReviewItemStatus::New);
    assert_eq!(item.target_domain, None);
    assert_eq!(item.confidence, 0.84);

    let evidence_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM review_item_evidence
        WHERE review_item_id = $1
          AND observation_id = $2
        "#,
    )
    .bind(&item.review_item_id)
    .bind(&observation_id)
    .fetch_one(&pool)
    .await
    .expect("review evidence count");

    assert_eq!(evidence_count, 1);

    let open = review_store
        .list_by_status(ReviewItemStatus::New, 25)
        .await
        .expect("list new review items");
    assert!(
        open.iter()
            .any(|candidate| candidate.review_item_id == item.review_item_id)
    );

    let detected_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM event_log
        WHERE event_type = 'task.candidate.detected.v1'
          AND subject ->> 'review_item_id' = $1
        "#,
    )
    .bind(&item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("candidate detected event count");
    assert_eq!(detected_count, 1);

    let available_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM event_log
        WHERE event_type = 'review.item.available.v1'
          AND subject ->> 'review_item_id' = $1
        "#,
    )
    .bind(&item.review_item_id)
    .fetch_one(&pool)
    .await
    .expect("review available event count");
    assert_eq!(available_count, 1);
}

#[tokio::test]
async fn review_inbox_filters_active_and_all_lists_against_postgres() {
    let Some((_, observation_store, review_store)) =
        live_review_context("review list filters").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let new_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                "Prepare migration notes",
                "New task candidate to review.",
                0.75,
            ),
            &[NewReviewItemEvidence::new(observation_id.clone())],
        )
        .await
        .expect("create new review item");

    let reviewed_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialDecision,
                "Archive obsolete item",
                "Decision candidate to process.",
                0.76,
            ),
            &[NewReviewItemEvidence::new(observation_id.clone())],
        )
        .await
        .expect("create review item to move into review");

    let dismissed_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialProject,
                "Retire old note",
                "Project candidate should be dismissed.",
                0.5,
            ),
            &[NewReviewItemEvidence::new(observation_id)],
        )
        .await
        .expect("create dismissible review item");
    let _dismissed_item = review_store
        .set_status(&dismissed_item.review_item_id, ReviewItemStatus::Dismissed)
        .await
        .expect("dismiss review item");

    let in_review_item = review_store
        .set_status(&reviewed_item.review_item_id, ReviewItemStatus::InReview)
        .await
        .expect("move item into review");

    let active = review_store
        .list_open(100)
        .await
        .expect("list open review items");
    let active_ids: Vec<&str> = active
        .iter()
        .map(|item| item.review_item_id.as_str())
        .collect();

    assert!(active_ids.contains(&new_item.review_item_id.as_str()));
    assert!(active_ids.contains(&in_review_item.review_item_id.as_str()));
    assert!(!active_ids.contains(&dismissed_item.review_item_id.as_str()));

    let all = review_store
        .list_all(100)
        .await
        .expect("list all review items");
    let all_ids: Vec<&str> = all
        .iter()
        .map(|item| item.review_item_id.as_str())
        .collect();

    assert!(all_ids.contains(&new_item.review_item_id.as_str()));
    assert!(all_ids.contains(&in_review_item.review_item_id.as_str()));
    assert!(all_ids.contains(&dismissed_item.review_item_id.as_str()));
}

#[tokio::test]
async fn review_inbox_lifecycle_approves_promotes_dismisses_and_archives_against_postgres() {
    let Some((_pool, observation_store, review_store)) =
        live_review_context("review lifecycle").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialDecision,
                "Buy a NAS",
                "A decision candidate needs explicit review.",
                0.91,
            ),
            &[NewReviewItemEvidence::new(observation_id.clone())],
        )
        .await
        .expect("create decision review item");

    let in_review = review_store
        .set_status(&item.review_item_id, ReviewItemStatus::InReview)
        .await
        .expect("move review item into review");
    assert_eq!(in_review.status, ReviewItemStatus::InReview);

    let approved = review_store
        .set_status(&item.review_item_id, ReviewItemStatus::Approved)
        .await
        .expect("approve review item");
    assert_eq!(approved.status, ReviewItemStatus::Approved);

    let approved_again = review_store
        .set_status(&item.review_item_id, ReviewItemStatus::Approved)
        .await
        .expect("repeat approve review item idempotently");
    assert_eq!(approved_again.status, ReviewItemStatus::Approved);

    let promoted = review_store
        .promote(
            &item.review_item_id,
            ReviewPromotionTarget::new("decisions", "decision", format!("decision:v1:{suffix}")),
        )
        .await
        .expect("promote review item");
    assert_eq!(promoted.status, ReviewItemStatus::Promoted);
    assert_eq!(promoted.target_domain.as_deref(), Some("decisions"));
    assert_eq!(promoted.target_entity_kind.as_deref(), Some("decision"));

    let dismissed_item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::KnowledgeCandidate,
                "Dismiss low-value note",
                "Candidate is not useful enough for promotion.",
                0.51,
            ),
            &[NewReviewItemEvidence::new(observation_id)],
        )
        .await
        .expect("create dismissable review item");
    let dismissed = review_store
        .set_status(&dismissed_item.review_item_id, ReviewItemStatus::Dismissed)
        .await
        .expect("dismiss review item");
    assert_eq!(dismissed.status, ReviewItemStatus::Dismissed);

    let archived = review_store
        .set_status(&dismissed_item.review_item_id, ReviewItemStatus::Archived)
        .await
        .expect("archive review item");
    assert_eq!(archived.status, ReviewItemStatus::Archived);

    for event_type in [
        "decision.candidate.detected.v1",
        "review.item.available.v1",
        "review.item.approved.v1",
        "review.item.promoted.v1",
    ] {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM event_log
            WHERE event_type = $1
              AND subject ->> 'review_item_id' = $2
            "#,
        )
        .bind(event_type)
        .bind(&item.review_item_id)
        .fetch_one(&_pool)
        .await
        .expect("review lifecycle event count");
        assert_eq!(count, 1, "missing lifecycle event {event_type}");
    }

    let dismissed_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM event_log
        WHERE event_type = 'review.item.dismissed.v1'
          AND subject ->> 'review_item_id' = $1
        "#,
    )
    .bind(&dismissed_item.review_item_id)
    .fetch_one(&_pool)
    .await
    .expect("dismissed event count");
    assert_eq!(dismissed_count, 1);
}

#[tokio::test]
async fn review_inbox_status_with_observation_materializes_transition_link_against_postgres() {
    let Some((pool, observation_store, review_store)) =
        live_review_context("review status observation link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let source_observation_id = seed_manual_note(&observation_store, suffix).await;

    let item = review_store
        .create_with_evidence(
            &NewReviewItem::new(
                ReviewItemKind::PotentialTask,
                "Status owner path",
                "Review status change should materialize its own review transition link.",
                0.73,
            ),
            &[NewReviewItemEvidence::new(source_observation_id)],
        )
        .await
        .expect("create review item");

    let review_observation = observation_store
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                U
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/risk_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/risk_engine.rs`
- Size bytes / Размер в байтах: `2824`
- Included characters / Включено символов: `2824`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::engines::risk::{
    RiskAttentionStatus, RiskEngine, RiskSeverity, RiskSignal,
};

#[test]
fn risk_engine_derives_attention_status_from_unresolved_severity() {
    let no_risks = RiskEngine::derive_attention_status(&[]);
    assert_eq!(no_risks, RiskAttentionStatus::Healthy);
    assert_eq!(no_risks.as_persona_health_status(), "healthy");

    let needs_attention = RiskEngine::derive_attention_status(&[
        RiskSignal::resolved(RiskSeverity::Critical),
        RiskSignal::unresolved(RiskSeverity::Low),
        RiskSignal::unresolved(RiskSeverity::Medium),
    ]);
    assert_eq!(needs_attention, RiskAttentionStatus::NeedsAttention);
    assert_eq!(
        needs_attention.as_persona_health_status(),
        "needs_attention"
    );

    let at_risk = RiskEngine::derive_attention_status(&[
        RiskSignal::unresolved(RiskSeverity::Medium),
        RiskSignal::unresolved(RiskSeverity::High),
    ]);
    assert_eq!(at_risk, RiskAttentionStatus::AtRisk);
    assert_eq!(at_risk.as_persona_health_status(), "at_risk");
}

#[test]
fn risk_severity_rejects_unknown_compatibility_values() {
    let error = RiskSeverity::parse("urgent").expect_err("unknown severity must be rejected");

    assert_eq!(error.to_string(), "invalid risk severity `urgent`");
}

#[test]
fn risk_engine_builds_source_backed_persona_observation_draft() {
    let draft = RiskEngine::persona_observation(
        "person:v1:email:alice@example.com",
        "relationship_attention",
        "Open evidence-backed relationship risk requires owner review.",
        "high",
        "communication_messages:message-1",
    )
    .expect("source-backed risk observation should be valid");

    assert_eq!(draft.affected_entity_kind, "persona");
    assert_eq!(
        draft.affected_entity_id,
        "person:v1:email:alice@example.com"
    );
    assert_eq!(draft.risk_type, "relationship_attention");
    assert_eq!(
        draft.evidence,
        "Open evidence-backed relationship risk requires owner review."
    );
    assert_eq!(draft.source, "communication_messages:message-1");
    assert_eq!(draft.confidence, 0.5);
    assert_eq!(draft.severity, RiskSeverity::High);
    assert_eq!(draft.severity.as_str(), "high");
    assert_eq!(draft.suggested_handling_state, "review_now");
    assert_eq!(draft.review_state, "suggested");
}

#[test]
fn risk_engine_rejects_unsourced_persona_observation() {
    let error = RiskEngine::persona_observation(
        "person:v1:email:alice@example.com",
        "relationship_attention",
        "Open evidence-backed relationship risk requires owner review.",
        "high",
        " ",
    )
    .expect_err("risk observation source should be required");

    assert_eq!(
        error.to_string(),
        "risk observation source must not be empty"
    );
}
```

### `backend/tests/search.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/search.rs`
- Size bytes / Размер в байтах: `7722`
- Included characters / Включено символов: `7722`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::engines::search::{SearchDocument, SearchError, SearchIndex};

#[test]
fn search_index_returns_message_by_body_term() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Budget review".to_owned(),
            body: "Please review the Q2 budget before Monday.".to_owned(),
        })
        .expect("index document");
    index.commit().expect("commit index");

    let results = index.search("budget", 10).expect("search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].object_id, "message-1");
    assert_eq!(results[0].object_kind, "message");
}

#[test]
fn search_index_rejects_blank_required_document_fields() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    for (field_name, document) in [
        (
            "object_id",
            SearchDocument {
                object_id: " ".to_owned(),
                object_kind: "message".to_owned(),
                title: "Budget review".to_owned(),
                body: "Please review the Q2 budget before Monday.".to_owned(),
            },
        ),
        (
            "object_kind",
            SearchDocument {
                object_id: "message-1".to_owned(),
                object_kind: " ".to_owned(),
                title: "Budget review".to_owned(),
                body: "Please review the Q2 budget before Monday.".to_owned(),
            },
        ),
        (
            "title",
            SearchDocument {
                object_id: "message-1".to_owned(),
                object_kind: "message".to_owned(),
                title: " ".to_owned(),
                body: "Please review the Q2 budget before Monday.".to_owned(),
            },
        ),
    ] {
        let error = index
            .upsert_document(&document)
            .expect_err("blank document field must fail");

        assert!(matches!(error, SearchError::EmptyField(actual) if actual == field_name));
    }
}

#[test]
fn search_index_rejects_blank_query() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    let error = index.search(" \t", 10).expect_err("blank query must fail");

    assert!(matches!(error, SearchError::EmptyField("query")));
}

#[test]
fn search_index_rejects_zero_limit() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    let error = index.search("budget", 0).expect_err("zero limit must fail");

    assert!(matches!(error, SearchError::InvalidLimit));
}

#[test]
fn search_index_replaces_existing_document_identity() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Budget review".to_owned(),
            body: "Please review the Q2 budget before Monday.".to_owned(),
        })
        .expect("index first document");
    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
            body: "Please review the implementation roadmap before Monday.".to_owned(),
        })
        .expect("replace document");
    index.commit().expect("commit index");

    let old_results = index.search("budget", 10).expect("search old term");
    let new_results = index.search("roadmap", 10).expect("search new term");

    assert_eq!(old_results, Vec::new());
    assert_eq!(
        new_results,
        vec![makosh_hub_backend::engines::search::SearchResult {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
        }]
    );
}

#[test]
fn search_index_replaces_committed_document_identity() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Budget review".to_owned(),
            body: "Please review the Q2 budget before Monday.".to_owned(),
        })
        .expect("index first document");
    index.commit().expect("commit first version");

    let old_results = index.search("budget", 10).expect("search old term");
    assert_eq!(old_results.len(), 1);

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
            body: "Please review the implementation roadmap before Monday.".to_owned(),
        })
        .expect("replace committed document");
    index.commit().expect("commit second version");

    let old_results = index.search("budget", 10).expect("search old term");
    let new_results = index.search("roadmap", 10).expect("search new term");

    assert_eq!(old_results, Vec::new());
    assert_eq!(
        new_results,
        vec![makosh_hub_backend::engines::search::SearchResult {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
        }]
    );
}

#[test]
fn search_index_accepts_blank_body_for_title_only_documents() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "document-1".to_owned(),
            object_kind: "document".to_owned(),
            title: "PDF metadata overview".to_owned(),
            body: " ".to_owned(),
        })
        .expect("index title-only document");
    index.commit().expect("commit index");

    let results = index.search("metadata", 10).expect("search title term");

    assert_eq!(
        results,
        vec![makosh_hub_backend::engines::search::SearchResult {
            object_id: "document-1".to_owned(),
            object_kind: "document".to_owned(),
            title: "PDF metadata overview".to_owned(),
        }]
    );
}

#[test]
fn search_index_distinguishes_delimiter_bearing_document_identities() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "c".to_owned(),
            object_kind: "a:b".to_owned(),
            title: "First identity".to_owned(),
            body: "sharedterm".to_owned(),
        })
        .expect("index first document");
    index
        .upsert_document(&SearchDocument {
            object_id: "b:c".to_owned(),
            object_kind: "a".to_owned(),
            title: "Second identity".to_owned(),
            body: "sharedterm".to_owned(),
        })
        .expect("index second document");
    index.commit().expect("commit index");

    let results = index.search("sharedterm", 10).expect("search");

    assert_eq!(results.len(), 2);
}
```

### `backend/tests/secret_vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/secret_vault.rs`
- Size bytes / Размер в байтах: `15834`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use tempfile::tempdir;

use makosh_hub_backend::platform::secrets::{DatabaseEncryptedSecretVault, EncryptedSecretVault};
use makosh_hub_backend::platform::secrets::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReference, SecretReferenceStore,
    SecretResolutionError, SecretResolver, SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::vault::{
    EntropyEvent, HostVault, HostVaultConfig, SecretEntryContext, VaultMode,
};

#[tokio::test]
async fn encrypted_vault_persists_secrets_without_plaintext_leakage() {
    let directory = tempdir().expect("tempdir");
    let vault_path = directory.path().join("makosh-secrets.vault.json");
    let vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("correct horse battery staple").expect("vault key"),
    );

    vault
        .store_secret("secret:test:oauth", "gmail-refresh-token")
        .expect("store secret");

    let file_contents = fs::read_to_string(&vault_path).expect("vault file");
    assert!(!file_contents.contains("gmail-refresh-token"));

    let resolved = vault
        .resolve(&secret_reference("secret:test:oauth"))
        .await
        .expect("resolve secret");
    assert_eq!(resolved.expose_for_runtime(), "gmail-refresh-token");
    assert_eq!(
        format!("{resolved:?}"),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[tokio::test]
async fn encrypted_vault_rejects_wrong_master_key() {
    let directory = tempdir().expect("tempdir");
    let vault_path = directory.path().join("makosh-secrets.vault.json");
    let vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("correct horse battery staple").expect("vault key"),
    );
    vault
        .store_secret("secret:test:password", "imap-app-password")
        .expect("store secret");

    let wrong_key_vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("wrong master key").expect("wrong vault key"),
    );
    let error = wrong_key_vault
        .resolve(&secret_reference("secret:test:password"))
        .await
        .expect_err("wrong master key must fail");

    assert!(matches!(error, SecretResolutionError::StoreFailure { .. }));
}

#[tokio::test]
async fn database_encrypted_vault_persists_ciphertext_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = DatabaseEncryptedSecretVault::new(
        pool.clone(),
        ResolvedSecret::new("database vault key").expect("vault key"),
    );
    let secret_ref = format!("secret:test:database-vault:{}", unique_suffix());

    let reference = secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::DatabaseEncryptedVault,
            "Database encrypted test secret",
        ))
        .await
        .expect("store database vault secret reference");
    vault
        .store_secret(&secret_ref, "database-vault-secret")
        .await
        .expect("store database vault secret");

    let ciphertext: String = sqlx::query_scalar(
        r#"
        SELECT ciphertext
        FROM encrypted_secret_vault_entries
        WHERE secret_ref = $1
        "#,
    )
    .bind(&secret_ref)
    .fetch_one(&pool)
    .await
    .expect("load stored ciphertext");
    assert!(!ciphertext.contains("database-vault-secret"));

    let resolved = vault
        .resolve(&reference)
        .await
        .expect("resolve database vault secret");
    assert_eq!(resolved.expose_for_runtime(), "database-vault-secret");
}

#[tokio::test]
async fn database_encrypted_vault_rejects_wrong_master_key_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = DatabaseEncryptedSecretVault::new(
        pool.clone(),
        ResolvedSecret::new("database vault key").expect("vault key"),
    );
    let wrong_key_vault = DatabaseEncryptedSecretVault::new(
        pool,
        ResolvedSecret::new("wrong database vault key").expect("wrong vault key"),
    );
    let secret_ref = format!("secret:test:database-vault-wrong-key:{}", unique_suffix());

    let reference = secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::DatabaseEncryptedVault,
            "Database encrypted wrong key test secret",
        ))
        .await
        .expect("store database vault secret reference");
    vault
        .store_secret(&secret_ref, "database-vault-secret")
        .await
        .expect("store database vault secret");

    let error = wrong_key_vault
        .resolve(&reference)
        .await
        .expect_err("wrong database vault key must fail");

    assert!(matches!(error, SecretResolutionError::StoreFailure { .. }));
}

#[test]
fn host_vault_requires_entropy_threshold_before_create() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());

    vault
        .collect_entropy(entropy_events(1_999))
        .expect("collect entropy");
    let error = vault.create().expect_err("insufficient entropy must fail");

    assert!(error.to_string().contains("insufficient vault entropy"));
}

#[tokio::test]
async fn host_vault_create_unlock_store_and_resolve_secret() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    let status = vault.create().expect("create vault");
    assert_eq!(status.state, VaultMode::Unlocked);

    let metadata = serde_json::json!({
        "provider": "imap",
        "account_id": "acct-host-vault"
    });
    vault
        .store_secret(
            "secret:provider-account:acct-host-vault:imap_password",
            "host-vault-password",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "acct-host-vault",
                purpose: "imap_password",
                secret_kind: "password",
                label: "IMAP password",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");

    let database =
        fs::read_to_string(directory.path().join("vault").join("vault.db")).unwrap_or_default();
    assert!(!database.contains("host-vault-password"));

    let resolved = vault
        .resolve(&host_vault_secret_reference(
            "secret:provider-account:acct-host-vault:imap_password",
            SecretKind::Password,
        ))
        .await
        .expect("resolve host vault secret");
    assert_eq!(resolved.expose_for_runtime(), "host-vault-password");

    vault.lock().expect("lock vault");
    assert_eq!(vault.status().expect("status").state, VaultMode::Locked);
    vault.unlock().expect("unlock vault");
    assert_eq!(
        vault
            .read_secret("secret:provider-account:acct-host-vault:imap_password")
            .expect("read after unlock"),
        "host-vault-password"
    );
}

#[tokio::test]
async fn host_vault_unlock_existing_reopens_session_after_runtime_restart() {
    let directory = tempdir().expect("tempdir");
    let vault_home = directory.path().join("vault");
    let dev_key_path = directory.path().join("dev").join("master.key");
    let metadata = serde_json::json!({
        "provider": "imap",
        "account_id": "acct-host-vault-restart"
    });
    let secret_ref = "secret:provider-account:acct-host-vault-restart:imap_password";

    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    vault.create().expect("create vault");
    vault
        .store_secret(
            secret_ref,
            "restart-secret",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "acct-host-vault-restart",
                purpose: "imap_password",
                secret_kind: "password",
                label: "IMAP password",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");

    let restarted = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("restarted host vault");
    assert_eq!(restarted.status().expect("status").state, VaultMode::Locked);

    let status = restarted.unlock_existing().expect("unlock existing vault");
    assert_eq!(status.state, VaultMode::Unlocked);
    assert_eq!(
        restarted
            .read_secret(secret_ref)
            .expect("read restarted secret"),
        "restart-secret"
    );
}

#[test]
fn host_vault_delete_removes_secret_and_manifest() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    vault.create().expect("create vault");
    let metadata = serde_json::json!({
        "provider": "whatsapp_web",
        "account_id": "acct-whatsapp-delete"
    });
    let secret_ref = "secret:provider-account:acct-whatsapp-delete:whatsapp_web_session_key";
    vault
        .store_secret(
            secret_ref,
            "session-material",
            SecretEntryContext {
                entry_kind: "provider_session",
                account_id: "acct-whatsapp-delete",
                purpose: "whatsapp_web_session_key",
                secret_kind: "other",
                label: "WhatsApp session credential",
                metadata: &metadata,
            },
        )
        .expect("store host vault secret");

    assert_eq!(vault.account_secret_manifest().expect("manifest").len(), 1);
    assert!(vault.delete_secret(secret_ref).expect("delete secret"));
    assert!(!vault.delete_secret(secret_ref).expect("idempotent delete"));
    assert!(
        vault
            .account_secret_manifest()
            .expect("manifest")
            .is_empty()
    );
    let error = vault
        .read_secret(secret_ref)
        .expect_err("secret should be removed");
    assert!(
        error
            .to_string()
            .contains("secret was not found in host vault")
    );
}

#[tokio::test]
async fn host_vault_rejects_tampered_ciphertext() {
    let directory = tempdir().expect("tempdir");
    let vault = test_host_vault(directory.path());
    vault
        .collect_entropy(entropy_events(2_000))
        .expect("collect entropy");
    vault.create().expect("create vault");
    let metadata = serde_json::json!({});
    let secret_ref = "secret:provider-account:tampered:imap_password";
    vault
        .store_secret(
            secret_ref,
            "tamper-secret",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "tampered",
                purpose: "imap_password",
                secret_kind: "password",
                label: "IMAP password",
                metadata: &metadata,
            },
        )
        .
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/secrets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/secrets.rs`
- Size bytes / Размер в байтах: `5324`
- Included characters / Включено символов: `5324`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use serde_json::json;

use chrono::Utc;

use makosh_hub_backend::platform::secrets::{
    InMemorySecretResolver, NewSecretReference, SecretKind, SecretReference, SecretReferenceStore,
    SecretResolutionError, SecretResolver, SecretStoreKind,
};
use makosh_hub_backend::platform::storage::Database;

#[test]
fn secret_reference_enums_reject_unsupported_values() {
    assert_eq!(
        SecretKind::try_from("oauth_token").expect("oauth token kind"),
        SecretKind::OauthToken
    );
    assert_eq!(
        SecretKind::try_from("app_password").expect("app password kind"),
        SecretKind::AppPassword
    );
    assert_eq!(
        SecretStoreKind::try_from("os_keychain").expect("os keychain store kind"),
        SecretStoreKind::OsKeychain
    );
    assert_eq!(
        SecretStoreKind::try_from("database_encrypted_vault")
            .expect("database encrypted vault store kind"),
        SecretStoreKind::DatabaseEncryptedVault
    );
    assert!(SecretKind::try_from("plain_text").is_err());
    assert!(SecretStoreKind::try_from("postgres").is_err());
}

#[tokio::test]
async fn in_memory_secret_resolver_resolves_test_double_references_without_debug_leaking_value() {
    let mut resolver = InMemorySecretResolver::new();
    resolver
        .insert("secret:test:oauth", "fake-runtime-secret")
        .expect("insert in-memory secret");

    let reference = test_secret_reference(
        "secret:test:oauth",
        SecretKind::OauthToken,
        SecretStoreKind::TestDouble,
    );
    let resolved = resolver
        .resolve(&reference)
        .await
        .expect("resolve test secret");

    assert_eq!(resolved.expose_for_runtime(), "fake-runtime-secret");
    assert!(!format!("{resolved:?}").contains("fake-runtime-secret"));
}

#[tokio::test]
async fn in_memory_secret_resolver_reports_missing_test_double_references() {
    let resolver = InMemorySecretResolver::new();
    let reference = test_secret_reference(
        "secret:test:missing",
        SecretKind::Password,
        SecretStoreKind::TestDouble,
    );

    let error = resolver
        .resolve(&reference)
        .await
        .expect_err("missing in-memory secret should fail");

    assert_eq!(
        error,
        SecretResolutionError::MissingSecret {
            secret_ref: "secret:test:missing".to_owned()
        }
    );
}

#[tokio::test]
async fn in_memory_secret_resolver_rejects_non_test_double_store_kinds() {
    let mut resolver = InMemorySecretResolver::new();
    resolver
        .insert("secret:os:keychain", "fake-runtime-secret")
        .expect("insert in-memory secret");
    let reference = test_secret_reference(
        "secret:os:keychain",
        SecretKind::Password,
        SecretStoreKind::OsKeychain,
    );

    let error = resolver
        .resolve(&reference)
        .await
        .expect_err("non-test store kind should fail");

    assert_eq!(
        error,
        SecretResolutionError::UnsupportedStoreKind("os_keychain".to_owned())
    );
}

#[test]
fn resolved_secret_rejects_empty_values() {
    let error = InMemorySecretResolver::new()
        .insert("secret:test:empty", " ")
        .expect_err("empty secret value should fail");

    assert_eq!(error, SecretResolutionError::EmptySecretValue);
}

#[tokio::test]
async fn secret_references_store_only_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = SecretReferenceStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();
    let secret_ref = format!("secret:gmail:oauth:{suffix}");

    let stored = store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::OauthToken,
                SecretStoreKind::OsKeychain,
                "Gmail OAuth credential",
            )
            .metadata(json!({
                "service": "makosh",
                "account": format!("gmail-user-{suffix}@example.com")
            })),
        )
        .await
        .expect("store secret reference");

    assert_eq!(stored.secret_ref, secret_ref);
    assert_eq!(stored.secret_kind, SecretKind::OauthToken);
    assert_eq!(stored.store_kind, SecretStoreKind::OsKeychain);
    assert_eq!(stored.metadata["service"], "makosh");

    let loaded = store
        .secret_reference(&stored.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference exists");
    assert_eq!(loaded, stored);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

fn test_secret_reference(
    secret_ref: &str,
    secret_kind: SecretKind,
    store_kind: SecretStoreKind,
) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind,
        store_kind,
        label: "Test secret reference".to_owned(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}
```

### `backend/tests/settings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/settings.rs`
- Size bytes / Размер в байтах: `19550`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

use makosh_hub_backend::app::build_router_with_database;
use makosh_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, EmailProviderKind, NewProviderAccount,
};
use makosh_hub_backend::platform::settings::{ApplicationSettingsStore, SettingValueKind};
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "settings-api-test-token";

static SETTINGS_DB_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn application_settings_store_lists_seeded_settings_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    let settings = store.list_settings().await.expect("list settings");

    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "ai.chat_model")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "server.http_addr")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.layout")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.sidebar")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.theme")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.locale")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "frontend.ui_state")
    );
    assert!(
        settings
            .iter()
            .any(|setting| setting.setting_key == "ui.theme")
    );
    assert!(
        settings
            .iter()
            .all(|setting| !setting.setting_key.contains("password"))
    );
}

#[tokio::test]
async fn application_settings_include_frontend_layout_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let layout_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.layout")
        .expect("frontend layout setting");

    assert_eq!(layout_setting.category, "frontend");
    assert_eq!(layout_setting.value_kind, SettingValueKind::Json);
    assert_eq!(layout_setting.value["schemaVersion"], json!(2));
    assert!(layout_setting.value["views"].is_object());
    assert!(layout_setting.is_editable);
}

#[tokio::test]
async fn application_settings_include_frontend_sidebar_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let sidebar_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.sidebar")
        .expect("frontend sidebar setting");

    assert_eq!(sidebar_setting.category, "frontend");
    assert_eq!(sidebar_setting.value_kind, SettingValueKind::Json);
    assert_eq!(sidebar_setting.metadata["schema_version"], json!(3));
    assert!(sidebar_setting.value["groups"].is_array());
    assert!(sidebar_setting.value["hiddenItemIds"].is_array());
    if sidebar_setting.value["schemaVersion"] == json!(3) {
        assert!(sidebar_setting.value["rootItemIds"].is_array());
        assert_eq!(
            sidebar_setting.value["rootItemIds"][1],
            json!("group:communications")
        );
        assert_eq!(
            sidebar_setting.value["groups"][0]["itemIds"][0],
            json!("communications.mail")
        );
        assert_eq!(
            sidebar_setting.value["groups"][0]["separatorBeforeItemIds"],
            json!([])
        );
    }
    assert!(sidebar_setting.is_editable);
}

#[tokio::test]
async fn application_settings_include_frontend_theme_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let theme_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.theme")
        .expect("frontend theme setting");

    assert_eq!(theme_setting.category, "frontend");
    assert_eq!(theme_setting.value_kind, SettingValueKind::Json);
    assert_eq!(theme_setting.metadata["schema_version"], json!(1));
    assert_eq!(theme_setting.value["schemaVersion"], json!(1));
    assert_eq!(
        theme_setting.value["shellBackground"],
        json!("network-mesh")
    );
    assert_eq!(theme_setting.value["backgroundBrightness"], json!(70));
    assert_eq!(theme_setting.value["accentColor"], json!("teal"));
    assert_eq!(theme_setting.value["panelOpacity"], json!(70));
    assert_eq!(theme_setting.value["panelBlur"], json!(12));
    assert!(theme_setting.is_editable);
}

#[tokio::test]
async fn application_settings_include_frontend_ui_state_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = ApplicationSettingsStore::new(database.pool().expect("configured pool").clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    let settings = store.list_settings().await.expect("list settings");

    let ui_state_setting = settings
        .iter()
        .find(|setting| setting.setting_key == "frontend.ui_state")
        .expect("frontend ui state setting");

    assert_eq!(ui_state_setting.category, "frontend");
    assert_eq!(ui_state_setting.value_kind, SettingValueKind::Json);
    assert_eq!(ui_state_setting.metadata["ui_control"], json!("hidden"));
    assert_eq!(ui_state_setting.metadata["schema_version"], json!(1));
    assert_eq!(
        ui_state_setting.metadata["stores_private_content"],
        json!(false)
    );
    assert_eq!(ui_state_setting.value["schemaVersion"], json!(1));
    assert!(ui_state_setting.is_editable);
}

#[tokio::test]
async fn application_settings_update_repairs_missing_declared_setting_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ApplicationSettingsStore::new(pool.clone());
    store
        .repair_declared_settings()
        .await
        .expect("repair settings");

    sqlx::query("DELETE FROM application_settings WHERE setting_key = 'frontend.theme'")
        .execute(&pool)
        .await
        .expect("delete declared setting");

    let updated = store
        .update_setting_value(
            "frontend.theme",
            &json!({
                "schemaVersion": 1,
                "shellBackground": "forest-network",
                "backgroundBrightness": 60,
                "accentColor": "cyan",
                "panelOpacity": 80,
                "panelBlur": 16
            }),
            "settings-test",
        )
        .await
        .expect("update repairs missing declared setting");

    assert_eq!(updated.setting_key, "frontend.theme");
    assert_eq!(updated.value["shellBackground"], json!("forest-network"));
    assert_eq!(
        updated.updated_by_actor_id.as_deref(),
        Some("settings-test")
    );

    let restored = store
        .setting("frontend.theme")
        .await
        .expect("fetch repaired setting")
        .expect("frontend theme setting restored");
    assert_eq!(restored.value["accentColor"], json!("cyan"));

    store
        .update_setting_value(
            "frontend.theme",
            &json!({
                "schemaVersion": 1,
                "shellBackground": "network-mesh",
                "backgroundBrightness": 70,
                "accentColor": "teal",
                "panelOpacity": 70,
                "panelBlur": 12
            }),
            "settings-test",
        )
        .await
        .expect("restore frontend theme default");
}

#[tokio::test]
async fn database_startup_repairs_declared_application_settings_against_postgres() {
    let _guard = SETTINGS_DB_TEST_LOCK.lock().await;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    sqlx::query("DELETE FROM application_settings WHERE setting_key = 'frontend.api_base_url'")
        .execute(&pool)
        .await
        .expect("delete declared setting");
    sqlx::query(
        r#"
        UPDATE application_settings
        SET
            value = '"broken"'::jsonb,
            label = 'Broken density',
            metadata = '{}'::jsonb
        WHERE setting_key = 'ui.density'
        "#,
    )
    .execute(&pool)
    .await
    .expect("corrupt declared setting");
    sqlx::query(
        r#"
        INSERT INTO application_settings (
            setting_key,
            category,
            value_kind,
            value,
            label,
            description,
            metadata
        )
        VALUES (
            'custom.unexpected',
            'custom',
            'string',
            '"manual"'::jsonb,
            'Manual custom setting',
            'This row must not become a supported setting surface.',
            '{}'::jsonb
        )
        ON CONFLICT (setting_key) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert undeclared setting");

    drop(pool);
    drop(database);

    let repaired_database = Database::connect(Some(&database_url))
        .await
        .expect("database reconnect repairs settings");
    let store =
        ApplicationSettingsStore::new(repaired_database.pool().expect("configured pool").clone());
    let settings = store.list_settings().await.expect("list repaired settings");

    let api_base_url_setti
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

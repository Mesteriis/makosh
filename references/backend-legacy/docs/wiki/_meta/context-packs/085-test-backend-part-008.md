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

- Chunk ID / ID чанка: `085-test-backend-part-008`
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

### `backend/tests/messages/workflow.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/messages/workflow.rs`
- Size bytes / Размер в байтах: `10136`
- Included characters / Включено символов: `10136`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::communications::messages::{
    LocalMessageState, WorkflowState, WorkflowStateCount, project_raw_email_message,
};

use super::support::{
    live_projection_context, record_raw_email_message, store_provider_account, unique_suffix,
};

#[test]
fn workflow_state_from_str_all_valid() {
    for (input, expected) in [
        ("new", WorkflowState::New),
        ("reviewed", WorkflowState::Reviewed),
        ("needs_action", WorkflowState::NeedsAction),
        ("waiting", WorkflowState::Waiting),
        ("done", WorkflowState::Done),
        ("archived", WorkflowState::Archived),
        ("muted", WorkflowState::Muted),
        ("spam", WorkflowState::Spam),
    ] {
        let state = input.parse::<WorkflowState>().expect("valid state");
        assert_eq!(state, expected, "from_str({input}) should match");
    }
}

#[test]
fn workflow_state_from_str_invalid() {
    assert!("".parse::<WorkflowState>().is_err());
    assert!("invalid_state".parse::<WorkflowState>().is_err());
    assert!("NEW".parse::<WorkflowState>().is_err());
}

#[test]
fn workflow_state_as_str_roundtrips() {
    let states = [
        WorkflowState::New,
        WorkflowState::Reviewed,
        WorkflowState::NeedsAction,
        WorkflowState::Waiting,
        WorkflowState::Done,
        WorkflowState::Archived,
        WorkflowState::Muted,
        WorkflowState::Spam,
    ];

    for state in &states {
        let s = state.as_str();
        let roundtripped = s.parse::<WorkflowState>().expect("roundtrip");
        assert_eq!(*state, roundtripped, "roundtrip for {s}");
    }
}

#[test]
fn workflow_state_valid_transitions() {
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Reviewed
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::NeedsAction
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Archived
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Muted
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Spam
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Reviewed,
        &WorkflowState::New
    ));

    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Done
    ));
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::Waiting
    ));

    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Done
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Waiting
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::NeedsAction,
        &WorkflowState::Archived
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Spam,
        &WorkflowState::New
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Done,
        &WorkflowState::Archived
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Archived,
        &WorkflowState::Reviewed
    ));
    assert!(WorkflowState::is_valid_transition(
        &WorkflowState::Archived,
        &WorkflowState::NeedsAction
    ));

    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::New,
        &WorkflowState::New
    ));
    assert!(!WorkflowState::is_valid_transition(
        &WorkflowState::Done,
        &WorkflowState::Done
    ));
}

#[test]
fn workflow_state_serde_roundtrips() {
    let json = serde_json::to_string(&WorkflowState::NeedsAction).expect("serialize");
    assert_eq!(json, "\"needs_action\"");

    let deserialized: WorkflowState =
        serde_json::from_str("\"needs_action\"").expect("deserialize");
    assert_eq!(deserialized, WorkflowState::NeedsAction);

    let deserialized_new: WorkflowState = serde_json::from_str("\"new\"").expect("deserialize");
    assert_eq!(deserialized_new, WorkflowState::New);
}

#[tokio::test]
async fn message_workflow_state_transition_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("workflow state transition").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_workflow_{suffix}");
    let raw_record_id = format!("raw_workflow_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Workflow Gmail",
        format!("workflow-{suffix}@example.com"),
    )
    .await;
    let raw = record_raw_email_message(
        &communication_store,
        &account_id,
        &raw_record_id,
        &format!("provider-workflow-{suffix}"),
        "Workflow test subject",
        "Workflow test body",
    )
    .await;

    let projected = project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message");
    assert_eq!(projected.workflow_state.as_str(), "new");

    let updated = message_store
        .transition_workflow_state(&projected.message_id, WorkflowState::NeedsAction)
        .await
        .expect("transition to needs_action");
    assert_eq!(updated.workflow_state.as_str(), "needs_action");

    let updated = message_store
        .transition_workflow_state(&updated.message_id, WorkflowState::Done)
        .await
        .expect("transition to done");
    assert_eq!(updated.workflow_state.as_str(), "done");

    let updated = message_store
        .transition_workflow_state(&updated.message_id, WorkflowState::Archived)
        .await
        .expect("transition to archived");
    assert_eq!(updated.workflow_state.as_str(), "archived");

    let fetched = message_store
        .message(&projected.message_id)
        .await
        .expect("fetch message")
        .expect("message exists");
    assert_eq!(fetched.workflow_state.as_str(), "archived");
}

#[tokio::test]
async fn message_state_counts_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message state counts").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_counts_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Counts Gmail",
        format!("counts-{suffix}@example.com"),
    )
    .await;

    for i in 0..2 {
        let raw = record_raw_email_message(
            &communication_store,
            &account_id,
            &format!("raw_counts_{suffix}_{i}"),
            &format!("provider-counts-{suffix}-{i}"),
            &format!("Counts subject {i}"),
            &format!("Counts body {i}"),
        )
        .await;
        project_raw_email_message(&message_store, &raw)
            .await
            .expect("project message");
    }

    let counts = message_store
        .count_messages_by_state(Some(&account_id))
        .await
        .expect("count messages");

    let new_count = counts
        .iter()
        .find(|c| c.state.as_str() == "new")
        .map(|c| c.count)
        .unwrap_or(0);
    assert!(new_count >= 2, "expected at least 2 new messages");

    let messages = message_store
        .list_messages(
            Some(&account_id),
            None,
            None,
            None,
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list messages");
    assert!(!messages.is_empty());

    message_store
        .transition_workflow_state(&messages[0].message.message_id, WorkflowState::Done)
        .await
        .expect("transition to done");

    let counts = message_store
        .count_messages_by_state(Some(&account_id))
        .await
        .expect("count messages after transition");

    let done_count = counts
        .iter()
        .find(|c| c.state.as_str() == "done")
        .map(|c| c.count)
        .unwrap_or(0);
    assert_eq!(done_count, 1, "expected 1 done message");
}

#[tokio::test]
async fn message_list_filtering_by_state_against_postgres() {
    let Some((_, communication_store, message_store)) =
        live_projection_context("message list filtering").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let account_id = format!("acct_filter_{suffix}");

    store_provider_account(
        &communication_store,
        &account_id,
        "Filter Gmail",
        format!("filter-{suffix}@example.com"),
    )
    .await;

    for i in 0..3 {
        let raw = record_raw_email_message(
            &communication_store,
            &account_id,
            &format!("raw_filter_{suffix}_{i}"),
            &format!("provider-filter-{suffix}-{i}"),
            &format!("Filter subject {i}"),
            &format!("Filter body {i}"),
        )
        .await;
        project_raw_email_message(&message_store, &raw)
            .await
            .expect("project message");
    }

    let new_messages = message_store
        .list_messages(
            Some(&account_id),
            Some(WorkflowState::New),
            None,
            None,
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list new messages");
    assert!(new_messages.len() >= 3, "expected at least 3 new messages");

    let done_messages = message_store
        .list_messages(
            Some(&account_id),
            Some(WorkflowState::Done),
            None,
            None,
            LocalMessageState::Active,
            10,
        )
        .await
        .expect("list done messages");
    assert_eq!(done_messages.len(), 0, "expected 0 done messages");
}

#[test]
fn workflow_state_count_serialization() {
    let count = WorkflowStateCount {
        state: WorkflowState::NeedsAction,
        count: 42,
    };
    let json = serde_json::to_value(&count).expect("serialize");
    assert_eq!(json["state"], "needs_action");
    assert_eq!(json["count"], 42);
}
```

### `backend/tests/messages_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/messages_architecture.rs`
- Size bytes / Размер в байтах: `1702`
- Included characters / Включено символов: `1702`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn message_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_message_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "message test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_message_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_message_test_violations(&path, violations);
            continue;
        }
        if !is_message_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_message_test_file(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value.starts_with("messages"))
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/obligation_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/obligation_engine.rs`
- Size bytes / Размер в байтах: `4187`
- Included characters / Включено символов: `4187`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::engines::obligation::{
    ObligationCandidateKind, ObligationEngine, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationExtractionInput, ObligationReviewState,
};

#[test]
fn obligation_engine_detects_owner_promise_from_communication() {
    let input = ObligationExtractionInput::communication(
        "message:proposal-commitment",
        "I will send the revised project proposal by Friday 5pm. Thanks.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    )
    .beneficiary(ObligationEntityKind::Project, "project:v1:client-dossier");

    let result = ObligationEngine::detect_candidates(&input).expect("detect obligation candidates");

    assert_eq!(result.obligations.len(), 1);
    assert_eq!(result.follow_ups.len(), 1);
    assert_eq!(result.task_candidates.len(), 1);

    let candidate = &result.obligations[0];
    assert_eq!(candidate.kind, ObligationCandidateKind::Commitment);
    assert_eq!(candidate.statement, "send the revised project proposal");
    assert_eq!(
        candidate.quote,
        "I will send the revised project proposal by Friday 5pm."
    );
    assert_eq!(candidate.due_text.as_deref(), Some("Friday 5pm"));
    assert_eq!(candidate.condition, None);
    assert_eq!(candidate.confidence, 0.84);
    assert_eq!(candidate.review_state, ObligationReviewState::Suggested);
    assert_eq!(
        candidate.obligated_entity_kind,
        ObligationEntityKind::Persona
    );
    assert_eq!(
        candidate.obligated_entity_id,
        "person:v1:email:owner@example.com"
    );
    assert_eq!(
        candidate.beneficiary_entity_kind,
        Some(ObligationEntityKind::Project)
    );
    assert_eq!(
        candidate.beneficiary_entity_id.as_deref(),
        Some("project:v1:client-dossier")
    );
    assert_eq!(
        candidate.evidence_source_kind,
        ObligationEvidenceSourceKind::Communication
    );
    assert_eq!(candidate.evidence_source_id, "message:proposal-commitment");

    assert_eq!(
        result.task_candidates[0].statement,
        "send the revised project proposal"
    );
    assert_eq!(
        result.follow_ups[0].source_obligation_statement,
        "send the revised project proposal"
    );
}

#[test]
fn obligation_engine_detects_request_to_owner_without_autoconfirming() {
    let input = ObligationExtractionInput::communication(
        "message:agreement-request",
        "Please send the signed agreement before Monday morning.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let result = ObligationEngine::detect_candidates(&input).expect("detect obligation candidates");

    assert_eq!(result.obligations.len(), 1);
    let candidate = &result.obligations[0];
    assert_eq!(candidate.kind, ObligationCandidateKind::Request);
    assert_eq!(candidate.statement, "send the signed agreement");
    assert_eq!(candidate.due_text.as_deref(), Some("Monday morning"));
    assert_eq!(candidate.review_state, ObligationReviewState::Suggested);
    assert_eq!(candidate.confidence, 0.76);
}

#[test]
fn obligation_engine_ignores_deadline_without_commitment_language() {
    let input = ObligationExtractionInput::communication(
        "message:office-hours",
        "The office closes by Friday 5pm. The report was already sent.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let result = ObligationEngine::detect_candidates(&input).expect("detect obligation candidates");

    assert_eq!(result.obligations, Vec::new());
    assert_eq!(result.task_candidates, Vec::new());
    assert_eq!(result.follow_ups, Vec::new());
}

#[test]
fn obligation_engine_rejects_empty_source_evidence_before_detection() {
    let input = ObligationExtractionInput::communication(
        " ",
        "I will send the revised project proposal by Friday 5pm.",
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
    );

    let error =
        ObligationEngine::detect_candidates(&input).expect_err("empty source id must be rejected");

    assert_eq!(error.to_string(), "source_id must not be empty");
}
```

### `backend/tests/obligations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/obligations.rs`
- Size bytes / Размер в байтах: `15058`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use makosh_hub_backend::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationReviewState, ObligationRiskState, ObligationStatus, ObligationStore,
    ObligationStoreError,
};
use makosh_hub_backend::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore,
};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::graph_projection::GraphProjectionService;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[tokio::test]
async fn obligation_store_upserts_evidence_backed_obligation_without_creating_task_against_postgres()
 {
    let Some((pool, obligation_store)) =
        live_obligation_context("evidence backed obligation upsert").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let obligated_persona_id = format!("person:v1:email:owner-{suffix}@example.com");
    let beneficiary_project_id = format!("project:v1:obligation-{suffix}");
    let evidence_source_id = format!("message:obligation:{suffix}");

    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        obligated_persona_id.clone(),
        "Send the revised project proposal",
        0.88,
        ObligationReviewState::UserConfirmed,
    )
    .beneficiary(
        ObligationEntityKind::Project,
        beneficiary_project_id.clone(),
    )
    .condition("before the stakeholder review")
    .risk_state(ObligationRiskState::Watch)
    .metadata(json!({"channel": "email", "scope": "proposal"}));
    let first_evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .quote("I will send the revised project proposal before the review.")
    .confidence(0.91)
    .metadata(json!({"message_part": "body", "revision": 1}));
    let second_evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        evidence_source_id.clone(),
    )
    .quote("Updated evidence for the revised proposal commitment.")
    .confidence(0.93)
    .metadata(json!({"message_part": "body", "revision": 2}));

    let first = obligation_store
        .upsert_with_evidence(&obligation, std::slice::from_ref(&first_evidence))
        .await
        .expect("first obligation upsert");
    let second = obligation_store
        .upsert_with_evidence(&obligation, &[second_evidence])
        .await
        .expect("idempotent obligation upsert");

    assert_eq!(first.obligation_id, second.obligation_id);
    assert_eq!(first.obligated_entity_kind, ObligationEntityKind::Persona);
    assert_eq!(first.obligated_entity_id, obligated_persona_id);
    assert_eq!(
        first.beneficiary_entity_kind,
        Some(ObligationEntityKind::Project)
    );
    assert_eq!(first.beneficiary_entity_id, Some(beneficiary_project_id));
    assert_eq!(first.statement, "Send the revised project proposal");
    assert_eq!(first.status, ObligationStatus::Open);
    assert_eq!(first.review_state, ObligationReviewState::UserConfirmed);
    assert_eq!(first.risk_state, ObligationRiskState::Watch);
    assert_eq!(
        first.condition.as_deref(),
        Some("before the stakeholder review")
    );
    assert_eq!(first.confidence, 0.88);

    let evidence_row = sqlx::query(
        r#"
        SELECT quote, confidence::float8 AS confidence, metadata
        FROM obligation_evidence
        WHERE obligation_id = $1
          AND source_kind = $2
          AND source_id = $3
        "#,
    )
    .bind(&first.obligation_id)
    .bind(ObligationEvidenceSourceKind::Communication.as_str())
    .bind(&evidence_source_id)
    .fetch_one(&pool)
    .await
    .expect("stored obligation evidence");
    let quote: Option<String> = evidence_row.try_get("quote").expect("evidence quote");
    let evidence_confidence: f64 = evidence_row
        .try_get("confidence")
        .expect("evidence confidence");
    let metadata: Value = evidence_row.try_get("metadata").expect("evidence metadata");
    assert_eq!(
        quote.as_deref(),
        Some("Updated evidence for the revised proposal commitment.")
    );
    assert_eq!(evidence_confidence, 0.93);
    assert_eq!(metadata, json!({"message_part": "body", "revision": 2}));

    let listed = obligation_store
        .list_for_entity(
            ObligationEntityKind::Persona,
            &first.obligated_entity_id,
            10,
        )
        .await
        .expect("obligations for obligated persona");
    assert!(
        listed
            .iter()
            .any(|item| item.obligation_id == first.obligation_id)
    );

    GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await
        .expect("project obligation graph");

    let obligation_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'obligation' AND stable_key = $1",
    )
    .bind(&first.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation graph node");
    let obligated_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'person' AND stable_key = $1",
    )
    .bind(&first.obligated_entity_id)
    .fetch_one(&pool)
    .await
    .expect("obligated persona graph node");
    let beneficiary_node_id: String = sqlx::query_scalar(
        "SELECT node_id FROM graph_nodes WHERE node_kind = 'project' AND stable_key = $1",
    )
    .bind(first.beneficiary_entity_id.as_deref().expect("beneficiary"))
    .fetch_one(&pool)
    .await
    .expect("beneficiary project graph node");

    let obligated_edge_id: String = sqlx::query_scalar(
        r#"
        SELECT edge_id
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND review_state = 'user_confirmed'
          AND properties @> $3
          AND valid_to IS NULL
        "#,
    )
    .bind(&obligation_node_id)
    .bind(&obligated_node_id)
    .bind(json!({"domain": "obligation", "link_role": "obligated_entity"}))
    .fetch_one(&pool)
    .await
    .expect("obligation to obligated entity graph edge");
    let beneficiary_edge_id: String = sqlx::query_scalar(
        r#"
        SELECT edge_id
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'entity_relationship'
          AND review_state = 'user_confirmed'
          AND properties @> $3
          AND valid_to IS NULL
        "#,
    )
    .bind(&obligation_node_id)
    .bind(&beneficiary_node_id)
    .bind(json!({"domain": "obligation", "link_role": "beneficiary_entity"}))
    .fetch_one(&pool)
    .await
    .expect("obligation to beneficiary entity graph edge");

    for edge_id in [obligated_edge_id, beneficiary_edge_id] {
        let graph_evidence_row = sqlx::query(
            r#"
            SELECT source_kind, source_id, excerpt, metadata
            FROM graph_evidence
            WHERE edge_id = $1
            "#,
        )
        .bind(edge_id)
        .fetch_one(&pool)
        .await
        .expect("obligation graph evidence");
        let graph_source_kind: String = graph_evidence_row
            .try_get("source_kind")
            .expect("graph evidence source kind");
        let graph_source_id: String = graph_evidence_row
            .try_get("source_id")
            .expect("graph evidence source id");
        let graph_excerpt: Option<String> = graph_evidence_row
            .try_get("excerpt")
            .expect("graph evidence excerpt");
        let graph_evidence_metadata: Value = graph_evidence_row
            .try_get("metadata")
            .expect("graph evidence metadata");

        assert_eq!(graph_source_kind, "obligation");
        assert_eq!(graph_source_id, first.obligation_id);
        assert_eq!(
            graph_excerpt.as_deref(),
            Some("Updated evidence for the revised proposal commitment.")
        );
        assert_eq!(
            graph_evidence_metadata,
            json!({
                "domain": "obligation",
                "source_kind": "communication",
                "source_id": evidence_source_id
            })
        );
    }

    let task_link_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1",
    )
    .bind(&first.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation task link count");
    let task_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tasks WHERE source_id = $1")
            .bind(&evidence_source_id)
            .fetch_one(&pool)
            .await
            .expect("task count for obligation evidence source");

    assert_eq!(task_link_count, 0);
    assert_eq!(task_count, 0);
}

#[tokio::test]
async fn obligation_store_rejects_missing_evidence_before_database_write() {
    let store = disconnected_obligation_store();
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        "Reply with the signed agreement",
        0.8,
        ObligationReviewState::Suggested,
    );

    let error = store
        .upsert_with_evidence(&obligation, &[])
        .await
        .expect_err("obligation without evidence must fail before database write");

    assert!(matches!(error, ObligationStoreError::MissingEvidence));
}

#[tokio::test]
async fn obligation_store_rejects_invalid_confidence_before_database_write() {
    let store = disconnected_obligation_store();
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        "Reply with the signed agreement",
        1.1,
        ObligationReviewState::Suggested,
    );
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        "message:invalid-obligation-confidence",
    );

    let error = store
        .upsert_with_evidence(&obligation, &[evidence])
        .await
        .expect_err("invalid confidence must fail before database write");

    assert!(matches!(
        error,
        ObligationStoreError::InvalidScore("confidence", _)
    ));
}

#[tokio::test]
async fn obligation_store_rejects_partial_beneficiary_before_database_write() {
    let store = disconnected_obligation_store();
    let mut obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        "Reply with the signed agreement",
        0.8,
        ObligationReviewState::Suggested,
    );
    obligation.beneficiary_entity_kind = Some(ObligationEntityKind::Organization);
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        "message:partial-beneficiary",
    );

    let error = store
        .upsert_with_evidence(&obligation, &[evidence])
        .await
        .expect_err("partial beneficiary must fail before database write");

    assert!(matches!(error, ObligationStoreError::PartialBeneficiary));
}

#[tokio::test]
async fn obligation_store_rejects_missing_observation_evidence_against_postgres() {
    let Some((_pool, store)) =
        live_obligation_context("missing obligation observation evidence").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        "person:v1:email:owner@example.com",
        format!("Observation-backed obligation {suffix}"),
        0.8,
        ObligationReviewState::Suggested,
    );
    let evidence =
        NewObligationEvidence::observation(format!("observation:v1:missing-obligation:{suffix}"));

    let error = store
        .upsert_with_evidence(&obligation, &[evidence])

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/obligations_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/obligations_api.rs`
- Size bytes / Размер в байтах: `9821`
- Included characters / Включено символов: `9821`
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
use makosh_hub_backend::domains::obligations::{
    NewObligation, NewObligationEvidence, Obligation, ObligationEntityKind,
    ObligationEvidenceSourceKind, ObligationReviewState, ObligationStore,
};
use makosh_hub_backend::platform::storage::Database;

const LOCAL_API_TOKEN: &str = "obligations-api-test-token";

#[tokio::test]
async fn obligations_list_returns_entity_scoped_obligations() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let obligated_persona_id = format!("person:v1:email:obligation-api-{suffix}@example.com");
    let stored = seed_obligation(&pool, suffix, &obligated_persona_id).await;

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/obligations?entity_kind=persona&entity_id={obligated_persona_id}&limit=10"
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
        .find(|item| item["obligation_id"] == json!(stored.obligation_id))
        .expect("seeded obligation");

    assert_eq!(item["obligated_entity_kind"], "persona");
    assert_eq!(item["obligated_entity_id"], obligated_persona_id);
    assert_eq!(item["statement"], stored.statement);
    assert_eq!(item["review_state"], "suggested");
}

#[tokio::test]
async fn obligations_list_returns_global_suggested_review_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let suggested_persona_id =
        format!("person:v1:email:obligation-global-suggested-{suffix}@example.com");
    let confirmed_persona_id =
        format!("person:v1:email:obligation-global-confirmed-{suffix}@example.com");
    let suggested = seed_obligation_with_review_state(
        &pool,
        suffix,
        &suggested_persona_id,
        ObligationReviewState::Suggested,
    )
    .await;
    let confirmed = seed_obligation_with_review_state(
        &pool,
        suffix + 1,
        &confirmed_persona_id,
        ObligationReviewState::UserConfirmed,
    )
    .await;

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/obligations?review_state=suggested&limit=10",
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
            .any(|item| item["obligation_id"] == json!(suggested.obligation_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["obligation_id"] != json!(confirmed.obligation_id))
    );
    assert!(
        items
            .iter()
            .all(|item| item["review_state"] == json!("suggested"))
    );
}

#[tokio::test]
async fn put_obligation_review_updates_review_state_with_observation_trail() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let (app, pool) = app_and_pool(&database_url).await;
    let suffix = unique_suffix();
    let obligated_persona_id = format!("person:v1:email:obligation-review-{suffix}@example.com");
    let stored = seed_obligation(&pool, suffix, &obligated_persona_id).await;
    let obligation_id = path_segment(&stored.obligation_id);

    let response = app
        .oneshot(json_put_request(
            &format!("/api/v1/obligations/{obligation_id}/review"),
            json!({
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["obligation_id"], stored.obligation_id);
    assert_eq!(body["review_state"], "user_confirmed");

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM obligations WHERE obligation_id = $1")
            .bind(&stored.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("stored review state");
    let link_row = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'obligations'
           AND entity_kind = 'obligation'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&stored.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation observation link");
    let observation_id: String = link_row.try_get("observation_id").expect("observation id");
    let metadata: Value = link_row.try_get("metadata").expect("link metadata");
    let task_link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(&stored.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("task link count");

    assert_eq!(review_state, "user_confirmed");
    assert_eq!(metadata["review_state"], "user_confirmed");
    assert_eq!(task_link_count, 0);

    let observation_row =
        sqlx::query("SELECT origin_kind, payload FROM observations WHERE observation_id = $1")
            .bind(&observation_id)
            .fetch_one(&pool)
            .await
            .expect("obligation observation");
    let origin_kind: String = observation_row.try_get("origin_kind").expect("origin kind");
    let payload: Value = observation_row.try_get("payload").expect("payload");
    assert_eq!(origin_kind, "manual");
    assert_eq!(payload["obligation_id"], json!(stored.obligation_id));
    assert_eq!(payload["review_state"], "user_confirmed");

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'obligation_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&stored.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "obligation");
    assert_eq!(review_item.2, stored.obligation_id);
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

async fn seed_obligation(pool: &PgPool, suffix: u128, obligated_persona_id: &str) -> Obligation {
    seed_obligation_with_review_state(
        pool,
        suffix,
        obligated_persona_id,
        ObligationReviewState::Suggested,
    )
    .await
}

async fn seed_obligation_with_review_state(
    pool: &PgPool,
    suffix: u128,
    obligated_persona_id: &str,
    review_state: ObligationReviewState,
) -> Obligation {
    let obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        obligated_persona_id,
        format!("Send obligation API evidence package {suffix}"),
        0.82,
        review_state,
    )
    .metadata(json!({"source": "obligations_api_test"}));
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Communication,
        format!("message:obligation-api:{suffix}"),
    )
    .quote("I will send the obligation API evidence package.")
    .confidence(0.9)
    .metadata(json!({"source": "obligations_api_test"}));

    ObligationStore::new(pool.clone())
        .upsert_with_evidence(&obligation, &[evidence])
        .await
        .expect("seed obligation")
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

### `backend/tests/observations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/observations.rs`
- Size bytes / Размер в байтах: `18461`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::{TimeZone, Utc};
use makosh_hub_backend::platform::observations::{
    NewObservation, NewObservationIngestionRun, NewObservationLink, ObservationIngestionRunStatus,
    ObservationOriginKind, ObservationStore,
};
use makosh_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

#[tokio::test]
async fn manual_capture_creates_observation_without_vault_source_against_postgres() {
    let Some((pool, store)) = live_observation_context("manual capture without vault").await else {
        return;
    };
    let suffix = unique_suffix();
    let observed_at = Utc.with_ymd_and_hms(2026, 6, 18, 8, 30, 0).unwrap();

    let stored = store
        .capture(
            &NewObservation::new(
                "VOICE_RECORDING",
                ObservationOriginKind::Manual,
                observed_at,
                json!({
                    "transcript": "Record the Макошь evidence architecture decision.",
                    "duration_seconds": 42
                }),
                format!("manual://voice-memo/{suffix}"),
            )
            .confidence(0.96)
            .provenance(json!({
                "captured_by": "manual_voice_memo",
                "device": "local_desktop"
            })),
        )
        .await
        .expect("capture manual observation");

    assert_eq!(stored.kind_code, "VOICE_RECORDING");
    assert_eq!(stored.origin_kind, ObservationOriginKind::Manual);
    assert_eq!(stored.vault_source_id, None);
    assert!(stored.content_hash.starts_with("sha256:"));

    let row = sqlx::query(
        r#"
        SELECT payload, provenance
        FROM observations
        WHERE observation_id = $1
          AND vault_source_id IS NULL
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("stored observation row");
    let payload: Value = row.try_get("payload").expect("payload");
    let provenance: Value = row.try_get("provenance").expect("provenance");

    assert_eq!(
        payload["transcript"],
        json!("Record the Макошь evidence architecture decision.")
    );
    assert_eq!(provenance["captured_by"], json!("manual_voice_memo"));

    let event_row = sqlx::query(
        r#"
        SELECT event_id, correlation_id, causation_id, subject
        FROM event_log
        WHERE event_type = 'observation.captured.v1'
          AND subject ->> 'observation_id' = $1
        "#,
    )
    .bind(&stored.observation_id)
    .fetch_one(&pool)
    .await
    .expect("observation captured event row");
    let event_id: String = event_row.try_get("event_id").expect("event_id");
    let correlation_id: Option<String> =
        event_row.try_get("correlation_id").expect("correlation_id");
    let causation_id: Option<String> = event_row.try_get("causation_id").expect("causation_id");
    let subject: Value = event_row.try_get("subject").expect("subject");

    assert_eq!(
        event_id,
        format!("event:v1:observation-captured:{}", stored.observation_id)
    );
    assert_eq!(
        correlation_id.as_deref(),
        Some(stored.observation_id.as_str())
    );
    assert_eq!(causation_id, None);
    assert_eq!(subject["kind"], json!("observation"));
    assert_eq!(subject["entity_id"], json!(stored.observation_id));
    assert_eq!(subject["observation_id"], json!(stored.observation_id));
    assert_eq!(subject["observation_kind"], json!("VOICE_RECORDING"));
}

#[tokio::test]
async fn manual_note_creates_observation_without_vault_source_against_postgres() {
    let Some((_pool, store)) = live_observation_context("manual note without vault").await else {
        return;
    };
    let suffix = unique_suffix();

    let stored = store
        .capture(
            &NewObservation::new(
                "DOCUMENT",
                ObservationOriginKind::Manual,
                Utc.with_ymd_and_hms(2026, 6, 18, 8, 45, 0).unwrap(),
                json!({
                    "title": format!("Manual note {suffix}"),
                    "body": "Create note should also land in canonical evidence."
                }),
                format!("manual://note/{suffix}"),
            )
            .confidence(0.91)
            .provenance(json!({
                "captured_by": "manual_note"
            })),
        )
        .await
        .expect("capture manual note observation");

    assert_eq!(stored.kind_code, "DOCUMENT");
    assert_eq!(stored.origin_kind, ObservationOriginKind::Manual);
    assert_eq!(stored.vault_source_id, None);
}

#[tokio::test]
async fn observations_are_append_only_and_survive_provider_deletion_against_postgres() {
    let Some((pool, store)) = live_observation_context("append-only deletion").await else {
        return;
    };
    let suffix = unique_suffix();
    let source_ref = format!("gmail://account/{suffix}/message/provider-{suffix}");
    let observed_at = Utc.with_ymd_and_hms(2026, 6, 18, 9, 0, 0).unwrap();

    let imported = store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::VaultSource,
                observed_at,
                json!({
                    "provider": "gmail",
                    "message_id": format!("provider-{suffix}"),
                    "subject": "Evidence store"
                }),
                source_ref.clone(),
            )
            .vault_source_id(format!("vault_source:gmail:{suffix}"))
            .confidence(0.99),
        )
        .await
        .expect("capture provider message");

    let update_error =
        sqlx::query("UPDATE observations SET payload = $1 WHERE observation_id = $2")
            .bind(json!({"mutated": true}))
            .bind(&imported.observation_id)
            .execute(&pool)
            .await
            .expect_err("observation update must be blocked");
    assert!(
        update_error.to_string().contains("append-only"),
        "unexpected update error: {update_error}"
    );

    let delete_error = sqlx::query("DELETE FROM observations WHERE observation_id = $1")
        .bind(&imported.observation_id)
        .execute(&pool)
        .await
        .expect_err("observation delete must be blocked");
    assert!(
        delete_error.to_string().contains("append-only"),
        "unexpected delete error: {delete_error}"
    );

    let deletion = store
        .capture(
            &NewObservation::new(
                "COMMUNICATION_MESSAGE_DELETED",
                ObservationOriginKind::VaultSource,
                observed_at,
                json!({
                    "provider": "gmail",
                    "message_id": format!("provider-{suffix}"),
                    "deletion_observed": true
                }),
                source_ref.clone(),
            )
            .vault_source_id(format!("vault_source:gmail:{suffix}"))
            .confidence(0.93),
        )
        .await
        .expect("capture provider deletion observation");

    assert_ne!(imported.observation_id, deletion.observation_id);

    let rows = sqlx::query(
        r#"
        SELECT kind.code
        FROM observations observation
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE observation.source_ref = $1
        ORDER BY observation.captured_at ASC
        "#,
    )
    .bind(&source_ref)
    .fetch_all(&pool)
    .await
    .expect("observations for source ref");
    let codes: Vec<String> = rows
        .into_iter()
        .map(|row| row.try_get("code").expect("kind code"))
        .collect();

    assert_eq!(
        codes,
        vec!["COMMUNICATION_MESSAGE", "COMMUNICATION_MESSAGE_DELETED"]
    );
}

#[tokio::test]
async fn observation_platform_persists_links_and_ingestion_runs_against_postgres() {
    let Some((_pool, store)) = live_observation_context("observation links and runs").await else {
        return;
    };
    let suffix = unique_suffix();

    let observation = store
        .capture(
            &NewObservation::new(
                "MEETING_TRANSCRIPT",
                ObservationOriginKind::FileImport,
                Utc.with_ymd_and_hms(2026, 6, 18, 9, 30, 0).unwrap(),
                json!({
                    "meeting_id": format!("meeting:v1:{suffix}"),
                    "transcript": "Action: prepare NAS purchase context."
                }),
                format!("import://meeting-transcript/{suffix}"),
            )
            .confidence(0.92),
        )
        .await
        .expect("capture imported transcript observation");

    let link = store
        .upsert_link(
            &NewObservationLink::new(
                observation.observation_id.clone(),
                "meetings",
                "meeting",
                format!("meeting:v1:{suffix}"),
            )
            .relationship_kind("evidence_for")
            .confidence(0.88)
            .metadata(json!({
                "linked_by": "ingestion_test"
            })),
        )
        .await
        .expect("upsert observation link");
    assert_eq!(link.domain, "meetings");

    let links = store
        .list_links(&observation.observation_id)
        .await
        .expect("list observation links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].entity_kind, "meeting");

    let started = store
        .start_ingestion_run(&NewObservationIngestionRun::new(
            format!("ingestion-run:v1:{suffix}"),
            observation.observation_id.clone(),
            "meetings/transcript-ingestion",
        ))
        .await
        .expect("start ingestion run");
    assert_eq!(started.status, ObservationIngestionRunStatus::Running);

    let finished = store
        .finish_ingestion_run(
            &started.ingestion_run_id,
            ObservationIngestionRunStatus::Succeeded,
            &json!({
                "produced": ["meeting", "task_candidate", "knowledge_candidate"]
            }),
            None,
        )
        .await
        .expect("finish ingestion run");
    assert_eq!(finished.status, ObservationIngestionRunStatus::Succeeded);
    assert!(finished.finished_at.is_some());

    let runs = store
        .list_ingestion_runs(&observation.observation_id)
        .await
        .expect("list ingestion runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].pipeline, "meetings/transcript-ingestion");
    assert_eq!(runs[0].output["produced"][0], json!("meeting"));
}

#[tokio::test]
async fn canonical_observation_kind_definitions_are_seeded_against_postgres() {
    let Some((_pool, store)) = live_observation_context("canonical kind definitions").await else {
        return;
    };

    let definitions = store
        .list_kind_definitions()
        .await
        .expect("list observation kind definitions");
    let codes: HashSet<&str> = definitions
        .iter()
        .map(|definition| definition.code.as_str())
        .collect();

    for required in [
        "COMMUNICATION_MESSAGE",
        "COMMUNICATION_DRAFT",
        "COMMUNICATION_FOLDER",
        "COMMUNICATION_SAVED_SEARCH",
        "COMMUNICATION_OUTBOX",
        "COMMUNICATION_DELIVERY_STATUS",
        "COMMUNICATION_READ_RECEIPT",
        "CONTRADICTION_OBSERVATION",
        "COMMUNICATION_MESSAGE_DELETED",
        "COMMUNICATION_ATTACHMENT",
        "MEETING",
        "MEETING_RECORDING",
        "MEETING_TRANSCRIPT",
        "DOCUMENT",
        "VOICE_RECORDING",
        "BROWSER_CAPTURE",
        "CONTACT_RECORD",
        "CALENDAR_EVENT",
        "CALENDAR_EVENT_DELETED",
    ] {
        assert!(
            codes.contains(required),
            "missing kind definition {required}"
        );
    }
}

#[tokio::test]
async fn browser_capture_creates_observation_without_vault_source_against_postgres() {
    let Some((_pool, store)) = live_obse
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/ollama.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/ollama.rs`
- Size bytes / Размер в байтах: `5764`
- Included characters / Включено символов: `5764`
- Truncated / Обрезано: `no`

```rust
use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::net::TcpListener;

use makosh_hub_backend::integrations::ollama::client::{
    OllamaClient, OllamaClientConfig, OllamaError,
};

#[tokio::test]
async fn ollama_client_round_trips_chat_embed_tags_and_version() {
    let base_url = spawn_fake_ollama(FakeOllamaMode::Ok).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(base_url, "qwen3:4b", "qwen3-embedding:4b").with_timeout_seconds(5),
    )
    .expect("client");

    let version = client.version().await.expect("version");
    assert_eq!(version, "0.17.4");

    let tags = client.tags().await.expect("tags");
    assert!(tags.contains(&"qwen3:4b".to_owned()));
    assert!(tags.contains(&"qwen3-embedding:4b".to_owned()));

    let chat = client
        .chat("Return exactly: makosh-ai-ok")
        .await
        .expect("chat");
    assert_eq!(chat.content, "makosh-ai-ok");
    assert_eq!(chat.model, "qwen3:4b");

    let embedding = client
        .embed("Макошь memory retrieval")
        .await
        .expect("embed");
    assert_eq!(embedding.model, "qwen3-embedding:4b");
    assert_eq!(embedding.embedding.len(), 2560);
}

#[tokio::test]
async fn ollama_client_strips_qwen_thinking_blocks_from_chat_content() {
    let base_url = spawn_fake_ollama(FakeOllamaMode::ThinkingContent).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(base_url, "qwen3:4b", "qwen3-embedding:4b").with_timeout_seconds(5),
    )
    .expect("client");

    let chat = client.chat("answer from sources").await.expect("chat");
    assert_eq!(chat.content, "Final cited answer.");
}

#[tokio::test]
async fn ollama_client_reports_missing_models_and_malformed_json() {
    let missing_url = spawn_fake_ollama(FakeOllamaMode::MissingModels).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(missing_url, "qwen3:4b", "qwen3-embedding:4b")
            .with_timeout_seconds(5),
    )
    .expect("client");
    let error = client
        .validate_required_models()
        .await
        .expect_err("missing models");
    assert!(matches!(error, OllamaError::MissingModel { .. }));

    let malformed_url = spawn_fake_ollama(FakeOllamaMode::MalformedJson).await;
    let client = OllamaClient::new(
        OllamaClientConfig::new(malformed_url, "qwen3:4b", "qwen3-embedding:4b")
            .with_timeout_seconds(5),
    )
    .expect("client");
    let error = client.chat("hello").await.expect_err("malformed response");
    assert!(matches!(error, OllamaError::Protocol(_)));
}

#[derive(Clone, Copy)]
enum FakeOllamaMode {
    Ok,
    ThinkingContent,
    MissingModels,
    MalformedJson,
}

async fn spawn_fake_ollama(mode: FakeOllamaMode) -> String {
    let app = Router::new()
        .route(
            "/api/version",
            get(|| async { Json(json!({ "version": "0.17.4" })) }),
        )
        .route(
            "/api/tags",
            get(move || async move {
                let models = match mode {
                    FakeOllamaMode::MissingModels => vec![json!({ "name": "llama3.2:3b" })],
                    _ => vec![
                        json!({ "name": "qwen3:4b" }),
                        json!({ "name": "qwen3-embedding:4b" }),
                    ],
                };
                Json(json!({ "models": models }))
            }),
        )
        .route(
            "/api/chat",
            post(move |Json(_body): Json<Value>| async move {
                match mode {
                    FakeOllamaMode::MalformedJson => (
                        StatusCode::OK,
                        Json(json!({ "model": "qwen3:4b", "message": {} })),
                    ),
                    FakeOllamaMode::ThinkingContent => (
                        StatusCode::OK,
                        Json(json!({
                            "model": "qwen3:4b",
                            "message": {
                                "role": "assistant",
                                "content": "<think>private chain of thought</think>\nFinal cited answer."
                            },
                            "done": true,
                            "total_duration": 10_000_000u64,
                            "prompt_eval_count": 8u32,
                            "eval_count": 3u32
                        })),
                    ),
                    _ => (
                        StatusCode::OK,
                        Json(json!({
                            "model": "qwen3:4b",
                            "message": { "role": "assistant", "content": "makosh-ai-ok" },
                            "done": true,
                            "total_duration": 10_000_000u64,
                            "prompt_eval_count": 8u32,
                            "eval_count": 3u32
                        })),
                    ),
                }
            }),
        )
        .route(
            "/api/embed",
            post(move |Json(_body): Json<Value>| async move {
                let embedding = vec![0.001_f32; 2560];
                Json(json!({
                    "model": "qwen3-embedding:4b",
                    "embeddings": [embedding],
                    "total_duration": 10_000_000u64,
                    "prompt_eval_count": 4u32
                }))
            }),
        );

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake ollama");
    });

    format!("http://{address}")
}
```

### `backend/tests/omniroute.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/omniroute.rs`
- Size bytes / Размер в байтах: `7423`
- Included characters / Включено символов: `7423`
- Truncated / Обрезано: `no`

```rust
use std::net::SocketAddr;

use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use makosh_hub_backend::integrations::omniroute::client::{
    OmniRouteClient, OmniRouteClientConfig, OmniRouteError,
};
use makosh_hub_backend::platform::secrets::ResolvedSecret;
use serde_json::{Value, json};
use tokio::net::TcpListener;

#[tokio::test]
async fn omniroute_client_round_trips_openai_compatible_models_chat_and_embeddings() {
    let base_url = spawn_fake_omniroute(FakeOmniRouteMode::Ok).await;
    let client = OmniRouteClient::new(
        OmniRouteClientConfig::new(
            base_url,
            "codex/gpt-5.5",
            "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
            ResolvedSecret::new("test-omniroute-key").expect("secret"),
        )
        .with_timeout_seconds(5),
    )
    .expect("client");

    let models = client.models().await.expect("models");
    assert!(models.contains(&"codex/gpt-5.5".to_owned()));
    assert!(models.contains(&"openai-compatible-chat-ollama-pve/qwen3-embedding:4b".to_owned()));
    client
        .validate_required_models()
        .await
        .expect("required models");

    let chat = client
        .chat("Return exactly: makosh-omniroute-ok")
        .await
        .expect("chat");
    assert_eq!(chat.content, "makosh-omniroute-ok");
    assert_eq!(chat.model, "codex/gpt-5.5");

    let embedding = client
        .embed("Макошь source-backed retrieval")
        .await
        .expect("embedding");
    assert_eq!(
        embedding.model,
        "openai-compatible-chat-ollama-pve/qwen3-embedding:4b"
    );
    assert_eq!(embedding.embedding.len(), 2560);
}

#[tokio::test]
async fn omniroute_client_reports_auth_missing_models_and_malformed_json() {
    let unauthorized_url = spawn_fake_omniroute(FakeOmniRouteMode::Unauthorized).await;
    let client = omniroute_client(unauthorized_url);
    let error = client.models().await.expect_err("unauthorized response");
    assert!(matches!(error, OmniRouteError::Endpoint { status: 401 }));

    let missing_url = spawn_fake_omniroute(FakeOmniRouteMode::MissingModels).await;
    let client = omniroute_client(missing_url);
    let error = client
        .validate_required_models()
        .await
        .expect_err("missing models");
    assert!(matches!(error, OmniRouteError::MissingModel { .. }));

    let malformed_url = spawn_fake_omniroute(FakeOmniRouteMode::MalformedJson).await;
    let client = omniroute_client(malformed_url);
    let error = client.chat("hello").await.expect_err("malformed response");
    assert!(matches!(error, OmniRouteError::Protocol(_)));
}

#[derive(Clone, Copy)]
enum FakeOmniRouteMode {
    Ok,
    Unauthorized,
    MissingModels,
    MalformedJson,
}

fn omniroute_client(base_url: String) -> OmniRouteClient {
    OmniRouteClient::new(
        OmniRouteClientConfig::new(
            base_url,
            "codex/gpt-5.5",
            "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
            ResolvedSecret::new("test-omniroute-key").expect("secret"),
        )
        .with_timeout_seconds(5),
    )
    .expect("client")
}

async fn spawn_fake_omniroute(mode: FakeOmniRouteMode) -> String {
    let app = Router::new()
        .route(
            "/v1/models",
            get(move |headers: HeaderMap| async move {
                if !authorized(&headers) || matches!(mode, FakeOmniRouteMode::Unauthorized) {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "unauthorized"})),
                    );
                }
                let models = match mode {
                    FakeOmniRouteMode::MissingModels => {
                        vec![json!({ "id": "openrouter/openrouter/free" })]
                    }
                    _ => vec![
                        json!({ "id": "codex/gpt-5.5" }),
                        json!({ "id": "openai-compatible-chat-ollama-pve/qwen3-embedding:4b" }),
                    ],
                };
                (
                    StatusCode::OK,
                    Json(json!({ "object": "list", "data": models })),
                )
            }),
        )
        .route(
            "/v1/chat/completions",
            post(
                move |headers: HeaderMap, Json(_body): Json<Value>| async move {
                    if !authorized(&headers) || matches!(mode, FakeOmniRouteMode::Unauthorized) {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "unauthorized"})),
                        );
                    }
                    match mode {
                        FakeOmniRouteMode::MalformedJson => (
                            StatusCode::OK,
                            Json(json!({
                                "id": "chatcmpl_fake",
                                "model": "codex/gpt-5.5",
                                "choices": [{ "message": {} }]
                            })),
                        ),
                        _ => (
                            StatusCode::OK,
                            Json(json!({
                                "id": "chatcmpl_fake",
                                "model": "codex/gpt-5.5",
                                "choices": [
                                    {
                                        "index": 0,
                                        "message": {
                                            "role": "assistant",
                                            "content": "<think>hidden</think>\nmakosh-omniroute-ok"
                                        }
                                    }
                                ]
                            })),
                        ),
                    }
                },
            ),
        )
        .route(
            "/v1/embeddings",
            post(
                move |headers: HeaderMap, Json(_body): Json<Value>| async move {
                    if !authorized(&headers) || matches!(mode, FakeOmniRouteMode::Unauthorized) {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "unauthorized"})),
                        );
                    }
                    (
                        StatusCode::OK,
                        Json(json!({
                            "model": "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
                            "data": [
                                {
                                    "index": 0,
                                    "embedding": vec![0.002_f32; 2560]
                                }
                            ]
                        })),
                    )
                },
            ),
        );

    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("listener");
    let address = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fake omniroute");
    });

    format!("http://{address}/v1")
}

fn authorized(headers: &HeaderMap) -> bool {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        == Some("Bearer test-omniroute-key")
}
```

### `backend/tests/organizations_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/organizations_api.rs`
- Size bytes / Размер в байтах: `22340`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use testkit::factories::contact::ContactFactory;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::organizations::enrichment::OrgEnrichmentStore;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::storage::Database;

const T: &str = "orgs-test-token";

fn cfg() -> AppConfig {
    testkit::app::config_with_secret(T)
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("req")
}
fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}
fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn jb(r: axum::response::Response) -> Value {
    let b = to_bytes(r.into_body(), usize::MAX).await.expect("b");
    serde_json::from_slice(&b).expect("json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

fn enc(v: &str) -> String {
    url::form_urlencoded::byte_serialize(v.as_bytes()).collect()
}

async fn router(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        testkit::app::config_with_secret_and_database_url(T, db),
        database,
    )
}

// Helper: create org, return org_id
async fn mkorg(app: &axum::Router, s: u128) -> String {
    let r = app
        .clone()
        .oneshot(post(
            "/api/v1/organizations",
            json!({"display_name": format!("T{s}"), "org_type": "technology"}),
        ))
        .await
        .expect("r");
    if r.status().is_success() {
        jb(r).await["organization_id"]
            .as_str()
            .unwrap_or("org:unknown")
            .to_owned()
    } else {
        format!("org:bad:{s}")
    }
}

// ── Auth ───────────────────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_auth_reject() {
    let r = build_router(cfg());
    let resp = r
        .oneshot(
            Request::builder()
                .uri("/api/v1/organizations")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("r");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── CRUD ───────────────────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_crud() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;

    // Create
    let r = a
        .clone()
        .oneshot(post(
            "/api/v1/organizations",
            json!({"display_name": format!("A{s}"), "org_type": "technology"}),
        ))
        .await
        .expect("r");
    assert!(r.status().is_success(), "create={}", r.status());
    let oid = jb(r).await["organization_id"].as_str().unwrap().to_owned();

    // Get
    let r = a
        .clone()
        .oneshot(get(&format!("/api/v1/organizations/{}", enc(&oid))))
        .await
        .expect("r");
    assert!(r.status().is_success(), "get={}", r.status());

    // Update
    let r = a
        .clone()
        .oneshot(put(
            &format!("/api/v1/organizations/{}", enc(&oid)),
            json!({"display_name": format!("U{s}")}),
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "update={}", r.status());

    // Archive
    let r = a
        .oneshot(post(
            &format!("/api/v1/organizations/{}/archive", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("r");
    assert!(r.status().is_success(), "archive={}", r.status());
}

#[tokio::test]
async fn orgs_list() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    mkorg(&a, s).await;
    let r = a.oneshot(get("/api/v1/organizations")).await.expect("r");
    assert!(r.status().is_success(), "list={}", r.status());
}

#[tokio::test]
async fn orgs_search() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let a = router(&db).await;
    let r = a
        .oneshot(get("/api/v1/organizations/search?q=test"))
        .await
        .expect("r");
    assert!(r.status().is_success(), "search={}", r.status());
}

#[tokio::test]
async fn orgs_not_found_404() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    let r = a
        .oneshot(get(&format!("/api/v1/organizations/org:nx{s}")))
        .await
        .expect("r");
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ── Sub-resource read endpoints ────────────────────────────────────────────
macro_rules! org_test {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let s = uid();
            let a = router(&db).await;
            let oid = mkorg(&a, s).await;
            let r = a.oneshot(get(&format!($path, enc(&oid)))).await.expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
            );
        }
    };
}

org_test!(orgs_identities, "/api/v1/organizations/{}/identities");
org_test!(orgs_aliases, "/api/v1/organizations/{}/aliases");
org_test!(orgs_domains, "/api/v1/organizations/{}/domains");
org_test!(orgs_departments, "/api/v1/organizations/{}/departments");
org_test!(orgs_contacts, "/api/v1/organizations/{}/contacts");
org_test!(orgs_related, "/api/v1/organizations/{}/related");
org_test!(orgs_timeline, "/api/v1/organizations/{}/timeline");
org_test!(orgs_portals, "/api/v1/organizations/{}/portals");
org_test!(orgs_procedures, "/api/v1/organizations/{}/procedures");
org_test!(orgs_playbooks, "/api/v1/organizations/{}/playbooks");
org_test!(orgs_templates, "/api/v1/organizations/{}/templates");
org_test!(orgs_financial, "/api/v1/organizations/{}/financial");
org_test!(orgs_contracts, "/api/v1/organizations/{}/contracts");
org_test!(orgs_compliance, "/api/v1/organizations/{}/compliance");
org_test!(orgs_services, "/api/v1/organizations/{}/services");
org_test!(orgs_products, "/api/v1/organizations/{}/products");
org_test!(orgs_enrichment, "/api/v1/organizations/{}/enrichment");
org_test!(orgs_risks, "/api/v1/organizations/{}/risks");
org_test!(orgs_health, "/api/v1/organizations/{}/health");
org_test!(orgs_dossier, "/api/v1/organizations/{}/dossier");
org_test!(orgs_brief, "/api/v1/organizations/{}/brief");
org_test!(orgs_context_pack, "/api/v1/organizations/{}/context-pack");

#[tokio::test]
async fn orgs_enrichment_apply_captures_observation_against_postgres() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    let oid = mkorg(&a, s).await;
    let database = Database::connect(Some(&db)).await.expect("db");
    let pool = database.pool().expect("configured pool").clone();
    let enrichment = OrgEnrichmentStore::new(pool.clone())
        .upsert(
            &oid,
            "crunchbase",
            json!({
                "fact": "Raised seed round"
            }),
            0.81,
        )
        .await
        .expect("create org enrichment result");

    let response = a
        .oneshot(post(
            &format!(
                "/api/v1/organizations/{}/enrichment/{}/apply",
                enc(&oid),
                enc(&enrichment.id)
            ),
            json!({}),
        ))
        .await
        .expect("response");
    assert!(
        response.status().is_success(),
        "apply={}",
        response.status()
    );

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'organization_enrichment_result'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&enrichment.id)
    .fetch_one(&pool)
    .await
    .expect("organization enrichment observation link count");
    assert_eq!(link_count, 1);
}

// ── Identity / Alias / Department creation ─────────────────────────────────
macro_rules! org_post_test {
    ($name:ident, $path:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let s = uid();
            let a = router(&db).await;
            let oid = mkorg(&a, s).await;
            let r = a
                .oneshot(post(&format!($path, enc(&oid)), $body))
                .await
                .expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
            );
        }
    };
}

org_post_test!(
    orgs_post_identity,
    "/api/v1/organizations/{}/identities",
    json!({"identity_type": "email_domain", "identity_value": "ex.com", "source": "manual"})
);
org_post_test!(
    orgs_post_alias,
    "/api/v1/organizations/{}/aliases",
    json!({"name": "AliasCo", "alias_type": "former_name", "source": "manual"})
);
org_post_test!(
    orgs_post_department,
    "/api/v1/organizations/{}/departments",
    json!({"name": "Engineering", "description": "eng"})
);

// ── Watchlist toggle ───────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_watchlist_toggle() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    let oid = mkorg(&a, s).await;
    let r = a
        .oneshot(post(
            &format!("/api/v1/organizations/{}/watchlist", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "watchlist={}", r.status());
}

#[tokio::test]
async fn organization_manual_entrypoints_capture_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let app = router(&db).await;
    let suffix = uid();
    let oid = mkorg(&app, suffix).await;
    let pool = Database::connect(Some(&db))
        .await
        .expect("db")
        .pool()
        .expect("pool")
        .clone();

    let create_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'organization'
           AND entity_id = $1
           AND metadata ->> 'action' = 'create'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&oid)
    .fetch_one(&pool)
    .await
    .expect("organization create observation link");

    let identity_response = app
        .clone()
        .oneshot(post(
            &format!("/api/v1/organizations/{}/identities", enc(&oid)),
            json!({
                "identity_type": "email_domain",
                "identity_value": format!
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/person_identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/person_identity.rs`
- Size bytes / Размер в байтах: `232`
- Included characters / Включено символов: `232`
- Truncated / Обрезано: `no`

```rust
#[path = "person_identity/events.rs"]
mod events;
#[path = "person_identity/merge_split.rs"]
mod merge_split;
#[path = "person_identity/refresh_ordering.rs"]
mod refresh_ordering;
#[path = "person_identity/support.rs"]
mod support;
```

### `backend/tests/person_identity/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/person_identity/events.rs`
- Size bytes / Размер в байтах: `5002`
- Included characters / Включено символов: `5002`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::persons::identity::{
    PersonIdentityReviewCommand, PersonIdentityReviewState,
};

use super::support::{
    build_review_event, identity_candidate_id_from_persons, live_person_identity_context,
    seed_normalized_persons, unique_suffix,
};

#[tokio::test]
async fn person_identity_reject_suppresses_candidate_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Sam Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("sam.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("sam.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-reject-{suffix}"),
            identity_candidate_id: identity_candidate_id.clone(),
            review_state: PersonIdentityReviewState::UserRejected,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("reject candidate");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh again");

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");
}

#[tokio::test]
async fn person_identity_review_event_rebuilds_state_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Pat Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("pat.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("pat.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let confirm_event = build_review_event(
        &identity_candidate_id,
        PersonIdentityReviewState::UserConfirmed,
        "event-reviewer",
        &format!("identity-event-confirm-{suffix}"),
    );
    let reject_event = build_review_event(
        &identity_candidate_id,
        PersonIdentityReviewState::UserRejected,
        "event-reviewer",
        &format!("identity-event-reject-{suffix}"),
    );

    context
        .event_store
        .append(&confirm_event)
        .await
        .expect("append confirm event");
    context
        .event_store
        .append(&reject_event)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event.event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    context
        .store
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm event");

    let reject_event = context
        .event_store
        .get_by_id(&reject_event.event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    context
        .store
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject event");

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");

    let event_id: String = sqlx::query_scalar(
        "SELECT event_id FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load event id");
    assert_eq!(
        event_id,
        format!("person_identity_review:identity-event-reject-{suffix}")
    );
}
```

### `backend/tests/person_identity/merge_split.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/person_identity/merge_split.rs`
- Size bytes / Размер в байтах: `8793`
- Included characters / Включено символов: `8793`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::persons::identity::{
    PersonIdentityReviewCommand, PersonIdentityReviewState,
};

use super::support::{
    identity_candidate_id_from_persons, live_person_identity_context, ordered_person_ids,
    seed_normalized_persons, split_identity_candidate_id_from_persons, unique_suffix,
};

#[tokio::test]
async fn person_identity_refresh_creates_conservative_merge_candidate_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Alex Meridian {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("alex.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("alex.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let created = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    assert!(created >= 1);

    let (left_id, right_id) = ordered_person_ids(&left.person_id, &right.person_id);
    let candidate_id = format!("identity_candidate:v1:merge_persons:{left_id}:{right_id}");
    let row: (String, String, String) = sqlx::query_as(
        r#"
        SELECT identity_candidate_id, candidate_kind, review_state
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate row");
    assert_eq!(row.0, candidate_id);
    assert_eq!(row.1, "merge_persons");
    assert_eq!(row.2, "suggested");
}

#[tokio::test]
async fn person_identity_confirm_records_review_without_mutating_persons_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Jordan Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("jordan.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("jordan.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");

    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let command = PersonIdentityReviewCommand {
        command_id: format!("identity-confirm-{suffix}"),
        identity_candidate_id: identity_candidate_id.clone(),
        review_state: PersonIdentityReviewState::UserConfirmed,
        actor_id: "tests-reviewer".to_owned(),
    };

    let result = context
        .store
        .set_review_state(&command)
        .await
        .expect("confirm identity candidate");
    assert_eq!(
        result.review_state,
        PersonIdentityReviewState::UserConfirmed
    );

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_confirmed");

    let persons =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE person_id IN ($1, $2)")
            .bind(&left.person_id)
            .bind(&right.person_id)
            .fetch_one(&context.pool)
            .await
            .expect("persons remain");
    assert_eq!(persons, 2);
}

#[tokio::test]
async fn person_identity_confirm_materializes_split_candidate_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Morgan Split Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("morgan.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("morgan.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");
    let merge_candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-confirm-for-split-{suffix}"),
            identity_candidate_id: merge_candidate_id,
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge candidate");

    let split_candidate_id =
        split_identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let row: (String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT candidate_kind, review_state, evidence_summary, confidence
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&split_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("split candidate row");

    assert_eq!(row.0, "split_person");
    assert_eq!(row.1, "suggested");
    assert!(
        row.2
            .starts_with("Previously confirmed merge can be split:")
    );
    assert_eq!(row.3, 1.0);
}

#[tokio::test]
async fn person_identity_confirmed_split_removes_merge_from_detail_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();
    let shared_name = format!("Taylor Split Detail {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("taylor.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("taylor.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");
    let merge_candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-confirm-detail-{suffix}"),
            identity_candidate_id: merge_candidate_id.clone(),
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge candidate");

    let detail = context
        .store
        .person_identity(&left.person_id)
        .await
        .expect("person identity detail");
    assert!(
        detail
            .items
            .iter()
            .any(|item| item.identity_candidate_id == merge_candidate_id)
    );

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh split candidates");
    let split_candidate_id =
        split_identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-split-detail-{suffix}"),
            identity_candidate_id: split_candidate_id,
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm split candidate");

    let detail = context
        .store
        .person_identity(&left.person_id)
        .await
        .expect("person identity detail after split");
    assert!(!detail.items.iter().any(|item| {
        item.candidate_kind == "merge_persons" && item.identity_candidate_id == merge_candidate_id
    }));
}
```

### `backend/tests/person_identity/refresh_ordering.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/person_identity/refresh_ordering.rs`
- Size bytes / Размер в байтах: `4783`
- Included characters / Включено символов: `4783`
- Truncated / Обрезано: `no`

```rust
use super::support::{
    age_identity_candidate, assert_identity_candidate_exists, confirm_identity_candidate,
    exclude_persons_from_name_merge_refresh, identity_candidate_id_from_persons,
    identity_candidate_updated_at, live_person_identity_context, promote_identity_candidate,
    seed_normalized_persons, split_identity_candidate_id_from_persons, unique_suffix,
};

#[tokio::test]
async fn person_identity_refresh_skips_existing_split_when_generating_next_split_against_postgres()
{
    let Some(context) = live_person_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();

    let first_left = context
        .person_store
        .upsert_email_person(&format!("first.left-{suffix}@example.com"))
        .await
        .expect("upsert first left person");
    let first_right = context
        .person_store
        .upsert_email_person(&format!("first.right-{suffix}@example.com"))
        .await
        .expect("upsert first right person");
    let second_left = context
        .person_store
        .upsert_email_person(&format!("second.left-{suffix}@example.com"))
        .await
        .expect("upsert second left person");
    let second_right = context
        .person_store
        .upsert_email_person(&format!("second.right-{suffix}@example.com"))
        .await
        .expect("upsert second right person");

    seed_normalized_persons(
        &context,
        &first_left.person_id,
        &first_right.person_id,
        &format!("First Split Existing {suffix}"),
    )
    .await
    .expect("seed first pair display names");
    seed_normalized_persons(
        &context,
        &second_left.person_id,
        &second_right.person_id,
        &format!("Second Split Pending {suffix}"),
    )
    .await
    .expect("seed second pair display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");

    let first_merge_candidate_id =
        identity_candidate_id_from_persons(&first_left.person_id, &first_right.person_id);
    let second_merge_candidate_id =
        identity_candidate_id_from_persons(&second_left.person_id, &second_right.person_id);
    let first_split_candidate_id =
        split_identity_candidate_id_from_persons(&first_left.person_id, &first_right.person_id);
    let second_split_candidate_id =
        split_identity_candidate_id_from_persons(&second_left.person_id, &second_right.person_id);

    confirm_identity_candidate(
        &context,
        &first_merge_candidate_id,
        &format!("identity-confirm-first-split-skip-{suffix}"),
    )
    .await
    .expect("confirm first merge candidate");
    confirm_identity_candidate(
        &context,
        &second_merge_candidate_id,
        &format!("identity-confirm-second-split-skip-{suffix}"),
    )
    .await
    .expect("confirm second merge candidate");

    exclude_persons_from_name_merge_refresh(
        &context,
        &[
            &first_left.person_id,
            &first_right.person_id,
            &second_left.person_id,
            &second_right.person_id,
        ],
        suffix,
    )
    .await
    .expect("exclude persons from merge refresh");

    promote_identity_candidate(&context, &first_merge_candidate_id)
        .await
        .expect("promote first merge candidate");
    let _ = context
        .store
        .refresh_candidates(1)
        .await
        .expect("create first split candidate");
    assert_identity_candidate_exists(&context, &first_split_candidate_id)
        .await
        .expect("first split candidate exists");

    age_identity_candidate(&context, &first_split_candidate_id)
        .await
        .expect("age first split candidate");
    let first_split_updated_at_before =
        identity_candidate_updated_at(&context, &first_split_candidate_id)
            .await
            .expect("first split updated_at before second refresh");

    promote_identity_candidate(&context, &second_merge_candidate_id)
        .await
        .expect("promote second merge candidate");
    promote_identity_candidate(&context, &first_merge_candidate_id)
        .await
        .expect("promote first merge candidate above second");

    let _ = context
        .store
        .refresh_candidates(1)
        .await
        .expect("create second split candidate");

    assert_identity_candidate_exists(&context, &second_split_candidate_id)
        .await
        .expect("second split candidate exists");
    let first_split_updated_at_after =
        identity_candidate_updated_at(&context, &first_split_candidate_id)
            .await
            .expect("first split updated_at after second refresh");
    assert_eq!(first_split_updated_at_after, first_split_updated_at_before);
}
```

### `backend/tests/person_identity/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/person_identity/support.rs`
- Size bytes / Размер в байтах: `6742`
- Included characters / Включено символов: `6742`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::identity::{
    PersonIdentityError, PersonIdentityReviewCommand, PersonIdentityReviewState,
    PersonIdentityStore,
};
use makosh_hub_backend::platform::events::{EventStore, NewEventEnvelope};
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPool;

const CONTACT_IDENTITY_REVIEW_EVENT_TYPE: &str = "person_identity.review_state_changed";

pub(crate) struct PersonIdentityTestContext {
    pub(crate) pool: PgPool,
    pub(crate) store: PersonIdentityStore,
    pub(crate) event_store: EventStore,
    pub(crate) person_store: PersonProjectionStore,
}

pub(crate) async fn live_person_identity_context() -> Option<PersonIdentityTestContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    person_identity_context(&database_url).await
}

async fn person_identity_context(database_url: &str) -> Option<PersonIdentityTestContext> {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(PersonIdentityTestContext {
        pool: pool.clone(),
        store: PersonIdentityStore::new(pool.clone()),
        event_store: EventStore::new(pool.clone()),
        person_store: PersonProjectionStore::new(pool.clone()),
    })
}

pub(crate) async fn seed_normalized_persons(
    context: &PersonIdentityTestContext,
    left_person_id: &str,
    right_person_id: &str,
    display_name: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        UPDATE persons
        SET display_name = $1
        WHERE person_id = $2 OR person_id = $3
        "#,
    )
    .bind(display_name)
    .bind(left_person_id)
    .bind(right_person_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn confirm_identity_candidate(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
    command_id: &str,
) -> Result<(), PersonIdentityError> {
    context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: command_id.to_owned(),
            identity_candidate_id: identity_candidate_id.to_owned(),
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await?;

    Ok(())
}

pub(crate) async fn exclude_persons_from_name_merge_refresh(
    context: &PersonIdentityTestContext,
    person_ids: &[&str],
    suffix: u128,
) -> Result<(), PersonIdentityError> {
    for (index, person_id) in person_ids.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE persons
            SET display_name = $1
            WHERE person_id = $2
            "#,
        )
        .bind(format!(
            "identity-refresh-skip-{suffix}-{index}@example.com"
        ))
        .bind(person_id)
        .execute(&context.pool)
        .await?;
    }

    Ok(())
}

pub(crate) async fn promote_identity_candidate(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        UPDATE person_identity_candidates
        SET updated_at = clock_timestamp()
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn age_identity_candidate(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        UPDATE person_identity_candidates
        SET updated_at = clock_timestamp() - INTERVAL '1 hour'
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn assert_identity_candidate_exists(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM person_identity_candidates
            WHERE identity_candidate_id = $1
        )
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?
    .then_some(())
    .ok_or(PersonIdentityError::IdentityCandidateNotFound)
}

pub(crate) async fn identity_candidate_updated_at(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<chrono::DateTime<Utc>, PersonIdentityError> {
    let updated_at = sqlx::query_scalar(
        r#"
        SELECT updated_at
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?;

    Ok(updated_at)
}

pub(crate) fn identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_persons:{left_person_id}:{right_person_id}")
}

pub(crate) fn split_identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:split_person:{left_person_id}:{right_person_id}")
}

pub(crate) fn ordered_person_ids(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_owned(), right_id.to_owned())
    } else {
        (right_id.to_owned(), left_id.to_owned())
    }
}

pub(crate) fn build_review_event(
    identity_candidate_id: &str,
    review_state: PersonIdentityReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        format!("person_identity_review:{command_id}"),
        CONTACT_IDENTITY_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "person_identity_review",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({"kind": "person_identity_review"}),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "identity_candidate_id": identity_candidate_id,
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
```

### `backend/tests/person_identity_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/person_identity_api.rs`
- Size bytes / Размер в байтах: `21702`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use tower::ServiceExt;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::identity::PersonIdentityStore;
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::events::{EventConsumerConfig, EventConsumerRunner};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::review_inbox::{
    PERSON_IDENTITY_REVIEW_INBOX_CONSUMER, project_person_identity_review_event,
};

const LOCAL_API_TOKEN: &str = "person-identity-api-test-token";

#[tokio::test]
async fn identity_candidates_reject_missing_local_api_secret() {
    let app = build_router(config_with_api_token());

    let response = app
        .oneshot(get_request("/api/v1/identity-candidates"))
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
async fn identity_candidates_returns_safe_candidate_payload() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let context = PersonIdentityApiContext {
        person_store: PersonProjectionStore::new(pool.clone()),
    };
    let shared_name = format!("Identity Api Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_persons(&pool, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    promote_identity_candidate(&pool, &candidate_id)
        .await
        .expect("promote candidate");
    run_person_identity_review_inbox_consumer(pool.clone()).await;

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/identity-candidates?limit=100",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    assert!(!items.is_empty());

    let item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(candidate_id))
        .expect("candidate payload");

    assert_eq!(item["candidate_kind"], "merge_persons");
    assert_eq!(item["review_state"], "suggested");
    assert_eq!(item["left_person_id"], json!(left.person_id));
    assert_eq!(item["right_person_id"], json!(right.person_id));
    assert!(item["evidence_summary"].is_string());
    assert!(item["confidence"].is_number());

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'identity_candidate_id'
        FROM review_items review_item
        WHERE review_item.metadata->>'identity_candidate_id' = $1
        ORDER BY review_item.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&candidate_id)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review item");
    assert_eq!(review_item.1, "identity_candidate");
    assert_eq!(review_item.2, "identity_candidates");
    assert_eq!(review_item.3, candidate_id);

    let observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kind.code AS kind_code
        FROM review_item_evidence evidence
        JOIN observations observation
          ON observation.observation_id = evidence.observation_id
        JOIN observation_kind_definitions kind
          ON kind.kind_definition_id = observation.kind_definition_id
        WHERE evidence.review_item_id = $1
        ORDER BY evidence.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(&review_item.0)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review evidence observation kind");
    assert_eq!(observation_kind, "PERSON_IDENTITY_CANDIDATE");
}

#[tokio::test]
async fn identity_candidates_returns_split_candidate_for_confirmed_merge() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Api Split {suffix}");

    let left = person_store
        .upsert_email_person(&format!("split-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_person(&format!("split-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_persons(&pool, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let merge_candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let command_id = format!("identity-api-split-confirm-{suffix}");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .clone()
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/identity-candidates/{merge_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    run_person_identity_review_inbox_consumer(pool.clone()).await;

    let split_candidate_id =
        split_identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    promote_identity_candidate(&pool, &split_candidate_id)
        .await
        .expect("promote split candidate");

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/identity-candidates?limit=100",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items");
    let split_item = items
        .iter()
        .find(|value| value["identity_candidate_id"] == json!(split_candidate_id))
        .expect("split candidate payload");

    assert_eq!(split_item["candidate_kind"], "split_person");
    assert_eq!(split_item["review_state"], "suggested");
    let evidence_summary = split_item["evidence_summary"]
        .as_str()
        .expect("evidence summary");
    assert!(evidence_summary.starts_with("Previously confirmed merge can be split:"));
    assert!(evidence_summary.contains(&left.person_id));
    assert!(evidence_summary.contains(&right.person_id));
}

#[tokio::test]
async fn put_identity_candidate_review_confirms_candidate() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();

    let person_store = PersonProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Review Api {suffix}");

    let left = person_store
        .upsert_email_person(&format!("review-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_person(&format!("review-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    seed_normalized_persons(&pool, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let store = PersonIdentityStore::new(pool.clone());
    let _ = store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let command_id = format!("identity-api-confirm-{suffix}");

    let app = build_router_with_database(
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str()),
        database,
    );

    let response = app
        .oneshot(json_put_request_with_actor(
            &format!("/api/v1/identity-candidates/{identity_candidate_id}/review"),
            json!({
                "command_id": command_id,
                "review_state": "user_confirmed",
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    run_person_identity_review_inbox_consumer(pool.clone()).await;
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({
            "identity_candidate_id": identity_candidate_id,
            "review_state": "user_confirmed",
            "event_id": format!("person_identity_review:{command_id}"),
        })
    );

    let review_item: (String, String, String) = sqlx::query_as(
        r#"
        SELECT status, target_entity_kind, target_entity_id
        FROM review_items
        WHERE metadata->>'identity_candidate_id' = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(&identity_candidate_id)
    .fetch_one(&pool)
    .await
    .expect("identity candidate review item");
    assert_eq!(review_item.0, "promoted");
    assert_eq!(review_item.1, "identity_candidate");
    assert_eq!(review_item.2, identity_candidate_id);
}

async fn run_person_identity_review_inbox_consumer(pool: PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(PERSON_IDENTITY_REVIEW_INBOX_CONSUMER),
    );

    for _ in 0..10 {
        let handler_pool = pool.clone();
        let report = runner
            .process_next_batch(|event| {
                project_person_identity_review_event(handler_pool.clone(), event)
            })
            .await
            .expect("person identity review inbox consumer");
        if report.processed == 0 {
            break;
        }
    }
}

#[tokio::test]
async fn person_identity_returns_confirmed_links_for_pe
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/person_identity_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/person_identity_architecture.rs`
- Size bytes / Размер в байтах: `2002`
- Included characters / Включено символов: `2002`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn person_identity_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_person_identity_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "person identity test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_person_identity_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_person_identity_test_violations(&path, violations);
            continue;
        }
        if !is_person_identity_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_person_identity_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "person_identity.rs" || file_name == "person_identity_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "person_identity")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons.rs`
- Size bytes / Размер в байтах: `324`
- Included characters / Включено символов: `324`
- Truncated / Обрезано: `no`

```rust
#[path = "persons/health_dossier.rs"]
mod health_dossier;
#[path = "persons/identities.rs"]
mod identities;
#[path = "persons/memory_preferences.rs"]
mod memory_preferences;
#[path = "persons/projection.rs"]
mod projection;
#[path = "persons/relationships.rs"]
mod relationships;
#[path = "persons/support.rs"]
mod support;
```

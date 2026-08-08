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

- Chunk ID / ID чанка: `086-test-backend-part-009`
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

### `backend/tests/persons/health_dossier.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons/health_dossier.rs`
- Size bytes / Размер в байтах: `5852`
- Included characters / Включено символов: `5852`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::expertise::PersonExpertiseStore;
use makosh_hub_backend::domains::persons::health::PersonHealthStore;
use makosh_hub_backend::domains::persons::investigator::PersonInvestigator;
use makosh_hub_backend::domains::persons::memory::{PersonFactStore, PersonPreferenceStore};
use makosh_hub_backend::domains::persons::trust::PersonRiskStore;

use super::support::{live_persons_pool, unique_suffix};

#[tokio::test]
async fn person_risk_report_and_resolve_materializes_health_status_cache_against_postgres() {
    let Some(pool) = live_persons_pool("person risk health adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let risk_store = PersonRiskStore::new(pool.clone());
    let health_store = PersonHealthStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("health-risk-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let risk = risk_store
        .report(
            &person.person_id,
            "relationship_attention",
            "Open evidence-backed relationship risk requires owner review.",
            "high",
            "risk-engine:test",
        )
        .await
        .expect("report risk");

    let stored_risk: (String, String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT risk_type, description, severity, source, confidence::float8 AS confidence
        FROM person_risks
        WHERE id::text = $1
        "#,
    )
    .bind(&risk.id)
    .fetch_one(&pool)
    .await
    .expect("stored risk observation");
    assert_eq!(stored_risk.0, "relationship_attention");
    assert_eq!(
        stored_risk.1,
        "Open evidence-backed relationship risk requires owner review."
    );
    assert_eq!(stored_risk.2, "high");
    assert_eq!(stored_risk.3, "risk-engine:test");
    assert_eq!(stored_risk.4, 0.5);

    let health = health_store
        .get(&person.person_id)
        .await
        .expect("load health")
        .expect("health row");
    assert_eq!(health.health_status, "at_risk");
    assert_eq!(health.open_risks, 1);

    risk_store
        .resolve(&risk.id, "owner reviewed and closed the risk")
        .await
        .expect("resolve risk");

    let health = health_store
        .get(&person.person_id)
        .await
        .expect("load health after resolve")
        .expect("health row after resolve");
    assert_eq!(health.health_status, "healthy");
    assert_eq!(health.open_risks, 0);
}

#[tokio::test]
async fn person_dossier_includes_target_sections_and_source_refs_against_postgres() {
    let Some(pool) = live_persons_pool("person dossier read-model").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let fact_store = PersonFactStore::new(pool.clone());
    let preference_store = PersonPreferenceStore::new(pool.clone());
    let expertise_store = PersonExpertiseStore::new(pool.clone());
    let investigator = PersonInvestigator::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("dossier-read-model-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    fact_store
        .upsert(
            &person.person_id,
            "interest",
            "local-first systems",
            "message:dossier-interest",
            0.9,
        )
        .await
        .expect("insert interest fact");
    fact_store
        .upsert(
            &person.person_id,
            "project",
            "Макошь Memory Graph",
            "document:dossier-project",
            0.8,
        )
        .await
        .expect("insert project fact");
    fact_store
        .upsert(
            &person.person_id,
            "organization",
            "Макошь Lab",
            "relationship:dossier-organization",
            0.85,
        )
        .await
        .expect("insert organization fact");
    preference_store
        .upsert(
            &person.person_id,
            "communication:preferred_channel",
            "telegram",
            "message:dossier-preference",
        )
        .await
        .expect("insert communication preference");
    expertise_store
        .upsert(
            &person.person_id,
            "Rust backend architecture",
            Some("software"),
            "document:dossier-skill",
            0.95,
        )
        .await
        .expect("insert expertise");

    let dossier = investigator
        .assemble_dossier(&person.person_id)
        .await
        .expect("assemble dossier");
    let dossier_json = serde_json::to_value(&dossier).expect("dossier json");

    assert_eq!(dossier_json["interests"][0]["value"], "local-first systems");
    assert_eq!(
        dossier_json["interests"][0]["source_refs"][0],
        "message:dossier-interest"
    );
    assert_eq!(dossier_json["projects"][0]["value"], "Макошь Memory Graph");
    assert_eq!(dossier_json["organizations"][0]["value"], "Макошь Lab");
    assert_eq!(
        dossier_json["skills"][0]["value"],
        "Rust backend architecture"
    );
    assert_eq!(
        dossier_json["communication_patterns"][0]["value"],
        "telegram"
    );
    assert!(
        dossier_json["source_refs"]
            .as_array()
            .expect("source refs array")
            .iter()
            .any(|source| source == "document:dossier-skill")
    );
    assert!(
        dossier_json["generated_at"].is_string(),
        "dossier must include generated_at"
    );
    assert!(
        dossier_json["ai_observations"].is_array(),
        "dossier must expose ai_observations as a labeled derived section"
    );
}
```

### `backend/tests/persons/identities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons/identities.rs`
- Size bytes / Размер в байтах: `4888`
- Included characters / Включено символов: `4888`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::core::PersonsIdentityStore;

use super::support::{live_persons_pool, unique_suffix};

#[tokio::test]
async fn person_identities_accept_document_and_message_traces_against_postgres() {
    let Some(pool) = live_persons_pool("persona identity trace type").await else {
        return;
    };
    let projection_store = PersonProjectionStore::new(pool.clone());
    let identity_store = PersonsIdentityStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = projection_store
        .upsert_email_person(&format!("identity-trace-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let document_trace = identity_store
        .upsert(
            &person.person_id,
            "document_mention",
            &format!("document:v1:{suffix}:identity-trace"),
            "document_processing",
        )
        .await
        .expect("upsert document mention identity trace");
    let message_trace = identity_store
        .upsert(
            &person.person_id,
            "message_participant",
            &format!("message:v1:{suffix}:identity-trace"),
            "communication_projection",
        )
        .await
        .expect("upsert message participant identity trace");

    assert_eq!(document_trace.identity_type, "document_mention");
    assert_eq!(document_trace.source, "document_processing");
    assert_eq!(message_trace.identity_type, "message_participant");
    assert_eq!(message_trace.source, "communication_projection");

    let identities = identity_store
        .list_by_person(&person.person_id)
        .await
        .expect("list persona identities");
    assert!(
        identities
            .iter()
            .any(|identity| identity.identity_type == "document_mention")
    );
    assert!(
        identities
            .iter()
            .any(|identity| identity.identity_type == "message_participant")
    );
}

#[tokio::test]
async fn person_identities_accept_disputed_status_against_postgres() {
    let Some(pool) = live_persons_pool("persona identity disputed status").await else {
        return;
    };
    let projection_store = PersonProjectionStore::new(pool.clone());
    let identity_store = PersonsIdentityStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = projection_store
        .upsert_email_person(&format!("identity-disputed-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let identity = identity_store
        .upsert(
            &person.person_id,
            "email",
            &format!("identity-disputed-trace-{suffix}@example.com"),
            "manual",
        )
        .await
        .expect("upsert identity");

    identity_store
        .update_status(&identity.id, "disputed")
        .await
        .expect("mark identity as disputed");

    let identities = identity_store
        .list_by_person(&person.person_id)
        .await
        .expect("list persona identities");
    let updated = identities
        .iter()
        .find(|candidate| candidate.id == identity.id)
        .expect("updated identity");
    assert_eq!(updated.status, "disputed");
}

#[tokio::test]
async fn person_identities_support_unattached_trace_assignment_against_postgres() {
    let Some(pool) = live_persons_pool("unattached persona identity trace").await else {
        return;
    };
    let projection_store = PersonProjectionStore::new(pool.clone());
    let identity_store = PersonsIdentityStore::new(pool.clone());
    let suffix = unique_suffix();

    let trace = identity_store
        .create_unattached(
            "message_participant",
            &format!("message:v1:{suffix}:unattached-participant"),
            "communication_projection",
        )
        .await
        .expect("create unattached identity trace");
    assert_eq!(trace.person_id.as_deref(), None);
    assert_eq!(trace.identity_type, "message_participant");
    assert_eq!(trace.source, "communication_projection");

    let person = projection_store
        .upsert_email_person(&format!("attach-trace-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let attached = identity_store
        .attach_to_persona(&trace.id, &person.person_id)
        .await
        .expect("attach identity trace to persona");

    assert_eq!(attached.id, trace.id);
    assert_eq!(
        attached.person_id.as_deref(),
        Some(person.person_id.as_str())
    );
    assert_eq!(attached.status, "active");

    let identities = identity_store
        .list_by_person(&person.person_id)
        .await
        .expect("list persona identities");
    assert!(identities.iter().any(|identity| {
        identity.id == trace.id && identity.person_id.as_deref() == Some(person.person_id.as_str())
    }));
}
```

### `backend/tests/persons/memory_preferences.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons/memory_preferences.rs`
- Size bytes / Размер в байтах: `12266`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::core::{NewPersonPersona, PersonPersonaStore};
use makosh_hub_backend::domains::persons::enrichment::PersonEnrichmentStore;
use makosh_hub_backend::domains::persons::enrichment_engine::EnrichmentResultStore;
use makosh_hub_backend::domains::persons::health::PersonHealthStore;
use makosh_hub_backend::domains::persons::memory::PersonFactStore;
use serde_json::json;

use super::support::{live_persons_pool, unique_suffix};

#[tokio::test]
async fn person_persona_upsert_and_delete_materializes_interaction_preferences_against_postgres() {
    let Some(pool) = live_persons_pool("person persona interaction preference adapter").await
    else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let persona_store = PersonPersonaStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("persona-context-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let persona_id = format!("interaction-context-{suffix}");

    let context = persona_store
        .upsert(&NewPersonPersona {
            persona_id: persona_id.clone(),
            person_id: person.person_id.clone(),
            name: "Work Context".to_owned(),
            context: Some("Professional replies for project updates".to_owned()),
            default_tone: Some("concise".to_owned()),
            default_language: Some("en".to_owned()),
            preferred_channel: Some("email".to_owned()),
        })
        .await
        .expect("upsert interaction context");

    let source = format!("person_personas:{persona_id}");
    let preferences: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT preference_type, value, source
        FROM person_preferences
        WHERE person_id = $1 AND source = $2
        ORDER BY preference_type
        "#,
    )
    .bind(&person.person_id)
    .bind(&source)
    .fetch_all(&pool)
    .await
    .expect("interaction context preferences");

    assert_eq!(
        preferences,
        vec![
            (
                format!("interaction_context:{persona_id}:context"),
                "Professional replies for project updates".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:default_language"),
                "en".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:default_tone"),
                "concise".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:name"),
                "Work Context".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:preferred_channel"),
                "email".to_owned(),
                source.clone(),
            ),
        ]
    );

    let deleted = persona_store
        .delete(&context.persona_id)
        .await
        .expect("delete interaction context");
    assert!(deleted);

    let remaining_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM person_preferences WHERE person_id = $1 AND source = $2",
    )
    .bind(&person.person_id)
    .bind(&source)
    .fetch_one(&pool)
    .await
    .expect("remaining interaction context preference count");
    assert_eq!(remaining_count, 0);
}

#[tokio::test]
async fn person_notes_materialize_persona_memory_card_against_postgres() {
    let Some(pool) = live_persons_pool("person notes memory adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_store = PersonEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("notes-memory-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("persons.notes:{}", person.person_id);

    enrichment_store
        .set_notes(
            &person.person_id,
            "Remember that this Persona prefers concise written summaries.",
        )
        .await
        .expect("set notes");

    let root_notes: String = sqlx::query_scalar("SELECT notes FROM persons WHERE person_id = $1")
        .bind(&person.person_id)
        .fetch_one(&pool)
        .await
        .expect("root compatibility notes");
    assert_eq!(
        root_notes,
        "Remember that this Persona prefers concise written summaries."
    );

    let memory_card: (String, String, String, i16) = sqlx::query_as(
        r#"
        SELECT title, description, source, importance
        FROM person_memory_cards
        WHERE person_id = $1 AND source = $2
        "#,
    )
    .bind(&person.person_id)
    .bind(&source)
    .fetch_one(&pool)
    .await
    .expect("notes memory card");
    assert_eq!(memory_card.0, "Compatibility notes");
    assert_eq!(
        memory_card.1,
        "Remember that this Persona prefers concise written summaries."
    );
    assert_eq!(memory_card.2, source);
    assert_eq!(memory_card.3, 5);
}

#[tokio::test]
async fn person_fact_upsert_uses_memory_engine_source_backed_draft_against_postgres() {
    let Some(pool) = live_persons_pool("person fact memory engine adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let fact_store = PersonFactStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("fact-memory-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let fact = fact_store
        .upsert(
            &person.person_id,
            " interest ",
            " local-first systems ",
            " communication_messages:message-1 ",
            0.84,
        )
        .await
        .expect("upsert fact");

    assert_eq!(fact.person_id, person.person_id);
    assert_eq!(fact.fact_type, "interest");
    assert_eq!(fact.value, "local-first systems");
    assert_eq!(fact.source, "communication_messages:message-1");
    assert!((fact.confidence - 0.84).abs() < 0.0001);
    assert!(fact.is_active);

    let stored_fact: (String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT fact_type, value, source, confidence::float8 AS confidence
        FROM person_facts
        WHERE id::text = $1
        "#,
    )
    .bind(&fact.id)
    .fetch_one(&pool)
    .await
    .expect("stored fact");
    assert_eq!(stored_fact.0, "interest");
    assert_eq!(stored_fact.1, "local-first systems");
    assert_eq!(stored_fact.2, "communication_messages:message-1");
    assert!((stored_fact.3 - 0.84).abs() < 0.0001);
}

#[tokio::test]
async fn person_favorite_toggle_materializes_ui_preference_against_postgres() {
    let Some(pool) = live_persons_pool("person favorite preference adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_store = PersonEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("favorite-preference-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("persons.is_favorite:{}", person.person_id);

    let is_favorite = enrichment_store
        .toggle_favorite(&person.person_id)
        .await
        .expect("toggle favorite on");
    assert!(is_favorite);

    let preference: (String, String, String, f32) = sqlx::query_as(
        r#"
        SELECT preference_type, value, source, confidence
        FROM person_preferences
        WHERE person_id = $1 AND preference_type = 'ui:favorite'
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("favorite UI preference");
    assert_eq!(preference.0, "ui:favorite");
    assert_eq!(preference.1, "true");
    assert_eq!(preference.2, source);
    assert_eq!(preference.3, 1.0);

    let is_favorite = enrichment_store
        .toggle_favorite(&person.person_id)
        .await
        .expect("toggle favorite off");
    assert!(!is_favorite);

    let preference_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("remaining favorite preference count");
    assert_eq!(preference_count, 0);
}

#[tokio::test]
async fn person_enrichment_result_upsert_materializes_pending_source_backed_candidate_against_postgres()
 {
    let Some(pool) = live_persons_pool("person enrichment result candidate").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_result_store = EnrichmentResultStore::new(pool);
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("enrichment-candidate-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let result = enrichment_result_store
        .upsert(
            &person.person_id,
            "communication_messages:message-1",
            json!({
                "field": "communication_style",
                "value": "concise asynchronous summaries",
                "extracted_claim": "prefers concise asynchronous summaries"
            }),
            0.82,
        )
        .await
        .expect("upsert enrichment result");

    assert_eq!(result.person_id, person.person_id);
    assert_eq!(result.source, "communication_messages:message-1");
    assert!((result.confidence - 0.82).abs() < 0.0001);
    assert_eq!(result.status, "pending");
    assert_eq!(result.data["field"], "communication_style");
    assert_eq!(
        result.data["_enrichment"]["affected_entity_kind"],
        "persona"
    );
    assert_eq!(
        result.data["_enrichment"]["affected_entity_id"],
        person.person_id
    );
    assert_eq!(
        result.data["_enrichment"]["extracted_claim"],
        "prefers concise asynchronous summaries"
    );
    assert_eq!(result.data["_enrichment"]["review_state"], "pending");
    assert_eq!(result.data["_enrichment"]["freshness"], "current");
    assert_eq!(result.data["_enrichment"]["conflict_marker"], false);
}

#[tokio::test]
async fn person_watchlist_toggle_materializes_ui_preference_against_postgres() {
    let Some(pool) = live_persons_pool("person watchlist preference adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let health_store = PersonHealthStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("watchlist-preference-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("persons.watchlist:{}", person.person_id);

    let watchlist = health_store
        .toggle_watchlist(&person.person_id)
        .await
        .expect("toggle watchlist on");
    assert!(watchlist);

    let preference: (String, String, String, f32) = sqlx::query_as(
        r#"
        SELECT preference_type, value, source, confidence
        FROM person_preferences
        WHERE person_id = $1 AND preference_type = 'ui:watchlist'
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("watchlist UI preference");
    assert_eq!(preference.0, "ui:watchlist");
    assert_eq!(preference.1, "true");
    assert_eq!(preference.2, source);
    assert_eq!(preference.3, 1.0);

    let watchlist = health_store
        .toggle_watchlist(&person.person_id)
        .await
        .expect("toggle watchlist off");
    assert!(!watchlist);

    let preference_count: i64 = sqlx::query_scalar(

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/persons/projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons/projection.rs`
- Size bytes / Размер в байтах: `6973`
- Included characters / Включено символов: `6973`
- Truncated / Обрезано: `no`

```rust
use makosh_hub_backend::domains::persons::api::{
    PersonProjectionError, PersonProjectionStore, PersonaType,
    upsert_persons_from_message_participants,
};

use super::support::{
    disconnected_persons_store, live_persons_pool, live_persons_store, unique_suffix,
};

#[tokio::test]
async fn persons_projection_upserts_email_identities_against_postgres() {
    let Some(store) = live_persons_store("persons projection").await else {
        return;
    };
    let suffix = unique_suffix();

    let persons = upsert_persons_from_message_participants(
        &store,
        &[
            format!("alice-{suffix}@example.com"),
            format!("bob-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert persons");

    assert_eq!(persons.len(), 2);
    assert!(
        persons
            .iter()
            .any(|p| p.email_address == format!("alice-{suffix}@example.com"))
    );
    assert!(
        persons
            .iter()
            .any(|p| p.email_address == format!("bob-{suffix}@example.com"))
    );
}

#[tokio::test]
async fn persons_projection_normalizes_and_deduplicates_participants_against_postgres() {
    let Some(store) = live_persons_store("persons normalization and deduplication").await else {
        return;
    };
    let suffix = unique_suffix();
    let normalized_email = format!("alice-{suffix}@example.com");

    let persons = upsert_persons_from_message_participants(
        &store,
        &[
            format!(" Alice-{suffix}@Example.com "),
            format!("alice-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert normalized persons");

    assert_eq!(persons.len(), 1);
    assert_eq!(persons[0].email_address, normalized_email);
    assert_eq!(persons[0].display_name, normalized_email);
}

#[tokio::test]
async fn persons_projection_rejects_blank_email_participant() {
    let store = disconnected_persons_store();

    let error = upsert_persons_from_message_participants(&store, &[String::from("   ")])
        .await
        .expect_err("blank email input must fail");

    assert!(
        matches!(error, PersonProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );
}

#[tokio::test]
async fn persons_projection_rejects_invalid_batch_before_writing_against_postgres() {
    let Some(pool) = live_persons_pool("persons invalid batch atomicity").await else {
        return;
    };
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let valid_email = format!("valid-before-blank-{suffix}@example.com");

    let error = upsert_persons_from_message_participants(
        &store,
        &[valid_email.clone(), String::from("   ")],
    )
    .await
    .expect_err("invalid participant batch must fail");

    assert!(
        matches!(error, PersonProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );

    let count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM persons WHERE email_address = $1")
            .bind(&valid_email)
            .fetch_one(&pool)
            .await
            .expect("person count after rejected batch");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn persons_projection_distinguishes_delimiter_bearing_email_identities_against_postgres() {
    let Some(store) = live_persons_store("delimiter-bearing person identities").await else {
        return;
    };
    let suffix = unique_suffix();

    let left = store
        .upsert_email_person(&format!("person:{suffix}@example.com"))
        .await
        .expect("upsert delimiter-bearing person");
    let right = store
        .upsert_email_person(&format!("person-{suffix}@example.com"))
        .await
        .expect("upsert non-delimiter person");

    assert_ne!(left.person_id, right.person_id);
    assert!(left.person_id.starts_with("person:v1:email:"));
    assert!(right.person_id.starts_with("person:v1:email:"));
}

#[tokio::test]
async fn persons_projection_defaults_to_human_non_owner_persona_against_postgres() {
    let Some(store) = live_persons_store("persona defaults").await else {
        return;
    };
    let suffix = unique_suffix();

    let person = store
        .upsert_email_person(&format!("persona-default-{suffix}@example.com"))
        .await
        .expect("upsert default persona");

    assert_eq!(person.persona_type, PersonaType::Human);
    assert!(!person.is_self);
}

#[tokio::test]
async fn persons_projection_tracks_single_owner_persona_against_postgres() {
    let Some(store) = live_persons_store("single owner persona").await else {
        return;
    };
    let suffix = unique_suffix();
    let first = store
        .upsert_email_person(&format!("owner-first-{suffix}@example.com"))
        .await
        .expect("upsert first owner candidate");
    let second = store
        .upsert_email_person(&format!("owner-second-{suffix}@example.com"))
        .await
        .expect("upsert second owner candidate");

    let first_owner = store
        .set_owner_persona(&first.person_id)
        .await
        .expect("set first owner persona");
    assert!(first_owner.is_self);

    let second_owner = store
        .set_owner_persona(&second.person_id)
        .await
        .expect("move owner persona");
    assert!(second_owner.is_self);

    let owner = store
        .owner_persona()
        .await
        .expect("load owner persona")
        .expect("owner persona exists");
    assert_eq!(owner.person_id, second.person_id);
}

#[tokio::test]
async fn persons_projection_sets_supported_persona_type_against_postgres() {
    let Some(store) = live_persons_store("persona type update").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("ai-agent-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let updated = store
        .set_persona_type(&person.person_id, PersonaType::AiAgent)
        .await
        .expect("set persona type");

    assert_eq!(updated.persona_type, PersonaType::AiAgent);
    assert!(!updated.is_self);
}

#[tokio::test]
async fn persons_schema_rejects_invalid_persona_type_against_postgres() {
    let Some(pool) = live_persons_pool("invalid persona type").await else {
        return;
    };
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("invalid-type-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let error = sqlx::query("UPDATE persons SET person_type = 'contact' WHERE person_id = $1")
        .bind(&person.person_id)
        .execute(&pool)
        .await
        .expect_err("invalid persona type must violate the check constraint");

    assert!(
        error.to_string().contains("persons_person_type_check"),
        "expected persons_person_type_check violation, got {error}"
    );
}
```

### `backend/tests/persons/relationships.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons/relationships.rs`
- Size bytes / Размер в байтах: `12568`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{Duration, Utc};
use makosh_hub_backend::domains::obligations::{
    ObligationEntityKind, ObligationReviewState, ObligationStatus, ObligationStore,
};
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::core::PersonRoleStore;
use makosh_hub_backend::domains::persons::enrichment::PersonEnrichmentStore;
use makosh_hub_backend::domains::persons::intelligence::CommunicationFingerprint;
use makosh_hub_backend::domains::persons::trust::PersonPromiseStore;

use super::support::{live_persons_pool, run_person_derived_evidence_consumer, unique_suffix};

#[tokio::test]
async fn person_role_assign_and_remove_materializes_relationship_against_postgres() {
    let Some(pool) = live_persons_pool("person role relationship adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let role_store = PersonRoleStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("role-adapter-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let _role = role_store
        .assign(
            &person.person_id,
            "Technical Advisor",
            Some("persona:owner"),
        )
        .await
        .expect("assign person role");
    run_person_derived_evidence_consumer(pool.clone()).await;

    let relationship: (
        String,
        String,
        String,
        String,
        String,
        String,
        f64,
        f64,
        f64,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT
            relationship_id,
            source_entity_kind,
            source_entity_id,
            target_entity_kind,
            target_entity_id AS entity_id,
            review_state,
            trust_score::float8 AS trust_score,
            strength_score::float8 AS strength_score,
            confidence::float8 AS confidence,
            metadata
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = $1
          AND target_entity_kind = 'knowledge'
          AND target_entity_id = 'person_role:technical_advisor'
          AND relationship_type = 'has_role'
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("role relationship");

    assert_eq!(relationship.1, "persona");
    assert_eq!(relationship.2, person.person_id);
    assert_eq!(relationship.3, "knowledge");
    assert_eq!(relationship.4, "person_role:technical_advisor");
    assert_eq!(relationship.5, "user_confirmed");
    assert_eq!(relationship.6, 1.0);
    assert_eq!(relationship.7, 0.7);
    assert_eq!(relationship.8, 1.0);
    assert_eq!(relationship.9["compatibility_source"], "person_roles");
    assert_eq!(relationship.9["role"], "Technical Advisor");
    assert_eq!(relationship.9["assigned_by"], "persona:owner");

    let evidence: (String, String, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("role relationship evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(evidence.2.as_deref(), Some("Technical Advisor"));
    assert_eq!(evidence.3["compatibility_source"], "person_roles");
    let role_observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kinds.code
        FROM observations observation
        JOIN observation_kind_definitions kinds
          ON kinds.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&evidence.1)
    .fetch_one(&pool)
    .await
    .expect("role observation kind");
    assert_eq!(role_observation_kind, "PERSON_ROLE");

    let removed = role_store
        .remove(&person.person_id, "Technical Advisor")
        .await
        .expect("remove person role");
    assert!(removed);
    run_person_derived_evidence_consumer(pool.clone()).await;

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM relationships WHERE relationship_id = $1")
            .bind(&relationship.0)
            .fetch_one(&pool)
            .await
            .expect("demoted role relationship");
    assert_eq!(review_state, "user_rejected");
}

#[tokio::test]
async fn person_enrichment_trust_score_materializes_owner_relationship_against_postgres() {
    let Some(pool) = live_persons_pool("person enrichment trust relationship adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_store = PersonEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = person_store
        .upsert_email_person(&format!("trust-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    let target = person_store
        .upsert_email_person(&format!("trust-target-{suffix}@example.com"))
        .await
        .expect("upsert target persona");
    person_store
        .set_owner_persona(&owner.person_id)
        .await
        .expect("set owner persona");

    let fingerprint = CommunicationFingerprint {
        avg_message_length: Some(120),
        avg_response_hours: Some(3.5),
        frequent_topics: vec!["project".to_owned()],
        typical_tone: Some("friendly".to_owned()),
        detected_language: Some("en".to_owned()),
        writing_style: Some("balanced".to_owned()),
        preferred_time_of_day: None,
        trust_score: Some(82),
    };

    enrichment_store
        .enrich_person(&target.person_id, &fingerprint)
        .await
        .expect("enrich target persona");
    run_person_derived_evidence_consumer(pool.clone()).await;

    let relationship: (
        String,
        String,
        String,
        String,
        f64,
        f64,
        f64,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT
            relationship_id,
            source_entity_id,
            target_entity_id AS entity_id,
            review_state,
            trust_score::float8 AS trust_score,
            strength_score::float8 AS strength_score,
            confidence::float8 AS confidence,
            metadata
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = $1
          AND target_entity_kind = 'persona'
          AND target_entity_id = $2
          AND relationship_type = 'trusts'
        "#,
    )
    .bind(&owner.person_id)
    .bind(&target.person_id)
    .fetch_one(&pool)
    .await
    .expect("owner trust relationship");

    assert_eq!(relationship.1, owner.person_id);
    assert_eq!(relationship.2, target.person_id);
    assert_eq!(relationship.3, "suggested");
    assert_eq!(relationship.4, 0.82);
    assert_eq!(relationship.5, 0.5);
    assert_eq!(relationship.6, 1.0);
    assert_eq!(
        relationship.7["compatibility_source"],
        "persons.trust_score"
    );
    assert_eq!(relationship.7["trust_score"], 82);

    let evidence: (String, String, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("trust relationship evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(evidence.2.as_deref(), Some("trust_score=82"));
    assert_eq!(evidence.3["compatibility_source"], "persons.trust_score");
    assert_eq!(
        evidence.3["trust_source_reliability"]["signal_type"],
        "source_reliability"
    );
    assert_eq!(
        evidence.3["trust_source_reliability"]["affected_source"],
        format!("person_enrichment:{}:trust_score", target.person_id)
    );
    assert_eq!(
        evidence.3["trust_source_reliability"]["direction"],
        "positive"
    );
    assert_eq!(evidence.3["trust_source_reliability"]["confidence"], 0.82);
    let trust_observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kinds.code
        FROM observations observation
        JOIN observation_kind_definitions kinds
          ON kinds.kind_definition_id = observation.kind_definition_id
        WHERE observation.observation_id = $1
        "#,
    )
    .bind(&evidence.1)
    .fetch_one(&pool)
    .await
    .expect("trust observation kind");
    assert_eq!(trust_observation_kind, "PERSON_TRUST_SIGNAL");

    let review_item: (String, String, String, String) = sqlx::query_as(
        r#"
        SELECT
            review_item.review_item_id,
            review_item.item_kind,
            review_item.metadata->>'mirrored_from',
            review_item.metadata->>'relationship_id'
        FROM review_items review_item
        JOIN review_item_evidence evidence
          ON evidence.review_item_id = review_item.review_item_id
        WHERE evidence.observation_id = $1
          AND review_item.item_kind = 'potential_relationship'
          AND review_item.metadata->>'relationship_id' = $2
        "#,
    )
    .bind(&evidence.1)
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("relationship review mirror");
    assert_eq!(review_item.1, "potential_relationship");
    assert_eq!(review_item.2, "relationships");
    assert_eq!(review_item.3, relationship.0);
}

#[tokio::test]
async fn person_promise_create_materializes_user_confirmed_obligation_without_task_against_postgres()
 {
    let Some(pool) = live_persons_pool("person promise obligation adapter").await else {
        return;
    };
    let person_store = PersonProjectionStore::new(pool.clone());
    let promise_store = PersonPromiseStore::new(pool.clone());
    let obligation_store = ObligationStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("promise-adapter-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let due_at = Utc::now() + Duration::days(5);
    let description = format!("Send the persona promise evidence package {suffix}");

    let promise = promise_store
        .create(&person.person_id, &description, Some(due_at))
        .await
        .expect("create person promise");
    run_person_derived_evidence_consumer(pool.clone()).await;

    let obligations = obligation_store
        .list_for_entity(ObligationEntityKind::Persona, &person.person_id, 10)
        .await
        .expect("persona obligations");
    let obligation = obligations
        .iter()
        .find(|item| item.statement == description)
        .expect("person promise should create a durable Obligation");

    assert_eq!(
        obligation.review_state,
        ObligationReviewState::UserConfirmed
    );
    assert_eq!(obligation.status, ObligationStatus::Open);
    assert_eq!(
        obligation.due_at.map(|value| value.timestamp_micros()),
        Some(due_at.timestamp_micros())
    );
    assert_eq!(
        obligation.metadata["person_promise_id"],
        serde_json::json!(promise.id)
    );

    let evidence: (String, String, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, quote FROM obligation_evidence WHERE obligation_id = $1",
    )
    .bind(&obligation.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence.0, "observation");
    assert!(!evidence.1.is_empty());
    assert_eq!(evidence.2.as_deref(), Some(description.as_str()));
    let promise_observation_kind: String = sqlx::query_scalar(
        r#"
        SELECT kinds.code
        FROM observations observation
        JOIN observation_kind_definitions kinds
          ON kinds.kind_de
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/persons/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons/support.rs`
- Size bytes / Размер в байтах: `1830`
- Included characters / Включено символов: `1830`
- Truncated / Обрезано: `no`

```rust
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::platform::events::{EventConsumerConfig, EventConsumerRunner};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::person_derived_evidence::{
    PERSON_DERIVED_EVIDENCE_CONSUMER, project_person_derived_evidence_event,
};
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn live_persons_pool(_test_name: &str) -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub async fn live_persons_store(test_name: &str) -> Option<PersonProjectionStore> {
    let pool = live_persons_pool(test_name).await?;
    Some(PersonProjectionStore::new(pool))
}

pub fn disconnected_persons_store() -> PersonProjectionStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    PersonProjectionStore::new(pool)
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn run_person_derived_evidence_consumer(pool: PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(PERSON_DERIVED_EVIDENCE_CONSUMER),
    );
    runner
        .process_next_batch(|event| project_person_derived_evidence_event(pool.clone(), event))
        .await
        .expect("person derived evidence consumer");
}
```

### `backend/tests/persons_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api.rs`
- Size bytes / Размер в байтах: `666`
- Included characters / Включено символов: `666`
- Truncated / Обрезано: `no`

```rust
#[path = "persons_api/auth.rs"]
mod auth;
#[path = "persons_api/dossier_owner.rs"]
mod dossier_owner;
#[path = "persons_api/identity_traces.rs"]
mod identity_traces;
#[path = "persons_api/persona_routes.rs"]
mod persona_routes;
#[path = "persons_api/read_endpoints.rs"]
mod read_endpoints;
#[path = "persons_api/support.rs"]
mod support;
#[path = "persons_api/write_entrypoints_basic.rs"]
mod write_entrypoints_basic;
#[path = "persons_api/write_identity_timeline.rs"]
mod write_identity_timeline;
#[path = "persons_api/write_memory_observations.rs"]
mod write_memory_observations;
#[path = "persons_api/write_review_observations.rs"]
mod write_review_observations;
```

### `backend/tests/persons_api/auth.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/auth.rs`
- Size bytes / Размер в байтах: `636`
- Included characters / Включено символов: `636`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use super::support::{build_persons_app_without_database, get_request, json_body};

#[tokio::test]
async fn persons_rejects_missing_local_api_secret() {
    let app = build_persons_app_without_database();
    let response = app
        .oneshot(get_request("/api/v1/persons"))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;
    assert_eq!(
        body,
        json!({"error": "invalid_api_secret", "message": "missing or invalid x-makosh-secret header"})
    );
}
```

### `backend/tests/persons_api/dossier_owner.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/dossier_owner.rs`
- Size bytes / Размер в байтах: `9988`
- Included characters / Включено символов: `9988`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, build_persons_app_with_database, get_request_with_token,
    json_body, post_request_with_token, put_request_with_token, unique_suffix,
    urlencoding_percent_encode,
};

#[tokio::test]
async fn person_dossier_get_persists_snapshot_and_review_state_against_postgres() {
    let Some(database_url) = super::support::live_database_url("dossier snapshot API").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("dossier-snapshot-{suffix}@example.com"))
        .await
        .expect("upsert dossier persona");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/persons/{}/dossier",
                urlencoding_percent_encode(&person.person_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let snapshot_id = body["dossier_snapshot_id"]
        .as_str()
        .expect("dossier snapshot id")
        .to_owned();
    assert_eq!(body["review_state"], "suggested");
    assert_eq!(body["person"]["person_id"], person.person_id);

    let row = sqlx::query(
        r#"
        SELECT persona_id, review_state, dossier
        FROM persona_dossier_snapshots
        WHERE dossier_snapshot_id = $1
        "#,
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("stored dossier snapshot");
    assert_eq!(
        row.try_get::<String, _>("persona_id").expect("persona id"),
        person.person_id
    );
    assert_eq!(
        row.try_get::<String, _>("review_state")
            .expect("review state"),
        "suggested"
    );
    let stored_dossier = row.try_get::<Value, _>("dossier").expect("dossier json");
    assert_eq!(stored_dossier["person"]["person_id"], person.person_id);

    let dossier_refresh_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'dossier_snapshot'
           AND entity_id = $1
           AND relationship_kind = 'dossier_refresh'",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("dossier refresh observation link count");
    assert_eq!(dossier_refresh_link_count, 1);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/persons/{}/dossier/review",
                urlencoding_percent_encode(&person.person_id)
            ),
            json!({ "review_state": "user_confirmed" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("dossier review response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["dossier_snapshot_id"], snapshot_id);
    assert_eq!(body["review_state"], "user_confirmed");
    assert!(body["reviewed_at"].is_string());

    let dossier_review_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'dossier_snapshot'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("dossier review observation link count");
    assert_eq!(dossier_review_link_count, 1);
}

#[tokio::test]
async fn person_investigate_captures_observation_and_links_snapshot_against_postgres() {
    let Some(database_url) = super::support::live_database_url("person investigate API").await
    else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("investigate-snapshot-{suffix}@example.com"))
        .await
        .expect("upsert investigate persona");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/investigate",
                urlencoding_percent_encode(&person.person_id)
            ),
            json!({ "query": "refresh dossier" }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("investigate response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let snapshot_id = body["dossier_snapshot_id"]
        .as_str()
        .expect("dossier snapshot id")
        .to_owned();

    let investigation_observation: (String, String) = sqlx::query_as(
        "SELECT link.observation_id, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'dossier_snapshot'
           AND link.entity_id = $1
           AND link.relationship_kind = 'dossier_refresh'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("investigate observation");
    assert!(!investigation_observation.0.is_empty());
    assert_eq!(investigation_observation.1, "PERSON_MUTATION");
}

#[tokio::test]
async fn person_detail_not_found_returns_404() {
    let Some(database_url) = super::support::live_database_url("person detail").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            &format!("/api/v1/persons/person:nonexistent-{suffix}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn person_owner_get_and_put_uses_owner_persona_against_postgres() {
    let Some(database_url) = super::support::live_database_url("owner persona API").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE persons SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("owner-api-{suffix}@example.com"))
        .await
        .expect("upsert owner candidate");
    let other = store
        .upsert_email_person(&format!("not-owner-api-{suffix}@example.com"))
        .await
        .expect("upsert non-owner candidate");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/persons/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("initial owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body["owner_persona"].is_null());

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            "/api/v1/persons/owner",
            json!({ "person_id": owner.person_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("set owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["person_id"], owner.person_id);
    assert_eq!(body["owner_persona"]["is_self"], true);
    assert_eq!(body["owner_persona"]["persona_type"], "human");

    let owner_assignment_observation: (String, String) = sqlx::query_as(
        "SELECT link.observation_id, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'persona'
           AND link.entity_id = $1
           AND link.relationship_kind = 'owner_assignment'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&owner.person_id)
    .fetch_one(&pool)
    .await
    .expect("owner assignment observation");
    assert!(!owner_assignment_observation.0.is_empty());
    assert_eq!(owner_assignment_observation.1, "PERSON_MUTATION");

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/owner",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("owner response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["owner_persona"]["person_id"], owner.person_id);
    assert_ne!(body["owner_persona"]["person_id"], other.person_id);
}
```

### `backend/tests/persons_api/identity_traces.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/identity_traces.rs`
- Size bytes / Размер в байтах: `5128`
- Included characters / Включено символов: `5128`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::platform::storage::Database;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, get_request_with_token, json_body, post_request_with_token,
    put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn identity_traces_create_list_and_attach_unattached_trace() {
    let Some(database_url) = super::support::live_database_url("identity traces API").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    let create = app
        .clone()
        .oneshot(post_request_with_token(
            "/api/v1/identity-traces",
            json!({
                "identity_type": "message_participant",
                "identity_value": format!("message:v1:{suffix}:api-unattached"),
                "source": "communication_projection"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("create trace response");
    assert_eq!(create.status(), StatusCode::OK);
    let create_body = json_body(create).await;
    assert_eq!(create_body["person_id"], Value::Null);
    assert_eq!(create_body["identity_type"], "message_participant");
    assert_eq!(create_body["source"], "communication_projection");
    let identity_id = create_body["id"].as_str().expect("identity id").to_owned();

    let observation_row = sqlx::query(
        "SELECT kind.code AS kind_code, observation.origin_kind
         FROM observation_links link
         JOIN observations observation ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'identity_trace'
           AND link.entity_id = $1
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&identity_id)
    .fetch_one(&pool)
    .await
    .expect("identity trace observation");
    assert_eq!(
        observation_row
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "PERSON_RECORD_MUTATION"
    );
    assert_eq!(
        observation_row
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );

    let list = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/identity-traces?status=unattached",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("list traces response");
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = json_body(list).await;
    let items = list_body["items"].as_array().expect("items");
    assert!(items.iter().any(|item| item["id"] == identity_id
        && item["person_id"] == Value::Null
        && item["identity_type"] == "message_participant"));

    let person_store = PersonProjectionStore::new(pool.clone());
    let person = person_store
        .upsert_email_person(&format!("identity-trace-api-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let attach = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/identity-traces/{}/assignment",
                urlencoding_percent_encode(&identity_id)
            ),
            json!({ "person_id": person.person_id }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("attach trace response");
    assert_eq!(attach.status(), StatusCode::OK);
    let attach_body = json_body(attach).await;
    assert_eq!(attach_body["id"], identity_id);
    assert_eq!(attach_body["person_id"], person.person_id);
    assert_eq!(attach_body["status"], "active");

    let assignment_observation_row = sqlx::query(
        "SELECT kind.code AS kind_code, observation.origin_kind
         FROM observation_links link
         JOIN observations observation ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'identity_trace'
           AND link.entity_id = $1
           AND link.relationship_kind = 'trace_assignment'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&identity_id)
    .fetch_one(&pool)
    .await
    .expect("identity trace assignment observation");
    assert_eq!(
        assignment_observation_row
            .try_get::<String, _>("kind_code")
            .expect("assignment kind code"),
        "PERSON_RECORD_MUTATION"
    );
    assert_eq!(
        assignment_observation_row
            .try_get::<String, _>("origin_kind")
            .expect("assignment origin kind"),
        "manual"
    );
}
```

### `backend/tests/persons_api/persona_routes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/persona_routes.rs`
- Size bytes / Размер в байтах: `7194`
- Included characters / Включено символов: `7194`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;
use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, build_persons_app_with_database, get_request_with_token,
    json_body, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn persons_list_returns_ok() {
    let Some(database_url) = super::support::live_database_url("persons list").await else {
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token("/api/v1/persons", LOCAL_API_TOKEN))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn personas_routes_return_persona_native_schema_against_postgres() {
    let Some(database_url) = super::support::live_database_url("personas route").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE persons SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonProjectionStore::new(pool);
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("persona-native-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    store
        .set_owner_persona(&owner.person_id)
        .await
        .expect("set owner persona");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            "/api/v1/personas?limit=20",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("personas list response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let items = body["items"].as_array().expect("items array");
    assert!(
        items
            .iter()
            .any(|item| item["persona_id"] == owner.person_id && item["is_self"] == true),
        "personas list should include owner Persona: {body}"
    );

    let response = app
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona detail response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.person_id);
    assert_eq!(body["persona_type"], "human");
    assert_eq!(body["is_self"], true);
    assert_eq!(body["identity"]["display_name"], owner.display_name);
    assert_eq!(body["identity"]["email_address"], owner.email_address);
    assert_eq!(body["communication"]["primary_email"], owner.email_address);
    assert_eq!(body["compatibility"]["legacy_person_id"], owner.person_id);
    assert_eq!(body["compatibility"]["legacy_route"], "/api/v1/persons");
}

#[tokio::test]
async fn personas_put_updates_compatibility_projection_against_postgres() {
    let Some(database_url) = super::support::live_database_url("personas write route").await else {
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    sqlx::query("UPDATE persons SET is_self = false WHERE is_self = true")
        .execute(&pool)
        .await
        .expect("clear existing owner persona");
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = store
        .upsert_email_person(&format!("persona-native-write-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    let previous_owner = store
        .upsert_email_person(&format!("persona-native-write-prev-{suffix}@example.com"))
        .await
        .expect("upsert previous owner persona");
    store
        .set_owner_persona(&previous_owner.person_id)
        .await
        .expect("set previous owner persona");

    let app = build_persons_app_with_database(&database_url, database);

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            json!({
                "identity": {
                    "display_name": "Owner Persona"
                },
                "is_self": true
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona update response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["persona_id"], owner.person_id);
    assert_eq!(body["identity"]["display_name"], "Owner Persona");
    assert_eq!(body["is_self"], true);

    let row = sqlx::query(
        r#"
        SELECT display_name, is_self
        FROM persons
        WHERE person_id = $1
        "#,
    )
    .bind(&owner.person_id)
    .fetch_one(&pool)
    .await
    .expect("updated persona row");
    assert_eq!(
        row.try_get::<String, _>("display_name").unwrap(),
        "Owner Persona"
    );
    assert!(row.try_get::<bool, _>("is_self").unwrap());

    let persona_update_observation: (String, String) = sqlx::query_as(
        "SELECT link.observation_id, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'persona'
           AND link.entity_id = $1
           AND link.relationship_kind = 'persona_update'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&owner.person_id)
    .fetch_one(&pool)
    .await
    .expect("persona update observation link");
    assert!(!persona_update_observation.0.is_empty());
    assert_eq!(persona_update_observation.1, "PERSON_MUTATION");

    let previous_is_self: bool =
        sqlx::query_scalar("SELECT is_self FROM persons WHERE person_id = $1")
            .bind(&previous_owner.person_id)
            .fetch_one(&pool)
            .await
            .expect("previous owner row");
    assert!(!previous_is_self);

    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/personas/{}",
                urlencoding_percent_encode(&owner.person_id)
            ),
            json!({ "is_self": false }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona unset owner response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

### `backend/tests/persons_api/read_endpoints.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/read_endpoints.rs`
- Size bytes / Размер в байтах: `4585`
- Included characters / Включено символов: `4585`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;
use tower::ServiceExt;

use super::support::{LOCAL_API_TOKEN, build_persons_app, get_request_with_token, unique_suffix};

#[tokio::test]
async fn person_search_returns_ok() {
    let Some(database_url) = super::support::live_database_url("person search").await else {
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/search?q=alex",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

macro_rules! person_endpoint_test {
    ($name:ident, $path_suffix:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = super::support::live_database_url(stringify!($name)).await
            else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_persons_app(&database_url).await;
            let path = format!(
                "/api/v1/persons/person:nonexistent-{}{}",
                suffix, $path_suffix
            );
            let response = app
                .oneshot(get_request_with_token(&path, LOCAL_API_TOKEN))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "status={}",
                response.status()
            );
        }
    };
}

person_endpoint_test!(person_identities_list, "/identities");
person_endpoint_test!(person_roles_list, "/roles");
person_endpoint_test!(person_personas_list, "/personas");
person_endpoint_test!(person_facts_list, "/facts");
person_endpoint_test!(person_memory_cards_list, "/memory-cards");
person_endpoint_test!(person_preferences_list, "/preferences");
person_endpoint_test!(person_timeline_list, "/timeline");
person_endpoint_test!(person_snapshots_list, "/snapshots");
person_endpoint_test!(person_history_diff, "/history-diff");
person_endpoint_test!(person_enrichment_list, "/enrichment");
person_endpoint_test!(person_expertise_list, "/expertise");
person_endpoint_test!(person_promises_list, "/promises");
person_endpoint_test!(person_risks_list, "/risks");
person_endpoint_test!(person_health_get, "/health");
person_endpoint_test!(person_dossier_get, "/dossier");
person_endpoint_test!(person_meeting_prep_get, "/meeting-prep");
person_endpoint_test!(person_analytics_get, "/analytics");
person_endpoint_test!(person_export_get, "/export");
person_endpoint_test!(person_identity_detail, "/identity");

#[tokio::test]
async fn persons_health_returns_ok() {
    let Some(database_url) = super::support::live_database_url("persons health").await else {
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/health",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn persons_watchlist_returns_ok() {
    let Some(database_url) = super::support::live_database_url("persons watchlist").await else {
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/watchlist",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn identity_candidates_list_returns_ok() {
    let Some(database_url) = super::support::live_database_url("identity candidates").await else {
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/identity-candidates",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn person_expertise_search() {
    let Some(database_url) = super::support::live_database_url("person expertise search").await
    else {
        return;
    };
    let app = build_persons_app(&database_url).await;
    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/persons/search/expertise?q=rust",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "expertise search={}",
        response.status()
    );
}
```

### `backend/tests/persons_api/support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/support.rs`
- Size bytes / Размер в байтах: `4186`
- Included characters / Включено символов: `4186`
- Truncated / Обрезано: `no`

```rust
use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, header};
use serde_json::Value;

use makosh_hub_backend::app::{build_router, build_router_with_database};
use makosh_hub_backend::platform::config::AppConfig;
use makosh_hub_backend::platform::events::{EventConsumerConfig, EventConsumerRunner};
use makosh_hub_backend::platform::storage::Database;
use makosh_hub_backend::workflows::person_derived_evidence::{
    PERSON_DERIVED_EVIDENCE_CONSUMER, project_person_derived_evidence_event,
};
use sqlx::postgres::PgPool;

pub const LOCAL_API_TOKEN: &str = "persons-api-test-token";

pub fn config_with_api_token() -> AppConfig {
    app_config_with_pairs(Vec::new())
}

pub fn app_config_with_pairs(mut extra_pairs: Vec<(&'static str, String)>) -> AppConfig {
    let suffix = unique_suffix();
    let vault_home = format!("/tmp/makosh-persons-api-vault-{suffix}");
    let dev_key_path = format!("{vault_home}/dev.key");
    testkit::app::config_with_secret(LOCAL_API_TOKEN)
        .with_test_dev_vault_paths(vault_home, dev_key_path)
        .with_test_pairs(extra_pairs.drain(..))
        .expect("valid local API config")
}

pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub fn get_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub fn post_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn put_request_with_token(uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

pub fn urlencoding_percent_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos()
}

pub async fn build_persons_app(database_url: &str) -> axum::Router {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    build_persons_app_with_database(database_url, database)
}

pub fn build_persons_app_with_database(database_url: &str, database: Database) -> axum::Router {
    build_router_with_database(
        app_config_with_pairs(Vec::new()).with_test_database_url(database_url),
        database,
    )
}

pub fn build_persons_app_without_database() -> axum::Router {
    build_router(config_with_api_token())
}

pub async fn live_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    Some(database_url)
}

pub async fn run_person_derived_evidence_consumer(pool: PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(PERSON_DERIVED_EVIDENCE_CONSUMER),
    );
    runner
        .process_next_batch(|event| project_person_derived_evidence_event(pool.clone(), event))
        .await
        .expect("person derived evidence consumer");
}
```

### `backend/tests/persons_api/write_entrypoints_basic.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/write_entrypoints_basic.rs`
- Size bytes / Размер в байтах: `6131`
- Included characters / Включено символов: `6131`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app, delete_request_with_token, post_request_with_token,
    put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

macro_rules! person_post_test {
    ($name:ident, $path_suffix:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let Some(database_url) = super::support::live_database_url(stringify!($name)).await
            else {
                return;
            };
            let suffix = unique_suffix();
            let app = build_persons_app(&database_url).await;
            let pid = format!("person:nonexistent-{suffix}");
            let response = app
                .oneshot(post_request_with_token(
                    &format!(
                        "/api/v1/persons/{}/{}",
                        urlencoding_percent_encode(&pid),
                        $path_suffix
                    ),
                    $body,
                    LOCAL_API_TOKEN,
                ))
                .await
                .expect("response");
            assert!(
                !response.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                response.status()
            );
        }
    };
}

person_post_test!(
    person_post_fingerprint,
    "fingerprint",
    json!({"fingerprint_data": "test-fingerprint-data"})
);
person_post_test!(person_post_favorite, "favorite", json!({}));
person_post_test!(
    person_post_investigate,
    "investigate",
    json!({"query": "background check"})
);
person_post_test!(
    person_post_fact,
    "facts",
    json!({"fact_type": "preference", "value": "Likes coffee", "confidence": 0.9})
);
person_post_test!(
    person_post_memory_card,
    "memory-cards",
    json!({"title": "Memory card", "content": "Test memory content"})
);
person_post_test!(
    person_post_preference,
    "preferences",
    json!({"key": "communication_style", "value": "direct"})
);
person_post_test!(
    person_post_timeline,
    "timeline",
    json!({"event_type": "meeting", "description": "Test meeting", "occurred_at": "2027-01-01T00:00:00Z"})
);

#[tokio::test]
async fn person_put_notes() {
    let Some(database_url) = super::support::live_database_url("person put notes").await else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .oneshot(put_request_with_token(
            &format!("/api/v1/persons/{}/notes", urlencoding_percent_encode(&pid)),
            json!({"notes": "Test notes content"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "notes status={}",
        response.status()
    );
}

#[tokio::test]
async fn person_roles_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("person roles post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{}/roles", urlencoding_percent_encode(&pid)),
            json!({"role": "colleague", "organization": "TestCo"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "role post={}",
        response.status()
    );

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/roles/colleague",
                urlencoding_percent_encode(&pid)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "role delete={}",
        response.status()
    );
}

#[tokio::test]
async fn person_persona_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("person persona post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/personas",
                urlencoding_percent_encode(&pid)
            ),
            json!({"name": "Work Persona", "description": "Professional context"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "persona post={}",
        response.status()
    );

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/personas/pers:fake",
                urlencoding_percent_encode(&pid)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "persona delete={}",
        response.status()
    );
}

#[tokio::test]
async fn person_watchlist_toggle() {
    let Some(database_url) = super::support::live_database_url("person watchlist toggle").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let app = build_persons_app(&database_url).await;
    let pid = format!("person:nonexistent-{suffix}");
    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/watchlist",
                urlencoding_percent_encode(&pid)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "watchlist toggle={}",
        response.status()
    );
}
```

### `backend/tests/persons_api/write_identity_timeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/write_identity_timeline.rs`
- Size bytes / Размер в байтах: `6884`
- Included characters / Включено символов: `6884`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::platform::storage::Database;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app_with_database, delete_request_with_token, json_body,
    post_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn person_identity_post_and_delete() {
    let Some(database_url) =
        super::support::live_database_url("person identity post and delete").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&format!("identity-delete-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let app = build_persons_app_with_database(&database_url, database);
    let pid = person.person_id.clone();
    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{}/identities",
                urlencoding_percent_encode(&pid)
            ),
            json!({
                "identity_type": "email",
                "identity_value": format!("test-{suffix}@example.com"),
                "source": "manual"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "identity post={}",
        response.status()
    );
    let post_body = json_body(response).await;
    let identity_id = post_body["id"].as_str().expect("identity id").to_owned();

    let response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/identities/{}",
                urlencoding_percent_encode(&pid),
                urlencoding_percent_encode(&identity_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "identity delete={}",
        response.status()
    );
    let delete_body = json_body(response).await;
    assert_eq!(delete_body["deleted"], json!(true));

    let delete_observation_row = sqlx::query(
        "SELECT kind.code AS kind_code, observation.origin_kind, link.metadata->>'deleted' AS deleted
         FROM observation_links link
         JOIN observations observation ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'identity'
           AND link.entity_id = $1
           AND link.relationship_kind = 'identity_delete'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&identity_id)
    .fetch_one(&pool)
    .await
    .expect("identity delete observation");
    assert_eq!(
        delete_observation_row
            .try_get::<String, _>("kind_code")
            .expect("delete kind code"),
        "PERSON_RECORD_MUTATION"
    );
    assert_eq!(
        delete_observation_row
            .try_get::<String, _>("origin_kind")
            .expect("delete origin kind"),
        "manual"
    );
    assert_eq!(
        delete_observation_row
            .try_get::<String, _>("deleted")
            .expect("delete metadata"),
        "true"
    );
}

#[tokio::test]
async fn person_relationship_timeline_entrypoint_captures_observation_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("person relationship timeline observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&format!("relationship-event-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let app = build_persons_app_with_database(&database_url, database);
    let encoded_person_id = urlencoding_percent_encode(&person.person_id);

    let response = app
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/timeline"),
            json!({
                "event_type": "meeting",
                "title": format!("Meeting with {suffix}"),
                "description": "Manual relationship event should be observation-backed.",
                "occurred_at": "2027-01-01T00:00:00Z",
                "source": "manual",
                "related_entity_id": format!("evt:v1:{suffix}"),
                "related_entity_kind": "event"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("timeline response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let event_id = json_body(response).await["id"]
        .as_str()
        .expect("relationship event id")
        .to_owned();

    let source: String =
        sqlx::query_scalar("SELECT source FROM relationship_events WHERE id::text = $1")
            .bind(&event_id)
            .fetch_one(&pool)
            .await
            .expect("relationship event source");
    assert!(source.starts_with("observation:"));

    let observation_id = source
        .strip_prefix("observation:")
        .expect("observation prefix");
    let origin_kind: String =
        sqlx::query_scalar("SELECT origin_kind FROM observations WHERE observation_id = $1")
            .bind(observation_id)
            .fetch_one(&pool)
            .await
            .expect("relationship event observation");
    assert_eq!(origin_kind, "manual");
    let kind_code: String = sqlx::query_scalar(
        "SELECT kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(observation_id)
    .fetch_one(&pool)
    .await
    .expect("relationship event observation kind");
    assert_eq!(kind_code, "PERSON_RECORD_MUTATION");

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE observation_id = $1
           AND domain = 'persons'
           AND entity_kind = 'relationship_event'
           AND entity_id = $2",
    )
    .bind(observation_id)
    .bind(&event_id)
    .fetch_one(&pool)
    .await
    .expect("relationship event observation link count");
    assert_eq!(link_count, 1);
}
```

### `backend/tests/persons_api/write_memory_observations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/write_memory_observations.rs`
- Size bytes / Размер в байтах: `12171`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::platform::storage::Database;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app_with_database, json_body, post_request_with_token,
    put_request_with_token, run_person_derived_evidence_consumer, unique_suffix,
    urlencoding_percent_encode,
};

#[tokio::test]
async fn person_manual_memory_entrypoints_capture_observations_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("person manual memory observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&format!("manual-memory-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let app = build_persons_app_with_database(&database_url, database);
    let encoded_person_id = urlencoding_percent_encode(&person.person_id);

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/notes"),
            json!({"notes": "Manual persona notes from observation-backed API"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("notes response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let notes_card_source: String = sqlx::query_scalar(
        r#"
        SELECT source
        FROM person_memory_cards
        WHERE person_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("notes memory card source");
    assert!(notes_card_source.starts_with("observation:"));
    let notes_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'notes'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("notes observation link");

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/facts"),
            json!({
                "fact_type": "preference",
                "value": "local-first architecture",
                "confidence": 0.91
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fact response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let fact_body = json_body(response).await;
    let fact_id = fact_body["id"].as_str().expect("fact id");
    let fact_source: String =
        sqlx::query_scalar("SELECT source FROM person_facts WHERE id::text = $1")
            .bind(fact_id)
            .fetch_one(&pool)
            .await
            .expect("fact source");
    assert!(fact_source.starts_with("observation:"));

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/memory-cards"),
            json!({
                "title": "NAS shortlist",
                "description": "Shortlisted Synology and QNAP options",
                "importance": 7
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("memory card response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let card_body = json_body(response).await;
    let card_id = card_body["id"].as_str().expect("card id");
    let card_source: String =
        sqlx::query_scalar("SELECT source FROM person_memory_cards WHERE id::text = $1")
            .bind(card_id)
            .fetch_one(&pool)
            .await
            .expect("memory card source");
    assert!(card_source.starts_with("observation:"));

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/preferences"),
            json!({
                "preference_type": "timezone",
                "value": "Europe/Madrid"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("preference response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let pref_body = json_body(response).await;
    let pref_id = pref_body["id"].as_str().expect("preference id");
    let pref_source: String =
        sqlx::query_scalar("SELECT source FROM person_preferences WHERE id::text = $1")
            .bind(pref_id)
            .fetch_one(&pool)
            .await
            .expect("preference source");
    assert!(pref_source.starts_with("observation:"));

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/fingerprint"),
            json!({"fingerprint_data": "manual-fingerprint-trigger"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("fingerprint response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    run_person_derived_evidence_consumer(pool.clone()).await;

    let fingerprint_observation_row = sqlx::query(
        "SELECT observation.observation_id, observation.origin_kind, kind.code AS kind_code
         FROM observation_links link
         JOIN observations observation
           ON observation.observation_id = link.observation_id
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE link.domain = 'persons'
           AND link.entity_kind = 'persona'
           AND link.entity_id = $1
           AND link.relationship_kind = 'profile_enrichment'
         ORDER BY link.created_at DESC
         LIMIT 1",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("fingerprint observation row");
    assert_eq!(
        fingerprint_observation_row
            .try_get::<String, _>("origin_kind")
            .expect("fingerprint origin"),
        "manual"
    );
    assert_eq!(
        fingerprint_observation_row
            .try_get::<String, _>("kind_code")
            .expect("fingerprint kind"),
        "PERSON_MUTATION"
    );

    let trust_signal_count: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE kind.code = 'PERSON_TRUST_SIGNAL'
           AND observation.payload->>'person_id' = $1",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person trust signal count");
    assert!(trust_signal_count >= 1);

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/favorite"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("favorite response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let favorite_source: String = sqlx::query_scalar(
        "SELECT source FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("favorite source");
    assert!(favorite_source.starts_with("observation:"));
    let favorite_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'favorite_toggle'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("favorite observation link");

    let response = app
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/watchlist"),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("watchlist response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let watchlist_source: String = sqlx::query_scalar(
        "SELECT source FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:watchlist'",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("watchlist source");
    assert!(watchlist_source.starts_with("observation:"));
    let watchlist_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'watchlist_toggle'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("watchlist observation link");

    for source in [
        notes_card_source.clone(),
        fact_source.clone(),
        card_source.clone(),
        pref_source.clone(),
        favorite_source.clone(),
        watchlist_source.clone(),
    ] {
        let observation_id = source
            .strip_prefix("observation:")
            .expect("observation source prefix");
        let row = sqlx::query(
            "SELECT observation.observation_id, observation.origin_kind, kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("stored observation");
        assert_eq!(
            row.try_get::<String, _>("origin_kind")
                .expect("origin kind"),
            "manual"
        );
    }

    for source in [fact_source, pref_source] {
        let observation_id = source
            .strip_prefix("observation:")
            .expect("observation source prefix");
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("person record mutation kind code");
        assert_eq!(kind_code, "PERSON_RECORD_MUTATION");
    }

    for observation_id in [
        notes_observation_id,
        notes_card_source
            .strip_prefix("observation:")
            .expect("observation source prefix")
            .to_owned(),
        card_source
            .strip_prefix("observation:")
            .expect("observation source prefix")
            .to_owned(),
    ] {
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(observation_id)
        .fetch_one(&pool)
        .await
        .expect("person memory card kind code");
        assert_eq!(kind_code, "PERSON_MEMORY_CARD");
    }

    for observation_id in [favorite_observation_id, watchlist_observation_id] {
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/persons_api/write_review_observations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api/write_review_observations.rs`
- Size bytes / Размер в байтах: `14056`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::domains::persons::api::PersonProjectionStore;
use makosh_hub_backend::domains::persons::enrichment_engine::EnrichmentResultStore;
use makosh_hub_backend::domains::persons::identity::PersonIdentityStore;
use makosh_hub_backend::platform::storage::Database;

use super::support::{
    LOCAL_API_TOKEN, build_persons_app_with_database, delete_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};

#[tokio::test]
async fn person_enrichment_review_entrypoints_capture_observations_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("person enrichment review observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&format!("manual-enrichment-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let enrichment = EnrichmentResultStore::new(pool.clone())
        .upsert(
            &person.person_id,
            "linkedin",
            json!({
                "extracted_claim": "Works on canonical evidence architecture"
            }),
            0.88,
        )
        .await
        .expect("create enrichment result");
    let app = build_persons_app_with_database(&database_url, database);
    let encoded_person_id = urlencoding_percent_encode(&person.person_id);

    let response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{encoded_person_id}/enrichment/{}/apply",
                urlencoding_percent_encode(&enrichment.id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("enrichment apply response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let apply_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'enrichment_result'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&enrichment.id)
    .fetch_one(&pool)
    .await
    .expect("enrichment apply observation link count");
    assert_eq!(apply_link_count, 1);

    let enrichment_rejected = EnrichmentResultStore::new(pool.clone())
        .upsert(
            &person.person_id,
            "telegram",
            json!({
                "extracted_claim": "Prefers async communication"
            }),
            0.74,
        )
        .await
        .expect("create enrichment reject result");

    let response = app
        .oneshot(post_request_with_token(
            &format!(
                "/api/v1/persons/{encoded_person_id}/enrichment/{}/reject",
                urlencoding_percent_encode(&enrichment_rejected.id)
            ),
            json!({}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("enrichment reject response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let reject_link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'enrichment_result'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&enrichment_rejected.id)
    .fetch_one(&pool)
    .await
    .expect("enrichment reject observation link count");
    assert_eq!(reject_link_count, 1);
}

#[tokio::test]
async fn person_compatibility_entrypoints_capture_observations_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("person compatibility observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&format!("manual-compat-{suffix}@example.com"))
        .await
        .expect("upsert person");
    let app = build_persons_app_with_database(&database_url, database);
    let encoded_person_id = urlencoding_percent_encode(&person.person_id);

    let role_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/roles"),
            json!({"role": "colleague"}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("role response");
    assert_eq!(role_response.status(), axum::http::StatusCode::OK);
    let role_id = json_body(role_response).await["id"]
        .as_str()
        .expect("role id")
        .to_owned();
    let role_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'role'
           AND entity_id = $1
           AND metadata ->> 'action' = 'assign'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&role_id)
    .fetch_one(&pool)
    .await
    .expect("role observation link");

    let persona_response = app
        .clone()
        .oneshot(post_request_with_token(
            &format!("/api/v1/persons/{encoded_person_id}/personas"),
            json!({
                "persona_id": format!("pers:v1:manual:{suffix}"),
                "name": "Work Persona",
                "context": "Professional context",
                "preferred_channel": "email"
            }),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("persona response");
    assert_eq!(persona_response.status(), axum::http::StatusCode::OK);
    let persona_id = json_body(persona_response).await["persona_id"]
        .as_str()
        .expect("persona id")
        .to_owned();
    let persona_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'persona'
           AND entity_id = $1
           AND metadata ->> 'action' = 'upsert'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&persona_id)
    .fetch_one(&pool)
    .await
    .expect("persona observation link");
    let persona_pref_source: String = sqlx::query_scalar(
        "SELECT source
         FROM person_preferences
         WHERE person_id = $1
           AND preference_type = $2",
    )
    .bind(&person.person_id)
    .bind(format!(
        "interaction_context:{persona_id}:preferred_channel"
    ))
    .fetch_one(&pool)
    .await
    .expect("persona preference source");
    assert!(persona_pref_source.starts_with("observation:"));

    let delete_role_response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/roles/colleague",
                urlencoding_percent_encode(&person.person_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete role response");
    assert_eq!(delete_role_response.status(), axum::http::StatusCode::OK);
    let delete_role_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'role'
           AND entity_id = $1
           AND metadata ->> 'action' = 'delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(format!("{}:{}", person.person_id, "colleague"))
    .fetch_one(&pool)
    .await
    .expect("role delete observation link");

    let delete_persona_response = app
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/persons/{}/personas/{}",
                urlencoding_percent_encode(&person.person_id),
                urlencoding_percent_encode(&persona_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete persona response");
    assert_eq!(delete_persona_response.status(), axum::http::StatusCode::OK);
    let delete_persona_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'persons'
           AND entity_kind = 'persona'
           AND entity_id = $1
           AND metadata ->> 'action' = 'delete'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&persona_id)
    .fetch_one(&pool)
    .await
    .expect("persona delete observation link");

    for observation_id in [
        role_observation_id,
        persona_observation_id,
        persona_pref_source
            .strip_prefix("observation:")
            .expect("persona pref observation prefix")
            .to_owned(),
        delete_role_observation_id,
        delete_persona_observation_id,
    ] {
        let row = sqlx::query(
            "SELECT observation.observation_id, observation.origin_kind, kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(&observation_id)
        .fetch_one(&pool)
        .await
        .expect("stored observation");
        assert_eq!(
            row.try_get::<String, _>("origin_kind")
                .expect("origin kind"),
            "manual"
        );
        assert_eq!(
            row.try_get::<String, _>("kind_code").expect("kind code"),
            "PERSON_RECORD_MUTATION"
        );
    }
}

#[tokio::test]
async fn identity_candidate_review_captures_observation_against_postgres() {
    let Some(database_url) =
        super::support::live_database_url("identity candidate review observations").await
    else {
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let suffix = unique_suffix();
    let person_store = PersonProjectionStore::new(pool.clone());
    let shared_name = format!("Identity Review Observation {suffix}");
    let left = person_store
        .upsert_email_person(&format!("identity-review-left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = person_store
        .upsert_email_person(&format!("identity-review-right-{suffix}@example.com"))
        .await
        .expect("upsert right person");
    sqlx::query(
        r#"
        UPDATE persons
        SET display_name = $1
        WHERE person_id = $2 OR person_id = $3
        "#,
    )
    .bind(&shared_name)
    .bind(&left.person_id)
    .bind(&right.person_id)
    .execute(&pool)
    .await
    .expect("seed display names");

    let _ = PersonIdentityStore::new(pool.clone())
        .refresh_candidates(100)
        .await
        .expect("refresh identity candidates");
    let (left_person_id, right_person_id) = if left.person_id <= right.person_id {
        (left.person_id.clone(), right.person_id.clone())
    } else {
        (right.person_id.clone(), left.person_id.clone())
    };
    let identity_candidate_id =
        format!("identity_candidate:v1:merge_persons:{left_person_id}:{right_person_id}");

    let app = build_persons_app_with_database(&database_url, database);
    let response = app
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/identity-candidates/{}/review",
                urlencoding_percent_encode(&identity_candidate_id)
            ),
            json!({
                "command_id": format!
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/tests/persons_api_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_api_architecture.rs`
- Size bytes / Размер в байтах: `1962`
- Included characters / Включено символов: `1962`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn persons_api_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_persons_api_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "persons API test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_persons_api_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_persons_api_test_violations(&path, violations);
            continue;
        }
        if !is_persons_api_test_file(&path) {
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

fn is_persons_api_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "persons_api.rs" || file_name == "persons_api_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "persons_api")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

### `backend/tests/persons_architecture.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/tests/persons_architecture.rs`
- Size bytes / Размер в байтах: `1922`
- Included characters / Включено символов: `1922`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn persons_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_persons_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "persons test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_persons_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_persons_test_violations(&path, violations);
            continue;
        }
        if !is_persons_test_file(&path) {
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

fn is_persons_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "persons.rs" || file_name == "persons_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "persons")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
```

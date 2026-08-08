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

- Chunk ID / ID чанка: `037-source-backend-part-017`
- Group / Группа: `backend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/backend.md`

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

### `backend/src/domains/calendar/events/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/events/rows.rs`
- Size bytes / Размер в байтах: `1477`
- Included characters / Включено символов: `1477`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::models::CalendarEvent;

pub(super) fn row_to_event(row: PgRow) -> Result<CalendarEvent, sqlx::Error> {
    Ok(CalendarEvent {
        event_id: row.try_get("event_id")?,
        observation_id: row.try_get("observation_id")?,
        source_event_id: row.try_get("source_event_id")?,
        account_id: row.try_get("account_id")?,
        source_id: row.try_get("source_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        location: row.try_get("location")?,
        start_at: row.try_get("start_at")?,
        end_at: row.try_get("end_at")?,
        timezone: row.try_get("timezone")?,
        all_day: row.try_get("all_day")?,
        recurrence_rule: row.try_get("recurrence_rule")?,
        status: row.try_get("status")?,
        visibility: row.try_get("visibility")?,
        event_type: row.try_get("event_type")?,
        importance_score: row.try_get("importance_score")?,
        readiness_score: row.try_get("readiness_score")?,
        sync_status: row.try_get("sync_status")?,
        conference_url: row.try_get("conference_url")?,
        conference_provider: row.try_get("conference_provider")?,
        preparation_reminder_minutes: row.try_get("preparation_reminder_minutes")?,
        travel_buffer_minutes: row.try_get("travel_buffer_minutes")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/calendar/events/source_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/events/source_store.rs`
- Size bytes / Размер в байтах: `6117`
- Included characters / Включено символов: `6117`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use super::errors::CalendarError;
use super::models::CalendarSource;
use crate::platform::observations::link_domain_entity_in_transaction;

#[derive(Clone)]
pub struct CalendarSourceStore {
    pool: PgPool,
}

impl CalendarSourceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<CalendarSource, CalendarError> {
        self.create_with_observation(
            account_id,
            name,
            provider_calendar_id,
            color,
            timezone,
            None,
            "create",
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarSource, CalendarError> {
        let source_id = next_id("src");
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO calendar_sources (source_id, account_id, provider_calendar_id, name, color, timezone) VALUES ($1,$2,$3,$4,$5,$6) RETURNING source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at",
        )
        .bind(&source_id)
        .bind(account_id)
        .bind(provider_calendar_id)
        .bind(name)
        .bind(color)
        .bind(timezone)
        .fetch_one(&mut *transaction)
        .await?;
        let source = row_to_calendar_source(row).map_err(CalendarError::from)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                VaultOwnedEntityLink {
                    observation_id: observation_id.to_owned(),
                    domain: "calendar",
                    entity_kind: "calendar_source",
                    entity_id: source.source_id.clone(),
                    relationship_kind: relationship_kind.to_owned(),
                    base_metadata: json!({
                        "source_id": source.source_id,
                        "account_id": source.account_id,
                    }),
                    extra_metadata: metadata,
                },
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(source)
    }

    pub async fn list_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<CalendarSource>, CalendarError> {
        let rows = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE account_id=$1 ORDER BY name")
            .bind(account_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_calendar_source)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn get(&self, source_id: &str) -> Result<Option<CalendarSource>, CalendarError> {
        let row = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE source_id=$1")
            .bind(source_id).fetch_optional(&self.pool).await?;
        row.map(row_to_calendar_source)
            .transpose()
            .map_err(CalendarError::from)
    }
}
struct VaultOwnedEntityLink {
    observation_id: String,
    domain: &'static str,
    entity_kind: &'static str,
    entity_id: String,
    relationship_kind: String,
    base_metadata: serde_json::Value,
    extra_metadata: Option<serde_json::Value>,
}

async fn link_vault_owned_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    request: VaultOwnedEntityLink,
) -> Result<(), crate::platform::observations::ObservationStoreError> {
    let metadata = match request.extra_metadata {
        Some(extra) if request.base_metadata.is_object() && extra.is_object() => {
            let mut merged = request.base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => request.base_metadata,
    };

    link_domain_entity_in_transaction(
        transaction,
        &request.observation_id,
        request.domain,
        request.entity_kind,
        request.entity_id,
        Some(&request.relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
fn row_to_calendar_source(row: PgRow) -> Result<CalendarSource, sqlx::Error> {
    Ok(CalendarSource {
        source_id: row.try_get("source_id")?,
        account_id: row.try_get("account_id")?,
        provider_calendar_id: row.try_get("provider_calendar_id")?,
        name: row.try_get("name")?,
        color: row.try_get("color")?,
        timezone: row.try_get("timezone")?,
        visibility: row.try_get("visibility")?,
        read_only: row.try_get("read_only")?,
        sync_enabled: row.try_get("sync_enabled")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
fn next_id(prefix: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}:v1:{ts:x}")
}
```

### `backend/src/domains/calendar/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/evidence.rs`
- Size bytes / Размер в байтах: `1959`
- Included characters / Включено символов: `1959`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use crate::platform::observations::{
    ObservationStoreError, link_domain_entity, link_domain_entity_in_transaction,
};

pub(crate) async fn link_calendar_entity(
    pool: &PgPool,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    base_metadata: Value,
    extra_metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let metadata = merge_metadata(base_metadata, extra_metadata);
    link_domain_entity(
        pool,
        observation_id,
        "calendar",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        Some(metadata),
    )
    .await
}

pub(crate) async fn link_calendar_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    base_metadata: Value,
    extra_metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    let metadata = merge_metadata(base_metadata, extra_metadata);
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "calendar",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        Some(metadata),
    )
    .await
}

fn merge_metadata(base_metadata: Value, extra_metadata: Option<Value>) -> Value {
    match extra_metadata {
        Some(extra) if base_metadata.is_object() && extra.is_object() => {
            let mut merged = base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base_metadata,
    }
}
```

### `backend/src/domains/calendar/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/health.rs`
- Size bytes / Размер в байтах: `8683`
- Included characters / Включено символов: `8575`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

pub struct CalendarWatchtowerService;

impl CalendarWatchtowerService {
    pub async fn events_needing_preparation(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let soon = Utc::now() + Duration::hours(24);
        let now = Utc::now();
        let rows = sqlx::query(
            "SELECT ce.event_id, ce.title, ce.start_at, ce.status, ce.readiness_score FROM calendar_events ce WHERE ce.start_at BETWEEN $1 AND $2 AND ce.status = 'scheduled' AND (ce.readiness_score IS NULL OR ce.readiness_score < 0.5) ORDER BY ce.start_at ASC LIMIT 20"
        ).bind(now).bind(soon).fetch_all(pool).await?;
        let items: Vec<Value> = rows.iter().map(|r| json!({
            "event_id": r.try_get::<String, _>("event_id").unwrap_or_default(),
            "title": r.try_get::<String, _>("title").unwrap_or_default(),
            "start_at": r.try_get::<DateTime<Utc>, _>("start_at").ok(),
            "status": r.try_get::<String, _>("status").unwrap_or_default(),
            "readiness_score": r.try_get::<Option<f64>, _>("readiness_score").unwrap_or(None),
        })).collect();
        Ok(json!({"events_needing_preparation": items}))
    }

    pub async fn events_without_outcomes(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let now = Utc::now();
        let rows = sqlx::query(
            "SELECT ce.event_id, ce.title, ce.start_at FROM calendar_events ce LEFT JOIN meeting_notes mn ON ce.event_id = mn.event_id WHERE ce.end_at < $1 AND ce.status IN ('completed', 'in_progress') AND mn.id IS NULL ORDER BY ce.start_at DESC LIMIT 20"
        ).bind(now).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "event_id": r.try_get::<String, _>("event_id").unwrap_or_default(),
                    "title": r.try_get::<String, _>("title").unwrap_or_default(),
                    "start_at": r.try_get::<DateTime<Utc>, _>("start_at").ok(),
                })
            })
            .collect();
        Ok(json!({"events_without_notes": items}))
    }

    pub async fn weekly_brief(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let now = Utc::now();
        let week_end = now + Duration::days(7);

        // Upcoming events this week
        let events = sqlx::query("SELECT COUNT(*) as cnt FROM calendar_events WHERE start_at BETWEEN $1 AND $2 AND status='scheduled'")
            .bind(now).bind(week_end).fetch_one(pool).await?;
        let event_count: i64 = events.try_get("cnt")?;

        // Overdue deadlines
        let deadlines = sqlx::query(
            "SELECT COUNT(*) as cnt FROM deadline_events WHERE due_at < $1 AND status='active'",
        )
        .bind(now)
        .fetch_one(pool)
        .await?;
        let overdue_count: i64 = deadlines.try_get("cnt")?;

        // Past events without outcomes
        let past_no_outcomes = sqlx::query(
            "SELECT COUNT(*) as cnt FROM calendar_events ce LEFT JOIN meeting_notes mn ON ce.event_id=mn.event_id WHERE ce.end_at < $1 AND ce.status IN ('completed','in_progress') AND mn.id IS NULL"
        ).bind(now).fetch_one(pool).await?;
        let no_notes_count: i64 = past_no_outcomes.try_get("cnt")?;

        Ok(json!({
            "upcoming_events_this_week": event_count,
            "overdue_deadlines": overdue_count,
            "past_events_without_notes": no_notes_count,
            "week_start": now,
            "week_end": week_end,
        }))
    }

    pub async fn meeting_load_analysis(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let now = Utc::now();
        let week_ago = now - Duration::days(7);
        let rows = sqlx::query(
            "SELECT date_trunc('day', start_at) as day, COUNT(*) as cnt, COALESCE(SUM(EXTRACT(EPOCH FROM (end_at - start_at))/3600),0) as hours FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 GROUP BY day ORDER BY day"
        ).bind(week_ago).bind(now).fetch_all(pool).await?;
        let days: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "day": r.try_get::<Option<DateTime<Utc>>, _>("day").ok(),
                    "event_count": r.try_get::<Option<i64>, _>("cnt").unwrap_or(Some(0)),
                    "hours": r.try_get::<Option<f64>, _>("hours").unwrap_or(Some(0.0)),
                })
            })
            .collect();
        Ok(json!({"daily_load": days}))
    }
}

#[derive(Debug, Error)]
pub enum CalendarHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

// ── Expanded Analytics ────────────────────────────────────────────────────

impl CalendarWatchtowerService {
    /// Time distribution by category for a given week
    pub async fn time_distribution(
        pool: &PgPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Value, CalendarHealthError> {
        let rows = sqlx::query(
            "SELECT COALESCE(event_type, 'other') as cat, COUNT(*) as cnt, COALESCE(SUM(EXTRACT(EPOCH FROM (end_at - start_at))/3600), 0) as hours FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 AND status NOT IN ('cancelled', 'no_show') GROUP BY cat ORDER BY hours DESC"
        ).bind(from).bind(to).fetch_all(pool).await?;
        let categories: Vec<Value> = rows.iter().map(|r| json!({
            "category": crate::domains::calendar::intelligence::CalendarIntelligenceService::categorize_time(
                r.try_get::<String, _>("cat").unwrap_or_default().as_str(),
                ""
            ),
            "event_count": r.try_get::<Option<i64>, _>("cnt").unwrap_or(Some(0)),
            "hours": r.try_get::<Option<f64>, _>("hours").unwrap_or(Some(0.0)),
        })).collect();
        Ok(json!({"time_distribution": categories, "from": from, "to": to}))
    }

    /// Focus vs meetings balance
    pub async fn focus_balance(
        pool: &PgPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Value, CalendarHealthError> {
        let rows = sqlx::query(
            "SELECT COALESCE(event_type, 'other') as cat, COALESCE(SUM(EXTRACT(EPOCH FROM (end_at - start_at))/3600), 0) as hours FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 AND status NOT IN ('cancelled', 'no_show') GROUP BY cat"
        ).bind(from).bind(to).fetch_all(pool).await?;
        let mut meetings_h = 0f64;
        let mut focus_h = 0f64;
        let mut other_h = 0f64;
        for r in &rows {
            let cat: String = r.try_get("cat").unwrap_or_default();
            let h: f64 = r.try_get("hours").unwrap_or(0.0);
            match cat.as_str() {
                "meeting" => meetings_h += h,
                "focus" => focus_h += h,
                _ => other_h += h,
            }
        }
        let total = meetings_h + focus_h + other_h;
        let focus_pct = if total > 0.0 {
            (focus_h / total * 100.0).round()
        } else {
            0.0
        };
        let warning = if focus_h < meetings_h * 0.3 {
            Some("Low focus time relative to meetings")
        } else {
            None
        };
        Ok(json!({
            "meetings_hours": meetings_h, "focus_hours": focus_h, "other_hours": other_h,
            "total_hours": total, "focus_percentage": focus_pct, "warning": warning,
            "from": from, "to": to,
        }))
    }

    /// Back-to-back meeting detection
    pub async fn back_to_back_meetings(
        pool: &PgPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Value, CalendarHealthError> {
        let rows = sqlx::query(
            "SELECT title, start_at, end_at FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 AND event_type = 'meeting' AND status NOT IN ('cancelled', 'no_show') ORDER BY start_at"
        ).bind(from).bind(to).fetch_all(pool).await?;
        let events: Vec<(DateTime<Utc>, DateTime<Utc>, String)> = rows
            .iter()
            .map(|r| {
                (
                    r.try_get("start_at").unwrap_or(Utc::now()),
                    r.try_get("end_at").unwrap_or(Utc::now()),
                    r.try_get("title").unwrap_or_default(),
                )
            })
            .collect();
        let groups =
            crate::domains::calendar::intelligence::CalendarIntelligenceService::detect_back_to_back(&events);
        Ok(json!({"back_to_back_groups": groups, "from": from, "to": to}))
    }
}
```

### `backend/src/domains/calendar/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence.rs`
- Size bytes / Размер в байтах: `286`
- Included characters / Включено символов: `286`
- Truncated / Обрезано: `no`

```rust
mod analytics;
mod classification;
mod conference;
mod errors;
mod fingerprint;
mod location;
mod models;
mod scoring;

pub struct CalendarIntelligenceService;

pub use errors::CalendarIntelligenceError;
pub use models::{BackToBackGroup, EventAnalysis, EventFingerprint, LocationInfo};
```

### `backend/src/domains/calendar/intelligence/analytics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/analytics.rs`
- Size bytes / Размер в байтах: `2334`
- Included characters / Включено символов: `2334`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::{BackToBackGroup, CalendarIntelligenceService};

impl CalendarIntelligenceService {
    pub fn categorize_time(event_type: &str, title: &str) -> String {
        let title = title.to_lowercase();
        match event_type {
            "meeting" => "meetings".into(),
            "focus" => "focus".into(),
            "deadline" => "deadlines".into(),
            "travel" => "travel".into(),
            "tax" | "legal" | "government" => "admin".into(),
            "finance" => "finance".into(),
            "personal" | "birthday" => "personal".into(),
            "review" | "planning" => "planning".into(),
            _ => fallback_time_category(&title),
        }
    }

    pub fn detect_back_to_back(
        events: &[(DateTime<Utc>, DateTime<Utc>, String)],
    ) -> Vec<BackToBackGroup> {
        let mut sorted = events.to_vec();
        sorted.sort_by_key(|(start_at, _, _)| *start_at);
        let mut groups = Vec::new();
        let mut current: Vec<String> = Vec::new();

        for window in sorted.windows(2) {
            let (_, first_end_at, first_title) = &window[0];
            let (second_start_at, _, second_title) = &window[1];
            let gap = (*second_start_at - *first_end_at).num_minutes();

            if gap <= 5 && current.is_empty() {
                current.push(first_title.clone());
                current.push(second_title.clone());
            } else if gap <= 5 {
                current.push(second_title.clone());
            } else if !current.is_empty() {
                groups.push(BackToBackGroup {
                    titles: current.clone(),
                    count: current.len(),
                });
                current.clear();
            }
        }

        if !current.is_empty() {
            groups.push(BackToBackGroup {
                titles: current.clone(),
                count: current.len(),
            });
        }
        groups
    }
}

fn fallback_time_category(title: &str) -> String {
    if title.contains("meeting") || title.contains("call") {
        "meetings".into()
    } else if title.contains("focus") {
        "focus".into()
    } else if title.contains("lunch") || title.contains("dinner") || title.contains("coffee") {
        "personal".into()
    } else {
        "other".into()
    }
}
```

### `backend/src/domains/calendar/intelligence/classification.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/classification.rs`
- Size bytes / Размер в байтах: `1868`
- Included characters / Включено символов: `1776`
- Truncated / Обрезано: `no`

```rust
use super::CalendarIntelligenceService;

impl CalendarIntelligenceService {
    pub fn classify_event(title: &str, participants_count: usize, duration_minutes: i64) -> String {
        let title = title.to_lowercase();
        if contains_any(&title, &["meeting", "call", "sync", "созвон", "встреча"]) {
            return "meeting".into();
        }
        if contains_any(&title, &["deadline", "due", "срок", "дедлайн"]) {
            return "deadline".into();
        }
        if contains_any(&title, &["focus", "deep work", "фокус"]) {
            return "focus".into();
        }
        if contains_any(&title, &["travel", "flight", "поездка", "перелёт"]) {
            return "travel".into();
        }
        if contains_any(&title, &["review", "обзор", "ревью"]) {
            return "review".into();
        }
        if contains_any(&title, &["planning", "план"]) {
            return "planning".into();
        }
        if contains_any(&title, &["tax", "налог", "aeat", "declaracion"]) {
            return "tax".into();
        }
        if contains_any(&title, &["legal", "abogado", "lawyer"]) {
            return "legal".into();
        }
        if contains_any(&title, &["finance", "invoice", "счёт", "фактура"]) {
            return "finance".into();
        }
        if contains_any(&title, &["birthday", "день рождения"]) {
            return "birthday".into();
        }
        if contains_any(&title, &["reminder", "напомин"]) {
            return "reminder".into();
        }
        if participants_count > 2 || duration_minutes > 120 {
            return "meeting".into();
        }
        "personal".into()
    }
}

pub(super) fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
```

### `backend/src/domains/calendar/intelligence/conference.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/conference.rs`
- Size bytes / Размер в байтах: `1413`
- Included characters / Включено символов: `1413`
- Truncated / Обрезано: `no`

```rust
use super::CalendarIntelligenceService;

impl CalendarIntelligenceService {
    pub fn detect_conference_provider(url: &str) -> Option<String> {
        let url = url.to_lowercase();
        if url.contains("meet.google.com") {
            return Some("google_meet".into());
        }
        if url.contains("zoom.us") || url.contains("zoom.com") {
            return Some("zoom".into());
        }
        if url.contains("teams.microsoft.com") || url.contains("teams.live.com") {
            return Some("microsoft_teams".into());
        }
        if url.contains("meet.jit.si") {
            return Some("jitsi".into());
        }
        if url.contains("webex.com") {
            return Some("webex".into());
        }
        None
    }

    pub fn extract_conference_url(text: &str) -> Option<String> {
        let patterns = [
            "https://meet.google.com/",
            "https://zoom.us/j/",
            "https://teams.microsoft.com/l/meetup-join/",
            "https://meet.jit.si/",
        ];
        let lower = text.to_lowercase();
        for prefix in &patterns {
            if let Some(pos) = lower.find(prefix) {
                let end = lower[pos..]
                    .find(|character: char| character.is_whitespace())
                    .unwrap_or(lower[pos..].len());
                return Some(text[pos..pos + end].to_string());
            }
        }
        None
    }
}
```

### `backend/src/domains/calendar/intelligence/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/errors.rs`
- Size bytes / Размер в байтах: `151`
- Included characters / Включено символов: `151`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalendarIntelligenceError {
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),
}
```

### `backend/src/domains/calendar/intelligence/fingerprint.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/fingerprint.rs`
- Size bytes / Размер в байтах: `1550`
- Included characters / Включено символов: `1523`
- Truncated / Обрезано: `no`

```rust
use super::classification::contains_any;
use super::{CalendarIntelligenceService, EventFingerprint};

impl CalendarIntelligenceService {
    pub fn heuristic_fingerprint(
        title: &str,
        description: Option<&str>,
        event_type: &str,
    ) -> EventFingerprint {
        let combined = format!("{} {}", title, description.unwrap_or(""));
        let lower = combined.to_lowercase();
        let mut fingerprint = EventFingerprint {
            event_type: if event_type.trim().is_empty() {
                CalendarIntelligenceService::classify_event(title, 1, 60)
            } else {
                event_type.to_owned()
            },
            ..Default::default()
        };

        fingerprint.importance = fingerprint_importance(&lower);
        fingerprint.language = if contains_any(&lower, &["испанск", "espanol"]) {
            Some("es".into())
        } else {
            Some("en".into())
        };
        fingerprint.recurrence_hint = recurrence_hint(&lower);
        fingerprint
    }
}

fn fingerprint_importance(value: &str) -> f64 {
    if contains_any(value, &["important", "critical", "важно"]) {
        0.8
    } else if contains_any(value, &["client", "tax", "legal"]) {
        0.7
    } else {
        0.4
    }
}

fn recurrence_hint(value: &str) -> Option<String> {
    if contains_any(value, &["weekly", "еженедел"]) {
        Some("weekly".into())
    } else if contains_any(value, &["daily", "ежеднев"]) {
        Some("daily".into())
    } else {
        None
    }
}
```

### `backend/src/domains/calendar/intelligence/location.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/location.rs`
- Size bytes / Размер в байтах: `1620`
- Included characters / Включено символов: `1584`
- Truncated / Обрезано: `no`

```rust
use super::{CalendarIntelligenceService, LocationInfo};

impl CalendarIntelligenceService {
    pub fn parse_location(location: &str) -> LocationInfo {
        let lower = location.to_lowercase();
        let is_online = is_online_location(&lower);
        let parsed_name = parsed_location_name(location, &lower, is_online);
        let travel_buffer_minutes = if is_online { None } else { Some(15i32) };

        LocationInfo {
            is_online,
            parsed_name,
            travel_buffer_minutes,
        }
    }
}

fn is_online_location(location: &str) -> bool {
    location.contains("online")
        || location.contains("virtual")
        || location.contains("zoom")
        || location.contains("meet.google")
        || location.contains("teams.microsoft")
        || location.contains("video call")
        || location.contains("видеозвонок")
}

fn parsed_location_name(original: &str, lower: &str, is_online: bool) -> Option<String> {
    if lower.contains("office") || lower.contains("офис") {
        Some("Office".into())
    } else if lower.contains("home") || lower.contains("дома") {
        Some("Home".into())
    } else if lower.contains("cafe") || lower.contains("coffee") || lower.contains("кафе") {
        Some("Cafe".into())
    } else if lower.contains("airport") || lower.contains("аэропорт") {
        Some("Airport".into())
    } else if lower.contains("hotel") || lower.contains("отель") {
        Some("Hotel".into())
    } else if !is_online && !original.is_empty() {
        Some(original.to_string())
    } else {
        None
    }
}
```

### `backend/src/domains/calendar/intelligence/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/models.rs`
- Size bytes / Размер в байтах: `793`
- Included characters / Включено символов: `793`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventAnalysis {
    pub event_type: String,
    pub importance_score: f64,
    pub readiness_score: f64,
    pub risks: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventFingerprint {
    pub event_type: String,
    pub importance: f64,
    pub language: Option<String>,
    pub recurrence_hint: Option<String>,
    pub topics: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationInfo {
    pub is_online: bool,
    pub parsed_name: Option<String>,
    pub travel_buffer_minutes: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackToBackGroup {
    pub titles: Vec<String>,
    pub count: usize,
}
```

### `backend/src/domains/calendar/intelligence/scoring.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/intelligence/scoring.rs`
- Size bytes / Размер в байтах: `2228`
- Included characters / Включено символов: `2216`
- Truncated / Обрезано: `no`

```rust
use super::CalendarIntelligenceService;
use super::classification::contains_any;

impl CalendarIntelligenceService {
    pub fn calculate_importance(
        title: &str,
        participants_count: usize,
        has_project: bool,
        has_deadline: bool,
    ) -> f64 {
        let mut score: f64 = 0.3;
        let title = title.to_lowercase();
        if contains_any(&title, &["urgent", "asap", "срочно"]) {
            score += 0.3;
        }
        if contains_any(&title, &["client", "клиент", "customer"]) {
            score += 0.2;
        }
        if contains_any(&title, &["tax", "legal", "aeat"]) {
            score += 0.2;
        }
        if participants_count > 2 {
            score += 0.1;
        }
        if has_project {
            score += 0.1;
        }
        if has_deadline {
            score += 0.1;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn calculate_readiness(
        has_agenda: bool,
        has_docs: bool,
        has_context: bool,
        has_checklist: bool,
        has_participants: bool,
    ) -> f64 {
        let mut score: f64 = 0.0;
        if has_agenda {
            score += 0.25;
        }
        if has_docs {
            score += 0.2;
        }
        if has_context {
            score += 0.2;
        }
        if has_checklist {
            score += 0.15;
        }
        if has_participants {
            score += 0.2;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn detect_risks(
        has_agenda: bool,
        has_docs: bool,
        has_participants: bool,
        has_project: bool,
        is_upcoming_soon: bool,
    ) -> Vec<String> {
        let mut risks = Vec::new();
        if !has_agenda {
            risks.push("No agenda prepared".into());
        }
        if !has_docs {
            risks.push("No documents attached".into());
        }
        if !has_participants {
            risks.push("No participants resolved".into());
        }
        if !has_project {
            risks.push("Not linked to a project".into());
        }
        if is_upcoming_soon && (!has_agenda || !has_docs) {
            risks.push("Event is soon but preparation incomplete".into());
        }
        risks
    }
}
```

### `backend/src/domains/calendar/meetings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings.rs`
- Size bytes / Размер в байтах: `489`
- Included characters / Включено символов: `489`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod notes;
mod outcomes;
mod recordings;
mod rows;
mod transcripts;

pub use errors::MeetingsError;
pub use models::{EventRecording, EventTranscript, MeetingNote, MeetingOutcome};
pub use notes::MeetingNoteStore;
pub use outcomes::MeetingOutcomeStore;
pub use recordings::EventRecordingStore;
pub use recordings::EventRecordingStore as EventRecordingPort;
pub use transcripts::EventTranscriptStore;
pub use transcripts::EventTranscriptStore as EventTranscriptPort;
```

### `backend/src/domains/calendar/meetings/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings/errors.rs`
- Size bytes / Размер в байтах: `304`
- Included characters / Включено символов: `304`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum MeetingsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/calendar/meetings/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings/models.rs`
- Size bytes / Размер в байтах: `1469`
- Included characters / Включено символов: `1469`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingNote {
    pub id: String,
    pub event_id: String,
    pub content: String,
    pub format: String,
    pub source: String,
    pub linked_note_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingOutcome {
    pub id: String,
    pub event_id: String,
    pub outcome_type: String,
    pub title: String,
    pub description: Option<String>,
    pub owner_person_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub source: String,
    pub confidence: f64,
    pub linked_entity_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecording {
    pub id: String,
    pub event_id: String,
    pub file_path: Option<String>,
    pub source: String,
    pub duration_seconds: Option<i32>,
    pub transcript_id: Option<String>,
    pub processing_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventTranscript {
    pub id: String,
    pub event_id: String,
    pub text: String,
    pub language: String,
    pub summary: Option<String>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

### `backend/src/domains/calendar/meetings/notes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings/notes.rs`
- Size bytes / Размер в байтах: `2289`
- Included characters / Включено символов: `2289`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::calendar::evidence::link_calendar_entity;

use super::rows::{MEETING_NOTE_COLUMNS, row_to_meeting_note};
use super::{MeetingNote, MeetingsError};

#[derive(Clone)]
pub struct MeetingNoteStore {
    pool: PgPool,
}

impl MeetingNoteStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<MeetingNote>, MeetingsError> {
        let query = format!(
            "SELECT {MEETING_NOTE_COLUMNS} FROM meeting_notes WHERE event_id=$1 ORDER BY created_at DESC"
        );
        let rows = sqlx::query(&query)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_meeting_note).collect()
    }

    pub async fn create(
        &self,
        event_id: &str,
        content: &str,
        format: Option<&str>,
        source: Option<&str>,
    ) -> Result<MeetingNote, MeetingsError> {
        self.create_with_observation(event_id, content, format, source, None)
            .await
    }

    pub async fn create_with_observation(
        &self,
        event_id: &str,
        content: &str,
        format: Option<&str>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<MeetingNote, MeetingsError> {
        let query = format!(
            "INSERT INTO meeting_notes (event_id, content, format, source) VALUES ($1,$2,$3,$4) RETURNING {MEETING_NOTE_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(content)
            .bind(format.unwrap_or("markdown"))
            .bind(source.unwrap_or("manual"))
            .fetch_one(&self.pool)
            .await?;
        let note = row_to_meeting_note(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity(
                &self.pool,
                observation_id,
                "meeting_note",
                note.id.clone(),
                None,
                serde_json::json!({
                    "event_id": event_id,
                    "format": note.format,
                }),
                None,
            )
            .await?;
        }
        Ok(note)
    }
}
```

### `backend/src/domains/calendar/meetings/outcomes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings/outcomes.rs`
- Size bytes / Размер в байтах: `5254`
- Included characters / Включено символов: `5254`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::evidence::link_calendar_entity_in_transaction;

use super::rows::{MEETING_OUTCOME_COLUMNS, row_to_meeting_outcome};
use super::{MeetingOutcome, MeetingsError};

#[derive(Clone)]
pub struct MeetingOutcomeStore {
    pool: PgPool,
}

impl MeetingOutcomeStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<MeetingOutcome>, MeetingsError> {
        let query = format!(
            "SELECT {MEETING_OUTCOME_COLUMNS} FROM meeting_outcomes WHERE event_id=$1 ORDER BY outcome_type, title"
        );
        let rows = sqlx::query(&query)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_meeting_outcome).collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        source: Option<&str>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        self.add_with_observation(
            event_id,
            outcome_type,
            title,
            description,
            owner_id,
            due_date,
            source,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_with_observation(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let outcome = Self::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            outcome_type,
            title,
            description,
            owner_id,
            due_date,
            source,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(outcome)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let query = format!(
            "INSERT INTO meeting_outcomes (event_id, outcome_type, title, description, owner_person_id, due_date, source) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING {MEETING_OUTCOME_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(outcome_type)
            .bind(title)
            .bind(description)
            .bind(owner_id)
            .bind(due_date)
            .bind(source.unwrap_or("manual"))
            .fetch_one(&mut **transaction)
            .await?;
        let outcome = row_to_meeting_outcome(row)?;

        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "meeting_outcome",
                outcome.id.clone(),
                None,
                serde_json::json!({
                    "event_id": event_id,
                    "outcome_type": outcome.outcome_type,
                    "linked_entity_id": outcome.linked_entity_id,
                }),
                None,
            )
            .await?;
        }

        Ok(outcome)
    }

    pub(crate) async fn set_linked_entity_id_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        outcome_id: &str,
        linked_entity_id: &str,
    ) -> Result<MeetingOutcome, MeetingsError> {
        let query = format!(
            "UPDATE meeting_outcomes SET linked_entity_id=$2, updated_at=now() WHERE id=$1::uuid RETURNING {MEETING_OUTCOME_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(outcome_id)
            .bind(linked_entity_id)
            .fetch_one(&mut **transaction)
            .await?;
        row_to_meeting_outcome(row)
    }

    pub async fn follow_up_status(&self, event_id: &str) -> Result<Value, MeetingsError> {
        let rows = sqlx::query(
            "SELECT outcome_type, COUNT(*) as cnt FROM meeting_outcomes WHERE event_id=$1 GROUP BY outcome_type",
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;
        let mut status = serde_json::Map::new();
        for row in &rows {
            let outcome_type: String = row.try_get("outcome_type")?;
            let count: i64 = row.try_get("cnt")?;
            status.insert(outcome_type, serde_json::Value::Number(count.into()));
        }
        Ok(Value::Object(status))
    }
}
```

### `backend/src/domains/calendar/meetings/recordings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings/recordings.rs`
- Size bytes / Размер в байтах: `6045`
- Included characters / Включено символов: `6045`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::evidence::{
    link_calendar_entity, link_calendar_entity_in_transaction,
};

use super::rows::{EVENT_RECORDING_COLUMNS, row_to_event_recording};
use super::{EventRecording, MeetingsError};

#[derive(Clone)]
pub struct EventRecordingStore {
    pool: PgPool,
}

impl EventRecordingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<EventRecording>, MeetingsError> {
        let query =
            format!("SELECT {EVENT_RECORDING_COLUMNS} FROM event_recordings WHERE event_id=$1");
        let rows = sqlx::query(&query)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_event_recording).collect()
    }

    pub async fn add(
        &self,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
    ) -> Result<EventRecording, MeetingsError> {
        self.add_with_observation(event_id, file_path, source, duration_seconds, None)
            .await
    }

    pub async fn add_with_observation(
        &self,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let recording = Self::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            file_path,
            source,
            duration_seconds,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(recording)
    }

    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let query = format!(
            "INSERT INTO event_recordings (event_id, file_path, source, duration_seconds) VALUES ($1,$2,$3,$4) RETURNING {EVENT_RECORDING_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(file_path)
            .bind(source.unwrap_or("manual"))
            .bind(duration_seconds)
            .fetch_one(&mut **transaction)
            .await?;
        let recording = row_to_event_recording(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "event_recording",
                recording.id.clone(),
                None,
                serde_json::json!({
                    "event_id": event_id,
                    "duration_seconds": recording.duration_seconds,
                }),
                None,
            )
            .await?;
        }
        Ok(recording)
    }

    pub(crate) async fn find_by_file_path_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        file_path: &str,
    ) -> Result<Option<EventRecording>, MeetingsError> {
        let query = format!(
            "SELECT {EVENT_RECORDING_COLUMNS} FROM event_recordings WHERE event_id=$1 AND file_path=$2 ORDER BY created_at DESC LIMIT 1"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(file_path)
            .fetch_optional(&mut **transaction)
            .await?;
        row.map(row_to_event_recording).transpose()
    }

    pub(crate) async fn attach_transcript_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        recording_id: &str,
        transcript_id: &str,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let query = format!(
            "UPDATE event_recordings SET transcript_id=$2, processing_status='transcribed', updated_at=now() WHERE id::text=$1 RETURNING {EVENT_RECORDING_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(recording_id)
            .bind(transcript_id)
            .fetch_one(&mut **transaction)
            .await?;
        let recording = row_to_event_recording(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "event_recording",
                recording.id.clone(),
                Some("transcript_attached"),
                serde_json::json!({
                    "event_id": recording.event_id,
                    "transcript_id": transcript_id,
                    "processing_status": recording.processing_status,
                }),
                None,
            )
            .await?;
        }
        Ok(recording)
    }

    pub async fn find_by_file_path(
        &self,
        event_id: &str,
        file_path: &str,
    ) -> Result<Option<EventRecording>, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let recording =
            Self::find_by_file_path_in_transaction(&mut transaction, event_id, file_path).await?;
        transaction.rollback().await?;
        Ok(recording)
    }

    pub async fn attach_transcript(
        &self,
        recording_id: &str,
        transcript_id: &str,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let recording = Self::attach_transcript_in_transaction(
            &mut transaction,
            recording_id,
            transcript_id,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(recording)
    }
}
```

### `backend/src/domains/calendar/meetings/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings/rows.rs`
- Size bytes / Размер в байтах: `2906`
- Included characters / Включено символов: `2906`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::{EventRecording, EventTranscript, MeetingNote, MeetingOutcome, MeetingsError};

pub(super) const MEETING_NOTE_COLUMNS: &str =
    "id::text, event_id, content, format, source, linked_note_id, created_at, updated_at";
pub(super) const MEETING_OUTCOME_COLUMNS: &str = "id::text, event_id, outcome_type, title, description, owner_person_id, due_date, source, confidence, linked_entity_id, created_at, updated_at";
pub(super) const EVENT_RECORDING_COLUMNS: &str = "id::text, event_id, file_path, source, duration_seconds, transcript_id::text, processing_status, created_at, updated_at";
pub(super) const EVENT_TRANSCRIPT_COLUMNS: &str =
    "id::text, event_id, text, language, summary, model, created_at";

pub(super) fn row_to_meeting_note(row: PgRow) -> Result<MeetingNote, MeetingsError> {
    Ok(MeetingNote {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        content: row.try_get("content")?,
        format: row.try_get("format")?,
        source: row.try_get("source")?,
        linked_note_id: row.try_get("linked_note_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_meeting_outcome(row: PgRow) -> Result<MeetingOutcome, MeetingsError> {
    Ok(MeetingOutcome {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        outcome_type: row.try_get("outcome_type")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        owner_person_id: row.try_get("owner_person_id")?,
        due_date: row.try_get("due_date")?,
        source: row.try_get("source")?,
        confidence: f64::from(row.try_get::<f32, _>("confidence")?),
        linked_entity_id: row.try_get("linked_entity_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_event_recording(row: PgRow) -> Result<EventRecording, MeetingsError> {
    Ok(EventRecording {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        file_path: row.try_get("file_path")?,
        source: row.try_get("source")?,
        duration_seconds: row.try_get("duration_seconds")?,
        transcript_id: row.try_get("transcript_id")?,
        processing_status: row.try_get("processing_status")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_event_transcript(row: PgRow) -> Result<EventTranscript, MeetingsError> {
    Ok(EventTranscript {
        id: row.try_get("id")?,
        event_id: row.try_get("event_id")?,
        text: row.try_get("text")?,
        language: row.try_get("language")?,
        summary: row.try_get("summary")?,
        model: row.try_get("model")?,
        created_at: row.try_get("created_at")?,
    })
}
```

### `backend/src/domains/calendar/meetings/transcripts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/meetings/transcripts.rs`
- Size bytes / Размер в байтах: `2976`
- Included characters / Включено символов: `2976`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::evidence::link_calendar_entity_in_transaction;

use super::rows::{EVENT_TRANSCRIPT_COLUMNS, row_to_event_transcript};
use super::{EventTranscript, MeetingsError};

#[derive(Clone)]
pub struct EventTranscriptStore {
    pool: PgPool,
}

impl EventTranscriptStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<EventTranscript>, MeetingsError> {
        let query = format!(
            "SELECT {EVENT_TRANSCRIPT_COLUMNS} FROM event_transcripts WHERE event_id=$1 ORDER BY created_at DESC LIMIT 1"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_event_transcript).transpose()
    }

    pub async fn add_with_observation(
        &self,
        event_id: &str,
        text: &str,
        language: Option<&str>,
        summary: Option<&str>,
        model: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<EventTranscript, MeetingsError> {
        let mut transaction = self.pool.begin().await?;
        let transcript = Self::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            text,
            language,
            summary,
            model,
            observation_id,
        )
        .await?;
        transaction.commit().await?;
        Ok(transcript)
    }

    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        text: &str,
        language: Option<&str>,
        summary: Option<&str>,
        model: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<EventTranscript, MeetingsError> {
        let query = format!(
            "INSERT INTO event_transcripts (event_id, text, language, summary, model) VALUES ($1,$2,$3,$4,$5) RETURNING {EVENT_TRANSCRIPT_COLUMNS}"
        );
        let row = sqlx::query(&query)
            .bind(event_id)
            .bind(text)
            .bind(language.unwrap_or("en"))
            .bind(summary)
            .bind(model)
            .fetch_one(&mut **transaction)
            .await?;
        let transcript = row_to_event_transcript(row)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                transaction,
                observation_id,
                "event_transcript",
                transcript.id.clone(),
                Some("transcript_projection"),
                serde_json::json!({
                    "event_id": event_id,
                    "language": transcript.language,
                    "model": transcript.model,
                }),
                None,
            )
            .await?;
        }
        Ok(transcript)
    }
}
```

### `backend/src/domains/calendar/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/mod.rs`
- Size bytes / Размер в байтах: `232`
- Included characters / Включено символов: `232`
- Truncated / Обрезано: `no`

```rust
pub mod brain;
mod command_service;
pub mod core;
pub mod events;
pub(crate) mod evidence;
pub mod health;
pub mod intelligence;
pub mod meetings;
pub mod reminders;
pub mod rules;
pub mod scheduling;
pub mod service;
pub mod sync;
```

### `backend/src/domains/calendar/reminders.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/reminders.rs`
- Size bytes / Размер в байтах: `6340`
- Included characters / Включено символов: `6340`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

use super::evidence::link_calendar_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarReminder {
    pub id: String,
    pub event_id: String,
    pub reminder_type: String,
    pub minutes_before: Option<i32>,
    pub condition_json: Option<Value>,
    pub message: Option<String>,
    pub source: String,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CalendarReminderStore {
    pool: PgPool,
}

impl CalendarReminderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<CalendarReminder>, ReminderError> {
        let rows = sqlx::query("SELECT id::text, event_id, reminder_type, minutes_before, condition_json, message, source, is_active, last_triggered_at, created_at, updated_at FROM calendar_reminders WHERE event_id=$1 ORDER BY minutes_before NULLS LAST")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(CalendarReminder {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    reminder_type: r.try_get("reminder_type")?,
                    minutes_before: r.try_get("minutes_before")?,
                    condition_json: r.try_get("condition_json")?,
                    message: r.try_get("message")?,
                    source: r.try_get("source")?,
                    is_active: r.try_get("is_active")?,
                    last_triggered_at: r.try_get("last_triggered_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        event_id: &str,
        reminder_type: &str,
        minutes_before: Option<i32>,
        message: Option<&str>,
    ) -> Result<CalendarReminder, ReminderError> {
        self.create_with_source(event_id, reminder_type, minutes_before, message, "manual")
            .await
    }

    pub async fn create_with_source(
        &self,
        event_id: &str,
        reminder_type: &str,
        minutes_before: Option<i32>,
        message: Option<&str>,
        source: &str,
    ) -> Result<CalendarReminder, ReminderError> {
        self.create_with_observation(
            event_id,
            reminder_type,
            minutes_before,
            message,
            source,
            None,
        )
        .await
    }

    pub async fn create_with_observation(
        &self,
        event_id: &str,
        reminder_type: &str,
        minutes_before: Option<i32>,
        message: Option<&str>,
        source: &str,
        observation_id: Option<&str>,
    ) -> Result<CalendarReminder, ReminderError> {
        let row = sqlx::query("INSERT INTO calendar_reminders (event_id, reminder_type, minutes_before, message, source) VALUES ($1,$2,$3,$4,$5) RETURNING id::text, event_id, reminder_type, minutes_before, condition_json, message, source, is_active, last_triggered_at, created_at, updated_at")
            .bind(event_id).bind(reminder_type).bind(minutes_before).bind(message).bind(source).fetch_one(&self.pool).await?;
        let reminder = CalendarReminder {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            reminder_type: row.try_get("reminder_type")?,
            minutes_before: row.try_get("minutes_before")?,
            condition_json: row.try_get("condition_json")?,
            message: row.try_get("message")?,
            source: row.try_get("source")?,
            is_active: row.try_get("is_active")?,
            last_triggered_at: row.try_get("last_triggered_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity(
                &self.pool,
                observation_id,
                "event_reminder",
                reminder.id.clone(),
                None,
                serde_json::json!({
                    "event_id": event_id,
                    "reminder_type": reminder.reminder_type,
                    "minutes_before": reminder.minutes_before,
                }),
                None,
            )
            .await?;
        }
        Ok(reminder)
    }

    pub async fn set_active(&self, id: &str, active: bool) -> Result<(), ReminderError> {
        self.set_active_with_source(id, active, "manual").await
    }

    pub async fn set_active_with_source(
        &self,
        id: &str,
        active: bool,
        source: &str,
    ) -> Result<(), ReminderError> {
        self.set_active_with_observation(id, active, source, None, None)
            .await
    }

    pub async fn set_active_with_observation(
        &self,
        id: &str,
        active: bool,
        source: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<(), ReminderError> {
        sqlx::query(
            "UPDATE calendar_reminders SET is_active=$2, source=$3, updated_at=now() WHERE id=$1::uuid",
        )
        .bind(id)
        .bind(active)
        .bind(source)
        .execute(&self.pool)
        .await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity(
                &self.pool,
                observation_id,
                "event_reminder",
                id.to_owned(),
                None,
                serde_json::json!({
                    "active": active,
                    "action": "toggle",
                }),
                metadata,
            )
            .await?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ReminderError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
```

### `backend/src/domains/calendar/rules.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/rules.rs`
- Size bytes / Размер в байтах: `8275`
- Included characters / Включено символов: `8275`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

use super::evidence::link_calendar_entity_in_transaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarRule {
    pub rule_id: String,
    pub name: String,
    pub natural_language_description: Option<String>,
    pub compiled_dsl: Value,
    pub enabled: bool,
    pub approval_mode: String,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CalendarRuleStore {
    pool: PgPool,
}

impl CalendarRuleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<CalendarRule>, CalendarRuleError> {
        let rows = sqlx::query("SELECT rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at FROM calendar_rules ORDER BY name")
            .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(CalendarRule {
                    rule_id: r.try_get("rule_id")?,
                    name: r.try_get("name")?,
                    natural_language_description: r.try_get("natural_language_description")?,
                    compiled_dsl: r.try_get("compiled_dsl")?,
                    enabled: r.try_get("enabled")?,
                    approval_mode: r.try_get("approval_mode")?,
                    last_run_at: r.try_get("last_run_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        dsl: Value,
        approval_mode: Option<&str>,
    ) -> Result<CalendarRule, CalendarRuleError> {
        self.create_with_observation(name, description, dsl, approval_mode, None, "create", None)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        name: &str,
        description: Option<&str>,
        dsl: Value,
        approval_mode: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CalendarRule, CalendarRuleError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let rule_id = format!("rule:v1:{ts:x}");
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO calendar_rules (rule_id, name, natural_language_description, compiled_dsl, approval_mode) VALUES ($1,$2,$3,$4,$5) RETURNING rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at")
            .bind(&rule_id).bind(name).bind(description).bind(&dsl).bind(approval_mode.unwrap_or("suggest_only")).fetch_one(&mut *transaction).await?;
        let rule = CalendarRule {
            rule_id: row.try_get("rule_id")?,
            name: row.try_get("name")?,
            natural_language_description: row.try_get("natural_language_description")?,
            compiled_dsl: row.try_get("compiled_dsl")?,
            enabled: row.try_get("enabled")?,
            approval_mode: row.try_get("approval_mode")?,
            last_run_at: row.try_get("last_run_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar_rule",
                rule.rule_id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "rule_id": rule.rule_id,
                    "approval_mode": rule.approval_mode,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(rule)
    }

    pub async fn update(
        &self,
        rule_id: &str,
        update: &RuleUpdate,
    ) -> Result<CalendarRule, CalendarRuleError> {
        self.update_with_observation(rule_id, update, None, "update", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        rule_id: &str,
        update: &RuleUpdate,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<CalendarRule, CalendarRuleError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("UPDATE calendar_rules SET name=COALESCE($2,name), natural_language_description=COALESCE($3,natural_language_description), compiled_dsl=COALESCE($4,compiled_dsl), enabled=COALESCE($5,enabled), approval_mode=COALESCE($6,approval_mode), updated_at=now() WHERE rule_id=$1 RETURNING rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at")
            .bind(rule_id).bind(update.name.as_deref()).bind(update.description.as_deref()).bind(update.dsl.as_ref()).bind(update.enabled).bind(update.approval_mode.as_deref()).fetch_one(&mut *transaction).await?;
        let rule = CalendarRule {
            rule_id: row.try_get("rule_id")?,
            name: row.try_get("name")?,
            natural_language_description: row.try_get("natural_language_description")?,
            compiled_dsl: row.try_get("compiled_dsl")?,
            enabled: row.try_get("enabled")?,
            approval_mode: row.try_get("approval_mode")?,
            last_run_at: row.try_get("last_run_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar_rule",
                rule.rule_id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "rule_id": rule.rule_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(rule)
    }

    pub async fn delete(&self, rule_id: &str) -> Result<(), CalendarRuleError> {
        self.delete_with_observation(rule_id, None, "delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        rule_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<(), CalendarRuleError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("DELETE FROM calendar_rules WHERE rule_id=$1")
            .bind(rule_id)
            .execute(&mut *transaction)
            .await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar_rule",
                rule_id.to_owned(),
                Some(relationship_kind),
                serde_json::json!({
                    "rule_id": rule_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RuleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub dsl: Option<Value>,
    pub enabled: Option<bool>,
    pub approval_mode: Option<String>,
}

#[derive(Debug, Error)]
pub enum CalendarRuleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/calendar/scheduling.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/calendar/scheduling.rs`
- Size bytes / Размер в байтах: `11776`
- Included characters / Включено символов: `11428`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;

use super::evidence::link_calendar_entity_in_transaction;

// ── DeadlineEvent ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadlineEvent {
    pub id: String,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<String>,
    pub title: String,
    pub due_at: DateTime<Utc>,
    pub severity: String,
    pub status: String,
    pub linked_calendar_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct DeadlineStore {
    pool: PgPool,
}

impl DeadlineStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<DeadlineEvent>, SchedulingError> {
        let limit = limit.clamp(1, 200);
        let rows = if let Some(s) = status {
            sqlx::query("SELECT id::text, source_entity_type, source_entity_id, title, due_at, severity, status, linked_calendar_event_id, created_at, updated_at FROM deadline_events WHERE status=$1 ORDER BY due_at ASC LIMIT $2")
                .bind(s).bind(limit).fetch_all(&self.pool).await?
        } else {
            sqlx::query("SELECT id::text, source_entity_type, source_entity_id, title, due_at, severity, status, linked_calendar_event_id, created_at, updated_at FROM deadline_events ORDER BY due_at ASC LIMIT $1")
                .bind(limit).fetch_all(&self.pool).await?
        };
        rows.into_iter()
            .map(|r| {
                Ok(DeadlineEvent {
                    id: r.try_get("id")?,
                    source_entity_type: r.try_get("source_entity_type")?,
                    source_entity_id: r.try_get("source_entity_id")?,
                    title: r.try_get("title")?,
                    due_at: r.try_get("due_at")?,
                    severity: r.try_get("severity")?,
                    status: r.try_get("status")?,
                    linked_calendar_event_id: r.try_get("linked_calendar_event_id")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        title: &str,
        due_at: DateTime<Utc>,
        severity: Option<&str>,
        entity_type: Option<&str>,
        entity_id: Option<&str>,
    ) -> Result<DeadlineEvent, SchedulingError> {
        self.create_with_observation(
            title,
            due_at,
            severity,
            entity_type,
            entity_id,
            None,
            "create",
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        title: &str,
        due_at: DateTime<Utc>,
        severity: Option<&str>,
        entity_type: Option<&str>,
        entity_id: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<DeadlineEvent, SchedulingError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO deadline_events (title, due_at, severity, source_entity_type, source_entity_id) VALUES ($1,$2,$3,$4,$5) RETURNING id::text, source_entity_type, source_entity_id, title, due_at, severity, status, linked_calendar_event_id, created_at, updated_at")
            .bind(title).bind(due_at).bind(severity.unwrap_or("medium")).bind(entity_type).bind(entity_id).fetch_one(&mut *transaction).await?;
        let deadline = DeadlineEvent {
            id: row.try_get("id")?,
            source_entity_type: row.try_get("source_entity_type")?,
            source_entity_id: row.try_get("source_entity_id")?,
            title: row.try_get("title")?,
            due_at: row.try_get("due_at")?,
            severity: row.try_get("severity")?,
            status: row.try_get("status")?,
            linked_calendar_event_id: row.try_get("linked_calendar_event_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "deadline_event",
                deadline.id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "deadline_id": deadline.id,
                    "source_entity_type": deadline.source_entity_type,
                    "source_entity_id": deadline.source_entity_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(deadline)
    }
}

// ── FocusBlock ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FocusBlock {
    pub id: String,
    pub title: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub purpose: Option<String>,
    pub linked_project_id: Option<String>,
    pub protection_level: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FocusBlockStore {
    pool: PgPool,
}

impl FocusBlockStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<FocusBlock>, SchedulingError> {
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query("SELECT id::text, title, start_at, end_at, purpose, linked_project_id, protection_level, status, created_at, updated_at FROM focus_blocks WHERE ($1::timestamptz IS NULL OR end_at>=$1) AND ($2::timestamptz IS NULL OR start_at<=$2) ORDER BY start_at ASC LIMIT $3")
            .bind(from).bind(to).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(FocusBlock {
                    id: r.try_get("id")?,
                    title: r.try_get("title")?,
                    start_at: r.try_get("start_at")?,
                    end_at: r.try_get("end_at")?,
                    purpose: r.try_get("purpose")?,
                    linked_project_id: r.try_get("linked_project_id")?,
                    protection_level: r.try_get("protection_level")?,
                    status: r.try_get("status")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn create(
        &self,
        title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        purpose: Option<&str>,
        project_id: Option<&str>,
        protection: Option<&str>,
    ) -> Result<FocusBlock, SchedulingError> {
        self.create_with_observation(
            title, start_at, end_at, purpose, project_id, protection, None, "create", None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        purpose: Option<&str>,
        project_id: Option<&str>,
        protection: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<Value>,
    ) -> Result<FocusBlock, SchedulingError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO focus_blocks (title, start_at, end_at, purpose, linked_project_id, protection_level) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id::text, title, start_at, end_at, purpose, linked_project_id, protection_level, status, created_at, updated_at")
            .bind(title).bind(start_at).bind(end_at).bind(purpose).bind(project_id).bind(protection.unwrap_or("medium")).fetch_one(&mut *transaction).await?;
        let focus_block = FocusBlock {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            start_at: row.try_get("start_at")?,
            end_at: row.try_get("end_at")?,
            purpose: row.try_get("purpose")?,
            linked_project_id: row.try_get("linked_project_id")?,
            protection_level: row.try_get("protection_level")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity_in_transaction(
                &mut transaction,
                observation_id,
                "focus_block",
                focus_block.id.clone(),
                Some(relationship_kind),
                serde_json::json!({
                    "focus_block_id": focus_block.id,
                    "linked_project_id": focus_block.linked_project_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(focus_block)
    }
}

// ── SmartSchedulingService ─────────────────────────────────────────────────

pub struct SmartSchedulingService;

impl SmartSchedulingService {
    /// Heuristic: find free slots by checking event gaps today+tomorrow
    pub fn find_slots(
        existing_events: &[(DateTime<Utc>, DateTime<Utc>)],
        duration_min: i64,
        lookahead_hours: i64,
    ) -> Vec<Slot> {
        let now = Utc::now();
        let end = now + Duration::hours(lookahead_hours);
        let mut slots = Vec::new();
        let mut sorted: Vec<_> = existing_events
            .iter()
            .filter(|(s, e)| *e > now && *s < end)
            .collect();
        sorted.sort_by_key(|(s, _)| *s);

        let mut cursor = now;
        for (s, e) in sorted {
            if *s > cursor {
                let gap = (*s - cursor).num_minutes();
                if gap >= duration_min {
                    slots.push(Slot {
                        start: cursor,
                        end: *s,
                        duration_minutes: gap,
                    });
                }
            }
            cursor = cursor.max(*e);
        }
        if end > cursor {
            let gap = (end - cursor).num_minutes();
            if gap >= duration_min {
                slots.push(Slot {
                    start: cursor,
                    end,
                    duration_minutes: gap,
                });
            }
        }
        slots
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_minutes: i64,
}

#[derive(Debug, Error)]
pub enum SchedulingError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
```

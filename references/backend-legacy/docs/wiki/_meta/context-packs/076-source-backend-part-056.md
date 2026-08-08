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

- Chunk ID / ID чанка: `076-source-backend-part-056`
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

### `backend/src/workflows/mail_background_sync/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store.rs`
- Size bytes / Размер в байтах: `427`
- Included characters / Включено символов: `427`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

mod account;
mod orphaned;
mod run_finish;
mod run_latest;
mod run_progress;
mod run_start;
mod scheduling;
mod settings;
mod statuses;

#[derive(Clone)]
pub struct MailSyncStore {
    pub(in crate::workflows::mail_background_sync::store) pool: PgPool,
}

pub(super) type MailSyncStatePort = MailSyncStore;

impl MailSyncStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### `backend/src/workflows/mail_background_sync/store/account.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/account.rs`
- Size bytes / Размер в байтах: `614`
- Included characters / Включено символов: `614`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::MailSyncError;
use super::MailSyncStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync::store) async fn require_account(
        &self,
        account_id: &str,
    ) -> Result<(), MailSyncError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM communication_provider_accounts WHERE account_id = $1)",
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;
        if exists {
            Ok(())
        } else {
            Err(MailSyncError::AccountNotFound)
        }
    }
}
```

### `backend/src/workflows/mail_background_sync/store/orphaned.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/orphaned.rs`
- Size bytes / Размер в байтах: `2310`
- Included characters / Включено символов: `2310`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::super::errors::MailSyncError;
use super::super::evidence::capture_mail_sync_run_observation;
use super::super::rows::row_to_run;
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn mark_orphaned_active_runs_failed(
        &self,
        now: DateTime<Utc>,
    ) -> Result<u64, MailSyncError> {
        let mut transaction = self.pool.begin().await?;
        let rows = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'failed',
                phase = 'failed',
                progress_mode = 'none',
                progress_percent = NULL,
                error_code = 'backend_restarted',
                error_message = 'Mail sync run was interrupted by backend restart',
                completed_at = $1,
                next_run_at = $1,
                updated_at = $1
            WHERE status IN ('queued', 'running', 'recoverable_full_resync_needed')
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(now)
        .fetch_all(&mut *transaction)
        .await?;
        let affected = rows.len() as u64;
        for row in rows {
            let run = row_to_run(row)?;
            capture_mail_sync_run_observation(
                &mut transaction,
                &run,
                "COMMUNICATION_MAIL_SYNC_RUN_STATUS",
                "orphaned_failed",
                now,
                "mail.background_sync.mark_orphaned_active_runs_failed",
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(affected)
    }
}
```

### `backend/src/workflows/mail_background_sync/store/run_finish.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/run_finish.rs`
- Size bytes / Размер в байтах: `3343`
- Included characters / Включено символов: `3343`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::super::errors::MailSyncError;
use super::super::events::sync_run_finished_event;
use super::super::evidence::capture_mail_sync_run_observation;
use super::super::models::{FinishRun, MailSyncRun};
use super::super::rows::row_to_run;
use super::MailSyncStore;
use crate::platform::events::EventStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync) async fn finish_run(
        &self,
        run_id: &str,
        finish: FinishRun,
    ) -> Result<MailSyncRun, MailSyncError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = $2,
                phase = $3,
                progress_mode = $4,
                progress_percent = $5,
                processed_messages = $6,
                estimated_total_messages = $7,
                fetched_messages = $8,
                projected_messages = $9,
                upserted_persons = $10,
                upserted_organizations = $11,
                checkpoint_after = $12,
                checkpoint_saved = $13,
                error_code = $14,
                error_message = $15,
                completed_at = now(),
                next_run_at = $16,
                updated_at = now()
            WHERE run_id = $1
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(run_id)
        .bind(finish.status.as_str())
        .bind(finish.phase.as_str())
        .bind(finish.progress_mode.as_str())
        .bind(finish.progress_percent)
        .bind(finish.processed_messages)
        .bind(finish.estimated_total_messages)
        .bind(finish.fetched_messages)
        .bind(finish.projected_messages)
        .bind(finish.upserted_persons)
        .bind(finish.upserted_organizations)
        .bind(finish.checkpoint_after.unwrap_or_else(|| json!({})))
        .bind(finish.checkpoint_saved)
        .bind(finish.error_code)
        .bind(finish.error_message)
        .bind(finish.next_run_at)
        .fetch_one(&mut *transaction)
        .await?;

        let run = row_to_run(row)?;
        capture_mail_sync_run_observation(
            &mut transaction,
            &run,
            "COMMUNICATION_MAIL_SYNC_RUN_STATUS",
            &run.status,
            run.completed_at.unwrap_or(run.started_at),
            "mail.background_sync.finish_run",
        )
        .await?;
        let event = sync_run_finished_event(&run)?;
        EventStore::append_in_transaction(&mut transaction, &event).await?;
        transaction.commit().await?;

        Ok(run)
    }
}
```

### `backend/src/workflows/mail_background_sync/store/run_latest.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/run_latest.rs`
- Size bytes / Размер в байтах: `1506`
- Included characters / Включено символов: `1506`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::MailSyncError;
use super::super::models::MailSyncRunResponse;
use super::super::rows::row_to_run;
use super::MailSyncStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync) async fn latest_run_response(
        &self,
        account_id: &str,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let row = sqlx::query(
            r#"
            SELECT
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            FROM communication_mail_sync_runs
            WHERE account_id = $1
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MailSyncError::RunNotFound);
        };

        row_to_run(row).map(Into::into)
    }
}
```

### `backend/src/workflows/mail_background_sync/store/run_progress.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/run_progress.rs`
- Size bytes / Размер в байтах: `4701`
- Included characters / Включено символов: `4701`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;

use super::super::errors::MailSyncError;
use super::super::events::sync_run_progress_event;
use super::super::evidence::capture_mail_sync_run_observation;
use super::super::models::ProgressUpdate;
use super::super::rows::row_to_run;
use super::MailSyncStore;
use crate::platform::events::EventStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync) async fn update_progress(
        &self,
        update: ProgressUpdate<'_>,
    ) -> Result<(), MailSyncError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'running',
                phase = $2,
                progress_mode = $3,
                progress_percent = $4,
                processed_messages = $5,
                estimated_total_messages = $6,
                current_batch_size = $7,
                updated_at = now()
            WHERE run_id = $1
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(update.run_id)
        .bind(update.phase.as_str())
        .bind(update.progress_mode.as_str())
        .bind(update.progress_percent)
        .bind(update.processed_messages)
        .bind(update.estimated_total_messages)
        .bind(update.current_batch_size)
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(row) = row {
            let run = row_to_run(row)?;
            capture_mail_sync_run_observation(
                &mut transaction,
                &run,
                "COMMUNICATION_MAIL_SYNC_RUN_STATUS",
                "progress",
                Utc::now(),
                "mail.background_sync.update_progress",
            )
            .await?;
            let event = sync_run_progress_event(&run)?;
            EventStore::append_in_transaction(&mut transaction, &event).await?;
        }
        transaction.commit().await?;

        Ok(())
    }

    pub(in crate::workflows::mail_background_sync) async fn mark_recoverable_full_resync(
        &self,
        run_id: &str,
        error_code: &'static str,
    ) -> Result<(), MailSyncError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'recoverable_full_resync_needed',
                phase = 'listing',
                progress_mode = 'indeterminate',
                error_code = $2,
                error_message = 'Gmail history expired; restarting full mailbox listing',
                updated_at = now()
            WHERE run_id = $1
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(run_id)
        .bind(error_code)
        .fetch_optional(&mut *transaction)
        .await?;
        if let Some(row) = row {
            let run = row_to_run(row)?;
            capture_mail_sync_run_observation(
                &mut transaction,
                &run,
                "COMMUNICATION_MAIL_SYNC_RUN_STATUS",
                "recoverable_full_resync_needed",
                Utc::now(),
                "mail.background_sync.mark_recoverable_full_resync",
            )
            .await?;
        }
        transaction.commit().await?;

        Ok(())
    }
}
```

### `backend/src/workflows/mail_background_sync/store/run_start.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/run_start.rs`
- Size bytes / Размер в байтах: `3158`
- Included characters / Включено символов: `3158`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::errors::MailSyncError;
use super::super::events::sync_run_started_event;
use super::super::evidence::capture_mail_sync_run_observation;
use super::super::models::{MailSyncRun, MailSyncSettings, MailSyncTrigger};
use super::super::rows::row_to_run;
use super::super::validation::{mail_sync_run_id, validate_account_id};
use super::MailSyncStore;
use crate::platform::events::EventStore;

impl MailSyncStore {
    pub(in crate::workflows::mail_background_sync) async fn start_run(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
        settings: &MailSyncSettings,
        checkpoint_before: Option<Value>,
    ) -> Result<MailSyncRun, MailSyncError> {
        validate_account_id(account_id)?;
        let run_id = mail_sync_run_id(account_id);
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            INSERT INTO communication_mail_sync_runs (
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                current_batch_size,
                checkpoint_before
            )
            VALUES ($1, $2, $3, 'running', 'listing', 'indeterminate', $4, $5)
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(&run_id)
        .bind(account_id.trim())
        .bind(trigger.as_str())
        .bind(settings.batch_size)
        .bind(checkpoint_before.unwrap_or_else(|| json!({})))
        .fetch_one(&mut *transaction)
        .await;

        match result {
            Ok(row) => {
                let run = row_to_run(row)?;
                capture_mail_sync_run_observation(
                    &mut transaction,
                    &run,
                    "COMMUNICATION_MAIL_SYNC_RUN",
                    "started",
                    run.started_at,
                    "mail.background_sync.start_run",
                )
                .await?;
                let event = sync_run_started_event(&run)?;
                EventStore::append_in_transaction(&mut transaction, &event).await?;
                transaction.commit().await?;
                Ok(run)
            }
            Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
                Err(MailSyncError::RunAlreadyActive)
            }
            Err(error) => Err(MailSyncError::Sqlx(error)),
        }
    }
}
```

### `backend/src/workflows/mail_background_sync/store/scheduling.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/scheduling.rs`
- Size bytes / Размер в байтах: `2305`
- Included characters / Включено символов: `2305`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::super::errors::MailSyncError;
use super::super::models::MailSyncDueAccount;
use super::super::rows::row_to_due_account;
use super::super::{DEFAULT_MAIL_SYNC_BATCH_SIZE, DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS};
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn due_accounts(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<MailSyncDueAccount>, MailSyncError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (account_id)
                    account_id,
                    status,
                    completed_at,
                    next_run_at
                FROM communication_mail_sync_runs
                ORDER BY account_id, started_at DESC
            )
            SELECT
                a.account_id,
                COALESCE(settings.batch_size, $2) AS batch_size,
                COALESCE(settings.poll_interval_seconds, $3) AS poll_interval_seconds
            FROM communication_provider_accounts a
            LEFT JOIN communication_account_sync_settings settings ON settings.account_id = a.account_id
            LEFT JOIN latest ON latest.account_id = a.account_id
            WHERE a.provider_kind IN ('gmail', 'icloud', 'imap')
              AND COALESCE(settings.sync_enabled, true)
              AND NOT EXISTS (
                  SELECT 1
                  FROM communication_mail_sync_runs active
                  WHERE active.account_id = a.account_id
                    AND active.status IN ('queued', 'running', 'recoverable_full_resync_needed')
              )
              AND (
                  COALESCE(
                      latest.next_run_at,
                      latest.completed_at + (COALESCE(settings.poll_interval_seconds, $3)::text || ' seconds')::interval,
                      $1
                  ) <= $1
              )
            ORDER BY latest.completed_at ASC NULLS FIRST, a.account_id ASC
            LIMIT $4
            "#,
        )
        .bind(now)
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .bind(DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_due_account).collect()
    }
}
```

### `backend/src/workflows/mail_background_sync/store/settings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/settings.rs`
- Size bytes / Размер в байтах: `2470`
- Included characters / Включено символов: `2470`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::MailSyncError;
use super::super::models::{MailSyncSettings, MailSyncSettingsUpdate};
use super::super::rows::row_to_settings;
use super::super::validation::{validate_account_id, validate_settings};
use super::super::{DEFAULT_MAIL_SYNC_BATCH_SIZE, DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS};
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn settings_for_account(
        &self,
        account_id: &str,
    ) -> Result<MailSyncSettings, MailSyncError> {
        validate_account_id(account_id)?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (account_id, batch_size, poll_interval_seconds)
            VALUES ($1, $2, $3)
            ON CONFLICT (account_id) DO UPDATE SET account_id = EXCLUDED.account_id
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .bind(DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS)
        .fetch_one(&self.pool)
        .await?;

        row_to_settings(row)
    }

    pub async fn update_settings(
        &self,
        account_id: &str,
        update: MailSyncSettingsUpdate,
    ) -> Result<MailSyncSettings, MailSyncError> {
        validate_account_id(account_id)?;
        validate_settings(update.batch_size, update.poll_interval_seconds)?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (
                account_id,
                sync_enabled,
                batch_size,
                poll_interval_seconds,
                updated_at
            )
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                sync_enabled = EXCLUDED.sync_enabled,
                batch_size = EXCLUDED.batch_size,
                poll_interval_seconds = EXCLUDED.poll_interval_seconds,
                updated_at = now()
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(update.sync_enabled)
        .bind(update.batch_size)
        .bind(update.poll_interval_seconds)
        .fetch_one(&self.pool)
        .await?;

        row_to_settings(row)
    }
}
```

### `backend/src/workflows/mail_background_sync/store/statuses.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/store/statuses.rs`
- Size bytes / Размер в байтах: `3016`
- Included characters / Включено символов: `3016`
- Truncated / Обрезано: `no`

```rust
use super::super::DEFAULT_MAIL_SYNC_BATCH_SIZE;
use super::super::errors::MailSyncError;
use super::super::models::MailSyncStatus;
use super::super::rows::row_to_status;
use super::MailSyncStore;

impl MailSyncStore {
    pub async fn sync_statuses(&self) -> Result<Vec<MailSyncStatus>, MailSyncError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (account_id)
                    account_id,
                    status,
                    phase,
                    progress_mode,
                    progress_percent,
                    processed_messages,
                    estimated_total_messages,
                    current_batch_size,
                    started_at,
                    completed_at,
                    next_run_at,
                    error_code,
                    error_message,
                    fetched_messages,
                    projected_messages,
                    upserted_persons,
                    upserted_organizations
                FROM communication_mail_sync_runs
                ORDER BY account_id, started_at DESC
            )
            SELECT
                a.account_id,
                COALESCE(latest.status, 'idle') AS status,
                COALESCE(latest.phase, 'idle') AS phase,
                COALESCE(latest.progress_mode, 'none') AS progress_mode,
                latest.progress_percent,
                COALESCE(latest.processed_messages, 0) AS processed_messages,
                latest.estimated_total_messages,
                COALESCE(latest.current_batch_size, COALESCE(settings.batch_size, $1)) AS current_batch_size,
                latest.started_at AS last_started_at,
                latest.completed_at AS last_completed_at,
                COALESCE(
                    latest.next_run_at,
                    CASE
                        WHEN COALESCE(settings.sync_enabled, true) THEN now()
                        ELSE NULL
                    END
                ) AS next_run_at,
                latest.error_code AS last_error_code,
                latest.error_message AS last_error_message,
                COALESCE(latest.fetched_messages, 0) AS last_fetched_messages,
                COALESCE(latest.projected_messages, 0) AS last_projected_messages,
                COALESCE(latest.upserted_persons, 0) AS last_upserted_persons,
                COALESCE(latest.upserted_organizations, 0) AS last_upserted_organizations
            FROM communication_provider_accounts a
            LEFT JOIN communication_account_sync_settings settings ON settings.account_id = a.account_id
            LEFT JOIN latest ON latest.account_id = a.account_id
            WHERE a.provider_kind IN ('gmail', 'icloud', 'imap')
            ORDER BY a.display_name ASC, a.account_id ASC
            "#,
        )
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_status).collect()
    }
}
```

### `backend/src/workflows/mail_background_sync/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mail_background_sync/validation.rs`
- Size bytes / Размер в байтах: `1855`
- Included characters / Включено символов: `1855`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, TimeDelta, Utc};

use crate::vault::{HostVault, HostVaultError, VaultMode};

use super::errors::MailSyncError;
use super::models::MailSyncSettings;
use super::{MAX_BATCH_SIZE, MAX_POLL_INTERVAL_SECONDS, MIN_POLL_INTERVAL_SECONDS};

pub(super) fn require_unlocked_vault(vault: &HostVault) -> Result<(), HostVaultError> {
    match vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(HostVaultError::Locked),
        VaultMode::Uninitialized => Err(HostVaultError::Uninitialized),
    }
}

pub(super) fn validate_account_id(account_id: &str) -> Result<(), MailSyncError> {
    if account_id.trim().is_empty() {
        return Err(MailSyncError::InvalidSetting {
            field: "account_id",
            message: "must not be empty",
        });
    }
    Ok(())
}

pub(super) fn validate_settings(
    batch_size: i32,
    poll_interval_seconds: i32,
) -> Result<(), MailSyncError> {
    if !(1..=MAX_BATCH_SIZE).contains(&batch_size) {
        return Err(MailSyncError::InvalidSetting {
            field: "batch_size",
            message: "must be between 1 and 500",
        });
    }
    if !(MIN_POLL_INTERVAL_SECONDS..=MAX_POLL_INTERVAL_SECONDS).contains(&poll_interval_seconds) {
        return Err(MailSyncError::InvalidSetting {
            field: "poll_interval_seconds",
            message: "must be between 60 and 86400",
        });
    }
    Ok(())
}

pub(super) fn next_run_at(settings: &MailSyncSettings) -> Option<DateTime<Utc>> {
    if settings.sync_enabled {
        Some(Utc::now() + TimeDelta::seconds(i64::from(settings.poll_interval_seconds)))
    } else {
        None
    }
}

pub(super) fn mail_sync_run_id(account_id: &str) -> String {
    format!(
        "mail-sync-run:v1:{}:{}",
        account_id.trim(),
        Utc::now().timestamp_micros()
    )
}
```

### `backend/src/workflows/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/mod.rs`
- Size bytes / Размер в байтах: `753`
- Included characters / Включено символов: `753`
- Truncated / Обрезано: `no`

```rust
pub mod consistency_review;
pub mod email_fixture_pipeline;
pub mod email_intelligence;
pub mod email_sync_pipeline;
pub mod graph_projection;
pub mod mail_background_sync;
pub mod person_derived_evidence;
pub mod project_link_review_effects;
pub mod realtime_conversation_memory_pipeline;
pub mod realtime_conversation_radar_projection;
pub mod realtime_conversation_transcript_execution;
pub mod realtime_conversation_transcript_projection;
pub mod review_inbox;
pub mod review_mirror;
pub mod review_promotion;
pub mod task_creation;
pub mod telegram_media_storage;
pub mod workflow_action_person_projection;
pub mod yandex_telemost_calendar_matching;
pub mod zoom_calendar_matching;
pub mod zoom_participant_identity;
pub mod zoom_signal_detection;
```

### `backend/src/workflows/person_derived_evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/person_derived_evidence.rs`
- Size bytes / Размер в байтах: `12941`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationReviewPort,
    ObligationReviewState, ObligationStoreError,
};
use crate::domains::persons::core::{
    PERSON_ROLE_ASSIGNED_EVENT_TYPE, PERSON_ROLE_REMOVED_EVENT_TYPE, person_role_knowledge_id,
};
use crate::domains::persons::enrichment::PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::persons::trust::PERSON_PROMISE_CREATED_EVENT_TYPE;
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewPort,
    RelationshipReviewState, RelationshipStoreError, relationship_id,
};
use crate::engines::trust::{TrustEngine, TrustEngineError};
use crate::platform::events::{EventStoreError, StoredEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationStoreError,
};
use crate::workflows::review_mirror::{ReviewMirrorError, ensure_relationship_review_item};

pub const PERSON_DERIVED_EVIDENCE_CONSUMER: &str = "person_derived_evidence";

#[derive(Debug, Error)]
pub enum PersonDerivedEvidenceWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },
}

pub async fn project_person_derived_evidence_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_person_derived_evidence_event_inner(&pool, &event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

async fn project_person_derived_evidence_event_inner(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    match event.event.event_type.as_str() {
        PERSON_ROLE_ASSIGNED_EVENT_TYPE => materialize_role_assigned(pool, event).await,
        PERSON_ROLE_REMOVED_EVENT_TYPE => materialize_role_removed(pool, event).await,
        PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE => materialize_trust_score(pool, event).await,
        PERSON_PROMISE_CREATED_EVENT_TYPE => materialize_promise(pool, event).await,
        _ => Ok(()),
    }
}

async fn materialize_role_assigned(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let person_id = required_string(&event.event.payload, "person_id")?;
    let role = required_string(&event.event.payload, "role")?;
    let assigned_by = optional_string(&event.event.payload, "assigned_by");
    let role_knowledge_id = optional_string(&event.event.payload, "role_knowledge_id")
        .map(str::to_owned)
        .unwrap_or_else(|| person_role_knowledge_id(role));

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSON_ROLE",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "person_id": person_id,
                    "role": role,
                    "assigned_by": assigned_by,
                    "action": "assign",
                }),
                format!("person://{person_id}/roles/{role_knowledge_id}"),
            )
            .provenance(json!({
                "captured_by": "person_derived_evidence.role_assigned",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: person_id.to_owned(),
        target_entity_kind: RelationshipEntityKind::Knowledge,
        target_entity_id: role_knowledge_id,
        relationship_type: "has_role".to_owned(),
        trust_score: 1.0,
        strength_score: 0.7,
        confidence: 1.0,
        review_state: RelationshipReviewState::UserConfirmed,
        valid_from: None,
        valid_to: None,
        metadata: json!({
            "compatibility_source": "person_roles",
            "role": role,
            "assigned_by": assigned_by,
        }),
    };
    let evidence = NewRelationshipEvidence::observation(observation.observation_id)
        .excerpt(role)
        .metadata(json!({
            "compatibility_source": "person_roles",
        }));

    let _ = RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    Ok(())
}

async fn materialize_role_removed(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let person_id = required_string(&event.event.payload, "person_id")?;
    let role = required_string(&event.event.payload, "role")?;
    let role_knowledge_id = optional_string(&event.event.payload, "role_knowledge_id")
        .map(str::to_owned)
        .unwrap_or_else(|| person_role_knowledge_id(role));
    let relationship_id = relationship_id(
        RelationshipEntityKind::Persona,
        person_id,
        "has_role",
        RelationshipEntityKind::Knowledge,
        &role_knowledge_id,
    );

    let _ = RelationshipReviewPort::new(pool.clone())
        .set_review_state(&relationship_id, RelationshipReviewState::UserRejected)
        .await?;
    Ok(())
}

async fn materialize_trust_score(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let person_id = required_string(&event.event.payload, "person_id")?;
    let trust_score = required_i16(&event.event.payload, "trust_score")?;
    let normalized_confidence = f64::from(trust_score.clamp(0, 100)) / 100.0;
    let source_observation_id = optional_string(&event.event.payload, "source_observation_id");
    let evidence_text = format!("trust_score={trust_score}");
    let source_reliability = TrustEngine::source_reliability_signal(
        &format!("person_enrichment:{person_id}:trust_score"),
        &evidence_text,
        normalized_confidence,
    )?;

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSON_TRUST_SIGNAL",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "person_id": person_id,
                    "trust_score": trust_score,
                    "source_observation_id": source_observation_id,
                    "action": "trust_score_enrichment",
                }),
                format!("person://{person_id}/trust-score"),
            )
            .confidence(normalized_confidence)
            .provenance(json!({
                "captured_by": "person_derived_evidence.trust_score",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let Some(owner_person_id) = owner_persona_id(pool, person_id).await? else {
        return Ok(());
    };

    let relationship_signal = TrustEngine::persona_compatibility_score_signal(trust_score);
    let relationship = NewRelationship::between_personas(
        owner_person_id.clone(),
        person_id.to_owned(),
        relationship_signal.relationship_type,
        relationship_signal.trust_score,
        relationship_signal.strength_score,
        relationship_signal.confidence,
        RelationshipReviewState::Suggested,
    )
    .metadata(json!({
        "compatibility_source": "persons.trust_score",
        "trust_score": trust_score,
    }));
    let evidence = NewRelationshipEvidence::observation(observation.observation_id.clone())
        .excerpt(evidence_text)
        .metadata(json!({
            "compatibility_source": "persons.trust_score",
            "source_observation_id": source_observation_id,
            "trust_source_reliability": {
                "signal_type": source_reliability.kind.as_str(),
                "affected_source": source_reliability.affected_source,
                "direction": source_reliability.direction.as_str(),
                "confidence": source_reliability.confidence,
            }
        }));
    let relationship = RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    let _ = ensure_relationship_review_item(
        pool,
        &relationship.relationship_id,
        &relationship.relationship_type,
        relationship.source_entity_kind.as_str(),
        &relationship.source_entity_id,
        relationship.target_entity_kind.as_str(),
        &relationship.target_entity_id,
        relationship.confidence,
        Some("trust_score enrichment suggests a persona trust relationship"),
        &observation.observation_id,
    )
    .await?;

    Ok(())
}

async fn materialize_promise(
    pool: &PgPool,
    event: &StoredEventEnvelope,
) -> Result<(), PersonDerivedEvidenceWorkflowError> {
    let promise_id = required_string(&event.event.payload, "promise_id")?;
    let person_id = required_string(&event.event.payload, "person_id")?;
    let description = required_string(&event.event.payload, "description")?;
    let due_at: Option<DateTime<Utc>> = serde_json::from_value(
        event
            .event
            .payload
            .get("due_at")
            .cloned()
            .unwrap_or(Value::Null),
    )?;

    let observation = ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PERSON_PROMISE",
                ObservationOriginKind::LocalRuntime,
                event.event.occurred_at,
                json!({
                    "promise_id": promise_id,
                    "person_id": person_id,
                    "description": description,
                    "due_at": &due_at,
                    "action": "create",
                }),
                format!("person://{person_id}/promises/{promise_id}"),
            )
            .provenance(json!({
                "captured_by": "person_derived_evidence.promise_created",
                "event_id": event.event.event_id,
            })),
        )
        .await?;

    let mut obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        person_id.to_owned(),
        description.to_owned(),
        1.0,
        ObligationReviewState::UserConfirmed,
    )
    .metadata(json!({
        "compatibility_source": "person_promises",
        "person_promise_id": promise_id,
    }));
    if let Some(due_at) = due_at {
        obligation = obligation.due_at(due_at);
    }
    let evidence = NewObligationEvidence::observation(observation.observation_id)
        .quote(description)
        .metadata(json!({
            "compatibility_source": "person_promises",
            "person_promise_id": promise_id,
        }));
    let _ = ObligationReviewPort::new(pool.clone())
        .upsert_with_evidence(&obligation, &[evidence])
        .await?;

    Ok(())
}

async fn owner_persona_id(
    pool: &PgPool,
    target_person_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT person_id
        FROM persons
        WHERE is_self = true
          AND person_id <> $1
        LIMIT 1
        "#,
    )
    .bind(target_person_id)
    .fetch_optional(pool)
    .await
}

fn required_string<'a>(
    payload: &'a Value,
    field:
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/workflows/project_link_review_effects.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/project_link_review_effects.rs`
- Size bytes / Размер в байтах: `11985`
- Included characters / Включено символов: `11985`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::decisions::{
    DecisionEntityKind, DecisionReviewPortError, DecisionReviewState, NewDecision,
    NewDecisionEvidence, NewDecisionImpactedEntity,
};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewError, ProjectLinkReviewState, ProjectLinkTargetKind,
};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewPort,
    RelationshipReviewPortError, RelationshipReviewState,
};
use crate::platform::events::{EventEnvelope, EventStoreError, StoredEventEnvelope};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationPortError,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, ensure_relationship_review_item,
    sync_relationship_review_state_in_transaction,
};

pub const PROJECT_LINK_REVIEW_EFFECTS_CONSUMER: &str = "project_link_review_effects";
pub const PROJECT_LINK_REVIEW_EVENT_TYPE: &str = "project.link_review_state_changed";

#[derive(Debug, Error)]
pub enum ProjectLinkReviewEffectsWorkflowError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationPortError),

    #[error(transparent)]
    Decision(#[from] DecisionReviewPortError),

    #[error(transparent)]
    Relationship(#[from] RelationshipReviewPortError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error(transparent)]
    ProjectLinkReview(#[from] ProjectLinkReviewError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },
}

pub async fn project_link_review_effect_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_link_review_effect(&pool, &event.event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn project_link_review_effect(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ProjectLinkReviewEffectsWorkflowError> {
    project_link_review_effect_inner(pool, event).await
}

async fn project_link_review_effect_inner(
    pool: &PgPool,
    event: &EventEnvelope,
) -> Result<(), ProjectLinkReviewEffectsWorkflowError> {
    if event.event_type != PROJECT_LINK_REVIEW_EVENT_TYPE {
        return Ok(());
    }

    let review = ProjectLinkReviewEffect::from_payload(&event.payload)?;
    let observation = capture_review_observation(pool, event, &review).await?;
    let relationship =
        materialize_relationship(pool, event, &review, &observation.observation_id).await?;
    sync_relationship_review_item(pool, &relationship, &observation.observation_id).await?;

    if review.review_state == ProjectLinkReviewState::UserConfirmed {
        let _ = materialize_decision(pool, event, &review, &observation.observation_id).await?;
    }

    Ok(())
}

async fn capture_review_observation(
    pool: &PgPool,
    event: &EventEnvelope,
    review: &ProjectLinkReviewEffect,
) -> Result<crate::platform::observations::Observation, ProjectLinkReviewEffectsWorkflowError> {
    Ok(ObservationPort::new(pool.clone())
        .capture(
            &NewObservation::new(
                "PROJECT_LINK_REVIEW",
                ObservationOriginKind::LocalRuntime,
                event.occurred_at,
                json!({
                    "project_id": review.project_id,
                    "target_kind": review.target_kind.as_str(),
                    "target_id": review.target_id,
                    "review_state": review.review_state.as_str(),
                }),
                format!(
                    "project://{}/link-review/{}/{}",
                    review.project_id,
                    review.target_kind.as_str(),
                    review.target_id
                ),
            )
            .confidence(review.confidence())
            .provenance(json!({
                "captured_by": "project_link_review_effects",
                "event_id": event.event_id,
            })),
        )
        .await?)
}

async fn materialize_relationship(
    pool: &PgPool,
    event: &EventEnvelope,
    review: &ProjectLinkReviewEffect,
    observation_id: &str,
) -> Result<crate::domains::relationships::Relationship, ProjectLinkReviewEffectsWorkflowError> {
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Project,
        source_entity_id: review.project_id.clone(),
        target_entity_kind: review.relationship_target_kind(),
        target_entity_id: review.target_id.clone(),
        relationship_type: review.relationship_type().to_owned(),
        trust_score: review.confidence(),
        strength_score: review.confidence(),
        confidence: review.confidence(),
        review_state: review.relationship_review_state(),
        valid_from: None,
        valid_to: None,
        metadata: json!({
            "compatibility_table": "project_link_reviews",
            "project_link_review_event_id": event.event_id,
            "project_id": review.project_id,
            "target_kind": review.target_kind.as_str(),
            "target_id": review.target_id,
        }),
    };
    let evidence = NewRelationshipEvidence::observation(observation_id.to_owned())
        .excerpt(review.evidence_text())
        .metadata(json!({
            "compatibility_table": "project_link_reviews",
            "event_id": event.event_id,
        }));

    Ok(RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?)
}

async fn sync_relationship_review_item(
    pool: &PgPool,
    relationship: &crate::domains::relationships::Relationship,
    observation_id: &str,
) -> Result<(), ProjectLinkReviewEffectsWorkflowError> {
    let _ = ensure_relationship_review_item(
        pool,
        &relationship.relationship_id,
        &relationship.relationship_type,
        relationship.source_entity_kind.as_str(),
        &relationship.source_entity_id,
        relationship.target_entity_kind.as_str(),
        &relationship.target_entity_id,
        relationship.confidence,
        None,
        observation_id,
    )
    .await?;

    let mut transaction = pool.begin().await?;
    sync_relationship_review_state_in_transaction(&mut transaction, relationship).await?;
    transaction.commit().await?;
    Ok(())
}

async fn materialize_decision(
    pool: &PgPool,
    event: &EventEnvelope,
    review: &ProjectLinkReviewEffect,
    observation_id: &str,
) -> Result<crate::domains::decisions::Decision, ProjectLinkReviewEffectsWorkflowError> {
    let decision = NewDecision::new(
        "Project link review confirmed",
        format!(
            "User confirmed a {} link candidate for this project.",
            review.target_kind.as_str()
        ),
        1.0,
        DecisionReviewState::UserConfirmed,
    )
    .decided_at(event.occurred_at)
    .metadata(json!({
        "project_link_review_event_id": event.event_id,
        "project_id": review.project_id,
        "target_kind": review.target_kind.as_str(),
        "target_id": review.target_id,
    }));
    let evidence = NewDecisionEvidence::observation(observation_id.to_owned())
        .quote(review.evidence_text())
        .metadata(json!({
            "compatibility_table": "project_link_reviews",
            "event_id": event.event_id,
        }));
    let impacted_entities = [
        NewDecisionImpactedEntity::new(DecisionEntityKind::Project, review.project_id.clone())
            .impact_type("project_link_review"),
        NewDecisionImpactedEntity::new(review.decision_target_kind(), review.target_id.clone())
            .impact_type("project_link_review"),
    ];

    Ok(
        crate::domains::decisions::DecisionReviewPort::new(pool.clone())
            .upsert_with_evidence(&decision, &[evidence], &impacted_entities)
            .await?,
    )
}

struct ProjectLinkReviewEffect {
    project_id: String,
    target_kind: ProjectLinkTargetKind,
    target_id: String,
    review_state: ProjectLinkReviewState,
}

impl ProjectLinkReviewEffect {
    fn from_payload(payload: &Value) -> Result<Self, ProjectLinkReviewEffectsWorkflowError> {
        Ok(Self {
            project_id: required_string(payload, "project_id")?.to_owned(),
            target_kind: ProjectLinkTargetKind::parse(required_string(payload, "target_kind")?)?,
            target_id: required_string(payload, "target_id")?.to_owned(),
            review_state: ProjectLinkReviewState::parse(required_string(payload, "review_state")?)?,
        })
    }

    fn relationship_target_kind(&self) -> RelationshipEntityKind {
        match self.target_kind {
            ProjectLinkTargetKind::Message => RelationshipEntityKind::Communication,
            ProjectLinkTargetKind::Document => RelationshipEntityKind::Document,
        }
    }

    fn decision_target_kind(&self) -> DecisionEntityKind {
        match self.target_kind {
            ProjectLinkTargetKind::Message => DecisionEntityKind::Communication,
            ProjectLinkTargetKind::Document => DecisionEntityKind::Document,
        }
    }

    fn relationship_type(&self) -> &'static str {
        match self.target_kind {
            ProjectLinkTargetKind::Message => "project_has_message",
            ProjectLinkTargetKind::Document => "project_has_document",
        }
    }

    fn relationship_review_state(&self) -> RelationshipReviewState {
        match self.review_state {
            ProjectLinkReviewState::Suggested => RelationshipReviewState::Suggested,
            ProjectLinkReviewState::UserConfirmed => RelationshipReviewState::UserConfirmed,
            ProjectLinkReviewState::UserRejected => RelationshipReviewState::UserRejected,
        }
    }

    fn confidence(&self) -> f64 {
        match self.review_state {
            ProjectLinkReviewState::Suggested => 0.65,
            ProjectLinkReviewState::UserConfirmed => 1.0,
            ProjectLinkReviewState::UserRejected => 0.0,
        }
    }

    fn evidence_text(&self) -> &'static str {
        match (self.target_kind, self.review_state) {
            (ProjectLinkTargetKind::Message, ProjectLinkReviewState::Suggested) => {
                "User reset message link review for project."
            }
            (ProjectLinkTargetKind::Document, ProjectLinkReviewState::Suggested) => {
                "User reset document link review for project."
            }
            (ProjectLinkTargetKind::Message, ProjectLinkReviewState::UserConfirmed) => {
                "User confirmed message link to project."
            }
            (ProjectLinkTargetKind::Document, ProjectLinkReviewState::UserConfirmed) => {
                "User confirmed document link to project."
            }
            (ProjectLinkTargetKind::Message, ProjectLinkReviewState::UserRejected) => {
                "User rejected message link review for project."
            }
            (ProjectLinkTargetKind::Document, ProjectLinkReviewState::UserRejected) => {
                "User rejected document link review for project."
            }
        }
    }
}

fn required_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, ProjectLinkReviewEffectsWorkflowError> {
    let raw =
        payload
            .get(field)
            .ok_or(ProjectLinkReviewEffectsWorkflowError::MissingPayloadField(
                field,
            ))?;
    raw.as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(
            || ProjectLinkReviewEffectsWorkflowError::InvalidPayloadField {
                field,
                value: raw.to_string(),
            },
        )
}
```

### `backend/src/workflows/realtime_conversation_memory_pipeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/realtime_conversation_memory_pipeline.rs`
- Size bytes / Размер в байтах: `1839`
- Included characters / Включено символов: `1839`
- Truncated / Обрезано: `no`

```rust
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::engines::call_intelligence::{CallIntelligenceEngine, CallIntelligencePipelinePlan};
use crate::platform::realtime_conversation::CallBundleManifest;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RealtimeConversationMemoryPipelinePlan {
    pub bundle_id: String,
    pub account_id: String,
    pub conference_id: Option<String>,
    pub provider_kind: String,
    pub stage: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub audio_path: String,
    pub call_intelligence: CallIntelligencePipelinePlan,
    pub follow_up_events: Vec<String>,
}

pub fn plan_memory_pipeline(
    manifest: &CallBundleManifest,
) -> RealtimeConversationMemoryPipelinePlan {
    let engine = CallIntelligenceEngine;
    RealtimeConversationMemoryPipelinePlan {
        bundle_id: manifest.bundle_id.clone(),
        account_id: manifest.account_id.clone(),
        conference_id: manifest.provider_conference_id.clone(),
        provider_kind: manifest.provider_kind.as_str().to_owned(),
        stage: "queued_after_local_recording".to_owned(),
        bundle_root: manifest.layout.root.clone(),
        manifest_path: Path::new(&manifest.layout.root)
            .join(&manifest.layout.manifest)
            .to_string_lossy()
            .into_owned(),
        audio_path: Path::new(&manifest.layout.root)
            .join(&manifest.layout.audio_mp3)
            .to_string_lossy()
            .into_owned(),
        call_intelligence: engine.plan_from_bundle(manifest),
        follow_up_events: vec![
            "realtime_conversation.transcript.requested".to_owned(),
            "realtime_conversation.knowledge.extracted".to_owned(),
            "realtime_conversation.radar_signals.detected".to_owned(),
        ],
    }
}
```

### `backend/src/workflows/realtime_conversation_radar_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/realtime_conversation_radar_projection.rs`
- Size bytes / Размер в байтах: `3929`
- Included characters / Включено символов: `3929`
- Truncated / Обрезано: `no`

```rust
use crate::platform::realtime_conversation::CallBundleManifest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RealtimeConversationRadarSignalCandidate {
    pub signal_kind: String,
    pub title: String,
    pub confidence: f32,
    pub evidence: Value,
    pub promotion_policy: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RealtimeConversationRadarProjectionContext {
    pub live_stream_watch_url: Option<String>,
    pub unknown_cohost_emails: Vec<String>,
    pub recording_session_id: Option<String>,
}

pub fn call_bundle_radar_candidates(
    manifest: &CallBundleManifest,
    projection_context: &RealtimeConversationRadarProjectionContext,
) -> Vec<RealtimeConversationRadarSignalCandidate> {
    let mut candidates = Vec::new();
    let base_evidence = serde_json::json!({
        "bundle_id": manifest.bundle_id,
        "provider_kind": manifest.provider_kind.as_str(),
        "provider_conference_id": manifest.provider_conference_id,
        "join_url": manifest.join_url,
        "calendar_event_id": manifest.calendar_event_id,
        "project_id": manifest.project_id,
        "organization_id": manifest.organization_id,
    });

    if manifest.join_url.is_some()
        && manifest.calendar_event_id.is_none()
        && manifest.project_id.is_none()
        && manifest.organization_id.is_none()
    {
        candidates.push(RealtimeConversationRadarSignalCandidate {
            signal_kind: "unmatched_meeting_link".to_owned(),
            title: "Review unmatched Telemost meeting link".to_owned(),
            confidence: 0.72,
            evidence: serde_json::json!({
                "bundle": base_evidence,
                "reason": "provider_conference_has_no_calendar_project_or_organization_binding",
            }),
            promotion_policy: "radar_review_required_before_calendar_or_project_link".to_owned(),
        });
    }

    if let Some(watch_url) = projection_context.live_stream_watch_url.as_deref() {
        candidates.push(RealtimeConversationRadarSignalCandidate {
            signal_kind: "live_stream_reference".to_owned(),
            title: "Review Telemost live stream reference".to_owned(),
            confidence: 0.78,
            evidence: serde_json::json!({
                "bundle": base_evidence,
                "watch_url": watch_url,
            }),
            promotion_policy: "radar_review_required_before_live_stream_promotion".to_owned(),
        });
    }

    if !projection_context.unknown_cohost_emails.is_empty() {
        candidates.push(RealtimeConversationRadarSignalCandidate {
            signal_kind: "unknown_cohosts".to_owned(),
            title: "Review Telemost cohosts without confirmed persona mapping".to_owned(),
            confidence: 0.68,
            evidence: serde_json::json!({
                "bundle": base_evidence,
                "unknown_cohost_emails": projection_context.unknown_cohost_emails,
            }),
            promotion_policy: "radar_review_required_before_relationship_or_persona_link"
                .to_owned(),
        });
    }

    candidates.push(RealtimeConversationRadarSignalCandidate {
        signal_kind: "local_recording_artifact".to_owned(),
        title: "Review local Telemost recording artifact".to_owned(),
        confidence: 0.88,
        evidence: serde_json::json!({
            "bundle": base_evidence,
            "recording_session_id": projection_context.recording_session_id,
            "artifacts": {
                "audio": manifest.layout.audio_mp3,
                "speaker_hints": manifest.layout.speaker_hints_jsonl,
                "event_track": manifest.layout.event_track_jsonl,
            }
        }),
        promotion_policy: "radar_review_required_before_document_or_memory_promotion".to_owned(),
    });

    candidates
}
```

### `backend/src/workflows/realtime_conversation_transcript_execution.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/realtime_conversation_transcript_execution.rs`
- Size bytes / Размер в байтах: `11969`
- Included characters / Включено символов: `11969`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;
use tokio::task::spawn_blocking;

use crate::application::{
    complete_realtime_conversation_transcript_bridge,
    provider_runtime_contracts::{YandexTelemostError, YandexTelemostTranscriptBridgeRequest},
};
use crate::platform::events::{EventBus, EventStoreError, StoredEventEnvelope};
use crate::platform::realtime_conversation::{
    CallBundleManifest, REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED,
};

pub const REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER: &str =
    "realtime_conversation_transcript_execution";
const TRANSCRIBER_PATH_ENV: &str = "MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER";
const TRANSCRIBER_ARGS_JSON_ENV: &str = "MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_ARGS_JSON";
const TRANSCRIBER_TIMEOUT_SECONDS_ENV: &str =
    "MAKOSH_REALTIME_CONVERSATION_TRANSCRIBER_TIMEOUT_SECONDS";
const DEFAULT_TRANSCRIBER_TIMEOUT_SECONDS: u64 = 900;

#[derive(Debug, Error)]
pub enum RealtimeConversationTranscriptExecutionError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    TranscriptBridge(#[from] YandexTelemostError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },

    #[error("{0} is not configured")]
    MissingConfiguration(&'static str),

    #[error("{0} must be a JSON string array or integer seconds")]
    InvalidConfiguration(&'static str),

    #[error("transcript execution only supports provider `{expected}`, got `{actual}`")]
    UnsupportedProvider {
        expected: &'static str,
        actual: String,
    },

    #[error("transcriber command timed out after {0} seconds")]
    CommandTimeout(u64),

    #[error("transcriber command failed with status {status}: {stderr}")]
    CommandFailed { status: i32, stderr: String },
}

pub async fn execute_realtime_conversation_transcript_request_event(
    pool: PgPool,
    event_bus: EventBus,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    execute_realtime_conversation_transcript_request_event_inner(&pool, &event_bus, event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub fn realtime_conversation_transcriber_is_configured() -> bool {
    std::env::var(TRANSCRIBER_PATH_ENV)
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

async fn execute_realtime_conversation_transcript_request_event_inner(
    pool: &PgPool,
    event_bus: &EventBus,
    event: StoredEventEnvelope,
) -> Result<(), RealtimeConversationTranscriptExecutionError> {
    if event.event.event_type != REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED {
        return Ok(());
    }

    let payload = TranscriptExecutionPayload::from_payload(&event.event.payload)?;
    if payload.provider_kind != "yandex_telemost" {
        return Err(
            RealtimeConversationTranscriptExecutionError::UnsupportedProvider {
                expected: "yandex_telemost",
                actual: payload.provider_kind,
            },
        );
    }

    let manifest: CallBundleManifest =
        serde_json::from_str(&fs::read_to_string(&payload.manifest_path)?)?;
    let output = run_transcriber_command(&payload, &manifest).await?;
    let request = YandexTelemostTranscriptBridgeRequest {
        account_id: payload.account_id,
        conference_id: payload.conference_id,
        bundle_id: payload.bundle_id,
        bundle_root: payload.bundle_root,
        transcript_text: output.transcript_text,
        segments: output.segments,
        language_code: output.language_code,
        stt_provider: output.stt_provider,
        summary: output.summary,
        confidence: output.confidence,
        metadata: output.metadata,
    };
    let event_store = crate::platform::events::EventStore::new(pool.clone());
    let _ =
        complete_realtime_conversation_transcript_bridge(&event_store, Some(event_bus), &request)
            .await?;
    Ok(())
}

#[derive(Clone, Debug)]
struct TranscriptExecutionPayload {
    bundle_id: String,
    account_id: String,
    conference_id: Option<String>,
    provider_kind: String,
    bundle_root: String,
    manifest_path: String,
    audio_path: String,
}

impl TranscriptExecutionPayload {
    fn from_payload(payload: &Value) -> Result<Self, RealtimeConversationTranscriptExecutionError> {
        Ok(Self {
            bundle_id: required_string(payload, "bundle_id")?,
            account_id: required_string(payload, "account_id")?,
            conference_id: optional_string(payload, "conference_id"),
            provider_kind: required_string(payload, "provider_kind")?,
            bundle_root: required_absolute_path(payload, "bundle_root")?,
            manifest_path: required_absolute_path(payload, "manifest_path")?,
            audio_path: required_absolute_path(payload, "audio_path")?,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct LocalTranscriptCommandOutput {
    transcript_text: String,
    #[serde(default = "default_json_array")]
    segments: Value,
    #[serde(default)]
    language_code: Option<String>,
    stt_provider: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default = "default_json_object")]
    metadata: Value,
}

#[derive(Clone, Debug)]
struct LocalTranscriberConfig {
    executable: String,
    args: Vec<String>,
    timeout_seconds: u64,
}

impl LocalTranscriberConfig {
    fn from_env() -> Result<Self, RealtimeConversationTranscriptExecutionError> {
        let executable = std::env::var(TRANSCRIBER_PATH_ENV).map_err(|_| {
            RealtimeConversationTranscriptExecutionError::MissingConfiguration(TRANSCRIBER_PATH_ENV)
        })?;
        let executable = executable.trim().to_owned();
        if executable.is_empty() {
            return Err(
                RealtimeConversationTranscriptExecutionError::MissingConfiguration(
                    TRANSCRIBER_PATH_ENV,
                ),
            );
        }

        let args = match std::env::var(TRANSCRIBER_ARGS_JSON_ENV) {
            Ok(value) if !value.trim().is_empty() => {
                let parsed: Value = serde_json::from_str(&value)?;
                let Some(items) = parsed.as_array() else {
                    return Err(
                        RealtimeConversationTranscriptExecutionError::InvalidConfiguration(
                            TRANSCRIBER_ARGS_JSON_ENV,
                        ),
                    );
                };
                let mut args = Vec::with_capacity(items.len());
                for item in items {
                    let Some(item) = item.as_str() else {
                        return Err(
                            RealtimeConversationTranscriptExecutionError::InvalidConfiguration(
                                TRANSCRIBER_ARGS_JSON_ENV,
                            ),
                        );
                    };
                    args.push(item.to_owned());
                }
                args
            }
            _ => Vec::new(),
        };

        let timeout_seconds = match std::env::var(TRANSCRIBER_TIMEOUT_SECONDS_ENV) {
            Ok(value) if !value.trim().is_empty() => value.trim().parse().map_err(|_| {
                RealtimeConversationTranscriptExecutionError::InvalidConfiguration(
                    TRANSCRIBER_TIMEOUT_SECONDS_ENV,
                )
            })?,
            _ => DEFAULT_TRANSCRIBER_TIMEOUT_SECONDS,
        };

        Ok(Self {
            executable,
            args,
            timeout_seconds,
        })
    }
}

async fn run_transcriber_command(
    payload: &TranscriptExecutionPayload,
    manifest: &CallBundleManifest,
) -> Result<LocalTranscriptCommandOutput, RealtimeConversationTranscriptExecutionError> {
    let config = LocalTranscriberConfig::from_env()?;
    let payload = payload.clone();
    let manifest_json = serde_json::to_string(manifest)?;
    let timeout_seconds = config.timeout_seconds;
    let output = tokio::time::timeout(
        Duration::from_secs(timeout_seconds),
        spawn_blocking(move || {
            let mut command = Command::new(&config.executable);
            command.args(&config.args);
            command.env("MAKOSH_TRANSCRIPT_BUNDLE_ID", &payload.bundle_id);
            command.env("MAKOSH_TRANSCRIPT_ACCOUNT_ID", &payload.account_id);
            command.env(
                "MAKOSH_TRANSCRIPT_CONFERENCE_ID",
                payload.conference_id.as_deref().unwrap_or(""),
            );
            command.env("MAKOSH_TRANSCRIPT_PROVIDER_KIND", &payload.provider_kind);
            command.env("MAKOSH_TRANSCRIPT_BUNDLE_ROOT", &payload.bundle_root);
            command.env("MAKOSH_TRANSCRIPT_MANIFEST_PATH", &payload.manifest_path);
            command.env("MAKOSH_TRANSCRIPT_AUDIO_PATH", &payload.audio_path);
            command.env("MAKOSH_TRANSCRIPT_MANIFEST_JSON", manifest_json);
            command.output()
        }),
    )
    .await
    .map_err(|_| RealtimeConversationTranscriptExecutionError::CommandTimeout(timeout_seconds))??;
    let output = output?;

    if !output.status.success() {
        return Err(
            RealtimeConversationTranscriptExecutionError::CommandFailed {
                status: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            },
        );
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

fn required_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, RealtimeConversationTranscriptExecutionError> {
    let value = payload
        .get(field)
        .ok_or(RealtimeConversationTranscriptExecutionError::MissingPayloadField(field))?;
    let value = value.as_str().ok_or_else(|| {
        RealtimeConversationTranscriptExecutionError::InvalidPayloadField {
            field,
            value: value.to_string(),
        }
    })?;
    let value = value.trim();
    if value.is_empty() {
        return Err(
            RealtimeConversationTranscriptExecutionError::InvalidPayloadField {
                field,
                value: String::new(),
            },
        );
    }
    Ok(value.to_owned())
}

fn optional_string(payload: &Value, field: &'static str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn required_absolute_path(
    payload: &Value,
    field: &'static str,
) -> Result<String, RealtimeConversationTranscriptExecutionError> {
    let value = required_string(payload, field)?;
    if !Path::new(&value).is_absolute() {
        return Err(
            RealtimeConversationTranscriptExecutionError::InvalidPayloadField { field, value },
        );
    }
    Ok(value)
}

fn default_json_array() -> Value {
    json!([])
}

fn default_json_object() -> Value {
    json!({})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_execution_payload_requires_absolute_paths() {
        let error = TranscriptExecutionPayload::from_payload(&json!({
            "bundle_id": "bundle-1",
            "account_id": "telemost-main",
            "provider_kind": "yandex_telemost",
            "bundle_root": "relative/root",
            "manifest_path": "/tmp/manifest.json",
            "audio_path": "/tmp/audio.mp3"
        }))
        .expect_err("relative bundle_root must fail");

        assert!(error.to_string().contains("bundle_root"));
    }
}
```

### `backend/src/workflows/realtime_conversation_transcript_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/realtime_conversation_transcript_projection.rs`
- Size bytes / Размер в байтах: `11212`
- Included characters / Включено символов: `11212`
- Truncated / Обрезано: `no`

```rust
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::meetings::{EventRecordingPort, EventTranscriptPort};
use crate::domains::documents::core::{DocumentImportPort, NewDocumentImport};
use crate::platform::events::{EventStoreError, StoredEventEnvelope};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationPort};
use crate::platform::realtime_conversation::{
    CallBundleManifest, REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED,
};

pub const REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER: &str =
    "realtime_conversation_transcript_projection";

#[derive(Debug, Error)]
pub enum RealtimeConversationTranscriptProjectionError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    DocumentImport(#[from] crate::domains::documents::core::DocumentImportError),

    #[error(transparent)]
    Observation(#[from] crate::platform::observations::ObservationStoreError),

    #[error(transparent)]
    Meetings(#[from] crate::domains::calendar::meetings::MeetingsError),

    #[error("event payload is missing required field {0}")]
    MissingPayloadField(&'static str),

    #[error("event payload field {field} is invalid: {value}")]
    InvalidPayloadField { field: &'static str, value: String },
}

pub async fn project_realtime_conversation_transcript_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_realtime_conversation_transcript_event_inner(&pool, event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

async fn project_realtime_conversation_transcript_event_inner(
    pool: &PgPool,
    event: StoredEventEnvelope,
) -> Result<(), RealtimeConversationTranscriptProjectionError> {
    if event.event.event_type != REALTIME_CONVERSATION_TRANSCRIPT_COMPLETED {
        return Ok(());
    }

    let projection = TranscriptProjectionPayload::from_payload(&event.event.payload)?;
    let transcript_markdown = fs::read_to_string(&projection.transcript_markdown_path)?;
    let transcript_json: Value =
        serde_json::from_str(&fs::read_to_string(&projection.transcript_json_path)?)?;
    let manifest: CallBundleManifest =
        serde_json::from_str(&fs::read_to_string(&projection.manifest_path)?)?;

    let mut transaction = pool.begin().await?;
    let observation = ObservationPort::capture_in_transaction(
        &mut transaction,
        &NewObservation::new(
            "MEETING_TRANSCRIPT",
            ObservationOriginKind::LocalRuntime,
            event.event.occurred_at,
            json!({
                "bundle_id": projection.bundle_id,
                "account_id": projection.account_id,
                "conference_id": projection.conference_id,
                "calendar_event_id": manifest.calendar_event_id,
                "provider_kind": manifest.provider_kind.as_str(),
                "language_code": projection.language_code,
                "stt_provider": projection.stt_provider,
                "confidence": projection.confidence,
                "summary": projection.summary,
                "segment_count": projection.segment_count,
                "transcript": transcript_json,
            }),
            format!("call-bundle://{}/transcript", projection.bundle_id),
        )
        .confidence(projection.confidence)
        .provenance(json!({
            "captured_by": "realtime_conversation_transcript_projection",
            "event_id": event.event.event_id,
            "transcript_json_path": projection.transcript_json_path,
            "transcript_markdown_path": projection.transcript_markdown_path,
        })),
    )
    .await?;

    let document_id = transcript_document_id(&projection.bundle_id);
    let document = NewDocumentImport::markdown(
        &document_id,
        transcript_document_title(&manifest, &projection),
        transcript_markdown,
    );
    let _ = DocumentImportPort::import_document_manual_with_observation_in_transaction(
        &mut transaction,
        &document,
        format!("call-bundle://{}/transcript-document", projection.bundle_id),
        json!({
            "captured_by": "realtime_conversation_transcript_projection",
            "event_id": event.event.event_id,
            "bundle_id": projection.bundle_id,
        }),
        Some(&observation.observation_id),
        Some("transcript_projection"),
        Some(json!({
            "bundle_id": projection.bundle_id,
            "document_role": "meeting_transcript",
        })),
    )
    .await?;

    if let Some(calendar_event_id) = manifest
        .calendar_event_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let audio_path = bundle_audio_path(&manifest);
        let recording = match EventRecordingPort::find_by_file_path_in_transaction(
            &mut transaction,
            calendar_event_id,
            &audio_path,
        )
        .await?
        {
            Some(recording) => recording,
            None => {
                EventRecordingPort::add_with_observation_in_transaction(
                    &mut transaction,
                    calendar_event_id,
                    Some(&audio_path),
                    Some("yandex_telemost_local_recording"),
                    None,
                    Some(&observation.observation_id),
                )
                .await?
            }
        };
        let event_transcript = EventTranscriptPort::add_with_observation_in_transaction(
            &mut transaction,
            calendar_event_id,
            &projection.transcript_text,
            projection.language_code.as_deref(),
            projection.summary.as_deref(),
            Some(&projection.stt_provider),
            Some(&observation.observation_id),
        )
        .await?;
        let _ = EventRecordingPort::attach_transcript_in_transaction(
            &mut transaction,
            &recording.id,
            &event_transcript.id,
            Some(&observation.observation_id),
        )
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

#[derive(Clone, Debug)]
struct TranscriptProjectionPayload {
    bundle_id: String,
    account_id: String,
    conference_id: Option<String>,
    manifest_path: String,
    transcript_json_path: String,
    transcript_markdown_path: String,
    transcript_text: String,
    language_code: Option<String>,
    stt_provider: String,
    confidence: f64,
    summary: Option<String>,
    segment_count: usize,
}

impl TranscriptProjectionPayload {
    fn from_payload(
        payload: &Value,
    ) -> Result<Self, RealtimeConversationTranscriptProjectionError> {
        Ok(Self {
            bundle_id: required_string(payload, "bundle_id")?,
            account_id: required_string(payload, "account_id")?,
            conference_id: optional_string(payload, "conference_id"),
            manifest_path: required_string(payload, "manifest_path")?,
            transcript_json_path: required_string(payload, "transcript_json_path")?,
            transcript_markdown_path: required_string(payload, "transcript_markdown_path")?,
            transcript_text: required_string(payload, "transcript_text")?,
            language_code: optional_string(payload, "language_code"),
            stt_provider: required_string(payload, "stt_provider")?,
            confidence: optional_f64(payload, "confidence").unwrap_or(0.82),
            summary: optional_string(payload, "summary"),
            segment_count: payload
                .get("segment_count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
        })
    }
}

fn transcript_document_id(bundle_id: &str) -> String {
    format!("realtime-conversation-transcript:{bundle_id}")
}

fn transcript_document_title(
    manifest: &CallBundleManifest,
    projection: &TranscriptProjectionPayload,
) -> String {
    if let Some(conference_id) = projection
        .conference_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("Telemost transcript {conference_id}");
    }
    if let Some(provider_conference_id) = manifest
        .provider_conference_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("Telemost transcript {provider_conference_id}");
    }
    format!("Telemost transcript {}", projection.bundle_id)
}

fn bundle_audio_path(manifest: &CallBundleManifest) -> String {
    Path::new(&manifest.layout.root)
        .join(&manifest.layout.audio_mp3)
        .to_string_lossy()
        .into_owned()
}

fn required_string(
    payload: &Value,
    field: &'static str,
) -> Result<String, RealtimeConversationTranscriptProjectionError> {
    let value = payload
        .get(field)
        .ok_or(RealtimeConversationTranscriptProjectionError::MissingPayloadField(field))?;
    let value = value.as_str().ok_or_else(|| {
        RealtimeConversationTranscriptProjectionError::InvalidPayloadField {
            field,
            value: value.to_string(),
        }
    })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(
            RealtimeConversationTranscriptProjectionError::InvalidPayloadField {
                field,
                value: value.to_owned(),
            },
        );
    }
    if !Path::new(trimmed).is_absolute()
        && matches!(
            field,
            "manifest_path" | "transcript_json_path" | "transcript_markdown_path"
        )
    {
        return Err(
            RealtimeConversationTranscriptProjectionError::InvalidPayloadField {
                field,
                value: value.to_owned(),
            },
        );
    }
    Ok(trimmed.to_owned())
}

fn optional_string(payload: &Value, field: &'static str) -> Option<String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn optional_f64(payload: &Value, field: &'static str) -> Option<f64> {
    payload.get(field).and_then(Value::as_f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_document_id_is_stable() {
        assert_eq!(
            transcript_document_id("bundle-1"),
            "realtime-conversation-transcript:bundle-1"
        );
    }

    #[test]
    fn transcript_projection_payload_requires_absolute_paths() {
        let payload = json!({
            "bundle_id": "bundle-1",
            "account_id": "telemost-main",
            "manifest_path": "manifest.json",
            "transcript_json_path": "/tmp/transcript.json",
            "transcript_markdown_path": "/tmp/transcript.md",
            "stt_provider": "whisper-local"
        });

        let error = TranscriptProjectionPayload::from_payload(&payload).expect_err("invalid path");

        assert!(error.to_string().contains("manifest_path"));
    }
}
```

### `backend/src/workflows/review_inbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/review_inbox.rs`
- Size bytes / Размер в байтах: `20065`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::decisions::DecisionReviewPort;
use crate::domains::obligations::ObligationReviewPort;
use crate::domains::persons::identity::{
    PersonIdentityCandidatePayload, PersonIdentityError, load_identity_candidate_payload,
    parse_person_identity_candidate_kind, parse_person_identity_review_state,
    person_identity_candidate_detected_event_type,
};
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxError, ReviewInboxPort, ReviewItemKind,
};
use crate::domains::tasks::candidates::TaskCandidatePort;
use crate::platform::events::{EventStoreError, StoredEventEnvelope};
use crate::workflows::email_intelligence::{
    EmailIntelligenceService, EmailKnowledgeCandidate, EmailSummaryContract,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, ensure_decision_review_item, ensure_obligation_review_item,
    ensure_relationship_review_item, ensure_task_candidate_review_item,
    sync_identity_candidate_review_state_in_transaction, sync_identity_candidate_to_review,
};

#[derive(Debug, Error)]
pub enum ReviewInboxWorkflowError {
    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Decision(#[from] crate::domains::decisions::DecisionReviewPortError),

    #[error(transparent)]
    Obligation(#[from] crate::domains::obligations::ObligationReviewPortError),

    #[error(transparent)]
    Relationship(#[from] crate::domains::relationships::RelationshipReviewPortError),

    #[error(transparent)]
    TaskCandidate(#[from] crate::domains::tasks::candidates::TaskCandidateError),

    #[error(transparent)]
    PersonIdentity(#[from] PersonIdentityError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn refresh_message_task_candidates_into_review(
    pool: &PgPool,
    message_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if message_ids.is_empty() {
        return Ok(0);
    }

    let refreshed = TaskCandidatePort::new(pool.clone())
        .refresh_message_candidates_for_ids(message_ids)
        .await?;
    let observation_ids = load_message_observation_ids(pool, message_ids).await?;
    let _ = sync_task_candidates_to_review_for_observations(pool, &observation_ids).await?;
    Ok(refreshed)
}

pub async fn refresh_message_decisions_into_review(
    pool: &PgPool,
    message_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if message_ids.is_empty() {
        return Ok(0);
    }

    let refreshed = DecisionReviewPort::new(pool.clone())
        .refresh_message_candidates_for_ids(message_ids)
        .await?;
    let observation_ids = load_message_observation_ids(pool, message_ids).await?;
    let _ = sync_decisions_to_review_for_observations(pool, &observation_ids).await?;
    Ok(refreshed)
}

pub async fn refresh_message_knowledge_candidates_into_review(
    pool: &PgPool,
    messages: &[ProjectedMessage],
) -> Result<usize, ReviewInboxWorkflowError> {
    if messages.is_empty() {
        return Ok(0);
    }

    let review_store = ReviewInboxPort::new(pool.clone());
    let mut mirrored = 0;
    for message in messages {
        let summary_contract = message_summary_contract(message);
        for (candidate_group, candidate) in knowledge_candidates(&summary_contract) {
            let summary = if candidate.evidence.trim().is_empty() {
                format!("Source-backed {candidate_group} candidate from communication evidence")
            } else {
                candidate.evidence.clone()
            };
            let item = NewReviewItem::new(
                ReviewItemKind::KnowledgeCandidate,
                candidate.title.clone(),
                summary,
                knowledge_candidate_confidence(candidate_group),
            )
            .metadata(json!({
                "mirrored_from": "message_summary_contract",
                "message_id": message.message_id,
                "observation_id": message.observation_id,
                "candidate_group": candidate_group,
                "candidate_title": candidate.title,
            }));
            let evidence = NewReviewItemEvidence::new(message.observation_id.clone())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "message_summary_contract",
                    "message_id": message.message_id,
                    "candidate_group": candidate_group,
                }));
            let _ = review_store
                .create_with_evidence(&item, &[evidence])
                .await?;
            mirrored += 1;
        }
    }

    Ok(mirrored)
}

pub async fn refresh_message_people_candidates_into_review(
    pool: &PgPool,
    messages: &[ProjectedMessage],
) -> Result<usize, ReviewInboxWorkflowError> {
    if messages.is_empty() {
        return Ok(0);
    }

    let review_store = ReviewInboxPort::new(pool.clone());
    let mut mirrored = 0;
    for message in messages {
        let summary_contract = message_summary_contract(message);
        for candidate in &summary_contract.persona_candidates {
            let summary = if candidate.evidence.trim().is_empty() {
                "Source-backed persona candidate from communication evidence".to_owned()
            } else {
                candidate.evidence.clone()
            };
            let item = NewReviewItem::new(
                ReviewItemKind::NewPerson,
                candidate.title.clone(),
                summary,
                0.68,
            )
            .metadata(json!({
                "mirrored_from": "message_summary_contract",
                "message_id": message.message_id,
                "observation_id": message.observation_id,
                "candidate_group": "persona",
                "candidate_title": candidate.title,
            }));
            let evidence = NewReviewItemEvidence::new(message.observation_id.clone())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "message_summary_contract",
                    "message_id": message.message_id,
                    "candidate_group": "persona",
                }));
            let _ = review_store
                .create_with_evidence(&item, &[evidence])
                .await?;
            mirrored += 1;
        }

        for candidate in &summary_contract.organization_candidates {
            let summary = if candidate.evidence.trim().is_empty() {
                "Source-backed organization candidate from communication evidence".to_owned()
            } else {
                candidate.evidence.clone()
            };
            let item = NewReviewItem::new(
                ReviewItemKind::NewOrganization,
                candidate.title.clone(),
                summary,
                0.7,
            )
            .metadata(json!({
                "mirrored_from": "message_summary_contract",
                "message_id": message.message_id,
                "observation_id": message.observation_id,
                "candidate_group": "organization",
                "candidate_title": candidate.title,
            }));
            let evidence = NewReviewItemEvidence::new(message.observation_id.clone())
                .role("primary")
                .metadata(json!({
                    "mirrored_from": "message_summary_contract",
                    "message_id": message.message_id,
                    "candidate_group": "organization",
                }));
            let _ = review_store
                .create_with_evidence(&item, &[evidence])
                .await?;
            mirrored += 1;
        }
    }

    Ok(mirrored)
}

pub const PERSON_IDENTITY_REVIEW_INBOX_CONSUMER: &str = "person_identity_review_inbox";

pub async fn project_person_identity_review_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    project_person_identity_review_event_inner(&pool, event)
        .await
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

async fn project_person_identity_review_event_inner(
    pool: &PgPool,
    event: StoredEventEnvelope,
) -> Result<(), ReviewInboxWorkflowError> {
    if event.event.event_type == person_identity_candidate_detected_event_type() {
        let payload = identity_candidate_payload_from_event(&event)?;
        sync_identity_candidate_to_review(pool, &payload).await?;
        return Ok(());
    }

    if event.event.event_type == "person_identity.review_state_changed" {
        let identity_candidate_id =
            required_event_string(&event.event.payload, "identity_candidate_id")?;
        let review_state = parse_person_identity_review_state(required_event_string(
            &event.event.payload,
            "review_state",
        )?)?;
        let mut transaction = pool.begin().await?;
        let payload =
            load_identity_candidate_payload(&mut transaction, identity_candidate_id).await?;
        sync_identity_candidate_review_state_in_transaction(
            &mut transaction,
            identity_candidate_id,
            review_state,
            &payload,
        )
        .await?;
        transaction.commit().await?;
    }

    Ok(())
}

fn identity_candidate_payload_from_event(
    event: &StoredEventEnvelope,
) -> Result<PersonIdentityCandidatePayload, PersonIdentityError> {
    let payload = &event.event.payload;
    Ok(PersonIdentityCandidatePayload {
        candidate_kind: parse_person_identity_candidate_kind(required_event_string(
            payload,
            "candidate_kind",
        )?)?,
        left_person_id: required_event_string(payload, "left_person_id")?.to_owned(),
        right_person_id: payload
            .get("right_person_id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        email_address: payload
            .get("email_address")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        evidence_summary: required_event_string(payload, "evidence_summary")?.to_owned(),
        confidence: payload
            .get("confidence")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| PersonIdentityError::MissingPayloadField("confidence".to_owned()))?,
    })
}

fn required_event_string<'a>(
    payload: &'a serde_json::Value,
    field: &str,
) -> Result<&'a str, PersonIdentityError> {
    payload
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| PersonIdentityError::MissingPayloadField(field.to_owned()))
}

pub async fn sync_task_candidates_to_review_for_observations(
    pool: &PgPool,
    observation_ids: &[String],
) -> Result<usize, ReviewInboxWorkflowError> {
    if observation_ids.is_empty() {
        return Ok(0);
    }

    let rows = sqlx::query(
        r#"
        SELECT
            task_candidate_id,
            source_kind,
            source_id,
            candidate_kind,
            candidate_metadata,
            project_id,
            title,
            due_text,
            assignee_label,
            confidence,
            evidence_excerpt,
            observation_id
        FROM task_candidates
        WHERE source_kind = 'observation'
          AND review_state = 'suggested'
          AND observation_id = ANY($1)
        ORDER BY updated_at DESC, task_candidate_id
        "#,
    )
    .bind(observation_ids.to_vec())
    .fetch_all(pool)
    .await?;
    let row_count = rows.len();

    for row in rows {
        let task_candidate_id: String = row.try_get("task_candidate_id")?;
        let observation_id: String = row.try_get("observation_id")?;
        let candidate = crate::domains::tasks::candi
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/workflows/review_mirror.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/review_mirror.rs`
- Size bytes / Размер в байтах: `35344`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::decisions::{Decision, DecisionReviewState};
use crate::domains::obligations::{Obligation, ObligationReviewState};
use crate::domains::persons::identity::{
    PersonIdentityCandidateKind, PersonIdentityCandidatePayload, PersonIdentityReviewState,
};
use crate::domains::projects::link_reviews::{ProjectLinkReviewState, ProjectLinkTargetKind};
use crate::domains::relationships::{Relationship, RelationshipReviewState};
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxError, ReviewInboxPort, ReviewItem,
    ReviewItemKind, ReviewItemStatus, ReviewPromotionTarget,
};
use crate::domains::tasks::candidates::{
    StoredCandidateRow, TaskCandidateReviewState, task_id_from_candidate,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationPort, ObservationPortError,
};

#[derive(Debug, Error)]
pub enum ReviewMirrorError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),

    #[error(transparent)]
    Observation(#[from] ObservationPortError),

    #[error("review-backed observation is required: {0}")]
    ObservationRequired(String),
}

pub async fn sync_decision_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    decision: &Decision,
) -> Result<(), ReviewMirrorError> {
    let evidence_row = sqlx::query(
        r#"
        SELECT observation_id
        FROM decision_evidence
        WHERE decision_id = $1
          AND observation_id IS NOT NULL
        ORDER BY created_at ASC, evidence_id ASC
        LIMIT 1
        "#,
    )
    .bind(&decision.decision_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        ReviewInboxError::ReviewItemNotFound(format!("decision:{}", decision.decision_id))
    })?;
    let observation_id: String = evidence_row.try_get("observation_id")?;
    let review_item = ensure_decision_review_item_in_transaction(
        transaction,
        &decision.decision_id,
        &decision.title,
        &decision.rationale,
        decision.confidence,
        &observation_id,
    )
    .await?;

    sync_decision_review_item_status_in_transaction(transaction, decision, &review_item).await
}

pub(crate) async fn sync_decision_review_state_with_observation(
    pool: &sqlx::postgres::PgPool,
    decision: &Decision,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let review_item = ensure_decision_review_item_in_transaction(
        &mut transaction,
        &decision.decision_id,
        &decision.title,
        &decision.rationale,
        decision.confidence,
        observation_id,
    )
    .await?;
    sync_decision_review_item_status_in_transaction(&mut transaction, decision, &review_item)
        .await?;
    transaction.commit().await?;
    Ok(())
}

async fn sync_decision_review_item_status_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    decision: &Decision,
    review_item: &ReviewItem,
) -> Result<(), ReviewMirrorError> {
    match decision.review_state {
        DecisionReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        DecisionReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        DecisionReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new("decisions", "decision", &decision.decision_id),
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn sync_obligation_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &Obligation,
) -> Result<(), ReviewMirrorError> {
    let evidence_row = sqlx::query(
        r#"
        SELECT observation_id, quote
        FROM obligation_evidence
        WHERE obligation_id = $1
          AND observation_id IS NOT NULL
        ORDER BY created_at ASC, evidence_id ASC
        LIMIT 1
        "#,
    )
    .bind(&obligation.obligation_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        ReviewInboxError::ReviewItemNotFound(format!("obligation:{}", obligation.obligation_id))
    })?;
    let observation_id: String = evidence_row.try_get("observation_id")?;
    let summary: Option<String> = evidence_row.try_get("quote")?;
    let review_item = ensure_obligation_review_item_in_transaction(
        transaction,
        &obligation.obligation_id,
        &obligation.statement,
        summary.as_deref(),
        obligation.confidence,
        &observation_id,
    )
    .await?;

    sync_obligation_review_item_status_in_transaction(transaction, obligation, &review_item).await
}

pub(crate) async fn sync_obligation_review_state_with_observation(
    pool: &sqlx::postgres::PgPool,
    obligation: &Obligation,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let review_item = ensure_obligation_review_item_in_transaction(
        &mut transaction,
        &obligation.obligation_id,
        &obligation.statement,
        None,
        obligation.confidence,
        observation_id,
    )
    .await?;
    sync_obligation_review_item_status_in_transaction(&mut transaction, obligation, &review_item)
        .await?;
    transaction.commit().await?;
    Ok(())
}

async fn sync_obligation_review_item_status_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    obligation: &Obligation,
    review_item: &ReviewItem,
) -> Result<(), ReviewMirrorError> {
    match obligation.review_state {
        ObligationReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        ObligationReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        ObligationReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new("obligations", "obligation", &obligation.obligation_id),
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn sync_relationship_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    relationship: &Relationship,
) -> Result<(), ReviewMirrorError> {
    let evidence_row = sqlx::query(
        r#"
        SELECT observation_id, excerpt
        FROM relationship_evidence
        WHERE relationship_id = $1
          AND observation_id IS NOT NULL
        ORDER BY created_at ASC, evidence_id ASC
        LIMIT 1
        "#,
    )
    .bind(&relationship.relationship_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        ReviewInboxError::ReviewItemNotFound(format!(
            "relationship:{}",
            relationship.relationship_id
        ))
    })?;
    let observation_id: String = evidence_row.try_get("observation_id")?;
    let summary: Option<String> = evidence_row.try_get("excerpt")?;
    let review_item = ensure_relationship_review_item_in_transaction(
        transaction,
        &relationship.relationship_id,
        &relationship.relationship_type,
        relationship.source_entity_kind.as_str(),
        &relationship.source_entity_id,
        relationship.target_entity_kind.as_str(),
        &relationship.target_entity_id,
        relationship.confidence,
        summary.as_deref(),
        &observation_id,
    )
    .await?;

    match relationship.review_state {
        RelationshipReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        RelationshipReviewState::SystemAccepted => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Approved,
            )
            .await?;
        }
        RelationshipReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        RelationshipReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "relationships",
                    "relationship",
                    &relationship.relationship_id,
                ),
            )
            .await?;
        }
    }

    Ok(())
}

pub(crate) async fn sync_relationship_review_state_with_observation(
    pool: &sqlx::postgres::PgPool,
    relationship: &Relationship,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    let mut transaction = pool.begin().await?;
    let review_item = ensure_relationship_review_item_in_transaction(
        &mut transaction,
        &relationship.relationship_id,
        &relationship.relationship_type,
        relationship.source_entity_kind.as_str(),
        &relationship.source_entity_id,
        relationship.target_entity_kind.as_str(),
        &relationship.target_entity_id,
        relationship.confidence,
        None,
        observation_id,
    )
    .await?;

    match relationship.review_state {
        RelationshipReviewState::Suggested => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewItemStatus::New,
            )
            .await?;
        }
        RelationshipReviewState::SystemAccepted => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Approved,
            )
            .await?;
        }
        RelationshipReviewState::UserRejected => {
            let _ = ReviewInboxPort::transition_status_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewItemStatus::Dismissed,
            )
            .await?;
        }
        RelationshipReviewState::UserConfirmed => {
            let _ = ReviewInboxPort::promote_in_transaction(
                &mut transaction,
                &review_item.review_item_id,
                ReviewPromotionTarget::new(
                    "relationships",
                    "relationship",
                    &relationship.relationship_id,
                ),
            )
            .await?;
        }
    }

    transaction.commit().await?;
    Ok(())
}

pub(crate) async fn sync_identity_candidate_to_review(
    pool: &sqlx::postgres::PgPool,
    payload: &PersonIdentityCandidatePayload,
) -> Result<(), ReviewMirrorError> {
    let observ
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/workflows/review_promotion/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/review_promotion/mod.rs`
- Size bytes / Размер в байтах: `43167`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::decisions::{
    DecisionReviewPort, DecisionReviewState, NewDecision, NewDecisionEvidence,
};
use crate::domains::documents::core::{DocumentImportError, DocumentImportPort, NewDocumentImport};
use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationReviewPort,
    ObligationReviewState,
};
use crate::domains::organizations::api::{OrganizationCommandPort, OrganizationError};
use crate::domains::persons::api::{PersonProjectionError, PersonProjectionPort};
use crate::domains::persons::identity::{
    PersonIdentityPort, PersonIdentityReviewCommand, PersonIdentityReviewState,
};
use crate::domains::projects::core::ProjectCommandPort;
use crate::domains::projects::core::{NewProject, ProjectCommandPortError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewPort, ProjectLinkReviewState, ProjectLinkTargetKind,
};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewPort,
    RelationshipReviewState,
};
use crate::domains::review::{
    ReviewInboxError, ReviewInboxPort, ReviewItem, ReviewItemEvidenceRecord, ReviewItemKind,
    ReviewPromotionTarget,
};
use crate::domains::tasks::api::{NewTask, TaskCommandPort, TaskError};
use crate::domains::tasks::core::{ObligationTaskLinkPort, TaskCoreError};
use crate::platform::observations::{
    NewObservation, Observation, ObservationOriginKind, ObservationPort, ObservationPortError,
    link_domain_entity, materialize_review_transition_link,
};

#[derive(Debug, Error)]
pub enum ReviewPromotionError {
    #[error(transparent)]
    ReviewInbox(#[from] ReviewInboxError),
    #[error(transparent)]
    Task(#[from] TaskError),
    #[error(transparent)]
    TaskCore(#[from] TaskCoreError),
    #[error(transparent)]
    DocumentImport(#[from] DocumentImportError),
    #[error(transparent)]
    Decision(#[from] crate::domains::decisions::DecisionReviewPortError),
    #[error(transparent)]
    Obligation(#[from] crate::domains::obligations::ObligationReviewPortError),
    #[error(transparent)]
    Relationship(#[from] crate::domains::relationships::RelationshipReviewPortError),
    #[error(transparent)]
    PersonIdentity(#[from] crate::domains::persons::identity::PersonIdentityError),
    #[error(transparent)]
    PersonProjection(#[from] PersonProjectionError),
    #[error(transparent)]
    ProjectLinkReview(#[from] crate::domains::projects::link_reviews::ProjectLinkReviewError),
    #[error(transparent)]
    ProjectCommandPort(#[from] ProjectCommandPortError),
    #[error(transparent)]
    Organization(#[from] OrganizationError),
    #[error(transparent)]
    Observation(#[from] ObservationPortError),
    #[error("{0}")]
    InvalidTarget(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct ReviewPromotionService {
    pool: PgPool,
}

impl ReviewPromotionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn promote(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
    ) -> Result<ReviewItem, ReviewPromotionError> {
        self.promote_with_observation(review_item_id, target, None, None)
            .await
    }

    pub async fn promote_with_observation(
        &self,
        review_item_id: &str,
        target: ReviewPromotionTarget,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<ReviewItem, ReviewPromotionError> {
        let review_store = ReviewInboxPort::new(self.pool.clone());
        let item = review_store.get(review_item_id).await?;
        let evidence = review_store.list_evidence(review_item_id).await?;
        let resolved_target = self.materialize_target(&item, &target, &evidence).await?;
        Ok(review_store
            .promote_with_observation(review_item_id, resolved_target, observation_id, metadata)
            .await?)
    }

    async fn materialize_target(
        &self,
        item: &ReviewItem,
        target: &ReviewPromotionTarget,
        evidence: &[ReviewItemEvidenceRecord],
    ) -> Result<ReviewPromotionTarget, ReviewPromotionError> {
        match item.item_kind {
            ReviewItemKind::NewPerson => {
                let person_id = self
                    .upsert_person_from_review(item, target, evidence)
                    .await?;
                Ok(ReviewPromotionTarget::new("persons", "persona", person_id))
            }
            ReviewItemKind::NewOrganization => {
                let organization_id = self
                    .upsert_organization_from_review(item, target, evidence)
                    .await?;
                Ok(ReviewPromotionTarget::new(
                    "organizations",
                    "organization",
                    organization_id,
                ))
            }
            ReviewItemKind::IdentityCandidate => {
                let identity_candidate_id =
                    metadata_string(&item.metadata, "identity_candidate_id")
                        .unwrap_or_else(|| item.review_item_id.clone());
                let review_observation = self
                    .capture_review_transition_observation(
                        item,
                        "identity_candidate_review_promotion",
                        json!({
                            "identity_candidate_id": identity_candidate_id,
                            "review_state": "user_confirmed",
                        }),
                        format!("review-item://{}/identity-candidate", item.review_item_id),
                    )
                    .await?;
                let result = PersonIdentityPort::new(self.pool.clone())
                    .set_review_state(&PersonIdentityReviewCommand {
                        command_id: format!("review-promotion:{}", item.review_item_id),
                        identity_candidate_id: identity_candidate_id.clone(),
                        review_state: PersonIdentityReviewState::UserConfirmed,
                        actor_id: "review_promotion".to_owned(),
                    })
                    .await?;
                self.link_review_transition_observation(
                    &review_observation,
                    "persons",
                    "identity_candidate",
                    &result.identity_candidate_id,
                    json!({
                        "review_item_id": item.review_item_id,
                        "review_state": result.review_state.as_str(),
                        "event_id": result.event_id,
                    }),
                )
                .await?;
                Ok(ReviewPromotionTarget::new(
                    "persons",
                    "identity_candidate",
                    result.identity_candidate_id,
                ))
            }
            ReviewItemKind::ProjectLinkCandidate => {
                let project_id = metadata_string(&item.metadata, "project_id")
                    .unwrap_or_else(|| item.review_item_id.clone());
                let target_kind = metadata_string(&item.metadata, "target_kind")
                    .unwrap_or_else(|| "message".to_owned());
                let target_id = metadata_string(&item.metadata, "target_id")
                    .unwrap_or_else(|| item.review_item_id.clone());
                let review_observation = self
                    .capture_review_transition_observation(
                        item,
                        "project_link_review_promotion",
                        json!({
                            "project_id": project_id,
                            "target_kind": target_kind,
                            "target_id": target_id,
                            "review_state": "user_confirmed",
                        }),
                        format!(
                            "review-item://{}/project-link/{}/{}",
                            item.review_item_id, target_kind, target_id
                        ),
                    )
                    .await?;
                let result = ProjectLinkReviewPort::new(self.pool.clone())
                    .set_review_state(&ProjectLinkReviewCommand {
                        command_id: format!("review-promotion:{}", item.review_item_id),
                        project_id: project_id.clone(),
                        target_kind: match target_kind.as_str() {
                            "document" => ProjectLinkTargetKind::Document,
                            _ => ProjectLinkTargetKind::Message,
                        },
                        target_id: target_id.clone(),
                        review_state: ProjectLinkReviewState::UserConfirmed,
                        actor_id: "review_promotion".to_owned(),
                    })
                    .await?;
                self.link_review_transition_observation(
                    &review_observation,
                    "projects",
                    "project_link_review",
                    &result.event_id,
                    json!({
                        "review_item_id": item.review_item_id,
                        "project_id": result.project_id,
                        "target_kind": result.target_kind.as_str(),
                        "target_id": result.target_id,
                        "review_state": result.review_state.as_str(),
                    }),
                )
                .await?;
                Ok(ReviewPromotionTarget::new(
                    "projects",
                    "project_link_candidate",
                    format!(
                        "{}:{}:{}",
                        result.project_id,
                        result.target_kind.as_str(),
                        result.target_id
                    ),
                ))
            }
            ReviewItemKind::ContradictionCandidate => Err(ReviewPromotionError::InvalidTarget(
                "contradiction review items cannot be promoted".to_owned(),
            )),
            ReviewItemKind::PotentialTask => {
                let primary_observation_id =
                    primary_observation_id(evidence).unwrap_or_else(|| item.review_item_id.clone());
                let review_observation = self
                    .capture_review_transition_observation(
                        item,
                        "task_review_promotion",
                        json!({
                            "source_observation_id": primary_observation_id,
                            "target_kind": "task",
                            "review_state": "promoted",
                        }),
                        format!("review-item://{}/task", item.review_item_id),
                    )
                    .await?;
                let task = TaskCommandPort::new(self.pool.clone())
                    .create(&NewTask {
                        title: item.title.clone(),
                        description: Some(item.summary.clone()),
                        provenance_kind: Some("review_item".to_owned()),
                        provenance_id: Some(item.review_item_id.clone()),
                        source_kind: Some("observation".to_owned()),
                        source_id: Some(primary_observation_id),
                        source_type: Some("observation".to_owned()),
                        makosh_status: Some("ready".to_owned()),
                        priority_score: Some(item.confidence),
                        why: Some(item.summary.clone()),
                        tags: Some(json!(["review_promoted"])),
                        ..Default::default()
                    })
                    .await?;
                self.link_review_transition_observation(
                    &review_observation,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/workflows/task_creation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/workflows/task_creation.rs`
- Size bytes / Размер в байтах: `2423`
- Included characters / Включено символов: `2423`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::postgres::Postgres;
use sqlx::{PgPool, Transaction};

use crate::app::ApiError;
use crate::domains::tasks::api::{NewTask, Task, TaskCommandPort};
use crate::domains::tasks::core::materialize_task_observation_link_in_transaction;

pub(crate) struct WorkflowTaskCreateInput {
    pub title: String,
    pub description: Option<String>,
    pub provenance_kind: Option<String>,
    pub provenance_id: Option<String>,
    pub source_kind: String,
    pub source_id: String,
    pub source_type: String,
    pub due_at: Option<DateTime<Utc>>,
    pub created_from_event_id: String,
    pub created_by_actor_id: String,
    pub projection_observation_id: Option<String>,
    pub projection_metadata: Option<Value>,
}

pub(crate) async fn create_task_from_workflow_input(
    pool: &PgPool,
    transaction: &mut Transaction<'_, Postgres>,
    input: WorkflowTaskCreateInput,
) -> Result<Task, ApiError> {
    let task_store = TaskCommandPort::new(pool.clone());
    let task = task_store
        .create_in_transaction(
            transaction,
            &NewTask {
                title: input.title,
                description: input.description,
                provenance_kind: input.provenance_kind,
                provenance_id: input.provenance_id,
                source_kind: Some(input.source_kind),
                source_id: Some(input.source_id),
                source_type: Some(input.source_type),
                project_id: None,
                makosh_status: Some("new".to_owned()),
                priority_score: None,
                area: None,
                why: None,
                due_at: input.due_at,
                energy_type: None,
                confidentiality: Some("private_local".to_owned()),
                tags: None,
                linked_person_id: None,
                linked_organization_id: None,
                created_from_event_id: Some(input.created_from_event_id),
                created_by_actor_id: Some(input.created_by_actor_id),
            },
        )
        .await
        .map_err(ApiError::from)?;

    materialize_task_observation_link_in_transaction(
        transaction,
        input.projection_observation_id.as_deref(),
        Some("workflow_action_projection"),
        &task.task_id,
        input.projection_metadata,
    )
    .await
    .map_err(ApiError::from)?;

    Ok(task)
}
```

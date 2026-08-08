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

- Chunk ID / ID чанка: `034-source-backend-part-014`
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

### `backend/src/app/vault_reconciliation/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/errors.rs`
- Size bytes / Размер в байтах: `655`
- Included characters / Включено символов: `655`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::calendar::events::CalendarError;
use crate::domains::communications::core::CommunicationIngestionError;
use crate::platform::secrets::SecretReferenceError;
use crate::vault::HostVaultError;

#[derive(Debug, Error)]
pub(super) enum HostVaultReconciliationError {
    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Calendar(#[from] CalendarError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/app/vault_reconciliation/lifecycle.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/lifecycle.rs`
- Size bytes / Размер в байтах: `1456`
- Included characters / Включено символов: `1456`
- Truncated / Обрезано: `no`

```rust
use crate::app::AppState;
use crate::vault::VaultMode;

use super::service::reconcile_host_vault_manifest;

pub(crate) fn spawn_host_vault_manifest_reconciliation(state: &AppState) {
    if state.config.database_url().is_none() {
        return;
    }
    let Ok(status) = state.vault.status() else {
        tracing::warn!("host vault reconciliation skipped: vault status unavailable");
        return;
    };
    if status.state != VaultMode::Unlocked {
        return;
    }
    let Some(pool) = state.database.pool().cloned() else {
        return;
    };
    let vault = state.vault.clone();
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        tracing::warn!("host vault reconciliation skipped: no Tokio runtime");
        return;
    };

    handle.spawn(async move {
        match reconcile_host_vault_manifest(pool, vault).await {
            Ok(summary)
                if summary.restored_accounts > 0 || summary.restored_calendar_accounts > 0 =>
            {
                tracing::info!(
                    restored_accounts = summary.restored_accounts,
                    restored_calendar_accounts = summary.restored_calendar_accounts,
                    "host vault manifest reconciliation completed"
                );
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(error = %error, "host vault manifest reconciliation failed");
            }
        }
    });
}
```

### `backend/src/app/vault_reconciliation/manifest_enrichment.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/manifest_enrichment.rs`
- Size bytes / Размер в байтах: `2607`
- Included characters / Включено символов: `2607`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::vault::{HostVault, HostVaultManifestEntry, SecretEntryContext};

use super::errors::HostVaultReconciliationError;

pub(super) async fn enrich_manifest_entry_from_postgres(
    pool: &PgPool,
    vault: &HostVault,
    entry: &mut HostVaultManifestEntry,
) -> Result<(), HostVaultReconciliationError> {
    if entry.entry_kind != "provider_credential" {
        return Ok(());
    }

    let Some(row) = provider_account_metadata_row(pool, entry).await? else {
        return Ok(());
    };

    let metadata = manifest_metadata_from_row(&row, entry)?;
    vault.upsert_account_secret_manifest_entry(
        &entry.secret_ref,
        SecretEntryContext {
            entry_kind: &entry.entry_kind,
            account_id: &entry.account_id,
            purpose: &entry.purpose,
            secret_kind: &entry.secret_kind,
            label: &entry.label,
            metadata: &metadata,
        },
    )?;
    entry.metadata = metadata;
    Ok(())
}

async fn provider_account_metadata_row(
    pool: &PgPool,
    entry: &HostVaultManifestEntry,
) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT cpa.provider_kind, cpa.display_name, cpa.external_account_id, cpa.config
        FROM communication_provider_accounts cpa
        JOIN communication_provider_account_secret_refs refs
          ON refs.account_id = cpa.account_id
         AND refs.secret_purpose = $2
         AND refs.secret_ref = $3
        WHERE cpa.account_id = $1
        "#,
    )
    .bind(&entry.account_id)
    .bind(&entry.purpose)
    .bind(&entry.secret_ref)
    .fetch_optional(pool)
    .await
}

fn manifest_metadata_from_row(
    row: &sqlx::postgres::PgRow,
    entry: &HostVaultManifestEntry,
) -> Result<Value, sqlx::Error> {
    let provider_kind: String = row.try_get("provider_kind")?;
    let display_name: String = row.try_get("display_name")?;
    let external_account_id: String = row.try_get("external_account_id")?;
    let config: Value = row.try_get("config")?;
    let mut metadata = json!({
        "provider": provider_kind,
        "account_id": entry.account_id.clone(),
        "display_name": display_name,
        "external_account_id": external_account_id,
        "provider_account_config": config
    });
    if let Some(connected_services) = metadata
        .get("provider_account_config")
        .and_then(|config| config.get("connected_services"))
        .cloned()
    {
        metadata["connected_services"] = connected_services;
    }
    Ok(metadata)
}
```

### `backend/src/app/vault_reconciliation/metadata.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/metadata.rs`
- Size bytes / Размер в байтах: `1982`
- Included characters / Включено символов: `1982`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::domains::communications::core::EmailProviderKind;

pub(super) fn fallback_provider_account_config(
    provider_kind: EmailProviderKind,
    metadata: &Value,
    external_account_id: &str,
) -> Value {
    let connected_services = metadata
        .get("connected_services")
        .cloned()
        .unwrap_or_else(|| json!(["mail"]));
    match provider_kind {
        EmailProviderKind::Gmail => json!({
            "auth": "oauth",
            "api": "gmail",
            "connected_services": connected_services,
            "history_stream_id": "gmail:history"
        }),
        EmailProviderKind::Icloud => json!({
            "host": "imap.mail.me.com",
            "port": 993,
            "tls": true,
            "mailbox": "INBOX",
            "username": external_account_id,
            "connected_services": connected_services
        }),
        EmailProviderKind::Imap => json!({
            "username": external_account_id,
            "connected_services": connected_services
        }),
        _ => json!({}),
    }
}

pub(super) fn fallback_display_name(
    provider_kind: EmailProviderKind,
    label: &str,
    account_id: &str,
) -> String {
    let trimmed = label.trim();
    if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("IMAP password") {
        return trimmed.to_owned();
    }
    match provider_kind {
        EmailProviderKind::Gmail => "Google Workspace".to_owned(),
        EmailProviderKind::Icloud => "iCloud".to_owned(),
        EmailProviderKind::Imap => account_id.to_owned(),
        _ => account_id.to_owned(),
    }
}

pub(super) fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub(super) fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}
```

### `backend/src/app/vault_reconciliation/provider_recovery.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/provider_recovery.rs`
- Size bytes / Размер в байтах: `3025`
- Included characters / Включено символов: `3025`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::domains::communications::core::{EmailProviderKind, ProviderAccountSecretPurpose};
use crate::platform::secrets::{SecretKind, SecretStoreKind};
use crate::vault::HostVaultManifestEntry;

use super::metadata::{
    fallback_display_name, fallback_provider_account_config, metadata_string, non_empty,
};

pub(super) struct RecoverableProviderSecret {
    pub(super) account_id: String,
    pub(super) provider_kind: EmailProviderKind,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) secret_ref: String,
    pub(super) secret_kind: SecretKind,
    pub(super) store_kind: SecretStoreKind,
    pub(super) secret_purpose: ProviderAccountSecretPurpose,
    pub(super) label: String,
    pub(super) secret_metadata: Value,
    pub(super) provider_account_config: Value,
}

impl RecoverableProviderSecret {
    pub(super) fn from_manifest(entry: HostVaultManifestEntry) -> Option<Self> {
        if entry.entry_kind != "provider_credential" {
            return None;
        }
        let provider = metadata_string(&entry.metadata, "provider")?;
        let provider_kind = EmailProviderKind::try_from(provider.as_str()).ok()?;
        if !matches!(
            provider_kind,
            EmailProviderKind::Gmail | EmailProviderKind::Icloud | EmailProviderKind::Imap
        ) {
            return None;
        }

        let secret_kind = SecretKind::try_from(entry.secret_kind.as_str()).ok()?;
        let store_kind = SecretStoreKind::try_from(entry.store_kind.as_str()).ok()?;
        let secret_purpose = ProviderAccountSecretPurpose::try_from(entry.purpose.as_str()).ok()?;
        if !secret_purpose.accepts_secret_kind(secret_kind) {
            return None;
        }

        let account_id =
            non_empty(metadata_string(&entry.metadata, "account_id")).unwrap_or(entry.account_id);
        let display_name = non_empty(metadata_string(&entry.metadata, "display_name"))
            .unwrap_or_else(|| fallback_display_name(provider_kind, &entry.label, &account_id));
        let external_account_id =
            non_empty(metadata_string(&entry.metadata, "external_account_id"))
                .unwrap_or_else(|| account_id.clone());
        let provider_account_config = entry
            .metadata
            .get("provider_account_config")
            .filter(|value| value.is_object())
            .cloned()
            .unwrap_or_else(|| {
                fallback_provider_account_config(
                    provider_kind,
                    &entry.metadata,
                    &external_account_id,
                )
            });

        Some(Self {
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            secret_ref: entry.secret_ref,
            secret_kind,
            store_kind,
            secret_purpose,
            label: entry.label,
            secret_metadata: entry.metadata,
            provider_account_config,
        })
    }
}
```

### `backend/src/app/vault_reconciliation/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/service.rs`
- Size bytes / Размер в байтах: `3518`
- Included characters / Включено символов: `3518`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

use crate::domains::calendar::events::CalendarAccountStore;
use crate::domains::communications::core::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore, NewProviderAccount,
    NewProviderAccountSecretBinding,
};
use crate::platform::secrets::{NewSecretReference, SecretReferenceStore};
use crate::vault::HostVault;

use super::calendar_restore::restore_linked_calendar_account;
use super::errors::HostVaultReconciliationError;
use super::manifest_enrichment::enrich_manifest_entry_from_postgres;
use super::provider_recovery::RecoverableProviderSecret;
use super::summary::HostVaultReconciliationSummary;

pub(super) async fn reconcile_host_vault_manifest(
    pool: PgPool,
    vault: HostVault,
) -> Result<HostVaultReconciliationSummary, HostVaultReconciliationError> {
    let manifest = vault.account_secret_manifest()?;
    let secret_store = SecretReferenceStore::new(pool.clone());
    let provider_account_store = CommunicationProviderAccountStore::new(pool.clone());
    let provider_secret_binding_store = CommunicationProviderSecretBindingStore::new(pool.clone());
    let calendar_store = CalendarAccountStore::new(pool.clone());
    let mut summary = HostVaultReconciliationSummary::default();

    for mut entry in manifest {
        enrich_manifest_entry_from_postgres(&pool, &vault, &mut entry).await?;
        let Some(recoverable) = RecoverableProviderSecret::from_manifest(entry) else {
            continue;
        };
        restore_secret_reference(&secret_store, &recoverable).await?;
        restore_provider_account(&provider_account_store, &recoverable, &mut summary).await?;
        restore_provider_account_secret_binding(&provider_secret_binding_store, &recoverable)
            .await?;

        if restore_linked_calendar_account(&calendar_store, &recoverable).await? {
            summary.restored_calendar_accounts += 1;
        }
    }

    Ok(summary)
}

async fn restore_secret_reference(
    store: &SecretReferenceStore,
    secret: &RecoverableProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret.secret_ref,
                secret.secret_kind,
                secret.store_kind,
                &secret.label,
            )
            .metadata(secret.secret_metadata.clone()),
        )
        .await?;
    Ok(())
}

async fn restore_provider_account(
    store: &CommunicationProviderAccountStore,
    secret: &RecoverableProviderSecret,
    summary: &mut HostVaultReconciliationSummary,
) -> Result<(), HostVaultReconciliationError> {
    if store.get(&secret.account_id).await?.is_some() {
        return Ok(());
    }

    store
        .restore(
            &NewProviderAccount::new(
                &secret.account_id,
                secret.provider_kind,
                &secret.display_name,
                &secret.external_account_id,
            )
            .config(secret.provider_account_config.clone()),
        )
        .await?;
    summary.restored_accounts += 1;
    Ok(())
}

async fn restore_provider_account_secret_binding(
    store: &CommunicationProviderSecretBindingStore,
    secret: &RecoverableProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .restore(&NewProviderAccountSecretBinding::new(
            &secret.account_id,
            secret.secret_purpose,
            &secret.secret_ref,
        ))
        .await?;
    Ok(())
}
```

### `backend/src/app/vault_reconciliation/summary.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/vault_reconciliation/summary.rs`
- Size bytes / Размер в байтах: `198`
- Included characters / Включено символов: `198`
- Truncated / Обрезано: `no`

```rust
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct HostVaultReconciliationSummary {
    pub(super) restored_accounts: usize,
    pub(super) restored_calendar_accounts: usize,
}
```

### `backend/src/application/ai_signal_dispatch.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/ai_signal_dispatch.rs`
- Size bytes / Размер в байтах: `665`
- Included characters / Включено символов: `665`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::platform::events::{EventEnvelope, EventStoreError};

pub(crate) async fn dispatch_ai_runtime_signal(
    pool: PgPool,
    event_kind: &str,
    source_id: &str,
    subject: Value,
    payload: Value,
    provenance: Value,
    correlation_id: Option<&str>,
) -> Result<Option<EventEnvelope>, EventStoreError> {
    crate::domains::signal_hub::dispatch_ai_helper_signal(
        pool,
        event_kind,
        source_id,
        subject,
        payload,
        provenance,
        correlation_id,
    )
    .await
    .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}
```

### `backend/src/application/bootstrap.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/bootstrap.rs`
- Size bytes / Размер в байтах: `94596`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::platform::events::EventBus;
use crate::vault::{HostVault, VaultMode};

static MAIL_BACKGROUND_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_OUTBOX_DELIVERY_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static TELEGRAM_COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_RUNTIME_RESTORE_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_TOKEN_MAINTENANCE_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_RECORDING_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_RETENTION_CLEANUP_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_RUNTIME_EVENT_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSON_DERIVED_EVIDENCE_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_SIGNAL_DETECTION_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_CALENDAR_MATCHING_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ZOOM_PARTICIPANT_IDENTITY_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static YANDEX_TELEMOST_RETENTION_CLEANUP_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static YANDEX_TELEMOST_CALENDAR_MATCHING_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_CONSUMER_DATABASES: LazyLock<
    Mutex<HashSet<String>>,
> = LazyLock::new(|| Mutex::new(HashSet::new()));
static PERSON_IDENTITY_REVIEW_INBOX_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static PROJECT_LINK_REVIEW_EFFECTS_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_CONSUMER_DATABASES: LazyLock<
    Mutex<HashSet<String>>,
> = LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_HUB_RAW_SIGNAL_CONSUMER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static EVENT_OUTBOX_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static SIGNAL_REPLAY_DISPATCHER_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

const MAIL_BACKGROUND_SYNC_RUNTIME: &str = "mail_background_sync";
const MAIL_OUTBOX_DELIVERY_RUNTIME: &str = "mail_outbox_delivery";
const TELEGRAM_COMMAND_EXECUTOR_RUNTIME: &str = "telegram_command_executor";
const WHATSAPP_COMMAND_EXECUTOR_RUNTIME: &str = "whatsapp_command_executor";
const WHATSAPP_RUNTIME_RESTORE_RECONCILIATION_RUNTIME: &str =
    "whatsapp_runtime_restore_reconciliation";
const WHATSAPP_NATIVE_MD_STARTUP_RESTORE_CONFIG_KEY: &str = "native_md_live_smoke_enabled";
const WHATSAPP_NATIVE_MD_STARTUP_RESTORE_ALIAS_CONFIG_KEY: &str =
    "whatsapp_native_md_live_smoke_enabled";
const WHATSAPP_NATIVE_MD_RUNTIME_FEATURE_DISABLED_BLOCKER: &str =
    "whatsapp_native_md_runtime_feature_disabled";
const WHATSAPP_STARTUP_RESTORE_FAILED_BLOCKER: &str = "whatsapp_startup_restore_failed";
const ZOOM_TOKEN_MAINTENANCE_RUNTIME: &str = "zoom_token_maintenance";
const ZOOM_TOKEN_MAINTENANCE_TICK_SECONDS: u64 = 60;
const ZOOM_TOKEN_MAINTENANCE_REFRESH_EXPIRING_WITHIN_SECONDS: i64 = 300;
const ZOOM_RECORDING_SYNC_RUNTIME: &str = "zoom_recording_sync";
const ZOOM_RECORDING_SYNC_TICK_SECONDS: u64 = 300;
const ZOOM_RECORDING_SYNC_LOOKBACK_DAYS: i64 = 7;
const ZOOM_RETENTION_CLEANUP_RUNTIME: &str = "zoom_retention_cleanup";
const ZOOM_RETENTION_CLEANUP_TICK_SECONDS: u64 = 3600;
const ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT: i64 = 100;
const YANDEX_TELEMOST_RETENTION_CLEANUP_RUNTIME: &str = "yandex_telemost_retention_cleanup";
const YANDEX_TELEMOST_RETENTION_CLEANUP_TICK_SECONDS: u64 = 3600;
const YANDEX_TELEMOST_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT: i64 = 100;
const WHATSAPP_RUNTIME_EVENT_CONSUMER_RUNTIME: &str = "whatsapp_runtime_event_projection";
const WHATSAPP_PROVIDER_OBSERVATION_RECONCILIATION_RUNTIME: &str =
    "whatsapp_provider_observation_reconciliation";
const COMMUNICATION_PROVIDER_OBSERVATION_RUNTIME: &str =
    "communication_provider_observation_projection";
const PERSON_DERIVED_EVIDENCE_RUNTIME: &str = "person_derived_evidence";
const ZOOM_SIGNAL_DETECTION_RUNTIME: &str = "zoom_signal_detection";
const ZOOM_CALENDAR_MATCHING_RUNTIME: &str = "zoom_calendar_matching";
const ZOOM_PARTICIPANT_IDENTITY_RUNTIME: &str = "zoom_participant_identity";
const YANDEX_TELEMOST_CALENDAR_MATCHING_RUNTIME: &str = "yandex_telemost_calendar_matching";
const REALTIME_CONVERSATION_TRANSCRIPT_EXECUTION_RUNTIME: &str =
    "realtime_conversation_transcript_execution";
const PERSON_IDENTITY_REVIEW_INBOX_RUNTIME: &str = "person_identity_review_inbox";
const PROJECT_LINK_REVIEW_EFFECTS_RUNTIME: &str = "project_link_review_effects";
const REALTIME_CONVERSATION_TRANSCRIPT_PROJECTION_RUNTIME: &str =
    "realtime_conversation_transcript_projection";
const SIGNAL_HUB_RAW_SIGNAL_RUNTIME: &str = "signal_hub_raw_signal_dispatcher";
const EVENT_OUTBOX_DISPATCHER_RUNTIME: &str = "event_outbox_dispatcher";
const SIGNAL_REPLAY_DISPATCHER_RUNTIME: &str = "signal_replay_dispatcher";

#[derive(Clone)]
pub(crate) struct ApplicationBootstrapContext {
    pub(crate) pool: Option<PgPool>,
    pub(crate) database_url: Option<String>,
    pub(crate) nats_server_url: Option<String>,
    pub(crate) zoom_token_maintenance_scheduler_enabled: bool,
    pub(crate) zoom_recording_sync_scheduler_enabled: bool,
    pub(crate) zoom_retention_cleanup_scheduler_enabled: bool,
    pub(crate) vault: HostVault,
    pub(crate) telegram_runtime: TelegramRuntimeManager,
    pub(crate) event_bus: EventBus,
}

pub(crate) fn start_background_services(context: ApplicationBootstrapContext) {
    start_mail_background_sync(context.clone());
    start_mail_outbox_delivery(context.clone());
    start_telegram_command_executor(context.clone());
    start_whatsapp_command_executor(context.clone());
    start_whatsapp_runtime_restore_reconciliation(context.clone());
    start_zoom_token_maintenance(context.clone());
    start_zoom_recording_sync(context.clone());
    start_zoom_retention_cleanup(context.clone());
    start_yandex_telemost_retention_cleanup(context.clone());
    start_whatsapp_runtime_event_projection(context.clone());
    start_whatsapp_provider_observation_reconciliation(context.clone());
    start_communication_provider_observation_projection(context.clone());
    start_person_derived_evidence_projection(context.clone());
    start_zoom_signal_detection_projection(context.clone());
    start_zoom_calendar_matching_projection(context.clone());
    start_zoom_participant_identity_projection(context.clone());
    start_yandex_telemost_calendar_matching_projection(context.clone());
    start_realtime_conversation_transcript_execution(context.clone());
    start_person_identity_review_inbox_projection(context.clone());
    start_project_link_review_effects_projection(context.clone());
    start_realtime_conversation_transcript_projection(context.clone());
    start_signal_hub_raw_signal_dispatcher(context.clone());
    start_event_outbox_dispatcher(context.clone());
    start_signal_replay_dispatcher(context);
}

fn start_mail_background_sync(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_mail_background_sync_scheduler(&database_url) {
        return;
    }
    let vault = context.vault;

    tokio::spawn(async move {
        let store = crate::application::mail_background_sync::MailSyncStore::new(pool.clone());
        let service = crate::application::mail_background_sync::MailBackgroundSyncService::new(
            pool.clone(),
            vault.clone(),
            crate::application::mail_background_sync::DEFAULT_MAIL_SYNC_BLOB_ROOT,
            Arc::new(
                crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort::new(
                    pool.clone(),
                    vault,
                    Arc::new(
                        crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                            pool.clone(),
                        ),
                    ),
                    crate::application::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                ),
            ),
        );
        if let Err(error) = store.mark_orphaned_active_runs_failed(Utc::now()).await {
            tracing::warn!(error = %error, "mail background sync startup recovery failed");
        }
        let mut tick = tokio::time::interval(Duration::from_secs(30));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "mail",
                MAIL_BACKGROUND_SYNC_RUNTIME,
                json!({
                    "label": "Mail background sync",
                    "scope": "scheduler",
                }),
            )
            .await
            {
                continue;
            }
            if let Err(error) = service.run_due_accounts().await {
                tracing::warn!(error = %error, "mail background sync scheduler tick failed");
            }
        }
    });
}

fn start_mail_outbox_delivery(context: ApplicationBootstrapContext) {
    let Some(pool) = context.pool else {
        return;
    };
    let Some(database_url) = context.database_url else {
        return;
    };
    if !register_mail_outbox_delivery_scheduler(&database_url) {
        return;
    }
    let vault = context.vault;

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(10));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tick.tick().await;
            if !runtime_allows_processing(
                &pool,
                "mail",
                MAIL_OUTBOX_DELIVERY_RUNTIME,
                json!({
                    "label": "Mail outbox delivery",
                    "scope": "scheduler",
                }),
            )
            .await
            {
                continue;
            }
            if !host_vault_is_unlocked(&vault) {
                continue;
            }
            let store =
                crate::domains::communications::outbox::CommunicationOutboxStore::new(pool.clone());
            let sender =
                crate::domains::communications::outbox::CommunicationOutboxEmailSender::new(

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/calendar_meeting_outcomes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/calendar_meeting_outcomes.rs`
- Size bytes / Размер в байтах: `8662`
- Included characters / Включено символов: `8662`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use thiserror::Error;

use crate::domains::calendar::evidence::link_calendar_entity_in_transaction;
use crate::domains::calendar::meetings::{MeetingOutcome, MeetingOutcomeStore, MeetingsError};
use crate::domains::decisions::{
    DecisionEntityKind, DecisionEvidenceSourceKind, DecisionReviewState, DecisionStore,
    DecisionStoreError, NewDecision, NewDecisionEvidence, NewDecisionImpactedEntity,
};
use crate::domains::obligations::{
    NewObligation, NewObligationEvidence, ObligationEntityKind, ObligationEvidenceSourceKind,
    ObligationReviewState, ObligationStore, ObligationStoreError,
};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};
use crate::workflows::review_mirror::{
    ReviewMirrorError, sync_decision_review_state_in_transaction,
    sync_obligation_review_state_in_transaction,
};

#[derive(Clone)]
pub struct CalendarMeetingOutcomeApplicationService {
    pool: PgPool,
}

impl CalendarMeetingOutcomeApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_manual(
        &self,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_person_id: Option<&str>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<MeetingOutcome, CalendarMeetingOutcomeApplicationError> {
        let observation = ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    "MEETING",
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    json!({
                        "event_id": event_id,
                        "outcome_type": outcome_type,
                        "title": title,
                        "description": description,
                        "owner_person_id": owner_person_id,
                        "due_date": due_date,
                    }),
                    format!("calendar-event://{event_id}/meeting-outcome"),
                )
                .provenance(json!({
                    "captured_by": "calendar_meeting_outcome_application.add_manual",
                    "operation": "add_manual",
                })),
            )
            .await?;

        let mut transaction = self.pool.begin().await?;
        let mut outcome = MeetingOutcomeStore::add_with_observation_in_transaction(
            &mut transaction,
            event_id,
            outcome_type,
            title,
            description,
            owner_person_id,
            due_date,
            Some(&format!("observation:{}", observation.observation_id)),
            Some(&observation.observation_id),
        )
        .await?;

        if let Some(linked_entity_id) =
            linked_entity_for_outcome(&mut transaction, &outcome).await?
        {
            outcome = MeetingOutcomeStore::set_linked_entity_id_in_transaction(
                &mut transaction,
                &outcome.id,
                &linked_entity_id,
            )
            .await?;
            link_calendar_entity_in_transaction(
                &mut transaction,
                &observation.observation_id,
                "meeting_outcome",
                outcome.id.clone(),
                None,
                json!({
                    "event_id": event_id,
                    "outcome_type": outcome.outcome_type,
                    "linked_entity_id": outcome.linked_entity_id,
                }),
                None,
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(outcome)
    }
}

async fn linked_entity_for_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
) -> Result<Option<String>, CalendarMeetingOutcomeApplicationError> {
    let evidence_observation_id = calendar_event_observation_id(transaction, &outcome.event_id)
        .await?
        .unwrap_or_else(|| outcome.event_id.clone());
    match outcome.outcome_type.as_str() {
        "decision" => {
            linked_decision_for_outcome(transaction, outcome, &evidence_observation_id).await
        }
        "promise" => {
            linked_obligation_for_outcome(transaction, outcome, &evidence_observation_id).await
        }
        _ => Ok(None),
    }
}

async fn calendar_event_observation_id(
    transaction: &mut Transaction<'_, Postgres>,
    event_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM calendar_events
        WHERE event_id = $1
        "#,
    )
    .bind(event_id)
    .fetch_optional(&mut **transaction)
    .await
}

async fn linked_decision_for_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
    observation_id: &str,
) -> Result<Option<String>, CalendarMeetingOutcomeApplicationError> {
    let decision = NewDecision::new(
        outcome.title.clone(),
        outcome
            .description
            .clone()
            .unwrap_or_else(|| outcome.title.clone()),
        outcome.confidence,
        DecisionReviewState::Suggested,
    )
    .metadata(json!({
        "source_domain": "calendar",
        "source_entity_kind": "meeting_outcome",
        "meeting_outcome_id": outcome.id,
        "event_id": outcome.event_id,
    }));
    let evidence =
        NewDecisionEvidence::new(DecisionEvidenceSourceKind::Event, outcome.event_id.clone())
            .with_observation_id(Some(observation_id.to_owned()))
            .quote(
                outcome
                    .description
                    .clone()
                    .unwrap_or_else(|| outcome.title.clone()),
            )
            .metadata(json!({
                "source_domain": "calendar",
                "meeting_outcome_id": outcome.id,
            }));
    let impacted_entity =
        NewDecisionImpactedEntity::new(DecisionEntityKind::Event, outcome.event_id.clone())
            .impact_type("meeting_outcome")
            .metadata(json!({ "meeting_outcome_id": outcome.id }));
    let stored = DecisionStore::upsert_with_evidence_in_transaction(
        transaction,
        &decision,
        &[evidence],
        &[impacted_entity],
    )
    .await?;
    sync_decision_review_state_in_transaction(transaction, &stored).await?;
    Ok(Some(stored.decision_id))
}

async fn linked_obligation_for_outcome(
    transaction: &mut Transaction<'_, Postgres>,
    outcome: &MeetingOutcome,
    observation_id: &str,
) -> Result<Option<String>, CalendarMeetingOutcomeApplicationError> {
    let Some(owner_person_id) = outcome
        .owner_person_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(None);
    };
    let mut obligation = NewObligation::new(
        ObligationEntityKind::Persona,
        owner_person_id.to_owned(),
        outcome.title.clone(),
        outcome.confidence,
        ObligationReviewState::Suggested,
    )
    .metadata(json!({
        "source_domain": "calendar",
        "source_entity_kind": "meeting_outcome",
        "meeting_outcome_id": outcome.id,
        "event_id": outcome.event_id,
    }));
    if let Some(due_date) = outcome.due_date {
        obligation = obligation.due_at(due_date);
    }
    let evidence = NewObligationEvidence::new(
        ObligationEvidenceSourceKind::Event,
        outcome.event_id.clone(),
    )
    .with_observation_id(Some(observation_id.to_owned()))
    .quote(
        outcome
            .description
            .clone()
            .unwrap_or_else(|| outcome.title.clone()),
    )
    .metadata(json!({
        "source_domain": "calendar",
        "meeting_outcome_id": outcome.id,
    }));
    let stored =
        ObligationStore::upsert_with_evidence_in_transaction(transaction, &obligation, &[evidence])
            .await?;
    sync_obligation_review_state_in_transaction(transaction, &stored).await?;
    Ok(Some(stored.obligation_id))
}

#[derive(Debug, Error)]
pub enum CalendarMeetingOutcomeApplicationError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Meetings(#[from] MeetingsError),

    #[error(transparent)]
    Decision(#[from] DecisionStoreError),

    #[error(transparent)]
    Obligation(#[from] ObligationStoreError),

    #[error(transparent)]
    ReviewMirror(#[from] ReviewMirrorError),
}
```

### `backend/src/application/communication_fixture_ingest.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/communication_fixture_ingest.rs`
- Size bytes / Размер в байтах: `120732`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::PgPool;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

use crate::application::provider_runtime_contracts::WhatsAppProviderRuntimeRef;
use crate::application::review_inbox::{
    ReviewInboxWorkflowError, refresh_message_decisions_into_review,
    refresh_message_knowledge_candidates_into_review,
    refresh_message_people_candidates_into_review, refresh_message_task_candidates_into_review,
};
use crate::domains::communications::core::CommunicationIngestionPort;
use crate::domains::communications::messages::{
    CommunicationSignalProjectionError, MessageProjectionError, MessageProjectionStore,
    NewProjectedMessage, ProviderChannelMessageStore, ProviderCommunicationMessagePortError,
    project_accepted_signal_if_runtime_allows, project_whatsapp_content_observed,
    project_whatsapp_delivery_state_observed,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, NewCommunicationAttachment,
    NewCommunicationBlob,
};
use crate::domains::persons::core::{PersonCoreError, PersonsIdentityStore};
use crate::domains::signal_hub::{
    SignalHubError, dispatch_telegram_raw_signal, dispatch_whatsapp_raw_signal,
};
use crate::integrations::telegram::client::{
    NewTelegramMessage, TelegramError, TelegramMessageIngestResult, TelegramStore, telegram_chat_id,
};
use crate::integrations::whatsapp::client::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappWebCallIngestResult, WhatsappWebDialogIngestResult,
    WhatsappWebError, WhatsappWebMediaIngestResult, WhatsappWebMessageDeleteIngestResult,
    WhatsappWebMessageIngestResult, WhatsappWebMessageUpdateIngestResult,
    WhatsappWebParticipantIngestResult, WhatsappWebPresenceIngestResult,
    WhatsappWebReactionIngestResult, WhatsappWebReceiptIngestResult,
    WhatsappWebRuntimeEventIngestResult, WhatsappWebStatusDeleteIngestResult,
    WhatsappWebStatusIngestResult, WhatsappWebStatusViewIngestResult,
};
use crate::platform::calls::CallError;
use crate::platform::calls::{CallDirection, CallIntelligenceStore, CallState, NewTelegramCall};
use crate::platform::communications::NewRawCommunicationRecord;
use crate::platform::events::bus::{telegram_event_types, whatsapp_event_types};
use crate::platform::events::{EventBus, EventStore, EventStoreError, NewEventEnvelope};

const AUDIT_ACTOR_ID: &str = "makosh-frontend";
const WHATSAPP_CHANNEL_KINDS: &[&str] = &["whatsapp_web", "whatsapp_business_cloud"];
static WHATSAPP_FIXTURE_EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct TelegramFixtureIngestApplicationService {
    pool: PgPool,
    store: TelegramStore,
    event_store: EventStore,
    event_bus: EventBus,
}

impl TelegramFixtureIngestApplicationService {
    pub(crate) fn new(
        pool: PgPool,
        store: TelegramStore,
        event_store: EventStore,
        event_bus: EventBus,
    ) -> Self {
        Self {
            pool,
            store,
            event_store,
            event_bus,
        }
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.store.ingest_fixture_message(request).await?;
        let stored_raw = CommunicationIngestionPort::new(self.pool.clone())
            .record_raw_source(&observed.raw)
            .await?;
        let Some(accepted_event) =
            dispatch_telegram_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "telegram fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        let Some(projected) =
            project_accepted_signal_if_runtime_allows(self.pool.clone(), &accepted_event).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "telegram fixture signal did not produce an accepted projection".to_owned(),
            ));
        };
        let response = TelegramMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        };

        self.store
            .recompute_chat_unread_count(&observed.telegram_chat_id)
            .await?;
        let message_ids = vec![response.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;

        let event = build_event(
            telegram_event_types::MESSAGE_CREATED,
            &request.account_id,
            &response.message_id,
            telegram_message_snapshot_payload(
                &self.store,
                &response.message_id,
                json!({
                    "provider_chat_id": &request.provider_chat_id,
                    "provider_message_id": &request.provider_message_id,
                    "delivery_state": request.delivery_state.as_str(),
                    "runtime_kind": "fixture",
                }),
            )
            .await?,
        );
        self.publish_event(event).await?;

        Ok(response)
    }

    async fn publish_event(
        &self,
        event: NewEventEnvelope,
    ) -> Result<(), CommunicationFixtureIngestError> {
        if let Err(error) = self.event_store.append(&event).await {
            tracing::warn!(error = %error, "failed to append fixture ingest event");
            return Err(error.into());
        }
        let _ = self.event_bus.broadcast(event);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct WhatsappFixtureIngestApplicationService {
    pool: PgPool,
    runtime: WhatsAppProviderRuntimeRef,
    event_store: EventStore,
    event_bus: EventBus,
}

impl WhatsappFixtureIngestApplicationService {
    pub(crate) fn new(
        pool: PgPool,
        runtime: WhatsAppProviderRuntimeRef,
        event_store: EventStore,
        event_bus: EventBus,
    ) -> Self {
        Self {
            pool,
            runtime,
            event_store,
            event_bus,
        }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) fn event_store(&self) -> &EventStore {
        &self.event_store
    }

    pub(crate) fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub(crate) async fn ingest_message(
        &self,
        request: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_with_reconciliation_source(request, "provider_observed.fixture_message")
            .await
    }

    pub(crate) async fn ingest_runtime_bridge_message(
        &self,
        request: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        self.ingest_message_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_message",
        )
        .await
    }

    async fn ingest_message_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebMessage,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebMessageIngestResult, CommunicationFixtureIngestError> {
        let observed = self.runtime.ingest_fixture_message(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = CommunicationIngestionPort::new(self.pool.clone())
            .record_raw_source(&observed_raw)
            .await?;
        let Some(accepted_event) =
            dispatch_whatsapp_raw_signal(self.pool.clone(), &stored_raw).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal was not accepted by Signal Hub".to_owned(),
            ));
        };
        let Some(projected) =
            project_accepted_signal_if_runtime_allows(self.pool.clone(), &accepted_event).await?
        else {
            return Err(CommunicationFixtureIngestError::SignalControlBlocked(
                "whatsapp fixture signal did not produce an accepted projection".to_owned(),
            ));
        };
        self.project_whatsapp_message_refs(
            request,
            &projected.message_id,
            &projected.raw_record_id,
        )
        .await?;
        self.upsert_whatsapp_person_identity_traces_for_message(
            request,
            &stored_raw.observation_id,
        )
        .await?;
        self.publish_whatsapp_command_reconciled_events(
            self.runtime
                .reconcile_fixture_message_commands(request)
                .await?,
            reconciliation_source,
        )
        .await?;
        let message_ids = vec![projected.message_id.clone()];
        refresh_message_decisions_into_review(&self.pool, &message_ids).await?;
        refresh_message_task_candidates_into_review(&self.pool, &message_ids).await?;
        if let Some(projected_message) = MessageProjectionStore::new(self.pool.clone())
            .message(&projected.message_id)
            .await?
        {
            let _ = refresh_message_people_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
            let _ = refresh_message_knowledge_candidates_into_review(
                &self.pool,
                std::slice::from_ref(&projected_message),
            )
            .await?;
        }

        Ok(WhatsappWebMessageIngestResult {
            raw_record_id: projected.raw_record_id,
            message_id: projected.message_id,
        })
    }

    pub(crate) async fn ingest_reaction(
        &self,
        request: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        self.ingest_reaction_with_reconciliation_source(
            request,
            "provider_observed.fixture_reaction",
        )
        .await
    }

    pub(crate) async fn ingest_runtime_bridge_reaction(
        &self,
        request: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        self.ingest_reaction_with_reconciliation_source(
            request,
            "provider_observed.runtime_bridge_reaction",
        )
        .await
    }

    async fn ingest_reaction_with_reconciliation_source(
        &self,
        request: &NewWhatsappWebReaction,
        reconciliation_source: &str,
    ) -> Result<WhatsappWebReactionIngestResult, CommunicationFixtureIngestError> {
        let account_context = self
            .whatsapp_account_projection_context(&request.account_id)
            .await?;
        let observed = self.runtime.ingest_fixture_reaction(request).await?;
        let observed_raw =
            annotate_whatsapp_raw_observed_source(&observed.raw, reconciliation_source)?;
        let stored_raw = self.record_and_accept_whatsapp_raw(&observed_raw).await?;
        let message = ProviderChannelMessageStore::new(self.pool.clone())
            .message_by_provider_record_id(
                &request.account_id,
                &request.provider_message_id,
                WHATSAPP_CHANNEL_KINDS,
            )
            .await?
            .ok_or_else(|| {
                CommunicationFixtureIngestError::SignalControlBlocked(format!(
                    "whatsa
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/communication_provider_writes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/communication_provider_writes.rs`
- Size bytes / Размер в байтах: `52121`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::collections::{HashMap, HashSet, VecDeque};

use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use sqlx::Row;
use thiserror::Error;

use crate::application::communication_fixture_ingest::{
    build_event, telegram_message_snapshot_payload,
};
use crate::application::telegram_runtime::{self, TelegramRuntimeUseCaseContext};
use crate::domains::communications::core::CommunicationIngestionPort;
use crate::domains::communications::messages::{
    CommunicationSignalProjectionError, ProviderChannelMessageStore,
    project_accepted_signal_if_runtime_allows,
};
use crate::domains::signal_hub::{SignalHubError, dispatch_telegram_raw_signal};
use crate::integrations::telegram::client::lifecycle;
use crate::integrations::telegram::client::models::messages::{
    TelegramDeleteRequest, TelegramEditRequest, TelegramForwardChainResponse, TelegramForwardRef,
    TelegramForwardRequest, TelegramLifecycleResponse, TelegramManualSendRequest,
    TelegramManualSendResponse, TelegramMessageReferenceSummary, TelegramMessageTombstone,
    TelegramMessageTombstoneListResponse, TelegramMessageVersion,
    TelegramMessageVersionListResponse, TelegramPinRequest, TelegramReaction,
    TelegramReactionGroup, TelegramReactionListResponse, TelegramReactionRequest,
    TelegramReactionResponse, TelegramReactionSummary, TelegramReplyChainResponse,
    TelegramReplyRef, TelegramReplyRequest, TelegramRestoreVisibilityRequest,
};
use crate::integrations::telegram::client::{TelegramError, TelegramStore};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, NewApiAuditRecord};
use crate::platform::events::bus::telegram_event_types;
use crate::platform::events::{EventBus, EventStore, NewEventEnvelope};

const AUDIT_ACTOR_ID: &str = "makosh-frontend";
const CANONICAL_REFERENCE_CHAIN_DEPTH: usize = 16;
const CANONICAL_REFERENCE_CHAIN_EDGES: usize = 128;

pub(crate) fn new_telegram_command_id() -> String {
    lifecycle::new_command_id()
}

async fn list_canonical_message_versions(
    pool: &sqlx::PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageVersion>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT
            version_id,
            message_id,
            account_id,
            provider_message_id,
            COALESCE(provider_conversation_id, '') AS provider_chat_id,
            version_number,
            body_text,
            edited_at AS edit_timestamp,
            source_event,
            diff_payload AS raw_diff_payload,
            provenance,
            created_at
        FROM communication_message_versions
        WHERE message_id = $1
        ORDER BY version_number ASC, created_at ASC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TelegramMessageVersion {
                version_id: row.try_get("version_id")?,
                message_id: row.try_get("message_id")?,
                account_id: row.try_get("account_id")?,
                provider_message_id: row.try_get("provider_message_id")?,
                provider_chat_id: row.try_get("provider_chat_id")?,
                version_number: row.try_get("version_number")?,
                body_text: row.try_get("body_text")?,
                edit_timestamp: row.try_get("edit_timestamp")?,
                source_event: row.try_get("source_event")?,
                raw_diff_payload: row.try_get("raw_diff_payload")?,
                provenance: row.try_get("provenance")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

async fn list_canonical_message_tombstones(
    pool: &sqlx::PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageTombstone>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT
            tombstone_id,
            message_id,
            account_id,
            provider_message_id,
            COALESCE(provider_conversation_id, '') AS provider_chat_id,
            reason_class,
            actor_class,
            observed_at,
            source_event,
            is_provider_delete,
            is_local_visible,
            metadata,
            provenance,
            created_at
        FROM communication_message_tombstones
        WHERE message_id = $1
        ORDER BY observed_at ASC, created_at ASC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TelegramMessageTombstone {
                tombstone_id: row.try_get("tombstone_id")?,
                message_id: row.try_get("message_id")?,
                account_id: row.try_get("account_id")?,
                provider_message_id: row.try_get("provider_message_id")?,
                provider_chat_id: row.try_get("provider_chat_id")?,
                reason_class: row.try_get("reason_class")?,
                actor_class: row.try_get("actor_class")?,
                observed_at: row.try_get("observed_at")?,
                source_event: row.try_get("source_event")?,
                is_provider_delete: row.try_get("is_provider_delete")?,
                is_local_visible: row.try_get("is_local_visible")?,
                metadata: row.try_get("metadata")?,
                provenance: row.try_get("provenance")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

async fn list_canonical_reactions(
    pool: &sqlx::PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReaction>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT
            reaction_id,
            message_id,
            account_id,
            provider_message_id,
            COALESCE(provider_conversation_id, '') AS provider_chat_id,
            COALESCE(sender_identity_id, provider_actor_id, reaction_id) AS sender_id,
            sender_display_name,
            reaction AS reaction_emoji,
            is_active,
            observed_at,
            source_event,
            provider_actor_id,
            metadata,
            provenance,
            created_at,
            updated_at
        FROM communication_message_reactions
        WHERE message_id = $1
          AND is_active = true
        ORDER BY observed_at DESC, created_at DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TelegramReaction {
                reaction_id: row.try_get("reaction_id")?,
                message_id: row.try_get("message_id")?,
                account_id: row.try_get("account_id")?,
                provider_message_id: row.try_get("provider_message_id")?,
                provider_chat_id: row.try_get("provider_chat_id")?,
                sender_id: row.try_get("sender_id")?,
                sender_display_name: row.try_get("sender_display_name")?,
                reaction_emoji: row.try_get("reaction_emoji")?,
                is_active: row.try_get("is_active")?,
                observed_at: row.try_get("observed_at")?,
                source_event: row.try_get("source_event")?,
                provider_actor_id: row.try_get("provider_actor_id")?,
                metadata: row.try_get("metadata")?,
                provenance: row.try_get("provenance")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .collect()
}

fn canonical_reaction_summary(
    message_id: &str,
    reactions: &[TelegramReaction],
) -> TelegramReactionSummary {
    let total_reactions = reactions.len() as i64;
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for reaction in reactions {
        groups
            .entry(reaction.reaction_emoji.clone())
            .or_default()
            .push(
                reaction
                    .sender_display_name
                    .clone()
                    .unwrap_or_else(|| reaction.sender_id.clone()),
            );
    }
    let grouped_reactions = groups
        .into_iter()
        .map(|(reaction_emoji, senders)| TelegramReactionGroup {
            reaction_emoji,
            count: senders.len() as i64,
            senders,
        })
        .collect();
    TelegramReactionSummary {
        message_id: message_id.to_owned(),
        total_reactions,
        active_reactions: total_reactions,
        reactions: grouped_reactions,
    }
}

async fn list_canonical_reference_summaries(
    pool: &sqlx::PgPool,
    message_ids: Vec<String>,
) -> Result<HashMap<String, TelegramMessageReferenceSummary>, TelegramError> {
    if message_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT
            message_id,
            provider_record_id,
            conversation_id,
            subject,
            sender,
            sender_display_name,
            body_text,
            occurred_at
        FROM communication_messages
        WHERE message_id = ANY($1)
        "#,
    )
    .bind(&message_ids)
    .fetch_all(pool)
    .await?;

    let mut summaries = HashMap::new();
    for row in rows {
        let message_id: String = row.try_get("message_id")?;
        summaries.insert(
            message_id.clone(),
            TelegramMessageReferenceSummary {
                message_id,
                provider_message_id: row.try_get("provider_record_id")?,
                provider_chat_id: row.try_get("conversation_id")?,
                chat_title: row.try_get("subject")?,
                sender: row.try_get("sender")?,
                sender_display_name: row.try_get("sender_display_name")?,
                text: row.try_get("body_text")?,
                occurred_at: row.try_get("occurred_at")?,
            },
        );
    }
    Ok(summaries)
}

async fn canonical_reply_refs_by_target(
    pool: &sqlx::PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT
            message_ref_id AS reply_ref_id,
            source_message_id,
            target_message_id,
            account_id,
            COALESCE(provider_conversation_id, '') AS provider_chat_id,
            COALESCE(source_provider_id, '') AS source_provider_id,
            COALESCE(target_provider_id, '') AS target_provider_id,
            depth AS reply_depth,
            COALESCE((metadata->>'is_topic_reply')::boolean, false) AS is_topic_reply,
            metadata->>'topic_id' AS topic_id,
            metadata,
            provenance,
            created_at
        FROM communication_message_refs
        WHERE ref_kind = 'reply'
          AND target_message_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(TelegramReplyRef {
                reply_ref_id: row.try_get("reply_ref_id")?,
                source_message_id: row.try_get("source_message_id")?,
                target_message_id: row.try_get("target_message_id")?,
                account_id: row.try_get("account_id")?,
                provider_chat_id: row.try_get("provider_chat_id")?,
                source_provider_id: row.try_get("source_provider_id")?,
                target_provider_id: row.try_get("target_provider_id")?,
                reply_depth: row.try_get("reply_depth")?,
                is_topic_reply: row.try_get("is_topic_reply")?,
                topic_id: row.try_get("topic_id")?,
                source_message_summary: None,
                target_message_summary: None,
                metadata: row.try_get("metadata")?,
                provenance: row.try_get("provenance")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

async fn canonical_reply_refs_by_source(
    pool: &sqlx::PgPool,
    message_id: &str,
) -> Result<Vec<TelegramReplyRef>, TelegramError> {

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/communication_send.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/communication_send.rs`
- Size bytes / Размер в байтах: `4848`
- Included characters / Включено символов: `4848`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationProviderAccountStore,
};
use crate::domains::communications::service::{
    CommunicationCommandService, CommunicationCommandServiceError, CommunicationOutboxSendCommand,
};
use crate::platform::audit::ApiAuditError;
use crate::platform::audit::NewApiAuditRecord;
use crate::platform::communications::OutgoingEmail;

#[derive(Clone)]
pub(crate) struct CommunicationSendDependencies {
    pool: sqlx::postgres::PgPool,
    audit_log: crate::platform::audit::ApiAuditLog,
}

impl CommunicationSendDependencies {
    pub(crate) fn new(
        pool: sqlx::postgres::PgPool,
        audit_log: crate::platform::audit::ApiAuditLog,
    ) -> Self {
        Self { pool, audit_log }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CommunicationSendRequest {
    pub(crate) account_id: String,
    pub(crate) to: Vec<String>,
    pub(crate) cc: Vec<String>,
    pub(crate) bcc: Vec<String>,
    pub(crate) subject: String,
    pub(crate) body_text: String,
    pub(crate) body_html: Option<String>,
    pub(crate) in_reply_to: Option<String>,
    pub(crate) references: Vec<String>,
    pub(crate) draft_id: Option<String>,
    pub(crate) scheduled_send_at: Option<DateTime<Utc>>,
    pub(crate) undo_send_seconds: Option<i64>,
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug)]
pub(crate) struct CommunicationSendResult {
    pub(crate) message_id: String,
    pub(crate) outbox_id: Option<String>,
    pub(crate) accepted: Vec<String>,
    pub(crate) accepted_recipients: Vec<String>,
    pub(crate) transport: String,
    pub(crate) status: String,
    pub(crate) scheduled_send_at: Option<DateTime<Utc>>,
    pub(crate) undo_deadline_at: Option<DateTime<Utc>>,
    pub(crate) failure_reason: Option<String>,
}

pub(crate) async fn send_email(
    deps: &CommunicationSendDependencies,
    req: CommunicationSendRequest,
) -> Result<CommunicationSendResult, CommunicationSendError> {
    let scheduled_send_at = req.scheduled_send_at;
    let undo_send_seconds = req.undo_send_seconds;
    let draft_id = req.draft_id.clone();
    let account = CommunicationProviderAccountStore::new(deps.pool.clone())
        .get(&req.account_id)
        .await?
        .ok_or(CommunicationSendError::ProviderAccountNotFound)?;
    let email = OutgoingEmail {
        from: account.external_account_id.clone(),
        to: req.to,
        cc: req.cc,
        bcc: req.bcc,
        subject: req.subject,
        body_text: req.body_text,
        body_html: req.body_html,
        in_reply_to: req.in_reply_to,
        references: req.references,
    };

    if email
        .to
        .iter()
        .chain(email.cc.iter())
        .chain(email.bcc.iter())
        .all(|recipient| recipient.trim().is_empty())
    {
        return Err(CommunicationSendError::InvalidRequest(
            "at least one recipient is required",
        ));
    }
    if !req.metadata.is_object() {
        return Err(CommunicationSendError::InvalidRequest(
            "message metadata must be a JSON object",
        ));
    }

    let recipient_count = email.to.len() + email.cc.len() + email.bcc.len();
    let accepted_recipients = email
        .to
        .iter()
        .chain(email.cc.iter())
        .chain(email.bcc.iter())
        .cloned()
        .collect::<Vec<_>>();
    let item = CommunicationCommandService::new(deps.pool.clone())
        .enqueue_outbox_send(
            &account,
            &email,
            &CommunicationOutboxSendCommand {
                draft_id,
                scheduled_send_at,
                undo_send_seconds,
                metadata: req.metadata,
            },
        )
        .await?;

    deps.audit_log
        .record(&NewApiAuditRecord::communication_email_send(
            "makosh-frontend",
            &account.account_id,
            recipient_count,
        ))
        .await?;

    Ok(CommunicationSendResult {
        message_id: item.outbox_id.clone(),
        outbox_id: Some(item.outbox_id),
        accepted: accepted_recipients.clone(),
        accepted_recipients,
        transport: "outbox".to_owned(),
        status: item.status.as_str().to_owned(),
        scheduled_send_at: item.scheduled_send_at,
        undo_deadline_at: item.undo_deadline_at,
        failure_reason: None,
    })
}

#[derive(Debug, Error)]
pub(crate) enum CommunicationSendError {
    #[error("{0}")]
    InvalidRequest(&'static str),

    #[error("provider account was not found")]
    ProviderAccountNotFound,

    #[error(transparent)]
    CommunicationIngestion(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Command(#[from] CommunicationCommandServiceError),

    #[error(transparent)]
    Audit(#[from] ApiAuditError),
}
```

### `backend/src/application/consistency_review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/consistency_review.rs`
- Size bytes / Размер в байтах: `56`
- Included characters / Включено символов: `56`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::consistency_review::*;
```

### `backend/src/application/email_intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/email_intelligence.rs`
- Size bytes / Размер в байтах: `56`
- Included characters / Включено символов: `56`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::email_intelligence::*;
```

### `backend/src/application/mail_background_sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/mail_background_sync.rs`
- Size bytes / Размер в байтах: `58`
- Included characters / Включено символов: `58`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::mail_background_sync::*;
```

### `backend/src/application/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/mod.rs`
- Size bytes / Размер в байтах: `2694`
- Included characters / Включено символов: `2694`
- Truncated / Обрезано: `no`

```rust
pub(crate) mod ai_signal_dispatch;
pub(crate) mod bootstrap;
pub mod calendar_meeting_outcomes;
pub(crate) mod communication_fixture_ingest;
pub(crate) mod communication_provider_writes;
pub(crate) mod communication_send;
pub(crate) mod consistency_review;
pub(crate) mod email_intelligence;
pub(crate) mod mail_background_sync;
pub mod organization_contact_links;
pub(crate) mod person_derived_evidence;
pub(crate) mod project_link_review_effects;
pub(crate) mod project_link_review_mirror;
pub(crate) mod provider_runtime_contracts;
pub(crate) mod provider_runtime_services;
pub(crate) mod realtime_conversation_transcript_execution;
pub(crate) mod realtime_conversation_transcript_projection;
pub(crate) mod review_inbox;
pub(crate) mod review_promotion;
pub mod review_transitions;
pub(crate) mod signal_hub_replay;
pub(crate) mod task_creation;
pub(crate) mod telegram_runtime;
pub(crate) mod whatsapp_command_executor;
pub(crate) mod whatsapp_provider_observation_reconciliation;
pub(crate) mod whatsapp_runtime_event_projection;
pub(crate) mod whatsapp_runtime_signal_ingest;
pub(crate) mod workflow_action_person_projection;
pub(crate) mod yandex_telemost_calendar_matching;
pub(crate) mod zoom_calendar_matching;
pub(crate) mod zoom_participant_identity;
pub(crate) mod zoom_signal_detection;

pub(crate) use ai_signal_dispatch::*;
pub(crate) use bootstrap::*;
pub use calendar_meeting_outcomes::*;
pub(crate) use communication_fixture_ingest::*;
pub(crate) use communication_provider_writes::*;
pub(crate) use communication_send::*;
pub(crate) use consistency_review::*;
pub(crate) use email_intelligence::*;
pub(crate) use mail_background_sync::*;
pub use organization_contact_links::*;
pub(crate) use person_derived_evidence::*;
pub(crate) use project_link_review_effects::*;
pub(crate) use project_link_review_mirror::*;
pub(crate) use provider_runtime_contracts::*;
pub(crate) use provider_runtime_services::*;
pub(crate) use realtime_conversation_transcript_execution::*;
pub(crate) use realtime_conversation_transcript_projection::*;
pub(crate) use review_inbox::*;
pub(crate) use review_promotion::*;
pub use review_transitions::*;
pub use signal_hub_replay::*;
pub(crate) use task_creation::*;
pub(crate) use telegram_runtime::*;
pub(crate) use whatsapp_command_executor::*;
pub(crate) use whatsapp_provider_observation_reconciliation::*;
pub(crate) use whatsapp_runtime_event_projection::*;
pub(crate) use whatsapp_runtime_signal_ingest::*;
pub(crate) use workflow_action_person_projection::*;
pub(crate) use yandex_telemost_calendar_matching::*;
pub(crate) use zoom_calendar_matching::*;
pub(crate) use zoom_participant_identity::*;
pub(crate) use zoom_signal_detection::*;
```

### `backend/src/application/organization_contact_links.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/organization_contact_links.rs`
- Size bytes / Размер в байтах: `3975`
- Included characters / Включено символов: `3975`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::organizations::core::OrgContactLink;
use crate::domains::organizations::service::{
    OrganizationCommandService, OrganizationCommandServiceError,
};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind,
    RelationshipEvidenceSourceKind, RelationshipReviewPort, RelationshipReviewPortError,
    RelationshipReviewState,
};

#[derive(Clone)]
pub struct OrganizationContactLinkApplicationService {
    pool: PgPool,
}

impl OrganizationContactLinkApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn link_contact_manual(
        &self,
        organization_id: &str,
        person_id: &str,
        role: Option<&str>,
        department: Option<&str>,
        requested_source: &str,
    ) -> Result<OrgContactLink, OrganizationContactLinkApplicationError> {
        let link = OrganizationCommandService::new(self.pool.clone())
            .link_contact_manual(
                organization_id,
                person_id,
                role,
                department,
                requested_source,
            )
            .await?;

        materialize_member_of_relationship(
            &self.pool,
            &link,
            RelationshipReviewState::UserConfirmed,
            manual_contact_link_evidence(&link),
        )
        .await?;

        Ok(link)
    }
}

fn manual_contact_link_evidence(link: &OrgContactLink) -> NewRelationshipEvidence {
    if let Some(observation_id) = link.source.strip_prefix("observation:") {
        return NewRelationshipEvidence::observation(observation_id.to_owned())
            .excerpt(relationship_excerpt())
            .metadata(relationship_evidence_metadata(link));
    }

    NewRelationshipEvidence::new(
        RelationshipEvidenceSourceKind::Organization,
        link.organization_id.clone(),
    )
    .excerpt(relationship_excerpt())
    .metadata(relationship_evidence_metadata(link))
}

async fn materialize_member_of_relationship(
    pool: &PgPool,
    link: &OrgContactLink,
    review_state: RelationshipReviewState,
    evidence: NewRelationshipEvidence,
) -> Result<(), RelationshipReviewPortError> {
    let relationship = NewRelationship {
        source_entity_kind: RelationshipEntityKind::Persona,
        source_entity_id: link.person_id.clone(),
        target_entity_kind: RelationshipEntityKind::Organization,
        target_entity_id: link.organization_id.clone(),
        relationship_type: "member_of".to_owned(),
        trust_score: 0.5,
        strength_score: 0.5,
        confidence: link.confidence,
        review_state,
        valid_from: link.valid_from,
        valid_to: link.valid_to,
        metadata: json!({
            "compatibility_table": "organization_contact_links",
            "compatibility_record_id": link.id,
            "organization_id": link.organization_id,
            "person_id": link.person_id,
            "role": link.role,
            "department": link.department,
            "source": link.source,
        }),
    };
    let _ = RelationshipReviewPort::new(pool.clone())
        .upsert_with_evidence(&relationship, &[evidence])
        .await?;
    Ok(())
}

fn relationship_excerpt() -> String {
    "Persona is linked to organization through compatibility organization contact data.".to_owned()
}

fn relationship_evidence_metadata(link: &OrgContactLink) -> serde_json::Value {
    json!({
        "compatibility_table": "organization_contact_links",
        "compatibility_record_id": link.id,
        "organization_id": link.organization_id,
        "person_id": link.person_id,
    })
}

#[derive(Debug, Error)]
pub enum OrganizationContactLinkApplicationError {
    #[error(transparent)]
    Organization(#[from] OrganizationCommandServiceError),

    #[error(transparent)]
    Relationship(#[from] RelationshipReviewPortError),
}
```

### `backend/src/application/person_derived_evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/person_derived_evidence.rs`
- Size bytes / Размер в байтах: `61`
- Included characters / Включено символов: `61`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::person_derived_evidence::*;
```

### `backend/src/application/project_link_review_effects.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/project_link_review_effects.rs`
- Size bytes / Размер в байтах: `65`
- Included characters / Включено символов: `65`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::project_link_review_effects::*;
```

### `backend/src/application/project_link_review_mirror.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/project_link_review_mirror.rs`
- Size bytes / Размер в байтах: `1510`
- Included characters / Включено символов: `1510`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::domains::projects::link_reviews::{ProjectLinkReviewState, ProjectLinkTargetKind};
use crate::workflows::review_mirror::ReviewMirrorError;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn ensure_project_link_candidate_review_item(
    pool: &sqlx::postgres::PgPool,
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    title: &str,
    summary: &str,
    confidence: f64,
    observation_id: &str,
    graph_node_id: Option<&str>,
) -> Result<(), ReviewMirrorError> {
    crate::workflows::review_mirror::ensure_project_link_candidate_review_item(
        pool,
        project_id,
        target_kind,
        target_id,
        title,
        summary,
        confidence,
        observation_id,
        graph_node_id,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn sync_project_link_review_state_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: &str,
    target_kind: ProjectLinkTargetKind,
    target_id: &str,
    review_state: ProjectLinkReviewState,
    title: &str,
    summary: &str,
    confidence: f64,
    observation_id: &str,
) -> Result<(), ReviewMirrorError> {
    crate::workflows::review_mirror::sync_project_link_review_state_in_transaction(
        transaction,
        project_id,
        target_kind,
        target_id,
        review_state,
        title,
        summary,
        confidence,
        observation_id,
    )
    .await
}
```

### `backend/src/application/provider_runtime_contracts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/provider_runtime_contracts.rs`
- Size bytes / Размер в байтах: `10546`
- Included characters / Включено символов: `10546`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use sqlx::PgPool;

pub(crate) use crate::integrations::telegram::client::commands::{
    list_commands_filtered, list_commands_filtered as list_telegram_commands_filtered,
};
pub(crate) use crate::integrations::telegram::client::models::messages::{
    TelegramCommandKind, TelegramCommandListResponse, TelegramDeleteRequest, TelegramEditRequest,
    TelegramForwardChainResponse, TelegramForwardRequest, TelegramLifecycleResponse,
    TelegramManualSendRequest, TelegramManualSendResponse, TelegramMessage,
    TelegramMessageTombstoneListResponse, TelegramMessageVersionListResponse, TelegramPinRequest,
    TelegramProviderWriteCommand, TelegramReactionListResponse, TelegramReactionRequest,
    TelegramReactionResponse, TelegramReplyChainResponse, TelegramReplyRequest,
    TelegramRestoreVisibilityRequest,
};
pub(crate) use crate::integrations::telegram::client::topics::{
    get_topic as get_telegram_topic, list_topic_message_ids as list_telegram_topic_message_ids,
    list_topics as list_telegram_topics, search_topics as search_telegram_topics_projection,
};
pub(crate) use crate::integrations::telegram::client::{
    NewTelegramMessage, ProviderCommunicationMessage, TelegramAccount,
    TelegramAccountLifecycleResponse, TelegramAccountListResponse, TelegramAccountSetupRequest,
    TelegramAccountSetupResponse, TelegramAttachmentAnchor, TelegramAttachmentDownloadStateUpdate,
    TelegramChat, TelegramChatGroupFilter, TelegramChatGroupFilterListResponse, TelegramChatMember,
    TelegramError, TelegramLiveAccountSetupRequest, TelegramMessageIngestResult,
    TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest, TelegramQrLoginStatusResponse,
    TelegramSecretVault, TelegramTopic, TelegramTopicCloseRequest, TelegramTopicCreateRequest,
    TelegramTopicLifecycleResponse, TelegramTopicListResponse, ensure_telegram_account_active,
    telegram_chat_id,
};
pub(crate) use crate::integrations::telegram::runtime::{
    TelegramChatSyncRequest, TelegramChatSyncResponse, TelegramHistorySyncRequest,
    TelegramHistorySyncResponse, TelegramMediaDownloadRequest, TelegramMediaDownloadResponse,
    TelegramMediaSendType, TelegramRuntimeRestartRequest, TelegramRuntimeStartRequest,
    TelegramRuntimeStatus, TelegramRuntimeStopRequest,
};
pub(crate) mod qr_login {
    pub(crate) use crate::integrations::telegram::tdjson::{
        cancel_qr_login, start_qr_login, submit_qr_login_password,
    };
}
pub(crate) mod lifecycle {
    pub(crate) use crate::integrations::telegram::client::lifecycle::*;
}
pub(crate) mod models {
    pub(crate) use crate::integrations::telegram::client::models::*;

    pub(crate) mod messages {
        pub(crate) use crate::integrations::telegram::client::models::messages::*;
    }
}
pub(crate) use crate::integrations::telegram::client::TelegramStore as TelegramProviderRuntimeStore;
pub(crate) use crate::integrations::whatsapp::client::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebCallIngestResult, WhatsappWebDeliveryState,
    WhatsappWebDialogIngestResult, WhatsappWebError, WhatsappWebMediaIngestResult,
    WhatsappWebMessage, WhatsappWebMessageDeleteIngestResult, WhatsappWebMessageIngestResult,
    WhatsappWebMessageUpdateIngestResult, WhatsappWebParticipantIngestResult,
    WhatsappWebPresenceIngestResult, WhatsappWebReactionIngestResult,
    WhatsappWebReceiptIngestResult, WhatsappWebRuntimeEventIngestResult, WhatsappWebSession,
    WhatsappWebStatusDeleteIngestResult, WhatsappWebStatusIngestResult,
    WhatsappWebStatusViewIngestResult,
};
pub(crate) use crate::integrations::whatsapp::runtime::{
    WhatsAppAuthorizedSessionCredentialWrite, WhatsAppCommandDeadLetterRequest,
    WhatsAppConversationCommandRequest, WhatsAppCredentialBinding, WhatsAppDeleteRequest,
    WhatsAppEditRequest, WhatsAppForwardRequest, WhatsAppMediaDownloadRequest,
    WhatsAppMediaUploadRequest, WhatsAppPairCodeSession, WhatsAppPairCodeStartRequest,
    WhatsAppProviderCommand, WhatsAppProviderCommandListResponse, WhatsAppProviderCommandResponse,
    WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape, WhatsAppQrLinkSession,
    WhatsAppQrLinkStartRequest, WhatsAppReactionRequest, WhatsAppReplyRequest,
    WhatsAppRuntimeHealth, WhatsAppRuntimeRelinkRequest, WhatsAppRuntimeRemoveRequest,
    WhatsAppRuntimeRemoveResponse, WhatsAppRuntimeRevokeRequest, WhatsAppRuntimeStartRequest,
    WhatsAppRuntimeStatus, WhatsAppRuntimeStopRequest, WhatsAppStatusPublishRequest,
    WhatsAppTextSendRequest, WhatsAppVoiceNoteSendRequest,
    whatsapp_business_cloud_access_token_secret_ref, whatsapp_business_cloud_app_secret_ref,
    whatsapp_business_cloud_runtime, whatsapp_business_cloud_webhook_verify_token_ref,
    whatsapp_native_md_runtime, whatsapp_provider_runtime_mux, whatsapp_web_companion_runtime,
};
pub(crate) use crate::integrations::zoom::client::ZoomStore as ZoomProviderRuntimeStore;
pub(crate) use crate::integrations::zoom::client::{
    ZoomAccount, ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomError, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomOAuthCompleteRequest,
    ZoomOAuthPendingGrant, ZoomOAuthStartRequest, ZoomOAuthStartResponse,
    ZoomRecordingImportAuditResponse, ZoomRecordingImportRemoveRequest,
    ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingSyncRequest, ZoomRecordingSyncResult,
    ZoomRetentionCleanupItem, ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse,
    ZoomRuntimeRemoveRequest, ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest,
    ZoomRuntimeStatus, ZoomRuntimeStopRequest, ZoomServerToServerAuthorizeRequest,
    ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult, ZoomTokenRefreshRequest,
    ZoomTokenRefreshResult, ZoomTranscriptFileImportRequest, ZoomTranscriptFileImportResult,
    ZoomTranscriptIngestResult, ZoomTranscriptObservationRequest, ZoomWebhookSubscription,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult,
};

pub(crate) type WhatsAppProviderRuntimeRef = Arc<dyn WhatsAppProviderRuntime>;

pub(crate) fn telegram_provider_runtime_store(pool: PgPool) -> TelegramProviderRuntimeStore {
    TelegramProviderRuntimeStore::new(
        pool.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::messages::ProviderChannelMessageStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationIngestionStore::new(pool.clone()),
        ),
        Arc::new(
            crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool,
            ),
        ),
    )
}

pub(crate) fn whatsapp_provider_runtime(pool: PgPool) -> WhatsAppProviderRuntimeRef {
    let provider_account_store = Arc::new(
        crate::domains::communications::core::CommunicationProviderAccountStore::new(pool.clone()),
    );
    let provider_secret_binding_store = Arc::new(
        crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
            pool.clone(),
        ),
    );
    let provider_channel_message_store = Arc::new(
        crate::domains::communications::messages::ProviderChannelMessageStore::new(pool.clone()),
    );
    let whatsapp_runtime_event_sink = Arc::new(
        crate::application::WhatsappRuntimeSignalIngestService::new(pool.clone()),
    );
    let web_companion_runtime = whatsapp_web_companion_runtime(
        pool.clone(),
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        provider_channel_message_store.clone(),
    );
    let native_md_runtime = whatsapp_native_md_runtime(
        pool.clone(),
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        provider_channel_message_store.clone(),
        whatsapp_runtime_event_sink,
    );
    let business_cloud_runtime = whatsapp_business_cloud_runtime(
        pool,
        provider_account_store.clone(),
        provider_secret_binding_store.clone(),
        provider_channel_message_store.clone(),
    );
    whatsapp_provider_runtime_mux(
        provider_account_store,
        web_companion_runtime,
        native_md_runtime,
        business_cloud_runtime,
    )
}

pub(crate) fn zoom_provider_runtime_store(
    pool: PgPool,
    event_bus: crate::platform::events::EventBus,
) -> ZoomProviderRuntimeStore {
    ZoomProviderRuntimeStore::new(
        pool.clone(),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::core::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::storage::CommunicationStorageStore::new(pool.clone()),
        ),
        crate::platform::calls::CallIntelligenceStore::new(pool.clone()),
        crate::platform::events::EventStore::new(pool),
        event_bus,
    )
}

pub(crate) use crate::integrations::yandex_telemost::client::{
    YandexTelemostError, YandexTelemostRetentionCleanupRequest,
    YandexTelemostRetentionCleanupResponse,
    YandexTelemostStore as YandexTelemostProviderRuntimeStore,
    YandexTelemostTranscriptBridgeRequest, YandexTelemostTranscriptBridgeResponse,
};
```

### `backend/src/application/provider_runtime_services.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/provider_runtime_services.rs`
- Size bytes / Размер в байтах: `88258`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::application::provider_runtime_contracts::{
    TelegramAccount, TelegramAccountSetupRequest, TelegramAccountSetupResponse,
    TelegramAttachmentDownloadStateUpdate, TelegramChat, TelegramChatMember,
    TelegramCommandListResponse, TelegramError, TelegramLiveAccountSetupRequest, TelegramMessage,
    TelegramProviderRuntimeStore, TelegramProviderWriteCommand, TelegramSecretVault, TelegramTopic,
    TelegramTopicLifecycleResponse, TelegramTopicListResponse,
    WhatsAppAuthorizedSessionCredentialWrite, WhatsAppConversationCommandRequest,
    WhatsAppCredentialBinding, WhatsAppDeleteRequest, WhatsAppEditRequest, WhatsAppForwardRequest,
    WhatsAppMediaDownloadRequest, WhatsAppMediaUploadRequest, WhatsAppPairCodeSession,
    WhatsAppPairCodeStartRequest, WhatsAppProviderCommand, WhatsAppProviderCommandListResponse,
    WhatsAppProviderCommandResponse, WhatsAppProviderRuntimeRef, WhatsAppProviderRuntimeShape,
    WhatsAppQrLinkSession, WhatsAppQrLinkStartRequest, WhatsAppReactionRequest,
    WhatsAppReplyRequest, WhatsAppRuntimeHealth, WhatsAppRuntimeRelinkRequest,
    WhatsAppRuntimeRemoveRequest, WhatsAppRuntimeRemoveResponse, WhatsAppRuntimeRevokeRequest,
    WhatsAppRuntimeStartRequest, WhatsAppRuntimeStatus, WhatsAppRuntimeStopRequest,
    WhatsAppStatusPublishRequest, WhatsAppTextSendRequest, WhatsAppVoiceNoteSendRequest,
    WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebError, WhatsappWebMessage, WhatsappWebSession,
    ZoomAccountListResponse, ZoomAccountSetupRequest, ZoomAccountSetupResponse,
    ZoomAuditEventResponse, ZoomAuthorizationResult, ZoomError, ZoomLiveAccountSetupRequest,
    ZoomMeetingIngestResult, ZoomMeetingObservationRequest, ZoomOAuthPendingGrant,
    ZoomOAuthStartRequest, ZoomProviderRuntimeStore, ZoomRecordingImportAuditResponse,
    ZoomRecordingImportRemoveRequest, ZoomRecordingImportRemoveResponse, ZoomRecordingIngestResult,
    ZoomRecordingMediaDownloadRequest, ZoomRecordingMediaImportResult,
    ZoomRecordingObservationRequest, ZoomRecordingSyncRequest, ZoomRecordingSyncResult,
    ZoomRetentionCleanupRequest, ZoomRetentionCleanupResponse, ZoomRuntimeRemoveRequest,
    ZoomRuntimeRemoveResponse, ZoomRuntimeStartRequest, ZoomRuntimeStatus, ZoomRuntimeStopRequest,
    ZoomServerToServerAuthorizeRequest, ZoomTokenMaintenanceRequest, ZoomTokenMaintenanceResult,
    ZoomTokenRefreshRequest, ZoomTokenRefreshResult, ZoomTranscriptFileImportRequest,
    ZoomTranscriptFileImportResult, ZoomTranscriptIngestResult, ZoomTranscriptObservationRequest,
    ZoomWebhookSubscriptionReconcileRequest, ZoomWebhookSubscriptionReconcileResult,
    ZoomWebhookSubscriptionRemoveRequest, ZoomWebhookSubscriptionRemoveResult,
    ZoomWebhookSubscriptionStatusRequest, ZoomWebhookSubscriptionStatusResult, get_telegram_topic,
    lifecycle, list_telegram_commands_filtered, list_telegram_topic_message_ids,
    list_telegram_topics, search_telegram_topics_projection,
};
use crate::platform::communications::ProviderAccount;
use crate::platform::events::EventBus;
use crate::platform::secrets::SecretReferenceStore;
use crate::vault::HostVault;

#[derive(Clone)]
pub(crate) struct TelegramProviderRuntimeApplicationService {
    store: TelegramProviderRuntimeStore,
}

#[derive(Clone)]
pub(crate) struct ZoomProviderRuntimeApplicationService {
    store: ZoomProviderRuntimeStore,
}

#[derive(Clone)]
pub(crate) struct YandexTelemostProviderRuntimeApplicationService {
    store: crate::application::provider_runtime_contracts::YandexTelemostProviderRuntimeStore,
}

impl ZoomProviderRuntimeApplicationService {
    pub(crate) fn new(store: ZoomProviderRuntimeStore) -> Self {
        Self { store }
    }

    pub(crate) async fn setup_fixture_account(
        &self,
        request: &ZoomAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        self.store.setup_fixture_account(request).await
    }

    pub(crate) async fn setup_live_blocked_account(
        &self,
        request: &ZoomLiveAccountSetupRequest,
    ) -> Result<ZoomAccountSetupResponse, ZoomError> {
        self.store.setup_live_blocked_account(request).await
    }

    pub(crate) async fn start_oauth(
        &self,
        request: &ZoomOAuthStartRequest,
    ) -> Result<ZoomOAuthPendingGrant, ZoomError> {
        self.store.start_oauth(request).await
    }

    pub(crate) async fn complete_oauth(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        pending: ZoomOAuthPendingGrant,
        authorization_code: &str,
        external_account_id: Option<&str>,
    ) -> Result<ZoomAuthorizationResult, ZoomError> {
        self.store
            .complete_oauth(
                secret_store,
                vault,
                pending,
                authorization_code,
                external_account_id,
            )
            .await
    }

    pub(crate) async fn authorize_server_to_server(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomServerToServerAuthorizeRequest,
    ) -> Result<ZoomAuthorizationResult, ZoomError> {
        self.store
            .authorize_server_to_server(secret_store, vault, request)
            .await
    }

    pub(crate) async fn refresh_token(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomTokenRefreshRequest,
    ) -> Result<ZoomTokenRefreshResult, ZoomError> {
        self.store.refresh_token(secret_store, vault, request).await
    }

    pub(crate) async fn maintain_tokens(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomTokenMaintenanceRequest,
    ) -> Result<ZoomTokenMaintenanceResult, ZoomError> {
        self.store
            .maintain_tokens(secret_store, vault, request)
            .await
    }

    pub(crate) async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<ZoomAccountListResponse, ZoomError> {
        self.store.list_accounts(include_removed).await
    }

    pub(crate) async fn runtime_status(
        &self,
        account_id: &str,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        self.store.runtime_status(account_id).await
    }

    pub(crate) async fn start_runtime(
        &self,
        request: &ZoomRuntimeStartRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        self.store.start_runtime(request).await
    }

    pub(crate) async fn stop_runtime(
        &self,
        request: &ZoomRuntimeStopRequest,
    ) -> Result<ZoomRuntimeStatus, ZoomError> {
        self.store.stop_runtime(request).await
    }

    pub(crate) async fn remove_runtime(
        &self,
        request: &ZoomRuntimeRemoveRequest,
    ) -> Result<ZoomRuntimeRemoveResponse, ZoomError> {
        self.store.remove_runtime(request).await
    }

    pub(crate) async fn observe_meeting(
        &self,
        request: &ZoomMeetingObservationRequest,
    ) -> Result<ZoomMeetingIngestResult, ZoomError> {
        self.store.observe_meeting(request).await
    }

    pub(crate) async fn observe_recording(
        &self,
        request: &ZoomRecordingObservationRequest,
    ) -> Result<ZoomRecordingIngestResult, ZoomError> {
        self.store.observe_recording(request).await
    }

    pub(crate) async fn import_recording_media_download(
        &self,
        request: &ZoomRecordingMediaDownloadRequest,
        bearer_token: Option<&str>,
    ) -> Result<ZoomRecordingMediaImportResult, ZoomError> {
        self.store
            .import_recording_media_download(request, bearer_token)
            .await
    }

    pub(crate) async fn list_recording_imports(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<ZoomRecordingImportAuditResponse, ZoomError> {
        self.store.list_recording_imports(account_id, limit).await
    }

    pub(crate) async fn remove_recording_import(
        &self,
        account_id: &str,
        attachment_id: &str,
        request: &ZoomRecordingImportRemoveRequest,
    ) -> Result<ZoomRecordingImportRemoveResponse, ZoomError> {
        self.store
            .remove_recording_import(account_id, attachment_id, request)
            .await
    }

    pub(crate) async fn list_audit_events(
        &self,
        account_id: &str,
        limit: i64,
    ) -> Result<ZoomAuditEventResponse, ZoomError> {
        self.store.list_audit_events(account_id, limit).await
    }

    pub(crate) async fn cleanup_retention(
        &self,
        account_id: &str,
        request: &ZoomRetentionCleanupRequest,
    ) -> Result<ZoomRetentionCleanupResponse, ZoomError> {
        self.store.cleanup_retention(account_id, request).await
    }

    pub(crate) async fn observe_transcript(
        &self,
        request: &ZoomTranscriptObservationRequest,
    ) -> Result<ZoomTranscriptIngestResult, ZoomError> {
        self.store.observe_transcript(request).await
    }

    pub(crate) async fn import_transcript_file(
        &self,
        request: &ZoomTranscriptFileImportRequest,
    ) -> Result<ZoomTranscriptFileImportResult, ZoomError> {
        self.store.import_transcript_file(request).await
    }

    pub(crate) async fn import_transcript_file_download(
        &self,
        request: &ZoomTranscriptFileImportRequest,
        download_url: &str,
        download_token: Option<&str>,
    ) -> Result<ZoomTranscriptFileImportResult, ZoomError> {
        self.store
            .import_transcript_file_download(request, download_url, download_token)
            .await
    }

    pub(crate) async fn sync_recordings(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomRecordingSyncRequest,
        allow_remote_recording_downloads: bool,
        allow_remote_transcript_downloads: bool,
    ) -> Result<ZoomRecordingSyncResult, ZoomError> {
        self.store
            .sync_recordings(
                secret_store,
                vault,
                request,
                allow_remote_recording_downloads,
                allow_remote_transcript_downloads,
            )
            .await
    }

    pub(crate) async fn webhook_subscription_status(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionStatusRequest,
    ) -> Result<ZoomWebhookSubscriptionStatusResult, ZoomError> {
        self.store
            .webhook_subscription_status(secret_store, vault, request)
            .await
    }

    pub(crate) async fn reconcile_webhook_subscription(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionReconcileRequest,
    ) -> Result<ZoomWebhookSubscriptionReconcileResult, ZoomError> {
        self.store
            .reconcile_webhook_subscription(secret_store, vault, request)
            .await
    }

    pub(crate) async fn remove_webhook_subscription(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &HostVault,
        request: &ZoomWebhookSubscriptionRemoveRequest,
    ) -> Result<ZoomWebhookSubscriptionRemoveResult, ZoomError> {
        self.store
            .remove_webhook_subscription(secret_store, vault, request)
            .await
    }
}

impl YandexTelemostProviderRuntimeApplicationService {
    pub(crate) fn new(
        store: crate::application::provider_runtime_contracts::YandexTelemostProviderRuntimeStore,
    ) -> Self {
        Self { store }
    }

    pub(crate) async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<
        crate::integrations::yandex_telemost::client::YandexTelemostAccountListResponse,
        crate::integrations::yandex_telemost::client::YandexTelemostError,
    > {
        self.store.list_accounts(includ
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/application/realtime_conversation_transcript_execution.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/realtime_conversation_transcript_execution.rs`
- Size bytes / Размер в байтах: `740`
- Included characters / Включено символов: `740`
- Truncated / Обрезано: `no`

```rust
use crate::application::provider_runtime_contracts::{
    YandexTelemostError, YandexTelemostTranscriptBridgeRequest,
    YandexTelemostTranscriptBridgeResponse,
};
use crate::platform::events::{EventBus, EventStore};

pub(crate) use crate::workflows::realtime_conversation_transcript_execution::*;

pub(crate) async fn complete_realtime_conversation_transcript_bridge(
    event_store: &EventStore,
    event_bus: Option<&EventBus>,
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<YandexTelemostTranscriptBridgeResponse, YandexTelemostError> {
    crate::integrations::yandex_telemost::runtime_bridge::complete_yandex_telemost_transcript_bridge(
        event_store,
        event_bus,
        request,
    )
    .await
}
```

### `backend/src/application/realtime_conversation_transcript_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/application/realtime_conversation_transcript_projection.rs`
- Size bytes / Размер в байтах: `81`
- Included characters / Включено символов: `81`
- Truncated / Обрезано: `no`

```rust
pub(crate) use crate::workflows::realtime_conversation_transcript_projection::*;
```

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

- Chunk ID / ID чанка: `054-source-backend-part-034`
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

### `backend/src/domains/tasks/core/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/errors.rs`
- Size bytes / Размер в байтах: `571`
- Included characters / Включено символов: `571`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::domains::relationships::RelationshipStoreError;
use crate::engines::context_packs::ContextPackStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum TaskCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ContextPack(#[from] ContextPackStoreError),
    #[error(transparent)]
    Relationship(#[from] RelationshipStoreError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/tasks/core/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/evidence.rs`
- Size bytes / Размер в байтах: `4247`
- Included characters / Включено символов: `4247`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::{TaskCoreError, materialize_task_entity_link_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskEvidence {
    pub id: String,
    pub task_id: String,
    pub source_type: String,
    pub source_id: String,
    pub quote: Option<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskEvidenceStore {
    pool: PgPool,
}

impl TaskEvidenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, task_id: &str) -> Result<Vec<TaskEvidence>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, task_id, source_type, source_id, quote,
                   confidence::float8 AS confidence, created_at
            FROM task_evidence
            WHERE task_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskEvidence {
                    id: row.try_get("id")?,
                    task_id: row.try_get("task_id")?,
                    source_type: row.try_get("source_type")?,
                    source_id: row.try_get("source_id")?,
                    quote: row.try_get("quote")?,
                    confidence: row.try_get("confidence")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        task_id: &str,
        source_type: &str,
        source_id: &str,
        quote: Option<&str>,
        confidence: Option<f64>,
    ) -> Result<TaskEvidence, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let evidence = Self::add_in_transaction(
            &mut transaction,
            task_id,
            source_type,
            source_id,
            quote,
            confidence,
        )
        .await?;

        if evidence.source_type == "observation" {
            materialize_task_entity_link_in_transaction(
                &mut transaction,
                Some(&evidence.source_id),
                "task_evidence",
                &evidence.id,
                None,
                None,
                Some(json!({
                    "task_id": task_id,
                })),
            )
            .await?;
            materialize_task_entity_link_in_transaction(
                &mut transaction,
                Some(&evidence.source_id),
                "task",
                task_id,
                Some("supports"),
                Some(evidence.confidence),
                Some(json!({
                    "task_evidence_id": evidence.id,
                })),
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(evidence)
    }

    async fn add_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        task_id: &str,
        source_type: &str,
        source_id: &str,
        quote: Option<&str>,
        confidence: Option<f64>,
    ) -> Result<TaskEvidence, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_evidence (task_id, source_type, source_id, quote, confidence)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id::text, task_id, source_type, source_id, quote,
                      confidence::float8 AS confidence, created_at
            "#,
        )
        .bind(task_id)
        .bind(source_type)
        .bind(source_id)
        .bind(quote)
        .bind(confidence.unwrap_or(1.0))
        .fetch_one(&mut **transaction)
        .await?;

        Ok(TaskEvidence {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            source_type: row.try_get("source_type")?,
            source_id: row.try_get("source_id")?,
            quote: row.try_get("quote")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
```

### `backend/src/domains/tasks/core/external_identities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/external_identities.rs`
- Size bytes / Размер в байтах: `2269`
- Included characters / Включено символов: `2269`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::TaskCoreError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalTaskIdentity {
    pub id: String,
    pub task_id: String,
    pub provider: String,
    pub account_id: Option<String>,
    pub external_project_id: Option<String>,
    pub external_task_id: Option<String>,
    pub external_url: Option<String>,
    pub external_status: Option<String>,
    pub sync_status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ExternalTaskIdentityStore {
    pool: PgPool,
}

impl ExternalTaskIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, task_id: &str) -> Result<Vec<ExternalTaskIdentity>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, task_id, provider, account_id, external_project_id,
                   external_task_id, external_url, external_status, sync_status,
                   last_synced_at, created_at, updated_at
            FROM external_task_identities
            WHERE task_id = $1
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ExternalTaskIdentity {
                    id: row.try_get("id")?,
                    task_id: row.try_get("task_id")?,
                    provider: row.try_get("provider")?,
                    account_id: row.try_get("account_id")?,
                    external_project_id: row.try_get("external_project_id")?,
                    external_task_id: row.try_get("external_task_id")?,
                    external_url: row.try_get("external_url")?,
                    external_status: row.try_get("external_status")?,
                    sync_status: row.try_get("sync_status")?,
                    last_synced_at: row.try_get("last_synced_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
```

### `backend/src/domains/tasks/core/obligation_links.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/obligation_links.rs`
- Size bytes / Размер в байтах: `1298`
- Included characters / Включено символов: `1298`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::TaskCoreError;

#[derive(Clone)]
pub struct ObligationTaskLinkStore {
    pool: PgPool,
}

impl ObligationTaskLinkStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn link_fulfillment_task(
        &self,
        obligation_id: &str,
        task_id: &str,
    ) -> Result<(), TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        Self::link_fulfillment_task_in_transaction(&mut transaction, obligation_id, task_id)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn link_fulfillment_task_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        obligation_id: &str,
        task_id: &str,
    ) -> Result<(), TaskCoreError> {
        sqlx::query(
            r#"
            INSERT INTO obligation_task_links (
                obligation_id,
                task_id,
                link_kind
            )
            VALUES ($1, $2, 'fulfillment_task')
            ON CONFLICT (obligation_id, task_id, link_kind) DO NOTHING
            "#,
        )
        .bind(obligation_id)
        .bind(task_id)
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }
}
```

### `backend/src/domains/tasks/core/observation_links.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/observation_links.rs`
- Size bytes / Размер в байтах: `1556`
- Included characters / Включено символов: `1556`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::link_domain_entity_in_transaction;

use super::TaskCoreError;

pub(crate) async fn materialize_task_observation_link_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    relationship_kind: Option<&str>,
    task_id: &str,
    metadata: Option<Value>,
) -> Result<(), TaskCoreError> {
    let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "tasks",
        "task",
        task_id.to_owned(),
        relationship_kind.filter(|value| !value.is_empty()),
        None,
        metadata,
    )
    .await?;
    Ok(())
}

pub(crate) async fn materialize_task_entity_link_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: Option<&str>,
    entity_kind: &str,
    entity_id: &str,
    relationship_kind: Option<&str>,
    confidence: Option<f64>,
    metadata: Option<Value>,
) -> Result<(), TaskCoreError> {
    let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "tasks",
        entity_kind,
        entity_id.to_owned(),
        relationship_kind.filter(|value| !value.is_empty()),
        confidence,
        metadata,
    )
    .await?;
    Ok(())
}
```

### `backend/src/domains/tasks/core/provider_store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/provider_store.rs`
- Size bytes / Размер в байтах: `5216`
- Included characters / Включено символов: `5216`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::TaskCoreError;
use super::providers::TaskProviderAccount;
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, link_domain_entity_in_transaction,
};

pub struct TaskProviderStore {
    pool: PgPool,
}

impl TaskProviderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<TaskProviderAccount>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT account_id, provider, account_name, credentials_reference,
                   sync_mode, capabilities, created_at, updated_at
            FROM task_provider_accounts
            ORDER BY provider, account_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_task_provider_account).collect()
    }

    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        self.create_with_origin(
            provider,
            account_name,
            ObservationOriginKind::LocalRuntime,
            "tasks_api.post_task_provider",
        )
        .await
    }

    pub async fn create_with_origin(
        &self,
        provider: &str,
        account_name: &str,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        let account_id = next_id("tprov");
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "TASK_PROVIDER_ACCOUNT",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id,
                    "provider": provider,
                    "account_name": account_name,
                    "action": "create_task_provider_account",
                }),
                format!("task-provider://{account_id}"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "create_task_provider_account",
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO task_provider_accounts (account_id, provider, account_name)
            VALUES ($1, $2, $3)
            RETURNING account_id, provider, account_name, credentials_reference,
                      sync_mode, capabilities, created_at, updated_at
            "#,
        )
        .bind(&account_id)
        .bind(provider)
        .bind(account_name)
        .fetch_one(&mut *transaction)
        .await?;
        link_domain_owned_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "tasks",
            "task_provider_account",
            account_id.clone(),
            "create",
            json!({
                "provider": provider,
                "account_name": account_name,
            }),
            None,
        )
        .await?;
        transaction.commit().await?;

        row_to_task_provider_account(row)
    }
}

fn row_to_task_provider_account(row: PgRow) -> Result<TaskProviderAccount, TaskCoreError> {
    Ok(TaskProviderAccount {
        account_id: row.try_get("account_id")?,
        provider: row.try_get("provider")?,
        account_name: row.try_get("account_name")?,
        credentials_reference: row.try_get("credentials_reference")?,
        sync_mode: row.try_get("sync_mode")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
async fn link_domain_owned_entity_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    base_metadata: serde_json::Value,
    extra_metadata: Option<serde_json::Value>,
) -> Result<(), crate::platform::observations::ObservationStoreError> {
    let metadata = match extra_metadata {
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
    };

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        domain,
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}

fn next_id(prefix: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}:v1:{ts:x}")
}
```

### `backend/src/domains/tasks/core/providers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/providers.rs`
- Size bytes / Размер в байтах: `430`
- Included characters / Включено символов: `430`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskProviderAccount {
    pub account_id: String,
    pub provider: String,
    pub account_name: String,
    pub credentials_reference: Option<String>,
    pub sync_mode: String,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### `backend/src/domains/tasks/core/relations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/relations.rs`
- Size bytes / Размер в байтах: `7841`
- Included characters / Включено символов: `7841`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};

use super::{TaskCoreError, materialize_task_entity_link_in_transaction};
use crate::domains::relationships::{
    NewRelationship, NewRelationshipEvidence, RelationshipEntityKind, RelationshipReviewPort,
    RelationshipReviewState,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

const TASK_RELATIONSHIP_EVIDENCE_EXCERPT: &str =
    "Task relation was recorded through compatibility task relation data.";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRelation {
    pub id: String,
    pub task_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub relation_type: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskRelationStore {
    pool: PgPool,
}

impl TaskRelationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, task_id: &str) -> Result<Vec<TaskRelation>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, task_id, entity_type, entity_id, relation_type, source,
                   confidence::float8 AS confidence, created_at
            FROM task_relations
            WHERE task_id = $1
            ORDER BY relation_type
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskRelation {
                    id: row.try_get("id")?,
                    task_id: row.try_get("task_id")?,
                    entity_type: row.try_get("entity_type")?,
                    entity_id: row.try_get("entity_id")?,
                    relation_type: row.try_get("relation_type")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn link(
        &self,
        task_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
        source: &str,
    ) -> Result<TaskRelation, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO task_relations (task_id, entity_type, entity_id, relation_type, source)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT DO NOTHING
            RETURNING id::text, task_id, entity_type, entity_id, relation_type,
                      source, confidence::float8 AS confidence, created_at
            "#,
        )
        .bind(task_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(relation_type)
        .bind(source)
        .fetch_one(&mut *transaction)
        .await?;
        let relation = TaskRelation {
            id: row.try_get("id")?,
            task_id: row.try_get("task_id")?,
            entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?,
            relation_type: row.try_get("relation_type")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            created_at: row.try_get("created_at")?,
        };

        Self::materialize_observation_link_in_transaction(&mut transaction, &relation).await?;
        Self::materialize_relationship_in_transaction(&mut transaction, &relation).await?;
        transaction.commit().await?;

        Ok(relation)
    }

    async fn materialize_observation_link_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relation: &TaskRelation,
    ) -> Result<(), TaskCoreError> {
        let Some(observation_id) = relation
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };

        materialize_task_entity_link_in_transaction(
            transaction,
            Some(observation_id),
            "task_relation",
            &relation.id,
            None,
            None,
            Some(json!({
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
            })),
        )
        .await?;

        Ok(())
    }

    async fn materialize_relationship_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        relation: &TaskRelation,
    ) -> Result<(), TaskCoreError> {
        let Ok(target_entity_kind) = RelationshipEntityKind::parse(&relation.entity_type) else {
            return Ok(());
        };

        let observation_id = if let Some(observation_id) = relation
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        {
            observation_id.to_owned()
        } else {
            let observation = ObservationStore::capture_in_transaction(
                transaction,
                &Self::relation_observation(relation),
            )
            .await?;
            observation.observation_id
        };

        let relationship = NewRelationship {
            source_entity_kind: RelationshipEntityKind::Task,
            source_entity_id: relation.task_id.clone(),
            target_entity_kind,
            target_entity_id: relation.entity_id.clone(),
            relationship_type: relation.relation_type.clone(),
            trust_score: relation.confidence,
            strength_score: relation.confidence,
            confidence: relation.confidence,
            review_state: RelationshipReviewState::UserConfirmed,
            valid_from: None,
            valid_to: None,
            metadata: json!({
                "compatibility_table": "task_relations",
                "compatibility_record_id": relation.id,
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
                "source": relation.source,
            }),
        };
        let evidence = NewRelationshipEvidence::observation(observation_id)
            .excerpt(TASK_RELATIONSHIP_EVIDENCE_EXCERPT)
            .metadata(json!({
                "compatibility_table": "task_relations",
                "compatibility_record_id": relation.id,
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
            }));

        RelationshipReviewPort::upsert_with_evidence_in_transaction(
            transaction,
            &relationship,
            &[evidence],
        )
        .await?;

        Ok(())
    }

    fn relation_observation(relation: &TaskRelation) -> NewObservation {
        let origin_kind = ObservationOriginKind::parse(&relation.source)
            .unwrap_or(ObservationOriginKind::LocalRuntime);
        NewObservation::new(
            "TASK_MUTATION",
            origin_kind,
            relation.created_at,
            json!({
                "task_id": relation.task_id,
                "entity_type": relation.entity_type,
                "entity_id": relation.entity_id,
                "relation_type": relation.relation_type,
                "source": relation.source,
                "compatibility_record_id": relation.id,
            }),
            format!("task://{}/relation/{}", relation.task_id, relation.id),
        )
        .provenance(json!({
            "captured_by": "tasks.core.relations",
            "operation": "materialize_relationship",
            "source": relation.source,
        }))
    }
}
```

### `backend/src/domains/tasks/core/subtasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/core/subtasks.rs`
- Size bytes / Размер в байтах: `3958`
- Included characters / Включено символов: `3958`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::{TaskCoreError, materialize_task_entity_link_in_transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskSubtask {
    pub id: String,
    pub parent_task_id: String,
    pub child_task_id: String,
    pub sort_order: i32,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskSubtaskStore {
    pool: PgPool,
}

impl TaskSubtaskStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, parent_id: &str) -> Result<Vec<TaskSubtask>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, parent_task_id, child_task_id, sort_order, source, created_at
            FROM task_subtasks
            WHERE parent_task_id = $1
            ORDER BY sort_order
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(TaskSubtask {
                    id: row.try_get("id")?,
                    parent_task_id: row.try_get("parent_task_id")?,
                    child_task_id: row.try_get("child_task_id")?,
                    sort_order: row.try_get("sort_order")?,
                    source: row.try_get("source")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        parent_id: &str,
        child_id: &str,
        order: i32,
    ) -> Result<TaskSubtask, TaskCoreError> {
        self.add_with_source(parent_id, child_id, order, "manual")
            .await
    }

    pub async fn add_with_source(
        &self,
        parent_id: &str,
        child_id: &str,
        order: i32,
        source: &str,
    ) -> Result<TaskSubtask, TaskCoreError> {
        let mut transaction = self.pool.begin().await?;
        let subtask =
            Self::add_in_transaction(&mut transaction, parent_id, child_id, order, source).await?;

        if let Some(observation_id) = subtask
            .source
            .strip_prefix("observation:")
            .filter(|value| !value.is_empty())
        {
            materialize_task_entity_link_in_transaction(
                &mut transaction,
                Some(observation_id),
                "task_subtask",
                &subtask.id,
                None,
                None,
                Some(json!({
                    "parent_task_id": parent_id,
                    "child_task_id": child_id,
                })),
            )
            .await?;
        }

        transaction.commit().await?;
        Ok(subtask)
    }

    async fn add_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        parent_id: &str,
        child_id: &str,
        order: i32,
        source: &str,
    ) -> Result<TaskSubtask, TaskCoreError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_subtasks (parent_task_id, child_task_id, sort_order, source)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (parent_task_id, child_task_id)
            DO UPDATE SET sort_order = $3, source = $4
            RETURNING id::text, parent_task_id, child_task_id, sort_order, source, created_at
            "#,
        )
        .bind(parent_id)
        .bind(child_id)
        .bind(order)
        .bind(source)
        .fetch_one(&mut **transaction)
        .await?;

        Ok(TaskSubtask {
            id: row.try_get("id")?,
            parent_task_id: row.try_get("parent_task_id")?,
            child_task_id: row.try_get("child_task_id")?,
            sort_order: row.try_get("sort_order")?,
            source: row.try_get("source")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
```

### `backend/src/domains/tasks/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/health.rs`
- Size bytes / Размер в байтах: `5839`
- Included characters / Включено символов: `5839`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::engines::context_packs::{ContextPackKind, ContextPackStore, ContextPackStoreError};

pub struct TaskWatchtowerService;

impl TaskWatchtowerService {
    pub async fn overdue(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let now = Utc::now();
        let rows = sqlx::query("SELECT task_id, title, makosh_status, priority_score, due_at FROM tasks WHERE due_at < $1 AND makosh_status NOT IN ('done','cancelled','archived') ORDER BY priority_score DESC NULLS LAST LIMIT 30")
            .bind(now).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                    "title": r.try_get::<String,_>("title").unwrap_or_default(),
                    "status": r.try_get::<String,_>("makosh_status").unwrap_or_default(),
                    "priority": r.try_get::<Option<f64>,_>("priority_score").unwrap_or(None),
                    "due_at": r.try_get::<Option<DateTime<Utc>>,_>("due_at").unwrap_or(None),
                })
            })
            .collect();
        Ok(json!({"overdue": items}))
    }

    pub async fn waiting_too_long(pool: &PgPool, days: i64) -> Result<Value, TaskHealthError> {
        let threshold = Utc::now() - Duration::days(days);
        let rows = sqlx::query("SELECT task_id, title, waiting_reason, updated_at FROM tasks WHERE makosh_status='waiting' AND updated_at < $1 ORDER BY updated_at ASC LIMIT 20")
            .bind(threshold).fetch_all(pool).await?;
        let items: Vec<Value> = rows.iter().map(|r| json!({
            "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
            "title": r.try_get::<String,_>("title").unwrap_or_default(),
            "waiting_reason": r.try_get::<Option<String>,_>("waiting_reason").unwrap_or(None),
            "since": r.try_get::<DateTime<Utc>,_>("updated_at").ok(),
        })).collect();
        Ok(json!({"waiting_too_long": items}))
    }

    pub async fn without_context(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let rows = sqlx::query("SELECT task_id, title, makosh_status FROM tasks WHERE makosh_status NOT IN ('done','cancelled','archived') ORDER BY priority_score DESC NULLS LAST LIMIT 50")
            .fetch_all(pool)
            .await?;
        let context_store = ContextPackStore::new(pool.clone());
        let mut items = Vec::new();
        for row in rows {
            let task_id = row.try_get::<String, _>("task_id").unwrap_or_default();
            let has_context = context_store
                .exists(ContextPackKind::Task, &task_id)
                .await?;
            if has_context {
                continue;
            }
            items.push(json!({
                "task_id": task_id,
                "title": row.try_get::<String,_>("title").unwrap_or_default(),
                "status": row.try_get::<String,_>("makosh_status").unwrap_or_default(),
            }));
            if items.len() >= 20 {
                break;
            }
        }
        Ok(json!({"tasks_without_context": items}))
    }

    pub async fn stale_tasks(pool: &PgPool, days: i64) -> Result<Value, TaskHealthError> {
        let threshold = Utc::now() - Duration::days(days);
        let rows = sqlx::query("SELECT task_id, title, makosh_status, updated_at FROM tasks WHERE makosh_status NOT IN ('done','cancelled','archived') AND updated_at < $1 ORDER BY updated_at ASC LIMIT 20")
            .bind(threshold).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                    "title": r.try_get::<String,_>("title").unwrap_or_default(),
                    "status": r.try_get::<String,_>("makosh_status").unwrap_or_default(),
                    "since": r.try_get::<DateTime<Utc>,_>("updated_at").ok(),
                })
            })
            .collect();
        Ok(json!({"stale_tasks": items}))
    }

    pub async fn cycle_time(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let rows = sqlx::query("SELECT EXTRACT(EPOCH FROM (COALESCE(completed_at, now()) - created_at))/3600 as hours, makosh_status FROM tasks WHERE completed_at IS NOT NULL ORDER BY completed_at DESC LIMIT 50")
            .fetch_all(pool).await?;
        let hours: Vec<f64> = rows
            .iter()
            .filter_map(|r| r.try_get::<Option<f64>, _>("hours").unwrap_or(None))
            .collect();
        let avg = if hours.is_empty() {
            0.0
        } else {
            hours.iter().sum::<f64>() / hours.len() as f64
        };
        Ok(json!({"average_cycle_hours": avg, "completed_count": hours.len()}))
    }

    pub async fn workload(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let active = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE makosh_status IN ('new','triaged','ready','in_progress','waiting','blocked','review')")
            .fetch_one(pool).await?;
        let overdue = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE due_at < $1 AND makosh_status NOT IN ('done','cancelled','archived')")
            .bind(Utc::now()).fetch_one(pool).await?;
        Ok(json!({
            "active_count": active.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
            "overdue_count": overdue.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
        }))
    }
}

#[derive(Debug, Error)]
pub enum TaskHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ContextPack(#[from] ContextPackStoreError),
}
```

### `backend/src/domains/tasks/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/intelligence.rs`
- Size bytes / Размер в байтах: `4221`
- Included characters / Включено символов: `4215`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use thiserror::Error;

pub struct TaskIntelligenceService;

impl TaskIntelligenceService {
    pub fn calculate_priority(
        due_at: Option<DateTime<Utc>>,
        has_contact: bool,
        has_org: bool,
        has_project: bool,
        is_legal: bool,
        is_tax: bool,
        has_blockers: bool,
    ) -> f64 {
        let mut score: f64 = 0.2;
        if let Some(due) = due_at {
            let hours_left = (due - Utc::now()).num_hours();
            if hours_left <= 0 {
                score += 0.5;
            } else if hours_left <= 24 {
                score += 0.4;
            } else if hours_left <= 72 {
                score += 0.3;
            } else if hours_left <= 168 {
                score += 0.15;
            }
        }
        if is_legal || is_tax {
            score += 0.3;
        }
        if has_blockers {
            score += 0.15;
        }
        if has_contact {
            score += 0.1;
        }
        if has_org {
            score += 0.1;
        }
        if has_project {
            score += 0.05;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn calculate_risk(
        has_deadline_close: bool,
        missing_docs: bool,
        no_owner: bool,
        external_dep: bool,
        is_legal: bool,
        title: &str,
    ) -> f64 {
        let mut score: f64 = 0.1;
        if has_deadline_close {
            score += 0.3;
        }
        if missing_docs {
            score += 0.2;
        }
        if no_owner {
            score += 0.15;
        }
        if external_dep {
            score += 0.2;
        }
        if is_legal {
            score += 0.15;
        }
        let t = title.to_lowercase();
        if t.contains("urgent") || t.contains("asap") || t.contains("срочно") {
            score += 0.2;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn calculate_readiness(
        has_desc: bool,
        has_context: bool,
        has_docs: bool,
        has_deadline: bool,
        no_blockers: bool,
        contacts_resolved: bool,
    ) -> f64 {
        let mut score: f64 = 0.0;
        if has_desc {
            score += 0.2;
        }
        if has_context {
            score += 0.2;
        }
        if has_docs {
            score += 0.15;
        }
        if has_deadline {
            score += 0.15;
        }
        if no_blockers {
            score += 0.15;
        }
        if contacts_resolved {
            score += 0.15;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn detect_missing_context(
        has_desc: bool,
        has_context: bool,
        has_deadline: bool,
        has_contact: bool,
        has_project: bool,
    ) -> Vec<String> {
        let mut missing = Vec::new();
        if !has_desc {
            missing.push("No description".into());
        }
        if !has_context {
            missing.push("No context pack".into());
        }
        if !has_deadline {
            missing.push("No deadline".into());
        }
        if !has_contact {
            missing.push("No linked contact".into());
        }
        if !has_project {
            missing.push("No linked project".into());
        }
        missing
    }

    pub fn suggest_next_action(
        status: &str,
        _has_docs: bool,
        has_blockers: bool,
        waiting_reason: Option<&str>,
    ) -> String {
        match status {
            "new" | "triaged" => "Review and set priority".into(),
            "ready" => "Start working on this task".into(),
            "in_progress" => "Continue working".into(),
            "waiting" => format!("Follow up: {}", waiting_reason.unwrap_or("check status")),
            "blocked" => {
                if has_blockers {
                    "Resolve blockers first".into()
                } else {
                    "Investigate blocking reason".into()
                }
            }
            "review" => "Review and approve or request changes".into(),
            "done" => "Archive this task".into(),
            _ => "Review task context".into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum TaskIntelligenceError {
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),
}
```

### `backend/src/domains/tasks/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/mod.rs`
- Size bytes / Размер в байтах: `182`
- Included characters / Включено символов: `182`
- Truncated / Обрезано: `no`

```rust
pub mod api;
pub mod brain;
pub mod candidates;
mod command_service;
pub mod core;
pub mod health;
pub mod intelligence;
pub mod ports;
pub mod rules;
pub mod service;
pub mod sync;
```

### `backend/src/domains/tasks/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/ports.rs`
- Size bytes / Размер в байтах: `190`
- Included characters / Включено символов: `190`
- Truncated / Обрезано: `no`

```rust
pub use super::api::TaskStore as TaskCommandPort;
pub use super::candidates::TaskCandidateStore as TaskCandidatePort;
pub use super::core::ObligationTaskLinkStore as ObligationTaskLinkPort;
```

### `backend/src/domains/tasks/rules.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/rules.rs`
- Size bytes / Размер в байтах: `5104`
- Included characters / Включено символов: `5104`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskRule {
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
pub struct TaskRuleStore {
    pool: PgPool,
}
impl TaskRuleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self) -> Result<Vec<TaskRule>, TaskRuleError> {
        let rows = sqlx::query("SELECT rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at FROM task_rules ORDER BY name")
            .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskRule {
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
        desc: Option<&str>,
        dsl: Value,
        approval: Option<&str>,
    ) -> Result<TaskRule, TaskRuleError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let rule_id = format!("taskrule:v1:{ts:x}");
        let row = sqlx::query("INSERT INTO task_rules (rule_id, name, natural_language_description, compiled_dsl, approval_mode) VALUES ($1,$2,$3,$4,$5) RETURNING rule_id, name, natural_language_description, compiled_dsl, enabled, approval_mode, last_run_at, created_at, updated_at")
            .bind(&rule_id).bind(name).bind(desc).bind(&dsl).bind(approval.unwrap_or("suggest_only")).fetch_one(&self.pool).await?;
        Ok(TaskRule {
            rule_id: row.try_get("rule_id")?,
            name: row.try_get("name")?,
            natural_language_description: row.try_get("natural_language_description")?,
            compiled_dsl: row.try_get("compiled_dsl")?,
            enabled: row.try_get("enabled")?,
            approval_mode: row.try_get("approval_mode")?,
            last_run_at: row.try_get("last_run_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
    pub async fn delete(&self, rule_id: &str) -> Result<(), TaskRuleError> {
        sqlx::query("DELETE FROM task_rules WHERE rule_id=$1")
            .bind(rule_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_fields: Value,
    pub default_checklist: Value,
    pub default_priority: String,
    pub default_energy_type: Option<String>,
    pub required_documents: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TaskTemplateStore {
    pool: PgPool,
}
impl TaskTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self) -> Result<Vec<TaskTemplate>, TaskRuleError> {
        let rows = sqlx::query("SELECT template_id, name, description, default_fields, default_checklist, default_priority, default_energy_type, required_documents, created_at, updated_at FROM task_templates ORDER BY name")
            .fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(TaskTemplate {
                    template_id: r.try_get("template_id")?,
                    name: r.try_get("name")?,
                    description: r.try_get("description")?,
                    default_fields: r.try_get("default_fields")?,
                    default_checklist: r.try_get("default_checklist")?,
                    default_priority: r.try_get("default_priority")?,
                    default_energy_type: r.try_get("default_energy_type")?,
                    required_documents: r.try_get("required_documents")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum TaskRuleError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/tasks/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/service.rs`
- Size bytes / Размер в байтах: `35`
- Included characters / Включено символов: `35`
- Truncated / Обрезано: `no`

```rust
pub use super::command_service::*;
```

### `backend/src/domains/tasks/sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/tasks/sync.rs`
- Size bytes / Размер в байтах: `1035`
- Included characters / Включено символов: `1035`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use thiserror::Error;

pub fn export_task_md(
    title: &str,
    description: Option<&str>,
    status: &str,
    why: Option<&str>,
    outcome: Option<&str>,
) -> String {
    let mut md = format!("# {title}\n\n**Status:** {status}\n\n");
    if let Some(why) = why
        && !why.is_empty()
    {
        md.push_str(&format!("**Why:** {why}\n\n"));
    }
    if let Some(desc) = description
        && !desc.is_empty()
    {
        md.push_str(&format!("{desc}\n\n"));
    }
    if let Some(out) = outcome
        && !out.is_empty()
    {
        md.push_str(&format!("**Outcome:** {out}\n\n"));
    }
    md
}

pub fn export_task_json(
    title: &str,
    description: Option<&str>,
    status: &str,
    priority: Option<f64>,
    due_at: Option<&str>,
) -> Value {
    json!({ "title": title, "description": description, "status": status, "priority": priority, "due_at": due_at })
}

#[derive(Debug, Error)]
pub enum TaskSyncError {
    #[error("sync failed: {0}")]
    SyncFailed(String),
}
```

### `backend/src/engines/automation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation.rs`
- Size bytes / Размер в байтах: `375`
- Included characters / Включено символов: `375`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod dry_run;
mod errors;
mod evidence;
mod ids;
mod models;
mod policy;
mod rows;
mod store;
mod validation;

pub use errors::AutomationError;
pub use models::{
    AutomationPolicy, AutomationTemplate, NewAutomationPolicy, NewAutomationTemplate,
    TelegramSendDryRunRequest, TelegramSendDryRunResponse, object_from_pairs,
};
pub use store::AutomationStore;
```

### `backend/src/engines/automation/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/constants.rs`
- Size bytes / Размер в байтах: `240`
- Included characters / Включено символов: `240`
- Truncated / Обрезано: `no`

```rust
pub(super) const AUTOMATION_SEND_DRY_RUN_EVENT_TYPE: &str = "automation.telegram_send.dry_run";
pub(super) const AUTOMATION_SOURCE_KIND: &str = "automation_policy";
pub(super) const AUTOMATION_SOURCE_PROVIDER: &str = "local_policy_engine";
```

### `backend/src/engines/automation/dry_run.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/dry_run.rs`
- Size bytes / Размер в байтах: `3961`
- Included characters / Включено символов: `3961`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use crate::platform::events::{EventStore, NewEventEnvelope};

use super::constants::{
    AUTOMATION_SEND_DRY_RUN_EVENT_TYPE, AUTOMATION_SOURCE_KIND, AUTOMATION_SOURCE_PROVIDER,
};
use super::errors::AutomationError;
use super::evidence::capture_dry_run_observation;
use super::ids::sha256_hex;
use super::models::{TelegramSendDryRunRequest, TelegramSendDryRunResponse};
use super::policy::evaluate_policy;
use super::store::AutomationStore;
use super::validation::validate_non_empty;

pub(super) async fn dry_run_send(
    pool: &PgPool,
    request: &TelegramSendDryRunRequest,
    actor_id: &str,
) -> Result<TelegramSendDryRunResponse, AutomationError> {
    request.validate()?;
    let actor_id = validate_non_empty("actor_id", actor_id)?;
    let (policy, template) =
        AutomationStore::policy_with_template(pool, &request.policy_id).await?;
    let rendered_text = evaluate_policy(&policy, &template, request)?;
    let rendered_preview_hash = sha256_hex(rendered_text.as_bytes());
    let outbound_message_id = format!(
        "telegram_outbound:v4:{}",
        sha256_hex(
            [
                request.command_id.as_str(),
                request.policy_id.as_str(),
                request.provider_chat_id.as_str(),
                rendered_preview_hash.as_str(),
            ]
            .join("\0")
            .as_bytes()
        )
    );
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO telegram_outbound_messages (
            outbound_message_id,
            policy_id,
            template_id,
            account_id,
            provider_chat_id,
            send_mode,
            status,
            rendered_preview_hash,
            variables,
            source_context,
            actor_id
        )
        VALUES ($1, $2, $3, $4, $5, 'dry_run', 'allowed', $6, $7, $8, $9)
        ON CONFLICT (outbound_message_id)
        DO NOTHING
        "#,
    )
    .bind(&outbound_message_id)
    .bind(&policy.policy_id)
    .bind(&template.template_id)
    .bind(&policy.account_id)
    .bind(&request.provider_chat_id)
    .bind(&rendered_preview_hash)
    .bind(&request.variables)
    .bind(&request.source_context)
    .bind(&actor_id)
    .execute(&mut *transaction)
    .await?;

    let event_id = format!(
        "automation_telegram_send_dry_run:{}",
        request.command_id.trim()
    );
    let event = NewEventEnvelope::builder(
        event_id.clone(),
        AUTOMATION_SEND_DRY_RUN_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": AUTOMATION_SOURCE_KIND,
            "provider": AUTOMATION_SOURCE_PROVIDER,
            "policy_id": policy.policy_id,
        }),
        json!({
            "kind": "telegram_outbound_message",
            "id": outbound_message_id,
        }),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "command_id": request.command_id,
        "outbound_message_id": outbound_message_id,
        "policy_id": policy.policy_id,
        "template_id": template.template_id,
        "account_id": policy.account_id,
        "provider_chat_id": request.provider_chat_id,
        "rendered_preview_hash": rendered_preview_hash,
        "send_mode": "dry_run",
        "status": "allowed",
    }))
    .build()?;
    EventStore::append_in_transaction(&mut transaction, &event).await?;
    let response = TelegramSendDryRunResponse {
        outbound_message_id,
        policy_id: policy.policy_id,
        template_id: template.template_id,
        account_id: policy.account_id,
        provider_chat_id: request.provider_chat_id.clone(),
        rendered_text,
        rendered_preview_hash,
        status: "allowed".to_owned(),
        event_id,
    };
    capture_dry_run_observation(&mut transaction, request, &response, &actor_id, Utc::now())
        .await?;
    transaction.commit().await?;

    Ok(response)
}
```

### `backend/src/engines/automation/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/errors.rs`
- Size bytes / Размер в байтах: `981`
- Included characters / Включено символов: `981`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum AutomationError {
    #[error("invalid automation request: {0}")]
    InvalidRequest(String),

    #[error("automation policy was not found")]
    PolicyNotFound,

    #[error("automation policy is disabled")]
    PolicyDisabled,

    #[error("provider chat is not allowed by policy")]
    ChatNotAllowed,

    #[error("automation template variable is missing: {0}")]
    MissingTemplateVariable(String),

    #[error("automation template received undeclared variable: {0}")]
    UndeclaredTemplateVariable(String),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/engines/automation/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/evidence.rs`
- Size bytes / Размер в байтах: `5582`
- Included characters / Включено символов: `5582`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, link_domain_entity_in_transaction,
};

use super::errors::AutomationError;
use super::models::{
    AutomationPolicy, AutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};

pub(super) async fn capture_template_observation(
    transaction: &mut Transaction<'_, Postgres>,
    template: &AutomationTemplate,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AUTOMATION_TEMPLATE",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "template_id": template.template_id,
                "name": template.name,
                "body_template": template.body_template,
                "required_variables": template.required_variables,
                "operation": relationship_kind,
            }),
            format!("automation-template://{}", template.template_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_domain_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "automation",
        "template",
        template.template_id.clone(),
        Some(relationship_kind),
        None,
        Some(json!({
            "name": template.name,
            "required_variables": template.required_variables,
        })),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_policy_observation(
    transaction: &mut Transaction<'_, Postgres>,
    policy: &AutomationPolicy,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AUTOMATION_POLICY",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "policy_id": policy.policy_id,
                "template_id": policy.template_id,
                "name": policy.name,
                "enabled": policy.enabled,
                "account_id": policy.account_id,
                "allowed_chat_ids": policy.allowed_chat_ids,
                "trigger_kind": policy.trigger_kind,
                "max_sends_per_hour": policy.max_sends_per_hour,
                "quiet_hours": policy.quiet_hours,
                "expires_at": policy.expires_at,
                "conditions": policy.conditions,
                "operation": relationship_kind,
            }),
            format!("automation-policy://{}", policy.policy_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_domain_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "automation",
        "policy",
        policy.policy_id.clone(),
        Some(relationship_kind),
        None,
        Some(json!({
            "template_id": policy.template_id,
            "enabled": policy.enabled,
            "account_id": policy.account_id,
            "trigger_kind": policy.trigger_kind,
        })),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_dry_run_observation(
    transaction: &mut Transaction<'_, Postgres>,
    request: &TelegramSendDryRunRequest,
    response: &TelegramSendDryRunResponse,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_OUTBOUND_MESSAGE",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "outbound_message_id": response.outbound_message_id,
                "command_id": request.command_id,
                "policy_id": response.policy_id,
                "template_id": response.template_id,
                "account_id": response.account_id,
                "provider_chat_id": response.provider_chat_id,
                "rendered_preview_hash": response.rendered_preview_hash,
                "status": response.status,
                "send_mode": "dry_run",
                "variables": request.variables,
                "source_context": request.source_context,
            }),
            format!(
                "telegram-outbound-message://{}",
                response.outbound_message_id
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": "dry_run_allowed",
            "event_id": response.event_id,
        })),
    )
    .await?;
    link_domain_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "automation",
        "telegram_outbound_message",
        response.outbound_message_id.clone(),
        Some("dry_run_allowed"),
        None,
        Some(json!({
            "policy_id": response.policy_id,
            "template_id": response.template_id,
            "account_id": response.account_id,
            "provider_chat_id": response.provider_chat_id,
            "status": response.status,
            "send_mode": "dry_run",
        })),
    )
    .await?;
    Ok(())
}
```

### `backend/src/engines/automation/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/ids.rs`
- Size bytes / Размер в байтах: `190`
- Included characters / Включено символов: `190`
- Truncated / Обрезано: `no`

```rust
use sha2::{Digest, Sha256};

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}
```

### `backend/src/engines/automation/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/models.rs`
- Size bytes / Размер в байтах: `2234`
- Included characters / Включено символов: `2234`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewAutomationTemplate {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AutomationTemplate {
    pub template_id: String,
    pub name: String,
    pub body_template: String,
    pub required_variables: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewAutomationPolicy {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub allowed_chat_ids: Vec<String>,
    pub trigger_kind: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AutomationPolicy {
    pub policy_id: String,
    pub template_id: String,
    pub name: String,
    pub enabled: bool,
    pub account_id: String,
    pub allowed_chat_ids: Vec<String>,
    pub trigger_kind: String,
    pub max_sends_per_hour: i32,
    pub quiet_hours: Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramSendDryRunRequest {
    pub command_id: String,
    pub policy_id: String,
    pub provider_chat_id: String,
    #[serde(default)]
    pub variables: Value,
    #[serde(default)]
    pub source_context: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramSendDryRunResponse {
    pub outbound_message_id: String,
    pub policy_id: String,
    pub template_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub rendered_text: String,
    pub rendered_preview_hash: String,
    pub status: String,
    pub event_id: String,
}

pub fn object_from_pairs(pairs: impl IntoIterator<Item = (String, Value)>) -> Value {
    Value::Object(Map::from_iter(pairs))
}
```

### `backend/src/engines/automation/policy.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/policy.rs`
- Size bytes / Размер в байтах: `1704`
- Included characters / Включено символов: `1704`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::Value;

use super::errors::AutomationError;
use super::models::{AutomationPolicy, AutomationTemplate, TelegramSendDryRunRequest};

pub(super) fn evaluate_policy(
    policy: &AutomationPolicy,
    template: &AutomationTemplate,
    request: &TelegramSendDryRunRequest,
) -> Result<String, AutomationError> {
    if !policy.enabled {
        return Err(AutomationError::PolicyDisabled);
    }
    if let Some(expires_at) = policy.expires_at
        && expires_at < Utc::now()
    {
        return Err(AutomationError::InvalidRequest(
            "policy is expired".to_owned(),
        ));
    }
    if !policy
        .allowed_chat_ids
        .iter()
        .any(|chat_id| chat_id == &request.provider_chat_id)
    {
        return Err(AutomationError::ChatNotAllowed);
    }
    let variables = request
        .variables
        .as_object()
        .ok_or_else(|| AutomationError::InvalidRequest("variables must be an object".to_owned()))?;
    for key in variables.keys() {
        if !template
            .required_variables
            .iter()
            .any(|allowed| allowed == key)
        {
            return Err(AutomationError::UndeclaredTemplateVariable(key.clone()));
        }
    }

    let mut rendered = template.body_template.clone();
    for variable in &template.required_variables {
        let value = variables
            .get(variable)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| AutomationError::MissingTemplateVariable(variable.clone()))?;
        rendered = rendered.replace(&format!("{{{{{variable}}}}}"), value);
    }

    Ok(rendered)
}
```

### `backend/src/engines/automation/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/engines/automation/rows.rs`
- Size bytes / Размер в байтах: `1890`
- Included characters / Включено символов: `1890`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::AutomationError;
use super::models::{AutomationPolicy, AutomationTemplate};

pub(super) fn row_to_template(row: PgRow) -> Result<AutomationTemplate, AutomationError> {
    Ok(AutomationTemplate {
        template_id: row.try_get("template_id")?,
        name: row.try_get("name")?,
        body_template: row.try_get("body_template")?,
        required_variables: string_vec_from_value(row.try_get("required_variables")?)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_policy(row: PgRow) -> Result<AutomationPolicy, AutomationError> {
    Ok(AutomationPolicy {
        policy_id: row.try_get("policy_id")?,
        template_id: row.try_get("template_id")?,
        name: row.try_get("name")?,
        enabled: row.try_get("enabled")?,
        account_id: row.try_get("account_id")?,
        allowed_chat_ids: string_vec_from_value(row.try_get("allowed_chat_ids")?)?,
        trigger_kind: row.try_get("trigger_kind")?,
        max_sends_per_hour: row.try_get("max_sends_per_hour")?,
        quiet_hours: row.try_get("quiet_hours")?,
        expires_at: row.try_get("expires_at")?,
        conditions: row.try_get("conditions")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn string_vec_from_value(value: Value) -> Result<Vec<String>, AutomationError> {
    let values = value
        .as_array()
        .ok_or_else(|| AutomationError::InvalidRequest("expected array".to_owned()))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| AutomationError::InvalidRequest("expected string array".to_owned()))
        })
        .collect()
}
```

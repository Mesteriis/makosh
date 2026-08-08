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

- Chunk ID / ID чанка: `047-source-backend-part-027`
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

### `backend/src/domains/organizations/memory.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/memory.rs`
- Size bytes / Размер в байтах: `11477`
- Included characters / Включено символов: `10997`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

// ── OrgFact ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgFact {
    pub id: String,
    pub organization_id: String,
    pub fact_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgFactStore {
    pool: PgPool,
}
impl OrgFactStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgFact>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, fact_type, value, source, confidence::float8 AS confidence, last_verified_at, valid_from, valid_to, is_active, created_at, updated_at FROM organization_facts WHERE organization_id=$1 ORDER BY created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgFact {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    fact_type: r.try_get("fact_type")?,
                    value: r.try_get("value")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    last_verified_at: r.try_get("last_verified_at")?,
                    valid_from: r.try_get("valid_from")?,
                    valid_to: r.try_get("valid_to")?,
                    is_active: r.try_get("is_active")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        fact_type: &str,
        value: &str,
        source: &str,
        confidence: f64,
    ) -> Result<OrgFact, OrgMemoryError> {
        let row = sqlx::query("INSERT INTO organization_facts (organization_id, fact_type, value, source, confidence) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING RETURNING id::text, organization_id, fact_type, value, source, confidence::float8 AS confidence, last_verified_at, valid_from, valid_to, is_active, created_at, updated_at")
            .bind(org_id).bind(fact_type).bind(value).bind(source).bind(confidence).fetch_one(&self.pool).await?;
        Ok(OrgFact {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            fact_type: row.try_get("fact_type")?,
            value: row.try_get("value")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            valid_from: row.try_get("valid_from")?,
            valid_to: row.try_get("valid_to")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
    pub async fn decay_unverified(&self, threshold_days: i64) -> Result<u64, OrgMemoryError> {
        let result = sqlx::query("UPDATE organization_facts SET confidence = confidence * 0.5, updated_at = now() WHERE last_verified_at IS NULL OR last_verified_at < now() - ($1 || ' days')::interval")
            .bind(threshold_days).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }
}

// ── OrgMemoryCard ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgMemoryCard {
    pub id: String,
    pub organization_id: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub confidence: f64,
    pub importance: i16,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct OrgMemoryCardStore {
    pool: PgPool,
}
impl OrgMemoryCardStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgMemoryCard>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, title, description, source, confidence::float8 AS confidence, importance, created_at, last_verified_at FROM organization_memory_cards WHERE organization_id=$1 ORDER BY importance DESC, created_at DESC")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgMemoryCard {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    title: r.try_get("title")?,
                    description: r.try_get("description")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    importance: r.try_get("importance")?,
                    created_at: r.try_get("created_at")?,
                    last_verified_at: r.try_get("last_verified_at")?,
                })
            })
            .collect()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        title: &str,
        description: &str,
        source: &str,
        importance: i16,
    ) -> Result<OrgMemoryCard, OrgMemoryError> {
        let row = sqlx::query("INSERT INTO organization_memory_cards (organization_id, title, description, source, importance) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING RETURNING id::text, organization_id, title, description, source, confidence::float8 AS confidence, importance, created_at, last_verified_at")
            .bind(org_id).bind(title).bind(description).bind(source).bind(importance).fetch_one(&self.pool).await?;
        Ok(OrgMemoryCard {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            importance: row.try_get("importance")?,
            created_at: row.try_get("created_at")?,
            last_verified_at: row.try_get("last_verified_at")?,
        })
    }
}

// ── OrgPreference ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPreference {
    pub id: String,
    pub organization_id: String,
    pub preference_type: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPreferenceStore {
    pool: PgPool,
}
impl OrgPreferenceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPreference>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, preference_type, value, source, confidence::float8 AS confidence, last_verified_at, created_at, updated_at FROM organization_preferences WHERE organization_id=$1 ORDER BY preference_type")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgPreference {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    preference_type: r.try_get("preference_type")?,
                    value: r.try_get("value")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    last_verified_at: r.try_get("last_verified_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }
    pub async fn upsert(
        &self,
        org_id: &str,
        ptype: &str,
        value: &str,
        source: &str,
    ) -> Result<OrgPreference, OrgMemoryError> {
        let row = sqlx::query("INSERT INTO organization_preferences (organization_id, preference_type, value, source) VALUES ($1,$2,$3,$4) ON CONFLICT (organization_id, preference_type) DO UPDATE SET value=$3, source=$4, updated_at=now() RETURNING id::text, organization_id, preference_type, value, source, confidence::float8 AS confidence, last_verified_at, created_at, updated_at")
            .bind(org_id).bind(ptype).bind(value).bind(source).fetch_one(&self.pool).await?;
        Ok(OrgPreference {
            id: row.try_get("id")?,
            organization_id: row.try_get("organization_id")?,
            preference_type: row.try_get("preference_type")?,
            value: row.try_get("value")?,
            source: row.try_get("source")?,
            confidence: row.try_get("confidence")?,
            last_verified_at: row.try_get("last_verified_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

// ── OrgRequiredDocument ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgRequiredDocument {
    pub id: String,
    pub organization_id: String,
    pub document_type: String,
    pub description: Option<String>,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgRequiredDocStore {
    pool: PgPool,
}
impl OrgRequiredDocStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgRequiredDocument>, OrgMemoryError> {
        let rows = sqlx::query("SELECT id::text, organization_id, document_type, description, source, confidence::float8 AS confidence, created_at FROM organization_required_documents WHERE organization_id=$1 ORDER BY document_type")
            .bind(org_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(OrgRequiredDocument {
                    id: r.try_get("id")?,
                    organization_id: r.try_get("organization_id")?,
                    document_type: r.try_get("document_type")?,
                    description: r.try_get("description")?,
                    source: r.try_get("source")?,
                    confidence: r.try_get("confidence")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum OrgMemoryError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
```

### `backend/src/domains/organizations/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/mod.rs`
- Size bytes / Размер в байтах: `169`
- Included characters / Включено символов: `169`
- Truncated / Обрезано: `no`

```rust
pub mod api;
pub mod core;
pub mod enrichment;
pub mod finance;
pub mod health;
pub mod investigator;
pub mod memory;
pub mod ports;
pub mod service;
pub mod workflows;
```

### `backend/src/domains/organizations/ports.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/ports.rs`
- Size bytes / Размер в байтах: `139`
- Included characters / Включено символов: `139`
- Truncated / Обрезано: `no`

```rust
pub use super::api::OrganizationStore as OrganizationCommandPort;
pub use super::core::OrgContactLinkStore as OrganizationContactLinkPort;
```

### `backend/src/domains/organizations/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/service.rs`
- Size bytes / Размер в байтах: `11665`
- Included characters / Включено символов: `11665`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::api::{Organization, OrganizationError, OrganizationStore, OrganizationUpdate};
use super::core::{
    OrgAliasStore, OrgContactLink, OrgContactLinkStore, OrgCoreError, OrgDepartment,
    OrgDepartmentStore, OrgIdentityStore, OrganizationAlias, OrganizationIdentity,
};
use super::enrichment::{OrgEnrichmentError, OrgEnrichmentStore};
use super::health::{OrgHealthError, OrgHealthStore};

#[derive(Clone)]
pub struct OrganizationCommandService {
    pool: PgPool,
}

impl OrganizationCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_organization_manual(
        &self,
        display_name: &str,
        org_type: Option<&str>,
    ) -> Result<Organization, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "display_name": display_name,
                    "org_type": org_type,
                    "action": "create",
                }),
                format!("organizations://create/{display_name}"),
                json!({
                    "captured_by": "organizations_service.create_organization_manual",
                    "operation": "create_organization_manual",
                }),
            )
            .await?;

        Ok(OrganizationStore::new(self.pool.clone())
            .create_with_observation(display_name, org_type, &observation.observation_id)
            .await?)
    }

    pub async fn update_organization_manual(
        &self,
        organization_id: &str,
        update: &OrganizationUpdate,
    ) -> Result<Organization, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "update": serde_json::to_value(update).unwrap_or(Value::Null),
                    "action": "update",
                }),
                format!("organization://{organization_id}/update"),
                json!({
                    "captured_by": "organizations_service.update_organization_manual",
                    "operation": "update_organization_manual",
                }),
            )
            .await?;

        Ok(OrganizationStore::new(self.pool.clone())
            .update_with_observation(organization_id, update, &observation.observation_id)
            .await?)
    }

    pub async fn archive_organization_manual(
        &self,
        organization_id: &str,
    ) -> Result<(), OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "action": "archive",
                }),
                format!("organization://{organization_id}/archive"),
                json!({
                    "captured_by": "organizations_service.archive_organization_manual",
                    "operation": "archive_organization_manual",
                }),
            )
            .await?;

        OrganizationStore::new(self.pool.clone())
            .archive_with_observation(organization_id, &observation.observation_id)
            .await?;
        Ok(())
    }

    pub async fn add_identity_manual(
        &self,
        organization_id: &str,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<OrganizationIdentity, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("organization://{organization_id}/identities/{identity_type}"),
                json!({
                    "captured_by": "organizations_service.add_identity_manual",
                    "operation": "add_identity_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(OrgIdentityStore::new(self.pool.clone())
            .upsert_with_observation(
                organization_id,
                identity_type,
                identity_value,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn add_alias_manual(
        &self,
        organization_id: &str,
        name: &str,
        alias_type: &str,
        requested_source: &str,
    ) -> Result<OrganizationAlias, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "name": name,
                    "alias_type": alias_type,
                    "source": requested_source,
                }),
                format!("organization://{organization_id}/aliases/{alias_type}"),
                json!({
                    "captured_by": "organizations_service.add_alias_manual",
                    "operation": "add_alias_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(OrgAliasStore::new(self.pool.clone())
            .add_with_observation(
                organization_id,
                name,
                alias_type,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn add_department_manual(
        &self,
        organization_id: &str,
        name: &str,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<OrgDepartment, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "name": name,
                    "description": description,
                    "parent_department_id": parent_id,
                }),
                format!("organization://{organization_id}/departments/{name}"),
                json!({
                    "captured_by": "organizations_service.add_department_manual",
                    "operation": "add_department_manual",
                }),
            )
            .await?;

        Ok(OrgDepartmentStore::new(self.pool.clone())
            .add_with_observation(
                organization_id,
                name,
                description,
                parent_id,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn link_contact_manual(
        &self,
        organization_id: &str,
        person_id: &str,
        role: Option<&str>,
        department: Option<&str>,
        requested_source: &str,
    ) -> Result<OrgContactLink, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_RECORD_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "person_id": person_id,
                    "role": role,
                    "department": department,
                    "source": requested_source,
                }),
                format!("organization://{organization_id}/contacts/{person_id}"),
                json!({
                    "captured_by": "organizations_service.link_contact_manual",
                    "operation": "link_contact_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(OrgContactLinkStore::new(self.pool.clone())
            .link_with_observation(
                organization_id,
                person_id,
                role,
                department,
                Some(&format!("observation:{}", observation.observation_id)),
                Some(&observation.observation_id),
            )
            .await?)
    }

    pub async fn apply_enrichment_manual(
        &self,
        organization_id: &str,
        result_id: &str,
    ) -> Result<(), OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "REVIEW_TRANSITION",
                json!({
                    "organization_id": organization_id,
                    "result_id": result_id,
                    "operation": "organization_enrichment_apply",
                }),
                format!("organization://{organization_id}/enrichment/{result_id}/apply"),
                json!({
                    "captured_by": "organizations_service.apply_enrichment_manual",
                    "operation": "apply_enrichment_manual",
                }),
            )
            .await?;

        OrgEnrichmentStore::new(self.pool.clone())
            .apply_with_observation(result_id, &observation.observation_id)
            .await?;
        Ok(())
    }

    pub async fn toggle_watchlist_manual(
        &self,
        organization_id: &str,
    ) -> Result<bool, OrganizationCommandServiceError> {
        let observation = self
            .capture_manual(
                "ORGANIZATION_MUTATION",
                json!({
                    "organization_id": organization_id,
                    "action": "toggle_watchlist",
                }),
                format!("organization://{organization_id}/watchlist"),
                json!({
                    "captured_by": "organizations_service.toggle_watchlist_manual",
                    "operation": "toggle_watchlist_manual",
                }),
            )
            .await?;

        Ok(OrgHealthStore::new(self.pool.clone())
            .toggle_watchlist_with_observation(
                organization_id,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    async fn capture_manual(
        &self,
        kind: &str,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<crate::platform::observations::Observation, OrganizationCommandServiceError> {
        Ok(ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await?)
    }
}

#[derive(Debug, Error)]
pub enum OrganizationCommandServiceError {
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Organization(#[from] OrganizationError),

    #[error(transparent)]
    Core(#[from] OrgCoreError),

    #[error(transparent)]
    Enrichment(#[from] OrgEnrichmentError),

    #[error(transparent)]
    Health(#[from] OrgHealthError),
}
```

### `backend/src/domains/organizations/workflows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/workflows.rs`
- Size bytes / Размер в байтах: `381`
- Included characters / Включено символов: `381`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod playbooks;
mod portals;
mod procedures;
mod templates;
mod timeline;

pub use errors::OrgWorkflowError;
pub use playbooks::{OrgPlaybook, OrgPlaybookStore};
pub use portals::{OrgPortal, OrgPortalStore};
pub use procedures::{OrgProcedure, OrgProcedureStore};
pub use templates::{OrgTemplate, OrgTemplateStore};
pub use timeline::{OrgTimelineEvent, OrgTimelineStore};
```

### `backend/src/domains/organizations/workflows/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/workflows/errors.rs`
- Size bytes / Размер в байтах: `229`
- Included characters / Включено символов: `229`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrgWorkflowError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Timeline(#[from] crate::engines::timeline::TimelineEngineError),
}
```

### `backend/src/domains/organizations/workflows/playbooks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/workflows/playbooks.rs`
- Size bytes / Размер в байтах: `1947`
- Included characters / Включено символов: `1947`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPlaybook {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub trigger_condition: Option<String>,
    pub steps: Value,
    pub approval_mode: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPlaybookStore {
    pool: PgPool,
}

impl OrgPlaybookStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPlaybook>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, trigger_condition, steps, approval_mode,
                   enabled, last_run_at, created_at, updated_at
            FROM organization_playbooks
            WHERE organization_id=$1
            ORDER BY name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgPlaybook {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    trigger_condition: row.try_get("trigger_condition")?,
                    steps: row.try_get("steps")?,
                    approval_mode: row.try_get("approval_mode")?,
                    enabled: row.try_get("enabled")?,
                    last_run_at: row.try_get("last_run_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
```

### `backend/src/domains/organizations/workflows/portals.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/workflows/portals.rs`
- Size bytes / Размер в байтах: `2523`
- Included characters / Включено символов: `2523`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgPortal {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub url: String,
    pub portal_type: String,
    pub login_hint: Option<String>,
    pub secret_reference: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgPortalStore {
    pool: PgPool,
}

impl OrgPortalStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgPortal>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, url, portal_type, login_hint,
                   secret_reference, last_used_at, notes, created_at
            FROM organization_portals
            WHERE organization_id=$1
            ORDER BY portal_type, name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(portal_from_row).collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        name: &str,
        url: &str,
        portal_type: &str,
    ) -> Result<OrgPortal, OrgWorkflowError> {
        let row = sqlx::query(
            r#"
            INSERT INTO organization_portals (organization_id, name, url, portal_type)
            VALUES ($1,$2,$3,$4)
            RETURNING id::text, organization_id, name, url, portal_type, login_hint,
                      secret_reference, last_used_at, notes, created_at
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(url)
        .bind(portal_type)
        .fetch_one(&self.pool)
        .await?;

        portal_from_row(row)
    }
}

fn portal_from_row(row: sqlx::postgres::PgRow) -> Result<OrgPortal, OrgWorkflowError> {
    Ok(OrgPortal {
        id: row.try_get("id")?,
        organization_id: row.try_get("organization_id")?,
        name: row.try_get("name")?,
        url: row.try_get("url")?,
        portal_type: row.try_get("portal_type")?,
        login_hint: row.try_get("login_hint")?,
        secret_reference: row.try_get("secret_reference")?,
        last_used_at: row.try_get("last_used_at")?,
        notes: row.try_get("notes")?,
        created_at: row.try_get("created_at")?,
    })
}
```

### `backend/src/domains/organizations/workflows/procedures.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/workflows/procedures.rs`
- Size bytes / Размер в байтах: `1957`
- Included characters / Включено символов: `1957`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgProcedure {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Value,
    pub source: String,
    pub confidence: f64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgProcedureStore {
    pool: PgPool,
}

impl OrgProcedureStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgProcedure>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, description, steps, source,
                   confidence::float8 AS confidence,
                   last_used_at, created_at, updated_at
            FROM organization_procedures
            WHERE organization_id=$1
            ORDER BY name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgProcedure {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    steps: row.try_get("steps")?,
                    source: row.try_get("source")?,
                    confidence: row.try_get("confidence")?,
                    last_used_at: row.try_get("last_used_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
```

### `backend/src/domains/organizations/workflows/templates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/workflows/templates.rs`
- Size bytes / Размер в байтах: `1982`
- Included characters / Включено символов: `1982`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgTemplate {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub template_type: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub language: Option<String>,
    pub tone: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgTemplateStore {
    pool: PgPool,
}

impl OrgTemplateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, org_id: &str) -> Result<Vec<OrgTemplate>, OrgWorkflowError> {
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, name, template_type, subject, body, language,
                   tone, metadata, created_at, updated_at
            FROM organization_templates
            WHERE organization_id=$1
            ORDER BY name
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(OrgTemplate {
                    id: row.try_get("id")?,
                    organization_id: row.try_get("organization_id")?,
                    name: row.try_get("name")?,
                    template_type: row.try_get("template_type")?,
                    subject: row.try_get("subject")?,
                    body: row.try_get("body")?,
                    language: row.try_get("language")?,
                    tone: row.try_get("tone")?,
                    metadata: row.try_get("metadata")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}
```

### `backend/src/domains/organizations/workflows/timeline.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/organizations/workflows/timeline.rs`
- Size bytes / Размер в байтах: `3526`
- Included characters / Включено символов: `3526`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::PgPool;

use crate::engines::timeline::{TimelineEngine, TimelineEventDraft};

use super::errors::OrgWorkflowError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgTimelineEvent {
    pub id: String,
    pub organization_id: String,
    pub event_type: String,
    pub title: String,
    pub description: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub source: String,
    pub related_entity_id: Option<String>,
    pub related_entity_kind: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OrgTimelineStore {
    pool: PgPool,
}

impl OrgTimelineStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        org_id: &str,
        limit: i64,
    ) -> Result<Vec<OrgTimelineEvent>, OrgWorkflowError> {
        let limit = TimelineEngine::bounded_entity_limit(limit);
        let rows = sqlx::query(
            r#"
            SELECT id::text, organization_id, event_type, title, description, occurred_at,
                   source, related_entity_id, related_entity_kind,
                   confidence::float8 AS confidence, metadata, created_at
            FROM organization_timeline_events
            WHERE organization_id=$1
            ORDER BY occurred_at DESC
            LIMIT $2
            "#,
        )
        .bind(org_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(timeline_event_from_row).collect()
    }

    pub async fn add(
        &self,
        org_id: &str,
        event_type: &str,
        title: &str,
        occurred_at: DateTime<Utc>,
        source: &str,
    ) -> Result<OrgTimelineEvent, OrgWorkflowError> {
        TimelineEngine::validate_event(&TimelineEventDraft {
            entity_kind: "organization",
            entity_id: org_id,
            event_type,
            title,
            occurred_at,
            source,
        })?;

        let row = sqlx::query(
            r#"
            INSERT INTO organization_timeline_events (organization_id, event_type, title, occurred_at, source)
            VALUES ($1,$2,$3,$4,$5)
            RETURNING id::text, organization_id, event_type, title, description, occurred_at,
                      source, related_entity_id, related_entity_kind,
                      confidence::float8 AS confidence, metadata, created_at
            "#,
        )
        .bind(org_id)
        .bind(event_type)
        .bind(title)
        .bind(occurred_at)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;

        timeline_event_from_row(row)
    }
}

fn timeline_event_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<OrgTimelineEvent, OrgWorkflowError> {
    Ok(OrgTimelineEvent {
        id: row.try_get("id")?,
        organization_id: row.try_get("organization_id")?,
        event_type: row.try_get("event_type")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        occurred_at: row.try_get("occurred_at")?,
        source: row.try_get("source")?,
        related_entity_id: row.try_get("related_entity_id")?,
        related_entity_kind: row.try_get("related_entity_kind")?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
    })
}
```

### `backend/src/domains/persons/analytics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/analytics.rs`
- Size bytes / Размер в байтах: `6970`
- Included characters / Включено символов: `6970`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

/// Aggregated analytics for a person.
#[derive(Clone, Debug, Serialize)]
pub struct PersonAnalytics {
    pub person_id: String,
    pub relationship_score: f64,
    pub intelligence_score: f64,
    pub interaction_heatmap: Vec<HeatmapEntry>,
    pub communication_costs: CommunicationCosts,
    pub shared_context: SharedContext,
}

#[derive(Clone, Debug, Serialize)]
pub struct HeatmapEntry {
    pub day_of_week: i32,
    pub hour: i32,
    pub count: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommunicationCosts {
    pub avg_thread_length: f64,
    pub avg_response_hours: f64,
    pub follow_up_frequency: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SharedContext {
    pub shared_projects: i64,
    pub shared_documents: i64,
    pub shared_tasks: i64,
}

#[derive(Clone)]
pub struct PersonAnalyticsService {
    pool: PgPool,
}

impl PersonAnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Compute full analytics for a person.
    pub async fn compute(&self, person_id: &str) -> Result<PersonAnalytics, AnalyticsError> {
        let rel_score = self.relationship_score(person_id).await.unwrap_or(0.0);
        let intel_score = self.intelligence_score(person_id).await.unwrap_or(0.0);
        let heatmap = self
            .interaction_heatmap(person_id)
            .await
            .unwrap_or_default();
        let costs = self
            .communication_costs(person_id)
            .await
            .unwrap_or(CommunicationCosts {
                avg_thread_length: 0.0,
                avg_response_hours: 0.0,
                follow_up_frequency: 0.0,
            });
        let ctx = self
            .shared_context(person_id)
            .await
            .unwrap_or(SharedContext {
                shared_projects: 0,
                shared_documents: 0,
                shared_tasks: 0,
            });

        Ok(PersonAnalytics {
            person_id: person_id.to_string(),
            relationship_score: rel_score,
            intelligence_score: intel_score,
            interaction_heatmap: heatmap,
            communication_costs: costs,
            shared_context: ctx,
        })
    }

    /// Relationship score: weighted from interaction recency, count, trust.
    async fn relationship_score(&self, person_id: &str) -> Result<f64, AnalyticsError> {
        let row = sqlx::query(
            "SELECT COALESCE(interaction_count, 0) as ic, COALESCE(trust_score, 50) as ts FROM persons WHERE person_id = $1"
        ).bind(person_id).fetch_optional(&self.pool).await?;
        if let Some(r) = row {
            let ic: i32 = r.try_get("ic").unwrap_or(0);
            let ts: i16 = r.try_get("ts").unwrap_or(50);
            Ok((ic as f64 * 0.5 + ts as f64 * 0.5).min(100.0))
        } else {
            Ok(0.0)
        }
    }

    /// Intelligence score: completeness of person profile.
    async fn intelligence_score(&self, person_id: &str) -> Result<f64, AnalyticsError> {
        let row = sqlx::query(
            r#"SELECT
                (CASE WHEN language IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN tone IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN trust_score IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN preferred_channel IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN writing_style IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN timezone IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN person_type IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN primary_role IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN organization_reference IS NOT NULL THEN 10 ELSE 0 END +
                 CASE WHEN notes IS NOT NULL THEN 10 ELSE 0 END) as score
             FROM persons WHERE person_id = $1"#,
        )
        .bind(person_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .map(|r| r.try_get::<i32, _>("score").unwrap_or(0) as f64)
            .unwrap_or(0.0))
    }

    /// Interaction heatmap: message count by day-of-week and hour.
    async fn interaction_heatmap(
        &self,
        person_id: &str,
    ) -> Result<Vec<HeatmapEntry>, AnalyticsError> {
        let rows = sqlx::query(
            r#"SELECT
                extract(dow from occurred_at)::int as day_of_week,
                extract(hour from occurred_at)::int as hour,
                count(*) as count
             FROM communication_messages
             WHERE occurred_at IS NOT NULL
               AND (sender like $1 || '%' OR recipients like '%' || $1 || '%')
             GROUP BY 1, 2 ORDER BY 1, 2 LIMIT 168"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| HeatmapEntry {
                day_of_week: r.try_get("day_of_week").unwrap_or(0),
                hour: r.try_get("hour").unwrap_or(0),
                count: r.try_get("count").unwrap_or(0),
            })
            .collect())
    }

    /// Communication costs: avg thread length, response time, follow-up rate.
    async fn communication_costs(
        &self,
        person_id: &str,
    ) -> Result<CommunicationCosts, AnalyticsError> {
        // Simplified: use existing avg_response_hours and interaction_count from persons table
        let row = sqlx::query(
            "SELECT COALESCE(avg_response_hours, 0.0) as arh, interaction_count FROM persons WHERE person_id = $1"
        ).bind(person_id).fetch_optional(&self.pool).await?;
        if let Some(r) = row {
            let arh: f64 = r.try_get("arh").unwrap_or(0.0);
            let ic: i32 = r.try_get("interaction_count").unwrap_or(0);
            Ok(CommunicationCosts {
                avg_thread_length: if ic > 0 {
                    (ic as f64 / 10.0).min(50.0)
                } else {
                    0.0
                },
                avg_response_hours: arh,
                follow_up_frequency: if ic > 0 { 0.3 } else { 0.0 },
            })
        } else {
            Ok(CommunicationCosts {
                avg_thread_length: 0.0,
                avg_response_hours: 0.0,
                follow_up_frequency: 0.0,
            })
        }
    }

    /// Shared context counts.
    async fn shared_context(&self, person_id: &str) -> Result<SharedContext, AnalyticsError> {
        let proj_count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM graph_edges WHERE source_node_id = $1 AND relationship_type = 'person_involved_in_project'"
        ).bind(person_id).fetch_one(&self.pool).await.unwrap_or(0);

        Ok(SharedContext {
            shared_projects: proj_count,
            shared_documents: 0,
            shared_tasks: 0,
        })
    }
}

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/domains/persons/api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api.rs`
- Size bytes / Размер в байтах: `331`
- Included characters / Включено символов: `331`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod participants;
mod rows;
mod store;
mod validation;

pub use errors::PersonProjectionError;
pub use models::{Person, Persona, PersonaType};
pub use participants::upsert_persons_from_message_participants;
pub use store::PersonProjectionStore;
pub use store::PersonProjectionStore as PersonProjectionPort;
```

### `backend/src/domains/persons/api/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/errors.rs`
- Size bytes / Размер в байтах: `749`
- Included characters / Включено символов: `749`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersonProjectionError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] crate::platform::observations::ObservationStoreError),

    #[error("email address must not be empty")]
    EmptyEmailAddress,

    #[error("invalid email address: {0}")]
    InvalidEmailAddress(String),

    #[error("AI agent id must not be empty")]
    EmptyAiAgentId,

    #[error("invalid AI agent id: {0}")]
    InvalidAiAgentId(String),

    #[error("display name must not be empty")]
    EmptyDisplayName,

    #[error("person was not found: {0}")]
    PersonNotFound(String),

    #[error("invalid persona type: {0}")]
    InvalidPersonaType(String),
}
```

### `backend/src/domains/persons/api/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/models.rs`
- Size bytes / Размер в байтах: `1504`
- Included characters / Включено символов: `1502`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::errors::PersonProjectionError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaType {
    Human,
    AiAgent,
    OrganizationProxy,
    System,
}

impl PersonaType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::AiAgent => "ai_agent",
            Self::OrganizationProxy => "organization_proxy",
            Self::System => "system",
        }
    }
}

impl TryFrom<&str> for PersonaType {
    type Error = PersonProjectionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "human" => Ok(Self::Human),
            "ai_agent" => Ok(Self::AiAgent),
            "organization_proxy" => Ok(Self::OrganizationProxy),
            "system" => Ok(Self::System),
            _ => Err(PersonProjectionError::InvalidPersonaType(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Person {
    pub person_id: String,
    pub display_name: String,
    pub email_address: String,
    pub persona_type: PersonaType,
    pub is_self: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compatibility alias: Persona ≡ Person.
/// ADR-0084 defines Persona as the canonical model name.
/// Use `Persona` in new code; `Person` remains for DB/persistence compatibility.
pub type Persona = Person;
```

### `backend/src/domains/persons/api/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/participants.rs`
- Size bytes / Размер в байтах: `1116`
- Included characters / Включено символов: `1116`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashSet;

use super::errors::PersonProjectionError;
use super::models::Person;
use super::store::PersonProjectionStore;
use super::validation::normalize_email_address;

pub async fn upsert_persons_from_message_participants(
    store: &PersonProjectionStore,
    email_addresses: &[String],
) -> Result<Vec<Person>, PersonProjectionError> {
    let normalized_email_addresses = normalize_email_addresses(email_addresses)?;
    let mut persons = Vec::new();

    for email_address in normalized_email_addresses {
        persons.push(store.upsert_email_person(&email_address).await?);
    }

    Ok(persons)
}

fn normalize_email_addresses(
    email_addresses: &[String],
) -> Result<Vec<String>, PersonProjectionError> {
    let mut seen = HashSet::new();
    let mut normalized_email_addresses = Vec::new();

    for email_address in email_addresses {
        let normalized_email = normalize_email_address(email_address)?;
        if seen.insert(normalized_email.clone()) {
            normalized_email_addresses.push(normalized_email);
        }
    }

    Ok(normalized_email_addresses)
}
```

### `backend/src/domains/persons/api/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/rows.rs`
- Size bytes / Размер в байтах: `623`
- Included characters / Включено символов: `623`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonProjectionError;
use super::models::{Person, PersonaType};

pub(super) fn row_to_person(row: PgRow) -> Result<Person, PersonProjectionError> {
    Ok(Person {
        person_id: row.try_get("person_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        persona_type: PersonaType::try_from(row.try_get::<String, _>("person_type")?.as_str())?,
        is_self: row.try_get("is_self")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/persons/api/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store.rs`
- Size bytes / Размер в байтах: `398`
- Included characters / Включено символов: `398`
- Truncated / Обрезано: `no`

```rust
mod ai_agents;
mod email_projection;
mod owner;
mod persona_reads;
mod persona_type;
mod persona_writes;
mod review_projection;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonProjectionStore {
    pool: PgPool,
}

impl PersonProjectionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }
}
```

### `backend/src/domains/persons/api/store/ai_agents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store/ai_agents.rs`
- Size bytes / Размер в байтах: `4109`
- Included characters / Включено символов: `4109`
- Truncated / Обрезано: `no`

```rust
use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::{Person, PersonaType};
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::api::validation::{
    ai_agent_email_address, ai_agent_person_id, normalize_ai_agent_id, validate_display_name,
};
use crate::platform::graph::{GraphNodeKind, node_id};

impl PersonProjectionStore {
    pub async fn upsert_ai_agent_persona(
        &self,
        agent_id: &str,
        display_name: &str,
    ) -> Result<Person, PersonProjectionError> {
        let normalized_agent_id = normalize_ai_agent_id(agent_id)?;
        validate_display_name(display_name)?;
        let person_id = ai_agent_person_id(&normalized_agent_id);
        let email_address = ai_agent_email_address(&normalized_agent_id);
        let mut transaction = self.pool().begin().await?;

        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address,
                person_type,
                is_self
            )
            VALUES ($1, $2, $3, 'ai_agent', false)
            ON CONFLICT (person_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                email_address = EXCLUDED.email_address,
                person_type = 'ai_agent',
                is_self = false,
                updated_at = now()
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(&person_id)
        .bind(&email_address)
        .bind(&email_address)
        .fetch_one(&mut *transaction)
        .await?;

        let person = row_to_person(row)?;
        let graph_node_id = node_id(GraphNodeKind::Person, &person.person_id);
        sqlx::query(
            r#"
            INSERT INTO graph_nodes (
                node_id,
                node_kind,
                stable_key,
                label,
                properties
            )
            VALUES (
                $1,
                'person',
                $2,
                $3,
                jsonb_build_object(
                    'email_address', $3,
                    'persona_type', 'ai_agent',
                    'agent_id', $4
                )
            )
            ON CONFLICT (node_kind, stable_key)
            DO UPDATE SET
                label = EXCLUDED.label,
                properties = graph_nodes.properties || EXCLUDED.properties,
                updated_at = now()
            "#,
        )
        .bind(&graph_node_id)
        .bind(&person.person_id)
        .bind(&email_address)
        .bind(&normalized_agent_id)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO person_identities (
                person_id,
                identity_type,
                identity_value,
                source,
                confidence,
                status,
                metadata
            )
            VALUES (
                $1,
                'email',
                $2,
                'ai_agent_registry',
                1.0,
                'active',
                jsonb_build_object('agent_id', $3, 'persona_type', 'ai_agent')
            )
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                person_id = EXCLUDED.person_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                metadata = EXCLUDED.metadata,
                last_verified_at = now(),
                updated_at = now()
            "#,
        )
        .bind(&person.person_id)
        .bind(&email_address)
        .bind(&normalized_agent_id)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(person)
    }
}
```

### `backend/src/domains/persons/api/store/email_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store/email_projection.rs`
- Size bytes / Размер в байтах: `4916`
- Included characters / Включено символов: `4916`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::api::validation::{normalize_email_address, person_id_for_email};
use crate::domains::persons::core::link_persons_entity_in_transaction;

impl PersonProjectionStore {
    pub async fn upsert_email_person(
        &self,
        email_address: &str,
    ) -> Result<Person, PersonProjectionError> {
        self.upsert_email_person_internal(email_address, None).await
    }

    pub async fn upsert_email_person_with_observation(
        &self,
        email_address: &str,
        observation_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        self.upsert_email_person_internal(email_address, Some(observation_id))
            .await
    }

    async fn upsert_email_person_internal(
        &self,
        email_address: &str,
        observation_id: Option<&str>,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let (person, identity_id) =
            Self::upsert_email_person_in_transaction(&mut transaction, email_address).await?;
        if let Some(observation_id) = observation_id {
            Self::link_email_person_projection_in_transaction(
                &mut transaction,
                observation_id,
                &person,
                &identity_id,
                email_address,
                "email_sync_projection",
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(person)
    }

    pub(crate) async fn upsert_email_person_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        email_address: &str,
    ) -> Result<(Person, String), PersonProjectionError> {
        let normalized_email = normalize_email_address(email_address)?;
        let person_id = person_id_for_email(&normalized_email);

        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (email_address)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                updated_at = now()
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(&person_id)
        .bind(&normalized_email)
        .bind(&normalized_email)
        .fetch_one(&mut **transaction)
        .await?;

        let person = row_to_person(row)?;
        let identity_row = sqlx::query(
            r#"
            INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
            VALUES ($1, 'email', $2, 'email_sync', 1.0, 'active')
            ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
            DO UPDATE SET
                person_id = EXCLUDED.person_id,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                last_verified_at = now(),
                updated_at = now()
            RETURNING id::text
            "#,
        )
        .bind(&person.person_id)
        .bind(&normalized_email)
        .fetch_one(&mut **transaction)
        .await?;
        let identity_id = identity_row.try_get("id")?;

        Ok((person, identity_id))
    }

    pub(crate) async fn link_email_person_projection_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        observation_id: &str,
        person: &Person,
        identity_id: &str,
        identity_value: &str,
        relationship_kind: &str,
    ) -> Result<(), PersonProjectionError> {
        link_persons_entity_in_transaction(
            transaction,
            observation_id,
            "persona",
            person.person_id.clone(),
            Some(relationship_kind),
            Some(serde_json::json!({
                "projection": "persona",
                "identity_type": "email",
                "identity_value": identity_value,
            })),
        )
        .await?;
        link_persons_entity_in_transaction(
            transaction,
            observation_id,
            "identity",
            identity_id.to_owned(),
            Some(relationship_kind),
            Some(serde_json::json!({
                "projection": "identity",
                "person_id": person.person_id,
                "identity_type": "email",
                "identity_value": identity_value,
            })),
        )
        .await?;
        Ok(())
    }
}
```

### `backend/src/domains/persons/api/store/owner.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store/owner.rs`
- Size bytes / Размер в байтах: `2791`
- Included characters / Включено символов: `2791`
- Truncated / Обрезано: `no`

```rust
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::core::link_persons_entity_in_transaction;

impl PersonProjectionStore {
    pub async fn owner_persona(&self) -> Result<Option<Person>, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE is_self = true
            "#,
        )
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_person).transpose()
    }

    pub async fn set_owner_persona(
        &self,
        person_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = assign_owner_persona_in_transaction(&mut transaction, person_id).await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn set_owner_persona_with_observation(
        &self,
        person_id: &str,
        observation_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person = assign_owner_persona_in_transaction(&mut transaction, person_id).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            person_id,
            Some("owner_assignment"),
            None,
        )
        .await?;
        transaction.commit().await?;
        Ok(person)
    }
}

pub(super) async fn assign_owner_persona_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
) -> Result<Person, PersonProjectionError> {
    sqlx::query(
        r#"
        UPDATE persons
        SET is_self = false, updated_at = now()
        WHERE is_self = true AND person_id <> $1
        "#,
    )
    .bind(person_id)
    .execute(&mut **transaction)
    .await?;

    let row = sqlx::query(
        r#"
        UPDATE persons
        SET is_self = true, updated_at = now()
        WHERE person_id = $1
        RETURNING
            person_id,
            display_name,
            email_address,
            person_type,
            is_self,
            created_at,
            updated_at
        "#,
    )
    .bind(person_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| PersonProjectionError::PersonNotFound(person_id.to_owned()))?;

    row_to_person(row)
}
```

### `backend/src/domains/persons/api/store/persona_reads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store/persona_reads.rs`
- Size bytes / Размер в байтах: `1486`
- Included characters / Включено символов: `1486`
- Truncated / Обрезано: `no`

```rust
use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;

impl PersonProjectionStore {
    pub async fn list_personas(&self, limit: i64) -> Result<Vec<Person>, PersonProjectionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            ORDER BY updated_at DESC, created_at DESC, person_id
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(row_to_person).collect()
    }

    pub async fn get_persona(
        &self,
        persona_id: &str,
    ) -> Result<Option<Person>, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_person).transpose()
    }
}
```

### `backend/src/domains/persons/api/store/persona_type.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store/persona_type.rs`
- Size bytes / Размер в байтах: `1050`
- Included characters / Включено символов: `1050`
- Truncated / Обрезано: `no`

```rust
use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::{Person, PersonaType};
use crate::domains::persons::api::rows::row_to_person;

impl PersonProjectionStore {
    pub async fn set_persona_type(
        &self,
        person_id: &str,
        persona_type: PersonaType,
    ) -> Result<Person, PersonProjectionError> {
        let row = sqlx::query(
            r#"
            UPDATE persons
            SET person_type = $2, updated_at = now()
            WHERE person_id = $1
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .bind(persona_type.as_str())
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(person_id.to_owned()))?;

        row_to_person(row)
    }
}
```

### `backend/src/domains/persons/api/store/persona_writes.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store/persona_writes.rs`
- Size bytes / Размер в байтах: `4673`
- Included characters / Включено символов: `4673`
- Truncated / Обрезано: `no`

```rust
use super::PersonProjectionStore;
use super::owner::assign_owner_persona_in_transaction;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;
use crate::domains::persons::api::validation::validate_display_name;
use crate::domains::persons::core::link_persons_entity_in_transaction;

impl PersonProjectionStore {
    pub async fn update_persona(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;

        if let Some(display_name) = display_name {
            let display_name = validate_display_name(display_name)?;
            let result = sqlx::query(
                r#"
                UPDATE persons
                SET display_name = $2, updated_at = now()
                WHERE person_id = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonProjectionError::PersonNotFound(persona_id.to_owned()));
            }

            sqlx::query(
                r#"
                UPDATE graph_nodes
                SET label = $2, updated_at = now()
                WHERE node_kind = $3 AND stable_key = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .bind("person")
            .execute(&mut *transaction)
            .await?;
        }

        if set_self {
            assign_owner_persona_in_transaction(&mut transaction, persona_id).await?;
        }

        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(persona_id.to_owned()))?;

        let person = row_to_person(row)?;
        transaction.commit().await?;
        Ok(person)
    }

    pub async fn update_persona_with_observation(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
        observation_id: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;

        if let Some(display_name) = display_name {
            let display_name = validate_display_name(display_name)?;
            let result = sqlx::query(
                r#"
                UPDATE persons
                SET display_name = $2, updated_at = now()
                WHERE person_id = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                return Err(PersonProjectionError::PersonNotFound(persona_id.to_owned()));
            }

            sqlx::query(
                r#"
                UPDATE graph_nodes
                SET label = $2, updated_at = now()
                WHERE node_kind = $3 AND stable_key = $1
                "#,
            )
            .bind(persona_id)
            .bind(&display_name)
            .bind("person")
            .execute(&mut *transaction)
            .await?;
        }

        if set_self {
            assign_owner_persona_in_transaction(&mut transaction, persona_id).await?;
        }

        let row = sqlx::query(
            r#"
            SELECT
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            FROM persons
            WHERE person_id = $1
            "#,
        )
        .bind(persona_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or_else(|| PersonProjectionError::PersonNotFound(persona_id.to_owned()))?;

        let person = row_to_person(row)?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            persona_id,
            Some("persona_update"),
            None,
        )
        .await?;
        transaction.commit().await?;
        Ok(person)
    }
}
```

### `backend/src/domains/persons/api/store/review_projection.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/store/review_projection.rs`
- Size bytes / Размер в байтах: `2779`
- Included characters / Включено символов: `2779`
- Truncated / Обрезано: `no`

```rust
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use super::PersonProjectionStore;
use crate::domains::persons::api::errors::PersonProjectionError;
use crate::domains::persons::api::models::Person;
use crate::domains::persons::api::rows::row_to_person;

impl PersonProjectionStore {
    pub async fn upsert_review_person(
        &self,
        person_id: &str,
        display_name: &str,
    ) -> Result<Person, PersonProjectionError> {
        let mut transaction = self.pool().begin().await?;
        let person =
            Self::upsert_review_person_in_transaction(&mut transaction, person_id, display_name)
                .await?;
        transaction.commit().await?;
        Ok(person)
    }

    pub(crate) async fn upsert_review_person_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        person_id: &str,
        display_name: &str,
    ) -> Result<Person, PersonProjectionError> {
        let person_id = person_id.trim();
        let display_name = display_name.trim();
        if person_id.is_empty() {
            return Err(PersonProjectionError::PersonNotFound(
                "review promoted person_id must not be empty".to_owned(),
            ));
        }
        if display_name.is_empty() {
            return Err(PersonProjectionError::EmptyDisplayName);
        }

        let synthetic_email = format!("{person_id}@makosh.invalid");
        let row = sqlx::query(
            r#"
            INSERT INTO persons (
                person_id,
                display_name,
                email_address,
                person_type,
                is_self
            )
            VALUES ($1, $2, $3, 'human', false)
            ON CONFLICT (person_id)
            DO UPDATE SET
                display_name = EXCLUDED.display_name,
                updated_at = now()
            RETURNING
                person_id,
                display_name,
                email_address,
                person_type,
                is_self,
                created_at,
                updated_at
            "#,
        )
        .bind(person_id)
        .bind(display_name)
        .bind(&synthetic_email)
        .fetch_one(&mut **transaction)
        .await?;
        let person = row_to_person(row)?;

        sqlx::query(
            r#"
            INSERT INTO person_personas (
                persona_id,
                person_id,
                name
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (persona_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = now()
            "#,
        )
        .bind(person_id)
        .bind(person_id)
        .bind(display_name)
        .execute(&mut **transaction)
        .await?;

        Ok(person)
    }
}
```

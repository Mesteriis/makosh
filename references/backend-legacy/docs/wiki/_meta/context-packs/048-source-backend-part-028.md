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

- Chunk ID / ID чанка: `048-source-backend-part-028`
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

### `backend/src/domains/persons/api/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/api/validation.rs`
- Size bytes / Размер в байтах: `2002`
- Included characters / Включено символов: `2002`
- Truncated / Обрезано: `no`

```rust
use super::errors::PersonProjectionError;

pub(super) fn normalize_email_address(
    email_address: &str,
) -> Result<String, PersonProjectionError> {
    let normalized_email = email_addr_spec(email_address).trim().to_ascii_lowercase();
    if normalized_email.is_empty() {
        return Err(PersonProjectionError::EmptyEmailAddress);
    }
    if !normalized_email.contains('@') {
        return Err(PersonProjectionError::InvalidEmailAddress(normalized_email));
    }

    Ok(normalized_email)
}

fn email_addr_spec(value: &str) -> &str {
    let value = value.trim();
    if let Some((_, tail)) = value.rsplit_once('<')
        && let Some((addr, _)) = tail.split_once('>')
    {
        return addr.trim();
    }
    value.trim_matches('"')
}

pub(super) fn person_id_for_email(normalized_email: &str) -> String {
    let mut encoded = String::from("person:v1:email:");
    encoded.push_str(&normalized_email.len().to_string());
    encoded.push(':');
    encoded.push_str(normalized_email);
    encoded
}

pub(super) fn normalize_ai_agent_id(agent_id: &str) -> Result<String, PersonProjectionError> {
    let normalized = agent_id.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(PersonProjectionError::EmptyAiAgentId);
    }
    if !normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(PersonProjectionError::InvalidAiAgentId(agent_id.to_owned()));
    }
    Ok(normalized)
}

pub(super) fn validate_display_name(display_name: &str) -> Result<String, PersonProjectionError> {
    let display_name = display_name.trim();
    if display_name.is_empty() {
        return Err(PersonProjectionError::EmptyDisplayName);
    }
    Ok(display_name.to_owned())
}

pub(super) fn ai_agent_person_id(agent_id: &str) -> String {
    format!("persona:v1:ai_agent:{agent_id}")
}

pub(super) fn ai_agent_email_address(agent_id: &str) -> String {
    format!("{}@sh-inc.ru", agent_id.to_ascii_lowercase())
}
```

### `backend/src/domains/persons/command_service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/command_service.rs`
- Size bytes / Размер в байтах: `29261`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, ObservationStoreError,
};

use super::api::{Person, PersonProjectionError, PersonProjectionStore};
use super::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use super::enrichment::{PersonEnrichmentError, PersonEnrichmentStore};
use super::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use super::health::{PersonHealthError, PersonHealthStore};
use super::identity::{
    PersonIdentityError, PersonIdentityReviewCommand, PersonIdentityReviewCommandResult,
    PersonIdentityStore,
};
use super::intelligence::{PersonIntelligenceService, PersonMessage};
use super::investigator::{
    DossierReviewState, DossierSnapshot, InvestigatorError, PersonInvestigator,
};
use super::memory::{
    NewRelationshipEvent, PersonFact, PersonFactStore, PersonMemoryCard, PersonMemoryCardStore,
    PersonMemoryError, PersonPreference, PersonPreferenceStore, RelationshipEvent,
    RelationshipEventStore,
};

#[derive(Clone)]
pub struct PersonCommandService {
    pool: PgPool,
}

impl PersonCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_identity_trace_manual(
        &self,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<PersonIdentity, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("person-identity://trace/{identity_type}/{identity_value}"),
                json!({
                    "captured_by": "persons_service.create_identity_trace_manual",
                    "operation": "create_identity_trace_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .create_unattached_with_observation(
                identity_type,
                identity_value,
                &manual_record_source(requested_source, &observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn assign_identity_trace_manual(
        &self,
        identity_id: &str,
        person_id: &str,
    ) -> Result<PersonIdentity, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "identity_id": identity_id,
                    "person_id": person_id,
                    "action": "attach_identity_trace",
                }),
                format!("person-identity://trace/{identity_id}/assignment"),
                json!({
                    "captured_by": "persons_service.assign_identity_trace_manual",
                    "operation": "assign_identity_trace_manual",
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .attach_to_persona_with_observation(identity_id, person_id, &observation.observation_id)
            .await?)
    }

    pub async fn upsert_person_identity_manual(
        &self,
        person_id: &str,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<PersonIdentity, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("person://{person_id}/identities/{identity_type}"),
                json!({
                    "captured_by": "persons_service.upsert_person_identity_manual",
                    "operation": "upsert_person_identity_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .upsert_with_observation(
                person_id,
                identity_type,
                identity_value,
                &manual_record_source(requested_source, &observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn delete_person_identity_manual(
        &self,
        person_id: &str,
        identity_id: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "identity_id": identity_id,
                    "action": "delete_identity",
                }),
                format!("person://{person_id}/identities/{identity_id}/delete"),
                json!({
                    "captured_by": "persons_service.delete_person_identity_manual",
                    "operation": "delete_person_identity_manual",
                }),
            )
            .await?;

        Ok(PersonsIdentityStore::new(self.pool.clone())
            .delete_with_observation(person_id, identity_id, &observation.observation_id)
            .await?)
    }

    pub async fn assign_role_manual(
        &self,
        person_id: &str,
        role: &str,
    ) -> Result<PersonRole, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "role": role,
                    "action": "assign_role",
                }),
                format!("person://{person_id}/roles/{role}"),
                json!({
                    "captured_by": "persons_service.assign_role_manual",
                    "operation": "assign_role_manual",
                }),
            )
            .await?;

        Ok(PersonRoleStore::new(self.pool.clone())
            .assign_with_observation(person_id, role, None, Some(&observation.observation_id))
            .await?)
    }

    pub async fn remove_role_manual(
        &self,
        person_id: &str,
        role: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "role": role,
                    "action": "remove_role",
                }),
                format!("person://{person_id}/roles/{role}/delete"),
                json!({
                    "captured_by": "persons_service.remove_role_manual",
                    "operation": "remove_role_manual",
                }),
            )
            .await?;

        Ok(PersonRoleStore::new(self.pool.clone())
            .remove_with_observation(person_id, role, Some(&observation.observation_id))
            .await?)
    }

    pub async fn upsert_person_persona_manual(
        &self,
        persona: &NewPersonPersona,
    ) -> Result<PersonPersona, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": persona.person_id,
                    "persona_id": persona.persona_id,
                    "name": persona.name,
                    "context": persona.context,
                    "default_tone": persona.default_tone,
                    "default_language": persona.default_language,
                    "preferred_channel": persona.preferred_channel,
                    "action": "upsert_persona",
                }),
                format!(
                    "person://{}/personas/{}",
                    persona.person_id, persona.persona_id
                ),
                json!({
                    "captured_by": "persons_service.upsert_person_persona_manual",
                    "operation": "upsert_person_persona_manual",
                }),
            )
            .await?;

        Ok(PersonPersonaStore::new(self.pool.clone())
            .upsert_with_observation(
                persona,
                Some(&format!("observation:{}", observation.observation_id)),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn delete_person_persona_manual(
        &self,
        person_id: &str,
        persona_id: &str,
    ) -> Result<bool, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "persona_id": persona_id,
                    "action": "delete_persona",
                }),
                format!("person://{person_id}/personas/{persona_id}/delete"),
                json!({
                    "captured_by": "persons_service.delete_person_persona_manual",
                    "operation": "delete_person_persona_manual",
                }),
            )
            .await?;

        Ok(PersonPersonaStore::new(self.pool.clone())
            .delete_with_observation(
                person_id,
                persona_id,
                Some(&format!("observation:{}", observation.observation_id)),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_person_fact_manual(
        &self,
        person_id: &str,
        fact_type: &str,
        value: &str,
        requested_source: &str,
        confidence: f64,
    ) -> Result<PersonFact, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "person_id": person_id,
                    "fact_type": fact_type,
                    "value": value,
                    "source": requested_source,
                    "confidence": confidence,
                }),
                format!("person://{person_id}/facts/{fact_type}"),
                json!({
                    "captured_by": "persons_service.upsert_person_fact_manual",
                    "operation": "upsert_person_fact_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonFactStore::new(self.pool.clone())
            .upsert_with_observation(
                person_id,
                fact_type,
                value,
                &format!("observation:{}", observation.observation_id),
                confidence,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_person_memory_card_manual(
        &self,
        person_id: &str,
        title: &str,
        description: &str,
        requested_source: &str,
        importance: i16,
    ) -> Result<PersonMemoryCard, PersonCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSON_MEMORY_CARD",
                Utc::now(),

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/domains/persons/core.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/core.rs`
- Size bytes / Размер в байтах: `525`
- Included characters / Включено символов: `525`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod evidence;
mod identities;
mod interaction_contexts;
mod preferences;
mod roles;

pub use errors::PersonCoreError;
pub(crate) use evidence::{link_persons_entity, link_persons_entity_in_transaction};
pub use identities::{PersonIdentity, PersonsIdentityStore};
pub use interaction_contexts::{NewPersonPersona, PersonPersona, PersonPersonaStore};
pub(crate) use roles::person_role_knowledge_id;
pub use roles::{
    PERSON_ROLE_ASSIGNED_EVENT_TYPE, PERSON_ROLE_REMOVED_EVENT_TYPE, PersonRole, PersonRoleStore,
};
```

### `backend/src/domains/persons/core/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/core/errors.rs`
- Size bytes / Размер в байтах: `504`
- Included characters / Включено символов: `504`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::EventStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonCoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),

    #[error("person identity not found")]
    IdentityNotFound,

    #[error("person persona not found")]
    PersonaNotFound,
}
```

### `backend/src/domains/persons/core/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/core/evidence.rs`
- Size bytes / Размер в байтах: `1216`
- Included characters / Включено символов: `1216`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::{PgPool, Postgres};

use crate::platform::observations::{
    ObservationStoreError, link_domain_entity, link_domain_entity_in_transaction,
};

pub(crate) async fn link_persons_entity(
    pool: &PgPool,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity(
        pool,
        observation_id,
        "persons",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        metadata,
    )
    .await?;
    Ok(())
}

pub(crate) async fn link_persons_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: Option<&str>,
    metadata: Option<Value>,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "persons",
        entity_kind,
        entity_id.into(),
        relationship_kind,
        None,
        metadata,
    )
    .await?;
    Ok(())
}
```

### `backend/src/domains/persons/core/identities.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/core/identities.rs`
- Size bytes / Размер в байтах: `9968`
- Included characters / Включено символов: `9968`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::PersonCoreError;
use super::link_persons_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonIdentity {
    pub id: String,
    pub person_id: Option<String>,
    pub identity_type: String,
    pub identity_value: String,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonsIdentityStore {
    pool: PgPool,
}

impl PersonsIdentityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonIdentity>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, identity_type, identity_value, source,
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM person_identities WHERE person_id = $1 ORDER BY identity_type"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn list_unattached(
        &self,
        limit: i64,
    ) -> Result<Vec<PersonIdentity>, PersonCoreError> {
        let limit = limit.clamp(1, 200);
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, identity_type, identity_value, source,
               confidence::float8 AS confidence,
               last_verified_at, status, metadata, created_at, updated_at
               FROM person_identities
               WHERE person_id IS NULL
               ORDER BY updated_at DESC, id
               LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_identity).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_identities (person_id, identity_type, identity_value, source)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET updated_at = now()
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(person_id)
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn upsert_with_observation(
        &self,
        person_id: &str,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let identity = self
            .upsert(person_id, identity_type, identity_value, source)
            .await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "identity",
            identity.id.clone(),
            None,
            Some(json!({
                "person_id": identity.person_id,
                "identity_type": identity.identity_type,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn create_unattached(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        self.create_unattached_with_metadata(identity_type, identity_value, source, json!({}))
            .await
    }

    pub async fn create_unattached_with_metadata(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        metadata: Value,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_identities (
                   person_id, identity_type, identity_value, source, metadata
               )
               VALUES (NULL, $1, $2, $3, $4)
               ON CONFLICT (identity_type, identity_value) WHERE status = 'active'
               DO UPDATE SET
                   metadata = person_identities.metadata || EXCLUDED.metadata,
                   updated_at = now()
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_type)
        .bind(identity_value)
        .bind(source)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;
        row_to_identity(row)
    }

    pub async fn create_unattached_with_observation(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let identity = self
            .create_unattached(identity_type, identity_value, source)
            .await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "identity_trace",
            identity.id.clone(),
            None,
            Some(json!({
                "identity_type": identity.identity_type,
                "person_id": identity.person_id,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn create_unattached_with_metadata_and_observation(
        &self,
        identity_type: &str,
        identity_value: &str,
        source: &str,
        metadata: Value,
        observation_id: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let identity = self
            .create_unattached_with_metadata(identity_type, identity_value, source, metadata)
            .await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "identity_trace",
            identity.id.clone(),
            None,
            Some(json!({
                "identity_type": identity.identity_type,
                "person_id": identity.person_id,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn attach_to_persona(
        &self,
        identity_id: &str,
        person_id: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let row = sqlx::query(
            r#"UPDATE person_identities
               SET person_id = $2, status = 'active', updated_at = now()
               WHERE id::text = $1
               RETURNING id::text, person_id, identity_type, identity_value, source,
                         confidence::float8 AS confidence,
                         last_verified_at, status, metadata, created_at, updated_at"#,
        )
        .bind(identity_id)
        .bind(person_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersonCoreError::IdentityNotFound)?;
        row_to_identity(row)
    }

    pub async fn attach_to_persona_with_observation(
        &self,
        identity_id: &str,
        person_id: &str,
        observation_id: &str,
    ) -> Result<PersonIdentity, PersonCoreError> {
        let identity = self.attach_to_persona(identity_id, person_id).await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "identity_trace",
            identity.id.clone(),
            Some("trace_assignment"),
            Some(json!({
                "person_id": identity.person_id,
                "identity_type": identity.identity_type,
            })),
        )
        .await?;
        Ok(identity)
    }

    pub async fn update_status(
        &self,
        identity_id: &str,
        status: &str,
    ) -> Result<(), PersonCoreError> {
        sqlx::query(
            "UPDATE person_identities SET status = $2, updated_at = now() WHERE id::text = $1",
        )
        .bind(identity_id)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, identity_id: &str) -> Result<bool, PersonCoreError> {
        let result = sqlx::query("DELETE FROM person_identities WHERE id::text = $1")
            .bind(identity_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_with_observation(
        &self,
        person_id: &str,
        identity_id: &str,
        observation_id: &str,
    ) -> Result<bool, PersonCoreError> {
        let deleted = self.delete(identity_id).await?;
        link_persons_entity(
            &self.pool,
            observation_id,
            "identity",
            identity_id.to_owned(),
            Some("identity_delete"),
            Some(json!({
                "person_id": person_id,
                "deleted": deleted,
            })),
        )
        .await?;
        Ok(deleted)
    }
}

fn row_to_identity(row: PgRow) -> Result<PersonIdentity, PersonCoreError> {
    Ok(PersonIdentity {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        identity_type: row.try_get("identity_type")?,
        identity_value: row.try_get("identity_value")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        status: row.try_get("status")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/persons/core/interaction_contexts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/core/interaction_contexts.rs`
- Size bytes / Размер в байтах: `7953`
- Included characters / Включено символов: `7953`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use super::errors::PersonCoreError;
use super::link_persons_entity_in_transaction;
use super::preferences::{
    delete_interaction_preferences_in_transaction,
    materialize_interaction_preferences_in_transaction,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonPersona {
    pub persona_id: String,
    pub person_id: String,
    pub name: String,
    pub context: Option<String>,
    pub default_tone: Option<String>,
    pub default_language: Option<String>,
    pub preferred_channel: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonPersonaStore {
    pool: PgPool,
}

impl PersonPersonaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonPersona>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT persona_id, person_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM person_personas WHERE person_id = $1 ORDER BY name"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_persona).collect()
    }

    pub async fn upsert(
        &self,
        persona: &NewPersonPersona,
    ) -> Result<PersonPersona, PersonCoreError> {
        self.upsert_with_source(persona, None).await
    }

    pub async fn upsert_with_source(
        &self,
        persona: &NewPersonPersona,
        source: Option<&str>,
    ) -> Result<PersonPersona, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let persona = Self::upsert_in_transaction(&mut transaction, persona, source).await?;
        transaction.commit().await?;

        Ok(persona)
    }

    pub async fn upsert_with_observation(
        &self,
        persona: &NewPersonPersona,
        source: Option<&str>,
        observation_id: &str,
    ) -> Result<PersonPersona, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let persona = Self::upsert_in_transaction(&mut transaction, persona, source).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            persona.persona_id.clone(),
            None,
            Some(json!({
                "person_id": persona.person_id,
                "action": "upsert",
            })),
        )
        .await?;
        transaction.commit().await?;

        Ok(persona)
    }

    async fn upsert_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        persona: &NewPersonPersona,
        source: Option<&str>,
    ) -> Result<PersonPersona, PersonCoreError> {
        let row = sqlx::query(
            r#"INSERT INTO person_personas (persona_id, person_id, name, context, default_tone,
               default_language, preferred_channel)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (persona_id)
               DO UPDATE SET name = EXCLUDED.name, context = EXCLUDED.context,
                             default_tone = EXCLUDED.default_tone,
                             default_language = EXCLUDED.default_language,
                             preferred_channel = EXCLUDED.preferred_channel,
                             updated_at = now()
               RETURNING persona_id, person_id, name, context, default_tone, default_language,
                         preferred_channel, metadata, created_at, updated_at"#,
        )
        .bind(&persona.persona_id)
        .bind(&persona.person_id)
        .bind(&persona.name)
        .bind(&persona.context)
        .bind(&persona.default_tone)
        .bind(&persona.default_language)
        .bind(&persona.preferred_channel)
        .fetch_one(&mut **transaction)
        .await?;
        let persona = row_to_persona(row)?;

        let source = source
            .map(str::to_owned)
            .unwrap_or_else(|| format!("person_personas:{}", persona.persona_id));
        materialize_interaction_preferences_in_transaction(transaction, &persona, &source).await?;

        Ok(persona)
    }

    pub async fn delete(&self, persona_id: &str) -> Result<bool, PersonCoreError> {
        self.delete_with_source(persona_id, None).await
    }

    pub async fn delete_with_source(
        &self,
        persona_id: &str,
        source: Option<&str>,
    ) -> Result<bool, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let deleted = Self::delete_in_transaction(&mut transaction, persona_id, source).await?;
        transaction.commit().await?;
        Ok(deleted)
    }

    pub async fn delete_with_observation(
        &self,
        person_id: &str,
        persona_id: &str,
        source: Option<&str>,
        observation_id: &str,
    ) -> Result<bool, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let deleted = Self::delete_in_transaction(&mut transaction, persona_id, source).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            persona_id.to_owned(),
            None,
            Some(json!({
                "person_id": person_id,
                "action": "delete",
                "deleted": deleted,
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(deleted)
    }

    async fn delete_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        persona_id: &str,
        source: Option<&str>,
    ) -> Result<bool, PersonCoreError> {
        let existing_persona = sqlx::query(
            r#"SELECT persona_id, person_id, name, context, default_tone, default_language,
               preferred_channel, metadata, created_at, updated_at
               FROM person_personas
               WHERE persona_id = $1
               FOR UPDATE"#,
        )
        .bind(persona_id)
        .fetch_optional(&mut **transaction)
        .await?
        .map(row_to_persona)
        .transpose()?;

        let result = sqlx::query("DELETE FROM person_personas WHERE persona_id = $1")
            .bind(persona_id)
            .execute(&mut **transaction)
            .await?;
        let deleted = result.rows_affected() > 0;

        if let Some(existing_persona) = existing_persona
            && deleted
        {
            let source = source
                .map(str::to_owned)
                .unwrap_or_else(|| format!("person_personas:{}", existing_persona.persona_id));
            delete_interaction_preferences_in_transaction(transaction, &existing_persona, &source)
                .await?;
        }

        Ok(deleted)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewPersonPersona {
    pub persona_id: String,
    pub person_id: String,
    pub name: String,
    pub context: Option<String>,
    pub default_tone: Option<String>,
    pub default_language: Option<String>,
    pub preferred_channel: Option<String>,
}

fn row_to_persona(row: PgRow) -> Result<PersonPersona, PersonCoreError> {
    Ok(PersonPersona {
        persona_id: row.try_get("persona_id")?,
        person_id: row.try_get("person_id")?,
        name: row.try_get("name")?,
        context: row.try_get("context")?,
        default_tone: row.try_get("default_tone")?,
        default_language: row.try_get("default_language")?,
        preferred_channel: row.try_get("preferred_channel")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/persons/core/preferences.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/core/preferences.rs`
- Size bytes / Размер в байтах: `3424`
- Included characters / Включено символов: `3424`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use super::errors::PersonCoreError;
use super::interaction_contexts::PersonPersona;

pub(super) async fn materialize_interaction_preferences_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonPersona,
    source: &str,
) -> Result<(), PersonCoreError> {
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "name",
        Some(persona.name.as_str()),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "context",
        persona.context.as_deref(),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "default_tone",
        persona.default_tone.as_deref(),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "default_language",
        persona.default_language.as_deref(),
        source,
    )
    .await?;
    upsert_interaction_preference_in_transaction(
        transaction,
        persona,
        "preferred_channel",
        persona.preferred_channel.as_deref(),
        source,
    )
    .await?;

    Ok(())
}

pub(super) async fn delete_interaction_preferences_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonPersona,
    source: &str,
) -> Result<(), PersonCoreError> {
    for field in [
        "name",
        "context",
        "default_tone",
        "default_language",
        "preferred_channel",
    ] {
        sqlx::query(
            "DELETE FROM person_preferences
             WHERE person_id = $1 AND preference_type = $2 AND source = $3",
        )
        .bind(&persona.person_id)
        .bind(interaction_context_preference_type(
            &persona.persona_id,
            field,
        ))
        .bind(source)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

async fn upsert_interaction_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    persona: &PersonPersona,
    field: &str,
    value: Option<&str>,
    source: &str,
) -> Result<(), PersonCoreError> {
    let preference_type = interaction_context_preference_type(&persona.persona_id, field);
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        sqlx::query(
            "DELETE FROM person_preferences
             WHERE person_id = $1 AND preference_type = $2 AND source = $3",
        )
        .bind(&persona.person_id)
        .bind(preference_type)
        .bind(source)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO person_preferences (person_id, preference_type, value, source)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (person_id, preference_type)
         DO UPDATE SET value = EXCLUDED.value, source = EXCLUDED.source, updated_at = now()",
    )
    .bind(&persona.person_id)
    .bind(preference_type)
    .bind(value)
    .bind(source)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

fn interaction_context_preference_type(persona_id: &str, field: &str) -> String {
    format!("interaction_context:{persona_id}:{field}")
}

fn interaction_context_source(persona_id: &str) -> String {
    format!("person_personas:{persona_id}")
}
```

### `backend/src/domains/persons/core/roles.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/core/roles.rs`
- Size bytes / Размер в байтах: `7930`
- Included characters / Включено символов: `7930`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::platform::events::{EventStore, EventStoreError, NewEventEnvelope};

use super::errors::PersonCoreError;
use super::link_persons_entity_in_transaction;

pub const PERSON_ROLE_ASSIGNED_EVENT_TYPE: &str = "person.role.assigned";
pub const PERSON_ROLE_REMOVED_EVENT_TYPE: &str = "person.role.removed";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonRole {
    pub id: String,
    pub person_id: String,
    pub role: String,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonRoleStore {
    pool: PgPool,
}

impl PersonRoleStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_person(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonRole>, PersonCoreError> {
        let rows = sqlx::query(
            r#"SELECT id::text, person_id, role, assigned_by, assigned_at
               FROM person_roles WHERE person_id = $1 ORDER BY assigned_at"#,
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_role).collect()
    }

    pub async fn assign(
        &self,
        person_id: &str,
        role: &str,
        assigned_by: Option<&str>,
    ) -> Result<PersonRole, PersonCoreError> {
        self.assign_with_observation(person_id, role, assigned_by, None)
            .await
    }

    pub async fn assign_with_observation(
        &self,
        person_id: &str,
        role: &str,
        assigned_by: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<PersonRole, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"INSERT INTO person_roles (person_id, role, assigned_by)
               VALUES ($1, $2, $3)
               ON CONFLICT (person_id, role) DO UPDATE SET assigned_by = EXCLUDED.assigned_by
               RETURNING id::text, person_id, role, assigned_by, assigned_at"#,
        )
        .bind(person_id)
        .bind(role)
        .bind(assigned_by)
        .fetch_one(&mut *transaction)
        .await?;
        let role = row_to_role(row)?;

        if let Some(observation_id) = observation_id {
            link_persons_entity_in_transaction(
                &mut transaction,
                observation_id,
                "role",
                role.id.clone(),
                None,
                Some(json!({
                    "person_id": person_id,
                    "role": &role.role,
                    "action": "assign",
                })),
            )
            .await?;
        }
        append_role_assigned_event(&mut transaction, &role).await?;
        transaction.commit().await?;

        Ok(role)
    }

    pub async fn remove(&self, person_id: &str, role: &str) -> Result<bool, PersonCoreError> {
        self.remove_with_observation(person_id, role, None).await
    }

    pub async fn remove_with_observation(
        &self,
        person_id: &str,
        role: &str,
        observation_id: Option<&str>,
    ) -> Result<bool, PersonCoreError> {
        let mut transaction = self.pool.begin().await?;
        let existing_role = sqlx::query(
            r#"SELECT id::text, person_id, role, assigned_by, assigned_at
               FROM person_roles
               WHERE person_id = $1 AND role = $2
               FOR UPDATE"#,
        )
        .bind(person_id)
        .bind(role)
        .fetch_optional(&mut *transaction)
        .await?
        .map(row_to_role)
        .transpose()?;

        let result = sqlx::query("DELETE FROM person_roles WHERE person_id = $1 AND role = $2")
            .bind(person_id)
            .bind(role)
            .execute(&mut *transaction)
            .await?;
        let removed = result.rows_affected() > 0;

        if let Some(existing_role) = existing_role.as_ref()
            && removed
            && let Some(observation_id) = observation_id
        {
            link_persons_entity_in_transaction(
                &mut transaction,
                observation_id,
                "role",
                format!("{person_id}:{role}"),
                None,
                Some(json!({
                    "person_id": &existing_role.person_id,
                    "role": &existing_role.role,
                    "action": "delete",
                    "deleted": removed,
                })),
            )
            .await?;
        }

        if let Some(existing_role) = existing_role.as_ref()
            && removed
        {
            append_role_removed_event(&mut transaction, existing_role).await?;
        }

        transaction.commit().await?;

        Ok(removed)
    }
}

fn row_to_role(row: PgRow) -> Result<PersonRole, PersonCoreError> {
    Ok(PersonRole {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        role: row.try_get("role")?,
        assigned_by: row.try_get("assigned_by")?,
        assigned_at: row.try_get("assigned_at")?,
    })
}

pub(crate) fn person_role_knowledge_id(role: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in role.trim().to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_separator = false;
        } else if !slug.is_empty() && !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    while slug.ends_with('_') {
        slug.pop();
    }

    if slug.is_empty() {
        "person_role:unspecified".to_owned()
    } else {
        format!("person_role:{slug}")
    }
}

async fn append_role_assigned_event(
    transaction: &mut Transaction<'_, Postgres>,
    role: &PersonRole,
) -> Result<(), PersonCoreError> {
    let role_knowledge_id = person_role_knowledge_id(&role.role);
    let event = NewEventEnvelope::builder(
        format!(
            "person_role_assigned:{}:{role_knowledge_id}",
            role.person_id
        ),
        PERSON_ROLE_ASSIGNED_EVENT_TYPE,
        role.assigned_at,
        json!({
            "kind": "persons",
            "provider": "makosh",
            "source_id": &role.person_id,
        }),
        json!({
            "kind": "persona",
            "person_id": &role.person_id,
        }),
    )
    .payload(json!({
        "person_id": &role.person_id,
        "role": &role.role,
        "assigned_by": &role.assigned_by,
        "role_knowledge_id": role_knowledge_id,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    match EventStore::append_in_transaction(transaction, &event).await {
        Ok(_) => Ok(()),
        Err(error) if error.is_unique_violation() => Ok(()),
        Err(error) => Err(error.into()),
    }
}

async fn append_role_removed_event(
    transaction: &mut Transaction<'_, Postgres>,
    role: &PersonRole,
) -> Result<(), PersonCoreError> {
    let role_knowledge_id = person_role_knowledge_id(&role.role);
    let event = NewEventEnvelope::builder(
        format!("person_role_removed:{}", Uuid::now_v7()),
        PERSON_ROLE_REMOVED_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "persons",
            "provider": "makosh",
            "source_id": &role.person_id,
        }),
        json!({
            "kind": "persona",
            "person_id": &role.person_id,
        }),
    )
    .payload(json!({
        "person_id": &role.person_id,
        "role": &role.role,
        "assigned_by": &role.assigned_by,
        "role_knowledge_id": role_knowledge_id,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    EventStore::append_in_transaction(transaction, &event).await?;
    Ok(())
}
```

### `backend/src/domains/persons/enrichment.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment.rs`
- Size bytes / Размер в байтах: `301`
- Included characters / Включено символов: `301`
- Truncated / Обрезано: `no`

```rust
mod commands;
mod errors;
mod materialization;
mod models;
mod queries;
mod rows;
mod store;

pub use errors::PersonEnrichmentError;
pub use models::EnrichedPerson;
pub use store::PersonEnrichmentStore;

pub const PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE: &str = "person.enrichment.trust_score_changed";
```

### `backend/src/domains/persons/enrichment/commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment/commands.rs`
- Size bytes / Размер в байтах: `9163`
- Included characters / Включено символов: `9163`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::Row;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::domains::persons::core::link_persons_entity_in_transaction;
use crate::domains::persons::enrichment::PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE;
use crate::domains::persons::intelligence::CommunicationFingerprint;
use crate::platform::events::{EventStore, EventStoreError, NewEventEnvelope};

use super::errors::PersonEnrichmentError;
use super::materialization::{
    sync_favorite_preference_in_transaction, sync_notes_memory_card_in_transaction,
};
use super::models::EnrichedPerson;
use super::rows::{ENRICHED_PERSON_COLUMNS, row_to_enriched};
use super::store::PersonEnrichmentStore;

impl PersonEnrichmentStore {
    pub async fn enrich_person(
        &self,
        person_id: &str,
        fingerprint: &CommunicationFingerprint,
    ) -> Result<EnrichedPerson, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let sql = format!(
            "UPDATE persons SET \
             language = COALESCE($2, persons.language), \
             tone = COALESCE($3, persons.tone), \
             trust_score = COALESCE($4, persons.trust_score), \
             avg_response_hours = COALESCE($5, persons.avg_response_hours), \
             writing_style = COALESCE($6, persons.writing_style), \
             updated_at = now() \
             WHERE person_id = $1 RETURNING {ENRICHED_PERSON_COLUMNS}"
        );
        let row = sqlx::query(&sql)
            .bind(person_id)
            .bind(fingerprint.detected_language.as_deref())
            .bind(fingerprint.typical_tone.as_deref())
            .bind(fingerprint.trust_score)
            .bind(fingerprint.avg_response_hours)
            .bind(fingerprint.writing_style.as_deref())
            .fetch_optional(&mut *transaction)
            .await?;

        let Some(row) = row else {
            return Err(PersonEnrichmentError::NotFound);
        };
        let enriched = row_to_enriched(row)?;
        append_trust_score_changed_event(
            &mut transaction,
            person_id,
            fingerprint.trust_score,
            None,
        )
        .await?;
        transaction.commit().await?;

        Ok(enriched)
    }

    pub async fn enrich_person_with_observation(
        &self,
        person_id: &str,
        fingerprint: &CommunicationFingerprint,
        observation_id: &str,
    ) -> Result<EnrichedPerson, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let sql = format!(
            "UPDATE persons SET \
             language = COALESCE($2, persons.language), \
             tone = COALESCE($3, persons.tone), \
             trust_score = COALESCE($4, persons.trust_score), \
             avg_response_hours = COALESCE($5, persons.avg_response_hours), \
             writing_style = COALESCE($6, persons.writing_style), \
             updated_at = now() \
             WHERE person_id = $1 RETURNING {ENRICHED_PERSON_COLUMNS}"
        );
        let row = sqlx::query(&sql)
            .bind(person_id)
            .bind(fingerprint.detected_language.as_deref())
            .bind(fingerprint.typical_tone.as_deref())
            .bind(fingerprint.trust_score)
            .bind(fingerprint.avg_response_hours)
            .bind(fingerprint.writing_style.as_deref())
            .fetch_optional(&mut *transaction)
            .await?;

        let Some(row) = row else {
            return Err(PersonEnrichmentError::NotFound);
        };
        let enriched = row_to_enriched(row)?;
        append_trust_score_changed_event(
            &mut transaction,
            person_id,
            fingerprint.trust_score,
            Some(observation_id),
        )
        .await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "persona",
            person_id,
            Some("profile_enrichment"),
            Some(serde_json::json!({
                "manual_entrypoint": "post_person_fingerprint"
            })),
        )
        .await?;
        transaction.commit().await?;

        Ok(enriched)
    }

    pub async fn toggle_favorite(&self, person_id: &str) -> Result<bool, PersonEnrichmentError> {
        self.toggle_favorite_with_source(person_id, &format!("persons.is_favorite:{person_id}"))
            .await
    }

    pub async fn toggle_favorite_with_source(
        &self,
        person_id: &str,
        source: &str,
    ) -> Result<bool, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let is_favorite =
            Self::toggle_favorite_in_transaction(&mut transaction, person_id, source).await?;
        transaction.commit().await?;
        Ok(is_favorite)
    }

    pub async fn toggle_favorite_with_observation(
        &self,
        person_id: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<bool, PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        let is_favorite =
            Self::toggle_favorite_in_transaction(&mut transaction, person_id, source).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "favorite_toggle",
            person_id,
            None,
            Some(serde_json::json!({
                "is_favorite": is_favorite
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(is_favorite)
    }

    async fn toggle_favorite_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        person_id: &str,
        source: &str,
    ) -> Result<bool, PersonEnrichmentError> {
        let row = sqlx::query(
            "UPDATE persons SET is_favorite = NOT is_favorite, updated_at = now() \
             WHERE person_id = $1 RETURNING is_favorite",
        )
        .bind(person_id)
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let is_favorite = row.try_get("is_favorite").unwrap_or(false);
        sync_favorite_preference_in_transaction(transaction, person_id, is_favorite, source)
            .await?;
        Ok(is_favorite)
    }

    pub async fn set_notes(
        &self,
        person_id: &str,
        notes: &str,
    ) -> Result<(), PersonEnrichmentError> {
        self.set_notes_with_source(person_id, notes, &format!("persons.notes:{person_id}"))
            .await
    }

    pub async fn set_notes_with_source(
        &self,
        person_id: &str,
        notes: &str,
        source: &str,
    ) -> Result<(), PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::set_notes_in_transaction(&mut transaction, person_id, notes, source).await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn set_notes_with_observation(
        &self,
        person_id: &str,
        notes: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<(), PersonEnrichmentError> {
        let mut transaction = self.pool.begin().await?;
        Self::set_notes_in_transaction(&mut transaction, person_id, notes, source).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "notes",
            person_id,
            None,
            Some(serde_json::json!({
                "manual_entrypoint": "put_person_notes"
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn set_notes_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        person_id: &str,
        notes: &str,
        source: &str,
    ) -> Result<(), PersonEnrichmentError> {
        sqlx::query("UPDATE persons SET notes = $2, updated_at = now() WHERE person_id = $1")
            .bind(person_id)
            .bind(notes)
            .execute(&mut **transaction)
            .await?;
        sync_notes_memory_card_in_transaction(transaction, person_id, notes, source).await?;
        Ok(())
    }
}

async fn append_trust_score_changed_event(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    trust_score: Option<i16>,
    source_observation_id: Option<&str>,
) -> Result<(), PersonEnrichmentError> {
    let Some(trust_score) = trust_score else {
        return Ok(());
    };

    let event = NewEventEnvelope::builder(
        format!("person_trust_score_changed:{}", Uuid::now_v7()),
        PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "person_enrichment",
            "provider": "makosh",
            "source_id": person_id,
        }),
        json!({
            "kind": "persona",
            "person_id": person_id,
        }),
    )
    .payload(json!({
        "person_id": person_id,
        "trust_score": trust_score,
        "source_observation_id": source_observation_id,
    }))
    .build()
    .map_err(EventStoreError::from)?;

    EventStore::append_in_transaction(transaction, &event).await?;
    Ok(())
}
```

### `backend/src/domains/persons/enrichment/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment/errors.rs`
- Size bytes / Размер в байтах: `539`
- Included characters / Включено символов: `539`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::engines::trust::TrustEngineError;
use crate::platform::events::EventStoreError;
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonEnrichmentError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Trust(#[from] TrustEngineError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Event(#[from] EventStoreError),

    #[error("person not found")]
    NotFound,
}
```

### `backend/src/domains/persons/enrichment/materialization.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment/materialization.rs`
- Size bytes / Размер в байтах: `2239`
- Included characters / Включено символов: `2239`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::engines::enrichment::EnrichmentEngine;
use crate::engines::memory::MemoryEngine;

use super::errors::PersonEnrichmentError;

pub(in crate::domains::persons::enrichment) async fn sync_notes_memory_card_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    notes: &str,
    source: &str,
) -> Result<(), PersonEnrichmentError> {
    sqlx::query("DELETE FROM person_memory_cards WHERE person_id = $1 AND source = $2")
        .bind(person_id)
        .bind(source)
        .execute(&mut **transaction)
        .await?;

    let Some(memory_card) = MemoryEngine::persona_notes_memory_card(person_id, notes) else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO person_memory_cards (person_id, title, description, source, confidence, importance)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(person_id)
    .bind(memory_card.title)
    .bind(memory_card.description)
    .bind(source)
    .bind(memory_card.confidence)
    .bind(memory_card.importance)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub(in crate::domains::persons::enrichment) async fn sync_favorite_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    is_favorite: bool,
    source: &str,
) -> Result<(), PersonEnrichmentError> {
    if let Some(preference) = EnrichmentEngine::persona_favorite_preference(person_id, is_favorite)
    {
        sqlx::query(
            "INSERT INTO person_preferences (person_id, preference_type, value, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (person_id, preference_type)
             DO UPDATE SET value = $3, source = $4, confidence = $5, updated_at = now()",
        )
        .bind(person_id)
        .bind(preference.preference_type)
        .bind(preference.value)
        .bind(source)
        .bind(preference.confidence)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    }

    sqlx::query(
        "DELETE FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(person_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
```

### `backend/src/domains/persons/enrichment/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment/models.rs`
- Size bytes / Размер в байтах: `836`
- Included characters / Включено символов: `836`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedPerson {
    pub person_id: String,
    pub display_name: String,
    pub email_address: String,
    pub language: Option<String>,
    pub tone: Option<String>,
    pub trust_score: Option<i16>,
    pub avg_response_hours: Option<f64>,
    pub preferred_channel: Option<String>,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub interaction_count: i32,
    pub frequent_topics: Vec<String>,
    pub writing_style: Option<String>,
    pub person_metadata: Value,
    pub is_favorite: bool,
    pub notes: Option<String>,
    pub linked_projects: Vec<String>,
    pub linked_documents: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### `backend/src/domains/persons/enrichment/queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment/queries.rs`
- Size bytes / Размер в байтах: `2107`
- Included characters / Включено символов: `2107`
- Truncated / Обрезано: `no`

```rust
use super::errors::PersonEnrichmentError;
use super::models::EnrichedPerson;
use super::rows::{ENRICHED_PERSON_COLUMNS, row_to_enriched};
use super::store::PersonEnrichmentStore;

impl PersonEnrichmentStore {
    pub async fn get_enriched(
        &self,
        person_id: &str,
    ) -> Result<Option<EnrichedPerson>, PersonEnrichmentError> {
        let sql = format!("SELECT {ENRICHED_PERSON_COLUMNS} FROM persons WHERE person_id = $1");
        let row = sqlx::query(&sql)
            .bind(person_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_enriched).transpose()
    }

    pub async fn list_enriched(
        &self,
        favorites_only: bool,
        limit: i64,
    ) -> Result<Vec<EnrichedPerson>, PersonEnrichmentError> {
        let limit = limit.clamp(1, 100);
        let sql = if favorites_only {
            format!(
                "SELECT {ENRICHED_PERSON_COLUMNS} FROM persons WHERE is_favorite = true \
                 ORDER BY trust_score DESC NULLS LAST, interaction_count DESC LIMIT $1"
            )
        } else {
            format!(
                "SELECT {ENRICHED_PERSON_COLUMNS} FROM persons \
                 ORDER BY interaction_count DESC, trust_score DESC NULLS LAST LIMIT $1"
            )
        };
        let rows = sqlx::query(&sql).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enriched).collect()
    }

    pub async fn search_persons(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<EnrichedPerson>, PersonEnrichmentError> {
        let pattern = format!("%{}%", query.trim().to_lowercase());
        let sql = format!(
            "SELECT {ENRICHED_PERSON_COLUMNS} FROM persons \
             WHERE lower(display_name) LIKE $1 OR lower(email_address) LIKE $1 \
             ORDER BY interaction_count DESC LIMIT $2"
        );
        let rows = sqlx::query(&sql)
            .bind(&pattern)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(row_to_enriched).collect()
    }
}
```

### `backend/src/domains/persons/enrichment/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment/rows.rs`
- Size bytes / Размер в байтах: `1641`
- Included characters / Включено символов: `1641`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::PersonEnrichmentError;
use super::models::EnrichedPerson;

pub(in crate::domains::persons::enrichment) const ENRICHED_PERSON_COLUMNS: &str = "person_id, display_name, email_address, language, tone, trust_score, avg_response_hours, \
     preferred_channel, last_interaction_at, interaction_count, frequent_topics, writing_style, \
     person_metadata, is_favorite, notes, created_at, updated_at";

pub(in crate::domains::persons::enrichment) fn row_to_enriched(
    row: PgRow,
) -> Result<EnrichedPerson, PersonEnrichmentError> {
    Ok(EnrichedPerson {
        person_id: row.try_get("person_id")?,
        display_name: row.try_get("display_name")?,
        email_address: row.try_get("email_address")?,
        language: row.try_get("language")?,
        tone: row.try_get("tone")?,
        trust_score: row.try_get("trust_score")?,
        avg_response_hours: row.try_get("avg_response_hours")?,
        preferred_channel: row.try_get("preferred_channel")?,
        last_interaction_at: row.try_get("last_interaction_at")?,
        interaction_count: row.try_get("interaction_count")?,
        frequent_topics: serde_json::from_value(row.try_get("frequent_topics")?)
            .unwrap_or_default(),
        writing_style: row.try_get("writing_style")?,
        person_metadata: row.try_get("person_metadata")?,
        is_favorite: row.try_get("is_favorite")?,
        notes: row.try_get("notes")?,
        linked_projects: vec![],
        linked_documents: vec![],
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
```

### `backend/src/domains/persons/enrichment/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment/store.rs`
- Size bytes / Размер в байтах: `244`
- Included characters / Включено символов: `244`
- Truncated / Обрезано: `no`

```rust
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PersonEnrichmentStore {
    pub(in crate::domains::persons::enrichment) pool: PgPool,
}

impl PersonEnrichmentStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### `backend/src/domains/persons/enrichment_engine.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/enrichment_engine.rs`
- Size bytes / Размер в байтах: `5238`
- Included characters / Включено символов: `5238`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::engines::enrichment::{
    EnrichmentEngine, EnrichmentEngineError as SharedEnrichmentEngineError,
};
use crate::platform::observations::{
    ObservationStoreError, materialize_review_transition_link as materialize_review_link,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichmentResult {
    pub id: String,
    pub person_id: String,
    pub source: String,
    pub url: Option<String>,
    pub data: Value,
    pub confidence: f64,
    pub status: String,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EnrichmentResultStore {
    pool: PgPool,
}

impl EnrichmentResultStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        person_id: &str,
    ) -> Result<Vec<EnrichmentResult>, EnrichmentEngineError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, source, url, data, confidence::float8 AS confidence, status, last_checked_at, applied_at, created_at
             FROM enrichment_results WHERE person_id = $1 ORDER BY created_at DESC"
        ).bind(person_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_enrichment).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        source: &str,
        data: Value,
        confidence: f64,
    ) -> Result<EnrichmentResult, EnrichmentEngineError> {
        let extracted_claim = extracted_claim_from_data(&data)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| source.to_owned());
        let candidate = EnrichmentEngine::persona_observation_candidate(
            person_id,
            source,
            &extracted_claim,
            data,
            confidence,
        )?;

        let row = sqlx::query(
            "INSERT INTO enrichment_results (person_id, source, data, confidence, status)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, source, url, data, confidence::float8 AS confidence, status, last_checked_at, applied_at, created_at"
        ).bind(person_id).bind(&candidate.source).bind(&candidate.data).bind(candidate.confidence).bind(&candidate.review_state).fetch_one(&self.pool).await?;
        row_to_enrichment(row)
    }

    pub async fn apply(&self, id: &str) -> Result<(), EnrichmentEngineError> {
        self.apply_with_observation(id, None, None).await
    }

    pub async fn apply_with_observation(
        &self,
        id: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<(), EnrichmentEngineError> {
        sqlx::query("UPDATE enrichment_results SET status = 'applied', applied_at = now() WHERE id::text = $1")
            .bind(id).execute(&self.pool).await?;
        materialize_review_link(
            &self.pool,
            observation_id,
            "persons",
            "enrichment_result",
            id,
            "status",
            "applied",
            metadata,
        )
        .await?;
        Ok(())
    }

    pub async fn reject(&self, id: &str) -> Result<(), EnrichmentEngineError> {
        self.reject_with_observation(id, None, None).await
    }

    pub async fn reject_with_observation(
        &self,
        id: &str,
        observation_id: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<(), EnrichmentEngineError> {
        sqlx::query("UPDATE enrichment_results SET status = 'rejected' WHERE id::text = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        materialize_review_link(
            &self.pool,
            observation_id,
            "persons",
            "enrichment_result",
            id,
            "status",
            "rejected",
            metadata,
        )
        .await?;
        Ok(())
    }
}

fn extracted_claim_from_data(data: &Value) -> Option<&str> {
    data.get("extracted_claim")
        .or_else(|| data.get("claim"))
        .or_else(|| data.get("value"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|claim| !claim.is_empty())
}

fn row_to_enrichment(row: PgRow) -> Result<EnrichmentResult, EnrichmentEngineError> {
    Ok(EnrichmentResult {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        source: row.try_get("source")?,
        url: row.try_get("url")?,
        data: row.try_get("data")?,
        confidence: row.try_get("confidence")?,
        status: row.try_get("status")?,
        last_checked_at: row.try_get("last_checked_at")?,
        applied_at: row.try_get("applied_at")?,
        created_at: row.try_get("created_at")?,
    })
}

#[derive(Debug, Error)]
pub enum EnrichmentEngineError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Shared(#[from] SharedEnrichmentEngineError),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
    #[error("enrichment not found")]
    NotFound,
}
```

### `backend/src/domains/persons/expertise.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/expertise.rs`
- Size bytes / Размер в байтах: `3605`
- Included characters / Включено символов: `3605`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonExpertise {
    pub id: String,
    pub person_id: String,
    pub skill: String,
    pub domain: Option<String>,
    pub evidence: Option<String>,
    pub source: String,
    pub confidence: f64,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub endorsed_by_person_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct PersonExpertiseStore {
    pool: PgPool,
}

impl PersonExpertiseStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        person_id: &str,
    ) -> Result<Vec<PersonExpertise>, PersonExpertiseError> {
        let rows = sqlx::query(
            "SELECT id::text, person_id, skill, domain, evidence, source, confidence::float8 AS confidence,
             last_verified_at, endorsed_by_person_id, created_at, updated_at
             FROM person_expertise WHERE person_id = $1 ORDER BY confidence DESC",
        )
        .bind(person_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_expertise).collect()
    }

    pub async fn search_by_skill(
        &self,
        skill: &str,
        limit: i64,
    ) -> Result<Vec<PersonExpertise>, PersonExpertiseError> {
        let pattern = format!("%{}%", skill.trim().to_lowercase());
        let rows = sqlx::query(
            "SELECT id::text, person_id, skill, domain, evidence, source, confidence::float8 AS confidence,
             last_verified_at, endorsed_by_person_id, created_at, updated_at
             FROM person_expertise WHERE lower(skill) LIKE $1 ORDER BY confidence DESC LIMIT $2",
        )
        .bind(&pattern)
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_expertise).collect()
    }

    pub async fn upsert(
        &self,
        person_id: &str,
        skill: &str,
        domain: Option<&str>,
        source: &str,
        confidence: f64,
    ) -> Result<PersonExpertise, PersonExpertiseError> {
        let row = sqlx::query(
            "INSERT INTO person_expertise (person_id, skill, domain, source, confidence)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT DO NOTHING
             RETURNING id::text, person_id, skill, domain, evidence, source, confidence::float8 AS confidence,
                       last_verified_at, endorsed_by_person_id, created_at, updated_at",
        )
        .bind(person_id)
        .bind(skill)
        .bind(domain)
        .bind(source)
        .bind(confidence)
        .fetch_one(&self.pool)
        .await?;
        row_to_expertise(row)
    }
}

fn row_to_expertise(row: PgRow) -> Result<PersonExpertise, PersonExpertiseError> {
    Ok(PersonExpertise {
        id: row.try_get("id")?,
        person_id: row.try_get("person_id")?,
        skill: row.try_get("skill")?,
        domain: row.try_get("domain")?,
        evidence: row.try_get("evidence")?,
        source: row.try_get("source")?,
        confidence: row.try_get("confidence")?,
        last_verified_at: row.try_get("last_verified_at")?,
        endorsed_by_person_id: row.try_get("endorsed_by_person_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum PersonExpertiseError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/domains/persons/export.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/export.rs`
- Size bytes / Размер в байтах: `4341`
- Included characters / Включено символов: `4341`
- Truncated / Обрезано: `no`

```rust
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::persons::investigator::{InvestigatorError, PersonDossier, PersonInvestigator};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExportFormat {
    Markdown,
    Json,
    Pdf,
}

impl ExportFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Markdown => "text/markdown",
            Self::Json => "application/json",
            Self::Pdf => "application/pdf",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Json => "json",
            Self::Pdf => "pdf",
        }
    }
}

#[derive(Clone)]
pub struct PersonExportService {
    pool: PgPool,
}

impl PersonExportService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Export a person dossier in the requested format.
    pub async fn export(
        &self,
        person_id: &str,
        format: ExportFormat,
    ) -> Result<String, ExportError> {
        let investigator = PersonInvestigator::new(self.pool.clone());
        let dossier = investigator.assemble_dossier(person_id).await?;
        match format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(&dossier)?),
            ExportFormat::Markdown => Ok(render_markdown(&dossier)),
            ExportFormat::Pdf => {
                // PDF rendering requires external tooling; return Markdown for now
                Ok(render_markdown(&dossier))
            }
        }
    }
}

fn render_markdown(d: &PersonDossier) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", d.person.display_name));
    md.push_str(&format!("**Email**: {}\n\n", d.person.email_address));

    if let Some(role) = &d.person.tone {
        md.push_str(&format!("**Tone**: {role}\n"));
    }
    if let Some(lang) = &d.person.language {
        md.push_str(&format!("**Language**: {lang}\n"));
    }
    if let Some(score) = d.person.trust_score {
        md.push_str(&format!("**Trust**: {score}/100\n"));
    }
    md.push_str(&format!(
        "**Interactions**: {}\n\n",
        d.person.interaction_count
    ));

    if !d.person.frequent_topics.is_empty() {
        md.push_str("## Topics\n\n");
        for t in &d.person.frequent_topics {
            md.push_str(&format!("- {t}\n"));
        }
        md.push('\n');
    }

    if !d.memory_cards.is_empty() {
        md.push_str("## Memory Cards\n\n");
        for card in &d.memory_cards {
            md.push_str(&format!(
                "- **{}**: {} (importance: {})\n",
                card.title, card.description, card.importance
            ));
        }
        md.push('\n');
    }

    if !d.facts.is_empty() {
        md.push_str("## Facts\n\n");
        for fact in &d.facts {
            md.push_str(&format!(
                "- **{}**: {} (source: {}, confidence: {:.0}%)\n",
                fact.fact_type,
                fact.value,
                fact.source,
                fact.confidence * 100.0
            ));
        }
        md.push('\n');
    }

    if !d.timeline.is_empty() {
        md.push_str("## Timeline\n\n");
        for event in &d.timeline {
            md.push_str(&format!(
                "- **{}**: {} ({})\n",
                event.occurred_at.format("%Y-%m-%d"),
                event.title,
                event.event_type
            ));
        }
        md.push('\n');
    }

    if let Some(notes) = &d.person.notes
        && !notes.is_empty()
    {
        md.push_str(&format!("## Notes\n\n{notes}\n\n"));
    }

    if !d.summary.is_empty() {
        md.push_str(&format!("---\n\n*{summary}*\n", summary = d.summary));
    }

    md
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(transparent)]
    Investigator(#[from] InvestigatorError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("unsupported export format")]
    UnsupportedFormat,
}
```

### `backend/src/domains/persons/health.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/health.rs`
- Size bytes / Размер в байтах: `8475`
- Included characters / Включено символов: `8475`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

use crate::domains::persons::core::link_persons_entity_in_transaction;
use crate::platform::observations::ObservationStoreError;

#[derive(Clone, Debug, Serialize)]
pub struct PersonHealth {
    pub person_id: String,
    pub health_status: String,
    pub last_health_check: Option<DateTime<Utc>>,
    pub communication_gap_days: i32,
    pub watchlist: bool,
    pub interaction_count: i32,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub trust_score: Option<i16>,
    pub open_promises: i64,
    pub open_risks: i64,
}

#[derive(Clone)]
pub struct PersonHealthStore {
    pool: PgPool,
}

impl PersonHealthStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, person_id: &str) -> Result<Option<PersonHealth>, PersonHealthError> {
        let row = sqlx::query(
            r#"SELECT p.person_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               (SELECT count(*) FROM person_promises pp WHERE pp.person_id = p.person_id AND pp.status = 'pending') as open_promises,
               (SELECT count(*) FROM person_risks pr WHERE pr.person_id = p.person_id AND pr.resolved_at IS NULL) as open_risks
               FROM persons p WHERE p.person_id = $1"#
        ).bind(person_id).fetch_optional(&self.pool).await?;
        row.map(|r| PersonHealth {
            person_id: r.try_get("person_id").unwrap_or_default(),
            health_status: r
                .try_get("health_status")
                .unwrap_or_else(|_| "healthy".into()),
            last_health_check: r.try_get("last_health_check").ok(),
            communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
            watchlist: r.try_get("watchlist").unwrap_or(false),
            interaction_count: r.try_get("interaction_count").unwrap_or(0),
            last_interaction_at: r.try_get("last_interaction_at").ok(),
            trust_score: r.try_get("trust_score").ok(),
            open_promises: r.try_get("open_promises").unwrap_or(0),
            open_risks: r.try_get("open_risks").unwrap_or(0),
        })
        .map_or(Ok(None), |h| Ok(Some(h)))
    }

    pub async fn list_health(&self) -> Result<Vec<PersonHealth>, PersonHealthError> {
        let rows = sqlx::query(
            r#"SELECT p.person_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               0::bigint as open_promises, 0::bigint as open_risks
               FROM persons p WHERE p.health_status != 'healthy' ORDER BY p.last_interaction_at DESC NULLS LAST LIMIT 50"#
        ).fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|r| PersonHealth {
                person_id: r.try_get("person_id").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                last_interaction_at: r.try_get("last_interaction_at").ok(),
                trust_score: r.try_get("trust_score").ok(),
                open_promises: r.try_get("open_promises").unwrap_or(0),
                open_risks: r.try_get("open_risks").unwrap_or(0),
            })
            .collect())
    }

    pub async fn list_watchlist(&self) -> Result<Vec<PersonHealth>, PersonHealthError> {
        let rows = sqlx::query(
            r#"SELECT p.person_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               0::bigint as open_promises, 0::bigint as open_risks
               FROM persons p WHERE p.watchlist = true ORDER BY p.trust_score DESC NULLS LAST"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| PersonHealth {
                person_id: r.try_get("person_id").unwrap_or_default(),
                health_status: r
                    .try_get("health_status")
                    .unwrap_or_else(|_| "healthy".into()),
                last_health_check: r.try_get("last_health_check").ok(),
                communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
                watchlist: r.try_get("watchlist").unwrap_or(false),
                interaction_count: r.try_get("interaction_count").unwrap_or(0),
                last_interaction_at: r.try_get("last_interaction_at").ok(),
                trust_score: r.try_get("trust_score").ok(),
                open_promises: r.try_get("open_promises").unwrap_or(0),
                open_risks: r.try_get("open_risks").unwrap_or(0),
            })
            .collect())
    }

    pub async fn toggle_watchlist(&self, person_id: &str) -> Result<bool, PersonHealthError> {
        self.toggle_watchlist_with_source(person_id, &person_watchlist_source(person_id))
            .await
    }

    pub async fn toggle_watchlist_with_source(
        &self,
        person_id: &str,
        source: &str,
    ) -> Result<bool, PersonHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, person_id, source).await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    pub async fn toggle_watchlist_with_observation(
        &self,
        person_id: &str,
        source: &str,
        observation_id: &str,
    ) -> Result<bool, PersonHealthError> {
        let mut transaction = self.pool.begin().await?;
        let watchlist =
            Self::toggle_watchlist_in_transaction(&mut transaction, person_id, source).await?;
        link_persons_entity_in_transaction(
            &mut transaction,
            observation_id,
            "watchlist_toggle",
            person_id,
            None,
            Some(json!({
                "watchlist": watchlist
            })),
        )
        .await?;
        transaction.commit().await?;
        Ok(watchlist)
    }

    async fn toggle_watchlist_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        person_id: &str,
        source: &str,
    ) -> Result<bool, PersonHealthError> {
        let row = sqlx::query(
            "UPDATE persons SET watchlist = NOT watchlist WHERE person_id = $1 RETURNING watchlist",
        )
        .bind(person_id)
        .fetch_optional(&mut **transaction)
        .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let watchlist = row.try_get("watchlist").unwrap_or(false);
        sync_watchlist_preference_in_transaction(transaction, person_id, watchlist, source).await?;
        Ok(watchlist)
    }
}

async fn sync_watchlist_preference_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    person_id: &str,
    watchlist: bool,
    source: &str,
) -> Result<(), PersonHealthError> {
    if watchlist {
        sqlx::query(
            "INSERT INTO person_preferences (person_id, preference_type, value, source, confidence)
             VALUES ($1, 'ui:watchlist', 'true', $2, 1.0)
             ON CONFLICT (person_id, preference_type)
             DO UPDATE SET value = 'true', source = $2, confidence = 1.0, updated_at = now()",
        )
        .bind(person_id)
        .bind(source)
        .execute(&mut **transaction)
        .await?;
        return Ok(());
    }

    sqlx::query(
        "DELETE FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:watchlist'",
    )
    .bind(person_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

fn person_watchlist_source(person_id: &str) -> String {
    format!("persons.watchlist:{person_id}")
}

#[derive(Debug, Error)]
pub enum PersonHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Observation(#[from] ObservationStoreError),
}
```

### `backend/src/domains/persons/identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity.rs`
- Size bytes / Размер в байтах: `673`
- Included characters / Включено символов: `673`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod errors;
mod events;
mod models;
mod rows;
mod store;
mod upsert;
mod validation;

pub use errors::PersonIdentityError;
pub(crate) use models::PersonIdentityCandidatePayload;
pub use models::{
    PersonIdentityCandidate, PersonIdentityCandidateKind, PersonIdentityDetail,
    PersonIdentityReviewCommand, PersonIdentityReviewCommandResult, PersonIdentityReviewState,
};
pub use store::PersonIdentityStore;
pub use store::PersonIdentityStore as PersonIdentityPort;
pub(crate) use upsert::{
    load_identity_candidate_payload, parse_person_identity_candidate_kind,
    parse_person_identity_review_state, person_identity_candidate_detected_event_type,
};
```

### `backend/src/domains/persons/identity/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/constants.rs`
- Size bytes / Размер в байтах: `539`
- Included characters / Включено символов: `539`
- Truncated / Обрезано: `no`

```rust
pub(super) const PERSON_IDENTITY_REVIEW_EVENT_TYPE: &str = "person_identity.review_state_changed";
pub(super) const PERSON_IDENTITY_REVIEW_SOURCE_KIND: &str = "person_identity_review";
pub(super) const PERSON_IDENTITY_REVIEW_SOURCE_PROVIDER: &str = "local_api";
pub(super) const PERSON_IDENTITY_REVIEW_PREFIX: &str = "person_identity_review:";
pub(super) const PERSON_IDENTITY_ID_PREFIX: &str = "identity_candidate:v1:";
pub(super) const DEFAULT_LIMIT: i64 = 50;
pub(super) const MAX_LIMIT: i64 = 100;
pub(super) const MIN_LIMIT: i64 = 1;
```

### `backend/src/domains/persons/identity/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/errors.rs`
- Size bytes / Размер в байтах: `1177`
- Included characters / Включено символов: `1177`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::observations::ObservationStoreError;

#[derive(Debug, Error)]
pub enum PersonIdentityError {
    #[error("limit must be between 1 and 100")]
    InvalidLimit,

    #[error("field must not be empty: {0}")]
    EmptyField(String),

    #[error("candidate kind is not supported: {0}")]
    InvalidCandidateKind(String),

    #[error("review_state must be suggested, user_confirmed, or user_rejected")]
    InvalidReviewState(String),

    #[error("candidate was not found")]
    IdentityCandidateNotFound,

    #[error("payload must be an object")]
    InvalidPayload(String),

    #[error("payload field was missing: {0}")]
    MissingPayloadField(String),

    #[error("actor_id is missing from event")]
    MissingActorId,

    #[error("invalid review event type")]
    InvalidEventType,

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    EventEnvelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Observation(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/domains/persons/identity/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/domains/persons/identity/events.rs`
- Size bytes / Размер в байтах: `2184`
- Included characters / Включено символов: `2184`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use crate::platform::events::NewEventEnvelope;

use super::constants::{
    PERSON_IDENTITY_REVIEW_EVENT_TYPE, PERSON_IDENTITY_REVIEW_SOURCE_KIND,
    PERSON_IDENTITY_REVIEW_SOURCE_PROVIDER,
};
use super::errors::PersonIdentityError;
use super::models::PersonIdentityReviewState;
use super::validation::{as_object, required_payload_string};

pub(super) struct ReviewCommandEvent {
    pub(super) command_id: String,
    pub(super) identity_candidate_id: String,
    pub(super) review_state: PersonIdentityReviewState,
    pub(super) actor_id: String,
    pub(super) event_id: String,
    pub(super) occurred_at: DateTime<Utc>,
}

impl ReviewCommandEvent {
    pub(super) fn to_event(&self) -> Result<NewEventEnvelope, PersonIdentityError> {
        Ok(NewEventEnvelope::builder(
            self.event_id.clone(),
            PERSON_IDENTITY_REVIEW_EVENT_TYPE,
            self.occurred_at,
            json!({
                "kind": PERSON_IDENTITY_REVIEW_SOURCE_KIND,
                "provider": PERSON_IDENTITY_REVIEW_SOURCE_PROVIDER,
                "source_id": self.command_id.clone(),
            }),
            json!({
                "kind": "person_identity_review",
            }),
        )
        .actor(json!({ "actor_id": self.actor_id.clone() }))
        .payload(self.review_payload())
        .build()?)
    }

    fn review_payload(&self) -> Value {
        json!({
            "identity_candidate_id": self.identity_candidate_id,
            "review_state": self.review_state.as_str(),
        })
    }
}

#[derive(Debug)]
pub(super) struct ReviewEvent {
    pub(super) identity_candidate_id: String,
    pub(super) review_state: PersonIdentityReviewState,
}

impl ReviewEvent {
    pub(super) fn from_payload(payload: &Value) -> Result<Self, PersonIdentityError> {
        let payload = as_object(payload)?;
        Ok(Self {
            identity_candidate_id: required_payload_string(payload, "identity_candidate_id")?,
            review_state: PersonIdentityReviewState::parse(required_payload_string(
                payload,
                "review_state",
            )?)?,
        })
    }
}
```

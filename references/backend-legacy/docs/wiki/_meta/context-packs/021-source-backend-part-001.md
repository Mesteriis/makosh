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

- Chunk ID / ID чанка: `021-source-backend-part-001`
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

### `backend/build.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/build.rs`
- Size bytes / Размер в байтах: `815`
- Included characters / Включено символов: `815`
- Truncated / Обрезано: `no`

```rust
fn main() {
    println!("cargo:rerun-if-changed=../contracts/proto/makosh/common/v1/common.proto");
    println!("cargo:rerun-if-changed=../contracts/proto/makosh/signal_hub/v1/signal_hub.proto");
    println!(
        "cargo:rerun-if-changed=../contracts/proto/makosh/communications/v1/communications.proto"
    );
    println!("cargo:rerun-if-changed=../contracts/proto");
    connectrpc_build::Config::new()
        .files(&[
            "../contracts/proto/makosh/common/v1/common.proto",
            "../contracts/proto/makosh/signal_hub/v1/signal_hub.proto",
            "../contracts/proto/makosh/communications/v1/communications.proto",
        ])
        .includes(&["../contracts/proto"])
        .include_file("_connectrpc.rs")
        .compile()
        .expect("connectrpc codegen should succeed");
}
```

### `backend/src/ai/api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/api.rs`
- Size bytes / Размер в байтах: `571`
- Included characters / Включено символов: `571`
- Truncated / Обрезано: `no`

```rust
mod control_center;
mod helpers;
mod models;
mod runtime;

pub(crate) use control_center::{
    get_ai_models, get_ai_prompts, get_ai_providers, get_ai_settings_overview, patch_ai_provider,
    post_ai_prompt, post_ai_prompt_activate, post_ai_prompt_test, post_ai_prompt_version,
    post_ai_provider, post_ai_provider_consent, post_ai_provider_sync_models,
    post_ai_provider_test, put_ai_model_route,
};
pub(crate) use runtime::{
    get_ai_agents, get_ai_run, get_ai_runs, get_ai_status, post_ai_answer, post_ai_meeting_prep,
    post_ai_task_candidates_refresh,
};
```

### `backend/src/ai/api/control_center.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/api/control_center.rs`
- Size bytes / Размер в байтах: `7521`
- Included characters / Включено символов: `7521`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

use crate::ai::control_center::{
    AiControlCenterError, AiModelRoute, AiModelRouteUpdateRequest, AiPromptActivateRequest,
    AiPromptCreateRequest, AiPromptEvalRun, AiPromptTemplate, AiPromptTestRequest, AiPromptVersion,
    AiPromptVersionCreateRequest, AiProviderAccount, AiProviderCommandKind,
    AiProviderCommandResponse, AiProviderConsentRequest, AiProviderCreateRequest,
    AiProviderPatchRequest, AiSettingsOverviewResponse, store_api_key_in_host_vault,
};
use crate::app::{ApiError, AppState};
use crate::vault::{HostVaultError, VaultMode};

use super::helpers::{ai_control_center_store, request_actor_id};
use super::models::{AiModelListResponse, AiPromptListResponse, AiProviderListResponse};

pub(crate) async fn get_ai_settings_overview(
    State(state): State<AppState>,
) -> Result<Json<AiSettingsOverviewResponse>, ApiError> {
    Ok(Json(ai_control_center_store(&state)?.overview().await?))
}

pub(crate) async fn get_ai_providers(
    State(state): State<AppState>,
) -> Result<Json<AiProviderListResponse>, ApiError> {
    Ok(Json(AiProviderListResponse {
        items: ai_control_center_store(&state)?.list_providers().await?,
    }))
}

pub(crate) async fn post_ai_provider(
    State(state): State<AppState>,
    Json(request): Json<AiProviderCreateRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let api_key = request_api_key(&request.api_key);
    if api_key.is_some() {
        ensure_api_key_provider_kind(&request.provider_kind)?;
        ensure_host_vault_unlocked_for_api_key(&state)?;
    }
    let provider = store.create_provider(&request).await?;
    if let Some(api_key) = api_key {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, &api_key).await?;
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn patch_ai_provider(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderPatchRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    let store = ai_control_center_store(&state)?;
    let api_key = request_api_key(&request.api_key);
    if api_key.is_some() {
        let current = store
            .provider(&provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        ensure_api_key_provider_kind(&current.provider_kind)?;
        ensure_host_vault_unlocked_for_api_key(&state)?;
    }
    let provider = store.update_provider(&provider_id, &request).await?;
    if let Some(api_key) = api_key {
        let Some(pool) = state.database.pool() else {
            return Err(ApiError::DatabaseNotConfigured);
        };
        store_api_key_in_host_vault(pool, &state.vault, &provider.provider_id, &api_key).await?;
        let Some(provider) = store.provider(&provider.provider_id).await? else {
            return Err(ApiError::NotFound);
        };
        return Ok(Json(provider));
    }

    Ok(Json(provider))
}

pub(crate) async fn post_ai_provider_test(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .provider_command(&provider_id, AiProviderCommandKind::Test)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_sync_models(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<AiProviderCommandResponse>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .provider_command(&provider_id, AiProviderCommandKind::SyncModels)
            .await?,
    ))
}

pub(crate) async fn post_ai_provider_consent(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
    Json(request): Json<AiProviderConsentRequest>,
) -> Result<Json<AiProviderAccount>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .record_consent(&provider_id, &request)
            .await?,
    ))
}

pub(crate) async fn get_ai_models(
    State(state): State<AppState>,
) -> Result<Json<AiModelListResponse>, ApiError> {
    Ok(Json(AiModelListResponse {
        items: ai_control_center_store(&state)?.list_models().await?,
    }))
}

pub(crate) async fn put_ai_model_route(
    State(state): State<AppState>,
    Path(slot): Path<String>,
    Json(request): Json<AiModelRouteUpdateRequest>,
) -> Result<Json<AiModelRoute>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .put_model_route(&slot, &request)
            .await?,
    ))
}

pub(crate) async fn get_ai_prompts(
    State(state): State<AppState>,
) -> Result<Json<AiPromptListResponse>, ApiError> {
    Ok(Json(AiPromptListResponse {
        items: ai_control_center_store(&state)?.list_prompts().await?,
    }))
}

pub(crate) async fn post_ai_prompt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AiPromptCreateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt(&request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptVersionCreateRequest>,
) -> Result<Json<AiPromptVersion>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .create_prompt_version(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_activate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptActivateRequest>,
) -> Result<Json<AiPromptTemplate>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .activate_prompt_version(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

pub(crate) async fn post_ai_prompt_test(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(prompt_id): Path<String>,
    Json(request): Json<AiPromptTestRequest>,
) -> Result<Json<AiPromptEvalRun>, ApiError> {
    Ok(Json(
        ai_control_center_store(&state)?
            .test_prompt(&prompt_id, &request, &request_actor_id(&headers))
            .await?,
    ))
}

fn request_api_key(api_key: &Option<String>) -> Option<String> {
    api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn ensure_api_key_provider_kind(provider_kind: &str) -> Result<(), ApiError> {
    if provider_kind.trim() == "api" {
        return Ok(());
    }
    Err(AiControlCenterError::InvalidRequest(
        "API keys can only be configured for API providers".to_owned(),
    )
    .into())
}

fn ensure_host_vault_unlocked_for_api_key(state: &AppState) -> Result<(), ApiError> {
    match state.vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(HostVaultError::Locked.into()),
        VaultMode::Uninitialized => Err(HostVaultError::Uninitialized.into()),
    }
}
```

### `backend/src/ai/api/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/api/helpers.rs`
- Size bytes / Размер в байтах: `666`
- Included characters / Включено символов: `666`
- Truncated / Обрезано: `no`

```rust
use axum::http::HeaderMap;

use crate::ai::control_center::AiControlCenterStore;
use crate::app::{ApiError, AppState};

pub(super) fn ai_control_center_store(state: &AppState) -> Result<AiControlCenterStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(AiControlCenterStore::new(pool.clone()))
}

pub(super) fn request_actor_id(headers: &HeaderMap) -> String {
    headers
        .get("x-makosh-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("makosh-frontend")
        .to_owned()
}
```

### `backend/src/ai/api/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/api/models.rs`
- Size bytes / Размер в байтах: `447`
- Included characters / Включено символов: `447`
- Truncated / Обрезано: `no`

```rust
use serde::Serialize;

use crate::ai::control_center::{AiModelCatalogItem, AiPromptTemplate, AiProviderAccount};

#[derive(Serialize)]
pub(crate) struct AiProviderListResponse {
    pub(crate) items: Vec<AiProviderAccount>,
}

#[derive(Serialize)]
pub(crate) struct AiModelListResponse {
    pub(crate) items: Vec<AiModelCatalogItem>,
}

#[derive(Serialize)]
pub(crate) struct AiPromptListResponse {
    pub(crate) items: Vec<AiPromptTemplate>,
}
```

### `backend/src/ai/api/runtime.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/api/runtime.rs`
- Size bytes / Размер в байтах: `4807`
- Included characters / Включено символов: `4807`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiMeetingPrepRequest,
    AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::app::api_support::{
    AiRunListResponse, AiRunsQuery, ai_persona_attribution_port_optional, ai_run_store,
    ai_runtime_client, ai_runtime_settings, ai_service,
};
use crate::app::{ApiError, AppState};
pub(crate) async fn get_ai_status(
    State(state): State<AppState>,
) -> Result<Json<AiStatusResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let runtime = ai_runtime_client(&state, &runtime_settings)?;
    let version = runtime.version().await;
    let models = runtime.models().await;
    let chat_model = runtime_settings.chat_model;
    let embedding_model = runtime_settings.embedding_model;
    let chat_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &chat_model))
        .unwrap_or(false);
    let embedding_model_available = models
        .as_ref()
        .map(|models| models.iter().any(|model| model == &embedding_model))
        .unwrap_or(false);

    Ok(Json(AiStatusResponse {
        runtime: runtime.runtime_name().to_owned(),
        status: if version.is_ok()
            && models.is_ok()
            && chat_model_available
            && embedding_model_available
        {
            "ok"
        } else {
            "unavailable"
        }
        .to_owned(),
        version: version.ok().flatten(),
        chat_model,
        embedding_model,
        embedding_dimension: AI_EMBEDDING_DIMENSION,
        chat_model_available,
        embedding_model_available,
    }))
}

pub(crate) async fn get_ai_agents(
    State(state): State<AppState>,
) -> Result<Json<AiAgentListResponse>, ApiError> {
    let runtime_settings = ai_runtime_settings(&state).await?;
    let mut items = v3_agents(&runtime_settings.chat_model);

    if let Some(persona_attribution) = ai_persona_attribution_port_optional(&state) {
        for item in &mut items {
            let persona = persona_attribution
                .upsert_ai_agent_persona(item.agent_id, item.display_name)
                .await
                .map_err(crate::ai::core::AiError::from)?;
            item.persona_id = Some(persona.persona_id);
            item.persona_type = Some(persona.persona_type);
            item.persona_email = Some(persona.persona_email);
        }
    }

    Ok(Json(AiAgentListResponse { items }))
}

pub(crate) async fn get_ai_runs(
    State(state): State<AppState>,
    Query(query): Query<AiRunsQuery>,
) -> Result<Json<AiRunListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let runs = ai_run_store(&state)?.list_runs(limit).await?;

    Ok(Json(AiRunListResponse { items: runs }))
}

pub(crate) async fn get_ai_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<AiAgentRun>, ApiError> {
    let Some(run) = ai_run_store(&state)?.get_run(&run_id).await? else {
        return Err(ApiError::AiRunNotFound);
    };

    Ok(Json(run))
}

pub(crate) async fn post_ai_answer(
    State(state): State<AppState>,
    Json(request): Json<AiAnswerRequest>,
) -> Result<Json<crate::ai::core::AiAnswerResponse>, ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "makosh-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.answer(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_task_candidates_refresh(
    State(state): State<AppState>,
    Json(request): Json<AiTaskCandidateRefreshRequest>,
) -> Result<Json<crate::ai::core::AiTaskCandidateRefreshResponse>, ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "makosh-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.refresh_task_candidates(request, &actor_id).await?;

    Ok(Json(response))
}

pub(crate) async fn post_ai_meeting_prep(
    State(state): State<AppState>,
    Json(request): Json<AiMeetingPrepRequest>,
) -> Result<Json<crate::ai::core::AiMeetingPrepResponse>, ApiError> {
    ensure_ai_requests_allowed(&state).await?;
    let actor_id = "makosh-frontend".to_string();
    let service = ai_service(&state).await?;
    let response = service.meeting_prep(request, &actor_id).await?;

    Ok(Json(response))
}

async fn ensure_ai_requests_allowed(state: &AppState) -> Result<(), ApiError> {
    if crate::app::api_support::ai_requests_allowed(state).await? {
        return Ok(());
    }

    Err(ApiError::FailedPrecondition(
        "AI runtime is disabled by Signal Hub policy or runtime state".to_owned(),
    ))
}
```

### `backend/src/ai/control_center.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center.rs`
- Size bytes / Размер в байтах: `866`
- Included characters / Включено символов: `866`
- Truncated / Обрезано: `no`

```rust
mod availability;
mod catalog;
mod errors;
mod evidence;
mod models;
mod presets;
mod prompts;
mod providers;
mod routes;
mod rows;
mod store;
#[cfg(test)]
mod tests;
mod validation;
mod vault;

pub use errors::AiControlCenterError;
pub use models::{
    AiCapabilitySlot, AiModelCatalogItem, AiModelRoute, AiModelRouteUpdateRequest,
    AiPromptActivateRequest, AiPromptCreateRequest, AiPromptEvalRun, AiPromptTemplate,
    AiPromptTestRequest, AiPromptVersion, AiPromptVersionCreateRequest, AiProviderAccount,
    AiProviderCommandKind, AiProviderCommandResponse, AiProviderConsentRequest,
    AiProviderCreateRequest, AiProviderPatchRequest, AiProviderPreset, AiSettingsOverviewResponse,
};
pub use presets::{BUILT_IN_OLLAMA_PROVIDER_ID, OLLAMA_CHAT_MODEL, OLLAMA_EMBEDDING_MODEL};
pub use store::AiControlCenterStore;
pub use vault::store_api_key_in_host_vault;
```

### `backend/src/ai/control_center/availability.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/availability.rs`
- Size bytes / Размер в байтах: `2989`
- Included characters / Включено символов: `2989`
- Truncated / Обрезано: `no`

```rust
use super::errors::AiControlCenterError;
use super::models::{AiModelCatalogItem, AiProviderAccount};
use super::store::AiControlCenterStore;
use super::validation::validate_non_empty;

mod secrets;

impl AiControlCenterStore {
    pub async fn ensure_model_ready_for_private_context(
        &self,
        provider_id: &str,
        model_key: &str,
    ) -> Result<AiModelCatalogItem, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        validate_non_empty("model_key", model_key)?;
        let model = self
            .model(provider_id, model_key)
            .await?
            .ok_or(AiControlCenterError::ModelNotFound)?;
        if !model.is_available {
            return Err(AiControlCenterError::InvalidRequest(
                "AI model is unavailable".to_owned(),
            ));
        }
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        self.ensure_provider_ready_for_private_context(&provider)
            .await?;
        Ok(model)
    }

    pub async fn model_ready_for_private_context(
        &self,
        provider_id: &str,
        model_key: &str,
    ) -> Result<bool, AiControlCenterError> {
        match self
            .ensure_model_ready_for_private_context(provider_id, model_key)
            .await
        {
            Ok(_) => Ok(true),
            Err(
                AiControlCenterError::InvalidRequest(_)
                | AiControlCenterError::ModelNotFound
                | AiControlCenterError::ProviderNotFound,
            ) => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn ensure_provider_ready_for_private_context(
        &self,
        provider: &AiProviderAccount,
    ) -> Result<(), AiControlCenterError> {
        if provider.status == "disabled" {
            return Err(AiControlCenterError::InvalidRequest(
                "AI provider is disabled".to_owned(),
            ));
        }
        if provider.provider_kind != "api" {
            return provider_ready_status(&provider.status);
        }
        if provider.consent_state != "granted" {
            return Err(AiControlCenterError::InvalidRequest(
                "API provider requires remote-context consent before private-context use"
                    .to_owned(),
            ));
        }
        if !self
            .api_key_secret_configured(&provider.provider_id)
            .await?
        {
            return Err(AiControlCenterError::InvalidRequest(
                "API provider requires a host-vault API key before private-context use".to_owned(),
            ));
        }
        provider_ready_status(&provider.status)
    }
}

fn provider_ready_status(status: &str) -> Result<(), AiControlCenterError> {
    if status == "ready" {
        return Ok(());
    }
    Err(AiControlCenterError::InvalidRequest(
        "AI provider setup is incomplete".to_owned(),
    ))
}
```

### `backend/src/ai/control_center/availability/secrets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/availability/secrets.rs`
- Size bytes / Размер в байтах: `2003`
- Included characters / Включено символов: `2003`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiControlCenterError;
use super::super::store::AiControlCenterStore;
use super::super::validation::validate_non_empty;

const SECRET_PURPOSE_API_KEY: &str = "api_key";
const SECRET_KIND_API_TOKEN: &str = "api_token";
const SECRET_STORE_HOST_VAULT: &str = "host_vault";

impl AiControlCenterStore {
    pub(in crate::ai::control_center) async fn api_key_secret_configured(
        &self,
        provider_id: &str,
    ) -> Result<bool, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        let configured = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM ai_provider_secret_refs refs
                JOIN secret_references secrets ON secrets.secret_ref = refs.secret_ref
                WHERE refs.provider_id = $1
                    AND refs.secret_purpose = $2
                    AND secrets.secret_kind = $3
                    AND secrets.store_kind = $4
            )
            "#,
        )
        .bind(provider_id.trim())
        .bind(SECRET_PURPOSE_API_KEY)
        .bind(SECRET_KIND_API_TOKEN)
        .bind(SECRET_STORE_HOST_VAULT)
        .fetch_one(&self.pool)
        .await?;

        Ok(configured)
    }

    pub(in crate::ai::control_center) async fn api_key_secret_reference_is_host_vault(
        &self,
        secret_ref: &str,
    ) -> Result<bool, AiControlCenterError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let configured = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM secret_references
                WHERE secret_ref = $1
                    AND secret_kind = $2
                    AND store_kind = $3
            )
            "#,
        )
        .bind(secret_ref.trim())
        .bind(SECRET_KIND_API_TOKEN)
        .bind(SECRET_STORE_HOST_VAULT)
        .fetch_one(&self.pool)
        .await?;

        Ok(configured)
    }
}
```

### `backend/src/ai/control_center/catalog.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/catalog.rs`
- Size bytes / Размер в байтах: `4585`
- Included characters / Включено символов: `4585`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::errors::AiControlCenterError;
use super::evidence::capture_model_catalog_item_observation;
use super::models::{AiModelCatalogItem, AiProviderAccount};
use super::presets::curated_models_for;
use super::rows::row_to_model;
use super::store::AiControlCenterStore;
use super::validation::validate_non_empty;

impl AiControlCenterStore {
    pub async fn list_models(&self) -> Result<Vec<AiModelCatalogItem>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                provider_id,
                model_key,
                display_name,
                category,
                privacy,
                capabilities,
                context_window,
                embedding_dimension,
                is_available,
                metadata,
                created_at,
                updated_at
            FROM ai_model_catalog
            ORDER BY category ASC, privacy ASC, display_name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_model).collect()
    }

    pub async fn model(
        &self,
        provider_id: &str,
        model_key: &str,
    ) -> Result<Option<AiModelCatalogItem>, AiControlCenterError> {
        validate_non_empty("provider_id", provider_id)?;
        validate_non_empty("model_key", model_key)?;
        let row = sqlx::query(
            r#"
            SELECT
                provider_id,
                model_key,
                display_name,
                category,
                privacy,
                capabilities,
                context_window,
                embedding_dimension,
                is_available,
                metadata,
                created_at,
                updated_at
            FROM ai_model_catalog
            WHERE provider_id = $1 AND model_key = $2
            "#,
        )
        .bind(provider_id.trim())
        .bind(model_key.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_model).transpose()
    }

    pub(super) async fn seed_models_for_provider(
        &self,
        provider: &AiProviderAccount,
        actor: &str,
    ) -> Result<(), AiControlCenterError> {
        let mut transaction = self.pool.begin().await?;
        for model in curated_models_for(provider) {
            let row = sqlx::query(
                r#"
                INSERT INTO ai_model_catalog (
                    provider_id,
                    model_key,
                    display_name,
                    category,
                    privacy,
                    capabilities,
                    context_window,
                    embedding_dimension,
                    is_available,
                    metadata,
                    created_at,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, now(), now())
                ON CONFLICT (provider_id, model_key)
                DO UPDATE SET
                    display_name = EXCLUDED.display_name,
                    category = EXCLUDED.category,
                    privacy = EXCLUDED.privacy,
                    capabilities = EXCLUDED.capabilities,
                    context_window = EXCLUDED.context_window,
                    embedding_dimension = EXCLUDED.embedding_dimension,
                    is_available = true,
                    metadata = EXCLUDED.metadata,
                    updated_at = now()
                RETURNING
                    provider_id,
                    model_key,
                    display_name,
                    category,
                    privacy,
                    capabilities,
                    context_window,
                    embedding_dimension,
                    is_available,
                    metadata,
                    created_at,
                    updated_at
                "#,
            )
            .bind(&provider.provider_id)
            .bind(model.model_key)
            .bind(model.display_name)
            .bind(model.category)
            .bind(model.privacy)
            .bind(json!(model.capabilities))
            .bind(model.context_window)
            .bind(model.embedding_dimension)
            .bind(model.metadata)
            .fetch_one(&mut *transaction)
            .await?;
            let model = row_to_model(row)?;
            capture_model_catalog_item_observation(&mut transaction, &model, "seed", actor).await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
```

### `backend/src/ai/control_center/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/errors.rs`
- Size bytes / Размер в байтах: `1416`
- Included characters / Включено символов: `1416`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::observations::ObservationStoreError;
use crate::platform::secrets::SecretReferenceError;
use crate::vault::HostVaultError;

#[derive(Debug, Error)]
pub enum AiControlCenterError {
    #[error("AI provider was not found")]
    ProviderNotFound,

    #[error("AI model was not found")]
    ModelNotFound,

    #[error("AI prompt was not found")]
    PromptNotFound,

    #[error("AI prompt version was not found")]
    PromptVersionNotFound,

    #[error("invalid AI control center request: {0}")]
    InvalidRequest(String),

    #[error("invalid AI control center field `{field}`")]
    EmptyField { field: &'static str },

    #[error("AI control center payload contains secret-like data")]
    SecretLikePayload,

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl AiControlCenterError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::InvalidRequest(_)
                | Self::EmptyField { .. }
                | Self::SecretLikePayload
                | Self::ModelNotFound
                | Self::PromptNotFound
                | Self::PromptVersionNotFound
        )
    }
}
```

### `backend/src/ai/control_center/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/evidence.rs`
- Size bytes / Размер в байтах: `11485`
- Included characters / Включено символов: `11485`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, link_domain_entity_in_transaction,
};

use super::errors::AiControlCenterError;
use super::models::{
    AiModelCatalogItem, AiModelRoute, AiPromptEvalRun, AiPromptTemplate, AiPromptVersion,
    AiProviderAccount,
};

pub(super) async fn capture_provider_account_observation(
    transaction: &mut Transaction<'_, Postgres>,
    provider: &AiProviderAccount,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROVIDER_ACCOUNT",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "provider_id": provider.provider_id,
                "provider_kind": provider.provider_kind,
                "provider_key": provider.provider_key,
                "display_name": provider.display_name,
                "status": provider.status,
                "consent_state": provider.consent_state,
                "consented_at": provider.consented_at,
                "config": provider.config,
                "capabilities": provider.capabilities,
                "action": relationship_kind,
            }),
            format!("ai-provider://{}", provider.provider_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_account",
        provider.provider_id.clone(),
        relationship_kind,
        json!({
            "provider_kind": provider.provider_kind,
            "provider_key": provider.provider_key,
            "status": provider.status,
            "consent_state": provider.consent_state,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_provider_secret_binding_observation(
    transaction: &mut Transaction<'_, Postgres>,
    provider_id: &str,
    secret_purpose: &str,
    secret_ref: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let binding_id = format!("{provider_id}:{secret_purpose}");
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROVIDER_SECRET_BINDING",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "provider_id": provider_id,
                "secret_purpose": secret_purpose,
                "secret_ref": secret_ref,
                "action": "bind",
            }),
            format!("ai-provider://{provider_id}/secret-binding/{secret_purpose}"),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": "bind",
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_secret_binding",
        binding_id,
        "bind",
        json!({
            "provider_id": provider_id,
            "secret_purpose": secret_purpose,
            "secret_ref": secret_ref,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_model_route_observation(
    transaction: &mut Transaction<'_, Postgres>,
    route: &AiModelRoute,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_MODEL_ROUTE",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "capability_slot": route.capability_slot,
                "provider_id": route.provider_id,
                "model_key": route.model_key,
                "action": "put",
            }),
            format!("ai-model-route://{}", route.capability_slot),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": "put",
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "model_route",
        route.capability_slot.clone(),
        "put",
        json!({
            "provider_id": route.provider_id,
            "model_key": route.model_key,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_prompt_template_observation(
    transaction: &mut Transaction<'_, Postgres>,
    prompt: &AiPromptTemplate,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROMPT_TEMPLATE",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "prompt_id": prompt.prompt_id,
                "name": prompt.name,
                "entity_scope": prompt.entity_scope,
                "capability_slot": prompt.capability_slot,
                "description": prompt.description,
                "is_system": prompt.is_system,
                "active_version_id": prompt.active_version_id,
                "metadata": prompt.metadata,
                "action": relationship_kind,
            }),
            format!("ai-prompt://{}", prompt.prompt_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "prompt_template",
        prompt.prompt_id.clone(),
        relationship_kind,
        json!({
            "entity_scope": prompt.entity_scope,
            "capability_slot": prompt.capability_slot,
            "active_version_id": prompt.active_version_id,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_prompt_version_observation(
    transaction: &mut Transaction<'_, Postgres>,
    version: &AiPromptVersion,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROMPT_TEMPLATE_VERSION",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "prompt_version_id": version.prompt_version_id,
                "prompt_id": version.prompt_id,
                "version_label": version.version_label,
                "body_template": version.body_template,
                "variables": version.variables,
                "status": version.status,
                "created_by_actor_id": version.created_by_actor_id,
                "action": relationship_kind,
            }),
            format!("ai-prompt-version://{}", version.prompt_version_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "prompt_template_version",
        version.prompt_version_id.clone(),
        relationship_kind,
        json!({
            "prompt_id": version.prompt_id,
            "version_label": version.version_label,
            "status": version.status,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_prompt_eval_run_observation(
    transaction: &mut Transaction<'_, Postgres>,
    eval_run: &AiPromptEvalRun,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_PROMPT_EVAL_RUN",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "eval_run_id": eval_run.eval_run_id,
                "prompt_id": eval_run.prompt_id,
                "prompt_version_id": eval_run.prompt_version_id,
                "provider_id": eval_run.provider_id,
                "model_key": eval_run.model_key,
                "source_refs": eval_run.source_refs,
                "variables": eval_run.variables,
                "output_text": eval_run.output_text,
                "score": eval_run.score,
                "notes": eval_run.notes,
                "actor_id": eval_run.actor_id,
                "action": "test",
            }),
            format!("ai-prompt-eval://{}", eval_run.eval_run_id),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": "test",
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "prompt_eval_run",
        eval_run.eval_run_id.clone(),
        "test",
        json!({
            "prompt_id": eval_run.prompt_id,
            "prompt_version_id": eval_run.prompt_version_id,
            "provider_id": eval_run.provider_id,
            "model_key": eval_run.model_key,
        }),
    )
    .await?;
    Ok(())
}

pub(super) async fn capture_model_catalog_item_observation(
    transaction: &mut Transaction<'_, Postgres>,
    model: &AiModelCatalogItem,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiControlCenterError> {
    let entity_id = format!("{}:{}", model.provider_id, model.model_key);
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_MODEL_CATALOG_ITEM",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "provider_id": model.provider_id,
                "model_key": model.model_key,
                "display_name": model.display_name,
                "category": model.category,
                "privacy": model.privacy,
                "capabilities": model.capabilities,
                "context_window": model.context_window,
                "embedding_dimension": model.embedding_dimension,
                "is_available": model.is_available,
                "metadata": model.metadata,
                "action": relationship_kind,
            }),
            format!("ai-model-catalog://{entity_id}"),
        )
        .provenance(json!({
            "captured_by": actor,
            "action": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "model_catalog_item",
        entity_id,
        relationship_kind,
        json!({
            "provider_id": model.provider_id,
            "model_key": model.model_key,
            "category": model.category,
            "privacy": model.privacy,
        }),
    )
    .await?;
    Ok(())
}

async fn link_ai_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    metadata: serde_json::Value,
) -> Result<(), crate::platform::observations::ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "ai",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
```

### `backend/src/ai/control_center/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/models.rs`
- Size bytes / Размер в байтах: `7834`
- Included characters / Включено символов: `7834`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AiControlCenterError;
use super::validation::{
    string_array_value, validate_capability_slot, validate_cli_preset, validate_entity_scope,
    validate_non_empty, validate_provider_kind,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiSettingsOverviewResponse {
    pub providers: Vec<AiProviderAccount>,
    pub models: Vec<AiModelCatalogItem>,
    pub routes: Vec<AiModelRoute>,
    pub prompts: Vec<AiPromptTemplate>,
    pub eval_runs: Vec<AiPromptEvalRun>,
    pub capability_slots: Vec<AiCapabilitySlot>,
    pub provider_presets: Vec<AiProviderPreset>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiCapabilitySlot {
    pub slot: String,
    pub label: String,
    pub description: String,
    pub requires_embedding_dimension: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderPreset {
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub privacy: String,
    pub base_url: Option<String>,
    pub command_preset: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderAccount {
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub status: String,
    pub consent_state: String,
    pub consented_at: Option<DateTime<Utc>>,
    pub config: Value,
    pub capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderCreateRequest {
    pub provider_id: Option<String>,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub base_url: Option<String>,
    pub command_preset: Option<String>,
    pub config: Option<Value>,
    pub capabilities: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub remote_context_consent: Option<bool>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
}

impl AiProviderCreateRequest {
    pub(super) fn validate(&self) -> Result<(), AiControlCenterError> {
        let provider_kind = self.provider_kind.trim();
        validate_provider_kind(provider_kind)?;
        validate_non_empty("provider_key", &self.provider_key)?;
        validate_non_empty("display_name", &self.display_name)?;
        if provider_kind != "api" && has_api_key(&self.api_key) {
            return Err(AiControlCenterError::InvalidRequest(
                "API keys can only be configured for API providers".to_owned(),
            ));
        }
        if provider_kind == "cli" {
            let preset = self.command_preset.as_deref().ok_or_else(|| {
                AiControlCenterError::InvalidRequest(
                    "CLI provider requires command_preset".to_owned(),
                )
            })?;
            validate_cli_preset(preset)?;
        }
        Ok(())
    }
}

fn has_api_key(api_key: &Option<String>) -> bool {
    api_key
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderPatchRequest {
    pub display_name: Option<String>,
    pub base_url: Option<String>,
    pub config: Option<Value>,
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderConsentRequest {
    pub consented: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderCommandResponse {
    pub provider_id: String,
    pub command: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiProviderCommandKind {
    Test,
    SyncModels,
}

impl AiProviderCommandKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::SyncModels => "sync_models",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelCatalogItem {
    pub provider_id: String,
    pub model_key: String,
    pub display_name: String,
    pub category: String,
    pub privacy: String,
    pub capabilities: Vec<String>,
    pub context_window: Option<i32>,
    pub embedding_dimension: Option<i32>,
    pub is_available: bool,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelRoute {
    pub capability_slot: String,
    pub provider_id: String,
    pub model_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelRouteUpdateRequest {
    pub provider_id: String,
    pub model_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptTemplate {
    pub prompt_id: String,
    pub name: String,
    pub entity_scope: String,
    pub capability_slot: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub active_version_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptCreateRequest {
    pub prompt_id: Option<String>,
    pub name: String,
    pub entity_scope: String,
    pub capability_slot: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

impl AiPromptCreateRequest {
    pub(super) fn validate(&self) -> Result<(), AiControlCenterError> {
        validate_non_empty("name", &self.name)?;
        validate_entity_scope(&self.entity_scope)?;
        validate_capability_slot(&self.capability_slot)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptVersion {
    pub prompt_version_id: String,
    pub prompt_id: String,
    pub version_label: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub status: String,
    pub created_by_actor_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptVersionCreateRequest {
    pub prompt_version_id: Option<String>,
    pub version_label: Option<String>,
    pub body_template: String,
    pub variables: Vec<String>,
}

impl AiPromptVersionCreateRequest {
    pub(super) fn validate(&self) -> Result<(), AiControlCenterError> {
        validate_non_empty("body_template", &self.body_template)?;
        let _ = string_array_value(self.variables.clone(), "variables")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptActivateRequest {
    pub prompt_version_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptTestRequest {
    pub prompt_version_id: String,
    pub provider_id: String,
    pub model_key: String,
    pub variables: Value,
    pub source_refs: Option<Vec<Value>>,
    pub score: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptEvalRun {
    pub eval_run_id: String,
    pub prompt_id: String,
    pub prompt_version_id: String,
    pub provider_id: String,
    pub model_key: String,
    pub source_refs: Vec<Value>,
    pub variables: Value,
    pub output_text: String,
    pub score: Option<i32>,
    pub notes: Option<String>,
    pub actor_id: String,
    pub created_at: DateTime<Utc>,
}
```

### `backend/src/ai/control_center/presets.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/presets.rs`
- Size bytes / Размер в байтах: `10285`
- Included characters / Включено символов: `10285`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::ai::core::AI_EMBEDDING_DIMENSION;

use super::models::{AiCapabilitySlot, AiProviderAccount, AiProviderPreset};
use super::validation::CAPABILITY_SLOTS;

pub const BUILT_IN_OLLAMA_PROVIDER_ID: &str = "provider:built_in:ollama";
pub const OLLAMA_CHAT_MODEL: &str = "qwen3:4b";
pub const OLLAMA_EMBEDDING_MODEL: &str = "qwen3-embedding:4b";

pub(super) fn capability_slots() -> Vec<AiCapabilitySlot> {
    CAPABILITY_SLOTS
        .iter()
        .map(|slot| AiCapabilitySlot {
            slot: (*slot).to_owned(),
            label: settings_label(slot),
            description: capability_description(slot),
            requires_embedding_dimension: if *slot == "embeddings" {
                Some(AI_EMBEDDING_DIMENSION as i32)
            } else {
                None
            },
        })
        .collect()
}

pub(super) fn provider_presets() -> Vec<AiProviderPreset> {
    vec![
        AiProviderPreset {
            provider_kind: "built_in".to_owned(),
            provider_key: "ollama".to_owned(),
            display_name: "Built-in Ollama".to_owned(),
            privacy: "local".to_owned(),
            base_url: Some("http://127.0.0.1:11434".to_owned()),
            command_preset: None,
            capabilities: vec![
                "chat".to_owned(),
                "embeddings".to_owned(),
                "local_runtime".to_owned(),
            ],
        },
        AiProviderPreset {
            provider_kind: "cli".to_owned(),
            provider_key: "codex".to_owned(),
            display_name: "Codex CLI".to_owned(),
            privacy: "cli".to_owned(),
            base_url: None,
            command_preset: Some("codex".to_owned()),
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
        AiProviderPreset {
            provider_kind: "cli".to_owned(),
            provider_key: "claude".to_owned(),
            display_name: "Claude CLI".to_owned(),
            privacy: "cli".to_owned(),
            base_url: None,
            command_preset: Some("claude".to_owned()),
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
        AiProviderPreset {
            provider_kind: "api".to_owned(),
            provider_key: "openai".to_owned(),
            display_name: "OpenAI".to_owned(),
            privacy: "remote".to_owned(),
            base_url: Some("https://api.openai.com/v1".to_owned()),
            command_preset: None,
            capabilities: vec![
                "chat".to_owned(),
                "reasoning".to_owned(),
                "embeddings".to_owned(),
            ],
        },
        AiProviderPreset {
            provider_kind: "api".to_owned(),
            provider_key: "deepseek".to_owned(),
            display_name: "DeepSeek".to_owned(),
            privacy: "remote".to_owned(),
            base_url: Some("https://api.deepseek.com/v1".to_owned()),
            command_preset: None,
            capabilities: vec!["chat".to_owned(), "reasoning".to_owned()],
        },
        AiProviderPreset {
            provider_kind: "api".to_owned(),
            provider_key: "omniroute".to_owned(),
            display_name: "OmniRoute".to_owned(),
            privacy: "remote".to_owned(),
            base_url: Some("https://ai.sh-inc.ru/v1".to_owned()),
            command_preset: None,
            capabilities: vec![
                "chat".to_owned(),
                "embeddings".to_owned(),
                "routing".to_owned(),
            ],
        },
    ]
}

pub(super) struct CuratedModel {
    pub(super) model_key: &'static str,
    pub(super) display_name: &'static str,
    pub(super) category: &'static str,
    pub(super) privacy: &'static str,
    pub(super) capabilities: Vec<&'static str>,
    pub(super) context_window: Option<i32>,
    pub(super) embedding_dimension: Option<i32>,
    pub(super) metadata: Value,
}

pub(super) fn curated_models_for(provider: &AiProviderAccount) -> Vec<CuratedModel> {
    match (
        provider.provider_kind.as_str(),
        provider.provider_key.as_str(),
    ) {
        ("built_in", "ollama") => vec![
            CuratedModel {
                model_key: OLLAMA_CHAT_MODEL,
                display_name: "Qwen3 4B",
                category: "chat",
                privacy: "local",
                capabilities: vec!["chat", "reasoning", "summarization", "extraction"],
                context_window: Some(32768),
                embedding_dimension: None,
                metadata: json!({"curated": true, "pull_required": true}),
            },
            CuratedModel {
                model_key: OLLAMA_EMBEDDING_MODEL,
                display_name: "Qwen3 Embedding 4B",
                category: "embeddings",
                privacy: "local",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(AI_EMBEDDING_DIMENSION as i32),
                metadata: json!({"curated": true, "pull_required": true}),
            },
        ],
        ("api", "openai") => vec![
            CuratedModel {
                model_key: "gpt-5.5",
                display_name: "GPT-5.5",
                category: "reasoning",
                privacy: "remote",
                capabilities: vec!["chat", "reasoning", "summarization"],
                context_window: Some(128000),
                embedding_dimension: None,
                metadata: json!({"curated": true}),
            },
            CuratedModel {
                model_key: "text-embedding-3-large",
                display_name: "Text Embedding 3 Large",
                category: "embeddings",
                privacy: "remote",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(3072),
                metadata: json!({"curated": true, "embedding_route_supported": false}),
            },
        ],
        ("api", "deepseek") => vec![CuratedModel {
            model_key: "deepseek-chat",
            display_name: "DeepSeek Chat",
            category: "chat",
            privacy: "remote",
            capabilities: vec!["chat", "reasoning", "summarization"],
            context_window: Some(64000),
            embedding_dimension: None,
            metadata: json!({"curated": true}),
        }],
        ("api", "omniroute") => vec![
            CuratedModel {
                model_key: "codex/gpt-5.5",
                display_name: "Codex GPT-5.5",
                category: "reasoning",
                privacy: "remote",
                capabilities: vec!["chat", "reasoning", "summarization"],
                context_window: Some(128000),
                embedding_dimension: None,
                metadata: json!({"curated": true}),
            },
            CuratedModel {
                model_key: "openai-compatible-chat-ollama-pve/qwen3-embedding:4b",
                display_name: "Qwen3 Embedding via OmniRoute",
                category: "embeddings",
                privacy: "remote",
                capabilities: vec!["embeddings"],
                context_window: Some(8192),
                embedding_dimension: Some(AI_EMBEDDING_DIMENSION as i32),
                metadata: json!({"curated": true}),
            },
        ],
        ("cli", "codex") => vec![CuratedModel {
            model_key: "codex-cli/default",
            display_name: "Codex CLI Default",
            category: "reasoning",
            privacy: "cli",
            capabilities: vec!["chat", "reasoning"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": true, "command_preset": "codex"}),
        }],
        ("cli", "claude") => vec![CuratedModel {
            model_key: "claude-cli/default",
            display_name: "Claude CLI Default",
            category: "reasoning",
            privacy: "cli",
            capabilities: vec!["chat", "reasoning"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": true, "command_preset": "claude"}),
        }],
        _ => vec![CuratedModel {
            model_key: "custom/default",
            display_name: "Custom default",
            category: "chat",
            privacy: if provider.provider_kind == "api" {
                "remote"
            } else {
                "cli"
            },
            capabilities: vec!["chat"],
            context_window: None,
            embedding_dimension: None,
            metadata: json!({"curated": false}),
        }],
    }
}

pub(super) fn default_capabilities(provider_kind: &str, provider_key: &str) -> Vec<String> {
    match provider_kind {
        "built_in" => vec!["chat", "embeddings", "local_runtime"],
        "cli" => vec!["chat", "reasoning"],
        "api" if provider_key == "omniroute" => vec!["chat", "embeddings", "routing"],
        "api" => vec!["chat", "reasoning"],
        _ => vec!["chat"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn settings_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capability_description(slot: &str) -> String {
    match slot {
        "default_chat" => "General source-backed answers and chat.".to_owned(),
        "reasoning" => "Higher-effort planning and synthesis.".to_owned(),
        "summarization" => "Short summaries over local context.".to_owned(),
        "mail_intelligence" => "Communication analysis and operational context.".to_owned(),
        "reply_draft" => "Drafting replies without sending provider messages.".to_owned(),
        "extraction" => "Structured extraction from untrusted source text.".to_owned(),
        "embeddings" => "Semantic index embeddings; dimension constrained.".to_owned(),
        "meeting_prep" => "Meeting brief generation from local context.".to_owned(),
        _ => "AI capability.".to_owned(),
    }
}
```

### `backend/src/ai/control_center/prompts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/prompts.rs`
- Size bytes / Размер в байтах: `89`
- Included characters / Включено символов: `89`
- Truncated / Обрезано: `no`

```rust
mod activation;
mod eval_runs;
mod evaluation;
mod lookups;
mod templates;
mod versions;
```

### `backend/src/ai/control_center/prompts/activation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/prompts/activation.rs`
- Size bytes / Размер в байтах: `3884`
- Included characters / Включено символов: `3884`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiControlCenterError;
use super::super::evidence::{
    capture_prompt_template_observation, capture_prompt_version_observation,
};
use super::super::models::{AiPromptActivateRequest, AiPromptTemplate, AiPromptVersion};
use super::super::rows::{row_to_prompt, row_to_prompt_version};
use super::super::store::AiControlCenterStore;
use super::super::validation::validate_non_empty;

impl AiControlCenterStore {
    pub async fn activate_prompt_version(
        &self,
        prompt_id: &str,
        request: &AiPromptActivateRequest,
        actor_id: &str,
    ) -> Result<AiPromptTemplate, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("prompt_version_id", &request.prompt_version_id)?;
        validate_non_empty("actor_id", actor_id)?;
        let prompt = self
            .prompt(prompt_id)
            .await?
            .ok_or(AiControlCenterError::PromptNotFound)?;
        if prompt.is_system {
            return Err(AiControlCenterError::InvalidRequest(
                "system prompts are read-only".to_owned(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let version_exists: Option<String> = sqlx::query_scalar(
            "SELECT prompt_version_id FROM ai_prompt_template_versions WHERE prompt_id = $1 AND prompt_version_id = $2",
        )
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .fetch_optional(&mut *tx)
        .await?;
        if version_exists.is_none() {
            return Err(AiControlCenterError::PromptVersionNotFound);
        }
        sqlx::query(
            "UPDATE ai_prompt_template_versions SET status = 'draft', updated_at = now() WHERE prompt_id = $1 AND status = 'active'",
        )
        .bind(prompt_id.trim())
        .execute(&mut *tx)
        .await?;
        let version_row = sqlx::query(
            "UPDATE ai_prompt_template_versions SET status = 'active', updated_at = now() WHERE prompt_version_id = $1",
        )
        .bind(request.prompt_version_id.trim())
        .execute(&mut *tx)
        .await?;
        if version_row.rows_affected() == 0 {
            return Err(AiControlCenterError::PromptVersionNotFound);
        }
        let active_version_row = sqlx::query(
            r#"
            SELECT
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id,
                created_at,
                updated_at
            FROM ai_prompt_template_versions
            WHERE prompt_version_id = $1
            "#,
        )
        .bind(request.prompt_version_id.trim())
        .fetch_one(&mut *tx)
        .await?;
        let row = sqlx::query(
            r#"
            UPDATE ai_prompt_templates
            SET
                active_version_id = $2,
                updated_at = now()
            WHERE prompt_id = $1
            RETURNING
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .fetch_one(&mut *tx)
        .await?;
        let active_version: AiPromptVersion = row_to_prompt_version(active_version_row)?;
        let prompt = row_to_prompt(row)?;
        capture_prompt_version_observation(&mut tx, &active_version, "activate", actor_id.trim())
            .await?;
        capture_prompt_template_observation(&mut tx, &prompt, "activate", actor_id.trim()).await?;
        tx.commit().await?;

        Ok(prompt)
    }
}
```

### `backend/src/ai/control_center/prompts/eval_runs.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/prompts/eval_runs.rs`
- Size bytes / Размер в байтах: `1059`
- Included characters / Включено символов: `1059`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiControlCenterError;
use super::super::models::AiPromptEvalRun;
use super::super::rows::row_to_eval_run;
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub(in crate::ai::control_center) async fn list_prompt_eval_runs(
        &self,
        limit: i64,
    ) -> Result<Vec<AiPromptEvalRun>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id,
                created_at
            FROM ai_prompt_eval_runs
            ORDER BY created_at DESC, eval_run_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_eval_run).collect()
    }
}
```

### `backend/src/ai/control_center/prompts/evaluation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/prompts/evaluation.rs`
- Size bytes / Размер в байтах: `3406`
- Included characters / Включено символов: `3406`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::{Value, json};

use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_prompt_eval_run_observation;
use super::super::models::{AiPromptEvalRun, AiPromptTestRequest};
use super::super::rows::row_to_eval_run;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    object_value, reject_secret_like_json, render_prompt, validate_non_empty,
};

impl AiControlCenterStore {
    pub async fn test_prompt(
        &self,
        prompt_id: &str,
        request: &AiPromptTestRequest,
        actor_id: &str,
    ) -> Result<AiPromptEvalRun, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("actor_id", actor_id)?;
        validate_non_empty("prompt_version_id", &request.prompt_version_id)?;
        validate_non_empty("provider_id", &request.provider_id)?;
        validate_non_empty("model_key", &request.model_key)?;
        let version = self
            .prompt_version(prompt_id, &request.prompt_version_id)
            .await?
            .ok_or(AiControlCenterError::PromptVersionNotFound)?;
        let _model = self
            .ensure_model_ready_for_private_context(&request.provider_id, &request.model_key)
            .await?;
        let variables = object_value(request.variables.clone(), "variables")?;
        reject_secret_like_json(&Value::Object(variables.clone()))?;
        let source_refs = request.source_refs.clone().unwrap_or_default();
        let rendered = render_prompt(&version.body_template, &variables);
        let output_text = format!("Prompt studio preview\n\n{rendered}");
        let eval_run_id = format!(
            "prompt-eval:{}:{}",
            prompt_id.trim(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_eval_runs (
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                eval_run_id,
                prompt_id,
                prompt_version_id,
                provider_id,
                model_key,
                source_refs,
                variables,
                output_text,
                score,
                notes,
                actor_id,
                created_at
            "#,
        )
        .bind(eval_run_id)
        .bind(prompt_id.trim())
        .bind(request.prompt_version_id.trim())
        .bind(request.provider_id.trim())
        .bind(request.model_key.trim())
        .bind(json!(source_refs))
        .bind(Value::Object(variables))
        .bind(output_text)
        .bind(request.score)
        .bind(request.notes.as_deref().map(str::trim))
        .bind(actor_id.trim())
        .fetch_one(&mut *transaction)
        .await?;

        let eval_run = row_to_eval_run(row)?;
        capture_prompt_eval_run_observation(&mut transaction, &eval_run, actor_id.trim()).await?;
        transaction.commit().await?;
        Ok(eval_run)
    }
}
```

### `backend/src/ai/control_center/prompts/lookups.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/prompts/lookups.rs`
- Size bytes / Размер в байтах: `1798`
- Included characters / Включено символов: `1798`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiControlCenterError;
use super::super::models::{AiPromptTemplate, AiPromptVersion};
use super::super::rows::{row_to_prompt, row_to_prompt_version};
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub(super) async fn prompt(
        &self,
        prompt_id: &str,
    ) -> Result<Option<AiPromptTemplate>, AiControlCenterError> {
        let row = sqlx::query(
            r#"
            SELECT
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            FROM ai_prompt_templates
            WHERE prompt_id = $1
            "#,
        )
        .bind(prompt_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_prompt).transpose()
    }

    pub(super) async fn prompt_version(
        &self,
        prompt_id: &str,
        prompt_version_id: &str,
    ) -> Result<Option<AiPromptVersion>, AiControlCenterError> {
        let row = sqlx::query(
            r#"
            SELECT
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id,
                created_at,
                updated_at
            FROM ai_prompt_template_versions
            WHERE prompt_id = $1 AND prompt_version_id = $2
            "#,
        )
        .bind(prompt_id.trim())
        .bind(prompt_version_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_prompt_version).transpose()
    }
}
```

### `backend/src/ai/control_center/prompts/templates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/prompts/templates.rs`
- Size bytes / Размер в байтах: `3267`
- Included characters / Включено символов: `3267`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_prompt_template_observation;
use super::super::models::{AiPromptCreateRequest, AiPromptTemplate};
use super::super::rows::row_to_prompt;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    object_value, reject_secret_like_json, slug_id, validate_non_empty,
};

impl AiControlCenterStore {
    pub async fn list_prompts(&self) -> Result<Vec<AiPromptTemplate>, AiControlCenterError> {
        let rows = sqlx::query(
            r#"
            SELECT
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            FROM ai_prompt_templates
            ORDER BY entity_scope ASC, capability_slot ASC, name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_prompt).collect()
    }

    pub async fn create_prompt(
        &self,
        request: &AiPromptCreateRequest,
        actor_id: &str,
    ) -> Result<AiPromptTemplate, AiControlCenterError> {
        validate_non_empty("actor_id", actor_id)?;
        request.validate()?;
        let prompt_id = request
            .prompt_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "prompt:user:{}:{}",
                    request.entity_scope.trim(),
                    slug_id(request.name.trim())
                )
            });
        let metadata = object_value(
            request.metadata.clone().unwrap_or_else(|| json!({})),
            "metadata",
        )?;
        reject_secret_like_json(&Value::Object(metadata.clone()))?;
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_templates (
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, false, $6)
            RETURNING
                prompt_id,
                name,
                entity_scope,
                capability_slot,
                description,
                is_system,
                active_version_id,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_id)
        .bind(request.name.trim())
        .bind(request.entity_scope.trim())
        .bind(request.capability_slot.trim())
        .bind(request.description.as_deref().map(str::trim))
        .bind(Value::Object(metadata))
        .fetch_one(&mut *transaction)
        .await?;

        let prompt = row_to_prompt(row)?;
        capture_prompt_template_observation(&mut transaction, &prompt, "create", actor_id.trim())
            .await?;
        transaction.commit().await?;
        Ok(prompt)
    }
}
```

### `backend/src/ai/control_center/prompts/versions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/prompts/versions.rs`
- Size bytes / Размер в байтах: `3106`
- Included characters / Включено символов: `3106`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_prompt_version_observation;
use super::super::models::{AiPromptVersion, AiPromptVersionCreateRequest};
use super::super::rows::row_to_prompt_version;
use super::super::store::AiControlCenterStore;
use super::super::validation::{slug_id, string_array_value, validate_non_empty};

impl AiControlCenterStore {
    pub async fn create_prompt_version(
        &self,
        prompt_id: &str,
        request: &AiPromptVersionCreateRequest,
        actor_id: &str,
    ) -> Result<AiPromptVersion, AiControlCenterError> {
        validate_non_empty("prompt_id", prompt_id)?;
        validate_non_empty("actor_id", actor_id)?;
        request.validate()?;
        let prompt = self
            .prompt(prompt_id)
            .await?
            .ok_or(AiControlCenterError::PromptNotFound)?;
        if prompt.is_system {
            return Err(AiControlCenterError::InvalidRequest(
                "system prompts are read-only".to_owned(),
            ));
        }
        let variables = string_array_value(request.variables.clone(), "variables")?;
        let version_label = request
            .version_label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| Utc::now().format("v%Y%m%d%H%M%S").to_string());
        let prompt_version_id = request
            .prompt_version_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "prompt-version:{}:{}",
                    prompt_id.trim(),
                    slug_id(&version_label)
                )
            });
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_prompt_template_versions (
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id
            )
            VALUES ($1, $2, $3, $4, $5, 'draft', $6)
            RETURNING
                prompt_version_id,
                prompt_id,
                version_label,
                body_template,
                variables,
                status,
                created_by_actor_id,
                created_at,
                updated_at
            "#,
        )
        .bind(prompt_version_id)
        .bind(prompt_id.trim())
        .bind(version_label)
        .bind(request.body_template.trim())
        .bind(json!(variables))
        .bind(actor_id.trim())
        .fetch_one(&mut *transaction)
        .await?;

        let version = row_to_prompt_version(row)?;
        capture_prompt_version_observation(&mut transaction, &version, "create", actor_id.trim())
            .await?;
        transaction.commit().await?;
        Ok(version)
    }
}
```

### `backend/src/ai/control_center/providers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/providers.rs`
- Size bytes / Размер в байтах: `77`
- Included characters / Включено символов: `77`
- Truncated / Обрезано: `no`

```rust
mod commands;
mod consent;
mod create;
mod queries;
mod secrets;
mod update;
```

### `backend/src/ai/control_center/providers/commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/providers/commands.rs`
- Size bytes / Размер в байтах: `2513`
- Included characters / Включено символов: `2513`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiControlCenterError;
use super::super::models::{AiProviderCommandKind, AiProviderCommandResponse};
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub async fn provider_command(
        &self,
        provider_id: &str,
        command: AiProviderCommandKind,
    ) -> Result<AiProviderCommandResponse, AiControlCenterError> {
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        let (status, message) = match command {
            AiProviderCommandKind::Test => match provider.provider_kind.as_str() {
                "built_in" => ("ok", "Built-in runtime metadata is configured"),
                "cli" => ("ok", "CLI provider preset is allowlisted"),
                "api" => {
                    if provider.status == "disabled" {
                        ("disabled", "API provider is disabled")
                    } else if provider.consent_state != "granted" {
                        (
                            "needs_consent",
                            "API provider requires remote-context consent",
                        )
                    } else if !self
                        .api_key_secret_configured(&provider.provider_id)
                        .await?
                    {
                        ("needs_setup", "API provider requires a host-vault API key")
                    } else if provider.status != "ready" {
                        ("needs_setup", "API provider setup is incomplete")
                    } else {
                        (
                            "ok",
                            "API provider consent and host-vault key reference are configured; live network check is deferred",
                        )
                    }
                }
                _ => ("error", "Unsupported provider kind"),
            },
            AiProviderCommandKind::SyncModels => {
                self.seed_models_for_provider(
                    &provider,
                    "ai_control_center.provider_command.sync_models",
                )
                .await?;
                ("synced", "Curated model catalog synchronized")
            }
        };

        Ok(AiProviderCommandResponse {
            provider_id: provider.provider_id,
            command: command.as_str().to_owned(),
            status: status.to_owned(),
            message: message.to_owned(),
        })
    }
}
```

### `backend/src/ai/control_center/providers/consent.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/providers/consent.rs`
- Size bytes / Размер в байтах: `2171`
- Included characters / Включено символов: `2171`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_provider_account_observation;
use super::super::models::{AiProviderAccount, AiProviderConsentRequest};
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;

impl AiControlCenterStore {
    pub async fn record_consent(
        &self,
        provider_id: &str,
        request: &AiProviderConsentRequest,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        let provider = self
            .provider(provider_id)
            .await?
            .ok_or(AiControlCenterError::ProviderNotFound)?;
        if provider.provider_kind != "api" {
            return Err(AiControlCenterError::InvalidRequest(
                "Remote-context consent applies only to API providers".to_owned(),
            ));
        }
        let consent_state = if request.consented {
            "granted"
        } else {
            "revoked"
        };
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE ai_provider_accounts
            SET
                consent_state = $2,
                consented_at = CASE WHEN $2 = 'granted' THEN now() ELSE NULL END,
                updated_at = now()
            WHERE provider_id = $1
            RETURNING
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                consented_at,
                config,
                capabilities,
                created_at,
                updated_at
            "#,
        )
        .bind(provider_id.trim())
        .bind(consent_state)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(AiControlCenterError::ProviderNotFound)?;

        let provider = row_to_provider(row)?;
        capture_provider_account_observation(
            &mut transaction,
            &provider,
            "consent_recorded",
            "ai_control_center.record_consent",
        )
        .await?;
        transaction.commit().await?;
        Ok(provider)
    }
}
```

### `backend/src/ai/control_center/providers/create.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/ai/control_center/providers/create.rs`
- Size bytes / Размер в байтах: `4146`
- Included characters / Включено символов: `4146`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};
use sqlx::Row;

use super::super::errors::AiControlCenterError;
use super::super::evidence::capture_provider_account_observation;
use super::super::models::{AiProviderAccount, AiProviderCreateRequest};
use super::super::presets::default_capabilities;
use super::super::rows::row_to_provider;
use super::super::store::AiControlCenterStore;
use super::super::validation::{
    non_empty_optional, object_value, reject_secret_like_json, slug_id, string_array_value,
    validate_non_empty,
};

impl AiControlCenterStore {
    pub async fn create_provider(
        &self,
        request: &AiProviderCreateRequest,
    ) -> Result<AiProviderAccount, AiControlCenterError> {
        request.validate()?;
        let provider_kind = request.provider_kind.trim();
        let provider_key = request.provider_key.trim();
        let provider_id = request
            .provider_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "provider:{provider_kind}:{}",
                    slug_id(&format!("{}-{}", provider_key, request.display_name.trim()))
                )
            });
        validate_non_empty("provider_id", &provider_id)?;

        let mut config = object_value(
            request.config.clone().unwrap_or_else(|| json!({})),
            "config",
        )?;
        if let Some(base_url) = non_empty_optional(&request.base_url) {
            config.insert("base_url".to_owned(), json!(base_url));
        }
        if let Some(command_preset) = non_empty_optional(&request.command_preset) {
            config.insert("command_preset".to_owned(), json!(command_preset));
        }
        reject_secret_like_json(&Value::Object(config.clone()))?;

        let capabilities = string_array_value(
            request
                .capabilities
                .clone()
                .unwrap_or_else(|| default_capabilities(provider_kind, provider_key)),
            "capabilities",
        )?;
        let status = if request.enabled.unwrap_or(true) {
            match provider_kind {
                "api" => "needs_setup",
                _ => "ready",
            }
        } else {
            "disabled"
        };
        let consent_state = match provider_kind {
            "api" if request.remote_context_consent == Some(true) => "granted",
            "api" => "required",
            _ => "not_required",
        };

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO ai_provider_accounts (
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                consented_at,
                config,
                capabilities
            )
            VALUES ($1, $2, $3, $4, $5, $6, CASE WHEN $6 = 'granted' THEN now() ELSE NULL END, $7, $8)
            RETURNING
                provider_id,
                provider_kind,
                provider_key,
                display_name,
                status,
                consent_state,
                consented_at,
                config,
                capabilities,
                created_at,
                updated_at
            "#,
        )
        .bind(provider_id)
        .bind(provider_kind)
        .bind(provider_key)
        .bind(request.display_name.trim())
        .bind(status)
        .bind(consent_state)
        .bind(Value::Object(config))
        .bind(json!(capabilities))
        .fetch_one(&mut *transaction)
        .await?;

        let provider = row_to_provider(row)?;
        capture_provider_account_observation(
            &mut transaction,
            &provider,
            "create",
            "ai_control_center.create_provider",
        )
        .await?;
        transaction.commit().await?;
        self.seed_models_for_provider(&provider, "ai_control_center.create_provider")
            .await?;
        Ok(provider)
    }
}
```

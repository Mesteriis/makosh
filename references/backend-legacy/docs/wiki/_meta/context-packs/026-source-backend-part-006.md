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

- Chunk ID / ID чанка: `026-source-backend-part-006`
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

### `backend/src/app/error/response/integrations/call.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/integrations/call.rs`
- Size bytes / Размер в байтах: `704`
- Included characters / Включено символов: `704`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::platform::calls::CallError;

use super::super::ErrorParts;

pub(super) fn call_error_parts(error: CallError) -> ErrorParts {
    match error {
        CallError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_call_request",
            message,
            false,
        ),
        CallError::Sqlx(error) => {
            tracing::error!(error = %error, "call intelligence database operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "call_store_error",
                "call intelligence operation failed".to_owned(),
                false,
            )
        }
    }
}
```

### `backend/src/app/error/response/integrations/telegram.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/integrations/telegram.rs`
- Size bytes / Размер в байтах: `4104`
- Included characters / Включено символов: `4104`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::application::provider_runtime_contracts::TelegramError;

use super::super::ErrorParts;

pub(super) fn telegram_error_parts(error: TelegramError) -> ErrorParts {
    match error {
        TelegramError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_telegram_request",
            message,
            false,
        ),
        TelegramError::TdlibRuntimeUnavailable(error) => {
            tracing::warn!(error = %error, "Telegram TDLib runtime is unavailable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "telegram_tdlib_runtime_unavailable",
                "Telegram TDLib runtime is not configured on this host".to_owned(),
                false,
            )
        }
        TelegramError::TdlibRuntime(error) => {
            tracing::warn!(error = %error, "Telegram TDLib runtime operation failed");
            (
                StatusCode::BAD_GATEWAY,
                "telegram_tdlib_runtime_error",
                "Telegram TDLib runtime operation failed".to_owned(),
                false,
            )
        }
        TelegramError::QrGeneration(error) => {
            tracing::warn!(error = %error, "Telegram QR generation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram_qr_generation_error",
                "Telegram QR generation failed".to_owned(),
                false,
            )
        }
        TelegramError::QrLoginNotFound => (
            StatusCode::NOT_FOUND,
            "telegram_qr_login_not_found",
            "Telegram QR login setup was not found".to_owned(),
            false,
        ),
        TelegramError::ProviderAccountStore(error) => internal(
            error,
            "Telegram provider account store operation failed",
            "telegram_provider_account_store_error",
            "Telegram provider account store operation failed",
        ),
        TelegramError::MediaStorage(error) => internal(
            error,
            "Telegram media storage operation failed",
            "telegram_media_storage_error",
            "Telegram media storage operation failed",
        ),
        TelegramError::CommunicationMessagePort(error) => internal(
            error,
            "Telegram communication message port operation failed",
            "telegram_message_port_error",
            "Telegram message read model operation failed",
        ),
        TelegramError::SecretReference(error) => internal(
            error,
            "Telegram secret reference operation failed",
            "telegram_secret_reference_error",
            "Telegram secret reference operation failed",
        ),
        TelegramError::DatabaseVault(error) => internal(
            error,
            "Telegram database vault operation failed",
            "telegram_secret_vault_error",
            "Telegram secret vault operation failed",
        ),
        TelegramError::HostVault(error) => {
            tracing::warn!(error = %error, "Telegram host vault operation failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "telegram_host_vault_error",
                "Telegram host vault operation failed".to_owned(),
                false,
            )
        }
        TelegramError::ObservationStore(error) => internal(
            error,
            "Telegram observation trail operation failed",
            "telegram_observation_error",
            "Telegram observation trail operation failed",
        ),
        TelegramError::Sqlx(error) => internal(
            error,
            "Telegram database operation failed",
            "telegram_store_error",
            "Telegram store operation failed",
        ),
    }
}

fn internal(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
    message: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        message.to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/integrations/whatsapp.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/integrations/whatsapp.rs`
- Size bytes / Размер в байтах: `2562`
- Included characters / Включено символов: `2562`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::application::provider_runtime_contracts::WhatsappWebError;

use super::super::ErrorParts;

pub(super) fn whatsapp_web_error_parts(error: WhatsappWebError) -> ErrorParts {
    match error {
        WhatsappWebError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_whatsapp_web_request",
            message,
            false,
        ),
        WhatsappWebError::ProviderAccountStore(error) => internal(
            error,
            "WhatsApp Web provider account store operation failed",
            "whatsapp_web_provider_account_store_error",
            "WhatsApp Web provider account store operation failed",
        ),
        WhatsappWebError::CommunicationMessagePort(error) => internal(
            error,
            "WhatsApp Web communication message port operation failed",
            "whatsapp_web_message_port_error",
            "WhatsApp Web message read model operation failed",
        ),
        WhatsappWebError::ObservationStore(error) => internal(
            error,
            "WhatsApp Web observation store operation failed",
            "whatsapp_web_observation_error",
            "WhatsApp Web store operation failed",
        ),
        WhatsappWebError::SecretReference(error) => internal(
            error,
            "WhatsApp Web secret reference operation failed",
            "whatsapp_web_secret_reference_error",
            "WhatsApp Web credential metadata operation failed",
        ),
        WhatsappWebError::SecretResolution(error) => internal(
            error,
            "WhatsApp Web secret resolution operation failed",
            "whatsapp_web_secret_resolution_error",
            "WhatsApp Web credential resolution failed",
        ),
        WhatsappWebError::HostVault(error) => internal(
            error,
            "WhatsApp Web host vault operation failed",
            "whatsapp_web_host_vault_error",
            "WhatsApp Web credential vault operation failed",
        ),
        WhatsappWebError::Sqlx(error) => internal(
            error,
            "WhatsApp Web database operation failed",
            "whatsapp_web_store_error",
            "WhatsApp Web store operation failed",
        ),
    }
}

fn internal(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
    message: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        message.to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/integrations/yandex_telemost.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/integrations/yandex_telemost.rs`
- Size bytes / Размер в байтах: `4459`
- Included characters / Включено символов: `4459`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::application::provider_runtime_contracts::YandexTelemostError;

use super::super::ErrorParts;

pub(super) fn yandex_telemost_error_parts(error: YandexTelemostError) -> ErrorParts {
    match error {
        YandexTelemostError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_yandex_telemost_request",
            message,
            false,
        ),
        YandexTelemostError::ProviderAccountStore(error) => internal(
            error,
            "Yandex Telemost provider account store operation failed",
            "yandex_telemost_provider_account_store_error",
            "Yandex Telemost provider account store operation failed",
        ),
        YandexTelemostError::ProviderSecretBindingStore(error) => internal(
            error,
            "Yandex Telemost provider secret binding operation failed",
            "yandex_telemost_provider_secret_binding_error",
            "Yandex Telemost credential metadata operation failed",
        ),
        YandexTelemostError::EventStore(error) => internal(
            error,
            "Yandex Telemost event store operation failed",
            "yandex_telemost_event_store_error",
            "Yandex Telemost event store operation failed",
        ),
        YandexTelemostError::EventEnvelope(error) => internal(
            error,
            "Yandex Telemost event envelope operation failed",
            "yandex_telemost_event_envelope_error",
            "Yandex Telemost event envelope operation failed",
        ),
        YandexTelemostError::SecretReference(error) => internal(
            error,
            "Yandex Telemost secret reference operation failed",
            "yandex_telemost_secret_reference_error",
            "Yandex Telemost credential metadata operation failed",
        ),
        YandexTelemostError::SecretResolution(error) => internal(
            error,
            "Yandex Telemost secret resolution failed",
            "yandex_telemost_secret_resolution_error",
            "Yandex Telemost credential resolution failed",
        ),
        YandexTelemostError::HostVault(error) => internal(
            error,
            "Yandex Telemost host vault operation failed",
            "yandex_telemost_host_vault_error",
            "Yandex Telemost credential storage operation failed",
        ),
        YandexTelemostError::Http(error) => internal(
            error,
            "Yandex Telemost provider HTTP operation failed",
            "yandex_telemost_provider_http_error",
            "Yandex Telemost provider request failed",
        ),
        YandexTelemostError::Serialization(error) => internal(
            error,
            "Yandex Telemost serialization failed",
            "yandex_telemost_serialization_error",
            "Yandex Telemost serialization failed",
        ),
        YandexTelemostError::Io(error) => internal(
            error,
            "Yandex Telemost local recording bundle I/O failed",
            "yandex_telemost_local_bundle_io_error",
            "Yandex Telemost local recording bundle I/O failed",
        ),
        YandexTelemostError::ObservationStore(error) => internal(
            error,
            "Yandex Telemost observation capture failed",
            "yandex_telemost_observation_store_error",
            "Yandex Telemost observation capture failed",
        ),
        YandexTelemostError::ReviewInbox(error) => internal(
            error,
            "Yandex Telemost review inbox mirroring failed",
            "yandex_telemost_review_inbox_error",
            "Yandex Telemost review inbox mirroring failed",
        ),
        YandexTelemostError::Settings(error) if error.is_invalid_request() => (
            StatusCode::BAD_REQUEST,
            "invalid_yandex_telemost_setting",
            error.to_string(),
            false,
        ),
        YandexTelemostError::Settings(error) => internal(
            error,
            "Yandex Telemost settings operation failed",
            "yandex_telemost_settings_error",
            "Yandex Telemost settings operation failed",
        ),
    }
}

fn internal(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
    message: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        message.to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/integrations/zoom.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/integrations/zoom.rs`
- Size bytes / Размер в байтах: `3868`
- Included characters / Включено символов: `3868`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::application::provider_runtime_contracts::ZoomError;

use super::super::ErrorParts;

pub(super) fn zoom_error_parts(error: ZoomError) -> ErrorParts {
    match error {
        ZoomError::InvalidRequest(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_zoom_request",
            message,
            false,
        ),
        ZoomError::ProviderAccountStore(error) => internal(
            error,
            "Zoom provider account store operation failed",
            "zoom_provider_account_store_error",
            "Zoom provider account store operation failed",
        ),
        ZoomError::ProviderSecretBindingStore(error) => internal(
            error,
            "Zoom provider secret binding operation failed",
            "zoom_provider_secret_binding_error",
            "Zoom provider credential metadata operation failed",
        ),
        ZoomError::Call(error) => internal(
            error,
            "Zoom call projection operation failed",
            "zoom_call_projection_error",
            "Zoom call projection operation failed",
        ),
        ZoomError::EventStore(error) => internal(
            error,
            "Zoom event store operation failed",
            "zoom_event_store_error",
            "Zoom event store operation failed",
        ),
        ZoomError::EventEnvelope(error) => internal(
            error,
            "Zoom event envelope operation failed",
            "zoom_event_envelope_error",
            "Zoom event envelope operation failed",
        ),
        ZoomError::SecretReference(error) => internal(
            error,
            "Zoom secret reference operation failed",
            "zoom_secret_reference_error",
            "Zoom credential metadata operation failed",
        ),
        ZoomError::SecretResolution(error) => internal(
            error,
            "Zoom secret resolution failed",
            "zoom_secret_resolution_error",
            "Zoom credential resolution failed",
        ),
        ZoomError::HostVault(error) => internal(
            error,
            "Zoom host vault operation failed",
            "zoom_host_vault_error",
            "Zoom credential storage operation failed",
        ),
        ZoomError::Http(error) => internal(
            error,
            "Zoom provider HTTP operation failed",
            "zoom_provider_http_error",
            "Zoom provider authorization request failed",
        ),
        ZoomError::Serialization(error) => internal(
            error,
            "Zoom credential serialization failed",
            "zoom_credential_serialization_error",
            "Zoom credential storage operation failed",
        ),
        ZoomError::Sqlx(error) => internal(
            error,
            "Zoom database operation failed",
            "zoom_store_error",
            "Zoom store operation failed",
        ),
        ZoomError::Storage(error) => internal(
            error,
            "Zoom recording storage operation failed",
            "zoom_recording_storage_error",
            "Zoom recording storage operation failed",
        ),
        ZoomError::Io(error) => internal(
            error,
            "Zoom file operation failed",
            "zoom_io_error",
            "Zoom file operation failed",
        ),
        ZoomError::Settings(error) => internal(
            error,
            "Zoom retention settings operation failed",
            "zoom_settings_error",
            "Zoom retention policy operation failed",
        ),
    }
}

fn internal(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
    message: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        message.to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/knowledge.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/knowledge.rs`
- Size bytes / Размер в байтах: `2515`
- Included characters / Включено символов: `2515`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::Graph(error) => {
            tracing::error!(error = %error, "graph store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "graph_store_error",
                "graph store operation failed".to_owned(),
                false,
            )
        }
        ApiError::InvalidGraphQuery(message) => bad_request("invalid_graph_query", message),
        ApiError::Projects(error) => {
            tracing::error!(error = %error, "project API store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "project_store_error",
                "project store operation failed".to_owned(),
                false,
            )
        }
        ApiError::InvalidProjectQuery(message) => bad_request("invalid_project_query", message),
        ApiError::InvalidProjectLinkReview(message) => {
            bad_request("invalid_project_link_review", message)
        }
        ApiError::ProjectLinkTargetNotFound => (
            StatusCode::NOT_FOUND,
            "project_link_target_not_found",
            "project link target was not found".to_owned(),
            false,
        ),
        ApiError::ProjectLinkReview(error) => {
            tracing::error!(error = %error, "project link review store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "project_link_review_store_error",
                "project link review store operation failed".to_owned(),
                false,
            )
        }
        ApiError::GraphNotFound => (
            StatusCode::NOT_FOUND,
            "graph_node_not_found",
            "graph node was not found".to_owned(),
            false,
        ),
        ApiError::ProjectNotFound => (
            StatusCode::NOT_FOUND,
            "project_not_found",
            "project was not found".to_owned(),
            false,
        ),
        ApiError::NotFound => (
            StatusCode::NOT_FOUND,
            "event_not_found",
            "event was not found".to_owned(),
            false,
        ),
        _ => unreachable!("knowledge response mapper received non-knowledge ApiError"),
    }
}

fn bad_request(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::BAD_REQUEST, error, message.to_owned(), false)
}
```

### `backend/src/app/error/response/persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/persons.rs`
- Size bytes / Размер в байтах: `2895`
- Included characters / Включено символов: `2895`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use crate::domains::persons::api::PersonProjectionError;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::InvalidPersonaQuery(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_persona_query",
            message.to_owned(),
            false,
        ),
        ApiError::InvalidPersonIdentityReview(message) => (
            StatusCode::BAD_REQUEST,
            "invalid_person_identity_review",
            message.to_owned(),
            false,
        ),
        ApiError::PersonIdentityNotFound => (
            StatusCode::NOT_FOUND,
            "person_identity_candidate_not_found",
            "person identity candidate was not found".to_owned(),
            false,
        ),
        ApiError::PersonProjection(error) => projection_error_parts(error),
        ApiError::PersonIdentity(error) => {
            tracing::error!(
                error = %error,
                "person identity store operation failed"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_identity_store_error",
                "person identity store operation failed".to_owned(),
                false,
            )
        }
        _ => unreachable!("persons response mapper received non-person ApiError"),
    }
}

pub(super) fn projection_error_parts(error: PersonProjectionError) -> ErrorParts {
    match error {
        PersonProjectionError::PersonNotFound(_) => (
            StatusCode::NOT_FOUND,
            "person_not_found",
            "person was not found".to_owned(),
            false,
        ),
        PersonProjectionError::EmptyEmailAddress
        | PersonProjectionError::InvalidEmailAddress(_)
        | PersonProjectionError::EmptyAiAgentId
        | PersonProjectionError::InvalidAiAgentId(_)
        | PersonProjectionError::EmptyDisplayName
        | PersonProjectionError::InvalidPersonaType(_) => (
            StatusCode::BAD_REQUEST,
            "invalid_person_projection",
            error.to_string(),
            false,
        ),
        PersonProjectionError::Sqlx(error) => {
            tracing::error!(error = %error, "person projection operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_projection_error",
                "person projection operation failed".to_owned(),
                false,
            )
        }
        PersonProjectionError::Observation(error) => {
            tracing::error!(error = %error, "person projection observation operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "person_projection_error",
                "person projection operation failed".to_owned(),
                false,
            )
        }
    }
}
```

### `backend/src/app/error/response/platform.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/platform.rs`
- Size bytes / Размер в байтах: `3867`
- Included characters / Включено символов: `3867`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::DatabaseNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "database_not_configured",
            "DATABASE_URL is not configured".to_owned(),
            false,
        ),
        ApiError::SecretVaultNotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            "secret_vault_not_configured",
            "host vault must be initialized and unlocked for account setup".to_owned(),
            false,
        ),
        ApiError::HostVault(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "host_vault_error",
            error.to_string(),
            false,
        ),
        ApiError::InvalidEnvelope(error) => (
            StatusCode::BAD_REQUEST,
            "invalid_event_envelope",
            error.to_string(),
            false,
        ),
        ApiError::FailedPrecondition(message) => (
            StatusCode::PRECONDITION_FAILED,
            "failed_precondition",
            message,
            false,
        ),
        ApiError::Audit(error) => {
            tracing::error!(error = %error, "event API audit operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "api_audit_error",
                "API audit operation failed".to_owned(),
                false,
            )
        }
        ApiError::Store(error) if error.is_unique_violation() => (
            StatusCode::CONFLICT,
            "event_conflict",
            "event already exists or violates idempotency constraints".to_owned(),
            false,
        ),
        ApiError::Store(error) => {
            tracing::error!(error = %error, "event API store operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "event_store_error",
                "event store operation failed".to_owned(),
                false,
            )
        }
        ApiError::SettingNotFound => (
            StatusCode::NOT_FOUND,
            "setting_not_found",
            "application setting was not found".to_owned(),
            false,
        ),
        ApiError::Settings(error) if error.is_invalid_request() => (
            StatusCode::BAD_REQUEST,
            "invalid_application_setting",
            error.to_string(),
            false,
        ),
        ApiError::Settings(error) => {
            tracing::error!(error = %error, "application settings operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "application_settings_error",
                "application settings operation failed".to_owned(),
                false,
            )
        }
        ApiError::SignalHub(error) if error.is_invalid_request() => (
            StatusCode::BAD_REQUEST,
            "invalid_signal_hub_request",
            error.to_string(),
            false,
        ),
        ApiError::SignalHub(error) if error.is_not_found() => (
            StatusCode::NOT_FOUND,
            "signal_hub_not_found",
            error.to_string(),
            false,
        ),
        ApiError::SignalHub(error) if error.is_failed_precondition() => (
            StatusCode::PRECONDITION_FAILED,
            "signal_hub_precondition_failed",
            error.to_string(),
            false,
        ),
        ApiError::SignalHub(error) => {
            tracing::error!(error = %error, "Signal Hub operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "signal_hub_error",
                "Signal Hub operation failed".to_owned(),
                false,
            )
        }
        _ => unreachable!("platform response mapper received non-platform ApiError"),
    }
}
```

### `backend/src/app/error/response/review.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/review.rs`
- Size bytes / Размер в байтах: `4412`
- Included characters / Включено символов: `4412`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::InvalidTaskCandidateQuery(message) => {
            bad_request("invalid_task_candidate_query", message)
        }
        ApiError::InvalidTaskCandidateReview(message) => {
            bad_request("invalid_task_candidate_review", message)
        }
        ApiError::InvalidObligationQuery(message) => {
            bad_request("invalid_obligation_query", message)
        }
        ApiError::InvalidObligationReview(message) => {
            bad_request("invalid_obligation_review", message)
        }
        ApiError::InvalidDecisionQuery(message) => bad_request("invalid_decision_query", message),
        ApiError::InvalidDecisionReview(message) => bad_request("invalid_decision_review", message),
        ApiError::InvalidRelationshipQuery(message) => {
            bad_request("invalid_relationship_query", message)
        }
        ApiError::InvalidRelationshipReview(message) => {
            bad_request("invalid_relationship_review", message)
        }
        ApiError::InvalidContradictionQuery(message) => {
            bad_request("invalid_contradiction_query", message)
        }
        ApiError::InvalidContradictionReview(message) => {
            bad_request("invalid_contradiction_review", message)
        }
        ApiError::InvalidReviewQuery(message) => bad_request("invalid_review_query", message),
        ApiError::InvalidReviewItem(message) => bad_request("invalid_review_item", message),
        ApiError::TaskCandidateNotFound => {
            not_found("task_candidate_not_found", "task candidate was not found")
        }
        ApiError::TaskCandidate(error) => internal_store(
            error,
            "task candidate store operation failed",
            "task_candidate_store_error",
        ),
        ApiError::ObligationNotFound => {
            not_found("obligation_not_found", "obligation was not found")
        }
        ApiError::Obligation(error) => internal_store(
            error,
            "obligation store operation failed",
            "obligation_store_error",
        ),
        ApiError::DecisionNotFound => not_found("decision_not_found", "decision was not found"),
        ApiError::Decision(error) => internal_store(
            error,
            "decision store operation failed",
            "decision_store_error",
        ),
        ApiError::RelationshipNotFound => {
            not_found("relationship_not_found", "relationship was not found")
        }
        ApiError::Relationship(error) => internal_store(
            error,
            "relationship store operation failed",
            "relationship_store_error",
        ),
        ApiError::ContradictionObservationNotFound => not_found(
            "contradiction_observation_not_found",
            "contradiction observation was not found",
        ),
        ApiError::Consistency(error) => {
            tracing::error!(error = %error, "consistency engine operation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "consistency_engine_error",
                "consistency engine operation failed".to_owned(),
                false,
            )
        }
        ApiError::ReviewItemNotFound => {
            not_found("review_item_not_found", "review item was not found")
        }
        ApiError::ReviewInbox(error) => internal_store(
            error,
            "review inbox store operation failed",
            "review_inbox_store_error",
        ),
        ApiError::ReviewPromotion(error) => {
            internal_store(error, "review promotion failed", "review_promotion_error")
        }
        _ => unreachable!("review response mapper received non-review ApiError"),
    }
}

fn bad_request(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::BAD_REQUEST, error, message.to_owned(), false)
}

fn not_found(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::NOT_FOUND, error, message.to_owned(), false)
}

fn internal_store(
    error: impl std::fmt::Display,
    log: &'static str,
    code: &'static str,
) -> ErrorParts {
    tracing::error!(error = %error, "{log}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        code,
        log.to_owned(),
        false,
    )
}
```

### `backend/src/app/error/response/tasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/response/tasks.rs`
- Size bytes / Размер в байтах: `479`
- Included characters / Включено символов: `479`
- Truncated / Обрезано: `no`

```rust
use axum::http::StatusCode;

use super::super::types::ApiError;
use super::ErrorParts;

pub(super) fn parts(error: ApiError) -> ErrorParts {
    match error {
        ApiError::InvalidTaskQuery(message) => bad_request("invalid_task_query", message),
        _ => unreachable!("tasks response mapper received non-task ApiError"),
    }
}

fn bad_request(error: &'static str, message: &'static str) -> ErrorParts {
    (StatusCode::BAD_REQUEST, error, message.to_owned(), false)
}
```

### `backend/src/app/error/types.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/error/types.rs`
- Size bytes / Размер в байтах: `4476`
- Included characters / Включено символов: `4476`
- Truncated / Обрезано: `no`

```rust
use std::io;

use crate::ai::control_center::AiControlCenterError;
use crate::ai::core::AiError;
use crate::application::provider_runtime_contracts::{
    TelegramError, WhatsappWebError, YandexTelemostError, ZoomError,
};
use crate::application::review_promotion::ReviewPromotionError;
use crate::domains::calendar::events::CalendarError;
use crate::domains::communications::core::CommunicationIngestionError;
use crate::domains::communications::messages::MessageProjectionError;
use crate::domains::communications::storage::CommunicationStorageError;
use crate::domains::decisions::DecisionStoreError;
use crate::domains::documents::processing::DocumentProcessingError;
use crate::domains::obligations::ObligationStoreError;
use crate::domains::organizations::api::OrganizationError;
use crate::domains::persons::api::PersonProjectionError;
use crate::domains::persons::identity::PersonIdentityError;
use crate::domains::projects::core::ProjectStoreError;
use crate::domains::projects::link_reviews::ProjectLinkReviewError;
use crate::domains::relationships::RelationshipStoreError;
use crate::domains::review::ReviewInboxError;
use crate::domains::signal_hub::SignalHubError;
use crate::domains::tasks::candidates::TaskCandidateError;
use crate::engines::automation::AutomationError;
use crate::engines::consistency::ConsistencyError;
use crate::integrations::mail::accounts::EmailAccountSetupError;
use crate::platform::audit::ApiAuditError;
use crate::platform::calls::CallError;
use crate::platform::events::{EventEnvelopeError, EventStoreError};
use crate::platform::settings::SettingsError;
use crate::platform::storage::StorageError;
use crate::vault::HostVaultError;

pub enum ApiError {
    DatabaseNotConfigured,
    InvalidEnvelope(EventEnvelopeError),
    Audit(ApiAuditError),
    Store(EventStoreError),
    Graph(crate::domains::graph::core::GraphStoreError),
    InvalidGraphQuery(&'static str),
    InvalidPersonaQuery(&'static str),
    Projects(ProjectStoreError),
    InvalidProjectQuery(&'static str),
    InvalidProjectLinkReview(&'static str),
    InvalidTaskCandidateQuery(&'static str),
    InvalidTaskCandidateReview(&'static str),
    InvalidTaskQuery(&'static str),
    InvalidObligationQuery(&'static str),
    InvalidObligationReview(&'static str),
    InvalidDecisionQuery(&'static str),
    InvalidDecisionReview(&'static str),
    InvalidRelationshipQuery(&'static str),
    InvalidRelationshipReview(&'static str),
    InvalidContradictionQuery(&'static str),
    InvalidContradictionReview(&'static str),
    InvalidReviewQuery(&'static str),
    InvalidReviewItem(&'static str),
    FailedPrecondition(String),
    InvalidPersonIdentityReview(&'static str),
    InvalidDocumentProcessingQuery(&'static str),
    Settings(SettingsError),
    SignalHub(SignalHubError),
    SettingNotFound,
    DocumentProcessing(DocumentProcessingError),
    TaskCandidateNotFound,
    TaskCandidate(TaskCandidateError),
    ObligationNotFound,
    Obligation(ObligationStoreError),
    DecisionNotFound,
    Decision(DecisionStoreError),
    RelationshipNotFound,
    Relationship(RelationshipStoreError),
    ContradictionObservationNotFound,
    ReviewItemNotFound,
    ReviewInbox(ReviewInboxError),
    ReviewPromotion(ReviewPromotionError),
    Consistency(ConsistencyError),
    AiRunNotFound,
    Ai(AiError),
    AiControlCenter(AiControlCenterError),
    Telegram(TelegramError),
    WhatsappWeb(WhatsappWebError),
    Zoom(ZoomError),
    YandexTelemost(YandexTelemostError),
    Automation(AutomationError),
    Call(CallError),
    ProjectLinkTargetNotFound,
    ProjectLinkReview(ProjectLinkReviewError),
    PersonIdentityNotFound,
    PersonProjection(PersonProjectionError),
    PersonIdentity(PersonIdentityError),
    Messages(MessageProjectionError),
    CommunicationIngestion(CommunicationIngestionError),
    CommunicationStorage(CommunicationStorageError),
    InvalidCommunicationQuery(&'static str),
    EmailAccountDeleteConflict,
    ProviderWriteConfirmationRequired,
    CommunicationMessageNotFound,
    SecretVaultNotConfigured,
    HostVault(HostVaultError),
    AccountSetup(EmailAccountSetupError),
    AccountSetupState,
    AccountSetupPendingGrantNotFound,
    AccountSetupStateMismatch,
    GraphNotFound,
    ProjectNotFound,
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
```

### `backend/src/app/guard.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/guard.rs`
- Size bytes / Размер в байтах: `1853`
- Included characters / Включено символов: `1853`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use url::form_urlencoded;

#[derive(Serialize)]
struct SecretErrorResponse {
    error: &'static str,
    message: &'static str,
}

pub async fn require_secret(
    State(expected_secret): State<String>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    if expected_secret.is_empty() {
        return Err(secret_error_response());
    }

    let ok = has_valid_secret(req.headers(), req.uri(), expected_secret.as_str());

    if ok {
        Ok(next.run(req).await)
    } else {
        Err(secret_error_response())
    }
}

fn has_valid_secret(headers: &axum::http::HeaderMap, uri: &Uri, expected_secret: &str) -> bool {
    has_valid_secret_header(headers, expected_secret)
        || has_valid_websocket_secret_query(uri, expected_secret)
}

fn has_valid_secret_header(headers: &axum::http::HeaderMap, expected_secret: &str) -> bool {
    headers
        .get("x-makosh-secret")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|secret| secret == expected_secret)
}

fn has_valid_websocket_secret_query(uri: &Uri, expected_secret: &str) -> bool {
    if uri.path() != "/api/events/ws" && uri.path() != "/api/events/realtime/ws" {
        return false;
    }

    let Some(query) = uri.query() else {
        return false;
    };

    form_urlencoded::parse(query.as_bytes())
        .any(|(name, value)| name == "makosh_secret" && value == expected_secret)
}

fn secret_error_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(SecretErrorResponse {
            error: "invalid_api_secret",
            message: "missing or invalid x-makosh-secret header",
        }),
    )
        .into_response()
}
```

### `backend/src/app/handlers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers.rs`
- Size bytes / Размер в байтах: `567`
- Included characters / Включено символов: `567`
- Truncated / Обрезано: `no`

```rust
pub(crate) mod automation;
pub(crate) mod calendar;
pub(crate) mod calls;
pub(crate) mod communications;
pub(crate) mod consistency;
pub(crate) mod decisions;
pub(crate) mod documents;
pub(crate) mod events;
pub(crate) mod graph;
pub(crate) mod obligations;
pub(crate) mod organizations;
pub(crate) mod persons;
pub(crate) mod projects;
pub(crate) mod relationships;
pub(crate) mod review;
pub(crate) mod settings;
pub(crate) mod signal_hub;
pub(crate) mod tasks;
pub(crate) mod telegram;
pub(crate) mod whatsapp;
pub(crate) mod yandex_telemost;
pub(crate) mod zoom;
```

### `backend/src/app/handlers/automation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/automation.rs`
- Size bytes / Размер в байтах: `10412`
- Included characters / Включено символов: `10412`
- Truncated / Обрезано: `no`

```rust
use std::io;

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;
use url::form_urlencoded;

use crate::ai::core::{
    AI_EMBEDDING_DIMENSION, AiAgentListResponse, AiAgentRun, AiAnswerRequest, AiError,
    AiMeetingPrepRequest, AiService, AiStatusResponse, AiTaskCandidateRefreshRequest, v3_agents,
};
use crate::domains::communications::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
};
use crate::domains::persons::analytics::{AnalyticsError, PersonAnalyticsService};
use crate::domains::persons::enrichment_engine::{EnrichmentEngineError, EnrichmentResultStore};
use crate::domains::persons::expertise::{PersonExpertiseError, PersonExpertiseStore};
use crate::domains::persons::export::{ExportError, ExportFormat, PersonExportService};
use crate::domains::persons::investigator::{InvestigatorError, PersonInvestigator};
use crate::engines::automation::{
    AutomationError, AutomationPolicy, AutomationStore, AutomationTemplate, NewAutomationPolicy,
    NewAutomationTemplate, TelegramSendDryRunRequest, TelegramSendDryRunResponse,
};
use crate::platform::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::platform::calls::{
    CallDirection, CallError, CallIntelligenceStore, CallState, CallTranscript,
    FixtureSpeechToTextProvider, NewCallTranscript, NewTelegramCall, SpeechToTextProvider,
    TelegramCall, TranscriptStatus,
};
use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};
use crate::platform::config::AppConfig;

use crate::domains::persons::health::{PersonHealthError, PersonHealthStore};

use crate::domains::persons::trust::{PersonPromiseStore, PersonRiskStore, PersonTrustError};

use crate::domains::persons::memory::{
    NewRelationshipEvent, PersonFactStore, PersonMemoryCardStore, PersonMemoryError,
    PersonPreferenceStore, RelationshipEventStore,
};

use crate::domains::persons::core::{
    NewPersonPersona, PersonCoreError, PersonIdentity, PersonPersona, PersonPersonaStore,
    PersonRole, PersonRoleStore, PersonsIdentityStore,
};
use crate::domains::persons::identity::{
    PersonIdentityCandidate, PersonIdentityDetail, PersonIdentityError,
    PersonIdentityReviewCommand, PersonIdentityReviewState, PersonIdentityStore,
};

use crate::application::email_intelligence::{EmailIntelligenceError, EmailIntelligenceService};
use crate::domains::calendar::brain::{CalendarBrainError, CalendarBrainService};
use crate::domains::calendar::core::{
    CalendarCoreError, ContextPackInput, EventAgendaStore, EventChecklistStore,
    EventContextPackStore, EventParticipantStore, EventRelationStore,
};
use crate::domains::calendar::events::{
    CalendarAccountStore, CalendarAccountUpdate, CalendarError, CalendarEventListQuery,
    CalendarEventStore, CalendarEventUpdate, CalendarSourceStore, NewCalendarEvent,
};
use crate::domains::calendar::health::{CalendarHealthError, CalendarWatchtowerService};
use crate::domains::calendar::intelligence::CalendarIntelligenceService;
use crate::domains::calendar::meetings::{
    EventRecordingStore, EventTranscriptStore, MeetingNoteStore, MeetingOutcomeStore, MeetingsError,
};
use crate::domains::calendar::reminders::{CalendarReminderStore, ReminderError};
use crate::domains::calendar::rules::{CalendarRuleError, CalendarRuleStore, RuleUpdate};
use crate::domains::calendar::scheduling::{
    DeadlineStore, FocusBlockStore, SchedulingError, SmartSchedulingService,
};
use crate::domains::calendar::sync::{export_event_ics, export_event_md};
use crate::domains::communications::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage, ProjectedMessageSummary,
    WorkflowState,
};
use crate::domains::communications::storage::{
    CommunicationStorageError, CommunicationStorageStore, StoredCommunicationAttachmentWithBlob,
};
use crate::domains::documents::processing::{
    DocumentProcessingError, DocumentProcessingJob, DocumentProcessingRecord,
    DocumentProcessingRetryCommand, DocumentProcessingRetryCommandResult, DocumentProcessingStatus,
    DocumentProcessingStore,
};
use crate::domains::graph::core::{GraphNodeKind, node_id};
use crate::domains::organizations::api::{
    OrganizationError, OrganizationStore, OrganizationUpdate,
};
use crate::domains::projects::core::{ProjectListResponse, ProjectStore, ProjectStoreError};
use crate::domains::projects::link_reviews::{
    ProjectLinkReviewCommand, ProjectLinkReviewError, ProjectLinkReviewState,
    ProjectLinkReviewStore, ProjectLinkTargetKind,
};
use crate::domains::tasks::api::{NewTask, TaskError, TaskListQuery, TaskStore, TaskUpdate};
use crate::domains::tasks::brain::{TaskBrainError, TaskBrainService};
use crate::domains::tasks::candidates::{
    TaskCandidate, TaskCandidateError, TaskCandidateReviewCommand, TaskCandidateReviewState,
    TaskCandidateStore,
};
use crate::domains::tasks::core::{
    ExternalTaskIdentityStore, TaskChecklistStore, TaskContextPackStore, TaskCoreError,
    TaskEvidenceStore, TaskProviderStore, TaskRelationStore, TaskSubtaskStore,
};
use crate::domains::tasks::health::{TaskHealthError, TaskWatchtowerService};
use crate::domains::tasks::intelligence::TaskIntelligenceService;
use crate::domains::tasks::rules::{TaskRuleError, TaskRuleStore, TaskTemplateStore};
use crate::domains::tasks::sync::{export_task_json, export_task_md};
use crate::integrations::mail::accounts::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::integrations::ollama::client::{OllamaClient, OllamaClientConfig};
use crate::platform::events::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::platform::secrets::DatabaseEncryptedSecretVault;
use crate::platform::secrets::{SecretKind, SecretReferenceStore};
use crate::platform::settings::{
    AiRuntimeSettings, ApplicationSetting, ApplicationSettingsStore, SettingsError,
};
use crate::platform::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};

use crate::app::api_support::*;
use crate::app::{ApiError, AppState};

pub(crate) async fn post_policy_template(
    State(state): State<AppState>,
    Json(request): Json<PolicyTemplateApiRequest>,
) -> Result<Json<AutomationTemplate>, ApiError> {
    let actor_id = "makosh-frontend";
    Ok(Json(
        automation_store(&state)?
            .upsert_template(&request.into_template(), actor_id)
            .await?,
    ))
}

pub(crate) async fn get_policy_templates(
    State(state): State<AppState>,
) -> Result<Json<PolicyTemplateListResponse>, ApiError> {
    let items = automation_store(&state)?.list_templates().await?;

    Ok(Json(PolicyTemplateListResponse { items }))
}

pub(crate) async fn post_policy(
    State(state): State<AppState>,
    Json(request): Json<PolicyApiRequest>,
) -> Result<Json<AutomationPolicy>, ApiError> {
    let actor_id = "makosh-frontend";
    Ok(Json(
        automation_store(&state)?
            .upsert_policy(&request.into_policy(), actor_id)
            .await?,
    ))
}

pub(crate) async fn get_policies(
    State(state): State<AppState>,
) -> Result<Json<PolicyListResponse>, ApiError> {
    let items = automation_store(&state)?.list_policies().await?;

    Ok(Json(PolicyListResponse { items }))
}

pub(crate) async fn post_telegram_send_dry_run(
    State(state): State<AppState>,
    Json(request): Json<TelegramSendDryRunRequest>,
) -> Result<Json<TelegramSendDryRunResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();
    let response = match automation_store(&state)?
        .dry_run_send(&request, &actor_id)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            if let Some(decision) = telegram_send_dry_run_rejection_decision(&error, &request) {
                api_audit_log(&state)?
                    .record(
                        &NewApiAuditRecord::automation_telegram_send_dry_run_rejected(
                            &actor_id,
                            &request.command_id,
                            &request.policy_id,
                            &request.provider_chat_id,
                            &decision,
                        ),
                    )
                    .await?;
            }
            return Err(error.into());
        }
    };
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::automation_telegram_send_dry_run(
            &actor_id,
            &response.outbound_message_id,
            &response.policy_id,
            &response.template_id,
            &response.account_id,
            &response.provider_chat_id,
            &response.rendered_preview_hash,
        ))
        .await?;

    Ok(Json(response))
}

pub(crate) fn telegram_send_dry_run_rejection_decision(
    error: &AutomationError,
    request: &TelegramSendDryRunRequest,
) -> Option<CapabilityDecision> {
    let reason = match error {
        AutomationError::InvalidRequest(_) => "invalid_request",
        AutomationError::PolicyNotFound => "policy_not_found",
        AutomationError::PolicyDisabled => "policy_disabled",
        AutomationError::ChatNotAllowed => "provider_chat_not_allowed",
        AutomationError::MissingTemplateVariable(_) => "template_variable_missing",
        AutomationError::UndeclaredTemplateVariable(_) => "template_variable_undeclared",
        AutomationError::EventEnvelope(_)
        | AutomationError::EventStore(_)
        | AutomationError::ObservationStore(_)
        | AutomationError::Sqlx(_) => return None,
    };

    Some(CapabilityDecision::rejected_high_risk(
        CapabilityActionClass::Automation,
        "telegram.send",
        reason,
        non_empty_optional_string(&request.policy_id),
    ))
}

pub(crate) fn non_empty_optional_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}
```

### `backend/src/app/handlers/calendar/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/accounts.rs`
- Size bytes / Размер в байтах: `4596`
- Included characters / Включено символов: `4482`
- Truncated / Обрезано: `no`

```rust
use super::*;

#[derive(Serialize)]
pub(crate) struct CalendarAccountsResponse {
    items: Vec<crate::domains::calendar::events::CalendarAccount>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarAccountQuery {
    provider: Option<String>,
}

pub(crate) async fn get_calendar_accounts(
    State(state): State<AppState>,
    Query(query): Query<CalendarAccountQuery>,
) -> Result<Json<CalendarAccountsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<
        crate::domains::calendar::events::CalendarAccountStore,
    >(pool)
    .list(query.provider.as_deref())
    .await?;
    Ok(Json(CalendarAccountsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewCalendarAccountRequest {
    provider: String,
    account_name: String,
    email: Option<String>,
}

pub(crate) async fn post_calendar_account(
    State(state): State<AppState>,
    Json(req): Json<NewCalendarAccountRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let acct = CalendarCommandService::new(pool)
        .create_calendar_account_manual(&req.provider, &req.account_name, req.email.as_deref())
        .await?;
    Ok(Json(acct))
}

pub(crate) async fn get_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<crate::domains::calendar::events::CalendarAccountStore>(
        pool,
    )
    .get(&account_id)
    .await?
    .map(Json)
    .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(update): Json<CalendarAccountUpdate>,
) -> Result<Json<crate::domains::calendar::events::CalendarAccount>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let acct = CalendarCommandService::new(pool)
        .update_calendar_account_manual(&account_id, &update)
        .await?;
    Ok(Json(acct))
}

pub(crate) async fn delete_calendar_account(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CalendarCommandService::new(pool)
        .delete_calendar_account_manual(&account_id)
        .await?;
    Ok(Json(json!({"deleted": true})))
}

// ── Calendar Sources ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct CalendarSourcesResponse {
    items: Vec<crate::domains::calendar::events::CalendarSource>,
}

pub(crate) async fn get_calendar_sources(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<CalendarSourcesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<
        crate::domains::calendar::events::CalendarSourceStore,
    >(pool)
    .list_by_account(&account_id)
    .await?;
    Ok(Json(CalendarSourcesResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewCalendarSourceRequest {
    name: String,
    provider_calendar_id: Option<String>,
    color: Option<String>,
    timezone: Option<String>,
}

pub(crate) async fn post_calendar_source(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(req): Json<NewCalendarSourceRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarSource>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let src = CalendarCommandService::new(pool)
        .create_calendar_source_manual(
            &account_id,
            &req.name,
            req.provider_calendar_id.as_deref(),
            req.color.as_deref(),
            req.timezone.as_deref(),
        )
        .await?;
    Ok(Json(src))
}
```

### `backend/src/app/handlers/calendar/analytics.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/analytics.rs`
- Size bytes / Размер в байтах: `2539`
- Included characters / Включено символов: `2449`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Analytics: Time Distribution ───────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct AnalyticsRangeQuery {
    from: Option<String>,
    to: Option<String>,
}

pub(crate) async fn get_time_distribution(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let dist = CalendarWatchtowerService::time_distribution(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(dist))
}

pub(crate) async fn get_focus_balance(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let balance = CalendarWatchtowerService::focus_balance(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(balance))
}

pub(crate) async fn get_back_to_back(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let now = Utc::now();
    let from: DateTime<Utc> = query
        .from
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now - chrono::Duration::days(7));
    let to: DateTime<Utc> = query
        .to
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(now);
    let b2b = CalendarWatchtowerService::back_to_back_meetings(&pool, from, to)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(b2b))
}
```

### `backend/src/app/handlers/calendar/brain.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/brain.rs`
- Size bytes / Размер в байтах: `2879`
- Included characters / Включено символов: `2411`
- Truncated / Обрезано: `no`

```rust
use super::*;

// ── Calendar Brain ─────────────────────────────────────────────────────────

pub(crate) async fn get_event_brief(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = CalendarBrainService::meeting_brief(&pool, &event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(brief))
}

pub(crate) async fn post_generate_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = CalendarBrainService::generate_agenda(&pool, &event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(agenda))
}

// ── Weekly Brief ───────────────────────────────────────────────────────────

pub(crate) async fn get_weekly_brief(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let brief = CalendarWatchtowerService::weekly_brief(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(brief))
}

// ── Calendar Analytics ─────────────────────────────────────────────────────

pub(crate) async fn get_calendar_analytics(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let load = CalendarWatchtowerService::meeting_load_analysis(&pool)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(load))
}

// ── Calendar Brain ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CalendarBrainQueryParams {
    q: String,
}

pub(crate) async fn post_calendar_brain(
    State(state): State<AppState>,
    Json(req): Json<CalendarBrainQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let answer = CalendarBrainService::answer(&pool, &req.q)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(answer))
}
```

### `backend/src/app/handlers/calendar/events.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events.rs`
- Size bytes / Размер в байтах: `299`
- Included characters / Включено символов: `299`
- Truncated / Обрезано: `no`

```rust
mod agenda;
mod checklist;
mod context_pack;
mod crud;
mod participants;
mod relations;
mod status;

pub(crate) use agenda::*;
pub(crate) use checklist::*;
pub(crate) use context_pack::*;
pub(crate) use crud::*;
pub(crate) use participants::*;
pub(crate) use relations::*;
pub(crate) use status::*;
```

### `backend/src/app/handlers/calendar/events/agenda.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events/agenda.rs`
- Size bytes / Размер в байтах: `1250`
- Included characters / Включено символов: `1250`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(crate) async fn get_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = crate::app::api_support::app_store::<EventAgendaStore>(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&agenda).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetAgendaRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_agenda(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetAgendaRequest>,
) -> Result<Json<crate::domains::calendar::core::EventAgenda>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let items = req.items;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let agenda = CalendarCommandService::new(pool)
        .set_event_agenda_manual(&event_id, items, requested_source)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(agenda))
}
```

### `backend/src/app/handlers/calendar/events/checklist.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events/checklist.rs`
- Size bytes / Размер в байтах: `1283`
- Included characters / Включено символов: `1283`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(crate) async fn get_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let checklist = crate::app::api_support::app_store::<EventChecklistStore>(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&checklist).unwrap_or_default()))
}

#[derive(Deserialize)]
pub(crate) struct SetChecklistRequest {
    items: Value,
    source: Option<String>,
}

pub(crate) async fn post_event_checklist(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<SetChecklistRequest>,
) -> Result<Json<crate::domains::calendar::core::EventChecklist>, ApiError> {
    let requested_source = req.source.as_deref().unwrap_or("manual");
    let items = req.items;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let checklist = CalendarCommandService::new(pool)
        .set_event_checklist_manual(&event_id, items, requested_source)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(checklist))
}
```

### `backend/src/app/handlers/calendar/events/context_pack.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events/context_pack.rs`
- Size bytes / Размер в байтах: `1054`
- Included characters / Включено символов: `1054`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(crate) async fn get_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = crate::app::api_support::app_store::<EventContextPackStore>(pool)
        .get(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(serde_json::to_value(&pack).unwrap_or_default()))
}

pub(crate) async fn post_event_context_pack(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<ContextPackInput>,
) -> Result<Json<crate::domains::calendar::core::EventContextPack>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let pack = crate::app::api_support::app_store::<EventContextPackStore>(pool)
        .upsert(&event_id, &req)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(pack))
}
```

### `backend/src/app/handlers/calendar/events/crud.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events/crud.rs`
- Size bytes / Размер в байтах: `3056`
- Included characters / Включено символов: `3056`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Serialize)]
pub(crate) struct CalendarEventsResponse {
    items: Vec<crate::domains::calendar::events::CalendarEvent>,
}

#[derive(Deserialize)]
pub(crate) struct CalendarEventQuery {
    account_id: Option<String>,
    source_id: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    status: Option<String>,
    event_type: Option<String>,
    limit: Option<i64>,
}

pub(crate) async fn get_calendar_events(
    State(state): State<AppState>,
    Query(query): Query<CalendarEventQuery>,
) -> Result<Json<CalendarEventsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let list_query = CalendarEventListQuery {
        account_id: query.account_id,
        source_id: query.source_id,
        from: query.from,
        to: query.to,
        status: query.status,
        event_type: query.event_type,
        limit: query.limit,
    };
    let items = crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .list(&list_query)
        .await?;
    Ok(Json(CalendarEventsResponse { items }))
}

pub(crate) async fn post_calendar_event(
    State(state): State<AppState>,
    Json(req): Json<NewCalendarEvent>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .create_manual(&req)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn get_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .get(&event_id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

pub(crate) async fn put_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(update): Json<CalendarEventUpdate>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .update_manual(&event_id, &update)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn delete_calendar_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .delete_manual(&event_id)
        .await?;
    Ok(Json(json!({"deleted": true})))
}
```

### `backend/src/app/handlers/calendar/events/participants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events/participants.rs`
- Size bytes / Размер в байтах: `1620`
- Included characters / Включено символов: `1620`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Serialize)]
pub(crate) struct EventParticipantsResponse {
    items: Vec<crate::domains::calendar::core::EventParticipant>,
}

pub(crate) async fn get_event_participants(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventParticipantsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<EventParticipantStore>(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventParticipantsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewParticipantRequest {
    email: String,
    display_name: Option<String>,
    role: Option<String>,
    person_id: Option<String>,
    organization_id: Option<String>,
}

pub(crate) async fn post_event_participant(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewParticipantRequest>,
) -> Result<Json<crate::domains::calendar::core::EventParticipant>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let participant = CalendarCommandService::new(pool)
        .add_event_participant_manual(
            &event_id,
            &req.email,
            req.display_name.as_deref(),
            req.role.as_deref(),
            req.person_id.as_deref(),
            req.organization_id.as_deref(),
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(participant))
}
```

### `backend/src/app/handlers/calendar/events/relations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events/relations.rs`
- Size bytes / Размер в байтах: `1420`
- Included characters / Включено символов: `1420`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Serialize)]
pub(crate) struct EventRelationsResponse {
    items: Vec<crate::domains::calendar::core::EventRelation>,
}

pub(crate) async fn get_event_relations(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<EventRelationsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<EventRelationStore>(pool)
        .list(&event_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(EventRelationsResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct NewRelationRequest {
    entity_type: String,
    entity_id: String,
    relation_type: String,
}

pub(crate) async fn post_event_relation(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<NewRelationRequest>,
) -> Result<Json<crate::domains::calendar::core::EventRelation>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let relation = CalendarCommandService::new(pool)
        .link_event_relation_manual(
            &event_id,
            &req.entity_type,
            &req.entity_id,
            &req.relation_type,
        )
        .await
        .map_err(ApiError::from)?;
    Ok(Json(relation))
}
```

### `backend/src/app/handlers/calendar/events/status.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/calendar/events/status.rs`
- Size bytes / Размер в байтах: `1231`
- Included characters / Включено символов: `1231`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Deserialize)]
pub(crate) struct RescheduleRequest {
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
}

pub(crate) async fn post_calendar_event_reschedule(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(req): Json<RescheduleRequest>,
) -> Result<Json<crate::domains::calendar::events::CalendarEvent>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let event = crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .reschedule_manual(&event_id, req.start_at, req.end_at)
        .await?;
    Ok(Json(event))
}

pub(crate) async fn post_calendar_event_cancel(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::app_store::<CalendarEventStore>(pool)
        .set_status_manual(
            &event_id,
            "cancelled",
            "calendar_api.post_calendar_event_cancel",
        )
        .await?;
    Ok(Json(json!({"cancelled": true})))
}
```

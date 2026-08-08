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

- Chunk ID / ID чанка: `066-source-backend-part-046`
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

### `backend/src/integrations/telegram/tdjson/qr_login/worker_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login/worker_state.rs`
- Size bytes / Размер в байтах: `1075`
- Included characters / Включено символов: `1075`
- Truncated / Обрезано: `no`

```rust
use std::path::Path;
use std::sync::mpsc::Sender;

use crate::integrations::telegram::client::TelegramQrLoginStartRequest;

use super::super::client::TdJsonClient;
use super::super::qr_login_support::{
    PendingQrLoginMap, QrLoginWorkerCompletion, TelegramQrLoginCommand,
};

pub(super) struct QrLoginWorkerContext<'a> {
    pub(super) client: &'a TdJsonClient,
    pub(super) pending_logins: &'a PendingQrLoginMap,
    pub(super) setup_id: &'a str,
    pub(super) request: &'a TelegramQrLoginStartRequest,
    pub(super) command_tx: &'a Sender<TelegramQrLoginCommand>,
    pub(super) worker_completion: &'a QrLoginWorkerCompletion,
    pub(super) database_directory: &'a Path,
}

#[derive(Default)]
pub(super) struct QrLoginRuntimeState {
    pub(super) tdlib_parameters_sent: bool,
    pub(super) database_encryption_key_checked: bool,
    pub(super) qr_requested: bool,
    pub(super) qr_link_issued: bool,
    pub(super) password_check_in_flight: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum QrLoginEventOutcome {
    Continue,
    Complete,
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support.rs`
- Size bytes / Размер в байтах: `1001`
- Included characters / Включено символов: `1001`
- Truncated / Обрезано: `no`

```rust
mod authorization;
mod completion;
mod constants;
mod identifiers;
mod identity;
mod pending;
mod qr;
mod responses;
mod types;

pub(super) use authorization::{password_hint, state_allows_qr_request};
pub(super) use completion::{
    mark_worker_complete, new_worker_completion, wait_for_worker_completion,
};
pub(super) use constants::{QR_FIRST_LINK_TIMEOUT, QR_POLL_AFTER_MS, QR_SESSION_LIFETIME};
pub(super) use identifiers::{new_setup_id, short_thread_suffix};
pub(super) use identity::{fetch_authorized_user_identity, parse_tdlib_user_identity};
pub(super) use pending::{mark_pending_ready_status, mark_pending_status, upsert_pending_response};
pub(super) use qr::render_qr_svg;
pub(super) use responses::{
    password_waiting_response, qr_preparing_response, qr_waiting_response, ready_response,
};
pub(super) use types::{
    DrainedQrLoginCommand, QrLoginWorkerCompletion, TelegramQrLoginCommand, TelegramQrLoginIdentity,
};
pub(crate) use types::{PendingQrLoginMap, TelegramQrLoginSession};
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/authorization.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/authorization.rs`
- Size bytes / Размер в байтах: `766`
- Included characters / Включено символов: `766`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

pub(in crate::integrations::telegram::tdjson) fn password_hint(
    authorization_state: &Value,
) -> Option<String> {
    authorization_state
        .get("password_hint")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(in crate::integrations::telegram::tdjson) fn state_allows_qr_request(state_type: &str) -> bool {
    matches!(
        state_type,
        "authorizationStateWaitPhoneNumber"
            | "authorizationStateWaitPremiumPurchase"
            | "authorizationStateWaitEmailAddress"
            | "authorizationStateWaitEmailCode"
            | "authorizationStateWaitCode"
            | "authorizationStateWaitRegistration"
    )
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/completion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/completion.rs`
- Size bytes / Размер в байтах: `1241`
- Included characters / Включено символов: `1241`
- Truncated / Обрезано: `no`

```rust
use std::sync::{Arc, Condvar, Mutex};

use crate::integrations::telegram::client::TelegramError;

use super::constants::QR_CANCEL_WAIT_TIMEOUT;
use super::types::QrLoginWorkerCompletion;

pub(in crate::integrations::telegram::tdjson) fn new_worker_completion() -> QrLoginWorkerCompletion
{
    Arc::new((Mutex::new(false), Condvar::new()))
}

pub(in crate::integrations::telegram::tdjson) fn mark_worker_complete(
    worker_completion: &QrLoginWorkerCompletion,
) {
    let (lock, cvar) = &**worker_completion;
    if let Ok(mut completed) = lock.lock() {
        *completed = true;
        cvar.notify_all();
    }
}

pub(in crate::integrations::telegram::tdjson) fn wait_for_worker_completion(
    worker_completion: &QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let (lock, cvar) = &**worker_completion;
    let completed = lock.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login worker lock was poisoned".to_owned())
    })?;
    if *completed {
        return Ok(());
    }
    let _ = cvar
        .wait_timeout(completed, QR_CANCEL_WAIT_TIMEOUT)
        .map_err(|_| {
            TelegramError::TdlibRuntime("Telegram QR login worker lock was poisoned".to_owned())
        })?;
    Ok(())
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/constants.rs`
- Size bytes / Размер в байтах: `567`
- Included characters / Включено символов: `567`
- Truncated / Обрезано: `no`

```rust
use std::time::Duration;

pub(in crate::integrations::telegram::tdjson) const QR_FIRST_LINK_TIMEOUT: Duration =
    Duration::from_secs(20);
pub(in crate::integrations::telegram::tdjson) const QR_SESSION_LIFETIME: Duration =
    Duration::from_secs(10 * 60);
pub(in crate::integrations::telegram::tdjson) const QR_CANCEL_WAIT_TIMEOUT: Duration =
    Duration::from_secs(5);
pub(in crate::integrations::telegram::tdjson) const QR_GET_ME_TIMEOUT: Duration =
    Duration::from_secs(5);
pub(in crate::integrations::telegram::tdjson) const QR_POLL_AFTER_MS: u64 = 2_000;
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/identifiers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/identifiers.rs`
- Size bytes / Размер в байтах: `747`
- Included characters / Включено символов: `747`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use sha2::{Digest, Sha256};

use super::super::identifiers::safe_path_segment;

pub(in crate::integrations::telegram::tdjson) fn new_setup_id(account_id: &str) -> String {
    let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(account_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(timestamp.to_string().as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!(
        "telegram-qr-{}-{}",
        safe_path_segment(account_id),
        &digest[..16]
    )
}

pub(in crate::integrations::telegram::tdjson) fn short_thread_suffix(account_id: &str) -> String {
    safe_path_segment(account_id).chars().take(32).collect()
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/identity.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/identity.rs`
- Size bytes / Размер в байтах: `3592`
- Included characters / Включено символов: `3592`
- Truncated / Обрезано: `no`

```rust
use std::time::Instant;

use serde_json::{Value, json};

use crate::integrations::telegram::client::TelegramError;

use super::super::client::TdJsonClient;
use super::super::parsing::tdlib_error_message;
use super::constants::QR_GET_ME_TIMEOUT;
use super::types::TelegramQrLoginIdentity;

pub(in crate::integrations::telegram::tdjson) fn fetch_authorized_user_identity(
    client: &TdJsonClient,
) -> Result<Option<TelegramQrLoginIdentity>, TelegramError> {
    client.send_json(&json!({
        "@type": "getMe",
        "@extra": "makosh-get-me"
    }))?;

    let started_at = Instant::now();
    while started_at.elapsed() < QR_GET_ME_TIMEOUT {
        let Some(event) = client.receive_json(1.0)? else {
            continue;
        };

        if event.get("@type").and_then(Value::as_str) == Some("user") {
            return Ok(parse_tdlib_user_identity(&event));
        }

        if event.get("@extra").and_then(Value::as_str) == Some("makosh-get-me") {
            if let Some(message) = tdlib_error_message(&event) {
                return Err(TelegramError::TdlibRuntime(message));
            }
            return Ok(parse_tdlib_user_identity(&event));
        }
    }

    Ok(None)
}

pub(in crate::integrations::telegram::tdjson) fn parse_tdlib_user_identity(
    user: &Value,
) -> Option<TelegramQrLoginIdentity> {
    let user_id = user
        .get("id")
        .and_then(|value| {
            value
                .as_i64()
                .map(|value| value.to_string())
                .or_else(|| value.as_u64().map(|value| value.to_string()))
        })
        .filter(|value| !value.trim().is_empty())?;
    let username = tdlib_user_username(user);
    let safe_user_id = safe_account_identifier(&user_id);
    let suggested_account_id = username
        .as_deref()
        .map(safe_account_identifier)
        .filter(|value| !value.is_empty())
        .map(|username| format!("{safe_user_id}_account_{username}"))
        .unwrap_or_else(|| format!("{safe_user_id}_account"));
    let suggested_display_name = username
        .as_deref()
        .map(|value| format!("@{value}"))
        .unwrap_or_else(|| user_id.clone());
    let suggested_external_account_id = format!("telegram:{user_id}");

    Some(TelegramQrLoginIdentity {
        user_id,
        username,
        suggested_account_id,
        suggested_display_name,
        suggested_external_account_id,
    })
}

fn tdlib_user_username(user: &Value) -> Option<String> {
    user.get("username")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            user.get("usernames")
                .and_then(|value| value.get("active_usernames"))
                .and_then(Value::as_array)
                .and_then(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .find(|value| !value.trim().is_empty())
                })
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
}

fn safe_account_identifier(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();

    if sanitized.is_empty() {
        "telegram".to_owned()
    } else {
        sanitized
    }
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/pending.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/pending.rs`
- Size bytes / Размер в байтах: `3031`
- Included characters / Включено символов: `3031`
- Truncated / Обрезано: `no`

```rust
use std::sync::mpsc::Sender;

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};

use super::types::{
    PendingQrLoginMap, QrLoginWorkerCompletion, TelegramQrLoginCommand, TelegramQrLoginIdentity,
    TelegramQrLoginSession,
};

pub(in crate::integrations::telegram::tdjson) fn upsert_pending_response(
    pending_logins: &PendingQrLoginMap,
    response: TelegramQrLoginStatusResponse,
    command_tx: Sender<TelegramQrLoginCommand>,
    worker_completion: QrLoginWorkerCompletion,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    pending_logins.insert(
        response.setup_id.clone(),
        TelegramQrLoginSession {
            response,
            command_tx,
            worker_completion,
        },
    );
    Ok(())
}

pub(in crate::integrations::telegram::tdjson) fn mark_pending_status(
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    status: TelegramQrLoginStatus,
    message: &str,
    poll_after_ms: u64,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    if let Some(session) = pending_logins.get_mut(setup_id) {
        let response = &mut session.response;
        response.status = status;
        response.poll_after_ms = poll_after_ms;
        response.message = Some(message.to_owned());
        if !matches!(
            status,
            TelegramQrLoginStatus::WaitingQrScan | TelegramQrLoginStatus::WaitingPassword
        ) {
            response.expires_at = None;
        }
    }
    Ok(())
}

pub(in crate::integrations::telegram::tdjson) fn mark_pending_ready_status(
    pending_logins: &PendingQrLoginMap,
    setup_id: &str,
    message: &str,
    identity: Option<&TelegramQrLoginIdentity>,
) -> Result<(), TelegramError> {
    let mut pending_logins = pending_logins.lock().map_err(|_| {
        TelegramError::TdlibRuntime("Telegram QR login state lock was poisoned".to_owned())
    })?;
    if let Some(session) = pending_logins.get_mut(setup_id) {
        let response = &mut session.response;
        response.status = TelegramQrLoginStatus::Ready;
        response.poll_after_ms = 0;
        response.message = Some(message.to_owned());
        response.expires_at = None;
        if let Some(identity) = identity {
            response.telegram_user_id = Some(identity.user_id.clone());
            response.telegram_username = identity.username.clone();
            response.suggested_account_id = Some(identity.suggested_account_id.clone());
            response.suggested_display_name = Some(identity.suggested_display_name.clone());
            response.suggested_external_account_id =
                Some(identity.suggested_external_account_id.clone());
        }
    }
    Ok(())
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/qr.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/qr.rs`
- Size bytes / Размер в байтах: `465`
- Included characters / Включено символов: `465`
- Truncated / Обрезано: `no`

```rust
use qrcode::QrCode;
use qrcode::render::svg;

use crate::integrations::telegram::client::TelegramError;

pub(in crate::integrations::telegram::tdjson) fn render_qr_svg(
    link: &str,
) -> Result<String, TelegramError> {
    let code = QrCode::new(link.as_bytes())
        .map_err(|error| TelegramError::QrGeneration(format!("failed to encode QR: {error}")))?;
    Ok(code
        .render::<svg::Color<'_>>()
        .min_dimensions(240, 240)
        .build())
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/responses.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/responses.rs`
- Size bytes / Размер в байтах: `3529`
- Included characters / Включено символов: `3529`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};

use super::constants::QR_POLL_AFTER_MS;
use super::qr::render_qr_svg;
use super::types::TelegramQrLoginIdentity;

pub(in crate::integrations::telegram::tdjson) fn qr_waiting_response(
    setup_id: &str,
    account_id: &str,
    link: &str,
) -> Result<TelegramQrLoginStatusResponse, TelegramError> {
    Ok(TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: account_id.to_owned(),
        status: TelegramQrLoginStatus::WaitingQrScan,
        qr_link: Some(link.to_owned()),
        qr_svg: Some(render_qr_svg(link)?),
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: QR_POLL_AFTER_MS,
        message: Some("Scan this QR code from an already logged-in Telegram device.".to_owned()),
    })
}

pub(in crate::integrations::telegram::tdjson) fn qr_preparing_response(
    setup_id: &str,
    account_id: &str,
) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: account_id.to_owned(),
        status: TelegramQrLoginStatus::WaitingQrScan,
        qr_link: None,
        qr_svg: None,
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: 1_000,
        message: Some("Preparing Telegram QR code.".to_owned()),
    }
}

pub(in crate::integrations::telegram::tdjson) fn password_waiting_response(
    setup_id: &str,
    account_id: &str,
    message: &str,
) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: account_id.to_owned(),
        status: TelegramQrLoginStatus::WaitingPassword,
        qr_link: None,
        qr_svg: None,
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: QR_POLL_AFTER_MS,
        message: Some(message.to_owned()),
    }
}

pub(in crate::integrations::telegram::tdjson) fn ready_response(
    setup_id: &str,
    account_id: &str,
    message: &str,
    identity: Option<&TelegramQrLoginIdentity>,
) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: setup_id.to_owned(),
        account_id: identity
            .map(|identity| identity.suggested_account_id.clone())
            .unwrap_or_else(|| account_id.to_owned()),
        status: TelegramQrLoginStatus::Ready,
        qr_link: None,
        qr_svg: None,
        telegram_user_id: identity.map(|identity| identity.user_id.clone()),
        telegram_username: identity.and_then(|identity| identity.username.clone()),
        suggested_account_id: identity.map(|identity| identity.suggested_account_id.clone()),
        suggested_display_name: identity.map(|identity| identity.suggested_display_name.clone()),
        suggested_external_account_id: identity
            .map(|identity| identity.suggested_external_account_id.clone()),
        expires_at: None,
        poll_after_ms: 0,
        message: Some(message.to_owned()),
    }
}
```

### `backend/src/integrations/telegram/tdjson/qr_login_support/types.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/qr_login_support/types.rs`
- Size bytes / Размер в байтах: `1503`
- Included characters / Включено символов: `1503`
- Truncated / Обрезано: `no`

```rust
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};

use crate::integrations::telegram::client::TelegramQrLoginStatusResponse;

pub(crate) type PendingQrLoginMap = Arc<Mutex<HashMap<String, TelegramQrLoginSession>>>;
pub(in crate::integrations::telegram::tdjson) type QrLoginWorkerCompletion =
    Arc<(Mutex<bool>, Condvar)>;

#[derive(Clone)]
pub(crate) struct TelegramQrLoginSession {
    pub(crate) response: TelegramQrLoginStatusResponse,
    pub(in crate::integrations::telegram::tdjson) command_tx: Sender<TelegramQrLoginCommand>,
    pub(in crate::integrations::telegram::tdjson) worker_completion: QrLoginWorkerCompletion,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram::tdjson) enum TelegramQrLoginCommand {
    CheckPassword(String),
    Cancel,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram::tdjson) enum DrainedQrLoginCommand {
    None,
    PasswordSubmitted,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::integrations::telegram::tdjson) struct TelegramQrLoginIdentity {
    pub(in crate::integrations::telegram::tdjson) user_id: String,
    pub(in crate::integrations::telegram::tdjson) username: Option<String>,
    pub(in crate::integrations::telegram::tdjson) suggested_account_id: String,
    pub(in crate::integrations::telegram::tdjson) suggested_display_name: String,
    pub(in crate::integrations::telegram::tdjson) suggested_external_account_id: String,
}
```

### `backend/src/integrations/telegram/tdjson/requests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/requests.rs`
- Size bytes / Размер в байтах: `18673`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::integrations::telegram::client::{TelegramError, TelegramQrLoginStartRequest};
use crate::integrations::telegram::runtime::TelegramMediaSendType;

use super::identifiers::safe_path_segment;

pub(crate) fn set_tdlib_parameters_request(
    request: &TelegramQrLoginStartRequest,
    database_directory: &Path,
) -> Result<Value, TelegramError> {
    let api_id = request.required_api_id()?;
    let api_hash = request.required_api_hash()?;
    let database_directory = database_directory.to_string_lossy().into_owned();
    let files_directory = Path::new(&database_directory)
        .join("files")
        .to_string_lossy()
        .into_owned();

    let parameters = json!({
        "use_test_dc": false,
        "database_directory": database_directory,
        "files_directory": files_directory,
        "database_encryption_key": tdlib_database_encryption_key(request),
        "use_file_database": true,
        "use_chat_info_database": true,
        "use_message_database": true,
        "use_secret_chats": false,
        "api_id": api_id,
        "api_hash": api_hash,
        "system_language_code": "en",
        "device_model": "Макошь",
        "system_version": std::env::consts::OS,
        "application_version": env!("CARGO_PKG_VERSION"),
        "enable_storage_optimizer": true,
        "ignore_file_names": false
    });

    Ok(json!({
        "@type": "setTdlibParameters",
        "parameters": parameters,
        "@extra": "makosh-set-tdlib-parameters"
    }))
}

fn tdlib_database_encryption_key(request: &TelegramQrLoginStartRequest) -> String {
    request
        .session_encryption_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| STANDARD.encode(value.as_bytes()))
        .unwrap_or_default()
}

pub(crate) fn tdlib_database_directory(request: &TelegramQrLoginStartRequest) -> PathBuf {
    request
        .tdlib_data_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("docker/data/telegram").join(safe_path_segment(&request.account_id))
        })
}

pub(crate) fn check_database_encryption_key_request(
    request: &TelegramQrLoginStartRequest,
) -> Value {
    json!({
        "@type": "checkDatabaseEncryptionKey",
        "encryption_key": tdlib_database_encryption_key(request),
        "@extra": "makosh-check-database-encryption-key"
    })
}

pub(crate) fn tdlib_load_chats_request(limit: i32, extra: &str) -> Value {
    json!({
        "@type": "loadChats",
        "chat_list": null,
        "limit": tdlib_page_limit(limit),
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chats_request(limit: i32, extra: &str) -> Value {
    json!({
        "@type": "getChats",
        "chat_list": null,
        "limit": tdlib_page_limit(limit),
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chat_request(chat_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getChat",
        "chat_id": chat_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_basic_group_request(basic_group_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getBasicGroup",
        "basic_group_id": basic_group_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_basic_group_full_info_request(basic_group_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getBasicGroupFullInfo",
        "basic_group_id": basic_group_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chat_folder_request(chat_folder_id: i64, extra: &str) -> Value {
    json!({
        "@type": "getChatFolder",
        "chat_folder_id": chat_folder_id,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_get_chat_history_request(
    chat_id: i64,
    from_message_id: Option<i64>,
    limit: i32,
    only_local: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "getChatHistory",
        "chat_id": chat_id,
        "from_message_id": from_message_id.unwrap_or(0),
        "offset": 0,
        "limit": tdlib_page_limit(limit),
        "only_local": only_local,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_send_text_message_request(
    chat_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TelegramError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "text must not be empty".to_owned(),
        ));
    }

    Ok(json!({
        "@type": "sendMessage",
        "chat_id": chat_id,
        "input_message_content": {
            "@type": "inputMessageText",
            "text": {
                "@type": "formattedText",
                "text": text,
                "entities": []
            },
            "clear_draft": true
        },
        "@extra": extra.trim()
    }))
}

pub(crate) fn tdlib_send_media_message_request(
    chat_id: i64,
    media_type: TelegramMediaSendType,
    local_path: &str,
    caption: Option<&str>,
    filename: Option<&str>,
    extra: &str,
) -> Result<Value, TelegramError> {
    let local_path = local_path.trim();
    if local_path.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "media local_path must not be empty".to_owned(),
        ));
    }
    let input_file = json!({
        "@type": "inputFileLocal",
        "path": local_path
    });
    let caption = formatted_caption(caption);
    let input_message_content = match media_type {
        TelegramMediaSendType::Photo => json!({
            "@type": "inputMessagePhoto",
            "photo": input_file,
            "thumbnail": null,
            "added_sticker_file_ids": [],
            "width": 0,
            "height": 0,
            "caption": caption,
            "show_caption_above_media": false,
            "self_destruct_type": null,
            "has_spoiler": false
        }),
        TelegramMediaSendType::Video => json!({
            "@type": "inputMessageVideo",
            "video": input_file,
            "thumbnail": null,
            "added_sticker_file_ids": [],
            "duration": 0,
            "width": 0,
            "height": 0,
            "supports_streaming": true,
            "caption": caption,
            "show_caption_above_media": false,
            "self_destruct_type": null,
            "has_spoiler": false
        }),
        TelegramMediaSendType::Document => json!({
            "@type": "inputMessageDocument",
            "document": input_file,
            "thumbnail": null,
            "disable_content_type_detection": false,
            "caption": caption
        }),
        TelegramMediaSendType::Audio => json!({
            "@type": "inputMessageAudio",
            "audio": input_file,
            "album_cover_thumbnail": null,
            "duration": 0,
            "title": filename.unwrap_or_default(),
            "performer": "",
            "caption": caption
        }),
        TelegramMediaSendType::Voice => json!({
            "@type": "inputMessageVoiceNote",
            "voice_note": input_file,
            "duration": 0,
            "waveform": "",
            "caption": caption
        }),
        TelegramMediaSendType::Sticker => json!({
            "@type": "inputMessageSticker",
            "sticker": input_file,
            "thumbnail": null,
            "emoji": "",
            "width": 0,
            "height": 0
        }),
        TelegramMediaSendType::Animation => json!({
            "@type": "inputMessageAnimation",
            "animation": input_file,
            "thumbnail": null,
            "duration": 0,
            "width": 0,
            "height": 0,
            "caption": caption,
            "show_caption_above_media": false,
            "has_spoiler": false
        }),
    };

    Ok(json!({
        "@type": "sendMessage",
        "chat_id": chat_id,
        "input_message_content": input_message_content,
        "@extra": extra.trim()
    }))
}

fn formatted_caption(caption: Option<&str>) -> Value {
    json!({
        "@type": "formattedText",
        "text": caption.unwrap_or_default().trim(),
        "entities": []
    })
}

pub(crate) fn tdlib_download_file_request(file_id: i64, priority: i32, extra: &str) -> Value {
    json!({
        "@type": "downloadFile",
        "file_id": file_id,
        "priority": priority.clamp(1, 32),
        "offset": 0,
        "limit": 0,
        "synchronous": true,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_edit_message_text_request(
    chat_id: i64,
    message_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TelegramError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "edit text must not be empty".to_owned(),
        ));
    }
    Ok(json!({
        "@type": "editMessageText",
        "chat_id": chat_id,
        "message_id": message_id,
        "input_message_content": {
            "@type": "inputMessageText",
            "text": {
                "@type": "formattedText",
                "text": text,
                "entities": []
            },
            "clear_draft": false
        },
        "@extra": extra.trim()
    }))
}

pub(crate) fn tdlib_delete_messages_request(
    chat_id: i64,
    message_ids: &[i64],
    revoke: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "deleteMessages",
        "chat_id": chat_id,
        "message_ids": message_ids,
        "revoke": revoke,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_add_message_reaction_request(
    chat_id: i64,
    message_id: i64,
    reaction_emoji: &str,
    extra: &str,
) -> Value {
    json!({
        "@type": "addMessageReaction",
        "chat_id": chat_id,
        "message_id": message_id,
        "reaction_type": {
            "@type": "reactionTypeEmoji",
            "emoji": reaction_emoji.trim()
        },
        "is_big": false,
        "update_recent_reactions": true,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_remove_message_reaction_request(
    chat_id: i64,
    message_id: i64,
    reaction_emoji: &str,
    extra: &str,
) -> Value {
    json!({
        "@type": "removeMessageReaction",
        "chat_id": chat_id,
        "message_id": message_id,
        "reaction_type": {
            "@type": "reactionTypeEmoji",
            "emoji": reaction_emoji.trim()
        },
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_pin_chat_message_request(
    chat_id: i64,
    message_id: i64,
    disable_notification: bool,
    extra: &str,
) -> Value {
    json!({
        "@type": "pinChatMessage",
        "chat_id": chat_id,
        "message_id": message_id,
        "disable_notification": disable_notification,
        "only_for_self": false,
        "@extra": extra.trim()
    })
}

pub(crate) fn tdlib_send_reply_request(
    chat_id: i64,
    reply_to_message_id: i64,
    text: &str,
    extra: &str,
) -> Result<Value, TelegramError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(TelegramError::InvalidRequest(
            "reply text must not be empty".to_owned(),
        ));
    }
    Ok(json!({
        "@type": "sendMessage",
        "chat_id": chat_id,
        "reply_to": {
            "@type": "inputMessageReplyToMessage",
            "message_id": reply_to_message_id
        },
        "input_message_content": {
            "@type": "inputMessageText",
            "text": {
                "@type": "formattedText",
                "text": text,
                "entities": []
            },
            "clear_draft": true
        },
        "@extra": extra.trim()
    }))
}

pub(crate) fn tdlib_send_forward_request(
    chat_id: i64,
    from_chat_id: i64,
    message_id: i64,
    extra: &str,
) -> Va
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/tdjson/snapshots.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/snapshots.rs`
- Size bytes / Размер в байтах: `3913`
- Included characters / Включено символов: `3913`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::{TelegramChatKind, TelegramDeliveryState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibTopicSnapshot {
    pub(crate) provider_topic_id: i64,
    pub(crate) title: String,
    pub(crate) icon_emoji: Option<String>,
    pub(crate) is_pinned: bool,
    pub(crate) is_closed: bool,
    pub(crate) unread_count: i64,
    pub(crate) last_message_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) chat_kind: TelegramChatKind,
    pub(crate) title: String,
    pub(crate) username: Option<String>,
    pub(crate) last_message_at: Option<DateTime<Utc>>,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatFolderSnapshot {
    pub(crate) provider_folder_id: i64,
    pub(crate) title: String,
    pub(crate) icon_name: Option<String>,
    pub(crate) color_id: Option<i64>,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibChatMemberSnapshot {
    pub(crate) provider_member_id: String,
    pub(crate) display_name: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) role: String,
    pub(crate) status: String,
    pub(crate) is_admin: bool,
    pub(crate) is_owner: bool,
    pub(crate) permissions: Value,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibMessageSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) sender_id: String,
    pub(crate) sender_display_name: String,
    pub(crate) text: String,
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) delivery_state: TelegramDeliveryState,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibMessageDeleteSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_ids: Vec<String>,
    pub(crate) is_permanent: bool,
    pub(crate) from_cache: bool,
    pub(crate) source_event: String,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibMessageInteractionInfoSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) source_event: String,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibMessageContentSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) text: String,
    pub(crate) new_content: Value,
    pub(crate) source_event: String,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibMessageEditedSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) edit_timestamp: DateTime<Utc>,
    pub(crate) reply_markup: Option<Value>,
    pub(crate) source_event: String,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibMessagePinnedSnapshot {
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) is_pinned: bool,
    pub(crate) source_event: String,
    pub(crate) raw: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TelegramTdlibFileSnapshot {
    pub(crate) file_id: i64,
    pub(crate) size_bytes: Option<i64>,
    pub(crate) expected_size_bytes: Option<i64>,
    pub(crate) local_path: Option<String>,
    pub(crate) is_downloading_active: bool,
    pub(crate) is_downloading_completed: bool,
    pub(crate) downloaded_size_bytes: Option<i64>,
    pub(crate) remote_id: Option<String>,
    pub(crate) remote_unique_id: Option<String>,
    pub(crate) raw: Value,
}
```

### `backend/src/integrations/telegram/tdjson/tests.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/tdjson/tests.rs`
- Size bytes / Размер в байтах: `825`
- Included characters / Включено символов: `825`
- Truncated / Обрезано: `no`

```rust
use crate::integrations::telegram::client::{TelegramQrLoginStatus, TelegramQrLoginStatusResponse};

mod environment;
mod parsing_snapshots;
mod qr_login_flows;
mod request_builders;

fn test_qr_login_response(status: TelegramQrLoginStatus) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: "setup-id".to_owned(),
        account_id: "telegram-account".to_owned(),
        status,
        qr_link: Some("tg://login?token=test-token".to_owned()),
        qr_svg: Some("<svg></svg>".to_owned()),
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: 2_000,
        message: Some("Waiting".to_owned()),
    }
}
```

### `backend/src/integrations/whatsapp/client.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client.rs`
- Size bytes / Размер в байтах: `1718`
- Included characters / Включено символов: `1718`
- Truncated / Обрезано: `no`

```rust
mod constants;
mod errors;
mod ids;
mod models;
mod rows;
mod store;
mod validation;

pub use errors::WhatsappWebError;
pub use models::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebSession, NewWhatsappWebStatus,
    NewWhatsappWebStatusDelete, NewWhatsappWebStatusView, WhatsappLiveAccountSetupRequest,
    WhatsappWebAccountSetupRequest, WhatsappWebAccountSetupResponse, WhatsappWebCallIngestResult,
    WhatsappWebCompanionRuntime, WhatsappWebDeliveryState, WhatsappWebDialogIngestResult,
    WhatsappWebLinkState, WhatsappWebMediaIngestResult, WhatsappWebMessage,
    WhatsappWebMessageDeleteIngestResult, WhatsappWebMessageIngestResult,
    WhatsappWebMessageUpdateIngestResult, WhatsappWebObservedCall, WhatsappWebObservedDialog,
    WhatsappWebObservedMedia, WhatsappWebObservedMessage, WhatsappWebObservedMessageDelete,
    WhatsappWebObservedMessageUpdate, WhatsappWebObservedParticipant, WhatsappWebObservedPresence,
    WhatsappWebObservedReaction, WhatsappWebObservedReceipt, WhatsappWebObservedRuntimeEvent,
    WhatsappWebObservedStatus, WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView,
    WhatsappWebParticipantIngestResult, WhatsappWebPresenceIngestResult,
    WhatsappWebReactionIngestResult, WhatsappWebReceiptIngestResult,
    WhatsappWebRuntimeEventIngestResult, WhatsappWebSession, WhatsappWebStatusDeleteIngestResult,
    WhatsappWebStatusIngestResult, WhatsappWebStatusViewIngestResult,
};
pub use store::WhatsappWebStore;
```

### `backend/src/integrations/whatsapp/client/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/constants.rs`
- Size bytes / Размер в байтах: `1206`
- Included characters / Включено символов: `1206`
- Truncated / Обрезано: `no`

```rust
pub(crate) const WHATSAPP_WEB_MESSAGE_RECORD_KIND: &str = "whatsapp_web_message";
pub(crate) const WHATSAPP_WEB_REACTION_RECORD_KIND: &str = "whatsapp_web_reaction";
pub(crate) const WHATSAPP_WEB_MEDIA_RECORD_KIND: &str = "whatsapp_web_media";
pub(crate) const WHATSAPP_WEB_STATUS_RECORD_KIND: &str = "whatsapp_web_status";
pub(crate) const WHATSAPP_WEB_STATUS_VIEW_RECORD_KIND: &str = "whatsapp_web_status_view";
pub(crate) const WHATSAPP_WEB_STATUS_DELETE_RECORD_KIND: &str = "whatsapp_web_status_delete";
pub(crate) const WHATSAPP_WEB_PRESENCE_RECORD_KIND: &str = "whatsapp_web_presence";
pub(crate) const WHATSAPP_WEB_CALL_RECORD_KIND: &str = "whatsapp_web_call";
pub(crate) const WHATSAPP_WEB_RUNTIME_EVENT_RECORD_KIND: &str = "whatsapp_web_runtime_event";
pub(crate) const WHATSAPP_WEB_DIALOG_RECORD_KIND: &str = "whatsapp_web_dialog";
pub(crate) const WHATSAPP_WEB_PARTICIPANT_RECORD_KIND: &str = "whatsapp_web_participant";
pub(crate) const WHATSAPP_WEB_MESSAGE_UPDATE_RECORD_KIND: &str = "whatsapp_web_message_update";
pub(crate) const WHATSAPP_WEB_MESSAGE_DELETE_RECORD_KIND: &str = "whatsapp_web_message_delete";
pub(crate) const WHATSAPP_WEB_RECEIPT_RECORD_KIND: &str = "whatsapp_web_receipt";
```

### `backend/src/integrations/whatsapp/client/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/errors.rs`
- Size bytes / Размер в байтах: `976`
- Included characters / Включено символов: `976`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

use crate::platform::communications::ProviderCommunicationMessagePortError;
use crate::platform::observations::ObservationStoreError;
use crate::platform::secrets::{SecretReferenceError, SecretResolutionError};
use crate::vault::HostVaultError;

#[derive(Debug, Error)]
pub enum WhatsappWebError {
    #[error("invalid WhatsApp Web request: {0}")]
    InvalidRequest(String),

    #[error("WhatsApp Web provider account store operation failed: {0}")]
    ProviderAccountStore(String),

    #[error(transparent)]
    CommunicationMessagePort(#[from] ProviderCommunicationMessagePortError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    SecretResolution(#[from] SecretResolutionError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/integrations/whatsapp/client/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/ids.rs`
- Size bytes / Размер в байтах: `908`
- Included characters / Включено символов: `908`
- Truncated / Обрезано: `no`

```rust
use sha2::{Digest, Sha256};

pub(crate) fn whatsapp_web_session_id(account_id: &str) -> String {
    format!(
        "whatsapp_web_session:v5:{}",
        stable_hash(account_id.as_bytes())
    )
}

pub(crate) fn whatsapp_web_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v5:whatsapp_web:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

pub(crate) fn whatsapp_web_raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
) -> String {
    format!(
        "raw:v5:whatsapp_web:{}",
        stable_hash(
            [account_id, record_kind, provider_record_id]
                .join("\0")
                .as_bytes()
        )
    )
}

fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
```

### `backend/src/integrations/whatsapp/client/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/models.rs`
- Size bytes / Размер в байтах: `40800`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::platform::communications::CommunicationProviderKind;
use crate::platform::communications::NewRawCommunicationRecord;

use super::errors::WhatsappWebError;
use super::validation::{validate_non_empty, validate_object, validate_string_array};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WhatsappWebAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_shape: Option<String>,
    pub display_name: String,
    pub external_account_id: String,
    pub device_name: String,
    pub local_state_path: String,
}

#[derive(Clone, Deserialize, Eq, PartialEq)]
pub struct WhatsappLiveAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub provider_shape: String,
    pub display_name: String,
    pub external_account_id: String,
    pub device_name: Option<String>,
    pub local_state_path: Option<String>,
    pub api_access_token: Option<String>,
    pub app_secret: Option<String>,
    pub webhook_verify_token: Option<String>,
}

impl WhatsappLiveAccountSetupRequest {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        let provider_shape = validate_non_empty("provider_shape", &self.provider_shape)?;
        if provider_shape == "whatsapp_business_cloud" && self.device_name.is_some() {
            return Err(WhatsappWebError::InvalidRequest(
                "device_name is not supported for whatsapp_business_cloud".to_owned(),
            ));
        }
        if provider_shape == "whatsapp_business_cloud" {
            validate_non_empty(
                "api_access_token",
                self.api_access_token.as_deref().unwrap_or_default(),
            )?;
        } else if self.api_access_token.is_some() {
            return Err(WhatsappWebError::InvalidRequest(
                "api_access_token is only supported for whatsapp_business_cloud".to_owned(),
            ));
        } else if self.app_secret.is_some() || self.webhook_verify_token.is_some() {
            return Err(WhatsappWebError::InvalidRequest(
                "app_secret and webhook_verify_token are only supported for whatsapp_business_cloud"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

impl fmt::Debug for WhatsappLiveAccountSetupRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WhatsappLiveAccountSetupRequest")
            .field("account_id", &self.account_id)
            .field("provider_kind", &self.provider_kind)
            .field("provider_shape", &self.provider_shape)
            .field("display_name", &self.display_name)
            .field("external_account_id", &self.external_account_id)
            .field("device_name", &self.device_name)
            .field("local_state_path", &self.local_state_path)
            .field(
                "api_access_token",
                &self.api_access_token.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "app_secret",
                &self.app_secret.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "webhook_verify_token",
                &self.webhook_verify_token.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl WhatsappWebAccountSetupRequest {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        if let Some(provider_shape) = self.provider_shape.as_deref() {
            validate_non_empty("provider_shape", provider_shape)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub session: WhatsappWebSession,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewWhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: WhatsappWebCompanionRuntime,
    pub link_state: WhatsappWebLinkState,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewWhatsappWebSession {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("session_id", &self.session_id)?;
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("device_name", &self.device_name)?;
        validate_non_empty("local_state_path", &self.local_state_path)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WhatsappWebSession {
    pub session_id: String,
    pub account_id: String,
    pub device_name: String,
    pub companion_runtime: String,
    pub link_state: String,
    pub local_state_path: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebCompanionRuntime {
    Fixture,
    ManualWebview,
    Blocked,
    ApiCredentials,
}

impl WhatsappWebCompanionRuntime {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::ManualWebview => "manual_webview",
            Self::Blocked => "blocked",
            Self::ApiCredentials => "api_credentials",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsappWebLinkState {
    Fixture,
    QrPending,
    PairCodePending,
    Linked,
    Degraded,
    Revoked,
    Blocked,
}

impl WhatsappWebLinkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::QrPending => "qr_pending",
            Self::PairCodePending => "pair_code_pending",
            Self::Linked => "linked",
            Self::Degraded => "degraded",
            Self::Revoked => "revoked",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub reply_to_provider_message_id: Option<String>,
    pub forward_origin_chat_id: Option<String>,
    pub forward_origin_message_id: Option<String>,
    pub forward_origin_sender_id: Option<String>,
    pub forward_origin_sender_name: Option<String>,
    pub forwarded_at: Option<DateTime<Utc>>,
    #[serde(default = "default_json_object")]
    pub message_metadata: Value,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: WhatsappWebDeliveryState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebReaction {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_actor_id: String,
    pub sender_display_name: String,
    pub reaction: String,
    pub is_active: bool,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebReaction {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("provider_actor_id", &self.provider_actor_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("reaction", &self.reaction)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn provider_record_id(&self) -> String {
        format!(
            "{}:{}:{}",
            self.provider_message_id.trim(),
            self.provider_actor_id.trim(),
            self.reaction.trim()
        )
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_message_id,
            &self.provider_actor_id,
            &self.reaction,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebMedia {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub storage_kind: String,
    pub storage_path: String,
    pub import_batch_id: String,
    pub observed_at: DateTime<Utc>,
}

impl NewWhatsappWebMedia {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("provider_attachment_id", &self.provider_attachment_id)?;
        validate_non_empty("content_type", &self.content_type)?;
        validate_non_empty("sha256", &self.sha256)?;
        validate_non_empty("storage_kind", &self.storage_kind)?;
        validate_non_empty("storage_path", &self.storage_path)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    pub(crate) fn provider_record_id(&self) -> String {
        format!(
            "{}:{}",
            self.provider_message_id.trim(),
            self.provider_attachment_id.trim()
        )
    }

    pub(crate) fn source_fingerprint(&self) -> String {
        stable_source_fingerprint(&[
            &self.account_id,
            &self.provider_chat_id,
            &self.provider_message_id,
            &self.provider_attachment_id,
            &self.sha256,
        ])
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewWhatsappWebStatus {
    pub account_id: String,
    pub provider_status_id: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub sender_identity_kind: Option<String>,
    pub sender_address: Option<String>,
    pub sender_push_name: Option<String>,
    #[serde(default = "default_json_object")]
    pub sender_business_profile: Value,
    #[serde(default = "default_json_object")]
    pub sender_profile_photo_ref: Value,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl NewWhatsappWebStatus {
    pub(crate) fn validate(&self) -> Result<(), WhatsappWebError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_status_id", &self.provider_status_id)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_optional_non_empty("sender_identity_kind", self.sender_identity_kind.as_deref())?;
        validate_optional_non_empty("sender_address", self.sender_address.as_deref())?;
        validate_optional_non_empty("sender_push_n
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/whatsapp/client/rows.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/rows.rs`
- Size bytes / Размер в байтах: `2520`
- Included characters / Включено символов: `2520`
- Truncated / Обрезано: `no`

```rust
use sqlx::Row;
use sqlx::postgres::PgRow;

use crate::platform::communications::ProviderChannelMessage;

use super::errors::WhatsappWebError;
use super::models::{WhatsappWebMessage, WhatsappWebSession};

pub(crate) fn row_to_whatsapp_web_session(
    row: PgRow,
) -> Result<WhatsappWebSession, WhatsappWebError> {
    Ok(WhatsappWebSession {
        session_id: row.try_get("session_id")?,
        account_id: row.try_get("account_id")?,
        device_name: row.try_get("device_name")?,
        companion_runtime: row.try_get("companion_runtime")?,
        link_state: row.try_get("link_state")?,
        local_state_path: row.try_get("local_state_path")?,
        last_sync_at: row.try_get("last_sync_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(crate) fn row_to_whatsapp_web_message(
    row: PgRow,
) -> Result<WhatsappWebMessage, WhatsappWebError> {
    Ok(WhatsappWebMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_record_id")?,
        provider_chat_id: row.try_get("conversation_id")?,
        chat_title: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        sender_display_name: row.try_get("sender_display_name")?,
        text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        delivery_state: row.try_get("delivery_state")?,
        metadata: row.try_get("message_metadata")?,
    })
}

pub(crate) fn provider_channel_message_to_whatsapp_web_message(
    message: ProviderChannelMessage,
) -> WhatsappWebMessage {
    WhatsappWebMessage {
        message_id: message.message_id,
        raw_record_id: message.raw_record_id,
        account_id: message.account_id,
        provider_message_id: message.provider_record_id,
        provider_chat_id: Some(message.conversation_id),
        chat_title: message.subject,
        sender: message.sender,
        sender_display_name: message.sender_display_name,
        text: message.body_text,
        occurred_at: message.occurred_at,
        projected_at: message.projected_at,
        channel_kind: message.channel_kind,
        delivery_state: message.delivery_state,
        metadata: message.message_metadata,
    }
}
```

### `backend/src/integrations/whatsapp/client/store.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/store.rs`
- Size bytes / Размер в байтах: `1794`
- Included characters / Включено символов: `1794`
- Truncated / Обрезано: `no`

```rust
mod accounts;
mod evidence;
mod ingestion;
mod intelligence;
mod queries;
mod sessions;

use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderChannelMessageLookupPort, ProviderSecretBindingCommandPort,
};

#[derive(Clone)]
pub struct WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
}

impl WhatsappWebStore {
    pub fn new(
        pool: PgPool,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
        provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    ) -> Self {
        Self {
            pool,
            provider_account_store,
            provider_secret_binding_store,
            provider_channel_message_store,
        }
    }

    pub(in crate::integrations::whatsapp) fn provider_account_store(
        &self,
    ) -> &dyn ProviderAccountCommandPort {
        self.provider_account_store.as_ref()
    }

    pub(in crate::integrations::whatsapp) fn provider_secret_binding_store(
        &self,
    ) -> &dyn ProviderSecretBindingCommandPort {
        self.provider_secret_binding_store.as_ref()
    }

    pub(in crate::integrations::whatsapp) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(in crate::integrations::whatsapp::client) fn provider_channel_message_store(
        &self,
    ) -> &dyn ProviderChannelMessageLookupPort {
        self.provider_channel_message_store.as_ref()
    }
}
```

### `backend/src/integrations/whatsapp/client/store/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/store/accounts.rs`
- Size bytes / Размер в байтах: `9944`
- Included characters / Включено символов: `9944`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::platform::communications::{CommunicationProviderKind, NewProviderAccount};

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::ids::whatsapp_web_session_id;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebSession, WhatsappLiveAccountSetupRequest, WhatsappWebAccountSetupRequest,
    WhatsappWebAccountSetupResponse, WhatsappWebCompanionRuntime, WhatsappWebLinkState,
};

impl WhatsappWebStore {
    pub async fn setup_fixture_account(
        &self,
        request: &WhatsappWebAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        if !request.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(
                "provider_kind must be a WhatsApp provider".to_owned(),
            ));
        }
        let provider_shape = normalize_fixture_provider_shape(
            request.provider_kind,
            request.provider_shape.as_deref(),
        )?;
        let session_mode = fixture_session_mode(request.provider_kind);
        let setup_semantics = fixture_setup_semantics(request.provider_kind);

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "provider_shape": provider_shape,
            "local_state_path": request.local_state_path,
            "device_name": request.device_name,
            "lifecycle_state": "created",
            "setup_semantics": setup_semantics,
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name: request.device_name.clone(),
                companion_runtime: fixture_companion_runtime(request.provider_kind),
                link_state: WhatsappWebLinkState::Fixture,
                local_state_path: request.local_state_path.clone(),
                last_sync_at: None,
                metadata: json!({
                    "runtime": "fixture",
                    "provider_shape": provider_shape,
                    "setup_semantics": setup_semantics,
                    "session_mode": session_mode,
                }),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            session,
        })
    }

    pub async fn setup_live_blocked_account(
        &self,
        request: &WhatsappLiveAccountSetupRequest,
    ) -> Result<WhatsappWebAccountSetupResponse, WhatsappWebError> {
        request.validate()?;
        let provider_shape = normalize_provider_shape(&request.provider_shape)?;
        validate_live_provider_kind(request.provider_kind, provider_shape)?;
        let device_name = default_live_device_name(provider_shape, request.device_name.clone());
        let local_state_path = default_live_local_state_path(
            provider_shape,
            &request.account_id,
            request.local_state_path.clone(),
        );

        let account = NewProviderAccount::new(
            &request.account_id,
            request.provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "live_blocked",
            "provider_shape": provider_shape,
            "local_state_path": local_state_path,
            "device_name": device_name,
            "lifecycle_state": "created",
            "setup_semantics": live_setup_semantics(provider_shape),
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?;

        let session = self
            .upsert_session(&NewWhatsappWebSession {
                session_id: whatsapp_web_session_id(&request.account_id),
                account_id: stored_account.account_id.clone(),
                device_name,
                companion_runtime: live_companion_runtime(provider_shape),
                link_state: WhatsappWebLinkState::Blocked,
                local_state_path,
                last_sync_at: None,
                metadata: json!({
                    "runtime": "live_blocked",
                    "provider_shape": provider_shape,
                    "setup_semantics": live_setup_semantics(provider_shape),
                    "session_mode": live_session_mode(provider_shape),
                }),
            })
            .await?;

        Ok(WhatsappWebAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "live_blocked".to_owned(),
            session,
        })
    }
}

fn normalize_provider_shape(input: &str) -> Result<&str, WhatsappWebError> {
    let normalized = input.trim();
    match normalized {
        "whatsapp_web_companion" | "whatsapp_native_md" | "whatsapp_business_cloud" => {
            Ok(normalized)
        }
        _ => Err(WhatsappWebError::InvalidRequest(format!(
            "unsupported WhatsApp provider_shape `{input}`"
        ))),
    }
}

fn normalize_fixture_provider_shape(
    provider_kind: CommunicationProviderKind,
    requested_shape: Option<&str>,
) -> Result<&'static str, WhatsappWebError> {
    match requested_shape {
        Some(input) => {
            let normalized = normalize_provider_shape(input)?;
            validate_live_provider_kind(provider_kind, normalized)?;
            Ok(match normalized {
                "whatsapp_web_companion" => "whatsapp_web_companion",
                "whatsapp_native_md" => "whatsapp_native_md",
                "whatsapp_business_cloud" => "whatsapp_business_cloud",
                _ => unreachable!("normalize_provider_shape returned unsupported value"),
            })
        }
        None => Ok(fixture_provider_shape(provider_kind)),
    }
}

fn validate_live_provider_kind(
    provider_kind: CommunicationProviderKind,
    provider_shape: &str,
) -> Result<(), WhatsappWebError> {
    let expected_kind = match provider_shape {
        "whatsapp_business_cloud" => CommunicationProviderKind::WhatsappBusinessCloud,
        _ => CommunicationProviderKind::WhatsappWeb,
    };
    if provider_kind != expected_kind {
        return Err(WhatsappWebError::InvalidRequest(format!(
            "provider_kind `{}` is invalid for provider_shape `{provider_shape}`; expected `{}`",
            provider_kind.as_str(),
            expected_kind.as_str(),
        )));
    }
    Ok(())
}

fn default_live_device_name(provider_shape: &str, request_value: Option<String>) -> String {
    match provider_shape {
        "whatsapp_business_cloud" => "WhatsApp Business Cloud API".to_owned(),
        _ => request_value.unwrap_or_else(|| format!("{provider_shape} blocked runtime")),
    }
}

fn default_live_local_state_path(
    provider_shape: &str,
    account_id: &str,
    request_value: Option<String>,
) -> String {
    request_value.unwrap_or_else(|| match provider_shape {
        "whatsapp_business_cloud" => {
            format!("docker/data/whatsapp/business-cloud/{account_id}")
        }
        _ => format!("docker/data/whatsapp/blocked/{account_id}"),
    })
}

fn live_setup_semantics(provider_shape: &str) -> &'static str {
    match provider_shape {
        "whatsapp_business_cloud" => "business_cloud",
        _ => "personal_runtime",
    }
}

fn live_session_mode(provider_shape: &str) -> &'static str {
    match provider_shape {
        "whatsapp_business_cloud" => "api_credentials",
        _ => "device_session",
    }
}

fn live_companion_runtime(provider_shape: &str) -> WhatsappWebCompanionRuntime {
    match provider_shape {
        "whatsapp_business_cloud" => WhatsappWebCompanionRuntime::ApiCredentials,
        _ => WhatsappWebCompanionRuntime::Blocked,
    }
}

fn fixture_provider_shape(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => "whatsapp_business_cloud",
        CommunicationProviderKind::WhatsappWeb => "whatsapp_web_companion",
        _ => "whatsapp_web_companion",
    }
}

fn fixture_setup_semantics(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => "business_cloud",
        CommunicationProviderKind::WhatsappWeb => "personal_runtime",
        _ => "personal_runtime",
    }
}

fn fixture_session_mode(provider_kind: CommunicationProviderKind) -> &'static str {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => "api_credentials",
        CommunicationProviderKind::WhatsappWeb => "device_session",
        _ => "device_session",
    }
}

fn fixture_companion_runtime(
    provider_kind: CommunicationProviderKind,
) -> WhatsappWebCompanionRuntime {
    match provider_kind {
        CommunicationProviderKind::WhatsappBusinessCloud => {
            WhatsappWebCompanionRuntime::ApiCredentials
        }
        CommunicationProviderKind::WhatsappWeb => WhatsappWebCompanionRuntime::Fixture,
        _ => WhatsappWebCompanionRuntime::Fixture,
    }
}
```

### `backend/src/integrations/whatsapp/client/store/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/store/evidence.rs`
- Size bytes / Размер в байтах: `700`
- Included characters / Включено символов: `700`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{ObservationStoreError, link_domain_entity_in_transaction};

pub(super) async fn link_whatsapp_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    metadata: Value,
) -> Result<(), ObservationStoreError> {
    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        "communications",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}
```

### `backend/src/integrations/whatsapp/client/store/ingestion.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/store/ingestion.rs`
- Size bytes / Размер в байтах: `33044`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::{Map, Value, json};

use crate::platform::communications::NewRawCommunicationRecord;

use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::constants::{
    WHATSAPP_WEB_CALL_RECORD_KIND, WHATSAPP_WEB_DIALOG_RECORD_KIND, WHATSAPP_WEB_MEDIA_RECORD_KIND,
    WHATSAPP_WEB_MESSAGE_DELETE_RECORD_KIND, WHATSAPP_WEB_MESSAGE_RECORD_KIND,
    WHATSAPP_WEB_MESSAGE_UPDATE_RECORD_KIND, WHATSAPP_WEB_PARTICIPANT_RECORD_KIND,
    WHATSAPP_WEB_PRESENCE_RECORD_KIND, WHATSAPP_WEB_REACTION_RECORD_KIND,
    WHATSAPP_WEB_RECEIPT_RECORD_KIND, WHATSAPP_WEB_RUNTIME_EVENT_RECORD_KIND,
    WHATSAPP_WEB_STATUS_DELETE_RECORD_KIND, WHATSAPP_WEB_STATUS_RECORD_KIND,
    WHATSAPP_WEB_STATUS_VIEW_RECORD_KIND,
};
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::integrations::whatsapp::client::ids::whatsapp_web_raw_record_id;
use crate::integrations::whatsapp::client::models::{
    NewWhatsappWebCall, NewWhatsappWebDialog, NewWhatsappWebMedia, NewWhatsappWebMessage,
    NewWhatsappWebMessageDelete, NewWhatsappWebMessageUpdate, NewWhatsappWebParticipant,
    NewWhatsappWebPresence, NewWhatsappWebReaction, NewWhatsappWebReceipt,
    NewWhatsappWebRuntimeEvent, NewWhatsappWebStatus, NewWhatsappWebStatusDelete,
    NewWhatsappWebStatusView, WhatsappWebObservedCall, WhatsappWebObservedDialog,
    WhatsappWebObservedMedia, WhatsappWebObservedMessage, WhatsappWebObservedMessageDelete,
    WhatsappWebObservedMessageUpdate, WhatsappWebObservedParticipant, WhatsappWebObservedPresence,
    WhatsappWebObservedReaction, WhatsappWebObservedReceipt, WhatsappWebObservedRuntimeEvent,
    WhatsappWebObservedStatus, WhatsappWebObservedStatusDelete, WhatsappWebObservedStatusView,
};

impl WhatsappWebStore {
    pub async fn ingest_fixture_message(
        &self,
        message: &NewWhatsappWebMessage,
    ) -> Result<WhatsappWebObservedMessage, WhatsappWebError> {
        message.validate()?;
        let provider_account = self
            .provider_account_store()
            .get(&message.account_id)
            .await
            .map_err(|error| WhatsappWebError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` is not configured",
                    message.account_id
                ))
            })?;
        if !provider_account.provider_kind.is_whatsapp() {
            return Err(WhatsappWebError::InvalidRequest(format!(
                "account `{}` is not a WhatsApp Web provider account",
                message.account_id
            )));
        }

        let session = self
            .list_sessions(Some(&message.account_id), 1)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| {
                WhatsappWebError::InvalidRequest(format!(
                    "WhatsApp Web account `{}` has no session metadata",
                    message.account_id
                ))
            })?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            WHATSAPP_WEB_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            json!({
                "provider_chat_id": message.provider_chat_id,
                "chat_title": message.chat_title,
                "sender_id": message.sender_id,
                "sender_display_name": message.sender_display_name,
                "text": message.text,
                "reply_to_provider_message_id": message.reply_to_provider_message_id,
                "forward_origin_chat_id": message.forward_origin_chat_id,
                "forward_origin_message_id": message.forward_origin_message_id,
                "forward_origin_sender_id": message.forward_origin_sender_id,
                "forward_origin_sender_name": message.forward_origin_sender_name,
                "forwarded_at": message.forwarded_at,
                "message_metadata": normalized_whatsapp_message_metadata(
                    &message.text,
                    &message.message_metadata,
                ),
                "delivery_state": message.delivery_state.as_str(),
            }),
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": session.companion_runtime,
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        self.update_session_last_sync(&message.account_id, message.occurred_at)
            .await?;

        Ok(WhatsappWebObservedMessage { raw })
    }

    pub async fn ingest_fixture_reaction(
        &self,
        reaction: &NewWhatsappWebReaction,
    ) -> Result<WhatsappWebObservedReaction, WhatsappWebError> {
        reaction.validate()?;
        let context = self.fixture_ingest_context(&reaction.account_id).await?;
        let provider_record_id = reaction.provider_record_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &reaction.account_id,
            WHATSAPP_WEB_REACTION_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &reaction.account_id,
            WHATSAPP_WEB_REACTION_RECORD_KIND,
            &provider_record_id,
            reaction.source_fingerprint(),
            &reaction.import_batch_id,
            json!({
                "provider_chat_id": reaction.provider_chat_id,
                "provider_message_id": reaction.provider_message_id,
                "provider_actor_id": reaction.provider_actor_id,
                "sender_display_name": reaction.sender_display_name,
                "reaction": reaction.reaction,
                "is_active": reaction.is_active,
            }),
        )
        .occurred_at(reaction.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": reaction.account_id,
            "provider_chat_id": reaction.provider_chat_id,
        }));
        self.update_session_last_sync(&reaction.account_id, reaction.observed_at)
            .await?;

        Ok(WhatsappWebObservedReaction { raw })
    }

    pub async fn ingest_fixture_media(
        &self,
        media: &NewWhatsappWebMedia,
    ) -> Result<WhatsappWebObservedMedia, WhatsappWebError> {
        media.validate()?;
        let context = self.fixture_ingest_context(&media.account_id).await?;
        let provider_record_id = media.provider_record_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &media.account_id,
            WHATSAPP_WEB_MEDIA_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &media.account_id,
            WHATSAPP_WEB_MEDIA_RECORD_KIND,
            &provider_record_id,
            media.source_fingerprint(),
            &media.import_batch_id,
            json!({
                "provider_chat_id": media.provider_chat_id,
                "provider_message_id": media.provider_message_id,
                "provider_attachment_id": media.provider_attachment_id,
                "filename": media.filename,
                "content_type": media.content_type,
                "size_bytes": media.size_bytes,
                "sha256": media.sha256,
                "storage_kind": media.storage_kind,
                "storage_path": media.storage_path,
            }),
        )
        .occurred_at(media.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": media.account_id,
            "provider_chat_id": media.provider_chat_id,
        }));
        self.update_session_last_sync(&media.account_id, media.observed_at)
            .await?;

        Ok(WhatsappWebObservedMedia { raw })
    }

    pub async fn ingest_fixture_status(
        &self,
        status: &NewWhatsappWebStatus,
    ) -> Result<WhatsappWebObservedStatus, WhatsappWebError> {
        status.validate()?;
        let context = self.fixture_ingest_context(&status.account_id).await?;
        let raw_record_id = whatsapp_web_raw_record_id(
            &status.account_id,
            WHATSAPP_WEB_STATUS_RECORD_KIND,
            &status.provider_status_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &status.account_id,
            WHATSAPP_WEB_STATUS_RECORD_KIND,
            &status.provider_status_id,
            status.source_fingerprint(),
            &status.import_batch_id,
            json!({
                "provider_status_id": status.provider_status_id,
                "sender_id": status.sender_id,
                "sender_display_name": status.sender_display_name,
                "sender_identity_kind": status.sender_identity_kind,
                "sender_address": status.sender_address,
                "sender_push_name": status.sender_push_name,
                "sender_business_profile": status.sender_business_profile,
                "sender_profile_photo_ref": status.sender_profile_photo_ref,
                "text": status.text,
                "delivery_state": "received",
            }),
        )
        .occurred_at(status.occurred_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": status.account_id,
            "provider_status_id": status.provider_status_id,
        }));
        self.update_session_last_sync(&status.account_id, status.occurred_at)
            .await?;

        Ok(WhatsappWebObservedStatus { raw })
    }

    pub async fn ingest_fixture_status_view(
        &self,
        status_view: &NewWhatsappWebStatusView,
    ) -> Result<WhatsappWebObservedStatusView, WhatsappWebError> {
        status_view.validate()?;
        let context = self.fixture_ingest_context(&status_view.account_id).await?;
        let provider_record_id = status_view.provider_record_id();
        let raw_record_id = whatsapp_web_raw_record_id(
            &status_view.account_id,
            WHATSAPP_WEB_STATUS_VIEW_RECORD_KIND,
            &provider_record_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &status_view.account_id,
            WHATSAPP_WEB_STATUS_VIEW_RECORD_KIND,
            &provider_record_id,
            status_view.source_fingerprint(),
            &status_view.import_batch_id,
            json!({
                "provider_status_id": status_view.provider_status_id,
                "viewer_id": status_view.viewer_id,
                "viewer_display_name": status_view.viewer_display_name,
            }),
        )
        .occurred_at(status_view.observed_at)
        .provenance(json!({
            "provider": "whatsapp_web",
            "provider_kind": context.provider_kind,
            "runtime": context.runtime,
            "account_id": status_view.account_id,
            "provider_status_id": status_view.provider_status_id,
        }));
        self.update_session_last_sync(&status_view.account_id, status_view.observed_at)
            .await?;

        Ok(WhatsappWebObservedStatusView { raw })
    }

    pub async fn ingest_fixture_status_delete(
        &self,
        status_delete: &NewWhatsappWebStatusDelete,
    ) -> Result<WhatsappWebObserve
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/whatsapp/client/store/intelligence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/whatsapp/client/store/intelligence.rs`
- Size bytes / Размер в байтах: `335`
- Included characters / Включено символов: `335`
- Truncated / Обрезано: `no`

```rust
use super::WhatsappWebStore;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;

impl WhatsappWebStore {
    pub(in crate::integrations::whatsapp::client::store) async fn refresh_message_intelligence_candidates(
        &self,
        _message_id: &str,
    ) -> Result<(), WhatsappWebError> {
        Ok(())
    }
}
```

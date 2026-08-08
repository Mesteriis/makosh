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

- Chunk ID / ID чанка: `060-source-backend-part-040`
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

### `backend/src/integrations/omniroute/client/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client/models.rs`
- Size bytes / Размер в байтах: `242`
- Included characters / Включено символов: `242`
- Truncated / Обрезано: `no`

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct OmniRouteChatResult {
    pub model: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OmniRouteEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
}
```

### `backend/src/integrations/omniroute/client/transport.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client/transport.rs`
- Size bytes / Размер в байтах: `1561`
- Included characters / Включено символов: `1561`
- Truncated / Обрезано: `no`

```rust
use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;

use super::{OmniRouteClient, OmniRouteError};

impl OmniRouteClient {
    fn endpoint(&self, path: &str) -> Result<Url, OmniRouteError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| OmniRouteError::InvalidConfig(error.to_string()))
    }

    pub(super) async fn get_json<T>(&self, path: &str) -> Result<T, OmniRouteError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .get(self.endpoint(path)?)
            .bearer_auth(self.api_key.expose_for_runtime())
            .send()
            .await?;
        decode_response(response).await
    }

    pub(super) async fn post_json<T>(&self, path: &str, body: &Value) -> Result<T, OmniRouteError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .post(self.endpoint(path)?)
            .bearer_auth(self.api_key.expose_for_runtime())
            .json(body)
            .send()
            .await?;
        decode_response(response).await
    }
}

async fn decode_response<T>(response: reqwest::Response) -> Result<T, OmniRouteError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status();
    if !status.is_success() {
        return Err(OmniRouteError::Endpoint {
            status: status.as_u16(),
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|error| OmniRouteError::Protocol(error.to_string()))
}
```

### `backend/src/integrations/omniroute/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/mod.rs`
- Size bytes / Размер в байтах: `16`
- Included characters / Включено символов: `16`
- Truncated / Обрезано: `no`

```rust
pub mod client;
```

### `backend/src/integrations/telegram/client/accounts.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/accounts.rs`
- Size bytes / Размер в байтах: `97`
- Included characters / Включено символов: `97`
- Truncated / Обрезано: `no`

```rust
mod credential_bindings;
mod fixture_setup;
mod lifecycle;
mod live_credentials;
mod live_setup;
```

### `backend/src/integrations/telegram/client/accounts/credential_bindings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/accounts/credential_bindings.rs`
- Size bytes / Размер в байтах: `2098`
- Included characters / Включено символов: `2098`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::platform::communications::NewProviderAccountSecretBinding;
use crate::platform::secrets::{NewSecretReference, SecretReferenceStore};

use super::super::errors::TelegramError;
use super::super::identifiers::telegram_secret_ref;
use super::super::models::TelegramCredentialBinding;
use super::super::store::TelegramStore;
use super::super::vault::{TelegramCredentialWrite, TelegramSecretVault};

impl TelegramStore {
    pub(in crate::integrations::telegram::client::accounts) async fn store_account_credential(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        credential: TelegramCredentialWrite<'_>,
    ) -> Result<TelegramCredentialBinding, TelegramError> {
        let secret_ref = telegram_secret_ref(credential.account_id, credential.secret_purpose);
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    credential.secret_kind,
                    vault.store_kind(),
                    format!("{} for {}", credential.label, credential.account_id),
                )
                .metadata(json!({
                    "provider": credential.provider_kind.as_str(),
                    "account_id": credential.account_id,
                    "secret_purpose": credential.secret_purpose.as_str()
                })),
            )
            .await?;
        vault.store_secret(&secret_ref, &credential).await?;
        self.provider_secret_binding_store()
            .bind(&NewProviderAccountSecretBinding::new(
                credential.account_id,
                credential.secret_purpose,
                &secret_ref,
            ))
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        Ok(TelegramCredentialBinding {
            secret_purpose: credential.secret_purpose.as_str().to_owned(),
            secret_ref,
            secret_kind: credential.secret_kind,
            store_kind: vault.store_kind(),
        })
    }
}
```

### `backend/src/integrations/telegram/client/accounts/fixture_setup.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/accounts/fixture_setup.rs`
- Size bytes / Размер в байтах: `1658`
- Included characters / Включено символов: `1658`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use crate::platform::communications::NewProviderAccount;

use super::super::errors::TelegramError;
use super::super::models::{TelegramAccountSetupRequest, TelegramAccountSetupResponse};
use super::super::store::TelegramStore;

impl TelegramStore {
    pub async fn setup_fixture_account(
        &self,
        request: &TelegramAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        request.validate()?;
        let provider_kind = request.provider_kind;
        if !provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(
                "provider_kind must be telegram_user or telegram_bot".to_owned(),
            ));
        }

        let account = NewProviderAccount::new(
            &request.account_id,
            provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "tdlib_data_path": request.tdlib_data_path,
            "transcription_enabled": request.transcription_enabled,
        }));
        let stored_account = self
            .provider_account_store()
            .upsert(&account)
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        Ok(TelegramAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            transcription_enabled: request.transcription_enabled,
            credential_bindings: vec![],
        })
    }
}
```

### `backend/src/integrations/telegram/client/accounts/lifecycle.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/accounts/lifecycle.rs`
- Size bytes / Размер в байтах: `3688`
- Included characters / Включено символов: `3688`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;

use crate::platform::observations::ObservationOriginKind;

use super::super::errors::TelegramError;
use super::super::identifiers::{
    telegram_account_from_provider_account, telegram_account_lifecycle_state,
};
use super::super::models::TelegramAccount;
use super::super::store::TelegramStore;
use super::super::validation::validate_object;
use super::super::{TELEGRAM_ACCOUNT_LOGGED_OUT, TELEGRAM_ACCOUNT_REMOVED};

impl TelegramStore {
    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<Vec<TelegramAccount>, TelegramError> {
        let accounts = self
            .provider_account_store()
            .list()
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        Ok(accounts
            .into_iter()
            .filter(|account| account.provider_kind.is_telegram())
            .map(telegram_account_from_provider_account)
            .filter(|account| {
                include_removed || account.lifecycle_state != TELEGRAM_ACCOUNT_REMOVED
            })
            .collect())
    }

    pub async fn logout_account(&self, account_id: &str) -> Result<TelegramAccount, TelegramError> {
        self.update_account_lifecycle(account_id, TELEGRAM_ACCOUNT_LOGGED_OUT)
            .await
    }

    pub async fn remove_account(&self, account_id: &str) -> Result<TelegramAccount, TelegramError> {
        self.update_account_lifecycle(account_id, TELEGRAM_ACCOUNT_REMOVED)
            .await
    }

    async fn update_account_lifecycle(
        &self,
        account_id: &str,
        lifecycle_state: &'static str,
    ) -> Result<TelegramAccount, TelegramError> {
        let account = self.telegram_provider_account(account_id).await?;
        let current_state = telegram_account_lifecycle_state(&account);
        if current_state == TELEGRAM_ACCOUNT_REMOVED && lifecycle_state != TELEGRAM_ACCOUNT_REMOVED
        {
            return Err(TelegramError::InvalidRequest(format!(
                "Telegram account `{}` is removed",
                account.account_id
            )));
        }

        let mut config = account.config.clone();
        validate_object("config", &config)?;
        let Some(config_object) = config.as_object_mut() else {
            return Err(TelegramError::InvalidRequest(
                "config must be a JSON object".to_owned(),
            ));
        };
        let now = Utc::now();
        config_object.insert("lifecycle_state".to_owned(), json!(lifecycle_state));
        config_object.insert("lifecycle_updated_at".to_owned(), json!(now));
        match lifecycle_state {
            TELEGRAM_ACCOUNT_LOGGED_OUT => {
                config_object.insert("logged_out_at".to_owned(), json!(now));
            }
            TELEGRAM_ACCOUNT_REMOVED => {
                config_object.insert("removed_at".to_owned(), json!(now));
            }
            _ => {}
        }

        let updated = self
            .provider_account_store()
            .update_config_with_origin(
                &account.account_id,
                &config,
                ObservationOriginKind::LocalRuntime,
                "telegram.accounts.lifecycle.update",
                lifecycle_state,
            )
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{}` is not configured",
                    account.account_id
                ))
            })?;

        Ok(telegram_account_from_provider_account(updated))
    }
}
```

### `backend/src/integrations/telegram/client/accounts/live_credentials.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/accounts/live_credentials.rs`
- Size bytes / Размер в байтах: `4231`
- Included characters / Включено символов: `4231`
- Truncated / Обрезано: `no`

```rust
use crate::platform::communications::{CommunicationProviderKind, ProviderAccountSecretPurpose};
use crate::platform::secrets::{SecretKind, SecretReferenceStore};

use super::super::errors::TelegramError;
use super::super::models::{
    TelegramAccountSetupResponse, TelegramCredentialBinding, TelegramLiveAccountSetupRequest,
};
use super::super::store::TelegramStore;
use super::super::validation::required_optional_value;
use super::super::vault::{TelegramCredentialWrite, TelegramSecretVault};

impl TelegramStore {
    pub(in crate::integrations::telegram::client::accounts) async fn store_live_account_credentials(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<Vec<TelegramCredentialBinding>, TelegramError> {
        let mut credential_bindings = Vec::new();
        match request.provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if !request.is_qr_authorized_user_account() {
                    credential_bindings.push(
                        self.store_account_credential(
                            secret_store,
                            vault,
                            TelegramCredentialWrite {
                                account_id: &request.account_id,
                                provider_kind: request.provider_kind,
                                secret_purpose: ProviderAccountSecretPurpose::TelegramApiHash,
                                secret_kind: SecretKind::ApiToken,
                                label: "Telegram API hash",
                                value: required_optional_value(
                                    "api_hash",
                                    request.api_hash.as_deref(),
                                )?,
                            },
                        )
                        .await?,
                    );
                }
                if let Some(binding) = self.store_session_key(secret_store, vault, request).await? {
                    credential_bindings.push(binding);
                }
            }
            CommunicationProviderKind::TelegramBot => {
                credential_bindings.push(
                    self.store_account_credential(
                        secret_store,
                        vault,
                        TelegramCredentialWrite {
                            account_id: &request.account_id,
                            provider_kind: request.provider_kind,
                            secret_purpose: ProviderAccountSecretPurpose::TelegramBotToken,
                            secret_kind: SecretKind::ApiToken,
                            label: "Telegram bot token",
                            value: required_optional_value(
                                "bot_token",
                                request.bot_token.as_deref(),
                            )?,
                        },
                    )
                    .await?,
                );
            }
            _ => unreachable!("validated provider kind must be Telegram"),
        }

        Ok(credential_bindings)
    }

    async fn store_session_key(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<Option<TelegramCredentialBinding>, TelegramError> {
        let Some(session_encryption_key) = request
            .session_encryption_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        self.store_account_credential(
            secret_store,
            vault,
            TelegramCredentialWrite {
                account_id: &request.account_id,
                provider_kind: request.provider_kind,
                secret_purpose: ProviderAccountSecretPurpose::TelegramSessionKey,
                secret_kind: SecretKind::Other,
                label: "Telegram session encryption key",
                value: session_encryption_key.to_owned(),
            },
        )
        .await
        .map(Some)
    }
}
```

### `backend/src/integrations/telegram/client/accounts/live_setup.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/accounts/live_setup.rs`
- Size bytes / Размер в байтах: `2893`
- Included characters / Включено символов: `2893`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use crate::platform::communications::NewProviderAccount;
use crate::platform::secrets::SecretReferenceStore;

use super::super::errors::TelegramError;
use super::super::models::{TelegramAccountSetupResponse, TelegramLiveAccountSetupRequest};
use super::super::store::TelegramStore;
use super::super::vault::TelegramSecretVault;

impl TelegramStore {
    pub async fn setup_live_blocked_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        request.validate()?;
        let provider_kind = request.provider_kind;
        if !provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(
                "provider_kind must be telegram_user or telegram_bot".to_owned(),
            ));
        }

        let runtime = live_runtime(request);
        let stored_account = self
            .provider_account_store()
            .upsert(
                &NewProviderAccount::new(
                    &request.account_id,
                    provider_kind,
                    &request.display_name,
                    &request.external_account_id,
                )
                .config(live_account_config(request, runtime)),
            )
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        let credential_bindings = self
            .store_live_account_credentials(secret_store, vault, request)
            .await?;

        Ok(TelegramAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: runtime.to_owned(),
            transcription_enabled: request.transcription_enabled,
            credential_bindings,
        })
    }
}

fn live_runtime(request: &TelegramLiveAccountSetupRequest) -> &'static str {
    if request.is_qr_authorized_user_account() {
        "tdlib_qr_authorized"
    } else {
        "live_blocked"
    }
}

fn live_account_config(request: &TelegramLiveAccountSetupRequest, runtime: &str) -> Value {
    let mut config = json!({
        "runtime": runtime,
        "transcription_enabled": request.transcription_enabled,
    });
    if let Some(object) = config.as_object_mut() {
        if let Some(tdlib_data_path) = request
            .tdlib_data_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            object.insert("tdlib_data_path".to_owned(), json!(tdlib_data_path));
        }
        if !request.is_qr_authorized_user_account()
            && let Some(api_id) = request.api_id
        {
            object.insert("api_id".to_owned(), json!(api_id));
        }
    }

    config
}
```

### `backend/src/integrations/telegram/client/chat_metadata.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/chat_metadata.rs`
- Size bytes / Размер в байтах: `17971`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use serde_json::{Value, json};

use crate::integrations::telegram::tdjson::TelegramTdlibChatSnapshot;

pub(super) fn tdlib_chat_projection_metadata(
    snapshot: &TelegramTdlibChatSnapshot,
    raw_record_id: &str,
    owner_provider_user_id: &str,
) -> Value {
    let mut metadata = json!({
        "runtime": "tdlib",
        "raw_record_id": raw_record_id,
    });

    if let Some(permissions) = tdlib_chat_permissions_metadata(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        metadata_map.insert("tdlib_permissions".to_owned(), permissions);
    }
    if let Some(marked) = snapshot
        .raw
        .get("is_marked_as_unread")
        .and_then(Value::as_bool)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        metadata_map.insert("is_marked_as_unread".to_owned(), Value::Bool(marked));
    }
    if let Some(settings) = tdlib_notification_settings_metadata(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        let is_muted = settings
            .get("use_default_mute_for")
            .and_then(Value::as_bool)
            .zip(settings.get("mute_for").and_then(Value::as_i64))
            .is_some_and(|(use_default, mute_for)| !use_default && mute_for > 0);
        metadata_map.insert("tdlib_notification_settings".to_owned(), settings);
        metadata_map.insert("is_muted".to_owned(), Value::Bool(is_muted));
    }
    if let Some(positions) = tdlib_chat_positions_metadata(&snapshot.raw)
        && let Some(metadata_map) = metadata.as_object_mut()
    {
        let is_archived = positions
            .get("archive")
            .and_then(Value::as_object)
            .and_then(|archive| archive.get("order"))
            .and_then(Value::as_i64)
            .is_some_and(|order| order > 0);
        let is_pinned = ["main", "archive"]
            .into_iter()
            .filter_map(|key| positions.get(key))
            .filter_map(Value::as_object)
            .any(|value| {
                value
                    .get("is_pinned")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            });
        metadata_map.insert("tdlib_chat_positions".to_owned(), positions);
        metadata_map.insert("is_archived".to_owned(), Value::Bool(is_archived));
        metadata_map.insert("is_pinned".to_owned(), Value::Bool(is_pinned));
    }

    let Some(chat_type) = snapshot.raw.get("type").and_then(Value::as_object) else {
        return metadata;
    };
    let tdlib_chat_type = chat_type
        .get("@type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if let Some(private_user_id) = tdlib_private_user_id(chat_type) {
        project_private_chat_metadata(
            &mut metadata,
            tdlib_chat_type,
            &private_user_id,
            owner_provider_user_id,
        );
    }
    if let Some(basic_group_id) = tdlib_basic_group_id(chat_type) {
        project_basic_group_metadata(&mut metadata, tdlib_chat_type, basic_group_id);
    }
    if tdlib_chat_type != "chatTypeSupergroup" {
        return metadata;
    }

    project_supergroup_metadata(&mut metadata, snapshot, chat_type, tdlib_chat_type);
    metadata
}

fn project_private_chat_metadata(
    metadata: &mut Value,
    tdlib_chat_type: &str,
    private_user_id: &str,
    owner_provider_user_id: &str,
) {
    let Some(metadata_map) = metadata.as_object_mut() else {
        return;
    };
    metadata_map.insert(
        "tdlib_private_user_id".to_owned(),
        Value::String(private_user_id.to_owned()),
    );

    if tdlib_chat_type != "chatTypePrivate" {
        return;
    }
    let owner_user_id = normalized_telegram_user_id(owner_provider_user_id);
    if owner_user_id.as_deref() != Some(private_user_id) {
        return;
    }

    metadata_map.insert(
        "tdlib_chat_type".to_owned(),
        Value::String(tdlib_chat_type.to_owned()),
    );
    metadata_map.insert("is_saved_messages".to_owned(), Value::Bool(true));
    metadata_map.insert(
        "saved_messages_source".to_owned(),
        Value::String("tdlib_private_self_chat".to_owned()),
    );
}

fn project_supergroup_metadata(
    metadata: &mut Value,
    snapshot: &TelegramTdlibChatSnapshot,
    chat_type: &serde_json::Map<String, Value>,
    tdlib_chat_type: &str,
) {
    let Some(metadata_map) = metadata.as_object_mut() else {
        return;
    };
    metadata_map.insert(
        "tdlib_chat_type".to_owned(),
        Value::String(tdlib_chat_type.to_owned()),
    );
    metadata_map.insert("is_supergroup".to_owned(), Value::Bool(true));
    metadata_map.insert(
        "is_channel_supergroup".to_owned(),
        Value::Bool(
            chat_type
                .get("is_channel")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ),
    );
    metadata_map.insert(
        "is_forum".to_owned(),
        Value::Bool(
            chat_type
                .get("is_forum")
                .or_else(|| snapshot.raw.get("is_forum"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ),
    );
    if let Some(supergroup_id) = chat_type.get("supergroup_id").and_then(Value::as_i64) {
        metadata_map.insert(
            "tdlib_supergroup_id".to_owned(),
            Value::Number(serde_json::Number::from(supergroup_id)),
        );
    }
}

fn project_basic_group_metadata(metadata: &mut Value, tdlib_chat_type: &str, basic_group_id: i64) {
    let Some(metadata_map) = metadata.as_object_mut() else {
        return;
    };
    metadata_map.insert(
        "tdlib_chat_type".to_owned(),
        Value::String(tdlib_chat_type.to_owned()),
    );
    metadata_map.insert(
        "tdlib_basic_group_id".to_owned(),
        Value::Number(serde_json::Number::from(basic_group_id)),
    );
    metadata_map.insert("is_basic_group".to_owned(), Value::Bool(true));
}

fn tdlib_private_user_id(chat_type: &serde_json::Map<String, Value>) -> Option<String> {
    chat_type
        .get("user_id")
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
}

fn tdlib_basic_group_id(chat_type: &serde_json::Map<String, Value>) -> Option<i64> {
    (chat_type.get("@type").and_then(Value::as_str) == Some("chatTypeBasicGroup"))
        .then(|| chat_type.get("basic_group_id").and_then(Value::as_i64))
        .flatten()
}

fn normalized_telegram_user_id(external_account_id: &str) -> Option<String> {
    let value = external_account_id.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.strip_prefix("telegram:").unwrap_or(value).to_owned())
}

fn tdlib_chat_permissions_metadata(raw: &Value) -> Option<Value> {
    let permissions = raw.get("permissions").and_then(Value::as_object)?;
    let mut projected = serde_json::Map::new();

    for key in [
        "can_send_messages",
        "can_send_basic_messages",
        "can_send_audios",
        "can_send_documents",
        "can_send_photos",
        "can_send_videos",
        "can_send_video_notes",
        "can_send_voice_notes",
        "can_send_polls",
        "can_send_other_messages",
        "can_add_web_page_previews",
        "can_change_info",
        "can_invite_users",
        "can_pin_messages",
        "can_manage_topics",
    ] {
        if let Some(value) = permissions.get(key).and_then(Value::as_bool) {
            projected.insert(key.to_owned(), Value::Bool(value));
        }
    }

    if projected.is_empty() {
        None
    } else {
        Some(Value::Object(projected))
    }
}

fn tdlib_notification_settings_metadata(raw: &Value) -> Option<Value> {
    let settings = raw
        .get("notification_settings")
        .and_then(Value::as_object)?;
    let use_default_mute_for = settings.get("use_default_mute_for")?.as_bool()?;
    let mute_for = settings.get("mute_for")?.as_i64()?;
    Some(json!({
        "use_default_mute_for": use_default_mute_for,
        "mute_for": mute_for
    }))
}

fn tdlib_chat_positions_metadata(raw: &Value) -> Option<Value> {
    let positions = raw.get("positions").and_then(Value::as_array)?;
    let mut projected = serde_json::Map::new();
    let mut folder_ids = Vec::new();

    for position in positions {
        let list = position.get("list")?;
        let order = position.get("order").and_then(Value::as_i64).unwrap_or(0);
        let is_pinned = position
            .get("is_pinned")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        match list.get("@type").and_then(Value::as_str) {
            Some("chatListMain") => {
                projected.insert(
                    "main".to_owned(),
                    json!({"order": order, "is_pinned": is_pinned}),
                );
            }
            Some("chatListArchive") => {
                projected.insert(
                    "archive".to_owned(),
                    json!({"order": order, "is_pinned": is_pinned}),
                );
            }
            Some("chatListFolder") => {
                if let Some(folder_id) = list.get("chat_folder_id").and_then(Value::as_i64) {
                    folder_ids.push(Value::Number(folder_id.into()));
                }
            }
            _ => {}
        }
    }

    if !folder_ids.is_empty() {
        projected.insert("folder_ids".to_owned(), Value::Array(folder_ids));
    }

    (!projected.is_empty()).then_some(Value::Object(projected))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::integrations::telegram::client::TelegramChatKind;

    #[test]
    fn tdlib_chat_projection_metadata_preserves_supergroup_identity() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Supergroup".to_owned(),
            username: Some("release_team".to_owned()),
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {
                    "@type": "chatTypeSupergroup",
                    "supergroup_id": 555,
                    "is_channel": false,
                    "is_forum": true
                },
                "title": "Release Supergroup"
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["runtime"], "tdlib");
        assert_eq!(metadata["raw_record_id"], "raw-telegram-chat-123");
        assert_eq!(metadata["tdlib_chat_type"], "chatTypeSupergroup");
        assert_eq!(metadata["tdlib_supergroup_id"], 555);
        assert_eq!(metadata["is_supergroup"], true);
        assert_eq!(metadata["is_channel_supergroup"], false);
        assert_eq!(metadata["is_forum"], true);
    }

    #[test]
    fn tdlib_chat_projection_metadata_preserves_permissions() {
        let snapshot = TelegramTdlibChatSnapshot {
            provider_chat_id: "123456789".to_owned(),
            chat_kind: TelegramChatKind::Group,
            title: "Release Group".to_owned(),
            username: None,
            last_message_at: None,
            raw: json!({
                "@type": "chat",
                "id": 123456789,
                "type": {"@type": "chatTypeBasicGroup"},
                "permissions": {
                    "@type": "chatPermissions",
                    "can_send_basic_messages": true,
                    "can_send_polls": false,
                    "can_invite_users": true,
                    "can_pin_messages": false,
                    "ignored_non_boolean": "yes"
                }
            }),
        };

        let metadata = tdlib_chat_projection_metadata(&snapshot, "raw-telegram-chat-123", "42");

        assert_eq!(metadata["runtime"], "tdlib");
        assert_eq!(
            metadata["tdlib_permissions"]["can_send_basic_messages"],
            true
        );
        assert_eq!(metadata["tdlib_permissions"]["can
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/chat_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/chat_reconciliation.rs`
- Size bytes / Размер в байтах: `5934`
- Included characters / Включено символов: `5934`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::errors::TelegramError;
use super::lifecycle::{mark_command_mismatch, mark_command_reconciled};
use super::models::messages::TelegramProviderWriteCommand;
use super::rows::row_to_telegram_provider_write_command;

const PROVIDER_RECONCILIATION_CLOCK_SKEW: Duration = Duration::seconds(5);

#[allow(clippy::too_many_arguments)]
pub(super) async fn reconcile_dialog_boolean_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    observed_state_key: &str,
    expected_state_key: &str,
    observed_mismatch_key: &str,
    observed_state: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
    mismatch_error: &str,
    expected_state_for_command_kind: fn(&str) -> Option<bool>,
    extra_provider_state_fields: &[(&str, serde_json::Value)],
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND happened_at <= $3
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(observed_at + PROVIDER_RECONCILIATION_CLOCK_SKEW)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let Some(expected_state) = expected_state_for_command_kind(&command.command_kind) else {
            continue;
        };

        if expected_state != observed_state {
            let provider_state = dialog_boolean_reconciliation_payload(
                provider_chat_id,
                observed_via,
                expected_state_key,
                expected_state,
                observed_mismatch_key,
                observed_state,
                extra_provider_state_fields,
            );
            let result_payload = dialog_boolean_reconciliation_payload(
                provider_chat_id,
                observed_via,
                expected_state_key,
                expected_state,
                observed_mismatch_key,
                observed_state,
                &[
                    ("provider_observed_at", json!(observed_at)),
                    ("mismatch", json!(true)),
                ],
            );
            reconciled.push(
                mark_command_mismatch(
                    pool,
                    &command.command_id,
                    observed_at,
                    provider_state,
                    result_payload,
                    mismatch_error,
                )
                .await?,
            );
            continue;
        }

        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                dialog_boolean_reconciliation_payload(
                    provider_chat_id,
                    observed_via,
                    observed_state_key,
                    observed_state,
                    observed_state_key,
                    observed_state,
                    extra_provider_state_fields,
                ),
                dialog_boolean_reconciliation_payload(
                    provider_chat_id,
                    observed_via,
                    observed_state_key,
                    observed_state,
                    observed_state_key,
                    observed_state,
                    &[("provider_observed_at", json!(observed_at))],
                ),
            )
            .await?,
        );
    }
    Ok(reconciled)
}

pub(super) fn expected_archive_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "archive" => Some(true),
        "unarchive" => Some(false),
        _ => None,
    }
}

pub(super) fn expected_marked_as_unread_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "mark_unread" => Some(true),
        _ => None,
    }
}

pub(super) fn expected_mute_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "mute" => Some(true),
        "unmute" => Some(false),
        _ => None,
    }
}

pub(super) fn expected_pin_state_for_command_kind(command_kind: &str) -> Option<bool> {
    match command_kind {
        "pin" => Some(true),
        "unpin" => Some(false),
        _ => None,
    }
}

fn dialog_boolean_reconciliation_payload(
    provider_chat_id: &str,
    observed_via: &str,
    primary_key: &str,
    primary_value: bool,
    secondary_key: &str,
    secondary_value: bool,
    extra_fields: &[(&str, serde_json::Value)],
) -> serde_json::Value {
    let mut payload = serde_json::Map::from_iter([
        (
            "provider_chat_id".to_owned(),
            serde_json::Value::String(provider_chat_id.to_owned()),
        ),
        (
            "source".to_owned(),
            serde_json::Value::String(observed_via.to_owned()),
        ),
        (
            "observed_via".to_owned(),
            serde_json::Value::String(observed_via.to_owned()),
        ),
        (
            primary_key.to_owned(),
            serde_json::Value::Bool(primary_value),
        ),
    ]);
    if secondary_key != primary_key {
        payload.insert(
            secondary_key.to_owned(),
            serde_json::Value::Bool(secondary_value),
        );
    }
    for (key, value) in extra_fields {
        payload.insert((*key).to_owned(), value.clone());
    }
    serde_json::Value::Object(payload)
}
```

### `backend/src/integrations/telegram/client/chat_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/chat_state.rs`
- Size bytes / Размер в байтах: `17731`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::chat_reconciliation::{
    expected_archive_state_for_command_kind, expected_marked_as_unread_state_for_command_kind,
    expected_mute_state_for_command_kind, expected_pin_state_for_command_kind,
    reconcile_dialog_boolean_commands_from_provider_state,
};
use super::errors::TelegramError;
use super::lifecycle::mark_command_reconciled;
use super::models::messages::TelegramProviderWriteCommand;
use super::rows::row_to_telegram_provider_write_command;

const PROVIDER_RECONCILIATION_CLOCK_SKEW: Duration = Duration::seconds(5);
use super::store::TelegramStore;

const DIALOG_PIN_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different dialog pin state than requested";
const DIALOG_ARCHIVE_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different archive state than requested";
const DIALOG_MUTE_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different mute state than requested";
const DIALOG_MARK_UNREAD_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different unread state than requested";

impl TelegramStore {
    pub async fn apply_provider_marked_as_unread(
        &self,
        telegram_chat_id: &str,
        is_marked_as_unread: bool,
        source_event: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(
            "is_marked_as_unread".to_owned(),
            serde_json::Value::Bool(is_marked_as_unread),
        );
        metadata.insert(
            "marked_as_unread_source".to_owned(),
            serde_json::Value::String(source_event.to_owned()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn apply_provider_notification_settings(
        &self,
        telegram_chat_id: &str,
        use_default_mute_for: bool,
        mute_for: i64,
        source_event: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        let is_muted = !use_default_mute_for && mute_for > 0;
        metadata.insert("is_muted".to_owned(), serde_json::Value::Bool(is_muted));
        metadata.insert(
            "tdlib_notification_settings".to_owned(),
            json!({
                "use_default_mute_for": use_default_mute_for,
                "mute_for": mute_for.max(0),
            }),
        );
        metadata.insert(
            "mute_source".to_owned(),
            serde_json::Value::String(source_event.to_owned()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn apply_provider_chat_position(
        &self,
        telegram_chat_id: &str,
        position: &TelegramProviderChatPositionUpdate,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        let mut positions = metadata
            .remove("tdlib_chat_positions")
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();

        match position.list_kind.as_str() {
            "main" | "archive" => {
                if position.order > 0 {
                    positions.insert(
                        position.list_kind.clone(),
                        json!({
                            "order": position.order,
                            "is_pinned": position.is_pinned,
                        }),
                    );
                } else {
                    positions.remove(&position.list_kind);
                }
            }
            "folder" => {
                let mut folder_ids = positions
                    .get("folder_ids")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let folder_id = position.provider_folder_id.ok_or_else(|| {
                    TelegramError::InvalidRequest(
                        "folder chat position update missing provider_folder_id".to_owned(),
                    )
                })?;
                let folder_value = serde_json::Value::Number(folder_id.into());
                if position.order > 0 {
                    if !folder_ids.iter().any(|value| value == &folder_value) {
                        folder_ids.push(folder_value);
                    }
                } else {
                    folder_ids.retain(|value| value != &folder_value);
                }
                positions.insert(
                    "folder_ids".to_owned(),
                    serde_json::Value::Array(folder_ids),
                );
            }
            _ => {}
        }

        let is_archived = positions
            .get("archive")
            .and_then(serde_json::Value::as_object)
            .and_then(|archive| archive.get("order"))
            .and_then(serde_json::Value::as_i64)
            .is_some_and(|order| order > 0);
        let is_pinned = ["main", "archive"]
            .into_iter()
            .filter_map(|key| positions.get(key))
            .filter_map(serde_json::Value::as_object)
            .any(|value| {
                value
                    .get("is_pinned")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            });

        if position.list_kind == "folder" {
            let folder_ids = positions
                .get("folder_ids")
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            let primary_folder_id = folder_ids.first().and_then(serde_json::Value::as_i64);
            if folder_ids.is_empty() {
                metadata.remove("folder_labels");
                metadata.remove("folder_name");
                metadata.remove("provider_folder_id");
                metadata.remove("provider_folder_ids");
            } else {
                let existing_labels = metadata
                    .get("folder_labels")
                    .and_then(serde_json::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if existing_labels.is_empty() {
                    let fallback_labels = folder_ids
                        .iter()
                        .filter_map(serde_json::Value::as_i64)
                        .map(|folder_id| {
                            serde_json::Value::String(format!("Unknown folder {folder_id}"))
                        })
                        .collect::<Vec<_>>();
                    if let Some(primary_label) =
                        fallback_labels.first().and_then(|value| value.as_str())
                    {
                        metadata.insert(
                            "folder_name".to_owned(),
                            serde_json::Value::String(primary_label.to_owned()),
                        );
                    }
                    metadata.insert(
                        "folder_labels".to_owned(),
                        serde_json::Value::Array(fallback_labels),
                    );
                }
                metadata.insert(
                    "provider_folder_ids".to_owned(),
                    serde_json::Value::Array(folder_ids.clone()),
                );
                if let Some(primary_folder_id) = primary_folder_id {
                    metadata.insert(
                        "provider_folder_id".to_owned(),
                        serde_json::Value::Number(primary_folder_id.into()),
                    );
                }
            }
        }
        metadata.insert(
            "tdlib_chat_positions".to_owned(),
            serde_json::Value::Object(positions),
        );
        metadata.insert(
            "is_archived".to_owned(),
            serde_json::Value::Bool(is_archived),
        );
        metadata.insert("is_pinned".to_owned(), serde_json::Value::Bool(is_pinned));
        metadata.insert(
            "archive_source".to_owned(),
            serde_json::Value::String(position.source_event.clone()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelegramProviderChatPositionUpdate {
    pub list_kind: String,
    pub provider_folder_id: Option<i64>,
    pub order: i64,
    pub is_pinned: bool,
    pub source_event: String,
}

pub async fn reconcile_marked_as_unread_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    is_marked_as_unread: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_dialog_boolean_commands_from_provider_state(
        pool,
        account_id,
        provider_chat_id,
        "is_marked_as_unread",
        "expected_is_marked_as_unread",
        "observed_is_marked_as_unread",
        is_marked_as_unread,
        observed_at,
        observed_via,
        DIALOG_MARK_UNREAD_PROVIDER_MISMATCH_ERROR,
        expected_marked_as_unread_state_for_command_kind,
        &[],
    )
    .await
}

pub async fn reconcile_mark_read_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    last_read_inbox_message_id: &str,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let Some(observed_message_id) =
        telegram_provider_message_numeric_suffix(last_read_inbox_message_id)
    else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND command_kind = 'mark_read'
          AND status IN ('queued', 'retrying', 'executing')
          AND provider_message_id IS NOT NULL
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
          AND happened_at <= $3
        ORDER BY happened_at ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(observed_at + PROVIDER_RECONCILIATION_CLOCK_SKEW)
    .fetch_all(pool)
    .await
    .map_err(TelegramError::from)?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let Some(target_message_id) = telegram_provider_message_numeric_suffix(
            command.provider_message_id.as_deref().unwrap_or_default(),
        ) else {
            continue;
        };
        if target_message_id > observed_message_id {
            continue;
        }
        reconciled.push(
            super::commands::mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                json!({
                    "provider_chat_id": provider_chat_id,
                    "last_read_inbox_message_id": last_read_inbox_message_id,
                    "observed_via": observed_via,
                }),
                json!({
                    "source": observed_via,
                    "provider_chat_id": provider_chat_id,
                    "last_read_inbox_message_id": last_read_inbox_message_id,
                    "provider_observed_at": observed_at,
                }),
            )
            .await?,
        );
    }
    Ok(reconciled)
}

pub async fn reconcile_mute_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    use_default_mute_for: bool,
    mute_for: i64,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    reconcile_dialog_boolean_commands_from_provider_state(
        pool,
        account_id,
        provider_chat_id,
        "is_muted",

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/chats.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/chats.rs`
- Size bytes / Размер в байтах: `20242`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::integrations::telegram::tdjson::{
    TelegramTdlibChatFolderSnapshot, TelegramTdlibChatSnapshot,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

use super::TELEGRAM_CHAT_RECORD_KIND;
use super::chat_metadata::tdlib_chat_projection_metadata;
use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::identifiers::{stable_hash, telegram_chat_id, telegram_raw_record_id};
use super::models::{
    NewTelegramChat, TelegramChat, TelegramChatGroupFilter, TelegramChatMember, TelegramSyncState,
};
use super::rows::row_to_telegram_chat;
use super::store::TelegramStore;
use super::validation::validate_chat_list_limit;

#[path = "chats/metadata_flags.rs"]
mod metadata_flags;

async fn capture_chat_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    chat: &TelegramChat,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_CHAT",
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "telegram_chat_id": chat.telegram_chat_id,
                "account_id": chat.account_id,
                "provider_chat_id": chat.provider_chat_id,
                "chat_kind": chat.chat_kind,
                "title": chat.title,
                "username": chat.username,
                "sync_state": chat.sync_state,
                "last_message_at": chat.last_message_at,
                "metadata": chat.metadata,
                "operation": relationship_kind,
            }),
            match relationship_kind {
                "upsert" => format!("telegram-chat://{}", chat.telegram_chat_id),
                _ => format!(
                    "telegram-chat://{}/{}",
                    chat.telegram_chat_id, relationship_kind
                ),
            },
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "chat",
        chat.telegram_chat_id.clone(),
        relationship_kind,
        json!({
            "account_id": chat.account_id,
            "provider_chat_id": chat.provider_chat_id,
            "chat_kind": chat.chat_kind,
            "sync_state": chat.sync_state,
        }),
    )
    .await?;
    Ok(())
}

impl TelegramStore {
    pub async fn upsert_chat(&self, chat: &NewTelegramChat) -> Result<TelegramChat, TelegramError> {
        chat.validate()?;
        let telegram_chat_id = telegram_chat_id(&chat.account_id, &chat.provider_chat_id);
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_chats (
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
            ON CONFLICT (account_id, provider_chat_id)
            DO UPDATE SET
                chat_kind = EXCLUDED.chat_kind,
                title = EXCLUDED.title,
                username = COALESCE(EXCLUDED.username, telegram_chats.username),
                sync_state = EXCLUDED.sync_state,
                last_message_at = EXCLUDED.last_message_at,
                metadata = COALESCE(telegram_chats.metadata, '{}'::jsonb) || EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&telegram_chat_id)
        .bind(chat.account_id.trim())
        .bind(chat.provider_chat_id.trim())
        .bind(chat.chat_kind.as_str())
        .bind(chat.title.trim())
        .bind(
            chat.username
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(chat.sync_state.as_str())
        .bind(chat.last_message_at)
        .bind(&chat.metadata)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_telegram_chat(row)?;
        capture_chat_observation_in_transaction(
            &mut transaction,
            &stored,
            "upsert",
            "telegram.client.chats.upsert_chat",
            stored.updated_at,
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn list_chats(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let limit = validate_chat_list_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY COALESCE(last_message_at, updated_at) DESC, telegram_chat_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }

    async fn list_all_chats_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let rows = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE account_id = $1
            ORDER BY updated_at DESC, telegram_chat_id ASC
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }

    pub async fn list_chat_group_filters(
        &self,
        account_id: Option<&str>,
    ) -> Result<Vec<TelegramChatGroupFilter>, TelegramError> {
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query_as::<_, (String, String, String, i64, String, Option<i64>)>(
            r#"
            SELECT id, label, source, count, icon, provider_folder_id
            FROM (
                SELECT
                    'local:all'::text AS id,
                    'All'::text AS label,
                    'local'::text AS source,
                    COUNT(*)::bigint AS count,
                    'tabler:message'::text AS icon,
                    NULL::bigint AS provider_folder_id
                FROM telegram_chats
                WHERE ($1::text IS NULL OR account_id = $1)

                UNION ALL

                SELECT
                    'folder:' || folder_label AS id,
                    folder_label AS label,
                    'telegram'::text AS source,
                    COUNT(*)::bigint AS count,
                    'tabler:folder'::text AS icon,
                    MIN(provider_folder_id)::bigint AS provider_folder_id
                FROM (
                    SELECT
                        telegram_chat_id,
                        NULLIF(BTRIM(folder_labels.value), '') AS folder_label,
                        COALESCE(
                            NULLIF(BTRIM(provider_folder_ids.value), '')::bigint,
                            NULLIF(BTRIM(metadata->>'provider_folder_id'), '')::bigint
                        ) AS provider_folder_id
                    FROM telegram_chats
                    LEFT JOIN LATERAL jsonb_array_elements_text(COALESCE(metadata->'folder_labels', '[]'::jsonb))
                        WITH ORDINALITY AS folder_labels(value, folder_index) ON true
                    LEFT JOIN LATERAL jsonb_array_elements_text(COALESCE(metadata->'provider_folder_ids', '[]'::jsonb))
                        WITH ORDINALITY AS provider_folder_ids(value, folder_index)
                        ON provider_folder_ids.folder_index = folder_labels.folder_index
                    WHERE ($1::text IS NULL OR account_id = $1)

                    UNION ALL

                    SELECT
                        telegram_chat_id,
                        NULLIF(BTRIM(metadata->>'folder_name'), '') AS folder_label,
                        NULLIF(BTRIM(metadata->>'provider_folder_id'), '')::bigint AS provider_folder_id
                    FROM telegram_chats
                    WHERE ($1::text IS NULL OR account_id = $1)
                      AND jsonb_array_length(COALESCE(metadata->'folder_labels', '[]'::jsonb)) = 0
                ) folder_rows
                WHERE folder_label IS NOT NULL
                GROUP BY folder_label
            ) filters
            ORDER BY
                CASE WHEN source = 'local' THEN 0 ELSE 1 END,
                label ASC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, label, source, count, icon, provider_folder_id)| TelegramChatGroupFilter {
                    id,
                    label,
                    source,
                    count,
                    icon,
                    provider_folder_id,
                },
            )
            .collect())
    }

    pub(crate) async fn apply_provider_chat_folders(
        &self,
        account_id: &str,
        folders: &[TelegramTdlibChatFolderSnapshot],
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let folder_map = folders
            .iter()
            .map(|folder| (folder.provider_folder_id, folder))
            .collect::<std::collections::HashMap<_, _>>();
        let chats = self.list_all_chats_for_account(account_id).await?;
        let mut updated = Vec::new();

        for chat in chats {
            let Some(chat_metadata) = chat.metadata.as_object() else {
                continue;
            };
            let folder_ids = chat_metadata
                .get("tdlib_chat_positions")
                .and_then(serde_json::Value::as_object)
                .and_then(|positions| positions.get("folder_ids"))
                .and_then(serde_json::Value::as_array)
                .cloned()
                .unwrap_or_default();
            if folder_ids.is_empty() {
                continue;
            }

            let mut labels = Vec::new();
            let provider_folder_ids = folder_ids
                .into_iter()
                .filter_map(|value| value.as_i64())
                .collect::<Vec<_>>();
            let mut label_folder_ids = Vec::new();
            for folder_id in provider_folder_ids.iter().copied() {
                let Some(folder) = folder_map.get(&folder_id) else {
                    continue;

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/chats/metadata_flags.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/chats/metadata_flags.rs`
- Size bytes / Размер в байтах: `4909`
- Included characters / Включено символов: `4909`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};

use super::super::errors::TelegramError;
use super::super::models::TelegramChat;
use super::TelegramStore;

const TELEGRAM_CHANNEL_KINDS: &[&str] = &["telegram_user", "telegram_bot"];

impl TelegramStore {
    pub async fn set_chat_metadata_bool(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: bool,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(key.to_owned(), serde_json::Value::Bool(value));
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn set_chat_metadata_number(
        &self,
        telegram_chat_id: &str,
        key: &str,
        value: i64,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        metadata.insert(
            key.to_owned(),
            serde_json::Value::Number(serde_json::Number::from(value.max(0))),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn set_chat_last_read_at(
        &self,
        telegram_chat_id: &str,
        last_read_at: Option<DateTime<Utc>>,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        match last_read_at {
            Some(value) => {
                metadata.insert(
                    "last_read_at".to_owned(),
                    serde_json::Value::String(value.to_rfc3339()),
                );
            }
            None => {
                metadata.remove("last_read_at");
            }
        }
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn apply_provider_unread_counts(
        &self,
        telegram_chat_id: &str,
        unread_count: Option<i64>,
        unread_mention_count: Option<i64>,
        last_read_inbox_message_id: Option<&str>,
        source_event: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        if let Some(value) = unread_count {
            metadata.insert(
                "unread_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
            metadata.insert(
                "provider_unread_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
        }
        if let Some(value) = unread_mention_count {
            metadata.insert(
                "mention_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
            metadata.insert(
                "provider_unread_mention_count".to_owned(),
                serde_json::Value::Number(serde_json::Number::from(value.max(0))),
            );
        }
        if let Some(value) = last_read_inbox_message_id {
            metadata.insert(
                "last_read_inbox_provider_message_id".to_owned(),
                serde_json::Value::String(value.to_owned()),
            );
        }
        metadata.insert(
            "unread_count_source".to_owned(),
            serde_json::Value::String(source_event.to_owned()),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }

    pub async fn recompute_chat_unread_count(
        &self,
        telegram_chat_id: &str,
    ) -> Result<serde_json::Value, TelegramError> {
        let chat = self
            .telegram_chat_by_id(telegram_chat_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram chat `{telegram_chat_id}` was not found"
                ))
            })?;
        let mut metadata = self.chat_metadata_map(telegram_chat_id).await?;
        let last_read_at = metadata
            .get("last_read_at")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));
        let (unread_count, mention_count) = self
            .provider_channel_message_store()
            .unread_counts(
                &chat.account_id,
                &chat.provider_chat_id,
                TELEGRAM_CHANNEL_KINDS,
                last_read_at,
            )
            .await?;
        metadata.insert(
            "unread_count".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(unread_count.max(0))),
        );
        metadata.insert(
            "mention_count".to_owned(),
            serde_json::Value::Number(serde_json::Number::from(mention_count.max(0))),
        );
        self.persist_chat_metadata(telegram_chat_id, metadata).await
    }
}
```

### `backend/src/integrations/telegram/client/commands.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/commands.rs`
- Size bytes / Размер в байтах: `19643`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::postgres::Postgres;
use sqlx::{PgPool, Transaction};

use super::errors::TelegramError;
use super::evidence::link_telegram_entity_in_transaction;
use super::models::messages::TelegramProviderWriteCommand;
use super::rows::row_to_telegram_provider_write_command;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

#[path = "commands/queries.rs"]
mod queries;

pub use queries::{
    find_command_by_idempotency, list_commands, list_commands_filtered,
    list_queued_commands_for_execution,
};

pub const TELEGRAM_OUTBOX_WORKER_ID: &str = "telegram-outbox-worker";
const COMMAND_QUEUE_ACTOR: &str = "telegram.client.commands";

async fn capture_command_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command: &TelegramProviderWriteCommand,
    kind_code: &str,
    relationship_kind: &str,
    actor: &str,
    observed_at: DateTime<Utc>,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            kind_code,
            ObservationOriginKind::LocalRuntime,
            observed_at,
            json!({
                "command_id": command.command_id,
                "account_id": command.account_id,
                "command_kind": command.command_kind,
                "idempotency_key": command.idempotency_key,
                "provider_chat_id": command.provider_chat_id,
                "provider_message_id": command.provider_message_id,
                "capability_state": command.capability_state,
                "action_class": command.action_class,
                "confirmation_decision": command.confirmation_decision,
                "status": command.status,
                "retry_count": command.retry_count,
                "max_retries": command.max_retries,
                "last_error": command.last_error,
                "result_payload": command.result_payload,
                "target_ref": command.target_ref,
                "payload": command.payload,
                "audit_metadata": command.audit_metadata,
                "actor_id": command.actor_id,
                "next_attempt_at": command.next_attempt_at,
                "last_attempt_at": command.last_attempt_at,
                "locked_at": command.locked_at,
                "locked_by": command.locked_by,
                "provider_observed_at": command.provider_observed_at,
                "provider_state": command.provider_state,
                "reconciliation_status": command.reconciliation_status,
                "reconciled_at": command.reconciled_at,
                "dead_lettered_at": command.dead_lettered_at,
                "completed_at": command.completed_at,
                "operation": relationship_kind,
            }),
            match kind_code {
                "TELEGRAM_PROVIDER_WRITE_COMMAND" => {
                    format!("telegram-provider-command://{}", command.command_id)
                }
                _ => format!(
                    "telegram-provider-command://{}/status/{}",
                    command.command_id, relationship_kind
                ),
            },
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "provider_write_command",
        command.command_id.clone(),
        relationship_kind,
        json!({
            "command_kind": command.command_kind,
            "status": command.status,
            "reconciliation_status": command.reconciliation_status,
            "provider_chat_id": command.provider_chat_id,
            "provider_message_id": command.provider_message_id,
        }),
    )
    .await?;
    Ok(())
}

async fn fetch_command_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    let row = sqlx::query("SELECT * FROM telegram_provider_write_commands WHERE command_id = $1")
        .bind(command_id)
        .fetch_one(&mut **transaction)
        .await?;
    row_to_telegram_provider_write_command(row)
}

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

pub fn new_command_id() -> String {
    let now = Utc::now();
    format!(
        "tcmd_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("cmd_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_command(
    pool: &PgPool,
    command_id: &str,
    account_id: &str,
    command_kind: &str,
    idempotency_key: &str,
    provider_chat_id: &str,
    provider_message_id: Option<&str>,
    capability_state: &str,
    action_class: &str,
    confirmation_decision: &str,
    actor_id: &str,
    payload: serde_json::Value,
    target_ref: serde_json::Value,
    audit_metadata: serde_json::Value,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO telegram_provider_write_commands
            (command_id, account_id, command_kind, idempotency_key, provider_chat_id,
             provider_message_id, capability_state, action_class, confirmation_decision,
             status, retry_count, max_retries, actor_id, payload, target_ref, audit_metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'queued', 0, 3, $10, $11, $12, $13)
        "#,
    )
    .bind(command_id)
    .bind(account_id)
    .bind(command_kind)
    .bind(idempotency_key)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .bind(capability_state)
    .bind(action_class)
    .bind(confirmation_decision)
    .bind(actor_id)
    .bind(&payload)
    .bind(&target_ref)
    .bind(&audit_metadata)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND",
        "queued",
        COMMAND_QUEUE_ACTOR,
        command.happened_at,
    )
    .await?;
    transaction.commit().await?;
    Ok(command)
}

pub async fn update_command_status(
    pool: &PgPool,
    command_id: &str,
    status: &str,
    result_payload: serde_json::Value,
    last_error: Option<&str>,
    completed_at: Option<chrono::DateTime<Utc>>,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = $2, result_payload = $3, last_error = $4,
            completed_at = $5, updated_at = now()
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(status)
    .bind(&result_payload)
    .bind(last_error)
    .bind(completed_at)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "status_updated",
        COMMAND_QUEUE_ACTOR,
        command.updated_at,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn retry_command(pool: &PgPool, command_id: &str) -> Result<(), TelegramError> {
    schedule_command_retry(
        pool,
        command_id,
        Utc::now(),
        Utc::now() + Duration::seconds(30),
        "Telegram provider command retry scheduled",
    )
    .await
}

pub async fn schedule_command_retry(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    next_attempt_at: DateTime<Utc>,
    error_message: &str,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'retrying',
            next_attempt_at = $3,
            locked_at = NULL,
            locked_by = NULL,
            last_error = $4,
            reconciliation_status = 'not_observed',
            updated_at = $2
        WHERE command_id = $1
          AND status = 'executing'
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(next_attempt_at)
    .bind(error_message)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "retry_scheduled",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn dead_letter_command(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    error_message: &str,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'dead_letter',
            locked_at = NULL,
            locked_by = NULL,
            last_error = $3,
            dead_lettered_at = $2,
            updated_at = $2
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(error_message)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "dead_lettered",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn mark_command_awaiting_provider(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    result_payload: serde_json::Value,
) -> Result<(), TelegramError> {
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'executing',
            result_payload = $3,
            last_error = NULL,
            reconciliation_status = 'awaiting_provider',
            locked_at = NULL,
            locked_by = NULL,
            updated_at = $2
        WHERE command_id = $1
        "#,
    )
    .bind(command_id)
    .bind(now)
    .bind(&result_payload)
    .execute(&mut *transaction)
    .await?;
    let command = fetch_command_in_transaction(&mut transaction, command_id).await?;
    capture_command_observation_in_transaction(
        &mut transaction,
        &command,
        "TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS",
        "awaiting_provider",
        COMMAND_QUEUE_ACTOR,
        now,
    )
    .await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn mark_command_reconciled(
    pool: &PgPool,
    command_id: &str,
    now: DateTime<Utc>,
    provider_state: serde_json::Value,
    result_payload: serde_json::Value,
) -> Result<TelegramProviderWriteCommand, TelegramError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        UPDATE telegram_provider_write_commands
        SET status = 'completed',
            result_payload = $3,
            last_error = NULL,
            provider_observed_at = $2,
            provider_state = $4,
            reconciliation_status = 'observed',
            reconciled_at = $2,
            completed_at = $2,
            locked_at = NULL,
            locked_by = NULL,
            next_attempt_at = NULL,
            dead_lettered_at = NULL,
            updated_at = $2
        WHERE command_id = $1
        RETURNING *
        "#,
    )

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/integrations/telegram/client/commands/queries.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/commands/queries.rs`
- Size bytes / Размер в байтах: `2918`
- Included characters / Включено символов: `2918`
- Truncated / Обрезано: `no`

```rust
use sqlx::PgPool;

use super::super::errors::TelegramError;
use super::super::models::messages::TelegramProviderWriteCommand;
use super::super::rows::row_to_telegram_provider_write_command;

pub async fn find_command_by_idempotency(
    pool: &PgPool,
    account_id: &str,
    idempotency_key: &str,
) -> Result<Option<TelegramProviderWriteCommand>, TelegramError> {
    let row = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1 AND idempotency_key = $2
        "#,
    )
    .bind(account_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_telegram_provider_write_command).transpose()
}

pub async fn list_commands(
    pool: &PgPool,
    account_id: &str,
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    list_commands_filtered(pool, account_id, None, None, &[], limit).await
}

pub async fn list_commands_filtered(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: Option<&str>,
    provider_message_id: Option<&str>,
    command_kinds: &[String],
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND ($2::text IS NULL OR provider_chat_id = $2)
          AND ($3::text IS NULL OR provider_message_id = $3)
          AND (cardinality($4::text[]) = 0 OR command_kind = ANY($4::text[]))
        ORDER BY created_at DESC
        LIMIT $5
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .bind(command_kinds)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect()
}

pub async fn list_queued_commands_for_execution(
    pool: &PgPool,
    account_id: &str,
    limit: i64,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND status IN ('queued', 'retrying')
          AND retry_count < max_retries
          AND (next_attempt_at IS NULL OR next_attempt_at <= now())
          AND command_kind IN (
              'send_text', 'send_media', 'reply', 'forward',
              'edit', 'delete', 'react', 'unreact', 'pin', 'unpin',
              'mark_read', 'mark_unread', 'archive', 'unarchive',
              'mute', 'unmute', 'join', 'leave', 'folder_add', 'folder_remove',
              'admin_action'
          )
        ORDER BY COALESCE(next_attempt_at, created_at) ASC, created_at ASC, command_id ASC
        LIMIT $2
        "#,
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_provider_write_command)
        .collect()
}
```

### `backend/src/integrations/telegram/client/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/errors.rs`
- Size bytes / Размер в байтах: `1381`
- Included characters / Включено символов: `1381`
- Truncated / Обрезано: `no`

```rust
use crate::platform::communications::ProviderCommunicationMessagePortError;
use crate::platform::observations::ObservationStoreError;
use crate::platform::secrets::{DatabaseEncryptedVaultError, SecretReferenceError};
use crate::vault::HostVaultError;

#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("invalid Telegram request: {0}")]
    InvalidRequest(String),

    #[error("Telegram TDLib runtime is not available: {0}")]
    TdlibRuntimeUnavailable(String),

    #[error("Telegram TDLib runtime failed: {0}")]
    TdlibRuntime(String),

    #[error("Telegram QR generation failed: {0}")]
    QrGeneration(String),

    #[error("Telegram QR login setup was not found")]
    QrLoginNotFound,

    #[error("Telegram provider account store operation failed: {0}")]
    ProviderAccountStore(String),

    #[error("Telegram media storage operation failed: {0}")]
    MediaStorage(String),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    CommunicationMessagePort(#[from] ProviderCommunicationMessagePortError),

    #[error(transparent)]
    ObservationStore(#[from] ObservationStoreError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
```

### `backend/src/integrations/telegram/client/evidence.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/evidence.rs`
- Size bytes / Размер в байтах: `1227`
- Included characters / Включено символов: `1227`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sqlx::Transaction;
use sqlx::postgres::Postgres;

use crate::platform::observations::{ObservationStoreError, link_domain_entity_in_transaction};

pub(super) async fn link_telegram_entity_in_transaction(
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
        "telegram",
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}

pub(super) async fn link_communication_entity_in_transaction(
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

### `backend/src/integrations/telegram/client/identifiers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/identifiers.rs`
- Size bytes / Размер в байтах: `3593`
- Included characters / Включено символов: `3593`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::platform::communications::{ProviderAccount, ProviderAccountSecretPurpose};

use super::TELEGRAM_ACCOUNT_ACTIVE;
use super::errors::TelegramError;
use super::models::TelegramAccount;

pub(crate) fn telegram_chat_id(account_id: &str, provider_chat_id: &str) -> String {
    format!(
        "telegram_chat:v4:{}",
        stable_hash([account_id, provider_chat_id].join("\0").as_bytes())
    )
}

pub(super) fn telegram_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v4:telegram:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

pub(super) fn telegram_raw_record_id(
    account_id: &str,
    record_kind: &str,
    provider_record_id: &str,
) -> String {
    format!(
        "raw:v4:telegram:{}",
        stable_hash(
            [account_id, record_kind, provider_record_id]
                .join("\0")
                .as_bytes()
        )
    )
}

pub(crate) fn telegram_text_preview_hash(text: &str) -> String {
    format!("sha256:{}", stable_hash(text.trim().as_bytes()))
}

pub(super) fn telegram_account_runtime(account: &ProviderAccount) -> String {
    account
        .config
        .get("runtime")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_owned()
}

pub(crate) fn ensure_telegram_account_active(
    account: &ProviderAccount,
) -> Result<(), TelegramError> {
    let lifecycle_state = telegram_account_lifecycle_state(account);
    if lifecycle_state != TELEGRAM_ACCOUNT_ACTIVE {
        return Err(TelegramError::InvalidRequest(format!(
            "Telegram account `{}` is `{}` and cannot run provider operations",
            account.account_id, lifecycle_state
        )));
    }

    Ok(())
}

pub(super) fn telegram_account_from_provider_account(account: ProviderAccount) -> TelegramAccount {
    let runtime = telegram_account_runtime(&account);
    let lifecycle_state = telegram_account_lifecycle_state(&account);
    let transcription_enabled = account
        .config
        .get("transcription_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let tdlib_data_path = account
        .config
        .get("tdlib_data_path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    TelegramAccount {
        account_id: account.account_id,
        provider_kind: account.provider_kind.as_str().to_owned(),
        display_name: account.display_name,
        external_account_id: account.external_account_id,
        runtime,
        lifecycle_state,
        transcription_enabled,
        tdlib_data_path,
        created_at: account.created_at,
        updated_at: account.updated_at,
    }
}

pub(super) fn telegram_account_lifecycle_state(account: &ProviderAccount) -> String {
    account
        .config
        .get("lifecycle_state")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(TELEGRAM_ACCOUNT_ACTIVE)
        .to_owned()
}

pub(super) fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(super) fn telegram_secret_ref(
    account_id: &str,
    secret_purpose: ProviderAccountSecretPurpose,
) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        secret_purpose.as_str()
    )
}
```

### `backend/src/integrations/telegram/client/lifecycle.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/lifecycle.rs`
- Size bytes / Размер в байтах: `1296`
- Included characters / Включено символов: `1296`
- Truncated / Обрезано: `no`

```rust
mod ids;
mod message_versions;
mod operations;
mod provider_reconciliation;
mod tombstones;

pub use self::message_versions::{
    insert_message_version, latest_message_version, latest_version_number, list_message_versions,
    record_provider_edit_observation,
};
pub use self::operations::{
    record_delete, record_edit, record_pin_state, record_restore_visibility,
};
pub use self::provider_reconciliation::{
    reconcile_delete_commands_from_provider_state, reconcile_edit_commands_from_provider_state,
    reconcile_message_pin_commands_from_provider_state,
};
pub use self::tombstones::{
    insert_tombstone, is_message_visible, list_tombstones, record_provider_delete_observation,
};

pub use super::commands::{
    claim_due_commands_for_execution, dead_letter_command, find_command_by_idempotency,
    insert_command, list_commands, list_queued_commands_for_execution, manual_retry_command,
    mark_command_awaiting_provider, mark_command_mismatch, mark_command_reconciled, new_command_id,
    recover_stale_executing_commands, retry_command, schedule_command_retry, update_command_status,
};
pub use super::reactions::{add_reaction, list_reactions, reaction_summary, remove_reaction};
pub use super::references::{forward_chain, insert_forward_ref, insert_reply_ref, reply_chain};
```

### `backend/src/integrations/telegram/client/lifecycle/ids.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/lifecycle/ids.rs`
- Size bytes / Размер в байтах: `695`
- Included characters / Включено символов: `695`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use sha2::{Digest, Sha256};

fn stable_short_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_owned()
}

pub(super) fn new_version_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgver_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("ver_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}

pub(super) fn new_tombstone_id() -> String {
    let now = Utc::now();
    format!(
        "tmsgtomb_{}_{}",
        now.timestamp_millis(),
        stable_short_hash(&format!("tomb_{}", now.timestamp_nanos_opt().unwrap_or(0)))
    )
}
```

### `backend/src/integrations/telegram/client/lifecycle/message_versions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/lifecycle/message_versions.rs`
- Size bytes / Размер в байтах: `8476`
- Included characters / Включено символов: `8476`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};

use super::ids::new_version_id;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::evidence::link_telegram_entity_in_transaction;
use crate::integrations::telegram::client::models::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageVersion;
use crate::integrations::telegram::client::rows::row_to_telegram_message_version;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

async fn capture_message_version_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    version: &TelegramMessageVersion,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_MESSAGE_VERSION",
            ObservationOriginKind::LocalRuntime,
            version.created_at,
            json!({
                "version_id": version.version_id,
                "message_id": version.message_id,
                "account_id": version.account_id,
                "provider_message_id": version.provider_message_id,
                "provider_chat_id": version.provider_chat_id,
                "version_number": version.version_number,
                "body_text": version.body_text,
                "edit_timestamp": version.edit_timestamp,
                "source_event": version.source_event,
                "raw_diff_payload": version.raw_diff_payload,
                "provenance": version.provenance,
                "operation": relationship_kind,
            }),
            format!(
                "telegram-message-version://{}/{}",
                version.version_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "message_version",
        version.version_id.clone(),
        relationship_kind,
        json!({
            "message_id": version.message_id,
            "account_id": version.account_id,
            "provider_message_id": version.provider_message_id,
            "provider_chat_id": version.provider_chat_id,
            "version_number": version.version_number,
        }),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_message_version(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    version_number: i32,
    body_text: Option<&str>,
    edit_timestamp: DateTime<Utc>,
    source_event: Option<&str>,
    raw_diff: Value,
    provenance: Value,
) -> Result<TelegramMessageVersion, TelegramError> {
    let version_id = new_version_id();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_message_versions
            (version_id, message_id, account_id, provider_message_id, provider_chat_id,
             version_number, body_text, edit_timestamp, source_event,
             raw_diff_payload, provenance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(&version_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(version_number)
    .bind(body_text)
    .bind(edit_timestamp)
    .bind(source_event)
    .bind(&raw_diff)
    .bind(&provenance)
    .fetch_one(&mut *transaction)
    .await?;

    let version = row_to_telegram_message_version(row)?;
    capture_message_version_observation_in_transaction(
        &mut transaction,
        &version,
        "insert",
        "telegram.client.lifecycle.message_versions.insert_message_version",
    )
    .await?;
    transaction.commit().await?;
    Ok(version)
}

pub async fn list_message_versions(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageVersion>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_version)
        .collect()
}

pub async fn latest_message_version(
    pool: &PgPool,
    message_id: &str,
) -> Result<Option<TelegramMessageVersion>, TelegramError> {
    let row = sqlx::query(
        r#"
        SELECT *
        FROM telegram_message_versions
        WHERE message_id = $1
        ORDER BY version_number DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_telegram_message_version).transpose()
}

pub async fn latest_version_number(pool: &PgPool, message_id: &str) -> Result<i32, TelegramError> {
    let row: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(version_number), 0) as max_ver
        FROM telegram_message_versions
        WHERE message_id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(0))
}

pub async fn record_provider_edit_observation(
    pool: &PgPool,
    message: &TelegramMessage,
    body_text: &str,
    edit_timestamp: DateTime<Utc>,
    source_event: &str,
    raw_diff: Value,
    provenance: Value,
) -> Result<TelegramMessageVersion, TelegramError> {
    if let Some(existing) = latest_message_version(pool, &message.message_id).await?
        && existing.body_text.as_deref() == Some(body_text)
        && existing.source_event.as_deref() == Some(source_event)
        && existing.edit_timestamp == edit_timestamp
    {
        return Ok(existing);
    }

    let version_number = latest_version_number(pool, &message.message_id).await? + 1;
    insert_message_version(
        pool,
        &message.message_id,
        &message.account_id,
        &message.provider_message_id,
        message.provider_chat_id.as_deref().unwrap_or_default(),
        version_number,
        Some(body_text),
        edit_timestamp,
        Some(source_event),
        raw_diff,
        provenance,
    )
    .await
}

pub(crate) fn local_edit_diff(previous_text: Option<&str>, new_text: &str) -> Value {
    let previous_text_length = previous_text.map(text_len);
    let new_text_length = text_len(new_text);
    let text_length_delta =
        previous_text_length.map(|previous| new_text_length as i64 - previous as i64);

    json!({
        "previous_text_length": previous_text_length,
        "new_text_length": new_text_length,
        "text_length_delta": text_length_delta,
        "changed": previous_text != Some(new_text),
        "previous_preview": previous_text.map(text_preview),
        "new_preview": text_preview(new_text),
        "previous_sha256": previous_text.map(sha256_hex),
        "new_sha256": sha256_hex(new_text),
    })
}

fn text_len(text: &str) -> usize {
    text.chars().count()
}

fn text_preview(text: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 160;
    text.chars().take(MAX_PREVIEW_CHARS).collect()
}

fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::local_edit_diff;
    use serde_json::json;

    #[test]
    fn local_edit_diff_records_previous_and_new_text_metadata() {
        let diff = local_edit_diff(Some("before body"), "after body!");

        assert_eq!(diff["previous_text_length"], json!(11));
        assert_eq!(diff["new_text_length"], json!(11));
        assert_eq!(diff["text_length_delta"], json!(0));
        assert_eq!(diff["changed"], json!(true));
        assert_eq!(diff["previous_preview"], json!("before body"));
        assert_eq!(diff["new_preview"], json!("after body!"));
        assert_eq!(
            diff["previous_sha256"]
                .as_str()
                .expect("previous hash")
                .len(),
            64
        );
        assert_eq!(diff["new_sha256"].as_str().expect("new hash").len(), 64);
    }
}
```

### `backend/src/integrations/telegram/client/lifecycle/operations.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/lifecycle/operations.rs`
- Size bytes / Размер в байтах: `8163`
- Included characters / Включено символов: `8163`
- Truncated / Обрезано: `no`

```rust
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

use super::message_versions::{
    insert_message_version, latest_message_version, latest_version_number, local_edit_diff,
};
use super::tombstones::insert_tombstone;
use crate::integrations::telegram::client::TelegramStore;
use crate::integrations::telegram::client::commands::insert_command;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::models::messages::{
    TelegramCommandKind, TelegramDeleteRequest, TelegramEditRequest, TelegramLifecycleResponse,
    TelegramPinRequest, TelegramRestoreVisibilityRequest,
};

pub async fn record_edit(
    store: &TelegramStore,
    request: &TelegramEditRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let pool = store.pool();
    let now = Utc::now();
    let version_number = latest_version_number(pool, message_id).await? + 1;
    let previous_body = previous_message_body(store, message_id).await?;

    let _version = insert_message_version(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        version_number,
        Some(&request.new_text),
        now,
        None,
        local_edit_diff(previous_body.as_deref(), &request.new_text),
        json!({"event": "local_edit"}),
    )
    .await?;

    let idempotency_key = format!("edit:{}:{}", request.provider_message_id, version_number);
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::Edit.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "available",
        "provider_write",
        "confirmed",
        actor_id,
        json!({"new_text": &request.new_text}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"edit_version": version_number}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "edit".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "version_recorded".to_owned(),
        timestamp: now,
        version_number: Some(version_number),
        tombstone_id: None,
    })
}

async fn previous_message_body(
    store: &TelegramStore,
    message_id: &str,
) -> Result<Option<String>, TelegramError> {
    if let Some(version) = latest_message_version(store.pool(), message_id).await?
        && version.body_text.is_some()
    {
        return Ok(version.body_text);
    }

    Ok(store
        .provider_channel_message_store()
        .body_text(message_id)
        .await?)
}

pub async fn record_delete(
    pool: &PgPool,
    request: &TelegramDeleteRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();

    let tombstone = insert_tombstone(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        &request.reason_class,
        &request.actor_class,
        now,
        None,
        request.is_provider_delete,
        false,
    )
    .await?;

    let idempotency_key = format!(
        "delete:{}:{}",
        request.provider_message_id,
        now.timestamp_millis()
    );
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::Delete.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "available",
        "destructive",
        "confirmed",
        actor_id,
        json!({"reason_class": &request.reason_class, "is_provider_delete": request.is_provider_delete}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"tombstone_id": &tombstone.tombstone_id}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "delete".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "tombstone_recorded".to_owned(),
        timestamp: now,
        version_number: None,
        tombstone_id: Some(tombstone.tombstone_id),
    })
}

pub async fn record_restore_visibility(
    pool: &PgPool,
    request: &TelegramRestoreVisibilityRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let now = Utc::now();

    let tombstone = insert_tombstone(
        pool,
        message_id,
        &request.account_id,
        &request.provider_message_id,
        &request.provider_chat_id,
        "unknown",
        "owner",
        now,
        None,
        false,
        true,
    )
    .await?;

    let idempotency_key = format!(
        "restore_visibility:{}:{}",
        request.provider_message_id,
        now.timestamp_millis()
    );
    let _cmd = insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        TelegramCommandKind::RestoreVisibility.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "degraded",
        "local_write",
        "confirmed",
        actor_id,
        json!({"reason": &request.reason}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"tombstone_id": &tombstone.tombstone_id}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: "restore_visibility".to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: "visibility_restored".to_owned(),
        timestamp: now,
        version_number: None,
        tombstone_id: Some(tombstone.tombstone_id),
    })
}

pub async fn record_pin_state(
    store: &TelegramStore,
    request: &TelegramPinRequest,
    message_id: &str,
    actor_id: &str,
) -> Result<TelegramLifecycleResponse, TelegramError> {
    let pool = store.pool();
    let now = Utc::now();
    let message = store.message_by_id(message_id).await?.ok_or_else(|| {
        TelegramError::InvalidRequest(format!("telegram message `{message_id}` was not found"))
    })?;
    store
        .append_message_pin_observation(&message, request.is_pinned, now)
        .await?;

    let command_kind = if request.is_pinned {
        TelegramCommandKind::Pin
    } else {
        TelegramCommandKind::Unpin
    };
    let idempotency_key = format!(
        "{}:{}:{}",
        command_kind.as_str(),
        request.provider_message_id,
        now.timestamp_millis()
    );
    insert_command(
        pool,
        &request.command_id,
        &request.account_id,
        command_kind.as_str(),
        &idempotency_key,
        &request.provider_chat_id,
        Some(&request.provider_message_id),
        "degraded",
        "local_write",
        "confirmed",
        actor_id,
        json!({"is_pinned": request.is_pinned}),
        json!({"provider_message_id": &request.provider_message_id}),
        json!({"message_id": message_id, "is_pinned": request.is_pinned}),
    )
    .await?;

    Ok(TelegramLifecycleResponse {
        operation: command_kind.as_str().to_owned(),
        message_id: message_id.to_owned(),
        account_id: request.account_id.clone(),
        provider_chat_id: request.provider_chat_id.clone(),
        provider_message_id: request.provider_message_id.clone(),
        status: if request.is_pinned {
            "pinned".to_owned()
        } else {
            "unpinned".to_owned()
        },
        timestamp: now,
        version_number: None,
        tombstone_id: None,
    })
}
```

### `backend/src/integrations/telegram/client/lifecycle/provider_reconciliation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/lifecycle/provider_reconciliation.rs`
- Size bytes / Размер в байтах: `8908`
- Included characters / Включено символов: `8908`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::lifecycle::{
    mark_command_mismatch, mark_command_reconciled,
};
use crate::integrations::telegram::client::models::messages::TelegramProviderWriteCommand;
use crate::integrations::telegram::client::rows::row_to_telegram_provider_write_command;

const EDIT_PROVIDER_MISMATCH_ERROR: &str =
    "Provider observed a different message body than requested";
const PIN_PROVIDER_MISMATCH_ERROR: &str = "Provider observed a different pin state than requested";

pub async fn reconcile_edit_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    body_text: &str,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind = 'edit'
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let Some(new_text) = command
            .payload
            .get("new_text")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
        else {
            continue;
        };
        if new_text != body_text {
            let provider_state = json!({
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_body_text": new_text,
                "observed_body_text": body_text,
                "observed_via": observed_via,
            });
            let result_payload = json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_body_text": new_text,
                "observed_body_text": body_text,
                "provider_observed_at": observed_at,
                "mismatch": true,
            });
            reconciled.push(
                mark_command_mismatch(
                    pool,
                    &command.command_id,
                    observed_at,
                    provider_state,
                    result_payload,
                    EDIT_PROVIDER_MISMATCH_ERROR,
                )
                .await?,
            );
            continue;
        }

        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "body_text": body_text,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "body_text": body_text,
            "provider_observed_at": observed_at,
        });
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state,
                result_payload,
            )
            .await?,
        );
    }

    Ok(reconciled)
}

pub async fn reconcile_message_pin_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    is_pinned: bool,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind IN ('pin', 'unpin')
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let expected_is_pinned = match command.command_kind.as_str() {
            "pin" => Some(true),
            "unpin" => Some(false),
            _ => None,
        };
        let Some(expected_is_pinned) = expected_is_pinned else {
            continue;
        };
        if expected_is_pinned != is_pinned {
            let provider_state = json!({
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_is_pinned": expected_is_pinned,
                "observed_is_pinned": is_pinned,
                "observed_via": observed_via,
            });
            let result_payload = json!({
                "source": observed_via,
                "provider_chat_id": provider_chat_id,
                "provider_message_id": provider_message_id,
                "expected_is_pinned": expected_is_pinned,
                "observed_is_pinned": is_pinned,
                "provider_observed_at": observed_at,
                "mismatch": true,
            });
            reconciled.push(
                mark_command_mismatch(
                    pool,
                    &command.command_id,
                    observed_at,
                    provider_state,
                    result_payload,
                    PIN_PROVIDER_MISMATCH_ERROR,
                )
                .await?,
            );
            continue;
        }
        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_pinned": is_pinned,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_pinned": is_pinned,
            "provider_observed_at": observed_at,
        });
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state,
                result_payload,
            )
            .await?,
        );
    }

    Ok(reconciled)
}

pub async fn reconcile_delete_commands_from_provider_state(
    pool: &PgPool,
    account_id: &str,
    provider_chat_id: &str,
    provider_message_id: &str,
    observed_at: DateTime<Utc>,
    observed_via: &str,
) -> Result<Vec<TelegramProviderWriteCommand>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM telegram_provider_write_commands
        WHERE account_id = $1
          AND provider_chat_id = $2
          AND provider_message_id = $3
          AND command_kind = 'delete'
          AND status IN ('queued', 'retrying', 'executing')
          AND confirmation_decision IN ('confirmed', 'not_required')
          AND capability_state IN ('available', 'degraded')
        ORDER BY created_at ASC, command_id ASC
        "#,
    )
    .bind(account_id)
    .bind(provider_chat_id)
    .bind(provider_message_id)
    .fetch_all(pool)
    .await?;

    let mut reconciled = Vec::new();
    for row in rows {
        let command = row_to_telegram_provider_write_command(row)?;
        let provider_state = json!({
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_deleted": true,
            "observed_via": observed_via,
        });
        let result_payload = json!({
            "source": observed_via,
            "provider_chat_id": provider_chat_id,
            "provider_message_id": provider_message_id,
            "is_deleted": true,
            "provider_observed_at": observed_at,
        });
        reconciled.push(
            mark_command_reconciled(
                pool,
                &command.command_id,
                observed_at,
                provider_state,
                result_payload,
            )
            .await?,
        );
    }

    Ok(reconciled)
}
```

### `backend/src/integrations/telegram/client/lifecycle/tombstones.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/telegram/client/lifecycle/tombstones.rs`
- Size bytes / Размер в байтах: `7577`
- Included characters / Включено символов: `7577`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};

use super::ids::new_tombstone_id;
use crate::integrations::telegram::client::errors::TelegramError;
use crate::integrations::telegram::client::evidence::link_telegram_entity_in_transaction;
use crate::integrations::telegram::client::models::TelegramMessage;
use crate::integrations::telegram::client::models::messages::TelegramMessageTombstone;
use crate::integrations::telegram::client::rows::row_to_telegram_message_tombstone;
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

async fn capture_tombstone_observation_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    tombstone: &TelegramMessageTombstone,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), TelegramError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "TELEGRAM_MESSAGE_TOMBSTONE",
            ObservationOriginKind::LocalRuntime,
            tombstone.created_at,
            json!({
                "tombstone_id": tombstone.tombstone_id,
                "message_id": tombstone.message_id,
                "account_id": tombstone.account_id,
                "provider_message_id": tombstone.provider_message_id,
                "provider_chat_id": tombstone.provider_chat_id,
                "reason_class": tombstone.reason_class,
                "actor_class": tombstone.actor_class,
                "observed_at": tombstone.observed_at,
                "source_event": tombstone.source_event,
                "is_provider_delete": tombstone.is_provider_delete,
                "is_local_visible": tombstone.is_local_visible,
                "metadata": tombstone.metadata,
                "provenance": tombstone.provenance,
                "operation": relationship_kind,
            }),
            format!(
                "telegram-message-tombstone://{}/{}",
                tombstone.tombstone_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
            "provider": "telegram",
        })),
    )
    .await?;
    link_telegram_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "message_tombstone",
        tombstone.tombstone_id.clone(),
        relationship_kind,
        json!({
            "message_id": tombstone.message_id,
            "account_id": tombstone.account_id,
            "provider_message_id": tombstone.provider_message_id,
            "provider_chat_id": tombstone.provider_chat_id,
            "reason_class": tombstone.reason_class,
            "actor_class": tombstone.actor_class,
            "is_local_visible": tombstone.is_local_visible,
        }),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_tombstone(
    pool: &PgPool,
    message_id: &str,
    account_id: &str,
    provider_message_id: &str,
    provider_chat_id: &str,
    reason_class: &str,
    actor_class: &str,
    observed_at: DateTime<Utc>,
    source_event: Option<&str>,
    is_provider_delete: bool,
    is_local_visible: bool,
) -> Result<TelegramMessageTombstone, TelegramError> {
    let tombstone_id = new_tombstone_id();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_message_tombstones
            (tombstone_id, message_id, account_id, provider_message_id, provider_chat_id,
             reason_class, actor_class, observed_at, source_event,
             is_provider_delete, is_local_visible)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(&tombstone_id)
    .bind(message_id)
    .bind(account_id)
    .bind(provider_message_id)
    .bind(provider_chat_id)
    .bind(reason_class)
    .bind(actor_class)
    .bind(observed_at)
    .bind(source_event)
    .bind(is_provider_delete)
    .bind(is_local_visible)
    .fetch_one(&mut *transaction)
    .await?;

    let tombstone = row_to_telegram_message_tombstone(row)?;
    capture_tombstone_observation_in_transaction(
        &mut transaction,
        &tombstone,
        "insert",
        "telegram.client.lifecycle.tombstones.insert_tombstone",
    )
    .await?;
    transaction.commit().await?;
    Ok(tombstone)
}

pub async fn list_tombstones(
    pool: &PgPool,
    message_id: &str,
) -> Result<Vec<TelegramMessageTombstone>, TelegramError> {
    let rows = sqlx::query(
        r#"
        SELECT * FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_telegram_message_tombstone)
        .collect()
}

pub async fn is_message_visible(pool: &PgPool, message_id: &str) -> Result<bool, TelegramError> {
    let row: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT is_local_visible
        FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(true))
}

pub async fn record_provider_delete_observation(
    pool: &PgPool,
    message: &TelegramMessage,
    observed_at: DateTime<Utc>,
    source_event: &str,
    is_provider_delete: bool,
    from_cache: bool,
) -> Result<TelegramMessageTombstone, TelegramError> {
    let latest = sqlx::query(
        r#"
        SELECT *
        FROM telegram_message_tombstones
        WHERE message_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&message.message_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = latest {
        let tombstone = row_to_telegram_message_tombstone(row)?;
        if tombstone.reason_class == "deleted_by_provider"
            && tombstone.actor_class == "provider"
            && !tombstone.is_local_visible
        {
            return Ok(tombstone);
        }
    }

    let tombstone_id = new_tombstone_id();
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO telegram_message_tombstones
            (tombstone_id, message_id, account_id, provider_message_id, provider_chat_id,
             reason_class, actor_class, observed_at, source_event,
             is_provider_delete, is_local_visible, metadata, provenance)
        VALUES ($1, $2, $3, $4, $5, 'deleted_by_provider', 'provider', $6, $7, $8, false, $9, $10)
        RETURNING *
        "#,
    )
    .bind(&tombstone_id)
    .bind(&message.message_id)
    .bind(&message.account_id)
    .bind(&message.provider_message_id)
    .bind(message.provider_chat_id.as_deref().unwrap_or_default())
    .bind(observed_at)
    .bind(source_event)
    .bind(is_provider_delete)
    .bind(json!({
        "from_cache": from_cache,
        "provider_delete": is_provider_delete,
    }))
    .bind(json!({
        "provider": "telegram",
        "runtime": "tdlib",
        "source": source_event,
    }))
    .fetch_one(&mut *transaction)
    .await?;

    let tombstone = row_to_telegram_message_tombstone(row)?;
    capture_tombstone_observation_in_transaction(
        &mut transaction,
        &tombstone,
        "provider_delete",
        "telegram.client.lifecycle.tombstones.record_provider_delete_observation",
    )
    .await?;
    transaction.commit().await?;
    Ok(tombstone)
}
```

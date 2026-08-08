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

- Chunk ID / ID чанка: `058-source-backend-part-038`
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

### `backend/src/integrations/mail/accounts/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/helpers.rs`
- Size bytes / Размер в байтах: `2531`
- Included characters / Включено символов: `2531`
- Truncated / Обрезано: `no`

```rust
use std::time::Duration;

use aes_gcm::aead::rand_core::RngCore;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, TimeDelta, Utc};
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::platform::communications::EmailProviderKind;
use crate::platform::secrets::{SecretKind, SecretReference, SecretStoreKind};

pub(super) fn http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client configuration must be valid")
}

pub(super) fn expires_at(expires_in: Option<i64>) -> DateTime<Utc> {
    let seconds = expires_in.unwrap_or(3600).saturating_sub(60).max(60);
    Utc::now() + TimeDelta::seconds(seconds)
}

pub(super) fn oauth_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:oauth_token")
}

pub(super) fn imap_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:imap_password")
}

pub(super) fn smtp_secret_ref(account_id: &str) -> String {
    format!("secret:provider-account:{account_id}:smtp_password")
}

pub(super) fn email_provider_connected_services(
    provider_kind: EmailProviderKind,
) -> Option<&'static [&'static str]> {
    match provider_kind {
        EmailProviderKind::Gmail | EmailProviderKind::Icloud => {
            Some(&["mail", "calendar", "contacts"])
        }
        EmailProviderKind::Imap
        | EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb
        | EmailProviderKind::WhatsappBusinessCloud
        | EmailProviderKind::ZoomUser
        | EmailProviderKind::ZoomServerToServer
        | EmailProviderKind::YandexTelemostUser => None,
    }
}

pub(super) fn vault_secret_reference(
    secret_ref: &str,
    secret_kind: SecretKind,
    store_kind: SecretStoreKind,
) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind,
        store_kind,
        label: "encrypted vault secret".to_owned(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}

pub(super) fn random_url_token() -> String {
    let mut bytes = [0_u8; 32];
    aes_gcm::aead::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub(super) fn pkce_challenge(code_verifier: &str) -> String {
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}
```

### `backend/src/integrations/mail/accounts/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/models.rs`
- Size bytes / Размер в байтах: `9713`
- Included characters / Включено символов: `9713`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::platform::communications::EmailProviderKind;
use crate::platform::secrets::{SecretKind, SecretStoreKind};

use super::constants::{
    DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT, DEFAULT_GOOGLE_TOKEN_ENDPOINT,
    DEFAULT_GOOGLE_WORKSPACE_SCOPES,
};
use super::errors::EmailAccountSetupError;
use super::validation::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthSetupRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub app_return_url: Option<String>,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub scopes: Vec<String>,
}

impl GmailOAuthSetupRequest {
    pub fn new(
        account_id: impl Into<String>,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        client_id: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            client_id: client_id.into(),
            client_secret: None,
            redirect_uri: redirect_uri.into(),
            app_return_url: None,
            authorization_endpoint: DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT.to_owned(),
            token_endpoint: DEFAULT_GOOGLE_TOKEN_ENDPOINT.to_owned(),
            scopes: DEFAULT_GOOGLE_WORKSPACE_SCOPES
                .iter()
                .map(|scope| (*scope).to_owned())
                .collect(),
        }
    }

    pub fn external_account_id(mut self, external_account_id: impl Into<String>) -> Self {
        self.external_account_id = external_account_id.into();
        self
    }

    pub fn client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    pub fn app_return_url(mut self, app_return_url: impl Into<String>) -> Self {
        self.app_return_url = Some(app_return_url.into());
        self
    }

    pub fn authorization_endpoint(mut self, authorization_endpoint: impl Into<String>) -> Self {
        self.authorization_endpoint = authorization_endpoint.into();
        self
    }

    pub fn token_endpoint(mut self, token_endpoint: impl Into<String>) -> Self {
        self.token_endpoint = token_endpoint.into();
        self
    }

    pub fn scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes = scopes.into_iter().map(Into::into).collect();
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailAccountSetupError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("client_id", &self.client_id)?;
        validate_non_empty("redirect_uri", &self.redirect_uri)?;
        validate_non_empty("authorization_endpoint", &self.authorization_endpoint)?;
        validate_non_empty("token_endpoint", &self.token_endpoint)?;
        if self.scopes.is_empty() {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "scopes",
                message: "must not be empty",
            });
        }
        for scope in &self.scopes {
            validate_non_empty("scope", scope)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthPendingGrant {
    pub setup_id: String,
    pub account_id: String,
    pub authorization_url: String,
    pub state: String,
    pub code_verifier: String,
    pub request: GmailOAuthSetupRequest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub password: String,
    pub secret_kind: SecretKind,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_tls: Option<bool>,
    pub smtp_starttls: Option<bool>,
    pub smtp_username: Option<String>,
}

impl ImapAccountSetupRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_id: impl Into<String>,
        provider_kind: EmailProviderKind,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        tls: bool,
        mailbox: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            provider_kind,
            display_name: display_name.into(),
            external_account_id: external_account_id.into(),
            host: host.into(),
            port,
            tls,
            mailbox: mailbox.into(),
            username: username.into(),
            password: password.into(),
            secret_kind: SecretKind::Password,
            smtp_host: None,
            smtp_port: None,
            smtp_tls: None,
            smtp_starttls: None,
            smtp_username: None,
        }
    }

    pub fn secret_kind(mut self, secret_kind: SecretKind) -> Self {
        self.secret_kind = secret_kind;
        self
    }

    pub fn smtp_host(mut self, smtp_host: impl Into<String>) -> Self {
        self.smtp_host = Some(smtp_host.into());
        self
    }

    pub fn smtp_port(mut self, smtp_port: u16) -> Self {
        self.smtp_port = Some(smtp_port);
        self
    }

    pub fn smtp_tls(mut self, smtp_tls: bool) -> Self {
        self.smtp_tls = Some(smtp_tls);
        self
    }

    pub fn smtp_starttls(mut self, smtp_starttls: bool) -> Self {
        self.smtp_starttls = Some(smtp_starttls);
        self
    }

    pub fn smtp_username(mut self, smtp_username: impl Into<String>) -> Self {
        self.smtp_username = Some(smtp_username.into());
        self
    }

    pub(super) fn smtp_config(&self) -> ImapAccountSmtpConfig {
        let default_host = match self.provider_kind {
            EmailProviderKind::Icloud => "smtp.mail.me.com".to_owned(),
            _ => self.host.clone(),
        };
        ImapAccountSmtpConfig {
            host: self.smtp_host.clone().unwrap_or(default_host),
            port: self.smtp_port.unwrap_or(587),
            tls: self.smtp_tls.unwrap_or(true),
            starttls: self.smtp_starttls.unwrap_or(true),
            username: self
                .smtp_username
                .clone()
                .unwrap_or_else(|| self.external_account_id.clone()),
        }
    }

    pub(super) fn validate(&self) -> Result<(), EmailAccountSetupError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("host", &self.host)?;
        validate_non_empty("mailbox", &self.mailbox)?;
        validate_non_empty("username", &self.username)?;
        validate_non_empty("password", &self.password)?;
        if self.provider_kind == EmailProviderKind::Gmail {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "provider_kind",
                message: "must be icloud or imap",
            });
        }
        if self.port == 0 {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }
        if !matches!(
            self.secret_kind,
            SecretKind::AppPassword | SecretKind::Password
        ) {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "secret_kind",
                message: "must be app_password or password",
            });
        }
        let smtp_config = self.smtp_config();
        validate_non_empty("smtp_host", &smtp_config.host)?;
        validate_non_empty("smtp_username", &smtp_config.username)?;
        if smtp_config.port == 0 {
            return Err(EmailAccountSetupError::InvalidRequest {
                field: "smtp_port",
                message: "must be greater than zero",
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ImapAccountSmtpConfig {
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) tls: bool,
    pub(super) starttls: bool,
    pub(super) username: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EmailAccountSetupResult {
    pub account_id: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct GmailOAuthTokenBundle {
    pub(super) token_url: String,
    pub(super) client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) client_secret: Option<String>,
    pub(super) access_token: String,
    pub(super) refresh_token: String,
    pub(super) expires_at: DateTime<Utc>,
    pub(super) token_type: Option<String>,
    pub(super) scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OAuthTokenResponse {
    pub(super) access_token: String,
    pub(super) refresh_token: Option<String>,
    pub(super) expires_in: Option<i64>,
    pub(super) token_type: Option<String>,
    pub(super) scope: Option<String>,
}
```

### `backend/src/integrations/mail/accounts/service.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service.rs`
- Size bytes / Размер в байтах: `1095`
- Included characters / Включено символов: `1095`
- Truncated / Обрезано: `no`

```rust
mod constructors;
mod gmail_complete;
mod gmail_payloads;
mod gmail_refresh;
mod gmail_start;
mod imap;
mod imap_payloads;
mod stores;
mod token_http;

use reqwest::Client;
use sqlx::postgres::PgPool;
use std::sync::Arc;

use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderSecretBindingCommandPort,
};
use crate::platform::secrets::SecretReferenceStore;

use super::vault::AccountSecretVault;

#[derive(Clone)]
pub struct EmailAccountSetupService {
    pub(in crate::integrations::mail::accounts::service) pool: Option<PgPool>,
    pub(in crate::integrations::mail::accounts::service) secret_store: Option<SecretReferenceStore>,
    pub(in crate::integrations::mail::accounts::service) provider_account_store:
        Option<Arc<dyn ProviderAccountCommandPort>>,
    pub(in crate::integrations::mail::accounts::service) provider_secret_binding_store:
        Option<Arc<dyn ProviderSecretBindingCommandPort>>,
    pub(in crate::integrations::mail::accounts::service) vault: AccountSecretVault,
    pub(in crate::integrations::mail::accounts::service) http: Client,
}
```

### `backend/src/integrations/mail/accounts/service/constructors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/constructors.rs`
- Size bytes / Размер в байтах: `2481`
- Included characters / Включено символов: `2481`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderSecretBindingCommandPort,
};
use crate::platform::secrets::{DatabaseEncryptedSecretVault, SecretReferenceStore};
use crate::vault::HostVault;
use sqlx::postgres::PgPool;

use super::super::helpers::http_client;
use super::super::vault::AccountSecretVault;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub fn new(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: DatabaseEncryptedSecretVault,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            provider_account_store: Some(provider_account_store),
            provider_secret_binding_store: Some(provider_secret_binding_store),
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_for_vault_only(vault: DatabaseEncryptedSecretVault) -> Self {
        Self {
            pool: None,
            secret_store: None,
            provider_account_store: None,
            provider_secret_binding_store: None,
            vault: AccountSecretVault::Database(vault),
            http: http_client(),
        }
    }

    pub fn new_with_host_vault(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: HostVault,
        provider_account_store: Arc<dyn ProviderAccountCommandPort>,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            provider_account_store: Some(provider_account_store),
            provider_secret_binding_store: Some(provider_secret_binding_store),
            vault: AccountSecretVault::Host(vault),
            http: http_client(),
        }
    }

    pub fn new_with_host_vault_for_token_refresh(
        pool: PgPool,
        secret_store: SecretReferenceStore,
        vault: HostVault,
    ) -> Self {
        Self {
            pool: Some(pool),
            secret_store: Some(secret_store),
            provider_account_store: None,
            provider_secret_binding_store: None,
            vault: AccountSecretVault::Host(vault),
            http: http_client(),
        }
    }
}
```

### `backend/src/integrations/mail/accounts/service/gmail_complete.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/gmail_complete.rs`
- Size bytes / Размер в байтах: `4474`
- Included characters / Включено символов: `4474`
- Truncated / Обрезано: `no`

```rust
use crate::platform::communications::{
    EmailProviderKind, NewProviderAccount, NewProviderAccountSecretBinding,
    ProviderAccountSecretPurpose,
};
use crate::platform::secrets::{NewSecretReference, SecretKind};

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::{expires_at, oauth_secret_ref};
use super::super::models::{
    EmailAccountSetupResult, GmailOAuthPendingGrant, GmailOAuthTokenBundle,
};
use super::super::vault::SecretWriteContext;
use super::EmailAccountSetupService;
use super::gmail_payloads::{gmail_account_config, gmail_secret_metadata};

impl EmailAccountSetupService {
    pub async fn complete_gmail_oauth(
        &self,
        pending: GmailOAuthPendingGrant,
        authorization_code: &str,
    ) -> Result<EmailAccountSetupResult, EmailAccountSetupError> {
        super::super::validation::validate_non_empty("authorization_code", authorization_code)?;
        let external_account_id = if pending.request.external_account_id.trim().is_empty() {
            pending.account_id.clone()
        } else {
            pending.request.external_account_id.clone()
        };
        let token = self
            .exchange_authorization_code(&pending, authorization_code)
            .await?;
        let refresh_token =
            token
                .refresh_token
                .clone()
                .ok_or(EmailAccountSetupError::MissingProviderField {
                    field: "refresh_token",
                })?;
        let expires_at = expires_at(token.expires_in);
        let secret_ref = oauth_secret_ref(&pending.account_id);
        let token_bundle = GmailOAuthTokenBundle {
            token_url: pending.request.token_endpoint.clone(),
            client_id: pending.request.client_id.clone(),
            client_secret: pending.request.client_secret.clone(),
            access_token: token.access_token,
            refresh_token,
            expires_at,
            token_type: token.token_type,
            scope: token.scope,
        };
        let secret_store = self.secret_store()?;
        let provider_account_store = self.provider_account_store()?;
        let secret_binding_store = self.provider_secret_binding_store()?;
        let account_config = gmail_account_config(&pending);
        let secret_metadata =
            gmail_secret_metadata(&pending, &external_account_id, &account_config);
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    SecretKind::OauthToken,
                    self.vault.store_kind(),
                    format!(
                        "Gmail OAuth credential for {}",
                        pending.request.display_name
                    ),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        self.vault
            .store_secret(
                &secret_ref,
                &serde_json::to_string(&token_bundle)?,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &pending.account_id,
                    purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                    secret_kind: SecretKind::OauthToken,
                    label: "Gmail OAuth credential",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        provider_account_store
            .upsert(
                &NewProviderAccount::new(
                    &pending.account_id,
                    EmailProviderKind::Gmail,
                    &pending.request.display_name,
                    &external_account_id,
                )
                .config(account_config),
            )
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;
        secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &pending.account_id,
                ProviderAccountSecretPurpose::OauthToken,
                &secret_ref,
            ))
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;

        Ok(EmailAccountSetupResult {
            account_id: pending.account_id,
            secret_ref,
            secret_kind: SecretKind::OauthToken,
            store_kind: self.vault.store_kind(),
        })
    }
}
```

### `backend/src/integrations/mail/accounts/service/gmail_payloads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/gmail_payloads.rs`
- Size bytes / Размер в байтах: `1320`
- Included characters / Включено символов: `1320`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::constants::GOOGLE_GMAIL_SEND_SCOPE;
use super::super::models::GmailOAuthPendingGrant;

pub(in crate::integrations::mail::accounts::service) fn gmail_account_config(
    pending: &GmailOAuthPendingGrant,
) -> Value {
    json!({
        "auth": "oauth",
        "api": "gmail",
        "oauth_client_id": pending.request.client_id,
        "requested_scopes": pending.request.scopes,
        "gmail_send_enabled": gmail_send_scope_requested(pending),
        "connected_services": ["mail", "calendar", "contacts"],
        "history_stream_id": "gmail:history"
    })
}

pub(in crate::integrations::mail::accounts::service) fn gmail_secret_metadata(
    pending: &GmailOAuthPendingGrant,
    external_account_id: &str,
    account_config: &Value,
) -> Value {
    json!({
        "provider": "gmail",
        "account_id": pending.account_id,
        "display_name": pending.request.display_name,
        "external_account_id": external_account_id,
        "connected_services": ["mail", "calendar", "contacts"],
        "provider_account_config": account_config
    })
}

fn gmail_send_scope_requested(pending: &GmailOAuthPendingGrant) -> bool {
    pending
        .request
        .scopes
        .iter()
        .any(|scope| scope.trim() == GOOGLE_GMAIL_SEND_SCOPE)
}
```

### `backend/src/integrations/mail/accounts/service/gmail_refresh.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/gmail_refresh.rs`
- Size bytes / Размер в байтах: `2269`
- Included characters / Включено символов: `2269`
- Truncated / Обрезано: `no`

```rust
use chrono::{TimeDelta, Utc};
use serde_json::json;

use crate::platform::communications::ProviderAccountSecretPurpose;
use crate::platform::secrets::{ResolvedSecret, SecretKind, SecretResolver};

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::expires_at;
use super::super::models::GmailOAuthTokenBundle;
use super::super::validation::validate_non_empty;
use super::super::vault::SecretWriteContext;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub async fn refresh_gmail_access_token(
        &self,
        secret_ref: &str,
    ) -> Result<ResolvedSecret, EmailAccountSetupError> {
        validate_non_empty("secret_ref", secret_ref)?;
        let reference = self
            .vault
            .secret_reference(secret_ref, SecretKind::OauthToken);
        let resolved = self.vault.resolve(&reference).await?;
        let mut bundle: GmailOAuthTokenBundle =
            serde_json::from_str(resolved.expose_for_runtime())?;

        if bundle.expires_at > Utc::now() + TimeDelta::seconds(60) {
            return ResolvedSecret::new(bundle.access_token)
                .map_err(EmailAccountSetupError::Secret);
        }

        let refreshed = self.refresh_token(&bundle).await?;
        bundle.access_token = refreshed.access_token;
        if let Some(refresh_token) = refreshed.refresh_token {
            bundle.refresh_token = refresh_token;
        }
        bundle.expires_at = expires_at(refreshed.expires_in);
        bundle.token_type = refreshed.token_type;
        if refreshed.scope.is_some() {
            bundle.scope = refreshed.scope;
        }

        self.vault
            .store_secret(
                secret_ref,
                &serde_json::to_string(&bundle)?,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: secret_ref,
                    purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                    secret_kind: SecretKind::OauthToken,
                    label: "OAuth credential",
                    metadata: &json!({}),
                },
            )
            .await?;
        ResolvedSecret::new(bundle.access_token).map_err(EmailAccountSetupError::Secret)
    }
}
```

### `backend/src/integrations/mail/accounts/service/gmail_start.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/gmail_start.rs`
- Size bytes / Размер в байтах: `1582`
- Included characters / Включено символов: `1582`
- Truncated / Обрезано: `no`

```rust
use url::form_urlencoded;

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::{pkce_challenge, random_url_token};
use super::super::models::{GmailOAuthPendingGrant, GmailOAuthSetupRequest};
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub fn start_gmail_oauth(
        &self,
        request: GmailOAuthSetupRequest,
    ) -> Result<GmailOAuthPendingGrant, EmailAccountSetupError> {
        request.validate()?;
        let setup_id = random_url_token();
        let state = random_url_token();
        let code_verifier = random_url_token();
        let code_challenge = pkce_challenge(&code_verifier);
        let scope = request.scopes.join(" ");
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("response_type", "code")
            .append_pair("client_id", &request.client_id)
            .append_pair("redirect_uri", &request.redirect_uri)
            .append_pair("scope", &scope)
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .finish();
        let authorization_url = format!("{}?{query}", request.authorization_endpoint);

        Ok(GmailOAuthPendingGrant {
            setup_id,
            account_id: request.account_id.clone(),
            authorization_url,
            state,
            code_verifier,
            request,
        })
    }
}
```

### `backend/src/integrations/mail/accounts/service/imap.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/imap.rs`
- Size bytes / Размер в байтах: `4466`
- Included characters / Включено символов: `4466`
- Truncated / Обрезано: `no`

```rust
use crate::platform::communications::{
    NewProviderAccount, NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use crate::platform::secrets::NewSecretReference;

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::{imap_secret_ref, smtp_secret_ref};
use super::super::models::{EmailAccountSetupResult, ImapAccountSetupRequest};
use super::super::vault::SecretWriteContext;
use super::EmailAccountSetupService;
use super::imap_payloads::{imap_account_config, imap_secret_metadata};

impl EmailAccountSetupService {
    pub async fn setup_imap_account(
        &self,
        request: ImapAccountSetupRequest,
    ) -> Result<EmailAccountSetupResult, EmailAccountSetupError> {
        request.validate()?;
        let secret_ref = imap_secret_ref(&request.account_id);
        let smtp_secret_ref = smtp_secret_ref(&request.account_id);
        let account_config = imap_account_config(&request);
        let secret_metadata = imap_secret_metadata(&request, &account_config);

        let secret_store = self.secret_store()?;
        let provider_account_store = self.provider_account_store()?;
        let secret_binding_store = self.provider_secret_binding_store()?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    request.secret_kind,
                    self.vault.store_kind(),
                    format!("IMAP credential for {}", request.display_name),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        self.vault
            .store_secret(
                &secret_ref,
                &request.password,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &request.account_id,
                    purpose: ProviderAccountSecretPurpose::ImapPassword.as_str(),
                    secret_kind: request.secret_kind,
                    label: "IMAP password",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &smtp_secret_ref,
                    request.secret_kind,
                    self.vault.store_kind(),
                    format!("SMTP credential for {}", request.display_name),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        provider_account_store
            .upsert(
                &NewProviderAccount::new(
                    &request.account_id,
                    request.provider_kind,
                    &request.display_name,
                    &request.external_account_id,
                )
                .config(account_config),
            )
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;
        secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &request.account_id,
                ProviderAccountSecretPurpose::ImapPassword,
                &secret_ref,
            ))
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;
        self.vault
            .store_secret(
                &smtp_secret_ref,
                &request.password,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &request.account_id,
                    purpose: ProviderAccountSecretPurpose::SmtpPassword.as_str(),
                    secret_kind: request.secret_kind,
                    label: "SMTP password",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &request.account_id,
                ProviderAccountSecretPurpose::SmtpPassword,
                &smtp_secret_ref,
            ))
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;

        Ok(EmailAccountSetupResult {
            account_id: request.account_id,
            secret_ref,
            secret_kind: request.secret_kind,
            store_kind: self.vault.store_kind(),
        })
    }
}
```

### `backend/src/integrations/mail/accounts/service/imap_payloads.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/imap_payloads.rs`
- Size bytes / Размер в байтах: `1548`
- Included characters / Включено символов: `1548`
- Truncated / Обрезано: `no`

```rust
use serde_json::{Value, json};

use super::super::helpers::email_provider_connected_services;
use super::super::models::ImapAccountSetupRequest;

pub(in crate::integrations::mail::accounts::service) fn imap_account_config(
    request: &ImapAccountSetupRequest,
) -> Value {
    let smtp_config = request.smtp_config();
    let mut account_config = json!({
        "host": request.host,
        "port": request.port,
        "tls": request.tls,
        "mailbox": request.mailbox,
        "username": request.username,
        "smtp_host": smtp_config.host,
        "smtp_port": smtp_config.port,
        "smtp_tls": smtp_config.tls,
        "smtp_starttls": smtp_config.starttls,
        "smtp_username": smtp_config.username
    });
    if let Some(services) = email_provider_connected_services(request.provider_kind) {
        account_config["connected_services"] = json!(services);
    }
    account_config
}

pub(in crate::integrations::mail::accounts::service) fn imap_secret_metadata(
    request: &ImapAccountSetupRequest,
    account_config: &Value,
) -> Value {
    let mut secret_metadata = json!({
        "provider": request.provider_kind.as_str(),
        "account_id": request.account_id,
        "display_name": request.display_name,
        "external_account_id": request.external_account_id,
        "provider_account_config": account_config
    });
    if let Some(services) = email_provider_connected_services(request.provider_kind) {
        secret_metadata["connected_services"] = json!(services);
    }
    secret_metadata
}
```

### `backend/src/integrations/mail/accounts/service/stores.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/stores.rs`
- Size bytes / Размер в байтах: `1479`
- Included characters / Включено символов: `1479`
- Truncated / Обрезано: `no`

```rust
use std::sync::Arc;

use crate::platform::communications::{
    ProviderAccountCommandPort, ProviderSecretBindingCommandPort,
};
use crate::platform::secrets::SecretReferenceStore;
use sqlx::postgres::PgPool;

use super::super::errors::EmailAccountSetupError;
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub(in crate::integrations::mail::accounts::service) fn pool(
        &self,
    ) -> Result<&PgPool, EmailAccountSetupError> {
        self.pool
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    pub(in crate::integrations::mail::accounts::service) fn secret_store(
        &self,
    ) -> Result<&SecretReferenceStore, EmailAccountSetupError> {
        self.secret_store
            .as_ref()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    pub(in crate::integrations::mail::accounts::service) fn provider_account_store(
        &self,
    ) -> Result<Arc<dyn ProviderAccountCommandPort>, EmailAccountSetupError> {
        self.provider_account_store
            .clone()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }

    pub(in crate::integrations::mail::accounts::service) fn provider_secret_binding_store(
        &self,
    ) -> Result<Arc<dyn ProviderSecretBindingCommandPort>, EmailAccountSetupError> {
        self.provider_secret_binding_store
            .clone()
            .ok_or(EmailAccountSetupError::StoresNotConfigured)
    }
}
```

### `backend/src/integrations/mail/accounts/service/token_http.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/service/token_http.rs`
- Size bytes / Размер в байтах: `2000`
- Included characters / Включено символов: `2000`
- Truncated / Обрезано: `no`

```rust
use super::super::errors::EmailAccountSetupError;
use super::super::models::{GmailOAuthPendingGrant, GmailOAuthTokenBundle, OAuthTokenResponse};
use super::EmailAccountSetupService;

impl EmailAccountSetupService {
    pub(in crate::integrations::mail::accounts::service) async fn exchange_authorization_code(
        &self,
        pending: &GmailOAuthPendingGrant,
        authorization_code: &str,
    ) -> Result<OAuthTokenResponse, EmailAccountSetupError> {
        let mut form = vec![
            ("grant_type", "authorization_code".to_owned()),
            ("code", authorization_code.trim().to_owned()),
            ("client_id", pending.request.client_id.clone()),
            ("redirect_uri", pending.request.redirect_uri.clone()),
            ("code_verifier", pending.code_verifier.clone()),
        ];
        if let Some(client_secret) = &pending.request.client_secret {
            form.push(("client_secret", client_secret.clone()));
        }

        Ok(self
            .http
            .post(&pending.request.token_endpoint)
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<OAuthTokenResponse>()
            .await?)
    }

    pub(in crate::integrations::mail::accounts::service) async fn refresh_token(
        &self,
        bundle: &GmailOAuthTokenBundle,
    ) -> Result<OAuthTokenResponse, EmailAccountSetupError> {
        let mut form = vec![
            ("grant_type", "refresh_token".to_owned()),
            ("refresh_token", bundle.refresh_token.clone()),
            ("client_id", bundle.client_id.clone()),
        ];
        if let Some(client_secret) = &bundle.client_secret {
            form.push(("client_secret", client_secret.clone()));
        }

        Ok(self
            .http
            .post(&bundle.token_url)
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<OAuthTokenResponse>()
            .await?)
    }
}
```

### `backend/src/integrations/mail/accounts/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/validation.rs`
- Size bytes / Размер в байтах: `348`
- Included characters / Включено символов: `348`
- Truncated / Обрезано: `no`

```rust
use super::errors::EmailAccountSetupError;

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), EmailAccountSetupError> {
    if value.trim().is_empty() {
        return Err(EmailAccountSetupError::InvalidRequest {
            field,
            message: "must not be empty",
        });
    }

    Ok(())
}
```

### `backend/src/integrations/mail/accounts/vault.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/accounts/vault.rs`
- Size bytes / Размер в байтах: `2277`
- Included characters / Включено символов: `2277`
- Truncated / Обрезано: `no`

```rust
use serde_json::Value;

use crate::platform::secrets::{
    DatabaseEncryptedSecretVault, SecretKind, SecretReference, SecretResolutionFuture,
    SecretResolver, SecretStoreKind,
};
use crate::vault::{HostVault, SecretEntryContext};

use super::errors::EmailAccountSetupError;
use super::helpers::vault_secret_reference;

#[derive(Clone)]
pub(super) enum AccountSecretVault {
    Database(DatabaseEncryptedSecretVault),
    Host(HostVault),
}

impl AccountSecretVault {
    pub(super) fn store_kind(&self) -> SecretStoreKind {
        match self {
            Self::Database(_) => SecretStoreKind::DatabaseEncryptedVault,
            Self::Host(_) => SecretStoreKind::HostVault,
        }
    }

    pub(super) async fn store_secret(
        &self,
        secret_ref: &str,
        value: &str,
        context: SecretWriteContext<'_>,
    ) -> Result<(), EmailAccountSetupError> {
        match self {
            Self::Database(vault) => vault.store_secret(secret_ref, value).await?,
            Self::Host(vault) => vault.store_secret(
                secret_ref,
                value,
                SecretEntryContext {
                    entry_kind: context.entry_kind,
                    account_id: context.account_id,
                    purpose: context.purpose,
                    secret_kind: context.secret_kind.as_str(),
                    label: context.label,
                    metadata: context.metadata,
                },
            )?,
        }
        Ok(())
    }

    pub(super) fn secret_reference(
        &self,
        secret_ref: &str,
        secret_kind: SecretKind,
    ) -> SecretReference {
        vault_secret_reference(secret_ref, secret_kind, self.store_kind())
    }
}

pub(super) struct SecretWriteContext<'a> {
    pub(super) entry_kind: &'a str,
    pub(super) account_id: &'a str,
    pub(super) purpose: &'a str,
    pub(super) secret_kind: SecretKind,
    pub(super) label: &'a str,
    pub(super) metadata: &'a Value,
}

impl SecretResolver for AccountSecretVault {
    fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a> {
        match self {
            Self::Database(vault) => vault.resolve(reference),
            Self::Host(vault) => vault.resolve(reference),
        }
    }
}
```

### `backend/src/integrations/mail/gmail/client.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/client.rs`
- Size bytes / Размер в байтах: `269`
- Included characters / Включено символов: `269`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod gmail_api;
mod helpers;
mod imap;
mod models;
mod options;

pub use errors::EmailProviderNetworkError;
pub use gmail_api::GmailApiClient;
pub use imap::ImapNetworkClient;
pub use options::{GmailFetchOptions, GmailHistoryFetchOptions, ImapFetchOptions};
```

### `backend/src/integrations/mail/gmail/client/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/client/errors.rs`
- Size bytes / Размер в байтах: `1016`
- Included characters / Включено символов: `1016`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailProviderNetworkError {
    #[error("invalid provider request field {field}: {message}")]
    InvalidProviderRequest {
        field: &'static str,
        message: &'static str,
    },

    #[error("invalid provider response field {field}: {message}")]
    InvalidProviderResponse {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider response is missing required field: {field}")]
    MissingProviderField { field: &'static str },

    #[error("unexpected provider response: {message}")]
    UnexpectedProviderResponse { message: &'static str },

    #[error("provider operation timed out: {operation}")]
    ProviderTimeout { operation: &'static str },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tls(#[from] async_native_tls::Error),

    #[error(transparent)]
    Imap(#[from] async_imap::error::Error),
}
```

### `backend/src/integrations/mail/gmail/client/gmail_api.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/client/gmail_api.rs`
- Size bytes / Размер в байтах: `10740`
- Included characters / Включено символов: `10740`
- Truncated / Обрезано: `no`

```rust
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::json;

use crate::integrations::mail::send::{OutgoingEmail, SendResult, build_rfc2822_message};
use crate::integrations::mail::sync::{EmailSyncBatch, FetchedCommunicationSourceMessage};
use crate::platform::communications::EmailProviderKind;
use crate::platform::secrets::ResolvedSecret;

use super::errors::EmailProviderNetworkError;
use super::helpers::{
    gmail_history_checkpoint, gmail_message_list_checkpoint, parse_gmail_internal_date,
    select_latest_history_id, sha256_fingerprint, trim_base_url, validate_non_empty,
};
use super::models::{GmailHistoryResponse, GmailListResponse, GmailRawMessage, GmailSendResponse};
use super::options::{GmailFetchOptions, GmailHistoryFetchOptions};

#[derive(Clone)]
pub struct GmailApiClient {
    http: reqwest::Client,
    base_url: String,
    user_id: String,
}

impl GmailApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client configuration must be valid");

        Self {
            http,
            base_url: trim_base_url(base_url.into()),
            user_id: "me".to_owned(),
        }
    }

    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = user_id.into();
        self
    }

    pub async fn fetch_raw_messages(
        &self,
        access_token: &ResolvedSecret,
        options: &GmailFetchOptions,
    ) -> Result<EmailSyncBatch, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("user_id", &self.user_id)?;
        options.validate()?;

        let list_url = format!("{}/gmail/v1/users/{}/messages", self.base_url, self.user_id);
        let mut query = vec![
            ("maxResults", options.max_results.to_string()),
            ("includeSpamTrash", options.include_spam_trash.to_string()),
        ];
        if let Some(page_token) = &options.page_token {
            query.push(("pageToken", page_token.clone()));
        }
        if let Some(search_query) = &options.query {
            query.push(("q", search_query.clone()));
        }
        for label_id in &options.label_ids {
            query.push(("labelIds", label_id.clone()));
        }

        let list_response = self
            .http
            .get(list_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<GmailListResponse>()
            .await?;

        let mut messages = Vec::new();
        let mut latest_history_id = None;
        for listed_message in list_response.messages.unwrap_or_default() {
            validate_non_empty("gmail_message_id", &listed_message.id)?;
            let message_url = format!(
                "{}/gmail/v1/users/{}/messages/{}",
                self.base_url, self.user_id, listed_message.id
            );
            let raw_message = self
                .http
                .get(message_url)
                .bearer_auth(access_token.expose_for_runtime())
                .query(&[("format", "raw")])
                .send()
                .await?
                .error_for_status()?
                .json::<GmailRawMessage>()
                .await?;

            let provider_record_id = raw_message.id.unwrap_or(listed_message.id);
            let raw = raw_message
                .raw
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "raw" })?;
            let occurred_at = parse_gmail_internal_date(raw_message.internal_date.as_deref())?;
            latest_history_id =
                select_latest_history_id(latest_history_id, raw_message.history_id.as_deref());

            messages.push(FetchedCommunicationSourceMessage {
                source_fingerprint: sha256_fingerprint([
                    "gmail".as_bytes(),
                    provider_record_id.as_bytes(),
                    raw.as_bytes(),
                ]),
                provider_record_id: provider_record_id.clone(),
                occurred_at,
                payload: json!({
                    "provider": "gmail",
                    "id": provider_record_id,
                    "thread_id": raw_message.thread_id.or(listed_message.thread_id),
                    "label_ids": raw_message.label_ids,
                    "history_id": raw_message.history_id,
                    "internal_date": raw_message.internal_date,
                    "raw_base64url": raw
                }),
            });
        }

        let checkpoint =
            gmail_message_list_checkpoint(latest_history_id, list_response.next_page_token);

        Ok(EmailSyncBatch {
            provider_kind: EmailProviderKind::Gmail,
            stream_id: "gmail:history".to_owned(),
            checkpoint,
            messages,
        })
    }

    pub async fn fetch_history_raw_messages(
        &self,
        access_token: &ResolvedSecret,
        options: &GmailHistoryFetchOptions,
    ) -> Result<EmailSyncBatch, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("user_id", &self.user_id)?;
        options.validate()?;

        let history_url = format!("{}/gmail/v1/users/{}/history", self.base_url, self.user_id);
        let mut query = vec![
            ("startHistoryId", options.start_history_id.clone()),
            ("maxResults", options.max_results.to_string()),
            ("historyTypes", "messageAdded".to_owned()),
        ];
        if let Some(page_token) = &options.page_token {
            query.push(("pageToken", page_token.clone()));
        }

        let history_response = self
            .http
            .get(history_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<GmailHistoryResponse>()
            .await?;

        let mut message_ids = Vec::new();
        for history in history_response.history.unwrap_or_default() {
            for added in history.messages_added.unwrap_or_default() {
                if !message_ids.contains(&added.message.id) {
                    message_ids.push(added.message.id);
                }
            }
        }

        let mut messages = Vec::new();
        let mut latest_history_id = history_response.history_id.clone();
        for message_id in message_ids.into_iter().take(options.max_results as usize) {
            let raw_message = self.fetch_raw_message(access_token, &message_id).await?;
            let provider_record_id = raw_message.id.unwrap_or(message_id);
            let raw = raw_message
                .raw
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "raw" })?;
            let occurred_at = parse_gmail_internal_date(raw_message.internal_date.as_deref())?;
            latest_history_id =
                select_latest_history_id(latest_history_id, raw_message.history_id.as_deref());

            messages.push(FetchedCommunicationSourceMessage {
                source_fingerprint: sha256_fingerprint([
                    "gmail".as_bytes(),
                    provider_record_id.as_bytes(),
                    raw.as_bytes(),
                ]),
                provider_record_id: provider_record_id.clone(),
                occurred_at,
                payload: json!({
                    "provider": "gmail",
                    "id": provider_record_id,
                    "thread_id": raw_message.thread_id,
                    "label_ids": raw_message.label_ids,
                    "history_id": raw_message.history_id,
                    "internal_date": raw_message.internal_date,
                    "raw_base64url": raw
                }),
            });
        }

        let checkpoint = gmail_history_checkpoint(
            &options.start_history_id,
            latest_history_id,
            history_response.next_page_token,
        );

        Ok(EmailSyncBatch {
            provider_kind: EmailProviderKind::Gmail,
            stream_id: "gmail:history".to_owned(),
            checkpoint,
            messages,
        })
    }

    pub async fn send_message(
        &self,
        access_token: &ResolvedSecret,
        email: &OutgoingEmail,
    ) -> Result<SendResult, EmailProviderNetworkError> {
        validate_non_empty("base_url", &self.base_url)?;
        validate_non_empty("user_id", &self.user_id)?;
        if email
            .to
            .iter()
            .chain(email.cc.iter())
            .chain(email.bcc.iter())
            .all(|recipient| recipient.trim().is_empty())
        {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "recipients",
                message: "at least one recipient is required",
            });
        }

        let raw = URL_SAFE_NO_PAD.encode(build_rfc2822_message(email).as_bytes());
        let send_url = format!(
            "{}/gmail/v1/users/{}/messages/send",
            self.base_url, self.user_id
        );
        let response = self
            .http
            .post(send_url)
            .bearer_auth(access_token.expose_for_runtime())
            .json(&json!({ "raw": raw }))
            .send()
            .await?
            .error_for_status()?
            .json::<GmailSendResponse>()
            .await?;
        let message_id = response
            .id
            .ok_or(EmailProviderNetworkError::MissingProviderField { field: "id" })?;

        Ok(SendResult {
            message_id,
            accepted_recipients: email
                .to
                .iter()
                .chain(email.cc.iter())
                .chain(email.bcc.iter())
                .cloned()
                .collect(),
        })
    }

    async fn fetch_raw_message(
        &self,
        access_token: &ResolvedSecret,
        message_id: &str,
    ) -> Result<GmailRawMessage, EmailProviderNetworkError> {
        validate_non_empty("gmail_message_id", message_id)?;
        let message_url = format!(
            "{}/gmail/v1/users/{}/messages/{}",
            self.base_url, self.user_id, message_id
        );

        Ok(self
            .http
            .get(message_url)
            .bearer_auth(access_token.expose_for_runtime())
            .query(&[("format", "raw")])
            .send()
            .await?
            .error_for_status()?
            .json::<GmailRawMessage>()
            .await?)
    }
}
```

### `backend/src/integrations/mail/gmail/client/helpers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/client/helpers.rs`
- Size bytes / Размер в байтах: `7187`
- Included characters / Включено символов: `7187`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::errors::EmailProviderNetworkError;

pub(super) fn trim_base_url(base_url: String) -> String {
    base_url.trim().trim_end_matches('/').to_owned()
}

pub(super) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), EmailProviderNetworkError> {
    if value.trim().is_empty() {
        return Err(EmailProviderNetworkError::InvalidProviderRequest {
            field,
            message: "must not be empty",
        });
    }

    Ok(())
}

pub(super) fn parse_gmail_internal_date(
    internal_date: Option<&str>,
) -> Result<Option<DateTime<Utc>>, EmailProviderNetworkError> {
    let Some(internal_date) = internal_date else {
        return Ok(None);
    };
    let millis = internal_date.parse::<i64>().map_err(|_| {
        EmailProviderNetworkError::InvalidProviderResponse {
            field: "internal_date",
            message: "expected epoch milliseconds",
        }
    })?;

    Utc.timestamp_millis_opt(millis)
        .single()
        .ok_or(EmailProviderNetworkError::InvalidProviderResponse {
            field: "internal_date",
            message: "timestamp is out of range",
        })
        .map(Some)
}

pub(super) fn select_latest_history_id(
    current: Option<String>,
    candidate: Option<&str>,
) -> Option<String> {
    let Some(candidate) = candidate else {
        return current;
    };
    let Some(current) = current else {
        return Some(candidate.to_owned());
    };

    let current_number = current.parse::<u64>();
    let candidate_number = candidate.parse::<u64>();
    match (current_number, candidate_number) {
        (Ok(current_number), Ok(candidate_number)) if current_number >= candidate_number => {
            Some(current)
        }
        _ => Some(candidate.to_owned()),
    }
}

pub(super) fn gmail_message_list_checkpoint(
    history_id: Option<String>,
    next_page_token: Option<String>,
) -> Option<Value> {
    gmail_checkpoint(history_id, next_page_token, Some("messages"), None)
}

pub(super) fn gmail_history_checkpoint(
    start_history_id: &str,
    history_id: Option<String>,
    next_page_token: Option<String>,
) -> Option<Value> {
    gmail_checkpoint(
        history_id,
        next_page_token,
        Some("history"),
        Some(start_history_id),
    )
}

fn gmail_checkpoint(
    history_id: Option<String>,
    next_page_token: Option<String>,
    page_kind: Option<&'static str>,
    start_history_id: Option<&str>,
) -> Option<Value> {
    let history_id = history_id?;
    let mut checkpoint = json!({
        "provider": "gmail",
        "history_id": history_id
    });

    if let Some(next_page_token) = next_page_token {
        checkpoint["next_page_token"] = json!(next_page_token);
        if let Some(page_kind) = page_kind {
            checkpoint["page_kind"] = json!(page_kind);
        }
        if let Some(start_history_id) = start_history_id {
            checkpoint["start_history_id"] = json!(start_history_id);
        }
    }

    Some(checkpoint)
}

pub(super) fn imap_checkpoint(
    mailbox: &str,
    uid_validity: Option<u32>,
    latest_uid: Option<u32>,
) -> Value {
    let mut checkpoint = json!({
        "provider": "imap",
        "mailbox": mailbox
    });

    if let Some(uid_validity) = uid_validity {
        checkpoint["uid_validity"] = json!(uid_validity);
    }
    if let Some(latest_uid) = latest_uid {
        checkpoint["last_seen_uid"] = json!(latest_uid);
    }

    checkpoint
}

pub(super) fn next_imap_uid_floor(last_seen_uid: Option<u32>) -> Option<u32> {
    match last_seen_uid {
        Some(uid) => uid.checked_add(1),
        None => Some(1),
    }
}

pub(super) fn imap_uid_search_query(first_uid: u32) -> String {
    format!("UID {first_uid}:*")
}

pub(super) fn retain_uids_from_floor(uids: Vec<u32>, first_uid: u32) -> Vec<u32> {
    uids.into_iter().filter(|uid| *uid >= first_uid).collect()
}

pub(super) fn uid_set(uids: &[u32]) -> String {
    uids.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn select_uids_for_fetch(
    mut uids: Vec<u32>,
    max_messages: usize,
    latest_messages: bool,
) -> Vec<u32> {
    uids.sort_unstable();
    if latest_messages && uids.len() > max_messages {
        uids[uids.len() - max_messages..].to_vec()
    } else {
        uids.truncate(max_messages);
        uids
    }
}

pub(super) fn sha256_fingerprint<'a>(parts: impl IntoIterator<Item = &'a [u8]>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }

    format!("sha256:{}", hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        gmail_history_checkpoint, imap_uid_search_query, next_imap_uid_floor,
        retain_uids_from_floor, select_uids_for_fetch,
    };

    #[test]
    fn select_uids_for_fetch_keeps_latest_window_when_requested() {
        assert_eq!(
            select_uids_for_fetch(vec![43, 41, 42], 2, true),
            vec![42, 43]
        );
    }

    #[test]
    fn select_uids_for_fetch_keeps_oldest_window_for_sync_default() {
        assert_eq!(
            select_uids_for_fetch(vec![43, 41, 42], 2, false),
            vec![41, 42]
        );
    }

    #[test]
    fn imap_uid_search_uses_uid_criterion_after_checkpoint() {
        let first_uid = next_imap_uid_floor(Some(30144)).expect("next UID");

        assert_eq!(first_uid, 30145);
        assert_eq!(imap_uid_search_query(first_uid), "UID 30145:*");
    }

    #[test]
    fn imap_uid_search_starts_at_first_uid_without_checkpoint() {
        let first_uid = next_imap_uid_floor(None).expect("first UID");

        assert_eq!(first_uid, 1);
        assert_eq!(imap_uid_search_query(first_uid), "UID 1:*");
    }

    #[test]
    fn imap_uid_search_discards_star_wraparound_uid() {
        assert_eq!(
            retain_uids_from_floor(vec![30144], 30145),
            Vec::<u32>::new()
        );
        assert_eq!(
            retain_uids_from_floor(vec![30144, 30145, 30146], 30145),
            vec![30145, 30146]
        );
    }

    #[test]
    fn imap_uid_floor_stops_at_u32_max() {
        assert_eq!(next_imap_uid_floor(Some(u32::MAX)), None);
    }

    #[test]
    fn gmail_history_checkpoint_preserves_pagination_origin() {
        assert_eq!(
            gmail_history_checkpoint(
                "history-start",
                Some("history-latest".to_owned()),
                Some("history-next".to_owned()),
            ),
            Some(json!({
                "provider": "gmail",
                "history_id": "history-latest",
                "next_page_token": "history-next",
                "page_kind": "history",
                "start_history_id": "history-start"
            }))
        );
    }
}
```

### `backend/src/integrations/mail/gmail/client/imap.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/client/imap.rs`
- Size bytes / Размер в байтах: `5713`
- Included characters / Включено символов: `5713`
- Truncated / Обрезано: `no`

```rust
use std::fmt::Debug;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use futures::TryStreamExt;
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::integrations::mail::sync::{
    EmailSyncBatch, FetchedCommunicationSourceMessage, imap_mailbox_stream_id,
};
use crate::platform::secrets::ResolvedSecret;

use super::errors::EmailProviderNetworkError;
use super::helpers::{
    imap_checkpoint, imap_uid_search_query, next_imap_uid_floor, retain_uids_from_floor,
    select_uids_for_fetch, sha256_fingerprint, uid_set,
};
use super::options::ImapFetchOptions;

const IMAP_UID_FETCH_CHUNK_SIZE: usize = 10;
const IMAP_UID_FETCH_TIMEOUT_SECONDS: u64 = 60;

#[derive(Clone, Debug, Default)]
pub struct ImapNetworkClient;

impl ImapNetworkClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch_raw_messages(
        &self,
        password: &ResolvedSecret,
        options: &ImapFetchOptions,
    ) -> Result<EmailSyncBatch, EmailProviderNetworkError> {
        options.validate()?;

        let address = (options.host.as_str(), options.port);
        let tcp_stream = tokio::net::TcpStream::connect(address).await?;
        if options.tls {
            let tls_stream = async_native_tls::connect(options.host.as_str(), tcp_stream).await?;
            fetch_imap_with_client(async_imap::Client::new(tls_stream), password, options).await
        } else {
            fetch_imap_with_client(async_imap::Client::new(tcp_stream), password, options).await
        }
    }
}

async fn fetch_imap_with_client<T>(
    mut client: async_imap::Client<T>,
    password: &ResolvedSecret,
    options: &ImapFetchOptions,
) -> Result<EmailSyncBatch, EmailProviderNetworkError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    client
        .read_response()
        .await?
        .ok_or(EmailProviderNetworkError::UnexpectedProviderResponse {
            message: "missing IMAP greeting",
        })?;

    let mut session = client
        .login(&options.username, password.expose_for_runtime())
        .await
        .map_err(|(error, _client)| EmailProviderNetworkError::Imap(error))?;
    let mailbox = session.examine(&options.mailbox).await?;
    let requested_uid_floor = next_imap_uid_floor(options.last_seen_uid);
    let uids = match requested_uid_floor {
        Some(first_uid) => {
            let uids: Vec<u32> = session
                .uid_search(imap_uid_search_query(first_uid))
                .await?
                .into_iter()
                .collect();
            let uids = retain_uids_from_floor(uids, first_uid);
            select_uids_for_fetch(uids, options.max_messages, options.latest_messages)
        }
        None => Vec::new(),
    };

    let messages = fetch_imap_uid_chunks(&mut session, &mailbox, options, &uids).await?;
    let latest_uid = messages
        .iter()
        .filter_map(|message| message.provider_record_id.parse::<u32>().ok())
        .max()
        .or(options.last_seen_uid);
    session.logout().await?;

    Ok(EmailSyncBatch {
        provider_kind: options.provider_kind,
        stream_id: imap_mailbox_stream_id(&options.mailbox),
        checkpoint: Some(imap_checkpoint(
            &options.mailbox,
            mailbox.uid_validity,
            latest_uid,
        )),
        messages,
    })
}

async fn fetch_imap_uid_chunks<T>(
    session: &mut async_imap::Session<T>,
    mailbox: &async_imap::types::Mailbox,
    options: &ImapFetchOptions,
    uids: &[u32],
) -> Result<Vec<FetchedCommunicationSourceMessage>, EmailProviderNetworkError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    let mut messages = Vec::new();
    for chunk in uids.chunks(IMAP_UID_FETCH_CHUNK_SIZE) {
        let uid_set = uid_set(chunk);
        let fetched_messages =
            tokio::time::timeout(Duration::from_secs(IMAP_UID_FETCH_TIMEOUT_SECONDS), async {
                session
                    .uid_fetch(uid_set, "(UID BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
                    .await?
                    .try_collect::<Vec<_>>()
                    .await
            })
            .await
            .map_err(|_| EmailProviderNetworkError::ProviderTimeout {
                operation: "imap_uid_fetch",
            })??;

        for fetched_message in fetched_messages {
            let uid = fetched_message
                .uid
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "uid" })?;
            let body = fetched_message
                .body()
                .ok_or(EmailProviderNetworkError::MissingProviderField { field: "rfc822" })?;
            let uid_string = uid.to_string();
            let occurred_at = fetched_message
                .internal_date()
                .map(|internal_date| internal_date.with_timezone(&Utc));

            messages.push(FetchedCommunicationSourceMessage {
                provider_record_id: uid_string.clone(),
                source_fingerprint: sha256_fingerprint([
                    "imap".as_bytes(),
                    uid_string.as_bytes(),
                    body,
                ]),
                occurred_at,
                payload: json!({
                    "provider": options.provider_kind.as_str(),
                    "transport": "imap",
                    "mailbox": options.mailbox,
                    "uid": uid,
                    "uid_validity": mailbox.uid_validity,
                    "raw_rfc822_base64": BASE64_STANDARD.encode(body),
                    "rfc822_size": fetched_message.size
                }),
            });
        }
    }

    Ok(messages)
}
```

### `backend/src/integrations/mail/gmail/client/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/client/models.rs`
- Size bytes / Размер в байтах: `1594`
- Included characters / Включено символов: `1594`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailListResponse {
    pub(super) messages: Option<Vec<GmailListedMessage>>,
    pub(super) next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailListedMessage {
    pub(super) id: String,
    pub(super) thread_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailRawMessage {
    pub(super) id: Option<String>,
    pub(super) thread_id: Option<String>,
    pub(super) label_ids: Option<Vec<String>>,
    pub(super) history_id: Option<String>,
    pub(super) internal_date: Option<String>,
    pub(super) raw: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GmailSendResponse {
    pub(super) id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryResponse {
    pub(super) history: Option<Vec<GmailHistoryItem>>,
    pub(super) history_id: Option<String>,
    pub(super) next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryItem {
    pub(super) messages_added: Option<Vec<GmailHistoryMessageAdded>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryMessageAdded {
    pub(super) message: GmailHistoryMessage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GmailHistoryMessage {
    pub(super) id: String,
}
```

### `backend/src/integrations/mail/gmail/client/options.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/client/options.rs`
- Size bytes / Размер в байтах: `5112`
- Included characters / Включено символов: `5112`
- Truncated / Обрезано: `no`

```rust
use crate::platform::communications::EmailProviderKind;

use super::errors::EmailProviderNetworkError;
use super::helpers::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailFetchOptions {
    pub(super) max_results: u16,
    pub(super) query: Option<String>,
    pub(super) page_token: Option<String>,
    pub(super) label_ids: Vec<String>,
    pub(super) include_spam_trash: bool,
}

impl GmailFetchOptions {
    pub fn new(max_results: u16) -> Self {
        Self {
            max_results,
            query: None,
            page_token: None,
            label_ids: Vec::new(),
            include_spam_trash: true,
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn page_token(mut self, page_token: impl Into<String>) -> Self {
        self.page_token = Some(page_token.into());
        self
    }

    pub fn label_id(mut self, label_id: impl Into<String>) -> Self {
        self.label_ids.push(label_id.into());
        self
    }

    pub fn include_spam_trash(mut self, include_spam_trash: bool) -> Self {
        self.include_spam_trash = include_spam_trash;
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        if self.max_results == 0 || self.max_results > 500 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_results",
                message: "must be between 1 and 500",
            });
        }
        if let Some(query) = &self.query {
            validate_non_empty("query", query)?;
        }
        if let Some(page_token) = &self.page_token {
            validate_non_empty("page_token", page_token)?;
        }
        for label_id in &self.label_ids {
            validate_non_empty("label_id", label_id)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailHistoryFetchOptions {
    pub(super) start_history_id: String,
    pub(super) max_results: u16,
    pub(super) page_token: Option<String>,
}

impl GmailHistoryFetchOptions {
    pub fn new(start_history_id: impl Into<String>, max_results: u16) -> Self {
        Self {
            start_history_id: start_history_id.into(),
            max_results,
            page_token: None,
        }
    }

    pub fn page_token(mut self, page_token: impl Into<String>) -> Self {
        self.page_token = Some(page_token.into());
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("start_history_id", &self.start_history_id)?;
        if self.max_results == 0 || self.max_results > 500 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_results",
                message: "must be between 1 and 500",
            });
        }
        if let Some(page_token) = &self.page_token {
            validate_non_empty("page_token", page_token)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapFetchOptions {
    pub provider_kind: EmailProviderKind,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub last_seen_uid: Option<u32>,
    pub max_messages: usize,
    pub latest_messages: bool,
}

impl ImapFetchOptions {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        tls: bool,
        mailbox: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            provider_kind: EmailProviderKind::Imap,
            host: host.into(),
            port,
            tls,
            mailbox: mailbox.into(),
            username: username.into(),
            last_seen_uid: None,
            max_messages: 100,
            latest_messages: false,
        }
    }

    pub fn provider_kind(mut self, provider_kind: EmailProviderKind) -> Self {
        self.provider_kind = provider_kind;
        self
    }

    pub fn last_seen_uid(mut self, last_seen_uid: u32) -> Self {
        self.last_seen_uid = Some(last_seen_uid);
        self
    }

    pub fn max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = max_messages;
        self
    }

    pub fn latest_messages(mut self) -> Self {
        self.latest_messages = true;
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("host", &self.host)?;
        validate_non_empty("mailbox", &self.mailbox)?;
        validate_non_empty("username", &self.username)?;
        if self.port == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }
        if self.max_messages == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_messages",
                message: "must be greater than zero",
            });
        }

        Ok(())
    }
}
```

### `backend/src/integrations/mail/gmail/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/gmail/mod.rs`
- Size bytes / Размер в байтах: `16`
- Included characters / Включено символов: `16`
- Truncated / Обрезано: `no`

```rust
pub mod client;
```

### `backend/src/integrations/mail/imap_write.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/imap_write.rs`
- Size bytes / Размер в байтах: `3236`
- Included characters / Включено символов: `3236`
- Truncated / Обрезано: `no`

```rust
use crate::platform::secrets::ResolvedSecret;
use futures::TryStreamExt;
use std::fmt::Debug;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

pub struct ImapWriteClient;

#[derive(Clone, Copy, Debug)]
pub struct ImapWriteConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub tls: bool,
    pub username: &'a str,
    pub password: &'a ResolvedSecret,
    pub mailbox: &'a str,
}

impl Default for ImapWriteClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ImapWriteClient {
    pub fn new() -> Self {
        Self
    }
    fn uid_set(uids: &[u32]) -> String {
        uids.iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    pub async fn mark_seen(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
    ) -> Result<(), ImapWriteError> {
        with_imap_session(config, |mut s| async move {
            s.uid_store(Self::uid_set(uids), "+FLAGS (\\Seen)")
                .await?
                .try_collect::<Vec<_>>()
                .await?;
            Ok(())
        })
        .await
    }

    pub async fn delete_messages(
        &self,
        config: &ImapWriteConfig<'_>,
        uids: &[u32],
    ) -> Result<(), ImapWriteError> {
        with_imap_session(config, |mut s| async move {
            s.uid_store(Self::uid_set(uids), "+FLAGS (\\Deleted)")
                .await?
                .try_collect::<Vec<_>>()
                .await?;
            s.expunge().await?.try_collect::<Vec<_>>().await?;
            Ok(())
        })
        .await
    }
}

type ImapSession = async_imap::Session<Box<dyn AsyncReadWrite>>;
trait AsyncReadWrite: AsyncRead + AsyncWrite + Unpin + Send + Debug {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send + Debug> AsyncReadWrite for T {}

async fn with_imap_session<F, Fut>(config: &ImapWriteConfig<'_>, f: F) -> Result<(), ImapWriteError>
where
    F: FnOnce(ImapSession) -> Fut,
    Fut: std::future::Future<Output = Result<(), async_imap::error::Error>>,
{
    let address = (config.host, config.port);
    let tcp_stream = tokio::net::TcpStream::connect(address).await?;
    let session: ImapSession = if config.tls {
        let tls_stream = async_native_tls::connect(config.host, tcp_stream).await?;
        let client = async_imap::Client::new(Box::new(tls_stream) as Box<dyn AsyncReadWrite>);
        let mut s = client
            .login(config.username, config.password.expose_for_runtime())
            .await
            .map_err(|(e, _)| ImapWriteError::Imap(e))?;
        s.select(config.mailbox).await?;
        s
    } else {
        let client = async_imap::Client::new(Box::new(tcp_stream) as Box<dyn AsyncReadWrite>);
        let mut s = client
            .login(config.username, config.password.expose_for_runtime())
            .await
            .map_err(|(e, _)| ImapWriteError::Imap(e))?;
        s.select(config.mailbox).await?;
        s
    };
    f(session).await?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum ImapWriteError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tls(#[from] async_native_tls::Error),
    #[error(transparent)]
    Imap(#[from] async_imap::error::Error),
}
```

### `backend/src/integrations/mail/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/mod.rs`
- Size bytes / Размер в байтах: `136`
- Included characters / Включено символов: `136`
- Truncated / Обрезано: `no`

```rust
pub mod accounts;
pub mod gmail;
pub mod imap_write;
pub mod outbox;
pub mod rfc822;
pub mod send;
pub mod sync;
pub mod sync_provider;
```

### `backend/src/integrations/mail/outbox.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/outbox.rs`
- Size bytes / Размер в байтах: `1892`
- Included characters / Включено символов: `1892`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;

use sqlx::postgres::PgPool;

use crate::integrations::mail::accounts::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::GmailApiClient;
use crate::platform::communications::{
    EmailSendError, GmailOutboxSendRequest, GmailOutboxTransport, SendResult,
};
use crate::platform::secrets::SecretReferenceStore;
use crate::vault::HostVault;

#[derive(Clone)]
pub struct LiveGmailOutboxTransport {
    pool: PgPool,
    secret_store: SecretReferenceStore,
    vault: HostVault,
}

impl LiveGmailOutboxTransport {
    pub fn new(pool: PgPool, vault: HostVault) -> Self {
        Self {
            pool: pool.clone(),
            secret_store: SecretReferenceStore::new(pool),
            vault,
        }
    }
}

impl GmailOutboxTransport for LiveGmailOutboxTransport {
    fn send<'a>(
        &'a self,
        request: GmailOutboxSendRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>> {
        Box::pin(async move {
            let account_setup = EmailAccountSetupService::new_with_host_vault_for_token_refresh(
                self.pool.clone(),
                self.secret_store.clone(),
                self.vault.clone(),
            );
            let access_token = account_setup
                .refresh_gmail_access_token(request.oauth_secret_ref)
                .await
                .map_err(|error| EmailSendError::Provider(error.to_string()))?;

            GmailApiClient::new(request.api_base_url)
                .user_id("me")
                .send_message(&access_token, request.email)
                .await
                .map_err(|error| EmailSendError::Provider(error.to_string()))
        })
    }
}

pub(crate) fn gmail_outbox_transport(pool: PgPool, vault: HostVault) -> impl GmailOutboxTransport {
    LiveGmailOutboxTransport::new(pool, vault)
}
```

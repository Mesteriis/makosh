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

- Chunk ID / ID чанка: `059-source-backend-part-039`
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

### `backend/src/integrations/mail/rfc822.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/rfc822.rs`
- Size bytes / Размер в байтах: `198`
- Included characters / Включено символов: `198`
- Truncated / Обрезано: `no`

```rust
pub use crate::platform::communications::rfc822::{
    EmailRfc822ParseError, ParsedCommunicationSourceMessage, ParsedEmailAttachment,
    ParsedEmailAttachmentDisposition, parse_rfc822_message,
};
```

### `backend/src/integrations/mail/send.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/send.rs`
- Size bytes / Размер в байтах: `10245`
- Included characters / Включено символов: `10245`
- Truncated / Обрезано: `no`

```rust
use async_native_tls::TlsConnector;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use crate::platform::secrets::ResolvedSecret;

pub use crate::platform::communications::{
    EmailSendError, OutgoingEmail, SendResult, SmtpConfig, SmtpTransport,
};

#[derive(Clone, Default)]
pub struct LiveSmtpTransport;

impl SmtpTransport for LiveSmtpTransport {
    fn send<'a>(
        &'a self,
        config: &'a SmtpConfig,
        password: &'a ResolvedSecret,
        email: &'a OutgoingEmail,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>,
    > {
        Box::pin(async move { SmtpClient::new().send(config, password, email).await })
    }
}

pub(crate) fn smtp_outbox_transport() -> impl SmtpTransport {
    LiveSmtpTransport
}

pub struct SmtpClient;

impl Default for SmtpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SmtpClient {
    pub fn new() -> Self {
        Self
    }
    pub async fn send(
        &self,
        config: &SmtpConfig,
        password: &ResolvedSecret,
        email: &OutgoingEmail,
    ) -> Result<SendResult, EmailSendError> {
        let address = (config.host.as_str(), config.port);
        let tcp_stream = tokio::net::TcpStream::connect(address).await?;
        if config.starttls {
            starttls_smtp(tcp_stream, config, password, email).await
        } else if config.tls {
            let tls = TlsConnector::new();
            let tls_stream = tls.connect(&config.host, tcp_stream).await?;
            send_smtp(tls_stream, config, password, email).await
        } else {
            send_smtp(tcp_stream, config, password, email).await
        }
    }
}

async fn send_smtp<T: AsyncRead + AsyncWrite + Unpin>(
    stream: T,
    config: &SmtpConfig,
    password: &ResolvedSecret,
    email: &OutgoingEmail,
) -> Result<SendResult, EmailSendError> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    read_line(&mut reader, &mut buf).await?;
    send_smtp_after_greeting(reader, config, password, email).await
}

async fn starttls_smtp(
    stream: tokio::net::TcpStream,
    config: &SmtpConfig,
    password: &ResolvedSecret,
    email: &OutgoingEmail,
) -> Result<SendResult, EmailSendError> {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    let greeting = read_line(&mut reader, &mut buf).await?;
    if !greeting.starts_with("220") {
        return Err(EmailSendError::Protocol(greeting));
    }
    write_cmd(&mut reader, "EHLO makosh\r\n").await?;
    read_ehlo_response(&mut reader, &mut buf).await?;
    write_cmd(&mut reader, "STARTTLS\r\n").await?;
    let response = read_line(&mut reader, &mut buf).await?;
    if !response.starts_with("220") {
        return Err(EmailSendError::Protocol(response));
    }
    let tcp_stream = reader.into_inner();
    let tls = TlsConnector::new();
    let tls_stream = tls.connect(&config.host, tcp_stream).await?;
    send_smtp_after_greeting(BufReader::new(tls_stream), config, password, email).await
}

async fn send_smtp_after_greeting<T: AsyncRead + AsyncWrite + Unpin>(
    mut reader: BufReader<T>,
    config: &SmtpConfig,
    password: &ResolvedSecret,
    email: &OutgoingEmail,
) -> Result<SendResult, EmailSendError> {
    let mut buf = String::new();
    write_cmd(&mut reader, "EHLO makosh\r\n").await?;
    read_ehlo_response(&mut reader, &mut buf).await?;
    write_cmd(&mut reader, "AUTH LOGIN\r\n").await?;
    read_line(&mut reader, &mut buf).await?;
    write_cmd(
        &mut reader,
        &format!("{}\r\n", base64(config.username.as_bytes())),
    )
    .await?;
    read_line(&mut reader, &mut buf).await?;
    write_cmd(
        &mut reader,
        &format!("{}\r\n", base64(password.expose_for_runtime().as_bytes())),
    )
    .await?;
    read_line(&mut reader, &mut buf).await?;
    write_cmd(&mut reader, &format!("MAIL FROM:<{}>\r\n", email.from)).await?;
    read_line(&mut reader, &mut buf).await?;
    let mut accepted = Vec::new();
    for r in email
        .to
        .iter()
        .chain(email.cc.iter())
        .chain(email.bcc.iter())
    {
        write_cmd(&mut reader, &format!("RCPT TO:<{r}>\r\n")).await?;
        if read_line(&mut reader, &mut buf).await?.starts_with("250") {
            accepted.push(r.clone());
        }
    }
    write_cmd(&mut reader, "DATA\r\n").await?;
    read_line(&mut reader, &mut buf).await?;
    let msg = build_rfc2822_message(email);
    write_cmd(&mut reader, &format!("{msg}\r\n.\r\n")).await?;
    let resp = read_line(&mut reader, &mut buf).await?;
    let _ = write_cmd(&mut reader, "QUIT\r\n").await;
    Ok(SendResult {
        message_id: resp.split_whitespace().nth(1).unwrap_or("unknown").into(),
        accepted_recipients: accepted,
    })
}

async fn read_ehlo_response<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    buf: &mut String,
) -> Result<(), EmailSendError> {
    loop {
        let line = read_line(reader, buf).await?;
        if !line.starts_with("250-") {
            return Ok(());
        }
    }
}

async fn read_line<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    buf: &mut String,
) -> Result<String, EmailSendError> {
    buf.clear();
    reader.read_line(buf).await?;
    Ok(buf.trim().to_owned())
}

async fn write_cmd<W: AsyncWrite + Unpin>(
    writer: &mut W,
    data: &str,
) -> Result<(), EmailSendError> {
    writer.write_all(data.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

pub fn build_rfc2822_message(email: &OutgoingEmail) -> String {
    let now = chrono::Utc::now().to_rfc2822();
    let mut message = format!("From: {}\r\nTo: {}\r\n", email.from, email.to.join(", "));
    if !email.cc.is_empty() {
        message.push_str(&format!("Cc: {}\r\n", email.cc.join(", ")));
    }
    if let Some(ref r) = email.in_reply_to {
        message.push_str(&format!("In-Reply-To: {r}\r\n"));
    }
    if !email.references.is_empty() {
        message.push_str(&format!("References: {}\r\n", email.references.join(" ")));
    }
    message.push_str(&format!(
        "Date: {now}\r\nSubject: {}\r\nMIME-Version: 1.0\r\n",
        email.subject
    ));

    match email
        .body_html
        .as_deref()
        .map(str::trim)
        .filter(|body_html| !body_html.is_empty())
    {
        Some(body_html) => {
            let boundary = multipart_alternative_boundary(email);
            message.push_str(&format!(
                "Content-Type: multipart/alternative; boundary=\"{boundary}\"\r\n\r\n"
            ));
            message.push_str(&format!(
                "--{boundary}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{}\r\n",
                email.body_text
            ));
            message.push_str(&format!(
                "--{boundary}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n{body_html}\r\n"
            ));
            message.push_str(&format!("--{boundary}--"));
        }
        None => {
            message.push_str(&format!(
                "Content-Type: text/plain; charset=utf-8\r\n\r\n{}",
                email.body_text
            ));
        }
    }

    message
}

fn multipart_alternative_boundary(email: &OutgoingEmail) -> String {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    digest.update(email.from.as_bytes());
    digest.update(b"\0");
    digest.update(email.to.join(",").as_bytes());
    digest.update(b"\0");
    digest.update(email.subject.as_bytes());
    digest.update(b"\0");
    digest.update(email.body_text.as_bytes());
    digest.update(b"\0");
    if let Some(body_html) = &email.body_html {
        digest.update(body_html.as_bytes());
    }

    let digest = digest.finalize();
    let mut suffix = String::with_capacity(24);
    for byte in digest.iter().take(12) {
        suffix.push(hex_char(byte >> 4));
        suffix.push(hex_char(byte & 0x0f));
    }
    format!("makosh-alt-{suffix}")
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex nibble must fit in 0..=15"),
    }
}

fn base64(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outgoing_email(body_html: Option<String>) -> OutgoingEmail {
        OutgoingEmail {
            from: "sender@example.com".to_owned(),
            to: vec!["recipient@example.com".to_owned()],
            cc: vec!["copy@example.com".to_owned()],
            bcc: Vec::new(),
            subject: "Rich body".to_owned(),
            body_text: "Plain body".to_owned(),
            body_html,
            in_reply_to: Some("<parent@example.com>".to_owned()),
            references: vec!["<root@example.com>".to_owned()],
        }
    }

    #[test]
    fn rfc2822_builder_sends_plain_only_messages_as_text_plain() {
        let message = build_rfc2822_message(&outgoing_email(None));

        assert!(message.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(!message.contains("multipart/alternative"));
        assert!(message.ends_with("Plain body"));
    }

    #[test]
    fn rfc2822_builder_preserves_html_body_as_multipart_alternative() {
        let message = build_rfc2822_message(&outgoing_email(Some(
            "<p><strong>Rich body</strong></p>".into(),
        )));

        assert!(message.contains("MIME-Version: 1.0\r\n"));
        assert!(message.contains("Content-Type: multipart/alternative; boundary=\"makosh-alt-"));
        assert!(message.contains("Content-Type: text/plain; charset=utf-8\r\n"));
        assert!(message.contains("Content-Transfer-Encoding: 8bit\r\n\r\nPlain body\r\n"));
        assert!(message.contains("Content-Type: text/html; charset=utf-8\r\n"));
        assert!(message.contains(
            "Content-Transfer-Encoding: 8bit\r\n\r\n<p><strong>Rich body</strong></p>\r\n"
        ));
        assert!(message.contains("In-Reply-To: <parent@example.com>\r\n"));
        assert!(message.contains("References: <root@example.com>\r\n"));
    }
}
```

### `backend/src/integrations/mail/sync.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/sync.rs`
- Size bytes / Размер в байтах: `305`
- Included characters / Включено символов: `305`
- Truncated / Обрезано: `no`

```rust
mod errors;
mod models;
mod planning;

pub use errors::EmailSyncPlanError;
pub use models::{
    EmailSyncAdapterConfig, EmailSyncBatch, EmailSyncBlobImportReport, EmailSyncImportReport,
    EmailSyncPlan, FetchedCommunicationSourceMessage,
};
pub use planning::{imap_mailbox_stream_id, plan_email_sync};
```

### `backend/src/integrations/mail/sync/errors.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/sync/errors.rs`
- Size bytes / Размер в байтах: `61`
- Included characters / Включено символов: `61`
- Truncated / Обрезано: `no`

```rust
pub use crate::platform::communications::EmailSyncPlanError;
```

### `backend/src/integrations/mail/sync/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/sync/models.rs`
- Size bytes / Размер в байтах: `194`
- Included characters / Включено символов: `194`
- Truncated / Обрезано: `no`

```rust
pub use crate::platform::communications::{
    EmailSyncAdapterConfig, EmailSyncBatch, EmailSyncBlobImportReport, EmailSyncImportReport,
    EmailSyncPlan, FetchedCommunicationSourceMessage,
};
```

### `backend/src/integrations/mail/sync/planning.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/sync/planning.rs`
- Size bytes / Размер в байтах: `84`
- Included characters / Включено символов: `84`
- Truncated / Обрезано: `no`

```rust
pub use crate::platform::communications::{imap_mailbox_stream_id, plan_email_sync};
```

### `backend/src/integrations/mail/sync_provider.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mail/sync_provider.rs`
- Size bytes / Размер в байтах: `6676`
- Included characters / Включено символов: `6676`
- Truncated / Обрезано: `no`

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sqlx::postgres::PgPool;

use crate::integrations::mail::accounts::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::{
    EmailProviderNetworkError, GmailApiClient, GmailFetchOptions, GmailHistoryFetchOptions,
    ImapFetchOptions, ImapNetworkClient,
};
use crate::platform::communications::{
    EmailProviderSyncError, EmailProviderSyncPort, EmailSyncBatch, GmailHistoryFetchRequest,
    GmailMessageListFetchRequest, ImapMessageFetchRequest, ProviderAccountSecretPurpose,
    ProviderSecretBindingLookupPort,
};
use crate::platform::secrets::{ResolvedSecret, SecretReferenceStore, SecretResolver};
use crate::vault::HostVault;

#[derive(Clone)]
pub struct LiveEmailProviderSyncPort {
    pool: PgPool,
    vault: HostVault,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>,
    gmail_api_base_url: String,
}

impl LiveEmailProviderSyncPort {
    pub fn new(
        pool: PgPool,
        vault: HostVault,
        provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>,
        gmail_api_base_url: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            vault,
            provider_secret_binding_store,
            gmail_api_base_url: gmail_api_base_url.into(),
        }
    }

    async fn gmail_access_token(
        &self,
        account_id: &str,
    ) -> Result<ResolvedSecret, EmailProviderSyncError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(account_id, ProviderAccountSecretPurpose::OauthToken)
            .await
            .map_err(EmailProviderSyncError::credential)?
            .ok_or_else(EmailProviderSyncError::missing_credential)?;
        EmailAccountSetupService::new_with_host_vault_for_token_refresh(
            self.pool.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            self.vault.clone(),
        )
        .refresh_gmail_access_token(&binding.secret_ref)
        .await
        .map_err(EmailProviderSyncError::account_setup)
    }
}

async fn read_provider_secret(
    binding_store: &dyn ProviderSecretBindingLookupPort,
    secret_store: &SecretReferenceStore,
    resolver: &(impl SecretResolver + Sync + ?Sized),
    account_id: &str,
    secret_purpose: ProviderAccountSecretPurpose,
) -> Result<ResolvedSecret, EmailProviderSyncError> {
    let binding = binding_store
        .get_for_account(account_id, secret_purpose)
        .await
        .map_err(EmailProviderSyncError::credential)?
        .ok_or_else(EmailProviderSyncError::missing_credential)?;
    let reference = secret_store
        .secret_reference(&binding.secret_ref)
        .await
        .map_err(EmailProviderSyncError::credential)?
        .ok_or_else(EmailProviderSyncError::missing_credential)?;
    if !binding
        .secret_purpose
        .accepts_secret_kind(reference.secret_kind)
    {
        return Err(EmailProviderSyncError::credential(format!(
            "provider account secret kind is incompatible: secret_ref={}, secret_purpose={:?}, secret_kind={:?}",
            reference.secret_ref, binding.secret_purpose, reference.secret_kind
        )));
    }
    resolver
        .resolve(&reference)
        .await
        .map_err(EmailProviderSyncError::credential)
}

impl EmailProviderSyncPort for LiveEmailProviderSyncPort {
    fn fetch_gmail_message_list<'a>(
        &'a self,
        request: GmailMessageListFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>
    {
        Box::pin(async move {
            let access_token = self.gmail_access_token(&request.account_id).await?;
            let client = GmailApiClient::new(&self.gmail_api_base_url).user_id("me");
            let mut options = GmailFetchOptions::new(request.max_results);
            if let Some(token) = request.page_token {
                options = options.page_token(token);
            }
            client
                .fetch_raw_messages(&access_token, &options)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))
        })
    }

    fn fetch_gmail_history<'a>(
        &'a self,
        request: GmailHistoryFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>
    {
        Box::pin(async move {
            let access_token = self.gmail_access_token(&request.account_id).await?;
            let client = GmailApiClient::new(&self.gmail_api_base_url).user_id("me");
            let mut options =
                GmailHistoryFetchOptions::new(&request.start_history_id, request.max_results);
            if let Some(token) = request.page_token {
                options = options.page_token(token);
            }
            client
                .fetch_history_raw_messages(&access_token, &options)
                .await
                .map_err(|error| {
                    let history_expired = matches!(
                        &error,
                        EmailProviderNetworkError::Http(source)
                            if source.status().is_some_and(|status| status.as_u16() == 404)
                    );
                    EmailProviderSyncError::provider_network(error, history_expired)
                })
        })
    }

    fn fetch_imap_messages<'a>(
        &'a self,
        request: ImapMessageFetchRequest,
    ) -> Pin<Box<dyn Future<Output = Result<EmailSyncBatch, EmailProviderSyncError>> + Send + 'a>>
    {
        Box::pin(async move {
            let secret_store = SecretReferenceStore::new(self.pool.clone());
            let credential = read_provider_secret(
                self.provider_secret_binding_store.as_ref(),
                &secret_store,
                &self.vault,
                &request.account_id,
                ProviderAccountSecretPurpose::ImapPassword,
            )
            .await?;
            let mut options = ImapFetchOptions::new(
                &request.host,
                request.port,
                request.tls,
                &request.mailbox,
                &request.username,
            )
            .provider_kind(request.provider_kind)
            .max_messages(request.max_messages);
            if let Some(uid) = request.last_seen_uid {
                options = options.last_seen_uid(uid);
            }
            ImapNetworkClient::new()
                .fetch_raw_messages(&credential, &options)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))
        })
    }
}
```

### `backend/src/integrations/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/mod.rs`
- Size bytes / Размер в байтах: `144`
- Included characters / Включено символов: `144`
- Truncated / Обрезано: `no`

```rust
pub mod ai_runtime;
pub mod mail;
pub mod ollama;
pub mod omniroute;
pub mod telegram;
pub mod whatsapp;
pub mod yandex_telemost;
pub mod zoom;
```

### `backend/src/integrations/ollama/client.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client.rs`
- Size bytes / Размер в байтах: `1929`
- Included characters / Включено символов: `1929`
- Truncated / Обрезано: `no`

```rust
use std::time::Duration;

use reqwest::Url;

mod catalog;
mod chat;
mod config;
mod embeddings;
mod error;
mod models;
mod responses;
mod sanitization;
mod transport;

pub use config::OllamaClientConfig;
pub use error::OllamaError;
pub use models::{OllamaChatResult, OllamaEmbedResult};

#[derive(Clone)]
pub struct OllamaClient {
    pub(in crate::integrations::ollama::client) http: reqwest::Client,
    pub(in crate::integrations::ollama::client) base_url: Url,
    pub(in crate::integrations::ollama::client) chat_model: String,
    pub(in crate::integrations::ollama::client) embed_model: String,
}

impl OllamaClient {
    pub fn new(config: OllamaClientConfig) -> Result<Self, OllamaError> {
        if config.base_url.trim().is_empty() {
            return Err(OllamaError::InvalidConfig("base URL is empty".to_owned()));
        }
        if config.chat_model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig("chat model is empty".to_owned()));
        }
        if config.embed_model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        if config.timeout_seconds == 0 {
            return Err(OllamaError::InvalidConfig(
                "timeout must be greater than zero".to_owned(),
            ));
        }

        let base_url = Url::parse(&config.base_url)
            .map_err(|error| OllamaError::InvalidConfig(error.to_string()))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            base_url,
            chat_model: config.chat_model,
            embed_model: config.embed_model,
        })
    }

    pub fn chat_model(&self) -> &str {
        &self.chat_model
    }

    pub fn embedding_model(&self) -> &str {
        &self.embed_model
    }
}
```

### `backend/src/integrations/ollama/client/catalog.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/catalog.rs`
- Size bytes / Размер в байтах: `1224`
- Included characters / Включено символов: `1224`
- Truncated / Обрезано: `no`

```rust
use super::OllamaClient;
use super::error::OllamaError;
use super::responses::{TagsResponse, VersionResponse};

impl OllamaClient {
    pub async fn version(&self) -> Result<String, OllamaError> {
        let response: VersionResponse = self.get_json("/api/version").await?;
        if response.version.trim().is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama version response omitted version".to_owned(),
            ));
        }
        Ok(response.version)
    }

    pub async fn tags(&self) -> Result<Vec<String>, OllamaError> {
        let response: TagsResponse = self.get_json("/api/tags").await?;
        Ok(response
            .models
            .into_iter()
            .map(|model| model.name)
            .filter(|name| !name.trim().is_empty())
            .collect())
    }

    pub async fn validate_required_models(&self) -> Result<(), OllamaError> {
        let tags = self.tags().await?;
        for model in [&self.chat_model, &self.embed_model] {
            if !tags.iter().any(|tag| tag == model) {
                return Err(OllamaError::MissingModel {
                    model: model.to_owned(),
                });
            }
        }
        Ok(())
    }
}
```

### `backend/src/integrations/ollama/client/chat.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/chat.rs`
- Size bytes / Размер в байтах: `1685`
- Included characters / Включено символов: `1685`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::OllamaClient;
use super::error::OllamaError;
use super::models::OllamaChatResult;
use super::responses::ChatResponse;
use super::sanitization::strip_thinking_content;

impl OllamaClient {
    pub async fn chat(&self, prompt: &str) -> Result<OllamaChatResult, OllamaError> {
        self.chat_with_model(prompt, &self.chat_model).await
    }

    pub async fn chat_with_model(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<OllamaChatResult, OllamaError> {
        if model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig("chat model is empty".to_owned()));
        }
        let body = json!({
            "model": model,
            "stream": false,
            "think": false,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
        });
        let response: ChatResponse = self.post_json("/api/chat", &body).await?;
        let content = response
            .message
            .and_then(|message| message.content)
            .ok_or_else(|| {
                OllamaError::Protocol("Ollama chat response omitted assistant content".to_owned())
            })?;
        let content = strip_thinking_content(&content);
        if content.trim().is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama chat response content is empty".to_owned(),
            ));
        }

        Ok(OllamaChatResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            content,
            total_duration_ns: response.total_duration,
        })
    }
}
```

### `backend/src/integrations/ollama/client/config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/config.rs`
- Size bytes / Размер в байтах: `890`
- Included characters / Включено символов: `890`
- Truncated / Обрезано: `no`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaClientConfig {
    pub(in crate::integrations::ollama::client) base_url: String,
    pub(in crate::integrations::ollama::client) chat_model: String,
    pub(in crate::integrations::ollama::client) embed_model: String,
    pub(in crate::integrations::ollama::client) timeout_seconds: u64,
}

impl OllamaClientConfig {
    pub fn new(
        base_url: impl Into<String>,
        chat_model: impl Into<String>,
        embed_model: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            chat_model: chat_model.into(),
            embed_model: embed_model.into(),
            timeout_seconds: 120,
        }
    }

    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}
```

### `backend/src/integrations/ollama/client/embeddings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/embeddings.rs`
- Size bytes / Размер в байтах: `1649`
- Included characters / Включено символов: `1649`
- Truncated / Обрезано: `no`

```rust
use serde_json::json;

use super::OllamaClient;
use super::error::OllamaError;
use super::models::OllamaEmbedResult;
use super::responses::EmbedResponse;

impl OllamaClient {
    pub async fn embed(&self, input: &str) -> Result<OllamaEmbedResult, OllamaError> {
        self.embed_with_model(input, &self.embed_model).await
    }

    pub async fn embed_with_model(
        &self,
        input: &str,
        model: &str,
    ) -> Result<OllamaEmbedResult, OllamaError> {
        if model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "input": input,
        });
        let response: EmbedResponse = self.post_json("/api/embed", &body).await?;
        let embedding = response
            .embeddings
            .and_then(|mut embeddings| {
                if embeddings.is_empty() {
                    None
                } else {
                    Some(embeddings.remove(0))
                }
            })
            .or(response.embedding)
            .ok_or_else(|| {
                OllamaError::Protocol("Ollama embed response omitted embeddings".to_owned())
            })?;
        if embedding.is_empty() {
            return Err(OllamaError::Protocol(
                "Ollama embed response returned an empty vector".to_owned(),
            ));
        }

        Ok(OllamaEmbedResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            embedding,
            total_duration_ns: response.total_duration,
        })
    }
}
```

### `backend/src/integrations/ollama/client/error.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/error.rs`
- Size bytes / Размер в байтах: `472`
- Included characters / Включено символов: `472`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OllamaError {
    #[error("invalid Ollama client config: {0}")]
    InvalidConfig(String),

    #[error("Ollama endpoint returned HTTP {status}")]
    Endpoint { status: u16 },

    #[error("Ollama model `{model}` is not available")]
    MissingModel { model: String },

    #[error("Ollama protocol error: {0}")]
    Protocol(String),

    #[error("Ollama HTTP request failed")]
    Http(#[from] reqwest::Error),
}
```

### `backend/src/integrations/ollama/client/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/models.rs`
- Size bytes / Размер в байтах: `316`
- Included characters / Включено символов: `316`
- Truncated / Обрезано: `no`

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct OllamaChatResult {
    pub model: String,
    pub content: String,
    pub total_duration_ns: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OllamaEmbedResult {
    pub model: String,
    pub embedding: Vec<f32>,
    pub total_duration_ns: Option<u64>,
}
```

### `backend/src/integrations/ollama/client/responses.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/responses.rs`
- Size bytes / Размер в байтах: `946`
- Included characters / Включено символов: `946`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct VersionResponse {
    pub version: String,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct TagsResponse {
    pub models: Vec<TaggedModel>,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct TaggedModel {
    pub name: String,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct ChatResponse {
    pub model: Option<String>,
    pub message: Option<ChatMessage>,
    pub total_duration: Option<u64>,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct ChatMessage {
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub(in crate::integrations::ollama::client) struct EmbedResponse {
    pub model: Option<String>,
    pub embeddings: Option<Vec<Vec<f32>>>,
    pub embedding: Option<Vec<f32>>,
    pub total_duration: Option<u64>,
}
```

### `backend/src/integrations/ollama/client/sanitization.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/sanitization.rs`
- Size bytes / Размер в байтах: `633`
- Included characters / Включено символов: `633`
- Truncated / Обрезано: `no`

```rust
pub(in crate::integrations::ollama::client) fn strip_thinking_content(content: &str) -> String {
    let mut sanitized = content.trim().to_owned();
    while let Some(start) = sanitized.find("<think>") {
        let Some(end_offset) = sanitized[start..].find("</think>") else {
            sanitized.replace_range(start.., "");
            break;
        };
        let end = start + end_offset + "</think>".len();
        sanitized.replace_range(start..end, "");
    }

    if let Some(end) = sanitized.rfind("</think>") {
        sanitized = sanitized[end + "</think>".len()..].to_owned();
    }

    sanitized.trim().to_owned()
}
```

### `backend/src/integrations/ollama/client/transport.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/client/transport.rs`
- Size bytes / Размер в байтах: `1553`
- Included characters / Включено символов: `1553`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;
use serde_json::Value;

use super::OllamaClient;
use super::error::OllamaError;

impl OllamaClient {
    pub(in crate::integrations::ollama::client) fn endpoint(
        &self,
        path: &str,
    ) -> Result<reqwest::Url, OllamaError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| OllamaError::InvalidConfig(error.to_string()))
    }

    pub(in crate::integrations::ollama::client) async fn get_json<T>(
        &self,
        path: &str,
    ) -> Result<T, OllamaError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.http.get(self.endpoint(path)?).send().await?;
        decode_response(response).await
    }

    pub(in crate::integrations::ollama::client) async fn post_json<T>(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<T, OllamaError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self
            .http
            .post(self.endpoint(path)?)
            .json(body)
            .send()
            .await?;
        decode_response(response).await
    }
}

async fn decode_response<T>(response: reqwest::Response) -> Result<T, OllamaError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status();
    if !status.is_success() {
        return Err(OllamaError::Endpoint {
            status: status.as_u16(),
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|error| OllamaError::Protocol(error.to_string()))
}
```

### `backend/src/integrations/ollama/mod.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/ollama/mod.rs`
- Size bytes / Размер в байтах: `16`
- Included characters / Включено символов: `16`
- Truncated / Обрезано: `no`

```rust
pub mod client;
```

### `backend/src/integrations/omniroute/client.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client.rs`
- Size bytes / Размер в байтах: `2011`
- Included characters / Включено символов: `2011`
- Truncated / Обрезано: `no`

```rust
use std::time::Duration;

use reqwest::Url;

mod catalog;
mod chat;
mod config;
mod embeddings;
mod error;
mod models;
mod transport;

pub use config::OmniRouteClientConfig;
pub use error::OmniRouteError;
pub use models::{OmniRouteChatResult, OmniRouteEmbedResult};

#[derive(Clone)]
pub struct OmniRouteClient {
    http: reqwest::Client,
    base_url: Url,
    chat_model: String,
    embed_model: String,
    api_key: crate::platform::secrets::ResolvedSecret,
}

impl OmniRouteClient {
    pub fn new(config: OmniRouteClientConfig) -> Result<Self, OmniRouteError> {
        if config.base_url.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "base URL is empty".to_owned(),
            ));
        }
        if config.chat_model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "chat model is empty".to_owned(),
            ));
        }
        if config.embed_model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        if config.timeout_seconds == 0 {
            return Err(OmniRouteError::InvalidConfig(
                "timeout must be greater than zero".to_owned(),
            ));
        }

        let mut base_url = config.base_url.trim_end_matches('/').to_owned();
        base_url.push('/');
        let base_url = Url::parse(&base_url)
            .map_err(|error| OmniRouteError::InvalidConfig(error.to_string()))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            http,
            base_url,
            chat_model: config.chat_model,
            embed_model: config.embed_model,
            api_key: config.api_key,
        })
    }

    pub fn chat_model(&self) -> &str {
        &self.chat_model
    }

    pub fn embedding_model(&self) -> &str {
        &self.embed_model
    }
}
```

### `backend/src/integrations/omniroute/client/catalog.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client/catalog.rs`
- Size bytes / Размер в байтах: `975`
- Included characters / Включено символов: `975`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;

use super::{OmniRouteClient, OmniRouteError};

impl OmniRouteClient {
    pub async fn models(&self) -> Result<Vec<String>, OmniRouteError> {
        let response: ModelsResponse = self.get_json("models").await?;
        Ok(response
            .data
            .into_iter()
            .map(|model| model.id)
            .filter(|id| !id.trim().is_empty())
            .collect())
    }

    pub async fn validate_required_models(&self) -> Result<(), OmniRouteError> {
        let models = self.models().await?;
        for model in [&self.chat_model, &self.embed_model] {
            if !models.iter().any(|candidate| candidate == model) {
                return Err(OmniRouteError::MissingModel {
                    model: model.to_owned(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelItem>,
}

#[derive(Deserialize)]
struct ModelItem {
    id: String,
}
```

### `backend/src/integrations/omniroute/client/chat.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client/chat.rs`
- Size bytes / Размер в байтах: `2559`
- Included characters / Включено символов: `2559`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;
use serde_json::json;

use super::models::OmniRouteChatResult;
use super::{OmniRouteClient, OmniRouteError};

impl OmniRouteClient {
    pub async fn chat(&self, prompt: &str) -> Result<OmniRouteChatResult, OmniRouteError> {
        self.chat_with_model(prompt, &self.chat_model).await
    }

    pub async fn chat_with_model(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<OmniRouteChatResult, OmniRouteError> {
        if model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "chat model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "stream": false,
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
        });
        let response: ChatCompletionsResponse = self.post_json("chat/completions", &body).await?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| {
                OmniRouteError::Protocol(
                    "OmniRoute chat response omitted assistant content".to_owned(),
                )
            })?;
        let content = strip_thinking_content(&content);
        if content.trim().is_empty() {
            return Err(OmniRouteError::Protocol(
                "OmniRoute chat response content is empty".to_owned(),
            ));
        }

        Ok(OmniRouteChatResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            content,
        })
    }
}

fn strip_thinking_content(content: &str) -> String {
    let mut sanitized = content.trim().to_owned();
    while let Some(start) = sanitized.find("<think>") {
        let Some(end_offset) = sanitized[start..].find("</think>") else {
            sanitized.replace_range(start.., "");
            break;
        };
        let end = start + end_offset + "</think>".len();
        sanitized.replace_range(start..end, "");
    }

    if let Some(end) = sanitized.rfind("</think>") {
        sanitized = sanitized[end + "</think>".len()..].to_owned();
    }

    sanitized.trim().to_owned()
}

#[derive(Deserialize)]
struct ChatCompletionsResponse {
    model: Option<String>,
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: Option<String>,
}
```

### `backend/src/integrations/omniroute/client/config.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client/config.rs`
- Size bytes / Размер в байтах: `883`
- Included characters / Включено символов: `883`
- Truncated / Обрезано: `no`

```rust
use crate::platform::secrets::ResolvedSecret;

#[derive(Clone)]
pub struct OmniRouteClientConfig {
    pub(super) base_url: String,
    pub(super) chat_model: String,
    pub(super) embed_model: String,
    pub(super) api_key: ResolvedSecret,
    pub(super) timeout_seconds: u64,
}

impl OmniRouteClientConfig {
    pub fn new(
        base_url: impl Into<String>,
        chat_model: impl Into<String>,
        embed_model: impl Into<String>,
        api_key: ResolvedSecret,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            chat_model: chat_model.into(),
            embed_model: embed_model.into(),
            api_key,
            timeout_seconds: 120,
        }
    }

    pub fn with_timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}
```

### `backend/src/integrations/omniroute/client/embeddings.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client/embeddings.rs`
- Size bytes / Размер в байтах: `1629`
- Included characters / Включено символов: `1629`
- Truncated / Обрезано: `no`

```rust
use serde::Deserialize;
use serde_json::json;

use super::models::OmniRouteEmbedResult;
use super::{OmniRouteClient, OmniRouteError};

impl OmniRouteClient {
    pub async fn embed(&self, input: &str) -> Result<OmniRouteEmbedResult, OmniRouteError> {
        self.embed_with_model(input, &self.embed_model).await
    }

    pub async fn embed_with_model(
        &self,
        input: &str,
        model: &str,
    ) -> Result<OmniRouteEmbedResult, OmniRouteError> {
        if model.trim().is_empty() {
            return Err(OmniRouteError::InvalidConfig(
                "embedding model is empty".to_owned(),
            ));
        }
        let body = json!({
            "model": model,
            "input": input,
        });
        let response: EmbeddingsResponse = self.post_json("embeddings", &body).await?;
        let embedding = response
            .data
            .into_iter()
            .next()
            .map(|item| item.embedding)
            .ok_or_else(|| {
                OmniRouteError::Protocol("OmniRoute embeddings response omitted data".to_owned())
            })?;
        if embedding.is_empty() {
            return Err(OmniRouteError::Protocol(
                "OmniRoute embeddings response returned an empty vector".to_owned(),
            ));
        }

        Ok(OmniRouteEmbedResult {
            model: response.model.unwrap_or_else(|| model.to_owned()),
            embedding,
        })
    }
}

#[derive(Deserialize)]
struct EmbeddingsResponse {
    model: Option<String>,
    data: Vec<EmbeddingItem>,
}

#[derive(Deserialize)]
struct EmbeddingItem {
    embedding: Vec<f32>,
}
```

### `backend/src/integrations/omniroute/client/error.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/integrations/omniroute/client/error.rs`
- Size bytes / Размер в байтах: `562`
- Included characters / Включено символов: `562`
- Truncated / Обрезано: `no`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmniRouteError {
    #[error("invalid OmniRoute client config: {0}")]
    InvalidConfig(String),

    #[error("OmniRoute API key is not configured")]
    MissingApiKey,

    #[error("OmniRoute endpoint returned HTTP {status}")]
    Endpoint { status: u16 },

    #[error("OmniRoute model `{model}` is not available")]
    MissingModel { model: String },

    #[error("OmniRoute protocol error: {0}")]
    Protocol(String),

    #[error("OmniRoute HTTP request failed")]
    Http(#[from] reqwest::Error),
}
```

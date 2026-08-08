use std::fmt::Debug;
use std::time::Duration;

use async_imap::extensions::idle::IdleResponse;
use async_imap::types::{Name, NameAttribute};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use chrono::Utc;
use futures::TryStreamExt;
use serde_json::{Value, json};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::platform::communications::email_sync::imap_mailbox_stream_id;
use crate::platform::secrets::models::ResolvedSecret;
use makosh_communications_api::email_sync::{EmailSyncBatch, FetchedCommunicationSourceMessage};

use super::errors::EmailProviderNetworkError;
use super::helpers::{
    imap_checkpoint, imap_uid_search_query, next_imap_uid_floor, retain_uids_from_floor,
    select_uids_for_fetch, sha256_fingerprint, uid_set,
};
use super::options::{ImapFetchOptions, ImapIdleOptions, ImapMailboxListOptions};

const IMAP_UID_FETCH_CHUNK_SIZE: usize = 10;
const IMAP_UID_FETCH_TIMEOUT_SECONDS: u64 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapMailboxRole {
    All,
    Archive,
    Drafts,
    Flagged,
    Junk,
    Sent,
    Trash,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapMailboxDescriptor {
    pub name: String,
    pub roles: Vec<ImapMailboxRole>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapIdleOutcome {
    Changed,
    TimedOut,
    Unsupported,
}

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

    pub async fn list_mailboxes(
        &self,
        password: &ResolvedSecret,
        options: &ImapMailboxListOptions,
    ) -> Result<Vec<String>, EmailProviderNetworkError> {
        let descriptors = self.discover_mailboxes(password, options).await?;
        let mailboxes = descriptors
            .into_iter()
            .filter(|mailbox| !mailbox.roles.contains(&ImapMailboxRole::All))
            .map(|mailbox| mailbox.name)
            .collect::<Vec<_>>();
        if mailboxes.is_empty() {
            return Err(EmailProviderNetworkError::UnexpectedProviderResponse {
                message: "no selectable IMAP mailboxes",
            });
        }
        Ok(mailboxes)
    }

    pub async fn discover_mailboxes(
        &self,
        password: &ResolvedSecret,
        options: &ImapMailboxListOptions,
    ) -> Result<Vec<ImapMailboxDescriptor>, EmailProviderNetworkError> {
        options.validate()?;

        let address = (options.host.as_str(), options.port);
        let tcp_stream = tokio::net::TcpStream::connect(address).await?;
        if options.tls {
            let tls_stream = async_native_tls::connect(options.host.as_str(), tcp_stream).await?;
            discover_imap_mailboxes_with_client(
                async_imap::Client::new(tls_stream),
                password,
                options,
            )
            .await
        } else {
            discover_imap_mailboxes_with_client(
                async_imap::Client::new(tcp_stream),
                password,
                options,
            )
            .await
        }
    }

    pub async fn wait_for_change(
        &self,
        password: &ResolvedSecret,
        options: &ImapIdleOptions,
    ) -> Result<ImapIdleOutcome, EmailProviderNetworkError> {
        options.validate()?;

        let address = (options.host.as_str(), options.port);
        let tcp_stream = tokio::net::TcpStream::connect(address).await?;
        if options.tls {
            let tls_stream = async_native_tls::connect(options.host.as_str(), tcp_stream).await?;
            wait_for_imap_change_with_client(async_imap::Client::new(tls_stream), password, options)
                .await
        } else {
            wait_for_imap_change_with_client(async_imap::Client::new(tcp_stream), password, options)
                .await
        }
    }
}

async fn wait_for_imap_change_with_client<T>(
    mut client: async_imap::Client<T>,
    password: &ResolvedSecret,
    options: &ImapIdleOptions,
) -> Result<ImapIdleOutcome, EmailProviderNetworkError>
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
    if !session.capabilities().await?.has_str("IDLE") {
        session.logout().await?;
        return Ok(ImapIdleOutcome::Unsupported);
    }

    session.examine(&options.mailbox).await?;
    let mut idle = session.idle();
    idle.init().await?;
    let response = {
        let (wait, _interrupt) = idle.wait_with_timeout(options.timeout);
        wait.await?
    };
    let mut session = idle.done().await?;
    session.logout().await?;

    match response {
        IdleResponse::NewData(_) => Ok(ImapIdleOutcome::Changed),
        IdleResponse::Timeout => Ok(ImapIdleOutcome::TimedOut),
        IdleResponse::ManualInterrupt => {
            Err(EmailProviderNetworkError::UnexpectedProviderResponse {
                message: "IMAP IDLE connection ended before a change or timeout",
            })
        }
    }
}

async fn discover_imap_mailboxes_with_client<T>(
    mut client: async_imap::Client<T>,
    password: &ResolvedSecret,
    options: &ImapMailboxListOptions,
) -> Result<Vec<ImapMailboxDescriptor>, EmailProviderNetworkError>
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
    let names = {
        let stream = session.list(None, Some("*")).await?;
        stream.try_collect::<Vec<_>>().await?
    };
    session.logout().await?;

    let mut mailboxes = Vec::<ImapMailboxDescriptor>::new();
    for name in names {
        if imap_mailbox_name_is_selectable(&name) {
            let mailbox = name.name().trim().to_owned();
            if !mailbox.is_empty() && !mailboxes.iter().any(|item| item.name == mailbox) {
                mailboxes.push(ImapMailboxDescriptor {
                    name: mailbox,
                    roles: imap_mailbox_roles(name.attributes()),
                });
            }
        }
    }
    if mailboxes.is_empty() {
        return Err(EmailProviderNetworkError::UnexpectedProviderResponse {
            message: "no selectable IMAP mailboxes",
        });
    }

    Ok(mailboxes)
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
    let latest_uid = latest_imap_uid(&messages, options.last_seen_uid);
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
                    .uid_fetch(uid_set, "(UID FLAGS BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
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
            let uid_validity =
                mailbox
                    .uid_validity
                    .ok_or(EmailProviderNetworkError::MissingProviderField {
                        field: "uid_validity",
                    })?;
            let provider_record_id = imap_provider_record_id(&options.mailbox, uid_validity, uid);
            let occurred_at = fetched_message
                .internal_date()
                .map(|internal_date| internal_date.with_timezone(&Utc));
            let is_read = fetched_message
                .flags()
                .any(|flag| matches!(flag, async_imap::types::Flag::Seen));
            let is_starred = fetched_message
                .flags()
                .any(|flag| matches!(flag, async_imap::types::Flag::Flagged));

            messages.push(FetchedCommunicationSourceMessage {
                provider_record_id: provider_record_id.clone(),
                source_fingerprint: sha256_fingerprint([
                    "imap".as_bytes(),
                    provider_record_id.as_bytes(),
                    body,
                ]),
                occurred_at,
                payload: json!({
                    "provider": options.provider_kind.as_str(),
                    "transport": "imap",
                    "mailbox": options.mailbox,
                    "uid": uid,
                    "uid_validity": mailbox.uid_validity,
                    "is_read": is_read,
                    "is_starred": is_starred,
                    "raw_rfc822_base64": BASE64_STANDARD.encode(body),
                    "rfc822_size": fetched_message.size
                }),
            });
        }
    }

    Ok(messages)
}

fn imap_provider_record_id(mailbox: &str, uid_validity: u32, uid: u32) -> String {
    format!(
        "imap:v2:{}:{uid_validity}:{uid}",
        imap_mailbox_stream_id(mailbox)
    )
}

fn imap_mailbox_name_is_selectable(name: &Name) -> bool {
    !name
        .attributes()
        .iter()
        .any(|attribute| matches!(attribute, NameAttribute::NoSelect))
}

fn imap_mailbox_roles(attributes: &[NameAttribute<'_>]) -> Vec<ImapMailboxRole> {
    attributes
        .iter()
        .filter_map(|attribute| match attribute {
            NameAttribute::All => Some(ImapMailboxRole::All),
            NameAttribute::Archive => Some(ImapMailboxRole::Archive),
            NameAttribute::Drafts => Some(ImapMailboxRole::Drafts),
            NameAttribute::Flagged => Some(ImapMailboxRole::Flagged),
            NameAttribute::Junk => Some(ImapMailboxRole::Junk),
            NameAttribute::Sent => Some(ImapMailboxRole::Sent),
            NameAttribute::Trash => Some(ImapMailboxRole::Trash),
            _ => None,
        })
        .collect()
}

fn latest_imap_uid(
    messages: &[FetchedCommunicationSourceMessage],
    fallback: Option<u32>,
) -> Option<u32> {
    messages
        .iter()
        .filter_map(|message| payload_uid(&message.payload))
        .max()
        .or(fallback)
}

fn payload_uid(payload: &Value) -> Option<u32> {
    payload
        .get("uid")
        .and_then(Value::as_u64)
        .and_then(|uid| u32::try_from(uid).ok())
}

#[cfg(test)]
mod tests {
    use async_imap::types::NameAttribute;
    use serde_json::json;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    use super::{
        ImapIdleOutcome, ImapMailboxRole, discover_imap_mailboxes_with_client, imap_mailbox_roles,
        imap_provider_record_id, latest_imap_uid, wait_for_imap_change_with_client,
    };
    use crate::integrations::mail::gmail::client::options::{
        ImapIdleOptions, ImapMailboxListOptions,
    };
    use crate::platform::secrets::models::ResolvedSecret;
    use makosh_communications_api::email_sync::FetchedCommunicationSourceMessage;

    #[test]
    fn imap_provider_record_id_is_scoped_to_mailbox_uid_validity_and_uid() {
        assert_eq!(
            imap_provider_record_id("INBOX", 7, 42),
            "imap:v2:imap:INBOX:7:42"
        );
        assert_eq!(
            imap_provider_record_id("Junk", 9, 42),
            "imap:v2:imap:Junk:9:42"
        );
        assert_eq!(
            imap_provider_record_id("Projects:2026%Q2", 11, 42),
            "imap:v2:imap:Projects%3A2026%25Q2:11:42"
        );
    }

    #[test]
    fn special_use_attributes_map_to_provider_neutral_mailbox_roles() {
        assert_eq!(
            imap_mailbox_roles(&[
                NameAttribute::Archive,
                NameAttribute::Junk,
                NameAttribute::Extension("vendor-role".into()),
            ]),
            vec![ImapMailboxRole::Archive, ImapMailboxRole::Junk]
        );
    }

    #[tokio::test]
    async fn mailbox_discovery_preserves_special_use_roles_from_list_responses() {
        let (client_stream, server_stream) = tokio::io::duplex(8 * 1024);
        let server = tokio::spawn(async move {
            let (read_half, mut write_half) = tokio::io::split(server_stream);
            let mut lines = BufReader::new(read_half).lines();
            write_half
                .write_all(b"* OK special-use fixture ready\r\n")
                .await
                .expect("write greeting");
            while let Some(command) = lines.next_line().await.expect("read IMAP command") {
                let tag = command.split_whitespace().next().expect("command tag");
                if command.contains(" LOGIN ") {
                    write_half
                        .write_all(format!("{tag} OK LOGIN completed\r\n").as_bytes())
                        .await
                        .expect("write LOGIN response");
                } else if command.contains(" LIST ") {
                    write_half
                        .write_all(
                            format!(
                                "* LIST (\\Archive) \"/\" \"Archive\"\r\n\
                                 * LIST (\\Junk) \"/\" \"Junk Mail\"\r\n\
                                 * LIST (\\All) \"/\" \"All Mail\"\r\n\
                                 * LIST (\\NoSelect) \"/\" \"Containers\"\r\n\
                                 {tag} OK LIST completed\r\n"
                            )
                            .as_bytes(),
                        )
                        .await
                        .expect("write LIST response");
                } else if command.contains(" LOGOUT") {
                    write_half
                        .write_all(
                            format!("* BYE done\r\n{tag} OK LOGOUT completed\r\n").as_bytes(),
                        )
                        .await
                        .expect("write LOGOUT response");
                    break;
                }
            }
        });

        let password = ResolvedSecret::new("fixture-password").expect("password");
        let options = ImapMailboxListOptions::new("imap.example.test", 143, false, "owner");
        let mailboxes = discover_imap_mailboxes_with_client(
            async_imap::Client::new(client_stream),
            &password,
            &options,
        )
        .await
        .expect("discover special-use mailboxes");
        server.await.expect("IMAP fixture task");

        assert_eq!(mailboxes.len(), 3);
        assert_eq!(mailboxes[0].name, "Archive");
        assert_eq!(mailboxes[0].roles, vec![ImapMailboxRole::Archive]);
        assert_eq!(mailboxes[1].name, "Junk Mail");
        assert_eq!(mailboxes[1].roles, vec![ImapMailboxRole::Junk]);
        assert_eq!(mailboxes[2].name, "All Mail");
        assert_eq!(mailboxes[2].roles, vec![ImapMailboxRole::All]);
    }

    #[tokio::test]
    async fn idle_wait_reports_new_mail_and_closes_idle_session_cleanly() {
        let (client_stream, server_stream) = tokio::io::duplex(8 * 1024);
        let server = tokio::spawn(async move {
            let (read_half, mut write_half) = tokio::io::split(server_stream);
            let mut lines = BufReader::new(read_half).lines();
            let mut idle_tag = None::<String>;
            write_half
                .write_all(b"* OK IDLE fixture ready\r\n")
                .await
                .expect("write greeting");
            while let Some(command) = lines.next_line().await.expect("read IMAP command") {
                let tag = command.split_whitespace().next().expect("command tag");
                if command.contains(" LOGIN ") {
                    write_half
                        .write_all(format!("{tag} OK LOGIN completed\r\n").as_bytes())
                        .await
                        .expect("write LOGIN response");
                } else if command.contains(" CAPABILITY") {
                    write_half
                        .write_all(
                            format!(
                                "* CAPABILITY IMAP4rev1 IDLE\r\n{tag} OK CAPABILITY completed\r\n"
                            )
                            .as_bytes(),
                        )
                        .await
                        .expect("write CAPABILITY response");
                } else if command.contains(" EXAMINE ") {
                    write_half
                        .write_all(
                            format!(
                                "* FLAGS (\\Seen)\r\n* 1 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-ONLY] EXAMINE completed\r\n"
                            )
                            .as_bytes(),
                        )
                        .await
                        .expect("write EXAMINE response");
                } else if command.contains(" IDLE") {
                    idle_tag = Some(tag.to_owned());
                    write_half
                        .write_all(b"+ idling\r\n* 2 EXISTS\r\n")
                        .await
                        .expect("write IDLE change");
                } else if command == "DONE" {
                    write_half
                        .write_all(
                            format!(
                                "{} OK IDLE completed\r\n",
                                idle_tag.as_deref().expect("IDLE tag")
                            )
                            .as_bytes(),
                        )
                        .await
                        .expect("write IDLE completion");
                } else if command.contains(" LOGOUT") {
                    write_half
                        .write_all(
                            format!("* BYE done\r\n{tag} OK LOGOUT completed\r\n").as_bytes(),
                        )
                        .await
                        .expect("write LOGOUT response");
                    break;
                }
            }
        });

        let password = ResolvedSecret::new("fixture-password").expect("password");
        let options = ImapIdleOptions::new(
            "imap.example.test",
            143,
            false,
            "INBOX",
            "owner",
            std::time::Duration::from_secs(1),
        );
        let outcome = wait_for_imap_change_with_client(
            async_imap::Client::new(client_stream),
            &password,
            &options,
        )
        .await
        .expect("wait for IMAP IDLE change");
        server.await.expect("IMAP fixture task");

        assert_eq!(outcome, ImapIdleOutcome::Changed);
    }

    #[tokio::test]
    async fn idle_wait_uses_polling_fallback_when_server_has_no_idle_capability() {
        let (client_stream, server_stream) = tokio::io::duplex(8 * 1024);
        let server = tokio::spawn(async move {
            let (read_half, mut write_half) = tokio::io::split(server_stream);
            let mut lines = BufReader::new(read_half).lines();
            write_half
                .write_all(b"* OK polling fixture ready\r\n")
                .await
                .expect("write greeting");
            while let Some(command) = lines.next_line().await.expect("read IMAP command") {
                let tag = command.split_whitespace().next().expect("command tag");
                if command.contains(" LOGIN ") {
                    write_half
                        .write_all(format!("{tag} OK LOGIN completed\r\n").as_bytes())
                        .await
                        .expect("write LOGIN response");
                } else if command.contains(" CAPABILITY") {
                    write_half
                        .write_all(
                            format!("* CAPABILITY IMAP4rev1\r\n{tag} OK CAPABILITY completed\r\n")
                                .as_bytes(),
                        )
                        .await
                        .expect("write CAPABILITY response");
                } else if command.contains(" LOGOUT") {
                    write_half
                        .write_all(
                            format!("* BYE done\r\n{tag} OK LOGOUT completed\r\n").as_bytes(),
                        )
                        .await
                        .expect("write LOGOUT response");
                    break;
                } else {
                    panic!("unexpected command without IDLE capability: {command}");
                }
            }
        });

        let password = ResolvedSecret::new("fixture-password").expect("password");
        let options = ImapIdleOptions::new(
            "imap.example.test",
            143,
            false,
            "INBOX",
            "owner",
            std::time::Duration::from_secs(1),
        );
        let outcome = wait_for_imap_change_with_client(
            async_imap::Client::new(client_stream),
            &password,
            &options,
        )
        .await
        .expect("detect missing IMAP IDLE capability");
        server.await.expect("IMAP fixture task");

        assert_eq!(outcome, ImapIdleOutcome::Unsupported);
    }

    #[test]
    fn latest_imap_uid_uses_payload_uid_for_namespaced_records() {
        let messages = vec![FetchedCommunicationSourceMessage {
            provider_record_id: "imap:Junk:42".to_owned(),
            source_fingerprint: "sha256:latest-imap-uid".to_owned(),
            occurred_at: None,
            payload: json!({
                "mailbox": "Junk",
                "uid": 42
            }),
        }];

        assert_eq!(latest_imap_uid(&messages, Some(10)), Some(42));
    }
}

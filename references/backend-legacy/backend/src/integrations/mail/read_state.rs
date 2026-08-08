use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::{
    CommunicationProviderKind, ProviderAccount, ProviderSecretBindingLookupPort,
};
use std::sync::Arc;

use serde_json::Value;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::integrations::mail::accounts::service::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::{
    errors::EmailProviderNetworkError,
    gmail_api::GmailApiClient,
    imap::{ImapMailboxRole, ImapNetworkClient},
    options::ImapMailboxListOptions,
};
use crate::integrations::mail::imap_write::{ImapWriteClient, ImapWriteConfig, ImapWriteError};

use crate::platform::secrets::models::ResolvedSecret;
use crate::vault::HostVault;

use super::sync_provider::read_provider_secret;

pub struct EmailReadStateRequest<'a> {
    pub account: &'a ProviderAccount,
    pub provider_record_id: &'a str,
    pub message_metadata: &'a Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailProviderMessageMutation<'a> {
    SetRead(bool),
    SetImportant(bool),
    SetStarred(bool),
    Archive {
        destination_mailbox: Option<&'a str>,
    },
    Trash {
        destination_mailbox: Option<&'a str>,
    },
    MarkSpam {
        destination_mailbox: Option<&'a str>,
    },
    UnmarkSpam {
        destination_mailbox: Option<&'a str>,
    },
    AddLabel(&'a str),
    RemoveLabel(&'a str),
    MoveTo(&'a str),
    CopyTo(&'a str),
}

#[derive(Clone)]
pub struct LiveEmailReadStateService {
    pool: PgPool,
    vault: HostVault,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingLookupPort>,
    gmail_api_base_url: String,
}

impl LiveEmailReadStateService {
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

    pub async fn mark_message_read(
        &self,
        request: EmailReadStateRequest<'_>,
    ) -> Result<(), EmailReadStateError> {
        self.set_message_read(request, true).await
    }

    pub async fn set_message_read(
        &self,
        request: EmailReadStateRequest<'_>,
        is_read: bool,
    ) -> Result<(), EmailReadStateError> {
        self.apply_message_mutation(request, EmailProviderMessageMutation::SetRead(is_read))
            .await
    }

    pub async fn apply_message_mutation(
        &self,
        request: EmailReadStateRequest<'_>,
        mutation: EmailProviderMessageMutation<'_>,
    ) -> Result<(), EmailReadStateError> {
        match request.account.provider_kind {
            CommunicationProviderKind::Gmail => self.mutate_gmail_message(request, mutation).await,
            CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap => {
                self.mutate_imap_messages(
                    request.account,
                    std::slice::from_ref(request.message_metadata),
                    mutation,
                )
                .await
            }
            provider_kind => Err(EmailReadStateError::UnsupportedProvider(
                provider_kind.as_str(),
            )),
        }
    }

    pub async fn apply_gmail_batch_mutation(
        &self,
        account: &ProviderAccount,
        provider_record_ids: &[String],
        mutation: EmailProviderMessageMutation<'_>,
    ) -> Result<(), EmailReadStateError> {
        if account.provider_kind != CommunicationProviderKind::Gmail {
            return Err(EmailReadStateError::UnsupportedProvider(
                account.provider_kind.as_str(),
            ));
        }
        let access_token = self.gmail_access_token(&account.account_id).await?;
        let client = GmailApiClient::new(gmail_api_base_url(
            &account.config,
            &self.gmail_api_base_url,
        ))
        .user_id("me");
        let (add_labels, remove_labels) = gmail_label_ids_for_mutation(mutation);
        client
            .batch_modify_messages(
                &access_token,
                provider_record_ids,
                &add_labels,
                &remove_labels,
            )
            .await
            .map_err(EmailReadStateError::Gmail)
    }

    pub async fn apply_imap_batch_mutation(
        &self,
        account: &ProviderAccount,
        message_metadata: &[Value],
        mutation: EmailProviderMessageMutation<'_>,
    ) -> Result<(), EmailReadStateError> {
        if !matches!(
            account.provider_kind,
            CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap
        ) {
            return Err(EmailReadStateError::UnsupportedProvider(
                account.provider_kind.as_str(),
            ));
        }
        self.mutate_imap_messages(account, message_metadata, mutation)
            .await
    }

    async fn mutate_gmail_message(
        &self,
        request: EmailReadStateRequest<'_>,
        mutation: EmailProviderMessageMutation<'_>,
    ) -> Result<(), EmailReadStateError> {
        let access_token = self.gmail_access_token(&request.account.account_id).await?;
        let client = GmailApiClient::new(gmail_api_base_url(
            &request.account.config,
            &self.gmail_api_base_url,
        ))
        .user_id("me");
        let (add_labels, remove_labels) = gmail_label_ids_for_mutation(mutation);
        client
            .modify_message(
                &access_token,
                request.provider_record_id,
                &add_labels,
                &remove_labels,
            )
            .await
            .map_err(EmailReadStateError::Gmail)
    }

    async fn mutate_imap_messages(
        &self,
        account: &ProviderAccount,
        message_metadata: &[Value],
        mutation: EmailProviderMessageMutation<'_>,
    ) -> Result<(), EmailReadStateError> {
        let first_metadata = message_metadata
            .first()
            .ok_or(EmailReadStateError::InvalidConfig)?;
        let config = imap_write_config(account, first_metadata)?;
        let expected_uid_validity = imap_uid_validity(first_metadata);
        let mut uids = Vec::with_capacity(message_metadata.len());
        for metadata in message_metadata {
            if imap_write_config(account, metadata)? != config {
                return Err(EmailReadStateError::InvalidConfig);
            }
            if imap_uid_validity(metadata) != expected_uid_validity {
                return Err(EmailReadStateError::InvalidConfig);
            }
            let uid = imap_uid(metadata)?;
            if !uids.contains(&uid) {
                uids.push(uid);
            }
        }
        let secret_store = SecretReferenceStore::new(self.pool.clone());
        let password = read_provider_secret(
            self.provider_secret_binding_store.as_ref(),
            &secret_store,
            &self.vault,
            &account.account_id,
            ProviderAccountSecretPurpose::ImapPassword,
        )
        .await
        .map_err(|error| EmailReadStateError::Credential(error.to_string()))?;
        let discovered_destination = self
            .discover_imap_destination(&config, &password, mutation)
            .await?;
        let config = ImapWriteConfig {
            host: &config.host,
            port: config.port,
            tls: config.tls,
            username: &config.username,
            password: &password,
            mailbox: &config.mailbox,
            expected_uid_validity,
        };
        let client = ImapWriteClient::new();
        match mutation {
            EmailProviderMessageMutation::SetRead(true) => client.mark_seen(&config, &uids).await,
            EmailProviderMessageMutation::SetRead(false) => {
                client.mark_unseen(&config, &uids).await
            }
            EmailProviderMessageMutation::SetImportant(true) => {
                client.add_flags(&config, &uids, &["\\Flagged"]).await
            }
            EmailProviderMessageMutation::SetImportant(false) => {
                client.remove_flags(&config, &uids, &["\\Flagged"]).await
            }
            EmailProviderMessageMutation::SetStarred(true) => {
                client.add_flags(&config, &uids, &["\\Flagged"]).await
            }
            EmailProviderMessageMutation::SetStarred(false) => {
                client.remove_flags(&config, &uids, &["\\Flagged"]).await
            }
            EmailProviderMessageMutation::Archive {
                destination_mailbox,
            } => {
                client
                    .move_messages(
                        &config,
                        &uids,
                        required_destination_mailbox(
                            optional_destination_mailbox(destination_mailbox)
                                .or(discovered_destination.as_deref()),
                        )?,
                    )
                    .await
            }
            EmailProviderMessageMutation::Trash {
                destination_mailbox,
            }
            | EmailProviderMessageMutation::MarkSpam {
                destination_mailbox,
            }
            | EmailProviderMessageMutation::UnmarkSpam {
                destination_mailbox,
            } => {
                client
                    .move_messages(
                        &config,
                        &uids,
                        required_destination_mailbox(
                            optional_destination_mailbox(destination_mailbox)
                                .or(discovered_destination.as_deref()),
                        )?,
                    )
                    .await
            }
            EmailProviderMessageMutation::AddLabel(label) => {
                client.add_flags(&config, &uids, &[label]).await
            }
            EmailProviderMessageMutation::RemoveLabel(label) => {
                client.remove_flags(&config, &uids, &[label]).await
            }
            EmailProviderMessageMutation::MoveTo(mailbox) => {
                client.move_messages(&config, &uids, mailbox).await
            }
            EmailProviderMessageMutation::CopyTo(mailbox) => {
                client.copy_messages(&config, &uids, mailbox).await
            }
        }
        .map_err(EmailReadStateError::Imap)
    }

    async fn discover_imap_destination(
        &self,
        config: &ImapWriteAccountConfig,
        password: &ResolvedSecret,
        mutation: EmailProviderMessageMutation<'_>,
    ) -> Result<Option<String>, EmailReadStateError> {
        let (provided, role, default_mailbox) = match mutation {
            EmailProviderMessageMutation::Archive {
                destination_mailbox,
            } => (destination_mailbox, Some(ImapMailboxRole::Archive), None),
            EmailProviderMessageMutation::Trash {
                destination_mailbox,
            } => (destination_mailbox, Some(ImapMailboxRole::Trash), None),
            EmailProviderMessageMutation::MarkSpam {
                destination_mailbox,
            } => (destination_mailbox, Some(ImapMailboxRole::Junk), None),
            EmailProviderMessageMutation::UnmarkSpam {
                destination_mailbox,
            } => (destination_mailbox, None, Some("INBOX")),
            _ => (None, None, None),
        };
        if optional_destination_mailbox(provided).is_some() {
            return Ok(None);
        }
        if let Some(mailbox) = default_mailbox {
            return Ok(Some(mailbox.to_owned()));
        }
        let Some(role) = role else {
            return Ok(None);
        };
        let options =
            ImapMailboxListOptions::new(&config.host, config.port, config.tls, &config.username);
        let mailboxes = ImapNetworkClient::new()
            .discover_mailboxes(password, &options)
            .await
            .map_err(EmailReadStateError::ImapDiscovery)?;
        mailboxes
            .into_iter()
            .find(|mailbox| mailbox.roles.contains(&role))
            .map(|mailbox| Some(mailbox.name))
            .ok_or(EmailReadStateError::MissingDestinationMailbox)
    }

    async fn gmail_access_token(
        &self,
        account_id: &str,
    ) -> Result<ResolvedSecret, EmailReadStateError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(account_id, ProviderAccountSecretPurpose::OauthToken)
            .await
            .map_err(|error| EmailReadStateError::Credential(error.to_string()))?
            .ok_or(EmailReadStateError::MissingCredential)?;
        EmailAccountSetupService::new_with_host_vault_for_token_refresh(
            self.pool.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            self.vault.clone(),
        )
        .refresh_gmail_access_token(&binding.secret_ref)
        .await
        .map_err(|error| EmailReadStateError::Credential(error.to_string()))
    }
}

pub(crate) fn gmail_label_ids_for_mutation(
    mutation: EmailProviderMessageMutation<'_>,
) -> (Vec<&str>, Vec<&str>) {
    match mutation {
        EmailProviderMessageMutation::SetRead(true) => (vec![], vec!["UNREAD"]),
        EmailProviderMessageMutation::SetRead(false) => (vec!["UNREAD"], vec![]),
        EmailProviderMessageMutation::SetImportant(true) => (vec!["IMPORTANT"], vec![]),
        EmailProviderMessageMutation::SetImportant(false) => (vec![], vec!["IMPORTANT"]),
        EmailProviderMessageMutation::SetStarred(true) => (vec!["STARRED"], vec![]),
        EmailProviderMessageMutation::SetStarred(false) => (vec![], vec!["STARRED"]),
        EmailProviderMessageMutation::Archive { .. } => (vec![], vec!["INBOX"]),
        EmailProviderMessageMutation::Trash { .. } => (vec!["TRASH"], vec!["INBOX", "SPAM"]),
        EmailProviderMessageMutation::MarkSpam { .. } => (vec!["SPAM"], vec!["INBOX", "TRASH"]),
        EmailProviderMessageMutation::UnmarkSpam { .. } => (vec!["INBOX"], vec!["SPAM"]),
        EmailProviderMessageMutation::AddLabel(label)
        | EmailProviderMessageMutation::CopyTo(label) => (vec![label], vec![]),
        EmailProviderMessageMutation::RemoveLabel(label) => (vec![], vec![label]),
        EmailProviderMessageMutation::MoveTo(label) => (vec![label], vec!["INBOX"]),
    }
}

fn required_destination_mailbox(mailbox: Option<&str>) -> Result<&str, EmailReadStateError> {
    optional_destination_mailbox(mailbox).ok_or(EmailReadStateError::MissingDestinationMailbox)
}

fn optional_destination_mailbox(mailbox: Option<&str>) -> Option<&str> {
    mailbox.map(str::trim).filter(|value| !value.is_empty())
}

#[derive(Eq, PartialEq)]
struct ImapWriteAccountConfig {
    host: String,
    port: u16,
    tls: bool,
    username: String,
    mailbox: String,
}

fn imap_write_config(
    account: &ProviderAccount,
    message_metadata: &Value,
) -> Result<ImapWriteAccountConfig, EmailReadStateError> {
    let config = account
        .config
        .as_object()
        .ok_or(EmailReadStateError::InvalidConfig)?;
    let mailbox = message_metadata
        .get("mailbox")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| config.get("mailbox").and_then(Value::as_str))
        .ok_or(EmailReadStateError::InvalidConfig)?;
    Ok(ImapWriteAccountConfig {
        host: config_string(config, "host")?,
        port: config
            .get("port")
            .and_then(Value::as_u64)
            .filter(|value| *value > 0 && *value <= u64::from(u16::MAX))
            .map(|value| value as u16)
            .ok_or(EmailReadStateError::InvalidConfig)?,
        tls: config.get("tls").and_then(Value::as_bool).unwrap_or(true),
        username: config_string(config, "username")?,
        mailbox: mailbox.trim().to_owned(),
    })
}

pub(crate) fn imap_source_mailbox(
    account: &ProviderAccount,
    message_metadata: &Value,
) -> Result<String, EmailReadStateError> {
    imap_write_config(account, message_metadata).map(|config| config.mailbox)
}

fn imap_uid(message_metadata: &Value) -> Result<u32, EmailReadStateError> {
    message_metadata
        .get("uid")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0 && *value <= u64::from(u32::MAX))
        .map(|value| value as u32)
        .ok_or(EmailReadStateError::MissingImapUid)
}

fn imap_uid_validity(message_metadata: &Value) -> Option<u32> {
    message_metadata
        .get("uid_validity")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
}

fn config_string(
    config: &serde_json::Map<String, Value>,
    key: &'static str,
) -> Result<String, EmailReadStateError> {
    config
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or(EmailReadStateError::InvalidConfig)
}

fn gmail_api_base_url<'a>(config: &'a Value, fallback: &'a str) -> &'a str {
    config
        .get("gmail_api_base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
}

#[derive(Debug, Error)]
pub enum EmailReadStateError {
    #[error("provider does not support email read-state synchronization: {0}")]
    UnsupportedProvider(&'static str),
    #[error("provider account configuration is incomplete")]
    InvalidConfig,
    #[error("IMAP message metadata does not contain a UID")]
    MissingImapUid,
    #[error("provider mutation requires a destination mailbox or label")]
    MissingDestinationMailbox,
    #[error("provider credential is unavailable")]
    MissingCredential,
    #[error("provider credential could not be resolved: {0}")]
    Credential(String),
    #[error("Gmail read-state update failed: {0}")]
    Gmail(EmailProviderNetworkError),
    #[error("IMAP mailbox discovery failed: {0}")]
    ImapDiscovery(EmailProviderNetworkError),
    #[error("IMAP read-state update failed: {0}")]
    Imap(ImapWriteError),
}

impl EmailReadStateError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::MissingCredential | Self::Credential(_) => true,
            Self::Gmail(error) | Self::ImapDiscovery(error) => {
                provider_network_error_is_retryable(error)
            }
            Self::Imap(error) => !matches!(
                error,
                ImapWriteError::InvalidUidSet
                    | ImapWriteError::InvalidFlag
                    | ImapWriteError::InvalidMailbox
                    | ImapWriteError::UidValidityMismatch
            ),
            Self::UnsupportedProvider(_)
            | Self::InvalidConfig
            | Self::MissingImapUid
            | Self::MissingDestinationMailbox => false,
        }
    }
}

fn provider_network_error_is_retryable(error: &EmailProviderNetworkError) -> bool {
    match error {
        EmailProviderNetworkError::InvalidProviderRequest { .. } => false,
        EmailProviderNetworkError::Http(error) => error.status().is_none_or(|status| {
            status.is_server_error()
                || status == reqwest::StatusCode::REQUEST_TIMEOUT
                || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        }),
        EmailProviderNetworkError::InvalidProviderResponse { .. }
        | EmailProviderNetworkError::MissingProviderField { .. }
        | EmailProviderNetworkError::UnexpectedProviderResponse { .. }
        | EmailProviderNetworkError::ProviderTimeout { .. }
        | EmailProviderNetworkError::Io(_)
        | EmailProviderNetworkError::Tls(_)
        | EmailProviderNetworkError::Imap(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        EmailProviderMessageMutation, EmailReadStateError, gmail_label_ids_for_mutation, imap_uid,
        imap_write_config,
    };
    use crate::integrations::mail::gmail::client::errors::EmailProviderNetworkError;
    use crate::integrations::mail::imap_write::ImapWriteError;
    use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};

    #[test]
    fn imap_write_uses_the_message_mailbox_and_uid() {
        let account = ProviderAccount {
            account_id: "account-1".to_owned(),
            provider_kind: CommunicationProviderKind::Icloud,
            display_name: "iCloud".to_owned(),
            external_account_id: "owner@example.test".to_owned(),
            config: json!({
                "host": "imap.example.test",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "owner@example.test"
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let metadata = json!({ "mailbox": "Archive", "uid": 42 });

        let config = imap_write_config(&account, &metadata).expect("config");
        assert_eq!(config.mailbox, "Archive");
        assert_eq!(imap_uid(&metadata).expect("uid"), 42);
    }

    #[test]
    fn only_recoverable_provider_failures_are_retried() {
        assert!(EmailReadStateError::MissingCredential.is_retryable());
        assert!(EmailReadStateError::Credential("locked".to_owned()).is_retryable());
        assert!(!EmailReadStateError::InvalidConfig.is_retryable());
        assert!(!EmailReadStateError::MissingImapUid.is_retryable());
        assert!(!EmailReadStateError::MissingDestinationMailbox.is_retryable());
        assert!(!EmailReadStateError::UnsupportedProvider("mail").is_retryable());
        assert!(
            !EmailReadStateError::Gmail(EmailProviderNetworkError::InvalidProviderRequest {
                field: "message_id",
                message: "must not be empty",
            })
            .is_retryable()
        );
        assert!(!EmailReadStateError::Imap(ImapWriteError::InvalidMailbox).is_retryable());
    }

    #[test]
    fn gmail_not_spam_returns_message_to_inbox_and_removes_spam_label() {
        let (add_labels, remove_labels) =
            gmail_label_ids_for_mutation(EmailProviderMessageMutation::UnmarkSpam {
                destination_mailbox: None,
            });

        assert_eq!(add_labels, vec!["INBOX"]);
        assert_eq!(remove_labels, vec!["SPAM"]);
    }

    #[test]
    fn gmail_starred_mutation_uses_the_starred_system_label() {
        let (add_labels, remove_labels) =
            gmail_label_ids_for_mutation(EmailProviderMessageMutation::SetStarred(true));
        assert_eq!(add_labels, vec!["STARRED"]);
        assert!(remove_labels.is_empty());

        let (add_labels, remove_labels) =
            gmail_label_ids_for_mutation(EmailProviderMessageMutation::SetStarred(false));
        assert!(add_labels.is_empty());
        assert_eq!(remove_labels, vec!["STARRED"]);
    }
}

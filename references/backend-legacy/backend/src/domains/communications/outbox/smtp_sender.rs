use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use makosh_communications_api::email::{OutgoingEmail, SmtpConfig};
use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::communications::credentials::{
    ProviderCredentialError, ProviderCredentialReader,
};
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};

use crate::platform::communications::SmtpTransport;
use crate::platform::secrets::resolver::SecretResolver;

use super::CommunicationOutboxItem;
use super::attachments::load_sendable_attachments;
use super::delivery::{OutboxDeliveryError, OutboxEmailSender, OutboxSendReceipt};
use super::rfc822_message_id_for_outbox;

const ICLOUD_SMTP_HOST: &str = "smtp.mail.me.com";
const ICLOUD_SMTP_PORT: u16 = 587;

#[derive(Clone)]
pub struct SmtpOutboxEmailSender<R, T> {
    pool: PgPool,
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    secret_store: SecretReferenceStore,
    resolver: R,
    transport: T,
}

impl<R, T> SmtpOutboxEmailSender<R, T>
where
    R: SecretResolver,
    T: SmtpTransport,
{
    pub fn new(pool: PgPool, resolver: R, transport: T) -> Self {
        Self {
            pool: pool.clone(),
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            secret_store: SecretReferenceStore::new(pool),
            resolver,
            transport,
        }
    }
}

impl<R, T> OutboxEmailSender for SmtpOutboxEmailSender<R, T>
where
    R: SecretResolver + Send + Sync,
    T: SmtpTransport,
{
    fn send<'a>(
        &'a self,
        item: &'a CommunicationOutboxItem,
    ) -> Pin<Box<dyn Future<Output = Result<OutboxSendReceipt, OutboxDeliveryError>> + Send + 'a>>
    {
        Box::pin(async move {
            let account = self
                .provider_account_store
                .get(&item.account_id)
                .await
                .map_err(|error| delivery_error("provider account lookup failed", error))?
                .ok_or_else(|| {
                    OutboxDeliveryError::Transport("provider account was not found".to_owned())
                })?;
            let smtp_config = smtp_config_for_provider_account(&account)?;
            let credential_reader = ProviderCredentialReader::new(
                self.provider_secret_binding_store.clone(),
                self.secret_store.clone(),
                &self.resolver,
            );
            let credential = match credential_reader
                .read(
                    &account.account_id,
                    ProviderAccountSecretPurpose::SmtpPassword,
                )
                .await
            {
                Ok(credential) => credential,
                Err(error) if should_try_imap_credential_for_smtp(&account, &error) => {
                    tracing::warn!(
                        error = %error,
                        "SMTP credential unavailable; trying IMAP credential for SMTP"
                    );
                    credential_reader
                        .read(
                            &account.account_id,
                            ProviderAccountSecretPurpose::ImapPassword,
                        )
                        .await
                        .map_err(|error| {
                            delivery_error("SMTP credential is unavailable for this account", error)
                        })?
                }
                Err(error) => {
                    return Err(delivery_error(
                        "SMTP credential is unavailable for this account",
                        error,
                    ));
                }
            };
            let mut email = outgoing_email_from_outbox_item(item, &account);
            email.attachments = load_sendable_attachments(&self.pool, &item.outbox_id).await?;
            let result = self
                .transport
                .send(&smtp_config, &credential.secret, &email)
                .await
                .map_err(|error| delivery_error("SMTP send failed", error))?;

            Ok(OutboxSendReceipt {
                provider_message_id: result.message_id,
                accepted_recipients: result.accepted_recipients,
            })
        })
    }
}

pub fn smtp_config_for_provider_account(
    account: &ProviderAccount,
) -> Result<SmtpConfig, OutboxDeliveryError> {
    match account.provider_kind {
        CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap => {}
        CommunicationProviderKind::Gmail => {
            return Err(OutboxDeliveryError::Transport(
                "Gmail send is unavailable until OAuth send scopes are configured".to_owned(),
            ));
        }
        _ => {
            return Err(OutboxDeliveryError::Transport(
                "provider does not support SMTP send".to_owned(),
            ));
        }
    }

    let config = account.config.as_object();
    let host = config
        .and_then(|config| config.get("smtp_host"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| default_smtp_host(account.provider_kind).map(ToOwned::to_owned))
        .ok_or_else(|| {
            OutboxDeliveryError::Transport("SMTP config is unavailable for this account".to_owned())
        })?;
    let port = config
        .and_then(|config| config.get("smtp_port"))
        .and_then(Value::as_u64)
        .filter(|value| *value > 0 && *value <= u64::from(u16::MAX))
        .map(|value| value as u16)
        .or_else(|| default_smtp_port(account.provider_kind))
        .ok_or_else(|| {
            OutboxDeliveryError::Transport("SMTP port is unavailable for this account".to_owned())
        })?;
    let username = config
        .and_then(|config| config.get("smtp_username"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(account.external_account_id.as_str());
    let tls = config
        .and_then(|config| config.get("smtp_tls"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let starttls = config
        .and_then(|config| config.get("smtp_starttls"))
        .and_then(Value::as_bool)
        .unwrap_or_else(|| default_smtp_starttls(account.provider_kind));

    Ok(SmtpConfig::new(host, port, tls, username).starttls(starttls))
}

fn default_smtp_host(provider_kind: CommunicationProviderKind) -> Option<&'static str> {
    match provider_kind {
        CommunicationProviderKind::Icloud => Some(ICLOUD_SMTP_HOST),
        _ => None,
    }
}

fn default_smtp_port(provider_kind: CommunicationProviderKind) -> Option<u16> {
    match provider_kind {
        CommunicationProviderKind::Icloud => Some(ICLOUD_SMTP_PORT),
        _ => None,
    }
}

fn default_smtp_starttls(provider_kind: CommunicationProviderKind) -> bool {
    matches!(provider_kind, CommunicationProviderKind::Icloud)
}

fn should_try_imap_credential_for_smtp(
    account: &ProviderAccount,
    error: &ProviderCredentialError,
) -> bool {
    matches!(
        account.provider_kind,
        CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap
    ) && matches!(
        error,
        ProviderCredentialError::MissingBinding { .. }
            | ProviderCredentialError::MissingSecretReference { .. }
    )
}

pub fn outgoing_email_from_outbox_item(
    item: &CommunicationOutboxItem,
    account: &ProviderAccount,
) -> OutgoingEmail {
    OutgoingEmail {
        from: account.external_account_id.clone(),
        message_id: optional_metadata_string(&item.metadata, "rfc822_message_id")
            .or_else(|| Some(rfc822_message_id_for_outbox(&item.outbox_id))),
        to: item.to_recipients.clone(),
        cc: item.cc_recipients.clone(),
        bcc: item.bcc_recipients.clone(),
        subject: item.subject.clone(),
        body_text: item.body_text.clone(),
        body_html: item.body_html.clone(),
        in_reply_to: optional_metadata_string(&item.metadata, "in_reply_to"),
        references: metadata_string_array(&item.metadata, "references"),
        attachments: Vec::new(),
    }
}

fn optional_metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn metadata_string_array(metadata: &Value, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn delivery_error(
    public_message: &'static str,
    error: impl std::fmt::Display,
) -> OutboxDeliveryError {
    tracing::warn!(error = %error, "outbox SMTP delivery failed");
    OutboxDeliveryError::Transport(public_message.to_owned())
}

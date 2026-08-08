use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use sqlx::postgres::PgPool;

use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};

use crate::platform::communications::SmtpTransport;
use crate::platform::secrets::resolver::SecretResolver;
use makosh_communications_api::email::{GmailOutboxSendRequest, GmailOutboxTransport};

use super::CommunicationOutboxItem;
use super::attachments::load_sendable_attachments;
use super::delivery::{OutboxDeliveryError, OutboxEmailSender, OutboxSendReceipt};
use super::smtp_sender::SmtpOutboxEmailSender;
use super::smtp_sender::outgoing_email_from_outbox_item;

#[derive(Clone)]
pub struct CommunicationOutboxEmailSender<R, T, G> {
    pool: PgPool,
    provider_account_store: CommunicationProviderAccountStore,
    provider_secret_binding_store: CommunicationProviderSecretBindingStore,
    smtp_sender: SmtpOutboxEmailSender<R, T>,
    gmail_transport: G,
}

impl<R, T, G> CommunicationOutboxEmailSender<R, T, G>
where
    R: SecretResolver,
    T: SmtpTransport,
    G: GmailOutboxTransport,
{
    pub fn new(pool: PgPool, resolver: R, smtp_transport: T, gmail_transport: G) -> Self {
        Self {
            pool: pool.clone(),
            provider_account_store: CommunicationProviderAccountStore::new(pool.clone()),
            provider_secret_binding_store: CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
            smtp_sender: SmtpOutboxEmailSender::new(pool, resolver, smtp_transport),
            gmail_transport,
        }
    }
}

impl<R, T, G> OutboxEmailSender for CommunicationOutboxEmailSender<R, T, G>
where
    R: SecretResolver + Send + Sync,
    T: SmtpTransport,
    G: GmailOutboxTransport,
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

            if matches!(account.provider_kind, CommunicationProviderKind::Gmail)
                && gmail_send_enabled(&account.config)
            {
                return self.send_gmail(item, &account).await;
            }

            self.smtp_sender.send(item).await
        })
    }
}

impl<R, T, G> CommunicationOutboxEmailSender<R, T, G>
where
    R: SecretResolver,
    T: SmtpTransport,
    G: GmailOutboxTransport,
{
    async fn send_gmail(
        &self,
        item: &CommunicationOutboxItem,
        account: &ProviderAccount,
    ) -> Result<OutboxSendReceipt, OutboxDeliveryError> {
        let binding = self
            .provider_secret_binding_store
            .get_for_account(
                &account.account_id,
                ProviderAccountSecretPurpose::OauthToken,
            )
            .await
            .map_err(|error| delivery_error("Gmail OAuth credential lookup failed", error))?
            .ok_or_else(|| {
                OutboxDeliveryError::Transport(
                    "Gmail OAuth credential is unavailable for this account".to_owned(),
                )
            })?;
        let mut email = outgoing_email_from_outbox_item(item, account);
        email.attachments = load_sendable_attachments(&self.pool, &item.outbox_id).await?;
        let result = self
            .gmail_transport
            .send(GmailOutboxSendRequest {
                account_id: &account.account_id,
                oauth_secret_ref: &binding.secret_ref,
                api_base_url: gmail_api_base_url(&account.config),
                email: &email,
            })
            .await
            .map_err(|error| delivery_error("Gmail API send failed", error))?;

        Ok(OutboxSendReceipt {
            provider_message_id: result.message_id,
            accepted_recipients: result.accepted_recipients,
        })
    }
}

fn gmail_send_enabled(config: &Value) -> bool {
    config
        .get("gmail_send_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn gmail_api_base_url(config: &Value) -> &str {
    config
        .get("gmail_api_base_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("https://www.googleapis.com")
}

fn delivery_error(
    public_message: &'static str,
    error: impl std::fmt::Display,
) -> OutboxDeliveryError {
    tracing::warn!(error = %error, "provider outbox delivery failed");
    OutboxDeliveryError::Transport(public_message.to_owned())
}

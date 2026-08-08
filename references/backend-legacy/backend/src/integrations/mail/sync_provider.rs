use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::ProviderSecretBindingLookupPort;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::json;
use sqlx::postgres::PgPool;

use crate::integrations::mail::accounts::service::EmailAccountSetupService;
use crate::integrations::mail::gmail::client::{
    errors::EmailProviderNetworkError,
    gmail_api::GmailApiClient,
    imap::{ImapIdleOutcome, ImapNetworkClient},
    options::{
        GmailFetchOptions, GmailHistoryFetchOptions, ImapFetchOptions, ImapIdleOptions,
        ImapMailboxListOptions,
    },
};
use crate::platform::secrets::models::ResolvedSecret;
use crate::platform::secrets::resolver::SecretResolver;
use crate::vault::HostVault;
use makosh_communications_api::email_sync::EmailSyncBatch;
use makosh_communications_api::mail_resources::{
    DiscoveredMailProviderResource, EmailProviderSyncError, EmailProviderSyncPort,
    GmailHistoryFetchRequest, GmailMessageListFetchRequest, GmailResourceDiscoveryRequest,
    ImapIdleWaitOutcome, ImapIdleWaitRequest, ImapMailboxListRequest, ImapMessageFetchRequest,
    MailProviderResourceKind, MailProviderSemanticRole,
};

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

pub(crate) async fn read_provider_secret(
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

    fn list_imap_mailboxes<'a>(
        &'a self,
        request: ImapMailboxListRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, EmailProviderSyncError>> + Send + 'a>>
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
            let options = ImapMailboxListOptions::new(
                &request.host,
                request.port,
                request.tls,
                &request.username,
            );
            ImapNetworkClient::new()
                .list_mailboxes(&credential, &options)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))
        })
    }

    fn discover_gmail_resources<'a>(
        &'a self,
        request: GmailResourceDiscoveryRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<DiscoveredMailProviderResource>, EmailProviderSyncError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let access_token = self.gmail_access_token(&request.account_id).await?;
            GmailApiClient::new(&self.gmail_api_base_url)
                .user_id("me")
                .list_labels(&access_token)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))
        })
    }

    fn discover_imap_resources<'a>(
        &'a self,
        request: ImapMailboxListRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<DiscoveredMailProviderResource>, EmailProviderSyncError>>
                + Send
                + 'a,
        >,
    > {
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
            let options = ImapMailboxListOptions::new(
                &request.host,
                request.port,
                request.tls,
                &request.username,
            );
            let mailboxes = ImapNetworkClient::new()
                .discover_mailboxes(&credential, &options)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))?;
            Ok(mailboxes
                .into_iter()
                .map(|mailbox| {
                    let role_names = mailbox
                        .roles
                        .iter()
                        .map(|role| format!("{role:?}").to_ascii_lowercase())
                        .collect::<Vec<_>>();
                    DiscoveredMailProviderResource {
                        resource_kind: MailProviderResourceKind::Folder,
                        semantic_role: imap_semantic_role(&mailbox.name, &mailbox.roles),
                        provider_resource_id: mailbox.name.clone(),
                        display_name: mailbox.name,
                        selectable: true,
                        writable: true,
                        capabilities: json!({ "imap_special_use": role_names }),
                    }
                })
                .collect())
        })
    }

    fn wait_for_imap_change<'a>(
        &'a self,
        request: ImapIdleWaitRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<ImapIdleWaitOutcome, EmailProviderSyncError>> + Send + 'a>,
    > {
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
            let options = ImapIdleOptions::new(
                &request.host,
                request.port,
                request.tls,
                &request.mailbox,
                &request.username,
                request.timeout,
            );
            let outcome = ImapNetworkClient::new()
                .wait_for_change(&credential, &options)
                .await
                .map_err(|error| EmailProviderSyncError::provider_network(error, false))?;
            Ok(match outcome {
                ImapIdleOutcome::Changed => ImapIdleWaitOutcome::Changed,
                ImapIdleOutcome::TimedOut => ImapIdleWaitOutcome::TimedOut,
                ImapIdleOutcome::Unsupported => ImapIdleWaitOutcome::Unsupported,
            })
        })
    }
}

fn imap_semantic_role(
    mailbox_name: &str,
    roles: &[crate::integrations::mail::gmail::client::imap::ImapMailboxRole],
) -> Option<MailProviderSemanticRole> {
    use crate::integrations::mail::gmail::client::imap::ImapMailboxRole;

    if mailbox_name.eq_ignore_ascii_case("INBOX") {
        return Some(MailProviderSemanticRole::Inbox);
    }
    roles
        .iter()
        .map(|role| match role {
            ImapMailboxRole::All => MailProviderSemanticRole::All,
            ImapMailboxRole::Archive => MailProviderSemanticRole::Archive,
            ImapMailboxRole::Drafts => MailProviderSemanticRole::Drafts,
            ImapMailboxRole::Flagged => MailProviderSemanticRole::Flagged,
            ImapMailboxRole::Junk => MailProviderSemanticRole::Junk,
            ImapMailboxRole::Sent => MailProviderSemanticRole::Sent,
            ImapMailboxRole::Trash => MailProviderSemanticRole::Trash,
        })
        .next()
}

use makosh_communications_api::accounts::ProviderAccount;
mod gmail;
mod imap;
mod projection;
pub(super) mod summary;
pub(super) mod types;

use makosh_communications_api::email_sync::EmailSyncAdapterConfig;
use makosh_communications_api::mail_resources::{
    GmailResourceDiscoveryRequest, ImapMailboxListRequest,
};

use super::errors::ProviderSyncError;
use super::service::MailBackgroundSyncService;

use self::summary::ProviderSyncSummary;
use self::types::ImapAccountConfig;
use self::types::ProviderSyncContext;

enum MailProviderResourceDiscoveryRequest {
    Gmail(GmailResourceDiscoveryRequest),
    Imap(ImapMailboxListRequest),
}

fn resource_discovery_request(
    adapter: &EmailSyncAdapterConfig,
    account: &ProviderAccount,
) -> MailProviderResourceDiscoveryRequest {
    match adapter {
        EmailSyncAdapterConfig::Gmail { .. } => {
            MailProviderResourceDiscoveryRequest::Gmail(GmailResourceDiscoveryRequest {
                account_id: account.account_id.clone(),
            })
        }
        EmailSyncAdapterConfig::Imap {
            host, port, tls, ..
        } => MailProviderResourceDiscoveryRequest::Imap(ImapMailboxListRequest {
            account_id: account.account_id.clone(),
            host: host.clone(),
            port: *port,
            tls: *tls,
            username: account.external_account_id.clone(),
        }),
    }
}

impl MailBackgroundSyncService {
    pub(super) async fn refresh_provider_resources(
        &self,
        adapter: &EmailSyncAdapterConfig,
        account: &ProviderAccount,
    ) {
        let resources = match resource_discovery_request(adapter, account) {
            MailProviderResourceDiscoveryRequest::Gmail(request) => {
                self.provider_sync.discover_gmail_resources(request).await
            }
            MailProviderResourceDiscoveryRequest::Imap(request) => {
                self.provider_sync.discover_imap_resources(request).await
            }
        };
        let Ok(resources) = resources else {
            tracing::warn!(
                account_id = %account.account_id,
                provider_kind = account.provider_kind.as_str(),
                "mail provider resource discovery failed"
            );
            return;
        };
        if self
            .provider_resource_commands
            .record_discovered_resources(&account.account_id, &resources)
            .await
            .is_err()
        {
            tracing::warn!(
                account_id = %account.account_id,
                provider_kind = account.provider_kind.as_str(),
                "mail provider resource mapping persistence failed"
            );
        }
    }

    pub(super) async fn execute_provider_sync(
        &self,
        adapter: &EmailSyncAdapterConfig,
        context: ProviderSyncContext<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        match adapter {
            EmailSyncAdapterConfig::Gmail { .. } => self.sync_gmail(context).await,
            EmailSyncAdapterConfig::Imap {
                host,
                port,
                tls,
                mailboxes,
            } => {
                self.sync_imap(
                    context,
                    ImapAccountConfig {
                        host,
                        port: *port,
                        tls: *tls,
                        mailboxes,
                    },
                )
                .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use makosh_communications_api::email_sync::EmailSyncAdapterConfig;

    fn account(
        provider_kind: CommunicationProviderKind,
        config: serde_json::Value,
    ) -> ProviderAccount {
        ProviderAccount {
            account_id: "mail-resource-discovery".to_owned(),
            provider_kind,
            display_name: "Discovery fixture".to_owned(),
            external_account_id: "fixture@example.test".to_owned(),
            config,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn resource_discovery_request_uses_the_matching_provider_transport() {
        let gmail = resource_discovery_request(
            &EmailSyncAdapterConfig::Gmail {
                history_stream_id: "gmail:history".to_owned(),
            },
            &account(CommunicationProviderKind::Gmail, json!({})),
        );
        assert!(matches!(
            gmail,
            MailProviderResourceDiscoveryRequest::Gmail(_)
        ));

        let imap = resource_discovery_request(
            &EmailSyncAdapterConfig::Imap {
                host: "imap.example.test".to_owned(),
                port: 993,
                tls: true,
                mailboxes: vec!["INBOX".to_owned()],
            },
            &account(CommunicationProviderKind::Icloud, json!({})),
        );
        match imap {
            MailProviderResourceDiscoveryRequest::Imap(request) => {
                assert_eq!(request.host, "imap.example.test");
                assert_eq!(request.username, "fixture@example.test");
            }
            MailProviderResourceDiscoveryRequest::Gmail(_) => panic!("expected IMAP discovery"),
        }
    }
}

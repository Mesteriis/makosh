pub const MAIL_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/makosh.mail.v1.bin"));
pub const MAIL_CLIENT_CONTRACT_MAJOR: u32 = 1;
pub const MAIL_CLIENT_CONTRACT_REVISION: u32 = 14;
pub const MAIL_MODULE_ID: &str = "makosh-mail-runtime";
pub const MAIL_OWNER_ID: &str = "mail";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientContractV1 {
    AccountCatalog,
    AccountCredentialBind,
    AccountQuery,
    AccountRetire,
    AccountDelete,
    AccountLifecycleRetry,
    AccountLifecycleQuery,
    Sync,
    Delivery,
    DeliveryQuery,
    GmailOAuthStart,
    GmailOAuthComplete,
    GmailOAuthRefresh,
    GmailOAuthQuery,
    CompositionCommand,
    CompositionQuery,
    MessageFlagCommand,
    MessageFlagQuery,
    MessageLocationCommand,
    MessageLocationQuery,
    MessagePermanentDeleteCommand,
    MessagePermanentDeleteQuery,
    OperationalQuery,
    SyncHealthQuery,
}

impl MailClientContractV1 {
    pub const ALL: [Self; 24] = [
        Self::AccountCatalog,
        Self::AccountCredentialBind,
        Self::AccountDelete,
        Self::AccountLifecycleQuery,
        Self::AccountLifecycleRetry,
        Self::AccountQuery,
        Self::AccountRetire,
        Self::Delivery,
        Self::DeliveryQuery,
        Self::GmailOAuthComplete,
        Self::GmailOAuthQuery,
        Self::GmailOAuthRefresh,
        Self::GmailOAuthStart,
        Self::CompositionCommand,
        Self::CompositionQuery,
        Self::MessageFlagCommand,
        Self::MessageFlagQuery,
        Self::MessageLocationCommand,
        Self::MessageLocationQuery,
        Self::MessagePermanentDeleteCommand,
        Self::MessagePermanentDeleteQuery,
        Self::OperationalQuery,
        Self::Sync,
        Self::SyncHealthQuery,
    ];

    #[must_use]
    pub const fn capability_id(self) -> &'static str {
        match self {
            Self::AccountCatalog => "mail.account.catalog.query.v1",
            Self::AccountCredentialBind => "mail.account.credential.bind.v1",
            Self::AccountQuery => "mail.account.query.v1",
            Self::AccountRetire => "mail.account.retire.v1",
            Self::AccountDelete => "mail.account.delete.v1",
            Self::AccountLifecycleRetry => "mail.account.lifecycle.retry.v1",
            Self::AccountLifecycleQuery => "mail.account.lifecycle.query.v1",
            Self::Sync => "mail.sync.v1",
            Self::Delivery => "mail.delivery.v1",
            Self::DeliveryQuery => "mail.delivery.query.v1",
            Self::GmailOAuthStart => "mail.oauth.start.v1",
            Self::GmailOAuthComplete => "mail.oauth.complete.v1",
            Self::GmailOAuthRefresh => "mail.oauth.refresh.v1",
            Self::GmailOAuthQuery => "mail.oauth.query.v1",
            Self::CompositionCommand => "mail.composition.command.v1",
            Self::CompositionQuery => "mail.composition.query.v1",
            Self::MessageFlagCommand => "mail.message-flags.command.v1",
            Self::MessageFlagQuery => "mail.message-flags.query.v1",
            Self::MessageLocationCommand => "mail.message-location.command.v1",
            Self::MessageLocationQuery => "mail.message-location.query.v1",
            Self::MessagePermanentDeleteCommand => "mail.message-permanent-delete.command.v1",
            Self::MessagePermanentDeleteQuery => "mail.message-permanent-delete.query.v1",
            Self::OperationalQuery => "mail.operational.query.v1",
            Self::SyncHealthQuery => "mail.sync.health.query.v1",
        }
    }

    #[must_use]
    pub const fn contract_name(self) -> &'static str {
        self.capability_id()
    }

    #[must_use]
    pub const fn connect_path(self) -> &'static str {
        match self {
            Self::AccountCatalog => "/makosh.mail.account.v1.MailAccountCatalogService/List",
            Self::AccountCredentialBind => {
                "/makosh.mail.account.v1.MailAccountCredentialBindingService/Bind"
            }
            Self::AccountQuery => "/makosh.mail.account.v1.MailAccountQueryService/Get",
            Self::AccountRetire => {
                "/makosh.mail.account_lifecycle.v1.MailAccountRetireService/Retire"
            }
            Self::AccountDelete => {
                "/makosh.mail.account_lifecycle.v1.MailAccountDeleteService/Delete"
            }
            Self::AccountLifecycleRetry => {
                "/makosh.mail.account_lifecycle.v1.MailAccountLifecycleRetryService/Retry"
            }
            Self::AccountLifecycleQuery => {
                "/makosh.mail.account_lifecycle.v1.MailAccountLifecycleStatusService/Get"
            }
            Self::Sync => "/makosh.mail.v1.MailSyncService/Sync",
            Self::Delivery => "/makosh.mail.v1.MailDeliveryCommandService/Send",
            Self::DeliveryQuery => "/makosh.mail.v1.MailDeliveryQueryService/GetOperationStatus",
            Self::GmailOAuthStart => "/makosh.mail.v1.GmailOAuthStartService/Start",
            Self::GmailOAuthComplete => "/makosh.mail.v1.GmailOAuthCompleteService/Complete",
            Self::GmailOAuthRefresh => "/makosh.mail.v1.GmailOAuthRefreshService/Refresh",
            Self::GmailOAuthQuery => "/makosh.mail.v1.GmailOAuthQueryService/GetOperationStatus",
            Self::CompositionCommand => {
                "/makosh.mail.composition.v1.MailCompositionCommandService/Mutate"
            }
            Self::CompositionQuery => {
                "/makosh.mail.composition.v1.MailCompositionQueryService/Query"
            }
            Self::MessageFlagCommand => {
                "/makosh.mail.message_flags.v1.MailMessageFlagCommandService/Mutate"
            }
            Self::MessageFlagQuery => {
                "/makosh.mail.message_flags.v1.MailMessageFlagQueryService/GetOperationStatus"
            }
            Self::MessageLocationCommand => {
                "/makosh.mail.message_location.v1.MailMessageLocationCommandService/Mutate"
            }
            Self::MessageLocationQuery => {
                "/makosh.mail.message_location.v1.MailMessageLocationQueryService/GetOperationStatus"
            }
            Self::MessagePermanentDeleteCommand => {
                "/makosh.mail.message_permanent_delete.v1.MailMessagePermanentDeleteCommandService/Mutate"
            }
            Self::MessagePermanentDeleteQuery => {
                "/makosh.mail.message_permanent_delete.v1.MailMessagePermanentDeleteQueryService/GetOperationStatus"
            }
            Self::OperationalQuery => {
                "/makosh.mail.operational.v1.MailOperationalQueryService/Query"
            }
            Self::SyncHealthQuery => "/makosh.mail.sync_health.v1.MailSyncHealthQueryService/Query",
        }
    }

    #[must_use]
    pub fn from_contract_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|contract| contract.contract_name() == name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn client_contracts_have_unique_capabilities_names_and_routes() {
        assert!(!MAIL_CLIENT_DESCRIPTOR_SET_V1.is_empty());
        assert_eq!(
            MailClientContractV1::ALL
                .into_iter()
                .map(MailClientContractV1::capability_id)
                .collect::<BTreeSet<_>>()
                .len(),
            MailClientContractV1::ALL.len()
        );
        assert_eq!(
            MailClientContractV1::ALL
                .into_iter()
                .map(MailClientContractV1::connect_path)
                .collect::<BTreeSet<_>>()
                .len(),
            MailClientContractV1::ALL.len()
        );
    }

    #[test]
    fn umbrella_contract_is_not_a_route_identity() {
        assert_eq!(
            MailClientContractV1::from_contract_name("mail.client"),
            None
        );
    }
}

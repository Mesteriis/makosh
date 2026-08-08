//! Mail integration contract crate for ADR-0239.

pub const PACKAGE: &str = "makosh-mail-api";

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.v1.rs"));
}
pub mod account;
pub mod account_lifecycle;
pub mod account_lifecycle_wire;
pub mod account_lifecycle_wire_generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.mail.account_lifecycle.v1.rs"
    ));
}
pub mod account_wire;
pub mod account_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.account.v1.rs"));
}
pub mod client_contract;
pub mod client_wire;
pub mod composition;
pub mod composition_wire;
pub mod composition_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.composition.v1.rs"));
}
pub mod message_flags;
pub mod message_flags_wire;
pub mod message_flags_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.message_flags.v1.rs"));
}
pub mod message_location;
pub mod message_location_wire;
pub mod message_location_wire_generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.mail.message_location.v1.rs"
    ));
}
pub mod message_permanent_delete;
pub mod message_permanent_delete_wire;
pub mod message_permanent_delete_wire_generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/makosh.mail.message_permanent_delete.v1.rs"
    ));
}
pub mod oauth;
pub mod oauth_wire;
pub mod operational;
pub mod operational_wire;
pub mod operational_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.operational.v1.rs"));
}
pub mod sync_health;
pub mod sync_health_wire;
pub mod sync_health_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.sync_health.v1.rs"));
}
pub mod portability;
pub mod portability_wire_generated {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail.portability.v1.rs"));
}

pub use oauth::{
    GMAIL_OAUTH_ATTEMPT_TTL_SECONDS, GMAIL_OAUTH_AUTHORIZATION_HOST,
    GMAIL_OAUTH_AUTHORIZATION_PATH, GMAIL_OAUTH_HTTPS_PORT, GMAIL_OAUTH_TOKEN_HOST,
    GMAIL_OAUTH_TOKEN_PATH, GmailOAuthAuthorityV1, GmailOAuthCompleteRequestV1,
    GmailOAuthConfigurationV1, GmailOAuthEndpointV1, GmailOAuthOperationKindV1,
    GmailOAuthOperationStatusV1, GmailOAuthOutcomeV1, GmailOAuthRefreshRequestV1,
    GmailOAuthStartRequestV1, GmailOAuthStartedV1, GmailOAuthStatusRequestV1,
    valid_gmail_oauth_configuration,
};
pub use portability::{
    MAIL_ACCOUNT_EXPORT_MAJOR_V1, MAIL_SETTINGS_SCHEMA_MAJOR_V2, MAIL_SETTINGS_SCHEMA_REVISION_V2,
    MailAccountExportValidationErrorV1, validate_mail_account_export_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailClientRequestV1 {
    BindCredential(account::MailBindCredentialRequestV1),
    AccountCatalog(account::MailAccountCatalogRequestV1),
    AccountStatus(account::MailAccountStatusRequestV1),
    RetireAccount(account_lifecycle::MailAccountLifecycleCommandV1),
    DeleteAccount(account_lifecycle::MailAccountLifecycleCommandV1),
    RetryAccountLifecycle(account_lifecycle::MailAccountLifecycleRetryV1),
    AccountLifecycleStatus(account_lifecycle::MailAccountLifecycleStatusRequestV1),
    SyncInbox(MailSyncInboxRequestV1),
    SendMail(MailSendMailRequestV1),
    DeliveryStatus(MailDeliveryStatusRequestV1),
    GmailOAuthStart(GmailOAuthStartRequestV1),
    GmailOAuthComplete(GmailOAuthCompleteRequestV1),
    GmailOAuthRefresh(GmailOAuthRefreshRequestV1),
    GmailOAuthStatus(GmailOAuthStatusRequestV1),
    CompositionCommand(composition::MailCompositionCommandV1),
    CompositionQuery(composition::MailCompositionQueryV1),
    MessageFlagCommand(message_flags::MailMessageFlagCommandV1),
    MessageFlagStatus(message_flags::MailMessageFlagStatusRequestV1),
    MessageLocationCommand(message_location::MailMessageLocationCommandV1),
    MessageLocationStatus(message_location::MailMessageLocationStatusRequestV1),
    MessagePermanentDeleteCommand(message_permanent_delete::MailMessagePermanentDeleteCommandV1),
    MessagePermanentDeleteStatus(
        message_permanent_delete::MailMessagePermanentDeleteStatusRequestV1,
    ),
    OperationalQuery(operational::MailOperationalQueryV1),
    SyncHealthQuery(sync_health::MailSyncHealthQueryV1),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncInboxRequestV1 {
    pub operation_id: String,
    pub connection_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSendMailRequestV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub provider_conversation_id: String,
    pub recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub text_body: String,
    pub attachment_anchor_ids: Vec<[u8; 16]>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDeliveryStatusRequestV1 {
    pub operation_id: String,
    pub connection_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailClientResponseV1 {
    CredentialBinding(account::MailCredentialBindingReceiptV1),
    AccountCatalog(account::MailAccountCatalogV1),
    AccountStatus(account::MailAccountStatusV1),
    AccountLifecycle(account_lifecycle::MailAccountLifecycleReceiptV1),
    SyncInboxAccepted {
        operation_id: String,
    },
    MailAccepted {
        operation_id: String,
    },
    DeliveryStatus(Option<MailDeliveryOperationStatusV1>),
    GmailOAuthStarted(GmailOAuthStartedV1),
    GmailOAuthAccepted {
        operation_id: String,
    },
    GmailOAuthStatus(Option<GmailOAuthOperationStatusV1>),
    CompositionMutation(composition::MailCompositionMutationReceiptV1),
    CompositionQuery(composition::MailCompositionQueryResponseV1),
    MessageFlagAccepted(message_flags::MailMessageFlagAcceptedV1),
    MessageFlagStatus(Option<message_flags::MailMessageFlagOperationStatusV1>),
    MessageLocationAccepted(message_location::MailMessageLocationAcceptedV1),
    MessageLocationStatus(Option<message_location::MailMessageLocationOperationStatusV1>),
    MessagePermanentDeleteAccepted(message_permanent_delete::MailMessagePermanentDeleteAcceptedV1),
    MessagePermanentDeleteStatus(
        Option<message_permanent_delete::MailMessagePermanentDeleteOperationStatusV1>,
    ),
    OperationalQuery(operational::MailOperationalQueryResponseV1),
    SyncHealthQuery(sync_health::MailSyncHealthQueryResponseV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailDeliveryOutcomeV1 {
    Pending,
    Accepted,
    Rejected,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDeliveryOperationStatusV1 {
    pub operation_id: String,
    pub connection_id: String,
    pub outcome: MailDeliveryOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
    pub response_code: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountConfigurationV1 {
    pub connection_id: String,
    pub inbound: MailInboundTransportV1,
    pub sync_window: u32,
    pub sync_windows: u32,
    pub smtp_endpoint: Option<SmtpEndpointV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookProviderV1 {
    None,
    GooglePeople,
    IcloudCardDav,
}

pub const GOOGLE_PEOPLE_API_HOST_V1: &str = "people.googleapis.com";
pub const GOOGLE_PEOPLE_API_PORT_V1: u16 = 443;
pub const ICLOUD_CARDDAV_HOST_V1: &str = "contacts.icloud.com";
pub const ICLOUD_CARDDAV_PORT_V1: u16 = 443;
pub const ICLOUD_CARDDAV_BASE_PATH_V1: &str = "/";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAddressBookTlsEndpointV1 {
    pub host: String,
    pub port: u16,
    pub ca_certificate_pem: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailCardDavEndpointV1 {
    pub tls: MailAddressBookTlsEndpointV1,
    pub base_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAddressBookConfigurationV1 {
    pub provider: MailAddressBookProviderV1,
    pub carddav_username: Option<String>,
    pub google_people_endpoint: Option<MailAddressBookTlsEndpointV1>,
    pub carddav_endpoint: Option<MailCardDavEndpointV1>,
}

pub fn valid_address_book_configuration(
    configuration: &MailAddressBookConfigurationV1,
    inbound: &MailInboundTransportV1,
) -> bool {
    valid_address_book_configuration_for_authority_v1(
        configuration,
        inbound,
        AddressBookEndpointAuthorityV1::Production,
    )
}

#[cfg(feature = "conformance-test-support")]
pub fn valid_address_book_configuration_for_conformance_v1(
    configuration: &MailAddressBookConfigurationV1,
    inbound: &MailInboundTransportV1,
) -> bool {
    valid_address_book_configuration_for_authority_v1(
        configuration,
        inbound,
        AddressBookEndpointAuthorityV1::LoopbackConformance,
    )
}

#[derive(Clone, Copy)]
enum AddressBookEndpointAuthorityV1 {
    Production,
    #[cfg(feature = "conformance-test-support")]
    LoopbackConformance,
}

fn valid_address_book_configuration_for_authority_v1(
    configuration: &MailAddressBookConfigurationV1,
    inbound: &MailInboundTransportV1,
    authority: AddressBookEndpointAuthorityV1,
) -> bool {
    match configuration.provider {
        MailAddressBookProviderV1::None => {
            configuration.carddav_username.is_none()
                && configuration.google_people_endpoint.is_none()
                && configuration.carddav_endpoint.is_none()
        }
        MailAddressBookProviderV1::GooglePeople => {
            matches!(inbound, MailInboundTransportV1::Gmail(_))
                && configuration.carddav_username.is_none()
                && configuration.carddav_endpoint.is_none()
                && configuration
                    .google_people_endpoint
                    .as_ref()
                    .is_some_and(|endpoint| {
                        valid_google_people_endpoint_for_authority_v1(endpoint, authority)
                    })
        }
        MailAddressBookProviderV1::IcloudCardDav => {
            matches!(inbound, MailInboundTransportV1::Imap(_))
                && configuration.google_people_endpoint.is_none()
                && configuration
                    .carddav_username
                    .as_deref()
                    .is_some_and(|value| {
                        !value.trim().is_empty()
                            && value.len() <= 256
                            && !value.chars().any(char::is_control)
                    })
                && configuration
                    .carddav_endpoint
                    .as_ref()
                    .is_some_and(|endpoint| {
                        valid_carddav_endpoint_for_authority_v1(endpoint, authority)
                    })
        }
    }
}

#[must_use]
pub fn valid_google_people_endpoint_v1(endpoint: &MailAddressBookTlsEndpointV1) -> bool {
    valid_google_people_endpoint_for_authority_v1(
        endpoint,
        AddressBookEndpointAuthorityV1::Production,
    )
}

#[must_use]
pub fn valid_carddav_endpoint_v1(endpoint: &MailCardDavEndpointV1) -> bool {
    valid_carddav_endpoint_for_authority_v1(endpoint, AddressBookEndpointAuthorityV1::Production)
}

fn valid_google_people_endpoint_for_authority_v1(
    endpoint: &MailAddressBookTlsEndpointV1,
    authority: AddressBookEndpointAuthorityV1,
) -> bool {
    valid_address_book_tls_endpoint_for_authority_v1(
        endpoint,
        GOOGLE_PEOPLE_API_HOST_V1,
        GOOGLE_PEOPLE_API_PORT_V1,
        authority,
    )
}

fn valid_carddav_endpoint_for_authority_v1(
    endpoint: &MailCardDavEndpointV1,
    authority: AddressBookEndpointAuthorityV1,
) -> bool {
    valid_address_book_tls_endpoint_for_authority_v1(
        &endpoint.tls,
        ICLOUD_CARDDAV_HOST_V1,
        ICLOUD_CARDDAV_PORT_V1,
        authority,
    ) && !endpoint.base_path.is_empty()
        && endpoint.base_path.len() <= 2_048
        && endpoint.base_path.starts_with('/')
        && !endpoint.base_path.contains(['\r', '\n'])
}

fn valid_address_book_tls_endpoint_for_authority_v1(
    endpoint: &MailAddressBookTlsEndpointV1,
    production_host: &str,
    production_port: u16,
    authority: AddressBookEndpointAuthorityV1,
) -> bool {
    if !valid_host(&endpoint.host)
        || endpoint
            .ca_certificate_pem
            .as_deref()
            .is_some_and(|value| !valid_ca_certificate_pem(value))
    {
        return false;
    }
    match authority {
        AddressBookEndpointAuthorityV1::Production => {
            endpoint.host == production_host
                && endpoint.port == production_port
                && endpoint.ca_certificate_pem.is_none()
        }
        #[cfg(feature = "conformance-test-support")]
        AddressBookEndpointAuthorityV1::LoopbackConformance => {
            endpoint.port > 0 && matches!(endpoint.host.as_str(), "127.0.0.1" | "localhost")
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailInboundTransportV1 {
    Imap(MailImapConfigurationV1),
    Gmail(MailGmailConfigurationV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailImapConfigurationV1 {
    pub host: String,
    pub port: u16,
    pub username: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailGmailConfigurationV1 {
    pub user_id: String,
    pub from_address: Option<String>,
    pub api_endpoint: GmailApiEndpointV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailApiEndpointV1 {
    pub host: String,
    pub port: u16,
    pub ca_certificate_pem: Option<String>,
}

pub fn valid_account_configuration(configuration: &MailAccountConfigurationV1) -> bool {
    !configuration.connection_id.trim().is_empty()
        && valid_inbound_transport(&configuration.inbound)
        && valid_window(configuration.sync_window, configuration.sync_windows)
        && (!matches!(configuration.inbound, MailInboundTransportV1::Gmail(_))
            || configuration.smtp_endpoint.is_none())
        && configuration.smtp_endpoint.as_ref().is_none_or(|endpoint| {
            valid_host(&endpoint.host)
                && valid_smtp_port(endpoint.port)
                && !endpoint.username.trim().is_empty()
                && valid_mailbox(&endpoint.from_address)
                && endpoint
                    .ca_certificate_pem
                    .as_deref()
                    .is_none_or(valid_ca_certificate_pem)
        })
}

pub fn valid_inbound_transport(transport: &MailInboundTransportV1) -> bool {
    match transport {
        MailInboundTransportV1::Imap(configuration) => {
            valid_host(&configuration.host)
                && valid_port(configuration.port)
                && !configuration.username.trim().is_empty()
        }
        MailInboundTransportV1::Gmail(configuration) => {
            valid_gmail_user_id(&configuration.user_id)
                && configuration
                    .from_address
                    .as_deref()
                    .map_or(configuration.user_id == "me", valid_mailbox)
                && valid_gmail_api_endpoint(&configuration.api_endpoint)
        }
    }
}

#[must_use]
pub fn valid_gmail_api_endpoint(endpoint: &GmailApiEndpointV1) -> bool {
    if !valid_host(&endpoint.host)
        || endpoint
            .ca_certificate_pem
            .as_deref()
            .is_some_and(|value| !valid_ca_certificate_pem(value))
    {
        return false;
    }
    #[cfg(feature = "conformance-test-support")]
    {
        endpoint.port > 0 && matches!(endpoint.host.as_str(), "127.0.0.1" | "localhost")
    }
    #[cfg(not(feature = "conformance-test-support"))]
    {
        endpoint.host == GMAIL_API_HOST
            && endpoint.port == GMAIL_API_HTTPS_PORT
            && endpoint.ca_certificate_pem.is_none()
    }
}

pub fn valid_gmail_user_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

pub fn valid_mailbox(value: &str) -> bool {
    value.is_ascii()
        && !value.is_empty()
        && value.len() <= 320
        && !value.contains(char::is_whitespace)
        && !value.contains(['\r', '\n', '\0', '<', '>', '"'])
        && value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && valid_host(domain))
}

/// Mail/IMAP slice admits only these explicit statuses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailConnectionState {
    Provisioning,
    Ready,
    Syncing,
    Degraded,
    Retired,
}

/// Limited contract errors exposed by mail runtime boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContractError {
    InvalidHost,
    InvalidPort,
    InvalidOperation,
    InvalidPayload,
    WindowLimitExceeded,
}

/// Global constraints for the current slice.
pub const IMAP_PORT: u16 = 993;
pub const SMTP_IMPLICIT_TLS_PORT: u16 = 465;
pub const GMAIL_API_HOST: &str = "gmail.googleapis.com";
pub const GMAIL_API_HTTPS_PORT: u16 = 443;
pub const MAX_HOST_LEN: usize = 253;
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
pub const MAX_PLAIN_TEXT_BYTES: usize = 256 * 1024;
pub const MAX_DELIVERY_ATTACHMENTS: usize = 16;
pub const MAX_CA_CERTIFICATE_PEM_BYTES: usize = 64 * 1024;
pub const DEFAULT_WINDOW: u32 = 5_000;
pub const MAX_WINDOW: u32 = 1_000_000;
pub const MAX_WINDOWS: u32 = 1_000_000;
pub const SYNC_DEADLINE_SECONDS: u64 = 300;
pub const WINDOW_DEADLINE_SECONDS: u64 = 10;

pub type MailConnectionId = String;
pub type MailOperationId = String;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailCredentialPurpose {
    ImapPassword,
    GmailAccessToken,
    GmailRefreshCredential,
    SmtpPassword,
    IcloudCardDavPassword,
}

impl MailCredentialPurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImapPassword => "mail_imap_password",
            Self::GmailAccessToken => "mail_gmail_access_token",
            Self::GmailRefreshCredential => "mail_gmail_refresh_credential",
            Self::SmtpPassword => "mail_smtp_password",
            Self::IcloudCardDavPassword => "mail_icloud_carddav_password",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginImapConnection {
    pub connection_id: MailConnectionId,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub use_tls: bool,
}

#[derive(Clone, Debug)]
pub struct CompleteImapConnection {
    pub connection_id: MailConnectionId,
    pub operation_id: MailOperationId,
}

#[derive(Clone, Debug)]
pub struct SyncNow {
    pub connection_id: MailConnectionId,
    pub operation_id: MailOperationId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmtpEndpointV1 {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub from_address: String,
    pub ca_certificate_pem: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutgoingMailV1 {
    pub operation_id: MailOperationId,
    pub connection_id: MailConnectionId,
    pub provider_conversation_id: String,
    pub recipients: Vec<String>,
    pub cc_recipients: Vec<String>,
    pub bcc_recipients: Vec<String>,
    pub subject: String,
    pub text_body: String,
}

#[derive(Clone, Debug)]
pub struct GetConnection {
    pub connection_id: MailConnectionId,
}

#[derive(Clone, Debug)]
pub struct GetSyncStatus {
    pub connection_id: MailConnectionId,
}

#[derive(Clone, Debug)]
pub struct GetOperationStatus {
    pub operation_id: MailOperationId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MailOperation {
    pub operation_id: MailOperationId,
    pub state: MailConnectionState,
    pub window_size: u32,
}

#[derive(Clone, Debug)]
pub struct MailConnection {
    pub id: MailConnectionId,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub state: MailConnectionState,
    pub operation_id: Option<MailOperationId>,
}

pub fn valid_host(host: &str) -> bool {
    if host.trim().is_empty() {
        return false;
    }
    if host.len() > MAX_HOST_LEN {
        return false;
    }
    if host == "localhost" {
        return true;
    }
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    parts.iter().all(|part| {
        (!part.is_empty())
            && part.len() <= 63
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    })
}

pub fn valid_port(port: u16) -> bool {
    #[cfg(feature = "conformance-test-support")]
    {
        port > 0
    }
    #[cfg(not(feature = "conformance-test-support"))]
    {
        port == IMAP_PORT && port > 0
    }
}

pub fn valid_smtp_port(port: u16) -> bool {
    #[cfg(feature = "conformance-test-support")]
    {
        port > 0
    }
    #[cfg(not(feature = "conformance-test-support"))]
    {
        port == SMTP_IMPLICIT_TLS_PORT
    }
}

pub fn valid_window(window: u32, windows: u32) -> bool {
    window > 0 && window <= MAX_WINDOW && windows > 0 && windows <= MAX_WINDOWS
}

pub fn valid_message_bytes(bytes: usize) -> bool {
    bytes <= MAX_MESSAGE_BYTES
}

pub fn valid_plain_text_bytes(bytes: usize) -> bool {
    bytes <= MAX_PLAIN_TEXT_BYTES
}

#[must_use]
pub fn valid_ca_certificate_pem(value: &str) -> bool {
    value.is_ascii()
        && value.len() <= MAX_CA_CERTIFICATE_PEM_BYTES
        && value.starts_with("-----BEGIN CERTIFICATE-----\n")
        && value.ends_with("-----END CERTIFICATE-----\n")
}

#[cfg(test)]
mod conformance_port_tests {
    use super::*;

    #[test]
    fn gmail_pre_authorization_requires_exact_provider_alias_without_mailbox() {
        let api_endpoint = if cfg!(feature = "conformance-test-support") {
            GmailApiEndpointV1 {
                host: "127.0.0.1".to_owned(),
                port: GMAIL_API_HTTPS_PORT,
                ca_certificate_pem: None,
            }
        } else {
            GmailApiEndpointV1 {
                host: GMAIL_API_HOST.to_owned(),
                port: GMAIL_API_HTTPS_PORT,
                ca_certificate_pem: None,
            }
        };
        let mut configuration = MailAccountConfigurationV1 {
            connection_id: "gmail-account".to_owned(),
            inbound: MailInboundTransportV1::Gmail(MailGmailConfigurationV1 {
                user_id: "me".to_owned(),
                from_address: None,
                api_endpoint,
            }),
            sync_window: 100,
            sync_windows: 10,
            smtp_endpoint: None,
        };
        assert!(valid_account_configuration(&configuration));

        if let MailInboundTransportV1::Gmail(gmail) = &mut configuration.inbound {
            gmail.user_id = "opaque-legacy-account".to_owned();
        }
        assert!(!valid_account_configuration(&configuration));

        if let MailInboundTransportV1::Gmail(gmail) = &mut configuration.inbound {
            gmail.from_address = Some("owner@example.test".to_owned());
        }
        assert!(valid_account_configuration(&configuration));
        if let MailInboundTransportV1::Gmail(gmail) = &mut configuration.inbound {
            gmail.from_address = Some("not-a-mailbox".to_owned());
        }
        assert!(!valid_account_configuration(&configuration));
    }

    #[cfg(not(feature = "conformance-test-support"))]
    #[test]
    fn production_provider_transports_accept_only_their_exact_tls_endpoints() {
        assert!(valid_port(IMAP_PORT));
        assert!(!valid_port(19_993));
        assert!(valid_gmail_api_endpoint(&GmailApiEndpointV1 {
            host: GMAIL_API_HOST.to_owned(),
            port: GMAIL_API_HTTPS_PORT,
            ca_certificate_pem: None,
        }));
        assert!(!valid_gmail_api_endpoint(&GmailApiEndpointV1 {
            host: "localhost".to_owned(),
            port: 19_443,
            ca_certificate_pem: None,
        }));
    }

    #[cfg(feature = "conformance-test-support")]
    #[test]
    fn conformance_imap_transport_accepts_a_non_zero_fixture_port() {
        assert!(valid_port(IMAP_PORT));
        assert!(valid_port(19_993));
        assert!(!valid_port(0));
        assert!(valid_gmail_api_endpoint(&GmailApiEndpointV1 {
            host: "localhost".to_owned(),
            port: 19_443,
            ca_certificate_pem: None,
        }));
        assert!(!valid_gmail_api_endpoint(&GmailApiEndpointV1 {
            host: "gmail.example.test".to_owned(),
            port: 19_443,
            ca_certificate_pem: None,
        }));
        assert!(!valid_gmail_api_endpoint(&GmailApiEndpointV1 {
            host: "localhost".to_owned(),
            port: 0,
            ca_certificate_pem: None,
        }));
    }

    #[test]
    fn address_book_provider_requires_its_own_exact_endpoint_authority() {
        let production_google = MailAddressBookTlsEndpointV1 {
            host: GOOGLE_PEOPLE_API_HOST_V1.to_owned(),
            port: GOOGLE_PEOPLE_API_PORT_V1,
            ca_certificate_pem: None,
        };
        let production_carddav = MailCardDavEndpointV1 {
            tls: MailAddressBookTlsEndpointV1 {
                host: ICLOUD_CARDDAV_HOST_V1.to_owned(),
                port: ICLOUD_CARDDAV_PORT_V1,
                ca_certificate_pem: None,
            },
            base_path: ICLOUD_CARDDAV_BASE_PATH_V1.to_owned(),
        };
        assert!(valid_google_people_endpoint_v1(&production_google));
        assert!(valid_carddav_endpoint_v1(&production_carddav));

        let loopback_google = MailAddressBookTlsEndpointV1 {
            host: "127.0.0.1".to_owned(),
            port: 19_443,
            ca_certificate_pem: None,
        };
        let loopback_carddav = MailCardDavEndpointV1 {
            tls: MailAddressBookTlsEndpointV1 {
                host: "localhost".to_owned(),
                port: 19_444,
                ca_certificate_pem: None,
            },
            base_path: "/contacts/".to_owned(),
        };
        assert!(!valid_google_people_endpoint_v1(&loopback_google));
        assert!(!valid_carddav_endpoint_v1(&loopback_carddav));
        #[cfg(feature = "conformance-test-support")]
        {
            assert!(valid_google_people_endpoint_for_authority_v1(
                &loopback_google,
                AddressBookEndpointAuthorityV1::LoopbackConformance,
            ));
            assert!(valid_carddav_endpoint_for_authority_v1(
                &loopback_carddav,
                AddressBookEndpointAuthorityV1::LoopbackConformance,
            ));
        }
    }
}

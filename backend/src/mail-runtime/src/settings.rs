//! Mail-owned decoding of one admitted generic settings snapshot.

#[cfg(not(feature = "conformance-test-support"))]
use makosh_mail_api::valid_address_book_configuration;
#[cfg(feature = "conformance-test-support")]
use makosh_mail_api::valid_address_book_configuration_for_conformance_v1;
use makosh_mail_api::{
    GmailApiEndpointV1, GmailOAuthConfigurationV1, GmailOAuthEndpointV1,
    MailAccountConfigurationV1, MailAddressBookConfigurationV1, MailAddressBookProviderV1,
    MailAddressBookTlsEndpointV1, MailCardDavEndpointV1, MailGmailConfigurationV1,
    MailImapConfigurationV1, MailInboundTransportV1, SmtpEndpointV1, valid_account_configuration,
    valid_gmail_oauth_configuration,
};
pub use makosh_mail_api::{MAIL_SETTINGS_SCHEMA_MAJOR_V2, MAIL_SETTINGS_SCHEMA_REVISION_V2};
use makosh_runtime_protocol::v1::{
    SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1, SettingMutationAuthorityV1,
    SettingTargetScopeV1, SettingValueTypeV1, SettingValueV1, SettingsSchemaV1, SettingsSnapshotV1,
    setting_value_v1::Value,
};
use prost::Message;

const CONNECTION_ID: &str = "mail.connection_id";
const ADDRESS_BOOK_PROVIDER: &str = "mail.address_book.provider";
const ADDRESS_BOOK_CARDDAV_USERNAME: &str = "mail.address_book.carddav_username";
const ADDRESS_BOOK_CARDDAV_HOST: &str = "mail.address_book.carddav_host";
const ADDRESS_BOOK_CARDDAV_PORT: &str = "mail.address_book.carddav_port";
const ADDRESS_BOOK_CARDDAV_BASE_PATH: &str = "mail.address_book.carddav_base_path";
const ADDRESS_BOOK_CARDDAV_CA_CERTIFICATE_PEM: &str =
    "mail.address_book.carddav_ca_certificate_pem";
const ADDRESS_BOOK_GOOGLE_PEOPLE_HOST: &str = "mail.address_book.google_people_host";
const ADDRESS_BOOK_GOOGLE_PEOPLE_PORT: &str = "mail.address_book.google_people_port";
const ADDRESS_BOOK_GOOGLE_PEOPLE_CA_CERTIFICATE_PEM: &str =
    "mail.address_book.google_people_ca_certificate_pem";
const ADDRESS_BOOK_ENDPOINT_SETTING_IDS: [&str; 7] = [
    ADDRESS_BOOK_CARDDAV_HOST,
    ADDRESS_BOOK_CARDDAV_PORT,
    ADDRESS_BOOK_CARDDAV_BASE_PATH,
    ADDRESS_BOOK_CARDDAV_CA_CERTIFICATE_PEM,
    ADDRESS_BOOK_GOOGLE_PEOPLE_HOST,
    ADDRESS_BOOK_GOOGLE_PEOPLE_PORT,
    ADDRESS_BOOK_GOOGLE_PEOPLE_CA_CERTIFICATE_PEM,
];
const IMAP_HOST: &str = "mail.imap.host";
const IMAP_PORT: &str = "mail.imap.port";
const IMAP_USERNAME: &str = "mail.imap.username";
const SYNC_WINDOW: &str = "mail.sync.window";
const SYNC_WINDOWS: &str = "mail.sync.windows";
const SMTP_ENABLED: &str = "mail.smtp.enabled";
const SMTP_CA_CERTIFICATE_PEM: &str = "mail.smtp.ca_certificate_pem";
const SMTP_HOST: &str = "mail.smtp.host";
const SMTP_PORT: &str = "mail.smtp.port";
const SMTP_USERNAME: &str = "mail.smtp.username";
const SMTP_FROM_ADDRESS: &str = "mail.smtp.from_address";
const INBOUND_KIND: &str = "mail.inbound.kind";
const GMAIL_API_HOST: &str = "mail.gmail.api_host";
const GMAIL_API_PORT: &str = "mail.gmail.api_port";
const GMAIL_CA_CERTIFICATE_PEM: &str = "mail.gmail.ca_certificate_pem";
const GMAIL_USER_ID: &str = "mail.gmail.user_id";
const GMAIL_FROM_ADDRESS: &str = "mail.gmail.from_address";
const GMAIL_OAUTH_AUTHORIZATION_CA_CERTIFICATE_PEM: &str =
    "mail.gmail.oauth.authorization_ca_certificate_pem";
const GMAIL_OAUTH_AUTHORIZATION_HOST: &str = "mail.gmail.oauth.authorization_host";
const GMAIL_OAUTH_AUTHORIZATION_PATH: &str = "mail.gmail.oauth.authorization_path";
const GMAIL_OAUTH_AUTHORIZATION_PORT: &str = "mail.gmail.oauth.authorization_port";
const GMAIL_OAUTH_CLIENT_ID: &str = "mail.gmail.oauth.client_id";
const GMAIL_OAUTH_REDIRECT_URI: &str = "mail.gmail.oauth.redirect_uri";
const GMAIL_OAUTH_TOKEN_CA_CERTIFICATE_PEM: &str = "mail.gmail.oauth.token_ca_certificate_pem";
const GMAIL_OAUTH_TOKEN_HOST: &str = "mail.gmail.oauth.token_host";
const GMAIL_OAUTH_TOKEN_PATH: &str = "mail.gmail.oauth.token_path";
const GMAIL_OAUTH_TOKEN_PORT: &str = "mail.gmail.oauth.token_port";
const GMAIL_OAUTH_SETTING_IDS: [&str; 10] = [
    GMAIL_OAUTH_AUTHORIZATION_CA_CERTIFICATE_PEM,
    GMAIL_OAUTH_AUTHORIZATION_HOST,
    GMAIL_OAUTH_AUTHORIZATION_PATH,
    GMAIL_OAUTH_AUTHORIZATION_PORT,
    GMAIL_OAUTH_CLIENT_ID,
    GMAIL_OAUTH_REDIRECT_URI,
    GMAIL_OAUTH_TOKEN_CA_CERTIFICATE_PEM,
    GMAIL_OAUTH_TOKEN_HOST,
    GMAIL_OAUTH_TOKEN_PATH,
    GMAIL_OAUTH_TOKEN_PORT,
];

/// The Mail integration owns these non-secret configuration-instance settings.
/// They are owner-editable through Settings; credential bindings remain
/// Mail-owned state and never enter Settings or Communications.
#[must_use]
pub fn mail_settings_schema_v2() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: MAIL_SETTINGS_SCHEMA_MAJOR_V2,
        revision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
        definitions: vec![
            definition(
                ADDRESS_BOOK_CARDDAV_BASE_PATH,
                SettingValueTypeV1::String,
                "CardDAV base path",
            ),
            definition(
                ADDRESS_BOOK_CARDDAV_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "CardDAV CA certificate",
            ),
            definition(
                ADDRESS_BOOK_CARDDAV_HOST,
                SettingValueTypeV1::String,
                "CardDAV host",
            ),
            definition(
                ADDRESS_BOOK_CARDDAV_PORT,
                SettingValueTypeV1::UnsignedInteger,
                "CardDAV port",
            ),
            definition(
                ADDRESS_BOOK_CARDDAV_USERNAME,
                SettingValueTypeV1::String,
                "CardDAV username",
            ),
            definition(
                ADDRESS_BOOK_GOOGLE_PEOPLE_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "Google People CA certificate",
            ),
            definition(
                ADDRESS_BOOK_GOOGLE_PEOPLE_HOST,
                SettingValueTypeV1::String,
                "Google People host",
            ),
            definition(
                ADDRESS_BOOK_GOOGLE_PEOPLE_PORT,
                SettingValueTypeV1::UnsignedInteger,
                "Google People port",
            ),
            string_definition_with_default(ADDRESS_BOOK_PROVIDER, "Address-book provider", "none"),
            required_definition(CONNECTION_ID, SettingValueTypeV1::String, "Connection ID"),
            definition(GMAIL_API_HOST, SettingValueTypeV1::String, "Gmail API host"),
            definition(
                GMAIL_API_PORT,
                SettingValueTypeV1::UnsignedInteger,
                "Gmail API port",
            ),
            definition(
                GMAIL_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "Gmail CA certificate",
            ),
            definition(
                GMAIL_FROM_ADDRESS,
                SettingValueTypeV1::String,
                "Gmail from address",
            ),
            definition(
                GMAIL_OAUTH_AUTHORIZATION_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "Gmail OAuth authorization CA certificate",
            ),
            definition(
                GMAIL_OAUTH_AUTHORIZATION_HOST,
                SettingValueTypeV1::String,
                "Gmail OAuth authorization host",
            ),
            definition(
                GMAIL_OAUTH_AUTHORIZATION_PATH,
                SettingValueTypeV1::String,
                "Gmail OAuth authorization path",
            ),
            definition(
                GMAIL_OAUTH_AUTHORIZATION_PORT,
                SettingValueTypeV1::UnsignedInteger,
                "Gmail OAuth authorization port",
            ),
            definition(
                GMAIL_OAUTH_CLIENT_ID,
                SettingValueTypeV1::String,
                "Gmail OAuth client ID",
            ),
            definition(
                GMAIL_OAUTH_REDIRECT_URI,
                SettingValueTypeV1::String,
                "Gmail OAuth redirect URI",
            ),
            definition(
                GMAIL_OAUTH_TOKEN_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "Gmail OAuth token CA certificate",
            ),
            definition(
                GMAIL_OAUTH_TOKEN_HOST,
                SettingValueTypeV1::String,
                "Gmail OAuth token host",
            ),
            definition(
                GMAIL_OAUTH_TOKEN_PATH,
                SettingValueTypeV1::String,
                "Gmail OAuth token path",
            ),
            definition(
                GMAIL_OAUTH_TOKEN_PORT,
                SettingValueTypeV1::UnsignedInteger,
                "Gmail OAuth token port",
            ),
            definition(GMAIL_USER_ID, SettingValueTypeV1::String, "Gmail user ID"),
            definition(IMAP_HOST, SettingValueTypeV1::String, "IMAP host"),
            definition(IMAP_PORT, SettingValueTypeV1::UnsignedInteger, "IMAP port"),
            definition(IMAP_USERNAME, SettingValueTypeV1::String, "IMAP username"),
            required_definition(
                INBOUND_KIND,
                SettingValueTypeV1::String,
                "Inbound transport",
            ),
            definition(
                SMTP_CA_CERTIFICATE_PEM,
                SettingValueTypeV1::String,
                "SMTP CA certificate",
            ),
            required_definition(SMTP_ENABLED, SettingValueTypeV1::Boolean, "SMTP enabled"),
            definition(
                SMTP_FROM_ADDRESS,
                SettingValueTypeV1::String,
                "SMTP from address",
            ),
            definition(SMTP_HOST, SettingValueTypeV1::String, "SMTP host"),
            definition(SMTP_PORT, SettingValueTypeV1::UnsignedInteger, "SMTP port"),
            definition(SMTP_USERNAME, SettingValueTypeV1::String, "SMTP username"),
            required_definition(
                SYNC_WINDOW,
                SettingValueTypeV1::UnsignedInteger,
                "Sync window",
            ),
            required_definition(
                SYNC_WINDOWS,
                SettingValueTypeV1::UnsignedInteger,
                "Sync windows",
            ),
        ],
    }
}

#[must_use]
pub fn mail_settings_schema_bytes_v2() -> Vec<u8> {
    mail_settings_schema_v2().encode_to_vec()
}

fn definition(
    setting_id: &str,
    value_type: SettingValueTypeV1,
    display_name: &str,
) -> SettingDefinitionV1 {
    SettingDefinitionV1 {
        setting_id: setting_id.to_owned(),
        capability_id: String::new(),
        value_type: value_type as i32,
        mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
        target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
        apply_mode: SettingApplyModeV1::RestartModule as i32,
        client_visibility: SettingClientVisibilityV1::Editable as i32,
        fresh_owner_proof_required: true,
        kernel_controller_id: String::new(),
        display_name: display_name.to_owned(),
        default_value: None,
        optional: true,
    }
}

fn required_definition(
    setting_id: &str,
    value_type: SettingValueTypeV1,
    display_name: &str,
) -> SettingDefinitionV1 {
    let mut definition = definition(setting_id, value_type, display_name);
    definition.optional = false;
    definition
}

fn string_definition_with_default(
    setting_id: &str,
    display_name: &str,
    default_value: &str,
) -> SettingDefinitionV1 {
    let mut definition = definition(setting_id, SettingValueTypeV1::String, display_name);
    definition.default_value = Some(SettingValueV1 {
        value: Some(Value::StringValue(default_value.to_owned())),
    });
    definition
}

pub struct MailRuntimeSettingsV1 {
    pub account: MailAccountConfigurationV1,
    pub address_book: MailAddressBookConfigurationV1,
    pub gmail_oauth: Option<GmailOAuthConfigurationV1>,
}

pub fn decode(snapshot: &SettingsSnapshotV1) -> Result<MailRuntimeSettingsV1, String> {
    let inbound = match required_string(snapshot, INBOUND_KIND)?.as_str() {
        "imap" => {
            for setting_id in [
                GMAIL_API_HOST,
                GMAIL_API_PORT,
                GMAIL_CA_CERTIFICATE_PEM,
                GMAIL_FROM_ADDRESS,
                GMAIL_USER_ID,
            ] {
                absent(snapshot, setting_id)?;
            }
            for setting_id in GMAIL_OAUTH_SETTING_IDS {
                absent(snapshot, setting_id)?;
            }
            MailInboundTransportV1::Imap(MailImapConfigurationV1 {
                host: required_string(snapshot, IMAP_HOST)?,
                port: u16::try_from(required_unsigned(snapshot, IMAP_PORT)?)
                    .map_err(|_| invalid_settings())?,
                username: required_string(snapshot, IMAP_USERNAME)?,
            })
        }
        "gmail" => MailInboundTransportV1::Gmail(MailGmailConfigurationV1 {
            user_id: required_string(snapshot, GMAIL_USER_ID)?,
            from_address: optional_string(snapshot, GMAIL_FROM_ADDRESS)?,
            api_endpoint: GmailApiEndpointV1 {
                host: required_string(snapshot, GMAIL_API_HOST)?,
                port: u16::try_from(required_unsigned(snapshot, GMAIL_API_PORT)?)
                    .map_err(|_| invalid_settings())?,
                ca_certificate_pem: optional_string(snapshot, GMAIL_CA_CERTIFICATE_PEM)?,
            },
        }),
        _ => return Err(invalid_settings()),
    };
    let account = MailAccountConfigurationV1 {
        connection_id: required_string(snapshot, CONNECTION_ID)?,
        inbound,
        sync_window: u32::try_from(required_unsigned(snapshot, SYNC_WINDOW)?)
            .map_err(|_| invalid_settings())?,
        sync_windows: u32::try_from(required_unsigned(snapshot, SYNC_WINDOWS)?)
            .map_err(|_| invalid_settings())?,
        smtp_endpoint: smtp_endpoint(snapshot)?,
    };
    if !valid_account_configuration(&account) {
        return Err(invalid_settings());
    }
    match &account.inbound {
        MailInboundTransportV1::Imap(_) => {}
        MailInboundTransportV1::Gmail(_) => {
            for setting_id in [IMAP_HOST, IMAP_PORT, IMAP_USERNAME] {
                absent(snapshot, setting_id)?;
            }
        }
    }
    let gmail_oauth = match &account.inbound {
        MailInboundTransportV1::Gmail(_) => optional_gmail_oauth_configuration(snapshot)?,
        MailInboundTransportV1::Imap(_) => None,
    };
    let address_book = match optional_string(snapshot, ADDRESS_BOOK_PROVIDER)?.as_deref() {
        None | Some("none") => {
            absent(snapshot, ADDRESS_BOOK_CARDDAV_USERNAME)?;
            for setting_id in ADDRESS_BOOK_ENDPOINT_SETTING_IDS {
                absent(snapshot, setting_id)?;
            }
            MailAddressBookConfigurationV1 {
                provider: MailAddressBookProviderV1::None,
                carddav_username: None,
                google_people_endpoint: None,
                carddav_endpoint: None,
            }
        }
        Some("google_people") => {
            absent(snapshot, ADDRESS_BOOK_CARDDAV_USERNAME)?;
            for setting_id in [
                ADDRESS_BOOK_CARDDAV_HOST,
                ADDRESS_BOOK_CARDDAV_PORT,
                ADDRESS_BOOK_CARDDAV_BASE_PATH,
                ADDRESS_BOOK_CARDDAV_CA_CERTIFICATE_PEM,
            ] {
                absent(snapshot, setting_id)?;
            }
            MailAddressBookConfigurationV1 {
                provider: MailAddressBookProviderV1::GooglePeople,
                carddav_username: None,
                google_people_endpoint: Some(MailAddressBookTlsEndpointV1 {
                    host: required_string(snapshot, ADDRESS_BOOK_GOOGLE_PEOPLE_HOST)?,
                    port: u16::try_from(required_unsigned(
                        snapshot,
                        ADDRESS_BOOK_GOOGLE_PEOPLE_PORT,
                    )?)
                    .map_err(|_| invalid_settings())?,
                    ca_certificate_pem: optional_string(
                        snapshot,
                        ADDRESS_BOOK_GOOGLE_PEOPLE_CA_CERTIFICATE_PEM,
                    )?,
                }),
                carddav_endpoint: None,
            }
        }
        Some("icloud_carddav") => {
            for setting_id in [
                ADDRESS_BOOK_GOOGLE_PEOPLE_HOST,
                ADDRESS_BOOK_GOOGLE_PEOPLE_PORT,
                ADDRESS_BOOK_GOOGLE_PEOPLE_CA_CERTIFICATE_PEM,
            ] {
                absent(snapshot, setting_id)?;
            }
            MailAddressBookConfigurationV1 {
                provider: MailAddressBookProviderV1::IcloudCardDav,
                carddav_username: Some(required_string(snapshot, ADDRESS_BOOK_CARDDAV_USERNAME)?),
                google_people_endpoint: None,
                carddav_endpoint: Some(MailCardDavEndpointV1 {
                    tls: MailAddressBookTlsEndpointV1 {
                        host: required_string(snapshot, ADDRESS_BOOK_CARDDAV_HOST)?,
                        port: u16::try_from(required_unsigned(
                            snapshot,
                            ADDRESS_BOOK_CARDDAV_PORT,
                        )?)
                        .map_err(|_| invalid_settings())?,
                        ca_certificate_pem: optional_string(
                            snapshot,
                            ADDRESS_BOOK_CARDDAV_CA_CERTIFICATE_PEM,
                        )?,
                    },
                    base_path: required_string(snapshot, ADDRESS_BOOK_CARDDAV_BASE_PATH)?,
                }),
            }
        }
        _ => return Err(invalid_settings()),
    };
    #[cfg(feature = "conformance-test-support")]
    let address_book_is_valid =
        valid_address_book_configuration_for_conformance_v1(&address_book, &account.inbound);
    #[cfg(not(feature = "conformance-test-support"))]
    let address_book_is_valid = valid_address_book_configuration(&address_book, &account.inbound);
    if !address_book_is_valid {
        return Err(invalid_settings());
    }
    Ok(MailRuntimeSettingsV1 {
        account,
        address_book,
        gmail_oauth,
    })
}

fn optional_gmail_oauth_configuration(
    snapshot: &SettingsSnapshotV1,
) -> Result<Option<GmailOAuthConfigurationV1>, String> {
    if GMAIL_OAUTH_SETTING_IDS
        .iter()
        .all(|setting_id| !has_value(snapshot, setting_id))
    {
        return Ok(None);
    }
    let configuration = GmailOAuthConfigurationV1 {
        client_id: required_string(snapshot, GMAIL_OAUTH_CLIENT_ID)?,
        redirect_uri: required_string(snapshot, GMAIL_OAUTH_REDIRECT_URI)?,
        authorization_endpoint: GmailOAuthEndpointV1 {
            host: required_string(snapshot, GMAIL_OAUTH_AUTHORIZATION_HOST)?,
            port: u16::try_from(required_unsigned(snapshot, GMAIL_OAUTH_AUTHORIZATION_PORT)?)
                .map_err(|_| invalid_settings())?,
            path: required_string(snapshot, GMAIL_OAUTH_AUTHORIZATION_PATH)?,
            ca_certificate_pem: optional_string(
                snapshot,
                GMAIL_OAUTH_AUTHORIZATION_CA_CERTIFICATE_PEM,
            )?,
        },
        token_endpoint: GmailOAuthEndpointV1 {
            host: required_string(snapshot, GMAIL_OAUTH_TOKEN_HOST)?,
            port: u16::try_from(required_unsigned(snapshot, GMAIL_OAUTH_TOKEN_PORT)?)
                .map_err(|_| invalid_settings())?,
            path: required_string(snapshot, GMAIL_OAUTH_TOKEN_PATH)?,
            ca_certificate_pem: optional_string(snapshot, GMAIL_OAUTH_TOKEN_CA_CERTIFICATE_PEM)?,
        },
    };
    valid_gmail_oauth_configuration(&configuration)
        .then_some(Some(configuration))
        .ok_or_else(invalid_settings)
}

fn smtp_endpoint(snapshot: &SettingsSnapshotV1) -> Result<Option<SmtpEndpointV1>, String> {
    if !required_boolean(snapshot, SMTP_ENABLED)? {
        for setting_id in [
            SMTP_CA_CERTIFICATE_PEM,
            SMTP_HOST,
            SMTP_PORT,
            SMTP_USERNAME,
            SMTP_FROM_ADDRESS,
        ] {
            absent(snapshot, setting_id)?;
        }
        return Ok(None);
    }
    Ok(Some(SmtpEndpointV1 {
        host: required_string(snapshot, SMTP_HOST)?,
        port: u16::try_from(required_unsigned(snapshot, SMTP_PORT)?)
            .map_err(|_| invalid_settings())?,
        username: required_string(snapshot, SMTP_USERNAME)?,
        from_address: required_string(snapshot, SMTP_FROM_ADDRESS)?,
        ca_certificate_pem: optional_string(snapshot, SMTP_CA_CERTIFICATE_PEM)?,
    }))
}

fn required_string(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<String, String> {
    match value(snapshot, setting_id)? {
        Value::StringValue(value) if !value.trim().is_empty() => Ok(value.clone()),
        _ => Err(invalid_settings()),
    }
}

fn required_unsigned(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<u64, String> {
    match value(snapshot, setting_id)? {
        Value::UnsignedIntegerValue(value) => Ok(*value),
        _ => Err(invalid_settings()),
    }
}

fn optional_string(
    snapshot: &SettingsSnapshotV1,
    setting_id: &str,
) -> Result<Option<String>, String> {
    let entries = snapshot
        .values
        .iter()
        .filter(|entry| entry.setting_id == setting_id)
        .collect::<Vec<_>>();
    match entries.as_slice() {
        [] => Ok(None),
        [entry] => match entry.value.as_ref().and_then(|value| value.value.as_ref()) {
            Some(Value::StringValue(value)) if !value.trim().is_empty() => Ok(Some(value.clone())),
            _ => Err(invalid_settings()),
        },
        _ => Err(invalid_settings()),
    }
}

fn required_boolean(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<bool, String> {
    match value(snapshot, setting_id)? {
        Value::BooleanValue(value) => Ok(*value),
        _ => Err(invalid_settings()),
    }
}

fn absent(snapshot: &SettingsSnapshotV1, setting_id: &str) -> Result<(), String> {
    (!snapshot
        .values
        .iter()
        .any(|entry| entry.setting_id == setting_id))
    .then_some(())
    .ok_or_else(invalid_settings)
}

fn has_value(snapshot: &SettingsSnapshotV1, setting_id: &str) -> bool {
    snapshot
        .values
        .iter()
        .any(|entry| entry.setting_id == setting_id)
}

fn value<'a>(snapshot: &'a SettingsSnapshotV1, setting_id: &str) -> Result<&'a Value, String> {
    let mut selected = None;
    for entry in &snapshot.values {
        if entry.setting_id == setting_id {
            let value = entry.value.as_ref().and_then(|value| value.value.as_ref());
            if selected.replace(value).is_some() {
                return Err(invalid_settings());
            }
        }
    }
    selected.flatten().ok_or_else(invalid_settings)
}

fn invalid_settings() -> String {
    "Mail runtime settings are invalid".to_owned()
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::{
        SettingValueV1, SettingsValueEntryV1, setting_value_v1::Value,
    };
    use makosh_runtime_protocol::validation::descriptor::validate_settings_schema_v1;

    use super::*;

    #[test]
    fn schema_is_versioned_owner_editable_and_configuration_scoped() {
        let schema = mail_settings_schema_v2();

        assert_eq!(validate_settings_schema_v1(&schema), Ok(()));
        assert!(schema.definitions.iter().all(|definition| {
            definition.target_scope == SettingTargetScopeV1::ConfigurationInstance as i32
                && definition.apply_mode == SettingApplyModeV1::RestartModule as i32
                && definition.client_visibility == SettingClientVisibilityV1::Editable as i32
                && definition.fresh_owner_proof_required
        }));
        assert_eq!(schema.definitions.len(), 37);
        assert_eq!(
            schema
                .definitions
                .iter()
                .filter(|definition| !definition.optional)
                .map(|definition| definition.setting_id.as_str())
                .collect::<Vec<_>>(),
            [
                CONNECTION_ID,
                INBOUND_KIND,
                SMTP_ENABLED,
                SYNC_WINDOW,
                SYNC_WINDOWS
            ]
        );
    }

    #[test]
    fn production_gmail_endpoint_defaults_are_canonical() {
        assert_eq!(makosh_mail_api::GMAIL_API_HOST, "gmail.googleapis.com");
        assert_eq!(makosh_mail_api::GMAIL_API_HTTPS_PORT, 443);
    }

    #[test]
    fn address_book_provider_settings_do_not_reuse_mail_transport_endpoint() {
        let mut snapshot = gmail_pre_authorization_snapshot("me");
        snapshot.values.extend([
            settings_entry(
                ADDRESS_BOOK_PROVIDER,
                Value::StringValue("google_people".to_owned()),
            ),
            settings_entry(
                ADDRESS_BOOK_GOOGLE_PEOPLE_HOST,
                Value::StringValue(if cfg!(feature = "conformance-test-support") {
                    "localhost".to_owned()
                } else {
                    makosh_mail_api::GOOGLE_PEOPLE_API_HOST_V1.to_owned()
                }),
            ),
            settings_entry(
                ADDRESS_BOOK_GOOGLE_PEOPLE_PORT,
                Value::UnsignedIntegerValue(u64::from(makosh_mail_api::GOOGLE_PEOPLE_API_PORT_V1)),
            ),
        ]);
        snapshot
            .values
            .sort_by(|left, right| left.setting_id.cmp(&right.setting_id));

        let decoded = decode(&snapshot).expect("decode Google People authority");
        let endpoint = decoded
            .address_book
            .google_people_endpoint
            .expect("Google People endpoint");
        let MailInboundTransportV1::Gmail(gmail) = decoded.account.inbound else {
            panic!("Gmail transport");
        };
        assert_ne!(endpoint.host, gmail.api_endpoint.host);
        assert!(decoded.address_book.carddav_endpoint.is_none());
    }

    #[test]
    fn gmail_pre_authorization_decodes_only_for_exact_provider_alias() {
        let snapshot = gmail_pre_authorization_snapshot("me");
        let decoded = decode(&snapshot).expect("decode Gmail pre-authorization settings");
        let MailInboundTransportV1::Gmail(gmail) = decoded.account.inbound else {
            panic!("expected Gmail settings");
        };
        assert_eq!(gmail.user_id, "me");
        assert_eq!(gmail.from_address, None);

        let invalid = gmail_pre_authorization_snapshot("opaque-legacy-account");
        assert!(matches!(decode(&invalid), Err(error) if error == invalid_settings()));
    }

    fn gmail_pre_authorization_snapshot(user_id: &str) -> SettingsSnapshotV1 {
        fn entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
            SettingsValueEntryV1 {
                setting_id: setting_id.to_owned(),
                value: Some(SettingValueV1 { value: Some(value) }),
            }
        }

        let production_endpoint = GmailApiEndpointV1 {
            host: makosh_mail_api::GMAIL_API_HOST.to_owned(),
            port: makosh_mail_api::GMAIL_API_HTTPS_PORT,
            ca_certificate_pem: None,
        };
        let gmail_api_host = if makosh_mail_api::valid_gmail_api_endpoint(&production_endpoint) {
            makosh_mail_api::GMAIL_API_HOST
        } else {
            "127.0.0.1"
        };
        let mut values = vec![
            entry(
                CONNECTION_ID,
                Value::StringValue("gmail-account".to_owned()),
            ),
            entry(
                GMAIL_API_HOST,
                Value::StringValue(gmail_api_host.to_owned()),
            ),
            entry(
                GMAIL_API_PORT,
                Value::UnsignedIntegerValue(u64::from(makosh_mail_api::GMAIL_API_HTTPS_PORT)),
            ),
            entry(GMAIL_USER_ID, Value::StringValue(user_id.to_owned())),
            entry(INBOUND_KIND, Value::StringValue("gmail".to_owned())),
            entry(SMTP_ENABLED, Value::BooleanValue(false)),
            entry(SYNC_WINDOW, Value::UnsignedIntegerValue(100)),
            entry(SYNC_WINDOWS, Value::UnsignedIntegerValue(10)),
        ];
        values.sort_by(|left, right| left.setting_id.cmp(&right.setting_id));
        SettingsSnapshotV1 {
            target_id: "gmail-target".to_owned(),
            revision: 1,
            values,
        }
    }

    fn settings_entry(setting_id: &str, value: Value) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 { value: Some(value) }),
        }
    }
}

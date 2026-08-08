use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use serde_json::Value;

use crate::platform::secrets::models::{SecretKind, SecretStoreKind};
use crate::vault::models::HostVaultManifestEntry;

use super::metadata::{
    fallback_display_name, fallback_provider_account_config, metadata_string, non_empty,
};

pub(super) struct RecoverableProviderSecret {
    pub(super) account_id: String,
    pub(super) provider_kind: CommunicationProviderKind,
    pub(super) display_name: String,
    pub(super) external_account_id: String,
    pub(super) secret_ref: String,
    pub(super) secret_kind: SecretKind,
    pub(super) store_kind: SecretStoreKind,
    pub(super) secret_purpose: ProviderAccountSecretPurpose,
    pub(super) label: String,
    pub(super) secret_metadata: Value,
    pub(super) provider_account_config: Value,
}

impl RecoverableProviderSecret {
    pub(super) fn from_manifest(entry: HostVaultManifestEntry) -> Option<Self> {
        if !is_recoverable_provider_entry_kind(&entry.entry_kind) {
            return None;
        }
        let account_id =
            non_empty(metadata_string(&entry.metadata, "account_id")).unwrap_or_else(|| {
                account_id_from_provider_secret_ref(&entry.account_id)
                    .or_else(|| account_id_from_provider_secret_ref(&entry.secret_ref))
                    .unwrap_or_else(|| entry.account_id.clone())
            });
        let provider = metadata_string(&entry.metadata, "provider_kind")
            .or_else(|| metadata_string(&entry.metadata, "provider"))
            .or_else(|| legacy_provider_from_account_id(&account_id, &entry.purpose))?;
        let provider_kind = CommunicationProviderKind::try_from(provider.as_str()).ok()?;

        let secret_kind = SecretKind::try_from(entry.secret_kind.as_str()).ok()?;
        let store_kind = SecretStoreKind::try_from(entry.store_kind.as_str()).ok()?;
        let secret_purpose = ProviderAccountSecretPurpose::try_from(entry.purpose.as_str()).ok()?;
        if !secret_purpose.accepts_secret_kind(secret_kind) {
            return None;
        }

        let display_name = non_empty(metadata_string(&entry.metadata, "display_name"))
            .unwrap_or_else(|| fallback_display_name(provider_kind, &entry.label, &account_id));
        let external_account_id =
            non_empty(metadata_string(&entry.metadata, "external_account_id"))
                .or_else(|| legacy_external_account_id(provider_kind, &account_id))
                .unwrap_or_else(|| account_id.clone());
        let provider_account_config = entry
            .metadata
            .get("provider_account_config")
            .filter(|value| value.is_object())
            .cloned()
            .unwrap_or_else(|| {
                fallback_provider_account_config(
                    provider_kind,
                    &entry.metadata,
                    &external_account_id,
                )
            });

        Some(Self {
            account_id,
            provider_kind,
            display_name,
            external_account_id,
            secret_ref: entry.secret_ref,
            secret_kind,
            store_kind,
            secret_purpose,
            label: entry.label,
            secret_metadata: entry.metadata,
            provider_account_config,
        })
    }
}

fn account_id_from_provider_secret_ref(value: &str) -> Option<String> {
    let account_and_purpose = value.trim().strip_prefix("secret:provider-account:")?;
    let (account_id, _purpose) = account_and_purpose.rsplit_once(':')?;
    non_empty(Some(account_id.to_owned()))
}

fn legacy_provider_from_account_id(account_id: &str, purpose: &str) -> Option<String> {
    let normalized = account_id.trim().to_ascii_lowercase();
    if normalized.starts_with("gmail-") || normalized.starts_with("mail-gmail-") {
        return Some("gmail".to_owned());
    }
    if normalized.starts_with("icloud-") {
        return Some("icloud".to_owned());
    }
    if normalized.starts_with("imap-")
        || purpose == ProviderAccountSecretPurpose::ImapPassword.as_str()
    {
        return Some("imap".to_owned());
    }
    None
}

fn legacy_external_account_id(
    provider_kind: CommunicationProviderKind,
    account_id: &str,
) -> Option<String> {
    match provider_kind {
        CommunicationProviderKind::Gmail => {
            email_from_prefixed_account_id(account_id, &["mail-gmail-", "gmail-"], "gmail.com")
        }
        CommunicationProviderKind::Icloud => email_from_prefixed_account_id(
            account_id,
            &["icloud-"],
            if account_id.ends_with("-icloud-com") {
                "icloud.com"
            } else {
                "me.com"
            },
        ),
        _ => None,
    }
}

fn email_from_prefixed_account_id(
    account_id: &str,
    prefixes: &[&str],
    expected_domain: &str,
) -> Option<String> {
    let normalized = account_id.trim().to_ascii_lowercase();
    let local_and_domain = prefixes
        .iter()
        .find_map(|prefix| normalized.strip_prefix(prefix))?;
    if local_and_domain == "primary" {
        return None;
    }
    let domain_suffix = format!("-{}", expected_domain.replace('.', "-"));
    let local = local_and_domain.strip_suffix(&domain_suffix)?;
    non_empty(Some(format!(
        "{}@{}",
        local.replace('-', "."),
        expected_domain
    )))
}

pub(super) fn is_recoverable_provider_entry_kind(entry_kind: &str) -> bool {
    matches!(
        entry_kind.trim(),
        "provider_credential"
            | "provider_api_token"
            | "provider_client_secret"
            | "provider_session"
            | "provider_account_session"
    )
}

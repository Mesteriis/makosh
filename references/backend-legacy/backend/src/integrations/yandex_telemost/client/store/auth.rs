use super::super::errors::YandexTelemostError;
use crate::platform::secrets::models::{NewSecretReference, SecretKind, SecretStoreKind};
use crate::platform::secrets::store::SecretReferenceStore;
use crate::vault::HostVault;
use crate::vault::models::SecretEntryContext;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_provider_telemost::protocol::{
    YANDEX_TELEMOST_PROVIDER_KIND_STR, sanitize_yandex_telemost_payload, validate_json_object,
    validate_required,
};
use serde_json::{Value, json};

pub(super) async fn store_oauth_token(
    secret_store: &SecretReferenceStore,
    vault: &HostVault,
    account_id: &str,
    secret_ref: &str,
    token: &str,
    metadata: &Value,
) -> Result<(), YandexTelemostError> {
    validate_required("oauth_token", token)?;
    validate_json_object("metadata", metadata)?;
    let reference = NewSecretReference::new(
        secret_ref,
        SecretKind::OauthToken,
        SecretStoreKind::HostVault,
        "Yandex Telemost OAuth token",
    )
    .metadata(json!({
        "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "account_id": account_id,
        "secret_material": "excluded",
        "metadata": sanitize_yandex_telemost_payload(metadata.clone()),
    }));
    secret_store.upsert_secret_reference(&reference).await?;
    let vault_metadata = json!({
        "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "provider_kind": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "account_id": account_id,
        "secret_purpose": ProviderAccountSecretPurpose::YandexTelemostOauthToken.as_str(),
        "metadata": sanitize_yandex_telemost_payload(metadata.clone()),
    });
    vault.store_secret(
        secret_ref,
        token.trim(),
        SecretEntryContext {
            entry_kind: "provider_credential",
            account_id,
            purpose: ProviderAccountSecretPurpose::YandexTelemostOauthToken.as_str(),
            secret_kind: SecretKind::OauthToken.as_str(),
            label: "Yandex Telemost OAuth token",
            metadata: &vault_metadata,
        },
    )?;
    Ok(())
}

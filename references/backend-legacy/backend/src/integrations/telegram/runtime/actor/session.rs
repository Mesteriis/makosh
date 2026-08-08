use crate::integrations::telegram::client::errors::TelegramError;
use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_api::accounts::ProviderSecretBindingLookupPort;

use crate::platform::secrets::resolver::SecretResolver;

pub(in crate::integrations::telegram::runtime) async fn optional_telegram_session_key(
    binding_store: &dyn ProviderSecretBindingLookupPort,
    secret_store: &SecretReferenceStore,
    secret_resolver: &(impl SecretResolver + Sync + ?Sized),
    account_id: &str,
) -> Result<Option<String>, TelegramError> {
    let binding = binding_store
        .get_for_account(account_id, ProviderAccountSecretPurpose::TelegramSessionKey)
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!(
                "failed to resolve Telegram session encryption key: {error}"
            ))
        })?;
    let Some(binding) = binding else {
        return Ok(None);
    };
    let reference = secret_store
        .secret_reference(&binding.secret_ref)
        .await
        .map_err(|error| {
            TelegramError::TdlibRuntime(format!(
                "failed to resolve Telegram session encryption key: {error}"
            ))
        })?
        .ok_or_else(|| {
            TelegramError::TdlibRuntime(format!(
                "failed to resolve Telegram session encryption key: secret reference metadata not found: {}",
                binding.secret_ref
            ))
        })?;
    if !binding
        .secret_purpose
        .accepts_secret_kind(reference.secret_kind)
    {
        return Err(TelegramError::TdlibRuntime(format!(
            "failed to resolve Telegram session encryption key: incompatible secret kind for {}",
            reference.secret_ref
        )));
    }
    let secret = secret_resolver.resolve(&reference).await.map_err(|error| {
        TelegramError::TdlibRuntime(format!(
            "failed to resolve Telegram session encryption key: {error}"
        ))
    })?;

    Ok(Some(secret.expose_for_runtime().to_owned()))
}

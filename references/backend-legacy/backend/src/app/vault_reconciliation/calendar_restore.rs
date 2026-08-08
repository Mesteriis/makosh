use crate::domains::calendar::events::account_store::CalendarAccountStore;
use makosh_communications_api::accounts::CommunicationProviderKind;
use makosh_communications_api::accounts::ProviderAccountSecretPurpose;

use super::errors::HostVaultReconciliationError;
use super::provider_recovery::RecoverableProviderSecret;

pub(super) async fn restore_linked_calendar_account(
    calendar_store: &CalendarAccountStore,
    secret: &RecoverableProviderSecret,
) -> Result<bool, HostVaultReconciliationError> {
    match secret.provider_kind {
        CommunicationProviderKind::Gmail => {
            let calendar_account_id = format!("google-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .restore_google_workspace_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        CommunicationProviderKind::Icloud => {
            if secret.secret_purpose != ProviderAccountSecretPurpose::ImapPassword {
                return Ok(false);
            }
            let calendar_account_id = format!("icloud-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .restore_apple_icloud_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

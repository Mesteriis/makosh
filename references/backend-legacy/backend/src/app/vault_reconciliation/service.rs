use crate::platform::secrets::store::SecretReferenceStore;
use makosh_communications_api::accounts::NewProviderAccount;
use makosh_communications_api::accounts::NewProviderAccountSecretBinding;
use std::collections::HashMap;

use sqlx::postgres::PgPool;

use crate::ai::control_center::store::AiControlCenterStore;
use crate::domains::calendar::events::account_store::CalendarAccountStore;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};

use crate::platform::secrets::models::NewSecretReference;
use crate::vault::HostVault;

use super::ai_provider_recovery::RecoverableAiProviderSecret;
use super::calendar_restore::restore_linked_calendar_account;
use super::errors::HostVaultReconciliationError;
use super::manifest_enrichment::enrich_manifest_entry_from_postgres;
use super::provider_recovery::RecoverableProviderSecret;
use super::summary::HostVaultReconciliationSummary;

pub(super) async fn reconcile_host_vault_manifest(
    pool: PgPool,
    vault: HostVault,
) -> Result<HostVaultReconciliationSummary, HostVaultReconciliationError> {
    let mut manifest = vault.account_secret_manifest()?;
    manifest.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    let secret_store = SecretReferenceStore::new(pool.clone());
    let provider_account_store = CommunicationProviderAccountStore::new(pool.clone());
    let provider_secret_binding_store = CommunicationProviderSecretBindingStore::new(pool.clone());
    let calendar_store = CalendarAccountStore::new(pool.clone());
    let ai_store = AiControlCenterStore::new(pool.clone());
    let mut summary = HostVaultReconciliationSummary::default();
    let mut provider_restore_identities =
        existing_provider_restore_identities(&provider_account_store).await?;

    for mut entry in manifest {
        enrich_manifest_entry_from_postgres(&pool, &vault, &mut entry).await?;
        if let Some(recoverable) = RecoverableAiProviderSecret::from_manifest(entry.clone()) {
            restore_ai_secret_reference(&secret_store, &recoverable).await?;
            let restore_missing = ai_store
                .provider(&recoverable.restore.provider_id)
                .await?
                .is_none()
                || ai_store
                    .api_key_secret_ref(&recoverable.restore.provider_id)
                    .await?
                    .is_none();
            if restore_missing {
                ai_store
                    .restore_provider_from_vault(&recoverable.restore)
                    .await?;
                summary.restored_ai_providers += 1;
            }
            continue;
        }
        let Some(recoverable) = RecoverableProviderSecret::from_manifest(entry) else {
            continue;
        };
        if !claim_provider_restore_identity(
            &mut provider_restore_identities,
            &recoverable,
            &mut summary,
        ) {
            if vault.delete_secret(&recoverable.secret_ref)? {
                summary.purged_duplicate_provider_secrets += 1;
            }
            continue;
        }
        restore_secret_reference(&secret_store, &recoverable).await?;
        restore_provider_account(&provider_account_store, &recoverable, &mut summary).await?;
        restore_provider_account_secret_binding(&provider_secret_binding_store, &recoverable)
            .await?;

        if restore_linked_calendar_account(&calendar_store, &recoverable).await? {
            summary.restored_calendar_accounts += 1;
        }
    }

    Ok(summary)
}

async fn existing_provider_restore_identities(
    store: &CommunicationProviderAccountStore,
) -> Result<HashMap<String, String>, HostVaultReconciliationError> {
    let mut identities = HashMap::new();
    for account in store.list().await? {
        let Some(identity) =
            provider_restore_identity(account.provider_kind.as_str(), &account.external_account_id)
        else {
            continue;
        };
        identities.entry(identity).or_insert(account.account_id);
    }

    Ok(identities)
}

fn claim_provider_restore_identity(
    identities: &mut HashMap<String, String>,
    secret: &RecoverableProviderSecret,
    summary: &mut HostVaultReconciliationSummary,
) -> bool {
    let Some(identity) =
        provider_restore_identity(secret.provider_kind.as_str(), &secret.external_account_id)
    else {
        return true;
    };

    if let Some(account_id) = identities.get(&identity) {
        if account_id == &secret.account_id {
            return true;
        }
        summary.skipped_duplicate_provider_secrets += 1;
        tracing::warn!(
            provider_kind = secret.provider_kind.as_str(),
            external_account_id = %secret.external_account_id,
            duplicate_account_id = %secret.account_id,
            canonical_account_id = %account_id,
            "deduplicating provider account secret from host vault manifest"
        );
        return false;
    }

    identities.insert(identity, secret.account_id.clone());
    true
}

fn provider_restore_identity(provider_kind: &str, external_account_id: &str) -> Option<String> {
    let provider = provider_kind.trim();
    let external = external_account_id.trim().to_ascii_lowercase();
    if provider.is_empty() || external.is_empty() {
        return None;
    }

    Some(format!("{provider}:{external}"))
}

async fn restore_ai_secret_reference(
    store: &SecretReferenceStore,
    secret: &RecoverableAiProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret.restore.secret_ref,
                secret.secret_kind,
                secret.store_kind,
                &secret.restore.secret_label,
            )
            .metadata(secret.restore.secret_metadata.clone()),
        )
        .await?;
    Ok(())
}

async fn restore_secret_reference(
    store: &SecretReferenceStore,
    secret: &RecoverableProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret.secret_ref,
                secret.secret_kind,
                secret.store_kind,
                &secret.label,
            )
            .metadata(secret.secret_metadata.clone()),
        )
        .await?;
    Ok(())
}

async fn restore_provider_account(
    store: &CommunicationProviderAccountStore,
    secret: &RecoverableProviderSecret,
    summary: &mut HostVaultReconciliationSummary,
) -> Result<(), HostVaultReconciliationError> {
    if store.get(&secret.account_id).await?.is_some() {
        return Ok(());
    }

    store
        .restore(
            &NewProviderAccount::new(
                &secret.account_id,
                secret.provider_kind,
                &secret.display_name,
                &secret.external_account_id,
            )
            .config(secret.provider_account_config.clone()),
        )
        .await?;
    summary.restored_accounts += 1;
    Ok(())
}

async fn restore_provider_account_secret_binding(
    store: &CommunicationProviderSecretBindingStore,
    secret: &RecoverableProviderSecret,
) -> Result<(), HostVaultReconciliationError> {
    store
        .restore(&NewProviderAccountSecretBinding::new(
            &secret.account_id,
            secret.secret_purpose,
            &secret.secret_ref,
        ))
        .await?;
    Ok(())
}

use makosh_communications_api::accounts::NewProviderAccount;
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};

use crate::platform::secrets::models::NewSecretReference;

use super::super::errors::EmailAccountSetupError;
use super::super::helpers::{imap_secret_ref, smtp_secret_ref};
use super::super::models::{EmailAccountSetupResult, ImapAccountSetupRequest};
use super::super::vault::SecretWriteContext;
use super::EmailAccountSetupService;
use super::imap_payloads::{imap_account_config, imap_secret_metadata};

impl EmailAccountSetupService {
    pub async fn setup_imap_account(
        &self,
        request: ImapAccountSetupRequest,
    ) -> Result<EmailAccountSetupResult, EmailAccountSetupError> {
        request.validate()?;
        let secret_ref = imap_secret_ref(&request.account_id);
        let smtp_secret_ref = smtp_secret_ref(&request.account_id);
        let account_config = imap_account_config(&request);
        let secret_metadata = imap_secret_metadata(&request, &account_config);

        let secret_store = self.secret_store()?;
        let provider_account_store = self.provider_account_store()?;
        let secret_binding_store = self.provider_secret_binding_store()?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    request.secret_kind,
                    self.vault.store_kind(),
                    format!("IMAP credential for {}", request.display_name),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        self.vault
            .store_secret(
                &secret_ref,
                &request.password,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &request.account_id,
                    purpose: ProviderAccountSecretPurpose::ImapPassword.as_str(),
                    secret_kind: request.secret_kind,
                    label: "IMAP password",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &smtp_secret_ref,
                    request.secret_kind,
                    self.vault.store_kind(),
                    format!("SMTP credential for {}", request.display_name),
                )
                .metadata(secret_metadata.clone()),
            )
            .await?;
        provider_account_store
            .upsert(
                &NewProviderAccount::new(
                    &request.account_id,
                    request.provider_kind,
                    &request.display_name,
                    &request.external_account_id,
                )
                .config(account_config),
            )
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;
        secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &request.account_id,
                ProviderAccountSecretPurpose::ImapPassword,
                &secret_ref,
            ))
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;
        self.vault
            .store_secret(
                &smtp_secret_ref,
                &request.password,
                SecretWriteContext {
                    entry_kind: "provider_credential",
                    account_id: &request.account_id,
                    purpose: ProviderAccountSecretPurpose::SmtpPassword.as_str(),
                    secret_kind: request.secret_kind,
                    label: "SMTP password",
                    metadata: &secret_metadata,
                },
            )
            .await?;
        secret_binding_store
            .bind(&NewProviderAccountSecretBinding::new(
                &request.account_id,
                ProviderAccountSecretPurpose::SmtpPassword,
                &smtp_secret_ref,
            ))
            .await
            .map_err(|error| EmailAccountSetupError::ProviderAccountStore(error.to_string()))?;

        Ok(EmailAccountSetupResult {
            account_id: request.account_id,
            secret_ref,
            secret_kind: request.secret_kind,
            store_kind: self.vault.store_kind(),
        })
    }
}

use crate::platform::communications::email_sync::EmailSyncPlanError;
use crate::vault::errors::HostVaultError;
use makosh_communications_api::mail_resources::EmailProviderSyncErrorKind;

use super::super::errors::ProviderSyncError;

#[derive(Debug)]
pub(in crate::workflows::mail_background_sync) struct SanitizedSyncFailure {
    pub(in crate::workflows::mail_background_sync) code: String,
    pub(in crate::workflows::mail_background_sync) message: String,
}

impl SanitizedSyncFailure {
    pub(in crate::workflows::mail_background_sync) fn from_plan(error: EmailSyncPlanError) -> Self {
        tracing::warn!(error = %error, "mail sync provider configuration is invalid");
        Self {
            code: "provider_config_invalid".to_owned(),
            message: "Mail provider configuration is invalid".to_owned(),
        }
    }

    pub(in crate::workflows::mail_background_sync) fn from_vault(error: HostVaultError) -> Self {
        match error {
            HostVaultError::Locked => Self {
                code: "vault_locked".to_owned(),
                message: "Host vault is locked".to_owned(),
            },
            HostVaultError::Uninitialized => Self {
                code: "vault_uninitialized".to_owned(),
                message: "Host vault is not initialized".to_owned(),
            },
            other => {
                tracing::warn!(error = %other, "mail sync vault check failed");
                Self {
                    code: "vault_unavailable".to_owned(),
                    message: "Host vault is unavailable".to_owned(),
                }
            }
        }
    }
}

impl From<ProviderSyncError> for SanitizedSyncFailure {
    fn from(error: ProviderSyncError) -> Self {
        match error {
            ProviderSyncError::ProviderSync(error) => match error.kind {
                EmailProviderSyncErrorKind::MissingCredential
                | EmailProviderSyncErrorKind::Credential => Self {
                    code: "credential_unavailable".to_owned(),
                    message: "Provider credential is unavailable for this account".to_owned(),
                },
                EmailProviderSyncErrorKind::AccountSetup => Self {
                    code: "oauth_refresh_failed".to_owned(),
                    message: "OAuth access token refresh failed".to_owned(),
                },
                EmailProviderSyncErrorKind::ProviderNetwork => {
                    tracing::warn!(error = %error, "mail provider sync network call failed");
                    Self {
                        code: "provider_network_error".to_owned(),
                        message: "Mail provider network request failed".to_owned(),
                    }
                }
            },
            ProviderSyncError::Pipeline(error) => {
                tracing::error!(error = %error, "mail sync projection pipeline failed");
                Self {
                    code: "projection_failed".to_owned(),
                    message: "Mail sync projection failed".to_owned(),
                }
            }
            ProviderSyncError::Graph(error) => {
                tracing::error!(error = %error, "mail sync graph projection failed");
                Self {
                    code: "graph_projection_failed".to_owned(),
                    message: "Mail graph projection failed".to_owned(),
                }
            }
            ProviderSyncError::CommunicationEvidence(error) => {
                tracing::error!(error = %error, "mail sync communication evidence port failed");
                Self {
                    code: "communication_evidence_error".to_owned(),
                    message: "Mail sync communication evidence operation failed".to_owned(),
                }
            }
            ProviderSyncError::SyncState(error) => {
                tracing::error!(error = %error, "mail sync status store failed");
                Self {
                    code: "sync_store_error".to_owned(),
                    message: "Mail sync status store failed".to_owned(),
                }
            }
        }
    }
}

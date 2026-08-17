//! Mail-owned Gmail OAuth workflow and credential rotation.

use makosh_mail_api::{
    GMAIL_OAUTH_ATTEMPT_TTL_SECONDS, GmailOAuthAuthorityV1, GmailOAuthCompleteRequestV1,
    GmailOAuthOperationKindV1 as ApiOperationKindV1, GmailOAuthOperationStatusV1,
    GmailOAuthOutcomeV1, GmailOAuthStartedV1, MailCredentialPurpose, MailInboundTransportV1,
};
use makosh_mail_core::oauth::{
    derive_gmail_oauth_attempt, gmail_oauth_authorization_code_sha256,
    gmail_oauth_scope_authorizes_contacts_write, gmail_oauth_scope_sha256,
    gmail_oauth_state_sha256,
};
use makosh_mail_gmail::{
    GmailAdapterErrorV1, GmailAuthorizationCodeExchangeV1, GmailOAuthTokenResponseV1,
    GmailRefreshTokenRequestV1, exchange_authorization_code, gmail_authorization_url,
    gmail_scope_authorizes, refresh_access_token,
};
use makosh_mail_persistence::{
    GmailOAuthAttemptStartV1, GmailOAuthCredentialBindingV1, GmailOAuthOperationKindV1,
    GmailOAuthOperationOutcomeV1, GmailOAuthQueuedOperationV1, MailDurablePersistenceError,
};
use makosh_managed_vault_client::{
    ManagedProviderCredentialClientV2, ManagedProviderCredentialErrorV1,
    ManagedProviderCredentialRequestV1,
};
use makosh_vault_protocol::SecretClassV1;
use zeroize::Zeroizing;

use crate::admission::MAIL_CREDENTIAL_LEASE_TTL_SECONDS;
use crate::managed::{MailAdmittedRuntime, MailBootstrapError, MailBusyControlDispatcher};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailGmailOAuthDispatchErrorV1 {
    InvalidStoredOperation,
    Persistence,
    Rejected,
    OutcomeUnknown,
}

pub enum PreparedGmailOAuthProviderOperationV1 {
    Complete {
        queued: GmailOAuthQueuedOperationV1,
        request: GmailAuthorizationCodeExchangeV1,
    },
    Refresh {
        queued: GmailOAuthQueuedOperationV1,
        current: GmailOAuthCredentialBindingV1,
        request: GmailRefreshTokenRequestV1,
    },
}

pub struct CompletedGmailOAuthProviderOperationV1 {
    prepared: PreparedGmailOAuthProviderOperationV1,
    provider_result: Result<GmailOAuthTokenResponseV1, MailGmailOAuthDispatchErrorV1>,
}

impl CompletedGmailOAuthProviderOperationV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        match &self.prepared {
            PreparedGmailOAuthProviderOperationV1::Complete { queued, .. }
            | PreparedGmailOAuthProviderOperationV1::Refresh { queued, .. } => {
                &queued.connection_id
            }
        }
    }
}

impl MailAdmittedRuntime {
    pub async fn start_gmail_oauth(
        &self,
        operation_id: &str,
        authority: GmailOAuthAuthorityV1,
        requested_at_unix_seconds: i64,
    ) -> Result<GmailOAuthStartedV1, MailBootstrapError> {
        if !self.provider_io_permitted() {
            return Err(MailBootstrapError::Credential);
        }
        let configuration = self
            .gmail_oauth
            .as_ref()
            .ok_or(MailBootstrapError::Admission)?;
        if !matches!(self.account.inbound, MailInboundTransportV1::Gmail(_))
            || self.settings_revision == 0
        {
            return Err(MailBootstrapError::Admission);
        }
        let expires_at_unix_seconds = requested_at_unix_seconds
            .checked_add(GMAIL_OAUTH_ATTEMPT_TTL_SECONDS)
            .ok_or(MailBootstrapError::Admission)?;
        let mut setup_id_entropy = [0_u8; 16];
        let mut state_entropy = [0_u8; 32];
        let mut verifier_entropy = [0_u8; 32];
        getrandom::fill(&mut setup_id_entropy).map_err(|_| MailBootstrapError::Credential)?;
        getrandom::fill(&mut state_entropy).map_err(|_| MailBootstrapError::Credential)?;
        getrandom::fill(&mut verifier_entropy).map_err(|_| MailBootstrapError::Credential)?;
        let material =
            derive_gmail_oauth_attempt(&setup_id_entropy, &state_entropy, &verifier_entropy)
                .map_err(|_| MailBootstrapError::Credential)?;
        let authorization_url = gmail_authorization_url(
            configuration,
            &material.state,
            &material.code_challenge,
            authority,
        )
        .map_err(|_| MailBootstrapError::Admission)?;
        let stored = self
            .durable
            .start_gmail_oauth_attempt(&GmailOAuthAttemptStartV1 {
                operation_id: operation_id.to_owned(),
                setup_id: material.setup_id,
                connection_id: self.account.connection_id.clone(),
                state_sha256: material.state_sha256,
                authorization_url,
                code_verifier: material.code_verifier,
                settings_revision: self.settings_revision,
                created_at_unix_seconds: requested_at_unix_seconds,
                expires_at_unix_seconds,
                authority,
            })
            .await
            .map_err(map_persistence_error)?;
        Ok(GmailOAuthStartedV1 {
            operation_id: stored.operation_id,
            setup_id: stored.setup_id,
            authorization_url: stored.authorization_url,
            expires_at_unix_seconds: stored.expires_at_unix_seconds,
        })
    }

    pub async fn submit_gmail_oauth_complete(
        &self,
        request: &GmailOAuthCompleteRequestV1,
        requested_at_unix_seconds: i64,
    ) -> Result<String, MailBootstrapError> {
        if !self.provider_io_permitted() {
            return Err(MailBootstrapError::Credential);
        }
        self.gmail_oauth
            .as_ref()
            .ok_or(MailBootstrapError::Admission)?;
        let state_sha256 =
            gmail_oauth_state_sha256(&request.state).map_err(|_| MailBootstrapError::Admission)?;
        let authorization_code_sha256 =
            gmail_oauth_authorization_code_sha256(&request.authorization_code);
        self.durable
            .enqueue_gmail_oauth_complete(
                &request.operation_id,
                &request.setup_id,
                &state_sha256,
                &request.authorization_code,
                &authorization_code_sha256,
                requested_at_unix_seconds,
            )
            .await
            .map_err(map_persistence_error)?;
        Ok(request.operation_id.clone())
    }

    pub async fn submit_gmail_oauth_refresh(
        &self,
        operation_id: &str,
        requested_at_unix_seconds: i64,
    ) -> Result<String, MailBootstrapError> {
        if !self.provider_io_permitted() {
            return Err(MailBootstrapError::Credential);
        }
        self.gmail_oauth
            .as_ref()
            .ok_or(MailBootstrapError::Admission)?;
        self.durable
            .enqueue_gmail_oauth_refresh(
                operation_id,
                &self.account.connection_id,
                requested_at_unix_seconds,
            )
            .await
            .map_err(map_persistence_error)?;
        Ok(operation_id.to_owned())
    }

    pub async fn gmail_oauth_operation_status(
        &self,
        operation_id: &str,
    ) -> Result<Option<GmailOAuthOperationStatusV1>, MailBootstrapError> {
        self.durable
            .gmail_oauth_operation(operation_id)
            .await
            .map_err(map_persistence_error)
            .map(|operation| {
                operation.map(|operation| {
                    let in_flight = self.gmail_oauth_operation_in_flight.as_deref()
                        == Some(operation.operation_id.as_str());
                    GmailOAuthOperationStatusV1 {
                        operation_id: operation.operation_id,
                        kind: match operation.kind {
                            GmailOAuthOperationKindV1::Complete => ApiOperationKindV1::Complete,
                            GmailOAuthOperationKindV1::Refresh => ApiOperationKindV1::Refresh,
                        },
                        outcome: match operation.outcome {
                            GmailOAuthOperationOutcomeV1::Pending => GmailOAuthOutcomeV1::Pending,
                            GmailOAuthOperationOutcomeV1::Completed => {
                                GmailOAuthOutcomeV1::Completed
                            }
                            GmailOAuthOperationOutcomeV1::Rejected => GmailOAuthOutcomeV1::Rejected,
                            GmailOAuthOperationOutcomeV1::OutcomeUnknown if in_flight => {
                                GmailOAuthOutcomeV1::Pending
                            }
                            GmailOAuthOperationOutcomeV1::OutcomeUnknown => {
                                GmailOAuthOutcomeV1::OutcomeUnknown
                            }
                        },
                        requested_at_unix_seconds: operation.requested_at_unix_seconds,
                        completed_at_unix_seconds: operation.completed_at_unix_seconds,
                    }
                })
            })
    }

    pub async fn resolve_gmail_access_token(
        &mut self,
    ) -> Result<Zeroizing<Vec<u8>>, MailBootstrapError> {
        if !self.provider_io_permitted()
            || !matches!(self.account.inbound, MailInboundTransportV1::Gmail(_))
        {
            return Err(MailBootstrapError::Admission);
        }
        let binding = self
            .durable
            .gmail_oauth_credential_binding(&self.account.connection_id)
            .await
            .map_err(map_persistence_error)?
            .ok_or(MailBootstrapError::Credential)?;
        self.resolve_credential(
            MailCredentialPurpose::GmailAccessToken,
            binding.access_token_revision,
            SecretClassV1::ProviderCredential,
        )
        .map_err(map_credential_error)
    }

    pub async fn execute_next_gmail_oauth_operation(
        &mut self,
        dispatched_at_unix_seconds: i64,
        completed_at_unix_seconds: i64,
    ) -> Result<bool, MailGmailOAuthDispatchErrorV1> {
        if !self.provider_io_permitted() {
            return Ok(false);
        }
        let Some(prepared) = self
            .prepare_next_gmail_oauth_provider_operation(
                dispatched_at_unix_seconds,
                completed_at_unix_seconds,
            )
            .await?
        else {
            return Ok(false);
        };
        let completed = execute_gmail_oauth_provider_operation(prepared).await;
        self.finalize_gmail_oauth_provider_operation(completed, completed_at_unix_seconds)
            .await?;
        Ok(true)
    }

    pub async fn prepare_next_gmail_oauth_provider_operation(
        &mut self,
        dispatched_at_unix_seconds: i64,
        rejected_at_unix_seconds: i64,
    ) -> Result<Option<PreparedGmailOAuthProviderOperationV1>, MailGmailOAuthDispatchErrorV1> {
        if self.gmail_oauth_operation_in_flight.is_some() {
            return Err(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation);
        }
        if !self.provider_io_permitted() {
            return Ok(None);
        }
        let queued = self
            .durable
            .claim_next_gmail_oauth_operation(
                &self.account.connection_id,
                dispatched_at_unix_seconds,
            )
            .await
            .map_err(map_dispatch_persistence_error)?;
        let Some(queued) = queued else {
            return Ok(None);
        };
        let configuration = self
            .gmail_oauth
            .clone()
            .ok_or(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
        let client_secret = self
            .resolve_credential(
                MailCredentialPurpose::GmailOAuthClientSecret,
                1,
                SecretClassV1::ProviderCredential,
            )
            .map_err(classify_credential_resolution_error);
        let client_secret = match client_secret {
            Ok(secret) => Zeroizing::new(
                String::from_utf8(secret.as_slice().to_vec())
                    .map_err(|_| MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?,
            ),
            Err(MailGmailOAuthDispatchErrorV1::Rejected) => {
                return self
                    .persist_oauth_rejection(&queued.operation_id, rejected_at_unix_seconds)
                    .await
                    .map(|()| None);
            }
            Err(error) => return Err(error),
        };
        let prepared = match queued.kind {
            GmailOAuthOperationKindV1::Complete => {
                let authorization_code = queued
                    .authorization_code
                    .as_deref()
                    .ok_or(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
                let code_verifier = queued
                    .code_verifier
                    .as_deref()
                    .ok_or(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
                PreparedGmailOAuthProviderOperationV1::Complete {
                    request: GmailAuthorizationCodeExchangeV1 {
                        configuration,
                        authorization_code: authorization_code.to_owned(),
                        code_verifier: code_verifier.to_owned(),
                        client_secret,
                    },
                    queued,
                }
            }
            GmailOAuthOperationKindV1::Refresh => {
                if queued.authorization_code.is_some() || queued.code_verifier.is_some() {
                    return Err(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation);
                }
                let current = self
                    .durable
                    .gmail_oauth_credential_binding(&queued.connection_id)
                    .await
                    .map_err(map_dispatch_persistence_error)?
                    .ok_or(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
                let refresh_credential = self
                    .resolve_credential(
                        MailCredentialPurpose::GmailRefreshCredential,
                        current.refresh_credential_revision,
                        SecretClassV1::OAuthRefreshCredential,
                    )
                    .map_err(classify_credential_resolution_error);
                let refresh_credential = match refresh_credential {
                    Ok(credential) => credential,
                    Err(MailGmailOAuthDispatchErrorV1::Rejected) => {
                        return self
                            .persist_oauth_rejection(&queued.operation_id, rejected_at_unix_seconds)
                            .await
                            .map(|()| None);
                    }
                    Err(error) => return Err(error),
                };
                let refresh_credential = std::str::from_utf8(&refresh_credential)
                    .map_err(|_| MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
                PreparedGmailOAuthProviderOperationV1::Refresh {
                    request: GmailRefreshTokenRequestV1 {
                        configuration,
                        refresh_token: refresh_credential.to_owned(),
                        client_secret,
                    },
                    current,
                    queued,
                }
            }
        };
        self.gmail_oauth_operation_in_flight = Some(queued_operation_id(&prepared).to_owned());
        Ok(Some(prepared))
    }

    pub async fn finalize_gmail_oauth_provider_operation(
        &mut self,
        completed: CompletedGmailOAuthProviderOperationV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailGmailOAuthDispatchErrorV1> {
        let operation_id = match &completed.prepared {
            PreparedGmailOAuthProviderOperationV1::Complete { queued, .. }
            | PreparedGmailOAuthProviderOperationV1::Refresh { queued, .. } => {
                queued.operation_id.clone()
            }
        };
        if self.gmail_oauth_operation_in_flight.as_deref() != Some(operation_id.as_str()) {
            return Err(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation);
        }
        let result = self
            .finalize_gmail_oauth_provider_operation_inner(completed, completed_at_unix_seconds)
            .await;
        self.gmail_oauth_operation_in_flight = None;
        result
    }

    async fn finalize_gmail_oauth_provider_operation_inner(
        &mut self,
        completed: CompletedGmailOAuthProviderOperationV1,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailGmailOAuthDispatchErrorV1> {
        let token = match completed.provider_result {
            Ok(token) => token,
            Err(MailGmailOAuthDispatchErrorV1::Rejected) => {
                let operation_id = match &completed.prepared {
                    PreparedGmailOAuthProviderOperationV1::Complete { queued, .. }
                    | PreparedGmailOAuthProviderOperationV1::Refresh { queued, .. } => {
                        queued.operation_id.as_str()
                    }
                };
                return self
                    .persist_oauth_rejection(operation_id, completed_at_unix_seconds)
                    .await;
            }
            Err(error) => return Err(error),
        };
        let (queued, binding) = match completed.prepared {
            PreparedGmailOAuthProviderOperationV1::Complete { queued, .. } => {
                if token.refresh_token.is_none() {
                    return self
                        .persist_oauth_rejection(&queued.operation_id, completed_at_unix_seconds)
                        .await;
                }
                let authority = queued
                    .authority
                    .ok_or(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
                if !token
                    .scope
                    .as_deref()
                    .is_some_and(|scope| gmail_scope_authorizes(authority, scope))
                {
                    return self
                        .persist_oauth_rejection(&queued.operation_id, completed_at_unix_seconds)
                        .await;
                }
                let existing = self
                    .durable
                    .gmail_oauth_credential_binding(&queued.connection_id)
                    .await
                    .map_err(map_dispatch_persistence_error)?;
                let contacts_write_authorized =
                    gmail_oauth_scope_authorizes_contacts_write(token.scope.as_deref());
                let binding = self
                    .store_oauth_token_pair(
                        &queued.operation_id,
                        existing.as_ref(),
                        token,
                        authority == GmailOAuthAuthorityV1::PermanentDelete,
                        contacts_write_authorized,
                        completed_at_unix_seconds,
                    )
                    .await;
                (queued, binding)
            }
            PreparedGmailOAuthProviderOperationV1::Refresh {
                queued, current, ..
            } => {
                let binding = self
                    .rotate_oauth_token_pair(
                        &queued.operation_id,
                        &current,
                        token,
                        completed_at_unix_seconds,
                    )
                    .await;
                (queued, binding)
            }
        };
        let binding = match binding {
            Ok(binding) => binding,
            Err(MailGmailOAuthDispatchErrorV1::Rejected) => {
                return self
                    .persist_oauth_rejection(&queued.operation_id, completed_at_unix_seconds)
                    .await;
            }
            Err(error) => return Err(error),
        };
        self.durable
            .complete_gmail_oauth_operation(
                &queued.operation_id,
                &binding,
                completed_at_unix_seconds,
            )
            .await
            .map_err(map_dispatch_persistence_error)?;
        self.advance_current_provider_io_epoch()
            .map_err(|_| MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
        Ok(())
    }

    async fn store_oauth_token_pair(
        &mut self,
        operation_id: &str,
        existing: Option<&GmailOAuthCredentialBindingV1>,
        token: GmailOAuthTokenResponseV1,
        permanent_delete_authorized: bool,
        contacts_write_authorized: bool,
        completed_at_unix_seconds: i64,
    ) -> Result<GmailOAuthCredentialBindingV1, MailGmailOAuthDispatchErrorV1> {
        let access_token_expires_at_unix_seconds =
            oauth_expiry(completed_at_unix_seconds, token.expires_in)?;
        let access_token = Zeroizing::new(token.access_token);
        let refresh_credential = token
            .refresh_token
            .map(Zeroizing::new)
            .ok_or(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)?;
        let (access_record_id, access_revision) = match existing {
            Some(binding) => {
                let revision = next_revision(binding.access_token_revision)?;
                (
                    self.replace_credential(
                        MailCredentialPurpose::GmailAccessToken,
                        revision,
                        SecretClassV1::ProviderCredential,
                        binding.access_token_record_id,
                        access_token.as_bytes(),
                    )
                    .map_err(classify_post_provider_credential_mutation_error)?,
                    revision,
                )
            }
            None => (
                self.store_credential(
                    MailCredentialPurpose::GmailAccessToken,
                    1,
                    SecretClassV1::ProviderCredential,
                    access_token.as_bytes(),
                )
                .map_err(classify_post_provider_credential_mutation_error)?,
                1,
            ),
        };
        self.durable
            .checkpoint_gmail_oauth_access_record(operation_id, &access_record_id, access_revision)
            .await
            .map_err(map_dispatch_persistence_error)?;
        let (refresh_record_id, refresh_revision) = match existing {
            Some(binding) => {
                let revision = next_revision(binding.refresh_credential_revision)?;
                (
                    self.replace_credential(
                        MailCredentialPurpose::GmailRefreshCredential,
                        revision,
                        SecretClassV1::OAuthRefreshCredential,
                        binding.refresh_credential_record_id,
                        refresh_credential.as_bytes(),
                    )
                    .map_err(classify_post_provider_credential_mutation_error)?,
                    revision,
                )
            }
            None => (
                self.store_credential(
                    MailCredentialPurpose::GmailRefreshCredential,
                    1,
                    SecretClassV1::OAuthRefreshCredential,
                    refresh_credential.as_bytes(),
                )
                .map_err(classify_post_provider_credential_mutation_error)?,
                1,
            ),
        };
        self.durable
            .checkpoint_gmail_oauth_refresh_record(
                operation_id,
                &refresh_record_id,
                refresh_revision,
            )
            .await
            .map_err(map_dispatch_persistence_error)?;
        Ok(oauth_binding(
            (access_record_id, access_revision),
            (refresh_record_id, refresh_revision),
            access_token_expires_at_unix_seconds,
            gmail_oauth_scope_sha256(token.scope.as_deref()),
            permanent_delete_authorized,
            contacts_write_authorized,
        ))
    }

    async fn rotate_oauth_token_pair(
        &mut self,
        operation_id: &str,
        current: &GmailOAuthCredentialBindingV1,
        token: GmailOAuthTokenResponseV1,
        completed_at_unix_seconds: i64,
    ) -> Result<GmailOAuthCredentialBindingV1, MailGmailOAuthDispatchErrorV1> {
        let access_token_expires_at_unix_seconds =
            oauth_expiry(completed_at_unix_seconds, token.expires_in)?;
        let access_token = Zeroizing::new(token.access_token);
        let access_revision = next_revision(current.access_token_revision)?;
        let access_record_id = self
            .replace_credential(
                MailCredentialPurpose::GmailAccessToken,
                access_revision,
                SecretClassV1::ProviderCredential,
                current.access_token_record_id,
                access_token.as_bytes(),
            )
            .map_err(classify_post_provider_credential_mutation_error)?;
        self.durable
            .checkpoint_gmail_oauth_access_record(operation_id, &access_record_id, access_revision)
            .await
            .map_err(map_dispatch_persistence_error)?;
        let (refresh_record_id, refresh_revision) = match token.refresh_token {
            Some(refresh_credential) => {
                let refresh_credential = Zeroizing::new(refresh_credential);
                let revision = next_revision(current.refresh_credential_revision)?;
                let record_id = self
                    .replace_credential(
                        MailCredentialPurpose::GmailRefreshCredential,
                        revision,
                        SecretClassV1::OAuthRefreshCredential,
                        current.refresh_credential_record_id,
                        refresh_credential.as_bytes(),
                    )
                    .map_err(classify_post_provider_credential_mutation_error)?;
                self.durable
                    .checkpoint_gmail_oauth_refresh_record(operation_id, &record_id, revision)
                    .await
                    .map_err(map_dispatch_persistence_error)?;
                (record_id, revision)
            }
            None => (
                current.refresh_credential_record_id,
                current.refresh_credential_revision,
            ),
        };
        Ok(oauth_binding(
            (access_record_id, access_revision),
            (refresh_record_id, refresh_revision),
            access_token_expires_at_unix_seconds,
            token
                .scope
                .as_deref()
                .map(|scope| gmail_oauth_scope_sha256(Some(scope)))
                .unwrap_or(current.scope_sha256),
            token
                .scope
                .as_deref()
                .map(|scope| gmail_scope_authorizes(GmailOAuthAuthorityV1::PermanentDelete, scope))
                .unwrap_or(current.permanent_delete_authorized),
            token
                .scope
                .as_deref()
                .map(|scope| gmail_oauth_scope_authorizes_contacts_write(Some(scope)))
                .unwrap_or(current.contacts_write_authorized),
        ))
    }

    fn resolve_credential(
        &mut self,
        purpose: MailCredentialPurpose,
        revision: u64,
        secret_class: SecretClassV1,
    ) -> Result<Zeroizing<Vec<u8>>, ManagedProviderCredentialErrorV1> {
        let configuration_instance_id = self.configuration_instance_id.clone();
        let context = self.provider_credential_context.clone();
        self.with_blocking_provider_credential_request(|channel| {
            let mut dispatcher = MailBusyControlDispatcher;
            ManagedProviderCredentialClientV2::new(channel).resolve(
                &mut dispatcher,
                &context,
                ManagedProviderCredentialRequestV1 {
                    configuration_instance_id: &configuration_instance_id,
                    purpose_id: purpose.as_str(),
                    credential_revision: revision,
                    ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
                    secret_class,
                },
            )
        })
    }

    fn store_credential(
        &mut self,
        purpose: MailCredentialPurpose,
        revision: u64,
        secret_class: SecretClassV1,
        payload: &[u8],
    ) -> Result<[u8; 16], ManagedProviderCredentialErrorV1> {
        let configuration_instance_id = self.configuration_instance_id.clone();
        let context = self.provider_credential_context.clone();
        self.with_blocking_provider_credential_request(|channel| {
            let mut dispatcher = MailBusyControlDispatcher;
            ManagedProviderCredentialClientV2::new(channel).store_once(
                &mut dispatcher,
                &context,
                ManagedProviderCredentialRequestV1 {
                    configuration_instance_id: &configuration_instance_id,
                    purpose_id: purpose.as_str(),
                    credential_revision: revision,
                    ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
                    secret_class,
                },
                payload,
            )
        })
    }

    fn replace_credential(
        &mut self,
        purpose: MailCredentialPurpose,
        revision: u64,
        secret_class: SecretClassV1,
        prior_record_id: [u8; 16],
        payload: &[u8],
    ) -> Result<[u8; 16], ManagedProviderCredentialErrorV1> {
        let configuration_instance_id = self.configuration_instance_id.clone();
        let context = self.provider_credential_context.clone();
        self.with_blocking_provider_credential_request(|channel| {
            let mut dispatcher = MailBusyControlDispatcher;
            ManagedProviderCredentialClientV2::new(channel).replace_once(
                &mut dispatcher,
                &context,
                ManagedProviderCredentialRequestV1 {
                    configuration_instance_id: &configuration_instance_id,
                    purpose_id: purpose.as_str(),
                    credential_revision: revision,
                    ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
                    secret_class,
                },
                prior_record_id,
                payload,
            )
        })
    }

    async fn persist_oauth_rejection(
        &self,
        operation_id: &str,
        completed_at_unix_seconds: i64,
    ) -> Result<(), MailGmailOAuthDispatchErrorV1> {
        self.durable
            .reject_gmail_oauth_operation(operation_id, completed_at_unix_seconds)
            .await
            .map_err(map_dispatch_persistence_error)?;
        Err(MailGmailOAuthDispatchErrorV1::Rejected)
    }
}

#[must_use]
fn queued_operation_id(prepared: &PreparedGmailOAuthProviderOperationV1) -> &str {
    match prepared {
        PreparedGmailOAuthProviderOperationV1::Complete { queued, .. }
        | PreparedGmailOAuthProviderOperationV1::Refresh { queued, .. } => &queued.operation_id,
    }
}

pub async fn execute_gmail_oauth_provider_operation(
    prepared: PreparedGmailOAuthProviderOperationV1,
) -> CompletedGmailOAuthProviderOperationV1 {
    let provider_result = match &prepared {
        PreparedGmailOAuthProviderOperationV1::Complete { request, .. } => {
            exchange_authorization_code(request)
                .await
                .map_err(classify_provider_error)
        }
        PreparedGmailOAuthProviderOperationV1::Refresh { request, .. } => {
            refresh_access_token(request)
                .await
                .map_err(classify_provider_error)
        }
    };
    CompletedGmailOAuthProviderOperationV1 {
        prepared,
        provider_result,
    }
}

fn oauth_binding(
    access: ([u8; 16], u64),
    refresh: ([u8; 16], u64),
    access_token_expires_at_unix_seconds: i64,
    scope_sha256: [u8; 32],
    permanent_delete_authorized: bool,
    contacts_write_authorized: bool,
) -> GmailOAuthCredentialBindingV1 {
    GmailOAuthCredentialBindingV1 {
        access_token_record_id: access.0,
        access_token_revision: access.1,
        refresh_credential_record_id: refresh.0,
        refresh_credential_revision: refresh.1,
        access_token_expires_at_unix_seconds,
        scope_sha256,
        permanent_delete_authorized,
        contacts_write_authorized,
    }
}

fn oauth_expiry(
    completed_at_unix_seconds: i64,
    expires_in: u64,
) -> Result<i64, MailGmailOAuthDispatchErrorV1> {
    let expires_in =
        i64::try_from(expires_in).map_err(|_| MailGmailOAuthDispatchErrorV1::OutcomeUnknown)?;
    completed_at_unix_seconds
        .checked_add(expires_in)
        .filter(|expires_at| *expires_at > completed_at_unix_seconds)
        .ok_or(MailGmailOAuthDispatchErrorV1::OutcomeUnknown)
}

fn next_revision(revision: u64) -> Result<u64, MailGmailOAuthDispatchErrorV1> {
    revision
        .checked_add(1)
        .filter(|revision| *revision > 1)
        .ok_or(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation)
}

fn map_persistence_error(_error: MailDurablePersistenceError) -> MailBootstrapError {
    MailBootstrapError::Persistence
}

fn map_dispatch_persistence_error(
    error: MailDurablePersistenceError,
) -> MailGmailOAuthDispatchErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_gmail_oauth_persistence_error={error:?}");
    }
    MailGmailOAuthDispatchErrorV1::Persistence
}

fn map_credential_error(error: ManagedProviderCredentialErrorV1) -> MailBootstrapError {
    match error {
        ManagedProviderCredentialErrorV1::InvalidContext => MailBootstrapError::Admission,
        ManagedProviderCredentialErrorV1::Rejected
        | ManagedProviderCredentialErrorV1::Unavailable => MailBootstrapError::Credential,
    }
}

fn classify_credential_resolution_error(
    error: ManagedProviderCredentialErrorV1,
) -> MailGmailOAuthDispatchErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_gmail_oauth_credential_resolution_error={error:?}");
    }
    match error {
        ManagedProviderCredentialErrorV1::InvalidContext => {
            MailGmailOAuthDispatchErrorV1::InvalidStoredOperation
        }
        ManagedProviderCredentialErrorV1::Rejected => MailGmailOAuthDispatchErrorV1::Rejected,
        ManagedProviderCredentialErrorV1::Unavailable => {
            MailGmailOAuthDispatchErrorV1::OutcomeUnknown
        }
    }
}

fn classify_post_provider_credential_mutation_error(
    error: ManagedProviderCredentialErrorV1,
) -> MailGmailOAuthDispatchErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_gmail_oauth_post_provider_mutation_error={error:?}");
    }
    MailGmailOAuthDispatchErrorV1::OutcomeUnknown
}

fn classify_provider_error(error: GmailAdapterErrorV1) -> MailGmailOAuthDispatchErrorV1 {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_mail_gmail_oauth_provider_error={error:?}");
    }
    match error {
        GmailAdapterErrorV1::InvalidRequest => MailGmailOAuthDispatchErrorV1::Rejected,
        GmailAdapterErrorV1::OAuthProviderError(_) => MailGmailOAuthDispatchErrorV1::Rejected,
        GmailAdapterErrorV1::ProviderStatus(status)
            if (400..500).contains(&status) && status != 408 && status != 429 =>
        {
            MailGmailOAuthDispatchErrorV1::Rejected
        }
        GmailAdapterErrorV1::Transport
        | GmailAdapterErrorV1::ProviderStatus(_)
        | GmailAdapterErrorV1::InvalidResponse => MailGmailOAuthDispatchErrorV1::OutcomeUnknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_failures_distinguish_rejection_from_unknown_outcome() {
        assert_eq!(
            classify_provider_error(GmailAdapterErrorV1::ProviderStatus(400)),
            MailGmailOAuthDispatchErrorV1::Rejected
        );
        assert_eq!(
            classify_provider_error(GmailAdapterErrorV1::ProviderStatus(429)),
            MailGmailOAuthDispatchErrorV1::OutcomeUnknown
        );
        assert_eq!(
            classify_provider_error(GmailAdapterErrorV1::Transport),
            MailGmailOAuthDispatchErrorV1::OutcomeUnknown
        );
    }

    #[test]
    fn credential_failures_fail_closed_by_authority_and_availability() {
        assert_eq!(
            classify_credential_resolution_error(ManagedProviderCredentialErrorV1::Rejected),
            MailGmailOAuthDispatchErrorV1::Rejected
        );
        assert_eq!(
            classify_credential_resolution_error(ManagedProviderCredentialErrorV1::Unavailable),
            MailGmailOAuthDispatchErrorV1::OutcomeUnknown
        );
        assert_eq!(
            classify_credential_resolution_error(ManagedProviderCredentialErrorV1::InvalidContext),
            MailGmailOAuthDispatchErrorV1::InvalidStoredOperation
        );
    }

    #[test]
    fn post_provider_credential_mutation_is_always_outcome_unknown() {
        for error in [
            ManagedProviderCredentialErrorV1::Rejected,
            ManagedProviderCredentialErrorV1::Unavailable,
            ManagedProviderCredentialErrorV1::InvalidContext,
        ] {
            assert_eq!(
                classify_post_provider_credential_mutation_error(error),
                MailGmailOAuthDispatchErrorV1::OutcomeUnknown
            );
        }
    }
}

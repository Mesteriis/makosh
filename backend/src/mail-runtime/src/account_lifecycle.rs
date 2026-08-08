//! Mail account lifecycle coordination across Mail persistence and Vault.

use std::os::unix::net::UnixStream;

use makosh_mail_api::{
    MailCredentialPurpose,
    account_lifecycle::{
        MailAccountLifecycleActionV1, MailAccountLifecycleCommandV1, MailAccountLifecycleReceiptV1,
        MailAccountLifecycleRetryV1, MailAccountLifecycleStateV1,
        MailAccountLifecycleStatusRequestV1, MailCredentialLifecycleProgressV1,
        MailCredentialLifecycleStateV1, validate_lifecycle_retry,
        validate_lifecycle_status_request,
    },
};
use makosh_mail_persistence::{MailDurablePersistence, MailDurablePersistenceError};
use makosh_managed_vault_client::{
    ManagedProviderCredentialClientV2, ManagedProviderCredentialContextV1,
    ManagedProviderCredentialErrorV1, ManagedProviderCredentialRequestV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, RejectManagedControlRequestsV2,
};
use makosh_vault_protocol::SecretClassV1;

use crate::{
    admission::MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
    managed::execute_blocking_provider_credential_request,
};

#[derive(Debug)]
pub enum MailAccountLifecycleRuntimeErrorV1 {
    Admission,
    Persistence(MailDurablePersistenceError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAccountLifecycleCoordinatorV1 {
    provider_io_quiesced: bool,
}

impl MailAccountLifecycleCoordinatorV1 {
    #[must_use]
    pub const fn new(provider_io_quiesced: bool) -> Self {
        Self {
            provider_io_quiesced,
        }
    }

    #[must_use]
    pub const fn provider_io_permitted(&self) -> bool {
        !self.provider_io_quiesced
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn begin(
        &mut self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
        provider_context: &ManagedProviderCredentialContextV1,
        durable: &MailDurablePersistence,
        command: &MailAccountLifecycleCommandV1,
        action: MailAccountLifecycleActionV1,
        configuration_instance_id: &str,
        requested_at_unix_seconds: i64,
    ) -> Result<MailAccountLifecycleReceiptV1, MailAccountLifecycleRuntimeErrorV1> {
        self.provider_io_quiesced = true;
        let begin = durable
            .begin_account_lifecycle(
                command,
                action,
                configuration_instance_id,
                requested_at_unix_seconds,
            )
            .await
            .map_err(MailAccountLifecycleRuntimeErrorV1::Persistence)?;
        if !begin.created
            || matches!(
                begin.receipt.state,
                MailAccountLifecycleStateV1::Completed | MailAccountLifecycleStateV1::Rejected
            )
        {
            return Ok(begin.receipt);
        }
        execute_pending_credentials(
            control_channel,
            provider_context,
            durable,
            begin.receipt,
            configuration_instance_id,
            requested_at_unix_seconds,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn retry(
        &mut self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
        provider_context: &ManagedProviderCredentialContextV1,
        durable: &MailDurablePersistence,
        retry: &MailAccountLifecycleRetryV1,
        configuration_instance_id: &str,
        requested_at_unix_seconds: i64,
    ) -> Result<MailAccountLifecycleReceiptV1, MailAccountLifecycleRuntimeErrorV1> {
        validate_lifecycle_retry(retry)
            .map_err(|_| MailAccountLifecycleRuntimeErrorV1::Admission)?;
        self.provider_io_quiesced = true;
        let receipt = durable
            .account_lifecycle_receipt(&retry.connection_id, &retry.operation_id)
            .await
            .map_err(MailAccountLifecycleRuntimeErrorV1::Persistence)?
            .ok_or(MailAccountLifecycleRuntimeErrorV1::Admission)?;
        if receipt.lifecycle_revision != retry.expected_lifecycle_revision
            || matches!(
                receipt.state,
                MailAccountLifecycleStateV1::Completed | MailAccountLifecycleStateV1::Rejected
            )
        {
            return (receipt.lifecycle_revision == retry.expected_lifecycle_revision)
                .then_some(receipt)
                .ok_or(MailAccountLifecycleRuntimeErrorV1::Admission);
        }
        execute_pending_credentials(
            control_channel,
            provider_context,
            durable,
            receipt,
            configuration_instance_id,
            requested_at_unix_seconds,
        )
        .await
    }

    pub async fn status(
        &self,
        durable: &MailDurablePersistence,
        request: &MailAccountLifecycleStatusRequestV1,
    ) -> Result<MailAccountLifecycleReceiptV1, MailAccountLifecycleRuntimeErrorV1> {
        validate_lifecycle_status_request(request)
            .map_err(|_| MailAccountLifecycleRuntimeErrorV1::Admission)?;
        durable
            .account_lifecycle_receipt(&request.connection_id, &request.operation_id)
            .await
            .map_err(MailAccountLifecycleRuntimeErrorV1::Persistence)?
            .ok_or(MailAccountLifecycleRuntimeErrorV1::Admission)
    }
}

async fn execute_pending_credentials(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    provider_context: &ManagedProviderCredentialContextV1,
    durable: &MailDurablePersistence,
    mut receipt: MailAccountLifecycleReceiptV1,
    configuration_instance_id: &str,
    requested_at_unix_seconds: i64,
) -> Result<MailAccountLifecycleReceiptV1, MailAccountLifecycleRuntimeErrorV1> {
    for progress in receipt.credentials.clone() {
        if !matches!(
            progress.state,
            MailCredentialLifecycleStateV1::Pending
                | MailCredentialLifecycleStateV1::OutcomeUnknown
        ) {
            continue;
        }
        let state = mutate_vault_credential(
            control_channel,
            provider_context,
            configuration_instance_id,
            &progress,
            receipt.action,
        );
        receipt = durable
            .record_account_lifecycle_progress(
                &receipt.connection_id,
                &receipt.operation_id,
                progress.purpose,
                state,
                requested_at_unix_seconds,
            )
            .await
            .map_err(MailAccountLifecycleRuntimeErrorV1::Persistence)?;
    }
    Ok(receipt)
}

fn mutate_vault_credential(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    provider_context: &ManagedProviderCredentialContextV1,
    configuration_instance_id: &str,
    progress: &MailCredentialLifecycleProgressV1,
    action: MailAccountLifecycleActionV1,
) -> MailCredentialLifecycleStateV1 {
    let (purpose, secret_class) = match progress.purpose {
        makosh_mail_api::account::MailCredentialPurposeV1::ImapPassword => (
            MailCredentialPurpose::ImapPassword,
            SecretClassV1::ProviderCredential,
        ),
        makosh_mail_api::account::MailCredentialPurposeV1::SmtpPassword => (
            MailCredentialPurpose::SmtpPassword,
            SecretClassV1::ProviderCredential,
        ),
        makosh_mail_api::account::MailCredentialPurposeV1::GmailAccessToken => (
            MailCredentialPurpose::GmailAccessToken,
            SecretClassV1::ProviderCredential,
        ),
        makosh_mail_api::account::MailCredentialPurposeV1::GmailRefreshCredential => (
            MailCredentialPurpose::GmailRefreshCredential,
            SecretClassV1::OAuthRefreshCredential,
        ),
        makosh_mail_api::account::MailCredentialPurposeV1::IcloudCardDavPassword => (
            MailCredentialPurpose::IcloudCardDavPassword,
            SecretClassV1::ProviderCredential,
        ),
    };
    let request = ManagedProviderCredentialRequestV1 {
        configuration_instance_id,
        purpose_id: purpose.as_str(),
        credential_revision: progress.credential_revision,
        ttl_seconds: MAIL_CREDENTIAL_LEASE_TTL_SECONDS,
        secret_class,
    };
    let result = execute_blocking_provider_credential_request(control_channel, |channel| {
        let mut client = ManagedProviderCredentialClientV2::new(channel);
        let mut dispatcher = RejectManagedControlRequestsV2;
        match action {
            MailAccountLifecycleActionV1::Retire => {
                client.retire_once(&mut dispatcher, provider_context, request)
            }
            MailAccountLifecycleActionV1::Delete => {
                client.delete_once(&mut dispatcher, provider_context, request)
            }
        }
    });
    match result {
        Ok(()) => MailCredentialLifecycleStateV1::Completed,
        Err(ManagedProviderCredentialErrorV1::InvalidContext) => {
            MailCredentialLifecycleStateV1::Rejected
        }
        Err(
            ManagedProviderCredentialErrorV1::Rejected
            | ManagedProviderCredentialErrorV1::Unavailable,
        ) => MailCredentialLifecycleStateV1::OutcomeUnknown,
    }
}

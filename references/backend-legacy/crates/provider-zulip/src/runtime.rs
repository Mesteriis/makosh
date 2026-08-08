use chrono::Utc;
use makosh_provider_api::{
    CredentialLease, ProviderCommandDisposition, ProviderCommandEnvelope, ProviderCommandResult,
    ProviderId, ProviderManifest, ProviderRuntimePort, ProviderRuntimePortError,
    ProviderRuntimePortFuture, RuntimeTopology,
};
use serde_json::Value;

use crate::client::{ZulipApiClient, ZulipClientConfig};
use crate::command_execution::{
    ZulipCommandExecutionError, ZulipExecutableCommand, execute_zulip_command,
};

const ZULIP_PROVIDER_ID: &str = "zulip";

/// Account-scoped, non-secret configuration for a Zulip runtime instance.
///
/// The API key is deliberately absent: each command receives it through an
/// expiring `CredentialLease` owned by the vault boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipRuntimeConfig {
    account_id: String,
    base_url: String,
    bot_email: String,
}

impl ZulipRuntimeConfig {
    pub fn new(
        account_id: impl Into<String>,
        base_url: impl Into<String>,
        bot_email: impl Into<String>,
    ) -> Result<Self, ZulipRuntimeConfigError> {
        let account_id = required("account_id", account_id.into())?;
        let base_url = required("base_url", base_url.into())?;
        let bot_email = required("bot_email", bot_email.into())?;
        Ok(Self {
            account_id,
            base_url,
            bot_email,
        })
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }
}

/// In-process adapter for exactly one Zulip account.
///
/// Composition owns construction. The provider crate owns protocol validation
/// and remote execution, but neither persistence, vault lookup nor blob paths.
#[derive(Clone)]
pub struct ZulipInProcessRuntime {
    config: ZulipRuntimeConfig,
}

impl ZulipInProcessRuntime {
    pub fn new(config: ZulipRuntimeConfig) -> Self {
        Self { config }
    }
}

impl ProviderRuntimePort for ZulipInProcessRuntime {
    fn manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            ProviderId::parse(ZULIP_PROVIDER_ID).expect("static Zulip provider ID is valid"),
            1,
            ["messages.read", "messages.send", "attachments.read"],
            [RuntimeTopology::InProcess, RuntimeTopology::SharedConnector],
        )
        .expect("static Zulip manifest is valid")
    }

    fn execute<'a>(
        &'a self,
        command: &'a ProviderCommandEnvelope,
        credential: CredentialLease,
    ) -> ProviderRuntimePortFuture<'a> {
        Box::pin(async move {
            validate_command_binding(&self.config, command, &credential)?;
            let executable = executable_command(command)?;
            let api_key = std::str::from_utf8(credential.secret()).map_err(|_| {
                ProviderRuntimePortError::new("execute", "invalid_credential_encoding", false)
            })?;
            let client = ZulipApiClient::new(
                ZulipClientConfig::new(&self.config.base_url, &self.config.bot_email, api_key)
                    .map_err(|_| {
                        ProviderRuntimePortError::new(
                            "execute",
                            "invalid_runtime_configuration",
                            false,
                        )
                    })?,
            );
            let outcome = execute_zulip_command(&client, &executable)
                .await
                .map_err(runtime_error)?;
            ProviderCommandResult::new(
                command.command_id.clone(),
                command.provider_id.clone(),
                command.account_id.clone(),
                command.lease_epoch,
                Utc::now(),
                ProviderCommandDisposition::Completed,
                command.payload_version,
                outcome.result_payload,
            )
            .map_err(|error| ProviderRuntimePortError::new("execute", error.code(), false))
        })
    }
}

fn validate_command_binding(
    config: &ZulipRuntimeConfig,
    command: &ProviderCommandEnvelope,
    credential: &CredentialLease,
) -> Result<(), ProviderRuntimePortError> {
    if command.provider_id.as_str() != ZULIP_PROVIDER_ID {
        return Err(ProviderRuntimePortError::new(
            "execute",
            "provider_mismatch",
            false,
        ));
    }
    if command.account_id != config.account_id {
        return Err(ProviderRuntimePortError::new(
            "execute",
            "account_mismatch",
            false,
        ));
    }
    if command.deadline <= Utc::now() {
        return Err(ProviderRuntimePortError::new(
            "execute",
            "command_deadline_expired",
            false,
        ));
    }
    if credential.provider_id != ZULIP_PROVIDER_ID
        || credential.account_id != command.account_id
        || credential.epoch != command.lease_epoch
    {
        return Err(ProviderRuntimePortError::new(
            "execute",
            "credential_lease_mismatch",
            false,
        ));
    }
    if credential.is_expired_at(Utc::now()) {
        return Err(ProviderRuntimePortError::new(
            "execute",
            "credential_lease_expired",
            false,
        ));
    }
    Ok(())
}

fn executable_command(
    command: &ProviderCommandEnvelope,
) -> Result<ZulipExecutableCommand, ProviderRuntimePortError> {
    let command_kind = required_payload_string(&command.payload, "command_kind")?;
    if matches!(
        command_kind,
        "upload_file" | "send_stream_message_with_upload" | "send_direct_message_with_upload"
    ) {
        return Err(ProviderRuntimePortError::new(
            "execute",
            "blob_materialization_required",
            false,
        ));
    }
    let provider_message_id = optional_payload_string(&command.payload, "provider_message_id")?;
    let payload = command
        .payload
        .get("payload")
        .filter(|value| value.is_object())
        .cloned()
        .ok_or_else(|| {
            ProviderRuntimePortError::new("execute", "invalid_command_payload", false)
        })?;
    Ok(ZulipExecutableCommand::new(
        command.command_id.clone(),
        command_kind,
        provider_message_id,
        payload,
    ))
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, ProviderRuntimePortError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ProviderRuntimePortError::new("execute", "invalid_command_payload", false))
}

fn optional_payload_string(
    payload: &Value,
    field: &'static str,
) -> Result<Option<String>, ProviderRuntimePortError> {
    match payload.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(Some(value.trim().to_owned())),
        _ => Err(ProviderRuntimePortError::new(
            "execute",
            "invalid_command_payload",
            false,
        )),
    }
}

fn runtime_error(error: ZulipCommandExecutionError) -> ProviderRuntimePortError {
    let retryable = match &error {
        ZulipCommandExecutionError::ProviderApi { status } => *status >= 500,
        ZulipCommandExecutionError::Transport
        | ZulipCommandExecutionError::InvalidProviderResponse => true,
        ZulipCommandExecutionError::InvalidCommand { .. }
        | ZulipCommandExecutionError::InvalidClientConfiguration => false,
    };
    ProviderRuntimePortError::new("execute", error.error_kind(), retryable)
}

fn required(field: &'static str, value: String) -> Result<String, ZulipRuntimeConfigError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ZulipRuntimeConfigError::EmptyField(field));
    }
    Ok(value.to_owned())
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ZulipRuntimeConfigError {
    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

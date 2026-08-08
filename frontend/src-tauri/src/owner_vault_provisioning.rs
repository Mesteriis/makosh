//! Thin Tauri commands for the provider-neutral provisioning host adapter.

use makosh_owner_vault_provisioning_host::{
    AuthorizedProvisioningV1, CommittedProvisioningReceiptV1, OwnerVaultProvisioningHostV1,
    SanitizedProvisioningReceiptV1, SealedProvisioningCommandV1, StartedProvisioningHostSessionV1,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Default)]
pub(crate) struct OwnerVaultProvisioningHostStateV1 {
    host: OwnerVaultProvisioningHostV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartedProvisioningHostSessionResponseV1 {
    host_session_id: String,
    response_recipient_hpke_public_key_x25519: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SealProvisioningCommandRequestV1 {
    host_session_id: String,
    operation_id: Vec<u8>,
    action: i32,
    secret_class: i32,
    secret_payload: Vec<u8>,
    authorized: AuthorizedProvisioningRequestV1,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthorizedProvisioningRequestV1 {
    vault_runtime_generation: String,
    vault_hpke_public_key_x25519: Vec<u8>,
    audience_registration_id: String,
    audience_runtime_instance_id: String,
    audience_runtime_generation: String,
    audience_grant_epoch: String,
    lease_request_id: Vec<u8>,
    lease_operation_digest_sha256: Vec<u8>,
    command_request_id: Vec<u8>,
    lease_response_hpke_encapped_key: Vec<u8>,
    lease_response_ciphertext: Vec<u8>,
    lease_response_hpke_authentication_tag: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SealedProvisioningCommandResponseV1 {
    operation_digest_sha256: Vec<u8>,
    hpke_encapped_key: Vec<u8>,
    ciphertext: Vec<u8>,
    hpke_authentication_tag: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenProvisioningReceiptRequestV1 {
    host_session_id: String,
    committed: CommittedProvisioningReceiptRequestV1,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommittedProvisioningReceiptRequestV1 {
    vault_runtime_generation: String,
    command_request_id: Vec<u8>,
    operation_digest_sha256: Vec<u8>,
    receipt_hpke_encapped_key: Vec<u8>,
    receipt_ciphertext: Vec<u8>,
    receipt_hpke_authentication_tag: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SanitizedProvisioningReceiptResponseV1 {
    operation_id: Vec<u8>,
    action: i32,
    secret_revision: String,
    state: u8,
}

#[tauri::command]
pub(crate) fn owner_vault_provisioning_host_start(
    state: State<'_, OwnerVaultProvisioningHostStateV1>,
) -> Result<StartedProvisioningHostSessionResponseV1, String> {
    state.host.start().map(Into::into).map_err(sanitized_error)
}

#[tauri::command]
pub(crate) fn owner_vault_provisioning_host_seal(
    state: State<'_, OwnerVaultProvisioningHostStateV1>,
    request: SealProvisioningCommandRequestV1,
) -> Result<SealedProvisioningCommandResponseV1, String> {
    let authorized = AuthorizedProvisioningV1 {
        vault_runtime_generation: unsigned(&request.authorized.vault_runtime_generation)?,
        vault_hpke_public_key_x25519: array(request.authorized.vault_hpke_public_key_x25519)?,
        audience_registration_id: request.authorized.audience_registration_id,
        audience_runtime_instance_id: request.authorized.audience_runtime_instance_id,
        audience_runtime_generation: unsigned(&request.authorized.audience_runtime_generation)?,
        audience_grant_epoch: unsigned(&request.authorized.audience_grant_epoch)?,
        lease_request_id: array(request.authorized.lease_request_id)?,
        lease_operation_digest_sha256: array(request.authorized.lease_operation_digest_sha256)?,
        command_request_id: array(request.authorized.command_request_id)?,
        lease_response_hpke_encapped_key: request.authorized.lease_response_hpke_encapped_key,
        lease_response_ciphertext: request.authorized.lease_response_ciphertext,
        lease_response_hpke_authentication_tag: request
            .authorized
            .lease_response_hpke_authentication_tag,
    };
    state
        .host
        .seal(
            &request.host_session_id,
            authorized,
            array(request.operation_id)?,
            request.action,
            request.secret_class,
            request.secret_payload,
        )
        .map(Into::into)
        .map_err(sanitized_error)
}

#[tauri::command]
pub(crate) fn owner_vault_provisioning_host_open_receipt(
    state: State<'_, OwnerVaultProvisioningHostStateV1>,
    request: OpenProvisioningReceiptRequestV1,
) -> Result<SanitizedProvisioningReceiptResponseV1, String> {
    let committed = CommittedProvisioningReceiptV1 {
        vault_runtime_generation: unsigned(&request.committed.vault_runtime_generation)?,
        command_request_id: array(request.committed.command_request_id)?,
        operation_digest_sha256: array(request.committed.operation_digest_sha256)?,
        receipt_hpke_encapped_key: request.committed.receipt_hpke_encapped_key,
        receipt_ciphertext: request.committed.receipt_ciphertext,
        receipt_hpke_authentication_tag: request.committed.receipt_hpke_authentication_tag,
    };
    state
        .host
        .open_receipt(&request.host_session_id, committed)
        .map(Into::into)
        .map_err(sanitized_error)
}

#[tauri::command]
pub(crate) fn owner_vault_provisioning_host_cancel(
    state: State<'_, OwnerVaultProvisioningHostStateV1>,
    host_session_id: String,
) -> Result<(), String> {
    state.host.cancel(&host_session_id).map_err(sanitized_error)
}

impl From<StartedProvisioningHostSessionV1> for StartedProvisioningHostSessionResponseV1 {
    fn from(value: StartedProvisioningHostSessionV1) -> Self {
        Self {
            host_session_id: value.host_session_id,
            response_recipient_hpke_public_key_x25519: value
                .response_recipient_hpke_public_key_x25519
                .to_vec(),
        }
    }
}

impl From<SealedProvisioningCommandV1> for SealedProvisioningCommandResponseV1 {
    fn from(value: SealedProvisioningCommandV1) -> Self {
        Self {
            operation_digest_sha256: value.operation_digest_sha256.to_vec(),
            hpke_encapped_key: value.hpke_encapped_key,
            ciphertext: value.ciphertext,
            hpke_authentication_tag: value.hpke_authentication_tag,
        }
    }
}

impl From<SanitizedProvisioningReceiptV1> for SanitizedProvisioningReceiptResponseV1 {
    fn from(value: SanitizedProvisioningReceiptV1) -> Self {
        Self {
            operation_id: value.operation_id.to_vec(),
            action: value.action,
            secret_revision: value.secret_revision.to_string(),
            state: value.state,
        }
    }
}

fn array<const N: usize>(value: Vec<u8>) -> Result<[u8; N], String> {
    value
        .try_into()
        .map_err(|_| "owner Vault provisioning request is invalid".to_owned())
}

fn unsigned(value: &str) -> Result<u64, String> {
    value
        .parse()
        .map_err(|_| "owner Vault provisioning request is invalid".to_owned())
}

fn sanitized_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

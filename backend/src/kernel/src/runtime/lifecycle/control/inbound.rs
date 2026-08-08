//! Bounded inbound Vault route messages on a verified managed-runtime channel.

use std::io::ErrorKind;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    ManagedRuntimeBlobCustodyDelegationDeliveryV1, ManagedRuntimeBlobCustodyDelegationRequestV1,
    ManagedRuntimeBlobCustodyReleaseDeliveryV1, ManagedRuntimeBlobCustodyReleaseRequestV1,
    ManagedRuntimeBlobSessionDeliveryV1, ManagedRuntimeBlobSessionRequestV1,
    ManagedRuntimeClientRealtimePublishRequestV1, ManagedRuntimeClientRealtimePublishResponseV1,
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeEventCredentialDeliveryV1, ManagedRuntimeEventCredentialRequestV1,
    ManagedRuntimeModuleQueryRequestV1, ManagedRuntimeModuleQueryResponseV1,
    ManagedRuntimeModuleRequestRequestV1, ManagedRuntimeModuleRequestResponseV1,
    ManagedRuntimeOwnerDerivedKeyDeliveryV1, ManagedRuntimeOwnerDerivedKeyRequestV1,
    ManagedRuntimeProviderCredentialDeliveryV1, ManagedRuntimeProviderCredentialRequestV1,
    ManagedRuntimeReadyRequestV1, ManagedRuntimeVaultRouteResponseV1, VaultCiphertextResponseV1,
    VaultCiphertextRouteV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use prost::Message;

use makosh_runtime_protocol::validation::vault::validate_vault_ciphertext_route_v1;

use super::{MAX_FRAME_BYTES, read_frame, write_frame};

const VAULT_ROUTE_FIELD_TAG: u8 = 0x2a;
const READY_FIELD_TAG: u8 = 0x12;
const EVENT_CREDENTIAL_FIELD_TAG: u8 = 0x1a;
const PROVIDER_CREDENTIAL_FIELD_TAG: u8 = 0x22;
const BLOB_SESSION_FIELD_TAG: u8 = 0x32;
const OWNER_DERIVED_KEY_FIELD_TAG: u8 = 0x3a;
const BLOB_CUSTODY_RELEASE_FIELD_TAG: u8 = 0x72;
const BLOB_CUSTODY_DELEGATION_FIELD_TAG: u8 = 0x7a;

pub(crate) enum ManagedRuntimeInboundRequestV1 {
    Ready(ManagedRuntimeReadyRequestV1),
    VaultRoute(VaultCiphertextRouteV1),
    EventCredential(ManagedRuntimeEventCredentialRequestV1),
    ProviderCredential(ManagedRuntimeProviderCredentialRequestV1),
    OwnerDerivedKey(ManagedRuntimeOwnerDerivedKeyRequestV1),
    BlobSession(ManagedRuntimeBlobSessionRequestV1),
    BlobCustodyDelegation(ManagedRuntimeBlobCustodyDelegationRequestV1),
    BlobCustodyRelease(ManagedRuntimeBlobCustodyReleaseRequestV1),
    ModuleQuery(ManagedRuntimeModuleQueryRequestV1),
    ModuleRequest(ManagedRuntimeModuleRequestRequestV1),
    ClientRealtime(ManagedRuntimeClientRealtimePublishRequestV1),
}

pub(crate) fn decode_typed_request(
    request: ManagedRuntimeControlRequestV1,
) -> Result<ManagedRuntimeInboundRequestV1, String> {
    match request.operation {
        Some(Operation::Ready(value)) => Ok(ManagedRuntimeInboundRequestV1::Ready(value)),
        Some(Operation::RouteVaultCiphertext(value)) => {
            let route = value
                .route
                .filter(|route| validate_vault_ciphertext_route_v1(route).is_ok())
                .ok_or_else(|| "managed runtime Vault route is invalid".to_owned())?;
            Ok(ManagedRuntimeInboundRequestV1::VaultRoute(route))
        }
        Some(Operation::IssueEventCredential(value)) if valid_event_credential_request(&value) => {
            Ok(ManagedRuntimeInboundRequestV1::EventCredential(value))
        }
        Some(Operation::IssueProviderCredential(value))
            if valid_provider_credential_request(&value) =>
        {
            Ok(ManagedRuntimeInboundRequestV1::ProviderCredential(value))
        }
        Some(Operation::IssueOwnerDerivedKey(value)) if valid_owner_derived_key_request(&value) => {
            Ok(ManagedRuntimeInboundRequestV1::OwnerDerivedKey(value))
        }
        Some(Operation::IssueBlobSession(value))
            if crate::platform::blob::session::valid_request(&value) =>
        {
            Ok(ManagedRuntimeInboundRequestV1::BlobSession(value))
        }
        Some(Operation::DelegateBlobCustody(value))
            if crate::platform::blob::session::valid_delegation_request(&value) =>
        {
            Ok(ManagedRuntimeInboundRequestV1::BlobCustodyDelegation(value))
        }
        Some(Operation::ReleaseBlobCustody(value))
            if crate::platform::blob::release::valid_request(&value) =>
        {
            Ok(ManagedRuntimeInboundRequestV1::BlobCustodyRelease(value))
        }
        Some(Operation::RouteModuleQuery(value))
            if makosh_runtime_protocol::validation::module_query::validate_module_query_request_v1(
                &value,
            )
            .is_ok() =>
        {
            Ok(ManagedRuntimeInboundRequestV1::ModuleQuery(value))
        }
        Some(Operation::RouteModuleRequest(value))
            if makosh_runtime_protocol::validation::module_request::validate_module_request_request_v1(
                &value,
            )
            .is_ok() =>
        {
            Ok(ManagedRuntimeInboundRequestV1::ModuleRequest(value))
        }
        Some(Operation::PublishClientRealtime(value))
            if makosh_runtime_protocol::validation::client_realtime::validate_managed_client_realtime_publish_request_v1(
                &value,
            )
            .is_ok() =>
        {
            Ok(ManagedRuntimeInboundRequestV1::ClientRealtime(value))
        }
        _ => Err("managed runtime control request is invalid".to_owned()),
    }
}

pub(crate) fn client_realtime_response(
    result: Result<ManagedRuntimeClientRealtimePublishResponseV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(response) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::ClientRealtimePublish(response)),
            error_code: String::new(),
        },
        Err(error) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: client_realtime_error_code(&error).to_owned(),
        },
    }
}

fn client_realtime_error_code(error: &str) -> &'static str {
    if error.contains("unavailable") {
        "UNAVAILABLE"
    } else if error.contains("stale") || error.contains("prohibited") {
        "REJECTED"
    } else {
        "INVALID"
    }
}

pub(crate) fn module_query_response(
    result: Result<ManagedRuntimeModuleQueryResponseV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(response)
            if makosh_runtime_protocol::validation::module_query::validate_module_query_response_v1(
                &response,
            )
            .is_ok() =>
        {
            ManagedRuntimeControlResponseV1 {
                result: Some(ControlResult::ModuleQueryRoute(response)),
                error_code: String::new(),
            }
        }
        Ok(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_module_query_invalid_response".to_owned(),
        },
        Err(error) => {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_managed_module_query_error={error}");
            }
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: module_query_error_code(&error).to_owned(),
            }
        }
    }
}

fn module_query_error_code(error: &str) -> &'static str {
    if error.ends_with(" is unavailable") {
        "managed_module_query_unavailable"
    } else if error.ends_with(" is ambiguous") {
        "managed_module_query_ambiguous"
    } else {
        "managed_module_query_denied"
    }
}

pub(crate) fn module_request_response(
    result: Result<ManagedRuntimeModuleRequestResponseV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(response)
            if makosh_runtime_protocol::validation::module_request::validate_module_request_response_v1(
                &response,
            )
            .is_ok() =>
        {
            ManagedRuntimeControlResponseV1 {
                result: Some(ControlResult::ModuleRequestRoute(response)),
                error_code: String::new(),
            }
        }
        Ok(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_module_request_invalid_response".to_owned(),
        },
        Err(error) => {
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!("developer_managed_module_request_error={error}");
            }
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: module_request_error_code(&error).to_owned(),
            }
        }
    }
}

fn module_request_error_code(error: &str) -> &'static str {
    if error.ends_with(" is unavailable") {
        "managed_module_request_unavailable"
    } else if error.ends_with(" is ambiguous") {
        "managed_module_request_ambiguous"
    } else {
        "managed_module_request_denied"
    }
}

pub(crate) fn try_receive_vault_route(
    channel: &mut UnixStream,
) -> Result<Option<VaultCiphertextRouteV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&VAULT_ROUTE_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime Vault route is invalid".to_owned())?;
    let Some(Operation::RouteVaultCiphertext(request)) = request.operation else {
        return Err("managed runtime Vault route is invalid".to_owned());
    };
    let Some(route) = request.route else {
        return Err("managed runtime Vault route is invalid".to_owned());
    };
    read_frame(channel)?;
    Ok(Some(route))
}

pub(crate) fn try_receive_ready(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeReadyRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&READY_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime ready signal is invalid".to_owned())?;
    let Some(Operation::Ready(ready)) = request.operation else {
        return Ok(None);
    };
    read_frame(channel)?;
    Ok(Some(ready))
}

pub(crate) fn try_receive_event_credential(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeEventCredentialRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&EVENT_CREDENTIAL_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime event credential request is invalid".to_owned())?;
    let Some(Operation::IssueEventCredential(value)) = request.operation else {
        return Err("managed runtime event credential request is invalid".to_owned());
    };
    valid_event_credential_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime event credential request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn event_credential_request(
    frame: &[u8],
) -> Result<Option<ManagedRuntimeEventCredentialRequestV1>, String> {
    if frame.first() != Some(&EVENT_CREDENTIAL_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame)
        .map_err(|_| "managed runtime event credential request is invalid".to_owned())?;
    let Some(Operation::IssueEventCredential(value)) = request.operation else {
        return Err("managed runtime event credential request is invalid".to_owned());
    };
    valid_event_credential_request(&value)
        .then_some(value)
        .map(Some)
        .ok_or_else(|| "managed runtime event credential request is invalid".to_owned())
}

pub(crate) fn respond_vault_route(
    channel: &mut UnixStream,
    result: Result<VaultCiphertextResponseV1, String>,
) -> Result<(), String> {
    write_frame(channel, &vault_route_response(result).encode_to_vec())
}

pub(crate) fn vault_route_response(
    result: Result<VaultCiphertextResponseV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(response) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::VaultRoute(
                ManagedRuntimeVaultRouteResponseV1 {
                    response: Some(response),
                    error_code: String::new(),
                },
            )),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::VaultRoute(
                ManagedRuntimeVaultRouteResponseV1 {
                    response: None,
                    error_code: "managed_vault_route_denied".to_owned(),
                },
            )),
            error_code: "managed_vault_route_denied".to_owned(),
        },
    }
}

pub(crate) fn respond_event_credential(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeEventCredentialDeliveryV1, String>,
) -> Result<(), String> {
    write_frame(channel, &event_credential_response(result).encode_to_vec())
}

pub(crate) fn event_credential_response(
    result: Result<ManagedRuntimeEventCredentialDeliveryV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::EventCredentialDelivery(delivery)),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_event_credential_denied".to_owned(),
        },
    }
}

pub(crate) fn try_receive_provider_credential(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeProviderCredentialRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&PROVIDER_CREDENTIAL_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime provider credential request is invalid".to_owned())?;
    let Some(Operation::IssueProviderCredential(value)) = request.operation else {
        return Err("managed runtime provider credential request is invalid".to_owned());
    };
    valid_provider_credential_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime provider credential request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn provider_credential_request(
    frame: &[u8],
) -> Result<Option<ManagedRuntimeProviderCredentialRequestV1>, String> {
    if frame.first() != Some(&PROVIDER_CREDENTIAL_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame)
        .map_err(|_| "managed runtime provider credential request is invalid".to_owned())?;
    let Some(Operation::IssueProviderCredential(value)) = request.operation else {
        return Err("managed runtime provider credential request is invalid".to_owned());
    };
    valid_provider_credential_request(&value)
        .then_some(value)
        .map(Some)
        .ok_or_else(|| "managed runtime provider credential request is invalid".to_owned())
}

pub(crate) fn respond_provider_credential(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeProviderCredentialDeliveryV1, String>,
) -> Result<(), String> {
    write_frame(
        channel,
        &provider_credential_response(result).encode_to_vec(),
    )
}

pub(crate) fn provider_credential_response(
    result: Result<ManagedRuntimeProviderCredentialDeliveryV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::ProviderCredentialDelivery(delivery)),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_provider_credential_denied".to_owned(),
        },
    }
}

pub(crate) fn try_receive_owner_derived_key(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeOwnerDerivedKeyRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&OWNER_DERIVED_KEY_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime owner-derived key request is invalid".to_owned())?;
    let Some(Operation::IssueOwnerDerivedKey(value)) = request.operation else {
        return Err("managed runtime owner-derived key request is invalid".to_owned());
    };
    valid_owner_derived_key_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime owner-derived key request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn owner_derived_key_request(
    frame: &[u8],
) -> Result<Option<ManagedRuntimeOwnerDerivedKeyRequestV1>, String> {
    if frame.first() != Some(&OWNER_DERIVED_KEY_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame)
        .map_err(|_| "managed runtime owner-derived key request is invalid".to_owned())?;
    let Some(Operation::IssueOwnerDerivedKey(value)) = request.operation else {
        return Err("managed runtime owner-derived key request is invalid".to_owned());
    };
    valid_owner_derived_key_request(&value)
        .then_some(value)
        .map(Some)
        .ok_or_else(|| "managed runtime owner-derived key request is invalid".to_owned())
}

pub(crate) fn respond_owner_derived_key(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeOwnerDerivedKeyDeliveryV1, String>,
) -> Result<(), String> {
    write_frame(channel, &owner_derived_key_response(result).encode_to_vec())
}

pub(crate) fn owner_derived_key_response(
    result: Result<ManagedRuntimeOwnerDerivedKeyDeliveryV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::OwnerDerivedKeyDelivery(delivery)),
            error_code: String::new(),
        },
        Err(_) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "managed_owner_derived_key_denied".to_owned(),
        },
    }
}

pub(crate) fn try_receive_blob_session(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeBlobSessionRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&BLOB_SESSION_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
        .map_err(|_| "managed runtime Blob session request is invalid".to_owned())?;
    let Some(Operation::IssueBlobSession(value)) = request.operation else {
        return Err("managed runtime Blob session request is invalid".to_owned());
    };
    crate::platform::blob::session::valid_request(&value)
        .then_some(())
        .ok_or_else(|| "managed runtime Blob session request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(value))
}

pub(crate) fn blob_session_request(
    frame: &[u8],
) -> Result<Option<ManagedRuntimeBlobSessionRequestV1>, String> {
    if frame.first() != Some(&BLOB_SESSION_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame)
        .map_err(|_| "managed runtime Blob session request is invalid".to_owned())?;
    let Some(Operation::IssueBlobSession(value)) = request.operation else {
        return Err("managed runtime Blob session request is invalid".to_owned());
    };
    crate::platform::blob::session::valid_request(&value)
        .then_some(value)
        .map(Some)
        .ok_or_else(|| "managed runtime Blob session request is invalid".to_owned())
}

pub(crate) fn respond_blob_session(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeBlobSessionDeliveryV1, String>,
) -> Result<(), String> {
    write_frame(channel, &blob_session_response(result).encode_to_vec())
}

pub(crate) fn blob_session_response(
    result: Result<ManagedRuntimeBlobSessionDeliveryV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::BlobSessionDelivery(delivery)),
            error_code: String::new(),
        },
        Err(error) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: blob_session_error_code(&error).to_owned(),
        },
    }
}

fn blob_session_error_code(error: &str) -> &'static str {
    if error.ends_with(" is unavailable") {
        "managed_blob_session_unavailable"
    } else {
        "managed_blob_session_denied"
    }
}

pub(crate) fn try_receive_blob_custody_delegation(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeBlobCustodyDelegationRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&BLOB_CUSTODY_DELEGATION_FIELD_TAG) {
        return Ok(None);
    }
    let request = blob_custody_delegation_request(&frame)?
        .ok_or_else(|| "managed runtime Blob custody delegation request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(request))
}

pub(crate) fn blob_custody_delegation_request(
    frame: &[u8],
) -> Result<Option<ManagedRuntimeBlobCustodyDelegationRequestV1>, String> {
    if frame.first() != Some(&BLOB_CUSTODY_DELEGATION_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame)
        .map_err(|_| "managed runtime Blob custody delegation request is invalid".to_owned())?;
    let Some(Operation::DelegateBlobCustody(value)) = request.operation else {
        return Err("managed runtime Blob custody delegation request is invalid".to_owned());
    };
    crate::platform::blob::session::valid_delegation_request(&value)
        .then_some(value)
        .map(Some)
        .ok_or_else(|| "managed runtime Blob custody delegation request is invalid".to_owned())
}

pub(crate) fn respond_blob_custody_delegation(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeBlobCustodyDelegationDeliveryV1, String>,
) -> Result<(), String> {
    write_frame(
        channel,
        &blob_custody_delegation_response(result).encode_to_vec(),
    )
}

pub(crate) fn blob_custody_delegation_response(
    result: Result<ManagedRuntimeBlobCustodyDelegationDeliveryV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::BlobCustodyDelegation(delivery)),
            error_code: String::new(),
        },
        Err(error) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: if error.contains("unavailable") {
                "managed_blob_custody_delegation_unavailable"
            } else {
                "managed_blob_custody_delegation_denied"
            }
            .to_owned(),
        },
    }
}

pub(crate) fn try_receive_blob_custody_release(
    channel: &mut UnixStream,
) -> Result<Option<ManagedRuntimeBlobCustodyReleaseRequestV1>, String> {
    let Some(frame) = peek_complete_frame(channel)? else {
        return Ok(None);
    };
    if frame.first() != Some(&BLOB_CUSTODY_RELEASE_FIELD_TAG) {
        return Ok(None);
    }
    let request = blob_custody_release_request(&frame)?
        .ok_or_else(|| "managed runtime Blob custody release request is invalid".to_owned())?;
    read_frame(channel)?;
    Ok(Some(request))
}

pub(crate) fn blob_custody_release_request(
    frame: &[u8],
) -> Result<Option<ManagedRuntimeBlobCustodyReleaseRequestV1>, String> {
    if frame.first() != Some(&BLOB_CUSTODY_RELEASE_FIELD_TAG) {
        return Ok(None);
    }
    let request = ManagedRuntimeControlRequestV1::decode(frame)
        .map_err(|_| "managed runtime Blob custody release request is invalid".to_owned())?;
    let Some(Operation::ReleaseBlobCustody(value)) = request.operation else {
        return Err("managed runtime Blob custody release request is invalid".to_owned());
    };
    crate::platform::blob::release::valid_request(&value)
        .then_some(value)
        .map(Some)
        .ok_or_else(|| "managed runtime Blob custody release request is invalid".to_owned())
}

pub(crate) fn respond_blob_custody_release(
    channel: &mut UnixStream,
    result: Result<ManagedRuntimeBlobCustodyReleaseDeliveryV1, String>,
) -> Result<(), String> {
    write_frame(
        channel,
        &blob_custody_release_response(result).encode_to_vec(),
    )
}

pub(crate) fn blob_custody_release_response(
    result: Result<ManagedRuntimeBlobCustodyReleaseDeliveryV1, String>,
) -> ManagedRuntimeControlResponseV1 {
    match result {
        Ok(delivery) => ManagedRuntimeControlResponseV1 {
            result: Some(ControlResult::BlobCustodyRelease(delivery)),
            error_code: String::new(),
        },
        Err(error) => ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: if error.contains("unavailable") {
                "managed_blob_custody_release_unavailable"
            } else {
                "managed_blob_custody_release_denied"
            }
            .to_owned(),
        },
    }
}

fn peek_complete_frame(channel: &mut UnixStream) -> Result<Option<Vec<u8>>, String> {
    let mut header = [0_u8; 5];
    let header_length = match peek(channel, &mut header) {
        Ok(length) => length,
        Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let Some((prefix_length, payload_length)) = decode_length(&header[..header_length])? else {
        return Ok(None);
    };
    let total_length = prefix_length
        .checked_add(payload_length)
        .ok_or_else(|| "managed runtime Vault route is invalid".to_owned())?;
    let mut frame = vec![0_u8; total_length];
    let available = match peek(channel, &mut frame) {
        Ok(length) => length,
        Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if available < total_length {
        return Ok(None);
    }
    // `read_frame` returns the protobuf payload without its varint length prefix.
    // Keep the non-blocking probe equivalent so callers can inspect protobuf field
    // tags without accidentally treating the frame prefix as message content.
    Ok(Some(frame[prefix_length..].to_vec()))
}

fn peek(channel: &UnixStream, bytes: &mut [u8]) -> std::io::Result<usize> {
    let length = unsafe {
        libc::recv(
            channel.as_raw_fd(),
            bytes.as_mut_ptr().cast(),
            bytes.len(),
            libc::MSG_PEEK | libc::MSG_DONTWAIT,
        )
    };
    if length < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        usize::try_from(length).map_err(|_| std::io::Error::other("invalid socket frame length"))
    }
}

fn decode_length(bytes: &[u8]) -> Result<Option<(usize, usize)>, String> {
    let mut value = 0_u64;
    for (index, byte) in bytes.iter().copied().enumerate() {
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            let payload_length = usize::try_from(value)
                .map_err(|_| "managed runtime Vault route is invalid".to_owned())?;
            if payload_length == 0 || payload_length > MAX_FRAME_BYTES {
                return Err("managed runtime Vault route is invalid".to_owned());
            }
            return Ok(Some((index + 1, payload_length)));
        }
    }
    if bytes.len() == 5 {
        return Err("managed runtime Vault route is invalid".to_owned());
    }
    Ok(None)
}

fn valid_event_credential_request(value: &ManagedRuntimeEventCredentialRequestV1) -> bool {
    value.request_id.len() == 16
        && value.request_id.iter().any(|byte| *byte != 0)
        && value.credential_revision > 0
        && (1..=600).contains(&value.ttl_seconds)
        && value.recipient_public_key_x25519.len() == 32
}

fn valid_provider_credential_request(value: &ManagedRuntimeProviderCredentialRequestV1) -> bool {
    value.request_id.len() == 16
        && value.request_id.iter().any(|byte| *byte != 0)
        && !value.purpose_id.trim().is_empty()
        && value.purpose_id.len() <= 128
        && value.credential_revision > 0
        && (1..=600).contains(&value.ttl_seconds)
        && (1..=5).contains(&value.secret_class)
        && (1..=6).contains(&value.action)
        && value.recipient_public_key_x25519.len() == 32
        && valid_configuration_instance_id(&value.configuration_instance_id)
}

fn valid_owner_derived_key_request(value: &ManagedRuntimeOwnerDerivedKeyRequestV1) -> bool {
    value.request_id.len() == 16
        && value.request_id.iter().any(|byte| *byte != 0)
        && !value.purpose_id.trim().is_empty()
        && value.purpose_id.len() <= 128
        && value.purpose_id.is_ascii()
        && valid_configuration_instance_id(&value.capability_id)
        && value.key_schema_revision != 0
        && (1..=600).contains(&value.ttl_seconds)
        && value.recipient_public_key_x25519.len() == 32
}

fn valid_configuration_instance_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

#[cfg(test)]
mod blob_session_error_code_tests {
    use super::{
        BLOB_CUSTODY_DELEGATION_FIELD_TAG, BLOB_CUSTODY_RELEASE_FIELD_TAG,
        ManagedRuntimeInboundRequestV1, VAULT_ROUTE_FIELD_TAG, blob_custody_delegation_request,
        blob_custody_release_request, blob_session_error_code, decode_typed_request,
    };
    use makosh_runtime_protocol::v1::{
        BlobCustodyReleaseReasonV1, ManagedRuntimeBlobCustodyDelegationRequestV1,
        ManagedRuntimeBlobCustodyReleaseRequestV1, ManagedRuntimeControlRequestV1,
        ManagedRuntimeReadyRequestV1, ManagedRuntimeVaultRouteRequestV1,
        managed_runtime_control_request_v1::Operation,
    };
    use prost::Message;

    #[test]
    fn exposes_only_the_retryable_blob_availability_code() {
        assert_eq!(
            blob_session_error_code("managed runtime Blob custody transfer is unavailable"),
            "managed_blob_session_unavailable",
        );
        assert_eq!(
            blob_session_error_code("managed runtime Blob custody transfer is denied"),
            "managed_blob_session_denied",
        );
    }

    #[test]
    fn vault_route_uses_its_typed_control_oneof_tag() {
        let frame = ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::RouteVaultCiphertext(
                ManagedRuntimeVaultRouteRequestV1 { route: None },
            )),
        }
        .encode_to_vec();

        assert_eq!(frame.first(), Some(&VAULT_ROUTE_FIELD_TAG));
        assert_ne!(frame.first(), Some(&0x0a));
    }

    #[test]
    fn decodes_typed_ready_without_socket_or_field_tag_peeking() {
        let request = ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1 {
                registration_id: "communications".to_owned(),
                runtime_generation: 1,
                grant_epoch: 1,
            })),
        };

        assert!(matches!(
            decode_typed_request(request),
            Ok(ManagedRuntimeInboundRequestV1::Ready(_))
        ));
    }

    #[test]
    fn custody_release_uses_its_exact_typed_oneof_tag() {
        let frame = ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::ReleaseBlobCustody(
                ManagedRuntimeBlobCustodyReleaseRequestV1 {
                    operation_id: vec![1; 16],
                    capability_id: "attachment_security.blob.v1".to_owned(),
                    reference_id: vec![2; 16],
                    declared_size: 3,
                    receipt_sha256: vec![4; 32],
                    custody_source_proof: vec![5; 64],
                    reason: BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
                        as i32,
                },
            )),
        }
        .encode_to_vec();

        assert_eq!(frame.first(), Some(&BLOB_CUSTODY_RELEASE_FIELD_TAG));
        assert!(
            blob_custody_release_request(&frame)
                .expect("decode")
                .is_some()
        );
    }

    #[test]
    fn custody_delegation_uses_its_exact_typed_oneof_tag() {
        let frame = ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::DelegateBlobCustody(
                ManagedRuntimeBlobCustodyDelegationRequestV1 {
                    request_id: vec![1; 16],
                    capability_id: "attachment_security.blob.v1".to_owned(),
                    current_reference_id: vec![2; 16],
                    predecessor_custody_source_proof: vec![3; 64],
                    predecessor_evidence_id: vec![4; 16],
                    predecessor_evidence_envelope_sha256: vec![5; 32],
                    target_owner_id: "attachment_archive_inspection".to_owned(),
                    target_module_id: "makosh-attachment-archive-inspection-runtime".to_owned(),
                    target_capability_id: "attachment_archive_inspection.blob.v1".to_owned(),
                    target_request_contract: None,
                },
            )),
        }
        .encode_to_vec();

        assert_eq!(frame.first(), Some(&BLOB_CUSTODY_DELEGATION_FIELD_TAG));
        assert!(
            blob_custody_delegation_request(&frame)
                .expect("decode")
                .is_some()
        );
    }
}

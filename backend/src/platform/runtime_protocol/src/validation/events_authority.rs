//! Structural validation for the managed Events authority configuration and status.

use crate::v1::{
    ApplyEventsAccountJwtUpdateRequestV1, ApplyEventsAccountJwtUpdateResponseV1,
    DurableEnvelopeKindV1, EventHubConsumerTopologyV1, EventHubStreamTopologyV1,
    EventsAuthorityRuntimeConfigurationV1, EventsAuthorityRuntimeControlRequestV1,
    EventsAuthorityRuntimeControlResponseV1, EventsAuthorityRuntimeStateV1,
    EventsAuthorityRuntimeStatusV1, EventsRuntimeConsumerGrantV1,
    EventsRuntimeCredentialDeliveryV1, IssueEventsRuntimeCredentialRequestV1,
    ReconcileEventsTopologyRequestV1, ReconcileEventsTopologyResponseV1,
    events_authority_runtime_control_request_v1::Operation as RequestOperation,
    events_authority_runtime_control_response_v1::Result as ResponseResult,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventsAuthorityRuntimeValidationErrorV1 {
    InvalidConfiguration,
    InvalidRequest,
    InvalidResponse,
    InvalidStatus,
}

pub fn validate_events_authority_runtime_configuration(
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    (is_account_key(&configuration.account_public_key)
        && valid_id(&configuration.vault_instance_id)
        && configuration.vault_runtime_generation > 0
        && configuration.vault_hpke_public_key_x25519.len() == 32
        && configuration.signer_credential_revision > 0
        && valid_nats_endpoint(&configuration.nats_endpoint)
        && valid_nats_identity(&configuration.nats_username)
        && configuration.event_hub_credential_revision > 0)
        .then_some(())
        .ok_or(EventsAuthorityRuntimeValidationErrorV1::InvalidConfiguration)
}

pub fn validate_events_authority_runtime_control_request(
    request: &EventsAuthorityRuntimeControlRequestV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    match &request.operation {
        Some(RequestOperation::GetStatus(_)) => Ok(()),
        Some(RequestOperation::IssueRuntimeCredential(value)) => validate_credential_request(value),
        Some(RequestOperation::ReconcileTopology(value)) => validate_topology_request(value),
        Some(RequestOperation::ApplyAccountJwtUpdate(value)) => validate_account_jwt_update(value),
        None => Err(EventsAuthorityRuntimeValidationErrorV1::InvalidRequest),
    }
}

pub fn validate_events_authority_runtime_control_response(
    response: &EventsAuthorityRuntimeControlResponseV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    match (&response.result, response.error_code.is_empty()) {
        (Some(ResponseResult::Status(status)), true) => {
            validate_events_authority_runtime_status(status)
        }
        (Some(ResponseResult::CredentialDelivery(delivery)), true) => {
            validate_credential_delivery(delivery)
        }
        (Some(ResponseResult::TopologyReconciled(value)), true) => {
            validate_topology_response(value)
        }
        (Some(ResponseResult::AccountJwtUpdated(value)), true) => {
            validate_account_jwt_update_response(value)
        }
        (None, false) if valid_blocker_code(&response.error_code) => Ok(()),
        _ => Err(EventsAuthorityRuntimeValidationErrorV1::InvalidResponse),
    }
}

pub fn validate_account_jwt_update(
    request: &ApplyEventsAccountJwtUpdateRequestV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    (request.resolver_credential_revision > 0
        && !request.signed_account_jwt.is_empty()
        && request.signed_account_jwt.len() <= 16 * 1024
        && request.signed_account_jwt.is_ascii()
        && request
            .signed_account_jwt
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_')))
    .then_some(())
    .ok_or(EventsAuthorityRuntimeValidationErrorV1::InvalidRequest)
}

fn validate_account_jwt_update_response(
    response: &ApplyEventsAccountJwtUpdateResponseV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    (response.resolver_credential_revision > 0)
        .then_some(())
        .ok_or(EventsAuthorityRuntimeValidationErrorV1::InvalidResponse)
}

pub fn validate_topology_request(
    request: &ReconcileEventsTopologyRequestV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    (request.topology_revision > 0
        && valid_streams(&request.streams)
        && valid_consumers(&request.consumers, &request.streams))
    .then_some(())
    .ok_or(EventsAuthorityRuntimeValidationErrorV1::InvalidRequest)
}

fn validate_topology_response(
    response: &ReconcileEventsTopologyResponseV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    (response.topology_revision > 0 && response.stream_count > 0 && response.stream_count <= 5)
        .then_some(())
        .ok_or(EventsAuthorityRuntimeValidationErrorV1::InvalidResponse)
}

pub fn validate_credential_request(
    request: &IssueEventsRuntimeCredentialRequestV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    (valid_id(&request.logical_owner_id)
        && valid_id(&request.registration_id)
        && valid_id(&request.runtime_instance_id)
        && request.runtime_generation > 0
        && request.grant_epoch > 0
        && request.credential_revision > 0
        && request.ttl_seconds > 0
        && request.ttl_seconds <= 600
        && valid_request_id(&request.request_id)
        && request.recipient_public_key_x25519.len() == 32
        && valid_subjects(&request.publish_subjects)
        && request.subscribe_subjects.is_empty()
        && valid_consumer_grants(&request.consumer_grants)
        && (!request.publish_subjects.is_empty() || !request.consumer_grants.is_empty()))
    .then_some(())
    .ok_or(EventsAuthorityRuntimeValidationErrorV1::InvalidRequest)
}

fn valid_consumer_grants(values: &[EventsRuntimeConsumerGrantV1]) -> bool {
    values.len() <= 64
        && values.iter().all(|value| {
            valid_id(&value.durable_name)
                && valid_subjects(std::slice::from_ref(&value.filter_subject))
        })
        && values
            .iter()
            .map(|value| (&value.durable_name, &value.filter_subject))
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == values.len()
}

pub fn validate_credential_delivery(
    delivery: &EventsRuntimeCredentialDeliveryV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    (delivery.encapped_key.len() == 32
        && !delivery.ciphertext.is_empty()
        && delivery.ciphertext.len() <= 16 * 1024
        && delivery.tag.len() == 16)
        .then_some(())
        .ok_or(EventsAuthorityRuntimeValidationErrorV1::InvalidResponse)
}

pub fn validate_events_authority_runtime_status(
    status: &EventsAuthorityRuntimeStatusV1,
) -> Result<(), EventsAuthorityRuntimeValidationErrorV1> {
    let state = EventsAuthorityRuntimeStateV1::try_from(status.state)
        .map_err(|_| EventsAuthorityRuntimeValidationErrorV1::InvalidStatus)?;
    if status.runtime_generation == 0
        || status.grant_epoch == 0
        || status.vault_runtime_generation == 0
        || status.signer_credential_revision == 0
    {
        return Err(EventsAuthorityRuntimeValidationErrorV1::InvalidStatus);
    }
    match state {
        EventsAuthorityRuntimeStateV1::Ready if status.blocker_code.is_empty() => Ok(()),
        EventsAuthorityRuntimeStateV1::Blocked if valid_blocker_code(&status.blocker_code) => {
            Ok(())
        }
        _ => Err(EventsAuthorityRuntimeValidationErrorV1::InvalidStatus),
    }
}

fn is_account_key(value: &str) -> bool {
    value.len() == 56
        && value.starts_with('A')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_blocker_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_request_id(value: &[u8]) -> bool {
    value.len() == 16 && value.iter().any(|byte| *byte != 0)
}

fn valid_nats_endpoint(value: &str) -> bool {
    value.starts_with("nats://") && value.len() <= 256 && !value.contains(['@', '?', '#', ' '])
}

fn valid_nats_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_streams(values: &[EventHubStreamTopologyV1]) -> bool {
    !values.is_empty()
        && values.len() <= 5
        && values
            .windows(2)
            .all(|pair| pair[0].envelope_kind < pair[1].envelope_kind)
        && values.iter().all(|value| {
            valid_envelope_kind(value.envelope_kind)
                && (1..=1_073_741_824).contains(&value.max_bytes)
                && (1..=7_776_000_000).contains(&value.max_age_millis)
                && value.replicas == 1
        })
}

fn valid_consumers(
    consumers: &[EventHubConsumerTopologyV1],
    streams: &[EventHubStreamTopologyV1],
) -> bool {
    consumers.len() <= 512
        && consumers.windows(2).all(|pair| {
            (pair[0].envelope_kind, pair[0].durable_name.as_str())
                < (pair[1].envelope_kind, pair[1].durable_name.as_str())
        })
        && consumers.iter().all(|value| {
            streams
                .iter()
                .any(|stream| stream.envelope_kind == value.envelope_kind)
                && valid_envelope_kind(value.envelope_kind)
                && valid_id(&value.durable_name)
                && valid_consumer_subject(value.envelope_kind, &value.filter_subject)
                && (1..=4_096).contains(&value.max_ack_pending)
                && (1..=32).contains(&value.max_deliver)
                && (1..=600_000).contains(&value.ack_wait_millis)
        })
}

fn valid_envelope_kind(value: i32) -> bool {
    DurableEnvelopeKindV1::try_from(value)
        .ok()
        .is_some_and(|kind| kind != DurableEnvelopeKindV1::Unspecified)
}

fn valid_consumer_subject(envelope_kind: i32, subject: &str) -> bool {
    let token = match DurableEnvelopeKindV1::try_from(envelope_kind).ok() {
        Some(DurableEnvelopeKindV1::Command) => "command",
        Some(DurableEnvelopeKindV1::Event) => "event",
        Some(DurableEnvelopeKindV1::Observation) => "observation",
        Some(DurableEnvelopeKindV1::Result) => "result",
        Some(DurableEnvelopeKindV1::Ack) => "ack",
        _ => return false,
    };
    subject.starts_with(&format!("makosh.{token}.v1."))
        && subject.len() <= 256
        && !subject.contains(['>', '*'])
        && subject.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_subjects(values: &[String]) -> bool {
    values.len() <= 64
        && values.windows(2).all(|pair| pair[0] < pair[1])
        && values.iter().all(|value| valid_subject(value))
}

fn valid_subject(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

//! Vault-verified authority startup and sanitized status control.

use std::future::Future;
use std::os::unix::net::UnixStream;
use std::task::{Context, Poll, Waker};
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_events_authority::NatsJwtCredentialAuthorityV1;
use makosh_events_jetstream::{
    DurableSubjectV1, NatsAccountSignerFenceV1, NatsJwtConsumerGrantV1, NatsJwtPermissionSetV1,
    NatsRuntimeCredentialDeliveryBindingV1, NatsRuntimeCredentialFenceV1,
    NatsRuntimeCredentialRecipientPublicKeyV1,
};
use makosh_runtime_protocol::{
    v1::{
        EventsAuthorityRuntimeConfigurationV1, EventsAuthorityRuntimeControlRequestV1,
        EventsAuthorityRuntimeControlResponseV1, EventsAuthorityRuntimeStateV1,
        EventsAuthorityRuntimeStatusV1, EventsRuntimeCredentialDeliveryV1,
        IssueEventsRuntimeCredentialRequestV1, ManagedRuntimeControlRequestV1,
        ManagedRuntimeReadyRequestV1, events_authority_runtime_control_request_v1::Operation,
        events_authority_runtime_control_response_v1::Result as ResponseResult,
        managed_runtime_control_request_v1::Operation as ManagedOperation,
    },
    validation::events_authority::{
        validate_events_authority_runtime_configuration,
        validate_events_authority_runtime_control_request,
    },
};
use prost::Message;

use super::{
    account_jwt_update,
    framing::{read_frame, write_frame},
    handshake::{EventsAuthorityRuntimeIdentityV1, authenticate},
    topology, vault_context,
    vault_route::InheritedVaultRoutePortV1,
};

pub(crate) const AUTHORITY_RUNTIME_INSTANCE_ID: &str = "events_authority_runtime";

pub fn serve_inherited(
    inherited_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: EventsAuthorityRuntimeConfigurationV1,
) -> Result<(), String> {
    validate_events_authority_runtime_configuration(&configuration)
        .map_err(|_| "Events authority configuration is invalid".to_owned())?;
    let (channel, identity) =
        authenticate(inherited_channel, descriptor_bytes, settings_schema_bytes)?;
    serve_verified(channel, identity, configuration)
}

pub fn serve_inherited_on_channel(
    inherited_channel: UnixStream,
    descriptor_bytes: Vec<u8>,
    settings_schema_bytes: Vec<u8>,
    configuration: EventsAuthorityRuntimeConfigurationV1,
) -> Result<(), String> {
    serve_inherited(
        inherited_channel,
        descriptor_bytes,
        settings_schema_bytes,
        configuration,
    )
}

fn serve_verified(
    mut channel: UnixStream,
    identity: EventsAuthorityRuntimeIdentityV1,
    configuration: EventsAuthorityRuntimeConfigurationV1,
) -> Result<(), String> {
    let mut authority = create_verified_authority(&channel, &identity, &configuration)?;
    announce_ready(&mut channel, &identity)?;
    serve_control(channel, identity, configuration, &mut authority)
}

fn create_verified_authority(
    channel: &UnixStream,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> Result<NatsJwtCredentialAuthorityV1<InheritedVaultRoutePortV1>, String> {
    let context = vault_context::from_configuration(configuration)
        .map_err(|_| "Events authority Vault context is invalid".to_owned())?;
    let route = InheritedVaultRoutePortV1::new(
        channel
            .try_clone()
            .map_err(|_| "Events authority inherited channel is unavailable".to_owned())?,
    );
    let mut authority =
        NatsJwtCredentialAuthorityV1::new(configuration.account_public_key.clone(), route, context)
            .map_err(|_| "Events authority signer is unavailable".to_owned())?;
    let fence = NatsAccountSignerFenceV1::new(
        identity.registration_id().to_owned(),
        AUTHORITY_RUNTIME_INSTANCE_ID.to_owned(),
        identity.runtime_generation(),
        identity.grant_epoch(),
        configuration.signer_credential_revision,
    )
    .map_err(|_| "Events authority signer is unavailable".to_owned())?;
    let verification = complete_immediately(authority.verify_account_signer(&fence))
        .map_err(|_| "Events authority signer is unavailable".to_owned())?;
    verification.map_err(|_| "Events authority signer is unavailable".to_owned())?;
    Ok(authority)
}

fn announce_ready(
    channel: &mut UnixStream,
    identity: &EventsAuthorityRuntimeIdentityV1,
) -> Result<(), String> {
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Ready(ManagedRuntimeReadyRequestV1 {
            registration_id: identity.registration_id().to_owned(),
            runtime_generation: identity.runtime_generation(),
            grant_epoch: identity.grant_epoch(),
        })),
    };
    write_frame(channel, &request.encode_to_vec())
}

fn serve_control(
    mut channel: UnixStream,
    identity: EventsAuthorityRuntimeIdentityV1,
    configuration: EventsAuthorityRuntimeConfigurationV1,
    authority: &mut NatsJwtCredentialAuthorityV1<InheritedVaultRoutePortV1>,
) -> Result<(), String> {
    channel
        .set_read_timeout(None)
        .and_then(|_| channel.set_write_timeout(None))
        .map_err(|_| "Events authority inherited channel is unavailable".to_owned())?;
    loop {
        let response = read_frame(&mut channel)
            .and_then(|bytes| {
                EventsAuthorityRuntimeControlRequestV1::decode(bytes.as_slice())
                    .map_err(|_| "Events authority inherited frame is invalid".to_owned())
            })
            .map(|request| response_for(request, &channel, &identity, &configuration, authority))
            .unwrap_or_else(|_| error_response("invalid_request"));
        write_frame(&mut channel, &response.encode_to_vec())?;
    }
}

fn response_for(
    request: EventsAuthorityRuntimeControlRequestV1,
    channel: &UnixStream,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
    authority: &mut NatsJwtCredentialAuthorityV1<InheritedVaultRoutePortV1>,
) -> EventsAuthorityRuntimeControlResponseV1 {
    if validate_events_authority_runtime_control_request(&request).is_err() {
        return error_response("operation_not_available");
    }
    match request.operation {
        Some(Operation::GetStatus(_)) => EventsAuthorityRuntimeControlResponseV1 {
            result: Some(ResponseResult::Status(EventsAuthorityRuntimeStatusV1 {
                state: EventsAuthorityRuntimeStateV1::Ready as i32,
                runtime_generation: identity.runtime_generation(),
                grant_epoch: identity.grant_epoch(),
                vault_runtime_generation: configuration.vault_runtime_generation,
                signer_credential_revision: configuration.signer_credential_revision,
                blocker_code: String::new(),
            })),
            error_code: String::new(),
        },
        Some(Operation::IssueRuntimeCredential(request)) => {
            issue_credential_response(request, identity, configuration, authority)
        }
        Some(Operation::ReconcileTopology(request)) => {
            reconcile_topology_response(channel, request, identity, configuration)
        }
        Some(Operation::ApplyAccountJwtUpdate(request)) => {
            account_jwt_update::response(channel, request, identity, configuration)
        }
        None => error_response("operation_not_available"),
    }
}

fn reconcile_topology_response(
    channel: &UnixStream,
    request: makosh_runtime_protocol::v1::ReconcileEventsTopologyRequestV1,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> EventsAuthorityRuntimeControlResponseV1 {
    let stream_count = u32::try_from(request.streams.len()).unwrap_or_default();
    let consumer_count = u32::try_from(request.consumers.len()).unwrap_or_default();
    let revision = request.topology_revision;
    match topology::reconcile(channel, identity, configuration, request) {
        Ok(()) => EventsAuthorityRuntimeControlResponseV1 {
            result: Some(ResponseResult::TopologyReconciled(
                makosh_runtime_protocol::v1::ReconcileEventsTopologyResponseV1 {
                    topology_revision: revision,
                    stream_count,
                    consumer_count,
                },
            )),
            error_code: String::new(),
        },
        Err(()) => error_response("topology_reconcile_denied"),
    }
}

fn issue_credential_response(
    request: IssueEventsRuntimeCredentialRequestV1,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
    authority: &mut NatsJwtCredentialAuthorityV1<InheritedVaultRoutePortV1>,
) -> EventsAuthorityRuntimeControlResponseV1 {
    let result = issue_credential(request, identity, configuration, authority);
    match result {
        Ok(delivery) => EventsAuthorityRuntimeControlResponseV1 {
            result: Some(ResponseResult::CredentialDelivery(delivery)),
            error_code: String::new(),
        },
        Err(()) => error_response("credential_issue_denied"),
    }
}

fn issue_credential(
    request: IssueEventsRuntimeCredentialRequestV1,
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
    authority: &mut NatsJwtCredentialAuthorityV1<InheritedVaultRoutePortV1>,
) -> Result<EventsRuntimeCredentialDeliveryV1, ()> {
    let runtime_fence = runtime_fence(&request)?;
    let signer_fence = signer_fence(identity, configuration)?;
    let binding = delivery_binding(&request, &runtime_fence)?;
    let permissions = permission_set(&request)?;
    let now_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ())?
        .as_secs();
    let delivery = complete_immediately(authority.issue_sealed_runtime_credential(
        &signer_fence,
        &runtime_fence,
        permissions,
        now_unix_seconds,
        u64::from(request.ttl_seconds),
        &binding,
    ))
    .map_err(|_| ())?
    .map_err(|_| ())?;
    Ok(EventsRuntimeCredentialDeliveryV1 {
        encapped_key: delivery.encapped_key().to_vec(),
        ciphertext: delivery.ciphertext().to_vec(),
        tag: delivery.tag().to_vec(),
    })
}

fn runtime_fence(
    request: &IssueEventsRuntimeCredentialRequestV1,
) -> Result<NatsRuntimeCredentialFenceV1, ()> {
    NatsRuntimeCredentialFenceV1::new(
        request.logical_owner_id.clone(),
        request.registration_id.clone(),
        request.runtime_instance_id.clone(),
        request.runtime_generation,
        request.grant_epoch,
        request.credential_revision,
    )
    .map_err(|_| ())
}

fn signer_fence(
    identity: &EventsAuthorityRuntimeIdentityV1,
    configuration: &EventsAuthorityRuntimeConfigurationV1,
) -> Result<NatsAccountSignerFenceV1, ()> {
    NatsAccountSignerFenceV1::new(
        identity.registration_id().to_owned(),
        AUTHORITY_RUNTIME_INSTANCE_ID.to_owned(),
        identity.runtime_generation(),
        identity.grant_epoch(),
        configuration.signer_credential_revision,
    )
    .map_err(|_| ())
}

fn delivery_binding(
    request: &IssueEventsRuntimeCredentialRequestV1,
    runtime_fence: &NatsRuntimeCredentialFenceV1,
) -> Result<NatsRuntimeCredentialDeliveryBindingV1, ()> {
    let request_id = request.request_id.as_slice().try_into().map_err(|_| ())?;
    let recipient_key = request
        .recipient_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| ())?;
    let recipient_key =
        NatsRuntimeCredentialRecipientPublicKeyV1::from_bytes(recipient_key).map_err(|_| ())?;
    makosh_events_jetstream::bind_runtime_credential_delivery(
        runtime_fence,
        request_id,
        recipient_key,
    )
    .map_err(|_| ())
}

fn permission_set(
    request: &IssueEventsRuntimeCredentialRequestV1,
) -> Result<NatsJwtPermissionSetV1, ()> {
    let publish = parse_subjects(&request.publish_subjects)?;
    let consumers = request
        .consumer_grants
        .iter()
        .map(|grant| {
            NatsJwtConsumerGrantV1::new(
                DurableSubjectV1::parse(&grant.filter_subject).map_err(|_| ())?,
                grant.durable_name.clone(),
            )
            .map_err(|_| ())
        })
        .collect::<Result<Vec<_>, _>>()?;
    NatsJwtPermissionSetV1::new(publish, consumers).map_err(|_| ())
}

fn parse_subjects(values: &[String]) -> Result<Vec<DurableSubjectV1>, ()> {
    values
        .iter()
        .map(|value| DurableSubjectV1::parse(value).map_err(|_| ()))
        .collect()
}

fn error_response(error_code: &str) -> EventsAuthorityRuntimeControlResponseV1 {
    EventsAuthorityRuntimeControlResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}

fn complete_immediately<T>(future: impl Future<Output = T>) -> Result<T, ()> {
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let mut future = std::pin::pin!(future);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => Ok(value),
        Poll::Pending => Err(()),
    }
}

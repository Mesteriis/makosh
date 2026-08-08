//! Inherited-process conformance for the Events account-signing authority.

mod live;
mod resolver_update_live;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use makosh_events_authority_runtime_control::serve_inherited_on_channel;
use makosh_events_jetstream::NatsRuntimeCredentialFenceV1;
use makosh_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingV1, NatsRuntimeCredentialDeliveryV1,
    NatsRuntimeCredentialRecipientPublicKeyV1, NatsRuntimeCredentialRecipientV1,
};
use makosh_runtime_protocol::v1::{
    ApplyEventsAccountJwtUpdateRequestV1, DescribeManagedRuntimeResponseV1,
    EventsAuthorityRuntimeConfigurationV1, EventsAuthorityRuntimeControlRequestV1,
    EventsAuthorityRuntimeControlResponseV1, GetEventsAuthorityRuntimeStatusRequestV1,
    IssueEventsRuntimeCredentialRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
    ManagedRuntimeVaultRouteRequestV1, ManagedRuntimeVaultRouteResponseV1,
    ReconcileEventsTopologyRequestV1,
    events_authority_runtime_control_request_v1::Operation as AuthorityOperation,
    events_authority_runtime_control_response_v1::Result as AuthorityResult,
    managed_runtime_control_request_v1::Operation as ManagedOperation,
    managed_runtime_control_response_v1::Result as ManagedResult,
};
use prost::Message;

use super::support::encrypted_response;

#[test]
fn authority_runtime_requires_vault_verified_signer_before_ready_and_serves_only_status() {
    let (mut kernel, worker, account_seed) = start_runtime();
    complete_descriptor_and_signer_bootstrap(&mut kernel, &account_seed);
    assert_ready(&mut kernel);
    assert_status(&mut kernel);
    drop(kernel);
    assert!(worker.join().expect("authority worker").is_err());
}

#[test]
fn authority_runtime_returns_only_recipient_encrypted_runtime_credentials() {
    let (mut kernel, worker, account_seed) = start_runtime();
    complete_descriptor_and_signer_bootstrap(&mut kernel, &account_seed);
    assert_ready(&mut kernel);
    let recipient = NatsRuntimeCredentialRecipientV1::generate();
    let request = credential_request(recipient.public_key().as_bytes());
    write_frame(
        &mut kernel,
        &EventsAuthorityRuntimeControlRequestV1 {
            operation: Some(AuthorityOperation::IssueRuntimeCredential(request.clone())),
        }
        .encode_to_vec(),
    );
    answer_signer_resolution(&mut kernel, &account_seed);
    let delivery = read_credential_delivery(&mut kernel);
    assert_not_plaintext(&delivery, &account_seed);
    let credential = recipient
        .open(&credential_binding(&request), &delivery)
        .expect("recipient opens credential");
    assert!(credential.user_public_key().starts_with('U'));
    assert!(credential.expires_at_unix_seconds() > 0);
    drop(kernel);
    assert!(worker.join().expect("authority worker").is_err());
}

#[test]
fn authority_runtime_rejects_incomplete_topology_before_contacting_vault() {
    let (mut kernel, worker, account_seed) = start_runtime();
    complete_descriptor_and_signer_bootstrap(&mut kernel, &account_seed);
    assert_ready(&mut kernel);
    write_frame(
        &mut kernel,
        &EventsAuthorityRuntimeControlRequestV1 {
            operation: Some(AuthorityOperation::ReconcileTopology(
                ReconcileEventsTopologyRequestV1 {
                    topology_revision: 1,
                    streams: Vec::new(),
                    consumers: Vec::new(),
                },
            )),
        }
        .encode_to_vec(),
    );
    let response =
        EventsAuthorityRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
            .expect("topology rejection response");
    assert_eq!(response.error_code, "operation_not_available");
    assert!(response.result.is_none());
    drop(kernel);
    assert!(worker.join().expect("authority worker").is_err());
}

#[test]
fn authority_runtime_rejects_an_invalid_account_jwt_before_contacting_vault() {
    let (mut kernel, worker, account_seed) = start_runtime();
    complete_descriptor_and_signer_bootstrap(&mut kernel, &account_seed);
    assert_ready(&mut kernel);
    write_frame(
        &mut kernel,
        &EventsAuthorityRuntimeControlRequestV1 {
            operation: Some(AuthorityOperation::ApplyAccountJwtUpdate(
                ApplyEventsAccountJwtUpdateRequestV1 {
                    resolver_credential_revision: 1,
                    signed_account_jwt: "not-a-jwt".to_owned(),
                },
            )),
        }
        .encode_to_vec(),
    );
    let response =
        EventsAuthorityRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
            .expect("Account JWT rejection response");
    assert_eq!(response.error_code, "account_jwt_update_denied");
    assert!(response.result.is_none());
    drop(kernel);
    assert!(worker.join().expect("authority worker").is_err());
}

fn start_runtime() -> (
    UnixStream,
    std::thread::JoinHandle<Result<(), String>>,
    String,
) {
    let account = nats_jwt::KeyPair::new_account();
    let account_seed = account.seed().expect("account seed");
    start_runtime_with_account(account.public_key(), account_seed)
}

pub(super) fn start_runtime_with_account(
    account_public_key: String,
    account_seed: String,
) -> (
    UnixStream,
    std::thread::JoinHandle<Result<(), String>>,
    String,
) {
    let configuration = EventsAuthorityRuntimeConfigurationV1 {
        account_public_key,
        vault_instance_id: "vault_instance".to_owned(),
        vault_runtime_generation: 7,
        vault_hpke_public_key_x25519: vault_public_key().to_vec(),
        signer_credential_revision: 2,
        nats_endpoint: nats_endpoint(),
        nats_username: nats_username(),
        event_hub_credential_revision: 3,
    };
    let (server, kernel) = UnixStream::pair().expect("inherited channel");
    let worker = std::thread::spawn(move || {
        serve_inherited_on_channel(server, vec![1], vec![2], configuration)
    });
    (kernel, worker, account_seed)
}

fn nats_endpoint() -> String {
    std::env::var("MAKOSH_NATS_TEST_ENDPOINT")
        .unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned())
}

fn nats_username() -> String {
    std::env::var("MAKOSH_NATS_EVENT_HUB_USERNAME").unwrap_or_else(|_| "event_hub".to_owned())
}

fn complete_descriptor_and_signer_bootstrap(kernel: &mut UnixStream, account_seed: &str) {
    let describe = ManagedRuntimeControlRequestV1::decode(read_frame(kernel).as_slice())
        .expect("describe request");
    assert!(matches!(
        describe.operation,
        Some(ManagedOperation::Describe(_))
    ));
    let response = ManagedRuntimeControlResponseV1 {
        result: Some(ManagedResult::Describe(DescribeManagedRuntimeResponseV1 {
            registration_id: "events_authority".to_owned(),
            runtime_generation: 4,
            grant_epoch: 9,
        })),
        error_code: String::new(),
    };
    write_frame(kernel, &response.encode_to_vec());
    answer_signer_resolution(kernel, account_seed);
}

fn credential_request(
    recipient_public_key_x25519: &[u8; 32],
) -> IssueEventsRuntimeCredentialRequestV1 {
    IssueEventsRuntimeCredentialRequestV1 {
        logical_owner_id: "contacts".to_owned(),
        registration_id: "contacts_module".to_owned(),
        runtime_instance_id: "runtime_1".to_owned(),
        runtime_generation: 3,
        grant_epoch: 8,
        credential_revision: 2,
        publish_subjects: vec!["makosh.event.v1.contacts.changed.v1".to_owned()],
        subscribe_subjects: Vec::new(),
        ttl_seconds: 60,
        request_id: vec![7; 16],
        recipient_public_key_x25519: recipient_public_key_x25519.to_vec(),
        consumer_grants: Vec::new(),
    }
}

fn credential_binding(
    request: &IssueEventsRuntimeCredentialRequestV1,
) -> NatsRuntimeCredentialDeliveryBindingV1 {
    let fence = NatsRuntimeCredentialFenceV1::new(
        request.logical_owner_id.clone(),
        request.registration_id.clone(),
        request.runtime_instance_id.clone(),
        request.runtime_generation,
        request.grant_epoch,
        request.credential_revision,
    )
    .expect("runtime fence");
    let recipient = request
        .recipient_public_key_x25519
        .as_slice()
        .try_into()
        .expect("recipient key");
    let recipient = NatsRuntimeCredentialRecipientPublicKeyV1::from_bytes(recipient)
        .expect("recipient key validates");
    makosh_events_jetstream::bind_runtime_credential_delivery(&fence, [7; 16], recipient)
        .expect("delivery binding")
}

fn read_credential_delivery(kernel: &mut UnixStream) -> NatsRuntimeCredentialDeliveryV1 {
    let response = EventsAuthorityRuntimeControlResponseV1::decode(read_frame(kernel).as_slice())
        .expect("credential response");
    let Some(AuthorityResult::CredentialDelivery(value)) = response.result else {
        panic!("credential response is missing");
    };
    assert!(response.error_code.is_empty());
    NatsRuntimeCredentialDeliveryV1::from_parts(value.encapped_key, value.ciphertext, value.tag)
        .expect("opaque credential delivery")
}

fn assert_not_plaintext(delivery: &NatsRuntimeCredentialDeliveryV1, account_seed: &str) {
    assert!(
        !delivery
            .ciphertext()
            .windows(account_seed.len())
            .any(|value| value == account_seed.as_bytes())
    );
}

fn answer_signer_resolution(kernel: &mut UnixStream, account_seed: &str) {
    answer_vault_request(kernel, vec![b'a'; 32]);
    answer_vault_request(kernel, account_seed.as_bytes().to_vec());
}

fn answer_vault_request(kernel: &mut UnixStream, payload: Vec<u8>) {
    let request = ManagedRuntimeControlRequestV1::decode(read_frame(kernel).as_slice())
        .expect("managed control request");
    let Some(ManagedOperation::RouteVaultCiphertext(request)) = request.operation else {
        panic!("Vault route request");
    };
    respond_to_vault_route(kernel, request, payload);
}

fn assert_ready(kernel: &mut UnixStream) {
    let ready = ManagedRuntimeControlRequestV1::decode(read_frame(kernel).as_slice())
        .expect("ready request");
    assert!(
        matches!(ready.operation, Some(ManagedOperation::Ready(ManagedRuntimeReadyRequestV1 {
        registration_id, runtime_generation: 4, grant_epoch: 9,
    })) if registration_id == "events_authority")
    );
}

fn assert_status(kernel: &mut UnixStream) {
    let request = EventsAuthorityRuntimeControlRequestV1 {
        operation: Some(AuthorityOperation::GetStatus(
            GetEventsAuthorityRuntimeStatusRequestV1 {},
        )),
    };
    write_frame(kernel, &request.encode_to_vec());
    let status = EventsAuthorityRuntimeControlResponseV1::decode(read_frame(kernel).as_slice())
        .expect("status response");
    assert!(matches!(status.result, Some(AuthorityResult::Status(value))
        if status.error_code.is_empty() && value.runtime_generation == 4 && value.grant_epoch == 9
            && value.vault_runtime_generation == 7 && value.signer_credential_revision == 2));
}

fn respond_to_vault_route(
    kernel: &mut UnixStream,
    request: ManagedRuntimeVaultRouteRequestV1,
    payload: Vec<u8>,
) {
    let route = request.route.expect("ciphertext route");
    let response = encrypted_response(&route, payload).expect("encrypted Vault response");
    write_frame(
        kernel,
        &ManagedRuntimeControlResponseV1 {
            result: Some(ManagedResult::VaultRoute(
                ManagedRuntimeVaultRouteResponseV1 {
                    response: Some(response),
                    error_code: String::new(),
                },
            )),
            error_code: String::new(),
        }
        .encode_to_vec(),
    );
}

fn vault_public_key() -> [u8; 32] {
    makosh_vault_protocol::VaultResponseRecipientV1::generate()
        .public_key()
        .as_bytes()
        .to_owned()
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let length = usize::try_from(read_varint(stream)).expect("frame length");
    let mut bytes = vec![0; length];
    stream.read_exact(&mut bytes).expect("read frame");
    bytes
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) {
    let mut length = u32::try_from(bytes.len()).expect("frame length");
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream.write_all(&prefix).expect("write length");
    stream.write_all(bytes).expect("write frame");
    stream.flush().expect("flush frame");
}

fn read_varint(stream: &mut UnixStream) -> u64 {
    let mut value = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).expect("read length byte");
        value |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            return value;
        }
    }
    panic!("frame length is invalid");
}

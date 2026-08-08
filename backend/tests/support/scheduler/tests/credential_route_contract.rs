//! Scheduler's inherited control FD is the sole NATS credential path.

use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    thread,
};

use makosh_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingInputV1, NatsRuntimeCredentialDeliveryBindingV1,
    NatsRuntimeCredentialRecipientPublicKeyV1, RuntimeNatsJwtCredentialV1,
};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeEventCredentialDeliveryV1, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_scheduler_jetstream::{SchedulerNatsCredentialErrorV1, request_runtime_credential};
use nats_jwt::KeyPair;
use prost::Message;

const OWNER: &str = "platform";
const REGISTRATION: &str = "scheduler_registration";
const RUNTIME: &str = "scheduler_runtime";
const GENERATION: u64 = 4;
const EPOCH: u64 = 7;
const REVISION: u64 = 2;

#[test]
fn scheduler_opens_only_the_credential_bound_to_its_runtime_request() {
    let expected = credential();
    let expected_public_key = expected.user_public_key().to_owned();
    let (mut scheduler, relay) = UnixStream::pair().expect("private runtime channel");
    let relay = thread::spawn(move || respond_with_credential(relay, expected));

    let actual = request_runtime_credential(
        &mut scheduler,
        OWNER,
        REGISTRATION,
        RUNTIME,
        GENERATION,
        EPOCH,
        REVISION,
    )
    .expect("Kernel relay delivery");

    assert_eq!(actual.user_public_key(), expected_public_key);
    relay.join().expect("relay thread");
}

#[test]
fn scheduler_rejects_an_error_response_without_exposing_credential_material() {
    let (mut scheduler, mut relay) = UnixStream::pair().expect("private runtime channel");
    let relay = thread::spawn(move || {
        let _request = read_request(&mut relay);
        write_response(
            &mut relay,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "credential_not_authorized".to_owned(),
            },
        );
    });

    let result = request_runtime_credential(
        &mut scheduler,
        OWNER,
        REGISTRATION,
        RUNTIME,
        GENERATION,
        EPOCH,
        REVISION,
    );

    assert!(matches!(
        result,
        Err(SchedulerNatsCredentialErrorV1::Rejected)
    ));
    relay.join().expect("relay thread");
}

fn respond_with_credential(mut relay: UnixStream, credential: RuntimeNatsJwtCredentialV1) {
    let request = read_request(&mut relay);
    let Some(Operation::IssueEventCredential(request)) = request.operation else {
        panic!("expected an event credential request");
    };
    assert_eq!(request.credential_revision, REVISION);
    assert_eq!(request.ttl_seconds, 300);
    let request_id = request.request_id.try_into().expect("request ID length");
    let recipient = NatsRuntimeCredentialRecipientPublicKeyV1::from_bytes(
        request
            .recipient_public_key_x25519
            .try_into()
            .expect("recipient key length"),
    )
    .expect("recipient public key");
    let binding =
        NatsRuntimeCredentialDeliveryBindingV1::new(NatsRuntimeCredentialDeliveryBindingInputV1 {
            logical_owner_id: OWNER.to_owned(),
            registration_id: REGISTRATION.to_owned(),
            runtime_instance_id: RUNTIME.to_owned(),
            runtime_generation: GENERATION,
            grant_epoch: EPOCH,
            credential_revision: REVISION,
            request_id,
            recipient_public_key: recipient,
        })
        .expect("delivery binding");
    let delivery = credential.seal_for(&binding).expect("encrypted delivery");
    write_response(
        &mut relay,
        ManagedRuntimeControlResponseV1 {
            result: Some(ResponseResult::EventCredentialDelivery(
                ManagedRuntimeEventCredentialDeliveryV1 {
                    encapped_key: delivery.encapped_key().to_vec(),
                    ciphertext: delivery.ciphertext().to_vec(),
                    tag: delivery.tag().to_vec(),
                    consumer_bindings: Vec::new(),
                    publish_subjects: Vec::new(),
                },
            )),
            error_code: String::new(),
        },
    );
}

fn read_request(channel: &mut UnixStream) -> ManagedRuntimeControlRequestV1 {
    let length = read_length(channel);
    let mut bytes = vec![0_u8; length];
    channel.read_exact(&mut bytes).expect("runtime request");
    ManagedRuntimeControlRequestV1::decode(bytes.as_slice()).expect("request protobuf")
}

fn write_response(channel: &mut UnixStream, response: ManagedRuntimeControlResponseV1) {
    let bytes = response.encode_to_vec();
    let mut frame = Vec::with_capacity(bytes.len() + 5);
    encode_length(bytes.len(), &mut frame);
    frame.extend_from_slice(&bytes);
    channel.write_all(&frame).expect("runtime response");
}

fn read_length(channel: &mut UnixStream) -> usize {
    let mut value = 0_usize;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        channel.read_exact(&mut byte).expect("frame length");
        value |= usize::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return value;
        }
    }
    panic!("malformed frame length");
}

fn encode_length(mut value: usize, output: &mut Vec<u8>) {
    while value >= 0x80 {
        output.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    output.push(value as u8);
}

fn credential() -> RuntimeNatsJwtCredentialV1 {
    let key = KeyPair::new_user();
    RuntimeNatsJwtCredentialV1::new(
        "test-jwt".to_owned(),
        key.seed().expect("user seed"),
        key.public_key(),
        u64::MAX,
    )
    .expect("runtime credential")
}

//! Managed Authority JWT issuance against the resolver-backed NATS contour.

use std::{sync::Arc, time::Duration};

use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, NatsRuntimeCredentialFenceV1, RuntimeNatsIdentity,
    RuntimePublishPermitV1, StreamKindV1,
};
use makosh_events_protocol::v1::{
    ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
    SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
};
use makosh_events_protocol::{
    NatsRuntimeCredentialDeliveryBindingV1, NatsRuntimeCredentialDeliveryV1,
    NatsRuntimeCredentialRecipientV1,
};
use makosh_runtime_protocol::v1::ManagedRuntimeEventCredentialRequestV1;
use prost::Message;
use prost_types::Timestamp;

use super::fixture::LiveAuthorityFixture;
use crate::platform::events::credential::handler::EventCredentialHandlerV1;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeEventCredentialHandler, ManagedRuntimeExpectation,
};

const ENDPOINT: &str = "MAKOSH_NATS_JWT_TEST_ENDPOINT";
const ACCOUNT_PUBLIC_KEY: &str = "MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE";
const ACCOUNT_SIGNER_SEED: &str = "MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE";
const EVENT_HUB_CREDS: &str = "MAKOSH_NATS_JWT_EVENT_HUB_CREDS_FILE";
const REQUEST_ID: [u8; 16] = [1; 16];

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires resolver-backed Docker JetStream and managed runtime binaries"]
async fn kernel_managed_authority_delivers_a_broker_accepted_jwt() {
    require_live_environment();
    let endpoint = required(ENDPOINT);
    create_event_stream(&endpoint).await;
    let fixture = LiveAuthorityFixture::start(
        &read_fixture(ACCOUNT_PUBLIC_KEY),
        read_fixture(ACCOUNT_SIGNER_SEED).as_bytes(),
        &endpoint,
        "event_hub",
        None,
    );
    let grant_epoch = fixture
        .store()
        .module_registration("notes-publisher")
        .expect("read notes publisher")
        .expect("notes publisher registration")
        .grant_epoch();
    let recipient = NatsRuntimeCredentialRecipientV1::generate();
    let delivery = issue_credential(&fixture, &recipient, grant_epoch);
    let credential = recipient
        .open(&delivery_binding(&recipient, grant_epoch), &delivery)
        .expect("open Authority ciphertext only in the runtime");
    let runtime = JetStreamClient::connect_runtime_with_jwt(
        &endpoint,
        RuntimeNatsIdentity::new("notes_runtime", 2, grant_epoch).expect("runtime identity"),
        credential,
    )
    .await
    .expect("resolver accepts Authority-issued runtime JWT");

    assert_runtime_permissions(&runtime, grant_epoch).await;
}

fn issue_credential(
    fixture: &LiveAuthorityFixture,
    recipient: &NatsRuntimeCredentialRecipientV1,
    grant_epoch: u64,
) -> NatsRuntimeCredentialDeliveryV1 {
    let handler = EventCredentialHandlerV1::new(
        Arc::clone(fixture.store()),
        "events_authority".to_owned(),
        fixture.supervisor().relay_port(),
    )
    .expect("Events authority credential handler");
    let response = handler
        .issue_event_credential(
            &expectation(grant_epoch),
            ManagedRuntimeEventCredentialRequestV1 {
                request_id: REQUEST_ID.to_vec(),
                credential_revision: 1,
                ttl_seconds: 300,
                recipient_public_key_x25519: recipient.public_key().as_bytes().to_vec(),
            },
        )
        .expect("Kernel routes request to managed Authority");
    NatsRuntimeCredentialDeliveryV1::from_parts(
        response.encapped_key,
        response.ciphertext,
        response.tag,
    )
    .expect("opaque Authority delivery")
}

fn expectation(grant_epoch: u64) -> ManagedRuntimeExpectation {
    ManagedRuntimeExpectation::new(
        "notes-publisher",
        "notes_runtime",
        "notes-publisher-module",
        2,
        grant_epoch,
        [1; 32],
        None,
    )
}

fn delivery_binding(
    recipient: &NatsRuntimeCredentialRecipientV1,
    grant_epoch: u64,
) -> NatsRuntimeCredentialDeliveryBindingV1 {
    let fence = NatsRuntimeCredentialFenceV1::new(
        "test_owner",
        "notes-publisher",
        "notes_runtime",
        2,
        grant_epoch,
        1,
    )
    .expect("runtime credential fence");
    makosh_events_jetstream::bind_runtime_credential_delivery(
        &fence,
        REQUEST_ID,
        recipient.public_key().clone(),
    )
    .expect("delivery binding")
}

async fn assert_runtime_permissions(
    runtime: &makosh_events_jetstream::RuntimeJetStreamConnection,
    grant_epoch: u64,
) {
    let allowed = subject("changed");
    let allowed_permit = permit(grant_epoch, allowed.clone());
    runtime
        .publish_exact(&allowed_permit, &event_envelope("changed").encode_to_vec())
        .await
        .expect("broker accepts the Authority-granted subject");

    let forbidden = subject("other");
    let forbidden_permit = permit(grant_epoch, forbidden);
    assert!(
        runtime
            .publish_exact(&forbidden_permit, &event_envelope("other").encode_to_vec())
            .await
            .is_err(),
        "broker must enforce the JWT subject grant after the local permit passes"
    );
}

fn permit(grant_epoch: u64, subject: DurableSubjectV1) -> RuntimePublishPermitV1 {
    RuntimePublishPermitV1::new(
        "notes-publisher",
        "notes_runtime",
        2,
        grant_epoch,
        vec![subject],
    )
    .expect("runtime publish permit")
}

fn subject(contract: &str) -> DurableSubjectV1 {
    DurableSubjectV1::new(StreamKindV1::Event, "notes", contract, 1).expect("event subject")
}

async fn create_event_stream(endpoint: &str) {
    let options = async_nats::ConnectOptions::with_credentials_file(required(EVENT_HUB_CREDS))
        .await
        .expect("read Event Hub credentials");
    let client = options.connect(endpoint).await.expect("connect Event Hub");
    async_nats::jetstream::new(client)
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "MAKOSH_EVENT_V1".to_owned(),
            subjects: vec!["makosh.event.v1.>".to_owned()],
            max_bytes: 1_048_576,
            max_age: Duration::from_secs(3600),
            num_replicas: 1,
            storage: async_nats::jetstream::stream::StorageType::File,
            ..Default::default()
        })
        .await
        .expect("create Event Hub stream");
}

fn event_envelope(contract: &str) -> DurableEnvelopeV1 {
    let message_id = vec![7; 16];
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.clone(),
        contract: Some(ContractRefV1 {
            owner: "notes".to_owned(),
            name: contract.to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![9; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: "notes_runtime".to_owned(),
            runtime_instance_id: vec![3; 16],
            runtime_generation: 2,
        }),
        recorded_at: Some(Timestamp {
            seconds: 1,
            nanos: 0,
        }),
        partition_key: b"notes_partition".to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: message_id,
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: b"notes_runtime".to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: b"notes_runtime".to_vec(),
            epoch: 1,
        }),
        semantics: Some(Semantics::Event(EventMetadataV1 {
            occurred_at: Some(Timestamp {
                seconds: 1,
                nanos: 0,
            }),
        })),
        payload: vec![1, 2, 3],
    }
}

fn require_live_environment() {
    assert_eq!(
        std::env::var("MAKOSH_EVENTS_MANAGED_JWT_TEST").as_deref(),
        Ok("1")
    );
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(required(name))
        .expect("read JWT resolver fixture")
        .trim()
        .to_owned()
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set for JWT conformance"))
}

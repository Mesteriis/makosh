use std::time::Duration;

use futures_util::StreamExt;
use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, DurableSubjectV1, EventHubTopologyPlanV1, JetStreamClient,
    NatsPasswordCredentialV1, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, StreamBudgetV1, StreamKindV1, StreamSpecV1,
};
use makosh_events_protocol::v1::{
    ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, FenceKindV1,
    SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
};
use prost::Message;
use prost_types::Timestamp;

const ENDPOINT: &str = "MAKOSH_NATS_TEST_ENDPOINT";
const HUB_USER: &str = "MAKOSH_NATS_EVENT_HUB_USERNAME";
const HUB_PASSWORD: &str = "MAKOSH_NATS_EVENT_HUB_PASSWORD";
const RUNTIME_USER: &str = "MAKOSH_NATS_RUNTIME_USERNAME";
const RUNTIME_PASSWORD: &str = "MAKOSH_NATS_RUNTIME_PASSWORD";

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the authenticated Docker JetStream contour"]
async fn authenticated_runtime_publishes_exact_bytes_only_to_its_catalog_subject() {
    let endpoint = required(ENDPOINT);
    configure_event_hub(&endpoint).await;
    let expected = publish_from_runtime(&endpoint).await;
    assert_exact_runtime_delivery(&endpoint, &expected).await;
}

pub(super) async fn configure_event_hub(endpoint: &str) {
    let hub = JetStreamClient::connect_event_hub(
        endpoint,
        NatsPasswordCredentialV1::new(required(HUB_USER), required(HUB_PASSWORD))
            .expect("hub credential"),
    )
    .await
    .expect("connect event hub");
    let stream = StreamSpecV1::new(
        StreamKindV1::Event,
        StreamBudgetV1::new(1_048_576, Duration::from_secs(3600), 1).expect("stream budget"),
    );
    let consumer_budget =
        ConsumerBudgetV1::new(16, 3, Duration::from_secs(2)).expect("consumer budget");
    let consumer = ConsumerSpecV1::new(
        StreamKindV1::Event,
        "notes_projection",
        "makosh.event.v1.notes.changed.v1",
        consumer_budget,
    )
    .expect("consumer spec");
    let topology = EventHubTopologyPlanV1::new(vec![stream], vec![consumer])
        .expect("declared Event Hub topology");
    hub.reconcile(&topology)
        .await
        .expect("reconcile Event Hub topology");
}

async fn publish_from_runtime(endpoint: &str) -> Vec<u8> {
    let runtime = JetStreamClient::connect_runtime(
        endpoint,
        RuntimeNatsIdentity::new("notes_runtime", 1, 1).expect("runtime identity"),
        NatsPasswordCredentialV1::new(required(RUNTIME_USER), required(RUNTIME_PASSWORD))
            .expect("runtime credential"),
    )
    .await
    .expect("connect runtime");
    let permit = RuntimePublishPermitV1::new(
        "registration_notes",
        "notes_runtime",
        1,
        1,
        vec![
            DurableSubjectV1::new(StreamKindV1::Event, "notes", "changed", 1)
                .expect("exact publish subject"),
        ],
    )
    .expect("runtime publish permit");
    let expected = event_envelope("changed").encode_to_vec();
    let first = runtime
        .publish_exact(&permit, &expected)
        .await
        .expect("publish first copy");
    let second = runtime
        .publish_exact(&permit, &expected)
        .await
        .expect("publish duplicate copy");
    assert_eq!(first.stream(), "MAKOSH_EVENT_V1");
    assert!(!first.duplicate());
    assert!(second.duplicate());
    assert!(
        runtime
            .publish_exact(&permit, &event_envelope("other").encode_to_vec())
            .await
            .is_err()
    );
    expected
}

pub(super) async fn assert_exact_runtime_delivery(endpoint: &str, expected: &[u8]) {
    let runtime = JetStreamClient::connect_runtime(
        endpoint,
        RuntimeNatsIdentity::new("notes_runtime", 1, 1).expect("runtime identity"),
        NatsPasswordCredentialV1::new(required(RUNTIME_USER), required(RUNTIME_PASSWORD))
            .expect("runtime credential"),
    )
    .await
    .expect("connect runtime consumer");
    let permit = RuntimeSubscribePermitV1::new(
        "registration_notes",
        "notes_runtime",
        1,
        1,
        ConsumerSpecV1::new(
            StreamKindV1::Event,
            "notes_projection",
            "makosh.event.v1.notes.changed.v1",
            ConsumerBudgetV1::new(16, 3, Duration::from_secs(2)).expect("consumer budget"),
        )
        .expect("consumer spec"),
    )
    .expect("runtime subscribe permit");
    let consumer = runtime
        .open_pull_consumer(&permit)
        .await
        .expect("open exact runtime consumer");
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await
        .expect("fetch messages");
    let message = tokio::time::timeout(Duration::from_secs(2), messages.next())
        .await
        .expect("delivery timeout")
        .expect("delivery missing")
        .expect("delivery error");
    assert_eq!(message.payload.as_ref(), expected);
    message
        .ack()
        .await
        .expect("acknowledge after durable observation");
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set for JetStream conformance"))
}

pub(super) fn event_envelope(contract: &str) -> DurableEnvelopeV1 {
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
            runtime_generation: 1,
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

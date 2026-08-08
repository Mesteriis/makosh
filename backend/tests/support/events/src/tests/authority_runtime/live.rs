//! Live broker conformance for authority-owned Event Hub reconciliation.

use async_nats::jetstream::consumer::PullConsumer;
use makosh_runtime_protocol::v1::{
    DurableEnvelopeKindV1, EventHubConsumerTopologyV1, EventHubStreamTopologyV1,
    EventsAuthorityRuntimeControlRequestV1, EventsAuthorityRuntimeControlResponseV1,
    ReconcileEventsTopologyRequestV1,
    events_authority_runtime_control_request_v1::Operation as AuthorityOperation,
    events_authority_runtime_control_response_v1::Result as AuthorityResult,
};
use prost::Message;

use super::{
    answer_vault_request, assert_ready, complete_descriptor_and_signer_bootstrap, read_frame,
    start_runtime, write_frame,
};

const LEASE_ID: &str = "0123456789abcdef0123456789abcdef";

#[test]
#[ignore = "requires the authenticated Docker JetStream contour"]
fn authority_runtime_reconciles_broker_topology_through_vault_credential() {
    let endpoint = required("MAKOSH_NATS_TEST_ENDPOINT");
    let (mut kernel, worker, account_seed) = start_runtime();
    complete_descriptor_and_signer_bootstrap(&mut kernel, &account_seed);
    assert_ready(&mut kernel);
    write_frame(&mut kernel, &reconcile_request().encode_to_vec());
    answer_vault_request(&mut kernel, LEASE_ID.as_bytes().to_vec());
    answer_vault_request(
        &mut kernel,
        required("MAKOSH_NATS_EVENT_HUB_PASSWORD").into_bytes(),
    );
    assert_reconciled(&mut kernel);
    assert_broker_topology(&endpoint);
    drop(kernel);
    assert!(worker.join().expect("authority worker").is_err());
}

fn reconcile_request() -> EventsAuthorityRuntimeControlRequestV1 {
    EventsAuthorityRuntimeControlRequestV1 {
        operation: Some(AuthorityOperation::ReconcileTopology(
            ReconcileEventsTopologyRequestV1 {
                topology_revision: 1,
                streams: vec![EventHubStreamTopologyV1 {
                    envelope_kind: DurableEnvelopeKindV1::Event as i32,
                    max_bytes: 1_048_576,
                    max_age_millis: 3_600_000,
                    replicas: 1,
                }],
                consumers: vec![EventHubConsumerTopologyV1 {
                    envelope_kind: DurableEnvelopeKindV1::Event as i32,
                    durable_name: "notes_projection".to_owned(),
                    filter_subject: "makosh.event.v1.notes.changed.v1".to_owned(),
                    max_ack_pending: 16,
                    max_deliver: 3,
                    ack_wait_millis: 2_000,
                }],
            },
        )),
    }
}

fn assert_reconciled(kernel: &mut std::os::unix::net::UnixStream) {
    let response = EventsAuthorityRuntimeControlResponseV1::decode(read_frame(kernel).as_slice())
        .expect("topology reconciliation response");
    assert!(matches!(
        response.result,
        Some(AuthorityResult::TopologyReconciled(value))
            if response.error_code.is_empty()
                && value.topology_revision == 1
                && value.stream_count == 1
                && value.consumer_count == 1
    ));
}

fn assert_broker_topology(endpoint: &str) {
    tokio::runtime::Runtime::new()
        .expect("test runtime")
        .block_on(async {
            let client = async_nats::ConnectOptions::new()
                .user_and_password(
                    required("MAKOSH_NATS_EVENT_HUB_USERNAME"),
                    required("MAKOSH_NATS_EVENT_HUB_PASSWORD"),
                )
                .connect(endpoint)
                .await
                .expect("connect Event Hub verifier");
            let context = async_nats::jetstream::new(client);
            let stream = context
                .get_stream("MAKOSH_EVENT_V1")
                .await
                .expect("Event stream is reconciled");
            assert_eq!(stream.cached_info().config.max_bytes, 1_048_576);
            let consumer: PullConsumer = stream
                .get_consumer("notes_projection")
                .await
                .expect("Event consumer is reconciled");
            assert_eq!(
                consumer.cached_info().config.filter_subject,
                "makosh.event.v1.notes.changed.v1"
            );
        });
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set for JetStream conformance"))
}

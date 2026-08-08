use super::*;

pub(super) fn configure_scheduler_jetstream(store: &SqliteControlStore) {
    let configuration = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let contracts = event_catalog::resolve_contracts(store).expect("resolve Event Hub contracts");
    let plan = event_topology::plan(&contracts, &configuration).expect("plan Event Hub topology");
    let endpoint = configuration.nats_endpoint().to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(&endpoint)
                    .await
                    .expect("connect JetStream"),
            );
            for stream in plan.streams() {
                let (name, subject) = scheduler_stream_details(stream.kind());
                context
                    .create_stream(async_nats::jetstream::stream::Config {
                        name: name.to_owned(),
                        subjects: vec![subject.to_owned()],
                        ..Default::default()
                    })
                    .await
                    .expect("create Scheduler Event stream");
            }
            for consumer in plan.consumers() {
                let stream_name = if consumer.subject().as_str().starts_with("makosh.ack.") {
                    "MAKOSH_ACK_V1"
                } else {
                    "MAKOSH_RESULT_V1"
                };
                context
                    .create_consumer_on_stream(
                        async_nats::jetstream::consumer::pull::Config {
                            durable_name: Some(consumer.durable_name().to_owned()),
                            filter_subject: consumer.subject().as_str(),
                            ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                            ack_wait: Duration::from_millis(
                                consumer.delivery_policy().ack_wait_millis().into(),
                            ),
                            max_deliver: i64::from(consumer.delivery_policy().max_deliver()),
                            max_ack_pending: i64::from(consumer.max_in_flight()),
                            ..Default::default()
                        },
                        stream_name,
                    )
                    .await
                    .expect("create Scheduler receipt consumer");
            }
        });
}

fn scheduler_stream_details(
    kind: event_topology::subject::EventStreamKindV1,
) -> (&'static str, &'static str) {
    match kind {
        event_topology::subject::EventStreamKindV1::Command => {
            ("MAKOSH_COMMAND_V1", "makosh.command.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Ack => ("MAKOSH_ACK_V1", "makosh.ack.v1.>"),
        event_topology::subject::EventStreamKindV1::Result => {
            ("MAKOSH_RESULT_V1", "makosh.result.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Event => {
            ("MAKOSH_EVENT_V1", "makosh.event.v1.>")
        }
        event_topology::subject::EventStreamKindV1::Observation => {
            ("MAKOSH_OBSERVATION_V1", "makosh.observation.v1.>")
        }
    }
}

pub(super) fn configure_scheduler_delivery_observer(store: &SqliteControlStore) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("JetStream observer runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(&endpoint)
                    .await
                    .expect("connect Scheduler observer"),
            );
            context
                .create_consumer_on_stream(
                    async_nats::jetstream::consumer::pull::Config {
                        durable_name: Some("scheduler_recovery_delivery".to_owned()),
                        filter_subject: "makosh.command.v1.platform.maintenance.v1".to_owned(),
                        deliver_policy: async_nats::jetstream::consumer::DeliverPolicy::New,
                        ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                        max_ack_pending: 1,
                        ..Default::default()
                    },
                    "MAKOSH_COMMAND_V1",
                )
                .await
                .expect("create Scheduler recovery delivery observer");
        });
}

pub(super) fn recovered_scheduler_delivery(store: &SqliteControlStore) -> DurableEnvelopeV1 {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("JetStream delivery runtime")
        .block_on(async move {
            let context = async_nats::jetstream::new(
                async_nats::connect(&endpoint)
                    .await
                    .expect("connect Scheduler delivery observer"),
            );
            let stream = context
                .get_stream("MAKOSH_COMMAND_V1")
                .await
                .expect("read Scheduler command stream");
            let consumer: async_nats::jetstream::consumer::PullConsumer = stream
                .get_consumer("scheduler_recovery_delivery")
                .await
                .expect("read Scheduler recovery delivery observer");
            let mut messages = consumer
                .fetch()
                .max_messages(1)
                .expires(Duration::from_secs(25))
                .messages()
                .await
                .expect("fetch recovered Scheduler delivery");
            let message = tokio::time::timeout(Duration::from_secs(25), messages.next())
                .await
                .expect("recovered Scheduler delivery timeout")
                .expect("recovered Scheduler delivery missing")
                .expect("recovered Scheduler delivery error");
            let envelope = DurableEnvelopeV1::decode(message.payload.as_ref())
                .expect("recovered Scheduler delivery envelope");
            message
                .ack()
                .await
                .expect("acknowledge recovered Scheduler delivery");
            envelope
        })
}

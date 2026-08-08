//! Managed authority reconciliation against the password-authenticated contour.

use async_nats::jetstream::consumer::PullConsumer;
use nats_jwt::KeyPair;

use super::fixture::LiveAuthorityFixture;
use crate::platform::events::reconciliation;

#[test]
#[ignore = "requires disposable authenticated Docker JetStream and managed runtime binaries"]
fn kernel_managed_authority_reconciles_jetstream_through_live_vault() {
    require_live_environment();
    let signer = KeyPair::new_account();
    let fixture = LiveAuthorityFixture::start(
        &signer.public_key(),
        signer.seed().expect("account signing seed").as_bytes(),
        &required("MAKOSH_NATS_TEST_ENDPOINT"),
        &required("MAKOSH_NATS_EVENT_HUB_USERNAME"),
        Some(&required("MAKOSH_NATS_EVENT_HUB_PASSWORD")),
    );

    assert_eq!(
        reconciliation::apply(fixture.store(), &fixture.supervisor().relay_port())
            .expect("reconcile Event Hub"),
        (1, 1, 1)
    );
    assert_broker_topology(&fixture.consumer_name());
}

fn require_live_environment() {
    assert_eq!(
        std::env::var("MAKOSH_EVENTS_MANAGED_AUTHORITY_TEST").as_deref(),
        Ok("1")
    );
}

fn assert_broker_topology(consumer_name: &str) {
    tokio::runtime::Runtime::new()
        .expect("test runtime")
        .block_on(async {
            let client = async_nats::ConnectOptions::new()
                .user_and_password(
                    required("MAKOSH_NATS_EVENT_HUB_USERNAME"),
                    required("MAKOSH_NATS_EVENT_HUB_PASSWORD"),
                )
                .connect(required("MAKOSH_NATS_TEST_ENDPOINT"))
                .await
                .expect("connect Event Hub verifier");
            let stream = async_nats::jetstream::new(client)
                .get_stream("MAKOSH_EVENT_V1")
                .await
                .expect("Event stream is reconciled");
            let consumer: PullConsumer = stream
                .get_consumer(consumer_name)
                .await
                .expect("Event consumer is reconciled");
            assert_eq!(stream.cached_info().config.max_bytes, 1_048_576);
            assert_eq!(
                consumer.cached_info().config.filter_subject,
                "makosh.event.v1.notes.changed.v1"
            );
        });
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set for JetStream conformance"))
}

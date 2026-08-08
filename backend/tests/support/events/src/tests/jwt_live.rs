use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, DurableSubjectV1, JetStreamClient, NatsJwtConsumerGrantV1,
    NatsJwtPermissionSetV1, NatsRuntimeCredentialFenceV1, RuntimeNatsIdentity,
    RuntimeNatsJwtIssuerV1, RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
};
use prost::Message;

use super::jetstream_live::event_envelope;

const ENDPOINT: &str = "MAKOSH_NATS_JWT_TEST_ENDPOINT";
const ACCOUNT_PUBLIC_KEY_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE";
const ACCOUNT_SIGNING_SEED_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE";
const EVENT_HUB_CREDS_FILE: &str = "MAKOSH_NATS_JWT_EVENT_HUB_CREDS_FILE";

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the JWT resolver Docker JetStream contour"]
async fn jwt_runtime_credential_is_verified_and_broker_fences_subjects() {
    let endpoint = required(ENDPOINT);
    create_event_stream(&endpoint).await;
    let account_public_key = read_fixture(ACCOUNT_PUBLIC_KEY_FILE);
    let account_signing_seed = read_fixture(ACCOUNT_SIGNING_SEED_FILE);
    let issuer = RuntimeNatsJwtIssuerV1::from_account_signing_seed(
        &account_public_key,
        account_signing_seed,
    )
    .expect("test account signing authority");
    let runtime = connect_runtime(&endpoint, &issuer).await;

    let allowed_subject = subject("changed");
    let allowed_permit = permit(allowed_subject.clone());
    runtime
        .publish_exact(&allowed_permit, &event_envelope("changed").encode_to_vec())
        .await
        .expect("broker accepts the exact JWT-granted subject");
    assert_exact_runtime_delivery(&runtime).await;

    let forbidden_permit = permit(subject("other"));
    assert!(
        runtime
            .publish_exact(&forbidden_permit, &event_envelope("other").encode_to_vec())
            .await
            .is_err()
    );
    assert_unknown_signer_is_rejected(&endpoint, &account_public_key).await;
}

async fn create_event_stream(endpoint: &str) {
    let credentials = required(EVENT_HUB_CREDS_FILE);
    let options = async_nats::ConnectOptions::with_credentials_file(credentials)
        .await
        .expect("read Event Hub credentials");
    let client = options.connect(endpoint).await.expect("connect Event Hub");
    let context = async_nats::jetstream::new(client);
    let stream = context
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "MAKOSH_EVENT_V1".to_owned(),
            subjects: vec!["makosh.event.v1.>".to_owned()],
            max_bytes: 1_048_576,
            max_age: Duration::from_secs(3600),
            num_replicas: 1,
            storage: async_nats::jetstream::stream::StorageType::File,
            ..async_nats::jetstream::stream::Config::default()
        })
        .await
        .expect("create Event Hub stream");
    stream
        .get_or_create_consumer(
            "notes_projection",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("notes_projection".to_owned()),
                filter_subject: subject("changed").as_str(),
                ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                ack_wait: Duration::from_secs(2),
                max_deliver: 3,
                max_ack_pending: 16,
                max_batch: 16,
                max_expires: Duration::from_secs(2),
                ..async_nats::jetstream::consumer::pull::Config::default()
            },
        )
        .await
        .expect("create exact Event Hub consumer");
}

async fn assert_exact_runtime_delivery(
    runtime: &makosh_events_jetstream::RuntimeJetStreamConnection,
) {
    let permit =
        RuntimeSubscribePermitV1::new("registration_notes", "notes_runtime", 2, 5, consumer_spec())
            .expect("runtime subscribe permit");
    let consumer = runtime
        .open_pull_consumer(&permit)
        .await
        .expect("JWT opens only its exact durable consumer");
    let mut messages = consumer
        .fetch()
        .max_messages(1)
        .messages()
        .await
        .expect("fetch exact delivery");
    let message = tokio::time::timeout(Duration::from_secs(2), messages.next())
        .await
        .expect("delivery timeout")
        .expect("delivery missing")
        .expect("delivery error");
    assert_eq!(
        message.payload.as_ref(),
        event_envelope("changed").encode_to_vec()
    );
    message.ack().await.expect("ack exact delivery");
}

async fn connect_runtime(
    endpoint: &str,
    issuer: &RuntimeNatsJwtIssuerV1,
) -> makosh_events_jetstream::RuntimeJetStreamConnection {
    let credential = issuer
        .issue_runtime_credential(&fence(), permissions(), unix_seconds(), 300)
        .expect("issue runtime JWT");
    JetStreamClient::connect_runtime_with_jwt(
        endpoint,
        RuntimeNatsIdentity::new("notes_runtime", 2, 5).expect("runtime identity"),
        credential,
    )
    .await
    .expect("connect runtime through NATS JWT verification")
}

async fn assert_unknown_signer_is_rejected(endpoint: &str, account_public_key: &str) {
    let unknown_signer = nats_jwt::KeyPair::new_account();
    let issuer = RuntimeNatsJwtIssuerV1::from_account_signing_seed(
        account_public_key.to_owned(),
        unknown_signer.seed().expect("unknown signing seed"),
    )
    .expect("isolated unknown issuer");
    let credential = issuer
        .issue_runtime_credential(&fence(), permissions(), unix_seconds(), 300)
        .expect("issue unknown runtime JWT");
    assert!(
        JetStreamClient::connect_runtime_with_jwt(
            endpoint,
            RuntimeNatsIdentity::new("notes_runtime", 2, 5).expect("runtime identity"),
            credential,
        )
        .await
        .is_err()
    );
}

fn permissions() -> NatsJwtPermissionSetV1 {
    let consumer = NatsJwtConsumerGrantV1::new(subject("changed"), "notes_projection")
        .expect("consumer grant");
    NatsJwtPermissionSetV1::new(vec![subject("changed")], vec![consumer])
        .expect("exact runtime permission")
}

fn consumer_spec() -> ConsumerSpecV1 {
    ConsumerSpecV1::new(
        StreamKindV1::Event,
        "notes_projection",
        subject("changed").as_str(),
        ConsumerBudgetV1::new(16, 3, Duration::from_secs(2)).expect("consumer budget"),
    )
    .expect("consumer spec")
}

fn permit(subject: DurableSubjectV1) -> RuntimePublishPermitV1 {
    RuntimePublishPermitV1::new("registration_notes", "notes_runtime", 2, 5, vec![subject])
        .expect("runtime permit")
}

fn subject(contract: &str) -> DurableSubjectV1 {
    DurableSubjectV1::new(StreamKindV1::Event, "notes", contract, 1).expect("subject")
}

fn fence() -> NatsRuntimeCredentialFenceV1 {
    NatsRuntimeCredentialFenceV1::new("notes", "registration_notes", "notes_runtime", 2, 5, 3)
        .expect("runtime fence")
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(required(name))
        .expect("read JWT test fixture")
        .trim()
        .to_owned()
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_secs()
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set for JWT conformance"))
}

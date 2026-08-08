use makosh_events_jetstream::{
    NatsAccountJwtUpdateV1, NatsResolverAccountJwtPublisherV1, NatsResolverSystemCredentialsV1,
};

const ENDPOINT: &str = "MAKOSH_NATS_JWT_TEST_ENDPOINT";
const ACCOUNT_PUBLIC_KEY_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE";
const ACCOUNT_JWT_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_JWT_FILE";
const SYSTEM_CREDENTIALS_FILE: &str = "MAKOSH_NATS_JWT_RESOLVER_UPDATE_CREDS_FILE";

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the JWT resolver Docker contour"]
async fn system_account_publishes_a_bound_account_jwt() {
    let update = NatsAccountJwtUpdateV1::new(
        read_fixture(ACCOUNT_PUBLIC_KEY_FILE),
        read_fixture(ACCOUNT_JWT_FILE),
    )
    .expect("bound Account JWT");
    let credentials = NatsResolverSystemCredentialsV1::new(read_fixture(SYSTEM_CREDENTIALS_FILE))
        .expect("System Account credentials");

    NatsResolverAccountJwtPublisherV1::publish(&required(ENDPOINT), &credentials, &update)
        .await
        .expect("NATS resolver accepts the current Account JWT");
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(required(name))
        .expect("read resolver fixture")
        .trim()
        .to_owned()
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set for JWT conformance"))
}

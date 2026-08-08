use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, NatsJwtPermissionSetV1, NatsRuntimeCredentialFenceV1,
    RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimeNatsJwtIssuerV1, StreamKindV1,
};

const ENDPOINT: &str = "MAKOSH_NATS_JWT_TEST_ENDPOINT";
const ACCOUNT_PUBLIC_KEY_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE";
const ACCOUNT_SIGNING_SEED_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE";
const READY_FILE: &str = "MAKOSH_NATS_JWT_REVOCATION_READY_FILE";
const PROCEED_FILE: &str = "MAKOSH_NATS_JWT_REVOCATION_PROCEED_FILE";

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the JWT resolver Docker revocation contour"]
async fn resolver_claim_update_disconnects_an_active_runtime() {
    let (runtime, public_key) = connect_runtime().await;
    std::fs::write(required(READY_FILE), public_key).expect("notify conformance harness");
    wait_for_file(&required(PROCEED_FILE)).await;
    assert!(wait_for_disconnect(&runtime).await);
}

async fn connect_runtime() -> (RuntimeJetStreamConnection, String) {
    let issuer = RuntimeNatsJwtIssuerV1::from_account_signing_seed(
        read_fixture(ACCOUNT_PUBLIC_KEY_FILE),
        read_fixture(ACCOUNT_SIGNING_SEED_FILE),
    )
    .expect("test account signing authority");
    let credential = issuer
        .issue_runtime_credential(&fence(), permissions(), unix_seconds(), 300)
        .expect("issue runtime JWT");
    let public_key = credential.user_public_key().to_owned();
    let runtime = JetStreamClient::connect_runtime_with_jwt(
        &required(ENDPOINT),
        RuntimeNatsIdentity::new("notes_runtime", 2, 5).expect("runtime identity"),
        credential,
    )
    .await
    .unwrap_or_else(|error| panic!("connect generated runtime before revocation: {error}"));
    (runtime, public_key)
}

async fn wait_for_file(path: &str) {
    for _ in 0..600 {
        if Path::new(path).exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for revocation command");
}

async fn wait_for_disconnect(runtime: &RuntimeJetStreamConnection) -> bool {
    for _ in 0..100 {
        if !runtime.is_connected() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

fn fence() -> NatsRuntimeCredentialFenceV1 {
    NatsRuntimeCredentialFenceV1::new("notes", "registration_notes", "notes_runtime", 2, 5, 3)
        .expect("runtime fence")
}

fn permissions() -> NatsJwtPermissionSetV1 {
    let subject =
        DurableSubjectV1::new(StreamKindV1::Event, "notes", "changed", 1).expect("subject");
    NatsJwtPermissionSetV1::new(vec![subject], Vec::new()).expect("exact runtime permission")
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(required(name))
        .expect("read JWT fixture")
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

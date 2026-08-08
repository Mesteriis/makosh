//! Live inherited-control conformance for Account JWT resolver updates.

use makosh_runtime_protocol::v1::{
    ApplyEventsAccountJwtUpdateRequestV1, EventsAuthorityRuntimeControlRequestV1,
    EventsAuthorityRuntimeControlResponseV1,
    events_authority_runtime_control_request_v1::Operation as AuthorityOperation,
    events_authority_runtime_control_response_v1::Result as AuthorityResult,
};
use prost::Message;

use super::{
    answer_vault_request, assert_ready, complete_descriptor_and_signer_bootstrap, read_frame,
    start_runtime_with_account, write_frame,
};

const ACCOUNT_PUBLIC_KEY_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_PUBLIC_KEY_FILE";
const ACCOUNT_SIGNING_SEED_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_SIGNING_SEED_FILE";
const ACCOUNT_JWT_FILE: &str = "MAKOSH_NATS_JWT_ACCOUNT_JWT_FILE";
const RESOLVER_CREDENTIALS_FILE: &str = "MAKOSH_NATS_JWT_RESOLVER_UPDATE_CREDS_FILE";
const LEASE_ID: &str = "0123456789abcdef0123456789abcdef";

#[test]
#[ignore = "requires the JWT resolver Docker contour"]
fn authority_runtime_updates_the_resolver_only_with_vault_system_credentials() {
    let (mut kernel, worker, signer_seed) = start_runtime_with_account(
        read_fixture(ACCOUNT_PUBLIC_KEY_FILE),
        read_fixture(ACCOUNT_SIGNING_SEED_FILE),
    );
    complete_descriptor_and_signer_bootstrap(&mut kernel, &signer_seed);
    assert_ready(&mut kernel);
    write_frame(
        &mut kernel,
        &EventsAuthorityRuntimeControlRequestV1 {
            operation: Some(AuthorityOperation::ApplyAccountJwtUpdate(
                ApplyEventsAccountJwtUpdateRequestV1 {
                    resolver_credential_revision: 1,
                    signed_account_jwt: read_fixture(ACCOUNT_JWT_FILE),
                },
            )),
        }
        .encode_to_vec(),
    );
    answer_vault_request(&mut kernel, LEASE_ID.as_bytes().to_vec());
    answer_vault_request(
        &mut kernel,
        read_fixture(RESOLVER_CREDENTIALS_FILE).into_bytes(),
    );
    let response =
        EventsAuthorityRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
            .expect("Account JWT update response");
    assert!(matches!(
        response.result,
        Some(AuthorityResult::AccountJwtUpdated(value))
            if response.error_code.is_empty() && value.resolver_credential_revision == 1
    ));
    drop(kernel);
    assert!(worker.join().expect("authority worker").is_err());
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

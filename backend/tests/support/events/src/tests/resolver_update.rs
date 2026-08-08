use makosh_events_jetstream::{
    NatsAccountJwtUpdateV1, NatsResolverCredentialFenceV1, NatsResolverCredentialLeaseAdapterV1,
    NatsResolverSystemCredentialsV1, NatsVaultRouteContextV1, ResolverUpdateErrorV1,
};
use nats_jwt::{KeyPair, Token};

use super::support::ScriptedVaultRoute;

#[test]
fn resolver_update_accepts_an_account_jwt_bound_to_its_account_subject() {
    let operator = KeyPair::new_operator();
    let account = KeyPair::new_account();
    let signer = KeyPair::new_account();
    let jwt = Token::new_account(account.public_key())
        .add_signing_key(signer.public_key())
        .sign(&operator);
    assert_eq!(operator.public_key().len(), 56);
    assert_eq!(account.public_key().len(), 56);
    assert_eq!(signer.public_key().len(), 56);
    let update = NatsAccountJwtUpdateV1::new(account.public_key(), jwt);

    if let Err(error) = update {
        panic!("valid Account JWT was rejected: {error:?}");
    }
}

#[test]
fn resolver_update_rejects_a_jwt_for_a_different_account() {
    let operator = KeyPair::new_operator();
    let account = KeyPair::new_account();
    let different_account = KeyPair::new_account();
    let jwt = Token::new_account(account.public_key()).sign(&operator);

    let result = NatsAccountJwtUpdateV1::new(different_account.public_key(), jwt);

    assert!(matches!(
        result,
        Err(ResolverUpdateErrorV1::AccountMismatch)
    ));
}

#[test]
fn system_credentials_require_both_bounded_creds_sections() {
    let result = NatsResolverSystemCredentialsV1::new("-----BEGIN NATS USER JWT-----\nvalue");

    assert!(matches!(
        result,
        Err(ResolverUpdateErrorV1::InvalidCredentials)
    ));
}

#[tokio::test]
async fn resolver_credentials_are_vault_routed_and_fenced_to_the_authority_runtime() {
    let mut adapter = NatsResolverCredentialLeaseAdapterV1::new(
        ScriptedVaultRoute::new(vec![
            Ok(b"0123456789abcdef0123456789abcdef".to_vec()),
            Ok(system_credentials().into_bytes()),
        ]),
        context(),
    );
    let fence =
        NatsResolverCredentialFenceV1::new("events_authority", "events_authority_runtime", 4, 9, 3)
            .expect("resolver fence");

    adapter
        .resolve_system_credentials(&fence)
        .await
        .expect("System credentials");

    let route = adapter.into_route_port();
    assert_eq!(route.routes.len(), 2);
    assert!(route.routes.iter().all(|request| {
        request.registration_id == "events_authority"
            && request.runtime_instance_id == "events_authority_runtime"
            && request.caller_runtime_generation == 4
            && request.grant_epoch == 9
    }));
}

#[test]
fn resolver_credential_fence_rejects_an_unfenced_revision() {
    assert!(NatsResolverCredentialFenceV1::new(
        "events_authority",
        "events_authority_runtime",
        4,
        9,
        0,
    )
    .is_err());
}

fn context() -> NatsVaultRouteContextV1 {
    let public_key = makosh_vault_protocol::VaultResponseRecipientV1::generate()
        .public_key()
        .as_bytes()
        .to_owned();
    NatsVaultRouteContextV1::new("vault_instance".to_owned(), 7, public_key).expect("Vault context")
}

fn system_credentials() -> String {
    [
        "-----BEGIN NATS USER JWT-----",
        "credential",
        "------END NATS USER JWT------",
        "",
        "-----BEGIN USER NKEY SEED-----",
        "credential",
        "------END USER NKEY SEED------",
    ]
    .join("\n")
}

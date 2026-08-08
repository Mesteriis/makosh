//! NATS account-signing authority conformance tests.

use makosh_events_authority::{NatsCredentialAuthorityErrorV1, NatsJwtCredentialAuthorityV1};
use makosh_events_jetstream::{
    DurableSubjectV1, NatsAccountSignerFenceV1, NatsJwtPermissionSetV1,
    NatsRuntimeCredentialFenceV1, NatsVaultRouteContextV1, StreamKindV1,
};
use nats_jwt::KeyPair;
use zeroize::Zeroizing;

use super::support::ScriptedVaultRoute;

#[tokio::test]
async fn authority_enrolls_the_validated_account_signer_only_through_vault() {
    let account = KeyPair::new_account();
    let account_public_key = account.public_key();
    let account_seed = account.seed().expect("account seed");
    let mut authority = NatsJwtCredentialAuthorityV1::new(
        account_public_key,
        ScriptedVaultRoute::new(vec![Ok(vec![b'a'; 32]), Ok(vec![7; 16])]),
        context(),
    )
    .expect("authority");

    authority
        .enroll_account_signer(
            &signer_fence(),
            Zeroizing::new(account_seed.as_bytes().to_vec()),
        )
        .await
        .expect("account signer enrollment");

    let route = authority.into_route_port();
    assert_eq!(route.routes.len(), 2);
    assert!(route.routes.iter().all(|request| {
        request.registration_id == "events_authority"
            && request.runtime_instance_id == "events_authority_runtime"
            && request.caller_runtime_generation == 4
            && request.grant_epoch == 9
            && !request
                .ciphertext
                .windows(account_seed.len())
                .any(|window| window == account_seed.as_bytes())
    }));
}

#[tokio::test]
async fn authority_issues_runtime_jwt_after_one_vault_resolution() {
    let account = KeyPair::new_account();
    let account_public_key = account.public_key();
    let account_seed = account.seed().expect("account seed");
    let mut authority = NatsJwtCredentialAuthorityV1::new(
        account_public_key,
        ScriptedVaultRoute::new(vec![
            Ok(vec![b'a'; 32]),
            Ok(account_seed.as_bytes().to_vec()),
        ]),
        context(),
    )
    .expect("authority");

    let credential = authority
        .issue_runtime_credential(&signer_fence(), &runtime_fence(), permissions(), 1_000, 60)
        .await
        .expect("runtime credential");
    assert!(credential.user_public_key().starts_with('U'));
    assert_eq!(credential.expires_at_unix_seconds(), 1_060);
    assert!(!format!("{credential:?}").contains(account_seed.as_str()));

    let route = authority.into_route_port();
    assert_eq!(route.routes.len(), 2);
    assert!(route.routes.iter().all(|request| {
        request.registration_id == "events_authority"
            && request.runtime_instance_id == "events_authority_runtime"
            && !request
                .ciphertext
                .windows(account_seed.len())
                .any(|window| window == account_seed.as_bytes())
    }));
}

#[tokio::test]
async fn authority_rejects_a_non_account_signer_before_any_vault_call() {
    let account = KeyPair::new_account();
    let user = KeyPair::new_user();
    let user_seed = user.seed().expect("user seed");
    let mut authority = NatsJwtCredentialAuthorityV1::new(
        account.public_key(),
        ScriptedVaultRoute::new(Vec::new()),
        context(),
    )
    .expect("authority");

    assert_eq!(
        authority
            .enroll_account_signer(
                &signer_fence(),
                Zeroizing::new(user_seed.as_bytes().to_vec())
            )
            .await,
        Err(NatsCredentialAuthorityErrorV1::Rejected)
    );
    assert!(authority.into_route_port().routes.is_empty());
}

fn context() -> NatsVaultRouteContextV1 {
    NatsVaultRouteContextV1::new("vault_instance".to_owned(), 7, vault_public_key())
        .expect("Vault context")
}

fn signer_fence() -> NatsAccountSignerFenceV1 {
    NatsAccountSignerFenceV1::new("events_authority", "events_authority_runtime", 4, 9, 2)
        .expect("signer fence")
}

fn runtime_fence() -> NatsRuntimeCredentialFenceV1 {
    NatsRuntimeCredentialFenceV1::new("notes", "registration_notes", "notes_runtime", 2, 5, 3)
        .expect("runtime fence")
}

fn permissions() -> NatsJwtPermissionSetV1 {
    let subject =
        DurableSubjectV1::new(StreamKindV1::Event, "notes", "changed", 1).expect("durable subject");
    NatsJwtPermissionSetV1::new(vec![subject], Vec::new()).expect("permission set")
}

fn vault_public_key() -> [u8; 32] {
    makosh_vault_protocol::VaultResponseRecipientV1::generate()
        .public_key()
        .as_bytes()
        .to_owned()
}

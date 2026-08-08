use makosh_events_jetstream::{
    DurableSubjectV1, NatsJwtConsumerGrantV1, NatsJwtIssueErrorV1, NatsJwtPermissionSetV1,
    NatsRuntimeCredentialFenceV1, RuntimeNatsJwtIssuerV1, StreamKindV1,
};
use nats_jwt::KeyPair;

const NOW_UNIX_SECONDS: u64 = 1_800_000_000;

#[test]
fn issues_short_lived_non_bearer_runtime_jwt_without_diagnostic_disclosure() {
    let account = KeyPair::new_account();
    let signing_key = KeyPair::new_account();
    let signing_seed = signing_key.seed().expect("account signing seed");
    let issuer = RuntimeNatsJwtIssuerV1::from_account_signing_seed(
        account.public_key(),
        signing_seed.clone(),
    )
    .expect("JWT issuer");
    let credential = issuer
        .issue_runtime_credential(&fence(), permissions(), NOW_UNIX_SECONDS, 300)
        .expect("runtime JWT");

    assert!(credential.user_public_key().starts_with('U'));
    assert_eq!(credential.expires_at_unix_seconds(), NOW_UNIX_SECONDS + 300);
    let diagnostics = format!("{issuer:?} {credential:?}");
    assert!(!diagnostics.contains(&signing_seed));
    assert!(!diagnostics.contains(credential.user_public_key()));
}

#[test]
fn rejects_invalid_signing_authority_and_unbounded_runtime_jwt_requests() {
    assert!(matches!(
        RuntimeNatsJwtIssuerV1::from_account_signing_seed("AINVALID", "not-a-seed"),
        Err(NatsJwtIssueErrorV1::InvalidIssuer)
    ));

    let account = KeyPair::new_account();
    let signing_key = KeyPair::new_account();
    let issuer = RuntimeNatsJwtIssuerV1::from_account_signing_seed(
        account.public_key(),
        signing_key.seed().expect("account signing seed"),
    )
    .expect("JWT issuer");
    for invalid_ttl in [0, 601] {
        assert!(matches!(
            issuer.issue_runtime_credential(&fence(), permissions(), NOW_UNIX_SECONDS, invalid_ttl),
            Err(NatsJwtIssueErrorV1::InvalidTtl)
        ));
    }
}

#[test]
fn rejects_empty_permission_sets_before_creating_runtime_key_material() {
    assert!(matches!(
        NatsJwtPermissionSetV1::new(Vec::new(), Vec::new()),
        Err(NatsJwtIssueErrorV1::InvalidPermissions)
    ));
}

fn fence() -> NatsRuntimeCredentialFenceV1 {
    NatsRuntimeCredentialFenceV1::new("notes", "registration_notes", "notes_runtime", 2, 5, 3)
        .expect("runtime fence")
}

fn permissions() -> NatsJwtPermissionSetV1 {
    let publish =
        DurableSubjectV1::new(StreamKindV1::Event, "notes", "changed", 1).expect("publish subject");
    let consume = NatsJwtConsumerGrantV1::new(
        DurableSubjectV1::new(StreamKindV1::Command, "notes", "apply", 1)
            .expect("consumer subject"),
        "notes_apply",
    )
    .expect("consumer grant");
    NatsJwtPermissionSetV1::new(vec![publish], vec![consume]).expect("permissions")
}

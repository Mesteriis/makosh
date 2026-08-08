use makosh_storage_control::{
    StorageLifecycleErrorV1, StorageLifecycleStateV1, StorageLifecycleV1,
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};

fn binding(
    storage_generation: u64,
    role_epoch: u64,
    credential_lease_revision: u64,
    runtime_principal: &str,
    _: &str,
) -> StorageBindingV1 {
    let identity = StorageBindingIdentityV1::new(
        "storage_main".into(),
        "makosh".into(),
        "notes".into(),
        "registration_notes".into(),
        "runtime_notes".into(),
    )
    .expect("valid storage identity");
    let fences = StorageBindingFencesV1::new(
        storage_generation,
        1,
        1,
        role_epoch,
        credential_lease_revision,
        1,
    )
    .expect("non-zero fences");
    let budgets = StorageEffectiveBudgetsV1::new(4, 5_000).expect("valid budgets");
    let access = StorageBindingAccessV1::new(
        runtime_principal.into(),
        "runtime_registration_notes_1".into(),
        budgets,
        [1; 32],
    )
    .expect("valid access");
    StorageBindingV1::new(identity, fences, access).expect("valid storage lifecycle binding")
}

#[test]
fn requires_revocation_before_replacing_an_active_binding() {
    let first = binding(1, 1, 1, "runtime_notes_1", "pool_notes_1");
    let latest = binding(2, 2, 2, "runtime_notes_2", "pool_notes_2");
    let mut lifecycle = StorageLifecycleV1::default();

    lifecycle
        .activate(first)
        .expect("first binding is accepted");
    assert_eq!(
        lifecycle.activate(latest),
        Err(StorageLifecycleErrorV1::RotationRequiresRevocation)
    );
    assert_eq!(lifecycle.state(), StorageLifecycleStateV1::Active);
}

#[test]
fn preserves_the_active_binding_until_the_revoke_sequence_starts() {
    let mut lifecycle = StorageLifecycleV1::default();

    lifecycle
        .activate(binding(2, 2, 2, "runtime_notes_2", "pool_notes_2"))
        .expect("initial binding");

    assert_eq!(
        lifecycle
            .active_binding()
            .map(|binding| binding.access().runtime_principal()),
        Some("runtime_notes_2")
    );
}

#[test]
fn blocks_replacement_until_revocation_is_completed() {
    let mut lifecycle = StorageLifecycleV1::default();
    lifecycle
        .activate(binding(1, 1, 1, "runtime_notes_1", "pool_notes_1"))
        .expect("initial binding");

    let revoking = lifecycle.begin_revocation().expect("begin revocation");
    assert_eq!(revoking.access().runtime_principal(), "runtime_notes_1");
    assert_eq!(lifecycle.state(), StorageLifecycleStateV1::Revoking);
    assert_eq!(
        lifecycle.activate(binding(2, 2, 2, "runtime_notes_2", "pool_notes_2")),
        Err(StorageLifecycleErrorV1::RevocationInProgress)
    );

    lifecycle
        .complete_revocation()
        .expect("complete infrastructure fencing");
    lifecycle
        .activate(binding(2, 2, 2, "runtime_notes_2", "pool_notes_2"))
        .expect("accept new binding after fencing");
}

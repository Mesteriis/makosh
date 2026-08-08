use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingErrorV1, StorageBindingFencesV1,
    StorageBindingIdentityV1, StorageEffectiveBudgetsV1,
    v1::{
        StorageBindingV1 as StorageBindingMessageV1,
        StorageEffectiveBudgetsV1 as StorageEffectiveBudgetsMessageV1,
    },
    validation::validate_storage_binding_message,
};

fn binding_message() -> StorageBindingMessageV1 {
    StorageBindingMessageV1 {
        storage_instance_id: "storage_main".into(),
        storage_generation: 1,
        database_id: "makosh".into(),
        owner: "notes".into(),
        registration_id: "registration_notes".into(),
        runtime_instance_id: "runtime_notes".into(),
        runtime_generation: 1,
        grant_epoch: 1,
        role_epoch: 1,
        runtime_principal: "runtime_notes".into(),
        pool_alias: "runtime_registration_notes_1".into(),
        effective_budgets: Some(StorageEffectiveBudgetsMessageV1 {
            max_connections: 4,
            statement_timeout_millis: 5_000,
        }),
        credential_lease_revision: 1,
        storage_bundle_revision: 1,
        storage_bundle_digest: vec![1; 32],
    }
}

#[test]
fn rejects_partial_fences_and_empty_bundle_digest() {
    assert_eq!(
        StorageBindingFencesV1::new(1, 1, 0, 1, 1, 1),
        Err(StorageBindingErrorV1::Fence)
    );

    let budgets = StorageEffectiveBudgetsV1::new(1, 1).expect("bounded budget");
    assert_eq!(
        StorageBindingAccessV1::new("runtime".into(), "pool".into(), budgets, [0; 32]),
        Err(StorageBindingErrorV1::Digest)
    );
}

#[test]
fn rejects_identity_outside_the_storage_naming_contract() {
    let result = StorageBindingIdentityV1::new(
        "storage".into(),
        "makosh".into(),
        "Notes".into(),
        "registration".into(),
        "runtime".into(),
    );

    assert_eq!(result, Err(StorageBindingErrorV1::Owner));
}

#[test]
fn rejects_a_pool_alias_that_is_not_derived_from_the_runtime_generation() {
    let mut binding = binding_message();
    binding.pool_alias = "runtime_registration_notes_2".into();

    assert_eq!(
        validate_storage_binding_message(&binding),
        Err(StorageBindingErrorV1::PoolAlias)
    );
}

#[test]
fn validates_the_fully_fenced_protobuf_binding() {
    assert_eq!(validate_storage_binding_message(&binding_message()), Ok(()));
}

#[test]
fn rejects_protobuf_binding_without_budgets_or_digest() {
    let mut missing_budgets = binding_message();
    missing_budgets.effective_budgets = None;
    let mut short_digest = binding_message();
    short_digest.storage_bundle_digest.clear();

    assert_eq!(
        validate_storage_binding_message(&missing_budgets),
        Err(StorageBindingErrorV1::Budget)
    );
    assert_eq!(
        validate_storage_binding_message(&short_digest),
        Err(StorageBindingErrorV1::Digest)
    );
}

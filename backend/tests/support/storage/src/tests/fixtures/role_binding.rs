//! One fully fenced binding used to derive PostgreSQL role specifications.

use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};

pub fn storage_role_binding(owner: &str, runtime_principal: &str) -> StorageBindingV1 {
    storage_role_binding_in_database("makosh", owner, runtime_principal)
}

pub fn storage_role_binding_in_database(
    database_id: &str,
    owner: &str,
    runtime_principal: &str,
) -> StorageBindingV1 {
    let registration_id = format!("registration_{owner}");
    let identity = StorageBindingIdentityV1::new(
        "storage_main".into(),
        database_id.into(),
        owner.into(),
        registration_id.clone(),
        format!("runtime_{owner}"),
    )
    .expect("valid storage test identity");
    let fences =
        StorageBindingFencesV1::new(1, 1, 1, 1, 1, 1).expect("non-zero storage test fences");
    let budgets = StorageEffectiveBudgetsV1::new(8, 5_000).expect("bounded storage test budgets");
    let access = StorageBindingAccessV1::new(
        runtime_principal.into(),
        format!("runtime_{registration_id}_1"),
        budgets,
        [1; 32],
    )
    .expect("valid storage test access");
    StorageBindingV1::new(identity, fences, access).expect("valid storage test binding")
}

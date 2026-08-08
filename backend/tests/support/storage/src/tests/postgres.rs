use makosh_storage_postgres::{StorageRoleErrorV1, StorageRoleSpecV1};

use super::fixtures::storage_role_binding;

#[test]
fn rejects_an_unsafe_ddl_owner_identifier() {
    let result = StorageRoleSpecV1::from_binding(
        "ddl_owner;drop".into(),
        storage_role_binding("notes", "runtime_notes"),
    );

    assert_eq!(result, Err(StorageRoleErrorV1::Identifier));
}

#[test]
fn derives_a_credential_free_runtime_role_specification_from_a_binding() {
    let binding = storage_role_binding("notes", "runtime_notes");
    let spec = StorageRoleSpecV1::from_binding("ddl_notes".into(), binding.clone())
        .expect("valid storage role specification from binding");

    assert_eq!(spec.ddl_owner(), "ddl_notes");
    assert_eq!(spec.owner_id(), "notes");
    assert_eq!(spec.runtime_principal(), "runtime_notes");
    assert_eq!(spec.max_connections(), 8);
    assert_eq!(spec.statement_timeout_millis(), 5_000);
    assert_eq!(spec.binding(), &binding);
}

#[test]
fn derives_a_bounded_stable_platform_ddl_role_from_the_owner_identity() {
    let spec = StorageRoleSpecV1::platform_binding(storage_role_binding("notes", "runtime_notes"))
        .expect("derive platform role specification");

    assert!(spec.ddl_owner().starts_with("storage_ddl_"));
    assert!(spec.ddl_owner().len() <= 63);
}

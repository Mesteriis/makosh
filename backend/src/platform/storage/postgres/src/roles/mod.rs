//! PostgreSQL roles owned by Storage Control.

mod audit;
mod ledger;
mod privileges;
mod reconciliation;
mod revocation;
mod spec;

pub use audit::{
    StorageDataPrivilegeAuditV1, StorageRoleAuditV1, read_storage_data_privilege_audit,
    read_storage_role_audit,
};
pub use ledger::{ensure_role_ledger_binding, repair_legacy_storage_ledger_binding};
pub use privileges::reconcile_owner_data_privileges;
pub use reconciliation::{
    ensure_storage_roles, read_runtime_role_scram_verifier, set_runtime_role_password,
};
pub use revocation::{
    PostgresRuntimeFenceAdapterV1, PostgresRuntimeFenceV1, fence_postgres_runtime_role,
};
pub use spec::{StorageRoleErrorV1, StorageRoleSpecV1};

pub(crate) use privileges::revoke_owner_data_privileges;

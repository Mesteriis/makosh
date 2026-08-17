//! PostgreSQL admin adapter used exclusively by Storage Control runtime.

mod bootstrap;
mod connection;
mod migrations;
mod readiness;
mod roles;

pub const PLATFORM_ADMIN_USERNAME: &str = "makosh_postgres_admin";

pub use bootstrap::{InitdbPasswordFileV1, ensure_platform_schemas};
pub use connection::{
    PostgresAdapterErrorV1, PostgresAdminConnectorV1, PostgresRuntimeSessionProbeV1,
};
pub use migrations::apply_storage_bundle;
pub use readiness::{PostgresReadinessV1, read_readiness};
pub use roles::{
    PostgresRuntimeFenceAdapterV1, PostgresRuntimeFenceV1, StorageDataPrivilegeAuditV1,
    StorageRoleAuditV1, StorageRoleErrorV1, StorageRoleSpecV1, ensure_storage_roles,
    fence_postgres_runtime_role, read_runtime_role_scram_verifier,
    read_storage_data_privilege_audit, read_storage_role_audit, reconcile_owner_data_privileges,
    repair_legacy_storage_ledger_binding, set_runtime_role_password,
};

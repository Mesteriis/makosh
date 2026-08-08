//! Durable current-role binding with identity and fence checks.

use sqlx::{query, query_as};

use crate::{PostgresAdapterErrorV1, PostgresAdminConnectorV1};

use super::StorageRoleSpecV1;

pub async fn ensure_role_ledger_binding(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let requested = RoleLedgerBindingV1::from_spec(spec)?;
    match read_owner_binding(connector, spec).await? {
        Some(current) => reconcile_existing_binding(connector, &current, &requested).await,
        None => insert_new_binding(connector, &requested).await,
    }
}

async fn read_owner_binding(
    connector: &PostgresAdminConnectorV1,
    spec: &StorageRoleSpecV1,
) -> Result<Option<StoredRoleLedgerBindingV1>, PostgresAdapterErrorV1> {
    query_as::<_, RoleLedgerRow>(
        "SELECT owner_id, ddl_owner, runtime_principal, registration_id, runtime_instance_id, \
                storage_generation, runtime_generation, grant_epoch, role_epoch, \
                credential_lease_revision, storage_bundle_revision \
         FROM makosh_platform.storage_role_ledger WHERE owner_id = $1",
    )
    .bind(spec.owner_id())
    .fetch_optional(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::RoleBinding)?
    .map(StoredRoleLedgerBindingV1::from_row)
    .transpose()
}

async fn reconcile_existing_binding(
    connector: &PostgresAdminConnectorV1,
    current: &StoredRoleLedgerBindingV1,
    requested: &RoleLedgerBindingV1,
) -> Result<(), PostgresAdapterErrorV1> {
    if !current.permits(requested) {
        return Err(PostgresAdapterErrorV1::RoleBinding);
    }
    if current.matches(requested) {
        return Ok(());
    }
    update_binding(connector, requested).await
}

async fn insert_new_binding(
    connector: &PostgresAdminConnectorV1,
    requested: &RoleLedgerBindingV1,
) -> Result<(), PostgresAdapterErrorV1> {
    let result = query(
        "INSERT INTO makosh_platform.storage_role_ledger ( \
             owner_id, ddl_owner, runtime_principal, registration_id, runtime_instance_id, \
             storage_generation, runtime_generation, grant_epoch, role_epoch, \
             credential_lease_revision, storage_bundle_revision \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) ON CONFLICT DO NOTHING",
    )
    .bind(&requested.owner_id)
    .bind(&requested.ddl_owner)
    .bind(&requested.runtime_principal)
    .bind(&requested.registration_id)
    .bind(&requested.runtime_instance_id)
    .bind(requested.storage_generation)
    .bind(requested.runtime_generation)
    .bind(requested.grant_epoch)
    .bind(requested.role_epoch)
    .bind(requested.credential_lease_revision)
    .bind(requested.storage_bundle_revision)
    .execute(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::RoleBinding)?;
    if result.rows_affected() != 1 {
        return Err(PostgresAdapterErrorV1::RoleBinding);
    }
    Ok(())
}

async fn update_binding(
    connector: &PostgresAdminConnectorV1,
    requested: &RoleLedgerBindingV1,
) -> Result<(), PostgresAdapterErrorV1> {
    query(
        "UPDATE makosh_platform.storage_role_ledger SET \
             runtime_principal = $2, registration_id = $3, runtime_instance_id = $4, \
             storage_generation = $5, runtime_generation = $6, grant_epoch = $7, \
             role_epoch = $8, credential_lease_revision = $9, storage_bundle_revision = $10, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE owner_id = $1 AND ddl_owner = $11",
    )
    .bind(&requested.owner_id)
    .bind(&requested.runtime_principal)
    .bind(&requested.registration_id)
    .bind(&requested.runtime_instance_id)
    .bind(requested.storage_generation)
    .bind(requested.runtime_generation)
    .bind(requested.grant_epoch)
    .bind(requested.role_epoch)
    .bind(requested.credential_lease_revision)
    .bind(requested.storage_bundle_revision)
    .bind(&requested.ddl_owner)
    .execute(connector.pool())
    .await
    .map_err(|_| PostgresAdapterErrorV1::RoleBinding)?;
    Ok(())
}

type RoleLedgerRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
);

#[derive(Eq, PartialEq)]
struct RoleLedgerBindingV1 {
    owner_id: String,
    ddl_owner: String,
    runtime_principal: String,
    registration_id: String,
    runtime_instance_id: String,
    storage_generation: i64,
    runtime_generation: i64,
    grant_epoch: i64,
    role_epoch: i64,
    credential_lease_revision: i64,
    storage_bundle_revision: i64,
}

impl RoleLedgerBindingV1 {
    fn from_spec(spec: &StorageRoleSpecV1) -> Result<Self, PostgresAdapterErrorV1> {
        let identity = spec.binding().identity();
        let fences = spec.binding().fences();
        Ok(Self {
            owner_id: spec.owner_id().to_owned(),
            ddl_owner: spec.ddl_owner().to_owned(),
            runtime_principal: spec.runtime_principal().to_owned(),
            registration_id: identity.registration_id().to_owned(),
            runtime_instance_id: identity.runtime_instance_id().to_owned(),
            storage_generation: database_fence(fences.storage_generation())?,
            runtime_generation: database_fence(fences.runtime_generation())?,
            grant_epoch: database_fence(fences.grant_epoch())?,
            role_epoch: database_fence(fences.role_epoch())?,
            credential_lease_revision: database_fence(fences.credential_lease_revision())?,
            storage_bundle_revision: database_fence(fences.storage_bundle_revision())?,
        })
    }

    fn permits(&self, requested: &Self) -> bool {
        self == requested || self.is_strict_successor(requested)
    }

    fn is_strict_successor(&self, requested: &Self) -> bool {
        self.owner_id == requested.owner_id
            && self.ddl_owner == requested.ddl_owner
            && self.registration_id == requested.registration_id
            && self.runtime_principal != requested.runtime_principal
            && self.runtime_instance_id != requested.runtime_instance_id
            && requested.storage_generation >= self.storage_generation
            && requested.runtime_generation > self.runtime_generation
            && requested.grant_epoch >= self.grant_epoch
            && requested.role_epoch > self.role_epoch
            && requested.credential_lease_revision > self.credential_lease_revision
            && requested.storage_bundle_revision >= self.storage_bundle_revision
    }
}

enum StoredRoleLedgerBindingV1 {
    Legacy {
        owner_id: String,
        ddl_owner: String,
        runtime_principal: String,
    },
    Current(RoleLedgerBindingV1),
}

impl StoredRoleLedgerBindingV1 {
    fn from_row(row: RoleLedgerRow) -> Result<Self, PostgresAdapterErrorV1> {
        let (
            owner_id,
            ddl_owner,
            runtime_principal,
            registration_id,
            runtime_instance_id,
            storage,
            runtime,
            grant,
            role,
            lease,
            bundle,
        ) = row;
        let values = (
            registration_id,
            runtime_instance_id,
            storage,
            runtime,
            grant,
            role,
            lease,
            bundle,
        );
        if values.0.is_none()
            && values.1.is_none()
            && values.2.is_none()
            && values.3.is_none()
            && values.4.is_none()
            && values.5.is_none()
            && values.6.is_none()
            && values.7.is_none()
        {
            return Ok(Self::Legacy {
                owner_id,
                ddl_owner,
                runtime_principal,
            });
        }
        Ok(Self::Current(RoleLedgerBindingV1 {
            owner_id,
            ddl_owner,
            runtime_principal,
            registration_id: values.0.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
            runtime_instance_id: values.1.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
            storage_generation: values.2.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
            runtime_generation: values.3.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
            grant_epoch: values.4.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
            role_epoch: values.5.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
            credential_lease_revision: values.6.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
            storage_bundle_revision: values.7.ok_or(PostgresAdapterErrorV1::RoleBinding)?,
        }))
    }

    fn permits(&self, requested: &RoleLedgerBindingV1) -> bool {
        match self {
            Self::Legacy {
                owner_id,
                ddl_owner,
                runtime_principal,
            } => {
                owner_id == &requested.owner_id
                    && ddl_owner == &requested.ddl_owner
                    && runtime_principal == &requested.runtime_principal
            }
            Self::Current(current) => current.permits(requested),
        }
    }

    fn matches(&self, requested: &RoleLedgerBindingV1) -> bool {
        matches!(self, Self::Current(current) if current == requested)
    }
}

fn database_fence(value: u64) -> Result<i64, PostgresAdapterErrorV1> {
    i64::try_from(value).map_err(|_| PostgresAdapterErrorV1::RoleBinding)
}

#[cfg(test)]
mod tests {
    use super::RoleLedgerBindingV1;

    fn binding(
        runtime_principal: &str,
        runtime_instance_id: &str,
        runtime_generation: i64,
        role_epoch: i64,
        credential_lease_revision: i64,
    ) -> RoleLedgerBindingV1 {
        RoleLedgerBindingV1 {
            owner_id: "platform.scheduler".to_owned(),
            ddl_owner: "storage_ddl_platform_scheduler".to_owned(),
            runtime_principal: runtime_principal.to_owned(),
            registration_id: "scheduler".to_owned(),
            runtime_instance_id: runtime_instance_id.to_owned(),
            storage_generation: 1,
            runtime_generation,
            grant_epoch: 1,
            role_epoch,
            credential_lease_revision,
            storage_bundle_revision: 1,
        }
    }

    #[test]
    fn successor_accepts_fence_gaps_left_by_failed_launch_reservations() {
        let current = binding("runtime_scheduler_1", "scheduler-1", 1, 1, 1);
        let successor = binding("runtime_scheduler_4", "scheduler-4", 4, 4, 4);

        assert!(current.permits(&successor));
    }

    #[test]
    fn successor_still_rejects_non_advancing_fences() {
        let current = binding("runtime_scheduler_1", "scheduler-1", 1, 3, 3);
        let stale_role = binding("runtime_scheduler_4", "scheduler-4", 4, 3, 4);
        let stale_lease = binding("runtime_scheduler_4", "scheduler-4", 4, 4, 3);

        assert!(!current.permits(&stale_role));
        assert!(!current.permits(&stale_lease));
    }
}

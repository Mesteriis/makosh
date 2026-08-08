//! Validated opaque PostgreSQL role names and hard connection limits.

use makosh_storage_protocol::StorageBindingV1;
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRoleErrorV1 {
    Identifier,
    Fence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageRoleSpecV1 {
    ddl_owner: String,
    binding: StorageBindingV1,
}

impl StorageRoleSpecV1 {
    pub fn platform_binding(binding: StorageBindingV1) -> Result<Self, StorageRoleErrorV1> {
        let digest = Sha256::digest(binding.identity().owner().as_bytes());
        let ddl_owner = format!(
            "storage_ddl_{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            digest[0],
            digest[1],
            digest[2],
            digest[3],
            digest[4],
            digest[5],
            digest[6],
            digest[7],
            digest[8],
            digest[9],
            digest[10],
            digest[11],
        );
        Self::from_binding(ddl_owner, binding)
    }

    pub fn from_binding(
        ddl_owner: String,
        binding: StorageBindingV1,
    ) -> Result<Self, StorageRoleErrorV1> {
        if !valid_identifier(&ddl_owner) {
            return Err(StorageRoleErrorV1::Identifier);
        }
        if binding_fences_fit_database(&binding).is_err() {
            return Err(StorageRoleErrorV1::Fence);
        }
        Ok(Self { ddl_owner, binding })
    }

    pub fn owner_id(&self) -> &str {
        self.binding.identity().owner()
    }

    pub fn storage_schema(&self) -> &'static str {
        if self.owner_id() == "scheduler" {
            "makosh_platform"
        } else {
            "makosh_data"
        }
    }

    pub fn ddl_owner(&self) -> &str {
        &self.ddl_owner
    }

    pub fn runtime_principal(&self) -> &str {
        self.binding.access().runtime_principal()
    }

    pub fn max_connections(&self) -> u16 {
        self.binding.access().effective_budgets().max_connections()
    }

    pub fn statement_timeout_millis(&self) -> u32 {
        self.binding
            .access()
            .effective_budgets()
            .statement_timeout_millis()
    }

    pub fn binding(&self) -> &StorageBindingV1 {
        &self.binding
    }
}

fn binding_fences_fit_database(binding: &StorageBindingV1) -> Result<(), ()> {
    let fences = binding.fences();
    [
        fences.storage_generation(),
        fences.runtime_generation(),
        fences.grant_epoch(),
        fences.role_epoch(),
        fences.credential_lease_revision(),
        fences.storage_bundle_revision(),
    ]
    .into_iter()
    .try_for_each(|value| i64::try_from(value).map(|_| ()).map_err(|_| ()))
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

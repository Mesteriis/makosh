//! Immutable public routing context supplied by the trusted Kernel launch path.

use makosh_vault_protocol::{VaultTransportPublicKey, validate_vault_instance_id};

#[derive(Clone)]
pub struct StorageVaultRouteContextV1 {
    vault_instance_id: String,
    vault_runtime_generation: u64,
    public_key: VaultTransportPublicKey,
}

impl StorageVaultRouteContextV1 {
    pub fn new(
        vault_instance_id: String,
        vault_runtime_generation: u64,
        public_key_x25519: [u8; 32],
    ) -> Result<Self, StorageVaultRouteContextErrorV1> {
        let public_key = VaultTransportPublicKey::from_bytes(public_key_x25519)
            .map_err(|_| StorageVaultRouteContextErrorV1::InvalidPublicKey)?;
        if validate_vault_instance_id(&vault_instance_id).is_err() || vault_runtime_generation == 0
        {
            return Err(StorageVaultRouteContextErrorV1::InvalidGeneration);
        }
        Ok(Self {
            vault_instance_id,
            vault_runtime_generation,
            public_key,
        })
    }

    #[must_use]
    pub const fn vault_runtime_generation(&self) -> u64 {
        self.vault_runtime_generation
    }

    #[must_use]
    pub fn vault_instance_id(&self) -> &str {
        &self.vault_instance_id
    }

    #[must_use]
    pub fn public_key(&self) -> &VaultTransportPublicKey {
        &self.public_key
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageVaultRouteContextErrorV1 {
    InvalidGeneration,
    InvalidPublicKey,
}

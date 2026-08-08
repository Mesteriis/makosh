//! Immutable Vault route context supplied by the verified Blob runtime launch.

use makosh_vault_protocol::{LeaseAudienceV1, VaultTransportPublicKey, validate_vault_instance_id};

#[derive(Clone)]
pub struct BlobVaultRouteContextV1 {
    vault_instance_id: String,
    vault_runtime_generation: u64,
    public_key: VaultTransportPublicKey,
    route_audience: LeaseAudienceV1,
}

impl BlobVaultRouteContextV1 {
    pub fn new(
        vault_instance_id: String,
        vault_runtime_generation: u64,
        public_key_x25519: [u8; 32],
        route_audience: LeaseAudienceV1,
    ) -> Result<Self, BlobVaultRouteContextErrorV1> {
        let public_key = VaultTransportPublicKey::from_bytes(public_key_x25519)
            .map_err(|_| BlobVaultRouteContextErrorV1::InvalidPublicKey)?;
        if validate_vault_instance_id(&vault_instance_id).is_err() || vault_runtime_generation == 0
        {
            return Err(BlobVaultRouteContextErrorV1::InvalidGeneration);
        }
        Ok(Self {
            vault_instance_id,
            vault_runtime_generation,
            public_key,
            route_audience,
        })
    }

    #[must_use]
    pub fn vault_instance_id(&self) -> &str {
        &self.vault_instance_id
    }

    #[must_use]
    pub const fn vault_runtime_generation(&self) -> u64 {
        self.vault_runtime_generation
    }

    #[must_use]
    pub fn public_key(&self) -> &VaultTransportPublicKey {
        &self.public_key
    }

    #[must_use]
    pub fn route_audience(&self) -> &LeaseAudienceV1 {
        &self.route_audience
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlobVaultRouteContextErrorV1 {
    InvalidGeneration,
    InvalidPublicKey,
}

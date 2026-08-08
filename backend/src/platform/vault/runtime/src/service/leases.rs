//! Memory-only credential lease fence for one Vault runtime generation.

use std::collections::BTreeMap;

use getrandom::fill;
use makosh_vault_protocol::{
    CredentialLeaseV1, LeaseAudienceV1, LeaseIdV1, VaultLeaseIssueRequestV1,
    validate_vault_instance_id,
};

const LEASE_ID_BYTES: usize = 16;
const MAX_ACTIVE_LEASES: usize = 1_024;

pub struct LeaseManager {
    vault_instance_id: String,
    runtime_generation: u64,
    active: BTreeMap<String, ActiveLease>,
}

impl LeaseManager {
    pub fn new(vault_instance_id: String, runtime_generation: u64) -> Result<Self, LeaseError> {
        if validate_vault_instance_id(&vault_instance_id).is_err() || runtime_generation == 0 {
            return Err(LeaseError::InvalidRuntime);
        }
        Ok(Self {
            vault_instance_id,
            runtime_generation,
            active: BTreeMap::new(),
        })
    }

    pub fn issue(
        &mut self,
        request: VaultLeaseIssueRequestV1,
        issued_at_unix_seconds: u64,
    ) -> Result<CredentialLeaseV1, LeaseError> {
        if request.vault_instance_id() != self.vault_instance_id {
            return Err(LeaseError::WrongVaultInstance);
        }
        if request.vault_runtime_generation() != self.runtime_generation {
            return Err(LeaseError::StaleRuntimeGeneration);
        }
        self.active
            .retain(|_, active| issued_at_unix_seconds < active.lease.expires_at_unix_seconds());
        if self.active.len() >= MAX_ACTIVE_LEASES {
            return Err(LeaseError::Capacity);
        }
        let lease_id = generate_lease_id()?;
        let requested_ttl_seconds = self.requested_ttl_seconds(&request);
        let lease = CredentialLeaseV1::new(
            lease_id,
            request,
            issued_at_unix_seconds,
            requested_ttl_seconds,
            true,
        )
        .map_err(|_| LeaseError::InvalidLease)?;
        self.active.insert(
            lease.lease_id().as_str().to_owned(),
            ActiveLease {
                lease: lease.clone(),
                resolved: false,
            },
        );
        Ok(lease)
    }

    pub fn consume_once(
        &mut self,
        lease_id: &LeaseIdV1,
        audience: &LeaseAudienceV1,
        now_unix_seconds: u64,
    ) -> Result<CredentialLeaseV1, LeaseError> {
        let active = self
            .active
            .get(lease_id.as_str())
            .ok_or(LeaseError::UnknownOrInvalidatedLease)?;
        let request = active.lease.request();
        if request.vault_instance_id() != self.vault_instance_id
            || request.vault_runtime_generation() != self.runtime_generation
        {
            return Err(LeaseError::StaleRuntimeGeneration);
        }
        if request.audience() != audience {
            return Err(LeaseError::AudienceOrGrantEpochMismatch);
        }
        if now_unix_seconds >= active.lease.expires_at_unix_seconds() {
            let _ = active;
            self.active.remove(lease_id.as_str());
            return Err(LeaseError::ExpiredLease);
        }
        let active = self
            .active
            .get(lease_id.as_str())
            .ok_or(LeaseError::UnknownOrInvalidatedLease)?;
        if active.lease.single_resolve() && active.resolved {
            return Err(LeaseError::AlreadyResolved);
        }
        let lease = active.lease.clone();
        // A successful one-shot use must release its bounded slot immediately.
        // Keeping consumed leases until their TTL turns normal request volume into
        // a capacity denial while providing no additional replay protection: the
        // lease has already been consumed and cannot be accepted again.
        self.active.remove(lease_id.as_str());
        Ok(lease)
    }

    pub fn invalidate_audience(&mut self, audience: &LeaseAudienceV1) {
        self.active
            .retain(|_, active| active.lease.request().audience() != audience);
    }

    pub fn advance_generation(&mut self, next_generation: u64) -> Result<(), LeaseError> {
        if next_generation <= self.runtime_generation {
            return Err(LeaseError::InvalidRuntime);
        }
        self.runtime_generation = next_generation;
        self.active.clear();
        Ok(())
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    fn requested_ttl_seconds(&self, request: &VaultLeaseIssueRequestV1) -> u32 {
        request.purpose().requested_lease_ttl_seconds()
    }
}

struct ActiveLease {
    lease: CredentialLeaseV1,
    resolved: bool,
}

fn generate_lease_id() -> Result<LeaseIdV1, LeaseError> {
    let mut bytes = [0; LEASE_ID_BYTES];
    fill(&mut bytes).map_err(|_| LeaseError::Randomness)?;
    let mut encoded = String::with_capacity(LEASE_ID_BYTES * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("String write cannot fail");
    }
    LeaseIdV1::new(encoded).map_err(|_| LeaseError::InvalidLease)
}

#[derive(Debug, Eq, PartialEq)]
pub enum LeaseError {
    InvalidRuntime,
    WrongVaultInstance,
    StaleRuntimeGeneration,
    UnknownOrInvalidatedLease,
    AudienceOrGrantEpochMismatch,
    ExpiredLease,
    AlreadyResolved,
    InvalidLease,
    Capacity,
    Randomness,
}

#[cfg(test)]
mod tests {
    use makosh_vault_protocol::{
        LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1,
        VaultPurposeRequestV1,
    };

    use super::{LeaseError, LeaseManager};

    fn audience() -> LeaseAudienceV1 {
        LeaseAudienceV1::new("module".to_owned(), "runtime".to_owned(), 1, 1)
            .expect("test audience")
    }

    fn request() -> VaultLeaseIssueRequestV1 {
        VaultLeaseIssueRequestV1::new(
            "vault-runtime".to_owned(),
            1,
            1,
            "owner".to_owned(),
            VaultPurposeRequestV1::new(
                "purpose".to_owned(),
                "instance".to_owned(),
                vec![SecretClassV1::OwnerDerivedKey],
                vec![VaultActionV1::IssueOwnerDerivedKey],
                60,
            )
            .expect("test purpose"),
            audience(),
        )
        .expect("test request")
    }

    #[test]
    fn consuming_a_one_shot_lease_releases_its_bounded_slot() {
        let mut manager =
            LeaseManager::new("vault-runtime".to_owned(), 1).expect("test lease manager");
        let lease = manager.issue(request(), 1).expect("issue lease");

        manager
            .consume_once(lease.lease_id(), &audience(), 1)
            .expect("consume lease");

        assert_eq!(
            manager.consume_once(lease.lease_id(), &audience(), 1),
            Err(LeaseError::UnknownOrInvalidatedLease),
        );
        manager.issue(request(), 1).expect("reuse released slot");
    }
}

//! Exact write-only owner provisioning input for the SQLCipher actor.

use makosh_vault_protocol::{VaultActionV1, VaultProvisioningReceiptV1};
use zeroize::Zeroizing;

use crate::SecretRecordScope;

pub struct VaultProvisioningMutationV1 {
    operation_id: [u8; 16],
    action: VaultActionV1,
    scope: SecretRecordScope,
    payload: Zeroizing<Vec<u8>>,
    changed_at_unix_seconds: u64,
}

impl VaultProvisioningMutationV1 {
    pub fn new(
        operation_id: [u8; 16],
        action: VaultActionV1,
        scope: SecretRecordScope,
        payload: Vec<u8>,
        changed_at_unix_seconds: u64,
    ) -> Result<Self, VaultProvisioningMutationError> {
        let payload_required = matches!(action, VaultActionV1::Create | VaultActionV1::ReplaceCas);
        if operation_id == [0; 16]
            || changed_at_unix_seconds == 0
            || !matches!(
                action,
                VaultActionV1::Create
                    | VaultActionV1::ReplaceCas
                    | VaultActionV1::Retire
                    | VaultActionV1::Delete
            )
            || payload_required == payload.is_empty()
        {
            return Err(VaultProvisioningMutationError::Invalid);
        }
        Ok(Self {
            operation_id,
            action,
            scope,
            payload: Zeroizing::new(payload),
            changed_at_unix_seconds,
        })
    }

    pub(crate) const fn operation_id(&self) -> &[u8; 16] {
        &self.operation_id
    }

    pub(crate) const fn action(&self) -> VaultActionV1 {
        self.action
    }

    pub(crate) const fn scope(&self) -> &SecretRecordScope {
        &self.scope
    }

    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub(crate) const fn changed_at_unix_seconds(&self) -> u64 {
        self.changed_at_unix_seconds
    }
}

pub type VaultProvisioningMutationReceiptV1 = VaultProvisioningReceiptV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultProvisioningMutationError {
    Invalid,
}

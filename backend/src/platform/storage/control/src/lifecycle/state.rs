//! In-memory lifecycle for one owner runtime's fenced storage binding.

use makosh_storage_protocol::StorageBindingV1;

use super::StorageLifecycleErrorV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageLifecycleStateV1 {
    Idle,
    Active,
    Revoking,
}

#[derive(Default)]
pub struct StorageLifecycleV1 {
    binding: Option<StorageBindingV1>,
    state: LifecycleState,
}

#[derive(Default)]
enum LifecycleState {
    #[default]
    Idle,
    Active,
    Revoking,
}

impl StorageLifecycleV1 {
    pub fn activate(&mut self, binding: StorageBindingV1) -> Result<(), StorageLifecycleErrorV1> {
        match self.state {
            LifecycleState::Idle => self.activate_initial(binding),
            LifecycleState::Active => Err(StorageLifecycleErrorV1::RotationRequiresRevocation),
            LifecycleState::Revoking => Err(StorageLifecycleErrorV1::RevocationInProgress),
        }
    }

    pub fn begin_revocation(&mut self) -> Result<&StorageBindingV1, StorageLifecycleErrorV1> {
        if !matches!(self.state, LifecycleState::Active) {
            return Err(StorageLifecycleErrorV1::NotRevoking);
        }
        self.state = LifecycleState::Revoking;
        self.binding
            .as_ref()
            .ok_or(StorageLifecycleErrorV1::NotRevoking)
    }

    pub fn complete_revocation(&mut self) -> Result<(), StorageLifecycleErrorV1> {
        if !matches!(self.state, LifecycleState::Revoking) {
            return Err(StorageLifecycleErrorV1::NotRevoking);
        }
        self.binding = None;
        self.state = LifecycleState::Idle;
        Ok(())
    }

    pub fn state(&self) -> StorageLifecycleStateV1 {
        match self.state {
            LifecycleState::Idle => StorageLifecycleStateV1::Idle,
            LifecycleState::Active => StorageLifecycleStateV1::Active,
            LifecycleState::Revoking => StorageLifecycleStateV1::Revoking,
        }
    }

    pub fn active_binding(&self) -> Option<&StorageBindingV1> {
        matches!(self.state, LifecycleState::Active)
            .then(|| self.binding.as_ref())
            .flatten()
    }

    fn activate_initial(
        &mut self,
        binding: StorageBindingV1,
    ) -> Result<(), StorageLifecycleErrorV1> {
        if self.binding.is_some() {
            return Err(StorageLifecycleErrorV1::RotationRequiresRevocation);
        }
        self.binding = Some(binding);
        self.state = LifecycleState::Active;
        Ok(())
    }
}

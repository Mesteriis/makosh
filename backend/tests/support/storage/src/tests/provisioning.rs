use makosh_storage_control::{
    StorageLifecycleStateV1, StorageLifecycleV1, StorageProvisionerV1, StorageProvisioningErrorV1,
    StorageProvisioningPortV1,
};
use makosh_storage_protocol::StorageBindingV1;

use super::fixtures::storage_role_binding;

#[test]
fn provisions_in_required_order_before_publishing_the_pool() {
    let mut port = RecordingProvisioner::default();
    let mut lifecycle = StorageLifecycleV1::default();

    StorageProvisionerV1
        .provision(
            &mut port,
            &mut lifecycle,
            &storage_role_binding("notes", "runtime_notes"),
        )
        .expect("complete provisioning");

    assert_eq!(port.events, ["role", "migrations", "lease", "pool"]);
    assert_eq!(lifecycle.state(), StorageLifecycleStateV1::Active);
}

#[test]
fn stops_before_pool_publication_when_an_earlier_stage_fails() {
    let mut port = RecordingProvisioner::failing_at("lease");
    let mut lifecycle = StorageLifecycleV1::default();

    let error = StorageProvisionerV1.provision(
        &mut port,
        &mut lifecycle,
        &storage_role_binding("notes", "runtime_notes"),
    );

    assert!(
        matches!(error, Err(failure) if failure.error() == StorageProvisioningErrorV1::Lease && failure.partial_fence_applied())
    );
    assert_eq!(port.events, ["role", "migrations", "lease", "fence"]);
    assert_eq!(lifecycle.state(), StorageLifecycleStateV1::Idle);
}

#[derive(Default)]
struct RecordingProvisioner {
    events: Vec<&'static str>,
    failure: Option<&'static str>,
}

impl RecordingProvisioner {
    fn failing_at(stage: &'static str) -> Self {
        Self {
            events: Vec::new(),
            failure: Some(stage),
        }
    }

    fn record(
        &mut self,
        stage: &'static str,
        error: StorageProvisioningErrorV1,
    ) -> Result<(), StorageProvisioningErrorV1> {
        self.events.push(stage);
        (self.failure != Some(stage)).then_some(()).ok_or(error)
    }
}

impl StorageProvisioningPortV1 for RecordingProvisioner {
    fn ensure_role(&mut self, _: &StorageBindingV1) -> Result<(), StorageProvisioningErrorV1> {
        self.record("role", StorageProvisioningErrorV1::Role)
    }

    fn apply_migrations_and_privileges(
        &mut self,
        _: &StorageBindingV1,
    ) -> Result<(), StorageProvisioningErrorV1> {
        self.record("migrations", StorageProvisioningErrorV1::Migration)
    }

    fn issue_credential_lease(
        &mut self,
        _: &StorageBindingV1,
    ) -> Result<(), StorageProvisioningErrorV1> {
        self.record("lease", StorageProvisioningErrorV1::Lease)
    }

    fn publish_pool(&mut self, _: &StorageBindingV1) -> Result<(), StorageProvisioningErrorV1> {
        self.record("pool", StorageProvisioningErrorV1::Pool)
    }

    fn fence_partial_binding(&mut self, _: &StorageBindingV1) -> bool {
        self.events.push("fence");
        true
    }
}

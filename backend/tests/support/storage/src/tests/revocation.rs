use makosh_storage_control::{
    StorageFenceOutcomeV1, StorageLifecycleStateV1, StorageLifecycleV1, StoragePoolFenceCommandV1,
    StoragePoolFencePortV1, StoragePostgresFencePortV1, StorageRevocationErrorV1, StorageRevokerV1,
    StorageVaultLeasePortV1,
};
use makosh_storage_protocol::StorageBindingV1;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

use super::fixtures::storage_role_binding;

#[test]
fn completes_a_revoke_only_after_every_boundary_is_fenced() {
    let mut lifecycle = active_lifecycle();
    let events = recorded_events();
    let mut vault = RecordingVault::new(events.clone(), StorageFenceOutcomeV1::Applied);
    let mut pool = RecordingPool::new(events.clone(), StorageFenceOutcomeV1::Applied);
    let mut postgres = RecordingPostgres::new(events.clone(), StorageFenceOutcomeV1::Applied);

    let report = revoke(&mut lifecycle, &mut vault, &mut pool, &mut postgres)
        .expect("every infrastructure boundary is fenced");

    assert!(report.is_complete());
    assert_eq!(lifecycle.state(), StorageLifecycleStateV1::Idle);
    assert_eq!(
        *events.borrow(),
        [
            "vault",
            "pool:pause",
            "pool:disable",
            "pool:kill",
            "postgres"
        ]
    );
}

#[test]
fn keeps_the_binding_in_revoking_when_a_boundary_is_unavailable() {
    let mut lifecycle = active_lifecycle();
    let events = recorded_events();
    let mut vault = RecordingVault::new(events.clone(), StorageFenceOutcomeV1::Unavailable);
    let mut pool = RecordingPool::new(events.clone(), StorageFenceOutcomeV1::Applied);
    let mut postgres = RecordingPostgres::new(events.clone(), StorageFenceOutcomeV1::Applied);

    let error = revoke(&mut lifecycle, &mut vault, &mut pool, &mut postgres)
        .expect_err("unavailable vault denies completion");

    assert!(matches!(error, StorageRevocationErrorV1::Incomplete(report) if !report.is_complete()));
    assert_eq!(lifecycle.state(), StorageLifecycleStateV1::Revoking);
    assert_eq!(
        *events.borrow(),
        [
            "vault",
            "pool:pause",
            "pool:disable",
            "pool:kill",
            "postgres"
        ]
    );
}

fn recorded_events() -> Rc<RefCell<Vec<&'static str>>> {
    Rc::new(RefCell::new(Vec::new()))
}

fn revoke(
    lifecycle: &mut StorageLifecycleV1,
    vault: &mut RecordingVault,
    pool: &mut RecordingPool,
    postgres: &mut RecordingPostgres,
) -> Result<makosh_storage_control::StorageRevocationReportV1, StorageRevocationErrorV1> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("test runtime")
        .block_on(StorageRevokerV1.revoke(lifecycle, vault, pool, postgres))
}

fn active_lifecycle() -> StorageLifecycleV1 {
    let mut lifecycle = StorageLifecycleV1::default();
    lifecycle
        .activate(storage_role_binding("notes", "runtime_notes"))
        .expect("valid initial binding");
    lifecycle
}

struct RecordingVault {
    events: Rc<RefCell<Vec<&'static str>>>,
    outcome: StorageFenceOutcomeV1,
}

impl RecordingVault {
    fn new(events: Rc<RefCell<Vec<&'static str>>>, outcome: StorageFenceOutcomeV1) -> Self {
        Self { events, outcome }
    }
}

impl StorageVaultLeasePortV1 for RecordingVault {
    fn invalidate_lease(
        &mut self,
        _: &StorageBindingV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send {
        self.events.borrow_mut().push("vault");
        let outcome = self.outcome;
        async move { outcome }
    }
}

struct RecordingPool {
    events: Rc<RefCell<Vec<&'static str>>>,
    outcome: StorageFenceOutcomeV1,
}

impl RecordingPool {
    fn new(events: Rc<RefCell<Vec<&'static str>>>, outcome: StorageFenceOutcomeV1) -> Self {
        Self { events, outcome }
    }
}

impl StoragePoolFencePortV1 for RecordingPool {
    fn apply_pool_fence(
        &mut self,
        _: &StorageBindingV1,
        command: StoragePoolFenceCommandV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send {
        self.events.borrow_mut().push(match command {
            StoragePoolFenceCommandV1::Pause => "pool:pause",
            StoragePoolFenceCommandV1::Disable => "pool:disable",
            StoragePoolFenceCommandV1::Kill => "pool:kill",
        });
        let outcome = self.outcome;
        async move { outcome }
    }
}

struct RecordingPostgres {
    events: Rc<RefCell<Vec<&'static str>>>,
    outcome: StorageFenceOutcomeV1,
}

impl RecordingPostgres {
    fn new(events: Rc<RefCell<Vec<&'static str>>>, outcome: StorageFenceOutcomeV1) -> Self {
        Self { events, outcome }
    }
}

impl StoragePostgresFencePortV1 for RecordingPostgres {
    fn fence_runtime_role(
        &mut self,
        _: &StorageBindingV1,
    ) -> impl Future<Output = StorageFenceOutcomeV1> + Send {
        self.events.borrow_mut().push("postgres");
        let outcome = self.outcome;
        async move { outcome }
    }
}

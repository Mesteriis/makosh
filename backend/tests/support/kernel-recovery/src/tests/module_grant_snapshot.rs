use std::sync::{Arc, Barrier};

use makosh_kernel_control_store::{
    ModuleGrantSnapshot, ModuleRegistration, ModuleRegistrationState,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

#[test]
fn concurrent_registration_and_grant_reads_are_one_atomic_snapshot() {
    let path = fixture_path();
    let store = Arc::new(
        SqliteControlStore::create(&path, "instance-atomic-snapshot", 1)
            .expect("create control store"),
    );
    let registration = ModuleRegistration::new(
        "registration-atomic",
        "module-atomic",
        "owner-atomic",
        [3; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(
            &registration,
            &["capability.read".to_owned(), "capability.write".to_owned()],
        )
        .expect("create registration");
    store
        .approve_module_registration("registration-atomic", &["capability.read".to_owned()])
        .expect("approve initial grants");

    let start = Arc::new(Barrier::new(2));
    let writer = spawn_grant_writer(Arc::clone(&store), Arc::clone(&start));
    start.wait();
    for _ in 0..100 {
        let snapshot = store
            .module_grant_snapshot("registration-atomic")
            .expect("read atomic snapshot")
            .expect("registration exists");
        assert_consistent(&snapshot);
    }
    writer.join().expect("writer thread");
    drop(store);
    std::fs::remove_file(path).expect("remove control store");
}

fn spawn_grant_writer(
    store: Arc<SqliteControlStore>,
    start: Arc<Barrier>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        start.wait();
        for revision in 0..20 {
            store
                .transition_module_registration(
                    "registration-atomic",
                    ModuleRegistrationState::Suspended,
                )
                .expect("suspend registration");
            let capability = if revision % 2 == 0 {
                "capability.write"
            } else {
                "capability.read"
            };
            store
                .approve_module_registration("registration-atomic", &[capability.to_owned()])
                .expect("replace grants");
        }
    })
}

fn assert_consistent(snapshot: &ModuleGrantSnapshot) {
    let registration = snapshot.registration();
    match registration.state() {
        ModuleRegistrationState::Approved => {
            let grants = snapshot.effective_grants().expect("approved grants");
            assert_eq!(grants.registration_id(), registration.registration_id());
            assert_eq!(grants.grant_epoch(), registration.grant_epoch());
            assert!(matches!(
                grants.capability_ids(),
                [capability] if capability == "capability.read" || capability == "capability.write"
            ));
        }
        ModuleRegistrationState::Suspended => assert!(snapshot.effective_grants().is_none()),
        unexpected => panic!("unexpected registration state: {unexpected:?}"),
    }
}

fn fixture_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "makosh-module-grant-snapshot-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

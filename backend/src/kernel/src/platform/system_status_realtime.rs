use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::identity::browser_gateway::system_status::client_system_status;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const RECONCILE_INTERVAL: Duration = Duration::from_secs(1);

pub(crate) fn run(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    realtime: &InMemoryBrowserRealtimeSource,
    shutdown_requested: &AtomicBool,
) -> Result<(), String> {
    while !shutdown_requested.load(Ordering::Acquire) {
        reconcile(store, supervisor, realtime)?;
        std::thread::sleep(RECONCILE_INTERVAL);
    }
    Ok(())
}

pub(crate) fn reconcile(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    realtime: &InMemoryBrowserRealtimeSource,
) -> Result<usize, String> {
    let owners = realtime.admitted_owner_ids()?;
    if owners.is_empty() {
        return Ok(0);
    }
    let statuses = client_system_status(store, supervisor, true);
    let occurred_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system status realtime clock is unavailable".to_owned())?
        .as_millis()
        .try_into()
        .map_err(|_| "system status realtime clock is invalid".to_owned())?;
    let mut published = 0;
    for owner_id in owners {
        match realtime.reconcile_system_status(&owner_id, &statuses, occurred_at) {
            Ok(changed) => published += usize::from(changed),
            Err(error) if error == "Gateway realtime owner is not admitted" => {}
            Err(error) => return Err(error),
        }
    }
    Ok(published)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::time::{SystemTime, UNIX_EPOCH};

    use makosh_gateway_runtime::InMemoryBrowserRealtimeSource;
    use makosh_kernel_control_store_sqlite::SqliteControlStore;

    use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

    use super::reconcile;

    #[test]
    fn admitted_owner_receives_one_initial_snapshot_and_no_unchanged_duplicate() {
        let root = std::env::temp_dir().join(format!(
            "makosh-system-status-realtime-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("test directory");
        let store = SqliteControlStore::create(&root.join("control.sqlite"), "test-instance", 1)
            .expect("control store");
        let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));
        let realtime = InMemoryBrowserRealtimeSource::new(4).expect("realtime source");
        realtime.admit_owner("owner-1").expect("admitted owner");

        assert_eq!(
            reconcile(&store, &supervisor, &realtime).expect("initial reconcile"),
            1
        );
        assert_eq!(
            reconcile(&store, &supervisor, &realtime).expect("unchanged reconcile"),
            0
        );

        std::fs::remove_dir_all(root).expect("remove test directory");
    }
}

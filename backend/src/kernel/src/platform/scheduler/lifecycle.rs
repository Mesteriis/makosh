//! Fail-closed reconciliation of an already admitted Scheduler runtime.
//!
//! An active Scheduler Storage binding is the durable desired-running record:
//! it is created only by the owner-authorized launch flow. A revoking binding
//! is deliberately excluded, so this worker never turns an intentional stop
//! into an automatic restart.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use makosh_kernel_control_store::{PlatformStorageBindingStateV1, PlatformStorageBindingV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::launch;
use crate::platform::storage::successor;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const CHANGE_POLL_INTERVAL: Duration = Duration::from_millis(250);
const STABLE_POLL_INTERVAL: Duration = Duration::from_secs(1);
const TOPOLOGY_STABLE_OBSERVATIONS: u8 = 8;

enum ReconcileOutcome {
    Idle,
    Stable,
    ObservingTopology,
    Started,
    Refreshed,
}

/// Runs for the lifetime of the Kernel control plane. Reconciliation never
/// starts an unbound Scheduler and never retries forever after repeated launch
/// failures; an owner-authorized start/restart must re-establish a healthy
/// runtime before automatic crash recovery resumes.
pub(crate) fn serve(
    store: Arc<SqliteControlStore>,
    kernel: &Path,
    runtime_dir: &Path,
    shutdown_requested: Arc<AtomicBool>,
    supervisor: ManagedRuntimeSupervisor,
    initial_topology_fingerprint: Option<[u8; 32]>,
) -> Result<(), String> {
    let mut blocked = false;
    let mut active_topology_fingerprint = initial_topology_fingerprint;
    let mut pending_topology_fingerprint = None;
    let mut pending_topology_observations = 0;
    while !shutdown_requested.load(Ordering::Acquire) {
        if blocked {
            if scheduler_is_active(&store, &supervisor)? {
                blocked = false;
            }
            wait_for_poll(&shutdown_requested, STABLE_POLL_INTERVAL);
            continue;
        }
        let poll_interval = match reconcile_once(
            &store,
            kernel,
            runtime_dir,
            &supervisor,
            &mut active_topology_fingerprint,
            &mut pending_topology_fingerprint,
            &mut pending_topology_observations,
        ) {
            Ok(ReconcileOutcome::Refreshed) => {
                tracing::debug!(event = "scheduler.topology.refreshed");
                CHANGE_POLL_INTERVAL
            }
            Ok(ReconcileOutcome::ObservingTopology | ReconcileOutcome::Started) => {
                CHANGE_POLL_INTERVAL
            }
            Ok(ReconcileOutcome::Idle | ReconcileOutcome::Stable) => STABLE_POLL_INTERVAL,
            Err(error) => {
                if let Some(registration_id) = active_scheduler_binding(&store)
                    .ok()
                    .flatten()
                    .map(|binding| binding.registration_id().to_owned())
                {
                    let _ = supervisor.record_failure(&registration_id, error);
                }
                // A failed reconcile may already have revoked the predecessor and
                // reserved a successor. Retrying would fence that successor again
                // and erase the failure evidence, so require owner intervention.
                blocked = true;
                STABLE_POLL_INTERVAL
            }
        };
        wait_for_poll(&shutdown_requested, poll_interval);
    }
    Ok(())
}

fn wait_for_poll(shutdown_requested: &AtomicBool, interval: Duration) {
    let deadline = std::time::Instant::now() + interval;
    while !shutdown_requested.load(Ordering::Acquire) && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn reconcile_once(
    store: &SqliteControlStore,
    kernel: &Path,
    runtime_dir: &Path,
    supervisor: &ManagedRuntimeSupervisor,
    active_topology_fingerprint: &mut Option<[u8; 32]>,
    pending_topology_fingerprint: &mut Option<[u8; 32]>,
    pending_topology_observations: &mut u8,
) -> Result<ReconcileOutcome, String> {
    let Some(binding) = active_scheduler_binding(store)? else {
        *active_topology_fingerprint = None;
        clear_pending_topology(pending_topology_fingerprint, pending_topology_observations);
        return Ok(ReconcileOutcome::Idle);
    };
    if supervisor.is_active(binding.registration_id())? {
        let expected_topology_fingerprint =
            launch::topology_fingerprint(store, binding.registration_id(), binding.grant_epoch())?;
        if active_topology_fingerprint
            .as_ref()
            .is_some_and(|current| current == &expected_topology_fingerprint)
        {
            clear_pending_topology(pending_topology_fingerprint, pending_topology_observations);
            return Ok(ReconcileOutcome::Stable);
        }
        if active_topology_fingerprint.is_none() {
            *active_topology_fingerprint = Some(expected_topology_fingerprint);
            clear_pending_topology(pending_topology_fingerprint, pending_topology_observations);
            return Ok(ReconcileOutcome::Stable);
        }
        if !observe_stable_topology(
            expected_topology_fingerprint,
            pending_topology_fingerprint,
            pending_topology_observations,
        ) {
            return Ok(ReconcileOutcome::ObservingTopology);
        }
        let issue = successor::issue_after(&binding)?;
        let (reservation, successor) = successor::reserve(
            supervisor,
            store,
            binding.registration_id(),
            binding.capability_id(),
            issue,
        )?;
        launch::start_from_reservation(
            supervisor,
            store,
            kernel,
            runtime_dir,
            reservation,
            &successor,
        )?;
        *active_topology_fingerprint = Some(expected_topology_fingerprint);
        clear_pending_topology(pending_topology_fingerprint, pending_topology_observations);
        return Ok(ReconcileOutcome::Refreshed);
    }
    clear_pending_topology(pending_topology_fingerprint, pending_topology_observations);
    let issue = successor::issue_after(&binding)?;
    let (reservation, successor) = successor::reserve(
        supervisor,
        store,
        binding.registration_id(),
        binding.capability_id(),
        issue,
    )?;
    launch::start_from_reservation(
        supervisor,
        store,
        kernel,
        runtime_dir,
        reservation,
        &successor,
    )?;
    *active_topology_fingerprint = Some(launch::topology_fingerprint(
        store,
        successor.registration_id(),
        successor.grant_epoch(),
    )?);
    Ok(ReconcileOutcome::Started)
}

fn observe_stable_topology(
    fingerprint: [u8; 32],
    pending_fingerprint: &mut Option<[u8; 32]>,
    observations: &mut u8,
) -> bool {
    if pending_fingerprint.as_ref() == Some(&fingerprint) {
        *observations = observations.saturating_add(1);
    } else {
        *pending_fingerprint = Some(fingerprint);
        *observations = 1;
    }
    *observations >= TOPOLOGY_STABLE_OBSERVATIONS
}

fn clear_pending_topology(pending_fingerprint: &mut Option<[u8; 32]>, observations: &mut u8) {
    *pending_fingerprint = None;
    *observations = 0;
}

pub(crate) fn capture_active_topology_fingerprint(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) -> Result<Option<[u8; 32]>, String> {
    let Some(binding) = active_scheduler_binding(store)? else {
        return Ok(None);
    };
    if !supervisor.is_active(binding.registration_id())? {
        return Ok(None);
    }
    launch::topology_fingerprint(store, binding.registration_id(), binding.grant_epoch()).map(Some)
}

fn scheduler_is_active(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) -> Result<bool, String> {
    active_scheduler_binding(store)?
        .map(|binding| supervisor.is_active(binding.registration_id()))
        .transpose()
        .map(|active| active.unwrap_or(false))
}

pub(crate) fn active_scheduler_binding(
    store: &SqliteControlStore,
) -> Result<Option<PlatformStorageBindingV1>, String> {
    let mut bindings = Vec::new();
    for snapshot in store
        .approved_module_grant_snapshots()
        .map_err(|_| "Scheduler lifecycle registrations are unavailable".to_owned())?
    {
        if snapshot.registration().module_id() != "scheduler" {
            continue;
        }
        let Some(grants) = snapshot.effective_grants() else {
            continue;
        };
        for capability_id in grants.capability_ids() {
            let binding = store
                .platform_storage_binding(snapshot.registration().registration_id(), capability_id)
                .map_err(|_| "Scheduler lifecycle Storage binding is unavailable".to_owned())?;
            if binding
                .as_ref()
                .is_some_and(|value| value.state() == PlatformStorageBindingStateV1::Active)
            {
                bindings.push(binding.expect("binding was checked as present"));
            }
        }
    }
    match bindings.len() {
        0 => Ok(None),
        1 => Ok(bindings.pop()),
        _ => Err("Scheduler lifecycle has multiple active Storage bindings".to_owned()),
    }
}

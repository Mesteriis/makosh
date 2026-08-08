//! Resolves Event Hub route entries from approved Control Store state.

use makosh_kernel_control_store::{
    ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1, ModuleEventRouteRequestInputV1,
    ModuleEventRouteRequestV1, ModuleGrantSnapshot, ModuleRegistryStore,
};
use makosh_kernel_control_store_sqlite::StoreError;

const SCHEDULER_MODULE_ID_V1: &str = "scheduler";
const SCHEDULER_OWNER_ID_V1: &str = "scheduler";
const SCHEDULER_DISPATCH_CAPABILITY_ID_V1: &str = "events.scheduler.dispatch";
const SCHEDULER_DISPATCH_MAX_IN_FLIGHT_V1: u16 = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventCatalogEntryV1 {
    registration_id: String,
    module_id: String,
    grant_epoch: u64,
    capability_id: String,
    route: ModuleEventRouteRequestV1,
}

impl EventCatalogEntryV1 {
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn module_id(&self) -> &str {
        &self.module_id
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }

    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    #[must_use]
    pub fn route(&self) -> &ModuleEventRouteRequestV1 {
        &self.route
    }
}

pub fn resolve<S>(store: &S) -> Result<Vec<EventCatalogEntryV1>, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    let snapshots = store
        .approved_module_grant_snapshots()
        .map_err(|error| format!("{error:?}"))?;
    let mut entries = snapshots
        .iter()
        .map(|snapshot| resolve_snapshot(store, snapshot))
        .collect::<Result<Vec<_>, _>>()
        .map(|entries| entries.into_iter().flatten().collect::<Vec<_>>())?;
    entries.extend(scheduler_dispatch_entries(store, &snapshots, &entries)?);
    Ok(entries)
}

fn resolve_snapshot<S>(
    store: &S,
    snapshot: &ModuleGrantSnapshot,
) -> Result<Vec<EventCatalogEntryV1>, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    let registration = snapshot.registration();
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "approved module snapshot lacks effective grants".to_owned())?;
    let mut entries = Vec::new();
    for capability_id in grants.capability_ids() {
        let routes = store
            .module_event_route_requests(registration.registration_id(), capability_id)
            .map_err(|error| format!("{error:?}"))?;
        entries.extend(routes.into_iter().map(|route| EventCatalogEntryV1 {
            registration_id: registration.registration_id().to_owned(),
            module_id: registration.module_id().to_owned(),
            grant_epoch: grants.grant_epoch(),
            capability_id: capability_id.clone(),
            route,
        }));
    }
    Ok(entries)
}

fn scheduler_dispatch_entries<S>(
    store: &S,
    snapshots: &[ModuleGrantSnapshot],
    existing: &[EventCatalogEntryV1],
) -> Result<Vec<EventCatalogEntryV1>, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    let mut schedulers = snapshots.iter().filter(|snapshot| {
        let registration = snapshot.registration();
        registration.module_id() == SCHEDULER_MODULE_ID_V1
            && registration.owner_id() == SCHEDULER_OWNER_ID_V1
            && snapshot.effective_grants().is_some_and(|grants| {
                grants
                    .capability_ids()
                    .binary_search_by(|candidate| {
                        candidate.as_str().cmp(SCHEDULER_DISPATCH_CAPABILITY_ID_V1)
                    })
                    .is_ok()
            })
    });
    let Some(scheduler) = schedulers.next() else {
        return Ok(Vec::new());
    };
    if schedulers.next().is_some() {
        return Err("approved Scheduler dispatch authority is ambiguous".to_owned());
    }
    let scheduler_grants = scheduler
        .effective_grants()
        .ok_or_else(|| "approved Scheduler lacks effective grants".to_owned())?;
    let scheduler_registration = scheduler.registration();
    let mut derived = Vec::new();
    for snapshot in snapshots {
        let Some(grants) = snapshot.effective_grants() else {
            continue;
        };
        for capability_id in grants.capability_ids() {
            let requests = store
                .module_scheduler_job_requests(
                    snapshot.registration().registration_id(),
                    capability_id,
                )
                .map_err(|error| format!("{error:?}"))?;
            for request in requests {
                let route = ModuleEventRouteRequestV1::new(ModuleEventRouteRequestInputV1 {
                    registration_id: scheduler_registration.registration_id().to_owned(),
                    capability_id: SCHEDULER_DISPATCH_CAPABILITY_ID_V1.to_owned(),
                    envelope_kind: ModuleEventEnvelopeKindV1::Command,
                    contract_owner: request.owner().to_owned(),
                    contract_name: request.name().to_owned(),
                    contract_major: request.major(),
                    contract_revision: request.revision(),
                    contract_schema_sha256: *request.schema_sha256(),
                    direction: ModuleEventRouteDirectionV1::Publish,
                    max_in_flight: SCHEDULER_DISPATCH_MAX_IN_FLIGHT_V1,
                    delivery_policy: None,
                });
                let duplicate = existing.iter().chain(derived.iter()).any(|entry| {
                    entry.registration_id == scheduler_registration.registration_id()
                        && entry.capability_id == SCHEDULER_DISPATCH_CAPABILITY_ID_V1
                        && entry.route == route
                });
                if !duplicate {
                    derived.push(EventCatalogEntryV1 {
                        registration_id: scheduler_registration.registration_id().to_owned(),
                        module_id: scheduler_registration.module_id().to_owned(),
                        grant_epoch: scheduler_grants.grant_epoch(),
                        capability_id: SCHEDULER_DISPATCH_CAPABILITY_ID_V1.to_owned(),
                        route,
                    });
                }
            }
        }
    }
    Ok(derived)
}

#[cfg(test)]
mod tests {
    use makosh_kernel_control_store::{
        ModuleDescriptorRegistrationRequestsV1, ModuleEventDeliveryPolicyV1,
        ModuleEventSubscriptionRequirementV1, ModuleRegistration, ModuleRegistrationState,
        ModuleSchedulerJobRequestV1,
    };
    use makosh_kernel_control_store_sqlite::SqliteControlStore;

    use super::*;

    #[test]
    fn derives_exact_scheduler_publisher_from_approved_job_kind() {
        let root = fixture_root("derived-dispatch");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("fixture directory");
        let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("control store");
        register_scheduler(&store);
        register_job_owner(&store);

        let entries = resolve(&store).expect("event catalog entries");
        let publishers = entries
            .iter()
            .filter(|entry| {
                entry.registration_id() == "scheduler_registration"
                    && entry.capability_id() == SCHEDULER_DISPATCH_CAPABILITY_ID_V1
                    && entry.route().direction() == ModuleEventRouteDirectionV1::Publish
            })
            .collect::<Vec<_>>();
        assert_eq!(publishers.len(), 1);
        assert_eq!(
            publishers[0].route().contract_owner(),
            "communication_delayed_delivery"
        );
        assert_eq!(publishers[0].route().contract_name(), "execute");
        assert_eq!(publishers[0].route().contract_schema_sha256(), &[9; 32]);
        std::fs::remove_dir_all(root).expect("remove fixture");
    }

    fn fixture_root(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "makosh-event-catalog-{}-{label}",
            std::process::id()
        ))
    }

    fn register_scheduler(store: &SqliteControlStore) {
        let registration = ModuleRegistration::new(
            "scheduler_registration",
            SCHEDULER_MODULE_ID_V1,
            SCHEDULER_OWNER_ID_V1,
            [1; 32],
            ModuleRegistrationState::Pending,
            1,
        );
        let capabilities = vec![SCHEDULER_DISPATCH_CAPABILITY_ID_V1.to_owned()];
        store
            .create_pending_registration(&registration, &capabilities)
            .expect("scheduler registration");
        store
            .approve_module_registration("scheduler_registration", &capabilities)
            .expect("scheduler approval");
    }

    fn register_job_owner(store: &SqliteControlStore) {
        let registration = ModuleRegistration::new(
            "delayed_delivery_registration",
            "delayed_delivery",
            "communication_delayed_delivery",
            [2; 32],
            ModuleRegistrationState::Pending,
            1,
        );
        let capabilities = vec!["scheduler_due".to_owned()];
        let routes = [ModuleEventRouteRequestV1::new(
            ModuleEventRouteRequestInputV1 {
                registration_id: registration.registration_id().to_owned(),
                capability_id: capabilities[0].clone(),
                envelope_kind: ModuleEventEnvelopeKindV1::Command,
                contract_owner: "communication_delayed_delivery".to_owned(),
                contract_name: "execute".to_owned(),
                contract_major: 1,
                contract_revision: 1,
                contract_schema_sha256: [9; 32],
                direction: ModuleEventRouteDirectionV1::Consume,
                max_in_flight: 8,
                delivery_policy: Some(ModuleEventDeliveryPolicyV1::new(
                    ModuleEventSubscriptionRequirementV1::Required,
                    3,
                    2_000,
                )),
            },
        )];
        let jobs = [ModuleSchedulerJobRequestV1::new(
            registration.registration_id(),
            &capabilities[0],
            "communication_delayed_delivery",
            "execute",
            1,
            1,
            [9; 32],
        )];
        store
            .create_pending_registration_with_descriptor_requests(
                &registration,
                &capabilities,
                ModuleDescriptorRegistrationRequestsV1 {
                    storage: &[],
                    events: &routes,
                    blobs: &[],
                    scheduler: &jobs,
                    vault_purposes: &[],
                    client_rpc_routes: &[],
                    client_blob_routes: &[],
                    client_realtime_routes: &[],
                    query_rpc_routes: &[],
                    request_rpc_routes: &[],
                    contract_dependencies: &[],
                },
            )
            .expect("job owner registration");
        store
            .approve_module_registration(registration.registration_id(), &capabilities)
            .expect("job owner approval");
    }
}

use makosh_kernel_control_store::{
    ModuleEventEnvelopeKindV1, PlatformEventHubTopologyV1, PlatformEventStreamBudgetV1,
};
use makosh_kernel_control_store_sqlite::{SqliteControlStore, StoreError};

use super::common::unique_target_root;

#[test]
fn event_hub_topology_is_durable_and_revision_fenced() {
    let root = unique_target_root("makosh-event-hub-topology");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");

    let first = topology(1, "nats://127.0.0.1:4222", "event_hub", 1, 1_048_576);
    store
        .record_platform_event_hub_topology(&first)
        .expect("record topology");
    assert_eq!(
        store.platform_event_hub_topology().expect("read topology"),
        Some(first.clone())
    );
    assert!(matches!(
        store.record_platform_event_hub_topology(&first),
        Err(StoreError::PlatformEventHubTopologyRevisionConflict)
    ));

    let replacement = topology(2, "nats://127.0.0.1:4222", "event_hub", 2, 2_097_152);
    store
        .record_platform_event_hub_topology(&replacement)
        .expect("replace topology");
    assert_eq!(
        store
            .platform_event_hub_topology()
            .expect("read replacement"),
        Some(replacement)
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn event_hub_topology_requires_each_canonical_stream_and_bounded_budget() {
    let root = unique_target_root("makosh-event-hub-topology-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");

    let source = topology(1, "nats://127.0.0.1:4222", "event_hub", 1, 1_048_576);
    let incomplete = PlatformEventHubTopologyV1::new(
        1,
        "nats://127.0.0.1:4222",
        "event_hub",
        1,
        source.stream_budgets()[..4].to_vec(),
    );
    assert!(matches!(
        store.record_platform_event_hub_topology(&incomplete),
        Err(StoreError::InvalidPlatformEventHubTopology)
    ));
    let invalid_budget = PlatformEventHubTopologyV1::new(
        1,
        "nats://127.0.0.1:4222",
        "event_hub",
        1,
        vec![
            PlatformEventStreamBudgetV1::new(ModuleEventEnvelopeKindV1::Command, 0, 3_600_000, 1,);
            5
        ],
    );
    assert!(matches!(
        store.record_platform_event_hub_topology(&invalid_budget),
        Err(StoreError::InvalidPlatformEventHubTopology)
    ));
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn event_hub_topology_rejects_secret_bearing_endpoint_and_invalid_credential_fence() {
    let root = unique_target_root("makosh-event-hub-topology-connection-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");

    let secret_bearing = topology(
        1,
        "nats://event_hub:secret@127.0.0.1:4222",
        "event_hub",
        1,
        1,
    );
    let stale_credential = topology(1, "nats://127.0.0.1:4222", "event_hub", 0, 1);
    let invalid_identity = topology(1, "nats://127.0.0.1:4222", "EventHub", 1, 1);

    for topology in [secret_bearing, stale_credential, invalid_identity] {
        assert!(matches!(
            store.record_platform_event_hub_topology(&topology),
            Err(StoreError::InvalidPlatformEventHubTopology)
        ));
    }
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn topology(
    revision: u64,
    nats_endpoint: &str,
    nats_username: &str,
    credential_revision: u64,
    max_bytes: u64,
) -> PlatformEventHubTopologyV1 {
    let kinds = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ];
    PlatformEventHubTopologyV1::new(
        revision,
        nats_endpoint,
        nats_username,
        credential_revision,
        kinds
            .into_iter()
            .map(|kind| PlatformEventStreamBudgetV1::new(kind, max_bytes, 3_600_000, 1))
            .collect(),
    )
}

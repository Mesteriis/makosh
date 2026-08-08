use crate::collector::storage::{TelemetryRetentionV1, TelemetrySegmentStore};
use crate::fixtures::directory::unique_directory;
use makosh_telemetry_protocol::{
    TelemetryPriorityV1, TelemetrySignalInputV1, TelemetrySignalKindV1, TelemetrySignalV1,
    TelemetrySourceV1,
};

#[test]
fn segment_store_rotates_and_prunes_to_the_configured_byte_budget() {
    let directory = unique_directory("retention");
    let retention = TelemetryRetentionV1::new(64, 128, 60).expect("valid retention");
    let store = TelemetrySegmentStore::open(directory.clone(), retention).expect("open store");
    for timestamp in 0..8 {
        store.append(&signal(timestamp)).expect("append signal");
    }
    let total = std::fs::read_dir(&directory)
        .expect("read segments")
        .filter_map(Result::ok)
        .map(|entry| entry.metadata().expect("segment metadata").len())
        .sum::<u64>();
    assert!(total <= 128, "retention must bound total segment bytes");
    std::fs::remove_dir_all(directory).expect("remove test directory");
}

#[test]
fn retention_policy_rejects_an_impossible_byte_budget() {
    assert!(TelemetryRetentionV1::new(129, 128, 60).is_err());
}

fn signal(timestamp: i32) -> TelemetrySignalV1 {
    TelemetrySignalV1::new(TelemetrySignalInputV1 {
        observed_at_utc_millis: i64::from(timestamp),
        source: TelemetrySourceV1::new("runtime-42".to_owned(), "module.lifecycle".to_owned())
            .expect("source"),
        kind: TelemetrySignalKindV1::Lifecycle,
        priority: TelemetryPriorityV1::Info,
        operation: "runtime.lifecycle.transition".to_owned(),
        error_class: None,
        trace_id: None,
        dropped_count: 0,
    })
    .expect("signal")
}

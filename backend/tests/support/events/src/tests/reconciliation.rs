use std::time::Duration;

use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, EventHubTopologyPlanV1, StreamBudgetV1, StreamKindV1,
    StreamSpecV1,
};

#[test]
fn topology_plan_rejects_duplicate_or_undeclared_consumers() {
    let stream = StreamSpecV1::new(
        StreamKindV1::Event,
        StreamBudgetV1::new(1_024, Duration::from_secs(60), 1).expect("stream budget"),
    );
    let consumer = ConsumerSpecV1::new(
        StreamKindV1::Event,
        "notes_projection",
        "makosh.event.v1.notes.changed.v1",
        ConsumerBudgetV1::new(1, 1, Duration::from_secs(1)).expect("consumer budget"),
    )
    .expect("consumer spec");

    assert!(EventHubTopologyPlanV1::new(vec![stream], vec![consumer.clone()]).is_ok());
    assert!(EventHubTopologyPlanV1::new(vec![], vec![consumer.clone()]).is_err());
    assert!(EventHubTopologyPlanV1::new(vec![stream, stream], vec![consumer]).is_err());
}

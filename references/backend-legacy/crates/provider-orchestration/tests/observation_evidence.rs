use chrono::{TimeZone, Utc};
use makosh_provider_api::{ProviderId, ProviderObservationEnvelope, ProviderObservationInput};
use makosh_provider_orchestration::observation_to_raw_communication_record;
use serde_json::json;

#[test]
fn observation_conversion_preserves_canonical_evidence_identity_and_provenance() {
    let observation = ProviderObservationEnvelope::try_from(ProviderObservationInput::new(
        "obs-1",
        ProviderId::parse("zulip").expect("provider id"),
        "account-1",
        "message",
        "message-1",
        "sha256:record",
        "batch-1",
        Utc.timestamp_opt(100, 0).single().expect("timestamp"),
        Utc.timestamp_opt(90, 0).single().expect("timestamp"),
        "queue:1",
        0,
        json!({"message": "metadata only"}),
        json!({"provider": "zulip"}),
    ))
    .expect("observation");

    let record = observation_to_raw_communication_record(observation);
    assert_eq!(record.raw_record_id, "obs-1");
    assert_eq!(record.account_id, "account-1");
    assert_eq!(record.occurred_at, Utc.timestamp_opt(90, 0).single());
    assert_eq!(record.provenance, json!({"provider": "zulip"}));
}

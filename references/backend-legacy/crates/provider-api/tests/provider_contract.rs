use chrono::{TimeZone, Utc};
use makosh_blob_api::BlobRef;
use makosh_provider_api::{
    CredentialLease, ProviderCommandEnvelope, ProviderCommandInput, ProviderId, ProviderManifest,
    ProviderObservationEnvelope, ProviderObservationInput, RuntimeTopology,
};
use serde_json::json;

#[test]
fn provider_manifest_requires_a_supported_runtime_topology() {
    let provider_id = ProviderId::parse("zulip").expect("provider id");
    let manifest = ProviderManifest::new(
        provider_id,
        1,
        ["messages.read", "messages.send"],
        [RuntimeTopology::InProcess, RuntimeTopology::SharedConnector],
    )
    .expect("manifest");

    assert!(manifest.supports(RuntimeTopology::InProcess));
    assert!(manifest.supports(RuntimeTopology::SharedConnector));
    assert!(!manifest.supports(RuntimeTopology::PerAccountConnector));
}

#[test]
fn provider_command_envelope_rejects_an_expired_deadline() {
    let provider_id = ProviderId::parse("zulip").expect("provider id");
    let deadline = Utc
        .timestamp_opt(1_700_000_000, 0)
        .single()
        .expect("deadline");

    let error = ProviderCommandEnvelope::try_from(ProviderCommandInput::new(
        "command-1",
        "idempotency-1",
        provider_id,
        "account-1",
        deadline,
        deadline,
        0,
        1,
        json!({"kind": "send_stream_message"}),
    ))
    .expect_err("expired command must be rejected");

    assert_eq!(error.code(), "expired_deadline");
}

#[test]
fn provider_observation_preserves_evidence_provenance() {
    let observed_at = Utc
        .timestamp_opt(1_700_000_001, 0)
        .single()
        .expect("observed at");
    let observation = ProviderObservationEnvelope::try_from(ProviderObservationInput::new(
        "observation-1",
        ProviderId::parse("zulip").expect("provider id"),
        "account-1",
        "zulip_message",
        "42",
        "sha256:fixture",
        "zulip-event-queue",
        observed_at,
        observed_at,
        "42",
        0,
        json!({"message": "fixture"}),
        json!({"provider_event_id": 42}),
    ))
    .expect("observation");

    assert_eq!(observation.provider_id.as_str(), "zulip");
    assert_eq!(observation.provider_cursor, "42");
    assert_eq!(observation.provenance["provider_event_id"], json!(42));
}

#[test]
fn leases_and_blob_references_redact_capabilities_in_debug_output() {
    let issued_at = Utc
        .timestamp_opt(1_700_000_000, 0)
        .single()
        .expect("issued at");
    let expires_at = Utc
        .timestamp_opt(1_700_000_060, 0)
        .single()
        .expect("expires at");
    let provider_id = ProviderId::parse("zulip").expect("provider id");
    let lease = CredentialLease::new(
        provider_id.as_str(),
        "account-1",
        "command_execution",
        7,
        issued_at,
        expires_at,
        b"private-fixture-secret",
    )
    .expect("credential lease");
    let blob = BlobRef::new("blob-1", "account-1", "private-blob-capability", expires_at)
        .expect("blob ref");

    assert!(!format!("{lease:?}").contains("private-fixture-secret"));
    assert!(!format!("{blob:?}").contains("private-blob-capability"));
    assert!(lease.is_expired_at(expires_at));
    assert!(blob.is_expired_at(expires_at));
}

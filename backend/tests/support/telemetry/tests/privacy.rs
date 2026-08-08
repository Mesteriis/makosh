use makosh_telemetry_protocol::{
    TelemetryIdentityErrorV1, TelemetryPriorityV1, TelemetrySignalErrorV1, TelemetrySignalInputV1,
    TelemetrySignalKindV1, TelemetrySignalV1, TelemetrySourceV1,
};

fn source() -> TelemetrySourceV1 {
    TelemetrySourceV1::new("runtime-42".into(), "module.lifecycle".into()).unwrap()
}

#[test]
fn signal_only_accepts_allowlisted_technical_fields() {
    let signal = TelemetrySignalV1::new(TelemetrySignalInputV1 {
        observed_at_utc_millis: 1_000,
        source: source(),
        kind: TelemetrySignalKindV1::Lifecycle,
        priority: TelemetryPriorityV1::Info,
        operation: "runtime.start".into(),
        error_class: None,
        trace_id: Some("trace-42".into()),
        dropped_count: 0,
    });
    assert!(signal.is_ok());
}

#[test]
fn source_identity_rejects_private_or_user_provided_text() {
    assert_eq!(
        TelemetrySourceV1::new("owner@example.test".into(), "module".into()),
        Err(TelemetryIdentityErrorV1::InvalidRuntimeId),
    );
}

#[test]
fn signal_rejects_private_content_in_every_text_field() {
    let invalid_operation = TelemetrySignalV1::new(TelemetrySignalInputV1 {
        observed_at_utc_millis: 1_000,
        source: source(),
        kind: TelemetrySignalKindV1::Log,
        priority: TelemetryPriorityV1::Error,
        operation: "mail subject: private".into(),
        error_class: None,
        trace_id: None,
        dropped_count: 0,
    });
    assert_eq!(
        invalid_operation,
        Err(TelemetrySignalErrorV1::InvalidOperation)
    );
    let invalid_trace = TelemetrySignalV1::new(TelemetrySignalInputV1 {
        observed_at_utc_millis: 1_000,
        source: source(),
        kind: TelemetrySignalKindV1::Trace,
        priority: TelemetryPriorityV1::Info,
        operation: "rpc.call".into(),
        error_class: None,
        trace_id: Some("alice@example.test".into()),
        dropped_count: 0,
    });
    assert_eq!(invalid_trace, Err(TelemetrySignalErrorV1::InvalidTraceId));
}

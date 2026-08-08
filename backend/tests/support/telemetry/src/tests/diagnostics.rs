use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::collector::control;
use crate::collector::storage::{TelemetryRetentionV1, TelemetrySegmentStore};
use crate::fixtures::directory::unique_directory;
use makosh_runtime_protocol::v1::{
    GetTelemetryDiagnosticsRequestV1, TelemetryRuntimeControlRequestV1,
    TelemetryRuntimeControlResponseV1,
    telemetry_runtime_control_request_v1::Operation as RequestOperation,
    telemetry_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_telemetry_protocol::{
    TelemetryPriorityV1, TelemetrySignalInputV1, TelemetrySignalKindV1, TelemetrySignalV1,
    TelemetrySourceV1,
};
use prost::Message;

#[test]
fn collector_returns_only_aggregate_diagnostics_over_its_inherited_channel() {
    let directory = unique_directory("diagnostics");
    let store = TelemetrySegmentStore::open(
        directory.clone(),
        TelemetryRetentionV1::new(1024, 2048, 60).expect("retention"),
    )
    .expect("open store");
    store.append(&signal()).expect("append signal");
    let (mut kernel, collector) = UnixStream::pair().expect("control pair");
    let worker = std::thread::spawn(move || control::serve_diagnostics(collector, store));

    write_frame(&mut kernel, &diagnostics_request());
    let response = TelemetryRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
        .expect("typed diagnostics response");
    assert_eq!(response.error_code, "");
    assert!(matches!(
        response.result,
        Some(ResponseResult::Diagnostics(diagnostics))
            if diagnostics.segment_count == 1 && diagnostics.total_bytes > 0
    ));
    drop(kernel);
    worker
        .join()
        .expect("collector worker")
        .expect("serve diagnostics");
    std::fs::remove_dir_all(directory).expect("remove test directory");
}

#[test]
fn collector_rejects_malformed_control_without_echoing_input() {
    let directory = unique_directory("diagnostics-malformed");
    let store = TelemetrySegmentStore::open(
        directory.clone(),
        TelemetryRetentionV1::new(1024, 2048, 60).expect("retention"),
    )
    .expect("open store");
    let (mut kernel, collector) = UnixStream::pair().expect("control pair");
    let worker = std::thread::spawn(move || control::serve_diagnostics(collector, store));

    write_frame(&mut kernel, b"private-message-content");
    let response = TelemetryRuntimeControlResponseV1::decode(read_frame(&mut kernel).as_slice())
        .expect("typed error response");
    assert!(response.result.is_none());
    assert_eq!(response.error_code, "invalid_request");
    drop(kernel);
    worker
        .join()
        .expect("collector worker")
        .expect("serve diagnostics");
    std::fs::remove_dir_all(directory).expect("remove test directory");
}

fn diagnostics_request() -> Vec<u8> {
    TelemetryRuntimeControlRequestV1 {
        operation: Some(RequestOperation::GetDiagnostics(
            GetTelemetryDiagnosticsRequestV1 {},
        )),
    }
    .encode_to_vec()
}

fn signal() -> TelemetrySignalV1 {
    TelemetrySignalV1::new(TelemetrySignalInputV1 {
        observed_at_utc_millis: 1,
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

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) {
    stream
        .write_all(&[bytes.len() as u8])
        .expect("frame length");
    stream.write_all(bytes).expect("frame bytes");
    stream.flush().expect("flush frame");
}

fn read_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = [0_u8; 1];
    stream.read_exact(&mut length).expect("frame length");
    let mut bytes = vec![0_u8; usize::from(length[0])];
    stream.read_exact(&mut bytes).expect("frame bytes");
    bytes
}

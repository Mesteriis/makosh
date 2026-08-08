//! Aggregate-only diagnostics served over an authenticated inherited control FD.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    TelemetryDiagnosticsV1, TelemetryRuntimeControlRequestV1, TelemetryRuntimeControlResponseV1,
    telemetry_runtime_control_request_v1::Operation as RequestOperation,
    telemetry_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::validation::telemetry::validate_telemetry_runtime_control_request;
use prost::Message;

use crate::storage::TelemetrySegmentStore;

use super::framing::{read_frame, write_frame};

pub fn serve(mut stream: UnixStream, store: TelemetrySegmentStore) -> Result<(), String> {
    developer_log("ready");
    while let Ok(request) = read_frame(&mut stream) {
        developer_log("request_received");
        let response = response_for(&request, &store);
        write_frame(&mut stream, &response.encode_to_vec())?;
        developer_log("response_sent");
    }
    developer_log("control_closed");
    Ok(())
}

fn developer_log(event: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_telemetry_diagnostics event={event}");
    }
}

fn response_for(
    request: &[u8],
    store: &TelemetrySegmentStore,
) -> TelemetryRuntimeControlResponseV1 {
    let result = TelemetryRuntimeControlRequestV1::decode(request)
        .map_err(|_| "invalid_request")
        .and_then(|request| {
            validate_telemetry_runtime_control_request(&request)
                .map_err(|_| "operation_not_available")?;
            diagnostics_response(request, store)
        });
    result.unwrap_or_else(error_response)
}

fn diagnostics_response(
    request: TelemetryRuntimeControlRequestV1,
    store: &TelemetrySegmentStore,
) -> Result<TelemetryRuntimeControlResponseV1, &'static str> {
    match request.operation {
        Some(RequestOperation::GetDiagnostics(_)) => store
            .diagnostics()
            .map(|diagnostics| TelemetryRuntimeControlResponseV1 {
                result: Some(ResponseResult::Diagnostics(TelemetryDiagnosticsV1 {
                    segment_count: diagnostics.segment_count(),
                    total_bytes: diagnostics.total_bytes(),
                })),
                error_code: String::new(),
            })
            .map_err(|_| "collector_unavailable"),
        None => Err("operation_not_available"),
    }
}

fn error_response(error_code: &'static str) -> TelemetryRuntimeControlResponseV1 {
    TelemetryRuntimeControlResponseV1 {
        result: None,
        error_code: error_code.to_owned(),
    }
}

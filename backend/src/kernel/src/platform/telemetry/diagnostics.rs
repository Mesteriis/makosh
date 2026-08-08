//! Owner-authorized, aggregate-only diagnostics relay for Telemetry Collector.

use makosh_runtime_protocol::v1::{
    GetTelemetryDiagnosticsRequestV1, TelemetryRuntimeControlRequestV1,
    TelemetryRuntimeControlResponseV1,
    telemetry_runtime_control_request_v1::Operation as RequestOperation,
    telemetry_runtime_control_response_v1::Result as ResponseResult,
};
use makosh_runtime_protocol::validation::telemetry::validate_telemetry_runtime_control_response;
use prost::Message;

use crate::platform::telemetry::binding::TELEMETRY_PROCESS_ID;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryDiagnostics {
    segment_count: u32,
    total_bytes: u64,
}

impl TelemetryDiagnostics {
    #[must_use]
    pub const fn segment_count(self) -> u32 {
        self.segment_count
    }

    #[must_use]
    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }
}

pub fn read(supervisor: &ManagedRuntimeSupervisor) -> Result<TelemetryDiagnostics, String> {
    let response = supervisor
        .relay(TELEMETRY_PROCESS_ID, request().encode_to_vec())
        .inspect_err(|error| developer_diagnostics_error("relay", error))?;
    parse(&response).inspect_err(|error| developer_diagnostics_error("response", error))
}

fn developer_diagnostics_error(stage: &str, error: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        println!("developer_telemetry_diagnostics stage={stage} error={error}");
    }
}

pub(crate) fn parse(response: &[u8]) -> Result<TelemetryDiagnostics, String> {
    let response = TelemetryRuntimeControlResponseV1::decode(response)
        .map_err(|_| "Telemetry diagnostics response is invalid".to_owned())?;
    validate_telemetry_runtime_control_response(&response)
        .map_err(|_| "Telemetry diagnostics response is invalid".to_owned())?;
    let Some(ResponseResult::Diagnostics(diagnostics)) = response.result else {
        return Err("Telemetry diagnostics response is unavailable".to_owned());
    };
    Ok(TelemetryDiagnostics {
        segment_count: diagnostics.segment_count,
        total_bytes: diagnostics.total_bytes,
    })
}

fn request() -> TelemetryRuntimeControlRequestV1 {
    TelemetryRuntimeControlRequestV1 {
        operation: Some(RequestOperation::GetDiagnostics(
            GetTelemetryDiagnosticsRequestV1 {},
        )),
    }
}

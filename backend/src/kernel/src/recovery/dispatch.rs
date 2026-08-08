use std::path::Path;

use makosh_gateway_protocol::v1::{
    GetRecoveryStatusResponseV1, RecoveryControlRequestV1, RecoveryControlResponseV1,
    ShutdownKernelResponseV1, ValidateControlStoreResponseV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{ControlStoreStatusV1, KernelStateV1, RecoveryStatusV1};

use crate::control_store::lifecycle::open_validated_control_store;

use super::export::export_recovery_control_store;

pub(super) struct RecoveryAction {
    pub(super) response: RecoveryControlResponseV1,
    pub(super) shutdown: bool,
}

pub(super) fn recovery_response(
    request: RecoveryControlRequestV1,
    store_path: &Path,
    online_store: Option<&SqliteControlStore>,
) -> RecoveryAction {
    use makosh_gateway_protocol::v1::recovery_control_request_v1::Operation;
    use makosh_gateway_protocol::v1::recovery_control_response_v1::Result;

    match request.operation {
        Some(Operation::GetRecoveryStatus(_)) => status_action(store_path, online_store, true),
        Some(Operation::ValidateControlStore(_)) => status_action(store_path, online_store, false),
        Some(Operation::ExportControlStore(_)) => {
            export_recovery_control_store(store_path, online_store)
        }
        Some(Operation::ShutdownKernel(_)) => RecoveryAction {
            response: RecoveryControlResponseV1 {
                result: Some(Result::ShutdownKernel(ShutdownKernelResponseV1 {})),
                error_code: String::new(),
            },
            shutdown: true,
        },
        None => RecoveryAction {
            response: RecoveryControlResponseV1 {
                result: None,
                error_code: "operation_not_available".to_owned(),
            },
            shutdown: false,
        },
    }
}

fn status_action(
    store_path: &Path,
    online_store: Option<&SqliteControlStore>,
    recovery: bool,
) -> RecoveryAction {
    use makosh_gateway_protocol::v1::recovery_control_response_v1::Result;

    let status = recovery_status(store_path, online_store);
    let result = if recovery {
        Result::GetRecoveryStatus(GetRecoveryStatusResponseV1 {
            status: Some(status),
        })
    } else {
        Result::ValidateControlStore(ValidateControlStoreResponseV1 {
            status: Some(status),
        })
    };
    RecoveryAction {
        response: RecoveryControlResponseV1 {
            result: Some(result),
            error_code: String::new(),
        },
        shutdown: false,
    }
}

fn recovery_status(
    store_path: &Path,
    online_store: Option<&SqliteControlStore>,
) -> RecoveryStatusV1 {
    let generation = online_store
        .map(|store| store.snapshot().generation())
        .or_else(|| {
            open_validated_control_store(store_path)
                .ok()
                .map(|store| store.snapshot().generation())
        });
    match generation {
        Some(generation) => RecoveryStatusV1 {
            state: KernelStateV1::RecoveryOnly as i32,
            control_store_status: ControlStoreStatusV1::Trustworthy as i32,
            kernel_generation: generation,
            blocker_code: String::new(),
        },
        None => RecoveryStatusV1 {
            state: KernelStateV1::RecoveryOnly as i32,
            control_store_status: ControlStoreStatusV1::Unavailable as i32,
            kernel_generation: 0,
            blocker_code: "control_store_unavailable".to_owned(),
        },
    }
}

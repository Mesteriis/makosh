use std::path::Path;

use makosh_gateway_protocol::v1::{ExportControlStoreResponseV1, RecoveryControlResponseV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::control_store::lifecycle::open_validated_control_store;
use crate::infrastructure::filesystem::prepare_owner_private_directory;

use super::dispatch::RecoveryAction;

pub(super) fn export_recovery_control_store(
    store_path: &Path,
    online_store: Option<&SqliteControlStore>,
) -> RecoveryAction {
    let Some(data_dir) = store_path.parent() else {
        return unavailable_export_action();
    };
    let export_dir = data_dir.join("recovery");
    if prepare_owner_private_directory(&export_dir).is_err() {
        return unavailable_export_action();
    }
    let destination = export_dir.join("control-store.sqlite");
    let opened_store;
    let store = if let Some(store) = online_store {
        store
    } else {
        let Ok(store) = open_validated_control_store(store_path) else {
            return unavailable_export_action();
        };
        opened_store = store;
        &opened_store
    };
    let Ok(export) = store.export_to(&destination) else {
        return unavailable_export_action();
    };
    let Ok(export_size_bytes) = std::fs::metadata(destination).map(|metadata| metadata.len())
    else {
        return unavailable_export_action();
    };

    use makosh_gateway_protocol::v1::recovery_control_response_v1::Result;
    RecoveryAction {
        response: RecoveryControlResponseV1 {
            result: Some(Result::ExportControlStore(ExportControlStoreResponseV1 {
                export_sha256: export.sha256_bytes().to_vec(),
                export_size_bytes,
            })),
            error_code: String::new(),
        },
        shutdown: false,
    }
}

fn unavailable_export_action() -> RecoveryAction {
    RecoveryAction {
        response: RecoveryControlResponseV1 {
            result: None,
            error_code: "control_store_export_unavailable".to_owned(),
        },
        shutdown: false,
    }
}

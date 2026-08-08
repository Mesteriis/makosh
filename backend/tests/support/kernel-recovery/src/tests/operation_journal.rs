use makosh_kernel_control_store::{
    OperationAdmissionV1, OperationIdV1, OperationStatusV1, OperationTerminalOutcomeV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::common::unique_target_root;

#[test]
fn operation_journal_is_idempotent_and_retains_the_terminal_outcome() {
    let root = unique_target_root("makosh-operation-journal");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let operation_id = OperationIdV1::new([7; 16]);
    let digest = [8; 32];
    assert_eq!(
        store
            .admit_operation(operation_id, digest, 1_000)
            .expect("admit operation"),
        OperationAdmissionV1::Admitted
    );
    assert_admitted_status(&store, operation_id);
    let terminal = complete_succeeded(&store, operation_id, digest);
    assert_terminal_idempotency(&store, operation_id, digest, terminal);
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn assert_admitted_status(store: &SqliteControlStore, operation_id: OperationIdV1) {
    assert_eq!(
        store
            .operation_status(operation_id)
            .expect("read admitted operation"),
        Some(OperationStatusV1::Admitted)
    );
}

fn complete_succeeded(
    store: &SqliteControlStore,
    operation_id: OperationIdV1,
    digest: [u8; 32],
) -> OperationStatusV1 {
    store
        .complete_operation(
            operation_id,
            digest,
            &OperationTerminalOutcomeV1::Succeeded {
                response_digest: [9; 32],
            },
        )
        .expect("complete operation");
    OperationStatusV1::Terminal(OperationTerminalOutcomeV1::Succeeded {
        response_digest: [9; 32],
    })
}

fn assert_terminal_idempotency(
    store: &SqliteControlStore,
    operation_id: OperationIdV1,
    digest: [u8; 32],
    terminal: OperationStatusV1,
) {
    assert_eq!(
        store
            .operation_status(operation_id)
            .expect("read completed operation"),
        Some(terminal.clone())
    );
    assert_eq!(
        store
            .admit_operation(operation_id, digest, 2_000)
            .expect("repeat operation"),
        OperationAdmissionV1::Duplicate(terminal.clone())
    );
    store
        .complete_operation(
            operation_id,
            digest,
            &OperationTerminalOutcomeV1::Succeeded {
                response_digest: [9; 32],
            },
        )
        .expect("idempotent completion");
    assert!(store.admit_operation(operation_id, [6; 32], 2_000).is_err());
    assert!(
        store
            .complete_operation(
                operation_id,
                digest,
                &OperationTerminalOutcomeV1::Failed {
                    code: "different".to_owned()
                }
            )
            .is_err()
    );
}

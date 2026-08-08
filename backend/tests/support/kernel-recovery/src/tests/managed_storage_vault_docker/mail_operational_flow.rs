//! Managed Gateway conformance for the Mail-owned operational read model.

use std::time::Duration;

use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSyncInboxRequestV1,
    client_contract::MailClientContractV1,
    message_flags::{
        MailMessageFlagCommandV1, MailMessageFlagKindV1, MailMessageFlagOperationOutcomeV1,
        MailMessageFlagStatusRequestV1,
    },
    message_location::{
        MailMessageLocationCommandV1, MailMessageLocationKindV1,
        MailMessageLocationOperationOutcomeV1, MailMessageLocationStatusRequestV1,
    },
    message_permanent_delete::{
        MailMessagePermanentDeleteCommandV1, MailMessagePermanentDeleteConfirmationV1,
        MailMessagePermanentDeleteOperationOutcomeV1, MailMessagePermanentDeleteStatusRequestV1,
    },
    operational::{
        MailFolderKindV1, MailMessageFlagV1, MailOperationalQueryResponseV1, MailOperationalQueryV1,
    },
};
use makosh_mail_runtime::client_port::{
    MailClientPortErrorV1, decode_module_response, encode_module_request,
};

use super::*;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

pub(super) fn assert_mail_operational_read(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let folders = query_operational(
        store,
        supervisor,
        mail,
        71,
        MailOperationalQueryV1::ListFolders {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 20,
        },
    );
    let MailOperationalQueryResponseV1::Folders(folders) = folders else {
        panic!("Mail folders query returned the wrong response")
    };
    assert_eq!(folders.items.len(), 3);
    let inbox = folders
        .items
        .iter()
        .find(|folder| folder.kind == MailFolderKindV1::Inbox)
        .expect("Mail folder discovery preserves the Inbox role");
    assert_eq!(inbox.connection_id, MAIL_ACCOUNT_ID);
    assert_eq!(inbox.folder_id, "INBOX");
    assert_eq!(inbox.total_messages, 1);
    assert_eq!(inbox.unread_messages, 1);
    assert!(folders.items.iter().any(|folder| {
        folder.folder_id == "Archive" && folder.kind == MailFolderKindV1::Archive
    }));
    assert!(
        folders.items.iter().any(|folder| {
            folder.folder_id == "Trash" && folder.kind == MailFolderKindV1::Trash
        })
    );

    let messages = query_operational(
        store,
        supervisor,
        mail,
        72,
        MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some(inbox.folder_id.clone()),
            provider_thread_id: None,
            cursor: None,
            limit: 1,
        },
    );
    let MailOperationalQueryResponseV1::Messages(messages) = messages else {
        panic!("Mail messages query returned the wrong response")
    };
    assert_eq!(messages.items.len(), 1);
    let summary = &messages.items[0];
    assert_eq!(summary.connection_id, MAIL_ACCOUNT_ID);
    assert_eq!(summary.folder_ids, ["INBOX"]);
    assert_eq!(
        summary.subject.as_deref(),
        Some("managed attachment evidence")
    );
    assert_eq!(summary.sender.as_deref(), Some("source@example.test"));
    assert_eq!(summary.recipients, ["owner@example.test"]);
    assert!(summary.has_plain_text);
    assert!(summary.has_attachments);
    assert!(summary.observation_anchor_id.iter().any(|byte| *byte != 0));
    let cursor = messages
        .next_cursor
        .clone()
        .expect("bounded Mail message page cursor");
    let message_id = summary.message_id.clone();

    let detail = query_operational(
        store,
        supervisor,
        mail,
        73,
        MailOperationalQueryV1::GetMessage {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            message_id,
        },
    );
    let MailOperationalQueryResponseV1::Message(detail) = detail else {
        panic!("Mail message detail query returned the wrong response")
    };
    assert_eq!(detail.summary, *summary);

    assert_operational_response_is_private(store, supervisor, mail);
    assert_cross_account_query_is_rejected(store, supervisor, mail);
    assert_cursor_scope_is_enforced(store, supervisor, mail, &cursor);
    assert_stale_operational_generation_is_rejected(store, supervisor, mail);

    std::thread::sleep(Duration::from_secs(1));
    sync_mail(
        store,
        supervisor,
        mail,
        77,
        "managed-mail-operational-cursor-stale",
    );
    wait_for_successful_sync_run(
        store,
        supervisor,
        mail,
        "managed-mail-operational-cursor-stale",
        1,
        78,
    );
    assert_stale_cursor_is_rejected(store, supervisor, mail, &cursor);
}

pub(super) fn assert_mail_message_flags(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    imap: &MailImapFixture,
) -> String {
    let messages = query_operational(
        store,
        supervisor,
        mail,
        78,
        MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some("INBOX".to_owned()),
            provider_thread_id: None,
            cursor: None,
            limit: 1,
        },
    );
    let MailOperationalQueryResponseV1::Messages(messages) = messages else {
        panic!("Mail message flag setup returned the wrong response")
    };
    let message_id = messages.items[0].message_id.clone();
    assert_opaque_imap_message_id(&message_id);
    let command = MailMessageFlagCommandV1 {
        operation_id: "managed-mail-message-read-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.clone(),
        kind: MailMessageFlagKindV1::Read,
        target_value: true,
    };
    let provider_mutations_before = imap.message_flag_mutations();

    let accepted = route_message_flag_command(store, supervisor, mail, 79, command.clone());
    assert_eq!(accepted, command.operation_id);

    let status = (0..50)
        .find_map(|attempt| {
            let status = query_message_flag_status(
                store,
                supervisor,
                mail,
                80 + attempt,
                &command.operation_id,
            );
            match status.outcome {
                MailMessageFlagOperationOutcomeV1::Pending => {
                    std::thread::sleep(Duration::from_millis(100));
                    None
                }
                _ => Some(status),
            }
        })
        .expect("Mail message flag operation reaches a terminal status");
    assert_eq!(status.outcome, MailMessageFlagOperationOutcomeV1::Succeeded);
    assert!(
        status
            .projection_revision
            .is_some_and(|revision| revision > 0)
    );
    assert_eq!(
        imap.message_flag_mutations(),
        provider_mutations_before + 1,
        "managed flag command must reach the provider exactly once"
    );

    let detail = query_operational(
        store,
        supervisor,
        mail,
        131,
        MailOperationalQueryV1::GetMessage {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            message_id: message_id.clone(),
        },
    );
    let MailOperationalQueryResponseV1::Message(detail) = detail else {
        panic!("Mail message flag projection returned the wrong response")
    };
    assert!(detail.summary.flags.contains(&MailMessageFlagV1::Read));
    let folders = query_operational(
        store,
        supervisor,
        mail,
        132,
        MailOperationalQueryV1::ListFolders {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 20,
        },
    );
    let MailOperationalQueryResponseV1::Folders(folders) = folders else {
        panic!("Mail message flag folder reconciliation returned the wrong response")
    };
    let inbox = folders
        .items
        .iter()
        .find(|folder| folder.kind == MailFolderKindV1::Inbox)
        .expect("Mail flag reconciliation preserves the Inbox role");
    assert_eq!(inbox.unread_messages, 0);
    assert!(folders.items.iter().any(|folder| {
        folder.folder_id == "Archive" && folder.kind == MailFolderKindV1::Archive
    }));
    assert!(
        folders.items.iter().any(|folder| {
            folder.folder_id == "Trash" && folder.kind == MailFolderKindV1::Trash
        })
    );

    let replayed = route_message_flag_command(store, supervisor, mail, 133, command);
    assert_eq!(replayed, accepted);
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(
        imap.message_flag_mutations(),
        provider_mutations_before + 1,
        "an exact replayed message flag command must not reach the provider twice"
    );
    message_id
}

pub(super) fn assert_mail_identity_survives_restart_and_stale_locator_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    imap: &MailImapFixture,
    expected_message_id: &str,
) {
    let messages = query_operational(
        store,
        supervisor,
        mail,
        140,
        MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some("INBOX".to_owned()),
            provider_thread_id: None,
            cursor: None,
            limit: 1,
        },
    );
    let MailOperationalQueryResponseV1::Messages(messages) = messages else {
        panic!("Mail restart identity query returned the wrong response")
    };
    assert_eq!(messages.items.len(), 1);
    assert_eq!(messages.items[0].message_id, expected_message_id);
    assert_opaque_imap_message_id(&messages.items[0].message_id);

    imap.set_uid_validity(2);
    let command = MailMessageFlagCommandV1 {
        operation_id: "managed-mail-message-stale-locator-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: expected_message_id.to_owned(),
        kind: MailMessageFlagKindV1::Starred,
        target_value: true,
    };
    let provider_connections_before = imap.accepted_connections();
    let provider_mutations_before = imap.message_flag_mutations();
    route_message_flag_command(store, supervisor, mail, 141, command.clone());
    let status = (0..50)
        .find_map(|attempt| {
            let status = query_message_flag_status(
                store,
                supervisor,
                mail,
                142 + attempt,
                &command.operation_id,
            );
            match status.outcome {
                MailMessageFlagOperationOutcomeV1::Pending => {
                    std::thread::sleep(Duration::from_millis(100));
                    None
                }
                _ => Some(status),
            }
        })
        .expect("stale IMAP locator reaches a terminal status");
    assert_eq!(
        status.outcome,
        MailMessageFlagOperationOutcomeV1::Rejected,
        "a changed provider UIDVALIDITY must reject the stored locator"
    );
    assert!(
        imap.accepted_connections() > provider_connections_before,
        "Mail must load the persisted locator and check it against the provider after restart"
    );
    assert_eq!(
        imap.message_flag_mutations(),
        provider_mutations_before,
        "stale UIDVALIDITY must be rejected before UID STORE"
    );

    let detail = query_operational(
        store,
        supervisor,
        mail,
        193,
        MailOperationalQueryV1::GetMessage {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            message_id: expected_message_id.to_owned(),
        },
    );
    let MailOperationalQueryResponseV1::Message(detail) = detail else {
        panic!("stale IMAP locator projection query returned the wrong response")
    };
    assert!(!detail.summary.flags.contains(&MailMessageFlagV1::Starred));
}

pub(super) fn assert_mail_message_archive(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    imap: &MailImapFixture,
) -> String {
    let messages = query_operational(
        store,
        supervisor,
        mail,
        200,
        MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some("INBOX".to_owned()),
            provider_thread_id: None,
            cursor: None,
            limit: 1,
        },
    );
    let MailOperationalQueryResponseV1::Messages(messages) = messages else {
        panic!("Mail message location setup returned the wrong response")
    };
    let message_id = messages.items[0].message_id.clone();
    assert_opaque_imap_message_id(&message_id);
    let command = MailMessageLocationCommandV1 {
        operation_id: "managed-mail-message-archive-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.clone(),
        kind: MailMessageLocationKindV1::Archive,
        target_folder_id: None,
    };
    let provider_mutations_before = imap.message_location_mutations();
    let accepted = route_message_location_command(store, supervisor, mail, 201, command.clone());
    assert_eq!(accepted, command.operation_id);
    let status =
        wait_for_message_location_status(store, supervisor, mail, 202, &command.operation_id);
    assert_eq!(
        status.outcome,
        MailMessageLocationOperationOutcomeV1::Succeeded
    );
    assert!(
        status
            .projection_revision
            .is_some_and(|revision| revision > 0)
    );
    assert_eq!(imap.message_mailbox(), "Archive");
    assert_eq!(
        imap.message_location_mutations(),
        provider_mutations_before + 1,
        "managed archive must reach IMAP exactly once"
    );
    assert_message_folders(store, supervisor, mail, 253, &message_id, &["Archive"]);

    let replayed = route_message_location_command(store, supervisor, mail, 254, command);
    assert_eq!(replayed, accepted);
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(
        imap.message_location_mutations(),
        provider_mutations_before + 1,
        "an exact replayed location command must not reach IMAP twice"
    );
    message_id
}

pub(super) fn assert_mail_message_location_survives_restart_and_fails_closed(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    imap: &MailImapFixture,
    message_id: &str,
) {
    let provider_mutations_before = imap.message_location_mutations();
    let trash = MailMessageLocationCommandV1 {
        operation_id: "managed-mail-message-trash-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.to_owned(),
        kind: MailMessageLocationKindV1::Trash,
        target_folder_id: None,
    };
    route_message_location_command(store, supervisor, mail, 260, trash.clone());
    let trash_status =
        wait_for_message_location_status(store, supervisor, mail, 261, &trash.operation_id);
    assert_eq!(
        trash_status.outcome,
        MailMessageLocationOperationOutcomeV1::Succeeded,
        "restart must restore the Archive locator and move the same stable message to Trash"
    );
    assert_eq!(imap.message_mailbox(), "Trash");
    assert_eq!(
        imap.message_location_mutations(),
        provider_mutations_before + 1
    );
    assert_message_folders(store, supervisor, mail, 312, message_id, &["Trash"]);

    imap.set_uid_validity(10);
    let stale = MailMessageLocationCommandV1 {
        operation_id: "managed-mail-message-restore-stale-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.to_owned(),
        kind: MailMessageLocationKindV1::Restore,
        target_folder_id: None,
    };
    route_message_location_command(store, supervisor, mail, 313, stale.clone());
    let stale_status =
        wait_for_message_location_status(store, supervisor, mail, 314, &stale.operation_id);
    assert_eq!(
        stale_status.outcome,
        MailMessageLocationOperationOutcomeV1::Rejected
    );
    assert_eq!(
        imap.message_location_mutations(),
        provider_mutations_before + 1,
        "stale UIDVALIDITY must reject before UID MOVE"
    );

    imap.set_uid_validity(9);
    imap.set_move_supported(false);
    let unsupported = MailMessageLocationCommandV1 {
        operation_id: "managed-mail-message-restore-unsupported-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.to_owned(),
        kind: MailMessageLocationKindV1::Restore,
        target_folder_id: None,
    };
    route_message_location_command(store, supervisor, mail, 365, unsupported.clone());
    let unsupported_status =
        wait_for_message_location_status(store, supervisor, mail, 366, &unsupported.operation_id);
    assert_eq!(
        unsupported_status.outcome,
        MailMessageLocationOperationOutcomeV1::Unsupported
    );
    assert_eq!(
        imap.message_location_mutations(),
        provider_mutations_before + 1,
        "server without MOVE/UIDPLUS must not receive UID MOVE"
    );
    assert_message_folders(store, supervisor, mail, 417, message_id, &["Trash"]);
}

pub(super) fn assert_mail_message_permanent_delete_is_fenced_and_replay_safe(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    imap: &MailImapFixture,
    message_id: &str,
) {
    let projection_revision =
        assert_message_folders(store, supervisor, mail, 418, message_id, &["Trash"]);
    assert!(
        projection_revision > 1,
        "managed stale-revision proof needs an earlier positive projection revision"
    );
    let provider_deletions_before = imap.message_permanent_deletions();

    let stale = MailMessagePermanentDeleteCommandV1 {
        operation_id: "managed-mail-message-permanent-delete-stale-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.to_owned(),
        expected_projection_revision: projection_revision - 1,
        confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
    };
    assert_message_permanent_delete_command_rejected(store, supervisor, mail, 419, stale);
    assert_eq!(
        imap.message_permanent_deletions(),
        provider_deletions_before
    );

    imap.set_move_supported(false);
    let unsupported = MailMessagePermanentDeleteCommandV1 {
        operation_id: "managed-mail-message-permanent-delete-unsupported-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.to_owned(),
        expected_projection_revision: projection_revision,
        confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
    };
    route_message_permanent_delete_command(store, supervisor, mail, 471, unsupported.clone());
    let unsupported_status = wait_for_message_permanent_delete_status(
        store,
        supervisor,
        mail,
        472,
        &unsupported.operation_id,
    );
    assert_eq!(
        unsupported_status.outcome,
        MailMessagePermanentDeleteOperationOutcomeV1::Unsupported
    );
    assert_eq!(
        imap.message_permanent_deletions(),
        provider_deletions_before
    );

    imap.set_move_supported(true);
    let command = MailMessagePermanentDeleteCommandV1 {
        operation_id: "managed-mail-message-permanent-delete-1".to_owned(),
        connection_id: MAIL_ACCOUNT_ID.to_owned(),
        message_id: message_id.to_owned(),
        expected_projection_revision: projection_revision,
        confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
    };
    let accepted =
        route_message_permanent_delete_command(store, supervisor, mail, 523, command.clone());
    assert_eq!(accepted, command.operation_id);
    let status = wait_for_message_permanent_delete_status(
        store,
        supervisor,
        mail,
        524,
        &command.operation_id,
    );
    assert_eq!(
        status.outcome,
        MailMessagePermanentDeleteOperationOutcomeV1::Succeeded
    );
    assert!(
        status
            .deletion_projection_revision
            .is_some_and(|revision| revision > projection_revision)
    );
    assert_eq!(
        imap.message_permanent_deletions(),
        provider_deletions_before + 1,
        "managed permanent delete must reach exact UID EXPUNGE once"
    );
    assert_message_absent_from_trash(store, supervisor, mail, 575, message_id);

    let replayed = route_message_permanent_delete_command(store, supervisor, mail, 576, command);
    assert_eq!(replayed, accepted);
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(
        imap.message_permanent_deletions(),
        provider_deletions_before + 1,
        "an exact replayed permanent delete must not reach IMAP twice"
    );
}

fn route_message_location_command(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    command: MailMessageLocationCommandV1,
) -> String {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::MessageLocationCommand(command),
    )
    .expect("encode Mail message location command");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::MessageLocationCommand,
        &request,
    )
    .expect("route Mail message location command");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::MessageLocationCommand, &bytes)
            .expect("decode Mail message location command response");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::MessageLocationAccepted(response) = response else {
        panic!("Mail message location command returned the wrong response")
    };
    response.operation_id
}

fn wait_for_message_location_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) -> makosh_mail_api::message_location::MailMessageLocationOperationStatusV1 {
    (0..50)
        .find_map(|attempt| {
            let status = query_message_location_status(
                store,
                supervisor,
                mail,
                request_id + attempt,
                operation_id,
            );
            match status.outcome {
                MailMessageLocationOperationOutcomeV1::Pending => {
                    std::thread::sleep(Duration::from_millis(100));
                    None
                }
                _ => Some(status),
            }
        })
        .expect("Mail message location operation reaches a terminal status")
}

fn query_message_location_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) -> makosh_mail_api::message_location::MailMessageLocationOperationStatusV1 {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::MessageLocationStatus(MailMessageLocationStatusRequestV1 {
            operation_id: operation_id.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("encode Mail message location status query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::MessageLocationQuery,
        &request,
    )
    .expect("route Mail message location status query");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::MessageLocationQuery, &bytes)
            .expect("decode Mail message location status response");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::MessageLocationStatus(Some(response)) = response else {
        panic!("Mail message location status returned the wrong response")
    };
    response
}

fn route_message_permanent_delete_command(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    command: MailMessagePermanentDeleteCommandV1,
) -> String {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::MessagePermanentDeleteCommand(command),
    )
    .expect("encode Mail message permanent delete command");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::MessagePermanentDeleteCommand,
        &request,
    )
    .expect("route Mail message permanent delete command");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::MessagePermanentDeleteCommand, &bytes)
            .expect("decode Mail message permanent delete command response");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::MessagePermanentDeleteAccepted(response) = response else {
        panic!("Mail message permanent delete command returned the wrong response")
    };
    response.operation_id
}

fn assert_message_permanent_delete_command_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    command: MailMessagePermanentDeleteCommandV1,
) {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::MessagePermanentDeleteCommand(command),
    )
    .expect("encode rejected Mail message permanent delete command");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::MessagePermanentDeleteCommand,
        &request,
    )
    .expect("route rejected Mail message permanent delete command");
    assert!(matches!(
        decode_module_response(MailClientContractV1::MessagePermanentDeleteCommand, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    ));
}

fn wait_for_message_permanent_delete_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) -> makosh_mail_api::message_permanent_delete::MailMessagePermanentDeleteOperationStatusV1 {
    (0..50)
        .find_map(|attempt| {
            let status = query_message_permanent_delete_status(
                store,
                supervisor,
                mail,
                request_id + attempt,
                operation_id,
            );
            match status.outcome {
                MailMessagePermanentDeleteOperationOutcomeV1::Pending => {
                    std::thread::sleep(Duration::from_millis(100));
                    None
                }
                _ => Some(status),
            }
        })
        .expect("Mail message permanent delete operation reaches a terminal status")
}

fn query_message_permanent_delete_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) -> makosh_mail_api::message_permanent_delete::MailMessagePermanentDeleteOperationStatusV1 {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::MessagePermanentDeleteStatus(
            MailMessagePermanentDeleteStatusRequestV1 {
                operation_id: operation_id.to_owned(),
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
            },
        ),
    )
    .expect("encode Mail message permanent delete status query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::MessagePermanentDeleteQuery,
        &request,
    )
    .expect("route Mail message permanent delete status query");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::MessagePermanentDeleteQuery, &bytes)
            .expect("decode Mail message permanent delete status response");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::MessagePermanentDeleteStatus(Some(response)) = response else {
        panic!("Mail message permanent delete status returned the wrong response")
    };
    response
}

fn assert_message_folders(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    message_id: &str,
    expected: &[&str],
) -> u64 {
    let detail = query_operational(
        store,
        supervisor,
        mail,
        request_id,
        MailOperationalQueryV1::GetMessage {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            message_id: message_id.to_owned(),
        },
    );
    let MailOperationalQueryResponseV1::Message(detail) = detail else {
        panic!("Mail message location projection returned the wrong response")
    };
    assert_eq!(detail.summary.message_id, message_id);
    assert_eq!(
        detail.summary.folder_ids,
        expected
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
    );
    detail.summary.projection_revision
}

fn assert_message_absent_from_trash(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    message_id: &str,
) {
    let response = query_operational(
        store,
        supervisor,
        mail,
        request_id,
        MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some("Trash".to_owned()),
            provider_thread_id: None,
            cursor: None,
            limit: 20,
        },
    );
    let MailOperationalQueryResponseV1::Messages(messages) = response else {
        panic!("Mail Trash query returned the wrong response after permanent delete")
    };
    assert!(
        messages
            .items
            .iter()
            .all(|message| message.message_id != message_id),
        "permanent delete must remove the provider operational projection"
    );
}

fn assert_opaque_imap_message_id(message_id: &str) {
    let digest = message_id
        .strip_prefix("imap:v1:")
        .expect("IMAP message identity uses the versioned Mail-owned namespace");
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert!(!message_id.contains("INBOX"));
}

fn route_message_flag_command(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    command: MailMessageFlagCommandV1,
) -> String {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::MessageFlagCommand(command),
    )
    .expect("encode Mail message flag command");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::MessageFlagCommand,
        &request,
    )
    .expect("route Mail message flag command");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::MessageFlagCommand, &bytes)
            .expect("decode Mail message flag command response");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::MessageFlagAccepted(response) = response else {
        panic!("Mail message flag command returned the wrong response")
    };
    response.operation_id
}

fn query_message_flag_status(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) -> makosh_mail_api::message_flags::MailMessageFlagOperationStatusV1 {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::MessageFlagStatus(MailMessageFlagStatusRequestV1 {
            operation_id: operation_id.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("encode Mail message flag status query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::MessageFlagQuery,
        &request,
    )
    .expect("route Mail message flag status query");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::MessageFlagQuery, &bytes)
            .expect("decode Mail message flag status response");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::MessageFlagStatus(Some(response)) = response else {
        panic!("Mail message flag status returned the wrong response")
    };
    response
}

fn query_operational(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    query: MailOperationalQueryV1,
) -> MailOperationalQueryResponseV1 {
    let request = encode_module_request(request_id, &MailClientRequestV1::OperationalQuery(query))
        .expect("encode Mail operational query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route Mail operational query");
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes)
            .expect("decode Mail operational query");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::OperationalQuery(response) = response else {
        panic!("Mail operational route returned the wrong response")
    };
    response
}

fn assert_operational_response_is_private(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        74,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: None,
            provider_thread_id: None,
            cursor: None,
            limit: 20,
        }),
    )
    .expect("encode Mail privacy query");
    let response = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route Mail privacy query");
    for forbidden in [
        b"managed-mail-imap-password".as_slice(),
        b"Content-Type: multipart/mixed".as_slice(),
        b"Y2xlYW4tcm9vbS1hdHRhY2htZW50".as_slice(),
        b"makosh-fixture--".as_slice(),
    ] {
        assert!(
            !response
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "Mail operational response exposed provider or credential bytes"
        );
    }
}

fn assert_cross_account_query_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        75,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListFolders {
            connection_id: "other-mail-account".to_owned(),
            cursor: None,
            limit: 20,
        }),
    )
    .expect("encode cross-account Mail query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route rejected cross-account Mail query");
    assert_eq!(
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert_runtime_active(supervisor, mail);
}

fn assert_cursor_scope_is_enforced(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    cursor: &str,
) {
    let request = encode_module_request(
        76,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: None,
            provider_thread_id: None,
            cursor: Some(cursor.to_owned()),
            limit: 1,
        }),
    )
    .expect("encode wrong-scope Mail cursor query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route rejected wrong-scope Mail cursor query");
    assert_eq!(
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert_runtime_active(supervisor, mail);
}

fn assert_stale_cursor_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    cursor: &str,
) {
    let request = encode_module_request(
        78,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some("INBOX".to_owned()),
            provider_thread_id: None,
            cursor: Some(cursor.to_owned()),
            limit: 1,
        }),
    )
    .expect("encode stale Mail cursor query");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::OperationalQuery,
        &request,
    )
    .expect("route rejected stale Mail cursor query");
    assert_eq!(
        decode_module_response(MailClientContractV1::OperationalQuery, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert_runtime_active(supervisor, mail);
}

fn assert_stale_operational_generation_is_rejected(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let request = encode_module_request(
        79,
        &MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListFolders {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 20,
        }),
    )
    .expect("encode stale-generation Mail query");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation + 1,
        mail.grant_epoch,
        MailClientContractV1::OperationalQuery.capability_id(),
        &request,
    );
    assert_eq!(
        route_managed_client_request(store, &supervisor.relay_port(), &route)
            .expect_err("stale Mail operational generation"),
        "managed runtime fence is stale"
    );
}

pub(super) fn sync_mail(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: operation_id.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("encode Mail sync for cursor invalidation");
    let bytes = route(
        store,
        supervisor,
        mail,
        MailClientContractV1::Sync,
        &request,
    )
    .expect("route Mail sync for cursor invalidation");
    let (_, response) = decode_module_response(MailClientContractV1::Sync, &bytes)
        .expect("decode Mail sync for cursor invalidation");
    assert_eq!(
        response,
        MailClientResponseV1::SyncInboxAccepted {
            operation_id: operation_id.to_owned(),
        }
    );
}

pub(super) fn mail_operational_message_count(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
) -> usize {
    let response = query_operational(
        store,
        supervisor,
        mail,
        request_id,
        MailOperationalQueryV1::ListMessages {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            folder_id: Some("INBOX".to_owned()),
            provider_thread_id: None,
            cursor: None,
            limit: 200,
        },
    );
    let MailOperationalQueryResponseV1::Messages(messages) = response else {
        panic!("Mail operational message count returned the wrong response")
    };
    messages.items.len()
}

fn route(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request: &[u8],
) -> Result<Vec<u8>, String> {
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        contract.capability_id(),
        request,
    );
    route_managed_client_request(store, &supervisor.relay_port(), &route)
}

fn assert_runtime_active(supervisor: &ManagedRuntimeSupervisor, mail: &StartedMailRuntime) {
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("observe Mail after rejected operational query"),
        "rejected Mail operational query must not terminate the managed runtime"
    );
}

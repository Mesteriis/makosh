//! Mail integration process root for the exact Kernel-inherited runtime contract.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_mail_retained_evidence_replay_persistence::RetainedMailReplayErrorV1;
use makosh_mail_runtime::managed::{
    CompletedImapSyncProviderOperationV1, ImapSyncProviderPageDeliveryV1,
    MailDeliveryDispatchErrorV1, MailMessageFlagDispatchErrorV1,
    MailMessageLocationDispatchErrorV1, MailMessagePermanentDeleteDispatchErrorV1,
    execute_imap_sync_provider_operation, execute_mail_delivery_provider_operation,
    execute_mail_message_flag_provider_operation, execute_mail_message_location_provider_operation,
    execute_mail_message_permanent_delete_provider_operation,
};
use makosh_mail_runtime::{
    MailRuntimeAdmission,
    attachment_security_outbox::MailAttachmentSecurityOutboxRelayError,
    communications_outbox::MailCommunicationsOutboxRelayError,
    delivery_intent_consumer::MailDeliveryIntentConsumeErrorV1,
    delivery_intent_outbox::MailDeliveryIntentOutboxRelayErrorV1,
    delivery_intent_worker::{
        MailDeliveryIntentWorkerErrorV1, process_next_mail_delivery_intent_v1,
    },
    gmail_oauth::{
        CompletedGmailOAuthProviderOperationV1, MailGmailOAuthDispatchErrorV1,
        execute_gmail_oauth_provider_operation,
    },
    gmail_sync_worker::{
        CompletedGmailSyncProviderOperationV1, GmailSyncProviderPageDeliveryV1,
        execute_gmail_sync_provider_operation,
    },
    managed,
    person_source_fetch_worker::MailPersonSourceFetchWorkerErrorV1,
    retained_evidence_replay_consumer::MailReplayCommandConsumeErrorV1,
    retained_evidence_replay_result::MailReplayResultRelayErrorV1,
    settings,
};
use makosh_runtime_protocol::{
    v1::ManagedIntegrationRuntimeConfigurationV1,
    validation::{
        descriptor::{
            decode_settings_schema_v1, decode_settings_snapshot_v1,
            validate_settings_snapshot_against_schema_v1,
        },
        managed_integration_runtime::validate_managed_integration_runtime_configuration,
    },
};
use prost::Message;

struct InheritedPaths {
    descriptor: PathBuf,
    settings_schema: PathBuf,
    settings_snapshot: PathBuf,
    runtime_configuration: PathBuf,
    runtime_instance_id: String,
}

const MAX_CLIENT_DELIVERIES_PER_TICK: usize = 32;
const MAX_SYNC_PAGES_PER_ACCOUNT_PER_TICK: usize = 1;
const RUNTIME_TICK_INTERVAL: Duration = Duration::from_millis(100);

struct ActiveGmailSyncProviderOperationV1 {
    completion: tokio::task::JoinHandle<CompletedGmailSyncProviderOperationV1>,
    pages: tokio::sync::mpsc::Receiver<GmailSyncProviderPageDeliveryV1>,
    connection_id: String,
    operation_id: String,
    deadline_at_unix_seconds: i64,
    timed_out: bool,
}

struct ActiveImapSyncProviderOperationV1 {
    completion: tokio::task::JoinHandle<CompletedImapSyncProviderOperationV1>,
    pages: Option<std::sync::mpsc::Receiver<ImapSyncProviderPageDeliveryV1>>,
    connection_id: String,
    operation_id: String,
    deadline_at_unix_seconds: i64,
    timed_out: bool,
}

struct ActiveMailDeliveryProviderOperationV1 {
    completion: tokio::task::JoinHandle<Result<bool, MailDeliveryDispatchErrorV1>>,
    fence_epoch: u64,
}

struct ActiveMailMessageFlagProviderOperationV1 {
    completion: tokio::task::JoinHandle<Result<bool, MailMessageFlagDispatchErrorV1>>,
    fence_epoch: u64,
}

struct ActiveMailMessageLocationProviderOperationV1 {
    completion: tokio::task::JoinHandle<Result<bool, MailMessageLocationDispatchErrorV1>>,
    fence_epoch: u64,
}

struct ActiveMailMessagePermanentDeleteProviderOperationV1 {
    completion: tokio::task::JoinHandle<Result<bool, MailMessagePermanentDeleteDispatchErrorV1>>,
    fence_epoch: u64,
}

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    match arguments.next().as_deref() {
        Some(command) if command == OsStr::new("serve-inherited") => {
            serve_inherited(&mut arguments.peekable())
        }
        _ => Err("Mail runtime command is unavailable".to_owned()),
    }
}

fn serve_inherited<I>(arguments: &mut std::iter::Peekable<I>) -> Result<(), String>
where
    I: Iterator<Item = OsString>,
{
    let paths = parse_paths(arguments)?;
    let descriptor = read_contract(&paths.descriptor)?;
    let schema_bytes = read_contract(&paths.settings_schema)?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "Mail runtime settings schema is invalid".to_owned())?;
    let selected_snapshot_bytes = read_contract(&paths.settings_snapshot)?;
    let selected_snapshot = decode_settings_snapshot_v1(&selected_snapshot_bytes)
        .map_err(|_| "Mail runtime settings snapshot is invalid".to_owned())?;
    validate_settings_snapshot_against_schema_v1(&schema, &selected_snapshot)
        .map_err(|_| "Mail runtime settings snapshot is invalid".to_owned())?;
    let configuration = ManagedIntegrationRuntimeConfigurationV1::decode(
        read_contract(&paths.runtime_configuration)?.as_slice(),
    )
    .map_err(|_| "Mail runtime configuration is invalid".to_owned())?;
    validate_managed_integration_runtime_configuration(&configuration)
        .map_err(|_| "Mail runtime configuration is invalid".to_owned())?;
    if configuration.runtime_instance_id != paths.runtime_instance_id {
        return Err("Mail runtime configuration is stale".to_owned());
    }
    let configuration_snapshots = if configuration.configuration_instances.is_empty() {
        vec![(
            configuration.configuration_instance_id.clone(),
            selected_snapshot,
        )]
    } else {
        let selected = configuration
            .configuration_instances
            .iter()
            .find(|instance| {
                instance.configuration_instance_id == configuration.configuration_instance_id
            })
            .ok_or_else(|| "Mail runtime settings catalog is invalid".to_owned())?;
        if selected.settings_snapshot_bytes != selected_snapshot_bytes {
            return Err("Mail runtime settings catalog is stale".to_owned());
        }
        configuration
            .configuration_instances
            .iter()
            .map(|instance| {
                let snapshot = decode_settings_snapshot_v1(&instance.settings_snapshot_bytes)
                    .map_err(|_| "Mail runtime settings catalog is invalid".to_owned())?;
                validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
                    .map_err(|_| "Mail runtime settings catalog is invalid".to_owned())?;
                Ok((instance.configuration_instance_id.clone(), snapshot))
            })
            .collect::<Result<Vec<_>, String>>()?
    };
    let storage = configuration
        .storage
        .clone()
        .ok_or_else(|| "Mail runtime configuration is invalid".to_owned())?;
    let admissions = configuration_snapshots
        .into_iter()
        .map(|(configuration_instance_id, snapshot)| {
            let settings = settings::decode(&snapshot)?;
            Ok(MailRuntimeAdmission {
                logical_owner_id: configuration.logical_owner_id.clone(),
                logical_human_owner_id: configuration.logical_human_owner_id.clone(),
                configuration_instance_id,
                module_registration_id: configuration.registration_id.clone(),
                runtime_instance_id: configuration.runtime_instance_id.clone(),
                runtime_generation: configuration.runtime_generation,
                grant_epoch: configuration.grant_epoch,
                vault_runtime_generation: storage.vault_runtime_generation,
                settings_revision: snapshot.revision,
                account: settings.account,
                address_book: settings.address_book,
                gmail_oauth: settings.gmail_oauth,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let control_channel = inherited_control_channel()?;
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|_| "Mail runtime executor is unavailable".to_owned())?;
    let mut admitted = runtime
        .block_on(managed::open_admitted_runtime_catalog(
            control_channel,
            descriptor,
            schema_bytes,
            &admissions,
            storage,
            &configuration.event_hub_endpoint,
            configuration.event_credential_revision,
        ))
        .map_err(|error| {
            developer_diagnostic(&format!("developer_mail_admission_error={error:?}"));
            "Mail runtime admission was rejected".to_owned()
        })?;
    let mut gmail_oauth_provider_operations =
        BTreeMap::<String, tokio::task::JoinHandle<CompletedGmailOAuthProviderOperationV1>>::new();
    let mut imap_sync_provider_operations =
        BTreeMap::<String, ActiveImapSyncProviderOperationV1>::new();
    let mut gmail_sync_provider_operations =
        BTreeMap::<String, ActiveGmailSyncProviderOperationV1>::new();
    let mut delivery_provider_operations =
        BTreeMap::<String, ActiveMailDeliveryProviderOperationV1>::new();
    let mut message_flag_provider_operations =
        BTreeMap::<String, ActiveMailMessageFlagProviderOperationV1>::new();
    let mut message_location_provider_operations =
        BTreeMap::<String, ActiveMailMessageLocationProviderOperationV1>::new();
    let mut message_permanent_delete_provider_operations =
        BTreeMap::<String, ActiveMailMessagePermanentDeleteProviderOperationV1>::new();
    let mut replay_indexed_at_unix_seconds = 0;
    'runtime: loop {
        drain_client_deliveries(&runtime, &mut admitted)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Mail runtime clock is unavailable".to_owned())?;
        let now = i64::try_from(now.as_secs())
            .map_err(|_| "Mail runtime clock is unavailable".to_owned())?;
        if now > replay_indexed_at_unix_seconds {
            match runtime.block_on(admitted.index_retained_attachment_scan_candidates(now)) {
                Ok(_) => replay_indexed_at_unix_seconds = now,
                Err(RetainedMailReplayErrorV1::StorageUnavailable) => {}
                Err(error) => {
                    developer_diagnostic(&format!("developer_mail_replay_index_error={error:?}"));
                    return Err("Mail retained evidence replay index failed".to_owned());
                }
            }
        }
        expire_pending_sync_operations(&runtime, &mut admitted, now)?;
        expire_active_gmail_sync_operation(
            &runtime,
            &mut admitted,
            &mut gmail_sync_provider_operations,
            now,
        )?;
        expire_active_imap_sync_operation(
            &runtime,
            &mut admitted,
            &mut imap_sync_provider_operations,
            now,
        )?;
        drain_client_deliveries(&runtime, &mut admitted)?;
        let finished_gmail_oauth = gmail_oauth_provider_operations
            .iter()
            .filter(|(_, operation)| operation.is_finished())
            .map(|(connection_id, _)| connection_id.clone())
            .collect::<Vec<_>>();
        for active_connection_id in finished_gmail_oauth {
            let operation = gmail_oauth_provider_operations
                .remove(&active_connection_id)
                .expect("finished Gmail OAuth provider operation");
            let completed = runtime
                .block_on(operation)
                .map_err(|_| "Mail runtime Gmail OAuth provider worker failed".to_owned())?;
            let connection_id = completed.connection_id().to_owned();
            if connection_id != active_connection_id {
                return Err("Mail runtime Gmail OAuth account binding is invalid".to_owned());
            }
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime Gmail OAuth account selection failed".to_owned())?;
            handle_gmail_oauth_dispatch_result(
                runtime.block_on(admitted.finalize_gmail_oauth_provider_operation(completed, now)),
            )?;
        }
        for connection_id in admitted.connection_ids() {
            if gmail_oauth_provider_operations.contains_key(&connection_id) {
                continue;
            }
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime Gmail OAuth account selection failed".to_owned())?;
            match runtime.block_on(admitted.prepare_next_gmail_oauth_provider_operation(now, now)) {
                Ok(Some(prepared)) => {
                    gmail_oauth_provider_operations.insert(
                        connection_id,
                        runtime.spawn(execute_gmail_oauth_provider_operation(prepared)),
                    );
                }
                Ok(None) => {}
                Err(error) => handle_gmail_oauth_dispatch_result(Err(error))?,
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        for operation in imap_sync_provider_operations.values_mut() {
            let Some(pages) = operation.pages.as_ref() else {
                continue;
            };
            for _ in 0..MAX_SYNC_PAGES_PER_ACCOUNT_PER_TICK {
                let Ok(delivery) = pages.try_recv() else {
                    break;
                };
                let connection_id = delivery.connection_id().to_owned();
                admitted
                    .select_account(&connection_id)
                    .map_err(|_| "Mail runtime IMAP sync account selection failed".to_owned())?;
                match runtime.block_on(admitted.finalize_imap_sync_provider_page(delivery)) {
                    Ok(changed) if changed > 0 => admitted
                        .mark_operational_projection_changed(&connection_id)
                        .map_err(|_| "Mail operational realtime state is invalid".to_owned())?,
                    Ok(_) => {}
                    Err(error) => developer_diagnostic(&format!(
                        "developer_mail_imap_sync_page_error={error:?}"
                    )),
                }
            }
        }
        let finished_imap_sync = imap_sync_provider_operations
            .iter()
            .filter(|(_, operation)| !operation.timed_out && operation.completion.is_finished())
            .map(|(connection_id, _)| connection_id.clone())
            .collect::<Vec<_>>();
        for active_connection_id in finished_imap_sync {
            let operation = imap_sync_provider_operations
                .remove(&active_connection_id)
                .expect("finished IMAP sync provider operation");
            let completed = runtime
                .block_on(operation.completion)
                .map_err(|_| "Mail runtime IMAP sync provider worker failed".to_owned())?;
            if !operation.timed_out {
                let connection_id = completed.connection_id().to_owned();
                if connection_id != active_connection_id {
                    return Err("Mail runtime IMAP sync account binding is invalid".to_owned());
                }
                admitted
                    .select_account(&connection_id)
                    .map_err(|_| "Mail runtime IMAP sync account selection failed".to_owned())?;
                if let Err(error) =
                    runtime.block_on(admitted.finalize_imap_sync_provider_operation(completed, now))
                {
                    developer_diagnostic(&format!("developer_mail_imap_sync_error={error:?}"));
                }
            }
        }
        for connection_id in admitted.connection_ids() {
            if imap_sync_provider_operations.contains_key(&connection_id) {
                continue;
            }
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime sync account selection failed".to_owned())?;
            match runtime.block_on(admitted.prepare_pending_imap_sync()) {
                Ok(Some(prepared)) => {
                    let prepared_connection_id = prepared.connection_id().to_owned();
                    if prepared_connection_id != connection_id {
                        return Err("Mail runtime IMAP sync account binding is invalid".to_owned());
                    }
                    let operation_id = prepared.operation_id().to_owned();
                    let deadline_at_unix_seconds = prepared.deadline_at_unix_seconds();
                    let (page_sender, pages) = std::sync::mpsc::sync_channel(1);
                    imap_sync_provider_operations.insert(
                        connection_id.clone(),
                        ActiveImapSyncProviderOperationV1 {
                            completion: runtime.spawn_blocking(move || {
                                execute_imap_sync_provider_operation(prepared, page_sender)
                            }),
                            pages: Some(pages),
                            connection_id,
                            operation_id,
                            deadline_at_unix_seconds,
                            timed_out: false,
                        },
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    developer_diagnostic(&format!(
                        "developer_mail_imap_sync_prepare_error={error:?}"
                    ));
                    return Err("Mail runtime IMAP sync preparation failed".to_owned());
                }
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        for operation in gmail_sync_provider_operations.values_mut() {
            for _ in 0..MAX_SYNC_PAGES_PER_ACCOUNT_PER_TICK {
                let Ok(delivery) = operation.pages.try_recv() else {
                    break;
                };
                let connection_id = delivery.connection_id().to_owned();
                admitted
                    .select_account(&connection_id)
                    .map_err(|_| "Mail runtime Gmail sync account selection failed".to_owned())?;
                match runtime.block_on(admitted.finalize_gmail_sync_provider_page(delivery)) {
                    Ok(changed) if changed > 0 => admitted
                        .mark_operational_projection_changed(&connection_id)
                        .map_err(|_| "Mail operational realtime state is invalid".to_owned())?,
                    Ok(_) => {}
                    Err(error) => developer_diagnostic(&format!(
                        "developer_mail_gmail_sync_page_error={error:?}"
                    )),
                }
            }
        }
        let finished_gmail_sync = gmail_sync_provider_operations
            .iter()
            .filter(|(_, operation)| !operation.timed_out && operation.completion.is_finished())
            .map(|(connection_id, _)| connection_id.clone())
            .collect::<Vec<_>>();
        for active_connection_id in finished_gmail_sync {
            let operation = gmail_sync_provider_operations
                .remove(&active_connection_id)
                .expect("finished Gmail sync provider operation");
            let completed = runtime
                .block_on(operation.completion)
                .map_err(|_| "Mail runtime Gmail sync provider worker failed".to_owned())?;
            let connection_id = completed.connection_id().to_owned();
            if connection_id != active_connection_id {
                return Err("Mail runtime Gmail sync account binding is invalid".to_owned());
            }
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime Gmail sync account selection failed".to_owned())?;
            if let Err(error) =
                runtime.block_on(admitted.finalize_gmail_sync_provider_operation(completed, now))
            {
                developer_diagnostic(&format!("developer_mail_gmail_sync_error={error:?}"));
            }
        }
        for connection_id in admitted.connection_ids() {
            if gmail_sync_provider_operations.contains_key(&connection_id) {
                continue;
            }
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime Gmail sync account selection failed".to_owned())?;
            match runtime.block_on(admitted.prepare_pending_gmail_sync()) {
                Ok(Some(prepared)) => {
                    let prepared_connection_id = prepared.connection_id().to_owned();
                    if prepared_connection_id != connection_id {
                        return Err("Mail runtime Gmail sync account binding is invalid".to_owned());
                    }
                    let operation_id = prepared.operation_id().to_owned();
                    let deadline_at_unix_seconds = prepared.deadline_at_unix_seconds();
                    let (page_sender, pages) = tokio::sync::mpsc::channel(1);
                    gmail_sync_provider_operations.insert(
                        connection_id.clone(),
                        ActiveGmailSyncProviderOperationV1 {
                            completion: runtime.spawn(execute_gmail_sync_provider_operation(
                                prepared,
                                page_sender,
                            )),
                            pages,
                            connection_id,
                            operation_id,
                            deadline_at_unix_seconds,
                            timed_out: false,
                        },
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    developer_diagnostic(&format!(
                        "developer_mail_gmail_sync_prepare_error={error:?}"
                    ));
                    return Err("Mail runtime Gmail sync preparation failed".to_owned());
                }
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        reconcile_delivery_provider_operations(
            &runtime,
            &admitted,
            &mut delivery_provider_operations,
        )?;
        reconcile_message_flag_provider_operations(
            &runtime,
            &admitted,
            &mut message_flag_provider_operations,
        )?;
        reconcile_message_location_provider_operations(
            &runtime,
            &admitted,
            &mut message_location_provider_operations,
        )?;
        reconcile_message_permanent_delete_provider_operations(
            &runtime,
            &admitted,
            &mut message_permanent_delete_provider_operations,
        )?;
        for connection_id in admitted.connection_ids() {
            admitted
                .select_account(&connection_id)
                .map_err(|_| "Mail runtime account selection failed".to_owned())?;
            if !delivery_provider_operations.contains_key(&connection_id) {
                match runtime.block_on(admitted.prepare_next_delivery_provider_operation(now, now))
                {
                    Ok(Some(prepared)) => {
                        let fence_epoch = prepared.fence_epoch;
                        delivery_provider_operations.insert(
                            connection_id.clone(),
                            ActiveMailDeliveryProviderOperationV1 {
                                completion: runtime
                                    .spawn(execute_mail_delivery_provider_operation(prepared)),
                                fence_epoch,
                            },
                        );
                    }
                    Ok(None) => {}
                    Err(error) => handle_mail_delivery_result(Err(error))?,
                }
            }
            let message_mutation_busy = message_flag_provider_operations
                .contains_key(&connection_id)
                || message_location_provider_operations.contains_key(&connection_id)
                || message_permanent_delete_provider_operations.contains_key(&connection_id);
            let mut message_mutation_scheduled = message_mutation_busy;
            if !message_mutation_scheduled {
                match runtime.block_on(admitted.prepare_next_message_flag_provider_operation(now)) {
                    Ok(Some(prepared)) => {
                        let fence_epoch = prepared.fence_epoch;
                        message_flag_provider_operations.insert(
                            connection_id.clone(),
                            ActiveMailMessageFlagProviderOperationV1 {
                                completion: runtime
                                    .spawn(execute_mail_message_flag_provider_operation(prepared)),
                                fence_epoch,
                            },
                        );
                        message_mutation_scheduled = true;
                    }
                    Ok(None) => {}
                    Err(error) => handle_mail_message_flag_result(Err(error))?,
                }
            }
            if !message_mutation_scheduled {
                match runtime
                    .block_on(admitted.prepare_next_message_location_provider_operation(now))
                {
                    Ok(Some(prepared)) => {
                        let fence_epoch = prepared.fence_epoch;
                        message_location_provider_operations.insert(
                            connection_id.clone(),
                            ActiveMailMessageLocationProviderOperationV1 {
                                completion: runtime.spawn(
                                    execute_mail_message_location_provider_operation(prepared),
                                ),
                                fence_epoch,
                            },
                        );
                        message_mutation_scheduled = true;
                    }
                    Ok(None) => {}
                    Err(error) => handle_mail_message_location_result(Err(error))?,
                }
            }
            if !message_mutation_scheduled {
                match runtime.block_on(
                    admitted.prepare_next_message_permanent_delete_provider_operation(now),
                ) {
                    Ok(Some(prepared)) => {
                        let fence_epoch = prepared.fence_epoch;
                        message_permanent_delete_provider_operations.insert(
                            connection_id.clone(),
                            ActiveMailMessagePermanentDeleteProviderOperationV1 {
                                completion: runtime.spawn(
                                    execute_mail_message_permanent_delete_provider_operation(
                                        prepared,
                                    ),
                                ),
                                fence_epoch,
                            },
                        );
                    }
                    Ok(None) => {}
                    Err(error) => handle_mail_message_permanent_delete_result(Err(error))?,
                }
            }
            execute_account_queues(&runtime, &mut admitted, now)?;
            drain_client_deliveries(&runtime, &mut admitted)?;
        }
        match runtime.block_on(admitted.try_consume_delivery_intent(now)) {
            Ok(_) | Err(MailDeliveryIntentConsumeErrorV1::Unavailable) => {}
            Err(MailDeliveryIntentConsumeErrorV1::Persistence) => {
                developer_diagnostic("developer_mail_delivery_intent_inbox_persistence_failed");
                defer_runtime_tick();
                continue 'runtime;
            }
            Err(_) => {
                developer_diagnostic("developer_mail_delivery_intent_invalid");
                return Err("Mail delivery-intent command is invalid".to_owned());
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        match runtime.block_on(admitted.try_consume_person_source_fetch(now)) {
            Ok(_) | Err(MailPersonSourceFetchWorkerErrorV1::ProviderUnavailable) => {}
            Err(MailPersonSourceFetchWorkerErrorV1::Persistence)
            | Err(MailPersonSourceFetchWorkerErrorV1::EventUnavailable) => {
                developer_diagnostic("developer_mail_person_source_fetch_retryable");
                defer_runtime_tick();
                continue 'runtime;
            }
            Err(error) => {
                developer_diagnostic(&format!(
                    "developer_mail_person_source_fetch_error={error:?}"
                ));
                return Err("Mail Person-source fetch command is invalid".to_owned());
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        runtime
            .block_on(admitted.try_consume_attachment_anchor_handoff(now))
            .map_err(|_| {
                developer_diagnostic("developer_mail_attachment_anchor_handoff_failed");
                "Mail runtime attachment-anchor handoff failed".to_owned()
            })?;
        runtime
            .block_on(admitted.try_consume_attachment_safety_state(now))
            .map_err(|_| {
                developer_diagnostic("developer_mail_attachment_safety_projection_failed");
                "Mail runtime attachment safety projection failed".to_owned()
            })?;
        drain_client_deliveries(&runtime, &mut admitted)?;
        match runtime.block_on(admitted.try_consume_replay_command(now)) {
            Ok(_)
            | Err(MailReplayCommandConsumeErrorV1::EventUnavailable)
            | Err(MailReplayCommandConsumeErrorV1::ReplayRetryable)
            | Err(MailReplayCommandConsumeErrorV1::Persistence(
                makosh_mail_retained_evidence_replay_persistence::RetainedMailReplayErrorV1::StorageUnavailable,
            )) => {}
            Err(error) => {
                developer_diagnostic(&format!("developer_mail_replay_command_error={error:?}"));
                return Err("Mail replay command is invalid".to_owned());
            }
        }
        match runtime.block_on(admitted.relay_communications_outbox(now)) {
            Ok(_) => {}
            Err(MailCommunicationsOutboxRelayError::Unavailable) => {
                developer_diagnostic("developer_mail_outbox_relay_unavailable");
            }
            Err(MailCommunicationsOutboxRelayError::Persistence(_)) => {
                developer_diagnostic("developer_mail_outbox_persistence_failed");
                defer_runtime_tick();
                continue 'runtime;
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        match runtime.block_on(admitted.relay_delivery_intent_outbox(now)) {
            Ok(_) | Err(MailDeliveryIntentOutboxRelayErrorV1::Unavailable) => {}
            Err(MailDeliveryIntentOutboxRelayErrorV1::Persistence(_)) => {
                developer_diagnostic("developer_mail_delivery_intent_outbox_persistence_failed");
                defer_runtime_tick();
                continue 'runtime;
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        let now_unix_millis = now
            .checked_mul(1_000)
            .ok_or_else(|| "Mail runtime clock overflow".to_owned())?;
        match runtime.block_on(admitted.relay_person_source_lifecycle_outbox(now_unix_millis)) {
            Ok(_) | Err(makosh_mail_runtime::person_source_producer::MailPersonSourceProducerErrorV1::EventUnavailable) => {}
            Err(makosh_mail_runtime::person_source_producer::MailPersonSourceProducerErrorV1::PersistenceUnavailable) => {
                developer_diagnostic("developer_mail_person_source_lifecycle_persistence_failed");
                defer_runtime_tick();
                continue 'runtime;
            }
            Err(_) => return Err("Mail Person-source lifecycle outbox is invalid".to_owned()),
        }
        match runtime.block_on(admitted.relay_person_source_fetch_outbox(now_unix_millis)) {
            Ok(_) | Err(MailPersonSourceFetchWorkerErrorV1::EventUnavailable) => {}
            Err(MailPersonSourceFetchWorkerErrorV1::Persistence) => {
                developer_diagnostic(
                    "developer_mail_person_source_fetch_outbox_persistence_failed",
                );
                defer_runtime_tick();
                continue 'runtime;
            }
            Err(_) => return Err("Mail Person-source fetch outbox is invalid".to_owned()),
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        match runtime.block_on(admitted.relay_attachment_security_outbox(now)) {
            Ok(_) => {}
            Err(MailAttachmentSecurityOutboxRelayError::Unavailable) => {
                developer_diagnostic("developer_mail_attachment_security_outbox_relay_unavailable");
            }
            Err(MailAttachmentSecurityOutboxRelayError::Persistence(_)) => {
                developer_diagnostic(
                    "developer_mail_attachment_security_outbox_persistence_failed",
                );
                defer_runtime_tick();
                continue 'runtime;
            }
        }
        drain_client_deliveries(&runtime, &mut admitted)?;
        match runtime.block_on(admitted.relay_replay_result(now)) {
            Ok(_)
            | Err(MailReplayResultRelayErrorV1::EventUnavailable)
            | Err(MailReplayResultRelayErrorV1::Persistence(
                makosh_mail_retained_evidence_replay_persistence::RetainedMailReplayErrorV1::StorageUnavailable,
            )) => {}
            Err(error) => {
                developer_diagnostic(&format!("developer_mail_replay_result_error={error:?}"));
                return Err("Mail replay result relay failed".to_owned());
            }
        }
        admitted
            .publish_pending_operational_realtime(
                u64::try_from(now_unix_millis)
                    .map_err(|_| "Mail runtime clock is unavailable".to_owned())?,
            )
            .map_err(|_| "Mail operational realtime publication failed".to_owned())?;
        std::thread::sleep(RUNTIME_TICK_INTERVAL);
    }
}

fn expire_pending_sync_operations(
    runtime: &tokio::runtime::Runtime,
    admitted: &mut managed::MailAdmittedRuntime,
    now_unix_seconds: i64,
) -> Result<(), String> {
    for connection_id in admitted.connection_ids() {
        admitted
            .select_account(&connection_id)
            .map_err(|_| "Mail runtime sync account selection failed".to_owned())?;
        runtime
            .block_on(admitted.expire_pending_sync_operation(now_unix_seconds))
            .map_err(|_| "Mail runtime pending sync expiration failed".to_owned())?;
    }
    Ok(())
}

fn expire_active_gmail_sync_operation(
    runtime: &tokio::runtime::Runtime,
    admitted: &mut managed::MailAdmittedRuntime,
    operations: &mut BTreeMap<String, ActiveGmailSyncProviderOperationV1>,
    now_unix_seconds: i64,
) -> Result<(), String> {
    let expired = operations
        .iter()
        .filter(|(_, active)| now_unix_seconds >= active.deadline_at_unix_seconds)
        .map(|(connection_id, _)| connection_id.clone())
        .collect::<Vec<_>>();
    for connection_id in expired {
        let active = operations
            .get_mut(&connection_id)
            .expect("expired Gmail sync provider operation");
        if !active.timed_out {
            active.completion.abort();
            active.timed_out = true;
        }
        admitted
            .select_account(&active.connection_id)
            .map_err(|_| "Mail runtime Gmail sync account selection failed".to_owned())?;
        match runtime.block_on(
            admitted.expire_sync_operation(&active.operation_id, active.deadline_at_unix_seconds),
        ) {
            Ok(()) => {
                operations.remove(&connection_id);
            }
            Err(error) => developer_diagnostic(&format!(
                "developer_mail_gmail_sync_expiration_error={error:?}"
            )),
        }
    }
    Ok(())
}

fn expire_active_imap_sync_operation(
    runtime: &tokio::runtime::Runtime,
    admitted: &mut managed::MailAdmittedRuntime,
    operations: &mut BTreeMap<String, ActiveImapSyncProviderOperationV1>,
    now_unix_seconds: i64,
) -> Result<(), String> {
    let expired = operations
        .iter()
        .filter(|(_, active)| now_unix_seconds >= active.deadline_at_unix_seconds)
        .map(|(connection_id, _)| connection_id.clone())
        .collect::<Vec<_>>();
    for connection_id in expired {
        let active = operations
            .get_mut(&connection_id)
            .expect("expired IMAP sync provider operation");
        if !active.timed_out {
            active.pages = None;
            active.timed_out = true;
        }
        admitted
            .select_account(&active.connection_id)
            .map_err(|_| "Mail runtime IMAP sync account selection failed".to_owned())?;
        match runtime.block_on(
            admitted.expire_sync_operation(&active.operation_id, active.deadline_at_unix_seconds),
        ) {
            Ok(()) => {
                operations.remove(&connection_id);
            }
            Err(error) => developer_diagnostic(&format!(
                "developer_mail_imap_sync_expiration_error={error:?}"
            )),
        }
    }
    Ok(())
}

fn drain_client_deliveries(
    runtime: &tokio::runtime::Runtime,
    admitted: &mut managed::MailAdmittedRuntime,
) -> Result<(), String> {
    for _ in 0..MAX_CLIENT_DELIVERIES_PER_TICK {
        match runtime.block_on(admitted.try_handle_client_delivery()) {
            Ok(true) => {}
            Ok(false) => return Ok(()),
            Err(error) => {
                developer_diagnostic(&format!("developer_mail_client_delivery_error={error:?}"));
                return Err("Mail runtime client delivery failed".to_owned());
            }
        }
    }
    Ok(())
}

fn execute_account_queues(
    runtime: &tokio::runtime::Runtime,
    admitted: &mut managed::MailAdmittedRuntime,
    now: i64,
) -> Result<(), String> {
    match runtime.block_on(process_next_mail_delivery_intent_v1(admitted, now)) {
        Ok(_) => {}
        Err(MailDeliveryIntentWorkerErrorV1::InvalidClock) => {
            return Err("Mail delivery-intent worker clock is invalid".to_owned());
        }
        Err(MailDeliveryIntentWorkerErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_delivery_intent_persistence_failed");
            return Ok(());
        }
        Err(MailDeliveryIntentWorkerErrorV1::Runtime) => {
            developer_diagnostic("developer_mail_delivery_intent_runtime_failed");
            return Err("Mail delivery-intent runtime status failed".to_owned());
        }
        Err(MailDeliveryIntentWorkerErrorV1::ResultEnvelope) => {
            developer_diagnostic("developer_mail_delivery_intent_result_invalid");
            return Err("Mail delivery-intent result envelope is invalid".to_owned());
        }
    }
    Ok(())
}

fn reconcile_delivery_provider_operations(
    runtime: &tokio::runtime::Runtime,
    admitted: &managed::MailAdmittedRuntime,
    active: &mut BTreeMap<String, ActiveMailDeliveryProviderOperationV1>,
) -> Result<(), String> {
    let stale = active
        .iter()
        .filter_map(|(connection_id, operation)| {
            admitted
                .provider_io_epoch(connection_id)
                .ok()
                .filter(|epoch| *epoch != operation.fence_epoch)
                .map(|_| connection_id.clone())
        })
        .collect::<Vec<_>>();
    for connection_id in stale {
        if let Some(operation) = active.remove(&connection_id) {
            operation.completion.abort();
        }
    }
    let finished = active
        .iter()
        .filter(|(_, operation)| operation.completion.is_finished())
        .map(|(connection_id, _)| connection_id.clone())
        .collect::<Vec<_>>();
    for connection_id in finished {
        let operation = active
            .remove(&connection_id)
            .expect("finished Mail delivery provider operation");
        let result = runtime
            .block_on(operation.completion)
            .map_err(|_| "Mail delivery provider worker failed".to_owned())?;
        handle_mail_delivery_result(result)?;
    }
    Ok(())
}

fn handle_mail_delivery_result(
    result: Result<bool, MailDeliveryDispatchErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_) => Ok(()),
        Err(MailDeliveryDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_delivery_rejected");
            Ok(())
        }
        Err(MailDeliveryDispatchErrorV1::AttachmentRejected) => {
            developer_diagnostic("developer_mail_delivery_attachment_rejected");
            Ok(())
        }
        Err(MailDeliveryDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_delivery_outcome_unknown");
            Ok(())
        }
        Err(MailDeliveryDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_delivery_command_invalid");
            Err("Mail runtime delivery command is invalid".to_owned())
        }
        Err(MailDeliveryDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_delivery_persistence_failed");
            Ok(())
        }
    }
}

fn reconcile_message_flag_provider_operations(
    runtime: &tokio::runtime::Runtime,
    admitted: &managed::MailAdmittedRuntime,
    active: &mut BTreeMap<String, ActiveMailMessageFlagProviderOperationV1>,
) -> Result<(), String> {
    let stale = active
        .iter()
        .filter_map(|(connection_id, operation)| {
            admitted
                .provider_io_epoch(connection_id)
                .ok()
                .filter(|epoch| *epoch != operation.fence_epoch)
                .map(|_| connection_id.clone())
        })
        .collect::<Vec<_>>();
    for connection_id in stale {
        if let Some(operation) = active.remove(&connection_id) {
            operation.completion.abort();
        }
    }
    let finished = active
        .iter()
        .filter(|(_, operation)| operation.completion.is_finished())
        .map(|(connection_id, _)| connection_id.clone())
        .collect::<Vec<_>>();
    for connection_id in finished {
        let operation = active
            .remove(&connection_id)
            .expect("finished Mail message flag provider operation");
        let result = runtime
            .block_on(operation.completion)
            .map_err(|_| "Mail message flag provider worker failed".to_owned())?;
        handle_mail_message_flag_result(result)?;
    }
    Ok(())
}

fn handle_mail_message_flag_result(
    result: Result<bool, MailMessageFlagDispatchErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_) => Ok(()),
        Err(MailMessageFlagDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_message_flag_rejected");
            Ok(())
        }
        Err(MailMessageFlagDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_message_flag_outcome_unknown");
            Ok(())
        }
        Err(MailMessageFlagDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_message_flag_command_invalid");
            Err("Mail runtime message flag command is invalid".to_owned())
        }
        Err(MailMessageFlagDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_message_flag_persistence_failed");
            Ok(())
        }
    }
}

fn reconcile_message_location_provider_operations(
    runtime: &tokio::runtime::Runtime,
    admitted: &managed::MailAdmittedRuntime,
    active: &mut BTreeMap<String, ActiveMailMessageLocationProviderOperationV1>,
) -> Result<(), String> {
    let stale = active
        .iter()
        .filter_map(|(connection_id, operation)| {
            admitted
                .provider_io_epoch(connection_id)
                .ok()
                .filter(|epoch| *epoch != operation.fence_epoch)
                .map(|_| connection_id.clone())
        })
        .collect::<Vec<_>>();
    for connection_id in stale {
        if let Some(operation) = active.remove(&connection_id) {
            operation.completion.abort();
        }
    }
    let finished = active
        .iter()
        .filter(|(_, operation)| operation.completion.is_finished())
        .map(|(connection_id, _)| connection_id.clone())
        .collect::<Vec<_>>();
    for connection_id in finished {
        let operation = active
            .remove(&connection_id)
            .expect("finished Mail message location provider operation");
        let result = runtime
            .block_on(operation.completion)
            .map_err(|_| "Mail message location provider worker failed".to_owned())?;
        handle_mail_message_location_result(result)?;
    }
    Ok(())
}

fn handle_mail_message_location_result(
    result: Result<bool, MailMessageLocationDispatchErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_) => Ok(()),
        Err(MailMessageLocationDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_message_location_rejected");
            Ok(())
        }
        Err(MailMessageLocationDispatchErrorV1::ProviderUnsupported) => {
            developer_diagnostic("developer_mail_message_location_unsupported");
            Ok(())
        }
        Err(MailMessageLocationDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_message_location_outcome_unknown");
            Ok(())
        }
        Err(MailMessageLocationDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_message_location_command_invalid");
            Err("Mail runtime message location command is invalid".to_owned())
        }
        Err(MailMessageLocationDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_message_location_persistence_failed");
            Ok(())
        }
    }
}

fn reconcile_message_permanent_delete_provider_operations(
    runtime: &tokio::runtime::Runtime,
    admitted: &managed::MailAdmittedRuntime,
    active: &mut BTreeMap<String, ActiveMailMessagePermanentDeleteProviderOperationV1>,
) -> Result<(), String> {
    let stale = active
        .iter()
        .filter_map(|(connection_id, operation)| {
            admitted
                .provider_io_epoch(connection_id)
                .ok()
                .filter(|epoch| *epoch != operation.fence_epoch)
                .map(|_| connection_id.clone())
        })
        .collect::<Vec<_>>();
    for connection_id in stale {
        if let Some(operation) = active.remove(&connection_id) {
            operation.completion.abort();
        }
    }
    let finished = active
        .iter()
        .filter(|(_, operation)| operation.completion.is_finished())
        .map(|(connection_id, _)| connection_id.clone())
        .collect::<Vec<_>>();
    for connection_id in finished {
        let operation = active
            .remove(&connection_id)
            .expect("finished Mail permanent delete provider operation");
        let result = runtime
            .block_on(operation.completion)
            .map_err(|_| "Mail permanent delete provider worker failed".to_owned())?;
        handle_mail_message_permanent_delete_result(result)?;
    }
    Ok(())
}

fn handle_mail_message_permanent_delete_result(
    result: Result<bool, MailMessagePermanentDeleteDispatchErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(_) => Ok(()),
        Err(MailMessagePermanentDeleteDispatchErrorV1::ProviderRejected) => {
            developer_diagnostic("developer_mail_message_permanent_delete_rejected");
            Ok(())
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::ProviderUnsupported) => {
            developer_diagnostic("developer_mail_message_permanent_delete_unsupported");
            Ok(())
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::ReauthorizationRequired) => {
            developer_diagnostic(
                "developer_mail_message_permanent_delete_reauthorization_required",
            );
            Ok(())
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::ProviderOutcomeUnknown) => {
            developer_diagnostic("developer_mail_message_permanent_delete_outcome_unknown");
            Ok(())
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::InvalidStoredCommand) => {
            developer_diagnostic("developer_mail_message_permanent_delete_command_invalid");
            Err("Mail runtime permanent delete command is invalid".to_owned())
        }
        Err(MailMessagePermanentDeleteDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_message_permanent_delete_persistence_failed");
            Ok(())
        }
    }
}

fn defer_runtime_tick() {
    std::thread::sleep(RUNTIME_TICK_INTERVAL);
}

fn developer_diagnostic(message: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("{message}");
    }
}

fn handle_gmail_oauth_dispatch_result(
    result: Result<(), MailGmailOAuthDispatchErrorV1>,
) -> Result<(), String> {
    match result {
        Ok(()) => Ok(()),
        Err(MailGmailOAuthDispatchErrorV1::Rejected) => {
            developer_diagnostic("developer_mail_gmail_oauth_rejected");
            Ok(())
        }
        Err(MailGmailOAuthDispatchErrorV1::OutcomeUnknown) => {
            developer_diagnostic("developer_mail_gmail_oauth_outcome_unknown");
            Ok(())
        }
        Err(MailGmailOAuthDispatchErrorV1::InvalidStoredOperation) => {
            developer_diagnostic("developer_mail_gmail_oauth_operation_invalid");
            Err("Mail runtime Gmail OAuth operation is invalid".to_owned())
        }
        Err(MailGmailOAuthDispatchErrorV1::Persistence) => {
            developer_diagnostic("developer_mail_gmail_oauth_persistence_failed");
            Ok(())
        }
    }
}

fn parse_paths<I>(arguments: &mut std::iter::Peekable<I>) -> Result<InheritedPaths, String>
where
    I: Iterator<Item = OsString>,
{
    let descriptor = required_path(arguments, "--descriptor-path")?;
    let settings_schema = required_path(arguments, "--settings-schema-path")?;
    let settings_snapshot = required_path(arguments, "--settings-snapshot-path")?;
    let runtime_configuration = required_path(arguments, "--runtime-configuration-path")?;
    let runtime_instance_id = required_string(arguments, "--runtime-instance-id")?;
    if arguments.next().is_some() || runtime_instance_id.trim().is_empty() {
        return Err("Mail runtime arguments are invalid".to_owned());
    }
    Ok(InheritedPaths {
        descriptor,
        settings_schema,
        settings_snapshot,
        runtime_configuration,
        runtime_instance_id,
    })
}

fn required_path<I>(arguments: &mut I, name: &str) -> Result<PathBuf, String>
where
    I: Iterator<Item = OsString>,
{
    required_string(arguments, name).map(PathBuf::from)
}

fn required_string<I>(arguments: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = OsString>,
{
    if arguments.next().as_deref() != Some(OsStr::new(name)) {
        return Err("Mail runtime arguments are invalid".to_owned());
    }
    arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| "Mail runtime arguments are invalid".to_owned())
}

fn inherited_control_channel() -> Result<UnixStream, String> {
    let duplicated = unsafe { libc::dup(std::io::stdin().as_raw_fd()) };
    if duplicated < 0 {
        return Err("Mail runtime inherited control channel is unavailable".to_owned());
    }
    Ok(unsafe { UnixStream::from_raw_fd(duplicated) })
}

#[cfg(test)]
mod provider_job_tests {
    use super::*;

    #[test]
    fn pending_provider_jobs_for_two_accounts_do_not_own_the_catalog_actor() {
        let runtime = tokio::runtime::Runtime::new().expect("runtime executor");
        let (_release_delivery, pending_delivery) = tokio::sync::oneshot::channel::<()>();
        let (_release_flag, pending_flag) = tokio::sync::oneshot::channel::<()>();
        let (_release_location, pending_location) = tokio::sync::oneshot::channel::<()>();
        let (_release_delete, pending_delete) = tokio::sync::oneshot::channel::<()>();
        let delivery = ActiveMailDeliveryProviderOperationV1 {
            completion: runtime.spawn(async move {
                let _ = pending_delivery.await;
                Ok(true)
            }),
            fence_epoch: 3,
        };
        let flag = ActiveMailMessageFlagProviderOperationV1 {
            completion: runtime.spawn(async move {
                let _ = pending_flag.await;
                Ok(true)
            }),
            fence_epoch: 9,
        };
        let location = ActiveMailMessageLocationProviderOperationV1 {
            completion: runtime.spawn(async move {
                let _ = pending_location.await;
                Ok(true)
            }),
            fence_epoch: 5,
        };
        let delete = ActiveMailMessagePermanentDeleteProviderOperationV1 {
            completion: runtime.spawn(async move {
                let _ = pending_delete.await;
                Ok(true)
            }),
            fence_epoch: 7,
        };

        assert!(!delivery.completion.is_finished());
        assert!(!flag.completion.is_finished());
        assert!(!location.completion.is_finished());
        assert!(!delete.completion.is_finished());
        assert_eq!(
            runtime.block_on(async { "client-query-served" }),
            "client-query-served"
        );

        delivery.completion.abort();
        flag.completion.abort();
        location.completion.abort();
        delete.completion.abort();
        assert!(
            runtime
                .block_on(delivery.completion)
                .expect_err("delivery abort")
                .is_cancelled()
        );
        assert!(
            runtime
                .block_on(location.completion)
                .expect_err("location abort")
                .is_cancelled()
        );
        assert!(
            runtime
                .block_on(delete.completion)
                .expect_err("delete abort")
                .is_cancelled()
        );
        assert!(
            runtime
                .block_on(flag.completion)
                .expect_err("flag abort")
                .is_cancelled()
        );
    }
}

fn read_contract(path: &Path) -> Result<Vec<u8>, String> {
    const MAX_CONTRACT_BYTES: u64 = 512 * 1024;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "Mail runtime contract is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err("Mail runtime contract is unavailable".to_owned());
    }
    std::fs::read(path).map_err(|_| "Mail runtime contract is unavailable".to_owned())
}

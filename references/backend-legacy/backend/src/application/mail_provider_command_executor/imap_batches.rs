use makosh_communications_api::commands::{CommunicationProviderCommand, ProviderCommandQueuePort};
use std::collections::BTreeMap;

use chrono::Utc;
use serde_json::{Value, json};

use super::{
    MailProviderCommandExecutionReport, MailProviderCommandWorker, MailProviderCommandWorkerError,
    mutation_for_command, resolve_provider_command_payload,
};
use crate::integrations::mail::read_state::{EmailProviderMessageMutation, imap_source_mailbox};
use makosh_communications_api::accounts::ProviderAccount;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ImapBatchKey {
    command_kind: String,
    source_mailbox: String,
    destination: Option<String>,
}

struct PreparedImapCommand {
    command: CommunicationProviderCommand,
    payload: Value,
    message_metadata: Value,
    desired_is_read: Option<bool>,
}

impl<Q> MailProviderCommandWorker<Q>
where
    Q: ProviderCommandQueuePort,
{
    pub(super) async fn execute_imap_commands(
        &self,
        account: &ProviderAccount,
        commands: Vec<CommunicationProviderCommand>,
        report: &mut MailProviderCommandExecutionReport,
    ) -> Result<(), MailProviderCommandWorkerError> {
        let mut groups = BTreeMap::<ImapBatchKey, Vec<PreparedImapCommand>>::new();
        for command in commands {
            let Some(message_id) = command
                .target_ref
                .get("message_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                self.dead_letter_invalid_command(
                    &command,
                    "mail provider command is missing a message target",
                )
                .await?;
                report.dead_lettered += 1;
                continue;
            };
            let Some(message) = self.message_store.message(message_id).await? else {
                self.dead_letter_invalid_command(
                    &command,
                    "mail provider command target message does not exist",
                )
                .await?;
                report.dead_lettered += 1;
                continue;
            };
            let payload = resolve_provider_command_payload(
                &self.provider_resource_store,
                account,
                &command.command_kind,
                &command.payload,
            )
            .await?;
            let mutation = match mutation_for_command(&command.command_kind, &payload, account) {
                Ok(mutation) => mutation,
                Err(error) => {
                    self.record_mutation_failure(&command, &error, report)
                        .await?;
                    continue;
                }
            };
            let desired_is_read = match mutation {
                EmailProviderMessageMutation::SetRead(value) => Some(value),
                _ => None,
            };
            if let Some(desired_is_read) = desired_is_read
                && message.is_read != desired_is_read
            {
                self.command_queue
                    .mark_completed_at_epoch(
                        &command.command_id,
                        "mail",
                        Utc::now(),
                        json!({
                            "provider_message_id": command.provider_message_id,
                            "desired_is_read": desired_is_read,
                            "superseded": true,
                            "current_is_read": message.is_read,
                        }),
                        command.lease_epoch,
                    )
                    .await?;
                report.completed += 1;
                continue;
            }
            let source_mailbox = match imap_source_mailbox(account, &message.message_metadata) {
                Ok(mailbox) => mailbox,
                Err(error) => {
                    self.record_mutation_failure(&command, &error, report)
                        .await?;
                    continue;
                }
            };
            groups
                .entry(ImapBatchKey {
                    command_kind: command.command_kind.clone(),
                    source_mailbox,
                    destination: mutation_destination(mutation),
                })
                .or_default()
                .push(PreparedImapCommand {
                    command,
                    payload,
                    message_metadata: message.message_metadata,
                    desired_is_read,
                });
        }

        for group in groups.into_values() {
            self.execute_imap_group(account, group, report).await?;
        }
        Ok(())
    }

    async fn execute_imap_group(
        &self,
        account: &ProviderAccount,
        group: Vec<PreparedImapCommand>,
        report: &mut MailProviderCommandExecutionReport,
    ) -> Result<(), MailProviderCommandWorkerError> {
        let first = &group[0];
        let mutation =
            match mutation_for_command(&first.command.command_kind, &first.payload, account) {
                Ok(mutation) => mutation,
                Err(error) => {
                    for item in &group {
                        self.record_mutation_failure(&item.command, &error, report)
                            .await?;
                    }
                    return Ok(());
                }
            };
        let operation = imap_operation(mutation);
        let message_metadata = group
            .iter()
            .map(|item| item.message_metadata.clone())
            .collect::<Vec<_>>();
        let result = self
            .read_state
            .apply_imap_batch_mutation(account, &message_metadata, mutation)
            .await;

        match result {
            Ok(()) => {
                let batch_size = group.len();
                let batched = batch_size > 1;
                for item in group {
                    self.command_queue
                        .mark_completed_at_epoch(
                            &item.command.command_id,
                            "mail",
                            Utc::now(),
                            json!({
                                    "provider_message_id": item.command.provider_message_id,
                                    "desired_is_read": item.desired_is_read,
                                    "command_kind": item.command.command_kind,
                                    "provider_operation": operation,
                                    "imap_batch": batched,
                                "batch_size": batch_size,
                            }),
                            item.command.lease_epoch,
                        )
                        .await?;
                    report.completed += 1;
                }
                if batched {
                    report.provider_batches += 1;
                    report.batched_commands += batch_size;
                }
            }
            Err(error) => {
                for item in &group {
                    self.record_mutation_failure(&item.command, &error, report)
                        .await?;
                }
            }
        }
        Ok(())
    }
}

fn mutation_destination(mutation: EmailProviderMessageMutation<'_>) -> Option<String> {
    let destination = match mutation {
        EmailProviderMessageMutation::Archive {
            destination_mailbox,
        }
        | EmailProviderMessageMutation::Trash {
            destination_mailbox,
        }
        | EmailProviderMessageMutation::MarkSpam {
            destination_mailbox,
        }
        | EmailProviderMessageMutation::UnmarkSpam {
            destination_mailbox,
        } => destination_mailbox,
        EmailProviderMessageMutation::AddLabel(label)
        | EmailProviderMessageMutation::RemoveLabel(label)
        | EmailProviderMessageMutation::MoveTo(label)
        | EmailProviderMessageMutation::CopyTo(label) => Some(label),
        EmailProviderMessageMutation::SetRead(_)
        | EmailProviderMessageMutation::SetImportant(_)
        | EmailProviderMessageMutation::SetStarred(_) => None,
    };
    destination
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn imap_operation(mutation: EmailProviderMessageMutation<'_>) -> &'static str {
    match mutation {
        EmailProviderMessageMutation::SetRead(_)
        | EmailProviderMessageMutation::SetImportant(_)
        | EmailProviderMessageMutation::SetStarred(_)
        | EmailProviderMessageMutation::AddLabel(_)
        | EmailProviderMessageMutation::RemoveLabel(_) => "uid_store",
        EmailProviderMessageMutation::Archive { .. }
        | EmailProviderMessageMutation::Trash { .. }
        | EmailProviderMessageMutation::MarkSpam { .. }
        | EmailProviderMessageMutation::UnmarkSpam { .. }
        | EmailProviderMessageMutation::MoveTo(_) => "uid_move",
        EmailProviderMessageMutation::CopyTo(_) => "uid_copy",
    }
}

#[cfg(test)]
mod tests {
    use crate::platform::secrets::store::SecretReferenceStore;
    use std::io::{BufRead, BufReader, Write};
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use makosh_backend_testkit::context::TestContext;
    use serde_json::json;
    use tempfile::tempdir;

    use super::MailProviderCommandWorker;
    use crate::domains::communications::messages::projection::project_raw_email_message;
    use crate::domains::communications::messages::store::MessageProjectionStore;
    use makosh_communications_api::accounts::{
        CommunicationProviderKind, NewProviderAccount, NewProviderAccountSecretBinding,
        ProviderAccountSecretPurpose,
    };
    use makosh_communications_api::commands::NewCommunicationProviderCommand;
    use makosh_communications_api::evidence::NewRawCommunicationRecord;
    use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
    use makosh_communications_postgres::provider_store::{
        CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
    };
    use makosh_communications_postgres::store::CommunicationIngestionStore;

    use crate::platform::secrets::models::{NewSecretReference, SecretKind, SecretStoreKind};
    use crate::vault::HostVault;
    use crate::vault::models::{EntropyEvent, HostVaultConfig, SecretEntryContext};

    #[tokio::test]
    async fn compatible_imap_commands_share_one_uid_store_and_complete_individually() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let server = MockImapBatchServer::start();
        let account_id = "mail-worker-imap-batch";
        let secret_ref = format!("secret:provider-account:{account_id}:imap_password");
        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(
                &NewProviderAccount::new(
                    account_id,
                    CommunicationProviderKind::Imap,
                    "IMAP batch fixture",
                    "batch@example.test",
                )
                .config(json!({
                    "host": "127.0.0.1",
                    "port": server.addr().port(),
                    "tls": false,
                    "username": "batch@example.test",
                    "mailbox": "INBOX"
                })),
            )
            .await
            .expect("store IMAP account");
        SecretReferenceStore::new(pool.clone())
            .upsert_secret_reference(&NewSecretReference::new(
                &secret_ref,
                SecretKind::Password,
                SecretStoreKind::HostVault,
                "IMAP batch credential",
            ))
            .await
            .expect("store IMAP secret reference");
        CommunicationProviderSecretBindingStore::new(pool.clone())
            .bind(&NewProviderAccountSecretBinding::new(
                account_id,
                ProviderAccountSecretPurpose::ImapPassword,
                &secret_ref,
            ))
            .await
            .expect("bind IMAP secret");

        let vault_root = tempdir().expect("vault root");
        let vault = HostVault::new(HostVaultConfig {
            home: vault_root.path().join("vault"),
            dev_mode: true,
            dev_key_path: vault_root.path().join("dev").join("master.key"),
        })
        .expect("host vault");
        vault
            .collect_entropy(vault_entropy_events(2_000))
            .expect("collect vault entropy");
        vault.create().expect("create host vault");
        vault
            .store_secret(
                &secret_ref,
                "imap-batch-password",
                SecretEntryContext {
                    entry_kind: "provider_credential",
                    account_id,
                    purpose: ProviderAccountSecretPurpose::ImapPassword.as_str(),
                    secret_kind: SecretKind::Password.as_str(),
                    label: "IMAP batch credential",
                    metadata: &json!({ "provider": "imap" }),
                },
            )
            .expect("store IMAP password");

        let communication_store = CommunicationIngestionStore::new(pool.clone());
        let message_store = MessageProjectionStore::new(pool.clone());
        let command_store = CommunicationProviderCommandStore::new(pool.clone());
        for (index, uid) in [41_u32, 42_u32].into_iter().enumerate() {
            let fixture_number = index + 1;
            let provider_record_id = uid.to_string();
            let raw = communication_store
                .record_raw_source(&NewRawCommunicationRecord::new(
                    format!("raw-imap-batch-{fixture_number}"),
                    account_id,
                    "email_message",
                    &provider_record_id,
                    format!("sha256:imap-batch-{fixture_number}"),
                    "imap-batch-import",
                    json!({
                        "subject": format!("Batch message {fixture_number}"),
                        "from": "sender@example.test",
                        "to": ["owner@example.test"],
                        "body_text": "Batch mutation fixture",
                        "mailbox": "INBOX",
                        "uid": uid,
                        "uid_validity": 999,
                        "is_read": true
                    }),
                ))
                .await
                .expect("record raw IMAP message");
            let projected = project_raw_email_message(&message_store, &raw)
                .await
                .expect("project IMAP message");
            command_store
                .enqueue(
                    &NewCommunicationProviderCommand::new(
                        format!("imap-batch-command-{fixture_number}"),
                        account_id,
                        "mail",
                        "important",
                        format!("imap-batch-idempotency-{fixture_number}"),
                        "test-actor",
                    )
                    .provider_message_id(&provider_record_id)
                    .target_ref(json!({ "message_id": projected.message_id })),
                )
                .await
                .expect("enqueue IMAP command");
        }

        let worker = MailProviderCommandWorker::new(pool, vault, "http://127.0.0.1:1");
        let report = worker
            .execute_due(chrono::Utc::now(), 10)
            .await
            .expect("execute IMAP batch");

        assert_eq!(report.claimed, 2);
        assert_eq!(report.completed, 2);
        assert_eq!(report.retrying, 0);
        assert_eq!(report.dead_lettered, 0);
        assert_eq!(report.provider_batches, 1);
        assert_eq!(report.batched_commands, 2);

        let commands = server.commands();
        let stores = commands
            .iter()
            .filter(|command| command.to_ascii_uppercase().contains("UID STORE"))
            .collect::<Vec<_>>();
        assert_eq!(stores.len(), 1, "expected one UID STORE: {commands:?}");
        assert!(stores[0].contains("41,42"), "batched UID set: {stores:?}");
        assert!(stores[0].contains("+FLAGS.SILENT (\\Flagged)"));

        let durable_commands = command_store
            .list(account_id, "mail", 10)
            .await
            .expect("list completed commands");
        assert_eq!(durable_commands.len(), 2);
        assert!(durable_commands.iter().all(|command| {
            command.status == "completed"
                && command.result_payload["imap_batch"] == true
                && command.result_payload["batch_size"] == 2
                && command.result_payload["provider_operation"] == "uid_store"
        }));
    }

    fn vault_entropy_events(count: usize) -> Vec<EntropyEvent> {
        (0..count)
            .map(|index| EntropyEvent {
                x: (index % 997) as f64,
                y: (index % 577) as f64,
                dx: (index % 11) as f64 - 5.0,
                dy: (index % 13) as f64 - 6.0,
                timestamp_ms: index as f64 * 5.0,
                velocity: (index % 19) as f64 / 10.0,
                acceleration: (index % 23) as f64 / 100.0,
                interval_ms: 5.0,
            })
            .collect()
    }

    struct MockImapBatchServer {
        addr: SocketAddr,
        commands: Arc<Mutex<Vec<String>>>,
        handle: Option<thread::JoinHandle<()>>,
    }

    impl MockImapBatchServer {
        fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind IMAP batch server");
            let addr = listener.local_addr().expect("IMAP batch server addr");
            let commands = Arc::new(Mutex::new(Vec::new()));
            let commands_for_thread = Arc::clone(&commands);
            let handle = thread::spawn(move || {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .expect("set IMAP timeout");
                stream
                    .write_all(b"* OK IMAP batch fixture ready\r\n")
                    .expect("write IMAP greeting");
                let mut reader = BufReader::new(stream.try_clone().expect("clone IMAP stream"));
                loop {
                    let mut line = String::new();
                    let Ok(bytes) = reader.read_line(&mut line) else {
                        break;
                    };
                    if bytes == 0 {
                        break;
                    }
                    let command = line.trim_end().to_owned();
                    commands_for_thread
                        .lock()
                        .expect("IMAP batch commands lock")
                        .push(command.clone());
                    let Some(tag) = command.split_whitespace().next() else {
                        break;
                    };
                    if command.contains(" LOGIN ") {
                        write!(stream, "{tag} OK LOGIN completed\r\n")
                            .expect("write LOGIN response");
                    } else if command.contains(" SELECT ") {
                        write!(
                            stream,
                            "* FLAGS (\\Seen \\Flagged)\r\n* 2 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 999] UIDs valid\r\n{tag} OK [READ-WRITE] SELECT completed\r\n"
                        )
                        .expect("write SELECT response");
                    } else if command.to_ascii_uppercase().contains("UID STORE") {
                        write!(stream, "{tag} OK STORE completed\r\n")
                            .expect("write STORE response");
                    } else if command.contains(" LOGOUT") {
                        write!(stream, "* BYE done\r\n{tag} OK LOGOUT completed\r\n")
                            .expect("write LOGOUT response");
                        break;
                    } else {
                        write!(stream, "{tag} BAD unexpected command\r\n")
                            .expect("write BAD response");
                    }
                }
            });
            Self {
                addr,
                commands,
                handle: Some(handle),
            }
        }

        fn addr(&self) -> SocketAddr {
            self.addr
        }

        fn commands(&self) -> Vec<String> {
            self.commands
                .lock()
                .expect("IMAP batch commands lock")
                .clone()
        }
    }

    impl Drop for MockImapBatchServer {
        fn drop(&mut self) {
            let _ = TcpStream::connect(self.addr);
            if let Some(handle) = self.handle.take() {
                handle.join().expect("IMAP batch server join");
            }
        }
    }
}

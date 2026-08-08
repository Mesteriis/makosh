use makosh_communications_api::commands::{CommunicationProviderCommand, ProviderCommandQueuePort};
use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use serde_json::json;

use super::{
    MailProviderCommandExecutionReport, MailProviderCommandWorker, MailProviderCommandWorkerError,
    mutation_for_command, resolve_provider_command_payload,
};
use crate::integrations::mail::read_state::{
    EmailProviderMessageMutation, EmailReadStateRequest, gmail_label_ids_for_mutation,
};
use makosh_communications_api::accounts::ProviderAccount;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GmailBatchKey {
    command_kind: String,
    add_labels: Vec<String>,
    remove_labels: Vec<String>,
}

struct PreparedGmailCommand {
    command: CommunicationProviderCommand,
    payload: serde_json::Value,
    provider_record_id: String,
    desired_is_read: Option<bool>,
}

impl<Q> MailProviderCommandWorker<Q>
where
    Q: ProviderCommandQueuePort,
{
    pub(super) async fn execute_gmail_commands(
        &self,
        account: &ProviderAccount,
        commands: Vec<CommunicationProviderCommand>,
        report: &mut MailProviderCommandExecutionReport,
    ) -> Result<(), MailProviderCommandWorkerError> {
        let mut groups = BTreeMap::<GmailBatchKey, Vec<PreparedGmailCommand>>::new();
        for command in commands {
            let Some(message_id) = command
                .target_ref
                .get("message_id")
                .and_then(serde_json::Value::as_str)
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
            let (add_labels, remove_labels) = gmail_label_ids_for_mutation(mutation);
            groups
                .entry(GmailBatchKey {
                    command_kind: command.command_kind.clone(),
                    add_labels: add_labels.into_iter().map(ToOwned::to_owned).collect(),
                    remove_labels: remove_labels.into_iter().map(ToOwned::to_owned).collect(),
                })
                .or_default()
                .push(PreparedGmailCommand {
                    command,
                    payload,
                    provider_record_id: message.provider_record_id,
                    desired_is_read,
                });
        }

        for group in groups.into_values() {
            self.execute_gmail_group(account, group, report).await?;
        }
        Ok(())
    }

    async fn execute_gmail_group(
        &self,
        account: &ProviderAccount,
        group: Vec<PreparedGmailCommand>,
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
        let empty_metadata = json!({});
        let result = if group.len() == 1 {
            self.read_state
                .apply_message_mutation(
                    EmailReadStateRequest {
                        account,
                        provider_record_id: &first.provider_record_id,
                        message_metadata: &empty_metadata,
                    },
                    mutation,
                )
                .await
        } else {
            let provider_record_ids = group
                .iter()
                .map(|item| item.provider_record_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            self.read_state
                .apply_gmail_batch_mutation(account, &provider_record_ids, mutation)
                .await
        };

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
                                    "batch_modify": batched,
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

#[cfg(test)]
mod tests {
    use crate::platform::secrets::store::SecretReferenceStore;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;

    use makosh_backend_testkit::context::TestContext;
    use serde_json::{Value, json};
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
    async fn compatible_gmail_commands_share_one_provider_batch_and_complete_individually() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let server = MockGmailBatchServer::start();
        let account_id = "mail-worker-gmail-batch";
        let secret_ref = format!("secret:provider-account:{account_id}:oauth_token");
        let communication_store = CommunicationIngestionStore::new(pool.clone());
        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(
                &NewProviderAccount::new(
                    account_id,
                    CommunicationProviderKind::Gmail,
                    "Gmail batch fixture",
                    "batch@example.test",
                )
                .config(json!({
                    "auth": "oauth",
                    "api": "gmail",
                    "gmail_api_base_url": server.base_url()
                })),
            )
            .await
            .expect("store Gmail account");
        SecretReferenceStore::new(pool.clone())
            .upsert_secret_reference(&NewSecretReference::new(
                &secret_ref,
                SecretKind::OauthToken,
                SecretStoreKind::HostVault,
                "Gmail batch OAuth credential",
            ))
            .await
            .expect("store OAuth secret reference");
        CommunicationProviderSecretBindingStore::new(pool.clone())
            .bind(&NewProviderAccountSecretBinding::new(
                account_id,
                ProviderAccountSecretPurpose::OauthToken,
                &secret_ref,
            ))
            .await
            .expect("bind OAuth secret");

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
                &json!({
                    "token_url": "http://127.0.0.1:1/token",
                    "client_id": "desktop-client-id",
                    "access_token": "gmail-batch-access-token",
                    "refresh_token": "gmail-batch-refresh-token",
                    "expires_at": "2999-01-01T00:00:00Z",
                    "scope": "https://www.googleapis.com/auth/gmail.modify"
                })
                .to_string(),
                SecretEntryContext {
                    entry_kind: "provider_credential",
                    account_id,
                    purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                    secret_kind: SecretKind::OauthToken.as_str(),
                    label: "Gmail batch OAuth credential",
                    metadata: &json!({ "provider": "gmail" }),
                },
            )
            .expect("store OAuth token bundle");

        let message_store = MessageProjectionStore::new(pool.clone());
        let command_store = CommunicationProviderCommandStore::new(pool.clone());
        for index in 1..=2 {
            let provider_record_id = format!("gmail-batch-message-{index}");
            let raw = communication_store
                .record_raw_source(&NewRawCommunicationRecord::new(
                    format!("raw-gmail-batch-{index}"),
                    account_id,
                    "email_message",
                    &provider_record_id,
                    format!("sha256:gmail-batch-{index}"),
                    "gmail-batch-import",
                    json!({
                        "subject": format!("Batch message {index}"),
                        "from": "sender@example.test",
                        "to": ["owner@example.test"],
                        "body_text": "Batch mutation fixture",
                        "label_ids": ["INBOX"]
                    }),
                ))
                .await
                .expect("record raw Gmail message");
            let projected = project_raw_email_message(&message_store, &raw)
                .await
                .expect("project Gmail message");
            command_store
                .enqueue(
                    &NewCommunicationProviderCommand::new(
                        format!("gmail-batch-command-{index}"),
                        account_id,
                        "mail",
                        "important",
                        format!("gmail-batch-idempotency-{index}"),
                        "test-actor",
                    )
                    .provider_message_id(&provider_record_id)
                    .target_ref(json!({ "message_id": projected.message_id })),
                )
                .await
                .expect("enqueue Gmail command");
        }

        let worker = MailProviderCommandWorker::new(pool, vault, server.base_url());
        let report = worker
            .execute_due(chrono::Utc::now(), 10)
            .await
            .expect("execute Gmail batch");

        assert_eq!(report.claimed, 2);
        assert_eq!(report.completed, 2);
        assert_eq!(report.retrying, 0);
        assert_eq!(report.dead_lettered, 0);
        assert_eq!(report.provider_batches, 1);
        assert_eq!(report.batched_commands, 2);

        let requests = server.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].request_line,
            "POST /gmail/v1/users/me/messages/batchModify HTTP/1.1"
        );
        assert_eq!(
            requests[0].authorization.as_deref(),
            Some("Bearer gmail-batch-access-token")
        );
        let body: Value = serde_json::from_str(&requests[0].body).expect("batch request JSON");
        assert_eq!(
            body["ids"],
            json!(["gmail-batch-message-1", "gmail-batch-message-2"])
        );
        assert_eq!(body["addLabelIds"], json!(["IMPORTANT"]));
        assert_eq!(body["removeLabelIds"], json!([]));

        let commands = command_store
            .list(account_id, "mail", 10)
            .await
            .expect("list completed commands");
        assert_eq!(commands.len(), 2);
        assert!(commands.iter().all(|command| {
            command.status == "completed"
                && command.result_payload["batch_modify"] == true
                && command.result_payload["batch_size"] == 2
                && command.result_payload["command_kind"] == "important"
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

    #[derive(Clone, Debug)]
    struct RecordedHttpRequest {
        request_line: String,
        authorization: Option<String>,
        body: String,
    }

    struct MockGmailBatchServer {
        addr: SocketAddr,
        requests: Arc<Mutex<Vec<RecordedHttpRequest>>>,
        handle: Option<thread::JoinHandle<()>>,
    }

    impl MockGmailBatchServer {
        fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind Gmail batch server");
            let addr = listener.local_addr().expect("Gmail batch server addr");
            let requests = Arc::new(Mutex::new(Vec::new()));
            let requests_for_thread = Arc::clone(&requests);
            let handle = thread::spawn(move || {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let request = read_http_request(&mut stream);
                if request.request_line.is_empty() {
                    return;
                }
                requests_for_thread
                    .lock()
                    .expect("Gmail batch requests lock")
                    .push(request);
                write_http_response(&mut stream, "{}");
            });
            Self {
                addr,
                requests,
                handle: Some(handle),
            }
        }

        fn base_url(&self) -> String {
            format!("http://{}", self.addr)
        }

        fn requests(&self) -> Vec<RecordedHttpRequest> {
            self.requests
                .lock()
                .expect("Gmail batch requests lock")
                .clone()
        }
    }

    impl Drop for MockGmailBatchServer {
        fn drop(&mut self) {
            let _ = TcpStream::connect(self.addr);
            if let Some(handle) = self.handle.take() {
                handle.join().expect("Gmail batch server join");
            }
        }
    }

    fn read_http_request(stream: &mut TcpStream) -> RecordedHttpRequest {
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("set Gmail batch read timeout");
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .expect("read Gmail batch request line");
        let mut authorization = None;
        let mut content_length = 0usize;
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .expect("read Gmail batch header");
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("authorization") {
                    authorization = Some(value.trim().to_owned());
                } else if name.eq_ignore_ascii_case("content-length") {
                    content_length = value.trim().parse().expect("content length");
                }
            }
        }
        let mut body = vec![0_u8; content_length];
        reader
            .read_exact(&mut body)
            .expect("read Gmail batch request body");
        RecordedHttpRequest {
            request_line: request_line.trim_end().to_owned(),
            authorization,
            body: String::from_utf8(body).expect("Gmail batch request UTF-8"),
        }
    }

    fn write_http_response(stream: &mut TcpStream, body: &str) {
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("write Gmail batch response");
    }
}

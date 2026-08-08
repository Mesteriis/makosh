use makosh_communications_api::accounts::{CommunicationProviderKind, ProviderAccount};
use makosh_communications_api::commands::{
    CommunicationProviderCommand, ProviderCommandQueuePort, ProviderCommandQueuePortError,
};
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::communications::messages::errors::MessageProjectionError;
use crate::domains::communications::messages::store::MessageProjectionStore;
use crate::domains::communications::provider_resources::{
    MailProviderResourceError, MailProviderResourceStore,
};
use crate::integrations::mail::read_state::{
    EmailProviderMessageMutation, EmailReadStateError, LiveEmailReadStateService,
};
use crate::vault::HostVault;
use makosh_communications_api::mail_resources::{
    MailProviderResourceKind, MailProviderSemanticRole,
};
use makosh_communications_postgres::errors::CommunicationIngestionError;
use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
use makosh_communications_postgres::provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};

mod gmail_batches;
mod imap_batches;

const MAIL_COMMAND_EXECUTION_LEASE_SECONDS: i64 = 300;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MailProviderCommandExecutionReport {
    pub claimed: usize,
    pub completed: usize,
    pub retrying: usize,
    pub dead_lettered: usize,
    pub provider_batches: usize,
    pub batched_commands: usize,
}

pub struct MailProviderCommandWorker<Q = CommunicationProviderCommandStore> {
    account_store: CommunicationProviderAccountStore,
    command_queue: Q,
    message_store: MessageProjectionStore,
    pub(super) provider_resource_store: MailProviderResourceStore,
    read_state: LiveEmailReadStateService,
}

impl MailProviderCommandWorker<CommunicationProviderCommandStore> {
    pub fn new(pool: PgPool, vault: HostVault, gmail_api_base_url: impl Into<String>) -> Self {
        Self::with_command_queue(
            pool.clone(),
            vault,
            gmail_api_base_url,
            CommunicationProviderCommandStore::new(pool),
        )
    }
}

impl<Q> MailProviderCommandWorker<Q> {
    pub fn with_command_queue(
        pool: PgPool,
        vault: HostVault,
        gmail_api_base_url: impl Into<String>,
        command_queue: Q,
    ) -> Self {
        Self {
            account_store: CommunicationProviderAccountStore::new(pool.clone()),
            command_queue,
            message_store: MessageProjectionStore::new(pool.clone()),
            provider_resource_store: MailProviderResourceStore::new(pool.clone()),
            read_state: LiveEmailReadStateService::new(
                pool.clone(),
                vault,
                Arc::new(CommunicationProviderSecretBindingStore::new(pool)),
                gmail_api_base_url,
            ),
        }
    }
}

impl<Q> MailProviderCommandWorker<Q>
where
    Q: ProviderCommandQueuePort,
{
    pub async fn execute_due(
        &self,
        now: DateTime<Utc>,
        limit_per_account: i64,
    ) -> Result<MailProviderCommandExecutionReport, MailProviderCommandWorkerError> {
        let mut report = MailProviderCommandExecutionReport::default();
        for account in self
            .account_store
            .list()
            .await?
            .into_iter()
            .filter(|account| {
                matches!(
                    account.provider_kind,
                    CommunicationProviderKind::Gmail
                        | CommunicationProviderKind::Icloud
                        | CommunicationProviderKind::Imap
                )
            })
        {
            for recovered in self
                .command_queue
                .recover_stale_executing(
                    &account.account_id,
                    "mail",
                    now,
                    Duration::seconds(MAIL_COMMAND_EXECUTION_LEASE_SECONDS),
                )
                .await?
            {
                if recovered.status == "dead_letter" {
                    report.dead_lettered += 1;
                }
            }
            let commands = self
                .command_queue
                .claim_due(&account.account_id, "mail", now, limit_per_account)
                .await?;
            report.claimed += commands.len();
            match account.provider_kind {
                CommunicationProviderKind::Gmail => {
                    self.execute_gmail_commands(&account, commands, &mut report)
                        .await?;
                }
                CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap => {
                    self.execute_imap_commands(&account, commands, &mut report)
                        .await?;
                }
                _ => unreachable!("mail command worker filters unsupported providers"),
            }
        }
        Ok(report)
    }

    async fn dead_letter_invalid_command(
        &self,
        command: &CommunicationProviderCommand,
        error: &str,
    ) -> Result<(), ProviderCommandQueuePortError> {
        self.command_queue
            .mark_terminal_failed_at_epoch(
                &command.command_id,
                "mail",
                Utc::now(),
                error,
                json!({
                    "kind": "invalid_mail_provider_command",
                    "command_kind": command.command_kind,
                    "retryable": false,
                }),
                command.lease_epoch,
            )
            .await?;
        Ok(())
    }

    async fn record_mutation_failure(
        &self,
        command: &CommunicationProviderCommand,
        error: &EmailReadStateError,
        report: &mut MailProviderCommandExecutionReport,
    ) -> Result<(), ProviderCommandQueuePortError> {
        let retryable = error.is_retryable();
        let result_payload = json!({
            "kind": "mail_provider_mutation",
            "command_kind": command.command_kind,
            "retryable": retryable,
        });
        let updated = if retryable {
            self.command_queue
                .mark_failed_at_epoch(
                    &command.command_id,
                    "mail",
                    Utc::now(),
                    &error.to_string(),
                    result_payload,
                    command.lease_epoch,
                )
                .await?
        } else {
            self.command_queue
                .mark_terminal_failed_at_epoch(
                    &command.command_id,
                    "mail",
                    Utc::now(),
                    &error.to_string(),
                    result_payload,
                    command.lease_epoch,
                )
                .await?
        };
        if updated
            .as_ref()
            .is_some_and(|item| item.status == "dead_letter")
        {
            report.dead_lettered += 1;
        } else {
            report.retrying += 1;
        }
        Ok(())
    }
}

fn mutation_for_command<'a>(
    command_kind: &str,
    payload: &'a serde_json::Value,
    account: &'a ProviderAccount,
) -> Result<EmailProviderMessageMutation<'a>, EmailReadStateError> {
    let payload_value = |key: &str| {
        payload
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    let account_value = |key: &str| {
        account
            .config
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    let destination = |payload_key: &str, config_key: &str| {
        payload_value(payload_key).or_else(|| account_value(config_key))
    };

    match command_kind {
        "mark_read" => Ok(EmailProviderMessageMutation::SetRead(true)),
        "mark_unread" => Ok(EmailProviderMessageMutation::SetRead(false)),
        "important" => Ok(EmailProviderMessageMutation::SetImportant(true)),
        "not_important" => Ok(EmailProviderMessageMutation::SetImportant(false)),
        "star" => Ok(EmailProviderMessageMutation::SetStarred(true)),
        "unstar" => Ok(EmailProviderMessageMutation::SetStarred(false)),
        "archive" => Ok(EmailProviderMessageMutation::Archive {
            destination_mailbox: destination("provider_mailbox", "archive_mailbox"),
        }),
        "trash" => Ok(EmailProviderMessageMutation::Trash {
            destination_mailbox: destination("provider_mailbox", "trash_mailbox"),
        }),
        "mark_spam" => Ok(EmailProviderMessageMutation::MarkSpam {
            destination_mailbox: destination("provider_mailbox", "spam_mailbox"),
        }),
        "mark_not_spam" => Ok(EmailProviderMessageMutation::UnmarkSpam {
            destination_mailbox: destination("provider_mailbox", "inbox_mailbox"),
        }),
        "add_label" => provider_label(payload, account)
            .map(EmailProviderMessageMutation::AddLabel)
            .ok_or(EmailReadStateError::MissingDestinationMailbox),
        "remove_label" => provider_label(payload, account)
            .map(EmailProviderMessageMutation::RemoveLabel)
            .ok_or(EmailReadStateError::MissingDestinationMailbox),
        "move_folder" => payload_value("provider_mailbox")
            .map(EmailProviderMessageMutation::MoveTo)
            .ok_or(EmailReadStateError::MissingDestinationMailbox),
        "copy_folder" => payload_value("provider_mailbox")
            .map(EmailProviderMessageMutation::CopyTo)
            .ok_or(EmailReadStateError::MissingDestinationMailbox),
        _ => Err(EmailReadStateError::UnsupportedProvider("command_kind")),
    }
}

async fn resolve_provider_command_payload(
    resource_store: &MailProviderResourceStore,
    account: &ProviderAccount,
    command_kind: &str,
    payload: &Value,
) -> Result<Value, MailProviderResourceError> {
    let mut resolved = payload.clone();

    if account.provider_kind == CommunicationProviderKind::Gmail
        && matches!(command_kind, "add_label" | "remove_label")
        && !payload_has_value(&resolved, "provider_label_id")
        && let Some(label) = resolved
            .get("label")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        && let Some(resource) = resource_store
            .resource_for_display_name(&account.account_id, MailProviderResourceKind::Label, label)
            .await?
            .filter(|resource| resource.writable)
    {
        resolved["provider_label_id"] = json!(resource.provider_resource_id);
    }

    if account.provider_kind != CommunicationProviderKind::Gmail
        && !payload_has_value(&resolved, "provider_mailbox")
    {
        let semantic_role = match command_kind {
            "archive" => Some(MailProviderSemanticRole::Archive),
            "trash" => Some(MailProviderSemanticRole::Trash),
            "mark_spam" => Some(MailProviderSemanticRole::Junk),
            "mark_not_spam" => Some(MailProviderSemanticRole::Inbox),
            _ => None,
        };
        if let Some(semantic_role) = semantic_role
            && let Some(resource) = resource_store
                .resource_for_role(
                    &account.account_id,
                    MailProviderResourceKind::Folder,
                    semantic_role,
                )
                .await?
                .filter(|resource| resource.writable)
        {
            resolved["provider_mailbox"] = json!(resource.provider_resource_id);
        }
    }

    if matches!(command_kind, "move_folder" | "copy_folder")
        && !payload_has_value(&resolved, "provider_mailbox")
        && let Some(local_folder_id) = resolved
            .get("folder_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        && let Some(resource) = resource_store
            .resource_for_local_folder(&account.account_id, local_folder_id)
            .await?
            .filter(|resource| resource.writable)
    {
        resolved["provider_mailbox"] = json!(resource.provider_resource_id);
    }

    Ok(resolved)
}

fn payload_has_value(payload: &Value, key: &str) -> bool {
    payload
        .get(key)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn provider_label<'a>(
    payload: &'a serde_json::Value,
    account: &ProviderAccount,
) -> Option<&'a str> {
    let value = |key: &str| {
        payload
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    if account.provider_kind == CommunicationProviderKind::Gmail {
        value("provider_label_id")
    } else {
        value("provider_label_id").or_else(|| value("label"))
    }
}

#[derive(Debug, Error)]
pub enum MailProviderCommandWorkerError {
    #[error(transparent)]
    Account(#[from] CommunicationIngestionError),
    #[error(transparent)]
    Command(#[from] ProviderCommandQueuePortError),
    #[error(transparent)]
    Message(#[from] MessageProjectionError),
    #[error(transparent)]
    ProviderResource(#[from] MailProviderResourceError),
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use makosh_backend_testkit::context::TestContext;
    use serde_json::json;
    use tempfile::tempdir;

    use super::{
        MailProviderCommandWorker, mutation_for_command, resolve_provider_command_payload,
    };
    use crate::domains::communications::provider_resources::{
        MailProviderResourceStore, NewMailProviderResource,
    };
    use crate::integrations::mail::read_state::{
        EmailProviderMessageMutation, EmailReadStateError,
    };
    use makosh_communications_api::accounts::{
        CommunicationProviderKind, NewProviderAccount, ProviderAccount,
    };
    use makosh_communications_api::commands::NewCommunicationProviderCommand;
    use makosh_communications_api::mail_resources::{
        MailProviderResourceKind, MailProviderSemanticRole,
    };
    use makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore;
    use makosh_communications_postgres::provider_store::CommunicationProviderAccountStore;

    use crate::vault::HostVault;
    use crate::vault::models::HostVaultConfig;

    fn provider_account(
        provider_kind: CommunicationProviderKind,
        config: serde_json::Value,
    ) -> ProviderAccount {
        ProviderAccount {
            account_id: "mail-mutation-account".to_owned(),
            provider_kind,
            display_name: "Mail mutation fixture".to_owned(),
            external_account_id: "mutation@example.test".to_owned(),
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn gmail_labels_require_provider_ids_and_custom_moves_require_explicit_mapping() {
        let gmail = provider_account(
            CommunicationProviderKind::Gmail,
            json!({ "archive_mailbox": "ARCHIVE" }),
        );
        assert!(matches!(
            mutation_for_command("add_label", &json!({ "label": "Follow-up" }), &gmail),
            Err(EmailReadStateError::MissingDestinationMailbox)
        ));
        assert_eq!(
            mutation_for_command(
                "add_label",
                &json!({ "label": "Follow-up", "provider_label_id": "Label_42" }),
                &gmail,
            )
            .expect("mapped Gmail label"),
            EmailProviderMessageMutation::AddLabel("Label_42")
        );
        assert!(matches!(
            mutation_for_command("move_folder", &json!({}), &gmail),
            Err(EmailReadStateError::MissingDestinationMailbox)
        ));

        let imap = provider_account(CommunicationProviderKind::Imap, json!({}));
        assert_eq!(
            mutation_for_command("add_label", &json!({ "label": "Follow-up" }), &imap)
                .expect("IMAP keyword"),
            EmailProviderMessageMutation::AddLabel("Follow-up")
        );
        assert_eq!(
            mutation_for_command("mark_not_spam", &json!({}), &imap)
                .expect("IMAP not-spam mutation"),
            EmailProviderMessageMutation::UnmarkSpam {
                destination_mailbox: None,
            }
        );
    }

    #[tokio::test]
    async fn provider_resource_mappings_override_legacy_destinations_and_resolve_gmail_labels() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let imap_account_id = "mail-command-resource-imap";
        let gmail_account_id = "mail-command-resource-gmail";
        let accounts = CommunicationProviderAccountStore::new(pool.clone());
        accounts
            .upsert(
                &NewProviderAccount::new(
                    imap_account_id,
                    CommunicationProviderKind::Imap,
                    "IMAP mapping fixture",
                    "imap-mapping@example.test",
                )
                .config(json!({ "archive_mailbox": "Legacy Archive" })),
            )
            .await
            .expect("IMAP account");
        accounts
            .upsert(&NewProviderAccount::new(
                gmail_account_id,
                CommunicationProviderKind::Gmail,
                "Gmail mapping fixture",
                "gmail-mapping@example.test",
            ))
            .await
            .expect("Gmail account");
        let resources = MailProviderResourceStore::new(pool);
        resources
            .upsert_discovered(
                &NewMailProviderResource::new(
                    imap_account_id,
                    MailProviderResourceKind::Folder,
                    "Archive/2026",
                    "Archive",
                )
                .semantic_role(MailProviderSemanticRole::Archive),
            )
            .await
            .expect("archive mapping");
        resources
            .upsert_discovered(
                &NewMailProviderResource::new(
                    imap_account_id,
                    MailProviderResourceKind::Folder,
                    "INBOX",
                    "Inbox",
                )
                .semantic_role(MailProviderSemanticRole::Inbox),
            )
            .await
            .expect("inbox mapping");
        resources
            .upsert_discovered(
                &NewMailProviderResource::new(
                    gmail_account_id,
                    MailProviderResourceKind::Label,
                    "Label_42",
                    "Follow up",
                )
                .semantic_role(MailProviderSemanticRole::User),
            )
            .await
            .expect("Gmail label mapping");

        let imap_account = accounts
            .get(imap_account_id)
            .await
            .expect("IMAP account query")
            .expect("IMAP account");
        let gmail_account = accounts
            .get(gmail_account_id)
            .await
            .expect("Gmail account query")
            .expect("Gmail account");
        assert_eq!(
            resolve_provider_command_payload(&resources, &imap_account, "archive", &json!({}))
                .await
                .expect("IMAP archive payload")
                .get("provider_mailbox")
                .and_then(serde_json::Value::as_str),
            Some("Archive/2026")
        );
        assert_eq!(
            resolve_provider_command_payload(
                &resources,
                &imap_account,
                "mark_not_spam",
                &json!({}),
            )
            .await
            .expect("IMAP inbox payload")
            .get("provider_mailbox")
            .and_then(serde_json::Value::as_str),
            Some("INBOX")
        );
        assert_eq!(
            resolve_provider_command_payload(
                &resources,
                &gmail_account,
                "add_label",
                &json!({ "label": "Follow up" }),
            )
            .await
            .expect("Gmail label payload")
            .get("provider_label_id")
            .and_then(serde_json::Value::as_str),
            Some("Label_42")
        );
    }

    #[tokio::test]
    async fn invalid_commands_are_dead_lettered_without_stopping_the_batch() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let account_id = "mail-worker-invalid-commands";
        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(&NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Imap,
                "Invalid command fixture",
                "invalid@example.test",
            ))
            .await
            .expect("store provider account");

        let command_store = CommunicationProviderCommandStore::new(pool.clone());
        command_store
            .enqueue(&NewCommunicationProviderCommand::new(
                "mail-invalid-target",
                account_id,
                "mail",
                "mark_read",
                "mail-invalid-target",
                "test-actor",
            ))
            .await
            .expect("enqueue command without target");
        command_store
            .enqueue(
                &NewCommunicationProviderCommand::new(
                    "mail-missing-message",
                    account_id,
                    "mail",
                    "mark_read",
                    "mail-missing-message",
                    "test-actor",
                )
                .target_ref(json!({ "message_id": "missing-message" })),
            )
            .await
            .expect("enqueue command for missing message");

        let vault_home = tempdir().expect("vault home");
        let dev_key_home = tempdir().expect("vault key home");
        let worker = MailProviderCommandWorker::new(
            pool,
            HostVault::new(HostVaultConfig {
                home: vault_home.path().to_path_buf(),
                dev_mode: true,
                dev_key_path: dev_key_home.path().join("vault.key"),
            })
            .expect("host vault"),
            "http://127.0.0.1:1",
        );

        let report = worker
            .execute_due(Utc::now(), 10)
            .await
            .expect("invalid commands must not abort the worker");
        assert_eq!(report.claimed, 2);
        assert_eq!(report.completed, 0);
        assert_eq!(report.retrying, 0);
        assert_eq!(report.dead_lettered, 2);

        let commands = command_store
            .list(account_id, "mail", 10)
            .await
            .expect("list commands");
        assert_eq!(commands.len(), 2);
        assert!(
            commands
                .iter()
                .all(|command| command.status == "dead_letter")
        );
        assert!(commands.iter().all(|command| {
            command.result_payload["retryable"] == false
                && command.result_payload["kind"] == "invalid_mail_provider_command"
        }));
    }

    #[tokio::test]
    async fn stale_execution_leases_are_retried_then_dead_lettered() {
        let context = TestContext::new().await;
        let pool = context.pool().clone();
        let account_id = "mail-worker-stale-command";
        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(&NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Imap,
                "Stale command fixture",
                "stale@example.test",
            ))
            .await
            .expect("store provider account");
        let command_store = CommunicationProviderCommandStore::new(pool);
        command_store
            .enqueue(&NewCommunicationProviderCommand::new(
                "mail-stale-command",
                account_id,
                "mail",
                "mark_read",
                "mail-stale-command",
                "test-actor",
            ))
            .await
            .expect("enqueue stale command fixture");

        let first_attempt_at = Utc::now() - Duration::minutes(30);
        let first_claim = command_store
            .claim_due(account_id, "mail", first_attempt_at, 10)
            .await
            .expect("claim first attempt");
        assert_eq!(first_claim.len(), 1);
        assert_eq!(first_claim[0].retry_count, 1);

        let second_attempt_at = first_attempt_at + Duration::minutes(10);
        let first_recovery = command_store
            .recover_stale_executing(account_id, "mail", second_attempt_at, Duration::minutes(5))
            .await
            .expect("recover first stale lease");
        assert_eq!(first_recovery.len(), 1);
        assert_eq!(first_recovery[0].status, "retrying");
        assert_eq!(first_recovery[0].result_payload["retryable"], true);

        let second_claim = command_store
            .claim_due(account_id, "mail", second_attempt_at, 10)
            .await
            .expect("claim second attempt");
        assert_eq!(second_claim[0].retry_count, 2);
        let third_attempt_at = second_attempt_at + Duration::minutes(10);
        command_store
            .recover_stale_executing(account_id, "mail", third_attempt_at, Duration::minutes(5))
            .await
            .expect("recover second stale lease");
        let third_claim = command_store
            .claim_due(account_id, "mail", third_attempt_at, 10)
            .await
            .expect("claim final attempt");
        assert_eq!(third_claim[0].retry_count, 3);

        let terminal_recovery = command_store
            .recover_stale_executing(
                account_id,
                "mail",
                third_attempt_at + Duration::minutes(10),
                Duration::minutes(5),
            )
            .await
            .expect("dead-letter exhausted stale lease");
        assert_eq!(terminal_recovery.len(), 1);
        assert_eq!(terminal_recovery[0].status, "dead_letter");
        assert_eq!(terminal_recovery[0].result_payload["retryable"], false);
        assert!(terminal_recovery[0].dead_lettered_at.is_some());
    }
}

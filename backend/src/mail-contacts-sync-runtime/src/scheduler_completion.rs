use makosh_mail_contacts_sync_persistence::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    MailContactsSyncScheduledTerminalOutcomeV1, OutboxEnvelopeV1,
    QueueMailContactsSyncScheduledTerminalV1,
};
use makosh_scheduler_protocol::v1::JobRunOutcomeV1;

use crate::{
    MailContactsSyncDueRuntimeContextV1, MailContactsSyncTerminalReceiptBindingV1,
    build_mail_contacts_sync_terminal_receipt_from_binding_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncScheduledCompletionErrorV1 {
    InvalidTime,
    ReceiptBuild,
    Persistence(MailContactsSyncPersistenceErrorV1),
}

pub async fn queue_mail_contacts_sync_terminal_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    logical_owner_id: &str,
    due_context: &MailContactsSyncDueRuntimeContextV1,
    now_unix_millis: i64,
) -> Result<bool, MailContactsSyncScheduledCompletionErrorV1> {
    let now = u64::try_from(now_unix_millis)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(MailContactsSyncScheduledCompletionErrorV1::InvalidTime)?;
    let Some(pending) = persistence
        .pending_scheduled_terminal(logical_owner_id)
        .await
        .map_err(MailContactsSyncScheduledCompletionErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let outcome = match pending.outcome {
        MailContactsSyncScheduledTerminalOutcomeV1::Succeeded => JobRunOutcomeV1::Succeeded,
        MailContactsSyncScheduledTerminalOutcomeV1::Failed => JobRunOutcomeV1::Failed,
    };
    let receipt = build_mail_contacts_sync_terminal_receipt_from_binding_v1(
        MailContactsSyncTerminalReceiptBindingV1 {
            run_id: pending.run_id,
            command_message_id: pending.command_message_id,
            lease_epoch: pending.lease_epoch,
            lease_expires_at_unix_millis: pending.lease_expires_at_unix_millis,
        },
        outcome,
        now,
        due_context,
    )
    .map_err(|_| MailContactsSyncScheduledCompletionErrorV1::ReceiptBuild)?;
    persistence
        .queue_scheduled_terminal(&QueueMailContactsSyncScheduledTerminalV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            run_id: pending.run_id,
            terminal_receipt: OutboxEnvelopeV1 {
                message_id: receipt.message_id,
                envelope_sha256: receipt.envelope_sha256,
                envelope_bytes: receipt.envelope_bytes,
            },
            queued_at_unix_millis: now_unix_millis,
        })
        .await
        .map_err(MailContactsSyncScheduledCompletionErrorV1::Persistence)
}

use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, receive_runtime_pull_delivery,
};
use makosh_mail_contacts_sync_core::{MailContactsSyncDraftV1, MailContactsSyncTriggerV1};
use makosh_mail_contacts_sync_persistence::{
    AcceptScheduledMailContactsSyncDueOutcomeV1, AcceptScheduledMailContactsSyncDueV1,
    MailContactsSyncPersistenceV1, OutboxEnvelopeV1,
};
use makosh_scheduler_protocol::v1::JobRunOutcomeV1;

use crate::{
    MailContactsSyncDueRuntimeContextV1, MailContactsSyncRuntimeSettingsV1,
    build_mail_contacts_sync_terminal_receipt_v1,
    commands::{InitialFetchCommandContextV1, build_initial_fetch_command_v1},
    decode_mail_contacts_sync_due_command_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncScheduledExecutionContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub authoritative_now_unix_millis: i64,
    pub due_context: MailContactsSyncDueRuntimeContextV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncScheduledExecutionOutcomeV1 {
    Launched,
    Duplicate,
    SkippedDisabled,
    SkippedUnknownConfiguration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncScheduledExecutionErrorV1 {
    InvalidDue,
    LeaseExpired,
    CommandBuild,
    Persistence,
    EventUnavailable,
}

pub async fn process_mail_contacts_sync_due_payload_v1(
    persistence: &MailContactsSyncPersistenceV1,
    context: &MailContactsSyncScheduledExecutionContextV1,
    configurations: &[(String, MailContactsSyncRuntimeSettingsV1)],
    exact_bytes: &[u8],
) -> Result<MailContactsSyncScheduledExecutionOutcomeV1, MailContactsSyncScheduledExecutionErrorV1>
{
    let due = decode_mail_contacts_sync_due_command_v1(exact_bytes, &context.due_context)
        .map_err(|_| MailContactsSyncScheduledExecutionErrorV1::InvalidDue)?;
    let now = u64::try_from(context.authoritative_now_unix_millis)
        .ok()
        .filter(|now| *now > 0 && *now < due.lease_expires_at_unix_millis)
        .ok_or(MailContactsSyncScheduledExecutionErrorV1::LeaseExpired)?;
    let selected = configurations
        .iter()
        .find(|(configuration_instance_id, _)| {
            configuration_instance_id == &due.configuration_instance_id
        });
    let launch = selected
        .filter(|(_, settings)| settings.enabled)
        .map(|(_, settings)| MailContactsSyncDraftV1 {
            run_id: due.run_id,
            operation_id: due.run_id,
            account_id: settings.account_id.clone(),
            direction: settings.direction,
            trigger: MailContactsSyncTriggerV1::Scheduled,
        });
    let mut durable_messages = vec![durable_message(&due.acceptance_receipt)];
    if let Some(draft) = launch.as_ref() {
        durable_messages.push(
            build_initial_fetch_command_v1(
                due.run_id,
                &draft.account_id,
                &InitialFetchCommandContextV1 {
                    logical_owner_id: context.logical_owner_id.clone(),
                    runtime_instance_id: context.runtime_instance_id.clone(),
                    runtime_generation: context.due_context.runtime_generation,
                    authoritative_now_unix_millis: context.authoritative_now_unix_millis,
                },
            )
            .map_err(|_| MailContactsSyncScheduledExecutionErrorV1::CommandBuild)?,
        );
    }
    if launch.is_none() {
        durable_messages.push(durable_message(
            &build_mail_contacts_sync_terminal_receipt_v1(
                &due,
                JobRunOutcomeV1::Succeeded,
                now,
                &context.due_context,
            )
            .map_err(|_| MailContactsSyncScheduledExecutionErrorV1::LeaseExpired)?,
        ));
    }
    let result = persistence
        .accept_scheduled_due(AcceptScheduledMailContactsSyncDueV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            command_message_id: due.command_message_id,
            command_envelope_sha256: due.command_envelope_sha256,
            scheduler_run_id: due.run_id,
            lease_epoch: due.lease_epoch,
            lease_expires_at_unix_millis: due.lease_expires_at_unix_millis,
            launch,
            durable_messages,
            occurred_at_unix_millis: context.authoritative_now_unix_millis,
        })
        .await
        .map_err(|_| MailContactsSyncScheduledExecutionErrorV1::Persistence)?;
    Ok(match result {
        AcceptScheduledMailContactsSyncDueOutcomeV1::Launched(_) => {
            MailContactsSyncScheduledExecutionOutcomeV1::Launched
        }
        AcceptScheduledMailContactsSyncDueOutcomeV1::Duplicate(_) => {
            MailContactsSyncScheduledExecutionOutcomeV1::Duplicate
        }
        AcceptScheduledMailContactsSyncDueOutcomeV1::Skipped if selected.is_some() => {
            MailContactsSyncScheduledExecutionOutcomeV1::SkippedDisabled
        }
        AcceptScheduledMailContactsSyncDueOutcomeV1::Skipped => {
            MailContactsSyncScheduledExecutionOutcomeV1::SkippedUnknownConfiguration
        }
    })
}

pub(crate) async fn consume_mail_contacts_sync_due_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    context: &MailContactsSyncScheduledExecutionContextV1,
    configurations: &[(String, MailContactsSyncRuntimeSettingsV1)],
) -> Result<bool, MailContactsSyncScheduledExecutionErrorV1> {
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailContactsSyncScheduledExecutionErrorV1::EventUnavailable)?;
    match process_mail_contacts_sync_due_payload_v1(
        persistence,
        context,
        configurations,
        delivery.exact_bytes(),
    )
    .await
    {
        Ok(_) => {
            delivery
                .acknowledge()
                .await
                .map_err(|_| MailContactsSyncScheduledExecutionErrorV1::EventUnavailable)?;
            Ok(true)
        }
        Err(MailContactsSyncScheduledExecutionErrorV1::InvalidDue) => {
            delivery
                .acknowledge()
                .await
                .map_err(|_| MailContactsSyncScheduledExecutionErrorV1::EventUnavailable)?;
            Ok(true)
        }
        Err(error) => Err(error),
    }
}

fn durable_message(message: &crate::MailContactsSyncDueMessageV1) -> OutboxEnvelopeV1 {
    OutboxEnvelopeV1 {
        message_id: message.message_id,
        envelope_sha256: message.envelope_sha256,
        envelope_bytes: message.envelope_bytes.clone(),
    }
}

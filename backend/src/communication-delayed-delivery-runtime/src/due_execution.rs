use std::os::unix::net::UnixStream;

use makosh_communication_delayed_delivery_event_adapters::{
    DecodedDelayedDeliveryDueCommandV1, DelayedDeliveryDueContractV1, DelayedDeliveryDueMessageV1,
    DelayedDeliveryDueRuntimeContextV1, build_delayed_delivery_terminal_receipt_v1,
    decode_delayed_delivery_due_command_v1,
};
use makosh_communication_delayed_delivery_execution::{
    ClaimDueExecutionV1, DelayedDeliveryDurableMessageV1, DelayedDeliveryExecutionClaimV1,
    DelayedDeliveryExecutionOutcomeV1, ExecutionStoreErrorV1, SchedulerExecutionFenceV1,
    SchedulerReceiptErrorV1, SchedulerReceiptFactoryPortV1, SchedulerTerminalOutcomeV1,
    execute_due_delivery_v1,
};
use makosh_communication_delayed_delivery_persistence::CommunicationDelayedDeliveryPersistenceV1;
use makosh_communication_delayed_delivery_runtime_adapters::ManagedDelayedDeliveryRuntimePortV1;
use makosh_communication_delayed_delivery_store_adapters::DelayedDeliveryExecutionStoreAdapterV1;
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_scheduler_protocol::{SCHEDULER_JOB_DESCRIPTOR_SET_V1, v1::JobRunOutcomeV1};
use sha2::{Digest, Sha256};

use crate::COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1;

pub(crate) struct DelayedDeliveryDueExecutionContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

pub(crate) async fn consume_due_delivery_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    context: &DelayedDeliveryDueExecutionContextV1,
    now_unix_millis: u64,
) -> Result<bool, DelayedDeliveryDueExecutionErrorV1> {
    if context.logical_owner_id.is_empty()
        || context.runtime_instance_id == [0; 16]
        || context.runtime_generation == 0
        || context.grant_epoch == 0
        || now_unix_millis == 0
    {
        return Err(DelayedDeliveryDueExecutionErrorV1::InvalidCommand);
    }
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| DelayedDeliveryDueExecutionErrorV1::EventUnavailable)?;
    let due_context = due_runtime_context(context);
    let due = match decode_delayed_delivery_due_command_v1(delivery.exact_bytes(), &due_context) {
        Ok(due) => due,
        Err(_) => return discard_invalid_due_command(delivery).await,
    };
    let command = execution_command(context, &due, now_unix_millis);
    let mut store = DelayedDeliveryExecutionStoreAdapterV1::new(persistence.clone());
    let mut runtime_port = ManagedDelayedDeliveryRuntimePortV1::new(
        channel,
        dispatcher,
        COMMUNICATION_DELAYED_DELIVERY_BLOB_CAPABILITY_ID_V1,
    )
    .map_err(|_| DelayedDeliveryDueExecutionErrorV1::InvalidCommand)?;
    let mut receipts = DueReceiptFactoryV1 {
        due,
        context: due_context,
    };
    let outcome = execute_due_delivery_v1(
        &mut store,
        &mut runtime_port,
        &mut receipts,
        &command,
        now_unix_millis,
    )
    .await
    .map_err(|error| match error {
        makosh_communication_delayed_delivery_execution::DelayedDeliveryWorkerErrorV1::Store(
            error,
        ) => DelayedDeliveryDueExecutionErrorV1::Store(error),
    })?;
    if matches!(outcome, DelayedDeliveryExecutionOutcomeV1::Retryable) {
        return Ok(false);
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| DelayedDeliveryDueExecutionErrorV1::EventUnavailable)?;
    Ok(true)
}

async fn discard_invalid_due_command(
    delivery: RuntimePullDeliveryV1,
) -> Result<bool, DelayedDeliveryDueExecutionErrorV1> {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_delayed_delivery_due_rejected=invalid_command");
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| DelayedDeliveryDueExecutionErrorV1::EventUnavailable)?;
    Ok(true)
}

fn due_runtime_context(
    context: &DelayedDeliveryDueExecutionContextV1,
) -> DelayedDeliveryDueRuntimeContextV1 {
    let schema_sha256: [u8; 32] = Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).into();
    DelayedDeliveryDueRuntimeContextV1 {
        runtime_instance_id: context.runtime_instance_id,
        runtime_generation: context.runtime_generation,
        grant_epoch: context.grant_epoch,
        contract: DelayedDeliveryDueContractV1 {
            job_revision: 1,
            job_schema_sha256: schema_sha256,
            receipt_revision: 1,
            receipt_schema_sha256: schema_sha256,
        },
    }
}

fn execution_command(
    context: &DelayedDeliveryDueExecutionContextV1,
    due: &DecodedDelayedDeliveryDueCommandV1,
    claimed_at_unix_millis: u64,
) -> ClaimDueExecutionV1 {
    ClaimDueExecutionV1 {
        logical_owner_id: context.logical_owner_id.clone(),
        delayed_operation_id: due.delayed_operation_id,
        command_message_id: due.command_message_id,
        command_envelope_sha256: due.command_envelope_sha256,
        fence: SchedulerExecutionFenceV1 {
            run_id: due.run_id,
            schedule_revision: due.schedule_revision,
            lease_epoch: due.lease_epoch,
            lease_expires_at_unix_millis: due.lease_expires_at_unix_millis,
        },
        acceptance_receipt: durable_message(&due.acceptance_receipt),
        claimed_at_unix_millis,
    }
}

struct DueReceiptFactoryV1 {
    due: DecodedDelayedDeliveryDueCommandV1,
    context: DelayedDeliveryDueRuntimeContextV1,
}

impl SchedulerReceiptFactoryPortV1 for DueReceiptFactoryV1 {
    fn terminal_receipt(
        &mut self,
        claim: &DelayedDeliveryExecutionClaimV1,
        outcome: SchedulerTerminalOutcomeV1,
        observed_at_unix_millis: u64,
    ) -> Result<DelayedDeliveryDurableMessageV1, SchedulerReceiptErrorV1> {
        if claim.delayed_operation_id != self.due.delayed_operation_id
            || claim.fence.run_id != self.due.run_id
            || claim.fence.schedule_revision != self.due.schedule_revision
            || claim.fence.lease_epoch != self.due.lease_epoch
            || claim.fence.lease_expires_at_unix_millis != self.due.lease_expires_at_unix_millis
        {
            return Err(SchedulerReceiptErrorV1::InvalidEnvelope);
        }
        let outcome = match outcome {
            SchedulerTerminalOutcomeV1::Succeeded => JobRunOutcomeV1::Succeeded,
            SchedulerTerminalOutcomeV1::Failed => JobRunOutcomeV1::Failed,
        };
        build_delayed_delivery_terminal_receipt_v1(
            &self.due,
            outcome,
            observed_at_unix_millis,
            &self.context,
        )
        .map(|message| durable_message(&message))
        .map_err(|_| SchedulerReceiptErrorV1::InvalidEnvelope)
    }
}

fn durable_message(message: &DelayedDeliveryDueMessageV1) -> DelayedDeliveryDurableMessageV1 {
    DelayedDeliveryDurableMessageV1 {
        message_id: message.message_id,
        contract_kind: message.contract_kind,
        envelope_sha256: message.envelope_sha256,
        envelope_bytes: message.envelope_bytes.clone(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DelayedDeliveryDueExecutionErrorV1 {
    InvalidCommand,
    Store(ExecutionStoreErrorV1),
    EventUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_context_uses_one_exact_scheduler_descriptor() {
        let context = due_runtime_context(&DelayedDeliveryDueExecutionContextV1 {
            logical_owner_id: "owner".to_owned(),
            runtime_instance_id: [7; 16],
            runtime_generation: 2,
            grant_epoch: 3,
        });
        assert_eq!(
            context.contract.job_schema_sha256,
            context.contract.receipt_schema_sha256
        );
        assert_eq!(context.contract.job_revision, 1);
        assert_eq!(context.contract.receipt_revision, 1);
    }
}

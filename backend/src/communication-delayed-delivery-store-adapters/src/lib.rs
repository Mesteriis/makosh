#![forbid(unsafe_code)]

use makosh_communication_delayed_delivery_execution as execution;
use makosh_communication_delayed_delivery_persistence as persistence;

pub const PACKAGE: &str = "makosh-communication-delayed-delivery-store-adapters";

#[derive(Clone)]
pub struct DelayedDeliveryExecutionStoreAdapterV1 {
    persistence: persistence::CommunicationDelayedDeliveryPersistenceV1,
}

impl DelayedDeliveryExecutionStoreAdapterV1 {
    pub fn new(persistence: persistence::CommunicationDelayedDeliveryPersistenceV1) -> Self {
        Self { persistence }
    }
}

impl execution::ExecutionStorePortV1 for DelayedDeliveryExecutionStoreAdapterV1 {
    async fn claim_due(
        &mut self,
        command: &execution::ClaimDueExecutionV1,
    ) -> Result<execution::ClaimDueExecutionOutcomeV1, execution::ExecutionStoreErrorV1> {
        self.persistence
            .claim_due_execution(&persistence_claim_command(command))
            .await
            .map(execution_claim_outcome)
            .map_err(execution_store_error)
    }

    async fn mark_accepted(
        &mut self,
        command: &execution::MarkDeliveryAcceptedV1,
    ) -> Result<(), execution::ExecutionStoreErrorV1> {
        self.persistence
            .mark_delivery_accepted(&persistence_accepted_command(command))
            .await
            .map_err(execution_store_error)
    }

    async fn mark_failed(
        &mut self,
        command: &execution::MarkDeliveryFailedV1,
    ) -> Result<(), execution::ExecutionStoreErrorV1> {
        self.persistence
            .mark_delivery_failed(&persistence_failed_command(command))
            .await
            .map_err(execution_store_error)
    }
}

impl execution::CleanupStorePortV1 for DelayedDeliveryExecutionStoreAdapterV1 {
    async fn next_pending_cleanup(
        &mut self,
        logical_owner_id: &str,
        now_unix_millis: u64,
    ) -> Result<Option<execution::DelayedDeliveryBodyCleanupJobV1>, execution::ExecutionStoreErrorV1>
    {
        self.persistence
            .next_body_cleanup(logical_owner_id, now_unix_millis)
            .await
            .map(|job| job.map(execution_cleanup_job))
            .map_err(execution_store_error)
    }

    async fn complete_cleanup(
        &mut self,
        job: &execution::DelayedDeliveryBodyCleanupJobV1,
        completed_at_unix_millis: u64,
    ) -> Result<(), execution::ExecutionStoreErrorV1> {
        self.persistence
            .complete_body_cleanup(
                &job.logical_owner_id,
                &job.delayed_operation_id,
                completed_at_unix_millis,
            )
            .await
            .map_err(execution_store_error)
    }

    async fn reschedule_cleanup(
        &mut self,
        job: &execution::DelayedDeliveryBodyCleanupJobV1,
        next_attempt_at_unix_millis: u64,
        rescheduled_at_unix_millis: u64,
    ) -> Result<(), execution::ExecutionStoreErrorV1> {
        self.persistence
            .reschedule_body_cleanup(
                &job.logical_owner_id,
                &job.delayed_operation_id,
                job.attempt_count,
                next_attempt_at_unix_millis,
                rescheduled_at_unix_millis,
            )
            .await
            .map_err(execution_store_error)
    }
}

fn persistence_claim_command(
    command: &execution::ClaimDueExecutionV1,
) -> persistence::ClaimDueExecutionV1 {
    persistence::ClaimDueExecutionV1 {
        logical_owner_id: command.logical_owner_id.clone(),
        delayed_operation_id: command.delayed_operation_id,
        command_message_id: command.command_message_id,
        command_envelope_sha256: command.command_envelope_sha256,
        fence: persistence_fence(&command.fence),
        acceptance_receipt: persistence_message(&command.acceptance_receipt),
        claimed_at_unix_millis: command.claimed_at_unix_millis,
    }
}

fn persistence_claim(
    claim: &execution::DelayedDeliveryExecutionClaimV1,
) -> persistence::DelayedDeliveryExecutionClaimV1 {
    persistence::DelayedDeliveryExecutionClaimV1 {
        logical_owner_id: claim.logical_owner_id.clone(),
        delayed_operation_id: claim.delayed_operation_id,
        delivery_operation_id: claim.delivery_operation_id,
        conversation_id: claim.conversation_id,
        reply_to_message_id: claim.reply_to_message_id,
        body_receipt: persistence::DelayedDeliveryBodyReceiptV1 {
            reference_id: claim.body_receipt.reference_id,
            declared_bytes: claim.body_receipt.declared_bytes,
            sha256: claim.body_receipt.sha256,
            custody_proof: claim.body_receipt.custody_proof.clone(),
        },
        fence: persistence_fence(&claim.fence),
    }
}

fn persistence_accepted_command(
    command: &execution::MarkDeliveryAcceptedV1,
) -> persistence::MarkDeliveryAcceptedV1 {
    persistence::MarkDeliveryAcceptedV1 {
        claim: persistence_claim(&command.claim),
        terminal_receipt: persistence_message(&command.terminal_receipt),
        accepted_at_unix_millis: command.accepted_at_unix_millis,
    }
}

fn persistence_failed_command(
    command: &execution::MarkDeliveryFailedV1,
) -> persistence::MarkDeliveryFailedV1 {
    persistence::MarkDeliveryFailedV1 {
        claim: persistence_claim(&command.claim),
        error_code: command.error_code,
        terminal_receipt: persistence_message(&command.terminal_receipt),
        failed_at_unix_millis: command.failed_at_unix_millis,
    }
}

fn persistence_fence(
    fence: &execution::SchedulerExecutionFenceV1,
) -> persistence::SchedulerExecutionFenceV1 {
    persistence::SchedulerExecutionFenceV1 {
        run_id: fence.run_id,
        schedule_revision: fence.schedule_revision,
        lease_epoch: fence.lease_epoch,
        lease_expires_at_unix_millis: fence.lease_expires_at_unix_millis,
    }
}

fn persistence_message(
    message: &execution::DelayedDeliveryDurableMessageV1,
) -> persistence::DelayedDeliveryDurableMessageV1 {
    persistence::DelayedDeliveryDurableMessageV1 {
        message_id: message.message_id,
        contract_kind: message.contract_kind,
        envelope_sha256: message.envelope_sha256,
        envelope_bytes: message.envelope_bytes.clone(),
    }
}

fn execution_claim_outcome(
    outcome: persistence::ClaimDueExecutionOutcomeV1,
) -> execution::ClaimDueExecutionOutcomeV1 {
    match outcome {
        persistence::ClaimDueExecutionOutcomeV1::Claimed(claim) => {
            execution::ClaimDueExecutionOutcomeV1::Claimed(execution_claim(claim))
        }
        persistence::ClaimDueExecutionOutcomeV1::Duplicate(claim) => {
            execution::ClaimDueExecutionOutcomeV1::Duplicate(execution_claim(claim))
        }
    }
}

fn execution_claim(
    claim: persistence::DelayedDeliveryExecutionClaimV1,
) -> execution::DelayedDeliveryExecutionClaimV1 {
    execution::DelayedDeliveryExecutionClaimV1 {
        logical_owner_id: claim.logical_owner_id,
        delayed_operation_id: claim.delayed_operation_id,
        delivery_operation_id: claim.delivery_operation_id,
        conversation_id: claim.conversation_id,
        reply_to_message_id: claim.reply_to_message_id,
        body_receipt: execution::DelayedDeliveryBodyReceiptV1 {
            reference_id: claim.body_receipt.reference_id,
            declared_bytes: claim.body_receipt.declared_bytes,
            sha256: claim.body_receipt.sha256,
            custody_proof: claim.body_receipt.custody_proof,
        },
        fence: execution::SchedulerExecutionFenceV1 {
            run_id: claim.fence.run_id,
            schedule_revision: claim.fence.schedule_revision,
            lease_epoch: claim.fence.lease_epoch,
            lease_expires_at_unix_millis: claim.fence.lease_expires_at_unix_millis,
        },
    }
}

fn execution_cleanup_job(
    job: persistence::DelayedDeliveryBodyCleanupJobV1,
) -> execution::DelayedDeliveryBodyCleanupJobV1 {
    execution::DelayedDeliveryBodyCleanupJobV1 {
        logical_owner_id: job.logical_owner_id,
        delayed_operation_id: job.delayed_operation_id,
        body_receipt: execution::DelayedDeliveryBodyReceiptV1 {
            reference_id: job.body_receipt.reference_id,
            declared_bytes: job.body_receipt.declared_bytes,
            sha256: job.body_receipt.sha256,
            custody_proof: job.body_receipt.custody_proof,
        },
        reason: match job.reason {
            persistence::DelayedDeliveryBodyCleanupReasonV1::DeliveryAccepted => {
                execution::BodyCleanupReasonV1::DeliveryAccepted
            }
            persistence::DelayedDeliveryBodyCleanupReasonV1::DeliveryRejected => {
                execution::BodyCleanupReasonV1::DeliveryRejected
            }
            persistence::DelayedDeliveryBodyCleanupReasonV1::DeliveryCancelled => {
                execution::BodyCleanupReasonV1::DeliveryCancelled
            }
        },
        attempt_count: job.attempt_count,
    }
}

fn execution_store_error(
    error: persistence::DelayedDeliveryPersistenceErrorV1,
) -> execution::ExecutionStoreErrorV1 {
    match error {
        persistence::DelayedDeliveryPersistenceErrorV1::InvalidInput => {
            execution::ExecutionStoreErrorV1::InvalidInput
        }
        persistence::DelayedDeliveryPersistenceErrorV1::InvalidRow
        | persistence::DelayedDeliveryPersistenceErrorV1::StorageUnavailable => {
            execution::ExecutionStoreErrorV1::Unavailable
        }
        persistence::DelayedDeliveryPersistenceErrorV1::Conflict
        | persistence::DelayedDeliveryPersistenceErrorV1::StaleRevision => {
            execution::ExecutionStoreErrorV1::Conflict
        }
        persistence::DelayedDeliveryPersistenceErrorV1::ClaimLost => {
            execution::ExecutionStoreErrorV1::ClaimLost
        }
        persistence::DelayedDeliveryPersistenceErrorV1::NotFound => {
            execution::ExecutionStoreErrorV1::NotFound
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim() -> persistence::DelayedDeliveryExecutionClaimV1 {
        persistence::DelayedDeliveryExecutionClaimV1 {
            logical_owner_id: "owner-1".to_owned(),
            delayed_operation_id: [1; 16],
            delivery_operation_id: [2; 16],
            conversation_id: [3; 16],
            reply_to_message_id: Some([4; 16]),
            body_receipt: persistence::DelayedDeliveryBodyReceiptV1 {
                reference_id: [5; 16],
                declared_bytes: 6,
                sha256: [7; 32],
                custody_proof: vec![8; 32],
            },
            fence: persistence::SchedulerExecutionFenceV1 {
                run_id: [9; 16],
                schedule_revision: 10,
                lease_epoch: 11,
                lease_expires_at_unix_millis: 12,
            },
        }
    }

    fn execution_claim_fixture() -> execution::DelayedDeliveryExecutionClaimV1 {
        execution_claim(claim())
    }

    fn message() -> execution::DelayedDeliveryDurableMessageV1 {
        execution::DelayedDeliveryDurableMessageV1 {
            message_id: [13; 16],
            contract_kind: "scheduler.job_run.result.v1",
            envelope_sha256: [14; 32],
            envelope_bytes: vec![15; 64],
        }
    }

    #[test]
    fn preserves_claim_identity_and_fence_across_the_store_boundary() {
        let source = claim();
        let mapped = execution_claim(source.clone());
        assert_eq!(mapped.logical_owner_id, source.logical_owner_id);
        assert_eq!(mapped.delayed_operation_id, source.delayed_operation_id);
        assert_eq!(mapped.delivery_operation_id, source.delivery_operation_id);
        assert_eq!(
            mapped.body_receipt.custody_proof,
            source.body_receipt.custody_proof
        );
        assert_eq!(mapped.fence.run_id, source.fence.run_id);
        assert_eq!(mapped.fence.lease_epoch, source.fence.lease_epoch);
    }

    #[test]
    fn preserves_duplicate_outcome_and_fails_closed_on_invalid_rows() {
        assert!(matches!(
            execution_claim_outcome(persistence::ClaimDueExecutionOutcomeV1::Duplicate(claim())),
            execution::ClaimDueExecutionOutcomeV1::Duplicate(_)
        ));
        assert_eq!(
            execution_store_error(persistence::DelayedDeliveryPersistenceErrorV1::InvalidRow),
            execution::ExecutionStoreErrorV1::Unavailable
        );
        assert_eq!(
            execution_store_error(persistence::DelayedDeliveryPersistenceErrorV1::StaleRevision),
            execution::ExecutionStoreErrorV1::Conflict
        );
    }

    #[test]
    fn preserves_due_and_terminal_commands_without_inventing_store_fields() {
        let execution_claim = execution_claim_fixture();
        let due = execution::ClaimDueExecutionV1 {
            logical_owner_id: execution_claim.logical_owner_id.clone(),
            delayed_operation_id: execution_claim.delayed_operation_id,
            command_message_id: [16; 16],
            command_envelope_sha256: [17; 32],
            fence: execution_claim.fence.clone(),
            acceptance_receipt: message(),
            claimed_at_unix_millis: 18,
        };
        let persisted_due = persistence_claim_command(&due);
        assert_eq!(persisted_due.command_message_id, due.command_message_id);
        assert_eq!(
            persisted_due.command_envelope_sha256,
            due.command_envelope_sha256
        );
        assert_eq!(
            persisted_due.acceptance_receipt.envelope_bytes,
            due.acceptance_receipt.envelope_bytes
        );

        let accepted = execution::MarkDeliveryAcceptedV1 {
            claim: execution_claim.clone(),
            terminal_receipt: message(),
            accepted_at_unix_millis: 19,
        };
        let persisted_accepted = persistence_accepted_command(&accepted);
        assert_eq!(
            persisted_accepted.claim.delayed_operation_id,
            execution_claim.delayed_operation_id
        );
        assert_eq!(
            persisted_accepted.terminal_receipt.message_id,
            accepted.terminal_receipt.message_id
        );

        let failed = execution::MarkDeliveryFailedV1 {
            claim: execution_claim,
            error_code: 7,
            terminal_receipt: message(),
            failed_at_unix_millis: 20,
        };
        let persisted_failed = persistence_failed_command(&failed);
        assert_eq!(persisted_failed.error_code, failed.error_code);
        assert_eq!(
            persisted_failed.failed_at_unix_millis,
            failed.failed_at_unix_millis
        );
    }
}

use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::{
    ClaimDueExecutionOutcomeV1, ClaimDueExecutionV1, DelayedDeliveryIntentRequestV1,
    DelayedDeliveryIntentResponseV1, DelayedDeliveryRuntimePortV1, ExecutionStoreErrorV1,
    ExecutionStorePortV1, MarkDeliveryAcceptedV1, MarkDeliveryFailedV1,
    SchedulerReceiptFactoryPortV1, SchedulerTerminalOutcomeV1, decode_delivery_intent_response_v1,
    ports::receipt_matches_body,
};

const ERROR_DELIVERY_INTENT_REJECTED_V1: u16 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryExecutionOutcomeV1 {
    Accepted,
    Rejected,
    Retryable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayedDeliveryWorkerErrorV1 {
    Store(ExecutionStoreErrorV1),
}

pub async fn execute_due_delivery_v1(
    store: &mut impl ExecutionStorePortV1,
    runtime_port: &mut impl DelayedDeliveryRuntimePortV1,
    receipt_factory: &mut impl SchedulerReceiptFactoryPortV1,
    command: &ClaimDueExecutionV1,
    now_unix_millis: u64,
) -> Result<DelayedDeliveryExecutionOutcomeV1, DelayedDeliveryWorkerErrorV1> {
    let claim = match store
        .claim_due(command)
        .await
        .map_err(DelayedDeliveryWorkerErrorV1::Store)?
    {
        ClaimDueExecutionOutcomeV1::Claimed(claim)
        | ClaimDueExecutionOutcomeV1::Duplicate(claim) => claim,
    };
    let body = match runtime_port.read_once(&claim).await {
        Ok(body) => Zeroizing::new(body),
        Err(_) => return Ok(DelayedDeliveryExecutionOutcomeV1::Retryable),
    };
    if !receipt_matches_body(&claim.body_receipt, &body)
        || <[u8; 32]>::from(Sha256::digest(&*body)) != claim.body_receipt.sha256
    {
        return Ok(DelayedDeliveryExecutionOutcomeV1::Retryable);
    }
    let payload = DelayedDeliveryIntentRequestV1 {
        operation_id: claim.delivery_operation_id,
        conversation_id: claim.conversation_id,
        reply_to_message_id: claim.reply_to_message_id,
        body_utf8: body.to_vec(),
    }
    .encode();
    let response = match runtime_port
        .request(claim.delivery_operation_id, payload)
        .await
    {
        Ok(payload) => decode_delivery_intent_response_v1(claim.delivery_operation_id, &payload),
        Err(error) => Err(error),
    };
    match response {
        Ok(DelayedDeliveryIntentResponseV1::Accepted { .. }) => {
            let terminal_receipt = match receipt_factory.terminal_receipt(
                &claim,
                SchedulerTerminalOutcomeV1::Succeeded,
                now_unix_millis,
            ) {
                Ok(receipt) => receipt,
                Err(_) => return Ok(DelayedDeliveryExecutionOutcomeV1::Retryable),
            };
            store
                .mark_accepted(&MarkDeliveryAcceptedV1 {
                    claim: claim.clone(),
                    terminal_receipt,
                    accepted_at_unix_millis: now_unix_millis,
                })
                .await
                .map_err(DelayedDeliveryWorkerErrorV1::Store)?;
            Ok(DelayedDeliveryExecutionOutcomeV1::Accepted)
        }
        Ok(DelayedDeliveryIntentResponseV1::Rejected) => {
            let terminal_receipt = match receipt_factory.terminal_receipt(
                &claim,
                SchedulerTerminalOutcomeV1::Failed,
                now_unix_millis,
            ) {
                Ok(receipt) => receipt,
                Err(_) => return Ok(DelayedDeliveryExecutionOutcomeV1::Retryable),
            };
            store
                .mark_failed(&MarkDeliveryFailedV1 {
                    claim: claim.clone(),
                    error_code: ERROR_DELIVERY_INTENT_REJECTED_V1,
                    terminal_receipt,
                    failed_at_unix_millis: now_unix_millis,
                })
                .await
                .map_err(DelayedDeliveryWorkerErrorV1::Store)?;
            Ok(DelayedDeliveryExecutionOutcomeV1::Rejected)
        }
        Ok(DelayedDeliveryIntentResponseV1::Retryable) | Err(_) => {
            Ok(DelayedDeliveryExecutionOutcomeV1::Retryable)
        }
    }
}

#[cfg(test)]
mod tests {
    use makosh_communication_delivery_intent_api::wire::{
        DeliveryIntentErrorCodeV1, DeliveryIntentStatusV1, SubmitDeliveryIntentResponseV1,
    };
    use prost::Message;

    use super::*;
    use crate::{
        BodyReadErrorV1, BodyReadPortV1, DelayedDeliveryBodyReceiptV1,
        DelayedDeliveryDurableMessageV1, DelayedDeliveryExecutionClaimV1,
        DeliveryIntentRequestErrorV1, DeliveryIntentRequestPortV1, SchedulerExecutionFenceV1,
        SchedulerReceiptErrorV1,
    };

    struct StoreFixture {
        claim: DelayedDeliveryExecutionClaimV1,
        accepted: bool,
        failed: bool,
    }

    impl ExecutionStorePortV1 for StoreFixture {
        async fn claim_due(
            &mut self,
            _: &ClaimDueExecutionV1,
        ) -> Result<ClaimDueExecutionOutcomeV1, ExecutionStoreErrorV1> {
            Ok(ClaimDueExecutionOutcomeV1::Claimed(self.claim.clone()))
        }

        async fn mark_accepted(
            &mut self,
            _: &MarkDeliveryAcceptedV1,
        ) -> Result<(), ExecutionStoreErrorV1> {
            self.accepted = true;
            Ok(())
        }

        async fn mark_failed(
            &mut self,
            _: &MarkDeliveryFailedV1,
        ) -> Result<(), ExecutionStoreErrorV1> {
            self.failed = true;
            Ok(())
        }
    }

    struct RuntimeFixture {
        body: Vec<u8>,
    }

    impl BodyReadPortV1 for RuntimeFixture {
        async fn read_once(
            &mut self,
            _: &DelayedDeliveryExecutionClaimV1,
        ) -> Result<Vec<u8>, BodyReadErrorV1> {
            Ok(self.body.clone())
        }
    }

    impl DeliveryIntentRequestPortV1 for RuntimeFixture {
        async fn request(
            &mut self,
            request_id: [u8; 16],
            _: Vec<u8>,
        ) -> Result<Vec<u8>, DeliveryIntentRequestErrorV1> {
            Ok(SubmitDeliveryIntentResponseV1 {
                intent_id: request_id.to_vec(),
                status: DeliveryIntentStatusV1::DeliveryIntentStatusAccepted as i32,
                error: DeliveryIntentErrorCodeV1::DeliveryIntentErrorCodeUnspecified as i32,
            }
            .encode_to_vec())
        }
    }

    struct ReceiptFixture;

    impl SchedulerReceiptFactoryPortV1 for ReceiptFixture {
        fn terminal_receipt(
            &mut self,
            _: &DelayedDeliveryExecutionClaimV1,
            _: SchedulerTerminalOutcomeV1,
            _: u64,
        ) -> Result<DelayedDeliveryDurableMessageV1, SchedulerReceiptErrorV1> {
            Ok(DelayedDeliveryDurableMessageV1 {
                message_id: [11; 16],
                contract_kind: "scheduler.job_run.result.v1",
                envelope_sha256: [12; 32],
                envelope_bytes: vec![13; 32],
            })
        }
    }

    #[tokio::test]
    async fn accepts_with_stable_request_after_durable_terminal_commit() {
        let body = b"scheduled hello".to_vec();
        let mut store = StoreFixture {
            claim: claim(&body),
            accepted: false,
            failed: false,
        };
        let outcome = execute_due_delivery_v1(
            &mut store,
            &mut RuntimeFixture { body },
            &mut ReceiptFixture,
            &command(),
            20_000,
        )
        .await
        .expect("execution");
        assert_eq!(outcome, DelayedDeliveryExecutionOutcomeV1::Accepted);
        assert!(store.accepted);
        assert!(!store.failed);
    }

    #[tokio::test]
    async fn refuses_body_bytes_that_do_not_match_the_custody_receipt() {
        let expected = b"scheduled hello".to_vec();
        let mut store = StoreFixture {
            claim: claim(&expected),
            accepted: false,
            failed: false,
        };
        let outcome = execute_due_delivery_v1(
            &mut store,
            &mut RuntimeFixture {
                body: b"tampered".to_vec(),
            },
            &mut ReceiptFixture,
            &command(),
            20_000,
        )
        .await
        .expect("execution");
        assert_eq!(outcome, DelayedDeliveryExecutionOutcomeV1::Retryable);
        assert!(!store.accepted);
        assert!(!store.failed);
    }

    fn claim(body: &[u8]) -> DelayedDeliveryExecutionClaimV1 {
        DelayedDeliveryExecutionClaimV1 {
            logical_owner_id: "owner-1".to_owned(),
            delayed_operation_id: [1; 16],
            delivery_operation_id: [2; 16],
            conversation_id: [3; 16],
            reply_to_message_id: None,
            body_receipt: DelayedDeliveryBodyReceiptV1 {
                reference_id: [4; 16],
                declared_bytes: u64::try_from(body.len()).expect("body length"),
                sha256: Sha256::digest(body).into(),
                custody_proof: vec![5; 32],
            },
            fence: SchedulerExecutionFenceV1 {
                run_id: [6; 16],
                schedule_revision: 7,
                lease_epoch: 8,
                lease_expires_at_unix_millis: 30_000,
            },
        }
    }

    fn command() -> ClaimDueExecutionV1 {
        ClaimDueExecutionV1 {
            logical_owner_id: "owner-1".to_owned(),
            delayed_operation_id: [1; 16],
            command_message_id: [9; 16],
            command_envelope_sha256: [10; 32],
            fence: SchedulerExecutionFenceV1 {
                run_id: [6; 16],
                schedule_revision: 7,
                lease_epoch: 8,
                lease_expires_at_unix_millis: 30_000,
            },
            acceptance_receipt: DelayedDeliveryDurableMessageV1 {
                message_id: [14; 16],
                contract_kind: "scheduler.job_run.acceptance.v1",
                envelope_sha256: [15; 32],
                envelope_bytes: vec![16; 32],
            },
            claimed_at_unix_millis: 19_000,
        }
    }
}

//! Durable Scheduler receipt consumption with acknowledgement after commit.

use makosh_scheduler::decode_job_run_receipt_envelope_v1;
use makosh_scheduler_protocol::{
    SchedulerReceiptDeliveryPortV1, SchedulerReceiptDeliveryV1, v1::JobRunOutcomeV1,
};

use super::{
    SchedulerRunAcceptanceOutcomeV1, SchedulerRunAcceptanceV1, SchedulerRunTerminalResultOutcomeV1,
    SchedulerRunTerminalResultV1,
};
use crate::{SchedulerPostgresStoreV1, SchedulerRunClaimErrorV1};

/// One Scheduler-owned durable consumer. Its receipt port is resolved by
/// Kernel/Event-Hub composition and restricts it to an exact catalog consumer.
pub struct SchedulerReceiptConsumerV1<'a, P> {
    port: P,
    store: &'a SchedulerPostgresStoreV1,
}

impl<'a, P> SchedulerReceiptConsumerV1<'a, P>
where
    P: SchedulerReceiptDeliveryPortV1,
{
    #[must_use]
    pub const fn new(port: P, store: &'a SchedulerPostgresStoreV1) -> Self {
        Self { port, store }
    }

    /// Processes one receipt and acknowledges JetStream only after PostgreSQL
    /// has durably applied its exact fenced acceptance or terminal result.
    pub async fn consume_one(
        &mut self,
    ) -> Result<SchedulerReceiptConsumeOutcomeV1, SchedulerReceiptConsumeErrorV1> {
        let delivery = self
            .port
            .receive()
            .await
            .map_err(|_| SchedulerReceiptConsumeErrorV1::ConsumerUnavailable)?;
        let outcome = self.apply(delivery.exact_bytes()).await?;
        delivery
            .acknowledge()
            .await
            .map_err(|_| SchedulerReceiptConsumeErrorV1::AcknowledgementUnavailable)?;
        Ok(outcome)
    }

    async fn apply(
        &self,
        bytes: &[u8],
    ) -> Result<SchedulerReceiptConsumeOutcomeV1, SchedulerReceiptConsumeErrorV1> {
        let receipt = decode_job_run_receipt_envelope_v1(bytes)
            .map_err(|_| SchedulerReceiptConsumeErrorV1::InvalidReceipt)?;
        match JobRunOutcomeV1::try_from(receipt.outcome).ok() {
            Some(JobRunOutcomeV1::Accepted) => self.apply_acceptance(&receipt).await,
            Some(
                JobRunOutcomeV1::Succeeded
                | JobRunOutcomeV1::RetryableFailed
                | JobRunOutcomeV1::Failed
                | JobRunOutcomeV1::Cancelled,
            ) => self.apply_terminal(&receipt).await,
            Some(JobRunOutcomeV1::Unspecified) | None => {
                Err(SchedulerReceiptConsumeErrorV1::InvalidReceipt)
            }
        }
    }

    async fn apply_acceptance(
        &self,
        receipt: &makosh_scheduler_protocol::v1::JobRunReceiptV1,
    ) -> Result<SchedulerReceiptConsumeOutcomeV1, SchedulerReceiptConsumeErrorV1> {
        let acceptance = SchedulerRunAcceptanceV1::try_from(receipt)
            .map_err(|_| SchedulerReceiptConsumeErrorV1::InvalidReceipt)?;
        match self.store.accept_receipt(&acceptance).await {
            Ok(SchedulerRunAcceptanceOutcomeV1::Applied) => {
                Ok(SchedulerReceiptConsumeOutcomeV1::AcceptanceApplied)
            }
            Ok(SchedulerRunAcceptanceOutcomeV1::AlreadyApplied) => {
                Ok(SchedulerReceiptConsumeOutcomeV1::AcceptanceAlreadyApplied)
            }
            Err(error) => Err(map_store_error(error)),
        }
    }

    async fn apply_terminal(
        &self,
        receipt: &makosh_scheduler_protocol::v1::JobRunReceiptV1,
    ) -> Result<SchedulerReceiptConsumeOutcomeV1, SchedulerReceiptConsumeErrorV1> {
        let terminal = SchedulerRunTerminalResultV1::try_from(receipt)
            .map_err(|_| SchedulerReceiptConsumeErrorV1::InvalidReceipt)?;
        match self.store.finish_receipt(&terminal).await {
            Ok(SchedulerRunTerminalResultOutcomeV1::Applied) => {
                Ok(SchedulerReceiptConsumeOutcomeV1::TerminalApplied)
            }
            Ok(SchedulerRunTerminalResultOutcomeV1::AlreadyApplied) => {
                Ok(SchedulerReceiptConsumeOutcomeV1::TerminalAlreadyApplied)
            }
            Err(error) => Err(map_store_error(error)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerReceiptConsumeOutcomeV1 {
    AcceptanceApplied,
    AcceptanceAlreadyApplied,
    TerminalApplied,
    TerminalAlreadyApplied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerReceiptConsumeErrorV1 {
    ConsumerUnavailable,
    InvalidReceipt,
    PredecessorPending,
    PersistenceBusy,
    PersistenceDenied,
    PersistenceUnavailable,
    AcknowledgementUnavailable,
}

fn map_store_error(error: SchedulerRunClaimErrorV1) -> SchedulerReceiptConsumeErrorV1 {
    match error {
        SchedulerRunClaimErrorV1::Unavailable => {
            SchedulerReceiptConsumeErrorV1::PersistenceUnavailable
        }
        SchedulerRunClaimErrorV1::PendingMissing => {
            SchedulerReceiptConsumeErrorV1::PredecessorPending
        }
        SchedulerRunClaimErrorV1::ConcurrencyBusy
        | SchedulerRunClaimErrorV1::ConcurrencyExhausted => {
            SchedulerReceiptConsumeErrorV1::PersistenceBusy
        }
        SchedulerRunClaimErrorV1::Denied | SchedulerRunClaimErrorV1::AlreadyClaimed => {
            SchedulerReceiptConsumeErrorV1::PersistenceDenied
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_stream_ordering_and_capacity_are_retryable_without_hiding_denials() {
        assert_eq!(
            map_store_error(SchedulerRunClaimErrorV1::PendingMissing),
            SchedulerReceiptConsumeErrorV1::PredecessorPending
        );
        for error in [
            SchedulerRunClaimErrorV1::ConcurrencyBusy,
            SchedulerRunClaimErrorV1::ConcurrencyExhausted,
        ] {
            assert_eq!(
                map_store_error(error),
                SchedulerReceiptConsumeErrorV1::PersistenceBusy
            );
        }
        for error in [
            SchedulerRunClaimErrorV1::Denied,
            SchedulerRunClaimErrorV1::AlreadyClaimed,
        ] {
            assert_eq!(
                map_store_error(error),
                SchedulerReceiptConsumeErrorV1::PersistenceDenied
            );
        }
    }
}

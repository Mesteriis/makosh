//! Binding between the Scheduler receipt payload and its durable outer envelope.

use makosh_events_protocol::{
    v1::{
        AckDispositionV1, AckStageV1, DurableEnvelopeV1, ResultOutcomeV1,
        durable_envelope_v1::Semantics,
    },
    validation::envelope::decode_envelope_v1,
};
use makosh_scheduler_protocol::{
    SchedulerReceiptValidationErrorV1,
    v1::{JobRunOutcomeV1, JobRunReceiptV1},
    validate_job_run_receipt_v1,
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerReceiptEnvelopeErrorV1 {
    InvalidEnvelope,
    InvalidReceipt,
    InvalidBinding,
}

/// Decodes one owner-produced receipt only when its durable envelope proves it
/// is the acceptance or terminal result for that exact Scheduler dispatch.
pub fn decode_job_run_receipt_envelope_v1(
    bytes: &[u8],
) -> Result<JobRunReceiptV1, SchedulerReceiptEnvelopeErrorV1> {
    let envelope =
        decode_envelope_v1(bytes).map_err(|_| SchedulerReceiptEnvelopeErrorV1::InvalidEnvelope)?;
    let receipt = JobRunReceiptV1::decode(envelope.payload.as_slice())
        .map_err(|_| SchedulerReceiptEnvelopeErrorV1::InvalidReceipt)?;
    validate_job_run_receipt_v1(&receipt).map_err(map_receipt_validation)?;
    receipt_envelope_binding_is_valid(&envelope, &receipt)
        .then_some(receipt)
        .ok_or(SchedulerReceiptEnvelopeErrorV1::InvalidBinding)
}

fn receipt_envelope_binding_is_valid(
    envelope: &DurableEnvelopeV1,
    receipt: &JobRunReceiptV1,
) -> bool {
    envelope.correlation_id == receipt.job_run_id
        && match JobRunOutcomeV1::try_from(receipt.outcome).ok() {
            Some(JobRunOutcomeV1::Accepted) => acceptance_binding_is_valid(envelope, receipt),
            Some(JobRunOutcomeV1::Succeeded)
            | Some(JobRunOutcomeV1::RetryableFailed)
            | Some(JobRunOutcomeV1::Failed)
            | Some(JobRunOutcomeV1::Cancelled) => terminal_binding_is_valid(envelope, receipt),
            Some(JobRunOutcomeV1::Unspecified) | None => false,
        }
}

fn acceptance_binding_is_valid(envelope: &DurableEnvelopeV1, receipt: &JobRunReceiptV1) -> bool {
    let Some(Semantics::Ack(ack)) = envelope.semantics.as_ref() else {
        return false;
    };
    ack.acknowledged_message_id == receipt.command_message_id
        && ack.stage == AckStageV1::DurableAcceptance as i32
        && matches!(
            AckDispositionV1::try_from(ack.disposition).ok(),
            Some(AckDispositionV1::Applied | AckDispositionV1::Duplicate)
        )
        && timestamp_millis(ack.acknowledged_at.as_ref()) == Some(receipt.observed_at_unix_millis)
}

fn terminal_binding_is_valid(envelope: &DurableEnvelopeV1, receipt: &JobRunReceiptV1) -> bool {
    let Some(Semantics::Result(result)) = envelope.semantics.as_ref() else {
        return false;
    };
    result.command_id == receipt.job_run_id
        && result.command_message_id == receipt.command_message_id
        && result_outcome_matches(receipt.outcome, result.outcome)
        && timestamp_millis(result.completed_at.as_ref()) == Some(receipt.observed_at_unix_millis)
}

fn result_outcome_matches(receipt_outcome: i32, result_outcome: i32) -> bool {
    matches!(
        (
            JobRunOutcomeV1::try_from(receipt_outcome).ok(),
            ResultOutcomeV1::try_from(result_outcome).ok(),
        ),
        (
            Some(JobRunOutcomeV1::Succeeded),
            Some(ResultOutcomeV1::Succeeded)
        ) | (
            Some(JobRunOutcomeV1::RetryableFailed | JobRunOutcomeV1::Failed),
            Some(ResultOutcomeV1::Failed),
        ) | (
            Some(JobRunOutcomeV1::Cancelled),
            Some(ResultOutcomeV1::Cancelled)
        )
    )
}

fn timestamp_millis(timestamp: Option<&prost_types::Timestamp>) -> Option<i64> {
    let timestamp = timestamp?;
    (timestamp.nanos % 1_000_000 == 0)
        .then_some(())
        .and_then(|()| timestamp.seconds.checked_mul(1_000))
        .and_then(|seconds| seconds.checked_add(i64::from(timestamp.nanos / 1_000_000)))
}

fn map_receipt_validation(_: SchedulerReceiptValidationErrorV1) -> SchedulerReceiptEnvelopeErrorV1 {
    SchedulerReceiptEnvelopeErrorV1::InvalidReceipt
}

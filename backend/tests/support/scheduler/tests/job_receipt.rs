//! Scheduler receipt validation without owner implementation code.

use makosh_events_protocol::v1::{
    AckDispositionV1, AckMetadataV1, AckStageV1, ActorKindV1, ActorRefV1, ContractRefV1,
    DurableEnvelopeV1, FenceKindV1, ResultMetadataV1, ResultOutcomeV1, SourceFenceV1, SourceRefV1,
    durable_envelope_v1::Semantics,
};
use makosh_scheduler::{SchedulerReceiptEnvelopeErrorV1, decode_job_run_receipt_envelope_v1};
use makosh_scheduler_protocol::{
    SchedulerReceiptValidationErrorV1, v1, validate_job_run_receipt_v1,
};
use prost::Message;
use prost_types::Timestamp;

#[test]
fn scheduler_receipt_accepts_matching_unexpired_ack() {
    assert_eq!(validate_job_run_receipt_v1(&receipt()), Ok(()));
}

#[test]
fn scheduler_receipt_rejects_a_stale_or_foreign_lease() {
    let mut value = receipt();
    value.lease.as_mut().expect("lease").run_id = vec![9; 16];
    assert_eq!(
        validate_job_run_receipt_v1(&value),
        Err(SchedulerReceiptValidationErrorV1::InvalidLease)
    );
    let mut expired = receipt();
    expired.observed_at_unix_millis = 2_000;
    assert_eq!(
        validate_job_run_receipt_v1(&expired),
        Err(SchedulerReceiptValidationErrorV1::InvalidObservedAt)
    );
}

#[test]
fn scheduler_receipt_envelope_binds_durable_acceptance_to_exact_dispatch() {
    let value = receipt();
    let envelope = acceptance_envelope(&value);
    assert_eq!(
        decode_job_run_receipt_envelope_v1(&envelope.encode_to_vec()),
        Ok(value)
    );
}

#[test]
fn scheduler_receipt_envelope_binds_terminal_result_to_exact_dispatch() {
    let mut value = receipt();
    value.outcome = v1::JobRunOutcomeV1::RetryableFailed as i32;
    let envelope = terminal_envelope(&value);
    assert_eq!(
        decode_job_run_receipt_envelope_v1(&envelope.encode_to_vec()),
        Ok(value)
    );
}

#[test]
fn scheduler_receipt_envelope_rejects_wrong_stage_or_terminal_outcome() {
    let value = receipt();
    let mut acceptance = acceptance_envelope(&value);
    let Some(Semantics::Ack(metadata)) = acceptance.semantics.as_mut() else {
        panic!("ack metadata");
    };
    metadata.stage = AckStageV1::CanonicalPersistence as i32;
    assert_eq!(
        decode_job_run_receipt_envelope_v1(&acceptance.encode_to_vec()),
        Err(SchedulerReceiptEnvelopeErrorV1::InvalidBinding)
    );

    let mut terminal = value.clone();
    terminal.outcome = v1::JobRunOutcomeV1::Succeeded as i32;
    let mut envelope = terminal_envelope(&terminal);
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
        panic!("result metadata");
    };
    metadata.outcome = ResultOutcomeV1::Failed as i32;
    assert_eq!(
        decode_job_run_receipt_envelope_v1(&envelope.encode_to_vec()),
        Err(SchedulerReceiptEnvelopeErrorV1::InvalidBinding)
    );
}

fn receipt() -> v1::JobRunReceiptV1 {
    v1::JobRunReceiptV1 {
        job_run_id: vec![3; 16],
        command_message_id: vec![4; 16],
        lease: Some(v1::JobLeaseV1 {
            run_id: vec![3; 16],
            epoch: 1,
            expires_at_unix_millis: 2_000,
        }),
        outcome: v1::JobRunOutcomeV1::Accepted as i32,
        observed_at_unix_millis: 1_000,
    }
}

fn acceptance_envelope(receipt: &v1::JobRunReceiptV1) -> DurableEnvelopeV1 {
    envelope(
        receipt,
        Semantics::Ack(AckMetadataV1 {
            acknowledged_message_id: receipt.command_message_id.clone(),
            stage: AckStageV1::DurableAcceptance as i32,
            disposition: AckDispositionV1::Applied as i32,
            acknowledged_at: Some(timestamp(receipt.observed_at_unix_millis)),
        }),
    )
}

fn terminal_envelope(receipt: &v1::JobRunReceiptV1) -> DurableEnvelopeV1 {
    let outcome = match v1::JobRunOutcomeV1::try_from(receipt.outcome).expect("outcome") {
        v1::JobRunOutcomeV1::Succeeded => ResultOutcomeV1::Succeeded,
        v1::JobRunOutcomeV1::RetryableFailed | v1::JobRunOutcomeV1::Failed => {
            ResultOutcomeV1::Failed
        }
        v1::JobRunOutcomeV1::Cancelled => ResultOutcomeV1::Cancelled,
        _ => panic!("terminal outcome"),
    };
    envelope(
        receipt,
        Semantics::Result(ResultMetadataV1 {
            command_id: receipt.job_run_id.clone(),
            command_message_id: receipt.command_message_id.clone(),
            outcome: outcome as i32,
            completed_at: Some(timestamp(receipt.observed_at_unix_millis)),
            execution_attempt: 1,
        }),
    )
}

fn envelope(receipt: &v1::JobRunReceiptV1, semantics: Semantics) -> DurableEnvelopeV1 {
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: vec![8; 16],
        contract: Some(ContractRefV1 {
            owner: "mail".to_owned(),
            name: "job_receipt".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![9; 32],
        }),
        source: Some(SourceRefV1 {
            module_id: "mail_runtime".to_owned(),
            runtime_instance_id: vec![10; 16],
            runtime_generation: 2,
        }),
        recorded_at: Some(timestamp(receipt.observed_at_unix_millis)),
        partition_key: b"scheduler".to_vec(),
        causation_message_id: receipt.command_message_id.clone(),
        correlation_id: receipt.job_run_id.clone(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: b"mail_runtime".to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: b"mail_runtime".to_vec(),
            epoch: 2,
        }),
        semantics: Some(semantics),
        payload: receipt.encode_to_vec(),
    }
}

fn timestamp(value: i64) -> Timestamp {
    Timestamp {
        seconds: value.div_euclid(1_000),
        nanos: i32::try_from(value.rem_euclid(1_000) * 1_000_000).expect("millisecond nanos"),
    }
}

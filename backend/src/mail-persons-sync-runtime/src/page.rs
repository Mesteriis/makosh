use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{
        ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1, ResultMetadataV1,
        ResultOutcomeV1, SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
    },
    validation::envelope::{decode_envelope_v1, validate_envelope_v1},
};
use makosh_mail_address_book_contract::{
    MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1, MAIL_RUNTIME_MODULE_ID_V1,
    MailAddressBookEnvelopeContextV1, MailPersonSourceContractV1,
    build_fetch_mail_person_source_page_command_v1, validate_mail_person_source_page_completed_v1,
    validate_mail_person_source_page_rejected_v1,
    wire_person_source::{
        FetchMailPersonSourcePageCommandV1, MailPersonSourcePageCompletedV1,
        MailPersonSourcePageRejectedV1, MailPersonSourceRejectCodeV1,
    },
};
use makosh_mail_persons_sync_api::{
    MailPersonsSyncContractV1, mail_persons_sync_page_receipt_id_v1,
    mail_persons_sync_run_result_id_v1, validate_mail_persons_sync_page_receipt_v1,
    validate_mail_persons_sync_run_result_v1,
    wire::{
        MailPersonsSyncPageIdentityV1, MailPersonsSyncPageReceiptV1, MailPersonsSyncRejectCodeV1,
        MailPersonsSyncRunOutcomeV1, MailPersonsSyncRunResultV1,
    },
};
use makosh_mail_persons_sync_persistence::{
    CompleteMailPersonsSyncPageV1, MailPersonsSyncEnvelopeRecordV1,
    MailPersonsSyncPageContinuationV1, MailPersonsSyncPageFinalizationContextV1,
    MailPersonsSyncPersistenceErrorV1, MailPersonsSyncPersistenceV1, MailPersonsSyncRunContextV1,
    MailPersonsSyncStoredRejectCodeV1,
};
use makosh_scheduler_protocol::v1::JobRunOutcomeV1;
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

use crate::scheduler::{
    MailPersonsSyncSchedulerContextV1, MailPersonsSyncSchedulerErrorV1, build_scheduler_receipt_v1,
};
use crate::{
    MAIL_PERSONS_SYNC_MODULE_ID_V1, MailPersonsSyncEnvelopeContextV1, source_runtime_public_id_v1,
};

const COMMAND_DEADLINE_SECONDS_V1: i64 = 300;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailPersonsSyncPageContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub now_unix_millis: i64,
}

pub(crate) fn build_finished_page_outputs_v1(
    run: &MailPersonsSyncRunContextV1,
    finalization: &MailPersonsSyncPageFinalizationContextV1,
    context: &MailPersonsSyncPageContextV1,
) -> Result<
    (
        MailPersonsSyncEnvelopeRecordV1,
        MailPersonsSyncEnvelopeRecordV1,
    ),
    MailPersonsSyncPageErrorV1,
> {
    let outcome = if finalization.rejected {
        MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeRejected
    } else {
        MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeSucceeded
    };
    let code = if finalization.rejected {
        MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeConflict
    } else {
        MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeUnspecified
    };
    let result_id = mail_persons_sync_run_result_id_v1(
        &context.logical_owner_id,
        &finalization.account_public_id,
        &run.run_id,
    )
    .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
    let page_sources = finalization
        .observed_sources
        .checked_add(finalization.updated_sources)
        .and_then(|count| count.checked_add(finalization.removed_sources))
        .ok_or(MailPersonsSyncPageErrorV1::InvalidPayload)?;
    let payload = MailPersonsSyncRunResultV1 {
        result_id: result_id.to_vec(),
        run_id: run.run_id.to_vec(),
        logical_owner_id: context.logical_owner_id.clone(),
        outcome: outcome as i32,
        processed_pages: run.processed_pages + 1,
        processed_sources: run.processed_sources + u64::from(page_sources),
        code: code as i32,
        completed_at: Some(timestamp(context.now_unix_millis)?),
        account_public_id: finalization.account_public_id.to_vec(),
    };
    validate_mail_persons_sync_run_result_v1(&payload)
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
    let run_result = build_workflow_result_v1(
        MailPersonsSyncContractV1::RunResult,
        result_id,
        run.run_id,
        finalization.completion_message_id,
        run.run_id,
        payload.encode_to_vec(),
        context,
    )?;
    let scheduler_context = MailPersonsSyncSchedulerContextV1 {
        logical_owner_id: context.logical_owner_id.clone(),
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        now_unix_millis: context.now_unix_millis,
    };
    let envelope_context = MailPersonsSyncEnvelopeContextV1 {
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: nanos(context.now_unix_millis)?,
    };
    let terminal = build_scheduler_receipt_v1(
        run.scheduler_message_id,
        run.run_id,
        run.lease_epoch,
        run.lease_expires_at_unix_millis,
        if finalization.rejected {
            JobRunOutcomeV1::RetryableFailed
        } else {
            JobRunOutcomeV1::Succeeded
        },
        &scheduler_context,
        &envelope_context,
    )
    .map_err(scheduler_error)?;
    Ok((
        persistence_record(&run_result),
        persistence_record(&terminal),
    ))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncPageErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(MailPersonsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_mail_person_source_page_once_v1(
    persistence: &MailPersonsSyncPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    contract: MailPersonSourceContractV1,
    context: &MailPersonsSyncPageContextV1,
) -> Result<bool, MailPersonsSyncPageErrorV1> {
    if !matches!(
        contract,
        MailPersonSourceContractV1::PageCompleted | MailPersonSourceContractV1::PageRejected
    ) {
        return Err(MailPersonsSyncPageErrorV1::InvalidPayload);
    }
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| MailPersonsSyncPageErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    if let Some(completion) = prepare_page_v1(persistence, &record, contract, context).await? {
        persistence
            .complete_page_once(&completion)
            .await
            .map_err(MailPersonsSyncPageErrorV1::Persistence)?;
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| MailPersonsSyncPageErrorV1::EventUnavailable)?;
    Ok(true)
}

async fn prepare_page_v1(
    persistence: &MailPersonsSyncPersistenceV1,
    record: &OutboxRecordV1,
    contract: MailPersonSourceContractV1,
    context: &MailPersonsSyncPageContextV1,
) -> Result<Option<CompleteMailPersonsSyncPageV1>, MailPersonsSyncPageErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    let (
        command_id,
        run_id,
        owner,
        account_public_id,
        page_sequence,
        observed,
        updated,
        removed,
        has_more,
        page_digest,
        rejection,
        result_time,
    ) = match contract {
        MailPersonSourceContractV1::PageCompleted => {
            let value = MailPersonSourcePageCompletedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
            validate_mail_person_source_page_completed_v1(&value)
                .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
            (
                id16(&value.command_id)?,
                id16(&value.run_id)?,
                value.logical_owner_id,
                id16(&value.account_public_id)?,
                value.page_sequence,
                value.observed_sources,
                value.updated_sources,
                value.removed_sources,
                value.has_more,
                id32(&value.page_digest)?,
                None,
                value
                    .completed_at
                    .ok_or(MailPersonsSyncPageErrorV1::InvalidPayload)?,
            )
        }
        MailPersonSourceContractV1::PageRejected => {
            let value = MailPersonSourcePageRejectedV1::decode(envelope.payload.as_slice())
                .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
            validate_mail_person_source_page_rejected_v1(&value)
                .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
            let rejection = map_page_rejection_v1(value.code, value.retryable)?;
            let page_digest: [u8; 32] = Sha256::digest(value.encode_to_vec()).into();
            (
                id16(&value.command_id)?,
                id16(&value.run_id)?,
                value.logical_owner_id,
                id16(&value.account_public_id)?,
                value.page_sequence,
                0,
                0,
                0,
                false,
                page_digest,
                Some(rejection),
                value
                    .rejected_at
                    .ok_or(MailPersonsSyncPageErrorV1::InvalidPayload)?,
            )
        }
        _ => return Err(MailPersonsSyncPageErrorV1::InvalidPayload),
    };
    validate_page_result_semantics_v1(
        &envelope,
        record,
        contract,
        run_id,
        page_sequence,
        command_id,
        result_time,
    )?;
    if persistence
        .terminal_page_result_is_known(
            &context.logical_owner_id,
            account_public_id,
            run_id,
            page_sequence,
            command_id,
        )
        .await
        .map_err(MailPersonsSyncPageErrorV1::Persistence)?
    {
        return Ok(None);
    }
    let run = persistence
        .load_run_context(&context.logical_owner_id, run_id)
        .await
        .map_err(MailPersonsSyncPageErrorV1::Persistence)?;
    let result_time_unix_millis =
        timestamp_unix_millis_v1(&result_time).ok_or(MailPersonsSyncPageErrorV1::InvalidPayload)?;
    validate_page_freshness_v1(&run, context, result_time_unix_millis)?;
    if owner != context.logical_owner_id
        || account_public_id != run.account_public_id
        || page_sequence != run.next_page_sequence
        || envelope.partition_key != run_id
        || envelope.correlation_id != run_id
    {
        return Err(MailPersonsSyncPageErrorV1::InvalidPayload);
    }
    let identity = MailPersonsSyncPageIdentityV1 {
        logical_owner_id: owner.clone(),
        account_public_id: account_public_id.to_vec(),
        run_id: run_id.to_vec(),
        page_sequence,
    };
    let receipt_id = mail_persons_sync_page_receipt_id_v1(&identity, page_digest)
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
    let completed_at = timestamp(context.now_unix_millis)?;
    let receipt_payload = MailPersonsSyncPageReceiptV1 {
        receipt_id: receipt_id.to_vec(),
        run_id: run_id.to_vec(),
        logical_owner_id: owner.clone(),
        page_sequence,
        observed_sources: observed,
        updated_sources: updated,
        removed_sources: removed,
        persons_commands: observed + updated + removed,
        page_digest: page_digest.to_vec(),
        completed_at: Some(completed_at),
        account_public_id: account_public_id.to_vec(),
    };
    validate_mail_persons_sync_page_receipt_v1(&receipt_payload)
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
    let receipt = build_workflow_result_v1(
        MailPersonsSyncContractV1::PageReceipt,
        receipt_id,
        command_id,
        *record.message_id(),
        run_id,
        receipt_payload.encode_to_vec(),
        context,
    )?;
    let continuation = if has_more {
        let next_page = page_sequence
            .checked_add(1)
            .ok_or(MailPersonsSyncPageErrorV1::InvalidPayload)?;
        let fetch_id = digest16(
            b"mail-persons-sync.fetch-page.v1",
            &run_id,
            &next_page.to_be_bytes(),
        );
        let fetch = build_fetch_mail_person_source_page_command_v1(
            FetchMailPersonSourcePageCommandV1 {
                command_id: fetch_id.to_vec(),
                run_id: run_id.to_vec(),
                logical_owner_id: owner.clone(),
                account_public_id: account_public_id.to_vec(),
                page_sequence: next_page,
                page_size: MAIL_PERSON_SOURCE_MAX_PAGE_SIZE_V1,
            },
            (context.now_unix_millis / 1_000 + COMMAND_DEADLINE_SECONDS_V1)
                .min(run.lease_expires_at_unix_millis / 1_000),
            &MailAddressBookEnvelopeContextV1 {
                module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
                runtime_instance_id: context.runtime_instance_id.clone(),
                runtime_generation: context.runtime_generation,
                recorded_at_unix_seconds: context.now_unix_millis / 1_000,
                recorded_at_nanos: nanos(context.now_unix_millis)?,
            },
        )
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
        MailPersonsSyncPageContinuationV1::NextPage {
            next_fetch: persistence_record(&fetch),
        }
    } else if rejection.is_none() && observed + updated + removed > 0 {
        MailPersonsSyncPageContinuationV1::AwaitingPersons
    } else {
        let outcome = if rejection.is_some() {
            MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeRejected
        } else {
            MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeSucceeded
        };
        let code = rejection
            .map(|(code, _)| code)
            .unwrap_or(MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeUnspecified);
        let result_id = mail_persons_sync_run_result_id_v1(&owner, &account_public_id, &run_id)
            .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
        let run_payload = MailPersonsSyncRunResultV1 {
            result_id: result_id.to_vec(),
            run_id: run_id.to_vec(),
            logical_owner_id: owner.clone(),
            outcome: outcome as i32,
            processed_pages: run.processed_pages + 1,
            processed_sources: run.processed_sources + u64::from(observed + updated + removed),
            code: code as i32,
            completed_at: Some(timestamp(context.now_unix_millis)?),
            account_public_id: account_public_id.to_vec(),
        };
        validate_mail_persons_sync_run_result_v1(&run_payload)
            .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
        let run_result = build_workflow_result_v1(
            MailPersonsSyncContractV1::RunResult,
            result_id,
            run_id,
            *record.message_id(),
            run_id,
            run_payload.encode_to_vec(),
            context,
        )?;
        let scheduler_context = MailPersonsSyncSchedulerContextV1 {
            logical_owner_id: owner.clone(),
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            now_unix_millis: context.now_unix_millis,
        };
        let envelope_context = MailPersonsSyncEnvelopeContextV1 {
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            recorded_at_unix_seconds: context.now_unix_millis / 1_000,
            recorded_at_nanos: nanos(context.now_unix_millis)?,
        };
        let scheduler_terminal = build_scheduler_receipt_v1(
            run.scheduler_message_id,
            run_id,
            run.lease_epoch,
            run.lease_expires_at_unix_millis,
            rejection
                .map(|(_, outcome)| outcome)
                .unwrap_or(JobRunOutcomeV1::Succeeded),
            &scheduler_context,
            &envelope_context,
        )
        .map_err(scheduler_error)?;
        MailPersonsSyncPageContinuationV1::Finished {
            run_result: persistence_record(&run_result),
            scheduler_terminal: persistence_record(&scheduler_terminal),
        }
    };
    Ok(Some(CompleteMailPersonsSyncPageV1 {
        logical_owner_id: owner,
        account_public_id,
        run_id,
        page_sequence,
        completion: persistence_record(record),
        page_digest,
        observed_sources: observed,
        updated_sources: updated,
        removed_sources: removed,
        has_more,
        page_receipt: persistence_record(&receipt),
        rejection_code: rejection.map(|(code, _)| match code {
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeInvalidRequest => {
                MailPersonsSyncStoredRejectCodeV1::InvalidRequest
            }
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeConflict => {
                MailPersonsSyncStoredRejectCodeV1::Conflict
            }
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeSourceUnavailable => {
                MailPersonsSyncStoredRejectCodeV1::SourceUnavailable
            }
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodePolicy => {
                MailPersonsSyncStoredRejectCodeV1::Policy
            }
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeUnspecified => {
                unreachable!("validated page rejection cannot be unspecified")
            }
        }),
        continuation,
        completed_at_unix_millis: context.now_unix_millis,
    }))
}

fn validate_page_freshness_v1(
    run: &MailPersonsSyncRunContextV1,
    context: &MailPersonsSyncPageContextV1,
    result_time_unix_millis: i64,
) -> Result<(), MailPersonsSyncPageErrorV1> {
    if !matches!(run.state, 1 | 2)
        || context.logical_owner_id.is_empty()
        || context.runtime_instance_id.is_empty()
        || context.runtime_generation == 0
        || context.now_unix_millis <= 0
        || context.now_unix_millis > run.lease_expires_at_unix_millis
        || result_time_unix_millis > context.now_unix_millis
        || result_time_unix_millis > run.lease_expires_at_unix_millis
    {
        Err(MailPersonsSyncPageErrorV1::InvalidPayload)
    } else {
        Ok(())
    }
}

fn map_page_rejection_v1(
    code: i32,
    retryable: bool,
) -> Result<(MailPersonsSyncRejectCodeV1, JobRunOutcomeV1), MailPersonsSyncPageErrorV1> {
    let source = MailPersonSourceRejectCodeV1::try_from(code)
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)?;
    let expected_retryable =
        source == MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeSourceUnavailable;
    if source == MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeUnspecified
        || retryable != expected_retryable
    {
        return Err(MailPersonsSyncPageErrorV1::InvalidPayload);
    }
    let workflow = match source {
        MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeInvalidRequest => {
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeInvalidRequest
        }
        MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeAccountUnavailable
        | MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeSourceUnavailable => {
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeSourceUnavailable
        }
        MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodePolicy => {
            MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodePolicy
        }
        MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeUnspecified => {
            return Err(MailPersonsSyncPageErrorV1::InvalidPayload);
        }
    };
    Ok((
        workflow,
        if retryable {
            JobRunOutcomeV1::RetryableFailed
        } else {
            JobRunOutcomeV1::Failed
        },
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_workflow_result_v1(
    contract: MailPersonsSyncContractV1,
    message_id: [u8; 16],
    command_id: [u8; 16],
    command_message_id: [u8; 16],
    run_id: [u8; 16],
    payload: Vec<u8>,
    context: &MailPersonsSyncPageContextV1,
) -> Result<OutboxRecordV1, MailPersonsSyncPageErrorV1> {
    let reference = contract.reference();
    let envelope_context = MailPersonsSyncEnvelopeContextV1 {
        runtime_instance_id: context.runtime_instance_id.clone(),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: nanos(context.now_unix_millis)?,
    };
    let envelope = DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: message_id.to_vec(),
        contract: Some(contract_ref(reference)),
        source: Some(SourceRefV1 {
            module_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.to_owned(),
            runtime_instance_id: source_runtime_public_id_v1(&envelope_context).to_vec(),
            runtime_generation: context.runtime_generation,
        }),
        recorded_at: Some(timestamp(context.now_unix_millis)?),
        partition_key: run_id.to_vec(),
        causation_message_id: command_message_id.to_vec(),
        correlation_id: run_id.to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::Module as i32,
            actor_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: MAIL_PERSONS_SYNC_MODULE_ID_V1.as_bytes().to_vec(),
            epoch: context.runtime_generation,
        }),
        semantics: Some(Semantics::Result(ResultMetadataV1 {
            command_id: command_id.to_vec(),
            command_message_id: command_message_id.to_vec(),
            outcome: ResultOutcomeV1::Succeeded as i32,
            completed_at: Some(timestamp(context.now_unix_millis)?),
            execution_attempt: 1,
        })),
        payload,
    };
    validate_envelope_v1(&envelope).map_err(|_| MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    OutboxRecordV1::accept(envelope.encode_to_vec())
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidEnvelope)
}

fn validate_mail_result_envelope(
    envelope: &DurableEnvelopeV1,
    record: &OutboxRecordV1,
    contract: MailPersonSourceContractV1,
) -> Result<(), MailPersonsSyncPageErrorV1> {
    let expected = contract.reference();
    crate::inbound::validate_exact_inbound_identity_v1(
        envelope,
        record,
        crate::inbound::ExactInboundIdentityV1 {
            contract: &expected,
            source_module_id: MAIL_RUNTIME_MODULE_ID_V1,
            actor_kind: ActorKindV1::Module,
        },
    )
    .map_err(|()| MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    let actual = envelope
        .contract
        .as_ref()
        .ok_or(MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    let source = envelope
        .source
        .as_ref()
        .ok_or(MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    let actor = envelope
        .actor
        .as_ref()
        .ok_or(MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    let fence = envelope
        .source_fence
        .as_ref()
        .ok_or(MailPersonsSyncPageErrorV1::InvalidEnvelope)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonsSyncPageErrorV1::InvalidEnvelope);
    };
    if actual.owner != expected.owner
        || actual.name != expected.name
        || actual.major != expected.major
        || actual.revision != expected.revision
        || actual.schema_sha256 != expected.schema_sha256
        || envelope.message_id.as_slice() != record.message_id()
        || source.module_id != MAIL_RUNTIME_MODULE_ID_V1
        || source.runtime_generation == 0
        || actor.kind != ActorKindV1::Module as i32
        || actor.actor_id != MAIL_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.kind != FenceKindV1::RuntimeLease as i32
        || fence.scope_id != MAIL_RUNTIME_MODULE_ID_V1.as_bytes()
        || fence.epoch != source.runtime_generation
        || metadata.execution_attempt == 0
    {
        return Err(MailPersonsSyncPageErrorV1::InvalidEnvelope);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_page_result_semantics_v1(
    envelope: &DurableEnvelopeV1,
    record: &OutboxRecordV1,
    contract: MailPersonSourceContractV1,
    run_id: [u8; 16],
    page_sequence: u64,
    command_id: [u8; 16],
    result_time: Timestamp,
) -> Result<(), MailPersonsSyncPageErrorV1> {
    validate_mail_result_envelope(envelope, record, contract)?;
    let Some(Semantics::Result(metadata)) = envelope.semantics.as_ref() else {
        return Err(MailPersonsSyncPageErrorV1::InvalidEnvelope);
    };
    let expected_command_id = digest16(
        b"mail-persons-sync.fetch-page.v1",
        &run_id,
        &page_sequence.to_be_bytes(),
    );
    let expected_message_id = mail_result_id_v1(
        b"mail-person-source-fetch-result-v1",
        &command_id,
        contract.name().as_bytes(),
    );
    let expected_outcome = match contract {
        MailPersonSourceContractV1::PageCompleted => ResultOutcomeV1::Succeeded,
        MailPersonSourceContractV1::PageRejected => ResultOutcomeV1::Rejected,
        _ => return Err(MailPersonsSyncPageErrorV1::InvalidPayload),
    };
    if command_id != expected_command_id
        || envelope.message_id != expected_message_id
        || envelope.partition_key != run_id
        || envelope.correlation_id != run_id
        || envelope.causation_message_id != expected_command_id
        || envelope.recorded_at.as_ref() != Some(&result_time)
        || metadata.command_id != command_id
        || metadata.command_message_id != expected_command_id
        || metadata.outcome != expected_outcome as i32
        || metadata.completed_at.as_ref() != Some(&result_time)
    {
        Err(MailPersonsSyncPageErrorV1::InvalidEnvelope)
    } else {
        Ok(())
    }
}

fn mail_result_id_v1(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update([0]);
    digest.update(first);
    digest.update([0]);
    digest.update(second);
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

fn persistence_record(record: &OutboxRecordV1) -> MailPersonsSyncEnvelopeRecordV1 {
    MailPersonsSyncEnvelopeRecordV1 {
        message_id: *record.message_id(),
        envelope_sha256: *record.envelope_sha256(),
        envelope_bytes: record.exact_bytes().to_vec(),
    }
}

fn contract_ref(value: makosh_runtime_protocol::v1::ContractReferenceV1) -> ContractRefV1 {
    ContractRefV1 {
        owner: value.owner,
        name: value.name,
        major: value.major,
        revision: value.revision,
        schema_sha256: value.schema_sha256,
    }
}

fn scheduler_error(error: MailPersonsSyncSchedulerErrorV1) -> MailPersonsSyncPageErrorV1 {
    match error {
        MailPersonsSyncSchedulerErrorV1::InvalidEnvelope => {
            MailPersonsSyncPageErrorV1::InvalidEnvelope
        }
        MailPersonsSyncSchedulerErrorV1::InvalidPayload => {
            MailPersonsSyncPageErrorV1::InvalidPayload
        }
        MailPersonsSyncSchedulerErrorV1::Persistence(error) => {
            MailPersonsSyncPageErrorV1::Persistence(error)
        }
        MailPersonsSyncSchedulerErrorV1::EventUnavailable => {
            MailPersonsSyncPageErrorV1::EventUnavailable
        }
    }
}

fn timestamp(value: i64) -> Result<Timestamp, MailPersonsSyncPageErrorV1> {
    Ok(Timestamp {
        seconds: value / 1_000,
        nanos: nanos(value)?,
    })
}
fn nanos(value: i64) -> Result<i32, MailPersonsSyncPageErrorV1> {
    i32::try_from((value % 1_000) * 1_000_000)
        .map_err(|_| MailPersonsSyncPageErrorV1::InvalidPayload)
}
fn timestamp_unix_millis_v1(value: &Timestamp) -> Option<i64> {
    if !(0..1_000_000_000).contains(&value.nanos) || value.nanos % 1_000_000 != 0 {
        return None;
    }
    value
        .seconds
        .checked_mul(1_000)?
        .checked_add(i64::from(value.nanos / 1_000_000))
}
fn id16(value: &[u8]) -> Result<[u8; 16], MailPersonsSyncPageErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailPersonsSyncPageErrorV1::InvalidPayload)
}
fn id32(value: &[u8]) -> Result<[u8; 32], MailPersonsSyncPageErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 32]| id.iter().any(|byte| *byte != 0))
        .ok_or(MailPersonsSyncPageErrorV1::InvalidPayload)
}
fn digest16(label: &[u8], first: &[u8], second: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    for part in [label, first, second] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..16].try_into().expect("SHA-256 prefix")
}

#[cfg(test)]
mod tests {
    use makosh_mail_address_book_contract::{
        MailAddressBookResultEnvelopeContextV1, build_mail_person_source_page_completed_v1,
        build_mail_person_source_page_rejected_v1,
        wire_person_source::{
            MailPersonSourcePageCompletedV1, MailPersonSourcePageRejectedV1,
            MailPersonSourceRejectCodeV1,
        },
    };

    use super::*;

    #[test]
    fn next_page_result_is_fresh_while_run_is_in_continuation_state() {
        let run = MailPersonsSyncRunContextV1 {
            account_public_id: [0x31; 16],
            run_id: [0x51; 16],
            state: 2,
            next_page_sequence: 2,
            processed_pages: 1,
            processed_sources: 0,
            rejection_code: None,
            scheduler_message_id: [0x41; 16],
            lease_epoch: 1,
            lease_expires_at_unix_millis: 1_800_000_100_000,
        };
        validate_page_freshness_v1(
            &run,
            &MailPersonsSyncPageContextV1 {
                logical_owner_id: "owner-1".to_owned(),
                runtime_instance_id: "workflow-runtime".to_owned(),
                runtime_generation: 1,
                now_unix_millis: 1_800_000_001_000,
            },
            1_800_000_000_000,
        )
        .expect("continued page result");
    }

    fn completed_record() -> OutboxRecordV1 {
        let run_id = [0x51; 16];
        let command_id = digest16(
            b"mail-persons-sync.fetch-page.v1",
            &run_id,
            &1_u64.to_be_bytes(),
        );
        build_mail_person_source_page_completed_v1(
            command_id,
            MailPersonSourcePageCompletedV1 {
                command_id: command_id.to_vec(),
                run_id: run_id.to_vec(),
                logical_owner_id: "owner-1".to_owned(),
                account_public_id: vec![0x31; 16],
                page_sequence: 1,
                observed_sources: 0,
                updated_sources: 0,
                removed_sources: 0,
                has_more: false,
                page_digest: vec![0x71; 32],
                completed_at: Some(Timestamp {
                    seconds: 1_800_000_000,
                    nanos: 0,
                }),
            },
            &MailAddressBookResultEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime".to_owned(),
                runtime_generation: 1,
                completed_at_unix_seconds: 1_800_000_000,
                completed_at_nanos: 0,
                execution_attempt: 1,
            },
        )
        .expect("completed fixture")
    }

    fn mutate_record(
        record: &OutboxRecordV1,
        mutate: impl FnOnce(&mut DurableEnvelopeV1),
    ) -> OutboxRecordV1 {
        let mut envelope = decode_envelope_v1(record.exact_bytes()).expect("decode fixture");
        mutate(&mut envelope);
        OutboxRecordV1::accept(envelope.encode_to_vec()).expect("accept mutation")
    }

    fn rejected_record() -> OutboxRecordV1 {
        let run_id = [0x51; 16];
        let command_id = digest16(
            b"mail-persons-sync.fetch-page.v1",
            &run_id,
            &1_u64.to_be_bytes(),
        );
        build_mail_person_source_page_rejected_v1(
            command_id,
            MailPersonSourcePageRejectedV1 {
                command_id: command_id.to_vec(),
                run_id: run_id.to_vec(),
                logical_owner_id: "owner-1".to_owned(),
                account_public_id: vec![0x31; 16],
                page_sequence: 1,
                code: MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeSourceUnavailable
                    as i32,
                retryable: true,
                rejected_at: Some(Timestamp {
                    seconds: 1_800_000_000,
                    nanos: 0,
                }),
            },
            &MailAddressBookResultEnvelopeContextV1 {
                runtime_instance_id: "mail-runtime".to_owned(),
                runtime_generation: 1,
                completed_at_unix_seconds: 1_800_000_000,
                completed_at_nanos: 0,
                execution_attempt: 1,
            },
        )
        .expect("rejected fixture")
    }

    #[test]
    fn page_result_identity_outcome_time_and_causation_are_exact() {
        let valid = completed_record();
        let validate = |record: &OutboxRecordV1| {
            let envelope = decode_envelope_v1(record.exact_bytes()).expect("decode");
            validate_page_result_semantics_v1(
                &envelope,
                record,
                MailPersonSourceContractV1::PageCompleted,
                [0x51; 16],
                1,
                digest16(
                    b"mail-persons-sync.fetch-page.v1",
                    &[0x51; 16],
                    &1_u64.to_be_bytes(),
                ),
                Timestamp {
                    seconds: 1_800_000_000,
                    nanos: 0,
                },
            )
        };
        validate(&valid).expect("valid exact result");
        for invalid in [
            mutate_record(&valid, |envelope| {
                envelope.causation_message_id = vec![9; 16]
            }),
            mutate_record(&valid, |envelope| envelope.partition_key = vec![9; 16]),
            mutate_record(&valid, |envelope| {
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result")
                };
                metadata.command_message_id = vec![9; 16];
            }),
            mutate_record(&valid, |envelope| {
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result")
                };
                metadata.outcome = ResultOutcomeV1::Rejected as i32;
            }),
            mutate_record(&valid, |envelope| {
                envelope.recorded_at = Some(Timestamp {
                    seconds: 1_800_000_001,
                    nanos: 0,
                });
            }),
        ] {
            assert_eq!(
                validate(&invalid),
                Err(MailPersonsSyncPageErrorV1::InvalidEnvelope)
            );
        }
    }

    #[test]
    fn rejected_page_result_uses_the_same_exact_identity_matrix() {
        let valid = rejected_record();
        let validate = |record: &OutboxRecordV1| {
            let envelope = decode_envelope_v1(record.exact_bytes()).expect("decode");
            validate_page_result_semantics_v1(
                &envelope,
                record,
                MailPersonSourceContractV1::PageRejected,
                [0x51; 16],
                1,
                digest16(
                    b"mail-persons-sync.fetch-page.v1",
                    &[0x51; 16],
                    &1_u64.to_be_bytes(),
                ),
                Timestamp {
                    seconds: 1_800_000_000,
                    nanos: 0,
                },
            )
        };
        validate(&valid).expect("valid exact rejection");
        for invalid in [
            mutate_record(&valid, |envelope| envelope.correlation_id = vec![9; 16]),
            mutate_record(&valid, |envelope| {
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result")
                };
                metadata.command_id = vec![9; 16];
            }),
            mutate_record(&valid, |envelope| {
                let Some(Semantics::Result(metadata)) = envelope.semantics.as_mut() else {
                    panic!("result")
                };
                metadata.completed_at = Some(Timestamp {
                    seconds: 1_800_000_001,
                    nanos: 0,
                });
            }),
        ] {
            assert_eq!(
                validate(&invalid),
                Err(MailPersonsSyncPageErrorV1::InvalidEnvelope)
            );
        }
    }

    #[test]
    fn page_rejection_code_and_retryability_map_exactly() {
        assert_eq!(
            map_page_rejection_v1(
                MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeSourceUnavailable as i32,
                true,
            ),
            Ok((
                MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeSourceUnavailable,
                JobRunOutcomeV1::RetryableFailed,
            )),
        );
        for (source, expected) in [
            (
                MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeInvalidRequest,
                MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeInvalidRequest,
            ),
            (
                MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeAccountUnavailable,
                MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeSourceUnavailable,
            ),
            (
                MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodePolicy,
                MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodePolicy,
            ),
        ] {
            assert_eq!(
                map_page_rejection_v1(source as i32, false),
                Ok((expected, JobRunOutcomeV1::Failed)),
            );
        }
        assert!(
            map_page_rejection_v1(
                MailPersonSourceRejectCodeV1::MailPersonSourceRejectCodeInvalidRequest as i32,
                true,
            )
            .is_err()
        );
    }

    #[test]
    fn page_transition_and_publication_are_bounded_by_run_lease() {
        let run = MailPersonsSyncRunContextV1 {
            account_public_id: [0x31; 16],
            run_id: [0x51; 16],
            state: 1,
            next_page_sequence: 1,
            processed_pages: 0,
            processed_sources: 0,
            rejection_code: None,
            scheduler_message_id: [0x41; 16],
            lease_epoch: 1,
            lease_expires_at_unix_millis: 20_000,
        };
        let context = MailPersonsSyncPageContextV1 {
            logical_owner_id: "owner-1".to_owned(),
            runtime_instance_id: "runtime-1".to_owned(),
            runtime_generation: 1,
            now_unix_millis: 20_001,
        };
        assert_eq!(
            validate_page_freshness_v1(&run, &context, 19_999),
            Err(MailPersonsSyncPageErrorV1::InvalidPayload),
        );
        let mut before_expiry = context;
        before_expiry.now_unix_millis = 19_999;
        assert_eq!(
            validate_page_freshness_v1(&run, &before_expiry, 20_001),
            Err(MailPersonsSyncPageErrorV1::InvalidPayload)
        );
        assert_eq!(
            validate_page_freshness_v1(&run, &before_expiry, 19_999),
            Ok(())
        );
    }
}

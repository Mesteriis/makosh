use makosh_communication_delayed_delivery_api::wire::{
    CancelDelayedDeliveryRequestV1, CancelDelayedDeliveryResponseV1,
    DelayedDeliveryErrorCodeV1 as WireError, DelayedDeliveryReceiptKindV1 as WireReceipt,
    DelayedDeliveryStateV1 as WireState, GetDelayedDeliveryStatusRequestV1,
    GetDelayedDeliveryStatusResponseV1, ScheduleDelayedDeliveryRequestV1,
    ScheduleDelayedDeliveryResponseV1,
};
use makosh_communication_delayed_delivery_core::{
    DelayedDeliveryDraftV1, DelayedDeliveryLifecycleV1, DelayedDeliveryStateV1,
    prepare_delayed_delivery_v1, request_cancellation_v1,
};
use makosh_communication_delayed_delivery_event_adapters::{
    DelayedDeliverySchedulerCommandContextV1, DelayedDeliverySchedulerMessageV1,
    build_scheduler_command_v1,
};
use makosh_communication_delayed_delivery_persistence::{
    CommunicationDelayedDeliveryPersistenceV1, CreateDelayedDeliveryOperationOutcomeV1,
    CreateDelayedDeliveryOperationV1, DelayedDeliveryBodyReceiptV1,
    DelayedDeliveryDurableMessageV1, DelayedDeliveryOperationStatusV1,
    DelayedDeliveryPersistenceErrorV1, RequestDelayedDeliveryCancellationV1,
};
use makosh_communication_delayed_delivery_runtime_adapters::{
    DelayedDeliveryBodyCustodyReceiptV1, ManagedDelayedDeliveryRuntimePortV1,
};
use makosh_scheduler_protocol::{
    SCHEDULER_JOB_DESCRIPTOR_SET_V1,
    v1::{
        CancelOneShotScheduleV1, EnsureOneShotScheduleV1, JobKindV1,
        SchedulerScheduleControlCommandV1,
        scheduler_schedule_control_command_v1::Operation as SchedulerOperation,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const JOB_OWNER_V1: &str = "communication_delayed_delivery";
const JOB_NAME_V1: &str = "execute";
const SCHEDULE_REVISION_V1: u64 = 1;
const JOB_DEADLINE_MILLIS_V1: u64 = 300_000;
const JOB_MAX_ATTEMPTS_V1: u32 = 8;
const JOB_RETRY_BACKOFF_MILLIS_V1: u64 = 5_000;
const COMMAND_DEADLINE_SECONDS_V1: i64 = 30;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelayedDeliveryClientContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub authoritative_now_unix_millis: u64,
}

pub async fn schedule_delayed_delivery_payload_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    custody: &mut ManagedDelayedDeliveryRuntimePortV1<'_>,
    context: &DelayedDeliveryClientContextV1,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = ScheduleDelayedDeliveryRequestV1::decode(payload) else {
        return schedule_error(
            Vec::new(),
            WireError::DelayedDeliveryErrorCodeInvalidRequest,
        );
    };
    let response_id = request.delayed_operation_id.clone();
    let body_utf8 = Zeroizing::new(request.body_utf8);
    let Some(delayed_operation_id) = id16(&request.delayed_operation_id) else {
        return schedule_error(
            response_id,
            WireError::DelayedDeliveryErrorCodeInvalidRequest,
        );
    };
    let operation = match prepare_delayed_delivery_v1(
        DelayedDeliveryDraftV1 {
            delayed_operation_id,
            delivery_operation_id: match id16(&request.delivery_operation_id) {
                Some(value) => value,
                None => {
                    return schedule_error(
                        response_id,
                        WireError::DelayedDeliveryErrorCodeInvalidRequest,
                    );
                }
            },
            conversation_id: match id16(&request.conversation_id) {
                Some(value) => value,
                None => {
                    return schedule_error(
                        response_id,
                        WireError::DelayedDeliveryErrorCodeInvalidRequest,
                    );
                }
            },
            reply_to_message_id: match optional_id16(request.reply_to_message_id.as_deref()) {
                Ok(value) => value,
                Err(()) => {
                    return schedule_error(
                        response_id,
                        WireError::DelayedDeliveryErrorCodeInvalidRequest,
                    );
                }
            },
            body_utf8: body_utf8.to_vec(),
            deliver_at_unix_millis: request.deliver_at_unix_millis,
        },
        context.authoritative_now_unix_millis,
    ) {
        Ok(operation) => operation,
        Err(_) => {
            return schedule_error(
                response_id,
                WireError::DelayedDeliveryErrorCodeInvalidRequest,
            );
        }
    };
    let custody_receipt =
        match custody.materialize_body(&context.logical_owner_id, delayed_operation_id, &body_utf8)
        {
            Ok(receipt) => persistence_receipt(receipt),
            Err(_) => {
                return schedule_error(response_id, WireError::DelayedDeliveryErrorCodeUnavailable);
            }
        };
    let scheduler_command = match ensure_scheduler_command(&operation)
        .and_then(|command| scheduler_message(command, context))
    {
        Some(message) => message,
        None => {
            return schedule_error(response_id, WireError::DelayedDeliveryErrorCodeUnavailable);
        }
    };
    match persistence
        .create_operation(&CreateDelayedDeliveryOperationV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            operation,
            body_receipt: custody_receipt,
            scheduler_command,
            created_at_unix_millis: context.authoritative_now_unix_millis,
        })
        .await
    {
        Ok(CreateDelayedDeliveryOperationOutcomeV1::Created { state_revision }) => {
            schedule_response(
                delayed_operation_id,
                state_revision,
                WireReceipt::DelayedDeliveryReceiptKindAccepted,
            )
        }
        Ok(CreateDelayedDeliveryOperationOutcomeV1::Existing { state_revision }) => {
            schedule_response(
                delayed_operation_id,
                state_revision,
                WireReceipt::DelayedDeliveryReceiptKindExisting,
            )
        }
        Err(DelayedDeliveryPersistenceErrorV1::Conflict) => schedule_error(
            response_id,
            WireError::DelayedDeliveryErrorCodeInvalidRequest,
        ),
        Err(_) => schedule_error(response_id, WireError::DelayedDeliveryErrorCodeUnavailable),
    }
}

pub async fn cancel_delayed_delivery_payload_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    context: &DelayedDeliveryClientContextV1,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = CancelDelayedDeliveryRequestV1::decode(payload) else {
        return cancel_error(
            Vec::new(),
            WireError::DelayedDeliveryErrorCodeInvalidRequest,
        );
    };
    let response_id = request.delayed_operation_id.clone();
    let Some(delayed_operation_id) = id16(&request.delayed_operation_id) else {
        return cancel_error(
            response_id,
            WireError::DelayedDeliveryErrorCodeInvalidRequest,
        );
    };
    let status = match persistence
        .status(&context.logical_owner_id, &delayed_operation_id)
        .await
    {
        Ok(status) => status,
        Err(DelayedDeliveryPersistenceErrorV1::NotFound) => {
            return cancel_error(response_id, WireError::DelayedDeliveryErrorCodeNotFound);
        }
        Err(_) => {
            return cancel_error(response_id, WireError::DelayedDeliveryErrorCodeUnavailable);
        }
    };
    let repeated = status.state == DelayedDeliveryStateV1::CancelRequested
        && status.state_revision == request.expected_revision.saturating_add(1);
    if repeated {
        return cancel_response(
            delayed_operation_id,
            status.state,
            status.state_revision,
            WireReceipt::DelayedDeliveryReceiptKindExisting,
        );
    }
    if request_cancellation_v1(
        DelayedDeliveryLifecycleV1 {
            state: status.state,
            revision: status.state_revision,
        },
        request.expected_revision,
    )
    .is_err()
    {
        return cancel_error(
            response_id,
            WireError::DelayedDeliveryErrorCodeStaleRevision,
        );
    }
    let scheduler_command = match scheduler_message(
        cancel_scheduler_command(
            delayed_operation_id,
            status
                .scheduler_schedule_revision
                .unwrap_or(SCHEDULE_REVISION_V1),
            request.expected_revision,
        ),
        context,
    ) {
        Some(message) => message,
        None => {
            return cancel_error(response_id, WireError::DelayedDeliveryErrorCodeUnavailable);
        }
    };
    match persistence
        .request_cancellation(&RequestDelayedDeliveryCancellationV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            delayed_operation_id,
            expected_revision: request.expected_revision,
            scheduler_command,
            requested_at_unix_millis: context.authoritative_now_unix_millis,
        })
        .await
    {
        Ok(status) => cancel_response(
            delayed_operation_id,
            status.state,
            status.state_revision,
            WireReceipt::DelayedDeliveryReceiptKindAccepted,
        ),
        Err(DelayedDeliveryPersistenceErrorV1::StaleRevision) => cancel_error(
            response_id,
            WireError::DelayedDeliveryErrorCodeStaleRevision,
        ),
        Err(_) => cancel_error(response_id, WireError::DelayedDeliveryErrorCodeUnavailable),
    }
}

pub async fn get_delayed_delivery_status_payload_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetDelayedDeliveryStatusRequestV1::decode(payload) else {
        return status_error(
            Vec::new(),
            WireError::DelayedDeliveryErrorCodeInvalidRequest,
        );
    };
    let response_id = request.delayed_operation_id.clone();
    let Some(delayed_operation_id) = id16(&request.delayed_operation_id) else {
        return status_error(
            response_id,
            WireError::DelayedDeliveryErrorCodeInvalidRequest,
        );
    };
    match persistence
        .status(logical_owner_id, &delayed_operation_id)
        .await
    {
        Ok(status) => status_response(status),
        Err(DelayedDeliveryPersistenceErrorV1::NotFound) => {
            status_error(response_id, WireError::DelayedDeliveryErrorCodeNotFound)
        }
        Err(_) => status_error(response_id, WireError::DelayedDeliveryErrorCodeUnavailable),
    }
}

fn ensure_scheduler_command(
    operation: &makosh_communication_delayed_delivery_core::DelayedDeliveryOperationV1,
) -> Option<SchedulerScheduleControlCommandV1> {
    let scope = scope_id(operation.delayed_operation_id());
    Some(SchedulerScheduleControlCommandV1 {
        operation_id: operation.delayed_operation_id().to_vec(),
        operation: Some(SchedulerOperation::EnsureOneShot(EnsureOneShotScheduleV1 {
            schedule_id: operation.delayed_operation_id().to_vec(),
            schedule_revision: SCHEDULE_REVISION_V1,
            job_kind: Some(job_kind()),
            job_contract_revision: 1,
            job_schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
            scope_id: scope.clone(),
            concurrency_key: scope,
            due_at_unix_millis: i64::try_from(operation.deliver_at_unix_millis()).ok()?,
            deadline_millis: JOB_DEADLINE_MILLIS_V1,
            max_attempts: JOB_MAX_ATTEMPTS_V1,
            retry_base_backoff_millis: JOB_RETRY_BACKOFF_MILLIS_V1,
        })),
    })
}

fn cancel_scheduler_command(
    delayed_operation_id: [u8; 16],
    schedule_revision: u64,
    expected_workflow_revision: u64,
) -> SchedulerScheduleControlCommandV1 {
    SchedulerScheduleControlCommandV1 {
        operation_id: cancel_operation_id(delayed_operation_id, expected_workflow_revision)
            .to_vec(),
        operation: Some(SchedulerOperation::CancelOneShot(CancelOneShotScheduleV1 {
            schedule_id: delayed_operation_id.to_vec(),
            expected_schedule_revision: schedule_revision,
            job_kind: Some(job_kind()),
        })),
    }
}

fn scheduler_message(
    command: SchedulerScheduleControlCommandV1,
    context: &DelayedDeliveryClientContextV1,
) -> Option<DelayedDeliveryDurableMessageV1> {
    let seconds = i64::try_from(context.authoritative_now_unix_millis / 1_000).ok()?;
    let nanos = i32::try_from((context.authoritative_now_unix_millis % 1_000) * 1_000_000).ok()?;
    let message = build_scheduler_command_v1(
        command,
        &DelayedDeliverySchedulerCommandContextV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            runtime_instance_id: context.runtime_instance_id,
            runtime_generation: context.runtime_generation,
            grant_epoch: context.grant_epoch,
            contract_revision: 1,
            contract_schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).into(),
            recorded_at_unix_seconds: seconds,
            recorded_at_nanos: nanos,
            deadline_unix_seconds: seconds.checked_add(COMMAND_DEADLINE_SECONDS_V1)?,
        },
    )
    .ok()?;
    Some(persistence_message(message))
}

fn persistence_message(
    message: DelayedDeliverySchedulerMessageV1,
) -> DelayedDeliveryDurableMessageV1 {
    DelayedDeliveryDurableMessageV1 {
        message_id: message.message_id,
        contract_kind: message.contract_kind,
        envelope_sha256: message.envelope_sha256,
        envelope_bytes: message.envelope_bytes,
    }
}

fn persistence_receipt(
    receipt: DelayedDeliveryBodyCustodyReceiptV1,
) -> DelayedDeliveryBodyReceiptV1 {
    DelayedDeliveryBodyReceiptV1 {
        reference_id: receipt.reference_id,
        declared_bytes: receipt.declared_bytes,
        sha256: receipt.sha256,
        custody_proof: receipt.custody_proof,
    }
}

fn job_kind() -> JobKindV1 {
    JobKindV1 {
        owner: JOB_OWNER_V1.to_owned(),
        name: JOB_NAME_V1.to_owned(),
        major: 1,
    }
}

fn cancel_operation_id(
    delayed_operation_id: [u8; 16],
    expected_workflow_revision: u64,
) -> [u8; 16] {
    let digest = Sha256::new()
        .chain_update(b"makosh.communication-delayed-delivery.cancel.v1\0")
        .chain_update(delayed_operation_id)
        .chain_update(expected_workflow_revision.to_be_bytes())
        .finalize();
    digest[..16].try_into().expect("SHA-256 prefix is exact")
}

fn scope_id(value: &[u8; 16]) -> String {
    use std::fmt::Write;
    value
        .iter()
        .fold(String::with_capacity(32), |mut out, byte| {
            write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
            out
        })
}

fn id16(value: &[u8]) -> Option<[u8; 16]> {
    let value: [u8; 16] = value.try_into().ok()?;
    value.iter().any(|byte| *byte != 0).then_some(value)
}

fn optional_id16(value: Option<&[u8]>) -> Result<Option<[u8; 16]>, ()> {
    value.map(|value| id16(value).ok_or(())).transpose()
}

const fn wire_state(state: DelayedDeliveryStateV1) -> WireState {
    match state {
        DelayedDeliveryStateV1::Accepted => WireState::DelayedDeliveryStateAccepted,
        DelayedDeliveryStateV1::SchedulePending => WireState::DelayedDeliveryStateSchedulePending,
        DelayedDeliveryStateV1::Scheduled => WireState::DelayedDeliveryStateScheduled,
        DelayedDeliveryStateV1::Due => WireState::DelayedDeliveryStateDue,
        DelayedDeliveryStateV1::Dispatching => WireState::DelayedDeliveryStateDispatching,
        DelayedDeliveryStateV1::DeliveryAccepted => WireState::DelayedDeliveryStateDeliveryAccepted,
        DelayedDeliveryStateV1::CancelRequested => WireState::DelayedDeliveryStateCancelRequested,
        DelayedDeliveryStateV1::Cancelled => WireState::DelayedDeliveryStateCancelled,
        DelayedDeliveryStateV1::Failed => WireState::DelayedDeliveryStateFailed,
    }
}

fn schedule_response(
    delayed_operation_id: [u8; 16],
    state_revision: u64,
    receipt: WireReceipt,
) -> Vec<u8> {
    ScheduleDelayedDeliveryResponseV1 {
        delayed_operation_id: delayed_operation_id.to_vec(),
        state: WireState::DelayedDeliveryStateSchedulePending as i32,
        state_revision,
        receipt: receipt as i32,
        error: WireError::DelayedDeliveryErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn schedule_error(delayed_operation_id: Vec<u8>, error: WireError) -> Vec<u8> {
    ScheduleDelayedDeliveryResponseV1 {
        delayed_operation_id,
        state: WireState::DelayedDeliveryStateUnspecified as i32,
        state_revision: 0,
        receipt: WireReceipt::DelayedDeliveryReceiptKindUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn cancel_error(delayed_operation_id: Vec<u8>, error: WireError) -> Vec<u8> {
    CancelDelayedDeliveryResponseV1 {
        delayed_operation_id,
        state: WireState::DelayedDeliveryStateUnspecified as i32,
        state_revision: 0,
        receipt: WireReceipt::DelayedDeliveryReceiptKindUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn cancel_response(
    delayed_operation_id: [u8; 16],
    state: DelayedDeliveryStateV1,
    state_revision: u64,
    receipt: WireReceipt,
) -> Vec<u8> {
    CancelDelayedDeliveryResponseV1 {
        delayed_operation_id: delayed_operation_id.to_vec(),
        state: wire_state(state) as i32,
        state_revision,
        receipt: receipt as i32,
        error: WireError::DelayedDeliveryErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn status_response(status: DelayedDeliveryOperationStatusV1) -> Vec<u8> {
    GetDelayedDeliveryStatusResponseV1 {
        delayed_operation_id: status.delayed_operation_id.to_vec(),
        state: wire_state(status.state) as i32,
        state_revision: status.state_revision,
        requested_due_at_unix_millis: status.deliver_at_unix_millis,
        delivery_operation_id: Some(status.delivery_operation_id.to_vec()),
        created_at_unix_millis: status.created_at_unix_millis,
        updated_at_unix_millis: status.updated_at_unix_millis,
        error: WireError::DelayedDeliveryErrorCodeUnspecified as i32,
    }
    .encode_to_vec()
}

fn status_error(delayed_operation_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetDelayedDeliveryStatusResponseV1 {
        delayed_operation_id,
        state: WireState::DelayedDeliveryStateUnspecified as i32,
        state_revision: 0,
        requested_due_at_unix_millis: 0,
        delivery_operation_id: None,
        created_at_unix_millis: 0,
        updated_at_unix_millis: 0,
        error: error as i32,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_policy_and_cancel_identity_are_stable_and_distinct() {
        let cancel = cancel_operation_id([1; 16], 2);
        assert_eq!(cancel, cancel_operation_id([1; 16], 2));
        assert_ne!(cancel, [1; 16]);
        assert_ne!(cancel, cancel_operation_id([1; 16], 3));
        assert_eq!(JOB_DEADLINE_MILLIS_V1, 300_000);
        assert_eq!(JOB_MAX_ATTEMPTS_V1, 8);
    }

    #[test]
    fn wire_state_mapping_covers_every_domain_state() {
        assert_eq!(
            wire_state(DelayedDeliveryStateV1::DeliveryAccepted),
            WireState::DelayedDeliveryStateDeliveryAccepted
        );
        assert_eq!(
            wire_state(DelayedDeliveryStateV1::CancelRequested),
            WireState::DelayedDeliveryStateCancelRequested
        );
        assert_eq!(
            wire_state(DelayedDeliveryStateV1::Failed),
            WireState::DelayedDeliveryStateFailed
        );
    }
}

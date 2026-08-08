use makosh_ai_contracts::{
    validate_attachment_translation_inference_request_v1,
    wire::{
        AiInferenceTerminalStatusV1, AiProviderTranslationRequestV1,
        AttachmentTranslationInferenceRequestV1,
    },
};
use makosh_ai_inference_core::{
    AiAttachmentTranslationExecutionPlanV1, AiInferenceCoreErrorV1, AiInferenceRunStateV1,
    accept_attachment_translation_inference_v1, attachment_translation_inference_execution_plan_v1,
    begin_attachment_translation_inference_v1, build_attachment_translation_provider_input_v1,
    complete_attachment_translation_inference_v1, reject_attachment_translation_inference_v1,
};
use makosh_ai_inference_persistence::{
    AiAttachmentTranslationTransitionV1, AiInferencePersistenceErrorV1, AiInferencePersistenceV1,
    PersistedAiAttachmentTranslationRunV1,
};
use prost::Message;

use crate::{
    managed_ports::{
        AiInferenceExecutionPortsV1, AiInferenceProviderPortErrorV1, AiInferenceSourcePortErrorV1,
    },
    worker::AiInferenceWorkerErrorV1,
};

pub(crate) async fn execute_attachment_translation_payload_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Result<Vec<u8>, AiInferenceWorkerErrorV1> {
    let request = AttachmentTranslationInferenceRequestV1::decode(payload)
        .map_err(|_| AiInferenceWorkerErrorV1::InvalidRequest)?;
    if request.logical_owner_id != logical_owner_id
        || validate_attachment_translation_inference_request_v1(&request).is_err()
    {
        return Err(AiInferenceWorkerErrorV1::InvalidRequest);
    }
    let accepted = accept_attachment_translation_inference_v1(request).map_err(core_error)?;
    let persisted = persistence
        .accept_attachment_translation_run(accepted)
        .await
        .map_err(persistence_error)?
        .persisted;
    let terminal = drive_attachment_translation_run_v1(persistence, ports, persisted).await?;
    terminal
        .run
        .terminal_result
        .map(|result| result.encode_to_vec())
        .ok_or(AiInferenceWorkerErrorV1::Unavailable)
}

pub(crate) async fn recover_pending_attachment_translations_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    logical_owner_id: &str,
) -> Result<u32, AiInferenceWorkerErrorV1> {
    let pending = persistence
        .load_recoverable_attachment_translation_runs(logical_owner_id, 128)
        .await
        .map_err(persistence_error)?;
    let mut recovered = 0;
    for run in pending {
        match drive_attachment_translation_run_v1(persistence, ports, run).await {
            Ok(_) => recovered += 1,
            Err(AiInferenceWorkerErrorV1::Unavailable) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(recovered)
}

async fn drive_attachment_translation_run_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    mut persisted: PersistedAiAttachmentTranslationRunV1,
) -> Result<PersistedAiAttachmentTranslationRunV1, AiInferenceWorkerErrorV1> {
    if matches!(
        persisted.run.state,
        AiInferenceRunStateV1::Ready | AiInferenceRunStateV1::Rejected
    ) {
        return Ok(persisted);
    }
    let plan =
        attachment_translation_inference_execution_plan_v1(&persisted.run).map_err(core_error)?;
    let source = match ports.materialize_attachment_translation_source(&plan) {
        Ok(source) => source,
        Err(AiInferenceSourcePortErrorV1::InvalidReceipt) => {
            return reject_and_persist(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput,
            )
            .await;
        }
        Err(AiInferenceSourcePortErrorV1::Unavailable) => {
            return Err(AiInferenceWorkerErrorV1::Unavailable);
        }
    };
    let input = match build_attachment_translation_provider_input_v1(&plan, &source) {
        Ok(input) => input,
        Err(_) => {
            return reject_and_persist(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput,
            )
            .await;
        }
    };
    if persisted.run.state == AiInferenceRunStateV1::Accepted {
        let (executing, _) =
            begin_attachment_translation_inference_v1(&persisted.run, persisted.run.revision)
                .map_err(core_error)?;
        persisted = persistence
            .persist_attachment_translation_transition(AiAttachmentTranslationTransitionV1 {
                current_revision: persisted.run.revision,
                next_run: executing,
            })
            .await
            .map_err(persistence_error)?;
    }
    let provider_result = match ports.translate(provider_request(&plan, input)) {
        Ok(result) => result,
        Err(AiInferenceProviderPortErrorV1::Rejected) => {
            return reject_and_persist(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected,
            )
            .await;
        }
        Err(AiInferenceProviderPortErrorV1::Unavailable) => {
            return reject_and_persist(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable,
            )
            .await;
        }
    };
    let ready = complete_attachment_translation_inference_v1(
        &persisted.run,
        persisted.run.revision,
        provider_result,
    )
    .map_err(core_error)?;
    persistence
        .persist_attachment_translation_transition(AiAttachmentTranslationTransitionV1 {
            current_revision: persisted.run.revision,
            next_run: ready,
        })
        .await
        .map_err(persistence_error)
}

fn provider_request(
    plan: &AiAttachmentTranslationExecutionPlanV1,
    input_utf8: Vec<u8>,
) -> AiProviderTranslationRequestV1 {
    AiProviderTranslationRequestV1 {
        request_id: plan.run_id.to_vec(),
        input_utf8,
        target_language: plan.target_language,
        maximum_output_bytes: plan.maximum_output_bytes,
        maximum_output_tokens: plan.maximum_output_tokens,
        egress_policy: plan.egress_policy,
        egress_policy_revision: plan.egress_policy_revision,
    }
}

async fn reject_and_persist(
    persistence: &AiInferencePersistenceV1,
    persisted: PersistedAiAttachmentTranslationRunV1,
    status: AiInferenceTerminalStatusV1,
) -> Result<PersistedAiAttachmentTranslationRunV1, AiInferenceWorkerErrorV1> {
    let rejected =
        reject_attachment_translation_inference_v1(&persisted.run, persisted.run.revision, status)
            .map_err(core_error)?;
    persistence
        .persist_attachment_translation_transition(AiAttachmentTranslationTransitionV1 {
            current_revision: persisted.run.revision,
            next_run: rejected,
        })
        .await
        .map_err(persistence_error)
}

fn core_error(error: AiInferenceCoreErrorV1) -> AiInferenceWorkerErrorV1 {
    match error {
        AiInferenceCoreErrorV1::InvalidRequest
        | AiInferenceCoreErrorV1::InvalidProviderResult
        | AiInferenceCoreErrorV1::InvalidResult => AiInferenceWorkerErrorV1::InvalidRequest,
        AiInferenceCoreErrorV1::RevisionConflict | AiInferenceCoreErrorV1::InvalidTransition => {
            AiInferenceWorkerErrorV1::Conflict
        }
    }
}

fn persistence_error(error: AiInferencePersistenceErrorV1) -> AiInferenceWorkerErrorV1 {
    match error {
        AiInferencePersistenceErrorV1::StorageUnavailable => AiInferenceWorkerErrorV1::Unavailable,
        AiInferencePersistenceErrorV1::RequestConflict
        | AiInferencePersistenceErrorV1::RevisionConflict
        | AiInferencePersistenceErrorV1::InvalidTransition => AiInferenceWorkerErrorV1::Conflict,
        AiInferencePersistenceErrorV1::InvalidInput | AiInferencePersistenceErrorV1::InvalidRow => {
            AiInferenceWorkerErrorV1::InvalidRequest
        }
    }
}

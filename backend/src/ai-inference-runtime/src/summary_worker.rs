use makosh_ai_contracts::{
    validate_summary_inference_request_v1,
    wire::{
        AiInferenceTerminalStatusV1, AiProviderSummaryGenerationRequestV1,
        CommunicationSummaryInferenceRequestV1,
    },
};
use makosh_ai_inference_core::{
    AiInferenceCoreErrorV1, AiInferenceRunStateV1, AiSummaryExecutionPlanV1,
    accept_summary_inference_v1, begin_summary_inference_v1, build_summary_provider_input_v1,
    complete_summary_inference_v1, reject_summary_inference_v1,
    summary_inference_execution_plan_v1,
};
use makosh_ai_inference_persistence::{
    AiInferencePersistenceErrorV1, AiInferencePersistenceV1, AiSummaryTransitionV1,
    PersistedAiSummaryRunV1,
};
use prost::Message;

use crate::{
    managed_ports::{
        AiInferenceExecutionPortsV1, AiInferenceProviderPortErrorV1, AiInferenceSourcePortErrorV1,
    },
    worker::AiInferenceWorkerErrorV1,
};

pub(crate) async fn execute_summary_payload_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Result<Vec<u8>, AiInferenceWorkerErrorV1> {
    let request = CommunicationSummaryInferenceRequestV1::decode(payload)
        .map_err(|_| AiInferenceWorkerErrorV1::InvalidRequest)?;
    if request.logical_owner_id != logical_owner_id
        || validate_summary_inference_request_v1(&request).is_err()
    {
        return Err(AiInferenceWorkerErrorV1::InvalidRequest);
    }
    let accepted = accept_summary_inference_v1(request).map_err(core_error)?;
    let persisted = persistence
        .accept_summary_run(accepted)
        .await
        .map_err(persistence_error)?
        .persisted;
    let terminal = drive_summary_run_v1(persistence, ports, persisted).await?;
    terminal
        .run
        .terminal_result
        .map(|result| result.encode_to_vec())
        .ok_or(AiInferenceWorkerErrorV1::Unavailable)
}

pub(crate) async fn recover_pending_summaries_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    logical_owner_id: &str,
) -> Result<u32, AiInferenceWorkerErrorV1> {
    let pending = persistence
        .load_recoverable_summary_runs(logical_owner_id, 128)
        .await
        .map_err(persistence_error)?;
    let mut recovered = 0;
    for run in pending {
        match drive_summary_run_v1(persistence, ports, run).await {
            Ok(_) => recovered += 1,
            Err(AiInferenceWorkerErrorV1::Unavailable) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(recovered)
}

async fn drive_summary_run_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    mut persisted: PersistedAiSummaryRunV1,
) -> Result<PersistedAiSummaryRunV1, AiInferenceWorkerErrorV1> {
    if matches!(
        persisted.run.state,
        AiInferenceRunStateV1::Ready | AiInferenceRunStateV1::Rejected
    ) {
        return Ok(persisted);
    }
    let plan = summary_inference_execution_plan_v1(&persisted.run).map_err(core_error)?;
    let source = match ports.materialize_summary_source(&plan) {
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
    let input = match build_summary_provider_input_v1(&plan, &source) {
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
        let (executing, _) = begin_summary_inference_v1(&persisted.run, persisted.run.revision)
            .map_err(core_error)?;
        persisted = persistence
            .persist_summary_transition(AiSummaryTransitionV1 {
                current_revision: persisted.run.revision,
                next_run: executing,
            })
            .await
            .map_err(persistence_error)?;
    }
    let provider_result = match ports.generate_summary(provider_request(&plan, input)) {
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
    let ready =
        complete_summary_inference_v1(&persisted.run, persisted.run.revision, provider_result)
            .map_err(core_error)?;
    persistence
        .persist_summary_transition(AiSummaryTransitionV1 {
            current_revision: persisted.run.revision,
            next_run: ready,
        })
        .await
        .map_err(persistence_error)
}

fn provider_request(
    plan: &AiSummaryExecutionPlanV1,
    input_utf8: Vec<u8>,
) -> AiProviderSummaryGenerationRequestV1 {
    AiProviderSummaryGenerationRequestV1 {
        request_id: plan.run_id.to_vec(),
        input_utf8,
        language: plan.language,
        length: plan.length,
        maximum_output_bytes: plan.maximum_output_bytes,
        maximum_output_tokens: plan.maximum_output_tokens,
        egress_policy: plan.egress_policy,
        egress_policy_revision: plan.egress_policy_revision,
    }
}

async fn reject_and_persist(
    persistence: &AiInferencePersistenceV1,
    persisted: PersistedAiSummaryRunV1,
    status: AiInferenceTerminalStatusV1,
) -> Result<PersistedAiSummaryRunV1, AiInferenceWorkerErrorV1> {
    let rejected = reject_summary_inference_v1(&persisted.run, persisted.run.revision, status)
        .map_err(core_error)?;
    persistence
        .persist_summary_transition(AiSummaryTransitionV1 {
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

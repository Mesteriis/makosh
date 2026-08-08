use makosh_ai_contracts::{
    validate_reply_inference_request_v1,
    wire::{
        AiInferenceTerminalStatusV1, AiProviderReplyGenerationRequestV1,
        CommunicationReplySuggestionInferenceRequestV1,
    },
};
use makosh_ai_inference_core::{
    AiInferenceCoreErrorV1, AiInferenceExecutionPlanV1, AiInferenceRunStateV1,
    accept_reply_inference_v1, begin_reply_inference_v1, build_reply_provider_input_v1,
    complete_reply_inference_v1, reject_reply_inference_v1, reply_inference_execution_plan_v1,
};
use makosh_ai_inference_persistence::{
    AiInferencePersistenceErrorV1, AiInferencePersistenceV1, AiInferenceTransitionV1,
    PersistedAiInferenceRunV1,
};
use prost::Message;

use crate::managed_ports::{
    AiInferenceExecutionPortsV1, AiInferenceProviderPortErrorV1, AiInferenceSourcePortErrorV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiInferenceWorkerErrorV1 {
    InvalidRequest,
    Conflict,
    Unavailable,
}

pub(crate) async fn execute_payload_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Result<Vec<u8>, AiInferenceWorkerErrorV1> {
    let request = CommunicationReplySuggestionInferenceRequestV1::decode(payload)
        .map_err(|_| AiInferenceWorkerErrorV1::InvalidRequest)?;
    if request.logical_owner_id != logical_owner_id
        || validate_reply_inference_request_v1(&request).is_err()
    {
        return Err(AiInferenceWorkerErrorV1::InvalidRequest);
    }
    let accepted = accept_reply_inference_v1(request).map_err(core_error)?;
    let persisted = persistence
        .accept_run(accepted)
        .await
        .map_err(persistence_error)?
        .persisted;
    let terminal = drive_run_v1(persistence, ports, persisted).await?;
    terminal
        .run
        .terminal_result
        .map(|result| result.encode_to_vec())
        .ok_or(AiInferenceWorkerErrorV1::Unavailable)
}

pub(crate) async fn recover_pending_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    logical_owner_id: &str,
) -> Result<u32, AiInferenceWorkerErrorV1> {
    let pending = persistence
        .load_recoverable_runs(logical_owner_id, 1)
        .await
        .map_err(persistence_error)?;
    let mut recovered = 0_u32;
    for run in pending {
        match drive_run_v1(persistence, ports, run).await {
            Ok(_) => recovered += 1,
            Err(AiInferenceWorkerErrorV1::Unavailable) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(recovered)
}

async fn drive_run_v1(
    persistence: &AiInferencePersistenceV1,
    ports: &mut dyn AiInferenceExecutionPortsV1,
    mut persisted: PersistedAiInferenceRunV1,
) -> Result<PersistedAiInferenceRunV1, AiInferenceWorkerErrorV1> {
    if matches!(
        persisted.run.state,
        AiInferenceRunStateV1::Ready | AiInferenceRunStateV1::Rejected
    ) {
        return Ok(persisted);
    }
    let plan = reply_inference_execution_plan_v1(&persisted.run).map_err(core_error)?;
    let source = match ports.materialize_source(&plan) {
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
    if persisted.run.state == AiInferenceRunStateV1::Accepted {
        let (executing, _) =
            begin_reply_inference_v1(&persisted.run, persisted.run.revision).map_err(core_error)?;
        persisted = persistence
            .persist_transition(AiInferenceTransitionV1 {
                current_revision: persisted.run.revision,
                next_run: executing,
            })
            .await
            .map_err(persistence_error)?;
    }
    let provider_input = build_reply_provider_input_v1(&plan, &source).map_err(core_error)?;
    let provider_request = provider_request(&plan, provider_input);
    let provider_result = match ports.generate_reply(provider_request) {
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
        complete_reply_inference_v1(&persisted.run, persisted.run.revision, provider_result)
            .map_err(core_error)?;
    persistence
        .persist_transition(AiInferenceTransitionV1 {
            current_revision: persisted.run.revision,
            next_run: ready,
        })
        .await
        .map_err(persistence_error)
}

fn provider_request(
    plan: &AiInferenceExecutionPlanV1,
    input_utf8: Vec<u8>,
) -> AiProviderReplyGenerationRequestV1 {
    AiProviderReplyGenerationRequestV1 {
        request_id: plan.run_id.to_vec(),
        input_utf8,
        tone: plan.tone,
        language: plan.language,
        subject_policy: plan.subject_policy,
        maximum_output_bytes: plan.maximum_output_bytes,
        maximum_output_tokens: plan.maximum_output_tokens,
        egress_policy: plan.egress_policy,
        egress_policy_revision: plan.egress_policy_revision,
    }
}

async fn reject_and_persist(
    persistence: &AiInferencePersistenceV1,
    persisted: PersistedAiInferenceRunV1,
    status: AiInferenceTerminalStatusV1,
) -> Result<PersistedAiInferenceRunV1, AiInferenceWorkerErrorV1> {
    let rejected = reject_reply_inference_v1(&persisted.run, persisted.run.revision, status)
        .map_err(core_error)?;
    persistence
        .persist_transition(AiInferenceTransitionV1 {
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

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{
        AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        wire::{
            AiEgressPolicyV1, AiPrivateSourceReceiptV1, AiReplyLanguageV1, AiReplySubjectPolicyV1,
            AiReplyToneV1,
        },
    };

    use super::*;

    #[test]
    fn provider_request_contains_policy_and_source_but_no_provider_identity() {
        let plan = AiInferenceExecutionPlanV1 {
            run_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            request_digest: [2; 32],
            source: AiPrivateSourceReceiptV1 {
                reference_id: vec![3; 16],
                declared_bytes: 5,
                sha256: vec![4; 32],
                custody_transfer_source_proof: vec![5; 48],
            },
            tone: AiReplyToneV1::AiReplyToneWarm as i32,
            language: AiReplyLanguageV1::AiReplyLanguageRussian as i32,
            subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
            maximum_output_bytes: 4096,
            maximum_output_tokens: 512,
            egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
            egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        };
        let request = provider_request(&plan, b"private source".to_vec());
        assert_eq!(request.request_id, plan.run_id);
        assert_eq!(request.input_utf8, b"private source");
        assert_eq!(request.egress_policy_revision, 1);
    }
}

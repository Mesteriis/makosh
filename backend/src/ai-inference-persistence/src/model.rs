use makosh_ai_contracts::{
    validate_reply_inference_request_v1, validate_reply_inference_result_v1,
};
use makosh_ai_inference_core::{AiInferenceRunStateV1, AiInferenceRunV1};

pub const AI_INFERENCE_RECOVERY_LIMIT_V1: u32 = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedAiInferenceRunV1 {
    pub run: AiInferenceRunV1,
    pub selected_provider_settings_revision: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiInferenceTransitionV1 {
    pub current_revision: u64,
    pub next_run: AiInferenceRunV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiInferencePersistenceOutcomeV1 {
    pub persisted: PersistedAiInferenceRunV1,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiInferencePersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    RevisionConflict,
    InvalidTransition,
}

pub(crate) fn validate_accepted(
    run: &AiInferenceRunV1,
) -> Result<(), AiInferencePersistenceErrorV1> {
    validate_run(run)?;
    if run.revision != 1
        || run.state != AiInferenceRunStateV1::Accepted
        || run.terminal_result.is_some()
    {
        return Err(AiInferencePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_run(run: &AiInferenceRunV1) -> Result<(), AiInferencePersistenceErrorV1> {
    validate_reply_inference_request_v1(&run.request)
        .map_err(|_| AiInferencePersistenceErrorV1::InvalidInput)?;
    if run.revision == 0 {
        return Err(AiInferencePersistenceErrorV1::InvalidInput);
    }
    match (&run.state, &run.terminal_result) {
        (AiInferenceRunStateV1::Accepted | AiInferenceRunStateV1::Executing, None) => Ok(()),
        (AiInferenceRunStateV1::Ready | AiInferenceRunStateV1::Rejected, Some(result)) => {
            validate_reply_inference_result_v1(result)
                .map_err(|_| AiInferencePersistenceErrorV1::InvalidInput)
        }
        _ => Err(AiInferencePersistenceErrorV1::InvalidInput),
    }
}

pub(crate) fn validate_transition(
    current: &PersistedAiInferenceRunV1,
    transition: &AiInferenceTransitionV1,
) -> Result<Option<u64>, AiInferencePersistenceErrorV1> {
    if current.run.revision != transition.current_revision
        || transition.next_run.revision != transition.current_revision + 1
        || current.run.request != transition.next_run.request
    {
        return Err(AiInferencePersistenceErrorV1::RevisionConflict);
    }
    let selected = match (current.run.state, transition.next_run.state) {
        (AiInferenceRunStateV1::Accepted, AiInferenceRunStateV1::Executing)
        | (AiInferenceRunStateV1::Accepted, AiInferenceRunStateV1::Rejected)
        | (AiInferenceRunStateV1::Executing, AiInferenceRunStateV1::Rejected) => None,
        (AiInferenceRunStateV1::Executing, AiInferenceRunStateV1::Ready) => transition
            .next_run
            .terminal_result
            .as_ref()
            .and_then(|value| value.inference_receipt.as_ref())
            .map(|receipt| receipt.provider_settings_revision),
        _ => return Err(AiInferencePersistenceErrorV1::InvalidTransition),
    };
    if current.selected_provider_settings_revision.is_some()
        || transition.next_run.state == AiInferenceRunStateV1::Ready && selected.is_none()
    {
        return Err(AiInferencePersistenceErrorV1::InvalidTransition);
    }
    Ok(selected)
}

pub(crate) fn validate_persisted_settings(
    persisted: &PersistedAiInferenceRunV1,
) -> Result<(), AiInferencePersistenceErrorV1> {
    match persisted.run.state {
        AiInferenceRunStateV1::Accepted | AiInferenceRunStateV1::Executing
            if persisted.selected_provider_settings_revision.is_none() =>
        {
            Ok(())
        }
        AiInferenceRunStateV1::Ready if persisted.selected_provider_settings_revision.is_some() => {
            Ok(())
        }
        AiInferenceRunStateV1::Rejected
            if persisted.selected_provider_settings_revision.is_none() =>
        {
            Ok(())
        }
        _ => Err(AiInferencePersistenceErrorV1::InvalidRow),
    }
}

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{
        AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
        AI_LOCAL_EGRESS_POLICY_REVISION_V1, seal_reply_inference_request_v1,
        wire::{
            AiContextReceiptV1, AiEgressPolicyV1, AiInferenceTerminalStatusV1,
            AiPrivateSourceReceiptV1, AiReplyLanguageV1, AiReplySubjectPolicyV1, AiReplyToneV1,
            AiUseCaseV1, CommunicationReplySuggestionInferenceRequestV1,
        },
    };
    use makosh_ai_inference_core::{
        accept_reply_inference_v1, begin_reply_inference_v1, reject_reply_inference_v1,
    };

    use super::*;

    fn accepted() -> AiInferenceRunV1 {
        let request =
            seal_reply_inference_request_v1(CommunicationReplySuggestionInferenceRequestV1 {
                run_id: vec![1; 16],
                context: Some(AiContextReceiptV1 {
                    context_id: vec![2; 16],
                    use_case: AiUseCaseV1::AiUseCaseCommunicationReplySuggestion as i32,
                    source_evidence_id: vec![3; 16],
                    source_evidence_revision: 4,
                    contract_major: AI_CONTRACT_MAJOR_V1,
                    contract_revision: AI_CONTRACT_REVISION_V1,
                    contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
                    request_digest: Vec::new(),
                }),
                source: Some(AiPrivateSourceReceiptV1 {
                    reference_id: vec![5; 16],
                    declared_bytes: 6,
                    sha256: vec![7; 32],
                    custody_transfer_source_proof: vec![8; 64],
                }),
                tone: AiReplyToneV1::AiReplyToneWarm as i32,
                language: AiReplyLanguageV1::AiReplyLanguageAuto as i32,
                subject_policy: AiReplySubjectPolicyV1::AiReplySubjectPolicyPreserve as i32,
                maximum_output_bytes: 4_096,
                maximum_output_tokens: 512,
                egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
                egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
                logical_owner_id: "owner-1".to_owned(),
            })
            .expect("request");
        accept_reply_inference_v1(request).expect("accepted")
    }

    #[test]
    fn provider_settings_are_not_fabricated_before_execution() {
        let accepted = PersistedAiInferenceRunV1 {
            run: accepted(),
            selected_provider_settings_revision: None,
        };
        let (executing, _) = begin_reply_inference_v1(&accepted.run, 1).expect("executing");
        let selected = validate_transition(
            &accepted,
            &AiInferenceTransitionV1 {
                current_revision: 1,
                next_run: executing,
            },
        )
        .expect("transition");
        assert_eq!(selected, None);
    }

    #[test]
    fn accepted_rejection_has_no_provider_settings() {
        let accepted = PersistedAiInferenceRunV1 {
            run: accepted(),
            selected_provider_settings_revision: None,
        };
        let rejected = reject_reply_inference_v1(
            &accepted.run,
            1,
            AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy,
        )
        .expect("rejected");
        assert_eq!(
            validate_transition(
                &accepted,
                &AiInferenceTransitionV1 {
                    current_revision: 1,
                    next_run: rejected,
                },
            ),
            Ok(None)
        );
    }
}

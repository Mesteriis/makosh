use makosh_ai_contracts::{
    validate_translation_inference_request_v1, validate_translation_inference_result_v1,
};
use makosh_ai_inference_core::{AiInferenceRunStateV1, AiTranslationRunV1};

use crate::AiInferencePersistenceErrorV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedAiTranslationRunV1 {
    pub run: AiTranslationRunV1,
    pub selected_provider_settings_revision: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiTranslationTransitionV1 {
    pub current_revision: u64,
    pub next_run: AiTranslationRunV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiTranslationPersistenceOutcomeV1 {
    pub persisted: PersistedAiTranslationRunV1,
    pub replayed: bool,
}

pub(crate) fn validate_translation_accepted(
    run: &AiTranslationRunV1,
) -> Result<(), AiInferencePersistenceErrorV1> {
    validate_translation_run(run)?;
    if run.revision != 1
        || run.state != AiInferenceRunStateV1::Accepted
        || run.terminal_result.is_some()
    {
        return Err(AiInferencePersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_translation_run(
    run: &AiTranslationRunV1,
) -> Result<(), AiInferencePersistenceErrorV1> {
    validate_translation_inference_request_v1(&run.request)
        .map_err(|_| AiInferencePersistenceErrorV1::InvalidInput)?;
    if run.revision == 0 {
        return Err(AiInferencePersistenceErrorV1::InvalidInput);
    }
    match (&run.state, &run.terminal_result) {
        (AiInferenceRunStateV1::Accepted | AiInferenceRunStateV1::Executing, None) => Ok(()),
        (AiInferenceRunStateV1::Ready | AiInferenceRunStateV1::Rejected, Some(result)) => {
            validate_translation_inference_result_v1(result)
                .map_err(|_| AiInferencePersistenceErrorV1::InvalidInput)
        }
        _ => Err(AiInferencePersistenceErrorV1::InvalidInput),
    }
}

pub(crate) fn validate_translation_transition(
    current: &PersistedAiTranslationRunV1,
    transition: &AiTranslationTransitionV1,
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
            .and_then(|result| result.inference_receipt.as_ref())
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

pub(crate) fn validate_translation_persisted(
    persisted: &PersistedAiTranslationRunV1,
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

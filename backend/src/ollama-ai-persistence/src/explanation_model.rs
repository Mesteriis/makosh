use makosh_ai_contracts::validate_provider_explanation_result_v1;
use makosh_ollama_ai_core::{OllamaAiRunStateV1, OllamaExplanationRunV1};

use crate::{OllamaAiPersistenceErrorV1, model::validate_owner};

#[derive(Clone, PartialEq)]
pub struct PersistedOllamaExplanationRunV1 {
    pub logical_owner_id: String,
    pub run: OllamaExplanationRunV1,
}

#[derive(Clone, PartialEq)]
pub struct OllamaExplanationTransitionV1 {
    pub logical_owner_id: String,
    pub current_revision: u64,
    pub next_run: OllamaExplanationRunV1,
}

#[derive(Clone, PartialEq)]
pub struct OllamaExplanationPersistenceOutcomeV1 {
    pub persisted: PersistedOllamaExplanationRunV1,
    pub replayed: bool,
}

pub(crate) fn validate_explanation_run(
    run: &OllamaExplanationRunV1,
) -> Result<(), OllamaAiPersistenceErrorV1> {
    let model_binding_valid = match run.state {
        OllamaAiRunStateV1::Accepted => run.selected_model_digest.is_none(),
        OllamaAiRunStateV1::Executing
        | OllamaAiRunStateV1::Ready
        | OllamaAiRunStateV1::Uncertain => run.selected_model_digest.is_some(),
        OllamaAiRunStateV1::Rejected => true,
    };
    if run.request_id == [0; 16]
        || run.request_digest == [0; 32]
        || run.settings_revision == 0
        || !model_binding_valid
        || run.revision == 0
    {
        return Err(OllamaAiPersistenceErrorV1::InvalidInput);
    }
    match (run.state, run.terminal_result.as_ref()) {
        (
            OllamaAiRunStateV1::Accepted
            | OllamaAiRunStateV1::Executing
            | OllamaAiRunStateV1::Uncertain,
            None,
        ) => Ok(()),
        (OllamaAiRunStateV1::Ready, Some(result))
            if result.request_id == run.request_id
                && run.selected_model_digest.as_ref().is_some_and(|digest| {
                    result.model_revision_sha256.as_slice() == digest.as_slice()
                })
                && result.provider_settings_revision == run.settings_revision
                && validate_provider_explanation_result_v1(result).is_ok() =>
        {
            Ok(())
        }
        (OllamaAiRunStateV1::Rejected, Some(result))
            if result.request_id == run.request_id
                && validate_provider_explanation_result_v1(result).is_ok() =>
        {
            Ok(())
        }
        _ => Err(OllamaAiPersistenceErrorV1::InvalidInput),
    }
}

pub(crate) fn validate_explanation_accepted(
    logical_owner_id: &str,
    run: &OllamaExplanationRunV1,
) -> Result<(), OllamaAiPersistenceErrorV1> {
    validate_explanation_run(run)?;
    if !validate_owner(logical_owner_id)
        || run.revision != 1
        || run.state != OllamaAiRunStateV1::Accepted
        || run.terminal_result.is_some()
    {
        return Err(OllamaAiPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_explanation_transition(
    current: &PersistedOllamaExplanationRunV1,
    transition: &OllamaExplanationTransitionV1,
) -> Result<(), OllamaAiPersistenceErrorV1> {
    validate_explanation_run(&transition.next_run)?;
    if current.logical_owner_id != transition.logical_owner_id
        || current.run.revision != transition.current_revision
        || transition.next_run.revision != transition.current_revision + 1
        || current.run.request_id != transition.next_run.request_id
        || current.run.request_digest != transition.next_run.request_digest
        || current.run.settings_revision != transition.next_run.settings_revision
        || current.run.selected_model_digest.is_some()
            && current.run.selected_model_digest != transition.next_run.selected_model_digest
    {
        return Err(OllamaAiPersistenceErrorV1::RevisionConflict);
    }
    if !matches!(
        (current.run.state, transition.next_run.state),
        (OllamaAiRunStateV1::Accepted, OllamaAiRunStateV1::Executing)
            | (OllamaAiRunStateV1::Accepted, OllamaAiRunStateV1::Rejected)
            | (OllamaAiRunStateV1::Executing, OllamaAiRunStateV1::Ready)
            | (OllamaAiRunStateV1::Executing, OllamaAiRunStateV1::Rejected)
            | (OllamaAiRunStateV1::Executing, OllamaAiRunStateV1::Uncertain)
    ) {
        return Err(OllamaAiPersistenceErrorV1::InvalidTransition);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted() -> OllamaExplanationRunV1 {
        OllamaExplanationRunV1 {
            request_id: [1; 16],
            request_digest: [2; 32],
            settings_revision: 3,
            selected_model_digest: None,
            revision: 1,
            state: OllamaAiRunStateV1::Accepted,
            terminal_result: None,
        }
    }

    #[test]
    fn explanation_lifecycle_requires_exact_revision_and_model_binding() {
        let current = PersistedOllamaExplanationRunV1 {
            logical_owner_id: "owner-1".to_owned(),
            run: accepted(),
        };
        let executing = OllamaExplanationRunV1 {
            selected_model_digest: Some([4; 32]),
            revision: 2,
            state: OllamaAiRunStateV1::Executing,
            ..accepted()
        };
        assert_eq!(
            validate_explanation_transition(
                &current,
                &OllamaExplanationTransitionV1 {
                    logical_owner_id: "owner-1".to_owned(),
                    current_revision: 1,
                    next_run: executing,
                }
            ),
            Ok(())
        );
    }
}

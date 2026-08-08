use makosh_ai_contracts::validate_provider_translation_result_v1;
use makosh_ollama_ai_core::{OllamaAiRunStateV1, OllamaTranslationRunV1};

use crate::{OllamaAiPersistenceErrorV1, model::validate_owner};

#[derive(Clone, Eq, PartialEq)]
pub struct PersistedOllamaTranslationRunV1 {
    pub logical_owner_id: String,
    pub run: OllamaTranslationRunV1,
}

#[derive(Clone, Eq, PartialEq)]
pub struct OllamaTranslationTransitionV1 {
    pub logical_owner_id: String,
    pub current_revision: u64,
    pub next_run: OllamaTranslationRunV1,
}

#[derive(Clone, Eq, PartialEq)]
pub struct OllamaTranslationPersistenceOutcomeV1 {
    pub persisted: PersistedOllamaTranslationRunV1,
    pub replayed: bool,
}

pub(crate) fn validate_translation_run(
    run: &OllamaTranslationRunV1,
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
                && validate_provider_translation_result_v1(result).is_ok() =>
        {
            Ok(())
        }
        (OllamaAiRunStateV1::Rejected, Some(result))
            if result.request_id == run.request_id
                && validate_provider_translation_result_v1(result).is_ok() =>
        {
            Ok(())
        }
        _ => Err(OllamaAiPersistenceErrorV1::InvalidInput),
    }
}

pub(crate) fn validate_translation_accepted(
    logical_owner_id: &str,
    run: &OllamaTranslationRunV1,
) -> Result<(), OllamaAiPersistenceErrorV1> {
    validate_translation_run(run)?;
    if !validate_owner(logical_owner_id)
        || run.revision != 1
        || run.state != OllamaAiRunStateV1::Accepted
        || run.terminal_result.is_some()
    {
        return Err(OllamaAiPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_translation_transition(
    current: &PersistedOllamaTranslationRunV1,
    transition: &OllamaTranslationTransitionV1,
) -> Result<(), OllamaAiPersistenceErrorV1> {
    validate_translation_run(&transition.next_run)?;
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

    fn accepted() -> OllamaTranslationRunV1 {
        OllamaTranslationRunV1 {
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
    fn translation_lifecycle_requires_exact_revision_and_model_binding() {
        let current = PersistedOllamaTranslationRunV1 {
            logical_owner_id: "owner-1".to_owned(),
            run: accepted(),
        };
        let executing = OllamaTranslationRunV1 {
            selected_model_digest: Some([4; 32]),
            revision: 2,
            state: OllamaAiRunStateV1::Executing,
            ..accepted()
        };
        assert_eq!(
            validate_translation_transition(
                &current,
                &OllamaTranslationTransitionV1 {
                    logical_owner_id: "owner-1".to_owned(),
                    current_revision: 1,
                    next_run: executing,
                }
            ),
            Ok(())
        );
    }
}

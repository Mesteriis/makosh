use makosh_ai_contracts::validate_provider_reply_generation_result_v1;
use makosh_ollama_ai_core::{OllamaAiRunStateV1, OllamaAiRunV1};

#[derive(Clone, Eq, PartialEq)]
pub struct PersistedOllamaAiRunV1 {
    pub logical_owner_id: String,
    pub run: OllamaAiRunV1,
}

#[derive(Clone, Eq, PartialEq)]
pub struct OllamaAiTransitionV1 {
    pub logical_owner_id: String,
    pub current_revision: u64,
    pub next_run: OllamaAiRunV1,
}

#[derive(Clone, Eq, PartialEq)]
pub struct OllamaAiPersistenceOutcomeV1 {
    pub persisted: PersistedOllamaAiRunV1,
    pub replayed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiPersistenceErrorV1 {
    InvalidInput,
    InvalidRow,
    StorageUnavailable,
    RequestConflict,
    RevisionConflict,
    InvalidTransition,
}

pub(crate) fn validate_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

pub(crate) fn validate_run(run: &OllamaAiRunV1) -> Result<(), OllamaAiPersistenceErrorV1> {
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
        (OllamaAiRunStateV1::Accepted | OllamaAiRunStateV1::Executing, None)
        | (OllamaAiRunStateV1::Uncertain, None) => Ok(()),
        (OllamaAiRunStateV1::Ready, Some(result))
            if result.request_id == run.request_id
                && run.selected_model_digest.as_ref().is_some_and(|digest| {
                    result.model_revision_sha256.as_slice() == digest.as_slice()
                })
                && result.provider_settings_revision == run.settings_revision
                && validate_provider_reply_generation_result_v1(result).is_ok() =>
        {
            Ok(())
        }
        (OllamaAiRunStateV1::Rejected, Some(result))
            if result.request_id == run.request_id
                && validate_provider_reply_generation_result_v1(result).is_ok() =>
        {
            Ok(())
        }
        _ => Err(OllamaAiPersistenceErrorV1::InvalidInput),
    }
}

pub(crate) fn validate_accepted(
    logical_owner_id: &str,
    run: &OllamaAiRunV1,
) -> Result<(), OllamaAiPersistenceErrorV1> {
    validate_run(run)?;
    if !validate_owner(logical_owner_id)
        || run.revision != 1
        || run.state != OllamaAiRunStateV1::Accepted
        || run.terminal_result.is_some()
    {
        return Err(OllamaAiPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) fn validate_transition(
    current: &PersistedOllamaAiRunV1,
    transition: &OllamaAiTransitionV1,
) -> Result<(), OllamaAiPersistenceErrorV1> {
    validate_run(&transition.next_run)?;
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
    use makosh_ollama_ai_core::OllamaAiRunStateV1;

    use super::*;

    fn accepted() -> OllamaAiRunV1 {
        OllamaAiRunV1 {
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
    fn only_exact_revision_fenced_lifecycle_is_persistable() {
        let current = PersistedOllamaAiRunV1 {
            logical_owner_id: "owner-1".to_owned(),
            run: accepted(),
        };
        let executing = OllamaAiRunV1 {
            selected_model_digest: Some([4; 32]),
            revision: 2,
            state: OllamaAiRunStateV1::Executing,
            ..accepted()
        };
        assert_eq!(
            validate_transition(
                &current,
                &OllamaAiTransitionV1 {
                    logical_owner_id: "owner-1".to_owned(),
                    current_revision: 1,
                    next_run: executing,
                }
            ),
            Ok(())
        );
    }
}

use makosh_ai_contracts::{
    validate_provider_explanation_request_v1,
    wire::{AiInferenceTerminalStatusV1, AiProviderExplanationRequestV1},
};
use makosh_ollama_ai_api::OllamaAiRuntimeSettingsV1;
use makosh_ollama_ai_core::{
    OllamaAiRunStateV1, OllamaExplanationRunV1, accept_ollama_explanation_request_v1,
    begin_ollama_explanation_request_v1, complete_ollama_explanation_request_v1,
    mark_ollama_explanation_uncertain_v1, reject_ollama_explanation_request_v1,
};
use makosh_ollama_ai_http::OllamaAiHttpErrorV1;
use makosh_ollama_ai_persistence::{
    OllamaAiPersistenceErrorV1, OllamaAiPersistenceV1, OllamaExplanationTransitionV1,
    PersistedOllamaExplanationRunV1,
};
use prost::Message;

use crate::worker::{OllamaAiExecutionPortV1, OllamaAiWorkerErrorV1};

pub(crate) async fn execute_explanation_payload_v1(
    persistence: &OllamaAiPersistenceV1,
    port: &mut impl OllamaAiExecutionPortV1,
    logical_owner_id: &str,
    settings: &OllamaAiRuntimeSettingsV1,
    payload: &[u8],
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    let request = AiProviderExplanationRequestV1::decode(payload)
        .map_err(|_| OllamaAiWorkerErrorV1::InvalidRequest)?;
    validate_provider_explanation_request_v1(&request)
        .map_err(|_| OllamaAiWorkerErrorV1::InvalidRequest)?;
    let accepted = accept_ollama_explanation_request_v1(&request, settings)
        .map_err(|_| OllamaAiWorkerErrorV1::InvalidRequest)?;
    let persisted = persistence
        .accept_explanation_run(logical_owner_id, accepted)
        .await
        .map_err(persistence_error)?
        .persisted;
    drive_run(persistence, port, persisted, &request, settings).await
}

async fn drive_run(
    persistence: &OllamaAiPersistenceV1,
    port: &mut impl OllamaAiExecutionPortV1,
    mut persisted: PersistedOllamaExplanationRunV1,
    request: &AiProviderExplanationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    if let Some(result) = &persisted.run.terminal_result {
        return Ok(result.encode_to_vec());
    }
    if persisted.run.state == OllamaAiRunStateV1::Uncertain {
        return Err(OllamaAiWorkerErrorV1::Uncertain);
    }
    if persisted.run.state == OllamaAiRunStateV1::Executing {
        let uncertain = mark_ollama_explanation_uncertain_v1(&persisted.run)
            .ok_or(OllamaAiWorkerErrorV1::Conflict)?;
        persist(persistence, &persisted, uncertain).await?;
        return Err(OllamaAiWorkerErrorV1::Uncertain);
    }
    let model = match port.discover(settings).await {
        Ok(model) if model.model == settings.chat_model => model,
        Ok(_) | Err(OllamaAiHttpErrorV1::ModelMismatch | OllamaAiHttpErrorV1::ModelUnavailable) => {
            return reject(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected,
            )
            .await;
        }
        Err(_) => {
            return reject(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable,
            )
            .await;
        }
    };
    let (executing, plan) =
        begin_ollama_explanation_request_v1(&persisted.run, request, settings, model.digest)
            .map_err(|_| OllamaAiWorkerErrorV1::Conflict)?;
    persisted = persist(persistence, &persisted, executing).await?;
    let generated = match port.generate_explanation(settings, &plan).await {
        Ok(value) => value,
        Err(OllamaAiHttpErrorV1::Unavailable) => return uncertain(persistence, persisted).await,
        Err(_) => {
            return reject(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected,
            )
            .await;
        }
    };
    let confirmed = match port.discover(settings).await {
        Ok(value) => value,
        Err(_) => return uncertain(persistence, persisted).await,
    };
    if confirmed.model != plan.model || confirmed.digest != plan.model_digest {
        return uncertain(persistence, persisted).await;
    }
    let ready = match complete_ollama_explanation_request_v1(&persisted.run, &plan, generated) {
        Ok(value) => value,
        Err(_) => {
            return reject(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected,
            )
            .await;
        }
    };
    let ready = persist(persistence, &persisted, ready).await?;
    Ok(ready
        .run
        .terminal_result
        .ok_or(OllamaAiWorkerErrorV1::Conflict)?
        .encode_to_vec())
}

async fn reject(
    persistence: &OllamaAiPersistenceV1,
    persisted: PersistedOllamaExplanationRunV1,
    status: AiInferenceTerminalStatusV1,
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    let next = reject_ollama_explanation_request_v1(&persisted.run, status)
        .ok_or(OllamaAiWorkerErrorV1::Conflict)?;
    let next = persist(persistence, &persisted, next).await?;
    Ok(next
        .run
        .terminal_result
        .ok_or(OllamaAiWorkerErrorV1::Conflict)?
        .encode_to_vec())
}

async fn uncertain(
    persistence: &OllamaAiPersistenceV1,
    persisted: PersistedOllamaExplanationRunV1,
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    let next = mark_ollama_explanation_uncertain_v1(&persisted.run)
        .ok_or(OllamaAiWorkerErrorV1::Conflict)?;
    persist(persistence, &persisted, next).await?;
    Err(OllamaAiWorkerErrorV1::Uncertain)
}

async fn persist(
    persistence: &OllamaAiPersistenceV1,
    current: &PersistedOllamaExplanationRunV1,
    next_run: OllamaExplanationRunV1,
) -> Result<PersistedOllamaExplanationRunV1, OllamaAiWorkerErrorV1> {
    persistence
        .persist_explanation_transition(OllamaExplanationTransitionV1 {
            logical_owner_id: current.logical_owner_id.clone(),
            current_revision: current.run.revision,
            next_run,
        })
        .await
        .map_err(persistence_error)
}

fn persistence_error(error: OllamaAiPersistenceErrorV1) -> OllamaAiWorkerErrorV1 {
    match error {
        OllamaAiPersistenceErrorV1::RequestConflict
        | OllamaAiPersistenceErrorV1::RevisionConflict
        | OllamaAiPersistenceErrorV1::InvalidTransition => OllamaAiWorkerErrorV1::Conflict,
        OllamaAiPersistenceErrorV1::InvalidInput | OllamaAiPersistenceErrorV1::InvalidRow => {
            OllamaAiWorkerErrorV1::InvalidRequest
        }
        OllamaAiPersistenceErrorV1::StorageUnavailable => OllamaAiWorkerErrorV1::Unavailable,
    }
}

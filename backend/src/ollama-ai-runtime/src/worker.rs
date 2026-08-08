use makosh_ai_contracts::{
    validate_provider_reply_generation_request_v1,
    wire::{AiInferenceTerminalStatusV1, AiProviderReplyGenerationRequestV1},
};
use makosh_ollama_ai_api::OllamaAiRuntimeSettingsV1;
use makosh_ollama_ai_core::{
    OllamaAiRunStateV1, OllamaGenerationPlanV1, OllamaHttpGenerationV1, accept_ollama_request_v1,
    begin_ollama_request_v1, complete_ollama_request_v1, mark_ollama_uncertain_v1,
    reject_ollama_request_v1,
};
use makosh_ollama_ai_http::{
    OllamaAiHttpErrorV1, OllamaModelRevisionV1, discover_model_revision_v1, generate_reply_v1,
};
use makosh_ollama_ai_persistence::{
    OllamaAiPersistenceErrorV1, OllamaAiPersistenceV1, OllamaAiTransitionV1, PersistedOllamaAiRunV1,
};
use prost::Message;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OllamaAiWorkerErrorV1 {
    InvalidRequest,
    Conflict,
    Unavailable,
    Uncertain,
}

pub(crate) trait OllamaAiExecutionPortV1 {
    async fn discover(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
    ) -> Result<OllamaModelRevisionV1, OllamaAiHttpErrorV1>;

    async fn generate(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &OllamaGenerationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1>;

    async fn generate_summary(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &makosh_ollama_ai_core::OllamaSummaryGenerationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1>;

    async fn generate_translation(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &makosh_ollama_ai_core::OllamaTranslationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1>;

    async fn generate_explanation(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &makosh_ollama_ai_core::OllamaExplanationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1>;
}

pub(crate) struct LocalOllamaAiExecutionPortV1;

impl OllamaAiExecutionPortV1 for LocalOllamaAiExecutionPortV1 {
    async fn discover(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
    ) -> Result<OllamaModelRevisionV1, OllamaAiHttpErrorV1> {
        discover_model_revision_v1(settings).await
    }

    async fn generate(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &OllamaGenerationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
        generate_reply_v1(settings, plan).await
    }

    async fn generate_summary(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &makosh_ollama_ai_core::OllamaSummaryGenerationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
        makosh_ollama_ai_http::generate_summary_v1(settings, plan).await
    }

    async fn generate_translation(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &makosh_ollama_ai_core::OllamaTranslationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
        makosh_ollama_ai_http::generate_translation_v1(settings, plan).await
    }

    async fn generate_explanation(
        &mut self,
        settings: &OllamaAiRuntimeSettingsV1,
        plan: &makosh_ollama_ai_core::OllamaExplanationPlanV1,
    ) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
        makosh_ollama_ai_http::generate_explanation_v1(settings, plan).await
    }
}

pub(crate) async fn execute_payload_v1(
    persistence: &OllamaAiPersistenceV1,
    port: &mut impl OllamaAiExecutionPortV1,
    logical_human_owner_id: &str,
    settings: &OllamaAiRuntimeSettingsV1,
    payload: &[u8],
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    let request = AiProviderReplyGenerationRequestV1::decode(payload)
        .map_err(|_| OllamaAiWorkerErrorV1::InvalidRequest)?;
    validate_provider_reply_generation_request_v1(&request)
        .map_err(|_| OllamaAiWorkerErrorV1::InvalidRequest)?;
    let accepted = accept_ollama_request_v1(&request, settings)
        .map_err(|_| OllamaAiWorkerErrorV1::InvalidRequest)?;
    let outcome = persistence
        .accept_run(logical_human_owner_id, accepted)
        .await
        .map_err(persistence_error_v1)?;
    drive_run_v1(persistence, port, outcome.persisted, &request, settings).await
}

async fn drive_run_v1(
    persistence: &OllamaAiPersistenceV1,
    port: &mut impl OllamaAiExecutionPortV1,
    mut persisted: PersistedOllamaAiRunV1,
    request: &AiProviderReplyGenerationRequestV1,
    settings: &OllamaAiRuntimeSettingsV1,
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    if let Some(result) = &persisted.run.terminal_result {
        return Ok(result.encode_to_vec());
    }
    if persisted.run.state == OllamaAiRunStateV1::Uncertain {
        return Err(OllamaAiWorkerErrorV1::Uncertain);
    }
    if persisted.run.state == OllamaAiRunStateV1::Executing {
        let uncertain =
            mark_ollama_uncertain_v1(&persisted.run).ok_or(OllamaAiWorkerErrorV1::Conflict)?;
        persist_transition_v1(persistence, &persisted, uncertain).await?;
        return Err(OllamaAiWorkerErrorV1::Uncertain);
    }
    let model = match port.discover(settings).await {
        Ok(model) if model.model == settings.chat_model => model,
        Ok(_) | Err(OllamaAiHttpErrorV1::ModelMismatch | OllamaAiHttpErrorV1::ModelUnavailable) => {
            return reject_and_encode_v1(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected,
            )
            .await;
        }
        Err(_) => {
            return reject_and_encode_v1(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable,
            )
            .await;
        }
    };
    let (executing, plan) =
        begin_ollama_request_v1(&persisted.run, request, settings, model.digest)
            .map_err(|_| OllamaAiWorkerErrorV1::Conflict)?;
    persisted = persist_transition_v1(persistence, &persisted, executing).await?;
    let generated = match port.generate(settings, &plan).await {
        Ok(generated) => generated,
        Err(OllamaAiHttpErrorV1::Unavailable) => {
            return mark_uncertain_v1(persistence, persisted).await;
        }
        Err(_) => {
            return reject_and_encode_v1(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected,
            )
            .await;
        }
    };
    let confirmed = match port.discover(settings).await {
        Ok(confirmed) => confirmed,
        Err(_) => return mark_uncertain_v1(persistence, persisted).await,
    };
    if confirmed.model != plan.model || confirmed.digest != plan.model_digest {
        return mark_uncertain_v1(persistence, persisted).await;
    }
    let ready = match complete_ollama_request_v1(&persisted.run, &plan, generated) {
        Ok(ready) => ready,
        Err(_) => {
            return reject_and_encode_v1(
                persistence,
                persisted,
                AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected,
            )
            .await;
        }
    };
    let ready = persist_transition_v1(persistence, &persisted, ready).await?;
    Ok(ready
        .run
        .terminal_result
        .ok_or(OllamaAiWorkerErrorV1::Conflict)?
        .encode_to_vec())
}

async fn reject_and_encode_v1(
    persistence: &OllamaAiPersistenceV1,
    persisted: PersistedOllamaAiRunV1,
    status: AiInferenceTerminalStatusV1,
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    let rejected =
        reject_ollama_request_v1(&persisted.run, status).ok_or(OllamaAiWorkerErrorV1::Conflict)?;
    let rejected = persist_transition_v1(persistence, &persisted, rejected).await?;
    Ok(rejected
        .run
        .terminal_result
        .ok_or(OllamaAiWorkerErrorV1::Conflict)?
        .encode_to_vec())
}

async fn mark_uncertain_v1(
    persistence: &OllamaAiPersistenceV1,
    persisted: PersistedOllamaAiRunV1,
) -> Result<Vec<u8>, OllamaAiWorkerErrorV1> {
    let uncertain =
        mark_ollama_uncertain_v1(&persisted.run).ok_or(OllamaAiWorkerErrorV1::Conflict)?;
    persist_transition_v1(persistence, &persisted, uncertain).await?;
    Err(OllamaAiWorkerErrorV1::Uncertain)
}

async fn persist_transition_v1(
    persistence: &OllamaAiPersistenceV1,
    current: &PersistedOllamaAiRunV1,
    next_run: makosh_ollama_ai_core::OllamaAiRunV1,
) -> Result<PersistedOllamaAiRunV1, OllamaAiWorkerErrorV1> {
    persistence
        .persist_transition(OllamaAiTransitionV1 {
            logical_owner_id: current.logical_owner_id.clone(),
            current_revision: current.run.revision,
            next_run,
        })
        .await
        .map_err(persistence_error_v1)
}

fn persistence_error_v1(error: OllamaAiPersistenceErrorV1) -> OllamaAiWorkerErrorV1 {
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

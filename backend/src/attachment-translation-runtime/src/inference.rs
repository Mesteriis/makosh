use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    attachment_translation_inference_contract_reference_v1,
    validate_attachment_translation_inference_request_v1,
    validate_attachment_translation_inference_result_v1,
    wire::{
        AiDetectedLanguageV1, AiInferenceCompletenessV1, AiInferenceTerminalStatusV1,
        AiTranslationLanguageV1, AttachmentTranslationInferenceRequestV1,
        AttachmentTranslationInferenceResultV1,
    },
};
use makosh_attachment_translation_core::{
    AttachmentTranslationCompletenessV1, AttachmentTranslationDetectedLanguageV1,
    AttachmentTranslationLanguageV1, AttachmentTranslationPendingResultV1,
    AttachmentTranslationRejectionCodeV1, AttachmentTranslationTransitionV1,
};
use makosh_attachment_translation_persistence::{
    AttachmentTranslationInferenceResultV1 as PersistInferenceResultV1,
    AttachmentTranslationMaterializationResultV1, AttachmentTranslationPersistenceErrorV1,
    AttachmentTranslationPersistenceV1, PersistedAttachmentTranslationRunV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        validate_module_request_request_v1, validate_module_request_response_v1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::blob_materialization::{
    AttachmentTranslationBlobErrorV1, materialize_translation_result_v1,
};

const INFERENCE_DEADLINE_MILLIS_V1: u32 = 30_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentTranslationInferenceErrorV1 {
    InvalidRequest,
    InvalidResult,
    Blob(AttachmentTranslationBlobErrorV1),
    Persistence(AttachmentTranslationPersistenceErrorV1),
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentTranslationInferenceExecutionV1 {
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub occurred_at_unix_millis: i64,
}

pub async fn complete_attachment_translation_inference_v1(
    persistence: &AttachmentTranslationPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    run: &PersistedAttachmentTranslationRunV1,
    request: AttachmentTranslationInferenceRequestV1,
    execution: AttachmentTranslationInferenceExecutionV1,
) -> Result<bool, AttachmentTranslationInferenceErrorV1> {
    validate_request_for_run(run, &request)?;
    let result = match route_inference(channel, dispatcher, request) {
        Ok(result) => result,
        Err(RouteInferenceErrorV1::Rejected) => {
            persist_rejection(
                persistence,
                run,
                AttachmentTranslationRejectionCodeV1::InferenceRejected,
                execution.occurred_at_unix_millis,
            )
            .await?;
            return Ok(false);
        }
        Err(RouteInferenceErrorV1::Unavailable) => {
            return Err(AttachmentTranslationInferenceErrorV1::Unavailable);
        }
    };
    let terminal = terminal_result(run, result)?;
    match terminal {
        AttachmentTranslationTerminalResultV1::Ready { pending, bytes } => {
            let inference_message_id =
                inference_message_id(run.draft.run_id, pending.inference_request_digest);
            persistence
                .persist_inference_result(PersistInferenceResultV1 {
                    message_id: inference_message_id,
                    envelope_sha256: Sha256::digest(&bytes).into(),
                    logical_owner_id: run.logical_owner_id.clone(),
                    run_id: run.draft.run_id,
                    transition: AttachmentTranslationTransitionV1::InferenceCompleted(pending),
                    occurred_at_unix_millis: execution.occurred_at_unix_millis,
                })
                .await
                .map_err(AttachmentTranslationInferenceErrorV1::Persistence)?;
            let artifact =
                materialize_translation_result_v1(channel, dispatcher, run.draft.run_id, &bytes)
                    .map_err(AttachmentTranslationInferenceErrorV1::Blob)?;
            persistence
                .persist_materialization_result(AttachmentTranslationMaterializationResultV1 {
                    message_id: materialization_message_id(run.draft.run_id, artifact.sha256),
                    result_sha256: artifact.sha256,
                    logical_owner_id: run.logical_owner_id.clone(),
                    run_id: run.draft.run_id,
                    transition: AttachmentTranslationTransitionV1::ResultMaterialized {
                        artifact_id: artifact.reference_id,
                    },
                    runtime_generation: execution.runtime_generation,
                    grant_epoch: execution.grant_epoch,
                    occurred_at_unix_millis: execution.occurred_at_unix_millis,
                })
                .await
                .map_err(AttachmentTranslationInferenceErrorV1::Persistence)?;
            Ok(true)
        }
        AttachmentTranslationTerminalResultV1::Rejected(code) => {
            persist_rejection(persistence, run, code, execution.occurred_at_unix_millis).await?;
            Ok(false)
        }
    }
}

pub(crate) fn validate_request_for_run(
    run: &PersistedAttachmentTranslationRunV1,
    request: &AttachmentTranslationInferenceRequestV1,
) -> Result<(), AttachmentTranslationInferenceErrorV1> {
    validate_attachment_translation_inference_request_v1(request)
        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidRequest)?;
    let context = request
        .context
        .as_ref()
        .ok_or(AttachmentTranslationInferenceErrorV1::InvalidRequest)?;
    if request.run_id.as_slice() != run.draft.run_id
        || request.logical_owner_id != run.logical_owner_id
        || context.request_digest.as_slice()
            != run
                .status
                .inference_request_digest
                .ok_or(AttachmentTranslationInferenceErrorV1::InvalidRequest)?
        || request
            .source
            .as_ref()
            .map(|source| source.sha256.as_slice())
            != run.status.source_sha256.as_ref().map(<[u8; 32]>::as_slice)
    {
        return Err(AttachmentTranslationInferenceErrorV1::InvalidRequest);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteInferenceErrorV1 {
    Rejected,
    Unavailable,
}

fn route_inference(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: AttachmentTranslationInferenceRequestV1,
) -> Result<AttachmentTranslationInferenceResultV1, RouteInferenceErrorV1> {
    let request_id: [u8; 16] = request
        .run_id
        .as_slice()
        .try_into()
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    let routed = ManagedRuntimeModuleRequestRequestV1 {
        request_id: request_id.to_vec(),
        contract: Some(attachment_translation_inference_contract_reference_v1()),
        request_payload: request.encode_to_vec(),
        deadline_millis: INFERENCE_DEADLINE_MILLIS_V1,
        response_blob_capability_id: String::new(),
    };
    validate_module_request_request_v1(&routed).map_err(|_| RouteInferenceErrorV1::Rejected)?;
    let response = channel
        .request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::RouteModuleRequest(routed)),
            },
            dispatcher,
        )
        .map_err(|_| RouteInferenceErrorV1::Unavailable)?;
    if !response.error_code.is_empty() {
        return Err(RouteInferenceErrorV1::Unavailable);
    }
    let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
        return Err(RouteInferenceErrorV1::Unavailable);
    };
    validate_module_request_response_v1(&response)
        .map_err(|_| RouteInferenceErrorV1::Unavailable)?;
    if response.request_id.as_slice() != request_id {
        return Err(RouteInferenceErrorV1::Unavailable);
    }
    if !response.error_code.is_empty() {
        return Err(RouteInferenceErrorV1::Rejected);
    }
    let result =
        AttachmentTranslationInferenceResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    validate_attachment_translation_inference_result_v1(&result)
        .map_err(|_| RouteInferenceErrorV1::Rejected)?;
    Ok(result)
}

enum AttachmentTranslationTerminalResultV1 {
    Ready {
        pending: AttachmentTranslationPendingResultV1,
        bytes: Vec<u8>,
    },
    Rejected(AttachmentTranslationRejectionCodeV1),
}

fn terminal_result(
    run: &PersistedAttachmentTranslationRunV1,
    result: AttachmentTranslationInferenceResultV1,
) -> Result<AttachmentTranslationTerminalResultV1, AttachmentTranslationInferenceErrorV1> {
    let request_digest = array32(&result.request_digest)?;
    let source_sha256 = array32(&result.source_sha256)?;
    if result.run_id.as_slice() != run.draft.run_id
        || Some(request_digest) != run.status.inference_request_digest
        || Some(source_sha256) != run.status.source_sha256
    {
        return Err(AttachmentTranslationInferenceErrorV1::InvalidResult);
    }
    match AiInferenceTerminalStatusV1::try_from(result.terminal_status)
        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidResult)?
    {
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady => {
            let target_language = core_target_language(result.target_language)?;
            if target_language != run.draft.target_language {
                return Err(AttachmentTranslationInferenceErrorV1::InvalidResult);
            }
            let translated_sha256: [u8; 32] = Sha256::digest(&result.translated_text_utf8).into();
            Ok(AttachmentTranslationTerminalResultV1::Ready {
                pending: AttachmentTranslationPendingResultV1 {
                    translated_sha256,
                    translated_size_bytes: u64::try_from(result.translated_text_utf8.len())
                        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidResult)?,
                    detected_source_language: core_detected_language(
                        result.detected_source_language,
                    )?,
                    target_language,
                    completeness: core_completeness(result.completeness)?,
                    confidence_basis_points: result.confidence_basis_points,
                    inference_request_digest: request_digest,
                    source_sha256,
                },
                bytes: result.translated_text_utf8,
            })
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy => {
            Ok(AttachmentTranslationTerminalResultV1::Rejected(
                AttachmentTranslationRejectionCodeV1::Policy,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput => {
            Ok(AttachmentTranslationTerminalResultV1::Rejected(
                AttachmentTranslationRejectionCodeV1::InvalidRequest,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
        | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected => {
            Ok(AttachmentTranslationTerminalResultV1::Rejected(
                AttachmentTranslationRejectionCodeV1::InferenceRejected,
            ))
        }
        AiInferenceTerminalStatusV1::AiInferenceTerminalStatusUnspecified => {
            Err(AttachmentTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

async fn persist_rejection(
    persistence: &AttachmentTranslationPersistenceV1,
    run: &PersistedAttachmentTranslationRunV1,
    rejection: AttachmentTranslationRejectionCodeV1,
    occurred_at_unix_millis: i64,
) -> Result<(), AttachmentTranslationInferenceErrorV1> {
    let message_id = rejection_message_id(run.draft.run_id, rejection);
    persistence
        .persist_inference_result(PersistInferenceResultV1 {
            message_id,
            envelope_sha256: Sha256::digest(message_id).into(),
            logical_owner_id: run.logical_owner_id.clone(),
            run_id: run.draft.run_id,
            transition: AttachmentTranslationTransitionV1::Reject(rejection),
            occurred_at_unix_millis,
        })
        .await
        .map(|_| ())
        .map_err(AttachmentTranslationInferenceErrorV1::Persistence)
}

fn core_target_language(
    value: i32,
) -> Result<AttachmentTranslationLanguageV1, AttachmentTranslationInferenceErrorV1> {
    match AiTranslationLanguageV1::try_from(value)
        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidResult)?
    {
        AiTranslationLanguageV1::AiTranslationLanguageEnglish => {
            Ok(AttachmentTranslationLanguageV1::English)
        }
        AiTranslationLanguageV1::AiTranslationLanguageRussian => {
            Ok(AttachmentTranslationLanguageV1::Russian)
        }
        AiTranslationLanguageV1::AiTranslationLanguageSpanish => {
            Ok(AttachmentTranslationLanguageV1::Spanish)
        }
        AiTranslationLanguageV1::AiTranslationLanguageUnspecified => {
            Err(AttachmentTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_detected_language(
    value: i32,
) -> Result<AttachmentTranslationDetectedLanguageV1, AttachmentTranslationInferenceErrorV1> {
    match AiDetectedLanguageV1::try_from(value)
        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidResult)?
    {
        AiDetectedLanguageV1::AiDetectedLanguageUnknown => {
            Ok(AttachmentTranslationDetectedLanguageV1::Unknown)
        }
        AiDetectedLanguageV1::AiDetectedLanguageEnglish => {
            Ok(AttachmentTranslationDetectedLanguageV1::English)
        }
        AiDetectedLanguageV1::AiDetectedLanguageRussian => {
            Ok(AttachmentTranslationDetectedLanguageV1::Russian)
        }
        AiDetectedLanguageV1::AiDetectedLanguageSpanish => {
            Ok(AttachmentTranslationDetectedLanguageV1::Spanish)
        }
        AiDetectedLanguageV1::AiDetectedLanguageUnspecified => {
            Err(AttachmentTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

fn core_completeness(
    value: i32,
) -> Result<AttachmentTranslationCompletenessV1, AttachmentTranslationInferenceErrorV1> {
    match AiInferenceCompletenessV1::try_from(value)
        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidResult)?
    {
        AiInferenceCompletenessV1::AiInferenceCompletenessComplete => {
            Ok(AttachmentTranslationCompletenessV1::Complete)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessPartial => {
            Ok(AttachmentTranslationCompletenessV1::Partial)
        }
        AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified => {
            Err(AttachmentTranslationInferenceErrorV1::InvalidResult)
        }
    }
}

fn array32(value: &[u8]) -> Result<[u8; 32], AttachmentTranslationInferenceErrorV1> {
    value
        .try_into()
        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidResult)
}

fn inference_message_id(run_id: [u8; 16], request_digest: [u8; 32]) -> [u8; 16] {
    deterministic_id(
        b"makosh.attachment-translation.inference-result.v1\0",
        run_id,
        request_digest,
    )
}

fn materialization_message_id(run_id: [u8; 16], sha256: [u8; 32]) -> [u8; 16] {
    deterministic_id(
        b"makosh.attachment-translation.materialization-result.v1\0",
        run_id,
        sha256,
    )
}

fn rejection_message_id(
    run_id: [u8; 16],
    rejection: AttachmentTranslationRejectionCodeV1,
) -> [u8; 16] {
    deterministic_id(
        b"makosh.attachment-translation.inference-rejection.v1\0",
        run_id,
        [rejection as u8; 32],
    )
}

fn deterministic_id(label: &[u8], run_id: [u8; 16], digest: [u8; 32]) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    hasher.update(run_id);
    hasher.update(digest);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_and_materialization_identities_are_distinct() {
        assert_ne!(
            inference_message_id([1; 16], [2; 32]),
            materialization_message_id([1; 16], [2; 32])
        );
    }
}

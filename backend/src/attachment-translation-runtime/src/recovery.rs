use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    seal_attachment_translation_inference_request_v1,
    wire::{AiPrivateSourceReceiptV1, AttachmentTranslationInferenceRequestV1},
};
use makosh_attachment_translation_core::AttachmentTranslationStateV1;
use makosh_attachment_translation_persistence::{
    AttachmentTranslationPersistenceV1, PersistedAttachmentTranslationRunV1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use prost::Message;

use crate::{
    blob_materialization::{
        materialize_ai_source_from_authority_v1, release_translation_source_blobs_v1,
    },
    inference::{
        AttachmentTranslationInferenceErrorV1, AttachmentTranslationInferenceExecutionV1,
        complete_attachment_translation_inference_v1, validate_request_for_run,
    },
};

#[allow(clippy::too_many_arguments)]
pub async fn recover_attachment_translation_once_v1(
    persistence: &AttachmentTranslationPersistenceV1,
    channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    logical_owner_id: &str,
    runtime_generation: u64,
    grant_epoch: u64,
    occurred_at_unix_millis: i64,
) -> Result<bool, AttachmentTranslationInferenceErrorV1> {
    let runs = persistence
        .load_recoverable_runs(logical_owner_id)
        .await
        .map_err(AttachmentTranslationInferenceErrorV1::Persistence)?;
    let Some(run) = runs.into_iter().find(|run| {
        matches!(
            run.status.state,
            AttachmentTranslationStateV1::AwaitingInference
                | AttachmentTranslationStateV1::MaterializingResult
        )
    }) else {
        return Ok(false);
    };
    let request = decode_persisted_request(&run)?;
    validate_request_for_run(&run, &request)?;
    let authority = run
        .source_authority
        .clone()
        .ok_or(AttachmentTranslationInferenceErrorV1::InvalidRequest)?;
    let ai_source =
        materialize_ai_source_from_authority_v1(channel, dispatcher, run.draft.run_id, &authority)
            .map_err(AttachmentTranslationInferenceErrorV1::Blob)?;
    let request = refresh_runtime_bound_source(&run, request, ai_source.clone())?;
    let accepted = complete_attachment_translation_inference_v1(
        persistence,
        channel,
        dispatcher,
        &run,
        request,
        AttachmentTranslationInferenceExecutionV1 {
            runtime_generation,
            grant_epoch,
            occurred_at_unix_millis,
        },
    )
    .await?;
    release_translation_source_blobs_v1(
        channel,
        dispatcher,
        run.draft.run_id,
        &ai_source,
        &authority,
        accepted,
    )
    .map_err(AttachmentTranslationInferenceErrorV1::Blob)?;
    persistence
        .complete_source_cleanup(
            logical_owner_id,
            &run.draft.run_id,
            &authority,
            occurred_at_unix_millis,
        )
        .await
        .map_err(AttachmentTranslationInferenceErrorV1::Persistence)?;
    Ok(true)
}

fn decode_persisted_request(
    run: &PersistedAttachmentTranslationRunV1,
) -> Result<AttachmentTranslationInferenceRequestV1, AttachmentTranslationInferenceErrorV1> {
    AttachmentTranslationInferenceRequestV1::decode(
        run.inference_request_bytes
            .as_deref()
            .ok_or(AttachmentTranslationInferenceErrorV1::InvalidRequest)?,
    )
    .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidRequest)
}

fn refresh_runtime_bound_source(
    run: &PersistedAttachmentTranslationRunV1,
    mut request: AttachmentTranslationInferenceRequestV1,
    source: AiPrivateSourceReceiptV1,
) -> Result<AttachmentTranslationInferenceRequestV1, AttachmentTranslationInferenceErrorV1> {
    request.source = Some(source);
    let request = seal_attachment_translation_inference_request_v1(request)
        .map_err(|_| AttachmentTranslationInferenceErrorV1::InvalidRequest)?;
    validate_request_for_run(run, &request)?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{
        AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
        AI_LOCAL_EGRESS_POLICY_REVISION_V1,
        wire::{AiContextReceiptV1, AiEgressPolicyV1, AiTranslationLanguageV1, AiUseCaseV1},
    };
    use makosh_attachment_translation_core::{
        AttachmentTranslationDraftV1, AttachmentTranslationLanguageV1,
        AttachmentTranslationStatusV1,
    };
    use makosh_attachment_translation_persistence::AttachmentTranslationSourceAuthorityV1;
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn recovery_rotates_runtime_bound_proof_without_changing_canonical_request() {
        let source_sha256: [u8; 32] = Sha256::digest(b"text").into();
        let original = seal_attachment_translation_inference_request_v1(
            AttachmentTranslationInferenceRequestV1 {
                run_id: vec![1; 16],
                context: Some(AiContextReceiptV1 {
                    context_id: vec![2; 16],
                    use_case: AiUseCaseV1::AiUseCaseAttachmentTranslation as i32,
                    source_evidence_id: vec![3; 16],
                    source_evidence_revision: 7,
                    contract_major: AI_CONTRACT_MAJOR_V1,
                    contract_revision: AI_CONTRACT_REVISION_V1,
                    contract_schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
                    request_digest: Vec::new(),
                }),
                source: Some(AiPrivateSourceReceiptV1 {
                    reference_id: vec![4; 16],
                    declared_bytes: 4,
                    sha256: source_sha256.to_vec(),
                    custody_transfer_source_proof: vec![5; 64],
                }),
                target_language: AiTranslationLanguageV1::AiTranslationLanguageRussian as i32,
                maximum_output_bytes: 4_096,
                maximum_output_tokens: 512,
                egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
                egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
                logical_owner_id: "owner-1".to_owned(),
            },
        )
        .expect("seal original request");
        let original_digest: [u8; 32] = original
            .context
            .as_ref()
            .expect("context")
            .request_digest
            .as_slice()
            .try_into()
            .expect("digest");
        let run = PersistedAttachmentTranslationRunV1 {
            logical_owner_id: "owner-1".to_owned(),
            draft: AttachmentTranslationDraftV1 {
                run_id: [1; 16],
                operation_id: [6; 16],
                source_extraction_run_id: [3; 16],
                expected_source_revision: 7,
                target_language: AttachmentTranslationLanguageV1::Russian,
            },
            request_fingerprint: [7; 32],
            status: AttachmentTranslationStatusV1 {
                state: AttachmentTranslationStateV1::AwaitingInference,
                state_revision: 3,
                source_sha256: Some(source_sha256),
                inference_request_digest: Some(original_digest),
                pending_result: None,
                artifact: None,
                rejection: None,
            },
            inference_request_bytes: Some(original.encode_to_vec()),
            source_authority: Some(AttachmentTranslationSourceAuthorityV1 {
                reference_id: [8; 16],
                declared_bytes: 4,
                sha256: source_sha256,
                custody_proof: vec![9; 64],
            }),
            cleanup_completed_at_unix_millis: None,
            created_at_unix_millis: 10,
            updated_at_unix_millis: 11,
        };
        let refreshed = refresh_runtime_bound_source(
            &run,
            original,
            AiPrivateSourceReceiptV1 {
                reference_id: vec![4; 16],
                declared_bytes: 4,
                sha256: source_sha256.to_vec(),
                custody_transfer_source_proof: vec![10; 64],
            },
        )
        .expect("refresh runtime-bound proof");

        assert_eq!(
            refreshed.context.expect("context").request_digest,
            original_digest
        );
        assert_eq!(
            refreshed
                .source
                .expect("source")
                .custody_transfer_source_proof,
            vec![10; 64]
        );
    }
}

use makosh_ai_contracts::{
    decode_summary_source_content_v1, validate_provider_summary_generation_result_v1,
    validate_summary_inference_request_v1, validate_summary_inference_result_v1,
    wire::{
        AiInferenceCompletenessV1, AiInferenceReceiptV1, AiInferenceTerminalStatusV1,
        AiPrivateSourceReceiptV1, AiProviderSummaryGenerationResultV1, AiSummaryLanguageV1,
        AiSummaryLengthV1, CommunicationSummaryInferenceRequestV1,
        CommunicationSummaryInferenceResultV1,
    },
};
use sha2::{Digest, Sha256};

use crate::{
    AI_INFERENCE_PROVIDER_POLICY_REVISION_V1, AiInferenceCoreErrorV1, AiInferenceRunStateV1,
};

pub const AI_SUMMARY_SOURCE_BODY_EXCERPT_BYTES_V1: usize = 8_000;
const AI_SUMMARY_PROMPT_POLICY_V1: &[u8] =
    b"makosh-ai-communication-summary-v1;sender-subject-body;utf8-prefix-bytes=8000";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiSummaryRunV1 {
    pub request: CommunicationSummaryInferenceRequestV1,
    pub revision: u64,
    pub state: AiInferenceRunStateV1,
    pub terminal_result: Option<CommunicationSummaryInferenceResultV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiSummaryExecutionPlanV1 {
    pub run_id: [u8; 16],
    pub logical_owner_id: String,
    pub request_digest: [u8; 32],
    pub source: AiPrivateSourceReceiptV1,
    pub language: i32,
    pub length: i32,
    pub maximum_output_bytes: u32,
    pub maximum_output_tokens: u32,
    pub egress_policy: i32,
    pub egress_policy_revision: u32,
}

pub fn accept_summary_inference_v1(
    request: CommunicationSummaryInferenceRequestV1,
) -> Result<AiSummaryRunV1, AiInferenceCoreErrorV1> {
    validate_summary_inference_request_v1(&request)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidRequest)?;
    Ok(AiSummaryRunV1 {
        request,
        revision: 1,
        state: AiInferenceRunStateV1::Accepted,
        terminal_result: None,
    })
}

pub fn begin_summary_inference_v1(
    run: &AiSummaryRunV1,
    expected_revision: u64,
) -> Result<(AiSummaryRunV1, AiSummaryExecutionPlanV1), AiInferenceCoreErrorV1> {
    require_revision(run, expected_revision)?;
    if run.state != AiInferenceRunStateV1::Accepted || run.terminal_result.is_some() {
        return Err(AiInferenceCoreErrorV1::InvalidTransition);
    }
    let plan = summary_inference_execution_plan_v1(run)?;
    Ok((
        AiSummaryRunV1 {
            request: run.request.clone(),
            revision: run.revision + 1,
            state: AiInferenceRunStateV1::Executing,
            terminal_result: None,
        },
        plan,
    ))
}

pub fn summary_inference_execution_plan_v1(
    run: &AiSummaryRunV1,
) -> Result<AiSummaryExecutionPlanV1, AiInferenceCoreErrorV1> {
    if !matches!(
        run.state,
        AiInferenceRunStateV1::Accepted | AiInferenceRunStateV1::Executing
    ) || run.terminal_result.is_some()
    {
        return Err(AiInferenceCoreErrorV1::InvalidTransition);
    }
    let context = run
        .request
        .context
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    Ok(AiSummaryExecutionPlanV1 {
        run_id: id16(&run.request.run_id)?,
        logical_owner_id: run.request.logical_owner_id.clone(),
        request_digest: id32(&context.request_digest)?,
        source: run
            .request
            .source
            .clone()
            .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?,
        language: run.request.language,
        length: run.request.length,
        maximum_output_bytes: run.request.maximum_output_bytes,
        maximum_output_tokens: run.request.maximum_output_tokens,
        egress_policy: run.request.egress_policy,
        egress_policy_revision: run.request.egress_policy_revision,
    })
}

pub fn build_summary_provider_input_v1(
    plan: &AiSummaryExecutionPlanV1,
    source_content_bytes: &[u8],
) -> Result<Vec<u8>, AiInferenceCoreErrorV1> {
    if source_content_bytes.is_empty()
        || u64::try_from(source_content_bytes.len()).ok() != Some(plan.source.declared_bytes)
    {
        return Err(AiInferenceCoreErrorV1::InvalidRequest);
    }
    let content = decode_summary_source_content_v1(source_content_bytes)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidRequest)?;
    let sender = std::str::from_utf8(&content.sender_utf8)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidRequest)?;
    let subject = std::str::from_utf8(&content.subject_utf8)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidRequest)?;
    let body = std::str::from_utf8(&content.body_utf8)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidRequest)?;
    let body = utf8_prefix(body, AI_SUMMARY_SOURCE_BODY_EXCERPT_BYTES_V1);
    Ok(format!("Sender: {sender}\nSubject: {subject}\nBody:\n{body}").into_bytes())
}

pub fn complete_summary_inference_v1(
    run: &AiSummaryRunV1,
    expected_revision: u64,
    provider_result: AiProviderSummaryGenerationResultV1,
) -> Result<AiSummaryRunV1, AiInferenceCoreErrorV1> {
    require_revision(run, expected_revision)?;
    if run.state != AiInferenceRunStateV1::Executing
        || run.terminal_result.is_some()
        || provider_result.request_id != run.request.run_id
    {
        return Err(AiInferenceCoreErrorV1::InvalidTransition);
    }
    validate_provider_summary_generation_result_v1(&provider_result)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidProviderResult)?;
    let context = run
        .request
        .context
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let source = run
        .request
        .source
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let status = AiInferenceTerminalStatusV1::try_from(provider_result.terminal_status)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidProviderResult)?;
    let ready = status == AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady;
    let result = CommunicationSummaryInferenceResultV1 {
        run_id: run.request.run_id.clone(),
        request_digest: context.request_digest.clone(),
        source_sha256: source.sha256.clone(),
        summary_utf8: provider_result.summary_utf8,
        resolved_language: provider_result.resolved_language,
        resolved_length: provider_result.resolved_length,
        inference_receipt: ready.then(|| AiInferenceReceiptV1 {
            model_revision_sha256: provider_result.model_revision_sha256,
            prompt_policy_sha256: summary_prompt_policy_sha256_v1().to_vec(),
            provider_settings_revision: provider_result.provider_settings_revision,
            provider_policy_revision: AI_INFERENCE_PROVIDER_POLICY_REVISION_V1,
        }),
        completeness: provider_result.completeness,
        confidence_basis_points: provider_result.confidence_basis_points,
        terminal_status: status as i32,
    };
    validate_summary_inference_result_v1(&result)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidResult)?;
    Ok(AiSummaryRunV1 {
        request: run.request.clone(),
        revision: run.revision + 1,
        state: if ready {
            AiInferenceRunStateV1::Ready
        } else {
            AiInferenceRunStateV1::Rejected
        },
        terminal_result: Some(result),
    })
}

pub fn reject_summary_inference_v1(
    run: &AiSummaryRunV1,
    expected_revision: u64,
    terminal_status: AiInferenceTerminalStatusV1,
) -> Result<AiSummaryRunV1, AiInferenceCoreErrorV1> {
    require_revision(run, expected_revision)?;
    if !matches!(
        run.state,
        AiInferenceRunStateV1::Accepted | AiInferenceRunStateV1::Executing
    ) || run.terminal_result.is_some()
        || !matches!(
            terminal_status,
            AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedInput
                | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusRejectedPolicy
                | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderUnavailable
                | AiInferenceTerminalStatusV1::AiInferenceTerminalStatusProviderRejected
        )
    {
        return Err(AiInferenceCoreErrorV1::InvalidTransition);
    }
    let context = run
        .request
        .context
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let source = run
        .request
        .source
        .as_ref()
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)?;
    let result = CommunicationSummaryInferenceResultV1 {
        run_id: run.request.run_id.clone(),
        request_digest: context.request_digest.clone(),
        source_sha256: source.sha256.clone(),
        summary_utf8: Vec::new(),
        resolved_language: AiSummaryLanguageV1::AiSummaryLanguageUnspecified as i32,
        resolved_length: AiSummaryLengthV1::AiSummaryLengthUnspecified as i32,
        inference_receipt: None,
        completeness: AiInferenceCompletenessV1::AiInferenceCompletenessUnspecified as i32,
        confidence_basis_points: 0,
        terminal_status: terminal_status as i32,
    };
    validate_summary_inference_result_v1(&result)
        .map_err(|_| AiInferenceCoreErrorV1::InvalidResult)?;
    Ok(AiSummaryRunV1 {
        request: run.request.clone(),
        revision: run.revision + 1,
        state: AiInferenceRunStateV1::Rejected,
        terminal_result: Some(result),
    })
}

#[must_use]
pub fn summary_prompt_policy_sha256_v1() -> [u8; 32] {
    Sha256::digest(AI_SUMMARY_PROMPT_POLICY_V1).into()
}

fn require_revision(run: &AiSummaryRunV1, expected: u64) -> Result<(), AiInferenceCoreErrorV1> {
    if expected == 0 || run.revision != expected {
        return Err(AiInferenceCoreErrorV1::RevisionConflict);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], AiInferenceCoreErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 16]| value.iter().any(|byte| *byte != 0))
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)
}

fn id32(value: &[u8]) -> Result<[u8; 32], AiInferenceCoreErrorV1> {
    value
        .try_into()
        .ok()
        .filter(|value: &[u8; 32]| value.iter().any(|byte| *byte != 0))
        .ok_or(AiInferenceCoreErrorV1::InvalidRequest)
}

fn utf8_prefix(value: &str, maximum_bytes: usize) -> &str {
    if value.len() <= maximum_bytes {
        return value;
    }
    let mut end = maximum_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

#[cfg(test)]
mod tests {
    use makosh_ai_contracts::{
        AI_CONTRACT_MAJOR_V1, AI_CONTRACT_REVISION_V1, AI_CONTRACTS_SCHEMA_SHA256,
        AI_LOCAL_EGRESS_POLICY_REVISION_V1, encode_summary_source_content_v1,
        seal_summary_inference_request_v1,
        wire::{
            AiContextReceiptV1, AiEgressPolicyV1, AiPrivateSourceReceiptV1,
            AiProviderSummaryGenerationResultV1, AiSummarySourceContentV1, AiUseCaseV1,
        },
    };

    use super::*;

    fn accepted() -> AiSummaryRunV1 {
        accept_summary_inference_v1(
            seal_summary_inference_request_v1(CommunicationSummaryInferenceRequestV1 {
                run_id: vec![1; 16],
                context: Some(AiContextReceiptV1 {
                    context_id: vec![2; 16],
                    use_case: AiUseCaseV1::AiUseCaseCommunicationSummary as i32,
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
                language: AiSummaryLanguageV1::AiSummaryLanguageAuto as i32,
                length: AiSummaryLengthV1::AiSummaryLengthShort as i32,
                maximum_output_bytes: 4096,
                maximum_output_tokens: 512,
                egress_policy: AiEgressPolicyV1::AiEgressPolicyLocalOnly as i32,
                egress_policy_revision: AI_LOCAL_EGRESS_POLICY_REVISION_V1,
                logical_owner_id: "owner-1".to_owned(),
            })
            .expect("sealed"),
        )
        .expect("accepted")
    }

    #[test]
    fn summary_lifecycle_is_distinct_and_revision_fenced() {
        let accepted = accepted();
        let content = encode_summary_source_content_v1(&AiSummarySourceContentV1 {
            sender_utf8: b"sender@example.test".to_vec(),
            subject_utf8: b"Subject".to_vec(),
            body_utf8: b"Body".to_vec(),
        })
        .expect("content");
        let mut accepted = accepted;
        accepted
            .request
            .source
            .as_mut()
            .expect("source")
            .declared_bytes = content.len() as u64;
        let (executing, plan) = begin_summary_inference_v1(&accepted, 1).expect("executing");
        assert!(
            std::str::from_utf8(&build_summary_provider_input_v1(&plan, &content).expect("input"))
                .is_ok()
        );
        let ready = complete_summary_inference_v1(
            &executing,
            2,
            AiProviderSummaryGenerationResultV1 {
                request_id: vec![1; 16],
                summary_utf8: b"Summary".to_vec(),
                resolved_language: AiSummaryLanguageV1::AiSummaryLanguageEnglish as i32,
                resolved_length: AiSummaryLengthV1::AiSummaryLengthShort as i32,
                model_revision_sha256: vec![9; 32],
                input_tokens: 10,
                output_tokens: 2,
                terminal_status: AiInferenceTerminalStatusV1::AiInferenceTerminalStatusReady as i32,
                completeness: AiInferenceCompletenessV1::AiInferenceCompletenessComplete as i32,
                confidence_basis_points: 8000,
                provider_settings_revision: 3,
            },
        )
        .expect("ready");
        assert_eq!(ready.state, AiInferenceRunStateV1::Ready);
        assert_eq!(
            ready.terminal_result.expect("result").summary_utf8,
            b"Summary"
        );
    }
}

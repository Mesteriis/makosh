#![forbid(unsafe_code)]

mod attachment_translation;
mod explanation;
mod translation;
mod validation;

pub use attachment_translation::{
    AI_ATTACHMENT_TRANSLATION_MAX_SOURCE_BYTES_V1,
    compute_attachment_translation_inference_request_digest_v1,
    seal_attachment_translation_inference_request_v1,
    validate_attachment_translation_inference_request_v1,
    validate_attachment_translation_inference_result_v1,
    validate_attachment_translation_source_text_v1,
};
pub use explanation::{
    compute_explanation_inference_request_digest_v1,
    compute_provider_explanation_request_digest_v1, decode_explanation_inference_result_v1,
    decode_explanation_source_content_v1, decode_provider_explanation_result_v1,
    encode_explanation_inference_result_v1, encode_explanation_source_content_v1,
    encode_provider_explanation_result_v1, seal_explanation_inference_request_v1,
    validate_explanation_inference_request_v1, validate_explanation_inference_result_v1,
    validate_explanation_source_content_v1, validate_provider_explanation_request_v1,
    validate_provider_explanation_result_v1,
};
use makosh_runtime_protocol::v1::ContractReferenceV1;
pub use translation::{
    compute_provider_translation_request_digest_v1,
    compute_translation_inference_request_digest_v1, decode_translation_source_content_v1,
    encode_translation_source_content_v1, seal_translation_inference_request_v1,
    validate_provider_translation_request_v1, validate_provider_translation_result_v1,
    validate_translation_inference_request_v1, validate_translation_inference_result_v1,
    validate_translation_source_content_v1,
};
pub use validation::{
    AiContractValidationErrorV1, compute_provider_reply_generation_request_digest_v1,
    compute_provider_summary_generation_request_digest_v1,
    compute_reply_inference_request_digest_v1, compute_summary_inference_request_digest_v1,
    decode_reply_source_content_v1, decode_summary_source_content_v1,
    encode_reply_source_content_v1, encode_summary_source_content_v1,
    seal_reply_inference_request_v1, seal_summary_inference_request_v1,
    validate_provider_reply_generation_request_v1, validate_provider_reply_generation_result_v1,
    validate_provider_summary_generation_request_v1,
    validate_provider_summary_generation_result_v1, validate_reply_inference_request_v1,
    validate_reply_inference_result_v1, validate_reply_source_content_v1,
    validate_summary_inference_request_v1, validate_summary_inference_result_v1,
    validate_summary_source_content_v1,
};

pub const PACKAGE: &str = "makosh-ai-contracts";
pub const AI_OWNER_V1: &str = "ai";
pub const AI_CONTRACT_MAJOR_V1: u32 = 1;
pub const AI_CONTRACT_REVISION_V1: u32 = 4;
pub const COMMUNICATION_REPLY_INFERENCE_CONTRACT_NAME_V1: &str =
    "communication_reply_suggestion_inference";
pub const AI_PROVIDER_REPLY_GENERATION_CONTRACT_NAME_V1: &str = "ai_provider_reply_generation";
pub const COMMUNICATION_SUMMARY_INFERENCE_CONTRACT_NAME_V1: &str =
    "communication_summary_inference";
pub const AI_PROVIDER_SUMMARY_GENERATION_CONTRACT_NAME_V1: &str = "ai_provider_summary_generation";
pub const COMMUNICATION_TRANSLATION_INFERENCE_CONTRACT_NAME_V1: &str =
    "communication_translation_inference";
pub const ATTACHMENT_TRANSLATION_INFERENCE_CONTRACT_NAME_V1: &str =
    "attachment_translation_inference";
pub const AI_PROVIDER_TRANSLATION_CONTRACT_NAME_V1: &str = "ai_provider_translation";
pub const COMMUNICATION_EXPLANATION_INFERENCE_CONTRACT_NAME_V1: &str =
    "communication_explanation_inference";
pub const AI_PROVIDER_EXPLANATION_CONTRACT_NAME_V1: &str = "ai_provider_explanation";
pub const AI_INFERENCE_REQUEST_CAPABILITY_ID_V1: &str = "ai.inference.request.v1";
pub const AI_PROVIDER_GENERATION_CAPABILITY_ID_V1: &str = "ai.provider.generate.v1";
pub const AI_SUMMARY_REQUEST_CAPABILITY_ID_V1: &str = "ai.summary.request.v1";
pub const AI_PROVIDER_SUMMARY_CAPABILITY_ID_V1: &str = "ai.provider.summarize.v1";
pub const AI_TRANSLATION_REQUEST_CAPABILITY_ID_V1: &str = "ai.translation.request.v1";
pub const AI_ATTACHMENT_TRANSLATION_REQUEST_CAPABILITY_ID_V1: &str =
    "ai.attachment-translation.request.v1";
pub const AI_PROVIDER_TRANSLATION_CAPABILITY_ID_V1: &str = "ai.provider.translate.v1";
pub const AI_EXPLANATION_REQUEST_CAPABILITY_ID_V1: &str = "ai.explanation.request.v1";
pub const AI_PROVIDER_EXPLANATION_CAPABILITY_ID_V1: &str = "ai.provider.explain.v1";
pub const AI_INFERENCE_BLOB_CAPABILITY_ID_V1: &str = "ai.inference.blob.v1";
pub const AI_INFERENCE_MODULE_ID_V1: &str = "makosh-ai-inference-runtime";
pub const AI_MAX_PRIVATE_SOURCE_BYTES_V1: u64 = 256 * 1024;
pub const AI_MAX_OUTPUT_BYTES_V1: u32 = 64 * 1024;
pub const AI_MAX_OUTPUT_TOKENS_V1: u32 = 4_096;
pub const AI_MAX_SENDER_BYTES_V1: usize = 512;
pub const AI_MAX_SUBJECT_BYTES_V1: usize = 998;
pub const AI_MAX_CUSTODY_PROOF_BYTES_V1: usize = 2_048;
pub const AI_LOCAL_EGRESS_POLICY_REVISION_V1: u32 = 1;
pub const AI_MAX_EXPLANATION_REASONS_V1: u32 = 8;
pub const AI_MAX_EXPLANATION_REASON_TEXT_BYTES_V1: u32 = 512;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.ai.contracts.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/ai_contracts_schema.rs"));

pub const AI_CONTRACTS_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ai-contracts-v1.bin"));

#[must_use]
pub fn communication_reply_inference_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_REPLY_INFERENCE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn ai_provider_reply_generation_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(AI_PROVIDER_REPLY_GENERATION_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_summary_inference_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_SUMMARY_INFERENCE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn ai_provider_summary_generation_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(AI_PROVIDER_SUMMARY_GENERATION_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_translation_inference_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_TRANSLATION_INFERENCE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn attachment_translation_inference_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(ATTACHMENT_TRANSLATION_INFERENCE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn ai_provider_translation_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(AI_PROVIDER_TRANSLATION_CONTRACT_NAME_V1)
}

#[must_use]
pub fn communication_explanation_inference_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(COMMUNICATION_EXPLANATION_INFERENCE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn ai_provider_explanation_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(AI_PROVIDER_EXPLANATION_CONTRACT_NAME_V1)
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: AI_OWNER_V1.to_owned(),
        name: name.to_owned(),
        major: AI_CONTRACT_MAJOR_V1,
        revision: AI_CONTRACT_REVISION_V1,
        schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::ContractReferenceV1;

    use super::*;

    #[test]
    fn contracts_are_exact_and_ai_owned() {
        assert_eq!(
            communication_reply_inference_contract_reference_v1(),
            ContractReferenceV1 {
                owner: "ai".to_owned(),
                name: "communication_reply_suggestion_inference".to_owned(),
                major: 1,
                revision: 4,
                schema_sha256: AI_CONTRACTS_SCHEMA_SHA256.to_vec(),
            }
        );
        assert_eq!(
            ai_provider_reply_generation_contract_reference_v1().owner,
            "ai"
        );
        assert_eq!(
            communication_summary_inference_contract_reference_v1().name,
            "communication_summary_inference"
        );
        assert_eq!(
            ai_provider_summary_generation_contract_reference_v1().name,
            "ai_provider_summary_generation"
        );
        assert_eq!(
            communication_translation_inference_contract_reference_v1().name,
            "communication_translation_inference"
        );
        assert_eq!(
            ai_provider_translation_contract_reference_v1().name,
            "ai_provider_translation"
        );
        assert_eq!(
            attachment_translation_inference_contract_reference_v1().name,
            "attachment_translation_inference"
        );
        assert_eq!(
            communication_explanation_inference_contract_reference_v1().name,
            "communication_explanation_inference"
        );
        assert_eq!(
            ai_provider_explanation_contract_reference_v1().name,
            "ai_provider_explanation"
        );
    }
}

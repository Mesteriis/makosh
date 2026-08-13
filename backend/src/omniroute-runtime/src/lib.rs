#![forbid(unsafe_code)]
mod admission;
pub use admission::omniroute_module_descriptor_v1;
use makosh_ai_contracts::{
    AI_PROVIDER_EXPLANATION_CONTRACT_NAME_V1, AI_PROVIDER_REPLY_GENERATION_CONTRACT_NAME_V1,
    AI_PROVIDER_SUMMARY_GENERATION_CONTRACT_NAME_V1, AI_PROVIDER_TRANSLATION_CONTRACT_NAME_V1,
    validate_provider_explanation_request_v1, validate_provider_reply_generation_request_v1,
    validate_provider_summary_generation_request_v1, validate_provider_translation_request_v1,
    wire::{
        AiProviderExplanationRequestV1, AiProviderReplyGenerationRequestV1,
        AiProviderSummaryGenerationRequestV1, AiProviderTranslationRequestV1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};
pub const PACKAGE: &str = "makosh-omniroute-runtime";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmniRouteRuntimeErrorV1 {
    InvalidRequest,
}
pub fn validate_typed_provider_request_v1(
    contract: &str,
    payload: &[u8],
) -> Result<[u8; 32], OmniRouteRuntimeErrorV1> {
    match contract {
        AI_PROVIDER_REPLY_GENERATION_CONTRACT_NAME_V1 => {
            let v = AiProviderReplyGenerationRequestV1::decode(payload)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?;
            validate_provider_reply_generation_request_v1(&v)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?
        }
        AI_PROVIDER_SUMMARY_GENERATION_CONTRACT_NAME_V1 => {
            let v = AiProviderSummaryGenerationRequestV1::decode(payload)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?;
            validate_provider_summary_generation_request_v1(&v)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?
        }
        AI_PROVIDER_TRANSLATION_CONTRACT_NAME_V1 => {
            let v = AiProviderTranslationRequestV1::decode(payload)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?;
            validate_provider_translation_request_v1(&v)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?
        }
        AI_PROVIDER_EXPLANATION_CONTRACT_NAME_V1 => {
            let v = AiProviderExplanationRequestV1::decode(payload)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?;
            validate_provider_explanation_request_v1(&v)
                .map_err(|_| OmniRouteRuntimeErrorV1::InvalidRequest)?
        }
        _ => return Err(OmniRouteRuntimeErrorV1::InvalidRequest),
    }
    Ok(Sha256::digest(payload).into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_generic_or_unknown_provider_request() {
        assert_eq!(
            validate_typed_provider_request_v1("chat", b"{}"),
            Err(OmniRouteRuntimeErrorV1::InvalidRequest)
        );
    }
}

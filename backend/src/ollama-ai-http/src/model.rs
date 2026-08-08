use makosh_ollama_ai_core::{
    OllamaExplanationPlanV1, OllamaGenerationPlanV1, OllamaHttpGenerationV1,
    OllamaSummaryGenerationPlanV1, OllamaTranslationPlanV1,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiHttpErrorV1 {
    InvalidConfiguration,
    InvalidRequest,
    Unavailable,
    Rejected,
    Protocol,
    ModelUnavailable,
    ModelMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaModelRevisionV1 {
    pub model: String,
    pub digest: [u8; 32],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TagsResponseV1 {
    models: Vec<TagModelV1>,
}

#[derive(Deserialize)]
struct TagModelV1 {
    name: String,
    model: String,
    digest: String,
}

#[derive(Serialize)]
struct ChatRequestV1<'a> {
    model: &'a str,
    messages: [ChatMessageV1<'a>; 1],
    stream: bool,
    think: bool,
    format: ReplyJsonSchemaV1,
    options: ChatOptionsV1,
}

#[derive(Serialize)]
struct SummaryChatRequestV1<'a> {
    model: &'a str,
    messages: [ChatMessageV1<'a>; 1],
    stream: bool,
    think: bool,
    format: SummaryJsonSchemaV1,
    options: ChatOptionsV1,
}

#[derive(Serialize)]
struct TranslationChatRequestV1<'a> {
    model: &'a str,
    messages: [ChatMessageV1<'a>; 1],
    stream: bool,
    think: bool,
    format: TranslationJsonSchemaV1,
    options: ChatOptionsV1,
}

#[derive(Serialize)]
struct ExplanationChatRequestV1<'a> {
    model: &'a str,
    messages: [ChatMessageV1<'a>; 1],
    stream: bool,
    think: bool,
    format: ExplanationJsonSchemaV1,
    options: ChatOptionsV1,
}

#[derive(Serialize)]
struct ReplyJsonSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: ReplyJsonPropertiesV1,
    required: [&'static str; 3],
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

#[derive(Serialize)]
struct ReplyJsonPropertiesV1 {
    subject: JsonStringSchemaV1,
    body: JsonStringSchemaV1,
    language: JsonLanguageSchemaV1,
}

#[derive(Serialize)]
struct SummaryJsonSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: SummaryJsonPropertiesV1,
    required: [&'static str; 2],
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

#[derive(Serialize)]
struct SummaryJsonPropertiesV1 {
    summary: JsonStringSchemaV1,
    language: JsonLanguageSchemaV1,
}

#[derive(Serialize)]
struct TranslationJsonSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: TranslationJsonPropertiesV1,
    required: [&'static str; 2],
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

#[derive(Serialize)]
struct TranslationJsonPropertiesV1 {
    translated_text: JsonStringSchemaV1,
    detected_source_language: JsonDetectedLanguageSchemaV1,
}

#[derive(Serialize)]
struct ExplanationJsonSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: ExplanationJsonPropertiesV1,
    required: [&'static str; 3],
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

#[derive(Serialize)]
struct ExplanationJsonPropertiesV1 {
    reasons: ExplanationReasonArraySchemaV1,
    completeness: JsonCompletenessSchemaV1,
    confidence_basis_points: JsonBasisPointsSchemaV1,
}

#[derive(Serialize)]
struct ExplanationReasonArraySchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    items: ExplanationReasonSchemaV1,
    #[serde(rename = "maxItems")]
    max_items: u32,
}

#[derive(Serialize)]
struct ExplanationReasonSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: ExplanationReasonPropertiesV1,
    required: [&'static str; 4],
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

#[derive(Serialize)]
struct ExplanationReasonPropertiesV1 {
    kind: JsonReasonKindSchemaV1,
    explanation: JsonBoundedStringSchemaV1,
    source_basis: JsonSourceBasisSchemaV1,
    confidence_basis_points: JsonBasisPointsSchemaV1,
}

#[derive(Serialize)]
struct JsonReasonKindSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "enum")]
    allowed: [&'static str; 8],
}

#[derive(Serialize)]
struct JsonSourceBasisSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "enum")]
    allowed: [&'static str; 4],
}

#[derive(Serialize)]
struct JsonCompletenessSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "enum")]
    allowed: [&'static str; 2],
}

#[derive(Serialize)]
struct JsonBoundedStringSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "maxLength")]
    max_length: u32,
}

#[derive(Serialize)]
struct JsonBasisPointsSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    minimum: u32,
    maximum: u32,
}

#[derive(Serialize)]
struct JsonStringSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct JsonLanguageSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "enum")]
    allowed: [&'static str; 3],
}

#[derive(Serialize)]
struct JsonDetectedLanguageSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "enum")]
    allowed: [&'static str; 4],
}

#[derive(Serialize)]
struct ChatMessageV1<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatOptionsV1 {
    temperature: u8,
    num_predict: u32,
}

#[derive(Deserialize)]
struct ChatResponseV1 {
    model: String,
    message: ChatResponseMessageV1,
    done: bool,
    prompt_eval_count: u32,
    eval_count: u32,
}

#[derive(Deserialize)]
struct ChatResponseMessageV1 {
    role: String,
    content: String,
    #[serde(default)]
    thinking: String,
}

pub(crate) fn decode_model_revision_v1(
    body: &[u8],
    selected_model: &str,
) -> Result<OllamaModelRevisionV1, OllamaAiHttpErrorV1> {
    let response: TagsResponseV1 =
        serde_json::from_slice(body).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    let mut matches = response
        .models
        .into_iter()
        .filter(|candidate| candidate.name == selected_model || candidate.model == selected_model);
    let model = matches
        .next()
        .ok_or(OllamaAiHttpErrorV1::ModelUnavailable)?;
    if matches.next().is_some() {
        return Err(OllamaAiHttpErrorV1::ModelMismatch);
    }
    Ok(OllamaModelRevisionV1 {
        model: selected_model.to_owned(),
        digest: decode_sha256_hex_v1(&model.digest)?,
    })
}

pub(crate) fn encode_chat_request_v1(
    plan: &OllamaGenerationPlanV1,
) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let prompt =
        std::str::from_utf8(&plan.prompt_utf8).map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)?;
    serde_json::to_vec(&ChatRequestV1 {
        model: &plan.model,
        messages: [ChatMessageV1 {
            role: "user",
            content: prompt,
        }],
        stream: false,
        think: false,
        format: reply_json_schema_v1(),
        options: ChatOptionsV1 {
            temperature: 0,
            num_predict: plan.maximum_output_tokens,
        },
    })
    .map(Zeroizing::new)
    .map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)
}

pub(crate) fn encode_summary_chat_request_v1(
    plan: &OllamaSummaryGenerationPlanV1,
) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let prompt =
        std::str::from_utf8(&plan.prompt_utf8).map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)?;
    serde_json::to_vec(&SummaryChatRequestV1 {
        model: &plan.model,
        messages: [ChatMessageV1 {
            role: "user",
            content: prompt,
        }],
        stream: false,
        think: false,
        format: summary_json_schema_v1(),
        options: ChatOptionsV1 {
            temperature: 0,
            num_predict: plan.maximum_output_tokens,
        },
    })
    .map(Zeroizing::new)
    .map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)
}

pub(crate) fn encode_translation_chat_request_v1(
    plan: &OllamaTranslationPlanV1,
) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let prompt =
        std::str::from_utf8(&plan.prompt_utf8).map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)?;
    serde_json::to_vec(&TranslationChatRequestV1 {
        model: &plan.model,
        messages: [ChatMessageV1 {
            role: "user",
            content: prompt,
        }],
        stream: false,
        think: false,
        format: translation_json_schema_v1(),
        options: ChatOptionsV1 {
            temperature: 0,
            num_predict: plan.maximum_output_tokens,
        },
    })
    .map(Zeroizing::new)
    .map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)
}

pub(crate) fn encode_explanation_chat_request_v1(
    plan: &OllamaExplanationPlanV1,
) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let prompt =
        std::str::from_utf8(&plan.prompt_utf8).map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)?;
    serde_json::to_vec(&ExplanationChatRequestV1 {
        model: &plan.model,
        messages: [ChatMessageV1 {
            role: "user",
            content: prompt,
        }],
        stream: false,
        think: false,
        format: explanation_json_schema_v1(plan),
        options: ChatOptionsV1 {
            temperature: 0,
            num_predict: plan.maximum_output_tokens,
        },
    })
    .map(Zeroizing::new)
    .map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)
}

fn reply_json_schema_v1() -> ReplyJsonSchemaV1 {
    ReplyJsonSchemaV1 {
        kind: "object",
        properties: ReplyJsonPropertiesV1 {
            subject: JsonStringSchemaV1 { kind: "string" },
            body: JsonStringSchemaV1 { kind: "string" },
            language: JsonLanguageSchemaV1 {
                kind: "string",
                allowed: ["english", "spanish", "russian"],
            },
        },
        required: ["subject", "body", "language"],
        additional_properties: false,
    }
}

fn summary_json_schema_v1() -> SummaryJsonSchemaV1 {
    SummaryJsonSchemaV1 {
        kind: "object",
        properties: SummaryJsonPropertiesV1 {
            summary: JsonStringSchemaV1 { kind: "string" },
            language: JsonLanguageSchemaV1 {
                kind: "string",
                allowed: ["english", "spanish", "russian"],
            },
        },
        required: ["summary", "language"],
        additional_properties: false,
    }
}

fn translation_json_schema_v1() -> TranslationJsonSchemaV1 {
    TranslationJsonSchemaV1 {
        kind: "object",
        properties: TranslationJsonPropertiesV1 {
            translated_text: JsonStringSchemaV1 { kind: "string" },
            detected_source_language: JsonDetectedLanguageSchemaV1 {
                kind: "string",
                allowed: ["unknown", "english", "spanish", "russian"],
            },
        },
        required: ["translated_text", "detected_source_language"],
        additional_properties: false,
    }
}

fn explanation_json_schema_v1(plan: &OllamaExplanationPlanV1) -> ExplanationJsonSchemaV1 {
    ExplanationJsonSchemaV1 {
        kind: "object",
        properties: ExplanationJsonPropertiesV1 {
            reasons: ExplanationReasonArraySchemaV1 {
                kind: "array",
                max_items: plan.maximum_reasons,
                items: ExplanationReasonSchemaV1 {
                    kind: "object",
                    properties: ExplanationReasonPropertiesV1 {
                        kind: JsonReasonKindSchemaV1 {
                            kind: "string",
                            allowed: [
                                "urgency",
                                "financial_attention",
                                "legal_or_contractual",
                                "reply_requested",
                                "deadline",
                                "attachment_reference",
                                "marketing_or_bulk",
                                "other_attention",
                            ],
                        },
                        explanation: JsonBoundedStringSchemaV1 {
                            kind: "string",
                            max_length: plan.maximum_reason_text_bytes,
                        },
                        source_basis: JsonSourceBasisSchemaV1 {
                            kind: "string",
                            allowed: ["subject", "body", "canonical_metadata", "combined"],
                        },
                        confidence_basis_points: JsonBasisPointsSchemaV1 {
                            kind: "integer",
                            minimum: 0,
                            maximum: 10_000,
                        },
                    },
                    required: [
                        "kind",
                        "explanation",
                        "source_basis",
                        "confidence_basis_points",
                    ],
                    additional_properties: false,
                },
            },
            completeness: JsonCompletenessSchemaV1 {
                kind: "string",
                allowed: ["complete", "partial"],
            },
            confidence_basis_points: JsonBasisPointsSchemaV1 {
                kind: "integer",
                minimum: 0,
                maximum: 10_000,
            },
        },
        required: ["reasons", "completeness", "confidence_basis_points"],
        additional_properties: false,
    }
}

pub(crate) fn decode_chat_response_v1(
    body: &[u8],
    plan: &OllamaGenerationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    let mut response: ChatResponseV1 =
        serde_json::from_slice(body).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    if response.model != plan.model
        || !response.done
        || response.message.role != "assistant"
        || !response.message.thinking.is_empty()
        || response.message.content.is_empty()
        || response.message.content.len() > plan.maximum_output_bytes as usize
    {
        return Err(OllamaAiHttpErrorV1::ModelMismatch);
    }
    Ok(OllamaHttpGenerationV1 {
        content_json_utf8: Zeroizing::new(
            std::mem::take(&mut response.message.content).into_bytes(),
        ),
        model_digest: plan.model_digest,
        input_tokens: response.prompt_eval_count,
        output_tokens: response.eval_count,
    })
}

pub(crate) fn decode_summary_chat_response_v1(
    body: &[u8],
    plan: &OllamaSummaryGenerationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    let mut response: ChatResponseV1 =
        serde_json::from_slice(body).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    if response.model != plan.model
        || !response.done
        || response.message.role != "assistant"
        || !response.message.thinking.is_empty()
        || response.message.content.is_empty()
        || response.message.content.len() > plan.maximum_output_bytes as usize
    {
        return Err(OllamaAiHttpErrorV1::ModelMismatch);
    }
    Ok(OllamaHttpGenerationV1 {
        content_json_utf8: Zeroizing::new(
            std::mem::take(&mut response.message.content).into_bytes(),
        ),
        model_digest: plan.model_digest,
        input_tokens: response.prompt_eval_count,
        output_tokens: response.eval_count,
    })
}

pub(crate) fn decode_translation_chat_response_v1(
    body: &[u8],
    plan: &OllamaTranslationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    let mut response: ChatResponseV1 =
        serde_json::from_slice(body).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    if response.model != plan.model
        || !response.done
        || response.message.role != "assistant"
        || !response.message.thinking.is_empty()
        || response.message.content.is_empty()
        || response.message.content.len() > plan.maximum_output_bytes as usize
    {
        return Err(OllamaAiHttpErrorV1::ModelMismatch);
    }
    Ok(OllamaHttpGenerationV1 {
        content_json_utf8: Zeroizing::new(
            std::mem::take(&mut response.message.content).into_bytes(),
        ),
        model_digest: plan.model_digest,
        input_tokens: response.prompt_eval_count,
        output_tokens: response.eval_count,
    })
}

pub(crate) fn decode_explanation_chat_response_v1(
    body: &[u8],
    plan: &OllamaExplanationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    let mut response: ChatResponseV1 =
        serde_json::from_slice(body).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    if response.model != plan.model
        || !response.done
        || response.message.role != "assistant"
        || !response.message.thinking.is_empty()
        || response.message.content.is_empty()
        || response.message.content.len() > plan.maximum_response_bytes as usize
    {
        return Err(OllamaAiHttpErrorV1::ModelMismatch);
    }
    Ok(OllamaHttpGenerationV1 {
        content_json_utf8: Zeroizing::new(
            std::mem::take(&mut response.message.content).into_bytes(),
        ),
        model_digest: plan.model_digest,
        input_tokens: response.prompt_eval_count,
        output_tokens: response.eval_count,
    })
}

fn decode_sha256_hex_v1(value: &str) -> Result<[u8; 32], OllamaAiHttpErrorV1> {
    if value.len() != 64 {
        return Err(OllamaAiHttpErrorV1::Protocol);
    }
    let mut digest = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = (hex_nibble_v1(chunk[0])? << 4) | hex_nibble_v1(chunk[1])?;
    }
    (digest != [0; 32])
        .then_some(digest)
        .ok_or(OllamaAiHttpErrorV1::Protocol)
}

fn hex_nibble_v1(value: u8) -> Result<u8, OllamaAiHttpErrorV1> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(OllamaAiHttpErrorV1::Protocol),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_model_discovery_decodes_revision() {
        let body = br#"{"models":[{"name":"gemma3:latest","model":"gemma3:latest","digest":"0909090909090909090909090909090909090909090909090909090909090909"}]}"#;
        assert_eq!(
            decode_model_revision_v1(body, "gemma3:latest"),
            Ok(OllamaModelRevisionV1 {
                model: "gemma3:latest".to_owned(),
                digest: [9; 32],
            })
        );
    }

    #[test]
    fn reply_schema_is_closed_and_language_bounded() {
        assert_eq!(
            serde_json::to_string(&reply_json_schema_v1()).expect("reply JSON schema"),
            r#"{"type":"object","properties":{"subject":{"type":"string"},"body":{"type":"string"},"language":{"type":"string","enum":["english","spanish","russian"]}},"required":["subject","body","language"],"additionalProperties":false}"#
        );
    }

    #[test]
    fn summary_schema_is_closed_and_has_no_extraction_fields() {
        assert_eq!(
            serde_json::to_string(&summary_json_schema_v1()).expect("summary JSON schema"),
            r#"{"type":"object","properties":{"summary":{"type":"string"},"language":{"type":"string","enum":["english","spanish","russian"]}},"required":["summary","language"],"additionalProperties":false}"#
        );
    }

    #[test]
    fn translation_schema_is_closed_and_detected_language_can_be_unknown() {
        assert_eq!(
            serde_json::to_string(&translation_json_schema_v1()).expect("translation JSON schema"),
            r#"{"type":"object","properties":{"translated_text":{"type":"string"},"detected_source_language":{"type":"string","enum":["unknown","english","spanish","russian"]}},"required":["translated_text","detected_source_language"],"additionalProperties":false}"#
        );
    }

    #[test]
    fn explanation_schema_is_closed_and_uses_exact_bounded_taxonomy() {
        let plan = OllamaExplanationPlanV1 {
            request_id: [1; 16],
            request_digest: [2; 32],
            model: "gemma3:latest".to_owned(),
            model_digest: [3; 32],
            prompt_utf8: Zeroizing::new(b"source".to_vec()),
            maximum_output_tokens: 512,
            timeout_millis: 5_000,
            settings_revision: 1,
            maximum_reasons: 8,
            maximum_reason_text_bytes: 512,
            maximum_response_bytes: 5_376,
        };
        assert_eq!(
            serde_json::to_string(&explanation_json_schema_v1(&plan))
                .expect("explanation JSON schema"),
            r#"{"type":"object","properties":{"reasons":{"type":"array","items":{"type":"object","properties":{"kind":{"type":"string","enum":["urgency","financial_attention","legal_or_contractual","reply_requested","deadline","attachment_reference","marketing_or_bulk","other_attention"]},"explanation":{"type":"string","maxLength":512},"source_basis":{"type":"string","enum":["subject","body","canonical_metadata","combined"]},"confidence_basis_points":{"type":"integer","minimum":0,"maximum":10000}},"required":["kind","explanation","source_basis","confidence_basis_points"],"additionalProperties":false},"maxItems":8},"completeness":{"type":"string","enum":["complete","partial"]},"confidence_basis_points":{"type":"integer","minimum":0,"maximum":10000}},"required":["reasons","completeness","confidence_basis_points"],"additionalProperties":false}"#
        );
    }
}

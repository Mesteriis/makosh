//! Pure owner-local lifecycle and normalization rules for derived Communications search.

use makosh_communications_api::{
    CanonicalCommunicationProjectionV1, CanonicalMessageMutationV1,
    CommunicationBodyBlobReferenceV1, CommunicationConversationIdV1, CommunicationMessageIdV1,
    CommunicationObservationIdV1,
};

pub const COMMUNICATIONS_SEARCH_MAX_QUERY_BYTES_V1: usize = 512;
pub const COMMUNICATIONS_SEARCH_MAX_QUERY_TOKENS_V1: usize = 16;
pub const COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1: usize = 256 * 1024;
pub const COMMUNICATIONS_SEARCH_MAX_DOCUMENT_TOKENS_V1: usize = 2_048;
pub const COMMUNICATIONS_SEARCH_PROJECTION_REVISION_V1: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSearchQueryV1 {
    pub tokens: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSearchDocumentV1 {
    pub tokens: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsSearchIndexJobV1 {
    pub evidence_id: CommunicationObservationIdV1,
    pub message_id: CommunicationMessageIdV1,
    pub conversation_id: CommunicationConversationIdV1,
    pub blob: CommunicationBodyBlobReferenceV1,
    pub observed_at_unix_seconds: i64,
    pub projection_revision: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchIndexDecisionV1 {
    Index(CommunicationsSearchIndexJobV1),
    Remove {
        evidence_id: CommunicationObservationIdV1,
        message_id: CommunicationMessageIdV1,
        projection_revision: u32,
        observed_at_unix_seconds: i64,
    },
    Reject {
        evidence_id: CommunicationObservationIdV1,
        message_id: CommunicationMessageIdV1,
        projection_revision: u32,
        observed_at_unix_seconds: i64,
        reason: CommunicationsSearchIndexRejectionV1,
    },
    Ignore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchIndexRejectionV1 {
    DocumentLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchTokenErrorV1 {
    Empty,
    TooLarge,
    TooManyTokens,
}

pub fn decide_search_index_v1(
    projection: &CanonicalCommunicationProjectionV1,
    projection_revision: u32,
) -> CommunicationsSearchIndexDecisionV1 {
    if projection_revision == 0 {
        return CommunicationsSearchIndexDecisionV1::Ignore;
    }
    let Some(message) = projection.message.as_ref() else {
        return CommunicationsSearchIndexDecisionV1::Ignore;
    };
    if message.mutation == CanonicalMessageMutationV1::Delete {
        return CommunicationsSearchIndexDecisionV1::Remove {
            evidence_id: projection.summary.evidence_id,
            message_id: message.message_id,
            projection_revision,
            observed_at_unix_seconds: projection.summary.observed_at_unix_seconds,
        };
    }
    let Some(blob) = projection.summary.body_blob.as_ref() else {
        return CommunicationsSearchIndexDecisionV1::Remove {
            evidence_id: projection.summary.evidence_id,
            message_id: message.message_id,
            projection_revision,
            observed_at_unix_seconds: projection.summary.observed_at_unix_seconds,
        };
    };
    if blob.declared_bytes
        > u64::try_from(COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1).expect("bounded constant")
    {
        return CommunicationsSearchIndexDecisionV1::Reject {
            evidence_id: projection.summary.evidence_id,
            message_id: message.message_id,
            projection_revision,
            observed_at_unix_seconds: projection.summary.observed_at_unix_seconds,
            reason: CommunicationsSearchIndexRejectionV1::DocumentLimit,
        };
    }
    CommunicationsSearchIndexDecisionV1::Index(CommunicationsSearchIndexJobV1 {
        evidence_id: projection.summary.evidence_id,
        message_id: message.message_id,
        conversation_id: message.conversation_id,
        blob: blob.clone(),
        observed_at_unix_seconds: projection.summary.observed_at_unix_seconds,
        projection_revision,
    })
}

pub fn normalize_search_query_v1(
    query: &str,
) -> Result<CommunicationsSearchQueryV1, CommunicationsSearchTokenErrorV1> {
    normalize_tokens(
        query,
        COMMUNICATIONS_SEARCH_MAX_QUERY_BYTES_V1,
        COMMUNICATIONS_SEARCH_MAX_QUERY_TOKENS_V1,
    )
    .map(|tokens| CommunicationsSearchQueryV1 { tokens })
}

pub fn normalize_search_document_tokens_v1(
    document: &str,
) -> Result<CommunicationsSearchDocumentV1, CommunicationsSearchTokenErrorV1> {
    normalize_tokens(
        document,
        COMMUNICATIONS_SEARCH_MAX_DOCUMENT_BYTES_V1,
        COMMUNICATIONS_SEARCH_MAX_DOCUMENT_TOKENS_V1,
    )
    .map(|tokens| CommunicationsSearchDocumentV1 { tokens })
}

fn normalize_tokens(
    value: &str,
    max_bytes: usize,
    max_tokens: usize,
) -> Result<Vec<String>, CommunicationsSearchTokenErrorV1> {
    if value.is_empty() {
        return Err(CommunicationsSearchTokenErrorV1::Empty);
    }
    if value.len() > max_bytes {
        return Err(CommunicationsSearchTokenErrorV1::TooLarge);
    }
    let mut tokens = Vec::new();
    let mut token = String::new();
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            token.push(character);
            continue;
        }
        push_token(&mut tokens, &mut token, max_tokens)?;
    }
    push_token(&mut tokens, &mut token, max_tokens)?;
    (!tokens.is_empty())
        .then_some(tokens)
        .ok_or(CommunicationsSearchTokenErrorV1::Empty)
}

fn push_token(
    tokens: &mut Vec<String>,
    token: &mut String,
    max_tokens: usize,
) -> Result<(), CommunicationsSearchTokenErrorV1> {
    if token.is_empty() {
        return Ok(());
    }
    if !tokens.iter().any(|existing| existing == token) {
        if tokens.len() == max_tokens {
            return Err(CommunicationsSearchTokenErrorV1::TooManyTokens);
        }
        tokens.push(std::mem::take(token));
    } else {
        token.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_normalization_is_unicode_case_folded_and_deduplicated() {
        let query = normalize_search_query_v1("Привет, ПРИВЕТ! Mail-42").expect("query");
        assert_eq!(query.tokens, ["привет", "mail", "42"]);
    }

    #[test]
    fn document_normalization_rejects_excessive_token_cardinality() {
        let document = (0..=COMMUNICATIONS_SEARCH_MAX_DOCUMENT_TOKENS_V1)
            .map(|index| format!("word{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            normalize_search_document_tokens_v1(&document),
            Err(CommunicationsSearchTokenErrorV1::TooManyTokens),
        );
    }
}

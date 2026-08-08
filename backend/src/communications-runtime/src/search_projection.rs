//! Assembles one opaque persistence write from a domain-approved search index job.

use makosh_communications_domain::{
    CommunicationsSearchIndexJobV1, normalize_search_document_tokens_v1,
};
use makosh_communications_persistence::CommunicationsSearchProjectionWriteV1;

use crate::search_digest::keyed_search_token_digest_v1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchProjectionAssemblyErrorV1 {
    InvalidDocument,
    InvalidKey,
}

pub fn assemble_search_projection_write_v1(
    job: &CommunicationsSearchIndexJobV1,
    document: &str,
    key: &[u8],
    indexed_at_unix_seconds: i64,
) -> Result<CommunicationsSearchProjectionWriteV1, CommunicationsSearchProjectionAssemblyErrorV1> {
    let document = normalize_search_document_tokens_v1(document)
        .map_err(|_| CommunicationsSearchProjectionAssemblyErrorV1::InvalidDocument)?;
    let token_digests = document
        .tokens
        .iter()
        .map(|token| {
            keyed_search_token_digest_v1(key, token)
                .map_err(|_| CommunicationsSearchProjectionAssemblyErrorV1::InvalidKey)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CommunicationsSearchProjectionWriteV1 {
        evidence_id: job.evidence_id,
        message_id: job.message_id,
        conversation_id: job.conversation_id,
        observed_at_unix_seconds: job.observed_at_unix_seconds,
        projection_revision: job.projection_revision,
        indexed_at_unix_seconds,
        token_digests,
    })
}

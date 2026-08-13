#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-search-core";
pub const SEARCH_TOKEN_KEY_BYTES_V1: usize = 32;
pub const SEARCH_TOKEN_DIGEST_BYTES_V1: usize = 32;
pub const SEARCH_QUERY_MAX_BYTES_V1: usize = 512;
pub const SEARCH_QUERY_MAX_TOKENS_V1: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchProjectionDocumentV1 {
    pub logical_owner_id: String,
    pub source_owner: String,
    pub entity_kind: String,
    pub entity_id: [u8; 16],
    pub source_revision: u64,
    pub lifecycle_state: String,
    pub occurred_at_unix_millis: i64,
    pub deleted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchCoreErrorV1 {
    InvalidOwner,
    InvalidKind,
    InvalidEntity,
    InvalidRevision,
    InvalidTime,
    InvalidQuery,
    InvalidKey,
}

pub fn validate_search_projection_document_v1(
    value: &SearchProjectionDocumentV1,
) -> Result<(), SearchCoreErrorV1> {
    validate_atom(&value.logical_owner_id).map_err(|_| SearchCoreErrorV1::InvalidOwner)?;
    validate_atom(&value.source_owner).map_err(|_| SearchCoreErrorV1::InvalidOwner)?;
    validate_atom(&value.entity_kind).map_err(|_| SearchCoreErrorV1::InvalidKind)?;
    if value.entity_id.iter().all(|byte| *byte == 0) {
        return Err(SearchCoreErrorV1::InvalidEntity);
    }
    if value.source_revision == 0 {
        return Err(SearchCoreErrorV1::InvalidRevision);
    }
    if value.occurred_at_unix_millis <= 0 {
        return Err(SearchCoreErrorV1::InvalidTime);
    }
    if !value.deleted {
        validate_atom(&value.lifecycle_state).map_err(|_| SearchCoreErrorV1::InvalidKind)?;
    } else if !value.lifecycle_state.is_empty() {
        return Err(SearchCoreErrorV1::InvalidKind);
    }
    Ok(())
}

pub fn search_query_token_digests_v1(
    owner_derived_key: &[u8; SEARCH_TOKEN_KEY_BYTES_V1],
    query: &str,
) -> Result<Vec<[u8; SEARCH_TOKEN_DIGEST_BYTES_V1]>, SearchCoreErrorV1> {
    if owner_derived_key.iter().all(|byte| *byte == 0) {
        return Err(SearchCoreErrorV1::InvalidKey);
    }
    if query.is_empty() || query.len() > SEARCH_QUERY_MAX_BYTES_V1 {
        return Err(SearchCoreErrorV1::InvalidQuery);
    }
    let mut tokens = query
        .split(|character: char| !character.is_alphanumeric())
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>();
    tokens.sort();
    tokens.dedup();
    if tokens.is_empty() || tokens.len() > SEARCH_QUERY_MAX_TOKENS_V1 {
        return Err(SearchCoreErrorV1::InvalidQuery);
    }
    let mut digests = tokens
        .into_iter()
        .map(|token| {
            let mut hash = Sha256::new();
            for value in [
                b"makosh.search.token.v1".as_slice(),
                owner_derived_key.as_slice(),
                token.as_bytes(),
            ] {
                hash.update((value.len() as u64).to_be_bytes());
                hash.update(value);
            }
            hash.finalize().into()
        })
        .collect::<Vec<_>>();
    digests.sort_unstable();
    Ok(digests)
}

fn validate_atom(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Err(())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> SearchProjectionDocumentV1 {
        SearchProjectionDocumentV1 {
            logical_owner_id: "owner-1".to_owned(),
            source_owner: "tasks".to_owned(),
            entity_kind: "task".to_owned(),
            entity_id: [1; 16],
            source_revision: 3,
            lifecycle_state: "active".to_owned(),
            occurred_at_unix_millis: 1_800_000_000_000,
            deleted: false,
        }
    }

    #[test]
    fn projection_document_is_public_structural_identity_only() {
        assert_eq!(validate_search_projection_document_v1(&document()), Ok(()));
        let mut invalid = document();
        invalid.source_revision = 0;
        assert_eq!(
            validate_search_projection_document_v1(&invalid),
            Err(SearchCoreErrorV1::InvalidRevision)
        );
        let mut tombstone = document();
        tombstone.deleted = true;
        tombstone.lifecycle_state.clear();
        assert_eq!(validate_search_projection_document_v1(&tombstone), Ok(()));
    }

    #[test]
    fn query_tokens_are_bounded_deduplicated_and_keyed() {
        let first = search_query_token_digests_v1(&[7; 32], "Alpha beta ALPHA").unwrap();
        let replay = search_query_token_digests_v1(&[7; 32], "beta alpha").unwrap();
        assert_eq!(first, replay);
        assert_eq!(first.len(), 2);
        assert!(first.windows(2).all(|pair| pair[0] < pair[1]));
        assert_ne!(
            first,
            search_query_token_digests_v1(&[8; 32], "alpha beta").unwrap()
        );
        assert_eq!(
            search_query_token_digests_v1(&[0; 32], "alpha"),
            Err(SearchCoreErrorV1::InvalidKey)
        );
    }
}

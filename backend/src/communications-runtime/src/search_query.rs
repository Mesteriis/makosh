//! Executes an owner-local search query with a transient Vault-derived key.

use makosh_communications_api::CommunicationSearchHitV1;
use makosh_communications_domain::normalize_search_query_v1;
use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use std::os::unix::net::UnixStream;

use crate::{
    canonical_read_cursor::{
        CanonicalReadCursorKindV1, decode_descending_cursor_v1, encode_descending_cursor_v1,
    },
    query::CanonicalQueryPageV1,
    search_access::{CommunicationsSearchAccessErrorV1, CommunicationsSearchAccessV1},
    search_digest::keyed_search_token_digest_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchQueryErrorV1 {
    InvalidQuery,
    InvalidCursor,
    Unavailable,
}

pub async fn search_communications_v1(
    persistence: &CommunicationsDurablePersistence,
    access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    query: &str,
    cursor: &[u8],
    limit: u16,
) -> Result<CanonicalQueryPageV1<CommunicationSearchHitV1>, CommunicationsSearchQueryErrorV1> {
    if !(1..=100).contains(&limit) {
        return Err(CommunicationsSearchQueryErrorV1::InvalidQuery);
    }
    let key = access
        .ensure_index_key(control_channel, dispatcher)
        .map_err(access_error)?;
    let digests = query_token_digests_v1(query, &key)?;
    let scope = digests
        .iter()
        .map(|digest| digest.as_slice())
        .collect::<Vec<_>>();
    let after = decode_descending_cursor_v1(cursor, CanonicalReadCursorKindV1::Search, &scope)
        .map_err(|_| CommunicationsSearchQueryErrorV1::InvalidCursor)?;
    let page = persistence
        .search_by_token_digests(&digests, after, limit)
        .await
        .map_err(|_| CommunicationsSearchQueryErrorV1::Unavailable)?;
    let next_cursor = if page.has_more {
        page.items
            .last()
            .map(|item| {
                encode_descending_cursor_v1(
                    CanonicalReadCursorKindV1::Search,
                    &scope,
                    item.observed_at_unix_seconds,
                    item.message_id.bytes(),
                )
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(CanonicalQueryPageV1 {
        items: page.items,
        next_cursor,
    })
}

fn query_token_digests_v1(
    query: &str,
    key: &[u8],
) -> Result<Vec<[u8; 32]>, CommunicationsSearchQueryErrorV1> {
    let normalized = normalize_search_query_v1(query)
        .map_err(|_| CommunicationsSearchQueryErrorV1::InvalidQuery)?;
    normalized
        .tokens
        .iter()
        .map(|token| {
            keyed_search_token_digest_v1(key, token)
                .map_err(|_| CommunicationsSearchQueryErrorV1::Unavailable)
        })
        .collect()
}

fn access_error(_: CommunicationsSearchAccessErrorV1) -> CommunicationsSearchQueryErrorV1 {
    CommunicationsSearchQueryErrorV1::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_digests_are_normalized_and_do_not_retain_query_text() {
        let digests = query_token_digests_v1("Привет, ПРИВЕТ!", &[7; 32]).expect("digests");
        assert_eq!(digests.len(), 1);
        assert_eq!(
            query_token_digests_v1("", &[7; 32]),
            Err(CommunicationsSearchQueryErrorV1::InvalidQuery),
        );
    }
}

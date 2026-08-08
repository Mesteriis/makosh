//! Generated client port for owner-local Communications saved searches.

use std::os::unix::net::UnixStream;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_communications_api::CommunicationSearchHitV1;
use makosh_communications_domain::validate_saved_search_draft_v1;
use makosh_communications_persistence::{
    CommunicationsDurablePersistence, CommunicationsSavedSearchListAfterV1,
    CommunicationsSavedSearchMutationErrorV1, CommunicationsSavedSearchSummaryV1,
    CommunicationsSavedSearchWriteV1,
};
use makosh_communications_saved_query_api::{
    CommunicationsSavedSearchRequestV1, CommunicationsSavedSearchResponseV1,
    CreateSavedSearchRequestV1, DeleteSavedSearchResponseV1, ExecuteSavedSearchResponseV1,
    ListSavedSearchesResponseV1, SAVED_SEARCH_CONTRACT_MAJOR_V1, SAVED_SEARCH_CONTRACT_NAME_V1,
    SAVED_SEARCH_CONTRACT_REVISION_V1, SavedSearchErrorCodeV1, SavedSearchHitV1,
    SavedSearchMutationResponseV1, SavedSearchSummaryV1, communications_saved_search_request_v1,
    communications_saved_search_response_v1,
};
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;

use crate::admission::{
    COMMUNICATIONS_MODULE_ID, COMMUNICATIONS_OWNER_ID,
    COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION,
};
use crate::canonical_read_cursor::{
    CanonicalReadCursorKindV1, decode_descending_cursor_v1, encode_descending_cursor_v1,
};
use crate::search_access::CommunicationsSearchAccessV1;
use crate::search_digest::keyed_search_token_digest_v1;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSavedSearchClientPortErrorV1 {
    Protocol,
    Unavailable,
}

pub fn encode_module_saved_search_request_v1(
    request_id: u64,
    payload: &[u8],
) -> Result<Vec<u8>, CommunicationsSavedSearchClientPortErrorV1> {
    if request_id == 0 || payload.is_empty() {
        return Err(CommunicationsSavedSearchClientPortErrorV1::Protocol);
    }
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        owner_id: COMMUNICATIONS_OWNER_ID.to_owned(),
        contract: Some(saved_search_contract()),
        request_id,
        request_payload: payload.to_vec(),
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    }
    .encode_to_vec())
}

pub async fn handle_module_saved_search_request_v1(
    persistence: &CommunicationsDurablePersistence,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    bytes: &[u8],
) -> Result<Vec<u8>, CommunicationsSavedSearchClientPortErrorV1> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|_| CommunicationsSavedSearchClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != COMMUNICATIONS_MODULE_ID
        || envelope.owner_id != COMMUNICATIONS_OWNER_ID
        || envelope.contract.as_ref() != Some(&saved_search_contract())
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
    {
        return Err(CommunicationsSavedSearchClientPortErrorV1::Protocol);
    }
    let request = CommunicationsSavedSearchRequestV1::decode(envelope.request_payload.as_slice())
        .map_err(|_| CommunicationsSavedSearchClientPortErrorV1::Protocol)?;
    let response = manage_saved_search_v1(
        persistence,
        search_access,
        control_channel,
        dispatcher,
        request,
    )
    .await;
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id: envelope.request_id,
        response_payload: response.encode_to_vec(),
        error_code: String::new(),
    }
    .encode_to_vec())
}

async fn manage_saved_search_v1(
    persistence: &CommunicationsDurablePersistence,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: CommunicationsSavedSearchRequestV1,
) -> CommunicationsSavedSearchResponseV1 {
    if request.protocol_major != SAVED_SEARCH_CONTRACT_MAJOR_V1 {
        return error_response(SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest);
    }
    let result = match request.operation {
        Some(communications_saved_search_request_v1::Operation::List(request)) => {
            list_saved_searches(persistence, request.limit, &request.cursor).await
        }
        Some(communications_saved_search_request_v1::Operation::Create(request)) => {
            create_saved_search(
                persistence,
                search_access,
                control_channel,
                dispatcher,
                request,
            )
            .await
        }
        Some(communications_saved_search_request_v1::Operation::Replace(request)) => {
            replace_saved_search(
                persistence,
                search_access,
                control_channel,
                dispatcher,
                request,
            )
            .await
        }
        Some(communications_saved_search_request_v1::Operation::Delete(request)) => {
            delete_saved_search(
                persistence,
                &request.saved_search_id,
                request.expected_revision,
            )
            .await
        }
        Some(communications_saved_search_request_v1::Operation::Execute(request)) => {
            execute_saved_search(
                persistence,
                &request.saved_search_id,
                request.limit,
                &request.cursor,
            )
            .await
        }
        None => Err(SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest),
    };
    match result {
        Ok(result) => CommunicationsSavedSearchResponseV1 {
            result: Some(result),
            error: SavedSearchErrorCodeV1::SavedSearchErrorCodeUnspecified as i32,
        },
        Err(error) => error_response(error),
    }
}

async fn list_saved_searches(
    persistence: &CommunicationsDurablePersistence,
    limit: u32,
    cursor: &[u8],
) -> Result<communications_saved_search_response_v1::Result, SavedSearchErrorCodeV1> {
    let limit = page_limit(limit)?;
    let after = decode_descending_cursor_v1(cursor, CanonicalReadCursorKindV1::SavedSearch, &[])
        .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest)?
        .map(|value| CommunicationsSavedSearchListAfterV1 {
            updated_at_unix_seconds: value.observed_at_unix_seconds,
            saved_search_id: value.canonical_id,
        });
    let page = persistence
        .list_saved_searches(after, limit)
        .await
        .map_err(map_persistence_error)?;
    let next_cursor = if page.has_more {
        page.items
            .last()
            .map(|item| {
                encode_descending_cursor_v1(
                    CanonicalReadCursorKindV1::SavedSearch,
                    &[],
                    item.updated_at_unix_seconds,
                    item.saved_search_id,
                )
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(communications_saved_search_response_v1::Result::List(
        ListSavedSearchesResponseV1 {
            items: page.items.iter().map(summary_to_wire).collect(),
            next_cursor,
        },
    ))
}

async fn create_saved_search(
    persistence: &CommunicationsDurablePersistence,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: CreateSavedSearchRequestV1,
) -> Result<communications_saved_search_response_v1::Result, SavedSearchErrorCodeV1> {
    let write = prepare_write(
        search_access,
        control_channel,
        dispatcher,
        SavedSearchDraftInputV1 {
            saved_search_id: &request.saved_search_id,
            name: &request.name,
            description: request.description.as_deref(),
            account_id: request.account_id.as_deref(),
            query: &request.query,
        },
    )?;
    let item = persistence
        .create_saved_search(&write)
        .await
        .map_err(map_persistence_error)?;
    Ok(mutation_result(&item))
}

async fn replace_saved_search(
    persistence: &CommunicationsDurablePersistence,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: makosh_communications_saved_query_api::ReplaceSavedSearchRequestV1,
) -> Result<communications_saved_search_response_v1::Result, SavedSearchErrorCodeV1> {
    let write = prepare_write(
        search_access,
        control_channel,
        dispatcher,
        SavedSearchDraftInputV1 {
            saved_search_id: &request.saved_search_id,
            name: &request.name,
            description: request.description.as_deref(),
            account_id: request.account_id.as_deref(),
            query: &request.query,
        },
    )?;
    let item = persistence
        .replace_saved_search(request.expected_revision, &write)
        .await
        .map_err(map_persistence_error)?;
    Ok(mutation_result(&item))
}

async fn delete_saved_search(
    persistence: &CommunicationsDurablePersistence,
    saved_search_id: &[u8],
    expected_revision: u64,
) -> Result<communications_saved_search_response_v1::Result, SavedSearchErrorCodeV1> {
    let saved_search_id = id16(saved_search_id)?;
    let revision = persistence
        .delete_saved_search(saved_search_id, expected_revision, current_unix_seconds()?)
        .await
        .map_err(map_persistence_error)?;
    Ok(communications_saved_search_response_v1::Result::Delete(
        DeleteSavedSearchResponseV1 {
            saved_search_id: saved_search_id.to_vec(),
            revision,
        },
    ))
}

async fn execute_saved_search(
    persistence: &CommunicationsDurablePersistence,
    saved_search_id: &[u8],
    limit: u32,
    cursor: &[u8],
) -> Result<communications_saved_search_response_v1::Result, SavedSearchErrorCodeV1> {
    let saved_search_id = id16(saved_search_id)?;
    let limit = page_limit(limit)?;
    let definition = persistence
        .saved_search_definition(saved_search_id)
        .await
        .map_err(map_persistence_error)?;
    if definition.key_schema_revision != COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION {
        return Err(SavedSearchErrorCodeV1::SavedSearchErrorCodeKeyRevisionStale);
    }
    let revision_bytes = definition.summary.revision.to_be_bytes();
    let mut scope = vec![saved_search_id.as_slice(), revision_bytes.as_slice()];
    if let Some(account_id) = definition.summary.account_id.as_ref() {
        scope.push(account_id.as_slice());
    }
    scope.extend(
        definition
            .token_digests
            .iter()
            .map(|digest| digest.as_slice()),
    );
    let after = decode_descending_cursor_v1(
        cursor,
        CanonicalReadCursorKindV1::SavedSearchExecution,
        &scope,
    )
    .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest)?;
    let page = persistence
        .search_by_token_digests_scoped(
            &definition.token_digests,
            definition.summary.account_id,
            after,
            limit,
        )
        .await
        .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeUnavailable)?;
    let next_cursor = if page.has_more {
        page.items
            .last()
            .map(|item| {
                encode_descending_cursor_v1(
                    CanonicalReadCursorKindV1::SavedSearchExecution,
                    &scope,
                    item.observed_at_unix_seconds,
                    item.message_id.bytes(),
                )
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Ok(communications_saved_search_response_v1::Result::Execute(
        ExecuteSavedSearchResponseV1 {
            hits: page.items.iter().map(hit_to_wire).collect(),
            next_cursor,
            definition_revision: definition.summary.revision,
        },
    ))
}

struct SavedSearchDraftInputV1<'a> {
    saved_search_id: &'a [u8],
    name: &'a str,
    description: Option<&'a str>,
    account_id: Option<&'a [u8]>,
    query: &'a str,
}

fn prepare_write(
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    input: SavedSearchDraftInputV1<'_>,
) -> Result<CommunicationsSavedSearchWriteV1, SavedSearchErrorCodeV1> {
    let draft = validate_saved_search_draft_v1(
        input.saved_search_id,
        input.name,
        input.description,
        input.account_id,
        input.query,
    )
    .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest)?;
    let key = search_access
        .ensure_index_key(control_channel, dispatcher)
        .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeUnavailable)?;
    let token_digests = draft
        .normalized_tokens
        .iter()
        .map(|token| {
            keyed_search_token_digest_v1(&key, token)
                .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeUnavailable)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CommunicationsSavedSearchWriteV1 {
        saved_search_id: draft.saved_search_id,
        name: draft.name,
        description: draft.description,
        account_id: draft.account_id,
        token_digests,
        key_schema_revision: COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION,
        changed_at_unix_seconds: current_unix_seconds()?,
    })
}

fn mutation_result(
    item: &CommunicationsSavedSearchSummaryV1,
) -> communications_saved_search_response_v1::Result {
    communications_saved_search_response_v1::Result::Mutation(SavedSearchMutationResponseV1 {
        item: Some(summary_to_wire(item)),
    })
}

fn summary_to_wire(item: &CommunicationsSavedSearchSummaryV1) -> SavedSearchSummaryV1 {
    SavedSearchSummaryV1 {
        saved_search_id: item.saved_search_id.to_vec(),
        name: item.name.clone(),
        description: item.description.clone(),
        account_id: item.account_id.map(|value| value.to_vec()),
        token_count: u32::from(item.token_count),
        revision: item.revision,
        created_at_unix_seconds: item.created_at_unix_seconds,
        updated_at_unix_seconds: item.updated_at_unix_seconds,
    }
}

fn hit_to_wire(hit: &CommunicationSearchHitV1) -> SavedSearchHitV1 {
    SavedSearchHitV1 {
        evidence_id: hit.evidence_id.bytes().to_vec(),
        message_id: hit.message_id.bytes().to_vec(),
        conversation_id: hit.conversation_id.bytes().to_vec(),
        observed_at_unix_seconds: hit.observed_at_unix_seconds,
        matched_token_count: u32::from(hit.matched_token_count),
    }
}

fn error_response(error: SavedSearchErrorCodeV1) -> CommunicationsSavedSearchResponseV1 {
    CommunicationsSavedSearchResponseV1 {
        result: None,
        error: error as i32,
    }
}

const fn page_limit(value: u32) -> Result<u16, SavedSearchErrorCodeV1> {
    if value == 0 || value > 100 {
        Err(SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest)
    } else {
        Ok(value as u16)
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], SavedSearchErrorCodeV1> {
    value
        .try_into()
        .ok()
        .filter(|candidate: &[u8; 16]| candidate.iter().any(|byte| *byte != 0))
        .ok_or(SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest)
}

fn current_unix_seconds() -> Result<i64, SavedSearchErrorCodeV1> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeUnavailable)?
            .as_secs(),
    )
    .map_err(|_| SavedSearchErrorCodeV1::SavedSearchErrorCodeUnavailable)
}

const fn map_persistence_error(
    error: CommunicationsSavedSearchMutationErrorV1,
) -> SavedSearchErrorCodeV1 {
    match error {
        CommunicationsSavedSearchMutationErrorV1::Invalid => {
            SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest
        }
        CommunicationsSavedSearchMutationErrorV1::AccountNotFound
        | CommunicationsSavedSearchMutationErrorV1::NotFound => {
            SavedSearchErrorCodeV1::SavedSearchErrorCodeNotFound
        }
        CommunicationsSavedSearchMutationErrorV1::RevisionConflict => {
            SavedSearchErrorCodeV1::SavedSearchErrorCodeRevisionConflict
        }
        CommunicationsSavedSearchMutationErrorV1::StorageUnavailable => {
            SavedSearchErrorCodeV1::SavedSearchErrorCodeUnavailable
        }
    }
}

fn saved_search_contract() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: SAVED_SEARCH_CONTRACT_NAME_V1.to_owned(),
        major: SAVED_SEARCH_CONTRACT_MAJOR_V1,
        revision: SAVED_SEARCH_CONTRACT_REVISION_V1,
        schema_sha256:
            makosh_communications_saved_query_api::COMMUNICATIONS_SAVED_SEARCH_SCHEMA_SHA256
                .to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_communications_persistence::CommunicationsSavedSearchDefinitionV1;

    use super::*;

    #[test]
    fn port_rejects_zero_ids_and_unbounded_pages_before_storage() {
        assert_eq!(
            id16(&[0; 16]),
            Err(SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest)
        );
        assert_eq!(
            page_limit(101),
            Err(SavedSearchErrorCodeV1::SavedSearchErrorCodeInvalidRequest)
        );
    }

    #[test]
    fn wire_summary_never_contains_query_or_token_digest() {
        let definition = CommunicationsSavedSearchDefinitionV1 {
            summary: CommunicationsSavedSearchSummaryV1 {
                saved_search_id: [1; 16],
                name: "review".to_owned(),
                description: None,
                account_id: None,
                token_count: 1,
                revision: 1,
                created_at_unix_seconds: 1,
                updated_at_unix_seconds: 1,
            },
            token_digests: vec![[2; 32]],
            key_schema_revision: 1,
        };
        let bytes = summary_to_wire(&definition.summary).encode_to_vec();

        assert!(!bytes.windows(32).any(|window| window == [2; 32]));
    }
}

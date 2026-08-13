use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use makosh_search_api::{
    SEARCH_MODULE_ID_V1, SEARCH_OWNER_ID_V1, search_query_contract_reference_v1,
    search_status_contract_reference_v1,
    wire::{
        SearchCursorV1, SearchHitV1, SearchProjectionStatusRequestV1, SearchProjectionStatusV1,
        SearchQueryResultV1, SearchQueryV1,
    },
};
use makosh_search_core::search_query_token_digests_v1;
use makosh_search_persistence::{
    SearchCursorRecordV1, SearchPersistenceErrorV1, SearchPersistenceV1,
};
use prost::Message;

pub async fn dispatch_search_client_request_v1(
    persistence: &SearchPersistenceV1,
    logical_owner_id: &str,
    owner_derived_key: &[u8; 32],
    request: ModuleClientRequestV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == SEARCH_MODULE_ID_V1
        && request.owner_id == SEARCH_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty();
    let result = if accepted {
        dispatch(persistence, logical_owner_id, owner_derived_key, &request).await
    } else {
        Err("REJECTED")
    };
    match result {
        Ok(response_payload) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload,
            error_code: String::new(),
        },
        Err(error_code) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: error_code.to_owned(),
        },
    }
}

async fn dispatch(
    persistence: &SearchPersistenceV1,
    owner: &str,
    key: &[u8; 32],
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &search_query_contract_reference_v1() {
        let mut query = exact_decode::<SearchQueryV1>(&request.request_payload)?;
        accept_owner(&mut query.logical_owner_id, owner)?;
        let token_digests =
            search_query_token_digests_v1(key, &query.query).map_err(|_| "INVALID")?;
        if !(1..=100).contains(&query.limit) {
            return Err("INVALID");
        }
        let after = if query.after_cursor.is_empty() {
            None
        } else {
            let cursor = exact_decode::<SearchCursorV1>(&query.after_cursor)?;
            Some(SearchCursorRecordV1 {
                source_owner: cursor.source_owner,
                entity_kind: cursor.entity_kind,
                entity_id: id16(&cursor.entity_id)?,
            })
        };
        let requested_limit = query.limit;
        let fetch_limit = requested_limit.checked_add(1).ok_or("INVALID")?;
        let mut rows = persistence
            .query_active(owner, &token_digests, after.as_ref(), fetch_limit)
            .await
            .map_err(persistence_error)?;
        let has_more = rows.len() > usize::try_from(requested_limit).map_err(|_| "INVALID")?;
        if has_more {
            rows.pop();
        }
        let next_cursor = if has_more {
            rows.last()
                .map(|row| SearchCursorV1 {
                    source_owner: row.source_owner.clone(),
                    entity_kind: row.entity_kind.clone(),
                    entity_id: row.entity_id.to_vec(),
                })
                .map_or_else(Vec::new, |value| value.encode_to_vec())
        } else {
            Vec::new()
        };
        let generation = persistence
            .projection_status(owner)
            .await
            .map_err(persistence_error)?
            .active_generation;
        return Ok(SearchQueryResultV1 {
            hits: rows
                .into_iter()
                .map(|row| SearchHitV1 {
                    source_owner: row.source_owner,
                    entity_kind: row.entity_kind,
                    entity_id: row.entity_id.to_vec(),
                    source_revision: row.source_revision,
                    lifecycle_state: row.lifecycle_state,
                    occurred_at_unix_millis: row.occurred_at_unix_millis,
                    matched_token_count: u32::try_from(token_digests.len()).unwrap_or(16),
                })
                .collect(),
            next_cursor,
            projection_generation: generation,
        }
        .encode_to_vec());
    }
    if contract == &search_status_contract_reference_v1() {
        let mut status = exact_decode::<SearchProjectionStatusRequestV1>(&request.request_payload)?;
        accept_owner(&mut status.logical_owner_id, owner)?;
        let status = persistence
            .projection_status(owner)
            .await
            .map_err(persistence_error)?;
        return Ok(SearchProjectionStatusV1 {
            active_generation: status.active_generation,
            indexed_entities: status.indexed_entities,
            source_events: status.source_events,
            rebuilt_at_unix_millis: status.rebuilt_at_unix_millis,
        }
        .encode_to_vec());
    }
    Err("REJECTED")
}

fn exact_decode<M: Message + Default>(bytes: &[u8]) -> Result<M, &'static str> {
    let value = M::decode(bytes).map_err(|_| "REJECTED")?;
    (value.encode_to_vec() == bytes)
        .then_some(value)
        .ok_or("REJECTED")
}

fn accept_owner(payload: &mut String, owner: &str) -> Result<(), &'static str> {
    if payload.is_empty() {
        *payload = owner.to_owned();
        Ok(())
    } else if payload == owner {
        Ok(())
    } else {
        Err("REJECTED")
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    value
        .try_into()
        .ok()
        .filter(|id: &[u8; 16]| id.iter().any(|byte| *byte != 0))
        .ok_or("INVALID")
}

const fn persistence_error(error: SearchPersistenceErrorV1) -> &'static str {
    match error {
        SearchPersistenceErrorV1::NotFound => "NOT_FOUND",
        SearchPersistenceErrorV1::InvalidInput => "INVALID",
        SearchPersistenceErrorV1::Conflict | SearchPersistenceErrorV1::RevisionConflict => {
            "CONFLICT"
        }
        SearchPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_owner_is_empty_or_exact_and_cursor_is_last_returned() {
        let mut empty = String::new();
        assert_eq!(accept_owner(&mut empty, "owner-1"), Ok(()));
        assert_eq!(empty, "owner-1");
        let mut exact = "owner-1".to_owned();
        assert_eq!(accept_owner(&mut exact, "owner-1"), Ok(()));
        let mut conflict = "owner-2".to_owned();
        assert_eq!(accept_owner(&mut conflict, "owner-1"), Err("REJECTED"));
    }
}

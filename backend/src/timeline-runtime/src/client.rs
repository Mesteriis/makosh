use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use makosh_timeline_api::{
    TIMELINE_MODULE_ID_V1, TIMELINE_OWNER_ID_V1, timeline_list_contract_reference_v1,
    timeline_status_contract_reference_v1,
    wire::{
        ListTimelineRequestV1, ListTimelineResultV1, TimelineCursorV1, TimelineEntryV1,
        TimelineStatusRequestV1, TimelineStatusV1,
    },
};
use makosh_timeline_persistence::{
    TimelineCursorRecordV1, TimelinePersistenceErrorV1, TimelinePersistenceV1,
};
use prost::Message;
pub async fn dispatch_timeline_client_request_v1(
    persistence: &TimelinePersistenceV1,
    owner: &str,
    request: ModuleClientRequestV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == TIMELINE_MODULE_ID_V1
        && request.owner_id == TIMELINE_OWNER_ID_V1
        && request.logical_owner_id == owner
        && !request.authenticated_device_id.is_empty();
    let result = if accepted {
        dispatch(persistence, owner, &request).await
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
        Err(code) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: code.into(),
        },
    }
}
async fn dispatch(
    persistence: &TimelinePersistenceV1,
    owner: &str,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &timeline_list_contract_reference_v1() {
        let mut value = exact_decode::<ListTimelineRequestV1>(&request.request_payload)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        if !(1..=100).contains(&value.limit) {
            return Err("INVALID");
        }
        let after = if value.after_cursor.is_empty() {
            None
        } else {
            let cursor = exact_decode::<TimelineCursorV1>(&value.after_cursor)?;
            Some(TimelineCursorRecordV1 {
                occurred_at_unix_millis: cursor.occurred_at_unix_millis,
                source_owner: cursor.source_owner,
                entity_kind: cursor.entity_kind,
                entity_id: id16(&cursor.entity_id)?,
                source_revision: positive(cursor.source_revision)?,
                event_id: id16(&cursor.event_id)?,
            })
        };
        let mut rows = persistence
            .list_active(owner, after.as_ref(), value.limit + 1)
            .await
            .map_err(error)?;
        let has_more = rows.len() > usize::try_from(value.limit).map_err(|_| "INVALID")?;
        if has_more {
            rows.pop();
        }
        let next_cursor = if has_more {
            rows.last()
                .map(|row| TimelineCursorV1 {
                    occurred_at_unix_millis: row.occurred_at_unix_millis,
                    source_owner: row.source_owner.clone(),
                    entity_kind: row.entity_kind.clone(),
                    entity_id: row.entity_id.to_vec(),
                    source_revision: row.source_revision,
                    event_id: row.event_id.to_vec(),
                })
                .map_or_else(Vec::new, |value| value.encode_to_vec())
        } else {
            Vec::new()
        };
        let generation = persistence
            .status(owner)
            .await
            .map_err(error)?
            .active_generation;
        return Ok(ListTimelineResultV1 {
            entries: rows
                .into_iter()
                .map(|row| TimelineEntryV1 {
                    event_id: row.event_id.to_vec(),
                    source_owner: row.source_owner,
                    entity_kind: row.entity_kind,
                    entity_id: row.entity_id.to_vec(),
                    source_revision: row.source_revision,
                    lifecycle_state: row.lifecycle_state,
                    occurred_at_unix_millis: row.occurred_at_unix_millis,
                    tombstone: row.tombstone,
                })
                .collect(),
            next_cursor,
            projection_generation: generation,
        }
        .encode_to_vec());
    }
    if contract == &timeline_status_contract_reference_v1() {
        let mut value = exact_decode::<TimelineStatusRequestV1>(&request.request_payload)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        let status = persistence.status(owner).await.map_err(error)?;
        return Ok(TimelineStatusV1 {
            active_generation: status.active_generation,
            timeline_entries: status.timeline_entries,
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
fn accept_owner(value: &mut String, owner: &str) -> Result<(), &'static str> {
    if value.is_empty() {
        *value = owner.into();
        Ok(())
    } else if value == owner {
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
fn positive(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID")
}
const fn error(value: TimelinePersistenceErrorV1) -> &'static str {
    match value {
        TimelinePersistenceErrorV1::InvalidInput => "INVALID",
        TimelinePersistenceErrorV1::Conflict | TimelinePersistenceErrorV1::RevisionConflict => {
            "CONFLICT"
        }
        TimelinePersistenceErrorV1::NotFound => "NOT_FOUND",
        TimelinePersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owner_is_empty_or_exact() {
        let mut empty = String::new();
        assert_eq!(accept_owner(&mut empty, "owner-1"), Ok(()));
        let mut conflict = "owner-2".into();
        assert_eq!(accept_owner(&mut conflict, "owner-1"), Err("REJECTED"));
    }
}

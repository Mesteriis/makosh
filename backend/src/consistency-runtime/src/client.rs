use makosh_consistency_api::{
    CONSISTENCY_MODULE_ID_V1, CONSISTENCY_OWNER_ID_V1,
    consistency_contradictions_contract_reference_v1, consistency_status_contract_reference_v1,
    wire::{
        ConsistencyContradictionV1 as WireContradiction, ConsistencyEdgeV1 as WireEdge,
        ConsistencyNodeRefV1, ConsistencyStatusRequestV1, ConsistencyStatusV1,
        ListConsistencyContradictionsRequestV1, ListConsistencyContradictionsResultV1,
    },
};
use makosh_consistency_core::{ConsistencyEdgeV1, ConsistencyNodeV1, contradictions_v1};
use makosh_consistency_persistence::{ConsistencyPersistenceErrorV1, ConsistencyPersistenceV1};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;

pub async fn dispatch_consistency_client_request_v1(
    persistence: &ConsistencyPersistenceV1,
    owner: &str,
    request: ModuleClientRequestV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == CONSISTENCY_MODULE_ID_V1
        && request.owner_id == CONSISTENCY_OWNER_ID_V1
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
    persistence: &ConsistencyPersistenceV1,
    owner: &str,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &consistency_contradictions_contract_reference_v1() {
        let mut value =
            exact_decode::<ListConsistencyContradictionsRequestV1>(&request.request_payload)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        if !(1..=100).contains(&value.limit) {
            return Err("INVALID");
        }
        let after = if value.after_first_claim_id.is_empty() {
            None
        } else {
            Some(id16(&value.after_first_claim_id)?)
        };
        let edges = persistence.load_active_edges(owner).await.map_err(error)?;
        let mut contradictions = contradictions_v1(&edges).map_err(|_| "INVALID")?;
        contradictions.retain(|value| after.is_none_or(|after| value.first_claim.edge_id > after));
        let limit = usize::try_from(value.limit).map_err(|_| "INVALID")?;
        let has_more = contradictions.len() > limit;
        contradictions.truncate(limit);
        let next = if has_more {
            contradictions
                .last()
                .map_or_else(Vec::new, |value| value.first_claim.edge_id.to_vec())
        } else {
            Vec::new()
        };
        let generation = persistence
            .status(owner)
            .await
            .map_err(error)?
            .active_generation;
        return Ok(ListConsistencyContradictionsResultV1 {
            contradictions: contradictions
                .into_iter()
                .map(|value| WireContradiction {
                    first_claim: Some(wire_edge(value.first_claim)),
                    second_claim: Some(wire_edge(value.second_claim)),
                })
                .collect(),
            next_after_first_claim_id: next,
            projection_generation: generation,
        }
        .encode_to_vec());
    }
    if contract == &consistency_status_contract_reference_v1() {
        let mut value = exact_decode::<ConsistencyStatusRequestV1>(&request.request_payload)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        let status = persistence.status(owner).await.map_err(error)?;
        return Ok(ConsistencyStatusV1 {
            active_generation: status.active_generation,
            nodes: status.nodes,
            edges: status.edges,
            source_events: status.source_events,
            rebuilt_at_unix_millis: status.rebuilt_at_unix_millis,
        }
        .encode_to_vec());
    }
    Err("REJECTED")
}

fn wire_node(value: ConsistencyNodeV1) -> ConsistencyNodeRefV1 {
    ConsistencyNodeRefV1 {
        node_owner: value.owner,
        node_kind: value.kind,
        node_id: value.id.to_vec(),
    }
}

fn wire_edge(value: ConsistencyEdgeV1) -> WireEdge {
    WireEdge {
        edge_id: value.edge_id.to_vec(),
        source: Some(wire_node(value.source)),
        target: Some(wire_node(value.target)),
        edge_kind: value.edge_kind,
        source_revision: value.source_revision,
        occurred_at_unix_millis: value.occurred_at_unix_millis,
    }
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

const fn error(value: ConsistencyPersistenceErrorV1) -> &'static str {
    match value {
        ConsistencyPersistenceErrorV1::InvalidInput => "INVALID",
        ConsistencyPersistenceErrorV1::Conflict
        | ConsistencyPersistenceErrorV1::RevisionConflict => "CONFLICT",
        ConsistencyPersistenceErrorV1::NotFound => "NOT_FOUND",
        ConsistencyPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_is_empty_or_exact() {
        let mut value = String::new();
        assert_eq!(accept_owner(&mut value, "owner-1"), Ok(()));
        let mut conflict = "owner-2".into();
        assert_eq!(accept_owner(&mut conflict, "owner-1"), Err("REJECTED"));
    }
}

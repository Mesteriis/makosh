use makosh_graph_api::{
    GRAPH_MODULE_ID_V1, GRAPH_OWNER_ID_V1, graph_neighbors_contract_reference_v1,
    graph_path_contract_reference_v1, graph_status_contract_reference_v1,
    wire::{
        GraphEdgeV1 as WireEdge, GraphNeighborsRequestV1, GraphNeighborsResultV1, GraphNodeRefV1,
        GraphPathRequestV1, GraphPathResultV1, GraphStatusRequestV1, GraphStatusV1,
    },
};
use makosh_graph_core::{GraphEdgeV1, GraphNodeV1, shortest_path_v1};
use makosh_graph_persistence::{GraphPersistenceErrorV1, GraphPersistenceV1};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
pub async fn dispatch_graph_client_request_v1(
    persistence: &GraphPersistenceV1,
    owner: &str,
    request: ModuleClientRequestV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == GRAPH_MODULE_ID_V1
        && request.owner_id == GRAPH_OWNER_ID_V1
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
    persistence: &GraphPersistenceV1,
    owner: &str,
    request: &ModuleClientRequestV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &graph_neighbors_contract_reference_v1() {
        let mut value = exact_decode::<GraphNeighborsRequestV1>(&request.request_payload)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        let node = node(value.node)?;
        if !(1..=100).contains(&value.limit) {
            return Err("INVALID");
        }
        let after = if value.after_edge_id.is_empty() {
            None
        } else {
            Some(id16(&value.after_edge_id)?)
        };
        let all = persistence.load_active_edges(owner).await.map_err(error)?;
        let mut edges = all
            .into_iter()
            .filter(|edge| edge.source == node || edge.target == node)
            .filter(|edge| after.is_none_or(|after| edge.edge_id > after))
            .collect::<Vec<_>>();
        edges.sort_by_key(|edge| edge.edge_id);
        let has_more = edges.len() > usize::try_from(value.limit).map_err(|_| "INVALID")?;
        edges.truncate(usize::try_from(value.limit).map_err(|_| "INVALID")?);
        let next = if has_more {
            edges
                .last()
                .map_or_else(Vec::new, |edge| edge.edge_id.to_vec())
        } else {
            Vec::new()
        };
        let generation = persistence
            .status(owner)
            .await
            .map_err(error)?
            .active_generation;
        return Ok(GraphNeighborsResultV1 {
            edges: edges.into_iter().map(wire_edge).collect(),
            next_after_edge_id: next,
            projection_generation: generation,
        }
        .encode_to_vec());
    }
    if contract == &graph_path_contract_reference_v1() {
        let mut value = exact_decode::<GraphPathRequestV1>(&request.request_payload)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        let source = node(value.source)?;
        let target = node(value.target)?;
        let edges = persistence.load_active_edges(owner).await.map_err(error)?;
        let path =
            shortest_path_v1(&edges, &source, &target, value.max_hops).map_err(|_| "NOT_FOUND")?;
        let generation = persistence
            .status(owner)
            .await
            .map_err(error)?
            .active_generation;
        return Ok(GraphPathResultV1 {
            edges: path.into_iter().map(wire_edge).collect(),
            projection_generation: generation,
        }
        .encode_to_vec());
    }
    if contract == &graph_status_contract_reference_v1() {
        let mut value = exact_decode::<GraphStatusRequestV1>(&request.request_payload)?;
        accept_owner(&mut value.logical_owner_id, owner)?;
        let status = persistence.status(owner).await.map_err(error)?;
        return Ok(GraphStatusV1 {
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
fn node(value: Option<GraphNodeRefV1>) -> Result<GraphNodeV1, &'static str> {
    let value = value.ok_or("INVALID")?;
    let node = GraphNodeV1 {
        owner: value.node_owner,
        kind: value.node_kind,
        id: id16(&value.node_id)?,
    };
    makosh_graph_core::validate_graph_node_v1(&node).map_err(|_| "INVALID")?;
    Ok(node)
}
fn wire_node(value: GraphNodeV1) -> GraphNodeRefV1 {
    GraphNodeRefV1 {
        node_owner: value.owner,
        node_kind: value.kind,
        node_id: value.id.to_vec(),
    }
}
fn wire_edge(value: GraphEdgeV1) -> WireEdge {
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
const fn error(value: GraphPersistenceErrorV1) -> &'static str {
    match value {
        GraphPersistenceErrorV1::InvalidInput => "INVALID",
        GraphPersistenceErrorV1::Conflict | GraphPersistenceErrorV1::RevisionConflict => "CONFLICT",
        GraphPersistenceErrorV1::NotFound => "NOT_FOUND",
        GraphPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
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

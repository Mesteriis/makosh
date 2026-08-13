#![forbid(unsafe_code)]
use std::collections::{BTreeMap, VecDeque};
pub const PACKAGE: &str = "makosh-graph-core";
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GraphNodeV1 {
    pub owner: String,
    pub kind: String,
    pub id: [u8; 16],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphEdgeV1 {
    pub edge_id: [u8; 16],
    pub logical_owner_id: String,
    pub source: GraphNodeV1,
    pub target: GraphNodeV1,
    pub edge_kind: String,
    pub source_revision: u64,
    pub occurred_at_unix_millis: i64,
    pub deleted: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphCoreErrorV1 {
    InvalidOwner,
    InvalidNode,
    InvalidEdge,
    InvalidKind,
    InvalidRevision,
    InvalidTime,
    TooManyEdges,
    NoPath,
}
pub fn validate_graph_node_v1(value: &GraphNodeV1) -> Result<(), GraphCoreErrorV1> {
    atom(&value.owner).map_err(|_| GraphCoreErrorV1::InvalidOwner)?;
    atom(&value.kind).map_err(|_| GraphCoreErrorV1::InvalidKind)?;
    if value.id.iter().all(|byte| *byte == 0) {
        Err(GraphCoreErrorV1::InvalidNode)
    } else {
        Ok(())
    }
}
pub fn validate_graph_edge_v1(value: &GraphEdgeV1) -> Result<(), GraphCoreErrorV1> {
    atom(&value.logical_owner_id).map_err(|_| GraphCoreErrorV1::InvalidOwner)?;
    validate_graph_node_v1(&value.source)?;
    validate_graph_node_v1(&value.target)?;
    if value.edge_id.iter().all(|byte| *byte == 0) || value.source == value.target {
        return Err(GraphCoreErrorV1::InvalidEdge);
    }
    if value.source_revision == 0 {
        return Err(GraphCoreErrorV1::InvalidRevision);
    }
    if value.occurred_at_unix_millis <= 0 {
        return Err(GraphCoreErrorV1::InvalidTime);
    }
    if value.deleted {
        if !value.edge_kind.is_empty() {
            return Err(GraphCoreErrorV1::InvalidKind);
        }
    } else {
        atom(&value.edge_kind).map_err(|_| GraphCoreErrorV1::InvalidKind)?
    }
    Ok(())
}
pub fn shortest_path_v1(
    edges: &[GraphEdgeV1],
    source: &GraphNodeV1,
    target: &GraphNodeV1,
    max_hops: u32,
) -> Result<Vec<GraphEdgeV1>, GraphCoreErrorV1> {
    validate_graph_node_v1(source)?;
    validate_graph_node_v1(target)?;
    if !(1..=8).contains(&max_hops) || edges.len() > 10_000 {
        return Err(GraphCoreErrorV1::TooManyEdges);
    }
    let mut queue = VecDeque::from([(source.clone(), Vec::<GraphEdgeV1>::new())]);
    let mut seen = BTreeMap::from([(source.clone(), 0_u32)]);
    while let Some((node, path)) = queue.pop_front() {
        if &node == target {
            return Ok(path);
        }
        if path.len() >= max_hops as usize {
            continue;
        }
        for edge in edges
            .iter()
            .filter(|edge| !edge.deleted && (edge.source == node || edge.target == node))
        {
            let next = if edge.source == node {
                edge.target.clone()
            } else {
                edge.source.clone()
            };
            let depth =
                u32::try_from(path.len() + 1).map_err(|_| GraphCoreErrorV1::TooManyEdges)?;
            if seen.get(&next).is_some_and(|value| *value <= depth) {
                continue;
            }
            seen.insert(next.clone(), depth);
            let mut next_path = path.clone();
            next_path.push(edge.clone());
            queue.push_back((next, next_path));
        }
    }
    Err(GraphCoreErrorV1::NoPath)
}
fn atom(value: &str) -> Result<(), ()> {
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
    fn node(id: u8) -> GraphNodeV1 {
        GraphNodeV1 {
            owner: "persons".into(),
            kind: "person".into(),
            id: [id; 16],
        }
    }
    fn edge(id: u8, a: u8, b: u8) -> GraphEdgeV1 {
        GraphEdgeV1 {
            edge_id: [id; 16],
            logical_owner_id: "owner-1".into(),
            source: node(a),
            target: node(b),
            edge_kind: "confirmed_relationship".into(),
            source_revision: 1,
            occurred_at_unix_millis: 1000,
            deleted: false,
        }
    }
    #[test]
    fn path_is_bounded_confirmed_edges_only() {
        let path =
            shortest_path_v1(&[edge(10, 1, 2), edge(11, 2, 3)], &node(1), &node(3), 3).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(
            shortest_path_v1(&[edge(10, 1, 2)], &node(1), &node(3), 3),
            Err(GraphCoreErrorV1::NoPath)
        );
    }
}

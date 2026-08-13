#![forbid(unsafe_code)]
pub const PACKAGE: &str = "makosh-consistency-core";
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConsistencyNodeV1 {
    pub owner: String,
    pub kind: String,
    pub id: [u8; 16],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsistencyEdgeV1 {
    pub edge_id: [u8; 16],
    pub logical_owner_id: String,
    pub source: ConsistencyNodeV1,
    pub target: ConsistencyNodeV1,
    pub edge_kind: String,
    pub source_revision: u64,
    pub occurred_at_unix_millis: i64,
    pub deleted: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsistencyCoreErrorV1 {
    InvalidOwner,
    InvalidNode,
    InvalidEdge,
    InvalidKind,
    InvalidRevision,
    InvalidTime,
    TooManyEdges,
}
pub fn validate_consistency_node_v1(
    value: &ConsistencyNodeV1,
) -> Result<(), ConsistencyCoreErrorV1> {
    atom(&value.owner).map_err(|_| ConsistencyCoreErrorV1::InvalidOwner)?;
    atom(&value.kind).map_err(|_| ConsistencyCoreErrorV1::InvalidKind)?;
    if value.id.iter().all(|byte| *byte == 0) {
        Err(ConsistencyCoreErrorV1::InvalidNode)
    } else {
        Ok(())
    }
}
pub fn validate_consistency_edge_v1(
    value: &ConsistencyEdgeV1,
) -> Result<(), ConsistencyCoreErrorV1> {
    atom(&value.logical_owner_id).map_err(|_| ConsistencyCoreErrorV1::InvalidOwner)?;
    validate_consistency_node_v1(&value.source)?;
    validate_consistency_node_v1(&value.target)?;
    if value.edge_id.iter().all(|byte| *byte == 0) || value.source == value.target {
        return Err(ConsistencyCoreErrorV1::InvalidEdge);
    }
    if value.source_revision == 0 {
        return Err(ConsistencyCoreErrorV1::InvalidRevision);
    }
    if value.occurred_at_unix_millis <= 0 {
        return Err(ConsistencyCoreErrorV1::InvalidTime);
    }
    if value.deleted {
        if !value.edge_kind.is_empty() {
            return Err(ConsistencyCoreErrorV1::InvalidKind);
        }
    } else {
        atom(&value.edge_kind).map_err(|_| ConsistencyCoreErrorV1::InvalidKind)?
    }
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsistencyContradictionV1 {
    pub first_claim: ConsistencyEdgeV1,
    pub second_claim: ConsistencyEdgeV1,
}

pub fn contradictions_v1(
    edges: &[ConsistencyEdgeV1],
) -> Result<Vec<ConsistencyContradictionV1>, ConsistencyCoreErrorV1> {
    if edges.len() > 10_000 {
        return Err(ConsistencyCoreErrorV1::TooManyEdges);
    }
    for edge in edges {
        validate_consistency_edge_v1(edge)?;
    }
    let mut active = edges
        .iter()
        .filter(|edge| !edge.deleted)
        .collect::<Vec<_>>();
    active.sort_by_key(|edge| edge.edge_id);
    let mut result = Vec::new();
    for (index, first) in active.iter().enumerate() {
        for second in active.iter().skip(index + 1) {
            if first.source == second.source
                && first.edge_kind == second.edge_kind
                && first.target != second.target
            {
                result.push(ConsistencyContradictionV1 {
                    first_claim: (*first).clone(),
                    second_claim: (*second).clone(),
                });
            }
        }
    }
    Ok(result)
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
    fn node(id: u8) -> ConsistencyNodeV1 {
        ConsistencyNodeV1 {
            owner: "persons".into(),
            kind: "person".into(),
            id: [id; 16],
        }
    }
    fn edge(id: u8, a: u8, b: u8) -> ConsistencyEdgeV1 {
        ConsistencyEdgeV1 {
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
    fn contradiction_is_two_explicit_claims_and_never_a_truth_choice() {
        let first = edge(10, 1, 2);
        let mut second = edge(11, 1, 3);
        second.edge_kind = first.edge_kind.clone();
        let conflicts = contradictions_v1(&[first.clone(), second.clone()]).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].first_claim, first);
        assert_eq!(conflicts[0].second_claim, second);
        assert!(contradictions_v1(&[edge(12, 2, 3)]).unwrap().is_empty());
    }
}

use makosh_relationships_api::RelationshipRead;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct RelationshipListResponse {
    pub(crate) items: Vec<RelationshipRead>,
}

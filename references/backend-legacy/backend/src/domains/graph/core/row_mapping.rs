use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::GraphStoreError;
use super::models::{
    GraphEdge, GraphEvidenceSourceKind, GraphEvidenceSummary, GraphNode, GraphReviewState,
    RelationshipType,
};
use crate::platform::graph::GraphNodeKind;
use makosh_graph_api::GraphCount;

pub(super) fn row_to_node(row: PgRow) -> Result<GraphNode, GraphStoreError> {
    Ok(GraphNode {
        node_id: row.try_get("node_id")?,
        node_kind: parse_node_kind(row.try_get("node_kind")?)?,
        stable_key: row.try_get("stable_key")?,
        label: row.try_get("label")?,
        properties: row.try_get("properties")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_edge(row: PgRow) -> Result<GraphEdge, GraphStoreError> {
    Ok(GraphEdge {
        edge_id: row.try_get("edge_id")?,
        source_node_id: row.try_get("source_node_id")?,
        target_node_id: row.try_get("target_node_id")?,
        relationship_type: parse_relationship_type(row.try_get("relationship_type")?)?,
        confidence: row.try_get("confidence")?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        properties: row.try_get("properties")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_count(row: PgRow) -> Result<GraphCount, GraphStoreError> {
    Ok(GraphCount {
        key: row.try_get("key")?,
        count: row.try_get("count")?,
    })
}

pub(super) fn row_to_evidence_summary(row: PgRow) -> Result<GraphEvidenceSummary, GraphStoreError> {
    Ok(GraphEvidenceSummary {
        edge_id: row.try_get("edge_id")?,
        source_kind: parse_evidence_source_kind(row.try_get("source_kind")?)?,
        source_id: row.try_get("source_id")?,
        observation_id: row.try_get("observation_id")?,
        excerpt: row.try_get("excerpt")?,
        metadata: row.try_get("metadata")?,
    })
}

fn parse_node_kind(value: String) -> Result<GraphNodeKind, GraphStoreError> {
    match value.as_str() {
        "person" | "persona" => Ok(GraphNodeKind::Persona),
        "email_address" => Ok(GraphNodeKind::EmailAddress),
        "message" => Ok(GraphNodeKind::Message),
        "document" => Ok(GraphNodeKind::Document),
        "project" => Ok(GraphNodeKind::Project),
        "organization" => Ok(GraphNodeKind::Organization),
        "task" => Ok(GraphNodeKind::Task),
        "event" => Ok(GraphNodeKind::Event),
        "decision" => Ok(GraphNodeKind::Decision),
        "obligation" => Ok(GraphNodeKind::Obligation),
        "knowledge" => Ok(GraphNodeKind::Knowledge),
        _ => Err(GraphStoreError::UnknownNodeKind(value)),
    }
}

fn parse_relationship_type(value: String) -> Result<RelationshipType, GraphStoreError> {
    match value.as_str() {
        "person_has_email_address" | "persona_has_email_address" => {
            Ok(RelationshipType::PersonaHasEmailAddress)
        }
        "person_sent_message" | "persona_sent_message" => Ok(RelationshipType::PersonaSentMessage),
        "person_received_message" | "persona_received_message" => {
            Ok(RelationshipType::PersonaReceivedMessage)
        }
        "email_address_sent_message" => Ok(RelationshipType::EmailAddressSentMessage),
        "email_address_received_message" => Ok(RelationshipType::EmailAddressReceivedMessage),
        "project_has_message" => Ok(RelationshipType::ProjectHasMessage),
        "project_has_document" => Ok(RelationshipType::ProjectHasDocument),
        "project_involves_person" | "project_involves_persona" => {
            Ok(RelationshipType::ProjectInvolvesPersona)
        }
        "project_involves_email_address" => Ok(RelationshipType::ProjectInvolvesEmailAddress),
        "entity_relationship" => Ok(RelationshipType::EntityRelationship),
        _ => Err(GraphStoreError::UnknownRelationshipType(value)),
    }
}

fn parse_review_state(value: String) -> Result<GraphReviewState, GraphStoreError> {
    match value.as_str() {
        "system_accepted" => Ok(GraphReviewState::SystemAccepted),
        "suggested" => Ok(GraphReviewState::Suggested),
        "user_confirmed" => Ok(GraphReviewState::UserConfirmed),
        "user_rejected" => Ok(GraphReviewState::UserRejected),
        _ => Err(GraphStoreError::UnknownReviewState(value)),
    }
}

fn parse_evidence_source_kind(value: String) -> Result<GraphEvidenceSourceKind, GraphStoreError> {
    match value.as_str() {
        "contact" | "person" | "persona" => Ok(GraphEvidenceSourceKind::Persona),
        "message" => Ok(GraphEvidenceSourceKind::Message),
        "document" => Ok(GraphEvidenceSourceKind::Document),
        "relationship" => Ok(GraphEvidenceSourceKind::Relationship),
        "decision" => Ok(GraphEvidenceSourceKind::Decision),
        "obligation" => Ok(GraphEvidenceSourceKind::Obligation),
        "observation" => Ok(GraphEvidenceSourceKind::Observation),
        _ => Err(GraphStoreError::UnknownEvidenceSourceKind(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GraphEvidenceSourceKind, GraphNodeKind, RelationshipType, parse_evidence_source_kind,
        parse_node_kind, parse_relationship_type,
    };

    #[test]
    fn persona_graph_kinds_read_person_node_storage_and_persona_evidence_source() {
        assert_eq!(
            parse_node_kind("persona".to_owned()).expect("persona node kind"),
            GraphNodeKind::Persona
        );
        assert_eq!(
            parse_node_kind("person".to_owned()).expect("person node kind"),
            GraphNodeKind::Persona
        );
        assert_eq!(
            parse_evidence_source_kind("persona".to_owned()).expect("persona evidence kind"),
            GraphEvidenceSourceKind::Persona
        );
        assert_eq!(
            parse_evidence_source_kind("person".to_owned()).expect("person evidence kind"),
            GraphEvidenceSourceKind::Persona
        );
        assert_eq!(
            parse_evidence_source_kind("contact".to_owned()).expect("legacy contact evidence kind"),
            GraphEvidenceSourceKind::Persona
        );
        assert_eq!(
            parse_relationship_type("persona_has_email_address".to_owned())
                .expect("persona email relationship"),
            RelationshipType::PersonaHasEmailAddress
        );
        assert_eq!(
            parse_relationship_type("person_has_email_address".to_owned())
                .expect("legacy persona email relationship"),
            RelationshipType::PersonaHasEmailAddress
        );
        assert_eq!(
            parse_relationship_type("persona_sent_message".to_owned())
                .expect("persona sent relationship"),
            RelationshipType::PersonaSentMessage
        );
        assert_eq!(
            parse_relationship_type("person_sent_message".to_owned())
                .expect("legacy persona sent relationship"),
            RelationshipType::PersonaSentMessage
        );
        assert_eq!(
            parse_relationship_type("project_involves_persona".to_owned())
                .expect("project persona relationship"),
            RelationshipType::ProjectInvolvesPersona
        );
        assert_eq!(
            parse_relationship_type("project_involves_person".to_owned())
                .expect("legacy project persona relationship"),
            RelationshipType::ProjectInvolvesPersona
        );
    }
}

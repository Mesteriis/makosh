use std::collections::BTreeSet;

use serde_json::json;
use sqlx::{Postgres, Transaction};

use crate::domains::graph::core::models::{NewGraphEdge, NewGraphNode, RelationshipType};
use crate::domains::graph::ports::GraphProjectionPort;
use crate::platform::graph::GraphNodeKind;
use crate::platform::graph::node_id;
use makosh_projects_api::{ProjectMatchedDocument, ProjectMatchedMessage, ProjectProjectionSource};

use super::errors::GraphProjectionError;
use super::evidence::{project_document_evidence, project_message_evidence};
use super::helpers::{
    normalize_email_address, project_review_confidence, project_review_graph_state,
};
use super::models::GraphProjectionReport;
use super::service::GraphProjectionService;

impl GraphProjectionService {
    pub(super) async fn project_project(
        &self,
        project: &ProjectProjectionSource,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let messages = self
            .projects
            .matching_project_messages(&project.project.project_id)
            .await?;
        let documents = self
            .projects
            .matching_project_documents(&project.project.project_id)
            .await?;

        let mut transaction = self.pool.begin().await?;
        let project_node = GraphProjectionPort::upsert_node_in_transaction(
            &mut transaction,
            &NewGraphNode::new(
                GraphNodeKind::Project,
                &project.project.project_id,
                &project.project.name,
            )
            .properties(json!({
                "kind": project.project.kind,
                "status": project.project.status,
                "description": project.project.description,
                "owner_display_name": project.project.owner_display_name,
                "progress_percent": project.project.progress_percent,
                "start_date": project.project.start_date,
                "target_date": project.project.target_date,
                "keywords": project.keywords,
            })),
        )
        .await?;
        report.nodes_upserted += 1;

        self.delete_project_edges(&mut transaction, &project_node.node_id)
            .await?;

        for message in &messages {
            self.project_project_message(&mut transaction, &project_node.node_id, message, report)
                .await?;
            self.project_project_people(&mut transaction, &project_node.node_id, message, report)
                .await?;
        }

        for document in &documents {
            self.project_project_document(
                &mut transaction,
                &project_node.node_id,
                document,
                report,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn delete_project_edges(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
    ) -> Result<(), GraphProjectionError> {
        sqlx::query(
            r#"
            DELETE FROM graph_edges
            WHERE source_node_id = $1
              AND relationship_type IN (
                  'project_has_message',
                  'project_has_document',
                  'project_involves_persona',
                  'project_involves_email_address'
              )
            "#,
        )
        .bind(project_node_id)
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    async fn project_project_message(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        message: &ProjectMatchedMessage,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                project_node_id.to_owned(),
                node_id(GraphNodeKind::Message, &message.message_id),
                RelationshipType::ProjectHasMessage,
                project_review_confidence(message.review_state),
                project_review_graph_state(message.review_state),
            )
            .properties(json!({ "match_rule": "project_keyword" })),
            &[project_message_evidence(message)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn project_project_document(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        document: &ProjectMatchedDocument,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
            transaction,
            &NewGraphEdge::new(
                project_node_id.to_owned(),
                node_id(GraphNodeKind::Document, &document.document_id),
                RelationshipType::ProjectHasDocument,
                project_review_confidence(document.review_state),
                project_review_graph_state(document.review_state),
            )
            .properties(json!({ "match_rule": "project_keyword" })),
            &[project_document_evidence(document)],
        )
        .await?;
        report.edges_upserted += 1;
        report.evidence_upserted += 1;

        Ok(())
    }

    async fn project_project_people(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        project_node_id: &str,
        message: &ProjectMatchedMessage,
        report: &mut GraphProjectionReport,
    ) -> Result<(), GraphProjectionError> {
        let mut participant_emails = BTreeSet::new();
        participant_emails.insert(normalize_email_address(&message.sender));
        for recipient in &message.recipients {
            participant_emails.insert(normalize_email_address(recipient));
        }

        for participant_email in participant_emails {
            let endpoint = self
                .resolve_message_endpoint(transaction, &participant_email, report)
                .await?;
            GraphProjectionPort::upsert_edge_with_evidence_in_transaction(
                transaction,
                &NewGraphEdge::new(
                    project_node_id.to_owned(),
                    endpoint.node_id().to_owned(),
                    endpoint.project_relationship_type(),
                    project_review_confidence(message.review_state),
                    project_review_graph_state(message.review_state),
                )
                .properties(json!({ "match_rule": "project_keyword" })),
                &[project_message_evidence(message)],
            )
            .await?;
            report.edges_upserted += 1;
            report.evidence_upserted += 1;
        }

        Ok(())
    }
}

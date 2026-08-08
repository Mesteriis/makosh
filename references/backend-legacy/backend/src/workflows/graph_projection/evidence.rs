use serde_json::json;

use crate::domains::graph::core::models::{GraphEvidenceSourceKind, NewGraphEvidence};
use makosh_projects_api::{ProjectMatchedDocument, ProjectMatchedMessage};

use super::models::MessageRow;

pub(super) fn message_evidence(message: &MessageRow) -> NewGraphEvidence {
    NewGraphEvidence::new(GraphEvidenceSourceKind::Message, message.message_id.clone())
        .observation_id(message.observation_id.clone())
        .excerpt(message.subject.clone())
        .metadata(json!({
            "raw_record_id": message.raw_record_id,
            "observation_id": message.observation_id,
            "provider_record_id": message.provider_record_id,
        }))
}

pub(super) fn project_message_evidence(message: &ProjectMatchedMessage) -> NewGraphEvidence {
    NewGraphEvidence::new(GraphEvidenceSourceKind::Message, message.message_id.clone())
        .observation_id(message.observation_id.clone())
        .excerpt(message.subject.clone())
        .metadata(json!({
            "raw_record_id": message.raw_record_id,
            "observation_id": message.observation_id,
            "account_id": message.account_id,
            "provider_record_id": message.provider_record_id,
            "occurred_at": message.occurred_at,
            "projected_at": message.projected_at,
            "match_rule": "project_keyword",
        }))
}

pub(super) fn project_document_evidence(document: &ProjectMatchedDocument) -> NewGraphEvidence {
    NewGraphEvidence::new(
        GraphEvidenceSourceKind::Document,
        document.document_id.clone(),
    )
    .excerpt(document.title.clone())
    .metadata(json!({
        "document_kind": document.document_kind,
        "source_fingerprint": document.source_fingerprint,
        "imported_at": document.imported_at,
        "match_rule": "project_keyword",
    }))
}

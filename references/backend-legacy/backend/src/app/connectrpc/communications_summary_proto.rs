use crate::workflows::email_intelligence::models::{EmailKnowledgeCandidate, EmailSummaryContract};
use makosh_connectrpc_contracts::makosh::communications::v1::{
    MessageKnowledgeCandidate as ProtoMessageKnowledgeCandidate,
    MessageSummaryContract as ProtoMessageSummaryContract,
};

pub(super) fn summary_contract(item: EmailSummaryContract) -> ProtoMessageSummaryContract {
    ProtoMessageSummaryContract {
        key_points: item.key_points,
        action_items: item.action_items,
        risks: item.risks,
        deadlines: item.deadlines,
        event_candidates: item
            .event_candidates
            .into_iter()
            .map(knowledge_candidate)
            .collect(),
        persona_candidates: item
            .persona_candidates
            .into_iter()
            .map(knowledge_candidate)
            .collect(),
        organization_candidates: item
            .organization_candidates
            .into_iter()
            .map(knowledge_candidate)
            .collect(),
        document_candidates: item
            .document_candidates
            .into_iter()
            .map(knowledge_candidate)
            .collect(),
        agreement_candidates: item
            .agreement_candidates
            .into_iter()
            .map(knowledge_candidate)
            .collect(),
        ..Default::default()
    }
}
fn knowledge_candidate(item: EmailKnowledgeCandidate) -> ProtoMessageKnowledgeCandidate {
    ProtoMessageKnowledgeCandidate {
        title: item.title,
        evidence: item.evidence,
        ..Default::default()
    }
}

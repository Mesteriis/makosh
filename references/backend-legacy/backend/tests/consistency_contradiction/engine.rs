use makosh_hub_backend::engines::consistency::{
    engine::ConsistencyEngine,
    models::{
        AcceptedClaim, ContradictionReviewState, ContradictionSeverity, ContradictionSourceKind,
        EvidenceClaimExtractionInput, NewEvidenceClaim,
    },
};
use serde_json::json;

#[test]
fn consistency_engine_detects_direct_claim_contradiction_from_structured_claims() {
    let accepted = AcceptedClaim {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        claim_type: "location".to_owned(),
        value: "Berlin".to_owned(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: "person_fact:location:alex".to_owned(),
        confidence: 0.95,
    };
    let new_claim = NewEvidenceClaim {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        claim_type: "location".to_owned(),
        value: "Madrid".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:location-update".to_owned(),
        confidence: 0.87,
    };

    let observations = ConsistencyEngine::detect_claim_contradictions(&[accepted], &[new_claim])
        .expect("detect contradictions");

    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.old_source_id, "person_fact:location:alex");
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.new_source_id, "message:location-update");
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.87);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.review_state,
        ContradictionReviewState::Suggested
    );
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": "person:v1:email:alex@example.com"}])
    );
}

#[test]
fn consistency_engine_ignores_matching_claims_after_normalization() {
    let accepted = AcceptedClaim {
        subject_id: "project:v1:makosh".to_owned(),
        claim_type: "status".to_owned(),
        value: " Active ".to_owned(),
        source_kind: ContradictionSourceKind::Knowledge,
        source_id: "knowledge:project-status".to_owned(),
        confidence: 0.9,
    };
    let new_claim = NewEvidenceClaim {
        subject_id: "project:v1:makosh".to_owned(),
        claim_type: "status".to_owned(),
        value: "active".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:project-status".to_owned(),
        confidence: 0.8,
    };

    let observations = ConsistencyEngine::detect_claim_contradictions(&[accepted], &[new_claim])
        .expect("detect contradictions");

    assert_eq!(observations, Vec::new());
}

#[test]
fn consistency_engine_extracts_structured_claims_from_communication_evidence() {
    let input = EvidenceClaimExtractionInput {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:claim-extraction".to_owned(),
        text: "Location: Madrid\nStatus = active\nNotes without claim\nEmpty:".to_owned(),
        confidence: 0.81,
    };

    let claims =
        ConsistencyEngine::extract_evidence_claims(&input).expect("extract evidence claims");

    assert_eq!(
        claims,
        vec![
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "location".to_owned(),
                value: "Madrid".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:claim-extraction".to_owned(),
                confidence: 0.81,
            },
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "status".to_owned(),
                value: "active".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:claim-extraction".to_owned(),
                confidence: 0.81,
            },
        ]
    );
}

#[test]
fn consistency_engine_extracts_deterministic_natural_language_claims_from_evidence() {
    let input = EvidenceClaimExtractionInput {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:natural-language-claim-extraction".to_owned(),
        text: "Quick update: I am now in Madrid.\nThe project status is blocked.".to_owned(),
        confidence: 0.79,
    };

    let claims =
        ConsistencyEngine::extract_evidence_claims(&input).expect("extract evidence claims");

    assert_eq!(
        claims,
        vec![
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "location".to_owned(),
                value: "Madrid".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:natural-language-claim-extraction".to_owned(),
                confidence: 0.79,
            },
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "status".to_owned(),
                value: "blocked".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:natural-language-claim-extraction".to_owned(),
                confidence: 0.79,
            },
        ]
    );
}

#[test]
fn consistency_engine_detects_document_evidence_contradiction_after_claim_extraction() {
    let accepted = AcceptedClaim {
        subject_id: "project:v1:makosh".to_owned(),
        claim_type: "status".to_owned(),
        value: "green".to_owned(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: "memory:project-status".to_owned(),
        confidence: 0.92,
    };
    let document = EvidenceClaimExtractionInput {
        subject_id: "project:v1:makosh".to_owned(),
        source_kind: ContradictionSourceKind::Document,
        source_id: "document:weekly-report".to_owned(),
        text: "Status: blocked".to_owned(),
        confidence: 0.84,
    };

    let observations = ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[document])
        .expect("detect evidence contradictions");

    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.old_source_id, "memory:project-status");
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Document
    );
    assert_eq!(observation.new_source_id, "document:weekly-report");
    assert_eq!(observation.old_claim, "status=green");
    assert_eq!(observation.new_claim, "status=blocked");
    assert_eq!(observation.confidence, 0.84);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "status",
            "source_kind": "document"
        })
    );
}

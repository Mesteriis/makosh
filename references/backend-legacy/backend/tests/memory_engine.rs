use chrono::{Duration, TimeZone, Utc};
use makosh_hub_backend::engines::memory::{
    MemoryEngine,
    models::{
        MemoryCardDraft, MemoryContextSource, MemoryEntityRef, MemoryFactDraft, MemoryFactState,
    },
};

#[test]
fn memory_engine_builds_persona_notes_memory_card_draft() {
    let draft = MemoryEngine::persona_notes_memory_card(
        "person:v1:email:alice@example.com",
        "  Met Alice at the local-first workshop.  ",
    )
    .expect("notes should create a memory card draft");

    assert_eq!(draft.title, "Compatibility notes");
    assert_eq!(draft.description, "Met Alice at the local-first workshop.");
    assert_eq!(
        draft.source,
        "personas.notes:person:v1:email:alice@example.com"
    );
    assert_eq!(draft.confidence, 1.0);
    assert_eq!(draft.importance, 5);
}

#[test]
fn memory_engine_ignores_empty_persona_notes() {
    let draft = MemoryEngine::persona_notes_memory_card("person:v1:email:alice@example.com", "  ");

    assert!(draft.is_none());
}

#[test]
fn memory_engine_builds_source_backed_persona_fact_draft() {
    let draft = MemoryEngine::persona_fact_memory(
        "person:v1:email:alice@example.com",
        " interest ",
        " local-first systems ",
        " communication_messages:message-1 ",
        0.84,
    )
    .expect("source-backed persona fact should be valid");

    assert_eq!(draft.affected_entity_kind, "persona");
    assert_eq!(
        draft.affected_entity_id,
        "person:v1:email:alice@example.com"
    );
    assert_eq!(draft.fact_type, "interest");
    assert_eq!(draft.value, "local-first systems");
    assert_eq!(draft.source, "communication_messages:message-1");
    assert_eq!(draft.confidence, 0.84);
    assert_eq!(draft.review_state, "accepted");
    assert_eq!(draft.produced_by, "memory_engine");
}

#[test]
fn memory_engine_rejects_unsourced_persona_fact_draft() {
    let error = MemoryEngine::persona_fact_memory(
        "person:v1:email:alice@example.com",
        "interest",
        "local-first systems",
        " ",
        0.84,
    )
    .expect_err("fact source should be required");

    assert_eq!(error.to_string(), "memory source must not be empty");
}

#[test]
fn memory_engine_builds_source_backed_context_pack_for_entity() {
    let persona_id = "person:v1:email:alice@example.com";
    let facts = vec![
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "interest".to_owned(),
            value: "local-first systems".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.92,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "project".to_owned(),
            value: "Макошь".to_owned(),
            source: "documents:doc-1".to_owned(),
            confidence: 0.81,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: "person:v1:email:bob@example.com".to_owned(),
            fact_type: "interest".to_owned(),
            value: "unrelated".to_owned(),
            source: "communication_messages:message-2".to_owned(),
            confidence: 0.99,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
    ];
    let cards = vec![MemoryCardDraft {
        title: "Compatibility notes".to_owned(),
        description: "Met Alice at the local-first workshop.".to_owned(),
        source: format!("personas.notes:{persona_id}"),
        confidence: 1.0,
        importance: 5,
    }];

    let pack = MemoryEngine::context_pack("persona", persona_id, &facts, &cards, 10)
        .expect("context pack should be valid");

    assert_eq!(pack.affected_entity_kind, "persona");
    assert_eq!(pack.affected_entity_id, persona_id);
    assert_eq!(pack.items.len(), 3);
    assert_eq!(pack.items[0].item_kind, "memory_card");
    assert_eq!(pack.items[0].title, "Compatibility notes");
    assert_eq!(pack.items[0].source, format!("personas.notes:{persona_id}"));
    assert_eq!(pack.items[1].item_kind, "fact");
    assert_eq!(pack.items[1].title, "interest");
    assert_eq!(pack.items[1].body, "local-first systems");
    assert_eq!(pack.items[2].title, "project");
    assert_eq!(
        pack.source_citations,
        vec![
            format!("personas.notes:{persona_id}"),
            "communication_messages:message-1".to_owned(),
            "documents:doc-1".to_owned(),
        ]
    );
    assert_eq!(pack.produced_by, "memory_engine");
    assert_eq!(pack.confidence, 0.91);
}

#[test]
fn memory_engine_detects_missing_source_backed_fact_types_for_entity() {
    let persona_id = "person:v1:email:alice@example.com";
    let facts = vec![
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "interest".to_owned(),
            value: "local-first systems".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.92,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
        MemoryFactDraft {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "project".to_owned(),
            value: "Макошь".to_owned(),
            source: "documents:doc-1".to_owned(),
            confidence: 0.81,
            review_state: "accepted".to_owned(),
            produced_by: "memory_engine".to_owned(),
        },
    ];

    let gaps = MemoryEngine::memory_gaps(
        "persona",
        persona_id,
        &["interest", " project ", "preference", "interest"],
        &facts,
    )
    .expect("memory gaps should be valid");

    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].affected_entity_kind, "persona");
    assert_eq!(gaps[0].affected_entity_id, persona_id);
    assert_eq!(gaps[0].missing_fact_type, "preference");
    assert_eq!(
        gaps[0].source,
        "memory_engine:gap:persona:person:v1:email:alice@example.com:preference"
    );
    assert_eq!(gaps[0].review_state, "suggested");
    assert_eq!(gaps[0].produced_by, "memory_engine");
}

#[test]
fn memory_engine_detects_stale_source_backed_facts_for_entity() {
    let persona_id = "person:v1:email:alice@example.com";
    let as_of = Utc.with_ymd_and_hms(2026, 6, 13, 12, 0, 0).unwrap();
    let facts = vec![
        MemoryFactState {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "interest".to_owned(),
            value: "local-first systems".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.92,
            review_state: "accepted".to_owned(),
            last_verified_at: Some(as_of - Duration::days(120)),
        },
        MemoryFactState {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: persona_id.to_owned(),
            fact_type: "project".to_owned(),
            value: "Макошь".to_owned(),
            source: "documents:doc-1".to_owned(),
            confidence: 0.81,
            review_state: "accepted".to_owned(),
            last_verified_at: Some(as_of - Duration::days(4)),
        },
        MemoryFactState {
            affected_entity_kind: "persona".to_owned(),
            affected_entity_id: "person:v1:email:bob@example.com".to_owned(),
            fact_type: "interest".to_owned(),
            value: "unrelated".to_owned(),
            source: "communication_messages:message-2".to_owned(),
            confidence: 0.99,
            review_state: "accepted".to_owned(),
            last_verified_at: Some(as_of - Duration::days(120)),
        },
    ];

    let stale = MemoryEngine::stale_memory_candidates("persona", persona_id, &facts, as_of, 90)
        .expect("stale candidates should be valid");

    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].affected_entity_kind, "persona");
    assert_eq!(stale[0].affected_entity_id, persona_id);
    assert_eq!(stale[0].fact_type, "interest");
    assert_eq!(stale[0].value, "local-first systems");
    assert_eq!(stale[0].source, "communication_messages:message-1");
    assert_eq!(stale[0].last_verified_at, Some(as_of - Duration::days(120)));
    assert_eq!(stale[0].review_state, "suggested");
    assert_eq!(stale[0].produced_by, "memory_engine");
}

#[test]
fn memory_engine_assembles_cross_domain_context_for_related_entities() {
    let project_id = "project:makosh";
    let related_entities = vec![
        MemoryEntityRef {
            entity_kind: "communication".to_owned(),
            entity_id: "communication:email-1".to_owned(),
            relation_kind: "source_evidence".to_owned(),
        },
        MemoryEntityRef {
            entity_kind: "document".to_owned(),
            entity_id: "document:architecture-note".to_owned(),
            relation_kind: "context_document".to_owned(),
        },
    ];
    let sources = vec![
        MemoryContextSource {
            entity_kind: "project".to_owned(),
            entity_id: project_id.to_owned(),
            item_kind: "fact".to_owned(),
            title: "status".to_owned(),
            body: "Implementation alignment is in progress.".to_owned(),
            source: "projects:makosh".to_owned(),
            confidence: 0.95,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "communication".to_owned(),
            entity_id: "communication:email-1".to_owned(),
            item_kind: "communication_summary".to_owned(),
            title: "Email evidence".to_owned(),
            body: "The message introduced an obligation for the project.".to_owned(),
            source: "communication_messages:message-1".to_owned(),
            confidence: 0.91,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "document".to_owned(),
            entity_id: "document:architecture-note".to_owned(),
            item_kind: "decision".to_owned(),
            title: "Architecture decision".to_owned(),
            body: "Memory context must preserve source citations.".to_owned(),
            source: "documents:architecture-note".to_owned(),
            confidence: 0.88,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "organization".to_owned(),
            entity_id: "organization:unrelated".to_owned(),
            item_kind: "fact".to_owned(),
            title: "Unrelated organization".to_owned(),
            body: "This item is outside the requested context graph.".to_owned(),
            source: "organizations:unrelated".to_owned(),
            confidence: 1.0,
            review_state: "accepted".to_owned(),
        },
        MemoryContextSource {
            entity_kind: "communication".to_owned(),
            entity_id: "communication:email-1".to_owned(),
            item_kind: "fact".to_owned(),
            title: "Rejected evidence".to_owned(),
            body: "Rejected evidence must not enter the context pack.".to_owned(),
            source: "communication_messages:message-rejected".to_owned(),
            confidence: 0.99,
            review_state: "user_rejected".to_owned(),
        },
    ];

    let pack = MemoryEngine::cross_domain_context_pack(
        "project",
        project_id,
        &related_entities,
        &sources,
        10,
    )
    .expect("cross-domain context pack should be valid");

    assert_eq!(pack.root_entity_kind, "project");
    assert_eq!(pack.root_entity_id, project_id);
    assert_eq!(pack.items.len(), 3);
    assert_eq!(pack.items[0].entity_kind, "project");
    assert_eq!(pack.items[0].relation_kind, "self");
    assert_eq!(pack.items[1].entity_kind, "communication");
    assert_eq!(pack.items[1].relation_kind, "source_evidence");
    assert_eq!(pack.items[2].entity_kind, "document");
    assert_eq!(pack.items[2].relation_kind, "context_document");
    assert_eq!(
        pack.entity_citations,
        vec![
            "project:project:makosh".to_owned(),
            "communication:communication:email-1".to_owned(),
            "document:document:architecture-note".to_owned(),
        ]
    );
    assert_eq!(
        pack.source_citations,
        vec![
            "projects:makosh".to_owned(),
            "communication_messages:message-1".to_owned(),
            "documents:architecture-note".to_owned(),
        ]
    );
    assert_eq!(pack.confidence, 0.91);
    assert_eq!(pack.produced_by, "memory_engine");
}

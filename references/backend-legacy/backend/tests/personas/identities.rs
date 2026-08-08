use makosh_hub_backend::domains::personas::api::store::PersonaProjectionStore;
use makosh_hub_backend::domains::personas::core::identities::PersonaIdentityStore;

use super::support::{live_personas_pool, unique_suffix};

#[tokio::test]
async fn persona_identities_accept_document_and_message_traces_against_postgres() {
    let Some(pool) = live_personas_pool("persona identity trace type").await else {
        return;
    };
    let projection_store = PersonaProjectionStore::new(pool.clone());
    let identity_store = PersonaIdentityStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = projection_store
        .upsert_email_persona(&format!("identity-trace-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let document_trace = identity_store
        .upsert(
            &person.persona_id,
            "document_mention",
            &format!("document:v1:{suffix}:identity-trace"),
            "document_processing",
        )
        .await
        .expect("upsert document mention identity trace");
    let message_trace = identity_store
        .upsert(
            &person.persona_id,
            "message_participant",
            &format!("message:v1:{suffix}:identity-trace"),
            "communication_projection",
        )
        .await
        .expect("upsert message participant identity trace");

    assert_eq!(document_trace.identity_type, "document_mention");
    assert_eq!(document_trace.source, "document_processing");
    assert_eq!(message_trace.identity_type, "message_participant");
    assert_eq!(message_trace.source, "communication_projection");

    let identities = identity_store
        .list_by_person(&person.persona_id)
        .await
        .expect("list persona identities");
    assert!(
        identities
            .iter()
            .any(|identity| identity.identity_type == "document_mention")
    );
    assert!(
        identities
            .iter()
            .any(|identity| identity.identity_type == "message_participant")
    );
}

#[tokio::test]
async fn persona_identities_accept_disputed_status_against_postgres() {
    let Some(pool) = live_personas_pool("persona identity disputed status").await else {
        return;
    };
    let projection_store = PersonaProjectionStore::new(pool.clone());
    let identity_store = PersonaIdentityStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = projection_store
        .upsert_email_persona(&format!("identity-disputed-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let identity = identity_store
        .upsert(
            &person.persona_id,
            "email",
            &format!("identity-disputed-trace-{suffix}@example.com"),
            "manual",
        )
        .await
        .expect("upsert identity");

    identity_store
        .update_status(&identity.id, "disputed")
        .await
        .expect("mark identity as disputed");

    let identities = identity_store
        .list_by_person(&person.persona_id)
        .await
        .expect("list persona identities");
    let updated = identities
        .iter()
        .find(|candidate| candidate.id == identity.id)
        .expect("updated identity");
    assert_eq!(updated.status, "disputed");
}

#[tokio::test]
async fn persona_identities_support_unattached_trace_assignment_against_postgres() {
    let Some(pool) = live_personas_pool("unattached persona identity trace").await else {
        return;
    };
    let projection_store = PersonaProjectionStore::new(pool.clone());
    let identity_store = PersonaIdentityStore::new(pool.clone());
    let suffix = unique_suffix();

    let trace = identity_store
        .create_unattached(
            "message_participant",
            &format!("message:v1:{suffix}:unattached-participant"),
            "communication_projection",
        )
        .await
        .expect("create unattached identity trace");
    assert_eq!(trace.persona_id.as_deref(), None);
    assert_eq!(trace.identity_type, "message_participant");
    assert_eq!(trace.source, "communication_projection");

    let person = projection_store
        .upsert_email_persona(&format!("attach-trace-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let attached = identity_store
        .attach_to_persona(&trace.id, &person.persona_id)
        .await
        .expect("attach identity trace to persona");

    assert_eq!(attached.id, trace.id);
    assert_eq!(
        attached.persona_id.as_deref(),
        Some(person.persona_id.as_str())
    );
    assert_eq!(attached.status, "active");

    let identities = identity_store
        .list_by_person(&person.persona_id)
        .await
        .expect("list persona identities");
    assert!(identities.iter().any(|identity| {
        identity.id == trace.id
            && identity.persona_id.as_deref() == Some(person.persona_id.as_str())
    }));
}

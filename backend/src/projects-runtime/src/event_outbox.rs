use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_projects_persistence::{ProjectsPersistenceErrorV1, ProjectsPersistenceV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectsEventRelayErrorV1 {
    InvalidTimestamp,
    Persistence(ProjectsPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_projects_outbox_once_v1(
    persistence: &ProjectsPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, ProjectsEventRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(ProjectsEventRelayErrorV1::InvalidTimestamp);
    }
    let Some(claim) = persistence
        .claim_next_pending_outbox(logical_owner_id)
        .await
        .map_err(ProjectsEventRelayErrorV1::Persistence)?
    else {
        return Ok(false);
    };
    let record = claim.record().clone();
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| ProjectsEventRelayErrorV1::EventUnavailable)?;
    claim
        .mark_published(record.envelope_sha256, published_at_unix_millis)
        .await
        .map_err(ProjectsEventRelayErrorV1::Persistence)?;
    Ok(true)
}

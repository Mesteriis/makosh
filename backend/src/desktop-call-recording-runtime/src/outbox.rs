use makosh_desktop_call_recording_persistence::{
    DesktopCallRecordingRepositoryV1, PersistenceErrorV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopRecordingOutboxErrorV1 {
    InvalidTimestamp,
    Persistence(PersistenceErrorV1),
    Unavailable,
}

pub async fn relay_outbox_once_v1(
    persistence: &DesktopCallRecordingRepositoryV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_ms: i64,
) -> Result<usize, DesktopRecordingOutboxErrorV1> {
    if published_at_unix_ms <= 0 {
        return Err(DesktopRecordingOutboxErrorV1::InvalidTimestamp);
    }
    let records = persistence
        .pending_outbox(64)
        .await
        .map_err(DesktopRecordingOutboxErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, &record.exact_envelope_bytes)
            .await
            .map_err(|_| DesktopRecordingOutboxErrorV1::Unavailable)?;
        persistence
            .mark_outbox_delivered(
                record.event_id,
                record.envelope_sha256,
                published_at_unix_ms,
            )
            .await
            .map_err(DesktopRecordingOutboxErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

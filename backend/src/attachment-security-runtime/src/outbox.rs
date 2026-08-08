//! Exact-byte relay for owner-local verdict outbox records.

use makosh_attachment_security_persistence::{
    AttachmentSecurityPersistenceErrorV1, AttachmentSecurityPersistenceV1,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

pub async fn relay_attachment_security_verdict_outbox_once_v1(
    persistence: &AttachmentSecurityPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, AttachmentSecurityOutboxRelayErrorV1> {
    let records = persistence
        .pending_verdict_outbox(64)
        .await
        .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| AttachmentSecurityOutboxRelayErrorV1::Unavailable)?;
        persistence
            .mark_verdict_outbox_published(*record.message_id(), published_at_unix_seconds)
            .await
            .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

pub async fn relay_attachment_security_archive_delegation_outbox_once_v1(
    persistence: &AttachmentSecurityPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, AttachmentSecurityOutboxRelayErrorV1> {
    let records = persistence
        .pending_archive_delegation_outbox(64)
        .await
        .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| AttachmentSecurityOutboxRelayErrorV1::Unavailable)?;
        persistence
            .mark_archive_delegation_outbox_published(
                *record.message_id(),
                published_at_unix_seconds,
            )
            .await
            .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

pub async fn relay_attachment_security_text_delegation_outbox_once_v1(
    persistence: &AttachmentSecurityPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, AttachmentSecurityOutboxRelayErrorV1> {
    let records = persistence
        .pending_text_delegation_outbox(64)
        .await
        .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| AttachmentSecurityOutboxRelayErrorV1::Unavailable)?;
        persistence
            .mark_text_delegation_outbox_published(*record.message_id(), published_at_unix_seconds)
            .await
            .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

pub async fn relay_attachment_security_preview_delegation_outbox_once_v1(
    persistence: &AttachmentSecurityPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, AttachmentSecurityOutboxRelayErrorV1> {
    let records = persistence
        .pending_preview_delegation_outbox(64)
        .await
        .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|_| AttachmentSecurityOutboxRelayErrorV1::Unavailable)?;
        persistence
            .mark_preview_delegation_outbox_published(
                *record.message_id(),
                published_at_unix_seconds,
            )
            .await
            .map_err(AttachmentSecurityOutboxRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSecurityOutboxRelayErrorV1 {
    Persistence(AttachmentSecurityPersistenceErrorV1),
    Unavailable,
}

use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};
use makosh_mail_contacts_sync_persistence::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MailContactsSyncRelayErrorV1 {
    InvalidTimestamp,
    Persistence(MailContactsSyncPersistenceErrorV1),
    EventUnavailable,
}

pub(crate) async fn relay_mail_contacts_sync_outbox_once_v1(
    persistence: &MailContactsSyncPersistenceV1,
    logical_owner_id: &str,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_millis: i64,
) -> Result<bool, MailContactsSyncRelayErrorV1> {
    if published_at_unix_millis <= 0 {
        return Err(MailContactsSyncRelayErrorV1::InvalidTimestamp);
    }
    let Some(record) = persistence
        .unpublished_commands(logical_owner_id, 1)
        .await
        .map_err(MailContactsSyncRelayErrorV1::Persistence)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    connection
        .publish_exact(permit, &record.envelope_bytes)
        .await
        .map_err(|_| MailContactsSyncRelayErrorV1::EventUnavailable)?;
    persistence
        .mark_command_published(
            logical_owner_id,
            &record.message_id,
            &record.envelope_sha256,
            published_at_unix_millis,
        )
        .await
        .map_err(MailContactsSyncRelayErrorV1::Persistence)?;
    Ok(true)
}

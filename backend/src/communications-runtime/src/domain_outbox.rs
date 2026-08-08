use makosh_communications_persistence::{
    CommunicationsDurablePersistence, CommunicationsPersistenceError,
};
use makosh_events_jetstream::{RuntimeJetStreamConnection, RuntimePublishPermitV1};

pub async fn relay_domain_outbox_once(
    persistence: &CommunicationsDurablePersistence,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimePublishPermitV1,
    published_at_unix_seconds: i64,
) -> Result<usize, CommunicationsDomainOutboxRelayErrorV1> {
    let records = persistence
        .pending_domain_outbox(64)
        .await
        .map_err(CommunicationsDomainOutboxRelayErrorV1::Persistence)?;
    let mut published = 0;
    for record in records {
        connection
            .publish_exact(permit, record.exact_bytes())
            .await
            .map_err(|error| {
                if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                    eprintln!("developer_communications_domain_outbox_publish_error={error}");
                }
                CommunicationsDomainOutboxRelayErrorV1::Unavailable
            })?;
        persistence
            .mark_domain_outbox_published(record.message_id(), published_at_unix_seconds)
            .await
            .map_err(CommunicationsDomainOutboxRelayErrorV1::Persistence)?;
        published += 1;
    }
    Ok(published)
}

#[derive(Debug)]
pub enum CommunicationsDomainOutboxRelayErrorV1 {
    Persistence(CommunicationsPersistenceError),
    Unavailable,
}

impl CommunicationsDomainOutboxRelayErrorV1 {
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Unavailable
                | Self::Persistence(CommunicationsPersistenceError::StorageUnavailable)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_transient_infrastructure_failures_are_retryable() {
        assert!(CommunicationsDomainOutboxRelayErrorV1::Unavailable.is_retryable());
        assert!(
            CommunicationsDomainOutboxRelayErrorV1::Persistence(
                CommunicationsPersistenceError::StorageUnavailable,
            )
            .is_retryable()
        );
        assert!(
            !CommunicationsDomainOutboxRelayErrorV1::Persistence(
                CommunicationsPersistenceError::InvalidRow,
            )
            .is_retryable()
        );
    }
}

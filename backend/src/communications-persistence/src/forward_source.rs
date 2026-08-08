use makosh_events_protocol::delivery::OutboxRecordV1;
use sqlx::Row;

use crate::{CommunicationsConsumeOutcomeV1, CommunicationsDurablePersistence};

const MAX_FORWARD_SOURCE_BYTES_V1: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsCrossChannelForwardBodyReceiptV1 {
    pub reference_id: [u8; 16],
    pub declared_bytes: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunicationsCrossChannelForwardSourceSnapshotV1 {
    pub source_message_id: [u8; 16],
    pub target_conversation_id: [u8; 16],
    pub evidence_id: [u8; 16],
    pub evidence_revision: u64,
    pub body: CommunicationsCrossChannelForwardBodyReceiptV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsCrossChannelForwardSourceErrorV1 {
    InvalidRequest,
    SourceMissingOrInactive,
    TargetMissing,
    SameChannel,
    ContentUnavailable,
    ContentLimit,
    StaleRevision,
    InvalidRow,
    StorageUnavailable,
    InboxHashConflict,
    OutboxConflict,
}

impl CommunicationsDurablePersistence {
    pub async fn cross_channel_forward_source_snapshot(
        &self,
        source_message_id: [u8; 16],
        target_conversation_id: [u8; 16],
    ) -> Result<
        CommunicationsCrossChannelForwardSourceSnapshotV1,
        CommunicationsCrossChannelForwardSourceErrorV1,
    > {
        if !valid_id16(&source_message_id) || !valid_id16(&target_conversation_id) {
            return Err(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRequest);
        }
        let row = sqlx::query(
            "SELECT message.message_id, message.last_evidence_id AS evidence_id,
               message.canonical_revision, source_conversation.provider AS source_provider,
               (
                 SELECT target_conversation.provider
                 FROM makosh_data.communications_conversations target_conversation
                 WHERE target_conversation.conversation_id = $2
               ) AS target_provider,
               evidence.body_state, evidence.body_blob_reference_id,
               evidence.body_blob_declared_bytes, evidence.body_blob_sha256
             FROM makosh_data.communications_messages message
             JOIN makosh_data.communications_conversations source_conversation
               ON source_conversation.conversation_id = message.conversation_id
             JOIN makosh_data.communications_evidence_summaries evidence
               ON evidence.observation_id = message.last_evidence_id
             WHERE message.message_id = $1
               AND message.lifecycle_state = 1",
        )
        .bind(source_message_id.as_slice())
        .bind(target_conversation_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable)?
        .ok_or(CommunicationsCrossChannelForwardSourceErrorV1::SourceMissingOrInactive)?;
        let source_provider: i16 = row
            .try_get("source_provider")
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?;
        let target_provider: Option<i16> = row
            .try_get("target_provider")
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?;
        let target_provider =
            target_provider.ok_or(CommunicationsCrossChannelForwardSourceErrorV1::TargetMissing)?;
        if channel_kind(source_provider)? == channel_kind(target_provider)? {
            return Err(CommunicationsCrossChannelForwardSourceErrorV1::SameChannel);
        }
        let body_state: i16 = row
            .try_get("body_state")
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?;
        if body_state != 4 {
            return Err(CommunicationsCrossChannelForwardSourceErrorV1::ContentUnavailable);
        }
        let declared_bytes = u64::try_from(
            row.try_get::<i64, _>("body_blob_declared_bytes")
                .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
        )
        .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?;
        if !(1..=MAX_FORWARD_SOURCE_BYTES_V1).contains(&declared_bytes) {
            return Err(CommunicationsCrossChannelForwardSourceErrorV1::ContentLimit);
        }
        Ok(CommunicationsCrossChannelForwardSourceSnapshotV1 {
            source_message_id: id16(
                &row.try_get::<Vec<u8>, _>("message_id")
                    .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
            )?,
            target_conversation_id,
            evidence_id: id16(
                &row.try_get::<Vec<u8>, _>("evidence_id")
                    .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
            )?,
            evidence_revision: u64::try_from(
                row.try_get::<i64, _>("canonical_revision")
                    .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
            )
            .ok()
            .filter(|revision| *revision > 0)
            .ok_or(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
            body: CommunicationsCrossChannelForwardBodyReceiptV1 {
                reference_id: id16(
                    &row.try_get::<Vec<u8>, _>("body_blob_reference_id")
                        .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
                )?,
                declared_bytes,
                sha256: id32(
                    &row.try_get::<Vec<u8>, _>("body_blob_sha256")
                        .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
                )?,
            },
        })
    }

    pub async fn persist_cross_channel_forward_source_result(
        &self,
        command_message_id: [u8; 16],
        command_envelope_sha256: [u8; 32],
        expected_current_snapshot: Option<&CommunicationsCrossChannelForwardSourceSnapshotV1>,
        result_outbox: &OutboxRecordV1,
        created_at_unix_seconds: i64,
    ) -> Result<CommunicationsConsumeOutcomeV1, CommunicationsCrossChannelForwardSourceErrorV1>
    {
        if !valid_id16(&command_message_id)
            || !valid_sha256(&command_envelope_sha256)
            || !valid_id16(result_outbox.message_id())
            || created_at_unix_seconds <= 0
        {
            return Err(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRequest);
        }
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable)?;
        if let Some(snapshot) = expected_current_snapshot {
            validate_snapshot(snapshot)?;
            let current = sqlx::query(
                "SELECT message.canonical_revision,
                   source_conversation.provider AS source_provider,
                   target_conversation.provider AS target_provider
                 FROM makosh_data.communications_messages message
                 JOIN makosh_data.communications_conversations source_conversation
                   ON source_conversation.conversation_id = message.conversation_id
                 JOIN makosh_data.communications_conversations target_conversation
                   ON target_conversation.conversation_id = $2
                 WHERE message.message_id = $1
                   AND message.lifecycle_state = 1",
            )
            .bind(snapshot.source_message_id.as_slice())
            .bind(snapshot.target_conversation_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable)?;
            let Some(current) = current else {
                return Err(CommunicationsCrossChannelForwardSourceErrorV1::StaleRevision);
            };
            let revision = u64::try_from(
                current
                    .try_get::<i64, _>("canonical_revision")
                    .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?,
            )
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?;
            let source_provider: i16 = current
                .try_get("source_provider")
                .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?;
            let target_provider: i16 = current
                .try_get("target_provider")
                .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)?;
            if revision != snapshot.evidence_revision
                || channel_kind(source_provider)? == channel_kind(target_provider)?
            {
                return Err(CommunicationsCrossChannelForwardSourceErrorV1::StaleRevision);
            }
        }
        let inserted = sqlx::query(
            "INSERT INTO makosh_data.communications_event_inbox
               (message_id, envelope_sha256)
             VALUES ($1, $2)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(command_message_id.as_slice())
        .bind(command_envelope_sha256.as_slice())
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable)?;
        if inserted.rows_affected() == 0 {
            let existing: Option<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_sha256
                 FROM makosh_data.communications_event_inbox
                 WHERE message_id = $1",
            )
            .bind(command_message_id.as_slice())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable)?;
            return if existing.as_deref() == Some(command_envelope_sha256.as_slice()) {
                Ok(CommunicationsConsumeOutcomeV1::Duplicate)
            } else {
                Err(CommunicationsCrossChannelForwardSourceErrorV1::InboxHashConflict)
            };
        }
        let result = sqlx::query(
            "INSERT INTO makosh_data.communications_domain_outbox (
               message_id, envelope_sha256, exact_envelope_bytes,
               created_at_unix_seconds, published_at_unix_seconds
             ) VALUES ($1, $2, $3, $4, NULL)
             ON CONFLICT (message_id) DO NOTHING",
        )
        .bind(result_outbox.message_id().as_slice())
        .bind(result_outbox.envelope_sha256().as_slice())
        .bind(result_outbox.exact_bytes())
        .bind(created_at_unix_seconds)
        .execute(&mut *transaction)
        .await
        .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable)?;
        if result.rows_affected() != 1 {
            return Err(CommunicationsCrossChannelForwardSourceErrorV1::OutboxConflict);
        }
        transaction
            .commit()
            .await
            .map_err(|_| CommunicationsCrossChannelForwardSourceErrorV1::StorageUnavailable)?;
        Ok(CommunicationsConsumeOutcomeV1::Applied)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommunicationChannelKindV1 {
    Mail,
    Telegram,
    WhatsApp,
    Zulip,
}

fn channel_kind(
    provider: i16,
) -> Result<CommunicationChannelKindV1, CommunicationsCrossChannelForwardSourceErrorV1> {
    match provider {
        1 | 4 | 6 => Ok(CommunicationChannelKindV1::Mail),
        2 => Ok(CommunicationChannelKindV1::Telegram),
        3 => Ok(CommunicationChannelKindV1::WhatsApp),
        5 => Ok(CommunicationChannelKindV1::Zulip),
        _ => Err(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow),
    }
}

fn validate_snapshot(
    snapshot: &CommunicationsCrossChannelForwardSourceSnapshotV1,
) -> Result<(), CommunicationsCrossChannelForwardSourceErrorV1> {
    if !valid_id16(&snapshot.source_message_id)
        || !valid_id16(&snapshot.target_conversation_id)
        || !valid_id16(&snapshot.evidence_id)
        || snapshot.evidence_revision == 0
        || !valid_id16(&snapshot.body.reference_id)
        || !(1..=MAX_FORWARD_SOURCE_BYTES_V1).contains(&snapshot.body.declared_bytes)
        || !valid_sha256(&snapshot.body.sha256)
    {
        return Err(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRequest);
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], CommunicationsCrossChannelForwardSourceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_id16)
        .ok_or(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)
}

fn id32(value: &[u8]) -> Result<[u8; 32], CommunicationsCrossChannelForwardSourceErrorV1> {
    value
        .try_into()
        .ok()
        .filter(valid_sha256)
        .ok_or(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRow)
}

fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mail_transports_are_one_channel_kind() {
        assert_eq!(channel_kind(1), channel_kind(4));
        assert_eq!(channel_kind(4), channel_kind(6));
        assert_ne!(channel_kind(1), channel_kind(2));
    }

    #[test]
    fn snapshot_is_bounded_and_bodyless() {
        let mut snapshot = CommunicationsCrossChannelForwardSourceSnapshotV1 {
            source_message_id: [1; 16],
            target_conversation_id: [2; 16],
            evidence_id: [3; 16],
            evidence_revision: 4,
            body: CommunicationsCrossChannelForwardBodyReceiptV1 {
                reference_id: [5; 16],
                declared_bytes: 6,
                sha256: [7; 32],
            },
        };
        assert_eq!(validate_snapshot(&snapshot), Ok(()));
        snapshot.body.declared_bytes = MAX_FORWARD_SOURCE_BYTES_V1 + 1;
        assert_eq!(
            validate_snapshot(&snapshot),
            Err(CommunicationsCrossChannelForwardSourceErrorV1::InvalidRequest)
        );
    }
}

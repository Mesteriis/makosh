use makosh_persons_core::PersonsStateV1;

pub const PERSONS_MAX_ENVELOPE_BYTES_V1: usize = 256 * 1024;
pub const PERSONS_OUTBOX_READ_LIMIT_V1: i64 = 256;
pub const PERSONS_RECOVERY_ROW_LIMIT_V1: i64 = 4_096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyPersonsCommandV1 {
    pub logical_owner_id: String,
    pub command_message_id: [u8; 16],
    pub command_envelope_sha256: [u8; 32],
    pub command_id: [u8; 16],
    pub command_fingerprint: [u8; 32],
    pub expected_aggregate_revision: u64,
    pub received_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsEnvelopeRecordV1 {
    pub message_id: [u8; 16],
    pub envelope_sha256: [u8; 32],
    pub envelope_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsCommandCommitV1 {
    pub terminal_result: PersonsEnvelopeRecordV1,
    pub owner_events: Vec<PersonsEnvelopeRecordV1>,
    /// One bounded semantic kind/public-ID key per owner event. Keys are
    /// strictly increasing and are persisted with their per-command ordinal.
    pub owner_event_order_keys: Vec<Vec<u8>>,
    pub completed_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyPersonsCommandOutcomeV1 {
    pub replayed: bool,
    pub aggregate_revision: u64,
    pub terminal_result: PersonsEnvelopeRecordV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsOutboxRecordV1 {
    pub record: PersonsEnvelopeRecordV1,
    pub command_message_id: [u8; 16],
    /// Zero is reserved for rows backfilled from the V2 legacy outbox.
    pub resulting_owner_revision: u64,
    pub outbox_ordinal: u16,
    pub semantic_order_key: Vec<u8>,
    pub created_at_unix_millis: i64,
    pub published_at_unix_millis: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedPersonsOwnerV1 {
    pub aggregate_revision: u64,
    pub state: PersonsStateV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsPersistenceErrorV1 {
    InvalidInput,
    StorageUnavailable,
    CommandConflict,
    AggregateConflict,
    StateConflict,
    HashMismatch,
    MutationRejected,
}

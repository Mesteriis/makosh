//! Mail-owned attachment materialization, safety projection and delivery manifest schema.

pub const MAIL_SCHEMA_V5: &str = r#"
CREATE TABLE IF NOT EXISTS makosh_data.mail_attachment_safety_projections (
    attachment_anchor_id BYTEA PRIMARY KEY,
    state SMALLINT NOT NULL,
    evidence_id BYTEA,
    observed_at_unix_seconds BIGINT,
    CHECK (octet_length(attachment_anchor_id) = 16),
    CHECK (state BETWEEN 1 AND 6),
    CHECK ((evidence_id IS NULL AND observed_at_unix_seconds IS NULL)
        OR (octet_length(evidence_id) = 16 AND observed_at_unix_seconds IS NOT NULL))
);
CREATE TABLE IF NOT EXISTS makosh_data.mail_attachment_materializations (
    attachment_anchor_id BYTEA PRIMARY KEY,
    source_observation_id BYTEA NOT NULL UNIQUE,
    blob_reference_id BYTEA NOT NULL UNIQUE,
    receipt_sha256 BYTEA NOT NULL,
    declared_size BIGINT NOT NULL,
    filename TEXT,
    media_type TEXT NOT NULL,
    disposition SMALLINT NOT NULL,
    materialized_at_unix_seconds BIGINT NOT NULL,
    CHECK (octet_length(attachment_anchor_id) = 16),
    CHECK (octet_length(source_observation_id) = 16),
    CHECK (octet_length(blob_reference_id) = 16),
    CHECK (octet_length(receipt_sha256) = 32),
    CHECK (declared_size BETWEEN 1 AND 16777216),
    CHECK (filename IS NULL OR (octet_length(filename) BETWEEN 1 AND 512)),
    CHECK (octet_length(media_type) BETWEEN 3 AND 256),
    CHECK (disposition IN (1, 2)),
    CHECK (materialized_at_unix_seconds > 0)
);
ALTER TABLE makosh_data.mail_delivery_attempts
    ADD COLUMN IF NOT EXISTS request_sha256 BYTEA;
ALTER TABLE makosh_data.mail_delivery_attempts
    ADD COLUMN IF NOT EXISTS rendered_rfc822_sha256 BYTEA;
CREATE TABLE IF NOT EXISTS makosh_data.mail_delivery_attachment_manifest (
    operation_id TEXT NOT NULL
        REFERENCES makosh_data.mail_delivery_attempts (operation_id) ON DELETE CASCADE,
    ordinal SMALLINT NOT NULL,
    attachment_anchor_id BYTEA NOT NULL,
    blob_reference_id BYTEA NOT NULL,
    receipt_sha256 BYTEA NOT NULL,
    declared_size BIGINT NOT NULL,
    filename TEXT,
    media_type TEXT NOT NULL,
    disposition SMALLINT NOT NULL,
    safety_evidence_id BYTEA NOT NULL,
    PRIMARY KEY (operation_id, ordinal),
    UNIQUE (operation_id, attachment_anchor_id),
    CHECK (ordinal BETWEEN 0 AND 15),
    CHECK (octet_length(attachment_anchor_id) = 16),
    CHECK (octet_length(blob_reference_id) = 16),
    CHECK (octet_length(receipt_sha256) = 32),
    CHECK (declared_size BETWEEN 1 AND 16777216),
    CHECK (filename IS NULL OR (octet_length(filename) BETWEEN 1 AND 512)),
    CHECK (octet_length(media_type) BETWEEN 3 AND 256),
    CHECK (disposition IN (1, 2)),
    CHECK (octet_length(safety_evidence_id) = 16)
);
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum MailAttachmentDispositionV1 {
    Attachment = 1,
    Inline = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i16)]
pub enum MailAttachmentSafetyStateV1 {
    DescriptorOnly = 1,
    BlobPending = 2,
    BlobAdmitted = 3,
    Quarantined = 4,
    SafeForDelivery = 5,
    Rejected = 6,
}

impl MailAttachmentSafetyStateV1 {
    pub const fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::DescriptorOnly),
            2 => Some(Self::BlobPending),
            3 => Some(Self::BlobAdmitted),
            4 => Some(Self::Quarantined),
            5 => Some(Self::SafeForDelivery),
            6 => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAttachmentMaterializationV1 {
    pub source_observation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub blob_reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
    pub declared_size: u64,
    pub filename: Option<String>,
    pub media_type: String,
    pub disposition: MailAttachmentDispositionV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MailAttachmentSafetyTransitionV1 {
    pub attachment_anchor_id: [u8; 16],
    pub expected_state: MailAttachmentSafetyStateV1,
    pub next_state: MailAttachmentSafetyStateV1,
    pub evidence_id: [u8; 16],
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailDeliveryAttachmentManifestV1 {
    pub ordinal: u8,
    pub attachment_anchor_id: [u8; 16],
    pub blob_reference_id: [u8; 16],
    pub receipt_sha256: [u8; 32],
    pub declared_size: u64,
    pub filename: Option<String>,
    pub media_type: String,
    pub disposition: MailAttachmentDispositionV1,
    pub safety_evidence_id: [u8; 16],
}

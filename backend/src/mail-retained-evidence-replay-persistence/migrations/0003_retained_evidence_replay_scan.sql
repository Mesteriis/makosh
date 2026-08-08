CREATE TABLE makosh_data.mail_retained_evidence_replay_scan (
    message_id BYTEA PRIMARY KEY REFERENCES makosh_data.mail_attachment_security_outbox (
        message_id
    ),
    scanned_at_unix_seconds BIGINT NOT NULL CHECK (scanned_at_unix_seconds > 0)
);

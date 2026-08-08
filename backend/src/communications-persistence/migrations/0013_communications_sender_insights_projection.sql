ALTER TABLE makosh_data.communications_evidence_summaries
  ADD COLUMN participant_display_label TEXT CHECK (
    participant_display_label IS NULL
    OR (
      octet_length(participant_display_label) BETWEEN 1 AND 256
      AND participant_display_label = btrim(participant_display_label)
    )
  );

ALTER TABLE makosh_data.communications_observed_participants
  ADD COLUMN display_label TEXT CHECK (
    display_label IS NULL
    OR (
      octet_length(display_label) BETWEEN 1 AND 256
      AND display_label = btrim(display_label)
    )
  );

CREATE TABLE makosh_data.communications_sender_profiles (
  sender_id BYTEA PRIMARY KEY CHECK (octet_length(sender_id) = 16),
  display_label TEXT CHECK (
    display_label IS NULL
    OR (
      octet_length(display_label) BETWEEN 1 AND 256
      AND display_label = btrim(display_label)
    )
  ),
  first_observed_at_unix_seconds BIGINT NOT NULL,
  last_observed_at_unix_seconds BIGINT NOT NULL,
  last_evidence_id BYTEA NOT NULL CHECK (octet_length(last_evidence_id) = 16)
);

CREATE TABLE makosh_data.communications_message_sender_facts (
  message_id BYTEA PRIMARY KEY REFERENCES makosh_data.communications_messages (
    message_id
  ) CHECK (octet_length(message_id) = 16),
  sender_id BYTEA NOT NULL REFERENCES
    makosh_data.communications_sender_profiles (sender_id)
    CHECK (octet_length(sender_id) = 16),
  participant_id BYTEA NOT NULL REFERENCES
    makosh_data.communications_observed_participants (participant_id)
    CHECK (octet_length(participant_id) = 16),
  account_id BYTEA NOT NULL REFERENCES makosh_data.communications_accounts (
    account_id
  ) CHECK (octet_length(account_id) = 16),
  first_observed_at_unix_seconds BIGINT NOT NULL,
  last_observed_at_unix_seconds BIGINT NOT NULL,
  last_evidence_id BYTEA NOT NULL CHECK (octet_length(last_evidence_id) = 16)
);

CREATE INDEX communications_message_sender_facts_owner_rank
  ON makosh_data.communications_message_sender_facts (
    sender_id,
    last_observed_at_unix_seconds DESC,
    message_id ASC
  );

CREATE INDEX communications_message_sender_facts_account_rank
  ON makosh_data.communications_message_sender_facts (
    account_id,
    sender_id,
    last_observed_at_unix_seconds DESC,
    message_id ASC
  );

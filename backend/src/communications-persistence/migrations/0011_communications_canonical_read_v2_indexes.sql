CREATE INDEX communications_accounts_canonical_read_v2
  ON makosh_data.communications_accounts (
    last_observed_at_unix_seconds DESC,
    account_id ASC
  );

CREATE INDEX communications_conversations_canonical_read_v2
  ON makosh_data.communications_conversations (
    account_cursor_sha256,
    last_observed_at_unix_seconds DESC,
    conversation_id ASC
  );

CREATE INDEX communications_messages_canonical_read_v2
  ON makosh_data.communications_messages (
    conversation_id,
    last_observed_at_unix_seconds DESC,
    message_id ASC
  );

CREATE INDEX communications_observed_participants_canonical_read_v2
  ON makosh_data.communications_observed_participants (
    conversation_id,
    last_observed_at_unix_seconds DESC,
    participant_id ASC
  );

CREATE INDEX communications_attachment_anchors_canonical_read_v2
  ON makosh_data.communications_attachment_anchors (
    message_id,
    last_observed_at_unix_seconds DESC,
    attachment_anchor_id ASC
  );

CREATE INDEX communications_message_references_canonical_read_v2
  ON makosh_data.communications_message_references (
    source_message_id,
    observed_at_unix_seconds ASC,
    reference_kind ASC,
    reference_id ASC
  );

CREATE INDEX communications_evidence_summaries_canonical_read_v2
  ON makosh_data.communications_evidence_summaries (
    source_cursor_sha256,
    observed_at_unix_seconds DESC,
    observation_id ASC
  );

CREATE INDEX communications_derived_index_projections_canonical_read_v2
  ON makosh_data.communications_derived_index_projections (
    observed_at_unix_seconds DESC,
    message_id ASC
  );

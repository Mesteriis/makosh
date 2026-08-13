CREATE TABLE makosh_data.persons_lineage (
    logical_owner_id TEXT NOT NULL,
    lineage_sequence BIGINT NOT NULL CHECK (lineage_sequence > 0),
    change_kind SMALLINT NOT NULL CHECK (change_kind IN (1, 2)),
    source_person_id BYTEA NOT NULL CHECK (octet_length(source_person_id) = 16),
    target_person_id BYTEA NOT NULL CHECK (octet_length(target_person_id) = 16),
    preserved_display_name TEXT CHECK (
        preserved_display_name IS NULL OR char_length(preserved_display_name) BETWEEN 1 AND 240
    ),
    preserved_given_name TEXT CHECK (
        preserved_given_name IS NULL OR char_length(preserved_given_name) BETWEEN 1 AND 240
    ),
    preserved_family_name TEXT CHECK (
        preserved_family_name IS NULL OR char_length(preserved_family_name) BETWEEN 1 AND 240
    ),
    preserved_emails TEXT[] NOT NULL DEFAULT '{}'
        CHECK (cardinality(preserved_emails) <= 32)
        CHECK (array_position(preserved_emails, NULL) IS NULL)
        CHECK (octet_length(array_to_string(preserved_emails, '')) <= 10240),
    preserved_phones TEXT[] NOT NULL DEFAULT '{}'
        CHECK (cardinality(preserved_phones) <= 32)
        CHECK (array_position(preserved_phones, NULL) IS NULL)
        CHECK (octet_length(array_to_string(preserved_phones, '')) <= 512),
    selected_profile_fact_kinds SMALLINT[] NOT NULL DEFAULT '{}'
        CHECK (cardinality(selected_profile_fact_kinds) <= 5)
        CHECK (selected_profile_fact_kinds <@ ARRAY[1, 2, 3, 4, 5]::SMALLINT[])
        CHECK (cardinality(array_positions(selected_profile_fact_kinds, 1::SMALLINT)) <= 1)
        CHECK (cardinality(array_positions(selected_profile_fact_kinds, 2::SMALLINT)) <= 1)
        CHECK (cardinality(array_positions(selected_profile_fact_kinds, 3::SMALLINT)) <= 1)
        CHECK (cardinality(array_positions(selected_profile_fact_kinds, 4::SMALLINT)) <= 1)
        CHECK (cardinality(array_positions(selected_profile_fact_kinds, 5::SMALLINT)) <= 1),
    decision_id BYTEA NOT NULL CHECK (octet_length(decision_id) = 16),
    review_id BYTEA NOT NULL CHECK (octet_length(review_id) = 16),
    decision_revision BIGINT NOT NULL CHECK (decision_revision > 0),
    decided_by_owner_device_id BYTEA NOT NULL CHECK (octet_length(decided_by_owner_device_id) = 16),
    decided_at_unix_seconds BIGINT NOT NULL CHECK (decided_at_unix_seconds > 0),
    decided_at_nanos INTEGER NOT NULL CHECK (decided_at_nanos BETWEEN 0 AND 999999999),
    approved_action_digest BYTEA NOT NULL CHECK (octet_length(approved_action_digest) = 32),
    PRIMARY KEY (logical_owner_id, lineage_sequence),
    FOREIGN KEY (logical_owner_id) REFERENCES makosh_data.persons_owner_aggregates(logical_owner_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (logical_owner_id, source_person_id)
        REFERENCES makosh_data.persons_current(logical_owner_id, person_id) ON DELETE RESTRICT,
    FOREIGN KEY (logical_owner_id, target_person_id)
        REFERENCES makosh_data.persons_current(logical_owner_id, person_id) ON DELETE RESTRICT
);

CREATE TABLE makosh_data.persons_lineage_sources (
    logical_owner_id TEXT NOT NULL,
    lineage_sequence BIGINT NOT NULL,
    source_sequence INTEGER NOT NULL CHECK (source_sequence > 0),
    integration_public_id BYTEA NOT NULL CHECK (octet_length(integration_public_id) = 16),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    provider_source_contact_public_id BYTEA NOT NULL
        CHECK (octet_length(provider_source_contact_public_id) = 16),
    PRIMARY KEY (logical_owner_id, lineage_sequence, source_sequence),
    UNIQUE (
        logical_owner_id, lineage_sequence,
        integration_public_id, account_public_id, provider_source_contact_public_id
    ),
    FOREIGN KEY (logical_owner_id, lineage_sequence)
        REFERENCES makosh_data.persons_lineage(logical_owner_id, lineage_sequence) ON DELETE RESTRICT
);

CREATE TABLE makosh_data.persons_decision_receipts (
    logical_owner_id TEXT NOT NULL,
    decision_id BYTEA NOT NULL CHECK (octet_length(decision_id) = 16),
    action_digest BYTEA NOT NULL CHECK (octet_length(action_digest) = 32),
    review_id BYTEA NOT NULL CHECK (octet_length(review_id) = 16),
    decision_revision BIGINT NOT NULL CHECK (decision_revision > 0),
    decided_by_owner_device_id BYTEA NOT NULL CHECK (octet_length(decided_by_owner_device_id) = 16),
    decided_at_unix_seconds BIGINT NOT NULL CHECK (decided_at_unix_seconds > 0),
    decided_at_nanos INTEGER NOT NULL CHECK (decided_at_nanos BETWEEN 0 AND 999999999),
    PRIMARY KEY (logical_owner_id, decision_id),
    UNIQUE (decision_id),
    FOREIGN KEY (logical_owner_id) REFERENCES makosh_data.persons_owner_aggregates(logical_owner_id)
        ON DELETE RESTRICT
);

CREATE TABLE makosh_data.persons_decision_outcomes (
    logical_owner_id TEXT NOT NULL,
    decision_id BYTEA NOT NULL,
    person_id BYTEA NOT NULL CHECK (octet_length(person_id) = 16),
    resulting_person_revision BIGINT NOT NULL CHECK (resulting_person_revision > 0),
    PRIMARY KEY (logical_owner_id, decision_id, person_id),
    FOREIGN KEY (logical_owner_id, decision_id)
        REFERENCES makosh_data.persons_decision_receipts(logical_owner_id, decision_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (logical_owner_id, person_id)
        REFERENCES makosh_data.persons_current(logical_owner_id, person_id) ON DELETE RESTRICT
);

CREATE TABLE makosh_data.persons_command_inbox (
    logical_owner_id TEXT NOT NULL,
    command_message_id BYTEA NOT NULL CHECK (octet_length(command_message_id) = 16),
    command_envelope_sha256 BYTEA NOT NULL CHECK (octet_length(command_envelope_sha256) = 32),
    command_id BYTEA NOT NULL CHECK (octet_length(command_id) = 16),
    command_fingerprint BYTEA NOT NULL CHECK (octet_length(command_fingerprint) = 32),
    expected_aggregate_revision BIGINT NOT NULL CHECK (expected_aggregate_revision >= 0),
    resulting_aggregate_revision BIGINT CHECK (resulting_aggregate_revision > 0),
    terminal_message_id BYTEA CHECK (terminal_message_id IS NULL OR octet_length(terminal_message_id) = 16),
    terminal_envelope_sha256 BYTEA CHECK (
        terminal_envelope_sha256 IS NULL OR octet_length(terminal_envelope_sha256) = 32
    ),
    terminal_envelope_bytes BYTEA CHECK (
        terminal_envelope_bytes IS NULL OR octet_length(terminal_envelope_bytes) BETWEEN 1 AND 262144
    ),
    received_at_unix_millis BIGINT NOT NULL CHECK (received_at_unix_millis > 0),
    completed_at_unix_millis BIGINT CHECK (completed_at_unix_millis > 0),
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (logical_owner_id, command_message_id),
    UNIQUE (command_id),
    FOREIGN KEY (logical_owner_id) REFERENCES makosh_data.persons_owner_aggregates(logical_owner_id)
        ON DELETE RESTRICT,
    CHECK (
        (NOT completed AND resulting_aggregate_revision IS NULL AND terminal_message_id IS NULL
         AND terminal_envelope_sha256 IS NULL AND terminal_envelope_bytes IS NULL
         AND completed_at_unix_millis IS NULL)
        OR
        (completed AND resulting_aggregate_revision IS NOT NULL AND terminal_message_id IS NOT NULL
         AND terminal_envelope_sha256 IS NOT NULL AND terminal_envelope_bytes IS NOT NULL
         AND completed_at_unix_millis IS NOT NULL)
    ),
    CHECK (completed_at_unix_millis IS NULL OR completed_at_unix_millis >= received_at_unix_millis)
);

CREATE TABLE makosh_data.persons_outbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL CHECK (octet_length(message_id) = 16),
    envelope_sha256 BYTEA NOT NULL CHECK (octet_length(envelope_sha256) = 32),
    envelope_bytes BYTEA NOT NULL CHECK (octet_length(envelope_bytes) BETWEEN 1 AND 262144),
    created_at_unix_millis BIGINT NOT NULL CHECK (created_at_unix_millis > 0),
    published_at_unix_millis BIGINT CHECK (published_at_unix_millis > 0),
    PRIMARY KEY (logical_owner_id, message_id),
    FOREIGN KEY (logical_owner_id) REFERENCES makosh_data.persons_owner_aggregates(logical_owner_id)
        ON DELETE RESTRICT,
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

ALTER TABLE makosh_data.persons_lineage ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_lineage FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_lineage_owner_rls ON makosh_data.persons_lineage
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_lineage_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_lineage_sources FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_lineage_sources_owner_rls ON makosh_data.persons_lineage_sources
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_decision_receipts ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_decision_receipts FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_decision_receipts_owner_rls ON makosh_data.persons_decision_receipts
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_decision_outcomes ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_decision_outcomes FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_decision_outcomes_owner_rls ON makosh_data.persons_decision_outcomes
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_command_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_command_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_command_inbox_owner_rls ON makosh_data.persons_command_inbox
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_outbox_owner_rls ON makosh_data.persons_outbox
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

CREATE TABLE makosh_data.calendar_events (
    logical_owner_id TEXT NOT NULL,
    calendar_event_id BYTEA NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    starts_at_unix_seconds BIGINT NOT NULL,
    starts_at_nanos INTEGER NOT NULL,
    ends_at_unix_seconds BIGINT NOT NULL,
    ends_at_nanos INTEGER NOT NULL,
    timezone TEXT NOT NULL,
    event_state SMALLINT NOT NULL,
    event_revision BIGINT NOT NULL,
    created_at_unix_seconds BIGINT NOT NULL,
    created_at_nanos INTEGER NOT NULL,
    updated_at_unix_seconds BIGINT NOT NULL,
    updated_at_nanos INTEGER NOT NULL,
    PRIMARY KEY (logical_owner_id, calendar_event_id),
    CHECK (length(logical_owner_id) BETWEEN 1 AND 128),
    CHECK (length(calendar_event_id) = 16),
    CHECK (char_length(title) BETWEEN 1 AND 240),
    CHECK (char_length(description) <= 8000),
    CHECK (starts_at_unix_seconds > 0 AND ends_at_unix_seconds > 0),
    CHECK (starts_at_nanos BETWEEN 0 AND 999999999),
    CHECK (ends_at_nanos BETWEEN 0 AND 999999999),
    CHECK (ends_at_unix_seconds > starts_at_unix_seconds OR
        (ends_at_unix_seconds = starts_at_unix_seconds AND ends_at_nanos > starts_at_nanos)),
    CHECK (length(timezone) BETWEEN 1 AND 128),
    CHECK (event_state BETWEEN 1 AND 3),
    CHECK (event_revision > 0),
    CHECK (created_at_unix_seconds > 0),
    CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    CHECK (updated_at_unix_seconds >= created_at_unix_seconds),
    CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.calendar_participants (
    logical_owner_id TEXT NOT NULL,
    calendar_event_id BYTEA NOT NULL,
    participant_id BYTEA NOT NULL,
    display_name TEXT NOT NULL,
    address TEXT NOT NULL,
    participant_role SMALLINT NOT NULL,
    participant_response SMALLINT NOT NULL,
    updated_at_event_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, calendar_event_id, participant_id),
    UNIQUE (logical_owner_id, calendar_event_id, address),
    FOREIGN KEY (logical_owner_id, calendar_event_id)
        REFERENCES makosh_data.calendar_events (logical_owner_id, calendar_event_id) ON DELETE CASCADE,
    CHECK (length(participant_id) = 16),
    CHECK (char_length(display_name) BETWEEN 1 AND 200),
    CHECK (char_length(address) BETWEEN 1 AND 320),
    CHECK (participant_role BETWEEN 1 AND 3),
    CHECK (participant_response BETWEEN 1 AND 4),
    CHECK (updated_at_event_revision > 0)
);

CREATE TABLE makosh_data.calendar_constraints (
    logical_owner_id TEXT NOT NULL,
    calendar_event_id BYTEA NOT NULL,
    earliest_start_unix_seconds BIGINT NOT NULL,
    earliest_start_nanos INTEGER NOT NULL,
    latest_end_unix_seconds BIGINT NOT NULL,
    latest_end_nanos INTEGER NOT NULL,
    minimum_duration_minutes INTEGER NOT NULL,
    timezone TEXT NOT NULL,
    updated_at_event_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, calendar_event_id),
    FOREIGN KEY (logical_owner_id, calendar_event_id)
        REFERENCES makosh_data.calendar_events (logical_owner_id, calendar_event_id) ON DELETE CASCADE,
    CHECK (earliest_start_unix_seconds > 0 AND latest_end_unix_seconds > 0),
    CHECK (earliest_start_nanos BETWEEN 0 AND 999999999),
    CHECK (latest_end_nanos BETWEEN 0 AND 999999999),
    CHECK (minimum_duration_minutes > 0),
    CHECK (length(timezone) BETWEEN 1 AND 128),
    CHECK (updated_at_event_revision > 0)
);

CREATE TABLE makosh_data.calendar_reminders (
    logical_owner_id TEXT NOT NULL,
    calendar_event_id BYTEA NOT NULL,
    reminder_id BYTEA NOT NULL,
    due_at_unix_seconds BIGINT NOT NULL,
    due_at_nanos INTEGER NOT NULL,
    reminder_state SMALLINT NOT NULL,
    updated_at_event_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, calendar_event_id, reminder_id),
    FOREIGN KEY (logical_owner_id, calendar_event_id)
        REFERENCES makosh_data.calendar_events (logical_owner_id, calendar_event_id) ON DELETE CASCADE,
    CHECK (length(reminder_id) = 16),
    CHECK (due_at_unix_seconds > 0),
    CHECK (due_at_nanos BETWEEN 0 AND 999999999),
    CHECK (reminder_state BETWEEN 1 AND 3),
    CHECK (updated_at_event_revision > 0)
);

CREATE TABLE makosh_data.calendar_outcomes (
    logical_owner_id TEXT NOT NULL,
    calendar_event_id BYTEA NOT NULL,
    outcome_id BYTEA NOT NULL,
    outcome_kind SMALLINT NOT NULL,
    note TEXT NOT NULL,
    recorded_at_unix_seconds BIGINT NOT NULL,
    recorded_at_nanos INTEGER NOT NULL,
    recorded_at_event_revision BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, calendar_event_id, outcome_id),
    FOREIGN KEY (logical_owner_id, calendar_event_id)
        REFERENCES makosh_data.calendar_events (logical_owner_id, calendar_event_id) ON DELETE CASCADE,
    CHECK (length(outcome_id) = 16),
    CHECK (outcome_kind BETWEEN 1 AND 3),
    CHECK (char_length(note) <= 2000),
    CHECK (recorded_at_unix_seconds > 0),
    CHECK (recorded_at_nanos BETWEEN 0 AND 999999999),
    CHECK (recorded_at_event_revision > 0)
);

CREATE TABLE makosh_data.calendar_client_operations (
    logical_owner_id TEXT NOT NULL,
    operation_id BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    request_sha256 BYTEA NOT NULL,
    request_bytes BYTEA NOT NULL,
    calendar_event_id BYTEA NOT NULL,
    event_revision BIGINT NOT NULL,
    response_sha256 BYTEA NOT NULL,
    response_bytes BYTEA NOT NULL,
    received_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, operation_id),
    FOREIGN KEY (logical_owner_id, calendar_event_id)
        REFERENCES makosh_data.calendar_events (logical_owner_id, calendar_event_id) ON DELETE CASCADE,
    CHECK (operation_kind BETWEEN 1 AND 10),
    CHECK (length(operation_id) = 16),
    CHECK (length(request_sha256) = 32),
    CHECK (length(request_bytes) BETWEEN 1 AND 65536),
    CHECK (event_revision > 0),
    CHECK (length(response_sha256) = 32),
    CHECK (length(response_bytes) BETWEEN 1 AND 65536),
    CHECK (received_at_unix_millis > 0)
);

CREATE TABLE makosh_data.calendar_scheduler_inbox (
    logical_owner_id TEXT NOT NULL,
    message_id BYTEA NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    operation_kind SMALLINT NOT NULL,
    calendar_event_id BYTEA NOT NULL,
    reminder_id BYTEA,
    completed_at_unix_millis BIGINT NOT NULL,
    PRIMARY KEY (logical_owner_id, message_id),
    CHECK (length(message_id) = 16),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (operation_kind BETWEEN 1 AND 3),
    CHECK (length(calendar_event_id) = 16),
    CHECK (reminder_id IS NULL OR length(reminder_id) = 16),
    CHECK (completed_at_unix_millis > 0)
);

CREATE TABLE makosh_data.calendar_outbox (
    logical_owner_id TEXT NOT NULL,
    outbox_sequence BIGSERIAL NOT NULL,
    message_id BYTEA NOT NULL,
    semantic_kind SMALLINT NOT NULL,
    envelope_sha256 BYTEA NOT NULL,
    envelope_bytes BYTEA NOT NULL,
    created_at_unix_millis BIGINT NOT NULL,
    published_at_unix_millis BIGINT,
    PRIMARY KEY (logical_owner_id, message_id),
    UNIQUE (logical_owner_id, outbox_sequence),
    CHECK (length(message_id) = 16),
    CHECK (semantic_kind BETWEEN 1 AND 4),
    CHECK (length(envelope_sha256) = 32),
    CHECK (length(envelope_bytes) BETWEEN 1 AND 65536),
    CHECK (created_at_unix_millis > 0),
    CHECK (published_at_unix_millis IS NULL OR published_at_unix_millis >= created_at_unix_millis)
);

CREATE INDEX calendar_events_order_idx
ON makosh_data.calendar_events (logical_owner_id, calendar_event_id);
CREATE INDEX calendar_participants_order_idx
ON makosh_data.calendar_participants (logical_owner_id, calendar_event_id, participant_id);
CREATE INDEX calendar_reminders_order_idx
ON makosh_data.calendar_reminders (logical_owner_id, calendar_event_id, reminder_id);
CREATE INDEX calendar_outcomes_order_idx
ON makosh_data.calendar_outcomes (logical_owner_id, calendar_event_id, outcome_id);
CREATE INDEX calendar_outbox_pending_idx
ON makosh_data.calendar_outbox (logical_owner_id, outbox_sequence)
WHERE published_at_unix_millis IS NULL;

ALTER TABLE makosh_data.calendar_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_events FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_events_owner_policy ON makosh_data.calendar_events
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.calendar_participants ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_participants FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_participants_owner_policy ON makosh_data.calendar_participants
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.calendar_constraints ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_constraints FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_constraints_owner_policy ON makosh_data.calendar_constraints
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.calendar_reminders ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_reminders FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_reminders_owner_policy ON makosh_data.calendar_reminders
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.calendar_outcomes ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_outcomes FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_outcomes_owner_policy ON makosh_data.calendar_outcomes
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.calendar_client_operations ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_client_operations FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_client_operations_owner_policy ON makosh_data.calendar_client_operations
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.calendar_scheduler_inbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_scheduler_inbox FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_scheduler_inbox_owner_policy ON makosh_data.calendar_scheduler_inbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.calendar_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.calendar_outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY calendar_outbox_owner_policy ON makosh_data.calendar_outbox
USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

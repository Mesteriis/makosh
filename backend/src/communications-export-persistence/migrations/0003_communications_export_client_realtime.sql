CREATE TABLE makosh_data.communications_export_client_realtime (
    realtime_sequence BIGSERIAL PRIMARY KEY,
    logical_owner_id TEXT NOT NULL,
    export_id BYTEA NOT NULL,
    state SMALLINT NOT NULL,
    requested_items INTEGER NOT NULL,
    completed_items INTEGER NOT NULL,
    artifact_bytes BIGINT NOT NULL,
    rejection_code SMALLINT,
    occurred_at_unix_millis BIGINT NOT NULL,
    UNIQUE (logical_owner_id, export_id, state),
    CHECK (length(export_id) = 16),
    CHECK (state BETWEEN 1 AND 4),
    CHECK (requested_items BETWEEN 1 AND 64),
    CHECK (completed_items BETWEEN 0 AND requested_items),
    CHECK (artifact_bytes BETWEEN 0 AND 25165824),
    CHECK (rejection_code IS NULL OR rejection_code BETWEEN 1 AND 16),
    CHECK (occurred_at_unix_millis > 0)
);

CREATE INDEX communications_export_client_realtime_owner_idx
ON makosh_data.communications_export_client_realtime (
    logical_owner_id,
    realtime_sequence
);

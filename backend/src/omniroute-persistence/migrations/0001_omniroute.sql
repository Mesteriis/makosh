CREATE TABLE IF NOT EXISTS makosh_data.omniroute_runs (
 logical_owner_id TEXT NOT NULL,
 request_id BYTEA NOT NULL,
 contract_name TEXT NOT NULL,
 request_sha256 BYTEA NOT NULL,
 model_receipt_sha256 BYTEA NOT NULL,
 settings_revision BIGINT NOT NULL,
 state SMALLINT NOT NULL,
 terminal_result_bytes BYTEA,
 terminal_result_sha256 BYTEA,
 accepted_at_unix_millis BIGINT NOT NULL,
 completed_at_unix_millis BIGINT,
 PRIMARY KEY(logical_owner_id,request_id),
 CHECK(octet_length(request_id)=16),
 CHECK(octet_length(request_sha256)=32),
 CHECK(octet_length(model_receipt_sha256)=32),
 CHECK(settings_revision>0),
 CHECK(state BETWEEN 1 AND 4),
 CHECK((state IN (1,2) AND terminal_result_bytes IS NULL AND terminal_result_sha256 IS NULL AND completed_at_unix_millis IS NULL) OR (state IN (3,4) AND octet_length(terminal_result_bytes) BETWEEN 1 AND 262144 AND octet_length(terminal_result_sha256)=32 AND completed_at_unix_millis>=accepted_at_unix_millis))
);
ALTER TABLE makosh_data.omniroute_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.omniroute_runs FORCE ROW LEVEL SECURITY;
CREATE POLICY omniroute_runs_owner ON makosh_data.omniroute_runs USING (logical_owner_id=current_setting('makosh.logical_owner_id',true)) WITH CHECK (logical_owner_id=current_setting('makosh.logical_owner_id',true));

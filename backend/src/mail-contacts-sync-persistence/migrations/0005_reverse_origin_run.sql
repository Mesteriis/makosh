ALTER TABLE makosh_data.mail_contacts_sync_reverse_operations
ADD COLUMN origin_run_id BYTEA;

ALTER TABLE makosh_data.mail_contacts_sync_reverse_operations
ADD CONSTRAINT mail_contacts_sync_reverse_origin_run_length
CHECK (origin_run_id IS NULL OR length(origin_run_id) = 16);

CREATE INDEX mail_contacts_sync_reverse_origin_run_idx
ON makosh_data.mail_contacts_sync_reverse_operations (
    logical_owner_id, origin_run_id, state, operation_id
)
WHERE origin_run_id IS NOT NULL;

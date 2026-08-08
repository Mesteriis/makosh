CREATE TABLE IF NOT EXISTS makosh_data.mail_icloud_carddav_credential_bindings (
    connection_id TEXT PRIMARY KEY,
    configuration_instance_id TEXT NOT NULL,
    purpose SMALLINT NOT NULL CHECK (purpose = 5),
    credential_revision BIGINT NOT NULL CHECK (credential_revision > 0),
    binding_revision BIGINT NOT NULL CHECK (binding_revision > 0),
    state SMALLINT NOT NULL CHECK (state IN (2, 3, 4, 5)),
    applied_runtime_generation BIGINT,
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0)
);

CREATE TABLE IF NOT EXISTS makosh_data.mail_icloud_carddav_lifecycle_credentials (
    operation_id TEXT PRIMARY KEY
        REFERENCES makosh_data.mail_account_lifecycle_operations (operation_id),
    purpose SMALLINT NOT NULL CHECK (purpose = 5),
    binding_revision BIGINT,
    credential_revision BIGINT NOT NULL CHECK (credential_revision > 0),
    state SMALLINT NOT NULL CHECK (state IN (1, 2, 3, 4)),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    CHECK (binding_revision IS NULL OR binding_revision > 0)
);

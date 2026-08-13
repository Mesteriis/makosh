CREATE TABLE makosh_data.persons_owner_aggregates (
    logical_owner_id TEXT PRIMARY KEY
        CHECK (octet_length(logical_owner_id) BETWEEN 1 AND 128)
        CHECK (logical_owner_id ~ '^[a-z0-9._-]+$'),
    aggregate_revision BIGINT NOT NULL CHECK (aggregate_revision >= 0),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    updated_at_nanos INTEGER NOT NULL CHECK (updated_at_nanos BETWEEN 0 AND 999999999)
);

CREATE TABLE makosh_data.persons_current (
    logical_owner_id TEXT NOT NULL,
    person_id BYTEA NOT NULL CHECK (octet_length(person_id) = 16),
    lifecycle SMALLINT NOT NULL CHECK (lifecycle BETWEEN 1 AND 4),
    person_revision BIGINT NOT NULL CHECK (person_revision > 0),
    current_profile_revision BIGINT CHECK (current_profile_revision IS NULL OR current_profile_revision > 0),
    merged_into_person_id BYTEA CHECK (
        merged_into_person_id IS NULL OR (
            octet_length(merged_into_person_id) = 16 AND merged_into_person_id <> person_id
        )
    ),
    created_at_unix_seconds BIGINT NOT NULL CHECK (created_at_unix_seconds > 0),
    created_at_nanos INTEGER NOT NULL CHECK (created_at_nanos BETWEEN 0 AND 999999999),
    updated_at_unix_seconds BIGINT NOT NULL CHECK (updated_at_unix_seconds > 0),
    updated_at_nanos INTEGER NOT NULL CHECK (updated_at_nanos BETWEEN 0 AND 999999999),
    PRIMARY KEY (logical_owner_id, person_id),
    FOREIGN KEY (logical_owner_id) REFERENCES makosh_data.persons_owner_aggregates(logical_owner_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (logical_owner_id, merged_into_person_id)
        REFERENCES makosh_data.persons_current(logical_owner_id, person_id)
        ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
    CHECK ((lifecycle = 3) = (merged_into_person_id IS NOT NULL)),
    CHECK ((updated_at_unix_seconds, updated_at_nanos) >= (created_at_unix_seconds, created_at_nanos))
);

CREATE TABLE makosh_data.persons_profiles (
    logical_owner_id TEXT NOT NULL,
    person_id BYTEA NOT NULL,
    display_name TEXT CHECK (display_name IS NULL OR char_length(display_name) BETWEEN 1 AND 240),
    given_name TEXT CHECK (given_name IS NULL OR char_length(given_name) BETWEEN 1 AND 240),
    family_name TEXT CHECK (family_name IS NULL OR char_length(family_name) BETWEEN 1 AND 240),
    normalized_emails TEXT[] NOT NULL DEFAULT '{}'
        CHECK (cardinality(normalized_emails) <= 32)
        CHECK (array_position(normalized_emails, NULL) IS NULL)
        CHECK (octet_length(array_to_string(normalized_emails, '')) <= 10240),
    normalized_phones TEXT[] NOT NULL DEFAULT '{}'
        CHECK (cardinality(normalized_phones) <= 32)
        CHECK (array_position(normalized_phones, NULL) IS NULL)
        CHECK (octet_length(array_to_string(normalized_phones, '')) <= 512),
    profile_revision BIGINT NOT NULL CHECK (profile_revision > 0),
    PRIMARY KEY (logical_owner_id, person_id, profile_revision),
    FOREIGN KEY (logical_owner_id, person_id)
        REFERENCES makosh_data.persons_current(logical_owner_id, person_id) ON DELETE RESTRICT,
    CHECK (
        display_name IS NOT NULL OR given_name IS NOT NULL OR family_name IS NOT NULL
        OR cardinality(normalized_emails) > 0 OR cardinality(normalized_phones) > 0
    )
);

CREATE FUNCTION makosh_data.persons_reject_profile_history_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'persons profile history is immutable' USING ERRCODE = '55000';
END;
$$;

CREATE TRIGGER persons_profiles_immutable
BEFORE UPDATE OR DELETE ON makosh_data.persons_profiles
FOR EACH ROW EXECUTE FUNCTION makosh_data.persons_reject_profile_history_mutation();

ALTER TABLE makosh_data.persons_current
    ADD CONSTRAINT persons_current_profile_revision_fk
    FOREIGN KEY (logical_owner_id, person_id, current_profile_revision)
    REFERENCES makosh_data.persons_profiles(logical_owner_id, person_id, profile_revision)
    ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE makosh_data.persons_sources (
    logical_owner_id TEXT NOT NULL,
    integration_public_id BYTEA NOT NULL CHECK (octet_length(integration_public_id) = 16),
    account_public_id BYTEA NOT NULL CHECK (octet_length(account_public_id) = 16),
    provider_source_contact_public_id BYTEA NOT NULL
        CHECK (octet_length(provider_source_contact_public_id) = 16),
    person_id BYTEA CHECK (person_id IS NULL OR octet_length(person_id) = 16),
    removed BOOLEAN NOT NULL,
    display_name TEXT CHECK (display_name IS NULL OR char_length(display_name) BETWEEN 1 AND 240),
    normalized_emails TEXT[] NOT NULL DEFAULT '{}'
        CHECK (cardinality(normalized_emails) <= 32)
        CHECK (array_position(normalized_emails, NULL) IS NULL)
        CHECK (octet_length(array_to_string(normalized_emails, '')) <= 10240),
    normalized_phones TEXT[] NOT NULL DEFAULT '{}'
        CHECK (cardinality(normalized_phones) <= 32)
        CHECK (array_position(normalized_phones, NULL) IS NULL)
        CHECK (octet_length(array_to_string(normalized_phones, '')) <= 512),
    source_revision BIGINT NOT NULL CHECK (source_revision > 0),
    source_digest BYTEA NOT NULL CHECK (octet_length(source_digest) = 32),
    observed_at_unix_seconds BIGINT NOT NULL CHECK (observed_at_unix_seconds > 0),
    observed_at_nanos INTEGER NOT NULL CHECK (observed_at_nanos BETWEEN 0 AND 999999999),
    last_decision_id BYTEA CHECK (last_decision_id IS NULL OR octet_length(last_decision_id) = 16),
    last_review_id BYTEA CHECK (last_review_id IS NULL OR octet_length(last_review_id) = 16),
    last_decision_revision BIGINT CHECK (last_decision_revision IS NULL OR last_decision_revision > 0),
    last_decided_by_owner_device_id BYTEA CHECK (
        last_decided_by_owner_device_id IS NULL
        OR octet_length(last_decided_by_owner_device_id) = 16
    ),
    last_decided_at_unix_seconds BIGINT CHECK (
        last_decided_at_unix_seconds IS NULL OR last_decided_at_unix_seconds > 0
    ),
    last_decided_at_nanos INTEGER CHECK (
        last_decided_at_nanos IS NULL OR last_decided_at_nanos BETWEEN 0 AND 999999999
    ),
    last_approved_action_digest BYTEA CHECK (
        last_approved_action_digest IS NULL OR octet_length(last_approved_action_digest) = 32
    ),
    PRIMARY KEY (
        logical_owner_id,
        integration_public_id,
        account_public_id,
        provider_source_contact_public_id
    ),
    UNIQUE (integration_public_id, account_public_id, provider_source_contact_public_id),
    FOREIGN KEY (logical_owner_id) REFERENCES makosh_data.persons_owner_aggregates(logical_owner_id)
        ON DELETE RESTRICT,
    FOREIGN KEY (logical_owner_id, person_id)
        REFERENCES makosh_data.persons_current(logical_owner_id, person_id) ON DELETE RESTRICT,
    CHECK (removed = (person_id IS NULL)),
    CHECK (
        removed OR display_name IS NOT NULL
        OR cardinality(normalized_emails) > 0 OR cardinality(normalized_phones) > 0
    ),
    CHECK (
        (last_decision_id IS NULL AND last_review_id IS NULL AND last_decision_revision IS NULL
         AND last_decided_by_owner_device_id IS NULL AND last_decided_at_unix_seconds IS NULL
         AND last_decided_at_nanos IS NULL AND last_approved_action_digest IS NULL)
        OR
        (last_decision_id IS NOT NULL AND last_review_id IS NOT NULL
         AND last_decision_revision IS NOT NULL AND last_decided_by_owner_device_id IS NOT NULL
         AND last_decided_at_unix_seconds IS NOT NULL AND last_decided_at_nanos IS NOT NULL
         AND last_approved_action_digest IS NOT NULL)
    )
);

ALTER TABLE makosh_data.persons_owner_aggregates ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_owner_aggregates FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_owner_aggregates_owner_rls ON makosh_data.persons_owner_aggregates
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_current ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_current FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_current_owner_rls ON makosh_data.persons_current
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_profiles FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_profiles_owner_rls ON makosh_data.persons_profiles
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

ALTER TABLE makosh_data.persons_sources ENABLE ROW LEVEL SECURITY;
ALTER TABLE makosh_data.persons_sources FORCE ROW LEVEL SECURITY;
CREATE POLICY persons_sources_owner_rls ON makosh_data.persons_sources
    USING (logical_owner_id = current_setting('makosh.logical_owner_id', true))
    WITH CHECK (logical_owner_id = current_setting('makosh.logical_owner_id', true));

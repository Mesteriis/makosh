use makosh_storage_migrations::{MigrationAdmissionErrorV1, admit_owner_local_additive_sql};

#[test]
fn admits_owner_local_create_table() {
    let result = admit_owner_local_additive_sql(
        "notes",
        "CREATE TABLE makosh_data.notes_entries (entry_id uuid);",
    );
    assert_eq!(result, Ok(()));
}

#[test]
fn admits_canonical_owner_identifier_with_digits() {
    let result = admit_owner_local_additive_sql(
        "storage_runtime_revoke_20260717",
        "CREATE TABLE makosh_data.storage_runtime_revoke_20260717_entries (entry_id uuid);",
    );
    assert_eq!(result, Ok(()));
}

#[test]
fn admits_owner_local_add_column() {
    let result = admit_owner_local_additive_sql(
        "notes",
        "ALTER TABLE makosh_data.notes_entries ADD COLUMN title text;",
    );
    assert_eq!(result, Ok(()));
}

#[test]
fn admits_owner_local_simple_index_and_check_constraint() {
    let index = admit_owner_local_additive_sql(
        "notes",
        "CREATE INDEX notes_entries_lookup ON makosh_data.notes_entries (entry_id) WHERE entry_id IS NOT NULL;",
    );
    let constraint = admit_owner_local_additive_sql(
        "notes",
        "ALTER TABLE makosh_data.notes_entries ADD CONSTRAINT notes_entries_shape CHECK (entry_id IS NOT NULL);",
    );
    assert_eq!(index, Ok(()));
    assert_eq!(constraint, Ok(()));
}

#[test]
fn admits_the_exact_scheduler_platform_schema() {
    let result = admit_owner_local_additive_sql(
        "scheduler",
        "CREATE TABLE makosh_platform.scheduler_schedules (schedule_id bytea PRIMARY KEY);",
    );
    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_non_migration_statement() {
    let result = admit_owner_local_additive_sql("notes", "SELECT 1;");
    assert_eq!(result, Err(MigrationAdmissionErrorV1::Forbidden));
}

#[test]
fn rejects_wrong_schema_and_non_additive_alteration() {
    let wrong_schema = admit_owner_local_additive_sql(
        "notes",
        "CREATE TABLE public.notes_entries (entry_id uuid);",
    );
    let drop_column = admit_owner_local_additive_sql(
        "notes",
        "ALTER TABLE makosh_data.notes_entries DROP COLUMN title;",
    );
    let foreign_key = admit_owner_local_additive_sql(
        "notes",
        "ALTER TABLE makosh_data.notes_entries ADD CONSTRAINT notes_entries_foreign FOREIGN KEY (entry_id) REFERENCES makosh_data.other_entries (entry_id);",
    );
    assert_eq!(wrong_schema, Err(MigrationAdmissionErrorV1::Forbidden));
    assert_eq!(drop_column, Err(MigrationAdmissionErrorV1::Forbidden));
    assert_eq!(foreign_key, Err(MigrationAdmissionErrorV1::Forbidden));
}

//! Filesystem conformance for the Storage-owned PgBouncer database include.

use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

use makosh_storage_pgbouncer::{
    PLATFORM_ADMIN_USERNAME, PgBouncerAuthEntryV1, PgBouncerAuthFileV1,
    PgBouncerDatabaseConfigFileV1, PgBouncerRuntimeConfigV1, PoolAliasV1, PoolConfigErrorV1,
};
use tempfile::TempDir;
use zeroize::Zeroizing;

#[test]
fn replaces_a_private_database_include_in_stable_alias_order() {
    let directory = private_directory();
    let path = directory.path().join("databases.ini");

    let file = PgBouncerDatabaseConfigFileV1::replace(&path, &[config("notes"), config("mail")])
        .expect("replace private PgBouncer include");

    assert_eq!(file.path(), path);
    let metadata = std::fs::metadata(&path).expect("database include metadata");
    assert_eq!(metadata.mode() & 0o777, 0o600);
    assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
    assert!(!directory.path().join(".databases.ini.next").exists());
    assert_eq!(
        std::fs::read_to_string(path).expect("database include"),
        "[databases]\nruntime_registration_mail_1 = host=127.0.0.1 port=5432 dbname=makosh pool_mode=transaction max_db_client_connections=8\nruntime_registration_notes_1 = host=127.0.0.1 port=5432 dbname=makosh pool_mode=transaction max_db_client_connections=8\n"
    );
}

#[test]
fn refuses_untrusted_paths_and_duplicate_aliases() {
    let directory = private_directory();
    let target = directory.path().join("target.ini");
    let alias = directory.path().join("databases.ini");
    symlink(&target, &alias).expect("test symlink");
    assert!(matches!(
        PgBouncerDatabaseConfigFileV1::replace(&alias, &[config("notes")]),
        Err(PoolConfigErrorV1::FileSystem)
    ));

    let duplicate = directory.path().join("duplicate.ini");
    assert!(matches!(
        PgBouncerDatabaseConfigFileV1::replace(&duplicate, &[config("notes"), config("notes")]),
        Err(PoolConfigErrorV1::Identifier)
    ));
}

#[test]
fn replaces_a_private_auth_file_with_pooler_admin_and_scram_verifier() {
    let directory = private_directory();
    let path = directory.path().join("users.txt");
    let admin = Zeroizing::new(b"0123456789abcdef".to_vec());
    let verifier =
        Zeroizing::new("SCRAM-SHA-256$4096:c2FsdA==$c3RvcmVka2V5$c2VydmVya2V5".to_owned());

    let file = PgBouncerAuthFileV1::replace(
        &path,
        vec![
            PgBouncerAuthEntryV1::runtime_scram("runtime_mail", verifier)
                .expect("runtime SCRAM entry"),
            PgBouncerAuthEntryV1::pooler_admin(PLATFORM_ADMIN_USERNAME, &admin)
                .expect("pooler admin entry"),
        ],
    )
    .expect("replace private PgBouncer auth file");

    assert_eq!(file.path(), path);
    let metadata = std::fs::metadata(&path).expect("auth file metadata");
    assert_eq!(metadata.mode() & 0o777, 0o600);
    assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
    assert_eq!(
        std::fs::read_to_string(path).expect("auth file"),
        "\"makosh_pgbouncer_admin\" \"0123456789abcdef\"\n\"runtime_mail\" \"SCRAM-SHA-256$4096:c2FsdA==$c3RvcmVka2V5$c2VydmVya2V5\"\n"
    );
}

#[test]
fn refuses_an_invalid_runtime_scram_verifier() {
    assert!(matches!(
        PgBouncerAuthEntryV1::runtime_scram(
            "runtime_mail",
            Zeroizing::new("not-a-verifier".to_owned())
        ),
        Err(PoolConfigErrorV1::Identifier)
    ));
}

fn config(owner: &str) -> PgBouncerRuntimeConfigV1 {
    let alias = PoolAliasV1::new(&format!("registration_{owner}"), 1).expect("pool alias");
    PgBouncerRuntimeConfigV1::new(
        alias,
        "127.0.0.1".to_owned(),
        5432,
        "makosh".to_owned(),
        format!("runtime_{owner}"),
        8,
    )
    .expect("pool config")
}

fn private_directory() -> TempDir {
    let directory = TempDir::new().expect("private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private mode");
    directory
}

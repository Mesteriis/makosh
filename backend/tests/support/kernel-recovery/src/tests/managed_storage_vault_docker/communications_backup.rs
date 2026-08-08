//! Disposable Docker evidence that Storage-owned offline recovery restores
//! canonical Communications PostgreSQL state after destructive instance reset.

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use super::{required, storage_binary};

pub(super) fn assert_communications_storage_backup_restore(root: &Path) {
    const DATABASE: &str = "makosh_storage_authenticated";
    let backup_root = root.join("communications-postgres-backup");
    fs::create_dir(&backup_root).expect("create Communications backup directory");
    fs::set_permissions(&backup_root, fs::Permissions::from_mode(0o700))
        .expect("protect Communications backup directory");
    let tools = PostgresRecoveryTools::install(root, DATABASE);

    run_storage_recovery(
        "export-backup",
        [
            ("--pg-dump", tools.pg_dump.as_path()),
            (
                "--host",
                Path::new(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST")),
            ),
            (
                "--port",
                Path::new(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")),
            ),
            ("--database", Path::new(DATABASE)),
            ("--username", Path::new("makosh_postgres_admin")),
            ("--ssl-mode", Path::new("disable")),
            (
                "--password-file",
                Path::new(&required(
                    "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
                )),
            ),
            (
                "--output",
                backup_root.join("communications.dump").as_path(),
            ),
        ],
    );
    reset_disposable_database(&tools.container, DATABASE);
    let absent = postgres_command(
        &tools.container,
        DATABASE,
        "SELECT to_regclass('makosh_data.communications_evidence_summaries') IS NULL",
    );
    assert_eq!(
        absent.trim(),
        "t",
        "destructive reset must remove Communications state"
    );
    run_storage_recovery(
        "restore-backup",
        [
            ("--pg-restore", tools.pg_restore.as_path()),
            ("--psql", tools.psql.as_path()),
            (
                "--host",
                Path::new(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST")),
            ),
            (
                "--port",
                Path::new(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")),
            ),
            ("--database", Path::new(DATABASE)),
            ("--username", Path::new("makosh_postgres_admin")),
            ("--ssl-mode", Path::new("disable")),
            (
                "--password-file",
                Path::new(&required(
                    "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
                )),
            ),
            ("--input", backup_root.join("communications.dump").as_path()),
        ],
    );
    let restored = postgres_command(
        &tools.container,
        DATABASE,
        "SELECT count(*) > 0 FROM makosh_data.communications_evidence_summaries",
    );
    assert_eq!(
        restored.trim(),
        "t",
        "restored PostgreSQL must retain canonical Communications evidence"
    );
}

struct PostgresRecoveryTools {
    container: String,
    pg_dump: PathBuf,
    pg_restore: PathBuf,
    psql: PathBuf,
}

impl PostgresRecoveryTools {
    fn install(root: &Path, target_database: &str) -> Self {
        let container = required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_CONTAINER");
        assert!(container.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert!(
            target_database
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        );
        let directory = root.join("postgres-recovery-tools");
        fs::create_dir(&directory).expect("create PostgreSQL recovery tool directory");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .expect("protect PostgreSQL recovery tool directory");
        let pg_dump = write_recovery_tool(
            &directory,
            "pg_dump",
            &format!(
                "#!/bin/sh\nset -eu\noutput=\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --file) output=$2; shift 2 ;;\n    *) shift ;;\n  esac\ndone\n[ -n \"$output\" ]\nexec docker exec {container} sh -ceu 'export PGPASSWORD=\"$(cat /run/secrets/storage_postgres_admin_password)\"; exec pg_dump --username=makosh_postgres_admin --format=custom --no-owner --no-privileges --dbname=makosh_storage_authenticated' > \"$output\"\n"
            ),
        );
        let pg_restore = write_recovery_tool(
            &directory,
            "pg_restore",
            &format!(
                "#!/bin/sh\nset -eu\ninput=\nfor value in \"$@\"; do input=$value; done\n[ -n \"$input\" ]\ndocker cp \"$input\" {container}:/tmp/makosh-communications-recovery.dump\nexec docker exec {container} sh -ceu 'export PGPASSWORD=\"$(cat /run/secrets/storage_postgres_admin_password)\"; exec pg_restore --username=makosh_postgres_admin --no-owner --no-privileges --exit-on-error --single-transaction --dbname={target_database} /tmp/makosh-communications-recovery.dump'\n"
            ),
        );
        let psql = write_recovery_tool(
            &directory,
            "psql",
            &format!(
                "#!/bin/sh\nset -eu\nquery=\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --command) query=$2; shift 2 ;;\n    *) shift ;;\n  esac\ndone\n[ -n \"$query\" ]\nexec docker exec {container} sh -ceu 'export PGPASSWORD=\"$(cat /run/secrets/storage_postgres_admin_password)\"; exec psql --username=makosh_postgres_admin --tuples-only --no-align --dbname={target_database} --command \"$1\"' -- \"$query\"\n"
            ),
        );
        Self {
            container,
            pg_dump,
            pg_restore,
            psql,
        }
    }
}

fn write_recovery_tool(directory: &Path, name: &str, contents: &str) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, contents).expect("write disposable PostgreSQL recovery tool");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
        .expect("make disposable PostgreSQL recovery tool executable");
    path
}

fn run_storage_recovery<const N: usize>(command: &str, arguments: [(&str, &Path); N]) {
    let mut invocation = Command::new(storage_binary());
    invocation.arg(command);
    for (name, value) in arguments {
        invocation.arg(name).arg(value);
    }
    let output = invocation.output().expect("start Storage offline recovery");
    assert!(
        output.status.success(),
        "Storage offline recovery {command} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
}

fn postgres_command(container: &str, database: &str, query: &str) -> String {
    let output = Command::new("docker")
        .args([
            "exec", container, "sh", "-ceu",
            "export PGPASSWORD=\"$(cat /run/secrets/storage_postgres_admin_password)\"; exec psql --username=makosh_postgres_admin --tuples-only --no-align --dbname=\"$1\" --command \"$2\"",
            "--", database, query,
        ])
        .output()
        .expect("start disposable PostgreSQL command");
    assert!(
        output.status.success(),
        "disposable PostgreSQL command failed"
    );
    String::from_utf8(output.stdout).expect("PostgreSQL output is UTF-8")
}

fn reset_disposable_database(container: &str, database: &str) {
    assert!(
        database
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    );
    postgres_command(
        container,
        "postgres",
        &format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{database}' AND pid <> pg_backend_pid()"
        ),
    );
    postgres_command(container, "postgres", &format!("DROP DATABASE {database}"));
    postgres_command(
        container,
        "postgres",
        &format!("CREATE DATABASE {database}"),
    );
}

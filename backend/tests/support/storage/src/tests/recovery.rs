//! Regression coverage for the component-owned PostgreSQL recovery port.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::cli::parse_offline_recovery_command;

const PASSWORD: &str = "owner:secret\\recovery";

#[test]
fn storage_export_publishes_a_private_dump_without_secret_argv_or_environment() {
    let fixture = RecoveryFixture::new();
    let dump = fixture.root().join("postgres.dump");
    let trace = fixture.root().join("pg-dump.trace");
    let pg_dump = fixture.executable(
        "pg-dump",
        &format!(
            "trace={}\n: > \"$trace\"\nprintf 'PATH=%s\\n' \"$PATH\" >> \"$trace\"\nprintf 'PGSERVICEFILE=%s\\n' \"$PGSERVICEFILE\" >> \"$trace\"\nprintf 'PGPASSFILE=%s\\n' \"$PGPASSFILE\" >> \"$trace\"\nprevious=\nfor argument do\n  printf 'ARG=%s\\n' \"$argument\" >> \"$trace\"\n  if [ \"$previous\" = '--file' ]; then printf 'verified-custom-dump' > \"$argument\"; fi\n  previous=$argument\ndone\n",
            shell_quote(&trace)
        ),
    );
    let command = parse_command("export-backup", export_arguments(&fixture, &pg_dump, &dump));

    crate::recovery::execute(command).expect("export PostgreSQL backup");

    assert_eq!(fs::read(&dump).expect("read dump"), b"verified-custom-dump");
    assert_eq!(
        fs::metadata(&dump)
            .expect("dump metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let trace = fs::read_to_string(trace).expect("read pg_dump trace");
    assert!(trace.contains("PGSERVICEFILE="));
    assert!(trace.contains("PGPASSFILE="));
    assert!(trace.contains("ARG=--format=custom"));
    assert!(trace.contains("ARG=--no-owner"));
    assert!(trace.contains("ARG=--no-privileges"));
    assert!(!trace.contains(PASSWORD));
    fixture.assert_no_staging();
}

#[test]
fn storage_restore_requires_an_empty_target_and_never_invokes_pg_restore_on_rejection() {
    let fixture = RecoveryFixture::new();
    let input = fixture.private_file("postgres.dump", b"verified-custom-dump");
    let pg_restore_marker = fixture.root().join("pg-restore.invoked");
    let pg_restore = fixture.executable(
        "pg-restore",
        &format!("printf invoked > {}\n", shell_quote(&pg_restore_marker)),
    );
    let psql = fixture.executable("psql", "printf 'f\\n'\n");
    let command = parse_command(
        "restore-backup",
        restore_arguments(&fixture, &pg_restore, &psql, &input),
    );

    let error = crate::recovery::execute(command).expect_err("reject non-empty target");

    assert_eq!(error, "Storage restore target is invalid");
    assert!(!pg_restore_marker.exists());
    fixture.assert_no_staging();
}

#[test]
fn storage_restore_checks_the_migration_ledger_after_transactional_restore() {
    let fixture = RecoveryFixture::new();
    let input = fixture.private_file("postgres.dump", b"verified-custom-dump");
    let restore_trace = fixture.root().join("pg-restore.trace");
    let psql_trace = fixture.root().join("psql.trace");
    let pg_restore = fixture.executable(
        "pg-restore",
        &format!(
            "trace={}\n: > \"$trace\"\nfor argument do printf 'ARG=%s\\n' \"$argument\" >> \"$trace\"; done\n",
            shell_quote(&restore_trace)
        ),
    );
    let psql = fixture.executable(
        "psql",
        &format!(
            "trace={}\nfor argument do printf 'ARG=%s\\n' \"$argument\" >> \"$trace\"; done\nprintf 't\\n'\n",
            shell_quote(&psql_trace)
        ),
    );
    let command = parse_command(
        "restore-backup",
        restore_arguments(&fixture, &pg_restore, &psql, &input),
    );

    crate::recovery::execute(command).expect("restore PostgreSQL backup");

    let restore_trace = fs::read_to_string(restore_trace).expect("read restore trace");
    assert!(restore_trace.contains("ARG=--single-transaction"));
    assert!(restore_trace.contains("ARG=--exit-on-error"));
    assert!(!restore_trace.contains(PASSWORD));
    let psql_trace = fs::read_to_string(psql_trace).expect("read psql trace");
    assert!(psql_trace.contains("storage_migration_ledger"));
    assert!(!psql_trace.contains(PASSWORD));
    fixture.assert_no_staging();
}

#[test]
fn storage_recovery_cli_rejects_relative_paths_and_invalid_connection_tokens() {
    let fixture = RecoveryFixture::new();
    let mut relative = export_arguments(
        &fixture,
        Path::new("relative-pg-dump"),
        &fixture.root().join("output.dump"),
    )
    .into_iter()
    .peekable();
    assert!(parse_offline_recovery_command(OsStr::new("export-backup"), &mut relative).is_err());

    let pg_dump = fixture.private_file("pg-dump", b"binary");
    let mut invalid_host =
        export_arguments(&fixture, &pg_dump, &fixture.root().join("output.dump"));
    invalid_host[3] = OsString::from("host\npassword");
    let mut invalid_host = invalid_host.into_iter().peekable();
    assert!(
        parse_offline_recovery_command(OsStr::new("export-backup"), &mut invalid_host).is_err()
    );
}

fn parse_command(name: &str, arguments: Vec<OsString>) -> crate::cli::OfflineRecoveryCommand {
    parse_offline_recovery_command(OsStr::new(name), &mut arguments.into_iter().peekable())
        .expect("parse recovery command")
}

fn export_arguments(fixture: &RecoveryFixture, pg_dump: &Path, output: &Path) -> Vec<OsString> {
    let mut arguments = vec![OsString::from("--pg-dump"), pg_dump.as_os_str().to_owned()];
    arguments.extend(fixture.connection_arguments());
    arguments.extend([OsString::from("--output"), output.as_os_str().to_owned()]);
    arguments
}

fn restore_arguments(
    fixture: &RecoveryFixture,
    pg_restore: &Path,
    psql: &Path,
    input: &Path,
) -> Vec<OsString> {
    let mut arguments = vec![
        OsString::from("--pg-restore"),
        pg_restore.as_os_str().to_owned(),
        OsString::from("--psql"),
        psql.as_os_str().to_owned(),
    ];
    arguments.extend(fixture.connection_arguments());
    arguments.extend([OsString::from("--input"), input.as_os_str().to_owned()]);
    arguments
}

struct RecoveryFixture {
    root: TempDir,
    password_file: PathBuf,
}

impl RecoveryFixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("create fixture root");
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700))
            .expect("make fixture root private");
        let password_file = root.path().join("postgres.password");
        write_private(&password_file, format!("{PASSWORD}\n").as_bytes());
        Self {
            root,
            password_file,
        }
    }

    fn root(&self) -> &Path {
        self.root.path()
    }

    fn connection_arguments(&self) -> Vec<OsString> {
        [
            ("--host", OsString::from("127.0.0.1")),
            ("--port", OsString::from("5432")),
            ("--database", OsString::from("makosh")),
            ("--username", OsString::from("makosh_owner")),
            ("--ssl-mode", OsString::from("require")),
            ("--password-file", self.password_file.as_os_str().to_owned()),
        ]
        .into_iter()
        .flat_map(|(name, value)| [OsString::from(name), value])
        .collect()
    }

    fn executable(&self, name: &str, body: &str) -> PathBuf {
        let path = self.root().join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}")).expect("write fake executable");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .expect("make fake executable private");
        path
    }

    fn private_file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.root().join(name);
        write_private(&path, bytes);
        path
    }

    fn assert_no_staging(&self) {
        let has_staging = fs::read_dir(self.root())
            .expect("read fixture root")
            .filter_map(Result::ok)
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".makosh-postgres-")
            });
        assert!(!has_staging, "recovery staging must be removed");
    }
}

fn write_private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("write private fixture");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("make fixture private");
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

//! Component-owned offline PostgreSQL recovery operations.

mod private_files;

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::cli::{
    ExportPostgresBackupArguments, OfflineRecoveryCommand, RestorePostgresBackupArguments,
};
use private_files::{PrivateConnectionFiles, validate_executable, validate_private_file};

const EMPTY_TARGET_QUERY: &str = "SELECT count(*) = 0 FROM pg_catalog.pg_tables WHERE schemaname NOT IN ('pg_catalog', 'information_schema')";
const MIGRATION_LEDGER_QUERY: &str = "SELECT count(*) = 1 FROM information_schema.tables WHERE table_schema = 'makosh_platform' AND table_name = 'storage_migration_ledger'";

pub(crate) fn execute(command: OfflineRecoveryCommand) -> Result<(), String> {
    match command {
        OfflineRecoveryCommand::Export(arguments) => export_backup(&arguments),
        OfflineRecoveryCommand::Restore(arguments) => restore_backup(&arguments),
    }
}

pub(crate) fn export_backup(arguments: &ExportPostgresBackupArguments) -> Result<(), String> {
    validate_executable(&arguments.pg_dump, "pg_dump")?;
    let parent = prepare_absent_output(&arguments.output)?;
    let connection = PrivateConnectionFiles::create(&parent, &arguments.connection, "backup")?;
    let staged_dump = connection.directory().join("postgres.dump");
    let output = run_pg_dump(arguments, &connection, &staged_dump)?;
    require_success(output, "pg_dump")?;
    make_private_and_sync(&staged_dump)?;
    publish_dump(&staged_dump, &arguments.output, &parent)
}

pub(crate) fn restore_backup(arguments: &RestorePostgresBackupArguments) -> Result<(), String> {
    validate_executable(&arguments.pg_restore, "pg_restore")?;
    validate_executable(&arguments.psql, "psql")?;
    validate_private_file(&arguments.input, 64 * 1024 * 1024 * 1024, "restore input")?;
    let parent = arguments
        .input
        .parent()
        .ok_or_else(recovery_unavailable)?
        .to_path_buf();
    let connection = PrivateConnectionFiles::create(&parent, &arguments.connection, "restore")?;
    require_true_query(
        &arguments.psql,
        &connection,
        EMPTY_TARGET_QUERY,
        "restore target",
    )?;
    let output = run_pg_restore(arguments, &connection)?;
    require_success(output, "pg_restore")?;
    require_true_query(
        &arguments.psql,
        &connection,
        MIGRATION_LEDGER_QUERY,
        "restored migration ledger",
    )
}

fn run_pg_dump(
    arguments: &ExportPostgresBackupArguments,
    connection: &PrivateConnectionFiles,
    staged_dump: &Path,
) -> Result<Output, String> {
    let mut command = private_command(&arguments.pg_dump, connection);
    command.args([
        OsStr::new("--format=custom"),
        OsStr::new("--no-owner"),
        OsStr::new("--no-privileges"),
        OsStr::new("--file"),
        staged_dump.as_os_str(),
        OsStr::new("--dbname=service=makosh_recovery"),
    ]);
    command.output().map_err(|_| recovery_unavailable())
}

fn run_pg_restore(
    arguments: &RestorePostgresBackupArguments,
    connection: &PrivateConnectionFiles,
) -> Result<Output, String> {
    let mut command = private_command(&arguments.pg_restore, connection);
    command.args([
        OsStr::new("--no-owner"),
        OsStr::new("--no-privileges"),
        OsStr::new("--exit-on-error"),
        OsStr::new("--single-transaction"),
        OsStr::new("--dbname=service=makosh_recovery"),
        arguments.input.as_os_str(),
    ]);
    command.output().map_err(|_| recovery_unavailable())
}

fn require_true_query(
    psql: &Path,
    connection: &PrivateConnectionFiles,
    query: &str,
    label: &str,
) -> Result<(), String> {
    let mut command = private_command(psql, connection);
    command.args([
        "--tuples-only",
        "--no-align",
        "--dbname=service=makosh_recovery",
        "--command",
        query,
    ]);
    let output = command.output().map_err(|_| recovery_unavailable())?;
    require_success(output, "psql").and_then(|stdout| {
        (stdout.len() <= 16 && stdout.as_slice() == b"t")
            .then_some(())
            .ok_or_else(|| format!("Storage {label} is invalid"))
    })
}

fn private_command(binary: &Path, connection: &PrivateConnectionFiles) -> Command {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("HOME", connection.directory())
        .env("LC_ALL", "C")
        .env("PGPASSFILE", connection.password_path())
        .env("PGSERVICEFILE", connection.service_path());
    command
}

fn require_success(output: Output, label: &str) -> Result<Vec<u8>, String> {
    if !output.status.success() {
        return Err(format!("Storage {label} recovery command failed"));
    }
    let mut stdout = output.stdout;
    while stdout.last().is_some_and(u8::is_ascii_whitespace) {
        let _ = stdout.pop();
    }
    Ok(stdout)
}

fn prepare_absent_output(output: &Path) -> Result<PathBuf, String> {
    if !output.is_absolute() || fs::symlink_metadata(output).is_ok() {
        return Err("Storage backup output is invalid".to_owned());
    }
    let parent = output
        .parent()
        .ok_or_else(recovery_unavailable)?
        .to_path_buf();
    private_files::validate_private_directory(&parent, "backup output directory")?;
    Ok(parent)
}

fn make_private_and_sync(path: &Path) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|_| recovery_unavailable())?;
    validate_private_file(path, 64 * 1024 * 1024 * 1024, "PostgreSQL dump")?;
    fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| recovery_unavailable())
}

fn publish_dump(staged: &Path, output: &Path, parent: &Path) -> Result<(), String> {
    fs::hard_link(staged, output).map_err(|_| recovery_unavailable())?;
    if sync_directory(parent).is_ok() {
        return Ok(());
    }
    let _ = fs::remove_file(output);
    let _ = sync_directory(parent);
    Err(recovery_unavailable())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| recovery_unavailable())
}

fn recovery_unavailable() -> String {
    "Storage offline recovery is unavailable".to_owned()
}

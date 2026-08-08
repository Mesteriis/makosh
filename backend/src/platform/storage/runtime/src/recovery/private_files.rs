//! Private filesystem and libpq credential staging for recovery commands.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zeroize::{Zeroize, Zeroizing};

use crate::cli::PostgresConnectionArguments;

static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) struct PrivateConnectionFiles {
    directory: PathBuf,
    service_path: PathBuf,
    password_path: PathBuf,
}

impl PrivateConnectionFiles {
    pub(super) fn create(
        parent: &Path,
        connection: &PostgresConnectionArguments,
        purpose: &str,
    ) -> Result<Self, String> {
        validate_private_directory(parent, "recovery working directory")?;
        let directory = create_staging(parent, purpose)?;
        let service_path = directory.join("pg_service.conf");
        let password_path = directory.join("pgpass");
        let result = write_connection_files(connection, &service_path, &password_path);
        if result.is_err() {
            let _ = fs::remove_dir_all(&directory);
        }
        result.map(|()| Self {
            directory,
            service_path,
            password_path,
        })
    }

    pub(super) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(super) fn service_path(&self) -> &Path {
        &self.service_path
    }

    pub(super) fn password_path(&self) -> &Path {
        &self.password_path
    }
}

impl Drop for PrivateConnectionFiles {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

pub(super) fn validate_executable(path: &Path, label: &str) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("Storage {label} executable is invalid"));
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| format!("Storage {label} executable is invalid"))?;
    let mode = metadata.mode();
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > 1024 * 1024 * 1024
        || mode & 0o111 == 0
        || mode & 0o022 != 0
    {
        return Err(format!("Storage {label} executable is invalid"));
    }
    Ok(())
}

pub(super) fn validate_private_file(
    path: &Path,
    maximum_bytes: u64,
    label: &str,
) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("Storage {label} is invalid"));
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| format!("Storage {label} is invalid"))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum_bytes
        || metadata.mode() & 0o077 != 0
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err(format!("Storage {label} is invalid"));
    }
    Ok(())
}

pub(super) fn validate_private_directory(path: &Path, label: &str) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("Storage {label} is invalid"));
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| format!("Storage {label} is invalid"))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.mode() & 0o077 != 0
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err(format!("Storage {label} is invalid"));
    }
    Ok(())
}

fn write_connection_files(
    connection: &PostgresConnectionArguments,
    service_path: &Path,
    password_path: &Path,
) -> Result<(), String> {
    let mut password = read_password(&connection.password_file)?;
    let service = format!(
        "[makosh_recovery]\nhost={}\nport={}\ndbname={}\nuser={}\nsslmode={}\n",
        connection.host,
        connection.port,
        connection.database,
        connection.username,
        connection.ssl_mode
    );
    let password_entry = format!(
        "{}:{}:{}:{}:{}\n",
        escape_pgpass(&connection.host),
        connection.port,
        escape_pgpass(&connection.database),
        escape_pgpass(&connection.username),
        escape_pgpass(&password)
    );
    write_private_file(service_path, service.as_bytes())?;
    let result = write_private_file(password_path, password_entry.as_bytes());
    password.zeroize();
    result
}

fn read_password(path: &Path) -> Result<Zeroizing<String>, String> {
    validate_private_file(path, 4096, "PostgreSQL password file")?;
    let bytes = fs::read(path).map_err(|_| "Storage PostgreSQL password is invalid".to_owned())?;
    let mut password = String::from_utf8(bytes)
        .map(Zeroizing::new)
        .map_err(|_| "Storage PostgreSQL password is invalid".to_owned())?;
    if password.ends_with('\n') {
        let _ = password.pop();
        if password.ends_with('\r') {
            let _ = password.pop();
        }
    }
    let valid = !password.is_empty()
        && password.len() <= 1024
        && !password
            .bytes()
            .any(|byte| matches!(byte, b'\r' | b'\n' | 0));
    valid
        .then_some(password)
        .ok_or_else(|| "Storage PostgreSQL password is invalid".to_owned())
}

fn escape_pgpass(value: &str) -> String {
    value.replace('\\', "\\\\").replace(':', "\\:")
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "Storage recovery credential staging failed".to_owned())?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "Storage recovery credential staging failed".to_owned())
}

fn create_staging(parent: &Path, purpose: &str) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Storage recovery staging failed".to_owned())?
        .as_nanos();
    for _ in 0..32 {
        let counter = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".makosh-postgres-{purpose}-{}-{timestamp}-{counter}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
                    .map_err(|_| "Storage recovery staging failed".to_owned())?;
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err("Storage recovery staging failed".to_owned()),
        }
    }
    Err("Storage recovery staging failed".to_owned())
}

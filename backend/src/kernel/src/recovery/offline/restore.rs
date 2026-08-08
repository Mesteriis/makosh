//! Restore command path for owner-authorized whole-instance recovery.

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use makosh_runtime_protocol::{
    v1::{
        EventsAuthorityRuntimeControlRequestV1, EventsAuthorityRuntimeControlResponseV1,
        ReconcileEventsTopologyResponseV1, events_authority_runtime_control_request_v1::Operation,
        events_authority_runtime_control_response_v1::Result as ResponseResult,
    },
    validation::events_authority::validate_events_authority_runtime_control_request,
};
use makosh_secure_file::{SecureReadPolicy, read as read_secure_file};
use prost::Message;

use crate::cli::WholeInstanceRestoreCli;
use crate::identity::owner::authorization::{authorize, operation_digest};
use crate::platform::events::authority::binding::EVENTS_AUTHORITY_PROCESS_ID;
use crate::recovery::control_store_media;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;

use super::keys::read_secret_key;
use super::staging::StagedRecoveryComponents;
use crate::recovery::{
    media::{
        encryption::RecoveryMediaEncryptionKey, verification::verify_published_recovery_media,
    },
    process_port::{
        PostgresRecoveryCommandV1, ProcessWholeInstanceRestorePort, RecoveryComponentExecutables,
    },
    restore_coordinator::restore_verified_instance,
};

pub(crate) fn restore_instance(
    data_dir: Option<PathBuf>,
    restore: WholeInstanceRestoreCli,
) -> Result<(), String> {
    let data_dir = explicit_data_dir_for_restore(data_dir)?;
    validate_restore_inputs(&restore)?;
    confirm_restore(&data_dir, &restore.source)?;
    authorize_restore(&data_dir, &restore)?;
    let public_key = read_public_key(&restore.media_public_key_file)?;
    let manifest =
        verify_published_recovery_media(&restore.source, &restore.media_key_id, &public_key)?;
    let staged = StagedRecoveryComponents::prepare(
        &data_dir,
        manifest.inventory().blob_enabled(),
        manifest.inventory().scheduler_enabled(),
    )?;
    let event_relay = OfflineEventTopologyRelay;
    let components = staged.components();
    let mut port = restore_port(&data_dir, &restore, &components, &event_relay)?;
    let restore_workspace = prepare_restore_workspace(&data_dir)?;
    let encryption_key =
        RecoveryMediaEncryptionKey::new(read_secret_key(&restore.media_decryption_key_file)?);
    let result = restore_verified_instance(
        &restore.source,
        &restore.media_key_id,
        &public_key,
        &restore_workspace,
        &encryption_key,
        &mut port,
    );
    staged.remove()?;
    result
}

fn explicit_data_dir_for_restore(data_dir: Option<PathBuf>) -> Result<PathBuf, String> {
    data_dir
        .filter(|path| path.is_absolute())
        .ok_or_else(|| "whole-instance restore requires an explicit absolute --data-dir".to_owned())
}

fn validate_restore_inputs(restore: &WholeInstanceRestoreCli) -> Result<(), String> {
    for path in [
        &restore.source,
        &restore.media_decryption_key_file,
        &restore.media_public_key_file,
        &restore.vault_recovery_key_file,
        &restore.pg_restore,
        &restore.psql,
        &restore.postgres_password_file,
    ] {
        if !path.is_absolute() {
            return Err("whole-instance restore paths must be absolute".to_owned());
        }
    }
    if restore.media_key_id.is_empty()
        || restore.media_key_id.len() > 128
        || !restore.media_key_id.is_ascii()
    {
        return Err("recovery media key ID is invalid".to_owned());
    }
    for value in [
        &restore.postgres_host,
        &restore.postgres_database,
        &restore.postgres_username,
        &restore.postgres_ssl_mode,
    ] {
        if value.is_empty() || value.contains('\0') {
            return Err("PostgreSQL recovery connection input is invalid".to_owned());
        }
    }
    Ok(())
}

fn confirm_restore(data_dir: &Path, source: &Path) -> Result<(), String> {
    eprintln!("offline_whole_instance_restore=true");
    eprintln!("target_data_dir={}", data_dir.display());
    eprintln!("source_recovery_media={}", source.display());
    eprint!("Type RESTORE to confirm: ");
    std::io::stderr()
        .flush()
        .map_err(|error| error.to_string())?;
    let mut confirmation = String::new();
    std::io::stdin()
        .read_line(&mut confirmation)
        .map_err(|error| error.to_string())?;
    (confirmation.trim() == "RESTORE")
        .then_some(())
        .ok_or_else(|| "whole-instance restore was not confirmed".to_owned())
}

fn authorize_restore(data_dir: &Path, restore: &WholeInstanceRestoreCli) -> Result<(), String> {
    let (_, source_store, _) = control_store_media::open_source(&restore.source)?;
    authorize(
        data_dir,
        &source_store,
        "whole_instance_restore_v1",
        restore_authorization_digest(restore)?,
    )
}

fn restore_authorization_digest(restore: &WholeInstanceRestoreCli) -> Result<[u8; 32], String> {
    let source = restore
        .source
        .to_str()
        .ok_or_else(|| "recovery source media is not valid UTF-8".to_owned())?;
    let recovery_key = restore
        .vault_recovery_key_file
        .to_str()
        .ok_or_else(|| "vault recovery key path is not valid UTF-8".to_owned())?;
    operation_digest(&[
        source,
        &restore.media_key_id,
        &restore.media_public_key_file.to_string_lossy(),
        recovery_key,
    ])
}

fn restore_port<'a>(
    data_dir: &Path,
    restore: &WholeInstanceRestoreCli,
    components: &'a RecoveryComponentExecutables<'a>,
    event_relay: &'a dyn ManagedRuntimeRelay,
) -> Result<ProcessWholeInstanceRestorePort<'a>, String> {
    ProcessWholeInstanceRestorePort::open(
        data_dir.to_owned(),
        RecoveryComponentExecutables {
            vault: components.vault,
            storage: components.storage,
            blob: components.blob,
            scheduler: components.scheduler,
        },
        postgres_command_from_restore(restore)?,
        restore.vault_recovery_key_file.clone(),
        event_relay,
    )
}

fn prepare_restore_workspace(data_dir: &Path) -> Result<PathBuf, String> {
    let restore_workspace = data_dir.join(".makosh-recovery-restore");
    if restore_workspace.exists() {
        crate::infrastructure::filesystem::ensure_owner_private_directory(&restore_workspace)
            .map_err(|_| "restore workspace is invalid".to_owned())?;
    } else {
        std::fs::create_dir_all(&restore_workspace)
            .map_err(|_| "restore workspace is unavailable".to_owned())?;
        std::fs::set_permissions(&restore_workspace, std::fs::Permissions::from_mode(0o700))
            .map_err(|_| "restore workspace is unavailable".to_owned())?;
    }
    Ok(restore_workspace)
}

fn postgres_command_from_restore(
    restore: &WholeInstanceRestoreCli,
) -> Result<PostgresRecoveryCommandV1, String> {
    Ok(PostgresRecoveryCommandV1 {
        pg_dump: PathBuf::from("/usr/bin/true"),
        pg_restore: restore.pg_restore.clone(),
        psql: restore.psql.clone(),
        host: restore.postgres_host.clone(),
        port: restore.postgres_port,
        database: restore.postgres_database.clone(),
        username: restore.postgres_username.clone(),
        ssl_mode: restore.postgres_ssl_mode.clone(),
        password_file: restore.postgres_password_file.clone(),
    })
}

fn read_public_key(path: &Path) -> Result<Vec<u8>, String> {
    use p256::ecdsa::VerifyingKey;

    let bytes = read_secure_file(path, SecureReadPolicy::owner_private(1024))
        .map_err(|_| "recovery media public key file is unavailable".to_owned())?;
    let key: [u8; 65] = bytes
        .try_into()
        .map_err(|_| "recovery media public key file has an invalid length".to_owned())?;
    let _ = VerifyingKey::from_sec1_bytes(&key)
        .map_err(|_| "recovery media public key is invalid".to_owned())?;
    Ok(key.to_vec())
}

struct OfflineEventTopologyRelay;

impl ManagedRuntimeRelay for OfflineEventTopologyRelay {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        if registration_id != EVENTS_AUTHORITY_PROCESS_ID {
            return Err("Event Hub recovery topology relay is unavailable".to_owned());
        }
        let request = EventsAuthorityRuntimeControlRequestV1::decode(&*payload)
            .map_err(|_| "Event Hub recovery topology is invalid".to_owned())?;
        validate_events_authority_runtime_control_request(&request)
            .map_err(|_| "Event Hub recovery topology is invalid".to_owned())?;
        let request = match request.operation {
            Some(Operation::ReconcileTopology(value)) => value,
            _ => {
                return Err("Event Hub topology reconciliation request is not available".to_owned());
            }
        };
        Ok(EventsAuthorityRuntimeControlResponseV1 {
            result: Some(ResponseResult::TopologyReconciled(
                ReconcileEventsTopologyResponseV1 {
                    topology_revision: request.topology_revision,
                    stream_count: request
                        .streams
                        .len()
                        .try_into()
                        .map_err(|_| "Event Hub topology is oversized".to_owned())?,
                    consumer_count: request
                        .consumers
                        .len()
                        .try_into()
                        .map_err(|_| "Event Hub topology is oversized".to_owned())?,
                },
            )),
            error_code: String::new(),
        }
        .encode_to_vec())
    }
}

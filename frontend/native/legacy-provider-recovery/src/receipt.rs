use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{LegacyProviderRecoveryErrorV1, LegacyProviderRecoveryResultV1};
use crate::model::{
    LegacyProviderCandidateKindV1, LegacyProviderRecoveryStepDispositionV1,
    LegacyProviderRecoveryStepV1, LegacyProviderRecoveryTerminalStateV1,
};
use crate::private_files::{is_generation, is_sha256};

const RECEIPT_SCHEMA_REVISION: u16 = 1;
const MAX_RECEIPT_BYTES: u64 = 512 * 1024;
const OPERATION_DOMAIN: &[u8] = b"makosh-legacy-provider-recovery-v1\0";

pub(crate) struct LegacyProviderRecoveryReceiptStoreV1 {
    path: PathBuf,
    inventory: BTreeMap<String, LegacyProviderCandidateKindV1>,
    state: Mutex<RecoveryReceiptFileV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RecoveryReceiptFileV1 {
    schema_revision: u16,
    bundle_fingerprint_sha256: String,
    source_generation: String,
    candidates: BTreeMap<String, RecoveryCandidateReceiptV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RecoveryCandidateReceiptV1 {
    kind: LegacyProviderCandidateKindV1,
    target_configuration_instance_id: Option<String>,
    completed_step_identifiers: BTreeSet<String>,
    terminal_state: Option<LegacyProviderRecoveryTerminalStateV1>,
    steps: BTreeMap<String, RecoveryStepReceiptV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RecoveryStepStateV1 {
    Pending,
    OutcomeUnknown,
    Completed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct RecoveryStepReceiptV1 {
    operation_id: String,
    state: RecoveryStepStateV1,
    public_revision: Option<u64>,
}

impl LegacyProviderRecoveryReceiptStoreV1 {
    pub(crate) fn open(
        path: &Path,
        bundle_fingerprint_sha256: &str,
        source_generation: &str,
        inventory: BTreeMap<String, LegacyProviderCandidateKindV1>,
    ) -> LegacyProviderRecoveryResultV1<Self> {
        validate_receipt_path(path)?;
        if !is_sha256(bundle_fingerprint_sha256)
            || !is_generation(source_generation)
            || inventory.len() != 3
            || inventory.keys().any(|handle| !is_sha256(handle))
        {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        let state = if path.exists() {
            read_receipt(path)?
        } else {
            RecoveryReceiptFileV1 {
                schema_revision: RECEIPT_SCHEMA_REVISION,
                bundle_fingerprint_sha256: bundle_fingerprint_sha256.to_owned(),
                source_generation: source_generation.to_owned(),
                candidates: BTreeMap::new(),
            }
        };
        validate_state(
            &state,
            bundle_fingerprint_sha256,
            source_generation,
            &inventory,
        )?;
        Ok(Self {
            path: path.to_owned(),
            inventory,
            state: Mutex::new(state),
        })
    }

    pub(crate) fn begin_step(
        &self,
        handle: &str,
        step_identifier: &str,
        target_configuration_instance_id: Option<&str>,
        explicit_retry: bool,
    ) -> LegacyProviderRecoveryResultV1<LegacyProviderRecoveryStepV1> {
        let kind = self.kind(handle)?;
        validate_step_identifier(kind, step_identifier)?;
        validate_optional_target(target_configuration_instance_id)?;
        let mut state = self.lock_state()?;
        let fingerprint = state.bundle_fingerprint_sha256.clone();
        let candidate = state
            .candidates
            .entry(handle.to_owned())
            .or_insert_with(|| RecoveryCandidateReceiptV1 {
                kind,
                target_configuration_instance_id: target_configuration_instance_id
                    .map(str::to_owned),
                completed_step_identifiers: BTreeSet::new(),
                terminal_state: None,
                steps: BTreeMap::new(),
            });
        bind_target(candidate, target_configuration_instance_id)?;
        if candidate.kind != kind {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        let expected_operation_id = operation_id(&fingerprint, handle, step_identifier);
        let expected_operation_hex = hex(&expected_operation_id);
        let step_was_persisted = candidate.steps.contains_key(step_identifier);
        let step = candidate
            .steps
            .entry(step_identifier.to_owned())
            .or_insert_with(|| RecoveryStepReceiptV1 {
                operation_id: expected_operation_hex.clone(),
                state: RecoveryStepStateV1::Pending,
                public_revision: None,
            });
        if step.operation_id != expected_operation_hex {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        let disposition = match step.state {
            RecoveryStepStateV1::Completed => LegacyProviderRecoveryStepDispositionV1::Completed,
            RecoveryStepStateV1::OutcomeUnknown if explicit_retry => {
                step.state = RecoveryStepStateV1::Pending;
                LegacyProviderRecoveryStepDispositionV1::Execute
            }
            RecoveryStepStateV1::OutcomeUnknown => {
                LegacyProviderRecoveryStepDispositionV1::OutcomeUnknown
            }
            RecoveryStepStateV1::Pending
                if candidate
                    .completed_step_identifiers
                    .contains(step_identifier) =>
            {
                return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
            }
            RecoveryStepStateV1::Pending if step.public_revision.is_some() => {
                return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
            }
            RecoveryStepStateV1::Pending if !step_was_persisted => {
                LegacyProviderRecoveryStepDispositionV1::Execute
            }
            RecoveryStepStateV1::Pending => {
                step.state = RecoveryStepStateV1::OutcomeUnknown;
                LegacyProviderRecoveryStepDispositionV1::OutcomeUnknown
            }
        };
        let result = LegacyProviderRecoveryStepV1 {
            disposition,
            operation_id: expected_operation_id,
            target_configuration_instance_id: candidate.target_configuration_instance_id.clone(),
            public_revision: step.public_revision,
        };
        write_receipt(&self.path, &state)?;
        Ok(result)
    }

    pub(crate) fn terminal_state(
        &self,
        handle: &str,
    ) -> LegacyProviderRecoveryResultV1<Option<LegacyProviderRecoveryTerminalStateV1>> {
        self.kind(handle)?;
        Ok(self
            .lock_state()?
            .candidates
            .get(handle)
            .and_then(|candidate| candidate.terminal_state))
    }

    pub(crate) fn complete_step(
        &self,
        handle: &str,
        step_identifier: &str,
        operation_id: [u8; 16],
        target_configuration_instance_id: Option<&str>,
        public_revision: Option<u64>,
    ) -> LegacyProviderRecoveryResultV1<()> {
        let kind = self.kind(handle)?;
        validate_step_identifier(kind, step_identifier)?;
        validate_optional_target(target_configuration_instance_id)?;
        let mut state = self.lock_state()?;
        let fingerprint = state.bundle_fingerprint_sha256.clone();
        let candidate = state
            .candidates
            .get_mut(handle)
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
        bind_target(candidate, target_configuration_instance_id)?;
        let step = candidate
            .steps
            .get_mut(step_identifier)
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
        if step.operation_id != hex(&operation_id)
            || operation_id != operation_id_bytes(&fingerprint, handle, step_identifier)
        {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        step.state = RecoveryStepStateV1::Completed;
        step.public_revision = public_revision;
        candidate
            .completed_step_identifiers
            .insert(step_identifier.to_owned());
        write_receipt(&self.path, &state)
    }

    pub(crate) fn finish_candidate(
        &self,
        handle: &str,
        target_configuration_instance_id: &str,
        terminal_state: LegacyProviderRecoveryTerminalStateV1,
    ) -> LegacyProviderRecoveryResultV1<()> {
        let kind = self.kind(handle)?;
        validate_target(target_configuration_instance_id)?;
        validate_terminal_state(kind, terminal_state)?;
        let mut state = self.lock_state()?;
        let candidate = state
            .candidates
            .get_mut(handle)
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
        bind_target(candidate, Some(target_configuration_instance_id))?;
        if candidate.completed_step_identifiers.is_empty() {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        candidate.terminal_state = Some(terminal_state);
        write_receipt(&self.path, &state)
    }

    fn kind(&self, handle: &str) -> LegacyProviderRecoveryResultV1<LegacyProviderCandidateKindV1> {
        self.inventory
            .get(handle)
            .copied()
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidReceipt)
    }

    fn lock_state(&self) -> LegacyProviderRecoveryResultV1<MutexGuard<'_, RecoveryReceiptFileV1>> {
        self.state
            .lock()
            .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidReceipt)
    }
}

fn validate_state(
    state: &RecoveryReceiptFileV1,
    bundle_fingerprint_sha256: &str,
    source_generation: &str,
    inventory: &BTreeMap<String, LegacyProviderCandidateKindV1>,
) -> LegacyProviderRecoveryResultV1<()> {
    if state.schema_revision != RECEIPT_SCHEMA_REVISION
        || state.bundle_fingerprint_sha256 != bundle_fingerprint_sha256
        || state.source_generation != source_generation
        || state.candidates.len() > inventory.len()
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
    }
    for (handle, candidate) in &state.candidates {
        let kind = inventory
            .get(handle)
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
        if kind != &candidate.kind {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        validate_optional_target(candidate.target_configuration_instance_id.as_deref())?;
        if let Some(terminal) = candidate.terminal_state {
            validate_terminal_state(*kind, terminal)?;
        }
        if candidate.steps.len() > 24
            || candidate.completed_step_identifiers.len() > candidate.steps.len()
        {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        for (step_identifier, step) in &candidate.steps {
            validate_step_identifier(*kind, step_identifier)?;
            if step.operation_id
                != hex(&operation_id(
                    bundle_fingerprint_sha256,
                    handle,
                    step_identifier,
                ))
                || (step.state == RecoveryStepStateV1::Completed)
                    != candidate
                        .completed_step_identifiers
                        .contains(step_identifier)
            {
                return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
            }
        }
    }
    Ok(())
}

fn validate_step_identifier(
    kind: LegacyProviderCandidateKindV1,
    step: &str,
) -> LegacyProviderRecoveryResultV1<()> {
    let valid = match kind {
        LegacyProviderCandidateKindV1::Gmail => {
            matches!(
                step,
                "mail_gmail_create_target"
                    | "mail_gmail_update_settings"
                    | "mail_gmail_apply_settings"
            ) || revision_step(step, "mail_gmail_oauth_start_revision_")
        }
        LegacyProviderCandidateKindV1::Icloud => matches!(
            step,
            "mail_icloud_create_target"
                | "mail_icloud_update_settings"
                | "mail_icloud_apply_settings"
                | "mail_icloud_provision_imap_password"
                | "mail_icloud_bind_imap_password"
        ),
        LegacyProviderCandidateKindV1::TelegramUser => {
            matches!(
                step,
                "telegram_provision_api_hash"
                    | "telegram_provision_session_store_key"
                    | "telegram_provision_account"
            ) || revision_step(step, "telegram_update_settings_revision_")
                || revision_step(step, "telegram_apply_settings_revision_")
        }
    };
    if !valid {
        return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn revision_step(value: &str, prefix: &str) -> bool {
    let Some(revision) = value.strip_prefix(prefix) else {
        return false;
    };
    !revision.is_empty()
        && revision.bytes().all(|byte| byte.is_ascii_digit())
        && (revision == "0" || !revision.starts_with('0'))
        && revision.parse::<u64>().is_ok()
}

fn validate_terminal_state(
    kind: LegacyProviderCandidateKindV1,
    state: LegacyProviderRecoveryTerminalStateV1,
) -> LegacyProviderRecoveryResultV1<()> {
    let valid = match kind {
        LegacyProviderCandidateKindV1::Gmail => matches!(
            state,
            LegacyProviderRecoveryTerminalStateV1::Completed
                | LegacyProviderRecoveryTerminalStateV1::ReauthorizationRequired
                | LegacyProviderRecoveryTerminalStateV1::BlockedSource
                | LegacyProviderRecoveryTerminalStateV1::BlockedConfig
                | LegacyProviderRecoveryTerminalStateV1::OutcomeUnknown
        ),
        LegacyProviderCandidateKindV1::Icloud => matches!(
            state,
            LegacyProviderRecoveryTerminalStateV1::Completed
                | LegacyProviderRecoveryTerminalStateV1::BlockedSource
                | LegacyProviderRecoveryTerminalStateV1::BlockedConfig
                | LegacyProviderRecoveryTerminalStateV1::OutcomeUnknown
        ),
        LegacyProviderCandidateKindV1::TelegramUser => matches!(
            state,
            LegacyProviderRecoveryTerminalStateV1::Completed
                | LegacyProviderRecoveryTerminalStateV1::QrAuthorizationRequired
                | LegacyProviderRecoveryTerminalStateV1::BlockedSource
                | LegacyProviderRecoveryTerminalStateV1::BlockedConfig
                | LegacyProviderRecoveryTerminalStateV1::OutcomeUnknown
        ),
    };
    if !valid {
        return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn bind_target(
    candidate: &mut RecoveryCandidateReceiptV1,
    target: Option<&str>,
) -> LegacyProviderRecoveryResultV1<()> {
    let Some(target) = target else {
        return Ok(());
    };
    validate_target(target)?;
    match candidate.target_configuration_instance_id.as_deref() {
        Some(current) if current != target => Err(LegacyProviderRecoveryErrorV1::InvalidReceipt),
        Some(_) => Ok(()),
        None => {
            candidate.target_configuration_instance_id = Some(target.to_owned());
            Ok(())
        }
    }
}

fn validate_optional_target(target: Option<&str>) -> LegacyProviderRecoveryResultV1<()> {
    if let Some(target) = target {
        validate_target(target)?;
    }
    Ok(())
}

fn validate_target(target: &str) -> LegacyProviderRecoveryResultV1<()> {
    if target.is_empty()
        || target.len() > 256
        || !target
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
    }
    Ok(())
}

fn operation_id(bundle_fingerprint_sha256: &str, handle: &str, step_identifier: &str) -> [u8; 16] {
    operation_id_bytes(bundle_fingerprint_sha256, handle, step_identifier)
}

fn operation_id_bytes(
    bundle_fingerprint_sha256: &str,
    handle: &str,
    step_identifier: &str,
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(OPERATION_DOMAIN);
    hasher.update(bundle_fingerprint_sha256.as_bytes());
    hasher.update(b"\0");
    hasher.update(handle.as_bytes());
    hasher.update(b"\0");
    hasher.update(step_identifier.as_bytes());
    let digest = hasher.finalize();
    let mut operation_id = [0_u8; 16];
    operation_id.copy_from_slice(&digest[..16]);
    operation_id
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn validate_receipt_path(path: &Path) -> LegacyProviderRecoveryResultV1<()> {
    if !path.is_absolute() || path.file_name().is_none() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
    }
    let parent = path
        .parent()
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
    let metadata =
        fs::symlink_metadata(parent).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
    }
    require_private_permissions(&metadata)?;
    if path.exists() {
        let metadata = fs::symlink_metadata(path)
            .map_err(|_| LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
        if !metadata.file_type().is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() == 0
            || metadata.len() > MAX_RECEIPT_BYTES
        {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
        require_private_permissions(&metadata)?;
    }
    Ok(())
}

fn read_receipt(path: &Path) -> LegacyProviderRecoveryResultV1<RecoveryReceiptFileV1> {
    validate_receipt_path(path)?;
    let bytes = fs::read(path).map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
    serde_json::from_slice(&bytes).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidReceipt)
}

fn write_receipt(
    path: &Path,
    receipt: &RecoveryReceiptFileV1,
) -> LegacyProviderRecoveryResultV1<()> {
    let bytes =
        serde_json::to_vec(receipt).map_err(|_| LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_RECEIPT_BYTES {
        return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
    }
    let parent = path
        .parent()
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidReceipt)?;
    let mut random = [0_u8; 8];
    getrandom::getrandom(&mut random)
        .map_err(|_| LegacyProviderRecoveryErrorV1::CryptographyUnavailable)?;
    let temporary = parent.join(format!(
        ".receipt-{}-{}.tmp",
        std::process::id(),
        hex(&random)
    ));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        file.write_all(&bytes)
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        file.sync_all()
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        fs::rename(&temporary, path).map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        let directory = OpenOptions::new()
            .read(true)
            .open(parent)
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)?;
        directory
            .sync_all()
            .map_err(|_| LegacyProviderRecoveryErrorV1::IoUnavailable)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn require_private_permissions(metadata: &fs::Metadata) -> LegacyProviderRecoveryResultV1<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(LegacyProviderRecoveryErrorV1::InvalidReceipt);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn fixture() -> (
        tempfile::TempDir,
        PathBuf,
        BTreeMap<String, LegacyProviderCandidateKindV1>,
    ) {
        let temporary = tempdir().expect("create temporary directory");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(temporary.path(), fs::Permissions::from_mode(0o700))
                .expect("set private directory permissions");
        }
        let handle = "b".repeat(64);
        let inventory = BTreeMap::from([
            (handle, LegacyProviderCandidateKindV1::Icloud),
            ("c".repeat(64), LegacyProviderCandidateKindV1::Gmail),
            ("d".repeat(64), LegacyProviderCandidateKindV1::TelegramUser),
        ]);
        let path = temporary.path().join("receipt.v1.json");
        (temporary, path, inventory)
    }

    #[test]
    fn persists_pending_outcome_unknown_explicit_retry_and_completion() {
        let (_temporary, path, inventory) = fixture();
        let fingerprint = "a".repeat(64);
        let generation = "c".repeat(32);
        let handle = "b".repeat(64);
        let store = LegacyProviderRecoveryReceiptStoreV1::open(
            &path,
            &fingerprint,
            &generation,
            inventory.clone(),
        )
        .expect("open receipt");
        let first = store
            .begin_step(&handle, "mail_icloud_create_target", None, false)
            .expect("begin step");
        assert_eq!(
            first.disposition,
            LegacyProviderRecoveryStepDispositionV1::Execute
        );

        let reopened =
            LegacyProviderRecoveryReceiptStoreV1::open(&path, &fingerprint, &generation, inventory)
                .expect("reopen receipt");
        let uncertain = reopened
            .begin_step(&handle, "mail_icloud_create_target", None, false)
            .expect("surface outcome unknown");
        assert_eq!(
            uncertain.disposition,
            LegacyProviderRecoveryStepDispositionV1::OutcomeUnknown
        );
        let retry = reopened
            .begin_step(&handle, "mail_icloud_create_target", None, true)
            .expect("admit explicit retry");
        assert_eq!(
            retry.disposition,
            LegacyProviderRecoveryStepDispositionV1::Execute
        );
        assert_eq!(retry.operation_id, first.operation_id);
        reopened
            .complete_step(
                &handle,
                "mail_icloud_create_target",
                retry.operation_id,
                Some("mail-target"),
                Some(1),
            )
            .expect("complete step");
        let completed = reopened
            .begin_step(
                &handle,
                "mail_icloud_create_target",
                Some("mail-target"),
                false,
            )
            .expect("read completed step");
        assert_eq!(
            completed.disposition,
            LegacyProviderRecoveryStepDispositionV1::Completed
        );
        assert_eq!(completed.public_revision, Some(1));
        assert_eq!(
            completed.target_configuration_instance_id.as_deref(),
            Some("mail-target")
        );
    }

    #[test]
    fn rejects_changed_source_corruption_permissions_and_unknown_steps() {
        let (_temporary, path, inventory) = fixture();
        let fingerprint = "a".repeat(64);
        let generation = "c".repeat(32);
        let handle = "b".repeat(64);
        let store = LegacyProviderRecoveryReceiptStoreV1::open(
            &path,
            &fingerprint,
            &generation,
            inventory.clone(),
        )
        .expect("open receipt");
        assert_eq!(
            store.begin_step(&handle, "telegram_provision_account", None, false),
            Err(LegacyProviderRecoveryErrorV1::InvalidReceipt)
        );
        let step = store
            .begin_step(&handle, "mail_icloud_create_target", None, false)
            .expect("persist receipt");
        assert_ne!(step.operation_id, [0_u8; 16]);
        assert!(
            LegacyProviderRecoveryReceiptStoreV1::open(
                &path,
                &"d".repeat(64),
                &generation,
                inventory.clone(),
            )
            .is_err()
        );

        fs::write(&path, b"{\"schema_revision\":2}").expect("corrupt receipt");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .expect("set receipt permissions");
        }
        assert!(
            LegacyProviderRecoveryReceiptStoreV1::open(
                &path,
                &fingerprint,
                &generation,
                inventory.clone(),
            )
            .is_err()
        );

        fs::write(&path, b"{}").expect("replace receipt");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644))
                .expect("set broad receipt permissions");
        }
        assert!(LegacyProviderRecoveryReceiptStoreV1::open(
            &path,
            &fingerprint,
            &generation,
            inventory,
        )
        .is_err());
    }
}

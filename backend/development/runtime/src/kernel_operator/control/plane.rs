use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use makosh_kernel_control_store::{
    ExternalRuntimeAttestation, ModuleRegistrationState, OwnerPinnedArtifactBinding,
    OwnerPinnedArtifactBindingInputV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::control_store::lifecycle::{bootstrap_control_store, open_validated_control_store};
use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::identity::enrollment::store::prepare_pristine;
use crate::infrastructure::filesystem::{
    acquire_runtime_directory_lock, ensure_not_symlink, ensure_owner_private_directory,
    ensure_regular_file_or_absent, resolve_data_directory, resolve_runtime_directory,
};
use crate::kernel_operator::artifact::{
    approval_message, read_stable_regular_file, verify as verify_owner_pinned_artifact,
    verify_owner_proof,
};
use crate::kernel_operator::pairing::load_verified_identity;
use crate::modules::registration::registry as module_registry;

pub fn run_remote_pairing_owner_enrollment(
    data_dir_override: Option<PathBuf>,
    pairing_state_dir: &Path,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development remote pairing trusts an owner-private file-backed receipt and must not receive production data"
    );
    let identity = load_verified_identity(pairing_state_dir)?;
    let (_data_dir, _lock, store) = prepare_pristine(data_dir_override)?;
    store
        .claim_initial_owner(&identity)
        .map_err(|error| format!("{error:?}"))?;
    println!("development_remote_initial_owner_enrolled=true");
    println!("owner_id={}", identity.owner_id());
    println!("device_id={}", identity.device_id());
    Ok(())
}

pub fn run_module_registration(
    data_dir_override: Option<PathBuf>,
    descriptor_path: &Path,
) -> Result<(), String> {
    eprintln!("WARNING: development_full_platform_v1 accepts local untrusted module descriptors");
    if !descriptor_path.is_absolute() {
        return Err("module descriptor must be an absolute path".to_owned());
    }
    ensure_regular_file_or_absent(descriptor_path, "module descriptor")?;
    if !descriptor_path.exists() {
        return Err("module descriptor does not exist".to_owned());
    }
    let descriptor_bytes = std::fs::read(descriptor_path).map_err(|error| error.to_string())?;
    let data_dir = resolve_data_directory(data_dir_override)?;
    let data_dir_existed = data_dir.exists();
    ensure_not_symlink(&data_dir, "data directory")?;
    std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
    if data_dir_existed {
        ensure_owner_private_directory(&data_dir)?;
    } else {
        std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let runtime_dir_existed = runtime_dir.exists();
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    if runtime_dir_existed {
        ensure_owner_private_directory(&runtime_dir)?;
    } else {
        std::fs::set_permissions(&runtime_dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store = bootstrap_control_store(&data_dir, &data_dir.join("kernel-control-store.sqlite"))?;
    let registration = module_registry::register(&store, &descriptor_bytes)?;
    println!("module_registration_id={}", registration.registration_id());
    println!("module_registration_state=pending");
    Ok(())
}

pub fn run_module_approval(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    capability_ids: &[String],
) -> Result<(), String> {
    eprintln!("WARNING: file-backed device signer has no interactive presence proof");
    let data_dir = resolve_data_directory(data_dir_override)?;
    if !data_dir.exists() {
        return Err("development data directory does not exist".to_owned());
    }
    ensure_not_symlink(&data_dir, "data directory")?;
    ensure_owner_private_directory(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    ensure_owner_private_directory(&runtime_dir)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let store = open_validated_control_store(&data_dir.join("kernel-control-store.sqlite"))?;
    let grants = module_registry::approve(&data_dir, &store, registration_id, capability_ids)?;
    println!("module_registration_id={}", grants.registration_id());
    println!("module_grant_epoch={}", grants.grant_epoch());
    println!(
        "effective_capability_count={}",
        grants.capability_ids().len()
    );
    Ok(())
}

pub fn run_module_transition(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    state: &str,
) -> Result<(), String> {
    eprintln!("WARNING: file-backed device signer has no interactive presence proof");
    let data_dir = resolve_data_directory(data_dir_override.clone())?;
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let next = match state {
        "suspended" => ModuleRegistrationState::Suspended,
        "revoked" => ModuleRegistrationState::Revoked,
        _ => return Err("unsupported module transition state".to_owned()),
    };
    let registration = module_registry::transition(&data_dir, &store, registration_id, next)?;
    println!("module_registration_id={}", registration.registration_id());
    println!(
        "module_registration_state={}",
        registration.state().as_str()
    );
    println!("module_grant_epoch={}", registration.grant_epoch());
    Ok(())
}

pub fn run_module_status(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
) -> Result<(), String> {
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let status = module_registry::status(&store, registration_id)?;
    let registration = status.registration();
    println!("module_registration_id={}", registration.registration_id());
    println!(
        "module_registration_state={}",
        registration.state().as_str()
    );
    println!("module_grant_epoch={}", registration.grant_epoch());
    println!(
        "effective_capability_count={}",
        status.effective_capability_count()
    );
    if let Some(attestation) = status.external_runtime_attestation() {
        println!("external_runtime_id={}", attestation.runtime_id());
        println!(
            "external_runtime_generation={}",
            attestation.runtime_generation()
        );
    } else {
        println!("external_runtime_attested=false");
    }
    Ok(())
}

pub fn run_external_runtime_attestation(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    runtime_id: &str,
    runtime_generation: u64,
    distribution_artifact: &Path,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 records a locally verified artifact digest; this does not control Docker or authorize managed launch"
    );
    let distribution_sha256 = read_stable_regular_file(distribution_artifact)?
        .sha256()
        .to_owned();
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let snapshot = store
        .module_grant_snapshot(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| {
            "external runtime attestation requires an approved module registration".to_owned()
        })?;
    let grants = snapshot.effective_grants().ok_or_else(|| {
        "external runtime attestation requires an approved module registration".to_owned()
    })?;
    let attestation = ExternalRuntimeAttestation::new(
        registration_id,
        runtime_id,
        runtime_generation,
        grants.grant_epoch(),
        distribution_sha256,
    );
    store
        .attest_external_runtime(&attestation)
        .map_err(|error| format!("{error:?}"))?;
    println!("module_registration_id={registration_id}");
    println!("external_runtime_id={runtime_id}");
    println!("external_runtime_generation={runtime_generation}");
    println!("external_runtime_grant_epoch={}", grants.grant_epoch());
    Ok(())
}

pub fn run_owner_pinned_artifact_binding(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
    artifact_path: &Path,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 uses an exportable file signer; this records approval only and does not authorize a managed launch"
    );
    let data_dir = resolve_data_directory(data_dir_override.clone())?;
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let artifact = read_stable_regular_file(artifact_path)?;
    let binding_revision = store
        .effective_owner_pinned_artifact_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |binding| {
            binding
                .binding_revision()
                .checked_add(1)
                .ok_or_else(|| "owner-pinned artifact binding revision overflowed".to_owned())
        })?;
    let owner = store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| {
            "development owner-pinned artifact binding requires an enrolled owner".to_owned()
        })?;
    let signer = FileDeviceSigner::open_for_instance(&data_dir)?;
    if signer.public_key_sec1() != *owner.public_key_sec1() {
        return Err("file device signer does not match the enrolled owner device".to_owned());
    }
    let message = approval_message(
        store.snapshot().instance_id(),
        registration_id,
        binding_revision,
        &artifact,
    )?;
    let signature = signer.sign(&message);
    verify_owner_proof(&owner, &message, &signature)?;
    let binding = OwnerPinnedArtifactBinding::new(OwnerPinnedArtifactBindingInputV1 {
        registration_id: registration_id.to_owned(),
        binding_revision,
        canonical_artifact_path: artifact.canonical_path().to_owned(),
        artifact_sha256: *artifact.sha256(),
        artifact_size: artifact.size(),
        artifact_device: artifact.device(),
        artifact_inode: artifact.inode(),
        owner_signature_raw: signature,
    });
    store
        .record_owner_pinned_artifact_binding(&binding)
        .map_err(|error| format!("{error:?}"))?;
    println!("module_registration_id={registration_id}");
    println!("owner_pinned_artifact_binding_revision={binding_revision}");
    println!(
        "owner_pinned_artifact_sha256={}",
        hex_digest(artifact.sha256())
    );
    Ok(())
}

pub fn run_owner_pinned_artifact_preflight(
    data_dir_override: Option<PathBuf>,
    registration_id: &str,
) -> Result<(), String> {
    eprintln!(
        "WARNING: development_full_platform_v1 preflight verifies local bytes only; it does not spawn or supervise a process"
    );
    let (runtime_dir, store) = open_development_control_store(data_dir_override)?;
    let _lock = acquire_runtime_directory_lock(&runtime_dir)?;
    let artifact = verify_owner_pinned_artifact(&store, registration_id)?;
    println!("module_registration_id={registration_id}");
    println!(
        "owner_pinned_artifact_binding_revision={}",
        artifact.binding_revision()
    );
    println!("owner_pinned_artifact_preflight=verified");
    Ok(())
}

pub(crate) fn open_development_control_store(
    data_dir_override: Option<PathBuf>,
) -> Result<(PathBuf, SqliteControlStore), String> {
    let data_dir = resolve_data_directory(data_dir_override)?;
    if !data_dir.exists() {
        return Err("development data directory does not exist".to_owned());
    }
    ensure_not_symlink(&data_dir, "data directory")?;
    ensure_owner_private_directory(&data_dir)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    ensure_not_symlink(&runtime_dir, "runtime directory")?;
    std::fs::create_dir_all(&runtime_dir).map_err(|error| error.to_string())?;
    ensure_owner_private_directory(&runtime_dir)?;
    let store = open_validated_control_store(&data_dir.join("kernel-control-store.sqlite"))?;
    if store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .is_none()
    {
        return Err(
            "development control-plane operation requires an enrolled initial owner".to_owned(),
        );
    }
    Ok((runtime_dir, store))
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

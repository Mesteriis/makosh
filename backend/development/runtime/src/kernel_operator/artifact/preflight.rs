//! Verification of the stored development owner-pinned artifact record.

use std::path::{Path, PathBuf};

use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::digest::read_stable_regular_file;
use super::owner_proof::{approval_message, verify_owner_proof};

pub struct VerifiedOwnerPinnedArtifact {
    canonical_path: PathBuf,
    binding_revision: u64,
    sha256: [u8; 32],
}

impl VerifiedOwnerPinnedArtifact {
    #[must_use]
    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    #[must_use]
    pub fn binding_revision(&self) -> u64 {
        self.binding_revision
    }

    #[must_use]
    pub fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }
}

pub fn verify(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<VerifiedOwnerPinnedArtifact, String> {
    let binding = store
        .effective_owner_pinned_artifact_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| {
            "owner-pinned artifact preflight requires an approved registration and binding"
                .to_owned()
        })?;
    let owner = store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "owner-pinned artifact preflight requires an enrolled owner".to_owned())?;
    let artifact = read_stable_regular_file(Path::new(binding.canonical_artifact_path()))?;
    if artifact.canonical_path() != binding.canonical_artifact_path()
        || artifact.sha256() != binding.artifact_sha256()
        || artifact.size() != binding.artifact_size()
        || artifact.device() != binding.artifact_device()
        || artifact.inode() != binding.artifact_inode()
    {
        return Err("owner-pinned artifact no longer matches its approved binding".to_owned());
    }
    let message = approval_message(
        store.snapshot().instance_id(),
        registration_id,
        binding.binding_revision(),
        &artifact,
    )?;
    verify_owner_proof(&owner, &message, binding.owner_signature_raw())?;
    Ok(VerifiedOwnerPinnedArtifact {
        canonical_path: PathBuf::from(artifact.canonical_path()),
        binding_revision: binding.binding_revision(),
        sha256: *artifact.sha256(),
    })
}

//! Canonical proof bytes for the development owner-pinned artifact approval.

use super::digest::ArtifactDigest;
use makosh_kernel_control_store::InitialOwnerIdentity;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};

const OWNER_PINNED_ARTIFACT_DOMAIN: &[u8] = b"makosh.development.owner-pinned-artifact.v1\0";

pub fn approval_message(
    instance_id: &str,
    registration_id: &str,
    binding_revision: u64,
    artifact: &ArtifactDigest,
) -> Result<Vec<u8>, String> {
    let mut message = Vec::with_capacity(
        OWNER_PINNED_ARTIFACT_DOMAIN.len()
            + instance_id.len()
            + registration_id.len()
            + artifact.canonical_path().len()
            + 32
            + 32,
    );
    message.extend_from_slice(OWNER_PINNED_ARTIFACT_DOMAIN);
    append_text(&mut message, instance_id)?;
    append_text(&mut message, registration_id)?;
    message.extend_from_slice(&binding_revision.to_be_bytes());
    append_text(&mut message, artifact.canonical_path())?;
    message.extend_from_slice(artifact.sha256());
    message.extend_from_slice(&artifact.size().to_be_bytes());
    message.extend_from_slice(&artifact.device().to_be_bytes());
    message.extend_from_slice(&artifact.inode().to_be_bytes());
    Ok(message)
}

pub fn verify_owner_proof(
    owner: &InitialOwnerIdentity,
    message: &[u8],
    signature_raw: &[u8; 64],
) -> Result<(), String> {
    let verifying_key = VerifyingKey::from_sec1_bytes(owner.public_key_sec1())
        .map_err(|_| "enrolled owner public key is invalid".to_owned())?;
    let signature = Signature::from_slice(signature_raw)
        .map_err(|_| "owner-pinned artifact signature is invalid".to_owned())?;
    verifying_key
        .verify(message, &signature)
        .map_err(|_| "owner-pinned artifact proof verification failed".to_owned())
}

fn append_text(output: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let length = u16::try_from(value.len())
        .map_err(|_| "owner-pinned artifact proof field exceeds its limit".to_owned())?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

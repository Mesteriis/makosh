//! Process-local owner authorization using the baseline file signer.

use makosh_kernel_control_store::{HealthRecoveryStore, InitialOwnerIdentity, OwnerIdentityStore};
use makosh_kernel_control_store_sqlite::StoreError;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};

const AUTHORIZATION_DOMAIN: &[u8] = b"makosh.file-owner-authorization.v1\0";

pub fn authorize<S>(
    data_dir: &std::path::Path,
    store: &S,
    purpose: &str,
    operation_digest: [u8; 32],
) -> Result<(), String>
where
    S: HealthRecoveryStore + OwnerIdentityStore<Error = StoreError>,
{
    if purpose.is_empty() || purpose.len() > 128 || !purpose.is_ascii() {
        return Err("owner authorization purpose is invalid".to_owned());
    }
    let owner = store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "owner authorization requires an enrolled initial owner".to_owned())?;
    let signer = FileDeviceSigner::open_for_instance(data_dir)?;
    if signer.public_key_sec1() != *owner.public_key_sec1() {
        return Err("file device signer does not match the enrolled owner device".to_owned());
    }
    let mut nonce = [0_u8; 32];
    getrandom::fill(&mut nonce).map_err(|error| error.to_string())?;
    let message = authorization_message(
        store.control_store_snapshot().instance_id(),
        &owner,
        store.control_store_snapshot().generation(),
        purpose,
        operation_digest,
        nonce,
    )?;
    let signature = signer.sign(&message);
    verify(&owner, &message, &signature)
}

pub fn operation_digest(fields: &[&str]) -> Result<[u8; 32], String> {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    for field in fields {
        let length = u16::try_from(field.len())
            .map_err(|_| "owner authorization field exceeds its limit".to_owned())?;
        digest.update(length.to_be_bytes());
        digest.update(field.as_bytes());
    }
    Ok(digest.finalize().into())
}

fn authorization_message(
    instance_id: &str,
    owner: &InitialOwnerIdentity,
    kernel_generation: u64,
    purpose: &str,
    operation_digest: [u8; 32],
    nonce: [u8; 32],
) -> Result<Vec<u8>, String> {
    let mut message = Vec::with_capacity(
        AUTHORIZATION_DOMAIN.len()
            + instance_id.len()
            + owner.owner_id().len()
            + owner.device_id().len()
            + purpose.len()
            + 32
            + 32
            + 8,
    );
    message.extend_from_slice(AUTHORIZATION_DOMAIN);
    append_text(&mut message, instance_id)?;
    append_text(&mut message, owner.owner_id())?;
    append_text(&mut message, owner.device_id())?;
    message.extend_from_slice(&kernel_generation.to_be_bytes());
    append_text(&mut message, purpose)?;
    message.extend_from_slice(&operation_digest);
    message.extend_from_slice(&nonce);
    Ok(message)
}

fn append_text(message: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let length = u16::try_from(value.len())
        .map_err(|_| "owner authorization field exceeds its limit".to_owned())?;
    message.extend_from_slice(&length.to_be_bytes());
    message.extend_from_slice(value.as_bytes());
    Ok(())
}

fn verify(
    owner: &InitialOwnerIdentity,
    message: &[u8],
    signature_raw: &[u8; 64],
) -> Result<(), String> {
    let verifying_key = VerifyingKey::from_sec1_bytes(owner.public_key_sec1())
        .map_err(|_| "enrolled owner public key is invalid".to_owned())?;
    let signature = Signature::from_slice(signature_raw)
        .map_err(|_| "owner authorization signature is invalid".to_owned())?;
    verifying_key
        .verify(message, &signature)
        .map_err(|_| "owner authorization proof verification failed".to_owned())
}

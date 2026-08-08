//! File-backed device proof verification for server bootstrap pairing.

use makosh_kernel_control_store::InitialOwnerIdentity;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};

const PROOF_DOMAIN: &[u8] = b"makosh.server-bootstrap-pairing.v1\0";

pub fn verify(
    challenge: &[u8; 32],
    owner_id: &str,
    device_id: &str,
    public_key_hex: &str,
    signature_hex: &str,
) -> Result<InitialOwnerIdentity, String> {
    validate_identity(owner_id, "owner_id")?;
    validate_identity(device_id, "device_id")?;
    let public_key = decode_hex::<65>(public_key_hex, "device public key")?;
    let signature = Signature::from_slice(&decode_hex::<64>(signature_hex, "device signature")?)
        .map_err(|_| "device signature is invalid".to_owned())?;
    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key)
        .map_err(|_| "device public key is invalid".to_owned())?;
    verifying_key
        .verify(
            &proof_message(challenge, &public_key, owner_id, device_id)?,
            &signature,
        )
        .map_err(|_| "device proof is invalid".to_owned())?;
    Ok(InitialOwnerIdentity::new(owner_id, device_id, public_key))
}

pub fn proof_message(
    challenge: &[u8; 32],
    public_key: &[u8; 65],
    owner_id: &str,
    device_id: &str,
) -> Result<Vec<u8>, String> {
    validate_identity(owner_id, "owner_id")?;
    validate_identity(device_id, "device_id")?;
    let mut message = Vec::with_capacity(
        PROOF_DOMAIN.len()
            + challenge.len()
            + public_key.len()
            + owner_id.len()
            + device_id.len()
            + 4,
    );
    message.extend_from_slice(PROOF_DOMAIN);
    message.extend_from_slice(challenge);
    message.extend_from_slice(public_key);
    message.extend_from_slice(&(owner_id.len() as u16).to_be_bytes());
    message.extend_from_slice(owner_id.as_bytes());
    message.extend_from_slice(&(device_id.len() as u16).to_be_bytes());
    message.extend_from_slice(device_id.as_bytes());
    Ok(message)
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex<const N: usize>(value: &str, field: &str) -> Result<[u8; N], String> {
    if value.len() != N * 2 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{field} has an invalid encoding"));
    }
    let mut bytes = [0_u8; N];
    for (index, output) in bytes.iter_mut().enumerate() {
        *output = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| format!("{field} has an invalid encoding"))?;
    }
    Ok(bytes)
}

fn validate_identity(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(format!("{field} must be an ASCII identifier"));
    }
    Ok(())
}

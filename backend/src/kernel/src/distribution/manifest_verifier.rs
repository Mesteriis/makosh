//! Verifies a detached signed distribution manifest against release-pinned keys.

use makosh_runtime_protocol::v1::DistributionManifestV1;
use makosh_runtime_protocol::validation::distribution::decode_signed_distribution_manifest_v1;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use prost::Message;

use crate::distribution::trust_root::ReleaseTrustRoot;

pub fn verify(
    signed_bytes: &[u8],
    trust_root: &ReleaseTrustRoot,
) -> Result<DistributionManifestV1, String> {
    let signed = decode_signed_distribution_manifest_v1(signed_bytes)
        .map_err(|_| "signed distribution manifest is invalid".to_owned())?;
    let key = trust_root
        .verification_key(&signed.verification_key_id)
        .ok_or_else(|| "distribution verification key is not pinned".to_owned())?;
    let verifying_key = VerifyingKey::from_sec1_bytes(key.public_key_sec1())
        .map_err(|_| "pinned distribution verification key is invalid".to_owned())?;
    let signature = Signature::from_slice(&signed.signature_raw)
        .map_err(|_| "distribution manifest signature is invalid".to_owned())?;
    verifying_key
        .verify(&signed.raw_manifest_bytes, &signature)
        .map_err(|_| "distribution manifest signature verification failed".to_owned())?;
    DistributionManifestV1::decode(signed.raw_manifest_bytes.as_slice())
        .map_err(|_| "distribution manifest is invalid".to_owned())
}

//! Baseline initial-owner ceremony backed by the owner-private file signer.

use std::path::PathBuf;

use makosh_kernel_control_store::InitialOwnerIdentity;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::{
    v1::{InitialOwnerEnrollmentChallengeV1, InitialOwnerEnrollmentV1},
    validation::descriptor::validate_initial_owner_enrollment,
};
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use prost::Message;

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::identity::enrollment::store::prepare_pristine;

const INITIAL_ENROLLMENT_DOMAIN: &[u8] = b"makosh.initial-owner-enrollment.v1\0";

pub fn run(
    data_dir_override: Option<PathBuf>,
    owner_id: &str,
    device_id: &str,
) -> Result<(), String> {
    eprintln!(
        "WARNING: file-backed device signer is exportable; protect its owner-private directory and do not copy the key"
    );
    if !valid_identity(owner_id) || !valid_identity(device_id) {
        return Err("owner_id and device_id must be ASCII identifiers".to_owned());
    }
    let (data_dir, _lock, store) = prepare_pristine(data_dir_override)?;
    let mut nonce = [0_u8; 32];
    getrandom::fill(&mut nonce).map_err(|error| error.to_string())?;
    let challenge = InitialOwnerEnrollmentChallengeV1 {
        protocol_major: 1,
        instance_id: store.snapshot().instance_id().as_bytes().to_vec(),
        nonce: nonce.to_vec(),
        kernel_generation: store.snapshot().generation(),
    };
    let (signer, _created) = FileDeviceSigner::open_or_create_for_instance(&data_dir)?;
    let public_key = signer.public_key_sec1();
    let enrollment = InitialOwnerEnrollmentV1 {
        protocol_major: 1,
        device_public_key_sec1: public_key.to_vec(),
        challenge_signature_raw: signer.sign(&proof_message(&challenge)).to_vec(),
        owner_id: owner_id.to_owned(),
        device_id: device_id.to_owned(),
    };
    verify_and_claim(&store, &challenge, &enrollment)?;
    println!("file_initial_owner_enrolled=true");
    println!("owner_id={owner_id}");
    println!("device_id={device_id}");
    Ok(())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn proof_message(challenge: &InitialOwnerEnrollmentChallengeV1) -> Vec<u8> {
    let mut message = Vec::with_capacity(INITIAL_ENROLLMENT_DOMAIN.len() + challenge.encoded_len());
    message.extend_from_slice(INITIAL_ENROLLMENT_DOMAIN);
    challenge
        .encode(&mut message)
        .expect("Vec allocation cannot fail");
    message
}

fn verify_and_claim(
    store: &SqliteControlStore,
    challenge: &InitialOwnerEnrollmentChallengeV1,
    enrollment: &InitialOwnerEnrollmentV1,
) -> Result<(), String> {
    if !validate_initial_owner_enrollment(challenge, enrollment) {
        return Err("initial owner enrollment is malformed".to_owned());
    }
    let public_key: [u8; 65] = enrollment
        .device_public_key_sec1
        .clone()
        .try_into()
        .map_err(|_| "initial owner public key has an invalid length".to_owned())?;
    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key)
        .map_err(|_| "initial owner public key is invalid".to_owned())?;
    let signature = Signature::from_slice(&enrollment.challenge_signature_raw)
        .map_err(|_| "initial owner signature is invalid".to_owned())?;
    verifying_key
        .verify(&proof_message(challenge), &signature)
        .map_err(|_| "initial owner proof verification failed".to_owned())?;
    store
        .claim_initial_owner(&InitialOwnerIdentity::new(
            enrollment.owner_id.clone(),
            enrollment.device_id.clone(),
            public_key,
        ))
        .map_err(|error| format!("{error:?}"))
}

//! Shared active-owner-device proof for public Core Gateway control ceremonies.

use makosh_gateway_runtime::OwnerBrowserPrincipalV1;
use makosh_kernel_control_store::BrowserDeviceStateV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};

const LOOPBACK_DEVELOPMENT_SESSION_ID: &str = "loopback-development";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OwnerDeviceProofErrorV1 {
    InvalidArgument,
    PermissionDenied,
    Internal,
}

pub(crate) fn validate_active_principal(
    store: &SqliteControlStore,
    principal: &OwnerBrowserPrincipalV1,
) -> Result<(), OwnerDeviceProofErrorV1> {
    let owner = store
        .initial_owner_identity()
        .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
        .ok_or(OwnerDeviceProofErrorV1::PermissionDenied)?;
    if owner.owner_id() != principal.owner_id() {
        return Err(OwnerDeviceProofErrorV1::PermissionDenied);
    }
    if principal.session_id() == LOOPBACK_DEVELOPMENT_SESSION_ID {
        return (owner.device_id() == principal.device_id())
            .then_some(())
            .ok_or(OwnerDeviceProofErrorV1::PermissionDenied);
    }
    let device = store
        .browser_device_identity(principal.device_id())
        .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
        .ok_or(OwnerDeviceProofErrorV1::PermissionDenied)?;
    let enrollment = device.enrollment();
    if enrollment.owner_id() != principal.owner_id()
        || enrollment.device_id() != principal.device_id()
        || device.state() != BrowserDeviceStateV1::Active
        || device.identity_epoch()
            != store
                .current_identity_epoch()
                .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
    {
        return Err(OwnerDeviceProofErrorV1::PermissionDenied);
    }
    Ok(())
}

pub(crate) fn verify_fresh_proof(
    store: &SqliteControlStore,
    principal: &OwnerBrowserPrincipalV1,
    challenge_bytes: &[u8; 32],
    signature_raw: &[u8],
) -> Result<(), OwnerDeviceProofErrorV1> {
    validate_active_principal(store, principal)?;
    let public_key_sec1 = if principal.session_id() == LOOPBACK_DEVELOPMENT_SESSION_ID {
        store
            .initial_owner_identity()
            .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
            .ok_or(OwnerDeviceProofErrorV1::PermissionDenied)?
            .public_key_sec1()
            .to_vec()
    } else {
        store
            .browser_device_identity(principal.device_id())
            .map_err(|_| OwnerDeviceProofErrorV1::Internal)?
            .ok_or(OwnerDeviceProofErrorV1::PermissionDenied)?
            .enrollment()
            .browser_key_public_key()
            .to_vec()
    };
    let verifying_key = VerifyingKey::from_sec1_bytes(&public_key_sec1)
        .map_err(|_| OwnerDeviceProofErrorV1::PermissionDenied)?;
    let signature = Signature::from_slice(signature_raw)
        .map_err(|_| OwnerDeviceProofErrorV1::InvalidArgument)?;
    verifying_key
        .verify(challenge_bytes, &signature)
        .map_err(|_| OwnerDeviceProofErrorV1::PermissionDenied)
}

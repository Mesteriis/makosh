//! Exact owner, browser-device, descriptor and grant admission checks.

use makosh_gateway_protocol::v1::{
    OwnerVaultActionV1, OwnerVaultSecretClassV1, PrepareOwnerVaultProvisioningRequestV1,
};
use makosh_gateway_runtime::{OwnerVaultClientPrincipalV1, OwnerVaultProvisioningRouteErrorV1};
use makosh_kernel_control_store::{ModuleRegistrationState, ModuleVaultPurposeRequestV1};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_vault_protocol::{
    SecretClassV1, VaultActionV1, VaultPurposeRequestV1, VaultTransportPublicKey,
};

use crate::platform::gateway::owner_device_proof::{self, OwnerDeviceProofErrorV1};

#[derive(Debug)]
pub(super) struct AuthorizedTargetV1 {
    pub(super) logical_owner_id: String,
    pub(super) grant_epoch: u64,
    pub(super) purpose: VaultPurposeRequestV1,
    pub(super) response_recipient_public_key: [u8; 32],
}

pub(super) fn authorize_target(
    store: &SqliteControlStore,
    principal: &OwnerVaultClientPrincipalV1,
    request: &PrepareOwnerVaultProvisioningRequestV1,
) -> Result<AuthorizedTargetV1, OwnerVaultProvisioningRouteErrorV1> {
    owner_device_proof::validate_active_principal(store, principal).map_err(map_proof_error)?;
    if request.operation_id.len() != 16
        || request.operation_id.iter().all(|byte| *byte == 0)
        || request.secret_revision == 0
    {
        return Err(OwnerVaultProvisioningRouteErrorV1::InvalidArgument);
    }
    let response_recipient_public_key = request
        .response_recipient_hpke_public_key_x25519
        .as_slice()
        .try_into()
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::InvalidArgument)?;
    VaultTransportPublicKey::from_bytes(response_recipient_public_key)
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::InvalidArgument)?;
    let secret_class = secret_class(request.secret_class)?;
    let action = action(request.action)?;
    let snapshot = store
        .module_grant_snapshot(&request.target_registration_id)
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?
        .ok_or(OwnerVaultProvisioningRouteErrorV1::NotFound)?;
    let registration = snapshot.registration();
    let grants = snapshot
        .effective_grants()
        .ok_or(OwnerVaultProvisioningRouteErrorV1::PermissionDenied)?;
    if registration.state() != ModuleRegistrationState::Approved
        || registration.grant_epoch() != grants.grant_epoch()
        || grants
            .capability_ids()
            .binary_search_by(|candidate| candidate.as_str().cmp(&request.capability_id))
            .is_err()
    {
        return Err(OwnerVaultProvisioningRouteErrorV1::PermissionDenied);
    }
    let declared = store
        .module_vault_purpose_requests(registration.registration_id(), &request.capability_id)
        .map_err(|_| OwnerVaultProvisioningRouteErrorV1::Internal)?
        .into_iter()
        .find(|declared| purpose_matches(declared, request))
        .ok_or(OwnerVaultProvisioningRouteErrorV1::PermissionDenied)?;
    let purpose = VaultPurposeRequestV1::new(
        request.purpose_id.clone(),
        request.configuration_instance_id.clone(),
        vec![secret_class],
        vec![action],
        u32::from(declared.requested_lease_ttl_seconds()),
    )
    .map_err(|_| OwnerVaultProvisioningRouteErrorV1::InvalidArgument)?;
    Ok(AuthorizedTargetV1 {
        logical_owner_id: registration.owner_id().to_owned(),
        grant_epoch: grants.grant_epoch(),
        purpose,
        response_recipient_public_key,
    })
}

pub(super) fn verify_fresh_proof(
    store: &SqliteControlStore,
    principal: &OwnerVaultClientPrincipalV1,
    challenge_bytes: &[u8; 32],
    signature_raw: &[u8],
) -> Result<(), OwnerVaultProvisioningRouteErrorV1> {
    owner_device_proof::verify_fresh_proof(store, principal, challenge_bytes, signature_raw)
        .map_err(map_proof_error)
}

fn map_proof_error(error: OwnerDeviceProofErrorV1) -> OwnerVaultProvisioningRouteErrorV1 {
    match error {
        OwnerDeviceProofErrorV1::InvalidArgument => {
            OwnerVaultProvisioningRouteErrorV1::InvalidArgument
        }
        OwnerDeviceProofErrorV1::PermissionDenied => {
            OwnerVaultProvisioningRouteErrorV1::PermissionDenied
        }
        OwnerDeviceProofErrorV1::Internal => OwnerVaultProvisioningRouteErrorV1::Internal,
    }
}

fn purpose_matches(
    declared: &ModuleVaultPurposeRequestV1,
    request: &PrepareOwnerVaultProvisioningRequestV1,
) -> bool {
    declared.purpose_id() == request.purpose_id
        && secret_class(request.secret_class)
            .is_ok_and(|secret_class| declared.secret_class() == secret_class.code() as u8)
        && action(request.action).is_ok_and(|action| declared.action() == action.code() as u8)
        && declared.target_scope() == 1
}

fn secret_class(value: i32) -> Result<SecretClassV1, OwnerVaultProvisioningRouteErrorV1> {
    match OwnerVaultSecretClassV1::try_from(value).ok() {
        Some(OwnerVaultSecretClassV1::ProviderCredential) => Ok(SecretClassV1::ProviderCredential),
        Some(OwnerVaultSecretClassV1::OauthRefreshCredential) => {
            Ok(SecretClassV1::OAuthRefreshCredential)
        }
        Some(OwnerVaultSecretClassV1::SessionCredentialBlob) => {
            Ok(SecretClassV1::SessionCredentialBlob)
        }
        Some(OwnerVaultSecretClassV1::SessionStoreKey) => Ok(SecretClassV1::SessionStoreKey),
        _ => Err(OwnerVaultProvisioningRouteErrorV1::InvalidArgument),
    }
}

fn action(value: i32) -> Result<VaultActionV1, OwnerVaultProvisioningRouteErrorV1> {
    match OwnerVaultActionV1::try_from(value).ok() {
        Some(OwnerVaultActionV1::Create) => Ok(VaultActionV1::Create),
        Some(OwnerVaultActionV1::ReplaceCas) => Ok(VaultActionV1::ReplaceCas),
        Some(OwnerVaultActionV1::Retire) => Ok(VaultActionV1::Retire),
        Some(OwnerVaultActionV1::Delete) => Ok(VaultActionV1::Delete),
        _ => Err(OwnerVaultProvisioningRouteErrorV1::InvalidArgument),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use makosh_gateway_protocol::v1::{
        OwnerVaultActionV1, OwnerVaultSecretClassV1, PrepareOwnerVaultProvisioningRequestV1,
    };
    use makosh_gateway_runtime::{OwnerVaultClientPrincipalV1, OwnerVaultProvisioningRouteErrorV1};
    use makosh_kernel_control_store::{
        BrowserDeviceEnrollmentInputV1, BrowserDeviceEnrollmentV1, InitialOwnerIdentity,
        ModuleDescriptorRegistrationRequestsV1, ModuleRegistration, ModuleRegistrationState,
        ModuleVaultPurposeRequestV1,
    };
    use makosh_kernel_control_store_sqlite::SqliteControlStore;
    use makosh_runtime_protocol::v1::{VaultActionV1 as RuntimeVaultActionV1, VaultSecretClassV1};
    use makosh_vault_protocol::VaultResponseRecipientV1;
    use p256::ecdsa::{Signature, SigningKey, signature::Signer};

    use super::{authorize_target, secret_class, verify_fresh_proof};

    const HUMAN_OWNER: &str = "owner-primary";
    const DEVICE: &str = "browser-device";
    const REGISTRATION: &str = "mail-registration";
    const CAPABILITY: &str = "mail.credentials.setup";
    const PURPOSE: &str = "mail.imap.password";

    #[test]
    fn maps_session_store_key_without_widening_the_secret_class() {
        assert_eq!(
            secret_class(OwnerVaultSecretClassV1::SessionStoreKey as i32)
                .expect("session store key"),
            makosh_vault_protocol::SecretClassV1::SessionStoreKey
        );
        assert_eq!(
            secret_class(OwnerVaultSecretClassV1::Unspecified as i32)
                .expect_err("unspecified class must be rejected"),
            OwnerVaultProvisioningRouteErrorV1::InvalidArgument
        );
    }

    #[test]
    fn admits_only_current_owner_device_and_exact_approved_purpose() {
        let (root, store, principal) = fixture();
        let request = request();

        let target = authorize_target(&store, &principal, &request).expect("exact purpose");
        assert_eq!(target.logical_owner_id, "mail");
        assert!(target.grant_epoch > 0);

        let wrong_owner = OwnerVaultClientPrincipalV1::new(
            "owner-other",
            DEVICE,
            "1111111111111111111111111111111111111111111111111111111111111111",
        )
        .expect("principal");
        assert_eq!(
            authorize_target(&store, &wrong_owner, &request)
                .expect_err("wrong owner must be denied"),
            OwnerVaultProvisioningRouteErrorV1::PermissionDenied
        );

        let mut wrong_action = request.clone();
        wrong_action.action = OwnerVaultActionV1::Delete as i32;
        assert_eq!(
            authorize_target(&store, &principal, &wrong_action)
                .expect_err("undeclared action must be denied"),
            OwnerVaultProvisioningRouteErrorV1::PermissionDenied
        );

        let challenge = [5; 32];
        let signing_key = SigningKey::from_slice(&[7; 32]).expect("signing key");
        let signature: Signature = signing_key.sign(&challenge);
        verify_fresh_proof(&store, &principal, &challenge, &signature.to_bytes())
            .expect("fresh device proof");
        let wrong_signature: Signature = SigningKey::from_slice(&[6; 32])
            .expect("wrong signing key")
            .sign(&challenge);
        assert_eq!(
            verify_fresh_proof(&store, &principal, &challenge, &wrong_signature.to_bytes(),)
                .expect_err("foreign device proof must be denied"),
            OwnerVaultProvisioningRouteErrorV1::PermissionDenied
        );

        let identity_epoch = store.current_identity_epoch().expect("identity epoch");
        store
            .revoke_browser_device(DEVICE, identity_epoch)
            .expect("revoke browser device");
        assert_eq!(
            authorize_target(&store, &principal, &request)
                .expect_err("revoked browser device must be denied"),
            OwnerVaultProvisioningRouteErrorV1::PermissionDenied
        );

        drop(store);
        std::fs::remove_dir_all(root).expect("remove fixture");
    }

    fn fixture() -> (PathBuf, SqliteControlStore, OwnerVaultClientPrincipalV1) {
        let root = unique_root();
        std::fs::create_dir_all(&root).expect("create fixture root");
        let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store");
        let signing_key = SigningKey::from_slice(&[7; 32]).expect("signing key");
        let public_key: [u8; 65] = signing_key
            .verifying_key()
            .to_sec1_point(false)
            .as_bytes()
            .try_into()
            .expect("SEC1 key");
        store
            .claim_initial_owner(&InitialOwnerIdentity::new(
                HUMAN_OWNER,
                "owner-device",
                public_key,
            ))
            .expect("claim owner");
        let identity_epoch = store.current_identity_epoch().expect("identity epoch");
        let enrollment = BrowserDeviceEnrollmentV1::new(BrowserDeviceEnrollmentInputV1 {
            owner_id: HUMAN_OWNER.to_owned(),
            device_id: DEVICE.to_owned(),
            credential_id: vec![8; 32],
            cose_public_key: vec![9; 16],
            browser_key_public_key: public_key.to_vec(),
            rp_id: "localhost".to_owned(),
            sign_count: 0,
            backup_eligible: false,
            backup_state: false,
        })
        .expect("browser enrollment");
        store
            .admit_browser_device(&enrollment, identity_epoch)
            .expect("admit browser device");
        let registration = ModuleRegistration::new(
            REGISTRATION,
            "integration.mail",
            "mail",
            [3; 32],
            ModuleRegistrationState::Pending,
            1,
        );
        let purpose = ModuleVaultPurposeRequestV1::new(
            REGISTRATION,
            CAPABILITY,
            PURPOSE,
            120,
            VaultSecretClassV1::ProviderCredential as u8,
            RuntimeVaultActionV1::Create as u8,
            1,
        );
        store
            .create_pending_registration_with_all_descriptor_requests(
                &registration,
                &[CAPABILITY.to_owned()],
                ModuleDescriptorRegistrationRequestsV1 {
                    storage: &[],
                    events: &[],
                    blobs: &[],
                    scheduler: &[],
                    vault_purposes: std::slice::from_ref(&purpose),
                    client_rpc_routes: &[],
                    client_blob_routes: &[],
                    client_realtime_routes: &[],
                    query_rpc_routes: &[],
                    request_rpc_routes: &[],
                    contract_dependencies: &[],
                },
            )
            .expect("record registration");
        store
            .approve_module_registration(REGISTRATION, &[CAPABILITY.to_owned()])
            .expect("approve capability");
        let principal = OwnerVaultClientPrincipalV1::new(
            HUMAN_OWNER,
            DEVICE,
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .expect("principal");
        (root, store, principal)
    }

    fn request() -> PrepareOwnerVaultProvisioningRequestV1 {
        let recipient = VaultResponseRecipientV1::generate();
        PrepareOwnerVaultProvisioningRequestV1 {
            operation_id: vec![1; 16],
            target_registration_id: REGISTRATION.to_owned(),
            capability_id: CAPABILITY.to_owned(),
            configuration_instance_id: "mail-account-1".to_owned(),
            purpose_id: PURPOSE.to_owned(),
            secret_class: OwnerVaultSecretClassV1::ProviderCredential as i32,
            action: OwnerVaultActionV1::Create as i32,
            secret_revision: 1,
            response_recipient_hpke_public_key_x25519: recipient.public_key().as_bytes().to_vec(),
        }
    }

    fn unique_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "makosh-owner-vault-authorization-{}-{nanos}",
            std::process::id()
        ))
    }
}

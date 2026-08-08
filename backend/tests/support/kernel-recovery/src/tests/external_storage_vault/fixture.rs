//! Trusted fixture state for one attested external Storage runtime.

use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use makosh_kernel_control_store::{
    ExternalRuntimeIdentity, ModuleRegistration, ModuleRegistrationState,
    PlatformStorageBindingInputV1, PlatformStorageBindingV1,
};
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_store_sqlcipher::VaultStore;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::runtime::external::sessions::ExternalRuntimeSessions;
use crate::service::runtime::VaultService;
use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::session::VaultTransportReplayGuard;
use crate::vault::{StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1};

use super::route::ExternalStorageVaultRoute;

const REGISTRATION_ID: &str = "registration_1";
const RUNTIME_ID: &str = "runtime_1";
const RUNTIME_GENERATION: u64 = 5;
const VAULT_GENERATION: u64 = 7;

pub(super) struct ExternalStorageVaultFixture {
    root: std::path::PathBuf,
    store: Arc<SqliteControlStore>,
    route: Option<ExternalStorageVaultRoute>,
    context: StorageVaultRouteContextV1,
}

impl ExternalStorageVaultFixture {
    pub(super) fn new() -> Self {
        let root = crate::tests::common::unique_target_root("makosh-external-storage-vault");
        let data_dir = private_directory(&root.join("kernel"));
        let vault_dir = private_directory(&root.join("vault"));
        let store = Arc::new(
            SqliteControlStore::create(&root.join("control.sqlite"), "kernel_main", 1)
                .expect("Control Store"),
        );
        let kernel_signer = FileDeviceSigner::open_or_create_for_instance(&data_dir)
            .expect("Kernel file signer")
            .0;
        let external_signer =
            SigningKey::from_bytes((&[41_u8; 32]).into()).expect("external runtime signing key");
        let external_public_key = external_signer
            .verifying_key()
            .to_sec1_point(false)
            .as_bytes()
            .try_into()
            .expect("uncompressed external runtime key");
        authorize_runtime(&store, external_public_key);
        store
            .record_platform_storage_binding(&durable_binding(1, 1, "runtime_principal_1"))
            .expect("initial durable Storage binding");
        let (sessions, session_id) = authenticate_runtime(&store, &external_signer);
        let (service, keys) = open_vault(&vault_dir);
        let context = StorageVaultRouteContextV1::new(
            "vault_main".to_owned(),
            VAULT_GENERATION,
            *keys.public_key().as_bytes(),
        )
        .expect("Vault route context");
        let route = ExternalStorageVaultRoute::new(
            data_dir,
            Arc::clone(&store),
            sessions,
            session_id,
            service,
            keys,
            kernel_signer.public_key_sec1(),
        );
        Self {
            root,
            store,
            route: Some(route),
            context,
        }
    }

    pub(super) fn binding(
        &self,
        role_epoch: u64,
        credential_revision: u64,
        runtime_principal: &str,
    ) -> StorageBindingV1 {
        let identity = StorageBindingIdentityV1::new(
            "storage_main".to_owned(),
            "makosh".to_owned(),
            "owner_1".to_owned(),
            REGISTRATION_ID.to_owned(),
            RUNTIME_ID.to_owned(),
        )
        .expect("Storage identity");
        let fences = StorageBindingFencesV1::new(
            1,
            RUNTIME_GENERATION,
            3,
            role_epoch,
            credential_revision,
            1,
        )
        .expect("Storage fences");
        let budgets = StorageEffectiveBudgetsV1::new(8, 5_000).expect("Storage budgets");
        let access = StorageBindingAccessV1::new(
            runtime_principal.to_owned(),
            "runtime_registration_1_5".to_owned(),
            budgets,
            [1; 32],
        )
        .expect("Storage access");
        StorageBindingV1::new(identity, fences, access).expect("Storage binding")
    }

    pub(super) fn take_adapter(&mut self) -> StorageVaultLeaseAdapterV1<ExternalStorageVaultRoute> {
        StorageVaultLeaseAdapterV1::new(
            self.route.take().expect("Storage route is taken once"),
            self.context.clone(),
        )
    }

    pub(super) fn suspend_runtime(&self) {
        self.store
            .transition_module_registration(REGISTRATION_ID, ModuleRegistrationState::Suspended)
            .expect("suspend external runtime registration");
    }

    pub(super) fn rotate_binding(&self) {
        self.store
            .begin_platform_storage_binding_revocation("registration_1", "storage.access", 1)
            .expect("reserve initial Storage binding for rotation");
        self.store
            .record_platform_storage_binding(&durable_binding(2, 2, "runtime_principal_2"))
            .expect("rotated durable Storage binding");
    }
}

impl Drop for ExternalStorageVaultFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn private_directory(path: &std::path::Path) -> std::path::PathBuf {
    std::fs::create_dir_all(path).expect("private directory");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .expect("private directory mode");
    path.to_owned()
}

fn authorize_runtime(store: &SqliteControlStore, external_public_key: [u8; 65]) {
    let registration = ModuleRegistration::new(
        REGISTRATION_ID,
        "module_1",
        "owner_1",
        [1; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&registration, &["vault.lease.resolve".to_owned()])
        .expect("pending external registration");
    store
        .approve_module_registration(REGISTRATION_ID, &["vault.lease.resolve".to_owned()])
        .expect("approved external registration");
    store
        .bind_external_runtime_identity(&ExternalRuntimeIdentity::new(
            REGISTRATION_ID,
            external_public_key,
        ))
        .expect("external runtime identity");
}

fn authenticate_runtime(
    store: &SqliteControlStore,
    signer: &SigningKey,
) -> (ExternalRuntimeSessions, String) {
    let mut sessions = ExternalRuntimeSessions::default();
    let digest = [2; 32];
    let challenge = sessions
        .begin(
            store,
            REGISTRATION_ID,
            RUNTIME_ID,
            RUNTIME_GENERATION,
            digest,
        )
        .expect("external runtime challenge");
    let signature: Signature = signer.sign(&proof_message(&challenge, digest));
    let session_id = sessions
        .complete(
            store,
            challenge.challenge_id(),
            signature.to_bytes().as_slice(),
        )
        .expect("external runtime session")
        .session_id()
        .to_owned();
    (sessions, session_id)
}

fn proof_message(
    challenge: &crate::runtime::external::sessions::RuntimeChallenge,
    digest: [u8; 32],
) -> Vec<u8> {
    let mut proof = b"makosh.external-runtime-session.v1\0".to_vec();
    for value in [challenge.kernel_instance_id(), REGISTRATION_ID, RUNTIME_ID] {
        proof.extend_from_slice(&(value.len() as u16).to_be_bytes());
        proof.extend_from_slice(value.as_bytes());
    }
    proof.extend_from_slice(&RUNTIME_GENERATION.to_be_bytes());
    proof.extend_from_slice(&challenge.grant_epoch().to_be_bytes());
    proof.extend_from_slice(&digest);
    proof.extend_from_slice(challenge.bytes());
    proof
}

fn open_vault(vault_dir: &std::path::Path) -> (VaultService, VaultTransportKeyPair) {
    let provider = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"));
    let key = provider.load_or_create().expect("Vault file key");
    VaultStore::initialize(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        "vault_main",
        &key,
    )
    .expect("Vault store");
    let store = VaultStore::open(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        &key,
    )
    .expect("reopen Vault store");
    (
        VaultService::new(store, VAULT_GENERATION).expect("Vault service"),
        VaultTransportKeyPair::generate(),
    )
}

fn durable_binding(
    revision: u64,
    credential_revision: u64,
    runtime_principal: &str,
) -> PlatformStorageBindingV1 {
    PlatformStorageBindingV1::new(PlatformStorageBindingInputV1 {
        registration_id: REGISTRATION_ID.to_owned(),
        capability_id: "storage.access".to_owned(),
        owner_id: "owner_1".to_owned(),
        binding_revision: revision,
        topology_revision: 1,
        storage_generation: 1,
        runtime_instance_id: RUNTIME_ID.to_owned(),
        runtime_generation: RUNTIME_GENERATION,
        grant_epoch: 3,
        role_epoch: revision,
        runtime_principal: runtime_principal.to_owned(),
        connection_budget: 8,
        statement_timeout_millis: 5_000,
        credential_lease_revision: credential_revision,
        storage_bundle_revision: 1,
        storage_bundle_digest: [1; 32],
    })
    .expect("durable Storage binding")
}

pub(super) fn replay_guard() -> VaultTransportReplayGuard {
    VaultTransportReplayGuard::new(VAULT_GENERATION)
}

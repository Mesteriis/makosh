//! Real encrypted Storage-to-Vault routing through the Kernel fence handler.

use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use makosh_kernel_control_store::{PlatformManagedProcessBinding, PlatformManagedProcessLaunch};
use makosh_runtime_protocol::v1::{
    ManagedRuntimeControlRequestV1,
    managed_runtime_control_request_v1::Operation as ManagedOperation,
};
use makosh_vault_key_provider::WrappingKeyProvider;
use makosh_vault_key_provider_file::FileWrappingKeyProvider;
use makosh_vault_protocol::LeaseAudienceV1;
use makosh_vault_store_sqlcipher::VaultStore;
use prost::Message;

use super::common::*;
use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::platform::vault::managed_route::KernelManagedVaultRouteHandler;
use crate::runtime::lifecycle::control::{
    ManagedRuntimeExpectation, ManagedRuntimeVaultRouteHandler, establish_channel, inbound, relay,
};
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelay;
use crate::service::runtime::VaultService;
use crate::storage_control::vault_route::InheritedVaultRoutePortV1;
use crate::transport::keys::VaultTransportKeyPair;
use crate::vault::{
    StoragePlatformCredentialBootstrapV1, StoragePlatformCredentialPurposeV1,
    StorageVaultRouteContextV1, complete_immediately,
};

#[test]
fn kernel_routes_storage_credential_bootstrap_through_a_live_vault_service() {
    let root = unique_target_root("makosh-storage-vault-composition");
    let data_dir = private_directory(&root.join("kernel"));
    let vault_dir = private_directory(&root.join("vault"));
    let store = Arc::new(configured_store(&root));
    let signer = FileDeviceSigner::open_or_create_for_instance(&data_dir)
        .expect("Kernel file signer")
        .0;
    let keys = VaultTransportKeyPair::generate();
    let public_key = *keys.public_key().as_bytes();
    initialize_vault(&vault_dir);

    let (vault_kernel, vault_child) = UnixStream::pair().expect("Vault inherited channel");
    let vault = spawn_vault(
        vault_child,
        vault_dir.clone(),
        keys,
        signer.public_key_sec1(),
    );
    let mut vault_channel = establish_channel(vault_kernel, &vault_expectation())
        .expect("Kernel accepts Vault descriptor");
    assert_vault_ready(&mut vault_channel);
    let relays = Arc::new(AtomicU64::new(0));
    let direct_relay = Arc::new(DirectVaultRelay {
        channel: Mutex::new(vault_channel),
        calls: Arc::clone(&relays),
    });
    let handler = KernelManagedVaultRouteHandler::new(store, &data_dir, direct_relay);

    let (kernel_storage, storage_child) = UnixStream::pair().expect("Storage inherited channel");
    let bridge = std::thread::spawn(move || bridge_storage_routes(kernel_storage, handler));
    let credential = bootstrap_storage_credential(storage_child, public_key);

    assert_eq!(
        credential.len(),
        64,
        "Vault generated an opaque 256-bit token"
    );
    assert!(credential.iter().all(u8::is_ascii_hexdigit));
    assert_eq!(
        relays.load(Ordering::Acquire),
        6,
        "bounded lease route sequence"
    );

    bridge.join().expect("Storage-to-Kernel bridge");
    assert!(vault.join().expect("Vault service thread").is_err());
    std::fs::remove_dir_all(root).expect("remove composition fixture");
}

fn private_directory(path: &std::path::Path) -> std::path::PathBuf {
    std::fs::create_dir_all(path).expect("private directory");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .expect("private directory mode");
    path.to_owned()
}

fn configured_store(root: &std::path::Path) -> SqliteControlStore {
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "kernel-main", 1)
        .expect("Control Store");
    store
        .record_platform_managed_process_binding(&PlatformManagedProcessBinding::new(
            "vault",
            1,
            "distribution",
            "vault-runtime",
            [1; 32],
            [2; 32],
            None,
        ))
        .expect("Vault release binding");
    store
        .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
            "vault", 1, 1, 1, 1,
        ))
        .expect("Vault launch");
    store
        .record_platform_managed_process_binding(&PlatformManagedProcessBinding::new(
            "storage-control",
            1,
            "distribution",
            "storage-runtime",
            [4; 32],
            [3; 32],
            None,
        ))
        .expect("Storage release binding");
    store
        .record_platform_managed_process_launch(&PlatformManagedProcessLaunch::new(
            "storage-control",
            1,
            1,
            4,
            1,
        ))
        .expect("Storage launch");
    store
}

fn initialize_vault(vault_dir: &std::path::Path) {
    let provider = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"));
    let key = provider.load_or_create().expect("Vault file key");
    VaultStore::initialize(
        &vault_dir.join("vault.db"),
        &vault_dir.join("vault.anchor"),
        "vault-main",
        &key,
    )
    .expect("Vault store");
}

fn spawn_vault(
    channel: UnixStream,
    vault_dir: std::path::PathBuf,
    keys: VaultTransportKeyPair,
    authorization_key: [u8; 65],
) -> std::thread::JoinHandle<Result<(), String>> {
    std::thread::spawn(move || {
        let provider = FileWrappingKeyProvider::new(&vault_dir.join("platform-wrapping-key.bin"));
        let key = provider.load_or_create().expect("reopen Vault file key");
        let store = VaultStore::open(
            &vault_dir.join("vault.db"),
            &vault_dir.join("vault.anchor"),
            &key,
        )
        .expect("reopen Vault store");
        let mut service = VaultService::new(store, 1).expect("Vault service");
        let channel = crate::control::inherited::describe(
            channel,
            vault_descriptor().encode_to_vec(),
            Vec::new(),
        )?;
        crate::control::runtime::serve_on_channel(channel, &mut service, &keys, authorization_key)
    })
}

fn vault_expectation() -> ManagedRuntimeExpectation {
    let descriptor = vault_descriptor();
    ManagedRuntimeExpectation::new(
        "vault",
        "vault-runtime",
        "vault",
        1,
        1,
        Sha256::digest(descriptor.encode_to_vec()).into(),
        None,
    )
}

fn assert_vault_ready(channel: &mut UnixStream) {
    let request = ManagedRuntimeControlRequestV1::decode(
        read_frame(channel)
            .expect("read Vault ready frame")
            .as_slice(),
    )
    .expect("decode Vault ready frame");
    let Some(ManagedOperation::Ready(ready)) = request.operation else {
        panic!("Vault sends a typed ready frame");
    };
    assert_eq!(ready.registration_id, "vault");
    assert_eq!(ready.runtime_generation, 1);
    assert_eq!(ready.grant_epoch, 1);
}

fn vault_descriptor() -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "vault".to_owned(),
        owner_id: "vault".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "test".to_owned(),
        ..Default::default()
    }
}

fn bootstrap_storage_credential(channel: UnixStream, public_key: [u8; 32]) -> Vec<u8> {
    let context = StorageVaultRouteContextV1::new("vault-main".to_owned(), 1, public_key)
        .expect("Storage Vault context");
    let audience = LeaseAudienceV1::new(
        "storage-control".to_owned(),
        "storage-control".to_owned(),
        4,
        1,
    )
    .expect("Storage audience");
    let mut bootstrap = StoragePlatformCredentialBootstrapV1::new(
        InheritedVaultRoutePortV1::new(channel),
        context,
        audience,
        StoragePlatformCredentialPurposeV1::PgBouncerAdmin,
        "storage-main".to_owned(),
        1,
    )
    .expect("Storage credential bootstrap");
    complete_immediately(bootstrap.ensure_and_resolve())
        .expect("synchronous Storage route")
        .expect("Vault-generated Storage credential")
        .to_vec()
}

fn bridge_storage_routes(mut channel: UnixStream, handler: KernelManagedVaultRouteHandler) {
    while let Ok(frame) = read_frame(&mut channel) {
        let request = ManagedRuntimeControlRequestV1::decode(frame.as_slice())
            .expect("typed Storage managed control request");
        let route = match request.operation {
            Some(ManagedOperation::RouteVaultCiphertext(request)) => request.route,
            _ => None,
        };
        let response =
            route.map(|route| handler.route_vault_ciphertext(&storage_expectation(), route));
        write_response(&mut channel, response);
    }
}

fn storage_expectation() -> ManagedRuntimeExpectation {
    ManagedRuntimeExpectation::new(
        "storage-control",
        "storage-control",
        "storage",
        4,
        1,
        [3; 32],
        None,
    )
}

fn write_response(
    channel: &mut UnixStream,
    result: Option<Result<makosh_runtime_protocol::v1::VaultCiphertextResponseV1, String>>,
) {
    let result = result.unwrap_or_else(|| Err("managed Vault route is unavailable".to_owned()));
    if let Err(error) = &result
        && std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some()
    {
        eprintln!("developer_storage_vault_route_error={error}");
    }
    let response = inbound::vault_route_response(result);
    write_frame(channel, &response.encode_to_vec());
}

fn read_frame(channel: &mut UnixStream) -> Result<Vec<u8>, std::io::Error> {
    let mut length = 0_usize;
    for shift in (0..35).step_by(7) {
        let mut byte = [0; 1];
        channel.read_exact(&mut byte)?;
        length |= usize::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let mut frame = vec![0; length];
            channel.read_exact(&mut frame)?;
            return Ok(frame);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "frame length",
    ))
}

fn write_frame(channel: &mut UnixStream, bytes: &[u8]) {
    let mut length = u32::try_from(bytes.len()).expect("bounded composition frame");
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    channel
        .write_all(&prefix)
        .and_then(|_| channel.write_all(bytes))
        .and_then(|_| channel.flush())
        .expect("write composition frame");
}

struct DirectVaultRelay {
    channel: Mutex<UnixStream>,
    calls: Arc<AtomicU64>,
}

impl ManagedRuntimeRelay for DirectVaultRelay {
    fn relay(&self, registration_id: &str, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        if registration_id != "vault" {
            return Err("unexpected managed runtime".to_owned());
        }
        self.calls.fetch_add(1, Ordering::Release);
        let mut channel = self
            .channel
            .lock()
            .map_err(|_| "Vault relay lock is unavailable".to_owned())?;
        relay(&mut channel, &payload)
    }
}

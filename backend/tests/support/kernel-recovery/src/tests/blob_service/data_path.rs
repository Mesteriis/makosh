//! End-to-end direct Blob data socket proof through the live managed Vault.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::v1::{
    BlobBackupClassV1, BlobDataOperationV1, BlobDataReadRangeRequestV1, BlobDataRequestV1,
    BlobDataSessionGrantV1, BlobDataWriteRequestV1, blob_data_request_v1::Operation,
};
use prost::Message;
use sha2::{Digest, Sha256};

use super::super::common::*;
use super::live_vault_launch::{installed_release, short_runtime_directory};
use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::platform::blob::{binding, launch};
use crate::tests::platform_vault::live as vault_fixture;

#[test]
#[ignore = "builds and launches the real Blob and Vault runtime binaries"]
fn managed_blob_writes_and_reads_through_the_live_vault_route() {
    let root = unique_target_root("makosh-blob-data-path");
    let data = vault_fixture::private_directory(short_runtime_directory());
    let runtime = short_runtime_directory();
    vault_fixture::initialize_vault(&data);
    let release = installed_release(&root);
    let store = Arc::new(
        SqliteControlStore::create(&root.join("control.sqlite"), "kernel-main", 1)
            .expect("create Control Store"),
    );
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));

    vault_fixture::bind_and_start(&supervisor, &store, &data, release.kernel());
    binding::bind_installed_release(&store, release.kernel()).expect("bind Blob release");
    let blob_generation =
        launch::start_from_kernel(&supervisor, &store, release.kernel(), &data, &runtime)
            .expect("start Blob service");
    let socket = launch::data_socket_path(&data);
    let signer = FileDeviceSigner::open_for_instance(&data).expect("open Kernel signer");

    let write = request(
        &signer,
        BlobDataOperationV1::BlobDataOperationWriteV1,
        [1; 16],
        [3; 32],
        blob_generation,
        BlobCallerIdentityV1 {
            registration_id: "blob",
            capability_id: "blob-content.write",
            runtime_instance_id: "blob-runtime-v1",
            runtime_generation: 1,
            grant_epoch: 1,
        },
        b"hello",
    );
    let written =
        exchange(&socket, write.clone()).unwrap_or_else(|error| failure(&supervisor, error));
    assert!(written.accepted && written.error_code.is_empty());
    assert_denied(exchange(&socket, write).unwrap_or_else(|error| failure(&supervisor, error)));
    assert_cross_custody_read_denied(&socket, &signer, blob_generation, &supervisor);
    assert_stale_generation_denied(&socket, &signer, blob_generation, &supervisor);
    let read = exchange(
        &socket,
        request(
            &signer,
            BlobDataOperationV1::BlobDataOperationReadRangeV1,
            [2; 16],
            [4; 32],
            blob_generation,
            BlobCallerIdentityV1 {
                registration_id: "blob-successor",
                capability_id: "blob-content.read",
                runtime_instance_id: "blob-runtime-v2",
                runtime_generation: 8,
                grant_epoch: 9,
            },
            b"hello",
        ),
    )
    .unwrap_or_else(|error| failure(&supervisor, error));
    assert!(read.accepted && read.error_code.is_empty());
    assert_eq!(read.plaintext, b"hello");

    supervisor.shutdown().expect("stop managed children");
    std::fs::remove_dir_all(root).expect("remove fixture directory");
    std::fs::remove_dir_all(data).expect("remove short Kernel data directory");
    std::fs::remove_dir_all(runtime).expect("remove short runtime directory");
}

fn request(
    signer: &FileDeviceSigner,
    operation: BlobDataOperationV1,
    session_id: [u8; 16],
    binding: [u8; 32],
    blob_generation: u64,
    caller: BlobCallerIdentityV1<'_>,
    plaintext: &[u8],
) -> BlobDataRequestV1 {
    let expected_plaintext_sha256 =
        (!plaintext.is_empty()).then(|| Sha256::digest(plaintext).into());
    let grant = signed_grant(
        signer,
        operation,
        session_id,
        &binding,
        blob_generation,
        caller,
        expected_plaintext_sha256,
    );
    let operation = match operation {
        BlobDataOperationV1::BlobDataOperationWriteV1 => Operation::Write(BlobDataWriteRequestV1 {
            plaintext: plaintext.to_vec(),
        }),
        BlobDataOperationV1::BlobDataOperationReadRangeV1 => {
            Operation::ReadRange(BlobDataReadRangeRequestV1 {
                start: 0,
                end_exclusive: 5,
            })
        }
        BlobDataOperationV1::BlobDataOperationCustodyTransferV1 => unreachable!(),
        BlobDataOperationV1::BlobDataOperationUnspecifiedV1 => unreachable!(),
    };
    BlobDataRequestV1 {
        grant: Some(grant),
        channel_binding: binding.to_vec(),
        operation: Some(operation),
    }
}

fn signed_grant(
    signer: &FileDeviceSigner,
    operation: BlobDataOperationV1,
    session_id: [u8; 16],
    binding: &[u8; 32],
    blob_generation: u64,
    caller: BlobCallerIdentityV1<'_>,
    expected_plaintext_sha256: Option<[u8; 32]>,
) -> BlobDataSessionGrantV1 {
    let mut grant = BlobDataSessionGrantV1 {
        major: 1,
        kernel_instance_id: "kernel-main".to_owned(),
        session_id: session_id.to_vec(),
        channel_binding_sha256: Sha256::digest(binding).to_vec(),
        owner_id: "owner-1".to_owned(),
        registration_id: caller.registration_id.to_owned(),
        capability_id: caller.capability_id.to_owned(),
        runtime_instance_id: caller.runtime_instance_id.to_owned(),
        runtime_generation: caller.runtime_generation,
        grant_epoch: caller.grant_epoch,
        key_revision: 1,
        quota_max_bytes: 1024,
        custody_scope_id: "owner-1.content.v1".to_owned(),
        reference_id: [9; 16].to_vec(),
        declared_size: 5,
        reference_expires_at_unix_ms: 0,
        backup_class: BlobBackupClassV1::BlobBackupClassRequiredV1 as i32,
        operation: operation as i32,
        expires_at_unix_ms: unix_ms() + 10_000,
        kernel_authorization_signature_raw: Vec::new(),
        blob_runtime_generation: blob_generation,
        expected_plaintext_sha256: expected_plaintext_sha256.map_or_else(Vec::new, |v| v.to_vec()),
    };
    let mut message = b"makosh.blob-data-session.v1\0".to_vec();
    message.extend_from_slice(&grant.encode_to_vec());
    grant.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
    grant
}

#[derive(Clone, Copy)]
struct BlobCallerIdentityV1<'a> {
    registration_id: &'a str,
    capability_id: &'a str,
    runtime_instance_id: &'a str,
    runtime_generation: u64,
    grant_epoch: u64,
}

fn exchange(
    socket: &std::path::Path,
    request: BlobDataRequestV1,
) -> Result<makosh_runtime_protocol::v1::BlobDataResponseV1, String> {
    let mut stream = UnixStream::connect(socket).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    write_frame(&mut stream, &request.encode_to_vec())?;
    makosh_runtime_protocol::v1::BlobDataResponseV1::decode(read_frame(&mut stream)?.as_slice())
        .map_err(|error| error.to_string())
}

fn failure(supervisor: &ManagedRuntimeSupervisor, error: String) -> ! {
    std::thread::sleep(Duration::from_millis(100));
    panic!(
        "Blob direct data request failed: {error}; blob active: {:?}; blob failure: {:?}; Vault active: {:?}; Vault failure: {:?}",
        supervisor.is_active("blob"),
        supervisor.last_failure("blob"),
        supervisor.is_active("vault"),
        supervisor.last_failure("vault"),
    );
}

fn assert_denied(response: makosh_runtime_protocol::v1::BlobDataResponseV1) {
    assert!(!response.accepted);
    assert_eq!(response.error_code, "data_request_denied");
}

fn assert_stale_generation_denied(
    socket: &std::path::Path,
    signer: &FileDeviceSigner,
    blob_generation: u64,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let stale = request(
        signer,
        BlobDataOperationV1::BlobDataOperationWriteV1,
        [3; 16],
        [5; 32],
        blob_generation + 1,
        BlobCallerIdentityV1 {
            registration_id: "blob",
            capability_id: "blob-content.write",
            runtime_instance_id: "blob-runtime-v1",
            runtime_generation: 1,
            grant_epoch: 1,
        },
        b"hello",
    );
    assert_denied(exchange(socket, stale).unwrap_or_else(|error| failure(supervisor, error)));
}

fn assert_cross_custody_read_denied(
    socket: &std::path::Path,
    signer: &FileDeviceSigner,
    blob_generation: u64,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let mut cross_custody = request(
        signer,
        BlobDataOperationV1::BlobDataOperationReadRangeV1,
        [5; 16],
        [6; 32],
        blob_generation,
        BlobCallerIdentityV1 {
            registration_id: "blob",
            capability_id: "blob-content.read",
            runtime_instance_id: "blob-runtime-v1",
            runtime_generation: 1,
            grant_epoch: 1,
        },
        b"",
    );
    let grant = cross_custody.grant.as_mut().expect("Blob grant");
    grant.custody_scope_id = "owner-1.other-content.v1".to_owned();
    resign_grant(signer, grant);
    assert_denied(
        exchange(socket, cross_custody).unwrap_or_else(|error| failure(supervisor, error)),
    );
}

fn resign_grant(signer: &FileDeviceSigner, grant: &mut BlobDataSessionGrantV1) {
    grant.kernel_authorization_signature_raw.clear();
    let mut message = b"makosh.blob-data-session.v1\0".to_vec();
    message.extend_from_slice(&grant.encode_to_vec());
    grant.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
}

fn write_frame(stream: &mut UnixStream, bytes: &[u8]) -> Result<(), String> {
    let mut value = bytes.len() as u64;
    let mut prefix = Vec::new();
    while value >= 0x80 {
        prefix.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    prefix.push(value as u8);
    stream
        .write_all(&prefix)
        .map_err(|error| error.to_string())?;
    stream.write_all(bytes).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut length = 0_u64;
    for shift in (0..35).step_by(7) {
        let mut byte = [0_u8; 1];
        stream
            .read_exact(&mut byte)
            .map_err(|error| error.to_string())?;
        length |= u64::from(byte[0] & 0x7f) << shift;
        if byte[0] & 0x80 == 0 {
            let mut bytes = vec![0; length as usize];
            stream
                .read_exact(&mut bytes)
                .map_err(|error| error.to_string())?;
            return Ok(bytes);
        }
    }
    Err("Blob frame length is invalid".to_owned())
}

fn unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .try_into()
        .expect("millisecond clock fits u64")
}

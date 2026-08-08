#[cfg(test)]
#[path = "../../../../src/platform/vault/runtime/src/bootstrap/mod.rs"]
mod bootstrap;

#[cfg(test)]
mod control {
    #[path = "../../../../../src/platform/vault/runtime/src/control/inherited.rs"]
    pub(crate) mod inherited;
    #[path = "../../../../../src/platform/vault/runtime/src/control/runtime.rs"]
    pub(crate) mod runtime;
    #[path = "../../../../../src/platform/vault/runtime/src/control/socket.rs"]
    pub(crate) mod socket;
}

#[cfg(test)]
mod service {
    #[path = "../../../../../src/platform/vault/runtime/src/service/leases.rs"]
    pub(crate) mod leases;
    #[path = "../../../../../src/platform/vault/runtime/src/service/runtime.rs"]
    pub(crate) mod runtime;
}

#[cfg(test)]
mod transport {
    #[path = "../../../../../src/platform/vault/runtime/src/transport/keys.rs"]
    pub(crate) mod keys;
    #[path = "../../../../../src/platform/vault/runtime/src/transport/response.rs"]
    pub(crate) mod response;
    #[path = "../../../../../src/platform/vault/runtime/src/transport/route.rs"]
    pub(crate) mod route;
    #[path = "../../../../../src/platform/vault/runtime/src/transport/session.rs"]
    pub(crate) mod session;
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod foundation {
    use std::io::{Read, Write};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::os::unix::net::UnixStream;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::time::Duration;

    use makosh_runtime_protocol::v1::{
        GetVaultRuntimeStatusRequestV1, VaultRuntimeControlRequestV1,
        VaultRuntimeControlResponseV1, VaultRuntimeStateV1,
        vault_runtime_control_request_v1::Operation,
        vault_runtime_control_response_v1::Result as ResponseResult,
    };
    use makosh_vault_key_provider::WrappingKeyProvider;
    use makosh_vault_key_provider_file::FileWrappingKeyProvider;
    use makosh_vault_protocol::{
        LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultLeaseIssueRequestV1,
        VaultProtocolError, VaultPurposeRequestV1,
    };
    use makosh_vault_store_sqlcipher::{SecretRecordId, SecretRecordScope, VaultStore};
    use prost::Message;
    use tempfile::TempDir;

    use crate::service::leases::{LeaseError, LeaseManager};
    use crate::transport::keys::VaultTransportKeyPair;

    #[test]
    fn file_wrapping_key_opens_an_encrypted_sqlcipher_store_without_plaintext_metadata() {
        let temporary = TempDir::new().expect("temporary Vault directory");
        std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
            .expect("private temporary Vault directory");
        let key_path = temporary.path().join("platform-wrapping-key.bin");
        let database_path = temporary.path().join("vault.db");
        let anchor_path = temporary.path().join("vault.anchor");
        let provider = FileWrappingKeyProvider::new(&key_path);
        let key = provider.load_or_create().expect("file wrapping key");

        let store =
            VaultStore::initialize(&database_path, &anchor_path, "vault-instance-marker", &key)
                .expect("encrypted Vault store initialization");
        assert_eq!(store.instance_id(), "vault-instance-marker");
        assert_eq!(
            key_path.metadata().expect("key metadata").mode() & 0o777,
            0o600
        );
        assert_eq!(
            database_path.metadata().expect("database metadata").mode() & 0o777,
            0o600
        );
        assert_eq!(
            anchor_path.metadata().expect("anchor metadata").mode() & 0o777,
            0o600
        );

        let reopened_key = provider.load_or_create().expect("same wrapping key");
        assert_eq!(key.as_bytes(), reopened_key.as_bytes());
        let reopened = VaultStore::open(&database_path, &anchor_path, &reopened_key)
            .expect("reopen SQLCipher store");
        assert_eq!(reopened.instance_id(), "vault-instance-marker");
        let record_id = store_test_credential(&reopened);
        assert_test_credential_is_encrypted(&database_path);
        assert_wrong_scope_is_rejected(&reopened, &record_id);
        assert!(
            VaultStore::open(
                &database_path,
                &anchor_path,
                &makosh_vault_key_provider::WrappingKey::from_bytes([17; 32]),
            )
            .is_err()
        );
        tamper_anchor_and_assert_rejected(&database_path, &anchor_path, &reopened_key);
    }

    fn store_test_credential(store: &VaultStore) -> SecretRecordId {
        let record_request = VaultPurposeRequestV1::new(
            "mail.credential".to_owned(),
            "account-a".to_owned(),
            vec![SecretClassV1::ProviderCredential],
            vec![VaultActionV1::Resolve, VaultActionV1::Create],
            60,
        )
        .expect("typed record purpose");
        let record_scope = SecretRecordScope::new(
            "mail".to_owned(),
            &record_request,
            SecretClassV1::ProviderCredential,
            1,
        )
        .expect("typed record scope");
        let record_id = store
            .store_secret(&record_scope, b"credential-material-marker")
            .expect("authenticated record envelope");
        assert_eq!(
            store
                .resolve_scoped_secret(&record_id, &record_scope)
                .expect("authenticated record resolution")
                .as_slice(),
            b"credential-material-marker"
        );
        record_id
    }

    fn assert_test_credential_is_encrypted(database_path: &std::path::Path) {
        assert!(
            !std::fs::read(database_path)
                .expect("encrypted database bytes")
                .windows("vault-instance-marker".len())
                .any(|window| window == b"vault-instance-marker")
        );
        assert!(
            !std::fs::read(database_path)
                .expect("encrypted database bytes")
                .windows("credential-material-marker".len())
                .any(|window| window == b"credential-material-marker")
        );
    }

    fn assert_wrong_scope_is_rejected(store: &VaultStore, record_id: &SecretRecordId) {
        let wrong_scope = SecretRecordScope::new(
            "mail".to_owned(),
            &VaultPurposeRequestV1::new(
                "mail.credential".to_owned(),
                "account-b".to_owned(),
                vec![SecretClassV1::ProviderCredential],
                vec![VaultActionV1::Resolve],
                60,
            )
            .expect("wrong typed purpose"),
            SecretClassV1::ProviderCredential,
            1,
        )
        .expect("wrong typed scope");
        assert!(
            store
                .resolve_scoped_secret(record_id, &wrong_scope)
                .is_err()
        );
    }

    fn tamper_anchor_and_assert_rejected(
        database_path: &std::path::Path,
        anchor_path: &std::path::Path,
        wrapping_key: &makosh_vault_key_provider::WrappingKey,
    ) {
        let mut tampered_anchor = std::fs::read(anchor_path).expect("anchor bytes");
        let last_byte = tampered_anchor.last_mut().expect("nonempty anchor");
        *last_byte ^= 0x01;
        std::fs::write(anchor_path, tampered_anchor).expect("tamper synthetic anchor");
        assert!(VaultStore::open(database_path, anchor_path, wrapping_key).is_err());
    }

    #[test]
    fn file_adapter_and_public_contract_fail_closed_for_symlinks_and_ambiguous_scope() {
        let temporary = TempDir::new().expect("temporary Vault directory");
        std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
            .expect("private temporary Vault directory");
        let target = temporary.path().join("target");
        std::fs::write(&target, [0; 32]).expect("synthetic wrapping-key target");
        std::os::unix::fs::symlink(&target, temporary.path().join("platform-wrapping-key.bin"))
            .expect("synthetic symlink");
        assert!(
            FileWrappingKeyProvider::new(&temporary.path().join("platform-wrapping-key.bin"))
                .load_or_create()
                .is_err()
        );
        assert_eq!(
            VaultPurposeRequestV1::new(
                "mail.credential".to_owned(),
                "account-a".to_owned(),
                vec![
                    SecretClassV1::ProviderCredential,
                    SecretClassV1::ProviderCredential
                ],
                vec![VaultActionV1::Resolve],
                60,
            ),
            Err(VaultProtocolError::InvalidSecretClasses),
        );
    }

    #[test]
    fn credential_lease_is_memory_only_and_fenced_by_audience_epoch_and_generation() {
        let audience = LeaseAudienceV1::new(
            "registration-mail".to_owned(),
            "runtime-mail-1".to_owned(),
            1,
            7,
        )
        .expect("typed audience");
        let mut leases = LeaseManager::new("vault-instance".to_owned(), 3).expect("lease manager");
        let lease = leases
            .issue(lease_request(3, 1, &audience), 100)
            .expect("issue lease");

        assert_eq!(
            leases.consume_once(lease.lease_id(), &audience, 101),
            Ok(lease.clone())
        );
        assert_eq!(
            leases.consume_once(lease.lease_id(), &audience, 102),
            Err(LeaseError::UnknownOrInvalidatedLease)
        );
        assert_lease_fences(&mut leases, &audience, &lease);
        assert_lease_expiration(&mut leases, &audience);
    }

    fn lease_request(
        generation: u64,
        grant_epoch: u64,
        audience: &LeaseAudienceV1,
    ) -> VaultLeaseIssueRequestV1 {
        VaultLeaseIssueRequestV1::new(
            "vault-instance".to_owned(),
            generation,
            grant_epoch,
            "mail".to_owned(),
            VaultPurposeRequestV1::new(
                "mail.credential".to_owned(),
                "account-a".to_owned(),
                vec![SecretClassV1::ProviderCredential],
                vec![VaultActionV1::Resolve],
                60,
            )
            .expect("renewed purpose"),
            audience.to_owned(),
        )
        .expect("typed lease request")
    }

    fn assert_lease_fences(
        leases: &mut LeaseManager,
        audience: &LeaseAudienceV1,
        original: &makosh_vault_protocol::CredentialLeaseV1,
    ) {
        let renewed = leases
            .issue(lease_request(3, 1, audience), 200)
            .expect("new lease");
        let changed_epoch = LeaseAudienceV1::new(
            "registration-mail".to_owned(),
            "runtime-mail-1".to_owned(),
            1,
            8,
        )
        .expect("changed epoch audience");
        assert_eq!(
            leases.consume_once(renewed.lease_id(), &changed_epoch, 201),
            Err(LeaseError::AudienceOrGrantEpochMismatch)
        );
        leases.invalidate_audience(audience);
        assert_eq!(
            leases.consume_once(renewed.lease_id(), audience, 201),
            Err(LeaseError::UnknownOrInvalidatedLease)
        );

        leases.advance_generation(4).expect("new generation");
        assert_eq!(
            leases.consume_once(original.lease_id(), audience, 202),
            Err(LeaseError::UnknownOrInvalidatedLease)
        );
    }

    fn assert_lease_expiration(leases: &mut LeaseManager, audience: &LeaseAudienceV1) {
        let expiring = leases
            .issue(lease_request(4, 2, audience), 300)
            .expect("expiring lease");
        assert_eq!(
            leases.consume_once(expiring.lease_id(), audience, 360),
            Err(LeaseError::ExpiredLease)
        );
    }

    #[test]
    fn private_vault_socket_exposes_only_sanitized_runtime_status() {
        let temporary = TempDir::new().expect("temporary runtime directory");
        std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
            .expect("private temporary runtime directory");
        let runtime_dir = temporary.path().to_owned();
        let socket_path = runtime_dir.join("vault.sock");
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let server_shutdown = Arc::clone(&shutdown_requested);
        let server_thread = std::thread::spawn(move || {
            let keys = VaultTransportKeyPair::generate();
            crate::control::socket::serve(&runtime_dir, 9, &keys, &server_shutdown)
                .expect("private status server")
        });
        let request = VaultRuntimeControlRequestV1 {
            operation: Some(Operation::GetStatus(GetVaultRuntimeStatusRequestV1 {})),
        }
        .encode_to_vec();
        let bytes = request_status_with_retry(&socket_path, &request);
        let response = VaultRuntimeControlResponseV1::decode(bytes.as_slice())
            .expect("typed Vault status response");
        assert!(
            response.error_code.is_empty(),
            "private Vault status request failed: {:?}",
            response
        );
        match response.result {
            Some(ResponseResult::Status(status)) => {
                assert_eq!(status.state, VaultRuntimeStateV1::Ready as i32);
                assert_eq!(status.vault_runtime_generation, 9);
                assert_eq!(status.hpke_public_key_x25519.len(), 32);
                assert!(status.blocker_code.is_empty());
            }
            _ => panic!("expected Vault status response"),
        }
        shutdown_requested.store(true, Ordering::Release);
        server_thread.join().expect("private status server");
        assert!(!socket_path.exists(), "Vault socket is removed on shutdown");
    }

    fn connect_private_socket(path: &std::path::Path) -> UnixStream {
        for _ in 0..100 {
            match UnixStream::connect(path) {
                Ok(stream) => return stream,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                    ) =>
                {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("connect private Vault socket: {error}"),
            }
        }
        panic!("private Vault socket was not created");
    }

    fn request_status_with_retry(path: &std::path::Path, request: &[u8]) -> Vec<u8> {
        for _ in 0..8 {
            let mut client = connect_private_socket(path);
            if write_framed(&mut client, request).is_ok()
                && let Ok(response) = read_framed(&mut client)
            {
                return response;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("private Vault socket did not return a status response");
    }

    fn write_framed(stream: &mut UnixStream, bytes: &[u8]) -> std::io::Result<()> {
        let mut length = u32::try_from(bytes.len()).expect("bounded test frame");
        while length >= 0x80 {
            stream.write_all(&[(length as u8 & 0x7f) | 0x80])?;
            length >>= 7;
        }
        stream.write_all(&[length as u8])?;
        stream.write_all(bytes)?;
        stream.flush()
    }

    fn read_framed(stream: &mut UnixStream) -> std::io::Result<Vec<u8>> {
        let mut length = 0_u64;
        for shift in (0..35).step_by(7) {
            let mut byte = [0_u8; 1];
            stream.read_exact(&mut byte)?;
            length |= u64::from(byte[0] & 0x7f) << shift;
            if byte[0] & 0x80 == 0 {
                let mut bytes = vec![0; usize::try_from(length).expect("bounded test frame")];
                stream.read_exact(&mut bytes)?;
                return Ok(bytes);
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid test frame length",
        ))
    }
}

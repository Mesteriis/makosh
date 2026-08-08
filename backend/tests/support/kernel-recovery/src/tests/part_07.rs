use super::common::*;

#[test]
fn revoked_external_runtime_session_cannot_route_ciphertext_to_vault() {
    let root = unique_target_root("makosh-vault-route-session");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create control store");
    let signing_key = SigningKey::from_bytes((&[41_u8; 32]).into()).expect("test signing key");
    let public_key: [u8; 65] = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed public key");
    prepare_authorized_runtime(&store, public_key);

    let mut sessions = ExternalRuntimeSessions::default();
    let session_id = complete_session(&mut sessions, &store, &signing_key);
    let storage_runtime = sessions
        .authorize_storage_binding(&store, &session_id, "vault.lease.resolve")
        .expect("authorize exact external runtime session");
    assert_eq!(storage_runtime.registration_id(), "registration-1");
    assert_eq!(storage_runtime.runtime_id(), "runtime-1");
    assert_eq!(storage_runtime.runtime_generation(), 5);
    assert_eq!(storage_runtime.grant_epoch(), 3);
    assert!(
        sessions
            .authorize_vault_route(&store, &session_id, 7, current_route(3, 7))
            .is_ok()
    );
    let mut forged_generation = current_route(3, 7);
    forged_generation.caller_runtime_generation = 6;
    assert!(
        sessions
            .authorize_vault_route(&store, &session_id, 7, forged_generation)
            .is_err()
    );

    store
        .transition_module_registration("registration-1", ModuleRegistrationState::Suspended)
        .expect("revoke runtime grants");
    assert!(
        sessions
            .authorize_vault_route(&store, &session_id, 7, current_route(3, 7))
            .is_err()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn prepare_authorized_runtime(store: &SqliteControlStore, public_key: [u8; 65]) {
    let registration = ModuleRegistration::new(
        "registration-1",
        "module-1",
        "owner-1",
        [1; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&registration, &["vault.lease.resolve".to_owned()])
        .expect("create registration");
    store
        .approve_module_registration("registration-1", &["vault.lease.resolve".to_owned()])
        .expect("approve Vault route");
    store
        .bind_external_runtime_identity(&ExternalRuntimeIdentity::new("registration-1", public_key))
        .expect("bind runtime key");
}

fn complete_session(
    sessions: &mut ExternalRuntimeSessions,
    store: &SqliteControlStore,
    signing_key: &SigningKey,
) -> String {
    let digest = [2; 32];
    let challenge = sessions
        .begin(store, "registration-1", "runtime-1", 5, digest)
        .expect("begin session");
    let signature: Signature = signing_key.sign(&runtime_proof(&challenge, digest));
    sessions
        .complete(
            store,
            challenge.challenge_id(),
            signature.to_bytes().as_slice(),
        )
        .expect("complete session")
        .session_id()
        .to_owned()
}

fn runtime_proof(
    challenge: &crate::runtime::external::sessions::RuntimeChallenge,
    digest: [u8; 32],
) -> Vec<u8> {
    let mut proof = b"makosh.external-runtime-session.v1\0".to_vec();
    for value in [
        challenge.kernel_instance_id(),
        "registration-1",
        "runtime-1",
    ] {
        proof.extend_from_slice(&(value.len() as u16).to_be_bytes());
        proof.extend_from_slice(value.as_bytes());
    }
    proof.extend_from_slice(&5_u64.to_be_bytes());
    proof.extend_from_slice(&challenge.grant_epoch().to_be_bytes());
    proof.extend_from_slice(&digest);
    proof.extend_from_slice(challenge.bytes());
    proof
}

fn current_route(grant_epoch: u64, vault_runtime_generation: u64) -> VaultCiphertextRouteV1 {
    VaultCiphertextRouteV1 {
        major: 1,
        registration_id: "registration-1".to_owned(),
        runtime_instance_id: "runtime-1".to_owned(),
        caller_runtime_generation: 5,
        vault_runtime_generation,
        grant_epoch,
        request_id: vec![3; 16],
        operation_digest_sha256: vec![4; 32],
        direction: VaultCiphertextRouteDirectionV1::ToVault as i32,
        hpke_encapped_key: vec![5; 32],
        ciphertext: vec![6],
        hpke_authentication_tag: vec![7; 16],
        response_recipient_hpke_public_key_x25519: vec![8; 32],
        kernel_instance_id: String::new(),
        kernel_authorization_signature_raw: Vec::new(),
        storage_role_epoch: 0,
        storage_credential_lease_revision: 0,
        storage_runtime_principal: String::new(),
        storage_owner_id: String::new(),
    }
}

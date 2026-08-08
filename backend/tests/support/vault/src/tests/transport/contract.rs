use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultCiphertextFrameV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportError,
    VaultTransportPublicKey, VaultTransportSessionV1, seal,
};

use crate::transport::keys::VaultTransportKeyPair;
use crate::transport::response::encrypt_result;
use crate::transport::session::VaultTransportReplayGuard;

#[test]
fn public_hpke_sender_binds_ciphertext_to_generation_audience_epoch_and_request() {
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let binding = VaultTransportBindingV1::new(
        3,
        audience.clone(),
        [1; 16],
        [2; 32],
        VaultTransportDirectionV1::ToVault,
        [6; 32],
    )
    .expect("transport binding");
    let keys = VaultTransportKeyPair::generate();
    assert_eq!(
        VaultTransportPublicKey::from_bytes(*keys.public_key().as_bytes())
            .expect("deserialize generated public key"),
        keys.public_key().clone()
    );
    let frame = seal(keys.public_key(), &binding, b"credential-transport-marker")
        .expect("seal credential material");
    assert_eq!(frame.ciphertext_len(), "credential-transport-marker".len());
    assert_eq!(
        keys.open(&binding, &frame)
            .expect("open matching frame")
            .as_slice(),
        b"credential-transport-marker"
    );
    assert_rejects_context_substitution(&keys, frame, audience);
}

#[test]
fn store_command_round_trip_rejects_an_empty_credential_payload() {
    let command = VaultTransportCommandV1::StoreLease {
        lease_id: makosh_vault_protocol::LeaseIdV1::new("b".repeat(32))
            .expect("typed lease identifier"),
        secret_class: SecretClassV1::ProviderCredential,
        payload: b"credential-command-payload".to_vec(),
    };
    assert_eq!(
        makosh_vault_protocol::VaultTransportCommandV1::decode(&command.encode()),
        Ok(command)
    );
    assert_eq!(
        makosh_vault_protocol::VaultTransportCommandV1::decode(
            &VaultTransportCommandV1::RevokeAudience.encode(),
        ),
        Ok(VaultTransportCommandV1::RevokeAudience)
    );
    let mut empty = resolve_command().encode();
    empty[1] = 2;
    assert!(makosh_vault_protocol::VaultTransportCommandV1::decode(&empty).is_err());
}

#[test]
fn generated_token_command_carries_no_secret_payload() {
    let command = VaultTransportCommandV1::GenerateOpaqueToken {
        lease_id: makosh_vault_protocol::LeaseIdV1::new("c".repeat(32))
            .expect("typed lease identifier"),
        secret_class: SecretClassV1::PlatformCredential,
    };
    assert_eq!(
        VaultTransportCommandV1::decode(&command.encode()),
        Ok(command)
    );
    assert!(VaultTransportCommandV1::decode(&[1, 6, 4]).is_err());
}

#[test]
fn issue_lease_command_round_trip_preserves_all_fences() {
    let audience = LeaseAudienceV1::new(
        "registration-storage".to_owned(),
        "runtime-storage-1".to_owned(),
        4,
        9,
    )
    .expect("typed audience");
    let request = makosh_vault_protocol::VaultLeaseIssueRequestV1::new(
        "vault-instance".to_owned(),
        3,
        2,
        "storage".to_owned(),
        makosh_vault_protocol::VaultPurposeRequestV1::new(
            "storage.runtime.credential".to_owned(),
            "storage-notes".to_owned(),
            vec![SecretClassV1::PlatformCredential],
            vec![makosh_vault_protocol::VaultActionV1::Resolve],
            60,
        )
        .expect("typed purpose"),
        audience,
    )
    .expect("typed lease request");
    let command = VaultTransportCommandV1::IssueLease { request };

    assert_eq!(
        VaultTransportCommandV1::decode(&command.encode()),
        Ok(command)
    );
    assert!(VaultTransportCommandV1::decode(&[1, 5]).is_err());
}

fn assert_rejects_context_substitution(
    keys: &VaultTransportKeyPair,
    frame: makosh_vault_protocol::VaultCiphertextFrameV1,
    audience: LeaseAudienceV1,
) {
    let stale_epoch = VaultTransportBindingV1::new(
        3,
        LeaseAudienceV1::new(
            "registration-mail".to_owned(),
            "runtime-mail-1".to_owned(),
            1,
            8,
        )
        .expect("stale epoch audience"),
        [1; 16],
        [2; 32],
        VaultTransportDirectionV1::ToVault,
        [6; 32],
    )
    .expect("stale transport binding");
    assert_eq!(
        keys.open(&stale_epoch, &frame),
        Err(VaultTransportError::AuthenticationFailed)
    );
    let stale_runtime = VaultTransportBindingV1::new(
        3,
        LeaseAudienceV1::new(
            "registration-mail".to_owned(),
            "runtime-mail-1".to_owned(),
            2,
            7,
        )
        .expect("stale runtime audience"),
        [1; 16],
        [2; 32],
        VaultTransportDirectionV1::ToVault,
        [6; 32],
    )
    .expect("stale runtime binding");
    assert_eq!(
        keys.open(&stale_runtime, &frame),
        Err(VaultTransportError::AuthenticationFailed)
    );
    let reverse_direction = VaultTransportBindingV1::new(
        3,
        audience,
        [1; 16],
        [2; 32],
        VaultTransportDirectionV1::FromVault,
        [6; 32],
    )
    .expect("reverse direction binding");
    assert_eq!(
        keys.open(&reverse_direction, &frame),
        Err(VaultTransportError::AuthenticationFailed)
    );
}

#[test]
fn vault_private_transport_session_rejects_replays_wrong_direction_and_generation() {
    let keys = VaultTransportKeyPair::generate();
    let command = resolve_command();
    let session = sealed_session(
        &keys,
        3,
        [3; 16],
        VaultTransportDirectionV1::ToVault,
        &command,
    );
    let mut guard = VaultTransportReplayGuard::new(3);
    assert_eq!(
        guard.open_command(&keys, &session).expect("first delivery"),
        command
    );
    assert_eq!(
        guard.open_command(&keys, &session),
        Err(VaultTransportError::ReplayDetected)
    );

    let reverse = sealed_session(
        &keys,
        3,
        [4; 16],
        VaultTransportDirectionV1::FromVault,
        &resolve_command(),
    );
    assert_eq!(
        guard.open_command(&keys, &reverse),
        Err(VaultTransportError::WrongDirection)
    );
    let stale = sealed_session(
        &keys,
        2,
        [5; 16],
        VaultTransportDirectionV1::ToVault,
        &resolve_command(),
    );
    assert_eq!(
        guard.open_command(&keys, &stale),
        Err(VaultTransportError::InvalidBinding)
    );
}

#[test]
fn vault_private_transport_session_requires_the_decrypted_command_digest() {
    let keys = VaultTransportKeyPair::generate();
    let command = resolve_command();
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let binding = VaultTransportBindingV1::new(
        3,
        audience,
        [9; 16],
        [0; 32],
        VaultTransportDirectionV1::ToVault,
        [6; 32],
    )
    .expect("mismatched binding");
    let frame = seal(keys.public_key(), &binding, &command.encode()).expect("seal command");
    let session = VaultTransportSessionV1::new(binding, frame);

    assert_eq!(
        VaultTransportReplayGuard::new(3).open_command(&keys, &session),
        Err(VaultTransportError::InvalidBinding)
    );
}

#[test]
fn vault_result_is_reencrypted_for_the_aad_bound_response_recipient() {
    let vault_keys = VaultTransportKeyPair::generate();
    let response_recipient = VaultTransportKeyPair::generate();
    let command = resolve_command();
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let request = VaultTransportBindingV1::new(
        3,
        audience.clone(),
        [8; 16],
        command.operation_digest(),
        VaultTransportDirectionV1::ToVault,
        *response_recipient.public_key().as_bytes(),
    )
    .expect("request binding");
    let response = encrypt_result(&request, b"credential-result").expect("encrypt response");
    let binding = VaultTransportBindingV1::new(
        3,
        audience,
        [8; 16],
        command.operation_digest(),
        VaultTransportDirectionV1::FromVault,
        *response_recipient.public_key().as_bytes(),
    )
    .expect("response binding");
    let frame = VaultCiphertextFrameV1::from_parts(
        response.hpke_encapped_key,
        response.ciphertext,
        response.hpke_authentication_tag,
    )
    .expect("response frame");
    assert_eq!(
        response_recipient
            .open(&binding, &frame)
            .expect("open response")
            .as_slice(),
        b"credential-result"
    );
    assert!(vault_keys.public_key().as_bytes() != response_recipient.public_key().as_bytes());
}

fn sealed_session(
    keys: &VaultTransportKeyPair,
    generation: u64,
    request_id: [u8; 16],
    direction: VaultTransportDirectionV1,
    command: &VaultTransportCommandV1,
) -> VaultTransportSessionV1 {
    let audience = LeaseAudienceV1::new(
        "registration-mail".to_owned(),
        "runtime-mail-1".to_owned(),
        1,
        7,
    )
    .expect("typed audience");
    let binding = VaultTransportBindingV1::new(
        generation,
        audience,
        request_id,
        command.operation_digest(),
        direction,
        [6; 32],
    )
    .expect("transport binding");
    let frame =
        seal(keys.public_key(), &binding, &command.encode()).expect("seal credential material");
    VaultTransportSessionV1::new(binding, frame)
}

fn resolve_command() -> VaultTransportCommandV1 {
    VaultTransportCommandV1::ResolveLease {
        lease_id: makosh_vault_protocol::LeaseIdV1::new("a".repeat(32))
            .expect("typed lease identifier"),
        secret_class: makosh_vault_protocol::SecretClassV1::ProviderCredential,
    }
}

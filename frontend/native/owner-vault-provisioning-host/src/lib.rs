//! Provider-neutral native cryptography for the first-party Vault ceremony.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use makosh_vault_protocol::{
    LeaseAudienceV1, LeaseIdV1, SecretClassV1, VaultActionV1, VaultCiphertextFrameV1,
    VaultProvisioningReceiptV1, VaultResponseRecipientV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportPublicKey, seal,
};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

const MAX_PENDING_SESSIONS: usize = 16;
const HOST_SESSION_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartedProvisioningHostSessionV1 {
    pub host_session_id: String,
    pub response_recipient_hpke_public_key_x25519: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedProvisioningV1 {
    pub vault_runtime_generation: u64,
    pub vault_hpke_public_key_x25519: [u8; 32],
    pub audience_registration_id: String,
    pub audience_runtime_instance_id: String,
    pub audience_runtime_generation: u64,
    pub audience_grant_epoch: u64,
    pub lease_request_id: [u8; 16],
    pub lease_operation_digest_sha256: [u8; 32],
    pub command_request_id: [u8; 16],
    pub lease_response_hpke_encapped_key: Vec<u8>,
    pub lease_response_ciphertext: Vec<u8>,
    pub lease_response_hpke_authentication_tag: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealedProvisioningCommandV1 {
    pub operation_digest_sha256: [u8; 32],
    pub hpke_encapped_key: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub hpke_authentication_tag: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedProvisioningReceiptV1 {
    pub vault_runtime_generation: u64,
    pub command_request_id: [u8; 16],
    pub operation_digest_sha256: [u8; 32],
    pub receipt_hpke_encapped_key: Vec<u8>,
    pub receipt_ciphertext: Vec<u8>,
    pub receipt_hpke_authentication_tag: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanitizedProvisioningReceiptV1 {
    pub operation_id: [u8; 16],
    pub action: i32,
    pub secret_revision: u64,
    pub state: u8,
}

#[derive(Default)]
pub struct OwnerVaultProvisioningHostV1 {
    state: Mutex<HostStateV1>,
}

#[derive(Default)]
struct HostStateV1 {
    sessions: HashMap<String, HostSessionV1>,
}

struct HostSessionV1 {
    created_at: Instant,
    recipient: VaultResponseRecipientV1,
    sealed: Option<SealedContextV1>,
}

struct SealedContextV1 {
    vault_runtime_generation: u64,
    audience: LeaseAudienceV1,
    command_request_id: [u8; 16],
    operation_digest: [u8; 32],
    operation_id: [u8; 16],
    action: VaultActionV1,
    response_public_key: [u8; 32],
}

impl OwnerVaultProvisioningHostV1 {
    pub fn start(
        &self,
    ) -> Result<StartedProvisioningHostSessionV1, OwnerVaultProvisioningHostErrorV1> {
        let mut state = self.lock_state()?;
        state
            .sessions
            .retain(|_, session| session.created_at.elapsed() <= HOST_SESSION_TTL);
        if state.sessions.len() >= MAX_PENDING_SESSIONS {
            return Err(OwnerVaultProvisioningHostErrorV1::CapacityExceeded);
        }
        let recipient = VaultResponseRecipientV1::generate();
        let response_public_key = *recipient.public_key().as_bytes();
        let host_session_id = hex_sha256(&response_public_key);
        if state.sessions.contains_key(&host_session_id) {
            return Err(OwnerVaultProvisioningHostErrorV1::CapacityExceeded);
        }
        state.sessions.insert(
            host_session_id.clone(),
            HostSessionV1 {
                created_at: Instant::now(),
                recipient,
                sealed: None,
            },
        );
        Ok(StartedProvisioningHostSessionV1 {
            host_session_id,
            response_recipient_hpke_public_key_x25519: response_public_key,
        })
    }

    pub fn seal(
        &self,
        host_session_id: &str,
        authorized: AuthorizedProvisioningV1,
        operation_id: [u8; 16],
        action: i32,
        secret_class: i32,
        secret_payload: Vec<u8>,
    ) -> Result<SealedProvisioningCommandV1, OwnerVaultProvisioningHostErrorV1> {
        self.seal_custodied(
            host_session_id,
            authorized,
            operation_id,
            action,
            secret_class,
            Zeroizing::new(secret_payload),
        )
    }

    pub fn seal_custodied(
        &self,
        host_session_id: &str,
        authorized: AuthorizedProvisioningV1,
        operation_id: [u8; 16],
        action: i32,
        secret_class: i32,
        mut secret_payload: Zeroizing<Vec<u8>>,
    ) -> Result<SealedProvisioningCommandV1, OwnerVaultProvisioningHostErrorV1> {
        if operation_id == [0; 16] || secret_payload.is_empty() {
            return Err(OwnerVaultProvisioningHostErrorV1::InvalidInput);
        }
        let action = owner_vault_action_from_wire_code_v1(action)?;
        let secret_class = owner_vault_secret_class_from_wire_code_v1(secret_class)?;
        let mut state = self.lock_state()?;
        let session = current_session(&mut state, host_session_id)?;
        if session.sealed.is_some() {
            return Err(OwnerVaultProvisioningHostErrorV1::InvalidState);
        }
        let response_public_key = *session.recipient.public_key().as_bytes();
        let audience = LeaseAudienceV1::new(
            authorized.audience_registration_id,
            authorized.audience_runtime_instance_id,
            authorized.audience_runtime_generation,
            authorized.audience_grant_epoch,
        )
        .map_err(|_| OwnerVaultProvisioningHostErrorV1::InvalidInput)?;
        let lease_binding = VaultTransportBindingV1::new(
            authorized.vault_runtime_generation,
            audience.clone(),
            authorized.lease_request_id,
            authorized.lease_operation_digest_sha256,
            VaultTransportDirectionV1::FromVault,
            response_public_key,
        )
        .map_err(|_| OwnerVaultProvisioningHostErrorV1::InvalidInput)?;
        let lease_frame = VaultCiphertextFrameV1::from_parts(
            authorized.lease_response_hpke_encapped_key,
            authorized.lease_response_ciphertext,
            authorized.lease_response_hpke_authentication_tag,
        )
        .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        let lease_id = session
            .recipient
            .open(&lease_binding, &lease_frame)
            .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        let lease_id = LeaseIdV1::new(
            String::from_utf8(lease_id.to_vec())
                .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?,
        )
        .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        let mut command = VaultTransportCommandV1::ProvisionLease {
            lease_id,
            operation_id,
            action,
            secret_class,
            payload: secret_payload.to_vec(),
        };
        secret_payload.zeroize();
        let operation_digest = command.operation_digest();
        let command_binding = VaultTransportBindingV1::new(
            authorized.vault_runtime_generation,
            audience.clone(),
            authorized.command_request_id,
            operation_digest,
            VaultTransportDirectionV1::ToVault,
            response_public_key,
        )
        .map_err(|_| OwnerVaultProvisioningHostErrorV1::InvalidInput)?;
        let vault_key =
            VaultTransportPublicKey::from_bytes(authorized.vault_hpke_public_key_x25519)
                .map_err(|_| OwnerVaultProvisioningHostErrorV1::InvalidInput)?;
        let encoded_command = Zeroizing::new(command.encode());
        if let VaultTransportCommandV1::ProvisionLease { payload, .. } = &mut command {
            payload.zeroize();
        }
        let frame = seal(&vault_key, &command_binding, &encoded_command)
            .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        session.sealed = Some(SealedContextV1 {
            vault_runtime_generation: authorized.vault_runtime_generation,
            audience,
            command_request_id: authorized.command_request_id,
            operation_digest,
            operation_id,
            action,
            response_public_key,
        });
        Ok(SealedProvisioningCommandV1 {
            operation_digest_sha256: operation_digest,
            hpke_encapped_key: frame.encapped_key().to_vec(),
            ciphertext: frame.ciphertext().to_vec(),
            hpke_authentication_tag: frame.tag().to_vec(),
        })
    }

    pub fn open_receipt(
        &self,
        host_session_id: &str,
        committed: CommittedProvisioningReceiptV1,
    ) -> Result<SanitizedProvisioningReceiptV1, OwnerVaultProvisioningHostErrorV1> {
        let session = self
            .lock_state()?
            .sessions
            .remove(host_session_id)
            .ok_or(OwnerVaultProvisioningHostErrorV1::SessionUnavailable)?;
        if session.created_at.elapsed() > HOST_SESSION_TTL {
            return Err(OwnerVaultProvisioningHostErrorV1::SessionUnavailable);
        }
        let sealed = session
            .sealed
            .ok_or(OwnerVaultProvisioningHostErrorV1::InvalidState)?;
        if committed.vault_runtime_generation != sealed.vault_runtime_generation
            || committed.command_request_id != sealed.command_request_id
            || committed.operation_digest_sha256 != sealed.operation_digest
        {
            return Err(OwnerVaultProvisioningHostErrorV1::Rejected);
        }
        let binding = VaultTransportBindingV1::new(
            sealed.vault_runtime_generation,
            sealed.audience,
            sealed.command_request_id,
            sealed.operation_digest,
            VaultTransportDirectionV1::FromVault,
            sealed.response_public_key,
        )
        .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        let frame = VaultCiphertextFrameV1::from_parts(
            committed.receipt_hpke_encapped_key,
            committed.receipt_ciphertext,
            committed.receipt_hpke_authentication_tag,
        )
        .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        let receipt = session
            .recipient
            .open(&binding, &frame)
            .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        let receipt = VaultProvisioningReceiptV1::decode(&receipt)
            .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?;
        if receipt.operation_id() != &sealed.operation_id || receipt.action() != sealed.action {
            return Err(OwnerVaultProvisioningHostErrorV1::Rejected);
        }
        Ok(SanitizedProvisioningReceiptV1 {
            operation_id: *receipt.operation_id(),
            action: i32::try_from(receipt.action().code())
                .map_err(|_| OwnerVaultProvisioningHostErrorV1::Rejected)?,
            secret_revision: receipt.secret_revision(),
            state: receipt.state().code(),
        })
    }

    pub fn cancel(&self, host_session_id: &str) -> Result<(), OwnerVaultProvisioningHostErrorV1> {
        self.lock_state()?.sessions.remove(host_session_id);
        Ok(())
    }

    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HostStateV1>, OwnerVaultProvisioningHostErrorV1> {
        self.state
            .lock()
            .map_err(|_| OwnerVaultProvisioningHostErrorV1::Unavailable)
    }
}

fn current_session<'a>(
    state: &'a mut HostStateV1,
    host_session_id: &str,
) -> Result<&'a mut HostSessionV1, OwnerVaultProvisioningHostErrorV1> {
    let expired = state
        .sessions
        .get(host_session_id)
        .is_some_and(|session| session.created_at.elapsed() > HOST_SESSION_TTL);
    if expired {
        state.sessions.remove(host_session_id);
    }
    state
        .sessions
        .get_mut(host_session_id)
        .ok_or(OwnerVaultProvisioningHostErrorV1::SessionUnavailable)
}

pub fn owner_vault_action_from_wire_code_v1(
    value: i32,
) -> Result<VaultActionV1, OwnerVaultProvisioningHostErrorV1> {
    match value {
        1 => Ok(VaultActionV1::Create),
        2 => Ok(VaultActionV1::ReplaceCas),
        3 => Ok(VaultActionV1::Retire),
        4 => Ok(VaultActionV1::Delete),
        _ => Err(OwnerVaultProvisioningHostErrorV1::InvalidInput),
    }
}

pub fn owner_vault_secret_class_from_wire_code_v1(
    value: i32,
) -> Result<SecretClassV1, OwnerVaultProvisioningHostErrorV1> {
    match value {
        1 => Ok(SecretClassV1::ProviderCredential),
        2 => Ok(SecretClassV1::OAuthRefreshCredential),
        3 => Ok(SecretClassV1::SessionCredentialBlob),
        5 => Ok(SecretClassV1::SessionStoreKey),
        _ => Err(OwnerVaultProvisioningHostErrorV1::InvalidInput),
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerVaultProvisioningHostErrorV1 {
    InvalidInput,
    InvalidState,
    SessionUnavailable,
    CapacityExceeded,
    Rejected,
    Unavailable,
}

impl std::fmt::Display for OwnerVaultProvisioningHostErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "owner Vault provisioning input is invalid",
            Self::InvalidState => "owner Vault provisioning state is invalid",
            Self::SessionUnavailable => "owner Vault provisioning session is unavailable",
            Self::CapacityExceeded => "owner Vault provisioning capacity is unavailable",
            Self::Rejected => "owner Vault provisioning response was rejected",
            Self::Unavailable => "owner Vault provisioning host is unavailable",
        })
    }
}

impl std::error::Error for OwnerVaultProvisioningHostErrorV1 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_session_store_key_without_accepting_unknown_classes() {
        assert_eq!(
            owner_vault_secret_class_from_wire_code_v1(5).expect("session store key"),
            SecretClassV1::SessionStoreKey
        );
        assert_eq!(
            owner_vault_secret_class_from_wire_code_v1(4)
                .expect_err("unknown class must be rejected"),
            OwnerVaultProvisioningHostErrorV1::InvalidInput
        );
    }

    #[test]
    fn seals_secret_and_opens_only_the_sanitized_receipt() {
        let host = OwnerVaultProvisioningHostV1::default();
        let started = host.start().expect("start host session");
        let vault = VaultResponseRecipientV1::generate();
        let audience = LeaseAudienceV1::new(
            "mail-registration".into(),
            "owner-vault-runtime".into(),
            7,
            3,
        )
        .expect("audience");
        let lease_request_id = [4; 16];
        let lease_operation_digest = [5; 32];
        let command_request_id = [6; 16];
        let host_key =
            VaultTransportPublicKey::from_bytes(started.response_recipient_hpke_public_key_x25519)
                .expect("host key");
        let lease_binding = VaultTransportBindingV1::new(
            9,
            audience.clone(),
            lease_request_id,
            lease_operation_digest,
            VaultTransportDirectionV1::FromVault,
            started.response_recipient_hpke_public_key_x25519,
        )
        .expect("lease binding");
        let lease_frame = seal(
            &host_key,
            &lease_binding,
            b"0123456789abcdef0123456789abcdef",
        )
        .expect("seal lease");
        let authorized = AuthorizedProvisioningV1 {
            vault_runtime_generation: 9,
            vault_hpke_public_key_x25519: *vault.public_key().as_bytes(),
            audience_registration_id: "mail-registration".into(),
            audience_runtime_instance_id: "owner-vault-runtime".into(),
            audience_runtime_generation: 7,
            audience_grant_epoch: 3,
            lease_request_id,
            lease_operation_digest_sha256: lease_operation_digest,
            command_request_id,
            lease_response_hpke_encapped_key: lease_frame.encapped_key().to_vec(),
            lease_response_ciphertext: lease_frame.ciphertext().to_vec(),
            lease_response_hpke_authentication_tag: lease_frame.tag().to_vec(),
        };
        let operation_id = [8; 16];
        let sealed = host
            .seal(
                &started.host_session_id,
                authorized,
                operation_id,
                1,
                1,
                b"private-password".to_vec(),
            )
            .expect("seal command");
        let command_binding = VaultTransportBindingV1::new(
            9,
            audience.clone(),
            command_request_id,
            sealed.operation_digest_sha256,
            VaultTransportDirectionV1::ToVault,
            started.response_recipient_hpke_public_key_x25519,
        )
        .expect("command binding");
        let command_frame = VaultCiphertextFrameV1::from_parts(
            sealed.hpke_encapped_key,
            sealed.ciphertext,
            sealed.hpke_authentication_tag,
        )
        .expect("command frame");
        let opened_command = vault
            .open(&command_binding, &command_frame)
            .expect("open command");
        assert!(opened_command.ends_with(b"private-password"));

        let receipt = VaultProvisioningReceiptV1::new(
            operation_id,
            VaultActionV1::Create,
            1,
            makosh_vault_protocol::VaultProvisioningStateV1::Active,
        )
        .expect("receipt");
        let receipt_binding = VaultTransportBindingV1::new(
            9,
            audience,
            command_request_id,
            sealed.operation_digest_sha256,
            VaultTransportDirectionV1::FromVault,
            started.response_recipient_hpke_public_key_x25519,
        )
        .expect("receipt binding");
        let receipt_frame =
            seal(&host_key, &receipt_binding, &receipt.encode()).expect("seal receipt");
        let sanitized = host
            .open_receipt(
                &started.host_session_id,
                CommittedProvisioningReceiptV1 {
                    vault_runtime_generation: 9,
                    command_request_id,
                    operation_digest_sha256: sealed.operation_digest_sha256,
                    receipt_hpke_encapped_key: receipt_frame.encapped_key().to_vec(),
                    receipt_ciphertext: receipt_frame.ciphertext().to_vec(),
                    receipt_hpke_authentication_tag: receipt_frame.tag().to_vec(),
                },
            )
            .expect("open receipt");
        assert_eq!(sanitized.operation_id, operation_id);
        assert_eq!(sanitized.secret_revision, 1);
        assert_eq!(sanitized.state, 1);
        assert_eq!(
            host.open_receipt(
                &started.host_session_id,
                CommittedProvisioningReceiptV1 {
                    vault_runtime_generation: 9,
                    command_request_id,
                    operation_digest_sha256: [1; 32],
                    receipt_hpke_encapped_key: vec![1],
                    receipt_ciphertext: vec![1],
                    receipt_hpke_authentication_tag: vec![1],
                },
            ),
            Err(OwnerVaultProvisioningHostErrorV1::SessionUnavailable)
        );
    }
}

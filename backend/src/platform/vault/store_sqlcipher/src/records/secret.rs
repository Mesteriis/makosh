//! Authenticated per-record envelope for bounded Vault credential material.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use getrandom::fill;
use makosh_vault_protocol::{
    MAX_CREDENTIAL_BYTES, MAX_SESSION_CREDENTIAL_BYTES, SecretClassV1, VaultProtocolError,
    VaultPurposeRequestV1, validate_logical_owner_id,
};
use zeroize::Zeroizing;

const RECORD_ID_BYTES: usize = 16;
const NONCE_BYTES: usize = 24;
pub const CURRENT_KEY_EPOCH: u32 = 1;

#[derive(Clone, Eq, PartialEq)]
pub struct SecretRecordId([u8; RECORD_ID_BYTES]);

impl SecretRecordId {
    pub(crate) fn random() -> Result<Self, SecretRecordError> {
        let mut bytes = [0; RECORD_ID_BYTES];
        fill(&mut bytes).map_err(|_| SecretRecordError::Randomness)?;
        Ok(Self(bytes))
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8; RECORD_ID_BYTES] {
        &self.0
    }

    #[must_use]
    pub fn from_bytes(bytes: [u8; RECORD_ID_BYTES]) -> Self {
        Self(bytes)
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self, SecretRecordError> {
        let bytes = bytes
            .try_into()
            .map_err(|_| SecretRecordError::MalformedRecord)?;
        Ok(Self(bytes))
    }
}

#[derive(Clone)]
pub struct SecretRecordScope {
    logical_owner_id: String,
    configuration_instance_id: String,
    purpose_id: String,
    secret_class: SecretClassV1,
    secret_revision: u64,
}

impl SecretRecordScope {
    pub fn new(
        logical_owner_id: String,
        request: &VaultPurposeRequestV1,
        secret_class: SecretClassV1,
        secret_revision: u64,
    ) -> Result<Self, SecretRecordError> {
        validate_logical_owner_id(&logical_owner_id).map_err(SecretRecordError::Protocol)?;
        if !request.allowed_secret_classes().contains(&secret_class) {
            return Err(SecretRecordError::SecretClassNotAllowed);
        }
        if secret_revision == 0 {
            return Err(SecretRecordError::InvalidRevision);
        }
        Ok(Self {
            logical_owner_id,
            configuration_instance_id: request.configuration_instance_id().to_owned(),
            purpose_id: request.purpose_id().to_owned(),
            secret_class,
            secret_revision,
        })
    }

    pub(crate) fn matches_metadata(
        &self,
        logical_owner_id: &str,
        configuration_instance_id: &str,
        purpose_id: &str,
        secret_class: i64,
        secret_revision: i64,
        key_epoch: i64,
    ) -> bool {
        self.logical_owner_id == logical_owner_id
            && self.configuration_instance_id == configuration_instance_id
            && self.purpose_id == purpose_id
            && self.secret_class.code() == secret_class
            && i64::try_from(self.secret_revision) == Ok(secret_revision)
            && key_epoch == i64::from(CURRENT_KEY_EPOCH)
    }

    fn from_stored_metadata(
        logical_owner_id: &str,
        configuration_instance_id: &str,
        purpose_id: &str,
        secret_class: i64,
        secret_revision: i64,
        key_epoch: i64,
    ) -> Result<Self, SecretRecordError> {
        if key_epoch != i64::from(CURRENT_KEY_EPOCH) {
            return Err(SecretRecordError::MalformedRecord);
        }
        validate_logical_owner_id(logical_owner_id).map_err(SecretRecordError::Protocol)?;
        if !stored_identifier(configuration_instance_id) || !stored_identifier(purpose_id) {
            return Err(SecretRecordError::MalformedRecord);
        }
        let secret_class =
            SecretClassV1::from_code(secret_class).ok_or(SecretRecordError::MalformedRecord)?;
        let secret_revision = u64::try_from(secret_revision)
            .ok()
            .filter(|revision| *revision > 0)
            .ok_or(SecretRecordError::MalformedRecord)?;
        Ok(Self {
            logical_owner_id: logical_owner_id.to_owned(),
            configuration_instance_id: configuration_instance_id.to_owned(),
            purpose_id: purpose_id.to_owned(),
            secret_class,
            secret_revision,
        })
    }

    pub(crate) fn metadata(&self) -> (&str, &str, &str, i64, i64) {
        (
            &self.logical_owner_id,
            &self.configuration_instance_id,
            &self.purpose_id,
            self.secret_class.code(),
            i64::try_from(self.secret_revision).expect("validated Vault revision fits i64"),
        )
    }

    pub(crate) fn with_revision(&self, secret_revision: u64) -> Result<Self, SecretRecordError> {
        if secret_revision == 0 {
            return Err(SecretRecordError::InvalidRevision);
        }
        Ok(Self {
            logical_owner_id: self.logical_owner_id.clone(),
            configuration_instance_id: self.configuration_instance_id.clone(),
            purpose_id: self.purpose_id.clone(),
            secret_class: self.secret_class,
            secret_revision,
        })
    }

    pub fn matches_lease_request(
        &self,
        request: &makosh_vault_protocol::VaultLeaseIssueRequestV1,
    ) -> bool {
        self.logical_owner_id == request.logical_owner_id()
            && self.configuration_instance_id == request.purpose().configuration_instance_id()
            && self.purpose_id == request.purpose().purpose_id()
            && request
                .purpose()
                .allowed_secret_classes()
                .contains(&self.secret_class)
            && self.secret_revision == request.secret_revision()
    }

    pub(crate) fn replaces(&self, prior: &Self) -> bool {
        self.logical_owner_id == prior.logical_owner_id
            && self.configuration_instance_id == prior.configuration_instance_id
            && self.purpose_id == prior.purpose_id
            && self.secret_class == prior.secret_class
            && prior.secret_revision.checked_add(1) == Some(self.secret_revision)
    }

    fn aad(&self, record_id: &SecretRecordId) -> Vec<u8> {
        let mut aad = Vec::with_capacity(256);
        aad.extend_from_slice(b"HVREC1");
        aad.extend_from_slice(record_id.as_bytes());
        append_field(&mut aad, self.logical_owner_id.as_bytes());
        append_field(&mut aad, self.configuration_instance_id.as_bytes());
        append_field(&mut aad, self.purpose_id.as_bytes());
        aad.extend_from_slice(&self.secret_class.code().to_be_bytes());
        aad.extend_from_slice(&self.secret_revision.to_be_bytes());
        aad.extend_from_slice(&CURRENT_KEY_EPOCH.to_be_bytes());
        aad
    }
}

pub(crate) struct EncryptedSecretRecord {
    pub(crate) record_id: SecretRecordId,
    pub(crate) nonce: [u8; NONCE_BYTES],
    pub(crate) ciphertext: Vec<u8>,
}

pub(crate) struct StoredRecordReencryptionInput<'a> {
    pub(crate) record_id: &'a [u8],
    pub(crate) logical_owner_id: &'a str,
    pub(crate) configuration_instance_id: &'a str,
    pub(crate) purpose_id: &'a str,
    pub(crate) secret_class: i64,
    pub(crate) secret_revision: i64,
    pub(crate) key_epoch: i64,
    pub(crate) nonce: &'a [u8],
    pub(crate) ciphertext: &'a [u8],
    pub(crate) current_record_key: &'a [u8; 32],
    pub(crate) next_record_key: &'a [u8; 32],
}

pub(crate) fn encrypt(
    scope: &SecretRecordScope,
    payload: &[u8],
    record_key: &[u8; 32],
) -> Result<EncryptedSecretRecord, SecretRecordError> {
    let record_id = SecretRecordId::random()?;
    encrypt_with_record_id(scope, record_id, payload, record_key)
}

pub(crate) fn reencrypt_stored_record(
    input: StoredRecordReencryptionInput<'_>,
) -> Result<EncryptedSecretRecord, SecretRecordError> {
    let scope = SecretRecordScope::from_stored_metadata(
        input.logical_owner_id,
        input.configuration_instance_id,
        input.purpose_id,
        input.secret_class,
        input.secret_revision,
        input.key_epoch,
    )?;
    let record_id = SecretRecordId::from_slice(input.record_id)?;
    let plaintext = decrypt(
        &scope,
        record_id.clone(),
        input.nonce,
        input.ciphertext,
        input.current_record_key,
    )?;
    encrypt_with_record_id(
        &scope,
        record_id,
        plaintext.as_slice(),
        input.next_record_key,
    )
}

fn encrypt_with_record_id(
    scope: &SecretRecordScope,
    record_id: SecretRecordId,
    payload: &[u8],
    record_key: &[u8; 32],
) -> Result<EncryptedSecretRecord, SecretRecordError> {
    validate_payload(scope.secret_class, payload)?;
    let mut nonce_bytes = [0; NONCE_BYTES];
    fill(&mut nonce_bytes).map_err(|_| SecretRecordError::Randomness)?;
    let cipher =
        XChaCha20Poly1305::new_from_slice(record_key).map_err(|_| SecretRecordError::Cipher)?;
    let nonce = XNonce::try_from(nonce_bytes.as_slice()).map_err(|_| SecretRecordError::Cipher)?;
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: payload,
                aad: &scope.aad(&record_id),
            },
        )
        .map_err(|_| SecretRecordError::Cipher)?;
    Ok(EncryptedSecretRecord {
        record_id,
        nonce: nonce_bytes,
        ciphertext,
    })
}

fn stored_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

pub(crate) fn decrypt(
    scope: &SecretRecordScope,
    record_id: SecretRecordId,
    nonce: &[u8],
    ciphertext: &[u8],
    record_key: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, SecretRecordError> {
    let nonce: [u8; NONCE_BYTES] = nonce
        .try_into()
        .map_err(|_| SecretRecordError::MalformedRecord)?;
    let cipher =
        XChaCha20Poly1305::new_from_slice(record_key).map_err(|_| SecretRecordError::Cipher)?;
    let nonce =
        XNonce::try_from(nonce.as_slice()).map_err(|_| SecretRecordError::MalformedRecord)?;
    let plaintext = cipher
        .decrypt(
            &nonce,
            Payload {
                msg: ciphertext,
                aad: &scope.aad(&record_id),
            },
        )
        .map_err(|_| SecretRecordError::Cipher)?;
    validate_payload(scope.secret_class, &plaintext)?;
    Ok(Zeroizing::new(plaintext))
}

fn append_field(target: &mut Vec<u8>, field: &[u8]) {
    let length = u16::try_from(field.len()).expect("bounded Vault identifiers fit u16");
    target.extend_from_slice(&length.to_be_bytes());
    target.extend_from_slice(field);
}

fn validate_payload(secret_class: SecretClassV1, payload: &[u8]) -> Result<(), SecretRecordError> {
    let maximum = match secret_class {
        SecretClassV1::SessionCredentialBlob => MAX_SESSION_CREDENTIAL_BYTES,
        _ => MAX_CREDENTIAL_BYTES,
    };
    if payload.is_empty() || payload.len() > maximum {
        return Err(SecretRecordError::InvalidPayload);
    }
    Ok(())
}

#[derive(Debug)]
pub enum SecretRecordError {
    Protocol(VaultProtocolError),
    SecretClassNotAllowed,
    InvalidRevision,
    InvalidReplacement,
    InvalidPayload,
    Randomness,
    Cipher,
    MalformedRecord,
}

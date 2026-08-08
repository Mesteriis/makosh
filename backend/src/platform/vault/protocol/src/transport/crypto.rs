//! Public HPKE sender contract for credential material routed through Kernel.

use hpke::{
    Deserializable, Kem as KemTrait, OpModeR, OpModeS, Serializable,
    aead::{AeadTag, ChaCha20Poly1305},
    inout::InOutBuf,
    kdf::HkdfSha256,
    kem::X25519HkdfSha256,
};
use zeroize::Zeroizing;

use crate::{LeaseAudienceV1, MAX_SESSION_CREDENTIAL_BYTES};

const TRANSPORT_INFO: &[u8] = b"makosh-vault/hpke/v1";
const REQUEST_ID_BYTES: usize = 16;
const OPERATION_DIGEST_BYTES: usize = 32;
const X25519_BYTES: usize = 32;
const HPKE_TAG_BYTES: usize = 16;

type Kem = X25519HkdfSha256;
type Aead = ChaCha20Poly1305;
type Kdf = HkdfSha256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultTransportPublicKey([u8; X25519_BYTES]);

impl VaultTransportPublicKey {
    pub fn from_bytes(bytes: [u8; X25519_BYTES]) -> Result<Self, VaultTransportError> {
        <Kem as KemTrait>::PublicKey::from_bytes(&bytes)
            .map_err(|_| VaultTransportError::MalformedPublicKey)?;
        Ok(Self(bytes))
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8; X25519_BYTES] {
        &self.0
    }
}

/// Ephemeral response key owned by the runtime that initiated a Vault route.
///
/// This is deliberately distinct from the Vault runtime's long-lived transport
/// receiver: callers create one per request, disclose only its public key to
/// Vault, and retain the private key solely to open the matching response.
pub struct VaultResponseRecipientV1 {
    private_key: <Kem as KemTrait>::PrivateKey,
    public_key: VaultTransportPublicKey,
}

impl VaultResponseRecipientV1 {
    #[must_use]
    pub fn generate() -> Self {
        let (private_key, public_key) = Kem::gen_keypair();
        let bytes = public_key.to_bytes();
        let bytes = bytes
            .as_slice()
            .try_into()
            .expect("X25519 public key has a fixed size");
        Self {
            private_key,
            public_key: VaultTransportPublicKey::from_bytes(bytes)
                .expect("generated X25519 key is valid"),
        }
    }

    #[must_use]
    pub fn public_key(&self) -> &VaultTransportPublicKey {
        &self.public_key
    }

    pub fn open(
        &self,
        binding: &VaultTransportBindingV1,
        frame: &VaultCiphertextFrameV1,
    ) -> Result<Zeroizing<Vec<u8>>, VaultTransportError> {
        let encapped_key = <Kem as KemTrait>::EncappedKey::from_bytes(frame.encapped_key())
            .map_err(|_| VaultTransportError::MalformedFrame)?;
        let tag = AeadTag::<Aead>::from_bytes(frame.tag())
            .map_err(|_| VaultTransportError::MalformedFrame)?;
        let mut receiver = hpke::setup_receiver::<Aead, Kdf, Kem>(
            &OpModeR::Base,
            &self.private_key,
            &encapped_key,
            TRANSPORT_INFO,
        )
        .map_err(|_| VaultTransportError::MalformedFrame)?;
        let mut plaintext = frame.ciphertext().to_vec();
        receiver
            .open_inout_detached(
                InOutBuf::from(plaintext.as_mut_slice()),
                &binding.associated_data(),
                &tag,
            )
            .map_err(|_| VaultTransportError::AuthenticationFailed)?;
        Ok(Zeroizing::new(plaintext))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VaultTransportDirectionV1 {
    ToVault,
    FromVault,
}

pub struct VaultTransportBindingV1 {
    vault_runtime_generation: u64,
    audience: LeaseAudienceV1,
    request_id: [u8; REQUEST_ID_BYTES],
    operation_digest: [u8; OPERATION_DIGEST_BYTES],
    direction: VaultTransportDirectionV1,
    response_recipient_public_key: [u8; X25519_BYTES],
}

impl VaultTransportBindingV1 {
    pub fn new(
        vault_runtime_generation: u64,
        audience: LeaseAudienceV1,
        request_id: [u8; REQUEST_ID_BYTES],
        operation_digest: [u8; OPERATION_DIGEST_BYTES],
        direction: VaultTransportDirectionV1,
        response_recipient_public_key: [u8; X25519_BYTES],
    ) -> Result<Self, VaultTransportError> {
        if vault_runtime_generation == 0 {
            return Err(VaultTransportError::InvalidBinding);
        }
        VaultTransportPublicKey::from_bytes(response_recipient_public_key)?;
        Ok(Self {
            vault_runtime_generation,
            audience,
            request_id,
            operation_digest,
            direction,
            response_recipient_public_key,
        })
    }

    #[must_use]
    pub fn associated_data(&self) -> Vec<u8> {
        let mut aad = Vec::with_capacity(256);
        aad.extend_from_slice(b"HVHPKE1");
        aad.extend_from_slice(&self.vault_runtime_generation.to_be_bytes());
        append_field(&mut aad, self.audience.module_registration_id().as_bytes());
        append_field(&mut aad, self.audience.runtime_instance_id().as_bytes());
        aad.extend_from_slice(&self.audience.runtime_generation().to_be_bytes());
        aad.extend_from_slice(&self.audience.grant_epoch().to_be_bytes());
        aad.extend_from_slice(&self.request_id);
        aad.extend_from_slice(&self.operation_digest);
        aad.push(match self.direction {
            VaultTransportDirectionV1::ToVault => 1,
            VaultTransportDirectionV1::FromVault => 2,
        });
        aad.extend_from_slice(&self.response_recipient_public_key);
        aad
    }

    #[must_use]
    pub const fn vault_runtime_generation(&self) -> u64 {
        self.vault_runtime_generation
    }

    #[must_use]
    pub fn audience(&self) -> &LeaseAudienceV1 {
        &self.audience
    }

    #[must_use]
    pub const fn request_id(&self) -> &[u8; REQUEST_ID_BYTES] {
        &self.request_id
    }

    #[must_use]
    pub const fn operation_digest(&self) -> &[u8; OPERATION_DIGEST_BYTES] {
        &self.operation_digest
    }

    #[must_use]
    pub const fn direction(&self) -> VaultTransportDirectionV1 {
        self.direction
    }

    #[must_use]
    pub const fn response_recipient_public_key(&self) -> &[u8; X25519_BYTES] {
        &self.response_recipient_public_key
    }
}

pub struct VaultCiphertextFrameV1 {
    encapped_key: Vec<u8>,
    ciphertext: Vec<u8>,
    tag: Vec<u8>,
}

impl VaultCiphertextFrameV1 {
    pub fn from_parts(
        encapped_key: Vec<u8>,
        ciphertext: Vec<u8>,
        tag: Vec<u8>,
    ) -> Result<Self, VaultTransportError> {
        if ciphertext.is_empty() || ciphertext.len() > MAX_SESSION_CREDENTIAL_BYTES {
            return Err(VaultTransportError::OversizedFrame);
        }
        <Kem as KemTrait>::EncappedKey::from_bytes(&encapped_key)
            .map_err(|_| VaultTransportError::MalformedFrame)?;
        AeadTag::<Aead>::from_bytes(&tag).map_err(|_| VaultTransportError::MalformedFrame)?;
        if tag.len() != HPKE_TAG_BYTES {
            return Err(VaultTransportError::MalformedFrame);
        }
        Ok(Self {
            encapped_key,
            ciphertext,
            tag,
        })
    }

    #[must_use]
    pub fn encapped_key(&self) -> &[u8] {
        &self.encapped_key
    }

    #[must_use]
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    #[must_use]
    pub fn ciphertext_len(&self) -> usize {
        self.ciphertext.len()
    }

    #[must_use]
    pub fn tag(&self) -> &[u8] {
        &self.tag
    }
}

pub fn seal(
    recipient: &VaultTransportPublicKey,
    binding: &VaultTransportBindingV1,
    plaintext: &[u8],
) -> Result<VaultCiphertextFrameV1, VaultTransportError> {
    if plaintext.is_empty() || plaintext.len() > MAX_SESSION_CREDENTIAL_BYTES {
        return Err(VaultTransportError::OversizedFrame);
    }
    let recipient = <Kem as KemTrait>::PublicKey::from_bytes(recipient.as_bytes())
        .map_err(|_| VaultTransportError::MalformedPublicKey)?;
    let (encapped_key, mut sender) =
        hpke::setup_sender::<Aead, Kdf, Kem>(&OpModeS::Base, &recipient, TRANSPORT_INFO)
            .map_err(|_| VaultTransportError::MalformedPublicKey)?;
    let mut ciphertext = plaintext.to_vec();
    let tag = sender
        .seal_inout_detached(
            InOutBuf::from(ciphertext.as_mut_slice()),
            &binding.associated_data(),
        )
        .map_err(|_| VaultTransportError::EncryptionFailed)?;
    VaultCiphertextFrameV1::from_parts(
        encapped_key.to_bytes().as_slice().to_vec(),
        ciphertext,
        tag.to_bytes().as_slice().to_vec(),
    )
}

fn append_field(target: &mut Vec<u8>, field: &[u8]) {
    let length = u16::try_from(field.len()).expect("bounded Vault identifiers fit u16");
    target.extend_from_slice(&length.to_be_bytes());
    target.extend_from_slice(field);
}

#[derive(Debug, Eq, PartialEq)]
pub enum VaultTransportError {
    InvalidBinding,
    MalformedPublicKey,
    MalformedFrame,
    OversizedFrame,
    EncryptionFailed,
    AuthenticationFailed,
    WrongDirection,
    ReplayDetected,
    SessionCapacityExceeded,
}

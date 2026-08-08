//! Public encrypted NATS credential hand-off for one fenced runtime.
//!
//! The Event Hub issues the broker JWT, but a runtime opens the delivery itself.
//! Keeping this codec in the Events contract prevents a runtime from depending
//! on Event Hub's transport implementation or duplicating its crypto format.

use hpke::{
    Deserializable, Kem as KemTrait, OpModeR, OpModeS, Serializable,
    aead::{AeadTag, ChaCha20Poly1305},
    inout::InOutBuf,
    kdf::HkdfSha256,
    kem::X25519HkdfSha256,
};
use zeroize::Zeroizing;

const DELIVERY_INFO: &[u8] = b"makosh-events/nats-runtime-credential/v1";
const X25519_BYTES: usize = 32;
const REQUEST_ID_BYTES: usize = 16;
const TAG_BYTES: usize = 16;
const MAX_CREDENTIAL_BYTES: usize = 16 * 1024;
const CREDENTIAL_MAGIC: &[u8] = b"HENATS1";

type Kem = X25519HkdfSha256;
type Aead = ChaCha20Poly1305;
type Kdf = HkdfSha256;

/// Public ephemeral key supplied by the exact runtime that consumes a credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsRuntimeCredentialRecipientPublicKeyV1([u8; X25519_BYTES]);

impl NatsRuntimeCredentialRecipientPublicKeyV1 {
    pub fn from_bytes(
        bytes: [u8; X25519_BYTES],
    ) -> Result<Self, NatsRuntimeCredentialDeliveryErrorV1> {
        <Kem as KemTrait>::PublicKey::from_bytes(&bytes)
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidRecipient)?;
        Ok(Self(bytes))
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; X25519_BYTES] {
        &self.0
    }
}

/// One-time private recipient retained solely by the requesting runtime.
pub struct NatsRuntimeCredentialRecipientV1 {
    private_key: <Kem as KemTrait>::PrivateKey,
    public_key: NatsRuntimeCredentialRecipientPublicKeyV1,
}

impl NatsRuntimeCredentialRecipientV1 {
    #[must_use]
    pub fn generate() -> Self {
        let (private_key, public_key) = Kem::gen_keypair();
        let bytes = public_key.to_bytes();
        let bytes = bytes
            .as_slice()
            .try_into()
            .expect("X25519 key has fixed length");
        Self {
            private_key,
            public_key: NatsRuntimeCredentialRecipientPublicKeyV1::from_bytes(bytes)
                .expect("generated X25519 key is valid"),
        }
    }

    #[must_use]
    pub fn public_key(&self) -> &NatsRuntimeCredentialRecipientPublicKeyV1 {
        &self.public_key
    }

    pub fn open(
        &self,
        binding: &NatsRuntimeCredentialDeliveryBindingV1,
        delivery: &NatsRuntimeCredentialDeliveryV1,
    ) -> Result<RuntimeNatsJwtCredentialV1, NatsRuntimeCredentialDeliveryErrorV1> {
        binding.matches_recipient(&self.public_key)?;
        let encapped_key = <Kem as KemTrait>::EncappedKey::from_bytes(&delivery.encapped_key)
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidDelivery)?;
        let tag = AeadTag::<Aead>::from_bytes(&delivery.tag)
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidDelivery)?;
        let mut receiver = hpke::setup_receiver::<Aead, Kdf, Kem>(
            &OpModeR::Base,
            &self.private_key,
            &encapped_key,
            DELIVERY_INFO,
        )
        .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidDelivery)?;
        let mut plaintext = delivery.ciphertext.to_vec();
        receiver
            .open_inout_detached(
                InOutBuf::from(plaintext.as_mut_slice()),
                &binding.aad(),
                &tag,
            )
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::AuthenticationFailed)?;
        RuntimeNatsJwtCredentialV1::decode(&plaintext)
    }
}

/// Binds a credential delivery to exactly one runtime generation and request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsRuntimeCredentialDeliveryBindingV1 {
    logical_owner_id: String,
    registration_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    credential_revision: u64,
    request_id: [u8; REQUEST_ID_BYTES],
    recipient_public_key: NatsRuntimeCredentialRecipientPublicKeyV1,
}

pub struct NatsRuntimeCredentialDeliveryBindingInputV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub credential_revision: u64,
    pub request_id: [u8; REQUEST_ID_BYTES],
    pub recipient_public_key: NatsRuntimeCredentialRecipientPublicKeyV1,
}

impl NatsRuntimeCredentialDeliveryBindingV1 {
    pub fn new(
        fields: NatsRuntimeCredentialDeliveryBindingInputV1,
    ) -> Result<Self, NatsRuntimeCredentialDeliveryErrorV1> {
        (valid_id(&fields.logical_owner_id)
            && valid_id(&fields.registration_id)
            && valid_id(&fields.runtime_instance_id)
            && fields.runtime_generation > 0
            && fields.grant_epoch > 0
            && fields.credential_revision > 0
            && fields.request_id.iter().any(|byte| *byte != 0))
        .then_some(Self {
            logical_owner_id: fields.logical_owner_id,
            registration_id: fields.registration_id,
            runtime_instance_id: fields.runtime_instance_id,
            runtime_generation: fields.runtime_generation,
            grant_epoch: fields.grant_epoch,
            credential_revision: fields.credential_revision,
            request_id: fields.request_id,
            recipient_public_key: fields.recipient_public_key,
        })
        .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidBinding)
    }

    #[must_use]
    pub const fn request_id(&self) -> &[u8; REQUEST_ID_BYTES] {
        &self.request_id
    }

    #[must_use]
    pub fn recipient_public_key(&self) -> &NatsRuntimeCredentialRecipientPublicKeyV1 {
        &self.recipient_public_key
    }

    fn matches_recipient(
        &self,
        recipient: &NatsRuntimeCredentialRecipientPublicKeyV1,
    ) -> Result<(), NatsRuntimeCredentialDeliveryErrorV1> {
        (self.recipient_public_key == *recipient)
            .then_some(())
            .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidBinding)
    }

    fn aad(&self) -> Vec<u8> {
        let mut aad = Vec::with_capacity(512);
        aad.extend_from_slice(b"HENATSC1");
        append_field(&mut aad, self.logical_owner_id.as_bytes());
        append_field(&mut aad, self.registration_id.as_bytes());
        append_field(&mut aad, self.runtime_instance_id.as_bytes());
        aad.extend_from_slice(&self.runtime_generation.to_be_bytes());
        aad.extend_from_slice(&self.grant_epoch.to_be_bytes());
        aad.extend_from_slice(&self.credential_revision.to_be_bytes());
        aad.extend_from_slice(&self.request_id);
        aad.extend_from_slice(self.recipient_public_key.as_bytes());
        aad
    }
}

/// Opaque ciphertext relayed by Kernel and opened only by its runtime recipient.
pub struct NatsRuntimeCredentialDeliveryV1 {
    encapped_key: Vec<u8>,
    ciphertext: Zeroizing<Vec<u8>>,
    tag: Vec<u8>,
}

impl NatsRuntimeCredentialDeliveryV1 {
    pub fn from_parts(
        encapped_key: Vec<u8>,
        ciphertext: Vec<u8>,
        tag: Vec<u8>,
    ) -> Result<Self, NatsRuntimeCredentialDeliveryErrorV1> {
        valid_delivery_parts(&encapped_key, &ciphertext, &tag)?;
        Ok(Self {
            encapped_key,
            ciphertext: Zeroizing::new(ciphertext),
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
    pub fn tag(&self) -> &[u8] {
        &self.tag
    }
}

/// JWT and NKey proof material held only for one broker connection lifetime.
pub struct RuntimeNatsJwtCredentialV1 {
    jwt: Zeroizing<String>,
    user_seed: Zeroizing<String>,
    user_public_key: String,
    expires_at_unix_seconds: u64,
}

impl std::fmt::Debug for RuntimeNatsJwtCredentialV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeNatsJwtCredentialV1")
            .field("jwt", &"[redacted]")
            .field("user_seed", &"[redacted]")
            .field("user_public_key", &"[redacted]")
            .field("expires_at_unix_seconds", &self.expires_at_unix_seconds)
            .finish()
    }
}

impl RuntimeNatsJwtCredentialV1 {
    pub fn new(
        jwt: String,
        user_seed: String,
        user_public_key: String,
        expires_at_unix_seconds: u64,
    ) -> Result<Self, NatsRuntimeCredentialDeliveryErrorV1> {
        if jwt.is_empty()
            || user_public_key.len() != 56
            || !user_public_key.starts_with('U')
            || expires_at_unix_seconds == 0
        {
            return Err(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential);
        }
        let key = nats_jwt::KeyPair::from_seed(&user_seed)
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
        (key.public_key() == user_public_key)
            .then_some(Self {
                jwt: Zeroizing::new(jwt),
                user_seed: Zeroizing::new(user_seed),
                user_public_key,
                expires_at_unix_seconds,
            })
            .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)
    }

    #[must_use]
    pub fn user_public_key(&self) -> &str {
        &self.user_public_key
    }
    #[must_use]
    pub const fn expires_at_unix_seconds(&self) -> u64 {
        self.expires_at_unix_seconds
    }

    pub fn seal_for(
        &self,
        binding: &NatsRuntimeCredentialDeliveryBindingV1,
    ) -> Result<NatsRuntimeCredentialDeliveryV1, NatsRuntimeCredentialDeliveryErrorV1> {
        let recipient =
            <Kem as KemTrait>::PublicKey::from_bytes(binding.recipient_public_key.as_bytes())
                .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidRecipient)?;
        let plaintext = self.encode()?;
        let (encapped_key, mut sender) =
            hpke::setup_sender::<Aead, Kdf, Kem>(&OpModeS::Base, &recipient, DELIVERY_INFO)
                .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::EncryptionFailed)?;
        let mut ciphertext = plaintext.to_vec();
        let tag = sender
            .seal_inout_detached(InOutBuf::from(ciphertext.as_mut_slice()), &binding.aad())
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::EncryptionFailed)?;
        NatsRuntimeCredentialDeliveryV1::from_parts(
            encapped_key.to_bytes().as_slice().to_vec(),
            ciphertext,
            tag.to_bytes().as_slice().to_vec(),
        )
    }

    pub fn into_connection_material(self) -> (Zeroizing<String>, Zeroizing<String>, u64) {
        (self.jwt, self.user_seed, self.expires_at_unix_seconds)
    }

    fn encode(&self) -> Result<Zeroizing<Vec<u8>>, NatsRuntimeCredentialDeliveryErrorV1> {
        let jwt = self.jwt.as_bytes();
        let seed = self.user_seed.as_bytes();
        let key = self.user_public_key.as_bytes();
        let jwt_length = u16::try_from(jwt.len())
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
        let seed_length = u16::try_from(seed.len())
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
        let key_length = u8::try_from(key.len())
            .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
        let mut bytes =
            Vec::with_capacity(CREDENTIAL_MAGIC.len() + jwt.len() + seed.len() + key.len() + 13);
        bytes.extend_from_slice(CREDENTIAL_MAGIC);
        bytes.extend_from_slice(&self.expires_at_unix_seconds.to_be_bytes());
        bytes.extend_from_slice(&jwt_length.to_be_bytes());
        bytes.extend_from_slice(jwt);
        bytes.extend_from_slice(&seed_length.to_be_bytes());
        bytes.extend_from_slice(seed);
        bytes.push(key_length);
        bytes.extend_from_slice(key);
        Ok(Zeroizing::new(bytes))
    }

    fn decode(bytes: &[u8]) -> Result<Self, NatsRuntimeCredentialDeliveryErrorV1> {
        let mut cursor = CREDENTIAL_MAGIC.len();
        (bytes.get(..cursor) == Some(CREDENTIAL_MAGIC))
            .then_some(())
            .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
        let expires = read_u64(bytes, &mut cursor)?;
        let jwt_length = usize::from(read_u16(bytes, &mut cursor)?);
        let jwt = read_string(bytes, &mut cursor, jwt_length)?;
        let seed_length = usize::from(read_u16(bytes, &mut cursor)?);
        let seed = read_string(bytes, &mut cursor, seed_length)?;
        let key_length = usize::from(read_u8(bytes, &mut cursor)?);
        let key = read_string(bytes, &mut cursor, key_length)?;
        (cursor == bytes.len())
            .then_some(())
            .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
        Self::new(jwt, seed, key, expires)
    }
}

fn valid_delivery_parts(
    encapped_key: &[u8],
    ciphertext: &[u8],
    tag: &[u8],
) -> Result<(), NatsRuntimeCredentialDeliveryErrorV1> {
    if ciphertext.is_empty() || ciphertext.len() > MAX_CREDENTIAL_BYTES || tag.len() != TAG_BYTES {
        return Err(NatsRuntimeCredentialDeliveryErrorV1::InvalidDelivery);
    }
    <Kem as KemTrait>::EncappedKey::from_bytes(encapped_key)
        .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidDelivery)?;
    AeadTag::<Aead>::from_bytes(tag)
        .map(|_| ())
        .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidDelivery)
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, NatsRuntimeCredentialDeliveryErrorV1> {
    let value = bytes
        .get(*cursor..*cursor + 8)
        .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
    *cursor += 8;
    Ok(u64::from_be_bytes(value.try_into().expect("fixed length")))
}
fn read_u16(bytes: &[u8], cursor: &mut usize) -> Result<u16, NatsRuntimeCredentialDeliveryErrorV1> {
    let value = bytes
        .get(*cursor..*cursor + 2)
        .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
    *cursor += 2;
    Ok(u16::from_be_bytes(value.try_into().expect("fixed length")))
}
fn read_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8, NatsRuntimeCredentialDeliveryErrorV1> {
    let value = *bytes
        .get(*cursor)
        .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
    *cursor += 1;
    Ok(value)
}
fn read_string(
    bytes: &[u8],
    cursor: &mut usize,
    length: usize,
) -> Result<String, NatsRuntimeCredentialDeliveryErrorV1> {
    let value = bytes
        .get(*cursor..*cursor + length)
        .ok_or(NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)?;
    *cursor += length;
    String::from_utf8(value.to_vec())
        .map_err(|_| NatsRuntimeCredentialDeliveryErrorV1::InvalidCredential)
}
fn append_field(target: &mut Vec<u8>, field: &[u8]) {
    let length = u16::try_from(field.len()).expect("bounded NATS identifiers fit u16");
    target.extend_from_slice(&length.to_be_bytes());
    target.extend_from_slice(field);
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NatsRuntimeCredentialDeliveryErrorV1 {
    InvalidRecipient,
    InvalidBinding,
    InvalidDelivery,
    InvalidCredential,
    EncryptionFailed,
    AuthenticationFailed,
}

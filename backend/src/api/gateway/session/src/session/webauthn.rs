use std::time::Duration;

use url::Url;
use webauthn_rs_core::{
    WebauthnCore,
    proto::{
        AttestationConveyancePreference, AttestationFormat, COSEAlgorithm, COSEKey,
        CreationChallengeResponse, Credential, ParsedAttestation, PublicKeyCredential,
        RegisterPublicKeyCredential, RegisteredExtensions, RequestChallengeResponse,
        UserVerificationPolicy,
    },
};

const WEBAUTHN_TIMEOUT: Duration = Duration::from_secs(120);

pub struct BrowserWebauthnVerifier {
    core: WebauthnCore,
    rp_id: String,
}

#[derive(Clone)]
pub struct BrowserRegistrationCeremonyV1 {
    options: CreationChallengeResponse,
    state: webauthn_rs_core::proto::RegistrationState,
}

pub struct BrowserAuthenticationCeremonyV1 {
    options: RequestChallengeResponse,
    state: webauthn_rs_core::proto::AuthenticationState,
}

pub struct VerifiedBrowserCredentialV1 {
    credential: Credential,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCredentialMaterialV1 {
    credential_id: Vec<u8>,
    cose_public_key: Vec<u8>,
    sign_count: u32,
    backup_eligible: bool,
    backup_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserAssertionMaterialV1 {
    credential_id: Vec<u8>,
    sign_count: u32,
    backup_eligible: bool,
    backup_state: bool,
}

impl BrowserWebauthnVerifier {
    pub fn new(rp_id: &str, exact_https_origin: &str) -> Result<Self, String> {
        let origin = Url::parse(exact_https_origin).map_err(|_| "browser origin is invalid")?;
        validate_origin(rp_id, &origin)?;
        Ok(Self {
            core: WebauthnCore::new_unsafe_experts_only(
                rp_id,
                rp_id,
                vec![origin],
                WEBAUTHN_TIMEOUT,
                Some(false),
                Some(false),
            ),
            rp_id: rp_id.to_owned(),
        })
    }

    pub fn begin_registration(
        &self,
        owner_id: &str,
    ) -> Result<BrowserRegistrationCeremonyV1, String> {
        let builder = self
            .core
            .new_challenge_register_builder(owner_id.as_bytes(), owner_id, "Макошь owner")
            .map_err(webauthn_error)?;
        let (options, state) = self
            .core
            .generate_challenge_register(strict_registration(builder))
            .map_err(webauthn_error)?;
        Ok(BrowserRegistrationCeremonyV1 { options, state })
    }

    pub fn finish_registration(
        &self,
        ceremony: &BrowserRegistrationCeremonyV1,
        response: &RegisterPublicKeyCredential,
    ) -> Result<VerifiedBrowserCredentialV1, String> {
        let credential = self
            .core
            .register_credential(response, &ceremony.state, None)
            .map_err(webauthn_error)?;
        validate_credential(&credential)?;
        Ok(VerifiedBrowserCredentialV1 { credential })
    }

    pub fn begin_authentication(
        &self,
        credential: &VerifiedBrowserCredentialV1,
    ) -> Result<BrowserAuthenticationCeremonyV1, String> {
        let builder = self
            .core
            .new_challenge_authenticate_builder(
                vec![credential.credential.clone()],
                Some(UserVerificationPolicy::Required),
            )
            .map_err(webauthn_error)?
            .allow_backup_eligible_upgrade(true);
        let (options, state) = self
            .core
            .generate_challenge_authenticate(builder)
            .map_err(webauthn_error)?;
        Ok(BrowserAuthenticationCeremonyV1 { options, state })
    }

    pub fn credential_from_material(
        &self,
        material: BrowserCredentialMaterialV1,
    ) -> Result<VerifiedBrowserCredentialV1, String> {
        let credential = Credential {
            cred_id: material.credential_id.into(),
            cred: deserialize_es256_public_key(&material.cose_public_key)?,
            counter: material.sign_count,
            transports: None,
            user_verified: true,
            backup_eligible: material.backup_eligible,
            backup_state: material.backup_state,
            registration_policy: UserVerificationPolicy::Required,
            extensions: RegisteredExtensions::none(),
            attestation: ParsedAttestation::default(),
            attestation_format: AttestationFormat::None,
        };
        validate_credential(&credential)?;
        Ok(VerifiedBrowserCredentialV1 { credential })
    }

    pub fn finish_authentication(
        &self,
        ceremony: &BrowserAuthenticationCeremonyV1,
        response: &PublicKeyCredential,
    ) -> Result<BrowserAssertionMaterialV1, String> {
        let result = self
            .core
            .authenticate_credential(response, &ceremony.state)
            .map_err(webauthn_error)?;
        result
            .user_verified()
            .then_some(BrowserAssertionMaterialV1 {
                credential_id: result.cred_id().as_ref().to_vec(),
                sign_count: result.counter(),
                backup_eligible: result.backup_eligible(),
                backup_state: result.backup_state(),
            })
            .ok_or_else(|| "browser assertion does not satisfy the device-bound profile".to_owned())
    }

    pub(crate) fn rp_id(&self) -> &str {
        &self.rp_id
    }
}

impl BrowserRegistrationCeremonyV1 {
    #[must_use]
    pub const fn options(&self) -> &CreationChallengeResponse {
        &self.options
    }
}

impl BrowserAuthenticationCeremonyV1 {
    #[must_use]
    pub const fn options(&self) -> &RequestChallengeResponse {
        &self.options
    }
}

impl VerifiedBrowserCredentialV1 {
    pub fn material(&self) -> Result<BrowserCredentialMaterialV1, String> {
        Ok(BrowserCredentialMaterialV1 {
            credential_id: self.credential.cred_id.as_ref().to_vec(),
            cose_public_key: serde_cbor_2::to_vec(&self.credential.cred)
                .map_err(|_| "browser credential public key is invalid")?,
            sign_count: self.credential.counter,
            backup_eligible: self.credential.backup_eligible,
            backup_state: self.credential.backup_state,
        })
    }
}

impl BrowserCredentialMaterialV1 {
    pub fn new(
        credential_id: Vec<u8>,
        cose_public_key: Vec<u8>,
        sign_count: u32,
        backup_eligible: bool,
        backup_state: bool,
    ) -> Result<Self, String> {
        (!credential_id.is_empty()
            && credential_id.len() <= 1024
            && (16..=1024).contains(&cose_public_key.len())
            && (!backup_state || backup_eligible))
            .then_some(Self {
                credential_id,
                cose_public_key,
                sign_count,
                backup_eligible,
                backup_state,
            })
            .ok_or_else(|| "browser credential material is invalid".to_owned())
    }
    #[must_use]
    pub fn credential_id(&self) -> &[u8] {
        &self.credential_id
    }
    #[must_use]
    pub fn cose_public_key(&self) -> &[u8] {
        &self.cose_public_key
    }
    #[must_use]
    pub const fn sign_count(&self) -> u32 {
        self.sign_count
    }
    #[must_use]
    pub const fn backup_eligible(&self) -> bool {
        self.backup_eligible
    }
    #[must_use]
    pub const fn backup_state(&self) -> bool {
        self.backup_state
    }
}

impl BrowserAssertionMaterialV1 {
    #[must_use]
    pub fn credential_id(&self) -> &[u8] {
        &self.credential_id
    }
    #[must_use]
    pub const fn sign_count(&self) -> u32 {
        self.sign_count
    }
    #[must_use]
    pub const fn backup_eligible(&self) -> bool {
        self.backup_eligible
    }
    #[must_use]
    pub const fn backup_state(&self) -> bool {
        self.backup_state
    }
}

fn strict_registration(
    builder: webauthn_rs_core::ChallengeRegisterBuilder,
) -> webauthn_rs_core::ChallengeRegisterBuilder {
    builder
        .attestation(AttestationConveyancePreference::None)
        .credential_algorithms(vec![COSEAlgorithm::ES256])
        .user_verification_policy(UserVerificationPolicy::Required)
        .reject_synchronised_authenticators(false)
}

fn validate_origin(rp_id: &str, origin: &Url) -> Result<(), String> {
    let domain = origin
        .domain()
        .ok_or_else(|| "browser origin is invalid".to_owned())?;
    (origin.scheme() == "https"
        && origin.username().is_empty()
        && origin.password().is_none()
        && origin.path() == "/"
        && origin.query().is_none()
        && origin.fragment().is_none()
        && (domain == rp_id || domain.ends_with(&format!(".{rp_id}"))))
    .then_some(())
    .ok_or_else(|| "browser origin does not match relying party".to_owned())
}

fn validate_credential(credential: &Credential) -> Result<(), String> {
    credential
        .user_verified
        .then_some(())
        .ok_or_else(|| "browser credential does not satisfy the device-bound profile".to_owned())
}

fn deserialize_es256_public_key(value: &[u8]) -> Result<COSEKey, String> {
    let key: COSEKey = serde_cbor_2::from_slice(value)
        .map_err(|_| "browser credential public key is invalid".to_owned())?;
    (key.type_ == COSEAlgorithm::ES256)
        .then_some(key)
        .ok_or_else(|| "browser credential algorithm is invalid".to_owned())
}

fn webauthn_error(error: impl std::fmt::Display) -> String {
    format!("browser WebAuthn verification failed: {error}")
}

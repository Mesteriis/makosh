use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::webauthn::{
    BrowserRegistrationCeremonyV1, BrowserWebauthnVerifier, VerifiedBrowserCredentialV1,
};
use makosh_gateway_session_contract::{
    BrowserDevicePrincipalV1, BrowserEnrollmentAuthority, BrowserEnrollmentInputV1,
    BrowserEnrollmentV1, BrowserPairingAuthority, GatewayIdentityFenceV1,
};
use webauthn_rs_core::proto::{CreationChallengeResponse, RegisterPublicKeyCredential};

const PAIRING_TTL: Duration = Duration::from_secs(120);
const MAX_PENDING_PAIRINGS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerPairingApprovalV1 {
    owner_id: String,
    authorizing_device_id: String,
}

impl OwnerPairingApprovalV1 {
    pub fn new(
        owner_id: impl Into<String>,
        authorizing_device_id: impl Into<String>,
    ) -> Result<Self, String> {
        let owner_id = owner_id.into();
        let authorizing_device_id = authorizing_device_id.into();
        (valid_id(&owner_id) && valid_id(&authorizing_device_id))
            .then_some(Self {
                owner_id,
                authorizing_device_id,
            })
            .ok_or_else(|| "browser pairing approval is invalid".to_owned())
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub fn authorizing_device_id(&self) -> &str {
        &self.authorizing_device_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserPairingChallengeV1 {
    pairing_id: String,
    challenge_bytes: [u8; 32],
    owner_id: String,
    rp_id: String,
    expires_at_unix_millis: u64,
}

#[derive(Clone)]
pub struct BrowserWebauthnPairingCeremonyV1 {
    pairing: BrowserPairingChallengeV1,
    options: CreationChallengeResponse,
}

impl BrowserWebauthnPairingCeremonyV1 {
    #[must_use]
    pub fn pairing(&self) -> &BrowserPairingChallengeV1 {
        &self.pairing
    }
    #[must_use]
    pub const fn options(&self) -> &CreationChallengeResponse {
        &self.options
    }
}

impl BrowserPairingChallengeV1 {
    #[must_use]
    pub fn pairing_id(&self) -> &str {
        &self.pairing_id
    }
    #[must_use]
    pub fn challenge_bytes(&self) -> &[u8; 32] {
        &self.challenge_bytes
    }
    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub fn rp_id(&self) -> &str {
        &self.rp_id
    }
    #[must_use]
    pub const fn expires_at_unix_millis(&self) -> u64 {
        self.expires_at_unix_millis
    }
}

#[derive(Clone)]
struct PendingPairing {
    approval: OwnerPairingApprovalV1,
    challenge: BrowserPairingChallengeV1,
    fence: GatewayIdentityFenceV1,
    expires_at: Instant,
    registration: Option<BrowserRegistrationCeremonyV1>,
}

#[derive(Default)]
pub struct BrowserPairingManager {
    pending: HashMap<String, PendingPairing>,
}

impl BrowserPairingManager {
    pub fn begin<A: BrowserPairingAuthority>(
        &mut self,
        authority: &A,
        approval: OwnerPairingApprovalV1,
        rp_id: impl Into<String>,
        now_unix_millis: u64,
    ) -> Result<BrowserPairingChallengeV1, String> {
        self.purge();
        authority.require_current_owner(approval.owner_id())?;
        if self.pending.len() >= MAX_PENDING_PAIRINGS {
            return Err("browser pairing capacity reached".to_owned());
        }
        let rp_id = rp_id.into();
        if !valid_rp_id(&rp_id) {
            return Err("browser relying party is invalid".to_owned());
        }
        let pairing_id = random_id()?;
        let challenge = BrowserPairingChallengeV1 {
            pairing_id: pairing_id.clone(),
            challenge_bytes: random_bytes()?,
            owner_id: approval.owner_id().to_owned(),
            rp_id,
            expires_at_unix_millis: now_unix_millis.saturating_add(PAIRING_TTL.as_millis() as u64),
        };
        self.pending.insert(
            pairing_id,
            PendingPairing {
                approval,
                challenge: challenge.clone(),
                fence: authority.current_identity_fence()?,
                expires_at: Instant::now() + PAIRING_TTL,
                registration: None,
            },
        );
        Ok(challenge)
    }

    pub fn begin_webauthn<A: BrowserPairingAuthority>(
        &mut self,
        authority: &A,
        verifier: &BrowserWebauthnVerifier,
        approval: OwnerPairingApprovalV1,
        now_unix_millis: u64,
    ) -> Result<BrowserWebauthnPairingCeremonyV1, String> {
        let registration = verifier.begin_registration(approval.owner_id())?;
        let pairing = self.begin(authority, approval, verifier.rp_id(), now_unix_millis)?;
        self.pending
            .get_mut(pairing.pairing_id())
            .ok_or_else(unavailable)?
            .registration = Some(registration.clone());
        Ok(BrowserWebauthnPairingCeremonyV1 {
            pairing,
            options: registration.options().clone(),
        })
    }

    /// Returns the server-held registration options for an already owner
    /// approved pairing. Reading options does not consume the pairing: only a
    /// verified credential persisted by `finish_webauthn_and_admit` can do so.
    pub fn registration_options(
        &mut self,
        pairing_id: &str,
    ) -> Result<CreationChallengeResponse, String> {
        self.purge();
        self.pending
            .get(pairing_id)
            .and_then(|pairing| pairing.registration.as_ref())
            .map(|registration| registration.options().clone())
            .ok_or_else(unavailable)
    }

    pub fn consume<A, T, F>(
        &mut self,
        authority: &A,
        pairing_id: &str,
        verifier: F,
    ) -> Result<T, String>
    where
        A: BrowserPairingAuthority,
        F: FnOnce(&BrowserPairingChallengeV1) -> Result<T, String>,
    {
        self.consume_pending(authority, pairing_id, |pairing| {
            verifier(&pairing.challenge)
        })
    }

    pub fn finish_webauthn_and_admit<A: BrowserEnrollmentAuthority>(
        &mut self,
        authority: &A,
        verifier: &BrowserWebauthnVerifier,
        pairing_id: &str,
        response: &RegisterPublicKeyCredential,
        browser_key_public_key: &[u8],
    ) -> Result<BrowserDevicePrincipalV1, String> {
        let device_id = random_id()?;
        self.finish_webauthn_with(
            authority,
            verifier,
            pairing_id,
            response,
            |pairing, credential| {
                let material = credential.material()?;
                let enrollment = BrowserEnrollmentV1::new(BrowserEnrollmentInputV1 {
                    owner_id: pairing.challenge.owner_id().to_owned(),
                    device_id: device_id.clone(),
                    rp_id: pairing.challenge.rp_id().to_owned(),
                    credential_id: material.credential_id().to_vec(),
                    cose_public_key: material.cose_public_key().to_vec(),
                    browser_key_public_key: browser_key_public_key.to_vec(),
                    sign_count: material.sign_count(),
                    backup_eligible: material.backup_eligible(),
                    backup_state: material.backup_state(),
                    identity_fence: pairing.fence.clone(),
                })?;
                authority.admit_browser_device(&enrollment)
            },
        )
    }

    fn consume_pending<A, T, F>(
        &mut self,
        authority: &A,
        pairing_id: &str,
        verifier: F,
    ) -> Result<T, String>
    where
        A: BrowserPairingAuthority,
        F: FnOnce(&PendingPairing) -> Result<T, String>,
    {
        self.purge();
        let pairing = self
            .pending
            .get(pairing_id)
            .cloned()
            .ok_or_else(unavailable)?;
        authority.require_current_owner(pairing.approval.owner_id())?;
        if authority.current_identity_fence()? != pairing.fence {
            return Err("browser pairing is stale".to_owned());
        }
        let result = verifier(&pairing)?;
        self.pending.remove(pairing_id);
        Ok(result)
    }

    pub fn finish_webauthn<A, T, F>(
        &mut self,
        authority: &A,
        verifier: &BrowserWebauthnVerifier,
        pairing_id: &str,
        response: &RegisterPublicKeyCredential,
        persist: F,
    ) -> Result<T, String>
    where
        A: BrowserPairingAuthority,
        F: FnOnce(&VerifiedBrowserCredentialV1) -> Result<T, String>,
    {
        self.finish_webauthn_with(
            authority,
            verifier,
            pairing_id,
            response,
            |_, credential| persist(credential),
        )
    }

    fn finish_webauthn_with<A, T, F>(
        &mut self,
        authority: &A,
        verifier: &BrowserWebauthnVerifier,
        pairing_id: &str,
        response: &RegisterPublicKeyCredential,
        persist: F,
    ) -> Result<T, String>
    where
        A: BrowserPairingAuthority,
        F: FnOnce(&PendingPairing, &VerifiedBrowserCredentialV1) -> Result<T, String>,
    {
        let registration = self
            .pending
            .get(pairing_id)
            .and_then(|pairing| pairing.registration.clone())
            .ok_or_else(unavailable)?;
        self.consume_pending(authority, pairing_id, |pairing| {
            let credential = verifier.finish_registration(&registration, response)?;
            persist(pairing, &credential)
        })
    }

    fn purge(&mut self) {
        let now = Instant::now();
        self.pending.retain(|_, pairing| pairing.expires_at > now);
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_rp_id(value: &str) -> bool {
    value == "localhost"
        || (value.len() <= 253
            && value.split('.').count() >= 2
            && value.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                    && !label.starts_with('-')
                    && !label.ends_with('-')
            }))
}

fn unavailable() -> String {
    "browser pairing is unavailable".to_owned()
}

fn random_bytes() -> Result<[u8; 32], String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn random_id() -> Result<String, String> {
    Ok(random_bytes()?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

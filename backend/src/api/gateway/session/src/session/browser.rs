use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::webauthn::{
    BrowserAssertionMaterialV1, BrowserAuthenticationCeremonyV1, BrowserCredentialMaterialV1,
    BrowserWebauthnVerifier,
};
use makosh_gateway_session_contract::{
    BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
    GatewayIdentityFenceV1,
};
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use webauthn_rs_core::proto::{PublicKeyCredential, RequestChallengeResponse};

const SESSION_TTL: Duration = Duration::from_secs(300);
const MAX_SESSIONS: usize = 32;
const AUTHENTICATION_TTL: Duration = Duration::from_secs(120);
const MAX_PENDING_AUTHENTICATIONS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserSession {
    session_id: String,
    owner_id: String,
    device_id: String,
    expires_at_unix_millis: u64,
}

pub struct BrowserAuthenticationFinishInput<'a> {
    pub authentication_id: &'a str,
    pub response: &'a PublicKeyCredential,
    pub browser_key_signature: &'a [u8],
    pub now_unix_millis: u64,
}

/// Server-held WebAuthn state for one browser authentication attempt.
#[derive(Clone)]
pub struct BrowserWebauthnAuthenticationCeremonyV1 {
    authentication_id: String,
    options: RequestChallengeResponse,
    browser_key_challenge: [u8; 32],
}

impl BrowserWebauthnAuthenticationCeremonyV1 {
    #[must_use]
    pub fn authentication_id(&self) -> &str {
        &self.authentication_id
    }

    #[must_use]
    pub const fn options(&self) -> &RequestChallengeResponse {
        &self.options
    }

    #[must_use]
    pub const fn browser_key_challenge(&self) -> &[u8; 32] {
        &self.browser_key_challenge
    }
}

impl BrowserSession {
    pub(crate) fn lan_development(
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
    ) -> Result<Self, String> {
        Self::development("lan-development", owner_id, device_id)
    }

    pub(crate) fn loopback_development(
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
    ) -> Result<Self, String> {
        Self::development("loopback-development", owner_id, device_id)
    }

    fn development(
        session_id: &str,
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
    ) -> Result<Self, String> {
        let owner_id = owner_id.into();
        let device_id = device_id.into();
        (!owner_id.is_empty() && !device_id.is_empty())
            .then_some(Self {
                session_id: session_id.to_owned(),
                owner_id,
                device_id,
                expires_at_unix_millis: u64::MAX,
            })
            .ok_or_else(|| "developer mode principal is invalid".to_owned())
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
    #[must_use]
    pub const fn expires_at_unix_millis(&self) -> u64 {
        self.expires_at_unix_millis
    }
}

#[derive(Clone)]
struct SessionFence {
    identity: GatewayIdentityFenceV1,
    expires_at: Instant,
    session: BrowserSession,
}

struct PendingAuthentication {
    ceremony: BrowserAuthenticationCeremonyV1,
    browser_key_public_key: Vec<u8>,
    browser_key_challenge: [u8; 32],
    expires_at: Instant,
}

#[derive(Default)]
pub struct BrowserSessionManager {
    sessions: HashMap<String, SessionFence>,
}

#[derive(Default)]
pub struct BrowserAuthenticationManager {
    pending: HashMap<String, PendingAuthentication>,
}

impl BrowserAuthenticationManager {
    /// Starts a bounded, server-held WebAuthn ceremony for an active credential.
    /// The caller-supplied credential ID receives no authority by itself.
    pub fn begin<A: BrowserAuthenticationAuthority>(
        &mut self,
        authority: &A,
        verifier: &BrowserWebauthnVerifier,
        credential_id: &[u8],
    ) -> Result<BrowserWebauthnAuthenticationCeremonyV1, String> {
        self.purge();
        if self.pending.len() >= MAX_PENDING_AUTHENTICATIONS {
            return Err("browser authentication capacity reached".to_owned());
        }
        let credential = authority.active_browser_credential(credential_id)?;
        let browser_key_public_key = credential.browser_key_public_key().to_vec();
        let credential = verifier.credential_from_material(BrowserCredentialMaterialV1::new(
            credential.credential_id().to_vec(),
            credential.cose_public_key().to_vec(),
            credential.sign_count(),
            credential.backup_eligible(),
            credential.backup_state(),
        )?)?;
        let ceremony = verifier.begin_authentication(&credential)?;
        let authentication_id = random_id()?;
        let browser_key_challenge = random_bytes()?;
        let options = ceremony.options().clone();
        self.pending.insert(
            authentication_id.clone(),
            PendingAuthentication {
                ceremony,
                browser_key_public_key,
                browser_key_challenge,
                expires_at: Instant::now() + AUTHENTICATION_TTL,
            },
        );
        Ok(BrowserWebauthnAuthenticationCeremonyV1 {
            authentication_id,
            options,
            browser_key_challenge,
        })
    }

    /// Verifies a single stored ceremony, persists its assertion counter via
    /// the authority, and only then issues a fenced Gateway-local session.
    pub fn finish<A: BrowserAssertionAuthority>(
        &mut self,
        authority: &A,
        verifier: &BrowserWebauthnVerifier,
        sessions: &mut BrowserSessionManager,
        input: BrowserAuthenticationFinishInput<'_>,
    ) -> Result<BrowserSession, String> {
        self.purge();
        let pending = self
            .pending
            .get(input.authentication_id)
            .ok_or_else(authentication_unavailable)?;
        verify_browser_key_proof(
            &pending.browser_key_public_key,
            &pending.browser_key_challenge,
            input.browser_key_signature,
        )?;
        let assertion = verifier.finish_authentication(&pending.ceremony, input.response)?;
        let session = sessions.begin(authority, assertion, input.now_unix_millis)?;
        self.pending.remove(input.authentication_id);
        Ok(session)
    }

    fn purge(&mut self) {
        let now = Instant::now();
        self.pending.retain(|_, pending| pending.expires_at > now);
    }
}

impl BrowserSessionManager {
    pub fn begin<A: BrowserAssertionAuthority>(
        &mut self,
        authority: &A,
        assertion: BrowserAssertionMaterialV1,
        now_unix_millis: u64,
    ) -> Result<BrowserSession, String> {
        self.purge();
        if self.sessions.len() >= MAX_SESSIONS {
            return Err("browser session capacity reached".to_owned());
        }
        let principal = authority.accept_verified_browser_assertion(
            assertion.credential_id(),
            assertion.sign_count(),
            assertion.backup_eligible(),
            assertion.backup_state(),
        )?;
        let session_id = random_id()?;
        let session = BrowserSession {
            session_id: session_id.clone(),
            owner_id: principal.owner_id().to_owned(),
            device_id: principal.device_id().to_owned(),
            expires_at_unix_millis: now_unix_millis.saturating_add(SESSION_TTL.as_millis() as u64),
        };
        self.sessions.insert(
            session_id,
            SessionFence {
                identity: authority.current_identity_fence()?,
                expires_at: Instant::now() + SESSION_TTL,
                session: session.clone(),
            },
        );
        Ok(session)
    }

    pub fn authorize<A: BrowserDeviceAuthority>(
        &mut self,
        authority: &A,
        session_id: &str,
    ) -> Result<BrowserSession, String> {
        self.purge();
        let fence = self
            .sessions
            .get(session_id)
            .cloned()
            .ok_or_else(unavailable)?;
        if authority.current_identity_fence().ok() != Some(fence.identity)
            || authority
                .active_browser_device(fence.session.device_id())
                .is_err()
        {
            self.sessions.remove(session_id);
            return Err("browser session is stale".to_owned());
        }
        Ok(fence.session)
    }

    fn purge(&mut self) {
        let now = Instant::now();
        self.sessions.retain(|_, session| session.expires_at > now);
    }
}

fn unavailable() -> String {
    "browser device session is unavailable".to_owned()
}

fn authentication_unavailable() -> String {
    "browser authentication is unavailable".to_owned()
}
fn random_id() -> Result<String, String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn random_bytes() -> Result<[u8; 32], String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn verify_browser_key_proof(
    public_key: &[u8],
    challenge: &[u8; 32],
    signature_raw: &[u8],
) -> Result<(), String> {
    if public_key.len() != 65 || public_key.first() != Some(&4) || signature_raw.len() != 64 {
        return Err(authentication_unavailable());
    }
    let key =
        VerifyingKey::from_sec1_bytes(public_key).map_err(|_| authentication_unavailable())?;
    let signature =
        Signature::from_slice(signature_raw).map_err(|_| authentication_unavailable())?;
    key.verify(challenge, &signature)
        .map_err(|_| authentication_unavailable())
}

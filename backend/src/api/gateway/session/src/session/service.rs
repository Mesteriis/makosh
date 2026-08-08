use std::sync::Mutex;

use webauthn_rs_core::proto::PublicKeyCredential;

use super::{
    BrowserAuthenticationManager, BrowserSameOriginSessionV1, BrowserSession,
    BrowserSessionManager, BrowserWebauthnAuthenticationCeremonyV1, BrowserWebauthnVerifier,
};
use makosh_gateway_session_contract::{
    BrowserAuthenticationAuthority, ClientBootstrapAuthority, ClientBootstrapProjectionV1,
};

/// Composes browser authentication without exposing server-held WebAuthn or
/// session state to a transport adapter.
pub struct BrowserGatewaySessionService<A> {
    authority: A,
    verifier: Option<BrowserWebauthnVerifier>,
    exact_https_origin: String,
    authentications: Mutex<BrowserAuthenticationManager>,
    sessions: Mutex<BrowserSessionManager>,
    access_mode: BrowserGatewayAccessModeV1,
}

#[derive(Clone)]
pub enum BrowserGatewayAccessModeV1 {
    Paired,
    LanDevelopment(BrowserSession),
    LoopbackDevelopment(BrowserSession),
}

impl<A> BrowserGatewaySessionService<A>
where
    A: BrowserAuthenticationAuthority,
{
    pub fn new(
        authority: A,
        verifier: BrowserWebauthnVerifier,
        exact_https_origin: impl Into<String>,
    ) -> Result<Self, String> {
        let exact_https_origin = exact_https_origin.into();
        BrowserSameOriginSessionV1::require_mutation_origin(
            &exact_https_origin,
            &exact_https_origin,
        )?;
        Ok(Self {
            authority,
            verifier: Some(verifier),
            exact_https_origin,
            authentications: Mutex::new(BrowserAuthenticationManager::default()),
            sessions: Mutex::new(BrowserSessionManager::default()),
            access_mode: BrowserGatewayAccessModeV1::Paired,
        })
    }

    pub fn new_lan_development(
        authority: A,
        exact_https_origin: impl Into<String>,
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
    ) -> Result<Self, String> {
        let exact_https_origin = exact_https_origin.into();
        BrowserSameOriginSessionV1::require_lan_development_origin(
            &exact_https_origin,
            &exact_https_origin,
        )?;
        Ok(Self {
            authority,
            verifier: None,
            exact_https_origin,
            authentications: Mutex::new(BrowserAuthenticationManager::default()),
            sessions: Mutex::new(BrowserSessionManager::default()),
            access_mode: BrowserGatewayAccessModeV1::LanDevelopment(
                BrowserSession::lan_development(owner_id, device_id)?,
            ),
        })
    }

    pub fn new_loopback_development(
        authority: A,
        exact_origin: impl Into<String>,
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
    ) -> Result<Self, String> {
        let exact_origin = exact_origin.into();
        BrowserSameOriginSessionV1::require_loopback_development_origin(
            &exact_origin,
            &exact_origin,
        )?;
        Ok(Self {
            authority,
            verifier: None,
            exact_https_origin: exact_origin,
            authentications: Mutex::new(BrowserAuthenticationManager::default()),
            sessions: Mutex::new(BrowserSessionManager::default()),
            access_mode: BrowserGatewayAccessModeV1::LoopbackDevelopment(
                BrowserSession::loopback_development(owner_id, device_id)?,
            ),
        })
    }

    #[must_use]
    pub const fn is_lan_development(&self) -> bool {
        matches!(
            self.access_mode,
            BrowserGatewayAccessModeV1::LanDevelopment(_)
        )
    }

    #[must_use]
    pub const fn is_loopback_development(&self) -> bool {
        matches!(
            self.access_mode,
            BrowserGatewayAccessModeV1::LoopbackDevelopment(_)
        )
    }

    pub fn begin_authentication(
        &self,
        origin: &str,
        credential_id: &[u8],
    ) -> Result<BrowserWebauthnAuthenticationCeremonyV1, String> {
        self.require_mutation_origin(origin)?;
        let verifier = self
            .verifier
            .as_ref()
            .ok_or_else(|| "browser authentication is unavailable".to_owned())?;
        self.authentications
            .lock()
            .map_err(|_| unavailable())?
            .begin(&self.authority, verifier, credential_id)
    }

    /// Finishes one ceremony and returns only the secure cookie header. The
    /// opaque session identifier never crosses this service boundary.
    pub fn finish_authentication(
        &self,
        origin: &str,
        authentication_id: &str,
        response: &PublicKeyCredential,
        browser_key_signature: &[u8],
        now_unix_millis: u64,
    ) -> Result<String, String> {
        self.require_mutation_origin(origin)?;
        let verifier = self
            .verifier
            .as_ref()
            .ok_or_else(|| "browser authentication is unavailable".to_owned())?;
        let mut authentications = self.authentications.lock().map_err(|_| unavailable())?;
        let mut sessions = self.sessions.lock().map_err(|_| unavailable())?;
        let session = authentications.finish(
            &self.authority,
            verifier,
            &mut sessions,
            super::browser::BrowserAuthenticationFinishInput {
                authentication_id,
                response,
                browser_key_signature,
                now_unix_millis,
            },
        )?;
        BrowserSameOriginSessionV1::issue_cookie(session.session_id())
    }

    pub fn authorize(&self, cookie_header: &str) -> Result<BrowserSession, String> {
        let session_id = BrowserSameOriginSessionV1::session_id_from_cookie(cookie_header)?;
        self.sessions
            .lock()
            .map_err(|_| unavailable())?
            .authorize(&self.authority, &session_id)
    }

    pub fn authorize_request(&self, cookie_header: Option<&str>) -> Result<BrowserSession, String> {
        match &self.access_mode {
            BrowserGatewayAccessModeV1::Paired => self.authorize(
                cookie_header.ok_or_else(|| "browser device session is unavailable".to_owned())?,
            ),
            BrowserGatewayAccessModeV1::LanDevelopment(session) => Ok(session.clone()),
            BrowserGatewayAccessModeV1::LoopbackDevelopment(session) => Ok(session.clone()),
        }
    }

    pub fn client_bootstrap_for(
        &self,
        session: &BrowserSession,
    ) -> Result<ClientBootstrapProjectionV1, String>
    where
        A: ClientBootstrapAuthority,
    {
        self.authority
            .client_bootstrap(session.owner_id(), session.device_id())
    }

    pub fn require_mutation_origin(&self, origin: &str) -> Result<(), String> {
        match self.access_mode {
            BrowserGatewayAccessModeV1::Paired => {
                BrowserSameOriginSessionV1::require_mutation_origin(
                    origin,
                    &self.exact_https_origin,
                )
            }
            BrowserGatewayAccessModeV1::LanDevelopment(_) => {
                BrowserSameOriginSessionV1::require_lan_development_origin(
                    origin,
                    &self.exact_https_origin,
                )
            }
            BrowserGatewayAccessModeV1::LoopbackDevelopment(_) => {
                BrowserSameOriginSessionV1::require_loopback_development_origin(
                    origin,
                    &self.exact_https_origin,
                )
            }
        }
    }
}

fn unavailable() -> String {
    "browser Gateway session service is unavailable".to_owned()
}

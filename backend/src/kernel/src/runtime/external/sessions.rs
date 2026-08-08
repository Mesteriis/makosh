//! Bounded challenge and capability sessions for one private external runtime socket.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use makosh_kernel_control_store::ExternalRuntimeAttestation;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};

use crate::modules::capability::policy::permits_external_route;
use crate::modules::capability::router::{
    ExternalCapabilityRouteRequest, authorize_external_route,
};
use crate::platform::vault::ciphertext_route::{
    self as vault_ciphertext_route, ValidatedVaultCiphertextRoute,
};

const CHALLENGE_TTL: Duration = Duration::from_secs(60);
const SESSION_TTL: Duration = Duration::from_secs(300);
const MAX_PENDING_CHALLENGES: usize = 32;
const MAX_SESSIONS: usize = 32;
const MAX_BEGINS_PER_MINUTE: usize = 16;
const PROOF_DOMAIN: &[u8] = b"makosh.external-runtime-session.v1\0";
const VAULT_ROUTE_CAPABILITY_ID: &str = "vault.lease.resolve";

#[derive(Clone)]
struct Challenge {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    distribution_sha256: [u8; 32],
    kernel_instance_id: String,
    bytes: [u8; 32],
    expires_at: Instant,
}

#[derive(Clone)]
struct Session {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
    expires_at: Instant,
}

pub struct RuntimeChallenge {
    challenge_id: String,
    bytes: [u8; 32],
    kernel_instance_id: String,
    grant_epoch: u64,
    expires_at_unix_millis: u64,
}

impl RuntimeChallenge {
    #[must_use]
    pub fn challenge_id(&self) -> &str {
        &self.challenge_id
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
    #[must_use]
    pub fn kernel_instance_id(&self) -> &str {
        &self.kernel_instance_id
    }
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
    #[must_use]
    pub fn expires_at_unix_millis(&self) -> u64 {
        self.expires_at_unix_millis
    }
}

pub struct AuthenticatedRuntimeSession {
    session_id: String,
    grant_epoch: u64,
    expires_at_unix_millis: u64,
}

pub struct AuthorizedExternalRuntimeV1 {
    registration_id: String,
    runtime_id: String,
    runtime_generation: u64,
    grant_epoch: u64,
}

impl AuthorizedExternalRuntimeV1 {
    #[must_use]
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    #[must_use]
    pub fn runtime_id(&self) -> &str {
        &self.runtime_id
    }

    #[must_use]
    pub const fn runtime_generation(&self) -> u64 {
        self.runtime_generation
    }

    #[must_use]
    pub const fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
}

impl AuthenticatedRuntimeSession {
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    #[must_use]
    pub fn grant_epoch(&self) -> u64 {
        self.grant_epoch
    }
    #[must_use]
    pub fn expires_at_unix_millis(&self) -> u64 {
        self.expires_at_unix_millis
    }
}

#[derive(Default)]
pub struct ExternalRuntimeSessions {
    challenges: HashMap<String, Challenge>,
    sessions: HashMap<String, Session>,
    begins: VecDeque<Instant>,
}

impl ExternalRuntimeSessions {
    pub fn begin(
        &mut self,
        store: &SqliteControlStore,
        registration_id: &str,
        runtime_id: &str,
        runtime_generation: u64,
        distribution_sha256: [u8; 32],
    ) -> Result<RuntimeChallenge, String> {
        self.purge();
        self.admit_begin()?;
        let grants = current_grants(store, registration_id)?;
        if store
            .external_runtime_identity(registration_id)
            .map_err(format_store_error)?
            .is_none()
        {
            return Err("external runtime identity is not bound".to_owned());
        }
        let bytes = random_bytes()?;
        let challenge_id = random_id()?;
        let now = Instant::now();
        let expires_at = now + CHALLENGE_TTL;
        let kernel_instance_id = store.snapshot().instance_id().to_owned();
        self.challenges.insert(
            challenge_id.clone(),
            Challenge {
                registration_id: registration_id.to_owned(),
                runtime_id: runtime_id.to_owned(),
                runtime_generation,
                grant_epoch: grants.grant_epoch(),
                distribution_sha256,
                kernel_instance_id: kernel_instance_id.clone(),
                bytes,
                expires_at,
            },
        );
        Ok(RuntimeChallenge {
            challenge_id,
            bytes,
            kernel_instance_id,
            grant_epoch: grants.grant_epoch(),
            expires_at_unix_millis: unix_millis_after(CHALLENGE_TTL)?,
        })
    }

    pub fn complete(
        &mut self,
        store: &SqliteControlStore,
        challenge_id: &str,
        signature_raw: &[u8],
    ) -> Result<AuthenticatedRuntimeSession, String> {
        self.purge();
        let challenge = self
            .challenges
            .remove(challenge_id)
            .ok_or_else(|| "runtime challenge is unavailable".to_owned())?;
        if signature_raw.len() != 64 {
            return Err("external runtime signature is invalid".to_owned());
        }
        let grants = current_grants(store, &challenge.registration_id)?;
        if grants.grant_epoch() != challenge.grant_epoch {
            return Err("external runtime challenge is stale".to_owned());
        }
        let identity = store
            .external_runtime_identity(&challenge.registration_id)
            .map_err(format_store_error)?
            .ok_or_else(|| "external runtime identity is not bound".to_owned())?;
        let verifying_key = VerifyingKey::from_sec1_bytes(identity.public_key_sec1())
            .map_err(|_| "external runtime identity is invalid".to_owned())?;
        let signature = Signature::from_slice(signature_raw)
            .map_err(|_| "external runtime signature is invalid".to_owned())?;
        verifying_key
            .verify(&proof_message(&challenge)?, &signature)
            .map_err(|_| "external runtime proof verification failed".to_owned())?;
        ensure_attestation(store, &challenge, grants.grant_epoch())?;
        if self.sessions.len() >= MAX_SESSIONS {
            return Err("runtime session rate limited".to_owned());
        }
        let session_id = random_id()?;
        self.sessions.insert(
            session_id.clone(),
            Session {
                registration_id: challenge.registration_id,
                runtime_id: challenge.runtime_id,
                runtime_generation: challenge.runtime_generation,
                grant_epoch: challenge.grant_epoch,
                expires_at: Instant::now() + SESSION_TTL,
            },
        );
        Ok(AuthenticatedRuntimeSession {
            session_id,
            grant_epoch: grants.grant_epoch(),
            expires_at_unix_millis: unix_millis_after(SESSION_TTL)?,
        })
    }

    pub fn authorize(
        &mut self,
        store: &SqliteControlStore,
        session_id: &str,
        capability_id: &str,
    ) -> Result<u64, String> {
        self.purge();
        if !permits_external_route(capability_id) {
            return Err("capability route is prohibited by Kernel policy".to_owned());
        }
        let session = self
            .sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| "runtime session is unavailable".to_owned())?;
        let route = authorize_external_route(
            store,
            &ExternalCapabilityRouteRequest::new(
                &session.registration_id,
                &session.runtime_id,
                session.runtime_generation,
                capability_id,
            ),
        );
        match route {
            Ok(route) if route.grant_epoch() == session.grant_epoch => Ok(route.grant_epoch()),
            Ok(_) | Err(_) => {
                self.sessions.remove(session_id);
                Err("runtime session is stale or unauthorized".to_owned())
            }
        }
    }

    pub fn authorize_registration_action(
        &mut self,
        store: &SqliteControlStore,
        session_id: &str,
    ) -> Result<String, String> {
        self.purge();
        let session = self
            .sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| "runtime session is unavailable".to_owned())?;
        let grants = current_attestation(
            store,
            &session.registration_id,
            &session.runtime_id,
            session.runtime_generation,
        );
        match grants {
            Ok(grants) if grants.grant_epoch() == session.grant_epoch => {
                Ok(session.registration_id)
            }
            Ok(_) | Err(_) => {
                self.sessions.remove(session_id);
                Err("runtime session is stale or unauthorized".to_owned())
            }
        }
    }

    pub fn authorize_storage_binding(
        &mut self,
        store: &SqliteControlStore,
        session_id: &str,
        capability_id: &str,
    ) -> Result<AuthorizedExternalRuntimeV1, String> {
        self.purge();
        let session = self
            .sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| "runtime session is unavailable".to_owned())?;
        let grant_epoch = self.authorize(store, session_id, capability_id)?;
        (grant_epoch == session.grant_epoch)
            .then_some(AuthorizedExternalRuntimeV1 {
                registration_id: session.registration_id,
                runtime_id: session.runtime_id,
                runtime_generation: session.runtime_generation,
                grant_epoch,
            })
            .ok_or_else(|| "runtime session is stale or unauthorized".to_owned())
    }

    pub fn authorize_vault_route(
        &mut self,
        store: &SqliteControlStore,
        session_id: &str,
        vault_runtime_generation: u64,
        route: makosh_runtime_protocol::v1::VaultCiphertextRouteV1,
    ) -> Result<ValidatedVaultCiphertextRoute, String> {
        self.purge();
        let session = self
            .sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| "runtime session is unavailable".to_owned())?;
        let request = ExternalCapabilityRouteRequest::new(
            &session.registration_id,
            &session.runtime_id,
            session.runtime_generation,
            VAULT_ROUTE_CAPABILITY_ID,
        );
        let authorization = authorize_external_route(store, &request);
        match authorization {
            Ok(authorization) if authorization.grant_epoch() == session.grant_epoch => {
                let validated = vault_ciphertext_route::validate_for_authorized_external_runtime(
                    &authorization,
                    &request,
                    vault_runtime_generation,
                    route,
                )?;
                crate::runtime::external::storage::validate_vault_credential_fence(
                    store,
                    &session.registration_id,
                    &session.runtime_id,
                    session.runtime_generation,
                    session.grant_epoch,
                    validated.route(),
                )?;
                Ok(validated)
            }
            Ok(_) | Err(_) => {
                self.sessions.remove(session_id);
                Err("runtime session is stale or unauthorized".to_owned())
            }
        }
    }

    fn admit_begin(&mut self) -> Result<(), String> {
        let now = Instant::now();
        while self
            .begins
            .front()
            .is_some_and(|item| now.duration_since(*item) >= Duration::from_secs(60))
        {
            self.begins.pop_front();
        }
        if self.begins.len() >= MAX_BEGINS_PER_MINUTE
            || self.challenges.len() >= MAX_PENDING_CHALLENGES
        {
            return Err("runtime session rate limited".to_owned());
        }
        self.begins.push_back(now);
        Ok(())
    }

    fn purge(&mut self) {
        let now = Instant::now();
        self.challenges
            .retain(|_, challenge| challenge.expires_at > now);
        self.sessions.retain(|_, session| session.expires_at > now);
    }
}

fn current_attestation(
    store: &SqliteControlStore,
    registration_id: &str,
    runtime_id: &str,
    runtime_generation: u64,
) -> Result<makosh_kernel_control_store::GrantSet, String> {
    let snapshot = store
        .module_grant_snapshot(registration_id)
        .map_err(format_store_error)?
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    let grants = snapshot
        .effective_grants()
        .ok_or_else(|| "module registration is not approved".to_owned())?;
    let attestation = store
        .effective_external_runtime_attestation(registration_id)
        .map_err(format_store_error)?
        .ok_or_else(|| "external runtime requires a current attestation".to_owned())?;
    if attestation.runtime_id() != runtime_id
        || attestation.runtime_generation() != runtime_generation
        || attestation.grant_epoch() != grants.grant_epoch()
    {
        return Err("external runtime attestation is stale".to_owned());
    }
    Ok(grants.clone())
}

fn current_grants(
    store: &SqliteControlStore,
    registration_id: &str,
) -> Result<makosh_kernel_control_store::GrantSet, String> {
    store
        .module_grant_snapshot(registration_id)
        .map_err(format_store_error)?
        .and_then(|snapshot| snapshot.effective_grants().cloned())
        .ok_or_else(|| "module registration is not approved".to_owned())
}

fn ensure_attestation(
    store: &SqliteControlStore,
    challenge: &Challenge,
    grant_epoch: u64,
) -> Result<(), String> {
    let expected = ExternalRuntimeAttestation::new(
        &challenge.registration_id,
        &challenge.runtime_id,
        challenge.runtime_generation,
        grant_epoch,
        challenge.distribution_sha256,
    );
    let current = store
        .effective_external_runtime_attestation(&challenge.registration_id)
        .map_err(format_store_error)?;
    if current.as_ref() == Some(&expected) {
        return Ok(());
    }
    store
        .attest_external_runtime(&expected)
        .map_err(format_store_error)
}

fn proof_message(challenge: &Challenge) -> Result<Vec<u8>, String> {
    let mut message = Vec::with_capacity(PROOF_DOMAIN.len() + 128);
    message.extend_from_slice(PROOF_DOMAIN);
    for text in [
        &challenge.kernel_instance_id,
        &challenge.registration_id,
        &challenge.runtime_id,
    ] {
        let length = u16::try_from(text.len())
            .map_err(|_| "external runtime proof field is too large".to_owned())?;
        message.extend_from_slice(&length.to_be_bytes());
        message.extend_from_slice(text.as_bytes());
    }
    message.extend_from_slice(&challenge.runtime_generation.to_be_bytes());
    message.extend_from_slice(&challenge.grant_epoch.to_be_bytes());
    message.extend_from_slice(&challenge.distribution_sha256);
    message.extend_from_slice(&challenge.bytes);
    Ok(message)
}

fn random_id() -> Result<String, String> {
    let bytes = random_bytes()?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn random_bytes() -> Result<[u8; 32], String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn unix_millis_after(duration: Duration) -> Result<u64, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    u64::try_from(now.as_millis() + duration.as_millis())
        .map_err(|_| "runtime session expiry overflowed".to_owned())
}

fn format_store_error(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}

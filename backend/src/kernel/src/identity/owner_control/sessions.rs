//! Owner-device proof sessions for the private owner-control socket.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use makosh_gateway_protocol::owner_control_proof::owner_control_proof_message_v1;
use makosh_kernel_control_store::InitialOwnerIdentity;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};

const CHALLENGE_TTL: Duration = Duration::from_secs(60);
const SESSION_TTL: Duration = Duration::from_secs(300);
const MAX_PENDING_CHALLENGES: usize = 16;
const MAX_SESSIONS: usize = 16;
const MAX_BEGINS_PER_MINUTE: usize = 8;

#[derive(Clone)]
struct Challenge {
    kernel_instance_id: String,
    control_store_generation: u64,
    owner: InitialOwnerIdentity,
    bytes: [u8; 32],
    expires_at: Instant,
}

#[derive(Clone)]
struct Session {
    kernel_instance_id: String,
    control_store_generation: u64,
    owner: InitialOwnerIdentity,
    expires_at: Instant,
}

pub struct OwnerControlChallenge {
    challenge_id: String,
    bytes: [u8; 32],
    kernel_instance_id: String,
    owner_id: String,
    device_id: String,
    control_store_generation: u64,
    expires_at_unix_millis: u64,
}

impl OwnerControlChallenge {
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
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }
    #[must_use]
    pub fn control_store_generation(&self) -> u64 {
        self.control_store_generation
    }
    #[must_use]
    pub fn expires_at_unix_millis(&self) -> u64 {
        self.expires_at_unix_millis
    }
}

pub struct OwnerControlSession {
    session_id: String,
    expires_at_unix_millis: u64,
}

impl OwnerControlSession {
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    #[must_use]
    pub fn expires_at_unix_millis(&self) -> u64 {
        self.expires_at_unix_millis
    }
}

#[derive(Default)]
pub struct OwnerControlSessions {
    challenges: HashMap<String, Challenge>,
    sessions: HashMap<String, Session>,
    begins: VecDeque<Instant>,
}

impl OwnerControlSessions {
    pub fn begin(&mut self, store: &SqliteControlStore) -> Result<OwnerControlChallenge, String> {
        self.purge();
        self.admit_begin()?;
        let owner = current_owner(store)?;
        let bytes = random_bytes()?;
        let challenge_id = random_id()?;
        let kernel_instance_id = store.snapshot().instance_id().to_owned();
        let control_store_generation = store.snapshot().generation();
        self.challenges.insert(
            challenge_id.clone(),
            Challenge {
                kernel_instance_id: kernel_instance_id.clone(),
                control_store_generation,
                owner: owner.clone(),
                bytes,
                expires_at: Instant::now() + CHALLENGE_TTL,
            },
        );
        Ok(OwnerControlChallenge {
            challenge_id,
            bytes,
            kernel_instance_id,
            owner_id: owner.owner_id().to_owned(),
            device_id: owner.device_id().to_owned(),
            control_store_generation,
            expires_at_unix_millis: unix_millis_after(CHALLENGE_TTL)?,
        })
    }

    pub fn complete(
        &mut self,
        store: &SqliteControlStore,
        challenge_id: &str,
        signature_raw: &[u8],
    ) -> Result<OwnerControlSession, String> {
        self.purge();
        let challenge = self
            .challenges
            .remove(challenge_id)
            .ok_or_else(|| "owner control challenge is unavailable".to_owned())?;
        if signature_raw.len() != 64 {
            return Err("owner control signature is invalid".to_owned());
        }
        if store.snapshot().instance_id() != challenge.kernel_instance_id
            || store.snapshot().generation() != challenge.control_store_generation
            || current_owner(store)? != challenge.owner
        {
            return Err("owner control challenge is stale".to_owned());
        }
        let verifying_key = VerifyingKey::from_sec1_bytes(challenge.owner.public_key_sec1())
            .map_err(|_| "owner control identity is invalid".to_owned())?;
        let signature = Signature::from_slice(signature_raw)
            .map_err(|_| "owner control signature is invalid".to_owned())?;
        verifying_key
            .verify(&proof_message(&challenge)?, &signature)
            .map_err(|_| "owner control proof verification failed".to_owned())?;
        if self.sessions.len() >= MAX_SESSIONS {
            return Err("owner control session rate limited".to_owned());
        }
        let session_id = random_id()?;
        self.sessions.insert(
            session_id.clone(),
            Session {
                kernel_instance_id: challenge.kernel_instance_id,
                control_store_generation: challenge.control_store_generation,
                owner: challenge.owner,
                expires_at: Instant::now() + SESSION_TTL,
            },
        );
        Ok(OwnerControlSession {
            session_id,
            expires_at_unix_millis: unix_millis_after(SESSION_TTL)?,
        })
    }

    pub fn authorize(
        &mut self,
        store: &SqliteControlStore,
        session_id: &str,
    ) -> Result<(), String> {
        self.purge();
        let session = self
            .sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| "owner control session is unavailable".to_owned())?;
        let current = current_owner(store)?;
        if store.snapshot().instance_id() != session.kernel_instance_id
            || store.snapshot().generation() != session.control_store_generation
            || current != session.owner
        {
            self.sessions.remove(session_id);
            return Err("owner control session is stale".to_owned());
        }
        Ok(())
    }

    pub fn authorized_owner(
        &mut self,
        store: &SqliteControlStore,
        session_id: &str,
    ) -> Result<InitialOwnerIdentity, String> {
        self.authorize(store, session_id)?;
        current_owner(store)
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
            return Err("owner control session rate limited".to_owned());
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

fn current_owner(store: &SqliteControlStore) -> Result<InitialOwnerIdentity, String> {
    store
        .initial_owner_identity()
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "owner control requires an enrolled owner".to_owned())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn proof_message(challenge: &Challenge) -> Result<Vec<u8>, String> {
    owner_control_proof_message_v1(
        &challenge.kernel_instance_id,
        challenge.owner.owner_id(),
        challenge.owner.device_id(),
        challenge.control_store_generation,
        &challenge.bytes,
    )
}

fn random_id() -> Result<String, String> {
    Ok(random_bytes()?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
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
        .map_err(|_| "owner control session expiry overflowed".to_owned())
}

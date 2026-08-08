//! Bounded volatile state for the owner provisioning proof ceremony.

use std::collections::HashMap;

use makosh_gateway_protocol::v1::{
    CommitOwnerVaultProvisioningResponseV1, PrepareOwnerVaultProvisioningRequestV1,
};

const MAX_PENDING_CHALLENGES: usize = 64;
const MAX_AUTHORIZED_SESSIONS: usize = 64;

#[derive(Clone)]
pub(super) struct PendingChallengeV1 {
    pub(super) principal_owner_id: String,
    pub(super) principal_device_id: String,
    pub(super) principal_session_id: String,
    pub(super) request: PrepareOwnerVaultProvisioningRequestV1,
    pub(super) challenge_bytes: [u8; 32],
    pub(super) control_generation: u64,
    pub(super) identity_epoch: u64,
    pub(super) grant_epoch: u64,
    pub(super) expires_at_unix_millis: u64,
}

#[derive(Clone)]
pub(super) struct AuthorizedProvisioningSessionV1 {
    pub(super) principal_owner_id: String,
    pub(super) principal_device_id: String,
    pub(super) principal_session_id: String,
    pub(super) request: PrepareOwnerVaultProvisioningRequestV1,
    pub(super) expires_at_unix_millis: u64,
    pub(super) vault_runtime_generation: u64,
    pub(super) audience_registration_id: String,
    pub(super) audience_runtime_instance_id: String,
    pub(super) audience_runtime_generation: u64,
    pub(super) audience_grant_epoch: u64,
    pub(super) command_request_id: [u8; 16],
    pub(super) cached_commit: Option<CachedCommitV1>,
}

#[derive(Clone)]
pub(super) struct CachedCommitV1 {
    pub(super) request_fingerprint_sha256: [u8; 32],
    pub(super) response: CommitOwnerVaultProvisioningResponseV1,
}

#[derive(Default)]
pub(super) struct OwnerProvisioningStateV1 {
    pending: HashMap<String, PendingChallengeV1>,
    authorized: HashMap<String, AuthorizedProvisioningSessionV1>,
}

impl OwnerProvisioningStateV1 {
    pub(super) fn insert_pending(
        &mut self,
        challenge_id: String,
        challenge: PendingChallengeV1,
        now_unix_millis: u64,
    ) -> Result<(), ()> {
        self.expire(now_unix_millis);
        if self.pending.len() >= MAX_PENDING_CHALLENGES {
            return Err(());
        }
        self.pending.insert(challenge_id, challenge);
        Ok(())
    }

    pub(super) fn take_pending(
        &mut self,
        challenge_id: &str,
        now_unix_millis: u64,
    ) -> Option<PendingChallengeV1> {
        self.expire(now_unix_millis);
        self.pending.remove(challenge_id)
    }

    pub(super) fn insert_authorized(
        &mut self,
        session_id: String,
        session: AuthorizedProvisioningSessionV1,
        now_unix_millis: u64,
    ) -> Result<(), ()> {
        self.expire(now_unix_millis);
        if self.authorized.len() >= MAX_AUTHORIZED_SESSIONS {
            return Err(());
        }
        self.authorized.insert(session_id, session);
        Ok(())
    }

    pub(super) fn authorized(
        &mut self,
        session_id: &str,
        now_unix_millis: u64,
    ) -> Option<&mut AuthorizedProvisioningSessionV1> {
        self.expire(now_unix_millis);
        self.authorized.get_mut(session_id)
    }

    fn expire(&mut self, now_unix_millis: u64) {
        self.pending
            .retain(|_, challenge| challenge.expires_at_unix_millis > now_unix_millis);
        self.authorized
            .retain(|_, session| session.expires_at_unix_millis > now_unix_millis);
    }
}

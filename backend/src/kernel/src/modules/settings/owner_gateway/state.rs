//! Bounded volatile state for one owner Settings proof ceremony.

use std::collections::HashMap;

use makosh_gateway_protocol::v1::PrepareOwnerModuleSettingsRequestV1;

const MAX_PENDING_CHALLENGES: usize = 64;

#[derive(Clone)]
pub(super) struct PendingOwnerSettingsChallengeV1 {
    pub(super) principal_owner_id: String,
    pub(super) principal_device_id: String,
    pub(super) principal_session_id: String,
    pub(super) request: PrepareOwnerModuleSettingsRequestV1,
    pub(super) challenge_bytes: [u8; 32],
    pub(super) control_generation: u64,
    pub(super) identity_epoch: u64,
    pub(super) grant_epoch: u64,
    pub(super) expires_at_unix_millis: u64,
}

#[derive(Default)]
pub(super) struct OwnerSettingsChallengeStateV1 {
    pending: HashMap<String, PendingOwnerSettingsChallengeV1>,
}

impl OwnerSettingsChallengeStateV1 {
    pub(super) fn insert(
        &mut self,
        challenge_id: String,
        challenge: PendingOwnerSettingsChallengeV1,
        now_unix_millis: u64,
    ) -> Result<(), ()> {
        self.expire(now_unix_millis);
        if self.pending.len() >= MAX_PENDING_CHALLENGES {
            return Err(());
        }
        self.pending.insert(challenge_id, challenge);
        Ok(())
    }

    pub(super) fn take(
        &mut self,
        challenge_id: &str,
        now_unix_millis: u64,
    ) -> Option<PendingOwnerSettingsChallengeV1> {
        self.expire(now_unix_millis);
        self.pending.remove(challenge_id)
    }

    fn expire(&mut self, now_unix_millis: u64) {
        self.pending
            .retain(|_, challenge| challenge.expires_at_unix_millis > now_unix_millis);
    }
}

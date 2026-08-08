//! Vault-private authentication and replay fencing for HPKE transport sessions.

use std::collections::BTreeMap;

use makosh_runtime_protocol::v1::VaultCiphertextRouteV1;
use makosh_runtime_protocol::vault_request_id::vault_transport_request_position_v1;
use makosh_vault_protocol::{
    LeaseAudienceV1, SecretClassV1, VaultActionV1, VaultTransportBindingV1,
    VaultTransportCommandV1, VaultTransportDirectionV1, VaultTransportError,
    VaultTransportSessionV1,
};
use zeroize::Zeroizing;

use crate::service::runtime::{VaultService, VaultServiceError};
use crate::transport::keys::VaultTransportKeyPair;

pub const MAX_TRANSPORT_REPLAY_STREAMS: usize = 1_024;

type VaultTransportReplayStreamKeyV1 = (String, String, u64, u64, u64);

pub struct VaultTransportReplayGuard {
    runtime_generation: u64,
    stream_high_watermarks: BTreeMap<VaultTransportReplayStreamKeyV1, u64>,
}

impl VaultTransportReplayGuard {
    #[must_use]
    pub fn new(runtime_generation: u64) -> Self {
        Self {
            runtime_generation,
            stream_high_watermarks: BTreeMap::new(),
        }
    }

    pub fn open_command(
        &mut self,
        keys: &VaultTransportKeyPair,
        session: &VaultTransportSessionV1,
    ) -> Result<VaultTransportCommandV1, VaultTransportError> {
        self.validate_binding(session)?;
        let plaintext = keys.open(session.binding(), session.frame())?;
        let command = VaultTransportCommandV1::decode(&plaintext)
            .map_err(|_| VaultTransportError::InvalidBinding)?;
        if command.operation_digest() != *session.binding().operation_digest() {
            return Err(VaultTransportError::InvalidBinding);
        }
        self.consume_request_id(session.binding().audience(), session.binding().request_id())?;
        Ok(command)
    }

    fn validate_binding(
        &self,
        session: &VaultTransportSessionV1,
    ) -> Result<(), VaultTransportError> {
        if session.binding().vault_runtime_generation() != self.runtime_generation {
            return Err(VaultTransportError::InvalidBinding);
        }
        if session.binding().direction() != VaultTransportDirectionV1::ToVault {
            return Err(VaultTransportError::WrongDirection);
        }
        Ok(())
    }

    fn consume_request_id(
        &mut self,
        audience: &LeaseAudienceV1,
        request_id: &[u8; 16],
    ) -> Result<(), VaultTransportError> {
        let (stream_id, sequence) = vault_transport_request_position_v1(request_id)
            .ok_or(VaultTransportError::InvalidBinding)?;
        let stream_key = replay_stream_key(audience, stream_id);
        if let Some(high_watermark) = self.stream_high_watermarks.get_mut(&stream_key) {
            if sequence <= *high_watermark {
                return Err(VaultTransportError::ReplayDetected);
            }
            *high_watermark = sequence;
            return Ok(());
        }
        if self.stream_high_watermarks.len() == MAX_TRANSPORT_REPLAY_STREAMS {
            return Err(VaultTransportError::SessionCapacityExceeded);
        }
        self.stream_high_watermarks.insert(stream_key, sequence);
        Ok(())
    }
}

fn replay_stream_key(
    audience: &LeaseAudienceV1,
    stream_id: u64,
) -> VaultTransportReplayStreamKeyV1 {
    (
        audience.module_registration_id().to_owned(),
        audience.runtime_instance_id().to_owned(),
        audience.runtime_generation(),
        audience.grant_epoch(),
        stream_id,
    )
}

pub fn execute_session(
    guard: &mut VaultTransportReplayGuard,
    keys: &VaultTransportKeyPair,
    service: &mut VaultService,
    session: &VaultTransportSessionV1,
    now_unix_seconds: u64,
) -> Result<Zeroizing<Vec<u8>>, VaultSessionExecutionError> {
    let command = guard
        .open_command(keys, session)
        .map_err(VaultSessionExecutionError::Transport)?;
    service
        .execute_command_once(&command, session.binding().audience(), now_unix_seconds)
        .map_err(VaultSessionExecutionError::Service)
}

pub fn execute_storage_session(
    guard: &mut VaultTransportReplayGuard,
    keys: &VaultTransportKeyPair,
    service: &mut VaultService,
    session: &VaultTransportSessionV1,
    route: &VaultCiphertextRouteV1,
    now_unix_seconds: u64,
) -> Result<Zeroizing<Vec<u8>>, VaultSessionExecutionError> {
    let command = guard
        .open_command(keys, session)
        .map_err(VaultSessionExecutionError::Transport)?;
    validate_storage_command(&command, session.binding(), route)
        .map_err(VaultSessionExecutionError::Transport)?;
    service
        .execute_command_once(&command, session.binding().audience(), now_unix_seconds)
        .map_err(VaultSessionExecutionError::Service)
}

fn validate_storage_command(
    command: &VaultTransportCommandV1,
    binding: &VaultTransportBindingV1,
    route: &VaultCiphertextRouteV1,
) -> Result<(), VaultTransportError> {
    match command {
        VaultTransportCommandV1::IssueLease { request } => {
            let purpose = request.purpose();
            (request.vault_runtime_generation() == binding.vault_runtime_generation()
                && request.secret_revision() == route.storage_credential_lease_revision
                && request.logical_owner_id() == route.storage_owner_id
                && request.audience() == binding.audience()
                && purpose.purpose_id() == "storage.runtime.credential"
                && purpose.configuration_instance_id() == route.storage_runtime_principal
                && purpose.allowed_secret_classes() == [SecretClassV1::PlatformCredential]
                && matches!(
                    purpose.actions(),
                    [VaultActionV1::Create] | [VaultActionV1::Resolve] | [VaultActionV1::Delete]
                ))
            .then_some(())
            .ok_or(VaultTransportError::InvalidBinding)
        }
        VaultTransportCommandV1::ResolveLease { secret_class, .. }
        | VaultTransportCommandV1::GenerateOpaqueToken { secret_class, .. } => (*secret_class
            == SecretClassV1::PlatformCredential)
            .then_some(())
            .ok_or(VaultTransportError::InvalidBinding),
        VaultTransportCommandV1::RevokeAudience => Ok(()),
        VaultTransportCommandV1::StoreLease { .. }
        | VaultTransportCommandV1::RetireLease { .. } => Err(VaultTransportError::InvalidBinding),
        VaultTransportCommandV1::DeleteLease { secret_class, .. } => (*secret_class
            == SecretClassV1::PlatformCredential)
            .then_some(())
            .ok_or(VaultTransportError::InvalidBinding),
        VaultTransportCommandV1::EnsureOwnerDerivedKey { .. }
        | VaultTransportCommandV1::ReplaceLease { .. }
        | VaultTransportCommandV1::ProvisionLease { .. } => {
            Err(VaultTransportError::InvalidBinding)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum VaultSessionExecutionError {
    Transport(VaultTransportError),
    Service(VaultServiceError),
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::vault_request_id::vault_transport_request_id_v1;

    use super::*;

    fn audience(index: usize) -> LeaseAudienceV1 {
        LeaseAudienceV1::new(format!("module-{index}"), format!("runtime-{index}"), 1, 1)
            .expect("valid audience")
    }

    #[test]
    fn one_audience_accepts_more_than_the_old_request_capacity() {
        let mut guard = VaultTransportReplayGuard::new(1);
        let audience = audience(1);

        for sequence in 1..=u64::try_from(MAX_TRANSPORT_REPLAY_STREAMS + 1).expect("bounded test") {
            let request_id = vault_transport_request_id_v1(7, sequence).expect("positive position");
            guard
                .consume_request_id(&audience, &request_id)
                .expect("increasing sequence");
        }

        assert_eq!(guard.stream_high_watermarks.len(), 1);
    }

    #[test]
    fn repeated_lower_and_out_of_order_sequences_are_replays() {
        let mut guard = VaultTransportReplayGuard::new(1);
        let audience = audience(1);
        let second = vault_transport_request_id_v1(7, 2).expect("position");
        let first = vault_transport_request_id_v1(7, 1).expect("position");

        guard
            .consume_request_id(&audience, &second)
            .expect("first accepted sequence");
        assert!(matches!(
            guard.consume_request_id(&audience, &second),
            Err(VaultTransportError::ReplayDetected)
        ));
        assert!(matches!(
            guard.consume_request_id(&audience, &first),
            Err(VaultTransportError::ReplayDetected)
        ));
    }

    #[test]
    fn one_audience_accepts_independent_process_streams() {
        let mut guard = VaultTransportReplayGuard::new(1);
        let audience = audience(1);
        let first_stream = vault_transport_request_id_v1(7, 1).expect("position");
        let second_stream = vault_transport_request_id_v1(8, 1).expect("position");

        guard
            .consume_request_id(&audience, &first_stream)
            .expect("first process stream");
        guard
            .consume_request_id(&audience, &second_stream)
            .expect("second process stream");

        assert_eq!(guard.stream_high_watermarks.len(), 2);
    }

    #[test]
    fn capacity_is_bounded_by_distinct_audience_streams() {
        let mut guard = VaultTransportReplayGuard::new(1);

        for index in 0..MAX_TRANSPORT_REPLAY_STREAMS {
            let request_id =
                vault_transport_request_id_v1(u64::try_from(index + 1).expect("stream"), 1)
                    .expect("position");
            guard
                .consume_request_id(&audience(index), &request_id)
                .expect("audience stream within capacity");
        }
        let overflow_request_id = vault_transport_request_id_v1(1, 1).expect("overflow position");
        assert!(matches!(
            guard.consume_request_id(
                &audience(MAX_TRANSPORT_REPLAY_STREAMS),
                &overflow_request_id
            ),
            Err(VaultTransportError::SessionCapacityExceeded)
        ));
    }

    #[test]
    fn zero_stream_and_sequence_request_id_is_rejected() {
        let mut guard = VaultTransportReplayGuard::new(1);

        assert!(matches!(
            guard.consume_request_id(&audience(1), &[0_u8; 16]),
            Err(VaultTransportError::InvalidBinding)
        ));
    }
}

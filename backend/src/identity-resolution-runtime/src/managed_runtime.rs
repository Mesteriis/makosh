use crate::{
    IdentityResolutionExecutionContextV1, IdentityResolutionExecutionErrorV1,
    consume_persons_identity_evidence_once_v1,
};
use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity,
    RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamKindV1,
    request_managed_runtime_event_access_v2,
};
use makosh_identity_resolution_api::{
    IDENTITY_RESOLUTION_OWNER_ID_V1,
    identity_resolution_person_match_candidate_contract_reference_v1,
};
use makosh_identity_resolution_persistence::{
    IdentityResolutionPersistenceErrorV1, IdentityResolutionPersistenceV1,
};
use makosh_persons_api::persons_review_candidate_contract_reference_v1;
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlTransportErrorV2},
    v1::{
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1,
    },
};
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use std::os::unix::net::UnixStream;

const MAX_RELAY: usize = 4;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityResolutionRuntimeAdmissionV1 {
    pub module_owner_id: String,
    pub logical_human_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityResolutionManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(IdentityResolutionPersistenceErrorV1),
    ControlClosed,
    Unavailable,
}
pub struct IdentityResolutionManagedRuntimeV1 {
    admission: IdentityResolutionRuntimeAdmissionV1,
    control: ManagedControlChannelV2<UnixStream>,
    persistence: IdentityResolutionPersistenceV1,
    events: RuntimeJetStreamConnection,
    publish_permit: RuntimePublishPermitV1,
    subscription: RuntimeSubscribePermitV1,
}

impl IdentityResolutionManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control: UnixStream,
        descriptor: Vec<u8>,
        settings: Vec<u8>,
        admission: &IdentityResolutionRuntimeAdmissionV1,
        storage: ManagedStorageRuntimeConfigurationV1,
        event_endpoint: &str,
        event_revision: u64,
    ) -> Result<Self, IdentityResolutionManagedRuntimeErrorV1> {
        validate_admission(admission)?;
        if event_endpoint.is_empty() || event_revision == 0 {
            return Err(IdentityResolutionManagedRuntimeErrorV1::Admission);
        }
        let mut control = ManagedControlChannelV2::new(control);
        authenticate(&mut control, descriptor, settings, admission)?;
        let binding = storage_binding(&storage, admission)?;
        let public_key = storage
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage.vault_instance_id.clone(),
            storage.vault_runtime_generation,
            public_key,
        )
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control),
            vault_context,
        );
        let password = resolve(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
        let persistence = IdentityResolutionPersistenceV1::connect_runtime(
            &binding,
            &storage.database_id,
            &storage.pgbouncer_host,
            storage.pgbouncer_port,
            password,
        )
        .await
        .map_err(IdentityResolutionManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(IdentityResolutionManagedRuntimeErrorV1::Persistence)?;
        let mut control = leases.into_route_port().into_channel();
        let access = request_managed_runtime_event_access_v2(
            &mut control,
            &storage.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_revision,
        )
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::EventUnavailable)?;
        let identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
        let publish_permit = access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
        let expected_subject = DurableSubjectV1::new(
            StreamKindV1::Event,
            IDENTITY_RESOLUTION_OWNER_ID_V1.to_owned(),
            identity_resolution_person_match_candidate_contract_reference_v1().name,
            1,
        )
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
        if !publish_permit.permits_exact_subjects(&[expected_subject]) {
            return Err(IdentityResolutionManagedRuntimeErrorV1::Admission);
        }
        let mut subscriptions = access
            .subscribe_permits(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
        if subscriptions.len() != 1
            || subscriptions[0].contract()
                != Some(&persons_review_candidate_contract_reference_v1())
        {
            return Err(IdentityResolutionManagedRuntimeErrorV1::Admission);
        }
        let subscription = subscriptions.remove(0);
        let events = JetStreamClient::connect_runtime_with_jwt(
            event_endpoint,
            identity,
            access.into_credential(),
        )
        .await
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::EventUnavailable)?;
        events
            .open_pull_consumer(&subscription)
            .await
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::EventContract)?;
        signal_ready(&mut control, admission)?;
        control
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            control,
            persistence,
            events,
            publish_permit,
            subscription,
        })
    }
    pub async fn service_once(
        &mut self,
        now: i64,
    ) -> Result<bool, IdentityResolutionManagedRuntimeErrorV1> {
        if pump_control(&mut self.control)? {
            return Ok(true);
        }
        let context = IdentityResolutionExecutionContextV1 {
            logical_owner_id: self.admission.logical_human_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis: now,
        };
        let mut progressed = consume_persons_identity_evidence_once_v1(
            &self.persistence,
            &self.events,
            &self.subscription,
            &context,
        )
        .await
        .map_err(execution_error)?;
        for _ in 0..MAX_RELAY {
            let Some(claim) = self
                .persistence
                .claim_next_pending_outbox(&self.admission.logical_human_owner_id)
                .await
                .map_err(IdentityResolutionManagedRuntimeErrorV1::Persistence)?
            else {
                break;
            };
            let sha = claim.record().record.envelope_sha256;
            self.events
                .publish_exact(&self.publish_permit, &claim.record().record.envelope_bytes)
                .await
                .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::EventUnavailable)?;
            claim
                .mark_published(sha, now)
                .await
                .map_err(IdentityResolutionManagedRuntimeErrorV1::Persistence)?;
            progressed = true
        }
        if !progressed {
            tokio::time::sleep(std::time::Duration::from_millis(25)).await
        }
        Ok(progressed)
    }
    pub async fn wait_retry_delay(
        &mut self,
        delay: std::time::Duration,
    ) -> Result<bool, IdentityResolutionManagedRuntimeErrorV1> {
        let deadline = tokio::time::Instant::now() + delay;
        loop {
            match pump_control(&mut self.control) {
                Ok(true) => {}
                Ok(false) => {}
                Err(IdentityResolutionManagedRuntimeErrorV1::ControlClosed) => return Ok(false),
                Err(e) => return Err(e),
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(true);
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await
        }
    }
}
fn validate_admission(
    v: &IdentityResolutionRuntimeAdmissionV1,
) -> Result<(), IdentityResolutionManagedRuntimeErrorV1> {
    if v.module_owner_id != IDENTITY_RESOLUTION_OWNER_ID_V1
        || v.logical_human_owner_id.is_empty()
        || v.registration_id.is_empty()
        || v.runtime_instance_id.is_empty()
        || v.runtime_generation == 0
        || v.grant_epoch == 0
    {
        Err(IdentityResolutionManagedRuntimeErrorV1::Admission)
    } else {
        Ok(())
    }
}
fn authenticate(
    c: &mut ManagedControlChannelV2<UnixStream>,
    d: Vec<u8>,
    s: Vec<u8>,
    a: &IdentityResolutionRuntimeAdmissionV1,
) -> Result<(), IdentityResolutionManagedRuntimeErrorV1> {
    c.inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            c.inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Unavailable)?;
    let r = c
        .describe_managed_runtime(d, s)
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Unavailable)?;
    if r.registration_id != a.registration_id
        || r.runtime_generation != a.runtime_generation
        || r.grant_epoch != a.grant_epoch
    {
        return Err(IdentityResolutionManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}
fn signal_ready(
    c: &mut ManagedControlChannelV2<UnixStream>,
    a: &IdentityResolutionRuntimeAdmissionV1,
) -> Result<(), IdentityResolutionManagedRuntimeErrorV1> {
    c.signal_ready(ManagedRuntimeReadyRequestV1 {
        registration_id: a.registration_id.clone(),
        runtime_generation: a.runtime_generation,
        grant_epoch: a.grant_epoch,
    })
    .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Unavailable)?;
    c.inner_mut()
        .set_read_timeout(None)
        .and_then(|_| c.inner_mut().set_write_timeout(None))
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Unavailable)
}
fn pump_control(
    c: &mut ManagedControlChannelV2<UnixStream>,
) -> Result<bool, IdentityResolutionManagedRuntimeErrorV1> {
    match c.try_receive_request() {
        Ok(None) => Ok(false),
        Ok(Some((id, _))) => {
            c.write_response(
                id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_unexpected_request".to_owned(),
                },
            )
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Unavailable)?;
            Ok(true)
        }
        Err(ManagedControlTransportErrorV2::PeerClosed) => {
            Err(IdentityResolutionManagedRuntimeErrorV1::ControlClosed)
        }
        Err(_) => Err(IdentityResolutionManagedRuntimeErrorV1::Unavailable),
    }
}
async fn resolve(
    a: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    b: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, IdentityResolutionManagedRuntimeErrorV1> {
    for i in 0..20 {
        if let Ok(v) = a.ensure_runtime_credential(b).await {
            return Ok(v);
        }
        if i < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await
        }
    }
    Err(IdentityResolutionManagedRuntimeErrorV1::Unavailable)
}
fn storage_binding(
    c: &ManagedStorageRuntimeConfigurationV1,
    a: &IdentityResolutionRuntimeAdmissionV1,
) -> Result<StorageBindingV1, IdentityResolutionManagedRuntimeErrorV1> {
    if c.runtime_instance_id != a.runtime_instance_id
        || c.logical_owner_id != IDENTITY_RESOLUTION_OWNER_ID_V1
        || c.owner != IDENTITY_RESOLUTION_OWNER_ID_V1
        || c.storage_bundle_digest.len() != 32
        || c.storage_generation == 0
        || c.credential_revision == 0
        || c.role_epoch == 0
        || c.storage_bundle_revision == 0
    {
        return Err(IdentityResolutionManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        c.storage_instance_id.clone(),
        c.database_id.clone(),
        c.owner.clone(),
        a.registration_id.clone(),
        c.runtime_instance_id.clone(),
    )
    .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        c.storage_generation,
        a.runtime_generation,
        a.grant_epoch,
        c.role_epoch,
        c.credential_revision,
        c.storage_bundle_revision,
    )
    .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(c.max_connections)
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?,
        c.statement_timeout_millis,
    )
    .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        c.runtime_principal.clone(),
        c.pool_alias.clone(),
        budgets,
        c.storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| IdentityResolutionManagedRuntimeErrorV1::Admission)
}
const fn execution_error(
    e: IdentityResolutionExecutionErrorV1,
) -> IdentityResolutionManagedRuntimeErrorV1 {
    match e {
        IdentityResolutionExecutionErrorV1::Persistence(v) => {
            IdentityResolutionManagedRuntimeErrorV1::Persistence(v)
        }
        IdentityResolutionExecutionErrorV1::EventUnavailable => {
            IdentityResolutionManagedRuntimeErrorV1::EventUnavailable
        }
        IdentityResolutionExecutionErrorV1::InvalidEnvelope
        | IdentityResolutionExecutionErrorV1::InvalidPayload
        | IdentityResolutionExecutionErrorV1::InvalidContext => {
            IdentityResolutionManagedRuntimeErrorV1::EventContract
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contour_is_one_input_one_output_and_bounded() {
        assert_eq!(MAX_RELAY, 4);
        assert_eq!(
            persons_review_candidate_contract_reference_v1().owner,
            "persons"
        );
    }
}

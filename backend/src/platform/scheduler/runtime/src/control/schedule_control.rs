//! Module-originated schedule-control adaptation and durable worker loop.

use std::sync::mpsc::Sender;

use makosh_clock_protocol::{ClockDiscontinuityV1, ClockPolicyV1, UtcMillisV1};
use makosh_events_protocol::delivery::OutboxRelayErrorV1;
use makosh_events_protocol::delivery::{OutboxRecordV1, OutboxRelayOutcomeV1, relay_once};
use makosh_runtime_protocol::v1::{
    SchedulerRuntimeScheduleControlBindingV1, SchedulerRuntimeScheduleControlGrantV1,
};
use makosh_scheduler::{
    SchedulerApprovedJobV1, SchedulerDispatchIdentityV1, SchedulerScheduleControlAdmissionErrorV1,
    SchedulerScheduleControlContractV1, SchedulerScheduleControlGrantV1,
    SchedulerScheduleControlOperationV1, admit_schedule_control_command_v1,
    build_schedule_control_result_envelope_v1,
};
use makosh_scheduler_jetstream::SchedulerJetStreamScheduleControlPortV1;
use makosh_scheduler_persistence::{
    SchedulerPostgresStoreV1, SchedulerScheduleControlApplyErrorV1,
    SchedulerScheduleControlAuthorityV1, SchedulerScheduleControlDecisionV1,
    SchedulerScheduleControlMutationV1, SchedulerScheduleControlRejectionV1,
    SchedulerScheduleControlRequestV1, SchedulerScheduleControlResultOutboxV1,
    SchedulerScheduleUpsertV1,
};
use makosh_scheduler_protocol::{
    JobContractBindingV1, JobKindV1, SchedulerScheduleControlDeliveryPortV1,
    SchedulerScheduleControlDeliveryV1,
    v1::{SchedulerScheduleControlOutcomeV1, SchedulerScheduleControlResultV1},
};
use prost::Message;

use super::clock::SchedulerSystemClockV1;
use super::transient_retry::SchedulerTransientRetryV1;
use super::workers::{report_failure, wait_for_transient_retry};

pub(super) struct SchedulerScheduleControlWorkerConfigV1 {
    command_contract: SchedulerScheduleControlContractV1,
    result_contract: SchedulerScheduleControlContractV1,
    grants: Vec<SchedulerScheduleControlGrantV1>,
    source: SchedulerDispatchIdentityV1,
}

impl SchedulerScheduleControlWorkerConfigV1 {
    pub(super) fn from_runtime(
        runtime_id: &str,
        runtime_instance_id: [u8; 16],
        runtime_generation: u64,
        binding: Option<&SchedulerRuntimeScheduleControlBindingV1>,
        grants: &[SchedulerRuntimeScheduleControlGrantV1],
    ) -> Result<Option<Self>, String> {
        let Some(binding) = binding else {
            return grants
                .is_empty()
                .then_some(None)
                .ok_or_else(|| "Scheduler schedule-control grants are unbound".to_owned());
        };
        let command_contract = SchedulerScheduleControlContractV1::new(
            binding.command_contract_revision,
            fixed(&binding.command_schema_sha256)?,
        )
        .map_err(|_| "Scheduler schedule-control command contract is invalid".to_owned())?;
        let result_contract = SchedulerScheduleControlContractV1::new(
            binding.result_contract_revision,
            fixed(&binding.result_schema_sha256)?,
        )
        .map_err(|_| "Scheduler schedule-control result contract is invalid".to_owned())?;
        let grants = grants
            .iter()
            .map(grant_from_runtime)
            .collect::<Result<Vec<_>, _>>()?;
        let source = SchedulerDispatchIdentityV1::new(
            runtime_id.to_owned(),
            runtime_instance_id,
            runtime_generation,
        )
        .map_err(|_| "Scheduler schedule-control source is invalid".to_owned())?;
        Ok(Some(Self {
            command_contract,
            result_contract,
            grants,
            source,
        }))
    }
}

pub(super) async fn run_schedule_control_worker(
    mut port: SchedulerJetStreamScheduleControlPortV1,
    store: SchedulerPostgresStoreV1,
    configuration: SchedulerScheduleControlWorkerConfigV1,
    failure: Sender<()>,
) {
    let mut retry = SchedulerTransientRetryV1::default();
    loop {
        match run_once(&mut port, &store, &configuration).await {
            Ok(()) => retry.reset(),
            Err(error) if error.is_transient() => {
                if !wait_for_transient_retry(&mut retry, &failure, error.code()).await {
                    return;
                }
            }
            Err(error) => {
                report_failure(&failure, error.code());
                return;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScheduleControlWorkerFailureV1 {
    RelayResultInvalidEntry,
    RelayResultInvalidReceipt,
    RelayResultPersistence,
    RelayResultPublisherUnavailable,
    Receive,
    Clock,
    Request,
    Apply,
    Acknowledge,
}

impl ScheduleControlWorkerFailureV1 {
    const fn code(self) -> &'static str {
        match self {
            Self::RelayResultInvalidEntry => "schedule_control_relay_result_invalid_entry",
            Self::RelayResultInvalidReceipt => "schedule_control_relay_result_invalid_receipt",
            Self::RelayResultPersistence => "schedule_control_relay_result_persistence",
            Self::RelayResultPublisherUnavailable => {
                "schedule_control_relay_result_publisher_unavailable"
            }
            Self::Receive => "schedule_control_receive",
            Self::Clock => "schedule_control_clock",
            Self::Request => "schedule_control_request",
            Self::Apply => "schedule_control_apply",
            Self::Acknowledge => "schedule_control_acknowledge",
        }
    }

    const fn is_transient(self) -> bool {
        matches!(
            self,
            Self::RelayResultPublisherUnavailable | Self::Receive | Self::Acknowledge
        )
    }
}

async fn run_once(
    port: &mut SchedulerJetStreamScheduleControlPortV1,
    store: &SchedulerPostgresStoreV1,
    configuration: &SchedulerScheduleControlWorkerConfigV1,
) -> Result<(), ScheduleControlWorkerFailureV1> {
    let clock = SchedulerSystemClockV1::new(ClockPolicyV1::production_default());
    relay_results(port, store).await?;
    let delivery = port
        .receive()
        .await
        .map_err(|_| ScheduleControlWorkerFailureV1::Receive)?;
    let Some((delivery, admitted)) = admit_or_discard(
        delivery,
        &configuration.command_contract,
        &configuration.grants,
    )
    .await?
    else {
        return Ok(());
    };
    let reading = clock
        .read()
        .map_err(|_| ScheduleControlWorkerFailureV1::Clock)?;
    if reading.discontinuity() != ClockDiscontinuityV1::Stable {
        return Err(ScheduleControlWorkerFailureV1::Clock);
    }
    let received_at = reading.wall_utc();
    let request = request_from_admitted(admitted, received_at)
        .map_err(|_| ScheduleControlWorkerFailureV1::Request)?;
    let result_message_id = result_message_id(request.command());
    store
        .apply_schedule_control(&request, |decision| {
            let payload = result_payload(&request, decision);
            let envelope = build_schedule_control_result_envelope_v1(
                request.command(),
                payload,
                result_message_id,
                received_at,
                &configuration.source,
                &configuration.result_contract,
            )
            .map_err(|_| SchedulerScheduleControlApplyErrorV1::InvalidResult)?;
            OutboxRecordV1::accept(envelope.encode_to_vec())
                .map_err(|_| SchedulerScheduleControlApplyErrorV1::InvalidResult)
        })
        .await
        .map_err(|_| ScheduleControlWorkerFailureV1::Apply)?;
    relay_results(port, store).await?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| ScheduleControlWorkerFailureV1::Acknowledge)
}

async fn admit_or_discard<D>(
    delivery: D,
    command_contract: &SchedulerScheduleControlContractV1,
    grants: &[SchedulerScheduleControlGrantV1],
) -> Result<
    Option<(D, makosh_scheduler::SchedulerAdmittedScheduleControlV1)>,
    ScheduleControlWorkerFailureV1,
>
where
    D: SchedulerScheduleControlDeliveryV1,
{
    match admit_schedule_control_command_v1(delivery.exact_bytes(), command_contract, grants) {
        Ok(admitted) => Ok(Some((delivery, admitted))),
        Err(error) => {
            // Invalid, foreign and stale-fenced broker input is untrusted data,
            // not a Scheduler process failure. It receives no result authority,
            // but must be acknowledged so one poison delivery cannot starve the
            // exact durable consumer forever.
            if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                eprintln!(
                    "developer_scheduler_schedule_control_rejected={}",
                    admission_error_code(error)
                );
            }
            delivery
                .acknowledge()
                .await
                .map_err(|_| ScheduleControlWorkerFailureV1::Acknowledge)?;
            Ok(None)
        }
    }
}

const fn admission_error_code(error: SchedulerScheduleControlAdmissionErrorV1) -> &'static str {
    match error {
        SchedulerScheduleControlAdmissionErrorV1::InvalidEnvelope => "invalid_envelope",
        SchedulerScheduleControlAdmissionErrorV1::InvalidContract => "invalid_contract",
        SchedulerScheduleControlAdmissionErrorV1::InvalidGrant => "invalid_grant",
        SchedulerScheduleControlAdmissionErrorV1::InvalidCommand => "invalid_command",
        SchedulerScheduleControlAdmissionErrorV1::StaleFence => "stale_fence",
        SchedulerScheduleControlAdmissionErrorV1::ForeignJobKind => "foreign_job_kind",
    }
}

async fn relay_results(
    port: &SchedulerJetStreamScheduleControlPortV1,
    store: &SchedulerPostgresStoreV1,
) -> Result<(), ScheduleControlWorkerFailureV1> {
    let mut outbox = SchedulerScheduleControlResultOutboxV1::new(store);
    loop {
        match relay_once(&mut outbox, port)
            .await
            .map_err(map_relay_error)?
        {
            OutboxRelayOutcomeV1::Published { .. } => {}
            OutboxRelayOutcomeV1::Idle => return Ok(()),
        }
    }
}

const fn map_relay_error(error: OutboxRelayErrorV1) -> ScheduleControlWorkerFailureV1 {
    match error {
        OutboxRelayErrorV1::InvalidEntry => ScheduleControlWorkerFailureV1::RelayResultInvalidEntry,
        OutboxRelayErrorV1::InvalidReceipt => {
            ScheduleControlWorkerFailureV1::RelayResultInvalidReceipt
        }
        OutboxRelayErrorV1::Persistence => ScheduleControlWorkerFailureV1::RelayResultPersistence,
        OutboxRelayErrorV1::PublisherUnavailable => {
            ScheduleControlWorkerFailureV1::RelayResultPublisherUnavailable
        }
    }
}

fn request_from_admitted(
    admitted: makosh_scheduler::SchedulerAdmittedScheduleControlV1,
    received_at: UtcMillisV1,
) -> Result<SchedulerScheduleControlRequestV1, SchedulerScheduleControlApplyErrorV1> {
    let binding = admitted.grant().approved_binding();
    let authority = SchedulerScheduleControlAuthorityV1::new(
        admitted.grant().source_module_id().to_owned(),
        admitted.grant().source_owner().to_owned(),
        binding.job_kind().owner().to_owned(),
        binding.job_kind().name().to_owned(),
        binding.job_kind().major(),
    )?;
    let mutation = match admitted.operation() {
        SchedulerScheduleControlOperationV1::Ensure(schedule) => {
            SchedulerScheduleControlMutationV1::Ensure(Box::new(SchedulerScheduleUpsertV1::new(
                schedule.spec().clone(),
                schedule.next_due_at(),
                received_at,
            )))
        }
        SchedulerScheduleControlOperationV1::Cancel {
            schedule_id,
            expected_revision,
        } => SchedulerScheduleControlMutationV1::Cancel {
            schedule_id: *schedule_id,
            expected_revision: *expected_revision,
            cancelled_at: received_at,
        },
    };
    SchedulerScheduleControlRequestV1::new(
        admitted.command().clone(),
        *admitted.operation_id(),
        authority,
        mutation,
        received_at,
    )
}

fn result_payload(
    request: &SchedulerScheduleControlRequestV1,
    decision: SchedulerScheduleControlDecisionV1,
) -> SchedulerScheduleControlResultV1 {
    let (outcome, error_code) = match decision {
        SchedulerScheduleControlDecisionV1::Ensured => {
            (SchedulerScheduleControlOutcomeV1::Ensured, "")
        }
        SchedulerScheduleControlDecisionV1::Cancelled => {
            (SchedulerScheduleControlOutcomeV1::Cancelled, "")
        }
        SchedulerScheduleControlDecisionV1::TooLate => {
            (SchedulerScheduleControlOutcomeV1::TooLate, "")
        }
        SchedulerScheduleControlDecisionV1::Rejected(rejection) => (
            SchedulerScheduleControlOutcomeV1::Rejected,
            rejection_code(rejection),
        ),
    };
    SchedulerScheduleControlResultV1 {
        operation_id: request.operation_id().to_vec(),
        schedule_id: request.mutation().schedule_id().bytes().to_vec(),
        schedule_revision: request.mutation().schedule_revision().value(),
        outcome: outcome.into(),
        error_code: error_code.to_owned(),
    }
}

const fn rejection_code(rejection: SchedulerScheduleControlRejectionV1) -> &'static str {
    match rejection {
        SchedulerScheduleControlRejectionV1::ForeignAuthority => "foreign_authority",
        SchedulerScheduleControlRejectionV1::UnknownSchedule => "unknown_schedule",
        SchedulerScheduleControlRejectionV1::StaleRevision => "stale_revision",
        SchedulerScheduleControlRejectionV1::RevisionConflict => "revision_conflict",
        SchedulerScheduleControlRejectionV1::ConcurrencyBusy => "concurrency_busy",
    }
}

fn result_message_id(command: &OutboxRecordV1) -> [u8; 16] {
    const DOMAIN: &[u8; 16] = b"sched-result-v1!";
    let hash = command.envelope_sha256();
    let mut result = [0_u8; 16];
    for index in 0..16 {
        result[index] = hash[index] ^ hash[index + 16] ^ DOMAIN[index];
    }
    if result.iter().all(|byte| *byte == 0) {
        result[15] = 1;
    }
    result
}

fn grant_from_runtime(
    grant: &SchedulerRuntimeScheduleControlGrantV1,
) -> Result<SchedulerScheduleControlGrantV1, String> {
    let job_kind = JobKindV1::new(
        grant.job_owner.clone(),
        grant.job_name.clone(),
        u16::try_from(grant.job_major)
            .map_err(|_| "Scheduler schedule-control grant is invalid".to_owned())?,
    )
    .map_err(|_| "Scheduler schedule-control grant is invalid".to_owned())?;
    let binding = JobContractBindingV1::new(
        job_kind,
        grant.contract_name.clone(),
        grant.contract_revision,
        fixed(&grant.contract_schema_sha256)?,
    )
    .map_err(|_| "Scheduler schedule-control grant is invalid".to_owned())?;
    let approved = SchedulerApprovedJobV1::new(grant.source_owner.clone(), binding)
        .map_err(|_| "Scheduler schedule-control grant is invalid".to_owned())?;
    SchedulerScheduleControlGrantV1::new(
        grant.source_module_id.clone(),
        fixed(&grant.source_runtime_instance_id)?,
        grant.source_runtime_generation,
        grant.source_grant_epoch,
        approved,
    )
    .map_err(|_| "Scheduler schedule-control grant is invalid".to_owned())
}

fn fixed<const N: usize>(value: &[u8]) -> Result<[u8; N], String> {
    value
        .try_into()
        .map_err(|_| "Scheduler schedule-control fixed field is invalid".to_owned())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    struct RejectedDeliveryV1 {
        acknowledged: Arc<AtomicBool>,
    }

    impl SchedulerScheduleControlDeliveryV1 for RejectedDeliveryV1 {
        fn exact_bytes(&self) -> &[u8] {
            b"not-a-durable-envelope"
        }

        async fn acknowledge(
            self,
        ) -> Result<(), makosh_scheduler_protocol::SchedulerScheduleControlDeliveryErrorV1>
        {
            self.acknowledged.store(true, Ordering::Release);
            Ok(())
        }
    }

    #[test]
    fn runtime_configuration_maps_exact_contract_and_grant_fences() {
        let binding = SchedulerRuntimeScheduleControlBindingV1 {
            stream_name: "MAKOSH_COMMAND_V1".to_owned(),
            durable_name: "scheduler-schedule-control-v1".to_owned(),
            filter_subject: "makosh.command.v1.scheduler.schedule_control.v1".to_owned(),
            ack_wait_millis: 30_000,
            max_deliver: 5,
            max_ack_pending: 1,
            result_subject: "makosh.result.v1.scheduler.schedule_control.v1".to_owned(),
            command_contract_revision: 2,
            command_schema_sha256: vec![7; 32],
            result_contract_revision: 3,
            result_schema_sha256: vec![8; 32],
        };
        let grant = SchedulerRuntimeScheduleControlGrantV1 {
            source_module_id: "communication_delayed_delivery.runtime.v1".to_owned(),
            source_runtime_instance_id: vec![4; 16],
            source_runtime_generation: 5,
            source_grant_epoch: 6,
            source_owner: "communication_delayed_delivery".to_owned(),
            job_owner: "communication_delayed_delivery".to_owned(),
            job_name: "execute".to_owned(),
            job_major: 1,
            contract_name: "communication_delayed_delivery.execute".to_owned(),
            contract_revision: 1,
            contract_schema_sha256: vec![9; 32],
        };

        let configuration = SchedulerScheduleControlWorkerConfigV1::from_runtime(
            "scheduler",
            [3; 16],
            4,
            Some(&binding),
            &[grant],
        )
        .expect("configuration")
        .expect("enabled");

        assert_eq!(configuration.command_contract.revision(), 2);
        assert_eq!(configuration.result_contract.revision(), 3);
        assert_eq!(configuration.grants.len(), 1);
        assert_eq!(
            configuration.grants[0].source_module_id(),
            "communication_delayed_delivery.runtime.v1"
        );
        assert_eq!(
            configuration.grants[0].source_owner(),
            "communication_delayed_delivery"
        );
    }

    #[test]
    fn runtime_configuration_rejects_grants_without_transport_binding() {
        assert!(
            SchedulerScheduleControlWorkerConfigV1::from_runtime(
                "scheduler",
                [3; 16],
                4,
                None,
                &[SchedulerRuntimeScheduleControlGrantV1::default()],
            )
            .is_err()
        );
    }

    #[test]
    fn only_transport_failures_enter_transient_retry() {
        for failure in [
            ScheduleControlWorkerFailureV1::RelayResultPublisherUnavailable,
            ScheduleControlWorkerFailureV1::Receive,
            ScheduleControlWorkerFailureV1::Acknowledge,
        ] {
            assert!(failure.is_transient(), "{failure:?}");
        }
        for failure in [
            ScheduleControlWorkerFailureV1::RelayResultInvalidEntry,
            ScheduleControlWorkerFailureV1::RelayResultInvalidReceipt,
            ScheduleControlWorkerFailureV1::RelayResultPersistence,
            ScheduleControlWorkerFailureV1::Clock,
            ScheduleControlWorkerFailureV1::Request,
            ScheduleControlWorkerFailureV1::Apply,
        ] {
            assert!(!failure.is_transient(), "{failure:?}");
        }
    }

    #[test]
    fn rejected_delivery_is_acknowledged_without_failing_the_worker() {
        let acknowledged = Arc::new(AtomicBool::new(false));
        let delivery = RejectedDeliveryV1 {
            acknowledged: Arc::clone(&acknowledged),
        };
        let contract =
            SchedulerScheduleControlContractV1::new(1, [7; 32]).expect("schedule-control contract");
        let runtime = tokio::runtime::Runtime::new().expect("test runtime");

        let outcome = runtime
            .block_on(admit_or_discard(delivery, &contract, &[]))
            .expect("rejected delivery is handled");

        assert!(outcome.is_none());
        assert!(acknowledged.load(Ordering::Acquire));
    }
}

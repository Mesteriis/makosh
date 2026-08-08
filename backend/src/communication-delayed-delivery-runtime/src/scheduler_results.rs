use makosh_communication_delayed_delivery_event_adapters::{
    DecodedSchedulerScheduleResultV1, DelayedDeliverySchedulerResultContextV1,
    decode_scheduler_result_v1, scheduler_result_causation_id_v1,
};
use makosh_communication_delayed_delivery_persistence::{
    ApplySchedulerResultV1, CommunicationDelayedDeliveryPersistenceV1,
    DelayedDeliveryPersistenceErrorV1, SchedulerScheduleResultV1,
};
use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimePullDeliveryV1, RuntimeSubscribePermitV1,
    receive_runtime_pull_delivery,
};
use makosh_scheduler_protocol::SCHEDULER_JOB_DESCRIPTOR_SET_V1;
use sha2::{Digest, Sha256};

pub(crate) async fn consume_scheduler_result_v1(
    persistence: &CommunicationDelayedDeliveryPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    logical_owner_id: &str,
    received_at_unix_millis: u64,
) -> Result<bool, DelayedDeliverySchedulerResultErrorV1> {
    if received_at_unix_millis == 0 {
        return Err(DelayedDeliverySchedulerResultErrorV1::InvalidResult);
    }
    let delivery = receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| DelayedDeliverySchedulerResultErrorV1::EventUnavailable)?;
    let causation_id = match scheduler_result_causation_id_v1(delivery.exact_bytes()) {
        Ok(causation_id) => causation_id,
        Err(_) => return discard_invalid_scheduler_result(delivery, "invalid_envelope").await,
    };
    let decoded = match decode_scheduler_result_v1(
        delivery.exact_bytes(),
        &DelayedDeliverySchedulerResultContextV1 {
            expected_command_message_id: causation_id,
            contract_revision: 1,
            contract_schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).into(),
            received_at_unix_millis,
        },
    ) {
        Ok(decoded) => decoded,
        Err(_) => return discard_invalid_scheduler_result(delivery, "invalid_contract").await,
    };
    let owns_command = persistence
        .owns_scheduler_command(
            logical_owner_id,
            &decoded.delayed_operation_id,
            &causation_id,
        )
        .await
        .map_err(DelayedDeliverySchedulerResultErrorV1::Persistence)?;
    if !owns_command {
        return discard_invalid_scheduler_result(delivery, "foreign_command").await;
    }
    persistence
        .apply_scheduler_result(&ApplySchedulerResultV1 {
            logical_owner_id: logical_owner_id.to_owned(),
            delayed_operation_id: decoded.delayed_operation_id,
            message_id: decoded.message_id,
            envelope_sha256: decoded.envelope_sha256,
            result: persistence_result(decoded.result),
            received_at_unix_millis,
        })
        .await
        .map_err(DelayedDeliverySchedulerResultErrorV1::Persistence)?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| DelayedDeliverySchedulerResultErrorV1::EventUnavailable)?;
    Ok(true)
}

async fn discard_invalid_scheduler_result(
    delivery: RuntimePullDeliveryV1,
    reason: &str,
) -> Result<bool, DelayedDeliverySchedulerResultErrorV1> {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_delayed_delivery_scheduler_result_rejected={reason}");
    }
    delivery
        .acknowledge()
        .await
        .map_err(|_| DelayedDeliverySchedulerResultErrorV1::EventUnavailable)?;
    Ok(true)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DelayedDeliverySchedulerResultErrorV1 {
    InvalidResult,
    Persistence(DelayedDeliveryPersistenceErrorV1),
    EventUnavailable,
}

const fn persistence_result(result: DecodedSchedulerScheduleResultV1) -> SchedulerScheduleResultV1 {
    match result {
        DecodedSchedulerScheduleResultV1::Ensured { schedule_revision } => {
            SchedulerScheduleResultV1::Ensured { schedule_revision }
        }
        DecodedSchedulerScheduleResultV1::Cancelled => SchedulerScheduleResultV1::Cancelled,
        DecodedSchedulerScheduleResultV1::TooLate => SchedulerScheduleResultV1::TooLate,
        DecodedSchedulerScheduleResultV1::Rejected { error_code } => {
            SchedulerScheduleResultV1::Rejected { error_code }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scheduler_outcome_has_an_exact_persistence_mapping() {
        assert_eq!(
            persistence_result(DecodedSchedulerScheduleResultV1::Ensured {
                schedule_revision: 4,
            }),
            SchedulerScheduleResultV1::Ensured {
                schedule_revision: 4,
            }
        );
        assert_eq!(
            persistence_result(DecodedSchedulerScheduleResultV1::TooLate),
            SchedulerScheduleResultV1::TooLate
        );
    }
}

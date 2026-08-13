use makosh_events_jetstream::{
    RuntimeJetStreamConnection, RuntimeSubscribePermitV1, try_receive_runtime_pull_delivery,
};
use makosh_events_protocol::delivery::OutboxRecordV1;
use makosh_persons_persistence::PersonsPersistenceV1;

use crate::execution::{
    PersonsCommandExecutionErrorV1, PersonsCommandRuntimeContextV1,
    execute_persons_command_record_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsCommandConsumerErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence(makosh_persons_persistence::PersonsPersistenceErrorV1),
    EventUnavailable,
}

pub async fn consume_persons_command_once_v1(
    persistence: &PersonsPersistenceV1,
    connection: &RuntimeJetStreamConnection,
    permit: &RuntimeSubscribePermitV1,
    runtime: &PersonsCommandRuntimeContextV1,
) -> Result<bool, PersonsCommandConsumerErrorV1> {
    let Some(delivery) = try_receive_runtime_pull_delivery(connection, permit)
        .await
        .map_err(|_| PersonsCommandConsumerErrorV1::EventUnavailable)?
    else {
        return Ok(false);
    };
    let record = OutboxRecordV1::accept(delivery.exact_bytes().to_vec())
        .map_err(|_| PersonsCommandConsumerErrorV1::InvalidEnvelope)?;
    execute_persons_command_record_v1(persistence, &record, runtime)
        .await
        .map_err(execution_error)?;
    delivery
        .acknowledge()
        .await
        .map_err(|_| PersonsCommandConsumerErrorV1::EventUnavailable)?;
    Ok(true)
}

fn execution_error(error: PersonsCommandExecutionErrorV1) -> PersonsCommandConsumerErrorV1 {
    match error {
        PersonsCommandExecutionErrorV1::InvalidEnvelope => {
            PersonsCommandConsumerErrorV1::InvalidEnvelope
        }
        PersonsCommandExecutionErrorV1::InvalidPayload => {
            PersonsCommandConsumerErrorV1::InvalidPayload
        }
        PersonsCommandExecutionErrorV1::Persistence(error) => {
            PersonsCommandConsumerErrorV1::Persistence(error)
        }
    }
}

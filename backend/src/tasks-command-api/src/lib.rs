#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    TasksCommandEnvelopeBuildErrorV1, TasksCommandEnvelopeContextV1,
    build_create_task_from_reviewed_candidate_outbox_record_v1,
    build_task_created_from_reviewed_candidate_outbox_record_v1,
    build_task_creation_from_reviewed_candidate_rejected_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-tasks-command-api";
pub const TASKS_OWNER_ID_V1: &str = "tasks";
pub const TASKS_MODULE_ID_V1: &str = "makosh-tasks-runtime";
pub const TASKS_REVIEWED_CANDIDATE_COMMAND_CAPABILITY_ID_V1: &str =
    "tasks.reviewed-candidate.command.v1";
pub const TASKS_REVIEWED_CANDIDATE_BLOB_CAPABILITY_ID_V1: &str = "tasks.reviewed-candidate.blob.v1";
pub const CREATE_TASK_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1: &str =
    "tasks_create_from_reviewed_candidate";
pub const TASK_CREATED_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1: &str =
    "tasks_created_from_reviewed_candidate";
pub const TASK_CREATION_FROM_REVIEWED_CANDIDATE_REJECTED_CONTRACT_NAME_V1: &str =
    "tasks_creation_from_reviewed_candidate_rejected";
pub const TASKS_COMMAND_CONTRACT_MAJOR_V1: u32 = 1;
pub const TASKS_COMMAND_CONTRACT_REVISION_V1: u32 = 1;
pub const TASKS_REVIEWED_CANDIDATE_MAX_BLOB_BYTES_V1: u64 = 16 * 1024;
pub const TASKS_REVIEWED_CANDIDATE_MAX_PROOF_BYTES_V1: usize = 2_048;
pub const TASKS_REVIEWED_CANDIDATE_MAX_IN_FLIGHT_V1: u32 = 32;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.tasks.command.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/tasks_command_schema.rs"));

pub const TASKS_COMMAND_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/tasks-command-v1.bin"));

#[must_use]
pub fn create_task_from_reviewed_candidate_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(CREATE_TASK_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn task_created_from_reviewed_candidate_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference(TASK_CREATED_FROM_REVIEWED_CANDIDATE_CONTRACT_NAME_V1)
}

#[must_use]
pub fn task_creation_from_reviewed_candidate_rejected_contract_reference_v1() -> ContractReferenceV1
{
    contract_reference(TASK_CREATION_FROM_REVIEWED_CANDIDATE_REJECTED_CONTRACT_NAME_V1)
}

#[must_use]
pub fn create_task_from_reviewed_candidate_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        create_task_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn create_task_from_reviewed_candidate_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Command,
        create_task_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn task_created_from_reviewed_candidate_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        task_created_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn task_created_from_reviewed_candidate_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        task_created_from_reviewed_candidate_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

#[must_use]
pub fn task_creation_from_reviewed_candidate_rejected_publish_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        task_creation_from_reviewed_candidate_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Publish,
        EventSubscriptionRequirementV1::Unspecified,
    )
}

#[must_use]
pub fn task_creation_from_reviewed_candidate_rejected_consume_request_v1() -> CapabilityRequestV1 {
    event_route(
        DurableEnvelopeKindV1::Result,
        task_creation_from_reviewed_candidate_rejected_contract_reference_v1(),
        EventRouteDirectionV1::Consume,
        EventSubscriptionRequirementV1::Required,
    )
}

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: TASKS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: TASKS_COMMAND_CONTRACT_MAJOR_V1,
        revision: TASKS_COMMAND_CONTRACT_REVISION_V1,
        schema_sha256: TASKS_COMMAND_SCHEMA_SHA256_V1.to_vec(),
    }
}

fn event_route(
    envelope_kind: DurableEnvelopeKindV1,
    contract: ContractReferenceV1,
    direction: EventRouteDirectionV1,
    subscription_requirement: EventSubscriptionRequirementV1,
) -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: envelope_kind as i32,
            contract: Some(contract),
            direction: direction as i32,
            max_in_flight: TASKS_REVIEWED_CANDIDATE_MAX_IN_FLIGHT_V1,
            subscription_requirement: subscription_requirement as i32,
            max_deliver: u32::from(direction == EventRouteDirectionV1::Consume) * 10,
            ack_wait_millis: u32::from(direction == EventRouteDirectionV1::Consume) * 30_000,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_tasks_command_is_target_owned() {
        assert_eq!(TASKS_OWNER_ID_V1, "tasks");
        assert_eq!(TASKS_MODULE_ID_V1, "makosh-tasks-runtime");
        let Some(Request::EventRoute(route)) =
            create_task_from_reviewed_candidate_consume_request_v1().request
        else {
            panic!("command route");
        };
        assert_eq!(route.direction, EventRouteDirectionV1::Consume as i32);
        assert_eq!(
            route.subscription_requirement,
            EventSubscriptionRequirementV1::Required as i32
        );
    }

    #[test]
    fn durable_messages_exclude_candidate_presentation_text() {
        let source = include_str!("../proto/makosh/tasks/command/v1/tasks_command.proto");
        let command = source
            .split("message CreateTaskFromReviewedCandidateCommandV1")
            .nth(1)
            .and_then(|value| {
                value
                    .split("message TaskCreatedFromReviewedCandidateV1")
                    .next()
            })
            .expect("command section");
        assert!(!command.contains("string title"));
        assert!(!command.contains("due_text_hint"));
        assert!(!command.contains("assignee_label_hint"));
        assert!(!source.contains("provider_id"));
        assert!(!source.contains("project_id"));
        assert!(!source.contains("calendar"));
    }
}

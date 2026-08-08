//! Structural validation for the Scheduler managed-runtime boundary.

use std::collections::BTreeSet;

use crate::v1::{
    SchedulerRuntimeConfigurationV1, SchedulerRuntimeControlRequestV1,
    SchedulerRuntimeControlResponseV1, SchedulerRuntimeDispatchPublisherBindingV1,
    SchedulerRuntimeReceiptConsumerBindingV1, SchedulerRuntimeReceiptKindV1,
    SchedulerRuntimeScheduleControlBindingV1, SchedulerRuntimeScheduleControlGrantV1,
    SchedulerRuntimeStateV1, SchedulerRuntimeStatusV1, SchedulerScheduleUpsertOutcomeV1,
    UpsertSchedulerScheduleRequestV1, UpsertSchedulerScheduleResponseV1,
    scheduler_runtime_control_request_v1::Operation as RequestOperation,
    scheduler_runtime_control_response_v1::Result as ResponseResult,
};

const MAX_BATCH_LIMIT: u32 = 256;
const MIN_RECONCILE_INTERVAL_MILLIS: u32 = 100;
const MAX_RECONCILE_INTERVAL_MILLIS: u32 = 60_000;
const MAX_ACK_WAIT_MILLIS: u32 = 600_000;
const MAX_DELIVER: u32 = 32;
const MAX_ACK_PENDING: u32 = 4_096;
const MAX_RECEIPT_CONSUMERS: usize = 256;
const MAX_DISPATCH_PUBLISHERS: usize = 256;
const MAX_SCHEDULE_CONTROL_GRANTS: usize = 256;
const MAX_SCHEDULE_POLICY_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRuntimeValidationErrorV1 {
    InvalidConfiguration,
    InvalidRequest,
    InvalidResponse,
    InvalidStatus,
}

pub fn validate_scheduler_runtime_configuration(
    configuration: &SchedulerRuntimeConfigurationV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    let binding = configuration
        .storage_binding
        .as_ref()
        .ok_or(SchedulerRuntimeValidationErrorV1::InvalidConfiguration)?;
    (valid_id(&binding.database_id)
        && valid_id(&binding.storage_instance_id)
        && valid_owner(&binding.owner)
        && valid_host(&binding.pgbouncer_host)
        && valid_port(binding.pgbouncer_port)
        && valid_id(&binding.runtime_principal)
        && valid_pool_alias(&binding.pool_alias)
        && binding.storage_generation > 0
        && binding.credential_revision > 0
        && binding.role_epoch > 0
        && binding.storage_bundle_revision > 0
        && binding.storage_bundle_digest.len() == 32
        && binding.storage_bundle_digest.iter().any(|byte| *byte != 0)
        && valid_storage_budget(binding.max_connections, binding.statement_timeout_millis)
        && valid_id(&configuration.vault_instance_id)
        && configuration.vault_runtime_generation > 0
        && configuration.vault_hpke_public_key_x25519.len() == 32
        && valid_id(&configuration.runtime_instance_id)
        && valid_owner(&configuration.logical_owner_id)
        && binding.owner == configuration.logical_owner_id
        && valid_nats_endpoint(&configuration.nats_endpoint)
        && configuration.event_credential_revision > 0
        && valid_batch_limit(configuration.dispatch_batch_limit)
        && valid_batch_limit(configuration.receipt_batch_limit)
        && valid_reconcile_interval(configuration.reconcile_interval_millis)
        && valid_dispatch_publishers(&configuration.dispatch_publishers)
        && valid_receipt_consumers(&configuration.receipt_consumers)
        && valid_schedule_control(
            configuration.schedule_control.as_ref(),
            &configuration.schedule_control_grants,
        ))
    .then_some(())
    .ok_or(SchedulerRuntimeValidationErrorV1::InvalidConfiguration)
}

pub fn validate_scheduler_runtime_schedule_control_binding(
    binding: &SchedulerRuntimeScheduleControlBindingV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    valid_schedule_control_binding(binding)
        .then_some(())
        .ok_or(SchedulerRuntimeValidationErrorV1::InvalidConfiguration)
}

pub fn validate_scheduler_runtime_schedule_control_grant(
    grant: &SchedulerRuntimeScheduleControlGrantV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    valid_schedule_control_grant(grant)
        .then_some(())
        .ok_or(SchedulerRuntimeValidationErrorV1::InvalidConfiguration)
}

pub fn validate_scheduler_runtime_dispatch_publisher_binding(
    binding: &SchedulerRuntimeDispatchPublisherBindingV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    valid_dispatch_publisher(binding)
        .then_some(())
        .ok_or(SchedulerRuntimeValidationErrorV1::InvalidConfiguration)
}

pub fn validate_scheduler_runtime_receipt_consumer_binding(
    binding: &SchedulerRuntimeReceiptConsumerBindingV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    valid_receipt_consumer(binding)
        .then_some(())
        .ok_or(SchedulerRuntimeValidationErrorV1::InvalidConfiguration)
}

pub fn validate_scheduler_runtime_control_request(
    request: &SchedulerRuntimeControlRequestV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    match &request.operation {
        Some(RequestOperation::GetStatus(_)) => Ok(()),
        Some(RequestOperation::UpsertSchedule(schedule)) => validate_schedule_upsert(schedule),
        None => Err(SchedulerRuntimeValidationErrorV1::InvalidRequest),
    }
}

pub fn validate_scheduler_runtime_control_response(
    response: &SchedulerRuntimeControlResponseV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    match (&response.result, response.error_code.is_empty()) {
        (Some(ResponseResult::Status(status)), true) => validate_scheduler_runtime_status(status),
        (Some(ResponseResult::UpsertSchedule(schedule)), true) => {
            validate_schedule_upsert_response(schedule)
        }
        (None, false) if valid_blocker_code(&response.error_code) => Ok(()),
        _ => Err(SchedulerRuntimeValidationErrorV1::InvalidResponse),
    }
}

fn validate_schedule_upsert(
    schedule: &UpsertSchedulerScheduleRequestV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    (schedule.schedule_id.len() == 16
        && schedule.schedule_id.iter().any(|byte| *byte != 0)
        && schedule.schedule_revision > 0
        && valid_job_token(&schedule.job_owner)
        && valid_job_token(&schedule.job_name)
        && u16::try_from(schedule.job_major).is_ok_and(|major| major > 0)
        && valid_contract_name(&schedule.contract_name)
        && schedule.contract_revision > 0
        && schedule.contract_schema_sha256.len() == 32
        && schedule
            .contract_schema_sha256
            .iter()
            .any(|byte| *byte != 0)
        && valid_scope(&schedule.scope_id)
        && valid_concurrency_key(&schedule.concurrency_key)
        && !schedule.policy_canonical_bytes.is_empty()
        && schedule.policy_canonical_bytes.len() <= MAX_SCHEDULE_POLICY_BYTES)
        .then_some(())
        .ok_or(SchedulerRuntimeValidationErrorV1::InvalidRequest)
}

fn validate_schedule_upsert_response(
    response: &UpsertSchedulerScheduleResponseV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    (response.schedule_revision > 0
        && matches!(
            SchedulerScheduleUpsertOutcomeV1::try_from(response.outcome),
            Ok(SchedulerScheduleUpsertOutcomeV1::Inserted)
                | Ok(SchedulerScheduleUpsertOutcomeV1::Updated)
                | Ok(SchedulerScheduleUpsertOutcomeV1::Unchanged)
        ))
    .then_some(())
    .ok_or(SchedulerRuntimeValidationErrorV1::InvalidResponse)
}

pub fn validate_scheduler_runtime_status(
    status: &SchedulerRuntimeStatusV1,
) -> Result<(), SchedulerRuntimeValidationErrorV1> {
    let state = SchedulerRuntimeStateV1::try_from(status.state)
        .map_err(|_| SchedulerRuntimeValidationErrorV1::InvalidStatus)?;
    if status.runtime_generation == 0
        || status.grant_epoch == 0
        || status.storage_generation == 0
        || status.vault_runtime_generation == 0
        || status.event_credential_revision == 0
    {
        return Err(SchedulerRuntimeValidationErrorV1::InvalidStatus);
    }
    match state {
        SchedulerRuntimeStateV1::Ready if status.blocker_code.is_empty() => Ok(()),
        SchedulerRuntimeStateV1::Blocked if valid_blocker_code(&status.blocker_code) => Ok(()),
        _ => Err(SchedulerRuntimeValidationErrorV1::InvalidStatus),
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value.is_ascii()
        && !value.contains(['/', '@', ':', '?', '#', ' '])
}

fn valid_port(value: u32) -> bool {
    u16::try_from(value).is_ok_and(|port| port > 0)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_job_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_contract_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_scope(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.is_ascii()
}

fn valid_concurrency_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':'))
}

fn valid_pool_alias(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_storage_budget(max_connections: u32, statement_timeout_millis: u32) -> bool {
    max_connections > 0 && u16::try_from(max_connections).is_ok() && statement_timeout_millis > 0
}

fn valid_nats_endpoint(value: &str) -> bool {
    value.starts_with("nats://") && value.len() <= 512 && !value.contains(['@', '?', '#', ' '])
}

fn valid_batch_limit(value: u32) -> bool {
    (1..=MAX_BATCH_LIMIT).contains(&value)
}

fn valid_reconcile_interval(value: u32) -> bool {
    (MIN_RECONCILE_INTERVAL_MILLIS..=MAX_RECONCILE_INTERVAL_MILLIS).contains(&value)
}

fn valid_dispatch_publishers(values: &[SchedulerRuntimeDispatchPublisherBindingV1]) -> bool {
    !values.is_empty()
        && values.len() <= MAX_DISPATCH_PUBLISHERS
        && values.iter().all(valid_dispatch_publisher)
        && values
            .iter()
            .map(|value| value.subject.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            == values.len()
}

fn valid_dispatch_publisher(value: &SchedulerRuntimeDispatchPublisherBindingV1) -> bool {
    value.subject.starts_with("makosh.command.v1.") && valid_exact_subject(&value.subject)
}

fn valid_schedule_control(
    binding: Option<&SchedulerRuntimeScheduleControlBindingV1>,
    grants: &[SchedulerRuntimeScheduleControlGrantV1],
) -> bool {
    match binding {
        None => grants.is_empty(),
        Some(binding) => {
            !grants.is_empty()
                && grants.len() <= MAX_SCHEDULE_CONTROL_GRANTS
                && valid_schedule_control_binding(binding)
                && grants.iter().all(valid_schedule_control_grant)
                && grants
                    .iter()
                    .map(|grant| {
                        (
                            grant.source_module_id.as_str(),
                            grant.source_runtime_instance_id.as_slice(),
                            grant.source_runtime_generation,
                            grant.source_grant_epoch,
                            grant.job_owner.as_str(),
                            grant.job_name.as_str(),
                            grant.job_major,
                        )
                    })
                    .collect::<BTreeSet<_>>()
                    .len()
                    == grants.len()
        }
    }
}

fn valid_schedule_control_binding(binding: &SchedulerRuntimeScheduleControlBindingV1) -> bool {
    binding.stream_name == "MAKOSH_COMMAND_V1"
        && binding.filter_subject == "makosh.command.v1.scheduler.schedule_control.v1"
        && binding.result_subject == "makosh.result.v1.scheduler.schedule_control.v1"
        && binding.command_contract_revision > 0
        && nonzero_sha256(&binding.command_schema_sha256)
        && binding.result_contract_revision > 0
        && nonzero_sha256(&binding.result_schema_sha256)
        && valid_durable_name(&binding.durable_name)
        && (1..=MAX_ACK_WAIT_MILLIS).contains(&binding.ack_wait_millis)
        && (1..=MAX_DELIVER).contains(&binding.max_deliver)
        && (1..=MAX_ACK_PENDING).contains(&binding.max_ack_pending)
}

fn valid_schedule_control_grant(grant: &SchedulerRuntimeScheduleControlGrantV1) -> bool {
    valid_id(&grant.source_module_id)
        && grant.source_runtime_instance_id.len() == 16
        && grant
            .source_runtime_instance_id
            .iter()
            .any(|byte| *byte != 0)
        && grant.source_runtime_generation > 0
        && grant.source_grant_epoch > 0
        && valid_owner(&grant.source_owner)
        && valid_job_token(&grant.job_owner)
        && grant.source_owner == grant.job_owner
        && valid_job_token(&grant.job_name)
        && u16::try_from(grant.job_major).is_ok_and(|major| major > 0)
        && valid_contract_name(&grant.contract_name)
        && grant.contract_revision > 0
        && nonzero_sha256(&grant.contract_schema_sha256)
}

fn nonzero_sha256(value: &[u8]) -> bool {
    value.len() == 32 && value.iter().any(|byte| *byte != 0)
}

fn valid_receipt_consumers(values: &[SchedulerRuntimeReceiptConsumerBindingV1]) -> bool {
    !values.is_empty()
        && values.len() <= MAX_RECEIPT_CONSUMERS
        && values.iter().all(valid_receipt_consumer)
        && values
            .iter()
            .any(|value| value.kind == SchedulerRuntimeReceiptKindV1::Acceptance as i32)
        && values
            .iter()
            .any(|value| value.kind == SchedulerRuntimeReceiptKindV1::Terminal as i32)
        && values
            .iter()
            .map(|value| (value.durable_name.as_str(), value.filter_subject.as_str()))
            .collect::<BTreeSet<_>>()
            .len()
            == values.len()
}

fn valid_receipt_consumer(value: &SchedulerRuntimeReceiptConsumerBindingV1) -> bool {
    let Ok(kind) = SchedulerRuntimeReceiptKindV1::try_from(value.kind) else {
        return false;
    };
    let expected = match kind {
        SchedulerRuntimeReceiptKindV1::Acceptance => ("MAKOSH_ACK_V1", "makosh.ack.v1."),
        SchedulerRuntimeReceiptKindV1::Terminal => ("MAKOSH_RESULT_V1", "makosh.result.v1."),
        SchedulerRuntimeReceiptKindV1::Unspecified => return false,
    };
    value.stream_name == expected.0
        && valid_durable_name(&value.durable_name)
        && value.filter_subject.starts_with(expected.1)
        && valid_exact_subject(&value.filter_subject)
        && (1..=MAX_ACK_WAIT_MILLIS).contains(&value.ack_wait_millis)
        && (1..=MAX_DELIVER).contains(&value.max_deliver)
        && (1..=MAX_ACK_PENDING).contains(&value.max_ack_pending)
}

fn valid_durable_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
}

fn valid_exact_subject(value: &str) -> bool {
    value.len() <= 512
        && !value.contains(['*', '>', ' ', '@', '/', '?', '#'])
        && value.split('.').all(|part| !part.is_empty())
}

fn valid_blocker_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

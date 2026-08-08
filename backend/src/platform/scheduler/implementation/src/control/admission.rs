use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ActorKindV1, FenceKindV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_scheduler_protocol::{
    JobContractBindingV1, ScheduleIdV1, ScheduleRevisionV1,
    v1::{SchedulerScheduleControlCommandV1, scheduler_schedule_control_command_v1::Operation},
    validate_scheduler_schedule_control_command_v1,
};
use prost::Message;

use super::{
    SchedulerApprovedJobV1, SchedulerOneShotScheduleV1, map_approved_one_shot_schedule_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerScheduleControlContractV1 {
    revision: u32,
    schema_sha256: [u8; 32],
}

impl SchedulerScheduleControlContractV1 {
    pub fn new(
        revision: u32,
        schema_sha256: [u8; 32],
    ) -> Result<Self, SchedulerScheduleControlAdmissionErrorV1> {
        (revision > 0 && schema_sha256.iter().any(|byte| *byte != 0))
            .then_some(Self {
                revision,
                schema_sha256,
            })
            .ok_or(SchedulerScheduleControlAdmissionErrorV1::InvalidContract)
    }

    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }

    #[must_use]
    pub const fn schema_sha256(&self) -> &[u8; 32] {
        &self.schema_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerScheduleControlGrantV1 {
    source_module_id: String,
    source_runtime_instance_id: [u8; 16],
    source_runtime_generation: u64,
    source_grant_epoch: u64,
    approved_job: SchedulerApprovedJobV1,
}

impl SchedulerScheduleControlGrantV1 {
    pub fn new(
        source_module_id: String,
        source_runtime_instance_id: [u8; 16],
        source_runtime_generation: u64,
        source_grant_epoch: u64,
        approved_job: SchedulerApprovedJobV1,
    ) -> Result<Self, SchedulerScheduleControlAdmissionErrorV1> {
        (valid_module_id(&source_module_id)
            && source_runtime_instance_id.iter().any(|byte| *byte != 0)
            && source_runtime_generation > 0
            && source_grant_epoch > 0)
            .then_some(Self {
                source_module_id,
                source_runtime_instance_id,
                source_runtime_generation,
                source_grant_epoch,
                approved_job,
            })
            .ok_or(SchedulerScheduleControlAdmissionErrorV1::InvalidGrant)
    }

    #[must_use]
    pub fn source_module_id(&self) -> &str {
        &self.source_module_id
    }

    #[must_use]
    pub fn source_owner(&self) -> &str {
        self.approved_job.source_owner()
    }

    #[must_use]
    pub fn approved_binding(&self) -> &JobContractBindingV1 {
        self.approved_job.binding()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlOperationV1 {
    Ensure(Box<SchedulerOneShotScheduleV1>),
    Cancel {
        schedule_id: ScheduleIdV1,
        expected_revision: ScheduleRevisionV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerAdmittedScheduleControlV1 {
    command: OutboxRecordV1,
    operation_id: [u8; 16],
    grant: SchedulerScheduleControlGrantV1,
    operation: SchedulerScheduleControlOperationV1,
}

impl SchedulerAdmittedScheduleControlV1 {
    #[must_use]
    pub fn command(&self) -> &OutboxRecordV1 {
        &self.command
    }

    #[must_use]
    pub const fn operation_id(&self) -> &[u8; 16] {
        &self.operation_id
    }

    #[must_use]
    pub const fn grant(&self) -> &SchedulerScheduleControlGrantV1 {
        &self.grant
    }

    #[must_use]
    pub const fn operation(&self) -> &SchedulerScheduleControlOperationV1 {
        &self.operation
    }
}

pub fn admit_schedule_control_command_v1(
    exact_bytes: &[u8],
    contract: &SchedulerScheduleControlContractV1,
    grants: &[SchedulerScheduleControlGrantV1],
) -> Result<SchedulerAdmittedScheduleControlV1, SchedulerScheduleControlAdmissionErrorV1> {
    let command = OutboxRecordV1::accept(exact_bytes.to_vec())
        .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidEnvelope)?;
    let envelope = decode_envelope_v1(exact_bytes)
        .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidEnvelope)?;
    validate_contract(&envelope, contract)?;
    let grant = grants
        .iter()
        .find(|grant| grant_matches_envelope(grant, &envelope))
        .cloned()
        .ok_or(SchedulerScheduleControlAdmissionErrorV1::StaleFence)?;
    let payload = SchedulerScheduleControlCommandV1::decode(envelope.payload.as_slice())
        .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidCommand)?;
    validate_scheduler_schedule_control_command_v1(&payload)
        .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidCommand)?;
    let operation_id = fixed::<16>(&payload.operation_id)?;
    let operation = map_operation(&payload, &grant)?;
    Ok(SchedulerAdmittedScheduleControlV1 {
        command,
        operation_id,
        grant,
        operation,
    })
}

fn validate_contract(
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
    expected: &SchedulerScheduleControlContractV1,
) -> Result<(), SchedulerScheduleControlAdmissionErrorV1> {
    let contract = envelope
        .contract
        .as_ref()
        .ok_or(SchedulerScheduleControlAdmissionErrorV1::InvalidContract)?;
    let Some(Semantics::Command(metadata)) = envelope.semantics.as_ref() else {
        return Err(SchedulerScheduleControlAdmissionErrorV1::InvalidEnvelope);
    };
    (contract.owner == "scheduler"
        && contract.name == "schedule_control"
        && contract.major == 1
        && contract.revision == expected.revision
        && contract.schema_sha256 == expected.schema_sha256
        && metadata.target_capability == "scheduler_schedule_control")
        .then_some(())
        .ok_or(SchedulerScheduleControlAdmissionErrorV1::InvalidContract)
}

fn grant_matches_envelope(
    grant: &SchedulerScheduleControlGrantV1,
    envelope: &makosh_events_protocol::v1::DurableEnvelopeV1,
) -> bool {
    let Some(source) = envelope.source.as_ref() else {
        return false;
    };
    let Some(actor) = envelope.actor.as_ref() else {
        return false;
    };
    let Some(fence) = envelope.source_fence.as_ref() else {
        return false;
    };
    source.module_id == grant.source_module_id
        && source.runtime_instance_id == grant.source_runtime_instance_id
        && source.runtime_generation == grant.source_runtime_generation
        && actor.kind == ActorKindV1::Module as i32
        && actor.actor_id == grant.source_module_id.as_bytes()
        && fence.kind == FenceKindV1::GrantEpoch as i32
        && fence.scope_id == grant.source_module_id.as_bytes()
        && fence.epoch == grant.source_grant_epoch
}

fn map_operation(
    command: &SchedulerScheduleControlCommandV1,
    grant: &SchedulerScheduleControlGrantV1,
) -> Result<SchedulerScheduleControlOperationV1, SchedulerScheduleControlAdmissionErrorV1> {
    match command.operation.as_ref() {
        Some(Operation::EnsureOneShot(request)) => {
            job_matches(request.job_kind.as_ref(), grant)?;
            map_approved_one_shot_schedule_v1(request, &grant.approved_job)
                .map(Box::new)
                .map(SchedulerScheduleControlOperationV1::Ensure)
                .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidCommand)
        }
        Some(Operation::CancelOneShot(request)) => {
            job_matches(request.job_kind.as_ref(), grant)?;
            Ok(SchedulerScheduleControlOperationV1::Cancel {
                schedule_id: ScheduleIdV1::new(fixed(&request.schedule_id)?)
                    .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidCommand)?,
                expected_revision: ScheduleRevisionV1::new(request.expected_schedule_revision)
                    .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidCommand)?,
            })
        }
        None => Err(SchedulerScheduleControlAdmissionErrorV1::InvalidCommand),
    }
}

fn job_matches(
    job: Option<&makosh_scheduler_protocol::v1::JobKindV1>,
    grant: &SchedulerScheduleControlGrantV1,
) -> Result<(), SchedulerScheduleControlAdmissionErrorV1> {
    let job = job.ok_or(SchedulerScheduleControlAdmissionErrorV1::InvalidCommand)?;
    let approved = grant.approved_binding().job_kind();
    (job.owner == approved.owner()
        && job.name == approved.name()
        && job.major == u32::from(approved.major()))
    .then_some(())
    .ok_or(SchedulerScheduleControlAdmissionErrorV1::ForeignJobKind)
}

fn fixed<const N: usize>(
    value: &[u8],
) -> Result<[u8; N], SchedulerScheduleControlAdmissionErrorV1> {
    value
        .try_into()
        .map_err(|_| SchedulerScheduleControlAdmissionErrorV1::InvalidCommand)
}

fn valid_module_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerScheduleControlAdmissionErrorV1 {
    InvalidEnvelope,
    InvalidContract,
    InvalidGrant,
    InvalidCommand,
    StaleFence,
    ForeignJobKind,
}

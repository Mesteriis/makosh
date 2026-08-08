use makosh_clock_protocol::UtcMillisV1;

use super::{run::ScheduleRunLeaseV1, schedule::ScheduleSpecV1};
use crate::v1::{JobKindV1 as WireJobKindV1, JobLeaseV1, JobTriggerKindV1, ScheduledJobCommandV1};
use crate::validate_scheduled_job_command_v1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduledJobCommandBuildErrorV1 {
    InvalidTrigger,
    InvalidCommand,
}

pub fn build_scheduled_job_command_v1(
    schedule: &ScheduleSpecV1,
    lease: &ScheduleRunLeaseV1,
    scheduled_for: UtcMillisV1,
    trigger: JobTriggerKindV1,
) -> Result<ScheduledJobCommandV1, ScheduledJobCommandBuildErrorV1> {
    if trigger == JobTriggerKindV1::Unspecified {
        return Err(ScheduledJobCommandBuildErrorV1::InvalidTrigger);
    }
    let job_kind = schedule.binding().job_kind();
    let command = ScheduledJobCommandV1 {
        job_run_id: lease.run_id().bytes().to_vec(),
        job_kind: Some(WireJobKindV1 {
            owner: job_kind.owner().to_owned(),
            name: job_kind.name().to_owned(),
            major: u32::from(job_kind.major()),
        }),
        schedule_id: schedule.schedule_id().bytes().to_vec(),
        schedule_revision: schedule.revision().value(),
        scope_id: schedule.scope().value().to_owned(),
        trigger_kind: trigger as i32,
        scheduled_for_unix_millis: scheduled_for.value(),
        lease: Some(JobLeaseV1 {
            run_id: lease.run_id().bytes().to_vec(),
            epoch: lease.epoch(),
            expires_at_unix_millis: lease.expires_at().value(),
        }),
    };
    validate_scheduled_job_command_v1(&command)
        .map(|_| command)
        .map_err(|_| ScheduledJobCommandBuildErrorV1::InvalidCommand)
}

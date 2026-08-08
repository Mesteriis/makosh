use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{
    SchedulerReceiptValidationErrorV1,
    v1::{JobRunOutcomeV1, JobRunReceiptV1},
    validate_job_run_receipt_v1,
};

/// A validated acknowledgement for one immutable Scheduler command dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerRunAcceptanceV1 {
    run_id: [u8; 16],
    command_message_id: [u8; 16],
    lease_epoch: u64,
    observed_at: UtcMillisV1,
}

impl SchedulerRunAcceptanceV1 {
    #[must_use]
    pub const fn run_id(&self) -> [u8; 16] {
        self.run_id
    }

    #[must_use]
    pub const fn command_message_id(&self) -> [u8; 16] {
        self.command_message_id
    }

    #[must_use]
    pub const fn lease_epoch(&self) -> u64 {
        self.lease_epoch
    }

    #[must_use]
    pub const fn observed_at(&self) -> UtcMillisV1 {
        self.observed_at
    }
}

impl TryFrom<&JobRunReceiptV1> for SchedulerRunAcceptanceV1 {
    type Error = SchedulerRunAcceptanceErrorV1;

    fn try_from(receipt: &JobRunReceiptV1) -> Result<Self, Self::Error> {
        validate_job_run_receipt_v1(receipt).map_err(map_validation)?;
        let outcome = JobRunOutcomeV1::try_from(receipt.outcome)
            .map_err(|_| SchedulerRunAcceptanceErrorV1::Invalid)?;
        (outcome == JobRunOutcomeV1::Accepted)
            .then_some(())
            .ok_or(SchedulerRunAcceptanceErrorV1::NotAcceptance)?;
        let lease = receipt
            .lease
            .as_ref()
            .ok_or(SchedulerRunAcceptanceErrorV1::Invalid)?;
        Ok(Self {
            run_id: bytes(&receipt.job_run_id)?,
            command_message_id: bytes(&receipt.command_message_id)?,
            lease_epoch: lease.epoch,
            observed_at: UtcMillisV1::new(receipt.observed_at_unix_millis),
        })
    }
}

fn bytes(value: &[u8]) -> Result<[u8; 16], SchedulerRunAcceptanceErrorV1> {
    value
        .try_into()
        .map_err(|_| SchedulerRunAcceptanceErrorV1::Invalid)
}

fn map_validation(_: SchedulerReceiptValidationErrorV1) -> SchedulerRunAcceptanceErrorV1 {
    SchedulerRunAcceptanceErrorV1::Invalid
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerRunAcceptanceErrorV1 {
    Invalid,
    NotAcceptance,
}

//! Scheduler dispatch admission checks that do not require a live broker.

use makosh_runtime_protocol::v1::SchedulerRuntimeDispatchPublisherBindingV1;
use makosh_scheduler_jetstream::SchedulerJetStreamDispatchPortV1;
use makosh_scheduler_persistence::{SchedulerDispatchAdmissionV1, SchedulerMaterializationErrorV1};

#[test]
fn scheduler_accepts_only_exact_kernel_command_subjects() {
    let bindings = [SchedulerRuntimeDispatchPublisherBindingV1 {
        subject: "makosh.command.v1.owner_notes.sync_job.v1".to_owned(),
    }];
    assert!(SchedulerJetStreamDispatchPortV1::validate_bindings(&bindings).is_ok());
    assert_eq!(
        SchedulerDispatchAdmissionV1::new(Vec::new()),
        Err(SchedulerMaterializationErrorV1::InvalidAdmission)
    );
    assert_eq!(
        SchedulerDispatchAdmissionV1::new(["makosh.command.v1.owner.>".to_owned()]),
        Err(SchedulerMaterializationErrorV1::InvalidAdmission)
    );
}

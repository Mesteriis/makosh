use std::ffi::OsString;
use std::path::PathBuf;

use crate::runtime_cli::parse_recovery_arguments;
use makosh_scheduler::{SchedulerApprovedJobV1, map_approved_one_shot_schedule_v1};
use makosh_scheduler_protocol::v1::{
    CancelOneShotScheduleV1, EnsureOneShotScheduleV1, JobKindV1, SchedulerScheduleControlCommandV1,
    SchedulerScheduleControlOutcomeV1, SchedulerScheduleControlResultV1,
    scheduler_schedule_control_command_v1::Operation,
};
use makosh_scheduler_protocol::{
    JobContractBindingV1, JobKindV1 as CanonicalJobKindV1, MisfirePolicyV1, OverlapPolicyV1,
    ScheduleTriggerV1, SchedulerScheduleControlValidationErrorV1,
    validate_scheduler_schedule_control_command_v1, validate_scheduler_schedule_control_result_v1,
};

#[test]
fn scheduler_recovery_cli_accepts_only_fixed_non_secret_arguments() {
    let mut arguments = [
        "--host",
        "127.0.0.1",
        "--port",
        "5432",
        "--database",
        "makosh",
        "--username",
        "recovery",
        "--ssl-mode",
        "verify-full",
        "--password-file",
        "/private/password",
        "--storage-bundle",
        "/private/storage-bundle.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    let parsed: crate::runtime_cli::RecoveryArguments =
        parse_recovery_arguments(&mut arguments).expect("recovery arguments");
    assert_eq!(parsed.host, "127.0.0.1");
    assert_eq!(parsed.port, 5432);
    assert_eq!(parsed.password_file, PathBuf::from("/private/password"));
    assert_eq!(
        parsed.storage_bundle,
        PathBuf::from("/private/storage-bundle.pb")
    );
}

#[test]
fn scheduler_recovery_bundle_export_requires_one_absolute_output() {
    let mut valid = ["--output", "/private/storage-bundle.pb"]
        .into_iter()
        .map(OsString::from)
        .peekable();
    assert_eq!(
        crate::runtime_cli::parse_export_bundle_arguments(&mut valid),
        Ok(PathBuf::from("/private/storage-bundle.pb"))
    );
    let mut relative = ["--output", "storage-bundle.pb"]
        .into_iter()
        .map(OsString::from)
        .peekable();
    assert!(crate::runtime_cli::parse_export_bundle_arguments(&mut relative).is_err());
}

#[test]
fn scheduler_inherited_cli_remains_available_in_the_shared_parser() {
    let mut arguments = [
        "--descriptor-path",
        "/private/descriptor.pb",
        "--settings-schema-path",
        "/private/settings.pb",
        "--configuration-path",
        "/private/configuration.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    let parsed = crate::runtime_cli::parse_serve_inherited_arguments(&mut arguments)
        .expect("inherited arguments");
    assert_eq!(
        parsed.descriptor_path,
        PathBuf::from("/private/descriptor.pb")
    );
    assert_eq!(
        parsed.settings_schema_path,
        Some(PathBuf::from("/private/settings.pb"))
    );
    assert_eq!(
        parsed.configuration_path,
        PathBuf::from("/private/configuration.pb")
    );
}

#[test]
fn scheduler_recovery_cli_rejects_passwords_and_relative_files() {
    let mut secret = [
        "--host",
        "user:secret@localhost",
        "--port",
        "5432",
        "--database",
        "makosh",
        "--username",
        "recovery",
        "--ssl-mode",
        "disable",
        "--password-file",
        "/private/password",
        "--storage-bundle",
        "/private/storage-bundle.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    assert!(parse_recovery_arguments(&mut secret).is_err());

    let mut relative = [
        "--host",
        "localhost",
        "--port",
        "5432",
        "--database",
        "makosh",
        "--username",
        "recovery",
        "--ssl-mode",
        "disable",
        "--password-file",
        "password",
        "--storage-bundle",
        "/private/storage-bundle.pb",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    assert!(parse_recovery_arguments(&mut relative).is_err());
}

#[test]
fn module_schedule_control_accepts_only_bounded_one_shot_contracts() {
    let command = ensure_command();
    assert_eq!(
        validate_scheduler_schedule_control_command_v1(&command),
        Ok(())
    );

    let mut foreign_policy = command.clone();
    let Some(Operation::EnsureOneShot(request)) = foreign_policy.operation.as_mut() else {
        panic!("ensure operation");
    };
    request.max_attempts = 33;
    assert_eq!(
        validate_scheduler_schedule_control_command_v1(&foreign_policy),
        Err(SchedulerScheduleControlValidationErrorV1::InvalidPolicy)
    );

    let mut secret_scope = command;
    let Some(Operation::EnsureOneShot(request)) = secret_scope.operation.as_mut() else {
        panic!("ensure operation");
    };
    request.scope_id = "mailbox@example.com".to_owned();
    assert_eq!(
        validate_scheduler_schedule_control_command_v1(&secret_scope),
        Err(SchedulerScheduleControlValidationErrorV1::InvalidScope)
    );
}

#[test]
fn module_schedule_cancel_and_result_are_exact_and_sanitized() {
    let cancel = SchedulerScheduleControlCommandV1 {
        operation_id: vec![8; 16],
        operation: Some(Operation::CancelOneShot(CancelOneShotScheduleV1 {
            schedule_id: vec![9; 16],
            expected_schedule_revision: 2,
            job_kind: Some(JobKindV1 {
                owner: "communication_delayed_delivery".to_owned(),
                name: "execute".to_owned(),
                major: 1,
            }),
        })),
    };
    assert_eq!(
        validate_scheduler_schedule_control_command_v1(&cancel),
        Ok(())
    );

    let result = SchedulerScheduleControlResultV1 {
        operation_id: vec![8; 16],
        schedule_id: vec![9; 16],
        schedule_revision: 2,
        outcome: SchedulerScheduleControlOutcomeV1::Rejected.into(),
        error_code: "stale_revision".to_owned(),
    };
    assert_eq!(
        validate_scheduler_schedule_control_result_v1(&result),
        Ok(())
    );

    let mut raw_error = result;
    raw_error.error_code = "provider secret leaked".to_owned();
    assert_eq!(
        validate_scheduler_schedule_control_result_v1(&raw_error),
        Err(SchedulerScheduleControlValidationErrorV1::InvalidErrorCode)
    );
}

#[test]
fn approved_one_shot_mapping_uses_exact_catalog_contract_and_bounded_policy() {
    let command = ensure_command();
    let Some(Operation::EnsureOneShot(request)) = command.operation.as_ref() else {
        panic!("ensure operation");
    };
    let approved = approved_job();
    let mapped =
        map_approved_one_shot_schedule_v1(request, &approved).expect("approved one-shot schedule");

    assert_eq!(mapped.spec().binding(), approved.binding());
    assert_eq!(mapped.spec().schedule_id().bytes(), [9; 16]);
    assert_eq!(mapped.spec().revision().value(), 1);
    assert_eq!(mapped.spec().scope().value(), "delayed-operation-1");
    assert_eq!(
        mapped.spec().concurrency_key().value(),
        "delayed-operation-1"
    );
    assert_eq!(mapped.spec().policy().overlap(), OverlapPolicyV1::Forbid);
    assert_eq!(mapped.spec().policy().misfire(), MisfirePolicyV1::FireOnce);
    assert_eq!(mapped.spec().policy().retry().max_attempts(), 3);
    assert_eq!(
        mapped.spec().policy().trigger(),
        &ScheduleTriggerV1::At {
            due_at: mapped.next_due_at()
        }
    );
}

#[test]
fn one_shot_mapping_rejects_foreign_or_stale_catalog_contracts() {
    let command = ensure_command();
    let Some(Operation::EnsureOneShot(request)) = command.operation.as_ref() else {
        panic!("ensure operation");
    };
    let approved = approved_job();

    let mut foreign = request.clone();
    foreign.job_kind.as_mut().expect("job kind").owner = "mail".to_owned();
    assert!(map_approved_one_shot_schedule_v1(&foreign, &approved).is_err());

    let mut stale = request.clone();
    stale.job_contract_revision = 2;
    assert!(map_approved_one_shot_schedule_v1(&stale, &approved).is_err());
}

fn approved_job() -> SchedulerApprovedJobV1 {
    let kind = CanonicalJobKindV1::new(
        "communication_delayed_delivery".to_owned(),
        "execute".to_owned(),
        1,
    )
    .expect("canonical job kind");
    let binding = JobContractBindingV1::new(
        kind,
        "communication_delayed_delivery.execute".to_owned(),
        1,
        [5; 32],
    )
    .expect("catalog contract");
    SchedulerApprovedJobV1::new("communication_delayed_delivery".to_owned(), binding)
        .expect("owner-fenced job")
}

fn ensure_command() -> SchedulerScheduleControlCommandV1 {
    SchedulerScheduleControlCommandV1 {
        operation_id: vec![7; 16],
        operation: Some(Operation::EnsureOneShot(EnsureOneShotScheduleV1 {
            schedule_id: vec![9; 16],
            schedule_revision: 1,
            job_kind: Some(JobKindV1 {
                owner: "communication_delayed_delivery".to_owned(),
                name: "execute".to_owned(),
                major: 1,
            }),
            job_contract_revision: 1,
            job_schema_sha256: vec![5; 32],
            scope_id: "delayed-operation-1".to_owned(),
            concurrency_key: "delayed-operation-1".to_owned(),
            due_at_unix_millis: 1_800_000_000_000,
            deadline_millis: 30_000,
            max_attempts: 3,
            retry_base_backoff_millis: 1_000,
        })),
    }
}

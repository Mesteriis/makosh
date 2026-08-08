use makosh_kernel_control_store::PlatformEventsAuthorityConfigurationV1;
use makosh_runtime_protocol::v1::{
    EventsAuthorityRuntimeControlResponseV1, EventsAuthorityRuntimeStateV1,
    EventsAuthorityRuntimeStatusV1,
    events_authority_runtime_control_response_v1::Result as ResponseResult,
};

use crate::platform::events::authority::{launch, status};

use super::common::StagedRuntimeContracts;

const ACCOUNT_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[test]
fn events_authority_launch_arguments_contain_only_staged_contract_paths() {
    let root = super::common::unique_target_root("makosh-events-authority-arguments");
    let contracts = StagedRuntimeContracts::stage_with_runtime_configuration(
        &root.join("contracts"),
        b"descriptor",
        Some(b"schema"),
        Some(b"configuration"),
    )
    .expect("stage authority contracts");

    let arguments = launch::inherited_arguments(&contracts).expect("authority arguments");

    assert_eq!(arguments[0], "serve-inherited");
    assert_eq!(arguments[1], "--descriptor-path");
    assert_eq!(arguments[3], "--settings-schema-path");
    assert_eq!(arguments[5], "--configuration-path");
    assert!(!arguments.iter().any(|argument| argument.contains("seed")));
    assert!(
        !arguments
            .iter()
            .any(|argument| argument.contains("password"))
    );

    contracts
        .remove()
        .expect("remove staged authority contracts");
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn events_authority_status_requires_current_signer_revision() {
    let configuration = PlatformEventsAuthorityConfigurationV1::new(1, ACCOUNT_KEY, 4);

    let status = status::parse_current(ready_response(7, 4), 7, &configuration)
        .expect("current Events authority status");

    assert_eq!(status.runtime_generation(), 7);
    assert!(status::parse_current(ready_response(7, 3), 7, &configuration).is_err());
    assert!(status::parse_current(ready_response(6, 4), 7, &configuration).is_err());
}

fn ready_response(
    runtime_generation: u64,
    signer_credential_revision: u64,
) -> EventsAuthorityRuntimeControlResponseV1 {
    EventsAuthorityRuntimeControlResponseV1 {
        result: Some(ResponseResult::Status(EventsAuthorityRuntimeStatusV1 {
            state: EventsAuthorityRuntimeStateV1::Ready as i32,
            runtime_generation,
            grant_epoch: 2,
            vault_runtime_generation: 3,
            signer_credential_revision,
            blocker_code: String::new(),
        })),
        error_code: String::new(),
    }
}

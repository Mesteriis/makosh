use makosh_kernel_control_store::PlatformEventsAuthorityConfigurationV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use super::common::unique_target_root;

const ACCOUNT_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[test]
fn events_authority_configuration_is_durable_and_monotonically_fenced() {
    let root = unique_target_root("makosh-events-authority-configuration");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let first = configuration(1, 1);

    store
        .record_platform_events_authority_configuration(&first)
        .expect("record initial Events authority configuration");
    assert_eq!(
        store
            .platform_events_authority_configuration()
            .expect("read Events authority configuration"),
        Some(first.clone())
    );
    assert!(
        store
            .record_platform_events_authority_configuration(&configuration(1, 2))
            .is_err()
    );
    assert!(
        store
            .record_platform_events_authority_configuration(&configuration(2, 0))
            .is_err()
    );

    let rotated = configuration(2, 2);
    store
        .record_platform_events_authority_configuration(&rotated)
        .expect("rotate Events authority signer revision");
    assert_eq!(
        store
            .platform_events_authority_configuration()
            .expect("read rotated Events authority configuration"),
        Some(rotated)
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

#[test]
fn events_authority_configuration_rejects_non_account_identity() {
    let root = unique_target_root("makosh-events-authority-configuration-invalid");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "instance-1", 1)
        .expect("create Control Store");
    let invalid = PlatformEventsAuthorityConfigurationV1::new(1, "not-an-account-key", 1);

    assert!(
        store
            .record_platform_events_authority_configuration(&invalid)
            .is_err()
    );
    assert!(
        store
            .platform_events_authority_configuration()
            .expect("read absent Events authority configuration")
            .is_none()
    );
    std::fs::remove_dir_all(root).expect("remove fixture directory");
}

fn configuration(
    revision: u64,
    signer_credential_revision: u64,
) -> PlatformEventsAuthorityConfigurationV1 {
    PlatformEventsAuthorityConfigurationV1::new(revision, ACCOUNT_KEY, signer_credential_revision)
}

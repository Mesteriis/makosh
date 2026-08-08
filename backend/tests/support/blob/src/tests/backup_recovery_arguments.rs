use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::blob_service_cli::{
    OfflineRecoveryCommand, parse_offline_recovery_command, parse_serve_inherited_arguments,
};

#[test]
fn offline_blob_recovery_commands_preserve_the_exact_paths() {
    let mut export = [
        "--data-dir",
        "/private/makosh/blob",
        "--destination",
        "/private/makosh/recovery/blob",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    let export = parse_offline_recovery_command(&OsString::from("export-backup"), &mut export)
        .expect("parse export command");
    assert!(matches!(
        export,
        OfflineRecoveryCommand::Export { data_dir, destination }
            if data_dir == Path::new("/private/makosh/blob")
                && destination == Path::new("/private/makosh/recovery/blob")
    ));

    let mut restore = [
        "--source",
        "/private/makosh/recovery/blob",
        "--data-dir",
        "/private/makosh/blob-restored",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    let restore = parse_offline_recovery_command(&OsString::from("restore-backup"), &mut restore)
        .expect("parse restore command");
    assert!(matches!(
        restore,
        OfflineRecoveryCommand::Restore { source, data_dir }
            if source == Path::new("/private/makosh/recovery/blob")
                && data_dir == Path::new("/private/makosh/blob-restored")
    ));

    let mut verify = ["--source", "/private/makosh/recovery/blob"]
        .into_iter()
        .map(OsString::from)
        .peekable();
    let verify = parse_offline_recovery_command(&OsString::from("verify-backup"), &mut verify)
        .expect("parse verify command");
    assert!(matches!(
        verify,
        OfflineRecoveryCommand::Verify { source }
            if source == Path::new("/private/makosh/recovery/blob")
    ));
}

#[test]
fn blob_recovery_arguments_reject_relative_paths_without_changing_the_runtime_contract() {
    let mut inherited = [
        "--descriptor-path",
        "/private/makosh/descriptor",
        "--configuration-path",
        "/private/makosh/configuration",
    ]
    .into_iter()
    .map(OsString::from)
    .peekable();
    let inherited =
        parse_serve_inherited_arguments(&mut inherited).expect("parse inherited command");
    assert_eq!(
        inherited.descriptor_path,
        PathBuf::from("/private/makosh/descriptor")
    );
    assert_eq!(inherited.settings_schema_path, None);
    assert_eq!(
        inherited.configuration_path,
        PathBuf::from("/private/makosh/configuration")
    );

    let mut relative = ["--source", "relative"]
        .into_iter()
        .map(OsString::from)
        .peekable();
    assert!(
        parse_offline_recovery_command(&OsString::from("verify-backup"), &mut relative).is_err()
    );
}

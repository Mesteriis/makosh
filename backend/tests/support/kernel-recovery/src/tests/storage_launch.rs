use super::common::*;

#[test]
fn storage_launch_arguments_expose_only_staged_contract_paths() {
    let root = unique_target_root("makosh-storage-launch-arguments");
    let contracts = StagedRuntimeContracts::stage_with_runtime_configuration(
        &root.join("contracts"),
        b"descriptor",
        Some(b"schema"),
        Some(b"configuration"),
    )
    .expect("stage contracts");

    let arguments = storage_launch::inherited_arguments(&contracts);

    assert_eq!(arguments[0], "serve-inherited");
    assert_eq!(arguments[1], "--descriptor-path");
    assert_eq!(
        arguments[2],
        contracts.descriptor_path().display().to_string()
    );
    assert_eq!(arguments[3], "--settings-schema-path");
    assert_eq!(
        arguments[4],
        contracts
            .settings_schema_path()
            .expect("schema path")
            .display()
            .to_string()
    );
    assert_eq!(arguments[5], "--configuration-path");
    assert_eq!(
        arguments[6],
        contracts
            .runtime_configuration_path()
            .expect("configuration path")
            .display()
            .to_string()
    );
    assert!(
        !arguments
            .iter()
            .any(|argument| argument.contains("password"))
    );

    contracts.remove().expect("remove contracts");
    std::fs::remove_dir_all(root).expect("remove fixture");
}

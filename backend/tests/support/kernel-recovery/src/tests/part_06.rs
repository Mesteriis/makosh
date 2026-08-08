use super::common::*;

#[test]
fn staged_runtime_contracts_preserve_verified_bytes_and_remove_them_after_launch() {
    let root = unique_target_root("makosh-staged-runtime-contracts");
    let contracts = StagedRuntimeContracts::stage(
        &root.join("contracts"),
        b"verified-descriptor-bytes",
        Some(b"verified-settings-schema-bytes"),
    )
    .expect("stage contracts");
    let descriptor = contracts.descriptor_path().to_owned();
    let schema = contracts
        .settings_schema_path()
        .expect("schema path")
        .to_owned();
    assert_eq!(
        std::fs::read(&descriptor).expect("read descriptor"),
        b"verified-descriptor-bytes"
    );
    assert_eq!(
        std::fs::read(&schema).expect("read schema"),
        b"verified-settings-schema-bytes"
    );
    assert_eq!(
        std::fs::metadata(&descriptor)
            .expect("descriptor metadata")
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
    contracts.remove().expect("remove contracts");
    assert!(!descriptor.exists());
    assert!(!schema.exists());
    std::fs::remove_dir_all(root).expect("remove contract fixture");
}

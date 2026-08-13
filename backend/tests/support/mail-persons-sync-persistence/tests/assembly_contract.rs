use makosh_mail_persons_sync_assembly::{
    mail_persons_sync_artifact_fragment_v1, materialize_mail_persons_sync_assembly_v1,
};

#[test]
fn unsigned_fragment_is_deterministic_sorted_and_binds_contract_inputs() {
    let first = mail_persons_sync_artifact_fragment_v1("/private/build/runtime").expect("fragment");
    let second =
        mail_persons_sync_artifact_fragment_v1("/private/build/runtime").expect("deterministic");
    assert_eq!(first, second);
    let encoded = serde_json::to_string(&first).expect("json");
    assert_eq!(first.owner_id, "mail_persons_sync");
    assert_eq!(first.module_id, "makosh-mail-persons-sync-runtime");
    assert_eq!(first.artifacts.len(), 2);
    assert!(encoded.contains("mail_persons_sync.runtime.v1"));
    assert!(encoded.contains("mail_persons_sync.storage.v1"));
    assert!(encoded.contains("descriptor"));
    assert!(encoded.contains("settings_schema"));
    assert!(!encoded.contains("signature"));
    let ids = first
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_id())
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn materialization_writes_all_private_artifacts_and_rejects_partial_inputs() {
    let root = std::env::temp_dir().join(format!(
        "makosh-mail-persons-sync-assembly-{}",
        std::process::id()
    ));
    let runtime = root.join("runtime-bin");
    std::fs::create_dir_all(&root).expect("temp root");
    std::fs::write(&runtime, b"runtime").expect("runtime fixture");
    let output = root.join("out");
    let paths =
        materialize_mail_persons_sync_assembly_v1(&output, "build-1", &runtime).expect("assembly");
    for path in [
        &paths.descriptor,
        &paths.settings_schema,
        &paths.storage_bundle,
        &paths.artifact_fragment,
    ] {
        assert!(path.is_file(), "{}", path.display());
    }
    assert!(materialize_mail_persons_sync_assembly_v1(&root.join("bad"), "", &runtime).is_err());
    assert!(
        materialize_mail_persons_sync_assembly_v1(
            &root.join("bad2"),
            "build",
            &root.join("missing")
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).expect("cleanup");
}

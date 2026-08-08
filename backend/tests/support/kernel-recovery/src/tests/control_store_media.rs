use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::control_store::lifecycle::{bootstrap_control_store, open_validated_control_store};
use crate::recovery::control_store_media::{capture, restore_to_empty_target};

use super::common::unique_target_root;

#[test]
fn control_store_media_restores_authority_and_advances_all_fences() {
    let fixture = Fixture::new("control-store-media-round-trip");
    let source = bootstrap_control_store(
        &fixture.source,
        &fixture.source.join("kernel-control-store.sqlite"),
    )
    .expect("bootstrap source");
    let source_snapshot = source.snapshot().clone();

    capture(&fixture.source, &fixture.media).expect("capture Control Store media");
    assert_eq!(
        names(&fixture.media),
        [
            ".makosh-installation-anchor-v1",
            ".makosh-recovery-fence-v1",
            "control-store.sqlite",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    restore_to_empty_target(&fixture.media, &fixture.target).expect("restore Control Store");

    let restored =
        open_validated_control_store(&fixture.target.join("kernel-control-store.sqlite"))
            .expect("open restored Control Store");
    assert_eq!(
        restored.snapshot().instance_id(),
        source_snapshot.instance_id()
    );
    assert_eq!(
        restored.snapshot().generation(),
        source_snapshot.generation() + 1
    );
    assert_eq!(
        restored.snapshot().identity_epoch(),
        source_snapshot.identity_epoch() + 1
    );
    assert_eq!(
        restored.snapshot().grant_epoch(),
        source_snapshot.grant_epoch() + 1
    );
}

#[test]
fn control_store_media_rejects_replayed_authority_from_another_instance() {
    let first = Fixture::new("control-store-media-first");
    let second = Fixture::new("control-store-media-second");
    bootstrap(&first.source);
    bootstrap(&second.source);
    capture(&first.source, &first.media).expect("capture first instance");
    capture(&second.source, &second.media).expect("capture second instance");
    std::fs::copy(
        second.media.join(".makosh-recovery-fence-v1"),
        first.media.join(".makosh-recovery-fence-v1"),
    )
    .expect("replace authority fence");

    assert!(restore_to_empty_target(&first.media, &first.target).is_err());
    assert!(names(&first.target).is_empty());
}

#[test]
fn control_store_media_rejects_a_fence_that_does_not_match_its_snapshot() {
    let fixture = Fixture::new("control-store-media-fence-drift");
    bootstrap(&fixture.source);
    capture(&fixture.source, &fixture.media).expect("capture instance");
    restore_to_empty_target(&fixture.media, &fixture.target).expect("restore instance");
    std::fs::copy(
        fixture.target.join(".makosh-recovery-fence-v1"),
        fixture.media.join(".makosh-recovery-fence-v1"),
    )
    .expect("replace snapshot fence with later fence");
    let second_target = fixture.root.join("second-target");
    std::fs::create_dir(&second_target).expect("create second target");
    std::fs::set_permissions(&second_target, std::fs::Permissions::from_mode(0o700))
        .expect("private second target");

    assert!(restore_to_empty_target(&fixture.media, &second_target).is_err());
    assert!(names(&second_target).is_empty());
}

#[test]
fn control_store_media_rejects_noncanonical_component_layout() {
    let fixture = Fixture::new("control-store-media-layout");
    bootstrap(&fixture.source);
    capture(&fixture.source, &fixture.media).expect("capture instance");
    std::fs::write(fixture.media.join("unexpected"), b"unexpected").expect("add extra entry");
    std::fs::set_permissions(
        fixture.media.join("unexpected"),
        std::fs::Permissions::from_mode(0o600),
    )
    .expect("restrict extra entry");

    assert!(restore_to_empty_target(&fixture.media, &fixture.target).is_err());
    assert!(names(&fixture.target).is_empty());
}

fn bootstrap(data_dir: &Path) {
    bootstrap_control_store(data_dir, &data_dir.join("kernel-control-store.sqlite"))
        .expect("bootstrap Control Store");
}

fn names(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .expect("read fixture directory")
        .map(|entry| {
            entry
                .expect("fixture entry")
                .file_name()
                .into_string()
                .expect("fixture name")
        })
        .collect()
}

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    media: PathBuf,
    target: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = unique_target_root(name);
        let source = root.join("source");
        let media = root.join("media");
        let target = root.join("target");
        std::fs::create_dir_all(&source).expect("create source");
        std::fs::create_dir(&media).expect("create media");
        std::fs::create_dir(&target).expect("create target");
        for directory in [&root, &source, &media, &target] {
            std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
                .expect("private fixture directory");
        }
        Self {
            root,
            source,
            media,
            target,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

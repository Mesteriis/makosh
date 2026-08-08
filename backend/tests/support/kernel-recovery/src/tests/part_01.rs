use super::common::*;

#[test]
fn a_new_store_exposes_a_trustworthy_health_snapshot() {
    let store = ControlStore::new("instance-1", 1);

    assert_eq!(store.health(), StoreHealth::Trustworthy);
    assert_eq!(store.instance_id(), "instance-1");
    assert_eq!(store.generation(), 1);
}

#[test]
fn distribution_manifest_requires_exact_ordered_artifact_bindings() {
    let manifest = DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "makosh-desktop".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "build-1".to_owned(),
        target_triple: "aarch64-apple-darwin".to_owned(),
        generation: 1,
        artifacts: vec![DistributionManifestArtifactV1 {
            artifact_kind: DistributionArtifactKindV1::ModuleRuntime as i32,
            artifact_id: "runtime.mail".to_owned(),
            relative_path: "bin/mail".to_owned(),
            size_bytes: 1,
            sha256: vec![7; 32],
            descriptor_sha256: vec![8; 32],
            settings_schema_sha256: Vec::new(),
            required: true,
            descriptor_relative_path: "contracts/mail-descriptor.pb".to_owned(),
            descriptor_size_bytes: 1,
            settings_schema_relative_path: String::new(),
            settings_schema_size_bytes: 0,
            bound_module_id: String::new(),
        }],
    };
    assert_eq!(
        decode_distribution_manifest_v1(&manifest.encode_to_vec())
            .expect("manifest")
            .generation,
        1
    );
    let mut invalid = manifest.clone();
    invalid.artifacts[0].relative_path = "../mail".to_owned();
    assert!(decode_distribution_manifest_v1(&invalid.encode_to_vec()).is_err());
    let signed = SignedDistributionManifestV1 {
        verification_key_id: "release-2026".to_owned(),
        raw_manifest_bytes: manifest.encode_to_vec(),
        signature_raw: vec![9; 64],
    };
    assert!(decode_signed_distribution_manifest_v1(&signed.encode_to_vec()).is_ok());
}

#[test]
fn release_trust_root_accepts_only_ordered_valid_p256_public_keys() {
    let signing_key = SigningKey::from_bytes((&[9_u8; 32]).into()).expect("test key");
    let public_key_sec1: [u8; 65] = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed P-256 key");
    let root = ReleaseTrustRootV1 {
        major: 1,
        revision: 1,
        verification_keys: vec![ReleaseTrustRootKeyV1 {
            key_id: "release-2026".to_owned(),
            public_key_sec1: public_key_sec1.to_vec(),
        }],
    };
    assert!(ReleaseTrustRoot::decode(&root.encode_to_vec()).is_ok());
    let file_path = std::env::temp_dir().join(format!(
        "makosh-release-trust-root-{}-{}.pb",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
    ));
    std::fs::write(&file_path, root.encode_to_vec()).expect("write trust root");
    assert!(ReleaseTrustRoot::load(&file_path).is_ok());
    let link_path = file_path.with_extension("link");
    std::os::unix::fs::symlink(&file_path, &link_path).expect("link trust root");
    assert_eq!(
        ReleaseTrustRoot::load(&link_path)
            .err()
            .expect("symlink is rejected"),
        "release trust root is not a bounded regular file"
    );
    std::fs::remove_file(&link_path).expect("remove trust root link");
    std::fs::remove_file(&file_path).expect("remove trust root");

    let oversized_path = file_path.with_extension("oversized");
    std::fs::write(&oversized_path, vec![0_u8; 16 * 1024 + 1]).expect("write oversized trust root");
    assert_eq!(
        ReleaseTrustRoot::load(&oversized_path)
            .err()
            .expect("oversized trust root is rejected"),
        "release trust root is not a bounded regular file"
    );
    std::fs::remove_file(&oversized_path).expect("remove oversized trust root");

    let mut invalid = root;
    invalid.verification_keys[0].public_key_sec1[1] ^= 1;
    assert_eq!(
        ReleaseTrustRoot::decode(&invalid.encode_to_vec())
            .err()
            .expect("invalid point"),
        "release trust root is invalid"
    );
}

#[test]
fn macos_code_signature_identity_requires_an_exact_team_identifier_line() {
    assert!(macos_code_signature::has_expected_team_id(
        b"Authority=Developer ID\nTeamIdentifier=AB12CD34EF\n",
        "AB12CD34EF",
    ));
    assert!(!macos_code_signature::has_expected_team_id(
        b"TeamIdentifier=AB12CD34EF-suffix\n",
        "AB12CD34EF",
    ));
}

#[test]
fn macos_code_signature_adapter_fails_closed_for_a_mismatched_team_identity() {
    let result = macos_code_signature::verify(std::path::Path::new("/usr/bin/true"), "AB12CD34EF");
    if cfg!(target_os = "macos") {
        assert_eq!(
            result.expect_err("system binary must not match a fabricated Team ID"),
            "macOS code-signature team identity does not match"
        );
    } else {
        assert_eq!(
            result.expect_err("non-macOS platform"),
            "macOS code-signature verification is unavailable on this platform"
        );
    }
}

#[test]
fn distribution_manifest_verifier_requires_pinned_key_and_exact_signed_bytes() {
    let signing_key = SigningKey::from_bytes((&[7_u8; 32]).into()).expect("test key");
    let manifest = DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "makosh-desktop".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "build-1".to_owned(),
        target_triple: "aarch64-apple-darwin".to_owned(),
        generation: 1,
        artifacts: vec![DistributionManifestArtifactV1 {
            artifact_kind: DistributionArtifactKindV1::ModuleRuntime as i32,
            artifact_id: "runtime.mail".to_owned(),
            relative_path: "bin/mail".to_owned(),
            size_bytes: 1,
            sha256: vec![1; 32],
            descriptor_sha256: vec![2; 32],
            settings_schema_sha256: Vec::new(),
            required: true,
            descriptor_relative_path: "contracts/mail-descriptor.pb".to_owned(),
            descriptor_size_bytes: 1,
            settings_schema_relative_path: String::new(),
            settings_schema_size_bytes: 0,
            bound_module_id: String::new(),
        }],
    };
    let raw_manifest_bytes = manifest.encode_to_vec();
    let signature: Signature = signing_key.sign(&raw_manifest_bytes);
    let signed = SignedDistributionManifestV1 {
        verification_key_id: "release-2026".to_owned(),
        raw_manifest_bytes,
        signature_raw: signature.to_bytes().to_vec(),
    };
    let public_key_sec1: [u8; 65] = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("uncompressed P-256 key");
    let trust_root = release_trust_root(&[("release-2026", public_key_sec1)]);

    assert_eq!(
        distribution_manifest_verifier::verify(&signed.encode_to_vec(), &trust_root)
            .expect("verified manifest"),
        manifest
    );
    let another_root = release_trust_root(&[("release-2027", public_key_sec1)]);
    assert_eq!(
        distribution_manifest_verifier::verify(&signed.encode_to_vec(), &another_root)
            .expect_err("missing key"),
        "distribution verification key is not pinned"
    );
    let mut tampered = signed;
    tampered.raw_manifest_bytes[0] ^= 1;
    assert_eq!(
        distribution_manifest_verifier::verify(&tampered.encode_to_vec(), &trust_root)
            .expect_err("tampered manifest"),
        "signed distribution manifest is invalid"
    );
}

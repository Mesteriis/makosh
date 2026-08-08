use super::*;

pub(super) fn bundle_descriptor(
    schema: &SettingsSchemaV1,
    settings_schema_digest: [u8; 32],
    settings_schema_size: usize,
    identity: BundleIdentity,
) -> ModuleDescriptorV1 {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: identity.module_id.to_owned(),
        owner_id: identity.owner_id.to_owned(),
        module_kind: identity.module_kind as i32,
        module_version: "1".to_owned(),
        build_id: "build-1".to_owned(),
        capabilities: vec![CapabilityDescriptorV1 {
            capability_id: identity.capability_id.to_owned(),
            capability_revision: 1,
            criticality: CapabilityCriticalityV1::Required as i32,
            ..Default::default()
        }],
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: schema.major,
            revision: schema.revision,
            artifact_size_bytes: settings_schema_size as u64,
            sha256: settings_schema_digest.to_vec(),
        }),
        ..Default::default()
    }
}

pub(super) fn bundle_manifest(
    artifact_digest: [u8; 32],
    descriptor_digest: [u8; 32],
    settings_schema_digest: [u8; 32],
    descriptor_size: usize,
    settings_schema_size: usize,
    identity: BundleIdentity,
) -> DistributionManifestV1 {
    DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "makosh-desktop".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "build-1".to_owned(),
        target_triple: "aarch64-apple-darwin".to_owned(),
        generation: 1,
        artifacts: vec![DistributionManifestArtifactV1 {
            artifact_kind: DistributionArtifactKindV1::ModuleRuntime as i32,
            artifact_id: identity.artifact_id.to_owned(),
            relative_path: identity.artifact_relative_path.to_owned(),
            size_bytes: DistributionBundleFixture::artifact_bytes().len() as u64,
            sha256: artifact_digest.to_vec(),
            descriptor_sha256: descriptor_digest.to_vec(),
            settings_schema_sha256: settings_schema_digest.to_vec(),
            required: true,
            descriptor_relative_path: identity.descriptor_relative_path.to_owned(),
            descriptor_size_bytes: descriptor_size as u64,
            settings_schema_relative_path: identity.settings_schema_relative_path.to_owned(),
            settings_schema_size_bytes: settings_schema_size as u64,
            bound_module_id: String::new(),
        }],
    }
}

pub(super) fn sign_bundle_manifest(
    manifest: &DistributionManifestV1,
    signing_key: &SigningKey,
) -> (SignedDistributionManifestV1, ReleaseTrustRoot) {
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
    (
        signed,
        release_trust_root(&[("release-2026", public_key_sec1)]),
    )
}

pub(super) fn register_approved_module(
    store: &SqliteControlStore,
    registration_id: &str,
    module_id: &str,
    owner_id: &str,
    descriptor_digest: [u8; 32],
) {
    let registration = ModuleRegistration::new(
        registration_id,
        module_id,
        owner_id,
        descriptor_digest,
        ModuleRegistrationState::Pending,
        1,
    );
    store
        .create_pending_registration(&registration, &["read".to_owned()])
        .expect("create registration");
    store
        .approve_module_registration(registration_id, &["read".to_owned()])
        .expect("approve registration");
}

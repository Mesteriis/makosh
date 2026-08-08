use super::common::*;
use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeResponseV1, GetTelemetryDiagnosticsRequestV1,
    ManagedRuntimeReadyRequestV1, TelemetryDiagnosticsV1, TelemetryRuntimeControlRequestV1,
    TelemetryRuntimeControlResponseV1,
    managed_runtime_control_request_v1::Operation as ManagedOperation,
    managed_runtime_control_response_v1::Result as ManagedResult,
    telemetry_runtime_control_request_v1::Operation as TelemetryOperation,
    telemetry_runtime_control_response_v1::Result as TelemetryResult,
};

const TARGET: &str = "aarch64-apple-darwin";
const ARTIFACT_ID: &str = "platform.telemetry";

#[test]
fn kernel_starts_a_signed_telemetry_child_with_fenced_contracts() {
    let fixture = SignedTelemetryLaunchFixture::new(false);
    let store = fixture.bind_release();
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));

    assert_eq!(fixture.start(&supervisor, &store), 1);
    assert!(wait_for_active(&supervisor));
    let launch = store
        .platform_managed_process_launch("telemetry")
        .expect("read telemetry launch")
        .expect("launch record");
    assert_eq!(launch.runtime_generation(), 1);
    let diagnostics = telemetry_diagnostics::read(&supervisor).expect("relay diagnostics");
    assert_eq!(diagnostics.segment_count(), 0);
    assert_eq!(diagnostics.total_bytes(), 0);
    supervisor.stop("telemetry").expect("stop telemetry");
    assert!(!shutdown.load(Ordering::Acquire));
}

#[test]
fn kernel_bounds_telemetry_crash_restarts_without_stopping_kernel() {
    let fixture = SignedTelemetryLaunchFixture::new(true);
    let store = fixture.bind_release();
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));

    assert_eq!(fixture.start(&supervisor, &store), 1);
    assert!(wait_for_inactive(&supervisor));
    assert_eq!(fixture.crash_attempts(), 3);
    assert!(!shutdown.load(Ordering::Acquire));
}

#[test]
fn telemetry_diagnostics_accepts_only_the_exact_sanitized_summary() {
    let diagnostics =
        telemetry_diagnostics::parse(&diagnostics_response(2, 4096)).expect("parse diagnostics");
    assert_eq!(diagnostics.segment_count(), 2);
    assert_eq!(diagnostics.total_bytes(), 4096);
    assert!(telemetry_diagnostics::parse(&[0x10, 0x01]).is_err());
    assert!(telemetry_diagnostics::parse(b"private log payload").is_err());
}

struct SignedTelemetryLaunchFixture {
    root: std::path::PathBuf,
    kernel: std::path::PathBuf,
    data_dir: std::path::PathBuf,
    runtime_dir: std::path::PathBuf,
    crash_marker: std::path::PathBuf,
}

impl SignedTelemetryLaunchFixture {
    fn new(crash_after_describe: bool) -> Self {
        let root = unique_target_root("makosh-telemetry-managed-launch");
        let kernel = root.join("Макошь.app/Contents/MacOS/makosh-kernel");
        let resources = root.join("Макошь.app/Contents/Resources/makosh-kernel-release");
        let distribution = resources.join("distribution");
        let crash_marker = root.join("crash-attempts");
        std::fs::create_dir_all(kernel.parent().expect("Kernel directory"))
            .expect("create Kernel directory");
        std::fs::create_dir_all(&distribution).expect("create distribution");
        std::fs::write(&kernel, b"kernel").expect("write Kernel placeholder");
        let contracts = TelemetryContracts::new();
        let artifact = write_child(
            &distribution,
            &contracts,
            &crash_marker,
            crash_after_describe,
        );
        write_release_resources(&resources, &distribution, &artifact, &contracts);
        Self {
            data_dir: root.join("data"),
            runtime_dir: root.join("runtime"),
            root,
            kernel,
            crash_marker,
        }
    }

    fn bind_release(&self) -> SqliteControlStore {
        let store = SqliteControlStore::create(&self.root.join("control.sqlite"), "instance-1", 1)
            .expect("create Control Store");
        telemetry_binding::bind_installed_release(&store, &self.kernel)
            .expect("bind signed Telemetry release");
        store
    }

    fn start(&self, supervisor: &ManagedRuntimeSupervisor, store: &SqliteControlStore) -> u64 {
        telemetry_launch::start_from_kernel(
            supervisor,
            store,
            &self.kernel,
            &self.data_dir,
            &self.runtime_dir,
        )
        .expect("start signed Telemetry child")
    }

    fn crash_attempts(&self) -> usize {
        std::fs::read(&self.crash_marker)
            .expect("read crash marker")
            .len()
    }
}

impl Drop for SignedTelemetryLaunchFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct TelemetryContracts {
    descriptor: Vec<u8>,
    schema: Vec<u8>,
}

impl TelemetryContracts {
    fn new() -> Self {
        let schema = SettingsSchemaV1 {
            major: 1,
            revision: 1,
            ..Default::default()
        }
        .encode_to_vec();
        let descriptor = ModuleDescriptorV1 {
            descriptor_major: 1,
            descriptor_revision: 1,
            module_id: "telemetry".to_owned(),
            owner_id: "telemetry".to_owned(),
            module_kind: ModuleKindV1::Platform as i32,
            module_version: "1".to_owned(),
            build_id: "telemetry-build".to_owned(),
            capabilities: vec![CapabilityDescriptorV1 {
                capability_id: "collect".to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                ..Default::default()
            }],
            settings_schema_ref: Some(SettingsSchemaRefV1 {
                major: 1,
                revision: 1,
                artifact_size_bytes: schema.len() as u64,
                sha256: Sha256::digest(&schema).to_vec(),
            }),
            ..Default::default()
        }
        .encode_to_vec();
        Self { descriptor, schema }
    }
}

struct ArtifactMaterial {
    relative_path: &'static str,
    bytes: Vec<u8>,
}

fn write_child(
    distribution: &std::path::Path,
    contracts: &TelemetryContracts,
    marker: &std::path::Path,
    crash_after_describe: bool,
) -> ArtifactMaterial {
    let request = managed_describe(contracts);
    let payload = framed(request);
    let ready = framed(managed_ready());
    let relay_request = framed(diagnostics_request());
    let relay_response = framed(diagnostics_response(0, 0));
    let script = child_script(
        &payload,
        &ready,
        &relay_request,
        &relay_response,
        describe_response_length(),
        marker,
        crash_after_describe,
    );
    let relative_path = "bin/telemetry-collector";
    let path = distribution.join(relative_path);
    std::fs::create_dir_all(path.parent().expect("artifact parent"))
        .expect("create artifact parent");
    std::fs::write(&path, &script).expect("write telemetry child");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("make telemetry child executable");
    ArtifactMaterial {
        relative_path,
        bytes: script,
    }
}

fn managed_describe(contracts: &TelemetryContracts) -> Vec<u8> {
    ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes: contracts.descriptor.clone(),
                    settings_schema_bytes: contracts.schema.clone(),
                },
            ),
        ),
    }
    .encode_to_vec()
}

fn managed_ready() -> Vec<u8> {
    ManagedRuntimeControlRequestV1 {
        operation: Some(ManagedOperation::Ready(ManagedRuntimeReadyRequestV1 {
            registration_id: "telemetry".to_owned(),
            runtime_generation: 1,
            grant_epoch: 1,
        })),
    }
    .encode_to_vec()
}

fn describe_response_length() -> usize {
    framed(
        ManagedRuntimeControlResponseV1 {
            result: Some(ManagedResult::Describe(DescribeManagedRuntimeResponseV1 {
                registration_id: "telemetry".to_owned(),
                runtime_generation: 1,
                grant_epoch: 1,
            })),
            error_code: String::new(),
        }
        .encode_to_vec(),
    )
    .len()
}

fn diagnostics_request() -> Vec<u8> {
    TelemetryRuntimeControlRequestV1 {
        operation: Some(TelemetryOperation::GetDiagnostics(
            GetTelemetryDiagnosticsRequestV1 {},
        )),
    }
    .encode_to_vec()
}

fn diagnostics_response(segment_count: u32, total_bytes: u64) -> Vec<u8> {
    TelemetryRuntimeControlResponseV1 {
        result: Some(TelemetryResult::Diagnostics(TelemetryDiagnosticsV1 {
            segment_count,
            total_bytes,
        })),
        error_code: String::new(),
    }
    .encode_to_vec()
}

fn framed(payload: Vec<u8>) -> Vec<u8> {
    assert!(payload.len() < 128, "fixture frame stays single-byte");
    [vec![payload.len() as u8], payload].concat()
}

fn child_script(
    describe: &[u8],
    ready: &[u8],
    relay_request: &[u8],
    relay_response: &[u8],
    describe_response_length: usize,
    marker: &std::path::Path,
    crash_after_describe: bool,
) -> Vec<u8> {
    let after_describe = if crash_after_describe {
        "sleep 1\nexit 1".to_owned()
    } else {
        format!(
            "dd bs=1 count={} of=/dev/null 2>/dev/null\nprintf '{}' >&0\nsleep 30",
            relay_request.len(),
            shell_binary_literal(relay_response),
        )
    };
    format!(
        "#!/bin/sh\nprintf x >> '{}'\nprintf '{}' >&0\ndd bs=1 count={describe_response_length} of=/dev/null 2>/dev/null\nprintf '{}' >&0\n{after_describe}\n",
        marker.display(),
        shell_binary_literal(describe),
        shell_binary_literal(ready),
    )
    .into_bytes()
}

fn write_release_resources(
    resources: &std::path::Path,
    distribution: &std::path::Path,
    artifact: &ArtifactMaterial,
    contracts: &TelemetryContracts,
) {
    let descriptor_path = distribution.join("contracts/telemetry-descriptor.pb");
    let schema_path = distribution.join("contracts/telemetry-settings.pb");
    std::fs::create_dir_all(descriptor_path.parent().expect("contract parent"))
        .expect("create contract parent");
    std::fs::write(&descriptor_path, &contracts.descriptor).expect("write descriptor");
    std::fs::write(&schema_path, &contracts.schema).expect("write schema");
    let manifest = telemetry_manifest(artifact, contracts);
    let (signed, root) = sign_manifest(&manifest);
    std::fs::write(
        resources.join("makosh-signed-distribution-manifest.pb"),
        signed,
    )
    .expect("write signed manifest");
    std::fs::write(resources.join("makosh-release-trust-root.pb"), root).expect("write trust root");
}

fn telemetry_manifest(
    artifact: &ArtifactMaterial,
    contracts: &TelemetryContracts,
) -> DistributionManifestV1 {
    DistributionManifestV1 {
        major: 1,
        revision: 1,
        distribution_id: "makosh-telemetry-test".to_owned(),
        release_version: "1.0.0".to_owned(),
        build_id: "telemetry-build".to_owned(),
        target_triple: TARGET.to_owned(),
        generation: 1,
        artifacts: vec![DistributionManifestArtifactV1 {
            artifact_kind: DistributionArtifactKindV1::ModuleRuntime as i32,
            artifact_id: ARTIFACT_ID.to_owned(),
            relative_path: artifact.relative_path.to_owned(),
            size_bytes: artifact.bytes.len() as u64,
            sha256: Sha256::digest(&artifact.bytes).to_vec(),
            descriptor_sha256: Sha256::digest(&contracts.descriptor).to_vec(),
            settings_schema_sha256: Sha256::digest(&contracts.schema).to_vec(),
            required: true,
            descriptor_relative_path: "contracts/telemetry-descriptor.pb".to_owned(),
            descriptor_size_bytes: contracts.descriptor.len() as u64,
            settings_schema_relative_path: "contracts/telemetry-settings.pb".to_owned(),
            settings_schema_size_bytes: contracts.schema.len() as u64,
            bound_module_id: String::new(),
        }],
    }
}

fn sign_manifest(manifest: &DistributionManifestV1) -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::from_bytes((&[29_u8; 32]).into()).expect("signing key");
    let raw_manifest_bytes = manifest.encode_to_vec();
    let signature: Signature = signing_key.sign(&raw_manifest_bytes);
    let signed = SignedDistributionManifestV1 {
        verification_key_id: "telemetry-test-key".to_owned(),
        raw_manifest_bytes,
        signature_raw: signature.to_bytes().to_vec(),
    };
    let public_key: [u8; 65] = signing_key
        .verifying_key()
        .to_sec1_point(false)
        .as_bytes()
        .try_into()
        .expect("P-256 public key");
    let root = ReleaseTrustRootV1 {
        major: 1,
        revision: 1,
        verification_keys: vec![ReleaseTrustRootKeyV1 {
            key_id: "telemetry-test-key".to_owned(),
            public_key_sec1: public_key.to_vec(),
        }],
    };
    (signed.encode_to_vec(), root.encode_to_vec())
}

fn wait_for_active(supervisor: &ManagedRuntimeSupervisor) -> bool {
    wait_for(supervisor, true)
}

fn wait_for_inactive(supervisor: &ManagedRuntimeSupervisor) -> bool {
    wait_for(supervisor, false)
}

fn wait_for(supervisor: &ManagedRuntimeSupervisor, expected: bool) -> bool {
    for _ in 0..500 {
        if supervisor
            .is_active("telemetry")
            .expect("read worker state")
            == expected
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

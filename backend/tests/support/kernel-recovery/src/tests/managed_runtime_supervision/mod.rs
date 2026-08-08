//! Process supervision tests for admitted managed runtimes.

use super::common::*;

#[test]
fn managed_runtime_supervisor_prevents_duplicates_and_stops_on_kernel_shutdown() {
    let (root, staged, duplicate, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    supervisor
        .start(
            "registration-1".to_owned(),
            staged,
            runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start managed runtime");
    assert!(
        supervisor
            .is_active("registration-1")
            .expect("active runtime")
    );
    let duplicate_policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    assert_eq!(
        supervisor
            .start(
                "registration-1".to_owned(),
                duplicate,
                runtime_expectation(2, descriptor_digest),
                duplicate_policy,
            )
            .expect_err("duplicate runtime"),
        "managed runtime is already active for this registration"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    assert!(
        !supervisor
            .is_active("registration-1")
            .expect("reaped runtime")
    );
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn managed_runtime_supervisor_relays_bounded_opaque_frames_after_admission() {
    let (root, staged, _, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    supervisor
        .start(
            "registration-1".to_owned(),
            staged,
            runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start managed runtime");
    assert_eq!(
        supervisor
            .relay("registration-1", b"opaque-request".to_vec())
            .expect("relay opaque frame"),
        b"opaque-response"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn managed_runtime_supervisor_passes_explicit_arguments_to_the_staged_child() {
    let (root, staged, _, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    let contracts =
        StagedRuntimeContracts::stage(&root.join("contracts"), b"exact-verified-contract", None)
            .expect("stage runtime contracts");
    let descriptor_path = contracts.descriptor_path().to_owned();
    supervisor
        .start_with_arguments_and_contracts(
            "registration-1".to_owned(),
            staged,
            vec!["serve-inherited".to_owned()],
            runtime_expectation(1, descriptor_digest),
            policy,
            contracts,
        )
        .expect("start managed runtime");
    assert_eq!(
        supervisor
            .relay("registration-1", b"opaque-request".to_vec())
            .expect("relay opaque frame"),
        b"opaque-response-with-arguments"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    assert!(!descriptor_path.exists());
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn managed_runtime_supervisor_stops_one_rebound_process_without_global_shutdown() {
    let (root, staged, _, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    supervisor
        .start(
            "vault".to_owned(),
            staged,
            runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start managed runtime");
    supervisor.stop("vault").expect("stop one runtime");
    assert!(!shutdown_requested.load(Ordering::Acquire));
    assert!(!supervisor.is_active("vault").expect("reap runtime"));
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn managed_runtime_supervisor_quiesce_is_idempotent_until_join() {
    let (root, staged, _, descriptor_digest) = prepare_supervisor_artifacts();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("execution policy");
    supervisor
        .start(
            "vault".to_owned(),
            staged,
            runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start managed runtime");

    assert!(
        supervisor
            .request_stop_if_active("vault")
            .expect("request first quiesce")
    );
    assert!(
        supervisor
            .request_stop_if_active("vault")
            .expect("request idempotent quiesce")
    );
    assert!(
        supervisor
            .stop_if_active("vault")
            .expect("join quiesced runtime")
    );
    assert!(!shutdown_requested.load(Ordering::Acquire));
    assert!(
        !supervisor
            .is_active("vault")
            .expect("quiesced runtime is reaped")
    );
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

#[test]
fn single_attempt_policy_does_not_reuse_a_crashed_runtime_identity() {
    let (root, staged, descriptor_digest, attempts) = prepare_crashing_supervisor_artifact();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(shutdown_requested);
    let policy =
        ManagedChildExecutionPolicy::new(1, Duration::from_secs(30)).expect("single attempt");
    supervisor
        .start_with_arguments(
            "scheduler".to_owned(),
            staged,
            vec![attempts.display().to_string()],
            scheduler_runtime_expectation(1, descriptor_digest),
            policy,
        )
        .expect("start crashing managed runtime");

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while supervisor
        .is_active("scheduler")
        .expect("reap crashed runtime")
    {
        assert!(
            std::time::Instant::now() < deadline,
            "crashed single-attempt runtime did not stop before its deadline"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(!supervisor.is_active("scheduler").expect("runtime stopped"));
    assert_eq!(
        std::fs::read_to_string(&attempts).expect("read child attempts"),
        "attempt\n"
    );
    assert!(
        supervisor
            .last_failure("scheduler")
            .expect("read failure")
            .is_some(),
        "the failed single attempt must remain observable"
    );
    std::fs::remove_dir_all(root).expect("remove supervisor fixture");
}

fn prepare_supervisor_artifacts() -> (
    std::path::PathBuf,
    staged_native_artifact::StagedNativeArtifact,
    staged_native_artifact::StagedNativeArtifact,
    [u8; 32],
) {
    let root = unique_target_root("makosh-managed-runtime-supervisor");
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "mail".into(),
        owner_id: "communications".into(),
        module_kind: ModuleKindV1::Integration as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let descriptor_digest = Sha256::digest(&descriptor_bytes).into();
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes: descriptor_bytes.clone(),
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let request_bytes = request.encode_to_vec();
    std::fs::create_dir_all(&root).expect("create fixture root");
    let source = root.join("managed-child.sh");
    let payload = [vec![request_bytes.len() as u8], request_bytes].concat();
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\ndd bs=1 count=15 of=/dev/null 2>/dev/null\nif [ \"$1\" = serve-inherited ]; then printf '\\036opaque-response-with-arguments' >&0; else printf '\\017opaque-response' >&0; fi\nsleep 30\n",
            shell_binary_literal(&payload)
        ),
    )
    .expect("write child script");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read child script")).into();
    let staged =
        staged_native_artifact::stage(&source, &root.join("launch"), "managed-child", &digest)
            .expect("stage child script");
    let duplicate = staged_native_artifact::stage(
        &source,
        &root.join("launch"),
        "managed-child-duplicate",
        &digest,
    )
    .expect("stage duplicate child script");
    (root, staged, duplicate, descriptor_digest)
}

fn prepare_crashing_supervisor_artifact() -> (
    std::path::PathBuf,
    staged_native_artifact::StagedNativeArtifact,
    [u8; 32],
    std::path::PathBuf,
) {
    let root = unique_target_root("makosh-managed-runtime-single-attempt");
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "scheduler".into(),
        owner_id: "scheduler".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "build".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let descriptor_digest = Sha256::digest(&descriptor_bytes).into();
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(
            makosh_runtime_protocol::v1::managed_runtime_control_request_v1::Operation::Describe(
                DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes: Vec::new(),
                },
            ),
        ),
    };
    let request_bytes = request.encode_to_vec();
    std::fs::create_dir_all(&root).expect("create fixture root");
    let attempts = root.join("attempts");
    let source = root.join("crashing-child.sh");
    let payload = [vec![request_bytes.len() as u8], request_bytes].concat();
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\nprintf 'attempt\\n' >> \"$1\"\nexit 1\n",
            shell_binary_literal(&payload),
        ),
    )
    .expect("write crashing child script");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read child script")).into();
    let staged =
        staged_native_artifact::stage(&source, &root.join("launch"), "crashing-child", &digest)
            .expect("stage crashing child script");
    (root, staged, descriptor_digest, attempts)
}

fn runtime_expectation(generation: u64, descriptor_digest: [u8; 32]) -> ManagedRuntimeExpectation {
    ManagedRuntimeExpectation::new(
        "registration-1",
        "runtime-1",
        "mail",
        generation,
        2,
        descriptor_digest,
        None,
    )
}

fn scheduler_runtime_expectation(
    generation: u64,
    descriptor_digest: [u8; 32],
) -> ManagedRuntimeExpectation {
    ManagedRuntimeExpectation::new(
        "registration-1",
        "runtime-1",
        "scheduler",
        generation,
        2,
        descriptor_digest,
        None,
    )
}

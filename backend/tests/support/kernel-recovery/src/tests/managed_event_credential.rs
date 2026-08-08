//! Private managed-runtime request framing for opaque Events credentials.

use crate::runtime::lifecycle::control::ManagedRuntimeEventCredentialHandler;
use makosh_runtime_protocol::v1::{
    ManagedRuntimeEventCredentialDeliveryV1, ManagedRuntimeEventCredentialRequestV1,
    managed_runtime_control_request_v1::Operation as RequestOperation,
    managed_runtime_control_response_v1::Result as ResponseResult,
};

use super::common::*;

#[test]
fn managed_runtime_request_exposes_only_public_fences_and_receives_opaque_delivery() {
    let (mut kernel, mut child) = UnixStream::pair().expect("private control channel");
    let worker = std::thread::spawn(move || {
        write_test_frame(&mut child, &request_frame());
        let response =
            ManagedRuntimeControlResponseV1::decode(read_test_frame(&mut child).as_slice())
                .expect("credential response");
        assert!(
            matches!(response.result, Some(ResponseResult::EventCredentialDelivery(value))
            if response.error_code.is_empty() && value.ciphertext == vec![2; 32])
        );
    });

    let request = receive_request(&mut kernel);
    assert_eq!(request.request_id, vec![1; 16]);
    assert_eq!(request.credential_revision, 2);
    assert_eq!(request.ttl_seconds, 60);
    assert_eq!(request.recipient_public_key_x25519, vec![3; 32]);
    managed_runtime_control::inbound::respond_event_credential(
        &mut kernel,
        Ok(ManagedRuntimeEventCredentialDeliveryV1 {
            encapped_key: vec![1; 32],
            ciphertext: vec![2; 32],
            tag: vec![3; 16],
            consumer_bindings: Vec::new(),
            publish_subjects: Vec::new(),
        }),
    )
    .expect("opaque response");
    worker.join().expect("runtime worker");
}

#[test]
fn managed_runtime_supervisor_dispatches_event_credential_after_descriptor_handshake() {
    let (root, staged, expectation) = credential_child_fixture();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicU64::new(0));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown_requested));
    supervisor
        .configure_event_credential_handler(Arc::new(RecordingCredentialHandler {
            calls: Arc::clone(&calls),
        }))
        .expect("configure event credential handler");
    supervisor
        .start(
            "registration-events".to_owned(),
            staged,
            expectation,
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                .expect("managed execution policy"),
        )
        .expect("start managed runtime");

    assert!(
        wait_for_handler(&calls),
        "request reaches the Kernel handler"
    );
    shutdown_requested.store(true, Ordering::Release);
    supervisor.shutdown().expect("stop managed runtime");
    std::fs::remove_dir_all(root).expect("remove event credential fixture");
}

#[test]
fn managed_runtime_supervisor_stops_an_active_registration_idempotently() {
    let (root, staged, expectation) = credential_child_fixture();
    let calls = Arc::new(AtomicU64::new(0));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));
    supervisor
        .configure_event_credential_handler(Arc::new(RecordingCredentialHandler {
            calls: Arc::clone(&calls),
        }))
        .expect("configure event credential handler");
    supervisor
        .start(
            "registration-events".to_owned(),
            staged,
            expectation,
            ManagedChildExecutionPolicy::new(1, Duration::from_secs(30))
                .expect("managed execution policy"),
        )
        .expect("start managed runtime");
    assert!(wait_for_handler(&calls), "managed runtime becomes active");

    assert!(
        supervisor
            .stop_if_active("registration-events")
            .expect("first stop"),
        "the active registration was stopped"
    );
    assert!(
        !supervisor
            .stop_if_active("registration-events")
            .expect("idempotent stop"),
        "an already inactive registration remains inactive"
    );
    assert!(
        !supervisor
            .is_active("registration-events")
            .expect("inactive state")
    );
    std::fs::remove_dir_all(root).expect("remove event credential fixture");
}

struct RecordingCredentialHandler {
    calls: Arc<AtomicU64>,
}

impl ManagedRuntimeEventCredentialHandler for RecordingCredentialHandler {
    fn issue_event_credential(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeEventCredentialRequestV1,
    ) -> Result<ManagedRuntimeEventCredentialDeliveryV1, String> {
        if expectation.registration_id() != "registration-events"
            || expectation.runtime_instance_id() != "runtime-events"
            || expectation.runtime_generation() != 5
            || expectation.grant_epoch() != 7
            || request.request_id != vec![1; 16]
            || request.recipient_public_key_x25519 != vec![3; 32]
        {
            return Err("event credential fence mismatch".to_owned());
        }
        self.calls.fetch_add(1, Ordering::Release);
        Ok(ManagedRuntimeEventCredentialDeliveryV1 {
            encapped_key: vec![1; 32],
            ciphertext: vec![2; 32],
            tag: vec![3; 16],
            consumer_bindings: Vec::new(),
            publish_subjects: Vec::new(),
        })
    }
}

fn request_frame() -> Vec<u8> {
    ManagedRuntimeControlRequestV1 {
        operation: Some(RequestOperation::IssueEventCredential(
            ManagedRuntimeEventCredentialRequestV1 {
                request_id: vec![1; 16],
                credential_revision: 2,
                ttl_seconds: 60,
                recipient_public_key_x25519: vec![3; 32],
            },
        )),
    }
    .encode_to_vec()
}

fn receive_request(channel: &mut UnixStream) -> ManagedRuntimeEventCredentialRequestV1 {
    for _ in 0..40 {
        match managed_runtime_control::inbound::try_receive_event_credential(channel) {
            Ok(Some(request)) => return request,
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(error) => panic!("credential request: {error}"),
        }
    }
    panic!("credential request did not arrive")
}

fn credential_child_fixture() -> (
    std::path::PathBuf,
    staged_native_artifact::StagedNativeArtifact,
    ManagedRuntimeExpectation,
) {
    let root = unique_target_root("makosh-managed-event-credential");
    let descriptor = ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "events".into(),
        owner_id: "platform".into(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".into(),
        build_id: "test".into(),
        ..Default::default()
    };
    let descriptor_bytes = descriptor.encode_to_vec();
    let expectation = ManagedRuntimeExpectation::new(
        "registration-events",
        "runtime-events",
        "events",
        5,
        7,
        Sha256::digest(&descriptor_bytes).into(),
        None,
    );
    let payload = child_payload(descriptor_bytes);
    let source = root.join("managed-event-child.sh");
    std::fs::create_dir_all(&root).expect("create fixture directory");
    std::fs::write(
        &source,
        format!(
            "#!/bin/sh\nprintf '{}' >&0\nsleep 30\n",
            shell_binary_literal(&payload)
        ),
    )
    .expect("write event credential child");
    let digest: [u8; 32] =
        Sha256::digest(std::fs::read(&source).expect("read event credential child")).into();
    let staged =
        staged_native_artifact::stage(&source, &root.join("launch"), "event-child", &digest)
            .expect("stage event credential child");
    (root, staged, expectation)
}

fn child_payload(descriptor_bytes: Vec<u8>) -> Vec<u8> {
    let describe = ManagedRuntimeControlRequestV1 {
        operation: Some(RequestOperation::Describe(
            DescribeManagedRuntimeRequestV1 {
                descriptor_bytes,
                settings_schema_bytes: Vec::new(),
            },
        )),
    };
    let request = ManagedRuntimeControlRequestV1 {
        operation: Some(RequestOperation::IssueEventCredential(
            ManagedRuntimeEventCredentialRequestV1 {
                request_id: vec![1; 16],
                credential_revision: 2,
                ttl_seconds: 60,
                recipient_public_key_x25519: vec![3; 32],
            },
        )),
    };
    [
        frame(&describe.encode_to_vec()),
        frame(&request.encode_to_vec()),
    ]
    .concat()
}

fn frame(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() < 128, "test frame stays single-byte length");
    [vec![bytes.len() as u8], bytes.to_vec()].concat()
}

fn wait_for_handler(calls: &AtomicU64) -> bool {
    for _ in 0..40 {
        if calls.load(Ordering::Acquire) == 1 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

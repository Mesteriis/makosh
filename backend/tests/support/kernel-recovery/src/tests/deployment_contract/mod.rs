//! Deployment profile and initial-enrollment contract validation.

use super::common::*;
use makosh_runtime_protocol::v1::{
    DeploymentProfileV1, DeviceProofV1, DistributionArtifactV1, FileDeviceProofV1,
    InitialOwnerEnrollmentTransportV1, NativeExecutableDigestV1, OciImageDigestV1,
    RemotePairingEnrollmentV1, RuntimeLifecycleV1, device_proof_v1, distribution_artifact_v1,
    initial_owner_enrollment_transport_v1,
};
use makosh_runtime_protocol::validation::deployment::{
    DeploymentValidationError, MACOS_TAURI_TARGET, validate_deployment_binding,
    validate_initial_owner_enrollment_transport,
};

#[test]
fn initial_enrollment_contract_is_fixed_to_p256_and_a_single_pristine_generation() {
    let challenge = InitialOwnerEnrollmentChallengeV1 {
        protocol_major: 1,
        instance_id: vec![1; 32],
        nonce: vec![2; 32],
        kernel_generation: 1,
    };
    let proof = InitialOwnerEnrollmentV1 {
        protocol_major: 1,
        device_public_key_sec1: [vec![0x04], vec![3; 64]].concat(),
        challenge_signature_raw: vec![4; 64],
        owner_id: "owner-1".to_owned(),
        device_id: "device-1".to_owned(),
    };
    assert!(validate_initial_owner_enrollment(&challenge, &proof));
    assert!(!validate_initial_owner_enrollment(
        &InitialOwnerEnrollmentChallengeV1 {
            kernel_generation: 2,
            ..challenge
        },
        &proof
    ));
}

#[test]
fn deployment_contracts_bind_profile_lifecycle_artifact_and_file_proof() {
    let proof = DeviceProofV1 {
        proof: Some(device_proof_v1::Proof::FileEs256(FileDeviceProofV1 {
            public_key_sec1: [vec![0x04], vec![7; 64]].concat(),
            signature_raw: vec![8; 64],
        })),
    };
    let artifact = DistributionArtifactV1 {
        artifact: Some(distribution_artifact_v1::Artifact::NativeExecutable(
            NativeExecutableDigestV1 {
                target_triple: MACOS_TAURI_TARGET.to_owned(),
                sha256: vec![9; 32],
            },
        )),
    };
    assert!(
        validate_deployment_binding(
            DeploymentProfileV1::MacosTauriEmbedded as i32,
            RuntimeLifecycleV1::ManagedChild as i32,
            &artifact,
        )
        .is_ok()
    );
    assert_eq!(
        validate_deployment_binding(
            DeploymentProfileV1::MacosTauriEmbedded as i32,
            RuntimeLifecycleV1::ExternalCompose as i32,
            &artifact,
        ),
        Err(DeploymentValidationError::InvalidLifecycle)
    );
    let oci_artifact = DistributionArtifactV1 {
        artifact: Some(distribution_artifact_v1::Artifact::OciImage(
            OciImageDigestV1 {
                repository: "registry.example/makosh/kernel".to_owned(),
                sha256: vec![12; 32],
            },
        )),
    };
    assert!(
        validate_deployment_binding(
            DeploymentProfileV1::LinuxDockerServer as i32,
            RuntimeLifecycleV1::ExternalCompose as i32,
            &oci_artifact,
        )
        .is_ok()
    );
    assert_remote_pairing_transport(proof);
}

fn assert_remote_pairing_transport(proof: DeviceProofV1) {
    let remote = InitialOwnerEnrollmentTransportV1 {
        transport: Some(
            initial_owner_enrollment_transport_v1::Transport::RemotePairing(
                RemotePairingEnrollmentV1 {
                    endpoint: "https://127.0.0.1:9443".to_owned(),
                    tls_certificate_sha256: vec![10; 32],
                    one_time_token: vec![11; 32],
                    device_proof: Some(proof),
                },
            ),
        ),
    };
    assert!(validate_initial_owner_enrollment_transport(&remote).is_ok());
    let invalid_proof = InitialOwnerEnrollmentTransportV1 {
        transport: Some(
            initial_owner_enrollment_transport_v1::Transport::RemotePairing(
                RemotePairingEnrollmentV1 {
                    endpoint: "https://127.0.0.1:9443".to_owned(),
                    tls_certificate_sha256: vec![10; 32],
                    one_time_token: vec![11; 32],
                    device_proof: Some(DeviceProofV1::default()),
                },
            ),
        ),
    };
    assert_eq!(
        validate_initial_owner_enrollment_transport(&invalid_proof),
        Err(DeploymentValidationError::InvalidDeviceProof)
    );
}

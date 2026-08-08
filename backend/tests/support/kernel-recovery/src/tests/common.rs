pub(super) use crate::distribution::bundle_verifier as distribution_bundle_verifier;
pub(super) use crate::distribution::bundled_launch as bundled_managed_launch_binding;
pub(super) use crate::distribution::manifest_verifier as distribution_manifest_verifier;
pub(super) use crate::distribution::staged_artifact as staged_native_artifact;
pub(super) use crate::distribution::staged_contracts::StagedRuntimeContracts;
pub(super) use crate::distribution::trust_root::ReleaseTrustRoot;
pub(super) use crate::platform::macos::code_signature as macos_code_signature;
pub(super) use crate::platform::macos::release_resources as macos_release_resources;
pub(super) use crate::platform::storage::{binding as storage_binding, launch as storage_launch};
pub(super) use crate::platform::telemetry::{
    binding as telemetry_binding, diagnostics as telemetry_diagnostics, launch as telemetry_launch,
};
pub(super) use crate::platform::vault::ciphertext_route as vault_ciphertext_route;
pub(super) use crate::runtime::external::sessions::ExternalRuntimeSessions;
pub(super) use crate::runtime::lifecycle::control::{
    self as managed_runtime_control, ManagedRuntimeExpectation,
};
pub(super) use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;
pub(super) use crate::runtime::managed::execution::{
    self as bounded_managed_child_execution, ManagedChildExecutionPolicy,
};
pub(super) use crate::runtime::managed::supervisor as managed_child_supervisor;
pub(super) use makosh_events_protocol::{
    envelope::decode_envelope_v1,
    v1::{ActorKindV1, ActorRefV1, ContractRefV1, DurableEnvelopeV1, EventMetadataV1, SourceRefV1},
};
pub(super) use makosh_gateway_protocol::v1::{
    GetRecoveryStatusRequestV1, RecoveryControlRequestV1,
};
pub(super) use makosh_kernel_control_store::{
    BundledManagedLaunchBinding, ControlStore, ExternalRuntimeAttestation, ExternalRuntimeIdentity,
    InitialOwnerIdentity, ManagedLaunchRecord, ModuleRegistration, ModuleRegistrationState,
    PlatformManagedProcessBinding, PlatformManagedProcessLaunch, RecoveryFences,
    ServerBootstrapPairing, SettingsApplyState, SettingsDesiredSnapshot, SettingsInitialSnapshot,
    SettingsSchemaBinding, StoreHealth,
};
pub(super) use makosh_kernel_control_store_sqlite::{
    SqliteControlStore, StagedControlStoreRestore, StoreError,
};
pub(super) use makosh_runtime_protocol::v1::KernelStateV1;
pub(super) use makosh_runtime_protocol::v1::{
    DescribeManagedRuntimeRequestV1, ManagedRuntimeControlRequestV1,
    ManagedRuntimeControlResponseV1,
};
pub(super) use makosh_runtime_protocol::{
    v1::{
        CapabilityCriticalityV1, CapabilityDescriptorV1, DistributionArtifactKindV1,
        DistributionManifestArtifactV1, DistributionManifestV1, InitialOwnerEnrollmentChallengeV1,
        InitialOwnerEnrollmentV1, ModuleDescriptorV1, ModuleKindV1, ReleaseTrustRootKeyV1,
        ReleaseTrustRootV1, SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1,
        SettingMutationAuthorityV1, SettingTargetScopeV1, SettingValueTypeV1, SettingValueV1,
        SettingsSchemaRefV1, SettingsSchemaV1, SettingsSnapshotV1, SettingsValueEntryV1,
        SignedDistributionManifestV1, VaultActionV1, VaultCiphertextRouteDirectionV1,
        VaultCiphertextRouteV1, VaultSecretClassV1, VaultTargetScopeV1,
    },
    validation::descriptor::{
        decode_descriptor_v1, decode_settings_schema_v1, decode_settings_snapshot_v1,
        validate_initial_owner_enrollment, validate_settings_snapshot_against_schema_v1,
    },
    validation::distribution::{
        decode_distribution_manifest_v1, decode_signed_distribution_manifest_v1,
    },
};
pub(super) use p256::ecdsa::signature::Signer;
pub(super) use p256::ecdsa::{Signature, SigningKey};
pub(super) use prost::Message;
pub(super) use sha2::{Digest, Sha256};
pub(super) use std::io::{Read, Write};
pub(super) use std::os::unix::fs::PermissionsExt;
pub(super) use std::os::unix::net::UnixStream;
pub(super) use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
pub(super) use std::time::Duration;

static UNIQUE_TARGET_SUFFIX: AtomicU64 = AtomicU64::new(0);

pub(super) fn unique_target_root(prefix: &str) -> std::path::PathBuf {
    let suffix = UNIQUE_TARGET_SUFFIX.fetch_add(1, Ordering::Relaxed);
    std::env::current_dir()
        .expect("test working directory")
        .join("target")
        .join(format!(
            "{prefix}-{}-{}-{suffix}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
}

pub(super) fn release_trust_root(keys: &[(&str, [u8; 65])]) -> ReleaseTrustRoot {
    let root = ReleaseTrustRootV1 {
        major: 1,
        revision: 1,
        verification_keys: keys
            .iter()
            .map(|(key_id, public_key_sec1)| ReleaseTrustRootKeyV1 {
                key_id: (*key_id).to_owned(),
                public_key_sec1: public_key_sec1.to_vec(),
            })
            .collect(),
    };
    ReleaseTrustRoot::decode(&root.encode_to_vec()).expect("release trust root")
}

pub(super) fn write_test_frame(stream: &mut UnixStream, bytes: &[u8]) {
    assert!(bytes.len() < 128, "test frame stays single-byte length");
    stream
        .write_all(&[bytes.len() as u8])
        .expect("write frame length");
    stream.write_all(bytes).expect("write frame bytes");
    stream.flush().expect("flush frame");
}

pub(super) fn read_test_frame(stream: &mut UnixStream) -> Vec<u8> {
    let mut length = [0_u8; 1];
    stream.read_exact(&mut length).expect("read frame length");
    let mut bytes = vec![0_u8; usize::from(length[0])];
    stream.read_exact(&mut bytes).expect("read frame bytes");
    bytes
}

pub(super) fn shell_binary_literal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("\\{byte:03o}")).collect()
}

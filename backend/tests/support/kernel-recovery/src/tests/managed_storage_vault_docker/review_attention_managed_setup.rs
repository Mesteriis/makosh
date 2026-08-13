//! Exact managed Review domain admission and lifecycle for live conformance.

use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_review_attention_api::{REVIEW_ATTENTION_MODULE_ID_V1, REVIEW_ATTENTION_OWNER_V1};
use makosh_review_attention_persistence::schema::{
    REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V3, review_attention_storage_bundle_v1,
};
use makosh_review_attention_runtime::{
    REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1, review_attention_module_descriptor_v1,
    review_attention_settings_schema_bytes_v1,
};

use crate::platform::client_realtime::ClientRealtimePublishHandlerV1;

const REVIEW_ATTENTION_RELEASE_ARTIFACT_ID_V1: &str = "review_attention.runtime.v1";
pub(super) const REVIEW_ATTENTION_LOGICAL_OWNER_ID_V1: &str = "owner-1";

pub(super) struct AdmittedReviewAttentionRuntimeV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedReviewAttentionRuntimeV1 {
    pub(super) registration_id: String,
    pub(super) runtime_instance_id: String,
    pub(super) runtime_generation: u64,
    capability_ids: Vec<String>,
}

pub(super) fn installed_review_attention_release_v1(root: &Path) -> InstalledSignedBundle {
    InstalledSignedBundle::install(
        root,
        &[
            SignedRuntimeArtifact::new(
                "platform.storage",
                storage_binary(),
                descriptor("storage").encode_to_vec(),
            ),
            SignedRuntimeArtifact::new(
                "platform.vault",
                vault_binary(),
                descriptor("vault").encode_to_vec(),
            ),
            SignedRuntimeArtifact::new(
                REVIEW_ATTENTION_RELEASE_ARTIFACT_ID_V1,
                review_attention_binary(),
                review_attention_module_descriptor_v1("managed-review-attention-live")
                    .encode_to_vec(),
            )
            .with_settings_schema(review_attention_settings_schema_bytes_v1()),
        ],
    )
    .expect("install signed Review attention release")
}

pub(super) fn admit_review_attention_runtime_v1(
    store: &SqliteControlStore,
) -> AdmittedReviewAttentionRuntimeV1 {
    let descriptor = review_attention_module_descriptor_v1("managed-review-attention-live");
    assert_eq!(descriptor.module_id, REVIEW_ATTENTION_MODULE_ID_V1);
    assert_eq!(descriptor.owner_id, REVIEW_ATTENTION_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register exact Review attention descriptor");
    let capability_ids = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capability_ids,
    )
    .expect("approve exact Review attention capabilities");
    let settings = review_attention_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            REVIEW_ATTENTION_RELEASE_ARTIFACT_ID_V1,
            Sha256::digest(
                std::fs::read(review_attention_binary()).expect("Review attention runtime binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Review attention release binding");
    let bundle = review_attention_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                REVIEW_ATTENTION_OWNER_V1,
                u64::from(REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V3),
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Review attention Storage bundle"),
        )
        .expect("persist Review attention Storage bundle");
    AdmittedReviewAttentionRuntimeV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids,
    }
}

pub(super) fn prepare_review_attention_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedReviewAttentionRuntimeV1,
) -> AdmittedReviewAttentionRuntimeV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Review attention launch");
    let bundle = store
        .platform_storage_bundle(
            REVIEW_ATTENTION_OWNER_V1,
            u64::from(REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V3),
        )
        .expect("read Review attention Storage bundle")
        .expect("Review attention Storage bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(
            1,
            1,
            u64::from(REVIEW_ATTENTION_STORAGE_BUNDLE_REVISION_V3),
            *bundle.digest(),
        )
        .expect("Review attention Storage binding issue"),
    )
    .expect("issue Review attention Storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Review attention Storage binding");
    admitted
}

pub(super) fn configure_review_attention_realtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &Arc<SqliteControlStore>,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) {
    supervisor
        .configure_client_realtime_handler(Arc::new(ClientRealtimePublishHandlerV1::new(
            Arc::clone(store),
            realtime,
        )))
        .expect("configure Review attention client realtime handler");
}

pub(super) fn start_review_attention_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedReviewAttentionRuntimeV1,
) -> StartedReviewAttentionRuntimeV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Review attention launch reservation");
    start_reserved_review_attention_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        admitted,
    )
}

pub(super) fn restart_review_attention_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    predecessor: StartedReviewAttentionRuntimeV1,
) -> StartedReviewAttentionRuntimeV1 {
    let previous_generation = predecessor.runtime_generation;
    let previous_instance = predecessor.runtime_instance_id.clone();
    let binding = review_attention_storage_binding_v1(store, &predecessor.registration_id);
    let issue =
        storage_successor::issue_after(&binding).expect("derive Review attention successor");
    let (reservation, binding) = storage_successor::reserve(
        supervisor,
        store,
        &predecessor.registration_id,
        REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1,
        issue,
    )
    .expect("reserve Review attention successor");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Review attention successor");
    let successor = start_reserved_review_attention_runtime_v1(
        supervisor,
        store,
        runtime_dir,
        reservation,
        AdmittedReviewAttentionRuntimeV1 {
            registration_id: predecessor.registration_id,
            capability_ids: predecessor.capability_ids,
        },
    );
    assert_eq!(successor.runtime_generation, previous_generation + 1);
    assert_ne!(successor.runtime_instance_id, previous_instance);
    successor
}

fn start_reserved_review_attention_runtime_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    reservation: managed_launch::ManagedLaunchReservation,
    admitted: AdmittedReviewAttentionRuntimeV1,
) -> StartedReviewAttentionRuntimeV1 {
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    let binding = review_attention_storage_binding_v1(store, &admitted.registration_id);
    let topology =
        crate::platform::storage::topology::current(store).expect("Review Storage topology");
    let vault =
        vault_status::read_current(store, &supervisor.relay_port()).expect("live Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Review attention Storage configuration");
    managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: REVIEW_ATTENTION_OWNER_V1.to_owned(),
            registration_id: admitted.registration_id.clone(),
            runtime_instance_id: runtime_instance_id.clone(),
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: String::new(),
            event_credential_revision: 0,
            logical_human_owner_id: REVIEW_ATTENTION_LOGICAL_OWNER_ID_V1.to_owned(),
        },
    )
    .expect("start eventless Review attention domain");
    supervisor
        .wait_until_ready(&admitted.registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Review attention readiness: {error}; last_failure={:?}",
                supervisor.last_failure(&admitted.registration_id)
            )
        });
    StartedReviewAttentionRuntimeV1 {
        registration_id: admitted.registration_id,
        runtime_instance_id,
        runtime_generation,
        capability_ids: admitted.capability_ids,
    }
}

fn review_attention_storage_binding_v1(
    store: &SqliteControlStore,
    registration_id: &str,
) -> makosh_kernel_control_store::PlatformStorageBindingV1 {
    store
        .platform_storage_binding(registration_id, REVIEW_ATTENTION_STORAGE_CAPABILITY_ID_V1)
        .expect("read Review attention Storage binding")
        .filter(|binding| binding.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Review attention Storage binding")
}

fn review_attention_binary() -> PathBuf {
    binary("MAKOSH_REVIEW_ATTENTION_RUNTIME_BIN")
}

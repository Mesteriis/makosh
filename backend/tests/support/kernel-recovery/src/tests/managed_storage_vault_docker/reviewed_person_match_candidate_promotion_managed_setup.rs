use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_reviewed_person_match_candidate_promotion_persistence::reviewed_person_match_candidate_promotion_storage_bundle_v1;
use makosh_reviewed_person_match_candidate_promotion_runtime::{
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
    reviewed_person_match_candidate_promotion_module_descriptor_v1,
    reviewed_person_match_candidate_promotion_settings_schema_bytes_v1,
};

const PROMOTION_ARTIFACT_ID: &str = "reviewed.person-match-candidate-promotion.runtime.v1";

pub(super) struct AdmittedReviewedPersonMatchPromotionV1 {
    registration_id: String,
    capability_ids: Vec<String>,
}

pub(super) struct StartedReviewedPersonMatchPromotionV1 {
    pub(super) registration_id: String,
}

pub(super) enum ReviewedPersonMatchPromotionBootstrapOverrideV1 {
    None,
    StaleCredentialFence,
    UnavailableEventEndpoint(String),
}

pub(super) fn reviewed_person_match_promotion_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        PROMOTION_ARTIFACT_ID,
        reviewed_person_match_promotion_binary(),
        reviewed_person_match_candidate_promotion_module_descriptor_v1(
            "managed-review-pm-promotion-live",
        )
        .encode_to_vec(),
    )
    .with_settings_schema(reviewed_person_match_candidate_promotion_settings_schema_bytes_v1())
}

pub(super) fn installed_reviewed_person_match_promotion_release_v1(
    root: &Path,
) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(reviewed_person_match_promotion_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts)
        .expect("install signed reviewed Person match promotion release")
}

pub(super) fn admit_reviewed_person_match_promotion_v1(
    store: &SqliteControlStore,
) -> AdmittedReviewedPersonMatchPromotionV1 {
    let descriptor = reviewed_person_match_candidate_promotion_module_descriptor_v1(
        "managed-review-pm-promotion-live",
    );
    assert_eq!(
        descriptor.module_id,
        REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_MODULE_ID_V1
    );
    assert_eq!(
        descriptor.owner_id,
        REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1
    );
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register reviewed Person match promotion descriptor");
    let capabilities = descriptor
        .capabilities
        .iter()
        .map(|capability| capability.capability_id.clone())
        .collect::<Vec<_>>();
    crate::modules::registration::registry::approve_after_owner_authorization(
        store,
        registration.registration_id(),
        &capabilities,
    )
    .expect("approve reviewed Person match promotion capabilities");
    let settings = reviewed_person_match_candidate_promotion_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            PROMOTION_ARTIFACT_ID,
            Sha256::digest(
                std::fs::read(reviewed_person_match_promotion_binary()).expect("promotion binary"),
            )
            .into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record promotion release binding");
    let bundle = reviewed_person_match_candidate_promotion_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1,
                1,
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("promotion storage bundle"),
        )
        .expect("persist promotion storage bundle");
    AdmittedReviewedPersonMatchPromotionV1 {
        registration_id: registration.registration_id().to_owned(),
        capability_ids: capabilities,
    }
}

pub(super) fn prepare_reviewed_person_match_promotion_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedReviewedPersonMatchPromotionV1,
) -> AdmittedReviewedPersonMatchPromotionV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve reviewed Person match promotion launch");
    let bundle = store
        .platform_storage_bundle(REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_OWNER_V1, 1)
        .expect("read promotion bundle")
        .expect("promotion bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(1, 1, 1, *bundle.digest()).expect("promotion storage issue"),
    )
    .expect("issue promotion storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision promotion storage");
    admitted
}

pub(super) fn start_reviewed_person_match_promotion_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedReviewedPersonMatchPromotionV1,
) -> StartedReviewedPersonMatchPromotionV1 {
    launch_reviewed_person_match_promotion_v1(
        supervisor,
        store,
        runtime_dir,
        admitted,
        ReviewedPersonMatchPromotionBootstrapOverrideV1::None,
        true,
    )
}

pub(super) fn launch_reviewed_person_match_promotion_without_ready_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedReviewedPersonMatchPromotionV1,
    bootstrap_override: ReviewedPersonMatchPromotionBootstrapOverrideV1,
) -> StartedReviewedPersonMatchPromotionV1 {
    launch_reviewed_person_match_promotion_v1(
        supervisor,
        store,
        runtime_dir,
        admitted,
        bootstrap_override,
        false,
    )
}

fn launch_reviewed_person_match_promotion_v1(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedReviewedPersonMatchPromotionV1,
    bootstrap_override: ReviewedPersonMatchPromotionBootstrapOverrideV1,
    wait_until_ready: bool,
) -> StartedReviewedPersonMatchPromotionV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load promotion reservation");
    let binding = store
        .platform_storage_binding(
            &admitted.registration_id,
            REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read promotion binding")
        .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
        .expect("active promotion binding");
    let topology = crate::platform::storage::topology::current(store).expect("Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port()).expect("Vault status");
    let mut storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("promotion storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("Event topology")
        .expect("Event topology");
    let mut event_hub_endpoint = events.nats_endpoint().to_owned();
    match bootstrap_override {
        ReviewedPersonMatchPromotionBootstrapOverrideV1::None => {}
        ReviewedPersonMatchPromotionBootstrapOverrideV1::StaleCredentialFence => {
            storage.credential_revision = storage.credential_revision.saturating_add(1);
        }
        ReviewedPersonMatchPromotionBootstrapOverrideV1::UnavailableEventEndpoint(endpoint) => {
            event_hub_endpoint = endpoint;
        }
    }
    let registration_id = admitted.registration_id;
    let capability_ids = admitted.capability_ids;
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    managed_launch::start_reserved_workflow(
        supervisor,
        runtime_dir,
        reservation,
        ManagedWorkflowRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: REVIEW_PM_HUMAN_OWNER.to_owned(),
            registration_id: registration_id.clone(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint,
            event_credential_revision: events.credential_revision(),
            runtime_artifacts: Vec::new(),
            configuration_instance_id: String::new(),
            settings_revision: 0,
            configuration_instances: Vec::new(),
        },
        &capability_ids,
    )
    .expect("start reviewed Person match promotion");
    if wait_until_ready {
        supervisor
            .wait_until_ready(&registration_id)
            .unwrap_or_else(|error| {
                panic!(
                    "promotion readiness: {error}; last={:?}",
                    supervisor.last_failure(&registration_id)
                )
            });
    }
    StartedReviewedPersonMatchPromotionV1 { registration_id }
}

fn reviewed_person_match_promotion_binary() -> PathBuf {
    binary("MAKOSH_REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_BIN")
}

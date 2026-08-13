use super::*;

use makosh_kernel_control_store::PlatformStorageBindingStateV1;
use makosh_review_person_match_candidate_api::{
    REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1, REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1,
};
use makosh_review_person_match_candidate_persistence::review_person_match_candidate_storage_bundle_v1;
use makosh_review_person_match_candidate_runtime::{
    REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
    review_person_match_candidate_module_descriptor_v1,
    review_person_match_candidate_settings_schema_bytes_v1,
};

const ARTIFACT_ID: &str = "review.person-match-candidate.runtime.v1";
pub(super) const REVIEW_PM_HUMAN_OWNER: &str = "owner-1";

pub(super) struct AdmittedReviewPmV1 {
    registration_id: String,
}

pub(super) struct StartedReviewPmV1 {
    pub(super) registration_id: String,
}

pub(super) fn installed_review_pm_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(review_pm_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts).expect("install signed Review PM release")
}

pub(super) fn installed_review_pm_e2e_release(root: &Path) -> InstalledSignedBundle {
    let mut artifacts = communications_release_artifacts();
    artifacts.push(review_pm_release_artifact_v1());
    artifacts.push(reviewed_person_match_promotion_release_artifact_v1());
    artifacts.push(persons_release_artifact_v1());
    artifacts.push(identity_resolution_release_artifact_v1());
    InstalledSignedBundle::install(root, &artifacts).expect("install signed Review PM E2E release")
}

fn review_pm_release_artifact_v1() -> SignedRuntimeArtifact {
    SignedRuntimeArtifact::new(
        ARTIFACT_ID,
        review_pm_binary(),
        review_person_match_candidate_module_descriptor_v1("managed-review-pm-live")
            .encode_to_vec(),
    )
    .with_settings_schema(review_person_match_candidate_settings_schema_bytes_v1())
}

pub(super) fn admit_review_pm(store: &SqliteControlStore) -> AdmittedReviewPmV1 {
    let descriptor = review_person_match_candidate_module_descriptor_v1("managed-review-pm-live");
    assert_eq!(
        descriptor.module_id,
        REVIEW_PERSON_MATCH_CANDIDATE_MODULE_ID_V1
    );
    assert_eq!(descriptor.owner_id, REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1);
    let descriptor_bytes = descriptor.encode_to_vec();
    let registration = crate::modules::registration::registry::register(store, &descriptor_bytes)
        .expect("register Review PM descriptor");
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
    .expect("approve Review PM capabilities");
    let settings = review_person_match_candidate_settings_schema_bytes_v1();
    store
        .record_bundled_managed_launch_binding(&BundledManagedLaunchBinding::new(
            registration.registration_id(),
            1,
            "makosh-managed-runtime-conformance",
            ARTIFACT_ID,
            Sha256::digest(std::fs::read(review_pm_binary()).expect("Review PM binary")).into(),
            Sha256::digest(&descriptor_bytes).into(),
            Some(Sha256::digest(&settings).into()),
        ))
        .expect("record Review PM release binding");
    let bundle = review_person_match_candidate_storage_bundle_v1().encode_to_vec();
    store
        .record_platform_storage_bundle(
            &PlatformStorageBundleV1::new(
                REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1,
                1,
                Sha256::digest(&bundle).into(),
                bundle,
            )
            .expect("Review PM storage bundle"),
        )
        .expect("persist Review PM bundle");
    AdmittedReviewPmV1 {
        registration_id: registration.registration_id().to_owned(),
    }
}

pub(super) fn prepare_review_pm(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    admitted: AdmittedReviewPmV1,
) -> AdmittedReviewPmV1 {
    let reservation = managed_launch::reserve(supervisor, store, &admitted.registration_id)
        .expect("reserve Review PM launch");
    let bundle = store
        .platform_storage_bundle(REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1, 1)
        .expect("read Review PM bundle")
        .expect("Review PM bundle");
    let binding = issue_managed(
        store,
        &admitted.registration_id,
        reservation.runtime_instance_id(),
        reservation.runtime_generation(),
        REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
        StorageBindingIssueV1::new(1, 1, 1, *bundle.digest()).expect("Review PM storage issue"),
    )
    .expect("issue Review PM storage binding");
    crate::platform::storage::provisioning::apply_reserved_binding(supervisor, store, &binding)
        .expect("provision Review PM storage");
    admitted
}

pub(super) fn start_review_pm(
    supervisor: &ManagedRuntimeSupervisor,
    store: &SqliteControlStore,
    runtime_dir: &Path,
    admitted: AdmittedReviewPmV1,
) -> StartedReviewPmV1 {
    let reservation = managed_launch::load(supervisor, store, &admitted.registration_id)
        .expect("load Review PM reservation");
    let binding = store
        .platform_storage_binding(
            &admitted.registration_id,
            REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY_ID_V1,
        )
        .expect("read Review PM binding")
        .filter(|value| value.state() == PlatformStorageBindingStateV1::Active)
        .expect("active Review PM binding");
    let topology = crate::platform::storage::topology::current(store).expect("Storage topology");
    let vault = vault_status::read_current(store, &supervisor.relay_port()).expect("Vault status");
    let storage = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &binding,
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("Review PM storage configuration");
    let events = store
        .platform_event_hub_topology()
        .expect("Event topology")
        .expect("Event topology");
    let registration_id = admitted.registration_id;
    let runtime_instance_id = reservation.runtime_instance_id().to_owned();
    let runtime_generation = reservation.runtime_generation();
    let grant_epoch = reservation.grant_epoch();
    managed_launch::start_reserved_domain(
        supervisor,
        runtime_dir,
        reservation,
        ManagedDomainRuntimeConfigurationV1 {
            major: 1,
            logical_owner_id: REVIEW_PERSON_MATCH_CANDIDATE_OWNER_V1.to_owned(),
            logical_human_owner_id: REVIEW_PM_HUMAN_OWNER.to_owned(),
            registration_id: registration_id.clone(),
            runtime_instance_id,
            runtime_generation,
            grant_epoch,
            storage: Some(storage),
            event_hub_endpoint: events.nats_endpoint().to_owned(),
            event_credential_revision: events.credential_revision(),
        },
    )
    .expect("start Review PM domain");
    supervisor
        .wait_until_ready(&registration_id)
        .unwrap_or_else(|error| {
            panic!(
                "Review PM readiness: {error}; last={:?}",
                supervisor.last_failure(&registration_id)
            )
        });
    StartedReviewPmV1 { registration_id }
}

fn review_pm_binary() -> PathBuf {
    binary("MAKOSH_REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_BIN")
}

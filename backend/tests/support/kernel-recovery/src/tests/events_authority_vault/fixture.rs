//! Shared real managed Vault and Events authority fixture.

use std::path::{Path, PathBuf};

use makosh_kernel_control_store::{
    ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1,
    ModuleEventRouteRequestV1, ModuleEventSubscriptionRequirementV1, PlatformEventHubTopologyV1,
    PlatformEventStreamBudgetV1, PlatformEventsAuthorityConfigurationV1,
};
use makosh_runtime_protocol::v1::{
    ModuleDescriptorV1, ModuleKindV1, SettingsSchemaRefV1, SettingsSchemaV1,
};

use super::super::common::*;
use crate::platform::events::{authority, catalog, topology};
use crate::platform::managed::signed_bundle::{InstalledSignedBundle, SignedRuntimeArtifact};
use crate::tests::platform_vault::live as vault_fixture;

const EVENTS_AUTHORITY_ARTIFACT_ID: &str = "platform.events-authority";
const VAULT_ARTIFACT_ID: &str = "platform.vault";

pub(super) struct LiveAuthorityFixture {
    root: PathBuf,
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
}

impl LiveAuthorityFixture {
    pub(super) fn start(
        account_public_key: &str,
        signer_seed: &[u8],
        nats_endpoint: &str,
        nats_username: &str,
        event_hub_password: Option<&str>,
    ) -> Self {
        let root = unique_target_root("makosh-managed-events-authority-vault");
        let data = vault_fixture::private_directory(root.join("kernel"));
        initialize_vault(
            &data.join("vault"),
            &credential_directory(&root, signer_seed, event_hub_password),
        );
        let release = installed_release(&root);
        let store = Arc::new(configured_store(
            &root,
            account_public_key,
            nats_endpoint,
            nats_username,
        ));
        let supervisor = ManagedRuntimeSupervisor::new(Arc::new(AtomicBool::new(false)));
        vault_fixture::bind_and_start(&supervisor, &store, &data, release.kernel());
        authority::binding::bind_installed_release(&store, release.kernel())
            .expect("bind signed Events authority release");
        authority::launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &root.join("runtime"),
        )
        .expect("start signed Events authority");
        Self {
            root,
            store,
            supervisor,
        }
    }

    pub(super) fn store(&self) -> &Arc<SqliteControlStore> {
        &self.store
    }

    pub(super) fn supervisor(&self) -> &ManagedRuntimeSupervisor {
        &self.supervisor
    }

    pub(super) fn consumer_name(&self) -> String {
        let contracts =
            catalog::resolve_contracts(&*self.store).expect("resolve Event Hub catalog");
        let configuration = self
            .store
            .platform_event_hub_topology()
            .expect("read Event Hub topology")
            .expect("Event Hub topology");
        topology::plan(&contracts, &configuration)
            .expect("plan Event Hub topology")
            .consumers()[0]
            .durable_name()
            .to_owned()
    }
}

impl Drop for LiveAuthorityFixture {
    fn drop(&mut self) {
        let _ = self.supervisor.shutdown();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn credential_directory(
    root: &Path,
    signer_seed: &[u8],
    event_hub_password: Option<&str>,
) -> PathBuf {
    let directory = vault_fixture::private_directory(root.join("vault-credentials"));
    vault_fixture::write_private(
        &directory.join("pgbouncer-admin-password"),
        b"test-pgbouncer-password",
    );
    vault_fixture::write_private(
        &directory.join("postgres-admin-password"),
        b"test-postgres-password",
    );
    vault_fixture::write_private(&directory.join("nats-account-signer-seed"), signer_seed);
    if let Some(password) = event_hub_password {
        vault_fixture::write_private(
            &directory.join("nats-event-hub-password"),
            password.as_bytes(),
        );
    }
    directory
}

fn initialize_vault(data: &Path, credentials: &Path) {
    vault_fixture::private_directory(data.to_owned());
    let output = std::process::Command::new(vault_fixture::vault_binary())
        .args(["initialize", "--data-dir"])
        .arg(data)
        .args(["--instance-id", "kernel-main", "--platform-credential-dir"])
        .arg(credentials)
        .output()
        .expect("Vault initializer");
    assert!(output.status.success(), "Vault initialization failed");
}

fn configured_store(
    root: &Path,
    account_public_key: &str,
    nats_endpoint: &str,
    nats_username: &str,
) -> SqliteControlStore {
    let store = SqliteControlStore::create(&root.join("control.sqlite"), "kernel-main", 1)
        .expect("Control Store");
    store
        .record_platform_events_authority_configuration(
            &PlatformEventsAuthorityConfigurationV1::new(1, account_public_key, 1),
        )
        .expect("record Events authority configuration");
    store
        .record_platform_event_hub_topology(&event_hub_topology(nats_endpoint, nats_username))
        .expect("record Event Hub topology");
    register_event_contract(&store);
    store
}

fn event_hub_topology(endpoint: &str, username: &str) -> PlatformEventHubTopologyV1 {
    let budgets = [
        ModuleEventEnvelopeKindV1::Command,
        ModuleEventEnvelopeKindV1::Event,
        ModuleEventEnvelopeKindV1::Observation,
        ModuleEventEnvelopeKindV1::Result,
        ModuleEventEnvelopeKindV1::Ack,
    ]
    .into_iter()
    .map(|kind| PlatformEventStreamBudgetV1::new(kind, 1_048_576, 3_600_000, 1))
    .collect();
    PlatformEventHubTopologyV1::new(1, endpoint, username, 1, budgets)
}

fn register_event_contract(store: &SqliteControlStore) {
    register_route(
        store,
        "notes-publisher",
        "notes-publisher-module",
        ModuleEventRouteDirectionV1::Publish,
        "events.publish",
    );
    register_route(
        store,
        "notes-consumer",
        "notes-consumer-module",
        ModuleEventRouteDirectionV1::Consume,
        "events.consume",
    );
}

fn register_route(
    store: &SqliteControlStore,
    registration_id: &str,
    module_id: &str,
    direction: ModuleEventRouteDirectionV1,
    capability: &str,
) {
    let registration = ModuleRegistration::new(
        registration_id,
        module_id,
        "test_owner",
        [7; 32],
        ModuleRegistrationState::Pending,
        1,
    );
    let route = ModuleEventRouteRequestV1::new(
        makosh_kernel_control_store::ModuleEventRouteRequestInputV1 {
            registration_id: registration_id.to_owned(),
            capability_id: capability.to_owned(),
            envelope_kind: ModuleEventEnvelopeKindV1::Event,
            contract_owner: "notes".to_owned(),
            contract_name: "changed".to_owned(),
            contract_major: 1,
            contract_revision: 1,
            contract_schema_sha256: [8; 32],
            direction,
            max_in_flight: 16,
            delivery_policy: delivery_policy(direction),
        },
    );
    store
        .create_pending_registration_with_requests(
            &registration,
            &[capability.to_owned()],
            &[],
            &[route],
            &[],
        )
        .expect("record event contract route");
    store
        .approve_module_registration(registration_id, &[capability.to_owned()])
        .expect("approve event contract route");
}

fn delivery_policy(direction: ModuleEventRouteDirectionV1) -> Option<ModuleEventDeliveryPolicyV1> {
    match direction {
        ModuleEventRouteDirectionV1::Publish => None,
        ModuleEventRouteDirectionV1::Consume => Some(ModuleEventDeliveryPolicyV1::new(
            ModuleEventSubscriptionRequirementV1::Required,
            3,
            2_000,
        )),
    }
}

fn installed_release(root: &Path) -> InstalledSignedBundle {
    let schema = authority_schema();
    InstalledSignedBundle::install(
        root,
        &[
            SignedRuntimeArtifact::new(
                VAULT_ARTIFACT_ID,
                vault_fixture::vault_binary(),
                vault_fixture::vault_descriptor(),
            ),
            SignedRuntimeArtifact::new(
                EVENTS_AUTHORITY_ARTIFACT_ID,
                authority_binary(),
                authority_descriptor(&schema),
            )
            .with_settings_schema(schema),
        ],
    )
    .expect("install signed managed release")
}

fn authority_schema() -> Vec<u8> {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        ..Default::default()
    }
    .encode_to_vec()
}

fn authority_descriptor(schema: &[u8]) -> Vec<u8> {
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 1,
        module_id: "events".to_owned(),
        owner_id: "events".to_owned(),
        module_kind: ModuleKindV1::Platform as i32,
        module_version: "1".to_owned(),
        build_id: "managed-events-authority-live-test".to_owned(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: schema.len() as u64,
            sha256: Sha256::digest(schema).to_vec(),
        }),
        ..Default::default()
    }
    .encode_to_vec()
}

fn authority_binary() -> PathBuf {
    binary("MAKOSH_EVENTS_AUTHORITY_RUNTIME_BIN")
}

fn binary(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| panic!("{name} must name a regular binary"))
}

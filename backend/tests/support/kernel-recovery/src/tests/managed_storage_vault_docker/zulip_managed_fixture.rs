//! Reusable live platform contour for managed Zulip conformance flows.

use super::*;

use crate::identity::device::signer::DeviceSigner;

pub(super) struct ManagedZulipContour {
    pub(super) root: PathBuf,
    pub(super) data: PathBuf,
    pub(super) vault_dir: PathBuf,
    pub(super) fixture: ZulipHttpsFixture,
    pub(super) store: Arc<SqliteControlStore>,
    pub(super) shutdown: Arc<AtomicBool>,
    pub(super) supervisor: ManagedRuntimeSupervisor,
    pub(super) owner_signer: FileDeviceSigner,
    pub(super) seeded_credential: SeededZulipCredential,
    pub(super) zulip: StartedZulipRuntime,
    pub(super) child_stdio_capture: PathBuf,
}

impl ManagedZulipContour {
    pub(super) fn start(grant_profile: ZulipGrantProfileV1) -> Self {
        assert_eq!(
            std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
            Ok("1")
        );
        let root = private_directory(unique_target_root("makosh-managed-zulip-runtime"));
        let configuration_stdio_capture =
            private_directory(root.join("zulip-configuration-child-stdio"));
        let child_stdio_capture = private_directory(root.join("zulip-child-stdio"));
        let fixture = ZulipHttpsFixture::start(&root);
        unsafe {
            std::env::set_var(
                "MAKOSH_MANAGED_RUNTIME_CONFORMANCE_CA_CERT_FILE",
                fixture.ca_certificate_path(),
            );
        }
        let data = private_directory(short_communications_kernel_data_directory());
        let vault_dir = private_directory(data.join("vault"));
        initialize_vault(&vault_dir, &credential_directory());
        let seeded_credential = seed_zulip_vault(&vault_dir);
        let release = installed_communications_zulip_release(&root);
        unsafe {
            std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
        }
        let store = Arc::new(configured_communications_store(&root, release.kernel()));
        let (owner_signer, _) =
            FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
        store
            .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
                "owner-1",
                "desktop-1",
                owner_signer.public_key_sec1(),
            ))
            .expect("claim initial owner");
        let admitted_zulip = admit_zulip_runtime(&store, grant_profile);
        let shutdown = Arc::new(AtomicBool::new(false));
        let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
        configure_route_handler(&supervisor, &store, &data);
        supervisor
            .configure_event_credential_handler(Arc::new(
                UnauthenticatedNatsCredentialHandler::new(Arc::clone(&store)),
            ))
            .expect("configure Event credential handler");
        start_vault(&supervisor, &store, &data, release.kernel());
        assert_eq!(
            blob_launch::start_from_kernel(
                &supervisor,
                &store,
                release.kernel(),
                &data,
                &root.join("runtime"),
            )
            .expect("start signed Blob runtime"),
            1
        );
        start_storage(
            &supervisor,
            &store,
            release.kernel(),
            &storage_runtime_directory(),
        );
        issue_initial_communications_storage_binding(&store);
        crate::platform::storage::provisioning::apply_reserved_binding(
            &supervisor,
            &store,
            &communications_storage_binding(&store),
        )
        .expect("provision Communications Storage binding");
        let admitted_zulip = prepare_zulip_runtime(&supervisor, &store, admitted_zulip);
        configure_communications_jetstream(&store);
        start_communications_domain(&supervisor, &store, &root.join("runtime"));
        let mut zulip = start_zulip_runtime(
            &supervisor,
            &store,
            &data,
            &root.join("runtime"),
            admitted_zulip,
            fixture.realm_url(),
            Some(&configuration_stdio_capture),
        );
        assert_eq!(
            fixture.accepted_connections(),
            0,
            "configuration-only Zulip runtime must not contact the provider",
        );
        let binding = bind_zulip_credential(&store, &supervisor, &zulip, 0, 1);
        assert_eq!(binding.binding_revision, 1);
        zulip = restart_zulip_runtime(
            &supervisor,
            &store,
            &data,
            &root.join("runtime"),
            &zulip,
            fixture.realm_url(),
            Some(&child_stdio_capture),
        );
        Self {
            root,
            data,
            vault_dir,
            fixture,
            store,
            shutdown,
            supervisor,
            owner_signer,
            seeded_credential,
            zulip,
            child_stdio_capture,
        }
    }

    pub(super) fn shutdown_processes(&self) {
        self.supervisor.shutdown().expect("stop managed processes");
    }

    pub(super) fn finish(self) {
        unsafe {
            std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
            std::env::remove_var("MAKOSH_MANAGED_RUNTIME_CONFORMANCE_CA_CERT_FILE");
        }
        let Self {
            root,
            data,
            vault_dir,
            fixture,
            store,
            shutdown,
            supervisor,
            owner_signer,
            seeded_credential: _,
            zulip,
            child_stdio_capture,
        } = self;
        drop(zulip);
        drop(child_stdio_capture);
        drop(owner_signer);
        drop(supervisor);
        drop(shutdown);
        drop(store);
        drop(fixture);
        std::fs::remove_dir_all(root).expect("remove fixture");
        drop(vault_dir);
        std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
    }
}

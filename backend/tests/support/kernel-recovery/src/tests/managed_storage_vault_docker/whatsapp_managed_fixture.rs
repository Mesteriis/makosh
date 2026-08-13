//! Reusable live platform contour for managed WhatsApp conformance flows.

use super::*;

use crate::identity::device::signer::DeviceSigner;

pub(super) struct ManagedWhatsAppContour {
    pub(super) root: PathBuf,
    pub(super) data: PathBuf,
    pub(super) store: Arc<SqliteControlStore>,
    pub(super) shutdown: Arc<AtomicBool>,
    pub(super) supervisor: ManagedRuntimeSupervisor,
    pub(super) owner_signer: FileDeviceSigner,
    pub(super) whatsapp: StartedWhatsAppRuntime,
    pub(super) child_stdio_capture: PathBuf,
}

impl ManagedWhatsAppContour {
    pub(super) fn start(grant_profile: WhatsAppGrantProfileV1) -> Self {
        assert_eq!(
            std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
            Ok("1")
        );
        let root = private_directory(unique_target_root("makosh-managed-whatsapp-runtime"));
        let child_stdio_capture = private_directory(root.join("whatsapp-child-stdio"));
        let data = private_directory(short_communications_kernel_data_directory());
        let vault_dir = private_directory(data.join("vault"));
        initialize_vault(&vault_dir, &credential_directory());
        let release = installed_communications_whatsapp_release(&root);
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
        let admitted_whatsapp = admit_whatsapp_runtime(&store, grant_profile);
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
                &data.join("runtime"),
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
        let admitted_whatsapp = prepare_whatsapp_runtime(&supervisor, &store, admitted_whatsapp);
        configure_communications_jetstream(&store);
        start_communications_domain(&supervisor, &store, &data.join("runtime"));
        let whatsapp = start_whatsapp_runtime(
            &supervisor,
            &store,
            &data,
            &data.join("runtime"),
            admitted_whatsapp,
            Some(&child_stdio_capture),
        );
        Self {
            root,
            data,
            store,
            shutdown,
            supervisor,
            owner_signer,
            whatsapp,
            child_stdio_capture,
        }
    }

    pub(super) fn shutdown_processes(&self) {
        self.supervisor.shutdown().expect("stop managed processes");
    }

    pub(super) fn finish(self) {
        unsafe {
            std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
        }
        let Self {
            root,
            data,
            store,
            shutdown,
            supervisor,
            owner_signer,
            whatsapp,
            child_stdio_capture,
        } = self;
        drop(whatsapp);
        drop(child_stdio_capture);
        drop(owner_signer);
        drop(supervisor);
        drop(shutdown);
        drop(store);
        std::fs::remove_dir_all(root).expect("remove fixture");
        std::fs::remove_dir_all(data).expect("remove short kernel data fixture");
    }
}

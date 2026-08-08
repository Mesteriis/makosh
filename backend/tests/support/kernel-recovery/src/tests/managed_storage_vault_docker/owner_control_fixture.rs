//! Owner-authorized control session fixture shared by managed integration tests.

use super::*;

use std::os::unix::fs::{FileTypeExt, PermissionsExt};

use makosh_gateway_protocol::owner_control_client::{
    OwnerControlClientV1, OwnerControlProofSignerV1,
};

use crate::identity::device::signer::DeviceSigner;

struct LiveOwnerSigner<'a>(&'a FileDeviceSigner);

impl OwnerControlProofSignerV1 for LiveOwnerSigner<'_> {
    fn sign_owner_control_proof(&self, message: &[u8]) -> Result<[u8; 64], String> {
        Ok(self.0.sign(message))
    }
}

pub(super) struct OwnerRegistrationTransitionV1 {
    pub(super) state: String,
    pub(super) grant_epoch: u64,
}

pub(super) fn start_owner_control(
    data: &Path,
    store: &Arc<SqliteControlStore>,
    shutdown: &Arc<AtomicBool>,
    supervisor: &ManagedRuntimeSupervisor,
) -> (PathBuf, std::thread::JoinHandle<Result<(), String>>) {
    let runtime_dir = private_directory(data.join("owner-control-runtime"));
    let server_runtime_dir = runtime_dir.clone();
    let server_data = data.to_path_buf();
    let server_store = Arc::clone(store);
    let server_shutdown = Arc::clone(shutdown);
    let server_supervisor = supervisor.clone();
    let server = std::thread::spawn(move || {
        crate::identity::owner_control::serve(
            server_store,
            &server_data,
            &server_runtime_dir,
            server_shutdown,
            server_supervisor,
            None,
        )
    });
    for _ in 0..250 {
        if owner_control_socket_is_ready(&runtime_dir.join("owner.sock")) {
            return (runtime_dir, server);
        }
        if server.is_finished() {
            let outcome = server.join().expect("join failed owner control server");
            panic!("owner control server exited before socket readiness: {outcome:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("owner control socket did not become ready");
}

fn owner_control_socket_is_ready(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|metadata| {
        metadata.file_type().is_socket() && metadata.permissions().mode() & 0o777 == 0o600
    })
}

pub(super) fn transition_registration(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
    registration_id: &str,
    target_state: &str,
) -> OwnerRegistrationTransitionV1 {
    let client = OwnerControlClientV1::new(owner_runtime_dir);
    let owner_session = client
        .open_owner_session(&LiveOwnerSigner(signer))
        .expect("open owner-authorized control session");
    let transition = client
        .transition_module_registration(&owner_session, registration_id, target_state)
        .expect("transition managed module registration");
    OwnerRegistrationTransitionV1 {
        state: transition.registration_state,
        grant_epoch: transition.grant_epoch,
    }
}

pub(super) fn open_owner_control_client(
    owner_runtime_dir: &Path,
    signer: &FileDeviceSigner,
) -> (OwnerControlClientV1, String) {
    let client = OwnerControlClientV1::new(owner_runtime_dir);
    let owner_session = client
        .open_owner_session(&LiveOwnerSigner(signer))
        .expect("open owner-authorized control session");
    (client, owner_session)
}

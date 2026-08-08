//! Process-separated external Storage runtime conformance.

mod client;
mod control;
mod kernel;
mod transport;

use std::os::unix::fs::OpenOptionsExt;
use std::process::Command;
use std::time::{Duration, Instant};

use p256::ecdsa::SigningKey;

use self::control::{RUNTIME_GENERATION, RUNTIME_ID};

#[test]
#[ignore = "builds a disposable signed Kernel app and external runtime process"]
fn external_process_receives_a_live_vault_credential_after_owner_binding() {
    let mut kernel = kernel::RunningKernel::start().expect("disposable Kernel app");
    let signing_key = SigningKey::from_bytes((&[61_u8; 32]).into()).expect("external key");
    let registration_id = control::configure(&kernel, &signing_key).expect("owner setup");
    let paths = ProcessPaths::new(&kernel);
    write_config(&paths, &kernel, &registration_id, &signing_key).expect("process config");
    let mut runtime = start_external_process(&paths).expect("external runtime process");
    wait_for(&paths.attested).expect("external runtime attestation");
    control::issue_binding(&kernel, &registration_id, 1, 1).expect("owner Storage binding");
    create_private(&paths.binding, b"binding").expect("binding marker");
    wait_for(&paths.initial_result).expect("initial Vault credential result");
    control::rotate_binding(&kernel, &registration_id).expect("owner Storage binding rotation");
    create_private(&paths.rotation, b"rotation").expect("rotation marker");
    let status = runtime.wait().expect("external runtime completion");
    assert!(status.success(), "external runtime fixture failed");
    let digests = std::fs::read_to_string(&paths.result).expect("credential digest result");
    let (initial, rotated) = digests
        .trim_end()
        .split_once('\n')
        .expect("both credential digest results");
    assert_digest(initial);
    assert_digest(rotated);
    assert_ne!(initial, rotated, "the fenced credential is rotated");
    kernel.stop();
}

struct ProcessPaths {
    config: std::path::PathBuf,
    attested: std::path::PathBuf,
    binding: std::path::PathBuf,
    initial_result: std::path::PathBuf,
    rotation: std::path::PathBuf,
    result: std::path::PathBuf,
}

impl ProcessPaths {
    fn new(kernel: &kernel::RunningKernel) -> Self {
        Self {
            config: kernel.data_dir.join("external-storage-process.conf"),
            attested: kernel.data_dir.join("external-storage-attested"),
            binding: kernel.data_dir.join("external-storage-binding"),
            initial_result: kernel.data_dir.join("external-storage-initial-result"),
            rotation: kernel.data_dir.join("external-storage-rotation"),
            result: kernel.data_dir.join("external-storage-result"),
        }
    }
}

fn write_config(
    paths: &ProcessPaths,
    kernel: &kernel::RunningKernel,
    registration_id: &str,
    signing_key: &SigningKey,
) -> Result<(), String> {
    let scalar = signing_key.to_bytes();
    let config = format!(
        "socket_path={}\nregistration_id={registration_id}\nruntime_id={RUNTIME_ID}\nruntime_generation={RUNTIME_GENERATION}\nsigning_key={}\nattested_path={}\nbinding_path={}\ninitial_result_path={}\nrotation_path={}\nresult_path={}\n",
        kernel.runtime_socket.display(),
        hex(&scalar),
        paths.attested.display(),
        paths.binding.display(),
        paths.initial_result.display(),
        paths.rotation.display(),
        paths.result.display(),
    );
    create_private(&paths.config, config.as_bytes())
}

fn assert_digest(value: &str) {
    assert!(
        value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "expected SHA-256 digest"
    );
}

fn start_external_process(paths: &ProcessPaths) -> Result<std::process::Child, String> {
    std::env::current_exe()
        .map_err(|error| error.to_string())
        .and_then(|current| {
            Command::new(current)
                .args(["external_runtime_process_fixture", "--nocapture"])
                .env("MAKOSH_EXTERNAL_STORAGE_PROCESS_CONFIG", &paths.config)
                .spawn()
                .map_err(|error| error.to_string())
        })
}

fn wait_for(path: &std::path::Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if path.is_file() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err("external runtime did not attest".to_owned())
}

fn create_private(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| error.to_string())?;
    use std::io::Write;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| error.to_string())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

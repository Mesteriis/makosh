//! Live runtime-adapter conformance for the authenticated PgBouncer contour.

use makosh_storage_protocol::v1::{StorageDeploymentProfileV1, StorageRuntimeTopologyV1};
use zeroize::Zeroizing;

const AUTHENTICATED_TEST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_TEST";
const PASSWORD_FILE_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PASSWORD_FILE";
const PGBOUNCER_HOST_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_HOST";
const PGBOUNCER_PORT_ENV: &str = "MAKOSH_STORAGE_AUTHENTICATED_PGBOUNCER_PORT";

#[test]
#[ignore = "requires the disposable authenticated Storage Compose contour"]
fn authenticated_runtime_readiness_accepts_the_resolved_platform_credential() {
    assert_eq!(std::env::var(AUTHENTICATED_TEST_ENV).as_deref(), Ok("1"));

    let credential = Zeroizing::new(read_platform_credential());
    crate::admin::verify_platform_admin(&topology(), &credential)
        .expect("runtime adapter authenticates the resolved platform credential");
}

fn topology() -> StorageRuntimeTopologyV1 {
    StorageRuntimeTopologyV1 {
        topology_revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".to_owned(),
        database_id: "makosh".to_owned(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
        postgres_artifact_sha256: vec![1; 32],
        pgbouncer_artifact_sha256: vec![2; 32],
        postgres_host: "127.0.0.1".to_owned(),
        postgres_port: 5432,
        pgbouncer_host: required(PGBOUNCER_HOST_ENV),
        pgbouncer_port: port(PGBOUNCER_PORT_ENV),
        pgbouncer_postgres_host: "127.0.0.1".to_owned(),
        pgbouncer_postgres_port: 5432,
    }
}

fn read_platform_credential() -> Vec<u8> {
    let path = required(PASSWORD_FILE_ENV);
    let metadata = std::fs::symlink_metadata(&path).expect("platform credential metadata");
    assert!(metadata.is_file() && !metadata.file_type().is_symlink());
    let mut credential = std::fs::read(path).expect("platform credential file");
    while matches!(credential.last(), Some(b'\n' | b'\r')) {
        credential.pop();
    }
    assert!(!credential.is_empty());
    credential
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("storage integration test requires {name}"))
}

fn port(name: &str) -> u32 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("storage integration test requires a valid {name}"))
}

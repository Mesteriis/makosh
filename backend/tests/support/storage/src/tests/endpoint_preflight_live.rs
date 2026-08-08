//! Live endpoint reachability conformance for the disposable Compose contour.

use makosh_storage_control::{StorageEndpointPreflightV1, preflight_storage_endpoints};
use makosh_storage_protocol::v1::{StorageDeploymentProfileV1, StorageRuntimeTopologyV1};

const PG_HOST: &str = "MAKOSH_STORAGE_TEST_POSTGRES_HOST";
const PG_PORT: &str = "MAKOSH_STORAGE_TEST_POSTGRES_PORT";
const POOL_HOST: &str = "MAKOSH_STORAGE_TEST_PGBOUNCER_HOST";
const POOL_PORT: &str = "MAKOSH_STORAGE_TEST_PGBOUNCER_PORT";

#[test]
#[ignore = "requires the disposable development Compose contour"]
fn runtime_preflight_reaches_the_live_postgres_and_pgbouncer_endpoints() {
    let topology = StorageRuntimeTopologyV1 {
        topology_revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".into(),
        database_id: "makosh_development".into(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
        postgres_artifact_sha256: vec![1; 32],
        pgbouncer_artifact_sha256: vec![2; 32],
        postgres_host: value(PG_HOST),
        postgres_port: port(PG_PORT),
        pgbouncer_host: value(POOL_HOST),
        pgbouncer_port: port(POOL_PORT),
        pgbouncer_postgres_host: value(PG_HOST),
        pgbouncer_postgres_port: port(PG_PORT),
    };

    assert_eq!(
        preflight_storage_endpoints(&topology),
        StorageEndpointPreflightV1::Available
    );
}

fn value(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("storage integration test requires {name}"))
}

fn port(name: &str) -> u32 {
    value(name)
        .parse()
        .unwrap_or_else(|_| panic!("storage integration test requires a valid {name}"))
}

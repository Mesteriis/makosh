//! Endpoint preflight conformance without credentials or SQL.

use std::net::TcpListener;

use makosh_storage_control::{StorageEndpointPreflightV1, preflight_storage_endpoints};
use makosh_storage_protocol::v1::{StorageDeploymentProfileV1, StorageRuntimeTopologyV1};

#[test]
fn endpoint_preflight_accepts_two_reachable_endpoints() {
    let postgres = TcpListener::bind("127.0.0.1:0").expect("postgres listener");
    let pgbouncer = TcpListener::bind("127.0.0.1:0").expect("pgbouncer listener");

    assert_eq!(
        preflight_storage_endpoints(&topology(port(&postgres), port(&pgbouncer))),
        StorageEndpointPreflightV1::Available
    );
}

#[test]
fn endpoint_preflight_fails_closed_when_one_endpoint_is_unreachable() {
    let postgres = TcpListener::bind("127.0.0.1:0").expect("postgres listener");
    let unavailable = TcpListener::bind("127.0.0.1:0").expect("unavailable port");
    let unavailable_port = port(&unavailable);
    drop(unavailable);

    assert_eq!(
        preflight_storage_endpoints(&topology(port(&postgres), unavailable_port)),
        StorageEndpointPreflightV1::Unavailable
    );
}

fn topology(postgres_port: u16, pgbouncer_port: u16) -> StorageRuntimeTopologyV1 {
    StorageRuntimeTopologyV1 {
        topology_revision: 1,
        storage_generation: 1,
        storage_instance_id: "storage_main".into(),
        database_id: "makosh".into(),
        deployment_profile: StorageDeploymentProfileV1::MacosTauriEmbedded as i32,
        postgres_artifact_sha256: vec![1; 32],
        pgbouncer_artifact_sha256: vec![2; 32],
        postgres_host: "127.0.0.1".into(),
        postgres_port: u32::from(postgres_port),
        pgbouncer_host: "127.0.0.1".into(),
        pgbouncer_port: u32::from(pgbouncer_port),
        pgbouncer_postgres_host: "127.0.0.1".into(),
        pgbouncer_postgres_port: u32::from(postgres_port),
    }
}

fn port(listener: &TcpListener) -> u16 {
    listener.local_addr().expect("listener address").port()
}

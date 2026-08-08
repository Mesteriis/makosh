//! Checks only whether the declared infrastructure endpoints accept TCP.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use makosh_storage_protocol::v1::StorageRuntimeTopologyV1;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RESOLVED_ADDRESSES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageEndpointPreflightV1 {
    Available,
    Unavailable,
}

pub fn preflight_storage_endpoints(
    topology: &StorageRuntimeTopologyV1,
) -> StorageEndpointPreflightV1 {
    if endpoint_reachable(&topology.postgres_host, topology.postgres_port)
        && endpoint_reachable(&topology.pgbouncer_host, topology.pgbouncer_port)
    {
        StorageEndpointPreflightV1::Available
    } else {
        StorageEndpointPreflightV1::Unavailable
    }
}

fn endpoint_reachable(host: &str, port: u32) -> bool {
    let Ok(port) = u16::try_from(port) else {
        return false;
    };
    (host, port)
        .to_socket_addrs()
        .map(|addresses| {
            addresses
                .take(MAX_RESOLVED_ADDRESSES)
                .any(|address| TcpStream::connect_timeout(&address, CONNECT_TIMEOUT).is_ok())
        })
        .unwrap_or(false)
}

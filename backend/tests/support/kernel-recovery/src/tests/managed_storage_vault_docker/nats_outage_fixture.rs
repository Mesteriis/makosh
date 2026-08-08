//! Authenticated NATS outage controls shared by live integration flows.

use std::time::Instant;

use super::*;

pub(super) fn set_authenticated_nats_container_running(running: bool) {
    let container = std::env::var("MAKOSH_STORAGE_AUTHENTICATED_NATS_CONTAINER")
        .expect("authenticated NATS container");
    assert!(
        (12..=64).contains(&container.len())
            && container.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "authenticated NATS container id is invalid"
    );
    let mut command = std::process::Command::new("docker");
    if running {
        command.args(["start", &container]);
    } else {
        command.args(["stop", "--timeout", "1", &container]);
    }
    assert!(
        command
            .status()
            .expect("control authenticated NATS container")
            .success(),
        "authenticated NATS container state change failed"
    );
}

pub(super) fn wait_for_authenticated_nats_reconnect(
    runtime: &tokio::runtime::Runtime,
    client: &async_nats::Client,
    observer: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while client.connection_state() != async_nats::connection::State::Connected {
        assert!(
            Instant::now() < deadline,
            "{observer} did not reconnect to NATS"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    runtime
        .block_on(client.flush())
        .unwrap_or_else(|error| panic!("flush reconnected {observer}: {error}"));
}

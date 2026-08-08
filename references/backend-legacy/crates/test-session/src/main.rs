use std::env;
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};

use makosh_test_session::containers::labels::SESSION_ID_ENV;
use makosh_test_session::containers::nats::{NatsContainer, SESSION_NATS_HOST_PORT_ENV};
use makosh_test_session::containers::pgbouncer::{
    PgbouncerContainer, SESSION_PGBOUNCER_HOST_PORT_ENV,
};
use makosh_test_session::containers::postgres::{
    PostgresContainer, SESSION_POSTGRES_HOST_PORT_ENV,
};
use tokio::process::Command;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let command_args = env::args().skip(1).collect::<Vec<_>>();
    if command_args.is_empty() {
        eprintln!("usage: makosh-test-session <command> [args...]");
        std::process::exit(2);
    }

    let session_id = format!("makosh-test-{}", Uuid::new_v4());
    eprintln!("[makosh-test-session] session_id={session_id}");
    eprintln!("[makosh-test-session] cleaning stale Макошь testcontainers");
    cleanup_old_containers();

    eprintln!("[makosh-test-session] starting PostgreSQL testcontainer");
    let postgres_container = PostgresContainer::start_owned_with_session(&session_id).await;
    eprintln!(
        "[makosh-test-session] PostgreSQL ready on 127.0.0.1:{}",
        postgres_container.host_port()
    );
    eprintln!("[makosh-test-session] starting PgBouncer testcontainer");
    let pgbouncer_container = PgbouncerContainer::start_owned_with_session(&session_id).await;
    eprintln!(
        "[makosh-test-session] PgBouncer ready on 127.0.0.1:{}",
        pgbouncer_container.host_port()
    );
    eprintln!("[makosh-test-session] starting NATS testcontainer");
    let nats_container = NatsContainer::start_owned_with_session(&session_id).await;
    eprintln!(
        "[makosh-test-session] NATS ready on 127.0.0.1:{}",
        nats_container.host_port()
    );
    eprintln!(
        "[makosh-test-session] running command: {} {}",
        command_args[0],
        command_args[1..].join(" ")
    );
    let mut child = Command::new(&command_args[0])
        .args(&command_args[1..])
        .env(SESSION_ID_ENV, &session_id)
        .env(
            SESSION_POSTGRES_HOST_PORT_ENV,
            postgres_container.host_port().to_string(),
        )
        .env(
            SESSION_PGBOUNCER_HOST_PORT_ENV,
            pgbouncer_container.host_port().to_string(),
        )
        .env(
            SESSION_NATS_HOST_PORT_ENV,
            nats_container.host_port().to_string(),
        )
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|error| {
            panic!(
                "failed to run test session command '{}': {error}",
                command_args[0]
            )
        });

    let exit_code = tokio::select! {
        status = child.wait() => {
            status
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to wait for test session command '{}': {error}",
                        command_args[0]
                    )
                })
                .code()
                .unwrap_or(1)
        }
        _ = wait_for_shutdown_signal() => {
            eprintln!("[makosh-test-session] shutdown signal received; stopping child process");
            if let Err(error) = child.kill().await {
                eprintln!("failed to stop test session child process: {error}");
            }
            130
        }
    };

    eprintln!("[makosh-test-session] command exited with code {exit_code}");
    eprintln!("[makosh-test-session] stopping session testcontainers");
    drop(nats_container);
    drop(pgbouncer_container);
    drop(postgres_container);
    eprintln!("[makosh-test-session] cleaning session testcontainers");
    cleanup_session_containers(&session_id);
    eprintln!("[makosh-test-session] done");
    std::process::exit(exit_code);
}

fn cleanup_old_containers() {
    let max_age = env::var("MAKOSH_TESTCONTAINERS_CLEANUP_MAX_AGE_SECS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "7200".to_owned());
    run_cleanup_best_effort(&["--older-than-seconds", &max_age]);
}

fn cleanup_session_containers(session_id: &str) {
    run_cleanup_best_effort(&["--session-id", session_id]);
}

fn run_cleanup_best_effort(args: &[&str]) {
    let script = cleanup_script_path();
    let status = StdCommand::new("bash").arg(script).args(args).status();

    match status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("testcontainers cleanup exited with status {status}"),
        Err(error) => eprintln!("failed to run testcontainers cleanup: {error}"),
    }
}

fn cleanup_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/test/clean-testcontainers.sh")
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut interrupt = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
    let mut terminate = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

    tokio::select! {
        _ = interrupt.recv() => {}
        _ = terminate.recv() => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
}

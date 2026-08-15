#![allow(dead_code)]

// The recovery-only Kernel inventory compiles phase-gated owner and platform
// paths before their admission gate opens. Semantic Clippy lints remain
// mandatory; this only suppresses reachability noise at the binary root.

use clap::Parser;

mod cli;
mod control_store;
mod distribution;
mod identity;
mod infrastructure;
mod modules;
mod observability;
mod platform;
mod recovery;
mod runtime;

use cli::{BrowserPairingCommand, Cli, Command};

fn main() {
    if let Err(error) = observability::initialize() {
        eprintln!("kernel observability initialization failed: {error}");
        std::process::exit(1);
    }
    tracing::info!(event = "kernel.process.started");
    if let Err(error) = run(Cli::parse()) {
        tracing::error!(
            event = "kernel.bootstrap.failed",
            error.class = "kernel_bootstrap",
            error.message = %error,
        );
        std::process::exit(1);
    }
    tracing::info!(event = "kernel.process.stopped");
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::ControlStore { operation } => control_store::offline::run(cli.data_dir, operation),
        Command::WholeInstanceRecovery { operation } => {
            recovery::offline::run(cli.data_dir, *operation)
        }
        Command::DeviceKeyGenerate => identity::device::generation::run(cli.data_dir),
        Command::InitialOwnerEnroll {
            owner_id,
            device_id,
        } => identity::enrollment::initial::run(cli.data_dir, &owner_id, &device_id),
        Command::ServerBootstrapPairing {
            listen_address,
            ttl_seconds,
        } => identity::server_pairing::bootstrap::run(cli.data_dir, listen_address, ttl_seconds),
        Command::BrowserPairing {
            operation: BrowserPairingCommand::Create,
        } => identity::owner_control::cli::create_browser_pairing(cli.data_dir),
        command => runtime::run(cli.data_dir, command),
    }
}

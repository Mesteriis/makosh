//! Explicit CLI surface for development-only Kernel simulation operations.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::kernel_operator;
use crate::platform::commands as platform_commands;

#[derive(Parser)]
#[command(name = "makosh-development-kernel-operator")]
pub(crate) struct Cli {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Probe {
        #[arg(long)]
        postgres_address: SocketAddr,
        #[arg(long)]
        nats_address: SocketAddr,
    },
    Pairing {
        #[command(subcommand)]
        operation: PairingCommand,
    },
    HoldLock,
    InitialOwnerImportPairing {
        #[arg(long)]
        pairing_state_dir: PathBuf,
    },
    ModuleRegister {
        #[arg(long)]
        descriptor: PathBuf,
    },
    ModuleApprove {
        #[arg(long)]
        registration_id: String,
        #[arg(long, required = true)]
        capability: Vec<String>,
    },
    ModuleTransition {
        #[arg(long)]
        registration_id: String,
        #[arg(long, value_parser = ["suspended", "revoked"])]
        state: String,
    },
    ModuleStatus {
        #[arg(long)]
        registration_id: String,
    },
    ModuleExternalAttest {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        runtime_id: String,
        #[arg(long)]
        runtime_generation: u64,
        #[arg(long)]
        distribution_artifact: PathBuf,
    },
    ModuleAuthorizeExternalCapability {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        runtime_id: String,
        #[arg(long)]
        runtime_generation: u64,
        #[arg(long)]
        capability: String,
    },
    ModuleBindExternalRuntimeKey {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        public_key_sec1_hex: String,
    },
    ModuleOwnerPinArtifact {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        artifact: PathBuf,
    },
    ModuleOwnerPinnedPreflight {
        #[arg(long)]
        registration_id: String,
    },
    ModuleRunPinnedChild {
        #[arg(long)]
        registration_id: String,
        #[arg(long, default_value_t = 10)]
        max_runtime_seconds: u64,
    },
    ModuleAdmitSettingsSchema {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        descriptor: PathBuf,
        #[arg(long)]
        schema: PathBuf,
    },
    ModuleUpdateOperatorSettings {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        expected_revision: u64,
        #[arg(long)]
        snapshot: PathBuf,
    },
    ModuleAcknowledgeSettingsLifecycle {
        #[arg(long)]
        registration_id: String,
        #[arg(long)]
        revision: u64,
        #[arg(long, value_parser = ["validation_accepted", "validation_rejected", "apply_started", "external_restart_required", "runtime_applied"])]
        acknowledgement: String,
        #[arg(long)]
        reason_code: Option<String>,
    },
}

#[derive(Subcommand)]
enum PairingCommand {
    Create {
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        ttl_seconds: u64,
    },
    Consume {
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        token: String,
    },
    Listen {
        #[arg(long)]
        state_dir: PathBuf,
        #[arg(long)]
        listen_address: SocketAddr,
        #[arg(long)]
        idle_timeout_seconds: u64,
    },
    Proof {
        #[arg(long)]
        key_dir: PathBuf,
        #[arg(long)]
        challenge: String,
        #[arg(long)]
        owner_id: String,
        #[arg(long)]
        device_id: String,
    },
}

pub(crate) fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Probe {
            postgres_address,
            nats_address,
        } => platform_commands::probe(postgres_address, nats_address),
        Command::Pairing { operation } => run_pairing(operation),
        Command::HoldLock => hold_lock(cli.data_dir),
        Command::InitialOwnerImportPairing { pairing_state_dir } => {
            kernel_operator::run_remote_pairing_owner_enrollment(cli.data_dir, &pairing_state_dir)
        }
        command => run_module_command(cli.data_dir, command),
    }
}

fn run_pairing(command: PairingCommand) -> Result<(), String> {
    match command {
        PairingCommand::Create {
            state_dir,
            ttl_seconds,
        } => platform_commands::pairing_create(&state_dir, ttl_seconds),
        PairingCommand::Consume { state_dir, token } => {
            platform_commands::pairing_consume(&state_dir, &token)
        }
        PairingCommand::Listen {
            state_dir,
            listen_address,
            idle_timeout_seconds,
        } => platform_commands::pairing_listen(&state_dir, listen_address, idle_timeout_seconds),
        PairingCommand::Proof {
            key_dir,
            challenge,
            owner_id,
            device_id,
        } => platform_commands::pairing_proof(&key_dir, &challenge, &owner_id, &device_id),
    }
}

fn run_module_command(data_dir: Option<PathBuf>, command: Command) -> Result<(), String> {
    match command {
        Command::ModuleRegister { descriptor } => {
            kernel_operator::run_module_registration(data_dir, &descriptor)
        }
        Command::ModuleApprove {
            registration_id,
            capability,
        } => kernel_operator::run_module_approval(data_dir, &registration_id, &capability),
        Command::ModuleTransition {
            registration_id,
            state,
        } => kernel_operator::run_module_transition(data_dir, &registration_id, &state),
        Command::ModuleStatus { registration_id } => {
            kernel_operator::run_module_status(data_dir, &registration_id)
        }
        command => run_runtime_command(data_dir, command),
    }
}

fn run_runtime_command(data_dir: Option<PathBuf>, command: Command) -> Result<(), String> {
    match command {
        Command::ModuleExternalAttest {
            registration_id,
            runtime_id,
            runtime_generation,
            distribution_artifact,
        } => kernel_operator::run_external_runtime_attestation(
            data_dir,
            &registration_id,
            &runtime_id,
            runtime_generation,
            &distribution_artifact,
        ),
        Command::ModuleAuthorizeExternalCapability {
            registration_id,
            runtime_id,
            runtime_generation,
            capability,
        } => kernel_operator::authorize_external_capability(
            data_dir,
            &registration_id,
            &runtime_id,
            runtime_generation,
            &capability,
        ),
        Command::ModuleBindExternalRuntimeKey {
            registration_id,
            public_key_sec1_hex,
        } => kernel_operator::bind_external_runtime_identity(
            data_dir,
            &registration_id,
            &public_key_sec1_hex,
        ),
        Command::ModuleOwnerPinArtifact {
            registration_id,
            artifact,
        } => kernel_operator::run_owner_pinned_artifact_binding(
            data_dir,
            &registration_id,
            &artifact,
        ),
        Command::ModuleOwnerPinnedPreflight { registration_id } => {
            kernel_operator::run_owner_pinned_artifact_preflight(data_dir, &registration_id)
        }
        Command::ModuleRunPinnedChild {
            registration_id,
            max_runtime_seconds,
        } => kernel_operator::run_pinned_child(data_dir, &registration_id, max_runtime_seconds),
        command => run_settings_command(data_dir, command),
    }
}

fn run_settings_command(data_dir: Option<PathBuf>, command: Command) -> Result<(), String> {
    match command {
        Command::ModuleAdmitSettingsSchema {
            registration_id,
            descriptor,
            schema,
        } => {
            kernel_operator::admit_settings_schema(data_dir, &registration_id, &descriptor, &schema)
        }
        Command::ModuleUpdateOperatorSettings {
            registration_id,
            expected_revision,
            snapshot,
        } => kernel_operator::update_operator_settings(
            data_dir,
            &registration_id,
            expected_revision,
            &snapshot,
        ),
        Command::ModuleAcknowledgeSettingsLifecycle {
            registration_id,
            revision,
            acknowledgement,
            reason_code,
        } => kernel_operator::acknowledge_settings_lifecycle(
            data_dir,
            &registration_id,
            revision,
            &acknowledgement,
            reason_code.as_deref(),
        ),
        Command::Probe { .. }
        | Command::Pairing { .. }
        | Command::HoldLock
        | Command::InitialOwnerImportPairing { .. }
        | Command::ModuleRegister { .. }
        | Command::ModuleApprove { .. }
        | Command::ModuleTransition { .. }
        | Command::ModuleStatus { .. }
        | Command::ModuleExternalAttest { .. }
        | Command::ModuleAuthorizeExternalCapability { .. }
        | Command::ModuleBindExternalRuntimeKey { .. }
        | Command::ModuleOwnerPinArtifact { .. }
        | Command::ModuleOwnerPinnedPreflight { .. }
        | Command::ModuleRunPinnedChild { .. } => unreachable!("misrouted command"),
    }
}

fn hold_lock(data_dir: Option<PathBuf>) -> Result<(), String> {
    let (runtime_dir, _store) =
        kernel_operator::control::plane::open_development_control_store(data_dir)?;
    let _lock = crate::infrastructure::filesystem::acquire_runtime_directory_lock(&runtime_dir)?;
    println!("lock_held");
    loop {
        std::thread::park();
    }
}

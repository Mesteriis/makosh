//! Production Kernel command-line contract.

use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::platform::gateway::BrowserGatewayConfigurationV1;

#[derive(Parser)]
#[command(name = "makosh-kernel")]
pub(crate) struct Cli {
    #[arg(long)]
    pub(crate) data_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Status,
    Serve {
        #[command(flatten)]
        browser_gateway: BrowserGatewayCli,
    },
    DeviceKeyGenerate,
    InitialOwnerEnroll {
        #[arg(long)]
        owner_id: String,
        #[arg(long)]
        device_id: String,
    },
    ServerBootstrapPairing {
        #[arg(long)]
        listen_address: SocketAddr,
        #[arg(long, default_value_t = 300)]
        ttl_seconds: u64,
    },
    BrowserPairing {
        #[command(subcommand)]
        operation: BrowserPairingCommand,
    },
    ControlStore {
        #[command(subcommand)]
        operation: OfflineControlStoreCommand,
    },
    WholeInstanceRecovery {
        #[command(subcommand)]
        operation: Box<WholeInstanceRecoveryCommand>,
    },
}

/// Browser device enrollment is created only through the owner-private server
/// CLI. The browser never obtains a registration ceremony by itself.
#[derive(Subcommand)]
pub(crate) enum BrowserPairingCommand {
    Create,
}

/// Browser Gateway admission is intentionally opt-in. A running Kernel never
/// chooses an origin, RP ID or TLS identity on behalf of its operator.
#[derive(Args)]
pub(crate) struct BrowserGatewayCli {
    #[arg(long)]
    browser_gateway_listen_address: Option<SocketAddr>,
    #[arg(long)]
    browser_gateway_origin: Option<String>,
    #[arg(long)]
    browser_gateway_rp_id: Option<String>,
    #[arg(long)]
    browser_gateway_certificate_der: Option<PathBuf>,
    #[arg(long)]
    browser_gateway_private_key_der: Option<PathBuf>,
    /// Absolute path to one owner-private 64-hex process-local proof file for
    /// the loopback Vite development proxy. The proof itself must never appear
    /// in argv or environment.
    #[arg(long)]
    browser_gateway_development_proxy_proof_file: Option<PathBuf>,
    /// Starts the authenticated development platform for catalog admission,
    /// while deliberately withholding Scheduler and its topology reconciler.
    #[arg(long)]
    browser_gateway_development_admission_mode: bool,
    /// Exposes the explicitly configured Gateway as a paired-remote HTTPS
    /// listener with HTTP/2 and HTTP/3. Without this flag the listener stays
    /// loopback-only.
    #[arg(long)]
    browser_gateway_paired_remote: bool,
    /// Enables process-local private-LAN HTTP diagnostics. It never grants owner access.
    #[arg(long)]
    pub(crate) dangerous_lan_development: bool,
}

impl BrowserGatewayCli {
    pub(crate) fn into_configuration(
        self,
    ) -> Result<Option<BrowserGatewayConfigurationV1>, String> {
        let Self {
            browser_gateway_listen_address,
            browser_gateway_origin,
            browser_gateway_rp_id,
            browser_gateway_certificate_der,
            browser_gateway_private_key_der,
            browser_gateway_development_proxy_proof_file,
            browser_gateway_development_admission_mode,
            browser_gateway_paired_remote,
            dangerous_lan_development,
        } = self;
        let core_parameters = (
            browser_gateway_listen_address,
            browser_gateway_origin,
            browser_gateway_rp_id,
        );

        if let Some(proof_file) = browser_gateway_development_proxy_proof_file {
            if browser_gateway_certificate_der.is_some()
                || browser_gateway_private_key_der.is_some()
                || browser_gateway_paired_remote
                || dangerous_lan_development
            {
                return Err(
                    "loopback development proxy does not accept TLS, paired-remote or private-LAN inputs"
                        .to_owned(),
                );
            }
            let (Some(address), Some(origin), Some(rp_id)) = core_parameters else {
                return Err(
                    "loopback development proxy requires listener, origin and RP ID".to_owned(),
                );
            };
            let configuration = BrowserGatewayConfigurationV1::new_loopback_development_proxy(
                address,
                origin,
                rp_id,
                load_development_proxy_proof(&proof_file)?,
            )?;
            return if browser_gateway_development_admission_mode {
                configuration.for_development_admission().map(Some)
            } else {
                Ok(Some(configuration))
            };
        }

        if browser_gateway_development_admission_mode {
            return Err(
                "development admission mode requires loopback development proxy".to_owned(),
            );
        }

        match (
            core_parameters,
            browser_gateway_certificate_der,
            browser_gateway_private_key_der,
            browser_gateway_paired_remote,
            dangerous_lan_development,
        ) {
            ((None, None, None), None, None, false, false) => Ok(None),
            ((Some(address), Some(origin), Some(rp_id)), None, None, false, true) => {
                BrowserGatewayConfigurationV1::new_lan_development(address, origin, rp_id)
                    .map(Some)
            }
            (
                (Some(address), Some(origin), Some(rp_id)),
                Some(certificate),
                Some(private_key),
                paired_remote,
                false,
            ) => {
                let configuration = if paired_remote {
                    BrowserGatewayConfigurationV1::new_paired_remote(
                        address,
                        origin,
                        rp_id,
                        certificate,
                        private_key,
                    )
                } else {
                    BrowserGatewayConfigurationV1::new(
                        address,
                        origin,
                        rp_id,
                        certificate,
                        private_key,
                    )
                };
                configuration.map(Some)
            }
            _ => Err(
                "browser Gateway listener, origin, RP ID, certificate DER and private-key DER must be specified together; paired remote is explicit"
                    .to_owned(),
            ),
        }
    }
}

fn load_development_proxy_proof(path: &std::path::Path) -> Result<String, String> {
    if !path.is_absolute() {
        return Err("loopback development proxy proof file path must be absolute".to_owned());
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "loopback development proxy proof file is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err("loopback development proxy proof must be a regular file".to_owned());
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("loopback development proxy proof file must be owner-private".to_owned());
    }
    if metadata.len() != 64 {
        return Err("loopback development proxy proof is invalid".to_owned());
    }
    let proof = std::fs::read(path)
        .map_err(|_| "loopback development proxy proof file is unavailable".to_owned())?;
    if proof.len() != 64 || !proof.iter().all(u8::is_ascii_hexdigit) {
        return Err("loopback development proxy proof is invalid".to_owned());
    }
    String::from_utf8(proof).map_err(|_| "loopback development proxy proof is invalid".to_owned())
}

#[derive(Subcommand)]
pub(crate) enum OfflineControlStoreCommand {
    Restore {
        #[arg(long)]
        source: PathBuf,
    },
    Reset,
}

#[derive(Subcommand)]
pub(crate) enum WholeInstanceRecoveryCommand {
    Capture(WholeInstanceCaptureCli),
    Restore(WholeInstanceRestoreCli),
}

#[derive(Args)]
pub(crate) struct WholeInstanceCaptureCli {
    #[arg(long)]
    pub(crate) destination: PathBuf,
    #[arg(long)]
    pub(crate) media_encryption_key_file: PathBuf,
    #[arg(long)]
    pub(crate) media_signing_key_file: PathBuf,
    #[arg(long)]
    pub(crate) media_key_id: String,
    #[arg(long)]
    pub(crate) source_commit: String,
    #[arg(long)]
    pub(crate) cargo_lock_sha256: String,
    #[arg(long)]
    pub(crate) toolchain_sha256: String,
    #[arg(long)]
    pub(crate) policy_sha256: String,
    #[arg(long)]
    pub(crate) pg_dump: PathBuf,
    #[arg(long)]
    pub(crate) pg_restore: PathBuf,
    #[arg(long)]
    pub(crate) psql: PathBuf,
    #[arg(long)]
    pub(crate) postgres_host: String,
    #[arg(long)]
    pub(crate) postgres_port: u16,
    #[arg(long)]
    pub(crate) postgres_database: String,
    #[arg(long)]
    pub(crate) postgres_username: String,
    #[arg(long)]
    pub(crate) postgres_ssl_mode: String,
    #[arg(long)]
    pub(crate) postgres_password_file: PathBuf,
    #[arg(long)]
    pub(crate) include_blob: bool,
    #[arg(long)]
    pub(crate) include_scheduler: bool,
}

#[derive(Args)]
pub(crate) struct WholeInstanceRestoreCli {
    #[arg(long)]
    pub(crate) source: PathBuf,
    #[arg(long)]
    pub(crate) media_key_id: String,
    #[arg(long)]
    pub(crate) media_public_key_file: PathBuf,
    #[arg(long)]
    pub(crate) media_decryption_key_file: PathBuf,
    #[arg(long)]
    pub(crate) vault_recovery_key_file: PathBuf,
    #[arg(long)]
    pub(crate) pg_restore: PathBuf,
    #[arg(long)]
    pub(crate) psql: PathBuf,
    #[arg(long)]
    pub(crate) postgres_host: String,
    #[arg(long)]
    pub(crate) postgres_port: u16,
    #[arg(long)]
    pub(crate) postgres_database: String,
    #[arg(long)]
    pub(crate) postgres_username: String,
    #[arg(long)]
    pub(crate) postgres_ssl_mode: String,
    #[arg(long)]
    pub(crate) postgres_password_file: PathBuf,
}

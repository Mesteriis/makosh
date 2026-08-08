#[path = "makosh_email_sync_dev/account.rs"]
mod account;
#[path = "makosh_email_sync_dev/checkpoint.rs"]
mod checkpoint;
#[path = "makosh_email_sync_dev/config.rs"]
mod config;
#[path = "makosh_email_sync_dev/env.rs"]
mod env;
#[path = "makosh_email_sync_dev/errors.rs"]
mod errors;
#[path = "makosh_email_sync_dev/fetch.rs"]
mod fetch;
#[path = "makosh_email_sync_dev/provider.rs"]
mod provider;
#[path = "makosh_email_sync_dev/report.rs"]
mod report;
#[path = "makosh_email_sync_dev/runner.rs"]
mod runner;

use config::DevEmailSyncConfig;
use errors::DevEmailSyncError;
use runner::run_dev_email_sync;

#[tokio::main]
async fn main() -> Result<(), DevEmailSyncError> {
    makosh_hub_backend::app::router::init_tracing();

    let config = DevEmailSyncConfig::from_env()?;
    let report = run_dev_email_sync(config).await?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}

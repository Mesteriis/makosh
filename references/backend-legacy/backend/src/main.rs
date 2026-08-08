use makosh_hub_backend::platform::config::app_config::AppConfig;
use tracing::Instrument;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    makosh_hub_backend::app::router::init_tracing();
    let flow_id = std::env::var("MAKOSH_FLOW_ID").unwrap_or_else(|_| "unknown".to_owned());
    let runtime_span = tracing::info_span!("makosh_runtime", flow_id = %flow_id);

    async move {
        let config = AppConfig::from_env()?;
        makosh_hub_backend::app::router::run(config).await?;
        Ok(())
    }
    .instrument(runtime_span)
    .await
}

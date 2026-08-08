use makosh_hub_backend::app::router::init_tracing;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = AppConfig::from_env()?;
    let database_url = config
        .database_url()
        .ok_or("DATABASE_URL is required for migrations")?;

    Database::connect(Some(database_url)).await?;
    println!("Макошь backend migrations and startup repairs completed.");

    Ok(())
}

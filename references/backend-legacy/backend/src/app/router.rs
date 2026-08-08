// ADR-0073: app router owns HTTP composition; route groups live in
// focused modules so endpoint registration remains auditable without a god file.
use std::io;

use axum::extract::State;
use axum::http::{HeaderName, Method, StatusCode, header};
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;

use crate::app::error::types::AppError;
use crate::app::state::AccountSetupState;
use crate::app::state::AppState;
use crate::integrations::telegram::runtime::manager::TelegramRuntimeManager;
use crate::platform::config::app_config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::platform::storage::database::Database;
use crate::platform::storage::models;
use crate::vault::HostVault;
use crate::vault::models::HostVaultConfig;

mod routes;

pub(crate) struct ApplicationComponents {
    pub(crate) state: AppState,
    pub(crate) bootstrap: crate::application::bootstrap::ApplicationBootstrapContext,
}

pub(crate) fn compose_application(config: AppConfig, database: Database) -> ApplicationComponents {
    let nats_server_url = config.nats_server_url().map(ToOwned::to_owned);
    let vault = HostVault::new(HostVaultConfig {
        home: config.vault_home().to_path_buf(),
        dev_mode: config.dev_mode(),
        dev_key_path: config.dev_key_path().to_path_buf(),
    })
    .expect("host vault runtime must initialize");
    if let Err(error) = vault.unlock_existing() {
        tracing::warn!(error = %error, "host vault auto-unlock skipped");
    }
    let state = AppState {
        config,
        database,
        vault,
        account_setup: AccountSetupState::default(),
        telegram_runtime: TelegramRuntimeManager::default(),
        event_bus: InMemoryEventBus::new(),
    };
    let bootstrap = crate::application::bootstrap::ApplicationBootstrapContext {
        pool: state.database.pool().cloned(),
        database_url: state.database.database_url().map(ToOwned::to_owned),
        nats_server_url,
        config: state.config.clone(),
        zoom_token_maintenance_scheduler_enabled: state
            .config
            .zoom_token_maintenance_scheduler_enabled(),
        zoom_recording_sync_scheduler_enabled: state.config.zoom_recording_sync_scheduler_enabled(),
        zoom_retention_cleanup_scheduler_enabled: state
            .config
            .zoom_retention_cleanup_scheduler_enabled(),
        vault: state.vault.clone(),
        telegram_runtime: state.telegram_runtime.clone(),
        event_bus: state.event_bus.clone(),
    };
    ApplicationComponents { state, bootstrap }
}

pub fn build_router(config: AppConfig) -> Router {
    build_router_with_database(config, Database::disabled())
}

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    build_router_from_state(compose_application(config, database).state)
}

pub(crate) fn build_router_from_state(state: AppState) -> Router {
    let api_secret = state
        .config
        .local_api_secret()
        .unwrap_or_default()
        .to_owned();

    let connect_routes = crate::app::connectrpc::protected_routes(
        state.database.pool().cloned(),
        state.config.clone(),
        state.vault.clone(),
        api_secret.clone(),
    );

    Router::<AppState>::new()
        .merge(routes::public_routes())
        .merge(connect_routes)
        .merge(routes::protected_routes(api_secret))
        .with_state(state)
        .layer(local_frontend_cors_layer())
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    service: String,
}

#[derive(Serialize)]
pub(crate) struct ReadinessResponse {
    status: &'static str,
    service: String,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
pub(crate) struct ReadinessChecks {
    database: models::DatabaseReadiness,
    migrations: models::MigrationReadiness,
}

pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let http_addr = config.http_addr();
    let database = Database::connect(config.database_url()).await?;
    let listener = TcpListener::bind(http_addr).await?;

    tracing::info!(%http_addr, "starting Макошь backend");

    let components = compose_application(config, database);
    let runtime = crate::app::runtime::start_application_runtime(&components);
    let termination = runtime.termination_signal();
    let server_result = makosh_api::serve(
        listener,
        build_router_from_state(components.state),
        termination,
    )
    .await;
    let runtime_failure = runtime.shutdown().await;
    server_result?;
    if let Some(error) = runtime_failure {
        return Err(io::Error::other(error).into());
    }

    Ok(())
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let log_format = std::env::var("MAKOSH_LOG_FORMAT").unwrap_or_else(|_| "plain".to_owned());

    if log_format.eq_ignore_ascii_case("json") {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_current_span(true)
            .with_span_list(false)
            .flatten_event(true)
            .try_init();
        return;
    }

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

pub(crate) fn local_frontend_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin
                .to_str()
                .map(is_allowed_local_frontend_origin)
                .unwrap_or(false)
        }))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("connect-accept-encoding"),
            HeaderName::from_static("connect-content-encoding"),
            HeaderName::from_static("connect-protocol-version"),
            HeaderName::from_static("connect-timeout-ms"),
            header::CONTENT_TYPE,
            HeaderName::from_static("last-event-id"),
            HeaderName::from_static("x-makosh-actor-id"),
            HeaderName::from_static("x-makosh-secret"),
        ])
}

fn is_allowed_local_frontend_origin(origin: &str) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };

    matches!(
        (url.scheme(), url.host_str()),
        (
            "http" | "https",
            Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
        ) | ("http" | "https", Some("tauri.localhost"))
            | ("tauri", Some("localhost"))
    )
}

pub(crate) async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name().to_owned(),
    })
}

pub(crate) async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let database = state.database.readiness().await;
    let migrations = state.database.migration_readiness().await;
    let is_ready = database.status() == models::ReadinessStatus::Ok
        && migrations.status() == models::ReadinessStatus::Ok;

    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status: if is_ready { "ok" } else { "degraded" },
            service: state.config.service_name().to_owned(),
            checks: ReadinessChecks {
                database,
                migrations,
            },
        }),
    )
}

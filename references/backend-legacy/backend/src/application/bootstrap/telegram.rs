use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::Value;
use serde_json::json;
use sqlx::postgres::PgPool;
use tokio_util::sync::CancellationToken;

use super::{ApplicationBootstrapContext, runtime_allows_processing};
use crate::integrations::telegram::runtime::manager::TelegramRuntimeManager;
use crate::integrations::telegram::runtime::manager::command_executor::execute_queued_commands;
use crate::platform::config::app_config::AppConfig;
use crate::platform::events::bus::InMemoryEventBus;
use crate::vault::HostVault;

static TELEGRAM_COMMAND_EXECUTOR_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static TELEGRAM_RUNTIME_RECONCILIATION_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
const TELEGRAM_COMMAND_EXECUTOR_RUNTIME: &str = "telegram_command_executor";
const TELEGRAM_RUNTIME_RECONCILIATION_RUNTIME: &str = "telegram_runtime_reconciliation";
const TELEGRAM_RUNTIME_RECONCILIATION_TICK_SECONDS: u64 = 15;
const TELEGRAM_RUNTIME_CHAT_SYNC_LIMIT: i64 = 100;
const TELEGRAM_RUNTIME_RECONNECT_INITIAL_DELAY_SECONDS: u64 = 5;
const TELEGRAM_RUNTIME_RECONNECT_MAX_DELAY_SECONDS: u64 = 300;
pub(crate) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        telegram_command_executor_task(context.clone()),
        telegram_runtime_reconciliation_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("telegram"))
    .collect()
}

fn telegram_command_executor_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_telegram_command_executor(&database_url) {
        return None;
    }
    let runtime_pool = pool.clone();
    let runtime = context.telegram_runtime;
    let event_bus = context.event_bus;
    let telegram_store = crate::integrations::telegram::client::store::TelegramStore::new(
        pool.clone(),
        Arc::new(
            makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            makosh_communications_postgres::store::CommunicationIngestionStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(
                pool.clone(),
            ),
        ),
    );

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let runtime_pool = runtime_pool.clone();
        let runtime = runtime.clone();
        let event_bus = event_bus.clone();
        let telegram_store = telegram_store.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(5));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &runtime_pool,
                    "telegram",
                    TELEGRAM_COMMAND_EXECUTOR_RUNTIME,
                    json!({
                        "label": "Telegram command executor",
                        "scope": "runtime",
                    }),
                )
                .await
                {
                    continue;
                }
                execute_queued_commands(&telegram_store, &runtime, &event_bus, 10).await;
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        TELEGRAM_COMMAND_EXECUTOR_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn telegram_runtime_reconciliation_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_telegram_runtime_reconciliation(&database_url) {
        return None;
    }

    let vault = context.vault;
    let config = context.config;
    let event_bus = context.event_bus;
    let runtime = context.telegram_runtime;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        let config = config.clone();
        let event_bus = event_bus.clone();
        let runtime = runtime.clone();
        Box::pin(async move {
            let mut reconnects = HashMap::<String, RuntimeReconnectState>::new();
            let mut tick = tokio::time::interval(Duration::from_secs(
                TELEGRAM_RUNTIME_RECONCILIATION_TICK_SECONDS,
            ));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "telegram",
                    TELEGRAM_RUNTIME_RECONCILIATION_RUNTIME,
                    json!({
                        "label": "Telegram runtime reconciliation",
                        "scope": "runtime",
                    }),
                )
                .await
                {
                    continue;
                }

                if let Err(error) = reconcile_runtime_once(
                    &pool,
                    &vault,
                    &config,
                    &event_bus,
                    &runtime,
                    &mut reconnects,
                )
                .await
                {
                    tracing::warn!(
                        error = %error,
                        "telegram runtime reconciliation tick failed"
                    );
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        TELEGRAM_RUNTIME_RECONCILIATION_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

pub(super) fn register_telegram_command_executor(database_url: &str) -> bool {
    match TELEGRAM_COMMAND_EXECUTOR_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "telegram command executor registry is unavailable"
            );
            false
        }
    }
}

pub(super) fn register_telegram_runtime_reconciliation(database_url: &str) -> bool {
    match TELEGRAM_RUNTIME_RECONCILIATION_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "telegram runtime reconciliation registry is unavailable"
            );
            false
        }
    }
}

#[derive(Default)]
pub(super) struct RuntimeReconnectState {
    consecutive_failures: u32,
    retry_after: Option<tokio::time::Instant>,
}

pub(super) async fn reconcile_runtime_once(
    pool: &PgPool,
    vault: &HostVault,
    config: &AppConfig,
    event_bus: &InMemoryEventBus,
    runtime: &TelegramRuntimeManager,
    reconnects: &mut HashMap<String, RuntimeReconnectState>,
) -> Result<(), String> {
    let account_store =
        makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            pool.clone(),
        );
    let accounts = account_store
        .list()
        .await
        .map_err(|error| error.to_string())?;
    let enabled_account_ids = accounts
        .iter()
        .filter(|account| runtime_reconciliation_enabled(account))
        .map(|account| account.account_id.as_str())
        .collect::<HashSet<_>>();

    for account in accounts
        .iter()
        .filter(|account| account.provider_kind.is_telegram())
        .filter(|account| !enabled_account_ids.contains(account.account_id.as_str()))
    {
        if let Err(error) = runtime.stop_account(&account.account_id) {
            tracing::warn!(
                error = %error,
                account_id = %account.account_id,
                "telegram runtime reconciliation could not stop a disabled account"
            );
        }
        reconnects.remove(&account.account_id);
    }

    let runtime_context = crate::application::telegram_runtime::TelegramRuntimeUseCaseContext::new(
        crate::application::telegram_runtime::TelegramRuntimeUseCaseStores {
            provider_account_store:
                makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                    pool.clone(),
                ),
            provider_secret_binding_store:
                makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                    pool.clone(),
                ),
            telegram_store:
                crate::application::provider_runtime_factories::telegram_provider_runtime_store(
                    pool.clone(),
                ),
            secret_store: crate::platform::secrets::store::SecretReferenceStore::new(pool.clone()),
        },
        crate::application::telegram_runtime::TelegramRuntimeUseCaseRuntime {
            secret_resolver: vault,
            config,
            event_bus,
            runtime,
        },
    );

    for account in accounts
        .iter()
        .filter(|account| runtime_reconciliation_enabled(account))
    {
        let account_id = &account.account_id;
        let now = tokio::time::Instant::now();
        if reconnects
            .get(account_id)
            .and_then(|state| state.retry_after)
            .is_some_and(|retry_after| retry_after > now)
        {
            continue;
        }

        let status = match crate::application::telegram_runtime::runtime_status(
            &runtime_context,
            account_id,
        )
        .await
        {
            Ok(status) => status,
            Err(error) => {
                record_reconciliation_failure(reconnects, account_id, &error);
                continue;
            }
        };

        if status.status != "running" {
            let _ = runtime.stop_account(account_id);
            match crate::application::telegram_runtime::start_runtime(
                &runtime_context,
                &crate::integrations::telegram::runtime::models::TelegramRuntimeStartRequest {
                    account_id: account_id.clone(),
                },
            )
            .await
            {
                Ok(status) if status.status == "running" => {
                    tracing::info!(
                        account_id = %account_id,
                        runtime_kind = %status.runtime_kind,
                        "telegram runtime restored"
                    );
                }
                Ok(status) => {
                    record_reconciliation_failure(
                        reconnects,
                        account_id,
                        &format!("runtime entered `{}`", status.status),
                    );
                    continue;
                }
                Err(error) => {
                    record_reconciliation_failure(reconnects, account_id, &error);
                    continue;
                }
            }
        }

        match crate::application::telegram_runtime::sync_chats(
            &runtime_context,
            &crate::integrations::telegram::runtime::models::TelegramChatSyncRequest {
                account_id: account_id.clone(),
                limit: Some(TELEGRAM_RUNTIME_CHAT_SYNC_LIMIT),
            },
        )
        .await
        {
            Ok(report) => {
                reconnects.remove(account_id);
                if report.synced_count > 0 {
                    tracing::debug!(
                        account_id = %account_id,
                        synced_count = report.synced_count,
                        "telegram runtime chat fallback sync completed"
                    );
                }
            }
            Err(error) => {
                let _ = runtime.stop_account(account_id);
                record_reconciliation_failure(reconnects, account_id, &error);
            }
        }
    }

    Ok(())
}

pub(super) fn runtime_reconciliation_enabled(
    account: &makosh_communications_api::accounts::ProviderAccount,
) -> bool {
    if !account.provider_kind.is_telegram() || account.is_deleted() {
        return false;
    }

    let lifecycle_state = account
        .config
        .get("lifecycle_state")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|state| !state.is_empty())
        .unwrap_or("active");
    if lifecycle_state != "active" {
        return false;
    }

    if matches!(
        account.config.get("auth_state").and_then(Value::as_str),
        Some("logged_out" | "deleted")
    ) {
        return false;
    }

    if matches!(
        account
            .config
            .get("runtime_enabled")
            .and_then(Value::as_bool),
        Some(false)
    ) || matches!(
        account.config.get("sync_enabled").and_then(Value::as_bool),
        Some(false)
    ) {
        return false;
    }

    matches!(
        account.config.get("runtime").and_then(Value::as_str),
        Some("fixture" | "tdlib_qr_authorized")
    )
}

fn record_reconciliation_failure(
    reconnects: &mut HashMap<String, RuntimeReconnectState>,
    account_id: &str,
    error: &impl std::fmt::Display,
) {
    let state = reconnects.entry(account_id.to_owned()).or_default();
    state.consecutive_failures = state.consecutive_failures.saturating_add(1);
    let exponent = state.consecutive_failures.saturating_sub(1).min(6);
    let delay_seconds = TELEGRAM_RUNTIME_RECONNECT_INITIAL_DELAY_SECONDS
        .saturating_mul(2_u64.pow(exponent))
        .min(TELEGRAM_RUNTIME_RECONNECT_MAX_DELAY_SECONDS);
    state.retry_after = Some(tokio::time::Instant::now() + Duration::from_secs(delay_seconds));

    if matches!(state.consecutive_failures, 1 | 3 | 6) {
        tracing::warn!(
            account_id = %account_id,
            consecutive_failures = state.consecutive_failures,
            retry_after_seconds = delay_seconds,
            error = %error,
            "telegram runtime reconciliation failed; retrying without degrading the system"
        );
    }
}

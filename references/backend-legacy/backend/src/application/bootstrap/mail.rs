//! Mail-owned runtime task factories and duplicate-registration guards.

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use chrono::Utc;
use makosh_desktop_runtime::{
    RuntimeExitPolicy, RuntimeTaskClass, RuntimeTaskFactory, RuntimeTaskFuture, RuntimeTaskSpec,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::{ApplicationBootstrapContext, host_vault_is_unlocked, runtime_allows_processing};

static MAIL_BACKGROUND_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_ATTACHMENT_SCAN_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static ADDRESS_BOOK_SYNC_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_OUTBOX_DELIVERY_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_PROVIDER_COMMAND_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
static MAIL_AI_PIPELINE_DATABASES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));
const MAIL_BACKGROUND_SYNC_RUNTIME: &str = "mail_background_sync";
const MAIL_ATTACHMENT_SCAN_RUNTIME: &str = "mail_attachment_scan";
const ADDRESS_BOOK_SYNC_RUNTIME: &str = "address_book_sync";
const MAIL_OUTBOX_DELIVERY_RUNTIME: &str = "mail_outbox_delivery";
const MAIL_PROVIDER_COMMAND_RUNTIME: &str = "mail_provider_commands";
const MAIL_AI_PIPELINE_RUNTIME: &str = "mail_ai_pipeline";
pub(crate) fn runtime_task_specs(context: ApplicationBootstrapContext) -> Vec<RuntimeTaskSpec> {
    [
        mail_background_sync_task(context.clone()),
        mail_attachment_scan_task(context.clone()),
        crate::application::mail_imap_idle::runtime_task_spec(context.clone()),
        address_book_sync_task(context.clone()),
        mail_outbox_delivery_task(context.clone()),
        mail_provider_command_executor_task(context.clone()),
        mail_ai_pipeline_task(context),
    ]
    .into_iter()
    .flatten()
    .map(|task| task.with_lifecycle_source("mail"))
    .collect()
}

fn mail_background_sync_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_background_sync_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let store =
                crate::workflows::mail_background_sync::store::MailSyncStore::new(pool.clone());
            let service = crate::workflows::mail_background_sync::service::MailBackgroundSyncService::new(
            pool.clone(),
            vault.clone(),
            crate::platform::communications::DEFAULT_MAIL_SYNC_BLOB_ROOT,
            Arc::new(
                crate::integrations::mail::sync_provider::LiveEmailProviderSyncPort::new(
                    pool.clone(),
                    vault,
                    Arc::new(
                        makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                            pool.clone(),
                        ),
                    ),
                    crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                ),
            ),
            Arc::new(
                crate::domains::communications::provider_resources::MailProviderResourceStore::new(
                    pool.clone(),
                ),
            ),
            Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
            Arc::new(
                makosh_communications_postgres::store::CommunicationIngestionStore::new(
                    pool.clone(),
                ),
            ),
        );
            if let Err(error) = store.recover_interrupted_runs(Utc::now()).await {
                tracing::warn!(error = %error, "mail background sync startup recovery failed");
            }
            let mut tick = tokio::time::interval(Duration::from_secs(30));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "mail",
                    MAIL_BACKGROUND_SYNC_RUNTIME,
                    json!({
                        "label": "Mail background sync",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                if let Err(error) = service.run_due_accounts().await {
                    tracing::warn!(error = %error, "mail background sync scheduler tick failed");
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_BACKGROUND_SYNC_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_attachment_scan_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_attachment_scan_worker(&database_url) {
        return None;
    }

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        Box::pin(async move {
            let worker =
                crate::application::mail_attachment_scan_worker::MailAttachmentScanWorker::new(
                    pool.clone(),
                );
            let mut tick = tokio::time::interval(Duration::from_secs(60));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "mail",
                    MAIL_ATTACHMENT_SCAN_RUNTIME,
                    json!({
                        "label": "Mail attachment malware rescan",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                match worker.scan_due(50).await {
                    Ok(report)
                        if report.candidates_seen > 0
                            || report.verdicts_persisted > 0
                            || report.failures > 0 =>
                    {
                        tracing::info!(
                            candidates_seen = report.candidates_seen,
                            verdicts_persisted = report.verdicts_persisted,
                            retry_deferred = report.retry_deferred,
                            invalid_or_stale = report.invalid_or_stale,
                            failures = report.failures,
                            "mail attachment rescan tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "mail attachment rescan scheduler tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_ATTACHMENT_SCAN_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn address_book_sync_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_address_book_sync_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let service = crate::workflows::address_book_sync::AddressBookSyncService::new(
            pool.clone(),
            Arc::new(
                crate::integrations::mail::address_book_sync_provider::LiveAddressBookProviderSyncPort::new(
                    pool.clone(),
                    vault,
                    Arc::new(
                        makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                            pool.clone(),
                        ),
                    ),
                    crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                ),
            ),
            Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
        );
            let mut tick = tokio::time::interval(Duration::from_secs(300));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "mail",
                    ADDRESS_BOOK_SYNC_RUNTIME,
                    json!({
                        "label": "Address book sync",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                if let Err(error) = service.run_due_accounts().await {
                    tracing::warn!(error = %error, "address book sync scheduler tick failed");
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        ADDRESS_BOOK_SYNC_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_outbox_delivery_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_outbox_delivery_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(10));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "mail",
                    MAIL_OUTBOX_DELIVERY_RUNTIME,
                    json!({
                        "label": "Mail outbox delivery",
                        "scope": "scheduler",
                    }),
                )
                .await
                {
                    continue;
                }
                if !host_vault_is_unlocked(&vault) {
                    continue;
                }
                let store = crate::domains::communications::outbox::CommunicationOutboxStore::new(
                    pool.clone(),
                );
                let sender =
                    crate::domains::communications::outbox::provider_sender::CommunicationOutboxEmailSender::new(
                        pool.clone(),
                        vault.clone(),
                        crate::integrations::mail::send::smtp_outbox_transport(),
                        crate::integrations::mail::outbox::gmail_outbox_transport(
                            pool.clone(),
                            vault.clone(),
                        ),
                    );
                let worker = crate::domains::communications::outbox::delivery::EmailOutboxDeliveryWorker::new(
                    store, sender,
                );
                match worker.deliver_due(Utc::now(), 25).await {
                    Ok(report) if report.claimed > 0 => {
                        tracing::info!(
                            claimed = report.claimed,
                            sent = report.sent,
                            failed = report.failed,
                            retried = report.retried,
                            "mail outbox delivery scheduler tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "mail outbox delivery scheduler tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_OUTBOX_DELIVERY_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_provider_command_executor_task(
    context: ApplicationBootstrapContext,
) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_provider_command_scheduler(&database_url) {
        return None;
    }
    let vault = context.vault;
    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let worker =
                crate::application::mail_provider_command_executor::MailProviderCommandWorker::new(
                    pool.clone(),
                    vault.clone(),
                    crate::workflows::mail_background_sync::DEFAULT_GMAIL_API_BASE_URL,
                );
            // Read-state intent is queued after the reader dwell threshold, so keep
            // provider reconciliation responsive without polling full mailbox sync.
            let mut tick = tokio::time::interval(Duration::from_secs(2));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                &pool,
                "mail",
                MAIL_PROVIDER_COMMAND_RUNTIME,
                json!({ "label": "Mail provider command reconciliation", "scope": "scheduler" }),
            )
            .await
                || !host_vault_is_unlocked(&vault)
            {
                continue;
            }
                if let Err(error) = worker.execute_due(Utc::now(), 25).await {
                    tracing::warn!(error = %error, "mail provider command scheduler tick failed");
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_PROVIDER_COMMAND_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn mail_ai_pipeline_task(context: ApplicationBootstrapContext) -> Option<RuntimeTaskSpec> {
    let pool = context.pool?;
    let database_url = context.database_url?;
    if !register_mail_ai_pipeline(&database_url) {
        return None;
    }
    let config = context.config;
    let vault = context.vault;

    let task: RuntimeTaskFactory = Arc::new(move |cancellation: CancellationToken| {
        let pool = pool.clone();
        let config = config.clone();
        let vault = vault.clone();
        Box::pin(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(10));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = cancellation.cancelled() => return Ok(()),
                    _ = tick.tick() => {}
                }
                if !runtime_allows_processing(
                    &pool,
                    "ai",
                    MAIL_AI_PIPELINE_RUNTIME,
                    json!({
                        "label": "Mail AI pipeline",
                        "scope": "worker",
                    }),
                )
                .await
                {
                    continue;
                }

                let mail_ai_runtime =
                    super::mail_ai::mail_ai_hub_optional(&pool, &config, &vault).await;
                let (hub, external_body_egress_required) = match mail_ai_runtime {
                    Some((hub, external_body_egress_required)) => {
                        (Some(hub), external_body_egress_required)
                    }
                    None => (None, false),
                };
                let owner_query =
                    crate::application::persona_owner_query::PostgresPersonaOwnerQuery::new(
                        pool.clone(),
                    );
                let target_language = super::mail_ai::mail_ai_target_language(&owner_query).await;
                let service = crate::workflows::email_intelligence::pipeline::MailAiPipelineService::new(
                    pool.clone(),
                    hub,
                    target_language,
                    std::sync::Arc::new(
                        crate::domains::communications::sensitive_forwarding::SensitiveForwardingPgStore::new(
                            pool.clone(),
                        ),
                    ),
                )
                .requiring_external_body_egress(external_body_egress_required);
                match service.process_next_batch(10).await {
                    Ok(report)
                        if report.claimed > 0
                            || report.recovered > 0
                            || report.processed > 0
                            || report.failed > 0
                            || report.retrying > 0
                            || report.suppressed > 0 =>
                    {
                        tracing::info!(
                            claimed = report.claimed,
                            recovered = report.recovered,
                            processed = report.processed,
                            suppressed = report.suppressed,
                            failed = report.failed,
                            retrying = report.retrying,
                            review_candidates = report.review_candidates,
                            "mail AI pipeline tick completed"
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(error = %error, "mail AI pipeline tick failed");
                    }
                }
            }
        }) as RuntimeTaskFuture
    });
    Some(RuntimeTaskSpec::new(
        MAIL_AI_PIPELINE_RUNTIME,
        RuntimeTaskClass::Background,
        RuntimeExitPolicy::MarkDegraded,
        task,
    ))
}

fn register_mail_background_sync_scheduler(database_url: &str) -> bool {
    match MAIL_BACKGROUND_SYNC_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail background sync scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_mail_attachment_scan_worker(database_url: &str) -> bool {
    match MAIL_ATTACHMENT_SCAN_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(error = %error, "mail attachment rescan scheduler registry is unavailable");
            false
        }
    }
}

fn register_address_book_sync_scheduler(database_url: &str) -> bool {
    match ADDRESS_BOOK_SYNC_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "address book sync scheduler registry is unavailable"
            );
            false
        }
    }
}

pub(super) fn register_mail_outbox_delivery_scheduler(database_url: &str) -> bool {
    match MAIL_OUTBOX_DELIVERY_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail outbox delivery scheduler registry is unavailable"
            );
            false
        }
    }
}

fn register_mail_provider_command_scheduler(database_url: &str) -> bool {
    match MAIL_PROVIDER_COMMAND_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(error = %error, "mail provider command scheduler registry is unavailable");
            false
        }
    }
}

fn register_mail_ai_pipeline(database_url: &str) -> bool {
    match MAIL_AI_PIPELINE_DATABASES.lock() {
        Ok(mut databases) => databases.insert(database_url.to_owned()),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "mail AI pipeline registry is unavailable"
            );
            false
        }
    }
}

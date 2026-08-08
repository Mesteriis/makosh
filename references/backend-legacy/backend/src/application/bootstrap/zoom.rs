use chrono::Utc;
use makosh_provider_zoom::protocol::{
    ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS, ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE,
};
use sqlx::postgres::PgPool;

use crate::integrations::zoom::client::models::oauth_models::ZoomTokenMaintenanceRequest;
use crate::integrations::zoom::client::models::oauth_models::ZoomTokenMaintenanceResult;
use crate::platform::events::bus::InMemoryEventBus;
use crate::vault::HostVault;

pub(crate) mod tasks;

const ZOOM_TOKEN_MAINTENANCE_REFRESH_EXPIRING_WITHIN_SECONDS: i64 = 300;
const ZOOM_RECORDING_SYNC_LOOKBACK_DAYS: i64 = 7;
const ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT: i64 = 100;
pub(super) async fn run_zoom_token_maintenance_once(
    pool: &PgPool,
    vault: &HostVault,
    event_bus: &InMemoryEventBus,
) -> Result<ZoomTokenMaintenanceResult, String> {
    let secret_store = crate::platform::secrets::store::SecretReferenceStore::new(pool.clone());
    let service = crate::application::provider_runtime_factories::zoom_provider_runtime_service(
        pool.clone(),
        event_bus.clone(),
    );
    let request = ZoomTokenMaintenanceRequest {
        account_id: None,
        force: false,
        refresh_expiring_within_seconds: Some(
            ZOOM_TOKEN_MAINTENANCE_REFRESH_EXPIRING_WITHIN_SECONDS,
        ),
    };
    service
        .maintain_tokens(&secret_store, vault, &request)
        .await
        .map_err(|error| error.to_string())
}

pub(super) struct ZoomRecordingSyncSchedulerResult {
    pub(super) accounts_checked: usize,
    pub(super) accounts_synced: usize,
    pub(super) accounts_skipped: usize,
    pub(super) failed_count: usize,
    pub(super) meetings_recorded: usize,
    pub(super) recordings_recorded: usize,
    pub(super) media_downloads_recorded: usize,
    pub(super) transcripts_recorded: usize,
    pub(super) lookback_days: i64,
}

pub(super) struct ZoomRetentionCleanupSchedulerResult {
    pub(super) accounts_checked: usize,
    pub(super) accounts_cleaned: usize,
    pub(super) recordings_removed: usize,
    pub(super) transcripts_removed: usize,
    pub(super) limit_per_account: i64,
}

pub(super) async fn run_zoom_recording_sync_once(
    pool: &PgPool,
    vault: &HostVault,
    event_bus: &InMemoryEventBus,
) -> Result<ZoomRecordingSyncSchedulerResult, String> {
    let settings = crate::platform::settings::store::ApplicationSettingsStore::new(pool.clone());
    let allow_remote_transcript_downloads = settings
        .setting("privacy.zoom_remote_transcript_download_enabled")
        .await
        .map_err(|error| error.to_string())?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false);
    let allow_remote_recording_downloads = settings
        .setting("privacy.zoom_remote_recording_download_enabled")
        .await
        .map_err(|error| error.to_string())?
        .and_then(|setting| setting.value.as_bool())
        .unwrap_or(false);

    let secret_store = crate::platform::secrets::store::SecretReferenceStore::new(pool.clone());
    let service = crate::application::provider_runtime_factories::zoom_provider_runtime_service(
        pool.clone(),
        event_bus.clone(),
    );
    let accounts = service
        .list_accounts(false)
        .await
        .map_err(|error| error.to_string())?
        .items;
    let today = Utc::now().date_naive();
    let from = (today - chrono::TimeDelta::days(ZOOM_RECORDING_SYNC_LOOKBACK_DAYS))
        .format("%Y-%m-%d")
        .to_string();
    let to = today.format("%Y-%m-%d").to_string();
    let mut result = ZoomRecordingSyncSchedulerResult {
        accounts_checked: 0,
        accounts_synced: 0,
        accounts_skipped: 0,
        failed_count: 0,
        meetings_recorded: 0,
        recordings_recorded: 0,
        media_downloads_recorded: 0,
        transcripts_recorded: 0,
        lookback_days: ZOOM_RECORDING_SYNC_LOOKBACK_DAYS,
    };

    for account in accounts {
        if !account.provider_kind.starts_with("zoom_") {
            continue;
        }
        result.accounts_checked += 1;
        let status = service
            .runtime_status(&account.account_id)
            .await
            .map_err(|error| error.to_string())?;
        if !should_run_zoom_recording_sync(&status) {
            result.accounts_skipped += 1;
            continue;
        }
        let request = crate::integrations::zoom::client::models::ZoomRecordingSyncRequest {
            account_id: account.account_id.clone(),
            user_id: None,
            from: from.clone(),
            to: to.clone(),
            page_size: Some(ZOOM_PROVIDER_SYNC_DEFAULT_PAGE_SIZE),
            max_meetings: Some(ZOOM_PROVIDER_SYNC_DEFAULT_MAX_MEETINGS),
            api_base_url: None,
        };
        match service
            .sync_recordings(
                &secret_store,
                vault,
                &request,
                allow_remote_recording_downloads,
                allow_remote_transcript_downloads,
            )
            .await
        {
            Ok(sync) => {
                result.accounts_synced += 1;
                result.meetings_recorded += sync.meetings_recorded;
                result.recordings_recorded += sync.recordings_recorded;
                result.media_downloads_recorded += sync.media_downloads_recorded;
                result.transcripts_recorded += sync.transcripts_recorded;
                if !sync.failures.is_empty() {
                    result.failed_count += 1;
                }
            }
            Err(error) => {
                result.failed_count += 1;
                tracing::warn!(
                    error = %error,
                    account_id = %account.account_id,
                    "zoom recording sync failed for authorized runtime account"
                );
            }
        }
    }

    Ok(result)
}

pub(super) async fn run_zoom_retention_cleanup_once(
    pool: &PgPool,
    event_bus: &InMemoryEventBus,
) -> Result<ZoomRetentionCleanupSchedulerResult, String> {
    let service = crate::application::provider_runtime_factories::zoom_provider_runtime_service(
        pool.clone(),
        event_bus.clone(),
    );
    let accounts = service
        .list_accounts(false)
        .await
        .map_err(|error| error.to_string())?
        .items;
    let mut result = ZoomRetentionCleanupSchedulerResult {
        accounts_checked: 0,
        accounts_cleaned: 0,
        recordings_removed: 0,
        transcripts_removed: 0,
        limit_per_account: ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
    };

    for account in accounts {
        if !account.provider_kind.starts_with("zoom_") {
            continue;
        }
        result.accounts_checked += 1;
        let response = service
            .cleanup_retention(
                &account.account_id,
                &crate::integrations::zoom::client::models::ZoomRetentionCleanupRequest {
                    remove_recordings: true,
                    remove_transcripts: true,
                    limit: ZOOM_RETENTION_CLEANUP_LIMIT_PER_ACCOUNT,
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        if response.recordings_removed > 0 || response.transcripts_removed > 0 {
            result.accounts_cleaned += 1;
        }
        result.recordings_removed += response.recordings_removed;
        result.transcripts_removed += response.transcripts_removed;
    }

    Ok(result)
}
pub(super) fn should_run_zoom_recording_sync(
    status: &crate::integrations::zoom::client::models::ZoomRuntimeStatus,
) -> bool {
    status.live_runtime_available
        && matches!(status.status.as_str(), "running" | "degraded")
        && !status
            .runtime_blockers
            .iter()
            .any(|blocker| blocker == "zoom_token_rotation_required")
}

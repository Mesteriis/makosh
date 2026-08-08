use super::super::*;
use super::database::database_pool;
use crate::platform::calls::store::CallIntelligenceStore;
use crate::platform::secrets::store::SecretReferenceStore;
use std::sync::Arc;

use crate::app::api_support::stores::domain_stores::{api_audit_log, event_store};

use crate::application::provider_runtime_factories::{
    telegram_provider_runtime_service as build_telegram_provider_runtime_service,
    telegram_provider_runtime_store, whatsapp_provider_runtime,
    whatsapp_provider_runtime_service as build_whatsapp_provider_runtime_service,
    yandex_telemost_provider_runtime_service as build_yandex_telemost_provider_runtime_service,
    zoom_provider_runtime_service as build_zoom_provider_runtime_service,
};
use crate::application::provider_runtime_services::{
    TelegramProviderRuntimeApplicationService, WhatsAppProviderRuntimeRef,
    WhatsappProviderRuntimeApplicationService, YandexTelemostProviderRuntimeApplicationService,
    ZoomProviderRuntimeApplicationService,
};

fn build_telegram_provider_store(
    state: &AppState,
) -> Result<crate::integrations::telegram::client::store::TelegramStore, ApiError> {
    Ok(telegram_provider_runtime_store(database_pool(state)?))
}

pub(crate) fn telegram_provider_runtime_service(
    state: &AppState,
) -> Result<TelegramProviderRuntimeApplicationService, ApiError> {
    Ok(build_telegram_provider_runtime_service(database_pool(
        state,
    )?))
}

pub(crate) fn telegram_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn whatsapp_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn zulip_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn zoom_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn telegram_runtime_use_case_context(
    state: &AppState,
) -> Result<crate::application::telegram_runtime::TelegramRuntimeUseCaseContext<'_>, ApiError> {
    let pool = database_pool(state)?;
    Ok(
        crate::application::telegram_runtime::TelegramRuntimeUseCaseContext::new(
            crate::application::telegram_runtime::TelegramRuntimeUseCaseStores {
                provider_account_store:
                    makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                        pool.clone(),
                    ),
                provider_secret_binding_store:
                    makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                        pool.clone(),
                    ),
                telegram_store: build_telegram_provider_store(state)?,
                secret_store: SecretReferenceStore::new(pool),
            },
            crate::application::telegram_runtime::TelegramRuntimeUseCaseRuntime {
                secret_resolver: &state.vault,
                config: &state.config,
                event_bus: &state.event_bus,
                runtime: &state.telegram_runtime,
            },
        ),
    )
}

pub(crate) fn telegram_message_write_service(
    state: &AppState,
) -> Result<
    crate::application::communication_provider_writes::TelegramMessageWriteApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_provider_writes::TelegramMessageWriteApplicationService::new(
            build_telegram_provider_store(state)?,
            Arc::new(makosh_communications_postgres::canonical::CanonicalMessageReadStore::new(
                database_pool(state)?,
            )),
            api_audit_log(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn telegram_fixture_ingest_service(
    state: &AppState,
) -> Result<
    crate::application::telegram_fixture_ingest::TelegramFixtureIngestApplicationService,
    ApiError,
> {
    Ok(
        crate::application::telegram_fixture_ingest::TelegramFixtureIngestApplicationService::new(
            database_pool(state)?,
            crate::integrations::telegram::client::fixture_port::TelegramFixturePort::new(
                build_telegram_provider_store(state)?,
            ),
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

fn build_whatsapp_provider_store(state: &AppState) -> Result<WhatsAppProviderRuntimeRef, ApiError> {
    Ok(whatsapp_provider_runtime(database_pool(state)?))
}

pub(crate) fn whatsapp_provider_runtime_service(
    state: &AppState,
) -> Result<WhatsappProviderRuntimeApplicationService, ApiError> {
    Ok(build_whatsapp_provider_runtime_service(database_pool(
        state,
    )?))
}

pub(crate) fn zoom_provider_runtime_service(
    state: &AppState,
) -> Result<ZoomProviderRuntimeApplicationService, ApiError> {
    Ok(build_zoom_provider_runtime_service(
        database_pool(state)?,
        state.event_bus.clone(),
    ))
}

pub(crate) fn yandex_telemost_secret_reference_store(
    state: &AppState,
) -> Result<SecretReferenceStore, ApiError> {
    Ok(SecretReferenceStore::new(database_pool(state)?))
}

pub(crate) fn yandex_telemost_provider_runtime_store(
    state: &AppState,
) -> Result<crate::integrations::yandex_telemost::client::store::YandexTelemostStore, ApiError> {
    let pool = database_pool(state)?;
    Ok(
        crate::integrations::yandex_telemost::client::store::YandexTelemostStore::new(
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
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn yandex_telemost_provider_runtime_service(
    state: &AppState,
) -> Result<YandexTelemostProviderRuntimeApplicationService, ApiError> {
    Ok(build_yandex_telemost_provider_runtime_service(
        database_pool(state)?,
        state.event_bus.clone(),
    ))
}

pub(crate) fn whatsapp_fixture_ingest_service(
    state: &AppState,
) -> Result<
    crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService,
    ApiError,
> {
    Ok(
        crate::application::communication_fixture_ingest::WhatsappFixtureIngestApplicationService::new(
            database_pool(state)?,
            build_whatsapp_provider_store(state)?,
            event_store(state)?,
            state.event_bus.clone(),
        ),
    )
}

pub(crate) fn automation_store(state: &AppState) -> Result<AutomationStore, ApiError> {
    Ok(AutomationStore::new(database_pool(state)?))
}

pub(crate) fn call_intelligence_store(state: &AppState) -> Result<CallIntelligenceStore, ApiError> {
    Ok(CallIntelligenceStore::new(database_pool(state)?))
}

pub(crate) fn account_setup_service(
    state: &AppState,
) -> Result<EmailAccountSetupService, ApiError> {
    let pool = database_pool(state)?;
    Ok(EmailAccountSetupService::new_with_host_vault(
        pool.clone(),
        SecretReferenceStore::new(pool.clone()),
        state.vault.clone(),
        Arc::new(
            makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
                pool.clone(),
            ),
        ),
        Arc::new(
            makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(
                pool,
            ),
        ),
    ))
}

use sqlx::postgres::PgPool;
use std::sync::Arc;

use crate::application::provider_runtime_services::{
    TelegramProviderRuntimeApplicationService, WhatsAppProviderRuntimeRef,
    WhatsappProviderRuntimeApplicationService, YandexTelemostProviderRuntimeApplicationService,
    ZoomProviderRuntimeApplicationService,
};
use crate::integrations::yandex_telemost::client::store::YandexTelemostStore;
use crate::platform::events::bus::InMemoryEventBus;

pub(crate) fn telegram_provider_runtime_service(
    pool: PgPool,
) -> TelegramProviderRuntimeApplicationService {
    TelegramProviderRuntimeApplicationService::new(telegram_provider_runtime_store(pool))
}

pub(crate) fn whatsapp_provider_runtime_service(
    pool: PgPool,
) -> WhatsappProviderRuntimeApplicationService {
    WhatsappProviderRuntimeApplicationService::new(whatsapp_provider_runtime(pool))
}

pub(crate) fn zoom_provider_runtime_service(
    pool: PgPool,
    event_bus: InMemoryEventBus,
) -> ZoomProviderRuntimeApplicationService {
    ZoomProviderRuntimeApplicationService::new(zoom_provider_runtime_store(pool, event_bus))
}

pub(crate) fn yandex_telemost_provider_runtime_service(
    pool: PgPool,
    event_bus: InMemoryEventBus,
) -> YandexTelemostProviderRuntimeApplicationService {
    YandexTelemostProviderRuntimeApplicationService::new(YandexTelemostStore::new(
        Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(pool.clone())),
        makosh_events_postgres::store::EventStore::new(pool), event_bus,
    ))
}

pub(crate) fn telegram_provider_runtime_store(
    pool: PgPool,
) -> crate::integrations::telegram::client::store::TelegramStore {
    crate::integrations::telegram::client::store::TelegramStore::new(
        pool.clone(),
        Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(crate::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore::new(pool.clone())),
        Arc::new(makosh_communications_postgres::store::CommunicationIngestionStore::new(pool.clone())),
        Arc::new(crate::platform::communications::EventStoreProviderMessageObservationEventPort::new(pool)),
    )
}

pub(crate) fn whatsapp_provider_runtime(pool: PgPool) -> WhatsAppProviderRuntimeRef {
    let provider_account_store = Arc::new(
        makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(
            pool.clone(),
        ),
    );
    let provider_secret_binding_store = Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(pool.clone()));
    let provider_channel_message_store = Arc::new(crate::domains::communications::messages::provider_channel_store::ProviderChannelMessageStore::new(pool.clone()));
    let provider_command_mirror = Arc::new(
        makosh_communications_postgres::provider_commands::CommunicationProviderCommandStore::new(
            pool.clone(),
        ),
    );
    crate::integrations::whatsapp::runtime::whatsapp_web_companion_runtime(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
        provider_command_mirror,
    )
}

pub(crate) fn zoom_provider_runtime_store(
    pool: PgPool,
    event_bus: InMemoryEventBus,
) -> crate::integrations::zoom::client::store::ZoomStore {
    crate::integrations::zoom::client::store::ZoomStore::new(
        pool.clone(),
        Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderAccountStore::new(pool.clone())),
        Arc::new(makosh_communications_postgres::provider_store::CommunicationProviderSecretBindingStore::new(pool.clone())),
        Arc::new(crate::domains::communications::storage::store::CommunicationStorageStore::new(pool.clone())),
        crate::platform::calls::store::CallIntelligenceStore::new(pool.clone()),
        makosh_events_postgres::store::EventStore::new(pool), event_bus,
    )
}

//! Telegram-owned PostgreSQL persistence for operational projections and Communications outbox.

mod communications_outbox;
#[cfg(feature = "conformance-test-support")]
mod conformance;
mod delivery_intent;
mod delivery_intent_inbox;
mod delivery_intent_result_outbox;
mod durable;
mod owner_rls;
mod schema;

pub use communications_outbox::TelegramCommunicationsOutboxStoreV1;
#[cfg(feature = "conformance-test-support")]
pub use conformance::TelegramPersistenceConformanceV1;
pub use delivery_intent::{TELEGRAM_SCHEMA_V2, TelegramDeliveryRouteLocatorV1};
pub use delivery_intent_inbox::{
    ClaimedTelegramDeliveryIntentJobV1, TELEGRAM_DELIVERY_INTENT_MAX_ATTEMPTS_V1,
    TELEGRAM_SCHEMA_V3, TelegramDeliveryIntentAdmissionV1, TelegramDeliveryIntentInboxOutcomeV1,
    TelegramDeliveryIntentJobStateV1, TelegramDeliveryIntentJobV1, TelegramDeliveryIntentStoreV1,
};
pub use durable::{
    TELEGRAM_SCHEMA_V1, TelegramDurablePersistence, TelegramDurablePersistenceError,
};
pub use owner_rls::{
    TELEGRAM_OWNER_RLS_STORAGE_REVISION_V1, TELEGRAM_OWNER_RLS_TABLES_V1,
    telegram_owner_rls_sql_v1, telegram_owner_rls_storage_migration_v1,
};
pub use schema::{
    TELEGRAM_DELIVERY_INTENT_STORAGE_REVISION_V1, TELEGRAM_DELIVERY_ROUTE_STORAGE_REVISION_V1,
    TELEGRAM_STORAGE_BUNDLE_REVISION_V1, telegram_delivery_intent_storage_migration_v1,
    telegram_delivery_route_storage_migration_v1, telegram_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-telegram-persistence";

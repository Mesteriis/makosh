//! WhatsApp-owned durable storage. Communications receives only exact envelopes.

mod delivery_intent;
mod delivery_intent_result_outbox;
mod delivery_intent_store;
mod durable;
mod operational;
mod owner_rls;
mod schema;

pub use delivery_intent_store::{
    ClaimedWhatsAppDeliveryIntentJobV1, WHATSAPP_DELIVERY_INTENT_MAX_ATTEMPTS_V1,
    WHATSAPP_DELIVERY_INTENT_SCHEMA_V1, WhatsAppDeliveryIntentAdmissionV1,
    WhatsAppDeliveryIntentInboxOutcomeV1, WhatsAppDeliveryIntentJobStateV1,
    WhatsAppDeliveryIntentJobV1, WhatsAppDeliveryIntentStoreV1,
};
pub use durable::{
    WhatsAppClaimedCommandV1, WhatsAppDurablePersistence, WhatsAppDurablePersistenceError,
    WhatsAppHostObservationRecordV1, WhatsAppProviderCommandEnqueueV1,
    WhatsAppProviderCommandStateV1, WhatsAppProviderCommandStatusV1,
};
pub use operational::WhatsAppOperationalObservationV1;
pub use owner_rls::{
    WHATSAPP_OWNER_RLS_STORAGE_REVISION_V1, WHATSAPP_OWNER_RLS_TABLES_V1,
    whatsapp_owner_rls_sql_v1, whatsapp_owner_rls_storage_migration_v1,
};
pub use schema::{
    WHATSAPP_SCHEMA_V1, WHATSAPP_SCHEMA_V2, WHATSAPP_STORAGE_BUNDLE_REVISION_V1,
    WHATSAPP_STORAGE_BUNDLE_REVISION_V2, WHATSAPP_STORAGE_BUNDLE_REVISION_V3,
    WHATSAPP_STORAGE_BUNDLE_REVISION_V4, whatsapp_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-whatsapp-persistence";
pub use delivery_intent::{WHATSAPP_DELIVERY_ROUTE_SCHEMA_V1, WhatsAppDeliveryRouteLocatorV1};

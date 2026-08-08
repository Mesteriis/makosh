#![forbid(unsafe_code)]

mod identity;
mod model;
mod upsert;

pub use identity::{normalize_email_v1, normalize_phone_v1};
pub use model::{
    ContactIdentityMatchV1, ContactProviderKindV1, ContactProviderProvenanceV1, ContactTimestampV1,
    ContactUpsertDraftV1, ContactUpsertOutcomeV1, ContactV1, ContactsValidationErrorV1,
    derive_contact_id_v1, upsert_fingerprint_v1, validate_contact_v1,
};
pub use upsert::{ContactUpsertDecisionErrorV1, decide_contact_upsert_v1};

pub const PACKAGE: &str = "makosh-contacts-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_ACCOUNT_ID_BYTES_V1: usize = 256;
pub const MAX_PROVIDER_ENTRY_ID_BYTES_V1: usize = 512;
pub const MAX_PROVIDER_ETAG_BYTES_V1: usize = 512;
pub const MAX_DISPLAY_NAME_CHARS_V1: usize = 240;
pub const MAX_EMAILS_V1: usize = 32;
pub const MAX_PHONES_V1: usize = 32;

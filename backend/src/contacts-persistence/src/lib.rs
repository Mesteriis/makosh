#![forbid(unsafe_code)]

mod model;
mod provider_link;
mod provider_link_model;
mod repository;
mod schema;

#[cfg(feature = "conformance-test-support")]
mod conformance;

#[cfg(feature = "conformance-test-support")]
pub use conformance::ContactsPersistenceConformanceV1;

pub use model::{
    AppliedMailEntryCommandV1, ApplyMailEntryCommandV1, ContactMailEntryRejectCodeV1,
    ContactMailSyncSourceLinkV1, ContactMailSyncSourceRejectCodeV1, ContactMailSyncSourceResultV1,
    ContactMailSyncSourceSnapshotV1, ContactMutationOutboxV1, ContactsOutboxRecordV1,
    ContactsPersistenceErrorV1, PersistContactMailSyncSourceResultV1, RejectMailEntryCommandV1,
    RejectedMailEntryCommandV1, ReserveContactMailSyncSourceV1,
};
pub use provider_link_model::{
    AppliedMailProviderLinkCommandV1, BindMailProviderLinkCommandV1,
    ContactProviderLinkBindOutcomeV1, ContactProviderLinkBindRejectCodeV1,
};
pub use repository::ContactsPersistenceV1;
pub use schema::{
    CONTACTS_MAIL_PROVIDER_LINK_SCHEMA_V3, CONTACTS_MAIL_SYNC_SOURCE_SCHEMA_V2, CONTACTS_SCHEMA_V1,
    CONTACTS_STORAGE_BUNDLE_REVISION_V1, contacts_storage_bundle_v1,
};

pub const PACKAGE: &str = "makosh-contacts-persistence";

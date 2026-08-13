use std::collections::BTreeMap;

use crate::{DIGEST_BYTES_V1, STABLE_ID_BYTES_V1};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PublicIdV1(pub [u8; STABLE_ID_BYTES_V1]);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PersonIdV1(pub [u8; STABLE_ID_BYTES_V1]);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestV1(pub [u8; DIGEST_BYTES_V1]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonLifecycleV1 {
    Provisional,
    Active,
    Merged,
    Archived,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerProfileV1 {
    pub display_name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
}

impl OwnerProfileV1 {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.display_name.is_none()
            && self.given_name.is_none()
            && self.family_name.is_none()
            && self.emails.is_empty()
            && self.phones.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceLinkKeyV1 {
    pub integration_public_id: PublicIdV1,
    pub account_public_id: PublicIdV1,
    pub provider_source_contact_public_id: PublicIdV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceClaimsV1 {
    pub display_name: Option<String>,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceProvenanceV1 {
    pub revision: u64,
    pub digest: DigestV1,
    pub observed_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedSourceV1 {
    pub logical_owner_id: String,
    pub provenance: SourceProvenanceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObservationV1 {
    pub logical_owner_id: String,
    pub key: SourceLinkKeyV1,
    pub claims: SourceClaimsV1,
    pub provenance: SourceProvenanceV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionProvenanceV1 {
    pub decision_id: PublicIdV1,
    pub review_id: PublicIdV1,
    pub revision: u64,
    pub decided_by_owner_device_id: PublicIdV1,
    pub decided_at: TimestampV1,
    pub approved_action_digest: DigestV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachSourceActionV1 {
    pub logical_owner_id: String,
    pub from_person_id: PersonIdV1,
    pub expected_from_person_revision: u64,
    pub to_person_id: PersonIdV1,
    pub expected_to_person_revision: u64,
    pub source: SourceLinkKeyV1,
    pub expected_source_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetachSourceActionV1 {
    pub logical_owner_id: String,
    pub person_id: PersonIdV1,
    pub expected_person_revision: u64,
    pub source: SourceLinkKeyV1,
    pub expected_source_revision: u64,
    pub expected_detached_person_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergePersonsActionV1 {
    pub logical_owner_id: String,
    pub source_person_id: PersonIdV1,
    pub expected_source_person_revision: u64,
    pub target_person_id: PersonIdV1,
    pub expected_target_person_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SplitProfileFactKindV1 {
    DisplayName,
    GivenName,
    FamilyName,
    Emails,
    Phones,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SplitSourceSelectionV1 {
    pub source: SourceLinkKeyV1,
    pub expected_source_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitPersonActionV1 {
    pub logical_owner_id: String,
    pub merged_person_id: PersonIdV1,
    pub expected_merged_person_revision: u64,
    pub target_person_id: PersonIdV1,
    pub expected_target_person_revision: u64,
    pub source_selection: Vec<SplitSourceSelectionV1>,
    pub profile_fact_selection: Vec<SplitProfileFactKindV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLinkV1 {
    pub key: SourceLinkKeyV1,
    pub claims: SourceClaimsV1,
    pub provenance: SourceProvenanceV1,
    pub last_decision: Option<DecisionProvenanceV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPersonDraftV1 {
    pub person_id: PersonIdV1,
    pub logical_owner_id: String,
    pub owner_profile: OwnerProfileV1,
    pub created_at: TimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonV1 {
    pub person_id: PersonIdV1,
    pub logical_owner_id: String,
    pub lifecycle: PersonLifecycleV1,
    pub revision: u64,
    pub owner_profile: Option<OwnerProfileV1>,
    pub source_links: BTreeMap<SourceLinkKeyV1, SourceLinkV1>,
    pub merged_into: Option<PersonIdV1>,
    pub created_at: TimestampV1,
    pub updated_at: TimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IdentityMatchKindV1 {
    NormalizedEmail,
    NormalizedPhone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewCandidateV1 {
    pub candidate_id: PublicIdV1,
    pub first_person_id: PersonIdV1,
    pub second_person_id: PersonIdV1,
    pub first_source: SourceLinkKeyV1,
    pub second_source: SourceLinkKeyV1,
    pub match_kind: IdentityMatchKindV1,
    pub observed_at: TimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineageChangeKindV1 {
    Merge,
    Split,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineageRecordV1 {
    pub change_kind: LineageChangeKindV1,
    pub source_person_id: PersonIdV1,
    pub target_person_id: PersonIdV1,
    pub moved_sources: Vec<SourceLinkKeyV1>,
    pub preserved_source_profile: Option<OwnerProfileV1>,
    pub profile_fact_selection: Vec<SplitProfileFactKindV1>,
    pub decision: DecisionProvenanceV1,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PersonRevisionV1 {
    pub person_id: PersonIdV1,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmedActionStatusV1 {
    Applied,
    Replayed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmedActionOutcomeV1 {
    pub status: ConfirmedActionStatusV1,
    pub person_revisions: Vec<PersonRevisionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionReceiptV1 {
    pub logical_owner_id: String,
    pub action_digest: DigestV1,
    pub decision: DecisionProvenanceV1,
    pub outcome: ConfirmedActionOutcomeV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonsOwnerSnapshotV1 {
    pub logical_owner_id: String,
    pub persons: Vec<PersonV1>,
    pub removed_sources: Vec<(SourceLinkKeyV1, RemovedSourceV1)>,
    pub lineage: Vec<LineageRecordV1>,
    pub decision_receipts: Vec<DecisionReceiptV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceObservationOutcomeV1 {
    Created {
        person_id: PersonIdV1,
        review_candidates: Vec<ReviewCandidateV1>,
    },
    Updated {
        person_id: PersonIdV1,
        review_candidates: Vec<ReviewCandidateV1>,
    },
    Unchanged {
        person_id: PersonIdV1,
    },
}

impl SourceObservationOutcomeV1 {
    #[must_use]
    pub fn person_id(&self) -> PersonIdV1 {
        match self {
            Self::Created { person_id, .. }
            | Self::Updated { person_id, .. }
            | Self::Unchanged { person_id } => *person_id,
        }
    }

    #[must_use]
    pub fn review_candidates(&self) -> &[ReviewCandidateV1] {
        match self {
            Self::Created {
                review_candidates, ..
            }
            | Self::Updated {
                review_candidates, ..
            } => review_candidates,
            Self::Unchanged { .. } => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRemovalOutcomeV1 {
    pub person_id: Option<PersonIdV1>,
    pub archived: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonsTransitionErrorV1 {
    InvalidOwner,
    InvalidPublicId,
    InvalidPersonId,
    InvalidDigest,
    InvalidTimestamp,
    InvalidRevision,
    InvalidProfile,
    InvalidSourceClaims,
    InvalidEmail,
    InvalidPhone,
    ReviewCandidateLimitExceeded,
    PersonAlreadyExists,
    PersonNotFound,
    SourceNotFound,
    SourceOwnerConflict,
    StaleSourceRevision,
    SourceRevisionConflict,
    ExpectedRevisionConflict,
    ExpectedSourceRevisionConflict,
    DecisionRequired,
    ActionDigestMismatch,
    DecisionReuseConflict,
    DecisionTimestampRegression,
    OwnerMismatch,
    EmptySplitSelection,
    DuplicateSplitSelection,
    ProfileFactUnavailable,
    InvalidSnapshot,
    SamePerson,
    PersonMerged,
    LineageConflict,
}

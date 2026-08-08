use makosh_blob_protocol::{BlobCustodyScopeV1, BlobRefV1};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlobMetadataRecordV1 {
    reference: BlobRefV1,
    custody: BlobCustodyScopeV1,
    state: BlobMetadataStateV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlobDueDeletionV1 {
    reference: BlobRefV1,
    custody: BlobCustodyScopeV1,
    reservation: BlobDeletionReservationV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BlobMetadataStateV1 {
    PendingWrite,
    Active,
    DeleteReserved { not_before_unix_ms: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlobWriteReservationV1 {
    reference_id: [u8; 16],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlobDeletionReservationV1 {
    reference_id: [u8; 16],
    not_before_unix_ms: u64,
}

impl BlobMetadataRecordV1 {
    pub(crate) fn pending(reference: BlobRefV1, custody: BlobCustodyScopeV1) -> Self {
        Self {
            reference,
            custody,
            state: BlobMetadataStateV1::PendingWrite,
        }
    }

    pub(crate) fn from_parts(
        reference: BlobRefV1,
        custody: BlobCustodyScopeV1,
        state: BlobMetadataStateV1,
    ) -> Option<Self> {
        (reference.owner_id() == custody.owner_id()).then_some(Self {
            reference,
            custody,
            state,
        })
    }

    pub(crate) fn reference(&self) -> &BlobRefV1 {
        &self.reference
    }

    pub(crate) fn custody(&self) -> &BlobCustodyScopeV1 {
        &self.custody
    }

    pub(crate) const fn state(&self) -> BlobMetadataStateV1 {
        self.state
    }

    pub(crate) fn activate(&mut self) {
        self.state = BlobMetadataStateV1::Active;
    }

    pub(crate) fn reserve_deletion(&mut self, not_before_unix_ms: u64) {
        self.state = BlobMetadataStateV1::DeleteReserved { not_before_unix_ms };
    }

    pub(crate) fn belongs_to_quota(&self, custody: &BlobCustodyScopeV1) -> bool {
        self.custody == *custody
    }

    pub(crate) fn matches(&self, reference: &BlobRefV1, custody: &BlobCustodyScopeV1) -> bool {
        self.reference == *reference && self.custody == *custody
    }

    pub(crate) const fn counts_toward_quota(&self) -> bool {
        matches!(
            self.state,
            BlobMetadataStateV1::PendingWrite | BlobMetadataStateV1::Active
        )
    }

    pub(crate) const fn is_pending_write(&self) -> bool {
        matches!(self.state, BlobMetadataStateV1::PendingWrite)
    }

    pub(crate) fn due_deletion(&self, now_unix_ms: u64) -> Option<BlobDueDeletionV1> {
        let BlobMetadataStateV1::DeleteReserved { not_before_unix_ms } = self.state else {
            return None;
        };
        (not_before_unix_ms <= now_unix_ms).then(|| BlobDueDeletionV1 {
            reference: self.reference.clone(),
            custody: self.custody.clone(),
            reservation: BlobDeletionReservationV1::new(&self.reference, not_before_unix_ms),
        })
    }
}

impl BlobDueDeletionV1 {
    pub(crate) fn reference(&self) -> &BlobRefV1 {
        &self.reference
    }

    pub(crate) fn custody(&self) -> &BlobCustodyScopeV1 {
        &self.custody
    }

    pub(crate) fn reservation(&self) -> &BlobDeletionReservationV1 {
        &self.reservation
    }
}

impl BlobWriteReservationV1 {
    pub(crate) fn new(reference: &BlobRefV1) -> Self {
        Self {
            reference_id: *reference.reference_id(),
        }
    }

    pub(crate) fn matches(&self, reference: &BlobRefV1) -> bool {
        self.reference_id == *reference.reference_id()
    }
}

impl BlobDeletionReservationV1 {
    pub(crate) fn new(reference: &BlobRefV1, not_before_unix_ms: u64) -> Self {
        Self {
            reference_id: *reference.reference_id(),
            not_before_unix_ms,
        }
    }

    pub(crate) fn matches(&self, reference: &BlobRefV1) -> bool {
        self.reference_id == *reference.reference_id()
    }

    #[must_use]
    pub const fn not_before_unix_ms(&self) -> u64 {
        self.not_before_unix_ms
    }
}

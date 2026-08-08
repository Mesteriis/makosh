use makosh_clock_protocol::UtcMillisV1;
use makosh_scheduler_protocol::{
    ConcurrencyKeyV1, OverlapPolicyV1, ScheduleIdV1, ScheduleRunLeaseV1, ScheduleSpecV1,
};

use super::{ScheduleEntryV1, ScheduleLeaseStateV1};

/// Owner-neutral, non-durable schedule catalog. Persistence owns atomic commits.
#[derive(Default)]
pub struct ScheduleCatalogV1 {
    entries: Vec<ScheduleEntryV1>,
}

impl ScheduleCatalogV1 {
    pub fn reconcile(
        &mut self,
        spec: ScheduleSpecV1,
        next_due_at: UtcMillisV1,
    ) -> Result<ScheduleReconcileOutcomeV1, ScheduleCatalogErrorV1> {
        let Some(entry) = self.entry_mut(spec.schedule_id()) else {
            self.entries.push(ScheduleEntryV1::new(spec, next_due_at));
            return Ok(ScheduleReconcileOutcomeV1::Inserted);
        };
        let current = entry.spec();
        if spec.revision() < current.revision() {
            return Err(ScheduleCatalogErrorV1::StaleRevision);
        }
        if spec.revision() == current.revision() {
            return (spec == *current)
                .then_some(ScheduleReconcileOutcomeV1::Unchanged)
                .ok_or(ScheduleCatalogErrorV1::RevisionConflict);
        }
        entry.replace_spec(spec, next_due_at);
        Ok(ScheduleReconcileOutcomeV1::Updated)
    }

    pub fn claim_due(
        &mut self,
        lease: ScheduleRunLeaseV1,
        now: UtcMillisV1,
    ) -> Result<(), ScheduleCatalogErrorV1> {
        let index = self
            .entry_index(lease.schedule_id())
            .ok_or(ScheduleCatalogErrorV1::UnknownSchedule)?;
        let entry = &self.entries[index];
        if !entry.spec().enabled()
            || entry.spec().revision() != lease.schedule_revision()
            || entry.next_due_at() > now
            || matches!(entry.lease_state(now), ScheduleLeaseStateV1::Active(_))
            || !self.concurrency_allows(index, now)
        {
            return Err(ScheduleCatalogErrorV1::ClaimDenied);
        }
        self.entries[index].claim(lease);
        Ok(())
    }

    #[must_use]
    pub fn entry(&self, id: ScheduleIdV1) -> Option<&ScheduleEntryV1> {
        self.entries
            .iter()
            .find(|entry| entry.spec().schedule_id() == id)
    }

    fn entry_mut(&mut self, id: ScheduleIdV1) -> Option<&mut ScheduleEntryV1> {
        self.entries
            .iter_mut()
            .find(|entry| entry.spec().schedule_id() == id)
    }

    fn entry_index(&self, id: ScheduleIdV1) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.spec().schedule_id() == id)
    }

    fn concurrency_allows(&self, candidate: usize, now: UtcMillisV1) -> bool {
        let spec = self.entries[candidate].spec();
        let active = self.active_runs_for_key(candidate, spec.concurrency_key(), now);
        match spec.policy().overlap() {
            OverlapPolicyV1::Forbid
            | OverlapPolicyV1::Queue { .. }
            | OverlapPolicyV1::CoalesceLatest => active == 0,
            OverlapPolicyV1::AllowBounded { max_parallelism } => {
                active < usize::from(max_parallelism)
            }
        }
    }

    fn active_runs_for_key(
        &self,
        candidate: usize,
        key: &ConcurrencyKeyV1,
        now: UtcMillisV1,
    ) -> usize {
        self.entries
            .iter()
            .enumerate()
            .filter(|(index, entry)| {
                *index != candidate
                    && entry.spec().concurrency_key() == key
                    && matches!(entry.lease_state(now), ScheduleLeaseStateV1::Active(_))
            })
            .count()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleReconcileOutcomeV1 {
    Inserted,
    Unchanged,
    Updated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleCatalogErrorV1 {
    StaleRevision,
    RevisionConflict,
    UnknownSchedule,
    ClaimDenied,
}

#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-mail-contacts-sync-core";
pub const MAIL_CONTACTS_SYNC_MAX_ACCOUNT_ID_BYTES_V1: usize = 256;
pub const MAIL_CONTACTS_SYNC_MAX_CURSOR_BYTES_V1: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncDirectionV1 {
    ProviderToContacts,
    Bidirectional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncTriggerV1 {
    Manual,
    Scheduled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncStateV1 {
    Accepted,
    FetchingProviderPage,
    ApplyingContacts,
    WritingProvider,
    ReconcilingOutcome,
    Completed,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncRejectCodeV1 {
    InvalidRequest,
    AccountUnavailable,
    ProviderUnavailable,
    ContactsRejected,
    RemoteWriteBlocked,
    EtagConflict,
    OutcomeUnknown,
    Policy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncDraftV1 {
    pub run_id: [u8; 16],
    pub operation_id: [u8; 16],
    pub account_id: String,
    pub direction: MailContactsSyncDirectionV1,
    pub trigger: MailContactsSyncTriggerV1,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MailContactsSyncCountersV1 {
    pub provider_entries_seen: u64,
    pub contacts_created: u64,
    pub contacts_updated: u64,
    pub contacts_unchanged: u64,
    pub provider_entries_written: u64,
    pub rejected_entries: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncStatusV1 {
    pub state: MailContactsSyncStateV1,
    pub state_revision: u64,
    pub page_sequence: u64,
    pub continuation_cursor: Option<Vec<u8>>,
    pub counters: MailContactsSyncCountersV1,
    pub rejection: Option<MailContactsSyncRejectCodeV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailContactsSyncTransitionV1 {
    BeginProviderPage,
    ProviderPageObserved {
        page_sequence: u64,
        continuation_cursor: Option<Vec<u8>>,
        observed_entries: u32,
    },
    ContactsApplied {
        created: u32,
        updated: u32,
        unchanged: u32,
        rejected: u32,
    },
    BeginProviderWrite,
    ProviderWriteApplied {
        written: u32,
    },
    ReconcileOutcome,
    Complete,
    Reject(MailContactsSyncRejectCodeV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncValidationErrorV1 {
    InvalidRunId,
    InvalidOperationId,
    InvalidAccountId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncTransitionErrorV1 {
    InvalidTransition,
    InvalidPage,
    CounterOverflow,
    RevisionExhausted,
}

pub fn validate_mail_contacts_sync_draft_v1(
    draft: &MailContactsSyncDraftV1,
) -> Result<(), MailContactsSyncValidationErrorV1> {
    if zero(&draft.run_id) {
        return Err(MailContactsSyncValidationErrorV1::InvalidRunId);
    }
    if zero(&draft.operation_id) {
        return Err(MailContactsSyncValidationErrorV1::InvalidOperationId);
    }
    if draft.account_id.is_empty()
        || draft.account_id.len() > MAIL_CONTACTS_SYNC_MAX_ACCOUNT_ID_BYTES_V1
        || !draft.account_id.is_ascii()
        || draft.account_id.trim() != draft.account_id
    {
        return Err(MailContactsSyncValidationErrorV1::InvalidAccountId);
    }
    Ok(())
}

#[must_use]
pub fn accepted_mail_contacts_sync_status_v1() -> MailContactsSyncStatusV1 {
    MailContactsSyncStatusV1 {
        state: MailContactsSyncStateV1::Accepted,
        state_revision: 1,
        page_sequence: 0,
        continuation_cursor: None,
        counters: MailContactsSyncCountersV1::default(),
        rejection: None,
    }
}

pub fn transition_mail_contacts_sync_v1(
    current: &MailContactsSyncStatusV1,
    direction: MailContactsSyncDirectionV1,
    transition: MailContactsSyncTransitionV1,
) -> Result<MailContactsSyncStatusV1, MailContactsSyncTransitionErrorV1> {
    let mut next = current.clone();
    next.state_revision = next
        .state_revision
        .checked_add(1)
        .ok_or(MailContactsSyncTransitionErrorV1::RevisionExhausted)?;
    match (current.state, transition) {
        (MailContactsSyncStateV1::Accepted, MailContactsSyncTransitionV1::BeginProviderPage)
        | (
            MailContactsSyncStateV1::ApplyingContacts,
            MailContactsSyncTransitionV1::BeginProviderPage,
        ) => {
            next.state = MailContactsSyncStateV1::FetchingProviderPage;
        }
        (
            MailContactsSyncStateV1::FetchingProviderPage,
            MailContactsSyncTransitionV1::ProviderPageObserved {
                page_sequence,
                continuation_cursor,
                observed_entries,
            },
        ) => {
            if page_sequence != current.page_sequence + 1
                || continuation_cursor.as_ref().is_some_and(|value| {
                    value.is_empty() || value.len() > MAIL_CONTACTS_SYNC_MAX_CURSOR_BYTES_V1
                })
            {
                return Err(MailContactsSyncTransitionErrorV1::InvalidPage);
            }
            next.page_sequence = page_sequence;
            next.continuation_cursor = continuation_cursor;
            next.counters.provider_entries_seen =
                add(next.counters.provider_entries_seen, observed_entries)?;
            next.state = MailContactsSyncStateV1::ApplyingContacts;
        }
        (
            MailContactsSyncStateV1::ApplyingContacts,
            MailContactsSyncTransitionV1::ContactsApplied {
                created,
                updated,
                unchanged,
                rejected,
            },
        ) => {
            next.counters.contacts_created = add(next.counters.contacts_created, created)?;
            next.counters.contacts_updated = add(next.counters.contacts_updated, updated)?;
            next.counters.contacts_unchanged = add(next.counters.contacts_unchanged, unchanged)?;
            next.counters.rejected_entries = add(next.counters.rejected_entries, rejected)?;
        }
        (
            MailContactsSyncStateV1::ApplyingContacts,
            MailContactsSyncTransitionV1::BeginProviderWrite,
        ) if direction == MailContactsSyncDirectionV1::Bidirectional
            && current.continuation_cursor.is_none() =>
        {
            next.state = MailContactsSyncStateV1::WritingProvider;
        }
        (
            MailContactsSyncStateV1::WritingProvider,
            MailContactsSyncTransitionV1::ProviderWriteApplied { written },
        ) => {
            next.counters.provider_entries_written =
                add(next.counters.provider_entries_written, written)?;
        }
        (
            MailContactsSyncStateV1::WritingProvider,
            MailContactsSyncTransitionV1::ReconcileOutcome,
        ) => {
            next.state = MailContactsSyncStateV1::ReconcilingOutcome;
        }
        (MailContactsSyncStateV1::ApplyingContacts, MailContactsSyncTransitionV1::Complete)
            if current.continuation_cursor.is_none()
                && (direction == MailContactsSyncDirectionV1::ProviderToContacts
                    || direction == MailContactsSyncDirectionV1::Bidirectional
                        && current.counters.contacts_created == 0
                        && current.counters.contacts_updated == 0) =>
        {
            next.state = MailContactsSyncStateV1::Completed;
        }
        (MailContactsSyncStateV1::WritingProvider, MailContactsSyncTransitionV1::Complete)
        | (MailContactsSyncStateV1::ReconcilingOutcome, MailContactsSyncTransitionV1::Complete) => {
            next.state = MailContactsSyncStateV1::Completed;
        }
        (
            MailContactsSyncStateV1::Accepted
            | MailContactsSyncStateV1::FetchingProviderPage
            | MailContactsSyncStateV1::ApplyingContacts
            | MailContactsSyncStateV1::WritingProvider
            | MailContactsSyncStateV1::ReconcilingOutcome,
            MailContactsSyncTransitionV1::Reject(code),
        ) => {
            next.state = MailContactsSyncStateV1::Rejected;
            next.rejection = Some(code);
        }
        _ => return Err(MailContactsSyncTransitionErrorV1::InvalidTransition),
    }
    Ok(next)
}

fn add(current: u64, increment: u32) -> Result<u64, MailContactsSyncTransitionErrorV1> {
    current
        .checked_add(u64::from(increment))
        .ok_or(MailContactsSyncTransitionErrorV1::CounterOverflow)
}

fn zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_to_contacts_requires_page_completion_before_terminal_state() {
        let accepted = accepted_mail_contacts_sync_status_v1();
        let fetching = transition_mail_contacts_sync_v1(
            &accepted,
            MailContactsSyncDirectionV1::ProviderToContacts,
            MailContactsSyncTransitionV1::BeginProviderPage,
        )
        .expect("fetching");
        assert!(
            transition_mail_contacts_sync_v1(
                &fetching,
                MailContactsSyncDirectionV1::ProviderToContacts,
                MailContactsSyncTransitionV1::Complete
            )
            .is_err()
        );
        let applying = transition_mail_contacts_sync_v1(
            &fetching,
            MailContactsSyncDirectionV1::ProviderToContacts,
            MailContactsSyncTransitionV1::ProviderPageObserved {
                page_sequence: 1,
                continuation_cursor: None,
                observed_entries: 3,
            },
        )
        .expect("page");
        let completed = transition_mail_contacts_sync_v1(
            &applying,
            MailContactsSyncDirectionV1::ProviderToContacts,
            MailContactsSyncTransitionV1::Complete,
        )
        .expect("complete");
        assert_eq!(completed.state, MailContactsSyncStateV1::Completed);
        assert_eq!(completed.counters.provider_entries_seen, 3);
    }

    #[test]
    fn bidirectional_requires_write_phase_only_when_contacts_changed() {
        let accepted = accepted_mail_contacts_sync_status_v1();
        let fetching = transition_mail_contacts_sync_v1(
            &accepted,
            MailContactsSyncDirectionV1::Bidirectional,
            MailContactsSyncTransitionV1::BeginProviderPage,
        )
        .expect("fetching");
        let applying = transition_mail_contacts_sync_v1(
            &fetching,
            MailContactsSyncDirectionV1::Bidirectional,
            MailContactsSyncTransitionV1::ProviderPageObserved {
                page_sequence: 1,
                continuation_cursor: None,
                observed_entries: 0,
            },
        )
        .expect("page");
        let completed = transition_mail_contacts_sync_v1(
            &applying,
            MailContactsSyncDirectionV1::Bidirectional,
            MailContactsSyncTransitionV1::Complete,
        )
        .expect("empty bidirectional page completes without a fake write");
        assert_eq!(completed.state, MailContactsSyncStateV1::Completed);
        let changed = transition_mail_contacts_sync_v1(
            &applying,
            MailContactsSyncDirectionV1::Bidirectional,
            MailContactsSyncTransitionV1::ContactsApplied {
                created: 1,
                updated: 0,
                unchanged: 0,
                rejected: 0,
            },
        )
        .expect("changed contact");
        assert!(
            transition_mail_contacts_sync_v1(
                &changed,
                MailContactsSyncDirectionV1::Bidirectional,
                MailContactsSyncTransitionV1::Complete
            )
            .is_err()
        );
        let writing = transition_mail_contacts_sync_v1(
            &changed,
            MailContactsSyncDirectionV1::Bidirectional,
            MailContactsSyncTransitionV1::BeginProviderWrite,
        )
        .expect("writing");
        assert_eq!(writing.state, MailContactsSyncStateV1::WritingProvider);
    }

    #[test]
    fn outcome_unknown_enters_reconciliation_without_blind_retry() {
        let mut writing = accepted_mail_contacts_sync_status_v1();
        writing.state = MailContactsSyncStateV1::WritingProvider;
        let reconciling = transition_mail_contacts_sync_v1(
            &writing,
            MailContactsSyncDirectionV1::Bidirectional,
            MailContactsSyncTransitionV1::ReconcileOutcome,
        )
        .expect("reconcile");
        assert_eq!(
            reconciling.state,
            MailContactsSyncStateV1::ReconcilingOutcome
        );
        assert!(
            transition_mail_contacts_sync_v1(
                &reconciling,
                MailContactsSyncDirectionV1::Bidirectional,
                MailContactsSyncTransitionV1::BeginProviderWrite
            )
            .is_err()
        );
    }

    #[test]
    fn draft_and_cursor_are_bounded() {
        let draft = MailContactsSyncDraftV1 {
            run_id: [1; 16],
            operation_id: [2; 16],
            account_id: "mail-account-1".to_owned(),
            direction: MailContactsSyncDirectionV1::ProviderToContacts,
            trigger: MailContactsSyncTriggerV1::Manual,
        };
        assert_eq!(validate_mail_contacts_sync_draft_v1(&draft), Ok(()));
        let accepted = accepted_mail_contacts_sync_status_v1();
        let fetching = transition_mail_contacts_sync_v1(
            &accepted,
            draft.direction,
            MailContactsSyncTransitionV1::BeginProviderPage,
        )
        .expect("fetching");
        assert_eq!(
            transition_mail_contacts_sync_v1(
                &fetching,
                draft.direction,
                MailContactsSyncTransitionV1::ProviderPageObserved {
                    page_sequence: 1,
                    continuation_cursor: Some(vec![1; MAIL_CONTACTS_SYNC_MAX_CURSOR_BYTES_V1 + 1]),
                    observed_entries: 0
                }
            ),
            Err(MailContactsSyncTransitionErrorV1::InvalidPage)
        );
    }
}

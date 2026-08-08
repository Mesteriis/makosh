use makosh_mail_contacts_sync_api::wire::{
    GetMailContactsSyncRequestV1, GetMailContactsSyncResponseV1,
    MailContactsSyncDirectionV1 as WireDirection, MailContactsSyncErrorCodeV1 as WireError,
    MailContactsSyncStateV1 as WireState, MailContactsSyncTriggerV1 as WireTrigger,
    StartMailContactsSyncRequestV1, StartMailContactsSyncResponseV1,
};
use makosh_mail_contacts_sync_core::{
    MailContactsSyncDirectionV1, MailContactsSyncDraftV1, MailContactsSyncRejectCodeV1,
    MailContactsSyncStateV1, MailContactsSyncTriggerV1,
};
use makosh_mail_contacts_sync_persistence::{
    CreateMailContactsSyncOutcomeV1, CreateMailContactsSyncRunV1,
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
    PersistedMailContactsSyncRunV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    MailContactsSyncRuntimeSettingsV1,
    commands::{InitialFetchCommandContextV1, build_initial_fetch_command_v1},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncClientContextV1 {
    pub logical_owner_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub authoritative_now_unix_millis: i64,
    pub settings: MailContactsSyncRuntimeSettingsV1,
}

pub async fn start_mail_contacts_sync_payload_v1(
    persistence: &MailContactsSyncPersistenceV1,
    context: &MailContactsSyncClientContextV1,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = StartMailContactsSyncRequestV1::decode(payload) else {
        return start_error(
            Vec::new(),
            WireError::MailContactsSyncErrorCodeInvalidRequest,
        );
    };
    let operation_id = match id16(&request.operation_id) {
        Some(value) => value,
        None => {
            return start_error(
                Vec::new(),
                WireError::MailContactsSyncErrorCodeInvalidRequest,
            );
        }
    };
    let direction = match wire_direction(request.direction) {
        Some(value) if value == context.settings.direction => value,
        _ => return start_error(Vec::new(), WireError::MailContactsSyncErrorCodePolicy),
    };
    if request.protocol_major != 1
        || request.account_id != context.settings.account_id
        || context.authoritative_now_unix_millis <= 0
    {
        return start_error(
            Vec::new(),
            WireError::MailContactsSyncErrorCodeInvalidRequest,
        );
    }
    let run_id = digest16(
        b"mail-contacts-sync-run-v1",
        context.logical_owner_id.as_bytes(),
        &operation_id,
    );
    let initial_command = match build_initial_fetch_command_v1(
        run_id,
        &context.settings.account_id,
        &InitialFetchCommandContextV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            runtime_instance_id: context.runtime_instance_id.clone(),
            runtime_generation: context.runtime_generation,
            authoritative_now_unix_millis: context.authoritative_now_unix_millis,
        },
    ) {
        Ok(command) => command,
        Err(_) => {
            return start_error(
                run_id.to_vec(),
                WireError::MailContactsSyncErrorCodeUnavailable,
            );
        }
    };
    match persistence
        .create_run(CreateMailContactsSyncRunV1 {
            logical_owner_id: context.logical_owner_id.clone(),
            draft: MailContactsSyncDraftV1 {
                run_id,
                operation_id,
                account_id: context.settings.account_id.clone(),
                direction,
                trigger: MailContactsSyncTriggerV1::Manual,
            },
            initial_commands: vec![initial_command],
            created_at_unix_millis: context.authoritative_now_unix_millis,
        })
        .await
    {
        Ok(
            CreateMailContactsSyncOutcomeV1::Created(run)
            | CreateMailContactsSyncOutcomeV1::Existing(run),
        ) => StartMailContactsSyncResponseV1 {
            run_id: run.draft.run_id.to_vec(),
            state: wire_state(run.status.state) as i32,
            error: WireError::MailContactsSyncErrorCodeUnspecified as i32,
        }
        .encode_to_vec(),
        Err(error) => start_error(run_id.to_vec(), persistence_error(error)),
    }
}

pub async fn get_mail_contacts_sync_payload_v1(
    persistence: &MailContactsSyncPersistenceV1,
    logical_owner_id: &str,
    payload: &[u8],
) -> Vec<u8> {
    let Ok(request) = GetMailContactsSyncRequestV1::decode(payload) else {
        return get_error(
            Vec::new(),
            WireError::MailContactsSyncErrorCodeInvalidRequest,
        );
    };
    let Some(run_id) = id16(&request.run_id) else {
        return get_error(
            request.run_id,
            WireError::MailContactsSyncErrorCodeInvalidRequest,
        );
    };
    if request.protocol_major != 1 {
        return get_error(
            run_id.to_vec(),
            WireError::MailContactsSyncErrorCodeInvalidRequest,
        );
    }
    match persistence.load_run(logical_owner_id, &run_id).await {
        Ok(run) => get_response(run),
        Err(error) => get_error(run_id.to_vec(), persistence_error(error)),
    }
}

fn get_response(run: PersistedMailContactsSyncRunV1) -> Vec<u8> {
    GetMailContactsSyncResponseV1 {
        run_id: run.draft.run_id.to_vec(),
        account_id: run.draft.account_id,
        direction: wire_direction_out(run.draft.direction) as i32,
        trigger: wire_trigger(run.draft.trigger) as i32,
        state: wire_state(run.status.state) as i32,
        state_revision: run.status.state_revision,
        provider_entries_seen: run.status.counters.provider_entries_seen,
        contacts_created: run.status.counters.contacts_created,
        contacts_updated: run.status.counters.contacts_updated,
        contacts_unchanged: run.status.counters.contacts_unchanged,
        provider_entries_written: run.status.counters.provider_entries_written,
        rejected_entries: run.status.counters.rejected_entries,
        error: run.status.rejection.map_or(
            WireError::MailContactsSyncErrorCodeUnspecified,
            wire_rejection,
        ) as i32,
    }
    .encode_to_vec()
}

fn start_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    StartMailContactsSyncResponseV1 {
        run_id,
        state: WireState::MailContactsSyncStateUnspecified as i32,
        error: error as i32,
    }
    .encode_to_vec()
}

fn get_error(run_id: Vec<u8>, error: WireError) -> Vec<u8> {
    GetMailContactsSyncResponseV1 {
        run_id,
        error: error as i32,
        ..Default::default()
    }
    .encode_to_vec()
}

fn wire_direction(value: i32) -> Option<MailContactsSyncDirectionV1> {
    match WireDirection::try_from(value).ok()? {
        WireDirection::MailContactsSyncDirectionProviderToContacts => {
            Some(MailContactsSyncDirectionV1::ProviderToContacts)
        }
        WireDirection::MailContactsSyncDirectionBidirectional => {
            Some(MailContactsSyncDirectionV1::Bidirectional)
        }
        WireDirection::MailContactsSyncDirectionUnspecified => None,
    }
}

const fn wire_direction_out(value: MailContactsSyncDirectionV1) -> WireDirection {
    match value {
        MailContactsSyncDirectionV1::ProviderToContacts => {
            WireDirection::MailContactsSyncDirectionProviderToContacts
        }
        MailContactsSyncDirectionV1::Bidirectional => {
            WireDirection::MailContactsSyncDirectionBidirectional
        }
    }
}

const fn wire_trigger(value: MailContactsSyncTriggerV1) -> WireTrigger {
    match value {
        MailContactsSyncTriggerV1::Manual => WireTrigger::MailContactsSyncTriggerManual,
        MailContactsSyncTriggerV1::Scheduled => WireTrigger::MailContactsSyncTriggerScheduled,
    }
}

const fn wire_state(value: MailContactsSyncStateV1) -> WireState {
    match value {
        MailContactsSyncStateV1::Accepted => WireState::MailContactsSyncStateAccepted,
        MailContactsSyncStateV1::FetchingProviderPage => {
            WireState::MailContactsSyncStateFetchingProviderPage
        }
        MailContactsSyncStateV1::ApplyingContacts => {
            WireState::MailContactsSyncStateApplyingContacts
        }
        MailContactsSyncStateV1::WritingProvider => WireState::MailContactsSyncStateWritingProvider,
        MailContactsSyncStateV1::ReconcilingOutcome => {
            WireState::MailContactsSyncStateReconcilingOutcome
        }
        MailContactsSyncStateV1::Completed => WireState::MailContactsSyncStateCompleted,
        MailContactsSyncStateV1::Rejected => WireState::MailContactsSyncStateRejected,
    }
}

const fn wire_rejection(value: MailContactsSyncRejectCodeV1) -> WireError {
    match value {
        MailContactsSyncRejectCodeV1::InvalidRequest => {
            WireError::MailContactsSyncErrorCodeInvalidRequest
        }
        MailContactsSyncRejectCodeV1::AccountUnavailable => {
            WireError::MailContactsSyncErrorCodeAccountUnavailable
        }
        MailContactsSyncRejectCodeV1::ProviderUnavailable => {
            WireError::MailContactsSyncErrorCodeProviderUnavailable
        }
        MailContactsSyncRejectCodeV1::ContactsRejected => {
            WireError::MailContactsSyncErrorCodeContactsRejected
        }
        MailContactsSyncRejectCodeV1::RemoteWriteBlocked => {
            WireError::MailContactsSyncErrorCodeRemoteWriteBlocked
        }
        MailContactsSyncRejectCodeV1::EtagConflict => {
            WireError::MailContactsSyncErrorCodeEtagConflict
        }
        MailContactsSyncRejectCodeV1::OutcomeUnknown => {
            WireError::MailContactsSyncErrorCodeOutcomeUnknown
        }
        MailContactsSyncRejectCodeV1::Policy => WireError::MailContactsSyncErrorCodePolicy,
    }
}

const fn persistence_error(error: MailContactsSyncPersistenceErrorV1) -> WireError {
    match error {
        MailContactsSyncPersistenceErrorV1::InvalidInput
        | MailContactsSyncPersistenceErrorV1::RequestConflict
        | MailContactsSyncPersistenceErrorV1::InboxConflict
        | MailContactsSyncPersistenceErrorV1::InvalidTransition => {
            WireError::MailContactsSyncErrorCodeInvalidRequest
        }
        MailContactsSyncPersistenceErrorV1::NotFound => {
            WireError::MailContactsSyncErrorCodeNotFound
        }
        MailContactsSyncPersistenceErrorV1::InvalidRow
        | MailContactsSyncPersistenceErrorV1::StorageUnavailable
        | MailContactsSyncPersistenceErrorV1::RevisionConflict => {
            WireError::MailContactsSyncErrorCodeUnavailable
        }
    }
}

fn id16(value: &[u8]) -> Option<[u8; 16]> {
    let value: [u8; 16] = value.try_into().ok()?;
    value.iter().any(|byte| *byte != 0).then_some(value)
}

fn digest16(label: &[u8], left: &[u8], right: &[u8]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(label);
    digest.update([0]);
    digest.update(left);
    digest.update([0]);
    digest.update(right);
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

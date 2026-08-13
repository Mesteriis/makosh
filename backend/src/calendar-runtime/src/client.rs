use makosh_calendar_api::{
    CALENDAR_MODULE_ID_V1, CALENDAR_OWNER_ID_V1, CalendarEnvelopeContextV1,
    build_calendar_event_changed_outbox_record_v1,
    calendar_client_add_participant_contract_reference_v1,
    calendar_client_add_reminder_contract_reference_v1,
    calendar_client_create_contract_reference_v1, calendar_client_get_contract_reference_v1,
    calendar_client_list_contract_reference_v1,
    calendar_client_list_outcomes_contract_reference_v1,
    calendar_client_list_participants_contract_reference_v1,
    calendar_client_list_reminders_contract_reference_v1,
    calendar_client_record_outcome_contract_reference_v1,
    calendar_client_remove_participant_contract_reference_v1,
    calendar_client_remove_reminder_contract_reference_v1,
    calendar_client_search_contract_reference_v1,
    calendar_client_set_constraints_contract_reference_v1,
    calendar_client_set_state_contract_reference_v1, calendar_client_update_contract_reference_v1,
    calendar_client_update_participant_contract_reference_v1,
    client_wire::{
        AddCalendarParticipantRequestV1, AddCalendarReminderRequestV1, CalendarConstraintsV1,
        CalendarEventChangedV1, CalendarEventChildListRequestV1, CalendarEventMutationResultV1,
        CalendarEventStateV1 as WireEventState, CalendarEventV1 as WireEvent,
        CalendarOutcomeKindV1 as WireOutcomeKind, CalendarOutcomeV1 as WireOutcome,
        CalendarParticipantResponseV1 as WireParticipantResponse,
        CalendarParticipantRoleV1 as WireParticipantRole, CalendarParticipantV1 as WireParticipant,
        CalendarReminderStateV1 as WireReminderState, CalendarReminderV1 as WireReminder,
        CreateCalendarEventRequestV1, GetCalendarEventRequestV1, ListCalendarEventsRequestV1,
        ListCalendarEventsResultV1, ListCalendarOutcomesResultV1, ListCalendarParticipantsResultV1,
        ListCalendarRemindersResultV1, RecordCalendarOutcomeRequestV1,
        RemoveCalendarParticipantRequestV1, RemoveCalendarReminderRequestV1,
        SearchCalendarEventsRequestV1, SetCalendarConstraintsRequestV1,
        SetCalendarEventStateRequestV1, TimestampV1, UpdateCalendarEventRequestV1,
        UpdateCalendarParticipantRequestV1,
    },
};
use makosh_calendar_core::{
    CalendarEventDraftV1, CalendarEventRecordV1, CalendarEventStateV1, CalendarOutcomeKindV1,
    CalendarOutcomeV1, CalendarParticipantResponseV1, CalendarParticipantRoleV1,
    CalendarParticipantV1, CalendarReminderStateV1, CalendarReminderV1, CalendarTimestampV1,
    derive_calendar_reminder_id_v1,
};
use makosh_calendar_persistence::{
    CalendarLifecycleCommitV1, CalendarLifecycleMutationV1, CalendarLifecycleOperationOutcomeV1,
    CalendarLifecycleOperationV1, CalendarOutboxRecordV1, CalendarPersistenceErrorV1,
    CalendarPersistenceV1,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    CalendarSchedulerEnvelopeContextV1, build_cancel_reminder_schedule_v1,
    build_ensure_reminder_schedule_v1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalendarClientRuntimeContextV1 {
    pub runtime_instance_id: [u8; 16],
    pub runtime_generation: u64,
    pub scheduler_grant_epoch: u64,
    pub now_unix_millis: i64,
}

pub async fn dispatch_calendar_client_request_v1(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    request: ModuleClientRequestV1,
    context: CalendarClientRuntimeContextV1,
) -> ModuleClientResponseV1 {
    let accepted = request.protocol_major == 1
        && request.module_id == CALENDAR_MODULE_ID_V1
        && request.owner_id == CALENDAR_OWNER_ID_V1
        && request.logical_owner_id == logical_owner_id
        && !request.authenticated_device_id.is_empty()
        && context.runtime_instance_id.iter().any(|byte| *byte != 0)
        && context.runtime_generation > 0
        && context.scheduler_grant_epoch > 0
        && context.now_unix_millis > 0;
    let response = if accepted {
        dispatch(persistence, logical_owner_id, &request, context).await
    } else {
        Err("REJECTED")
    };
    match response {
        Ok(response_payload) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload,
            error_code: String::new(),
        },
        Err(error_code) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: error_code.to_owned(),
        },
    }
}

async fn dispatch(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    request: &ModuleClientRequestV1,
    context: CalendarClientRuntimeContextV1,
) -> Result<Vec<u8>, &'static str> {
    let contract = request.contract.as_ref().ok_or("REJECTED")?;
    if contract == &calendar_client_get_contract_reference_v1() {
        return get(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &calendar_client_list_contract_reference_v1() {
        return list(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &calendar_client_search_contract_reference_v1() {
        return search(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &calendar_client_list_participants_contract_reference_v1() {
        return list_participants(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &calendar_client_list_reminders_contract_reference_v1() {
        return list_reminders(persistence, logical_owner_id, &request.request_payload).await;
    }
    if contract == &calendar_client_list_outcomes_contract_reference_v1() {
        return list_outcomes(persistence, logical_owner_id, &request.request_payload).await;
    }

    let operation_id = decode_operation_id(contract, &request.request_payload)?;
    let request_sha256: [u8; 32] = Sha256::digest(&request.request_payload).into();
    if let Some(response) = persistence
        .load_operation_replay(
            logical_owner_id,
            operation_id,
            request_sha256,
            &request.request_payload,
        )
        .await
        .map_err(persistence_error)?
    {
        return Ok(response);
    }
    let mutation = decode_mutation(
        contract,
        logical_owner_id,
        &request.request_payload,
        context.now_unix_millis,
    )?;
    let operation = CalendarLifecycleOperationV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        operation_id,
        request_sha256,
        request_bytes: request.request_payload.clone(),
        received_at_unix_millis: context.now_unix_millis,
        mutation: mutation.clone(),
    };
    let envelope_context = CalendarEnvelopeContextV1 {
        module_id: CALENDAR_MODULE_ID_V1.to_owned(),
        runtime_instance_id: encode_id(context.runtime_instance_id),
        runtime_generation: context.runtime_generation,
        recorded_at_unix_seconds: context.now_unix_millis / 1_000,
        recorded_at_nanos: ((context.now_unix_millis % 1_000) * 1_000_000) as i32,
    };
    let scheduler_context = CalendarSchedulerEnvelopeContextV1 {
        logical_owner_id: logical_owner_id.to_owned(),
        runtime_instance_id: context.runtime_instance_id,
        runtime_generation: context.runtime_generation,
        grant_epoch: context.scheduler_grant_epoch,
        recorded_at_unix_millis: context.now_unix_millis,
    };
    let outcome = persistence
        .apply_lifecycle_operation(operation, |event| {
            build_commit(
                operation_id,
                &mutation,
                event,
                &envelope_context,
                &scheduler_context,
            )
        })
        .await
        .map_err(persistence_error)?;
    Ok(match outcome {
        CalendarLifecycleOperationOutcomeV1::Applied { response_bytes, .. }
        | CalendarLifecycleOperationOutcomeV1::Replayed { response_bytes } => response_bytes,
    })
}

fn build_commit(
    operation_id: [u8; 16],
    mutation: &CalendarLifecycleMutationV1,
    event: &CalendarEventRecordV1,
    envelope_context: &CalendarEnvelopeContextV1,
    scheduler_context: &CalendarSchedulerEnvelopeContextV1,
) -> Result<CalendarLifecycleCommitV1, CalendarPersistenceErrorV1> {
    let response = CalendarEventMutationResultV1 {
        operation_id: operation_id.to_vec(),
        event: Some(wire_event(event)),
    }
    .encode_to_vec();
    let changed = build_calendar_event_changed_outbox_record_v1(
        operation_id,
        CalendarEventChangedV1 {
            event_id: lifecycle_event_id(
                operation_id,
                event.calendar_event_id,
                event.event_revision,
            )
            .to_vec(),
            calendar_event_id: event.calendar_event_id.to_vec(),
            logical_owner_id: event.logical_owner_id.clone(),
            event_revision: event.event_revision,
            state: encode_event_state(event.state),
            occurred_at: Some(wire_timestamp(event.updated_at)),
        },
        envelope_context,
    )
    .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?;
    let mut outbox = vec![outbox_record(1, &changed)];
    match mutation {
        CalendarLifecycleMutationV1::AddReminder {
            operation_id,
            due_at,
            ..
        } => {
            let reminder_id =
                derive_calendar_reminder_id_v1(&event.calendar_event_id, operation_id)
                    .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?;
            let schedule = build_ensure_reminder_schedule_v1(
                *operation_id,
                reminder_id,
                timestamp_millis(*due_at)?,
                scheduler_context,
            )
            .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?;
            outbox.push(outbox_record(2, &schedule));
        }
        CalendarLifecycleMutationV1::RemoveReminder {
            operation_id,
            reminder_id,
            ..
        } => {
            let cancel =
                build_cancel_reminder_schedule_v1(*operation_id, *reminder_id, scheduler_context)
                    .map_err(|_| CalendarPersistenceErrorV1::InvalidInput)?;
            outbox.push(outbox_record(2, &cancel));
        }
        _ => {}
    }
    Ok(CalendarLifecycleCommitV1 {
        response_sha256: Sha256::digest(&response).into(),
        response_bytes: response,
        outbox,
    })
}

fn outbox_record(
    semantic_kind: i16,
    value: &makosh_events_protocol::delivery::OutboxRecordV1,
) -> CalendarOutboxRecordV1 {
    CalendarOutboxRecordV1 {
        message_id: *value.message_id(),
        semantic_kind,
        envelope_sha256: *value.envelope_sha256(),
        envelope_bytes: value.exact_bytes().to_vec(),
    }
}

fn decode_operation_id(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    bytes: &[u8],
) -> Result<[u8; 16], &'static str> {
    macro_rules! operation_id {
        ($reference:expr, $type:ty) => {
            if contract == &$reference {
                let value = <$type>::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
                if value.encode_to_vec() != bytes {
                    return Err("INVALID_ARGUMENT");
                }
                return id16(&value.operation_id);
            }
        };
    }
    operation_id!(
        calendar_client_create_contract_reference_v1(),
        CreateCalendarEventRequestV1
    );
    operation_id!(
        calendar_client_update_contract_reference_v1(),
        UpdateCalendarEventRequestV1
    );
    operation_id!(
        calendar_client_set_state_contract_reference_v1(),
        SetCalendarEventStateRequestV1
    );
    operation_id!(
        calendar_client_add_participant_contract_reference_v1(),
        AddCalendarParticipantRequestV1
    );
    operation_id!(
        calendar_client_update_participant_contract_reference_v1(),
        UpdateCalendarParticipantRequestV1
    );
    operation_id!(
        calendar_client_remove_participant_contract_reference_v1(),
        RemoveCalendarParticipantRequestV1
    );
    operation_id!(
        calendar_client_set_constraints_contract_reference_v1(),
        SetCalendarConstraintsRequestV1
    );
    operation_id!(
        calendar_client_add_reminder_contract_reference_v1(),
        AddCalendarReminderRequestV1
    );
    operation_id!(
        calendar_client_remove_reminder_contract_reference_v1(),
        RemoveCalendarReminderRequestV1
    );
    operation_id!(
        calendar_client_record_outcome_contract_reference_v1(),
        RecordCalendarOutcomeRequestV1
    );
    Err("REJECTED")
}

fn decode_mutation(
    contract: &makosh_runtime_protocol::v1::ContractReferenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
    now_unix_millis: i64,
) -> Result<CalendarLifecycleMutationV1, &'static str> {
    macro_rules! decode {
        ($type:ty) => {{
            let value = <$type>::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
            if value.encode_to_vec() != bytes {
                return Err("INVALID_ARGUMENT");
            }
            value
        }};
    }
    if contract == &calendar_client_create_contract_reference_v1() {
        let mut value = decode!(CreateCalendarEventRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::Create(CalendarEventDraftV1 {
            operation_id: id16(&value.operation_id)?,
            logical_owner_id: logical_owner_id.to_owned(),
            title: value.title,
            description: value.description,
            starts_at: raw_timestamp(value.starts_at)?,
            ends_at: raw_timestamp(value.ends_at)?,
            timezone: value.timezone,
            created_at: checked_timestamp(value.created_at, now_unix_millis)?,
        }))
    } else if contract == &calendar_client_update_contract_reference_v1() {
        let mut value = decode!(UpdateCalendarEventRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::Update {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            title: value.title,
            description: value.description,
            starts_at: value.starts_at.map(raw_wire_timestamp).transpose()?,
            ends_at: value.ends_at.map(raw_wire_timestamp).transpose()?,
            timezone: value.timezone,
            changed_at: checked_timestamp(value.updated_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_set_state_contract_reference_v1() {
        let mut value = decode!(SetCalendarEventStateRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::SetState {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            state: decode_event_state(value.state)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_add_participant_contract_reference_v1() {
        let mut value = decode!(AddCalendarParticipantRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::AddParticipant {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            display_name: value.display_name,
            address: value.address,
            role: decode_participant_role(value.role)?,
            response: decode_participant_response(value.response)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_update_participant_contract_reference_v1() {
        let mut value = decode!(UpdateCalendarParticipantRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::UpdateParticipant {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            participant_id: id16(&value.participant_id)?,
            display_name: value.display_name,
            address: value.address,
            role: value.role.map(decode_participant_role).transpose()?,
            response: value
                .response
                .map(decode_participant_response)
                .transpose()?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_remove_participant_contract_reference_v1() {
        let mut value = decode!(RemoveCalendarParticipantRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::RemoveParticipant {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            participant_id: id16(&value.participant_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_set_constraints_contract_reference_v1() {
        let mut value = decode!(SetCalendarConstraintsRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::SetConstraints {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            earliest_start: raw_timestamp(value.earliest_start)?,
            latest_end: raw_timestamp(value.latest_end)?,
            minimum_duration_minutes: value.minimum_duration_minutes,
            timezone: value.timezone,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_add_reminder_contract_reference_v1() {
        let mut value = decode!(AddCalendarReminderRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::AddReminder {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            due_at: raw_timestamp(value.due_at)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_remove_reminder_contract_reference_v1() {
        let mut value = decode!(RemoveCalendarReminderRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::RemoveReminder {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            reminder_id: id16(&value.reminder_id)?,
            changed_at: checked_timestamp(value.changed_at, now_unix_millis)?,
        })
    } else if contract == &calendar_client_record_outcome_contract_reference_v1() {
        let mut value = decode!(RecordCalendarOutcomeRequestV1);
        accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
        Ok(CalendarLifecycleMutationV1::RecordOutcome {
            operation_id: id16(&value.operation_id)?,
            calendar_event_id: id16(&value.calendar_event_id)?,
            expected_revision: positive_revision(value.expected_event_revision)?,
            kind: decode_outcome_kind(value.kind)?,
            note: value.note,
            recorded_at: checked_timestamp(value.recorded_at, now_unix_millis)?,
        })
    } else {
        Err("REJECTED")
    }
}

async fn get(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<GetCalendarEventRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    persistence
        .get_event(logical_owner_id, id16(&value.calendar_event_id)?)
        .await
        .map_err(persistence_error)?
        .map(|event| wire_event(&event).encode_to_vec())
        .ok_or("NOT_FOUND")
}

async fn list(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<ListCalendarEventsRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let limit = checked_limit(value.limit)?;
    let mut events = persistence
        .list_events(
            logical_owner_id,
            optional_id16(&value.after_calendar_event_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    Ok(paginate_events(&mut events, limit).encode_to_vec())
}

async fn search(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut value = exact_decode::<SearchCalendarEventsRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let limit = checked_limit(value.limit)?;
    let mut events = persistence
        .search_events(
            logical_owner_id,
            &value.query,
            optional_id16(&value.after_calendar_event_id)?,
            limit + 1,
        )
        .await
        .map_err(persistence_error)?;
    Ok(paginate_events(&mut events, limit).encode_to_vec())
}

async fn list_participants(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let (event, after, limit) = load_child_request(persistence, logical_owner_id, bytes).await?;
    let (items, next) = paginate_children(event.participants, after, limit, |value| {
        value.participant_id
    });
    Ok(ListCalendarParticipantsResultV1 {
        participants: items.iter().map(wire_participant).collect(),
        next_after_participant_id: next.map_or_else(Vec::new, |value| value.to_vec()),
    }
    .encode_to_vec())
}

async fn list_reminders(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let (event, after, limit) = load_child_request(persistence, logical_owner_id, bytes).await?;
    let (items, next) = paginate_children(event.reminders, after, limit, |value| value.reminder_id);
    Ok(ListCalendarRemindersResultV1 {
        reminders: items.iter().map(wire_reminder).collect(),
        next_after_reminder_id: next.map_or_else(Vec::new, |value| value.to_vec()),
    }
    .encode_to_vec())
}

async fn list_outcomes(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let (event, after, limit) = load_child_request(persistence, logical_owner_id, bytes).await?;
    let (items, next) = paginate_children(event.outcomes, after, limit, |value| value.outcome_id);
    Ok(ListCalendarOutcomesResultV1 {
        outcomes: items.iter().map(wire_outcome).collect(),
        next_after_outcome_id: next.map_or_else(Vec::new, |value| value.to_vec()),
    }
    .encode_to_vec())
}

async fn load_child_request(
    persistence: &CalendarPersistenceV1,
    logical_owner_id: &str,
    bytes: &[u8],
) -> Result<(CalendarEventRecordV1, Option<[u8; 16]>, usize), &'static str> {
    let mut value = exact_decode::<CalendarEventChildListRequestV1>(bytes)?;
    accept_owner(&mut value.logical_owner_id, logical_owner_id)?;
    let event = persistence
        .get_event(logical_owner_id, id16(&value.calendar_event_id)?)
        .await
        .map_err(persistence_error)?
        .ok_or("NOT_FOUND")?;
    Ok((
        event,
        optional_id16(&value.after_id)?,
        usize::from(checked_limit(value.limit)?),
    ))
}

fn paginate_events(
    events: &mut Vec<CalendarEventRecordV1>,
    limit: u16,
) -> ListCalendarEventsResultV1 {
    let has_more = events.len() > usize::from(limit);
    events.truncate(usize::from(limit));
    ListCalendarEventsResultV1 {
        events: events.iter().map(wire_event).collect(),
        next_after_calendar_event_id: if has_more {
            events
                .last()
                .map_or_else(Vec::new, |value| value.calendar_event_id.to_vec())
        } else {
            Vec::new()
        },
    }
}

fn paginate_children<T>(
    values: Vec<T>,
    after: Option<[u8; 16]>,
    limit: usize,
    id: impl Fn(&T) -> [u8; 16],
) -> (Vec<T>, Option<[u8; 16]>) {
    let mut filtered = values
        .into_iter()
        .filter(|value| after.is_none_or(|after| id(value) > after))
        .collect::<Vec<_>>();
    let has_more = filtered.len() > limit;
    filtered.truncate(limit);
    let next = has_more.then(|| id(filtered.last().expect("nonempty bounded page")));
    (filtered, next)
}

fn wire_event(value: &CalendarEventRecordV1) -> WireEvent {
    WireEvent {
        calendar_event_id: value.calendar_event_id.to_vec(),
        logical_owner_id: value.logical_owner_id.clone(),
        title: value.title.clone(),
        description: value.description.clone(),
        starts_at: Some(wire_timestamp(value.starts_at)),
        ends_at: Some(wire_timestamp(value.ends_at)),
        timezone: value.timezone.clone(),
        state: encode_event_state(value.state),
        event_revision: value.event_revision,
        constraints: value
            .constraints
            .as_ref()
            .map(|constraints| CalendarConstraintsV1 {
                earliest_start: Some(wire_timestamp(constraints.earliest_start)),
                latest_end: Some(wire_timestamp(constraints.latest_end)),
                minimum_duration_minutes: constraints.minimum_duration_minutes,
                timezone: constraints.timezone.clone(),
                updated_at_event_revision: constraints.updated_at_event_revision,
            }),
        created_at: Some(wire_timestamp(value.created_at)),
        updated_at: Some(wire_timestamp(value.updated_at)),
    }
}

fn wire_participant(value: &CalendarParticipantV1) -> WireParticipant {
    WireParticipant {
        participant_id: value.participant_id.to_vec(),
        display_name: value.display_name.clone(),
        address: value.address.clone(),
        role: match value.role {
            CalendarParticipantRoleV1::Organizer => {
                WireParticipantRole::CalendarParticipantRoleOrganizer as i32
            }
            CalendarParticipantRoleV1::Required => {
                WireParticipantRole::CalendarParticipantRoleRequired as i32
            }
            CalendarParticipantRoleV1::Optional => {
                WireParticipantRole::CalendarParticipantRoleOptional as i32
            }
        },
        response: match value.response {
            CalendarParticipantResponseV1::Pending => {
                WireParticipantResponse::CalendarParticipantResponsePending as i32
            }
            CalendarParticipantResponseV1::Accepted => {
                WireParticipantResponse::CalendarParticipantResponseAccepted as i32
            }
            CalendarParticipantResponseV1::Declined => {
                WireParticipantResponse::CalendarParticipantResponseDeclined as i32
            }
            CalendarParticipantResponseV1::Tentative => {
                WireParticipantResponse::CalendarParticipantResponseTentative as i32
            }
        },
        updated_at_event_revision: value.updated_at_event_revision,
    }
}

fn wire_reminder(value: &CalendarReminderV1) -> WireReminder {
    WireReminder {
        reminder_id: value.reminder_id.to_vec(),
        due_at: Some(wire_timestamp(value.due_at)),
        state: match value.state {
            CalendarReminderStateV1::Pending => {
                WireReminderState::CalendarReminderStatePending as i32
            }
            CalendarReminderStateV1::Fired => WireReminderState::CalendarReminderStateFired as i32,
            CalendarReminderStateV1::Cancelled => {
                WireReminderState::CalendarReminderStateCancelled as i32
            }
        },
        updated_at_event_revision: value.updated_at_event_revision,
    }
}

fn wire_outcome(value: &CalendarOutcomeV1) -> WireOutcome {
    WireOutcome {
        outcome_id: value.outcome_id.to_vec(),
        kind: match value.kind {
            CalendarOutcomeKindV1::Completed => {
                WireOutcomeKind::CalendarOutcomeKindCompleted as i32
            }
            CalendarOutcomeKindV1::Cancelled => {
                WireOutcomeKind::CalendarOutcomeKindCancelled as i32
            }
            CalendarOutcomeKindV1::NoShow => WireOutcomeKind::CalendarOutcomeKindNoShow as i32,
        },
        note: value.note.clone(),
        recorded_at: Some(wire_timestamp(value.recorded_at)),
        recorded_at_event_revision: value.recorded_at_event_revision,
    }
}

fn wire_timestamp(value: CalendarTimestampV1) -> TimestampV1 {
    TimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    }
}

fn raw_timestamp(value: Option<TimestampV1>) -> Result<CalendarTimestampV1, &'static str> {
    raw_wire_timestamp(value.ok_or("INVALID_ARGUMENT")?)
}

fn raw_wire_timestamp(value: TimestampV1) -> Result<CalendarTimestampV1, &'static str> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err("INVALID_ARGUMENT");
    }
    Ok(CalendarTimestampV1 {
        unix_seconds: value.unix_seconds,
        nanos: value.nanos,
    })
}

fn checked_timestamp(
    value: Option<TimestampV1>,
    now_unix_millis: i64,
) -> Result<CalendarTimestampV1, &'static str> {
    let value = raw_timestamp(value)?;
    if timestamp_millis(value).map_err(|_| "INVALID_ARGUMENT")? > now_unix_millis {
        return Err("INVALID_ARGUMENT");
    }
    Ok(value)
}

fn timestamp_millis(value: CalendarTimestampV1) -> Result<i64, CalendarPersistenceErrorV1> {
    value
        .unix_seconds
        .checked_mul(1_000)
        .and_then(|base| base.checked_add(i64::from(value.nanos / 1_000_000)))
        .filter(|value| *value > 0)
        .ok_or(CalendarPersistenceErrorV1::InvalidInput)
}

fn decode_event_state(value: i32) -> Result<CalendarEventStateV1, &'static str> {
    match WireEventState::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireEventState::CalendarEventStateScheduled => Ok(CalendarEventStateV1::Scheduled),
        WireEventState::CalendarEventStateCompleted => Ok(CalendarEventStateV1::Completed),
        WireEventState::CalendarEventStateCancelled => Ok(CalendarEventStateV1::Cancelled),
        WireEventState::CalendarEventStateUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn decode_participant_role(value: i32) -> Result<CalendarParticipantRoleV1, &'static str> {
    match WireParticipantRole::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireParticipantRole::CalendarParticipantRoleOrganizer => {
            Ok(CalendarParticipantRoleV1::Organizer)
        }
        WireParticipantRole::CalendarParticipantRoleRequired => {
            Ok(CalendarParticipantRoleV1::Required)
        }
        WireParticipantRole::CalendarParticipantRoleOptional => {
            Ok(CalendarParticipantRoleV1::Optional)
        }
        WireParticipantRole::CalendarParticipantRoleUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn decode_participant_response(value: i32) -> Result<CalendarParticipantResponseV1, &'static str> {
    match WireParticipantResponse::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireParticipantResponse::CalendarParticipantResponsePending => {
            Ok(CalendarParticipantResponseV1::Pending)
        }
        WireParticipantResponse::CalendarParticipantResponseAccepted => {
            Ok(CalendarParticipantResponseV1::Accepted)
        }
        WireParticipantResponse::CalendarParticipantResponseDeclined => {
            Ok(CalendarParticipantResponseV1::Declined)
        }
        WireParticipantResponse::CalendarParticipantResponseTentative => {
            Ok(CalendarParticipantResponseV1::Tentative)
        }
        WireParticipantResponse::CalendarParticipantResponseUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn decode_outcome_kind(value: i32) -> Result<CalendarOutcomeKindV1, &'static str> {
    match WireOutcomeKind::try_from(value).map_err(|_| "INVALID_ARGUMENT")? {
        WireOutcomeKind::CalendarOutcomeKindCompleted => Ok(CalendarOutcomeKindV1::Completed),
        WireOutcomeKind::CalendarOutcomeKindCancelled => Ok(CalendarOutcomeKindV1::Cancelled),
        WireOutcomeKind::CalendarOutcomeKindNoShow => Ok(CalendarOutcomeKindV1::NoShow),
        WireOutcomeKind::CalendarOutcomeKindUnspecified => Err("INVALID_ARGUMENT"),
    }
}

fn encode_event_state(value: CalendarEventStateV1) -> i32 {
    match value {
        CalendarEventStateV1::Scheduled => WireEventState::CalendarEventStateScheduled as i32,
        CalendarEventStateV1::Completed => WireEventState::CalendarEventStateCompleted as i32,
        CalendarEventStateV1::Cancelled => WireEventState::CalendarEventStateCancelled as i32,
    }
}

fn exact_decode<T>(bytes: &[u8]) -> Result<T, &'static str>
where
    T: Message + Default,
{
    let value = T::decode(bytes).map_err(|_| "INVALID_ARGUMENT")?;
    if value.encode_to_vec() != bytes {
        return Err("INVALID_ARGUMENT");
    }
    Ok(value)
}

fn accept_owner(payload_owner: &mut String, authenticated_owner: &str) -> Result<(), &'static str> {
    if payload_owner.is_empty() {
        *payload_owner = authenticated_owner.to_owned();
    } else if payload_owner != authenticated_owner {
        return Err("REJECTED");
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], &'static str> {
    let value: [u8; 16] = value.try_into().map_err(|_| "INVALID_ARGUMENT")?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or("INVALID_ARGUMENT")
}

fn optional_id16(value: &[u8]) -> Result<Option<[u8; 16]>, &'static str> {
    if value.is_empty() {
        Ok(None)
    } else {
        id16(value).map(Some)
    }
}

fn positive_revision(value: u64) -> Result<u64, &'static str> {
    (value > 0).then_some(value).ok_or("INVALID_ARGUMENT")
}

fn checked_limit(value: u32) -> Result<u16, &'static str> {
    match value {
        1..=200 => u16::try_from(value).map_err(|_| "INVALID_ARGUMENT"),
        _ => Err("INVALID_ARGUMENT"),
    }
}

fn lifecycle_event_id(
    operation_id: [u8; 16],
    calendar_event_id: [u8; 16],
    revision: u64,
) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"makosh.calendar.lifecycle-event-id.v1\0");
    hash.update(operation_id);
    hash.update(calendar_event_id);
    hash.update(revision.to_be_bytes());
    hash.finalize()[..16].try_into().expect("fixed digest")
}

fn encode_id(value: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(32);
    for byte in value {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn persistence_error(value: CalendarPersistenceErrorV1) -> &'static str {
    match value {
        CalendarPersistenceErrorV1::NotFound => "NOT_FOUND",
        CalendarPersistenceErrorV1::RevisionConflict => "REVISION_CONFLICT",
        CalendarPersistenceErrorV1::OperationConflict
        | CalendarPersistenceErrorV1::OutboxConflict => "CONFLICT",
        CalendarPersistenceErrorV1::InvalidInput | CalendarPersistenceErrorV1::InvalidRow => {
            "INVALID_ARGUMENT"
        }
        CalendarPersistenceErrorV1::StorageUnavailable => "UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_uses_last_returned_id_without_skipping_overflow() {
        let mut events = (1_u8..=3)
            .map(|id| CalendarEventRecordV1 {
                calendar_event_id: [id; 16],
                logical_owner_id: "owner-1".to_owned(),
                title: format!("Event {id}"),
                description: String::new(),
                starts_at: CalendarTimestampV1 {
                    unix_seconds: 10,
                    nanos: 0,
                },
                ends_at: CalendarTimestampV1 {
                    unix_seconds: 20,
                    nanos: 0,
                },
                timezone: "UTC".to_owned(),
                state: CalendarEventStateV1::Scheduled,
                event_revision: 1,
                participants: Vec::new(),
                constraints: None,
                reminders: Vec::new(),
                outcomes: Vec::new(),
                created_at: CalendarTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
                updated_at: CalendarTimestampV1 {
                    unix_seconds: 1,
                    nanos: 0,
                },
            })
            .collect();
        let page = paginate_events(&mut events, 2);
        assert_eq!(page.events.len(), 2);
        assert_eq!(page.next_after_calendar_event_id, vec![2; 16]);
    }

    #[test]
    fn empty_payload_owner_is_injected_but_conflict_is_rejected() {
        let mut empty = String::new();
        accept_owner(&mut empty, "owner-1").expect("owner");
        assert_eq!(empty, "owner-1");
        let mut conflicting = "owner-2".to_owned();
        assert_eq!(accept_owner(&mut conflicting, "owner-1"), Err("REJECTED"));
    }
}

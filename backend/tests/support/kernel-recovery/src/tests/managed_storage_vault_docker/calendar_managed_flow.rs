//! Actual Calendar lifecycle, Scheduler reminder, restart and owner-RLS contour.

use super::*;

use std::time::{Duration, Instant};

use makosh_calendar_api::{
    CALENDAR_ADD_PARTICIPANT_CONNECT_PATH_V1, CALENDAR_ADD_REMINDER_CONNECT_PATH_V1,
    CALENDAR_CLIENT_CAPABILITY_ID_V1, CALENDAR_GET_CONNECT_PATH_V1, CALENDAR_LIST_CONNECT_PATH_V1,
    CALENDAR_LIST_REMINDERS_CONNECT_PATH_V1, CALENDAR_MODULE_ID_V1, CALENDAR_OWNER_ID_V1,
    CALENDAR_RECORD_OUTCOME_CONNECT_PATH_V1, CALENDAR_SEARCH_CONNECT_PATH_V1,
    CALENDAR_SET_CONSTRAINTS_CONNECT_PATH_V1, CALENDAR_SET_STATE_CONNECT_PATH_V1,
    calendar_client_add_participant_contract_reference_v1,
    calendar_client_add_reminder_contract_reference_v1,
    calendar_client_create_contract_reference_v1, calendar_client_get_contract_reference_v1,
    calendar_client_list_contract_reference_v1,
    calendar_client_list_reminders_contract_reference_v1,
    calendar_client_record_outcome_contract_reference_v1,
    calendar_client_search_contract_reference_v1,
    calendar_client_set_constraints_contract_reference_v1,
    calendar_client_set_state_contract_reference_v1,
    client_wire::{
        AddCalendarParticipantRequestV1, AddCalendarReminderRequestV1,
        CalendarEventChildListRequestV1, CalendarEventMutationResultV1, CalendarEventStateV1,
        CalendarEventV1, CalendarOutcomeKindV1, CalendarParticipantResponseV1,
        CalendarParticipantRoleV1, CalendarReminderStateV1, CreateCalendarEventRequestV1,
        GetCalendarEventRequestV1, ListCalendarEventsRequestV1, ListCalendarEventsResultV1,
        ListCalendarRemindersResultV1, RecordCalendarOutcomeRequestV1,
        SearchCalendarEventsRequestV1, SetCalendarConstraintsRequestV1,
        SetCalendarEventStateRequestV1, TimestampV1,
    },
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};

use crate::identity::device::signer::DeviceSigner;

const PRIVATE_TITLE_V1: &str = "calendar-private-title-marker";
const PRIVATE_DESCRIPTION_V1: &str = "calendar-private-description-marker";
const PRIVATE_ADDRESS_V1: &str = "calendar-private-address@example.invalid";
const PRIVATE_OUTCOME_V1: &str = "calendar-private-outcome-marker";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Scheduler and Calendar binaries"]
fn managed_calendar_lifecycle_reminder_replays_and_restarts_with_owner_rls() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-calendar");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_calendar_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_calendar_store_v1(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Calendar owner signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            CALENDAR_LOGICAL_HUMAN_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Calendar logical owner");
    let admitted = admit_calendar_runtime_v1(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Calendar Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_scheduler_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &scheduler_binding(&store),
    )
    .expect("provision Scheduler Storage binding");
    let admitted = prepare_calendar_runtime_v1(&supervisor, &store, admitted);
    configure_communications_jetstream(&store);
    let scheduler_reservation = managed_launch::load(&supervisor, &store, SCHEDULER_REGISTRATION)
        .expect("load Scheduler reservation");
    assert_eq!(
        scheduler_launch::start_from_reservation(
            &supervisor,
            &store,
            release.kernel(),
            &root.join("runtime"),
            scheduler_reservation,
            &scheduler_binding(&store),
        )
        .expect("start Scheduler with Calendar grant"),
        1
    );
    let calendar = start_calendar_runtime_v1(&supervisor, &store, &root.join("runtime"), admitted);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Calendar wall clock")
        .as_millis() as i64;
    let timestamp = |millis: i64| TimestampV1 {
        unix_seconds: millis / 1_000,
        nanos: ((millis % 1_000) * 1_000_000) as i32,
    };
    let create = CreateCalendarEventRequestV1 {
        operation_id: vec![0x11; 16],
        logical_owner_id: String::new(),
        title: PRIVATE_TITLE_V1.to_owned(),
        description: PRIVATE_DESCRIPTION_V1.to_owned(),
        starts_at: Some(timestamp(now + 60_000)),
        ends_at: Some(timestamp(now + 3_600_000)),
        timezone: "Europe/Madrid".to_owned(),
        created_at: Some(timestamp(now)),
    };
    let first: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        1,
        calendar_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    let replayed: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        2,
        calendar_client_create_contract_reference_v1(),
        create.encode_to_vec(),
    );
    assert_eq!(first, replayed, "exact create replay response");
    let event = first.event.expect("created Calendar event");
    assert_eq!(event.event_revision, 1);

    let participant: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        3,
        calendar_client_add_participant_contract_reference_v1(),
        AddCalendarParticipantRequestV1 {
            operation_id: vec![0x12; 16],
            calendar_event_id: event.calendar_event_id.clone(),
            logical_owner_id: String::new(),
            expected_event_revision: 1,
            display_name: "Private participant".to_owned(),
            address: PRIVATE_ADDRESS_V1.to_owned(),
            role: CalendarParticipantRoleV1::CalendarParticipantRoleRequired as i32,
            response: CalendarParticipantResponseV1::CalendarParticipantResponseAccepted as i32,
            changed_at: Some(timestamp(now + 1)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        participant
            .event
            .as_ref()
            .expect("participant event")
            .event_revision,
        2
    );
    let constrained: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        4,
        calendar_client_set_constraints_contract_reference_v1(),
        SetCalendarConstraintsRequestV1 {
            operation_id: vec![0x13; 16],
            calendar_event_id: event.calendar_event_id.clone(),
            logical_owner_id: String::new(),
            expected_event_revision: 2,
            earliest_start: Some(timestamp(now + 30_000)),
            latest_end: Some(timestamp(now + 7_200_000)),
            minimum_duration_minutes: 30,
            timezone: "Europe/Madrid".to_owned(),
            changed_at: Some(timestamp(now + 2)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        constrained
            .event
            .as_ref()
            .expect("constrained event")
            .event_revision,
        3
    );
    let reminder: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        5,
        calendar_client_add_reminder_contract_reference_v1(),
        AddCalendarReminderRequestV1 {
            operation_id: vec![0x14; 16],
            calendar_event_id: event.calendar_event_id.clone(),
            logical_owner_id: String::new(),
            expected_event_revision: 3,
            due_at: Some(timestamp(now + 3_000)),
            changed_at: Some(timestamp(now + 3)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        reminder
            .event
            .as_ref()
            .expect("reminder event")
            .event_revision,
        4
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let reminders: ListCalendarRemindersResultV1 = route_calendar_v1(
            &store,
            &supervisor,
            &calendar,
            6,
            calendar_client_list_reminders_contract_reference_v1(),
            CalendarEventChildListRequestV1 {
                logical_owner_id: String::new(),
                calendar_event_id: event.calendar_event_id.clone(),
                after_id: Vec::new(),
                limit: 8,
            }
            .encode_to_vec(),
        );
        if reminders.reminders.len() == 1
            && reminders.reminders[0].state
                == CalendarReminderStateV1::CalendarReminderStateFired as i32
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Scheduler did not fire Calendar reminder"
        );
        std::thread::sleep(Duration::from_millis(100));
    }

    let completed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Calendar completion clock")
        .as_millis() as i64;
    let completed: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        7,
        calendar_client_set_state_contract_reference_v1(),
        SetCalendarEventStateRequestV1 {
            operation_id: vec![0x15; 16],
            calendar_event_id: event.calendar_event_id.clone(),
            logical_owner_id: String::new(),
            expected_event_revision: 5,
            state: CalendarEventStateV1::CalendarEventStateCompleted as i32,
            changed_at: Some(timestamp(completed_at)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        completed
            .event
            .as_ref()
            .expect("completed event")
            .event_revision,
        6
    );
    let outcome: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        8,
        calendar_client_record_outcome_contract_reference_v1(),
        RecordCalendarOutcomeRequestV1 {
            operation_id: vec![0x16; 16],
            calendar_event_id: event.calendar_event_id.clone(),
            logical_owner_id: String::new(),
            expected_event_revision: 6,
            kind: CalendarOutcomeKindV1::CalendarOutcomeKindCompleted as i32,
            note: PRIVATE_OUTCOME_V1.to_owned(),
            recorded_at: Some(timestamp(completed_at)),
        }
        .encode_to_vec(),
    );
    assert_eq!(
        outcome
            .event
            .as_ref()
            .expect("outcome event")
            .event_revision,
        7
    );

    let searched: ListCalendarEventsResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        9,
        calendar_client_search_contract_reference_v1(),
        SearchCalendarEventsRequestV1 {
            logical_owner_id: String::new(),
            query: "private-title".to_owned(),
            after_calendar_event_id: Vec::new(),
            limit: 8,
        }
        .encode_to_vec(),
    );
    assert_eq!(searched.events.len(), 1);

    let second: CalendarEventMutationResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        10,
        calendar_client_create_contract_reference_v1(),
        CreateCalendarEventRequestV1 {
            operation_id: vec![0x17; 16],
            logical_owner_id: String::new(),
            title: "Calendar secondary event".to_owned(),
            description: String::new(),
            starts_at: Some(timestamp(now + 120_000)),
            ends_at: Some(timestamp(now + 7_200_000)),
            timezone: "Europe/Madrid".to_owned(),
            created_at: Some(timestamp(completed_at)),
        }
        .encode_to_vec(),
    );
    let second_event_id = second
        .event
        .expect("second Calendar event")
        .calendar_event_id;
    let first_page: ListCalendarEventsResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        11,
        calendar_client_list_contract_reference_v1(),
        ListCalendarEventsRequestV1 {
            logical_owner_id: String::new(),
            after_calendar_event_id: Vec::new(),
            limit: 1,
        }
        .encode_to_vec(),
    );
    assert_eq!(first_page.events.len(), 1);
    assert_eq!(first_page.next_after_calendar_event_id.len(), 16);
    let second_page: ListCalendarEventsResultV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        12,
        calendar_client_list_contract_reference_v1(),
        ListCalendarEventsRequestV1 {
            logical_owner_id: String::new(),
            after_calendar_event_id: first_page.next_after_calendar_event_id,
            limit: 1,
        }
        .encode_to_vec(),
    );
    assert_eq!(second_page.events.len(), 1);
    assert!(second_page.next_after_calendar_event_id.is_empty());
    let mut paged_ids = vec![
        first_page.events[0].calendar_event_id.clone(),
        second_page.events[0].calendar_event_id.clone(),
    ];
    paged_ids.sort();
    paged_ids.dedup();
    assert_eq!(paged_ids.len(), 2, "Calendar pages must not duplicate rows");
    assert!(paged_ids.contains(&event.calendar_event_id));
    assert!(paged_ids.contains(&second_event_id));

    let before_restart = durable_calendar_snapshot_v1();
    assert_eq!(before_restart.0, 2);
    assert_eq!(
        before_restart.1, 7,
        "exact replay must not duplicate operation"
    );
    assert!(before_restart.2 >= 2, "Scheduler result and due inbox rows");
    assert_eq!(before_restart.4, 0, "Calendar relay must drain");
    assert_public_calendar_outbox_is_private_free_v1();

    let calendar =
        restart_calendar_runtime_v1(&supervisor, &store, &root.join("runtime"), calendar);
    let restarted: CalendarEventV1 = route_calendar_v1(
        &store,
        &supervisor,
        &calendar,
        13,
        calendar_client_get_contract_reference_v1(),
        GetCalendarEventRequestV1 {
            logical_owner_id: String::new(),
            calendar_event_id: event.calendar_event_id,
        }
        .encode_to_vec(),
    );
    assert_eq!(restarted.event_revision, 7);
    assert_eq!(durable_calendar_snapshot_v1(), before_restart);
    assert!(
        supervisor
            .is_active(&calendar.registration_id)
            .expect("Calendar active state")
    );
    assert_eq!(supervisor.last_failure(&calendar.registration_id), Ok(None));

    // Effective NOLOGIN/NOSUPERUSER/NOBYPASSRLS coverage spans all eight Calendar tables.
    tokio::runtime::Runtime::new()
        .expect("Calendar RLS runtime")
        .block_on(assert_review_owner_rls_v1(
            "makosh_calendar_rls_test",
            &[
                "calendar_events",
                "calendar_participants",
                "calendar_constraints",
                "calendar_reminders",
                "calendar_outcomes",
                "calendar_client_operations",
                "calendar_scheduler_inbox",
                "calendar_outbox",
            ],
        ));

    supervisor.shutdown().expect("stop Calendar contour");
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Calendar fixture");
}

fn route_calendar_v1<T: Message + Default>(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    calendar: &StartedCalendarRuntimeV1,
    request_id: u64,
    contract: makosh_runtime_protocol::v1::ContractReferenceV1,
    payload: Vec<u8>,
) -> T {
    let request = ModuleClientRequestV1 {
        protocol_major: 1,
        module_id: CALENDAR_MODULE_ID_V1.to_owned(),
        owner_id: CALENDAR_OWNER_ID_V1.to_owned(),
        contract: Some(contract),
        request_id,
        request_payload: payload,
        logical_owner_id: CALENDAR_LOGICAL_HUMAN_OWNER_ID_V1.to_owned(),
        authenticated_device_id: "desktop-1".to_owned(),
        authenticated_client_session_id: "session-1".to_owned(),
    }
    .encode_to_vec();
    let route = crate::modules::capability::router::ManagedCapabilityRouteRequest::new(
        &calendar.registration_id,
        &calendar.runtime_instance_id,
        calendar.runtime_generation,
        calendar.grant_epoch,
        CALENDAR_CLIENT_CAPABILITY_ID_V1,
        &request,
    );
    let bytes = crate::modules::capability::router::route_managed_client_request(
        store,
        &supervisor.relay_port(),
        &route,
    )
    .expect("route authenticated Calendar client request");
    let response = ModuleClientResponseV1::decode(bytes.as_slice()).expect("Calendar response");
    assert!(
        response.error_code.is_empty(),
        "Calendar request {request_id} failed: {}",
        response.error_code
    );
    T::decode(response.response_payload.as_slice()).expect("decode Calendar response payload")
}

fn durable_calendar_snapshot_v1() -> (i64, i64, i64, i64, i64) {
    tokio::runtime::Runtime::new()
        .expect("Calendar SQL runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            sqlx::query_as(
                "SELECT \
                 (SELECT count(*) FROM makosh_data.calendar_events WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.calendar_client_operations WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.calendar_scheduler_inbox WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.calendar_outbox WHERE logical_owner_id='owner-1'), \
                 (SELECT count(*) FROM makosh_data.calendar_outbox WHERE logical_owner_id='owner-1' AND published_at_unix_millis IS NULL)",
            )
            .fetch_one(&pool)
            .await
            .expect("Calendar durable snapshot")
        })
}

fn assert_public_calendar_outbox_is_private_free_v1() {
    tokio::runtime::Runtime::new()
        .expect("Calendar privacy runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            let rows: Vec<Vec<u8>> = sqlx::query_scalar(
                "SELECT envelope_bytes FROM makosh_data.calendar_outbox \
                 WHERE logical_owner_id='owner-1' ORDER BY outbox_sequence",
            )
            .fetch_all(&pool)
            .await
            .expect("Calendar public outbox bytes");
            assert!(!rows.is_empty());
            for row in rows {
                for marker in [
                    PRIVATE_TITLE_V1,
                    PRIVATE_DESCRIPTION_V1,
                    PRIVATE_ADDRESS_V1,
                    PRIVATE_OUTCOME_V1,
                ] {
                    assert!(
                        !row.windows(marker.len())
                            .any(|window| window == marker.as_bytes()),
                        "private Calendar marker escaped durable public outbox"
                    );
                }
            }
        });
}

const _: &str = CALENDAR_ADD_PARTICIPANT_CONNECT_PATH_V1;
const _: &str = CALENDAR_ADD_REMINDER_CONNECT_PATH_V1;
const _: &str = CALENDAR_GET_CONNECT_PATH_V1;
const _: &str = CALENDAR_LIST_CONNECT_PATH_V1;
const _: &str = CALENDAR_LIST_REMINDERS_CONNECT_PATH_V1;
const _: &str = CALENDAR_RECORD_OUTCOME_CONNECT_PATH_V1;
const _: &str = CALENDAR_SEARCH_CONNECT_PATH_V1;
const _: &str = CALENDAR_SET_CONSTRAINTS_CONNECT_PATH_V1;
const _: &str = CALENDAR_SET_STATE_CONNECT_PATH_V1;

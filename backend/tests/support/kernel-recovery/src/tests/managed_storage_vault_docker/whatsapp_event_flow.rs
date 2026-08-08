//! Live WhatsApp host execution and event-only Communications handoff.

use std::time::Instant;

use super::*;

use makosh_runtime_protocol::v1::ModuleClientResponseV1;
use makosh_whatsapp_api::{
    WhatsAppDialog, WhatsAppMessage, WhatsAppParticipant, WhatsAppProviderCommand,
    WhatsAppProviderCommandStateV1, WhatsAppProviderEvent, WhatsAppProviderEventKind,
    WhatsAppPublicClientRequestV1, WhatsAppPublicClientResponseV1,
    client_contract::WhatsAppClientContractV1,
    host_bridge::{
        HOST_BRIDGE_PROTOCOL_MAJOR, HOST_BRIDGE_PROTOCOL_REVISION, WhatsAppHostBridgeEnvelopeV1,
        WhatsAppHostObservationV1,
    },
    operational::{WhatsAppOperationalQueryResponseV1, WhatsAppOperationalQueryV1},
    realtime::{
        MAX_OPERATIONAL_REPLAY_LIMIT, WhatsAppOperationalReplayRequestV1,
        WhatsAppOperationalReplayResponseV1,
    },
};
use makosh_whatsapp_runtime::client_port::{decode_module_response, encode_module_request};
use prost::Message as _;

use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

const OBSERVATION_SUBJECT: &str = "makosh.observation.v1.communications.communication_observed.v1";
const CANONICAL_EVENT_SUBJECT: &str =
    "makosh.event.v1.communications.communication_evidence_recorded.v1";
const PRIVATE_COMMAND_TEXT: &str = "private WhatsApp body must stay integration-owned";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, WhatsApp, NATS and Communications binaries"]
fn managed_whatsapp_runtime_delivers_live_command_and_event_only_communications_handoff() {
    let mut contour = ManagedWhatsAppContour::start(WhatsAppGrantProfileV1::CommandAndQuery);
    let events = contour
        .store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("WhatsApp event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (client, mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect WhatsApp event observer");
        let observations = client
            .subscribe(OBSERVATION_SUBJECT)
            .await
            .expect("subscribe WhatsApp observations");
        let canonical_events = client
            .subscribe(CANONICAL_EVENT_SUBJECT)
            .await
            .expect("subscribe canonical Communications events");
        client
            .flush()
            .await
            .expect("activate WhatsApp event observers");
        (client, observations, canonical_events)
    });

    const OPERATION_ID: &str = "managed-whatsapp-live-command-1";
    const HOST_CLAIM_ID: &str = "managed-whatsapp-host-claim-1";
    assert_whatsapp_command_accepted(&contour, OPERATION_ID);
    execute_whatsapp_command(&contour, OPERATION_ID, HOST_CLAIM_ID);
    assert_whatsapp_operation_succeeded(&contour, OPERATION_ID);
    assert_command_result_stays_owner_local(
        &event_runtime,
        &mut observations,
        &mut canonical_events,
    );

    let message_bytes =
        submit_message_observation(&contour, "whatsapp-provider-event-1", "provider-message-1");
    let (observation_bytes, observation_message_id, canonical_message_id) =
        receive_whatsapp_observation(
            &event_runtime,
            &mut observations,
            &mut canonical_events,
            &contour,
            "initial message",
        );
    assert_private_command_text_absent(&observation_bytes);
    assert_ne!(
        observation_bytes, message_bytes,
        "the private host operation payload is not the durable observation envelope",
    );

    event_runtime.block_on(async {
        client
            .publish(OBSERVATION_SUBJECT, observation_bytes.clone().into())
            .await
            .expect("republish exact WhatsApp observation");
        client
            .flush()
            .await
            .expect("flush duplicate WhatsApp observation");
        let duplicate = tokio::time::timeout(Duration::from_secs(1), observations.next())
            .await
            .expect("duplicate WhatsApp observation timeout")
            .expect("duplicate WhatsApp observation");
        let duplicate = decode_envelope_v1(duplicate.payload.as_ref())
            .expect("duplicate WhatsApp observation envelope");
        assert_eq!(duplicate.message_id, observation_message_id);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate WhatsApp observation must not create a second Communications event",
        );
    });
    let initial_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);

    set_authenticated_nats_container_running(false);
    submit_message_observation(&contour, "whatsapp-provider-event-2", "provider-message-2");
    std::thread::sleep(Duration::from_millis(2_500));
    assert!(
        contour
            .supervisor
            .is_active(&contour.whatsapp.registration_id)
            .expect("read managed WhatsApp state"),
        "managed WhatsApp runtime must remain active while NATS is unavailable",
    );
    assert_eq!(
        contour
            .supervisor
            .last_failure(&contour.whatsapp.registration_id)
            .expect("read managed WhatsApp failure"),
        None,
    );
    set_authenticated_nats_container_running(true);
    wait_for_authenticated_nats_reconnect(&event_runtime, &client, "WhatsApp event observer");

    let (_, replayed_observation_id, replayed_canonical_id) = receive_whatsapp_observation(
        &event_runtime,
        &mut observations,
        &mut canonical_events,
        &contour,
        "outage replay",
    );
    assert_ne!(replayed_observation_id, observation_message_id);
    assert_ne!(
        replayed_canonical_id, canonical_message_id,
        "outage replay must deliver the second WhatsApp provider observation",
    );
    let replayed_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);
    assert_ne!(
        replayed_evidence_id, initial_evidence_id,
        "Communications query must expose the replayed WhatsApp evidence",
    );
    assert_whatsapp_operational_read(&mut contour);

    contour.shutdown_processes();
    contour.finish();
}

fn assert_whatsapp_operational_read(contour: &mut ManagedWhatsAppContour) {
    let mut host = WhatsAppHostBridgeTestClient::connect(&contour.whatsapp);
    let message = |body: &str| WhatsAppMessage {
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_chat_id: "provider-chat-operational".to_owned(),
        provider_message_id: "provider-message-operational".to_owned(),
        sender_id: "provider-sender-operational".to_owned(),
        sender_display_name: "Operational Sender".to_owned(),
        text: Some(body.to_owned()),
        reply_to_provider_message_id: None,
        occurred_at_unix_seconds: 1_785_000_100,
        delivery_state: None,
    };
    let initial = WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-message-initial".to_owned(),
        observed_at_unix_seconds: 1_785_000_100,
        observation: WhatsAppHostObservationV1::OperationalMessage(message("initial body")),
    };
    assert_eq!(host.submit_observation(&initial), initial.provider_event_id);
    assert_eq!(
        host.submit_observation(&initial),
        initial.provider_event_id,
        "exact duplicate host delivery is idempotent",
    );
    for (event_id, observed_at, body) in [
        (
            "whatsapp-operational-message-newer",
            1_785_000_102,
            "newest searchable body",
        ),
        (
            "whatsapp-operational-message-older",
            1_785_000_101,
            "stale body must not win",
        ),
    ] {
        assert_eq!(
            host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
                protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
                protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_event_id: event_id.to_owned(),
                observed_at_unix_seconds: observed_at,
                observation: WhatsAppHostObservationV1::OperationalMessage(message(body)),
            }),
            event_id,
        );
    }
    let dialog = WhatsAppDialog {
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_chat_id: "provider-chat-operational".to_owned(),
        title: "Operational Dialog".to_owned(),
        kind: "group".to_owned(),
        is_archived: Some(false),
        is_pinned: Some(true),
        is_muted: Some(false),
        is_unread: Some(true),
        unread_count: Some(3),
        participant_count: Some(2),
        observed_at_unix_seconds: 1_785_000_103,
    };
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-dialog".to_owned(),
        observed_at_unix_seconds: 1_785_000_103,
        observation: WhatsAppHostObservationV1::OperationalDialog(dialog.clone()),
    });
    let participant = WhatsAppParticipant {
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_chat_id: "provider-chat-operational".to_owned(),
        provider_identity_id: "provider-participant-operational".to_owned(),
        display_name: "Operational Participant".to_owned(),
        role: "member".to_owned(),
        status: "active".to_owned(),
        is_self: false,
        observed_at_unix_seconds: 1_785_000_104,
    };
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-participant".to_owned(),
        observed_at_unix_seconds: 1_785_000_104,
        observation: WhatsAppHostObservationV1::OperationalParticipant(participant.clone()),
    });
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-resync-complete".to_owned(),
        observed_at_unix_seconds: 1_785_000_105,
        observation: WhatsAppHostObservationV1::OperationalResyncState { complete: true },
    });
    drop(host);

    let messages = operational_query(
        contour,
        41,
        WhatsAppOperationalQueryV1::ListMessages {
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            provider_chat_id: Some("provider-chat-operational".to_owned()),
            cursor: None,
            limit: 1,
        },
    );
    let next_cursor = match messages {
        WhatsAppOperationalQueryResponseV1::Messages(page) => {
            assert_eq!(page.items.len(), 1);
            assert_eq!(
                page.items[0].text.as_deref(),
                Some("newest searchable body"),
                "older provider observation must not overwrite the current projection",
            );
            page.next_cursor.expect("bounded message cursor")
        }
        response => panic!("unexpected WhatsApp message response: {response:?}"),
    };
    assert!(matches!(
        operational_query(
            contour,
            42,
            WhatsAppOperationalQueryV1::ListMessages {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: Some("provider-chat-operational".to_owned()),
                cursor: Some(next_cursor),
                limit: 1,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Messages(page) if page.items.is_empty()
    ));
    assert!(matches!(
        operational_query(
            contour,
            43,
            WhatsAppOperationalQueryV1::SearchMessages {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: None,
                query: "searchable".to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Messages(page)
            if page.items.len() == 1
                && page.items[0].provider_message_id == "provider-message-operational"
    ));
    assert!(matches!(
        operational_query(
            contour,
            44,
            WhatsAppOperationalQueryV1::ListDialogs {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Dialogs(page) if page.items == [dialog]
    ));
    assert!(matches!(
        operational_query(
            contour,
            45,
            WhatsAppOperationalQueryV1::ListParticipants {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: "provider-chat-operational".to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Participants(page)
            if page.items == [participant.clone()]
    ));
    assert!(matches!(
        operational_query(
            contour,
            46,
            WhatsAppOperationalQueryV1::ListEvents {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                kind: Some(WhatsAppProviderEventKind::Message),
                provider_chat_id: Some("provider-chat-operational".to_owned()),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Events(page)
            if page.items.len() == 3
                && page.items.iter().all(
                    |event| matches!(event, WhatsAppProviderEvent::MessageObserved(_))
                )
    ));
    assert!(matches!(
        operational_query(
            contour,
            47,
            WhatsAppOperationalQueryV1::GetRuntimeStatus {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            },
        ),
        WhatsAppOperationalQueryResponseV1::RuntimeStatus(status)
            if status.projection_ready && status.latest_event_sequence >= 5
    ));
    let mut host = WhatsAppHostBridgeTestClient::connect(&contour.whatsapp);
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-message-deleted".to_owned(),
        observed_at_unix_seconds: 1_785_000_106,
        observation: WhatsAppHostObservationV1::MessageDeleted {
            provider_chat_id: "provider-chat-operational".to_owned(),
            provider_message_id: "provider-message-operational".to_owned(),
        },
    });
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-message-stale-after-delete".to_owned(),
        observed_at_unix_seconds: 1_785_000_105,
        observation: WhatsAppHostObservationV1::OperationalMessage(message(
            "stale body must not resurrect",
        )),
    });
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-participant-removed".to_owned(),
        observed_at_unix_seconds: 1_785_000_106,
        observation: WhatsAppHostObservationV1::OperationalParticipantRemoved {
            provider_chat_id: participant.provider_chat_id.clone(),
            provider_identity_id: participant.provider_identity_id.clone(),
        },
    });
    let mut stale_participant = participant.clone();
    stale_participant.observed_at_unix_seconds = 1_785_000_105;
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-participant-stale-after-remove".to_owned(),
        observed_at_unix_seconds: stale_participant.observed_at_unix_seconds,
        observation: WhatsAppHostObservationV1::OperationalParticipant(stale_participant),
    });
    let mut restart_message = message("durable body survives restart");
    restart_message.provider_message_id = "provider-message-restart".to_owned();
    restart_message.occurred_at_unix_seconds = 1_785_000_107;
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-message-restart".to_owned(),
        observed_at_unix_seconds: 1_785_000_107,
        observation: WhatsAppHostObservationV1::OperationalMessage(restart_message),
    });
    host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: "whatsapp-operational-message-receipt".to_owned(),
        observed_at_unix_seconds: 1_785_000_108,
        observation: WhatsAppHostObservationV1::Receipt {
            provider_chat_id: "provider-chat-operational".to_owned(),
            provider_message_id: "provider-message-restart".to_owned(),
            delivery_state: "delivered".to_owned(),
        },
    });
    drop(host);
    assert!(matches!(
        operational_query(
            contour,
            51,
            WhatsAppOperationalQueryV1::SearchMessages {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: None,
                query: "stale body".to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Messages(page) if page.items.is_empty()
    ));
    assert!(matches!(
        operational_query(
            contour,
            52,
            WhatsAppOperationalQueryV1::ListParticipants {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: "provider-chat-operational".to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Participants(page) if page.items.is_empty()
    ));
    assert!(matches!(
        operational_query(
            contour,
            53,
            WhatsAppOperationalQueryV1::SearchMessages {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: None,
                query: "survives restart".to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Messages(page)
            if page.items.len() == 1
                && page.items[0].provider_message_id == "provider-message-restart"
                && page.items[0].delivery_state.as_deref() == Some("delivered")
    ));
    let replay_cursor = assert_whatsapp_operational_replay(contour);
    let predecessor_generation = contour.whatsapp.runtime_generation;
    contour.whatsapp = restart_whatsapp_runtime(
        &contour.supervisor,
        contour.store.as_ref(),
        &contour.data,
        &contour.data.join("runtime"),
        &contour.whatsapp,
    );
    assert_eq!(
        contour.whatsapp.runtime_generation,
        predecessor_generation + 1,
    );
    assert!(matches!(
        operational_query(
            contour,
            49,
            WhatsAppOperationalQueryV1::SearchMessages {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: None,
                query: "survives restart".to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Messages(page)
            if page.items.len() == 1
                && page.items[0].provider_message_id == "provider-message-restart"
                && page.items[0].delivery_state.as_deref() == Some("delivered")
    ));
    assert!(matches!(
        operational_query(
            contour,
            50,
            WhatsAppOperationalQueryV1::GetRuntimeStatus {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            },
        ),
        WhatsAppOperationalQueryResponseV1::RuntimeStatus(status)
            if status.projection_ready && status.latest_event_sequence >= 11
    ));
    assert!(matches!(
        operational_query(
            contour,
            54,
            WhatsAppOperationalQueryV1::ListParticipants {
                account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
                provider_chat_id: "provider-chat-operational".to_owned(),
                cursor: None,
                limit: 10,
            },
        ),
        WhatsAppOperationalQueryResponseV1::Participants(page) if page.items.is_empty()
    ));
    assert_whatsapp_operational_replay_after_restart(contour, replay_cursor);
    assert_cross_account_operational_query_is_rejected(contour);
    assert_cross_account_operational_replay_is_rejected(contour);
}

fn assert_whatsapp_operational_replay(contour: &ManagedWhatsAppContour) -> u64 {
    let first = operational_replay(
        contour,
        55,
        WhatsAppOperationalReplayRequestV1 {
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            after_sequence: 0,
            limit: 4,
        },
    );
    assert_eq!(first.account_id, WHATSAPP_ACCOUNT_ID);
    assert!(!first.reset_required);
    assert_eq!(first.frames.len(), 4, "replay page must honor its limit");
    assert!(
        first
            .frames
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence),
        "replay frames must be strictly ascending",
    );
    let first_cursor = first.next_sequence;
    let earliest = first
        .earliest_available_sequence
        .expect("earliest replay sequence");
    let latest = first
        .latest_available_sequence
        .expect("latest replay sequence");
    assert_eq!(
        first.frames.first().map(|frame| frame.sequence),
        Some(earliest)
    );

    let remainder = operational_replay(
        contour,
        56,
        WhatsAppOperationalReplayRequestV1 {
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            after_sequence: first_cursor,
            limit: MAX_OPERATIONAL_REPLAY_LIMIT,
        },
    );
    assert!(!remainder.reset_required);
    assert!(
        remainder
            .frames
            .iter()
            .all(|frame| frame.sequence > first_cursor),
        "next page must not duplicate the previous cursor",
    );
    assert_eq!(
        remainder.frames.last().map(|frame| frame.sequence),
        Some(latest)
    );
    assert!(remainder.frames.iter().any(|frame| {
        matches!(
            frame.event,
            WhatsAppProviderEvent::ParticipantRemoved { .. }
        )
    }));
    assert!(
        remainder
            .frames
            .iter()
            .any(|frame| { matches!(frame.event, WhatsAppProviderEvent::ReceiptChanged { .. }) })
    );
    assert!(
        !format!("{first:?}{remainder:?}").contains(PRIVATE_COMMAND_TEXT),
        "provider command payload must not leak into operational replay",
    );

    let stale = operational_replay(
        contour,
        57,
        WhatsAppOperationalReplayRequestV1 {
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            after_sequence: latest + 1_000,
            limit: 10,
        },
    );
    assert!(stale.reset_required);
    assert!(stale.frames.is_empty());
    assert_eq!(stale.next_sequence, 0);
    assert_eq!(stale.earliest_available_sequence, Some(earliest));
    assert_eq!(stale.latest_available_sequence, Some(latest));
    latest
}

fn assert_whatsapp_operational_replay_after_restart(
    contour: &ManagedWhatsAppContour,
    replay_cursor: u64,
) {
    let caught_up = operational_replay(
        contour,
        58,
        WhatsAppOperationalReplayRequestV1 {
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            after_sequence: replay_cursor,
            limit: 10,
        },
    );
    assert!(!caught_up.reset_required);
    assert!(caught_up.frames.is_empty());
    assert_eq!(caught_up.next_sequence, replay_cursor);
    assert_eq!(caught_up.latest_available_sequence, Some(replay_cursor));

    let restarted = operational_replay(
        contour,
        59,
        WhatsAppOperationalReplayRequestV1 {
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            after_sequence: 0,
            limit: MAX_OPERATIONAL_REPLAY_LIMIT,
        },
    );
    assert!(!restarted.reset_required);
    assert_eq!(
        restarted.frames.last().map(|frame| frame.sequence),
        Some(replay_cursor),
        "successor runtime must replay the persisted predecessor journal",
    );
    assert!(restarted.frames.len() >= 11);
}

fn assert_cross_account_operational_query_is_rejected(contour: &ManagedWhatsAppContour) {
    let request_id = 48;
    let request = encode_module_request(
        request_id,
        &WhatsAppPublicClientRequestV1::OperationalQuery(
            WhatsAppOperationalQueryV1::GetRuntimeStatus {
                account_id: "another-whatsapp-account".to_owned(),
            },
        ),
    )
    .expect("encode cross-account WhatsApp operational query");
    let route = ManagedCapabilityRouteRequest::new(
        &contour.whatsapp.registration_id,
        &contour.whatsapp.runtime_instance_id,
        contour.whatsapp.runtime_generation,
        contour.whatsapp.grant_epoch,
        WhatsAppClientContractV1::OperationalQuery.capability_id(),
        &request,
    );
    let response = route_managed_client_request(
        contour.store.as_ref(),
        &contour.supervisor.relay_port(),
        &route,
    )
    .expect("route admitted cross-account WhatsApp query to owner runtime");
    let response =
        ModuleClientResponseV1::decode(response.as_slice()).expect("decode WhatsApp module error");
    assert_eq!(response.request_id, request_id);
    assert_eq!(response.error_code, "RUNTIME_UNAVAILABLE");
    assert!(
        response.response_payload.is_empty(),
        "cross-account rejection must not expose provider-owned data",
    );
}

fn assert_cross_account_operational_replay_is_rejected(contour: &ManagedWhatsAppContour) {
    let request_id = 60;
    let request = encode_module_request(
        request_id,
        &WhatsAppPublicClientRequestV1::OperationalReplay(WhatsAppOperationalReplayRequestV1 {
            account_id: "another-whatsapp-account".to_owned(),
            after_sequence: 0,
            limit: 10,
        }),
    )
    .expect("encode cross-account WhatsApp operational replay");
    let route = ManagedCapabilityRouteRequest::new(
        &contour.whatsapp.registration_id,
        &contour.whatsapp.runtime_instance_id,
        contour.whatsapp.runtime_generation,
        contour.whatsapp.grant_epoch,
        WhatsAppClientContractV1::OperationalRealtime.capability_id(),
        &request,
    );
    let response = route_managed_client_request(
        contour.store.as_ref(),
        &contour.supervisor.relay_port(),
        &route,
    )
    .expect("route admitted cross-account WhatsApp replay to owner runtime");
    let response =
        ModuleClientResponseV1::decode(response.as_slice()).expect("decode WhatsApp module error");
    assert_eq!(response.request_id, request_id);
    assert_eq!(response.error_code, "RUNTIME_UNAVAILABLE");
    assert!(response.response_payload.is_empty());
}

fn operational_query(
    contour: &ManagedWhatsAppContour,
    request_id: u64,
    query: WhatsAppOperationalQueryV1,
) -> WhatsAppOperationalQueryResponseV1 {
    match route_whatsapp_client(
        contour,
        WhatsAppClientContractV1::OperationalQuery,
        request_id,
        &WhatsAppPublicClientRequestV1::OperationalQuery(query),
    ) {
        WhatsAppPublicClientResponseV1::OperationalQuery(response) => response,
        response => panic!("unexpected WhatsApp operational response: {response:?}"),
    }
}

fn operational_replay(
    contour: &ManagedWhatsAppContour,
    request_id: u64,
    request: WhatsAppOperationalReplayRequestV1,
) -> WhatsAppOperationalReplayResponseV1 {
    match route_whatsapp_client(
        contour,
        WhatsAppClientContractV1::OperationalRealtime,
        request_id,
        &WhatsAppPublicClientRequestV1::OperationalReplay(request),
    ) {
        WhatsAppPublicClientResponseV1::OperationalReplay(response) => response,
        response => panic!("unexpected WhatsApp operational replay response: {response:?}"),
    }
}

fn assert_command_result_stays_owner_local(
    runtime: &tokio::runtime::Runtime,
    observations: &mut async_nats::Subscriber,
    canonical_events: &mut async_nats::Subscriber,
) {
    runtime.block_on(async {
        assert!(
            tokio::time::timeout(Duration::from_millis(500), observations.next())
                .await
                .is_err(),
            "WhatsApp terminal command result must not become Communications evidence",
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(500), canonical_events.next())
                .await
                .is_err(),
            "Communications must not create canonical truth from a provider command receipt",
        );
    });
}

fn assert_whatsapp_command_accepted(contour: &ManagedWhatsAppContour, operation_id: &str) {
    let response = route_whatsapp_client(
        contour,
        WhatsAppClientContractV1::Command,
        31,
        &WhatsAppPublicClientRequestV1::Command(WhatsAppProviderCommand::SendText {
            operation_id: operation_id.to_owned(),
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            provider_chat_id: "provider-chat-1".to_owned(),
            text: PRIVATE_COMMAND_TEXT.to_owned(),
        }),
    );
    assert!(
        matches!(
            response,
            WhatsAppPublicClientResponseV1::Accepted { operation_id: accepted }
                if accepted == operation_id
        ),
        "WhatsApp command must return only an accepted receipt",
    );
}

fn execute_whatsapp_command(
    contour: &ManagedWhatsAppContour,
    operation_id: &str,
    host_claim_id: &str,
) {
    let mut host = WhatsAppHostBridgeTestClient::connect(&contour.whatsapp);
    let commands = host.claim_commands(WHATSAPP_ACCOUNT_ID, host_claim_id);
    assert!(
        matches!(
            commands.as_slice(),
            [WhatsAppProviderCommand::SendText {
                operation_id: claimed_operation_id,
                account_id,
                provider_chat_id,
                text,
            }] if claimed_operation_id == operation_id
                && account_id == WHATSAPP_ACCOUNT_ID
                && provider_chat_id == "provider-chat-1"
                && text == PRIVATE_COMMAND_TEXT
        ),
        "the native host must lease the exact integration-owned provider command",
    );
    let provider_event_id = "whatsapp-command-result-1";
    assert_eq!(
        host.submit_observation(&WhatsAppHostBridgeEnvelopeV1 {
            protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
            protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
            account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
            provider_event_id: provider_event_id.to_owned(),
            observed_at_unix_seconds: 1_785_000_001,
            observation: WhatsAppHostObservationV1::CommandResult {
                operation_id: operation_id.to_owned(),
                provider_request_id: Some("provider-request-1".to_owned()),
                succeeded: true,
                host_claim_id: host_claim_id.to_owned(),
            },
        }),
        provider_event_id,
    );
}

fn assert_whatsapp_operation_succeeded(contour: &ManagedWhatsAppContour, operation_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_whatsapp_client(
            contour,
            WhatsAppClientContractV1::Query,
            32,
            &WhatsAppPublicClientRequestV1::OperationStatus {
                operation_id: operation_id.to_owned(),
            },
        );
        if matches!(
            response,
            WhatsAppPublicClientResponseV1::OperationStatus(Some(status))
                if status.operation_id == operation_id
                    && status.account_id == WHATSAPP_ACCOUNT_ID
                    && status.state == WhatsAppProviderCommandStateV1::Succeeded
                    && status.completed_at_unix_seconds.is_some()
        ) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "WhatsApp provider command did not reach terminal success",
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn submit_message_observation(
    contour: &ManagedWhatsAppContour,
    provider_event_id: &str,
    provider_message_id: &str,
) -> Vec<u8> {
    let envelope = WhatsAppHostBridgeEnvelopeV1 {
        protocol_major: HOST_BRIDGE_PROTOCOL_MAJOR,
        protocol_revision: HOST_BRIDGE_PROTOCOL_REVISION,
        account_id: WHATSAPP_ACCOUNT_ID.to_owned(),
        provider_event_id: provider_event_id.to_owned(),
        observed_at_unix_seconds: 1_785_000_002,
        observation: WhatsAppHostObservationV1::MessageIdentity {
            provider_chat_id: "provider-chat-1".to_owned(),
            provider_message_id: provider_message_id.to_owned(),
            sender_id: "provider-sender-1".to_owned(),
        },
    };
    let private_host_payload =
        makosh_whatsapp_api::host_bridge::encode_host_bridge_payload(&envelope)
            .expect("encode private WhatsApp host operation");
    let mut host = WhatsAppHostBridgeTestClient::connect(&contour.whatsapp);
    assert_eq!(host.submit_observation(&envelope), provider_event_id);
    private_host_payload
}

fn route_whatsapp_client(
    contour: &ManagedWhatsAppContour,
    contract: WhatsAppClientContractV1,
    request_id: u64,
    request: &WhatsAppPublicClientRequestV1,
) -> WhatsAppPublicClientResponseV1 {
    let encoded =
        encode_module_request(request_id, request).expect("encode WhatsApp client request");
    let relay = contour.supervisor.relay_port();
    let deadline = Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&contour.whatsapp.registration_id) != Ok(true) {
        assert!(
            Instant::now() < deadline,
            "managed WhatsApp runtime did not become ready: {:?}",
            contour
                .supervisor
                .last_failure(&contour.whatsapp.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &contour.whatsapp.registration_id,
            &contour.whatsapp.runtime_instance_id,
            contour.whatsapp.runtime_generation,
            contour.whatsapp.grant_epoch,
            contract.capability_id(),
            &encoded,
        );
        let last_error = match route_managed_client_request(contour.store.as_ref(), &relay, &route)
        {
            Ok(bytes) => match decode_module_response(contract, &bytes) {
                Ok((response_id, response)) if response_id == request_id => return response,
                outcome => format!("unexpected WhatsApp response: {outcome:?}"),
            },
            Err(error) => error,
        };
        assert!(
            Instant::now() < deadline,
            "WhatsApp client route {contract:?} request {request_id} remained unavailable: {last_error}; managed failure: {:?}",
            contour
                .supervisor
                .last_failure(&contour.whatsapp.registration_id),
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn receive_whatsapp_observation(
    runtime: &tokio::runtime::Runtime,
    observations: &mut async_nats::Subscriber,
    canonical_events: &mut async_nats::Subscriber,
    contour: &ManagedWhatsAppContour,
    phase: &str,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (observation, canonical) = runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} WhatsApp observation timeout"))
            .unwrap_or_else(|| panic!("{phase} WhatsApp observation"));
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} Communications event timeout"))
            .unwrap_or_else(|| panic!("{phase} Communications event"));
        (observation, canonical)
    });
    let observation_bytes = observation.payload.to_vec();
    assert_private_host_route_absent(&observation_bytes, &contour.whatsapp);
    let observation =
        decode_envelope_v1(&observation_bytes).expect("WhatsApp observation durable envelope");
    let source = observation
        .source
        .as_ref()
        .expect("WhatsApp observation source");
    assert_eq!(source.module_id, makosh_whatsapp_runtime::PACKAGE);
    assert_eq!(
        source.runtime_generation,
        contour.whatsapp.runtime_generation,
    );
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications event envelope");
    assert_eq!(
        canonical.causation_message_id, observation.message_id,
        "Communications must derive canonical evidence only from the typed WhatsApp observation",
    );
    (
        observation_bytes,
        observation.message_id,
        canonical.message_id,
    )
}

fn assert_private_host_route_absent(bytes: &[u8], runtime: &StartedWhatsAppRuntime) {
    let socket_path = runtime.host_bridge_socket_path.to_string_lossy();
    assert!(
        !bytes
            .windows(socket_path.len())
            .any(|window| window == socket_path.as_bytes()),
        "private WhatsApp host socket path must not enter the durable event",
    );
    assert!(
        !bytes
            .windows(runtime.route_binding_sha256.len())
            .any(|window| window == runtime.route_binding_sha256.as_slice()),
        "private WhatsApp host route binding must not enter the durable event",
    );
}

fn assert_private_command_text_absent(bytes: &[u8]) {
    assert!(
        !bytes
            .windows(PRIVATE_COMMAND_TEXT.len())
            .any(|window| window == PRIVATE_COMMAND_TEXT.as_bytes()),
        "provider command body must not enter the durable Communications envelope",
    );
}

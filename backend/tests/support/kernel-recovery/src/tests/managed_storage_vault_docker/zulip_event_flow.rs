//! Live Zulip command and typed event-only Communications handoff conformance.

use std::time::Instant;

use super::*;

use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use makosh_zulip_api::{
    ZulipClientRequestV1, ZulipClientResponseV1, ZulipCommandOperationOutcomeV1, ZulipCommandV1,
    client_contract::{ZULIP_MODULE_ID, ZulipClientContractV1},
    operational::{
        ZulipHistoryStateV1, ZulipOperationalEventKindV1, ZulipOperationalQueryResponseV1,
        ZulipOperationalQueryV1,
    },
    realtime::ZulipOperationalReplayRequestV1,
};
use makosh_zulip_runtime::client_port::{
    ZulipClientPortErrorV1, decode_module_response, encode_module_request,
};

use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

const OBSERVATION_SUBJECT: &str = "makosh.observation.v1.communications.communication_observed.v1";
const CANONICAL_EVENT_SUBJECT: &str =
    "makosh.event.v1.communications.communication_evidence_recorded.v1";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Zulip binaries"]
fn managed_zulip_runtime_delivers_live_command_and_event_only_communications_handoff() {
    let mut contour = ManagedZulipContour::start(ZulipGrantProfileV1::CommandAndQuery);
    let events = contour
        .store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology");
    let event_runtime = tokio::runtime::Runtime::new().expect("Zulip event observer runtime");
    let _event_runtime_context = event_runtime.enter();
    let (client, mut observations, mut canonical_events) = event_runtime.block_on(async {
        let client = async_nats::connect(events.nats_endpoint())
            .await
            .expect("connect Zulip event observer");
        let observations = client
            .subscribe(OBSERVATION_SUBJECT)
            .await
            .expect("subscribe Zulip observations");
        let canonical_events = client
            .subscribe(CANONICAL_EVENT_SUBJECT)
            .await
            .expect("subscribe canonical Communications events");
        client
            .flush()
            .await
            .expect("activate Zulip event observers");
        (client, observations, canonical_events)
    });

    const OPERATION_ID: &str = "managed-zulip-live-command-1";
    assert_zulip_history_and_operational_query(&contour);
    assert_cross_account_operational_query_is_rejected(&contour);
    assert_zulip_command_accepted(&contour, OPERATION_ID);
    assert_zulip_operation_completed(&contour, OPERATION_ID);
    assert_eq!(
        contour.fixture.message_commands(),
        1,
        "the accepted command must execute exactly once against the live provider"
    );

    assert_eq!(contour.fixture.release_next_event(), 1);
    let (observation_bytes, observation_message_id, canonical_message_id) =
        receive_zulip_observation(
            &event_runtime,
            &mut observations,
            &mut canonical_events,
            &contour,
            "initial",
        );
    assert_zulip_operational_replay(&contour, 9_101);
    let predecessor_generation = contour.zulip.runtime_generation;
    contour.zulip = restart_zulip_runtime(
        &contour.supervisor,
        contour.store.as_ref(),
        &contour.data,
        &contour.root.join("runtime"),
        &contour.zulip,
        contour.fixture.realm_url(),
        None,
    );
    assert_eq!(contour.zulip.runtime_generation, predecessor_generation + 1);
    assert_zulip_history_and_operational_query(&contour);
    assert_zulip_operational_replay(&contour, 9_101);
    event_runtime.block_on(async {
        client
            .publish(OBSERVATION_SUBJECT, observation_bytes.into())
            .await
            .expect("republish exact Zulip observation");
        client
            .flush()
            .await
            .expect("flush duplicate Zulip observation");
        let duplicate = tokio::time::timeout(Duration::from_secs(1), observations.next())
            .await
            .expect("duplicate Zulip observation timeout")
            .expect("duplicate Zulip observation");
        let duplicate = decode_envelope_v1(duplicate.payload.as_ref())
            .expect("duplicate Zulip observation envelope");
        assert_eq!(duplicate.message_id, observation_message_id);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate Zulip observation must not create a second Communications event"
        );
    });
    let initial_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);

    set_authenticated_nats_container_running(false);
    assert_eq!(contour.fixture.release_next_event(), 2);
    wait_for_served_event(&contour, 2);
    std::thread::sleep(Duration::from_millis(2_500));
    assert!(
        contour
            .supervisor
            .is_active(&contour.zulip.registration_id)
            .expect("read managed Zulip state"),
        "managed Zulip runtime must remain active while NATS is unavailable"
    );
    assert_eq!(
        contour
            .supervisor
            .last_failure(&contour.zulip.registration_id)
            .expect("read managed Zulip failure"),
        None
    );
    set_authenticated_nats_container_running(true);
    wait_for_authenticated_nats_reconnect(&event_runtime, &client, "Zulip event observer");

    let (_, replayed_observation_id, replayed_canonical_id) = receive_zulip_observation(
        &event_runtime,
        &mut observations,
        &mut canonical_events,
        &contour,
        "outage replay",
    );
    assert_ne!(replayed_observation_id, observation_message_id);
    assert_ne!(
        replayed_canonical_id, canonical_message_id,
        "outage replay must deliver the second provider observation"
    );
    let replayed_evidence_id =
        assert_communications_query_delivery(&contour.store, &contour.supervisor);
    assert_ne!(
        replayed_evidence_id, initial_evidence_id,
        "Communications query must expose the replayed Zulip evidence"
    );

    contour.shutdown_processes();
    contour.finish();
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, NATS, Communications and Zulip binaries"]
fn managed_zulip_private_surfaces_reject_malformed_provider_output() {
    let contour = ManagedZulipContour::start(ZulipGrantProfileV1::CommandAndQuery);
    let storage_credential = runtime_storage_credential_for_registration_v1(
        &contour.supervisor,
        &contour.store,
        &contour.data,
        &contour.zulip.registration_id,
        makosh_zulip_runtime::admission::ZULIP_STORAGE_CAPABILITY_ID,
    );
    assert_eq!(contour.fixture.release_malformed_event(), 1);
    let deadline = Instant::now() + Duration::from_secs(5);
    while contour.fixture.served_malformed_events() < 1 {
        assert!(
            Instant::now() < deadline,
            "managed Zulip runtime did not consume malformed provider output"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(250));
    assert!(
        contour
            .supervisor
            .is_active(&contour.zulip.registration_id)
            .expect("read Zulip privacy runtime state"),
        "malformed provider output must not terminate the Zulip runtime"
    );
    assert_eq!(
        contour
            .supervisor
            .relay_port()
            .is_ready(&contour.zulip.registration_id),
        Ok(true),
        "malformed provider output must not revoke readiness"
    );

    let public_response = route_zulip_client(
        &contour,
        ZulipClientContractV1::OperationalQuery,
        72,
        &ZulipClientRequestV1::OperationalQuery(ZulipOperationalQueryV1::GetAccountStatus {
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
        }),
    );
    let diagnostic = format!(
        "{:?}",
        contour
            .supervisor
            .last_failure(&contour.zulip.registration_id)
            .expect("read Zulip supervisor diagnostic")
    );
    assert_zulip_private_values_absent_v1(
        format!("{public_response:?}").as_bytes(),
        storage_credential.as_slice(),
        "typed Zulip client response",
    );
    assert_zulip_private_values_absent_v1(
        diagnostic.as_bytes(),
        storage_credential.as_slice(),
        "Zulip supervisor diagnostic",
    );
    assert_zulip_public_durable_surfaces_are_private_v1(storage_credential.as_slice());
    assert_supervised_zulip_child_output_is_private_v1(
        &contour.child_stdio_capture,
        storage_credential.as_slice(),
    );

    contour.shutdown_processes();
    contour.finish();
}

fn assert_zulip_private_values_absent_v1(bytes: &[u8], storage_credential: &[u8], surface: &str) {
    for private_value in [
        PRIVATE_ZULIP_MESSAGE_MARKER.as_bytes(),
        PRIVATE_ZULIP_RAW_PROVIDER_MARKER.as_bytes(),
        PRIVATE_ZULIP_LOCATOR_MARKER.as_bytes(),
        PRIVATE_ZULIP_QUEUE_MARKER.as_bytes(),
        b"managed-zulip-api-key".as_slice(),
        storage_credential,
    ] {
        assert!(!private_value.is_empty());
        assert!(
            !bytes
                .windows(private_value.len())
                .any(|window| window == private_value),
            "{surface} exposed a private Zulip value"
        );
        let encoded = private_value
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert!(
            !bytes
                .windows(encoded.len())
                .any(|window| window == encoded.as_bytes()),
            "{surface} exposed a hex-encoded private Zulip value"
        );
    }
}

fn assert_zulip_public_durable_surfaces_are_private_v1(storage_credential: &[u8]) {
    tokio::runtime::Runtime::new()
        .expect("Zulip privacy database runtime")
        .block_on(async {
            let pool = authenticated_storage_admin_pool_v1().await;
            for table in [
                "zulip_communications_outbox",
                "zulip_delivery_intent_result_outbox",
            ] {
                let rows: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                    "SELECT COALESCE(string_agg(row_to_json(source)::text, E'\\n'), '') \
                     FROM makosh_data.{table} AS source"
                )))
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|error| panic!("serialize public Zulip table {table}: {error}"));
                assert_zulip_private_values_absent_v1(
                    rows.as_bytes(),
                    storage_credential,
                    "durable public Zulip row",
                );
            }
        });
}

fn assert_supervised_zulip_child_output_is_private_v1(directory: &Path, storage_credential: &[u8]) {
    // The production supervisor keeps stdout/stderr null; this conformance-only
    // sink reads the exact supervised files of the active Zulip successor.
    let mut captures = std::fs::read_dir(directory)
        .expect("read Zulip child capture directory")
        .map(|entry| entry.expect("read Zulip child capture entry").path())
        .collect::<Vec<_>>();
    captures.sort();
    assert_eq!(captures.len(), 2, "one supervised Zulip child attempt");
    for capture in captures {
        let bytes = std::fs::read(capture).expect("read supervised Zulip child output");
        assert_zulip_private_values_absent_v1(
            &bytes,
            storage_credential,
            "supervised Zulip stdout/stderr",
        );
    }
}

fn assert_zulip_history_and_operational_query(contour: &ManagedZulipContour) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_zulip_client(
            contour,
            ZulipClientContractV1::OperationalQuery,
            21,
            &ZulipClientRequestV1::OperationalQuery(ZulipOperationalQueryV1::GetAccountStatus {
                account_id: ZULIP_ACCOUNT_ID.to_owned(),
            }),
        );
        let ZulipClientResponseV1::OperationalQuery(
            ZulipOperationalQueryResponseV1::AccountStatus(status),
        ) = response
        else {
            panic!("Zulip account status returned the wrong response")
        };
        if status.history_state == ZulipHistoryStateV1::Ready {
            assert!(status.projection_ready);
            assert_eq!(status.oldest_provider_message_id.as_deref(), Some("9001"));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Zulip history did not converge: {status:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(
        contour.fixture.history_pages() >= 2,
        "managed Zulip history must cross a real bounded multi-page provider contour"
    );

    let response = route_zulip_client(
        contour,
        ZulipClientContractV1::OperationalQuery,
        22,
        &ZulipClientRequestV1::OperationalQuery(ZulipOperationalQueryV1::SearchMessages {
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
            provider_conversation_id: Some("stream:44:history".to_owned()),
            query: "searchable".to_owned(),
            cursor: None,
            limit: 20,
        }),
    );
    let ZulipClientResponseV1::OperationalQuery(ZulipOperationalQueryResponseV1::Messages(page)) =
        response
    else {
        panic!("Zulip message search returned the wrong response")
    };
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].provider_message_id, "9002");
    assert_eq!(page.items[0].reactions.len(), 1);

    let response = route_zulip_client(
        contour,
        ZulipClientContractV1::OperationalQuery,
        23,
        &ZulipClientRequestV1::OperationalQuery(ZulipOperationalQueryV1::ListConversations {
            account_id: ZULIP_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 20,
        }),
    );
    let ZulipClientResponseV1::OperationalQuery(ZulipOperationalQueryResponseV1::Conversations(
        page,
    )) = response
    else {
        panic!("Zulip conversations returned the wrong response")
    };
    assert!(
        page.items.len() >= 2,
        "history conversations remain present alongside later event-driven conversations"
    );
    assert!(
        page.items
            .iter()
            .any(|conversation| conversation.provider_conversation_id == "stream:44:history")
    );
    assert!(
        page.items
            .iter()
            .any(|conversation| conversation.provider_conversation_id == "direct:55")
    );
}

fn assert_cross_account_operational_query_is_rejected(contour: &ManagedZulipContour) {
    let request =
        ZulipClientRequestV1::OperationalQuery(ZulipOperationalQueryV1::GetAccountStatus {
            account_id: "other-account".to_owned(),
        });
    let encoded = encode_module_request(24, &request).expect("encode cross-account query");
    let route = ManagedCapabilityRouteRequest::new(
        &contour.zulip.registration_id,
        &contour.zulip.runtime_instance_id,
        contour.zulip.runtime_generation,
        contour.zulip.grant_epoch,
        ZulipClientContractV1::OperationalQuery.capability_id(),
        &encoded,
    );
    let bytes = route_managed_client_request(
        contour.store.as_ref(),
        &contour.supervisor.relay_port(),
        &route,
    )
    .expect("route rejected Zulip query response");
    assert_eq!(
        decode_module_response(ZulipClientContractV1::OperationalQuery, &bytes),
        Err(ZulipClientPortErrorV1::Protocol)
    );
    assert!(
        contour
            .supervisor
            .is_active(&contour.zulip.registration_id)
            .expect("observe Zulip after rejected cross-account query"),
        "cross-account client payload must not terminate the managed runtime"
    );
}

fn assert_zulip_operational_replay(contour: &ManagedZulipContour, provider_message_id: i64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_zulip_client(
            contour,
            ZulipClientContractV1::OperationalRealtime,
            25,
            &ZulipClientRequestV1::OperationalReplay(ZulipOperationalReplayRequestV1 {
                account_id: ZULIP_ACCOUNT_ID.to_owned(),
                after_sequence: 0,
                limit: 20,
            }),
        );
        let ZulipClientResponseV1::OperationalReplay(response) = response else {
            panic!("Zulip operational replay returned the wrong response")
        };
        if let Some(frame) = response.frames.last() {
            assert_eq!(
                frame.event.provider_message_id,
                provider_message_id.to_string()
            );
            assert_eq!(
                frame.event.kind,
                ZulipOperationalEventKindV1::MessageUpserted
            );
            assert!(!response.reset_required);
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Zulip operational replay did not expose the provider event"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_zulip_command_accepted(contour: &ManagedZulipContour, operation_id: &str) {
    let request = ZulipClientRequestV1::Command(ZulipCommandV1::SendStream {
        operation_id: operation_id.to_owned(),
        account_id: ZULIP_ACCOUNT_ID.to_owned(),
        stream: "operations".to_owned(),
        topic: "managed".to_owned(),
        content: "managed Zulip provider command".to_owned(),
    });
    let response = route_zulip_client(contour, ZulipClientContractV1::Command, 31, &request);
    let ZulipClientResponseV1::CommandReceipt(receipt) = response else {
        panic!("Zulip command returned the wrong response type");
    };
    assert_eq!(receipt.operation_id, operation_id);
    assert_eq!(receipt.account_id, ZULIP_ACCOUNT_ID);
}

fn assert_zulip_operation_completed(contour: &ManagedZulipContour, operation_id: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let response = route_zulip_client(
            contour,
            ZulipClientContractV1::Query,
            32,
            &ZulipClientRequestV1::OperationStatus {
                operation_id: operation_id.to_owned(),
            },
        );
        let ZulipClientResponseV1::OperationStatus(status) = response else {
            panic!("Zulip operation query returned the wrong response type");
        };
        if let Some(status) = status {
            assert_eq!(status.operation_id, operation_id);
            assert_eq!(status.account_id, ZULIP_ACCOUNT_ID);
            match status.outcome {
                ZulipCommandOperationOutcomeV1::Accepted {
                    provider_message_id: Some(4242),
                    blob_ref: None,
                } => return,
                ZulipCommandOperationOutcomeV1::Rejected => {
                    panic!("Zulip provider command was rejected")
                }
                _ => {}
            }
        }
        assert!(
            Instant::now() < deadline,
            "Zulip provider command did not reach a terminal result"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn route_zulip_client(
    contour: &ManagedZulipContour,
    contract: ZulipClientContractV1,
    request_id: u64,
    request: &ZulipClientRequestV1,
) -> ZulipClientResponseV1 {
    let encoded = encode_module_request(request_id, request).expect("encode Zulip client request");
    let relay = contour.supervisor.relay_port();
    let deadline = Instant::now() + Duration::from_secs(10);
    while relay.is_ready(&contour.zulip.registration_id) != Ok(true) {
        assert!(
            Instant::now() < deadline,
            "managed Zulip runtime did not become ready: {:?}",
            contour
                .supervisor
                .last_failure(&contour.zulip.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    loop {
        let route = ManagedCapabilityRouteRequest::new(
            &contour.zulip.registration_id,
            &contour.zulip.runtime_instance_id,
            contour.zulip.runtime_generation,
            contour.zulip.grant_epoch,
            contract.capability_id(),
            &encoded,
        );
        let last_error = match route_managed_client_request(contour.store.as_ref(), &relay, &route)
        {
            Ok(bytes) => match decode_module_response(contract, &bytes) {
                Ok((response_id, response)) if response_id == request_id => return response,
                Ok((response_id, _)) => format!("unexpected response id {response_id}"),
                Err(ZulipClientPortErrorV1::Protocol) => "invalid Zulip route response".to_owned(),
                Err(ZulipClientPortErrorV1::Runtime) => "Zulip route runtime error".to_owned(),
            },
            Err(error) => error,
        };
        assert!(
            Instant::now() < deadline,
            "Zulip client route remained unavailable: {last_error}; managed failure: {:?}",
            contour
                .supervisor
                .last_failure(&contour.zulip.registration_id)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn receive_zulip_observation(
    runtime: &tokio::runtime::Runtime,
    observations: &mut async_nats::Subscriber,
    canonical_events: &mut async_nats::Subscriber,
    contour: &ManagedZulipContour,
    phase: &str,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let (observation, canonical) = runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} Zulip observation timeout"))
            .unwrap_or_else(|| panic!("{phase} Zulip observation"));
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .unwrap_or_else(|_| panic!("{phase} Communications event timeout"))
            .unwrap_or_else(|| panic!("{phase} Communications event"));
        (observation, canonical)
    });
    let observation_bytes = observation.payload.to_vec();
    let observation =
        decode_envelope_v1(&observation_bytes).expect("Zulip observation durable envelope");
    let source = observation
        .source
        .as_ref()
        .expect("Zulip observation source");
    assert_eq!(source.module_id, ZULIP_MODULE_ID);
    assert_eq!(source.runtime_generation, contour.zulip.runtime_generation);
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications event envelope");
    assert_eq!(
        canonical.causation_message_id, observation.message_id,
        "Communications must derive canonical evidence only from the typed Zulip observation"
    );
    (
        observation_bytes,
        observation.message_id,
        canonical.message_id,
    )
}

fn wait_for_served_event(contour: &ManagedZulipContour, expected: u64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while contour.fixture.served_events() < expected {
        assert!(
            Instant::now() < deadline,
            "managed Zulip runtime did not poll released provider event {expected}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

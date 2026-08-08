//! Live provider → Mail → Blob → Communications attachment lifecycle conformance.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use makosh_attachment_security_contract::v1::AttachmentSecurityScanCandidateObservedV1;
use makosh_communications_api::query_wire::{
    CommunicationsQueryRequestV1, ListAccountsRequestV1, ListConversationMessagesRequestV1,
    ListConversationsRequestV1, ListMessageAttachmentAnchorsRequestV1,
    communications_query_request_v1::Operation,
    communications_query_response_v1::Result as QueryResult,
};
use makosh_communications_attachment_contract::{
    AttachmentBlobAdmissionFactV1, AttachmentBlobAdmissionTransitionV1,
    AttachmentBlobExpectedStateV1, AttachmentObservationEnvelopeContextV1,
    anchor_recorded_v1::AttachmentAnchorRecordedV1,
    blob_admission_v1::{
        AttachmentBlobAdmissionObservationV1,
        AttachmentBlobAdmissionTransitionV1 as AttachmentBlobAdmissionTransitionWireV1,
    },
    build_attachment_blob_admission_outbox_record_v1,
    lifecycle_v1::{
        AttachmentSafetyStateChangedV1, AttachmentSafetyStateV1 as AttachmentSafetyStateWireV1,
    },
};
use makosh_communications_ingress::v1::CommunicationObservationV1;
use makosh_events_protocol::validation::envelope::decode_envelope_v1;
use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSyncInboxRequestV1,
    client_contract::MailClientContractV1,
};
use makosh_mail_runtime::{
    admission::MAIL_MODULE_ID,
    client_port::{decode_module_response, encode_module_request},
};
use prost::Message;

use super::*;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

pub(super) fn assert_mail_attachment_lifecycle(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) -> [u8; 16] {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let runtime = tokio::runtime::Runtime::new().expect("Mail attachment observer runtime");
    let _runtime_context = runtime.enter();
    let durable = runtime.block_on(connect_postgres());
    let (client, mut observations, mut anchors, mut admissions, mut candidates, mut state_changes) =
        runtime.block_on(async {
            let client = async_nats::connect(endpoint)
                .await
                .expect("connect Mail attachment observer");
            let observations = client
                .subscribe("makosh.observation.v1.communications.communication_observed.v1")
                .await
                .expect("subscribe Mail attachment observations");
            let anchors = client
                .subscribe(
                    "makosh.event.v1.communications.communication_attachment_anchor_recorded.v1",
                )
                .await
                .expect("subscribe attachment anchors");
            let admissions = client
                .subscribe(
                    "makosh.observation.v1.communications.\
                 communication_attachment_blob_admission_observed.v1",
                )
                .await
                .expect("subscribe Mail Blob admissions");
            let candidates = client
                .subscribe(
                    "makosh.observation.v1.attachment_security.\
                 attachment_security_scan_candidate_observed.v1",
                )
                .await
                .expect("subscribe Attachment Security candidates");
            let state_changes = client
                .subscribe(
                    "makosh.event.v1.communications.\
                 communication_attachment_safety_state_changed.v1",
                )
                .await
                .expect("subscribe attachment state changes");
            client
                .flush()
                .await
                .expect("activate Mail attachment observers");
            (
                client,
                observations,
                anchors,
                admissions,
                candidates,
                state_changes,
            )
        });

    assert_mail_sync(
        store,
        supervisor,
        mail,
        31,
        "managed-mail-attachment-source",
    );
    let attachment_observation = runtime
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    let observation = observations
                        .next()
                        .await
                        .expect("Mail attachment observation");
                    let envelope = decode_envelope_v1(observation.payload.as_ref())
                        .expect("Mail observation durable envelope");
                    let payload = CommunicationObservationV1::decode(envelope.payload.as_slice())
                        .expect("Mail observation payload");
                    if payload.attachment_descriptor.is_some() {
                        return envelope;
                    }
                }
            })
            .await
        })
        .expect("Mail attachment observation timeout");
    let anchor = runtime
        .block_on(async { tokio::time::timeout(Duration::from_secs(10), anchors.next()).await })
        .expect("Mail attachment anchor timeout")
        .expect("Mail attachment anchor");
    let anchor_envelope =
        decode_envelope_v1(anchor.payload.as_ref()).expect("attachment anchor envelope");
    let anchor_payload = AttachmentAnchorRecordedV1::decode(anchor_envelope.payload.as_slice())
        .expect("attachment anchor payload");
    assert_eq!(
        anchor_envelope
            .source
            .as_ref()
            .expect("Communications anchor source")
            .module_id,
        COMMUNICATIONS_REGISTRATION
    );
    let source_observation_id: [u8; 16] = anchor_payload
        .source_observation_id
        .as_slice()
        .try_into()
        .expect("Mail source observation id");
    assert_eq!(attachment_observation.message_id, source_observation_id);
    let attachment_anchor_id: [u8; 16] = anchor_payload
        .attachment_anchor_id
        .as_slice()
        .try_into()
        .expect("Communications attachment anchor id");
    let mapping = wait_for_mail_mapping(&runtime, &durable, source_observation_id);
    assert_eq!(mapping.attachment_anchor_id, attachment_anchor_id);
    assert_eq!(
        mapping.correlation_id.as_slice(),
        anchor_envelope.correlation_id
    );
    assert_eq!(
        mapping.media_cursor_sha256.as_slice(),
        anchor_payload.media_cursor_sha256
    );

    assert_mail_sync(store, supervisor, mail, 32, "managed-mail-attachment-admit");
    let mut admission_records = Vec::new();
    for expected_transition in [
        AttachmentBlobAdmissionTransitionWireV1::Requested as i32,
        AttachmentBlobAdmissionTransitionWireV1::Admitted as i32,
    ] {
        let message = runtime
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(10), admissions.next()).await
            })
            .expect("Mail Blob admission timeout")
            .expect("Mail Blob admission observation");
        let exact_bytes = message.payload.to_vec();
        let envelope =
            decode_envelope_v1(&exact_bytes).expect("Mail Blob admission durable envelope");
        let payload = AttachmentBlobAdmissionObservationV1::decode(envelope.payload.as_slice())
            .expect("Mail Blob admission payload");
        assert_eq!(payload.transition, expected_transition);
        assert_eq!(payload.attachment_anchor_id, attachment_anchor_id);
        assert_eq!(payload.evidence_id, source_observation_id);
        assert_eq!(envelope.causation_message_id, source_observation_id);
        assert_eq!(envelope.correlation_id, mapping.correlation_id);
        let source = envelope
            .source
            .as_ref()
            .expect("Mail Blob admission source");
        assert_eq!(source.module_id, MAIL_MODULE_ID);
        assert_eq!(source.runtime_generation, mail.runtime_generation);
        if expected_transition == AttachmentBlobAdmissionTransitionWireV1::Requested as i32 {
            assert!(payload.blob_reference_binding_sha256.is_empty());
        } else {
            assert_eq!(payload.blob_reference_binding_sha256.len(), 32);
            assert!(
                payload
                    .blob_reference_binding_sha256
                    .iter()
                    .any(|byte| *byte != 0)
            );
        }
        admission_records.push((envelope.message_id.clone(), exact_bytes));
    }
    for (index, (expected_state, next_state)) in [
        (
            AttachmentSafetyStateWireV1::DescriptorOnly as i32,
            AttachmentSafetyStateWireV1::BlobPending as i32,
        ),
        (
            AttachmentSafetyStateWireV1::BlobPending as i32,
            AttachmentSafetyStateWireV1::BlobAdmitted as i32,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let message = runtime
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(10), state_changes.next()).await
            })
            .expect("Communications attachment state timeout")
            .expect("Communications attachment state event");
        let envelope =
            decode_envelope_v1(message.payload.as_ref()).expect("attachment state envelope");
        let payload = AttachmentSafetyStateChangedV1::decode(envelope.payload.as_slice())
            .expect("attachment state payload");
        assert_eq!(payload.attachment_anchor_id, attachment_anchor_id);
        assert_eq!(payload.expected_state, expected_state);
        assert_eq!(payload.next_state, next_state);
        assert_eq!(envelope.causation_message_id, admission_records[index].0);
        assert_eq!(envelope.correlation_id, mapping.correlation_id);
    }
    let candidate = runtime
        .block_on(async { tokio::time::timeout(Duration::from_secs(10), candidates.next()).await })
        .expect("Attachment Security candidate timeout")
        .expect("Attachment Security candidate");
    let candidate_exact_bytes = candidate.payload.to_vec();
    let candidate_envelope = decode_envelope_v1(&candidate_exact_bytes)
        .expect("Attachment Security candidate durable envelope");
    let candidate_payload =
        AttachmentSecurityScanCandidateObservedV1::decode(candidate_envelope.payload.as_slice())
            .expect("Attachment Security candidate payload");
    assert_eq!(candidate_payload.attachment_anchor_id, attachment_anchor_id);
    assert_eq!(candidate_payload.blob_reference_id.len(), 16);
    assert_eq!(candidate_payload.blob_receipt_sha256.len(), 32);
    assert!((1..=2_048).contains(&candidate_payload.custody_transfer_source_proof.len()));
    assert!(candidate_payload.declared_size > 0);
    assert_eq!(
        candidate_envelope.causation_message_id,
        source_observation_id
    );
    assert_eq!(candidate_envelope.correlation_id, mapping.correlation_id);
    assert_eq!(
        candidate_envelope
            .source
            .as_ref()
            .expect("Mail candidate source")
            .module_id,
        MAIL_MODULE_ID
    );
    for forbidden in [
        b"attachment.txt".as_slice(),
        b"application/octet-stream".as_slice(),
        b"managed-mail-imap-password".as_slice(),
    ] {
        assert!(
            !candidate_exact_bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "scan candidate must not expose provider or credential data"
        );
    }
    assert_eq!(
        wait_for_attachment_state(store, supervisor, attachment_anchor_id),
        AttachmentSafetyStateWireV1::BlobAdmitted as u32,
        "Communications must expose the Blob-admitted owner state"
    );

    assert_mail_sync(
        store,
        supervisor,
        mail,
        33,
        "managed-mail-attachment-replay",
    );
    assert!(
        runtime
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(1), admissions.next()).await
            })
            .is_err(),
        "provider replay must not start a second Mail Blob admission"
    );
    assert!(
        runtime
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(1), candidates.next()).await
            })
            .is_err(),
        "provider replay must not create a second Attachment Security candidate"
    );

    publish_exact(
        &runtime,
        &client,
        "makosh.observation.v1.communications.\
         communication_attachment_blob_admission_observed.v1",
        &admission_records[1].1,
    );
    assert!(
        runtime
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(1), state_changes.next()).await
            })
            .is_err(),
        "exact terminal replay must not create another Communications state event"
    );

    let conflict = build_attachment_blob_admission_outbox_record_v1(
        &AttachmentBlobAdmissionFactV1 {
            attachment_anchor_id,
            source_observation_id: [0xAC; 16],
            correlation_id: mapping.correlation_id,
            media_cursor_sha256: mapping.media_cursor_sha256,
            expected_state: AttachmentBlobExpectedStateV1::DescriptorOnly,
            transition: AttachmentBlobAdmissionTransitionV1::Requested,
            observed_at_unix_seconds: current_unix_seconds(),
            blob_reference_binding_sha256: None,
        },
        &AttachmentObservationEnvelopeContextV1 {
            runtime_instance_id: mail.runtime_instance_id.clone(),
            runtime_generation: mail.runtime_generation,
            module_id: MAIL_MODULE_ID.to_owned(),
            recorded_at_unix_seconds: current_unix_seconds(),
            recorded_at_nanos: 0,
        },
    )
    .expect("build conflicting Mail attachment transition");
    publish_exact(
        &runtime,
        &client,
        "makosh.observation.v1.communications.\
         communication_attachment_blob_admission_observed.v1",
        conflict.exact_bytes(),
    );
    assert!(
        runtime
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(1), state_changes.next()).await
            })
            .is_err(),
        "stale expected state must not create a Communications state event"
    );
    assert_eq!(
        wait_for_attachment_state(store, supervisor, attachment_anchor_id),
        AttachmentSafetyStateWireV1::BlobAdmitted as u32,
        "CAS conflict must leave the terminal Communications state unchanged"
    );
    attachment_anchor_id
}

fn assert_mail_sync(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    operation_id: &str,
) {
    let request = encode_module_request(
        request_id,
        &MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: operation_id.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("encode exact Mail sync request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        MailClientContractV1::Sync.capability_id(),
        &request,
    );
    let response = route_managed_client_request(store, &supervisor.relay_port(), &route)
        .expect("route exact Mail sync request");
    let (response_id, response) = decode_module_response(MailClientContractV1::Sync, &response)
        .expect("decode exact Mail sync response");
    assert_eq!(response_id, request_id);
    assert_eq!(
        response,
        MailClientResponseV1::SyncInboxAccepted {
            operation_id: operation_id.to_owned(),
        }
    );
}

fn wait_for_mail_mapping(
    runtime: &tokio::runtime::Runtime,
    durable: &makosh_mail_persistence::MailDurablePersistence,
    source_observation_id: [u8; 16],
) -> makosh_mail_persistence::MailAttachmentAnchorMappingV1 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(mapping) = runtime
            .block_on(durable.attachment_anchor_mapping(source_observation_id))
            .expect("read Mail attachment mapping")
        {
            return mapping;
        }
        assert!(
            Instant::now() < deadline,
            "Mail did not consume the attachment anchor handoff"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(super) fn wait_for_attachment_state(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    attachment_anchor_id: [u8; 16],
) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let accounts = route_communications_query(
            store,
            supervisor,
            61,
            &CommunicationsQueryRequestV1 {
                protocol_major: 1,
                operation: Some(Operation::ListAccounts(ListAccountsRequestV1 {
                    limit: 16,
                    cursor: Vec::new(),
                })),
            }
            .encode_to_vec(),
        );
        let Some(QueryResult::ListAccounts(accounts)) = accounts.result else {
            panic!("Communications accounts query result");
        };
        for account in accounts.accounts {
            let conversations = route_communications_query(
                store,
                supervisor,
                62,
                &CommunicationsQueryRequestV1 {
                    protocol_major: 1,
                    operation: Some(Operation::ListConversations(ListConversationsRequestV1 {
                        account_cursor_sha256: account.account_cursor_sha256,
                        limit: 16,
                        cursor: Vec::new(),
                    })),
                }
                .encode_to_vec(),
            );
            let Some(QueryResult::ListConversations(conversations)) = conversations.result else {
                panic!("Communications conversations query result");
            };
            for conversation in conversations.conversations {
                let messages = route_communications_query(
                    store,
                    supervisor,
                    63,
                    &CommunicationsQueryRequestV1 {
                        protocol_major: 1,
                        operation: Some(Operation::ListConversationMessages(
                            ListConversationMessagesRequestV1 {
                                conversation_id: conversation.conversation_id,
                                limit: 16,
                                cursor: Vec::new(),
                            },
                        )),
                    }
                    .encode_to_vec(),
                );
                let Some(QueryResult::ListConversationMessages(messages)) = messages.result else {
                    panic!("Communications messages query result");
                };
                for message in messages.messages {
                    let anchors = route_communications_query(
                        store,
                        supervisor,
                        64,
                        &CommunicationsQueryRequestV1 {
                            protocol_major: 1,
                            operation: Some(Operation::ListMessageAttachmentAnchors(
                                ListMessageAttachmentAnchorsRequestV1 {
                                    message_id: message.message_id,
                                    limit: 16,
                                    cursor: Vec::new(),
                                },
                            )),
                        }
                        .encode_to_vec(),
                    );
                    let Some(QueryResult::ListMessageAttachmentAnchors(anchors)) = anchors.result
                    else {
                        panic!("Communications attachment anchors query result");
                    };
                    if let Some(anchor) = anchors
                        .anchors
                        .into_iter()
                        .find(|anchor| anchor.attachment_anchor_id == attachment_anchor_id)
                    {
                        return anchor.state;
                    }
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "Communications attachment state was not projected"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn publish_exact(
    runtime: &tokio::runtime::Runtime,
    client: &async_nats::Client,
    subject: &str,
    exact_bytes: &[u8],
) {
    runtime.block_on(async {
        async_nats::jetstream::new(client.clone())
            .publish(subject.to_owned(), exact_bytes.to_vec().into())
            .await
            .expect("publish attachment conformance envelope")
            .await
            .expect("acknowledge attachment conformance envelope");
    });
}

fn current_unix_seconds() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Mail attachment conformance clock")
            .as_secs(),
    )
    .expect("Mail attachment conformance timestamp")
}

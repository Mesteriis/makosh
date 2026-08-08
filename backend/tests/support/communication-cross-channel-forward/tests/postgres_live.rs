//! Disposable PostgreSQL proof for the cross-channel forward durable lifecycle.

use makosh_communication_cross_channel_forward_core::{
    CrossChannelForwardDraftV1, CrossChannelForwardStateV1,
};
use makosh_communication_cross_channel_forward_persistence::{
    CommunicationCrossChannelForwardPersistenceV1, CreateCrossChannelForwardOutcomeV1,
    CreateCrossChannelForwardV1, CrossChannelForwardBlobReceiptV1,
    CrossChannelForwardCleanupReasonV1, CrossChannelForwardDeliveryRejectedEventV1,
    CrossChannelForwardDeliverySubmittedEventV1, CrossChannelForwardPersistenceConformanceV1,
    CrossChannelForwardPersistenceErrorV1, CrossChannelForwardPreparedEventV1,
    CrossChannelForwardPreparedSourceV1, CrossChannelForwardWorkStageV1,
};
use makosh_communication_delivery_intent_ingress_api::{
    CommunicationDeliveryIntentIngressEnvelopeContextV1,
    build_communication_delivery_intent_rejected_outbox_record_v1,
    build_communication_delivery_intent_submit_outbox_record_v1,
    build_communication_delivery_intent_submitted_outbox_record_v1,
    wire::{
        CommunicationDeliveryIntentIngressRejectCodeV1, CommunicationDeliveryIntentRejectedV1,
        CommunicationDeliveryIntentSubmittedV1, DeliveryIntentBodySourceReceiptV1,
    },
};
use makosh_communications_cross_channel_forward_source_api::{
    CrossChannelForwardSourceEnvelopeContextV1,
    build_cross_channel_forward_source_prepare_outbox_record_v1,
    build_cross_channel_forward_source_prepared_outbox_record_v1,
    build_cross_channel_forward_source_rejected_outbox_record_v1,
    wire::{
        CrossChannelForwardBodySourceReceiptV1, CrossChannelForwardSourcePreparedV1,
        CrossChannelForwardSourceRejectCodeV1, CrossChannelForwardSourceRejectedV1,
    },
};

const POSTGRES_URL: &str = "MAKOSH_COMMUNICATION_CROSS_CHANNEL_FORWARD_POSTGRES_URL";
const OWNER: &str = "owner-1";

#[tokio::test]
#[ignore = "requires the disposable cross-channel forward PostgreSQL contour"]
async fn durable_forward_survives_reconnect_and_fences_conflicts_claims_and_cleanup() {
    let database_url = required(POSTGRES_URL);
    let persistence = connect(&database_url).await;
    CrossChannelForwardPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install cross-channel forward schema");
    persistence
        .verify_storage_ready()
        .await
        .expect("verify cross-channel forward storage");

    let create = create_command(1, 2, 3);
    assert_eq!(
        persistence.create_forward(create.clone()).await,
        Ok(CreateCrossChannelForwardOutcomeV1::Created { state_revision: 1 })
    );
    assert_eq!(
        persistence.create_forward(create).await,
        Ok(CreateCrossChannelForwardOutcomeV1::Existing { state_revision: 1 })
    );
    let conflicting = create_command(1, 9, 3);
    assert_eq!(
        persistence.create_forward(conflicting).await,
        Err(CrossChannelForwardPersistenceErrorV1::Conflict)
    );

    let preparing = persistence
        .claim_next_forward(OWNER, "worker-1", 1_100)
        .await
        .expect("claim source preparation")
        .expect("forward must be due");
    assert_eq!(
        preparing.stage,
        CrossChannelForwardWorkStageV1::PreparingSource
    );
    assert_eq!(preparing.prepared_source, None);
    let prepared_source = CrossChannelForwardPreparedSourceV1 {
        source_revision: 7,
        body_sha256: [5; 32],
        body_length: 12,
        blob_reference: vec![6; 16],
        custody_proof: vec![7; 32],
    };
    persistence
        .record_prepared_source(&preparing, &prepared_source, 1_200)
        .await
        .expect("persist prepared source without plaintext body");

    let prepared_claim = persistence
        .claim_next_forward(OWNER, "worker-2", 1_200)
        .await
        .expect("claim prepared source")
        .expect("prepared forward must be due");
    assert_eq!(
        prepared_claim.prepared_source,
        Some(prepared_source.clone())
    );
    persistence
        .reschedule_claim(&prepared_claim, 1_500, 1_250)
        .await
        .expect("persist dependency outage retry");
    drop(persistence);

    let reopened = connect(&database_url).await;
    assert_eq!(
        reopened
            .claim_next_forward(OWNER, "worker-3", 1_499)
            .await
            .expect("query before retry deadline"),
        None
    );
    let retried = reopened
        .claim_next_forward(OWNER, "worker-3", 1_500)
        .await
        .expect("claim after reconnect")
        .expect("retry must survive reconnect");
    assert_eq!(retried.attempt_count, 1);
    let dispatching = reopened
        .begin_dispatch(&retried, 1_600)
        .await
        .expect("enter durable dispatch state");
    assert_eq!(
        dispatching.stage,
        CrossChannelForwardWorkStageV1::Dispatching
    );
    let mut stale_dispatching = dispatching.clone();
    stale_dispatching.claim_epoch -= 1;
    assert_eq!(
        reopened
            .mark_delivery_accepted(&stale_dispatching, [8; 16], 1_700)
            .await,
        Err(CrossChannelForwardPersistenceErrorV1::ClaimLost)
    );
    reopened
        .mark_delivery_accepted(&dispatching, [8; 16], 1_700)
        .await
        .expect("persist downstream acceptance");

    let status = reopened
        .status(OWNER, &[1; 16])
        .await
        .expect("read terminal status");
    assert_eq!(status.state, CrossChannelForwardStateV1::DeliveryAccepted);
    assert_eq!(status.state_revision, 4);
    assert_eq!(status.delivery_intent_id, Some([8; 16]));
    assert_eq!(status.error_code, None);
    let transitions = reopened
        .client_realtime_window(OWNER, None, 16)
        .await
        .expect("replay client-safe state");
    assert_eq!(
        transitions
            .iter()
            .map(|transition| transition.state)
            .collect::<Vec<_>>(),
        vec![
            CrossChannelForwardStateV1::Accepted,
            CrossChannelForwardStateV1::PreparingSource,
            CrossChannelForwardStateV1::Dispatching,
            CrossChannelForwardStateV1::DeliveryAccepted,
        ]
    );

    let cleanup = reopened
        .next_cleanup(OWNER, 1_700)
        .await
        .expect("read cleanup queue")
        .expect("terminal source custody must be queued");
    assert_eq!(cleanup.forward_id, [1; 16]);
    assert_eq!(
        cleanup.reason,
        CrossChannelForwardCleanupReasonV1::DeliveryAccepted
    );
    reopened
        .reschedule_cleanup(OWNER, &[1; 16], 0, 2_000, 1_800)
        .await
        .expect("persist cleanup outage");
    drop(reopened);

    let after_cleanup_restart = connect(&database_url).await;
    assert_eq!(
        after_cleanup_restart
            .next_cleanup(OWNER, 1_999)
            .await
            .expect("cleanup before retry deadline"),
        None
    );
    let cleanup = after_cleanup_restart
        .next_cleanup(OWNER, 2_000)
        .await
        .expect("cleanup after retry deadline")
        .expect("cleanup retry survives reconnect");
    assert_eq!(cleanup.attempt_count, 1);
    after_cleanup_restart
        .complete_cleanup(OWNER, &[1; 16], 2_100)
        .await
        .expect("complete custody release");
    assert_eq!(
        after_cleanup_restart
            .next_cleanup(OWNER, 2_100)
            .await
            .expect("completed queue"),
        None
    );
    assert_eq!(
        after_cleanup_restart.status("owner-2", &[1; 16]).await,
        Err(CrossChannelForwardPersistenceErrorV1::NotFound)
    );
}

#[tokio::test]
#[ignore = "requires the disposable cross-channel forward PostgreSQL contour"]
async fn event_handoff_is_atomic_replay_fenced_and_survives_reconnect() {
    let database_url = required(POSTGRES_URL);
    let persistence = connect(&database_url).await;
    CrossChannelForwardPersistenceConformanceV1::install_schema(&persistence)
        .await
        .expect("install cross-channel forward schema");
    persistence
        .create_forward(create_command(1, 2, 3))
        .await
        .expect("create forward");

    assert_eq!(
        persistence
            .next_source_prepare_candidate(OWNER)
            .await
            .expect("source prepare candidate")
            .expect("accepted operation"),
        makosh_communication_cross_channel_forward_persistence::
            CrossChannelForwardSourcePrepareCandidateV1 {
                forward_id: [1; 16],
                source_message_id: [2; 16],
                target_conversation_id: [3; 16],
            }
    );
    let source_context = CrossChannelForwardSourceEnvelopeContextV1 {
        module_id: "makosh-communication-cross-channel-forward-runtime".to_owned(),
        runtime_instance_id: "forward-runtime-1".to_owned(),
        runtime_generation: 7,
        recorded_at_unix_seconds: 1_800_000_000,
        recorded_at_nanos: 0,
    };
    let source_prepare = build_cross_channel_forward_source_prepare_outbox_record_v1(
        [1; 16],
        [2; 16],
        [3; 16],
        OWNER,
        1_800_000_030,
        &source_context,
    )
    .expect("source prepare");
    persistence
        .persist_source_prepare_outbox(OWNER, [1; 16], &source_prepare, 1_100)
        .await
        .expect("persist source prepare outbox");
    assert_eq!(
        persistence
            .next_source_prepare_candidate(OWNER)
            .await
            .expect("source prepare candidate after commit"),
        None
    );
    persistence
        .persist_source_prepare_outbox(OWNER, [1; 16], &source_prepare, 1_100)
        .await
        .expect("exact source prepare replay");
    let pending = persistence
        .pending_event_outbox(16)
        .await
        .expect("source prepare outbox");
    assert_eq!(pending, vec![source_prepare.clone()]);
    persistence
        .mark_event_outbox_published(*source_prepare.message_id(), 1_150)
        .await
        .expect("mark source prepare published");

    let communications_context = CrossChannelForwardSourceEnvelopeContextV1 {
        module_id: "makosh-communications-runtime".to_owned(),
        runtime_instance_id: "communications-runtime-1".to_owned(),
        runtime_generation: 11,
        recorded_at_unix_seconds: 1_800_000_001,
        recorded_at_nanos: 0,
    };
    let source_prepared = build_cross_channel_forward_source_prepared_outbox_record_v1(
        [1; 16],
        CrossChannelForwardSourcePreparedV1 {
            forward_id: vec![1; 16],
            source_message_id: vec![2; 16],
            target_conversation_id: vec![3; 16],
            source_evidence_id: vec![9; 16],
            source_evidence_revision: 5,
            body_source: Some(CrossChannelForwardBodySourceReceiptV1 {
                reference_id: vec![6; 16],
                declared_bytes: 42,
                sha256: vec![7; 32],
                custody_transfer_source_proof: vec![8; 64],
            }),
            logical_owner_id: OWNER.to_owned(),
        },
        &communications_context,
    )
    .expect("source prepared");
    let prepared_event = CrossChannelForwardPreparedEventV1 {
        result_message_id: *source_prepared.message_id(),
        envelope_sha256: *source_prepared.envelope_sha256(),
        logical_owner_id: OWNER.to_owned(),
        forward_id: [1; 16],
        source_message_id: [2; 16],
        target_conversation_id: [3; 16],
        source_evidence_id: [9; 16],
        source_evidence_revision: 5,
        source_body: CrossChannelForwardBlobReceiptV1 {
            reference_id: [6; 16],
            declared_bytes: 42,
            sha256: [7; 32],
            custody_transfer_source_proof: vec![8; 64],
        },
    };
    let delivery_body = CrossChannelForwardBlobReceiptV1 {
        reference_id: [10; 16],
        declared_bytes: 42,
        sha256: [7; 32],
        custody_transfer_source_proof: vec![11; 64],
    };
    let delivery_submit = build_communication_delivery_intent_submit_outbox_record_v1(
        [1; 16],
        [3; 16],
        Some([4; 16]),
        DeliveryIntentBodySourceReceiptV1 {
            reference_id: delivery_body.reference_id.to_vec(),
            declared_bytes: delivery_body.declared_bytes,
            sha256: delivery_body.sha256.to_vec(),
            custody_transfer_source_proof: delivery_body.custody_transfer_source_proof.clone(),
        },
        OWNER,
        1_800_000_031,
        &CommunicationDeliveryIntentIngressEnvelopeContextV1 {
            module_id: "makosh-communication-cross-channel-forward-runtime".to_owned(),
            runtime_instance_id: "forward-runtime-1".to_owned(),
            runtime_generation: 7,
            recorded_at_unix_seconds: 1_800_000_001,
            recorded_at_nanos: 0,
        },
    )
    .expect("delivery submit");
    persistence
        .persist_source_prepared_and_delivery_submit(
            &prepared_event,
            &delivery_body,
            &delivery_submit,
            1_200,
        )
        .await
        .expect("atomic source result and delivery submit");
    persistence
        .persist_source_prepared_and_delivery_submit(
            &prepared_event,
            &delivery_body,
            &delivery_submit,
            1_200,
        )
        .await
        .expect("exact prepared replay");

    let status = persistence
        .status(OWNER, &[1; 16])
        .await
        .expect("dispatch status");
    assert_eq!(status.state, CrossChannelForwardStateV1::Dispatching);
    assert_eq!(status.delivery_intent_id, Some([1; 16]));
    let pending = persistence
        .pending_event_outbox(16)
        .await
        .expect("delivery submit outbox");
    assert_eq!(pending, vec![delivery_submit.clone()]);

    let mut conflicting = prepared_event;
    conflicting.envelope_sha256 = [13; 32];
    assert_eq!(
        persistence
            .persist_source_prepared_and_delivery_submit(
                &conflicting,
                &delivery_body,
                &delivery_submit,
                1_201,
            )
            .await,
        Err(CrossChannelForwardPersistenceErrorV1::Conflict)
    );
    drop(persistence);

    let reopened = connect(&database_url).await;
    assert_eq!(
        reopened
            .pending_event_outbox(16)
            .await
            .expect("outbox after reconnect"),
        vec![delivery_submit.clone()]
    );
    reopened
        .mark_event_outbox_published(*delivery_submit.message_id(), 1_300)
        .await
        .expect("mark delivery submit published");
    assert_eq!(
        reopened
            .pending_event_outbox(16)
            .await
            .expect("empty published outbox"),
        Vec::new()
    );
    let delivery_result = build_communication_delivery_intent_submitted_outbox_record_v1(
        *delivery_submit.message_id(),
        CommunicationDeliveryIntentSubmittedV1 {
            intent_id: vec![1; 16],
            logical_owner_id: OWNER.to_owned(),
        },
        &CommunicationDeliveryIntentIngressEnvelopeContextV1 {
            module_id: "makosh-communication-delivery-intent-runtime".to_owned(),
            runtime_instance_id: "delivery-runtime-1".to_owned(),
            runtime_generation: 13,
            recorded_at_unix_seconds: 1_800_000_003,
            recorded_at_nanos: 0,
        },
    )
    .expect("delivery submitted");
    let submitted_event = CrossChannelForwardDeliverySubmittedEventV1 {
        result_message_id: *delivery_result.message_id(),
        envelope_sha256: *delivery_result.envelope_sha256(),
        logical_owner_id: OWNER.to_owned(),
        delivery_intent_id: [1; 16],
        delivery_submit_message_id: *delivery_submit.message_id(),
    };
    reopened
        .persist_delivery_submitted(&submitted_event, 1_350)
        .await
        .expect("persist downstream durable admission");
    reopened
        .persist_delivery_submitted(&submitted_event, 1_350)
        .await
        .expect("exact delivery result replay");
    let terminal = reopened
        .status(OWNER, &[1; 16])
        .await
        .expect("terminal event-backed status");
    assert_eq!(terminal.state, CrossChannelForwardStateV1::DeliveryAccepted);
    assert_eq!(terminal.delivery_intent_id, Some([1; 16]));
    let cleanup = reopened
        .next_cleanup(OWNER, 1_350)
        .await
        .expect("event terminal cleanup")
        .expect("source custody cleanup must be durable");
    assert_eq!(cleanup.blob_reference, [6; 16]);
    assert_eq!(cleanup.declared_bytes, 42);
    assert_eq!(cleanup.sha256, [7; 32]);
    assert_eq!(cleanup.custody_proof, vec![8; 64]);
    reopened
        .complete_cleanup(OWNER, &[1; 16], 1_360)
        .await
        .expect("complete event terminal cleanup");
    let mut conflicting_delivery = submitted_event;
    conflicting_delivery.envelope_sha256 = [14; 32];
    assert_eq!(
        reopened
            .persist_delivery_submitted(&conflicting_delivery, 1_361)
            .await,
        Err(CrossChannelForwardPersistenceErrorV1::Conflict)
    );

    reopened
        .create_forward(create_command(30, 31, 32))
        .await
        .expect("create delivery-rejected forward");
    let rejected_delivery_prepare = build_cross_channel_forward_source_prepare_outbox_record_v1(
        [30; 16],
        [31; 16],
        [32; 16],
        OWNER,
        1_800_000_033,
        &source_context,
    )
    .expect("delivery-rejected source prepare");
    reopened
        .persist_source_prepare_outbox(OWNER, [30; 16], &rejected_delivery_prepare, 1_370)
        .await
        .expect("persist delivery-rejected source prepare");
    let rejected_delivery_source = CrossChannelForwardPreparedEventV1 {
        result_message_id: [33; 16],
        envelope_sha256: [34; 32],
        logical_owner_id: OWNER.to_owned(),
        forward_id: [30; 16],
        source_message_id: [31; 16],
        target_conversation_id: [32; 16],
        source_evidence_id: [35; 16],
        source_evidence_revision: 1,
        source_body: CrossChannelForwardBlobReceiptV1 {
            reference_id: [36; 16],
            declared_bytes: 42,
            sha256: [37; 32],
            custody_transfer_source_proof: vec![38; 64],
        },
    };
    let rejected_delivery_body = CrossChannelForwardBlobReceiptV1 {
        reference_id: [39; 16],
        declared_bytes: 42,
        sha256: [37; 32],
        custody_transfer_source_proof: vec![40; 64],
    };
    let rejected_delivery_submit = build_communication_delivery_intent_submit_outbox_record_v1(
        [30; 16],
        [32; 16],
        Some([4; 16]),
        DeliveryIntentBodySourceReceiptV1 {
            reference_id: rejected_delivery_body.reference_id.to_vec(),
            declared_bytes: rejected_delivery_body.declared_bytes,
            sha256: rejected_delivery_body.sha256.to_vec(),
            custody_transfer_source_proof: rejected_delivery_body
                .custody_transfer_source_proof
                .clone(),
        },
        OWNER,
        1_800_000_034,
        &source_context_for_delivery(),
    )
    .expect("rejected delivery submit");
    reopened
        .persist_source_prepared_and_delivery_submit(
            &rejected_delivery_source,
            &rejected_delivery_body,
            &rejected_delivery_submit,
            1_380,
        )
        .await
        .expect("persist delivery-rejected dispatch");
    let rejected_delivery_result =
        build_communication_delivery_intent_rejected_outbox_record_v1(
            *rejected_delivery_submit.message_id(),
            CommunicationDeliveryIntentRejectedV1 {
                intent_id: vec![30; 16],
                code: CommunicationDeliveryIntentIngressRejectCodeV1::
                    CommunicationDeliveryIntentIngressRejectCodePolicy as i32,
                logical_owner_id: OWNER.to_owned(),
            },
            &delivery_result_context(),
        )
        .expect("delivery rejected");
    let rejected_delivery_event = CrossChannelForwardDeliveryRejectedEventV1 {
        result_message_id: *rejected_delivery_result.message_id(),
        envelope_sha256: *rejected_delivery_result.envelope_sha256(),
        logical_owner_id: OWNER.to_owned(),
        delivery_intent_id: [30; 16],
        delivery_submit_message_id: *rejected_delivery_submit.message_id(),
        rejection_code: 4,
    };
    reopened
        .persist_delivery_rejected(&rejected_delivery_event, 1_390)
        .await
        .expect("persist downstream rejection");
    let rejected_delivery_status = reopened
        .status(OWNER, &[30; 16])
        .await
        .expect("delivery-rejected status");
    assert_eq!(
        rejected_delivery_status.state,
        CrossChannelForwardStateV1::Rejected
    );
    assert_eq!(rejected_delivery_status.error_code, Some(4));
    assert_eq!(
        reopened
            .next_cleanup(OWNER, 1_390)
            .await
            .expect("delivery-rejected cleanup")
            .expect("rejected delivery must enqueue cleanup")
            .reason,
        CrossChannelForwardCleanupReasonV1::Rejected
    );

    reopened
        .create_forward(create_command(20, 21, 22))
        .await
        .expect("create rejected forward");
    let rejected_prepare = build_cross_channel_forward_source_prepare_outbox_record_v1(
        [20; 16],
        [21; 16],
        [22; 16],
        OWNER,
        1_800_000_032,
        &source_context,
    )
    .expect("rejected source prepare");
    reopened
        .persist_source_prepare_outbox(OWNER, [20; 16], &rejected_prepare, 1_400)
        .await
        .expect("persist rejected source prepare");
    let rejected_result = build_cross_channel_forward_source_rejected_outbox_record_v1(
        [20; 16],
        CrossChannelForwardSourceRejectedV1 {
            forward_id: vec![20; 16],
            code: CrossChannelForwardSourceRejectCodeV1::CrossChannelForwardSourceRejectCodeSourceMissingOrInactive
                as i32,
            logical_owner_id: OWNER.to_owned(),
        },
        &communications_context,
    )
    .expect("source rejected");
    let rejected_event =
        makosh_communication_cross_channel_forward_persistence::CrossChannelForwardRejectedEventV1 {
            result_message_id: *rejected_result.message_id(),
            envelope_sha256: *rejected_result.envelope_sha256(),
            logical_owner_id: OWNER.to_owned(),
            forward_id: [20; 16],
            rejection_code: 2,
        };
    reopened
        .persist_source_rejected(&rejected_event, 1_500)
        .await
        .expect("persist source rejection");
    reopened
        .persist_source_rejected(&rejected_event, 1_500)
        .await
        .expect("exact source rejection replay");
    let rejected_status = reopened
        .status(OWNER, &[20; 16])
        .await
        .expect("rejected status");
    assert_eq!(rejected_status.state, CrossChannelForwardStateV1::Rejected);
    assert_eq!(rejected_status.error_code, Some(2));
}

fn create_command(
    forward_id: u8,
    source_message_id: u8,
    target_conversation_id: u8,
) -> CreateCrossChannelForwardV1 {
    CreateCrossChannelForwardV1 {
        logical_owner_id: OWNER.to_owned(),
        draft: CrossChannelForwardDraftV1 {
            forward_operation_id: [forward_id; 16],
            source_message_id: [source_message_id; 16],
            target_conversation_id: [target_conversation_id; 16],
            target_reply_to_message_id: Some([4; 16]),
        },
        created_at_unix_millis: 1_000,
    }
}

fn source_context_for_delivery() -> CommunicationDeliveryIntentIngressEnvelopeContextV1 {
    CommunicationDeliveryIntentIngressEnvelopeContextV1 {
        module_id: "makosh-communication-cross-channel-forward-runtime".to_owned(),
        runtime_instance_id: "forward-runtime-1".to_owned(),
        runtime_generation: 7,
        recorded_at_unix_seconds: 1_800_000_004,
        recorded_at_nanos: 0,
    }
}

fn delivery_result_context() -> CommunicationDeliveryIntentIngressEnvelopeContextV1 {
    CommunicationDeliveryIntentIngressEnvelopeContextV1 {
        module_id: "makosh-communication-delivery-intent-runtime".to_owned(),
        runtime_instance_id: "delivery-runtime-1".to_owned(),
        runtime_generation: 13,
        recorded_at_unix_seconds: 1_800_000_005,
        recorded_at_nanos: 0,
    }
}

async fn connect(database_url: &str) -> CommunicationCrossChannelForwardPersistenceV1 {
    CrossChannelForwardPersistenceConformanceV1::connect_url(database_url)
        .await
        .expect("connect cross-channel forward persistence")
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"))
}

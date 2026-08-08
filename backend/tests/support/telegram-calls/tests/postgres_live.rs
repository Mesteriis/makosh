use makosh_clock_protocol::UtcMillisV1;
use makosh_events_protocol::v1::{
    ActorKindV1, ActorRefV1, CommandMetadataV1, ContractRefV1, DurableEnvelopeV1, FenceKindV1,
    SourceFenceV1, SourceRefV1, durable_envelope_v1::Semantics,
};
use makosh_scheduler_protocol::{
    OwnerJobLeaseV1, SCHEDULER_JOB_DESCRIPTOR_SET_V1, build_owner_job_command_v1,
    v1::OwnerJobTriggerKindV1,
};
use makosh_telegram_calls_core::{
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1,
    TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1, TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1,
    TelegramCallCommand, TelegramCallDirection, TelegramCallDiscardReason,
    TelegramCallFailureCategory, TelegramCallMediaState, TelegramCallMediaUpdate,
    TelegramCallOperationState, TelegramProviderCallState, TelegramProviderCallUpdate,
    telegram_calls_realtime_backfill_idempotency_key_v1,
    telegram_calls_realtime_backfill_job_kind_v1, telegram_calls_realtime_backfill_lease_expiry_v1,
    telegram_calls_realtime_backfill_message_id_v1, telegram_calls_realtime_backfill_run_id_v1,
    telegram_calls_realtime_backfill_scope_v1,
};
use makosh_telegram_calls_persistence::{
    TelegramCallRealtimeEvent, TelegramCallRealtimePayload, TelegramCallsBackfillErrorV1,
    TelegramCallsBackfillPhaseV1, TelegramCallsBackfillStateV1, TelegramCallsPersistence,
    TelegramCallsPersistenceError,
};
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};

const DATABASE_URL_ENV: &str = "MAKOSH_TELEGRAM_CALLS_POSTGRES_URL";

#[tokio::test]
#[ignore = "requires a disposable PostgreSQL database"]
async fn durable_signaling_is_idempotent_fenced_and_restart_safe() {
    let database_url =
        std::env::var(DATABASE_URL_ENV).expect("Telegram Calls conformance URL must be set");
    let persistence = TelegramCallsPersistence::connect_for_conformance(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    persistence
        .reset_prerequisites_for_conformance()
        .await
        .expect("create exact prerequisite schema");
    persistence
        .apply_schema_for_conformance()
        .await
        .expect("apply call history and signaling migrations");
    complete_empty_calls_realtime_backfill(&persistence).await;

    let initiate = TelegramCallCommand::InitiateAudio {
        operation_id: "operation-initiate".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
        provider_user_id: "901".to_owned(),
    };
    let accepted = persistence
        .accept_call_command(&initiate, Some("900"), 1, 1, 100)
        .await
        .expect("durably accept initiate");
    assert!(!accepted.replayed);
    assert_eq!(
        accepted.operation.state,
        TelegramCallOperationState::Accepted
    );

    let replayed = persistence
        .accept_call_command(&initiate, Some("900"), 1, 1, 100)
        .await
        .expect("replay exact idempotency key");
    assert!(replayed.replayed);
    assert_eq!(replayed.operation, accepted.operation);

    let conflicting = TelegramCallCommand::InitiateAudio {
        operation_id: "operation-initiate".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
        provider_user_id: "902".to_owned(),
    };
    assert_eq!(
        persistence
            .accept_call_command(&conflicting, Some("900"), 1, 1, 100)
            .await
            .expect_err("same key with another payload must fail"),
        TelegramCallsPersistenceError::IdempotencyConflict
    );

    let claimed = persistence
        .claim_accepted_call_operations("account-1", 1, 1, 101, 10)
        .await
        .expect("claim accepted operation");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].state, TelegramCallOperationState::Dispatching);
    persistence
        .mark_call_operation_awaiting_provider("account-1", "operation-initiate", Some(77), 102)
        .await
        .expect("persist provider dispatch");

    let provider_update = TelegramProviderCallUpdate {
        account_id: "account-1".to_owned(),
        runtime_generation: 1,
        tdlib_call_id: 77,
        provider_call_unique_id: None,
        provider_user_id: "901".to_owned(),
        direction: TelegramCallDirection::Outgoing,
        state: TelegramProviderCallState::Pending,
        pending_created: true,
        pending_received: false,
        discard_reason: None,
        failure_category: None,
        observed_at_unix_seconds: 103,
    };
    let projected = persistence
        .ingest_provider_update("call-session-1", &provider_update)
        .await
        .expect("project provider call");
    assert!(!projected.replayed);

    let duplicate = persistence
        .ingest_provider_update("different-unused-session", &provider_update)
        .await
        .expect("replay duplicate provider update");
    assert!(duplicate.replayed);
    assert_eq!(duplicate.session.call_session_id, "call-session-1");

    let restarted = persistence.clone();
    let completed = restarted
        .call_operation("account-1", "operation-initiate")
        .await
        .expect("load operation after persistence restart")
        .expect("operation exists");
    assert_eq!(completed.state, TelegramCallOperationState::Completed);
    assert_eq!(completed.tdlib_call_id, Some(77));

    let media_ready = restarted
        .ingest_provider_update(
            "call-session-1",
            &TelegramProviderCallUpdate {
                state: TelegramProviderCallState::MediaReady,
                pending_created: false,
                pending_received: false,
                observed_at_unix_seconds: 104,
                ..provider_update
            },
        )
        .await
        .expect("project media-ready state before mute");
    let media_update = TelegramCallMediaUpdate {
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
        runtime_generation: 1,
        provider_revision: media_ready.session.revision,
        state: TelegramCallMediaState::Connecting,
        observed_at_unix_seconds: 105,
    };
    let connecting = restarted
        .ingest_media_update(&media_update)
        .await
        .expect("persist connecting media");
    assert_eq!(connecting.revision, 1);
    let active = restarted
        .ingest_media_update(&TelegramCallMediaUpdate {
            state: TelegramCallMediaState::Active,
            observed_at_unix_seconds: 106,
            ..media_update.clone()
        })
        .await
        .expect("persist active media");
    assert_eq!(active.revision, 2);
    assert_eq!(active.connected_at_unix_seconds, Some(106));
    let duplicate = restarted
        .ingest_media_update(&TelegramCallMediaUpdate {
            state: TelegramCallMediaState::Active,
            observed_at_unix_seconds: 107,
            ..media_update.clone()
        })
        .await
        .expect("replay duplicate media state");
    assert_eq!(duplicate.revision, 2);
    assert_eq!(
        restarted
            .ingest_media_update(&TelegramCallMediaUpdate {
                runtime_generation: 2,
                state: TelegramCallMediaState::Active,
                observed_at_unix_seconds: 108,
                ..media_update.clone()
            })
            .await
            .expect_err("stale runtime cannot mutate media"),
        TelegramCallsPersistenceError::IdentityConflict
    );
    assert_eq!(
        restarted
            .media_projection("account-1", "call-session-1")
            .await
            .expect("load media after persistence restart")
            .expect("media projection exists"),
        active
    );

    let mute = TelegramCallCommand::SetLocalMute {
        operation_id: "operation-mute".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
        muted: true,
    };
    restarted
        .accept_call_command(&mute, None, 1, 1, 109)
        .await
        .expect("accept local mute");
    let claimed = restarted
        .claim_accepted_call_operations("account-1", 1, 1, 110, 10)
        .await
        .expect("claim local mute");
    assert_eq!(claimed.len(), 1);
    restarted
        .complete_local_mute_operation("account-1", "operation-mute", 111)
        .await
        .expect("complete local mute");
    assert!(
        restarted
            .local_mute("account-1", "call-session-1")
            .await
            .expect("read local mute")
    );

    let end = TelegramCallCommand::End {
        operation_id: "operation-end-stale".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
    };
    restarted
        .accept_call_command(&end, None, 1, 1, 112)
        .await
        .expect("accept end under old fence");
    assert_eq!(
        restarted
            .reconcile_stale_call_operations("account-1", 2, 2, 113)
            .await
            .expect("fence stale command"),
        1
    );
    let failed = restarted
        .call_operation("account-1", "operation-end-stale")
        .await
        .expect("load fenced operation")
        .expect("fenced operation exists");
    assert_eq!(failed.state, TelegramCallOperationState::Failed);
    assert_eq!(
        failed.failure_category,
        Some(TelegramCallFailureCategory::Permission)
    );

    let ambiguous_end = TelegramCallCommand::End {
        operation_id: "operation-end-ambiguous".to_owned(),
        account_id: "account-1".to_owned(),
        call_session_id: "call-session-1".to_owned(),
    };
    restarted
        .accept_call_command(&ambiguous_end, None, 1, 1, 114)
        .await
        .expect("accept end before provider-ambiguous restart");
    let claimed = restarted
        .claim_accepted_call_operations("account-1", 1, 1, 115, 10)
        .await
        .expect("claim end before provider-ambiguous restart");
    assert_eq!(claimed.len(), 1);
    restarted
        .mark_call_operation_awaiting_provider("account-1", "operation-end-ambiguous", None, 116)
        .await
        .expect("persist ambiguous provider dispatch");
    assert_eq!(
        restarted
            .reconcile_stale_call_operations("account-1", 2, 2, 117)
            .await
            .expect("reconcile ambiguous provider dispatch"),
        1
    );
    let ambiguous = restarted
        .call_operation("account-1", "operation-end-ambiguous")
        .await
        .expect("load ambiguous operation")
        .expect("ambiguous operation exists");
    assert_eq!(ambiguous.state, TelegramCallOperationState::Failed);
    assert_eq!(
        ambiguous.failure_category,
        Some(TelegramCallFailureCategory::Unknown)
    );

    let realtime = restarted
        .realtime_after("account-1", 0, 100)
        .await
        .expect("replay unified call events");
    assert!(realtime.len() >= 10);
    assert!(
        realtime
            .windows(2)
            .all(|window| window[0].sequence < window[1].sequence)
    );
}

#[tokio::test]
#[ignore = "requires a disposable PostgreSQL database"]
async fn legacy_realtime_backfill_is_bounded_restart_safe_and_cursor_preserving() {
    let database_url =
        std::env::var(DATABASE_URL_ENV).expect("Telegram Calls conformance URL must be set");
    let persistence = TelegramCallsPersistence::connect_for_conformance(&database_url)
        .await
        .expect("connect to disposable PostgreSQL");
    persistence
        .reset_prerequisites_for_conformance()
        .await
        .expect("create exact prerequisite schema");
    persistence
        .apply_call_history_schema_for_conformance()
        .await
        .expect("apply legacy call history schema");

    for index in 0_i32..257 {
        persistence
            .ingest_legacy_provider_update_for_conformance(
                &format!("legacy-call-{index:03}"),
                &terminal_update(
                    index + 1,
                    1_000 + u64::try_from(index).expect("positive fixture index"),
                ),
            )
            .await
            .expect("persist one real legacy projection");
    }
    persistence
        .apply_calls_upgrade_schemas_for_conformance()
        .await
        .expect("apply signaling, media and owner-job DDL");
    let pre_backfill = persistence
        .ingest_pre_backfill_provider_update_for_conformance(
            "post-upgrade-call",
            &terminal_update(258, 2_000),
        )
        .await
        .expect("persist one pre-backfill dual-written frame");
    let old_cursor = pre_backfill.frame_sequence.expect("legacy external cursor");
    let before = persistence
        .realtime_after("account-1", 0, 200)
        .await
        .expect("pre-backfill public replay remains gated");
    assert!(before.is_empty());

    let envelope = calls_backfill_envelope(1_000, 1);
    let accepted = persistence
        .accept_calls_realtime_backfill_v1(&envelope)
        .await
        .expect("accept exact owner-local job");
    assert_eq!(accepted.state, TelegramCallsBackfillStateV1::Accepted);
    assert_eq!(
        persistence
            .accept_calls_realtime_backfill_v1(&envelope)
            .await
            .expect("duplicate exact command is idempotent"),
        accepted
    );
    let first_lease = persistence
        .claim_calls_realtime_backfill_v1(1, 2_000)
        .await
        .expect("claim first runtime lease");
    assert_eq!(first_lease.lease_epoch, 1);
    assert_eq!(first_lease.phase, TelegramCallsBackfillPhaseV1::Rebase);

    let rebased = persistence
        .execute_calls_realtime_backfill_batch_v1(1, 1, 2_001)
        .await
        .expect("rebase existing external cursor rows");
    assert_eq!(rebased.realtime_events_rebased, 1);
    assert_eq!(
        rebased.execution.phase,
        TelegramCallsBackfillPhaseV1::Backfill
    );
    let first_batch = persistence
        .execute_calls_realtime_backfill_batch_v1(1, 1, 2_002)
        .await
        .expect("copy bounded first source batch");
    assert_eq!(first_batch.source_frames_processed, 256);
    assert_eq!(first_batch.realtime_events_inserted, 256);
    assert_eq!(first_batch.execution.checkpoint_frame_sequence, 256);
    assert_eq!(
        first_batch.execution.state,
        TelegramCallsBackfillStateV1::Running
    );

    let takeover = persistence
        .claim_calls_realtime_backfill_v1(2, 2_003)
        .await
        .expect("new runtime generation takes over immediately");
    assert_eq!(takeover.lease_epoch, 2);
    assert_eq!(
        persistence
            .execute_calls_realtime_backfill_batch_v1(1, 1, 2_004)
            .await
            .expect_err("stale runtime lease must be fenced"),
        TelegramCallsBackfillErrorV1::StaleLease
    );
    let completed = persistence
        .execute_calls_realtime_backfill_batch_v1(2, 2, 2_005)
        .await
        .expect("resume at exact source checkpoint");
    assert_eq!(completed.source_frames_processed, 2);
    assert_eq!(completed.realtime_events_inserted, 1);
    assert_eq!(
        completed.execution.state,
        TelegramCallsBackfillStateV1::Succeeded
    );
    assert_eq!(completed.execution.processed_frame_count, 258);
    assert_eq!(completed.execution.backfilled_frame_count, 257);

    let restarted = persistence.clone();
    assert_eq!(
        restarted
            .calls_realtime_backfill_execution_v1()
            .await
            .expect("load durable terminal execution")
            .expect("execution exists")
            .state,
        TelegramCallsBackfillStateV1::Succeeded
    );
    let replay = replay_all_after(&restarted, old_cursor).await;
    assert_eq!(replay.len(), 258);
    assert!(
        replay
            .windows(2)
            .all(|window| window[0].sequence < window[1].sequence)
    );
    let call_ids = replay
        .iter()
        .map(|event| match &event.payload {
            TelegramCallRealtimePayload::Call { session, .. } => session.call_session_id.as_str(),
            TelegramCallRealtimePayload::Operation(_) => panic!("no operation fixture"),
        })
        .collect::<Vec<_>>();
    assert_eq!(call_ids.first().copied(), Some("legacy-call-000"));
    assert_eq!(call_ids.get(256).copied(), Some("legacy-call-256"));
    assert_eq!(call_ids.last().copied(), Some("post-upgrade-call"));
}

fn terminal_update(tdlib_call_id: i32, observed_at: u64) -> TelegramProviderCallUpdate {
    TelegramProviderCallUpdate {
        account_id: "account-1".to_owned(),
        runtime_generation: 1,
        tdlib_call_id,
        provider_call_unique_id: Some(10_000 + i64::from(tdlib_call_id)),
        provider_user_id: format!("provider-user-{tdlib_call_id}"),
        direction: TelegramCallDirection::Incoming,
        state: TelegramProviderCallState::Discarded,
        pending_created: false,
        pending_received: false,
        discard_reason: Some(TelegramCallDiscardReason::HungUp),
        failure_category: None,
        observed_at_unix_seconds: observed_at,
    }
}

fn calls_backfill_envelope(accepted_at_unix_millis: i64, runtime_generation: u64) -> Vec<u8> {
    let expires_at =
        telegram_calls_realtime_backfill_lease_expiry_v1(accepted_at_unix_millis).expect("expiry");
    let run_id = telegram_calls_realtime_backfill_run_id_v1();
    let payload = build_owner_job_command_v1(
        &telegram_calls_realtime_backfill_job_kind_v1(),
        &telegram_calls_realtime_backfill_scope_v1(),
        OwnerJobTriggerKindV1::UpgradeReconciliation,
        UtcMillisV1::new(accepted_at_unix_millis),
        OwnerJobLeaseV1::new(run_id, 1, UtcMillisV1::new(expires_at)).expect("lease"),
    )
    .expect("command")
    .encode_to_vec();
    DurableEnvelopeV1 {
        envelope_major: 1,
        envelope_revision: 1,
        message_id: telegram_calls_realtime_backfill_message_id_v1().to_vec(),
        contract: Some(ContractRefV1 {
            owner: TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_OWNER_V1.to_owned(),
            name: TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_NAME_V1.to_owned(),
            major: u32::from(TELEGRAM_CALLS_REALTIME_BACKFILL_JOB_MAJOR_V1),
            revision: 1,
            schema_sha256: Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).to_vec(),
        }),
        source: Some(SourceRefV1 {
            module_id: "makosh-telegram-runtime".to_owned(),
            runtime_instance_id: runtime_source_reference("runtime-1").to_vec(),
            runtime_generation,
        }),
        recorded_at: Some(timestamp(accepted_at_unix_millis)),
        partition_key: TELEGRAM_CALLS_REALTIME_BACKFILL_SCOPE_V1
            .as_bytes()
            .to_vec(),
        causation_message_id: Vec::new(),
        correlation_id: run_id.bytes().to_vec(),
        actor: Some(ActorRefV1 {
            kind: ActorKindV1::System as i32,
            actor_id: b"makosh-telegram-runtime".to_vec(),
        }),
        trace: None,
        source_fence: Some(SourceFenceV1 {
            kind: FenceKindV1::RuntimeLease as i32,
            scope_id: b"makosh-telegram-runtime".to_vec(),
            epoch: runtime_generation,
        }),
        semantics: Some(Semantics::Command(CommandMetadataV1 {
            command_id: run_id.bytes().to_vec(),
            target_capability: "job_execute".to_owned(),
            idempotency_key: telegram_calls_realtime_backfill_idempotency_key_v1().to_vec(),
            deadline: Some(timestamp(expires_at)),
            logical_attempt: 1,
        })),
        payload,
    }
    .encode_to_vec()
}

async fn complete_empty_calls_realtime_backfill(persistence: &TelegramCallsPersistence) {
    let envelope = calls_backfill_envelope(1_000, 1);
    persistence
        .accept_calls_realtime_backfill_v1(&envelope)
        .await
        .expect("accept empty Calls realtime backfill");
    let mut execution = persistence
        .claim_calls_realtime_backfill_v1(1, 1_001)
        .await
        .expect("claim empty Calls realtime backfill");
    for now_unix_millis in 1_002..1_006 {
        if execution.state == TelegramCallsBackfillStateV1::Succeeded {
            return;
        }
        execution = persistence
            .execute_calls_realtime_backfill_batch_v1(1, execution.lease_epoch, now_unix_millis)
            .await
            .expect("execute empty Calls realtime backfill")
            .execution;
    }
    assert_eq!(
        execution.state,
        TelegramCallsBackfillStateV1::Succeeded,
        "empty Calls realtime backfill must complete before signaling"
    );
}

fn timestamp(unix_millis: i64) -> Timestamp {
    Timestamp {
        seconds: unix_millis.div_euclid(1_000),
        nanos: i32::try_from(unix_millis.rem_euclid(1_000) * 1_000_000).expect("nanos"),
    }
}

fn runtime_source_reference(runtime_instance_id: &str) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"makosh.runtime.source-reference.v1\0");
    hasher.update(runtime_instance_id.as_bytes());
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..16].try_into().expect("fixed digest prefix")
}

async fn replay_all_after(
    persistence: &TelegramCallsPersistence,
    mut cursor: u64,
) -> Vec<TelegramCallRealtimeEvent> {
    let mut replay = Vec::new();
    loop {
        let page = persistence
            .realtime_after("account-1", cursor, 200)
            .await
            .expect("replay page");
        if page.is_empty() {
            return replay;
        }
        cursor = page.last().expect("non-empty page").sequence;
        replay.extend(page);
    }
}

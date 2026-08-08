//! Managed Gateway conformance for Mail sync replay and health history.

use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, MailSyncInboxRequestV1,
    client_contract::MailClientContractV1,
    sync_health::{
        MailSyncFailureCodeV1, MailSyncHealthQueryResponseV1, MailSyncHealthQueryV1,
        MailSyncOutcomeV1, MailSyncProviderPathReadinessV1, MailSyncTriggerV1,
    },
};
use makosh_mail_runtime::client_port::{
    MailClientPortErrorV1, decode_module_response, encode_module_request,
};

use super::*;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};

pub(super) fn assert_mail_sync_replay_and_health(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    operation_id: &str,
    expected_observed_messages: u64,
    request_id: u64,
) {
    let replay = route_request(
        store,
        supervisor,
        mail,
        MailClientContractV1::Sync,
        request_id,
        MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: operation_id.to_owned(),
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        }),
    )
    .expect("replay exact Mail sync operation");
    let (_, replay) = decode_module_response(MailClientContractV1::Sync, &replay)
        .expect("decode replayed Mail sync operation");
    assert_eq!(
        replay,
        MailClientResponseV1::SyncInboxAccepted {
            operation_id: operation_id.to_owned(),
        }
    );

    let status = query_sync_health(
        store,
        supervisor,
        mail,
        request_id + 1,
        MailSyncHealthQueryV1::GetStatus {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
        },
    );
    let MailSyncHealthQueryResponseV1::Status(status) = status else {
        panic!("Mail sync status query returned the wrong response")
    };
    assert_eq!(status.connection_id, MAIL_ACCOUNT_ID);
    assert_eq!(
        status.provider_path_readiness,
        MailSyncProviderPathReadinessV1::Ready
    );
    let latest = status.latest_run.expect("latest Mail sync run");
    assert_successful_run(&latest, mail, operation_id, expected_observed_messages);

    let runs = query_sync_health(
        store,
        supervisor,
        mail,
        request_id + 2,
        MailSyncHealthQueryV1::ListRuns {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: None,
            limit: 1,
        },
    );
    let MailSyncHealthQueryResponseV1::Runs(runs) = runs else {
        panic!("Mail sync run list returned the wrong response")
    };
    assert_eq!(runs.items.len(), 1);
    assert_successful_run(
        &runs.items[0],
        mail,
        operation_id,
        expected_observed_messages,
    );
    let cursor = runs.next_cursor.expect("bounded Mail sync run cursor");
    assert!(!cursor.contains(MAIL_ACCOUNT_ID));
    assert!(!cursor.contains(operation_id));

    let run = query_sync_health(
        store,
        supervisor,
        mail,
        request_id + 3,
        MailSyncHealthQueryV1::GetRun {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            operation_id: operation_id.to_owned(),
        },
    );
    let MailSyncHealthQueryResponseV1::Run(Some(run)) = run else {
        panic!("Mail sync run query returned the wrong response")
    };
    assert_successful_run(&run, mail, operation_id, expected_observed_messages);

    assert_rejected_query(
        store,
        supervisor,
        mail,
        request_id + 4,
        MailSyncHealthQueryV1::GetStatus {
            connection_id: "other-mail-account".to_owned(),
        },
    );
    assert_rejected_query(
        store,
        supervisor,
        mail,
        request_id + 5,
        MailSyncHealthQueryV1::ListRuns {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            cursor: Some(format!("{cursor}x")),
            limit: 1,
        },
    );
    assert_stale_generation_is_interrupted(mail);
}

pub(super) fn assert_sync_run_running(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    operation_id: &str,
    request_id: u64,
) {
    let response = query_sync_health(
        store,
        supervisor,
        mail,
        request_id,
        MailSyncHealthQueryV1::GetRun {
            connection_id: MAIL_ACCOUNT_ID.to_owned(),
            operation_id: operation_id.to_owned(),
        },
    );
    let MailSyncHealthQueryResponseV1::Run(Some(run)) = response else {
        panic!("running Mail sync query returned the wrong response")
    };
    assert_eq!(run.outcome, MailSyncOutcomeV1::Running);
    assert_eq!(run.observed_messages, 0);
}

pub(super) fn wait_for_successful_sync_run(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    operation_id: &str,
    expected_observed_messages: u64,
    request_id: u64,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut attempt = 0_u64;
    loop {
        let response = query_sync_health(
            store,
            supervisor,
            mail,
            request_id + attempt,
            MailSyncHealthQueryV1::GetRun {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                operation_id: operation_id.to_owned(),
            },
        );
        if let MailSyncHealthQueryResponseV1::Run(Some(run)) = response
            && run.outcome == MailSyncOutcomeV1::Succeeded
        {
            assert_successful_run(&run, mail, operation_id, expected_observed_messages);
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Mail sync did not reach a successful terminal state"
        );
        attempt = attempt.saturating_add(1);
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn query_sync_health(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    query: MailSyncHealthQueryV1,
) -> MailSyncHealthQueryResponseV1 {
    let bytes = route_request(
        store,
        supervisor,
        mail,
        MailClientContractV1::SyncHealthQuery,
        request_id,
        MailClientRequestV1::SyncHealthQuery(query),
    )
    .expect("route Mail sync health query");
    for forbidden in [
        b"managed-mail-imap-password".as_slice(),
        b"managed-mail-gmail-access-token".as_slice(),
        b"managed Gmail body".as_slice(),
        b"provider_cursor".as_slice(),
    ] {
        assert!(
            !bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden),
            "Mail sync health response exposed private provider bytes"
        );
    }
    let (actual_request_id, response) =
        decode_module_response(MailClientContractV1::SyncHealthQuery, &bytes)
            .expect("decode Mail sync health query");
    assert_eq!(actual_request_id, request_id);
    let MailClientResponseV1::SyncHealthQuery(response) = response else {
        panic!("Mail sync health route returned the wrong response")
    };
    response
}

fn assert_rejected_query(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    request_id: u64,
    query: MailSyncHealthQueryV1,
) {
    let bytes = route_request(
        store,
        supervisor,
        mail,
        MailClientContractV1::SyncHealthQuery,
        request_id,
        MailClientRequestV1::SyncHealthQuery(query),
    )
    .expect("route rejected Mail sync health query");
    assert_eq!(
        decode_module_response(MailClientContractV1::SyncHealthQuery, &bytes),
        Err(MailClientPortErrorV1::Runtime)
    );
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("observe Mail after rejected sync health query")
    );
}

fn route_request(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
    contract: MailClientContractV1,
    request_id: u64,
    request: MailClientRequestV1,
) -> Result<Vec<u8>, String> {
    let request = encode_module_request(request_id, &request).expect("encode Mail client request");
    let route = ManagedCapabilityRouteRequest::new(
        &mail.registration_id,
        &mail.runtime_instance_id,
        mail.runtime_generation,
        mail.grant_epoch,
        contract.capability_id(),
        &request,
    );
    route_managed_client_request(store, &supervisor.relay_port(), &route)
}

fn assert_successful_run(
    run: &makosh_mail_api::sync_health::MailSyncRunV1,
    mail: &StartedMailRuntime,
    operation_id: &str,
    expected_observed_messages: u64,
) {
    assert_eq!(run.operation_id, operation_id);
    assert_eq!(run.connection_id, MAIL_ACCOUNT_ID);
    assert_eq!(run.outcome, MailSyncOutcomeV1::Succeeded);
    assert_eq!(run.observed_messages, expected_observed_messages);
    assert_eq!(run.runtime_generation, mail.runtime_generation);
    assert!(run.completed_at_unix_seconds.is_some());
    assert_eq!(run.failure_code, None);
}

fn assert_stale_generation_is_interrupted(mail: &StartedMailRuntime) {
    let runtime = tokio::runtime::Runtime::new().expect("Mail sync health persistence runtime");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("current time after Unix epoch")
        .as_secs();
    let now = i64::try_from(now).expect("current Unix time");
    let durable = runtime.block_on(super::mail_event_flow::connect_postgres());
    let operation_id = "managed-mail-stale-generation-probe";
    let begin = runtime
        .block_on(durable.begin_sync_run(
            operation_id,
            MAIL_ACCOUNT_ID,
            MailSyncTriggerV1::Manual,
            mail.runtime_generation + 1,
            now,
        ))
        .expect("seed stale Mail sync run");
    assert!(matches!(
        begin,
        makosh_mail_persistence::MailSyncRunStartOutcomeV1::Started(_)
    ));
    assert_eq!(
        runtime
            .block_on(
                durable.interrupt_stale_sync_runs(mail.runtime_generation, now.saturating_add(1))
            )
            .expect("interrupt stale Mail sync run"),
        1
    );
    let response = runtime
        .block_on(durable.execute_sync_health_query(
            &MailSyncHealthQueryV1::GetRun {
                connection_id: MAIL_ACCOUNT_ID.to_owned(),
                operation_id: operation_id.to_owned(),
            },
            MailSyncProviderPathReadinessV1::Ready,
        ))
        .expect("query interrupted Mail sync run");
    let MailSyncHealthQueryResponseV1::Run(Some(run)) = response else {
        panic!("interrupted Mail sync run is missing")
    };
    assert_eq!(run.outcome, MailSyncOutcomeV1::Interrupted);
    assert_eq!(
        run.failure_code,
        Some(MailSyncFailureCodeV1::RuntimeRestarted)
    );
    assert_eq!(run.runtime_generation, mail.runtime_generation + 1);
}

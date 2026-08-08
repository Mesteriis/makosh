//! Full managed Archive Inspection through NATS outage, Gateway and replayable SSE.

use std::time::{Duration, Instant};

use super::*;
use super::{
    attachment_security_blob_fixture::AttachmentSecurityBlobSourceFixture,
    attachment_security_clamav_fixture::AttachmentSecurityClamAvFixture,
    attachment_security_event_flow::{
        assert_clean_attachment_security_verdict_flow, prepare_communications_attachment_for_scan,
    },
    mail_attachment_flow::wait_for_attachment_state,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use makosh_attachment_archive_inspection_api::{
    ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONNECT_PATH_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_CONTRACT_NAME_V1,
    ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_EVENT_KIND_V1,
    wire::{
        ArchiveInspectionErrorCodeV1, ArchiveInspectionStateV1, ArchiveInspectionStatusChangedV1,
        ArchiveKindV1, GetArchiveInspectionRequestV1, GetArchiveInspectionResponseV1,
        StartArchiveInspectionRequestV1, StartArchiveInspectionResponseV1,
    },
};
use makosh_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use sqlx::{
    Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use zeroize::Zeroizing;

use crate::identity::device::signer::DeviceSigner;

const PRIVATE_ARCHIVE_COMMENT: &[u8] = b"private-archive-comment";

type ArchiveInspectionGateway = makosh_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArchiveInspectionDiagnosticsV1 {
    candidates: i64,
    safety_facts: i64,
    custody_requests: i64,
    pending_custody_outbox: i64,
    custody_results: i64,
    jobs: i64,
    attempts: i64,
    reports: i64,
}

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS, Communications, Attachment Security and Archive Inspection binaries"]
fn managed_archive_inspection_reaches_gateway_sse_and_replays_after_restart() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let clamav = AttachmentSecurityClamAvFixture::start();
    let root = unique_target_root("makosh-managed-archive-inspection");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_archive_inspection_ensemble_release_v1(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    let (owner_signer, _) =
        FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            ARCHIVE_INSPECTION_LOGICAL_OWNER_ID_V1,
            "desktop-1",
            owner_signer.public_key_sec1(),
        ))
        .expect("claim Archive Inspection logical owner");
    super::super::browser_gateway_session::admit_browser_test_device(
        &store,
        ARCHIVE_INSPECTION_LOGICAL_OWNER_ID_V1,
    );

    let admitted_archive = admit_archive_inspection_runtime_v1(&store);
    let admitted_security = admit_attachment_security_runtime(&store);
    let blob_source = AttachmentSecurityBlobSourceFixture::admit(&store);
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    let realtime =
        makosh_gateway_runtime::InMemoryBrowserRealtimeSource::new(64).expect("realtime source");
    configure_route_handler(&supervisor, &store, &data);
    configure_archive_inspection_realtime_v1(&supervisor, &store, realtime.clone());
    supervisor
        .configure_event_credential_handler(Arc::new(UnauthenticatedNatsCredentialHandler::new(
            Arc::clone(&store),
        )))
        .expect("configure Event credential handler");
    start_vault(&supervisor, &store, &data, release.kernel());
    assert_eq!(
        blob_launch::start_from_kernel(
            &supervisor,
            &store,
            release.kernel(),
            &data,
            &root.join("runtime"),
        )
        .expect("start signed Blob runtime"),
        1
    );
    start_storage(
        &supervisor,
        &store,
        release.kernel(),
        &storage_runtime_directory(),
    );
    issue_initial_communications_storage_binding(&store);
    crate::platform::storage::provisioning::apply_reserved_binding(
        &supervisor,
        &store,
        &communications_storage_binding(&store),
    )
    .expect("provision Communications Storage binding");
    let admitted_security =
        prepare_attachment_security_runtime(&supervisor, &store, admitted_security);
    let admitted_archive =
        prepare_archive_inspection_runtime_v1(&supervisor, &store, admitted_archive);
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let security = start_attachment_security_runtime(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_security,
        clamav.port(),
    );
    let archive = start_archive_inspection_runtime_v1(
        &supervisor,
        &store,
        &root.join("runtime"),
        admitted_archive,
    );
    assert_eq!(archive.runtime_generation, 1);
    assert_eq!(security.runtime_generation, 1);

    let zip = private_empty_zip();
    let blob = blob_source.write(&store, &supervisor, &data, [0x91; 16], &zip);
    let attachment = prepare_communications_attachment_for_scan(
        &store,
        "archive-inspection",
        blob.declared_size,
        blob.receipt_sha256,
    );
    assert_clean_attachment_security_verdict_flow(&store, &attachment, &blob, &clamav, &zip);
    assert_eq!(
        wait_for_attachment_state(&store, &supervisor, attachment.attachment_anchor_id),
        makosh_communications_attachment_contract::lifecycle_v1::AttachmentSafetyStateV1::SafeForDelivery
            as u32
    );
    wait_for_archive_evidence();

    let gateway_runtime = tokio::runtime::Runtime::new().expect("Gateway runtime");
    let router = archive_inspection_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let cookie = super::super::browser_gateway_session::authenticate_gateway_router(
        &router,
        &gateway_runtime,
    );
    let request = StartArchiveInspectionRequestV1 {
        protocol_major: 1,
        operation_id: vec![0x92; 16],
        attachment_anchor_id: attachment.attachment_anchor_id.to_vec(),
    };

    set_authenticated_nats_container_running(false);
    let accepted = post_proto::<_, StartArchiveInspectionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(accepted.error, unspecified_error());
    assert_eq!(accepted.run_id.len(), 16);
    wait_for_pending_archive_custody_outbox();
    set_authenticated_nats_container_running(true);

    let ready = wait_for_ready_archive(&router, &gateway_runtime, &cookie, &accepted.run_id);
    assert_eq!(ready.attachment_anchor_id, attachment.attachment_anchor_id);
    assert_eq!(state(ready.state), ArchiveInspectionStateV1::Ready);
    assert_eq!(ready.error, unspecified_error());
    let report = ready.report.as_ref().expect("Archive Inspection report");
    assert_eq!(report.archive_kind, ArchiveKindV1::Zip as i32);
    assert_eq!(report.entry_count, 0);
    assert_eq!(report.total_uncompressed_bytes, 0);
    assert!(report.entries.is_empty());
    assert_private_archive_data_absent(&ready.encode_to_vec(), &blob);

    let first_event =
        read_terminal_archive_sse_event(&router, &gateway_runtime, &cookie, &accepted.run_id);
    let first_payload = ArchiveInspectionStatusChangedV1::decode(first_event.payload.as_slice())
        .expect("Archive Inspection realtime payload");
    assert_eq!(first_payload.run_id, accepted.run_id);
    assert_eq!(state(first_payload.state), ArchiveInspectionStateV1::Ready);
    assert_private_archive_data_absent(&first_event.encode_to_vec(), &blob);
    let first_cursor = first_event.cursor.clone();
    let completed_diagnostics = archive_diagnostics();
    assert_eq!(
        completed_diagnostics,
        ArchiveInspectionDiagnosticsV1 {
            candidates: 1,
            safety_facts: 1,
            custody_requests: 1,
            pending_custody_outbox: 0,
            custody_results: 1,
            jobs: 1,
            attempts: 1,
            reports: 1,
        }
    );

    let duplicate = post_proto::<_, StartArchiveInspectionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1,
        request.clone(),
    );
    assert_eq!(duplicate.run_id, accepted.run_id);
    assert_eq!(state(duplicate.state), ArchiveInspectionStateV1::Ready);
    assert_eq!(archive_diagnostics(), completed_diagnostics);

    let conflicting = post_proto::<_, StartArchiveInspectionResponseV1>(
        &router,
        &gateway_runtime,
        &cookie,
        ATTACHMENT_ARCHIVE_INSPECTION_COMMAND_CONNECT_PATH_V1,
        StartArchiveInspectionRequestV1 {
            attachment_anchor_id: vec![0x93; 16],
            ..request
        },
    );
    assert_eq!(
        error(conflicting.error),
        ArchiveInspectionErrorCodeV1::InvalidRequest
    );
    assert_eq!(archive_diagnostics(), completed_diagnostics);

    assert!(
        realtime
            .revoke_owner(ARCHIVE_INSPECTION_LOGICAL_OWNER_ID_V1)
            .expect("clear Archive Inspection Gateway replay cache")
    );
    let previous_generation = archive.runtime_generation;
    let archive =
        restart_archive_inspection_runtime_v1(&supervisor, &store, &root.join("runtime"), archive);
    assert_eq!(archive.runtime_generation, previous_generation + 1);
    let restarted_router =
        archive_inspection_gateway(&store, &supervisor, &root, &data, realtime.clone());
    let restarted_cookie =
        super::super::browser_gateway_session::authenticate_gateway_router_with_sign_count(
            &restarted_router,
            &gateway_runtime,
            2,
        );
    let replayed = get_archive(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed, ready);
    let replayed_event = read_terminal_archive_sse_event(
        &restarted_router,
        &gateway_runtime,
        &restarted_cookie,
        &accepted.run_id,
    );
    assert_eq!(replayed_event.cursor, first_cursor);
    assert_eq!(replayed_event.payload, first_event.payload);
    assert_private_archive_data_absent(&replayed_event.encode_to_vec(), &blob);
    assert_eq!(
        archive_diagnostics(),
        completed_diagnostics,
        "restart and replay must not transfer Blob custody or execute the parser twice"
    );

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove Archive Inspection fixture");
    std::fs::remove_dir_all(data).expect("remove short Archive Inspection Kernel fixture");
}

fn private_empty_zip() -> Vec<u8> {
    let mut zip = vec![0x50, 0x4b, 0x05, 0x06];
    zip.extend_from_slice(&[0; 16]);
    zip.extend_from_slice(&(PRIVATE_ARCHIVE_COMMENT.len() as u16).to_le_bytes());
    zip.extend_from_slice(PRIVATE_ARCHIVE_COMMENT);
    zip
}

fn wait_for_archive_evidence() {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = archive_diagnostics();
        if diagnostics.candidates == 1 && diagnostics.safety_facts == 1 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Archive Inspection did not persist source evidence: {diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_pending_archive_custody_outbox() {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let diagnostics = archive_diagnostics();
        if diagnostics.custody_requests == 1 && diagnostics.pending_custody_outbox == 1 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Archive Inspection did not retain custody command during NATS outage: {diagnostics:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_ready_archive(
    router: &ArchiveInspectionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetArchiveInspectionResponseV1 {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let response = get_archive(router, runtime, cookie, run_id);
        if matches!(
            state(response.state),
            ArchiveInspectionStateV1::Ready | ArchiveInspectionStateV1::Rejected
        ) {
            assert_eq!(
                state(response.state),
                ArchiveInspectionStateV1::Ready,
                "Archive Inspection rejected the clean ZIP: {response:?}"
            );
            return response;
        }
        assert!(
            Instant::now() < deadline,
            "Archive Inspection did not reach Ready: {response:?}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn get_archive(
    router: &ArchiveInspectionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> GetArchiveInspectionResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        ATTACHMENT_ARCHIVE_INSPECTION_QUERY_CONNECT_PATH_V1,
        GetArchiveInspectionRequestV1 {
            protocol_major: 1,
            run_id: run_id.to_vec(),
        },
    )
}

fn archive_inspection_gateway(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: makosh_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> ArchiveInspectionGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("archive-inspection-gateway-cert.der"),
        root.join("archive-inspection-gateway-key.der"),
    )
    .expect("Gateway configuration");
    crate::platform::gateway::gateway_service(
        Arc::clone(store),
        data,
        supervisor.clone(),
        realtime,
        &configuration,
        None,
    )
    .expect("compose Archive Inspection Gateway routes")
}

fn post_proto<M, R>(
    router: &ArchiveInspectionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    message: M,
) -> R
where
    M: Message,
    R: Message + Default,
{
    let payload = message.encode_to_vec();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let response = runtime.block_on(
            router.route(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/connect+proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(Bytes::from(payload.clone())))
                    .expect("Archive Inspection Gateway request"),
            ),
        );
        let status = response.status();
        let bytes = runtime
            .block_on(response.into_body().collect())
            .expect("Archive Inspection Gateway response")
            .to_bytes();
        if status == StatusCode::SERVICE_UNAVAILABLE && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(
            status,
            StatusCode::OK,
            "Archive Inspection Gateway response body: {}",
            String::from_utf8_lossy(&bytes)
        );
        return R::decode(bytes.as_ref()).expect("decode Archive Inspection Gateway response");
    }
}

fn read_terminal_archive_sse_event(
    router: &ArchiveInspectionGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    run_id: &[u8],
) -> ClientRealtimeEventV1 {
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Archive Inspection Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(8),
            find_terminal_archive_event(response.into_body(), run_id),
        )
        .await
        .expect("Archive Inspection SSE timeout")
    })
}

async fn find_terminal_archive_event<B>(mut body: B, run_id: &[u8]) -> ClientRealtimeEventV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Archive Inspection SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Archive Inspection SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Archive Inspection frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Archive Inspection realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != ATTACHMENT_ARCHIVE_INSPECTION_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload = ArchiveInspectionStatusChangedV1::decode(event.payload.as_slice())
                .expect("Archive Inspection realtime payload");
            if payload.run_id == run_id
                && matches!(
                    state(payload.state),
                    ArchiveInspectionStateV1::Ready | ArchiveInspectionStateV1::Rejected
                )
            {
                return event;
            }
        }
    }
    panic!("Gateway SSE closed before terminal Archive Inspection event");
}

fn archive_diagnostics() -> ArchiveInspectionDiagnosticsV1 {
    tokio::runtime::Runtime::new()
        .expect("Archive Inspection diagnostics runtime")
        .block_on(async {
            let password = Zeroizing::new(
                std::fs::read_to_string(required(
                    "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
                ))
                .expect("read disposable PostgreSQL credential")
                .trim()
                .to_owned(),
            );
            let options = PgConnectOptions::new()
                .host(&required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
                .port(
                    required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                        .parse()
                        .expect("valid PostgreSQL port"),
                )
                .username("makosh_postgres_admin")
                .password(password.as_str())
                .database("makosh_storage_authenticated")
                .ssl_mode(PgSslMode::Disable);
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .expect("connect Archive Inspection diagnostics");
            let row = sqlx::query(
                "SELECT \
                 (SELECT count(*) FROM makosh_data.attachment_archive_inspection_scan_candidates) AS candidates, \
                 (SELECT count(*) FROM makosh_data.attachment_archive_inspection_safety_facts) AS safety_facts, \
                 (SELECT count(*) FROM makosh_data.attachment_archive_inspection_custody_delegation_requests) AS custody_requests, \
                 (SELECT count(*) FROM makosh_data.attachment_archive_inspection_custody_delegation_requests WHERE state = 2 AND published_at_unix_millis IS NULL) AS pending_custody_outbox, \
                 (SELECT count(*) FROM makosh_data.attachment_archive_inspection_custody_result_inbox) AS custody_results, \
                 (SELECT count(*) FROM makosh_data.attachment_archive_inspection_jobs) AS jobs, \
                 (SELECT coalesce(sum(attempt_count), 0) FROM makosh_data.attachment_archive_inspection_jobs) AS attempts, \
                 (SELECT count(*) FROM makosh_data.attachment_archive_inspection_reports) AS reports",
            )
            .fetch_one(&pool)
            .await
            .expect("read Archive Inspection diagnostics");
            ArchiveInspectionDiagnosticsV1 {
                candidates: row.try_get("candidates").expect("candidate count"),
                safety_facts: row.try_get("safety_facts").expect("safety count"),
                custody_requests: row.try_get("custody_requests").expect("custody count"),
                pending_custody_outbox: row
                    .try_get("pending_custody_outbox")
                    .expect("pending custody count"),
                custody_results: row.try_get("custody_results").expect("result count"),
                jobs: row.try_get("jobs").expect("job count"),
                attempts: row.try_get("attempts").expect("attempt count"),
                reports: row.try_get("reports").expect("report count"),
            }
        })
}

fn assert_private_archive_data_absent(
    bytes: &[u8],
    blob: &super::attachment_security_blob_fixture::AttachmentSecurityFixtureBlobV1,
) {
    for private in [
        PRIVATE_ARCHIVE_COMMENT,
        blob.reference_id.as_slice(),
        blob.receipt_sha256.as_slice(),
        blob.custody_transfer_source_proof.as_slice(),
    ] {
        assert!(
            !bytes.windows(private.len()).any(|window| window == private),
            "private archive source data crossed the client boundary"
        );
    }
}

fn state(value: i32) -> ArchiveInspectionStateV1 {
    ArchiveInspectionStateV1::try_from(value).expect("known Archive Inspection state")
}

fn error(value: i32) -> ArchiveInspectionErrorCodeV1 {
    ArchiveInspectionErrorCodeV1::try_from(value).expect("known Archive Inspection error")
}

fn unspecified_error() -> i32 {
    ArchiveInspectionErrorCodeV1::Unspecified as i32
}

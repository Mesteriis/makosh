//! Live event-only Communications source preparation for Reply Suggestion.

use super::*;

use futures_util::StreamExt;
use makosh_communications_ai_source_api::{
    CommunicationReplySourceEnvelopeContextV1,
    build_communication_reply_source_prepare_outbox_record_v1,
    communication_reply_source_prepared_contract_reference_v1,
    communication_reply_source_rejected_contract_reference_v1,
    wire::{
        CommunicationReplySourcePreparedV1, CommunicationReplySourceRejectCodeV1,
        CommunicationReplySourceRejectedV1,
    },
};
use makosh_events_jetstream::DurableSubjectV1;
use makosh_events_protocol::v1::DurableEnvelopeV1;
use prost::Message;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use zeroize::Zeroizing;

const RESULT_SUBJECT: &str = "makosh.result.v1.communications.>";
const SOURCE_BODY: &[u8] = b"fixture source body for custody transfer";
const SOURCE_SENDER: &[u8] = b"Alice Example <alice@example.test>";
const SOURCE_SUBJECT: &[u8] = b"Quarterly update";

#[test]
#[ignore = "requires disposable Docker plus real managed Vault, Storage, Blob, NATS and Communications binaries"]
fn managed_communications_ai_source_is_event_only_and_revision_fenced() {
    assert_eq!(
        std::env::var("MAKOSH_STORAGE_AUTHENTICATED_TEST").as_deref(),
        Ok("1")
    );
    let root = unique_target_root("makosh-managed-communications-ai-source");
    let data = private_directory(short_communications_kernel_data_directory());
    initialize_vault(
        &private_directory(data.join("vault")),
        &credential_directory(),
    );
    let release = installed_communications_release(&root);
    unsafe {
        std::env::set_var("MAKOSH_TEST_KERNEL_EXECUTABLE", release.kernel());
    }
    let store = Arc::new(configured_communications_store(&root, release.kernel()));
    store
        .claim_initial_owner(&makosh_kernel_control_store::InitialOwnerIdentity::new(
            "owner-1",
            "desktop-1",
            [4; 65],
        ))
        .expect("claim Communications logical owner");
    let _ = FileDeviceSigner::open_or_create_for_instance(&data).expect("Kernel signer");
    let shutdown = Arc::new(AtomicBool::new(false));
    let supervisor = ManagedRuntimeSupervisor::new(Arc::clone(&shutdown));
    configure_route_handler(&supervisor, &store, &data);
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
    configure_communications_jetstream(&store);
    start_communications_domain(&supervisor, &store, &root.join("runtime"));
    let source_message_id = assert_communications_transferred_body_projection(
        &store,
        &supervisor,
        &data,
        release.kernel(),
        &root.join("runtime"),
        false,
    );
    let source_message_id: [u8; 16] = source_message_id
        .try_into()
        .expect("canonical source message ID");
    let topology =
        crate::platform::storage::topology::current(&store).expect("read Storage topology");
    let vault =
        vault_status::read_current(&store, &supervisor.relay_port()).expect("read Vault status");
    let database_id = crate::platform::storage::topology::to_managed_runtime_configuration(
        &topology,
        &communications_storage_binding(&store),
        store.snapshot().instance_id(),
        vault.runtime_generation(),
        vault.hpke_public_key_x25519(),
    )
    .expect("build Communications Storage configuration")
    .database_id;
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();

    tokio::runtime::Runtime::new()
        .expect("AI source conformance runtime")
        .block_on(async {
            let client = async_nats::connect(&endpoint)
                .await
                .expect("connect AI source observer");
            let mut results = client
                .subscribe(RESULT_SUBJECT)
                .await
                .expect("subscribe AI source results");
            let context = async_nats::jetstream::new(client);

            let current = prepare_command([0x71; 16], source_message_id, 2);
            publish(&context, &current).await;
            let prepared = receive_result(&mut results).await;
            assert_eq!(
                prepared
                    .contract
                    .as_ref()
                    .map(|contract| contract.name.as_str()),
                Some(communication_reply_source_prepared_contract_reference_v1().name.as_str())
            );
            let payload =
                CommunicationReplySourcePreparedV1::decode(prepared.payload.as_slice())
                    .expect("decode prepared AI source");
            assert_eq!(payload.run_id, vec![0x71; 16]);
            assert_eq!(payload.source_message_id, source_message_id);
            assert_eq!(payload.source_evidence_revision, 2);
            let receipt = payload.source_content.expect("target-bound source receipt");
            assert_eq!(receipt.reference_id.len(), 16);
            assert_eq!(receipt.sha256.len(), 32);
            assert!(!receipt.custody_transfer_source_proof.is_empty());
            assert_private_content_absent(&prepared.encode_to_vec());

            publish(&context, &current).await;
            assert!(
                tokio::time::timeout(Duration::from_secs(1), results.next())
                    .await
                    .is_err(),
                "duplicate source command must not emit a second result"
            );

            mutate_source(&database_id, source_message_id, SourceMutationV1::Edit).await;
            let stale = prepare_command([0x72; 16], source_message_id, 2);
            publish(&context, &stale).await;
            assert_rejection(
                receive_result(&mut results).await,
                [0x72; 16],
                CommunicationReplySourceRejectCodeV1::CommunicationReplySourceRejectCodeStaleRevision,
            );

            mutate_source(&database_id, source_message_id, SourceMutationV1::Deactivate).await;
            let inactive = prepare_command([0x73; 16], source_message_id, 3);
            publish(&context, &inactive).await;
            assert_rejection(
                receive_result(&mut results).await,
                [0x73; 16],
                CommunicationReplySourceRejectCodeV1::CommunicationReplySourceRejectCodeSourceMissingOrInactive,
            );
        });

    supervisor.shutdown().expect("stop managed processes");
    shutdown.store(true, Ordering::SeqCst);
    unsafe {
        std::env::remove_var("MAKOSH_TEST_KERNEL_EXECUTABLE");
    }
    std::fs::remove_dir_all(root).expect("remove AI source fixture");
    std::fs::remove_dir_all(data).expect("remove short AI source Kernel fixture");
}

fn prepare_command(
    run_id: [u8; 16],
    source_message_id: [u8; 16],
    expected_source_revision: u64,
) -> makosh_events_protocol::delivery::OutboxRecordV1 {
    build_communication_reply_source_prepare_outbox_record_v1(
        run_id,
        source_message_id,
        expected_source_revision,
        "owner-1",
        1_800_000_030,
        &CommunicationReplySourceEnvelopeContextV1 {
            module_id: "makosh-communication-reply-suggestion-runtime".to_owned(),
            runtime_instance_id: "reply-suggestion-runtime-1".to_owned(),
            runtime_generation: 1,
            recorded_at_unix_seconds: 1_800_000_000,
            recorded_at_nanos: 0,
        },
    )
    .expect("build exact AI source command")
}

async fn publish(
    context: &async_nats::jetstream::Context,
    record: &makosh_events_protocol::delivery::OutboxRecordV1,
) {
    let envelope =
        DurableEnvelopeV1::decode(record.exact_bytes()).expect("decode AI source command");
    let subject = DurableSubjectV1::from_envelope(&envelope)
        .expect("derive AI source command subject")
        .as_str();
    context
        .publish(subject, record.exact_bytes().to_vec().into())
        .await
        .expect("publish AI source command")
        .await
        .expect("acknowledge AI source command");
}

async fn receive_result(subscriber: &mut async_nats::Subscriber) -> DurableEnvelopeV1 {
    let message = tokio::time::timeout(Duration::from_secs(10), subscriber.next())
        .await
        .expect("AI source result timeout")
        .expect("AI source result stream");
    DurableEnvelopeV1::decode(message.payload.as_ref()).expect("decode AI source result")
}

fn assert_rejection(
    envelope: DurableEnvelopeV1,
    run_id: [u8; 16],
    code: CommunicationReplySourceRejectCodeV1,
) {
    assert_eq!(
        envelope
            .contract
            .as_ref()
            .map(|contract| contract.name.as_str()),
        Some(
            communication_reply_source_rejected_contract_reference_v1()
                .name
                .as_str()
        )
    );
    let payload = CommunicationReplySourceRejectedV1::decode(envelope.payload.as_slice())
        .expect("decode rejected AI source");
    assert_eq!(payload.run_id, run_id);
    assert_eq!(payload.code, code as i32);
    assert_private_content_absent(&envelope.encode_to_vec());
}

fn assert_private_content_absent(bytes: &[u8]) {
    for private in [SOURCE_BODY, SOURCE_SENDER, SOURCE_SUBJECT] {
        assert!(
            !bytes.windows(private.len()).any(|window| window == private),
            "private source content must stay in target-bound Blob"
        );
    }
}

#[derive(Clone, Copy)]
enum SourceMutationV1 {
    Edit,
    Deactivate,
}

async fn mutate_source(database_id: &str, message_id: [u8; 16], mutation: SourceMutationV1) {
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
        .database(database_id)
        .ssl_mode(PgSslMode::Disable);
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect AI source fixture database");
    match mutation {
        SourceMutationV1::Edit => {
            sqlx::query(
                "UPDATE makosh_data.communications_messages
                 SET canonical_revision = canonical_revision + 1
                 WHERE message_id = $1",
            )
            .bind(message_id.as_slice())
            .execute(&pool)
            .await
            .expect("advance source revision");
        }
        SourceMutationV1::Deactivate => {
            sqlx::query(
                "UPDATE makosh_data.communications_messages
                 SET lifecycle_state = 2
                 WHERE message_id = $1",
            )
            .bind(message_id.as_slice())
            .execute(&pool)
            .await
            .expect("deactivate source message");
        }
    }
    pool.close().await;
}

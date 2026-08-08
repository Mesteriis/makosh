//! Telegram-owned outbox to Communications inbox conformance.

use std::future::{Future, ready};

use makosh_events_jetstream::{
    DurableSubjectV1, JetStreamClient, NatsPasswordCredentialV1, RuntimeNatsIdentity,
    RuntimeOutboxPublisherV1, RuntimePublishPermitV1,
};
use makosh_events_protocol::{
    delivery::{
        ExactOutboxPublisherPortV1, OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1,
        OutboxRelayOutcomeV1, relay_once,
    },
    validation::envelope::decode_envelope_v1,
};
use makosh_telegram_api::{
    TelegramMessageObservation, TelegramMessageReferences, TelegramProviderEvent,
    TelegramRealtimeFrame,
};
use makosh_telegram_persistence::{
    TelegramCommunicationsOutboxStoreV1, TelegramDurablePersistence,
    TelegramPersistenceConformanceV1,
};
use makosh_telegram_runtime::{PACKAGE as TELEGRAM_MODULE_ID, TelegramRuntime};
use makosh_telegram_tdlib::{
    TdlibError, TdlibProviderUpdate, TdlibRequest, TdlibResponse, TdlibTransport,
};
use zeroize::Zeroizing;

use super::*;

const TELEGRAM_RUNTIME_ID: &str = "telegram-runtime-outbox-test";
const TELEGRAM_REGISTRATION_ID: &str = "telegram-runtime-conformance";

pub(super) fn assert_telegram_outbox_delivery(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    tokio::runtime::Runtime::new()
        .expect("Tokio runtime")
        .block_on(assert_live_flow(endpoint, supervisor));
}

async fn assert_live_flow(endpoint: String, supervisor: &ManagedRuntimeSupervisor) {
    let durable = connect_postgres().await;
    durable
        .initialize()
        .await
        .expect("initialize Telegram-owned persistence");
    persist_provider_observation(&durable).await;

    let pending = durable
        .pending_communications_outbox(2)
        .await
        .expect("read Telegram outbox");
    assert_eq!(pending.len(), 1);
    let record = pending[0].clone();
    let envelope = decode_envelope_v1(record.exact_bytes()).expect("Telegram durable envelope");
    let source = envelope.source.as_ref().expect("Telegram envelope source");
    assert_eq!(source.module_id, TELEGRAM_MODULE_ID);
    assert_eq!(source.runtime_generation, 1);

    let mut outage_store = TelegramCommunicationsOutboxStoreV1::new(&durable, 1_783_024_001);
    assert_eq!(
        relay_once(&mut outage_store, &UnavailablePublisher).await,
        Err(OutboxRelayErrorV1::PublisherUnavailable)
    );
    assert_eq!(
        durable
            .pending_communications_outbox(2)
            .await
            .expect("read replayable Telegram outbox"),
        vec![record.clone()],
        "publisher outage must leave the exact envelope pending"
    );

    let client = async_nats::connect(&endpoint)
        .await
        .expect("connect canonical event observer");
    let mut canonical_events = client
        .subscribe("makosh.event.v1.communications.communication_evidence_recorded.v1")
        .await
        .expect("subscribe to canonical Communications events");
    let identity =
        RuntimeNatsIdentity::new(TELEGRAM_RUNTIME_ID, 1, 1).expect("Telegram runtime identity");
    let subject = DurableSubjectV1::from_envelope(&envelope).expect("Telegram observation subject");
    let permit = RuntimePublishPermitV1::new(
        TELEGRAM_REGISTRATION_ID,
        TELEGRAM_RUNTIME_ID,
        1,
        1,
        vec![subject],
    )
    .expect("Telegram exact publish permit");
    let credential = NatsPasswordCredentialV1::new("telegram-test", "telegram-test")
        .expect("bounded test credential");
    let connection = JetStreamClient::connect_runtime(&endpoint, identity, credential)
        .await
        .expect("connect Telegram runtime to disposable JetStream");
    let publisher = RuntimeOutboxPublisherV1::new(&connection, &permit);
    let mut live_store = TelegramCommunicationsOutboxStoreV1::new(&durable, 1_783_024_002);

    assert!(matches!(
        relay_once(&mut live_store, &publisher)
            .await
            .expect("relay Telegram outbox"),
        OutboxRelayOutcomeV1::Published {
            duplicate: false,
            ..
        }
    ));
    let canonical =
        tokio::time::timeout(std::time::Duration::from_secs(5), canonical_events.next())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Telegram canonical event timeout: active={:?}, failure={:?}",
                    supervisor.is_active(COMMUNICATIONS_REGISTRATION),
                    supervisor.last_failure(COMMUNICATIONS_REGISTRATION),
                )
            })
            .expect("Telegram canonical Communications event missing");
    let canonical_envelope =
        decode_envelope_v1(canonical.payload.as_ref()).expect("canonical Communications envelope");
    assert_eq!(
        canonical_envelope.causation_message_id,
        record.message_id().to_vec()
    );
    assert!(
        durable
            .pending_communications_outbox(2)
            .await
            .expect("read published Telegram outbox")
            .is_empty()
    );

    assert!(
        TelegramPersistenceConformanceV1::reopen_publish_before_mark_window(
            &durable,
            record.message_id(),
        )
        .await
        .expect("simulate publish-before-mark crash window")
    );
    let mut replay_store = TelegramCommunicationsOutboxStoreV1::new(&durable, 1_783_024_003);
    assert!(matches!(
        relay_once(&mut replay_store, &publisher)
            .await
            .expect("replay Telegram outbox"),
        OutboxRelayOutcomeV1::Published {
            duplicate: true,
            ..
        }
    ));
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), canonical_events.next(),)
            .await
            .is_err(),
        "duplicate Telegram delivery must not produce a second canonical event"
    );
}

async fn persist_provider_observation(durable: &TelegramDurablePersistence) {
    let mut runtime = TelegramRuntime::new(UnusedProvider);
    runtime.set_admission(Some(makosh_telegram_runtime::TelegramRuntimeAdmission {
        logical_owner_id: "telegram".to_owned(),
        logical_human_owner_id: "owner-1".to_owned(),
        configuration_instance_id: "telegram-account-1".to_owned(),
        module_registration_id: TELEGRAM_REGISTRATION_ID.to_owned(),
        runtime_instance_id: TELEGRAM_RUNTIME_ID.to_owned(),
        runtime_generation: 1,
        grant_epoch: 1,
        vault_runtime_generation: 1,
        api_hash_revision: 1,
        session_encryption_key_revision: 1,
    }));
    let frame = TelegramRealtimeFrame {
        account_id: "telegram-account-1".to_owned(),
        sequence: 1,
        provider_cursor: Some("telegram-cursor-1".to_owned()),
        event: TelegramProviderEvent::MessageCreated(TelegramMessageObservation {
            account_id: "telegram-account-1".to_owned(),
            provider_chat_id: "telegram-chat-1".to_owned(),
            provider_message_id: "telegram-message-1".to_owned(),
            provider_topic_id: None,
            sender_id: "telegram-sender-1".to_owned(),
            sender_display_name: None,
            is_outgoing: false,
            text: None,
            media: None,
            references: TelegramMessageReferences::default(),
            observed_at_unix_seconds: 1_783_024_000,
        }),
    };
    runtime
        .persist_provider_frame_durable(durable, &frame, &mut |_| {
            Err(makosh_communications_ingress::BodyAdmissionFailureV1::PolicyRejected)
        })
        .await
        .expect("persist Telegram provider observation and outbox");
}

async fn connect_postgres() -> TelegramDurablePersistence {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let port = required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_PORT")
        .parse::<u16>()
        .expect("valid PostgreSQL port");
    TelegramPersistenceConformanceV1::connect(
        &required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"),
        port,
        "makosh_postgres_admin",
        password.as_str(),
        "makosh_storage_authenticated",
    )
    .await
    .expect("connect disposable PostgreSQL")
}

struct UnavailablePublisher;

impl ExactOutboxPublisherPortV1 for UnavailablePublisher {
    fn publish_exact(
        &self,
        _record: &OutboxRecordV1,
    ) -> impl Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send {
        ready(Err(OutboxRelayErrorV1::PublisherUnavailable))
    }
}

struct UnusedProvider;

impl TdlibTransport for UnusedProvider {
    fn request(&mut self, _request: TdlibRequest) -> Result<TdlibResponse, TdlibError> {
        Err(TdlibError::Transport(
            "provider request is outside event-flow conformance".to_owned(),
        ))
    }

    fn poll_updates(&mut self) -> Result<Vec<TdlibProviderUpdate>, TdlibError> {
        Ok(Vec::new())
    }
}

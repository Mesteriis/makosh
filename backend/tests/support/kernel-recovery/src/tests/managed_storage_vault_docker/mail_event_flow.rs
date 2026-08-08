//! Managed Mail-owned outbox to Communications event handoff conformance.

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use makosh_communications_ingress::{
    BodyAvailabilityV1, CommunicationDirectionV1, CommunicationEvidenceKindV1,
    ObservationEnvelopeContextV1, ProviderProvenanceV1, SourceEnvelope, SourceScopeEnvelope,
    build_observation_outbox_record_v1, new_scoped_communication_observation_draft,
};
use makosh_events_protocol::{delivery::OutboxRecordV1, validation::envelope::decode_envelope_v1};
use makosh_mail_persistence::{MailDurablePersistence, MailPersistenceConformanceV1};
use makosh_mail_runtime::admission::MAIL_MODULE_ID;
use zeroize::Zeroizing;

use super::*;

pub(super) fn assert_mail_event_only_communications_handoff(
    store: &SqliteControlStore,
    supervisor: &ManagedRuntimeSupervisor,
    mail: &StartedMailRuntime,
) {
    let endpoint = store
        .platform_event_hub_topology()
        .expect("read Event Hub topology")
        .expect("Event Hub topology")
        .nats_endpoint()
        .to_owned();
    let runtime = tokio::runtime::Runtime::new().expect("Mail event observer runtime");
    let _runtime_context = runtime.enter();
    let durable = runtime.block_on(connect_postgres());
    runtime
        .block_on(durable.initialize())
        .expect("initialize Mail-owned persistence");
    let (client, mut observations, mut canonical_events) = runtime.block_on(async {
        let client = async_nats::connect(endpoint)
            .await
            .expect("connect Mail event observer");
        let observations = client
            .subscribe("makosh.observation.v1.communications.communication_observed.v1")
            .await
            .expect("subscribe Mail observations");
        let canonical_events = client
            .subscribe("makosh.event.v1.communications.communication_evidence_recorded.v1")
            .await
            .expect("subscribe canonical Communications events");
        client.flush().await.expect("activate Mail event observers");
        (client, observations, canonical_events)
    });

    let initial = mail_observation(mail, "managed-mail-observation-1", 1_783_024_100);
    runtime
        .block_on(durable.enqueue_communications_outbox(&initial, 1_783_024_100))
        .expect("enqueue first Mail observation");
    let observation = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .ok()
            .flatten()
    });
    let observation = observation.unwrap_or_else(|| {
        let pending = runtime
            .block_on(durable.pending_communications_outbox(4))
            .expect("read Mail outbox after observation timeout");
        let active = supervisor
            .is_active(&mail.registration_id)
            .expect("read managed Mail state after observation timeout");
        let failure = supervisor
            .last_failure(&mail.registration_id)
            .expect("read managed Mail failure after observation timeout");
        panic!(
            "managed Mail observation timeout: active={active}, failure={failure:?}, pending={}",
            pending.len()
        );
    });
    let canonical = runtime
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(10), canonical_events.next()).await
        })
        .expect("canonical Communications event timeout")
        .expect("canonical Communications event");
    let observation_bytes = observation.payload.to_vec();
    let observation =
        decode_envelope_v1(&observation_bytes).expect("Mail observation durable envelope");
    assert_eq!(
        observation
            .source
            .as_ref()
            .expect("Mail observation source")
            .module_id,
        MAIL_MODULE_ID
    );
    assert_eq!(
        observation
            .source
            .as_ref()
            .expect("Mail observation source")
            .runtime_generation,
        mail.runtime_generation
    );
    let canonical =
        decode_envelope_v1(canonical.payload.as_ref()).expect("Communications event envelope");
    assert_eq!(
        canonical.causation_message_id, observation.message_id,
        "Communications must derive canonical evidence only from the typed Mail observation"
    );

    runtime.block_on(async {
        client
            .publish(
                "makosh.observation.v1.communications.communication_observed.v1",
                observation_bytes.into(),
            )
            .await
            .expect("republish exact Mail observation");
        client
            .flush()
            .await
            .expect("flush duplicate Mail observation");
        let duplicate = tokio::time::timeout(Duration::from_secs(1), observations.next())
            .await
            .expect("duplicate Mail observation timeout")
            .expect("duplicate Mail observation");
        let duplicate = decode_envelope_v1(duplicate.payload.as_ref())
            .expect("duplicate Mail observation envelope");
        assert_eq!(duplicate.message_id, observation.message_id);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), canonical_events.next())
                .await
                .is_err(),
            "duplicate Mail observation must not create a second Communications event"
        );
    });
    let initial_evidence_id = assert_communications_query_delivery(store, supervisor);

    set_authenticated_nats_container_running(false);
    let replay = mail_observation(mail, "managed-mail-observation-2", 1_783_024_101);
    runtime
        .block_on(durable.enqueue_communications_outbox(&replay, 1_783_024_101))
        .expect("enqueue Mail observation during NATS outage");
    std::thread::sleep(Duration::from_millis(2_500));
    let pending = runtime
        .block_on(durable.pending_communications_outbox(4))
        .expect("read replayable Mail outbox");
    assert_eq!(
        pending,
        vec![replay.clone()],
        "NATS outage must leave the exact Mail envelope pending"
    );
    assert!(
        supervisor
            .is_active(&mail.registration_id)
            .expect("read managed Mail state"),
        "managed Mail runtime must remain active while NATS is unavailable"
    );
    assert_eq!(
        supervisor
            .last_failure(&mail.registration_id)
            .expect("read managed Mail failure"),
        None
    );
    set_authenticated_nats_container_running(true);
    wait_for_authenticated_nats_reconnect(&runtime, &client, "Mail event observer");

    let (replayed_observation, replayed_canonical) = runtime.block_on(async {
        let observation = tokio::time::timeout(Duration::from_secs(10), observations.next())
            .await
            .expect("replayed Mail observation timeout")
            .expect("replayed Mail observation");
        let canonical = tokio::time::timeout(Duration::from_secs(10), canonical_events.next())
            .await
            .expect("replayed Communications event timeout")
            .expect("replayed Communications event");
        (observation, canonical)
    });
    let replayed_observation = decode_envelope_v1(replayed_observation.payload.as_ref())
        .expect("replayed Mail observation envelope");
    let replayed_canonical = decode_envelope_v1(replayed_canonical.payload.as_ref())
        .expect("replayed Communications event envelope");
    assert_eq!(
        replayed_observation.message_id,
        replay.message_id().as_slice()
    );
    assert_eq!(
        replayed_canonical.causation_message_id, replayed_observation.message_id,
        "Communications replay must retain typed Mail causation"
    );
    assert_ne!(
        replayed_canonical.message_id, canonical.message_id,
        "the outage replay must deliver the second Mail observation"
    );
    let replayed_evidence_id = assert_communications_query_delivery(store, supervisor);
    assert_ne!(
        replayed_evidence_id, initial_evidence_id,
        "Communications durable query must expose the replayed Mail evidence"
    );
}

fn mail_observation(
    mail: &StartedMailRuntime,
    observation_id: &str,
    recorded_at_unix_seconds: i64,
) -> OutboxRecordV1 {
    let draft = new_scoped_communication_observation_draft(
        observation_id,
        SourceEnvelope {
            provider: ProviderProvenanceV1::MailImap,
            external_record_id: observation_id.to_owned(),
            scope: Some(SourceScopeEnvelope {
                external_account_id: MAIL_ACCOUNT_ID.to_owned(),
                external_conversation_id: Some("managed-mail-conversation-1".to_owned()),
                external_participant_id: None,
                external_media_id: None,
                external_reply_to_record_id: None,
                external_forward_origin_record_id: None,
            }),
        },
        CommunicationEvidenceKindV1::EmailMessage,
        BodyAvailabilityV1::MetadataOnly,
        CommunicationDirectionV1::Incoming,
        Some(recorded_at_unix_seconds),
    )
    .expect("build Mail observation draft");
    build_observation_outbox_record_v1(
        &draft,
        &ObservationEnvelopeContextV1 {
            runtime_instance_id: mail.runtime_instance_id.clone(),
            runtime_generation: mail.runtime_generation,
            module_id: MAIL_MODULE_ID.to_owned(),
            recorded_at_unix_seconds,
            recorded_at_nanos: 0,
        },
    )
    .expect("build exact Mail observation envelope")
}

pub(super) async fn connect_postgres() -> MailDurablePersistence {
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
    MailPersistenceConformanceV1::connect(
        &required("MAKOSH_STORAGE_AUTHENTICATED_POSTGRES_HOST"),
        port,
        "makosh_postgres_admin",
        password.as_str(),
        "makosh_storage_authenticated",
    )
    .await
    .expect("connect disposable PostgreSQL")
}

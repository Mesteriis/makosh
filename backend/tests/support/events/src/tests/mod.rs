mod authority;
mod authority_runtime;
mod jetstream_live;
mod jwt_credentials;
mod jwt_live;
mod jwt_revocation_live;
mod reconciliation;
mod resolver_update;
mod resolver_update_live;
mod scaffolds;
mod support;
mod vault_credentials;

use std::time::Duration;

use makosh_events_jetstream::{
    ConsumerBudgetV1, ConsumerSpecV1, DurableSubjectV1, NatsPasswordCredentialV1,
    RuntimeNatsIdentity, RuntimePublishPermitV1, RuntimeSubscribePermitV1, StreamBudgetV1,
    StreamKindV1, canonical_message_id,
};
use makosh_events_protocol::delivery::{
    ExactOutboxPublisherPortV1, InboxDecisionV1, InboxRecordV1, OutboxEntryV1,
    OutboxPublishReceiptV1, OutboxRecordV1, OutboxRelayErrorV1, OutboxRelayOutcomeV1,
    OwnerOutboxStorePortV1, relay_once,
};
use prost::Message;

#[test]
fn renders_the_exact_versioned_subject_grammar() {
    let subject =
        DurableSubjectV1::new(StreamKindV1::Observation, "mail", "received", 2).expect("subject");

    assert_eq!(subject.as_str(), "makosh.observation.v1.mail.received.v2");
}

#[test]
fn rejects_wildcards_and_private_identifier_shapes() {
    for invalid in ["mail.*", "mail.>", "Jane@example.com", "personal chat"] {
        assert!(DurableSubjectV1::new(StreamKindV1::Event, invalid, "received", 1).is_err());
    }
}

#[test]
fn retains_only_bounded_single_replica_profiles() {
    assert!(StreamBudgetV1::new(1024, Duration::from_secs(60), 1).is_ok());
    assert!(StreamBudgetV1::new(0, Duration::from_secs(60), 1).is_err());
    assert!(StreamBudgetV1::new(1024, Duration::ZERO, 1).is_err());
    assert!(StreamBudgetV1::new(1024, Duration::from_secs(60), 2).is_err());
}

#[test]
fn forbids_wildcard_module_consumers() {
    let budget = ConsumerBudgetV1::new(16, 3, Duration::from_secs(30)).expect("budget");
    assert!(
        ConsumerSpecV1::new(
            StreamKindV1::Event,
            "notes_projection",
            "makosh.event.v1.notes.changed.v1",
            budget,
        )
        .is_ok()
    );
    assert!(
        ConsumerSpecV1::new(
            StreamKindV1::Event,
            "notes_projection",
            "makosh.event.v1.>",
            budget,
        )
        .is_err()
    );
}

#[test]
fn keeps_password_credentials_out_of_diagnostics_and_runtime_identity_fenced() {
    let credential = NatsPasswordCredentialV1::new("runtime_user", "secret").expect("credential");
    assert!(!format!("{credential:?}").contains("secret"));
    assert!(RuntimeNatsIdentity::new("runtime", 1, 1).is_ok());
    assert!(RuntimeNatsIdentity::new("runtime", 0, 1).is_err());
}

#[test]
fn fences_publish_permits_to_the_exact_runtime_generation_and_grant_epoch() {
    let subject =
        DurableSubjectV1::new(StreamKindV1::Event, "notes", "changed", 1).expect("subject");
    assert!(RuntimePublishPermitV1::new("registration", "runtime", 1, 1, vec![subject]).is_ok());
    assert!(RuntimePublishPermitV1::new("registration", "runtime", 0, 1, Vec::new()).is_err());
}

#[test]
fn fences_subscribe_permits_to_the_exact_runtime_generation_and_grant_epoch() {
    let consumer = ConsumerSpecV1::new(
        StreamKindV1::Event,
        "notes_projection",
        "makosh.event.v1.notes.changed.v1",
        ConsumerBudgetV1::new(16, 3, Duration::from_secs(30)).expect("budget"),
    )
    .expect("consumer");
    assert!(RuntimeSubscribePermitV1::new("registration", "runtime", 1, 1, consumer).is_ok());
    let invalid = ConsumerSpecV1::new(
        StreamKindV1::Event,
        "notes_projection",
        "makosh.event.v1.notes.changed.v1",
        ConsumerBudgetV1::new(16, 3, Duration::from_secs(30)).expect("budget"),
    )
    .expect("consumer");
    assert!(RuntimeSubscribePermitV1::new("registration", "runtime", 0, 1, invalid).is_err());
}

#[test]
fn outbox_keeps_exact_bytes_and_inbox_rejects_same_id_hash_conflicts() {
    let accepted =
        OutboxRecordV1::accept(jetstream_live::event_envelope("changed").encode_to_vec())
            .expect("accept valid outbox envelope");
    let inbox = InboxRecordV1::from_outbox(&accepted);
    let retry =
        OutboxRecordV1::accept(accepted.exact_bytes().to_vec()).expect("accept exact retry");
    let conflict = OutboxRecordV1::accept(jetstream_live::event_envelope("other").encode_to_vec())
        .expect("accept distinct bytes with the same fixture message ID");

    assert_eq!(inbox.classify(&retry), InboxDecisionV1::Duplicate);
    assert_eq!(inbox.classify(&conflict), InboxDecisionV1::HashConflict);
}

#[test]
fn formats_the_exact_nats_message_id_value() {
    assert_eq!(
        canonical_message_id(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
        "00010203-0405-0607-0809-0a0b0c0d0e0f",
    );
    assert!(canonical_message_id(&[0; 15]).is_empty());
}

#[tokio::test]
async fn outbox_relay_marks_owner_state_only_after_exact_publish_acknowledgement() {
    let record = OutboxRecordV1::accept(jetstream_live::event_envelope("changed").encode_to_vec())
        .expect("outbox record");
    let entry = OutboxEntryV1::new("outbox_1", record).expect("outbox entry");
    let publisher = ScriptedPublisher { fail: false };
    let mut store = ScriptedOutbox::new(entry.clone());

    assert_eq!(
        relay_once(&mut store, &publisher).await,
        Ok(OutboxRelayOutcomeV1::Published {
            outbox_id: "outbox_1".to_owned(),
            duplicate: false,
        })
    );
    assert_eq!(store.marked.as_deref(), Some("outbox_1"));

    let mut unavailable = ScriptedOutbox::new(entry);
    assert_eq!(
        relay_once(&mut unavailable, &ScriptedPublisher { fail: true }).await,
        Err(OutboxRelayErrorV1::PublisherUnavailable)
    );
    assert!(unavailable.marked.is_none());
}

struct ScriptedOutbox {
    pending: Option<OutboxEntryV1>,
    marked: Option<String>,
}

impl ScriptedOutbox {
    fn new(entry: OutboxEntryV1) -> Self {
        Self {
            pending: Some(entry),
            marked: None,
        }
    }
}

impl OwnerOutboxStorePortV1 for ScriptedOutbox {
    fn next_pending(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<OutboxEntryV1>, OutboxRelayErrorV1>> + Send
    {
        std::future::ready(Ok(self.pending.take()))
    }

    fn mark_published(
        &mut self,
        entry: &OutboxEntryV1,
        _receipt: &OutboxPublishReceiptV1,
    ) -> impl std::future::Future<Output = Result<(), OutboxRelayErrorV1>> + Send {
        self.marked = Some(entry.outbox_id().to_owned());
        std::future::ready(Ok(()))
    }
}

struct ScriptedPublisher {
    fail: bool,
}

impl ExactOutboxPublisherPortV1 for ScriptedPublisher {
    fn publish_exact(
        &self,
        _record: &OutboxRecordV1,
    ) -> impl std::future::Future<Output = Result<OutboxPublishReceiptV1, OutboxRelayErrorV1>> + Send
    {
        let result = if self.fail {
            Err(OutboxRelayErrorV1::PublisherUnavailable)
        } else {
            OutboxPublishReceiptV1::new("MAKOSH_EVENT_V1", 1, false)
        };
        std::future::ready(result)
    }
}

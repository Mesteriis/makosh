use makosh_communications_ingress::ProviderProvenanceV1;
use makosh_mail_api::{
    DEFAULT_WINDOW, IMAP_PORT, MAX_MESSAGE_BYTES, MAX_PLAIN_TEXT_BYTES, MAX_WINDOW, MAX_WINDOWS,
};
use makosh_mail_core::{
    ConnectionTracker, MailConnection, MailConnectionState, MailOperation, MailStatePolicy,
    bounded_window, draft_ingress_observation, validate_sync_request,
};

#[test]
fn sync_plan_bounds() {
    assert!(bounded_window(DEFAULT_WINDOW, 1).is_ok());
    assert!(bounded_window(DEFAULT_WINDOW, MAX_WINDOWS).is_ok());
    assert!(bounded_window(MAX_WINDOW, 1).is_ok());
    assert!(bounded_window(MAX_WINDOW + 1, 1).is_err());
    assert!(bounded_window(DEFAULT_WINDOW, MAX_WINDOWS + 1).is_err());
    assert!(bounded_window(0, 1).is_err());
}

#[test]
fn tracker_updates_state() {
    let mut tracker = ConnectionTracker::new();
    let connection = MailConnection {
        id: "conn-1".to_owned(),
        host: "mail.example.com".to_owned(),
        port: IMAP_PORT,
        username: "user".to_owned(),
        state: MailConnectionState::Provisioning,
        operation_id: None,
    };
    tracker.register_connection(&connection);
    assert_eq!(
        tracker.status_of("conn-1"),
        Some(MailConnectionState::Provisioning)
    );

    let operation = MailOperation {
        operation_id: "op-1".to_owned(),
        state: MailConnectionState::Syncing,
        window_size: DEFAULT_WINDOW,
    };
    tracker.set_syncing("conn-1", operation.clone());
    assert_eq!(
        tracker.status_of("conn-1"),
        Some(MailConnectionState::Syncing)
    );
    assert_eq!(tracker.operation_status("op-1"), Some(&operation));

    tracker.set_ready("conn-1");
    assert_eq!(
        tracker.status_of("conn-1"),
        Some(MailConnectionState::Ready)
    );

    tracker.set_retired("conn-1");
    assert_eq!(
        tracker.status_of("conn-1"),
        Some(MailConnectionState::Retired)
    );
}

#[test]
fn sync_request_validation() {
    assert!(validate_sync_request("mail.example.com", IMAP_PORT, 128).is_ok());
    assert!(validate_sync_request("bad host", IMAP_PORT, 128).is_err());
    assert!(validate_sync_request("mail.example.com", IMAP_PORT, MAX_MESSAGE_BYTES + 1).is_err());
    assert!(
        validate_sync_request("mail.example.com", IMAP_PORT, MAX_PLAIN_TEXT_BYTES + 1).is_err()
    );
}

#[test]
fn ingress_observation_validation() {
    let draft = draft_ingress_observation(
        "op-1",
        ProviderProvenanceV1::MailImap,
        "account-1",
        "source",
        200,
    )
    .unwrap();
    assert_eq!(draft.observation_id, "op-1");
    assert_eq!(draft.source.external_record_id, "source");
    assert_eq!(
        draft.body,
        makosh_communications_ingress::BodyAvailabilityV1::Unavailable
    );

    assert!(
        draft_ingress_observation(
            "op-2",
            ProviderProvenanceV1::MailImap,
            "account-1",
            "source",
            MAX_PLAIN_TEXT_BYTES + 1,
        )
        .is_err()
    );
}

#[test]
fn policy_defaults_are_stable() {
    let policy = MailStatePolicy::new();
    assert_eq!(policy.max_sync_windows, MAX_WINDOWS);
}

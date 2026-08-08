use makosh_provider_mail::gmail::{GmailHistoryResponse, GooglePeopleConnectionsResponse};
use serde_json::json;

#[test]
fn parses_gmail_history_wire_payload() {
    let response = serde_json::from_value::<GmailHistoryResponse>(json!({
        "historyId": "812",
        "nextPageToken": "page-2",
        "history": [{
            "messagesAdded": [{"message": {"id": "message-1"}}],
            "labelsRemoved": [{"message": {"id": "message-2"}}]
        }]
    }))
    .expect("Gmail history payload");

    assert_eq!(response.history_id.as_deref(), Some("812"));
    assert_eq!(response.next_page_token.as_deref(), Some("page-2"));
    let history = response.history.expect("history");
    assert_eq!(
        history[0].messages_added.as_ref().expect("added")[0]
            .message
            .id,
        "message-1"
    );
    assert_eq!(
        history[0].labels_removed.as_ref().expect("removed")[0]
            .message
            .id,
        "message-2"
    );
}

#[test]
fn parses_google_people_provider_payload() {
    let response = serde_json::from_value::<GooglePeopleConnectionsResponse>(json!({
        "nextPageToken": "next",
        "connections": [{
            "resourceName": "people/1", "etag": "etag-1",
            "names": [{"displayName": "Макошь Person"}],
            "emailAddresses": [{"value": "person@example.test"}],
            "phoneNumbers": [{"value": "+1 555 0100"}]
        }]
    }))
    .expect("Google People payload");

    assert_eq!(response.next_page_token.as_deref(), Some("next"));
    let person = &response.connections.expect("connections")[0];
    assert_eq!(person.resource_name.as_deref(), Some("people/1"));
    assert_eq!(
        person.names.as_ref().expect("names")[0]
            .display_name
            .as_deref(),
        Some("Макошь Person")
    );
    assert_eq!(
        person.email_addresses.as_ref().expect("emails")[0]
            .value
            .as_deref(),
        Some("person@example.test")
    );
    assert_eq!(
        person.phone_numbers.as_ref().expect("phones")[0]
            .value
            .as_deref(),
        Some("+1 555 0100")
    );
}

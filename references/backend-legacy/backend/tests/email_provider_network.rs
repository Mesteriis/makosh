use makosh_backend_testkit::context::TestContext;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use serde_json::json;
use sqlx::Row;

use makosh_communications_api::accounts::{
    CommunicationProviderKind, CommunicationProviderKind as PlatformEmailProviderKind,
    NewProviderAccount,
};
use makosh_communications_api::address_book::AddressBookProviderUpsertRequest;
use makosh_communications_api::email_sync::{EmailSyncBatch, FetchedCommunicationSourceMessage};
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::communications::storage::port::{
    CommunicationAttachmentPort, LocalBlobPort,
};
use makosh_hub_backend::integrations::mail::gmail::client::{
    gmail_api::GmailApiClient,
    imap::ImapNetworkClient,
    options::{GmailContactFetchOptions, GmailFetchOptions, ImapFetchOptions},
};
use makosh_hub_backend::platform::secrets::models::ResolvedSecret;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::email_sync_pipeline::recording::{
    record_email_sync_batch, record_email_sync_batch_with_mail_blobs,
};

#[tokio::test]
async fn gmail_api_client_fetches_raw_messages_with_bearer_token() {
    let server = MockGmailServer::start();
    let token = ResolvedSecret::new("gmail-access-token").expect("token");
    let client = GmailApiClient::new(server.base_url()).user_id("me");

    let batch = client
        .fetch_raw_messages(&token, &GmailFetchOptions::new(2).query("is:unread"))
        .await
        .expect("fetch gmail messages");

    assert_eq!(batch.provider_kind, CommunicationProviderKind::Gmail);
    assert_eq!(batch.stream_id, "gmail:history");
    assert_eq!(
        batch.checkpoint,
        Some(json!({
            "provider": "gmail",
            "history_id": "12345",
            "next_page_token": "next-page",
            "page_kind": "messages"
        }))
    );
    assert_eq!(batch.messages.len(), 1);

    let message = &batch.messages[0];
    assert_eq!(message.provider_record_id, "gmail-msg-1");
    assert_eq!(
        message.occurred_at,
        Utc.timestamp_millis_opt(1_770_000_000_000).single()
    );
    assert!(message.source_fingerprint.starts_with("sha256:"));
    assert_eq!(message.payload["provider"], "gmail");
    assert_eq!(message.payload["thread_id"], "thread-1");
    assert_eq!(
        message.payload["raw_base64url"],
        "U3ViamVjdDogR21haWwNCg0KSGVsbG8"
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .request_line
            .starts_with("GET /gmail/v1/users/me/messages?")
    );
    assert!(requests[0].request_line.contains("maxResults=2"));
    assert!(requests[0].request_line.contains("includeSpamTrash=true"));
    assert!(requests[0].request_line.contains("q=is%3Aunread"));
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer gmail-access-token")
    );
    assert_eq!(
        requests[1].request_line,
        "GET /gmail/v1/users/me/messages/gmail-msg-1?format=raw HTTP/1.1"
    );
    assert_eq!(
        requests[1].authorization.as_deref(),
        Some("Bearer gmail-access-token")
    );
}

#[tokio::test]
async fn gmail_api_client_modifies_single_and_batched_message_labels() {
    let server = MockPeopleServer::start(vec!["{}".to_owned(), "{}".to_owned()]);
    let token = ResolvedSecret::new("gmail-modify-token").expect("token");
    let client = GmailApiClient::new(server.base_url()).user_id("me");

    client
        .modify_message(&token, "gmail-msg-1", &["IMPORTANT"], &["UNREAD"])
        .await
        .expect("modify Gmail message");
    client
        .batch_modify_messages(
            &token,
            &["gmail-msg-1".to_owned(), "gmail-msg-2".to_owned()],
            &["STARRED"],
            &["INBOX"],
        )
        .await
        .expect("batch modify Gmail messages");

    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0].request_line,
        "POST /gmail/v1/users/me/messages/gmail-msg-1/modify HTTP/1.1"
    );
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer gmail-modify-token")
    );
    let single_body: serde_json::Value =
        serde_json::from_str(&requests[0].body).expect("single modify body");
    assert_eq!(single_body["addLabelIds"], json!(["IMPORTANT"]));
    assert_eq!(single_body["removeLabelIds"], json!(["UNREAD"]));

    assert_eq!(
        requests[1].request_line,
        "POST /gmail/v1/users/me/messages/batchModify HTTP/1.1"
    );
    let batch_body: serde_json::Value =
        serde_json::from_str(&requests[1].body).expect("batch modify body");
    assert_eq!(batch_body["ids"], json!(["gmail-msg-1", "gmail-msg-2"]));
    assert_eq!(batch_body["addLabelIds"], json!(["STARRED"]));
    assert_eq!(batch_body["removeLabelIds"], json!(["INBOX"]));
}

#[tokio::test]
async fn imap_network_client_fetches_raw_messages_by_uid_without_mutating_mailbox() {
    let server = MockImapServer::start();
    let password = ResolvedSecret::new("imap-password").expect("password");
    let client = ImapNetworkClient::new();
    let options = ImapFetchOptions::new(
        "127.0.0.1",
        server.addr().port(),
        false,
        "Archive",
        "imap-user@example.net",
    )
    .last_seen_uid(42)
    .max_messages(10);

    let batch = client
        .fetch_raw_messages(&password, &options)
        .await
        .expect("fetch IMAP messages");

    assert_eq!(batch.provider_kind, CommunicationProviderKind::Imap);
    assert_eq!(batch.stream_id, "imap:Archive");
    assert_eq!(
        batch.checkpoint,
        Some(json!({
            "provider": "imap",
            "mailbox": "Archive",
            "uid_validity": 999,
            "last_seen_uid": 43
        }))
    );
    assert_eq!(batch.messages.len(), 1);
    assert_eq!(
        batch.messages[0].provider_record_id,
        "imap:v2:imap:Archive:999:43"
    );
    assert_eq!(batch.messages[0].payload["provider"], "imap");
    assert_eq!(batch.messages[0].payload["mailbox"], "Archive");
    assert_eq!(
        batch.messages[0].payload["raw_rfc822_base64"],
        "U3ViamVjdDogSU1BUA0KDQpIZWxsbw=="
    );

    let commands = server.commands();
    assert!(
        commands
            .iter()
            .any(|command| command.contains("LOGIN") && command.contains("imap-user@example.net"))
    );
    assert!(
        commands
            .iter()
            .any(|command| command.contains("EXAMINE") && command.contains("Archive"))
    );
    assert!(
        commands
            .iter()
            .any(|command| command.contains("UID SEARCH UID 43:*"))
    );
    assert!(commands.iter().any(|command| {
        command.contains("UID FETCH 43 (UID FLAGS BODY.PEEK[] RFC822.SIZE INTERNALDATE)")
    }));
    for prohibited_command in ["SELECT", "STORE", "EXPUNGE", "COPY", "MOVE", "DELETE"] {
        assert!(
            !commands
                .iter()
                .any(|command| command.to_ascii_uppercase().contains(prohibited_command)),
            "IMAP client must not send mutating command `{prohibited_command}`: {commands:?}"
        );
    }
}

#[tokio::test]
async fn gmail_api_client_fetches_people_contacts_with_metadata() {
    let server = MockPeopleServer::start(vec![
        json!({
            "connections": [
                {
                    "resourceName": "people/c123",
                    "etag": "contact-etag-1",
                    "names": [{"displayName": "Maya Chen"}],
                    "emailAddresses": [{"value": "maya@example.com"}],
                    "phoneNumbers": [{"value": "+1 555 0100"}]
                }
            ],
            "nextPageToken": "contacts-next-page"
        })
        .to_string(),
    ]);
    let token = ResolvedSecret::new("people-access-token").expect("token");
    let client = GmailApiClient::new(server.base_url());

    let batch = client
        .fetch_entries(
            &token,
            &GmailContactFetchOptions::new(250).page_token("contacts-page-1"),
        )
        .await
        .expect("fetch address book entries");

    assert_eq!(batch.next_page_token.as_deref(), Some("contacts-next-page"));
    assert_eq!(batch.entries.len(), 1);
    let contact = &batch.entries[0];
    assert_eq!(contact.provider_address_book_entry_id, "people/c123");
    assert_eq!(contact.display_name.as_deref(), Some("Maya Chen"));
    assert_eq!(contact.email_addresses, vec!["maya@example.com"]);
    assert_eq!(contact.phone_numbers, vec!["+1 555 0100"]);
    assert_eq!(contact.etag.as_deref(), Some("contact-etag-1"));

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .request_line
            .starts_with("GET /v1/people/me/connections?")
    );
    assert!(requests[0].request_line.contains("pageSize=250"));
    assert!(
        requests[0]
            .request_line
            .contains("personFields=names%2CemailAddresses%2CphoneNumbers%2Cmetadata")
    );
    assert!(
        requests[0]
            .request_line
            .contains("pageToken=contacts-page-1")
    );
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer people-access-token")
    );
}

#[tokio::test]
async fn gmail_api_client_updates_linked_people_contact_with_etag() {
    let server = MockPeopleServer::start(vec![
        json!({
            "resourceName": "people/c123",
            "etag": "contact-etag-2",
            "names": [{"displayName": "Maya Chen"}],
            "emailAddresses": [{"value": "maya@example.com"}]
        })
        .to_string(),
    ]);
    let token = ResolvedSecret::new("people-access-token").expect("token");
    let client = GmailApiClient::new(server.base_url());

    let updated = client
        .upsert_entry(
            &token,
            &AddressBookProviderUpsertRequest {
                account_id: "acct-gmail".to_owned(),
                provider_kind: PlatformEmailProviderKind::Gmail,
                provider_address_book_entry_id: Some("people/c123".to_owned()),
                provider_etag: Some("contact-etag-1".to_owned()),
                display_name: "Maya Chen".to_owned(),
                email_address: Some("maya@example.com".to_owned()),
                phone_numbers: Vec::new(),
                remote_write_allowed: true,
            },
        )
        .await
        .expect("update address book entry");

    assert_eq!(updated.provider_address_book_entry_id, "people/c123");
    assert_eq!(updated.etag.as_deref(), Some("contact-etag-2"));

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .request_line
            .starts_with("PATCH /v1/people/c123:updateContact?")
    );
    assert!(
        requests[0]
            .request_line
            .contains("updatePersonFields=names%2CemailAddresses")
    );
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer people-access-token")
    );
    let body: serde_json::Value =
        serde_json::from_str(&requests[0].body).expect("json request body");
    assert_eq!(body["resourceName"], json!("people/c123"));
    assert_eq!(body["etag"], json!("contact-etag-1"));
    assert_eq!(body["names"][0]["unstructuredName"], json!("Maya Chen"));
    assert_eq!(
        body["emailAddresses"][0]["value"],
        json!("maya@example.com")
    );
    assert_eq!(
        body["metadata"]["sources"][0]["etag"],
        json!("contact-etag-1")
    );
}

#[tokio::test]
async fn gmail_api_client_creates_phone_only_people_contact() {
    let server = MockPeopleServer::start(vec![
        json!({
            "resourceName": "people/c-phone",
            "etag": "contact-etag-phone",
            "names": [{"displayName": "Phone Only Persona"}],
            "phoneNumbers": [{"value": "+1 555 0100"}]
        })
        .to_string(),
    ]);
    let token = ResolvedSecret::new("people-access-token").expect("token");
    let client = GmailApiClient::new(server.base_url());

    let created = client
        .upsert_entry(
            &token,
            &AddressBookProviderUpsertRequest {
                account_id: "acct-gmail".to_owned(),
                provider_kind: PlatformEmailProviderKind::Gmail,
                provider_address_book_entry_id: None,
                provider_etag: None,
                display_name: "Phone Only Persona".to_owned(),
                email_address: None,
                phone_numbers: vec!["+1 555 0100".to_owned()],
                remote_write_allowed: true,
            },
        )
        .await
        .expect("create phone-only address book entry");

    assert_eq!(created.provider_address_book_entry_id, "people/c-phone");
    assert_eq!(created.email_addresses, Vec::<String>::new());
    assert_eq!(created.phone_numbers, vec!["+1 555 0100".to_owned()]);

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].request_line,
        "POST /v1/people:createContact HTTP/1.1"
    );
    let body: serde_json::Value =
        serde_json::from_str(&requests[0].body).expect("json request body");
    assert_eq!(
        body["names"][0]["unstructuredName"],
        json!("Phone Only Persona")
    );
    assert!(body.get("emailAddresses").is_none());
    assert_eq!(body["phoneNumbers"][0]["value"], json!("+1 555 0100"));
}

#[tokio::test]
async fn email_sync_records_provider_network_batch_against_postgres() {
    let Some((database, store, suffix)) = live_sync_context("provider network batch").await else {
        return;
    };

    let account_id = format!("acct_network_batch_{suffix}");
    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::Gmail,
            "Network Gmail",
            format!("network-batch-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let batch = EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Gmail,
        stream_id: "gmail:history".to_owned(),
        checkpoint: Some(json!({"provider": "gmail", "history_id": "12345"})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: format!("gmail-network-message-{suffix}"),
            source_fingerprint: format!("sha256:gmail-network-message-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "gmail",
                "id": format!("gmail-network-message-{suffix}"),
                "raw_base64url": "U3ViamVjdA"
            }),
        }],
    };

    let report = record_email_sync_batch(
        &store,
        &account_id,
        &format!("provider-network-batch-{suffix}"),
        &batch,
    )
    .await
    .expect("record provider network batch");

    assert_eq!(report.inserted_or_existing_records, 1);
    assert!(report.checkpoint_saved);

    let pool = database.pool().expect("configured pool");
    let raw_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM communication_raw_records
        WHERE account_id = $1
          AND provider_record_id = $2
          AND payload ->> 'provider' = 'gmail'
        "#,
    )
    .bind(&account_id)
    .bind(&batch.messages[0].provider_record_id)
    .fetch_one(pool)
    .await
    .expect("count raw record");
    assert_eq!(raw_count, 1);

    let checkpoint = store
        .checkpoint(&account_id, &batch.stream_id)
        .await
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(checkpoint.checkpoint["history_id"], "12345");
}

#[tokio::test]
async fn email_sync_records_provider_batches_with_mail_blobs_against_postgres() {
    let Some((database, store, suffix)) = live_sync_context("provider blob batch").await else {
        return;
    };

    let pool = database.pool().expect("configured pool").clone();
    let mail_store = CommunicationAttachmentPort::new(pool.clone());
    let blob_root = tempfile::tempdir().expect("mail blob root");
    let blob_store = LocalBlobPort::new(blob_root.path());
    let gmail_account_id = format!("acct_blob_gmail_{suffix}");
    let imap_account_id = format!("acct_blob_imap_{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &gmail_account_id,
            CommunicationProviderKind::Gmail,
            "Blob Gmail",
            format!("blob-gmail-{suffix}@example.com"),
        ))
        .await
        .expect("store gmail provider account");
    store
        .upsert_provider_account(&NewProviderAccount::new(
            &imap_account_id,
            CommunicationProviderKind::Imap,
            "Blob IMAP",
            format!("blob-imap-{suffix}@example.net"),
        ))
        .await
        .expect("store imap provider account");

    let gmail_batch = EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Gmail,
        stream_id: "gmail:history".to_owned(),
        checkpoint: Some(json!({"provider": "gmail", "history_id": "blob-123"})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: format!("gmail-blob-message-{suffix}"),
            source_fingerprint: format!("sha256:gmail-blob-message-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_000_000).single(),
            payload: json!({
                "provider": "gmail",
                "id": format!("gmail-blob-message-{suffix}"),
                "raw_base64url": "U3ViamVjdDogR21haWwNCg0KSGVsbG8"
            }),
        }],
    };
    let imap_batch = EmailSyncBatch {
        provider_kind: CommunicationProviderKind::Imap,
        stream_id: "imap:INBOX".to_owned(),
        checkpoint: Some(json!({"provider": "imap", "last_seen_uid": 77})),
        messages: vec![FetchedCommunicationSourceMessage {
            provider_record_id: format!("imap-blob-message-{suffix}"),
            source_fingerprint: format!("sha256:imap-blob-message-{suffix}"),
            occurred_at: Utc.timestamp_millis_opt(1_770_000_100_000).single(),
            payload: json!({
                "provider": "imap",
                "uid": 77,
                "raw_rfc822_base64": "U3ViamVjdDogSU1BUA0KDQpIZWxsbw=="
            }),
        }],
    };

    let gmail_report = record_email_sync_batch_with_mail_blobs(
        &store,
        &mail_store,
        &blob_store,
        &gmail_account_id,
        &format!("provider-blob-gmail-{suffix}"),
        &gmail_batch,
    )
    .await
    .expect("record gmail provider blob batch");
    let imap_report = record_email_sync_batch_with_mail_blobs(
        &store,
        &mail_store,
        &blob_store,
        &imap_account_id,
        &format!("provider-blob-imap-{suffix}"),
        &imap_batch,
    )
    .await
    .expect("record imap provider blob batch");

    assert_eq!(gmail_report.inserted_or_existing_records, 1);
    assert_eq!(gmail_report.blobs_upserted, 1);
    assert!(gmail_report.checkpoint_saved);
    assert_eq!(imap_report.inserted_or_existing_records, 1);
    assert_eq!(imap_report.blobs_upserted, 1);
    assert!(imap_report.checkpoint_saved);

    let rows = sqlx::query(
        r#"
        SELECT account_id, payload
        FROM communication_raw_records
        WHERE account_id IN ($1, $2)
        ORDER BY account_id
        "#,
    )
    .bind(&gmail_account_id)
    .bind(&imap_account_id)
    .fetch_all(&pool)
    .await
    .expect("raw records");
    assert_eq!(rows.len(), 2);

    for row in rows {
        let payload: serde_json::Value = row.try_get("payload").expect("payload");
        assert!(payload.get("raw_base64url").is_none());
        assert!(payload.get("raw_rfc822_base64").is_none());
        assert!(
            payload["raw_blob_id"]
                .as_str()
                .expect("raw_blob_id")
                .starts_with("blob:v1:sha256:")
        );
        assert_eq!(payload["raw_blob_storage_kind"], "local_fs");
        let storage_path = payload["raw_blob_storage_path"]
            .as_str()
            .expect("raw_blob_storage_path");
        assert!(blob_root.path().join(storage_path).is_file());
        assert!(
            payload["raw_blob_sha256"]
                .as_str()
                .expect("raw_blob_sha256")
                .starts_with("sha256:")
        );
        assert!(payload["raw_blob_size_bytes"].as_i64().unwrap() > 0);
    }

    let blob_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM communication_mail_blobs")
        .fetch_one(&pool)
        .await
        .expect("mail blob count");
    assert!(blob_count >= 2);
}

struct MockGmailServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<RecordedHttpRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockGmailServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock Gmail server");
        let addr = listener.local_addr().expect("mock Gmail addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept Gmail request");
                let request = read_http_request(&mut stream);
                let request_line = request.request_line.clone();
                requests_for_thread
                    .lock()
                    .expect("requests lock")
                    .push(request);

                let body = if request_line.starts_with("GET /gmail/v1/users/me/messages?") {
                    json!({
                        "messages": [{"id": "gmail-msg-1", "threadId": "thread-1"}],
                        "nextPageToken": "next-page",
                        "resultSizeEstimate": 1
                    })
                    .to_string()
                } else if request_line
                    == "GET /gmail/v1/users/me/messages/gmail-msg-1?format=raw HTTP/1.1"
                {
                    json!({
                        "id": "gmail-msg-1",
                        "threadId": "thread-1",
                        "labelIds": ["INBOX"],
                        "historyId": "12345",
                        "internalDate": "1770000000000",
                        "raw": "U3ViamVjdDogR21haWwNCg0KSGVsbG8"
                    })
                    .to_string()
                } else {
                    json!({"error": "unexpected request", "request": request_line}).to_string()
                };

                write_http_response(&mut stream, &body);
            }
        });

        Self {
            addr,
            requests,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> Vec<RecordedHttpRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl Drop for MockGmailServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("mock Gmail server join");
        }
    }
}

struct MockPeopleServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<RecordedHttpRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockPeopleServer {
    fn start(responses: Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock People server");
        let addr = listener.local_addr().expect("mock People addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for body in responses {
                let (mut stream, _) = listener.accept().expect("accept People request");
                let request = read_http_request(&mut stream);
                requests_for_thread
                    .lock()
                    .expect("requests lock")
                    .push(request);
                write_http_response(&mut stream, &body);
            }
        });

        Self {
            addr,
            requests,
            handle: Some(handle),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn requests(&self) -> Vec<RecordedHttpRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl Drop for MockPeopleServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("mock People server join");
        }
    }
}

#[derive(Clone, Debug)]
struct RecordedHttpRequest {
    request_line: String,
    authorization: Option<String>,
    body: String,
}

fn read_http_request(stream: &mut TcpStream) -> RecordedHttpRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .expect("read request line");
    let request_line = request_line.trim_end().to_owned();
    let mut authorization = None;
    let mut content_length = 0usize;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read request header");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("authorization")
        {
            authorization = Some(value.trim().to_owned());
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).expect("read request body");
    }

    RecordedHttpRequest {
        request_line,
        authorization,
        body: String::from_utf8(body).expect("utf8 request body"),
    }
}

fn write_http_response(stream: &mut TcpStream, body: &str) {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    )
    .expect("write response");
}

struct MockImapServer {
    addr: SocketAddr,
    commands: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockImapServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock IMAP server");
        let addr = listener.local_addr().expect("mock IMAP addr");
        let commands = Arc::new(Mutex::new(Vec::new()));
        let commands_for_thread = Arc::clone(&commands);
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept IMAP request");
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("set IMAP timeout");
            stream
                .write_all(b"* OK makosh test imap ready\r\n")
                .expect("write IMAP greeting");

            let mut reader = BufReader::new(stream.try_clone().expect("clone IMAP stream"));
            loop {
                let mut line = String::new();
                let bytes = reader.read_line(&mut line).expect("read IMAP command");
                if bytes == 0 {
                    break;
                }
                let command = line.trim_end().to_owned();
                commands_for_thread
                    .lock()
                    .expect("commands lock")
                    .push(command.clone());
                let tag = command
                    .split_whitespace()
                    .next()
                    .expect("tagged IMAP command")
                    .to_owned();

                if command.contains("LOGIN") {
                    writeln!(stream, "{tag} OK LOGIN completed\r").expect("write LOGIN response");
                } else if command.contains("EXAMINE") {
                    write!(
                        stream,
                        "* FLAGS (\\Seen)\r\n* 1 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 999] UIDs valid\r\n* OK [UIDNEXT 44] Predicted next UID\r\n{tag} OK [READ-ONLY] EXAMINE completed\r\n"
                    )
                    .expect("write EXAMINE response");
                } else if command.contains("UID SEARCH") {
                    write!(stream, "* SEARCH 43\r\n{tag} OK SEARCH completed\r\n")
                        .expect("write SEARCH response");
                } else if command.contains("UID FETCH") {
                    write!(
                        stream,
                        "* 1 FETCH (UID 43 RFC822.SIZE 22 INTERNALDATE \"04-Jun-2026 12:00:00 +0000\" BODY[] {{22}}\r\nSubject: IMAP\r\n\r\nHello)\r\n{tag} OK FETCH completed\r\n"
                    )
                    .expect("write FETCH response");
                } else if command.contains("LOGOUT") {
                    write!(stream, "* BYE logging out\r\n{tag} OK LOGOUT completed\r\n")
                        .expect("write LOGOUT response");
                    break;
                } else {
                    write!(stream, "{tag} BAD unexpected command\r\n").expect("write BAD response");
                }
            }
        });

        Self {
            addr,
            commands,
            handle: Some(handle),
        }
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }

    fn commands(&self) -> Vec<String> {
        self.commands.lock().expect("commands lock").clone()
    }
}

impl Drop for MockImapServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("mock IMAP server join");
        }
    }
}

async fn live_sync_context(
    _test_name: &str,
) -> Option<(Database, CommunicationIngestionStore, u128)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());

    Some((database, store, unique_suffix()))
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

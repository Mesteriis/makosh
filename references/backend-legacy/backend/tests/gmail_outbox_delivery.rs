use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;
use makosh_hub_backend::domains::communications::outbox::delivery::EmailOutboxDeliveryWorker;
use makosh_hub_backend::domains::communications::outbox::provider_sender::CommunicationOutboxEmailSender;
use makosh_hub_backend::domains::communications::outbox::{
    CommunicationOutboxStatus, CommunicationOutboxStore, NewCommunicationOutboxItem,
};
use makosh_hub_backend::integrations::mail::outbox::LiveGmailOutboxTransport;
use makosh_hub_backend::integrations::mail::send::LiveSmtpTransport;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, SecretKind, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::HostVault;
use makosh_hub_backend::vault::models::{HostVaultConfig, SecretEntryContext};

const LOCAL_API_TOKEN: &str = "gmail-outbox-delivery-test-token";

#[tokio::test]
async fn outbox_delivery_worker_sends_gmail_items_through_gmail_api_against_postgres() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let config = makosh_backend_testkit::app::config_with_secret_and_database_url(
        LOCAL_API_TOKEN,
        database_url.as_str(),
    )
    .with_test_pairs([
        ("MAKOSH_DEV_MODE", "true"),
        (
            "MAKOSH_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "MAKOSH_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
    ])
    .expect("config");
    let app = build_router_with_database(config, database);
    unlock_test_vault(app).await;

    let gmail_api = MockGmailApiServer::start();
    let account_id = "gmail-outbox-enabled";
    let secret_ref = format!("secret:provider-account:{account_id}:oauth_token");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Gmail,
                "Gmail Outbox Enabled",
                "sender@gmail.com",
            )
            .config(json!({
                "auth": "oauth",
                "api": "gmail",
                "gmail_send_enabled": true,
                "gmail_api_base_url": gmail_api.base_url()
            })),
        )
        .await
        .expect("store gmail account");
    SecretReferenceStore::new(pool.clone())
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::HostVault,
            "Gmail outbox OAuth credential",
        ))
        .await
        .expect("store gmail OAuth secret reference");
    communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &secret_ref,
        ))
        .await
        .expect("bind gmail OAuth secret");
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .store_secret(
            &secret_ref,
            &json!({
                "token_url": "http://127.0.0.1:1/token",
                "client_id": "desktop-client-id",
                "access_token": "gmail-access-token",
                "refresh_token": "gmail-refresh-token",
                "expires_at": "2999-01-01T00:00:00Z",
                "scope": "https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/gmail.send"
            })
            .to_string(),
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id,
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "Gmail OAuth credential",
                metadata: &json!({ "provider": "gmail", "account_id": account_id }),
            },
        )
        .expect("store gmail OAuth bundle");

    let outbox_store = CommunicationOutboxStore::new(pool.clone());
    let now = Utc::now();
    let outbox_id = "outbox:gmail:scheduled";
    outbox_store
        .enqueue(&NewCommunicationOutboxItem {
            outbox_id: outbox_id.to_owned(),
            account_id: account_id.to_owned(),
            draft_id: None,
            to_recipients: vec!["recipient@example.com".to_owned()],
            cc_recipients: vec!["copy@example.com".to_owned()],
            bcc_recipients: Vec::new(),
            subject: "Scheduled Gmail API send".to_owned(),
            body_text: "Queued Gmail outbox body.".to_owned(),
            body_html: None,
            status: CommunicationOutboxStatus::Scheduled,
            scheduled_send_at: Some(now - Duration::seconds(1)),
            undo_deadline_at: Some(now - Duration::seconds(1)),
            metadata: json!({}),
        })
        .await
        .expect("enqueue gmail outbox item");

    let worker = EmailOutboxDeliveryWorker::new(
        outbox_store.clone(),
        CommunicationOutboxEmailSender::new(
            pool.clone(),
            vault.clone(),
            LiveSmtpTransport,
            LiveGmailOutboxTransport::new(pool.clone(), vault),
        ),
    );
    let report = worker
        .deliver_due(now + Duration::seconds(1), 10)
        .await
        .expect("deliver gmail outbox");
    assert_eq!(report.claimed, 1);
    assert_eq!(report.sent, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.retried, 0);

    let sent_items = outbox_store
        .list(Some(account_id), Some(CommunicationOutboxStatus::Sent), 10)
        .await
        .expect("list sent outbox");
    assert_eq!(sent_items.len(), 1);
    assert_eq!(
        sent_items[0].provider_message_id.as_deref(),
        Some("gmail-api-message-id")
    );

    let requests = gmail_api.requests();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .request_line
            .starts_with("POST /gmail/v1/users/me/messages/send ")
    );
    assert_eq!(
        requests[0].header("authorization").as_deref(),
        Some("Bearer gmail-access-token")
    );
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", LOCAL_API_TOKEN)
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn unlock_test_vault<S>(app: S)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let entropy_response = app
        .clone()
        .oneshot(post(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);

    let create_response = app
        .oneshot(post("/api/v1/vault/create", json!({})))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);
}

fn vault_entropy_events(count: usize) -> Vec<Value> {
    (0..count)
        .map(|index| {
            json!({
                "x": index % 997,
                "y": index % 577,
                "dx": (index % 11) as i64 - 5,
                "dy": (index % 13) as i64 - 6,
                "timestamp_ms": index * 5,
                "velocity": (index % 19) as f64 / 10.0,
                "acceleration": (index % 23) as f64 / 100.0,
                "interval_ms": 5
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct HttpRequest {
    request_line: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl HttpRequest {
    fn header(&self, name: &str) -> Option<String> {
        self.headers
            .iter()
            .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.clone())
    }
}

struct MockGmailApiServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockGmailApiServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind Gmail API server");
        let addr = listener.local_addr().expect("Gmail API server addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let request = read_http_request(&mut stream);
            if request.body.is_empty() {
                return;
            }
            requests_for_thread
                .lock()
                .expect("Gmail API requests lock")
                .push(request);
            write_http_response(
                &mut stream,
                &json!({
                    "id": "gmail-api-message-id",
                    "threadId": "gmail-api-thread-id",
                    "labelIds": ["SENT"]
                })
                .to_string(),
            );
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

    fn requests(&self) -> Vec<HttpRequest> {
        self.requests
            .lock()
            .expect("Gmail API requests lock")
            .clone()
    }
}

impl Drop for MockGmailApiServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("Gmail API server join");
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> HttpRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut content_length = 0usize;
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .expect("read request line");
    let request_line = request_line.trim_end().to_owned();
    let mut headers = Vec::new();

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read header line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_owned();
            let value = value.trim().to_owned();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().expect("content length");
            }
            headers.push((name, value));
        }
    }

    let mut body = vec![0_u8; content_length];
    use std::io::Read;
    reader.read_exact(&mut body).expect("read request body");

    HttpRequest {
        request_line,
        headers,
        body: String::from_utf8(body).expect("utf8 body"),
    }
}

fn write_http_response(stream: &mut TcpStream, body: &str) {
    let result = write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    if let Err(error) = result {
        assert_eq!(
            error.kind(),
            ErrorKind::BrokenPipe,
            "write response: {error}"
        );
    }
}

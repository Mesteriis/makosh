use makosh_communications_api::accounts::{CommunicationProviderKind, NewProviderAccount};
use makosh_communications_api::accounts::{
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::app::router::build_router_with_database;

use makosh_backend_testkit::context::TestContext;
use makosh_hub_backend::platform::secrets::models::{
    NewSecretReference, SecretKind, SecretStoreKind,
};
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::HostVault;
use makosh_hub_backend::vault::models::{HostVaultConfig, SecretEntryContext};

const LOCAL_API_TOKEN: &str = "gmail-send-api-test-token";

#[tokio::test]
async fn gmail_send_api_queues_outbox_when_send_scope_enabled_against_postgres() {
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
    unlock_test_vault(app.clone()).await;

    let gmail_api = MockGmailApiServer::start();
    let account_id = "gmail-send-enabled";
    let secret_ref = format!("secret:provider-account:{account_id}:oauth_token");
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    communication_store
        .upsert_provider_account(
            &NewProviderAccount::new(
                account_id,
                CommunicationProviderKind::Gmail,
                "Gmail Send Enabled",
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
            "Gmail send OAuth credential",
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

    let response = app
        .oneshot(post(
            "/api/v1/communications/send",
            json!({
                "account_id": account_id,
                "to": ["recipient@example.com"],
                "cc": ["copy@example.com"],
                "subject": "Gmail API send",
                "body_text": "Message body through Gmail API.",
                "confirmed_provider_write": true
            }),
        ))
        .await
        .expect("send response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["transport"], "outbox");
    assert_eq!(body["status"], "queued");
    assert_eq!(
        body["accepted_recipients"],
        json!(["recipient@example.com", "copy@example.com"])
    );
    let outbox_id = body["outbox_id"].as_str().expect("outbox id");
    assert_eq!(body["message_id"], json!(outbox_id));
    let outbox = sqlx::query(
        "SELECT status, to_participants, cc_participants, subject
         FROM communication_outbox
         WHERE outbox_id = $1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("gmail outbox item");
    let status: String = outbox.try_get("status").expect("outbox status");
    let subject: String = outbox.try_get("subject").expect("outbox subject");
    let to_participants: Value = outbox.try_get("to_participants").expect("to participants");
    let cc_participants: Value = outbox.try_get("cc_participants").expect("cc participants");
    assert_eq!(status, "queued");
    assert_eq!(subject, "Gmail API send");
    assert_eq!(to_participants, json!(["recipient@example.com"]));
    assert_eq!(cc_participants, json!(["copy@example.com"]));

    let link = sqlx::query(
        "SELECT observation_id, metadata
         FROM observation_links
         WHERE domain = 'communications'
           AND entity_kind = 'outbox_item'
           AND entity_id = $1
           AND relationship_kind = 'outbox_status_transition'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("gmail outbox observation link");
    let link_metadata: Value = link.try_get("metadata").expect("link metadata");
    assert_eq!(link_metadata["operation"], "outbox_enqueue");
    assert_eq!(link_metadata["status"], "queued");

    let requests = gmail_api.requests();
    assert!(requests.is_empty());
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

async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("read response body"),
    )
    .expect("response json")
}

#[derive(Clone, Debug)]
struct HttpRequest {
    body: String,
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
        }
    }

    let mut body = vec![0_u8; content_length];
    use std::io::Read;
    reader.read_exact(&mut body).expect("read request body");

    HttpRequest {
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

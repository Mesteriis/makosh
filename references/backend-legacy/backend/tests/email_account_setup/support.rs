#![allow(dead_code)]

use makosh_backend_testkit::context::TestContext;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

use makosh_communications_api::accounts::ProviderAccountSecretPurpose;
use makosh_communications_postgres::store::CommunicationIngestionStore;
use makosh_hub_backend::domains::calendar::events::account_store::CalendarAccountStore;
use makosh_hub_backend::domains::calendar::events::models::CalendarAccount;
use makosh_hub_backend::platform::secrets::models::SecretReference;
use makosh_hub_backend::platform::secrets::models::{SecretKind, SecretStoreKind};
use makosh_hub_backend::platform::secrets::store::SecretReferenceStore;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::vault::HostVault;

pub const LOCAL_API_TOKEN: &str = "account-setup-test-token";

#[derive(Clone, Debug)]
pub struct TokenRequest {
    pub body: String,
}

pub struct MockTokenServer {
    addr: SocketAddr,
    requests: Arc<Mutex<Vec<TokenRequest>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockTokenServer {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind token server");
        let addr = listener.local_addr().expect("token server addr");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let request = read_http_request(&mut stream);
                if request.body.is_empty() {
                    break;
                }
                let body = if request.body.contains("grant_type=refresh_token") {
                    json!({
                        "access_token": "gmail-refreshed-access-token",
                        "expires_in": 3600,
                        "token_type": "Bearer"
                    })
                    .to_string()
                } else {
                    json!({
                        "access_token": "gmail-access-token",
                        "refresh_token": "gmail-refresh-token",
                        "expires_in": 3600,
                        "token_type": "Bearer",
                        "scope": "https://www.googleapis.com/auth/gmail.readonly"
                    })
                    .to_string()
                };
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

    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub fn requests(&self) -> Vec<TokenRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl Drop for MockTokenServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("token server join");
        }
    }
}

pub struct MockSmtpServer {
    addr: SocketAddr,
    commands: Arc<Mutex<Vec<String>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockSmtpServer {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind SMTP server");
        let addr = listener.local_addr().expect("SMTP server addr");
        let commands = Arc::new(Mutex::new(Vec::new()));
        let commands_for_thread = Arc::clone(&commands);
        let handle = thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("set SMTP read timeout");
            write!(stream, "220 mock.smtp.local ESMTP\r\n").expect("write greeting");

            let mut reader = BufReader::new(stream.try_clone().expect("clone SMTP stream"));
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => break,
                    Err(error) => panic!("read SMTP line: {error}"),
                }
                let command = line.trim_end().to_owned();
                commands_for_thread
                    .lock()
                    .expect("SMTP commands lock")
                    .push(command.clone());
                if command.starts_with("EHLO") {
                    write!(stream, "250-mock.smtp.local\r\n250 AUTH LOGIN\r\n")
                        .expect("write EHLO response");
                } else if command == "AUTH LOGIN" {
                    write!(stream, "334 VXNlcm5hbWU6\r\n").expect("write username prompt");
                } else if command == "c2VuZGVyQGV4YW1wbGUuY29t" {
                    write!(stream, "334 UGFzc3dvcmQ6\r\n").expect("write password prompt");
                } else if command == "c210cC1hcHAtcGFzc3dvcmQ=" {
                    write!(stream, "235 Authentication successful\r\n").expect("write auth ok");
                } else if command.starts_with("MAIL FROM") || command.starts_with("RCPT TO") {
                    write!(stream, "250 OK\r\n").expect("write envelope ok");
                } else if command == "DATA" {
                    write!(stream, "354 End data with <CR><LF>.<CR><LF>\r\n")
                        .expect("write DATA response");
                    loop {
                        let mut data_line = String::new();
                        if reader
                            .read_line(&mut data_line)
                            .expect("read SMTP data line")
                            == 0
                        {
                            return;
                        }
                        let data_line = data_line.trim_end().to_owned();
                        commands_for_thread
                            .lock()
                            .expect("SMTP commands lock")
                            .push(data_line.clone());
                        if data_line == "." {
                            break;
                        }
                    }
                    write!(stream, "250 mock-message-id queued\r\n").expect("write queued");
                } else if command == "QUIT" {
                    write!(stream, "221 Bye\r\n").expect("write bye");
                    break;
                } else {
                    write!(stream, "250 OK\r\n").expect("write default ok");
                }
            }
        });

        Self {
            addr,
            commands,
            handle: Some(handle),
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn commands(&self) -> Vec<String> {
        self.commands.lock().expect("SMTP commands lock").clone()
    }
}

impl Drop for MockSmtpServer {
    fn drop(&mut self) {
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            handle.join().expect("SMTP server join");
        }
    }
}

fn read_http_request(stream: &mut TcpStream) -> TokenRequest {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("set read timeout");
    let mut reader = BufReader::new(stream);
    let mut content_length = 0usize;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read request line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().expect("content length");
        }
    }

    let mut body = vec![0_u8; content_length];
    use std::io::Read;
    reader.read_exact(&mut body).expect("read request body");

    TokenRequest {
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

pub async fn live_setup_context(
    _test_name: &str,
) -> Option<(
    Database,
    CommunicationIngestionStore,
    SecretReferenceStore,
    u128,
)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);

    Some((database, communication_store, secret_store, unique_suffix()))
}

pub fn secret_reference(secret_ref: &str) -> SecretReference {
    let now = chrono::Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind: SecretKind::OauthToken,
        store_kind: SecretStoreKind::DatabaseEncryptedVault,
        label: "Gmail OAuth".to_owned(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}

pub fn json_request_with_token_and_actor(
    uri: &str,
    body: Value,
    token: &str,
    _actor_id: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", token)
        .body(Body::from(body.to_string()))
        .expect("request")
}

pub fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}

pub fn delete_request_with_token(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(uri)
        .header("x-makosh-secret", token)
        .body(Body::empty())
        .expect("request")
}

pub async fn unlock_test_vault<S>(app: S)
where
    S: tower::Service<Request<Body>, Response = axum::response::Response> + Clone,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    let entropy_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/vault/collect-entropy",
            json!({ "events": vault_entropy_events(2_000) }),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("entropy response");
    assert_eq!(entropy_response.status(), StatusCode::OK);

    let create_response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/vault/create",
            json!({}),
            LOCAL_API_TOKEN,
            "makosh-frontend",
        ))
        .await
        .expect("vault create response");
    assert_eq!(create_response.status(), StatusCode::OK);
}

pub async fn wait_for_provider_account(
    communication_store: &CommunicationIngestionStore,
    account_id: &str,
) -> makosh_communications_api::accounts::ProviderAccount {
    for _ in 0..50 {
        if let Some(account) = communication_store
            .provider_account(account_id)
            .await
            .expect("load provider account")
        {
            return account;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("provider account {account_id} was not reconciled");
}

pub async fn wait_for_secret_reference(
    secret_store: &SecretReferenceStore,
    secret_ref: &str,
) -> SecretReference {
    for _ in 0..50 {
        if let Some(reference) = secret_store
            .secret_reference(secret_ref)
            .await
            .expect("load secret reference")
        {
            return reference;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("secret reference {secret_ref} was not reconciled");
}

pub async fn wait_for_provider_account_secret_binding(
    communication_store: &CommunicationIngestionStore,
    account_id: &str,
    secret_purpose: ProviderAccountSecretPurpose,
) -> makosh_communications_api::accounts::ProviderAccountSecretBinding {
    for _ in 0..50 {
        if let Some(binding) = communication_store
            .provider_account_secret_binding(account_id, secret_purpose)
            .await
            .expect("load provider account secret binding")
        {
            return binding;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("provider account secret binding {account_id}/{secret_purpose:?} was not reconciled");
}

pub async fn wait_for_calendar_account(
    calendar_store: &CalendarAccountStore,
    account_id: &str,
) -> CalendarAccount {
    for _ in 0..50 {
        if let Some(account) = calendar_store
            .get(account_id)
            .await
            .expect("load calendar account")
        {
            return account;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("calendar account {account_id} was not reconciled");
}

pub async fn wait_for_manifest_metadata_key(vault: &HostVault, secret_ref: &str, key: &str) {
    for _ in 0..50 {
        let has_key = vault
            .account_secret_manifest()
            .expect("read host vault manifest")
            .into_iter()
            .any(|entry| entry.secret_ref == secret_ref && entry.metadata.get(key).is_some());
        if has_key {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("manifest entry {secret_ref} was not enriched with {key}");
}

pub fn vault_entropy_events(count: usize) -> Vec<Value> {
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

pub async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json body")
}

pub async fn text_body(response: axum::response::Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    String::from_utf8(body.to_vec()).expect("utf8 body")
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

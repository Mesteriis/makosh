//! Loopback TLS Gmail API provider used by the managed Mail delivery contour.

use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rcgen::generate_simple_self_signed;
use rustls::{
    ServerConfig, ServerConnection, StreamOwned,
    pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
};

const MAX_HTTP_LINE_BYTES: usize = 8 * 1024;
const MAX_REQUEST_BODY_BYTES: usize = 2 * 1024 * 1024;
const GMAIL_INBOUND_MESSAGE: &[u8] = concat!(
    "From: gmail-source@example.test\r\n",
    "To: owner@example.test\r\n",
    "Subject: managed Gmail attachment evidence\r\n",
    "Content-Type: multipart/mixed; boundary=makosh-gmail-fixture\r\n",
    "\r\n",
    "--makosh-gmail-fixture\r\n",
    "Content-Type: text/plain; charset=utf-8\r\n",
    "\r\n",
    "managed Gmail body\r\n",
    "--makosh-gmail-fixture\r\n",
    "Content-Type: text/plain; name=gmail-evidence.txt\r\n",
    "Content-Disposition: attachment; filename=gmail-evidence.txt\r\n",
    "Content-Transfer-Encoding: base64\r\n",
    "\r\n",
    "Z21haWwtY2xlYW4tcm9vbS1hdHRhY2htZW50\r\n",
    "--makosh-gmail-fixture--\r\n",
)
.as_bytes();

#[derive(Clone)]
pub(super) struct GmailSentRequestV1 {
    pub(super) path: String,
    pub(super) authorization: String,
    pub(super) raw: String,
    pub(super) thread_id: String,
}

#[derive(Clone)]
pub(super) struct GooglePeopleWriteRequestV1 {
    pub(super) method: String,
    pub(super) path: String,
    pub(super) authorization: String,
    pub(super) body: serde_json::Value,
}

pub(super) struct MailGmailFixture {
    port: u16,
    ca_certificate_pem: String,
    accepted_mutations: Arc<AtomicUsize>,
    accepted_reads: Arc<AtomicUsize>,
    accepted_people_reads: Arc<AtomicUsize>,
    accepted_people_writes: Arc<AtomicUsize>,
    drop_next_people_write_response: Arc<AtomicBool>,
    ambiguous_people_write_committed: Arc<AtomicBool>,
    last_request: Arc<Mutex<Option<GmailSentRequestV1>>>,
    last_people_write: Arc<Mutex<Option<GooglePeopleWriteRequestV1>>>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

struct GmailFixtureState<'a> {
    accepted_mutations: &'a AtomicUsize,
    accepted_reads: &'a AtomicUsize,
    accepted_people_reads: &'a AtomicUsize,
    accepted_people_writes: &'a AtomicUsize,
    drop_next_people_write_response: &'a AtomicBool,
    ambiguous_people_write_committed: &'a AtomicBool,
    last_request: &'a Mutex<Option<GmailSentRequestV1>>,
    last_people_write: &'a Mutex<Option<GooglePeopleWriteRequestV1>>,
}

impl MailGmailFixture {
    pub(super) fn start() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified = generate_simple_self_signed(vec!["localhost".to_owned()])
            .expect("generate Gmail fixture certificate");
        let ca_certificate_pem = certified.cert.pem();
        let certificate = certified.cert.der().clone();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let server = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], key)
                .expect("configure Gmail fixture TLS"),
        );
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind Gmail fixture");
        listener
            .set_nonblocking(true)
            .expect("configure Gmail fixture listener");
        let port = listener.local_addr().expect("Gmail fixture address").port();
        let accepted_mutations = Arc::new(AtomicUsize::new(0));
        let accepted_reads = Arc::new(AtomicUsize::new(0));
        let accepted_people_reads = Arc::new(AtomicUsize::new(0));
        let accepted_people_writes = Arc::new(AtomicUsize::new(0));
        let drop_next_people_write_response = Arc::new(AtomicBool::new(false));
        let ambiguous_people_write_committed = Arc::new(AtomicBool::new(false));
        let last_request = Arc::new(Mutex::new(None));
        let last_people_write = Arc::new(Mutex::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_mutations = Arc::clone(&accepted_mutations);
        let worker_reads = Arc::clone(&accepted_reads);
        let worker_people_reads = Arc::clone(&accepted_people_reads);
        let worker_people_writes = Arc::clone(&accepted_people_writes);
        let worker_drop_people_response = Arc::clone(&drop_next_people_write_response);
        let worker_ambiguous_people_write = Arc::clone(&ambiguous_people_write_committed);
        let worker_request = Arc::clone(&last_request);
        let worker_people_write = Arc::clone(&last_people_write);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("configure blocking Gmail fixture connection");
                        stream
                            .set_read_timeout(Some(Duration::from_secs(10)))
                            .expect("configure Gmail fixture read timeout");
                        stream
                            .set_write_timeout(Some(Duration::from_secs(10)))
                            .expect("configure Gmail fixture write timeout");
                        let connection =
                            ServerConnection::new(Arc::clone(&server)).expect("Gmail TLS session");
                        let mut stream = StreamOwned::new(connection, stream);
                        let state = GmailFixtureState {
                            accepted_mutations: &worker_mutations,
                            accepted_reads: &worker_reads,
                            accepted_people_reads: &worker_people_reads,
                            accepted_people_writes: &worker_people_writes,
                            drop_next_people_write_response: &worker_drop_people_response,
                            ambiguous_people_write_committed: &worker_ambiguous_people_write,
                            last_request: &worker_request,
                            last_people_write: &worker_people_write,
                        };
                        serve_connection(&mut stream, &state);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("Gmail fixture accept failed: {error}"),
                }
            }
        });
        Self {
            port,
            ca_certificate_pem,
            accepted_mutations,
            accepted_reads,
            accepted_people_reads,
            accepted_people_writes,
            drop_next_people_write_response,
            ambiguous_people_write_committed,
            last_request,
            last_people_write,
            shutdown,
            worker: Some(worker),
        }
    }

    pub(super) fn port(&self) -> u16 {
        self.port
    }

    pub(super) fn ca_certificate_pem(&self) -> &str {
        &self.ca_certificate_pem
    }

    pub(super) fn accepted_mutations(&self) -> usize {
        self.accepted_mutations.load(Ordering::SeqCst)
    }

    pub(super) fn accepted_reads(&self) -> usize {
        self.accepted_reads.load(Ordering::SeqCst)
    }

    pub(super) fn accepted_people_reads(&self) -> usize {
        self.accepted_people_reads.load(Ordering::SeqCst)
    }

    pub(super) fn accepted_people_writes(&self) -> usize {
        self.accepted_people_writes.load(Ordering::SeqCst)
    }

    pub(super) fn drop_next_people_write_response(&self) {
        self.drop_next_people_write_response
            .store(true, Ordering::SeqCst);
    }

    pub(super) fn last_people_write(&self) -> GooglePeopleWriteRequestV1 {
        self.last_people_write
            .lock()
            .expect("lock Google People write")
            .clone()
            .expect("Google People write")
    }

    pub(super) fn last_request(&self) -> GmailSentRequestV1 {
        self.last_request
            .lock()
            .expect("lock Gmail fixture request")
            .clone()
            .expect("Gmail fixture request")
    }
}

impl Drop for MailGmailFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let outcome = worker.join();
            if !std::thread::panicking() {
                outcome.expect("join Gmail fixture");
            }
        }
    }
}

fn serve_connection(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    state: &GmailFixtureState<'_>,
) {
    let request_line = read_line(stream);
    let request_line = std::str::from_utf8(&request_line).expect("Gmail request line");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().expect("Gmail request method");
    let path = parts.next().expect("Gmail request path").to_owned();
    assert_eq!(parts.next(), Some("HTTP/1.1"));
    assert_eq!(parts.next(), None);

    let mut headers = BTreeMap::new();
    loop {
        let line = read_line(stream);
        if line == b"\r\n" {
            break;
        }
        let line = std::str::from_utf8(&line).expect("Gmail request header");
        let (name, value) = line
            .trim_end()
            .split_once(':')
            .expect("Gmail request header shape");
        assert!(
            headers
                .insert(name.to_ascii_lowercase(), value.trim().to_owned())
                .is_none(),
            "duplicate Gmail request header"
        );
    }
    match method {
        "GET" => {
            serve_get(
                stream,
                &path,
                &headers,
                state.accepted_people_reads,
                state.accepted_people_writes,
                state.ambiguous_people_write_committed,
            );
            state.accepted_reads.fetch_add(1, Ordering::SeqCst);
        }
        "POST" if path == "/gmail/v1/users/me/messages/send" => {
            serve_send(
                stream,
                &path,
                &headers,
                state.accepted_mutations,
                state.last_request,
            );
        }
        "POST" | "PATCH" => serve_people_write(stream, method, &path, &headers, state),
        _ => panic!("unsupported Gmail fixture method"),
    }
}

fn serve_get(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    path: &str,
    headers: &BTreeMap<String, String>,
    accepted_people_reads: &AtomicUsize,
    accepted_people_writes: &AtomicUsize,
    ambiguous_people_write_committed: &AtomicBool,
) {
    let body = if path.starts_with("/gmail/v1/users/me/messages?") {
        serde_json::to_vec(&serde_json::json!({
            "messages": [{
                "id": "gmail-inbound-1",
                "threadId": "gmail-inbound-thread-1"
            }]
        }))
        .expect("encode Gmail list response")
    } else if path == "/gmail/v1/users/me/messages/gmail-inbound-1?format=raw" {
        serde_json::to_vec(&serde_json::json!({
            "id": "gmail-inbound-1",
            "threadId": "gmail-inbound-thread-1",
            "labelIds": ["INBOX"],
            "historyId": "101",
            "internalDate": "1700000000000",
            "raw": URL_SAFE_NO_PAD.encode(GMAIL_INBOUND_MESSAGE)
        }))
        .expect("encode Gmail raw response")
    } else if path.starts_with("/gmail/v1/users/me/history?") {
        serde_json::to_vec(&serde_json::json!({
            "history": [{
                "messagesAdded": [{
                    "message": {
                        "id": "gmail-inbound-1"
                    }
                }]
            }],
            "historyId": "101"
        }))
        .expect("encode Gmail history response")
    } else if path.starts_with("/v1/people/me/connections?") {
        let managed_etag = if accepted_people_writes.load(Ordering::SeqCst) == 0 {
            "managed-etag-1"
        } else {
            "managed-etag-2"
        };
        assert_eq!(
            headers.get("authorization").map(String::as_str),
            Some("Bearer managed-mail-gmail-access-token")
        );
        accepted_people_reads.fetch_add(1, Ordering::SeqCst);
        let mut connections = vec![serde_json::json!({
            "resourceName": "people/managed-contact-1",
            "metadata": {
                "deleted": false,
                "sources": [{
                    "type": "CONTACT",
                    "id": "managed-contact-1",
                    "etag": managed_etag
                }]
            },
            "names": [{"displayName": "Private Managed Contact"}],
            "emailAddresses": [{"value": "private-managed-contact@example.test"}],
            "phoneNumbers": [{"value": "+12025550125"}]
        })];
        if ambiguous_people_write_committed.load(Ordering::SeqCst) {
            connections.push(serde_json::json!({
                "resourceName": "people/created-contact-1",
                "metadata": {
                    "deleted": false,
                    "sources": [{
                        "type": "CONTACT",
                        "id": "created-contact-1",
                        "etag": "created-etag-3"
                    }]
                },
                "names": [{"displayName": "Local Ambiguous Contact"}],
                "emailAddresses": [{"value": "local-create@example.test"}],
                "phoneNumbers": [{"value": "+12025550199"}]
            }));
        }
        serde_json::to_vec(&serde_json::json!({
            "connections": connections,
            "nextSyncToken": "sync-token-must-not-be-page-token"
        }))
        .expect("encode Google People response")
    } else {
        panic!("unsupported Gmail fixture GET path: {path}");
    };
    write_json_response(stream, &body);
}

fn serve_send(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    path: &str,
    headers: &BTreeMap<String, String>,
    accepted_mutations: &AtomicUsize,
    last_request: &Mutex<Option<GmailSentRequestV1>>,
) {
    assert_eq!(path, "/gmail/v1/users/me/messages/send");
    let body = read_json_body(stream, headers);
    let raw = body
        .get("raw")
        .and_then(serde_json::Value::as_str)
        .expect("Gmail raw message")
        .to_owned();
    let thread_id = body
        .get("threadId")
        .and_then(serde_json::Value::as_str)
        .expect("Gmail thread ID")
        .to_owned();
    let authorization = headers
        .get("authorization")
        .expect("Gmail authorization")
        .to_owned();
    *last_request.lock().expect("lock Gmail fixture request") = Some(GmailSentRequestV1 {
        path: path.to_owned(),
        authorization,
        raw,
        thread_id,
    });
    accepted_mutations.fetch_add(1, Ordering::SeqCst);

    let body = br#"{"id":"gmail-sent-1","threadId":"gmail-thread-1","labelIds":["SENT"]}"#;
    write_json_response(stream, body);
}

fn serve_people_write(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    state: &GmailFixtureState<'_>,
) {
    assert_eq!(
        headers.get("authorization").map(String::as_str),
        Some("Bearer managed-mail-gmail-access-token")
    );
    assert!(
        method == "POST" && path.starts_with("/v1/people:createContact?")
            || method == "PATCH"
                && (path.starts_with("/v1/people/managed-contact-1:updateContact?")
                    || path.starts_with("/v1/people/created-contact-1:updateContact?")),
        "unsupported Google People write route"
    );
    let body = read_json_body(stream, headers);
    *state
        .last_people_write
        .lock()
        .expect("lock Google People write") = Some(GooglePeopleWriteRequestV1 {
        method: method.to_owned(),
        path: path.to_owned(),
        authorization: headers
            .get("authorization")
            .expect("Google People authorization")
            .to_owned(),
        body,
    });
    state.accepted_people_writes.fetch_add(1, Ordering::SeqCst);
    if state
        .drop_next_people_write_response
        .swap(false, Ordering::SeqCst)
    {
        state
            .ambiguous_people_write_committed
            .store(true, Ordering::SeqCst);
        return;
    }
    let (resource_name, id, etag) = if method == "POST" {
        (
            "people/created-contact-1",
            "created-contact-1",
            "created-etag-1",
        )
    } else if path.starts_with("/v1/people/created-contact-1:updateContact?") {
        (
            "people/created-contact-1",
            "created-contact-1",
            "created-etag-2",
        )
    } else {
        (
            "people/managed-contact-1",
            "managed-contact-1",
            "managed-etag-2",
        )
    };
    let response = serde_json::to_vec(&serde_json::json!({
        "resourceName": resource_name,
        "metadata": {
            "sources": [{
                "type": "CONTACT",
                "id": id,
                "etag": etag
            }]
        },
        "names": [{"displayName": "Private Managed Contact"}],
        "emailAddresses": [{"value": "private-managed-contact@example.test"}],
        "phoneNumbers": [{"value": "+12025550125"}]
    }))
    .expect("encode Google People write response");
    write_json_response(stream, &response);
}

fn read_json_body(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    headers: &BTreeMap<String, String>,
) -> serde_json::Value {
    let content_length = headers
        .get("content-length")
        .expect("request content length")
        .parse::<usize>()
        .expect("request content length value");
    assert!(content_length <= MAX_REQUEST_BODY_BYTES);
    let mut body = vec![0_u8; content_length];
    stream.read_exact(&mut body).expect("read request body");
    serde_json::from_slice(&body).expect("decode request body")
}

fn write_json_response(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    body: &[u8],
) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .and_then(|_| stream.write_all(body))
        .and_then(|_| stream.flush())
        .expect("write Gmail response");
}

fn read_line(stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>) -> Vec<u8> {
    let mut line = Vec::new();
    while line.len() <= MAX_HTTP_LINE_BYTES {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).expect("read Gmail request");
        line.push(byte[0]);
        if line.ends_with(b"\r\n") {
            return line;
        }
    }
    panic!("Gmail fixture line exceeded its bound");
}

#[test]
fn inbound_fixture_contains_the_exact_bounded_attachment() {
    let metadata = makosh_mail_core::rfc822::attachment_metadata(GMAIL_INBOUND_MESSAGE);

    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].filename.as_deref(), Some("gmail-evidence.txt"));
    assert_eq!(metadata[0].media_type, "text/plain");
    assert_eq!(
        makosh_mail_core::rfc822::extract_attachment_part(
            GMAIL_INBOUND_MESSAGE,
            metadata[0].part_id,
        ),
        Ok(b"gmail-clean-room-attachment".to_vec())
    );
}

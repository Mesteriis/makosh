//! Loopback TLS CardDAV provider for managed Mail address-book conformance.

use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use rcgen::generate_simple_self_signed;
use rustls::{
    ServerConfig, ServerConnection, StreamOwned,
    pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer},
};

const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_BODY_BYTES: usize = 64 * 1024;

pub(super) struct MailCardDavFixture {
    port: u16,
    ca_certificate_pem: String,
    reports: Arc<AtomicUsize>,
    source_present: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl MailCardDavFixture {
    pub(super) fn start() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified = generate_simple_self_signed(vec!["localhost".to_owned()])
            .expect("generate CardDAV fixture certificate");
        let ca_certificate_pem = certified.cert.pem();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let server = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certified.cert.der().clone()], key)
                .expect("configure CardDAV fixture TLS"),
        );
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind CardDAV fixture");
        listener
            .set_nonblocking(true)
            .expect("configure CardDAV listener");
        let port = listener
            .local_addr()
            .expect("CardDAV fixture address")
            .port();
        let reports = Arc::new(AtomicUsize::new(0));
        let source_present = Arc::new(AtomicBool::new(true));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_reports = Arc::clone(&reports);
        let worker_source_present = Arc::clone(&source_present);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("configure blocking CardDAV fixture connection");
                        stream
                            .set_read_timeout(Some(Duration::from_secs(10)))
                            .expect("CardDAV read timeout");
                        stream
                            .set_write_timeout(Some(Duration::from_secs(10)))
                            .expect("CardDAV write timeout");
                        let connection = ServerConnection::new(Arc::clone(&server))
                            .expect("CardDAV TLS session");
                        serve_connection(
                            &mut StreamOwned::new(connection, stream),
                            &worker_reports,
                            &worker_source_present,
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("CardDAV fixture accept failed: {error}"),
                }
            }
        });
        Self {
            port,
            ca_certificate_pem,
            reports,
            source_present,
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

    pub(super) fn reports(&self) -> usize {
        self.reports.load(Ordering::SeqCst)
    }

    pub(super) fn remove_source(&self) {
        self.source_present.store(false, Ordering::SeqCst);
    }
}

impl Drop for MailCardDavFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let outcome = worker.join();
            if !std::thread::panicking() {
                outcome.expect("join CardDAV fixture");
            }
        }
    }
}

fn serve_connection(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    reports: &AtomicUsize,
    source_present: &AtomicBool,
) {
    let (method, path, headers) = read_request(stream);
    assert_eq!(
        headers.get("authorization").map(String::as_str),
        Some("Basic b3duZXJAZXhhbXBsZS50ZXN0Om1hbmFnZWQtbWFpbC1jYXJkZGF2LXBhc3N3b3Jk")
    );
    let body = match (method.as_str(), path.as_str()) {
        ("PROPFIND", "/") => concat!(
            "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\" ",
            "xmlns:card=\"urn:ietf:params:xml:ns:carddav\"><d:response><d:href>/</d:href>",
            "<d:propstat><d:prop><card:addressbook-home-set><d:href>/contacts/</d:href>",
            "</card:addressbook-home-set></d:prop></d:propstat></d:response></d:multistatus>"
        ),
        ("PROPFIND", "/contacts/") => concat!(
            "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\" ",
            "xmlns:card=\"urn:ietf:params:xml:ns:carddav\"><d:response>",
            "<d:href>/contacts/book/</d:href><d:propstat><d:prop><d:resourcetype>",
            "<d:collection/><card:addressbook/></d:resourcetype></d:prop></d:propstat>",
            "</d:response></d:multistatus>"
        ),
        ("REPORT", "/contacts/book/") => {
            reports.fetch_add(1, Ordering::SeqCst);
            if source_present.load(Ordering::SeqCst) {
                concat!(
                    "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\" ",
                    "xmlns:card=\"urn:ietf:params:xml:ns:carddav\"><d:response>",
                    "<d:href>/contacts/book/managed-1.vcf</d:href><d:propstat><d:prop>",
                    "<d:getetag>\"carddav-etag-1\"</d:getetag><card:address-data>",
                    "BEGIN:VCARD&#13;&#10;VERSION:3.0&#13;&#10;FN:Private CardDAV Contact&#13;&#10;",
                    "EMAIL:private-carddav@example.test&#13;&#10;TEL:+12025550126&#13;&#10;END:VCARD",
                    "</card:address-data></d:prop></d:propstat></d:response></d:multistatus>"
                )
            } else {
                concat!(
                    "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\" ",
                    "xmlns:card=\"urn:ietf:params:xml:ns:carddav\"></d:multistatus>"
                )
            }
        }
        _ => panic!("unsupported CardDAV request: {method} {path}"),
    };
    let response = format!(
        "HTTP/1.1 207 Multi-Status\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .expect("write CardDAV response");
    stream.flush().expect("flush CardDAV response");
}

fn read_request(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
) -> (String, String, BTreeMap<String, String>) {
    let mut bytes = Vec::new();
    let mut byte = [0_u8; 1];
    while !bytes.ends_with(b"\r\n\r\n") {
        assert!(bytes.len() < MAX_HEADER_BYTES, "CardDAV headers too large");
        stream.read_exact(&mut byte).expect("read CardDAV headers");
        bytes.push(byte[0]);
    }
    let header = std::str::from_utf8(&bytes).expect("CardDAV headers UTF-8");
    let mut lines = header.split("\r\n");
    let request_line = lines.next().expect("CardDAV request line");
    let mut request = request_line.split_whitespace();
    let method = request.next().expect("CardDAV method").to_owned();
    let path = request.next().expect("CardDAV path").to_owned();
    assert_eq!(request.next(), Some("HTTP/1.1"));
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').expect("CardDAV header shape");
        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
    }
    let content_length = headers
        .get("content-length")
        .expect("CardDAV content length")
        .parse::<usize>()
        .expect("CardDAV content length value");
    assert!(content_length <= MAX_BODY_BYTES);
    let mut body = vec![0; content_length];
    stream.read_exact(&mut body).expect("read CardDAV body");
    (method, path, headers)
}

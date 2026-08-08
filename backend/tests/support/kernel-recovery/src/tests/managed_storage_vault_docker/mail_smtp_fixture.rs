//! Loopback implicit-TLS SMTP provider used by the managed Mail delivery contour.

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
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

pub(super) struct MailSmtpFixture {
    port: u16,
    ca_certificate_pem: String,
    accepted_messages: Arc<AtomicUsize>,
    last_message: Arc<Mutex<Vec<u8>>>,
    disconnect_after_data: bool,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl MailSmtpFixture {
    pub(super) fn start() -> Self {
        Self::start_with_disconnect_after_data(false)
    }

    pub(super) fn start_outcome_unknown() -> Self {
        Self::start_with_disconnect_after_data(true)
    }

    fn start_with_disconnect_after_data(disconnect_after_data: bool) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let certified = generate_simple_self_signed(vec!["localhost".to_owned()])
            .expect("generate SMTP fixture certificate");
        let ca_certificate_pem = certified.cert.pem();
        let certificate = certified.cert.der().clone();
        let key =
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
        let server = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], key)
                .expect("configure SMTP fixture TLS"),
        );
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind SMTP fixture");
        listener
            .set_nonblocking(true)
            .expect("configure SMTP fixture listener");
        let port = listener.local_addr().expect("SMTP fixture address").port();
        let accepted_messages = Arc::new(AtomicUsize::new(0));
        let last_message = Arc::new(Mutex::new(Vec::new()));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_accepted = Arc::clone(&accepted_messages);
        let worker_message = Arc::clone(&last_message);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream
                            .set_nonblocking(false)
                            .expect("configure blocking SMTP fixture connection");
                        stream
                            .set_read_timeout(Some(Duration::from_secs(10)))
                            .expect("configure SMTP fixture read timeout");
                        stream
                            .set_write_timeout(Some(Duration::from_secs(10)))
                            .expect("configure SMTP fixture write timeout");
                        let connection =
                            ServerConnection::new(Arc::clone(&server)).expect("SMTP TLS session");
                        let mut stream = StreamOwned::new(connection, stream);
                        serve_connection(
                            &mut stream,
                            &worker_accepted,
                            &worker_message,
                            disconnect_after_data,
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("SMTP fixture accept failed: {error}"),
                }
            }
        });
        Self {
            port,
            ca_certificate_pem,
            accepted_messages,
            last_message,
            disconnect_after_data,
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

    pub(super) fn accepted_messages(&self) -> usize {
        self.accepted_messages.load(Ordering::SeqCst)
    }

    pub(super) fn last_message(&self) -> Vec<u8> {
        self.last_message
            .lock()
            .expect("lock SMTP fixture message")
            .clone()
    }

    pub(super) fn disconnects_after_data(&self) -> bool {
        self.disconnect_after_data
    }
}

impl Drop for MailSmtpFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let outcome = worker.join();
            if !std::thread::panicking() {
                outcome.expect("join SMTP fixture");
            }
        }
    }
}

fn serve_connection(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    accepted_messages: &AtomicUsize,
    last_message: &Mutex<Vec<u8>>,
    disconnect_after_data: bool,
) {
    write_response(stream, b"220 localhost ESMTP makosh fixture\r\n");
    expect_prefix(stream, b"EHLO ");
    write_response(stream, b"250-localhost\r\n250 AUTH PLAIN\r\n");
    expect_prefix(stream, b"AUTH PLAIN ");
    write_response(stream, b"235 2.7.0 authenticated\r\n");
    expect_prefix(stream, b"MAIL FROM:<");
    write_response(stream, b"250 2.1.0 sender accepted\r\n");
    expect_prefix(stream, b"RCPT TO:<");
    write_response(stream, b"250 2.1.5 recipient accepted\r\n");
    assert_eq!(read_line(stream), b"DATA\r\n");
    write_response(stream, b"354 end with <CRLF>.<CRLF>\r\n");
    let mut message = Vec::new();
    loop {
        let line = read_line(stream);
        if line == b".\r\n" {
            break;
        }
        message.extend_from_slice(&line);
    }
    *last_message.lock().expect("lock SMTP fixture message") = message;
    accepted_messages.fetch_add(1, Ordering::SeqCst);
    if disconnect_after_data {
        return;
    }
    write_response(stream, b"250 2.0.0 queued\r\n");
    assert_eq!(read_line(stream), b"QUIT\r\n");
    write_response(stream, b"221 2.0.0 closing\r\n");
}

fn expect_prefix(stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>, prefix: &[u8]) {
    let line = read_line(stream);
    assert!(
        line.starts_with(prefix),
        "SMTP fixture received unexpected command"
    );
}

fn read_line(stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>) -> Vec<u8> {
    let mut line = Vec::new();
    while line.len() <= 4 * 1024 {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).expect("read SMTP command");
        line.push(byte[0]);
        if line.ends_with(b"\r\n") {
            return line;
        }
    }
    panic!("SMTP fixture command exceeded its bound");
}

fn write_response(
    stream: &mut StreamOwned<ServerConnection, std::net::TcpStream>,
    response: &[u8],
) {
    stream.write_all(response).expect("write SMTP response");
    stream.flush().expect("flush SMTP response");
}

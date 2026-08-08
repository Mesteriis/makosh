//! Loopback-only IMAP protocol fixture for the feature-gated managed Mail conformance binary.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const FIXTURE_USERNAME: &str = "owner@example.test";
const FIXTURE_PASSWORD: &str = "managed-mail-imap-password";
const FIXTURE_UID: u32 = 42;
const STREAMING_FIRST_UID: u32 = 41;
const FIXTURE_UID_VALIDITY: u32 = 1;
const ARCHIVE_UID_VALIDITY: u32 = 7;
const ARCHIVE_UID: u32 = 84;
const TRASH_UID_VALIDITY: u32 = 9;
const TRASH_UID: u32 = 126;
const FIXTURE_MESSAGE: &[u8] = concat!(
    "From: source@example.test\r\n",
    "To: owner@example.test\r\n",
    "Subject: managed attachment evidence\r\n",
    "Content-Type: multipart/mixed; boundary=makosh-fixture\r\n",
    "\r\n",
    "--makosh-fixture\r\n",
    "Content-Type: text/plain; charset=utf-8\r\n",
    "\r\n",
    "managed Mail body\r\n",
    "--makosh-fixture\r\n",
    "Content-Type: application/pdf; name=evidence.pdf\r\n",
    "Content-Disposition: attachment; filename=evidence.pdf\r\n",
    "Content-Transfer-Encoding: base64\r\n",
    "\r\n",
    "Y2xlYW4tcm9vbS1hdHRhY2htZW50\r\n",
    "--makosh-fixture--\r\n",
)
.as_bytes();

#[derive(Clone)]
struct MailImapFixtureState {
    message_flag_mutations: Arc<AtomicUsize>,
    message_location_mutations: Arc<AtomicUsize>,
    message_permanent_deletions: Arc<AtomicUsize>,
    message_mailbox: Arc<Mutex<String>>,
    message_uid: Arc<AtomicU32>,
    move_supported: Arc<AtomicBool>,
    uid_validity: Arc<AtomicU32>,
    delete_marked: Arc<AtomicBool>,
    streaming_sync_pages: Arc<AtomicBool>,
    second_page_requested: Arc<AtomicBool>,
    release_second_page: Arc<AtomicBool>,
}

pub(super) struct MailImapFixture {
    port: u16,
    shutdown: Arc<AtomicBool>,
    accepted_connections: Arc<AtomicUsize>,
    state: MailImapFixtureState,
    worker: Option<JoinHandle<()>>,
}

impl MailImapFixture {
    pub(super) fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback IMAP fixture");
        listener
            .set_nonblocking(true)
            .expect("configure loopback IMAP fixture");
        let port = listener
            .local_addr()
            .expect("read loopback IMAP fixture address")
            .port();
        let shutdown = Arc::new(AtomicBool::new(false));
        let accepted_connections = Arc::new(AtomicUsize::new(0));
        let state = MailImapFixtureState {
            message_flag_mutations: Arc::new(AtomicUsize::new(0)),
            message_location_mutations: Arc::new(AtomicUsize::new(0)),
            message_permanent_deletions: Arc::new(AtomicUsize::new(0)),
            message_mailbox: Arc::new(Mutex::new("INBOX".to_owned())),
            message_uid: Arc::new(AtomicU32::new(FIXTURE_UID)),
            move_supported: Arc::new(AtomicBool::new(true)),
            uid_validity: Arc::new(AtomicU32::new(FIXTURE_UID_VALIDITY)),
            delete_marked: Arc::new(AtomicBool::new(false)),
            streaming_sync_pages: Arc::new(AtomicBool::new(false)),
            second_page_requested: Arc::new(AtomicBool::new(false)),
            release_second_page: Arc::new(AtomicBool::new(false)),
        };
        let worker_shutdown = Arc::clone(&shutdown);
        let worker_connections = Arc::clone(&accepted_connections);
        let worker_state = state.clone();
        let worker = thread::spawn(move || {
            while !worker_shutdown.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if worker_shutdown.load(Ordering::Acquire) {
                            break;
                        }
                        worker_connections.fetch_add(1, Ordering::AcqRel);
                        serve_connection(stream, worker_state.clone());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => panic!("accept loopback IMAP fixture connection"),
                }
            }
        });
        Self {
            port,
            shutdown,
            accepted_connections,
            state,
            worker: Some(worker),
        }
    }

    pub(super) const fn port(&self) -> u16 {
        self.port
    }

    pub(super) fn accepted_connections(&self) -> usize {
        self.accepted_connections.load(Ordering::Acquire)
    }

    pub(super) fn enable_streaming_sync_pages(&self) {
        self.state
            .second_page_requested
            .store(false, Ordering::Release);
        self.state
            .release_second_page
            .store(false, Ordering::Release);
        self.state
            .streaming_sync_pages
            .store(true, Ordering::Release);
    }

    pub(super) fn second_page_requested(&self) -> bool {
        self.state.second_page_requested.load(Ordering::Acquire)
    }

    pub(super) fn release_second_page(&self) {
        self.state
            .release_second_page
            .store(true, Ordering::Release);
    }

    pub(super) fn message_flag_mutations(&self) -> usize {
        self.state.message_flag_mutations.load(Ordering::Acquire)
    }

    pub(super) fn message_location_mutations(&self) -> usize {
        self.state
            .message_location_mutations
            .load(Ordering::Acquire)
    }

    pub(super) fn message_permanent_deletions(&self) -> usize {
        self.state
            .message_permanent_deletions
            .load(Ordering::Acquire)
    }

    pub(super) fn message_mailbox(&self) -> String {
        self.state
            .message_mailbox
            .lock()
            .expect("lock fixture message mailbox")
            .clone()
    }

    pub(super) fn set_move_supported(&self, supported: bool) {
        self.state
            .move_supported
            .store(supported, Ordering::Release);
    }

    pub(super) fn set_uid_validity(&self, uid_validity: u32) {
        assert!(uid_validity > 0, "fixture UIDVALIDITY must be positive");
        self.state
            .uid_validity
            .store(uid_validity, Ordering::Release);
    }
}

impl Drop for MailImapFixture {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(worker) = self.worker.take() {
            let result = worker.join();
            if !std::thread::panicking() {
                result.expect("join loopback IMAP fixture");
            }
        }
    }
}

fn serve_connection(mut stream: TcpStream, state: MailImapFixtureState) {
    let MailImapFixtureState {
        message_flag_mutations,
        message_location_mutations,
        message_permanent_deletions,
        message_mailbox,
        message_uid,
        move_supported,
        uid_validity,
        delete_marked,
        streaming_sync_pages,
        second_page_requested,
        release_second_page,
    } = state;
    stream
        .set_nonblocking(false)
        .and_then(|_| stream.set_read_timeout(Some(Duration::from_secs(15))))
        .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(15))))
        .expect("configure loopback IMAP fixture connection");
    stream
        .write_all(b"* OK makosh IMAP4rev1 fixture ready\r\n")
        .expect("write IMAP fixture greeting");
    let reader_stream = stream.try_clone().expect("clone fixture read stream");
    let mut lines = BufReader::new(reader_stream).lines();
    while let Some(command) = lines.next().transpose().expect("read IMAP fixture command") {
        let tag = command.split_whitespace().next().expect("IMAP command tag");
        let upper = command.to_ascii_uppercase();
        if upper.contains(" LOGIN ") {
            if !command.contains(FIXTURE_USERNAME) || !command.contains(FIXTURE_PASSWORD) {
                write_tagged(&mut stream, tag, "NO authentication failed");
                continue;
            }
            write_tagged(&mut stream, tag, "OK LOGIN completed");
        } else if upper.contains(" CAPABILITY") {
            let extensions = if move_supported.load(Ordering::Acquire) {
                " MOVE UIDPLUS"
            } else {
                ""
            };
            write!(
                stream,
                "* CAPABILITY IMAP4rev1{extensions}\r\n{tag} OK CAPABILITY completed\r\n"
            )
            .expect("write IMAP CAPABILITY response");
        } else if upper.contains(" LIST ") {
            write!(
                stream,
                "* LIST (\\HasNoChildren \\Inbox) \"/\" \"INBOX\"\r\n\
                 * LIST (\\HasNoChildren \\Archive) \"/\" \"Archive\"\r\n\
                 * LIST (\\HasNoChildren \\Trash) \"/\" \"Trash\"\r\n\
                 {tag} OK LIST completed\r\n"
            )
            .expect("write IMAP LIST response");
        } else if upper.contains(" EXAMINE ") {
            let current_uid_validity = uid_validity.load(Ordering::Acquire);
            write!(
                stream,
                "* FLAGS (\\Seen)\r\n* 1 EXISTS\r\n* OK [UIDVALIDITY {current_uid_validity}] valid\r\n\
                 {tag} OK [READ-ONLY] EXAMINE completed\r\n"
            )
            .expect("write IMAP EXAMINE response");
        } else if upper.contains(" SELECT ") {
            let current_uid_validity = uid_validity.load(Ordering::Acquire);
            let selected_mailbox = command
                .split_whitespace()
                .nth(2)
                .map(|value| value.trim_matches('"'))
                .expect("selected fixture mailbox");
            assert_eq!(
                selected_mailbox,
                message_mailbox
                    .lock()
                    .expect("lock fixture message mailbox")
                    .as_str(),
                "Mail mutation must select the exact current locator mailbox",
            );
            write!(
                stream,
                "* FLAGS (\\Seen \\Flagged)\r\n* 1 EXISTS\r\n* OK [UIDVALIDITY {current_uid_validity}] valid\r\n\
                 {tag} OK [READ-WRITE] SELECT completed\r\n"
            )
            .expect("write IMAP SELECT response");
        } else if upper.contains(" UID SEARCH ") {
            let current_uid = message_uid.load(Ordering::Acquire);
            if streaming_sync_pages.load(Ordering::Acquire) {
                write!(
                    stream,
                    "* SEARCH {STREAMING_FIRST_UID} {current_uid}\r\n\
                     {tag} OK UID SEARCH completed\r\n"
                )
                .expect("write streaming IMAP UID SEARCH response");
            } else {
                write!(
                    stream,
                    "* SEARCH {current_uid}\r\n{tag} OK UID SEARCH completed\r\n"
                )
                .expect("write IMAP UID SEARCH response");
            }
        } else if upper.contains(" UID FETCH ") {
            let requested_uid = command
                .split_whitespace()
                .nth(3)
                .and_then(|value| value.parse::<u32>().ok())
                .expect("exact fixture UID FETCH");
            if streaming_sync_pages.load(Ordering::Acquire) && requested_uid == FIXTURE_UID {
                second_page_requested.store(true, Ordering::Release);
                let deadline = Instant::now() + Duration::from_secs(10);
                while !release_second_page.load(Ordering::Acquire) && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(5));
                }
                assert!(
                    release_second_page.load(Ordering::Acquire),
                    "managed test did not release the second IMAP page"
                );
            }
            write!(
                stream,
                "* 1 FETCH (UID {requested_uid} RFC822.SIZE {} INTERNALDATE \
                 \"24-Jul-2026 12:00:00 +0000\" BODY[] {{{}}}\r\n",
                FIXTURE_MESSAGE.len(),
                FIXTURE_MESSAGE.len(),
            )
            .and_then(|_| stream.write_all(FIXTURE_MESSAGE))
            .and_then(|_| write!(stream, ")\r\n{tag} OK UID FETCH completed\r\n"))
            .expect("write IMAP UID FETCH response");
        } else if upper.contains(" UID STORE ") {
            let current_uid = message_uid.load(Ordering::Acquire);
            assert!(
                upper.contains(&format!("UID STORE {current_uid}"))
                    && upper.contains("FLAGS.SILENT"),
                "Mail mutation must use exact bounded UID and silent flags"
            );
            if upper.contains("\\DELETED") {
                delete_marked.store(true, Ordering::Release);
            } else {
                assert!(
                    upper.contains("\\SEEN") || upper.contains("\\FLAGGED"),
                    "Mail flag mutation must use a supported provider flag"
                );
                message_flag_mutations.fetch_add(1, Ordering::AcqRel);
            }
            write_tagged(&mut stream, tag, "OK UID STORE completed");
        } else if upper.contains(" UID EXPUNGE ") {
            let current_uid = message_uid.load(Ordering::Acquire);
            assert_eq!(
                upper.trim(),
                format!("{tag} UID EXPUNGE {current_uid}").to_ascii_uppercase(),
                "permanent delete must use exact UID EXPUNGE"
            );
            assert!(
                delete_marked.swap(false, Ordering::AcqRel),
                "UID EXPUNGE requires the same message to be marked Deleted first"
            );
            message_permanent_deletions.fetch_add(1, Ordering::AcqRel);
            write!(stream, "* 1 EXPUNGE\r\n{tag} OK UID EXPUNGE completed\r\n")
                .expect("write IMAP UID EXPUNGE response");
        } else if upper.contains(" UID MOVE ") {
            let current_uid = message_uid.load(Ordering::Acquire);
            assert!(
                upper.contains(&format!("UID MOVE {current_uid}")),
                "Mail location mutation must use the exact current locator UID",
            );
            let target = command
                .split_whitespace()
                .last()
                .map(|value| value.trim_matches('"'))
                .expect("fixture UID MOVE target");
            let (destination_uid_validity, destination_uid) = match target {
                "INBOX" => (FIXTURE_UID_VALIDITY, FIXTURE_UID),
                "Archive" => (ARCHIVE_UID_VALIDITY, ARCHIVE_UID),
                "Trash" => (TRASH_UID_VALIDITY, TRASH_UID),
                _ => panic!("unexpected fixture UID MOVE target"),
            };
            message_location_mutations.fetch_add(1, Ordering::AcqRel);
            *message_mailbox
                .lock()
                .expect("lock fixture message mailbox") = target.to_owned();
            message_uid.store(destination_uid, Ordering::Release);
            uid_validity.store(destination_uid_validity, Ordering::Release);
            write!(
                stream,
                "* OK [COPYUID {destination_uid_validity} {current_uid} {destination_uid}] moved\r\n\
                 * 1 EXPUNGE\r\n{tag} OK UID MOVE completed\r\n"
            )
            .expect("write IMAP UID MOVE response");
        } else if upper.contains(" LOGOUT") {
            write!(
                stream,
                "* BYE fixture complete\r\n{tag} OK LOGOUT completed\r\n"
            )
            .expect("write IMAP LOGOUT response");
            break;
        } else {
            write_tagged(&mut stream, tag, "BAD unsupported fixture command");
        }
        stream.flush().expect("flush IMAP fixture response");
    }
}

fn write_tagged(stream: &mut TcpStream, tag: &str, response: &str) {
    writeln!(stream, "{tag} {response}\r").expect("write tagged IMAP fixture response");
    stream.flush().expect("flush tagged IMAP fixture response");
}

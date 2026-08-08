//! IMAP provider adapter boundary for ADR-0239, ADR-0307 and ADR-0308.
//!
//! Sync discovers bounded selectable mailboxes and captures the selected
//! mailbox UIDVALIDITY. Convergent flag mutations are fenced by the private
//! mailbox/UIDVALIDITY/UID locator before `UID STORE`.

#![allow(clippy::items_after_test_module)]

use std::fmt::{Debug, Display, Formatter};
use std::time::{Duration, Instant};

use async_imap::{
    Session,
    types::{Flag, NameAttribute},
};
#[cfg(not(feature = "conformance-test-support"))]
use async_native_tls::TlsConnector;
use async_std::future;
use async_std::net::TcpStream;
use async_std::task;
use futures_util::TryStreamExt;
use futures_util::io::{AsyncRead, AsyncWrite};
use hermes_mail_api::{MAX_PLAIN_TEXT_BYTES, MAX_WINDOW, MAX_WINDOWS, WINDOW_DEADLINE_SECONDS};
use hermes_mail_core::rfc822::{
    AttachmentDispositionV1, Rfc822BodyContentV1, attachment_metadata, extract_attachment_part,
    operational_preview, readable_body_content, readable_text_body,
};
use imap_proto::{
    Response, Status,
    types::{ResponseCode, UidSetMember},
};

pub const PACKAGE: &str = "hermes-mail-imap";

#[cfg(not(feature = "conformance-test-support"))]
type ImapTransport = async_native_tls::TlsStream<TcpStream>;
#[cfg(feature = "conformance-test-support")]
type ImapTransport = TcpStream;

mod retry {
    #[derive(Clone, Copy)]
    pub struct ImapRetryPolicy {
        pub max_attempts: u8,
        pub delay_millis: u64,
    }

    // Retry policy is explicitly defined as policy data to make future timeout/attempt tuning
    // visible and testable without changing IMAP parsing/fetching logic.
    pub const MAX_SYNC_ATTEMPTS: u8 = 3;
    pub const RETRY_DELAY_MILLIS: u64 = 120;
    pub const IMAP_SYNC_RETRY_POLICY: ImapRetryPolicy = ImapRetryPolicy {
        max_attempts: MAX_SYNC_ATTEMPTS,
        delay_millis: RETRY_DELAY_MILLIS,
    };
}

pub const MAX_ATTEMPTS: u8 = retry::IMAP_SYNC_RETRY_POLICY.max_attempts;

const IMAP_UID_FETCH_CHUNK_SIZE: usize = 10;
const IMAP_SYNC_FINALIZATION_PAGE_SIZE: usize = 20;
const IMAP_UID_FETCH_TIMEOUT_SECONDS: u64 = WINDOW_DEADLINE_SECONDS;
const IMAP_SYNC_TIMEOUT_SECONDS: u64 = 300;
const SNAPSHOT_PREVIEW_BYTES: usize = 160;
const MAX_DISCOVERED_MAILBOXES: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct ImapMessage {
    pub uid: u32,
    pub subject: String,
    pub sender: Option<String>,
    pub recipients: Vec<String>,
    pub snippet: String,
    pub sent_at_unix_seconds: Option<i64>,
    pub flags: Vec<ImapMessageFlag>,
    pub has_plain_text: bool,
    pub plain_text_body: Option<Vec<u8>>,
    pub body_content: Option<Rfc822BodyContentV1>,
    pub attachments: Vec<ImapAttachment>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapMessageFlag {
    Read,
    Starred,
    Draft,
    Trashed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapMutableMessageFlagV1 {
    Read,
    Starred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapMailboxKindV1 {
    Inbox,
    Archive,
    Trash,
    Sent,
    Drafts,
    Spam,
    All,
    ProviderFolder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapMailboxV1 {
    pub mailbox_id: String,
    pub display_name: String,
    pub kind: ImapMailboxKindV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapSelectedMailboxV1 {
    pub mailbox_id: String,
    pub uid_validity: u32,
}

pub struct ImapMessageFlagAccessV1<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: &'a str,
    pub password: &'a str,
}

pub struct ImapMessageLocationAccessV1<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: &'a str,
    pub password: &'a str,
}

pub struct ImapMessageLocatorV1<'a> {
    pub mailbox_id: &'a str,
    pub uid_validity: u32,
    pub uid: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapMessageLocationResultV1 {
    pub mailbox_id: String,
    pub uid_validity: u32,
    pub uid: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImapAttachmentDisposition {
    Attachment,
    Inline,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ImapAttachment {
    pub part_id: u16,
    pub filename: Option<String>,
    pub media_type: String,
    pub declared_bytes: u64,
    pub disposition: ImapAttachmentDisposition,
    bytes: Vec<u8>,
}

impl ImapAttachment {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Debug for ImapAttachment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ImapAttachment")
            .field("part_id", &self.part_id)
            .field("filename", &self.filename)
            .field("media_type", &self.media_type)
            .field("declared_bytes", &self.declared_bytes)
            .field("disposition", &self.disposition)
            .field("byte_length", &self.bytes.len())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct ImapSyncResult {
    pub messages: Vec<ImapMessage>,
    pub mailboxes: Vec<ImapMailboxV1>,
    pub selected_mailbox: ImapSelectedMailboxV1,
    pub attempts: u8,
    pub window: u32,
    pub has_more: bool,
}

#[derive(Debug)]
pub struct ImapError {
    kind: &'static str,
    message: String,
}

impl Display for ImapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ImapError {}

impl ImapError {
    fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn is_definite_rejection(&self) -> bool {
        matches!(
            self.kind,
            "validation" | "stale_locator" | "provider_rejected"
        )
    }

    #[must_use]
    pub fn is_unsupported(&self) -> bool {
        self.kind == "unsupported"
    }

    fn is_retryable(&self) -> bool {
        matches!(self.kind, "network" | "timeout")
    }
}

pub fn sync_inbox<F>(
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    window: u32,
    windows: u32,
    finalize_page: F,
) -> Result<usize, String>
where
    F: FnMut(ImapSyncResult) -> Result<(), ()>,
{
    sync_inbox_prioritized(
        host,
        port,
        username,
        password,
        window,
        windows,
        &[],
        finalize_page,
    )
}

pub fn sync_inbox_prioritized<F>(
    host: &str,
    port: u16,
    username: &str,
    password: Option<&str>,
    window: u32,
    windows: u32,
    priority_uids: &[u32],
    mut finalize_page: F,
) -> Result<usize, String>
where
    F: FnMut(ImapSyncResult) -> Result<(), ()>,
{
    let password = password.ok_or_else(|| "imap password is required".to_owned())?;
    if !supports_read_only_sync(window) || !supports_read_only_windows(windows) {
        return Err("window unsupported for read-only sync".to_owned());
    }
    let limit = usize::try_from(window as u64 * windows as u64)
        .map_err(|_| "imap requested window does not fit runtime limits".to_owned())?;
    let page_size = sync_finalization_page_size(window)?;
    task::block_on(future::timeout(
        Duration::from_secs(IMAP_SYNC_TIMEOUT_SECONDS),
        imap_sync_pages_once(
            host,
            port,
            username,
            password,
            limit,
            page_size,
            priority_uids,
            &mut finalize_page,
        ),
    ))
    .map_err(|_| format!("imap sync exceeded {IMAP_SYNC_TIMEOUT_SECONDS}s deadline"))?
    .map_err(|error| format!("imap sync failed: {error}"))
}

pub fn set_message_flag(
    access: ImapMessageFlagAccessV1<'_>,
    locator: ImapMessageLocatorV1<'_>,
    flag: ImapMutableMessageFlagV1,
    target_value: bool,
) -> Result<(), ImapError> {
    if locator.uid == 0
        || locator.uid_validity == 0
        || !valid_mailbox_id(locator.mailbox_id)
        || access.username.trim().is_empty()
        || access.password.is_empty()
    {
        return Err(ImapError::new(
            "validation",
            "imap message flag mutation input is invalid",
        ));
    }
    task::block_on(async move {
        let mut session =
            open_session(access.host, access.port, access.username, access.password).await?;
        let selected = session.select(locator.mailbox_id).await.map_err(|error| {
            ImapError::new("protocol", format!("imap SELECT mailbox failed: {error}"))
        })?;
        if selected.uid_validity != Some(locator.uid_validity) {
            return Err(ImapError::new(
                "stale_locator",
                "imap mailbox UIDVALIDITY does not match the stored locator",
            ));
        }
        let flag_name = match flag {
            ImapMutableMessageFlagV1::Read => "\\Seen",
            ImapMutableMessageFlagV1::Starred => "\\Flagged",
        };
        let operation = if target_value {
            format!("+FLAGS.SILENT ({flag_name})")
        } else {
            format!("-FLAGS.SILENT ({flag_name})")
        };
        let updates = session
            .uid_store(locator.uid.to_string(), operation)
            .await
            .map_err(|error| {
                ImapError::new("protocol", format!("imap UID STORE failed: {error}"))
            })?;
        let _: Vec<_> = updates.try_collect().await.map_err(|error| {
            ImapError::new(
                "protocol",
                format!("imap UID STORE response failed: {error}"),
            )
        })?;
        session
            .logout()
            .await
            .map_err(|error| ImapError::new("protocol", format!("imap logout failed: {error}")))?;
        Ok(())
    })
}

pub fn move_message(
    access: ImapMessageLocationAccessV1<'_>,
    locator: ImapMessageLocatorV1<'_>,
    destination_mailbox_id: &str,
) -> Result<ImapMessageLocationResultV1, ImapError> {
    if locator.uid == 0
        || locator.uid_validity == 0
        || !valid_mailbox_id(locator.mailbox_id)
        || !valid_mailbox_id(destination_mailbox_id)
        || access.username.trim().is_empty()
        || access.password.is_empty()
    {
        return Err(ImapError::new(
            "validation",
            "imap message location mutation input is invalid",
        ));
    }
    if locator.mailbox_id == destination_mailbox_id {
        return Ok(ImapMessageLocationResultV1 {
            mailbox_id: destination_mailbox_id.to_owned(),
            uid_validity: locator.uid_validity,
            uid: locator.uid,
        });
    }
    task::block_on(async move {
        let mut session =
            open_session(access.host, access.port, access.username, access.password).await?;
        let capabilities = session.capabilities().await.map_err(|_| {
            ImapError::new("protocol", "imap CAPABILITY failed before message move")
        })?;
        if !capabilities.has_str("MOVE") || !capabilities.has_str("UIDPLUS") {
            return Err(ImapError::new(
                "unsupported",
                "imap server does not advertise MOVE and UIDPLUS",
            ));
        }
        let selected = session
            .select(locator.mailbox_id)
            .await
            .map_err(|_| ImapError::new("protocol", "imap SELECT failed before message move"))?;
        if selected.uid_validity != Some(locator.uid_validity) {
            return Err(ImapError::new(
                "stale_locator",
                "imap mailbox UIDVALIDITY does not match the stored locator",
            ));
        }
        let request_id = session
            .run_command(format!(
                "UID MOVE {} {}",
                locator.uid,
                imap_mailbox_argument(destination_mailbox_id)
            ))
            .await
            .map_err(|_| ImapError::new("protocol", "imap UID MOVE write failed"))?;
        let mut destination = None;
        loop {
            let response = session
                .read_response()
                .await
                .map_err(|_| ImapError::new("protocol", "imap UID MOVE response failed"))?
                .ok_or_else(|| {
                    ImapError::new("protocol", "imap connection closed during UID MOVE")
                })?;
            match response.parsed() {
                Response::Data {
                    code: Some(ResponseCode::CopyUid(uid_validity, source, target)),
                    ..
                }
                | Response::Done {
                    code: Some(ResponseCode::CopyUid(uid_validity, source, target)),
                    ..
                } => {
                    let Some(target_uid) = exact_single_uid(target) else {
                        return Err(ImapError::new(
                            "protocol",
                            "imap UID MOVE returned an inexact COPYUID mapping",
                        ));
                    };
                    if exact_single_uid(source) != Some(locator.uid) {
                        return Err(ImapError::new(
                            "protocol",
                            "imap UID MOVE returned an inexact COPYUID mapping",
                        ));
                    }
                    destination = Some((*uid_validity, target_uid));
                }
                Response::Done { tag, status, .. } if tag == &request_id => {
                    if status != &Status::Ok {
                        return Err(ImapError::new(
                            "protocol",
                            "imap UID MOVE did not complete successfully",
                        ));
                    }
                    break;
                }
                _ => {}
            }
        }
        let (uid_validity, uid) = destination.ok_or_else(|| {
            ImapError::new(
                "protocol",
                "imap UID MOVE succeeded without exact COPYUID evidence",
            )
        })?;
        let _ = session.logout().await;
        Ok(ImapMessageLocationResultV1 {
            mailbox_id: destination_mailbox_id.to_owned(),
            uid_validity,
            uid,
        })
    })
}

pub fn permanently_delete_message(
    access: ImapMessageLocationAccessV1<'_>,
    locator: ImapMessageLocatorV1<'_>,
) -> Result<(), ImapError> {
    if locator.uid == 0
        || locator.uid_validity == 0
        || !valid_mailbox_id(locator.mailbox_id)
        || access.username.trim().is_empty()
        || access.password.is_empty()
    {
        return Err(ImapError::new(
            "validation",
            "imap permanent delete input is invalid",
        ));
    }
    task::block_on(async move {
        let mut session =
            open_session(access.host, access.port, access.username, access.password).await?;
        let capabilities = session.capabilities().await.map_err(|_| {
            ImapError::new("protocol", "imap CAPABILITY failed before permanent delete")
        })?;
        if !capabilities.has_str("UIDPLUS") {
            return Err(ImapError::new(
                "unsupported",
                "imap server does not advertise UIDPLUS",
            ));
        }
        let selected = session.select(locator.mailbox_id).await.map_err(|_| {
            ImapError::new("protocol", "imap SELECT failed before permanent delete")
        })?;
        if selected.uid_validity != Some(locator.uid_validity) {
            return Err(ImapError::new(
                "stale_locator",
                "imap mailbox UIDVALIDITY does not match the stored locator",
            ));
        }
        run_exact_ok_command(
            &mut session,
            format!("UID STORE {} +FLAGS.SILENT (\\Deleted)", locator.uid),
            "UID STORE",
        )
        .await?;
        run_exact_ok_command(
            &mut session,
            format!("UID EXPUNGE {}", locator.uid),
            "UID EXPUNGE",
        )
        .await?;
        session
            .logout()
            .await
            .map_err(|_| ImapError::new("protocol", "imap logout failed after permanent delete"))?;
        Ok(())
    })
}

async fn run_exact_ok_command<T>(
    session: &mut Session<T>,
    command: String,
    operation: &str,
) -> Result<(), ImapError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    let request_id = session
        .run_command(command)
        .await
        .map_err(|_| ImapError::new("protocol", format!("imap {operation} write failed")))?;
    loop {
        let response = session
            .read_response()
            .await
            .map_err(|_| ImapError::new("protocol", format!("imap {operation} response failed")))?
            .ok_or_else(|| {
                ImapError::new(
                    "protocol",
                    format!("imap connection closed during {operation}"),
                )
            })?;
        if let Response::Done { tag, status, .. } = response.parsed()
            && tag == &request_id
        {
            return if matches!(status, &Status::Ok) {
                Ok(())
            } else {
                Err(ImapError::new(
                    "protocol",
                    format!("imap {operation} did not complete successfully"),
                ))
            };
        }
    }
}

#[cfg(test)]
fn sync_inbox_with_retry<F>(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    limit: u64,
    attempt: F,
) -> Result<ImapSyncResult, String>
where
    F: FnMut(&str, u16, &str, &str, usize, Duration) -> Result<ImapSyncResult, ImapError>,
{
    let attempted_limit = usize::try_from(limit)
        .map_err(|_| "imap requested window does not fit runtime limits".to_owned())?;
    sync_inbox_with_retry_policy(
        host,
        port,
        username,
        password,
        attempted_limit,
        retry::IMAP_SYNC_RETRY_POLICY,
        attempt,
    )
}

#[cfg(test)]
fn sync_inbox_with_retry_policy<F>(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    limit: usize,
    policy: retry::ImapRetryPolicy,
    mut attempt: F,
) -> Result<ImapSyncResult, String>
where
    F: FnMut(&str, u16, &str, &str, usize, Duration) -> Result<ImapSyncResult, ImapError>,
{
    let mut attempts = 0u8;
    let started_at = std::time::Instant::now();
    let sync_deadline = Duration::from_secs(IMAP_SYNC_TIMEOUT_SECONDS);
    while attempts < policy.max_attempts {
        let Some(remaining) = sync_deadline.checked_sub(started_at.elapsed()) else {
            return Err(format!(
                "imap sync exceeded {IMAP_SYNC_TIMEOUT_SECONDS}s deadline"
            ));
        };
        attempts += 1;
        match attempt(host, port, username, password, limit, remaining) {
            Ok(result) => {
                return Ok(ImapSyncResult {
                    attempts,
                    window: result.window,
                    messages: result.messages,
                    mailboxes: result.mailboxes,
                    selected_mailbox: result.selected_mailbox,
                    has_more: result.has_more,
                });
            }
            Err(error) => {
                eprintln!("imap sync attempt {attempts} failed: {error}");
                if error.is_retryable() && attempts < policy.max_attempts {
                    let delay = Duration::from_millis(policy.delay_millis);
                    if started_at.elapsed().saturating_add(delay) >= sync_deadline {
                        return Err(format!(
                            "imap sync exceeded {IMAP_SYNC_TIMEOUT_SECONDS}s deadline"
                        ));
                    }
                    std::thread::sleep(delay);
                    continue;
                }
                return Err(format!("imap sync failed: {error}"));
            }
        }
    }
    Err("imap sync failed: unexpected retry loop termination".to_owned())
}

pub fn supports_read_only_sync(window: u32) -> bool {
    window > 0 && window <= MAX_WINDOW
}

pub fn supports_read_only_windows(windows: u32) -> bool {
    windows > 0 && windows <= MAX_WINDOWS
}

fn sync_finalization_page_size(window: u32) -> Result<usize, String> {
    usize::try_from(window)
        .map(|window| window.min(IMAP_SYNC_FINALIZATION_PAGE_SIZE))
        .map_err(|_| "imap page window does not fit runtime limits".to_owned())
}

#[cfg(test)]
#[test]
fn supports_read_only_sync_uses_mail_window_limit_only() {
    assert!(supports_read_only_sync(MAX_WINDOW));
    assert!(!supports_read_only_sync(MAX_WINDOW + 1));
}

#[cfg(test)]
#[test]
fn sync_finalization_page_is_bounded_independently_of_the_total_window() {
    assert_eq!(
        sync_finalization_page_size(MAX_WINDOW).expect("supported Mail window"),
        IMAP_SYNC_FINALIZATION_PAGE_SIZE
    );
    assert_eq!(sync_finalization_page_size(1), Ok(1));
}

struct ImapSyncPlanV1 {
    session: Session<ImapTransport>,
    mailboxes: Vec<ImapMailboxV1>,
    selected_mailbox: ImapSelectedMailboxV1,
    fetch_uids: Vec<u32>,
    has_more: bool,
}

async fn imap_sync_pages_once<F>(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    requested: usize,
    page_size: usize,
    priority_uids: &[u32],
    finalize_page: &mut F,
) -> Result<usize, ImapError>
where
    F: FnMut(ImapSyncResult) -> Result<(), ()>,
{
    let ImapSyncPlanV1 {
        session,
        mailboxes,
        selected_mailbox,
        fetch_uids,
        has_more,
    } = discover_sync_plan(host, port, username, password, requested, priority_uids).await?;
    let mut active_session = Some(session);
    let page_count = fetch_uids.len().div_ceil(page_size);
    let mut observed_messages = 0_usize;
    for (page_index, page_uids) in fetch_uids.chunks(page_size).enumerate() {
        let mut attempts = 0_u8;
        let messages = loop {
            attempts = attempts.saturating_add(1);
            if active_session.is_none() {
                match reopen_selected_session(host, port, username, password, &selected_mailbox)
                    .await
                {
                    Ok(session) => active_session = Some(session),
                    Err(error) => {
                        if error.is_retryable() && attempts < retry::MAX_SYNC_ATTEMPTS {
                            task::sleep(Duration::from_millis(
                                retry::IMAP_SYNC_RETRY_POLICY.delay_millis,
                            ))
                            .await;
                            continue;
                        }
                        return Err(error);
                    }
                }
            }
            let result = fetch_messages(
                active_session
                    .as_mut()
                    .expect("IMAP page session must be available"),
                page_uids,
            )
            .await;
            match result {
                Ok(messages) => break messages,
                Err(error) => {
                    active_session = None;
                    if error.is_retryable() && attempts < retry::MAX_SYNC_ATTEMPTS {
                        task::sleep(Duration::from_millis(
                            retry::IMAP_SYNC_RETRY_POLICY.delay_millis,
                        ))
                        .await;
                        continue;
                    }
                    return Err(error);
                }
            }
        };
        let page_messages = messages.len();
        finalize_page(ImapSyncResult {
            messages,
            mailboxes: mailboxes.clone(),
            selected_mailbox: selected_mailbox.clone(),
            attempts,
            window: uids_window(page_uids.len()),
            has_more: has_more || page_index.saturating_add(1) < page_count,
        })
        .map_err(|()| {
            ImapError::new("page_finalization", "Mail rejected IMAP page finalization")
        })?;
        observed_messages = observed_messages.saturating_add(page_messages);
    }
    if let Some(mut session) = active_session {
        session
            .logout()
            .await
            .map_err(|error| ImapError::new("protocol", format!("imap logout failed: {error}")))?;
    }
    Ok(observed_messages)
}

async fn discover_sync_plan(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    requested: usize,
    priority_uids: &[u32],
) -> Result<ImapSyncPlanV1, ImapError> {
    let mut attempts = 0_u8;
    loop {
        attempts = attempts.saturating_add(1);
        match discover_sync_plan_once(host, port, username, password, requested, priority_uids)
            .await
        {
            Ok(plan) => return Ok(plan),
            Err(error) => {
                if error.is_retryable() && attempts < retry::MAX_SYNC_ATTEMPTS {
                    task::sleep(Duration::from_millis(
                        retry::IMAP_SYNC_RETRY_POLICY.delay_millis,
                    ))
                    .await;
                    continue;
                }
                return Err(error);
            }
        }
    }
}

async fn discover_sync_plan_once(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    requested: usize,
    priority_uids: &[u32],
) -> Result<ImapSyncPlanV1, ImapError> {
    let started = Instant::now();
    let mut session = open_session(host, port, username, password).await?;
    developer_imap_stage("login", started, 0);
    let list_started = Instant::now();
    let mailboxes = discover_mailboxes(&mut session).await?;
    developer_imap_stage("list", list_started, mailboxes.len());
    let inbox = mailboxes
        .iter()
        .find(|mailbox| mailbox.kind == ImapMailboxKindV1::Inbox)
        .ok_or_else(|| ImapError::new("protocol", "imap LIST did not return selectable INBOX"))?;
    let examine_started = Instant::now();
    let selected = session.examine(&inbox.mailbox_id).await.map_err(|error| {
        ImapError::new("protocol", format!("imap EXAMINE mailbox failed: {error}"))
    })?;
    developer_imap_stage("examine", examine_started, 0);
    let uid_validity = selected
        .uid_validity
        .filter(|value| *value > 0)
        .ok_or_else(|| ImapError::new("protocol", "imap mailbox omitted UIDVALIDITY"))?;
    let selected_mailbox_id = inbox.mailbox_id.clone();
    let all_uids = if requested == 0 {
        Vec::new()
    } else {
        let search_started = Instant::now();
        let ids = session.uid_search("UID 1:*").await.map_err(|error| {
            ImapError::new("protocol", format!("imap uid search failed: {error}"))
        })?;
        developer_imap_stage("search", search_started, ids.len());
        ids.into_iter().collect::<Vec<_>>()
    };
    let (fetch_uids, has_more) = select_latest_uids(all_uids, requested, priority_uids);
    Ok(ImapSyncPlanV1 {
        session,
        mailboxes,
        selected_mailbox: ImapSelectedMailboxV1 {
            mailbox_id: selected_mailbox_id,
            uid_validity,
        },
        fetch_uids,
        has_more,
    })
}

fn select_latest_uids(
    mut all_uids: Vec<u32>,
    requested: usize,
    priority_uids: &[u32],
) -> (Vec<u32>, bool) {
    all_uids.sort_unstable();
    let has_more = all_uids.len() > requested;
    let mut selected = Vec::with_capacity(requested.min(all_uids.len()));
    for uid in priority_uids {
        if selected.len() == requested {
            break;
        }
        if all_uids.binary_search(uid).is_ok() && !selected.contains(uid) {
            selected.push(*uid);
        }
    }
    for uid in all_uids.into_iter().rev() {
        if selected.len() == requested {
            break;
        }
        if !selected.contains(&uid) {
            selected.push(uid);
        }
    }
    (selected, has_more)
}

async fn reopen_selected_session(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    selected_mailbox: &ImapSelectedMailboxV1,
) -> Result<Session<ImapTransport>, ImapError> {
    let mut session = open_session(host, port, username, password).await?;
    let selected = session
        .examine(&selected_mailbox.mailbox_id)
        .await
        .map_err(|error| {
            ImapError::new("protocol", format!("imap EXAMINE mailbox failed: {error}"))
        })?;
    if selected.uid_validity != Some(selected_mailbox.uid_validity) {
        return Err(ImapError::new(
            "protocol",
            "imap UIDVALIDITY changed during sync",
        ));
    }
    Ok(session)
}

async fn discover_mailboxes<T>(session: &mut Session<T>) -> Result<Vec<ImapMailboxV1>, ImapError>
where
    T: AsyncRead + AsyncWrite + Unpin + Debug + Send,
{
    let mut names = session
        .list(None, Some("*"))
        .await
        .map_err(|error| ImapError::new("protocol", format!("imap LIST failed: {error}")))?;
    let mut mailboxes = Vec::new();
    while let Some(name) = names.try_next().await.map_err(|error| {
        ImapError::new("protocol", format!("imap LIST response failed: {error}"))
    })? {
        if name
            .attributes()
            .iter()
            .any(|attribute| matches!(attribute, NameAttribute::NoSelect))
        {
            continue;
        }
        if mailboxes.len() == MAX_DISCOVERED_MAILBOXES {
            return Err(ImapError::new(
                "bounds",
                "imap mailbox discovery exceeded the admitted limit",
            ));
        }
        let mailbox_id = name.name().trim();
        if !valid_mailbox_id(mailbox_id)
            || mailboxes
                .iter()
                .any(|mailbox: &ImapMailboxV1| mailbox.mailbox_id == mailbox_id)
        {
            continue;
        }
        mailboxes.push(ImapMailboxV1 {
            mailbox_id: mailbox_id.to_owned(),
            display_name: mailbox_id.to_owned(),
            kind: imap_mailbox_kind(mailbox_id, name.attributes()),
        });
    }
    if mailboxes.is_empty() {
        return Err(ImapError::new(
            "protocol",
            "imap LIST returned no selectable mailbox",
        ));
    }
    Ok(mailboxes)
}

fn imap_mailbox_kind(mailbox_id: &str, attributes: &[NameAttribute<'_>]) -> ImapMailboxKindV1 {
    if mailbox_id.eq_ignore_ascii_case("INBOX") {
        return ImapMailboxKindV1::Inbox;
    }
    if attributes
        .iter()
        .any(|attribute| matches!(attribute, NameAttribute::Archive))
    {
        ImapMailboxKindV1::Archive
    } else if attributes
        .iter()
        .any(|attribute| matches!(attribute, NameAttribute::Trash))
    {
        ImapMailboxKindV1::Trash
    } else if attributes
        .iter()
        .any(|attribute| matches!(attribute, NameAttribute::Sent))
    {
        ImapMailboxKindV1::Sent
    } else if attributes
        .iter()
        .any(|attribute| matches!(attribute, NameAttribute::Drafts))
    {
        ImapMailboxKindV1::Drafts
    } else if attributes
        .iter()
        .any(|attribute| matches!(attribute, NameAttribute::Junk))
    {
        ImapMailboxKindV1::Spam
    } else if attributes
        .iter()
        .any(|attribute| matches!(attribute, NameAttribute::All))
    {
        ImapMailboxKindV1::All
    } else {
        ImapMailboxKindV1::ProviderFolder
    }
}

fn valid_mailbox_id(mailbox_id: &str) -> bool {
    !mailbox_id.trim().is_empty()
        && mailbox_id.len() <= 512
        && mailbox_id.trim() == mailbox_id
        && !mailbox_id.contains(['\0', '\r', '\n'])
}

fn imap_mailbox_argument(mailbox_id: &str) -> String {
    format!(
        "\"{}\"",
        mailbox_id.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

fn exact_single_uid(values: &[UidSetMember]) -> Option<u32> {
    match values {
        [UidSetMember::Uid(uid)] => Some(*uid),
        [UidSetMember::UidRange(range)] if range.start() == range.end() => Some(*range.start()),
        _ => None,
    }
}

#[cfg(not(feature = "conformance-test-support"))]
async fn open_session(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<Session<async_native_tls::TlsStream<TcpStream>>, ImapError> {
    let address = (host, port);
    let tcp_stream = TcpStream::connect(address).await.map_err(|error| {
        ImapError::new(
            "network",
            format!("tcp connect to {host}:{port} failed: {error}"),
        )
    })?;
    let tls_stream = TlsConnector::new()
        .connect(host, tcp_stream)
        .await
        .map_err(|error| {
            ImapError::new(
                "tls",
                format!("tls connect to {host}:{port} failed: {error}"),
            )
        })?;
    login_session(tls_stream, username, password).await
}

#[cfg(feature = "conformance-test-support")]
async fn open_session(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<Session<TcpStream>, ImapError> {
    if !conformance_loopback_host(host) {
        return Err(ImapError::new(
            "conformance",
            "plaintext IMAP conformance transport is loopback-only",
        ));
    }
    let tcp_stream = TcpStream::connect((host, port)).await.map_err(|error| {
        ImapError::new(
            "network",
            format!("tcp connect to loopback fixture {host}:{port} failed: {error}"),
        )
    })?;
    login_session(tcp_stream, username, password).await
}

async fn login_session<T>(
    stream: T,
    username: &str,
    password: &str,
) -> Result<Session<T>, ImapError>
where
    T: AsyncRead + AsyncWrite + Debug + Send + Unpin,
{
    let mut client = async_imap::Client::new(stream);
    client
        .read_response()
        .await
        .map_err(|error| ImapError::new("protocol", format!("imap greeting failed: {error}")))?;
    let session = client
        .login(username, password)
        .await
        .map_err(|(error, _)| ImapError::new("auth", format!("imap login failed: {error}")))?;
    Ok(session)
}

#[cfg(feature = "conformance-test-support")]
fn conformance_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

async fn fetch_messages<T>(
    session: &mut Session<T>,
    uids: &[u32],
) -> Result<Vec<ImapMessage>, ImapError>
where
    T: AsyncRead + AsyncWrite + Debug + Send + Unpin,
{
    let mut messages = Vec::new();
    for chunk in uids.chunks(IMAP_UID_FETCH_CHUNK_SIZE) {
        let fetch_started = Instant::now();
        let fetched_messages =
            future::timeout(Duration::from_secs(IMAP_UID_FETCH_TIMEOUT_SECONDS), async {
                session
                    .uid_fetch(
                        uid_set(chunk),
                        "(UID BODY.PEEK[] RFC822.SIZE INTERNALDATE FLAGS)",
                    )
                    .await?
                    .try_collect::<Vec<_>>()
                    .await
            })
            .await
            .map_err(|_| {
                ImapError::new(
                    "timeout",
                    format!("uid fetch exceeded {IMAP_UID_FETCH_TIMEOUT_SECONDS}s window"),
                )
            })?
            .map_err(|error| ImapError::new("protocol", format!("uid fetch failed: {error}")))?;
        developer_imap_stage("fetch", fetch_started, fetched_messages.len());

        for message in fetched_messages {
            let uid = message
                .uid
                .ok_or_else(|| ImapError::new("protocol", "missing UID in fetched message"))?;
            let body = message
                .body()
                .ok_or_else(|| ImapError::new("protocol", "missing BODY.PEEK[] payload"))?;
            let (fallback_subject, fallback_snippet, fallback_has_plain_text) =
                decode_message_preview(body);
            let preview = operational_preview(body);
            let subject = preview
                .as_ref()
                .and_then(|preview| preview.subject.clone())
                .unwrap_or(fallback_subject);
            let sender = preview.as_ref().and_then(|preview| preview.sender.clone());
            let recipients = preview
                .as_ref()
                .map(|preview| preview.recipients.clone())
                .unwrap_or_default();
            let snippet = preview
                .as_ref()
                .and_then(|preview| preview.snippet.clone())
                .unwrap_or(fallback_snippet);
            let has_plain_text = preview
                .as_ref()
                .is_some_and(|preview| preview.has_plain_text)
                || fallback_has_plain_text;
            let flags = message
                .flags()
                .filter_map(|flag| match flag {
                    Flag::Seen => Some(ImapMessageFlag::Read),
                    Flag::Flagged => Some(ImapMessageFlag::Starred),
                    Flag::Draft => Some(ImapMessageFlag::Draft),
                    Flag::Deleted => Some(ImapMessageFlag::Trashed),
                    Flag::Answered | Flag::Recent | Flag::MayCreate | Flag::Custom(_) => None,
                })
                .collect();
            let attachments = decode_message_attachments(body);
            messages.push(ImapMessage {
                uid,
                subject,
                sender,
                recipients,
                snippet,
                sent_at_unix_seconds: message.internal_date().map(|date| date.timestamp()),
                flags,
                has_plain_text,
                plain_text_body: readable_text_body(body),
                body_content: readable_body_content(body),
                attachments,
            });
        }
    }
    Ok(messages)
}

fn developer_imap_stage(stage: &str, started: Instant, item_count: usize) {
    if std::env::var_os("HERMES_DEVELOPER_VERBOSE").is_some() {
        eprintln!(
            "developer_mail_imap_stage={stage} elapsed_millis={} item_count={item_count}",
            started.elapsed().as_millis()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_mail_api::{IMAP_PORT, MAX_WINDOW, MAX_WINDOWS};

    #[test]
    fn uid_fetch_timeout_matches_contract_window_deadline() {
        assert_eq!(IMAP_UID_FETCH_TIMEOUT_SECONDS, WINDOW_DEADLINE_SECONDS);
    }

    #[test]
    fn uid_fetch_chunk_is_transport_bounded() {
        assert_eq!(IMAP_UID_FETCH_CHUNK_SIZE, 10);
        const {
            assert!(IMAP_UID_FETCH_CHUNK_SIZE <= 25);
            assert!(IMAP_UID_FETCH_CHUNK_SIZE < MAX_WINDOW as usize);
        }
    }

    #[test]
    fn latest_uids_are_partitioned_deterministically_into_bounded_pages() {
        let (uids, has_more) = select_latest_uids(vec![8, 2, 7, 4, 6, 5, 3, 1], 6, &[]);
        let pages = uids.chunks(2).map(<[u32]>::to_vec).collect::<Vec<_>>();

        assert!(has_more);
        assert_eq!(pages, vec![vec![8, 7], vec![6, 5], vec![4, 3]]);
    }

    #[test]
    fn known_projection_uids_are_fetched_before_the_latest_window() {
        let (uids, has_more) = select_latest_uids((1..=12).collect(), 6, &[3, 9, 3]);

        assert!(has_more);
        assert_eq!(uids, vec![3, 9, 12, 11, 10, 8]);
    }

    #[test]
    fn whole_sync_deadline_is_five_minutes() {
        assert_eq!(IMAP_SYNC_TIMEOUT_SECONDS, 300);
    }

    #[test]
    fn default_retry_policy_max_attempts_match_public_constant() {
        assert_eq!(MAX_ATTEMPTS, retry::MAX_SYNC_ATTEMPTS);
        assert_eq!(retry::IMAP_SYNC_RETRY_POLICY.max_attempts, MAX_ATTEMPTS);
        assert_eq!(MAX_ATTEMPTS, 3);
    }

    #[test]
    fn retries_until_attempt_limit_on_repeated_failures() {
        let mut attempts = 0u8;
        let result = sync_inbox_with_retry(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            |_host, _port, _username, _password, _limit, _deadline| {
                attempts += 1;
                Err(ImapError::new("timeout", "temporary sync failure"))
            },
        );

        assert_eq!(attempts, MAX_ATTEMPTS);
        assert!(matches!(result, Err(error) if error.contains("imap sync failed")));
    }

    #[test]
    fn retries_respect_custom_retry_policy() {
        let mut attempts = 0u8;
        let policy = retry::ImapRetryPolicy {
            max_attempts: 2,
            delay_millis: 0,
        };
        let result = sync_inbox_with_retry_policy(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            policy,
            |_host, _port, _username, _password, _limit, _deadline| {
                attempts += 1;
                Err(ImapError::new("network", "temporary sync failure"))
            },
        );

        assert_eq!(attempts, 2);
        assert!(matches!(result, Err(error) if error.contains("imap sync failed")));
    }

    #[test]
    fn stops_retrying_after_first_success() {
        let mut attempts = 0u8;
        let result = sync_inbox_with_retry(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            |_host, _port, _username, _password, _limit, _deadline| {
                attempts += 1;
                if attempts < 3 {
                    return Err(ImapError::new("timeout", "temporary sync failure"));
                }
                Ok(ImapSyncResult {
                    attempts: 1,
                    window: 1,
                    messages: Vec::new(),
                    mailboxes: vec![ImapMailboxV1 {
                        mailbox_id: "INBOX".to_owned(),
                        display_name: "INBOX".to_owned(),
                        kind: ImapMailboxKindV1::Inbox,
                    }],
                    selected_mailbox: ImapSelectedMailboxV1 {
                        mailbox_id: "INBOX".to_owned(),
                        uid_validity: 1,
                    },
                    has_more: false,
                })
            },
        );

        assert!(result.is_ok());
        assert_eq!(attempts, 3);
        let summary = result.expect("success");
        assert_eq!(summary.attempts, 3);
    }

    #[test]
    fn succeeds_on_last_allowed_attempt() {
        let mut attempts = 0u8;
        let result = sync_inbox_with_retry(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            |_host, _port, _username, _password, _limit, _deadline| {
                attempts += 1;
                if attempts == MAX_ATTEMPTS {
                    return Ok(ImapSyncResult {
                        attempts: 1,
                        window: 1,
                        messages: Vec::new(),
                        mailboxes: vec![ImapMailboxV1 {
                            mailbox_id: "INBOX".to_owned(),
                            display_name: "INBOX".to_owned(),
                            kind: ImapMailboxKindV1::Inbox,
                        }],
                        selected_mailbox: ImapSelectedMailboxV1 {
                            mailbox_id: "INBOX".to_owned(),
                            uid_validity: 1,
                        },
                        has_more: false,
                    });
                }
                Err(ImapError::new("network", "temporary sync failure"))
            },
        );

        assert_eq!(attempts, MAX_ATTEMPTS);
        assert!(result.is_ok());
        let summary = result.expect("success on final attempt");
        assert_eq!(summary.attempts, MAX_ATTEMPTS);
    }

    #[test]
    fn sync_inbox_with_retry_carries_large_plan_limit_without_overflow() {
        let expected_limit = (MAX_WINDOW as u64) * (MAX_WINDOWS as u64);
        let expected_limit = usize::try_from(expected_limit).expect("plan limit should fit usize");
        let mut observed_limit = 0usize;
        let mut observed_attempts = 0u8;
        assert!(expected_limit > u32::MAX as usize);
        let result = sync_inbox_with_retry_policy(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            expected_limit,
            retry::ImapRetryPolicy {
                max_attempts: 1,
                delay_millis: 0,
            },
            |_host, _port, _username, _password, limit, _deadline| {
                observed_attempts += 1;
                observed_limit = limit;
                Err(ImapError::new("timeout", "temporary sync failure"))
            },
        );

        assert_eq!(observed_attempts, 1);
        assert_eq!(observed_limit, expected_limit);
        assert!(matches!(result, Err(error) if error.contains("imap sync failed")));
    }

    #[test]
    fn definite_provider_rejection_is_not_retried() {
        let mut attempts = 0u8;
        let result = sync_inbox_with_retry(
            "mail.example.com",
            IMAP_PORT,
            "alice",
            "secret",
            1,
            |_host, _port, _username, _password, _limit, _deadline| {
                attempts += 1;
                Err(ImapError::new("auth", "credentials rejected"))
            },
        );

        assert_eq!(attempts, 1);
        assert!(matches!(result, Err(error) if error.contains("imap sync failed")));
    }

    #[test]
    fn supports_read_only_window_limits_are_applied() {
        assert!(supports_read_only_sync(MAX_WINDOW));
        assert!(!supports_read_only_sync(MAX_WINDOW + 1));
        assert!(supports_read_only_windows(MAX_WINDOWS));
        assert!(!supports_read_only_windows(MAX_WINDOWS + 1));
    }

    #[test]
    fn message_flag_mutation_rejects_an_invalid_uid_before_network_io() {
        assert!(
            set_message_flag(
                ImapMessageFlagAccessV1 {
                    host: "mail.example.com",
                    port: 993,
                    username: "alice",
                    password: "secret",
                },
                ImapMessageLocatorV1 {
                    mailbox_id: "INBOX",
                    uid_validity: 1,
                    uid: 0,
                },
                ImapMutableMessageFlagV1::Read,
                true,
            )
            .is_err()
        );
    }

    #[test]
    fn message_location_mutation_validates_target_before_network_io() {
        let result = move_message(
            ImapMessageLocationAccessV1 {
                host: "mail.example.com",
                port: 993,
                username: "alice",
                password: "secret",
            },
            ImapMessageLocatorV1 {
                mailbox_id: "INBOX",
                uid_validity: 1,
                uid: 42,
            },
            "Archive\r\nUID EXPUNGE",
        );

        assert!(matches!(result, Err(error) if error.is_definite_rejection()));
    }

    #[test]
    fn same_mailbox_location_is_a_provider_noop() {
        assert_eq!(
            move_message(
                ImapMessageLocationAccessV1 {
                    host: "not-a-network-endpoint",
                    port: 1,
                    username: "alice",
                    password: "secret",
                },
                ImapMessageLocatorV1 {
                    mailbox_id: "INBOX",
                    uid_validity: 7,
                    uid: 42,
                },
                "INBOX",
            )
            .expect("same mailbox is already reconciled"),
            ImapMessageLocationResultV1 {
                mailbox_id: "INBOX".to_owned(),
                uid_validity: 7,
                uid: 42,
            }
        );
    }

    #[test]
    fn uid_move_mailbox_argument_is_quoted_and_escaped() {
        assert_eq!(
            imap_mailbox_argument("Owner \"Archive\" \\ 2026"),
            "\"Owner \\\"Archive\\\" \\\\ 2026\""
        );
    }

    #[test]
    fn mailbox_roles_come_only_from_canonical_name_and_special_use_attributes() {
        assert_eq!(imap_mailbox_kind("inbox", &[]), ImapMailboxKindV1::Inbox);
        assert_eq!(
            imap_mailbox_kind("Owner Archive", &[NameAttribute::Archive]),
            ImapMailboxKindV1::Archive
        );
        assert_eq!(
            imap_mailbox_kind("Bin", &[NameAttribute::Trash]),
            ImapMailboxKindV1::Trash
        );
        assert_eq!(
            imap_mailbox_kind("Projects", &[]),
            ImapMailboxKindV1::ProviderFolder
        );
    }

    #[test]
    fn extracts_only_explicit_bounded_attachment_metadata_from_nested_mime() {
        let message = concat!(
            "Content-Type: multipart/mixed; boundary=outer\r\n\r\n",
            "--outer\r\nContent-Type: text/plain\r\n\r\nhello\r\n",
            "--outer\r\nContent-Type: multipart/related; boundary=inner\r\n\r\n",
            "--inner\r\nContent-Type: application/pdf; name=report.pdf\r\n",
            "Content-Disposition: attachment; filename=report.pdf\r\n",
            "Content-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n",
            "--inner--\r\n--outer--\r\n",
        );

        assert_eq!(
            decode_message_attachments(message.as_bytes()),
            vec![ImapAttachment {
                part_id: 1,
                filename: Some("report.pdf".to_owned()),
                media_type: "application/pdf".to_owned(),
                declared_bytes: 5,
                disposition: ImapAttachmentDisposition::Attachment,
                bytes: b"hello".to_vec(),
            }],
        );
    }

    #[test]
    fn rejects_attachment_with_undecidable_transfer_encoding() {
        let message = concat!(
            "Content-Type: multipart/mixed; boundary=outer\r\n\r\n",
            "--outer\r\nContent-Type: application/octet-stream\r\n",
            "Content-Disposition: attachment\r\n",
            "Content-Transfer-Encoding: quoted-printable\r\n\r\nhello=20world\r\n",
            "--outer--\r\n",
        );

        assert!(decode_message_attachments(message.as_bytes()).is_empty());
    }

    #[test]
    fn preview_bound_never_splits_a_multibyte_character() {
        let body = format!(
            "Subject: unicode\r\n\r\n{}",
            "Почтовое сообщение ".repeat(SNAPSHOT_PREVIEW_BYTES)
        );

        let (_, snippet, has_plain_text) = decode_message_preview(body.as_bytes());

        assert!(has_plain_text);
        assert!(snippet.len() <= SNAPSHOT_PREVIEW_BYTES);
        assert!(std::str::from_utf8(snippet.as_bytes()).is_ok());
    }

    #[cfg(feature = "conformance-test-support")]
    #[test]
    fn plaintext_conformance_transport_is_loopback_only() {
        assert!(conformance_loopback_host("127.0.0.1"));
        assert!(conformance_loopback_host("::1"));
        assert!(conformance_loopback_host("localhost"));
        assert!(!conformance_loopback_host("imap.example.test"));
    }
}

fn decode_message_preview(body: &[u8]) -> (String, String, bool) {
    let text = readable_text_body(body)
        .and_then(|body| String::from_utf8(body).ok())
        .unwrap_or_default();
    let has_plain_text = !text.trim().is_empty();
    let mut snippet = if has_plain_text {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_owned()
    } else {
        String::new()
    };
    if has_plain_text && snippet.len() > SNAPSHOT_PREVIEW_BYTES {
        let boundary = floor_char_boundary(&snippet, SNAPSHOT_PREVIEW_BYTES);
        snippet.truncate(boundary);
    }
    if snippet.is_empty() {
        snippet = "message".to_owned();
    }
    let has_plain_text = snippet.len() <= MAX_PLAIN_TEXT_BYTES && has_plain_text;
    ("message".to_owned(), snippet, has_plain_text)
}

fn floor_char_boundary(value: &str, maximum_bytes: usize) -> usize {
    if value.len() <= maximum_bytes {
        return value.len();
    }
    let mut boundary = maximum_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

fn decode_message_attachments(raw_message: &[u8]) -> Vec<ImapAttachment> {
    attachment_metadata(raw_message)
        .into_iter()
        .filter_map(|attachment| {
            let bytes = extract_attachment_part(raw_message, attachment.part_id).ok()?;
            (u64::try_from(bytes.len()).ok()? == attachment.declared_bytes).then_some(
                ImapAttachment {
                    part_id: attachment.part_id,
                    filename: attachment.filename,
                    media_type: attachment.media_type,
                    declared_bytes: attachment.declared_bytes,
                    disposition: match attachment.disposition {
                        AttachmentDispositionV1::Attachment => {
                            ImapAttachmentDisposition::Attachment
                        }
                        AttachmentDispositionV1::Inline => ImapAttachmentDisposition::Inline,
                    },
                    bytes,
                },
            )
        })
        .collect()
}

fn uids_window(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

fn uid_set(uids: &[u32]) -> String {
    uids.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

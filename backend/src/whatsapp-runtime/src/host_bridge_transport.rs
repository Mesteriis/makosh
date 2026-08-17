//! Framed Unix transport for the admitted private WhatsApp host bridge.

use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use makosh_whatsapp_api::host_bridge::{
    decode_host_bridge_handshake, encode_host_bridge_handshake_accepted,
};

use crate::{host_bridge_port, managed::WhatsAppAdmittedRuntime};

const MAX_FRAME_BYTES: usize = 512 * 1024;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_millis(250);

struct PendingHostRequest {
    request: Vec<u8>,
    response: SyncSender<Result<Vec<u8>, ()>>,
}

/// Owns one persistent provider-host connection without lending the admitted
/// runtime to the blocking socket reader. The process actor consumes at most
/// one request per tick, so client delivery, outbox and realtime work continue
/// to make progress while the hidden WebView connection remains open.
pub struct WhatsAppHostBridgeSession {
    requests: Receiver<PendingHostRequest>,
    shutdown: UnixStream,
    worker: Option<JoinHandle<()>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhatsAppHostBridgeSessionProgress {
    Idle,
    Handled,
    Closed,
}

#[derive(Debug)]
pub enum WhatsAppHostBridgeTransportError {
    Closed,
    Frame,
    Io,
    Port,
    Handshake,
}

impl WhatsAppHostBridgeSession {
    pub fn start(
        mut stream: UnixStream,
        runtime: &WhatsAppAdmittedRuntime,
    ) -> Result<Self, WhatsAppHostBridgeTransportError> {
        stream
            .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
            .and_then(|_| stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT)))
            .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
        accept_host_bridge_handshake(&mut stream, runtime)?;
        stream
            .set_read_timeout(None)
            .and_then(|_| stream.set_write_timeout(None))
            .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
        let shutdown = stream
            .try_clone()
            .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
        let (request_sender, requests) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("makosh-whatsapp-host-bridge".to_owned())
            .spawn(move || run_connection_reader(stream, request_sender))
            .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
        Ok(Self {
            requests,
            shutdown,
            worker: Some(worker),
        })
    }

    pub fn try_handle_once(
        &mut self,
        runtime: &mut WhatsAppAdmittedRuntime,
        handle: &tokio::runtime::Handle,
    ) -> WhatsAppHostBridgeSessionProgress {
        let pending = match self.requests.try_recv() {
            Ok(pending) => pending,
            Err(TryRecvError::Empty) => return WhatsAppHostBridgeSessionProgress::Idle,
            Err(TryRecvError::Disconnected) => return WhatsAppHostBridgeSessionProgress::Closed,
        };
        let recorded_at = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(value) => value,
            Err(_) => {
                let _ = pending.response.send(Err(()));
                return WhatsAppHostBridgeSessionProgress::Closed;
            }
        };
        let recorded_at_unix_seconds = match i64::try_from(recorded_at.as_secs()) {
            Ok(value) => value,
            Err(_) => {
                let _ = pending.response.send(Err(()));
                return WhatsAppHostBridgeSessionProgress::Closed;
            }
        };
        let recorded_at_nanos = match i32::try_from(recorded_at.subsec_nanos()) {
            Ok(value) => value,
            Err(_) => {
                let _ = pending.response.send(Err(()));
                return WhatsAppHostBridgeSessionProgress::Closed;
            }
        };
        match handle.block_on(host_bridge_port::handle_host_request(
            runtime,
            &pending.request,
            recorded_at_unix_seconds,
            recorded_at_nanos,
        )) {
            Ok(response) => {
                if pending.response.send(Ok(response)).is_err() {
                    WhatsAppHostBridgeSessionProgress::Closed
                } else {
                    WhatsAppHostBridgeSessionProgress::Handled
                }
            }
            Err(_) => {
                let _ = pending.response.send(Err(()));
                WhatsAppHostBridgeSessionProgress::Closed
            }
        }
    }
}

impl Drop for WhatsAppHostBridgeSession {
    fn drop(&mut self) {
        let _ = self.shutdown.shutdown(std::net::Shutdown::Both);
        if self.worker.as_ref().is_some_and(JoinHandle::is_finished)
            && let Some(worker) = self.worker.take()
        {
            let _ = worker.join();
        }
        // A worker waiting for the actor response is released when `requests`
        // is dropped immediately after this destructor returns. Never join
        // that state here: doing so would make route shutdown self-deadlock.
    }
}

fn run_connection_reader(mut stream: UnixStream, requests: SyncSender<PendingHostRequest>) {
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(_) => return,
        };
        let (response, response_receiver) = mpsc::sync_channel(0);
        if requests
            .send(PendingHostRequest { request, response })
            .is_err()
        {
            return;
        }
        match response_receiver.recv() {
            Ok(Ok(response)) if write_frame(&mut stream, &response).is_ok() => {}
            _ => return,
        }
    }
}

pub fn serve_connection(
    mut stream: UnixStream,
    runtime: &mut WhatsAppAdmittedRuntime,
    handle: &tokio::runtime::Handle,
) -> Result<(), WhatsAppHostBridgeTransportError> {
    stream
        .set_nonblocking(false)
        .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
    accept_host_bridge_handshake(&mut stream, runtime)?;
    loop {
        let request = match read_frame(&mut stream) {
            Ok(request) => request,
            Err(WhatsAppHostBridgeTransportError::Closed) => return Ok(()),
            Err(error) => return Err(error),
        };
        let recorded_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
        let recorded_at_unix_seconds = i64::try_from(recorded_at.as_secs())
            .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
        let recorded_at_nanos = i32::try_from(recorded_at.subsec_nanos())
            .map_err(|_| WhatsAppHostBridgeTransportError::Io)?;
        let response = handle
            .block_on(host_bridge_port::handle_host_request(
                runtime,
                &request,
                recorded_at_unix_seconds,
                recorded_at_nanos,
            ))
            .map_err(|_| WhatsAppHostBridgeTransportError::Port)?;
        write_frame(&mut stream, &response)?;
    }
}

fn accept_host_bridge_handshake(
    stream: &mut UnixStream,
    runtime: &WhatsAppAdmittedRuntime,
) -> Result<(), WhatsAppHostBridgeTransportError> {
    let handshake = read_frame(stream)?;
    let handshake = decode_host_bridge_handshake(&handshake)
        .map_err(|_| WhatsAppHostBridgeTransportError::Handshake)?;
    if !runtime.accepts_host_bridge_handshake(&handshake) {
        return Err(WhatsAppHostBridgeTransportError::Handshake);
    }
    write_frame(stream, &encode_host_bridge_handshake_accepted())
}

fn read_frame(stream: &mut UnixStream) -> Result<Vec<u8>, WhatsAppHostBridgeTransportError> {
    let length = read_length(stream)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(WhatsAppHostBridgeTransportError::Frame);
    }
    let mut bytes = vec![0_u8; length];
    stream.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            WhatsAppHostBridgeTransportError::Closed
        } else {
            WhatsAppHostBridgeTransportError::Io
        }
    })?;
    Ok(bytes)
}

fn write_frame(
    stream: &mut UnixStream,
    bytes: &[u8],
) -> Result<(), WhatsAppHostBridgeTransportError> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME_BYTES {
        return Err(WhatsAppHostBridgeTransportError::Frame);
    }
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| WhatsAppHostBridgeTransportError::Frame)?;
    let mut prefix = Vec::with_capacity(5);
    while length >= 0x80 {
        prefix.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    prefix.push(length as u8);
    stream
        .write_all(&prefix)
        .and_then(|_| stream.write_all(bytes))
        .and_then(|_| stream.flush())
        .map_err(|_| WhatsAppHostBridgeTransportError::Io)
}

fn read_length(stream: &mut UnixStream) -> Result<usize, WhatsAppHostBridgeTransportError> {
    let mut value = 0_u64;
    for index in 0..5 {
        let mut byte = [0_u8; 1];
        match stream.read_exact(&mut byte) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
                return Err(WhatsAppHostBridgeTransportError::Closed);
            }
            Err(_) => return Err(WhatsAppHostBridgeTransportError::Io),
        }
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return usize::try_from(value).map_err(|_| WhatsAppHostBridgeTransportError::Frame);
        }
    }
    Err(WhatsAppHostBridgeTransportError::Frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_reader_yields_each_request_to_the_runtime_actor() {
        let (server, mut host) = UnixStream::pair().expect("host bridge pair");
        host.set_read_timeout(Some(Duration::from_secs(1)))
            .expect("host read timeout");
        let (requests, pending) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || run_connection_reader(server, requests));

        write_frame(&mut host, b"provider request").expect("provider request frame");
        let request = pending
            .recv_timeout(Duration::from_secs(1))
            .expect("runtime actor request");

        assert_eq!(request.request, b"provider request");
        request
            .response
            .send(Ok(b"runtime response".to_vec()))
            .expect("runtime actor response");
        assert_eq!(
            read_frame(&mut host).expect("provider response frame"),
            b"runtime response",
        );

        host.shutdown(std::net::Shutdown::Both)
            .expect("host shutdown");
        worker.join().expect("reader worker");
    }
}

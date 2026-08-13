//! Correlated duplex framing for one inherited managed-control stream.
//!
//! This module owns only frame identity, bounded I/O and request/response
//! routing. Operation authorization remains with Kernel and operation meaning
//! remains in the typed protocol messages.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use prost::Message;

use crate::v1::{
    DescribeManagedRuntimeRequestV1, DescribeManagedRuntimeResponseV1,
    ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlAckV1,
    ManagedRuntimeControlFrameV2, ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
    ManagedRuntimeReadyRequestV1, ModuleClientResponseV1, ModuleDescriptorV1,
    managed_runtime_control_frame_v2::Frame, managed_runtime_control_request_v1::Operation,
    managed_runtime_control_response_v1::Result as ControlResult,
};
use crate::validation::managed_control::{
    MANAGED_CONTROL_CORRELATION_ID_BYTES, MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
    validate_managed_control_frame_v2,
};

pub const MAX_MANAGED_CONTROL_FRAME_BYTES_V2: usize = 512 * 1024;
pub const MAX_MANAGED_CONTROL_PENDING_REQUESTS_V2: usize = 64;
const MANAGED_CONTROL_WRITE_TIMEOUT_V2: Duration = Duration::from_secs(5);
const MANAGED_CONTROL_WRITE_RETRY_INTERVAL_V2: Duration = Duration::from_millis(1);
static NEXT_CORRELATION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedControlTransportMajorV1 {
    LegacyV1,
    CorrelatedV2,
}

/// Selects one control transport major from the signed module descriptor.
///
/// The inherited FD never negotiates its version: Kernel chooses exactly one
/// declared major before spawn, then both endpoints use only that framing for
/// the lifetime of the managed session.
pub fn select_managed_control_transport(
    descriptor: &ModuleDescriptorV1,
) -> Result<ManagedControlTransportMajorV1, ManagedControlTransportErrorV2> {
    let range = descriptor
        .runtime_protocol_range
        .as_ref()
        .ok_or(ManagedControlTransportErrorV2::InvalidTransportSelection)?;
    if range.minimum_major != range.maximum_major || range.minimum_revision != 1 {
        return Err(ManagedControlTransportErrorV2::InvalidTransportSelection);
    }
    match range.minimum_major {
        1 => Ok(ManagedControlTransportMajorV1::LegacyV1),
        2 => Ok(ManagedControlTransportMajorV1::CorrelatedV2),
        _ => Err(ManagedControlTransportErrorV2::InvalidTransportSelection),
    }
}

#[derive(Debug)]
pub enum ManagedControlTransportErrorV2 {
    InvalidTransportSelection,
    InvalidCorrelationId,
    InvalidFrame,
    FrameTooLarge,
    Io(std::io::Error),
    UnexpectedResponse,
    UnexpectedRequest,
    DuplicateCorrelationId,
    PendingRequestLimit,
    PeerClosed,
}

impl From<std::io::Error> for ManagedControlTransportErrorV2 {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct ManagedControlChannelV2<S> {
    stream: S,
    pending_request_ids: BTreeSet<[u8; MANAGED_CONTROL_CORRELATION_ID_BYTES]>,
    pending_responses:
        BTreeMap<[u8; MANAGED_CONTROL_CORRELATION_ID_BYTES], ManagedRuntimeControlResponseV1>,
    read_buffer: Vec<u8>,
}

/// Typed owner request dispatch used while a platform call awaits its own
/// correlated response. The frame pump remains platform-owned; a domain only
/// supplies semantics for its declared inbound request.
pub trait ManagedControlRequestDispatcherV2<S> {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<S>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2>;
}

/// Default dispatcher for platform calls made before an owner has an inbound
/// request contract (for example startup-only credential acquisition).
pub struct RejectManagedControlRequestsV2;

impl<S: Read + Write> ManagedControlRequestDispatcherV2<S> for RejectManagedControlRequestsV2 {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<S>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        if let Some(Operation::ClientDelivery(delivery)) = request.operation
            && let Some(request) = delivery.request
        {
            return channel.write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ClientDelivery(
                        ManagedRuntimeClientDeliveryResponseV1 {
                            response: Some(ModuleClientResponseV1 {
                                protocol_major: 1,
                                request_id: request.request_id,
                                response_payload: Vec::new(),
                                error_code: "RUNTIME_UNAVAILABLE".to_owned(),
                            }),
                        },
                    )),
                    error_code: String::new(),
                },
            );
        }
        channel.write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        )
    }
}

impl<S> ManagedControlChannelV2<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            pending_request_ids: BTreeSet::new(),
            pending_responses: BTreeMap::new(),
            read_buffer: Vec::new(),
        }
    }

    pub fn into_inner(self) -> S {
        self.stream
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.stream
    }
}

#[cfg(unix)]
impl ManagedControlChannelV2<std::os::unix::net::UnixStream> {
    /// Observes Unix control-peer closure without consuming a pending frame.
    ///
    /// Bootstrap operations use this while awaiting non-control resources. A
    /// non-empty frame remains available to the correlated dispatcher.
    pub fn peer_closed_preserving_frames(&self) -> Result<bool, std::io::Error> {
        use std::os::fd::AsRawFd;

        let mut byte = [0_u8; 1];
        // SAFETY: `byte` is valid for one writable byte, the stream owns a live
        // file descriptor for the duration of the call, and MSG_PEEK prevents
        // this liveness probe from consuming control protocol bytes.
        let result = unsafe {
            libc::recv(
                self.stream.as_raw_fd(),
                byte.as_mut_ptr().cast(),
                byte.len(),
                libc::MSG_PEEK | libc::MSG_DONTWAIT,
            )
        };
        if result == 0 {
            Ok(true)
        } else if result > 0 {
            Ok(false)
        } else {
            match std::io::Error::last_os_error().kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted => Ok(false),
                _ => Err(std::io::Error::last_os_error()),
            }
        }
    }
}

impl<S: Read + Write> ManagedControlChannelV2<S> {
    pub fn describe_managed_runtime(
        &mut self,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
    ) -> Result<DescribeManagedRuntimeResponseV1, ManagedControlTransportErrorV2> {
        let mut bootstrap_dispatcher = RejectManagedControlRequestsV2;
        let response = self.request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::Describe(DescribeManagedRuntimeRequestV1 {
                    descriptor_bytes,
                    settings_schema_bytes,
                })),
            },
            &mut bootstrap_dispatcher,
        )?;
        match response.result {
            Some(ControlResult::Describe(identity))
                if response.error_code.is_empty()
                    && !identity.registration_id.is_empty()
                    && identity.runtime_generation != 0
                    && identity.grant_epoch != 0 =>
            {
                Ok(identity)
            }
            _ => Err(ManagedControlTransportErrorV2::InvalidFrame),
        }
    }

    pub fn signal_ready(
        &mut self,
        ready: ManagedRuntimeReadyRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let mut bootstrap_dispatcher = RejectManagedControlRequestsV2;
        self.signal_ready_with_dispatch(ready, &mut bootstrap_dispatcher)
    }

    pub fn signal_ready_with_dispatch(
        &mut self,
        ready: ManagedRuntimeReadyRequestV1,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<S>,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = self.request_next_with_dispatch(
            ManagedRuntimeControlRequestV1 {
                operation: Some(Operation::Ready(ready)),
            },
            dispatcher,
        )?;
        match response.result {
            Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {}))
                if response.error_code.is_empty() =>
            {
                Ok(())
            }
            _ => Err(ManagedControlTransportErrorV2::InvalidFrame),
        }
    }

    pub fn request_next<F>(
        &mut self,
        request: ManagedRuntimeControlRequestV1,
        dispatch_request: F,
    ) -> Result<ManagedRuntimeControlResponseV1, ManagedControlTransportErrorV2>
    where
        F: FnMut(
            &mut Self,
            [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            ManagedRuntimeControlRequestV1,
        ) -> Result<(), ManagedControlTransportErrorV2>,
    {
        self.request(next_correlation_id(), request, dispatch_request)
    }

    pub fn request_next_with_dispatch(
        &mut self,
        request: ManagedRuntimeControlRequestV1,
        dispatcher: &mut dyn ManagedControlRequestDispatcherV2<S>,
    ) -> Result<ManagedRuntimeControlResponseV1, ManagedControlTransportErrorV2> {
        self.request(
            next_correlation_id(),
            request,
            |channel, correlation_id, request| {
                dispatcher.dispatch_request(channel, correlation_id, request)
            },
        )
    }

    pub fn receive_request(
        &mut self,
    ) -> Result<
        (
            [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            ManagedRuntimeControlRequestV1,
        ),
        ManagedControlTransportErrorV2,
    > {
        let frame = self.read_frame()?;
        let correlation_id = correlation_id_from_slice(&frame.correlation_id)?;
        match frame.frame {
            Some(Frame::Request(request)) => Ok((correlation_id, request)),
            Some(Frame::Response(_)) => Err(ManagedControlTransportErrorV2::UnexpectedResponse),
            None => Err(ManagedControlTransportErrorV2::InvalidFrame),
        }
    }

    /// Receives one complete request when a nonblocking stream has one ready.
    ///
    /// Partial length prefixes and payloads remain private to this channel until
    /// the complete correlated frame is available; callers must not read the
    /// underlying stream directly.
    pub fn try_receive_request(
        &mut self,
    ) -> Result<
        Option<(
            [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            ManagedRuntimeControlRequestV1,
        )>,
        ManagedControlTransportErrorV2,
    > {
        let Some(frame) = self.try_read_frame()? else {
            return Ok(None);
        };
        let correlation_id = correlation_id_from_slice(&frame.correlation_id)?;
        match frame.frame {
            Some(Frame::Request(request)) => Ok(Some((correlation_id, request))),
            Some(Frame::Response(_)) => Err(ManagedControlTransportErrorV2::UnexpectedResponse),
            None => Err(ManagedControlTransportErrorV2::InvalidFrame),
        }
    }

    pub fn request<F>(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
        mut dispatch_request: F,
    ) -> Result<ManagedRuntimeControlResponseV1, ManagedControlTransportErrorV2>
    where
        F: FnMut(
            &mut Self,
            [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            ManagedRuntimeControlRequestV1,
        ) -> Result<(), ManagedControlTransportErrorV2>,
    {
        self.begin_pending(correlation_id)?;
        if let Err(error) = self.write_request(correlation_id, request) {
            self.pending_request_ids.remove(&correlation_id);
            return Err(error);
        }
        let result = (|| loop {
            if let Some(response) = self.pending_responses.remove(&correlation_id) {
                break Ok(response);
            }
            let frame = self.read_frame()?;
            let received_id = correlation_id_from_slice(&frame.correlation_id)?;
            match frame.frame {
                Some(Frame::Response(response)) if received_id == correlation_id => {
                    break Ok(response);
                }
                Some(Frame::Response(response))
                    if self.pending_request_ids.contains(&received_id) =>
                {
                    self.pending_responses.insert(received_id, response);
                }
                Some(Frame::Response(_)) => {
                    break Err(ManagedControlTransportErrorV2::UnexpectedResponse);
                }
                Some(Frame::Request(request)) => dispatch_request(self, received_id, request)?,
                None => break Err(ManagedControlTransportErrorV2::InvalidFrame),
            }
        })();
        self.pending_request_ids.remove(&correlation_id);
        self.pending_responses.remove(&correlation_id);
        result
    }

    pub fn write_request(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        self.write_frame(ManagedRuntimeControlFrameV2 {
            transport_major: MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
            correlation_id: correlation_id.to_vec(),
            frame: Some(Frame::Request(request)),
        })
    }

    pub fn write_response(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        response: ManagedRuntimeControlResponseV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        self.write_frame(ManagedRuntimeControlFrameV2 {
            transport_major: MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
            correlation_id: correlation_id.to_vec(),
            frame: Some(Frame::Response(response)),
        })
    }

    pub fn read_frame(
        &mut self,
    ) -> Result<ManagedRuntimeControlFrameV2, ManagedControlTransportErrorV2> {
        let bytes = self.read_length_delimited()?;
        let frame = ManagedRuntimeControlFrameV2::decode(bytes.as_slice())
            .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
        validate_managed_control_frame_v2(&frame)
            .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
        Ok(frame)
    }

    fn write_frame(
        &mut self,
        frame: ManagedRuntimeControlFrameV2,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        validate_managed_control_frame_v2(&frame)
            .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
        let bytes = frame.encode_to_vec();
        if bytes.len() > MAX_MANAGED_CONTROL_FRAME_BYTES_V2 {
            return Err(ManagedControlTransportErrorV2::FrameTooLarge);
        }
        write_length_delimited(&mut self.stream, &bytes)?;
        Ok(())
    }

    fn read_length_delimited(&mut self) -> Result<Vec<u8>, ManagedControlTransportErrorV2> {
        loop {
            if let Some(frame) = take_buffered_frame(&mut self.read_buffer)? {
                return Ok(frame);
            }
            let mut chunk = [0_u8; 4096];
            match self.stream.read(&mut chunk) {
                Ok(0) => return Err(ManagedControlTransportErrorV2::PeerClosed),
                Ok(read) => self.read_buffer.extend_from_slice(&chunk[..read]),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(ManagedControlTransportErrorV2::Io(error)),
            }
        }
    }

    fn try_read_frame(
        &mut self,
    ) -> Result<Option<ManagedRuntimeControlFrameV2>, ManagedControlTransportErrorV2> {
        loop {
            if let Some(bytes) = take_buffered_frame(&mut self.read_buffer)? {
                let frame = ManagedRuntimeControlFrameV2::decode(bytes.as_slice())
                    .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
                validate_managed_control_frame_v2(&frame)
                    .map_err(|_| ManagedControlTransportErrorV2::InvalidFrame)?;
                return Ok(Some(frame));
            }
            let mut chunk = [0_u8; 4096];
            match self.stream.read(&mut chunk) {
                Ok(0) => return Err(ManagedControlTransportErrorV2::PeerClosed),
                Ok(read) => self.read_buffer.extend_from_slice(&chunk[..read]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(None),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(ManagedControlTransportErrorV2::Io(error)),
            }
        }
    }

    fn begin_pending(
        &mut self,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    ) -> Result<(), ManagedControlTransportErrorV2> {
        if self.pending_request_ids.len() >= MAX_MANAGED_CONTROL_PENDING_REQUESTS_V2 {
            return Err(ManagedControlTransportErrorV2::PendingRequestLimit);
        }
        if !self.pending_request_ids.insert(correlation_id) {
            return Err(ManagedControlTransportErrorV2::DuplicateCorrelationId);
        }
        Ok(())
    }
}

fn take_buffered_frame(
    buffer: &mut Vec<u8>,
) -> Result<Option<Vec<u8>>, ManagedControlTransportErrorV2> {
    let Some((prefix_len, payload_len)) = buffered_frame_length(buffer)? else {
        return Ok(None);
    };
    let frame_len = prefix_len
        .checked_add(payload_len)
        .ok_or(ManagedControlTransportErrorV2::FrameTooLarge)?;
    if buffer.len() < frame_len {
        return Ok(None);
    }
    let payload = buffer[prefix_len..frame_len].to_vec();
    buffer.drain(..frame_len);
    Ok(Some(payload))
}

fn buffered_frame_length(
    buffer: &[u8],
) -> Result<Option<(usize, usize)>, ManagedControlTransportErrorV2> {
    let mut length = 0_u64;
    for shift in (0..35).step_by(7) {
        let index = shift / 7;
        let Some(byte) = buffer.get(index) else {
            return Ok(None);
        };
        length |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            let length = usize::try_from(length)
                .map_err(|_| ManagedControlTransportErrorV2::FrameTooLarge)?;
            if length == 0 {
                return Err(ManagedControlTransportErrorV2::PeerClosed);
            }
            if length > MAX_MANAGED_CONTROL_FRAME_BYTES_V2 {
                return Err(ManagedControlTransportErrorV2::FrameTooLarge);
            }
            return Ok(Some((index + 1, length)));
        }
    }
    Err(ManagedControlTransportErrorV2::FrameTooLarge)
}

fn next_correlation_id() -> [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES] {
    let sequence = NEXT_CORRELATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    correlation_id_for_process_sequence(std::process::id(), sequence)
}

fn correlation_id_for_process_sequence(
    process_id: u32,
    sequence: u64,
) -> [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES] {
    let mut correlation_id = [0_u8; MANAGED_CONTROL_CORRELATION_ID_BYTES];
    correlation_id[..4].copy_from_slice(b"HMC2");
    correlation_id[4..8].copy_from_slice(&process_id.to_be_bytes());
    correlation_id[8..].copy_from_slice(&sequence.to_be_bytes());
    correlation_id
}

fn correlation_id_from_slice(
    value: &[u8],
) -> Result<[u8; MANAGED_CONTROL_CORRELATION_ID_BYTES], ManagedControlTransportErrorV2> {
    value
        .try_into()
        .map_err(|_| ManagedControlTransportErrorV2::InvalidCorrelationId)
}

fn write_length_delimited(
    stream: &mut impl Write,
    bytes: &[u8],
) -> Result<(), ManagedControlTransportErrorV2> {
    let mut length =
        u32::try_from(bytes.len()).map_err(|_| ManagedControlTransportErrorV2::FrameTooLarge)?;
    let mut frame = Vec::with_capacity(bytes.len() + 5);
    while length >= 0x80 {
        frame.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    frame.push(length as u8);
    frame.extend_from_slice(bytes);

    let deadline = Instant::now() + MANAGED_CONTROL_WRITE_TIMEOUT_V2;
    let mut written = 0;
    while written < frame.len() {
        match stream.write(&frame[written..]) {
            Ok(0) => {
                return Err(std::io::Error::from(ErrorKind::WriteZero).into());
            }
            Ok(count) => written += count,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) if error.kind() == ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(MANAGED_CONTROL_WRITE_RETRY_INTERVAL_V2);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                return Err(std::io::Error::from(ErrorKind::TimedOut).into());
            }
            Err(error) => return Err(error.into()),
        }
    }
    loop {
        match stream.flush() {
            Ok(()) => break,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) if error.kind() == ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(MANAGED_CONTROL_WRITE_RETRY_INTERVAL_V2);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                return Err(std::io::Error::from(ErrorKind::TimedOut).into());
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Error, Write};
    use std::os::unix::net::UnixStream;
    use std::thread;

    use super::*;
    use crate::v1::{
        ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlRequestV1,
        ManagedRuntimeControlResponseV1, ModuleClientRequestV1, ProtocolRangeV1,
        managed_runtime_control_request_v1::Operation,
    };

    fn ready_request() -> ManagedRuntimeControlRequestV1 {
        ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::Ready(Default::default())),
        }
    }

    #[test]
    fn selects_only_an_exact_signed_control_transport_major() {
        let mut descriptor = ModuleDescriptorV1 {
            runtime_protocol_range: Some(ProtocolRangeV1 {
                minimum_major: 2,
                maximum_major: 2,
                minimum_revision: 1,
            }),
            ..ModuleDescriptorV1::default()
        };
        assert!(matches!(
            select_managed_control_transport(&descriptor),
            Ok(ManagedControlTransportMajorV1::CorrelatedV2)
        ));

        descriptor.runtime_protocol_range = Some(ProtocolRangeV1 {
            minimum_major: 1,
            maximum_major: 2,
            minimum_revision: 1,
        });
        assert!(matches!(
            select_managed_control_transport(&descriptor),
            Err(ManagedControlTransportErrorV2::InvalidTransportSelection)
        ));
    }

    fn rejected_response() -> ManagedRuntimeControlResponseV1 {
        ManagedRuntimeControlResponseV1 {
            result: None,
            error_code: "REJECTED".to_owned(),
        }
    }

    fn client_delivery_request(request_id: u64) -> ManagedRuntimeControlRequestV1 {
        ManagedRuntimeControlRequestV1 {
            operation: Some(Operation::ClientDelivery(
                ManagedRuntimeClientDeliveryRequestV1 {
                    request: Some(ModuleClientRequestV1 {
                        protocol_major: 1,
                        request_id,
                        ..ModuleClientRequestV1::default()
                    }),
                },
            )),
        }
    }

    #[test]
    fn bootstrap_dispatcher_returns_typed_unavailable_without_losing_outer_response() {
        let (left, right) = UnixStream::pair().expect("control pair");
        let child = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(left);
            let mut dispatcher = RejectManagedControlRequestsV2;
            channel.request(
                [1; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                ready_request(),
                |channel, id, request| dispatcher.dispatch_request(channel, id, request),
            )
        });

        let mut channel = ManagedControlChannelV2::new(right);
        let response = channel
            .request(
                [2; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                client_delivery_request(77),
                |channel, id, _| channel.write_response(id, rejected_response()),
            )
            .expect("typed bootstrap response");
        let Some(ControlResult::ClientDelivery(delivery)) = response.result else {
            panic!("bootstrap response is not a client delivery");
        };
        let response = delivery.response.expect("module response");
        assert_eq!(response.request_id, 77);
        assert_eq!(response.error_code, "RUNTIME_UNAVAILABLE");
        assert!(response.response_payload.is_empty());

        assert_eq!(
            child
                .join()
                .expect("join child")
                .expect("outer correlated response")
                .error_code,
            "REJECTED"
        );
    }

    #[test]
    fn concurrent_opposite_direction_requests_keep_their_responses_correlated() {
        let (left, right) = UnixStream::pair().expect("control pair");
        let peer = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(right);
            channel.request(
                [2; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                ready_request(),
                |channel, id, _| channel.write_response(id, rejected_response()),
            )
        });

        let mut channel = ManagedControlChannelV2::new(left);
        let response = channel
            .request(
                [1; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                ready_request(),
                |channel, id, _| channel.write_response(id, rejected_response()),
            )
            .expect("correlated response");

        assert_eq!(response.error_code, "REJECTED");
        assert_eq!(
            peer.join()
                .expect("peer join")
                .expect("peer response")
                .error_code,
            "REJECTED"
        );
    }

    #[test]
    fn buffers_an_outer_response_while_a_nested_request_is_in_flight() {
        let (left, right) = UnixStream::pair().expect("control pair");
        let peer = thread::spawn(move || -> Result<(), ManagedControlTransportErrorV2> {
            let mut channel = ManagedControlChannelV2::new(right);
            let (outer_id, _) = channel.receive_request()?;
            channel.write_request([2; MANAGED_CONTROL_CORRELATION_ID_BYTES], ready_request())?;
            let (nested_id, _) = channel.receive_request()?;
            channel.write_response(outer_id, rejected_response())?;
            channel.write_response(nested_id, rejected_response())?;
            let frame = channel.read_frame()?;
            assert_eq!(
                frame.correlation_id,
                vec![2; MANAGED_CONTROL_CORRELATION_ID_BYTES]
            );
            assert!(matches!(frame.frame, Some(Frame::Response(_))));
            Ok(())
        });

        let mut channel = ManagedControlChannelV2::new(left);
        let response = channel
            .request(
                [1; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                ready_request(),
                |channel, request_id, _| {
                    let nested = channel.request(
                        [3; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                        ready_request(),
                        |channel, nested_request_id, _| {
                            channel.write_response(nested_request_id, rejected_response())
                        },
                    )?;
                    assert_eq!(nested.error_code, "REJECTED");
                    channel.write_response(request_id, rejected_response())
                },
            )
            .expect("outer response");

        assert_eq!(response.error_code, "REJECTED");
        peer.join().expect("peer join").expect("peer response");
    }

    #[test]
    fn rejects_a_duplicate_pending_correlation_id() {
        let (stream, _peer) = UnixStream::pair().expect("control pair");
        let mut channel = ManagedControlChannelV2::new(stream);
        channel
            .begin_pending([7; MANAGED_CONTROL_CORRELATION_ID_BYTES])
            .expect("first pending request");

        assert!(matches!(
            channel.begin_pending([7; MANAGED_CONTROL_CORRELATION_ID_BYTES]),
            Err(ManagedControlTransportErrorV2::DuplicateCorrelationId)
        ));
    }

    #[test]
    fn receives_only_typed_requests_with_their_correlation_id() {
        let (writer, reader) = UnixStream::pair().expect("control pair");
        let writer = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(writer);
            channel
                .write_request([9; MANAGED_CONTROL_CORRELATION_ID_BYTES], ready_request())
                .expect("write request");
        });
        let mut channel = ManagedControlChannelV2::new(reader);
        let (correlation_id, request) = channel.receive_request().expect("receive request");

        assert_eq!(correlation_id, [9; MANAGED_CONTROL_CORRELATION_ID_BYTES]);
        assert!(request.operation.is_some());
        writer.join().expect("writer join");
    }

    #[test]
    fn peer_close_probe_preserves_pending_control_frame() {
        let (reader, writer) = UnixStream::pair().expect("control pair");
        let mut writer_channel = ManagedControlChannelV2::new(writer);
        writer_channel
            .write_request([9; MANAGED_CONTROL_CORRELATION_ID_BYTES], ready_request())
            .expect("write request");
        let mut reader_channel = ManagedControlChannelV2::new(reader);

        assert!(
            !reader_channel
                .peer_closed_preserving_frames()
                .expect("probe live peer")
        );
        let (correlation_id, request) = reader_channel
            .receive_request()
            .expect("probe did not consume request");
        assert_eq!(correlation_id, [9; MANAGED_CONTROL_CORRELATION_ID_BYTES]);
        assert!(matches!(request.operation, Some(Operation::Ready(_))));

        drop(writer_channel);
        assert!(
            reader_channel
                .peer_closed_preserving_frames()
                .expect("probe closed peer")
        );
    }

    #[test]
    fn retains_a_partial_nonblocking_frame_until_the_full_request_arrives() {
        let (reader, mut writer) = UnixStream::pair().expect("control pair");
        reader.set_nonblocking(true).expect("nonblocking reader");
        let frame = ManagedRuntimeControlFrameV2 {
            transport_major: MANAGED_CONTROL_TRANSPORT_MAJOR_V2,
            correlation_id: vec![4; MANAGED_CONTROL_CORRELATION_ID_BYTES],
            frame: Some(Frame::Request(ready_request())),
        }
        .encode_to_vec();
        let mut encoded = Vec::with_capacity(frame.len() + 1);
        encoded.push(u8::try_from(frame.len()).expect("small test frame"));
        encoded.extend_from_slice(&frame);
        let split = 3;
        writer
            .write_all(&encoded[..split])
            .expect("partial frame write");

        let mut channel = ManagedControlChannelV2::new(reader);
        assert!(
            channel
                .try_receive_request()
                .expect("partial request")
                .is_none()
        );

        writer
            .write_all(&encoded[split..])
            .expect("remaining frame write");
        let (correlation_id, request) = channel
            .try_receive_request()
            .expect("complete request")
            .expect("request available");
        assert_eq!(correlation_id, [4; MANAGED_CONTROL_CORRELATION_ID_BYTES]);
        assert!(matches!(request.operation, Some(Operation::Ready(_))));
    }

    #[test]
    fn resumes_a_partial_nonblocking_write_without_duplicating_frame_bytes() {
        struct PartialWouldBlockWriter {
            bytes: Vec<u8>,
            writes: usize,
        }

        impl Write for PartialWouldBlockWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.writes += 1;
                if self.writes == 2 {
                    return Err(Error::from(ErrorKind::WouldBlock));
                }
                let count = bytes.len().min(3);
                self.bytes.extend_from_slice(&bytes[..count]);
                Ok(count)
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let payload = b"nonblocking-control-frame";
        let mut writer = PartialWouldBlockWriter {
            bytes: Vec::new(),
            writes: 0,
        };

        write_length_delimited(&mut writer, payload).expect("bounded nonblocking write");

        assert_eq!(writer.bytes[0], payload.len() as u8);
        assert_eq!(&writer.bytes[1..], payload);
        assert!(writer.writes > 2);
    }

    #[test]
    fn releases_correlation_when_initial_write_fails() {
        let (stream, peer) = UnixStream::pair().expect("control pair");
        drop(peer);
        let mut channel = ManagedControlChannelV2::new(stream);
        let correlation_id = [3; MANAGED_CONTROL_CORRELATION_ID_BYTES];

        assert!(matches!(
            channel.request(correlation_id, ready_request(), |_, _, _| Ok(())),
            Err(ManagedControlTransportErrorV2::Io(_))
        ));
        assert!(channel.begin_pending(correlation_id).is_ok());
    }

    #[test]
    fn generates_distinct_non_zero_private_control_correlation_ids() {
        let first = next_correlation_id();
        let second = next_correlation_id();

        assert_ne!(first, second);
        assert!(first.iter().any(|byte| *byte != 0));
        assert!(second.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn namespaces_control_correlation_ids_by_requesting_process() {
        assert_ne!(
            correlation_id_for_process_sequence(101, 1),
            correlation_id_for_process_sequence(202, 1),
        );
    }

    #[test]
    fn completes_descriptor_and_ready_handshake_with_typed_responses() {
        let (client, server) = UnixStream::pair().expect("control pair");
        let server = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(server);
            let (describe_id, describe) = channel.receive_request().expect("describe request");
            assert!(matches!(describe.operation, Some(Operation::Describe(_))));
            channel
                .write_response(
                    describe_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Describe(DescribeManagedRuntimeResponseV1 {
                            registration_id: "communications".to_owned(),
                            runtime_generation: 1,
                            grant_epoch: 1,
                        })),
                        error_code: String::new(),
                    },
                )
                .expect("describe response");
            let (ready_id, ready) = channel.receive_request().expect("ready request");
            assert!(matches!(ready.operation, Some(Operation::Ready(_))));
            channel
                .write_response(
                    ready_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .expect("ready response");
        });

        let mut channel = ManagedControlChannelV2::new(client);
        let identity = channel
            .describe_managed_runtime(vec![1], vec![])
            .expect("descriptor identity");
        assert_eq!(identity.registration_id, "communications");
        channel
            .signal_ready(ManagedRuntimeReadyRequestV1 {
                registration_id: identity.registration_id,
                runtime_generation: identity.runtime_generation,
                grant_epoch: identity.grant_epoch,
            })
            .expect("ready acknowledgement");
        server.join().expect("server join");
    }
}

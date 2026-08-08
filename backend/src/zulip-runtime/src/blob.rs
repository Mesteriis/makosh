//! Provider-local, in-memory access to owner-authorized Blob data for Zulip.

use makosh_blob_client_contract::{BlobReadError, BlobReadPort};
use makosh_runtime_protocol::v1::{BlobDataOperationV1, BlobDataSessionGrantV1};

use crate::ZulipRuntimeErrorV1;

pub struct ZulipBlobSessionV1 {
    pub blob_ref: String,
    pub grant: BlobDataSessionGrantV1,
    pub channel_binding: Vec<u8>,
    pub declared_size: u64,
}

pub struct ZulipBlobMaterializer<R> {
    reader: R,
    sessions: Vec<ZulipBlobSessionV1>,
}

impl<R> ZulipBlobMaterializer<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            sessions: Vec::new(),
        }
    }

    pub fn register(&mut self, session: ZulipBlobSessionV1) -> Result<(), ZulipRuntimeErrorV1> {
        if session.blob_ref.trim().is_empty()
            || session.channel_binding.is_empty()
            || session.declared_size == 0
            || session.declared_size > 64 * 1024 * 1024
        {
            return Err(ZulipRuntimeErrorV1::Credential);
        }
        if session.grant.operation != BlobDataOperationV1::BlobDataOperationReadRangeV1 as i32
            || session.grant.declared_size != session.declared_size
        {
            return Err(ZulipRuntimeErrorV1::Credential);
        }
        if self
            .sessions
            .iter()
            .any(|existing| existing.blob_ref == session.blob_ref)
        {
            return Err(ZulipRuntimeErrorV1::OperationAlreadyKnown);
        }
        self.sessions.push(session);
        Ok(())
    }
}

pub struct ZulipBlobWriteMaterializer<W> {
    writer: W,
    sessions: Vec<ZulipBlobSessionV1>,
}

impl<W> ZulipBlobWriteMaterializer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            sessions: Vec::new(),
        }
    }

    pub fn register(&mut self, session: ZulipBlobSessionV1) -> Result<(), ZulipRuntimeErrorV1> {
        if session.blob_ref.trim().is_empty()
            || session.channel_binding.is_empty()
            || session.declared_size == 0
            || session.declared_size > 64 * 1024 * 1024
            || session.grant.operation != BlobDataOperationV1::BlobDataOperationWriteV1 as i32
            || session.grant.declared_size != session.declared_size
            || self
                .sessions
                .iter()
                .any(|existing| existing.blob_ref == session.blob_ref)
        {
            return Err(ZulipRuntimeErrorV1::Credential);
        }
        self.sessions.push(session);
        Ok(())
    }
}

impl ZulipBlobWriteMaterializer<makosh_blob_client::BlobDataClient> {
    pub fn write_download(
        &mut self,
        blob_ref: &str,
        bytes: Vec<u8>,
    ) -> Result<(), ZulipRuntimeErrorV1> {
        let index = self
            .sessions
            .iter()
            .position(|session| session.blob_ref == blob_ref)
            .ok_or(ZulipRuntimeErrorV1::Credential)?;
        let session = self.sessions.remove(index);
        if u64::try_from(bytes.len()).ok() != Some(session.declared_size) {
            return Err(ZulipRuntimeErrorV1::Credential);
        }
        self.writer
            .write(session.grant, session.channel_binding, bytes)
            .map_err(|_| ZulipRuntimeErrorV1::Credential)
    }
}

impl<R: BlobReadPort> ZulipBlobMaterializer<R> {
    pub fn take_bytes(&mut self, blob_ref: &str) -> Result<Vec<u8>, ZulipRuntimeErrorV1> {
        let index = self
            .sessions
            .iter()
            .position(|session| session.blob_ref == blob_ref)
            .ok_or(ZulipRuntimeErrorV1::Credential)?;
        let session = self.sessions.remove(index);
        let bytes = self
            .reader
            .read_range(
                session.grant,
                session.channel_binding,
                0,
                session.declared_size,
            )
            .map_err(map_read_error)?;
        if u64::try_from(bytes.len()).ok() != Some(session.declared_size) {
            return Err(ZulipRuntimeErrorV1::Credential);
        }
        Ok(bytes)
    }
}

fn map_read_error(error: BlobReadError) -> ZulipRuntimeErrorV1 {
    match error {
        BlobReadError::Unavailable | BlobReadError::Rejected | BlobReadError::InvalidResponse => {
            ZulipRuntimeErrorV1::Credential
        }
    }
}

//! Fixed-version binary codec for non-content Blob technical metadata.

use makosh_blob_protocol::{BlobBackupClassV1, BlobCustodyScopeV1, BlobRefV1};

use super::record::{BlobMetadataRecordV1, BlobMetadataStateV1};

const MAGIC: &[u8; 8] = b"HBLBM002";
const MAX_RECORD_BYTES: usize = 1024;
const CONTENT_FORMAT_REVISION: u8 = 2;
const CONTENT_KEY_SCHEMA_REVISION: u64 = 1;

pub(super) fn encode(record: &BlobMetadataRecordV1) -> Vec<u8> {
    let reference = record.reference();
    let custody = record.custody();
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(MAGIC);
    bytes.push(CONTENT_FORMAT_REVISION);
    bytes.extend_from_slice(&CONTENT_KEY_SCHEMA_REVISION.to_be_bytes());
    bytes.extend_from_slice(reference.reference_id());
    bytes.push(backup_class_code(reference.backup_class()));
    bytes.extend_from_slice(&reference.declared_size().to_be_bytes());
    append_optional_u64(&mut bytes, reference.expires_at_unix_ms());
    append_text(&mut bytes, reference.owner_id());
    append_text(&mut bytes, custody.custody_scope_id());
    append_state(&mut bytes, record.state());
    bytes
}

pub(super) fn decode(bytes: &[u8]) -> Option<BlobMetadataRecordV1> {
    if bytes.len() > MAX_RECORD_BYTES || bytes.get(..MAGIC.len())? != MAGIC {
        return None;
    }
    let mut cursor = Cursor::new(&bytes[MAGIC.len()..]);
    if cursor.byte()? != CONTENT_FORMAT_REVISION || cursor.u64()? != CONTENT_KEY_SCHEMA_REVISION {
        return None;
    }
    let reference_id = cursor.array()?;
    let backup_class = backup_class(cursor.byte()?)?;
    let declared_size = cursor.u64()?;
    let expires_at_unix_ms = cursor.optional_u64()?;
    let owner_id = cursor.text()?;
    let custody_scope_id = cursor.text()?;
    let state = cursor.state()?;
    if !cursor.is_exhausted() {
        return None;
    }
    let reference = BlobRefV1::new(
        reference_id,
        owner_id.clone(),
        declared_size,
        expires_at_unix_ms,
        backup_class,
    )
    .ok()?;
    let custody = BlobCustodyScopeV1::new(owner_id, custody_scope_id).ok()?;
    BlobMetadataRecordV1::from_parts(reference, custody, state)
}

fn append_optional_u64(target: &mut Vec<u8>, value: Option<u64>) {
    target.push(u8::from(value.is_some()));
    target.extend_from_slice(&value.unwrap_or_default().to_be_bytes());
}

fn append_text(target: &mut Vec<u8>, value: &str) {
    let length = u8::try_from(value.len()).expect("Blob contract identifiers are bounded");
    target.push(length);
    target.extend_from_slice(value.as_bytes());
}

fn append_state(target: &mut Vec<u8>, state: BlobMetadataStateV1) {
    match state {
        BlobMetadataStateV1::PendingWrite => target.extend_from_slice(&[1]),
        BlobMetadataStateV1::Active => target.extend_from_slice(&[2]),
        BlobMetadataStateV1::DeleteReserved { not_before_unix_ms } => {
            target.push(3);
            target.extend_from_slice(&not_before_unix_ms.to_be_bytes());
        }
    }
}

const fn backup_class_code(value: BlobBackupClassV1) -> u8 {
    match value {
        BlobBackupClassV1::Required => 1,
        BlobBackupClassV1::Rebuildable => 2,
        BlobBackupClassV1::Excluded => 3,
    }
}

const fn backup_class(value: u8) -> Option<BlobBackupClassV1> {
    match value {
        1 => Some(BlobBackupClassV1::Required),
        2 => Some(BlobBackupClassV1::Rebuildable),
        3 => Some(BlobBackupClassV1::Excluded),
        _ => None,
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn byte(&mut self) -> Option<u8> {
        let value = *self.bytes.get(self.offset)?;
        self.offset += 1;
        Some(value)
    }

    fn array(&mut self) -> Option<[u8; 16]> {
        self.take(16)?.try_into().ok()
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_be_bytes(self.take(8)?.try_into().ok()?))
    }

    fn optional_u64(&mut self) -> Option<Option<u64>> {
        match self.byte()? {
            0 => self.u64().map(|_| None),
            1 => self.u64().map(Some),
            _ => None,
        }
    }

    fn text(&mut self) -> Option<String> {
        let length = usize::from(self.byte()?);
        std::str::from_utf8(self.take(length)?)
            .ok()
            .map(str::to_owned)
    }

    fn state(&mut self) -> Option<BlobMetadataStateV1> {
        match self.byte()? {
            1 => Some(BlobMetadataStateV1::PendingWrite),
            2 => Some(BlobMetadataStateV1::Active),
            3 => self
                .u64()
                .map(|not_before_unix_ms| BlobMetadataStateV1::DeleteReserved {
                    not_before_unix_ms,
                }),
            _ => None,
        }
    }

    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let end = self.offset.checked_add(length)?;
        let value = self.bytes.get(self.offset..end)?;
        self.offset = end;
        Some(value)
    }

    const fn is_exhausted(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

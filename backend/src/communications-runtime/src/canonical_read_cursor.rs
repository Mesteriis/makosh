//! Opaque, scope-bound continuation cursors for canonical Communications reads.

use makosh_communications_persistence::{CanonicalReadAfterV1, CanonicalReferenceReadAfterV1};
use sha2::{Digest, Sha256};

const PREFIX: &[u8; 4] = b"HCR2";
const SCOPE_BYTES: usize = 16;
const CURSOR_BYTES: usize = 4 + 1 + SCOPE_BYTES + 8 + 16;
const REFERENCE_CURSOR_BYTES: usize = 4 + 1 + SCOPE_BYTES + 8 + 2 + 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CanonicalReadCursorKindV1 {
    Accounts = 1,
    Conversations = 2,
    Messages = 3,
    Participants = 4,
    AttachmentAnchors = 5,
    MessageReferences = 6,
    MessageEvidence = 7,
    Search = 8,
    SavedSearch = 9,
    SavedSearchExecution = 10,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalReadCursorErrorV1 {
    Malformed,
    WrongKind,
    WrongScope,
}

pub fn decode_descending_cursor_v1(
    cursor: &[u8],
    kind: CanonicalReadCursorKindV1,
    scope_parts: &[&[u8]],
) -> Result<Option<CanonicalReadAfterV1>, CanonicalReadCursorErrorV1> {
    if cursor.is_empty() {
        return Ok(None);
    }
    let (observed_at_unix_seconds, canonical_id) = decode_cursor_v1(cursor, kind, scope_parts)?;
    Ok(Some(CanonicalReadAfterV1 {
        observed_at_unix_seconds,
        canonical_id,
    }))
}

pub fn encode_descending_cursor_v1(
    kind: CanonicalReadCursorKindV1,
    scope_parts: &[&[u8]],
    observed_at_unix_seconds: i64,
    canonical_id: [u8; 16],
) -> Vec<u8> {
    encode_cursor_v1(kind, scope_parts, observed_at_unix_seconds, canonical_id)
}

pub fn decode_reference_cursor_v1(
    cursor: &[u8],
    scope_parts: &[&[u8]],
) -> Result<Option<CanonicalReferenceReadAfterV1>, CanonicalReadCursorErrorV1> {
    if cursor.is_empty() {
        return Ok(None);
    }
    if cursor.len() != REFERENCE_CURSOR_BYTES || &cursor[..PREFIX.len()] != PREFIX {
        return Err(CanonicalReadCursorErrorV1::Malformed);
    }
    if cursor[PREFIX.len()] != CanonicalReadCursorKindV1::MessageReferences as u8 {
        return Err(CanonicalReadCursorErrorV1::WrongKind);
    }
    let expected_scope = scope_hash_v1(CanonicalReadCursorKindV1::MessageReferences, scope_parts);
    if cursor[5..21] != expected_scope {
        return Err(CanonicalReadCursorErrorV1::WrongScope);
    }
    let observed_at_unix_seconds = i64::from_be_bytes(
        cursor[21..29]
            .try_into()
            .map_err(|_| CanonicalReadCursorErrorV1::Malformed)?,
    );
    Ok(Some(CanonicalReferenceReadAfterV1 {
        observed_at_unix_seconds,
        reference_kind: i16::from_be_bytes(
            cursor[29..31]
                .try_into()
                .map_err(|_| CanonicalReadCursorErrorV1::Malformed)?,
        ),
        reference_id: cursor[31..63]
            .try_into()
            .map_err(|_| CanonicalReadCursorErrorV1::Malformed)?,
    }))
}

pub fn encode_reference_cursor_v1(
    scope_parts: &[&[u8]],
    observed_at_unix_seconds: i64,
    reference_kind: i16,
    reference_id: [u8; 32],
) -> Vec<u8> {
    let mut cursor = Vec::with_capacity(REFERENCE_CURSOR_BYTES);
    cursor.extend_from_slice(PREFIX);
    cursor.push(CanonicalReadCursorKindV1::MessageReferences as u8);
    cursor.extend_from_slice(&scope_hash_v1(
        CanonicalReadCursorKindV1::MessageReferences,
        scope_parts,
    ));
    cursor.extend_from_slice(&observed_at_unix_seconds.to_be_bytes());
    cursor.extend_from_slice(&reference_kind.to_be_bytes());
    cursor.extend_from_slice(&reference_id);
    cursor
}

fn decode_cursor_v1(
    cursor: &[u8],
    kind: CanonicalReadCursorKindV1,
    scope_parts: &[&[u8]],
) -> Result<(i64, [u8; 16]), CanonicalReadCursorErrorV1> {
    if cursor.len() != CURSOR_BYTES || &cursor[..PREFIX.len()] != PREFIX {
        return Err(CanonicalReadCursorErrorV1::Malformed);
    }
    if cursor[PREFIX.len()] != kind as u8 {
        return Err(CanonicalReadCursorErrorV1::WrongKind);
    }
    let expected_scope = scope_hash_v1(kind, scope_parts);
    if cursor[5..21] != expected_scope {
        return Err(CanonicalReadCursorErrorV1::WrongScope);
    }
    let observed_at_unix_seconds = i64::from_be_bytes(
        cursor[21..29]
            .try_into()
            .map_err(|_| CanonicalReadCursorErrorV1::Malformed)?,
    );
    let canonical_id = cursor[29..45]
        .try_into()
        .map_err(|_| CanonicalReadCursorErrorV1::Malformed)?;
    Ok((observed_at_unix_seconds, canonical_id))
}

fn encode_cursor_v1(
    kind: CanonicalReadCursorKindV1,
    scope_parts: &[&[u8]],
    observed_at_unix_seconds: i64,
    canonical_id: [u8; 16],
) -> Vec<u8> {
    let mut cursor = Vec::with_capacity(CURSOR_BYTES);
    cursor.extend_from_slice(PREFIX);
    cursor.push(kind as u8);
    cursor.extend_from_slice(&scope_hash_v1(kind, scope_parts));
    cursor.extend_from_slice(&observed_at_unix_seconds.to_be_bytes());
    cursor.extend_from_slice(&canonical_id);
    cursor
}

fn scope_hash_v1(kind: CanonicalReadCursorKindV1, scope_parts: &[&[u8]]) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.communications.canonical-read.v2");
    digest.update([kind as u8]);
    for part in scope_parts {
        let part_len = u32::try_from(part.len()).expect("canonical read scope part is bounded");
        digest.update(part_len.to_be_bytes());
        digest.update(part);
    }
    digest.finalize()[..SCOPE_BYTES]
        .try_into()
        .expect("fixed SHA-256 prefix")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trip_is_kind_and_scope_bound() {
        let cursor = encode_descending_cursor_v1(
            CanonicalReadCursorKindV1::Messages,
            &[b"conversation-1"],
            42,
            [9; 16],
        );
        assert_eq!(cursor.len(), CURSOR_BYTES);
        assert_eq!(
            decode_descending_cursor_v1(
                &cursor,
                CanonicalReadCursorKindV1::Messages,
                &[b"conversation-1"],
            ),
            Ok(Some(CanonicalReadAfterV1 {
                observed_at_unix_seconds: 42,
                canonical_id: [9; 16],
            })),
        );
        assert_eq!(
            decode_descending_cursor_v1(
                &cursor,
                CanonicalReadCursorKindV1::Participants,
                &[b"conversation-1"],
            ),
            Err(CanonicalReadCursorErrorV1::WrongKind),
        );
        assert_eq!(
            decode_descending_cursor_v1(
                &cursor,
                CanonicalReadCursorKindV1::Messages,
                &[b"conversation-2"],
            ),
            Err(CanonicalReadCursorErrorV1::WrongScope),
        );
    }

    #[test]
    fn reference_cursor_preserves_the_full_unique_reference_id() {
        let cursor = encode_reference_cursor_v1(&[b"message"], 7, 2, [3; 32]);
        assert_eq!(cursor.len(), REFERENCE_CURSOR_BYTES);
        assert_eq!(
            decode_reference_cursor_v1(&cursor, &[b"message"]),
            Ok(Some(CanonicalReferenceReadAfterV1 {
                observed_at_unix_seconds: 7,
                reference_kind: 2,
                reference_id: [3; 32],
            })),
        );
    }
}

#![forbid(unsafe_code)]

use serde::Serialize;
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-communications-export-core";
pub const MAX_EXPORT_ITEMS_V1: usize = 64;
pub const MAX_EXPORT_SOURCE_BYTES_V1: usize = 16 * 1024 * 1024;
pub const MAX_EXPORT_ARTIFACT_BYTES_V1: usize = 24 * 1024 * 1024;
pub const MAX_PARTICIPANT_DISPLAY_LABEL_BYTES_V1: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceExportDirectionV1 {
    Incoming,
    Outgoing,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvidenceExportBodyV1 {
    AdmittedUtf8(Vec<u8>),
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceExportItemV1 {
    pub message_id: [u8; 16],
    pub conversation_id: [u8; 16],
    pub evidence_id: [u8; 16],
    pub evidence_revision: u64,
    pub direction: EvidenceExportDirectionV1,
    pub occurred_at_unix_seconds: i64,
    pub observed_at_unix_seconds: i64,
    pub participant_display_label: Option<String>,
    pub body: EvidenceExportBodyV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceExportManifestV1 {
    pub export_id: [u8; 16],
    pub logical_owner_id: String,
    pub created_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceExportEncodeErrorV1 {
    InvalidManifest,
    InvalidItem,
    DuplicateMessage,
    InvalidUtf8,
    SourceLimitExceeded,
    ArtifactLimitExceeded,
    Serialization,
}

pub fn encode_evidence_export_jsonl_v1(
    manifest: EvidenceExportManifestV1,
    items: &[EvidenceExportItemV1],
) -> Result<Vec<u8>, EvidenceExportEncodeErrorV1> {
    if !valid_id(&manifest.export_id)
        || !valid_logical_owner_id(&manifest.logical_owner_id)
        || manifest.created_at_unix_seconds <= 0
    {
        return Err(EvidenceExportEncodeErrorV1::InvalidManifest);
    }
    if items.is_empty() || items.len() > MAX_EXPORT_ITEMS_V1 {
        return Err(EvidenceExportEncodeErrorV1::InvalidItem);
    }

    let mut source_bytes = 0usize;
    for (index, item) in items.iter().enumerate() {
        validate_item(item)?;
        if items[..index]
            .iter()
            .any(|seen| seen.message_id == item.message_id)
        {
            return Err(EvidenceExportEncodeErrorV1::DuplicateMessage);
        }
        if let EvidenceExportBodyV1::AdmittedUtf8(bytes) = &item.body {
            source_bytes = source_bytes
                .checked_add(bytes.len())
                .ok_or(EvidenceExportEncodeErrorV1::SourceLimitExceeded)?;
            if source_bytes > MAX_EXPORT_SOURCE_BYTES_V1 {
                return Err(EvidenceExportEncodeErrorV1::SourceLimitExceeded);
            }
        }
    }

    let mut output = Vec::new();
    append_line(
        &mut output,
        &ManifestRecordV1 {
            record_type: "manifest",
            schema: "makosh.communications.evidence-export.v1",
            export_id: hex(&manifest.export_id),
            logical_owner_id: &manifest.logical_owner_id,
            created_at_unix_seconds: manifest.created_at_unix_seconds,
            item_count: items.len(),
        },
    )?;
    for item in items {
        let (body_state, body_utf8) = match &item.body {
            EvidenceExportBodyV1::AdmittedUtf8(bytes) => (
                "admitted_utf8",
                Some(
                    std::str::from_utf8(bytes)
                        .map_err(|_| EvidenceExportEncodeErrorV1::InvalidUtf8)?,
                ),
            ),
            EvidenceExportBodyV1::Unavailable => ("unavailable", None),
        };
        append_line(
            &mut output,
            &ItemRecordV1 {
                record_type: "message",
                message_id: hex(&item.message_id),
                conversation_id: hex(&item.conversation_id),
                evidence_id: hex(&item.evidence_id),
                evidence_revision: item.evidence_revision,
                direction: item.direction,
                occurred_at_unix_seconds: item.occurred_at_unix_seconds,
                observed_at_unix_seconds: item.observed_at_unix_seconds,
                participant_display_label: item.participant_display_label.as_deref(),
                body_state,
                body_utf8,
            },
        )?;
    }
    let content_sha256 = hex(&Sha256::digest(&output));
    append_line(
        &mut output,
        &ChecksumRecordV1 {
            record_type: "checksum",
            algorithm: "sha256",
            preceding_bytes_sha256: &content_sha256,
        },
    )?;
    if output.len() > MAX_EXPORT_ARTIFACT_BYTES_V1 {
        return Err(EvidenceExportEncodeErrorV1::ArtifactLimitExceeded);
    }
    Ok(output)
}

fn validate_item(item: &EvidenceExportItemV1) -> Result<(), EvidenceExportEncodeErrorV1> {
    if !valid_id(&item.message_id)
        || !valid_id(&item.conversation_id)
        || !valid_id(&item.evidence_id)
        || item.evidence_revision == 0
        || item.occurred_at_unix_seconds <= 0
        || item.observed_at_unix_seconds <= 0
        || item
            .participant_display_label
            .as_ref()
            .is_some_and(|label| {
                label.is_empty()
                    || label.len() > MAX_PARTICIPANT_DISPLAY_LABEL_BYTES_V1
                    || label.chars().any(char::is_control)
            })
    {
        return Err(EvidenceExportEncodeErrorV1::InvalidItem);
    }
    Ok(())
}

fn append_line<T: Serialize>(
    output: &mut Vec<u8>,
    value: &T,
) -> Result<(), EvidenceExportEncodeErrorV1> {
    serde_json::to_writer(&mut *output, value)
        .map_err(|_| EvidenceExportEncodeErrorV1::Serialization)?;
    output.push(b'\n');
    if output.len() > MAX_EXPORT_ARTIFACT_BYTES_V1 {
        return Err(EvidenceExportEncodeErrorV1::ArtifactLimitExceeded);
    }
    Ok(())
}

fn valid_id(id: &[u8; 16]) -> bool {
    id.iter().any(|byte| *byte != 0)
}

fn valid_logical_owner_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[derive(Serialize)]
struct ManifestRecordV1<'a> {
    record_type: &'a str,
    schema: &'a str,
    export_id: String,
    logical_owner_id: &'a str,
    created_at_unix_seconds: i64,
    item_count: usize,
}

#[derive(Serialize)]
struct ItemRecordV1<'a> {
    record_type: &'a str,
    message_id: String,
    conversation_id: String,
    evidence_id: String,
    evidence_revision: u64,
    direction: EvidenceExportDirectionV1,
    occurred_at_unix_seconds: i64,
    observed_at_unix_seconds: i64,
    participant_display_label: Option<&'a str>,
    body_state: &'a str,
    body_utf8: Option<&'a str>,
}

#[derive(Serialize)]
struct ChecksumRecordV1<'a> {
    record_type: &'a str,
    algorithm: &'a str,
    preceding_bytes_sha256: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(body: EvidenceExportBodyV1) -> EvidenceExportItemV1 {
        EvidenceExportItemV1 {
            message_id: [2; 16],
            conversation_id: [3; 16],
            evidence_id: [4; 16],
            evidence_revision: 1,
            direction: EvidenceExportDirectionV1::Incoming,
            occurred_at_unix_seconds: 1_700_000_000,
            observed_at_unix_seconds: 1_700_000_001,
            participant_display_label: Some("Alice".to_owned()),
            body,
        }
    }

    #[test]
    fn encoder_is_deterministic_and_contains_no_blob_or_provider_fields() {
        let manifest = EvidenceExportManifestV1 {
            export_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            created_at_unix_seconds: 1_800_000_000,
        };
        let first = encode_evidence_export_jsonl_v1(
            manifest.clone(),
            &[item(EvidenceExportBodyV1::AdmittedUtf8(
                "hello".as_bytes().to_vec(),
            ))],
        )
        .expect("export");
        let second = encode_evidence_export_jsonl_v1(
            manifest,
            &[item(EvidenceExportBodyV1::AdmittedUtf8(
                "hello".as_bytes().to_vec(),
            ))],
        )
        .expect("export");
        assert_eq!(first, second);
        let text = std::str::from_utf8(&first).expect("utf8");
        assert!(text.contains("\"body_utf8\":\"hello\""));
        assert!(text.contains("\"record_type\":\"checksum\""));
        assert!(!text.contains("blob"));
        assert!(!text.contains("provider"));
    }

    #[test]
    fn encoder_rejects_invalid_utf8_and_duplicate_messages() {
        assert_eq!(
            encode_evidence_export_jsonl_v1(
                EvidenceExportManifestV1 {
                    export_id: [1; 16],
                    logical_owner_id: "owner-1".to_owned(),
                    created_at_unix_seconds: 1_800_000_000,
                },
                &[item(EvidenceExportBodyV1::AdmittedUtf8(vec![0xff]))],
            ),
            Err(EvidenceExportEncodeErrorV1::InvalidUtf8)
        );
        let value = item(EvidenceExportBodyV1::Unavailable);
        assert_eq!(
            encode_evidence_export_jsonl_v1(
                EvidenceExportManifestV1 {
                    export_id: [1; 16],
                    logical_owner_id: "owner-1".to_owned(),
                    created_at_unix_seconds: 1_800_000_000,
                },
                &[value.clone(), value],
            ),
            Err(EvidenceExportEncodeErrorV1::DuplicateMessage)
        );
    }
}

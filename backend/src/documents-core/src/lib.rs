#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-documents-core";
pub const MAX_TITLE_BYTES_V1: usize = 512;
pub const MAX_DESCRIPTION_BYTES_V1: usize = 16_384;
pub const MAX_MEDIA_TYPE_BYTES_V1: usize = 256;
pub const MAX_FILE_NAME_BYTES_V1: usize = 1_024;
pub const MAX_SOURCE_ID_BYTES_V1: usize = 128;
pub const MAX_DOCUMENT_BYTES_V1: u64 = 1 << 40;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentStateV1 {
    Active,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentCustodyStateV1 {
    Unbound,
    Bound,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentSourceStateV1 {
    Active,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentBlobBindingV1 {
    pub blob_reference_id: [u8; 16],
    pub declared_size: u64,
    pub content_sha256: [u8; 32],
    pub state: DocumentCustodyStateV1,
    pub updated_at_document_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSourceV1 {
    pub source_id: [u8; 16],
    pub source_owner_id: String,
    pub source_record_id: String,
    pub source_revision: u64,
    pub evidence_digest: [u8; 32],
    pub state: DocumentSourceStateV1,
    pub updated_at_document_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentV1 {
    pub document_id: [u8; 16],
    pub logical_owner_id: String,
    pub title: String,
    pub description: String,
    pub media_type: String,
    pub original_file_name: String,
    pub declared_size: u64,
    pub content_sha256: [u8; 32],
    pub state: DocumentStateV1,
    pub custody: Option<DocumentBlobBindingV1>,
    pub document_revision: u64,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateDocumentV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub title: String,
    pub description: String,
    pub media_type: String,
    pub original_file_name: String,
    pub declared_size: u64,
    pub content_sha256: [u8; 32],
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentCoreErrorV1 {
    InvalidInput,
    RevisionConflict,
    StateConflict,
    RevisionOverflow,
}

impl DocumentV1 {
    pub fn create(input: CreateDocumentV1) -> Result<Self, DocumentCoreErrorV1> {
        validate_profile(DocumentProfileRefV1 {
            owner: &input.logical_owner_id,
            title: &input.title,
            description: &input.description,
            media_type: &input.media_type,
            file_name: &input.original_file_name,
            declared_size: input.declared_size,
            digest: &input.content_sha256,
            timestamp: input.created_at_unix_millis,
        })?;
        let CreateDocumentV1 {
            logical_owner_id,
            operation_id,
            title,
            description,
            media_type,
            original_file_name,
            declared_size,
            content_sha256,
            created_at_unix_millis,
        } = input;
        if operation_id.iter().all(|byte| *byte == 0) {
            return Err(DocumentCoreErrorV1::InvalidInput);
        }
        Ok(Self {
            document_id: derive_id(
                "makosh.documents.document.v1",
                &[logical_owner_id.as_bytes(), &operation_id],
            ),
            logical_owner_id,
            title,
            description,
            media_type,
            original_file_name,
            declared_size,
            content_sha256,
            state: DocumentStateV1::Active,
            custody: None,
            document_revision: 1,
            created_at_unix_millis,
            updated_at_unix_millis: created_at_unix_millis,
        })
    }

    pub fn update_metadata(
        &mut self,
        expected_revision: u64,
        title: Option<String>,
        description: Option<String>,
        media_type: Option<String>,
        original_file_name: Option<String>,
        updated_at_unix_millis: i64,
    ) -> Result<(), DocumentCoreErrorV1> {
        self.require_revision(expected_revision)?;
        let next_revision = next_revision(self.document_revision)?;
        let next_title = title.unwrap_or_else(|| self.title.clone());
        let next_description = description.unwrap_or_else(|| self.description.clone());
        let next_media_type = media_type.unwrap_or_else(|| self.media_type.clone());
        let next_file_name = original_file_name.unwrap_or_else(|| self.original_file_name.clone());
        validate_profile(DocumentProfileRefV1 {
            owner: &self.logical_owner_id,
            title: &next_title,
            description: &next_description,
            media_type: &next_media_type,
            file_name: &next_file_name,
            declared_size: self.declared_size,
            digest: &self.content_sha256,
            timestamp: updated_at_unix_millis,
        })?;
        if updated_at_unix_millis < self.updated_at_unix_millis {
            return Err(DocumentCoreErrorV1::InvalidInput);
        }
        self.title = next_title;
        self.description = next_description;
        self.media_type = next_media_type;
        self.original_file_name = next_file_name;
        self.updated_at_unix_millis = updated_at_unix_millis;
        self.document_revision = next_revision;
        Ok(())
    }

    pub fn set_state(
        &mut self,
        expected_revision: u64,
        state: DocumentStateV1,
        changed_at_unix_millis: i64,
    ) -> Result<(), DocumentCoreErrorV1> {
        self.require_revision(expected_revision)?;
        if self.state == state || changed_at_unix_millis < self.updated_at_unix_millis {
            return Err(DocumentCoreErrorV1::StateConflict);
        }
        self.document_revision = next_revision(self.document_revision)?;
        self.state = state;
        self.updated_at_unix_millis = changed_at_unix_millis;
        Ok(())
    }

    pub fn attach_blob(
        &mut self,
        expected_revision: u64,
        blob_reference_id: [u8; 16],
        declared_size: u64,
        content_sha256: [u8; 32],
        changed_at_unix_millis: i64,
    ) -> Result<(), DocumentCoreErrorV1> {
        self.require_revision(expected_revision)?;
        if self.state != DocumentStateV1::Active
            || self
                .custody
                .as_ref()
                .is_some_and(|value| value.state == DocumentCustodyStateV1::Bound)
            || blob_reference_id.iter().all(|byte| *byte == 0)
            || declared_size != self.declared_size
            || content_sha256 != self.content_sha256
            || changed_at_unix_millis < self.updated_at_unix_millis
        {
            return Err(DocumentCoreErrorV1::StateConflict);
        }
        let revision = next_revision(self.document_revision)?;
        self.custody = Some(DocumentBlobBindingV1 {
            blob_reference_id,
            declared_size,
            content_sha256,
            state: DocumentCustodyStateV1::Bound,
            updated_at_document_revision: revision,
        });
        self.document_revision = revision;
        self.updated_at_unix_millis = changed_at_unix_millis;
        Ok(())
    }

    pub fn release_blob(
        &mut self,
        expected_revision: u64,
        blob_reference_id: [u8; 16],
        changed_at_unix_millis: i64,
    ) -> Result<(), DocumentCoreErrorV1> {
        self.require_revision(expected_revision)?;
        let revision = next_revision(self.document_revision)?;
        let custody = self
            .custody
            .as_mut()
            .ok_or(DocumentCoreErrorV1::StateConflict)?;
        if custody.state != DocumentCustodyStateV1::Bound
            || custody.blob_reference_id != blob_reference_id
            || changed_at_unix_millis < self.updated_at_unix_millis
        {
            return Err(DocumentCoreErrorV1::StateConflict);
        }
        custody.state = DocumentCustodyStateV1::Released;
        custody.updated_at_document_revision = revision;
        self.document_revision = revision;
        self.updated_at_unix_millis = changed_at_unix_millis;
        Ok(())
    }

    fn require_revision(&self, expected: u64) -> Result<(), DocumentCoreErrorV1> {
        (expected != 0 && expected == self.document_revision)
            .then_some(())
            .ok_or(DocumentCoreErrorV1::RevisionConflict)
    }
}

pub fn add_source_v1(
    document: &mut DocumentV1,
    expected_revision: u64,
    source_owner_id: String,
    source_record_id: String,
    source_revision: u64,
    evidence_digest: [u8; 32],
    changed_at_unix_millis: i64,
) -> Result<DocumentSourceV1, DocumentCoreErrorV1> {
    document.require_revision(expected_revision)?;
    if !valid_identifier(&source_owner_id)
        || source_record_id.is_empty()
        || source_record_id.len() > MAX_SOURCE_ID_BYTES_V1
        || source_revision == 0
        || evidence_digest.iter().all(|byte| *byte == 0)
        || changed_at_unix_millis < document.updated_at_unix_millis
    {
        return Err(DocumentCoreErrorV1::InvalidInput);
    }
    let revision = next_revision(document.document_revision)?;
    let source_id = derive_id(
        "makosh.documents.source.v1",
        &[
            document.logical_owner_id.as_bytes(),
            &document.document_id,
            source_owner_id.as_bytes(),
            source_record_id.as_bytes(),
        ],
    );
    document.document_revision = revision;
    document.updated_at_unix_millis = changed_at_unix_millis;
    Ok(DocumentSourceV1 {
        source_id,
        source_owner_id,
        source_record_id,
        source_revision,
        evidence_digest,
        state: DocumentSourceStateV1::Active,
        updated_at_document_revision: revision,
    })
}

pub fn remove_source_v1(
    document: &mut DocumentV1,
    source: &mut DocumentSourceV1,
    expected_revision: u64,
    changed_at_unix_millis: i64,
) -> Result<(), DocumentCoreErrorV1> {
    document.require_revision(expected_revision)?;
    if source.state != DocumentSourceStateV1::Active
        || changed_at_unix_millis < document.updated_at_unix_millis
    {
        return Err(DocumentCoreErrorV1::StateConflict);
    }
    let revision = next_revision(document.document_revision)?;
    source.state = DocumentSourceStateV1::Removed;
    source.updated_at_document_revision = revision;
    document.document_revision = revision;
    document.updated_at_unix_millis = changed_at_unix_millis;
    Ok(())
}

struct DocumentProfileRefV1<'a> {
    owner: &'a str,
    title: &'a str,
    description: &'a str,
    media_type: &'a str,
    file_name: &'a str,
    declared_size: u64,
    digest: &'a [u8; 32],
    timestamp: i64,
}

fn validate_profile(profile: DocumentProfileRefV1<'_>) -> Result<(), DocumentCoreErrorV1> {
    let DocumentProfileRefV1 {
        owner,
        title,
        description,
        media_type,
        file_name,
        declared_size,
        digest,
        timestamp,
    } = profile;
    if !valid_identifier(owner)
        || title.trim().is_empty()
        || title.len() > MAX_TITLE_BYTES_V1
        || description.len() > MAX_DESCRIPTION_BYTES_V1
        || media_type.is_empty()
        || media_type.len() > MAX_MEDIA_TYPE_BYTES_V1
        || file_name.is_empty()
        || file_name.len() > MAX_FILE_NAME_BYTES_V1
        || !(1..=MAX_DOCUMENT_BYTES_V1).contains(&declared_size)
        || digest.iter().all(|byte| *byte == 0)
        || timestamp <= 0
    {
        return Err(DocumentCoreErrorV1::InvalidInput);
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn next_revision(value: u64) -> Result<u64, DocumentCoreErrorV1> {
    value
        .checked_add(1)
        .ok_or(DocumentCoreErrorV1::RevisionOverflow)
}

fn derive_id(domain: &str, chunks: &[&[u8]]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(domain.as_bytes());
    for chunk in chunks {
        hash.update((chunk.len() as u64).to_be_bytes());
        hash.update(chunk);
    }
    let digest = hash.finalize();
    let mut result = [0_u8; 16];
    result.copy_from_slice(&digest[..16]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> DocumentV1 {
        DocumentV1::create(CreateDocumentV1 {
            logical_owner_id: "owner-1".to_owned(),
            operation_id: [1; 16],
            title: "Document".to_owned(),
            description: "Description".to_owned(),
            media_type: "application/pdf".to_owned(),
            original_file_name: "document.pdf".to_owned(),
            declared_size: 10,
            content_sha256: [2; 32],
            created_at_unix_millis: 1_000,
        })
        .expect("document")
    }

    #[test]
    fn metadata_lifecycle_and_custody_are_revisioned_without_bytes() {
        let mut value = document();
        value
            .update_metadata(1, Some("Updated".into()), None, None, None, 1_001)
            .unwrap();
        value.attach_blob(2, [3; 16], 10, [2; 32], 1_002).unwrap();
        assert_eq!(
            value.custody.as_ref().unwrap().state,
            DocumentCustodyStateV1::Bound
        );
        value.release_blob(3, [3; 16], 1_003).unwrap();
        value
            .set_state(4, DocumentStateV1::Archived, 1_004)
            .unwrap();
        assert_eq!(value.document_revision, 5);
        assert_eq!(
            value.custody.unwrap().state,
            DocumentCustodyStateV1::Released
        );
    }

    #[test]
    fn provenance_and_overflow_fail_closed_without_partial_mutation() {
        let mut value = document();
        let mut source = add_source_v1(
            &mut value,
            1,
            "mail".into(),
            "message-1".into(),
            1,
            [4; 32],
            1_001,
        )
        .unwrap();
        remove_source_v1(&mut value, &mut source, 2, 1_002).unwrap();
        assert_eq!(source.state, DocumentSourceStateV1::Removed);
        value.document_revision = u64::MAX;
        let before = value.clone();
        assert_eq!(
            value.update_metadata(
                u64::MAX,
                Some("No mutation".into()),
                None,
                None,
                None,
                1_003
            ),
            Err(DocumentCoreErrorV1::RevisionOverflow)
        );
        assert_eq!(value, before);
    }
}

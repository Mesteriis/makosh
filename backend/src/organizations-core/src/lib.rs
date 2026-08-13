#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-organizations-core";
pub const STABLE_ID_BYTES_V1: usize = 16;
pub const EVIDENCE_DIGEST_BYTES_V1: usize = 32;
pub const MAX_LOGICAL_OWNER_ID_BYTES_V1: usize = 128;
pub const MAX_DISPLAY_NAME_CHARS_V1: usize = 240;
pub const MAX_LEGAL_NAME_CHARS_V1: usize = 320;
pub const MAX_DESCRIPTION_CHARS_V1: usize = 8_000;
pub const MAX_WEBSITE_CHARS_V1: usize = 512;
pub const MAX_INDUSTRY_CHARS_V1: usize = 160;
pub const MAX_PUBLIC_SOURCE_ID_BYTES_V1: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrganizationTimestampV1 {
    pub unix_seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizationStateV1 {
    Active,
    Archived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizationSourceStateV1 {
    Active,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationSourceV1 {
    pub source_id: [u8; STABLE_ID_BYTES_V1],
    pub source_owner_id: String,
    pub source_record_id: String,
    pub source_revision: u64,
    pub evidence_digest: [u8; EVIDENCE_DIGEST_BYTES_V1],
    pub state: OrganizationSourceStateV1,
    pub updated_at_organization_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationDraftV1 {
    pub operation_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub display_name: String,
    pub legal_name: String,
    pub description: String,
    pub website: String,
    pub industry: String,
    pub country_code: String,
    pub created_at: OrganizationTimestampV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationRecordV1 {
    pub organization_id: [u8; STABLE_ID_BYTES_V1],
    pub logical_owner_id: String,
    pub display_name: String,
    pub legal_name: String,
    pub description: String,
    pub website: String,
    pub industry: String,
    pub country_code: String,
    pub state: OrganizationStateV1,
    pub organization_revision: u64,
    pub sources: Vec<OrganizationSourceV1>,
    pub created_at: OrganizationTimestampV1,
    pub updated_at: OrganizationTimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrganizationLifecycleErrorV1 {
    InvalidOwner,
    InvalidOperationId,
    InvalidOrganizationId,
    InvalidDisplayName,
    InvalidProfile,
    InvalidWebsite,
    InvalidCountryCode,
    InvalidTimestamp,
    InvalidRevision,
    RevisionOverflow,
    InvalidStateTransition,
    Archived,
    InvalidSource,
    SourceExists,
    SourceNotFound,
    SourceRemoved,
}

pub fn derive_organization_id_v1(
    logical_owner_id: &str,
    operation_id: &[u8; STABLE_ID_BYTES_V1],
) -> Result<[u8; STABLE_ID_BYTES_V1], OrganizationLifecycleErrorV1> {
    if !valid_owner(logical_owner_id) {
        return Err(OrganizationLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(operation_id) {
        return Err(OrganizationLifecycleErrorV1::InvalidOperationId);
    }
    Ok(derive_id(
        b"makosh.organizations.organization-id.v1\0",
        &[logical_owner_id.as_bytes(), operation_id],
    ))
}

pub fn derive_organization_source_id_v1(
    organization_id: &[u8; STABLE_ID_BYTES_V1],
    source_owner_id: &str,
    source_record_id: &str,
) -> Result<[u8; STABLE_ID_BYTES_V1], OrganizationLifecycleErrorV1> {
    if !nonzero(organization_id)
        || !valid_public_id(source_owner_id)
        || !valid_public_id(source_record_id)
    {
        return Err(OrganizationLifecycleErrorV1::InvalidSource);
    }
    Ok(derive_id(
        b"makosh.organizations.source-id.v1\0",
        &[
            organization_id,
            source_owner_id.as_bytes(),
            source_record_id.as_bytes(),
        ],
    ))
}

pub fn create_organization_v1(
    draft: OrganizationDraftV1,
) -> Result<OrganizationRecordV1, OrganizationLifecycleErrorV1> {
    validate_profile(
        &draft.display_name,
        &draft.legal_name,
        &draft.description,
        &draft.website,
        &draft.industry,
        &draft.country_code,
    )?;
    validate_timestamp(draft.created_at)?;
    let organization = OrganizationRecordV1 {
        organization_id: derive_organization_id_v1(&draft.logical_owner_id, &draft.operation_id)?,
        logical_owner_id: draft.logical_owner_id,
        display_name: draft.display_name,
        legal_name: draft.legal_name,
        description: draft.description,
        website: draft.website,
        industry: draft.industry,
        country_code: draft.country_code,
        state: OrganizationStateV1::Active,
        organization_revision: 1,
        sources: Vec::new(),
        created_at: draft.created_at,
        updated_at: draft.created_at,
    };
    validate_organization_record_v1(&organization)?;
    Ok(organization)
}

#[allow(clippy::too_many_arguments)]
pub fn update_organization_v1(
    organization: &mut OrganizationRecordV1,
    expected_revision: u64,
    display_name: Option<String>,
    legal_name: Option<String>,
    description: Option<String>,
    website: Option<String>,
    industry: Option<String>,
    country_code: Option<String>,
    updated_at: OrganizationTimestampV1,
) -> Result<(), OrganizationLifecycleErrorV1> {
    require_active(organization, expected_revision, updated_at)?;
    if display_name.is_none()
        && legal_name.is_none()
        && description.is_none()
        && website.is_none()
        && industry.is_none()
        && country_code.is_none()
    {
        return Err(OrganizationLifecycleErrorV1::InvalidProfile);
    }
    let next_display_name = display_name.unwrap_or_else(|| organization.display_name.clone());
    let next_legal_name = legal_name.unwrap_or_else(|| organization.legal_name.clone());
    let next_description = description.unwrap_or_else(|| organization.description.clone());
    let next_website = website.unwrap_or_else(|| organization.website.clone());
    let next_industry = industry.unwrap_or_else(|| organization.industry.clone());
    let next_country_code = country_code.unwrap_or_else(|| organization.country_code.clone());
    validate_profile(
        &next_display_name,
        &next_legal_name,
        &next_description,
        &next_website,
        &next_industry,
        &next_country_code,
    )?;
    let revision = next_revision(organization)?;
    organization.display_name = next_display_name;
    organization.legal_name = next_legal_name;
    organization.description = next_description;
    organization.website = next_website;
    organization.industry = next_industry;
    organization.country_code = next_country_code;
    organization.organization_revision = revision;
    organization.updated_at = updated_at;
    Ok(())
}

pub fn set_organization_state_v1(
    organization: &mut OrganizationRecordV1,
    expected_revision: u64,
    state: OrganizationStateV1,
    changed_at: OrganizationTimestampV1,
) -> Result<(), OrganizationLifecycleErrorV1> {
    require_revision_and_time(organization, expected_revision, changed_at)?;
    if organization.state == state {
        return Err(OrganizationLifecycleErrorV1::InvalidStateTransition);
    }
    let revision = next_revision(organization)?;
    organization.state = state;
    organization.organization_revision = revision;
    organization.updated_at = changed_at;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn add_organization_source_v1(
    organization: &mut OrganizationRecordV1,
    expected_revision: u64,
    source_owner_id: String,
    source_record_id: String,
    source_revision: u64,
    evidence_digest: [u8; EVIDENCE_DIGEST_BYTES_V1],
    changed_at: OrganizationTimestampV1,
) -> Result<[u8; STABLE_ID_BYTES_V1], OrganizationLifecycleErrorV1> {
    require_active(organization, expected_revision, changed_at)?;
    if source_revision == 0 || !nonzero(&evidence_digest) {
        return Err(OrganizationLifecycleErrorV1::InvalidSource);
    }
    let source_id = derive_organization_source_id_v1(
        &organization.organization_id,
        &source_owner_id,
        &source_record_id,
    )?;
    if organization
        .sources
        .iter()
        .any(|source| source.source_id == source_id)
    {
        return Err(OrganizationLifecycleErrorV1::SourceExists);
    }
    let revision = next_revision(organization)?;
    organization.sources.push(OrganizationSourceV1 {
        source_id,
        source_owner_id,
        source_record_id,
        source_revision,
        evidence_digest,
        state: OrganizationSourceStateV1::Active,
        updated_at_organization_revision: revision,
    });
    organization.sources.sort_by_key(|source| source.source_id);
    organization.organization_revision = revision;
    organization.updated_at = changed_at;
    Ok(source_id)
}

pub fn remove_organization_source_v1(
    organization: &mut OrganizationRecordV1,
    expected_revision: u64,
    source_id: [u8; STABLE_ID_BYTES_V1],
    changed_at: OrganizationTimestampV1,
) -> Result<(), OrganizationLifecycleErrorV1> {
    require_active(organization, expected_revision, changed_at)?;
    let index = organization
        .sources
        .iter()
        .position(|source| source.source_id == source_id)
        .ok_or(OrganizationLifecycleErrorV1::SourceNotFound)?;
    if organization.sources[index].state != OrganizationSourceStateV1::Active {
        return Err(OrganizationLifecycleErrorV1::SourceRemoved);
    }
    let revision = next_revision(organization)?;
    organization.sources[index].state = OrganizationSourceStateV1::Removed;
    organization.sources[index].updated_at_organization_revision = revision;
    organization.organization_revision = revision;
    organization.updated_at = changed_at;
    Ok(())
}

pub fn validate_organization_record_v1(
    organization: &OrganizationRecordV1,
) -> Result<(), OrganizationLifecycleErrorV1> {
    if !valid_owner(&organization.logical_owner_id) {
        return Err(OrganizationLifecycleErrorV1::InvalidOwner);
    }
    if !nonzero(&organization.organization_id) || organization.organization_revision == 0 {
        return Err(OrganizationLifecycleErrorV1::InvalidOrganizationId);
    }
    validate_profile(
        &organization.display_name,
        &organization.legal_name,
        &organization.description,
        &organization.website,
        &organization.industry,
        &organization.country_code,
    )?;
    validate_timestamp(organization.created_at)?;
    validate_timestamp(organization.updated_at)?;
    if organization.updated_at < organization.created_at {
        return Err(OrganizationLifecycleErrorV1::InvalidTimestamp);
    }
    for source in &organization.sources {
        if source.source_revision == 0
            || !nonzero(&source.source_id)
            || !nonzero(&source.evidence_digest)
            || !valid_public_id(&source.source_owner_id)
            || !valid_public_id(&source.source_record_id)
            || source.updated_at_organization_revision == 0
            || source.updated_at_organization_revision > organization.organization_revision
        {
            return Err(OrganizationLifecycleErrorV1::InvalidSource);
        }
    }
    Ok(())
}

fn require_active(
    organization: &OrganizationRecordV1,
    expected_revision: u64,
    changed_at: OrganizationTimestampV1,
) -> Result<(), OrganizationLifecycleErrorV1> {
    require_revision_and_time(organization, expected_revision, changed_at)?;
    if organization.state != OrganizationStateV1::Active {
        return Err(OrganizationLifecycleErrorV1::Archived);
    }
    Ok(())
}

fn require_revision_and_time(
    organization: &OrganizationRecordV1,
    expected_revision: u64,
    changed_at: OrganizationTimestampV1,
) -> Result<(), OrganizationLifecycleErrorV1> {
    if expected_revision == 0 || expected_revision != organization.organization_revision {
        return Err(OrganizationLifecycleErrorV1::InvalidRevision);
    }
    validate_timestamp(changed_at)?;
    if changed_at < organization.updated_at {
        return Err(OrganizationLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn next_revision(organization: &OrganizationRecordV1) -> Result<u64, OrganizationLifecycleErrorV1> {
    organization
        .organization_revision
        .checked_add(1)
        .ok_or(OrganizationLifecycleErrorV1::RevisionOverflow)
}

fn validate_profile(
    display_name: &str,
    legal_name: &str,
    description: &str,
    website: &str,
    industry: &str,
    country_code: &str,
) -> Result<(), OrganizationLifecycleErrorV1> {
    validate_required_text(display_name, MAX_DISPLAY_NAME_CHARS_V1)
        .map_err(|_| OrganizationLifecycleErrorV1::InvalidDisplayName)?;
    for (value, limit) in [
        (legal_name, MAX_LEGAL_NAME_CHARS_V1),
        (description, MAX_DESCRIPTION_CHARS_V1),
        (industry, MAX_INDUSTRY_CHARS_V1),
    ] {
        validate_optional_text(value, limit)?;
    }
    if !website.is_empty()
        && (website.chars().count() > MAX_WEBSITE_CHARS_V1
            || !(website.starts_with("https://") || website.starts_with("http://"))
            || website.chars().any(char::is_control))
    {
        return Err(OrganizationLifecycleErrorV1::InvalidWebsite);
    }
    if !country_code.is_empty()
        && (country_code.len() != 2 || !country_code.bytes().all(|byte| byte.is_ascii_uppercase()))
    {
        return Err(OrganizationLifecycleErrorV1::InvalidCountryCode);
    }
    Ok(())
}

fn validate_required_text(value: &str, limit: usize) -> Result<(), OrganizationLifecycleErrorV1> {
    if value.trim().is_empty()
        || value.chars().count() > limit
        || value.chars().any(char::is_control)
    {
        return Err(OrganizationLifecycleErrorV1::InvalidProfile);
    }
    Ok(())
}

fn validate_optional_text(value: &str, limit: usize) -> Result<(), OrganizationLifecycleErrorV1> {
    if value.chars().count() > limit || value.chars().any(char::is_control) {
        return Err(OrganizationLifecycleErrorV1::InvalidProfile);
    }
    Ok(())
}

fn validate_timestamp(value: OrganizationTimestampV1) -> Result<(), OrganizationLifecycleErrorV1> {
    if value.unix_seconds <= 0 || !(0..1_000_000_000).contains(&value.nanos) {
        return Err(OrganizationLifecycleErrorV1::InvalidTimestamp);
    }
    Ok(())
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_LOGICAL_OWNER_ID_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_public_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PUBLIC_SOURCE_ID_BYTES_V1
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'/')
        })
}

fn nonzero(value: &[u8]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn derive_id(domain: &[u8], fields: &[&[u8]]) -> [u8; STABLE_ID_BYTES_V1] {
    let mut hash = Sha256::new();
    hash.update(domain);
    for field in fields {
        hash.update((field.len() as u64).to_be_bytes());
        hash.update(field);
    }
    hash.finalize()[..STABLE_ID_BYTES_V1]
        .try_into()
        .expect("fixed digest")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(seconds: i64) -> OrganizationTimestampV1 {
        OrganizationTimestampV1 {
            unix_seconds: seconds,
            nanos: 0,
        }
    }

    fn organization() -> OrganizationRecordV1 {
        create_organization_v1(OrganizationDraftV1 {
            operation_id: [1; 16],
            logical_owner_id: "owner-1".to_owned(),
            display_name: "Makosh Labs".to_owned(),
            legal_name: "Makosh Labs SL".to_owned(),
            description: "Owner-local organization".to_owned(),
            website: "https://example.invalid".to_owned(),
            industry: "Software".to_owned(),
            country_code: "ES".to_owned(),
            created_at: timestamp(10),
        })
        .expect("organization")
    }

    #[test]
    fn lifecycle_and_source_provenance_are_revisioned() {
        let mut value = organization();
        let source_id = add_organization_source_v1(
            &mut value,
            1,
            "knowledge".to_owned(),
            "public-note-1".to_owned(),
            3,
            [7; 32],
            timestamp(11),
        )
        .expect("source");
        assert_eq!(value.organization_revision, 2);
        assert_eq!(
            add_organization_source_v1(
                &mut value,
                2,
                "knowledge".to_owned(),
                "public-note-1".to_owned(),
                3,
                [7; 32],
                timestamp(12),
            ),
            Err(OrganizationLifecycleErrorV1::SourceExists)
        );
        remove_organization_source_v1(&mut value, 2, source_id, timestamp(12))
            .expect("remove source");
        set_organization_state_v1(&mut value, 3, OrganizationStateV1::Archived, timestamp(13))
            .expect("archive");
        assert_eq!(
            update_organization_v1(
                &mut value,
                4,
                Some("Changed".to_owned()),
                None,
                None,
                None,
                None,
                None,
                timestamp(14),
            ),
            Err(OrganizationLifecycleErrorV1::Archived)
        );
        set_organization_state_v1(&mut value, 4, OrganizationStateV1::Active, timestamp(14))
            .expect("restore");
        update_organization_v1(
            &mut value,
            5,
            Some("Changed".to_owned()),
            None,
            None,
            None,
            None,
            None,
            timestamp(15),
        )
        .expect("update");
        assert_eq!(value.organization_revision, 6);
        validate_organization_record_v1(&value).expect("valid record");
    }

    #[test]
    fn stable_ids_validation_and_overflow_fail_closed() {
        let first = derive_organization_id_v1("owner-1", &[1; 16]).expect("id");
        let second = derive_organization_id_v1("owner-1", &[1; 16]).expect("id");
        assert_eq!(first, second);
        assert_ne!(
            first,
            derive_organization_id_v1("owner-2", &[1; 16]).expect("id")
        );
        assert_eq!(
            create_organization_v1(OrganizationDraftV1 {
                operation_id: [1; 16],
                logical_owner_id: "owner-1".to_owned(),
                display_name: String::new(),
                legal_name: String::new(),
                description: String::new(),
                website: String::new(),
                industry: String::new(),
                country_code: String::new(),
                created_at: timestamp(10),
            }),
            Err(OrganizationLifecycleErrorV1::InvalidDisplayName)
        );
        let mut value = organization();
        value.organization_revision = u64::MAX;
        let before = value.clone();
        assert_eq!(
            set_organization_state_v1(
                &mut value,
                u64::MAX,
                OrganizationStateV1::Archived,
                timestamp(11),
            ),
            Err(OrganizationLifecycleErrorV1::RevisionOverflow)
        );
        assert_eq!(value, before);
        assert_eq!(
            update_organization_v1(
                &mut value,
                u64::MAX,
                Some("Would mutate".to_owned()),
                None,
                None,
                None,
                None,
                None,
                timestamp(11),
            ),
            Err(OrganizationLifecycleErrorV1::RevisionOverflow)
        );
        assert_eq!(value, before);
    }
}

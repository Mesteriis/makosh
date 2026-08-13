#![forbid(unsafe_code)]

pub const PACKAGE: &str = "makosh-mail-persons-sync-api";
pub const MAIL_PERSONS_SYNC_OWNER_V1: &str = "mail_persons_sync";
pub const MAIL_PERSONS_SYNC_CONTRACT_MAJOR_V1: u32 = 1;
pub const MAIL_PERSONS_SYNC_CONTRACT_REVISION_V1: u32 = 2;
pub const MAIL_PERSONS_SYNC_MAX_PAGE_SIZE_V1: u32 = 500;
// A run is deliberately capped at 4,096 provider pages (2,048,000 public
// source observations at the exact page bound) before a new run is required.
pub const MAIL_PERSONS_SYNC_MAX_PAGES_PER_RUN_V1: u64 = 4_096;

pub mod wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.mail_persons_sync.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/mail_persons_sync_schema.rs"));

pub const MAIL_PERSONS_SYNC_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/mail-persons-sync-v1.bin"));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncContractV1 {
    PageReceipt,
    RunResult,
}

impl MailPersonsSyncContractV1 {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PageReceipt => "mail_persons_sync_page_receipt",
            Self::RunResult => "mail_persons_sync_run_result",
        }
    }

    #[must_use]
    pub fn reference(self) -> makosh_runtime_protocol::v1::ContractReferenceV1 {
        makosh_runtime_protocol::v1::ContractReferenceV1 {
            owner: MAIL_PERSONS_SYNC_OWNER_V1.to_owned(),
            name: self.name().to_owned(),
            major: MAIL_PERSONS_SYNC_CONTRACT_MAJOR_V1,
            revision: MAIL_PERSONS_SYNC_CONTRACT_REVISION_V1,
            schema_sha256: MAIL_PERSONS_SYNC_SCHEMA_SHA256_V1.to_vec(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailPersonsSyncContractErrorV1 {
    InvalidPayload,
}

const PAGE_FINGERPRINT_DOMAIN_V1: &[u8] = b"makosh.mail-persons-sync.page-fingerprint.v1";
const PAGE_RECEIPT_ID_DOMAIN_V1: &[u8] = b"makosh.mail-persons-sync.page-receipt-id.v1";
const RUN_RESULT_ID_DOMAIN_V1: &[u8] = b"makosh.mail-persons-sync.run-result-id.v1";

pub fn validate_mail_persons_sync_page_identity_v1(
    value: &wire::MailPersonsSyncPageIdentityV1,
) -> Result<(), MailPersonsSyncContractErrorV1> {
    if valid_owner(&value.logical_owner_id)
        && valid_id16(&value.account_public_id)
        && valid_id16(&value.run_id)
        && value.page_sequence > 0
    {
        Ok(())
    } else {
        Err(MailPersonsSyncContractErrorV1::InvalidPayload)
    }
}

pub fn mail_persons_sync_page_fingerprint_v1(
    identity: &wire::MailPersonsSyncPageIdentityV1,
    page_digest: [u8; 32],
) -> Result<[u8; 32], MailPersonsSyncContractErrorV1> {
    page_identity_hash(PAGE_FINGERPRINT_DOMAIN_V1, identity, page_digest)
}

pub fn mail_persons_sync_page_receipt_id_v1(
    identity: &wire::MailPersonsSyncPageIdentityV1,
    page_digest: [u8; 32],
) -> Result<[u8; 16], MailPersonsSyncContractErrorV1> {
    let digest = page_identity_hash(PAGE_RECEIPT_ID_DOMAIN_V1, identity, page_digest)?;
    Ok(digest[..16].try_into().expect("SHA-256 prefix"))
}

pub fn mail_persons_sync_run_result_id_v1(
    logical_owner_id: &str,
    account_public_id: &[u8],
    run_id: &[u8],
) -> Result<[u8; 16], MailPersonsSyncContractErrorV1> {
    if !valid_owner(logical_owner_id) || !valid_id16(account_public_id) || !valid_id16(run_id) {
        return Err(MailPersonsSyncContractErrorV1::InvalidPayload);
    }
    let mut digest = sha2::Sha256::new();
    use sha2::Digest;
    digest_part(&mut digest, RUN_RESULT_ID_DOMAIN_V1);
    digest_part(&mut digest, logical_owner_id.as_bytes());
    digest_part(&mut digest, account_public_id);
    digest_part(&mut digest, run_id);
    let digest: [u8; 32] = digest.finalize().into();
    Ok(digest[..16].try_into().expect("SHA-256 prefix"))
}

fn page_identity_hash(
    domain: &[u8],
    identity: &wire::MailPersonsSyncPageIdentityV1,
    page_digest: [u8; 32],
) -> Result<[u8; 32], MailPersonsSyncContractErrorV1> {
    use sha2::Digest;
    validate_mail_persons_sync_page_identity_v1(identity)?;
    if page_digest.iter().all(|byte| *byte == 0) {
        return Err(MailPersonsSyncContractErrorV1::InvalidPayload);
    }
    let mut digest = sha2::Sha256::new();
    digest_part(&mut digest, domain);
    digest_part(&mut digest, identity.logical_owner_id.as_bytes());
    digest_part(&mut digest, &identity.account_public_id);
    digest_part(&mut digest, &identity.run_id);
    digest.update(identity.page_sequence.to_be_bytes());
    digest_part(&mut digest, &page_digest);
    Ok(digest.finalize().into())
}

fn digest_part(digest: &mut sha2::Sha256, value: &[u8]) {
    use sha2::Digest;
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

pub fn validate_mail_persons_sync_page_receipt_v1(
    value: &wire::MailPersonsSyncPageReceiptV1,
) -> Result<(), MailPersonsSyncContractErrorV1> {
    let identity = wire::MailPersonsSyncPageIdentityV1 {
        logical_owner_id: value.logical_owner_id.clone(),
        account_public_id: value.account_public_id.clone(),
        run_id: value.run_id.clone(),
        page_sequence: value.page_sequence,
    };
    let page_digest: [u8; 32] = value
        .page_digest
        .as_slice()
        .try_into()
        .map_err(|_| MailPersonsSyncContractErrorV1::InvalidPayload)?;
    let source_count = value
        .observed_sources
        .checked_add(value.updated_sources)
        .and_then(|count| count.checked_add(value.removed_sources));
    if value.receipt_id == mail_persons_sync_page_receipt_id_v1(&identity, page_digest)?
        && source_count == Some(value.persons_commands)
        && value.persons_commands <= MAIL_PERSONS_SYNC_MAX_PAGE_SIZE_V1
        && valid_timestamp(value.completed_at.as_ref())
    {
        Ok(())
    } else {
        Err(MailPersonsSyncContractErrorV1::InvalidPayload)
    }
}

pub fn validate_mail_persons_sync_run_result_v1(
    value: &wire::MailPersonsSyncRunResultV1,
) -> Result<(), MailPersonsSyncContractErrorV1> {
    use wire::{MailPersonsSyncRejectCodeV1 as RejectCode, MailPersonsSyncRunOutcomeV1 as Outcome};

    let outcome = Outcome::try_from(value.outcome)
        .map_err(|_| MailPersonsSyncContractErrorV1::InvalidPayload)?;
    let code = RejectCode::try_from(value.code)
        .map_err(|_| MailPersonsSyncContractErrorV1::InvalidPayload)?;
    let exact_pair = match outcome {
        Outcome::MailPersonsSyncRunOutcomeSucceeded => {
            value.processed_pages > 0 && code == RejectCode::MailPersonsSyncRejectCodeUnspecified
        }
        Outcome::MailPersonsSyncRunOutcomeRejected => {
            code != RejectCode::MailPersonsSyncRejectCodeUnspecified
        }
        Outcome::MailPersonsSyncRunOutcomeUnspecified => false,
    };
    let sources_within_pages = value
        .processed_pages
        .checked_mul(u64::from(MAIL_PERSONS_SYNC_MAX_PAGE_SIZE_V1))
        .is_some_and(|maximum| value.processed_sources <= maximum);
    if value.result_id
        == mail_persons_sync_run_result_id_v1(
            &value.logical_owner_id,
            &value.account_public_id,
            &value.run_id,
        )?
        && exact_pair
        && value.processed_pages <= MAIL_PERSONS_SYNC_MAX_PAGES_PER_RUN_V1
        && sources_within_pages
        && valid_timestamp(value.completed_at.as_ref())
    {
        Ok(())
    } else {
        Err(MailPersonsSyncContractErrorV1::InvalidPayload)
    }
}

fn valid_id16(value: &[u8]) -> bool {
    value.len() == 16 && value.iter().any(|byte| *byte != 0)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_timestamp(value: Option<&prost_types::Timestamp>) -> bool {
    value.is_some_and(|value| value.seconds > 0 && (0..1_000_000_000).contains(&value.nanos))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page_identity(account: u8) -> wire::MailPersonsSyncPageIdentityV1 {
        wire::MailPersonsSyncPageIdentityV1 {
            logical_owner_id: "owner-1".to_owned(),
            account_public_id: vec![account; 16],
            run_id: vec![2; 16],
            page_sequence: 1,
        }
    }

    #[test]
    fn workflow_contract_is_bounded_and_content_negative() {
        let source = include_str!("../proto/makosh/mail_persons_sync/v1/sync.proto");
        for message in ["MailPersonsSyncPageReceiptV1", "MailPersonsSyncRunResultV1"] {
            assert!(source.contains(&format!("message {message}")), "{message}");
        }
        for forbidden in [
            "provider_entry_id",
            "provider_etag",
            "continuation_cursor",
            "normalized_email",
            "normalized_phone",
            "credential",
            "private_locator",
            "raw_payload",
            "error_detail",
        ] {
            assert!(!source.to_lowercase().contains(forbidden), "{forbidden}");
        }
        assert_eq!(MAIL_PERSONS_SYNC_MAX_PAGE_SIZE_V1, 500);
    }

    #[test]
    fn page_receipt_counts_are_exact_and_bounded() {
        let identity = page_identity(3);
        let page_digest = [5; 32];
        let receipt = wire::MailPersonsSyncPageReceiptV1 {
            receipt_id: mail_persons_sync_page_receipt_id_v1(&identity, page_digest)
                .expect("receipt ID")
                .to_vec(),
            run_id: identity.run_id.clone(),
            logical_owner_id: identity.logical_owner_id.clone(),
            page_sequence: identity.page_sequence,
            observed_sources: 2,
            updated_sources: 3,
            removed_sources: 4,
            persons_commands: 9,
            page_digest: page_digest.to_vec(),
            completed_at: Some(prost_types::Timestamp {
                seconds: 1,
                nanos: 0,
            }),
            account_public_id: identity.account_public_id,
        };
        validate_mail_persons_sync_page_receipt_v1(&receipt).expect("bounded receipt");
        let mut invalid = receipt;
        invalid.persons_commands = 501;
        assert!(validate_mail_persons_sync_page_receipt_v1(&invalid).is_err());
    }

    #[test]
    fn run_result_requires_exact_outcome_code_pairing() {
        let account_public_id = vec![3; 16];
        let run_id = vec![2; 16];
        let success = wire::MailPersonsSyncRunResultV1 {
            result_id: mail_persons_sync_run_result_id_v1("owner-1", &account_public_id, &run_id)
                .expect("result ID")
                .to_vec(),
            run_id,
            logical_owner_id: "owner-1".to_owned(),
            outcome: wire::MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeSucceeded as i32,
            processed_pages: 1,
            processed_sources: 2,
            code: wire::MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeUnspecified as i32,
            completed_at: Some(prost_types::Timestamp {
                seconds: 1,
                nanos: 0,
            }),
            account_public_id,
        };
        validate_mail_persons_sync_run_result_v1(&success).expect("success result");
        let mut invalid = success;
        invalid.code = wire::MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeConflict as i32;
        assert!(validate_mail_persons_sync_run_result_v1(&invalid).is_err());
    }

    #[test]
    fn run_result_rejects_page_source_bounds_and_checked_overflow() {
        assert_eq!(MAIL_PERSONS_SYNC_MAX_PAGES_PER_RUN_V1, 4_096);
        let account_public_id = vec![3; 16];
        let run_id = vec![2; 16];
        let result_id = mail_persons_sync_run_result_id_v1("owner-1", &account_public_id, &run_id)
            .expect("result ID")
            .to_vec();
        let result = |processed_pages, processed_sources| wire::MailPersonsSyncRunResultV1 {
            result_id: result_id.clone(),
            run_id: run_id.clone(),
            logical_owner_id: "owner-1".to_owned(),
            outcome: wire::MailPersonsSyncRunOutcomeV1::MailPersonsSyncRunOutcomeSucceeded as i32,
            processed_pages,
            processed_sources,
            code: wire::MailPersonsSyncRejectCodeV1::MailPersonsSyncRejectCodeUnspecified as i32,
            completed_at: Some(prost_types::Timestamp {
                seconds: 1,
                nanos: 0,
            }),
            account_public_id: account_public_id.clone(),
        };
        validate_mail_persons_sync_run_result_v1(&result(
            MAIL_PERSONS_SYNC_MAX_PAGES_PER_RUN_V1,
            MAIL_PERSONS_SYNC_MAX_PAGES_PER_RUN_V1 * u64::from(MAIL_PERSONS_SYNC_MAX_PAGE_SIZE_V1),
        ))
        .expect("exact upper bound");
        assert!(validate_mail_persons_sync_run_result_v1(&result(1, 501)).is_err());
        assert!(
            validate_mail_persons_sync_run_result_v1(&result(
                MAIL_PERSONS_SYNC_MAX_PAGES_PER_RUN_V1 + 1,
                0,
            ))
            .is_err()
        );
        assert!(validate_mail_persons_sync_run_result_v1(&result(u64::MAX, 0)).is_err());
    }

    #[test]
    fn page_and_run_identities_bind_owner_account_run_page_and_digest() {
        let identity = page_identity(3);
        let digest = [5; 32];
        let fingerprint =
            mail_persons_sync_page_fingerprint_v1(&identity, digest).expect("fingerprint");
        assert_eq!(
            fingerprint,
            mail_persons_sync_page_fingerprint_v1(&identity, digest).expect("stable")
        );
        let receipt = mail_persons_sync_page_receipt_id_v1(&identity, digest).expect("receipt ID");
        assert_eq!(
            receipt,
            mail_persons_sync_page_receipt_id_v1(&identity, digest).expect("stable receipt")
        );

        let mut other_account = identity.clone();
        other_account.account_public_id = vec![4; 16];
        let mut other_owner = identity.clone();
        other_owner.logical_owner_id = "owner-2".to_owned();
        assert_ne!(
            fingerprint,
            mail_persons_sync_page_fingerprint_v1(&other_account, digest).expect("other account")
        );
        assert_ne!(
            receipt,
            mail_persons_sync_page_receipt_id_v1(&other_owner, digest).expect("other owner")
        );

        let run = mail_persons_sync_run_result_id_v1(
            &identity.logical_owner_id,
            &identity.account_public_id,
            &identity.run_id,
        )
        .expect("run result ID");
        assert_ne!(
            run,
            mail_persons_sync_run_result_id_v1(
                &other_account.logical_owner_id,
                &other_account.account_public_id,
                &other_account.run_id,
            )
            .expect("other account")
        );
        assert_ne!(
            run,
            mail_persons_sync_run_result_id_v1(
                &other_owner.logical_owner_id,
                &other_owner.account_public_id,
                &other_owner.run_id,
            )
            .expect("other owner")
        );
    }
}

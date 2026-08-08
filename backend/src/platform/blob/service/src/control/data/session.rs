//! Verifies one-use, Kernel-signed Blob data sessions before any content access.

use std::collections::HashMap;

use makosh_blob_protocol::{
    BlobAccessFenceV1, BlobBackupClassV1, BlobCustodyScopeV1, BlobQuotaGrantV1, BlobRefV1,
};
use makosh_runtime_protocol::v1::{
    BlobBackupClassV1 as WireBackupClass, BlobCustodyReleaseGrantV1, BlobCustodyReleaseReasonV1,
    BlobCustodySourceProofKindV1, BlobCustodySourceProofV1, BlobCustodyTransferGrantV1,
    BlobDataOperationV1, BlobDataSessionGrantV1,
};
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use prost::Message;
use sha2::{Digest, Sha256};

const SESSION_TTL_LIMIT_MS: u64 = 30_000;
const MAX_ACTIVE_SESSIONS: usize = 4_096;

pub(super) struct VerifiedBlobDataSessionV1 {
    reference: BlobRefV1,
    access: BlobAccessFenceV1,
    custody: BlobCustodyScopeV1,
    quota: BlobQuotaGrantV1,
    key_revision: u64,
    expected_plaintext_sha256: Option<[u8; 32]>,
}

pub(super) struct VerifiedBlobCustodyTransferV1 {
    source_reference: BlobRefV1,
    source_access: BlobAccessFenceV1,
    source_custody: BlobCustodyScopeV1,
    source_key_revision: u64,
    target_reference: BlobRefV1,
    target_access: BlobAccessFenceV1,
    target_custody: BlobCustodyScopeV1,
    target_quota: BlobQuotaGrantV1,
    target_key_revision: u64,
    expected_plaintext_sha256: [u8; 32],
}

pub(super) struct VerifiedBlobCustodyReleaseV1 {
    operation_id: [u8; 16],
    fingerprint: [u8; 32],
    reference: BlobRefV1,
    access: BlobAccessFenceV1,
    custody: BlobCustodyScopeV1,
    issued_at_unix_ms: u64,
}

impl VerifiedBlobCustodyReleaseV1 {
    pub(super) const fn operation_id(&self) -> &[u8; 16] {
        &self.operation_id
    }
    pub(super) const fn fingerprint(&self) -> &[u8; 32] {
        &self.fingerprint
    }
    pub(super) fn reference(&self) -> &BlobRefV1 {
        &self.reference
    }
    pub(super) fn access(&self) -> &BlobAccessFenceV1 {
        &self.access
    }
    pub(super) fn custody(&self) -> &BlobCustodyScopeV1 {
        &self.custody
    }
    pub(super) const fn issued_at_unix_ms(&self) -> u64 {
        self.issued_at_unix_ms
    }
}

impl VerifiedBlobCustodyTransferV1 {
    pub(super) fn source_reference(&self) -> &BlobRefV1 {
        &self.source_reference
    }
    pub(super) fn source_access(&self) -> &BlobAccessFenceV1 {
        &self.source_access
    }
    pub(super) fn source_custody(&self) -> &BlobCustodyScopeV1 {
        &self.source_custody
    }
    pub(super) const fn source_key_revision(&self) -> u64 {
        self.source_key_revision
    }
    pub(super) fn target_reference(&self) -> &BlobRefV1 {
        &self.target_reference
    }
    pub(super) fn target_access(&self) -> &BlobAccessFenceV1 {
        &self.target_access
    }
    pub(super) fn target_custody(&self) -> &BlobCustodyScopeV1 {
        &self.target_custody
    }
    pub(super) fn target_quota(&self) -> &BlobQuotaGrantV1 {
        &self.target_quota
    }
    pub(super) const fn target_key_revision(&self) -> u64 {
        self.target_key_revision
    }
    pub(super) const fn expected_plaintext_sha256(&self) -> &[u8; 32] {
        &self.expected_plaintext_sha256
    }
}

impl VerifiedBlobDataSessionV1 {
    pub(super) fn reference(&self) -> &BlobRefV1 {
        &self.reference
    }
    pub(super) fn access(&self) -> &BlobAccessFenceV1 {
        &self.access
    }
    pub(super) fn custody(&self) -> &BlobCustodyScopeV1 {
        &self.custody
    }
    pub(super) fn quota(&self) -> &BlobQuotaGrantV1 {
        &self.quota
    }
    pub(super) const fn key_revision(&self) -> u64 {
        self.key_revision
    }
    pub(super) const fn expected_plaintext_sha256(&self) -> Option<&[u8; 32]> {
        self.expected_plaintext_sha256.as_ref()
    }
}

pub(crate) struct BlobDataSessionVerifierV1 {
    kernel_instance_id: String,
    blob_runtime_generation: u64,
    key: VerifyingKey,
    consumed: HashMap<[u8; 16], u64>,
}

impl BlobDataSessionVerifierV1 {
    pub(crate) fn new(
        instance_id: String,
        blob_runtime_generation: u64,
        key_sec1: &[u8],
    ) -> Result<Self, ()> {
        if blob_runtime_generation == 0 {
            return Err(());
        }
        let key = VerifyingKey::from_sec1_bytes(key_sec1).map_err(|_| ())?;
        Ok(Self {
            kernel_instance_id: instance_id,
            blob_runtime_generation,
            key,
            consumed: HashMap::new(),
        })
    }

    pub(super) fn verify(
        &mut self,
        grant: BlobDataSessionGrantV1,
        binding: &[u8],
        expected_operation: BlobDataOperationV1,
        now_unix_ms: u64,
    ) -> Result<VerifiedBlobDataSessionV1, ()> {
        self.prune(now_unix_ms);
        let session_id: [u8; 16] = grant.session_id.as_slice().try_into().map_err(|_| ())?;
        if self.consumed.len() >= MAX_ACTIVE_SESSIONS || self.consumed.contains_key(&session_id) {
            denied("reused_session");
            return Err(());
        }
        validate_signed_grant(
            &self.kernel_instance_id,
            self.blob_runtime_generation,
            &self.key,
            &grant,
            binding,
            expected_operation,
            now_unix_ms,
        )?;
        let verified = decode_grant(&grant).map_err(|_| denied("grant_shape"))?;
        self.consumed.insert(session_id, grant.expires_at_unix_ms);
        Ok(verified)
    }

    pub(super) fn verify_custody_transfer(
        &mut self,
        grant: BlobCustodyTransferGrantV1,
        binding: &[u8],
        now_unix_ms: u64,
    ) -> Result<VerifiedBlobCustodyTransferV1, ()> {
        self.prune(now_unix_ms);
        let session_id: [u8; 16] = grant.session_id.as_slice().try_into().map_err(|_| ())?;
        if self.consumed.len() >= MAX_ACTIVE_SESSIONS || self.consumed.contains_key(&session_id) {
            return Err(());
        }
        validate_signed_transfer(
            &self.kernel_instance_id,
            self.blob_runtime_generation,
            &self.key,
            &grant,
            binding,
            now_unix_ms,
        )?;
        let verified = decode_transfer(&grant)?;
        self.consumed.insert(session_id, grant.expires_at_unix_ms);
        Ok(verified)
    }

    pub(super) fn verify_custody_release(
        &self,
        grant: &BlobCustodyReleaseGrantV1,
        now_unix_ms: u64,
    ) -> Result<VerifiedBlobCustodyReleaseV1, ()> {
        validate_signed_release(
            &self.kernel_instance_id,
            self.blob_runtime_generation,
            &self.key,
            grant,
            now_unix_ms,
        )?;
        decode_release(grant)
    }

    fn prune(&mut self, now_unix_ms: u64) {
        self.consumed.retain(|_, expiry| *expiry > now_unix_ms);
    }
}

fn validate_signed_release(
    instance_id: &str,
    blob_runtime_generation: u64,
    key: &VerifyingKey,
    grant: &BlobCustodyReleaseGrantV1,
    now: u64,
) -> Result<(), ()> {
    let valid_reason = matches!(
        BlobCustodyReleaseReasonV1::try_from(grant.reason),
        Ok(
            BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1
                | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalRejectedV1
                | BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalCancelledV1
        )
    );
    if grant.major != 1
        || grant.kernel_instance_id != instance_id
        || grant.blob_runtime_generation != blob_runtime_generation
        || grant.operation_id.len() != 16
        || grant.operation_id.iter().all(|byte| *byte == 0)
        || grant.reference_id.len() != 16
        || grant.reference_id.iter().all(|byte| *byte == 0)
        || grant.receipt_sha256.len() != 32
        || grant.receipt_sha256.iter().all(|byte| *byte == 0)
        || grant.custody_source_proof_sha256.len() != 32
        || grant
            .custody_source_proof_sha256
            .iter()
            .all(|byte| *byte == 0)
        || !valid_reason
        || grant.issued_at_unix_ms == 0
        || grant.issued_at_unix_ms > now
        || grant.expires_at_unix_ms <= now
        || grant.expires_at_unix_ms
            > grant
                .issued_at_unix_ms
                .checked_add(SESSION_TTL_LIMIT_MS)
                .ok_or(())?
        || grant.kernel_authorization_signature_raw.len() != 64
    {
        return Err(());
    }
    let signature =
        Signature::from_slice(&grant.kernel_authorization_signature_raw).map_err(|_| ())?;
    let mut unsigned = grant.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"makosh.blob-custody-release.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    key.verify(&message, &signature).map_err(|_| ())
}

fn validate_signed_transfer(
    instance_id: &str,
    blob_runtime_generation: u64,
    key: &VerifyingKey,
    grant: &BlobCustodyTransferGrantV1,
    binding: &[u8],
    now: u64,
) -> Result<(), ()> {
    if grant.major != 1
        || grant.kernel_instance_id != instance_id
        || grant.blob_runtime_generation != blob_runtime_generation
        || grant.session_id.len() != 16
        || grant.session_id.iter().all(|byte| *byte == 0)
        || grant.channel_binding_sha256.len() != 32
        || binding.len() != 32
        || grant.evidence_id.len() != 16
        || grant.evidence_id.iter().all(|byte| *byte == 0)
        || grant.evidence_envelope_sha256.len() != 32
        || grant.expires_at_unix_ms <= now
        || grant.expires_at_unix_ms > now.checked_add(SESSION_TTL_LIMIT_MS).ok_or(())?
        || grant.kernel_authorization_signature_raw.len() != 64
    {
        return Err(());
    }
    if Sha256::digest(binding).as_slice() != grant.channel_binding_sha256.as_slice() {
        return Err(());
    }
    let signature =
        Signature::from_slice(&grant.kernel_authorization_signature_raw).map_err(|_| ())?;
    let mut unsigned = grant.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"makosh.blob-custody-transfer.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    key.verify(&message, &signature).map_err(|_| ())
}

fn validate_signed_grant(
    instance_id: &str,
    blob_runtime_generation: u64,
    key: &VerifyingKey,
    grant: &BlobDataSessionGrantV1,
    binding: &[u8],
    expected: BlobDataOperationV1,
    now: u64,
) -> Result<(), ()> {
    if grant.major != 1 {
        return reject("major");
    }
    if grant.kernel_instance_id != instance_id {
        return reject("kernel_instance");
    }
    if grant.blob_runtime_generation != blob_runtime_generation {
        return reject("runtime_generation");
    }
    if grant.session_id.len() != 16 || grant.session_id.iter().all(|byte| *byte == 0) {
        return reject("session_id");
    }
    if grant.channel_binding_sha256.len() != 32 || binding.len() != 32 {
        return reject("binding_shape");
    }
    if grant.expires_at_unix_ms <= now {
        return reject("expired");
    }
    if grant.expires_at_unix_ms > now.checked_add(SESSION_TTL_LIMIT_MS).ok_or(())? {
        return reject("ttl");
    }
    if BlobDataOperationV1::try_from(grant.operation).ok() != Some(expected) {
        return reject("operation");
    }
    if grant.kernel_authorization_signature_raw.len() != 64 {
        return reject("signature_shape");
    }
    if Sha256::digest(binding).as_slice() != grant.channel_binding_sha256.as_slice() {
        return reject("binding");
    }
    let signature = Signature::from_slice(&grant.kernel_authorization_signature_raw)
        .map_err(|_| denied("signature_encoding"))?;
    let mut unsigned = grant.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"makosh.blob-data-session.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    key.verify(&message, &signature)
        .map_err(|_| denied("signature"))
}

fn denied(stage: &str) {
    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
        eprintln!("developer_blob_session_denied stage={stage}");
    }
}

fn reject<T>(stage: &str) -> Result<T, ()> {
    denied(stage);
    Err(())
}

fn decode_release(grant: &BlobCustodyReleaseGrantV1) -> Result<VerifiedBlobCustodyReleaseV1, ()> {
    let operation_id = grant.operation_id.as_slice().try_into().map_err(|_| ())?;
    let reference = BlobRefV1::new(
        grant.reference_id.as_slice().try_into().map_err(|_| ())?,
        grant.owner_id.clone(),
        grant.declared_size,
        (grant.reference_expires_at_unix_ms != 0).then_some(grant.reference_expires_at_unix_ms),
        backup_class(grant.backup_class)?,
    )
    .map_err(|_| ())?;
    let access = BlobAccessFenceV1::new(
        grant.owner_id.clone(),
        grant.registration_id.clone(),
        grant.capability_id.clone(),
        grant.runtime_instance_id.clone(),
        grant.runtime_generation,
        grant.grant_epoch,
    )
    .map_err(|_| ())?;
    let custody = BlobCustodyScopeV1::new(grant.owner_id.clone(), grant.custody_scope_id.clone())
        .map_err(|_| ())?;
    let fingerprint = Sha256::digest(grant.encode_to_vec()).into();
    Ok(VerifiedBlobCustodyReleaseV1 {
        operation_id,
        fingerprint,
        reference,
        access,
        custody,
        issued_at_unix_ms: grant.issued_at_unix_ms,
    })
}

fn decode_grant(grant: &BlobDataSessionGrantV1) -> Result<VerifiedBlobDataSessionV1, ()> {
    let reference_id = grant
        .reference_id
        .as_slice()
        .try_into()
        .map_err(|_| denied("reference_id"))?;
    let backup_class =
        match WireBackupClass::try_from(grant.backup_class).map_err(|_| denied("backup_class"))? {
            WireBackupClass::BlobBackupClassRequiredV1 => BlobBackupClassV1::Required,
            WireBackupClass::BlobBackupClassRebuildableV1 => BlobBackupClassV1::Rebuildable,
            WireBackupClass::BlobBackupClassExcludedV1 => BlobBackupClassV1::Excluded,
            WireBackupClass::BlobBackupClassUnspecifiedV1 => return reject("backup_class"),
        };
    let reference = BlobRefV1::new(
        reference_id,
        grant.owner_id.clone(),
        grant.declared_size,
        (grant.reference_expires_at_unix_ms != 0).then_some(grant.reference_expires_at_unix_ms),
        backup_class,
    )
    .map_err(|_| denied("reference"))?;
    let access = BlobAccessFenceV1::new(
        grant.owner_id.clone(),
        grant.registration_id.clone(),
        grant.capability_id.clone(),
        grant.runtime_instance_id.clone(),
        grant.runtime_generation,
        grant.grant_epoch,
    )
    .map_err(|_| denied("access_fence"))?;
    let custody = BlobCustodyScopeV1::new(grant.owner_id.clone(), grant.custody_scope_id.clone())
        .map_err(|_| denied("custody_scope"))?;
    let quota = BlobQuotaGrantV1::new(
        grant.owner_id.clone(),
        grant.registration_id.clone(),
        grant.capability_id.clone(),
        grant.grant_epoch,
        grant.quota_max_bytes,
        custody.clone(),
    )
    .map_err(|_| denied("quota"))?;
    if !quota.matches(&access) || grant.key_revision == 0 {
        return reject("quota_match");
    }
    let expected_plaintext_sha256 = if grant.expected_plaintext_sha256.is_empty() {
        None
    } else {
        let expected: [u8; 32] = grant
            .expected_plaintext_sha256
            .as_slice()
            .try_into()
            .map_err(|_| denied("expected_plaintext_sha256"))?;
        if expected.iter().all(|byte| *byte == 0) {
            return reject("expected_plaintext_sha256");
        }
        Some(expected)
    };
    Ok(VerifiedBlobDataSessionV1 {
        reference,
        access,
        custody,
        quota,
        key_revision: grant.key_revision,
        expected_plaintext_sha256,
    })
}

fn decode_transfer(
    grant: &BlobCustodyTransferGrantV1,
) -> Result<VerifiedBlobCustodyTransferV1, ()> {
    let source = grant.source.as_ref().ok_or(())?;
    if !valid_source_proof_lineage(source) {
        return Err(());
    }
    let backup_class = backup_class(source.backup_class)?;
    let source_reference = BlobRefV1::new(
        source.reference_id.as_slice().try_into().map_err(|_| ())?,
        source.owner_id.clone(),
        source.declared_size,
        (source.reference_expires_at_unix_ms != 0).then_some(source.reference_expires_at_unix_ms),
        backup_class,
    )
    .map_err(|_| ())?;
    let source_access = BlobAccessFenceV1::new(
        source.owner_id.clone(),
        source.registration_id.clone(),
        source.capability_id.clone(),
        source.runtime_instance_id.clone(),
        source.runtime_generation,
        source.grant_epoch,
    )
    .map_err(|_| ())?;
    let source_custody =
        BlobCustodyScopeV1::new(source.owner_id.clone(), source.custody_scope_id.clone())
            .map_err(|_| ())?;
    let target_reference = BlobRefV1::new(
        grant
            .target_reference_id
            .as_slice()
            .try_into()
            .map_err(|_| ())?,
        grant.target_owner_id.clone(),
        source.declared_size,
        (source.reference_expires_at_unix_ms != 0).then_some(source.reference_expires_at_unix_ms),
        backup_class,
    )
    .map_err(|_| ())?;
    let target_access = BlobAccessFenceV1::new(
        grant.target_owner_id.clone(),
        grant.target_registration_id.clone(),
        grant.target_capability_id.clone(),
        grant.target_runtime_instance_id.clone(),
        grant.target_runtime_generation,
        grant.target_grant_epoch,
    )
    .map_err(|_| ())?;
    let target_custody = BlobCustodyScopeV1::new(
        grant.target_owner_id.clone(),
        grant.target_custody_scope_id.clone(),
    )
    .map_err(|_| ())?;
    let target_quota = BlobQuotaGrantV1::new(
        grant.target_owner_id.clone(),
        grant.target_registration_id.clone(),
        grant.target_capability_id.clone(),
        grant.target_grant_epoch,
        grant.target_quota_max_bytes,
        target_custody.clone(),
    )
    .map_err(|_| ())?;
    // The Kernel-signed transfer grant is the authority for an exact
    // cross-owner handoff. Source and target access fences remain distinct;
    // reapplying owner equality here would reject every target-bound grant.
    if !target_quota.matches(&target_access)
        || source.key_revision == 0
        || grant.target_key_revision == 0
    {
        return Err(());
    }
    Ok(VerifiedBlobCustodyTransferV1 {
        source_reference,
        source_access,
        source_custody,
        source_key_revision: source.key_revision,
        target_reference,
        target_access,
        target_custody,
        target_quota,
        target_key_revision: grant.target_key_revision,
        expected_plaintext_sha256: source
            .receipt_sha256
            .as_slice()
            .try_into()
            .map_err(|_| ())?,
    })
}

fn valid_source_proof_lineage(source: &BlobCustodySourceProofV1) -> bool {
    match BlobCustodySourceProofKindV1::try_from(source.proof_kind).ok() {
        None => false,
        Some(
            BlobCustodySourceProofKindV1::BlobCustodySourceProofKindUnspecifiedV1
            | BlobCustodySourceProofKindV1::BlobCustodySourceProofKindOriginalWriteV1,
        ) => source.delegation_id.is_empty() && source.predecessor_proof_sha256.is_empty(),
        Some(
            BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1,
        ) => {
            source.delegation_id.len() == 16
                && source.delegation_id.iter().any(|byte| *byte != 0)
                && source.predecessor_proof_sha256.len() == 32
                && source
                    .predecessor_proof_sha256
                    .iter()
                    .any(|byte| *byte != 0)
        }
    }
}

fn backup_class(value: i32) -> Result<BlobBackupClassV1, ()> {
    match WireBackupClass::try_from(value).map_err(|_| ())? {
        WireBackupClass::BlobBackupClassRequiredV1 => Ok(BlobBackupClassV1::Required),
        WireBackupClass::BlobBackupClassRebuildableV1 => Ok(BlobBackupClassV1::Rebuildable),
        WireBackupClass::BlobBackupClassExcludedV1 => Ok(BlobBackupClassV1::Excluded),
        WireBackupClass::BlobBackupClassUnspecifiedV1 => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::{
        BlobBackupClassV1 as WireBackupClass, BlobCustodyReleaseGrantV1,
        BlobCustodyReleaseReasonV1, BlobCustodySourceProofKindV1, BlobCustodySourceProofV1,
        BlobCustodyTransferGrantV1,
    };
    use p256::ecdsa::{Signature, SigningKey, signature::Signer};
    use prost::Message;

    use super::{BlobBackupClassV1, BlobDataSessionVerifierV1, decode_transfer};

    #[test]
    fn signed_release_preserves_exact_reference_and_rejects_tampering() {
        let signer = SigningKey::from_bytes((&[9_u8; 32]).into()).expect("signing key");
        let public_key = signer.verifying_key().to_sec1_point(false);
        let mut grant = BlobCustodyReleaseGrantV1 {
            major: 1,
            kernel_instance_id: "kernel-1".to_owned(),
            operation_id: vec![1; 16],
            owner_id: "attachment_security".to_owned(),
            registration_id: "attachment-security-registration".to_owned(),
            capability_id: "attachment_security.blob.release.v1".to_owned(),
            runtime_instance_id: "attachment-security-runtime".to_owned(),
            runtime_generation: 3,
            grant_epoch: 4,
            reference_id: vec![2; 16],
            declared_size: 64,
            receipt_sha256: vec![3; 32],
            custody_source_proof_sha256: vec![4; 32],
            custody_scope_id: "attachment-security-scan".to_owned(),
            reason: BlobCustodyReleaseReasonV1::BlobCustodyReleaseReasonTerminalAcceptedV1 as i32,
            issued_at_unix_ms: 90,
            expires_at_unix_ms: 110,
            blob_runtime_generation: 7,
            backup_class: WireBackupClass::BlobBackupClassRequiredV1 as i32,
            reference_expires_at_unix_ms: 120,
            ..Default::default()
        };
        let mut message = b"makosh.blob-custody-release.v1\0".to_vec();
        message.extend_from_slice(&grant.encode_to_vec());
        let signature: Signature = signer.sign(&message);
        grant.kernel_authorization_signature_raw = signature.to_bytes().to_vec();
        let verifier =
            BlobDataSessionVerifierV1::new("kernel-1".to_owned(), 7, public_key.as_bytes())
                .expect("verifier");

        let verified = verifier
            .verify_custody_release(&grant, 100)
            .expect("verified release");
        assert_eq!(verified.operation_id(), &[1; 16]);
        assert_eq!(verified.reference().reference_id(), &[2; 16]);
        assert_eq!(verified.reference().expires_at_unix_ms(), Some(120));
        assert_eq!(
            verified.reference().backup_class(),
            BlobBackupClassV1::Required
        );

        grant.declared_size = 65;
        assert!(verifier.verify_custody_release(&grant, 100).is_err());
    }

    #[test]
    fn kernel_signed_transfer_keeps_distinct_cross_owner_fences() {
        let mut grant = BlobCustodyTransferGrantV1 {
            source: Some(BlobCustodySourceProofV1 {
                owner_id: "mail".to_owned(),
                registration_id: "mail-registration".to_owned(),
                capability_id: "mail.blob.v1".to_owned(),
                runtime_instance_id: "mail-runtime".to_owned(),
                runtime_generation: 1,
                grant_epoch: 2,
                key_revision: 2,
                reference_id: vec![1; 16],
                declared_size: 4,
                receipt_sha256: vec![2; 32],
                backup_class: WireBackupClass::BlobBackupClassRequiredV1 as i32,
                custody_scope_id: "mail-attachments".to_owned(),
                ..Default::default()
            }),
            target_owner_id: "attachment_security".to_owned(),
            target_registration_id: "dynamic-target-registration".to_owned(),
            target_capability_id: "attachment_security.blob.v1".to_owned(),
            target_runtime_instance_id: "attachment-security-runtime".to_owned(),
            target_runtime_generation: 3,
            target_grant_epoch: 4,
            target_key_revision: 4,
            target_quota_max_bytes: 1024,
            target_reference_id: vec![3; 16],
            target_custody_scope_id: "attachment-security-scan".to_owned(),
            ..Default::default()
        };

        let verified = decode_transfer(&grant).expect("decode cross-owner transfer");
        assert_eq!(verified.source_reference().owner_id(), "mail");
        assert_eq!(
            verified.target_reference().owner_id(),
            "attachment_security"
        );
        assert_eq!(verified.source_access().owner_id(), "mail");
        assert_eq!(verified.target_access().owner_id(), "attachment_security");

        let source = grant.source.as_mut().expect("source");
        source.proof_kind =
            BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1
                as i32;
        assert!(decode_transfer(&grant).is_err());
        let source = grant.source.as_mut().expect("source");
        source.delegation_id = vec![4; 16];
        source.predecessor_proof_sha256 = vec![5; 32];
        assert!(decode_transfer(&grant).is_ok());
    }
}

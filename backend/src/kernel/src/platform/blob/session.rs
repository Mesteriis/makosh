//! Issues one-use Blob data sessions from approved capability quotas.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_kernel_control_store::ModuleBlobOperationV1;
use makosh_kernel_control_store_sqlite::SqliteControlStore;
use makosh_runtime_protocol::v1::{
    BlobCustodySourceProofKindV1, BlobCustodySourceProofV1, BlobCustodyTransferGrantV1,
    BlobDataOperationV1, BlobDataSessionGrantV1, ManagedRuntimeBlobCustodyDelegationDeliveryV1,
    ManagedRuntimeBlobCustodyDelegationRequestV1, ManagedRuntimeBlobSessionDeliveryV1,
    ManagedRuntimeBlobSessionRequestV1,
};
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::platform::blob::{catalog, launch, status};
use crate::runtime::lifecycle::control::{
    ManagedRuntimeBlobSessionHandler, ManagedRuntimeExpectation,
};
use crate::runtime::lifecycle::fence::current_managed_runtime_matches;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeRelayPort;

const MAX_SESSION_TTL_SECONDS: u32 = 30;
const CUSTODY_SOURCE_PROOF_TTL_MS: u64 = 24 * 60 * 60 * 1_000;
const BLOB_CONTENT_KEY_SCHEMA_REVISION: u64 = 1;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum CustodySourceProofUseV1 {
    Transfer,
    Release,
}

#[derive(Clone, Copy)]
struct CustodyProofTargetV1<'a> {
    owner_id: &'a str,
    module_id: &'a str,
    capability_id: &'a str,
}

#[derive(Clone, Copy)]
struct CustodyProofLineageV1<'a> {
    kind: BlobCustodySourceProofKindV1,
    delegation_id: &'a [u8],
    predecessor_proof_sha256: &'a [u8],
}

impl CustodyProofLineageV1<'static> {
    const ORIGINAL_WRITE: Self = Self {
        kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindOriginalWriteV1,
        delegation_id: &[],
        predecessor_proof_sha256: &[],
    };
}

/// Kernel authority for an exact direct Blob data operation.
pub(crate) struct BlobSessionHandlerV1 {
    store: Arc<SqliteControlStore>,
    relay: ManagedRuntimeRelayPort,
    kernel_data_dir: PathBuf,
    runtime_dir: PathBuf,
}

impl BlobSessionHandlerV1 {
    #[must_use]
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        relay: ManagedRuntimeRelayPort,
        kernel_data_dir: PathBuf,
        runtime_dir: PathBuf,
    ) -> Self {
        Self {
            store,
            relay,
            kernel_data_dir,
            runtime_dir,
        }
    }
}

impl ManagedRuntimeBlobSessionHandler for BlobSessionHandlerV1 {
    fn issue_blob_session(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobSessionRequestV1,
    ) -> Result<ManagedRuntimeBlobSessionDeliveryV1, String> {
        if !current_managed_runtime_matches(
            &*self.store,
            expectation.registration_id(),
            expectation.runtime_instance_id(),
            expectation.runtime_generation(),
            expectation.grant_epoch(),
        )
        .map_err(|_| "managed runtime Blob session request is denied".to_owned())?
        {
            return Err("managed runtime Blob session request is denied".to_owned());
        }
        let entry = catalog::resolve(&*self.store)?
            .into_iter()
            .find(|entry| {
                entry.registration_id() == expectation.registration_id()
                    && entry.capability_id() == request.capability_id
                    && entry.grant_epoch() == expectation.grant_epoch()
            })
            .ok_or_else(|| "managed runtime Blob session request is denied".to_owned())?;
        let operation = i32::try_from(request.operation)
            .ok()
            .and_then(|value| BlobDataOperationV1::try_from(value).ok())
            .filter(|value| {
                matches!(
                    value,
                    BlobDataOperationV1::BlobDataOperationWriteV1
                        | BlobDataOperationV1::BlobDataOperationReadRangeV1
                        | BlobDataOperationV1::BlobDataOperationCustodyTransferV1
                )
            })
            .ok_or_else(|| "managed runtime Blob session request is denied".to_owned())?;
        if entry.request().owner_id().is_empty()
            || !module_blob_operation(operation).is_some_and(|value| entry.request().allows(value))
            || request.declared_size == 0
            || request.declared_size > entry.request().max_bytes()
        {
            return Err("managed runtime Blob session request is denied".to_owned());
        }
        if operation == BlobDataOperationV1::BlobDataOperationCustodyTransferV1 {
            return self.issue_custody_transfer(expectation, request, entry);
        }
        if !request.custody_source_proof.is_empty()
            || !request.evidence_id.is_empty()
            || !request.evidence_envelope_sha256.is_empty()
            || (operation != BlobDataOperationV1::BlobDataOperationWriteV1
                && has_custody_target(&request))
        {
            return Err("managed runtime Blob session request is denied".to_owned());
        }
        let now = now_unix_ms()?;
        let expires_at_unix_ms = now
            .checked_add(u64::from(request.ttl_seconds) * 1_000)
            .ok_or_else(|| "managed runtime Blob session request is denied".to_owned())?;
        let blob = status::read_current(&self.store, &self.relay)?;
        let mut session_id = [0_u8; 16];
        getrandom::fill(&mut session_id)
            .map_err(|_| "managed runtime Blob session request is unavailable".to_owned())?;
        if session_id.iter().all(|byte| *byte == 0) {
            return Err("managed runtime Blob session request is unavailable".to_owned());
        }
        let mut grant = BlobDataSessionGrantV1 {
            major: 1,
            kernel_instance_id: self.store.snapshot().instance_id().to_owned(),
            session_id: session_id.to_vec(),
            channel_binding_sha256: request.channel_binding_sha256,
            owner_id: entry.request().owner_id().to_owned(),
            registration_id: expectation.registration_id().to_owned(),
            capability_id: request.capability_id,
            runtime_instance_id: expectation.runtime_instance_id().to_owned(),
            runtime_generation: expectation.runtime_generation(),
            grant_epoch: expectation.grant_epoch(),
            key_revision: BLOB_CONTENT_KEY_SCHEMA_REVISION,
            quota_max_bytes: entry.request().max_bytes(),
            reference_id: request.reference_id,
            declared_size: request.declared_size,
            reference_expires_at_unix_ms: 0,
            backup_class: i32::try_from(request.backup_class)
                .map_err(|_| "managed runtime Blob session request is denied".to_owned())?,
            operation: operation as i32,
            expires_at_unix_ms,
            kernel_authorization_signature_raw: Vec::new(),
            blob_runtime_generation: blob.runtime_generation(),
            expected_plaintext_sha256: request.receipt_sha256.clone(),
            custody_scope_id: entry.request().custody_scope_id().to_owned(),
        };
        let signer = FileDeviceSigner::open_for_instance(&self.kernel_data_dir)?;
        let mut message = b"makosh.blob-data-session.v1\0".to_vec();
        message.extend_from_slice(&grant.encode_to_vec());
        grant.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
        let custody_transfer_source_proof = if request.receipt_sha256.is_empty()
            || operation != BlobDataOperationV1::BlobDataOperationWriteV1
        {
            Vec::new()
        } else {
            issue_custody_source_proof(
                &signer,
                &grant,
                &request.receipt_sha256,
                CustodyProofTargetV1 {
                    owner_id: &request.custody_target_owner_id,
                    module_id: &request.custody_target_module_id,
                    capability_id: &request.custody_target_capability_id,
                },
                now,
                CustodyProofLineageV1::ORIGINAL_WRITE,
            )?
        };
        Ok(ManagedRuntimeBlobSessionDeliveryV1 {
            data_socket_path: launch::data_socket_path(&self.runtime_dir)
                .display()
                .to_string(),
            grant: Some(grant),
            custody_transfer_source_proof,
            custody_transfer_grant: None,
        })
    }

    fn delegate_blob_custody(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobCustodyDelegationRequestV1,
    ) -> Result<ManagedRuntimeBlobCustodyDelegationDeliveryV1, String> {
        if !valid_delegation_request(&request)
            || !current_managed_runtime_matches(
                &*self.store,
                expectation.registration_id(),
                expectation.runtime_instance_id(),
                expectation.runtime_generation(),
                expectation.grant_epoch(),
            )
            .map_err(|_| custody_delegation_denied())?
        {
            return Err(custody_delegation_denied());
        }
        let source = catalog::resolve(&*self.store)?
            .into_iter()
            .find(|entry| {
                entry.registration_id() == expectation.registration_id()
                    && entry.capability_id() == request.capability_id
                    && entry.grant_epoch() == expectation.grant_epoch()
                    && entry
                        .request()
                        .allows(ModuleBlobOperationV1::CustodyTransfer)
            })
            .ok_or_else(custody_delegation_denied)?;
        let target = resolve_delegation_target(&self.store, expectation, &request)?;
        let now = now_unix_ms()?;
        let signer = FileDeviceSigner::open_for_instance(&self.kernel_data_dir)?;
        let predecessor = verify_custody_source_proof(
            &request.predecessor_custody_source_proof,
            &signer.public_key_sec1(),
            self.store.snapshot().instance_id(),
            now,
            CustodySourceProofUseV1::Release,
        )
        .map_err(|_| custody_delegation_denied())?;
        let expected_reference = transfer_target_reference(
            &predecessor,
            &request.predecessor_evidence_id,
            &request.predecessor_evidence_envelope_sha256,
        );
        if expected_reference.as_slice() != request.current_reference_id.as_slice()
            || !proof_authorizes_target(
                &predecessor,
                source.request().owner_id(),
                expectation.module_id(),
                &request.capability_id,
            )
        {
            return Err(custody_delegation_denied());
        }
        let grant = BlobDataSessionGrantV1 {
            major: 1,
            kernel_instance_id: self.store.snapshot().instance_id().to_owned(),
            session_id: Vec::new(),
            channel_binding_sha256: Vec::new(),
            owner_id: source.request().owner_id().to_owned(),
            registration_id: expectation.registration_id().to_owned(),
            capability_id: request.capability_id,
            runtime_instance_id: expectation.runtime_instance_id().to_owned(),
            runtime_generation: expectation.runtime_generation(),
            grant_epoch: expectation.grant_epoch(),
            key_revision: BLOB_CONTENT_KEY_SCHEMA_REVISION,
            quota_max_bytes: source.request().max_bytes(),
            reference_id: request.current_reference_id,
            declared_size: predecessor.declared_size,
            reference_expires_at_unix_ms: predecessor.reference_expires_at_unix_ms,
            backup_class: predecessor.backup_class,
            operation: BlobDataOperationV1::BlobDataOperationCustodyTransferV1 as i32,
            expires_at_unix_ms: 0,
            kernel_authorization_signature_raw: Vec::new(),
            blob_runtime_generation: 0,
            expected_plaintext_sha256: predecessor.receipt_sha256.clone(),
            custody_scope_id: source.request().custody_scope_id().to_owned(),
        };
        let predecessor_proof_sha256 = Sha256::digest(&request.predecessor_custody_source_proof);
        let proof = issue_custody_source_proof(
            &signer,
            &grant,
            &predecessor.receipt_sha256,
            CustodyProofTargetV1 {
                owner_id: &target.owner_id,
                module_id: &target.module_id,
                capability_id: &target.capability_id,
            },
            now,
            CustodyProofLineageV1 {
                kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1,
                delegation_id: &request.request_id,
                predecessor_proof_sha256: predecessor_proof_sha256.as_slice(),
            },
        )?;
        Ok(ManagedRuntimeBlobCustodyDelegationDeliveryV1 {
            request_id: request.request_id,
            custody_transfer_source_proof: proof,
            resolved_target_owner_id: target.owner_id,
            resolved_target_module_id: target.module_id,
            resolved_target_capability_id: target.capability_id,
        })
    }
}

impl BlobSessionHandlerV1 {
    fn issue_custody_transfer(
        &self,
        expectation: &ManagedRuntimeExpectation,
        request: ManagedRuntimeBlobSessionRequestV1,
        target: crate::platform::blob::catalog::BlobQuotaCatalogEntryV1,
    ) -> Result<ManagedRuntimeBlobSessionDeliveryV1, String> {
        if request.receipt_sha256.len() != 32
            || request.reference_id.len() != 16
            || request.evidence_id.len() != 16
            || request.evidence_envelope_sha256.len() != 32
            || request.custody_source_proof.is_empty()
            || request.custody_source_proof.len() > 2_048
            || request.declared_size == 0
            || !matches!(request.backup_class, 1..=3)
        {
            return Err("managed runtime Blob custody transfer is denied".to_owned());
        }
        let now = now_unix_ms()?;
        let signer = FileDeviceSigner::open_for_instance(&self.kernel_data_dir)?;
        let source = verify_custody_source_proof(
            &request.custody_source_proof,
            &signer.public_key_sec1(),
            self.store.snapshot().instance_id(),
            now,
            CustodySourceProofUseV1::Transfer,
        )
        .inspect_err(|_| {
            tracing::warn!(
                event = "blob.custody_transfer.denied",
                denial.stage = "source_proof",
            );
        })?;
        let source_matches = source.reference_id == request.reference_id
            && source.declared_size == request.declared_size
            && source.receipt_sha256 == request.receipt_sha256;
        let target_authorized = proof_authorizes_target(
            &source,
            target.request().owner_id(),
            expectation.module_id(),
            &request.capability_id,
        );
        let source_grant_current = catalog::resolve(&*self.store)?.iter().any(|entry| {
            entry.registration_id() == source.registration_id.as_str()
                && entry.capability_id() == source.capability_id.as_str()
                && entry.grant_epoch() == source.grant_epoch
                && entry.request().owner_id() == source.owner_id.as_str()
                && entry.request().custody_scope_id() == source.custody_scope_id
                && required_source_operation(&source)
                    .is_some_and(|operation| entry.request().allows(operation))
        });
        if !source_matches || !target_authorized || !source_grant_current {
            tracing::warn!(
                event = "blob.custody_transfer.denied",
                denial.stage = "binding",
                proof.source_matches = source_matches,
                proof.target_authorized = target_authorized,
                proof.source_grant_current = source_grant_current,
            );
            return Err("managed runtime Blob custody transfer is denied".to_owned());
        }
        let blob = status::read_current(&self.store, &self.relay)?;
        let mut session_id = [0_u8; 16];
        getrandom::fill(&mut session_id)
            .map_err(|_| "managed runtime Blob custody transfer is unavailable".to_owned())?;
        let target_reference_id = transfer_target_reference(
            &source,
            &request.evidence_id,
            &request.evidence_envelope_sha256,
        );
        let expires_at_unix_ms = now
            .checked_add(u64::from(request.ttl_seconds) * 1_000)
            .ok_or_else(|| "managed runtime Blob custody transfer is unavailable".to_owned())?;
        let mut grant = BlobCustodyTransferGrantV1 {
            major: 1,
            kernel_instance_id: self.store.snapshot().instance_id().to_owned(),
            session_id: session_id.to_vec(),
            channel_binding_sha256: request.channel_binding_sha256,
            evidence_id: request.evidence_id,
            evidence_envelope_sha256: request.evidence_envelope_sha256,
            source: Some(source),
            target_owner_id: target.request().owner_id().to_owned(),
            target_registration_id: expectation.registration_id().to_owned(),
            target_capability_id: request.capability_id,
            target_runtime_instance_id: expectation.runtime_instance_id().to_owned(),
            target_runtime_generation: expectation.runtime_generation(),
            target_grant_epoch: expectation.grant_epoch(),
            target_key_revision: BLOB_CONTENT_KEY_SCHEMA_REVISION,
            target_quota_max_bytes: target.request().max_bytes(),
            target_reference_id: target_reference_id.to_vec(),
            expires_at_unix_ms,
            blob_runtime_generation: blob.runtime_generation(),
            kernel_authorization_signature_raw: Vec::new(),
            target_custody_scope_id: target.request().custody_scope_id().to_owned(),
        };
        let mut message = b"makosh.blob-custody-transfer.v1\0".to_vec();
        message.extend_from_slice(&grant.encode_to_vec());
        grant.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
        Ok(ManagedRuntimeBlobSessionDeliveryV1 {
            data_socket_path: launch::data_socket_path(&self.runtime_dir)
                .display()
                .to_string(),
            grant: None,
            custody_transfer_source_proof: Vec::new(),
            custody_transfer_grant: Some(grant),
        })
    }
}

pub(crate) fn valid_request(request: &ManagedRuntimeBlobSessionRequestV1) -> bool {
    request.request_id.len() == 16
        && request.request_id.iter().any(|byte| *byte != 0)
        && !request.capability_id.is_empty()
        && request.capability_id.len() <= 128
        && request.channel_binding_sha256.len() == 32
        && request.reference_id.len() == 16
        && request.reference_id.iter().any(|byte| *byte != 0)
        && request.declared_size > 0
        && (1..=3).contains(&request.backup_class)
        && (1..=MAX_SESSION_TTL_SECONDS).contains(&request.ttl_seconds)
        && (request.receipt_sha256.is_empty()
            || (request.receipt_sha256.len() == 32
                && request.receipt_sha256.iter().any(|byte| *byte != 0)))
        && valid_custody_target_request(request)
}

pub(crate) fn valid_delegation_request(
    request: &ManagedRuntimeBlobCustodyDelegationRequestV1,
) -> bool {
    request.request_id.len() == 16
        && request.request_id.iter().any(|byte| *byte != 0)
        && valid_target_token(&request.capability_id)
        && request.current_reference_id.len() == 16
        && request.current_reference_id.iter().any(|byte| *byte != 0)
        && !request.predecessor_custody_source_proof.is_empty()
        && request.predecessor_custody_source_proof.len() <= 2_048
        && request.predecessor_evidence_id.len() == 16
        && request
            .predecessor_evidence_id
            .iter()
            .any(|byte| *byte != 0)
        && request.predecessor_evidence_envelope_sha256.len() == 32
        && request
            .predecessor_evidence_envelope_sha256
            .iter()
            .any(|byte| *byte != 0)
        && valid_delegation_target(request)
}

struct ResolvedCustodyTargetV1 {
    owner_id: String,
    module_id: String,
    capability_id: String,
}

fn valid_delegation_target(request: &ManagedRuntimeBlobCustodyDelegationRequestV1) -> bool {
    let has_explicit = valid_target_token(&request.target_owner_id)
        && valid_target_token(&request.target_module_id)
        && valid_target_token(&request.target_capability_id);
    let explicit_empty = request.target_owner_id.is_empty()
        && request.target_module_id.is_empty()
        && request.target_capability_id.is_empty();
    let has_contract = request
        .target_request_contract
        .as_ref()
        .is_some_and(valid_request_contract);
    (has_explicit && request.target_request_contract.is_none()) || (explicit_empty && has_contract)
}

fn valid_request_contract(contract: &makosh_runtime_protocol::v1::ContractReferenceV1) -> bool {
    valid_target_token(&contract.owner)
        && valid_target_token(&contract.name)
        && contract.major > 0
        && contract.revision > 0
        && contract.schema_sha256.len() == 32
        && contract.schema_sha256.iter().any(|byte| *byte != 0)
}

fn resolve_delegation_target(
    store: &makosh_kernel_control_store_sqlite::SqliteControlStore,
    expectation: &ManagedRuntimeExpectation,
    request: &ManagedRuntimeBlobCustodyDelegationRequestV1,
) -> Result<ResolvedCustodyTargetV1, String> {
    if let Some(contract) = request.target_request_contract.as_ref() {
        let provider = crate::modules::capability::module_request::resolve_provider_for_caller(
            store,
            expectation,
            contract,
        )?;
        let mut targets = catalog::resolve(store)?.into_iter().filter(|entry| {
            entry.registration_id() == provider.registration.registration_id()
                && entry.module_id() == provider.registration.module_id()
                && entry.grant_epoch() == provider.registration.grant_epoch()
                && entry.request().allows(ModuleBlobOperationV1::ReadRange)
        });
        let target = targets
            .next()
            .ok_or_else(|| "managed runtime Blob provider target is unavailable".to_owned())?;
        if targets.next().is_some() {
            return Err("managed runtime Blob provider target is ambiguous".to_owned());
        }
        return Ok(ResolvedCustodyTargetV1 {
            owner_id: target.request().owner_id().to_owned(),
            module_id: target.module_id().to_owned(),
            capability_id: target.capability_id().to_owned(),
        });
    }
    Ok(ResolvedCustodyTargetV1 {
        owner_id: request.target_owner_id.clone(),
        module_id: request.target_module_id.clone(),
        capability_id: request.target_capability_id.clone(),
    })
}

pub(super) fn verify_custody_source_proof(
    encoded: &[u8],
    public_key_sec1: &[u8; 65],
    kernel_instance_id: &str,
    now_unix_ms: u64,
    proof_use: CustodySourceProofUseV1,
) -> Result<BlobCustodySourceProofV1, String> {
    let proof = BlobCustodySourceProofV1::decode(encoded)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    if proof.major != 1
        || proof.kernel_instance_id != kernel_instance_id
        || proof.owner_id.is_empty()
        || proof.registration_id.is_empty()
        || proof.capability_id.is_empty()
        || proof.runtime_instance_id.is_empty()
        || proof.runtime_generation == 0
        || proof.grant_epoch == 0
        || proof.key_revision == 0
        || proof.custody_scope_id.is_empty()
        || proof.reference_id.len() != 16
        || proof.reference_id.iter().all(|byte| *byte == 0)
        || proof.declared_size == 0
        || proof.receipt_sha256.len() != 32
        || proof.receipt_sha256.iter().all(|byte| *byte == 0)
        || proof.issued_at_unix_ms == 0
        || proof.expires_at_unix_ms <= proof.issued_at_unix_ms
        || (proof_use == CustodySourceProofUseV1::Transfer
            && proof.expires_at_unix_ms <= now_unix_ms)
        || proof.issued_at_unix_ms > now_unix_ms
        || proof.kernel_authorization_signature_raw.len() != 64
        || !valid_proof_target(&proof)
        || !valid_proof_lineage(&proof)
    {
        return Err("managed runtime Blob custody transfer is denied".to_owned());
    }
    let signature = Signature::from_slice(&proof.kernel_authorization_signature_raw)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    let key = VerifyingKey::from_sec1_bytes(public_key_sec1)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    let mut unsigned = proof.clone();
    unsigned.kernel_authorization_signature_raw.clear();
    let mut message = b"makosh.blob-custody-source-proof.v1\0".to_vec();
    message.extend_from_slice(&unsigned.encode_to_vec());
    key.verify(&message, &signature)
        .map_err(|_| "managed runtime Blob custody transfer is denied".to_owned())?;
    Ok(proof)
}

fn transfer_target_reference(
    source: &BlobCustodySourceProofV1,
    evidence_id: &[u8],
    envelope_hash: &[u8],
) -> [u8; 16] {
    let mut digest = Sha256::new();
    if source.proof_kind
        == BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1
            as i32
    {
        digest.update(b"makosh.blob-custody-target-reference.v2\0");
        digest.update(&source.delegation_id);
        digest.update(&source.reference_id);
        digest.update(source.target_owner_id.as_bytes());
        digest.update([0]);
        digest.update(source.target_module_id.as_bytes());
        digest.update([0]);
        digest.update(source.target_capability_id.as_bytes());
        digest.update([0]);
    } else {
        digest.update(b"makosh.blob-custody-target-reference.v3\0");
        update_semantic_reference_field(&mut digest, &source.proof_kind.to_be_bytes());
        update_semantic_reference_field(&mut digest, source.kernel_instance_id.as_bytes());
        update_semantic_reference_field(&mut digest, source.owner_id.as_bytes());
        update_semantic_reference_field(&mut digest, source.registration_id.as_bytes());
        update_semantic_reference_field(&mut digest, source.capability_id.as_bytes());
        update_semantic_reference_field(&mut digest, &source.reference_id);
        update_semantic_reference_field(&mut digest, &source.declared_size.to_be_bytes());
        update_semantic_reference_field(&mut digest, &source.receipt_sha256);
        update_semantic_reference_field(&mut digest, &source.key_revision.to_be_bytes());
        update_semantic_reference_field(&mut digest, &source.backup_class.to_be_bytes());
        update_semantic_reference_field(&mut digest, source.custody_scope_id.as_bytes());
        update_semantic_reference_field(&mut digest, source.target_owner_id.as_bytes());
        update_semantic_reference_field(&mut digest, source.target_module_id.as_bytes());
        update_semantic_reference_field(&mut digest, source.target_capability_id.as_bytes());
    }
    digest.update(evidence_id);
    digest.update(envelope_hash);
    let hash: [u8; 32] = digest.finalize().into();
    let mut reference_id = [0_u8; 16];
    reference_id.copy_from_slice(&hash[..16]);
    if reference_id.iter().all(|byte| *byte == 0) {
        reference_id[0] = 1;
    }
    reference_id
}

fn update_semantic_reference_field(digest: &mut Sha256, value: &[u8]) {
    digest.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    digest.update(value);
}

fn issue_custody_source_proof(
    signer: &FileDeviceSigner,
    grant: &BlobDataSessionGrantV1,
    receipt_sha256: &[u8],
    target: CustodyProofTargetV1<'_>,
    now_unix_ms: u64,
    lineage: CustodyProofLineageV1<'_>,
) -> Result<Vec<u8>, String> {
    let expires_at_unix_ms = now_unix_ms
        .checked_add(CUSTODY_SOURCE_PROOF_TTL_MS)
        .ok_or_else(|| "managed runtime Blob session request is unavailable".to_owned())?;
    let mut proof = BlobCustodySourceProofV1 {
        major: 1,
        kernel_instance_id: grant.kernel_instance_id.clone(),
        owner_id: grant.owner_id.clone(),
        registration_id: grant.registration_id.clone(),
        capability_id: grant.capability_id.clone(),
        runtime_instance_id: grant.runtime_instance_id.clone(),
        runtime_generation: grant.runtime_generation,
        grant_epoch: grant.grant_epoch,
        key_revision: grant.key_revision,
        reference_id: grant.reference_id.clone(),
        declared_size: grant.declared_size,
        receipt_sha256: receipt_sha256.to_vec(),
        issued_at_unix_ms: now_unix_ms,
        expires_at_unix_ms,
        kernel_authorization_signature_raw: Vec::new(),
        backup_class: grant.backup_class,
        reference_expires_at_unix_ms: grant.reference_expires_at_unix_ms,
        target_owner_id: target.owner_id.to_owned(),
        target_module_id: target.module_id.to_owned(),
        target_capability_id: target.capability_id.to_owned(),
        custody_scope_id: grant.custody_scope_id.clone(),
        proof_kind: lineage.kind as i32,
        delegation_id: lineage.delegation_id.to_vec(),
        predecessor_proof_sha256: lineage.predecessor_proof_sha256.to_vec(),
    };
    let mut message = b"makosh.blob-custody-source-proof.v1\0".to_vec();
    message.extend_from_slice(&proof.encode_to_vec());
    proof.kernel_authorization_signature_raw = signer.sign(&message).to_vec();
    Ok(proof.encode_to_vec())
}

fn valid_proof_lineage(proof: &BlobCustodySourceProofV1) -> bool {
    match BlobCustodySourceProofKindV1::try_from(proof.proof_kind).ok() {
        None => false,
        Some(
            BlobCustodySourceProofKindV1::BlobCustodySourceProofKindUnspecifiedV1
            | BlobCustodySourceProofKindV1::BlobCustodySourceProofKindOriginalWriteV1,
        ) => proof.delegation_id.is_empty() && proof.predecessor_proof_sha256.is_empty(),
        Some(
            BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1,
        ) => {
            proof.delegation_id.len() == 16
                && proof.delegation_id.iter().any(|byte| *byte != 0)
                && proof.predecessor_proof_sha256.len() == 32
                && proof.predecessor_proof_sha256.iter().any(|byte| *byte != 0)
        }
    }
}

fn required_source_operation(proof: &BlobCustodySourceProofV1) -> Option<ModuleBlobOperationV1> {
    match BlobCustodySourceProofKindV1::try_from(proof.proof_kind).ok()? {
        BlobCustodySourceProofKindV1::BlobCustodySourceProofKindUnspecifiedV1
        | BlobCustodySourceProofKindV1::BlobCustodySourceProofKindOriginalWriteV1 => {
            Some(ModuleBlobOperationV1::Write)
        }
        BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1 => {
            Some(ModuleBlobOperationV1::CustodyTransfer)
        }
    }
}

fn custody_delegation_denied() -> String {
    "managed runtime Blob custody delegation is denied".to_owned()
}

const fn module_blob_operation(operation: BlobDataOperationV1) -> Option<ModuleBlobOperationV1> {
    match operation {
        BlobDataOperationV1::BlobDataOperationWriteV1 => Some(ModuleBlobOperationV1::Write),
        BlobDataOperationV1::BlobDataOperationReadRangeV1 => Some(ModuleBlobOperationV1::ReadRange),
        BlobDataOperationV1::BlobDataOperationCustodyTransferV1 => {
            Some(ModuleBlobOperationV1::CustodyTransfer)
        }
        BlobDataOperationV1::BlobDataOperationUnspecifiedV1 => None,
    }
}

fn valid_custody_target_request(request: &ManagedRuntimeBlobSessionRequestV1) -> bool {
    let populated = [
        request.custody_target_owner_id.as_str(),
        request.custody_target_module_id.as_str(),
        request.custody_target_capability_id.as_str(),
    ];
    populated.iter().all(|value| value.is_empty())
        || (request.operation == BlobDataOperationV1::BlobDataOperationWriteV1 as u32
            && !request.receipt_sha256.is_empty()
            && populated.iter().all(|value| valid_target_token(value)))
}

fn has_custody_target(request: &ManagedRuntimeBlobSessionRequestV1) -> bool {
    !request.custody_target_owner_id.is_empty()
        || !request.custody_target_module_id.is_empty()
        || !request.custody_target_capability_id.is_empty()
}

fn has_proof_target(proof: &BlobCustodySourceProofV1) -> bool {
    !proof.target_owner_id.is_empty()
        || !proof.target_module_id.is_empty()
        || !proof.target_capability_id.is_empty()
}

fn valid_proof_target(proof: &BlobCustodySourceProofV1) -> bool {
    let populated = [
        proof.target_owner_id.as_str(),
        proof.target_module_id.as_str(),
        proof.target_capability_id.as_str(),
    ];
    populated.iter().all(|value| value.is_empty())
        || populated.iter().all(|value| valid_target_token(value))
}

pub(super) fn proof_authorizes_target(
    proof: &BlobCustodySourceProofV1,
    target_owner_id: &str,
    target_module_id: &str,
    target_capability_id: &str,
) -> bool {
    if has_proof_target(proof) {
        proof.target_owner_id == target_owner_id
            && proof.target_module_id == target_module_id
            && proof.target_capability_id == target_capability_id
    } else {
        proof.owner_id == target_owner_id
    }
}

fn valid_target_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn now_unix_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "managed runtime Blob session request is unavailable".to_owned())?
        .as_millis()
        .try_into()
        .map_err(|_| "managed runtime Blob session request is unavailable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let mut nonce = [0_u8; 16];
            getrandom::fill(&mut nonce).expect("temporary directory nonce");
            let name = nonce
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let path = std::env::temp_dir().join(format!("makosh-blob-delegation-{name}"));
            std::fs::create_dir(&path).expect("temporary data directory");
            Self(path)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn unbound_proof_remains_same_owner_only() {
        let proof = BlobCustodySourceProofV1 {
            owner_id: "mail".to_owned(),
            ..Default::default()
        };

        assert!(proof_authorizes_target(
            &proof,
            "mail",
            "mail-secondary",
            "mail.blob.v1",
        ));
        assert!(!proof_authorizes_target(
            &proof,
            "communications",
            "communications",
            "communications.blob.v1",
        ));
    }

    #[test]
    fn bound_proof_requires_the_exact_cross_owner_target() {
        let proof = BlobCustodySourceProofV1 {
            owner_id: "mail".to_owned(),
            target_owner_id: "attachment_security".to_owned(),
            target_module_id: "makosh-attachment-security-runtime".to_owned(),
            target_capability_id: "attachment_security.blob.v1".to_owned(),
            ..Default::default()
        };

        assert!(valid_proof_target(&proof));
        assert!(proof_authorizes_target(
            &proof,
            "attachment_security",
            "makosh-attachment-security-runtime",
            "attachment_security.blob.v1",
        ));
        assert!(!proof_authorizes_target(
            &proof,
            "communications",
            "communications",
            "communications.blob.v1",
        ));

        let partial = BlobCustodySourceProofV1 {
            target_owner_id: "attachment_security".to_owned(),
            ..Default::default()
        };
        assert!(!valid_proof_target(&partial));
    }

    #[test]
    fn custody_delegation_request_is_exact_and_bounded() {
        let mut request = ManagedRuntimeBlobCustodyDelegationRequestV1 {
            request_id: vec![1; 16],
            capability_id: "attachment_security.blob.v1".to_owned(),
            current_reference_id: vec![2; 16],
            predecessor_custody_source_proof: vec![3; 128],
            predecessor_evidence_id: vec![4; 16],
            predecessor_evidence_envelope_sha256: vec![5; 32],
            target_owner_id: "attachment_archive_inspection".to_owned(),
            target_module_id: "makosh-attachment-archive-inspection-runtime".to_owned(),
            target_capability_id: "attachment_archive_inspection.blob.v1".to_owned(),
            target_request_contract: None,
        };
        assert!(valid_delegation_request(&request));

        request.target_module_id.clear();
        assert!(!valid_delegation_request(&request));
        request.target_module_id = "makosh-attachment-archive-inspection-runtime".to_owned();
        request.predecessor_evidence_envelope_sha256 = vec![0; 32];
        assert!(!valid_delegation_request(&request));
    }

    #[test]
    fn custody_delegation_accepts_exactly_one_explicit_or_resolved_provider_target() {
        let contract = makosh_runtime_protocol::v1::ContractReferenceV1 {
            owner: "speech_to_text".to_owned(),
            name: "speech_to_text_provider_transcribe".to_owned(),
            major: 1,
            revision: 1,
            schema_sha256: vec![7; 32],
        };
        let mut request = ManagedRuntimeBlobCustodyDelegationRequestV1 {
            request_id: vec![1; 16],
            capability_id: "speech_to_text.blob.v1".to_owned(),
            current_reference_id: vec![2; 16],
            predecessor_custody_source_proof: vec![3; 128],
            predecessor_evidence_id: vec![4; 16],
            predecessor_evidence_envelope_sha256: vec![5; 32],
            target_owner_id: String::new(),
            target_module_id: String::new(),
            target_capability_id: String::new(),
            target_request_contract: Some(contract),
        };
        assert!(valid_delegation_request(&request));

        request.target_owner_id = "whisper_stt".to_owned();
        request.target_module_id = "makosh-whisper-stt-runtime".to_owned();
        request.target_capability_id = "speech_to_text.provider.v1".to_owned();
        assert!(!valid_delegation_request(&request));
    }

    #[test]
    fn proof_kind_selects_the_exact_source_capability_operation() {
        let original = BlobCustodySourceProofV1 {
            proof_kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindOriginalWriteV1
                as i32,
            ..Default::default()
        };
        assert_eq!(
            required_source_operation(&original),
            Some(ModuleBlobOperationV1::Write)
        );
        assert!(valid_proof_lineage(&original));

        let redelegated = BlobCustodySourceProofV1 {
            proof_kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1 as i32,
            delegation_id: vec![7; 16],
            predecessor_proof_sha256: vec![8; 32],
            ..Default::default()
        };
        assert_eq!(
            required_source_operation(&redelegated),
            Some(ModuleBlobOperationV1::CustodyTransfer)
        );
        assert!(valid_proof_lineage(&redelegated));

        let partial = BlobCustodySourceProofV1 {
            proof_kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1 as i32,
            delegation_id: vec![7; 16],
            ..Default::default()
        };
        assert!(!valid_proof_lineage(&partial));
    }

    #[test]
    fn redelegation_retry_keeps_one_target_reference() {
        let mut first = BlobCustodySourceProofV1 {
            proof_kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1 as i32,
            delegation_id: vec![9; 16],
            predecessor_proof_sha256: vec![10; 32],
            reference_id: vec![11; 16],
            target_owner_id: "attachment_archive_inspection".to_owned(),
            target_module_id: "makosh-attachment-archive-inspection-runtime".to_owned(),
            target_capability_id: "attachment_archive_inspection.blob.v1".to_owned(),
            issued_at_unix_ms: 100,
            kernel_authorization_signature_raw: vec![12; 64],
            ..Default::default()
        };
        let first_bytes = first.encode_to_vec();
        let original = first.clone();
        first.issued_at_unix_ms = 200;
        first.kernel_authorization_signature_raw = vec![13; 64];
        let second_bytes = first.encode_to_vec();
        assert_ne!(first_bytes, second_bytes);

        let evidence_id = [14; 16];
        let envelope_sha256 = [15; 32];
        assert_eq!(
            transfer_target_reference(&original, &evidence_id, &envelope_sha256),
            transfer_target_reference(&first, &evidence_id, &envelope_sha256),
        );
    }

    #[test]
    fn refreshed_original_write_proof_keeps_one_semantic_target_reference() {
        let mut source = BlobCustodySourceProofV1 {
            proof_kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindOriginalWriteV1
                as i32,
            kernel_instance_id: "kernel-1".to_owned(),
            owner_id: "mail".to_owned(),
            registration_id: "mail-registration".to_owned(),
            capability_id: "mail.blob.v1".to_owned(),
            runtime_instance_id: "mail-runtime-1".to_owned(),
            runtime_generation: 1,
            grant_epoch: 2,
            key_revision: 1,
            reference_id: vec![1; 16],
            declared_size: 42,
            receipt_sha256: vec![2; 32],
            issued_at_unix_ms: 100,
            expires_at_unix_ms: 200,
            kernel_authorization_signature_raw: vec![3; 64],
            backup_class: 1,
            target_owner_id: "speech_to_text".to_owned(),
            target_module_id: "makosh-speech-to-text-runtime".to_owned(),
            target_capability_id: "speech_to_text.blob.v1".to_owned(),
            custody_scope_id: "mail.private_content.v1".to_owned(),
            ..Default::default()
        };
        let first = source.encode_to_vec();
        let original = source.clone();
        source.runtime_instance_id = "mail-runtime-2".to_owned();
        source.runtime_generation = 2;
        source.grant_epoch = 3;
        source.issued_at_unix_ms = 300;
        source.expires_at_unix_ms = 400;
        source.kernel_authorization_signature_raw = vec![4; 64];
        let refreshed = source.encode_to_vec();
        assert_ne!(first, refreshed);
        assert_eq!(
            transfer_target_reference(&original, &[5; 16], &[6; 32]),
            transfer_target_reference(&source, &[5; 16], &[6; 32]),
        );
        let stable = transfer_target_reference(&source, &[5; 16], &[6; 32]);
        source.receipt_sha256[0] ^= 1;
        assert_ne!(
            stable,
            transfer_target_reference(&source, &[5; 16], &[6; 32])
        );
    }

    #[test]
    fn signed_predecessor_lineage_mints_only_an_exact_redelegation() {
        let directory = TestDirectory::new();
        let (signer, _) =
            FileDeviceSigner::open_or_create_for_instance(directory.path()).expect("signer");
        let predecessor_grant = BlobDataSessionGrantV1 {
            major: 1,
            kernel_instance_id: "kernel-1".to_owned(),
            owner_id: "mail".to_owned(),
            registration_id: "mail-registration".to_owned(),
            capability_id: "mail.blob.v1".to_owned(),
            runtime_instance_id: "mail-runtime".to_owned(),
            runtime_generation: 3,
            grant_epoch: 4,
            key_revision: 1,
            reference_id: vec![1; 16],
            declared_size: 42,
            backup_class: 1,
            custody_scope_id: "mail-blob-custody".to_owned(),
            ..Default::default()
        };
        let predecessor_bytes = issue_custody_source_proof(
            &signer,
            &predecessor_grant,
            &[2; 32],
            CustodyProofTargetV1 {
                owner_id: "attachment_security",
                module_id: "makosh-attachment-security-runtime",
                capability_id: "attachment_security.blob.v1",
            },
            100,
            CustodyProofLineageV1::ORIGINAL_WRITE,
        )
        .expect("predecessor proof");
        let predecessor = verify_custody_source_proof(
            &predecessor_bytes,
            &signer.public_key_sec1(),
            "kernel-1",
            101,
            CustodySourceProofUseV1::Release,
        )
        .expect("verified predecessor");
        let evidence_id = [3; 16];
        let evidence_sha256 = [4; 32];
        let current_reference =
            transfer_target_reference(&predecessor, &evidence_id, &evidence_sha256);
        let current_grant = BlobDataSessionGrantV1 {
            major: 1,
            kernel_instance_id: "kernel-1".to_owned(),
            owner_id: "attachment_security".to_owned(),
            registration_id: "security-registration".to_owned(),
            capability_id: "attachment_security.blob.v1".to_owned(),
            runtime_instance_id: "security-runtime".to_owned(),
            runtime_generation: 5,
            grant_epoch: 6,
            key_revision: 1,
            reference_id: current_reference.to_vec(),
            declared_size: predecessor.declared_size,
            backup_class: predecessor.backup_class,
            custody_scope_id: "attachment-security-custody".to_owned(),
            ..Default::default()
        };
        let predecessor_sha256 = Sha256::digest(&predecessor_bytes);
        let redelegated_bytes = issue_custody_source_proof(
            &signer,
            &current_grant,
            &predecessor.receipt_sha256,
            CustodyProofTargetV1 {
                owner_id: "attachment_archive_inspection",
                module_id: "makosh-attachment-archive-inspection-runtime",
                capability_id: "attachment_archive_inspection.blob.v1",
            },
            200,
            CustodyProofLineageV1 {
                kind: BlobCustodySourceProofKindV1::BlobCustodySourceProofKindCurrentCustodianRedelegationV1,
                delegation_id: &[5; 16],
                predecessor_proof_sha256: predecessor_sha256.as_slice(),
            },
        )
        .expect("redelegated proof");
        let redelegated = verify_custody_source_proof(
            &redelegated_bytes,
            &signer.public_key_sec1(),
            "kernel-1",
            201,
            CustodySourceProofUseV1::Transfer,
        )
        .expect("verified redelegation");
        assert_eq!(redelegated.reference_id, current_reference);
        assert_eq!(
            required_source_operation(&redelegated),
            Some(ModuleBlobOperationV1::CustodyTransfer)
        );
        assert!(proof_authorizes_target(
            &redelegated,
            "attachment_archive_inspection",
            "makosh-attachment-archive-inspection-runtime",
            "attachment_archive_inspection.blob.v1",
        ));

        let mut altered = redelegated;
        altered.target_owner_id = "communications".to_owned();
        assert!(
            verify_custody_source_proof(
                &altered.encode_to_vec(),
                &signer.public_key_sec1(),
                "kernel-1",
                201,
                CustodySourceProofUseV1::Transfer,
            )
            .is_err()
        );
    }
}

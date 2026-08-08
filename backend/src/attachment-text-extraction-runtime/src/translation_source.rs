//! Event-only source producer for the bounded Attachment Translation workflow.

use std::os::unix::net::UnixStream;

use makosh_attachment_text_extraction_api::ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1;
use makosh_attachment_text_extraction_persistence::{
    AttachmentTextExtractionPersistenceErrorV1, AttachmentTextExtractionPersistenceV1,
    PersistTranslationSourceResultV1, TranslationSourceSnapshotOutcomeV1,
    TranslationSourceSnapshotV1,
};
use makosh_attachment_translation_ingress::{
    ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1,
    ATTACHMENT_TRANSLATION_BLOB_TARGET_MODULE_ID_V1, AttachmentTranslationSourceEnvelopeContextV1,
    attachment_translation_source_request_id_v1,
    attachment_translation_source_requested_contract_reference_v1,
    build_attachment_translation_source_prepared_outbox_record_v1,
    build_attachment_translation_source_rejected_outbox_record_v1,
    wire::{
        AttachmentTranslationSourcePreparedV1, AttachmentTranslationSourceRejectCodeV1,
        AttachmentTranslationSourceRejectedV1, RequestAttachmentTranslationSourceV1,
    },
};
use makosh_events_protocol::{
    delivery::OutboxRecordV1,
    v1::{ContractRefV1, durable_envelope_v1::Semantics},
    validation::envelope::decode_envelope_v1,
};
use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
use prost::Message;

use crate::blob::{BlobErrorV1, read_artifact_v1, write_translation_source_v1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TranslationSourceDeliveryErrorV1 {
    InvalidEnvelope,
    InvalidPayload,
    Persistence,
    Unavailable,
}

pub(crate) async fn process_translation_source_delivery_v1(
    persistence: &AttachmentTextExtractionPersistenceV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    exact_envelope_bytes: &[u8],
    logical_owner_id: &str,
    runtime_instance_id: &str,
    runtime_generation: u64,
    processed_at_unix_millis: i64,
) -> Result<(), TranslationSourceDeliveryErrorV1> {
    let record = OutboxRecordV1::accept(exact_envelope_bytes.to_vec())
        .map_err(|_| TranslationSourceDeliveryErrorV1::InvalidEnvelope)?;
    let request = decode_request(&record, logical_owner_id)?;
    if persistence
        .translation_source_request_already_processed(
            logical_owner_id,
            *record.message_id(),
            *record.envelope_sha256(),
            request.request_id,
            request.translation_run_id,
            request.source_extraction_run_id,
            request.expected_source_revision,
        )
        .await
        .map_err(persistence_error)?
    {
        return Ok(());
    }
    let context = AttachmentTranslationSourceEnvelopeContextV1 {
        module_id: ATTACHMENT_TEXT_EXTRACTION_MODULE_ID_V1.to_owned(),
        runtime_instance_id: runtime_instance_id.to_owned(),
        runtime_generation,
        recorded_at_unix_seconds: processed_at_unix_millis / 1_000,
        recorded_at_nanos: i32::try_from((processed_at_unix_millis % 1_000) * 1_000_000)
            .map_err(|_| TranslationSourceDeliveryErrorV1::InvalidPayload)?,
    };
    let snapshot = persistence
        .translation_source_snapshot(
            logical_owner_id,
            request.source_extraction_run_id,
            request.expected_source_revision,
        )
        .await
        .map_err(persistence_error)?;
    match snapshot {
        TranslationSourceSnapshotOutcomeV1::NotReady => {
            persist_rejection(
                persistence,
                &record,
                &request,
                AttachmentTranslationSourceRejectCodeV1::NotReady,
                &context,
                processed_at_unix_millis,
            )
            .await
        }
        TranslationSourceSnapshotOutcomeV1::StaleRevision => {
            persist_rejection(
                persistence,
                &record,
                &request,
                AttachmentTranslationSourceRejectCodeV1::StaleRevision,
                &context,
                processed_at_unix_millis,
            )
            .await
        }
        TranslationSourceSnapshotOutcomeV1::Ready(snapshot) => {
            prepare_source(
                persistence,
                control_channel,
                &record,
                &request,
                snapshot,
                &context,
                processed_at_unix_millis,
            )
            .await
        }
    }
}

async fn prepare_source(
    persistence: &AttachmentTextExtractionPersistenceV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    record: &OutboxRecordV1,
    request: &DecodedTranslationSourceRequestV1,
    snapshot: TranslationSourceSnapshotV1,
    context: &AttachmentTranslationSourceEnvelopeContextV1,
    processed_at_unix_millis: i64,
) -> Result<(), TranslationSourceDeliveryErrorV1> {
    let bytes = match read_artifact_v1(control_channel, &snapshot.artifact) {
        Ok(bytes) => bytes,
        Err(BlobErrorV1::InvalidReceipt) => {
            return persist_rejection(
                persistence,
                record,
                request,
                AttachmentTranslationSourceRejectCodeV1::Policy,
                context,
                processed_at_unix_millis,
            )
            .await;
        }
        Err(BlobErrorV1::Unavailable) => {
            return Err(TranslationSourceDeliveryErrorV1::Unavailable);
        }
    };
    let receipt = match write_translation_source_v1(
        control_channel,
        request.translation_run_id,
        snapshot.source_revision,
        &snapshot.artifact,
        bytes,
    ) {
        Ok(receipt) => receipt,
        Err(BlobErrorV1::InvalidReceipt) => {
            return persist_rejection(
                persistence,
                record,
                request,
                AttachmentTranslationSourceRejectCodeV1::Policy,
                context,
                processed_at_unix_millis,
            )
            .await;
        }
        Err(BlobErrorV1::Unavailable) => {
            return Err(TranslationSourceDeliveryErrorV1::Unavailable);
        }
    };
    let result = build_attachment_translation_source_prepared_outbox_record_v1(
        *record.message_id(),
        AttachmentTranslationSourcePreparedV1 {
            request_id: request.request_id.to_vec(),
            translation_run_id: request.translation_run_id.to_vec(),
            source_extraction_run_id: request.source_extraction_run_id.to_vec(),
            source_revision: snapshot.source_revision,
            source_reference_id: receipt.reference_id.to_vec(),
            declared_size: receipt.declared_size,
            receipt_sha256: receipt.receipt_sha256.to_vec(),
            custody_transfer_source_proof: receipt.custody_transfer_source_proof,
            logical_owner_id: request.logical_owner_id.clone(),
        },
        context,
    )
    .map_err(|_| TranslationSourceDeliveryErrorV1::InvalidPayload)?;
    let persisted = persistence
        .persist_translation_source_result(
            &request.logical_owner_id,
            &persistence_record(record, request, &result, processed_at_unix_millis),
            Some(snapshot),
        )
        .await;
    match persisted {
        Ok(_) => Ok(()),
        Err(AttachmentTextExtractionPersistenceErrorV1::EvidenceConflict) => {
            persist_rejection(
                persistence,
                record,
                request,
                AttachmentTranslationSourceRejectCodeV1::StaleRevision,
                context,
                processed_at_unix_millis,
            )
            .await
        }
        Err(error) => Err(persistence_error(error)),
    }
}

async fn persist_rejection(
    persistence: &AttachmentTextExtractionPersistenceV1,
    record: &OutboxRecordV1,
    request: &DecodedTranslationSourceRequestV1,
    code: AttachmentTranslationSourceRejectCodeV1,
    context: &AttachmentTranslationSourceEnvelopeContextV1,
    processed_at_unix_millis: i64,
) -> Result<(), TranslationSourceDeliveryErrorV1> {
    let result = build_attachment_translation_source_rejected_outbox_record_v1(
        *record.message_id(),
        AttachmentTranslationSourceRejectedV1 {
            request_id: request.request_id.to_vec(),
            translation_run_id: request.translation_run_id.to_vec(),
            source_extraction_run_id: request.source_extraction_run_id.to_vec(),
            code: code as i32,
            logical_owner_id: request.logical_owner_id.clone(),
        },
        context,
    )
    .map_err(|_| TranslationSourceDeliveryErrorV1::InvalidPayload)?;
    persistence
        .persist_translation_source_result(
            &request.logical_owner_id,
            &persistence_record(record, request, &result, processed_at_unix_millis),
            None,
        )
        .await
        .map(|_| ())
        .map_err(persistence_error)
}

fn persistence_record(
    request_record: &OutboxRecordV1,
    request: &DecodedTranslationSourceRequestV1,
    result: &OutboxRecordV1,
    processed_at_unix_millis: i64,
) -> PersistTranslationSourceResultV1 {
    PersistTranslationSourceResultV1 {
        request_message_id: *request_record.message_id(),
        request_envelope_sha256: *request_record.envelope_sha256(),
        request_id: request.request_id,
        translation_run_id: request.translation_run_id,
        source_extraction_run_id: request.source_extraction_run_id,
        expected_source_revision: request.expected_source_revision,
        result_message_id: *result.message_id(),
        result_envelope_sha256: *result.envelope_sha256(),
        exact_result_envelope_bytes: result.exact_bytes().to_vec(),
        processed_at_unix_millis,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedTranslationSourceRequestV1 {
    request_id: [u8; 16],
    translation_run_id: [u8; 16],
    source_extraction_run_id: [u8; 16],
    expected_source_revision: u64,
    logical_owner_id: String,
}

fn decode_request(
    record: &OutboxRecordV1,
    logical_owner_id: &str,
) -> Result<DecodedTranslationSourceRequestV1, TranslationSourceDeliveryErrorV1> {
    let envelope = decode_envelope_v1(record.exact_bytes())
        .map_err(|_| TranslationSourceDeliveryErrorV1::InvalidEnvelope)?;
    if !exact_contract(
        envelope.contract.as_ref(),
        &attachment_translation_source_requested_contract_reference_v1(),
    ) || envelope.source.as_ref().is_none_or(|source| {
        source.module_id != ATTACHMENT_TRANSLATION_BLOB_TARGET_MODULE_ID_V1
            || source.runtime_generation == 0
    }) {
        return Err(TranslationSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let Some(Semantics::Command(command)) = envelope.semantics else {
        return Err(TranslationSourceDeliveryErrorV1::InvalidEnvelope);
    };
    if command.target_capability != ATTACHMENT_TEXT_EXTRACTION_TRANSLATION_SOURCE_CAPABILITY_ID_V1 {
        return Err(TranslationSourceDeliveryErrorV1::InvalidEnvelope);
    }
    let payload = RequestAttachmentTranslationSourceV1::decode(envelope.payload.as_slice())
        .map_err(|_| TranslationSourceDeliveryErrorV1::InvalidPayload)?;
    let request_id = id16(&payload.request_id)?;
    let translation_run_id = id16(&payload.translation_run_id)?;
    let source_extraction_run_id = id16(&payload.source_extraction_run_id)?;
    if payload.expected_source_revision == 0
        || request_id
            != attachment_translation_source_request_id_v1(
                translation_run_id,
                source_extraction_run_id,
                payload.expected_source_revision,
            )
        || command.command_id.as_slice() != request_id
        || record.message_id() != &request_id
        || payload.logical_owner_id != logical_owner_id
        || !valid_owner(logical_owner_id)
    {
        return Err(TranslationSourceDeliveryErrorV1::InvalidPayload);
    }
    Ok(DecodedTranslationSourceRequestV1 {
        request_id,
        translation_run_id,
        source_extraction_run_id,
        expected_source_revision: payload.expected_source_revision,
        logical_owner_id: payload.logical_owner_id,
    })
}

fn exact_contract(actual: Option<&ContractRefV1>, expected: &impl ContractLikeV1) -> bool {
    actual.is_some_and(|actual| {
        actual.owner == expected.owner()
            && actual.name == expected.name()
            && actual.major == expected.major()
            && actual.revision == expected.revision()
            && actual.schema_sha256 == expected.schema_sha256()
    })
}

trait ContractLikeV1 {
    fn owner(&self) -> &str;
    fn name(&self) -> &str;
    fn major(&self) -> u32;
    fn revision(&self) -> u32;
    fn schema_sha256(&self) -> &[u8];
}

impl ContractLikeV1 for makosh_runtime_protocol::v1::ContractReferenceV1 {
    fn owner(&self) -> &str {
        &self.owner
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn major(&self) -> u32 {
        self.major
    }
    fn revision(&self) -> u32 {
        self.revision
    }
    fn schema_sha256(&self) -> &[u8] {
        &self.schema_sha256
    }
}

fn id16(value: &[u8]) -> Result<[u8; 16], TranslationSourceDeliveryErrorV1> {
    let value: [u8; 16] = value
        .try_into()
        .map_err(|_| TranslationSourceDeliveryErrorV1::InvalidPayload)?;
    value
        .iter()
        .any(|byte| *byte != 0)
        .then_some(value)
        .ok_or(TranslationSourceDeliveryErrorV1::InvalidPayload)
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn persistence_error(
    error: AttachmentTextExtractionPersistenceErrorV1,
) -> TranslationSourceDeliveryErrorV1 {
    match error {
        AttachmentTextExtractionPersistenceErrorV1::StorageUnavailable => {
            TranslationSourceDeliveryErrorV1::Unavailable
        }
        _ => TranslationSourceDeliveryErrorV1::Persistence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_attachment_translation_ingress::{
        AttachmentTranslationSourceEnvelopeContextV1,
        build_request_attachment_translation_source_outbox_record_v1,
    };

    #[test]
    fn request_decoder_accepts_only_exact_target_owned_command() {
        let translation_run_id = [2; 16];
        let source_extraction_run_id = [3; 16];
        let request_id = attachment_translation_source_request_id_v1(
            translation_run_id,
            source_extraction_run_id,
            4,
        );
        let record = build_request_attachment_translation_source_outbox_record_v1(
            RequestAttachmentTranslationSourceV1 {
                request_id: request_id.to_vec(),
                translation_run_id: translation_run_id.to_vec(),
                source_extraction_run_id: source_extraction_run_id.to_vec(),
                expected_source_revision: 4,
                logical_owner_id: "owner-1".to_owned(),
            },
            1_800_000_030,
            &AttachmentTranslationSourceEnvelopeContextV1 {
                module_id: ATTACHMENT_TRANSLATION_BLOB_TARGET_MODULE_ID_V1.to_owned(),
                runtime_instance_id: "translation-1".to_owned(),
                runtime_generation: 2,
                recorded_at_unix_seconds: 1_800_000_000,
                recorded_at_nanos: 0,
            },
        )
        .unwrap();
        assert_eq!(
            decode_request(&record, "owner-1")
                .unwrap()
                .source_extraction_run_id,
            source_extraction_run_id
        );
        assert_eq!(
            decode_request(&record, "owner-2"),
            Err(TranslationSourceDeliveryErrorV1::InvalidPayload)
        );
    }
}

use std::os::unix::net::UnixStream;

use makosh_ai_contracts::{
    AI_INFERENCE_BLOB_CAPABILITY_ID_V1, ai_provider_explanation_contract_reference_v1,
    ai_provider_reply_generation_contract_reference_v1,
    ai_provider_summary_generation_contract_reference_v1,
    ai_provider_translation_contract_reference_v1, validate_provider_explanation_request_v1,
    validate_provider_explanation_result_v1, validate_provider_reply_generation_request_v1,
    validate_provider_reply_generation_result_v1, validate_provider_summary_generation_request_v1,
    validate_provider_summary_generation_result_v1, validate_provider_translation_request_v1,
    validate_provider_translation_result_v1,
    wire::{
        AiProviderExplanationRequestV1, AiProviderExplanationResultV1,
        AiProviderReplyGenerationRequestV1, AiProviderReplyGenerationResultV1,
        AiProviderSummaryGenerationRequestV1, AiProviderSummaryGenerationResultV1,
        AiProviderTranslationRequestV1, AiProviderTranslationResultV1,
    },
};
use makosh_ai_inference_core::{
    AiAttachmentTranslationExecutionPlanV1, AiExplanationExecutionPlanV1,
    AiInferenceExecutionPlanV1, AiSummaryExecutionPlanV1, AiTranslationExecutionPlanV1,
};
use makosh_blob_client::{
    BlobDataClient, ManagedBlobCustodyTransferRequestV1, ManagedBlobSessionRequestV1,
    request_managed_blob_custody_transfer_v2, request_managed_blob_session_v2,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, ManagedControlRequestDispatcherV2},
    v1::{
        BlobDataOperationV1, ManagedRuntimeControlRequestV1, ManagedRuntimeModuleRequestRequestV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_request::{
        MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1, validate_module_request_request_v1,
        validate_module_request_response_v1,
    },
};
use prost::Message;
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiInferenceSourcePortErrorV1 {
    InvalidReceipt,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiInferenceProviderPortErrorV1 {
    Rejected,
    Unavailable,
}

pub(crate) trait AiInferenceExecutionPortsV1 {
    fn materialize_source(
        &mut self,
        plan: &AiInferenceExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1>;

    fn generate_reply(
        &mut self,
        request: AiProviderReplyGenerationRequestV1,
    ) -> Result<AiProviderReplyGenerationResultV1, AiInferenceProviderPortErrorV1>;

    fn materialize_summary_source(
        &mut self,
        plan: &AiSummaryExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1>;

    fn generate_summary(
        &mut self,
        request: AiProviderSummaryGenerationRequestV1,
    ) -> Result<AiProviderSummaryGenerationResultV1, AiInferenceProviderPortErrorV1>;

    fn materialize_translation_source(
        &mut self,
        plan: &AiTranslationExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1>;

    fn materialize_attachment_translation_source(
        &mut self,
        plan: &AiAttachmentTranslationExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1>;

    fn translate(
        &mut self,
        request: AiProviderTranslationRequestV1,
    ) -> Result<AiProviderTranslationResultV1, AiInferenceProviderPortErrorV1>;

    fn materialize_explanation_source(
        &mut self,
        plan: &AiExplanationExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1>;

    fn explain(
        &mut self,
        request: AiProviderExplanationRequestV1,
    ) -> Result<AiProviderExplanationResultV1, AiInferenceProviderPortErrorV1>;
}

pub(crate) struct ManagedAiInferenceExecutionPortsV1<'a> {
    pub control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub dispatcher: &'a mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
}

impl AiInferenceExecutionPortsV1 for ManagedAiInferenceExecutionPortsV1<'_> {
    fn materialize_source(
        &mut self,
        plan: &AiInferenceExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1> {
        materialize_source(
            self.control_channel,
            self.dispatcher,
            &plan.source,
            &plan.run_id,
            &plan.request_digest,
        )
    }

    fn generate_reply(
        &mut self,
        mut request: AiProviderReplyGenerationRequestV1,
    ) -> Result<AiProviderReplyGenerationResultV1, AiInferenceProviderPortErrorV1> {
        validate_provider_reply_generation_request_v1(&request)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let mut payload = request.encode_to_vec();
        request.input_utf8.zeroize();
        let routed = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(ai_provider_reply_generation_contract_reference_v1()),
            request_payload: std::mem::take(&mut payload),
            deadline_millis: MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1,
            response_blob_capability_id: String::new(),
        };
        validate_module_request_request_v1(&routed)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let response = self
            .control_channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(routed)),
                },
                self.dispatcher,
            )
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if response.request_id != request_id {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        }
        if !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Rejected);
        }
        let result =
            AiProviderReplyGenerationResultV1::decode(response.response_payload.as_slice())
                .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        validate_provider_reply_generation_result_v1(&result)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        Ok(result)
    }

    fn materialize_summary_source(
        &mut self,
        plan: &AiSummaryExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1> {
        materialize_source(
            self.control_channel,
            self.dispatcher,
            &plan.source,
            &plan.run_id,
            &plan.request_digest,
        )
    }

    fn generate_summary(
        &mut self,
        mut request: AiProviderSummaryGenerationRequestV1,
    ) -> Result<AiProviderSummaryGenerationResultV1, AiInferenceProviderPortErrorV1> {
        validate_provider_summary_generation_request_v1(&request)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let mut payload = request.encode_to_vec();
        request.input_utf8.zeroize();
        let routed = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(ai_provider_summary_generation_contract_reference_v1()),
            request_payload: std::mem::take(&mut payload),
            deadline_millis: MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1,
            response_blob_capability_id: String::new(),
        };
        validate_module_request_request_v1(&routed)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let response = self
            .control_channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(routed)),
                },
                self.dispatcher,
            )
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if response.request_id != request_id || !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Rejected);
        }
        let result =
            AiProviderSummaryGenerationResultV1::decode(response.response_payload.as_slice())
                .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        validate_provider_summary_generation_result_v1(&result)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        Ok(result)
    }

    fn materialize_translation_source(
        &mut self,
        plan: &AiTranslationExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1> {
        materialize_source(
            self.control_channel,
            self.dispatcher,
            &plan.source,
            &plan.run_id,
            &plan.request_digest,
        )
    }

    fn materialize_attachment_translation_source(
        &mut self,
        plan: &AiAttachmentTranslationExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1> {
        materialize_source(
            self.control_channel,
            self.dispatcher,
            &plan.source,
            &plan.run_id,
            &plan.request_digest,
        )
    }

    fn translate(
        &mut self,
        mut request: AiProviderTranslationRequestV1,
    ) -> Result<AiProviderTranslationResultV1, AiInferenceProviderPortErrorV1> {
        validate_provider_translation_request_v1(&request)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let mut payload = request.encode_to_vec();
        request.input_utf8.zeroize();
        let routed = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(ai_provider_translation_contract_reference_v1()),
            request_payload: std::mem::take(&mut payload),
            deadline_millis: MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1,
            response_blob_capability_id: String::new(),
        };
        validate_module_request_request_v1(&routed)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let response = self
            .control_channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(routed)),
                },
                self.dispatcher,
            )
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if response.request_id != request_id || !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Rejected);
        }
        let result = AiProviderTranslationResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        validate_provider_translation_result_v1(&result)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        Ok(result)
    }

    fn materialize_explanation_source(
        &mut self,
        plan: &AiExplanationExecutionPlanV1,
    ) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1> {
        materialize_source(
            self.control_channel,
            self.dispatcher,
            &plan.source,
            &plan.run_id,
            &plan.request_digest,
        )
    }

    fn explain(
        &mut self,
        mut request: AiProviderExplanationRequestV1,
    ) -> Result<AiProviderExplanationResultV1, AiInferenceProviderPortErrorV1> {
        validate_provider_explanation_request_v1(&request)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let request_id: [u8; 16] = request
            .request_id
            .as_slice()
            .try_into()
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let mut payload = request.encode_to_vec();
        request.input_utf8.zeroize();
        let routed = ManagedRuntimeModuleRequestRequestV1 {
            request_id: request_id.to_vec(),
            contract: Some(ai_provider_explanation_contract_reference_v1()),
            request_payload: std::mem::take(&mut payload),
            deadline_millis: MODULE_REQUEST_MAX_DEADLINE_MILLIS_V1,
            response_blob_capability_id: String::new(),
        };
        validate_module_request_request_v1(&routed)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        let response = self
            .control_channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::RouteModuleRequest(routed)),
                },
                self.dispatcher,
            )
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        }
        let Some(ControlResult::ModuleRequestRoute(response)) = response.result else {
            return Err(AiInferenceProviderPortErrorV1::Unavailable);
        };
        validate_module_request_response_v1(&response)
            .map_err(|_| AiInferenceProviderPortErrorV1::Unavailable)?;
        if response.request_id != request_id || !response.error_code.is_empty() {
            return Err(AiInferenceProviderPortErrorV1::Rejected);
        }
        let result = AiProviderExplanationResultV1::decode(response.response_payload.as_slice())
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        validate_provider_explanation_result_v1(&result)
            .map_err(|_| AiInferenceProviderPortErrorV1::Rejected)?;
        Ok(result)
    }
}

fn materialize_source(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    source: &makosh_ai_contracts::wire::AiPrivateSourceReceiptV1,
    run_id: &[u8; 16],
    request_digest: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, AiInferenceSourcePortErrorV1> {
    let source_reference_id: [u8; 16] = source
        .reference_id
        .as_slice()
        .try_into()
        .map_err(|_| AiInferenceSourcePortErrorV1::InvalidReceipt)?;
    let source_sha256: [u8; 32] = source
        .sha256
        .as_slice()
        .try_into()
        .map_err(|_| AiInferenceSourcePortErrorV1::InvalidReceipt)?;
    let transfer = request_managed_blob_custody_transfer_v2(
        control_channel,
        dispatcher,
        ManagedBlobCustodyTransferRequestV1 {
            capability_id: AI_INFERENCE_BLOB_CAPABILITY_ID_V1,
            source_reference_id: &source_reference_id,
            declared_size: source.declared_bytes,
            receipt_sha256: &source_sha256,
            custody_source_proof: &source.custody_transfer_source_proof,
            evidence_id: run_id,
            evidence_envelope_sha256: request_digest,
        },
    )
    .map_err(|_| AiInferenceSourcePortErrorV1::Unavailable)?;
    let target_reference_id: [u8; 16] = transfer
        .grant
        .target_reference_id
        .as_slice()
        .try_into()
        .map_err(|_| AiInferenceSourcePortErrorV1::InvalidReceipt)?;
    BlobDataClient::new(&transfer.data_socket_path)
        .and_then(|client| client.custody_transfer(transfer.grant, transfer.channel_binding))
        .map_err(|_| AiInferenceSourcePortErrorV1::Unavailable)?;
    let read = request_managed_blob_session_v2(
        control_channel,
        dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: AI_INFERENCE_BLOB_CAPABILITY_ID_V1,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &target_reference_id,
            declared_size: source.declared_bytes,
            backup_class: 1,
            receipt_sha256: Some(&source_sha256),
            custody_target: None,
        },
    )
    .map_err(|_| AiInferenceSourcePortErrorV1::Unavailable)?;
    let bytes = BlobDataClient::new(read.data_socket_path)
        .and_then(|client| {
            client.read_range(read.grant, read.channel_binding, 0, source.declared_bytes)
        })
        .map_err(|_| AiInferenceSourcePortErrorV1::Unavailable)?;
    if bytes.len() != usize::try_from(source.declared_bytes).unwrap_or(usize::MAX)
        || Sha256::digest(&bytes).as_slice() != source_sha256
    {
        return Err(AiInferenceSourcePortErrorV1::InvalidReceipt);
    }
    Ok(Zeroizing::new(bytes))
}

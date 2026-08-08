//! Exact contract dispatcher for Communications client-facing module requests.

use std::os::unix::net::UnixStream;
use std::sync::Arc;

use makosh_communications_call_evidence_persistence::CommunicationsCallEvidencePersistenceV1;
use makosh_communications_persistence::CommunicationsDurablePersistence;
use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::{ModuleClientRequestV1, ModuleClientResponseV1};
use prost::Message;

use crate::admission::{
    communications_call_evidence_query_contract_reference_v1,
    communications_content_read_contract_reference_v1,
    communications_content_ticket_contract_reference_v1,
    communications_query_contract_reference_v1, communications_saved_search_contract_reference_v1,
    communications_sender_insights_contract_reference_v1,
};
use crate::call_evidence_client_port::{
    CallEvidenceClientPortErrorV1, handle_call_evidence_client_request_v1,
};
use crate::content_blob_client_port::{
    CommunicationsContentBlobClientPortErrorV1, handle_module_content_blob_request_v1,
};
use crate::content_ticket_client_port::{
    CommunicationsContentTicketClientPortErrorV1, handle_module_content_ticket_request_v1,
};
use crate::content_ticket_store::CommunicationsContentTicketStoreV1;
use crate::query_client_port::{
    CommunicationsQueryClientPortErrorV1, handle_module_query_request_v1,
};
use crate::saved_search_port::{
    CommunicationsSavedSearchClientPortErrorV1, handle_module_saved_search_request_v1,
};
use crate::search_access::CommunicationsSearchAccessV1;
use crate::sender_insights_port::{
    CommunicationsSenderInsightsClientPortErrorV1, handle_module_sender_insights_request_v1,
};

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

pub(crate) struct CommunicationsClientRequestDependenciesV1<'a> {
    pub(crate) persistence: &'a CommunicationsDurablePersistence,
    pub(crate) call_evidence_persistence: &'a CommunicationsCallEvidencePersistenceV1,
    pub(crate) logical_human_owner_id: &'a str,
    pub(crate) tickets: &'a Arc<CommunicationsContentTicketStoreV1>,
}

pub(crate) async fn dispatch_module_client_request_v1(
    dependencies: &CommunicationsClientRequestDependenciesV1<'_>,
    search_access: &mut CommunicationsSearchAccessV1,
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    nested_dispatcher: &mut dyn ManagedControlRequestDispatcherV2<UnixStream>,
    request: &ModuleClientRequestV1,
) -> ModuleClientResponseV1 {
    let encoded = request.encode_to_vec();
    let result = if request.contract.as_ref()
        == Some(&communications_call_evidence_query_contract_reference_v1())
    {
        handle_call_evidence_client_request_v1(
            dependencies.call_evidence_persistence,
            dependencies.logical_human_owner_id,
            &encoded,
        )
        .await
        .map_err(map_call_evidence_error)
    } else if request.contract.as_ref() == Some(&communications_query_contract_reference_v1()) {
        handle_module_query_request_v1(
            dependencies.persistence,
            search_access,
            control_channel,
            nested_dispatcher,
            &encoded,
        )
        .await
        .map_err(map_query_error)
    } else if request.contract.as_ref()
        == Some(&communications_content_ticket_contract_reference_v1())
    {
        handle_module_content_ticket_request_v1(
            dependencies.persistence,
            dependencies.tickets,
            &encoded,
        )
        .await
        .map_err(map_ticket_error)
    } else if request.contract.as_ref()
        == Some(&communications_content_read_contract_reference_v1())
    {
        handle_module_content_blob_request_v1(
            dependencies.persistence,
            dependencies.tickets,
            &encoded,
        )
        .await
        .map_err(map_blob_error)
    } else if request.contract.as_ref()
        == Some(&communications_saved_search_contract_reference_v1())
    {
        handle_module_saved_search_request_v1(
            dependencies.persistence,
            search_access,
            control_channel,
            nested_dispatcher,
            &encoded,
        )
        .await
        .map_err(map_saved_search_error)
    } else if request.contract.as_ref()
        == Some(&communications_sender_insights_contract_reference_v1())
    {
        handle_module_sender_insights_request_v1(dependencies.persistence, &encoded)
            .await
            .map_err(map_sender_insights_error)
    } else {
        return module_error(request.request_id, "REJECTED");
    };
    match result {
        Ok(bytes) => ModuleClientResponseV1::decode(bytes.as_slice())
            .ok()
            .filter(|response| {
                response.protocol_major == MODULE_CLIENT_PROTOCOL_MAJOR
                    && response.request_id == request.request_id
            })
            .unwrap_or_else(|| module_error(request.request_id, "UNAVAILABLE")),
        Err(error_code) => module_error(request.request_id, error_code),
    }
}

const fn map_call_evidence_error(error: CallEvidenceClientPortErrorV1) -> &'static str {
    match error {
        CallEvidenceClientPortErrorV1::Protocol => "REJECTED",
        CallEvidenceClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

fn module_error(request_id: u64, error_code: &str) -> ModuleClientResponseV1 {
    ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload: Vec::new(),
        error_code: error_code.to_owned(),
    }
}

const fn map_query_error(error: CommunicationsQueryClientPortErrorV1) -> &'static str {
    match error {
        CommunicationsQueryClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsQueryClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

const fn map_ticket_error(error: CommunicationsContentTicketClientPortErrorV1) -> &'static str {
    match error {
        CommunicationsContentTicketClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsContentTicketClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

const fn map_blob_error(error: CommunicationsContentBlobClientPortErrorV1) -> &'static str {
    match error {
        CommunicationsContentBlobClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsContentBlobClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

const fn map_saved_search_error(error: CommunicationsSavedSearchClientPortErrorV1) -> &'static str {
    match error {
        CommunicationsSavedSearchClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsSavedSearchClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

const fn map_sender_insights_error(
    error: CommunicationsSenderInsightsClientPortErrorV1,
) -> &'static str {
    match error {
        CommunicationsSenderInsightsClientPortErrorV1::Protocol => "REJECTED",
        CommunicationsSenderInsightsClientPortErrorV1::Unavailable => "UNAVAILABLE",
    }
}

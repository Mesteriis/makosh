#![forbid(unsafe_code)]

mod envelope;

pub use envelope::{
    DocumentsEnvelopeBuildErrorV1, DocumentsEnvelopeContextV1,
    build_document_changed_outbox_record_v1,
};

use makosh_runtime_protocol::v1::{
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, capability_request_v1::Request,
};

pub const PACKAGE: &str = "makosh-documents-api";
pub const DOCUMENTS_OWNER_ID_V1: &str = "documents";
pub const DOCUMENTS_MODULE_ID_V1: &str = "makosh-documents-runtime";
pub const DOCUMENTS_BLOB_CAPABILITY_ID_V1: &str = "documents.blob.v1";
pub const DOCUMENTS_CLIENT_CAPABILITY_ID_V1: &str = "documents.client.v1";
pub const DOCUMENTS_LIFECYCLE_EVENT_CAPABILITY_ID_V1: &str = "documents.lifecycle.event.v1";
pub const DOCUMENTS_STORAGE_CAPABILITY_ID_V1: &str = "documents.storage.v1";
pub const DOCUMENTS_CLIENT_CONTRACT_MAJOR_V1: u32 = 1;
pub const DOCUMENTS_CLIENT_CONTRACT_REVISION_V1: u32 = 1;

pub mod client_wire {
    include!(concat!(env!("OUT_DIR"), "/makosh.documents.client.v1.rs"));
}

include!(concat!(env!("OUT_DIR"), "/documents_client_schema.rs"));

pub const DOCUMENTS_CLIENT_DESCRIPTOR_SET_V1: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/documents-client-v1.bin"));

fn contract_reference(name: &str) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: DOCUMENTS_OWNER_ID_V1.to_owned(),
        name: name.to_owned(),
        major: DOCUMENTS_CLIENT_CONTRACT_MAJOR_V1,
        revision: DOCUMENTS_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: DOCUMENTS_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

macro_rules! client_contract {
    ($function:ident, $name:literal) => {
        #[must_use]
        pub fn $function() -> ContractReferenceV1 {
            contract_reference($name)
        }
    };
}

client_contract!(
    documents_client_create_contract_reference_v1,
    "documents_client_create"
);
client_contract!(
    documents_client_update_contract_reference_v1,
    "documents_client_update"
);
client_contract!(
    documents_client_set_state_contract_reference_v1,
    "documents_client_set_state"
);
client_contract!(
    documents_client_attach_blob_contract_reference_v1,
    "documents_client_attach_blob"
);
client_contract!(
    documents_client_release_blob_contract_reference_v1,
    "documents_client_release_blob"
);
client_contract!(
    documents_client_add_source_contract_reference_v1,
    "documents_client_add_source"
);
client_contract!(
    documents_client_remove_source_contract_reference_v1,
    "documents_client_remove_source"
);
client_contract!(
    documents_client_get_contract_reference_v1,
    "documents_client_get"
);
client_contract!(
    documents_client_list_contract_reference_v1,
    "documents_client_list"
);
client_contract!(
    documents_client_search_contract_reference_v1,
    "documents_client_search"
);
client_contract!(
    documents_client_list_sources_contract_reference_v1,
    "documents_client_list_sources"
);

#[must_use]
pub fn documents_lifecycle_event_contract_reference_v1() -> ContractReferenceV1 {
    contract_reference("document_changed")
}

#[must_use]
pub fn documents_lifecycle_event_publish_request_v1() -> CapabilityRequestV1 {
    CapabilityRequestV1 {
        request: Some(Request::EventRoute(EventRouteRequestV1 {
            envelope_kind: DurableEnvelopeKindV1::Event as i32,
            contract: Some(documents_lifecycle_event_contract_reference_v1()),
            direction: EventRouteDirectionV1::Publish as i32,
            max_in_flight: 32,
            subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
            max_deliver: 0,
            ack_wait_millis: 0,
        })),
    }
}

pub const DOCUMENTS_CLIENT_CONNECT_PATHS_V1: [&str; 11] = [
    "/makosh.documents.client.v1.DocumentsCommandService/Create",
    "/makosh.documents.client.v1.DocumentsCommandService/Update",
    "/makosh.documents.client.v1.DocumentsCommandService/SetState",
    "/makosh.documents.client.v1.DocumentsCommandService/AttachBlob",
    "/makosh.documents.client.v1.DocumentsCommandService/ReleaseBlob",
    "/makosh.documents.client.v1.DocumentsCommandService/AddSource",
    "/makosh.documents.client.v1.DocumentsCommandService/RemoveSource",
    "/makosh.documents.client.v1.DocumentsQueryService/Get",
    "/makosh.documents.client.v1.DocumentsQueryService/List",
    "/makosh.documents.client.v1.DocumentsQueryService/Search",
    "/makosh.documents.client.v1.DocumentsQueryService/ListSources",
];

#[must_use]
pub fn documents_client_routes_v1() -> [(ContractReferenceV1, &'static str); 11] {
    [
        (
            documents_client_create_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[0],
        ),
        (
            documents_client_update_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[1],
        ),
        (
            documents_client_set_state_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[2],
        ),
        (
            documents_client_attach_blob_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[3],
        ),
        (
            documents_client_release_blob_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[4],
        ),
        (
            documents_client_add_source_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[5],
        ),
        (
            documents_client_remove_source_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[6],
        ),
        (
            documents_client_get_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[7],
        ),
        (
            documents_client_list_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[8],
        ),
        (
            documents_client_search_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[9],
        ),
        (
            documents_client_list_sources_contract_reference_v1(),
            DOCUMENTS_CLIENT_CONNECT_PATHS_V1[10],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_client_and_capability_surface_is_byte_free() {
        assert_eq!(DOCUMENTS_CLIENT_CONNECT_PATHS_V1.len(), 11);
        assert_eq!(
            [
                DOCUMENTS_BLOB_CAPABILITY_ID_V1,
                DOCUMENTS_CLIENT_CAPABILITY_ID_V1,
                DOCUMENTS_LIFECYCLE_EVENT_CAPABILITY_ID_V1,
                DOCUMENTS_STORAGE_CAPABILITY_ID_V1,
            ],
            [
                "documents.blob.v1",
                "documents.client.v1",
                "documents.lifecycle.event.v1",
                "documents.storage.v1",
            ]
        );
        let descriptor = String::from_utf8_lossy(DOCUMENTS_CLIENT_DESCRIPTOR_SET_V1);
        for forbidden in [
            "content_bytes",
            "storage_path",
            "private_locator",
            "provider_account",
        ] {
            assert!(!descriptor.contains(forbidden));
        }
    }
}

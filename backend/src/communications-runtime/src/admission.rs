//! Exact descriptor and capability admission for the Communications owner runtime.

use makosh_communications_ai_source_api::{
    communication_explanation_source_prepare_consume_request_v1,
    communication_explanation_source_prepared_contract_reference_v1,
    communication_explanation_source_prepared_publish_request_v1,
    communication_explanation_source_rejected_contract_reference_v1,
    communication_explanation_source_rejected_publish_request_v1,
    communication_reply_source_prepare_consume_request_v1,
    communication_reply_source_prepared_contract_reference_v1,
    communication_reply_source_prepared_publish_request_v1,
    communication_reply_source_rejected_contract_reference_v1,
    communication_reply_source_rejected_publish_request_v1,
    communication_summary_source_prepare_consume_request_v1,
    communication_summary_source_prepared_contract_reference_v1,
    communication_summary_source_prepared_publish_request_v1,
    communication_summary_source_rejected_contract_reference_v1,
    communication_summary_source_rejected_publish_request_v1,
    communication_translation_source_prepare_consume_request_v1,
    communication_translation_source_prepared_contract_reference_v1,
    communication_translation_source_prepared_publish_request_v1,
    communication_translation_source_rejected_contract_reference_v1,
    communication_translation_source_rejected_publish_request_v1,
};
use makosh_communications_api::{
    COMMUNICATION_EVIDENCE_SCHEMA_SHA256, COMMUNICATIONS_QUERY_SCHEMA_SHA256,
};
use makosh_communications_attachment_contract::admission::{
    COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
    communication_attachment_anchor_recorded_contract_reference_v1,
    communication_attachment_blob_admission_observed_contract_reference_v1,
    communication_attachment_safety_state_changed_contract_reference_v1,
    communication_attachment_safety_verdict_observed_contract_reference_v1,
};
use makosh_communications_call_evidence_api::{
    CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1, CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
    CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1, CALL_EVIDENCE_CLIENT_OWNER_V1,
    CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1, CALL_EVIDENCE_QUERY_CONNECT_PATH_V1,
    CALL_EVIDENCE_QUERY_CONTRACT_NAME_V1, CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1,
};
use makosh_communications_call_evidence_ingress::{
    call_evidence_observed_consume_request_v1, call_evidence_observed_contract_reference_v1,
};
use makosh_communications_content_api::{
    COMMUNICATIONS_CONTENT_READ_SCHEMA_SHA256, COMMUNICATIONS_CONTENT_TICKET_SCHEMA_SHA256,
    CONTENT_CONTRACT_MAJOR_V1, CONTENT_CONTRACT_REVISION_V1, CONTENT_READ_BLOB_PATH_V1,
    CONTENT_READ_CONTRACT_NAME_V1, CONTENT_TICKET_CONNECT_PATH_V1, CONTENT_TICKET_CONTRACT_NAME_V1,
    MAX_MESSAGE_BODY_BYTES_V1,
};
use makosh_communications_cross_channel_forward_source_api::{
    cross_channel_forward_source_prepare_consume_request_v1,
    cross_channel_forward_source_prepared_contract_reference_v1,
    cross_channel_forward_source_prepared_publish_request_v1,
    cross_channel_forward_source_rejected_contract_reference_v1,
    cross_channel_forward_source_rejected_publish_request_v1,
};
use makosh_communications_evidence_export_source_api::{
    evidence_export_prepare_consume_request_v1, evidence_export_prepared_contract_reference_v1,
    evidence_export_prepared_publish_request_v1, evidence_export_rejected_contract_reference_v1,
    evidence_export_rejected_publish_request_v1,
};
use makosh_communications_ingress::admission::{
    COMMUNICATION_OBSERVED_MAX_IN_FLIGHT, communication_observed_contract_reference_v1,
};
use makosh_communications_ingress::{
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID, COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID,
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
};
use makosh_communications_note_source_api::{
    COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID_V1,
    communication_note_source_prepare_consume_request_v1,
    communication_note_source_prepared_contract_reference_v1,
    communication_note_source_prepared_publish_request_v1,
    communication_note_source_rejected_contract_reference_v1,
    communication_note_source_rejected_publish_request_v1,
};
use makosh_communications_recipient_source_api::{
    COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID_V1,
    communication_recipient_source_prepare_consume_request_v1,
    communication_recipient_source_prepared_contract_reference_v1,
    communication_recipient_source_prepared_publish_request_v1,
    communication_recipient_source_rejected_contract_reference_v1,
    communication_recipient_source_rejected_publish_request_v1,
};
use makosh_communications_retained_evidence_replay_contract::{
    communications_replay_command_consume_request_v1,
    communications_replay_result_publish_request_v1,
};
use makosh_communications_saved_query_api::{
    COMMUNICATIONS_SAVED_SEARCH_SCHEMA_SHA256, SAVED_SEARCH_CONNECT_PATH_V1,
    SAVED_SEARCH_CONTRACT_MAJOR_V1, SAVED_SEARCH_CONTRACT_NAME_V1,
    SAVED_SEARCH_CONTRACT_REVISION_V1,
};
use makosh_communications_sender_insights_api::{
    COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256, SENDER_INSIGHTS_CONNECT_PATH_V1,
    SENDER_INSIGHTS_CONTRACT_MAJOR_V1, SENDER_INSIGHTS_CONTRACT_NAME_V1,
    SENDER_INSIGHTS_CONTRACT_REVISION_V1,
};
use makosh_communications_task_source_api::{
    COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID_V1,
    communication_task_source_prepare_consume_request_v1,
    communication_task_source_prepared_contract_reference_v1,
    communication_task_source_prepared_publish_request_v1,
    communication_task_source_rejected_contract_reference_v1,
    communication_task_source_rejected_publish_request_v1,
};
use makosh_runtime_protocol::v1::{
    BlobQuotaOperationV1, BlobQuotaRequestV1, CapabilityCriticalityV1, CapabilityDescriptorV1,
    CapabilityRequestV1, ContractReferenceV1, DurableEnvelopeKindV1, EventRouteDirectionV1,
    EventRouteRequestV1, EventSubscriptionRequirementV1, ModuleDescriptorV1, ModuleKindV1,
    ProtocolRangeV1, ProvidedSurfaceKindV1, ProvidedSurfaceV1, RuntimeBudgetRequestV1,
    SettingsSchemaRefV1, SettingsSchemaV1, StorageNamespaceRequestV1, VaultActionV1,
    VaultPurposeRequestV1, VaultSecretClassV1, VaultTargetScopeV1, capability_request_v1::Request,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub const COMMUNICATIONS_BLOB_CAPABILITY_ID: &str =
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID;
pub const COMMUNICATIONS_EVENTS_CAPABILITY_ID: &str = "communications.events.v1";
pub const COMMUNICATION_EVIDENCE_CONTRACT_REVISION: u32 = 2;
pub const COMMUNICATIONS_OBSERVE_CAPABILITY_ID: &str = "communications.observe.v1";
pub const COMMUNICATIONS_CALL_EVIDENCE_OBSERVE_CAPABILITY_ID: &str =
    "communications.call-evidence.observe.v1";
pub const COMMUNICATIONS_ATTACHMENT_BLOB_ADMISSION_OBSERVE_CAPABILITY_ID: &str =
    "communications.attachment.blob-admission.observe.v1";
pub const COMMUNICATIONS_ATTACHMENT_SAFETY_VERDICT_OBSERVE_CAPABILITY_ID: &str =
    "communications.attachment.safety-verdict.observe.v1";
pub const COMMUNICATIONS_QUERY_CAPABILITY_ID: &str = "communications.query.v1";
pub const COMMUNICATIONS_CONTENT_CAPABILITY_ID: &str = "communications.content.v1";
pub const COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID: &str = "communications.ai-reply-source.v1";
pub const COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.ai-reply-source.blob.v1";
pub const COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID: &str =
    "communications.ai-explanation-source.v1";
pub const COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.ai-explanation-source.blob.v1";
pub const COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID: &str = "communications.ai-summary-source.v1";
pub const COMMUNICATIONS_SUMMARY_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.ai-summary-source.blob.v1";
pub const COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID: &str =
    "communications.ai-translation-source.v1";
pub const COMMUNICATIONS_TRANSLATION_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.ai-translation-source.blob.v1";
pub const COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID: &str =
    COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID_V1;
pub const COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID: &str =
    "communications.retained-evidence-replay.v1";
pub const COMMUNICATIONS_RECIPIENT_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.recipient-source.blob.v1";
pub const COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID: &str =
    COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID_V1;
pub const COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.note-source.blob.v1";
pub const COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID: &str =
    COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID_V1;
pub const COMMUNICATIONS_TASK_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.task-source.blob.v1";
pub const COMMUNICATIONS_SAVED_SEARCH_CAPABILITY_ID: &str = "communications.saved-search.v1";
pub const COMMUNICATIONS_SENDER_INSIGHTS_CAPABILITY_ID: &str = "communications.sender-insights.v1";
pub const COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID: &str = "communications.export-source.v1";
pub const COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.export-source.blob.v1";
pub const COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID: &str =
    "communications.cross-channel-forward-source.blob.v1";
pub const COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID: &str =
    "communications.cross-channel-forward-source.v1";
pub const COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID: &str = "communications.search.index.v1";
pub const COMMUNICATIONS_STORAGE_CAPABILITY_ID: &str = "communications.storage.v1";
pub const COMMUNICATIONS_MODULE_ID: &str = COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID;
pub const COMMUNICATIONS_OWNER_ID: &str = COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID;
pub const COMMUNICATIONS_BLOB_QUOTA_BYTES: u64 = 1 << 30;
pub const COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID: &str = "communications.evidence.body.v1";
pub const COMMUNICATIONS_STORAGE_CONNECTION_BUDGET: u32 = 8;
pub const COMMUNICATIONS_STORAGE_STATEMENT_TIMEOUT_MILLIS: u32 = 5_000;
pub const COMMUNICATIONS_EVENT_MAX_DELIVER: u32 = 8;
pub const COMMUNICATIONS_EVENT_ACK_WAIT_MILLIS: u32 = 30_000;
pub const COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID: &str = "communications.search.index";
pub const COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION: u32 = 1;
pub const COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS: u32 = 60;

#[must_use]
pub fn communications_admission_capabilities_v1() -> Vec<CapabilityDescriptorV1> {
    vec![
        communications_explanation_source_blob_capability_v1(),
        communications_explanation_source_capability_v1(),
        communications_ai_source_blob_capability_v1(),
        communications_ai_source_capability_v1(),
        communications_summary_source_blob_capability_v1(),
        communications_summary_source_capability_v1(),
        communications_translation_source_blob_capability_v1(),
        communications_translation_source_capability_v1(),
        communications_attachment_blob_admission_observe_capability_v1(),
        communications_attachment_safety_verdict_observe_capability_v1(),
        communications_blob_capability_v1(),
        communications_call_evidence_client_capability_v1(),
        communications_call_evidence_observe_capability_v1(),
        communications_content_capability_v1(),
        communications_cross_channel_forward_source_blob_capability_v1(),
        communications_cross_channel_forward_source_capability_v1(),
        communications_events_capability_v1(),
        communications_export_source_blob_capability_v1(),
        communications_export_source_capability_v1(),
        communications_note_source_blob_capability_v1(),
        communications_note_source_capability_v1(),
        communications_observe_capability_v1(),
        communications_query_capability_v1(),
        communications_recipient_source_blob_capability_v1(),
        communications_recipient_source_capability_v1(),
        communications_retained_evidence_replay_capability_v1(),
        communications_saved_search_capability_v1(),
        communications_search_index_capability_v1(),
        communications_sender_insights_capability_v1(),
        communications_storage_capability_v1(),
        communications_task_source_blob_capability_v1(),
        communications_task_source_capability_v1(),
    ]
}

#[must_use]
pub fn communications_retained_evidence_replay_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![
            communications_replay_command_consume_request_v1(),
            communications_replay_result_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_note_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_note_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_note_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            communication_note_source_prepare_consume_request_v1(),
            communication_note_source_prepared_publish_request_v1(),
            communication_note_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_note_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_task_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_task_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_task_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            communication_task_source_prepare_consume_request_v1(),
            communication_task_source_prepared_publish_request_v1(),
            communication_task_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_task_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_TASK_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_recipient_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_recipient_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_recipient_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            communication_recipient_source_prepare_consume_request_v1(),
            communication_recipient_source_prepared_publish_request_v1(),
            communication_recipient_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_recipient_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_RECIPIENT_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_explanation_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_explanation_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_explanation_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            communication_explanation_source_prepare_consume_request_v1(),
            communication_explanation_source_prepared_publish_request_v1(),
            communication_explanation_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_explanation_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_ai_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_reply_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_reply_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            communication_reply_source_prepare_consume_request_v1(),
            communication_reply_source_prepared_publish_request_v1(),
            communication_reply_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_ai_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_summary_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_summary_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_summary_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            communication_summary_source_prepare_consume_request_v1(),
            communication_summary_source_prepared_publish_request_v1(),
            communication_summary_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_summary_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_SUMMARY_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_translation_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_translation_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(communication_translation_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            communication_translation_source_prepare_consume_request_v1(),
            communication_translation_source_prepared_publish_request_v1(),
            communication_translation_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_translation_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_TRANSLATION_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_call_evidence_client_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::QueryRpc as i32,
                contract: Some(communications_call_evidence_query_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                contract: Some(communications_call_evidence_query_contract_reference_v1()),
                client_rpc_route: Some(makosh_runtime_protocol::v1::ClientRpcRouteV1 {
                    path: CALL_EVIDENCE_QUERY_CONNECT_PATH_V1.to_owned(),
                }),
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRealtime as i32,
                contract: Some(communications_call_evidence_realtime_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_call_evidence_observe_capability_v1() -> CapabilityDescriptorV1 {
    let observation = call_evidence_observed_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_CALL_EVIDENCE_OBSERVE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(observation),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![call_evidence_observed_consume_request_v1()],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_cross_channel_forward_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(cross_channel_forward_source_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(cross_channel_forward_source_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            cross_channel_forward_source_prepare_consume_request_v1(),
            cross_channel_forward_source_prepared_publish_request_v1(),
            cross_channel_forward_source_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_cross_channel_forward_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_export_source_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::Write as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_export_source_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(evidence_export_prepared_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(evidence_export_rejected_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            evidence_export_prepare_consume_request_v1(),
            evidence_export_prepared_publish_request_v1(),
            evidence_export_rejected_publish_request_v1(),
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_content_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_CONTENT_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                contract: Some(communications_content_ticket_contract_reference_v1()),
                client_rpc_route: Some(makosh_runtime_protocol::v1::ClientRpcRouteV1 {
                    path: CONTENT_TICKET_CONNECT_PATH_V1.to_owned(),
                }),
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientBlob as i32,
                contract: Some(communications_content_read_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: Some(makosh_runtime_protocol::v1::ClientBlobRouteV1 {
                    path: CONTENT_READ_BLOB_PATH_V1.to_owned(),
                    max_response_bytes: MAX_MESSAGE_BODY_BYTES_V1,
                }),
            },
        ],
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![BlobQuotaOperationV1::ReadRange as i32],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_blob_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_BLOB_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::BlobQuota(BlobQuotaRequestV1 {
                max_bytes: COMMUNICATIONS_BLOB_QUOTA_BYTES,
                custody_scope_id: COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID.to_owned(),
                allowed_operations: vec![
                    BlobQuotaOperationV1::ReadRange as i32,
                    BlobQuotaOperationV1::CustodyTransfer as i32,
                ],
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_events_capability_v1() -> CapabilityDescriptorV1 {
    let recorded = communication_evidence_recorded_contract_reference_v1();
    let attachment_state_changed =
        communication_attachment_safety_state_changed_contract_reference_v1();
    let attachment_anchor_recorded =
        communication_attachment_anchor_recorded_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_EVENTS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(recorded.clone()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(attachment_state_changed.clone()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::DurablePublisher as i32,
                contract: Some(attachment_anchor_recorded.clone()),
                client_rpc_route: None,
                client_blob_route: None,
            },
        ],
        requests: vec![
            CapabilityRequestV1 {
                request: Some(Request::EventRoute(EventRouteRequestV1 {
                    envelope_kind: DurableEnvelopeKindV1::Event as i32,
                    contract: Some(recorded),
                    direction: EventRouteDirectionV1::Publish as i32,
                    max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
                    subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
                    max_deliver: 0,
                    ack_wait_millis: 0,
                })),
            },
            CapabilityRequestV1 {
                request: Some(Request::EventRoute(EventRouteRequestV1 {
                    envelope_kind: DurableEnvelopeKindV1::Event as i32,
                    contract: Some(attachment_anchor_recorded),
                    direction: EventRouteDirectionV1::Publish as i32,
                    max_in_flight: COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
                    subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
                    max_deliver: 0,
                    ack_wait_millis: 0,
                })),
            },
            CapabilityRequestV1 {
                request: Some(Request::EventRoute(EventRouteRequestV1 {
                    envelope_kind: DurableEnvelopeKindV1::Event as i32,
                    contract: Some(attachment_state_changed),
                    direction: EventRouteDirectionV1::Publish as i32,
                    max_in_flight: COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
                    subscription_requirement: EventSubscriptionRequirementV1::Unspecified as i32,
                    max_deliver: 0,
                    ack_wait_millis: 0,
                })),
            },
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_observe_capability_v1() -> CapabilityDescriptorV1 {
    let observed = communication_observed_contract_reference_v1();
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_OBSERVE_CAPABILITY_ID.to_owned(),
        capability_revision: 2,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(observed.clone()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::EventRoute(EventRouteRequestV1 {
                envelope_kind: DurableEnvelopeKindV1::Observation as i32,
                contract: Some(observed),
                direction: EventRouteDirectionV1::Consume as i32,
                max_in_flight: COMMUNICATION_OBSERVED_MAX_IN_FLIGHT,
                subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                max_deliver: COMMUNICATIONS_EVENT_MAX_DELIVER,
                ack_wait_millis: COMMUNICATIONS_EVENT_ACK_WAIT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_attachment_blob_admission_observe_capability_v1() -> CapabilityDescriptorV1 {
    attachment_observation_consumer_capability_v1(
        COMMUNICATIONS_ATTACHMENT_BLOB_ADMISSION_OBSERVE_CAPABILITY_ID,
        communication_attachment_blob_admission_observed_contract_reference_v1(),
    )
}

#[must_use]
pub fn communications_attachment_safety_verdict_observe_capability_v1() -> CapabilityDescriptorV1 {
    attachment_observation_consumer_capability_v1(
        COMMUNICATIONS_ATTACHMENT_SAFETY_VERDICT_OBSERVE_CAPABILITY_ID,
        communication_attachment_safety_verdict_observed_contract_reference_v1(),
    )
}

fn attachment_observation_consumer_capability_v1(
    capability_id: &str,
    observation: ContractReferenceV1,
) -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: capability_id.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::DurableConsumer as i32,
            contract: Some(observation.clone()),
            client_rpc_route: None,
            client_blob_route: None,
        }],
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::EventRoute(EventRouteRequestV1 {
                envelope_kind: DurableEnvelopeKindV1::Observation as i32,
                contract: Some(observation),
                direction: EventRouteDirectionV1::Consume as i32,
                max_in_flight: COMMUNICATION_ATTACHMENT_MAX_IN_FLIGHT,
                subscription_requirement: EventSubscriptionRequirementV1::Required as i32,
                max_deliver: COMMUNICATIONS_EVENT_MAX_DELIVER,
                ack_wait_millis: COMMUNICATIONS_EVENT_ACK_WAIT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_query_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_QUERY_CAPABILITY_ID.to_owned(),
        capability_revision: 3,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::QueryRpc as i32,
                contract: Some(communications_query_contract_reference_v1()),
                client_rpc_route: None,
                client_blob_route: None,
            },
            ProvidedSurfaceV1 {
                kind: ProvidedSurfaceKindV1::ClientRpc as i32,
                contract: Some(communications_query_contract_reference_v1()),
                client_rpc_route: Some(makosh_runtime_protocol::v1::ClientRpcRouteV1 {
                    path: "/makosh.communications.query.v1.CommunicationsQueryService/Query"
                        .to_owned(),
                }),
                client_blob_route: None,
            },
        ],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_saved_search_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_SAVED_SEARCH_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(communications_saved_search_contract_reference_v1()),
            client_rpc_route: Some(makosh_runtime_protocol::v1::ClientRpcRouteV1 {
                path: SAVED_SEARCH_CONNECT_PATH_V1.to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_sender_insights_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_SENDER_INSIGHTS_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        provides: vec![ProvidedSurfaceV1 {
            kind: ProvidedSurfaceKindV1::ClientRpc as i32,
            contract: Some(communications_sender_insights_contract_reference_v1()),
            client_rpc_route: Some(makosh_runtime_protocol::v1::ClientRpcRouteV1 {
                path: SENDER_INSIGHTS_CONNECT_PATH_V1.to_owned(),
            }),
            client_blob_route: None,
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_search_index_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::VaultPurpose(VaultPurposeRequestV1 {
                purpose_id: COMMUNICATIONS_SEARCH_INDEX_PURPOSE_ID.to_owned(),
                requested_lease_ttl_seconds: COMMUNICATIONS_SEARCH_INDEX_LEASE_TTL_SECONDS,
                allowed_secret_classes: vec![VaultSecretClassV1::OwnerDerivedKey as i32],
                actions: vec![VaultActionV1::IssueOwnerDerivedKey as i32],
                target_scope: VaultTargetScopeV1::OwnerDerivedProjectionKey as i32,
                key_schema_revision: COMMUNICATIONS_SEARCH_INDEX_KEY_SCHEMA_REVISION,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_storage_capability_v1() -> CapabilityDescriptorV1 {
    CapabilityDescriptorV1 {
        capability_id: COMMUNICATIONS_STORAGE_CAPABILITY_ID.to_owned(),
        capability_revision: 1,
        criticality: CapabilityCriticalityV1::Required as i32,
        requests: vec![CapabilityRequestV1 {
            request: Some(Request::StorageNamespace(StorageNamespaceRequestV1 {
                owner_id: "communications".to_owned(),
                connection_budget: COMMUNICATIONS_STORAGE_CONNECTION_BUDGET,
                timeout_millis: COMMUNICATIONS_STORAGE_STATEMENT_TIMEOUT_MILLIS,
            })),
        }],
        ..Default::default()
    }
}

#[must_use]
pub fn communications_query_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: "communications.query".to_owned(),
        major: 1,
        revision: 1,
        schema_sha256: COMMUNICATIONS_QUERY_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communications_call_evidence_query_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CALL_EVIDENCE_CLIENT_OWNER_V1.to_owned(),
        name: CALL_EVIDENCE_QUERY_CONTRACT_NAME_V1.to_owned(),
        major: CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
        revision: CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn communications_call_evidence_realtime_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: CALL_EVIDENCE_CLIENT_OWNER_V1.to_owned(),
        name: CALL_EVIDENCE_REALTIME_CONTRACT_NAME_V1.to_owned(),
        major: CALL_EVIDENCE_CLIENT_CONTRACT_MAJOR_V1,
        revision: CALL_EVIDENCE_CLIENT_CONTRACT_REVISION_V1,
        schema_sha256: CALL_EVIDENCE_CLIENT_SCHEMA_SHA256_V1.to_vec(),
    }
}

#[must_use]
pub fn communications_content_ticket_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: CONTENT_TICKET_CONTRACT_NAME_V1.to_owned(),
        major: CONTENT_CONTRACT_MAJOR_V1,
        revision: CONTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_CONTENT_TICKET_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communications_content_read_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: CONTENT_READ_CONTRACT_NAME_V1.to_owned(),
        major: CONTENT_CONTRACT_MAJOR_V1,
        revision: CONTENT_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_CONTENT_READ_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communications_saved_search_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: SAVED_SEARCH_CONTRACT_NAME_V1.to_owned(),
        major: SAVED_SEARCH_CONTRACT_MAJOR_V1,
        revision: SAVED_SEARCH_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_SAVED_SEARCH_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communications_sender_insights_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: SENDER_INSIGHTS_CONTRACT_NAME_V1.to_owned(),
        major: SENDER_INSIGHTS_CONTRACT_MAJOR_V1,
        revision: SENDER_INSIGHTS_CONTRACT_REVISION_V1,
        schema_sha256: COMMUNICATIONS_SENDER_INSIGHTS_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communication_evidence_recorded_contract_reference_v1() -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: COMMUNICATIONS_OWNER_ID.to_owned(),
        name: "communication_evidence_recorded".to_owned(),
        major: 1,
        revision: COMMUNICATION_EVIDENCE_CONTRACT_REVISION,
        schema_sha256: COMMUNICATION_EVIDENCE_SCHEMA_SHA256.to_vec(),
    }
}

#[must_use]
pub fn communications_settings_schema_v1() -> SettingsSchemaV1 {
    SettingsSchemaV1 {
        major: 1,
        revision: 1,
        definitions: Vec::new(),
    }
}

#[must_use]
pub fn communications_settings_schema_bytes_v1() -> Vec<u8> {
    communications_settings_schema_v1().encode_to_vec()
}

#[must_use]
pub fn communications_module_descriptor_v1(build_id: &str) -> ModuleDescriptorV1 {
    let settings_schema = communications_settings_schema_bytes_v1();
    ModuleDescriptorV1 {
        descriptor_major: 1,
        descriptor_revision: 8,
        module_id: COMMUNICATIONS_MODULE_ID.to_owned(),
        owner_id: COMMUNICATIONS_OWNER_ID.to_owned(),
        module_kind: ModuleKindV1::Domain as i32,
        module_version: "1".to_owned(),
        build_id: build_id.to_owned(),
        runtime_protocol_range: Some(ProtocolRangeV1 {
            minimum_major: 2,
            maximum_major: 2,
            minimum_revision: 1,
        }),
        capabilities: communications_admission_capabilities_v1(),
        settings_schema_ref: Some(SettingsSchemaRefV1 {
            major: 1,
            revision: 1,
            artifact_size_bytes: settings_schema.len() as u64,
            sha256: Sha256::digest(&settings_schema).to_vec(),
        }),
        runtime_budget_request: Some(RuntimeBudgetRequestV1 {
            max_processes: 1,
            max_connections: COMMUNICATIONS_STORAGE_CONNECTION_BUDGET,
            max_memory_bytes: 512 * 1024 * 1024,
            max_cpu_millis: 1_000,
        }),
        display_name: "Communications".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::validation::descriptor::{
        validate_descriptor_v1, validate_settings_schema_v1,
    };

    use super::*;

    #[test]
    fn first_owner_descriptor_is_valid_and_exact() {
        let descriptor = communications_module_descriptor_v1("test");

        assert_eq!(validate_descriptor_v1(&descriptor), Ok(()));
        assert_eq!(
            descriptor.runtime_protocol_range,
            Some(ProtocolRangeV1 {
                minimum_major: 2,
                maximum_major: 2,
                minimum_revision: 1,
            })
        );
        assert_eq!(
            descriptor
                .capabilities
                .iter()
                .map(|capability| capability.capability_id.as_str())
                .collect::<Vec<_>>(),
            [
                COMMUNICATIONS_EXPLANATION_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_EXPLANATION_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_AI_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_AI_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_SUMMARY_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_SUMMARY_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_TRANSLATION_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_TRANSLATION_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_ATTACHMENT_BLOB_ADMISSION_OBSERVE_CAPABILITY_ID,
                COMMUNICATIONS_ATTACHMENT_SAFETY_VERDICT_OBSERVE_CAPABILITY_ID,
                COMMUNICATIONS_BLOB_CAPABILITY_ID,
                CALL_EVIDENCE_CLIENT_CAPABILITY_ID_V1,
                COMMUNICATIONS_CALL_EVIDENCE_OBSERVE_CAPABILITY_ID,
                COMMUNICATIONS_CONTENT_CAPABILITY_ID,
                COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_CROSS_CHANNEL_FORWARD_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_EVENTS_CAPABILITY_ID,
                COMMUNICATIONS_EXPORT_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_EXPORT_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_NOTE_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_NOTE_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_OBSERVE_CAPABILITY_ID,
                COMMUNICATIONS_QUERY_CAPABILITY_ID,
                COMMUNICATIONS_RECIPIENT_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_RECIPIENT_SOURCE_CAPABILITY_ID,
                COMMUNICATIONS_RETAINED_EVIDENCE_REPLAY_CAPABILITY_ID,
                COMMUNICATIONS_SAVED_SEARCH_CAPABILITY_ID,
                COMMUNICATIONS_SEARCH_INDEX_CAPABILITY_ID,
                COMMUNICATIONS_SENDER_INSIGHTS_CAPABILITY_ID,
                COMMUNICATIONS_STORAGE_CAPABILITY_ID,
                COMMUNICATIONS_TASK_SOURCE_BLOB_CAPABILITY_ID,
                COMMUNICATIONS_TASK_SOURCE_CAPABILITY_ID,
            ]
        );
        assert!(
            descriptor
                .capabilities
                .iter()
                .flat_map(|capability| capability.requests.iter())
                .filter_map(|request| match request.request.as_ref() {
                    Some(Request::BlobQuota(quota))
                        if quota.custody_scope_id == COMMUNICATIONS_BLOB_CUSTODY_SCOPE_ID =>
                    {
                        Some(quota.max_bytes)
                    }
                    _ => None,
                })
                .all(|max_bytes| max_bytes == COMMUNICATIONS_BLOB_QUOTA_BYTES),
            "every capability sharing Communications Blob custody must request the same quota"
        );
        assert_eq!(
            validate_settings_schema_v1(&communications_settings_schema_v1()),
            Ok(())
        );
    }
}

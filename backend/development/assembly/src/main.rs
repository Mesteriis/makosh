//! Owner-authorized composition of the exact local development module plan.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use makosh_gateway_protocol::owner_control_client::{
    OwnerControlClientV1, OwnerControlProofSignerV1,
};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use sha2::{Digest, Sha256};

const STATE_FILE: &str = "development-assembly-state-v1";
const ENSEMBLE_RESERVATION_FILE: &str = "development-ensemble-reservation-v2";
const DEVICE_KEY_FILE: &str = "device-es256.key";
const COMMUNICATIONS_RUNTIME_ARTIFACT: &str = "communications.runtime.v1";
const COMMUNICATIONS_STORAGE_ARTIFACT: &str = "communications.storage.v1";
const COMMUNICATIONS_STORAGE_CAPABILITY: &str = "communications.storage.v1";
const COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT: &str = "communications_export.runtime.v1";
const COMMUNICATIONS_EXPORT_STORAGE_ARTIFACT: &str = "communications_export.storage.v1";
const COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY: &str = "communications_export.storage.v1";
const COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT: &str =
    "communication_delivery_intent.runtime.v1";
const COMMUNICATION_DELIVERY_INTENT_STORAGE_ARTIFACT: &str =
    "communication_delivery_intent.storage.v1";
const COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY: &str =
    "communication_delivery_intent.storage.v1";
const COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT: &str = "communication_bulk_action.runtime.v1";
const COMMUNICATION_BULK_ACTION_STORAGE_ARTIFACT: &str = "communication_bulk_action.storage.v1";
const COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY: &str = "communication_bulk_action.storage.v1";
const COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT: &str =
    "communication_delayed_delivery.runtime.v1";
const COMMUNICATION_DELAYED_DELIVERY_STORAGE_ARTIFACT: &str =
    "communication_delayed_delivery.storage.v1";
const COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY: &str =
    "communication.delayed_delivery.storage.v1";
const ATTACHMENT_SECURITY_RUNTIME_ARTIFACT: &str = "attachment_security.runtime.v1";
const ATTACHMENT_SECURITY_STORAGE_ARTIFACT: &str = "attachment_security.storage.v1";
const ATTACHMENT_SECURITY_STORAGE_CAPABILITY: &str = "attachment_security.storage.v1";
const ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT: &str = "attachment_text_extraction.runtime.v1";
const ATTACHMENT_TEXT_EXTRACTION_STORAGE_ARTIFACT: &str = "attachment_text_extraction.storage.v1";
const ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY: &str = "attachment_text_extraction.storage.v1";
const ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT: &str = "attachment_preview.runtime.v1";
const ATTACHMENT_PREVIEW_STORAGE_ARTIFACT: &str = "attachment_preview.storage.v1";
const ATTACHMENT_PREVIEW_STORAGE_CAPABILITY: &str = "attachment_preview.storage.v1";
const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT: &str =
    "attachment_preview_evidence_replay.runtime.v1";
const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_ARTIFACT: &str =
    "attachment_preview_evidence_replay.storage.v1";
const ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY: &str =
    "attachment_preview_evidence_replay.storage.v1";
const ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT: &str = "attachment_translation.runtime.v1";
const ATTACHMENT_TRANSLATION_STORAGE_ARTIFACT: &str = "attachment_translation.storage.v1";
const ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY: &str = "attachment_translation.storage.v1";
const AI_INFERENCE_RUNTIME_ARTIFACT: &str = "ai_inference.runtime.v1";
const AI_INFERENCE_STORAGE_ARTIFACT: &str = "ai_inference.storage.v1";
const AI_INFERENCE_STORAGE_CAPABILITY: &str = "ai.inference.storage.v1";
const OLLAMA_AI_RUNTIME_ARTIFACT: &str = "ollama_ai.runtime.v1";
const OLLAMA_AI_STORAGE_ARTIFACT: &str = "ollama_ai.storage.v1";
const OLLAMA_AI_STORAGE_CAPABILITY: &str = "ollama.ai.storage.v1";
const SPEECH_TO_TEXT_RUNTIME_ARTIFACT: &str = "speech_to_text.runtime.v1";
const SPEECH_TO_TEXT_STORAGE_ARTIFACT: &str = "speech_to_text.storage.v1";
const SPEECH_TO_TEXT_STORAGE_CAPABILITY: &str = "speech_to_text.storage.v1";
const WHISPER_STT_RUNTIME_ARTIFACT: &str = "whisper_stt.runtime.v1";
const WHISPER_STT_STORAGE_ARTIFACT: &str = "whisper_stt.storage.v1";
const WHISPER_STT_STORAGE_CAPABILITY: &str = "whisper_stt.storage.v1";
const CALENDAR_RUNTIME_ARTIFACT: &str = "calendar.runtime.v1";
const CALENDAR_STORAGE_ARTIFACT: &str = "calendar.storage.v1";
const CALENDAR_STORAGE_CAPABILITY: &str = "calendar.storage.v1";
const ORGANIZATIONS_RUNTIME_ARTIFACT: &str = "organizations.runtime.v1";
const ORGANIZATIONS_STORAGE_ARTIFACT: &str = "organizations.storage.v1";
const ORGANIZATIONS_STORAGE_CAPABILITY: &str = "organizations.storage.v1";
const DOCUMENTS_RUNTIME_ARTIFACT: &str = "documents.runtime.v1";
const DOCUMENTS_STORAGE_ARTIFACT: &str = "documents.storage.v1";
const DOCUMENTS_STORAGE_CAPABILITY: &str = "documents.storage.v1";
const RELATIONSHIPS_RUNTIME_ARTIFACT: &str = "relationships.runtime.v1";
const RELATIONSHIPS_STORAGE_ARTIFACT: &str = "relationships.storage.v1";
const RELATIONSHIPS_STORAGE_CAPABILITY: &str = "relationships.storage.v1";
const PROJECTS_RUNTIME_ARTIFACT: &str = "projects.runtime.v1";
const PROJECTS_STORAGE_ARTIFACT: &str = "projects.storage.v1";
const PROJECTS_STORAGE_CAPABILITY: &str = "projects.storage.v1";
const DECISIONS_RUNTIME_ARTIFACT: &str = "decisions.runtime.v1";
const DECISIONS_STORAGE_ARTIFACT: &str = "decisions.storage.v1";
const DECISIONS_STORAGE_CAPABILITY: &str = "decisions.storage.v1";
const REVIEW_OBLIGATION_CANDIDATE_RUNTIME_ARTIFACT: &str = "review.obligation-candidate.runtime.v1";
const REVIEW_OBLIGATION_CANDIDATE_STORAGE_ARTIFACT: &str = "review.obligation-candidate.storage.v1";
const REVIEW_OBLIGATION_CANDIDATE_STORAGE_CAPABILITY: &str =
    "review.obligation-candidate.storage.v1";
const OBLIGATIONS_RUNTIME_ARTIFACT: &str = "obligations.runtime.v1";
const OBLIGATIONS_STORAGE_ARTIFACT: &str = "obligations.storage.v1";
const OBLIGATIONS_STORAGE_CAPABILITY: &str = "obligations.storage.v1";
const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT: &str =
    "reviewed_obligation_candidate_promotion.runtime.v1";
const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_ARTIFACT: &str =
    "reviewed_obligation_candidate_promotion.storage.v1";
const REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_CAPABILITY: &str =
    "reviewed_obligation_candidate_promotion.storage.v1";
const MAIL_RUNTIME_ARTIFACT: &str = "mail.runtime.v1";
const MAIL_STORAGE_ARTIFACT: &str = "mail.storage.v1";
const MAIL_STORAGE_CAPABILITY: &str = "mail.storage.v1";
const PERSONS_RUNTIME_ARTIFACT: &str = "persons.runtime.v1";
const PERSONS_STORAGE_ARTIFACT: &str = "persons.storage.v1";
const PERSONS_STORAGE_CAPABILITY: &str = "persons.storage.v1";
const IDENTITY_RESOLUTION_RUNTIME_ARTIFACT: &str = "identity_resolution.runtime.v1";
const IDENTITY_RESOLUTION_STORAGE_ARTIFACT: &str = "identity_resolution.storage.v1";
const IDENTITY_RESOLUTION_STORAGE_CAPABILITY: &str = "identity_resolution.storage.v1";
const SEARCH_RUNTIME_ARTIFACT: &str = "search.runtime.v1";
const SEARCH_STORAGE_ARTIFACT: &str = "search.storage.v1";
const SEARCH_STORAGE_CAPABILITY: &str = "search.storage.v1";
const TIMELINE_RUNTIME_ARTIFACT: &str = "timeline.runtime.v1";
const TIMELINE_STORAGE_ARTIFACT: &str = "timeline.storage.v1";
const TIMELINE_STORAGE_CAPABILITY: &str = "timeline.storage.v1";
const GRAPH_RUNTIME_ARTIFACT: &str = "graph.runtime.v1";
const GRAPH_STORAGE_ARTIFACT: &str = "graph.storage.v1";
const GRAPH_STORAGE_CAPABILITY: &str = "graph.storage.v1";
const MEMORY_RUNTIME_ARTIFACT: &str = "memory.runtime.v1";
const MEMORY_STORAGE_ARTIFACT: &str = "memory.storage.v1";
const MEMORY_STORAGE_CAPABILITY: &str = "memory.storage.v1";
const CONSISTENCY_RUNTIME_ARTIFACT: &str = "consistency.runtime.v1";
const CONSISTENCY_STORAGE_ARTIFACT: &str = "consistency.storage.v1";
const CONSISTENCY_STORAGE_CAPABILITY: &str = "consistency.storage.v1";
const RISK_RUNTIME_ARTIFACT: &str = "risk.runtime.v1";
const RISK_STORAGE_ARTIFACT: &str = "risk.storage.v1";
const RISK_STORAGE_CAPABILITY: &str = "risk.storage.v1";
const ZOOM_RUNTIME_ARTIFACT: &str = "zoom.runtime.v1";
const ZOOM_STORAGE_ARTIFACT: &str = "zoom.storage.v1";
const ZOOM_STORAGE_CAPABILITY: &str = "zoom.storage.v1";
const TELEMOST_RUNTIME_ARTIFACT: &str = "telemost.runtime.v1";
const TELEMOST_STORAGE_ARTIFACT: &str = "telemost.storage.v1";
const TELEMOST_STORAGE_CAPABILITY: &str = "telemost.storage.v1";
const OMNIROUTE_RUNTIME_ARTIFACT: &str = "omniroute.runtime.v1";
const OMNIROUTE_STORAGE_ARTIFACT: &str = "omniroute.storage.v1";
const OMNIROUTE_STORAGE_CAPABILITY: &str = "omniroute.ai.storage.v1";
const MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT: &str = "mail_persons_sync.runtime.v1";
const MAIL_PERSONS_SYNC_STORAGE_ARTIFACT: &str = "mail_persons_sync.storage.v1";
const MAIL_PERSONS_SYNC_STORAGE_CAPABILITY: &str = "mail_persons_sync.storage.v1";
const REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT: &str =
    "review.person-match-candidate.runtime.v1";
const REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_ARTIFACT: &str =
    "review.person-match-candidate.storage.v1";
const REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY: &str =
    "review.person-match-candidate.storage.v1";
const REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT: &str =
    "reviewed-person-match-candidate-promotion.runtime.v1";
const REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_ARTIFACT: &str =
    "reviewed-person-match-candidate-promotion.storage.v1";
const REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY: &str =
    "reviewed-person-match-candidate-promotion.storage.v1";
const TELEGRAM_RUNTIME_ARTIFACT: &str = "telegram.runtime.v1";
const TELEGRAM_STORAGE_ARTIFACT: &str = "telegram.storage.v1";
const TELEGRAM_STORAGE_CAPABILITY: &str = "telegram.storage.v1";
const WHATSAPP_RUNTIME_ARTIFACT: &str = "whatsapp.runtime.v1";
const WHATSAPP_STORAGE_ARTIFACT: &str = "whatsapp.storage.v1";
const WHATSAPP_STORAGE_CAPABILITY: &str = "whatsapp.storage.v1";
const ZULIP_RUNTIME_ARTIFACT: &str = "zulip.runtime.v1";
const ZULIP_STORAGE_ARTIFACT: &str = "zulip.storage.v1";
const ZULIP_STORAGE_CAPABILITY: &str = "zulip.storage.v1";

#[derive(Parser)]
#[command(name = "makosh-development-assembly")]
struct Cli {
    #[arg(long)]
    data_dir: PathBuf,
    #[arg(long, default_value = "makosh-local-development")]
    distribution_id: String,
    #[arg(long, default_value_t = 1)]
    distribution_generation: u64,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ProvisionPlatform,
    RuntimeDirectory,
    Admit,
    StartEnsemble,
    Status,
    RetireRegistration {
        #[arg(long)]
        registration_id: String,
    },
}

#[derive(Clone, Copy)]
enum ModuleRuntimeKindV1 {
    Domain,
    Engine,
    Integration,
    Workflow,
}

#[derive(Clone, Copy)]
struct ModulePlanV1 {
    runtime_artifact_id: &'static str,
    storage_artifact_id: &'static str,
    storage_capability_id: &'static str,
    runtime_kind: ModuleRuntimeKindV1,
    request_host_bridge: bool,
}

const MODULE_PLAN: [ModulePlanV1; 41] = [
    ModulePlanV1 {
        runtime_artifact_id: COMMUNICATIONS_RUNTIME_ARTIFACT,
        storage_artifact_id: COMMUNICATIONS_STORAGE_ARTIFACT,
        storage_capability_id: COMMUNICATIONS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
        storage_artifact_id: COMMUNICATIONS_EXPORT_STORAGE_ARTIFACT,
        storage_capability_id: COMMUNICATIONS_EXPORT_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
        storage_artifact_id: COMMUNICATION_DELIVERY_INTENT_STORAGE_ARTIFACT,
        storage_capability_id: COMMUNICATION_DELIVERY_INTENT_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
        storage_artifact_id: COMMUNICATION_BULK_ACTION_STORAGE_ARTIFACT,
        storage_capability_id: COMMUNICATION_BULK_ACTION_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
        storage_artifact_id: COMMUNICATION_DELAYED_DELIVERY_STORAGE_ARTIFACT,
        storage_capability_id: COMMUNICATION_DELAYED_DELIVERY_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
        storage_artifact_id: ATTACHMENT_SECURITY_STORAGE_ARTIFACT,
        storage_capability_id: ATTACHMENT_SECURITY_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
        storage_artifact_id: ATTACHMENT_TEXT_EXTRACTION_STORAGE_ARTIFACT,
        storage_capability_id: ATTACHMENT_TEXT_EXTRACTION_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
        storage_artifact_id: ATTACHMENT_PREVIEW_STORAGE_ARTIFACT,
        storage_capability_id: ATTACHMENT_PREVIEW_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
        storage_artifact_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_ARTIFACT,
        storage_capability_id: ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
        storage_artifact_id: ATTACHMENT_TRANSLATION_STORAGE_ARTIFACT,
        storage_capability_id: ATTACHMENT_TRANSLATION_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: MAIL_RUNTIME_ARTIFACT,
        storage_artifact_id: MAIL_STORAGE_ARTIFACT,
        storage_capability_id: MAIL_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: TELEGRAM_RUNTIME_ARTIFACT,
        storage_artifact_id: TELEGRAM_STORAGE_ARTIFACT,
        storage_capability_id: TELEGRAM_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: WHATSAPP_RUNTIME_ARTIFACT,
        storage_artifact_id: WHATSAPP_STORAGE_ARTIFACT,
        storage_capability_id: WHATSAPP_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: true,
    },
    ModulePlanV1 {
        runtime_artifact_id: ZULIP_RUNTIME_ARTIFACT,
        storage_artifact_id: ZULIP_STORAGE_ARTIFACT,
        storage_capability_id: ZULIP_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: PERSONS_RUNTIME_ARTIFACT,
        storage_artifact_id: PERSONS_STORAGE_ARTIFACT,
        storage_capability_id: PERSONS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
        storage_artifact_id: MAIL_PERSONS_SYNC_STORAGE_ARTIFACT,
        storage_capability_id: MAIL_PERSONS_SYNC_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
        storage_artifact_id: REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_ARTIFACT,
        storage_capability_id: REVIEW_PERSON_MATCH_CANDIDATE_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
        storage_artifact_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_ARTIFACT,
        storage_capability_id: REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: AI_INFERENCE_RUNTIME_ARTIFACT,
        storage_artifact_id: AI_INFERENCE_STORAGE_ARTIFACT,
        storage_capability_id: AI_INFERENCE_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: OLLAMA_AI_RUNTIME_ARTIFACT,
        storage_artifact_id: OLLAMA_AI_STORAGE_ARTIFACT,
        storage_capability_id: OLLAMA_AI_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
        storage_artifact_id: SPEECH_TO_TEXT_STORAGE_ARTIFACT,
        storage_capability_id: SPEECH_TO_TEXT_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: WHISPER_STT_RUNTIME_ARTIFACT,
        storage_artifact_id: WHISPER_STT_STORAGE_ARTIFACT,
        storage_capability_id: WHISPER_STT_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: CALENDAR_RUNTIME_ARTIFACT,
        storage_artifact_id: CALENDAR_STORAGE_ARTIFACT,
        storage_capability_id: CALENDAR_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ORGANIZATIONS_RUNTIME_ARTIFACT,
        storage_artifact_id: ORGANIZATIONS_STORAGE_ARTIFACT,
        storage_capability_id: ORGANIZATIONS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: DOCUMENTS_RUNTIME_ARTIFACT,
        storage_artifact_id: DOCUMENTS_STORAGE_ARTIFACT,
        storage_capability_id: DOCUMENTS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: RELATIONSHIPS_RUNTIME_ARTIFACT,
        storage_artifact_id: RELATIONSHIPS_STORAGE_ARTIFACT,
        storage_capability_id: RELATIONSHIPS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: PROJECTS_RUNTIME_ARTIFACT,
        storage_artifact_id: PROJECTS_STORAGE_ARTIFACT,
        storage_capability_id: PROJECTS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: REVIEW_OBLIGATION_CANDIDATE_RUNTIME_ARTIFACT,
        storage_artifact_id: REVIEW_OBLIGATION_CANDIDATE_STORAGE_ARTIFACT,
        storage_capability_id: REVIEW_OBLIGATION_CANDIDATE_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: OBLIGATIONS_RUNTIME_ARTIFACT,
        storage_artifact_id: OBLIGATIONS_STORAGE_ARTIFACT,
        storage_capability_id: OBLIGATIONS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
        storage_artifact_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_ARTIFACT,
        storage_capability_id: REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Workflow,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: DECISIONS_RUNTIME_ARTIFACT,
        storage_artifact_id: DECISIONS_STORAGE_ARTIFACT,
        storage_capability_id: DECISIONS_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Domain,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: IDENTITY_RESOLUTION_RUNTIME_ARTIFACT,
        storage_artifact_id: IDENTITY_RESOLUTION_STORAGE_ARTIFACT,
        storage_capability_id: IDENTITY_RESOLUTION_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: SEARCH_RUNTIME_ARTIFACT,
        storage_artifact_id: SEARCH_STORAGE_ARTIFACT,
        storage_capability_id: SEARCH_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: TIMELINE_RUNTIME_ARTIFACT,
        storage_artifact_id: TIMELINE_STORAGE_ARTIFACT,
        storage_capability_id: TIMELINE_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: GRAPH_RUNTIME_ARTIFACT,
        storage_artifact_id: GRAPH_STORAGE_ARTIFACT,
        storage_capability_id: GRAPH_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: MEMORY_RUNTIME_ARTIFACT,
        storage_artifact_id: MEMORY_STORAGE_ARTIFACT,
        storage_capability_id: MEMORY_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: CONSISTENCY_RUNTIME_ARTIFACT,
        storage_artifact_id: CONSISTENCY_STORAGE_ARTIFACT,
        storage_capability_id: CONSISTENCY_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: RISK_RUNTIME_ARTIFACT,
        storage_artifact_id: RISK_STORAGE_ARTIFACT,
        storage_capability_id: RISK_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Engine,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: ZOOM_RUNTIME_ARTIFACT,
        storage_artifact_id: ZOOM_STORAGE_ARTIFACT,
        storage_capability_id: ZOOM_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: TELEMOST_RUNTIME_ARTIFACT,
        storage_artifact_id: TELEMOST_STORAGE_ARTIFACT,
        storage_capability_id: TELEMOST_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
    ModulePlanV1 {
        runtime_artifact_id: OMNIROUTE_RUNTIME_ARTIFACT,
        storage_artifact_id: OMNIROUTE_STORAGE_ARTIFACT,
        storage_capability_id: OMNIROUTE_STORAGE_CAPABILITY,
        runtime_kind: ModuleRuntimeKindV1::Integration,
        request_host_bridge: false,
    },
];
const PRE_OBLIGATIONS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 27] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
    AI_INFERENCE_RUNTIME_ARTIFACT,
    OLLAMA_AI_RUNTIME_ARTIFACT,
    SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
    WHISPER_STT_RUNTIME_ARTIFACT,
    CALENDAR_RUNTIME_ARTIFACT,
    ORGANIZATIONS_RUNTIME_ARTIFACT,
    DOCUMENTS_RUNTIME_ARTIFACT,
    RELATIONSHIPS_RUNTIME_ARTIFACT,
    PROJECTS_RUNTIME_ARTIFACT,
];
const PRE_PROJECTS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 26] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
    AI_INFERENCE_RUNTIME_ARTIFACT,
    OLLAMA_AI_RUNTIME_ARTIFACT,
    SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
    WHISPER_STT_RUNTIME_ARTIFACT,
    CALENDAR_RUNTIME_ARTIFACT,
    ORGANIZATIONS_RUNTIME_ARTIFACT,
    DOCUMENTS_RUNTIME_ARTIFACT,
    RELATIONSHIPS_RUNTIME_ARTIFACT,
];
const PRE_RELATIONSHIPS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 25] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
    AI_INFERENCE_RUNTIME_ARTIFACT,
    OLLAMA_AI_RUNTIME_ARTIFACT,
    SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
    WHISPER_STT_RUNTIME_ARTIFACT,
    CALENDAR_RUNTIME_ARTIFACT,
    ORGANIZATIONS_RUNTIME_ARTIFACT,
    DOCUMENTS_RUNTIME_ARTIFACT,
];
const PRE_DOCUMENTS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 24] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
    AI_INFERENCE_RUNTIME_ARTIFACT,
    OLLAMA_AI_RUNTIME_ARTIFACT,
    SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
    WHISPER_STT_RUNTIME_ARTIFACT,
    CALENDAR_RUNTIME_ARTIFACT,
    ORGANIZATIONS_RUNTIME_ARTIFACT,
];
const PRE_ORGANIZATIONS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 23] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
    AI_INFERENCE_RUNTIME_ARTIFACT,
    OLLAMA_AI_RUNTIME_ARTIFACT,
    SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
    WHISPER_STT_RUNTIME_ARTIFACT,
    CALENDAR_RUNTIME_ARTIFACT,
];
const PRE_CALENDAR_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 22] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
    AI_INFERENCE_RUNTIME_ARTIFACT,
    OLLAMA_AI_RUNTIME_ARTIFACT,
    SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
    WHISPER_STT_RUNTIME_ARTIFACT,
];
const PRE_SPEECH_TO_TEXT_WHISPER_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 20] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
    AI_INFERENCE_RUNTIME_ARTIFACT,
    OLLAMA_AI_RUNTIME_ARTIFACT,
];
const PRE_AI_OLLAMA_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 18] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
    PERSONS_RUNTIME_ARTIFACT,
    MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
    REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
    REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
];
const PRE_EXPORT_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 6] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_DELIVERY_INTENT_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 7] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_BULK_ACTION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 8] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_DELAYED_DELIVERY_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 9] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_TEXT_EXTRACTION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 10] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_ATTACHMENT_PREVIEW_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 11] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 12] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_ATTACHMENT_TRANSLATION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 13] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];
const PRE_CONTACTS_SYNC_MODULE_PLAN_RUNTIME_ARTIFACTS_V3: [&str; 14] = [
    COMMUNICATIONS_RUNTIME_ARTIFACT,
    COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
    COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
    COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
    COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
    ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
    ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
    ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
    ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
    MAIL_RUNTIME_ARTIFACT,
    TELEGRAM_RUNTIME_ARTIFACT,
    WHATSAPP_RUNTIME_ARTIFACT,
    ZULIP_RUNTIME_ARTIFACT,
];

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("development assembly failed: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    validate_cli(&cli)?;
    let data_dir = cli
        .data_dir
        .canonicalize()
        .map_err(|_| "development data directory is unavailable".to_owned())?;
    let state_path = data_dir.join(STATE_FILE);
    match cli.command {
        Command::RuntimeDirectory => {
            println!("{}", runtime_directory(&data_dir)?.display());
            Ok(())
        }
        Command::ProvisionPlatform => {
            provision_platform(&data_dir)?;
            println!("development_platform=provisioned");
            Ok(())
        }
        Command::Status => {
            let state = read_state_if_present(&state_path)?;
            let status = development_assembly_status(
                state.as_ref(),
                &cli.distribution_id,
                cli.distribution_generation,
            )?;
            println!("development_assembly={status}");
            Ok(())
        }
        Command::Admit => {
            let client = client(&data_dir)?;
            let signer = FileOwnerSigner::open(&data_dir)?;
            let owner_session_id = client.open_owner_session(&signer).map_err(|error| {
                admission_error("development_assembly", "open_owner_session", error)
            })?;
            let reservation_path = data_dir.join(ENSEMBLE_RESERVATION_FILE);
            let existing_state = read_state_if_present(&state_path)?;
            let reconciliation = reconcile_plan(
                &client,
                &owner_session_id,
                &cli.distribution_id,
                cli.distribution_generation,
                existing_state.as_ref(),
                &state_path,
                &reservation_path,
            )?;
            if reconciliation.outcome != ReconciliationOutcomeV1::Current {
                write_state(&state_path, &reconciliation.state)?;
                remove_reservation(&reservation_path)?;
            }
            println!("development_assembly={}", reconciliation.outcome.as_str());
            Ok(())
        }
        Command::StartEnsemble => {
            let state = read_state(&state_path)?;
            if state.distribution_id != cli.distribution_id
                || state.distribution_generation != cli.distribution_generation
            {
                return Err("development assembly state does not match the release".to_owned());
            }
            let client = client(&data_dir)?;
            let signer = FileOwnerSigner::open(&data_dir)?;
            let owner_session_id = client.open_owner_session(&signer).map_err(|error| {
                admission_error("development_assembly", "open_owner_session", error)
            })?;
            start_ensemble(&client, &owner_session_id, &state)?;
            Ok(())
        }
        Command::RetireRegistration { registration_id } => {
            if registration_id.is_empty()
                || registration_id.len() > 128
                || !registration_id.is_ascii()
            {
                return Err("development registration id is invalid".to_owned());
            }
            let client = client(&data_dir)?;
            let signer = FileOwnerSigner::open(&data_dir)?;
            let owner_session_id = client.open_owner_session(&signer)?;
            let retired = client.transition_module_registration(
                &owner_session_id,
                &registration_id,
                "revoked",
            )?;
            println!("development_registration={}", retired.registration_state);
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReconciliationOutcomeV1 {
    Current,
    Admitted,
    Updated,
}

impl ReconciliationOutcomeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Admitted => "admitted",
            Self::Updated => "updated",
        }
    }
}

struct ReconciliationResultV1 {
    state: DevelopmentAssemblyStateV1,
    outcome: ReconciliationOutcomeV1,
}

fn development_assembly_status(
    state: Option<&DevelopmentAssemblyStateV1>,
    distribution_id: &str,
    distribution_generation: u64,
) -> Result<&'static str, String> {
    let Some(state) = state else {
        return Ok("missing");
    };
    if state.distribution_id != distribution_id {
        return Err("development assembly distribution identity changed".to_owned());
    }
    match distribution_generation.cmp(&state.distribution_generation) {
        std::cmp::Ordering::Equal => Ok("current"),
        std::cmp::Ordering::Greater => Ok("stale"),
        std::cmp::Ordering::Less => {
            Err("development assembly release rollback is not automatic".to_owned())
        }
    }
}

fn start_ensemble(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    state: &DevelopmentAssemblyStateV1,
) -> Result<(), String> {
    if state.modules.len() != MODULE_PLAN.len() {
        return Err("development assembly module state is incomplete".to_owned());
    }
    for (plan, module) in MODULE_PLAN.iter().zip(&state.modules) {
        if module.runtime_artifact_id != plan.runtime_artifact_id
            || module.storage_capability_id != plan.storage_capability_id
        {
            return Err("development assembly module state does not match the plan".to_owned());
        }
        if requires_real_provider_evidence(plan.runtime_artifact_id) {
            println!(
                "{}_runtime=blocked_provider_evidence",
                plan.runtime_artifact_id
            );
            continue;
        }
        match plan.runtime_kind {
            ModuleRuntimeKindV1::Domain => {
                client.start_reserved_domain_runtime(
                    owner_session_id,
                    &module.registration_id,
                    &module.storage_capability_id,
                )?;
            }
            ModuleRuntimeKindV1::Engine => {
                client.start_reserved_engine_runtime(
                    owner_session_id,
                    &module.registration_id,
                    &module.storage_capability_id,
                )?;
            }
            ModuleRuntimeKindV1::Workflow => {
                let started = client.start_reserved_workflow_runtime(
                    owner_session_id,
                    &module.registration_id,
                    &module.storage_capability_id,
                )?;
                println!(
                    "{}_runtime={}",
                    plan.runtime_artifact_id, started.launch_state
                );
                continue;
            }
            ModuleRuntimeKindV1::Integration => {
                let started = client.start_reserved_integration_runtime(
                    owner_session_id,
                    &module.registration_id,
                    &module.storage_capability_id,
                    "",
                    plan.request_host_bridge,
                )?;
                println!(
                    "{}_runtime={}",
                    plan.runtime_artifact_id, started.launch_state
                );
                continue;
            }
        }
        println!("{}_runtime=accepted", plan.runtime_artifact_id);
    }
    Ok(())
}

fn requires_real_provider_evidence(runtime_artifact_id: &str) -> bool {
    matches!(
        runtime_artifact_id,
        ZOOM_RUNTIME_ARTIFACT | TELEMOST_RUNTIME_ARTIFACT | OMNIROUTE_RUNTIME_ARTIFACT
    )
}

fn validate_cli(cli: &Cli) -> Result<(), String> {
    if !cli.data_dir.is_absolute()
        || cli.distribution_id.is_empty()
        || cli.distribution_id.len() > 128
        || !cli.distribution_id.is_ascii()
        || cli.distribution_generation == 0
    {
        return Err("development assembly arguments are invalid".to_owned());
    }
    Ok(())
}

fn client(data_dir: &Path) -> Result<OwnerControlClientV1, String> {
    Ok(OwnerControlClientV1::new(&runtime_directory(data_dir)?))
}

fn runtime_directory(data_dir: &Path) -> Result<PathBuf, String> {
    let directories = directories::ProjectDirs::from("dev", "makosh", "makosh")
        .ok_or_else(|| "OS-standard local runtime directory is unavailable".to_owned())?;
    let instance_key = Sha256::digest(data_dir.as_os_str().as_encoded_bytes())
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(directories.cache_dir().join("runtime").join(instance_key))
}

fn provision_platform(data_dir: &Path) -> Result<(), String> {
    let credential_directory = data_dir.join("developer-platform-credentials");
    ensure_private_directory(&credential_directory)?;
    let runtime_directory = runtime_directory(data_dir)?;
    ensure_private_directory(&runtime_directory)?;
    let pgbouncer_directory = runtime_directory.join("storage").join("pgbouncer");
    let pgbouncer_auth_directory = pgbouncer_directory.join("auth");
    ensure_private_directory(&pgbouncer_directory)?;
    ensure_private_directory(&pgbouncer_auth_directory)?;
    write_private_if_absent(&pgbouncer_directory.join("databases.ini"), b"[databases]\n")?;

    for name in [
        "postgres-admin-password",
        "pgbouncer-admin-password",
        "nats-event-hub-password",
    ] {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes)
            .map_err(|_| "development platform credentials are unavailable".to_owned())?;
        write_private_if_absent(&credential_directory.join(name), hex(&bytes).as_bytes())?;
    }

    let seed_path = credential_directory.join("nats-account-signer-seed");
    let public_path = credential_directory.join("nats-account-public-key");
    match (
        std::fs::symlink_metadata(&seed_path),
        std::fs::symlink_metadata(&public_path),
    ) {
        (Err(seed_error), Err(public_error))
            if seed_error.kind() == std::io::ErrorKind::NotFound
                && public_error.kind() == std::io::ErrorKind::NotFound =>
        {
            let signer = nats_jwt::KeyPair::new_account();
            let seed = signer
                .seed()
                .map_err(|_| "development NATS signer is unavailable".to_owned())?;
            write_private_if_absent(&seed_path, seed.as_bytes())?;
            write_private_if_absent(&public_path, signer.public_key().as_bytes())?;
        }
        (Ok(_), Ok(_)) => {
            let seed = read_private_string(&seed_path)?;
            let expected_public = nats_jwt::KeyPair::from_seed(&seed)
                .map_err(|_| "development NATS signer is invalid".to_owned())?
                .public_key();
            if read_private_string(&public_path)? != expected_public {
                return Err("development NATS signer files do not match".to_owned());
            }
        }
        _ => return Err("development NATS signer state is partial".to_owned()),
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && metadata.is_dir()
                && metadata.permissions().mode() & 0o077 == 0 =>
        {
            Ok(())
        }
        Ok(_) => Err("development platform directory is invalid".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => std::fs::create_dir_all(path)
            .and_then(|()| std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)))
            .map_err(|_| "development platform directory is unavailable".to_owned()),
        Err(_) => Err("development platform directory is unavailable".to_owned()),
    }
}

fn write_private_if_absent(path: &Path, bytes: &[u8]) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && metadata.is_file()
                && metadata.permissions().mode() & 0o077 == 0
                && metadata.len() > 0 =>
        {
            Ok(())
        }
        Ok(_) => Err("development platform file is invalid".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .map_err(|_| "development platform file is unavailable".to_owned())?;
            file.write_all(bytes)
                .and_then(|()| file.sync_all())
                .map_err(|_| "development platform file is unavailable".to_owned())
        }
        Err(_) => Err("development platform file is unavailable".to_owned()),
    }
}

fn read_private_string(path: &Path) -> Result<String, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "development platform file is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() == 0
        || metadata.len() > 4_096
    {
        return Err("development platform file is invalid".to_owned());
    }
    std::fs::read_to_string(path)
        .map(|value| value.trim().to_owned())
        .map_err(|_| "development platform file is unavailable".to_owned())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn reconcile_plan(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    distribution_id: &str,
    distribution_generation: u64,
    existing_state: Option<&DevelopmentAssemblyStateV1>,
    state_path: &Path,
    reservation_path: &Path,
) -> Result<ReconciliationResultV1, String> {
    if let Some(reservation) = read_reservation_if_present(reservation_path)? {
        let reservation_release =
            validate_reservation_release(&reservation, distribution_id, distribution_generation)?;
        if existing_state.is_some_and(|state| {
            state.distribution_id != reservation.distribution_id
                || state.distribution_generation > reservation.distribution_generation
        }) {
            return Err("development ensemble reservation is stale".to_owned());
        }
        let state = finish_ensemble_bindings(client, owner_session_id, reservation)?;
        if reservation_release == ReservationReleaseV1::Predecessor {
            write_state(state_path, &state)?;
            remove_reservation(reservation_path)?;
            return Ok(ReconciliationResultV1 {
                state: refresh_plan(
                    client,
                    owner_session_id,
                    distribution_id,
                    distribution_generation,
                    &state,
                    reservation_path,
                )?,
                outcome: ReconciliationOutcomeV1::Updated,
            });
        }
        return Ok(ReconciliationResultV1 {
            state,
            outcome: if existing_state.is_some() {
                ReconciliationOutcomeV1::Updated
            } else {
                ReconciliationOutcomeV1::Admitted
            },
        });
    }
    if let Some(state) = existing_state {
        development_assembly_status(Some(state), distribution_id, distribution_generation)?;
        if state.distribution_generation == distribution_generation {
            validate_state_plan(state)?;
            return Ok(ReconciliationResultV1 {
                state: state.clone(),
                outcome: ReconciliationOutcomeV1::Current,
            });
        }
        return Ok(ReconciliationResultV1 {
            state: refresh_plan(
                client,
                owner_session_id,
                distribution_id,
                distribution_generation,
                state,
                reservation_path,
            )?,
            outcome: ReconciliationOutcomeV1::Updated,
        });
    }

    let mut modules = Vec::with_capacity(MODULE_PLAN.len());
    for module in MODULE_PLAN {
        modules.push(reserve_new_module(
            client,
            owner_session_id,
            distribution_id,
            distribution_generation,
            &module,
        )?);
    }
    let reservation = EnsembleReservationV2 {
        distribution_id: distribution_id.to_owned(),
        distribution_generation,
        modules,
    };
    write_reservation(reservation_path, &reservation)?;
    Ok(ReconciliationResultV1 {
        state: finish_ensemble_bindings(client, owner_session_id, reservation)?,
        outcome: ReconciliationOutcomeV1::Admitted,
    })
}

fn reserve_new_module(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    distribution_id: &str,
    distribution_generation: u64,
    module: &ModulePlanV1,
) -> Result<ModuleReservationV1, String> {
    let proposal = client
        .propose_bundled_managed_artifact(
            owner_session_id,
            module.runtime_artifact_id,
            distribution_id,
            distribution_generation,
            operation_id(module.runtime_artifact_id),
        )
        .map_err(|error| admission_error(module.runtime_artifact_id, "propose", error))?;
    let status = client
        .module_registration_status(&proposal.registration_id)
        .map_err(|error| admission_error(module.runtime_artifact_id, "status", error))?;
    match status.registration_state.as_str() {
        "pending" => {
            client
                .approve_module_registration(
                    owner_session_id,
                    &proposal.registration_id,
                    proposal.requested_capability_ids.clone(),
                )
                .map_err(|error| admission_error(module.runtime_artifact_id, "approve", error))?;
        }
        "approved"
            if usize::try_from(status.effective_capability_count).ok()
                == Some(proposal.requested_capability_ids.len()) => {}
        _ => return Err("development module admission state is invalid".to_owned()),
    }
    client
        .bind_bundled_managed_release(
            owner_session_id,
            &proposal.registration_id,
            module.runtime_artifact_id,
        )
        .map_err(|error| admission_error(module.runtime_artifact_id, "bind_release", error))?;
    let storage = client
        .admit_bundled_storage_artifact(
            owner_session_id,
            module.storage_artifact_id,
            distribution_id,
            distribution_generation,
        )
        .map_err(|error| admission_error(module.runtime_artifact_id, "admit_storage", error))?;
    let storage_capability_id = exact_requested_capability(
        proposal.requested_capability_ids.iter().map(String::as_str),
        module.storage_capability_id,
    )
    .map_err(|error| admission_error(module.runtime_artifact_id, "select_storage", error))?;
    let reservation = client
        .reserve_bundled_managed_runtime(owner_session_id, &proposal.registration_id)
        .map_err(|error| admission_error(module.runtime_artifact_id, "reserve_runtime", error))?;
    Ok(ModuleReservationV1 {
        runtime_artifact_id: module.runtime_artifact_id.to_owned(),
        registration_id: proposal.registration_id,
        storage_capability_id,
        runtime_instance_id: reservation.runtime_instance_id,
        runtime_generation: reservation.runtime_generation,
        role_epoch: 1,
        credential_lease_revision: 1,
        storage_bundle_revision: storage.storage_bundle_revision,
        storage_bundle_digest: storage.storage_bundle_digest.try_into().map_err(|_| {
            admission_error(
                module.runtime_artifact_id,
                "admit_storage",
                "Storage bundle digest is invalid".to_owned(),
            )
        })?,
    })
}

fn refresh_plan(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    distribution_id: &str,
    distribution_generation: u64,
    current: &DevelopmentAssemblyStateV1,
    reservation_path: &Path,
) -> Result<DevelopmentAssemblyStateV1, String> {
    validate_refreshable_state_plan(current)?;
    if current.distribution_id != distribution_id
        || distribution_generation <= current.distribution_generation
    {
        return Err("development assembly release update is invalid".to_owned());
    }

    let mut modules = Vec::with_capacity(MODULE_PLAN.len());
    for plan in &MODULE_PLAN {
        let Some(previous) = current
            .modules
            .iter()
            .find(|module| module.runtime_artifact_id == plan.runtime_artifact_id)
        else {
            modules.push(reserve_new_module(
                client,
                owner_session_id,
                distribution_id,
                distribution_generation,
                plan,
            )?);
            continue;
        };
        let live_storage = client
            .managed_storage_binding_status(
                owner_session_id,
                &previous.registration_id,
                &previous.storage_capability_id,
            )
            .map_err(|error| {
                admission_error(plan.runtime_artifact_id, "inspect_storage_binding", error)
            })?;
        let (role_epoch, credential_lease_revision) = refresh_storage_successor_fences(
            previous,
            live_storage.binding_revision,
            live_storage.role_epoch,
            live_storage.credential_lease_revision,
        )
        .map_err(|error| {
            admission_error(plan.runtime_artifact_id, "inspect_storage_binding", error)
        })?;
        match live_storage.binding_state.as_str() {
            "active" | "revoking" => complete_storage_binding_revocation(|| {
                client
                    .begin_managed_storage_binding_revocation(
                        owner_session_id,
                        &previous.registration_id,
                        &previous.storage_capability_id,
                        live_storage.binding_revision,
                    )
                    .map(|_| ())
            })
            .map_err(|error| {
                admission_error(plan.runtime_artifact_id, "revoke_storage_binding", error)
            })?,
            _ => {
                return Err(admission_error(
                    plan.runtime_artifact_id,
                    "inspect_storage_binding",
                    "Storage binding state is invalid".to_owned(),
                ));
            }
        }
        client
            .upgrade_bundled_managed_registration(
                owner_session_id,
                &previous.registration_id,
                plan.runtime_artifact_id,
                distribution_id,
                distribution_generation,
            )
            .map_err(|error| {
                admission_error(plan.runtime_artifact_id, "upgrade_registration", error)
            })?;
        client
            .bind_bundled_managed_release(
                owner_session_id,
                &previous.registration_id,
                plan.runtime_artifact_id,
            )
            .map_err(|error| admission_error(plan.runtime_artifact_id, "bind_release", error))?;
        let storage = client
            .admit_bundled_storage_artifact(
                owner_session_id,
                plan.storage_artifact_id,
                distribution_id,
                distribution_generation,
            )
            .map_err(|error| admission_error(plan.runtime_artifact_id, "admit_storage", error))?;
        let reservation = client
            .reserve_bundled_managed_runtime(owner_session_id, &previous.registration_id)
            .map_err(|error| admission_error(plan.runtime_artifact_id, "reserve_runtime", error))?;
        modules.push(ModuleReservationV1 {
            runtime_artifact_id: previous.runtime_artifact_id.clone(),
            registration_id: previous.registration_id.clone(),
            storage_capability_id: previous.storage_capability_id.clone(),
            runtime_instance_id: reservation.runtime_instance_id,
            runtime_generation: reservation.runtime_generation,
            role_epoch,
            credential_lease_revision,
            storage_bundle_revision: storage.storage_bundle_revision,
            storage_bundle_digest: storage.storage_bundle_digest.try_into().map_err(|_| {
                admission_error(
                    plan.runtime_artifact_id,
                    "admit_storage",
                    "Storage bundle digest is invalid".to_owned(),
                )
            })?,
        });
    }
    let reservation = EnsembleReservationV2 {
        distribution_id: distribution_id.to_owned(),
        distribution_generation,
        modules,
    };
    write_reservation(reservation_path, &reservation)?;
    finish_ensemble_bindings(client, owner_session_id, reservation)
}

fn complete_storage_binding_revocation(
    mut revoke: impl FnMut() -> Result<(), String>,
) -> Result<(), String> {
    match revoke() {
        Ok(()) => Ok(()),
        Err(_) => revoke(),
    }
}

fn refresh_storage_successor_fences(
    previous: &ModuleAssemblyStateV1,
    binding_revision: u64,
    role_epoch: u64,
    credential_lease_revision: u64,
) -> Result<(u64, u64), String> {
    if binding_revision < previous.storage_binding_revision
        || role_epoch < previous.role_epoch
        || credential_lease_revision < previous.credential_lease_revision
    {
        return Err("Live Storage binding regressed behind the assembly checkpoint".to_owned());
    }
    successor_fences(role_epoch, credential_lease_revision)
}

fn successor_fences(role_epoch: u64, credential_lease_revision: u64) -> Result<(u64, u64), String> {
    Ok((
        role_epoch
            .checked_add(1)
            .ok_or_else(|| "development assembly role epoch overflowed".to_owned())?,
        credential_lease_revision.checked_add(1).ok_or_else(|| {
            "development assembly credential lease revision overflowed".to_owned()
        })?,
    ))
}

fn validate_state_plan(state: &DevelopmentAssemblyStateV1) -> Result<(), String> {
    if !state_matches_runtime_artifact_plan(
        state,
        &MODULE_PLAN
            .iter()
            .map(|plan| plan.runtime_artifact_id)
            .collect::<Vec<_>>(),
    ) {
        return Err("development assembly module state does not match the plan".to_owned());
    }
    Ok(())
}

fn validate_refreshable_state_plan(state: &DevelopmentAssemblyStateV1) -> Result<(), String> {
    if validate_state_plan(state).is_ok()
        || state_matches_pre_zoom_telemost_omniroute_plan(state)
        || state_matches_pre_memory_consistency_risk_plan(state)
        || state_matches_pre_search_timeline_graph_plan(state)
        || state_matches_pre_identity_resolution_plan(state)
        || state_matches_pre_decisions_plan(state)
        || state_matches_runtime_artifact_plan(state, &PRE_EXPORT_MODULE_PLAN_RUNTIME_ARTIFACTS_V3)
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_DELIVERY_INTENT_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_BULK_ACTION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_DELAYED_DELIVERY_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_TEXT_EXTRACTION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_ATTACHMENT_PREVIEW_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_ATTACHMENT_TRANSLATION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_CONTACTS_SYNC_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_AI_OLLAMA_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_SPEECH_TO_TEXT_WHISPER_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_CALENDAR_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_ORGANIZATIONS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_DOCUMENTS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_RELATIONSHIPS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_PROJECTS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
        || state_matches_runtime_artifact_plan(
            state,
            &PRE_OBLIGATIONS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3,
        )
    {
        return Ok(());
    }
    Err("development assembly module state does not match a refreshable plan".to_owned())
}

fn state_matches_pre_zoom_telemost_omniroute_plan(state: &DevelopmentAssemblyStateV1) -> bool {
    let runtime_artifact_ids = MODULE_PLAN[..38]
        .iter()
        .map(|plan| plan.runtime_artifact_id)
        .collect::<Vec<_>>();
    state_matches_runtime_artifact_plan(state, &runtime_artifact_ids)
}

fn state_matches_pre_memory_consistency_risk_plan(state: &DevelopmentAssemblyStateV1) -> bool {
    let runtime_artifact_ids = MODULE_PLAN[..35]
        .iter()
        .map(|plan| plan.runtime_artifact_id)
        .collect::<Vec<_>>();
    state_matches_runtime_artifact_plan(state, &runtime_artifact_ids)
}

fn state_matches_pre_decisions_plan(state: &DevelopmentAssemblyStateV1) -> bool {
    let runtime_artifact_ids = MODULE_PLAN[..30]
        .iter()
        .map(|plan| plan.runtime_artifact_id)
        .collect::<Vec<_>>();
    state_matches_runtime_artifact_plan(state, &runtime_artifact_ids)
}

fn state_matches_pre_identity_resolution_plan(state: &DevelopmentAssemblyStateV1) -> bool {
    let runtime_artifact_ids = MODULE_PLAN[..31]
        .iter()
        .map(|plan| plan.runtime_artifact_id)
        .collect::<Vec<_>>();
    state_matches_runtime_artifact_plan(state, &runtime_artifact_ids)
}

fn state_matches_pre_search_timeline_graph_plan(state: &DevelopmentAssemblyStateV1) -> bool {
    let runtime_artifact_ids = MODULE_PLAN[..32]
        .iter()
        .map(|plan| plan.runtime_artifact_id)
        .collect::<Vec<_>>();
    state_matches_runtime_artifact_plan(state, &runtime_artifact_ids)
}

fn state_matches_runtime_artifact_plan(
    state: &DevelopmentAssemblyStateV1,
    runtime_artifact_ids: &[&str],
) -> bool {
    state.modules.len() == runtime_artifact_ids.len()
        && runtime_artifact_ids
            .iter()
            .zip(&state.modules)
            .all(|(runtime_artifact_id, module)| {
                module.runtime_artifact_id == *runtime_artifact_id
                    && MODULE_PLAN.iter().any(|plan| {
                        plan.runtime_artifact_id == *runtime_artifact_id
                            && module.storage_capability_id == plan.storage_capability_id
                    })
            })
}

fn admission_error(artifact_id: &str, phase: &str, error: String) -> String {
    format!("module={artifact_id} phase={phase}: {error}")
}

fn finish_ensemble_bindings(
    client: &OwnerControlClientV1,
    owner_session_id: &str,
    reservation: EnsembleReservationV2,
) -> Result<DevelopmentAssemblyStateV1, String> {
    if reservation.modules.len() != MODULE_PLAN.len() {
        return Err("development ensemble reservation is incomplete".to_owned());
    }
    let mut modules = Vec::with_capacity(MODULE_PLAN.len());
    for (plan, module) in MODULE_PLAN.iter().zip(reservation.modules) {
        if module.runtime_artifact_id != plan.runtime_artifact_id
            || module.storage_capability_id != plan.storage_capability_id
        {
            return Err("development ensemble reservation does not match the plan".to_owned());
        }
        let issued = client
            .issue_managed_storage_binding(
                owner_session_id,
                &module.registration_id,
                &module.storage_capability_id,
                &module.runtime_instance_id,
                module.runtime_generation,
                module.role_epoch,
                module.credential_lease_revision,
                module.storage_bundle_revision,
                module.storage_bundle_digest.to_vec(),
            )
            .map_err(|error| {
                admission_error(plan.runtime_artifact_id, "issue_storage_binding", error)
            })?;
        modules.push(ModuleAssemblyStateV1 {
            runtime_artifact_id: module.runtime_artifact_id,
            registration_id: module.registration_id,
            storage_capability_id: module.storage_capability_id,
            storage_binding_revision: issued.binding_revision,
            role_epoch: module.role_epoch,
            credential_lease_revision: module.credential_lease_revision,
        });
    }
    Ok(DevelopmentAssemblyStateV1 {
        distribution_id: reservation.distribution_id,
        distribution_generation: reservation.distribution_generation,
        modules,
    })
}

fn exact_requested_capability<'a>(
    capabilities: impl Iterator<Item = &'a str>,
    expected_capability_id: &str,
) -> Result<String, String> {
    let values = capabilities
        .filter(|capability| *capability == expected_capability_id)
        .collect::<Vec<_>>();
    match values.as_slice() {
        [capability] => Ok((*capability).to_owned()),
        _ => Err("module must request its exact Storage capability".to_owned()),
    }
}

fn operation_id(artifact_id: &str) -> [u8; 16] {
    let mut digest = Sha256::new();
    digest.update(b"makosh.local-development-assembly.proposal.v2");
    digest.update([0]);
    digest.update(artifact_id.as_bytes());
    digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has a fixed size")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DevelopmentAssemblyStateV1 {
    distribution_id: String,
    distribution_generation: u64,
    modules: Vec<ModuleAssemblyStateV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModuleAssemblyStateV1 {
    runtime_artifact_id: String,
    registration_id: String,
    storage_capability_id: String,
    storage_binding_revision: u64,
    role_epoch: u64,
    credential_lease_revision: u64,
}

struct EnsembleReservationV2 {
    distribution_id: String,
    distribution_generation: u64,
    modules: Vec<ModuleReservationV1>,
}

struct ModuleReservationV1 {
    runtime_artifact_id: String,
    registration_id: String,
    storage_capability_id: String,
    runtime_instance_id: String,
    runtime_generation: u64,
    role_epoch: u64,
    credential_lease_revision: u64,
    storage_bundle_revision: u64,
    storage_bundle_digest: [u8; 32],
}

const MAX_DEVELOPMENT_RESERVATION_BYTES: u64 = 64 * 1024;

fn write_reservation(path: &Path, reservation: &EnsembleReservationV2) -> Result<(), String> {
    if reservation.modules.len() != MODULE_PLAN.len() {
        return Err("development ensemble reservation is incomplete".to_owned());
    }
    let mut bytes = format!(
        "version=3\ndistribution_id={}\ndistribution_generation={}\nmodule_count={}\n",
        reservation.distribution_id,
        reservation.distribution_generation,
        reservation.modules.len(),
    );
    for (index, module) in reservation.modules.iter().enumerate() {
        bytes.push_str(&format!(
            "module.{index}.runtime_artifact_id={}\nmodule.{index}.registration_id={}\nmodule.{index}.storage_capability_id={}\nmodule.{index}.runtime_instance_id={}\nmodule.{index}.runtime_generation={}\nmodule.{index}.role_epoch={}\nmodule.{index}.credential_lease_revision={}\nmodule.{index}.storage_bundle_revision={}\nmodule.{index}.storage_bundle_digest={}\n",
            module.runtime_artifact_id,
            module.registration_id,
            module.storage_capability_id,
            module.runtime_instance_id,
            module.runtime_generation,
            module.role_epoch,
            module.credential_lease_revision,
            module.storage_bundle_revision,
            hex(&module.storage_bundle_digest),
        ));
    }
    if bytes.len() as u64 > MAX_DEVELOPMENT_RESERVATION_BYTES {
        return Err("development ensemble reservation is incomplete".to_owned());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "development ensemble reservation cannot be staged".to_owned())?;
    file.write_all(bytes.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|_| "development ensemble reservation cannot be staged".to_owned())
}

fn read_reservation_if_present(path: &Path) -> Result<Option<EnsembleReservationV2>, String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata)
            if !metadata.file_type().is_symlink()
                && metadata.is_file()
                && metadata.permissions().mode() & 0o077 == 0
                && metadata.len() <= MAX_DEVELOPMENT_RESERVATION_BYTES =>
        {
            read_reservation(path).map(Some)
        }
        Ok(_) => Err("development ensemble reservation is invalid".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("development ensemble reservation is unavailable".to_owned()),
    }
}

fn read_reservation(path: &Path) -> Result<EnsembleReservationV2, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| "development ensemble reservation is unavailable".to_owned())?;
    let fields = content
        .lines()
        .map(|line| {
            line.split_once('=')
                .ok_or_else(|| "development ensemble reservation is invalid".to_owned())
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
    let version = fields
        .get("version")
        .copied()
        .ok_or_else(|| "development ensemble reservation is invalid".to_owned())?;
    let fields_per_module = match version {
        "2" => 7,
        "3" => 9,
        _ => return Err("development ensemble reservation is invalid".to_owned()),
    };
    if parse_positive_field(&fields, "module_count")? as usize != MODULE_PLAN.len()
        || fields.len() != 4 + MODULE_PLAN.len() * fields_per_module
    {
        return Err("development ensemble reservation is invalid".to_owned());
    }
    let modules = MODULE_PLAN
        .iter()
        .enumerate()
        .map(|(index, plan)| {
            let field = |name: &str| format!("module.{index}.{name}");
            let runtime_artifact_id =
                reservation_required_field(&fields, &field("runtime_artifact_id"))?;
            if runtime_artifact_id != plan.runtime_artifact_id {
                return Err("development ensemble reservation is invalid".to_owned());
            }
            Ok(ModuleReservationV1 {
                runtime_artifact_id: runtime_artifact_id.to_owned(),
                registration_id: reservation_required_field(&fields, &field("registration_id"))?
                    .to_owned(),
                storage_capability_id: reservation_required_field(
                    &fields,
                    &field("storage_capability_id"),
                )?
                .to_owned(),
                runtime_instance_id: reservation_required_field(
                    &fields,
                    &field("runtime_instance_id"),
                )?
                .to_owned(),
                runtime_generation: parse_positive_field(&fields, &field("runtime_generation"))?,
                role_epoch: if version == "2" {
                    1
                } else {
                    parse_positive_field(&fields, &field("role_epoch"))?
                },
                credential_lease_revision: if version == "2" {
                    1
                } else {
                    parse_positive_field(&fields, &field("credential_lease_revision"))?
                },
                storage_bundle_revision: parse_positive_field(
                    &fields,
                    &field("storage_bundle_revision"),
                )?,
                storage_bundle_digest: decode_hex_32(reservation_required_field(
                    &fields,
                    &field("storage_bundle_digest"),
                )?)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(EnsembleReservationV2 {
        distribution_id: reservation_required_field(&fields, "distribution_id")?.to_owned(),
        distribution_generation: parse_positive_field(&fields, "distribution_generation")?,
        modules,
    })
}

fn reservation_required_field<'a>(
    fields: &'a std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<&'a str, String> {
    fields
        .get(name)
        .copied()
        .filter(|value| !value.is_empty() && value.len() <= 256 && value.is_ascii())
        .ok_or_else(|| "development ensemble reservation is invalid".to_owned())
}

fn parse_positive_field(
    fields: &std::collections::BTreeMap<&str, &str>,
    name: &str,
) -> Result<u64, String> {
    reservation_required_field(fields, name)?
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "development ensemble reservation is invalid".to_owned())
}

fn decode_hex_32(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("development ensemble reservation is invalid".to_owned());
    }
    let mut output = [0_u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "development ensemble reservation is invalid".to_owned())?;
    }
    Ok(output)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReservationReleaseV1 {
    Exact,
    Predecessor,
}

fn validate_reservation_release(
    reservation: &EnsembleReservationV2,
    distribution_id: &str,
    distribution_generation: u64,
) -> Result<ReservationReleaseV1, String> {
    if reservation.distribution_id != distribution_id
        || reservation.distribution_generation > distribution_generation
    {
        return Err("development ensemble reservation does not match the release".to_owned());
    }
    if reservation.distribution_generation == distribution_generation {
        Ok(ReservationReleaseV1::Exact)
    } else {
        Ok(ReservationReleaseV1::Predecessor)
    }
}

fn remove_reservation(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("development ensemble reservation cannot be removed".to_owned()),
    }
}

fn write_state(path: &Path, state: &DevelopmentAssemblyStateV1) -> Result<(), String> {
    if state.modules.len() != MODULE_PLAN.len() {
        return Err("development assembly state is incomplete".to_owned());
    }
    if let Ok(metadata) = std::fs::symlink_metadata(path)
        && (metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() > 16_384)
    {
        return Err("development assembly state cannot be replaced".to_owned());
    }
    let mut bytes = format!(
        "version=3\ndistribution_id={}\ndistribution_generation={}\nmodule_count={}\n",
        state.distribution_id,
        state.distribution_generation,
        state.modules.len(),
    );
    for (index, module) in state.modules.iter().enumerate() {
        bytes.push_str(&format!(
            "module.{index}.runtime_artifact_id={}\nmodule.{index}.registration_id={}\nmodule.{index}.storage_capability_id={}\nmodule.{index}.storage_binding_revision={}\nmodule.{index}.role_epoch={}\nmodule.{index}.credential_lease_revision={}\n",
            module.runtime_artifact_id,
            module.registration_id,
            module.storage_capability_id,
            module.storage_binding_revision,
            module.role_epoch,
            module.credential_lease_revision,
        ));
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|_| "development assembly state cannot be staged".to_owned())?;
    let result = file
        .write_all(bytes.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|_| "development assembly state cannot be staged".to_owned())
        .and_then(|()| {
            std::fs::rename(&temporary, path)
                .map_err(|_| "development assembly state cannot be committed".to_owned())
        });
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

fn read_state_if_present(path: &Path) -> Result<Option<DevelopmentAssemblyStateV1>, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => read_state(path).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("development assembly state is unavailable".to_owned()),
    }
}

fn read_state(path: &Path) -> Result<DevelopmentAssemblyStateV1, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "development assembly state is unavailable".to_owned())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > 16_384
    {
        return Err("development assembly state is invalid".to_owned());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|_| "development assembly state is unavailable".to_owned())?;
    let fields = content
        .lines()
        .map(|line| {
            line.split_once('=')
                .ok_or_else(|| "development assembly state is invalid".to_owned())
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
    let version = fields
        .get("version")
        .copied()
        .ok_or_else(|| "development assembly state is invalid".to_owned())?;
    let fields_per_module = match version {
        "2" => 3,
        "3" => 6,
        _ => return Err("development assembly state is invalid".to_owned()),
    };
    let module_count = required_field(&fields, "module_count")?
        .parse::<usize>()
        .map_err(|_| "development assembly state is invalid".to_owned())?;
    if ![
        MODULE_PLAN.len(),
        MODULE_PLAN.len() - 1,
        MODULE_PLAN.len() - 2,
        MODULE_PLAN.len() - 3,
        35,
        32,
        31,
        30,
        PRE_EXPORT_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_DELIVERY_INTENT_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_BULK_ACTION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_DELAYED_DELIVERY_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_TEXT_EXTRACTION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_ATTACHMENT_PREVIEW_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_ATTACHMENT_TRANSLATION_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_CONTACTS_SYNC_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_AI_OLLAMA_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_SPEECH_TO_TEXT_WHISPER_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_CALENDAR_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_ORGANIZATIONS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_DOCUMENTS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_RELATIONSHIPS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_PROJECTS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
        PRE_OBLIGATIONS_MODULE_PLAN_RUNTIME_ARTIFACTS_V3.len(),
    ]
    .contains(&module_count)
        || fields.len() != 4 + module_count * fields_per_module
    {
        return Err("development assembly state is invalid".to_owned());
    }
    let modules = (0..module_count)
        .map(|index| {
            let field = |name: &str| format!("module.{index}.{name}");
            let runtime_artifact_id = required_field(&fields, &field("runtime_artifact_id"))?;
            let storage_capability_id = required_field(&fields, &field("storage_capability_id"))?;
            if !MODULE_PLAN.iter().any(|plan| {
                runtime_artifact_id == plan.runtime_artifact_id
                    && storage_capability_id == plan.storage_capability_id
            }) {
                return Err("development assembly state is invalid".to_owned());
            }
            Ok(ModuleAssemblyStateV1 {
                runtime_artifact_id: runtime_artifact_id.to_owned(),
                registration_id: required_field(&fields, &field("registration_id"))?.to_owned(),
                storage_capability_id: storage_capability_id.to_owned(),
                storage_binding_revision: if version == "2" {
                    1
                } else {
                    parse_state_positive_field(&fields, &field("storage_binding_revision"))?
                },
                role_epoch: if version == "2" {
                    1
                } else {
                    parse_state_positive_field(&fields, &field("role_epoch"))?
                },
                credential_lease_revision: if version == "2" {
                    1
                } else {
                    parse_state_positive_field(&fields, &field("credential_lease_revision"))?
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let state = DevelopmentAssemblyStateV1 {
        distribution_id: required_field(&fields, "distribution_id")?.to_owned(),
        distribution_generation: required_field(&fields, "distribution_generation")?
            .parse()
            .map_err(|_| "development assembly state is invalid".to_owned())?,
        modules,
    };
    if validate_refreshable_state_plan(&state).is_err()
        || state.distribution_generation == 0
        || std::iter::once(state.distribution_id.as_str())
            .chain(state.modules.iter().flat_map(|module| {
                [
                    module.runtime_artifact_id.as_str(),
                    module.registration_id.as_str(),
                    module.storage_capability_id.as_str(),
                ]
            }))
            .any(|value| value.is_empty() || value.len() > 128 || !value.is_ascii())
    {
        return Err("development assembly state is invalid".to_owned());
    }
    Ok(state)
}

fn required_field<'a>(
    fields: &'a std::collections::BTreeMap<&str, &str>,
    key: &str,
) -> Result<&'a str, String> {
    fields
        .get(key)
        .copied()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "development assembly state is invalid".to_owned())
}

fn parse_state_positive_field(
    fields: &std::collections::BTreeMap<&str, &str>,
    key: &str,
) -> Result<u64, String> {
    required_field(fields, key)?
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "development assembly state is invalid".to_owned())
}

struct FileOwnerSigner(SigningKey);

impl FileOwnerSigner {
    fn open(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join(DEVICE_KEY_FILE);
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| "owner device signer is unavailable".to_owned())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() != 32
        {
            return Err("owner device signer is unavailable".to_owned());
        }
        let mut bytes = [0_u8; 32];
        File::open(path)
            .and_then(|mut file| file.read_exact(&mut bytes))
            .map_err(|_| "owner device signer is unavailable".to_owned())?;
        SigningKey::from_bytes((&bytes).into())
            .map(Self)
            .map_err(|_| "owner device signer is unavailable".to_owned())
    }
}

impl OwnerControlProofSignerV1 for FileOwnerSigner {
    fn sign_owner_control_proof(&self, message: &[u8]) -> Result<[u8; 64], String> {
        let signature: Signature = self.0.sign(message);
        Ok(signature.to_bytes().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_FILE_COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temporary_state_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "makosh-development-assembly-{label}-{}-{}",
            std::process::id(),
            TEST_FILE_COUNTER.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn fixture_state(distribution_generation: u64) -> DevelopmentAssemblyStateV1 {
        DevelopmentAssemblyStateV1 {
            distribution_id: "makosh-local-development".to_owned(),
            distribution_generation,
            modules: MODULE_PLAN
                .iter()
                .enumerate()
                .map(|(index, plan)| ModuleAssemblyStateV1 {
                    runtime_artifact_id: plan.runtime_artifact_id.to_owned(),
                    registration_id: format!("registration-{index}"),
                    storage_capability_id: plan.storage_capability_id.to_owned(),
                    storage_binding_revision: u64::try_from(index + 1).unwrap(),
                    role_epoch: 2,
                    credential_lease_revision: 3,
                })
                .collect(),
        }
    }

    fn fixture_reservation(distribution_generation: u64) -> EnsembleReservationV2 {
        EnsembleReservationV2 {
            distribution_id: "makosh-local-development".to_owned(),
            distribution_generation,
            modules: MODULE_PLAN
                .iter()
                .enumerate()
                .map(|(index, plan)| ModuleReservationV1 {
                    runtime_artifact_id: plan.runtime_artifact_id.to_owned(),
                    registration_id: format!("registration-{index}"),
                    storage_capability_id: plan.storage_capability_id.to_owned(),
                    runtime_instance_id: format!("runtime-instance-{index}"),
                    runtime_generation: 4,
                    role_epoch: 4,
                    credential_lease_revision: 4,
                    storage_bundle_revision: 1,
                    storage_bundle_digest: [u8::try_from(index).unwrap(); 32],
                })
                .collect(),
        }
    }

    fn fixture_state_before_zoom_telemost_omniroute(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != ZOOM_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != TELEMOST_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != OMNIROUTE_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_ai_ollama(distribution_generation: u64) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_calendar(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != AI_INFERENCE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != OLLAMA_AI_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != SPEECH_TO_TEXT_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != WHISPER_STT_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_speech_to_text_whisper(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_calendar(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != SPEECH_TO_TEXT_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != WHISPER_STT_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_decisions(distribution_generation: u64) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_zoom_telemost_omniroute(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != DECISIONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != IDENTITY_RESOLUTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != SEARCH_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != TIMELINE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != GRAPH_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MEMORY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != CONSISTENCY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != RISK_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_identity_resolution(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_zoom_telemost_omniroute(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != IDENTITY_RESOLUTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != SEARCH_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != TIMELINE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != GRAPH_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MEMORY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != CONSISTENCY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != RISK_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_search_timeline_graph(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_zoom_telemost_omniroute(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != SEARCH_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != TIMELINE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != GRAPH_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MEMORY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != CONSISTENCY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != RISK_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_memory_consistency_risk(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_zoom_telemost_omniroute(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != MEMORY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != CONSISTENCY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != RISK_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_calendar(distribution_generation: u64) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_relationships(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != CALENDAR_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ORGANIZATIONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != DOCUMENTS_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_organizations(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_relationships(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != ORGANIZATIONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != DOCUMENTS_RUNTIME_ARTIFACT
        });
        state
    }

    fn fixture_state_before_documents(distribution_generation: u64) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_relationships(distribution_generation);
        state
            .modules
            .retain(|module| module.runtime_artifact_id != DOCUMENTS_RUNTIME_ARTIFACT);
        state
    }

    fn fixture_state_before_relationships(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_projects(distribution_generation);
        state
            .modules
            .retain(|module| module.runtime_artifact_id != RELATIONSHIPS_RUNTIME_ARTIFACT);
        state
    }

    fn fixture_state_before_projects(distribution_generation: u64) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_obligations(distribution_generation);
        state
            .modules
            .retain(|module| module.runtime_artifact_id != PROJECTS_RUNTIME_ARTIFACT);
        state
    }

    fn fixture_state_before_obligations(
        distribution_generation: u64,
    ) -> DevelopmentAssemblyStateV1 {
        let mut state = fixture_state_before_decisions(distribution_generation);
        state.modules.retain(|module| {
            module.runtime_artifact_id != REVIEW_OBLIGATION_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != OBLIGATIONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        state
    }

    fn write_test_state(path: &Path, bytes: &[u8]) {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .unwrap();
        file.write_all(bytes).unwrap();
        file.sync_all().unwrap();
    }

    fn encode_state_v3(state: &DevelopmentAssemblyStateV1) -> Vec<u8> {
        let mut bytes = format!(
            "version=3\ndistribution_id={}\ndistribution_generation={}\nmodule_count={}\n",
            state.distribution_id,
            state.distribution_generation,
            state.modules.len(),
        );
        for (index, module) in state.modules.iter().enumerate() {
            bytes.push_str(&format!(
                "module.{index}.runtime_artifact_id={}\nmodule.{index}.registration_id={}\nmodule.{index}.storage_capability_id={}\nmodule.{index}.storage_binding_revision={}\nmodule.{index}.role_epoch={}\nmodule.{index}.credential_lease_revision={}\n",
                module.runtime_artifact_id,
                module.registration_id,
                module.storage_capability_id,
                module.storage_binding_revision,
                module.role_epoch,
                module.credential_lease_revision,
            ));
        }
        bytes.into_bytes()
    }

    #[test]
    fn development_plan_keeps_domains_workflows_engines_and_integrations_as_distinct_artifacts() {
        assert_eq!(MODULE_PLAN.len(), 41);
        assert_eq!(
            MODULE_PLAN
                .iter()
                .map(|module| module.runtime_artifact_id)
                .collect::<Vec<_>>(),
            vec![
                COMMUNICATIONS_RUNTIME_ARTIFACT,
                COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT,
                COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT,
                COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT,
                COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT,
                ATTACHMENT_SECURITY_RUNTIME_ARTIFACT,
                ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT,
                ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT,
                ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT,
                ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT,
                MAIL_RUNTIME_ARTIFACT,
                TELEGRAM_RUNTIME_ARTIFACT,
                WHATSAPP_RUNTIME_ARTIFACT,
                ZULIP_RUNTIME_ARTIFACT,
                PERSONS_RUNTIME_ARTIFACT,
                MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT,
                REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT,
                REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
                AI_INFERENCE_RUNTIME_ARTIFACT,
                OLLAMA_AI_RUNTIME_ARTIFACT,
                SPEECH_TO_TEXT_RUNTIME_ARTIFACT,
                WHISPER_STT_RUNTIME_ARTIFACT,
                CALENDAR_RUNTIME_ARTIFACT,
                ORGANIZATIONS_RUNTIME_ARTIFACT,
                DOCUMENTS_RUNTIME_ARTIFACT,
                RELATIONSHIPS_RUNTIME_ARTIFACT,
                PROJECTS_RUNTIME_ARTIFACT,
                REVIEW_OBLIGATION_CANDIDATE_RUNTIME_ARTIFACT,
                OBLIGATIONS_RUNTIME_ARTIFACT,
                REVIEWED_OBLIGATION_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT,
                DECISIONS_RUNTIME_ARTIFACT,
                IDENTITY_RESOLUTION_RUNTIME_ARTIFACT,
                SEARCH_RUNTIME_ARTIFACT,
                TIMELINE_RUNTIME_ARTIFACT,
                GRAPH_RUNTIME_ARTIFACT,
                MEMORY_RUNTIME_ARTIFACT,
                CONSISTENCY_RUNTIME_ARTIFACT,
                RISK_RUNTIME_ARTIFACT,
                ZOOM_RUNTIME_ARTIFACT,
                TELEMOST_RUNTIME_ARTIFACT,
                OMNIROUTE_RUNTIME_ARTIFACT,
            ],
        );
        assert!(matches!(
            MODULE_PLAN[1].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert!(matches!(
            MODULE_PLAN[2].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert!(matches!(
            MODULE_PLAN[3].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert!(matches!(
            MODULE_PLAN[4].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert_eq!(
            MODULE_PLAN[5].runtime_artifact_id,
            "attachment_security.runtime.v1",
        );
        assert_eq!(
            MODULE_PLAN[5].storage_artifact_id,
            "attachment_security.storage.v1",
        );
        assert!(matches!(
            MODULE_PLAN[6].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert_eq!(
            MODULE_PLAN[6].runtime_artifact_id,
            "attachment_text_extraction.runtime.v1",
        );
        assert!(matches!(
            MODULE_PLAN[7].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert_eq!(
            MODULE_PLAN[7].runtime_artifact_id,
            "attachment_preview.runtime.v1",
        );
        assert_eq!(
            MODULE_PLAN[7].storage_artifact_id,
            "attachment_preview.storage.v1",
        );
        assert!(matches!(
            MODULE_PLAN[8].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert_eq!(
            MODULE_PLAN[8].runtime_artifact_id,
            "attachment_preview_evidence_replay.runtime.v1",
        );
        assert!(matches!(
            MODULE_PLAN[9].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert_eq!(
            MODULE_PLAN[9].runtime_artifact_id,
            "attachment_translation.runtime.v1",
        );
        assert!(matches!(
            MODULE_PLAN[14].runtime_kind,
            ModuleRuntimeKindV1::Domain
        ));
        assert_eq!(MODULE_PLAN[14].runtime_artifact_id, "persons.runtime.v1");
        assert!(matches!(
            MODULE_PLAN[15].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert_eq!(
            MODULE_PLAN[15].runtime_artifact_id,
            "mail_persons_sync.runtime.v1",
        );
        assert!(matches!(
            MODULE_PLAN[16].runtime_kind,
            ModuleRuntimeKindV1::Domain
        ));
        assert_eq!(
            MODULE_PLAN[16].runtime_artifact_id,
            "review.person-match-candidate.runtime.v1",
        );
        assert!(matches!(
            MODULE_PLAN[17].runtime_kind,
            ModuleRuntimeKindV1::Workflow
        ));
        assert!(matches!(
            MODULE_PLAN[18].runtime_kind,
            ModuleRuntimeKindV1::Engine
        ));
        assert!(matches!(
            MODULE_PLAN[19].runtime_kind,
            ModuleRuntimeKindV1::Integration
        ));
        assert!(matches!(
            MODULE_PLAN[20].runtime_kind,
            ModuleRuntimeKindV1::Engine
        ));
        assert!(matches!(
            MODULE_PLAN[21].runtime_kind,
            ModuleRuntimeKindV1::Integration
        ));
        assert!(matches!(
            MODULE_PLAN[22].runtime_kind,
            ModuleRuntimeKindV1::Domain
        ));
        assert!(matches!(
            MODULE_PLAN[23].runtime_kind,
            ModuleRuntimeKindV1::Domain
        ));
        assert!(matches!(
            MODULE_PLAN[25].runtime_kind,
            ModuleRuntimeKindV1::Domain
        ));
        assert!(matches!(
            MODULE_PLAN[30].runtime_kind,
            ModuleRuntimeKindV1::Domain
        ));
        assert!(matches!(
            MODULE_PLAN[31].runtime_kind,
            ModuleRuntimeKindV1::Engine
        ));
        assert!(matches!(
            MODULE_PLAN[32].runtime_kind,
            ModuleRuntimeKindV1::Engine
        ));
        assert!(matches!(
            MODULE_PLAN[33].runtime_kind,
            ModuleRuntimeKindV1::Engine
        ));
        assert!(matches!(
            MODULE_PLAN[34].runtime_kind,
            ModuleRuntimeKindV1::Engine
        ));
    }

    #[test]
    fn development_start_does_not_launch_provider_placeholders_without_real_evidence() {
        let blocked = MODULE_PLAN
            .iter()
            .filter(|plan| requires_real_provider_evidence(plan.runtime_artifact_id))
            .map(|plan| plan.runtime_artifact_id)
            .collect::<Vec<_>>();

        assert_eq!(
            blocked,
            vec![
                ZOOM_RUNTIME_ARTIFACT,
                TELEMOST_RUNTIME_ARTIFACT,
                OMNIROUTE_RUNTIME_ARTIFACT,
            ]
        );
    }

    #[test]
    fn current_full_plan_reservation_is_bounded_and_resumable() {
        let path = temporary_state_path("full-plan-reservation");
        let reservation = fixture_reservation(4);

        write_reservation(&path, &reservation).expect("full reservation must be writable");
        let metadata = std::fs::metadata(&path).expect("reservation metadata");
        let restored = read_reservation_if_present(&path)
            .expect("full reservation must be readable")
            .expect("reservation must be present");
        std::fs::remove_file(path).expect("reservation fixture must be removable");

        assert!(metadata.len() > 16_384);
        assert!(metadata.len() <= MAX_DEVELOPMENT_RESERVATION_BYTES);
        assert_eq!(restored.distribution_generation, 4);
        assert_eq!(restored.modules.len(), MODULE_PLAN.len());
    }

    #[test]
    fn runtime_directory_keeps_control_socket_within_the_portable_macos_limit() {
        let runtime_socket = runtime_directory(Path::new("/private/tmp/makosh-development"))
            .unwrap()
            .join("runtime.sock");

        assert!(
            runtime_socket.as_os_str().as_encoded_bytes().len() <= 103,
            "development runtime socket path exceeds the portable macOS limit: {}",
            runtime_socket.display(),
        );
    }

    #[test]
    fn storage_capability_selection_is_exact() {
        assert_eq!(
            exact_requested_capability(
                ["mail.query.v1", "mail.storage.v1"].into_iter(),
                "mail.storage.v1",
            ),
            Ok("mail.storage.v1".to_owned()),
        );
        assert!(
            exact_requested_capability(["mail.query.v1"].into_iter(), "mail.storage.v1").is_err()
        );
        assert!(
            exact_requested_capability(
                ["mail.storage.v1", "mail.storage.v1"].into_iter(),
                "mail.storage.v1",
            )
            .is_err()
        );
    }

    #[test]
    fn proposal_operation_ids_are_stable_and_artifact_scoped() {
        assert_eq!(
            operation_id(COMMUNICATIONS_RUNTIME_ARTIFACT),
            operation_id(COMMUNICATIONS_RUNTIME_ARTIFACT),
        );
        assert_ne!(
            operation_id(COMMUNICATIONS_RUNTIME_ARTIFACT),
            operation_id(MAIL_RUNTIME_ARTIFACT),
        );
    }

    #[test]
    fn assembly_status_requires_the_same_monotonic_distribution() {
        let state = fixture_state(7);
        assert_eq!(
            development_assembly_status(Some(&state), "makosh-local-development", 7),
            Ok("current"),
        );
        assert_eq!(
            development_assembly_status(Some(&state), "makosh-local-development", 8),
            Ok("stale"),
        );
        assert!(development_assembly_status(Some(&state), "makosh-local-development", 6).is_err());
        assert!(development_assembly_status(Some(&state), "other-distribution", 8).is_err());
        assert_eq!(
            development_assembly_status(None, "makosh-local-development", 1),
            Ok("missing"),
        );
    }

    #[test]
    fn state_v3_round_trip_preserves_successor_fences() {
        let path = temporary_state_path("v3");
        let state = fixture_state(8);
        write_state(&path, &state).unwrap();
        assert_eq!(read_state(&path), Ok(state));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_export_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-export-v3");
        let mut legacy = fixture_state_before_ai_ollama(25);
        legacy.modules.retain(|module| {
            module.runtime_artifact_id != COMMUNICATIONS_EXPORT_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&legacy));

        let state = read_state(&path).unwrap();
        assert_eq!(state, legacy);
        assert!(validate_refreshable_state_plan(&state).is_ok());
        assert!(validate_state_plan(&state).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_delivery_intent_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-delivery-intent-v3");
        let mut state = fixture_state_before_ai_ollama(26);
        state.modules.retain(|module| {
            module.runtime_artifact_id != COMMUNICATION_DELIVERY_INTENT_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_bulk_action_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-bulk-action-v3");
        let mut state = fixture_state_before_ai_ollama(27);
        state.modules.retain(|module| {
            module.runtime_artifact_id != COMMUNICATION_BULK_ACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_delayed_delivery_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-delayed-delivery-v3");
        let mut state = fixture_state_before_ai_ollama(28);
        state.modules.retain(|module| {
            module.runtime_artifact_id != COMMUNICATION_DELAYED_DELIVERY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_text_extraction_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-text-extraction-v3");
        let mut state = fixture_state_before_ai_ollama(29);
        state.modules.retain(|module| {
            module.runtime_artifact_id != ATTACHMENT_TEXT_EXTRACTION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_attachment_preview_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-attachment-preview-v3");
        let mut state = fixture_state_before_ai_ollama(30);
        state.modules.retain(|module| {
            module.runtime_artifact_id != ATTACHMENT_PREVIEW_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_attachment_preview_evidence_replay_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-attachment-preview-evidence-replay-v3");
        let mut state = fixture_state_before_ai_ollama(31);
        state.modules.retain(|module| {
            module.runtime_artifact_id != ATTACHMENT_PREVIEW_EVIDENCE_REPLAY_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_attachment_translation_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-attachment-translation-v3");
        let mut state = fixture_state_before_ai_ollama(32);
        state.modules.retain(|module| {
            module.runtime_artifact_id != ATTACHMENT_TRANSLATION_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_contacts_sync_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-contacts-sync-v3");
        let mut state = fixture_state_before_ai_ollama(33);
        state.modules.retain(|module| {
            module.runtime_artifact_id != PERSONS_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != MAIL_PERSONS_SYNC_RUNTIME_ARTIFACT
                && module.runtime_artifact_id != REVIEW_PERSON_MATCH_CANDIDATE_RUNTIME_ARTIFACT
                && module.runtime_artifact_id
                    != REVIEWED_PERSON_MATCH_CANDIDATE_PROMOTION_RUNTIME_ARTIFACT
        });
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_ai_ollama_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-ai-ollama-v3");
        let state = fixture_state_before_ai_ollama(34);
        assert_eq!(state.modules.len(), 18);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_speech_to_text_whisper_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-speech-to-text-whisper-v3");
        let state = fixture_state_before_speech_to_text_whisper(35);
        assert_eq!(state.modules.len(), 20);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_calendar_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-calendar-v3");
        let state = fixture_state_before_calendar(36);
        assert_eq!(state.modules.len(), 22);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_organizations_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-organizations-v3");
        let state = fixture_state_before_organizations(37);
        assert_eq!(state.modules.len(), 23);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_decisions_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-decisions-v3");
        let state = fixture_state_before_decisions(42);
        assert_eq!(state.modules.len(), 30);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_identity_resolution_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-identity-resolution-v3");
        let state = fixture_state_before_identity_resolution(43);
        assert_eq!(state.modules.len(), 31);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_search_timeline_graph_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-search-timeline-graph-v3");
        let state = fixture_state_before_search_timeline_graph(44);
        assert_eq!(state.modules.len(), 32);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_memory_consistency_risk_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-memory-consistency-risk-v3");
        let state = fixture_state_before_memory_consistency_risk(45);
        assert_eq!(state.modules.len(), 35);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_zoom_telemost_omniroute_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-zoom-telemost-omniroute-v3");
        let state = fixture_state_before_zoom_telemost_omniroute(46);
        assert_eq!(state.modules.len(), 38);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_documents_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-documents-v3");
        let state = fixture_state_before_documents(38);
        assert_eq!(state.modules.len(), 24);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_relationships_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-relationships-v3");
        let state = fixture_state_before_relationships(39);
        assert_eq!(state.modules.len(), 25);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_projects_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-projects-v3");
        let state = fixture_state_before_projects(40);
        assert_eq!(state.modules.len(), 26);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pre_obligations_state_v3_is_refreshable_but_not_current() {
        let path = temporary_state_path("pre-obligations-v3");
        let state = fixture_state_before_obligations(41);
        assert_eq!(state.modules.len(), 27);
        write_test_state(&path, &encode_state_v3(&state));

        let restored = read_state(&path).unwrap();
        assert_eq!(restored, state);
        assert!(validate_refreshable_state_plan(&restored).is_ok());
        assert!(validate_state_plan(&restored).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn legacy_state_v2_migrates_the_only_implemented_initial_fences() {
        let path = temporary_state_path("v2");
        let mut bytes = format!(
            concat!(
                "version=2\n",
                "distribution_id=makosh-local-development\n",
                "distribution_generation=1\n",
                "module_count={}\n",
            ),
            MODULE_PLAN.len(),
        );
        for (index, plan) in MODULE_PLAN.iter().enumerate() {
            bytes.push_str(&format!(
                "module.{index}.runtime_artifact_id={}\nmodule.{index}.registration_id=registration-{index}\nmodule.{index}.storage_capability_id={}\n",
                plan.runtime_artifact_id, plan.storage_capability_id,
            ));
        }
        write_test_state(&path, bytes.as_bytes());

        let state = read_state(&path).unwrap();
        assert_eq!(state.distribution_generation, 1);
        assert!(state.modules.iter().all(|module| {
            module.storage_binding_revision == 1
                && module.role_epoch == 1
                && module.credential_lease_revision == 1
        }));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn successor_fences_advance_without_reusing_a_credential_revision() {
        assert_eq!(successor_fences(2, 3), Ok((3, 4)));
        assert!(successor_fences(u64::MAX, 1).is_err());
        assert!(successor_fences(1, u64::MAX).is_err());
    }

    #[test]
    fn refresh_uses_live_storage_fences_after_owner_settings_apply() {
        let previous = fixture_state(33).modules.remove(0);
        assert_eq!(
            refresh_storage_successor_fences(
                &previous,
                previous.storage_binding_revision + 2,
                previous.role_epoch + 2,
                previous.credential_lease_revision + 2,
            ),
            Ok((
                previous.role_epoch + 3,
                previous.credential_lease_revision + 3
            )),
        );
        assert!(
            refresh_storage_successor_fences(
                &previous,
                previous.storage_binding_revision - 1,
                previous.role_epoch,
                previous.credential_lease_revision,
            )
            .is_err()
        );
    }

    #[test]
    fn refresh_retries_a_revocation_after_kernel_fences_storage() {
        let mut attempts = 0;
        assert_eq!(
            complete_storage_binding_revocation(|| {
                attempts += 1;
                (attempts == 2)
                    .then_some(())
                    .ok_or_else(|| "revocation is incomplete".to_owned())
            }),
            Ok(()),
        );
        assert_eq!(attempts, 2);

        let mut denied_attempts = 0;
        assert!(
            complete_storage_binding_revocation(|| {
                denied_attempts += 1;
                Err("operation denied".to_owned())
            })
            .is_err()
        );
        assert_eq!(denied_attempts, 2);
    }

    #[test]
    fn predecessor_reservation_must_finish_before_the_requested_successor() {
        let reservation = EnsembleReservationV2 {
            distribution_id: "makosh-local-development".to_owned(),
            distribution_generation: 18,
            modules: Vec::new(),
        };
        assert_eq!(
            validate_reservation_release(&reservation, "makosh-local-development", 18,),
            Ok(ReservationReleaseV1::Exact)
        );
        assert_eq!(
            validate_reservation_release(&reservation, "makosh-local-development", 19,),
            Ok(ReservationReleaseV1::Predecessor)
        );
        assert!(
            validate_reservation_release(&reservation, "makosh-local-development", 17,).is_err()
        );
        assert!(validate_reservation_release(&reservation, "another-distribution", 19).is_err());
    }
}

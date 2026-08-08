use makosh_communications_api::accounts::ProviderAccountCommandPort;
use makosh_communications_api::accounts::ProviderSecretBindingCommandPort;
use makosh_communications_api::commands::ProviderCommandMirrorPort;
use std::sync::Arc;

use serde_json::{Value, json};
use sqlx::postgres::PgPool;

use super::{WhatsAppProviderRuntime, WhatsAppProviderRuntimeShape, WhatsappWebStore};
use crate::platform::communications::ProviderChannelMessageLookupPort;

pub(crate) fn build_runtime(
    pool: PgPool,
    provider_account_store: Arc<dyn ProviderAccountCommandPort>,
    provider_secret_binding_store: Arc<dyn ProviderSecretBindingCommandPort>,
    provider_channel_message_store: Arc<dyn ProviderChannelMessageLookupPort>,
    provider_command_mirror: Arc<dyn ProviderCommandMirrorPort>,
) -> Arc<dyn WhatsAppProviderRuntime> {
    Arc::new(WhatsappWebStore::new(
        pool,
        provider_account_store,
        provider_secret_binding_store,
        provider_channel_message_store,
        provider_command_mirror,
    ))
}

pub(super) fn web_companion_bridge_contract_health_check() -> Value {
    json!({
        "driver_id": "webview_companion_bridge",
        "readiness": "hidden_desktop_webview_runtime_with_metadata_only_dispatch",
        "public_availability": false,
        "runtime_kind": "webview_companion",
        "provider_shape": WhatsAppProviderRuntimeShape::WebCompanion.as_str(),
        "desktop_producer": {
            "artifact": "frontend/src-tauri/src/whatsapp_companion.rs",
            "commands": [
            "start_hidden_whatsapp_webview",
                "whatsapp_web_companion_manifest",
                "whatsapp_web_companion_relay_observation"
            ],
            "driver_id": "tauri_hidden_webview_companion",
            "window_label_prefix": "whatsapp-companion",
            "target_url": "https://web.whatsapp.com/",
            "owner_visible": false,
            "hidden_headless_mode": "required_tauri_webview_not_headless_browser",
            "tauri_ipc_available_to_companion_window": true,
            "tauri_ipc_scope": "allowlisted_runtime_event_relay_only",
            "event_extractor": "contract_injected_relay_dispatch_available",
            "public_availability": false
        },
        "event_extractor": {
            "state": "contract_injected_relay_dispatch_available",
            "artifact": "frontend/src-tauri/src/whatsapp_companion.rs",
            "capability_artifact": "frontend/src-tauri/capabilities/whatsapp-companion-relay.json",
            "initialization_script": "installed_on_hidden_companion_webview",
            "script_scope": "main_frame_only",
            "origin_guard": "https://web.whatsapp.com",
            "navigation_guard": "https://web.whatsapp.com_only",
            "relay_channel": "tauri_allowlisted_companion_runtime_bridge_dispatch",
            "runtime_bridge_dispatch": "runtime_events_bridge_wired_smoke_pending",
            "runtime_bridge_dispatch_path": "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
            "dispatch_payload": "NewWhatsappWebRuntimeEvent",
            "relay_command": "whatsapp_web_companion_relay_observation",
            "relay_command_policy": {
                "remote_capability_url": "https://web.whatsapp.com",
                "window_label_pattern": "whatsapp-companion-*",
                "caller_window_label_must_match_account": true,
                "metadata_sanitizer": "secret_and_private_content_key_drop",
                "backend_auth": "X-Макошь-Secret_from_tauri_process_env_only",
                "backend_target": "loopback_http_runtime_bridge_only",
                "typed_projection": "not_attempted_until_richer_typed_payload",
                "domain_mutation": "forbidden",
                "command_completion": "forbidden"
            },
            "allowed_observations": [
                "runtime_lifecycle_metadata",
                "sync_lifecycle_metadata",
                "message_identity_metadata",
                "receipt_metadata",
                "reaction_metadata",
                "dialog_metadata",
                "participant_metadata",
                "presence_metadata",
                "call_metadata",
                "status_metadata",
                "media_metadata_without_bytes"
            ],
            "forbidden_reads": [
                "cookies",
                "web_storage",
                "indexed_db",
                "browser_profile_secrets",
                "session_material",
                "message_bodies",
                "media_bytes"
            ],
            "next_gate": "manual_live_smoke_before_public_availability"
        },
        "owner_visibility": {
            "hidden_runtime_required": true,
            "hidden_headless_mode": "tauri_webview_only_not_external_headless_browser",
            "owner_controls_required": ["start", "stop", "revoke", "relink", "remove", "health", "command_audit"]
        },
        "session_storage": {
            "binding_store": "host_vault",
            "binding_purpose": "whatsapp_web_session_key",
            "postgres_policy": "metadata_bindings_only_no_session_cookie_or_local_profile_secret"
        },
        "event_sink": {
            "kind": "protected_runtime_bridge_routes",
            "raw_evidence_policy": "append_only_sanitized_metadata",
            "accepted_event_policy": "signal_hub_acceptance_before_projection",
            "runtime_to_domain_calls": "forbidden",
            "required_event_families": [
                "runtime_lifecycle",
                "sync_lifecycle",
                "messages",
                "message_updates",
                "message_deletes",
                "receipts",
                "reactions",
                "dialogs",
                "participants",
                "presence",
                "calls_metadata",
                "statuses",
                "status_views",
                "status_deletes",
                "media_metadata",
                "media_lifecycle",
                "unsupported_evidence"
            ],
            "endpoint_paths": [
                "/api/v1/integrations/whatsapp/runtime-bridge/messages",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
                "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
                "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
                "/api/v1/integrations/whatsapp/runtime-bridge/reactions",
                "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
                "/api/v1/integrations/whatsapp/runtime-bridge/participants",
                "/api/v1/integrations/whatsapp/runtime-bridge/presence",
                "/api/v1/integrations/whatsapp/runtime-bridge/calls",
                "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
                "/api/v1/integrations/whatsapp/runtime-bridge/status-views",
                "/api/v1/integrations/whatsapp/runtime-bridge/status-deletes",
                "/api/v1/integrations/whatsapp/runtime-bridge/media",
                "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
                "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
                "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle"
            ]
        },
        "command_channel": {
            "kind": "durable_outbox",
            "claim_path": "/api/v1/integrations/whatsapp/runtime-bridge/commands/claim",
            "failure_path": "/api/v1/integrations/whatsapp/runtime-bridge/commands/{command_id}/failed",
            "completion_rule": "provider_observed_event_reconciliation_required"
        },
        "redaction_policy": {
            "session_material": "excluded",
            "cookies": "excluded",
            "browser_profile_secrets": "excluded",
            "qr_pair_code_artifacts": "transient_memory_only",
            "message_bodies_in_health": "excluded",
            "media_bytes": "excluded"
        },
        "blockers": [
            "whatsapp_visible_runtime_required",
            "whatsapp_webview_runtime_panel_action_not_implemented",
            "manual_live_smoke_required"
        ]
    })
}

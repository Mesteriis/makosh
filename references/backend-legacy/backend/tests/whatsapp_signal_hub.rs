use std::fs;
use std::path::{Path, PathBuf};

use makosh_hub_backend::platform::events::bus::{sanitize_event_payload, whatsapp_event_types};
use serde_json::json;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend has repository parent")
        .to_path_buf()
}

#[test]
fn whatsapp_signal_hub_covers_webview_observation_event_families() {
    let root = repo_root();
    let signal_hub_whatsapp = read(root.join("backend/src/domains/signal_hub/whatsapp.rs"));
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let event_producers =
        provider_handler.clone() + &read_all_sources(root.join("backend/src/application"));

    for fixture in signal_hub_fixtures() {
        if let Some(raw_record_kind) = fixture.raw_record_kind {
            assert!(
                signal_hub_whatsapp.contains(&format!(
                    "\"{raw_record_kind}\" => \"{}\"",
                    fixture.signal_kind
                )),
                "Signal Hub raw WhatsApp mapping missing for {raw_record_kind} ({})",
                fixture.matrix_label
            );
        } else {
            assert!(
                signal_hub_whatsapp.contains("_ => \"message\""),
                "Signal Hub raw WhatsApp mapping must default unknown message-like records to message"
            );
        }

        assert!(
            signal_hub_whatsapp.contains("signal.raw.whatsapp.{event_kind}.observed"),
            "WhatsApp raw signals must use the Signal Hub raw observed event family"
        );
        assert!(
            event_producers.contains(fixture.realtime_event_constant),
            "WhatsApp event producers must emit realtime event constant {} ({})",
            fixture.realtime_event_constant,
            fixture.matrix_label
        );
    }
}

#[test]
fn whatsapp_runtime_lifecycle_event_fixtures_are_sanitized_and_complete() {
    let root = repo_root();
    let provider_handler = read(root.join("backend/src/app/provider_runtime_handlers/whatsapp.rs"));
    let event_producers =
        provider_handler + &read_all_sources(root.join("backend/src/application"));
    let event_bus = read(root.join("backend/src/platform/events/bus.rs"));

    for (required, handler_marker) in [
        (
            whatsapp_event_types::RUNTIME_STATUS_CHANGED,
            "RUNTIME_STATUS_CHANGED",
        ),
        (whatsapp_event_types::RUNTIME_EVENT, "RUNTIME_EVENT"),
        (
            whatsapp_event_types::SESSION_LINK_STATE_CHANGED,
            "SESSION_LINK_STATE_CHANGED",
        ),
        (whatsapp_event_types::SYNC_STARTED, "SYNC_STARTED"),
        (whatsapp_event_types::SYNC_PROGRESS, "SYNC_PROGRESS"),
        (whatsapp_event_types::SYNC_COMPLETED, "SYNC_COMPLETED"),
        (whatsapp_event_types::SYNC_FAILED, "SYNC_FAILED"),
        (
            whatsapp_event_types::COMMAND_STATUS_CHANGED,
            "COMMAND_STATUS_CHANGED",
        ),
        (
            whatsapp_event_types::COMMAND_RECONCILED,
            "COMMAND_RECONCILED",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_REQUESTED,
            "MEDIA_UPLOAD_REQUESTED",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_STARTED,
            "whatsapp.media.upload.started",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_PROGRESS,
            "whatsapp.media.upload.progress",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_COMPLETED,
            "whatsapp.media.upload.completed",
        ),
        (
            whatsapp_event_types::MEDIA_UPLOAD_FAILED,
            "MEDIA_UPLOAD_FAILED",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_REQUESTED,
            "MEDIA_DOWNLOAD_REQUESTED",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_STARTED,
            "whatsapp.media.download.started",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_PROGRESS,
            "whatsapp.media.download.progress",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_COMPLETED,
            "whatsapp.media.download.completed",
        ),
        (
            whatsapp_event_types::MEDIA_DOWNLOAD_FAILED,
            "MEDIA_DOWNLOAD_FAILED",
        ),
    ] {
        assert!(
            event_bus.contains(required),
            "WhatsApp event bus constants must expose {required}"
        );
        assert!(
            event_producers.contains(handler_marker),
            "WhatsApp provider handler/runtime bridge or worker must emit {required}"
        );
    }

    let sanitized = sanitize_event_payload(json!({
        "account_id": "wa-1",
        "session_key": "secret-session",
        "access_token": "secret-token",
        "password": "secret-password",
        "raw_body": "private body",
        "safe": "kept"
    }));

    assert_eq!(sanitized["safe"], json!("kept"));
    for secret_key in ["session_key", "access_token", "password", "raw_body"] {
        assert!(
            sanitized.get(secret_key).is_none(),
            "sanitized WhatsApp event payload must remove {secret_key}"
        );
    }
}

#[test]
fn whatsapp_retired_runtime_artifacts_are_absent() {
    let root = repo_root();
    let makefile = read(root.join("Makefile"));
    for path in [
        "scripts/whatsapp-business-cloud-edge-readiness.mjs",
        "scripts/whatsapp-live-smoke-collect-evidence.mjs",
        "scripts/whatsapp-live-smoke-evidence.mjs",
        "scripts/whatsapp-live-smoke-readiness.mjs",
        "scripts/whatsapp-native-md-sdk-gap-readiness.mjs",
        "docs/integrations/whatsapp/fixture-test-matrix.md",
        "docs/integrations/whatsapp/live-smoke-checklist.md",
    ] {
        assert!(
            !root.join(path).exists(),
            "retired WhatsApp artifact must stay deleted: {path}"
        );
    }
    for target in [
        "whatsapp-live-smoke-collect-evidence:",
        "whatsapp-business-cloud-edge-readiness:",
        "whatsapp-native-md-sdk-gap-readiness:",
    ] {
        assert!(
            !makefile.contains(target),
            "Makefile must not retain retired WhatsApp target {target}"
        );
    }
}

#[test]
fn whatsapp_hidden_webview_bridge_is_metadata_only_and_event_spine_bound() {
    let root = repo_root();
    let runtime = read(root.join("backend/src/integrations/whatsapp/runtime/web_companion.rs"));
    let tauri = read(root.join("frontend/src-tauri/src/whatsapp_companion.rs"));
    let bridge = read(root.join("frontend/src/integrations/whatsapp/api/whatsappCompanion.ts"));

    assert!(
        runtime.contains("hidden_desktop_webview_runtime")
            && runtime.contains("metadata_only")
            && runtime.contains("provider_observed_event_reconciliation_required")
            && !runtime.contains("whatsapp_native_md")
            && !runtime.contains("whatsapp_business_cloud"),
        "the backend runtime must expose only the hidden metadata-only WebView path"
    );
    assert!(
        tauri.contains("start_hidden_whatsapp_webview")
            && tauri.contains(".visible(false)")
            && !tauri.contains(".show()")
            && !tauri.contains(".set_focus()"),
        "the Tauri companion must stay hidden and must not steal focus"
    );
    assert!(
        bridge.contains("startHiddenWhatsappWebview")
            && bridge.contains("start_hidden_whatsapp_webview")
            && !bridge.contains("fetch("),
        "the frontend bridge must use the local Tauri command, not an HTTP bypass"
    );
}

struct SignalHubFixture {
    raw_record_kind: Option<&'static str>,
    signal_kind: &'static str,
    matrix_label: &'static str,
    realtime_event_constant: &'static str,
}

fn signal_hub_fixtures() -> Vec<SignalHubFixture> {
    vec![
        SignalHubFixture {
            raw_record_kind: None,
            signal_kind: "message",
            matrix_label: "whatsapp_message",
            realtime_event_constant: "MESSAGE_CREATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_message_update"),
            signal_kind: "message_update",
            matrix_label: "whatsapp_message_update",
            realtime_event_constant: "MESSAGE_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_message_delete"),
            signal_kind: "message_delete",
            matrix_label: "whatsapp_message_delete",
            realtime_event_constant: "MESSAGE_DELETED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_receipt"),
            signal_kind: "receipt",
            matrix_label: "whatsapp_receipt",
            realtime_event_constant: "RECEIPT_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_reaction"),
            signal_kind: "reaction",
            matrix_label: "whatsapp_reaction",
            realtime_event_constant: "REACTION_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_dialog"),
            signal_kind: "dialog",
            matrix_label: "whatsapp_dialog",
            realtime_event_constant: "DIALOG_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_participant"),
            signal_kind: "participant",
            matrix_label: "whatsapp_participant",
            realtime_event_constant: "PARTICIPANT_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_presence"),
            signal_kind: "presence",
            matrix_label: "whatsapp_presence",
            realtime_event_constant: "PRESENCE_CHANGED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_call"),
            signal_kind: "call_metadata",
            matrix_label: "whatsapp_call_metadata",
            realtime_event_constant: "CALL_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_status"),
            signal_kind: "status",
            matrix_label: "whatsapp_status",
            realtime_event_constant: "STATUS_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_status_view"),
            signal_kind: "status_view",
            matrix_label: "whatsapp_status_view",
            realtime_event_constant: "STATUS_UPDATED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_status_delete"),
            signal_kind: "status_delete",
            matrix_label: "whatsapp_status_delete",
            realtime_event_constant: "STATUS_DELETED",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_media"),
            signal_kind: "media",
            matrix_label: "whatsapp_media",
            realtime_event_constant: "whatsapp.media.download.completed",
        },
        SignalHubFixture {
            raw_record_kind: Some("whatsapp_web_runtime_event"),
            signal_kind: "runtime_event",
            matrix_label: "whatsapp_runtime_event",
            realtime_event_constant: "RUNTIME_EVENT",
        },
    ]
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn read_all_sources(root: PathBuf) -> String {
    let mut output = String::new();
    collect_sources(&root, &mut output);
    output
}

fn collect_sources(path: &Path, output: &mut String) {
    if path.is_file() {
        if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("rs")
        ) {
            output.push_str(&read(path.to_path_buf()));
            output.push('\n');
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_sources(&entry.path(), output);
    }
}

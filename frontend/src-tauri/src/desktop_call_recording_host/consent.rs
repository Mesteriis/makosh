use std::sync::mpsc;

use makosh_desktop_call_recording_api::wire::BeginDesktopCaptureCommandV1;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tauri::{AppHandle, Manager};

pub(super) trait ConsentAuthorityV1: Send + Sync + 'static {
    fn request(
        &self,
        app: &AppHandle,
        command: &BeginDesktopCaptureCommandV1,
        selected_input_label: &str,
    ) -> Result<bool, &'static str>;
}

#[derive(Clone, Copy)]
pub(super) struct NativeConsentAuthorityV1;

impl ConsentAuthorityV1 for NativeConsentAuthorityV1 {
    fn request(
        &self,
        app: &AppHandle,
        command: &BeginDesktopCaptureCommandV1,
        selected_input_label: &str,
    ) -> Result<bool, &'static str> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let app_for_dialog = app.clone();
        let description = consent_description(command, selected_input_label)?;
        app.run_on_main_thread(move || {
            let mut dialog = MessageDialog::new()
                .set_level(MessageLevel::Warning)
                .set_title("Record audio for call transcription?")
                .set_description(description)
                .set_buttons(MessageButtons::OkCancelCustom(
                    "Start recording".to_owned(),
                    "Cancel".to_owned(),
                ));
            if let Some(window) = app_for_dialog.get_webview_window("main") {
                dialog = dialog.set_parent(&window);
            }
            let accepted = match dialog.show() {
                MessageDialogResult::Ok => true,
                MessageDialogResult::Custom(value) => value == "Start recording",
                _ => false,
            };
            let _ = sender.send(accepted);
        })
        .map_err(|_| "consent_ui_unavailable")?;
        receiver.recv().map_err(|_| "consent_ui_unavailable")
    }
}

fn consent_description(
    command: &BeginDesktopCaptureCommandV1,
    selected_input_label: &str,
) -> Result<String, &'static str> {
    if command.call_evidence_id.len() != 16
        || command.call_evidence_revision == 0
        || command.maximum_duration_millis == 0
        || selected_input_label.trim().is_empty()
    {
        return Err("consent_contract_invalid");
    }
    let anchor = command.call_evidence_id[..6]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let maximum_minutes = command.maximum_duration_millis.div_ceil(60_000);
    Ok(format!(
        "Purpose: call transcription\nCall: {anchor} (revision {})\nAudio input: {}\nMaximum duration: {maximum_minutes} minutes\n\nRecording starts only after you press Start recording and macOS grants microphone access.",
        command.call_evidence_revision,
        selected_input_label.trim(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immutable_prompt_binds_call_input_purpose_and_duration() {
        let description = consent_description(
            &BeginDesktopCaptureCommandV1 {
                call_evidence_id: vec![0x12; 16],
                call_evidence_revision: 7,
                maximum_duration_millis: 61_000,
                ..Default::default()
            },
            "Built-in Microphone",
        )
        .expect("description");
        assert!(description.contains("Purpose: call transcription"));
        assert!(description.contains("121212121212 (revision 7)"));
        assert!(description.contains("Audio input: Built-in Microphone"));
        assert!(description.contains("Maximum duration: 2 minutes"));
    }
}

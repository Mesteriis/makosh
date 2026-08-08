mod capture;
mod consent;
mod transport;

use std::sync::Mutex;
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use capture::{NativeCaptureV1, SelectedInputV1};
use consent::{ConsentAuthorityV1, NativeConsentAuthorityV1};
use makosh_desktop_call_recording_api::{
    CANONICAL_AUDIO_FORMAT_V1, CONSENT_PURPOSE_V1,
    wire::{
        BeginDesktopCaptureCommandV1, DesktopCaptureCompletedV1, DesktopCaptureRejectedV1,
        DesktopCaptureStartedV1, DesktopRecordingHostCommandClaimV1, DesktopRecordingHostCommandV1,
        DesktopRecordingHostObservationV1, DesktopRecordingHostOperationV1,
        desktop_recording_host_command_v1::Command,
        desktop_recording_host_observation_v1::Observation,
        desktop_recording_host_operation_v1::Operation,
    },
};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State, ipc::CapabilityBuilder};
use transport::HostRouteClientV1;

const CLAIM_LEASE_SECONDS: u32 = 10;
const CLAIM_LIMIT: u32 = 4;
const CLAIM_INTERVAL: Duration = Duration::from_millis(250);
const OS_PERMISSION_REVISION_V1: u32 = 1;

#[derive(Default)]
pub(crate) struct DesktopCallRecordingHostStateV1 {
    worker: Mutex<Option<HostWorkerHandleV1>>,
    admission_watcher: Mutex<Option<HostWorkerHandleV1>>,
}

struct HostWorkerHandleV1 {
    stop: Sender<()>,
    join: Option<JoinHandle<()>>,
}

struct ActiveCaptureV1 {
    command_id: [u8; 16],
    claim_id: [u8; 16],
    challenge_id: [u8; 16],
    recording_evidence_id: [u8; 16],
    started_at_unix_ms: i64,
    capture: NativeCaptureV1,
}

pub(crate) fn watch_for_route_admission(
    app: AppHandle,
    state: State<'_, DesktopCallRecordingHostStateV1>,
) -> Result<(), String> {
    let mut watcher = state
        .admission_watcher
        .lock()
        .map_err(|_| "Desktop recording admission state is unavailable".to_owned())?;
    if watcher.is_some() {
        return Ok(());
    }
    let (stop, stop_receiver) = mpsc::channel();
    let join = std::thread::Builder::new()
        .name("desktop-call-recording-route-admission".to_owned())
        .spawn(move || {
            while stop_receiver.recv_timeout(CLAIM_INTERVAL).is_err() {
                if !HostRouteClientV1::admitted_route_exists(&app) {
                    continue;
                }
                let capability = CapabilityBuilder::new("desktop-call-recording-route-admitted")
                    .window("main")
                    .permission("allow-desktop-call-recording-host-connect")
                    .permission("allow-desktop-call-recording-host-disconnect");
                if app.add_capability(capability).is_ok() {
                    return;
                }
            }
        })
        .map_err(|_| "Desktop recording admission watcher is unavailable".to_owned())?;
    *watcher = Some(HostWorkerHandleV1 {
        stop,
        join: Some(join),
    });
    Ok(())
}

#[tauri::command]
pub(crate) async fn desktop_call_recording_host_connect(
    app: AppHandle,
    state: State<'_, DesktopCallRecordingHostStateV1>,
    registration_id: String,
) -> Result<(), String> {
    let route = HostRouteClientV1::load(&app, registration_id.trim())?;
    let mut worker = state
        .worker
        .lock()
        .map_err(|_| "Desktop recording host state is unavailable".to_owned())?;
    if worker.is_some() {
        return Ok(());
    }
    let (stop, stop_receiver) = mpsc::channel();
    let join = std::thread::Builder::new()
        .name("desktop-call-recording-host".to_owned())
        .spawn(move || run_worker(app, route, stop_receiver))
        .map_err(|_| "Desktop recording host is unavailable".to_owned())?;
    *worker = Some(HostWorkerHandleV1 {
        stop,
        join: Some(join),
    });
    Ok(())
}

#[tauri::command]
pub(crate) async fn desktop_call_recording_host_disconnect(
    state: State<'_, DesktopCallRecordingHostStateV1>,
) -> Result<(), String> {
    let worker = state
        .worker
        .lock()
        .map_err(|_| "Desktop recording host state is unavailable".to_owned())?
        .take();
    if let Some(mut worker) = worker {
        let _ = worker.stop.send(());
        if let Some(join) = worker.join.take() {
            join.join()
                .map_err(|_| "Desktop recording host stopped unexpectedly".to_owned())?;
        }
    }
    Ok(())
}

impl Drop for DesktopCallRecordingHostStateV1 {
    fn drop(&mut self) {
        for slot in [&mut self.worker, &mut self.admission_watcher] {
            if let Ok(worker) = slot.get_mut()
                && let Some(mut worker) = worker.take()
            {
                let _ = worker.stop.send(());
                if let Some(join) = worker.join.take() {
                    let _ = join.join();
                }
            }
        }
    }
}

fn run_worker(app: AppHandle, route: HostRouteClientV1, stop: mpsc::Receiver<()>) {
    let consent = NativeConsentAuthorityV1;
    let mut active = None;
    loop {
        if stop.try_recv().is_ok() {
            finish_active(&route, active.take(), None);
            return;
        }
        if active
            .as_ref()
            .is_some_and(|capture: &ActiveCaptureV1| capture.capture.reached_limit())
        {
            finish_active(&route, active.take(), None);
        } else if active
            .as_ref()
            .is_some_and(|capture: &ActiveCaptureV1| capture.capture.failed())
        {
            reject_active(&route, active.take(), "audio_capture_failed");
        }
        if let Ok((claim_id, commands)) = claim_commands(&route) {
            for command in commands {
                handle_command(&app, &route, &consent, claim_id, command, &mut active);
            }
        }
        std::thread::sleep(CLAIM_INTERVAL);
    }
}

fn claim_commands(
    route: &HostRouteClientV1,
) -> Result<([u8; 16], Vec<DesktopRecordingHostCommandV1>), String> {
    let claim_id = random_id()?;
    let lease = route.claim(DesktopRecordingHostOperationV1 {
        operation: Some(Operation::ClaimCommands(
            DesktopRecordingHostCommandClaimV1 {
                host_claim_id: claim_id.to_vec(),
                lease_seconds: CLAIM_LEASE_SECONDS,
                limit: CLAIM_LIMIT,
            },
        )),
    })?;
    Ok((claim_id, lease.commands))
}

fn handle_command(
    app: &AppHandle,
    route: &HostRouteClientV1,
    consent: &impl ConsentAuthorityV1,
    claim_id: [u8; 16],
    command: DesktopRecordingHostCommandV1,
    active: &mut Option<ActiveCaptureV1>,
) {
    let Ok(command_id) = id16(&command.command_id) else {
        return;
    };
    match command.command {
        Some(Command::BeginCapture(begin)) if active.is_none() => {
            let Ok(()) = validate_begin(&begin) else {
                reject_begin(route, command_id, claim_id, &begin, "host_contract_invalid");
                return;
            };
            if now_unix_ms().is_none_or(|now| now >= begin.expires_at_unix_ms) {
                reject_begin(
                    route,
                    command_id,
                    claim_id,
                    &begin,
                    "consent_challenge_expired",
                );
                return;
            }
            let Ok(input) = SelectedInputV1::system_default() else {
                reject_begin(
                    route,
                    command_id,
                    claim_id,
                    &begin,
                    "audio_input_unavailable",
                );
                return;
            };
            match consent.request(app, &begin, &input.label) {
                Ok(true) => {}
                Ok(false) => {
                    reject_begin(route, command_id, claim_id, &begin, "user_cancelled");
                    return;
                }
                Err(code) => {
                    reject_begin(route, command_id, claim_id, &begin, code);
                    return;
                }
            }
            let capture = match input.start(begin.maximum_duration_millis) {
                Ok(capture) => capture,
                Err(code) => {
                    reject_begin(route, command_id, claim_id, &begin, code);
                    return;
                }
            };
            let Some(started_at_unix_ms) = now_unix_ms() else {
                reject_begin(
                    route,
                    command_id,
                    claim_id,
                    &begin,
                    "host_clock_unavailable",
                );
                return;
            };
            let Ok(challenge_id) = id16(&begin.challenge_id) else {
                return;
            };
            let Ok(recording_evidence_id) = id16(&begin.recording_evidence_id) else {
                return;
            };
            let observation = DesktopCaptureStartedV1 {
                command_id: command_id.to_vec(),
                host_claim_id: claim_id.to_vec(),
                challenge_id: challenge_id.to_vec(),
                recording_evidence_id: recording_evidence_id.to_vec(),
                started_at_unix_ms,
                os_permission_revision: OS_PERMISSION_REVISION_V1,
            };
            if observe(route, Observation::CaptureStarted(observation)).is_ok() {
                *active = Some(ActiveCaptureV1 {
                    command_id,
                    claim_id,
                    challenge_id,
                    recording_evidence_id,
                    started_at_unix_ms,
                    capture,
                });
            }
        }
        Some(Command::StopCapture(stop))
            if active.as_ref().is_some_and(|capture| {
                capture.recording_evidence_id.as_slice() == stop.recording_evidence_id
            }) =>
        {
            finish_active(route, active.take(), Some((command_id, claim_id)));
        }
        _ => {}
    }
}

fn finish_active(
    route: &HostRouteClientV1,
    active: Option<ActiveCaptureV1>,
    terminal_command: Option<([u8; 16], [u8; 16])>,
) {
    let Some(active) = active else { return };
    let ended_at_unix_ms = loop {
        let Some(now) = now_unix_ms() else {
            reject_active_value(route, active, "host_clock_unavailable");
            return;
        };
        if now > active.started_at_unix_ms {
            break now;
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    let (command_id, claim_id) = terminal_command.unwrap_or((active.command_id, active.claim_id));
    let recording_evidence_id = active.recording_evidence_id;
    let challenge_id = active.challenge_id;
    let started_at_unix_ms = active.started_at_unix_ms;
    let wav = match active.capture.finish() {
        Ok(wav) => wav,
        Err(code) => {
            reject_ids(
                route,
                command_id,
                claim_id,
                challenge_id,
                recording_evidence_id,
                code,
            );
            return;
        }
    };
    let audio_sha256 = Sha256::digest(&wav).to_vec();
    let _ = observe(
        route,
        Observation::CaptureCompleted(DesktopCaptureCompletedV1 {
            command_id: command_id.to_vec(),
            host_claim_id: claim_id.to_vec(),
            challenge_id: challenge_id.to_vec(),
            recording_evidence_id: recording_evidence_id.to_vec(),
            started_at_unix_ms,
            ended_at_unix_ms,
            canonical_wav_bytes: wav,
            audio_sha256,
        }),
    );
}

fn reject_active(route: &HostRouteClientV1, active: Option<ActiveCaptureV1>, code: &str) {
    if let Some(active) = active {
        reject_active_value(route, active, code);
    }
}

fn reject_active_value(route: &HostRouteClientV1, active: ActiveCaptureV1, code: &str) {
    reject_ids(
        route,
        active.command_id,
        active.claim_id,
        active.challenge_id,
        active.recording_evidence_id,
        code,
    );
}

fn reject_begin(
    route: &HostRouteClientV1,
    command_id: [u8; 16],
    claim_id: [u8; 16],
    begin: &BeginDesktopCaptureCommandV1,
    code: &str,
) {
    let (Ok(challenge_id), Ok(recording_evidence_id)) = (
        id16(&begin.challenge_id),
        id16(&begin.recording_evidence_id),
    ) else {
        return;
    };
    reject_ids(
        route,
        command_id,
        claim_id,
        challenge_id,
        recording_evidence_id,
        code,
    );
}

fn reject_ids(
    route: &HostRouteClientV1,
    command_id: [u8; 16],
    claim_id: [u8; 16],
    challenge_id: [u8; 16],
    recording_evidence_id: [u8; 16],
    code: &str,
) {
    let _ = observe(
        route,
        Observation::CaptureRejected(DesktopCaptureRejectedV1 {
            command_id: command_id.to_vec(),
            host_claim_id: claim_id.to_vec(),
            challenge_id: challenge_id.to_vec(),
            recording_evidence_id: recording_evidence_id.to_vec(),
            rejection_code: code.to_owned(),
        }),
    );
}

fn observe(route: &HostRouteClientV1, observation: Observation) -> Result<(), String> {
    let recording_id = match &observation {
        Observation::CaptureStarted(value) => value.recording_evidence_id.clone(),
        Observation::CaptureCompleted(value) => value.recording_evidence_id.clone(),
        Observation::CaptureRejected(value) => value.recording_evidence_id.clone(),
    };
    let mut last_error = "Desktop recording host observation was rejected".to_owned();
    for attempt in 0..3 {
        match route.observe(DesktopRecordingHostOperationV1 {
            operation: Some(Operation::Observation(DesktopRecordingHostObservationV1 {
                observation: Some(observation.clone()),
            })),
        }) {
            Ok(accepted) if accepted.recording_evidence_id == recording_id => return Ok(()),
            Ok(_) => last_error = "Desktop recording host observation was rejected".to_owned(),
            Err(error) => last_error = error,
        }
        if attempt < 2 {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    Err(last_error)
}

fn validate_begin(value: &BeginDesktopCaptureCommandV1) -> Result<(), ()> {
    id16(&value.challenge_id).map_err(|_| ())?;
    id16(&value.recording_evidence_id).map_err(|_| ())?;
    id16(&value.call_evidence_id).map_err(|_| ())?;
    if value.device_actor_sha256.len() != 32
        || value.expires_at_unix_ms <= 0
        || value.maximum_duration_millis == 0
        || value.consent_policy_revision == 0
        || value.consent_purpose != CONSENT_PURPOSE_V1
        || value.canonical_audio_format != CANONICAL_AUDIO_FORMAT_V1
        || value.call_evidence_revision == 0
    {
        return Err(());
    }
    Ok(())
}

fn id16(value: &[u8]) -> Result<[u8; 16], ()> {
    let id: [u8; 16] = value.try_into().map_err(|_| ())?;
    id.iter().any(|byte| *byte != 0).then_some(id).ok_or(())
}

fn random_id() -> Result<[u8; 16], String> {
    let mut id = [0_u8; 16];
    getrandom::fill(&mut id)
        .map_err(|_| "Desktop recording host entropy is unavailable".to_owned())?;
    id.iter()
        .any(|byte| *byte != 0)
        .then_some(id)
        .ok_or_else(|| "Desktop recording host entropy is unavailable".to_owned())
}

fn now_unix_ms() -> Option<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_begin() -> BeginDesktopCaptureCommandV1 {
        BeginDesktopCaptureCommandV1 {
            challenge_id: vec![1; 16],
            recording_evidence_id: vec![2; 16],
            device_actor_sha256: vec![3; 32],
            expires_at_unix_ms: 10,
            maximum_duration_millis: 1_000,
            consent_policy_revision: 1,
            consent_purpose: CONSENT_PURPOSE_V1.to_owned(),
            canonical_audio_format: CANONICAL_AUDIO_FORMAT_V1.to_owned(),
            call_evidence_id: vec![4; 16],
            call_evidence_revision: 1,
        }
    }

    #[test]
    fn host_has_no_capture_or_worker_before_explicit_connect() {
        let state = DesktopCallRecordingHostStateV1::default();
        assert!(state.worker.lock().expect("worker").is_none());
    }

    #[test]
    fn begin_contract_is_exact_and_has_no_boolean_consent() {
        assert!(validate_begin(&valid_begin()).is_ok());
        let mut invalid = valid_begin();
        invalid.consent_purpose = "meeting_archive".to_owned();
        assert!(validate_begin(&invalid).is_err());
        let proto = include_str!(
            "../../../../backend/src/desktop-call-recording-api/proto/makosh/desktop_call_recording/v1/recording.proto"
        );
        assert!(!proto.contains("consent_attested"));
    }
}

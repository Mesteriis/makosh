//! Long-lived Telegram process orchestration around the provider runtime.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use makosh_blob_client::{
    BlobClientError, BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use makosh_communications_ingress::{
    BodyAdmissionFailureV1, BodyBlobReceiptV1, COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID, COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
};
use makosh_runtime_protocol::v1::BlobDataOperationV1;
use makosh_runtime_protocol::{
    managed_control::{
        ManagedControlChannelV2, ManagedControlRequestDispatcherV2, ManagedControlTransportErrorV2,
    },
    v1::{
        ManagedRuntimeClientDeliveryResponseV1, ManagedRuntimeControlRequestV1,
        ManagedRuntimeControlResponseV1, ModuleClientResponseV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES,
    validation::module_client::{
        validate_module_client_request_v1, validate_module_client_response_v1,
    },
};
use makosh_telegram_automation_persistence::TelegramAutomationPersistence;
use makosh_telegram_calls_core::{
    TelegramCallDirection, TelegramCallDiscardReason, TelegramCallFailureCategory,
    TelegramCallMediaState, TelegramCallMediaUpdate, TelegramProviderCallState,
    TelegramProviderCallUpdate,
};
use makosh_telegram_calls_persistence::{TelegramCallsPersistence, TelegramCallsPersistenceError};
use makosh_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use makosh_telegram_tdlib::{
    TdJsonTransport, TdlibAuthorizationEvent, TdlibAuthorizationUpdate, TdlibCallDirection,
    TdlibCallDiscardReason, TdlibCallFailureCategory, TdlibCallObservation, TdlibCallState,
    TdlibError,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    TelegramCallProviderUpdate, TelegramDurableProjectionError, TelegramRuntime,
    TelegramRuntimeComposition,
    admission::TELEGRAM_BLOB_CAPABILITY_ID,
    bootstrap::{
        TelegramAdmittedProviderLoop, TelegramAdmittedRuntime,
        TelegramProviderReconfigurationContextV1, resolve_provider_reconfiguration_parameters,
    },
    call_evidence_outbox::{
        TelegramCallEvidenceOutboxRelayErrorV1, relay_call_evidence_outbox_once_v1,
    },
    calls_execution::TelegramCallExecutionError,
    client_transport::{self, TelegramClientTransportError},
    delivery_intent_consumer::{
        TelegramDeliveryIntentConsumeErrorV1, TelegramDeliveryIntentResultContextV1,
        consume_next_telegram_delivery_intent_v1,
    },
    delivery_intent_outbox::{
        TelegramDeliveryIntentOutboxRelayErrorV1, relay_telegram_delivery_intent_outbox_once_v1,
    },
    delivery_intent_worker::{
        TelegramDeliveryIntentWorkerContextV1, TelegramDeliveryIntentWorkerErrorV1,
        process_next_telegram_delivery_intent_v1,
    },
};

#[derive(Debug)]
pub enum TelegramProcessTick {
    Authorization(Option<TdlibAuthorizationEvent>),
    Runtime {
        frames: usize,
        provider_cursor: Option<String>,
    },
    Idle,
}

#[derive(Debug)]
pub enum TelegramDurableProcessError {
    Provider(TdlibError),
    Persistence(TelegramDurablePersistenceError),
    Calls(TelegramCallsPersistenceError),
    Projection(TelegramDurableProjectionError),
    CallExecution(TelegramCallExecutionError),
}

const TELEGRAM_MEDIA_CHUNK_BYTES: usize = 1024 * 1024;
const TELEGRAM_MEDIA_ADMISSION_MAX_FAILURES: u8 = 8;

type TelegramMediaHashResult = Result<(u64, [u8; 32]), &'static str>;

#[derive(Default)]
struct TelegramMediaAdmissionQueue {
    pending: VecDeque<makosh_telegram_tdlib::TdlibDownloadedFile>,
    hashing: Option<TelegramMediaHashJob>,
    uploading: Option<TelegramMediaUploadJob>,
}

struct TelegramMediaHashJob {
    downloaded: makosh_telegram_tdlib::TdlibDownloadedFile,
    receiver: Receiver<TelegramMediaHashResult>,
}

struct TelegramMediaUploadJob {
    downloaded: makosh_telegram_tdlib::TdlibDownloadedFile,
    provider_file: File,
    declared_size: u64,
    receipt_sha256: [u8; 32],
    reference_id: [u8; 16],
    offset: u64,
    failures: u8,
    retry_not_before: Instant,
    aborting: bool,
}

enum TelegramMediaAdmissionProgress {
    Idle,
    Completed(makosh_telegram_api::TelegramFileSnapshot),
    Failed(&'static str),
}

pub struct TelegramProcessLoop {
    composition: TelegramRuntimeComposition,
    provider_cursor: Option<String>,
    authorization_status: Option<makosh_telegram_api::TelegramAuthorizationStatus>,
    authorization_status_revision: u64,
    published_authorization_status_revision: u64,
    pending_operational_sequence: u64,
    published_operational_sequence: u64,
    durable_restore_required: bool,
    pending_downloaded_files: Vec<makosh_telegram_tdlib::TdlibDownloadedFile>,
}

impl TelegramProcessLoop {
    #[must_use]
    pub fn new(composition: TelegramRuntimeComposition) -> Self {
        let authorization_status = composition.has_runtime().then(telegram_ready_status);
        let authorization_status_revision = u64::from(authorization_status.is_some());
        Self {
            composition,
            provider_cursor: None,
            authorization_status,
            authorization_status_revision,
            published_authorization_status_revision: 0,
            pending_operational_sequence: 0,
            published_operational_sequence: 0,
            durable_restore_required: true,
            pending_downloaded_files: Vec::new(),
        }
    }

    pub fn composition_mut(&mut self) -> &mut TelegramRuntimeComposition {
        &mut self.composition
    }

    #[must_use]
    pub fn composition(&self) -> &TelegramRuntimeComposition {
        &self.composition
    }

    #[must_use]
    pub fn authorization_status(
        &self,
    ) -> Option<&makosh_telegram_api::TelegramAuthorizationStatus> {
        self.authorization_status.as_ref()
    }

    fn update_authorization_status(
        &mut self,
        status: makosh_telegram_api::TelegramAuthorizationStatus,
    ) {
        if self.authorization_status.as_ref() == Some(&status) {
            return;
        }
        self.authorization_status = Some(status);
        self.authorization_status_revision = self.authorization_status_revision.saturating_add(1);
    }

    fn pending_authorization_status_changed(
        &self,
    ) -> Option<(makosh_telegram_api::TelegramAuthorizationStatus, u64)> {
        if self.authorization_status_revision <= self.published_authorization_status_revision {
            return None;
        }
        self.authorization_status
            .clone()
            .map(|status| (status, self.authorization_status_revision))
    }

    fn mark_authorization_status_published(&mut self, revision: u64) {
        if revision == self.authorization_status_revision {
            self.published_authorization_status_revision = revision;
        }
    }

    fn observe_operational_sequence(&mut self, sequence: u64) {
        self.pending_operational_sequence = self.pending_operational_sequence.max(sequence);
    }

    fn pending_operational_projection_changed(&self) -> Option<u64> {
        (self.pending_operational_sequence > self.published_operational_sequence)
            .then_some(self.pending_operational_sequence)
    }

    fn mark_operational_projection_published(&mut self, sequence: u64) {
        self.published_operational_sequence = self.published_operational_sequence.max(sequence);
    }

    pub fn serve_client_connection_durable(
        &mut self,
        stream: UnixStream,
        durable: &TelegramDurablePersistence,
        automation: &TelegramAutomationPersistence,
        calls: &TelegramCallsPersistence,
        handle: &tokio::runtime::Handle,
    ) -> Result<(), TelegramClientTransportError> {
        {
            let runtime = self
                .composition
                .runtime_mut()
                .ok_or(TelegramClientTransportError::RuntimeUnavailable)?;
            client_transport::serve_connection_durable(
                stream, runtime, durable, automation, calls, handle,
            )?;
        }
        if self.has_pending_runtime_reconfiguration() {
            Err(TelegramClientTransportError::Reconfiguration)
        } else {
            Ok(())
        }
    }

    fn begin_pending_runtime_reconfiguration(
        &mut self,
        durable: &TelegramDurablePersistence,
        handle: &tokio::runtime::Handle,
        authorization_parameters: makosh_telegram_tdlib::TdlibAuthorizationParameters,
    ) -> Result<(), TelegramClientTransportError> {
        let pending = self
            .composition
            .runtime_mut()
            .and_then(TelegramRuntime::take_pending_runtime_reconfiguration);
        let Some(pending) = pending else {
            return Ok(());
        };
        let applying = handle
            .block_on(durable.mark_runtime_reconfiguration_applying(&pending.reconfiguration_id))
            .map_err(|_| TelegramClientTransportError::Reconfiguration)?;
        if self
            .composition
            .begin_runtime_reconfiguration(applying.clone(), authorization_parameters)
            .is_err()
        {
            let _ = handle.block_on(
                TelegramRuntime::<TdJsonTransport>::fail_runtime_reconfiguration_durable(
                    durable,
                    &applying,
                    "PROVIDER_RESTART_UNAVAILABLE",
                ),
            );
            return Err(TelegramClientTransportError::Reconfiguration);
        }
        self.durable_restore_required = true;
        Ok(())
    }

    fn has_pending_runtime_reconfiguration(&mut self) -> bool {
        self.composition
            .runtime_mut()
            .is_some_and(|runtime| runtime.has_pending_runtime_reconfiguration())
    }

    #[must_use]
    pub fn durable_restore_required(&self) -> bool {
        self.durable_restore_required
    }

    pub fn mark_durable_restore_complete(&mut self) {
        self.durable_restore_required = false;
        self.composition.clear_pending_runtime_reconfiguration();
    }

    fn take_downloaded_files(&mut self) -> Vec<makosh_telegram_tdlib::TdlibDownloadedFile> {
        std::mem::take(&mut self.pending_downloaded_files)
    }

    async fn persist_admitted_downloaded_file(
        &mut self,
        durable: &TelegramDurablePersistence,
        file: makosh_telegram_api::TelegramFileSnapshot,
    ) -> Result<(), TelegramDurableProcessError> {
        self.composition
            .runtime_mut()
            .ok_or_else(|| {
                TelegramDurableProcessError::Provider(TdlibError::Protocol(
                    "Telegram downloaded file has no runtime".to_owned(),
                ))
            })?
            .persist_admitted_downloaded_file_durable(durable, file)
            .await
            .map_err(TelegramDurableProcessError::Projection)
    }

    pub fn poll_once(&mut self, timeout: Duration) -> Result<TelegramProcessTick, TdlibError> {
        if self.composition.has_pending_authorization() {
            let event = self.composition.poll_authorization(timeout)?;
            if let Some(event) = &event {
                self.update_authorization_status(authorization_status(event));
            }
            return Ok(event
                .map(|value| TelegramProcessTick::Authorization(Some(value)))
                .unwrap_or(TelegramProcessTick::Idle));
        }
        if self.composition.has_runtime() {
            let batch = self
                .composition
                .poll_runtime_events(self.provider_cursor.clone())?;
            if let Some(cursor) = batch
                .frames
                .last()
                .and_then(|frame| frame.provider_cursor.clone())
            {
                self.provider_cursor = Some(cursor);
            }
            if let Some(sequence) = batch.frames.last().map(|frame| frame.sequence) {
                self.observe_operational_sequence(sequence);
            }
            return Ok(TelegramProcessTick::Runtime {
                frames: batch.frames.len() + batch.call_updates.len(),
                provider_cursor: self.provider_cursor.clone(),
            });
        }
        Ok(TelegramProcessTick::Idle)
    }

    pub async fn poll_once_durable<F>(
        &mut self,
        timeout: Duration,
        durable: &TelegramDurablePersistence,
        calls: &TelegramCallsPersistence,
        body_admitter: &mut F,
    ) -> Result<TelegramProcessTick, TelegramDurableProcessError>
    where
        F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
    {
        if self.composition.has_pending_authorization() {
            let event = self
                .composition
                .poll_authorization(timeout)
                .map_err(TelegramDurableProcessError::Provider)?;
            if let Some(event) = &event {
                self.update_authorization_status(authorization_status(event));
            }
            return Ok(event
                .map(|value| TelegramProcessTick::Authorization(Some(value)))
                .unwrap_or(TelegramProcessTick::Idle));
        }
        if self.composition.has_runtime() {
            let mut batch = self
                .composition
                .poll_runtime_events(self.provider_cursor.clone())
                .map_err(TelegramDurableProcessError::Provider)?;
            let downloaded_file_count = batch.downloaded_files.len();
            self.pending_downloaded_files
                .extend(std::mem::take(&mut batch.downloaded_files));
            for frame in &batch.frames {
                durable
                    .append_provider_event(frame)
                    .await
                    .map_err(TelegramDurableProcessError::Persistence)?;
                if let Some(runtime) = self.composition.runtime_mut() {
                    runtime
                        .persist_provider_frame_durable(durable, frame, body_admitter)
                        .await
                        .map_err(TelegramDurableProcessError::Projection)?;
                }
            }
            let (runtime_generation, runtime_instance_id, logical_human_owner_id) = self
                .composition
                .runtime_admission()
                .map(|admission| {
                    (
                        admission.runtime_generation,
                        admission.runtime_instance_id.clone(),
                        admission.logical_human_owner_id.clone(),
                    )
                })
                .ok_or_else(|| {
                    TelegramDurableProcessError::Provider(TdlibError::Protocol(
                        "Telegram call update has no admitted runtime fence".to_owned(),
                    ))
                })?;
            let observed_at_unix_seconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| {
                    TelegramDurableProcessError::Provider(TdlibError::Protocol(
                        "Telegram runtime clock is unavailable".to_owned(),
                    ))
                })?
                .as_secs();
            let call_update_count = batch.call_updates.len();
            for call_provider_update in batch.call_updates {
                match call_provider_update {
                    TelegramCallProviderUpdate::Observation(observation) => {
                        if observation.is_video {
                            continue;
                        }
                        let session = persist_call_observation(
                            calls,
                            &observation,
                            runtime_generation,
                            &logical_human_owner_id,
                            &runtime_instance_id,
                            observed_at_unix_seconds,
                        )
                        .await?;
                        if session.state.is_terminal()
                            && let Some(runtime) = self.composition.runtime_mut()
                        {
                            runtime
                                .stop_call_media_session(&session.call_session_id)
                                .map_err(TelegramDurableProcessError::CallExecution)?;
                        }
                    }
                    TelegramCallProviderUpdate::Ready {
                        observation,
                        material,
                    } => {
                        if observation.is_video {
                            continue;
                        }
                        let session = persist_call_observation(
                            calls,
                            &observation,
                            runtime_generation,
                            &logical_human_owner_id,
                            &runtime_instance_id,
                            observed_at_unix_seconds,
                        )
                        .await?;
                        if session.state != TelegramProviderCallState::MediaReady
                            || session.runtime_generation != runtime_generation
                        {
                            continue;
                        }
                        self.composition
                            .runtime_mut()
                            .ok_or_else(|| {
                                TelegramDurableProcessError::Provider(TdlibError::Protocol(
                                    "Telegram call ready update has no runtime".to_owned(),
                                ))
                            })?
                            .start_call_media_session(&session, material)
                            .map_err(TelegramDurableProcessError::CallExecution)?;
                        self.drain_call_media_events(calls, &session, observed_at_unix_seconds, 16)
                            .await?;
                    }
                    TelegramCallProviderUpdate::Signaling {
                        account_id,
                        tdlib_call_id,
                        data,
                    } => {
                        let Some(session) = calls
                            .call_by_runtime_identity(
                                &account_id,
                                runtime_generation,
                                tdlib_call_id,
                            )
                            .await
                            .map_err(TelegramDurableProcessError::Calls)?
                        else {
                            continue;
                        };
                        if session.state != TelegramProviderCallState::MediaReady {
                            continue;
                        }
                        self.composition
                            .runtime_mut()
                            .ok_or_else(|| {
                                TelegramDurableProcessError::Provider(TdlibError::Protocol(
                                    "Telegram call signaling has no runtime".to_owned(),
                                ))
                            })?
                            .receive_call_signaling_data(&session.call_session_id, data)
                            .map_err(TelegramDurableProcessError::CallExecution)?;
                        self.drain_call_media_events(calls, &session, observed_at_unix_seconds, 16)
                            .await?;
                    }
                }
            }
            if let Some(cursor) = batch
                .frames
                .last()
                .and_then(|frame| frame.provider_cursor.clone())
            {
                self.provider_cursor = Some(cursor);
            }
            if let Some(sequence) = batch.frames.last().map(|frame| frame.sequence) {
                self.observe_operational_sequence(sequence);
            }
            return Ok(TelegramProcessTick::Runtime {
                frames: batch.frames.len() + call_update_count + downloaded_file_count,
                provider_cursor: self.provider_cursor.clone(),
            });
        }
        Ok(TelegramProcessTick::Idle)
    }

    async fn drain_call_media_events(
        &mut self,
        calls: &TelegramCallsPersistence,
        call: &makosh_telegram_calls_core::TelegramCallSession,
        observed_at_unix_seconds: u64,
        limit: usize,
    ) -> Result<(), TelegramDurableProcessError> {
        for _ in 0..limit {
            let media_state = self
                .composition
                .runtime_mut()
                .ok_or_else(|| {
                    TelegramDurableProcessError::Provider(TdlibError::Protocol(
                        "Telegram call media has no runtime".to_owned(),
                    ))
                })?
                .poll_call_media_event(&call.call_session_id, call.tdlib_call_id)
                .map_err(TelegramDurableProcessError::CallExecution)?;
            let Some(media_state) = media_state else {
                continue;
            };
            let media_state = call_media_state(media_state);
            calls
                .ingest_media_update(&TelegramCallMediaUpdate {
                    account_id: call.account_id.clone(),
                    call_session_id: call.call_session_id.clone(),
                    runtime_generation: call.runtime_generation,
                    provider_revision: call.revision,
                    state: media_state,
                    observed_at_unix_seconds,
                })
                .await
                .map_err(TelegramDurableProcessError::Calls)?;
            if media_state == TelegramCallMediaState::Failed {
                self.composition
                    .runtime_mut()
                    .ok_or_else(|| {
                        TelegramDurableProcessError::Provider(TdlibError::Protocol(
                            "Telegram failed call teardown has no runtime".to_owned(),
                        ))
                    })?
                    .stop_call_media_session(&call.call_session_id)
                    .map_err(TelegramDurableProcessError::CallExecution)?;
                break;
            }
        }
        Ok(())
    }

    pub fn run_until<F, H>(
        &mut self,
        timeout: Duration,
        mut should_stop: F,
        mut on_tick: H,
    ) -> Result<(), TdlibError>
    where
        F: FnMut() -> bool,
        H: FnMut(TelegramProcessTick),
    {
        while !should_stop() {
            on_tick(self.poll_once(timeout)?);
        }
        Ok(())
    }
}

/// Runs the provider side of an admitted runtime without exposing a private
/// provider client socket. Core capability routing owns client request delivery.
pub fn serve_admitted_provider_loop(
    admitted: TelegramAdmittedRuntime,
    executor: &tokio::runtime::Runtime,
) -> Result<(), String> {
    let admitted = admitted.into_provider_loop();
    let TelegramAdmittedProviderLoop {
        mut control_channel,
        account_id,
        composition,
        durable,
        automation,
        calls,
        mut reconfiguration_context,
        event_connection,
        event_publish_permit,
        delivery_intent_subscribe_permit,
    } = admitted;
    let mut process = TelegramProcessLoop::new(composition);
    let mut media_admission = TelegramMediaAdmissionQueue::default();

    loop {
        handle_client_delivery(
            &mut control_channel,
            &mut process,
            &durable,
            &automation,
            &calls,
            &mut reconfiguration_context,
            executor,
        )?;
        if let Some(admission) = process.composition().runtime_admission() {
            let consumed_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?;
            let result_context = TelegramDeliveryIntentResultContextV1 {
                runtime_instance_id: admission.runtime_instance_id.clone(),
                runtime_generation: admission.runtime_generation,
                completed_at_unix_seconds: i64::try_from(consumed_at.as_secs())
                    .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?,
                completed_at_nanos: i32::try_from(consumed_at.subsec_nanos())
                    .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?,
            };
            match executor.block_on(consume_next_telegram_delivery_intent_v1(
                &durable.delivery_intent_store(),
                &event_connection,
                &delivery_intent_subscribe_permit,
                &admission.logical_human_owner_id,
                &result_context,
            )) {
                Ok(_) | Err(TelegramDeliveryIntentConsumeErrorV1::Unavailable) => {}
                Err(TelegramDeliveryIntentConsumeErrorV1::Persistence) => {
                    return Err("Telegram delivery-intent inbox persistence failed".to_owned());
                }
                Err(_) => {
                    return Err("Telegram delivery-intent delivery is invalid".to_owned());
                }
            }
        }
        let poll = {
            let mut body_admitter =
                |plaintext: &[u8]| admit_telegram_plaintext(&mut control_channel, plaintext);
            executor.block_on(process.poll_once_durable(
                Duration::from_millis(25),
                &durable,
                &calls,
                &mut body_admitter,
            ))
        };
        let provider_tick =
            poll.map_err(|error| format!("Telegram runtime provider loop failed: {error:?}"))?;
        for downloaded in process.take_downloaded_files() {
            let existing = executor
                .block_on(durable.file(
                    &downloaded.snapshot.account_id,
                    &downloaded.snapshot.provider_file_id,
                ))
                .map_err(|_| "Telegram downloaded file projection is unavailable".to_owned())?;
            if existing.as_ref().is_some_and(valid_blob_file_snapshot) {
                continue;
            }
            media_admission.push(downloaded);
        }
        match media_admission.advance(&mut control_channel) {
            TelegramMediaAdmissionProgress::Completed(file) => executor
                .block_on(process.persist_admitted_downloaded_file(&durable, file))
                .map_err(|error| {
                    format!("Telegram downloaded file persistence failed: {error:?}")
                })?,
            TelegramMediaAdmissionProgress::Failed(stage) => {
                if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                    eprintln!("developer_telegram_downloaded_file_unavailable stage={stage}");
                }
            }
            TelegramMediaAdmissionProgress::Idle => {}
        }
        if let Some((status, revision)) = process.pending_authorization_status_changed() {
            let admission = process
                .composition()
                .configured_admission()
                .cloned()
                .ok_or_else(|| "Telegram runtime admission is unavailable".to_owned())?;
            let occurred_at_unix_millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?
                .as_millis()
                .try_into()
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?;
            let mut dispatcher = TelegramBusyControlDispatcher;
            match crate::client_realtime::publish_authorization_status_changed_v1(
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_human_owner_id,
                admission.runtime_generation,
                revision,
                occurred_at_unix_millis,
                &status,
            ) {
                Ok(()) => process.mark_authorization_status_published(revision),
                Err(crate::client_realtime::TelegramClientRealtimeErrorV1::Unavailable) => {}
                Err(crate::client_realtime::TelegramClientRealtimeErrorV1::InvalidEvent) => {
                    return Err("Telegram authorization realtime status is invalid".to_owned());
                }
            }
        }
        if let Some(sequence) = process.pending_operational_projection_changed() {
            let admission = process
                .composition()
                .runtime_admission()
                .cloned()
                .ok_or_else(|| "Telegram runtime admission is unavailable".to_owned())?;
            let occurred_at_unix_millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?
                .as_millis()
                .try_into()
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?;
            let mut dispatcher = TelegramBusyControlDispatcher;
            match crate::client_realtime::publish_operational_projection_changed_v1(
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_human_owner_id,
                admission.runtime_generation,
                &account_id,
                sequence,
                occurred_at_unix_millis,
            ) {
                Ok(()) => process.mark_operational_projection_published(sequence),
                Err(crate::client_realtime::TelegramClientRealtimeErrorV1::Unavailable) => {}
                Err(crate::client_realtime::TelegramClientRealtimeErrorV1::InvalidEvent) => {
                    return Err("Telegram operational realtime event is invalid".to_owned());
                }
            }
        }
        if process.durable_restore_required() && process.composition().has_runtime() {
            let runtime = process
                .composition_mut()
                .runtime_mut()
                .ok_or_else(|| "Telegram runtime provider disappeared during restore".to_owned())?;
            executor
                .block_on(runtime.restore_account_state_durable(&durable, &account_id, 10_000))
                .map_err(|error| format!("Telegram durable state restore failed: {error:?}"))?;
            executor
                .block_on(
                    runtime.complete_pending_runtime_reconfiguration_durable(&durable, &account_id),
                )
                .map_err(|error| {
                    format!("Telegram runtime reconfiguration completion failed: {error:?}")
                })?;
            if let Some((_, latest_sequence)) = executor
                .block_on(durable.provider_event_sequence_bounds(&account_id))
                .map_err(|_| "Telegram operational realtime restore is unavailable".to_owned())?
            {
                process.observe_operational_sequence(latest_sequence);
            }
            process.mark_durable_restore_complete();
        }
        let delivery_intent_context = process.composition().runtime_admission().map(|admission| {
            TelegramDeliveryIntentWorkerContextV1 {
                runtime_instance_id: admission.runtime_instance_id.clone(),
                runtime_generation: admission.runtime_generation,
            }
        });
        if let Some(runtime) = process.composition_mut().runtime_mut() {
            let now_unix_seconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?
                .as_secs();
            let now_i64 = i64::try_from(now_unix_seconds)
                .map_err(|_| "Telegram runtime clock is unavailable".to_owned())?;
            let delivery_intent_context = delivery_intent_context
                .as_ref()
                .ok_or_else(|| "Telegram runtime admission is unavailable".to_owned())?;
            match executor.block_on(process_next_telegram_delivery_intent_v1(
                &mut control_channel,
                runtime,
                &durable,
                delivery_intent_context,
                now_i64,
            )) {
                Ok(_) => {}
                Err(TelegramDeliveryIntentWorkerErrorV1::InvalidClock) => {
                    return Err("Telegram delivery-intent worker clock is invalid".to_owned());
                }
                Err(TelegramDeliveryIntentWorkerErrorV1::InvalidRuntime) => {
                    return Err("Telegram delivery-intent runtime identity is invalid".to_owned());
                }
                Err(TelegramDeliveryIntentWorkerErrorV1::Persistence) => {
                    return Err("Telegram delivery-intent persistence failed".to_owned());
                }
                Err(TelegramDeliveryIntentWorkerErrorV1::ResultEnvelope) => {
                    return Err("Telegram delivery-intent result is invalid".to_owned());
                }
            }
            executor
                .block_on(runtime.execute_due_durable_operations(
                    &durable,
                    &account_id,
                    now_unix_seconds,
                    16,
                    "telegram-provider-runtime",
                    |intent| {
                        let mut dispatcher = TelegramBusyControlDispatcher;
                        request_managed_blob_session_v2(
                            &mut control_channel,
                            &mut dispatcher,
                            ManagedBlobSessionRequestV1 {
                                capability_id: TELEGRAM_BLOB_CAPABILITY_ID,
                                operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
                                reference_id: &intent.reference_id,
                                declared_size: intent.declared_size,
                                backup_class: intent.backup_class,
                                receipt_sha256: None,
                                custody_target: None,
                            },
                        )
                        .map_err(|_| {
                            TdlibError::Protocol(
                                "Telegram Blob session request was denied".to_owned(),
                            )
                        })
                    },
                ))
                .map_err(|error| format!("Telegram durable execution failed: {error:?}"))?;
            executor
                .block_on(runtime.execute_due_call_operations(
                    &calls,
                    &account_id,
                    now_unix_seconds,
                    16,
                ))
                .map_err(|error| format!("Telegram call execution failed: {error:?}"))?;
            if runtime.has_call_signaling_media() {
                if let Some((active_media, media_state)) = runtime
                    .poll_active_call_media_event()
                    .map_err(|error| format!("Telegram call media polling failed: {error:?}"))?
                {
                    let media_state = call_media_state(media_state);
                    executor
                        .block_on(calls.ingest_media_update(&TelegramCallMediaUpdate {
                            account_id: active_media.account_id.clone(),
                            call_session_id: active_media.call_session_id.clone(),
                            runtime_generation: active_media.runtime_generation,
                            provider_revision: active_media.provider_revision,
                            state: media_state,
                            observed_at_unix_seconds: now_unix_seconds,
                        }))
                        .map_err(|_| "Telegram call media projection failed".to_owned())?;
                    if media_state == TelegramCallMediaState::Failed {
                        runtime
                            .stop_call_media_session(&active_media.call_session_id)
                            .map_err(|error| {
                                format!("Telegram failed call teardown failed: {error:?}")
                            })?;
                    }
                }
            }
        }
        let published_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Telegram runtime clock is unavailable".to_owned())
            .and_then(|duration| {
                i64::try_from(duration.as_secs())
                    .map_err(|_| "Telegram runtime clock is unavailable".to_owned())
            })?;
        match executor.block_on(
            crate::communications_outbox::relay_communications_outbox_once(
                &durable,
                &event_connection,
                &event_publish_permit,
                published_at_unix_seconds,
            ),
        ) {
            Ok(_) => {}
            Err(
                crate::communications_outbox::TelegramCommunicationsOutboxRelayError::Unavailable,
            ) => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(
                crate::communications_outbox::TelegramCommunicationsOutboxRelayError::Persistence,
            ) => {
                return Err("Telegram runtime outbox persistence failed".to_owned());
            }
        }
        match executor.block_on(relay_telegram_delivery_intent_outbox_once_v1(
            &durable.delivery_intent_store(),
            &event_connection,
            &event_publish_permit,
            published_at_unix_seconds,
        )) {
            Ok(_) | Err(TelegramDeliveryIntentOutboxRelayErrorV1::Unavailable) => {}
            Err(TelegramDeliveryIntentOutboxRelayErrorV1::Persistence(_)) => {
                return Err("Telegram delivery-intent outbox persistence failed".to_owned());
            }
        }
        match executor.block_on(relay_call_evidence_outbox_once_v1(
            &calls,
            &event_connection,
            &event_publish_permit,
            published_at_unix_seconds,
        )) {
            Ok(_) | Err(TelegramCallEvidenceOutboxRelayErrorV1::Unavailable) => {}
            Err(TelegramCallEvidenceOutboxRelayErrorV1::Persistence) => {
                return Err("Telegram call evidence outbox persistence failed".to_owned());
            }
        }
        if provider_tick_needs_idle_pause(&provider_tick) {
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}

fn telegram_ready_status() -> makosh_telegram_api::TelegramAuthorizationStatus {
    makosh_telegram_api::TelegramAuthorizationStatus {
        state: "ready".to_owned(),
        qr_link: None,
        password_hint: None,
    }
}

fn provider_tick_needs_idle_pause(tick: &TelegramProcessTick) -> bool {
    matches!(
        tick,
        TelegramProcessTick::Idle | TelegramProcessTick::Runtime { frames: 0, .. }
    )
}

fn admit_telegram_plaintext(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    plaintext: &[u8],
) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
    if plaintext.is_empty() || plaintext.len() > makosh_telegram_api::MAX_TEXT_BYTES {
        return Err(BodyAdmissionFailureV1::SizeLimitExceeded);
    }
    let mut reference_id = [0_u8; 16];
    getrandom::fill(&mut reference_id).map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    if reference_id.iter().all(|byte| *byte == 0) {
        return Err(BodyAdmissionFailureV1::SourceUnavailable);
    }
    let sha256: [u8; 32] = Sha256::digest(plaintext).into();
    control_channel
        .inner_mut()
        .set_nonblocking(false)
        .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let mut dispatcher = TelegramBusyControlDispatcher;
    let session = request_managed_blob_session_v2(
        control_channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: TELEGRAM_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationWriteV1,
            reference_id: &reference_id,
            declared_size: u64::try_from(plaintext.len())
                .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
            backup_class: 1,
            receipt_sha256: Some(&sha256),
            custody_target: Some(ManagedBlobCustodyTargetV1 {
                owner_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
                module_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID,
                capability_id: COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
            }),
        },
    );
    let restored = control_channel.inner_mut().set_nonblocking(true);
    let session = session.map_err(|_| BodyAdmissionFailureV1::PolicyRejected)?;
    restored.map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    let custody_transfer_source_proof = session.custody_transfer_source_proof;
    BlobDataClient::new(session.data_socket_path)
        .and_then(|client| client.write(session.grant, session.channel_binding, plaintext.to_vec()))
        .map_err(|_| BodyAdmissionFailureV1::SourceUnavailable)?;
    Ok(BodyBlobReceiptV1 {
        blob_ref: format!("blob-content:{}", hex_reference_id(&reference_id)),
        reference_id,
        declared_bytes: u64::try_from(plaintext.len())
            .map_err(|_| BodyAdmissionFailureV1::SizeLimitExceeded)?,
        sha256,
        custody_transfer_source_proof,
        media_type: "text/plain".to_owned(),
    })
}

fn valid_blob_file_snapshot(file: &makosh_telegram_api::TelegramFileSnapshot) -> bool {
    file.is_downloaded
        && file
            .blob_reference_id
            .as_ref()
            .is_some_and(|value| value.len() == 16)
        && file
            .blob_plaintext_sha256
            .as_ref()
            .is_some_and(|value| value.len() == 32)
        && file.blob_backup_class.is_some_and(|value| value > 0)
}

impl TelegramMediaAdmissionQueue {
    fn push(&mut self, downloaded: makosh_telegram_tdlib::TdlibDownloadedFile) {
        let account_id = &downloaded.snapshot.account_id;
        let provider_file_id = &downloaded.snapshot.provider_file_id;
        let already_pending = self.pending.iter().any(|candidate| {
            candidate.snapshot.account_id == *account_id
                && candidate.snapshot.provider_file_id == *provider_file_id
        });
        let already_hashing = self.hashing.as_ref().is_some_and(|candidate| {
            candidate.downloaded.snapshot.account_id == *account_id
                && candidate.downloaded.snapshot.provider_file_id == *provider_file_id
        });
        let already_uploading = self.uploading.as_ref().is_some_and(|candidate| {
            candidate.downloaded.snapshot.account_id == *account_id
                && candidate.downloaded.snapshot.provider_file_id == *provider_file_id
        });
        if !already_pending && !already_hashing && !already_uploading {
            self.pending.push_back(downloaded);
        }
    }

    fn advance(
        &mut self,
        control_channel: &mut ManagedControlChannelV2<UnixStream>,
    ) -> TelegramMediaAdmissionProgress {
        if self.uploading.is_none() {
            if let Err(stage) = self.promote_finished_hash() {
                return TelegramMediaAdmissionProgress::Failed(stage);
            }
            if self.hashing.is_none() {
                if let Err(stage) = self.start_next_hash() {
                    return TelegramMediaAdmissionProgress::Failed(stage);
                }
            }
            if let Err(stage) = self.promote_finished_hash() {
                return TelegramMediaAdmissionProgress::Failed(stage);
            }
        }
        let Some(uploading) = self.uploading.as_mut() else {
            return TelegramMediaAdmissionProgress::Idle;
        };
        if Instant::now() < uploading.retry_not_before {
            return TelegramMediaAdmissionProgress::Idle;
        }
        if uploading.aborting {
            return match abort_telegram_media_upload(control_channel, uploading) {
                Ok(()) => {
                    self.uploading = None;
                    TelegramMediaAdmissionProgress::Idle
                }
                Err(stage) => {
                    schedule_telegram_media_retry(uploading);
                    TelegramMediaAdmissionProgress::Failed(stage)
                }
            };
        }
        match upload_one_telegram_media_chunk(control_channel, uploading) {
            Ok(false) => TelegramMediaAdmissionProgress::Idle,
            Ok(true) => {
                let completed = self
                    .uploading
                    .take()
                    .expect("completed Telegram media upload exists");
                let mut file = completed.downloaded.snapshot;
                file.size_bytes = Some(completed.declared_size);
                file.downloaded_size_bytes = Some(completed.declared_size);
                file.blob_reference_id = Some(completed.reference_id.to_vec());
                file.blob_plaintext_sha256 = Some(completed.receipt_sha256.to_vec());
                file.blob_backup_class = Some(1);
                TelegramMediaAdmissionProgress::Completed(file)
            }
            Err(stage) => {
                uploading.failures = uploading.failures.saturating_add(1);
                let permanent = matches!(
                    stage,
                    "read_provider_file" | "size_provider_file" | "bound_provider_file"
                );
                if permanent || uploading.failures >= TELEGRAM_MEDIA_ADMISSION_MAX_FAILURES {
                    uploading.aborting = true;
                    uploading.retry_not_before = Instant::now();
                } else {
                    schedule_telegram_media_retry(uploading);
                }
                TelegramMediaAdmissionProgress::Failed(stage)
            }
        }
    }

    fn start_next_hash(&mut self) -> Result<(), &'static str> {
        let Some(downloaded) = self.pending.pop_front() else {
            return Ok(());
        };
        let local_path = downloaded.local_path.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        if std::thread::Builder::new()
            .name("telegram-media-hash".to_owned())
            .spawn(move || {
                let _ = sender.send(hash_telegram_provider_file(&local_path));
            })
            .is_err()
        {
            self.pending.push_front(downloaded);
            return Err("start_provider_file_hash");
        }
        self.hashing = Some(TelegramMediaHashJob {
            downloaded,
            receiver,
        });
        Ok(())
    }

    fn promote_finished_hash(&mut self) -> Result<(), &'static str> {
        let result = match self.hashing.as_ref() {
            Some(job) => match job.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => Some(Err("hash_provider_file")),
            },
            None => None,
        };
        let Some(result) = result else {
            return Ok(());
        };
        let job = self.hashing.take().expect("finished hash job exists");
        let (declared_size, receipt_sha256) = result?;
        let provider_file =
            File::open(&job.downloaded.local_path).map_err(|_| "read_provider_file")?;
        if provider_file
            .metadata()
            .map_err(|_| "size_provider_file")?
            .len()
            != declared_size
        {
            return Err("changed_provider_file");
        }
        let mut reference_id = [0_u8; 16];
        getrandom::fill(&mut reference_id).map_err(|_| "identify_provider_file")?;
        if reference_id.iter().all(|byte| *byte == 0) {
            return Err("identify_provider_file");
        }
        self.uploading = Some(TelegramMediaUploadJob {
            downloaded: job.downloaded,
            provider_file,
            declared_size,
            receipt_sha256,
            reference_id,
            offset: 0,
            failures: 0,
            retry_not_before: Instant::now(),
            aborting: false,
        });
        Ok(())
    }
}

fn hash_telegram_provider_file(path: &std::path::Path) -> TelegramMediaHashResult {
    let mut provider_file = File::open(path).map_err(|_| "read_provider_file")?;
    let declared_size = provider_file
        .metadata()
        .map_err(|_| "size_provider_file")?
        .len();
    if declared_size == 0 || declared_size > crate::admission::TELEGRAM_MEDIA_CLIENT_MAX_BYTES_V1 {
        return Err("bound_provider_file");
    }
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; TELEGRAM_MEDIA_CHUNK_BYTES];
    loop {
        let read = provider_file
            .read(&mut buffer)
            .map_err(|_| "read_provider_file")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok((declared_size, hasher.finalize().into()))
}

fn upload_one_telegram_media_chunk(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    upload: &mut TelegramMediaUploadJob,
) -> Result<bool, &'static str> {
    let remaining = upload
        .declared_size
        .checked_sub(upload.offset)
        .ok_or("size_provider_file")?;
    if remaining == 0 {
        return Ok(true);
    }
    let chunk_len = usize::try_from(
        remaining.min(u64::try_from(TELEGRAM_MEDIA_CHUNK_BYTES).map_err(|_| "size_provider_file")?),
    )
    .map_err(|_| "size_provider_file")?;
    upload
        .provider_file
        .seek(SeekFrom::Start(upload.offset))
        .map_err(|_| "read_provider_file")?;
    let mut chunk = vec![0_u8; chunk_len];
    upload
        .provider_file
        .read_exact(&mut chunk)
        .map_err(|_| "read_provider_file")?;
    let end = upload
        .offset
        .checked_add(u64::try_from(chunk_len).map_err(|_| "size_provider_file")?)
        .ok_or("size_provider_file")?;
    let complete = end == upload.declared_size;
    let session = request_telegram_media_upload_session(control_channel, upload)?;
    BlobDataClient::new(session.data_socket_path.clone())
        .and_then(|client| {
            client.write_chunk(
                session.grant.clone(),
                session.channel_binding.clone(),
                upload.offset,
                chunk,
                complete,
            )
        })
        .map_err(|error| blob_write_error_stage(error, upload.offset == 0))?;
    upload.offset = end;
    upload.failures = 0;
    upload.retry_not_before = Instant::now();
    Ok(complete)
}

fn request_telegram_media_upload_session(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    upload: &TelegramMediaUploadJob,
) -> Result<makosh_blob_client::ManagedBlobSessionV1, &'static str> {
    control_channel
        .inner_mut()
        .set_nonblocking(false)
        .map_err(|_| "prepare_blob_session")?;
    let result = (|| {
        let mut dispatcher = TelegramBusyControlDispatcher;
        request_managed_blob_session_v2(
            control_channel,
            &mut dispatcher,
            ManagedBlobSessionRequestV1 {
                capability_id: crate::admission::TELEGRAM_BLOB_CAPABILITY_ID,
                operation: BlobDataOperationV1::BlobDataOperationWriteV1,
                reference_id: &upload.reference_id,
                declared_size: upload.declared_size,
                backup_class: 1,
                receipt_sha256: Some(&upload.receipt_sha256),
                custody_target: None,
            },
        )
        .map_err(|_| "request_blob_session")
    })();
    let restored = control_channel.inner_mut().set_nonblocking(true);
    restored.map_err(|_| "restore_control_channel")?;
    result
}

fn abort_telegram_media_upload(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    upload: &TelegramMediaUploadJob,
) -> Result<(), &'static str> {
    let session = request_telegram_media_upload_session(control_channel, upload)?;
    BlobDataClient::new(session.data_socket_path.clone())
        .and_then(|client| {
            client.abort_write(session.grant.clone(), session.channel_binding.clone())
        })
        .map_err(blob_abort_error_stage)
}

fn blob_write_error_stage(error: BlobClientError, initial_chunk: bool) -> &'static str {
    match (initial_chunk, error) {
        (true, BlobClientError::Rejected(_)) => "write_provider_file_initial_rejected",
        (false, BlobClientError::Rejected(_)) => "write_provider_file_resume_rejected",
        (true, BlobClientError::Connect(_)) => "write_provider_file_initial_connect",
        (false, BlobClientError::Connect(_)) => "write_provider_file_resume_connect",
        (true, BlobClientError::Io(_)) => "write_provider_file_initial_io",
        (false, BlobClientError::Io(_)) => "write_provider_file_resume_io",
        (true, BlobClientError::Unavailable) => "write_provider_file_initial_unavailable",
        (false, BlobClientError::Unavailable) => "write_provider_file_resume_unavailable",
        (true, _) => "write_provider_file_initial_protocol",
        (false, _) => "write_provider_file_resume_protocol",
    }
}

fn blob_abort_error_stage(error: BlobClientError) -> &'static str {
    match error {
        BlobClientError::Rejected(_) => "abort_provider_file_write_rejected",
        BlobClientError::Connect(_) => "abort_provider_file_write_connect",
        BlobClientError::Io(_) => "abort_provider_file_write_io",
        BlobClientError::Unavailable => "abort_provider_file_write_unavailable",
        _ => "abort_provider_file_write_protocol",
    }
}

fn schedule_telegram_media_retry(upload: &mut TelegramMediaUploadJob) {
    let exponent = u32::from(upload.failures.min(5));
    upload.retry_not_before = Instant::now() + Duration::from_millis(100 * 2_u64.pow(exponent));
}

fn hex_reference_id(reference_id: &[u8; 16]) -> String {
    reference_id
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn persist_call_observation(
    calls: &TelegramCallsPersistence,
    observation: &TdlibCallObservation,
    runtime_generation: u64,
    logical_human_owner_id: &str,
    runtime_instance_id: &str,
    observed_at_unix_seconds: u64,
) -> Result<makosh_telegram_calls_core::TelegramCallSession, TelegramDurableProcessError> {
    let update = call_update(observation, runtime_generation, observed_at_unix_seconds);
    let suggested_call_session_id = if update.direction == TelegramCallDirection::Outgoing {
        calls
            .pending_outgoing_call_session_id(
                &update.account_id,
                update.runtime_generation,
                update.tdlib_call_id,
                &update.provider_user_id,
            )
            .await
            .map_err(TelegramDurableProcessError::Calls)?
            .unwrap_or_else(|| call_session_id(&update))
    } else {
        call_session_id(&update)
    };
    calls
        .ingest_provider_update_with_call_evidence(
            &suggested_call_session_id,
            &update,
            logical_human_owner_id,
            runtime_instance_id,
        )
        .await
        .map(|persisted| persisted.session)
        .map_err(TelegramDurableProcessError::Calls)
}

fn call_update(
    observation: &TdlibCallObservation,
    runtime_generation: u64,
    observed_at_unix_seconds: u64,
) -> TelegramProviderCallUpdate {
    TelegramProviderCallUpdate {
        account_id: observation.account_id.clone(),
        runtime_generation,
        tdlib_call_id: observation.tdlib_call_id,
        provider_call_unique_id: observation.provider_call_unique_id,
        provider_user_id: observation.provider_user_id.clone(),
        direction: match observation.direction {
            TdlibCallDirection::Incoming => TelegramCallDirection::Incoming,
            TdlibCallDirection::Outgoing => TelegramCallDirection::Outgoing,
        },
        state: match observation.state {
            TdlibCallState::Pending => TelegramProviderCallState::Pending,
            TdlibCallState::ExchangingKeys => TelegramProviderCallState::ExchangingKeys,
            TdlibCallState::Ready => TelegramProviderCallState::MediaReady,
            TdlibCallState::HangingUp => TelegramProviderCallState::HangingUp,
            TdlibCallState::Discarded => TelegramProviderCallState::Discarded,
            TdlibCallState::Error => TelegramProviderCallState::Error,
        },
        pending_created: observation.pending_created,
        pending_received: observation.pending_received,
        discard_reason: observation.discard_reason.map(|reason| match reason {
            TdlibCallDiscardReason::Empty => TelegramCallDiscardReason::Empty,
            TdlibCallDiscardReason::Missed => TelegramCallDiscardReason::Missed,
            TdlibCallDiscardReason::Declined => TelegramCallDiscardReason::Declined,
            TdlibCallDiscardReason::Disconnected => TelegramCallDiscardReason::Disconnected,
            TdlibCallDiscardReason::HungUp => TelegramCallDiscardReason::HungUp,
        }),
        failure_category: observation.failure_category.map(|category| match category {
            TdlibCallFailureCategory::Network => TelegramCallFailureCategory::Network,
            TdlibCallFailureCategory::NotAvailable => TelegramCallFailureCategory::NotAvailable,
            TdlibCallFailureCategory::Permission => TelegramCallFailureCategory::Permission,
            TdlibCallFailureCategory::Unknown => TelegramCallFailureCategory::Unknown,
        }),
        observed_at_unix_seconds,
    }
}

fn call_media_state(
    state: makosh_telegram_call_media_contract::TelegramCallMediaStateV1,
) -> TelegramCallMediaState {
    match state {
        makosh_telegram_call_media_contract::TelegramCallMediaStateV1::Connecting => {
            TelegramCallMediaState::Connecting
        }
        makosh_telegram_call_media_contract::TelegramCallMediaStateV1::Established => {
            TelegramCallMediaState::Active
        }
        makosh_telegram_call_media_contract::TelegramCallMediaStateV1::Reconnecting => {
            TelegramCallMediaState::Reconnecting
        }
        makosh_telegram_call_media_contract::TelegramCallMediaStateV1::Failed => {
            TelegramCallMediaState::Failed
        }
    }
}

fn call_session_id(update: &TelegramProviderCallUpdate) -> String {
    let mut digest = Sha256::new();
    digest.update(update.account_id.as_bytes());
    digest.update([0]);
    if let Some(provider_call_unique_id) = update.provider_call_unique_id {
        digest.update(b"provider");
        digest.update(provider_call_unique_id.to_be_bytes());
    } else {
        digest.update(b"runtime");
        digest.update(update.runtime_generation.to_be_bytes());
        digest.update(update.tdlib_call_id.to_be_bytes());
    }
    let digest = digest.finalize();
    let mut reference_id = [0_u8; 16];
    reference_id.copy_from_slice(&digest[..16]);
    format!("tg-call-{}", hex_reference_id(&reference_id))
}

fn handle_client_delivery(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    process: &mut TelegramProcessLoop,
    durable: &TelegramDurablePersistence,
    automation: &TelegramAutomationPersistence,
    calls: &TelegramCallsPersistence,
    reconfiguration_context: &mut TelegramProviderReconfigurationContextV1,
    executor: &tokio::runtime::Runtime,
) -> Result<(), String> {
    let Some((correlation_id, control_request)) = channel
        .try_receive_request()
        .map_err(|_| "Telegram runtime control channel is unavailable".to_owned())?
    else {
        return Ok(());
    };
    let request = match control_request.operation {
        Some(Operation::ClientDelivery(delivery)) => match delivery.request {
            Some(request) => request,
            None => {
                write_control_error(
                    channel,
                    correlation_id,
                    "managed_runtime_control_invalid_client_delivery",
                )?;
                return Ok(());
            }
        },
        _ => {
            write_control_error(
                channel,
                correlation_id,
                "managed_runtime_control_unexpected_request",
            )?;
            return Ok(());
        }
    };
    if validate_module_client_request_v1(&request).is_err() {
        write_client_delivery_response(
            channel,
            correlation_id,
            ModuleClientResponseV1 {
                protocol_major: 1,
                request_id: request.request_id,
                response_payload: Vec::new(),
                error_code: "REJECTED".to_owned(),
            },
        )?;
        return Ok(());
    }
    let runtime_available = process.composition().has_runtime();
    let authorization_status = process.authorization_status().cloned();
    let configuration_response = {
        let mut dispatcher = TelegramBusyControlDispatcher;
        executor.block_on(crate::configuration_client_port::try_handle(
            &request.encode_to_vec(),
            crate::configuration_client_port::TelegramConfigurationClientContextV1 {
                runtime_available,
                composition: process.composition_mut(),
                authorization_status: authorization_status.as_ref(),
                durable,
                control_channel: channel,
                dispatcher: &mut dispatcher,
                reconfiguration_context,
            },
        ))
    };
    let response = match configuration_response {
        Ok(Some(payload)) => ModuleClientResponseV1::decode(payload.as_slice())
            .map_err(|_| "Telegram runtime client response is invalid".to_owned())?,
        Err(error) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: client_transport::module_error_code(&error).to_owned(),
        },
        Ok(None) if runtime_available => {
            let runtime = process
                .composition_mut()
                .runtime_mut()
                .ok_or_else(|| "Telegram runtime disappeared during delivery".to_owned())?;
            authorize_media_for_request(channel, runtime, &request)?;
            match executor.block_on(client_transport::handle_durable_request(
                runtime,
                durable,
                automation,
                calls,
                &request.encode_to_vec(),
            )) {
                Ok(payload) => ModuleClientResponseV1::decode(payload.as_slice())
                    .map_err(|_| "Telegram runtime client response is invalid".to_owned())?,
                Err(error) => {
                    if std::env::var_os("MAKOSH_DEVELOPER_VERBOSE").is_some() {
                        eprintln!(
                            "developer_telegram_client_request_error request_id={} error={error:?}",
                            request.request_id,
                        );
                    }
                    ModuleClientResponseV1 {
                        protocol_major: 1,
                        request_id: request.request_id,
                        response_payload: Vec::new(),
                        error_code: client_transport::module_error_code(&error).to_owned(),
                    }
                }
            }
        }
        Ok(None) => ModuleClientResponseV1 {
            protocol_major: 1,
            request_id: request.request_id,
            response_payload: Vec::new(),
            error_code: "RUNTIME_UNAVAILABLE".to_owned(),
        },
    };
    validate_module_client_response_v1(&response)
        .map_err(|_| "Telegram runtime client response is invalid".to_owned())?;
    write_client_delivery_response(channel, correlation_id, response)?;
    if !process.has_pending_runtime_reconfiguration() {
        return Ok(());
    }
    let mut dispatcher = TelegramBusyControlDispatcher;
    let authorization_parameters = resolve_provider_reconfiguration_parameters(
        channel,
        &mut dispatcher,
        reconfiguration_context,
    )
    .map_err(|_| "Telegram runtime reconfiguration credentials are unavailable".to_owned())?;
    process
        .begin_pending_runtime_reconfiguration(durable, executor.handle(), authorization_parameters)
        .map_err(|_| "Telegram runtime reconfiguration failed".to_owned())
}

fn authorize_media_for_request<T: makosh_telegram_tdlib::TdlibTransport>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    runtime: &mut crate::TelegramRuntime<T>,
    request: &makosh_runtime_protocol::v1::ModuleClientRequestV1,
) -> Result<(), String> {
    let Ok(command) = makosh_telegram_api::client_wire::decode_command(&request.request_payload)
    else {
        return Ok(());
    };
    let makosh_telegram_api::TelegramProviderCommand::SendMedia(media) = command else {
        return Ok(());
    };
    let mut dispatcher = TelegramBusyControlDispatcher;
    let session = request_managed_blob_session_v2(
        channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: TELEGRAM_BLOB_CAPABILITY_ID,
            operation: BlobDataOperationV1::BlobDataOperationReadRangeV1,
            reference_id: &media.blob.reference_id,
            declared_size: media.blob.declared_size,
            backup_class: media.blob.backup_class,
            receipt_sha256: None,
            custody_target: None,
        },
    )
    .map_err(|_| "Telegram Blob session request was denied".to_owned())?;
    runtime
        .authorize_media_session(session, &media.blob)
        .map_err(|_| "Telegram Blob session was rejected".to_owned())
}

struct TelegramBusyControlDispatcher;

impl ManagedControlRequestDispatcherV2<UnixStream> for TelegramBusyControlDispatcher {
    fn dispatch_request(
        &mut self,
        channel: &mut ManagedControlChannelV2<UnixStream>,
        correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
        request: ManagedRuntimeControlRequestV1,
    ) -> Result<(), ManagedControlTransportErrorV2> {
        let response = match request.operation {
            Some(Operation::ClientDelivery(delivery)) => match delivery.request {
                Some(request) if validate_module_client_request_v1(&request).is_ok() => {
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::ClientDelivery(
                            ManagedRuntimeClientDeliveryResponseV1 {
                                response: Some(ModuleClientResponseV1 {
                                    protocol_major: 1,
                                    request_id: request.request_id,
                                    response_payload: Vec::new(),
                                    error_code: "RUNTIME_BUSY".to_owned(),
                                }),
                            },
                        )),
                        error_code: String::new(),
                    }
                }
                _ => ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            },
            _ => ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: "managed_runtime_control_unexpected_request".to_owned(),
            },
        };
        channel.write_response(correlation_id, response)
    }
}

fn write_client_delivery_response(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    response: ModuleClientResponseV1,
) -> Result<(), String> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: Some(ControlResult::ClientDelivery(
                    ManagedRuntimeClientDeliveryResponseV1 {
                        response: Some(response),
                    },
                )),
                error_code: String::new(),
            },
        )
        .map_err(|_| "Telegram runtime control response failed".to_owned())
}

fn write_control_error(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    correlation_id: [u8; MANAGED_CONTROL_CORRELATION_ID_BYTES],
    error_code: &str,
) -> Result<(), String> {
    channel
        .write_response(
            correlation_id,
            ManagedRuntimeControlResponseV1 {
                result: None,
                error_code: error_code.to_owned(),
            },
        )
        .map_err(|_| "Telegram runtime control response failed".to_owned())
}

fn authorization_status(
    event: &TdlibAuthorizationEvent,
) -> makosh_telegram_api::TelegramAuthorizationStatus {
    match event {
        TdlibAuthorizationEvent::QrLink(link) => makosh_telegram_api::TelegramAuthorizationStatus {
            state: "waiting_qr_scan".to_owned(),
            qr_link: Some(link.clone()),
            password_hint: None,
        },
        TdlibAuthorizationEvent::State(state) => {
            let (state_name, password_hint) = match state {
                TdlibAuthorizationUpdate::WaitingParameters => ("waiting_parameters", None),
                TdlibAuthorizationUpdate::WaitingEncryptionKey => ("waiting_encryption_key", None),
                TdlibAuthorizationUpdate::WaitingQrScan => ("waiting_qr_scan", None),
                TdlibAuthorizationUpdate::WaitingPassword { hint } => {
                    ("waiting_password", hint.clone())
                }
                TdlibAuthorizationUpdate::Ready => ("ready", None),
                TdlibAuthorizationUpdate::Closing => ("closing", None),
                TdlibAuthorizationUpdate::Closed => ("closed", None),
                TdlibAuthorizationUpdate::Error { .. } => ("error", None),
                TdlibAuthorizationUpdate::Other(_) => ("other", None),
            };
            makosh_telegram_api::TelegramAuthorizationStatus {
                state: state_name.to_owned(),
                qr_link: None,
                password_hint,
            }
        }
    }
}

#[cfg(test)]
mod control_dispatch_tests {
    use std::fs;
    use std::os::unix::net::UnixStream;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    use makosh_runtime_protocol::managed_control::ManagedControlChannelV2;
    use makosh_runtime_protocol::v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlAckV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ModuleClientRequestV1,
        managed_runtime_control_frame_v2::Frame, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    };
    use makosh_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;
    use sha2::{Digest, Sha256};

    use super::{
        TelegramBusyControlDispatcher, TelegramProcessTick, hash_telegram_provider_file,
        provider_tick_needs_idle_pause, telegram_ready_status,
    };

    #[test]
    fn restored_runtime_reports_ready_before_the_first_provider_update() {
        let status = telegram_ready_status();

        assert_eq!(status.state, "ready");
        assert!(status.qr_link.is_none());
        assert!(status.password_hint.is_none());
    }

    #[test]
    fn empty_provider_ticks_are_paced_without_delaying_real_updates() {
        assert!(provider_tick_needs_idle_pause(&TelegramProcessTick::Idle));
        assert!(provider_tick_needs_idle_pause(
            &TelegramProcessTick::Runtime {
                frames: 0,
                provider_cursor: None,
            }
        ));
        assert!(!provider_tick_needs_idle_pause(
            &TelegramProcessTick::Runtime {
                frames: 1,
                provider_cursor: Some("cursor".to_owned()),
            }
        ));
    }

    #[test]
    fn media_hash_worker_reads_the_provider_file_without_loading_it_whole() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "makosh-telegram-media-hash-{}-{nonce}",
            std::process::id()
        ));
        let content = vec![17_u8; 2 * 1024 * 1024 + 37];
        fs::write(&path, &content).expect("write media fixture");

        let result = hash_telegram_provider_file(&path).expect("hash media fixture");
        fs::remove_file(&path).expect("remove media fixture");

        assert_eq!(result.0, content.len() as u64);
        assert_eq!(result.1, <[u8; 32]>::from(Sha256::digest(&content)));
    }

    #[test]
    fn nested_client_delivery_gets_a_correlated_busy_response_without_stealing_platform_reply() {
        let (runtime, kernel) = UnixStream::pair().expect("control pair");
        let kernel = thread::spawn(move || {
            let mut channel = ManagedControlChannelV2::new(kernel);
            let (platform_id, _) = channel.receive_request().expect("platform request");
            channel
                .write_request(
                    [7; MANAGED_CONTROL_CORRELATION_ID_BYTES],
                    ManagedRuntimeControlRequestV1 {
                        operation: Some(Operation::ClientDelivery(
                            ManagedRuntimeClientDeliveryRequestV1 {
                                request: Some(ModuleClientRequestV1 {
                                    protocol_major: 1,
                                    module_id: "telegram".to_owned(),
                                    owner_id: "telegram".to_owned(),
                                    contract: Some(ContractReferenceV1 {
                                        owner: "telegram".to_owned(),
                                        name: "query".to_owned(),
                                        major: 1,
                                        revision: 1,
                                        schema_sha256: vec![1; 32],
                                    }),
                                    request_id: 41,
                                    request_payload: vec![1],
                                    logical_owner_id: String::new(),
                                    authenticated_device_id: String::new(),
                                    authenticated_client_session_id: String::new(),
                                }),
                            },
                        )),
                    },
                )
                .expect("client delivery");
            let nested = channel.read_frame().expect("busy response");
            assert_eq!(
                nested.correlation_id,
                vec![7; MANAGED_CONTROL_CORRELATION_ID_BYTES]
            );
            let Some(Frame::Response(response)) = nested.frame else {
                panic!("nested response");
            };
            let Some(ControlResult::ClientDelivery(delivery)) = response.result else {
                panic!("client delivery response");
            };
            assert_eq!(
                delivery.response.expect("module response").error_code,
                "RUNTIME_BUSY"
            );
            channel
                .write_response(
                    platform_id,
                    ManagedRuntimeControlResponseV1 {
                        result: Some(ControlResult::Ack(ManagedRuntimeControlAckV1 {})),
                        error_code: String::new(),
                    },
                )
                .expect("platform response");
        });

        let mut channel = ManagedControlChannelV2::new(runtime);
        let mut dispatcher = TelegramBusyControlDispatcher;
        let response = channel
            .request_next_with_dispatch(
                ManagedRuntimeControlRequestV1 {
                    operation: Some(Operation::Ready(ManagedRuntimeReadyRequestV1::default())),
                },
                &mut dispatcher,
            )
            .expect("correlated platform response");
        assert!(matches!(response.result, Some(ControlResult::Ack(_))));
        kernel.join().expect("kernel join");
    }
}

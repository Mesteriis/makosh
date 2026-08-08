//! Long-lived Telegram process orchestration around the provider runtime.

use std::os::unix::net::UnixStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hermes_blob_client::{
    BlobDataClient, ManagedBlobCustodyTargetV1, ManagedBlobSessionRequestV1,
    request_managed_blob_session_v2,
};
use hermes_communications_ingress::{
    BodyAdmissionFailureV1, BodyBlobReceiptV1, COMMUNICATIONS_BLOB_CUSTODY_TARGET_CAPABILITY_ID,
    COMMUNICATIONS_BLOB_CUSTODY_TARGET_MODULE_ID, COMMUNICATIONS_BLOB_CUSTODY_TARGET_OWNER_ID,
};
use hermes_runtime_protocol::v1::BlobDataOperationV1;
use hermes_runtime_protocol::{
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
use hermes_telegram_automation_persistence::TelegramAutomationPersistence;
use hermes_telegram_calls_core::{
    TelegramCallDirection, TelegramCallDiscardReason, TelegramCallFailureCategory,
    TelegramCallMediaState, TelegramCallMediaUpdate, TelegramProviderCallState,
    TelegramProviderCallUpdate,
};
use hermes_telegram_calls_persistence::{TelegramCallsPersistence, TelegramCallsPersistenceError};
use hermes_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use hermes_telegram_tdlib::{
    TdJsonTransport, TdlibAuthorizationEvent, TdlibAuthorizationUpdate, TdlibCallDirection,
    TdlibCallDiscardReason, TdlibCallFailureCategory, TdlibCallObservation, TdlibCallState,
    TdlibError,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    TelegramCallProviderUpdate, TelegramDurableProjectionError, TelegramRuntime,
    TelegramRuntimeComposition,
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

pub struct TelegramProcessLoop {
    composition: TelegramRuntimeComposition,
    provider_cursor: Option<String>,
    authorization_status: Option<hermes_telegram_api::TelegramAuthorizationStatus>,
    authorization_status_revision: u64,
    published_authorization_status_revision: u64,
    durable_restore_required: bool,
}

impl TelegramProcessLoop {
    #[must_use]
    pub fn new(composition: TelegramRuntimeComposition) -> Self {
        Self {
            composition,
            provider_cursor: None,
            authorization_status: None,
            authorization_status_revision: 0,
            published_authorization_status_revision: 0,
            durable_restore_required: true,
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
    ) -> Option<&hermes_telegram_api::TelegramAuthorizationStatus> {
        self.authorization_status.as_ref()
    }

    fn update_authorization_status(
        &mut self,
        status: hermes_telegram_api::TelegramAuthorizationStatus,
    ) {
        if self.authorization_status.as_ref() == Some(&status) {
            return;
        }
        self.authorization_status = Some(status);
        self.authorization_status_revision = self.authorization_status_revision.saturating_add(1);
    }

    fn pending_authorization_status_changed(
        &self,
    ) -> Option<(hermes_telegram_api::TelegramAuthorizationStatus, u64)> {
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
        authorization_parameters: hermes_telegram_tdlib::TdlibAuthorizationParameters,
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
            let batch = self
                .composition
                .poll_runtime_events(self.provider_cursor.clone())
                .map_err(TelegramDurableProcessError::Provider)?;
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
            return Ok(TelegramProcessTick::Runtime {
                frames: batch.frames.len() + call_update_count,
                provider_cursor: self.provider_cursor.clone(),
            });
        }
        Ok(TelegramProcessTick::Idle)
    }

    async fn drain_call_media_events(
        &mut self,
        calls: &TelegramCallsPersistence,
        call: &hermes_telegram_calls_core::TelegramCallSession,
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
        poll.map_err(|error| format!("Telegram runtime provider loop failed: {error:?}"))?;
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
                Err(crate::client_realtime::TelegramAuthorizationRealtimeErrorV1::Unavailable) => {}
                Err(
                    crate::client_realtime::TelegramAuthorizationRealtimeErrorV1::InvalidStatus,
                ) => {
                    return Err("Telegram authorization realtime status is invalid".to_owned());
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
                                capability_id: "blob.content",
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
    }
}

fn admit_telegram_plaintext(
    control_channel: &mut ManagedControlChannelV2<UnixStream>,
    plaintext: &[u8],
) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1> {
    if plaintext.is_empty() || plaintext.len() > hermes_telegram_api::MAX_TEXT_BYTES {
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
            capability_id: "blob.content",
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
) -> Result<hermes_telegram_calls_core::TelegramCallSession, TelegramDurableProcessError> {
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
    state: hermes_telegram_call_media_contract::TelegramCallMediaStateV1,
) -> TelegramCallMediaState {
    match state {
        hermes_telegram_call_media_contract::TelegramCallMediaStateV1::Connecting => {
            TelegramCallMediaState::Connecting
        }
        hermes_telegram_call_media_contract::TelegramCallMediaStateV1::Established => {
            TelegramCallMediaState::Active
        }
        hermes_telegram_call_media_contract::TelegramCallMediaStateV1::Reconnecting => {
            TelegramCallMediaState::Reconnecting
        }
        hermes_telegram_call_media_contract::TelegramCallMediaStateV1::Failed => {
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
                Err(error) => ModuleClientResponseV1 {
                    protocol_major: 1,
                    request_id: request.request_id,
                    response_payload: Vec::new(),
                    error_code: client_transport::module_error_code(&error).to_owned(),
                },
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

fn authorize_media_for_request<T: hermes_telegram_tdlib::TdlibTransport>(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    runtime: &mut crate::TelegramRuntime<T>,
    request: &hermes_runtime_protocol::v1::ModuleClientRequestV1,
) -> Result<(), String> {
    let Ok(command) = hermes_telegram_api::client_wire::decode_command(&request.request_payload)
    else {
        return Ok(());
    };
    let hermes_telegram_api::TelegramProviderCommand::SendMedia(media) = command else {
        return Ok(());
    };
    let mut dispatcher = TelegramBusyControlDispatcher;
    let session = request_managed_blob_session_v2(
        channel,
        &mut dispatcher,
        ManagedBlobSessionRequestV1 {
            capability_id: "blob.content",
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
) -> hermes_telegram_api::TelegramAuthorizationStatus {
    match event {
        TdlibAuthorizationEvent::QrLink(link) => hermes_telegram_api::TelegramAuthorizationStatus {
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
            hermes_telegram_api::TelegramAuthorizationStatus {
                state: state_name.to_owned(),
                qr_link: None,
                password_hint,
            }
        }
    }
}

#[cfg(test)]
mod control_dispatch_tests {
    use std::os::unix::net::UnixStream;
    use std::thread;

    use hermes_runtime_protocol::managed_control::ManagedControlChannelV2;
    use hermes_runtime_protocol::v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryRequestV1, ManagedRuntimeControlAckV1,
        ManagedRuntimeControlRequestV1, ManagedRuntimeControlResponseV1,
        ManagedRuntimeReadyRequestV1, ModuleClientRequestV1,
        managed_runtime_control_frame_v2::Frame, managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    };
    use hermes_runtime_protocol::validation::managed_control::MANAGED_CONTROL_CORRELATION_ID_BYTES;

    use super::TelegramBusyControlDispatcher;

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

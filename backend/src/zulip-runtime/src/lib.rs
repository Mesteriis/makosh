//! Managed Zulip integration runtime.
//!
//! This crate composes provider-local HTTP, persistence and the public
//! Communications ingress contract. It never reaches Communications storage.

pub mod admission;
pub mod blob;
pub mod client_port;
mod client_realtime;
mod communications_outbox;
pub mod delivery_intent_consumer;
pub mod delivery_intent_execution;
pub mod delivery_intent_outbox;
pub mod delivery_intent_result;
pub mod delivery_intent_worker;
pub mod managed;

use makosh_communications_ingress::{
    BodyAdmissionFailureV1, BodyAvailabilityV1, BodyBlobReceiptV1, CommunicationEvidenceKindV1,
    CommunicationObservationDraft, ObservationEnvelopeBuildErrorV1, ObservationEnvelopeContextV1,
    build_observation_outbox_record_v1, with_admitted_body_blob, with_body_admission_failure,
};
use makosh_runtime_protocol::v1::BlobDataOperationV1;
use makosh_zulip_api::{
    ZulipCommandOperationStatusV1, ZulipCommandReceiptV1, ZulipCommandV1, ZulipEventQueueV1,
    ZulipPolledEventV1,
    client_contract::ZULIP_MODULE_ID,
    command_account_id, command_fingerprint_bytes, command_operation_id,
    operational::{
        ZulipHistoryStateV1, ZulipOperationalQueryResponseV1, ZulipOperationalQueryV1,
        operational_query_account_id,
    },
    realtime::{ZulipOperationalReplayRequestV1, ZulipOperationalReplayResponseV1},
};
use makosh_zulip_core::{ZulipCoreError, observation_drafts};
use makosh_zulip_http::{
    ZulipHttpConfigV1, ZulipHttpErrorV1, download_user_upload,
    execute_command as execute_http_command, fetch_message_history_page, poll_event_queue,
    register_event_queue, upload_file,
};
use makosh_zulip_persistence::{
    ZulipCommandOperationStateV1, ZulipDeliveryRouteLocatorV1, ZulipDurablePersistence,
    ZulipDurablePersistenceError, ZulipOperationalIngestV1, ZulipQueueCursorV1,
    ZulipQueuedCommandV1,
};
use sha2::{Digest, Sha256};
use std::sync::Mutex;

pub use communications_outbox::{
    ZulipCommunicationsOutboxRelayError, relay_communications_outbox_once,
};

pub const PACKAGE: &str = "makosh-zulip-runtime";

pub mod settings;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipRuntimeIdentityV1 {
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
}

#[derive(Clone)]
pub struct ZulipRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub logical_human_owner_id: String,
    pub configuration_instance_id: String,
    pub module_registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
    pub vault_runtime_generation: u64,
}

#[derive(Debug)]
pub enum ZulipRuntimeErrorV1 {
    Core(ZulipCoreError),
    Envelope(ObservationEnvelopeBuildErrorV1),
    Http(ZulipHttpErrorV1),
    Persistence(ZulipDurablePersistenceError),
    Credential,
    OperationAlreadyKnown,
    CommandFenced,
}

pub struct ZulipClaimedCommandV1 {
    queued: ZulipQueuedCommandV1,
    command: ZulipCommandV1,
}

impl ZulipClaimedCommandV1 {
    pub fn command(&self) -> &ZulipCommandV1 {
        &self.command
    }
}

/// Records a command before any worker may contact the provider.
pub async fn submit_command(
    durable: &ZulipDurablePersistence,
    command: &ZulipCommandV1,
    requested_at_unix_seconds: i64,
) -> Result<ZulipCommandReceiptV1, ZulipRuntimeErrorV1> {
    let command_sha256: [u8; 32] = Sha256::digest(command_fingerprint_bytes(command)).into();
    let operation_id = command_operation_id(command);
    if !durable
        .enqueue_command_operation(
            operation_id,
            command_account_id(command),
            &command_sha256,
            &makosh_zulip_api::client_wire::encode_command_request(command),
            requested_at_unix_seconds,
        )
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)?
    {
        return Err(ZulipRuntimeErrorV1::OperationAlreadyKnown);
    }
    Ok(ZulipCommandReceiptV1 {
        operation_id: operation_id.to_owned(),
        account_id: command_account_id(command).to_owned(),
    })
}

/// Claims and executes at most one previously persisted command. A command is
/// never implicitly retried after the durable dispatch fence has been written.
pub async fn execute_next_command(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
    dispatched_at_unix_seconds: i64,
    completed_at_unix_seconds: i64,
) -> Result<bool, ZulipRuntimeErrorV1> {
    execute_next_command_with_blob(
        durable,
        config,
        None,
        None,
        |_, _| Err(ZulipRuntimeErrorV1::Credential),
        dispatched_at_unix_seconds,
        completed_at_unix_seconds,
    )
    .await
}

pub async fn execute_next_command_with_blob(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
    blob_materializer: Option<
        &Mutex<Option<blob::ZulipBlobMaterializer<makosh_blob_client::BlobDataClient>>>,
    >,
    blob_write_materializer: Option<
        &Mutex<Option<blob::ZulipBlobWriteMaterializer<makosh_blob_client::BlobDataClient>>>,
    >,
    authorize_blob: impl FnMut(&ZulipCommandV1, BlobDataOperationV1) -> Result<(), ZulipRuntimeErrorV1>,
    dispatched_at_unix_seconds: i64,
    completed_at_unix_seconds: i64,
) -> Result<bool, ZulipRuntimeErrorV1> {
    let Some(claimed) = claim_next_command(durable, dispatched_at_unix_seconds).await? else {
        return Ok(false);
    };
    execute_claimed_command_with_blob(
        durable,
        config,
        claimed,
        blob_materializer,
        blob_write_materializer,
        authorize_blob,
        || true,
        completed_at_unix_seconds,
    )
    .await
}

pub async fn claim_next_command(
    durable: &ZulipDurablePersistence,
    dispatched_at_unix_seconds: i64,
) -> Result<Option<ZulipClaimedCommandV1>, ZulipRuntimeErrorV1> {
    let Some(queued) = durable
        .claim_next_command(dispatched_at_unix_seconds)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)?
    else {
        return Ok(None);
    };
    let command =
        makosh_zulip_api::client_wire::decode_command_request(&queued.exact_command_bytes)
            .map_err(|_| {
                ZulipRuntimeErrorV1::Persistence(ZulipDurablePersistenceError::InvalidRow)
            })?;
    let command_sha256: [u8; 32] = Sha256::digest(command_fingerprint_bytes(&command)).into();
    if queued.operation_id != command_operation_id(&command)
        || queued.account_id != command_account_id(&command)
        || queued.command_sha256 != command_sha256
    {
        return Err(ZulipRuntimeErrorV1::Persistence(
            ZulipDurablePersistenceError::InvalidRow,
        ));
    }
    Ok(Some(ZulipClaimedCommandV1 { queued, command }))
}

pub async fn execute_claimed_command_with_blob(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
    claimed: ZulipClaimedCommandV1,
    blob_materializer: Option<
        &Mutex<Option<blob::ZulipBlobMaterializer<makosh_blob_client::BlobDataClient>>>,
    >,
    blob_write_materializer: Option<
        &Mutex<Option<blob::ZulipBlobWriteMaterializer<makosh_blob_client::BlobDataClient>>>,
    >,
    mut authorize_blob: impl FnMut(
        &ZulipCommandV1,
        BlobDataOperationV1,
    ) -> Result<(), ZulipRuntimeErrorV1>,
    fence_is_current: impl Fn() -> bool,
    completed_at_unix_seconds: i64,
) -> Result<bool, ZulipRuntimeErrorV1> {
    if !fence_is_current() {
        return Err(ZulipRuntimeErrorV1::CommandFenced);
    }
    let ZulipClaimedCommandV1 { queued, command } = claimed;
    let (execution, provider_upload_started, completed_blob_ref) = match &command {
        ZulipCommandV1::SendStreamWithUpload {
            stream,
            topic,
            content,
            blob,
            filename,
            ..
        } => {
            authorize_blob(&command, BlobDataOperationV1::BlobDataOperationReadRangeV1)?;
            let bytes = take_blob_bytes(blob_materializer, &blob.blob_ref)?;
            let uri = upload_file(config, filename, &bytes)
                .await
                .map_err(ZulipRuntimeErrorV1::Http)?;
            let send = ZulipCommandV1::SendStream {
                operation_id: command_operation_id(&command).to_owned(),
                account_id: command_account_id(&command).to_owned(),
                stream: stream.clone(),
                topic: topic.clone(),
                content: content_with_upload_uri(content, &uri),
            };
            (execute_http_command(config, &send).await, true, None)
        }
        ZulipCommandV1::SendDirectWithUpload {
            recipients,
            content,
            blob,
            filename,
            ..
        } => {
            authorize_blob(&command, BlobDataOperationV1::BlobDataOperationReadRangeV1)?;
            let bytes = take_blob_bytes(blob_materializer, &blob.blob_ref)?;
            let uri = upload_file(config, filename, &bytes)
                .await
                .map_err(ZulipRuntimeErrorV1::Http)?;
            let send = ZulipCommandV1::SendDirect {
                operation_id: command_operation_id(&command).to_owned(),
                account_id: command_account_id(&command).to_owned(),
                recipients: recipients.clone(),
                content: content_with_upload_uri(content, &uri),
            };
            (execute_http_command(config, &send).await, true, None)
        }
        ZulipCommandV1::DownloadAttachment {
            upload_path, blob, ..
        } => {
            authorize_blob(&command, BlobDataOperationV1::BlobDataOperationWriteV1)?;
            let bytes = download_user_upload(config, upload_path)
                .await
                .map(|(bytes, _)| bytes)
                .map_err(ZulipRuntimeErrorV1::Http)?;
            write_downloaded_blob(blob_write_materializer, &blob.blob_ref, bytes)?;
            (
                Ok(makosh_zulip_http::ZulipHttpResponseV1 {
                    status: 200,
                    provider_message_id: None,
                }),
                true,
                Some(blob.blob_ref.as_str()),
            )
        }
        _ => (execute_http_command(config, &command).await, false, None),
    };
    if !fence_is_current() {
        return Err(ZulipRuntimeErrorV1::CommandFenced);
    }
    match execution {
        Ok(response) => {
            durable
                .complete_command_operation(
                    &queued.operation_id,
                    &queued.command_sha256,
                    ZulipCommandOperationStateV1::Accepted,
                    response.provider_message_id,
                    completed_blob_ref,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(ZulipRuntimeErrorV1::Persistence)?;
            Ok(true)
        }
        Err(error @ (ZulipHttpErrorV1::InvalidCommand | ZulipHttpErrorV1::Rejected))
            if !provider_upload_started =>
        {
            durable
                .complete_command_operation(
                    &queued.operation_id,
                    &queued.command_sha256,
                    ZulipCommandOperationStateV1::Rejected,
                    None,
                    None,
                    completed_at_unix_seconds,
                )
                .await
                .map_err(ZulipRuntimeErrorV1::Persistence)?;
            Err(ZulipRuntimeErrorV1::Http(error))
        }
        Err(error) => Err(ZulipRuntimeErrorV1::Http(error)),
    }
}

fn write_downloaded_blob(
    materializer: Option<
        &Mutex<Option<blob::ZulipBlobWriteMaterializer<makosh_blob_client::BlobDataClient>>>,
    >,
    blob_ref: &str,
    bytes: Vec<u8>,
) -> Result<(), ZulipRuntimeErrorV1> {
    let materializer = materializer.ok_or(ZulipRuntimeErrorV1::Credential)?;
    materializer
        .lock()
        .map_err(|_| ZulipRuntimeErrorV1::Credential)?
        .as_mut()
        .ok_or(ZulipRuntimeErrorV1::Credential)?
        .write_download(blob_ref, bytes)
}

fn take_blob_bytes(
    materializer: Option<
        &Mutex<Option<blob::ZulipBlobMaterializer<makosh_blob_client::BlobDataClient>>>,
    >,
    blob_ref: &str,
) -> Result<Vec<u8>, ZulipRuntimeErrorV1> {
    let materializer = materializer.ok_or(ZulipRuntimeErrorV1::Credential)?;
    materializer
        .lock()
        .map_err(|_| ZulipRuntimeErrorV1::Credential)?
        .as_mut()
        .ok_or(ZulipRuntimeErrorV1::Credential)?
        .take_bytes(blob_ref)
}

fn content_with_upload_uri(content: &str, upload_uri: &str) -> String {
    format!("{content}\n{upload_uri}")
}

pub async fn command_operation_status(
    durable: &ZulipDurablePersistence,
    operation_id: &str,
) -> Result<Option<ZulipCommandOperationStatusV1>, ZulipRuntimeErrorV1> {
    durable
        .command_operation_status(operation_id)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)
}

impl ZulipRuntimeIdentityV1 {
    #[must_use]
    pub fn observation_context(
        &self,
        recorded_at_unix_seconds: i64,
        recorded_at_nanos: i32,
    ) -> ObservationEnvelopeContextV1 {
        ObservationEnvelopeContextV1 {
            runtime_instance_id: self.runtime_instance_id.clone(),
            runtime_generation: self.runtime_generation,
            module_id: ZULIP_MODULE_ID.to_owned(),
            recorded_at_unix_seconds,
            recorded_at_nanos,
        }
    }
}

pub async fn acquire_event_queue(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
) -> Result<ZulipEventQueueV1, ZulipRuntimeErrorV1> {
    match durable
        .current_cursor(&config.account.account_id)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)?
    {
        Some(cursor) => Ok(ZulipEventQueueV1 {
            queue_id: cursor.queue_id,
            last_event_id: cursor.last_event_id,
        }),
        None => register_event_queue(config)
            .await
            .map_err(ZulipRuntimeErrorV1::Http),
    }
}

pub async fn poll_once<F>(
    durable: &ZulipDurablePersistence,
    identity: &ZulipRuntimeIdentityV1,
    config: &ZulipHttpConfigV1,
    queue: &mut ZulipEventQueueV1,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
    body_admitter: &mut F,
) -> Result<usize, ZulipRuntimeErrorV1>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let events = poll_event_queue(config, queue)
        .await
        .map_err(ZulipRuntimeErrorV1::Http)?;
    accept_polled_events(
        durable,
        identity,
        &config.account.account_id,
        queue,
        events,
        recorded_at_unix_seconds,
        recorded_at_nanos,
        body_admitter,
    )
    .await
}

pub async fn accept_polled_events<F>(
    durable: &ZulipDurablePersistence,
    identity: &ZulipRuntimeIdentityV1,
    account_id: &str,
    queue: &mut ZulipEventQueueV1,
    events: Vec<ZulipPolledEventV1>,
    recorded_at_unix_seconds: i64,
    recorded_at_nanos: i32,
    body_admitter: &mut F,
) -> Result<usize, ZulipRuntimeErrorV1>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let mut accepted = 0;
    for event in events {
        if accept_polled_event(
            durable,
            identity,
            ZulipPolledEventContextV1 {
                account_id,
                queue_id: &queue.queue_id,
                event: &event,
                recorded_at_unix_seconds,
                recorded_at_nanos,
            },
            body_admitter,
        )
        .await?
        {
            accepted += 1;
        }
        queue.last_event_id = queue.last_event_id.max(event.event_id);
    }
    Ok(accepted)
}

pub struct ZulipPolledEventContextV1<'a> {
    pub account_id: &'a str,
    pub queue_id: &'a str,
    pub event: &'a ZulipPolledEventV1,
    pub recorded_at_unix_seconds: i64,
    pub recorded_at_nanos: i32,
}

pub async fn accept_polled_event<F>(
    durable: &ZulipDurablePersistence,
    identity: &ZulipRuntimeIdentityV1,
    context: ZulipPolledEventContextV1<'_>,
    body_admitter: &mut F,
) -> Result<bool, ZulipRuntimeErrorV1>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let cursor = ZulipQueueCursorV1 {
        account_id: context.account_id.to_owned(),
        queue_id: context.queue_id.to_owned(),
        last_event_id: context.event.event_id,
    };
    let mut records = Vec::new();
    let mut delivery_route_locators = Vec::new();
    for observation in &context.event.observations {
        for draft in observation_drafts(observation).map_err(ZulipRuntimeErrorV1::Core)? {
            let draft = admit_message_body(draft, observation, body_admitter)?;
            if draft.kind == CommunicationEvidenceKindV1::ChatMessage {
                let scope = draft
                    .source
                    .scope
                    .as_ref()
                    .ok_or(ZulipRuntimeErrorV1::Persistence(
                        ZulipDurablePersistenceError::InvalidRow,
                    ))?;
                let provider_chat_id = scope.external_conversation_id.as_deref().ok_or(
                    ZulipRuntimeErrorV1::Persistence(ZulipDurablePersistenceError::InvalidRow),
                )?;
                delivery_route_locators.push(
                    ZulipDeliveryRouteLocatorV1::new(
                        &scope.external_account_id,
                        provider_chat_id,
                        &draft.source.external_record_id,
                    )
                    .map_err(ZulipRuntimeErrorV1::Persistence)?,
                );
            }
            records.push(
                build_observation_outbox_record_v1(
                    &draft,
                    &identity.observation_context(
                        context.recorded_at_unix_seconds,
                        context.recorded_at_nanos,
                    ),
                )
                .map_err(ZulipRuntimeErrorV1::Envelope)?,
            );
        }
    }
    durable
        .record_operational_events_and_enqueue(&ZulipOperationalIngestV1 {
            cursor: &cursor,
            events: &context.event.observations,
            communications_outbox: &records,
            delivery_route_locators: &delivery_route_locators,
            observed_at_unix_seconds: context.recorded_at_unix_seconds,
        })
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)
}

pub async fn sync_history_page(
    durable: &ZulipDurablePersistence,
    config: &ZulipHttpConfigV1,
    observed_at_unix_seconds: i64,
) -> Result<bool, ZulipRuntimeErrorV1> {
    let status = durable
        .execute_operational_query(&ZulipOperationalQueryV1::GetAccountStatus {
            account_id: config.account.account_id.clone(),
        })
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)?;
    let ZulipOperationalQueryResponseV1::AccountStatus(status) = status else {
        return Err(ZulipRuntimeErrorV1::Persistence(
            ZulipDurablePersistenceError::InvalidRow,
        ));
    };
    if status.history_state == ZulipHistoryStateV1::Ready {
        return Ok(false);
    }
    let page =
        fetch_message_history_page(config, status.oldest_provider_message_id.as_deref(), 100)
            .await
            .map_err(ZulipRuntimeErrorV1::Http)?;
    durable
        .record_history_page(&config.account.account_id, &page, observed_at_unix_seconds)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)?;
    Ok(true)
}

pub async fn execute_operational_query(
    durable: &ZulipDurablePersistence,
    configured_account_id: &str,
    query: &ZulipOperationalQueryV1,
) -> Result<ZulipOperationalQueryResponseV1, ZulipRuntimeErrorV1> {
    if operational_query_account_id(query) != configured_account_id {
        return Err(ZulipRuntimeErrorV1::Persistence(
            ZulipDurablePersistenceError::InvalidRow,
        ));
    }
    durable
        .execute_operational_query(query)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)
}

pub async fn replay_operational_events(
    durable: &ZulipDurablePersistence,
    configured_account_id: &str,
    request: &ZulipOperationalReplayRequestV1,
) -> Result<ZulipOperationalReplayResponseV1, ZulipRuntimeErrorV1> {
    if request.account_id != configured_account_id {
        return Err(ZulipRuntimeErrorV1::Persistence(
            ZulipDurablePersistenceError::InvalidRow,
        ));
    }
    durable
        .replay_operational_events(request)
        .await
        .map_err(ZulipRuntimeErrorV1::Persistence)
}

fn admit_message_body<F>(
    draft: CommunicationObservationDraft,
    event: &makosh_zulip_api::ZulipEventV1,
    body_admitter: &mut F,
) -> Result<CommunicationObservationDraft, ZulipRuntimeErrorV1>
where
    F: FnMut(&[u8]) -> Result<BodyBlobReceiptV1, BodyAdmissionFailureV1>,
{
    let makosh_zulip_api::ZulipEventV1::Message {
        content: Some(content),
        ..
    } = event
    else {
        return Ok(draft);
    };
    if draft.body != BodyAvailabilityV1::Unavailable {
        return Ok(draft);
    }
    if content.trim().is_empty() || content.len() > 256 * 1024 {
        return with_body_admission_failure(draft, BodyAdmissionFailureV1::SizeLimitExceeded)
            .map_err(|_| ZulipRuntimeErrorV1::Core(ZulipCoreError::InvalidEvent));
    }
    match body_admitter(content.as_bytes()) {
        Ok(receipt) => {
            let mut admitted = draft;
            admitted.body = BodyAvailabilityV1::AdmittedBlob;
            with_admitted_body_blob(admitted, receipt)
                .map_err(|_| ZulipRuntimeErrorV1::Core(ZulipCoreError::InvalidEvent))
        }
        Err(failure) => with_body_admission_failure(draft, failure)
            .map_err(|_| ZulipRuntimeErrorV1::Core(ZulipCoreError::InvalidEvent)),
    }
}

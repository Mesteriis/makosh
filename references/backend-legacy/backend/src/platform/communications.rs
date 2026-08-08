use makosh_communications_api::accounts::ProviderAccount;
use makosh_communications_api::errors::CommunicationContractError;
use makosh_communications_api::provider_messages::{
    ProviderAttachmentDownloadStateUpdate, ProviderChannelMessage, ProviderHeuristicMember,
    ProviderMessageAttachmentAnchor, ProviderMessageObservationEvent,
    ProviderMessageProjectionObservationContext, ProviderMessageReferenceSummary,
};
use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::platform::secrets::models::ResolvedSecret;
pub mod errors;
use errors::ProviderCommunicationMessagePortError;

pub mod attachment_text;
pub mod email_sync;
pub mod mbox;
pub mod raw_signals;
pub mod rfc822;

pub const DEFAULT_MAIL_SYNC_BLOB_ROOT: &str = "docker/data/mail";

pub type ProviderChannelMessagePortFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderCommunicationMessagePortError>> + Send + 'a>>;

pub type ProviderMessageObservationEventFuture<'a> = Pin<
    Box<
        dyn Future<Output = Result<Option<i64>, ProviderCommunicationMessagePortError>> + Send + 'a,
    >,
>;

pub trait ProviderMessageObservationEventPort: Send + Sync {
    fn append_provider_message_observation<'a>(
        &'a self,
        observation: ProviderMessageObservationEvent<'a>,
    ) -> ProviderMessageObservationEventFuture<'a>;
}

#[derive(Clone)]
pub struct EventStoreProviderMessageObservationEventPort {
    event_store: makosh_events_postgres::store::EventStore,
}

impl EventStoreProviderMessageObservationEventPort {
    pub fn new(pool: sqlx::postgres::PgPool) -> Self {
        Self {
            event_store: makosh_events_postgres::store::EventStore::new(pool),
        }
    }
}

impl ProviderMessageObservationEventPort for EventStoreProviderMessageObservationEventPort {
    fn append_provider_message_observation<'a>(
        &'a self,
        observation: ProviderMessageObservationEvent<'a>,
    ) -> ProviderMessageObservationEventFuture<'a> {
        Box::pin(async move {
            validate_provider_observation_event(&observation).map_err(|error| {
                ProviderCommunicationMessagePortError::InvalidRequest(error.to_string())
            })?;
            let payload_hash = sha256_json(observation.payload)?;
            let idempotency_key = provider_observation_idempotency_key(
                observation.provider,
                observation.account_id,
                observation.event_kind,
                observation.external_message_id,
                observation.external_event_id,
                &payload_hash,
            );
            let event_type =
                provider_observation_event_type(observation.provider, observation.event_kind);
            let builder = makosh_events_api::NewEventEnvelope::builder(
                format!(
                    "evt_provider_observation_{}",
                    stable_event_id_fragment(&idempotency_key)
                ),
                event_type,
                observation.observed_at,
                json!({
                    "kind": "provider_observation",
                    "provider": observation.provider,
                    "account_id": observation.account_id,
                    "source_id": idempotency_key,
                }),
                json!({
                    "kind": "provider_message",
                    "provider": observation.provider,
                    "id": observation.external_message_id,
                    "message_id": observation.message_id,
                }),
            )
            .payload(json!({
                "provider_kind": observation.channel_kind,
                "account_id": observation.account_id,
                "external_event_id": observation.external_event_id,
                "external_message_id": observation.external_message_id,
                "message_id": observation.message_id,
                "event_kind": observation.event_kind,
                "observed_at": observation.observed_at,
                "payload_hash": payload_hash,
                "payload": observation.payload,
            }))
            .provenance(json!({
                "provider": observation.provider,
                "ownership": "provider_observation_fact",
            }));
            let correlation_id = observation
                .correlation_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(&idempotency_key);
            let mut builder = builder.correlation_id(correlation_id);
            if let Some(causation_id) = observation.causation_id {
                builder = builder.causation_id(causation_id);
            }
            let event = builder.build()?;

            self.event_store
                .append_for_dispatch_idempotent(&event)
                .await
                .map_err(Into::into)
        })
    }
}

use makosh_communications_api::email::{EmailSendError, OutgoingEmail, SendResult, SmtpConfig};

pub trait SmtpTransport: Clone + Send + Sync {
    fn send<'a>(
        &'a self,
        config: &'a SmtpConfig,
        password: &'a ResolvedSecret,
        email: &'a OutgoingEmail,
    ) -> Pin<Box<dyn Future<Output = Result<SendResult, EmailSendError>> + Send + 'a>>;
}

pub trait ProviderChannelMessageLookupPort: Send + Sync {
    fn message_by_id<'a>(
        &'a self,
        message_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn message_by_provider_record_id<'a>(
        &'a self,
        account_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn recent_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        conversation_id: Option<&'a str>,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn messages_by_ids<'a>(
        &'a self,
        message_ids: &'a [String],
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn search_messages<'a>(
        &'a self,
        account_id: Option<&'a str>,
        conversation_id: Option<&'a str>,
        query: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn pinned_messages<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn body_text<'a>(
        &'a self,
        message_id: &'a str,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<String>, ProviderCommunicationMessagePortError>>
                + Send
                + 'a,
        >,
    >;

    fn message_ids_by_metadata_string<'a>(
        &'a self,
        metadata_key: &'a str,
        metadata_value: &'a str,
        channel_kinds: &'a [&'a str],
        limit: i64,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Vec<String>, ProviderCommunicationMessagePortError>>
                + Send
                + 'a,
        >,
    >;

    fn message_id_by_provider_record_id<'a>(
        &'a self,
        account_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<String>, ProviderCommunicationMessagePortError>>
                + Send
                + 'a,
        >,
    >;

    fn reference_summaries<'a>(
        &'a self,
        message_ids: &'a [String],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderMessageReferenceSummary>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn heuristic_members<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        query: Option<&'a str>,
        channel_kinds: &'a [&'a str],
        limit: i64,
        offset: i64,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<ProviderHeuristicMember>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn attachment_anchor<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        provider_record_id: &'a str,
        channel_kinds: &'a [&'a str],
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderMessageAttachmentAnchor>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn unread_counts<'a>(
        &'a self,
        account_id: &'a str,
        conversation_id: &'a str,
        channel_kinds: &'a [&'a str],
        last_read_at: Option<DateTime<Utc>>,
    ) -> ProviderChannelMessagePortFuture<'a, (i64, i64)>;
}

pub trait ProviderChannelMessageCommandPort: ProviderChannelMessageLookupPort {
    fn apply_metadata<'a>(
        &'a self,
        message_id: &'a str,
        metadata: &'a Value,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn set_delivery_state<'a>(
        &'a self,
        message_id: &'a str,
        delivery_state: &'a str,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn apply_content_update<'a>(
        &'a self,
        message_id: &'a str,
        body_text: &'a str,
        metadata: &'a Value,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn apply_pinned_state<'a>(
        &'a self,
        message_id: &'a str,
        is_pinned: bool,
        observed_at: DateTime<Utc>,
        context: ProviderMessageProjectionObservationContext<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<ProviderChannelMessage>,
                        ProviderCommunicationMessagePortError,
                    >,
                > + Send
                + 'a,
        >,
    >;

    fn update_attachment_download_state<'a>(
        &'a self,
        update: ProviderAttachmentDownloadStateUpdate<'a>,
    ) -> ProviderChannelMessagePortFuture<'a, Option<ProviderChannelMessage>>;
}

fn validate_provider_observation_event(
    observation: &ProviderMessageObservationEvent<'_>,
) -> Result<(), CommunicationContractError> {
    validate_non_empty("provider", observation.provider)?;
    validate_non_empty("account_id", observation.account_id)?;
    validate_non_empty("channel_kind", observation.channel_kind)?;
    validate_non_empty("message_id", observation.message_id)?;
    validate_non_empty("external_message_id", observation.external_message_id)?;
    validate_non_empty("event_kind", observation.event_kind)?;
    validate_object("payload", observation.payload)
}

fn provider_observation_event_type(provider: &str, event_kind: &str) -> String {
    if provider == "telegram" {
        return match event_kind {
            "content_observed" => "signal.raw.telegram.message.content.observed".to_owned(),
            "metadata_observed" => "signal.raw.telegram.message.metadata.observed".to_owned(),
            "delivery_state_observed" => {
                "signal.raw.telegram.message.delivery_state.observed".to_owned()
            }
            "provider_identity_observed" => {
                "signal.raw.telegram.message.provider_identity.observed".to_owned()
            }
            "pinned_state_observed" => {
                "signal.raw.telegram.message.pinned_state.observed".to_owned()
            }
            "attachment_download_state_observed" => {
                "signal.raw.telegram.attachment.download_state.observed".to_owned()
            }
            other => format!("signal.raw.telegram.message.{other}.observed"),
        };
    }
    format!("integration.{provider}.message.{event_kind}")
}

fn provider_observation_idempotency_key(
    provider: &str,
    account_id: &str,
    event_kind: &str,
    external_message_id: &str,
    external_event_id: Option<&str>,
    payload_hash: &str,
) -> String {
    if let Some(external_event_id) = external_event_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("{provider}:{account_id}:external_event:{external_event_id}");
    }
    format!("{provider}:{account_id}:{event_kind}:{external_message_id}:{payload_hash}")
}

fn stable_event_id_fragment(value: &str) -> String {
    value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char
            } else {
                '_'
            }
        })
        .collect()
}

fn sha256_json(value: &Value) -> Result<String, ProviderCommunicationMessagePortError> {
    let encoded = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), CommunicationContractError> {
    if value.trim().is_empty() {
        Err(CommunicationContractError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), CommunicationContractError> {
    if value.is_object() {
        Ok(())
    } else {
        Err(CommunicationContractError::NonObjectJson(field))
    }
}

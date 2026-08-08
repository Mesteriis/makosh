use chrono::Utc;
use makosh_events_api::{EventEnvelope, NewEventEnvelope, StoredEventEnvelope};
use makosh_signal_hub_api::raw_signals::RawSignalPersistencePort;
use makosh_signal_hub_postgres::raw_signals::adapter::RawSignalStore;
use serde_json::{Value, json};
use sqlx::postgres::PgPool;
use std::sync::Arc;

use super::policies::{SignalPolicyDecision, SignalPolicyEvaluator};
use super::store::{SignalHubError, SignalHubStore};
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

pub const SIGNAL_HUB_RAW_SIGNAL_CONSUMER: &str = "signal_hub_raw_signal_dispatcher";
const SIGNAL_HUB_RAW_SIGNAL_RUNTIME_SOURCE: &str = "system";

pub async fn process_signal_hub_raw_event(
    pool: PgPool,
    event: StoredEventEnvelope,
) -> Result<(), EventStoreError> {
    if !event.event.event_type.starts_with("signal.raw.") {
        return Ok(());
    }

    let service = SignalHubSignalService::new(
        Arc::new(RawSignalStore::new(pool.clone())),
        EventStore::new(pool),
    );
    service
        .process_raw_signal(&event.event)
        .await
        .map(|_| ())
        .map_err(|error| EventStoreError::ConsumerHandlerFailed(error.to_string()))
}

pub async fn signal_hub_raw_dispatcher_allows_processing(
    signal_store: &SignalHubStore,
) -> Result<bool, SignalHubError> {
    signal_store.restore_system_sources().await?;
    Ok(crate::platform::events::runtime::runtime_allows_processing(
        signal_store.pool(),
        SIGNAL_HUB_RAW_SIGNAL_RUNTIME_SOURCE,
        SIGNAL_HUB_RAW_SIGNAL_CONSUMER,
        &json!({
            "label": "Signal Hub raw signal dispatcher",
            "scope": "consumer",
        }),
    )
    .await?)
}

#[derive(Clone)]
pub struct SignalHubSignalService {
    raw_signal_store: Arc<dyn RawSignalPersistencePort>,
    event_store: EventStore,
}

impl SignalHubSignalService {
    pub fn new(
        raw_signal_store: Arc<dyn RawSignalPersistencePort>,
        event_store: EventStore,
    ) -> Self {
        Self {
            raw_signal_store,
            event_store,
        }
    }

    pub async fn process_raw_signal(
        &self,
        raw_event: &EventEnvelope,
    ) -> Result<SignalProcessingOutcome, SignalHubError> {
        let parsed = ParsedRawSignal::parse(raw_event)?;
        let connection_id = self
            .raw_signal_store
            .resolve_connection_id(&parsed.source_code, raw_event)
            .await?;
        let policies = self.raw_signal_store.list_active_policies().await?;
        let decision = SignalPolicyEvaluator::new(Utc::now()).decide(
            &parsed.source_code,
            connection_id.as_deref(),
            &raw_event.event_type,
            &policies,
        );

        match decision {
            SignalPolicyDecision::Allow => {
                let accepted = build_derived_event(
                    raw_event,
                    &format!(
                        "signal.accepted.{}.{}",
                        parsed.source_code, parsed.event_kind
                    ),
                    signal_decision_payload("accepted", None, connection_id.as_deref()),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&accepted)
                    .await?;
                Ok(SignalProcessingOutcome::Accepted {
                    event_id: accepted.event_id,
                })
            }
            SignalPolicyDecision::Rejected { reason } => {
                let rejected = build_derived_event(
                    raw_event,
                    &format!(
                        "signal.rejected.{}.{}",
                        parsed.source_code, parsed.event_kind
                    ),
                    signal_decision_payload(
                        "rejected",
                        Some(reason.as_str()),
                        connection_id.as_deref(),
                    ),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&rejected)
                    .await?;
                Ok(SignalProcessingOutcome::Rejected { reason })
            }
            SignalPolicyDecision::Muted { reason } => {
                let muted = build_derived_event(
                    raw_event,
                    &format!("signal.muted.{}.{}", parsed.source_code, parsed.event_kind),
                    signal_decision_payload(
                        "muted",
                        Some(reason.as_str()),
                        connection_id.as_deref(),
                    ),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&muted)
                    .await?;
                Ok(SignalProcessingOutcome::Muted { reason })
            }
            SignalPolicyDecision::Paused { reason } => {
                self.raw_signal_store
                    .record_paused_event(
                        raw_event,
                        &parsed.source_code,
                        connection_id.as_deref(),
                        &reason,
                    )
                    .await?;
                let paused = build_derived_event(
                    raw_event,
                    &format!("signal.paused.{}.{}", parsed.source_code, parsed.event_kind),
                    signal_decision_payload(
                        "paused",
                        Some(reason.as_str()),
                        connection_id.as_deref(),
                    ),
                )?;
                self.event_store
                    .append_for_dispatch_idempotent(&paused)
                    .await?;
                Ok(SignalProcessingOutcome::Paused { reason })
            }
        }
    }

    pub async fn replay_raw_signal(
        &self,
        raw_event: &EventEnvelope,
    ) -> Result<SignalProcessingOutcome, SignalHubError> {
        let parsed = ParsedRawSignal::parse(raw_event)?;
        let connection_id = self
            .raw_signal_store
            .resolve_connection_id(&parsed.source_code, raw_event)
            .await?;
        let accepted = build_derived_event(
            raw_event,
            &format!(
                "signal.accepted.{}.{}",
                parsed.source_code, parsed.event_kind
            ),
            signal_decision_payload("replayed", None, connection_id.as_deref()),
        )?;
        self.event_store
            .append_for_dispatch_idempotent(&accepted)
            .await?;
        Ok(SignalProcessingOutcome::Accepted {
            event_id: accepted.event_id,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SignalProcessingOutcome {
    Accepted { event_id: String },
    Rejected { reason: String },
    Muted { reason: String },
    Paused { reason: String },
}

struct ParsedRawSignal {
    source_code: String,
    event_kind: String,
}

impl ParsedRawSignal {
    fn parse(raw_event: &EventEnvelope) -> Result<Self, SignalHubError> {
        let parts: Vec<&str> = raw_event.event_type.split('.').collect();
        if parts.len() < 5 || parts[0] != "signal" || parts[1] != "raw" {
            return Err(SignalHubError::InvalidRawSignalEventType(
                raw_event.event_type.clone(),
            ));
        }
        if parts.last() != Some(&"observed") {
            return Err(SignalHubError::InvalidRawSignalEventType(
                raw_event.event_type.clone(),
            ));
        }

        let source_code = source_code_from_value(&raw_event.source)
            .or_else(|| parts.get(2).map(|value| (*value).to_owned()))
            .ok_or(SignalHubError::MissingSourceCode)?;
        let event_kind = parts[3..parts.len() - 1].join(".");
        if event_kind.trim().is_empty() {
            return Err(SignalHubError::InvalidRawSignalEventType(
                raw_event.event_type.clone(),
            ));
        }

        Ok(Self {
            source_code,
            event_kind,
        })
    }
}

fn source_code_from_value(value: &Value) -> Option<String> {
    value
        .get("source_code")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|source_code| !source_code.is_empty())
        .map(ToOwned::to_owned)
}

fn signal_decision_payload(
    decision: &str,
    reason: Option<&str>,
    connection_id: Option<&str>,
) -> Value {
    let mut payload = json!({
        "decision": decision,
    });

    if let Some(reason) = reason {
        payload["reason"] = json!(reason);
    }

    if let Some(connection_id) = connection_id {
        payload["connection_id"] = json!(connection_id);
    }

    payload
}

fn build_derived_event(
    raw_event: &EventEnvelope,
    event_type: &str,
    decision: Value,
) -> Result<NewEventEnvelope, SignalHubError> {
    let event_id = format!("{}_{}", event_type.replace('.', "_"), raw_event.event_id);
    let event = NewEventEnvelope::builder(
        event_id,
        event_type,
        Utc::now(),
        raw_event.source.clone(),
        raw_event.subject.clone(),
    )
    .payload(raw_event.payload.clone())
    .provenance(json!({
        "signal_hub": decision,
        "raw_event_id": raw_event.event_id,
        "raw_event_provenance": raw_event.provenance,
    }))
    .causation_id(raw_event.event_id.clone());

    let event = match &raw_event.correlation_id {
        Some(correlation_id) => event.correlation_id(correlation_id.clone()),
        None => event,
    };

    Ok(event.build()?)
}

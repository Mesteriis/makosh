use makosh_events_api::NewEventEnvelope;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::store::{SignalHubError, SignalHubStore};
use makosh_events_postgres::store::EventStore;

const TEST_SIGNAL_FIXTURES_TOML: &str =
    include_str!("../../../fixtures/signal_hub/test_signals.toml");

static TEST_SIGNAL_FIXTURE_CATALOG: LazyLock<Result<SignalFixtureCatalog, toml::de::Error>> =
    LazyLock::new(|| toml::from_str(TEST_SIGNAL_FIXTURES_TOML));

#[derive(Clone)]
pub struct SignalFixtureSourceService {
    signal_store: SignalHubStore,
    event_store: EventStore,
}

impl SignalFixtureSourceService {
    pub fn new(signal_store: SignalHubStore, event_store: EventStore) -> Self {
        Self {
            signal_store,
            event_store,
        }
    }

    pub async fn emit_fixture(
        &self,
        request: &SignalFixtureEmitRequest,
    ) -> Result<SignalFixtureEmission, SignalHubError> {
        let fixture_id = request.fixture_id.trim();
        if fixture_id.is_empty() {
            return Err(SignalHubError::EmptyField("fixture_id"));
        }

        let fixture = fixture_signal_by_id(fixture_id)?;
        self.ensure_source_exists(&fixture.source_code).await?;

        let raw_signal = build_fixture_raw_signal(fixture)?;
        self.event_store
            .append_for_dispatch_idempotent(&raw_signal)
            .await?;

        Ok(SignalFixtureEmission {
            fixture_id: fixture.fixture_id.to_owned(),
            raw_event_id: raw_signal.event_id,
            event_type: fixture.event_type.to_owned(),
            source_code: fixture.source_code.to_owned(),
            correlation_id: fixture.correlation_id.as_deref().map(ToOwned::to_owned),
        })
    }

    pub fn list_fixture_sources(&self) -> Result<Vec<SignalFixtureSource>, SignalHubError> {
        let catalog = fixture_catalog()?;
        Ok(catalog
            .fixtures
            .iter()
            .map(|fixture| SignalFixtureSource {
                fixture_id: fixture.fixture_id.clone(),
                source_code: fixture.source_code.clone(),
                event_type: fixture.event_type.clone(),
                correlation_id: fixture.correlation_id.clone(),
                occurred_at: fixture.occurred_at,
                summary: fixture_summary(fixture),
            })
            .collect())
    }

    async fn ensure_source_exists(&self, source_code: &str) -> Result<(), SignalHubError> {
        let exists = self
            .signal_store
            .list_sources()
            .await?
            .into_iter()
            .any(|source| source.code == source_code);
        if exists {
            Ok(())
        } else {
            Err(SignalHubError::SourceNotFound(source_code.to_owned()))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalFixtureEmitRequest {
    pub fixture_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalFixtureEmission {
    pub fixture_id: String,
    pub raw_event_id: String,
    pub event_type: String,
    pub source_code: String,
    pub correlation_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalFixtureSource {
    pub fixture_id: String,
    pub source_code: String,
    pub event_type: String,
    pub correlation_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize)]
struct SignalFixtureCatalog {
    schema_version: u32,
    fixtures: Vec<SignalFixtureDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
struct SignalFixtureDefinition {
    fixture_id: String,
    #[serde(rename = "source")]
    source_code: String,
    event_type: String,
    source_id: String,
    subject_kind: String,
    subject_entity_id: String,
    occurred_at: DateTime<Utc>,
    correlation_id: Option<String>,
    payload: Value,
    #[serde(default = "empty_object")]
    provenance: Value,
}

fn fixture_signal_by_id(
    fixture_id: &str,
) -> Result<&'static SignalFixtureDefinition, SignalHubError> {
    let catalog = fixture_catalog()?;

    if catalog.schema_version != 1 {
        return Err(SignalHubError::InvalidFixtureCatalog(format!(
            "unsupported test signal fixture schema version: {}",
            catalog.schema_version
        )));
    }

    catalog
        .fixtures
        .iter()
        .find(|fixture| fixture.fixture_id == fixture_id)
        .ok_or_else(|| SignalHubError::FixtureNotFound(fixture_id.to_owned()))
}

fn fixture_catalog() -> Result<&'static SignalFixtureCatalog, SignalHubError> {
    TEST_SIGNAL_FIXTURE_CATALOG.as_ref().map_err(|error| {
        SignalHubError::InvalidFixtureCatalog(format!(
            "failed to parse test signal fixtures: {error}"
        ))
    })
}

fn build_fixture_raw_signal(
    fixture: &SignalFixtureDefinition,
) -> Result<NewEventEnvelope, SignalHubError> {
    let event = NewEventEnvelope::builder(
        fixture_raw_event_id(fixture),
        &fixture.event_type,
        fixture.occurred_at,
        json!({
            "kind": "signal_source",
            "source_code": fixture.source_code,
            "source_id": fixture.source_id,
            "runtime_kind": "fixture",
            "fixture_id": fixture.fixture_id,
        }),
        json!({
            "kind": fixture.subject_kind,
            "source_code": fixture.source_code,
            "entity_id": fixture.subject_entity_id,
            "fixture_id": fixture.fixture_id,
        }),
    )
    .payload(fixture.payload.clone())
    .provenance(build_fixture_provenance(fixture)?);

    let event = match fixture.correlation_id.as_deref() {
        Some(correlation_id) if !correlation_id.trim().is_empty() => {
            event.correlation_id(correlation_id.trim().to_owned())
        }
        _ => event,
    };

    Ok(event.build()?)
}

fn build_fixture_provenance(fixture: &SignalFixtureDefinition) -> Result<Value, SignalHubError> {
    let mut provenance = match fixture.provenance.clone() {
        Value::Object(map) => map,
        other => {
            return Err(SignalHubError::InvalidFixtureCatalog(format!(
                "fixture provenance must be an object for {} but was {}",
                fixture.fixture_id, other
            )));
        }
    };
    provenance.insert("source".to_owned(), json!("signal_fixture_catalog"));
    provenance.insert("fixture_id".to_owned(), json!(fixture.fixture_id));
    provenance.insert("runtime_kind".to_owned(), json!("fixture"));
    provenance.insert("source_code".to_owned(), json!(fixture.source_code));

    Ok(Value::Object(provenance))
}

fn fixture_raw_event_id(fixture: &SignalFixtureDefinition) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fixture.fixture_id.as_bytes());
    hasher.update([0]);
    hasher.update(fixture.source_code.as_bytes());
    hasher.update([0]);
    hasher.update(fixture.event_type.as_bytes());
    format!("evt_signal_fixture_{:x}", hasher.finalize())
}

fn empty_object() -> Value {
    json!({})
}

fn fixture_summary(fixture: &SignalFixtureDefinition) -> String {
    fixture
        .payload
        .get("summary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| fixture.event_type.clone())
}

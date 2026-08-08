use makosh_events_api::{EventEnvelope, NewEventEnvelopeBuilder, StoredEventEnvelope};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceContext {
    pub correlation_id: String,
    pub causation_id: Option<String>,
}

impl TraceContext {
    pub fn root(root_id: impl Into<String>) -> Self {
        Self {
            correlation_id: root_id.into(),
            causation_id: None,
        }
    }

    pub fn child_of(parent: &EventEnvelope) -> Self {
        Self {
            correlation_id: parent
                .correlation_id
                .clone()
                .unwrap_or_else(|| parent.event_id.clone()),
            causation_id: Some(parent.event_id.clone()),
        }
    }

    pub fn child_of_stored(parent: &StoredEventEnvelope) -> Self {
        Self::child_of(&parent.event)
    }

    pub fn apply(self, builder: NewEventEnvelopeBuilder) -> NewEventEnvelopeBuilder {
        let builder = builder.correlation_id(self.correlation_id);
        match self.causation_id {
            Some(causation_id) => builder.causation_id(causation_id),
            None => builder,
        }
    }
}

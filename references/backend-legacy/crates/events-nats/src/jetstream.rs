use thiserror::Error;

use makosh_events_api::EventEnvelope;

const MAKOSH_EVENTS_STREAM: &str = "makosh_events";

#[derive(Clone)]
pub struct NatsJetStreamEventBus {
    context: async_nats::jetstream::Context,
}

impl NatsJetStreamEventBus {
    pub async fn connect(server_url: &str) -> Result<Self, NatsJetStreamEventBusError> {
        let client = async_nats::connect(server_url)
            .await
            .map_err(|error| NatsJetStreamEventBusError::Connect(error.to_string()))?;
        let context = async_nats::jetstream::new(client);
        context
            .get_or_create_stream(async_nats::jetstream::stream::Config {
                name: MAKOSH_EVENTS_STREAM.to_owned(),
                subjects: vec!["signal.>".to_owned()],
                ..Default::default()
            })
            .await
            .map_err(|error| NatsJetStreamEventBusError::Stream(error.to_string()))?;

        Ok(Self { context })
    }

    pub async fn publish(&self, event: &EventEnvelope) -> Result<(), NatsJetStreamEventBusError> {
        let payload = serde_json::to_vec(event)?;
        let ack = self
            .context
            .publish(event_subject(event), payload.into())
            .await
            .map_err(|error| NatsJetStreamEventBusError::Publish(error.to_string()))?;
        ack.await
            .map_err(|error| NatsJetStreamEventBusError::PublishAck(error.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum NatsJetStreamEventBusError {
    #[error("failed to connect to NATS JetStream: {0}")]
    Connect(String),

    #[error("failed to ensure JetStream event stream: {0}")]
    Stream(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("failed to publish NATS JetStream event: {0}")]
    Publish(String),

    #[error("failed to confirm NATS JetStream publish: {0}")]
    PublishAck(String),
}

fn event_subject(event: &EventEnvelope) -> String {
    event.event_type.clone()
}

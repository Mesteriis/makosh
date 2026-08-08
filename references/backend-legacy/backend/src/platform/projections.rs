use makosh_events_api::StoredEventEnvelope;
use std::future::Future;

use thiserror::Error;

use makosh_events_postgres::cursors::ProjectionCursorStore;
use makosh_events_postgres::errors::EventStoreError;
use makosh_events_postgres::store::EventStore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionBatchOutcome {
    pub processed_count: usize,
    pub last_processed_position: i64,
}

pub async fn run_projection_batch<F, Fut>(
    events: &EventStore,
    cursors: &ProjectionCursorStore,
    projection_name: &str,
    batch_size: u32,
    mut handler: F,
) -> Result<ProjectionBatchOutcome, ProjectionRunnerError>
where
    F: FnMut(StoredEventEnvelope) -> Fut,
    Fut: Future<Output = Result<(), ProjectionHandlerError>>,
{
    if batch_size == 0 {
        return Err(ProjectionRunnerError::InvalidBatchSize);
    }

    let start_position = cursors.last_processed_position(projection_name).await?;
    let batch = events
        .list_after_position(start_position, batch_size)
        .await?;

    let mut processed_count = 0;
    let mut last_processed_position = start_position;

    for event in batch {
        let position = event.position;
        handler(event).await?;
        last_processed_position = cursors.save_position(projection_name, position).await?;
        processed_count += 1;
    }

    Ok(ProjectionBatchOutcome {
        processed_count,
        last_processed_position,
    })
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct ProjectionHandlerError {
    message: String,
}

impl ProjectionHandlerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProjectionRunnerError {
    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Handler(#[from] ProjectionHandlerError),

    #[error("projection batch size must be greater than zero")]
    InvalidBatchSize,
}

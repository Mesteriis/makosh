use chrono::Utc;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;
use serde_json::{Value, json};

use super::command_service::{CommunicationCommandService, CommunicationCommandServiceError};

impl CommunicationCommandService {
    pub(super) async fn capture_observation(
        &self,
        operation: &'static str,
        kind: &'static str,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<makosh_observations_api::models::Observation, CommunicationCommandServiceError>
    {
        ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::Manual,
                    Utc::now(),
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await
            .map_err(
                |source| CommunicationCommandServiceError::ObservationCapture { operation, source },
            )
    }

    pub(super) async fn capture_message_flag_observation(
        &self,
        message_id: &str,
        operation: &'static str,
        payload: Value,
    ) -> Result<makosh_observations_api::models::Observation, CommunicationCommandServiceError>
    {
        self.capture_observation(
            "message flag action",
            "COMMUNICATION_MESSAGE",
            json!({
                "message_id": message_id,
                "operation": operation,
                "payload": payload,
            }),
            format!("message://{message_id}/{operation}"),
            json!({
                "captured_by": "mail_service.message_flags",
                "operation": operation,
            }),
        )
        .await
    }
}

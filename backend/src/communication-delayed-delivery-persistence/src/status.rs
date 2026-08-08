use crate::{
    CommunicationDelayedDeliveryPersistenceV1, DelayedDeliveryOperationStatusV1,
    DelayedDeliveryPersistenceErrorV1,
    operations::{status_from_row, valid_owner},
    valid_id16,
};

impl CommunicationDelayedDeliveryPersistenceV1 {
    pub async fn status(
        &self,
        logical_owner_id: &str,
        delayed_operation_id: &[u8; 16],
    ) -> Result<DelayedDeliveryOperationStatusV1, DelayedDeliveryPersistenceErrorV1> {
        if !valid_owner(logical_owner_id) || !valid_id16(delayed_operation_id) {
            return Err(DelayedDeliveryPersistenceErrorV1::InvalidInput);
        }
        let row = sqlx::query(
            "SELECT delayed_operation_id, delivery_operation_id, state, state_revision,
                    deliver_at_unix_millis, scheduler_schedule_revision, error_code,
                    created_at_unix_millis, updated_at_unix_millis
             FROM makosh_data.communication_delayed_delivery_operations
             WHERE logical_owner_id = $1 AND delayed_operation_id = $2",
        )
        .bind(logical_owner_id)
        .bind(delayed_operation_id.as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| DelayedDeliveryPersistenceErrorV1::StorageUnavailable)?
        .ok_or(DelayedDeliveryPersistenceErrorV1::NotFound)?;
        status_from_row(&row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_validation_rejects_empty_unbounded_and_control_values() {
        assert!(!valid_owner(""));
        assert!(!valid_owner(&"a".repeat(129)));
        assert!(!valid_owner("owner\nother"));
        assert!(!valid_owner("owner with space"));
        assert!(valid_owner("owner-1"));
    }
}

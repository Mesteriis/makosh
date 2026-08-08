use makosh_events_api::EventEnvelopeError;

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), EventEnvelopeError> {
    if value.trim().is_empty() {
        return Err(EventEnvelopeError::EmptyField(field_name));
    }

    Ok(())
}

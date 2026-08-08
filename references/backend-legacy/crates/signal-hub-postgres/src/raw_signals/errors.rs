use makosh_signal_hub_api::raw_signals::{
    RawSignalPersistenceError, RawSignalPersistenceErrorKind,
};

pub(super) fn storage_error(error: sqlx::Error) -> RawSignalPersistenceError {
    RawSignalPersistenceError::new(RawSignalPersistenceErrorKind::Storage, error)
}

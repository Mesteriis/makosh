use makosh_kernel_control_store::{
    OperationAdmissionV1, OperationIdV1, OperationJournalStore, OperationStatusV1,
    OperationTerminalOutcomeV1,
};
use rusqlite::{OptionalExtension, params};

use crate::{SqliteControlStore, StoreError};

impl SqliteControlStore {
    pub fn admit_operation(
        &self,
        operation_id: OperationIdV1,
        request_digest: [u8; 32],
        deadline_unix_millis: u64,
    ) -> Result<OperationAdmissionV1, StoreError> {
        let deadline = i64::try_from(deadline_unix_millis)
            .map_err(|_| StoreError::InvalidOperationJournalEntry)?;
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let existing = load_status(&transaction, operation_id)?;
            let admission = match existing {
                Some((digest, status)) if digest == request_digest => {
                    OperationAdmissionV1::Duplicate(status)
                }
                Some(_) => return Err(StoreError::OperationRequestDigestConflict),
                None => {
                    transaction.execute("INSERT INTO makosh_kernel_operation_journal (operation_id, request_digest, deadline_unix_millis, terminal_kind, terminal_payload) VALUES (?1, ?2, ?3, NULL, NULL)", params![operation_id.as_bytes().as_slice(), request_digest.as_slice(), deadline])?;
                    OperationAdmissionV1::Admitted
                }
            };
            transaction.commit()?;
            Ok(admission)
        })
    }

    pub fn complete_operation(
        &self,
        operation_id: OperationIdV1,
        request_digest: [u8; 32],
        outcome: &OperationTerminalOutcomeV1,
    ) -> Result<(), StoreError> {
        let outcome = outcome.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let Some((digest, status)) = load_status(&transaction, operation_id)? else {
                return Err(StoreError::OperationMissing);
            };
            if digest != request_digest {
                return Err(StoreError::OperationRequestDigestConflict);
            }
            if status != OperationStatusV1::Admitted {
                return if status == OperationStatusV1::Terminal(outcome) {
                    Ok(())
                } else {
                    Err(StoreError::OperationOutcomeConflict)
                };
            }
            let (kind, payload) = encode_outcome(&outcome);
            transaction.execute("UPDATE makosh_kernel_operation_journal SET terminal_kind=?1, terminal_payload=?2 WHERE operation_id=?3", params![kind, payload, operation_id.as_bytes().as_slice()])?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn operation_status(
        &self,
        operation_id: OperationIdV1,
    ) -> Result<Option<OperationStatusV1>, StoreError> {
        self.with_connection(move |connection| {
            load_status(connection, operation_id).map(|value| value.map(|(_, status)| status))
        })
    }
}

impl OperationJournalStore for SqliteControlStore {
    type Error = StoreError;

    fn admit_operation(
        &self,
        id: OperationIdV1,
        digest: [u8; 32],
        deadline: u64,
    ) -> Result<OperationAdmissionV1, Self::Error> {
        Self::admit_operation(self, id, digest, deadline)
    }

    fn complete_operation(
        &self,
        id: OperationIdV1,
        digest: [u8; 32],
        outcome: &OperationTerminalOutcomeV1,
    ) -> Result<(), Self::Error> {
        Self::complete_operation(self, id, digest, outcome)
    }

    fn operation_status(
        &self,
        id: OperationIdV1,
    ) -> Result<Option<OperationStatusV1>, Self::Error> {
        Self::operation_status(self, id)
    }
}

fn load_status(
    connection: &rusqlite::Connection,
    id: OperationIdV1,
) -> Result<Option<([u8; 32], OperationStatusV1)>, StoreError> {
    connection.query_row("SELECT request_digest, terminal_kind, terminal_payload FROM makosh_kernel_operation_journal WHERE operation_id=?1", [id.as_bytes().as_slice()], |row| {
        let digest: Vec<u8> = row.get(0)?;
        let kind: Option<String> = row.get(1)?;
        let payload: Option<Vec<u8>> = row.get(2)?;
        let digest = digest.try_into().map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, 32))?;
        Ok((digest, decode_status(kind.as_deref(), payload.as_deref()).ok_or(rusqlite::Error::InvalidQuery)?))
    }).optional().map_err(StoreError::from)
}

fn encode_outcome(outcome: &OperationTerminalOutcomeV1) -> (&'static str, Vec<u8>) {
    match outcome {
        OperationTerminalOutcomeV1::Succeeded { response_digest } => {
            ("succeeded", response_digest.to_vec())
        }
        OperationTerminalOutcomeV1::Rejected { code } => ("rejected", code.as_bytes().to_vec()),
        OperationTerminalOutcomeV1::Failed { code } => ("failed", code.as_bytes().to_vec()),
    }
}

fn decode_status(kind: Option<&str>, payload: Option<&[u8]>) -> Option<OperationStatusV1> {
    match (kind, payload) {
        (None, None) => Some(OperationStatusV1::Admitted),
        (Some("succeeded"), Some(bytes)) => Some(OperationStatusV1::Terminal(
            OperationTerminalOutcomeV1::Succeeded {
                response_digest: bytes.try_into().ok()?,
            },
        )),
        (Some("rejected"), Some(bytes)) => Some(OperationStatusV1::Terminal(
            OperationTerminalOutcomeV1::Rejected {
                code: String::from_utf8(bytes.to_vec()).ok()?,
            },
        )),
        (Some("failed"), Some(bytes)) => Some(OperationStatusV1::Terminal(
            OperationTerminalOutcomeV1::Failed {
                code: String::from_utf8(bytes.to_vec()).ok()?,
            },
        )),
        _ => None,
    }
}

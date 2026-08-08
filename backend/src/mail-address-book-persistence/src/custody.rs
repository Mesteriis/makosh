use sqlx::Row;

use crate::{
    MailAddressBookPersistenceErrorV1, MailAddressBookPersistenceV1,
    MailAddressBookTargetSnapshotReceiptV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookSnapshotCustodyOutcomeV1 {
    Recorded,
    AlreadyRecorded,
}

impl MailAddressBookPersistenceV1 {
    pub async fn record_target_snapshot_receipt(
        &self,
        command_id: [u8; 16],
        receipt: MailAddressBookTargetSnapshotReceiptV1,
        recorded_at_unix_seconds: i64,
    ) -> Result<MailAddressBookSnapshotCustodyOutcomeV1, MailAddressBookPersistenceErrorV1> {
        if zero(&command_id)
            || zero(&receipt.reference_id)
            || zero(&receipt.receipt_sha256)
            || recorded_at_unix_seconds <= 0
        {
            return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
        }
        let mut transaction = self.pool.begin().await.map_err(storage_error)?;
        let row = sqlx::query(
            "SELECT state, contact_snapshot_sha256,
                    target_contact_snapshot_reference_id,
                    target_contact_snapshot_receipt_sha256,
                    snapshot_custody_recorded_at_unix_seconds
             FROM makosh_data.mail_address_book_upsert_inbox
             WHERE command_id = $1
             FOR UPDATE",
        )
        .bind(command_id.as_slice())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(MailAddressBookPersistenceErrorV1::NotFound)?;
        let state: i16 = row.try_get("state").map_err(row_error)?;
        let source_receipt_sha256: Vec<u8> =
            row.try_get("contact_snapshot_sha256").map_err(row_error)?;
        if state != 0 || source_receipt_sha256.as_slice() != receipt.receipt_sha256 {
            return Err(MailAddressBookPersistenceErrorV1::Conflict);
        }
        let existing = receipt_from_row(&row)?;
        let outcome = match existing {
            Some(existing) if existing == receipt => {
                MailAddressBookSnapshotCustodyOutcomeV1::AlreadyRecorded
            }
            Some(_) => return Err(MailAddressBookPersistenceErrorV1::Conflict),
            None => {
                let updated = sqlx::query(
                    "UPDATE makosh_data.mail_address_book_upsert_inbox
                     SET target_contact_snapshot_reference_id = $2,
                         target_contact_snapshot_receipt_sha256 = $3,
                         snapshot_custody_recorded_at_unix_seconds = $4
                     WHERE command_id = $1
                       AND state = 0
                       AND target_contact_snapshot_reference_id IS NULL
                       AND target_contact_snapshot_receipt_sha256 IS NULL
                       AND snapshot_custody_recorded_at_unix_seconds IS NULL",
                )
                .bind(command_id.as_slice())
                .bind(receipt.reference_id.as_slice())
                .bind(receipt.receipt_sha256.as_slice())
                .bind(recorded_at_unix_seconds)
                .execute(&mut *transaction)
                .await
                .map_err(storage_error)?;
                if updated.rows_affected() != 1 {
                    return Err(MailAddressBookPersistenceErrorV1::Conflict);
                }
                MailAddressBookSnapshotCustodyOutcomeV1::Recorded
            }
        };
        transaction.commit().await.map_err(storage_error)?;
        Ok(outcome)
    }
}

fn receipt_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<MailAddressBookTargetSnapshotReceiptV1>, MailAddressBookPersistenceErrorV1> {
    let reference_id = optional_id::<16>(row, "target_contact_snapshot_reference_id")?;
    let receipt_sha256 = optional_id::<32>(row, "target_contact_snapshot_receipt_sha256")?;
    let recorded_at: Option<i64> = row
        .try_get("snapshot_custody_recorded_at_unix_seconds")
        .map_err(row_error)?;
    match (reference_id, receipt_sha256, recorded_at) {
        (None, None, None) => Ok(None),
        (Some(reference_id), Some(receipt_sha256), Some(recorded_at)) if recorded_at > 0 => {
            Ok(Some(MailAddressBookTargetSnapshotReceiptV1 {
                reference_id,
                receipt_sha256,
            }))
        }
        _ => Err(MailAddressBookPersistenceErrorV1::InvalidRow),
    }
}

fn optional_id<const WIDTH: usize>(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<[u8; WIDTH]>, MailAddressBookPersistenceErrorV1> {
    row.try_get::<Option<Vec<u8>>, _>(column)
        .map_err(row_error)?
        .map(|value| {
            let value: [u8; WIDTH] = value.try_into().map_err(row_error)?;
            (!zero(&value))
                .then_some(value)
                .ok_or(MailAddressBookPersistenceErrorV1::InvalidRow)
        })
        .transpose()
}

fn zero(value: &[u8]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

fn storage_error<T>(_: T) -> MailAddressBookPersistenceErrorV1 {
    MailAddressBookPersistenceErrorV1::StorageUnavailable
}

fn row_error<T>(_: T) -> MailAddressBookPersistenceErrorV1 {
    MailAddressBookPersistenceErrorV1::InvalidRow
}

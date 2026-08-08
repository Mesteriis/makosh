//! Mail-owned provider execution for accepted address-book upserts.

use makosh_mail_address_book_contract::{
    MailAddressBookResultEnvelopeContextV1,
    build_mail_address_book_entry_upsert_rejected_result_v1,
    build_mail_address_book_entry_upserted_result_v1,
    wire::{
        MailAddressBookEntryUpsertRejectedV1, MailAddressBookEntryUpsertedV1,
        MailAddressBookRejectCodeV1,
    },
};
use makosh_mail_address_book_persistence::{
    MailAddressBookDispatchOutcomeV1, MailAddressBookPersistenceErrorV1,
    MailAddressBookPersistenceV1, PendingMailAddressBookUpsertV1,
};
use makosh_mail_api::{MailAddressBookProviderV1, MailInboundTransportV1};
use makosh_mail_google_people::{GooglePeopleAdapterErrorV1, GooglePeopleUpsertV1};

use crate::{
    address_book_provider::google_people_client_v1,
    address_book_snapshot::{
        MailAddressBookSnapshotErrorV1, read_contact_snapshot_v1,
        transfer_contact_snapshot_custody_v1,
    },
    managed::{MailAdmittedRuntime, MailBootstrapError},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailAddressBookWorkerErrorV1 {
    InvalidClock,
    Persistence,
    ResultEnvelope,
}

pub async fn process_next_mail_address_book_upsert_v1(
    runtime: &mut MailAdmittedRuntime,
    now_unix_seconds: i64,
) -> Result<bool, MailAddressBookWorkerErrorV1> {
    if now_unix_seconds <= 0 {
        return Err(MailAddressBookWorkerErrorV1::InvalidClock);
    }
    if let Some(job) = runtime
        .address_book_persistence
        .uncertain_upserts(1)
        .await
        .map_err(persistence_error)?
        .into_iter()
        .next()
    {
        persist_rejection(
            &runtime.address_book_persistence,
            &runtime.runtime_instance_id,
            runtime.runtime_generation,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown,
            now_unix_seconds,
        )
        .await?;
        return Ok(true);
    }
    let Some(job) = runtime
        .address_book_persistence
        .pending_upserts(1)
        .await
        .map_err(persistence_error)?
        .into_iter()
        .next()
    else {
        return Ok(false);
    };
    if runtime.select_account(&job.admission.account_id).is_err()
        || !runtime.provider_io_permitted()
    {
        persist_rejection(
            &runtime.address_book_persistence,
            &runtime.runtime_instance_id,
            runtime.runtime_generation,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable,
            now_unix_seconds,
        )
        .await?;
        return Ok(true);
    }
    let provider = runtime.address_book.provider;
    if provider == MailAddressBookProviderV1::None {
        persist_rejection(
            &runtime.address_book_persistence,
            &runtime.runtime_instance_id,
            runtime.runtime_generation,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable,
            now_unix_seconds,
        )
        .await?;
        return Ok(true);
    }
    if provider == MailAddressBookProviderV1::IcloudCardDav {
        persist_rejection(
            &runtime.address_book_persistence,
            &runtime.runtime_instance_id,
            runtime.runtime_generation,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeReadOnlyProvider,
            now_unix_seconds,
        )
        .await?;
        return Ok(true);
    }
    if !matches!(runtime.account.inbound, MailInboundTransportV1::Gmail(_)) {
        return reject(
            runtime,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable,
            now_unix_seconds,
        )
        .await;
    }
    let Some(endpoint) = runtime.address_book.google_people_endpoint.clone() else {
        return reject(
            runtime,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeAccountUnavailable,
            now_unix_seconds,
        )
        .await;
    };
    let binding = runtime
        .durable
        .gmail_oauth_credential_binding(&job.admission.account_id)
        .await
        .map_err(|_| MailAddressBookWorkerErrorV1::Persistence)?;
    let Some(binding) = binding else {
        return reject(
            runtime,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable,
            now_unix_seconds,
        )
        .await;
    };
    if !binding.contacts_write_authorized {
        return reject(
            runtime,
            &job,
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeWriteScopeRequired,
            now_unix_seconds,
        )
        .await;
    }
    let access_token = match runtime.resolve_gmail_access_token().await {
        Ok(token) => token,
        Err(MailBootstrapError::Persistence) => {
            return Err(MailAddressBookWorkerErrorV1::Persistence);
        }
        Err(_) => {
            return reject(
                runtime,
                &job,
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable,
                now_unix_seconds,
            )
            .await;
        }
    };
    let access_token = match std::str::from_utf8(&access_token) {
        Ok(token) => token,
        Err(_) => {
            return reject(
                runtime,
                &job,
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeCredentialUnavailable,
                now_unix_seconds,
            )
            .await;
        }
    };
    let target_snapshot_receipt = match job.target_snapshot_receipt {
        Some(receipt) => receipt,
        None => match transfer_contact_snapshot_custody_v1(&mut runtime.control_channel, &job) {
            Ok(receipt) => {
                runtime
                    .address_book_persistence
                    .record_target_snapshot_receipt(
                        job.admission.command_id,
                        receipt,
                        now_unix_seconds,
                    )
                    .await
                    .map_err(persistence_error)?;
                receipt
            }
            Err(error) => {
                return reject(runtime, &job, snapshot_reject_code(error), now_unix_seconds).await;
            }
        },
    };
    let source = match read_contact_snapshot_v1(
        &mut runtime.control_channel,
        &job,
        &target_snapshot_receipt,
    ) {
        Ok(source) => source,
        Err(error) => {
            return reject(runtime, &job, snapshot_reject_code(error), now_unix_seconds).await;
        }
    };
    match runtime
        .address_book_persistence
        .mark_dispatch_started(job.admission.command_id, now_unix_seconds)
        .await
        .map_err(persistence_error)?
    {
        MailAddressBookDispatchOutcomeV1::Started => {}
        MailAddressBookDispatchOutcomeV1::AlreadyDispatching => {
            return reject(
                runtime,
                &job,
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown,
                now_unix_seconds,
            )
            .await;
        }
        MailAddressBookDispatchOutcomeV1::AlreadyCompleted => return Ok(true),
    }
    let link = source.target_account_link.as_ref();
    let request = GooglePeopleUpsertV1 {
        resource_name: link.map(|value| value.provider_entry_id.clone()),
        expected_etag: link.and_then(|value| value.provider_etag.clone()),
        display_name: source.display_name,
        email_addresses: source.email_addresses,
        phone_numbers: source.phone_numbers,
    };
    let result = match google_people_client_v1(&endpoint) {
        Ok(client) => client.upsert_contact(access_token, &request).await,
        Err(error) => Err(error),
    };
    match result {
        Ok(upserted) => {
            let result = build_mail_address_book_entry_upserted_result_v1(
                job.admission.command_message_id,
                MailAddressBookEntryUpsertedV1 {
                    command_id: job.admission.command_id.to_vec(),
                    run_id: job.admission.run_id.to_vec(),
                    provider_entry_id: upserted.resource_name,
                    provider_etag: upserted.etag,
                    applied_contact_revision: job.admission.expected_contact_revision,
                    provider_kind: makosh_mail_address_book_contract::wire::MailAddressBookProviderKindV1::MailAddressBookProviderKindGooglePeople as i32,
                },
                &result_context(runtime, &job, now_unix_seconds),
            )
            .map_err(|_| MailAddressBookWorkerErrorV1::ResultEnvelope)?;
            runtime
                .address_book_persistence
                .complete_upsert_command(job.admission.command_id, &result, now_unix_seconds)
                .await
                .map_err(persistence_error)?;
            Ok(true)
        }
        Err(error) => reject(runtime, &job, provider_reject_code(error), now_unix_seconds).await,
    }
}

async fn reject(
    runtime: &MailAdmittedRuntime,
    job: &PendingMailAddressBookUpsertV1,
    code: MailAddressBookRejectCodeV1,
    now_unix_seconds: i64,
) -> Result<bool, MailAddressBookWorkerErrorV1> {
    persist_rejection(
        &runtime.address_book_persistence,
        &runtime.runtime_instance_id,
        runtime.runtime_generation,
        job,
        code,
        now_unix_seconds,
    )
    .await?;
    Ok(true)
}

async fn persist_rejection(
    persistence: &MailAddressBookPersistenceV1,
    runtime_instance_id: &str,
    runtime_generation: u64,
    job: &PendingMailAddressBookUpsertV1,
    code: MailAddressBookRejectCodeV1,
    now_unix_seconds: i64,
) -> Result<(), MailAddressBookWorkerErrorV1> {
    let result = build_mail_address_book_entry_upsert_rejected_result_v1(
        job.admission.command_message_id,
        MailAddressBookEntryUpsertRejectedV1 {
            command_id: job.admission.command_id.to_vec(),
            run_id: job.admission.run_id.to_vec(),
            code: code as i32,
            outcome_unknown: code
                == MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown,
        },
        &MailAddressBookResultEnvelopeContextV1 {
            runtime_instance_id: runtime_instance_id.to_owned(),
            runtime_generation,
            completed_at_unix_seconds: now_unix_seconds,
            completed_at_nanos: 0,
            execution_attempt: job.execution_attempt,
        },
    )
    .map_err(|_| MailAddressBookWorkerErrorV1::ResultEnvelope)?;
    persistence
        .complete_upsert_command(job.admission.command_id, &result, now_unix_seconds)
        .await
        .map_err(persistence_error)?;
    Ok(())
}

fn result_context(
    runtime: &MailAdmittedRuntime,
    job: &PendingMailAddressBookUpsertV1,
    now_unix_seconds: i64,
) -> MailAddressBookResultEnvelopeContextV1 {
    MailAddressBookResultEnvelopeContextV1 {
        runtime_instance_id: runtime.runtime_instance_id.clone(),
        runtime_generation: runtime.runtime_generation,
        completed_at_unix_seconds: now_unix_seconds,
        completed_at_nanos: 0,
        execution_attempt: job.execution_attempt,
    }
}

fn persistence_error(_: MailAddressBookPersistenceErrorV1) -> MailAddressBookWorkerErrorV1 {
    MailAddressBookWorkerErrorV1::Persistence
}

fn provider_reject_code(error: GooglePeopleAdapterErrorV1) -> MailAddressBookRejectCodeV1 {
    match error {
        GooglePeopleAdapterErrorV1::EtagConflict => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeEtagConflict
        }
        GooglePeopleAdapterErrorV1::OutcomeUnknown
        | GooglePeopleAdapterErrorV1::InvalidResponse => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown
        }
        GooglePeopleAdapterErrorV1::Unavailable => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        }
        GooglePeopleAdapterErrorV1::InvalidRequest
        | GooglePeopleAdapterErrorV1::ProviderRejected(_) => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest
        }
    }
}

fn snapshot_reject_code(error: MailAddressBookSnapshotErrorV1) -> MailAddressBookRejectCodeV1 {
    match error {
        MailAddressBookSnapshotErrorV1::InvalidReceipt => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeInvalidRequest
        }
        MailAddressBookSnapshotErrorV1::CustodyDenied => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodePolicy
        }
        MailAddressBookSnapshotErrorV1::Unavailable => {
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_request_protocol_failures_are_outcome_unknown() {
        for error in [
            GooglePeopleAdapterErrorV1::OutcomeUnknown,
            GooglePeopleAdapterErrorV1::InvalidResponse,
        ] {
            assert_eq!(
                provider_reject_code(error),
                MailAddressBookRejectCodeV1::MailAddressBookRejectCodeOutcomeUnknown
            );
        }
        assert_eq!(
            provider_reject_code(GooglePeopleAdapterErrorV1::Unavailable),
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeProviderUnavailable
        );
        assert_eq!(
            provider_reject_code(GooglePeopleAdapterErrorV1::EtagConflict),
            MailAddressBookRejectCodeV1::MailAddressBookRejectCodeEtagConflict
        );
    }
}

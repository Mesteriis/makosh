use std::os::unix::net::UnixStream;

use makosh_contacts_command_api::{
    bind_mail_address_book_provider_link_rejected_contract_reference_v1,
    contact_upsert_rejected_contract_reference_v1, contact_upserted_contract_reference_v1,
    mail_address_book_provider_link_bound_contract_reference_v1,
};
use makosh_contacts_mail_sync_source_api::{
    contact_changed_for_mail_sync_contract_reference_v1,
    contact_mail_sync_source_prepared_contract_reference_v1,
    contact_mail_sync_source_rejected_contract_reference_v1,
};
use makosh_events_jetstream::{
    JetStreamClient, RuntimeJetStreamConnection, RuntimeNatsIdentity, RuntimePublishPermitV1,
    RuntimeSubscribePermitV1, request_managed_runtime_event_access_v2,
};
use makosh_mail_address_book_contract::MailAddressBookContractV1;
use makosh_mail_contacts_sync_api::{
    MAIL_CONTACTS_SYNC_MODULE_ID_V1, MAIL_CONTACTS_SYNC_OWNER_ID_V1,
    mail_contacts_sync_query_contract_v1, mail_contacts_sync_start_contract_v1,
    wire::{
        MailContactsSyncErrorCodeV1, MailContactsSyncStateV1, StartMailContactsSyncRequestV1,
        StartMailContactsSyncResponseV1,
    },
};
use makosh_mail_contacts_sync_persistence::{
    MailContactsSyncPersistenceErrorV1, MailContactsSyncPersistenceV1,
};
use makosh_runtime_protocol::{
    managed_control::{ManagedControlChannelV2, RejectManagedControlRequestsV2},
    v1::{
        ContractReferenceV1, ManagedRuntimeClientDeliveryResponseV1,
        ManagedRuntimeControlResponseV1, ManagedRuntimeReadyRequestV1,
        ManagedStorageRuntimeConfigurationV1, ModuleClientRequestV1, ModuleClientResponseV1,
        managed_runtime_control_request_v1::Operation,
        managed_runtime_control_response_v1::Result as ControlResult,
    },
    validation::module_client::{
        validate_module_client_request_v1, validate_module_client_response_v1,
    },
};
use makosh_scheduler_protocol::SCHEDULER_JOB_DESCRIPTOR_SET_V1;
use makosh_storage_protocol::{
    StorageBindingAccessV1, StorageBindingFencesV1, StorageBindingIdentityV1, StorageBindingV1,
    StorageEffectiveBudgetsV1,
};
use makosh_storage_vault::{
    InheritedKernelVaultRouteV2, StorageVaultLeaseAdapterV1, StorageVaultRouteContextV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{
    MailContactsSyncContactsResultErrorV1, MailContactsSyncProviderEventErrorV1,
    MailContactsSyncProviderRuntimeContextV1, MailContactsSyncRuntimeSettingsV1,
    MailContactsSyncScheduledExecutionContextV1, MailContactsSyncScheduledExecutionErrorV1,
    client_port::{
        MailContactsSyncClientContextV1, get_mail_contacts_sync_payload_v1,
        start_mail_contacts_sync_payload_v1,
    },
    client_realtime::{MailContactsSyncRealtimeErrorV1, MailContactsSyncRealtimePublisherV1},
    consume_contact_upsert_rejected_once_v1, consume_contact_upserted_once_v1,
    consume_mail_address_book_entry_once_v1, consume_mail_address_book_page_completed_once_v1,
    consume_mail_address_book_page_rejected_once_v1,
    event_outbox::{MailContactsSyncRelayErrorV1, relay_mail_contacts_sync_outbox_once_v1},
    provider_link_results::{
        MailContactsSyncProviderLinkResultContextV1, MailContactsSyncProviderLinkResultErrorV1,
        consume_provider_link_bound_once_v1, consume_provider_link_rejected_once_v1,
    },
    provider_write_results::{
        MailContactsSyncProviderWriteResultContextV1, MailContactsSyncProviderWriteResultErrorV1,
        consume_mail_entry_upsert_rejected_once_v1, consume_mail_entry_upserted_once_v1,
    },
    reverse_change::{
        MailContactsSyncReverseChangeContextV1, MailContactsSyncReverseChangeErrorV1,
        consume_contact_changed_once_v1,
    },
    scheduler_completion::queue_mail_contacts_sync_terminal_once_v1,
    scheduler_due::{MailContactsSyncDueContractV1, MailContactsSyncDueRuntimeContextV1},
    scheduler_execution::consume_mail_contacts_sync_due_once_v1,
    source_results::{
        MailContactsSyncSourceResultContextV1, MailContactsSyncSourceResultErrorV1,
        consume_source_prepared_once_v1, consume_source_rejected_once_v1,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailContactsSyncRuntimeAdmissionV1 {
    pub logical_owner_id: String,
    pub registration_id: String,
    pub runtime_instance_id: String,
    pub runtime_generation: u64,
    pub grant_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContactsSyncManagedRuntimeErrorV1 {
    Admission,
    EventContract,
    EventUnavailable,
    Persistence(MailContactsSyncPersistenceErrorV1),
    Unavailable,
}

struct MailContactsSyncSubscriptionsV1 {
    contact_changed: RuntimeSubscribePermitV1,
    contact_rejected: RuntimeSubscribePermitV1,
    contact_source_prepared: RuntimeSubscribePermitV1,
    contact_source_rejected: RuntimeSubscribePermitV1,
    contact_upserted: RuntimeSubscribePermitV1,
    provider_link_bound: RuntimeSubscribePermitV1,
    provider_link_rejected: RuntimeSubscribePermitV1,
    mail_entry_observed: RuntimeSubscribePermitV1,
    mail_entry_upsert_rejected: RuntimeSubscribePermitV1,
    mail_entry_upserted: RuntimeSubscribePermitV1,
    mail_page_completed: RuntimeSubscribePermitV1,
    mail_page_rejected: RuntimeSubscribePermitV1,
    scheduler_due: RuntimeSubscribePermitV1,
}

pub struct MailContactsSyncManagedRuntimeV1 {
    admission: MailContactsSyncRuntimeAdmissionV1,
    configurations: Vec<(String, MailContactsSyncRuntimeSettingsV1)>,
    control_channel: ManagedControlChannelV2<UnixStream>,
    persistence: MailContactsSyncPersistenceV1,
    event_connection: RuntimeJetStreamConnection,
    event_publish_permit: RuntimePublishPermitV1,
    subscriptions: MailContactsSyncSubscriptionsV1,
    client_realtime: MailContactsSyncRealtimePublisherV1,
}

impl MailContactsSyncManagedRuntimeV1 {
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        control_channel: UnixStream,
        descriptor_bytes: Vec<u8>,
        settings_schema_bytes: Vec<u8>,
        admission: &MailContactsSyncRuntimeAdmissionV1,
        configurations: Vec<(String, MailContactsSyncRuntimeSettingsV1)>,
        storage_configuration: ManagedStorageRuntimeConfigurationV1,
        event_hub_endpoint: &str,
        event_credential_revision: u64,
    ) -> Result<Self, MailContactsSyncManagedRuntimeErrorV1> {
        validate_admission(admission, &configurations)?;
        if event_hub_endpoint.trim().is_empty() || event_credential_revision == 0 {
            return Err(MailContactsSyncManagedRuntimeErrorV1::Admission);
        }
        let mut control_channel = ManagedControlChannelV2::new(control_channel);
        authenticate(
            &mut control_channel,
            descriptor_bytes,
            settings_schema_bytes,
            admission,
        )?;
        let binding = storage_binding(&storage_configuration, admission)?;
        let vault_public_key = storage_configuration
            .vault_hpke_public_key_x25519
            .as_slice()
            .try_into()
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
        let vault_context = StorageVaultRouteContextV1::new(
            storage_configuration.vault_instance_id.clone(),
            storage_configuration.vault_runtime_generation,
            vault_public_key,
        )
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
        let mut leases = StorageVaultLeaseAdapterV1::new(
            InheritedKernelVaultRouteV2::new(control_channel),
            vault_context,
        );
        let password = resolve_storage_credential(&mut leases, &binding).await?;
        let password = std::str::from_utf8(&password)
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
        let persistence = MailContactsSyncPersistenceV1::connect_runtime(
            &binding,
            &storage_configuration.database_id,
            &storage_configuration.pgbouncer_host,
            storage_configuration.pgbouncer_port,
            password,
        )
        .await
        .map_err(MailContactsSyncManagedRuntimeErrorV1::Persistence)?;
        persistence
            .verify_storage_ready()
            .await
            .map_err(MailContactsSyncManagedRuntimeErrorV1::Persistence)?;

        let mut control_channel = leases.into_route_port().into_channel();
        let event_access = request_managed_runtime_event_access_v2(
            &mut control_channel,
            &storage_configuration.logical_owner_id,
            &admission.registration_id,
            &admission.runtime_instance_id,
            admission.runtime_generation,
            admission.grant_epoch,
            event_credential_revision,
        )
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::EventUnavailable)?;
        let event_identity = RuntimeNatsIdentity::new(
            admission.runtime_instance_id.clone(),
            admission.runtime_generation,
            admission.grant_epoch,
        )
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
        let event_publish_permit = event_access
            .publish_permit(
                &admission.registration_id,
                &admission.runtime_instance_id,
                admission.runtime_generation,
                admission.grant_epoch,
            )
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
        let subscriptions = bind_subscriptions(
            event_access
                .subscribe_permits(
                    &admission.registration_id,
                    &admission.runtime_instance_id,
                    admission.runtime_generation,
                    admission.grant_epoch,
                )
                .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?,
        )?;
        let event_connection = JetStreamClient::connect_runtime_with_jwt(
            event_hub_endpoint,
            event_identity,
            event_access.into_credential(),
        )
        .await
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::EventUnavailable)?;
        let mut client_realtime = MailContactsSyncRealtimePublisherV1::default();
        let mut dispatcher = RejectManagedControlRequestsV2;
        client_realtime
            .publish_pending(
                &persistence,
                &mut control_channel,
                &mut dispatcher,
                &admission.logical_owner_id,
            )
            .await
            .map_err(realtime_error)?;
        signal_ready(&mut control_channel, admission)?;
        control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
        Ok(Self {
            admission: admission.clone(),
            configurations,
            control_channel,
            persistence,
            event_connection,
            event_publish_permit,
            subscriptions,
            client_realtime,
        })
    }

    pub async fn pump_control_once(
        &mut self,
        now_unix_millis: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        let Some((correlation_id, request)) = self
            .control_channel
            .try_receive_request()
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?
        else {
            return Ok(false);
        };
        let Some(Operation::ClientDelivery(delivery)) = request.operation else {
            return self.write_client_error(correlation_id);
        };
        let Some(request) = delivery
            .request
            .filter(|request| validate_module_client_request_v1(request).is_ok())
        else {
            return self.write_client_error(correlation_id);
        };
        let response = dispatch_client(
            &self.persistence,
            &self.admission,
            &self.configurations,
            request,
            now_unix_millis,
        )
        .await;
        if validate_module_client_response_v1(&response).is_err() {
            return Err(MailContactsSyncManagedRuntimeErrorV1::Unavailable);
        }
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: Some(ControlResult::ClientDelivery(
                        ManagedRuntimeClientDeliveryResponseV1 {
                            response: Some(response),
                        },
                    )),
                    error_code: String::new(),
                },
            )
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }

    pub async fn consume_contact_rejected_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_contact_upsert_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.contact_rejected,
            &self.provider_context(now),
        )
        .await
        .map_err(contacts_error)
    }

    pub async fn consume_contact_changed_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_contact_changed_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.contact_changed,
            &self.configurations,
            &MailContactsSyncReverseChangeContextV1 {
                logical_owner_id: &self.admission.logical_owner_id,
                runtime_instance_id: &self.admission.runtime_instance_id,
                runtime_generation: self.admission.runtime_generation,
                now_unix_millis: now,
            },
        )
        .await
        .map_err(reverse_change_error)
    }

    pub async fn consume_contact_upserted_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_contact_upserted_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.contact_upserted,
            &self.provider_context(now),
        )
        .await
        .map_err(contacts_error)
    }

    pub async fn consume_contact_source_prepared_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_source_prepared_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.contact_source_prepared,
            &self.source_result_context(now),
        )
        .await
        .map_err(source_result_error)
    }

    pub async fn consume_provider_link_bound_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_provider_link_bound_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.provider_link_bound,
            &MailContactsSyncProviderLinkResultContextV1 {
                logical_owner_id: &self.admission.logical_owner_id,
                now_unix_millis: now,
            },
        )
        .await
        .map_err(provider_link_result_error)
    }

    pub async fn consume_provider_link_rejected_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_provider_link_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.provider_link_rejected,
            &MailContactsSyncProviderLinkResultContextV1 {
                logical_owner_id: &self.admission.logical_owner_id,
                now_unix_millis: now,
            },
        )
        .await
        .map_err(provider_link_result_error)
    }

    pub async fn consume_contact_source_rejected_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_source_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.contact_source_rejected,
            &self.source_result_context(now),
        )
        .await
        .map_err(source_result_error)
    }

    pub async fn consume_mail_entry_upserted_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_mail_entry_upserted_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.mail_entry_upserted,
            &self.provider_write_result_context(now),
        )
        .await
        .map_err(provider_write_result_error)
    }

    pub async fn consume_mail_entry_upsert_rejected_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_mail_entry_upsert_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.mail_entry_upsert_rejected,
            &self.provider_write_result_context(now),
        )
        .await
        .map_err(provider_write_result_error)
    }

    pub async fn consume_mail_entry_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_mail_address_book_entry_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.mail_entry_observed,
            &self.provider_context(now),
        )
        .await
        .map_err(provider_error)
    }

    pub async fn consume_mail_page_completed_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_mail_address_book_page_completed_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.mail_page_completed,
            &self.provider_context(now),
        )
        .await
        .map_err(provider_error)
    }

    pub async fn consume_mail_page_rejected_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_mail_address_book_page_rejected_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.mail_page_rejected,
            &self.provider_context(now),
        )
        .await
        .map_err(provider_error)
    }

    pub async fn consume_scheduler_due_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        consume_mail_contacts_sync_due_once_v1(
            &self.persistence,
            &self.event_connection,
            &self.subscriptions.scheduler_due,
            &MailContactsSyncScheduledExecutionContextV1 {
                logical_owner_id: self.admission.logical_owner_id.clone(),
                runtime_instance_id: self.admission.runtime_instance_id.clone(),
                authoritative_now_unix_millis: now,
                due_context: self.scheduler_due_context(),
            },
            &self.configurations,
        )
        .await
        .map_err(scheduled_error)
    }

    pub async fn queue_scheduler_terminal_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        queue_mail_contacts_sync_terminal_once_v1(
            &self.persistence,
            &self.admission.logical_owner_id,
            &self.scheduler_due_context(),
            now,
        )
        .await
        .map_err(scheduled_completion_error)
    }

    fn scheduler_due_context(&self) -> MailContactsSyncDueRuntimeContextV1 {
        let schema_sha256: [u8; 32] = Sha256::digest(SCHEDULER_JOB_DESCRIPTOR_SET_V1).into();
        MailContactsSyncDueRuntimeContextV1 {
            runtime_instance_id: runtime_instance_bytes(&self.admission.runtime_instance_id),
            runtime_generation: self.admission.runtime_generation,
            grant_epoch: self.admission.grant_epoch,
            contract: MailContactsSyncDueContractV1 {
                job_revision: 1,
                job_schema_sha256: schema_sha256,
                receipt_revision: 1,
                receipt_schema_sha256: schema_sha256,
            },
        }
    }

    pub async fn relay_outbox_once(
        &self,
        now: i64,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        relay_mail_contacts_sync_outbox_once_v1(
            &self.persistence,
            &self.admission.logical_owner_id,
            &self.event_connection,
            &self.event_publish_permit,
            now,
        )
        .await
        .map_err(relay_error)
    }

    pub async fn pump_client_realtime_once(
        &mut self,
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        self.control_channel
            .inner_mut()
            .set_nonblocking(false)
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
        let mut dispatcher = RejectManagedControlRequestsV2;
        let result = self
            .client_realtime
            .publish_pending(
                &self.persistence,
                &mut self.control_channel,
                &mut dispatcher,
                &self.admission.logical_owner_id,
            )
            .await
            .map_err(realtime_error);
        self.control_channel
            .inner_mut()
            .set_nonblocking(true)
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
        result
    }

    fn provider_context(&self, now: i64) -> MailContactsSyncProviderRuntimeContextV1 {
        MailContactsSyncProviderRuntimeContextV1 {
            logical_owner_id: self.admission.logical_owner_id.clone(),
            runtime_instance_id: self.admission.runtime_instance_id.clone(),
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis: now,
        }
    }

    fn source_result_context(&self, now: i64) -> MailContactsSyncSourceResultContextV1<'_> {
        MailContactsSyncSourceResultContextV1 {
            logical_owner_id: &self.admission.logical_owner_id,
            runtime_instance_id: &self.admission.runtime_instance_id,
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis: now,
        }
    }

    fn provider_write_result_context(
        &self,
        now: i64,
    ) -> MailContactsSyncProviderWriteResultContextV1<'_> {
        MailContactsSyncProviderWriteResultContextV1 {
            logical_owner_id: &self.admission.logical_owner_id,
            runtime_instance_id: &self.admission.runtime_instance_id,
            runtime_generation: self.admission.runtime_generation,
            now_unix_millis: now,
        }
    }

    fn write_client_error(
        &mut self,
        correlation_id: [u8; 16],
    ) -> Result<bool, MailContactsSyncManagedRuntimeErrorV1> {
        self.control_channel
            .write_response(
                correlation_id,
                ManagedRuntimeControlResponseV1 {
                    result: None,
                    error_code: "managed_runtime_control_invalid_client_delivery".to_owned(),
                },
            )
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
        Ok(true)
    }
}

async fn dispatch_client(
    persistence: &MailContactsSyncPersistenceV1,
    admission: &MailContactsSyncRuntimeAdmissionV1,
    configurations: &[(String, MailContactsSyncRuntimeSettingsV1)],
    request: ModuleClientRequestV1,
    now: i64,
) -> ModuleClientResponseV1 {
    let identity_is_valid = request.protocol_major == 1
        && request.module_id == MAIL_CONTACTS_SYNC_MODULE_ID_V1
        && request.owner_id == MAIL_CONTACTS_SYNC_OWNER_ID_V1
        && request.logical_owner_id == admission.logical_owner_id;
    let (response_payload, accepted_route) = if identity_is_valid
        && request.contract.as_ref() == Some(&mail_contacts_sync_start_contract_v1())
    {
        let selected = StartMailContactsSyncRequestV1::decode(request.request_payload.as_slice())
            .ok()
            .and_then(|start| {
                configurations
                    .iter()
                    .find(|(_, settings)| settings.account_id == start.account_id)
            });
        if let Some((_, settings)) = selected {
            (
                start_mail_contacts_sync_payload_v1(
                    persistence,
                    &MailContactsSyncClientContextV1 {
                        logical_owner_id: admission.logical_owner_id.clone(),
                        runtime_instance_id: admission.runtime_instance_id.clone(),
                        runtime_generation: admission.runtime_generation,
                        authoritative_now_unix_millis: now,
                        settings: settings.clone(),
                    },
                    &request.request_payload,
                )
                .await,
                true,
            )
        } else {
            (
                StartMailContactsSyncResponseV1 {
                    run_id: Vec::new(),
                    state: MailContactsSyncStateV1::MailContactsSyncStateUnspecified as i32,
                    error: MailContactsSyncErrorCodeV1::MailContactsSyncErrorCodeInvalidRequest
                        as i32,
                }
                .encode_to_vec(),
                true,
            )
        }
    } else if identity_is_valid
        && request.contract.as_ref() == Some(&mail_contacts_sync_query_contract_v1())
    {
        (
            get_mail_contacts_sync_payload_v1(
                persistence,
                &admission.logical_owner_id,
                &request.request_payload,
            )
            .await,
            true,
        )
    } else {
        (Vec::new(), false)
    };
    ModuleClientResponseV1 {
        protocol_major: 1,
        request_id: request.request_id,
        response_payload,
        error_code: if accepted_route {
            String::new()
        } else {
            "REJECTED".to_owned()
        },
    }
}

fn bind_subscriptions(
    permits: Vec<RuntimeSubscribePermitV1>,
) -> Result<MailContactsSyncSubscriptionsV1, MailContactsSyncManagedRuntimeErrorV1> {
    if permits.len() != 13 {
        return Err(MailContactsSyncManagedRuntimeErrorV1::Admission);
    }
    Ok(MailContactsSyncSubscriptionsV1 {
        contact_changed: exact_permit(
            &permits,
            &contact_changed_for_mail_sync_contract_reference_v1(),
        )?,
        contact_rejected: exact_permit(&permits, &contact_upsert_rejected_contract_reference_v1())?,
        contact_source_prepared: exact_permit(
            &permits,
            &contact_mail_sync_source_prepared_contract_reference_v1(),
        )?,
        contact_source_rejected: exact_permit(
            &permits,
            &contact_mail_sync_source_rejected_contract_reference_v1(),
        )?,
        contact_upserted: exact_permit(&permits, &contact_upserted_contract_reference_v1())?,
        provider_link_bound: exact_permit(
            &permits,
            &mail_address_book_provider_link_bound_contract_reference_v1(),
        )?,
        provider_link_rejected: exact_permit(
            &permits,
            &bind_mail_address_book_provider_link_rejected_contract_reference_v1(),
        )?,
        mail_entry_observed: exact_permit(
            &permits,
            &MailAddressBookContractV1::EntryObserved.reference(),
        )?,
        mail_entry_upsert_rejected: exact_permit(
            &permits,
            &MailAddressBookContractV1::EntryUpsertRejected.reference(),
        )?,
        mail_entry_upserted: exact_permit(
            &permits,
            &MailAddressBookContractV1::EntryUpserted.reference(),
        )?,
        mail_page_completed: exact_permit(
            &permits,
            &MailAddressBookContractV1::PageCompleted.reference(),
        )?,
        mail_page_rejected: exact_permit(
            &permits,
            &MailAddressBookContractV1::PageRejected.reference(),
        )?,
        scheduler_due: exact_permit(&permits, &super::admission::scheduler_job_contract_v1())?,
    })
}

fn exact_permit(
    permits: &[RuntimeSubscribePermitV1],
    contract: &ContractReferenceV1,
) -> Result<RuntimeSubscribePermitV1, MailContactsSyncManagedRuntimeErrorV1> {
    let mut matching = permits.iter().filter(|permit| {
        permit.contract().is_some_and(|actual| {
            actual.owner == contract.owner
                && actual.name == contract.name
                && actual.major == contract.major
                && actual.revision == contract.revision
                && actual.schema_sha256 == contract.schema_sha256
        })
    });
    let permit = matching
        .next()
        .cloned()
        .ok_or(MailContactsSyncManagedRuntimeErrorV1::Admission)?;
    if matching.next().is_some() {
        return Err(MailContactsSyncManagedRuntimeErrorV1::Admission);
    }
    Ok(permit)
}

fn validate_admission(
    admission: &MailContactsSyncRuntimeAdmissionV1,
    configurations: &[(String, MailContactsSyncRuntimeSettingsV1)],
) -> Result<(), MailContactsSyncManagedRuntimeErrorV1> {
    let ordered = configurations.windows(2).all(|pair| pair[0].0 < pair[1].0);
    let unique_accounts = configurations
        .iter()
        .map(|(_, settings)| settings.account_id.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == configurations.len();
    if admission.logical_owner_id.is_empty()
        || admission.logical_owner_id == MAIL_CONTACTS_SYNC_OWNER_ID_V1
        || admission.registration_id.is_empty()
        || admission.runtime_instance_id.is_empty()
        || admission.runtime_generation == 0
        || admission.grant_epoch == 0
        || configurations.is_empty()
        || configurations.len() > 32
        || !ordered
        || !unique_accounts
    {
        return Err(MailContactsSyncManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn authenticate(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    descriptor: Vec<u8>,
    settings: Vec<u8>,
    admission: &MailContactsSyncRuntimeAdmissionV1,
) -> Result<(), MailContactsSyncManagedRuntimeErrorV1> {
    channel
        .inner_mut()
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .and_then(|_| {
            channel
                .inner_mut()
                .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        })
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
    let response = channel
        .describe_managed_runtime(descriptor, settings)
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
    if response.registration_id != admission.registration_id
        || response.runtime_generation != admission.runtime_generation
        || response.grant_epoch != admission.grant_epoch
    {
        return Err(MailContactsSyncManagedRuntimeErrorV1::Admission);
    }
    Ok(())
}

fn signal_ready(
    channel: &mut ManagedControlChannelV2<UnixStream>,
    admission: &MailContactsSyncRuntimeAdmissionV1,
) -> Result<(), MailContactsSyncManagedRuntimeErrorV1> {
    channel
        .signal_ready(ManagedRuntimeReadyRequestV1 {
            registration_id: admission.registration_id.clone(),
            runtime_generation: admission.runtime_generation,
            grant_epoch: admission.grant_epoch,
        })
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)?;
    channel
        .inner_mut()
        .set_read_timeout(None)
        .and_then(|_| channel.inner_mut().set_write_timeout(None))
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Unavailable)
}

async fn resolve_storage_credential(
    leases: &mut StorageVaultLeaseAdapterV1<InheritedKernelVaultRouteV2>,
    binding: &StorageBindingV1,
) -> Result<zeroize::Zeroizing<Vec<u8>>, MailContactsSyncManagedRuntimeErrorV1> {
    for attempt in 0..20 {
        if let Ok(password) = leases.ensure_runtime_credential(binding).await {
            return Ok(password);
        }
        if attempt < 19 {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    Err(MailContactsSyncManagedRuntimeErrorV1::Unavailable)
}

fn storage_binding(
    configuration: &ManagedStorageRuntimeConfigurationV1,
    admission: &MailContactsSyncRuntimeAdmissionV1,
) -> Result<StorageBindingV1, MailContactsSyncManagedRuntimeErrorV1> {
    if configuration.runtime_instance_id != admission.runtime_instance_id
        || configuration.logical_owner_id != MAIL_CONTACTS_SYNC_OWNER_ID_V1
        || configuration.owner != MAIL_CONTACTS_SYNC_OWNER_ID_V1
        || configuration.storage_bundle_digest.len() != 32
        || configuration.storage_generation == 0
        || configuration.credential_revision == 0
        || configuration.role_epoch == 0
        || configuration.storage_bundle_revision == 0
    {
        return Err(MailContactsSyncManagedRuntimeErrorV1::Admission);
    }
    let identity = StorageBindingIdentityV1::new(
        configuration.storage_instance_id.clone(),
        configuration.database_id.clone(),
        configuration.owner.clone(),
        admission.registration_id.clone(),
        configuration.runtime_instance_id.clone(),
    )
    .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
    let fences = StorageBindingFencesV1::new(
        configuration.storage_generation,
        admission.runtime_generation,
        admission.grant_epoch,
        configuration.role_epoch,
        configuration.credential_revision,
        configuration.storage_bundle_revision,
    )
    .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
    let budgets = StorageEffectiveBudgetsV1::new(
        u16::try_from(configuration.max_connections)
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?,
        configuration.statement_timeout_millis,
    )
    .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
    let access = StorageBindingAccessV1::new(
        configuration.runtime_principal.clone(),
        configuration.pool_alias.clone(),
        budgets,
        configuration
            .storage_bundle_digest
            .as_slice()
            .try_into()
            .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?,
    )
    .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)?;
    StorageBindingV1::new(identity, fences, access)
        .map_err(|_| MailContactsSyncManagedRuntimeErrorV1::Admission)
}

fn runtime_instance_bytes(value: &str) -> [u8; 16] {
    Sha256::digest(value.as_bytes())[..16]
        .try_into()
        .expect("SHA-256 prefix has exact length")
}

fn contacts_error(
    error: MailContactsSyncContactsResultErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncContactsResultErrorV1::InvalidEnvelope
        | MailContactsSyncContactsResultErrorV1::InvalidPayload => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncContactsResultErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncContactsResultErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn reverse_change_error(
    error: MailContactsSyncReverseChangeErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncReverseChangeErrorV1::InvalidEnvelope
        | MailContactsSyncReverseChangeErrorV1::InvalidPayload => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncReverseChangeErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncReverseChangeErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn source_result_error(
    error: MailContactsSyncSourceResultErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncSourceResultErrorV1::InvalidEnvelope
        | MailContactsSyncSourceResultErrorV1::InvalidPayload => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncSourceResultErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncSourceResultErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn provider_error(
    error: MailContactsSyncProviderEventErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncProviderEventErrorV1::InvalidEnvelope
        | MailContactsSyncProviderEventErrorV1::InvalidPayload => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncProviderEventErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncProviderEventErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn scheduled_error(
    error: MailContactsSyncScheduledExecutionErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncScheduledExecutionErrorV1::InvalidDue
        | MailContactsSyncScheduledExecutionErrorV1::LeaseExpired
        | MailContactsSyncScheduledExecutionErrorV1::CommandBuild => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncScheduledExecutionErrorV1::Persistence => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(
                MailContactsSyncPersistenceErrorV1::StorageUnavailable,
            )
        }
        MailContactsSyncScheduledExecutionErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn provider_write_result_error(
    error: MailContactsSyncProviderWriteResultErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncProviderWriteResultErrorV1::InvalidEnvelope
        | MailContactsSyncProviderWriteResultErrorV1::InvalidPayload => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncProviderWriteResultErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncProviderWriteResultErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn provider_link_result_error(
    error: MailContactsSyncProviderLinkResultErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncProviderLinkResultErrorV1::InvalidEnvelope
        | MailContactsSyncProviderLinkResultErrorV1::InvalidPayload => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncProviderLinkResultErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncProviderLinkResultErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn scheduled_completion_error(
    error: crate::MailContactsSyncScheduledCompletionErrorV1,
) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        crate::MailContactsSyncScheduledCompletionErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        crate::MailContactsSyncScheduledCompletionErrorV1::InvalidTime
        | crate::MailContactsSyncScheduledCompletionErrorV1::ReceiptBuild => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
    }
}

fn relay_error(error: MailContactsSyncRelayErrorV1) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncRelayErrorV1::InvalidTimestamp => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncRelayErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncRelayErrorV1::EventUnavailable => {
            MailContactsSyncManagedRuntimeErrorV1::EventUnavailable
        }
    }
}

fn realtime_error(error: MailContactsSyncRealtimeErrorV1) -> MailContactsSyncManagedRuntimeErrorV1 {
    match error {
        MailContactsSyncRealtimeErrorV1::InvalidTransition => {
            MailContactsSyncManagedRuntimeErrorV1::EventContract
        }
        MailContactsSyncRealtimeErrorV1::Persistence(error) => {
            MailContactsSyncManagedRuntimeErrorV1::Persistence(error)
        }
        MailContactsSyncRealtimeErrorV1::Unavailable => {
            MailContactsSyncManagedRuntimeErrorV1::Unavailable
        }
    }
}

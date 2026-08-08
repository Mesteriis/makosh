use makosh_mail_api::client_contract::{
    MAIL_CLIENT_CONTRACT_MAJOR, MAIL_CLIENT_CONTRACT_REVISION, MAIL_CLIENT_DESCRIPTOR_SET_V1,
    MAIL_MODULE_ID, MAIL_OWNER_ID, MailClientContractV1,
};
use makosh_mail_api::{
    MailClientRequestV1, MailClientResponseV1, account_lifecycle_wire, account_wire, client_wire,
    composition_wire, message_flags_wire, message_location_wire, message_permanent_delete_wire,
    oauth_wire, operational_wire, sync_health_wire,
};
use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::managed::MailAdmittedRuntime;

const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailClientPortErrorV1 {
    Protocol,
    Runtime,
}

fn mail_client_contract(contract: MailClientContractV1) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: MAIL_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: MAIL_CLIENT_CONTRACT_MAJOR,
        revision: MAIL_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(MAIL_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn validate_contract(
    reference: &ContractReferenceV1,
) -> Result<MailClientContractV1, MailClientPortErrorV1> {
    let contract = MailClientContractV1::from_contract_name(&reference.name)
        .ok_or(MailClientPortErrorV1::Protocol)?;
    if reference != &mail_client_contract(contract) {
        return Err(MailClientPortErrorV1::Protocol);
    }
    Ok(contract)
}

fn request_contract(request: &MailClientRequestV1) -> MailClientContractV1 {
    match request {
        MailClientRequestV1::AccountCatalog(_) => MailClientContractV1::AccountCatalog,
        MailClientRequestV1::BindCredential(_) => MailClientContractV1::AccountCredentialBind,
        MailClientRequestV1::AccountStatus(_) => MailClientContractV1::AccountQuery,
        MailClientRequestV1::RetireAccount(_) => MailClientContractV1::AccountRetire,
        MailClientRequestV1::DeleteAccount(_) => MailClientContractV1::AccountDelete,
        MailClientRequestV1::RetryAccountLifecycle(_) => {
            MailClientContractV1::AccountLifecycleRetry
        }
        MailClientRequestV1::AccountLifecycleStatus(_) => {
            MailClientContractV1::AccountLifecycleQuery
        }
        MailClientRequestV1::SyncInbox(_) => MailClientContractV1::Sync,
        MailClientRequestV1::SendMail(_) => MailClientContractV1::Delivery,
        MailClientRequestV1::DeliveryStatus(_) => MailClientContractV1::DeliveryQuery,
        MailClientRequestV1::GmailOAuthStart(_) => MailClientContractV1::GmailOAuthStart,
        MailClientRequestV1::GmailOAuthComplete(_) => MailClientContractV1::GmailOAuthComplete,
        MailClientRequestV1::GmailOAuthRefresh(_) => MailClientContractV1::GmailOAuthRefresh,
        MailClientRequestV1::GmailOAuthStatus(_) => MailClientContractV1::GmailOAuthQuery,
        MailClientRequestV1::CompositionCommand(_) => MailClientContractV1::CompositionCommand,
        MailClientRequestV1::CompositionQuery(_) => MailClientContractV1::CompositionQuery,
        MailClientRequestV1::MessageFlagCommand(_) => MailClientContractV1::MessageFlagCommand,
        MailClientRequestV1::MessageFlagStatus(_) => MailClientContractV1::MessageFlagQuery,
        MailClientRequestV1::MessageLocationCommand(_) => {
            MailClientContractV1::MessageLocationCommand
        }
        MailClientRequestV1::MessageLocationStatus(_) => MailClientContractV1::MessageLocationQuery,
        MailClientRequestV1::MessagePermanentDeleteCommand(_) => {
            MailClientContractV1::MessagePermanentDeleteCommand
        }
        MailClientRequestV1::MessagePermanentDeleteStatus(_) => {
            MailClientContractV1::MessagePermanentDeleteQuery
        }
        MailClientRequestV1::OperationalQuery(_) => MailClientContractV1::OperationalQuery,
        MailClientRequestV1::SyncHealthQuery(_) => MailClientContractV1::SyncHealthQuery,
    }
}

fn encode_request_payload(request: &MailClientRequestV1) -> Result<Vec<u8>, MailClientPortErrorV1> {
    match request {
        MailClientRequestV1::AccountCatalog(value) => {
            Ok(account_wire::encode_catalog_request(value))
        }
        MailClientRequestV1::BindCredential(value) => {
            account_wire::encode_bind_request(value).map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::AccountStatus(value) => {
            account_wire::encode_status_request(value).map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::RetireAccount(value) | MailClientRequestV1::DeleteAccount(value) => {
            account_lifecycle_wire::encode_command(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::RetryAccountLifecycle(value) => {
            account_lifecycle_wire::encode_retry(value).map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::AccountLifecycleStatus(value) => {
            account_lifecycle_wire::encode_status_request(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::SyncInbox(value) => Ok(client_wire::encode_sync_request(value)),
        MailClientRequestV1::SendMail(value) => Ok(client_wire::encode_delivery_request(value)),
        MailClientRequestV1::DeliveryStatus(value) => {
            Ok(client_wire::encode_delivery_status_request(value))
        }
        MailClientRequestV1::GmailOAuthStart(value) => Ok(oauth_wire::encode_start_request(value)),
        MailClientRequestV1::GmailOAuthComplete(value) => {
            Ok(oauth_wire::encode_complete_request(value))
        }
        MailClientRequestV1::GmailOAuthRefresh(value) => {
            Ok(oauth_wire::encode_refresh_request(value))
        }
        MailClientRequestV1::GmailOAuthStatus(value) => {
            Ok(oauth_wire::encode_status_request(value))
        }
        MailClientRequestV1::CompositionCommand(value) => {
            composition_wire::encode_composition_command(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::CompositionQuery(value) => {
            composition_wire::encode_composition_query(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::MessageFlagCommand(value) => {
            message_flags_wire::encode_message_flag_command(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::MessageFlagStatus(value) => {
            message_flags_wire::encode_message_flag_status_request(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::MessageLocationCommand(value) => {
            message_location_wire::encode_message_location_command(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::MessageLocationStatus(value) => {
            message_location_wire::encode_message_location_status_request(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::MessagePermanentDeleteCommand(value) => {
            message_permanent_delete_wire::encode_message_permanent_delete_command(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::MessagePermanentDeleteStatus(value) => {
            message_permanent_delete_wire::encode_message_permanent_delete_status_request(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::OperationalQuery(value) => {
            operational_wire::encode_operational_query(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientRequestV1::SyncHealthQuery(value) => {
            sync_health_wire::encode_sync_health_query(value)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
    }
}

fn decode_request_payload(
    contract: MailClientContractV1,
    bytes: &[u8],
) -> Result<MailClientRequestV1, MailClientPortErrorV1> {
    match contract {
        MailClientContractV1::AccountCatalog => account_wire::decode_catalog_request(bytes)
            .map(MailClientRequestV1::AccountCatalog)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::AccountCredentialBind => account_wire::decode_bind_request(bytes)
            .map(MailClientRequestV1::BindCredential)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::AccountQuery => account_wire::decode_status_request(bytes)
            .map(MailClientRequestV1::AccountStatus)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::AccountRetire => account_lifecycle_wire::decode_command(bytes)
            .map(MailClientRequestV1::RetireAccount)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::AccountDelete => account_lifecycle_wire::decode_command(bytes)
            .map(MailClientRequestV1::DeleteAccount)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::AccountLifecycleRetry => account_lifecycle_wire::decode_retry(bytes)
            .map(MailClientRequestV1::RetryAccountLifecycle)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::AccountLifecycleQuery => {
            account_lifecycle_wire::decode_status_request(bytes)
                .map(MailClientRequestV1::AccountLifecycleStatus)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::Sync => client_wire::decode_sync_request(bytes)
            .map(MailClientRequestV1::SyncInbox)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::Delivery => client_wire::decode_delivery_request(bytes)
            .map(MailClientRequestV1::SendMail)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::DeliveryQuery => client_wire::decode_delivery_status_request(bytes)
            .map(MailClientRequestV1::DeliveryStatus)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthStart => oauth_wire::decode_start_request(bytes)
            .map(MailClientRequestV1::GmailOAuthStart)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthComplete => oauth_wire::decode_complete_request(bytes)
            .map(MailClientRequestV1::GmailOAuthComplete)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthRefresh => oauth_wire::decode_refresh_request(bytes)
            .map(MailClientRequestV1::GmailOAuthRefresh)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::GmailOAuthQuery => oauth_wire::decode_status_request(bytes)
            .map(MailClientRequestV1::GmailOAuthStatus)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::CompositionCommand => {
            composition_wire::decode_composition_command(bytes)
                .map(MailClientRequestV1::CompositionCommand)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::CompositionQuery => composition_wire::decode_composition_query(bytes)
            .map(MailClientRequestV1::CompositionQuery)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::MessageFlagCommand => {
            message_flags_wire::decode_message_flag_command(bytes)
                .map(MailClientRequestV1::MessageFlagCommand)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::MessageFlagQuery => {
            message_flags_wire::decode_message_flag_status_request(bytes)
                .map(MailClientRequestV1::MessageFlagStatus)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::MessageLocationCommand => {
            message_location_wire::decode_message_location_command(bytes)
                .map(MailClientRequestV1::MessageLocationCommand)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::MessageLocationQuery => {
            message_location_wire::decode_message_location_status_request(bytes)
                .map(MailClientRequestV1::MessageLocationStatus)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::MessagePermanentDeleteCommand => {
            message_permanent_delete_wire::decode_message_permanent_delete_command(bytes)
                .map(MailClientRequestV1::MessagePermanentDeleteCommand)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::MessagePermanentDeleteQuery => {
            message_permanent_delete_wire::decode_message_permanent_delete_status_request(bytes)
                .map(MailClientRequestV1::MessagePermanentDeleteStatus)
                .map_err(|_| MailClientPortErrorV1::Protocol)
        }
        MailClientContractV1::OperationalQuery => operational_wire::decode_operational_query(bytes)
            .map(MailClientRequestV1::OperationalQuery)
            .map_err(|_| MailClientPortErrorV1::Protocol),
        MailClientContractV1::SyncHealthQuery => sync_health_wire::decode_sync_health_query(bytes)
            .map(MailClientRequestV1::SyncHealthQuery)
            .map_err(|_| MailClientPortErrorV1::Protocol),
    }
}

pub fn encode_module_request(
    request_id: u64,
    request: &MailClientRequestV1,
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    if request_id == 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let contract = request_contract(request);
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: MAIL_MODULE_ID.to_owned(),
        owner_id: MAIL_OWNER_ID.to_owned(),
        contract: Some(mail_client_contract(contract)),
        request_id,
        request_payload: encode_request_payload(request)?,
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_request(
    bytes: &[u8],
) -> Result<(u64, MailClientContractV1, MailClientRequestV1), MailClientPortErrorV1> {
    let envelope =
        ModuleClientRequestV1::decode(bytes).map_err(|_| MailClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != MAIL_MODULE_ID
        || envelope.owner_id != MAIL_OWNER_ID
        || envelope.request_id == 0
    {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let contract = validate_contract(
        envelope
            .contract
            .as_ref()
            .ok_or(MailClientPortErrorV1::Protocol)?,
    )?;
    let request = decode_request_payload(contract, &envelope.request_payload)?;
    Ok((envelope.request_id, contract, request))
}

pub async fn handle_client_request(
    runtime: &mut MailAdmittedRuntime,
    bytes: &[u8],
    requested_at_unix_seconds: i64,
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    if requested_at_unix_seconds <= 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let (request_id, contract, request) = decode_module_request(bytes)?;
    if let Some(connection_id) = request_connection_id(&request) {
        runtime
            .select_account(connection_id)
            .map_err(|_| MailClientPortErrorV1::Runtime)?;
    }
    let response = match request {
        MailClientRequestV1::AccountCatalog(_) => {
            let catalog = runtime
                .account_catalog()
                .await
                .map_err(|_| MailClientPortErrorV1::Runtime)?;
            MailClientResponseV1::AccountCatalog(catalog)
        }
        MailClientRequestV1::BindCredential(value) => runtime
            .bind_account_credential(&value, requested_at_unix_seconds)
            .await
            .map(MailClientResponseV1::CredentialBinding)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::AccountStatus(value) => runtime
            .account_status(&value.connection_id)
            .await
            .map(MailClientResponseV1::AccountStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::RetireAccount(value) => runtime
            .apply_account_lifecycle(
                &value,
                makosh_mail_api::account_lifecycle::MailAccountLifecycleActionV1::Retire,
                requested_at_unix_seconds,
            )
            .await
            .map(MailClientResponseV1::AccountLifecycle)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::DeleteAccount(value) => runtime
            .apply_account_lifecycle(
                &value,
                makosh_mail_api::account_lifecycle::MailAccountLifecycleActionV1::Delete,
                requested_at_unix_seconds,
            )
            .await
            .map(MailClientResponseV1::AccountLifecycle)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::RetryAccountLifecycle(value) => runtime
            .retry_account_lifecycle(&value, requested_at_unix_seconds)
            .await
            .map(MailClientResponseV1::AccountLifecycle)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::AccountLifecycleStatus(value) => runtime
            .account_lifecycle_status(&value)
            .await
            .map(MailClientResponseV1::AccountLifecycle)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::SyncInbox(value) => {
            runtime
                .accept_sync_operation(&value.operation_id, requested_at_unix_seconds)
                .await
                .map_err(|_| MailClientPortErrorV1::Runtime)?;
            MailClientResponseV1::SyncInboxAccepted {
                operation_id: value.operation_id,
            }
        }
        MailClientRequestV1::SendMail(value) => runtime
            .submit_delivery(&value, requested_at_unix_seconds)
            .await
            .map(|operation_id| MailClientResponseV1::MailAccepted { operation_id })
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::DeliveryStatus(value) => runtime
            .delivery_operation_status(&value.operation_id)
            .await
            .map(MailClientResponseV1::DeliveryStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthStart(value) => runtime
            .start_gmail_oauth(
                &value.operation_id,
                value.authority,
                requested_at_unix_seconds,
            )
            .await
            .map(MailClientResponseV1::GmailOAuthStarted)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthComplete(value) => runtime
            .submit_gmail_oauth_complete(&value, requested_at_unix_seconds)
            .await
            .map(|operation_id| MailClientResponseV1::GmailOAuthAccepted { operation_id })
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthRefresh(value) => runtime
            .submit_gmail_oauth_refresh(&value.operation_id, requested_at_unix_seconds)
            .await
            .map(|operation_id| MailClientResponseV1::GmailOAuthAccepted { operation_id })
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::GmailOAuthStatus(value) => runtime
            .gmail_oauth_operation_status(&value.operation_id)
            .await
            .map(MailClientResponseV1::GmailOAuthStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::CompositionCommand(value) => runtime
            .composition_command(&value, requested_at_unix_seconds)
            .await
            .map(MailClientResponseV1::CompositionMutation)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::CompositionQuery(value) => runtime
            .composition_query(&value)
            .await
            .map(MailClientResponseV1::CompositionQuery)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::MessageFlagCommand(value) => runtime
            .submit_message_flag_command(&value, requested_at_unix_seconds)
            .await
            .map(MailClientResponseV1::MessageFlagAccepted)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::MessageFlagStatus(value) => runtime
            .message_flag_operation_status(&value)
            .await
            .map(MailClientResponseV1::MessageFlagStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::MessageLocationCommand(value) => runtime
            .submit_message_location_command(&value, requested_at_unix_seconds)
            .await
            .map(MailClientResponseV1::MessageLocationAccepted)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::MessageLocationStatus(value) => runtime
            .message_location_operation_status(&value)
            .await
            .map(MailClientResponseV1::MessageLocationStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::MessagePermanentDeleteCommand(value) => runtime
            .submit_message_permanent_delete_command(&value, requested_at_unix_seconds)
            .await
            .map(MailClientResponseV1::MessagePermanentDeleteAccepted)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::MessagePermanentDeleteStatus(value) => runtime
            .message_permanent_delete_operation_status(&value)
            .await
            .map(MailClientResponseV1::MessagePermanentDeleteStatus)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::OperationalQuery(value) => runtime
            .operational_query(&value)
            .await
            .map(MailClientResponseV1::OperationalQuery)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
        MailClientRequestV1::SyncHealthQuery(value) => runtime
            .sync_health_query(&value)
            .await
            .map(MailClientResponseV1::SyncHealthQuery)
            .map_err(|_| MailClientPortErrorV1::Runtime)?,
    };
    encode_module_response(request_id, contract, &response)
}

fn request_connection_id(request: &MailClientRequestV1) -> Option<&str> {
    match request {
        MailClientRequestV1::AccountCatalog(_) => None,
        MailClientRequestV1::BindCredential(value) => Some(&value.connection_id),
        MailClientRequestV1::AccountStatus(value) => Some(&value.connection_id),
        MailClientRequestV1::RetireAccount(value) | MailClientRequestV1::DeleteAccount(value) => {
            Some(&value.connection_id)
        }
        MailClientRequestV1::RetryAccountLifecycle(value) => Some(&value.connection_id),
        MailClientRequestV1::AccountLifecycleStatus(value) => Some(&value.connection_id),
        MailClientRequestV1::SyncInbox(value) => Some(&value.connection_id),
        MailClientRequestV1::SendMail(value) => Some(&value.connection_id),
        MailClientRequestV1::DeliveryStatus(value) => Some(&value.connection_id),
        MailClientRequestV1::GmailOAuthStart(value) => Some(&value.connection_id),
        MailClientRequestV1::GmailOAuthComplete(value) => Some(&value.connection_id),
        MailClientRequestV1::GmailOAuthRefresh(value) => Some(&value.connection_id),
        MailClientRequestV1::GmailOAuthStatus(value) => Some(&value.connection_id),
        MailClientRequestV1::CompositionCommand(value) => {
            Some(makosh_mail_api::composition::composition_command_connection_id(value))
        }
        MailClientRequestV1::CompositionQuery(value) => {
            Some(makosh_mail_api::composition::composition_query_connection_id(value))
        }
        MailClientRequestV1::MessageFlagCommand(value) => Some(&value.connection_id),
        MailClientRequestV1::MessageFlagStatus(value) => Some(&value.connection_id),
        MailClientRequestV1::MessageLocationCommand(value) => Some(&value.connection_id),
        MailClientRequestV1::MessageLocationStatus(value) => Some(&value.connection_id),
        MailClientRequestV1::MessagePermanentDeleteCommand(value) => Some(&value.connection_id),
        MailClientRequestV1::MessagePermanentDeleteStatus(value) => Some(&value.connection_id),
        MailClientRequestV1::OperationalQuery(value) => {
            Some(makosh_mail_api::operational::operational_query_connection_id(value))
        }
        MailClientRequestV1::SyncHealthQuery(value) => {
            Some(makosh_mail_api::sync_health::sync_health_query_connection_id(value))
        }
    }
}

fn encode_module_response(
    request_id: u64,
    contract: MailClientContractV1,
    response: &MailClientResponseV1,
) -> Result<Vec<u8>, MailClientPortErrorV1> {
    if request_id == 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let response_payload = match (contract, response) {
        (MailClientContractV1::AccountCatalog, MailClientResponseV1::AccountCatalog(catalog)) => {
            account_wire::encode_account_catalog(catalog)
                .map_err(|_| MailClientPortErrorV1::Protocol)?
        }
        (
            MailClientContractV1::AccountCredentialBind,
            MailClientResponseV1::CredentialBinding(receipt),
        ) => account_wire::encode_binding_receipt(receipt)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (MailClientContractV1::AccountQuery, MailClientResponseV1::AccountStatus(status)) => {
            account_wire::encode_account_status(status)
                .map_err(|_| MailClientPortErrorV1::Protocol)?
        }
        (
            MailClientContractV1::AccountRetire
            | MailClientContractV1::AccountDelete
            | MailClientContractV1::AccountLifecycleRetry
            | MailClientContractV1::AccountLifecycleQuery,
            MailClientResponseV1::AccountLifecycle(receipt),
        ) => account_lifecycle_wire::encode_receipt(receipt)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (MailClientContractV1::Sync, MailClientResponseV1::SyncInboxAccepted { operation_id }) => {
            client_wire::encode_sync_response(operation_id)
        }
        (MailClientContractV1::Delivery, MailClientResponseV1::MailAccepted { operation_id }) => {
            client_wire::encode_delivery_response(operation_id)
        }
        (MailClientContractV1::DeliveryQuery, MailClientResponseV1::DeliveryStatus(status)) => {
            client_wire::encode_delivery_status_response(status.as_ref())
        }
        (
            MailClientContractV1::GmailOAuthStart,
            MailClientResponseV1::GmailOAuthStarted(response),
        ) => oauth_wire::encode_start_response(response),
        (
            MailClientContractV1::GmailOAuthComplete | MailClientContractV1::GmailOAuthRefresh,
            MailClientResponseV1::GmailOAuthAccepted { operation_id },
        ) => oauth_wire::encode_accepted_response(operation_id),
        (MailClientContractV1::GmailOAuthQuery, MailClientResponseV1::GmailOAuthStatus(status)) => {
            oauth_wire::encode_status_response(status.as_ref())
        }
        (
            MailClientContractV1::CompositionCommand,
            MailClientResponseV1::CompositionMutation(receipt),
        ) => composition_wire::encode_composition_receipt(receipt)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::CompositionQuery,
            MailClientResponseV1::CompositionQuery(response),
        ) => composition_wire::encode_composition_query_response(response)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::MessageFlagCommand,
            MailClientResponseV1::MessageFlagAccepted(accepted),
        ) => message_flags_wire::encode_message_flag_accepted(accepted)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::MessageFlagQuery,
            MailClientResponseV1::MessageFlagStatus(status),
        ) => message_flags_wire::encode_message_flag_status_response(status.as_ref())
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::MessageLocationCommand,
            MailClientResponseV1::MessageLocationAccepted(accepted),
        ) => message_location_wire::encode_message_location_accepted(accepted)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::MessageLocationQuery,
            MailClientResponseV1::MessageLocationStatus(status),
        ) => message_location_wire::encode_message_location_status_response(status.as_ref())
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::MessagePermanentDeleteCommand,
            MailClientResponseV1::MessagePermanentDeleteAccepted(accepted),
        ) => message_permanent_delete_wire::encode_message_permanent_delete_accepted(accepted)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::MessagePermanentDeleteQuery,
            MailClientResponseV1::MessagePermanentDeleteStatus(status),
        ) => message_permanent_delete_wire::encode_message_permanent_delete_status_response(
            status.as_ref(),
        )
        .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::OperationalQuery,
            MailClientResponseV1::OperationalQuery(response),
        ) => operational_wire::encode_operational_query_response(response)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        (
            MailClientContractV1::SyncHealthQuery,
            MailClientResponseV1::SyncHealthQuery(response),
        ) => sync_health_wire::encode_sync_health_response(response)
            .map_err(|_| MailClientPortErrorV1::Protocol)?,
        _ => return Err(MailClientPortErrorV1::Protocol),
    };
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub fn decode_module_response(
    contract: MailClientContractV1,
    bytes: &[u8],
) -> Result<(u64, MailClientResponseV1), MailClientPortErrorV1> {
    let envelope =
        ModuleClientResponseV1::decode(bytes).map_err(|_| MailClientPortErrorV1::Protocol)?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR || envelope.request_id == 0 {
        return Err(MailClientPortErrorV1::Protocol);
    }
    if !envelope.error_code.is_empty() {
        return if envelope.response_payload.is_empty()
            && matches!(
                envelope.error_code.as_str(),
                "INVALID_ARGUMENT" | "REJECTED" | "RUNTIME_UNAVAILABLE"
            ) {
            Err(MailClientPortErrorV1::Runtime)
        } else {
            Err(MailClientPortErrorV1::Protocol)
        };
    }
    if envelope.response_payload.is_empty() {
        return Err(MailClientPortErrorV1::Protocol);
    }
    let response = match contract {
        MailClientContractV1::AccountCatalog => {
            account_wire::decode_account_catalog(&envelope.response_payload)
                .map(MailClientResponseV1::AccountCatalog)
        }
        MailClientContractV1::AccountCredentialBind => {
            account_wire::decode_binding_receipt(&envelope.response_payload)
                .map(MailClientResponseV1::CredentialBinding)
        }
        MailClientContractV1::AccountQuery => {
            account_wire::decode_account_status(&envelope.response_payload)
                .map(MailClientResponseV1::AccountStatus)
        }
        MailClientContractV1::AccountRetire
        | MailClientContractV1::AccountDelete
        | MailClientContractV1::AccountLifecycleRetry
        | MailClientContractV1::AccountLifecycleQuery => {
            account_lifecycle_wire::decode_receipt(&envelope.response_payload)
                .map(MailClientResponseV1::AccountLifecycle)
        }
        MailClientContractV1::Sync => client_wire::decode_sync_response(&envelope.response_payload),
        MailClientContractV1::Delivery => {
            client_wire::decode_delivery_response(&envelope.response_payload)
        }
        MailClientContractV1::DeliveryQuery => {
            client_wire::decode_delivery_status_response(&envelope.response_payload)
        }
        MailClientContractV1::GmailOAuthStart => {
            oauth_wire::decode_start_response(&envelope.response_payload)
        }
        MailClientContractV1::GmailOAuthComplete | MailClientContractV1::GmailOAuthRefresh => {
            oauth_wire::decode_accepted_response(&envelope.response_payload)
        }
        MailClientContractV1::GmailOAuthQuery => {
            oauth_wire::decode_status_response(&envelope.response_payload)
        }
        MailClientContractV1::CompositionCommand => {
            composition_wire::decode_composition_receipt(&envelope.response_payload)
                .map(MailClientResponseV1::CompositionMutation)
        }
        MailClientContractV1::CompositionQuery => {
            composition_wire::decode_composition_query_response(&envelope.response_payload)
                .map(MailClientResponseV1::CompositionQuery)
        }
        MailClientContractV1::MessageFlagCommand => {
            message_flags_wire::decode_message_flag_accepted(&envelope.response_payload)
                .map(MailClientResponseV1::MessageFlagAccepted)
        }
        MailClientContractV1::MessageFlagQuery => {
            message_flags_wire::decode_message_flag_status_response(&envelope.response_payload)
                .map(MailClientResponseV1::MessageFlagStatus)
        }
        MailClientContractV1::MessageLocationCommand => {
            message_location_wire::decode_message_location_accepted(&envelope.response_payload)
                .map(MailClientResponseV1::MessageLocationAccepted)
        }
        MailClientContractV1::MessageLocationQuery => {
            message_location_wire::decode_message_location_status_response(
                &envelope.response_payload,
            )
            .map(MailClientResponseV1::MessageLocationStatus)
        }
        MailClientContractV1::MessagePermanentDeleteCommand => {
            message_permanent_delete_wire::decode_message_permanent_delete_accepted(
                &envelope.response_payload,
            )
            .map(MailClientResponseV1::MessagePermanentDeleteAccepted)
        }
        MailClientContractV1::MessagePermanentDeleteQuery => {
            message_permanent_delete_wire::decode_message_permanent_delete_status_response(
                &envelope.response_payload,
            )
            .map(MailClientResponseV1::MessagePermanentDeleteStatus)
        }
        MailClientContractV1::OperationalQuery => {
            operational_wire::decode_operational_query_response(&envelope.response_payload)
                .map(MailClientResponseV1::OperationalQuery)
        }
        MailClientContractV1::SyncHealthQuery => {
            sync_health_wire::decode_sync_health_response(&envelope.response_payload)
                .map(MailClientResponseV1::SyncHealthQuery)
        }
    }
    .map_err(|_| MailClientPortErrorV1::Protocol)?;
    Ok((envelope.request_id, response))
}

#[cfg(test)]
mod tests {
    use makosh_mail_api::{
        MailDeliveryStatusRequestV1, MailSendMailRequestV1, MailSyncInboxRequestV1,
        account::{
            MailAccountCatalogRequestV1, MailAccountCatalogV1, MailAccountReadinessV1,
            MailAccountStatusRequestV1, MailAccountStatusV1, MailBindCredentialRequestV1,
            MailConnectorProfileV1, MailCredentialPurposeV1, MailProviderPathReadinessV1,
        },
        account_lifecycle::{
            MailAccountLifecycleCommandV1, MailAccountLifecycleRetryV1,
            MailAccountLifecycleStatusRequestV1,
        },
        message_flags::{
            MailMessageFlagAcceptedV1, MailMessageFlagCommandV1, MailMessageFlagKindV1,
            MailMessageFlagOperationOutcomeV1, MailMessageFlagOperationStatusV1,
            MailMessageFlagStatusRequestV1,
        },
        message_location::{
            MailMessageLocationAcceptedV1, MailMessageLocationCommandV1, MailMessageLocationKindV1,
            MailMessageLocationOperationOutcomeV1, MailMessageLocationOperationStatusV1,
            MailMessageLocationStatusRequestV1,
        },
        message_permanent_delete::{
            MailMessagePermanentDeleteAcceptedV1, MailMessagePermanentDeleteCommandV1,
            MailMessagePermanentDeleteConfirmationV1, MailMessagePermanentDeleteOperationOutcomeV1,
            MailMessagePermanentDeleteOperationStatusV1, MailMessagePermanentDeleteStatusRequestV1,
        },
        operational::{
            MailOperationalPageV1, MailOperationalQueryResponseV1, MailOperationalQueryV1,
        },
        sync_health::{
            MailSyncHealthQueryResponseV1, MailSyncHealthQueryV1, MailSyncProviderPathReadinessV1,
            MailSyncStatusV1,
        },
    };

    use super::*;

    fn sync_request() -> MailClientRequestV1 {
        MailClientRequestV1::SyncInbox(MailSyncInboxRequestV1 {
            operation_id: "sync-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
        })
    }

    fn delivery_request() -> MailClientRequestV1 {
        MailClientRequestV1::SendMail(MailSendMailRequestV1 {
            operation_id: "delivery-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
            provider_conversation_id: "conversation".to_owned(),
            recipients: vec!["recipient@example.com".to_owned()],
            cc_recipients: Vec::new(),
            bcc_recipients: Vec::new(),
            subject: "subject".to_owned(),
            text_body: "body".to_owned(),
            attachment_anchor_ids: Vec::new(),
        })
    }

    fn delivery_query() -> MailClientRequestV1 {
        MailClientRequestV1::DeliveryStatus(MailDeliveryStatusRequestV1 {
            operation_id: "delivery-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
        })
    }

    fn operational_query() -> MailClientRequestV1 {
        MailClientRequestV1::OperationalQuery(MailOperationalQueryV1::ListFolders {
            connection_id: "mail-account".to_owned(),
            cursor: None,
            limit: 100,
        })
    }

    fn sync_health_query() -> MailClientRequestV1 {
        MailClientRequestV1::SyncHealthQuery(MailSyncHealthQueryV1::GetStatus {
            connection_id: "mail-account".to_owned(),
        })
    }

    fn message_flag_command() -> MailClientRequestV1 {
        MailClientRequestV1::MessageFlagCommand(MailMessageFlagCommandV1 {
            operation_id: "flag-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
            message_id: "provider-message".to_owned(),
            kind: MailMessageFlagKindV1::Read,
            target_value: true,
        })
    }

    fn message_flag_query() -> MailClientRequestV1 {
        MailClientRequestV1::MessageFlagStatus(MailMessageFlagStatusRequestV1 {
            operation_id: "flag-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
        })
    }

    fn message_location_command() -> MailClientRequestV1 {
        MailClientRequestV1::MessageLocationCommand(MailMessageLocationCommandV1 {
            operation_id: "location-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
            message_id: "stable-message".to_owned(),
            kind: MailMessageLocationKindV1::Move,
            target_folder_id: Some("archive-folder".to_owned()),
        })
    }

    fn message_location_query() -> MailClientRequestV1 {
        MailClientRequestV1::MessageLocationStatus(MailMessageLocationStatusRequestV1 {
            operation_id: "location-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
        })
    }

    fn message_permanent_delete_command() -> MailClientRequestV1 {
        MailClientRequestV1::MessagePermanentDeleteCommand(MailMessagePermanentDeleteCommandV1 {
            operation_id: "permanent-delete-operation".to_owned(),
            connection_id: "mail-account".to_owned(),
            message_id: "stable-message".to_owned(),
            expected_projection_revision: 3,
            confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
        })
    }

    fn message_permanent_delete_query() -> MailClientRequestV1 {
        MailClientRequestV1::MessagePermanentDeleteStatus(
            MailMessagePermanentDeleteStatusRequestV1 {
                operation_id: "permanent-delete-operation".to_owned(),
                connection_id: "mail-account".to_owned(),
            },
        )
    }

    #[test]
    fn account_binding_and_query_use_independent_exact_contracts() {
        let bind = MailClientRequestV1::BindCredential(MailBindCredentialRequestV1 {
            connection_id: "mail-account".to_owned(),
            purpose: MailCredentialPurposeV1::ImapPassword,
            expected_binding_revision: 0,
            credential_revision: 1,
        });
        let query = MailClientRequestV1::AccountStatus(MailAccountStatusRequestV1 {
            connection_id: "mail-account".to_owned(),
        });

        let encoded_bind = encode_module_request(4, &bind).expect("account bind");
        let (_, bind_contract, decoded_bind) =
            decode_module_request(&encoded_bind).expect("decode account bind");
        let encoded_query = encode_module_request(5, &query).expect("account query");
        let (_, query_contract, decoded_query) =
            decode_module_request(&encoded_query).expect("decode account query");

        assert_eq!(bind_contract, MailClientContractV1::AccountCredentialBind);
        assert_eq!(query_contract, MailClientContractV1::AccountQuery);
        assert_eq!(decoded_bind, bind);
        assert_eq!(decoded_query, query);
        assert_ne!(bind_contract, query_contract);
    }

    #[test]
    fn account_catalog_uses_its_exact_query_contract_and_response_codec() {
        let request = MailClientRequestV1::AccountCatalog(MailAccountCatalogRequestV1);
        let encoded = encode_module_request(6, &request).expect("account catalog");
        let (request_id, contract, decoded) =
            decode_module_request(&encoded).expect("decode account catalog");
        assert_eq!(request_id, 6);
        assert_eq!(contract, MailClientContractV1::AccountCatalog);
        assert_eq!(decoded, request);

        let response = MailClientResponseV1::AccountCatalog(MailAccountCatalogV1 {
            accounts: vec![MailAccountStatusV1 {
                connection_id: "mail-account".to_owned(),
                configuration_instance_id: "mail-configuration".to_owned(),
                settings_revision: 1,
                runtime_generation: 2,
                readiness: MailAccountReadinessV1::Ready,
                connector_profile: MailConnectorProfileV1::Gmail,
                sync_readiness: MailProviderPathReadinessV1::Ready,
                delivery_readiness: MailProviderPathReadinessV1::Ready,
                bindings: Vec::new(),
                lifecycle_revision: 0,
                lifecycle_operation_id: None,
            }],
        });
        let encoded_response =
            encode_module_response(6, contract, &response).expect("account catalog response");
        assert_eq!(
            decode_module_response(contract, &encoded_response),
            Ok((6, response))
        );
    }

    #[test]
    fn account_lifecycle_actions_retry_and_status_are_independent_contracts() {
        let command = MailAccountLifecycleCommandV1 {
            operation_id: "account-retire".to_owned(),
            connection_id: "mail-account".to_owned(),
            expected_lifecycle_revision: 0,
        };
        let requests = [
            (
                MailClientRequestV1::RetireAccount(command.clone()),
                MailClientContractV1::AccountRetire,
            ),
            (
                MailClientRequestV1::DeleteAccount(command),
                MailClientContractV1::AccountDelete,
            ),
            (
                MailClientRequestV1::RetryAccountLifecycle(MailAccountLifecycleRetryV1 {
                    operation_id: "account-retire".to_owned(),
                    connection_id: "mail-account".to_owned(),
                    expected_lifecycle_revision: 1,
                }),
                MailClientContractV1::AccountLifecycleRetry,
            ),
            (
                MailClientRequestV1::AccountLifecycleStatus(MailAccountLifecycleStatusRequestV1 {
                    operation_id: "account-retire".to_owned(),
                    connection_id: "mail-account".to_owned(),
                }),
                MailClientContractV1::AccountLifecycleQuery,
            ),
        ];
        for (index, (request, expected_contract)) in requests.into_iter().enumerate() {
            let encoded =
                encode_module_request(u64::try_from(index + 1).expect("request ID"), &request)
                    .expect("lifecycle request");
            let (_, contract, decoded) =
                decode_module_request(&encoded).expect("decode lifecycle request");
            assert_eq!(contract, expected_contract);
            assert_eq!(decoded, request);
        }
    }

    #[test]
    fn sync_request_uses_only_the_exact_sync_contract() {
        let encoded = encode_module_request(1, &sync_request()).expect("sync request");
        let (request_id, contract, decoded) =
            decode_module_request(&encoded).expect("decode sync request");

        assert_eq!(request_id, 1);
        assert_eq!(contract, MailClientContractV1::Sync);
        assert_eq!(decoded, sync_request());
    }

    #[test]
    fn operational_query_uses_its_exact_contract_and_response_codec() {
        let encoded = encode_module_request(7, &operational_query()).expect("operational query");
        let (request_id, contract, decoded) =
            decode_module_request(&encoded).expect("decode operational query");
        assert_eq!(request_id, 7);
        assert_eq!(contract, MailClientContractV1::OperationalQuery);
        assert_eq!(decoded, operational_query());

        let response = MailClientResponseV1::OperationalQuery(
            MailOperationalQueryResponseV1::Folders(MailOperationalPageV1 {
                items: Vec::new(),
                next_cursor: None,
            }),
        );
        let encoded_response =
            encode_module_response(7, contract, &response).expect("operational response");
        assert_eq!(
            decode_module_response(contract, &encoded_response),
            Ok((7, response))
        );
    }

    #[test]
    fn sync_health_query_uses_its_exact_contract_and_response_codec() {
        let encoded = encode_module_request(8, &sync_health_query()).expect("sync health query");
        let (request_id, contract, decoded) =
            decode_module_request(&encoded).expect("decode sync health query");
        assert_eq!(request_id, 8);
        assert_eq!(contract, MailClientContractV1::SyncHealthQuery);
        assert_eq!(decoded, sync_health_query());

        let response = MailClientResponseV1::SyncHealthQuery(
            MailSyncHealthQueryResponseV1::Status(MailSyncStatusV1 {
                connection_id: "mail-account".to_owned(),
                provider_path_readiness: MailSyncProviderPathReadinessV1::Ready,
                latest_run: None,
                consecutive_failures: 0,
                last_success_at_unix_seconds: None,
                projection_revision: 1,
            }),
        );
        let encoded_response =
            encode_module_response(8, contract, &response).expect("sync health response");
        assert_eq!(
            decode_module_response(contract, &encoded_response),
            Ok((8, response))
        );
    }

    #[test]
    fn message_flag_command_and_status_use_independent_exact_contracts() {
        let command = message_flag_command();
        let encoded = encode_module_request(9, &command).expect("message flag command");
        let (_, command_contract, decoded) =
            decode_module_request(&encoded).expect("decode message flag command");
        assert_eq!(command_contract, MailClientContractV1::MessageFlagCommand);
        assert_eq!(decoded, command);
        let response = MailClientResponseV1::MessageFlagAccepted(MailMessageFlagAcceptedV1 {
            operation_id: "flag-operation".to_owned(),
        });
        let bytes =
            encode_module_response(9, command_contract, &response).expect("accepted response");
        assert_eq!(
            decode_module_response(command_contract, &bytes),
            Ok((9, response))
        );

        let query = message_flag_query();
        let encoded = encode_module_request(10, &query).expect("message flag query");
        let (_, query_contract, decoded) =
            decode_module_request(&encoded).expect("decode message flag query");
        assert_eq!(query_contract, MailClientContractV1::MessageFlagQuery);
        assert_eq!(decoded, query);
        let response =
            MailClientResponseV1::MessageFlagStatus(Some(MailMessageFlagOperationStatusV1 {
                operation_id: "flag-operation".to_owned(),
                connection_id: "mail-account".to_owned(),
                message_id: "provider-message".to_owned(),
                kind: MailMessageFlagKindV1::Read,
                target_value: true,
                outcome: MailMessageFlagOperationOutcomeV1::Succeeded,
                requested_at_unix_seconds: 100,
                completed_at_unix_seconds: Some(101),
                projection_revision: Some(2),
            }));
        let bytes = encode_module_response(10, query_contract, &response)
            .expect("message flag status response");
        assert_eq!(
            decode_module_response(query_contract, &bytes),
            Ok((10, response))
        );
        assert_ne!(command_contract, query_contract);
    }

    #[test]
    fn message_location_command_and_status_use_independent_exact_contracts() {
        let command = message_location_command();
        let encoded = encode_module_request(11, &command).expect("message location command");
        let (_, command_contract, decoded) =
            decode_module_request(&encoded).expect("decode message location command");
        assert_eq!(
            command_contract,
            MailClientContractV1::MessageLocationCommand
        );
        assert_eq!(decoded, command);
        let response =
            MailClientResponseV1::MessageLocationAccepted(MailMessageLocationAcceptedV1 {
                operation_id: "location-operation".to_owned(),
            });
        let bytes = encode_module_response(11, command_contract, &response)
            .expect("message location accepted response");
        assert_eq!(
            decode_module_response(command_contract, &bytes),
            Ok((11, response))
        );

        let query = message_location_query();
        let encoded = encode_module_request(12, &query).expect("message location query");
        let (_, query_contract, decoded) =
            decode_module_request(&encoded).expect("decode message location query");
        assert_eq!(query_contract, MailClientContractV1::MessageLocationQuery);
        assert_eq!(decoded, query);
        let response = MailClientResponseV1::MessageLocationStatus(Some(
            MailMessageLocationOperationStatusV1 {
                operation_id: "location-operation".to_owned(),
                connection_id: "mail-account".to_owned(),
                message_id: "stable-message".to_owned(),
                kind: MailMessageLocationKindV1::Move,
                target_folder_id: Some("archive-folder".to_owned()),
                outcome: MailMessageLocationOperationOutcomeV1::Succeeded,
                requested_at_unix_seconds: 100,
                completed_at_unix_seconds: Some(101),
                projection_revision: Some(3),
            },
        ));
        let bytes = encode_module_response(12, query_contract, &response)
            .expect("message location status response");
        assert_eq!(
            decode_module_response(query_contract, &bytes),
            Ok((12, response))
        );
        assert_ne!(command_contract, query_contract);
    }

    #[test]
    fn message_permanent_delete_command_and_status_use_independent_exact_contracts() {
        let command = message_permanent_delete_command();
        let encoded = encode_module_request(13, &command).expect("permanent delete command");
        let (_, command_contract, decoded) =
            decode_module_request(&encoded).expect("decode permanent delete command");
        assert_eq!(
            command_contract,
            MailClientContractV1::MessagePermanentDeleteCommand
        );
        assert_eq!(decoded, command);
        let response = MailClientResponseV1::MessagePermanentDeleteAccepted(
            MailMessagePermanentDeleteAcceptedV1 {
                operation_id: "permanent-delete-operation".to_owned(),
            },
        );
        let bytes = encode_module_response(13, command_contract, &response)
            .expect("permanent delete accepted response");
        assert_eq!(
            decode_module_response(command_contract, &bytes),
            Ok((13, response))
        );

        let query = message_permanent_delete_query();
        let encoded = encode_module_request(14, &query).expect("permanent delete query");
        let (_, query_contract, decoded) =
            decode_module_request(&encoded).expect("decode permanent delete query");
        assert_eq!(
            query_contract,
            MailClientContractV1::MessagePermanentDeleteQuery
        );
        assert_eq!(decoded, query);
        let response = MailClientResponseV1::MessagePermanentDeleteStatus(Some(
            MailMessagePermanentDeleteOperationStatusV1 {
                operation_id: "permanent-delete-operation".to_owned(),
                connection_id: "mail-account".to_owned(),
                message_id: "stable-message".to_owned(),
                expected_projection_revision: 3,
                confirmation: MailMessagePermanentDeleteConfirmationV1::Confirmed,
                outcome: MailMessagePermanentDeleteOperationOutcomeV1::Succeeded,
                requested_at_unix_seconds: 100,
                completed_at_unix_seconds: Some(101),
                deletion_projection_revision: Some(4),
            },
        ));
        let bytes = encode_module_response(14, query_contract, &response)
            .expect("permanent delete status response");
        assert_eq!(
            decode_module_response(query_contract, &bytes),
            Ok((14, response))
        );
        assert_ne!(command_contract, query_contract);
    }

    #[test]
    fn delivery_payload_is_rejected_under_sync_contract() {
        let encoded = encode_module_request(1, &delivery_request()).expect("delivery request");
        let mut envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("envelope");
        envelope.contract = Some(mail_client_contract(MailClientContractV1::Sync));

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(MailClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn delivery_command_and_query_use_independent_contracts() {
        let command = encode_module_request(2, &delivery_request()).expect("delivery request");
        let (_, command_contract, _) =
            decode_module_request(&command).expect("decode delivery request");
        let query = encode_module_request(3, &delivery_query()).expect("delivery query");
        let (_, query_contract, _) = decode_module_request(&query).expect("decode delivery query");

        assert_eq!(command_contract, MailClientContractV1::Delivery);
        assert_eq!(query_contract, MailClientContractV1::DeliveryQuery);
        assert_ne!(command_contract, query_contract);
    }

    #[test]
    fn umbrella_client_contract_is_not_admitted() {
        let encoded = encode_module_request(1, &sync_request()).expect("sync request");
        let mut envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("envelope");
        envelope.contract.as_mut().expect("contract").name = "mail.client".to_owned();

        assert_eq!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(MailClientPortErrorV1::Protocol)
        );
    }

    #[test]
    fn stable_empty_error_response_is_runtime_rejection_not_protocol_corruption() {
        let rejection = ModuleClientResponseV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            request_id: 9,
            response_payload: Vec::new(),
            error_code: "REJECTED".to_owned(),
        }
        .encode_to_vec();
        assert_eq!(
            decode_module_response(MailClientContractV1::Delivery, &rejection),
            Err(MailClientPortErrorV1::Runtime)
        );

        let mut invalid =
            ModuleClientResponseV1::decode(rejection.as_slice()).expect("decode rejection");
        invalid.response_payload = vec![1];
        assert_eq!(
            decode_module_response(MailClientContractV1::Delivery, &invalid.encode_to_vec()),
            Err(MailClientPortErrorV1::Protocol)
        );
    }
}

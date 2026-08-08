//! Telegram-owned client subset available before provider authorization.

use std::os::unix::net::UnixStream;

use makosh_runtime_protocol::managed_control::{
    ManagedControlChannelV2, ManagedControlRequestDispatcherV2,
};
use makosh_runtime_protocol::v1::ModuleClientRequestV1;
use makosh_telegram_api::{
    TelegramAccount, TelegramAccountState, TelegramClientRequest, TelegramClientResponse,
    TelegramRuntimeState, client_contract::TelegramClientContractV1, validate_setup,
};
use makosh_telegram_persistence::TelegramDurablePersistence;
use prost::Message;

use crate::{
    TelegramRuntimeComposition,
    bootstrap::{
        TelegramProviderReconfigurationContextV1, credential_revisions,
        resolve_provider_setup_parameters,
    },
    client_port::{decode_module_request, encode_module_response},
    client_transport::TelegramClientTransportError,
};

pub(crate) struct TelegramConfigurationClientContextV1<'a, D> {
    pub(crate) runtime_available: bool,
    pub(crate) composition: &'a mut TelegramRuntimeComposition,
    pub(crate) authorization_status: Option<&'a makosh_telegram_api::TelegramAuthorizationStatus>,
    pub(crate) durable: &'a TelegramDurablePersistence,
    pub(crate) control_channel: &'a mut ManagedControlChannelV2<UnixStream>,
    pub(crate) dispatcher: &'a mut D,
    pub(crate) reconfiguration_context: &'a mut TelegramProviderReconfigurationContextV1,
}

pub(crate) async fn try_handle<D>(
    request: &[u8],
    context: TelegramConfigurationClientContextV1<'_, D>,
) -> Result<Option<Vec<u8>>, TelegramClientTransportError>
where
    D: ManagedControlRequestDispatcherV2<UnixStream>,
{
    if !owns_telegram_client_contract(request)? {
        return Ok(None);
    }
    let TelegramConfigurationClientContextV1 {
        runtime_available,
        composition,
        authorization_status,
        durable,
        control_channel,
        dispatcher,
        reconfiguration_context,
    } = context;
    let (request_id, contract, request) =
        decode_module_request(request).map_err(TelegramClientTransportError::Port)?;
    let response = match request {
        TelegramClientRequest::AuthorizationStatus => {
            TelegramClientResponse::AuthorizationStatus(authorization_status.cloned().unwrap_or(
                makosh_telegram_api::TelegramAuthorizationStatus {
                    state: "starting".to_owned(),
                    qr_link: None,
                    password_hint: None,
                },
            ))
        }
        TelegramClientRequest::SubmitAuthorizationPassword { password } => {
            composition.submit_password(&password).map_err(|error| {
                TelegramClientTransportError::Port(
                    crate::client_port::TelegramClientPortError::Provider(error),
                )
            })?;
            TelegramClientResponse::AuthorizationPasswordAccepted
        }
        TelegramClientRequest::ProvisionAccount { setup } if !runtime_available => {
            if setup.account_id != composition.configured_account_id()
                || validate_setup(&setup).is_err()
                || reconfiguration_context
                    .configuration_instance_id()
                    .trim()
                    .is_empty()
            {
                return Err(TelegramClientTransportError::Port(
                    crate::client_port::TelegramClientPortError::Protocol(
                        "Telegram configuration account is invalid".to_owned(),
                    ),
                ));
            }
            let (api_hash_revision, session_encryption_key_revision) =
                credential_revisions(&setup.credentials).map_err(|_| {
                    TelegramClientTransportError::Port(
                        crate::client_port::TelegramClientPortError::Protocol(
                            "Telegram credential binding is invalid".to_owned(),
                        ),
                    )
                })?;
            let parameters = resolve_provider_setup_parameters(
                control_channel,
                dispatcher,
                reconfiguration_context,
                api_hash_revision,
                session_encryption_key_revision,
            )
            .map_err(|_| TelegramClientTransportError::RuntimeUnavailable)?;
            let account = TelegramAccount {
                account_id: setup.account_id.clone(),
                display_name: setup.display_name.clone(),
                external_account_id: setup.external_account_id.clone(),
                state: TelegramAccountState::Provisioning,
                runtime_state: TelegramRuntimeState::Stopped,
                runtime_epoch: 0,
            };
            durable
                .upsert_account(&account, &setup.credentials)
                .await
                .map_err(|error| {
                    TelegramClientTransportError::Port(
                        crate::client_port::TelegramClientPortError::Persistence(error),
                    )
                })?;
            composition
                .begin_account_authorization(setup, parameters)
                .map_err(|error| {
                    TelegramClientTransportError::Port(
                        crate::client_port::TelegramClientPortError::Provider(error),
                    )
                })?;
            reconfiguration_context
                .bind_credential_revisions(api_hash_revision, session_encryption_key_revision);
            TelegramClientResponse::Account(account)
        }
        TelegramClientRequest::ListAccounts if !runtime_available => {
            TelegramClientResponse::Accounts(durable.accounts().await.map_err(|error| {
                TelegramClientTransportError::Port(
                    crate::client_port::TelegramClientPortError::Persistence(error),
                )
            })?)
        }
        TelegramClientRequest::GetAccount { account_id } if !runtime_available => {
            TelegramClientResponse::Account(
                durable
                    .account(&account_id)
                    .await
                    .map_err(|error| {
                        TelegramClientTransportError::Port(
                            crate::client_port::TelegramClientPortError::Persistence(error),
                        )
                    })?
                    .map(|(account, _)| account)
                    .ok_or_else(|| {
                        TelegramClientTransportError::Port(
                            crate::client_port::TelegramClientPortError::Protocol(
                                "Telegram account is unknown".to_owned(),
                            ),
                        )
                    })?,
            )
        }
        _ => return Ok(None),
    };
    encode_module_response(contract, request_id, &response)
        .map(Some)
        .map_err(TelegramClientTransportError::Port)
}

fn owns_telegram_client_contract(request: &[u8]) -> Result<bool, TelegramClientTransportError> {
    let envelope = ModuleClientRequestV1::decode(request).map_err(|error| {
        TelegramClientTransportError::Port(crate::client_port::TelegramClientPortError::Codec(
            error.to_string(),
        ))
    })?;
    let contract = envelope.contract.as_ref().ok_or_else(|| {
        TelegramClientTransportError::Port(crate::client_port::TelegramClientPortError::Protocol(
            "Telegram client contract is missing".to_owned(),
        ))
    })?;
    Ok(TelegramClientContractV1::from_contract_name(&contract.name).is_some())
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::ContractReferenceV1;

    use super::*;

    #[test]
    fn configuration_port_declines_another_telegram_build_unit_contract() {
        let request = ModuleClientRequestV1 {
            protocol_major: 1,
            module_id: "makosh-telegram-runtime".to_owned(),
            owner_id: "telegram".to_owned(),
            contract: Some(ContractReferenceV1 {
                owner: "telegram".to_owned(),
                name: "telegram.calls.query.v1".to_owned(),
                major: 1,
                revision: 1,
                schema_sha256: vec![7; 32],
            }),
            request_id: 1,
            request_payload: vec![1],
            logical_owner_id: String::new(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
        }
        .encode_to_vec();

        assert!(matches!(owns_telegram_client_contract(&request), Ok(false)));
    }
}

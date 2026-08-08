//! Typed client port for Telegram operational commands, queries and replay.

use makosh_runtime_protocol::v1::{
    ContractReferenceV1, ModuleClientRequestV1, ModuleClientResponseV1,
};
use makosh_telegram_api::{
    MAX_PAGE_SIZE, TelegramClientRequest, TelegramClientResponse,
    client_contract::{
        TELEGRAM_CLIENT_CONTRACT_MAJOR, TELEGRAM_CLIENT_CONTRACT_REVISION,
        TELEGRAM_CLIENT_DESCRIPTOR_SET_V1, TELEGRAM_MODULE_ID, TELEGRAM_OWNER_ID,
        TelegramClientContractV1,
    },
};
use makosh_telegram_core::project_message;
use makosh_telegram_persistence::{TelegramDurablePersistence, TelegramDurablePersistenceError};
use makosh_telegram_tdlib::{TdlibError, TdlibTransport};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::{TelegramDurableLifecycleError, TelegramRuntime};

#[derive(Debug)]
pub enum TelegramClientPortError {
    Provider(TdlibError),
    Persistence(TelegramDurablePersistenceError),
    Protocol(String),
    Codec(String),
    Reconfiguration(&'static str),
}

fn reconfiguration_error_code(error: TelegramDurableLifecycleError) -> &'static str {
    match error {
        TelegramDurableLifecycleError::Contract(
            makosh_telegram_api::TelegramContractError::RuntimeEpochConflict,
        ) => "RUNTIME_EPOCH_CONFLICT",
        TelegramDurableLifecycleError::Contract(
            makosh_telegram_api::TelegramContractError::ReconfigurationCollision,
        ) => "RECONFIGURATION_COLLISION",
        TelegramDurableLifecycleError::Contract(
            makosh_telegram_api::TelegramContractError::ReconfigurationInProgress,
        ) => "RECONFIGURATION_IN_PROGRESS",
        TelegramDurableLifecycleError::Contract(
            makosh_telegram_api::TelegramContractError::ReconfigurationUnknown,
        ) => "RECONFIGURATION_UNKNOWN",
        TelegramDurableLifecycleError::Contract(_) => "RECONFIGURATION_REJECTED",
        TelegramDurableLifecycleError::Persistence(_) => "RECONFIGURATION_UNAVAILABLE",
    }
}

pub(crate) const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

fn telegram_client_contract(contract: TelegramClientContractV1) -> ContractReferenceV1 {
    ContractReferenceV1 {
        owner: TELEGRAM_OWNER_ID.to_owned(),
        name: contract.contract_name().to_owned(),
        major: TELEGRAM_CLIENT_CONTRACT_MAJOR,
        revision: TELEGRAM_CLIENT_CONTRACT_REVISION,
        schema_sha256: Sha256::digest(TELEGRAM_CLIENT_DESCRIPTOR_SET_V1).to_vec(),
    }
}

fn validate_contract(
    contract: &ContractReferenceV1,
) -> Result<TelegramClientContractV1, TelegramClientPortError> {
    let route = TelegramClientContractV1::from_contract_name(&contract.name).ok_or_else(|| {
        TelegramClientPortError::Protocol(
            "Telegram client contract reference is not admitted".to_owned(),
        )
    })?;
    let expected = telegram_client_contract(route);
    if contract != &expected {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client contract reference is not admitted".to_owned(),
        ));
    }
    Ok(route)
}

fn request_contract(
    request: &TelegramClientRequest,
) -> Result<TelegramClientContractV1, TelegramClientPortError> {
    match request {
        TelegramClientRequest::AuthorizationStatus
        | TelegramClientRequest::SubmitAuthorizationPassword { .. } => {
            Ok(TelegramClientContractV1::Authorization)
        }
        TelegramClientRequest::ProvisionAccount { .. }
        | TelegramClientRequest::RetryCommand { .. }
        | TelegramClientRequest::ListAccounts
        | TelegramClientRequest::GetAccount { .. }
        | TelegramClientRequest::RetireAccount { .. } => Ok(TelegramClientContractV1::Lifecycle),
        TelegramClientRequest::Command(_) => Ok(TelegramClientContractV1::Command),
        TelegramClientRequest::Query(_) => Ok(TelegramClientContractV1::Query),
        TelegramClientRequest::Replay { .. } => Ok(TelegramClientContractV1::Realtime),
        TelegramClientRequest::Reconfiguration(_) => Ok(TelegramClientContractV1::Reconfiguration),
    }
}

fn lifecycle_wire_request(
    request: &TelegramClientRequest,
) -> Option<makosh_telegram_api::client_wire::TelegramLifecycleRequest> {
    use makosh_telegram_api::client_wire::TelegramLifecycleRequest;
    match request {
        TelegramClientRequest::ProvisionAccount { setup } => {
            Some(TelegramLifecycleRequest::Provision(setup.clone()))
        }
        TelegramClientRequest::RetryCommand {
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
        } => Some(TelegramLifecycleRequest::Retry {
            operation_id: operation_id.clone(),
            now_unix_seconds: *now_unix_seconds,
            next_attempt_at_unix_seconds: *next_attempt_at_unix_seconds,
        }),
        TelegramClientRequest::ListAccounts => Some(TelegramLifecycleRequest::ListAccounts),
        TelegramClientRequest::GetAccount { account_id } => {
            Some(TelegramLifecycleRequest::GetAccount {
                account_id: account_id.clone(),
            })
        }
        TelegramClientRequest::RetireAccount { account_id } => {
            Some(TelegramLifecycleRequest::RetireAccount {
                account_id: account_id.clone(),
            })
        }
        _ => None,
    }
}

fn client_request_from_lifecycle(
    request: makosh_telegram_api::client_wire::TelegramLifecycleRequest,
) -> TelegramClientRequest {
    use makosh_telegram_api::client_wire::TelegramLifecycleRequest;
    match request {
        TelegramLifecycleRequest::Provision(setup) => {
            TelegramClientRequest::ProvisionAccount { setup }
        }
        TelegramLifecycleRequest::Retry {
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
        } => TelegramClientRequest::RetryCommand {
            operation_id,
            now_unix_seconds,
            next_attempt_at_unix_seconds,
        },
        TelegramLifecycleRequest::ListAccounts => TelegramClientRequest::ListAccounts,
        TelegramLifecycleRequest::GetAccount { account_id } => {
            TelegramClientRequest::GetAccount { account_id }
        }
        TelegramLifecycleRequest::RetireAccount { account_id } => {
            TelegramClientRequest::RetireAccount { account_id }
        }
        TelegramLifecycleRequest::Replay {
            account_id,
            after_sequence,
            limit,
        } => TelegramClientRequest::Replay {
            account_id,
            after_sequence,
            limit,
        },
    }
}

pub fn encode_module_request(
    request_id: u64,
    request: &TelegramClientRequest,
) -> Result<Vec<u8>, TelegramClientPortError> {
    if request_id == 0 {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request id must be non-zero".to_owned(),
        ));
    }
    let contract = request_contract(request)?;
    let request_payload = match contract {
        TelegramClientContractV1::Authorization => {
            let request = match request {
                TelegramClientRequest::AuthorizationStatus => {
                    makosh_telegram_api::client_wire::TelegramAuthorizationRequest::Status
                }
                TelegramClientRequest::SubmitAuthorizationPassword { password } => {
                    makosh_telegram_api::client_wire::TelegramAuthorizationRequest::SubmitPassword(
                        password.clone(),
                    )
                }
                _ => {
                    return Err(TelegramClientPortError::Protocol(
                        "Telegram authorization route received another request family".to_owned(),
                    ));
                }
            };
            makosh_telegram_api::client_wire::encode_request(&request)
        }
        TelegramClientContractV1::Lifecycle => {
            let lifecycle = lifecycle_wire_request(request).ok_or_else(|| {
                TelegramClientPortError::Protocol(
                    "Telegram lifecycle request has no generated wire mapping".to_owned(),
                )
            })?;
            makosh_telegram_api::client_wire::encode_lifecycle_request(&lifecycle)
        }
        TelegramClientContractV1::Command => match request {
            TelegramClientRequest::Command(command) => {
                makosh_telegram_api::client_wire::encode_command(command)
            }
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram command route received another request family".to_owned(),
                ));
            }
        },
        TelegramClientContractV1::Query => match request {
            TelegramClientRequest::Query(query) => {
                makosh_telegram_api::client_wire::encode_query(query)
            }
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram query route received another request family".to_owned(),
                ));
            }
        },
        TelegramClientContractV1::Realtime => match request {
            TelegramClientRequest::Replay {
                account_id,
                after_sequence,
                limit,
            } => makosh_telegram_api::client_wire::encode_realtime_request(
                account_id,
                *after_sequence,
                *limit,
            ),
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram realtime route received another request family".to_owned(),
                ));
            }
        },
        TelegramClientContractV1::Reconfiguration => match request {
            TelegramClientRequest::Reconfiguration(request) => {
                makosh_telegram_api::client_wire::encode_reconfiguration_request(request)
            }
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram reconfiguration route received another request family".to_owned(),
                ));
            }
        },
    };
    let envelope = ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: TELEGRAM_MODULE_ID.to_owned(),
        owner_id: TELEGRAM_OWNER_ID.to_owned(),
        contract: Some(telegram_client_contract(contract)),
        request_id,
        request_payload,
        logical_owner_id: String::new(),
        authenticated_device_id: String::new(),
        authenticated_client_session_id: String::new(),
    };
    Ok(envelope.encode_to_vec())
}

pub fn decode_module_request(
    bytes: &[u8],
) -> Result<(u64, TelegramClientContractV1, TelegramClientRequest), TelegramClientPortError> {
    let (request_id, contract, request_payload) = decode_module_request_payload(bytes)?;
    let request = match contract {
        TelegramClientContractV1::Authorization => {
            match makosh_telegram_api::client_wire::decode_request(&request_payload).map_err(
                |_| {
                    TelegramClientPortError::Protocol(
                        "Telegram authorization request payload is invalid".to_owned(),
                    )
                },
            )? {
                makosh_telegram_api::client_wire::TelegramAuthorizationRequest::Status => {
                    TelegramClientRequest::AuthorizationStatus
                }
                makosh_telegram_api::client_wire::TelegramAuthorizationRequest::SubmitPassword(
                    password,
                ) => TelegramClientRequest::SubmitAuthorizationPassword { password },
            }
        }
        TelegramClientContractV1::Lifecycle => {
            let request =
                makosh_telegram_api::client_wire::decode_lifecycle_request(&request_payload)
                    .map(client_request_from_lifecycle)
                    .map_err(|_| {
                        TelegramClientPortError::Protocol(
                            "Telegram lifecycle request payload is invalid".to_owned(),
                        )
                    })?;
            if matches!(request, TelegramClientRequest::Replay { .. }) {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram replay requires the separately admitted realtime contract".to_owned(),
                ));
            }
            request
        }
        TelegramClientContractV1::Command => {
            makosh_telegram_api::client_wire::decode_command(&request_payload)
                .map(TelegramClientRequest::Command)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram command request payload is invalid".to_owned(),
                    )
                })?
        }
        TelegramClientContractV1::Query => {
            makosh_telegram_api::client_wire::decode_query(&request_payload)
                .map(TelegramClientRequest::Query)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram query request payload is invalid".to_owned(),
                    )
                })?
        }
        TelegramClientContractV1::Realtime => {
            let (account_id, after_sequence, limit) =
                makosh_telegram_api::client_wire::decode_realtime_request(&request_payload)
                    .map_err(|_| {
                        TelegramClientPortError::Protocol(
                            "Telegram realtime request payload is invalid".to_owned(),
                        )
                    })?;
            TelegramClientRequest::Replay {
                account_id,
                after_sequence,
                limit,
            }
        }
        TelegramClientContractV1::Reconfiguration => {
            makosh_telegram_api::client_wire::decode_reconfiguration_request(&request_payload)
                .map(TelegramClientRequest::Reconfiguration)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram reconfiguration request payload is invalid".to_owned(),
                    )
                })?
        }
    };
    Ok((request_id, contract, request))
}

pub fn decode_module_request_payload(
    bytes: &[u8],
) -> Result<(u64, TelegramClientContractV1, Vec<u8>), TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != TELEGRAM_MODULE_ID
        || envelope.owner_id != TELEGRAM_OWNER_ID
        || envelope.request_id == 0
    {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request routing metadata is not admitted".to_owned(),
        ));
    }
    let contract = validate_contract(envelope.contract.as_ref().ok_or_else(|| {
        TelegramClientPortError::Protocol("Telegram client contract is missing".to_owned())
    })?)?;
    if envelope.request_payload.is_empty() {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client request payload is empty".to_owned(),
        ));
    }
    Ok((envelope.request_id, contract, envelope.request_payload))
}

pub fn encode_module_response_payload(
    request_id: u64,
    response_payload: Vec<u8>,
) -> Result<Vec<u8>, TelegramClientPortError> {
    if request_id == 0 || response_payload.is_empty() {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response payload is invalid".to_owned(),
        ));
    }
    Ok(ModuleClientResponseV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        request_id,
        response_payload,
        error_code: String::new(),
    }
    .encode_to_vec())
}

pub fn encode_module_response(
    contract: TelegramClientContractV1,
    request_id: u64,
    response: &TelegramClientResponse,
) -> Result<Vec<u8>, TelegramClientPortError> {
    if request_id == 0 {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response id must be non-zero".to_owned(),
        ));
    }
    let response_payload = match contract {
        TelegramClientContractV1::Authorization => {
            let response = match response {
                TelegramClientResponse::AuthorizationStatus(status) => {
                    makosh_telegram_api::client_wire::TelegramAuthorizationResponse::Status(
                        status.clone(),
                    )
                }
                TelegramClientResponse::AuthorizationPasswordAccepted => {
                    makosh_telegram_api::client_wire::TelegramAuthorizationResponse::PasswordAccepted
                }
                _ => {
                    return Err(TelegramClientPortError::Protocol(
                        "Telegram authorization response has another response family".to_owned(),
                    ));
                }
            };
            makosh_telegram_api::client_wire::encode_response(&response)
        }
        TelegramClientContractV1::Lifecycle => {
            makosh_telegram_api::client_wire::encode_lifecycle_response(response).ok_or_else(
                || {
                    TelegramClientPortError::Protocol(
                        "Telegram lifecycle response has another response family".to_owned(),
                    )
                },
            )?
        }
        TelegramClientContractV1::Command => match response {
            TelegramClientResponse::Operation(operation) => {
                makosh_telegram_api::client_wire::encode_command_response(operation)
            }
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram command response has another response family".to_owned(),
                ));
            }
        },
        TelegramClientContractV1::Query => match response {
            TelegramClientResponse::Query(query_response) => {
                makosh_telegram_api::client_wire::encode_query_response(query_response).ok_or_else(
                    || {
                        TelegramClientPortError::Protocol(
                            "Telegram query response has no generated wire mapping".to_owned(),
                        )
                    },
                )?
            }
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram query response has another response family".to_owned(),
                ));
            }
        },
        TelegramClientContractV1::Realtime => match response {
            TelegramClientResponse::Realtime(page) => {
                makosh_telegram_api::client_wire::encode_realtime_response(page)
            }
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram realtime response has another response family".to_owned(),
                ));
            }
        },
        TelegramClientContractV1::Reconfiguration => match response {
            TelegramClientResponse::Reconfiguration(reconfiguration) => {
                makosh_telegram_api::client_wire::encode_reconfiguration_response(reconfiguration)
            }
            _ => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram reconfiguration response has another response family".to_owned(),
                ));
            }
        },
    };
    encode_module_response_payload(request_id, response_payload)
}

pub fn decode_module_response(
    contract: TelegramClientContractV1,
    bytes: &[u8],
) -> Result<(u64, TelegramClientResponse), TelegramClientPortError> {
    let envelope = ModuleClientResponseV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR || envelope.request_id == 0 {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response routing metadata is invalid".to_owned(),
        ));
    }
    if !envelope.error_code.is_empty() {
        return Err(TelegramClientPortError::Protocol(envelope.error_code));
    }
    if envelope.response_payload.is_empty() {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client response payload is empty".to_owned(),
        ));
    }
    let response = match contract {
        TelegramClientContractV1::Authorization => {
            match makosh_telegram_api::client_wire::decode_response(&envelope.response_payload)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram authorization response payload is invalid".to_owned(),
                    )
                })? {
                makosh_telegram_api::client_wire::TelegramAuthorizationResponse::Status(
                    status,
                ) => TelegramClientResponse::AuthorizationStatus(status),
                makosh_telegram_api::client_wire::TelegramAuthorizationResponse::PasswordAccepted => {
                    TelegramClientResponse::AuthorizationPasswordAccepted
                }
            }
        }
        TelegramClientContractV1::Lifecycle => {
            makosh_telegram_api::client_wire::decode_lifecycle_response(&envelope.response_payload)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram lifecycle response payload is invalid".to_owned(),
                    )
                })?
        }
        TelegramClientContractV1::Command => {
            makosh_telegram_api::client_wire::decode_command_response(&envelope.response_payload)
                .map(TelegramClientResponse::Operation)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram command response payload is invalid".to_owned(),
                    )
                })?
        }
        TelegramClientContractV1::Query => {
            makosh_telegram_api::client_wire::decode_query_response(&envelope.response_payload)
                .map(TelegramClientResponse::Query)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram query response payload is invalid".to_owned(),
                    )
                })?
        }
        TelegramClientContractV1::Realtime => {
            makosh_telegram_api::client_wire::decode_realtime_response(&envelope.response_payload)
                .map(TelegramClientResponse::Realtime)
                .map_err(|_| {
                    TelegramClientPortError::Protocol(
                        "Telegram realtime response payload is invalid".to_owned(),
                    )
                })?
        }
        TelegramClientContractV1::Reconfiguration => {
            makosh_telegram_api::client_wire::decode_reconfiguration_response(
                &envelope.response_payload,
            )
            .map(TelegramClientResponse::Reconfiguration)
            .map_err(|_| {
                TelegramClientPortError::Protocol(
                    "Telegram reconfiguration response payload is invalid".to_owned(),
                )
            })?
        }
    };
    Ok((envelope.request_id, response))
}

pub struct TelegramClientPort<'a, T> {
    runtime: &'a mut TelegramRuntime<T>,
}

impl<'a, T: TdlibTransport> TelegramClientPort<'a, T> {
    pub fn new(runtime: &'a mut TelegramRuntime<T>) -> Self {
        Self { runtime }
    }

    pub async fn handle_module_request_durable(
        &mut self,
        bytes: &[u8],
        durable: &TelegramDurablePersistence,
    ) -> Result<Vec<u8>, TelegramClientPortError> {
        let (request_id, contract, request) = decode_module_request(bytes)?;
        let response = match request {
            TelegramClientRequest::ProvisionAccount { setup } => self
                .runtime
                .provision_account_durable(durable, setup)
                .await
                .map(TelegramClientResponse::Account)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::ListAccounts => TelegramClientResponse::Accounts(
                durable
                    .accounts()
                    .await
                    .map_err(TelegramClientPortError::Persistence)?,
            ),
            TelegramClientRequest::GetAccount { account_id } => TelegramClientResponse::Account(
                durable
                    .account(&account_id)
                    .await
                    .map_err(TelegramClientPortError::Persistence)?
                    .map(|(account, _credentials)| account)
                    .ok_or_else(|| {
                        TelegramClientPortError::Protocol("Telegram account is unknown".to_owned())
                    })?,
            ),
            TelegramClientRequest::RetireAccount { account_id } => self
                .runtime
                .retire_account_durable(durable, &account_id)
                .await
                .map(TelegramClientResponse::Account)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::Command(command) => self
                .runtime
                .execute_provider_command_durable(durable, command)
                .await
                .map(TelegramClientResponse::Operation)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::RetryCommand {
                operation_id,
                now_unix_seconds,
                next_attempt_at_unix_seconds,
            } => self
                .runtime
                .retry_operation_durable(
                    durable,
                    &operation_id,
                    now_unix_seconds,
                    next_attempt_at_unix_seconds,
                )
                .await
                .map(TelegramClientResponse::Operation)
                .map_err(|error| TelegramClientPortError::Protocol(format!("{error:?}")))?,
            TelegramClientRequest::Reconfiguration(request) => self
                .runtime
                .runtime_reconfiguration_durable(durable, &request)
                .await
                .map(TelegramClientResponse::Reconfiguration)
                .map_err(|error| {
                    TelegramClientPortError::Reconfiguration(reconfiguration_error_code(error))
                })?,
            TelegramClientRequest::Replay {
                account_id,
                after_sequence,
                limit,
            } => {
                self.replay_durable(durable, &account_id, after_sequence, limit)
                    .await?
            }
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::LoadHistory {
                    account_id,
                    provider_chat_id,
                    from_message_id,
                    mode,
                    limit,
                },
            ) => {
                let page = self
                    .runtime
                    .load_history_with_options(
                        &account_id,
                        &provider_chat_id,
                        from_message_id,
                        mode,
                        limit,
                    )
                    .map_err(TelegramClientPortError::Provider)?;
                for observation in &page.items {
                    let projection = project_message(observation).map_err(|_| {
                        TelegramClientPortError::Protocol(
                            "Telegram history projection is invalid".to_owned(),
                        )
                    })?;
                    durable
                        .upsert_message(&projection)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?;
                }
                TelegramClientResponse::Query(
                    makosh_telegram_api::TelegramProviderQueryResponse::HistoryPage(page),
                )
            }
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::Operations { account_id, limit },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::Operations(
                    durable
                        .operations_for_account(&account_id, i64::from(limit))
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::ChatAvatar {
                    account_id,
                    provider_chat_id,
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::ChatAvatar(
                    durable
                        .chat_avatar(&account_id, &provider_chat_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::Attachment { attachment_id, .. },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::Attachment(
                    durable
                        .attachment(&attachment_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::MessageReferences {
                    message_id, ..
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::MessageReferences(
                    durable
                        .message_references(&message_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::ReplyChain {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::ReplyChain(
                    durable
                        .reply_chain(
                            &account_id,
                            &provider_chat_id,
                            &provider_message_id,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::ForwardChain {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::ForwardChain(
                    durable
                        .forward_chain(
                            &account_id,
                            &provider_chat_id,
                            &provider_message_id,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::AttachmentForMessage {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::Attachment(
                    durable
                        .attachment_for_message(
                            &account_id,
                            &provider_chat_id,
                            &provider_message_id,
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(makosh_telegram_api::TelegramProviderQuery::File {
                account_id,
                provider_file_id,
            }) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::File(
                    durable
                        .file(&account_id, &provider_file_id)
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::Commands {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                    command_kinds,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::Commands(
                    durable
                        .command_records_for_account(
                            &account_id,
                            provider_chat_id.as_deref(),
                            provider_message_id.as_deref(),
                            &command_kinds,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::ReactionSummary {
                    account_id,
                    provider_chat_id,
                    provider_message_id,
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::ReactionSummary(
                    durable
                        .reaction_summary(&format!(
                            "telegram:{account_id}:{provider_chat_id}:{provider_message_id}"
                        ))
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(
                makosh_telegram_api::TelegramProviderQuery::TopicMessageIds {
                    account_id,
                    provider_chat_id,
                    provider_topic_id,
                    limit,
                },
            ) => TelegramClientResponse::Query(
                makosh_telegram_api::TelegramProviderQueryResponse::TopicMessageIds(
                    durable
                        .message_ids_for_topic(
                            &account_id,
                            &provider_chat_id,
                            &provider_topic_id,
                            i64::from(limit),
                        )
                        .await
                        .map_err(TelegramClientPortError::Persistence)?,
                ),
            ),
            TelegramClientRequest::Query(query) => self
                .runtime
                .execute_provider_query(query)
                .map(TelegramClientResponse::Query)
                .map_err(TelegramClientPortError::Provider)?,
            TelegramClientRequest::AuthorizationStatus
            | TelegramClientRequest::SubmitAuthorizationPassword { .. } => {
                return Err(TelegramClientPortError::Protocol(
                    "Telegram authorization requests require the authorization port".to_owned(),
                ));
            }
        };
        encode_module_response(contract, request_id, &response)
    }

    pub async fn replay_durable(
        &self,
        durable: &TelegramDurablePersistence,
        account_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<TelegramClientResponse, TelegramClientPortError> {
        if account_id.trim().is_empty() || limit == 0 || limit > MAX_PAGE_SIZE {
            return Err(TelegramClientPortError::Protocol(
                "Telegram realtime replay limit is invalid".to_owned(),
            ));
        }
        if self.runtime.account(account_id).is_none() {
            return Err(TelegramClientPortError::Protocol(
                "Telegram realtime account is unknown".to_owned(),
            ));
        }
        let bounds = durable
            .provider_event_sequence_bounds(account_id)
            .await
            .map_err(TelegramClientPortError::Persistence)?;
        let (earliest_available_sequence, latest_sequence) = bounds.unwrap_or((0, 0));
        let reset_required = match bounds {
            None => after_sequence > 0,
            Some((earliest, latest)) => {
                after_sequence > latest
                    || (after_sequence > 0 && after_sequence.saturating_add(1) < earliest)
            }
        };
        let frames = if reset_required {
            Vec::new()
        } else {
            durable
                .replay_provider_events_after(account_id, after_sequence, i64::from(limit))
                .await
                .map_err(TelegramClientPortError::Persistence)?
        };
        if frames.iter().any(|frame| frame.account_id != account_id)
            || !frames
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        {
            return Err(TelegramClientPortError::Protocol(
                "Telegram realtime persistence returned an invalid page".to_owned(),
            ));
        }
        let next_after_sequence = frames.last().map_or_else(
            || {
                if reset_required {
                    latest_sequence
                } else {
                    after_sequence
                }
            },
            |frame| frame.sequence,
        );
        Ok(TelegramClientResponse::Realtime(
            makosh_telegram_api::TelegramRealtimeReplayPage {
                frames,
                earliest_available_sequence,
                latest_sequence,
                next_after_sequence,
                reset_required,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use makosh_telegram_api::TelegramProviderQuery;

    fn load_chats_request() -> TelegramClientRequest {
        TelegramClientRequest::Query(TelegramProviderQuery::LoadChats {
            account_id: "account-1".to_owned(),
            limit: 25,
        })
    }

    #[test]
    fn query_request_uses_exact_route_contract_and_schema_digest() {
        let encoded = encode_module_request(41, &load_chats_request()).expect("encode query");
        let envelope = ModuleClientRequestV1::decode(encoded.as_slice()).expect("decode envelope");
        let contract = envelope.contract.expect("contract");

        assert_eq!(contract.owner, "telegram");
        assert_eq!(contract.name, "telegram.query.v1");
        assert_eq!(contract.major, 1);
        assert_eq!(contract.revision, 6);
        assert_eq!(contract.schema_sha256.len(), 32);
    }

    #[test]
    fn reconfiguration_request_round_trips_only_through_the_exact_contract() {
        let request = TelegramClientRequest::Reconfiguration(
            makosh_telegram_api::TelegramRuntimeReconfigurationRequest::Begin {
                reconfiguration_id: "reconfiguration-1".to_owned(),
                account_id: "account-1".to_owned(),
                expected_runtime_epoch: 7,
            },
        );
        let encoded = encode_module_request(46, &request).expect("encode reconfiguration");

        assert_eq!(
            decode_module_request(&encoded).expect("decode reconfiguration"),
            (46, TelegramClientContractV1::Reconfiguration, request)
        );
    }

    #[test]
    fn folder_reassignment_round_trips_only_through_the_command_contract() {
        let request = TelegramClientRequest::Command(
            makosh_telegram_api::TelegramProviderCommand::ReassignChatFolders {
                operation_id: "folder-reassign-1".to_owned(),
                account_id: "account-1".to_owned(),
                provider_chat_id: "chat-1".to_owned(),
                target_provider_folder_ids: vec![7, 11],
            },
        );
        let encoded = encode_module_request(47, &request).expect("encode reassignment");

        assert_eq!(
            decode_module_request(&encoded).expect("decode reassignment"),
            (47, TelegramClientContractV1::Command, request)
        );
    }

    #[test]
    fn query_request_decodes_only_as_query() {
        let request = load_chats_request();
        let encoded = encode_module_request(42, &request).expect("encode query");

        assert_eq!(
            decode_module_request(&encoded).expect("decode exact route"),
            (42, TelegramClientContractV1::Query, request)
        );
    }

    #[test]
    fn replay_request_round_trips_only_through_the_realtime_contract() {
        let request = TelegramClientRequest::Replay {
            account_id: "account-1".to_owned(),
            after_sequence: 41,
            limit: 100,
        };
        let encoded = encode_module_request(48, &request).expect("encode replay");

        assert_eq!(
            decode_module_request(&encoded).expect("decode replay"),
            (48, TelegramClientContractV1::Realtime, request)
        );
    }

    #[test]
    fn query_payload_is_rejected_under_lifecycle_contract() {
        let encoded = encode_module_request(43, &load_chats_request()).expect("encode query");
        let mut envelope =
            ModuleClientRequestV1::decode(encoded.as_slice()).expect("decode envelope");
        envelope.contract.as_mut().expect("contract").name = TelegramClientContractV1::Lifecycle
            .contract_name()
            .to_owned();

        assert!(matches!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(TelegramClientPortError::Protocol(_))
        ));
    }

    #[test]
    fn umbrella_client_contract_is_not_admitted() {
        let encoded = encode_module_request(44, &load_chats_request()).expect("encode query");
        let mut envelope =
            ModuleClientRequestV1::decode(encoded.as_slice()).expect("decode envelope");
        envelope.contract.as_mut().expect("contract").name = "telegram.client".to_owned();

        assert!(matches!(
            decode_module_request(&envelope.encode_to_vec()),
            Err(TelegramClientPortError::Protocol(_))
        ));
    }

    #[test]
    fn query_response_decodes_only_with_query_contract() {
        let response = TelegramClientResponse::Query(
            makosh_telegram_api::TelegramProviderQueryResponse::Chats(Vec::new()),
        );
        let encoded =
            encode_module_response(TelegramClientContractV1::Query, 45, &response).expect("encode");

        assert_eq!(
            decode_module_response(TelegramClientContractV1::Query, &encoded)
                .expect("decode query response"),
            (45, response)
        );
        assert!(matches!(
            decode_module_response(TelegramClientContractV1::Lifecycle, &encoded),
            Err(TelegramClientPortError::Protocol(_))
        ));
    }

    #[test]
    fn realtime_response_decodes_only_with_realtime_contract() {
        let response =
            TelegramClientResponse::Realtime(makosh_telegram_api::TelegramRealtimeReplayPage {
                frames: Vec::new(),
                earliest_available_sequence: 0,
                latest_sequence: 0,
                next_after_sequence: 0,
                reset_required: false,
            });
        let encoded = encode_module_response(TelegramClientContractV1::Realtime, 49, &response)
            .expect("encode realtime");

        assert_eq!(
            decode_module_response(TelegramClientContractV1::Realtime, &encoded)
                .expect("decode realtime response"),
            (49, response)
        );
        assert!(matches!(
            decode_module_response(TelegramClientContractV1::Query, &encoded),
            Err(TelegramClientPortError::Protocol(_))
        ));
    }
}

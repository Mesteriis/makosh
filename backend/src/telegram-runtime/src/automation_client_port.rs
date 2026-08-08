//! Telegram automation client adapter.
//!
//! This port owns only generated automation wire conversion and application
//! dispatch. Provider execution remains in the operational port and TDLib.

use std::time::{SystemTime, UNIX_EPOCH};

use makosh_runtime_protocol::v1::{ContractReferenceV1, ModuleClientRequestV1};
use makosh_telegram_automation_api::{
    contract::{
        TELEGRAM_AUTOMATION_CONTRACT_MAJOR, TELEGRAM_AUTOMATION_CONTRACT_REVISION,
        TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1, TELEGRAM_AUTOMATION_MODULE_ID,
        TELEGRAM_AUTOMATION_OWNER_ID, TelegramAutomationContractV1,
    },
    wire::{
        AutomationCommandRequestV1, AutomationCommandResponseV1, AutomationFailureCodeV1,
        AutomationFailureV1, AutomationPolicyListV1, AutomationPolicyV1,
        AutomationPreviewReceiptV1, AutomationQueryRequestV1, AutomationQueryResponseV1,
        AutomationTemplateListV1, AutomationTemplateV1, automation_command_request_v1,
        automation_command_response_v1, automation_query_request_v1, automation_query_response_v1,
    },
};
use makosh_telegram_automation_core::{
    AutomationError, AutomationPolicy, AutomationPolicyDraft, AutomationPreviewReceipt,
    AutomationPreviewRequest, AutomationTemplate, AutomationTemplateDraft, AutomationVariable,
    validate_identifier,
};
use makosh_telegram_automation_persistence::{
    PersistedMutation, TelegramAutomationPersistence, TelegramAutomationPersistenceError,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::client_port::{
    MODULE_CLIENT_PROTOCOL_MAJOR, TelegramClientPortError, encode_module_response_payload,
};

const MAX_LIST_LIMIT: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramAutomationRoute {
    Query,
    Command,
}

impl From<TelegramAutomationContractV1> for TelegramAutomationRoute {
    fn from(value: TelegramAutomationContractV1) -> Self {
        match value {
            TelegramAutomationContractV1::Query => Self::Query,
            TelegramAutomationContractV1::Command => Self::Command,
        }
    }
}

pub fn automation_route(
    bytes: &[u8],
) -> Result<Option<TelegramAutomationRoute>, TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let Some(contract) = envelope.contract.as_ref() else {
        return Err(TelegramClientPortError::Protocol(
            "Telegram client contract is missing".to_owned(),
        ));
    };
    let Some(route) = TelegramAutomationContractV1::from_contract_name(&contract.name) else {
        return Ok(None);
    };
    validate_automation_envelope(&envelope, contract, route)?;
    Ok(Some(route.into()))
}

pub async fn handle_automation_module_request(
    bytes: &[u8],
    persistence: &TelegramAutomationPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let now_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            TelegramClientPortError::Protocol("Telegram automation clock is invalid".to_owned())
        })?
        .as_secs();
    handle_automation_module_request_at(bytes, persistence, now_unix_seconds).await
}

pub async fn handle_automation_module_request_at(
    bytes: &[u8],
    persistence: &TelegramAutomationPersistence,
    now_unix_seconds: u64,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let envelope = ModuleClientRequestV1::decode(bytes)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let contract = envelope.contract.as_ref().ok_or_else(|| {
        TelegramClientPortError::Protocol("Telegram client contract is missing".to_owned())
    })?;
    let route =
        TelegramAutomationContractV1::from_contract_name(&contract.name).ok_or_else(|| {
            TelegramClientPortError::Protocol(
                "Telegram automation route is not admitted".to_owned(),
            )
        })?;
    validate_automation_envelope(&envelope, contract, route)?;
    let response_payload = match route {
        TelegramAutomationContractV1::Query => {
            handle_query(&envelope.request_payload, persistence).await?
        }
        TelegramAutomationContractV1::Command => {
            handle_command(&envelope.request_payload, persistence, now_unix_seconds).await?
        }
    };
    encode_module_response_payload(envelope.request_id, response_payload)
}

fn validate_automation_envelope(
    envelope: &ModuleClientRequestV1,
    contract: &ContractReferenceV1,
    route: TelegramAutomationContractV1,
) -> Result<(), TelegramClientPortError> {
    if envelope.protocol_major != MODULE_CLIENT_PROTOCOL_MAJOR
        || envelope.module_id != TELEGRAM_AUTOMATION_MODULE_ID
        || envelope.owner_id != TELEGRAM_AUTOMATION_OWNER_ID
        || envelope.request_id == 0
        || envelope.request_payload.is_empty()
        || contract.owner != TELEGRAM_AUTOMATION_OWNER_ID
        || contract.name != route.contract_name()
        || contract.major != TELEGRAM_AUTOMATION_CONTRACT_MAJOR
        || contract.revision != TELEGRAM_AUTOMATION_CONTRACT_REVISION
        || contract.schema_sha256
            != Sha256::digest(TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1).as_slice()
    {
        return Err(TelegramClientPortError::Protocol(
            "Telegram automation client routing metadata is not admitted".to_owned(),
        ));
    }
    Ok(())
}

async fn handle_query(
    payload: &[u8],
    persistence: &TelegramAutomationPersistence,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let request = AutomationQueryRequestV1::decode(payload)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let response = match request.request {
        Some(automation_query_request_v1::Request::ListTemplates(query)) => {
            match validated_page(&query.after_template_id, query.limit) {
                Ok(limit) => match persistence
                    .list_templates(&query.after_template_id, limit)
                    .await
                {
                    Ok(items) => {
                        let next_after_template_id =
                            next_template_cursor(&items, limit, &query.after_template_id);
                        query_response(automation_query_response_v1::Response::Templates(
                            AutomationTemplateListV1 {
                                items: items.iter().map(template_wire).collect(),
                                next_after_template_id,
                            },
                        ))
                    }
                    Err(error) => query_failure(persistence_failure(error)),
                },
                Err(error) => query_failure(core_failure(error)),
            }
        }
        Some(automation_query_request_v1::Request::GetTemplate(query)) => {
            if let Err(error) = validate_identifier("template_id", &query.template_id) {
                query_failure(core_failure(error))
            } else {
                match persistence.template(&query.template_id).await {
                    Ok(Some(template)) => query_response(
                        automation_query_response_v1::Response::Template(template_wire(&template)),
                    ),
                    Ok(None) => query_failure(not_found_failure("template_id")),
                    Err(error) => query_failure(persistence_failure(error)),
                }
            }
        }
        Some(automation_query_request_v1::Request::ListPolicies(query)) => {
            match validated_page(&query.after_policy_id, query.limit) {
                Ok(limit) => match persistence
                    .list_policies(&query.after_policy_id, limit)
                    .await
                {
                    Ok(items) => {
                        let next_after_policy_id =
                            next_policy_cursor(&items, limit, &query.after_policy_id);
                        query_response(automation_query_response_v1::Response::Policies(
                            AutomationPolicyListV1 {
                                items: items.iter().map(policy_wire).collect(),
                                next_after_policy_id,
                            },
                        ))
                    }
                    Err(error) => query_failure(persistence_failure(error)),
                },
                Err(error) => query_failure(core_failure(error)),
            }
        }
        Some(automation_query_request_v1::Request::GetPolicy(query)) => {
            if let Err(error) = validate_identifier("policy_id", &query.policy_id) {
                query_failure(core_failure(error))
            } else {
                match persistence.policy(&query.policy_id).await {
                    Ok(Some(policy)) => query_response(
                        automation_query_response_v1::Response::Policy(policy_wire(&policy)),
                    ),
                    Ok(None) => query_failure(not_found_failure("policy_id")),
                    Err(error) => query_failure(persistence_failure(error)),
                }
            }
        }
        Some(automation_query_request_v1::Request::GetPreviewReceipt(query)) => {
            if let Err(error) = validate_identifier("preview_id", &query.preview_id) {
                query_failure(core_failure(error))
            } else {
                match persistence.preview_receipt(&query.preview_id).await {
                    Ok(Some(receipt)) => {
                        query_response(automation_query_response_v1::Response::PreviewReceipt(
                            preview_wire(&receipt),
                        ))
                    }
                    Ok(None) => query_failure(not_found_failure("preview_id")),
                    Err(error) => query_failure(persistence_failure(error)),
                }
            }
        }
        None => query_failure(invalid_failure("request")),
    };
    Ok(response.encode_to_vec())
}

async fn handle_command(
    payload: &[u8],
    persistence: &TelegramAutomationPersistence,
    now_unix_seconds: u64,
) -> Result<Vec<u8>, TelegramClientPortError> {
    let request = AutomationCommandRequestV1::decode(payload)
        .map_err(|error| TelegramClientPortError::Codec(error.to_string()))?;
    let request_sha256: [u8; 32] = Sha256::digest(payload).into();
    match request.command {
        Some(automation_command_request_v1::Command::UpsertTemplate(command)) => {
            let draft = AutomationTemplateDraft {
                template_id: command.template_id,
                name: command.name,
                body_template: command.body_template,
                required_variables: command.required_variables,
            };
            match persistence
                .upsert_template(
                    &command.mutation_id,
                    &request_sha256,
                    command.expected_revision,
                    &draft,
                    now_unix_seconds,
                    |template| {
                        command_response(automation_command_response_v1::Response::Template(
                            template_wire(template),
                        ))
                        .encode_to_vec()
                    },
                )
                .await
            {
                Ok(result) => Ok(persisted_response(result)),
                Err(error) => Ok(command_failure(persistence_failure(error)).encode_to_vec()),
            }
        }
        Some(automation_command_request_v1::Command::UpsertPolicy(command)) => {
            let draft = AutomationPolicyDraft {
                policy_id: command.policy_id,
                template_id: command.template_id,
                name: command.name,
                enabled: command.enabled,
                account_id: command.account_id,
                provider_chat_ids: command.provider_chat_ids,
                expires_at_unix_seconds: command.expires_at_unix_seconds,
            };
            match persistence
                .upsert_policy(
                    &command.mutation_id,
                    &request_sha256,
                    command.expected_revision,
                    &draft,
                    now_unix_seconds,
                    |policy| {
                        command_response(automation_command_response_v1::Response::Policy(
                            policy_wire(policy),
                        ))
                        .encode_to_vec()
                    },
                )
                .await
            {
                Ok(result) => Ok(persisted_response(result)),
                Err(error) => Ok(command_failure(persistence_failure(error)).encode_to_vec()),
            }
        }
        Some(automation_command_request_v1::Command::PreviewPolicy(command)) => {
            let request = AutomationPreviewRequest {
                preview_id: command.preview_id,
                policy_id: command.policy_id,
                account_id: command.account_id,
                provider_chat_id: command.provider_chat_id,
                variables: command
                    .variables
                    .into_iter()
                    .map(|variable| AutomationVariable {
                        name: variable.name,
                        value: variable.value,
                    })
                    .collect(),
            };
            match persistence
                .preview_policy(&request_sha256, &request, now_unix_seconds, |preview| {
                    command_response(automation_command_response_v1::Response::Preview(
                        preview_wire(preview),
                    ))
                    .encode_to_vec()
                })
                .await
            {
                Ok(result) => Ok(persisted_response(result)),
                Err(error) => Ok(command_failure(persistence_failure(error)).encode_to_vec()),
            }
        }
        None => Ok(command_failure(invalid_failure("command")).encode_to_vec()),
    }
}

fn template_wire(template: &AutomationTemplate) -> AutomationTemplateV1 {
    AutomationTemplateV1 {
        template_id: template.template_id.clone(),
        name: template.name.clone(),
        body_template: template.body_template.clone(),
        required_variables: template.required_variables.clone(),
        revision: template.revision,
        created_at_unix_seconds: template.created_at_unix_seconds,
        updated_at_unix_seconds: template.updated_at_unix_seconds,
    }
}

fn policy_wire(policy: &AutomationPolicy) -> AutomationPolicyV1 {
    AutomationPolicyV1 {
        policy_id: policy.policy_id.clone(),
        template_id: policy.template_id.clone(),
        name: policy.name.clone(),
        enabled: policy.enabled,
        account_id: policy.account_id.clone(),
        provider_chat_ids: policy.provider_chat_ids.clone(),
        expires_at_unix_seconds: policy.expires_at_unix_seconds,
        revision: policy.revision,
        created_at_unix_seconds: policy.created_at_unix_seconds,
        updated_at_unix_seconds: policy.updated_at_unix_seconds,
    }
}

fn preview_wire(preview: &AutomationPreviewReceipt) -> AutomationPreviewReceiptV1 {
    AutomationPreviewReceiptV1 {
        preview_id: preview.preview_id.clone(),
        policy_id: preview.policy_id.clone(),
        policy_revision: preview.policy_revision,
        template_id: preview.template_id.clone(),
        template_revision: preview.template_revision,
        account_id: preview.account_id.clone(),
        provider_chat_id: preview.provider_chat_id.clone(),
        rendered_text: preview.rendered_text.clone(),
        rendered_sha256: preview.rendered_sha256.to_vec(),
        created_at_unix_seconds: preview.created_at_unix_seconds,
    }
}

fn persisted_response<T>(result: PersistedMutation<T>) -> Vec<u8> {
    match result {
        PersistedMutation::Applied {
            response_payload, ..
        }
        | PersistedMutation::Replayed { response_payload } => response_payload,
    }
}

fn command_response(
    response: automation_command_response_v1::Response,
) -> AutomationCommandResponseV1 {
    AutomationCommandResponseV1 {
        response: Some(response),
    }
}

fn command_failure(failure: AutomationFailureV1) -> AutomationCommandResponseV1 {
    command_response(automation_command_response_v1::Response::Failure(failure))
}

fn query_response(response: automation_query_response_v1::Response) -> AutomationQueryResponseV1 {
    AutomationQueryResponseV1 {
        response: Some(response),
    }
}

fn query_failure(failure: AutomationFailureV1) -> AutomationQueryResponseV1 {
    query_response(automation_query_response_v1::Response::Failure(failure))
}

fn validated_page(cursor: &str, limit: u32) -> Result<u32, AutomationError> {
    if limit == 0 || limit > MAX_LIST_LIMIT {
        return Err(AutomationError::InvalidRequest("limit"));
    }
    if !cursor.is_empty() {
        validate_identifier("after_id", cursor)?;
    }
    Ok(limit)
}

fn next_template_cursor(items: &[AutomationTemplate], limit: u32, previous: &str) -> String {
    if items.len() == limit as usize {
        items
            .last()
            .map(|item| item.template_id.clone())
            .unwrap_or_else(|| previous.to_owned())
    } else {
        String::new()
    }
}

fn next_policy_cursor(items: &[AutomationPolicy], limit: u32, previous: &str) -> String {
    if items.len() == limit as usize {
        items
            .last()
            .map(|item| item.policy_id.clone())
            .unwrap_or_else(|| previous.to_owned())
    } else {
        String::new()
    }
}

fn persistence_failure(error: TelegramAutomationPersistenceError) -> AutomationFailureV1 {
    match error {
        TelegramAutomationPersistenceError::MissingTemplate => not_found_failure("template_id"),
        TelegramAutomationPersistenceError::MissingAccount => not_found_failure("account_id"),
        TelegramAutomationPersistenceError::MissingPolicy => not_found_failure("policy_id"),
        TelegramAutomationPersistenceError::RevisionConflict => failure(
            AutomationFailureCodeV1::AutomationFailureCodeRevisionConflict,
            "expected_revision",
        ),
        TelegramAutomationPersistenceError::IdempotencyConflict => failure(
            AutomationFailureCodeV1::AutomationFailureCodeIdempotencyConflict,
            "idempotency_key",
        ),
        TelegramAutomationPersistenceError::Core(error) => core_failure(error),
        TelegramAutomationPersistenceError::Database
        | TelegramAutomationPersistenceError::InvalidRow => failure(
            AutomationFailureCodeV1::AutomationFailureCodeUnavailable,
            "",
        ),
    }
}

fn core_failure(error: AutomationError) -> AutomationFailureV1 {
    match error {
        AutomationError::InvalidRequest(field) => invalid_failure(field),
        AutomationError::NotFound(field) => not_found_failure(field),
        AutomationError::RevisionConflict => failure(
            AutomationFailureCodeV1::AutomationFailureCodeRevisionConflict,
            "expected_revision",
        ),
        AutomationError::IdempotencyConflict => failure(
            AutomationFailureCodeV1::AutomationFailureCodeIdempotencyConflict,
            "idempotency_key",
        ),
        AutomationError::PolicyDisabled => failure(
            AutomationFailureCodeV1::AutomationFailureCodePolicyDisabled,
            "policy_id",
        ),
        AutomationError::PolicyExpired => failure(
            AutomationFailureCodeV1::AutomationFailureCodePolicyExpired,
            "policy_id",
        ),
        AutomationError::ScopeDenied => failure(
            AutomationFailureCodeV1::AutomationFailureCodeScopeDenied,
            "provider_chat_id",
        ),
        AutomationError::MissingVariable => failure(
            AutomationFailureCodeV1::AutomationFailureCodeVariableMissing,
            "variables",
        ),
        AutomationError::UndeclaredVariable => failure(
            AutomationFailureCodeV1::AutomationFailureCodeVariableUndeclared,
            "variables",
        ),
        AutomationError::Unavailable => failure(
            AutomationFailureCodeV1::AutomationFailureCodeUnavailable,
            "",
        ),
    }
}

fn invalid_failure(field: &str) -> AutomationFailureV1 {
    failure(
        AutomationFailureCodeV1::AutomationFailureCodeInvalidRequest,
        field,
    )
}

fn not_found_failure(field: &str) -> AutomationFailureV1 {
    failure(
        AutomationFailureCodeV1::AutomationFailureCodeNotFound,
        field,
    )
}

fn failure(code: AutomationFailureCodeV1, field: &str) -> AutomationFailureV1 {
    AutomationFailureV1 {
        code: code as i32,
        field: field.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_envelope(
        route: TelegramAutomationContractV1,
        payload: Vec<u8>,
    ) -> ModuleClientRequestV1 {
        ModuleClientRequestV1 {
            protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
            module_id: TELEGRAM_AUTOMATION_MODULE_ID.to_owned(),
            owner_id: TELEGRAM_AUTOMATION_OWNER_ID.to_owned(),
            contract: Some(ContractReferenceV1 {
                owner: TELEGRAM_AUTOMATION_OWNER_ID.to_owned(),
                name: route.contract_name().to_owned(),
                major: TELEGRAM_AUTOMATION_CONTRACT_MAJOR,
                revision: TELEGRAM_AUTOMATION_CONTRACT_REVISION,
                schema_sha256: Sha256::digest(TELEGRAM_AUTOMATION_DESCRIPTOR_SET_V1).to_vec(),
            }),
            request_id: 42,
            request_payload: payload,
            logical_owner_id: String::new(),
            authenticated_device_id: String::new(),
            authenticated_client_session_id: String::new(),
        }
    }

    #[test]
    fn exact_automation_route_is_detected_without_accepting_umbrella_contract() {
        let query = AutomationQueryRequestV1 {
            request: Some(automation_query_request_v1::Request::GetTemplate(
                makosh_telegram_automation_api::wire::GetAutomationTemplateQueryV1 {
                    template_id: "template-1".to_owned(),
                },
            )),
        };
        let envelope = request_envelope(TelegramAutomationContractV1::Query, query.encode_to_vec());
        assert!(matches!(
            automation_route(&envelope.encode_to_vec()),
            Ok(Some(TelegramAutomationRoute::Query))
        ));

        let mut umbrella = envelope;
        umbrella.contract.as_mut().expect("contract").name = "telegram.client".to_owned();
        assert!(matches!(
            automation_route(&umbrella.encode_to_vec()),
            Ok(None)
        ));
    }

    #[test]
    fn wrong_schema_digest_is_rejected_before_payload_decode() {
        let mut envelope = request_envelope(
            TelegramAutomationContractV1::Command,
            AutomationCommandRequestV1 { command: None }.encode_to_vec(),
        );
        envelope.contract.as_mut().expect("contract").schema_sha256 = vec![0; 32];
        assert!(matches!(
            automation_route(&envelope.encode_to_vec()),
            Err(TelegramClientPortError::Protocol(_))
        ));
    }

    #[test]
    fn failures_are_typed_and_do_not_include_private_values() {
        assert_eq!(
            core_failure(AutomationError::MissingVariable),
            AutomationFailureV1 {
                code: AutomationFailureCodeV1::AutomationFailureCodeVariableMissing as i32,
                field: "variables".to_owned(),
            }
        );
        assert_eq!(
            persistence_failure(TelegramAutomationPersistenceError::Database).field,
            ""
        );
    }
}

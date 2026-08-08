use super::super::*;
use crate::app::handlers::communications::legal_export::{SendRequest, SendResponse};
use crate::app::provider_runtime_handlers::whatsapp::messages::{
    post_whatsapp_command_forward, post_whatsapp_command_reply,
};
use crate::application::communication_provider_models::{
    CommunicationForwardRequest, CommunicationProviderMessageCommandResponse,
    CommunicationReplyRequest,
};
use crate::application::communication_provider_writes::new_telegram_command_id;
use crate::domains::communications::command_service::CommunicationCommandService;
use crate::integrations::whatsapp::runtime::contracts::{
    WhatsAppForwardRequest, WhatsAppProviderCommandResponse, WhatsAppReplyRequest,
};

pub(crate) async fn post_v1_reply(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    if msg.channel_kind.starts_with("whatsapp") {
        let mut request: CommunicationReplyRequest = serde_json::from_value(req)
            .map_err(|_| ApiError::InvalidCommunicationQuery("invalid WhatsApp reply payload"))?;
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(next_whatsapp_command_id);
        request.command_id = Some(command_id.clone());
        let provider_chat_id =
            msg.conversation_id
                .clone()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "whatsapp message is missing provider conversation metadata",
                ))?;
        let response = post_whatsapp_command_reply(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppReplyRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("reply", &command_id),
                account_id: msg.account_id.clone(),
                provider_chat_id: provider_chat_id.clone(),
                reply_to_provider_message_id: msg.provider_record_id.clone(),
                text: request.text,
            }),
        )
        .await?
        .0;
        return Ok(Json(json!(
            whatsapp_command_response_to_communication_response(
                &command_id,
                &provider_chat_id,
                Some(&message_id),
                &response,
            )
        )));
    }
    if msg.channel_kind.starts_with("telegram") {
        let mut request: CommunicationReplyRequest = serde_json::from_value(req)
            .map_err(|_| ApiError::InvalidCommunicationQuery("invalid Telegram reply payload"))?;
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(new_telegram_command_id);
        request.command_id = Some(command_id.clone());
        let runtime_context = telegram_runtime_use_case_context(&state)?;
        let response = telegram_message_write_service(&state)?
            .reply_to_message(&runtime_context, &message_id, request)
            .await?;
        return Ok(Json(json!(
            CommunicationProviderMessageCommandResponse::telegram(command_id, &response)
        )));
    }

    let req: SendRequest = serde_json::from_value(req)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid reply payload"))?;
    let quoted = msg
        .body_text
        .lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    let _body = format!(
        "{}\n\nOn {}, {} wrote:\n{}",
        req.body_text,
        msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        msg.sender,
        quoted
    );
    Ok(Json(json!(SendResponse {
        message_id: format!(
            "reply-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ),
        outbox_id: None,
        accepted: req.to.clone(),
        accepted_recipients: req.to.clone(),
        transport: "local".to_owned(),
        status: "queued".to_owned(),
        scheduled_send_at: None,
        undo_deadline_at: None,
        failure_reason: None,
    })))
}

#[derive(Deserialize)]
pub(crate) struct ForwardRequest {
    pub(super) to: Vec<String>,
    pub(super) cc: Option<Vec<String>>,
    pub(super) note: Option<String>,
}

pub(crate) async fn post_v1_forward(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    if msg.channel_kind.starts_with("whatsapp") {
        let mut request: CommunicationForwardRequest = serde_json::from_value(req)
            .map_err(|_| ApiError::InvalidCommunicationQuery("invalid WhatsApp forward payload"))?;
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(next_whatsapp_command_id);
        request.command_id = Some(command_id.clone());
        let from_provider_chat_id =
            msg.conversation_id
                .clone()
                .ok_or(ApiError::InvalidCommunicationQuery(
                    "whatsapp message is missing provider conversation metadata",
                ))?;
        let response = post_whatsapp_command_forward(
            State(state.clone()),
            Path(message_id.clone()),
            Json(WhatsAppForwardRequest {
                command_id: Some(command_id.clone()),
                idempotency_key: whatsapp_command_idempotency_key("forward", &command_id),
                account_id: msg.account_id.clone(),
                provider_chat_id: request.conversation_id.clone(),
                from_provider_chat_id,
                from_provider_message_id: msg.provider_record_id.clone(),
                text: Some(msg.body_text.clone()),
            }),
        )
        .await?
        .0;
        return Ok(Json(json!(
            whatsapp_command_response_to_communication_response(
                &command_id,
                &request.conversation_id,
                Some(&message_id),
                &response,
            )
        )));
    }
    if msg.channel_kind.starts_with("telegram") {
        let mut request: CommunicationForwardRequest = serde_json::from_value(req)
            .map_err(|_| ApiError::InvalidCommunicationQuery("invalid Telegram forward payload"))?;
        let command_id = request
            .command_id
            .clone()
            .unwrap_or_else(new_telegram_command_id);
        request.command_id = Some(command_id.clone());
        let runtime_context = telegram_runtime_use_case_context(&state)?;
        let response = telegram_message_write_service(&state)?
            .forward_message(&runtime_context, &message_id, request)
            .await?;
        return Ok(Json(json!(
            CommunicationProviderMessageCommandResponse::telegram(command_id, &response)
        )));
    }

    let req: ForwardRequest = serde_json::from_value(req)
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid forward payload"))?;
    let cc = req.cc.unwrap_or_default();
    let note = req.note.as_deref().unwrap_or("");
    let fwd_body = format!(
        "{note}\n\n--- Forwarded message ---\nFrom: {}\nSubject: {}\nDate: {}\n\n{}",
        msg.sender,
        msg.subject,
        msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        msg.body_text
    );
    Ok(Json(
        serde_json::json!({"forwarded": true, "to": req.to, "cc": cc, "subject": format!("Fwd: {}", msg.subject), "body_preview": &fwd_body[..200.min(fwd_body.len())]}),
    ))
}

fn next_whatsapp_command_id() -> String {
    format!(
        "whatsapp-command-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

fn whatsapp_command_idempotency_key(operation: &str, command_id: &str) -> String {
    format!("communications:whatsapp:{operation}:{command_id}")
}

fn whatsapp_command_response_to_communication_response(
    command_id: &str,
    conversation_id: &str,
    message_id: Option<&str>,
    response: &WhatsAppProviderCommandResponse,
) -> CommunicationProviderMessageCommandResponse {
    CommunicationProviderMessageCommandResponse {
        message_id: message_id.unwrap_or(command_id).to_owned(),
        raw_record_id: String::new(),
        conversation_id: conversation_id.to_owned(),
        provider_chat_id: response.provider_chat_id.clone(),
        provider_message_id: response.provider_message_id.clone(),
        channel_kind: "whatsapp_web",
        status: "queued".to_owned(),
        command_id: response.command_id.clone(),
        provider: "whatsapp",
    }
}

#[derive(Deserialize)]
pub(crate) struct ReplyAllRequest {
    pub(super) body_text: String,
    pub(super) quote: Option<bool>,
}

pub(crate) async fn post_v1_reply_all(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ReplyAllRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let body = crate::domains::communications::actions::build_reply_body(
        &msg.sender,
        &msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        &msg.body_text,
        &req.body_text,
        req.quote.unwrap_or(true),
    );
    Ok(Json(
        serde_json::json!({"reply_all": true, "to": msg.recipients, "subject": format!("Re: {}", msg.subject), "body": body}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct ForwardEmlRequest {
    pub(super) to: Vec<String>,
}

pub(crate) async fn post_v1_forward_eml(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<ForwardEmlRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let eml = crate::domains::communications::actions::build_eml_forward(
        &msg.sender,
        &msg.occurred_at.map(|d| d.to_rfc2822()).unwrap_or_default(),
        &msg.subject,
        &msg.body_text,
        &req.to,
    );
    Ok(Json(
        serde_json::json!({"forward_eml": true, "eml_size": eml.len()}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct RedirectRequest {
    pub(super) to: Vec<String>,
    pub(super) cc: Option<Vec<String>>,
    pub(super) bcc: Option<Vec<String>>,
    pub(super) confirmed_provider_write: Option<bool>,
}

pub(crate) async fn post_v1_redirect(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<RedirectRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    if req.confirmed_provider_write != Some(true) {
        return Err(ApiError::ProviderWriteConfirmationRequired);
    }
    let to = non_empty_recipients(req.to);
    let cc = non_empty_recipients(req.cc.unwrap_or_default());
    let bcc = non_empty_recipients(req.bcc.unwrap_or_default());
    if to
        .iter()
        .chain(cc.iter())
        .chain(bcc.iter())
        .all(|recipient| recipient.trim().is_empty())
    {
        return Err(ApiError::InvalidCommunicationQuery(
            "at least one recipient is required",
        ));
    }

    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let recipient_count = to.len() + cc.len() + bcc.len();
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let outbox = CommunicationCommandService::new(pool)
        .enqueue_redirect_message(&msg.message_id, to.clone(), cc, bcc)
        .await?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::communication_email_send(
            "makosh-frontend",
            &outbox.account_id,
            recipient_count,
        ))
        .await?;

    Ok(Json(SendResponse {
        message_id: outbox.outbox_id.clone(),
        outbox_id: Some(outbox.outbox_id),
        accepted: outbox.to_recipients.clone(),
        accepted_recipients: outbox.to_recipients,
        transport: "outbox".to_owned(),
        status: outbox.status.as_str().to_owned(),
        scheduled_send_at: outbox.scheduled_send_at,
        undo_deadline_at: outbox.undo_deadline_at,
        failure_reason: None,
    }))
}

fn non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    recipients
        .into_iter()
        .map(|recipient| recipient.trim().to_owned())
        .filter(|recipient| !recipient.is_empty())
        .collect()
}

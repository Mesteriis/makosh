# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `029-source-backend-part-009`
- Group / Группа: `backend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/backend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `backend/src/app/handlers/communications/sending/bilingual_reply_flow.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/bilingual_reply_flow.rs`
- Size bytes / Размер в байтах: `8136`
- Included characters / Включено символов: `8136`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

const MAX_BILINGUAL_REPLY_TEXT_CHARS: usize = 64_000;
const BILINGUAL_REPLY_TONES: [&str; 5] = ["formal", "business", "friendly", "short", "detailed"];

struct AiSignalContext {
    event_kind: &'static str,
    subject: serde_json::Value,
    provenance: serde_json::Value,
}

#[derive(Deserialize)]
pub(crate) struct BilingualReplyFlowRequest {
    pub(super) reply_text_ru: String,
    pub(super) tone: String,
}

#[derive(Serialize)]
pub(crate) struct BilingualReplyFlowResponse {
    pub(super) message_id: String,
    pub(super) subject: String,
    pub(super) tone: String,
    pub(super) reply_language: &'static str,
    pub(super) send_ready: bool,
    pub(super) original: BilingualOriginal,
    pub(super) translation: BilingualTranslationStep,
    pub(super) reply: BilingualReplyDraft,
    pub(super) back_translation: BilingualTranslationStep,
}

#[derive(Serialize)]
pub(crate) struct BilingualOriginal {
    pub(super) language: String,
    pub(super) confidence: f32,
    pub(super) text: String,
}

#[derive(Serialize)]
pub(crate) struct BilingualTranslationStep {
    pub(super) target: String,
    pub(super) translated: bool,
    pub(super) text: Option<String>,
    pub(super) model: Option<String>,
    pub(super) reason: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct BilingualReplyDraft {
    pub(super) language: &'static str,
    pub(super) tone: String,
    pub(super) text: String,
}

pub(crate) async fn post_v1_bilingual_reply_flow(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<BilingualReplyFlowRequest>,
) -> Result<Json<BilingualReplyFlowResponse>, ApiError> {
    let reply_text = req.reply_text_ru.trim();
    if reply_text.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "reply_text_ru is required",
        ));
    }
    if reply_text.chars().count() > MAX_BILINGUAL_REPLY_TEXT_CHARS {
        return Err(ApiError::InvalidCommunicationQuery(
            "reply_text_ru exceeds maximum length",
        ));
    }

    let tone = normalize_bilingual_reply_tone(&req.tone)?;
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let detection =
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            &msg.body_text,
        );
    let original_language = detection.language.clone();
    let back_translation_target = if original_language == "unknown" {
        "en".to_owned()
    } else {
        original_language.clone()
    };
    let service = email_multilingual_service(&state).await?;

    let translation = bilingual_translation_step_with_signal(
        &state,
        &service,
        &msg.body_text,
        "ru",
        &message_id,
        AiSignalContext {
            event_kind: "bilingual_reply_inbound_translation",
            subject: json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "bilingual_reply_inbound_translation",
            }),
            provenance: json!({
                "source": "bilingual_reply_flow_inbound_translation",
                "message_id": message_id,
            }),
        },
    )
    .await?;
    let back_translation = bilingual_translation_step_with_signal(
        &state,
        &service,
        reply_text,
        &back_translation_target,
        &message_id,
        AiSignalContext {
            event_kind: "bilingual_reply_back_translation",
            subject: json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "bilingual_reply_back_translation",
            }),
            provenance: json!({
                "source": "bilingual_reply_flow_back_translation",
                "message_id": message_id,
            }),
        },
    )
    .await?;
    let send_ready = translation.translated && back_translation.translated;

    Ok(Json(BilingualReplyFlowResponse {
        message_id: msg.message_id,
        subject: reply_subject(&msg.subject),
        tone: tone.clone(),
        reply_language: "ru",
        send_ready,
        original: BilingualOriginal {
            language: original_language,
            confidence: detection.confidence,
            text: msg.body_text,
        },
        translation,
        reply: BilingualReplyDraft {
            language: "ru",
            tone,
            text: reply_text.to_owned(),
        },
        back_translation,
    }))
}

fn normalize_bilingual_reply_tone(value: &str) -> Result<String, ApiError> {
    let tone = value.trim().to_ascii_lowercase();
    if BILINGUAL_REPLY_TONES.contains(&tone.as_str()) {
        return Ok(tone);
    }
    Err(ApiError::InvalidCommunicationQuery(
        "unsupported bilingual reply tone",
    ))
}

async fn bilingual_translation_step(
    state: &AppState,
    service: &crate::domains::communications::multilingual::MultilingualService,
    text: &str,
    target_language: &str,
    message_id: &str,
) -> Result<BilingualTranslationStep, ApiError> {
    bilingual_translation_step_with_signal(
        state,
        service,
        text,
        target_language,
        message_id,
        AiSignalContext {
            event_kind: "bilingual_reply_translation",
            subject: json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "bilingual_reply_translation",
            }),
            provenance: json!({
                "source": "bilingual_reply_flow_translation",
                "message_id": message_id,
            }),
        },
    )
    .await
}

async fn bilingual_translation_step_with_signal(
    state: &AppState,
    service: &crate::domains::communications::multilingual::MultilingualService,
    text: &str,
    target_language: &str,
    message_id: &str,
    signal: AiSignalContext,
) -> Result<BilingualTranslationStep, ApiError> {
    match service.translate(text, target_language).await {
        Ok(Some(translation)) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::dispatch_ai_helper_signal(
                    pool.clone(),
                    signal.event_kind,
                    message_id,
                    signal.subject,
                    json!({
                        "target_language": translation.target_language,
                        "model": translation.model,
                    }),
                    signal.provenance,
                    None,
                )
                .await?;
            }

            Ok(BilingualTranslationStep {
                target: translation.target_language,
                translated: true,
                text: Some(translation.translated_text),
                model: Some(translation.model),
                reason: None,
            })
        }
        Ok(None) => Ok(BilingualTranslationStep {
            target: target_language.to_owned(),
            translated: false,
            text: None,
            model: None,
            reason: Some("no LLM configured".to_owned()),
        }),
        Err(error) => {
            tracing::warn!(
                error = %error,
                message_id = %message_id,
                target_language = %target_language,
                "bilingual reply translation failed"
            );
            Ok(BilingualTranslationStep {
                target: target_language.to_owned(),
                translated: false,
                text: None,
                model: None,
                reason: Some("translation runtime unavailable".to_owned()),
            })
        }
    }
}

fn reply_subject(subject: &str) -> String {
    if subject.trim_start().to_ascii_lowercase().starts_with("re:") {
        subject.to_owned()
    } else {
        format!("Re: {subject}")
    }
}
```

### `backend/src/app/handlers/communications/sending/certificates.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/certificates.rs`
- Size bytes / Размер в байтах: `5686`
- Included characters / Включено символов: `5686`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct CertsQuery {
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct CertsListResponse {
    pub(super) items: Vec<crate::domains::communications::signatures::CertificateRecord>,
}

pub(crate) async fn get_v1_certs(
    State(state): State<AppState>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::signatures::CertificateStore,
    >(pool);
    Ok(Json(CertsListResponse {
        items: store.list().await?,
    }))
}

#[derive(Deserialize)]
pub(crate) struct NewCertRequest {
    pub(super) cert_id: String,
    pub(super) owner_name: String,
    pub(super) issuer: String,
    pub(super) serial_number: Option<String>,
    pub(super) fingerprint_sha256: Option<String>,
    pub(super) valid_from: Option<DateTime<Utc>>,
    pub(super) valid_until: Option<DateTime<Utc>>,
    pub(super) cert_type: Option<String>,
    pub(super) provider: Option<String>,
    pub(super) storage_kind: Option<String>,
    pub(super) storage_ref: Option<String>,
    pub(super) trust_status: Option<String>,
    pub(super) is_revoked: Option<bool>,
    pub(super) usage: Option<Vec<String>>,
    pub(super) linked_message_id: Option<String>,
    pub(super) metadata: Option<Value>,
}

pub(crate) async fn post_v1_cert(
    State(state): State<AppState>,
    Json(req): Json<NewCertRequest>,
) -> Result<Json<crate::domains::communications::signatures::CertificateRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::signatures::CertificateStore,
    >(pool);
    Ok(Json(
        store
            .upsert(&crate::domains::communications::signatures::NewCertificate {
                cert_id: req.cert_id,
                owner_name: req.owner_name,
                issuer: req.issuer,
                serial_number: req.serial_number,
                fingerprint_sha256: req.fingerprint_sha256,
                valid_from: req.valid_from,
                valid_until: req.valid_until,
                cert_type: req
                    .cert_type
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::CertificateType::parse)
                    .unwrap_or(crate::domains::communications::signatures::CertificateType::Unknown),
                provider: req
                    .provider
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::CertificateProvider::parse)
                    .unwrap_or(crate::domains::communications::signatures::CertificateProvider::Other),
                storage_kind: req
                    .storage_kind
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::CertificateStorageKind::parse)
                    .unwrap_or(
                        crate::domains::communications::signatures::CertificateStorageKind::EncryptedVault,
                    ),
                storage_ref: req.storage_ref,
                trust_status: req
                    .trust_status
                    .as_deref()
                    .and_then(crate::domains::communications::signatures::TrustStatus::parse)
                    .unwrap_or(crate::domains::communications::signatures::TrustStatus::Untrusted),
                is_revoked: req.is_revoked.unwrap_or(false),
                usage: req.usage.unwrap_or_default(),
                linked_message_id: req.linked_message_id,
                metadata: req.metadata.unwrap_or(serde_json::json!({})),
            })
            .await?,
    ))
}

#[derive(Deserialize)]
pub(crate) struct ExpiringQuery {
    pub(super) days: Option<i64>,
}

pub(crate) async fn get_v1_certs_expiring(
    State(state): State<AppState>,
    Query(query): Query<ExpiringQuery>,
) -> Result<Json<CertsListResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<
        crate::domains::communications::signatures::CertificateStore,
    >(pool);
    Ok(Json(CertsListResponse {
        items: store.expiring_soon(query.days.unwrap_or(90)).await?,
    }))
}

pub(crate) async fn get_v1_signature_check(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::communications::signatures::SignatureDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::communications::signatures::SignatureDetector::detect_in_message(
            &msg.body_text,
            "",
        ),
    ))
}

pub(crate) async fn get_v1_spf_dkim(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let auth = crate::domains::communications::spf_dkim::parse_auth_headers(&msg.body_text);
    let risk = crate::domains::communications::spf_dkim::assess_auth_risk(&auth);
    Ok(Json(serde_json::json!({"auth": auth, "risk": risk})))
}
```

### `backend/src/app/handlers/communications/sending/extraction.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/extraction.rs`
- Size bytes / Размер в байтах: `1966`
- Included characters / Включено символов: `1966`
- Truncated / Обрезано: `no`

```rust
use super::super::*;

pub(crate) async fn post_v1_extract_tasks(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::domains::communications::extract::EmailExtractService::new(
        ai_runtime_port_optional(&state).await?,
    );
    let tasks = svc.extract_tasks(&msg).await?;
    let llm_task_count = tasks.iter().filter(|task| task.source == "llm").count();
    if llm_task_count > 0
        && let Some(pool) = state.database.pool()
    {
        crate::domains::signal_hub::dispatch_ai_helper_signal(
            pool.clone(),
            "message_task_extraction",
            &message_id,
            serde_json::json!({
                "kind": "communication_message",
                "source_code": "ai",
                "message_id": message_id,
                "operation": "task_extraction",
            }),
            serde_json::json!({
                "task_count": tasks.len(),
                "llm_task_count": llm_task_count,
            }),
            serde_json::json!({
                "source": "communication_message_task_extraction",
                "message_id": message_id,
            }),
            None,
        )
        .await?;
    }
    Ok(Json(serde_json::json!({"tasks": tasks})))
}

pub(crate) async fn post_v1_extract_notes(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let svc = crate::domains::communications::extract::EmailExtractService::new(None);
    let notes = svc.extract_notes(&msg).await?;
    Ok(Json(serde_json::json!({"notes": notes})))
}
```

### `backend/src/app/handlers/communications/sending/forwarding.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/forwarding.rs`
- Size bytes / Размер в байтах: `12861`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::super::*;
use crate::app::provider_runtime_handlers::whatsapp::{
    post_whatsapp_command_forward, post_whatsapp_command_reply,
};
use crate::application::communication_provider_writes::{
    CommunicationForwardRequest, CommunicationProviderMessageCommandResponse,
    CommunicationReplyRequest, new_telegram_command_id,
};
use crate::application::provider_runtime_contracts::{
    WhatsAppForwardRequest, WhatsAppProviderCommandResponse, WhatsAppReplyRequest,
};
use crate::domains::communications::service::CommunicationCommandService;

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
        channel_kind: if response.provider_kind == "whatsapp_business_cloud" {
            "whatsapp_business_cloud"
        } else {
            "whatsapp_web"
        },
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

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/handlers/communications/sending/local_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/local_state.rs`
- Size bytes / Размер в байтах: `2270`
- Included characters / Включено символов: `2270`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::domains::communications::service::CommunicationCommandService;

pub(crate) async fn post_v1_imap_mark_read(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    CommunicationCommandService::new(pool)
        .mark_message_imap_read(&message_id)
        .await?;
    Ok(Json(serde_json::json!({"marked_read": true})))
}

pub(crate) async fn post_v1_imap_delete(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated = CommunicationCommandService::new(pool)
        .move_message_to_local_trash(&message_id, "imap_delete_alias", "imap-delete-alias")
        .await?;
    Ok(Json(serde_json::json!({
        "deleted": true,
        "provider_deleted": false,
        "local_state": updated.local_state.as_str()
    })))
}

pub(crate) async fn post_v1_message_trash(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated = CommunicationCommandService::new(pool)
        .move_message_to_local_trash(&message_id, "message_trash", "user_deleted")
        .await?;
    Ok(Json(serde_json::json!({
        "message_id": updated.message_id,
        "local_state": updated.local_state.as_str(),
        "provider_deleted": false
    })))
}

pub(crate) async fn post_v1_message_restore(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let updated = CommunicationCommandService::new(pool)
        .restore_message_from_local_trash(&message_id)
        .await?;
    Ok(Json(serde_json::json!({
        "message_id": updated.message_id,
        "local_state": updated.local_state.as_str()
    })))
}
```

### `backend/src/app/handlers/communications/sending/multilingual.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/multilingual.rs`
- Size bytes / Размер в байтах: `13756`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```rust
use super::super::*;

const MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS: usize = 64_000;
const ATTACHMENT_TRANSLATION_SOURCE: &str = "caller_provided_extracted_text";

pub(crate) async fn get_v1_detect_language(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<crate::domains::communications::multilingual::LanguageDetection>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    Ok(Json(
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            &msg.body_text,
        ),
    ))
}

#[derive(Deserialize)]
pub(crate) struct TranslateRequest {
    pub(super) target_language: String,
}

#[derive(Deserialize)]
pub(crate) struct TranslateAttachmentRequest {
    pub(super) target_language: String,
    pub(super) source_text: String,
}

#[derive(Deserialize)]
pub(crate) struct TranslateThreadQuery {
    pub(super) account_id: String,
    pub(super) subject: String,
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadTranslationResponse {
    pub(super) account_id: String,
    pub(super) subject: String,
    pub(super) target_language: String,
    pub(super) items: Vec<ThreadTranslationItem>,
}

#[derive(Serialize)]
pub(crate) struct ThreadTranslationItem {
    pub(super) message_id: String,
    pub(super) original_language: String,
    pub(super) confidence: f32,
    pub(super) translated: bool,
    pub(super) text: Option<String>,
    pub(super) target: String,
    pub(super) model: Option<String>,
    pub(super) reason: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct AttachmentTranslationResponse {
    pub(super) attachment_id: String,
    pub(super) message_id: String,
    pub(super) filename: Option<String>,
    pub(super) original_language: String,
    pub(super) confidence: f32,
    pub(super) translated: bool,
    pub(super) text: Option<String>,
    pub(super) target: String,
    pub(super) model: Option<String>,
    pub(super) reason: Option<String>,
    pub(super) source: &'static str,
}

pub(crate) async fn post_v1_translate(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<Value>, ApiError> {
    let store = message_store(&state)?;
    let msg = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let service = email_multilingual_service(&state).await?;
    let detection =
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            &msg.body_text,
        );
    match service
        .translate(&msg.body_text, &req.target_language)
        .await?
    {
        Some(t) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::dispatch_ai_helper_signal(
                    pool.clone(),
                    "message_translation",
                    &message_id,
                    serde_json::json!({
                        "kind": "communication_message",
                        "source_code": "ai",
                        "message_id": message_id,
                        "operation": "translation",
                    }),
                    serde_json::json!({
                        "target_language": t.target_language,
                        "original_language": detection.language,
                        "model": t.model,
                    }),
                    serde_json::json!({
                        "source": "communication_message_translation",
                        "message_id": message_id,
                    }),
                    None,
                )
                .await?;
            }

            Ok(Json(
                serde_json::json!({"translated": true, "text": t.translated_text, "target": t.target_language, "model": t.model}),
            ))
        }
        None => Ok(Json(
            serde_json::json!({"translated": false, "reason": "no LLM configured"}),
        )),
    }
}

pub(crate) async fn post_v1_translate_attachment(
    State(state): State<AppState>,
    Path(attachment_id): Path<String>,
    Json(req): Json<TranslateAttachmentRequest>,
) -> Result<Json<AttachmentTranslationResponse>, ApiError> {
    let target_language = req.target_language.trim();
    if target_language.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "target_language is required",
        ));
    }

    let source_text = req.source_text.trim();
    if source_text.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "source_text is required",
        ));
    }
    if source_text.chars().count() > MAX_ATTACHMENT_TRANSLATION_SOURCE_CHARS {
        return Err(ApiError::InvalidCommunicationQuery(
            "source_text exceeds maximum length",
        ));
    }

    let attachment = communication_storage_store(&state)?
        .attachment_by_id(&attachment_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let detection =
        crate::domains::communications::multilingual::MultilingualService::detect_language(
            source_text,
        );
    let service = email_multilingual_service(&state).await?;

    match service.translate(source_text, target_language).await {
        Ok(Some(translation)) => {
            if let Some(pool) = state.database.pool() {
                crate::domains::signal_hub::dispatch_ai_helper_signal(
                    pool.clone(),
                    "attachment_translation",
                    &attachment.attachment.attachment_id,
                    serde_json::json!({
                        "kind": "communication_attachment",
                        "source_code": "ai",
                        "attachment_id": attachment.attachment.attachment_id,
                        "message_id": attachment.attachment.message_id,
                        "operation": "attachment_translation",
                    }),
                    serde_json::json!({
                        "target_language": translation.target_language,
                        "original_language": detection.language,
                        "model": translation.model,
                        "source": ATTACHMENT_TRANSLATION_SOURCE,
                    }),
                    serde_json::json!({
                        "source": "communication_attachment_translation",
                        "attachment_id": attachment.attachment.attachment_id,
                        "message_id": attachment.attachment.message_id,
                    }),
                    None,
                )
                .await?;
            }

            Ok(Json(AttachmentTranslationResponse {
                attachment_id: attachment.attachment.attachment_id,
                message_id: attachment.attachment.message_id,
                filename: attachment.attachment.filename,
                original_language: detection.language,
                confidence: detection.confidence,
                translated: true,
                text: Some(translation.translated_text),
                target: translation.target_language,
                model: Some(translation.model),
                reason: None,
                source: ATTACHMENT_TRANSLATION_SOURCE,
            }))
        }
        Ok(None) => Ok(Json(AttachmentTranslationResponse {
            attachment_id: attachment.attachment.attachment_id,
            message_id: attachment.attachment.message_id,
            filename: attachment.attachment.filename,
            original_language: detection.language,
            confidence: detection.confidence,
            translated: false,
            text: None,
            target: target_language.to_owned(),
            model: None,
            reason: Some("no LLM configured".to_owned()),
            source: ATTACHMENT_TRANSLATION_SOURCE,
        })),
        Err(error) => {
            tracing::warn!(
                error = %error,
                attachment_id = %attachment.attachment.attachment_id,
                "attachment translation failed"
            );
            Ok(Json(AttachmentTranslationResponse {
                attachment_id: attachment.attachment.attachment_id,
                message_id: attachment.attachment.message_id,
                filename: attachment.attachment.filename,
                original_language: detection.language,
                confidence: detection.confidence,
                translated: false,
                text: None,
                target: target_language.to_owned(),
                model: None,
                reason: Some("translation runtime unavailable".to_owned()),
                source: ATTACHMENT_TRANSLATION_SOURCE,
            }))
        }
    }
}

pub(crate) async fn post_v1_translate_thread(
    State(state): State<AppState>,
    Query(query): Query<TranslateThreadQuery>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<ThreadTranslationResponse>, ApiError> {
    let account_id = query.account_id.trim();
    let subject = query.subject.trim();
    let target_language = req.target_language.trim();
    if account_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "account_id is required",
        ));
    }
    if subject.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery("subject is required"));
    }
    if target_language.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "target_language is required",
        ));
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let thread_store = crate::app::api_support::app_store::<
        crate::domains::communications::threads::CommunicationThreadStore,
    >(pool);
    let messages = thread_store
        .thread_messages(account_id, subject, query.limit.unwrap_or(50))
        .await?;
    let service = email_multilingual_service(&state).await?;
    let mut items = Vec::with_capacity(messages.len());

    for message in messages {
        let detection =
            crate::domains::communications::multilingual::MultilingualService::detect_language(
                &message.body_text,
            );
        match service.translate(&message.body_text, target_language).await {
            Ok(Some(translation)) => {
                if let Some(pool) = state.database.pool() {
                    crate::domains::signal_hub::dispatch_ai_helper_signal(
                        pool.clone(),
                        "thread_message_translation",
                        &message.message_id,
                        serde_json::json!({
                            "kind": "communication_message",
                            "source_code": "ai",
                            "message_id": message.message_id,
                            "operation": "thread_message_translation",
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        serde_json::json!({
                            "target_language": translation.target_language,
                            "original_language": detection.language,
                            "model": translation.model,
                        }),
                        serde_json::json!({
                            "source": "communication_thread_message_translation",
                            "message_id": message.message_id,
                            "account_id": account_id,
                            "thread_subject": subject,
                        }),
                        None,
                    )
                    .await?;
                }

                items.push(ThreadTranslationItem {
                    message_id: message.message_
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `backend/src/app/handlers/communications/sending/provider_send.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/sending/provider_send.rs`
- Size bytes / Размер в байтах: `2401`
- Included characters / Включено символов: `2401`
- Truncated / Обрезано: `no`

```rust
use super::super::*;
use crate::application::communication_send::{
    CommunicationSendDependencies, CommunicationSendError, CommunicationSendRequest, send_email,
};
use serde_json::json;

pub(crate) async fn post_v1_send(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    if req.confirmed_provider_write != Some(true) {
        return Err(ApiError::ProviderWriteConfirmationRequired);
    }

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deps = CommunicationSendDependencies::new(pool, api_audit_log(&state)?);
    let result = send_email(
        &deps,
        CommunicationSendRequest {
            account_id: req.account_id,
            to: req.to,
            cc: req.cc.unwrap_or_default(),
            bcc: req.bcc.unwrap_or_default(),
            subject: req.subject,
            body_text: req.body_text,
            body_html: req.body_html,
            in_reply_to: req.in_reply_to,
            references: req.references.unwrap_or_default(),
            draft_id: req.draft_id,
            scheduled_send_at: req.scheduled_send_at,
            undo_send_seconds: req.undo_send_seconds,
            metadata: json!({}),
        },
    )
    .await
    .map_err(communication_send_api_error)?;

    Ok(Json(SendResponse {
        message_id: result.message_id,
        outbox_id: result.outbox_id,
        accepted: result.accepted,
        accepted_recipients: result.accepted_recipients,
        transport: result.transport,
        status: result.status,
        scheduled_send_at: result.scheduled_send_at,
        undo_deadline_at: result.undo_deadline_at,
        failure_reason: result.failure_reason,
    }))
}

fn communication_send_api_error(error: CommunicationSendError) -> ApiError {
    match error {
        CommunicationSendError::InvalidRequest(message) => {
            ApiError::InvalidCommunicationQuery(message)
        }
        CommunicationSendError::ProviderAccountNotFound => {
            ApiError::InvalidCommunicationQuery("provider account was not found")
        }
        CommunicationSendError::CommunicationIngestion(inner) => ApiError::from(inner),
        CommunicationSendError::Command(inner) => ApiError::from(inner),
        CommunicationSendError::Audit(inner) => ApiError::from(inner),
    }
}
```

### `backend/src/app/handlers/communications/templates_status.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/templates_status.rs`
- Size bytes / Размер в байтах: `8367`
- Included characters / Включено символов: `8367`
- Truncated / Обрезано: `no`

```rust
use super::*;
use crate::domains::communications::templates::{
    CommunicationMergePreviewRow, CommunicationTemplateStore, NewCommunicationTemplate,
};

const MAX_MAIL_MERGE_PREVIEW_ROWS: usize = 250;

#[derive(Deserialize)]
pub(crate) struct RenderTemplateRequest {
    pub(super) template_id: String,
    pub(super) variables: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub(crate) struct MailMergePreviewRequest {
    pub(super) template_id: String,
    pub(super) rows: Vec<MailMergePreviewRowRequest>,
}

#[derive(Deserialize)]
pub(crate) struct MailMergePreviewRowRequest {
    pub(super) row_id: String,
    pub(super) variables: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub(crate) struct UpsertTemplateRequest {
    pub(super) template_id: Option<String>,
    pub(super) name: String,
    pub(super) subject_template: Option<String>,
    pub(super) body_template: Option<String>,
    pub(super) content: Option<String>,
    pub(super) variables: Option<Vec<String>>,
    pub(super) language: Option<String>,
}

pub(crate) async fn get_v1_rich_templates(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let templates = crate::app::api_support::app_store::<CommunicationTemplateStore>(pool.clone())
        .list()
        .await?;
    Ok(Json(serde_json::json!({ "templates": templates })))
}

pub(crate) async fn post_v1_rich_template(
    State(state): State<AppState>,
    Json(req): Json<UpsertTemplateRequest>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let template = crate::app::api_support::app_store::<CommunicationTemplateStore>(pool.clone())
        .upsert(&NewCommunicationTemplate {
            template_id: req
                .template_id
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| {
                    let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
                    format!("mail_template:{timestamp}")
                }),
            name: req.name,
            subject_template: req
                .subject_template
                .or_else(|| req.content.clone())
                .unwrap_or_else(|| "Untitled template".to_owned()),
            body_template: req.body_template.or(req.content).unwrap_or_default(),
            variables: req.variables.unwrap_or_default(),
            language: req.language,
        })
        .await?;
    Ok(Json(
        serde_json::json!({ "saved": true, "template": template }),
    ))
}

pub(crate) async fn delete_v1_rich_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let template_id = template_id.trim();
    if template_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "template_id is required",
        ));
    }
    let deleted = crate::app::api_support::app_store::<CommunicationTemplateStore>(pool.clone())
        .delete(template_id)
        .await?;
    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({
        "template_id": template_id,
        "deleted": true
    })))
}

pub(crate) async fn get_v1_blockers()
-> Result<Json<Vec<crate::domains::communications::blockers::ArchitectureBlocker>>, ApiError> {
    Ok(Json(
        crate::domains::communications::blockers::list_blockers(),
    ))
}

pub(crate) async fn post_v1_render_template(
    State(state): State<AppState>,
    Json(req): Json<RenderTemplateRequest>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let store = crate::app::api_support::app_store::<CommunicationTemplateStore>(pool.clone());
    let template_id = req.template_id.trim();
    let Some(template) = store.get(template_id).await? else {
        return Err(ApiError::NotFound);
    };
    let vars = req.variables.unwrap_or_default();
    let rendered = store.render(&template, &vars)?;
    Ok(Json(serde_json::json!({
        "template_id": template.template_id,
        "variables": vars,
        "rendered": rendered
    })))
}

pub(crate) async fn post_v1_rich_template_mail_merge_preview(
    State(state): State<AppState>,
    Json(req): Json<MailMergePreviewRequest>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let template_id = req.template_id.trim();
    if template_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "template_id is required",
        ));
    }
    if req.rows.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "mail merge preview rows are required",
        ));
    }
    if req.rows.len() > MAX_MAIL_MERGE_PREVIEW_ROWS {
        return Err(ApiError::InvalidCommunicationQuery(
            "mail merge preview row limit exceeded",
        ));
    }
    let rows = req
        .rows
        .into_iter()
        .map(|row| {
            let row_id = row.row_id.trim().to_owned();
            if row_id.is_empty() {
                return Err(ApiError::InvalidCommunicationQuery("row_id is required"));
            }
            Ok(CommunicationMergePreviewRow {
                row_id,
                variables: row.variables.unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    let store = crate::app::api_support::app_store::<CommunicationTemplateStore>(pool.clone());
    let Some(template) = store.get(template_id).await? else {
        return Err(ApiError::NotFound);
    };
    let preview = store.render_mail_merge_preview(&template, rows)?;
    Ok(Json(serde_json::to_value(preview).map_err(|_| {
        ApiError::InvalidCommunicationQuery("mail merge preview response failed")
    })?))
}

#[derive(Deserialize)]
pub(crate) struct PersonListQuery {
    pub(super) favorites_only: Option<bool>,
    pub(super) limit: Option<i64>,
}

pub(crate) async fn get_v1_status(
    State(state): State<AppState>,
) -> Result<Json<V1StatusResponse>, ApiError> {
    let Some(_pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(Json(V1StatusResponse {
        version: "1.0",
        surfaces: V1Surfaces {
            messages: true,
            persons: true,
            search: true,
            documents: true,
            account_setup: true,
        },
        vault_status: state.vault.status()?,
    }))
}

pub(crate) async fn get_v1_vault_status(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.status()?))
}

#[derive(Deserialize)]
pub(crate) struct VaultEntropyBatchRequest {
    pub(super) events: Vec<EntropyEvent>,
}

pub(crate) async fn post_v1_vault_collect_entropy(
    State(state): State<AppState>,
    Json(request): Json<VaultEntropyBatchRequest>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.collect_entropy(request.events)?))
}

pub(crate) async fn post_v1_vault_create(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.create()?))
}

pub(crate) async fn post_v1_vault_unlock(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.unlock()?))
}

pub(crate) async fn post_v1_vault_recovery_export(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::RecoveryExportResponse>, ApiError> {
    Ok(Json(state.vault.export_recovery()?))
}

#[derive(Deserialize)]
pub(crate) struct VaultRecoveryImportRequest {
    pub(super) recovery_phrase: String,
}

pub(crate) async fn post_v1_vault_recovery_import(
    State(state): State<AppState>,
    Json(request): Json<VaultRecoveryImportRequest>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.import_recovery(&request.recovery_phrase)?))
}
```

### `backend/src/app/handlers/communications/workflow_actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions.rs`
- Size bytes / Размер в байтах: `440`
- Included characters / Включено символов: `440`
- Truncated / Обрезано: `no`

```rust
mod actions;
mod constants;
mod handler;
mod models;
mod response;
mod source;
mod validation;

pub(crate) use handler::execute_workflow_action;
pub(crate) use handler::post_v1_workflow_action;
pub(crate) use models::{
    WorkflowActionInput, WorkflowActionKind, WorkflowActionProvenance, WorkflowActionRequest,
    WorkflowActionResponse, WorkflowActionSource, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
```

### `backend/src/app/handlers/communications/workflow_actions/actions.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/actions.rs`
- Size bytes / Размер в байтах: `377`
- Included characters / Включено символов: `377`
- Truncated / Обрезано: `no`

```rust
mod archive;
mod calendar;
mod documents;
mod persons;
mod reply;
mod tasks;

pub(super) use archive::archive_response;
pub(super) use calendar::create_event_response;
pub(super) use documents::{create_document_response, link_document_response};
pub(super) use persons::create_contact_response;
pub(super) use reply::reply_response;
pub(super) use tasks::create_task_response;
```

### `backend/src/app/handlers/communications/workflow_actions/actions/archive.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/actions/archive.rs`
- Size bytes / Размер в байтах: `1853`
- Included characters / Включено символов: `1853`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::communications::messages::{
    MessageProjectionStore, ProjectedMessage, WorkflowState,
};

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::require_source_message;

pub(in crate::app::handlers::communications::workflow_actions) async fn archive_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let message = require_source_message(request, message)?;
    let updated = if message.workflow_state == WorkflowState::Archived {
        message.clone()
    } else {
        if !WorkflowState::is_valid_transition(&message.workflow_state, &WorkflowState::Archived) {
            return Err(ApiError::InvalidCommunicationQuery(
                "invalid workflow state transition",
            ));
        }
        MessageProjectionStore::transition_workflow_state_in_transaction(
            transaction,
            &message.message_id,
            WorkflowState::Archived,
        )
        .await?
    };
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        if updated.workflow_state == WorkflowState::Archived {
            WorkflowActionStatus::Archived
        } else {
            WorkflowActionStatus::Noop
        },
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Message,
            id: Some(updated.message_id),
        },
        Some(message),
        vec!["message workflow state transitioned locally".to_owned()],
    ))
}
```

### `backend/src/app/handlers/communications/workflow_actions/actions/calendar.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/actions/calendar.rs`
- Size bytes / Размер в байтах: `2906`
- Included characters / Включено символов: `2906`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use crate::domains::communications::messages::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::input_title;

pub(in crate::app::handlers::communications::workflow_actions) async fn create_event_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let input = request
        .input
        .as_ref()
        .ok_or(ApiError::InvalidCommunicationQuery(
            "create_event requires input",
        ))?;
    let start_at = input.starts_at.ok_or(ApiError::InvalidCommunicationQuery(
        "create_event requires starts_at",
    ))?;
    let end_at = input.ends_at.ok_or(ApiError::InvalidCommunicationQuery(
        "create_event requires ends_at",
    ))?;
    if end_at <= start_at {
        return Err(ApiError::InvalidCommunicationQuery(
            "create_event requires ends_at after starts_at",
        ));
    }
    let title = input_title(request, message, "New event")?;
    let event = CalendarEventStore::create_manual_with_observation_in_transaction(
        transaction,
        &NewCalendarEvent {
            source_event_id: Some(event_id.to_owned()),
            account_id: None,
            source_id: None,
            title,
            description: input.body.clone(),
            location: None,
            start_at,
            end_at,
            timezone: None,
            all_day: Some(false),
            recurrence_rule: None,
            status: Some("scheduled".to_owned()),
            visibility: Some("private".to_owned()),
            event_type: Some("meeting".to_owned()),
            conference_url: None,
            conference_provider: None,
            preparation_reminder_minutes: None,
            travel_buffer_minutes: None,
        },
        "mail.workflow_actions.create_event_response",
        message.map(|value| value.observation_id.as_str()),
        message.map(|value| {
            serde_json::json!({
                "workflow_action": "create_event",
                "message_id": value.message_id,
            })
        }),
    )
    .await?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::CalendarEvent,
            id: Some(event.event_id),
        },
        message,
        vec!["local calendar event created through workflow action".to_owned()],
    ))
}
```

### `backend/src/app/handlers/communications/workflow_actions/actions/documents.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/actions/documents.rs`
- Size bytes / Размер в байтах: `5246`
- Included characters / Включено символов: `5246`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::domains::communications::messages::ProjectedMessage;
use crate::domains::documents::core::{DocumentImportStore, NewDocumentImport};

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::{input_title, normalize_non_empty, require_source_message};

pub(in crate::app::handlers::communications::workflow_actions) async fn create_document_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
    note_mode: bool,
) -> Result<WorkflowActionResponse, ApiError> {
    let title = input_title(
        request,
        message,
        if note_mode {
            "New note"
        } else {
            "New document"
        },
    )?;
    let input = request.input.as_ref();
    let document_id = input
        .and_then(|value| value.document_id.as_ref())
        .map(|value| normalize_non_empty("document_id", value))
        .transpose()?
        .unwrap_or_else(|| {
            let prefix = if note_mode {
                "document:workflow-note"
            } else {
                "document:workflow"
            };
            format!("{prefix}:{command_id}")
        });
    let body = input
        .and_then(|value| value.body.as_ref())
        .map(String::as_str)
        .or_else(|| message.map(|value| value.body_text.as_str()))
        .unwrap_or(&title);
    let markdown = format!("# {title}\n\n{body}");
    let document = DocumentImportStore::import_document_manual_with_observation_in_transaction(
        transaction,
        &NewDocumentImport::markdown(document_id, title, markdown),
        format!("workflow-action://document/{command_id}"),
        serde_json::json!({
            "captured_by": "mail.workflow_actions.create_document_response",
            "workflow_action": if note_mode { "create_note" } else { "create_document" },
            "event_id": event_id,
        }),
        message.map(|value| value.observation_id.as_str()),
        Some("workflow_action_projection"),
        message.map(|value| {
            serde_json::json!({
                "workflow_action": if note_mode { "create_note" } else { "create_document" },
                "message_id": value.message_id,
            })
        }),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "workflow document import failed");
        ApiError::InvalidCommunicationQuery("workflow document import failed")
    })?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Document,
            id: Some(document.document_id),
        },
        message,
        vec!["markdown document imported through documents boundary".to_owned()],
    ))
}

pub(in crate::app::handlers::communications::workflow_actions) async fn link_document_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let message = require_source_message(request, message)?;
    let title = input_title(request, Some(message), "Linked communication document")?;
    let document_id = request
        .input
        .as_ref()
        .and_then(|value| value.document_id.as_ref())
        .map(|value| normalize_non_empty("document_id", value))
        .transpose()?
        .unwrap_or_else(|| format!("document:mail-message:{}", message.message_id));
    let markdown = format!("# {title}\n\n{}", message.body_text);
    let document = DocumentImportStore::import_document_manual_with_observation_in_transaction(
        transaction,
        &NewDocumentImport::markdown(document_id, title, markdown),
        format!("workflow-action://link-document/{command_id}"),
        serde_json::json!({
            "captured_by": "mail.workflow_actions.link_document_response",
            "workflow_action": "link_document",
            "event_id": event_id,
            "source_message_id": message.message_id,
        }),
        Some(message.observation_id.as_str()),
        Some("workflow_action_projection"),
        Some(serde_json::json!({
            "workflow_action": "link_document",
            "message_id": message.message_id,
        })),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "workflow link document import failed");
        ApiError::InvalidCommunicationQuery("workflow document import failed")
    })?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Linked,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Document,
            id: Some(document.document_id),
        },
        Some(message),
        vec!["message-to-document relation recorded in workflow event payload".to_owned()],
    ))
}
```

### `backend/src/app/handlers/communications/workflow_actions/actions/persons.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/actions/persons.rs`
- Size bytes / Размер в байтах: `1982`
- Included characters / Включено символов: `1982`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::application::workflow_action_person_projection::create_person_projection_in_transaction;
use crate::domains::communications::messages::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;

pub(in crate::app::handlers::communications::workflow_actions) async fn create_contact_response(
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let email = request
        .input
        .as_ref()
        .and_then(|value| value.email.as_ref())
        .map(String::as_str)
        .or_else(|| message.map(|value| value.sender.as_str()))
        .ok_or(ApiError::InvalidCommunicationQuery(
            "create_contact requires email or source message",
        ))?;
    let display_name = request
        .input
        .as_ref()
        .and_then(|value| value.display_name.as_ref())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let person_id = create_person_projection_in_transaction(
        transaction,
        command_id,
        event_id,
        email,
        display_name,
        message,
    )
    .await?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Person,
            id: Some(person_id),
        },
        message,
        vec![
            display_name
                .map(|value| format!("person upserted from communication identity for {value}"))
                .unwrap_or_else(|| "person upserted from communication identity".to_owned()),
        ],
    ))
}
```

### `backend/src/app/handlers/communications/workflow_actions/actions/reply.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/actions/reply.rs`
- Size bytes / Размер в байтах: `1047`
- Included characters / Включено символов: `1047`
- Truncated / Обрезано: `no`

```rust
use crate::app::ApiError;
use crate::domains::communications::messages::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::require_source_message;

pub(in crate::app::handlers::communications::workflow_actions) fn reply_response(
    command_id: &str,
    event_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let message = require_source_message(request, message)?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Opened,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Compose,
            id: Some(message.message_id.clone()),
        },
        Some(message),
        vec!["reply compose opened from selected communication message".to_owned()],
    ))
}
```

### `backend/src/app/handlers/communications/workflow_actions/actions/tasks.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/actions/tasks.rs`
- Size bytes / Размер в байтах: `2671`
- Included characters / Включено символов: `2671`
- Truncated / Обрезано: `no`

```rust
use sqlx::{Postgres, Transaction};

use crate::app::ApiError;
use crate::application::task_creation::{WorkflowTaskCreateInput, create_task_from_workflow_input};
use crate::domains::communications::messages::ProjectedMessage;

use super::super::models::{
    WorkflowActionRequest, WorkflowActionResponse, WorkflowActionStatus, WorkflowActionTarget,
    WorkflowActionTargetKind,
};
use super::super::response::base_response;
use super::super::validation::input_title;
use sqlx::PgPool;

pub(in crate::app::handlers::communications::workflow_actions) async fn create_task_response(
    pool: &PgPool,
    transaction: &mut Transaction<'_, Postgres>,
    command_id: &str,
    event_id: &str,
    actor_id: &str,
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
) -> Result<WorkflowActionResponse, ApiError> {
    let title = input_title(request, message, "New task")?;
    let input = request.input.as_ref();
    let task = create_task_from_workflow_input(
        pool,
        transaction,
        WorkflowTaskCreateInput {
            title,
            description: input.and_then(|value| value.body.clone()),
            provenance_kind: message.map(|_| "observation".to_owned()),
            provenance_id: message.map(|value| value.observation_id.clone()),
            source_kind: if message.is_some() {
                "observation".to_owned()
            } else {
                "manual".to_owned()
            },
            source_id: message
                .map(|value| value.observation_id.clone())
                .unwrap_or_else(|| command_id.to_owned()),
            source_type: message
                .map(|_| "observation")
                .unwrap_or("manual")
                .to_owned(),
            due_at: input.and_then(|value| value.due_at),
            created_from_event_id: event_id.to_owned(),
            created_by_actor_id: actor_id.to_owned(),
            projection_observation_id: message.map(|value| value.observation_id.clone()),
            projection_metadata: message.map(|value| {
                serde_json::json!({
                    "workflow_action": "create_task",
                    "message_id": value.message_id,
                    "created_from_event_id": event_id,
                })
            }),
        },
    )
    .await?;
    Ok(base_response(
        command_id,
        event_id,
        request.action.clone(),
        WorkflowActionStatus::Created,
        WorkflowActionTarget {
            kind: WorkflowActionTargetKind::Task,
            id: Some(task.task_id),
        },
        message,
        vec!["task created through local workflow action".to_owned()],
    ))
}
```

### `backend/src/app/handlers/communications/workflow_actions/constants.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/constants.rs`
- Size bytes / Размер в байтах: `73`
- Included characters / Включено символов: `73`
- Truncated / Обрезано: `no`

```rust
pub(super) const WORKFLOW_EVENT_TYPE: &str = "workflow.action_executed";
```

### `backend/src/app/handlers/communications/workflow_actions/handler.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/handler.rs`
- Size bytes / Размер в байтах: `5867`
- Included characters / Включено символов: `5867`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use chrono::Utc;
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::domains::communications::messages::MessageProjectionStore;
use crate::platform::events::{EventStore, NewEventEnvelope};

use super::actions::{
    archive_response, create_contact_response, create_document_response, create_event_response,
    create_task_response, link_document_response, reply_response,
};
use super::constants::WORKFLOW_EVENT_TYPE;
use super::models::{WorkflowActionKind, WorkflowActionRequest, WorkflowActionResponse};
use super::response::response_from_event;
use super::source::load_source_message;
use super::validation::{actor_id_from_headers, normalize_non_empty};

pub(crate) async fn post_v1_workflow_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<WorkflowActionRequest>,
) -> Result<Json<WorkflowActionResponse>, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let actor_id = actor_id_from_headers(&headers);
    let response = execute_workflow_action(&pool, &actor_id, request).await?;
    Ok(Json(response))
}

pub(crate) async fn execute_workflow_action(
    pool: &sqlx::postgres::PgPool,
    actor_id: &str,
    request: WorkflowActionRequest,
) -> Result<WorkflowActionResponse, ApiError> {
    let command_id = normalize_non_empty("command_id", &request.command_id)?;
    let event_id = format!("workflow_action:{command_id}");
    let event_store = crate::app::api_support::app_store::<EventStore>(pool.clone());
    if let Some(existing) = event_store.get_by_id(&event_id).await? {
        return response_from_event(existing);
    }

    let message_store = crate::app::api_support::app_store::<MessageProjectionStore>(pool.clone());
    let source_message = load_source_message(&message_store, request.source.as_ref()).await?;
    let mut transaction = pool
        .begin()
        .await
        .map_err(|error| ApiError::Store(error.into()))?;
    let response = match request.action.clone() {
        WorkflowActionKind::Reply => {
            reply_response(&command_id, &event_id, &request, source_message.as_ref())?
        }
        WorkflowActionKind::CreateTask => {
            create_task_response(
                pool,
                &mut transaction,
                &command_id,
                &event_id,
                actor_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::CreateNote => {
            create_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
                true,
            )
            .await?
        }
        WorkflowActionKind::CreateDocument => {
            create_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
                false,
            )
            .await?
        }
        WorkflowActionKind::CreateEvent => {
            create_event_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::LinkDocument => {
            link_document_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::CreateContact => {
            create_contact_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
        WorkflowActionKind::Archive => {
            archive_response(
                &mut transaction,
                &command_id,
                &event_id,
                &request,
                source_message.as_ref(),
            )
            .await?
        }
    };

    let event = NewEventEnvelope::builder(
        event_id.clone(),
        WORKFLOW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "workflow_action",
            "source_id": command_id,
        }),
        json!({
            "kind": "workflow_action",
            "id": command_id,
        }),
    )
    .actor(json!({ "actor_id": actor_id }))
    .payload(serde_json::to_value(&response).map_err(|_| {
        ApiError::InvalidCommunicationQuery("invalid workflow action response payload")
    })?)
    .provenance(json!({
        "source_kind": response.provenance.source_kind.as_deref(),
        "source_id": response.provenance.source_id.as_deref(),
        "confidence": response.provenance.confidence,
        "evidence": response.provenance.evidence.clone(),
    }))
    .correlation_id(command_id.clone())
    .build()
    .map_err(ApiError::InvalidEnvelope)?;

    match EventStore::append_in_transaction(&mut transaction, &event).await {
        Ok(_) => {
            transaction
                .commit()
                .await
                .map_err(|error| ApiError::Store(error.into()))?;
            Ok(response)
        }
        Err(error) if error.is_unique_violation() => {
            let _ = transaction.rollback().await;
            let Some(existing) = event_store.get_by_id(&event_id).await? else {
                return Err(ApiError::Store(error));
            };
            Ok(response_from_event(existing)?)
        }
        Err(error) => Err(ApiError::Store(error)),
    }
}
```

### `backend/src/app/handlers/communications/workflow_actions/models.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/models.rs`
- Size bytes / Размер в байтах: `2455`
- Included characters / Включено символов: `2455`
- Truncated / Обрезано: `no`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionKind {
    Reply,
    CreateTask,
    CreateNote,
    CreateDocument,
    CreateEvent,
    LinkDocument,
    CreateContact,
    Archive,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkflowActionSource {
    pub(crate) kind: String,
    pub(crate) id: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct WorkflowActionInput {
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) starts_at: Option<DateTime<Utc>>,
    pub(crate) ends_at: Option<DateTime<Utc>>,
    pub(crate) due_at: Option<DateTime<Utc>>,
    pub(crate) document_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkflowActionRequest {
    pub(crate) command_id: String,
    pub(crate) action: WorkflowActionKind,
    pub(crate) source: Option<WorkflowActionSource>,
    pub(crate) input: Option<WorkflowActionInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionStatus {
    Created,
    Updated,
    Linked,
    Opened,
    Archived,
    Noop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowActionTargetKind {
    Compose,
    Message,
    Task,
    Document,
    CalendarEvent,
    Person,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionTarget {
    pub(crate) kind: WorkflowActionTargetKind,
    pub(crate) id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_id: Option<String>,
    pub(crate) confidence: Option<f64>,
    pub(crate) evidence: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct WorkflowActionResponse {
    pub(crate) command_id: String,
    pub(crate) event_id: String,
    pub(crate) action: WorkflowActionKind,
    pub(crate) status: WorkflowActionStatus,
    pub(crate) target: WorkflowActionTarget,
    pub(crate) provenance: WorkflowActionProvenance,
}
```

### `backend/src/app/handlers/communications/workflow_actions/response.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/response.rs`
- Size bytes / Размер в байтах: `1420`
- Included characters / Включено символов: `1420`
- Truncated / Обрезано: `no`

```rust
use crate::app::ApiError;
use crate::domains::communications::messages::ProjectedMessage;
use crate::platform::events::EventEnvelope;

use super::models::{
    WorkflowActionKind, WorkflowActionProvenance, WorkflowActionResponse, WorkflowActionStatus,
    WorkflowActionTarget,
};

pub(super) fn base_response(
    command_id: &str,
    event_id: &str,
    action: WorkflowActionKind,
    status: WorkflowActionStatus,
    target: WorkflowActionTarget,
    message: Option<&ProjectedMessage>,
    evidence: Vec<String>,
) -> WorkflowActionResponse {
    WorkflowActionResponse {
        command_id: command_id.to_owned(),
        event_id: event_id.to_owned(),
        action,
        status,
        target,
        provenance: WorkflowActionProvenance {
            source_kind: message.map(|_| "communication_message".to_owned()),
            source_id: message.map(|value| value.message_id.clone()),
            confidence: None,
            evidence,
        },
    }
}

pub(super) fn response_from_event(
    event: EventEnvelope,
) -> Result<WorkflowActionResponse, ApiError> {
    let event_id = event.event_id.clone();
    serde_json::from_value::<WorkflowActionResponse>(event.payload).map_err(|error| {
        tracing::error!(error = %error, event_id = %event_id, "stored workflow action payload is invalid");
        ApiError::InvalidCommunicationQuery("stored workflow action payload is invalid")
    })
}
```

### `backend/src/app/handlers/communications/workflow_actions/source.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/source.rs`
- Size bytes / Размер в байтах: `854`
- Included characters / Включено символов: `854`
- Truncated / Обрезано: `no`

```rust
use crate::app::ApiError;
use crate::domains::communications::messages::{MessageProjectionStore, ProjectedMessage};

use super::models::WorkflowActionSource;
use super::validation::normalize_non_empty;

pub(super) async fn load_source_message(
    store: &MessageProjectionStore,
    source: Option<&WorkflowActionSource>,
) -> Result<Option<ProjectedMessage>, ApiError> {
    let Some(source) = source else {
        return Ok(None);
    };
    if source.kind != "communication_message" {
        return Err(ApiError::InvalidCommunicationQuery(
            "workflow action source kind must be communication_message",
        ));
    }
    let source_id = normalize_non_empty("source.id", &source.id)?;
    Ok(Some(
        store
            .message(&source_id)
            .await?
            .ok_or(ApiError::CommunicationMessageNotFound)?,
    ))
}
```

### `backend/src/app/handlers/communications/workflow_actions/validation.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_actions/validation.rs`
- Size bytes / Размер в байтах: `1889`
- Included characters / Включено символов: `1889`
- Truncated / Обрезано: `no`

```rust
use axum::http::HeaderMap;

use crate::app::ApiError;
use crate::domains::communications::messages::ProjectedMessage;

use super::models::WorkflowActionRequest;

pub(super) fn require_source_message<'a>(
    request: &WorkflowActionRequest,
    message: Option<&'a ProjectedMessage>,
) -> Result<&'a ProjectedMessage, ApiError> {
    if request.source.is_none() {
        return Err(ApiError::InvalidCommunicationQuery(
            "workflow action requires source message",
        ));
    }
    message.ok_or(ApiError::CommunicationMessageNotFound)
}

pub(super) fn input_title(
    request: &WorkflowActionRequest,
    message: Option<&ProjectedMessage>,
    fallback: &str,
) -> Result<String, ApiError> {
    if let Some(title) = request
        .input
        .as_ref()
        .and_then(|value| value.title.as_ref())
    {
        return normalize_non_empty("title", title);
    }
    if let Some(message) = message {
        return normalize_non_empty("title", &message.subject);
    }
    Ok(fallback.to_owned())
}

pub(super) fn normalize_non_empty(field: &'static str, value: &str) -> Result<String, ApiError> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(match field {
            "command_id" => "command_id must not be empty",
            "source.id" => "source id must not be empty",
            "document_id" => "document_id must not be empty",
            "title" => "title must not be empty",
            _ => "workflow action field must not be empty",
        }));
    }
    Ok(normalized)
}

pub(super) fn actor_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-makosh-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("makosh-frontend")
        .to_owned()
}
```

### `backend/src/app/handlers/communications/workflow_state.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/communications/workflow_state.rs`
- Size bytes / Размер в байтах: `7646`
- Included characters / Включено символов: `7646`
- Truncated / Обрезано: `no`

```rust
use super::*;
use crate::application::review_inbox::refresh_message_knowledge_candidates_into_review;
use crate::domains::communications::ai_state::{
    CommunicationAiState, CommunicationAiStateStore, CommunicationAiStateTransitionRequest,
};
use crate::domains::communications::service::CommunicationCommandService;

#[derive(Deserialize)]
pub(crate) struct WorkflowStateTransitionApiRequest {
    pub(super) workflow_state: String,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateTransitionApiResponse {
    pub(super) message_id: String,
    pub(super) workflow_state: String,
    pub(super) previous_state: String,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateCountsApiResponse {
    pub(super) counts: Vec<WorkflowStateCountApiItem>,
}

#[derive(Serialize)]
pub(crate) struct WorkflowStateCountApiItem {
    pub(super) state: String,
    pub(super) count: i64,
}

#[derive(Deserialize)]
pub(crate) struct WorkflowStateCountsQuery {
    pub(super) account_id: Option<String>,
    pub(super) local_state: Option<String>,
}
pub(crate) async fn put_v1_message_workflow_state(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(request): Json<WorkflowStateTransitionApiRequest>,
) -> Result<Json<WorkflowStateTransitionApiResponse>, ApiError> {
    let actor_id = "makosh-frontend".to_string();

    let new_state = request
        .workflow_state
        .parse::<WorkflowState>()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid workflow state value"))?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::message_workflow_state_set(
            &actor_id,
            &message_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let result = CommunicationCommandService::new(pool)
        .transition_message_workflow_state(&message_id, new_state, &actor_id)
        .await?;

    Ok(Json(WorkflowStateTransitionApiResponse {
        message_id: result.updated.message_id,
        workflow_state: result.updated.workflow_state.as_str().to_owned(),
        previous_state: result.previous_state,
    }))
}

pub(crate) async fn get_v1_message_workflow_state_counts(
    State(state): State<AppState>,
    Query(query): Query<WorkflowStateCountsQuery>,
) -> Result<Json<WorkflowStateCountsApiResponse>, ApiError> {
    let local_state = query
        .local_state
        .as_deref()
        .unwrap_or("active")
        .parse::<LocalMessageState>()
        .map_err(|_| ApiError::InvalidCommunicationQuery("invalid local_state value"))?;
    let counts = message_store(&state)?
        .count_messages_by_state_with_local_state(query.account_id.as_deref(), local_state)
        .await?
        .into_iter()
        .map(|c| WorkflowStateCountApiItem {
            state: c.state.as_str().to_owned(),
            count: c.count,
        })
        .collect();

    Ok(Json(WorkflowStateCountsApiResponse { counts }))
}

#[derive(Serialize)]
pub(crate) struct MessageAnalyzeResponse {
    pub(super) message_id: String,
    pub(super) analyzed: bool,
    pub(super) category: Option<String>,
    pub(super) summary: Option<String>,
    pub(super) summary_contract: EmailSummaryContract,
    pub(super) importance_score: Option<i16>,
    pub(super) workflow_state: String,
    pub(super) source: String,
    pub(super) confidence: Option<f64>,
    pub(super) evidence: Vec<String>,
}

pub(crate) async fn post_v1_message_analyze(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<MessageAnalyzeResponse>, ApiError> {
    let store = message_store(&state)?;
    let ai_state_store = crate::app::api_support::app_store::<CommunicationAiStateStore>(
        state
            .database
            .pool()
            .ok_or(ApiError::DatabaseNotConfigured)?
            .clone(),
    );

    let message = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;

    // Mark analysis as processing to reflect runtime activity for UI/state consumers.
    let _ = ai_state_store
        .transition(
            &message_id,
            CommunicationAiStateTransitionRequest {
                ai_state: CommunicationAiState::Processing,
                review_reason: None,
                last_error: None,
            },
        )
        .await?;

    // Always run heuristics (fast, no external dependency)
    let heuristic_score = EmailIntelligenceService::heuristic_score(&message);
    let heuristic_category = EmailIntelligenceService::heuristic_category(&message);
    let summary_contract = EmailIntelligenceService::heuristic_structured_summary(&message);

    store
        .set_ai_analysis(
            &message_id,
            heuristic_category.as_deref(),
            None,
            Some(heuristic_score),
        )
        .await?;
    let mut metadata = message.message_metadata.clone();
    metadata["ai_summary_contract"] = serde_json::to_value(&summary_contract).map_err(|_| {
        ApiError::InvalidCommunicationQuery("summary contract serialization failed")
    })?;
    store.set_message_metadata(&message_id, &metadata).await?;

    // If score is high, auto-transition to needs_action
    if heuristic_score >= 75 && message.workflow_state.as_str() == "new" {
        let _ = store
            .transition_workflow_state(&message_id, WorkflowState::NeedsAction)
            .await;
    }

    let _ = ai_state_store
        .transition(
            &message_id,
            CommunicationAiStateTransitionRequest {
                ai_state: CommunicationAiState::Processed,
                review_reason: None,
                last_error: None,
            },
        )
        .await?;

    let updated = store
        .message(&message_id)
        .await?
        .ok_or(ApiError::CommunicationMessageNotFound)?;
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let _ = refresh_message_knowledge_candidates_into_review(&pool, std::slice::from_ref(&updated))
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "message knowledge candidate review sync failed");
            ApiError::InvalidCommunicationQuery("message knowledge candidate review sync failed")
        })?;
    let evidence = crate::domains::communications::explain::explain_importance(&updated).reasons;

    Ok(Json(MessageAnalyzeResponse {
        message_id: updated.message_id,
        analyzed: true,
        category: updated.ai_category,
        summary: updated.ai_summary,
        summary_contract,
        importance_score: updated.importance_score,
        workflow_state: updated.workflow_state.as_str().to_owned(),
        source: "local_heuristic".to_owned(),
        confidence: None,
        evidence,
    }))
}

#[derive(Deserialize)]
pub(crate) struct ThreadListQuery {
    pub(super) account_id: Option<String>,
    pub(super) cursor: Option<String>,
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadListResponse {
    pub(super) items: Vec<crate::domains::communications::threads::CommunicationThread>,
    pub(super) next_cursor: Option<String>,
    pub(super) has_more: bool,
}

#[derive(Deserialize)]
pub(crate) struct ThreadMessagesQuery {
    pub(super) account_id: Option<String>,
    pub(super) subject: Option<String>,
    pub(super) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct ThreadMessagesResponse {
    pub(super) items: Vec<crate::domains::communications::threads::ThreadMessage>,
}
```

### `backend/src/app/handlers/consistency.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/consistency.rs`
- Size bytes / Размер в байтах: `3491`
- Included characters / Включено символов: `3491`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::app::{ApiError, AppState};
use crate::application::consistency_review::ContradictionReviewService;
use crate::engines::consistency::{
    ContradictionObservation, ContradictionObservationStore, ContradictionReviewState,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};

const CONTRADICTION_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_CONTRADICTION_LIMIT: i64 = 50;
const MIN_CONTRADICTION_LIMIT: i64 = 1;
const MAX_CONTRADICTION_LIMIT: i64 = 100;

#[derive(Debug, Deserialize)]
pub(crate) struct ContradictionListQuery {
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContradictionReviewApiRequest {
    review_state: String,
    resolution: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ContradictionListResponse {
    items: Vec<ContradictionObservation>,
}

pub(crate) async fn get_v1_contradictions(
    State(state): State<AppState>,
    Query(query): Query<ContradictionListQuery>,
) -> Result<Json<ContradictionListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let items = contradiction_store(&state)?.list_open(limit).await?;

    Ok(Json(ContradictionListResponse { items }))
}

pub(crate) async fn put_v1_contradiction_review(
    State(state): State<AppState>,
    Path(observation_id): Path<String>,
    Json(request): Json<ContradictionReviewApiRequest>,
) -> Result<Json<ContradictionObservation>, ApiError> {
    let observation_id = validate_required_value(&observation_id)?;
    let review_state = ContradictionReviewState::parse(&request.review_state)?;
    let resolution = request
        .resolution
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::contradiction_review_set(
            CONTRADICTION_API_ACTOR_ID,
            &observation_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let observation = ContradictionReviewService::new(pool)
        .review_manual(&observation_id, review_state, resolution)
        .await?;

    Ok(Json(observation))
}

fn contradiction_store(state: &AppState) -> Result<ContradictionObservationStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::app_store::<
        ContradictionObservationStore,
    >(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn validate_required_value(value: &str) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::InvalidContradictionReview(
            "missing required contradiction observation id",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_CONTRADICTION_LIMIT);
    if !(MIN_CONTRADICTION_LIMIT..=MAX_CONTRADICTION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidContradictionQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
```

### `backend/src/app/handlers/decisions/handlers.rs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/backend/src/app/handlers/decisions/handlers.rs`
- Size bytes / Размер в байтах: `4205`
- Included characters / Включено символов: `4205`
- Truncated / Обрезано: `no`

```rust
use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use super::models::{DecisionListQuery, DecisionListResponse, DecisionReviewApiRequest};
use crate::app::{ApiError, AppState};
use crate::application::DecisionReviewApplicationService;
use crate::domains::decisions::{Decision, DecisionEntityKind, DecisionReviewState, DecisionStore};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};

const DECISION_API_ACTOR_ID: &str = "makosh-frontend";
const DEFAULT_DECISION_LIMIT: i64 = 50;
const MIN_DECISION_LIMIT: i64 = 1;
const MAX_DECISION_LIMIT: i64 = 100;

pub(crate) async fn get_v1_decisions(
    State(state): State<AppState>,
    Query(query): Query<DecisionListQuery>,
) -> Result<Json<DecisionListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let store = decision_store(&state)?;
    let items = match (
        query.review_state.as_deref(),
        query.entity_kind.as_deref(),
        query.entity_id.as_deref(),
    ) {
        (Some(review_state), None, None) => {
            let review_state = parse_review_state(review_state)?;
            store.list_by_review_state(review_state, limit).await?
        }
        (None, Some(entity_kind), Some(entity_id)) => {
            let entity_kind = parse_required_entity_kind(Some(entity_kind))?;
            let entity_id = validate_required_query_value(Some(entity_id))?;
            store
                .list_for_entity(entity_kind, &entity_id, limit)
                .await?
        }
        (Some(_), _, _) => {
            return Err(ApiError::InvalidDecisionQuery(
                "review_state cannot be combined with entity filters",
            ));
        }
        (None, _, _) => {
            return Err(ApiError::InvalidDecisionQuery(
                "missing required decision query field",
            ));
        }
    };

    Ok(Json(DecisionListResponse { items }))
}

pub(crate) async fn put_v1_decision_review(
    State(state): State<AppState>,
    Path(decision_id): Path<String>,
    Json(request): Json<DecisionReviewApiRequest>,
) -> Result<Json<Decision>, ApiError> {
    let decision_id = validate_required_query_value(Some(&decision_id))?;
    let review_state = parse_review_state(&request.review_state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::decision_review_set(
            DECISION_API_ACTOR_ID,
            &decision_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let decision = DecisionReviewApplicationService::new(pool)
        .review_manual(&decision_id, review_state)
        .await?;

    Ok(Json(decision))
}

fn decision_store(state: &AppState) -> Result<DecisionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::app_store::<DecisionStore>(
        pool.clone(),
    ))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_required_entity_kind(value: Option<&str>) -> Result<DecisionEntityKind, ApiError> {
    let value = validate_required_query_value(value)?;
    DecisionEntityKind::parse(&value).map_err(ApiError::from)
}

fn parse_review_state(value: &str) -> Result<DecisionReviewState, ApiError> {
    DecisionReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidDecisionQuery(
            "missing required decision query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_DECISION_LIMIT);
    if !(MIN_DECISION_LIMIT..=MAX_DECISION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidDecisionQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
```

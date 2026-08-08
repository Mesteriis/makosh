---
chunk_id: 103-other-contracts
batch_id: batch-20260628T214902
group: contracts
role: other
source_status: pending
source_count: 4
generated_by: code-wiki-ru
---

# 103-other-contracts — contracts/other

- Target index: [[components/contracts]]
- Batch: `batch-20260628T214902`
- Source files: `4`

## Резюме

Чанк `103-other-contracts` требует обновить русскую wiki-страницу `components/contracts.md`.
На основе встроенных исходных файлов protobuf (`common.proto`, `communications.proto`, `event_envelope.proto`, `signal_hub.proto`) необходимо задокументировать структуру контрактов — ключевые сообщения, их поля и сервисные определения.
Страница должна давать целостное представление о контрактах, доступных в репозитории `makosh`, исключая любые домыслы о семантике, не подтверждённые исходными текстами.

## Предложенные страницы

#### `components/contracts.md`

```markdown
# Контракты (Contracts)

В этом разделе описаны контракты (protobuf-сообщения и сервисы), используемые
в проекте `makosh`. Каждый подраздел соответствует одному `.proto`-файлу
из директории `contracts/proto`.

> **Важно:** все утверждения ниже подкреплены исключительно текстом исходных
> файлов, встроенных в данный контекст. Семантика полей, выходящая за рамки
> их имён и типов, не подтверждена.

## Общие (Common)

Файл: `contracts/proto/makosh/common/v1/common.proto`
Package: `makosh.common.v1`

### `PageRequest`

Запрос страницы с пагинацией.

| Поле   | Тип      | Описание                         |
|--------|----------|----------------------------------|
| limit  | uint32   | Максимальное число записей       |
| cursor | string   | Курсор, указывающий на страницу  |

### `PageResponse`

Ответ со страницей.

| Поле        | Тип    | Описание                          |
|-------------|--------|-----------------------------------|
| next_cursor | string | Курсор для следующей страницы     |
| has_more    | bool   | Есть ли ещё результаты            |

---

## Коммуникации (Communications)

Файл: `contracts/proto/makosh/communications/v1/communications.proto`
Package: `makosh.communications.v1`

> ⚠️ Встроенный исходный файл обрезан после 12000 символов. Описаны только те
> сообщения, которые попали в видимый фрагмент. Часть определений, включая
> окончание `RichTemplateMailMergePreview...`, отсутствует.

### Основные сущности

#### `CommunicationMessageAttachment`

Вложение к сообщению.

| Поле                   | Тип      | Обязательность | Примечание                     |
|------------------------|----------|----------------|--------------------------------|
| attachment_id          | string   | —              |                                |
| message_id             | string   | —              |                                |
| raw_record_id          | string   | —              |                                |
| blob_id                | string   | —              |                                |
| provider_attachment_id | string   | —              |                                |
| filename               | optional string | optional |                        |
| content_type           | string   | —              |                                |
| size_bytes             | int64    | —              |                                |
| sha256                 | string   | —              |                                |
| disposition            | string   | —              |                                |
| scan_status            | string   | —              |                                |
| scan_engine            | optional string | optional |                        |
| scan_checked_at        | optional string | optional |                        |
| scan_summary           | optional string | optional |                        |
| scan_metadata_json     | string   | —              |                                |
| storage_kind           | string   | —              |                                |
| storage_path           | string   | —              |                                |
| created_at             | string   | —              |                                |
| updated_at             | string   | —              |                                |

#### `CommunicationMessage`

Основная модель сообщения.

| Поле                       | Тип              | Обязательность | Примечание                   |
|----------------------------|------------------|----------------|------------------------------|
| message_id                 | string           | —              |                              |
| raw_record_id              | string           | —              |                              |
| observation_id             | string           | —              |                              |
| account_id                 | string           | —              |                              |
| provider_record_id         | string           | —              |                              |
| subject                    | string           | —              |                              |
| sender                     | string           | —              |                              |
| recipients                 | repeated string  | —              |                              |
| body_text                  | string           | —              |                              |
| occurred_at                | optional string  | optional       |                              |
| projected_at               | string           | —              |                              |
| channel_kind               | string           | —              |                              |
| conversation_id            | optional string  | optional       |                              |
| sender_display_name        | optional string  | optional       |                              |
| delivery_state             | string           | —              |                              |
| message_metadata_json      | string           | —              |                              |
| workflow_state             | string           | —              |                              |
| importance_score           | optional int32   | optional       |                              |
| ai_category                | optional string  | optional       |                              |
| ai_summary                 | optional string  | optional       |                              |
| ai_summary_generated_at    | optional string  | optional       |                              |
| local_state                | string           | —              |                              |
| local_state_changed_at     | optional string  | optional       |                              |
| local_state_reason         | optional string  | optional       |                              |
| attachment_count           | int64            | —              |                              |

### Запросы и ответы (сообщения и действия)

#### Листинг и получение

- `ListMessagesRequest` – фильтры: account_id, workflow_state, channel_kind, conversation_id, query, match_mode, local_state; пагинация: cursor, limit.
- `ListMessagesResponse` – items (repeated CommunicationMessage), next_cursor (optional), has_more.
- `GetMessageRequest` – message_id.
- `GetMessageResponse` – item (CommunicationMessage), attachments (repeated CommunicationMessageAttachment).

#### Управление состоянием

- `TransitionMessageWorkflowStateRequest` – message_id, workflow_state.
- `TransitionMessageWorkflowStateResponse` – message_id, workflow_state, previous_state.
- `UpdateMessageLocalStateRequest` – message_id.
- `UpdateMessageLocalStateResponse` – message_id, local_state, provider_deleted (optional bool).

#### Действия с сообщением

- `MarkMessageReadRequest` – message_id.
- `MarkMessageReadResponse` – message_id, marked_read, workflow_state.
- `DeleteMessageFromProviderRequest` – message_id.
- `DeleteMessageFromProviderResponse` – message_id, deleted, local_state, provider_deleted (optional).
- `BulkMessageActionRequest` – action, message_ids (repeated), label (optional), snooze_until (optional).
- `BulkMessageActionResponse` – action, requested_count, matched_count, updated_count, not_found (repeated string).

#### Toggle-операции

- `MessageToggleRequest` – message_id.
- `ToggleMessagePinResponse` – message_id, pinned.
- `ToggleMessageImportantResponse` – message_id, important.
- `ToggleMessageMuteResponse` – message_id, muted.

#### Snooze и метки

- `SnoozeMessageRequest` – message_id, until.
- `SnoozeMessageResponse` – snoozed.
- `UpdateMessageLabelRequest` – message_id, label.
- `AddMessageLabelResponse` – labeled.
- `RemoveMessageLabelResponse` – removed.

### Аналитика и AI

- `MessageKnowledgeCandidate` – title, evidence.
- `MessageSummaryContract` – key_points, action_items, risks, deadlines, а также кандидаты: event_candidates, persona_candidates, organization_candidates, document_candidates, agreement_candidates (все repeated MessageKnowledgeCandidate).
- `AnalyzeMessageRequest` – message_id.
- `AnalyzeMessageResponse` – message_id, analyzed, category (optional), summary (optional), summary_contract (MessageSummaryContract), importance_score (optional int32), workflow_state, source, confidence (optional double), evidence (repeated string).
- `WorkflowActionSource` – kind, id.
- `WorkflowActionInput` – title, body, email, display_name, starts_at, ends_at, due_at, document_id (все optional).
- `WorkflowActionRequest` – command_id, action, source (optional WorkflowActionSource), input (optional WorkflowActionInput).
- `WorkflowActionTarget` – kind, id (optional).
- `WorkflowActionProvenance` – source_kind (optional), source_id (optional), confidence (optional double), evidence (repeated string).
- `WorkflowActionResponse` – command_id, event_id, action, status, target (WorkflowActionTarget), provenance (WorkflowActionProvenance).
- `ExplainMessageRequest` – message_id.
- `ExplainMessageResponse` – reasons (repeated string).
- `GetMessageSmartCcRequest` – message_id.
- `GetMessageSmartCcResponse` – suggestions (repeated string).
- `GetMessageExportRequest` – message_id, format.
- `GetMessageExportResponse` – content_type, content, filename.

### Аутентификация и подписи

- `MessageAuthResult` – result, domain (optional), ip (optional), selector (optional), policy (optional).
- `MessageAuthReport` – spf, dkim, dmarc (все optional MessageAuthResult), raw_headers (repeated string).
- `MessageAuthRiskReport` – has_spf, spf_pass, has_dkim, dkim_pass, has_dmarc, dmarc_pass, is_spoofed, risk_summary.
- `GetMessageAuthRequest` – message_id.
- `GetMessageAuthResponse` – auth (MessageAuthReport), risk (MessageAuthRiskReport).
- `GetMessageSignatureRequest` – message_id.
- `GetMessageSignatureResponse` – has_signature, signature_type (optional), signer_info (optional), is_valid (optional), cert_expiry_warning (optional).

### AI-ответы

- `AiReplyRequest` – message_id, tone (optional), language (optional), context (optional).
- `AiReplyResponse` – subject (optional), body (optional), tone (optional), language (optional), generated (optional), reason (optional).
- `AiReplyVariantsRequest` – message_id, languages (repeated string), tones (repeated string).
- `AiReplyVariantsResponse` – variants (repeated AiReplyResponse).

### Состояния и здоровье почтового ящика

- `WorkflowStateCount` – state, count.
- `ListMessageWorkflowStateCountsRequest` – account_id (optional), local_state (optional).
- `ListMessageWorkflowStateCountsResponse` – counts (repeated WorkflowStateCount).
- `MailboxHealth` – total_messages, unread, needs_action, waiting, done, archived, spam, important, with_attachments, average_importance (double), oldest_message_days (optional double).
- `GetMailboxHealthRequest` – account_id (optional).
- `GetMailboxHealthResponse` – item (MailboxHealth).
- `SenderStats` – sender, message_count, avg_importance (double), last_message_days (optional double).
- `ListTopSendersRequest` – account_id (optional), cursor (optional), limit.
- `ListTopSendersResponse` – items (repeated SenderStats), next_cursor (optional), has_more.

### Подписки (subscriptions)

- `SubscriptionSource` – sender, message_count, first_seen, last_seen, is_newsletter, has_unsubscribe.
- `ListSubscriptionsRequest` – account_id (optional), cursor (optional), limit.
- `ListSubscriptionsResponse` – items (repeated SubscriptionSource), next_cursor (optional), has_more.

### Архитектурные блокеры и персоны

- `CommunicationArchitectureBlocker` – section, feature, reason, resolution.
- `ListCommunicationBlockersRequest` – пустой.
- `ListCommunicationBlockersResponse` – items (repeated CommunicationArchitectureBlocker).
- `CommunicationPersona` – persona_id, account_id, name, display_name, signature, default_language (optional), default_tone (optional), is_default, metadata_json, created_at, updated_at.
- `ListCommunicationPersonasRequest` – пустой.
- `ListCommunicationPersonasResponse` – items (repeated CommunicationPersona).

### Богатые шаблоны (Rich Templates)

- `RichTemplate` – template_id, name, subject_template, body_template, variables (repeated string), placeholder_variables (repeated), undeclared_variables (repeated), unused_variables (repeated), malformed_placeholders (repeated), language (optional), created_at, updated_at.
- `ListRichTemplatesRequest` – пустой.
- `ListRichTemplatesResponse` – templates (repeated RichTemplate).
- `UpsertRichTemplateRequest` – template_id (optional), name, subject_template, body_template, variables (repeated string), language (optional).
- `UpsertRichTemplateResponse` – saved, template (RichTemplate).
- `DeleteRichTemplateRequest` – template_id.
- `DeleteRichTemplateResponse` – template_id, deleted.
- `RichTemplateRenderRequest` – template_id, variables (map<string,string>).
- `RenderedRichTemplate` – subject, body, missing_variables (repeated string), unresolved_variables (repeated), malformed_placeholders (repeated).
- `RichTemplateRenderResponse` – template_id, variables (map<string,string>), rendered (RenderedRichTemplate).
- `RichTemplateMailMergePreviewRow` – row_id, variables (map<string,string>).
- `RichTemplateMailMergePreviewRequest` – template_id, rows (repeated RichTemplateMailMergePreviewRow).
- `RichTemplateMailMergePreview...` – **обрезано в исходном фрагменте**.

---

## События (Events)

Файл: `contracts/proto/makosh/events/v1/event_envelope.proto`
Package: `makosh.events.v1`

### `EventEnvelope`

Обёртка события. Использует `google.protobuf.Timestamp` и `google.protobuf.Struct`.

| Поле            | Тип                           | Описание                                      |
|-----------------|-------------------------------|-----------------------------------------------|
| event_id        | string                        | Уникальный идентификатор события              |
| event_type      | string                        | Тип события                                   |
| schema_version  | int32                         | Версия схемы события                          |
| occurred_at     | google.protobuf.Timestamp     | Момент, когда событие произошло               |
| recorded_at     | google.protobuf.Timestamp     | Момент, когда событие было записано           |
| source          | google.protobuf.Struct         | Источник события                              |
| actor           | google.protobuf.Struct         | Действующее лицо                              |
| subject         | google.protobuf.Struct         | Объект события                                |
| payload         | google.protobuf.Struct         | Полезная нагрузка события                     |
| provenance      | google.protobuf.Struct         | Источник достоверности данных                 |
| causation_id    | string                        | Идентификатор причинно-следственной цепочки   |
| correlation_id  | string                        | Идентификатор корреляции                      |

---

## Хаб сигналов (Signal Hub)

Файл: `contracts/proto/makosh/signal_hub/v1/signal_hub.proto`
Package: `makosh.signal_hub.v1`

### Сущности

#### `SignalSource`

Источник сигналов.

| Поле                       | Тип    | Примечание                           |
|----------------------------|--------|--------------------------------------|
| id                         | string |                                      |
| code                       | string |                                      |
| display_name               | string |                                      |
| category                   | string |                                      |
| source_kind                | string |                                      |
| default_enabled            | bool   |                                      |
| supports_connections       | bool   |                                      |
| supports_runtime           | bool   |                                      |
| supports_replay            | bool   |                                      |
| supports_pause             | bool   |                                      |
| supports_mute              | bool   |                                      |
| capability_schema_version  | int32  |                                      |
| created_at                 | string |                                      |
| updated_at                 | string |                                      |

#### `SignalConnection`

Подключение к внешнему источнику.

| Поле            | Тип             | Примечание                 |
|-----------------|-----------------|----------------------------|
| id              | string          |                            |
| source_code     | string          |                            |
| display_name    | string          |                            |
| status          | string          |                            |
| profile         | optional string |                            |
| secret_ref      | optional string |                            |
| connected_at    | optional string |                            |
| last_seen_at    | optional string |                            |
| last_signal_at  | optional string |                            |
| last_sync_at    | optional string |                            |
| created_at      | string          |                            |
| updated_at      | string          |                            |
| settings_json   | string          |                            |

#### `SignalCapability`

Возможность источника (например, OAuth scope).

| Поле                  | Тип             | Примечание           |
|-----------------------|-----------------|----------------------|
| id                    | string          |                      |
| source_code           | string          |                      |
| connection_id         | optional string |                      |
| capability            | string          |                      |
| state                 | string          |                      |
| reason                | optional string |                      |
| requires_confirmation | bool            |                      |
| action_class          | string          |                      |
| updated_at            | string          |                      |

#### `SignalHealth`

Здоровье источника или подключения.

| Поле                      | Тип             | Примечание                        |
|---------------------------|-----------------|-----------------------------------|
| id                        | string          |                                   |
| source_code               | string          |                                   |
| connection_id             | optional string |                                   |
| level                     | string          |                                   |
| summary                   | string          |                                   |
| last_ok_at                | optional string |                                   |
| last_failure_at           | optional string |                                   |
| failure_count             | int32           |                                   |
| consecutive_failure_count | int32           |                                   |
| next_retry_at             | optional string |                                   |
| updated_at                | string          |                                   |
| evidence_json             | string          |                                   |

#### `SignalRuntimeState`

Состояние runtime-сущности (поток, подписка и т.п.).

| Поле                       | Тип             | Примечание                     |
|----------------------------|-----------------|--------------------------------|
| id                         | string          |                                |
| source_code                | string          |                                |
| connection_id              | optional string |                                |
| runtime_kind               | string          |                                |
| state                      | string          |                                |
| metadata_json              | string          |                                |
| updated_at                 | string          |                                |
| last_started_at            | optional string |                                |
| last_stopped_at            | optional string |                                |
| last_heartbeat_at          | optional string |                                |
| last_error_at              | optional string |                                |
| last_error_code            | optional string |                                |
| last_error_message_redacted | optional string | (сообщение об ошибке без секретов) |

#### `SignalPolicy`

Политика управления сигналами.

| Поле           | Тип             | Примечание                    |
|----------------|-----------------|-------------------------------|
| scope          | string          |                               |
| source_code    | optional string |                               |
| connection_id  | optional string |                               |
| event_pattern  | optional string |                               |
| mode           | string          |                               |
| reason         | string          |                               |
| expires_at     | optional string |                               |

#### `SignalProfile` и `SignalProfilePolicy`

Профиль – набор политик.

`SignalProfile`:

| Поле            | Тип                            | Примечание  |
|-----------------|--------------------------------|-------------|
| id              | string                         |             |
| code            | string                         |             |
| display_name    | string                         |             |
| description     | string                         |             |
| policy_count    | uint32                         |             |
| is_system       | bool                           |             |
| is_active       | bool                           |             |
| created_at      | string                         |             |
| updated_at      | string                         |             |
| source_policies | repeated SignalProfilePolicy   |             |

`SignalProfilePolicy`:

| Поле           | Тип             | Примечание |
|----------------|-----------------|------------|
| scope          | string          |            |
| source_code    | optional string |            |
| connection_id  | optional string |            |
| event_pattern  | optional string |            |
| mode           | string          |            |
| reason         | string          |            |

#### `SignalReplayRequest`

Запрос на повторное воспроизведение сигналов.

| Поле                  | Тип              | Примечание                              |
|-----------------------|------------------|-----------------------------------------|
| id                    | string           |                                         |
| source_code           | optional string  |                                         |
| connection_id         | optional string  |                                         |
| event_pattern         | optional string  |                                         |
| from_position         | optional int64   |                                         |
| to_position           | optional int64   |                                         |
| from_time             | optional string  |                                         |
| to_time               | optional string  |                                         |
| target_consumer       | optional string  |                                         |
| target_projection     | optional string  |                                         |
| status                | string           |                                         |
| requested_by          | string           |                                         |
| requested_at          | string           |                                         |
| started_at            | optional string  |                                         |
| completed_at          | optional string  |                                         |
| last_error_redacted   | optional string  | (сообщение об ошибке без секретов)      |
| replayed_count        | int32            |                                         |
| metadata_json         | string           |                                         |

#### Фикстуры

- `SignalFixtureSource` – fixture_id, source_code, event_type, correlation_id (optional), occurred_at, summary.
- Используется в запросе `EmitFixtureSignalRequest` (fixture_id) и ответе `EmitFixtureSignalResponse` (fixture_id, raw_event_id, event_type, source_code, correlation_id optional).
- `RestoreSystemFixtureRequest` – пустой.
- `RestoreSystemFixtureResponse` – sources_created, sources_repaired, profiles_created, profiles_repaired (все uint32).

### Запросы/ответы (управление)

- `ListSourcesRequest` / `ListSourcesResponse` – items (repeated SignalSource).
- `GetSourceRequest` (code) / `GetSourceResponse` – item (SignalSource).
- `ListCapabilitiesRequest` (source_code optional, connection_id optional) / `ListCapabilitiesResponse` – items (repeated SignalCapability).
- `ListFixtureSourcesRequest` / `ListFixtureSourcesResponse` – items (repeated SignalFixtureSource).
- `ListConnectionsRequest` / `ListConnectionsResponse` – items (repeated SignalConnection).
- `CreateConnectionRequest` (source_code, display_name, status, profile optional, secret_ref optional, settings_json) / `CreateConnectionResponse` – item (SignalConnection).
- `UpdateConnectionRequest` (id, display_name optional, status optional, profile optional, secret_ref optional, settings_json optional) / `UpdateConnectionResponse` – item.
- `RemoveConnectionRequest` (id) / `RemoveConnectionResponse` – item.
- `EnableSourceRequest` (source_code, reason optional) / `EnableSourceResponse` – source_code, cleared_count.
- `DisableSourceRequest` (source_code, reason optional) / `DisableSourceResponse` – source_code, policy_id.
- `DisableSignalsRequest` / `DisableSignalsResponse` – policy_id optional.
- `EnableSignalsRequest` / `EnableSignalsResponse` – cleared_count.
- `MuteSignalsRequest` / `MuteSignalsResponse` – policy_id optional.
- `UnmuteSignalsRequest` / `UnmuteSignalsResponse` – cleared_count.
- `PauseSignalsRequest` / `PauseSignalsResponse` – policy_id optional.
- `ResumeSignalsRequest` / `ResumeSignalsResponse` – cleared_count.
- `ListHealthRequest` / `ListHealthResponse` – items (repeated SignalHealth).
- `RunHealthCheckRequest` (source_code, connection_id optional) / `RunHealthCheckResponse` – item (SignalHealth).
- `ListRuntimeStatesRequest` / `ListRuntimeStatesResponse` – items (repeated SignalRuntimeState).
- `UpdateRuntimeStateRequest` (source_code, runtime_kind, state, metadata_json) / `UpdateRuntimeStateResponse` – item.
- `ListPoliciesRequest` / `ListPoliciesResponse` – items (repeated SignalPolicy).
- `CreatePolicyRequest` (scope, source_code optional, connection_id optional, event_pattern optional, mode, reason, expires_at optional) / `CreatePolicyResponse` – id.
- `ListProfilesRequest` / `ListProfilesResponse` – items (repeated SignalProfile).
- `CreateProfileRequest` (code, display_name, description, source_policies repeated SignalProfilePolicy) / `CreateProfileResponse` – item.
- `UpdateProfileRequest` (code, display_name optional, description optional, source_policies, update_source_policies) / `UpdateProfileResponse` – item.
- `RemoveProfileRequest` (code) / `RemoveProfileResponse` – item.
- `ApplyProfileRequest` (code) / `ApplyProfileResponse` – item.
- `ListReplayRequestsRequest` / `ListReplayRequestsResponse` – items (repeated SignalReplayRequest).
- `RequestReplayRequest` (source_code optional, connection_id optional, event_pattern optional, from_position optional, to_position optional, from_time optional, to_time optional, target_consumer optional, target_projection optional, metadata_json) / `RequestReplayResponse` – item.
- `EmitFixtureSignalRequest` / `EmitFixtureSignalResponse`.
- `RestoreSystemFixtureRequest` / `RestoreSystemFixtureResponse`.

### Сервис `SignalHubService`

Определяет следующие RPC:

- `ListSources`, `GetSource`
- `ListCapabilities`
- `ListFixtureSources`
- `ListConnections`, `CreateConnection`, `UpdateConnection`, `RemoveConnection`
- `EnableSource`, `DisableSource`
- `DisableSignals`, `EnableSignals`
- `MuteSignals`, `UnmuteSignals`
- `PauseSignals`, `ResumeSignals`
- `ListHealth`, `RunHealthCheck`
- `ListRuntimeStates`, `UpdateRuntimeState`
- `ListPolicies`, `CreatePolicy`
- `ListProfiles`, `CreateProfile`, `UpdateProfile`, `RemoveProfile`, `ApplyProfile`
- `ListReplayRequests`, `RequestReplay`
- `EmitFixtureSignal`
- `RestoreSystemFixture`
```

## Покрытие источников

| Исходный файл                                                                 | Факты, покрытые предложенной страницей |
|-------------------------------------------------------------------------------|----------------------------------------|
| `contracts/proto/makosh/common/v1/common.proto`                               | `PageRequest` (поля: limit, cursor), `PageResponse` (поля: next_cursor, has_more). |
| `contracts/proto/makosh/communications/v1/communications.proto` (truncated)   | Все перечисленные в документе сообщения из видимой части файла: `CommunicationMessageAttachment`, `CommunicationMessage`, `ListMessagesRequest/Response`, `GetMessageRequest/Response`, `TransitionMessageWorkflowState*`, `UpdateMessageLocalState*`, `MarkMessageRead*`, `DeleteMessageFromProvider*`, `BulkMessageAction*`, `Toggle*`, `SnoozeMessage*`, `UpdateMessageLabel*`, `MessageKnowledgeCandidate`, `MessageSummaryContract`, `AnalyzeMessage*`, `WorkflowActionSource/Input/Request/Target/Provenance/Response`, `ExplainMessage*`, `GetMessageSmartCc*`, `GetMessageExport*`, `MessageAuthResult/Report/RiskReport`, `GetMessageAuth*`, `GetMessageSignature*`, `AiReply*`, `WorkflowStateCount`, `ListMessageWorkflowStateCounts*`, `SubscriptionSource`, `ListSubscriptions*`, `MailboxHealth`, `GetMailboxHealth*`, `SenderStats`, `ListTopSenders*`, `CommunicationArchitectureBlocker`, `ListCommunicationBlockers*`, `CommunicationPersona`, `ListCommunicationPersonas*`, `RichTemplate`, `ListRichTemplates*`, `UpsertRichTemplate*`, `DeleteRichTemplate*`, `RichTemplateRender*`, `RichTemplateMailMergePreviewRow`, `RichTemplateMailMergePreviewRequest`. Сообщения после обрезки не покрыты. |
| `contracts/proto/makosh/events/v1/event_envelope.proto`                       | `EventEnvelope` (поля: event_id, event_type, schema_version, occurred_at, recorded_at, source, actor, subject, payload, provenance, causation_id, correlation_id). |
| `contracts/proto/makosh/signal_hub/v1/signal_hub.proto`                       | Все перечисленные сообщения: `SignalSource`, `SignalConnection`, `SignalCapability`, `SignalFixtureSource`, `SignalHealth`, `SignalRuntimeState`, `SignalPolicy`, `SignalProfile`, `SignalProfilePolicy`, `SignalReplayRequest`, а также все Request/Response-пары для операций над источниками, подключениями, политиками, профилями, воспроизведением, фикстурами, и полный список RPC сервиса `SignalHubService`. |

## Исходные файлы

- [`contracts/proto/makosh/common/v1/common.proto`](../../../../contracts/proto/makosh/common/v1/common.proto)
- [`contracts/proto/makosh/communications/v1/communications.proto`](../../../../contracts/proto/makosh/communications/v1/communications.proto)
- [`contracts/proto/makosh/events/v1/event_envelope.proto`](../../../../contracts/proto/makosh/events/v1/event_envelope.proto)
- [`contracts/proto/makosh/signal_hub/v1/signal_hub.proto`](../../../../contracts/proto/makosh/signal_hub/v1/signal_hub.proto)

## Кандидаты на drift

Из предоставленного контекста расхождения (drift) между кодом, документацией и ADR не видны.
Контекст включает только исходные `.proto`-файлы; нет других wiki-страниц, ADR или реализаций для сравнения.
Обращаем внимание, что файл `communications.proto` обрезан на 12000 символах, поэтому полный охват контракта не может быть проверен — потенциально не задокументированные сообщения могут отличаться от ожидаемых в других частях системы, но установить это из данного контекста невозможно.

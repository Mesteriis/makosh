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

- Chunk ID / ID чанка: `103-other-contracts`
- Group / Группа: `contracts`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/contracts.md`

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

### `contracts/proto/makosh/common/v1/common.proto`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/contracts/proto/makosh/common/v1/common.proto`
- Size bytes / Размер в байтах: `185`
- Included characters / Включено символов: `185`
- Truncated / Обрезано: `no`

```text
syntax = "proto3";

package makosh.common.v1;

message PageRequest {
  uint32 limit = 1;
  string cursor = 2;
}

message PageResponse {
  string next_cursor = 1;
  bool has_more = 2;
}
```

### `contracts/proto/makosh/communications/v1/communications.proto`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/contracts/proto/makosh/communications/v1/communications.proto`
- Size bytes / Размер в байтах: `31511`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
syntax = "proto3";

package makosh.communications.v1;

import "makosh/common/v1/common.proto";

message CommunicationMessageAttachment {
  string attachment_id = 1;
  string message_id = 2;
  string raw_record_id = 3;
  string blob_id = 4;
  string provider_attachment_id = 5;
  optional string filename = 6;
  string content_type = 7;
  int64 size_bytes = 8;
  string sha256 = 9;
  string disposition = 10;
  string scan_status = 11;
  optional string scan_engine = 12;
  optional string scan_checked_at = 13;
  optional string scan_summary = 14;
  string scan_metadata_json = 15;
  string storage_kind = 16;
  string storage_path = 17;
  string created_at = 18;
  string updated_at = 19;
}

message CommunicationMessage {
  string message_id = 1;
  string raw_record_id = 2;
  string observation_id = 3;
  string account_id = 4;
  string provider_record_id = 5;
  string subject = 6;
  string sender = 7;
  repeated string recipients = 8;
  string body_text = 9;
  optional string occurred_at = 10;
  string projected_at = 11;
  string channel_kind = 12;
  optional string conversation_id = 13;
  optional string sender_display_name = 14;
  string delivery_state = 15;
  string message_metadata_json = 16;
  string workflow_state = 17;
  optional int32 importance_score = 18;
  optional string ai_category = 19;
  optional string ai_summary = 20;
  optional string ai_summary_generated_at = 21;
  string local_state = 22;
  optional string local_state_changed_at = 23;
  optional string local_state_reason = 24;
  int64 attachment_count = 25;
}

message ListMessagesRequest {
  optional string account_id = 1;
  optional string workflow_state = 2;
  optional string channel_kind = 3;
  optional string conversation_id = 4;
  optional string query = 5;
  optional string match_mode = 6;
  optional string local_state = 7;
  optional string cursor = 8;
  uint32 limit = 9;
}

message ListMessagesResponse {
  repeated CommunicationMessage items = 1;
  optional string next_cursor = 2;
  bool has_more = 3;
}

message GetMessageRequest {
  string message_id = 1;
}

message GetMessageResponse {
  CommunicationMessage item = 1;
  repeated CommunicationMessageAttachment attachments = 2;
}

message TransitionMessageWorkflowStateRequest {
  string message_id = 1;
  string workflow_state = 2;
}

message TransitionMessageWorkflowStateResponse {
  string message_id = 1;
  string workflow_state = 2;
  string previous_state = 3;
}

message UpdateMessageLocalStateRequest {
  string message_id = 1;
}

message UpdateMessageLocalStateResponse {
  string message_id = 1;
  string local_state = 2;
  optional bool provider_deleted = 3;
}

message MarkMessageReadRequest {
  string message_id = 1;
}

message MarkMessageReadResponse {
  string message_id = 1;
  bool marked_read = 2;
  string workflow_state = 3;
}

message DeleteMessageFromProviderRequest {
  string message_id = 1;
}

message DeleteMessageFromProviderResponse {
  string message_id = 1;
  bool deleted = 2;
  string local_state = 3;
  optional bool provider_deleted = 4;
}

message BulkMessageActionRequest {
  string action = 1;
  repeated string message_ids = 2;
  optional string label = 3;
  optional string snooze_until = 4;
}

message BulkMessageActionResponse {
  string action = 1;
  uint32 requested_count = 2;
  uint32 matched_count = 3;
  uint32 updated_count = 4;
  repeated string not_found = 5;
}

message MessageToggleRequest {
  string message_id = 1;
}

message ToggleMessagePinResponse {
  string message_id = 1;
  bool pinned = 2;
}

message ToggleMessageImportantResponse {
  string message_id = 1;
  bool important = 2;
}

message ToggleMessageMuteResponse {
  string message_id = 1;
  bool muted = 2;
}

message SnoozeMessageRequest {
  string message_id = 1;
  string until = 2;
}

message SnoozeMessageResponse {
  bool snoozed = 1;
}

message UpdateMessageLabelRequest {
  string message_id = 1;
  string label = 2;
}

message AddMessageLabelResponse {
  bool labeled = 1;
}

message RemoveMessageLabelResponse {
  bool removed = 1;
}

message MessageKnowledgeCandidate {
  string title = 1;
  string evidence = 2;
}

message MessageSummaryContract {
  repeated string key_points = 1;
  repeated string action_items = 2;
  repeated string risks = 3;
  repeated string deadlines = 4;
  repeated MessageKnowledgeCandidate event_candidates = 5;
  repeated MessageKnowledgeCandidate persona_candidates = 6;
  repeated MessageKnowledgeCandidate organization_candidates = 7;
  repeated MessageKnowledgeCandidate document_candidates = 8;
  repeated MessageKnowledgeCandidate agreement_candidates = 9;
}

message AnalyzeMessageRequest {
  string message_id = 1;
}

message AnalyzeMessageResponse {
  string message_id = 1;
  bool analyzed = 2;
  optional string category = 3;
  optional string summary = 4;
  MessageSummaryContract summary_contract = 5;
  optional int32 importance_score = 6;
  string workflow_state = 7;
  string source = 8;
  optional double confidence = 9;
  repeated string evidence = 10;
}

message WorkflowActionSource {
  string kind = 1;
  string id = 2;
}

message WorkflowActionInput {
  optional string title = 1;
  optional string body = 2;
  optional string email = 3;
  optional string display_name = 4;
  optional string starts_at = 5;
  optional string ends_at = 6;
  optional string due_at = 7;
  optional string document_id = 8;
}

message WorkflowActionRequest {
  string command_id = 1;
  string action = 2;
  optional WorkflowActionSource source = 3;
  optional WorkflowActionInput input = 4;
}

message WorkflowActionTarget {
  string kind = 1;
  optional string id = 2;
}

message WorkflowActionProvenance {
  optional string source_kind = 1;
  optional string source_id = 2;
  optional double confidence = 3;
  repeated string evidence = 4;
}

message WorkflowActionResponse {
  string command_id = 1;
  string event_id = 2;
  string action = 3;
  string status = 4;
  WorkflowActionTarget target = 5;
  WorkflowActionProvenance provenance = 6;
}

message ExplainMessageRequest {
  string message_id = 1;
}

message ExplainMessageResponse {
  repeated string reasons = 1;
}

message GetMessageSmartCcRequest {
  string message_id = 1;
}

message GetMessageSmartCcResponse {
  repeated string suggestions = 1;
}

message GetMessageExportRequest {
  string message_id = 1;
  string format = 2;
}

message GetMessageExportResponse {
  string content_type = 1;
  string content = 2;
  string filename = 3;
}

message MessageAuthResult {
  string result = 1;
  optional string domain = 2;
  optional string ip = 3;
  optional string selector = 4;
  optional string policy = 5;
}

message MessageAuthReport {
  optional MessageAuthResult spf = 1;
  optional MessageAuthResult dkim = 2;
  optional MessageAuthResult dmarc = 3;
  repeated string raw_headers = 4;
}

message MessageAuthRiskReport {
  bool has_spf = 1;
  bool spf_pass = 2;
  bool has_dkim = 3;
  bool dkim_pass = 4;
  bool has_dmarc = 5;
  bool dmarc_pass = 6;
  bool is_spoofed = 7;
  string risk_summary = 8;
}

message GetMessageAuthRequest {
  string message_id = 1;
}

message GetMessageAuthResponse {
  MessageAuthReport auth = 1;
  MessageAuthRiskReport risk = 2;
}

message GetMessageSignatureRequest {
  string message_id = 1;
}

message GetMessageSignatureResponse {
  bool has_signature = 1;
  optional string signature_type = 2;
  optional string signer_info = 3;
  optional bool is_valid = 4;
  optional string cert_expiry_warning = 5;
}

message AiReplyRequest {
  string message_id = 1;
  optional string tone = 2;
  optional string language = 3;
  optional string context = 4;
}

message AiReplyResponse {
  optional string subject = 1;
  optional string body = 2;
  optional string tone = 3;
  optional string language = 4;
  optional bool generated = 5;
  optional string reason = 6;
}

message AiReplyVariantsRequest {
  string message_id = 1;
  repeated string languages = 2;
  repeated string tones = 3;
}

message AiReplyVariantsResponse {
  repeated AiReplyResponse variants = 1;
}

message WorkflowStateCount {
  string state = 1;
  int64 count = 2;
}

message ListMessageWorkflowStateCountsRequest {
  optional string account_id = 1;
  optional string local_state = 2;
}

message ListMessageWorkflowStateCountsResponse {
  repeated WorkflowStateCount counts = 1;
}

message SubscriptionSource {
  string sender = 1;
  int64 message_count = 2;
  string first_seen = 3;
  string last_seen = 4;
  bool is_newsletter = 5;
  bool has_unsubscribe = 6;
}

message ListSubscriptionsRequest {
  optional string account_id = 1;
  optional string cursor = 2;
  uint32 limit = 3;
}

message ListSubscriptionsResponse {
  repeated SubscriptionSource items = 1;
  optional string next_cursor = 2;
  bool has_more = 3;
}

message MailboxHealth {
  int64 total_messages = 1;
  int64 unread = 2;
  int64 needs_action = 3;
  int64 waiting = 4;
  int64 done = 5;
  int64 archived = 6;
  int64 spam = 7;
  int64 important = 8;
  int64 with_attachments = 9;
  double average_importance = 10;
  optional double oldest_message_days = 11;
}

message GetMailboxHealthRequest {
  optional string account_id = 1;
}

message GetMailboxHealthResponse {
  MailboxHealth item = 1;
}

message SenderStats {
  string sender = 1;
  int64 message_count = 2;
  double avg_importance = 3;
  optional double last_message_days = 4;
}

message ListTopSendersRequest {
  optional string account_id = 1;
  optional string cursor = 2;
  uint32 limit = 3;
}

message ListTopSendersResponse {
  repeated SenderStats items = 1;
  optional string next_cursor = 2;
  bool has_more = 3;
}

message CommunicationArchitectureBlocker {
  string section = 1;
  string feature = 2;
  string reason = 3;
  string resolution = 4;
}

message ListCommunicationBlockersRequest {}

message ListCommunicationBlockersResponse {
  repeated CommunicationArchitectureBlocker items = 1;
}

message CommunicationPersona {
  string persona_id = 1;
  string account_id = 2;
  string name = 3;
  string display_name = 4;
  string signature = 5;
  optional string default_language = 6;
  optional string default_tone = 7;
  bool is_default = 8;
  string metadata_json = 9;
  string created_at = 10;
  string updated_at = 11;
}

message ListCommunicationPersonasRequest {}

message ListCommunicationPersonasResponse {
  repeated CommunicationPersona items = 1;
}

message RichTemplate {
  string template_id = 1;
  string name = 2;
  string subject_template = 3;
  string body_template = 4;
  repeated string variables = 5;
  repeated string placeholder_variables = 6;
  repeated string undeclared_variables = 7;
  repeated string unused_variables = 8;
  repeated string malformed_placeholders = 9;
  optional string language = 10;
  string created_at = 11;
  string updated_at = 12;
}

message ListRichTemplatesRequest {}

message ListRichTemplatesResponse {
  repeated RichTemplate templates = 1;
}

message UpsertRichTemplateRequest {
  optional string template_id = 1;
  string name = 2;
  string subject_template = 3;
  string body_template = 4;
  repeated string variables = 5;
  optional string language = 6;
}

message UpsertRichTemplateResponse {
  bool saved = 1;
  RichTemplate template = 2;
}

message DeleteRichTemplateRequest {
  string template_id = 1;
}

message DeleteRichTemplateResponse {
  string template_id = 1;
  bool deleted = 2;
}

message RichTemplateRenderRequest {
  string template_id = 1;
  map<string, string> variables = 2;
}

message RenderedRichTemplate {
  string subject = 1;
  string body = 2;
  repeated string missing_variables = 3;
  repeated string unresolved_variables = 4;
  repeated string malformed_placeholders = 5;
}

message RichTemplateRenderResponse {
  string template_id = 1;
  map<string, string> variables = 2;
  RenderedRichTemplate rendered = 3;
}

message RichTemplateMailMergePreviewRow {
  string row_id = 1;
  map<string, string> variables = 2;
}

message RichTemplateMailMergePreviewRequest {
  string template_id = 1;
  repeated RichTemplateMailMergePreviewRow rows = 2;
}

message RichTemplateMailMergePrevi
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `contracts/proto/makosh/events/v1/event_envelope.proto`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/contracts/proto/makosh/events/v1/event_envelope.proto`
- Size bytes / Размер в байтах: `570`
- Included characters / Включено символов: `570`
- Truncated / Обрезано: `no`

```text
syntax = "proto3";

package makosh.events.v1;

import "google/protobuf/struct.proto";
import "google/protobuf/timestamp.proto";

message EventEnvelope {
  string event_id = 1;
  string event_type = 2;
  int32 schema_version = 3;
  google.protobuf.Timestamp occurred_at = 4;
  google.protobuf.Timestamp recorded_at = 5;
  google.protobuf.Struct source = 6;
  google.protobuf.Struct actor = 7;
  google.protobuf.Struct subject = 8;
  google.protobuf.Struct payload = 9;
  google.protobuf.Struct provenance = 10;
  string causation_id = 11;
  string correlation_id = 12;
}
```

### `contracts/proto/makosh/signal_hub/v1/signal_hub.proto`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/contracts/proto/makosh/signal_hub/v1/signal_hub.proto`
- Size bytes / Размер в байтах: `11978`
- Included characters / Включено символов: `11978`
- Truncated / Обрезано: `no`

```text
syntax = "proto3";

package makosh.signal_hub.v1;

message SignalSource {
  string id = 1;
  string code = 2;
  string display_name = 3;
  string category = 4;
  string source_kind = 5;
  bool default_enabled = 6;
  bool supports_connections = 7;
  bool supports_runtime = 8;
  bool supports_replay = 9;
  bool supports_pause = 10;
  bool supports_mute = 11;
  int32 capability_schema_version = 12;
  string created_at = 13;
  string updated_at = 14;
}

message ListSourcesRequest {}

message ListSourcesResponse {
  repeated SignalSource items = 1;
}

message GetSourceRequest {
  string code = 1;
}

message GetSourceResponse {
  SignalSource item = 1;
}

message SignalCapability {
  string id = 1;
  string source_code = 2;
  optional string connection_id = 3;
  string capability = 4;
  string state = 5;
  optional string reason = 6;
  bool requires_confirmation = 7;
  string action_class = 8;
  string updated_at = 9;
}

message ListCapabilitiesRequest {
  optional string source_code = 1;
  optional string connection_id = 2;
}

message ListCapabilitiesResponse {
  repeated SignalCapability items = 1;
}

message SignalFixtureSource {
  string fixture_id = 1;
  string source_code = 2;
  string event_type = 3;
  optional string correlation_id = 4;
  string occurred_at = 5;
  string summary = 6;
}

message ListFixtureSourcesRequest {}

message ListFixtureSourcesResponse {
  repeated SignalFixtureSource items = 1;
}

message SignalConnection {
  string id = 1;
  string source_code = 2;
  string display_name = 3;
  string status = 4;
  optional string profile = 5;
  optional string secret_ref = 6;
  optional string connected_at = 7;
  optional string last_seen_at = 8;
  optional string last_signal_at = 9;
  optional string last_sync_at = 10;
  string created_at = 11;
  string updated_at = 12;
  string settings_json = 13;
}

message ListConnectionsRequest {}

message ListConnectionsResponse {
  repeated SignalConnection items = 1;
}

message CreateConnectionRequest {
  string source_code = 1;
  string display_name = 2;
  string status = 3;
  optional string profile = 4;
  optional string secret_ref = 5;
  string settings_json = 6;
}

message CreateConnectionResponse {
  SignalConnection item = 1;
}

message UpdateConnectionRequest {
  string id = 1;
  optional string display_name = 2;
  optional string status = 3;
  optional string profile = 4;
  optional string secret_ref = 5;
  optional string settings_json = 6;
}

message UpdateConnectionResponse {
  SignalConnection item = 1;
}

message RemoveConnectionRequest {
  string id = 1;
}

message RemoveConnectionResponse {
  SignalConnection item = 1;
}

message SignalHealth {
  string id = 1;
  string source_code = 2;
  optional string connection_id = 3;
  string level = 4;
  string summary = 5;
  optional string last_ok_at = 6;
  optional string last_failure_at = 7;
  int32 failure_count = 8;
  int32 consecutive_failure_count = 9;
  optional string next_retry_at = 10;
  string updated_at = 11;
  string evidence_json = 12;
}

message ListHealthRequest {}

message ListHealthResponse {
  repeated SignalHealth items = 1;
}

message RunHealthCheckRequest {
  string source_code = 1;
  optional string connection_id = 2;
}

message RunHealthCheckResponse {
  SignalHealth item = 1;
}

message SignalRuntimeState {
  string id = 1;
  string source_code = 2;
  optional string connection_id = 3;
  string runtime_kind = 4;
  string state = 5;
  string metadata_json = 6;
  string updated_at = 7;
  optional string last_started_at = 8;
  optional string last_stopped_at = 9;
  optional string last_heartbeat_at = 10;
  optional string last_error_at = 11;
  optional string last_error_code = 12;
  optional string last_error_message_redacted = 13;
}

message ListRuntimeStatesRequest {}

message ListRuntimeStatesResponse {
  repeated SignalRuntimeState items = 1;
}

message UpdateRuntimeStateRequest {
  string source_code = 1;
  string runtime_kind = 2;
  string state = 3;
  string metadata_json = 4;
}

message UpdateRuntimeStateResponse {
  SignalRuntimeState item = 1;
}

message SignalPolicy {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  string mode = 5;
  string reason = 6;
  optional string expires_at = 7;
}

message ListPoliciesRequest {}

message ListPoliciesResponse {
  repeated SignalPolicy items = 1;
}

message CreatePolicyRequest {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  string mode = 5;
  string reason = 6;
  optional string expires_at = 7;
}

message CreatePolicyResponse {
  string id = 1;
}

message EnableSourceRequest {
  string source_code = 1;
  optional string reason = 2;
}

message EnableSourceResponse {
  string source_code = 1;
  uint32 cleared_count = 2;
}

message DisableSourceRequest {
  string source_code = 1;
  optional string reason = 2;
}

message DisableSourceResponse {
  string source_code = 1;
  string policy_id = 2;
}

message DisableSignalsRequest {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  optional string reason = 5;
}

message DisableSignalsResponse {
  optional string policy_id = 1;
}

message EnableSignalsRequest {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  optional string reason = 5;
}

message EnableSignalsResponse {
  uint32 cleared_count = 1;
}

message MuteSignalsRequest {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  optional string reason = 5;
}

message MuteSignalsResponse {
  optional string policy_id = 1;
}

message UnmuteSignalsRequest {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  optional string reason = 5;
}

message UnmuteSignalsResponse {
  uint32 cleared_count = 1;
}

message PauseSignalsRequest {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  optional string reason = 5;
}

message PauseSignalsResponse {
  optional string policy_id = 1;
}

message ResumeSignalsRequest {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  optional string reason = 5;
}

message ResumeSignalsResponse {
  uint32 cleared_count = 1;
}

message SignalProfile {
  string id = 1;
  string code = 2;
  string display_name = 3;
  string description = 4;
  uint32 policy_count = 5;
  bool is_system = 6;
  bool is_active = 7;
  string created_at = 8;
  string updated_at = 9;
  repeated SignalProfilePolicy source_policies = 10;
}

message SignalProfilePolicy {
  string scope = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  string mode = 5;
  string reason = 6;
}

message ListProfilesRequest {}

message ListProfilesResponse {
  repeated SignalProfile items = 1;
}

message CreateProfileRequest {
  string code = 1;
  string display_name = 2;
  string description = 3;
  repeated SignalProfilePolicy source_policies = 4;
}

message CreateProfileResponse {
  SignalProfile item = 1;
}

message UpdateProfileRequest {
  string code = 1;
  optional string display_name = 2;
  optional string description = 3;
  repeated SignalProfilePolicy source_policies = 4;
  bool update_source_policies = 5;
}

message UpdateProfileResponse {
  SignalProfile item = 1;
}

message RemoveProfileRequest {
  string code = 1;
}

message RemoveProfileResponse {
  SignalProfile item = 1;
}

message ApplyProfileRequest {
  string code = 1;
}

message ApplyProfileResponse {
  SignalProfile item = 1;
}

message SignalReplayRequest {
  string id = 1;
  optional string source_code = 2;
  optional string connection_id = 3;
  optional string event_pattern = 4;
  optional int64 from_position = 5;
  optional int64 to_position = 6;
  optional string from_time = 7;
  optional string to_time = 8;
  optional string target_consumer = 9;
  optional string target_projection = 10;
  string status = 11;
  string requested_by = 12;
  string requested_at = 13;
  optional string started_at = 14;
  optional string completed_at = 15;
  optional string last_error_redacted = 16;
  int32 replayed_count = 17;
  string metadata_json = 18;
}

message ListReplayRequestsRequest {}

message ListReplayRequestsResponse {
  repeated SignalReplayRequest items = 1;
}

message RequestReplayRequest {
  optional string source_code = 1;
  optional string connection_id = 2;
  optional string event_pattern = 3;
  optional int64 from_position = 4;
  optional int64 to_position = 5;
  optional string from_time = 6;
  optional string to_time = 7;
  optional string target_consumer = 8;
  optional string target_projection = 9;
  string metadata_json = 10;
}

message RequestReplayResponse {
  SignalReplayRequest item = 1;
}

message EmitFixtureSignalRequest {
  string fixture_id = 1;
}

message EmitFixtureSignalResponse {
  string fixture_id = 1;
  string raw_event_id = 2;
  string event_type = 3;
  string source_code = 4;
  optional string correlation_id = 5;
}

message RestoreSystemFixtureRequest {}

message RestoreSystemFixtureResponse {
  uint32 sources_created = 1;
  uint32 sources_repaired = 2;
  uint32 profiles_created = 3;
  uint32 profiles_repaired = 4;
}

service SignalHubService {
  rpc ListSources(ListSourcesRequest) returns (ListSourcesResponse);
  rpc GetSource(GetSourceRequest) returns (GetSourceResponse);
  rpc ListCapabilities(ListCapabilitiesRequest) returns (ListCapabilitiesResponse);
  rpc ListFixtureSources(ListFixtureSourcesRequest) returns (ListFixtureSourcesResponse);
  rpc ListConnections(ListConnectionsRequest) returns (ListConnectionsResponse);
  rpc CreateConnection(CreateConnectionRequest) returns (CreateConnectionResponse);
  rpc UpdateConnection(UpdateConnectionRequest) returns (UpdateConnectionResponse);
  rpc RemoveConnection(RemoveConnectionRequest) returns (RemoveConnectionResponse);
  rpc EnableSource(EnableSourceRequest) returns (EnableSourceResponse);
  rpc DisableSource(DisableSourceRequest) returns (DisableSourceResponse);
  rpc DisableSignals(DisableSignalsRequest) returns (DisableSignalsResponse);
  rpc EnableSignals(EnableSignalsRequest) returns (EnableSignalsResponse);
  rpc MuteSignals(MuteSignalsRequest) returns (MuteSignalsResponse);
  rpc UnmuteSignals(UnmuteSignalsRequest) returns (UnmuteSignalsResponse);
  rpc PauseSignals(PauseSignalsRequest) returns (PauseSignalsResponse);
  rpc ResumeSignals(ResumeSignalsRequest) returns (ResumeSignalsResponse);
  rpc ListHealth(ListHealthRequest) returns (ListHealthResponse);
  rpc RunHealthCheck(RunHealthCheckRequest) returns (RunHealthCheckResponse);
  rpc ListRuntimeStates(ListRuntimeStatesRequest) returns (ListRuntimeStatesResponse);
  rpc UpdateRuntimeState(UpdateRuntimeStateRequest) returns (UpdateRuntimeStateResponse);
  rpc ListPolicies(ListPoliciesRequest) returns (ListPoliciesResponse);
  rpc CreatePolicy(CreatePolicyRequest) returns (CreatePolicyResponse);
  rpc ListProfiles(ListProfilesRequest) returns (ListProfilesResponse);
  rpc CreateProfile(CreateProfileRequest) returns (CreateProfileResponse);
  rpc UpdateProfile(UpdateProfileRequest) returns (UpdateProfileResponse);
  rpc RemoveProfile(RemoveProfileRequest) returns (RemoveProfileResponse);
  rpc ApplyProfile(ApplyProfileRequest) returns (ApplyProfileResponse);
  rpc ListReplayRequests(ListReplayRequestsRequest) returns (ListReplayRequestsResponse);
  rpc RequestReplay(RequestReplayRequest) returns (RequestReplayResponse);
  rpc EmitFixtureSignal(EmitFixtureSignalRequest) returns (EmitFixtureSignalResponse);
  rpc RestoreSystemFixture(RestoreSystemFixtureRequest) returns (RestoreSystemFixtureResponse);
}
```

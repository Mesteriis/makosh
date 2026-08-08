---
chunk_id: 142-source-frontend-part-002
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 142-source-frontend-part-002 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Страница `components/frontend.md` должна документировать модуль `frontend/src/domains/communications/api/` – слой API-функций фронтенда для работы с коммуникациями. В чанк входят все ключевые файлы модуля: публичные баррели (`communications.ts`, `messageApi.ts`), ConnectRPC-слой (`connect/`), маппинги, а также REST-функции для звонков, сертификатов, AI-состояний, двуязычных ответов и провайдер-нейтральных каналов. Предлагаемая страница описывает архитектуру, перечень функций с категориями, транспортный слой и маппинг типов.

---

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Коммуникации (Frontend API)

## Обзор

Модуль `frontend/src/domains/communications/api/` предоставляет API-функции для работы с данными коммуникаций (email, звонки, мессенджеры) на фронтенде. Функции объединяются в публичный API в файле `communications.ts`, который реэкспортирует реализации из подмодулей.

## Архитектура

- **Публичный слой**: `messageApi.ts`, `folderApi.ts`, `outboxApi.ts` и другие файлы верхнего уровня предлагают типизированные функции. Они делегируют вызовы либо в ConnectRPC-слой, либо напрямую в REST API.
- **ConnectRPC-слой**: файлы в `connect/` (collections, insights, messageLifecycle) реализуют вызовы к `makosh.communications.v1.CommunicationsService` через сгенерированный ConnectRPC-клиент или универсальный POST-хелпер.
- **Маппинг**: `connect/mapping.ts` преобразует ответы сервера (camelCase) в доменные типы (snake_case) и нормализует перечисления.
- **REST-эндпоинты**: некоторые операции взаимодействуют напрямую с REST API по путям `/api/v1/communications/...` или `/api/v1/calls/...`.

## Функции

### Сообщения (Messages)

Основные операции с коммуникационными сообщениями. Реализованы в `messageApi.ts` и `connect/messageLifecycle.ts`. Большинство методов используют ConnectRPC-клиент; отдельные вызывают кастомный POST через `postCommunicationsConnectJson`.

- `fetchCommunicationMessages` — список сообщений с фильтрацией и курсорной пагинацией (`ListMessages`).
- `fetchCommunicationMessage` — получение детальной информации о сообщении (`GetMessage`).
- `transitionMessageWorkflowState` — перевод сообщения по workflow-статусу (`TransitionMessageWorkflowState`).
- `fetchMessageStateCounts` — количество сообщений по статусам (`ListMessageWorkflowStateCounts`).
- `trashMessage` / `restoreMessage` — удаление в корзину / восстановление (`TrashMessage` / `RestoreMessage`).
- `markMessageRead` — отметка «прочитано» (`MarkMessageRead`).
- `deleteMessageFromProvider` — удаление на стороне провайдера (`DeleteMessageFromProvider`).
- `bulkMessageAction` — массовое действие над сообщениями (`BulkMessageAction`).
- `analyzeMessage` — AI-анализ сообщения (`AnalyzeMessage`).
- `runWorkflowAction` — выполнение workflow-команды (кастомный POST).
- `fetchMessageExplain` — объяснение, почему сообщение попало в категорию (`GetMessageExplain`).
- `fetchMessageSmartCc` — предложения для копии (CC) (`GetMessageSmartCc`).
- `toggleMessagePin` / `toggleMessageImportant` / `toggleMessageMute` — переключение флагов `pinned`, `important`, `muted`.
- `snoozeMessage` — отложить сообщение до указанного времени (кастомный POST).
- `addMessageLabel` — добавить метку (кастомный POST).
- `exportMessage` — экспорт в форматах md, eml, json (`GetMessageExport`).
- `fetchMessageAuth` — проверка аутентификации (SPF/DKIM/DMARC) (`GetMessageAuth`).
- `fetchMessageSignature` — проверка наличия подписи (`GetMessageSignature`).
- `detectMessageLanguage` — определение языка (`DetectMessageLanguage`).
- `translateMessage` — перевод сообщения (`TranslateMessage`).
- `generateAiReply` / `generateAiReplyVariants` — генерация AI-ответа / вариантов.
- `extractMessageTasks` / `extractMessageNotes` — извлечение задач / заметок из сообщения.
- `searchEmails` — поиск сообщений по запросу (`SearchMessages`).

### Черновики (Drafts)

Функции в `messageApi.ts` (через `fetchCommunicationDraftsConnect` и др.):

- `fetchDrafts` — список черновиков с пагинацией (`ListDrafts`).
- `createDraft` — создание черновика.
- `deleteDraft` — удаление черновика.

### Исходящие (Outbox)

Файл `outboxApi.ts`:

- `fetchOutboxItems` — список элементов исходящих (`ListOutbox`).
- `undoOutboxItem` — отмена отправки (`UndoOutboxItem`).

### Вложения (Attachments)

Функции в `attachmentApi.ts`, `attachmentImportApi.ts` и `connect/collections.ts`:

- `searchAttachments` — поиск вложений с фильтрами (`SearchAttachments`).
- `inspectAttachmentArchive` — инспекция архива (`GetAttachmentArchiveInspection`).
- `previewAttachment` — безопасный предпросмотр (`GetAttachmentPreview`).
- `translateAttachment` — перевод вложения (`TranslateAttachment`).
- `importCommunicationAttachment` — импорт локального вложения через REST `POST /api/v1/communications/attachments/import`.

### Папки (Folders)

Файл `folderApi.ts`, ConnectRPC-слой в `connect/collections.ts`:

- `fetchCommunicationFolders` — список папок (`ListFolders`).
- `createCommunicationFolder` / `updateCommunicationFolder` / `deleteCommunicationFolder` — создание, обновление, удаление папки.
- `fetchFolderMessages` — сообщения в папке (`ListFolderMessages`).
- `copyMessageToFolder` / `moveMessageToFolder` — копирование / перемещение сообщения в папку.

### Сохранённые поиски (Saved Searches)

Реэкспортируются из `savedSearchApi.ts` (файл не включён в чанк); ConnectRPC-реализации в `connect/collections.ts`:

- `fetchSavedSearches`, `createSavedSearch`, `updateSavedSearch`, `deleteSavedSearch`.

### Треды (Threads)

Реэкспорт из `threadApi.ts` (файл не включён); ConnectRPC-реализации в `connect/collections.ts`:

- `fetchThreads` — список тредов (`ListThreads`).
- `fetchThreadMessages` — сообщения треда (`ListThreadMessages`).
- `translateThread` — перевод треда (`TranslateThread`).

### Звонки (Calls)

Файл `callApi.ts` использует REST:

- `fetchProviderCalls` — список звонков провайдера (`GET /api/v1/calls?limit=...` с параметрами `account_id` и `provider`).
- `fetchProviderCallTranscript` — транскрипция звонка (`GET /api/v1/calls/{callId}/transcript`).

### Сертификаты (Certificates)

Файл `certificateApi.ts` использует REST:

- `fetchMailCertificates` — список сертификатов (`GET /api/v1/communications/certificates`).
- `fetchExpiringMailCertificates` — истекающие сертификаты (`GET /api/v1/communications/certificates/expiring?days=...`).
- `createMailCertificate` — создание сертификата (`POST /api/v1/communications/certificates`).

### Уведомления о прочтении (Read Receipts)

Тест `readReceipts.test.ts` подтверждает:

- `recordReadReceipt` — запись уведомления о прочтении (`POST /api/v1/communications/read-receipts`).

### Двуязычный ответ (Bilingual Reply)

Файл `bilingualReplyFlow.ts`:

- `prepareBilingualReplyFlow` — подготовка двуязычного ответа (`POST /api/v1/communications/messages/{messageId}/bilingual-reply-flow`).

### AI-состояние сообщения

Файл `aiState.ts`:

- `fetchMessageAiState` — получение AI-состояния (`GET /api/v1/communications/messages/{messageId}/ai-state`).
- `updateMessageAiState` — обновление AI-состояния (`PUT /api/v1/communications/messages/{messageId}/ai-state`).

### Провайдер-нейтральные каналы (Provider Channels)

Файл `providerChannels.ts` предоставляет REST-методы для работы с беседами, сообщениями и темами мессенджеров:

- `fetchCommunicationConversations` — список бесед (`GET /api/v1/communications/conversations?...`).
- `searchCommunicationConversations` — поиск бесед (`GET /api/v1/communications/conversations/search?...`).
- `fetchCommunicationConversationDetail` — информация о беседе (`GET /api/v1/communications/conversations/{conversationId}`).
- `fetchCommunicationConversationMembers` — участники беседы.
- `fetchCommunicationMessages` — сообщения (с фильтрами `account_id`, `conversation_id`, `channel_kind`) (`GET /api/v1/communications/messages?...`).
- `searchCommunicationMessages` — поиск сообщений (`GET /api/v1/communications/search/messages?...`).
- `fetchCommunicationPinnedMessages` — закреплённые сообщения.
- `fetchCommunicationRawEvidence` — сырые данные доказательства (`GET /api/v1/communications/messages/{messageId}/raw-evidence`).
- `fetchCommunicationTopics` / `searchCommunicationTopics` / `fetchCommunicationTopicMessages` — темы и сообщения тем.

### Прочее

- **Синхронизация почты**: реэкспорт из `shared/mailSync/syncApi` (файл не включён).
- **Отправка**: `sendEmail`, `redirectMessage` из `sendApi.ts` (файл не включён).
- **Блокировки архитектуры**: `fetchCommunicationBlockers` (`ListCommunicationBlockers`).
- **Персоны**: `fetchPersonas` (`ListCommunicationPersonas`).
- **Rich-шаблоны**: `fetchRichTemplates`, `saveRichTemplate`, `deleteRichTemplate`, `renderRichTemplate`, `previewRichTemplateMailMerge`.
- **Общая сводка почтового ящика**: `fetchMailboxHealth` (`GetMailboxHealth`).
- **Частые отправители**: `fetchTopSenders` (`ListTopSenders`).
- **Подписки**: `fetchSubscriptions` (`ListSubscriptions`).

## Транспортный слой

- `connect/shared.ts` содержит `postCommunicationsConnectJson` — универсальный POST-запрос к `makosh.communications.v1.CommunicationsService/{method}` с заголовком `X-Макошь-Secret`.
- ConnectRPC-клиент получается через `getCommunicationsConnectClient()` из `platform/connect/communicationsClient` (детали клиента не включены в этот чанк).
- Прямые REST-запросы используют `ApiClient.instance.get/post/put`, базовый URL и секрет конфигурируются в `ApiClient`.

## Маппинг типов

`connect/mapping.ts` предоставляет функции:

- `mapMessageSummary` — преобразует сообщение из ответа ConnectRPC в доменный тип (превью тела, нормализация статусов).
- `mapAttachment`, `mapOutboxItem`, `mapDraftItem`, `mapSavedSearchItem`, `mapFolderItem`, `mapFolderMessageItem`, `mapRichTemplate` — аналогичные маппинги.
- `mapMessageSummaryContract` — маппинг контракта анализа (ключевые пункты, действия и т.п.).
- Нормализация: `normalizeWorkflowState`, `normalizeLocalState`, `normalizeScanStatus`, `normalizeDisposition`, `normalizeOutboxStatus` и др.
- `parseJsonObject` — безопасный парсинг JSON-строки в объект.
```

---

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `frontend/src/domains/communications/api/aiState.ts` | `fetchMessageAiState`, `updateMessageAiState` и их REST-пути |
| `frontend/src/domains/communications/api/attachmentApi.ts` | `searchAttachments`, `inspectAttachmentArchive`, `previewAttachment`, `translateAttachment` и их делегирование в ConnectRPC-слой |
| `frontend/src/domains/communications/api/attachmentImportApi.test.ts` & `attachmentImportApi.ts` | Типы `CommunicationAttachmentImportRequest`/`Response`, функция `importCommunicationAttachment`, эндпоинт `POST /api/v1/communications/attachments/import` |
| `frontend/src/domains/communications/api/bilingualReplyFlow.test.ts` & `bilingualReplyFlow.ts` | `prepareBilingualReplyFlow`, запрос/ответ, эндпоинт `POST /api/v1/communications/messages/{messageId}/bilingual-reply-flow` |
| `frontend/src/domains/communications/api/callApi.test.ts` & `callApi.ts` | `fetchProviderCalls`, `fetchProviderCallTranscript`, REST-эндпоинты `/api/v1/calls?...` и `/api/v1/calls/{callId}/transcript` |
| `frontend/src/domains/communications/api/certificateApi.ts` | `fetchMailCertificates`, `fetchExpiringMailCertificates`, `createMailCertificate`, соответствующие REST-пути |
| `frontend/src/domains/communications/api/communications.test.ts` | Тесты для `fetchCommunicationMessages`, `transitionMessageWorkflowState`, `fetchMessageStateCounts`, `searchEmails`, `detectMessageLanguage`, `translateMessage`, `extractMessageTasks`, `extractMessageNotes`, `analyzeMessage`, `fetchMessageExplain`, `fetchMessageSmartCc`, `exportMessage`, `fetchMessageAuth`, `fetchMessageSignature`, `generateAiReply`, `generateAiReplyVariants` и многих других, подтверждающие ConnectRPC-методы и структуры ответов |
| `frontend/src/domains/communications/api/communications.ts` | Баррель-реэкспорты функций из `messageApi`, `savedSearchApi`, `folderApi`, `sendApi`, `outboxApi`, `threadApi`, `callApi`, `attachmentApi`, `certificateApi`, `aiState` и ConnectRPC-слоя |
| `frontend/src/domains/communications/api/communicationsAttachmentsFolders.test.ts` | Тесты для `searchAttachments`, `translateAttachment`, `inspectAttachmentArchive`, `previewAttachment`, а также CRUD папок и операций `copyMessageToFolder`/`moveMessageToFolder` через ConnectRPC |
| `frontend/src/domains/communications/api/connect/collections.ts` | ConnectRPC-реализации для тредов, вложений, черновиков, сохранённых поисков, папок, исходящих; маппинги в доменные типы |
| `frontend/src/domains/communications/api/connect/insights.ts` | ConnectRPC-реализации для `fetchMessageStateCountsConnect`, `fetchSubscriptionsConnect`, `fetchMailboxHealthConnect`, `fetchTopSendersConnect`, `fetchCommunicationBlockersConnect`, `fetchCommunicationPersonasConnect`, `fetchRichTemplatesConnect`, `saveRichTemplateConnect`, `deleteRichTemplateConnect`, `renderRichTemplateConnect`, `previewRichTemplateMailMergeConnect`, `searchMessagesConnect`, `detectMessageLanguageConnect`, `translateMessageConnect` |
| `frontend/src/domains/communications/api/connect/mapping.ts` | Функции маппинга: `mapMessageSummary`, `mapAttachment`, `mapOutboxItem`, `mapDraftItem`, `mapSavedSearchItem`, `mapFolderItem`, `mapFolderMessageItem`, `mapFolderMessageActionResult`, `mapRichTemplate`, `mapMessageSummaryContract`, `mapKnowledgeCandidates`, `parseJsonObject`, нормализация статусов |
| `frontend/src/domains/communications/api/connect/messageLifecycle.ts` | ConnectRPC-реализации для сообщений: `fetchCommunicationMessagesConnect`, `fetchCommunicationMessageConnect`, `transitionMessageWorkflowStateConnect`, `trashMessageConnect`, `restoreMessageConnect`, `markMessageReadConnect`, `deleteMessageFromProviderConnect`, `bulkMessageActionConnect`, `toggleMessagePinConnect`, `toggleMessageImportantConnect`, `toggleMessageMuteConnect`, `snoozeMessageConnect`, `addMessageLabelConnect`, `analyzeMessageConnect`, `runWorkflowActionConnect`, `fetchMessageExplainConnect`, `fetchMessageSmartCcConnect`, `exportMessageConnect`, `fetchMessageAuthConnect` (и другие) |
| `frontend/src/domains/communications/api/connect/shared.ts` | `postCommunicationsConnectJson` – POST-хелпер к `CommunicationsService` с заголовком `X-Макошь-Secret` |
| `frontend/src/domains/communications/api/connectCommunications.test.ts` | Обширные тесты всех ConnectRPC-функций, подтверждающие вызовы, запросы и ответы |
| `frontend/src/domains/communications/api/connectCommunications.ts` | Баррель-реэкспорты ConnectRPC-функций из `connect/messageLifecycle`, `connect/insights`, `connect/collections` |
| `frontend/src/domains/communications/api/folderApi.ts` | Публичное API папок, делегирующее в ConnectRPC-слой |
| `frontend/src/domains/communications/api/messageApi.ts` | Публичное API сообщений, шаблонов, персон, блокировок и др., делегирующее в ConnectRPC-слой |
| `frontend/src/domains/communications/api/outboxApi.ts` | `fetchOutboxItems`, `undoOutboxItem`, делегирование в ConnectRPC-слой |
| `frontend/src/domains/communications/api/providerChannels.boundary.test.ts` & `providerChannels.ts` | Функции провайдер-нейтральных каналов, их REST-эндпоинты, отсутствие в них провайдер-специфичных интеграционных путей |
| `frontend/src/domains/communications/api/readReceipts.test.ts` | `recordReadReceipt`, эндпоинт `POST /api/v1/communications/read-receipts` |

---

## Исходные файлы

- [`frontend/src/domains/communications/api/aiState.ts`](../../../../frontend/src/domains/communications/api/aiState.ts)
- [`frontend/src/domains/communications/api/attachmentApi.ts`](../../../../frontend/src/domains/communications/api/attachmentApi.ts)
- [`frontend/src/domains/communications/api/attachmentImportApi.test.ts`](../../../../frontend/src/domains/communications/api/attachmentImportApi.test.ts)
- [`frontend/src/domains/communications/api/attachmentImportApi.ts`](../../../../frontend/src/domains/communications/api/attachmentImportApi.ts)
- [`frontend/src/domains/communications/api/bilingualReplyFlow.test.ts`](../../../../frontend/src/domains/communications/api/bilingualReplyFlow.test.ts)
- [`frontend/src/domains/communications/api/bilingualReplyFlow.ts`](../../../../frontend/src/domains/communications/api/bilingualReplyFlow.ts)
- [`frontend/src/domains/communications/api/callApi.test.ts`](../../../../frontend/src/domains/communications/api/callApi.test.ts)
- [`frontend/src/domains/communications/api/callApi.ts`](../../../../frontend/src/domains/communications/api/callApi.ts)
- [`frontend/src/domains/communications/api/certificateApi.ts`](../../../../frontend/src/domains/communications/api/certificateApi.ts)
- [`frontend/src/domains/communications/api/communications.test.ts`](../../../../frontend/src/domains/communications/api/communications.test.ts)
- [`frontend/src/domains/communications/api/communications.ts`](../../../../frontend/src/domains/communications/api/communications.ts)
- [`frontend/src/domains/communications/api/communicationsAttachmentsFolders.test.ts`](../../../../frontend/src/domains/communications/api/communicationsAttachmentsFolders.test.ts)
- [`frontend/src/domains/communications/api/connect/collections.ts`](../../../../frontend/src/domains/communications/api/connect/collections.ts)
- [`frontend/src/domains/communications/api/connect/insights.ts`](../../../../frontend/src/domains/communications/api/connect/insights.ts)
- [`frontend/src/domains/communications/api/connect/mapping.ts`](../../../../frontend/src/domains/communications/api/connect/mapping.ts)
- [`frontend/src/domains/communications/api/connect/messageLifecycle.ts`](../../../../frontend/src/domains/communications/api/connect/messageLifecycle.ts)
- [`frontend/src/domains/communications/api/connect/shared.ts`](../../../../frontend/src/domains/communications/api/connect/shared.ts)
- [`frontend/src/domains/communications/api/connectCommunications.test.ts`](../../../../frontend/src/domains/communications/api/connectCommunications.test.ts)
- [`frontend/src/domains/communications/api/connectCommunications.ts`](../../../../frontend/src/domains/communications/api/connectCommunications.ts)
- [`frontend/src/domains/communications/api/folderApi.ts`](../../../../frontend/src/domains/communications/api/folderApi.ts)
- [`frontend/src/domains/communications/api/messageApi.ts`](../../../../frontend/src/domains/communications/api/messageApi.ts)
- [`frontend/src/domains/communications/api/outboxApi.ts`](../../../../frontend/src/domains/communications/api/outboxApi.ts)
- [`frontend/src/domains/communications/api/providerChannels.boundary.test.ts`](../../../../frontend/src/domains/communications/api/providerChannels.boundary.test.ts)
- [`frontend/src/domains/communications/api/providerChannels.ts`](../../../../frontend/src/domains/communications/api/providerChannels.ts)
- [`frontend/src/domains/communications/api/readReceipts.test.ts`](../../../../frontend/src/domains/communications/api/readReceipts.test.ts)

## Кандидаты на drift

Из предоставленного контекста явных расхождений не видно. Однако:

- Баррель `communications.ts` реэкспортирует функции из `sendApi`, `savedSearchApi`, `threadApi` и `shared/mailSync/syncApi` – эти файлы не включены в чанк, их поведение и соответствие тестам не могут быть подтверждены.
- Тест `communicationsAttachmentsFolders.test.ts` содержит кейс для `redirectMessage`, реализация которого находится в `./sendApi` (не предоставлен); возможна организационная несогласованность размещения теста, но без реализации подтвердить drift невозможно.
- Часть файлов обрезана (помечены `yes`), поэтому полное покрытие утверждений не может быть гарантировано для обрезанных участков.

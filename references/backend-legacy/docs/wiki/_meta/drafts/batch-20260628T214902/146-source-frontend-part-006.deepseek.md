### Summary / Резюме

Обновляется страница `components/frontend.md` русской Obsidian-вики – добавляется описание фронтенд-модуля коммуникаций: формы сохранённых поисков и шаблонов, хелперы моделей страниц, система запросов на основе TanStack Query, а также панель WhatsApp. Содержимое основано исключительно на предоставленных исходных файлах.

### Proposed pages / Предлагаемые страницы

#### `components/frontend.md`

```markdown
# Компоненты фронтенда (Communications)

## Обзор

Фронтенд приложения `makosh` включает домен `communications`, отвечающий за работу с почтовыми сообщениями, чатами (WhatsApp) и шаблонами. Домен реализован на Vue 3 с TanStack Query, Zod и Vee-Validate.

## Формы

### Сохранённые поиски (Saved Search Form)

Форма позволяет создавать и редактировать сохранённые поиски и смарт-папки.

- **Схема** (`savedSearchFormSchema` из `frontend/src/domains/communications/forms/savedSearchForm.ts`): поля `name` (обязательно, до 120 символов), `description`, `query`, `workflow_state`, `local_state`, `channel_kind`, `is_smart_folder`, `match_mode` (по умолчанию `'all'`). Все строковые поля обрезаются.
- **Каналы** (`savedSearchChannelOptions`): `Email`, `Telegram`, `Any channel`. Telegram доступен как полноценный канал сохранённого поиска.
- **Пресеты** (`savedSearchPresetOptions`): три пресета только для email: `Needs action` (`workflow_state: 'needs_action'`), `Waiting` (`workflow_state: 'waiting'`), `Spam review` (`workflow_state: 'spam'`). Во всех установлено `is_smart_folder: true`, `local_state: 'active'`.
- **Конвертация в запрос** (`savedSearchFormToInput`): приводит значения формы к `SavedSearchInput` – обрезает строки, преобразует пустые `account_id`/`channel_kind` в `null`.
- **Чипсы фильтров** (`savedSearchFilterChips`): из текущих значений формы и опциональных правил формирует набор чипсов: текст запроса, правила (например, «Sender contains»), Workflow, Scope, Match, Channel, Mode (Smart folder / Saved search).
- **Диалог удаления** (`savedSearchDeleteDialogCopy`): формирует заголовок и сообщение, различая смарт-папки и сохранённые поиски.
- **Счётчик сообщений** (`savedSearchMessageCountLabel`): возвращает целое неотрицательное число сообщений в виде строки.

#### Дерево правил (`savedSearchRuleTree.ts`)

Модуль реализует парсинг, построение и валидацию логических правил поиска.

- **Типы**: `SavedSearchRule` (поле, оператор `:` или `=`, значение), `SavedSearchRuleCondition` (с `id`, `kind: 'rule'`), `SavedSearchRuleGroup` (с `id`, `kind: 'group'`, `matchMode`, список дочерних узлов), `SavedSearchRuleNode`.
- **Парсинг** (`parseSavedSearchQuery`):
  - Извлекает текстовый запрос, правила вида `subject:value`, `from:value`, `body:value`, `all=value`.
  - Распознаёт режим `mode:any`/`mode:all`.
  - Поддерживает явные логические выражения со скобками, `AND`, `OR`; результат возвращается в виде дерева.
- **Сборка запроса** (`composeSavedSearchQuery`): объединяет `plainQuery`, массив правил и режим. Значения с пробелами или кавычками оборачиваются в двойные кавычки.
- **Дерево в строку** (`composeSavedSearchRuleTreeQuery`): рекурсивно форматирует дерево групп, вставляя `AND`/`OR` и скобки.
- **Нормализация** (`normalizeSavedSearchBuilderState`): объединяет правила, полученные парсингом сырого запроса, с существующими, устраняет дубликаты.
- **Валидация**:
  - `validateSavedSearchRules` проверяет отсутствие пустых правил и дубликатов.
  - `validateSavedSearchRuleTree` проверяет наличие хотя бы одного правила или группы во всём дереве.
- **Вспомогательные функции**: `createSavedSearchRuleCondition`, `createSavedSearchRuleGroup`, `flattenSavedSearchRuleTree`.

### Шаблоны (Template Form)

Управление шаблонами писем с переменными и mail merge.

- **Схема** (`templateFormSchema` из `frontend/src/domains/communications/forms/templateForm.ts`): только имя (обязательно, до 120 символов, обрезается).
- **Извлечение переменных**:
  - `extractTemplateVariables` и `templateContentDiagnostics`: сканируют переданные строки (subject, body, bodyHtml) на плейсхолдеры `{{ variable }}`. Допустимые символы в имени: `[A-Za-z0-9_.-]`.
  - Возвращают список уникальных переменных и список некорректных плейсхолдеров (`malformedPlaceholders`).
- **Проверка заполнения**:
  - `missingTemplateVariables`: возвращает переменные, для которых значение пустое.
  - `templateMergeErrorMessage`: сообщение перед сохранением (например, «Fill template variables: project»).
  - `templateDiagnosticsErrorMessage`: сообщение при наличии `malformedPlaceholders`.
  - `storedTemplateDiagnosticMessages`: на основе сохранённых метаданных (`malformed_placeholders`, `undeclared_variables`, `unused_variables`) строит массив сообщений с уровнями `error`/`warning`.
- **Значения по умолчанию** (`defaultTemplateVariableValue`): для переменных `recipient`, `to`, `cc`, `bcc`, `subject`, `body`, `message`, `date`, `current_date` подставляет соответствующие поля контекста письма или текущую дату.
- **Разрешение переменных** (`resolveTemplateVariableValues`): заполняет все переменные шаблона, либо сохраняя существующие значения (`preserveExisting: true`), либо вычисляя дефолты.
- **Mail merge preview**:
  - `parseTemplateMailMergePreviewRows`: парсит JSON-массив объектов; каждый объект должен содержать `variables` (или поля верхнего уровня трактуются как переменные). Генерирует `row_id`, если не задан.
  - `stringifyTemplateMailMergePreviewRows`: сериализует обратно в JSON.
- **Сохранение** (`templateFormToInput`): собирает `RichTemplateUpsertRequest` – в `body_template` идёт `bodyHtml` (если есть), иначе `body`; `variables` извлекаются диагностикой; опционально добавляется `template_id`.

## Хелперы страниц (Communication Page Models)

Модуль `frontend/src/domains/communications/helpers/communicationPageModels.ts` содержит фабрики compose-форм и модели AI-инсайтов.

### Compose-формы

- **Создание**:
  - `newComposeForm(accountId, draftId)` – пустая форма нового письма.
  - `replyComposeForm(message, fallbackAccountId, draftId)` – ответ отправителю, тема с префиксом `Re:`, `inReplyTo` = `provider_record_id`.
  - `replyAllComposeForm` – как reply, но добавляет всех получателей исходного письма в `ccText`.
  - `forwardComposeForm` – пересылка, тема с префиксом `Fwd:`, тело включает заголовок «Forwarded message».
  - `draftToComposeForm` – восстанавливает черновик из `CommunicationDraft` в модель формы (включая scheduled send).
  - `threadReplyComposeForm` – ответ в треде с цитированием исходного сообщения в plain text и HTML; HTML-теги и спецсимволы экранируются.

- **Отправка** (`composeFormToSendRequest`): преобразует модель в `SendCommunicationRequest`, разбирает получателей из строк (через `splitComposeRecipients`), учитывает отложенную отправку и undo send.

### AI-инсайты

- **`emptyCommunicationMessageInsight(messageId)`** – создаёт пустую структуру с полями `messageId`, `explain`, `smartCc`, `auth`, `signature`, `language`, `aiReply`, `tasks`, `notes`, `translation`.
- **`aiSummaryContractFromMetadata(metadata)`** – извлекает из `metadata.ai_summary_contract` объект с нормализованными полями:
  - `key_points`, `action_items`, `risks`, `deadlines` – фильтруются строки.
  - Пять типов кандидатов: `event_candidates`, `persona_candidates`, `organization_candidates`, `document_candidates`, `agreement_candidates`. Каждый кандидат имеет `title` и `evidence`; строчные значения преобразуются в объект с `title = evidence = значение`.
  - При отсутствии или некорректном формате возвращается `null`.
- **`communicationKnowledgeSectionsFromSummaryContract`** – из `AiSummaryContract` формирует секции для UI (event, persona, organization, document, agreement), исключая пустые.
- **`communicationExtractionSectionsFromInsight`** – из `CommunicationMessageInsight` формирует секции `task` (задачи) и `note` (заметки) с заголовками, метаданными и текстом источника.

### Метки и snooze

- **`communicationMessageLabelsFromMetadata`** – извлекает уникальные непустые строки из `metadata.labels`.
- **`communicationMessageSnoozeUntilFromMetadata`** – возвращает строку `snooze_until` или `null`, если она не является строкой.

## Запросы (Queries)

### Политики обновления (`communicationQueryPolicies.ts`)

Определены три политики для TanStack Query:

| Политика                        | staleTime | refetchInterval | refetchOnReconnect | refetchOnWindowFocus |
| ------------------------------- | --------- | --------------- | ------------------ | -------------------- |
| `communicationRealtimeQueryOptions` | 10 с      | 60 с            | `true`             | `true`               |
| `communicationDetailQueryOptions`  | 30 с      | —               | `true`             | `true`               |
| `communicationReferenceQueryOptions` | 5 мин    | —               | `true`             | `false`              |

### Основные запросы (`mailCoreQueries.ts`)

- **`useMailListQuery`** – infinite-запрос с курсорной пагинацией (250 элементов на страницу); параметры: `accountId`, `workflowState`, `channelKind`, `query`, `localState`.
- **`useMessageQuery`** – детали одного сообщения по `messageId`.
- **`useMessageAiStateQuery`** – AI-состояние сообщения.
- **`useStateCountsQuery`** – количество сообщений по workflow-состояниям.
- **`useSyncStatusesQuery`** – статусы синхронизации почтовых ящиков.
- **`useMailSyncSettingsQuery` / `useUpdateMailSyncSettingsMutation`** – получение и обновление настроек синхронизации.
- **`useMailboxHealthQuery`** – здоровье почтового ящика.
- **`useConversationsQuery`** – infinite-запрос тредов (по 50 элементов).
- **`useThreadMessagesQuery`** – сообщения конкретного треда по `accountId` и `subject`.
- **`usePersonasQuery`** – список персон (справочные данные).

### Экшены (`mailActionQueries.ts`)

Мутации для действий над сообщениями; после успешного выполнения инвалидируются кеши сообщения и списков (при необходимости – синхронизации).

- Флаги: `useToggleMessagePinMutation`, `useToggleMessageImportantMutation`, `useToggleMessageMuteMutation`
- Чтение: `useMarkMessageReadMutation`, `useMarkMessageUnreadMutation` (через `bulkMessageAction`)
- Удаление на провайдере: `useDeleteMessageFromProviderMutation`
- Экспорт: `useExportMessageMutation` (формат `MessageExportFormat`)
- Метки: `useAddMessageLabelMutation`, `useRemoveMessageLabelMutation`
- Snooze: `useSnoozeMessageMutation`
- AI: `useAnalyzeMessageMutation`, `useGenerateAiReplyMutation` (tone + language), `useGenerateAiReplyVariantsMutation` (languages + tones), `useExplainMessageMutation`, `useDetectMessageLanguageMutation`
- Безопасность: `useReviewMessageSecurityMutation` (auth + signature)
- Получатели: `useReviewMessageRecipientsMutation` (Smart Cc)
- Перевод: `useTranslateMessageMutation`, `useTranslateThreadMutation`
- Извлечение: `useExtractMessageTasksMutation`, `useExtractMessageNotesMutation`
- Синхронизация: `useRunMailSyncNowMutation`, `useRunMailFullResyncMutation`

### Предварительная загрузка (`communicationPrefetch.ts`)

- Ключи кеша: `communicationMessageQueryKey`, `communicationListQueryKey`, `threadMessagesQueryKey`.
- Функции `prefetchCommunicationMessage`, `prefetchCommunicationMessageForAttachmentResult`, `prefetchThreadMessages`, `prefetchCommunicationListForSavedSearch` предзагружают данные с `staleTime = 30 000 мс`.
- Хуки-обёртки: `useCommunicationMessagePrefetch`, `useAttachmentSearchResultPrefetch`, `useThreadMessagesPrefetch`, `useSavedSearchCommunicationListPrefetch`.

### Черновики (`draftsInfiniteQuery`)

- Используется `useInfiniteQuery<DraftListResponse>` с курсором `next_cursor`, размер страницы 50; параметр `accountId`.

### Папки (`folderMailList.ts`)

- `folderMessagesToMailSummaries` маппит `FolderMessage` в `CommunicationMessageSummary` (все как email, delivery_state = `'folder'`).
- `useFolderMailList` принимает `folderId`, использует `useFolderMessagesQuery`, возвращает реактивный список сообщений.

### Звонки (`callQueries.ts`)

- `useProviderCallsQuery` – список звонков (параметры: `accountId`, `limit`, `provider`).
- `useProviderCallTranscriptQuery` – транскрипт звонка по `callId`.

### Сертификаты

- Подтверждено boundary-тестом (`mailCertificates.boundary.test.ts`): хуки `useMailCertificatesQuery`, `useExpiringMailCertificatesQuery`, `useCreateMailCertificateMutation` оборачивают API-функции `fetchMailCertificates`, `fetchExpiringMailCertificates`, `createMailCertificate`.

## WhatsApp-панель

### Хелперы (`WhatsAppCommunicationsPanel.helpers.ts`)

- **Тип `WhatsAppPanelMessage`** – расширенная модель с полями для чата, статуса и медиа.
- **Отображение**:
  - `messageTime` форматирует дату/время (локаль `en`, месяц + день + часы:минуты).
  - `memberLabel` – имя участника.
  - `mediaLabel`, `mediaAttachmentId`, `isPreviewableMediaItem`, `firstPreviewableMediaAttachmentId`, `mediaMetaLabel`, `mediaTime` – работа с медиа-элементами.
- **Статус-сообщения**:
  - `isStatusMessage` – определяет по `communication_object_type === 'status'` или `provider_chat_id === 'status-feed'`.
  - `statusMessageMediaItems` – медиа, привязанные к сообщению.
  - `statusAuthorHeadline`, `statusAuthorDetail` – данные автора статуса.
  - `statusViewSummary` – просмотры и последний зритель.
  - `statusDeletedSummary` – информация об удалении.
  - `statusMediaCountLabel` – количество медиа.
- **Мета-флаги** (`messageMetaFlags`): `@N` (упоминания), View once, Ephemeral, Sticker, Poll, Location, Contact card, System, Membership, Link, Status.
- **Детализация сообщений**:
  - `messagePreview` – текст или превью.
  - `messageMentionNames` – до 5 упоминаний.
  - `messageLinkPreview` – title, url, site.
  - `messagePollSummary` – опрос (вопрос, количество опций).
  - `messageLocationSummary` – координаты / адрес.
  - `messageContactCardSummary` – имя и телефон.
  - `messageStickerSummary` – эмодзи / название пака.
  - `messageSystemSummary` – текст системного сообщения.
  - `reactionSummary` – реакции (emoji, количество).

### UI-элементы (boundary-тест)

Подтверждено наличие строк в компонентах панели:
- Пересылка: `Forward target`, `Filter target conversations`, `Forward here`.
- Редактирование: `Edit draft`, `Edited text`, `Save edit`.
- Навигация: `Jump to message`, `Open source message`, `browserMode` (Timeline, All media, Images, Videos, Documents).
- Медиа-превью: `Media preview`, `Preview media`, поддержка audio/video/pdf.
- Метаданные: `whatsapp_link_preview`, `whatsapp_poll`, `whatsapp_location`, `whatsapp_contact_card`, `whatsapp_sticker`, `whatsapp_view_once`, `whatsapp_ephemeral`.
- Статусы: `Status author`, `Status media`, `status_view_count`, `status_last_viewer_display_name`, `status_deleted_at`, `status_author_business_profile`.
```

### Source coverage / Покрытие источников

| Источник | Покрытые факты |
| -------- | -------------- |
| `frontend/src/domains/communications/forms/savedSearchForm.ts` | Схема, опции каналов, пресеты, `savedSearchFormToInput`, `savedSearchFilterChips`, `savedSearchDeleteDialogCopy`, `savedSearchMessageCountLabel`, реэкспорт из rule tree. |
| `frontend/src/domains/communications/forms/savedSearchFormOptions.ts` | Списки состояний workflow, local, метки (labels) для состояний и режимов. |
| `frontend/src/domains/communications/forms/savedSearchRuleTree.ts` (truncated at 12000 chars) | Типы `SavedSearchRule`, `SavedSearchRuleGroup`, `SavedSearchRuleCondition`; `parseSavedSearchQuery`, `composeSavedSearchQuery`, `normalizeSavedSearchBuilderState`, `composeSavedSearchRuleTreeQuery`, `validateSavedSearchRules`, `validateSavedSearchRuleTree`, `flattenSavedSearchRuleTree`, `createSavedSearchRuleCondition`, `createSavedSearchRuleGroup`. |
| `frontend/src/domains/communications/forms/savedSearchForm.test.ts` | Поведение схемы (normalize, reject empty name), Telegram channel option, delete dialog copy, message count, filter chips (including match mode `any`), parsing rules, composing query, any-match mode, normalize builder state, nested rule groups, effective query resolution, validation (empty, duplicate). |
| `frontend/src/domains/communications/forms/templateForm.ts` | Схема, `extractTemplateVariables`, `templateContentDiagnostics`, `missingTemplateVariables`, сообщения диагностики, `defaultTemplateVariableValue`, `resolveTemplateVariableValues`, `parseTemplateMailMergePreviewRows`, `stringifyTemplateMailMergePreviewRows`, `templateFormToInput`. |
| `frontend/src/domains/communications/forms/templateForm.test.ts` | Поведение схемы, извлечение переменных, конвертация в запрос, rejection пустого имени, missing variables, malformed placeholders, stored diagnostics, default values, resolve variables, mail merge preview parse/stringify. |
| `frontend/src/domains/communications/helpers/communicationPageModels.ts` | `emptyCommunicationMessageInsight`, `communicationExtractionSectionsFromInsight`, `communicationKnowledgeSectionsFromSummaryContract`, `communicationMessageLabelsFromMetadata`, `communicationMessageSnoozeUntilFromMetadata`, `aiSummaryContractFromMetadata`, `replyComposeForm`, `replyAllComposeForm`, `forwardComposeForm`, `newComposeForm`, `threadReplyComposeForm`, `composeFormToSendRequest`, `draftToComposeForm`. |
| `frontend/src/domains/communications/helpers/communicationPageModels.test.ts` | Поведение всех вышеперечисленных функций с данными примеров. |
| `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.boundary.test.ts` | Наличие UI-строк и элементов: forward target selector, edit flow, jump to message, browserMode (Timeline, All media, Images, Videos, Documents), media preview, WhatsApp metadata (link_preview, poll, location, contact_card, sticker, view_once, ephemeral), Status author и Status media. |
| `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.helpers.ts` | Тип `WhatsAppPanelMessage`, хелперы `messageTime`, `memberLabel`, `mediaLabel`, `mediaAttachmentId`, `isPreviewableMediaItem`, `firstPreviewableMediaAttachmentId`, `mediaMetaLabel`, `mediaTime`, функции для статус-сообщений, `messageMetaFlags`, `messagePreview`, `messageMentionNames`, `messageLinkPreview`, `messagePollSummary`, `messageLocationSummary`, `messageContactCardSummary`, `messageStickerSummary`, `messageSystemSummary`, `reactionSummary`, `isStatusMessage`. |
| `frontend/src/domains/communications/queries/communicationQueryPolicies.ts` | Три политики: `communicationRealtimeQueryOptions`, `communicationDetailQueryOptions`, `communicationReferenceQueryOptions` с конкретными значениями staleTime, refetchInterval, refetchOnReconnect, refetchOnWindowFocus. |
| `frontend/src/domains/communications/queries/mailCoreQueries.ts` | `useMailListQuery`, `useMessageQuery`, `useMessageAiStateQuery`, `useStateCountsQuery`, `useSyncStatusesQuery`, `useMailSyncSettingsQuery`, `useUpdateMailSyncSettingsMutation`, `useMailboxHealthQuery`, `useConversationsQuery`, `useThreadMessagesQuery`, `usePersonasQuery`. |
| `frontend/src/domains/communications/queries/mailActionQueries.ts` | Все мутации действий над сообщениями, функция `invalidateMessageViews`/`invalidateSyncViews`, перечисление инвалидируемых ключей. |
| `frontend/src/domains/communications/queries/communicationPrefetch.ts` | Ключи кеша, функции `prefetchCommunicationMessage`, `prefetchCommunicationMessageForAttachmentResult`, `prefetchThreadMessages`, `prefetchCommunicationListForSavedSearch`, соответствующие хуки. |
| `frontend/src/domains/communications/queries/communicationPrefetch.test.ts` | Поведение всех prefetch-функций с проверкой вызовов API и заполнения кеша. |
| `frontend/src/domains/communications/queries/draftsInfiniteQuery.boundary.test.ts` | Наличие `useInfiniteQuery` с курсорной пагинацией для drafts. |
| `frontend/src/domains/communications/queries/folderMailList.ts` | `folderMessagesToMailSummaries`, `useFolderMailList`. |
| `frontend/src/domains/communications/queries/folderMailList.test.ts` | Маппинг `FolderMessage` → `CommunicationMessageSummary`. |
| `frontend/src/domains/communications/queries/callQueries.ts` | `useProviderCallsQuery`, `useProviderCallTranscriptQuery`. |
| `frontend/src/domains/communications/queries/aiReplyVariants.boundary.test.ts` | Наличие TanStack mutation для `generateAiReplyVariants`. |
| `frontend/src/domains/communications/queries/attachmentTranslationMutation.boundary.test.ts` | Наличие TanStack mutation для `translateAttachment`. |
| `frontend/src/domains/communications/queries/mailCertificates.boundary.test.ts` | Наличие хуков для сертификатов: `useMailCertificatesQuery`, `useExpiringMailCertificatesQuery`, `useCreateMailCertificateMutation`. |
| `frontend/src/domains/communications/queries/communicationQueryPolicies.boundary.test.ts` | Применение политик в `mailCoreQueries`, `mailWorkspaceQueries`, `mailOperationQueries`. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не видно. Boundary-тесты WhatsApp проверяют наличие конкретных строк в соответствующих `.vue` файлах, но без полных исходников панели невозможно подтвердить или опровергнуть их актуальность — данный факт сам по себе не квалифицируется как drift в рамках этого контекст-пакета.

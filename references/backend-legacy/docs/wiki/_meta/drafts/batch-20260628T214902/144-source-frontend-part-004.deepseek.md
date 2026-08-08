### Summary / Резюме

Целевая страница `components/frontend.md` русской Obsidian‑wiki должна быть дополнена или создана для документирования frontend‑компонентов и утилит домена `communications`. В предоставленном контексте находятся исходные файлы граничных тестов, описывающие контракты компонентов (пропсы, события, зависимости от TanStack Query/Virtual, отсутствие прямых API‑вызовов), а также утилиты для работы с вложениями, drag‑and‑drop, упорядочиванием папок и статусами исходящих. Wiki‑страница должна отражать эти факты на русском языке без выдумывания деталей, не подтверждённых исходниками.

### Proposed pages / Предлагаемые страницы

#### `components/frontend.md`

```markdown
# Frontend компоненты

Документирование frontend‑компонентов и утилит домена `communications` на основе исходных файлов и граничных тестов.

## MailResourceOverviewStrip

Отрисовывает обзор ресурсов почтового ящика. Использует виртуальный скроллинг через `@tanstack/vue-virtual` (`useVirtualizer`).

**Отображаемые секции:**
- `subscriptions` – подписки
- `topSenders` – частые отправители
- `blockers` – блокировщики

**Состояние и действия:**
- `hasMoreSubscriptions`, `hasMoreTopSenders` – флаги наличия дополнительных записей
- `loadMoreSubscriptions`, `loadMoreTopSenders` – загрузка следующих порций

Компонент не содержит прямых вызовов API (нет `fetch(`, `ApiClient`).

## MessageAiReplyPanel

Панель «AI Reply Review» для управления генерацией ответа с помощью ИИ.

**Элементы:**
- выбор тона `selectedAiReplyTone`
- выбор языка `selectedAiReplyLanguage`
- функция `generateAiReply`

**Взаимодействие с данными:** использует `useGenerateAiReplyVariantsMutation`, операции `generateVariants` и `applyAiReply`, а также реактивную переменную `replyVariants`.

Прямые обращения к API отсутствуют.

## MessageAttachmentsTab

Вкладка просмотра и перевода вложений сообщения.

**Запросы (TanStack Query):**
- `useAttachmentArchiveInspectionQuery`
- `useAttachmentPreviewQuery`
- `useTranslateAttachmentMutation`

**Данные перевода:** `attachmentTranslationTarget`, `attachmentTranslationResult`, `attachmentTranslationError`.

**Функции:** `translateSelectedAttachment`.

**Проверки типов вложений:**
- `isInspectableArchiveAttachment` – можно инспектировать архив
- `isPreviewableImageAttachment`
- `isPreviewablePdfAttachment`
- `isPreviewableAttachment`

**UI‑элементы:**
- панель `attachment-translation-panel` («Attachment translation»)
- кнопка «Inspect archive»
- секция «Attachment preview» с вариантами: `attachment-preview-image`, `attachment-preview-media`, `attachment-preview-document`
- условие `attachmentPreview.preview_kind === 'pdf'`
- использование `attachmentPreview.data_url`

При переводе передаётся `source_text: preview.text`.

Прямых API‑вызовов нет.

## MessageBodyTab

Основная вкладка тела сообщения с поддержкой билингвального ответа и AI‑фич.

**Билингвальный ответ:**
- Компонент `BilingualReplyPanel`
- `isBilingualReplyOpen`, `sendBilingualReply`
- Передаётся `messageId`

**AI-суммаризация:**
- `aiSummaryContractFromMetadata`, `summaryContract`
- Отображение секций: «Key points», «Action items», «Risks», «Deadlines»

**Извлечение знаний:**
- `communicationExtractionSectionsFromInsight`
- `communicationKnowledgeSectionsFromSummaryContract`
- `extractionSections`, `knowledgeSections`
- Панели «Extraction Review» и «Knowledge Review»

**Вложенные компоненты:**
- `MessageAiReplyPanel`
- `MessageTrustReviewPanel`
- `MessageLocalIntelligencePanel`

**Безопасность изображений:**
- `remoteImageUrls`, `shouldLoadRemoteImages`
- Прокси `remoteImageProxyUrl`
- Сообщение «Remote images blocked» и ссылка `/remote-image?url=`

Прямые вызовы API отсутствуют.

## MessageLocalIntelligencePanel

Панель локального анализа сообщения.

**Используемые мутации:**
- `useExplainMessageMutation`
- `useDetectMessageLanguageMutation`

**Отображаемые элементы:**
- «Importance» (важность)
- «Detect language» (определение языка)
- «Why this matters» (почему это важно)

Прямых API‑вызовов нет.

## MessageRelatedTab

Вкладка действий, связанных с сообщением: экспорт, метки, отложенная отправка, перенаправление.

**Экспорт:**
- Функция `exportMessage`
- Поддерживаемые форматы: `'md'` (Markdown), `'eml'` (EML), `'json'` (JSON)

**Метки и откладывание:**
- `communicationMessageLabelsFromMetadata`
- `communicationMessageSnoozeUntilFromMetadata`
- `addLabel`, `removeLabel`
- `snoozeMessage`, кнопка «Snooze», «Follow up»

**Операции:**
- `replyAll`, `forwardMessage`, `redirectMessage`
- `markMessageRead`, `markMessageUnread`, `deleteFromProvider`
- Отображение кнопок «Reply All», «Forward», «Redirect», «Read / Delete»
- Переменная `redirectRecipientsText`

**События:**
- `emit('markMessageRead')`
- `emit('markMessageUnread')`
- `emit('deleteFromProvider')`

Прямые вызовы API отсутствуют.

## MessageTrustReviewPanel

Панель проверки безопасности и предложений по получателям.

**Разделы:**
- «Security Review»
- «Recipient Suggestions»

**Данные:** `reviewSecurity` (включая `authRisk`), `reviewRecipients` (включая `smartCc`).

Прямых API‑вызовов нет.

## OutboxStatusStrip

Полоса состояния исходящих сообщений. Отрисовывает существующие данные запросов, не содержит собственной логики API или кэша.

**Входные данные:**
- `visibleOutboxStatusItems` – подготовленный список элементов
- `outboxStatusPresentation` – представление статуса

**События:**
- `undo: [outboxId: string]`
- `loadMore: []`
- `prefetchMore: []` – вызывается по `@mouseenter` и `@focus`
- Условие `v-if="hasMore"` для кнопки «Load more»

Внутри компонента нет `fetch(`, `ApiClient`, `useQuery`.

## RichComposeEditor

Редактор форматированного текста для создания писем, построенный на TipTap.

**Рантайм:**
- Импорт `richComposeExtensions` из `./richComposeExtensions`
- Использует `useEditor`, компонент `EditorContent`
- **Не** использует `contenteditable="true"` или `document.execCommand`
- **Не** создаёт собственные `Node.create` / `Mark.create`

**Команды форматирования:**
- `'paragraph'`, `'heading2'`, `'heading3'`
- `'alignLeft'`, `'alignCenter'`, `'alignRight'`
- `'bold'`, `'italic'`
- `'bulletList'`, `'orderedList'`
- `'blockquote'`, `'link'`, `'unlink'`
- `linkHref` и `updateAttributes`

**Схема расширений (файл `richComposeExtensions.ts`):**
- Импорт из `@tiptap/vue-3`
- Расширения: `RichHeading`, `RichLink`, `RichOrderedList`, `RichBlockquote`
- Нормализация: `normalizeMailComposeLinkHref`, `normalizeMailComposeTextAlign`, `getSafeTextAlignAttributes`
- Безопасность ссылок: `rel: 'noopener noreferrer'`, `target: '_blank'`

**Безопасность вставки/перетаскивания:**
- Функция `sanitizeMailComposePastedHtml`
- Обработчики `handlePaste`, `handleDrop` с `event.preventDefault()`
- Вставка очищенного HTML через `insertContent(sanitizeMailComposePastedHtml)`

**Перетаскивание файлов:**
- Событие `'attachments-dropped': [files: File[]]`
- Файлы из `event.dataTransfer.files` передаются наружу, а не вставляются в HTML

## SavedSearchRuleGroupEditor

Редактор групп правил сохранённого поиска. Отрисовывает свёрнутую структурную сводку.

**Входные данные:**
- Презентация из `./savedSearchRuleTreePresentation`
- `savedSearchRuleGroupDepthLabel`, `savedSearchRuleGroupSummary`

**Элементы:**
- `saved-search-group-builder-summary`
- Глубина через `:depth="nextDepth()"`
- Подпись: `{{ isRoot ? 'Match' : 'Group match' }}`

Прямые вызовы API отсутствуют.

## SavedSearchStrip

Панель сохранённых поисков с виртуальным горизонтальным скроллингом.

**Виртуализация:**
- `@tanstack/vue-virtual`, `useVirtualizer` с `horizontal: true`
- Два виртуальных списка: `virtualSmartFolders` и `virtualSavedSearches`
- Размеры: `smartFolderVirtualTotalSize`, `savedSearchVirtualTotalSize`
- Подгрузка: `fetchNextPage: fetchNextSmartFolderPage` / `fetchNextPage: fetchNextSavedSearchPage`

**Фильтры и состояние:**
- `savedSearchFilterChips` – чипсы фильтров
- Преобразование: `normalizeSavedSearchBuilderState`, `composeSavedSearchRuleTreeQuery`, `validateSavedSearchRuleTree`
- Пресеты: `savedSearchPresetOptions`, `applyPreset(preset)`
- Опции: `savedSearchWorkflowOptions`, `savedSearchLocalStateOptions`, `savedSearchChannelOptions`
- Текущие значения: `currentQuery`, `currentWorkflowState`, `currentLocalState`, `currentChannelKind`, `currentSearchDefaults`
- Активные чипсы: `activeFilterChips`

**Построитель правил:**
- `searchRuleTree` через `normalizeQueryIntoBuilder(formValues.query)`
- Вычисляемое `effectiveQueryPreview` и `ruleValidation = computed(() => validateSavedSearchRuleTree(searchRuleTree.value))`
- Компонент `SavedSearchRuleGroupEditor`
- Сообщение об ошибке: `saved-search-rule-error`
- Кнопка сохранения: `:disabled="isSaving || !ruleValidation.isValid"`

**Предзагрузка и скроллинг:**
- `useSavedSearchCommunicationListPrefetch`
- `handleSavedSearchPrefetch` по `@mouseenter` / `@focus`
- `handleSmartFolderVirtualScroll`, `handleSavedSearchVirtualScroll` по `@scroll`

Прямые вызовы API отсутствуют.

## ThreadAttachmentInsightPanel

Панель инсайтов по вложению в треде.

**Запросы:** те же, что и в `MessageAttachmentsTab`:
- `useAttachmentArchiveInspectionQuery`
- `useAttachmentPreviewQuery`
- `useTranslateAttachmentMutation`

**Отображаемые действия:**
- «Translate preview»
- «Inspect archive»
- «Attachment preview» / «Archive inspection» / «Thread attachment translation»

Проверки: `isPreviewableAttachment`, `isInspectableArchiveAttachment`, `isPreviewableImageAttachment`.

Прямых API‑вызовов нет.

## ThreadConversationView

Вид переписки в формате беседы. Отрисовывает список сообщений треда без собственных fetch‑запросов.

**Пропсы:**
- `thread: CommunicationThreadSummary`
- `messages: ThreadMessage[]`
- `isSendingReply: boolean`

**Управление развёртыванием:**
- `expandedMessageIds`, `autoExpandedThreadId`
- `canExpandAllMessages`, `expandAllMessages`
- `canCollapseAllMessages`, `collapseAllMessages`
- `expansionSummary`, `expandedMessageCount`

**Цитируемое содержимое:**
- `showQuotedContent`, `hasQuotedMessages`
- `message-quoted` блок, отображаемый по условию

**Inline‑ответы:**
- `activeReplyMessageId`, `activeReplyDraftId`, `inlineReplyHtml`
- `startInlineReply`, `cancelInlineReply`
- `continueReplyInCompose`, `saveInlineReplyDraft`, `sendInlineReply`
- Компонент `ThreadInlineReplyComposer`
- События: `@save-draft`, `@continue-in-compose`, `@send`

**Перевод треда:**
- Хук `useTranslateThreadMutation`
- `threadTranslationTarget`, `threadTranslationResult`, `threadTranslationError`
- `translatedThreadCount`, `translatedTextForMessage`
- Функция `handleTranslateThread`
- Панель `thread-translation-panel` («Thread translation review»)

**Вложения:** компонент `ThreadAttachmentInsightPanel` для каждого вложения сообщения.

**События наружу:**
- `openMessage: [messageId: string]`
- `replyToMessage: [message: ThreadMessage, bodyHtml: string, draftId: string]`
- `saveReplyDraft` и `sendReply` с аналогичными параметрами

**Дополнительно:**
- `formatAttachmentSize`, `scanStatusClass`
- `previewThreadMessageBody`, `splitThreadMessageBody`
- Данные `message.body_text`

Прямых вызовов `fetch(` или `ApiClient` нет.

## ThreadInlineReplyComposer

Редактор быстрого ответа прямо в треде.

**Пропсы:**
- `message: ThreadMessage`
- `bodyHtml: string`
- `isSendingReply: boolean`

**Интеграция:**
- Использует `RichComposeEditor`
- Двусторонняя привязка `v-model:body-html`

**Режим ревью:** `reviewingReply` с отображением `replyReviewRecipient` и `replyReviewSubject`.

**События:**
- `saveDraft: []`
- `continueInCompose: []`
- `send: []`

**UI:**
- «Review reply before sending», «Immediate provider send»
- Кнопка с текстом «Send» или «Sending...»

Прямых вызовов `fetch(`, `ApiClient`, `useQuery` нет.

## Утилиты

### attachmentSearchTable

Предоставляет колонки `attachmentSearchTableColumns` для TanStack Table и функцию `attachmentSearchTableRowId`. Тип результата поиска: `AttachmentSearchResult`. Колонки: `filename`, `message_subject`, `sender`, `size`, `scan_status`. Идентификатор строки – `attachment_id`.

### attachmentTable

Набор утилит для работы с вложениями типа `CommunicationAttachment`.

**Табличные колонки:**
- `attachmentTableColumns`: `filename`, `content_type`, `size`, `scan_status`
- `attachmentTableRowId` – возвращает `attachment_id`

**Форматирование и классы:**
- `formatAttachmentSize(bytes)`: формат `B`, `KB`, `MB`
- `scanStatusClass(status)`: CSS‑классы `att-scan--clean`, `att-scan--suspicious`, `att-scan--danger`, `att-scan--unknown`

**Проверки типов (все учитывают `scan_status` – только `not_scanned` или `clean`):**
- `isInspectableArchiveAttachment`: ZIP‑архивы (по content‑type или расширению `.zip`)
- `isPreviewableTextAttachment`: `text/*`, `application/json`, `xml`, `yaml`, а также расширения `.txt`, `.md`, `.csv`, `.json`, `.xml`, `.yaml`, `.yml`
- `isPreviewableImageAttachment`: `image/png`, `jpeg`, `gif`, `webp` и расширения `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`; **не** обрабатывает SVG
- `isPreviewableAudioAttachment`: `audio/*`, а также `.mp3`, `.m4a`, `.aac`, `.ogg`, `.opus`, `.wav`, `.webm`
- `isPreviewableVideoAttachment`: `video/*`, а также `.mp4`, `.webm`, `.mov`
- `isPreviewablePdfAttachment`: `application/pdf` или `.pdf`
- `isPreviewableAttachment`: объединяет все вышеперечисленные
- `isPreviewAllowedByScanStatus`: разрешает только `not_scanned` и `clean`

### mailDragDrop

Утилиты для drag‑and‑drop сообщений.

- Кастомный MIME‑тип: `MAIL_MESSAGE_DRAG_TYPE = 'application/x-makosh-mail-message-selection'`
- Тип `CommunicationMessageDragPayload` с полями `kind: 'mail-message-selection'`, `message_id`, `message_ids`
- `createCommunicationMessageDragPayload`: нормализует и уникализирует ID, возвращает JSON
- `parseCommunicationMessageDragPayload`: парсит и валидирует, возвращает `null` при ошибке
- `hasCommunicationMessageDragType`: проверяет наличие кастомного типа в списке
- Вспомогательные функции: `validMessageIdList`, `uniqueNonBlankIds`

### mailFolderOrdering

Утилиты для перетаскивания и переупорядочивания папок.

- Кастомный MIME‑тип: `MAIL_FOLDER_REORDER_DRAG_TYPE = 'application/x-makosh-mail-folder-reorder'`
- Типы: `CommunicationFolderReorderPayload`, `CommunicationFolderOrderUpdate`
- `createCommunicationFolderReorderPayload` / `parseCommunicationFolderReorderPayload`
- `hasCommunicationFolderReorderDragType`
- `buildCommunicationFolderReorderUpdates(folders, sourceId, targetId)` – вычисляет обновления `sort_order`:
  - Если есть зазор между позициями, вставляет перемещаемую папку с серединным значением (`midpointSortOrder`)
  - Иначе нормализует `sort_order` всех папок с шагом `SORT_ORDER_STEP = 1000`
  - Возвращает пустой массив при no‑op или отсутствии папок
- `mailFolderReorderStatus(folders, sourceId, targetId)` – строка вида «Moved {name} before {name}»

### mailFolderPresentation

Утилиты отображения иерархических папок.

- `CommunicationFolderDisplayRow` с полями: `folder`, `depth`, `leafName`, `pathPrefix`, `pathParts`
- `mailFolderColorClass(color)`: маппинг hex‑цвета в CSS‑класс (зелёный, жёлтый, красный, фиолетовый, по умолчанию синий)
- `deriveCommunicationFolderDisplayRow`: разбирает имя папки по `/`, определяет глубину и префикс
- `orderCommunicationFolderDisplayRows`: сортирует по `sort_order`, затем по сегментам пути, затем по `folder_id`
- `createChildFolderDraft`: возвращает `parentPath` и `sortOrder` для создания подпапки
- `mailFolderHierarchyDeleteImpact`: находит дочерние папки для предупреждения при удалении (до 3 имён)
- `isDescendantPath`: проверяет, является ли путь потомком

### outboxStatus

Утилиты для отображения статуса исходящих сообщений.

- `outboxStatusPresentation(item, now)` возвращает объект с полями:
  - `title`, `detail` – текстовые метки
  - `tone` – «success», «danger», «warning»
  - `icon` – иконка Tabler
  - `canUndo` – доступна ли отмена
  - При наличии `latest_read_receipt` в metadata показывает статус прочтения без раскрытия приватных деталей
  - При `delivery_status: failed` показывает SMTP‑статус, **не** включая diagnostic‑код
  - Для `status: 'queued'` – «Undo available», `canUndo: true`
  - Для `status: 'scheduled'` – «Retry scheduled» с указанием времени
- `visibleOutboxStatusItems(items, limit)` – фильтрует список: исключает отправленные сообщения без свежих доказательств доставки, оставляет актуальные (например, queued и с read‑receipt), ограничивает количество.
```

### Source coverage / Покрытие источников

Каждый исходный файл и факты, покрытые на предлагаемой странице `components/frontend.md`:

- `MailResourceOverviewStrip.boundary.test.ts` — присутствие секций `subscriptions`, `topSenders`, `blockers`; использование `@tanstack/vue-virtual` / `useVirtualizer`; управление флагами `hasMoreSubscriptions`/`hasMoreTopSenders` и действиями загрузки; отсутствие прямых API‑вызовов.
- `MessageAiReplyPanel.boundary.test.ts` — элементы тона, языка, `generateAiReply`; использование мутации `useGenerateAiReplyVariantsMutation` и переменной `replyVariants`; отсутствие API.
- `MessageAttachmentsTab.boundary.test.ts` — хуки `useAttachmentArchiveInspectionQuery`, `useAttachmentPreviewQuery`, `useTranslateAttachmentMutation`; состояние перевода; функции проверки вложений; UI‑элементы превью и инспекции; использование `source_text: preview.text`; отсутствие прямых API‑вызовов.
- `MessageBodyTab.boundary.test.ts` — встраивание `BilingualReplyPanel`; AI‑суммаризация с секциями Key Points, Action Items, Risks, Deadlines; извлечение знаний; вложенные панели; проксирование удалённых изображений; отсутствие API.
- `MessageLocalIntelligencePanel.boundary.test.ts` — хуки `useExplainMessageMutation` и `useDetectMessageLanguageMutation`; надписи «Importance», «Detect language», «Why this matters»; отсутствие API.
- `MessageRelatedTab.boundary.test.ts` — экспорт в `md`/`eml`/`json`; функции `replyAll`, `forwardMessage`, `redirectMessage`; метки, snooze; события `markMessageRead`/`markMessageUnread`/`deleteFromProvider`; отсутствие API.
- `MessageTrustReviewPanel.boundary.test.ts` — разделы «Security Review», «Recipient Suggestions»; переменные `authRisk`, `smartCc`; отсутствие API.
- `OutboxStatusStrip.boundary.test.ts` — передача данных через `visibleOutboxStatusItems`; события `undo`, `loadMore`, `prefetchMore`; префетчинг по mouseenter/focus; отсутствие внутри `fetch`, `ApiClient`, `useQuery`.
- `RichComposeEditor.boundary.test.ts` — использование TipTap вместо `contenteditable`/`execCommand`; список команд форматирования; расширения из `richComposeExtensions.ts` (нормализация ссылок, `rel: 'noopener noreferrer'`, `target: '_blank'`); санитизация вставленного HTML; эмит файлов при дропе.
- `SavedSearchRuleGroupEditor.boundary.test.ts` — работа с презентационными функциями; отображение глубины и сводки; отсутствие API.
- `SavedSearchStrip.boundary.test.ts` — горизонтальный виртуализатор; чипсы фильтров; построитель правил с валидацией; редактирование `searchRuleTree`; предзагрузка результатов; отсутствие прямых API‑вызовов.
- `ThreadAttachmentInsightPanel.boundary.test.ts` — те же хуки просмотра/перевода вложений; действия «Translate preview», «Inspect archive»; отсутствие API.
- `ThreadConversationView.boundary.test.ts` — пропсы `thread`, `messages`, `isSendingReply`; логика раскрытия сообщений и цитирования; inline‑ответы через `ThreadInlineReplyComposer`; перевод треда; события наружу; отсутствие прямых `fetch` и `ApiClient`.
- `ThreadInlineReplyComposer.boundary.test.ts` — пропсы, интеграция `RichComposeEditor`, режим ревью, события сохранения/отправки; отсутствие API.
- `attachmentSearchTable.test.ts` / `attachmentSearchTable.ts` — колонки TanStack Table для `AttachmentSearchResult`; функция `attachmentSearchTableRowId`.
- `attachmentTable.test.ts` / `attachmentTable.ts` — колонки для `CommunicationAttachment`; форматирование размера; классы статуса сканирования; все распознаватели типов вложений (`isPreviewable*`, `isInspectableArchiveAttachment`) с учётом `scan_status`.
- `mailDragDrop.test.ts` / `mailDragDrop.ts` — кастомный MIME‑тип; сериализация/парсинг payload перетаскивания; отбрасывание некорректных данных.
- `mailFolderOrdering.test.ts` / `mailFolderOrdering.ts` — кастомный MIME‑тип; алгоритм обновления `sort_order` (серединное значение или полная нормализация); статусная строка перемещения.
- `mailFolderPresentation.test.ts` / `mailFolderPresentation.ts` — разбор иерархических имён папок; сортировка; определение папок‑потомков для предупреждения удаления.
- `outboxStatus.test.ts` / `outboxStatus.ts` — приоритизация read‑receipt; скрытие приватного diagnostic‑кода; вычисление `canUndo` и статуса retry; фильтрация для компактного списка.

### Drift candidates / Кандидаты на drift

В предоставленном контексте (только граничные тесты и несколько утилит) видимых расхождений между кодом, документацией и ADR не обнаружено. Файлы `.vue` и реализации хуков не вложены, поэтому сравнить факты из тестов с полными контрактами компонентов невозможно — drift может существовать, но из доступных исходников он не виден.

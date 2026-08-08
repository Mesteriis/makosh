---
chunk_id: 132-other-frontend-part-005
batch_id: batch-20260628T214902
group: frontend
role: other
source_status: pending
source_count: 21
generated_by: code-wiki-ru
---

# 132-other-frontend-part-005 — frontend/other

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `21`

## Резюме

Создаётся новая страница русской Obsidian-вики `components/frontend.md`, которая описывает компонентную архитектуру фронтенда Макошь. На основе предоставленных исходных файлов документируются компоненты трёх доменов — `agents`, `calendar` и `communications` — включая их основные страницы, панели, сетки, списки и формы. Для каждого компонента фиксируется назначение, ключевые props/emits, используемые хранилища и запросы, а также видимые в коде элементы интерфейса. Описание строго опирается на встроенный исходный код без внешних предположений.

---

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Компоненты фронтенда (Frontend Components)

## Общая архитектура

Фронтенд Макошь построен на **Vue 3** с **TypeScript** и использует:
- `@tanstack/vue-query` — серверные запросы и кэширование
- `@tanstack/vue-virtual` — виртуализация списков и ленты папок
- `vee-validate` — валидация форм
- `tabler` — иконки через `shared/ui/Icon`
- `platform/i18n` — локализация (`useI18n`, метод `t`)

Компоненты организованы по доменам: `agents`, `calendar`, `communications`. Каждый домен содержит подкаталоги `components`, `views`, `stores`, `queries`, `types`, `api`, `forms`. Общие UI‑примитивы лежат в `shared/ui`.

---

## Домен Agents

### AgentsPage

**Файл:** `frontend/src/domains/agents/views/AgentsPage.vue`

Главная страница AI‑агентов. Компонует:
- `AgentsRuntimeMetrics` — сетка метрик
- `AgentsGrid` — карточки агентов
- `AgentsDetail` — детали выбранного агента
- `AgentsWorkflows` — формы взаимодействия с AI
- `AgentsRail` — боковая панель

Использует хранилище `useAgentsStore` и запрос `useAiWorkspaceQuery`. При обновлении данных (`watch(workspaceData)`) заполняет хранилище. Панель инструментов содержит кнопку **Refresh** и фильтр‑бары с состоянием моделей (Chat model ready/missing, Embedding ready/missing) через `aiModelSummary`.

---

### AgentsRail

**Файл:** `frontend/src/domains/agents/components/AgentsRail.vue`

Боковая панель с тремя секциями:

- **Runtime** — отображает `aiRuntimeSummary`, Owner Persona (`ownerPersona.display_name` или `'not set'`), Chat model, Embedding model.
- **Run History** — до 6 последних запусков. Для каждого: имя‑персона `{agent_id}@sh-inc.ru`, метка статуса (`runStatusLabel`), дата (`formatDateTime`) и длительность (`formatDuration`). При отсутствии запусков — `No AI runs persisted yet.`
- **Latest Citations** — до 3 цитирований из самого свежего запуска (через `safeCitations`). Для каждого: заголовок, отрывок. При отсутствии — `Citations appear after an answer or briefing run.`

Props:
- `aiRuns: AiRun[]`
- `aiStatus: AiStatus | null`
- `ownerPersona: OwnerPersona | null`
- `isAiLoading: boolean`

---

### AgentsRuntimeMetrics

**Файл:** `frontend/src/domains/agents/components/AgentsRuntimeMetrics.vue`

Шесть метрик‑карточек (`metric-card`):

| Метка               | Значение                                                     | Дополнительно                                   |
|---------------------|--------------------------------------------------------------|-------------------------------------------------|
| Runtime             | `aiRuntimeSummary`                                           | `Ollama {version}` или `Ollama`                 |
| Agents              | `aiAgents.length`                                            | `Registered` / `Not loaded`                     |
| Run History         | `aiRuns.length`                                              | `Persisted runs`                                |
| Embedding           | `aiStatus.embedding_dimension ?? 0`                          | `aiStatus.embedding_model ?? 'No model'`        |
| Suggested Tasks     | `0`                                                          | `Review queue`                                  |
| Latest Duration     | `formatDuration(aiRuns[0]?.duration_ms)`                     | `{имя‑персона}` или `No runs`                   |

Props:
- `aiStatus: AiStatus | null`
- `aiAgents: AiAgent[]`
- `aiRuns: AiRun[]`
- `isAiLoading: boolean`

---

### AgentsWorkflows

**Файл:** `frontend/src/domains/agents/components/AgentsWorkflows.vue`

Три формы‑блока, расположенные в CSS‑сетке `grid-template-columns: repeat(3, ...)`:

1. **Ask AI** — `textarea` (v‑model `aiQuestion`), кнопка **Ask** (эмитит `submitAnswer`).
2. **Prepare brief** — `textarea` (`aiMeetingTopic`), кнопка **Prepare** (эмитит `submitMeetingPrep`).
3. **Task extraction** — `textarea` (`aiTaskQuery`), кнопка **Refresh candidates** (эмитит `refreshTasks`).

Кнопки отключены (`disabled`), когда поле пусто или идёт отправка (флаги `isAiAnswerSubmitting`, `isAiMeetingPrepSubmitting`, `isAiTaskRefreshSubmitting`).

После получения результатов:
- `aiAnswerResult` → блок **Answer** с текстом и списком цитирований (source_id, source_kind, title, excerpt).
- `aiMeetingPrepResult` → блок **Meeting Brief** с брифингом и цитированиями.
- `aiTaskRefreshResult` → блок **Task Candidates** с текстом `{created_count} suggested candidates refreshed. Review them in Tasks.`

Emits: `update:aiQuestion`, `update:aiMeetingTopic`, `update:aiTaskQuery`, `submitAnswer`, `submitMeetingPrep`, `refreshTasks`.

---

## Домен Calendar

### CalendarPage

**Файл:** `frontend/src/domains/calendar/views/CalendarPage.vue`

Главная страница календаря. Интегрирует:
- `CalendarToolbar`
- `CalendarWeekGrid`
- `CalendarUpcoming`
- `CalendarSourceStatus`

Дополнительные блоки прямо в разметке:
- **Форма создания события** (показывается по `store.showNewEventForm`): поля title, event_type (`meeting`, `focus`, `deadline`, `personal`, `travel`, `tax`, `review`, `planning`), datetime‑local для start/end. Кнопки Create/Cancel. Вызывает `createCalendarEvent`.
- **Фильтр‑бар**: количество аккаунтов/событий, ошибка (`store.calendarError`), подсказка поиска.
- **Weekly Brief** (панель с метриками: Events, Overdue, No Notes). Кнопка обновления вызывает `loadWeeklyBrief`. Заполняется через `fetchWeeklyBrief`.
- **Event Detail** (появляется при `store.selectedEvent`): время, location, status, блок Brief (участники, контекст‑summary), блок Agenda (список `suggested_agenda`), кнопки Prepare и Complete.
- **CalendarSourceStatus** и **CalendarUpcoming** в боковой панели `stacked-rail`.

Использует хранилище `useCalendarStore`, запросы `useCalendarEventsQuery` и `useCalendarAccountsQuery`. Методы загрузки: `loadSources`, `loadWeeklyBrief`, `handleSearch` (через `searchCalendarEvents`), `handlePrepareEvent` (параллельная загрузка context pack, brief, agenda), `handleRefreshAll`.

---

### CalendarToolbar

**Файл:** `frontend/src/domains/calendar/components/CalendarToolbar.vue`

Панель инструментов:
- Инпут поиска с привязкой к `store.searchQuery` (эмитит `search-calendar` при вводе).
- Pill‑кнопки режимов просмотра: **Day**, **Week**, **Month**, **Agenda** (меняют `store.viewMode`).
- Кнопка **New Event** (эмитит `store.toggleNewEventForm`).
- Кнопка **Refresh** (эмитит `refresh-all`).

---

### CalendarUpcoming

**Файл:** `frontend/src/domains/calendar/components/CalendarUpcoming.vue`

Секция **Upcoming**. Фильтрует `calendarEvents` по `new Date(e.start_at) >= new Date()` и показывает до 8 событий. Каждое: дата (`formatEventDate`), время (`formatEventTime`), заголовок. При клике эмитит `prepare-event`. При отсутствии — `No upcoming events`.

Props: `calendarEvents: CalendarEvent[]`
Emits: `prepare-event(evt: CalendarEvent)`

---

### CalendarWeekGrid

**Файл:** `frontend/src/domains/calendar/components/CalendarWeekGrid.vue`

Таблица событий на неделю. Состоит из:
- **week-header** — колонки дней (`weekColumns`).
- **event-list** — строки событий. При `isCalendarLoading` — `Loading events...`, при отсутствии — `No events`. Отображает либо результаты поиска (`calendarSearchResults`), либо отфильтрованные за неделю (`filteredEvents`).

Строка события содержит:
- День (`formatEventDayShort`)
- Интервал времени (`formatEventTime(start) - formatEventTime(end)`)
- Заголовок
- Чип типа события (`eventTypeLabel`)
- Индикатор важности (красная точка, если `importance_score > 0.5`)
- Индикатор готовности (жёлтая точка, если `readiness_score < 0.5`)

При клике вызывает `onPrepareEvent`.

Footer: бейджи `{acct.account_name}` для каждого аккаунта.

---

### CalendarSourceStatus

**Файл:** `frontend/src/domains/calendar/components/CalendarSourceStatus.vue`

Панель **Calendars**. Если `calendarSources.length === 0` — показывает аккаунты (disabled‑checkbox с `account_name` и `provider`). Иначе — источники с `name` и `timezone`. Чекбоксы всегда `checked disabled`.

Props: `calendarSources: CalendarSource[]`, `calendarAccounts: CalendarAccount[]`.

---

## Домен Communications

### CommunicationsActionBar

**Файл:** `frontend/src/domains/communications/components/CommunicationsActionBar.vue`

Верхняя панель действий. Содержит:
- `<Teleport to="#makosh-topbar-slot">` для встраивания `CommunicationsTopbarSlot`.
- Строку статуса синхронизации (syncStatusMessage, syncError) с кнопкой закрытия.
- `MailSyncSettingsStrip` — настройки синхронизации.
- `HealthStrip` — здоровье ящика.
- `MailCertificateStrip` — сертификаты.
- `MailResourceOverviewStrip` — обзор подписок, топ‑отправителей, блокировок.
- `DraftStrip` — черновики.

Тосты‑уведомления (фиксированные) в нижней части экрана:
- Статус действия (`actionStatus`).
- Готовый экспорт (`lastMessageExport`) — ссылка для скачивания по data‑uri.
- Ошибка действия (`actionError`).
- Глобальная ошибка страницы (`pageError`) с кнопкой очистки.

Props: множество, включая `searchQuery`, `activeSectionId`, `stateCounts`, `syncSettings`, `health`, `drafts`, `actionStatus` и др.
Emits: `update:searchQuery`, `search`, `openAccountSetup`, `compose`, `syncNow`, `updateSyncSettings`, `clearSyncStatus`, `loadMoreSubscriptions`, `loadMoreTopSenders`, `selectSection`, `openDraft`, `deleteDraft`, `loadMoreDrafts`, `clearPageError`.

---

### CommunicationList

**Файл:** `frontend/src/domains/communications/components/CommunicationList.vue`

Виртуализированный список сообщений с `useVirtualizer` (размер элемента ~72px). Особенности:
- Автоподгрузка следующей страницы при скролле (порог 360px).
- Клавиатурная навигация:
  - `Ctrl/Cmd + A` — выбрать все видимые сообщения (`selectVisible`).
  - `Escape` — снять выделение (`clearSelection`).
  - `Space` — переключить выделение текущего с учётом Shift‑расширения.
  - `Shift + ArrowUp/Down` — расширить диапазон выделения.
- При изменении `selectedIndex` вызывается `virtualizer.scrollToIndex` с `align: 'center'`.
- Prefetch сообщения при наведении/фокусе через `useCommunicationMessagePrefetch`.

Props: `messages: CommunicationMessageSummary[]`, `selectedIndex`, `selectedMessageIds`, `isLoading`, `hasNextPage`, `isFetchingNextPage`.
Emits: `select`, `toggleSelection`, `selectVisible`, `clearSelection`, `loadMore`.

---

### CommunicationListItem

**Файл:** `frontend/src/domains/communications/components/CommunicationListItem.vue`

Строка сообщения:
- Чекбокс выделения (toggle selection с поддержкой Shift‑диапазона через emit).
- Иконка канала (`communicationChannelIcon`).
- Отправитель (`senderLabel`), тема (`message.subject`), жирный шрифт при `workflow_state === 'new'` (непрочитано).
- Метка важности — `!`, если `importance_score >= 7`.
- Время (`messageTime`).
- Превью (`conversationPreview`).
- Индикатор вложений при `attachment_count > 0`.

Drag & drop разрешён только когда сообщение выделено (`isChecked`). Payload содержит текущий `message_id` и все выделенные `selectedMessageIds` (через `MAIL_MESSAGE_DRAG_TYPE`).

Props: `message: CommunicationMessageSummary`, `isSelected`, `isChecked`, `selectedMessageIds`.
Emits: `select`, `toggleSelection(extendRange: boolean)`, `prefetch`.

---

### CommunicationViewer

**Файл:** `frontend/src/domains/communications/components/CommunicationViewer.vue`

Просмотрщик сообщения. При отсутствии `detail` — заглушка "Select a message to view". При наличии:

1. **Header** с кнопками: pin, star, bell‑off, mark read, trash, forward, и actions (replyAll, redirect, export, addLabel, removeLabel, snooze, и т.д.) — все через emits.
2. **Заголовок** с темой, отправителем, временем.
3. **AI State панель** — запрос `useMessageAiStateQuery`, отображение `ai_state`, `review_reason`/`last_error`. Кнопки для перехода состояний: Process, Review, Done, Failed, Archive — через `useUpdateMessageAiStateMutation`.
4. **Вкладки** (`Tabs`): Message, Attachments, Headers, Related, Timeline. Рендерятся компонентами `MessageBodyTab`, `MessageAttachmentsTab`, `MessageHeadersTab`, `MessageRelatedTab`, `MessageTimelineTab` (не детализированы в данном контексте).
5. **Emits**: помимо действий над сообщением, включает `sendBilingualReply(payload: BilingualReplyFlowResponse)`.

Props: `detail: CommunicationMessageDetailResponse | null`, `insight: CommunicationMessageInsight | null`, `activeTab: MessageContextTab` (с v‑model через emit `update:activeTab`).

---

### CommunicationFolderStrip

**Файлы:** `frontend/src/domains/communications/components/CommunicationFolderStrip.vue`, `CommunicationFolderStrip.css`

Горизонтальная виртуализированная лента папок. Основные возможности:

- **Отображение папок** с иерархическими отступами (CSS‑переменная `width` для глубины).
- **Drag‑drop перестановка** порядка папок (через `useCommunicationFolderReorder`).
- **Drag‑drop сообщений** на папку: перемещение (`move`) или копирование (`copy`, если зажат Alt). Реализовано через мутации `useCopyMessageToFolderMutation` / `useMoveMessageToFolderMutation`.
- **Создание/редактирование папки** — диалог `Dialog` с формой (`vee-validate`): поля имени (путь формируется из родительского пути и листового имени), цвета (5 предустановленных), порядка сортировки. Предпросмотр полного пути `folderPathPreview`. Валидация родительского пути через `validateCommunicationFolderParentPath`.
- **Удаление папки** — диалог подтверждения с отображением влияния (`mailFolderHierarchyDeleteImpact`): количество затронутых дочерних папок и их имена.
- **Бесконечная подгрузка** при горизонтальном скролле (запрос `useCommunicationFoldersQuery` с `fetchNextPage`).

Props: `accountId: string | null`, `activeId: string`.
Emits: `select(folderId), deleted(folder)`.

---

### AttachmentSearchPanel

**Файлы:** `frontend/src/domains/communications/components/AttachmentSearchPanel.vue`, `AttachmentSearchPanel.css`

Раскрывающаяся панель поиска вложений по метаданным.

- **Toggle** — кнопка с aria‑expanded, заголовком "Attachment search" и счётчиком результатов.
- **Форма поиска** (`vee-validate`, схема `attachmentSearchVeeValidationSchema`): поля Query, Content type, Scan status (select с опциями `attachmentScanStatusOptions`). Кнопка **Search**.
- **Таблица результатов** (`useVueTable`, `useVirtualizer`). Колонки:
  - filename (с иконкой типа `attachmentIcon` и именем файла)
  - message_subject (с иконкой mail)
  - size (`formatAttachmentSize`)
  - scan_status (цветной чип: clean / suspicious / danger через `scanStatusClass`)
- Поддержка prefetch результата при наведении/фокусе (`useAttachmentSearchResultPrefetch`).
- Кнопка **Load more** для пагинации (`fetchNextPage`). Автоподгрузка при скролле (порог 180px).

Props: `accountId: string | null`. Использует собственное состояние `isOpen`, `hasSubmitted`, `submittedRequest`.

---

### BilingualReplyPanel

**Файл:** `frontend/src/domains/communications/components/BilingualReplyPanel.vue`

Панель подготовки двуязычного ответа.

- **Форма**: textarea для текста ответа на русском (`replyTextRu`), select для тона (`bilingualReplyToneOptions`).
- **Кнопка Prepare** (мутация `usePrepareBilingualReplyFlowMutation`). При ошибке — отображает текст ошибки.
- **Результат** (после успешной мутации): сетка 2×2 с шагами:
  - **Original** — исходный язык, уверенность (confidence), текст.
  - **Translation** — статус (`translated` / `review required`), текст.
  - **Reply in Russian** — тон, текст.
  - **Back Translation** — статус, текст.
- **Кнопка Open Compose** — доступна, когда `result.send_ready === true`, эмитит `sendBilingualReply` с полным результатом.

Props: `messageId: string`.
Emits: `sendBilingualReply(payload: BilingualReplyFlowResponse)`.

---

### BulkActionsBar

**Файл:** `frontend/src/domains/communications/components/BulkActionsBar.vue`

Панель массовых действий, видимая при `selectedCount > 0`. Содержит:

- Счётчик "N selected".
- Кнопки действий (`action` → emit `action`):
  - `mark_read`, `mark_unread`, `archive`, `trash`, `pin`, `unpin`, `important`, `not_important`
  - `add_label "Follow up"` / `remove_label "Follow up"`
  - `snooze` с временем `nextBusinessMorningIso()` (следующее утро 9:00).
- Кнопка очистки выделения.
- Drag‑drop сообщений на кнопки действий (через `handleActionDrop` с проверкой `MAIL_MESSAGE_DRAG_TYPE`).

Props: `selectedCount: number`, `isRunning: boolean`.
Emits: `action(command: BulkActionCommand)`, `clear`.

---

### CommunicationsCallsPanel

**Файл:** `frontend/src/domains/communications/components/CommunicationsCallsPanel.vue` (обрезан)

Панель звонков и встреч. Режимы: `calls` (все) и `meetings` (только Zoom‑звонки, фильтр по `meetingProvider(call) === 'zoom'`).

- **Список звонков** из `useProviderCallsQuery`. Каждая строка: идентификатор (`describeCall`: `provider_call_id`, `call_state`, `direction`), тема (`metadata.topic`), дата.
- **Детали выбранного звонка**:
  - Мета‑поля: Provider, Provider id, Meeting id, Direction, State, Started, Ended, Host email, Transcript ref, Join url, количество Participants, количество Recording refs.
  - Чипы участников (`metadata.participants`): `participantLabel` (display_name/email/participant_id) + `participantSecondary`.
  - Чипы записей (`metadata.recording_refs`): `recordingLabel`, дата и размер файла.
  - Ссылка "Open join URL", если `metadata.join_url` не пуст.
- **Транскрипт** — загрузка через `useProviderCallTranscriptQuery`. Отображает мета‑данные: transcript_status, stt_provider, language_code, source_audio_ref, и текст `transcript_text`.

Props: `mode: 'calls' | 'meetings'`.

---

### CommunicationsContextInspector

**Файл:** `frontend/src/domains/communications/components/CommunicationsContextInspector.vue`

Боковая панель инспектора контекста.

- **Селектор режимов** (Context, Contact, Organization) — кнопки с иконками, active‑класс при `inspectorMode === id`.
- При отсутствии `detail` — "Select a message to inspect".
- При наличии:
  - Профиль отправителя: аватар (первая буква имени), имя, email.
  - **Summary** — `message.ai_summary` или "No AI summary available".
  - **Metadata** — `importance_score`, `ai_category`, `workflow_state`.
  - **Attachments** — `detail.attachments.length` files.

Props: `detail: CommunicationMessageDetailResponse | null`, `inspectorMode: InspectorMode` (v‑model через emit `update:inspectorMode`).

---

## Общие компоненты

Из `shared/ui` используются:
- `Icon` — векторные иконки Tabler.
- `Button` — кнопка с вариантами `default`, `ghost`, `outline`; размером `sm`; состоянием загрузки `loading`.
- `Tabs` — вкладки с `active` и событием `select`.
- `Dialog` — модальное окно (используется в `CommunicationFolderStrip`).

Локализация: `platform/i18n` предоставляет `useI18n()` и метод `t(key)`.
```

---

## Покрытие источников

| Файл | Покрытые факты |
|------|----------------|
| `frontend/src/domains/agents/components/AgentsRail.vue` | Секции Runtime, Run History, Citations; формат имени‑персоны; используемые функции. |
| `frontend/src/domains/agents/components/AgentsRuntimeMetrics.vue` | 6 метрик‑карточек, их значения и подписи; источник данных. |
| `frontend/src/domains/agents/components/AgentsWorkflows.vue` | Три формы (Ask, Brief, Tasks), их поля, emits, отображение результатов с цитатами. |
| `frontend/src/domains/agents/views/AgentsPage.vue` | Композиция страницы, используемые компоненты, хранилище, запросы, кнопка Refresh, фильтр‑бары. |
| `frontend/src/domains/calendar/components/CalendarSourceStatus.vue` | Отображение источников/аккаунтов с disabled‑checkboxes. |
| `frontend/src/domains/calendar/components/CalendarToolbar.vue` | Поиск, переключатель режимов (Day/Week/Month/Agenda), кнопки New Event, Refresh. |
| `frontend/src/domains/calendar/components/CalendarUpcoming.vue` | Фильтрация будущих событий, до 8 записей, формат даты/времени, emit `prepare-event`. |
| `frontend/src/domains/calendar/components/CalendarWeekGrid.vue` | Структура недельной таблицы, индикаторы важности/готовности, бейджи аккаунтов. |
| `frontend/src/domains/calendar/views/CalendarPage.vue` | Композиция страницы, форма создания события, поиск, панели Weekly Brief и Event Detail, загрузка источников/брифингов/контекста. |
| `frontend/src/domains/communications/components/AttachmentSearchPanel.css` + `.vue` | Раскрывающаяся панель, форма поиска (vee-validate), виртуализированная таблица, prefetch, пагинация, цветовые чипы сканирования. |
| `frontend/src/domains/communications/components/BilingualReplyPanel.vue` | Форма двуязычного ответа, мутация `usePrepareBilingualReplyFlowMutation`, шаги original/translation/reply/back‑translation, кнопка Open Compose. |
| `frontend/src/domains/communications/components/BulkActionsBar.vue` | Кнопки массовых действий (read/unread, archive, trash, pin, label, snooze), drag‑drop, очистка выделения. |
| `frontend/src/domains/communications/components/CommunicationFolderStrip.css` + `.vue` (обрезан) | Горизонтальная виртуальная лента папок, drag‑drop перестановка и перемещение/копирование сообщений, диалоги создания/редактирования/удаления, бесконечная подгрузка. |
| `frontend/src/domains/communications/components/CommunicationList.vue` | Виртуализированный список, клавиатурная навигация (Ctrl+A, Esc, Space, Shift+arrows), автоподгрузка, скролл к выделенному. |
| `frontend/src/domains/communications/components/CommunicationListItem.vue` | Элемент списка: чекбокс, иконка канала, отправитель, тема, важность, превью, вложения; drag‑drop payload. |
| `frontend/src/domains/communications/components/CommunicationViewer.vue` | Заглушка пустого состояния, хедер с действиями, AI State панель с кнопками перехода, вкладки (Message, Attachments, Headers, Related, Timeline). |
| `frontend/src/domains/communications/components/CommunicationsActionBar.vue` | Teleport в #makosh-topbar-slot, строка синхронизации, стрипы настроек/здоровья/ресурсов/черновиков, тосты (статус, экспорт, ошибка, глобальная ошибка). |
| `frontend/src/domains/communications/components/CommunicationsCallsPanel.vue` (обрезан) | Режимы calls/meetings, список звонков, детали метаданных, участники, записи, транскрипт. |
| `frontend/src/domains/communications/components/CommunicationsContextInspector.vue` | Профиль отправителя, селектор режимов (Context/Contact/Organization), Summary, Metadata, Attachments count. |

---

## Исходные файлы

- [`frontend/src/domains/agents/components/AgentsRail.vue`](../../../../frontend/src/domains/agents/components/AgentsRail.vue)
- [`frontend/src/domains/agents/components/AgentsRuntimeMetrics.vue`](../../../../frontend/src/domains/agents/components/AgentsRuntimeMetrics.vue)
- [`frontend/src/domains/agents/components/AgentsWorkflows.vue`](../../../../frontend/src/domains/agents/components/AgentsWorkflows.vue)
- [`frontend/src/domains/agents/views/AgentsPage.vue`](../../../../frontend/src/domains/agents/views/AgentsPage.vue)
- [`frontend/src/domains/calendar/components/CalendarSourceStatus.vue`](../../../../frontend/src/domains/calendar/components/CalendarSourceStatus.vue)
- [`frontend/src/domains/calendar/components/CalendarToolbar.vue`](../../../../frontend/src/domains/calendar/components/CalendarToolbar.vue)
- [`frontend/src/domains/calendar/components/CalendarUpcoming.vue`](../../../../frontend/src/domains/calendar/components/CalendarUpcoming.vue)
- [`frontend/src/domains/calendar/components/CalendarWeekGrid.vue`](../../../../frontend/src/domains/calendar/components/CalendarWeekGrid.vue)
- [`frontend/src/domains/calendar/views/CalendarPage.vue`](../../../../frontend/src/domains/calendar/views/CalendarPage.vue)
- [`frontend/src/domains/communications/components/AttachmentSearchPanel.css`](../../../../frontend/src/domains/communications/components/AttachmentSearchPanel.css)
- [`frontend/src/domains/communications/components/AttachmentSearchPanel.vue`](../../../../frontend/src/domains/communications/components/AttachmentSearchPanel.vue)
- [`frontend/src/domains/communications/components/BilingualReplyPanel.vue`](../../../../frontend/src/domains/communications/components/BilingualReplyPanel.vue)
- [`frontend/src/domains/communications/components/BulkActionsBar.vue`](../../../../frontend/src/domains/communications/components/BulkActionsBar.vue)
- [`frontend/src/domains/communications/components/CommunicationFolderStrip.css`](../../../../frontend/src/domains/communications/components/CommunicationFolderStrip.css)
- [`frontend/src/domains/communications/components/CommunicationFolderStrip.vue`](../../../../frontend/src/domains/communications/components/CommunicationFolderStrip.vue)
- [`frontend/src/domains/communications/components/CommunicationList.vue`](../../../../frontend/src/domains/communications/components/CommunicationList.vue)
- [`frontend/src/domains/communications/components/CommunicationListItem.vue`](../../../../frontend/src/domains/communications/components/CommunicationListItem.vue)
- [`frontend/src/domains/communications/components/CommunicationViewer.vue`](../../../../frontend/src/domains/communications/components/CommunicationViewer.vue)
- [`frontend/src/domains/communications/components/CommunicationsActionBar.vue`](../../../../frontend/src/domains/communications/components/CommunicationsActionBar.vue)
- [`frontend/src/domains/communications/components/CommunicationsCallsPanel.vue`](../../../../frontend/src/domains/communications/components/CommunicationsCallsPanel.vue)
- [`frontend/src/domains/communications/components/CommunicationsContextInspector.vue`](../../../../frontend/src/domains/communications/components/CommunicationsContextInspector.vue)

## Кандидаты на drift

Из предоставленного контекста расхождения между кодом и существующей документацией (или между разными частями кода) не видны. Все описанные факты основаны исключительно на встроенном исходном коде. Потенциальные точки для валидации — поведение импортируемых, но не включённых в контекст компонентов (`AgentsGrid`, `AgentsDetail`, `MessageBodyTab`, `MailSyncSettingsStrip` и др.) — не могут быть оценены в рамках текущего чанка.

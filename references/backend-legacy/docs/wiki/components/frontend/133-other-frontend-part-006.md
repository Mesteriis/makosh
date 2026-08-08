---
chunk_id: 133-other-frontend-part-006
batch_id: batch-20260628T214902
group: frontend
role: other
source_status: pending
source_count: 18
generated_by: code-wiki-ru
---

# 133-other-frontend-part-006 — frontend/other

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `18`

## Резюме

Добавить/обновить страницу `components/frontend.md` в русской Obsidian‑wiki проекта `makosh‑hub`. Страница документирует Vue‑компоненты модуля Communications (почта, треды, сообщения) на основе встроенных исходных файлов. Описаны назначение, входные параметры (`props`), события (`emits`), ключевые состояния и особенности UI каждого компонента. Материал ограничен предоставленным контекстом – внешние знания о фреймворке или проекте не добавлены.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Frontend компоненты

Документация Vue‑компонентов для рабочей области Communications в
frontend‑части Макошь. Все компоненты находятся в
`frontend/src/domains/communications/components/`.

---

## CommunicationsWorkbench

Основной контейнер, задающий сетку (CSS Grid) для трёх панелей: список,
детали и боковая панель (rail). При отсутствии rail переключается на
двухколоночный layout.

**Props:**

- `isLoading: boolean`
- `hasError: boolean`
- `hasRail: boolean`

**Слоты:**

- `list` – панель списка (левая колонка)
- `detail` – панель детального просмотра (центральная колонка)
- `rail` – боковая панель (правая колонка), отображается только когда
  `hasRail === true`

**Состояния:**

Классы `is-loading`, `has-error` добавляются к корневому элементу для
условной стилизации. Управление видимостью контента выполняется через
родительские компоненты.

---

## CommunicationsTopbarSlot

Верхняя панель с заголовком «Mail», поиском и кнопками действий.

**Props:**

- `searchQuery: string` – текущий текст поиска
- `isSyncBusy: boolean` – флаг занятости синхронизации

**Emits:**

- `update:searchQuery` – обновление текста поиска
- `search` – запуск поиска (Enter в поле ввода)
- `openAccountSetup` – открыть настройки учётной записи
- `compose` – открыть композер нового сообщения
- `syncNow` – запустить принудительную синхронизацию

**Кнопки:**

- «Add mail account» (иконка `tabler:mail-plus`) – `openAccountSetup`
- «Compose» (иконка `tabler:edit`) – `compose`
- «Refresh» (иконка `tabler:refresh`) – `syncNow`, при `isSyncBusy`
  кнопка заблокирована и иконка анимирована (`spin-icon`)

Поле поиска `aria-label="Search messages"`, placeholder
`"Search messages..."`.

---

## CommunicationsListPane

Левая навигационная панель, управляющая отображением списка сообщений
или бесед. Содержит состояния загрузки и ошибки.

**Props:**

- `accountId: string`
- `messages: CommunicationMessageSummary[]`
- `threads: CommunicationThreadSummary[]`
- `selectedIndex: number`
- `selectedThreadId: string`
- `selectedMessageIds: string[]`
- `navigatorMode: NavigatorMode`
- `isFolderMode: boolean`
- `isLoading: boolean`
- `hasNextPage: boolean`
- `isFetchingNextPage: boolean`
- `hasThreadNextPage: boolean`
- `isFetchingThreadNextPage: boolean`
- `errorMessage: string`

**Emits:**

- `select` – выбор сообщения/треда (индекс)
- `selectThread` – выбор треда (объект)
- `toggleSelection` – включение/исключение сообщения из множественного
  выделения (messageId, extendRange)
- `selectVisible` – выделить видимые сообщения (список messageId)
- `clearSelection` – снять выделение
- `loadMore` – подгрузка следующей страницы сообщений
- `loadMoreThreads` – подгрузка следующей страницы тредов
- `update:navigatorMode` – переключение режима навигации

**Логика:**

- При наличии `errorMessage` отображается сообщение об ошибке с иконкой
  `tabler:alert-circle`.
- При `isLoading` (и отсутствии ошибки) – индикатор загрузки с
  анимацией.
- Если `isFolderMode === false` и `navigatorMode` равен `'threads'` или
  `'contacts'` – рендерится `CommunicationsConversationList`.
- Иначе рендерится `CommunicationList` (папочный режим или другие
  значения `navigatorMode`).

---

## CommunicationsConversationList

Список бесед с переключением режимов «Threads» / «Contacts».
Поддерживает предзагрузку данных тредов при наведении и фокусе.

**Props:** (аналогично `CommunicationsListPane`, плюс
`hasThreadNextPage`, `isFetchingThreadNextPage`, `accountId`)

**Emits:**

- `select` – выбор сообщения по индексу (из списка сообщений)
- `selectThread` – выбор треда
- `loadMoreThreads` – подгрузка тредов
- `update:navigatorMode` – переключение режима

**Режим «Threads»:**

- Каждый элемент `thread-item` отображает: количество сообщений,
  участников, тему, индикатор непрочитанного (`has_open_action` →
  синяя точка), иконку вложения (`has_attachments`), время последней
  активности.
- При клике – `selectThread`.
- При `mouseenter` / `focus` – вызов `useThreadMessagesPrefetch` для
  предзагрузки сообщений треда.
- Кнопка «Load more» появляется при `hasThreadNextPage === true`.

**Режим «Contacts»:**

- Сообщения группируются по email отправителя (извлекается из поля
  `sender` через регулярное выражение `<(.+?)>`).
- Группы сортируются по времени последнего сообщения (убывание).
- Заголовок группы показывает `label` отправителя и количество
  сообщений.
- Каждое сообщение отображается с точкой непрочитанного
  (`workflow_state === 'new'`), темой и временем.
- Выбор сообщения вызывает `select(index)`.

---

## DraftStrip

Виртуализированная полоса черновиков в верхней части интерфейса.
Использует `@tanstack/vue-virtual`.

**Props:**

- `drafts: CommunicationDraft[]`
- `hasMore: boolean`
- `isLoadingMore: boolean`

**Emits:**

- `openDraft` – открыть черновик (объект `CommunicationDraft`)
- `deleteDraft` – удалить черновик по `draftId`
- `loadMore` – загрузить ещё черновики

**Особенности:**

- Заголовок: «Drafts (N)» с иконкой `tabler:edit`.
- Список виртуализирован (высота элемента ~46px, overscan 8).
- Каждый элемент: тема (или «(No subject)»), получатели, кнопка
  удаления.
- Кнопка «Load more drafts» внизу, с индикатором загрузки.

---

## HealthStrip

Информационная полоса со сводкой состояния почтового ящика.

**Props:**

- `health: MailboxHealth | null`

**Отображаемые метрики:**

- **Total** – `total_messages` (иконка `tabler:mail`)
- **Unread** – `unread` (иконка `tabler:mail-opened`)
- **Action** – `needs_action` (иконка `tabler:alert-circle`)
- **Waiting** – `waiting` (иконка `tabler:clock`)
- **Important** – `important` (иконка `tabler:star`)

**Цветовая индикация:**

Функция `healthToneClass` вычисляет CSS‑класс на основе отношения
значения к `total_messages`:

- > 80% – `health-item--danger` (красный)
- > 50% – `health-item--warning` (оранжевый)
- иначе – `health-item--success` (зелёный)

---

## MailResourceOverviewStrip

Обзорная панель с тремя колонками: подписки (newsletters), топ‑отправители,
блокирующие проблемы.

**Props:**

- `subscriptions: SubscriptionSource[]`
- `topSenders: SenderStats[]`
- `blockers: CommunicationArchitectureBlocker[]`
- `isLoading: boolean`
- `hasMoreSubscriptions: boolean`
- `isLoadingMoreSubscriptions: boolean`
- `hasMoreTopSenders: boolean`
- `isLoadingMoreTopSenders: boolean`

**Emits:**

- `loadMoreSubscriptions`
- `loadMoreTopSenders`

**Детали:**

- Подписки и топ‑отправители виртуализированы (`@tanstack/vue-virtual`)
  с оценкой высоты элемента 28px.
- Каждый чип отображает `sender` и `message_count`.
- Кнопки «More newsletters» / «More senders» для постраничной загрузки.
- Блокеры показываются первые 2 с классом `warning`, при отсутствии –
  надпись «No blockers».

---

## MailCertificateStrip

Сворачиваемая панель для управления почтовыми сертификатами.

**Особенности:**

- По умолчанию отображается заголовок «Certificates» с количеством
  хранимых и истекающих (за 90 дней) сертификатов.
- При разворачивании (`isOpen = true`):
  - Блок «Expiring certificates» – чипы с именем владельца и сроком
    действия.
  - Блок «Stored certificates» – чипы с именем и статусом доверия
    (предупреждающий стиль при отзыве или `trust_status !== 'trusted'`).
  - Форма добавления нового сертификата с полями:
    `cert_id`, `owner_name`, `issuer`, `fingerprint_sha256`, `valid_until`,
    `cert_type`, `provider`, `storage_kind`, `storage_ref`, `trust_status`,
    `usage`. Используется `vee-validate`. Кнопка «Save metadata»
    вызывает `useCreateMailCertificateMutation`.
- Запросы: `useMailCertificatesQuery`, `useExpiringMailCertificatesQuery(90)`.

---

## CommunicationsContextRail

Боковая панель контекста, отображаемая при выборе сообщения.

**Props:**

- `detail: CommunicationMessageDetailResponse | null`
- `projects: ProjectItem[]`
- `tasks: TaskItem[]`

**Содержимое:**

- Если `detail` отсутствует – заглушка «Select a message».
- При наличии `detail`:
  - **Sender** – аватар (первая буква имени), имя (`senderLabel`),
    email (`senderEmail`).
  - **Summary** – `ai_summary` или первые 200 символов `body_text`,
    либо «No summary».
  - **Related Projects** – список проектов с иконкой
    `tabler:briefcase`, либо «No related projects».
  - **Related Tasks** – список задач с иконкой `tabler:checkbox`,
    либо «No related tasks».

Компонент не генерирует событий.

---

## CommunicationsRailPane

Контейнер правой боковой панели. Переключает отображение между
`CommunicationsContextInspector` и `CommunicationsContextRail`.

**Props:**

- `detail: CommunicationMessageDetailResponse | null`
- `inspectorMode: InspectorMode`
- `projects: ProjectItem[]`
- `tasks: TaskItem[]`

**Emits:**

- `update:inspectorMode`

**Логика:**

- Если `detail` не равен `null`, рендерится `CommunicationsContextInspector`
  (с передачей `detail` и `inspectorMode`).
- Иначе – `CommunicationsContextRail` (с передачей `detail`, `projects`,
  `tasks`).

---

## CommunicationsDetailPane

Главная панель детального просмотра. Управляет отображением
одиночного сообщения или треда (цепочки сообщений).

**Props:**

- `detail: CommunicationMessageDetailResponse | null`
- `insight: CommunicationMessageInsight | null`
- `activeTab: MessageContextTab`
- `selectedThread: CommunicationThreadSummary | null`
- `threadMessages: ThreadMessage[]`
- `isThreadLoading: boolean`
- `threadErrorMessage: string`
- `isThreadReplySending: boolean`

**Emits:** (все события ретранслируются в родительский компонент)

- `update:activeTab`
- `reply`, `replyAll`, `forwardMessage`
- `redirectMessage` (с текстом получателей)
- `createTask`, `createNote`, `translate`
- `generateAiReply`, `applyAiReply`
- `reviewSecurity`, `reviewRecipients`, `analyze`
- `markMessageRead`, `markMessageUnread`
- `deleteFromProvider`, `togglePin`, `toggleImportant`, `mute`
- `exportMessage` (формат)
- `addLabel`, `removeLabel`
- `snoozeMessage` (until)
- `openCompose`
- `sendBilingualReply`
- `openThreadMessage`, `replyToThreadMessage`,
  `saveThreadReplyDraft`, `sendThreadReply`

**Логика:**

- Если `selectedThread` не `null` – рендерится `ThreadConversationView`.
- Иначе – `CommunicationViewer` с полным набором обработчиков.

---

## ComposeDrawer

Выдвижная панель (Sheet) для создания и отправки сообщений. Реализует
черновик, вложения, подписи, шаблоны и шаг подтверждения перед
отправкой.

**Основные возможности (из видимой части исходного кода):**

- Заголовок панели зависит от `composeForm.mode`:
  «Reply», «Forward» или «New Message».
- Поля формы: **To**, **CC**, **BCC**, **Subject**.
- Тело сообщения поддерживает два формата:
  `plain` (textarea) и `html` (RichComposeEditor или редактор source).
  Переключение режимов через `body-mode-toggle`.
- Вставка подписи (`ComposeSignaturePicker`) и шаблонов
  (`ComposeTemplatePicker`).
- Вложения: добавление через кнопку выбора файла и drag‑and‑drop.
  Файлы хранятся в локальном состоянии `stagedAttachments`. Отправка с
  вложениями в провайдер пока не реализована (соответствующая ошибка).
- Параметры доставки: запланированное время (`scheduledSendAt`) и
  задержка отмены (`undoSendSeconds`).
- Автосохранение черновика через `useComposeDraftAutosave`.
- Валидация через `useComposeValidation` (обязательность To, возможно, др. поля).
- Шаг **Review before sending**: отображает сводку полей и даёт кнопку
  «Send» / «Schedule» и кнопку «Edit» для возврата к редактированию.
- Действия: `handleSend` (отправка), `handleSaveDraft` (сохранение),
  `handleDeleteCurrentDraft` (удаление черновика), `handleClose` (закрытие с
  автосохранением).
- Сообщения об ошибках и статусные сообщения отображаются внутри формы.

**Стили:** определены в `ComposeDrawer.css` (макет, поля, тулбары,
зона вложений, шаг подтверждения и т.д.).

*Примечание: исходный файл обрезан – часть логики после 12000 символов
недоступна. Полное поведение может отличаться.*

---

## ComposeSignaturePicker

Выбор и вставка почтовой подписи из списка персон с непустыми
signature.

**Props:** отсутствуют.

**Emits:**

- `apply` – сигнатура для вставки (строка)

**Логика:**

- Получает список персон через `usePersonasQuery`.
- Фильтрует персоны, у которых `signature` не пуста.
- При выборе персоны и нажатии «Insert» вызывает `apply` с обрезанной
  сигнатурой.
- Кнопка «Insert» отключена, если сигнатура пуста или данные ещё
  загружаются.

---

## ComposeTemplatePicker

Панель выбора и применения шаблонов сообщений (rich-шаблоны).
Поддерживает библиотеку шаблонов, поиск, категории, предпросмотр,
редактирование переменных, mail‑merge и сохранение новых шаблонов.

**Props:**

- `toText: string`
- `ccText: string`
- `bccText: string`
- `subject: string`
- `body: string`
- `bodyHtml: string | null`

**Emits:**

- `apply` – объект `{ subject: string; bodyHtml: string }`
- `error` – сообщение об ошибке
- `saved` – имя сохранённого шаблона
- `deleted` – имя удалённого шаблона

**Ключевые элементы (из видимой части):**

- Поиск по названию/категории (`templateLibraryQuery`).
- Чипы категорий (`selectedCategory`).
- Список шаблонов с предпросмотром темы и тела, метаинформацией,
  диагностикой ошибок.
- Редактор переменных шаблона (`variableValues`).
- Предпросмотр mail‑merge с построчным вводом получателей и
  применением маппинга переменных (`recipientVariableMapping`).
- Кнопка «Apply» рендерит шаблон через `useRenderRichTemplateMutation`
  и вызывает `apply` с результатом.
- Возможность сохранить текущий контент как новый шаблон / дубликат
  (через `TemplateSaveForm`, валидацию `vee-validate`).
- Удаление шаблона с подтверждением.

*Примечание: исходный файл обрезан; документация покрывает только
первые ~12000 символов.*

---

## MessageAiReplyPanel

Панель генерации AI‑ответа на основе выбранного сообщения.

**Props:**

- `messageId: string | null`
- `insight: CommunicationMessageInsight | null`

**Emits:**

- `generateAiReply` – `{ tone: string; language: string }`
- `applyAiReply` – объект `AiReplyResponse`

**Элементы управления:**

- Выпадающие списки **Tone** (formal, business, friendly, short, detailed)
  и **Language** (English, Russian).
- Кнопка **Generate** вызывает `generateAiReply`.
- Кнопка **Variants** запрашивает несколько вариантов через
  `useGenerateAiReplyVariantsMutation`, отображает их с возможностью
  выбора «Apply».
- Сгенерированный ответ (`insight.aiReply`) показывает тему, тело,
  мета-теги (tone, language) и кнопку «Apply to compose».
- При ошибке вариантов выводится сообщение `ai-reply-error`.

---

## MessageAttachmentsTab

Вкладка просмотра вложений сообщения с таблицей, предпросмотром и
инспекцией архивов.

**Props:**

- `detail: CommunicationMessageDetailResponse | null`

**Таблица вложений:**

- Использует `@tanstack/vue-table`, колонки: filename, size, scan_status.
- Иконка вложения определяется через `attachmentIcon(content_type)`.
- Для превьюабельных вложений – кнопка «Preview image» / «Preview PDF» / «Preview».
- Для инспектируемых архивов – кнопка «Inspect archive».

**Панель предпросмотра (`attachment-preview-panel`):**

- Отображается при выборе вложения для предпросмотра.
- Показывает имя файла, статус сканирования.
- В зависимости от `preview_kind`:
  - `image` – `<img>` с `data_url`
  - `audio` – `<audio>` элемент
  - `video` – `<video>` элемент
  - `pdf` – `<iframe>` с `data_url`
  - `text` – `<pre>`, а также опция перевода с выбором языка и кнопкой
    «Translate preview». Результат перевода отображается в отдельной
    панели.
- Загрузка предпросмотра: `useAttachmentPreviewQuery`.
- Перевод: `useTranslateAttachmentMutation`.
- Ошибки и пустые состояния учтены.

**Инспекция архива (`archive-inspection-panel`):**

- Отображается при нажатии «Inspect archive».
- Использует `useAttachmentArchiveInspectionQuery`.
- После загрузки показывает содержимое архива (точное отображение
  обрезано).

*Примечание: исходный файл обрезан; часть разметки и логика
инспекции архива не попали в контекст.*
```

## Покрытие источников

- **`CommunicationsWorkbench.vue`** — имя компонента, сетка (две/три колонки), слоты `list`, `detail`, `rail`, классы `is-loading`/`has-error`.
- **`CommunicationsTopbarSlot.vue`** — name, props `searchQuery`/`isSyncBusy`, emits (search, openAccountSetup, compose, syncNow), кнопки и их иконки, анимация `spin-icon`.
- **`CommunicationsListPane.vue`** — name, props списка/тредов/пагинации, состояния ошибки/загрузки, условный рендеринг `CommunicationsConversationList` или `CommunicationList`, события `toggleSelection`, `selectVisible`, `clearSelection`, `update:navigatorMode`.
- **`CommunicationsConversationList.vue`** — name, режимы threads/contacts, группировка по email, сортировка контактов, prefetch `useThreadMessagesPrefetch`, индикаторы unread/attachments, load more threads.
- **`DraftStrip.vue`** — name, виртуализация `@tanstack/vue-virtual`, отображение черновиков, события `openDraft`, `deleteDraft`, `loadMore`.
- **`HealthStrip.vue`** — name, метрики (total, unread, action, waiting, important), цветовая индикация через `healthToneClass`.
- **`MailResourceOverviewStrip.vue`** — name, три колонки (subscriptions, top senders, blockers), виртуализация, кнопки «More», отображение первых 2 блокировщиков.
- **`MailCertificateStrip.vue`** — name, раскрывающаяся панель, список истекающих и хранимых сертификатов, форма добавления, поля формы, `useMailCertificatesQuery`, `useExpiringMailCertificatesQuery(90)`, мутация создания.
- **`CommunicationsContextRail.vue`** — name, отображаемые секции (Sender, Summary, Related Projects, Related Tasks), использование `senderLabel`/`senderEmail`, пустое состояние.
- **`CommunicationsRailPane.vue`** — name, условный рендеринг `CommunicationsContextInspector` / `CommunicationsContextRail`, emits `update:inspectorMode`.
- **`CommunicationsDetailPane.vue`** — name, условный рендеринг `ThreadConversationView` / `CommunicationViewer`, полный список пробрасываемых событий.
- **`ComposeDrawer.vue`** (truncated) — name, поля формы, форматы тела, подпись/шаблоны, вложения (staged), параметры доставки, автосохранение, валидация, шаг review, события отправки/сохранения/удаления, индикаторы состояния.
- **`ComposeDrawer.css`** — стили для drawer, полей, тулбара, вложений, review.
- **`ComposeSignaturePicker.vue`** — name, селект персон с подписями, кнопка «Insert», emit `apply`, состояния загрузки/пустоты.
- **`ComposeTemplatePicker.vue`** (truncated) — name, поиск, категории, список шаблонов, редактирование переменных, mail‑merge preview, кнопка «Apply», сохранение/удаление шаблонов.
- **`MessageAiReplyPanel.vue`** — name, выбор tone/language, генерация и отображение AI‑ответа, генерация вариантов, emit `generateAiReply`/`applyAiReply`.
- **`MessageAttachmentsTab.vue`** (truncated) — name, таблица вложений, колонки, кнопки предпросмотра/инспекции, панель предпросмотра (image/audio/video/pdf/text, перевод), панель инспекции архива.

## Исходные файлы

- [`frontend/src/domains/communications/components/CommunicationsContextRail.vue`](../../../../frontend/src/domains/communications/components/CommunicationsContextRail.vue)
- [`frontend/src/domains/communications/components/CommunicationsConversationList.vue`](../../../../frontend/src/domains/communications/components/CommunicationsConversationList.vue)
- [`frontend/src/domains/communications/components/CommunicationsDetailPane.vue`](../../../../frontend/src/domains/communications/components/CommunicationsDetailPane.vue)
- [`frontend/src/domains/communications/components/CommunicationsListPane.vue`](../../../../frontend/src/domains/communications/components/CommunicationsListPane.vue)
- [`frontend/src/domains/communications/components/CommunicationsRailPane.vue`](../../../../frontend/src/domains/communications/components/CommunicationsRailPane.vue)
- [`frontend/src/domains/communications/components/CommunicationsTopbarSlot.vue`](../../../../frontend/src/domains/communications/components/CommunicationsTopbarSlot.vue)
- [`frontend/src/domains/communications/components/CommunicationsWorkbench.vue`](../../../../frontend/src/domains/communications/components/CommunicationsWorkbench.vue)
- [`frontend/src/domains/communications/components/ComposeDrawer.css`](../../../../frontend/src/domains/communications/components/ComposeDrawer.css)
- [`frontend/src/domains/communications/components/ComposeDrawer.vue`](../../../../frontend/src/domains/communications/components/ComposeDrawer.vue)
- [`frontend/src/domains/communications/components/ComposeSignaturePicker.vue`](../../../../frontend/src/domains/communications/components/ComposeSignaturePicker.vue)
- [`frontend/src/domains/communications/components/ComposeTemplatePicker.css`](../../../../frontend/src/domains/communications/components/ComposeTemplatePicker.css)
- [`frontend/src/domains/communications/components/ComposeTemplatePicker.vue`](../../../../frontend/src/domains/communications/components/ComposeTemplatePicker.vue)
- [`frontend/src/domains/communications/components/DraftStrip.vue`](../../../../frontend/src/domains/communications/components/DraftStrip.vue)
- [`frontend/src/domains/communications/components/HealthStrip.vue`](../../../../frontend/src/domains/communications/components/HealthStrip.vue)
- [`frontend/src/domains/communications/components/MailCertificateStrip.vue`](../../../../frontend/src/domains/communications/components/MailCertificateStrip.vue)
- [`frontend/src/domains/communications/components/MailResourceOverviewStrip.vue`](../../../../frontend/src/domains/communications/components/MailResourceOverviewStrip.vue)
- [`frontend/src/domains/communications/components/MessageAiReplyPanel.vue`](../../../../frontend/src/domains/communications/components/MessageAiReplyPanel.vue)
- [`frontend/src/domains/communications/components/MessageAttachmentsTab.vue`](../../../../frontend/src/domains/communications/components/MessageAttachmentsTab.vue)

## Кандидаты на drift

Из предоставленного контекста не видно расхождений между кодом и
документированным поведением. Однако три файла предоставлены
обрезанными (`ComposeDrawer.vue`, `ComposeTemplatePicker.vue`,
`MessageAttachmentsTab.vue`). Для них описана только та часть логики,
которая попала в усечённый фрагмент; полное поведение может
отличаться, что потенциально может создать drift при последующем
добавлении недостающей информации. За пределами усечений
подтверждённых расхождений нет.

---
chunk_id: 137-other-frontend-part-010
batch_id: batch-20260628T214902
group: frontend
role: other
source_status: pending
source_count: 23
generated_by: code-wiki-ru
---

# 137-other-frontend-part-010 — frontend/other

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `23`

## Резюме

Страница `components/frontend.md` должна быть дополнена каталогом компонентов фронтенда, сгруппированных по доменам (Настройки, Задачи, Таймлайн, Интеграция почты). Основой служат исходные файлы, переданные в чанке. Для каждого компонента описывается его видимая из кода структура, свойства и поведение. Это позволит разработчикам и редакторам wiki получить обзорную документацию по фронтенд-слою, не обращаясь к исходникам напрямую.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Компоненты фронтенда

## Настройки (Settings)

### `SignalHubSettings`

Главная страница Signal Hub. Отображает вкладки (sources, profiles, policies, connections, runtime, health, replay), сводную полосу с агрегированными показателями (количество источников, включённых, состояние рантайма, перезапуски, подключения, здоровье, очередь воспроизведения, активный профиль) и кнопку «Restore Fixture».

- **Файл:** `frontend/src/domains/settings/components/SignalHubSettings.vue`
- **Импорты:** `SignalHubSourcesTab`, `SignalHubProfilesPoliciesTab`, `SignalHubOperationsTab`, `useSignalHubSettingsController`.
- **Управляет** переключением вкладок через `state.activeTab.value`.
- **Сводка:** `state.sources.value.length`, `state.enabledCount.value`, `state.activeRuntimeCount.value / state.runtimeCount.value`, `state.replayCount.value`, `state.connectedCount.value`, `state.unhealthyCount.value`, `state.replayPendingCount.value`, `state.activeProfile.value?.display_name ?? t('None')`.
- **Кнопка "Restore Fixture":** вызывает `state.handleRestoreFixture`, доступна пока `!state.isRestoringFixture.value`.

### `SignalHubSourcesTab`

Вкладка источников Signal Hub. Содержит каталог источников с левой стороны и инспектор выбранного источника — справа.

- **Файл:** `frontend/src/domains/settings/components/SignalHubSourcesTab.vue`
- **Фильтрация:** поисковый ввод `state.sourceSearch` и селектор категории `state.sourceCategory` (опции из `state.categories`, включая `'all'`).
- **Список:** рендерится из `filteredSources`. Каждый элемент — кнопка с иконкой, названием, кодом и пилюлей состояния `sourceControlState(policies, source)`.
- **Состояния загрузки/пустоты:** показывается «Loading sources...», «No matching sources.».
- **Инспектор:** при выборе источника (`selectedSource`) показывает:
  - код, версию схемы (`capability_schema_version`), состояние, дату обновления;
  - сетку возможностей (`capabilityLabels`);
  - таблицу capability с пилюлями состояния и флагом `requires_confirmation`;
  - кнопки «Enable Source» и «Disable Source» (зависят от `isUpdatingSignalControls`);
  - форму «Fixture Signal» (только если `source.code === 'fixture'`): выбор из `fixtureSources` по `fixture_id`, кнопка «Emit Fixture», отображение `event_type` и `raw_event_id` после отправки.

### `SignalHubProfilesPoliciesTab`

Вкладки профилей и политик. Включает форму создания/редактирования профиля и форму создания политики, а также списки существующих профилей и политик.

- **Файл:** `frontend/src/domains/settings/components/SignalHubProfilesPoliciesTab.vue`
- **Профили (вкладка `'profiles'`):**
  - Поля формы: Profile Code (`state.profileCodeInput`, заблокирован если есть `selectedProfile`), Display Name, Description, Scope (`event_pattern`, `source`, `connection`, `global`), Mode (`paused`, `muted`, `disabled`, `enabled`), Source (фильтруется по scope), Connection (если scope `connection`), Pattern (если scope `event_pattern`); Reason.
  - Возможность добавить черновую политику профиля (кнопка «Add Policy») и удалить её.
  - Список профилей: каждый элемент — кнопка с названием, кодом/количеством политик, описанием, пилюлей состояния (active/system/custom), и кнопка «Apply» либо статус «Applied».
  - Кнопка «Create Profile» / «Update Profile» зависит от наличия `selectedProfile`.
  - Системные профили (`is_system`) нельзя редактировать/удалять.
- **Политики (вкладка `'policies'`, не `'profiles'`):**
  - Форма создания политики: Scope, Mode, Source (если scope `source` или `connection`), Connection (если scope `connection`), Pattern (если scope `event_pattern`), Reason.
  - Кнопка «Create Policy»: `state.isCreatingPolicy.value || state.isUpdatingSignalControls.value` контролируют блокировку.
- **Обрезан исходный файл**; полное содержимое вкладки политик после формы создания не показано.

### `SignalHubOperationsTab`

Операционные вкладки: соединения, рантайм, мониторинг, воспроизведение.

- **Файл:** `frontend/src/domains/settings/components/SignalHubOperationsTab.vue`
- **Вкладка `'connections'`:**
  - Форма создания соединения: выбор source (из `connectionCapableSources`), Display Name, Profile.
  - Кнопка «Create Connection», блокируется при `isCreatingConnection`.
  - Список соединений: для каждого — иконка источника, display name, source_code / profile, сводка настроек (`formatSettingsSummary`), временная шкала (`formatConnectionTimeline`), пилюля статуса с тоном.
  - Действия: Connect, Pause, Mute, Disable, Remove. Каждая кнопка заблокирована, если `isUpdatingConnection` или статус уже целевой.
- **Вкладка `'runtime'`:**
  - Список состояний рантайма: для каждого — runtime_kind, source_code, временная шкала (`formatRuntimeTimeline`), ошибка (`formatRuntimeError`).
  - Действия: Run, Pause, Mute, Stop (блокировка по `isUpdatingRuntime` и целевому состоянию).
- **Вкладка `'health'`:**
  - Список health-записей: summary, source_code, статус (`formatHealthStatus`), свидетельство (`formatHealthEvidence`), пилюля уровня.
  - Кнопка «Run Check» (блокируется при `isRunningHealthCheck`).
- **Вкладка `'replay'`:**
  - Форма создания запроса на перезапуск: Source (фильтр по `supports_replay`), Connection (из `replayScopedConnections`), Event Pattern, Selector Mode (`all` / `position` / `time`), в зависимости от режима — поля From/To Position или From/To Time, Target Consumer (`replayTargetConsumers`), Target Projection (4 опции: communication_messages, person_derived_evidence, project_link_review_effects, timeline_event_log).
  - Кнопка Create Replay Request.
  - Список запросов на перезапуск (обрезан, видна только форма).

### `SignalHubSettings.css`

Сопутствующая таблица стилей для всех компонентов Signal Hub. Содержит селекторы для `.signal-tabs`, `.signal-summary-strip`, `.signal-sources-layout`, `.source-catalog`, `.source-inspector`, `.signal-table`, `.signal-pill`, `.policy-layout`, `.signal-runtime-row` и многих других. Адаптивная вёрстка: при ширине экрана ≤1100px изменяется раскладка.

- **Файл:** `frontend/src/domains/settings/components/SignalHubSettings.css`

### `AccentPicker`

Компонент выбора акцентного цвета. Отображает сетку кнопок с цветными кругами: Teal, Cyan, Blue, Violet, Amber, Rose. Активное состояние подсвечивается. Список идентификаторов берётся из `accentColorIds` (`platform/theme/settings`).

- **Файл:** `frontend/src/domains/settings/components/appearance/AccentPicker.vue`
- **Props:** `value` (`AccentColorId`), `title`, `description`.
- **Emits:** `change` с новым значением.

### `AppearanceHeader`

Заголовок раздела настройки внешнего вида. Показывает заголовок, описание, состояние сохранения (saving / saveStateLabel), возможное сообщение об ошибке, кнопку «Default» для сброса.

- **Файл:** `frontend/src/domains/settings/components/appearance/AppearanceHeader.vue`
- **Props:** `title`, `description`, `isSaving`, `saveStateLabel`, `persistenceError`.
- **Emits:** `reset`.

### `BackgroundPicker`

Выбор фонового изображения (shell background). Предлагает 11 вариантов: none, network-mesh, data-stream, node-frame, eclipse-grid, dna-blueprint, forest-network, forest-stream, knowledge-map, rune-gold, rune-teal. Каждый вариант имеет цветной предпросмотр и подпись. Идентификаторы — из `shellBackgroundIds`.

- **Файл:** `frontend/src/domains/settings/components/appearance/BackgroundPicker.vue`
- **Props:** `value` (`ShellBackgroundId`), `title`, `description`.
- **Emits:** `change`.

### `SpacingDensityControl`

Выбор плотности интерфейса: Compact, Normal, Comfortable. Идентификаторы из `spacingDensityIds`.

- **Файл:** `frontend/src/domains/settings/components/appearance/SpacingDensityControl.vue`
- **Props:** `value` (`SpacingDensity`), `title`, `description`.
- **Emits:** `change`.

### `ThemeRangeControl`

Ползунок для настройки числового параметра темы. Отображает метку, описание, текущее значение с единицей измерения, и input type="range" с заданными min, max, step. При движении ползунка генерирует событие `preview` (мгновенное), при отпускании — `commit`.

- **Файл:** `frontend/src/domains/settings/components/appearance/ThemeRangeControl.vue`
- **Props:** `id`, `label`, `description`, `value`, `min`, `max`, `step`, `unit`.
- **Emits:** `preview` (value: number), `commit`.

### `SidebarGroupEditor`

Редактор группы боковой панели. Состоит из заголовка с переименованием (input), кнопок перемещения вверх/вниз и удаления (кроме группы `'communications'`), и списка элементов `SidebarItemEditor`.

- **Файл:** `frontend/src/domains/settings/components/sidebar/SidebarGroupEditor.vue`
- **Props:** `group: SidebarNavGroup`, индексы, метки, опции групп, список скрытых элементов и текстовые метки.
- **Emits:** `rename`, `moveGroup`, `removeGroup`, `moveItemToGroup`, `moveItem`, `toggleDivider`, `toggleHidden`.

### `SidebarItemEditor`

Редактор отдельного элемента боковой панели. Показывает иконку, название, статус (показан/скрыт). Управляющие элементы: селект для перемещения в другую группу, кнопки вверх/вниз, переключатель разделителя (кроме первого элемента в группе), кнопка скрытия/отображения.

- **Файл:** `frontend/src/domains/settings/components/sidebar/SidebarItemEditor.vue`
- **Props:** `itemId`, `label`, `icon`, `hidden`, `statusLabel`, `moveTargetOptions`, `moveTargetValue`, `moveTargetPlaceholder`, `showDividerControl`, `dividerActive`, `dividerDisabled`, `dividerLabel`, `showLabel`, `hideLabel`.
- **Emits:** `moveToGroup`, `moveUp`, `moveDown`, `toggleDivider`, `toggleHidden`.

### `SidebarNavigationList`

Список корневых элементов боковой панели (группы и одиночные элементы). Для групп отображается иконка группы, название, количество элементов, кнопки перемещения и удаления. Одиночные элементы используют `SidebarItemEditor`.

- **Файл:** `frontend/src/domains/settings/components/sidebar/SidebarNavigationList.vue`
- **Props:** `entries: ResolvedSidebarRootEntry[]`, `hiddenItemIds`, `rootItemCount`, `groupOptions`, текстовые метки.
- **Emits:** `moveGroup`, `removeGroup`, `moveRootItem`, `moveItemToGroup`, `toggleHidden`.

### `SidebarSettingsSummary`

Правая панель сводки настроек боковой панели: предпросмотр структуры, список скрытых элементов с кнопкой «Show», список правил с бейджами.

- **Файл:** `frontend/src/domains/settings/components/sidebar/SidebarSettingsSummary.vue`
- **Props:** `entries`, `hiddenItemIds`, `itemLabels`, метки и `rules`.

### `SettingsPage`

Основная страница настроек. Содержит навигационное дерево с группами (General, Interface, Sources, Hub) и пунктами (Application, Language, Appearance, Sidebar, Integrations, Signal Hub, AI Hub). Для Integrations показывается счётчик провайдеров. При выборе раздела рендерится соответствующий компонент: `AppearanceSettings`, `LanguageSettings`, `ApplicationSettings`, `SidebarSettings`, `IntegrationsSettings`, `SignalHubSettings`, `AISettingsPanel`. Отображает сообщения об успехе (`store.actionMessage`) и ошибке (`store.errorMessage`).

- **Файл:** `frontend/src/domains/settings/views/SettingsPage.vue`
- **Хранилище:** `useSettingsStore` (управляет выбранным разделом).
- **Запрос:** `useApplicationSettingsQuery` — даёт `integrationCount`.

---

## Задачи (Tasks)

### `TaskList`

Виртуальный список активных задач и кандидатов. Объединяет `activeTasks` и `suggestedTaskCandidates`, разделяя их сепаратором «Review Queue». Использует `@tanstack/vue-virtual` для виртуализации.

- **Файл:** `frontend/src/domains/tasks/components/TaskList.vue`
- **Props:** `activeTasks: Task[]`, `suggestedTaskCandidates: TaskCandidate[]`, `isTasksLoading`, `setTaskCandidateReview`.
- **Строка-разделитель:** `task_candidate_id === '__separator__'`.
- **Активная задача:** чекбокс (disabled checked), название, источник (`taskSourceLabel`), проект (или «Unassigned»), время создания (`taskCreatedTime`), статус (`makosh_status`).
- **Кандидат:** название, источник, проект, уверенность (`taskConfidence`), кнопки «Confirm» и «Reject» (вызывают `setTaskCandidateReview` с `'user_confirmed'` или `'user_rejected'`).
- **Состояния:** при загрузке «Loading task state…», при отсутствии данных «No active tasks yet.».

### `TasksDecisionObligationReview`

Панель обзора решений и обязательств. Позволяет фильтровать по типу сущности (`DecisionEntityKind`: project, task, persona, communication, document, event, organization, knowledge) и идентификатору. Загружает список решений и обязательств (в том числе глобально, если идентификатор пуст). Для каждого элемента — кнопки «Confirm» и «Reject».

- **Файл:** `frontend/src/domains/tasks/components/TasksDecisionObligationReview.vue`
- **Props:** `decisions`, `obligations`, `entityKind`, `entityId`, `isLoading`, `error`, `reviewingItemId`, коллбэки на изменение скоупа, перезагрузку, ревью.
- **Отображение решения:** название, rationale, `formatDecisionEntity` + `formatDecisionTime`.
- **Отображение обязательства:** `statement`, `formatObligationEntity`, `risk_state`, `formatObligationDueTime`.

### `TasksPage`

Страница задач. Содержит метрики (активные задачи, предложенные кандидаты, состояние обзора), список `TaskList`, боковую панель с обзорной статистикой, `TasksDecisionObligationReview`, «Recent Candidate Signals» (первые 5 кандидатов) и «Active Task Sources».

- **Файл:** `frontend/src/domains/tasks/views/TasksPage.vue`
- **Запросы:** `useTaskCandidatesQuery`, `useTasksQuery`.
- **Хранилище:** `useTasksStore` — управляет списками решений, обязательств, состоянием загрузки, ошибками.
- **Загрузка контекстного обзора:** `loadContextReview` вызывает `fetchDecisions`/`fetchDecisionReviewItems` и `fetchObligations`/`fetchObligationReviewItems` в зависимости от наличия `entityId`.
- **Действия:** `setTaskCandidateReview`, `reviewDecisionItem`, `reviewObligationItem`.

---

## Таймлайн (Timeline)

### `TimelineFilters`

Фильтры таймлайна: чекбоксы для Messages, Documents, Tasks, Calendar, Notes, Decisions. Событие `toggleFilter` с ключом из `TimelineFiltersType`.

- **Файл:** `frontend/src/domains/timeline/components/TimelineFilters.vue`

### `TimelineStream`

Виртуальный поток сообщений таймлайна. Отображает события в хронологическом порядке с rail-dot, иконкой, отправителем, темой/превью тела и временем. Использует `useVirtualizer` из `@tanstack/vue-virtual`.

- **Файл:** `frontend/src/domains/timeline/components/TimelineStream.vue`
- **Сообщение:** `TimelineMessage` с полями `sender_display_name`, `sender`, `subject`, `body_text_preview`, `occurred_at`, `projected_at`.

### `TimelinePage`

Страница таймлайна. Состоит из `TimelineStream` и боковой панели с `TimelineFilters`. Данные загружаются через `useTimelineMessagesQuery` и сохраняются в `useTimelineStore`.

- **Файл:** `frontend/src/domains/timeline/views/TimelinePage.vue`

---

## Интеграция почты (Mail Integration)

### `AccountSetupModal`

Модальное окно для добавления почтового аккаунта. Два шага: выбор провайдера (Gmail, iCloud, IMAP) и ввод учётных данных. Использует `vee-validate` для валидации.

- **Файл:** `frontend/src/integrations/mail/components/AccountSetupModal.vue`
- **Шаг 1:** три кнопки с иконками и описаниями.
- **Шаг 2:** поля зависят от провайдера:
  - Для IMAP: display_name, email, IMAP хост/порт, username, password, SMTP хост/порт, чекбоксы IMAP TLS, SMTP TLS, SMTP STARTTLS.
  - Для iCloud: display_name, email, App Password.
  - Для Gmail: display_name, email (инициируется OAuth через `useStartGmailOAuthSetupMutation`, открывается URL авторизации).
- **Валидация:** `accountSetupVeeValidationSchema`.
- **Отправка:** через `imapEmailAccountSetupMutation` или `gmailOAuthSetupMutation`.
- **Навигация:** кнопка «Back» на шаге 2.
- **Обрезан исходный файл** — полный CSS и часть шаблона не показаны.

### `MailSyncSettingsStrip`

Панель настроек синхронизации почтового провайдера. Позволяет включить/отключить синхронизацию (`sync_enabled`), задать размер пакета (`batch_size`) и интервал опроса (`poll_interval_seconds`). Кнопка «Save» отправляет форму.

- **Файл:** `frontend/src/integrations/mail/components/MailSyncSettingsStrip.vue`
- **Props:** `settings: MailSyncSettings | null`, `isLoading`, `isSaving`.
- **Emits:** `update` с `MailSyncSettingsUpdate`.
- **Использует `vee-validate`:** `syncSettingsVeeValidationSchema`, `syncSettingsFormDefaults`, `syncSettingsFormToUpdate`.
- **Состояния:** когда `isLoading` – «Loading settings...», иначе «Enabled»/«Paused».
```

## Покрытие источников

- `SignalHubOperationsTab.vue` — внешний вид и поведение вкладок: соединения, рантайм, здоровье, перезапуск; поля формы, действия, сводные тексты.
- `SignalHubProfilesPoliciesTab.vue` — форма профиля (поля, блокировки, черновые политики), список профилей; начало формы политики (scope, mode, source, connection, pattern, reason).
- `SignalHubSettings.css` — полный набор CSS-классов и адаптивная вёрстка.
- `SignalHubSettings.vue` — структура страницы, вкладки, сводка, кнопка "Restore Fixture".
- `SignalHubSourcesTab.vue` — каталог источников, инспектор, fixture-форма.
- `AccentPicker.vue` — выбор акцентного цвета, идентификаторы из `accentColorIds`.
- `AppearanceHeader.vue` — заголовок с состоянием сохранения и сбросом.
- `BackgroundPicker.vue` — выбор фона, идентификаторы из `shellBackgroundIds`.
- `SpacingDensityControl.vue` — три варианта плотности.
- `ThemeRangeControl.vue` — ползунок с preview/commit.
- `SidebarGroupEditor.vue` — редактирование группы, перемещение, удаление.
- `SidebarItemEditor.vue` — редактирование элемента панели, скрытие/разделитель/перемещение.
- `SidebarNavigationList.vue` — корневые группы и элементы.
- `SidebarSettingsSummary.vue` — предпросмотр, скрытые элементы, правила.
- `SettingsPage.vue` — дерево навигации, разделы, интеграционный счётчик.
- `TaskList.vue` — виртуальный список задач и кандидатов, подтверждение/отклонение.
- `TasksDecisionObligationReview.vue` — обзор решений и обязательств, фильтрация по сущности.
- `TasksPage.vue` — метрики, список, боковая панель, загрузка контекстного обзора.
- `TimelineFilters.vue` — чекбоксы для типов событий.
- `TimelineStream.vue` — виртуальный хронологический поток сообщений.
- `TimelinePage.vue` — страница таймлайна, загрузка данных.
- `AccountSetupModal.vue` — двухшаговый мастер, поля для IMAP/iCloud/Gmail, валидация, OAuth.
- `MailSyncSettingsStrip.vue` — тумблер синхронизации, batch, poll, сохранение.

## Исходные файлы

- [`frontend/src/domains/settings/components/SignalHubOperationsTab.vue`](../../../../frontend/src/domains/settings/components/SignalHubOperationsTab.vue)
- [`frontend/src/domains/settings/components/SignalHubProfilesPoliciesTab.vue`](../../../../frontend/src/domains/settings/components/SignalHubProfilesPoliciesTab.vue)
- [`frontend/src/domains/settings/components/SignalHubSettings.css`](../../../../frontend/src/domains/settings/components/SignalHubSettings.css)
- [`frontend/src/domains/settings/components/SignalHubSettings.vue`](../../../../frontend/src/domains/settings/components/SignalHubSettings.vue)
- [`frontend/src/domains/settings/components/SignalHubSourcesTab.vue`](../../../../frontend/src/domains/settings/components/SignalHubSourcesTab.vue)
- [`frontend/src/domains/settings/components/appearance/AccentPicker.vue`](../../../../frontend/src/domains/settings/components/appearance/AccentPicker.vue)
- [`frontend/src/domains/settings/components/appearance/AppearanceHeader.vue`](../../../../frontend/src/domains/settings/components/appearance/AppearanceHeader.vue)
- [`frontend/src/domains/settings/components/appearance/BackgroundPicker.vue`](../../../../frontend/src/domains/settings/components/appearance/BackgroundPicker.vue)
- [`frontend/src/domains/settings/components/appearance/SpacingDensityControl.vue`](../../../../frontend/src/domains/settings/components/appearance/SpacingDensityControl.vue)
- [`frontend/src/domains/settings/components/appearance/ThemeRangeControl.vue`](../../../../frontend/src/domains/settings/components/appearance/ThemeRangeControl.vue)
- [`frontend/src/domains/settings/components/sidebar/SidebarGroupEditor.vue`](../../../../frontend/src/domains/settings/components/sidebar/SidebarGroupEditor.vue)
- [`frontend/src/domains/settings/components/sidebar/SidebarItemEditor.vue`](../../../../frontend/src/domains/settings/components/sidebar/SidebarItemEditor.vue)
- [`frontend/src/domains/settings/components/sidebar/SidebarNavigationList.vue`](../../../../frontend/src/domains/settings/components/sidebar/SidebarNavigationList.vue)
- [`frontend/src/domains/settings/components/sidebar/SidebarSettingsSummary.vue`](../../../../frontend/src/domains/settings/components/sidebar/SidebarSettingsSummary.vue)
- [`frontend/src/domains/settings/views/SettingsPage.vue`](../../../../frontend/src/domains/settings/views/SettingsPage.vue)
- [`frontend/src/domains/tasks/components/TaskList.vue`](../../../../frontend/src/domains/tasks/components/TaskList.vue)
- [`frontend/src/domains/tasks/components/TasksDecisionObligationReview.vue`](../../../../frontend/src/domains/tasks/components/TasksDecisionObligationReview.vue)
- [`frontend/src/domains/tasks/views/TasksPage.vue`](../../../../frontend/src/domains/tasks/views/TasksPage.vue)
- [`frontend/src/domains/timeline/components/TimelineFilters.vue`](../../../../frontend/src/domains/timeline/components/TimelineFilters.vue)
- [`frontend/src/domains/timeline/components/TimelineStream.vue`](../../../../frontend/src/domains/timeline/components/TimelineStream.vue)
- [`frontend/src/domains/timeline/views/TimelinePage.vue`](../../../../frontend/src/domains/timeline/views/TimelinePage.vue)
- [`frontend/src/integrations/mail/components/AccountSetupModal.vue`](../../../../frontend/src/integrations/mail/components/AccountSetupModal.vue)
- [`frontend/src/integrations/mail/components/MailSyncSettingsStrip.vue`](../../../../frontend/src/integrations/mail/components/MailSyncSettingsStrip.vue)

## Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не обнаружено. Документация на русском языке для этих компонентов ранее не была встроена в контекст, поэтому сверить не с чем.

---
chunk_id: 136-other-frontend-part-009
batch_id: batch-20260628T214902
group: frontend
role: other
source_status: pending
source_count: 23
generated_by: code-wiki-ru
---

# 136-other-frontend-part-009 — frontend/other

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `23`

## Резюме

Создать или обновить страницу `components/frontend.md` в русской Obsidian wiki – документацию архитектуры и ключевых компонентов фронтенда Макошь. Страница должна описать структуру доменов (`notes`, `organizations`, `personas`, `projects`, `review`, `settings`), перечислить входящие в них view-страницы и UI-компоненты, отметить общие паттерны (i18n, виртуализация через `@tanstack/vue-virtual`, компоновка `widget-frame`/`panel`, иконки `tabler:`, управление состоянием через Pinia-сторы и TanStack Query). Основанием служат только предоставленные исходные файлы.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Frontend Components

## Архитектура

Frontend построен по доменному принципу в `frontend/src/domains/`. Каждый домен содержит:

- `views/` – страничные компоненты (верхнеуровневая компоновка)
- `components/` – переиспользуемые UI-блоки домена
- `stores/` – Pinia-сторы домена
- `queries/` – обёртки TanStack Query для загрузки данных
- `types/` – TypeScript-типы домена
- `api/` – клиентские вызовы к API (присутствует в settings)

Все компоненты используют `<script setup lang="ts">` (Composition API).

## Домены и компоненты

### Notes

- **`NotesPage.vue`** – страница заметок; собирает фильтры, список и панель Insights.
- **`NotesList.vue`** – виртуализированный список заметок с локальным поиском (`placeholder="Search notes..."`) и состоянием пустого списка (`"No notes found"`). Поддерживает пропы `notes: NoteItem[]` и `searchQuery: string`. Событие `update:search-query` передаёт значение поиска родителю.
- **`NotesSourceFilters.vue`** – боковая панель с чекбоксами фильтрации по источникам (`Apple Notes`, `Obsidian`, `Gmail`, `Anytype`, `Outlook`) и тегам (`#project`, `#research`, `#meeting`, `#idea`, `#reference`, `#partnership`). Генерирует события `toggle-source` и `toggle-tag`.

### Organizations

- **`OrganizationsPage.vue`** – страница компаний; макет `org-layout` с колонками `320px 1fr`. Управляет выбором организации через `selectedOrganizationId`. Импортирует `OrganizationsList` и `OrganizationsDetail`.
- **`OrganizationsList.vue`** – список организаций; принимает `organizations: Organization[]`, `selectedOrganizationId`, флаг загрузки. Каждая строка – кнопка с `org.organization_id`, классом `active` при совпадении. Отображает `display_name`, `industry`, `country`, `status` и `watchlist`.
- **`OrganizationsDetail.vue`** – детальная карточка выбранной организации. При `selectedOrganization` показывает блоки: Status, About, Details (Website, Legal name, Registration, VAT, Interactions, Priority), Key People. При отсутствии выбора – заглушка `"No company selected"`.

### Personas

- **`PersonsPage.vue`** – страница персон; компоновка из `PersonsList`, `PersonsDetail` и `stacked-rail` с виджетами: AI Summary, `PersonsIdentityReview`, `PersonsIdentityTraceReview`, `PersonsRelationshipReview`, Related Documents, Recent Notes.
- **`PersonsList.vue`** – виртуализированный список персон с локальным поиском. Строка содержит `name`, `role`, `company`, `status`/`channel`.
- **`PersonsDetail.vue`** – детализация персоны: hero с именем/ролью/компанией/статусом, кнопки действий (mail, phone, video, whatsapp – все `disabled`), вкладки (Overview активна, остальные disabled), блоки: Person Information (жёстко заданные данные), Persona Dossier (с поддержкой загрузки/ошибки), Relationship Strength (числовая оценка 85), Recent Interactions (до 3 элементов), Active Projects (до 3).
- **`PersonsIdentityReview.vue`** – панель Person Identity Review. Показывает `suggestedIdentityCandidates` и `confirmedMergeIdentityCandidates`. Для каждого кандидата: кнопки Confirm/Reject, для подтверждённых слияний – Split. Поддерживает загрузку, ошибки, пустое состояние.
- **`PersonsIdentityTraceReview.vue`** – панель Unattached Traces. Отображает только трассы с `person_id === null`. Позволяет выбрать целевую персону из выпадающего списка (формируется из `persons`) и привязать трассу кнопкой Assign.
- **`PersonsRelationshipReview.vue`** – панель Relationship Review. Показывает связи в состоянии `suggested`. Для каждой: тип связи, peer-идентификатор (вычисляется через `relationshipPeer`), оценки trust/strength/confidence. Кнопки Confirm/Reject.

### Projects

- **`ProjectsPage.vue`** – страница проектов; связывает `ProjectsHero`, `ProjectsDashboard` и `ProjectsRail`. При отсутствии выбранного проекта дашборд скрыт.
- **`ProjectsHero.vue`** – верхний блок: состояния ошибки/пустоты/загрузки, карточка проекта с названием, статусом, описанием, кнопкой "Prepare brief" (disabled), мета-полоса (Owner, People, Start Date, Target Date, Progress с `<progress>`), переключатель проектов (если >1), вкладки (Overview активна, остальные disabled с количеством).
- **`ProjectsDashboard.vue`** – центральная панель: Project Summary (числа документов, сообщений, людей, связей графа), Knowledge Graph (радиальное представление с чипами), Project Timeline (с обработкой пустого состояния), Recent Communications, Top Documents, Source Evidence, Open Promises (заглушка "No task candidates").
- **`ProjectsRail.vue`** – боковая панель: Project Health (статус, прогресс, связи), Key People (список с email и количеством взаимодействий), Related Projects (до 4 связанных проектов).

### Review

- **`ReviewPage.vue`** – Review Workspace. Содержит:
  - Header с кнопкой Refresh.
  - Баннер ошибки.
  - Метрики: Review Items, Suggested, Relationships, Decisions, Obligations, Polygraph.
  - Панели: Canonical Inbox (элементы с действиями: Approve, Take, Dismiss, Promote, Archive; поля целевого домена/сущности), Relationships (Confirm/Reject), Decisions, Obligations, Polygraph (часть за пределами включённых 12000 символов – обрезана).
  - Использует функции-помощники: `reviewItemKindLabel`, `formatItemTime`, `canPromote`, `deriveDefaultPromotion` и др.
  - Промоушен элемента требует заполнения полей `target_domain`, `target_entity_kind`, `target_entity_id`.

### Settings

- **Legacy `AISettingsControlCenter.vue`** – удалённый статический плейсхолдер. Текущая реализация заменена на `AISettingsPanel.vue`, а пользовательская поверхность называется **AI Hub** и живёт в группе настроек **Hub**.
- **`AppearanceSettings.vue`** – настройки внешнего вида. Управляет: Shell Background, Shell Brightness (через `ThemeRangeControl` с предпросмотром), Accent Color, Panel Opacity, Panel Blur, Spacing Density. Использует `useThemeStore` и подкомпоненты `AccentPicker`, `BackgroundPicker`, `ThemeRangeControl`, `SpacingDensityControl`, `AppearanceHeader`. Сохранение через `saveThemePatch`/`commitThemeSettings`; сброс до дефолта.
- **`ApplicationSettings.vue`** – несекретные настройки приложения. Группирует настройки по категориям: General, Interface, AI, Privacy, Notifications. Каждая строка: label, description, `setting_key`, мета-флаги (Bootstrap, Restart, `env_var`). Элемент управления: select (если есть `allowed_values`), checkbox (для boolean), number, text. Кнопка Save появляется при изменении. Использует `useSettingsStore` и `groupSettingsByCategory`.
- **`IntegrationsSettings.vue`** – управление учётными записями. Группирует аккаунты: Mail (gmail, icloud, imap), Zoom (zoom_user, zoom_server_to_server), Yandex Telemost, Other. Для каждого: иконка, имя, провайдер, статус. При выборе – панель инспектора с деталями. Для почтовых аккаунтов доступны Export, Logout, Delete; для Zoom/Yandex Telemost – оболочки `ZoomSettingsPanelShell` / `YandexTelemostSettingsPanelShell`. Импорт почтовых настроек через JSON. (Файл обрезан после 12000 символов – часть стилей и Zoom/Telemost оболочки не включены.)
- **`LanguageSettings.vue`** – выбор языка интерфейса из списка: English (`en`), Русский (`ru`). Сохраняет выбор через `saveApplicationSetting(FRONTEND_LOCALE_SETTING_KEY, value)` с откатом при ошибке.
- **`SidebarSettings.vue`** – настройка боковой панели. Позволяет: добавлять/удалять/переименовывать группы, перемещать элементы и группы, скрывать/показывать домены. Использует `useSidebarStore`. Сохраняет состояние через `saveApplicationSetting(FRONTEND_SIDEBAR_SETTING_KEY, ...)`. Компоновка из двух колонок: редактор (`SidebarGroupEditor`, `SidebarNavigationList`) и сводка (`SidebarSettingsSummary`).

## Общие паттерны

- **i18n** – все компоненты получают функцию перевода через `const { t } = useI18n()` из `platform/i18n`. Тексты интерфейса оборачиваются в `t('key')`.
- **Виртуализация** – списки с потенциально большим числом элементов (`NotesList`, `PersonsList`) используют `@tanstack/vue-virtual`. Применяются `useVirtualizer`, `virtualItems`, `totalSize`, абсолютное позиционирование строк через `translateY`.
- **Иконки** – через компонент `Icon` с набором `tabler:*`. Размер задаётся пропом `:size`.
- **Вёрстка** – распространены классы: `widget-frame`, `panel`, `info-card`, `round-icon`, `primary-button`, `inline-error`, `muted-copy`. Активно используются CSS-переменные (`--hh-border`, `--hh-color-text-bright`, `--hh-radius-md` и т.д.).
- **Состояния** – загрузка/ошибка/пусто обрабатываются через условный рендеринг (`v-if="isLoading"`, `v-if="error"`, `v-else-if="data.length === 0"`). Кнопки действий блокируются (`disabled`) при отсутствии данных или выполнении операции.
- **Управление данными** – данные загружаются через TanStack Query (хуки вида `use*Query()`). Локальное состояние – `ref`, `computed`. Мутации и бизнес-логика вынесены в Pinia-сторы (импортируются из `../stores/...`).
```

## Покрытие источников

| Source File | Covered Facts |
|---|---|
| `frontend/src/domains/notes/components/NotesList.vue` | Виртуализация через `@tanstack/vue-virtual`, пропсы `notes`, `searchQuery`, событие `update:search-query`, локальный поиск, пустое состояние `"No notes found"`. |
| `frontend/src/domains/notes/components/NotesSourceFilters.vue` | Фильтры по источникам (`Apple Notes` … `Outlook`) и тегам (`#project` … `#partnership`), события `toggle-source`, `toggle-tag`. |
| `frontend/src/domains/notes/views/NotesPage.vue` | Компоновка из `NotesSourceFilters`, `NotesList`, `NotesInsights`; fallback-заметки при отсутствии данных; использование `useNotesStore` и `useNotesQuery`. |
| `frontend/src/domains/organizations/components/OrganizationsDetail.vue` | Проп `selectedOrganization: Record<string, unknown> \| null`; детализация: Status, About, Details, Key People; пустое состояние `"No company selected"`. |
| `frontend/src/domains/organizations/components/OrganizationsList.vue` | Список с пропами `organizations: Organization[]`, `selectedOrganizationId`, загрузка, событие `selectOrg`; стили активной строки. |
| `frontend/src/domains/organizations/views/OrganizationsPage.vue` | Макет `org-layout 320px 1fr`; логика выбора организации через `selectedOrganizationId`; использование `useOrganizationsQuery`. |
| `frontend/src/domains/personas/components/PersonsDetail.vue` | Детализация: hero, вкладки, блоки информации, досье, сила отношений, взаимодействия, проекты; кнопки действий disabled. |
| `frontend/src/domains/personas/components/PersonsIdentityReview.vue` | Панель подтверждения слияния/разделения персон; пропы для `suggestedIdentityCandidates`, `confirmedMergeIdentityCandidates`; загрузка/ошибка/пусто. |
| `frontend/src/domains/personas/components/PersonsIdentityTraceReview.vue` | Панель непривязанных traces; фильтрация по `person_id === null`; выбор персоны и кнопка Assign; проп `onAssign`. |
| `frontend/src/domains/personas/components/PersonsRelationshipReview.vue` | Панель предложенных связей; фильтрация `review_state === 'suggested'`; Confirm/Reject; вывод trust/strength/confidence. |
| `frontend/src/domains/personas/components/PersonsList.vue` | Виртуализированный список; пропы `personList`, `selectedPersonIndex`; событие `selectPerson`; строка с `name`, `role`, `company`, `status`. |
| `frontend/src/domains/personas/views/PersonsPage.vue` | Компоновка `PersonsList`, `PersonsDetail`, rail с AI Summary, identity review, trace review, relationship review, related documents, recent notes; маппинг `personList`. |
| `frontend/src/domains/projects/components/ProjectsDashboard.vue` | Панели: Summary, Knowledge Graph, Timeline, Communications, Documents, Source Evidence, Open Promises; обработка пустых состояний. |
| `frontend/src/domains/projects/components/ProjectsHero.vue` | Hero: состояние ошибки/пустоты/загрузки; мета-полоса; переключатель проектов; вкладки disabled. |
| `frontend/src/domains/projects/components/ProjectsRail.vue` | Боковая панель: Project Health, Key People, Related Projects; обработка `muted-copy` при отсутствии данных. |
| `frontend/src/domains/projects/views/ProjectsPage.vue` | Связка `ProjectsHero`, `ProjectsDashboard`, `ProjectsRail`; вычисляемые `selectedProjectRecord`, `selectedProjectStats`, `relatedProjectSummaries`. |
| `frontend/src/domains/review/views/ReviewPage.vue` (частично, обрезан) | Review Workspace: метрики, Canonical Inbox с действиями (Approve, Take, Dismiss, Promote, Archive), Relationships, Decisions, Obligations, Polygraph; форма промоута. |
| `frontend/src/domains/settings/components/AISettingsControlCenter.vue` | Legacy-плейсхолдер, удалён в пользу `AISettingsPanel.vue` и AI Hub surface. |
| `frontend/src/domains/settings/components/AppearanceSettings.vue` | Настройки внешнего вида: Shell Background, Brightness, Accent Color, Opacity, Blur, Spacing Density; подкомпоненты `AccentPicker`, `ThemeRangeControl` и др. |
| `frontend/src/domains/settings/components/ApplicationSettings.vue` | Несекретные настройки, группировка по категориям, автоопределение типа контрола, кнопка Save при изменении. |
| `frontend/src/domains/settings/components/IntegrationsSettings.vue` (частично, обрезан) | Учётные записи: группировка Mail/Zoom/Yandex/Other; инспектор; экспорт/импорт/удаление почтовых аккаунтов. |
| `frontend/src/domains/settings/components/LanguageSettings.vue` | Выбор языка интерфейса (`en`/`ru`), сохранение через `saveApplicationSetting`. |
| `frontend/src/domains/settings/components/SidebarSettings.vue` | Настройка боковой панели: группы, перемещение, скрытие; редактор и сводка; сохранение через `saveApplicationSetting`. |

## Исходные файлы

- [`frontend/src/domains/notes/components/NotesList.vue`](../../../../frontend/src/domains/notes/components/NotesList.vue)
- [`frontend/src/domains/notes/components/NotesSourceFilters.vue`](../../../../frontend/src/domains/notes/components/NotesSourceFilters.vue)
- [`frontend/src/domains/notes/views/NotesPage.vue`](../../../../frontend/src/domains/notes/views/NotesPage.vue)
- [`frontend/src/domains/organizations/components/OrganizationsDetail.vue`](../../../../frontend/src/domains/organizations/components/OrganizationsDetail.vue)
- [`frontend/src/domains/organizations/components/OrganizationsList.vue`](../../../../frontend/src/domains/organizations/components/OrganizationsList.vue)
- [`frontend/src/domains/organizations/views/OrganizationsPage.vue`](../../../../frontend/src/domains/organizations/views/OrganizationsPage.vue)
- [`frontend/src/domains/personas/components/PersonsDetail.vue`](../../../../frontend/src/domains/personas/components/PersonsDetail.vue)
- [`frontend/src/domains/personas/components/PersonsIdentityReview.vue`](../../../../frontend/src/domains/personas/components/PersonsIdentityReview.vue)
- [`frontend/src/domains/personas/components/PersonsIdentityTraceReview.vue`](../../../../frontend/src/domains/personas/components/PersonsIdentityTraceReview.vue)
- [`frontend/src/domains/personas/components/PersonsList.vue`](../../../../frontend/src/domains/personas/components/PersonsList.vue)
- [`frontend/src/domains/personas/components/PersonsRelationshipReview.vue`](../../../../frontend/src/domains/personas/components/PersonsRelationshipReview.vue)
- [`frontend/src/domains/personas/views/PersonsPage.vue`](../../../../frontend/src/domains/personas/views/PersonsPage.vue)
- [`frontend/src/domains/projects/components/ProjectsDashboard.vue`](../../../../frontend/src/domains/projects/components/ProjectsDashboard.vue)
- [`frontend/src/domains/projects/components/ProjectsHero.vue`](../../../../frontend/src/domains/projects/components/ProjectsHero.vue)
- [`frontend/src/domains/projects/components/ProjectsRail.vue`](../../../../frontend/src/domains/projects/components/ProjectsRail.vue)
- [`frontend/src/domains/projects/views/ProjectsPage.vue`](../../../../frontend/src/domains/projects/views/ProjectsPage.vue)
- [`frontend/src/domains/review/views/ReviewPage.vue`](../../../../frontend/src/domains/review/views/ReviewPage.vue)
- [`frontend/src/domains/settings/components/AISettingsControlCenter.vue`](../../../../frontend/src/domains/settings/components/AISettingsControlCenter.vue)
- [`frontend/src/domains/settings/components/AppearanceSettings.vue`](../../../../frontend/src/domains/settings/components/AppearanceSettings.vue)
- [`frontend/src/domains/settings/components/ApplicationSettings.vue`](../../../../frontend/src/domains/settings/components/ApplicationSettings.vue)
- [`frontend/src/domains/settings/components/IntegrationsSettings.vue`](../../../../frontend/src/domains/settings/components/IntegrationsSettings.vue)
- [`frontend/src/domains/settings/components/LanguageSettings.vue`](../../../../frontend/src/domains/settings/components/LanguageSettings.vue)
- [`frontend/src/domains/settings/components/SidebarSettings.vue`](../../../../frontend/src/domains/settings/components/SidebarSettings.vue)

## Кандидаты на drift

1. **`OrganizationsDetail.vue`**: проп `selectedOrganization` объявлен как `Record<string, unknown>`, но шаблон оперирует полями вроде `display_name`, `industry`, `website`, характерными для типа `Organization`. Импорт этого типа отсутствует, в отличие от `OrganizationsList.vue`. Возможно несоответствие типизации.
2. **`NotesPage.vue`**: импортирует `NotesInsights` из `../components/NotesInsights.vue`, однако этот файл не предоставлен в контексте. Wiki может потребовать обновления после появления компонента.
3. **`PersonsDetail.vue`**: содержит захардкоженные данные (телефон `+1 (555) 123-4567`, город `New York, USA`), которые могут быть заменены реальными данными в будущем. Это временное состояние, не отражённое в документации.
4. **`PersonsPage.vue`**: в вычисляемом `personList` поле `role` заполняется значением `p.preferred_channel`, а `company` – полем `p.email_address`. Такое сопоставление выглядит подозрительно; возможно, поля перепутаны, что может вызвать расхождение с реальной моделью `PersonItem`.
5. **Множество отключённых кнопок**: в `PersonsDetail` (mail/phone/video/whatsapp), `ProjectsHero` ("Prepare brief"), `PersonsList` ("New Person"), `AISettingsControlCenter` ("Test Connection", "Run Diagnostics", "View Logs"). Это указывает на запланированный, но не реализованный функционал. Wiki должна отражать текущее состояние, но важно отметить эти места как «ожидающие реализации».
6. **`AISettingsControlCenter.vue`**: жёстко заданные значения статус-карточек (`llama3.2:3b`, `v0.5.0`) могут не синхронизироваться с реальным backend-состоянием – потенциальный drift между UI и фактическими данными.
7. **`ReviewPage.vue`** обрезан после 12000 символов; часть панелей (Decisions, Obligations, Polygraph) не видна в предоставленном фрагменте. Полная документация этих секций невозможна без полного файла.
8. **`IntegrationsSettings.vue`** обрезан после 12000 символов; оболочки `ZoomSettingsPanelShell` и `YandexTelemostSettingsPanelShell` импортированы, но их содержимое не включено – поведение Zoom/Yandex Telemost панелей не подтверждено контекстом.

### Summary / Резюме

Целевая страница `components/frontend.md` отсутствует в предоставленном контексте, но должна быть создана или обновлена. На основе встроенных исходных файлов необходимо задокументировать архитектуру фронтенд-компонентов приложения Макошь: оболочку (`AppShell`, `Topbar`, `Sidebar`, `NotificationsDrawer`, `LayoutEditControls`), представления-обёртки (14 view-компонентов), доменные компоненты (`AgentsDetail`, `AgentsGrid`), используемые хранилища (stores) и ключевые CSS-переменные. Все утверждения в предлагаемой странице напрямую подкреплены приложенным кодом.

### Proposed pages / Предлагаемые страницы

#### `components/frontend.md`

```markdown
# Фронтенд-компоненты

## Обзор

Фронтенд приложения построен на Vue 3 + TypeScript и использует композиционный API (`<script setup>`). Маршрутизация осуществляется через `vue-router`, иконки — через `@iconify/vue`. Визуальная тема и многие аспекты поведения управляются глобальными хранилищами Pinia (доступны по импортам из `../../shared/stores/...`).

## Оболочка приложения (Shell)

### AppShell (`frontend/src/app/shell/AppShell.vue`)

Корневой компонент оболочки. Монтируется в `App.vue` (`frontend/src/app/App.vue`), который просто рендерит `<AppShell />`.

**Структура:**
- `<RouterView />` — точка вставки текущего представления.
- `<Sidebar />` — боковая панель навигации.
- `<Topbar />` — верхняя панель с заголовком, статусом и элементами управления.
- `<NotificationsDrawer />` — выдвижная панель уведомлений.

**Логика:**
- При монтировании вызывает `theme.hydrateThemeSettings()` для применения сохранённых настроек темы.
- Отслеживает `route.name` и `route.query.section` и синхронизирует хранилище навигации через `nav.syncFromRoute(name, section)`.

**Вёрстка:**
- Контейнер `.viewport-guard` занимает `100vw × 100vh` с `overflow: hidden`.
- Рабочая область `.desktop-shell` построена на CSS Grid с двумя колонками: ширина боковой панели и `minmax(--hh-shell-content-min-width, 1fr)`.
- При переходе в режим «rail» (свёрнутая боковая панель) ширина колонки боковой панели меняется на `--hh-shell-sidebar-width-rail`. Переход анимирован (280ms, кубическая кривая).
- Внутри рабочей области — flex-контейнер с `<Topbar />`, `<NotificationsDrawer />` и `<main.workspace-content>`.

### Sidebar (`frontend/src/app/shell/Sidebar.vue`)

Боковая панель навигации. Использует хранилища `useNavigationStore` и `useSidebarStore`.

**Элементы:**
- **Бренд:** логотип `/assets/makosh-logo-mark.png` и текст «Макошь Memory System». Клик ведёт на представление `home`.
- **Навигация:** динамически строится по `sidebar.sidebarRootEntries`. Поддерживает два вида записей:
  - `kind === 'item'` — одиночный пункт. При клике вызывается `nav.navigateTo(itemViewId)`.
  - `kind === 'group'` — раскрывающаяся группа. Заголовок переключает `nav.toggleSidebarGroup(groupId)`. Подпункты могут быть обычными или коммуникационными секциями (`navigateToCommunicationSection`).
- **Режим rail** (свёрнутая панель): включается/выключается кнопкой в футере. В rail-режиме группы показывают выпадающие списки при клике. Активная группа управляется через `nav.activeSidebarRailGroupId`.
- **Футер:** кнопка сворачивания/разворачивания панели и кнопка перехода в Settings.

**Активные состояния:**
- `isItemActive(itemViewId)` — сверяет `nav.currentView` с идентификатором пункта (для коммуникаций — сравнивает с `'communications'`).
- `isCommunicationItemActive(sectionId)` — проверяет `nav.activeCommunicationSection` при активном `nav.currentView === 'communications'`.

**Стилизация:** панель имеет переменную ширину, фон с прозрачностью, размытие (`backdrop-filter: blur`), тень и скругление (`var(--hh-radius-sidebar)`).

### Topbar (`frontend/src/app/shell/Topbar.vue`)

Верхняя панель. Использует `useNavigationStore`, `useNotificationsStore`, `useRealtimeStatusStore` и `useI18n`.

**Структура:**
- **Слот `#makosh-topbar-slot`** — место для вставки контента из дочерних представлений. Если слот пуст, показывается фолбэк с заголовком (`nav.activeView.title`) и подзаголовком (`subtitle`).
- **Индикатор статуса реального времени:** иконка облака, текстовая метка (`realtimeStatusLabel`) и точка. Цветовые тона: `success` (зелёный), `warning` (жёлтый), `danger` (красный) управляются классами, которые меняют CSS-переменные типа `--hh-status-success`, `--hh-status-danger`.
- **Кнопка уведомлений:** переключает `notifications.toggleNotificationsDrawer()`. Показывает бейдж с количеством (максимум «9+»).
- **Меню пользователя:** раскрывается по клику на кнопку с иконкой `tabler:menu-2`. Содержит:
  - Переключение языка (между `'ru'` и `'en'`) через `setLocale`.
  - Выход из приложения (`window.close()`).

**Адаптивность:** на экранах ≤900px индикатор статуса теряет текстовую метку и точку, оставляя только иконку.

### NotificationsDrawer (`frontend/src/app/shell/NotificationsDrawer.vue`)

Выдвижная панель уведомлений (правая сторона). Управляется через `useNotificationsStore`, `useNavigationStore` и `useI18n`.

**Отображение:**
- Появляется при `notifications.isNotificationsDrawerOpen === true`.
- Затемнённый фон (backdrop) с анимацией прозрачности.
- Сама панель с анимацией выезда справа (transform translateX).

**Состояния:**
- **Пустое:** иконка колокольчика с галочкой и текст «All caught up!» (или перевод через `t('notifications.empty')`).
- **Список уведомлений:** каждое уведомление (`notificationItems`) содержит:
  - Иконку, заголовок, время и тело сообщения.
  - При клике на уведомление вызывается `handleOpenTarget`, который открывает целевое представление через `nav.navigateTo(targetView)`.
  - Если тело длиннее 120 символов, показывается кнопка «chevron» для разворачивания/сворачивания (управляется `notifications.toggleNotificationExpanded`).
  - Кнопка закрытия отдельного уведомления (`dismissNotification`).

**Заголовок панели:** иконка `tabler:bell`, название из `t('notifications.title')` (фолбэк «Notifications»), счётчик уведомлений.

### LayoutEditControls (`frontend/src/app/shell/LayoutEditControls.vue`)

Компонент управления редактированием раскладки. Отображается только когда `editor.isLayoutEditing === true` (хранилище `useLayoutEditorStore`).

**Кнопки:**
- **Add Widget** — открывает панель добавления виджетов (`editor.openAddWidgetDrawer`).
- **Cancel** — отменяет редактирование (`editor.cancelLayoutEditing`).
- **Reset** — сбрасывает раскладку текущего представления (`editor.resetCurrentViewLayout`).
- **Save** — сохраняет настройки раскладки (`editor.saveLayoutSettings`).

## Представления (Views)

Все представления находятся в `frontend/src/app/views/` и являются тонкими обёртками над доменными страницами. Каждое использует ту же структуру: импорт страницы из соответствующей директории `../../domains/<domain>/views/<Domain>Page.vue` и рендеринг без дополнительной логики.

| Представление | Доменная страница |
|---|---|
| `HomeView.vue` | `../../domains/home/views/HomePage.vue` |
| `CalendarView.vue` | `../../domains/calendar/views/CalendarPage.vue` |
| `CommunicationsView.vue` | `../../domains/communications/views/CommunicationsPage.vue` |
| `DocumentsView.vue` | `../../domains/documents/views/DocumentsPage.vue` |
| `EventTracingView.vue` | `../../platform/event-tracing/EventTraceWorkspace.vue` |
| `KnowledgeView.vue` | `../../domains/knowledge/views/KnowledgePage.vue` |
| `NotesView.vue` | `../../domains/notes/views/NotesPage.vue` |
| `OrganizationsView.vue` | `../../domains/organizations/views/OrganizationsPage.vue` |
| `PersonsView.vue` | `../../domains/personas/views/PersonsPage.vue` |
| `ProjectsView.vue` | `../../domains/projects/views/ProjectsPage.vue` |
| `ReviewView.vue` | `../../domains/review/views/ReviewPage.vue` |
| `SettingsView.vue` | `../../domains/settings/views/SettingsPage.vue` |
| `TasksView.vue` | `../../domains/tasks/views/TasksPage.vue` |
| `TimelineView.vue` | `../../domains/timeline/views/TimelinePage.vue` |

Представление `EventTracingView` отличается тем, что импортирует компонент из `../../platform/event-tracing/EventTraceWorkspace.vue` (не из `domains/.../views`).

## Доменные компоненты (пример: Агенты)

### AgentsGrid (`frontend/src/domains/agents/components/AgentsGrid.vue`)

Сетка карточек AI-агентов.

**Входные параметры:**
- `agentCards: AgentCard[]` — массив данных агентов.
- `selectedAgentIndex: number` — индекс выбранного агента.
- `isAiLoading: boolean` — флаг загрузки.

**События:**
- `selectAgent` — эмитит индекс выбранного агента.

**Состояния:**
- **Загрузка** (`isAiLoading && agentCards.length === 0`): текст «Loading local AI agents.»
- **Пусто** (`agentCards.length === 0`): текст «No V3 agents returned by the backend.»
- **Список:** грид из трёх колонок (две колонки при ≤1360px). Каждая карточка — кнопка с иконкой агента, именем, описанием, статусом, количеством запусков (`tasks runs`) и успешных выполнений (`success`).

### AgentsDetail (`frontend/src/domains/agents/components/AgentsDetail.vue`)

Детальная панель выбранного агента.

**Входные параметры:**
- `selectedAgent: AgentCard | null` — выбранный агент или `null`.

**Содержимое при наличии агента:**
- Иконка с тоновым классом (`tone`), имя, модель.
- Вкладки: Overview (активна), Run History, Citations, Settings (все disabled).
- Описание (`summary`) и дополнительный текст про V3-агента.
- Диаграмма-заглушка (класс `.spark-chart`).
- Список возможностей: «Ollama Runtime», «pgvector Retrieval», «Source Citations», «Run Provenance», «Review Queue» с иконками-галочками.

**При отсутствии агента:** иконка `tabler:robot-off` и заголовок «No agent selected».

## Стилизация

Компоненты используют систему CSS-переменных (custom properties) с префиксом `--hh-` (Макошь). Ниже приведены некоторые переменные, обнаруженные в исходниках:

**Переменные оболочки:**
- `--hh-shell-sidebar-width` — ширина развёрнутой боковой панели.
- `--hh-shell-sidebar-width-rail` — ширина свёрнутой панели.
- `--hh-shell-content-min-width` — минимальная ширина контентной области.
- `--hh-shell-right-inset`, `--hh-shell-bottom-inset` — отступы оболочки.
- `--hh-shell-workspace-gap` — промежуток между элементами рабочей области.
- `--hh-shell-topbar-offset` — смещение/отступ верхней панели.

**Переменные панелей и фона:**
- `--hh-panel-bg`, `--hh-panel-alpha`, `--hh-panel-blur` — фон, прозрачность и размытие панелей.
- `--hh-bg` — основной фон.
- `--hh-radius-md`, `--hh-radius-sidebar` — скругления.

**Цвета текста и состояний:**
- `--hh-text-primary`, `--hh-text-secondary`, `--hh-text-muted` — цвета текста.
- `--hh-accent` — акцентный цвет.
- `--hh-border`, `--hh-border-accent-soft` — цвета границ.
- `--hh-hover-bg`, `--hh-active-bg` — фоны при наведении/активации.
- `--hh-status-success`, `--hh-status-danger`, `--hh-status-warning` — статусные цвета.

**Дополнительно:**
- `--hh-widget-panel` — минимальная высота панели виджета.
- `--hh-shadow-sidebar`, `--hh-shadow-panel` — тени.
- `--hh-space-section`, `--hh-space-panel` — отступы.
- `--hh-color-text-bright` — яркий цвет текста (используется в `AgentsDetail`).

Иконки поставляются через `@iconify/vue` и используют идентификаторы в формате `tabler:...` (набор иконок Tabler).

## Статические ресурсы

- `frontend/src-tauri/icons/icon.png` — бинарный файл (иконка приложения для Tauri).
- `frontend/src-tauri/resources/tdlib/LICENSE_1_0.txt` — текст Boost Software License 1.0 для tdlib.

Семантика этих файлов в предоставленных исходниках не раскрыта; документируется только их наличие и метаданные.
```

### Source coverage / Покрытие источников

| Файл | Факты, покрытые в предложенной странице |
|---|---|
| `frontend/src/app/App.vue` | Импортирует и рендерит `AppShell` |
| `frontend/src/app/shell/AppShell.vue` | Структура оболочки (RouterView, Sidebar, Topbar, NotificationsDrawer); использование useNavigationStore и useThemeStore; вызов hydrateThemeSettings при монтировании; отслеживание route.name и route.query.section; CSS-переменные и Grid-раскладка |
| `frontend/src/app/shell/Sidebar.vue` | Структура боковой панели; бренд с логотипом; навигация по sidebarRootEntries (items и groups); режимы rail и обычный; методы navigateTo, toggleSidebarGroup, navigateToCommunicationSection; активные состояния; используемые хранилища |
| `frontend/src/app/shell/Topbar.vue` | Слот topbar-slot с фолбэком; индикатор realtimeStatus с тонами; кнопка уведомлений с бейджем; меню пользователя с переключением языка (ru/en) и выходом; используемые хранилища и i18n |
| `frontend/src/app/shell/NotificationsDrawer.vue` | Отображение/скрытие по notifications.isNotificationsDrawerOpen; backdrop и анимации; заголовок со счётчиком; пустое состояние; список уведомлений с разворачиванием тела >120 символов, открытием целевого представления и удалением |
| `frontend/src/app/shell/LayoutEditControls.vue` | Условный рендеринг по editor.isLayoutEditing; кнопки Add Widget, Cancel, Reset, Save; используемое хранилище useLayoutEditorStore |
| `frontend/src/app/views/HomeView.vue` | Импорт HomePage и рендеринг без логики |
| `frontend/src/app/views/CalendarView.vue` | Импорт CalendarPage |
| `frontend/src/app/views/CommunicationsView.vue` | Импорт CommunicationsPage |
| `frontend/src/app/views/DocumentsView.vue` | Импорт DocumentsPage |
| `frontend/src/app/views/EventTracingView.vue` | Импорт EventTraceWorkspace |
| `frontend/src/app/views/KnowledgeView.vue` | Импорт KnowledgePage |
| `frontend/src/app/views/NotesView.vue` | Импорт NotesPage |
| `frontend/src/app/views/OrganizationsView.vue` | Импорт OrganizationsPage |
| `frontend/src/app/views/PersonsView.vue` | Импорт PersonsPage |
| `frontend/src/app/views/ProjectsView.vue` | Импорт ProjectsPage |
| `frontend/src/app/views/ReviewView.vue` | Импорт ReviewPage |
| `frontend/src/app/views/SettingsView.vue` | Импорт SettingsPage |
| `frontend/src/app/views/TasksView.vue` | Импорт TasksPage |
| `frontend/src/app/views/TimelineView.vue` | Импорт TimelinePage |
| `frontend/src/domains/agents/components/AgentsDetail.vue` | Параметр selectedAgent; состояния «агент выбран» и «нет агента»; вкладки; возможности; spark-chart |
| `frontend/src/domains/agents/components/AgentsGrid.vue` | Параметры agentCards, selectedAgentIndex, isAiLoading; состояния загрузки, пустоты и списка; эмит selectAgent |
| `frontend/src-tauri/icons/icon.png` | Упомянут как бинарный ресурс |
| `frontend/src-tauri/resources/tdlib/LICENSE_1_0.txt` | Упомянут как лицензия Boost 1.0 для tdlib |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом и документацией не выявлено. Файлы содержат только исходный код, описания API, конфигурации или ADR отсутствуют, поэтому drift-анализ невозможен без дополнительных источников.

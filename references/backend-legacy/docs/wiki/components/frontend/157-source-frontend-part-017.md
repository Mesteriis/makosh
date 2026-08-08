---
chunk_id: 157-source-frontend-part-017
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 23
generated_by: code-wiki-ru
---

# 157-source-frontend-part-017 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `23`

## Резюме

Страница `components/frontend.md` должна быть создана или обновлена, чтобы отразить ключевые общие модули фронтенда (хранилища Pinia, утилиты почтовой синхронизации, санитизацию писем, UI-компоненты, конфигурацию сборки и пр.) на основе предоставленного среза исходников. Страница документирует только те факты, которые прямо подтверждаются встроенными файлами.

## Предложенные страницы

#### `components/frontend.md`

```markdown
# Frontend

Общие модули и конфигурация фронтенд-приложения Макошь.

## Интеграция с почтовыми аккаунтами (`mailSync`)

### Типы

- `MailSyncSettings` — полные настройки синхронизации: `account_id`, `sync_enabled`, `batch_size`, `poll_interval_seconds`, `updated_at`.
- `MailSyncSettingsUpdate` — набор полей, отправляемых при обновлении: `sync_enabled`, `batch_size`, `poll_interval_seconds`.
- `MailSyncStatus` — статус синхронизации для одного аккаунта: `account_id`, `status`, `phase`, `progress_mode`, `progress_percent`, `processed_messages`, `estimated_total_messages`, `current_batch_size`, временные метки последнего запуска/завершения, код и сообщение последней ошибки, счётчики обработанных сущностей.
- `MailSyncStatusListResponse` — обёртка списка статусов `{ items: MailSyncStatus[] }`.
- `MailSyncRunResponse` — ответ на ручной запуск синхронизации: `run_id`, `account_id`, `trigger`, `status`, `phase`, `progress_mode`, счётчики, информация о контрольных точках, `failure_reason` (код и сообщение), временные метки.

*Источник:* `frontend/src/shared/mailSync/types.ts`

### API-клиент

Файл `frontend/src/shared/mailSync/syncApi.ts` предоставляет функции для взаимодействия с серверным API:

- `fetchMailSyncStatus()` — GET `/api/v1/integrations/mail/accounts/sync-status`.
- `fetchMailSyncSettings(accountId)` — GET `/api/v1/integrations/mail/accounts/{accountId}/sync-settings`.
- `updateMailSyncSettings(accountId, settings)` — PUT `/api/v1/integrations/mail/accounts/{accountId}/sync-settings`.
- `runMailSyncNow(accountId)` — POST `/api/v1/integrations/mail/accounts/{accountId}/sync-now`.
- `runMailFullResync(accountId)` — POST `/api/v1/integrations/mail/accounts/{accountId}/sync-full-resync`.

Все вызовы используют `ApiClient.instance`. Идентификатор аккаунта кодируется через `encodeURIComponent`.

### Хуки Vue Query

Файл `frontend/src/shared/mailSync/runtimeQueries.ts` экспортирует composable-функции на базе `@tanstack/vue-query`:

- `useSyncStatusesQuery()` — запрос списка статусов синхронизации. Ключ: `['communications', 'mail', 'sync-statuses']`.
- `useMailSyncSettingsQuery(accountId)` — запрос настроек синхронизации конкретного аккаунта. Ключ зависит от `accountId`: при `null` — `['communications', 'mail', 'sync-settings', null]`, иначе — `['communications', 'mail', 'sync-settings', <id>]`. Запрос отключён (`enabled: false`), когда `accountId` не передан.
- `useUpdateMailSyncSettingsMutation()` — мутация обновления настроек. При успехе инвалидирует ключи `['communications', 'mail', 'sync-settings', <accountId>]` и `['communications', 'mail', 'sync-statuses']`.
- `useRunMailSyncNowMutation()` — мутация ручного запуска синхронизации. При успехе инвалидирует: `['communications-list']`, `['communications-state-counts']`, `['communications', 'mail', 'sync-statuses']`, `['communications', 'mail', 'mailbox-health']`.
- `useRunMailFullResyncMutation()` — мутация полной ресинхронизации. Инвалидирует те же ключи, что и `useRunMailSyncNowMutation`.

### Форма настроек синхронизации

Файл `frontend/src/shared/mailSync/syncSettingsForm.ts` содержит:

- Zod-схему `syncSettingsFormSchema`:
  - `sync_enabled`: `z.boolean()`
  - `batch_size`: целое от 1 до 500
  - `poll_interval_seconds`: целое от 60 до 86400
- `syncSettingsVeeValidationSchema` — обёртка для `@vee-validate/zod`.
- `syncSettingsFormDefaults(settings)` — значения по умолчанию: `sync_enabled: true`, `batch_size: 100`, `poll_interval_seconds: 300` (если settings нет, иначе из переданного объекта).
- `syncSettingsFormToUpdate(values)` — преобразует значения формы в `MailSyncSettingsUpdate`.

## Санитизация HTML писем

Файл `frontend/src/shared/sanitize/emailHtml.ts` предоставляет функции безопасного рендеринга тела письма.

### Основные функции

- `renderMessageBody(input)` — принимает `{ bodyHtml, bodyText }`. Если `bodyHtml` не пустой, возвращает `{ kind: 'html', html: sanitizeEmailHtml(bodyHtml) }`. Иначе возвращает `{ kind: 'plain', html: normalizePlainText(bodyText) }`.
- `sanitizeEmailHtml(html)` — очищает HTML: удаляет заблокированные контейнеры, разрешает только теги из белого списка, проверяет атрибуты по политикам, экранирует текст, закрывает незакрытые теги.
- `normalizePlainText(text)` — экранирует HTML-сущности и заменяет переводы строк на `<br>`.
- `remoteImageUrlsFromHtml(html)` — возвращает список URL удалённых изображений (`http://` или `https://`) из санитизированного HTML.
- `rewriteRemoteImageSources(html, remoteSource)` — заменяет `src` у `<img>` с удалёнными источниками: либо на результат вызова `remoteSource(url)`, либо на однопиксельный `data:image/gif` с атрибутами `data-makosh-remote-src` и `aria-label="Remote image blocked"`.

### Политики безопасности

- **Разрешённые теги** (`ALLOWED_TAGS`): `a`, `blockquote`, `br`, `code`, `div`, `em`, `img`, `li`, `ol`, `p`, `pre`, `s`, `span`, `strong`, `table`, `tbody`, `td`, `tfoot`, `th`, `thead`, `tr`, `u`, `ul`.
- **Переименования тегов** (`TAG_RENAMES`): `b → strong`, `i → em`, `font → span`.
- **Пустые теги** (`VOID_TAGS`): `br`, `img`.
- **Заблокированные контейнеры** (`BLOCKED_CONTAINER_TAGS`): `script`, `style`, `iframe`, `object`, `embed`, `svg`, `math`, `form`, `head`, `noscript`, `template`. Также удаляются теги `meta` и `link`.
- **Атрибуты**:
  - Глобально разрешён только `title`.
  - Для `a` разрешены `href` (с проверкой URL) и `title`; ссылки получают `target="_blank"` и `rel="noreferrer noopener"`.
  - Для `img` разрешены `alt`, `height`, `src`, `title`, `width`; `src` может быть `cid:` или `http(s)://`.
  - Для `td`, `th` разрешены `colspan`, `rowspan` (только целые числа до 4 цифр).
  - Атрибуты, начинающиеся с `on`, и `style` блокируются.
- **URL в атрибутах**: разрешены `http://`, `https://`, `mailto:`. Небезопасные символы удаляются.
- **Проверка удалённых изображений**: `isRemoteImageUrl` проверяет, начинается ли значение с `http://` или `https://`.

Тест `frontend/src/shared/sanitize/emailHtml.boundary.test.ts` подтверждает, что модуль `emailHtml.ts` содержит указанные экспорты и не использует `fetch` или `ApiClient`.

## Хранилища состояния (Pinia stores)

### Навигация (`navigation`)

Файл `frontend/src/shared/stores/navigation.ts` определяет хранилище `useNavigationStore` (Pinia).

**Типы**:
- `PrimaryNavId` — основные разделы: `home`, `communications`, `timeline`, `persons`, `projects`, `tasks`, `calendar`, `documents`, `notes`, `knowledge`, `review`, `event-tracing`, `agents`.
- `CommunicationSectionId` — секции коммуникаций: `unified`, `inbox`, `waiting`, `needs_reply`, `mentions`, `mail`, `telegram`, `whatsapp`, `calls`, `meetings`.
- `SidebarViewId`, `AppViewId`, `RouteViewId` — идентификаторы представлений.

**Состояние и действия**:
- `currentView` — текущее представление (по умолчанию `home`).
- `activeCommunicationSection` — активная секция коммуникаций (по умолчанию `unified`).
- `isSidebarRail`, `isUserMenuOpen` — флаги интерфейса.
- `expandedSidebarGroupIds` — раскрытые группы боковой панели (изначально `['communications']`).
- `activeSidebarRailGroupId` — активная группа в rail-режиме.
- `activeWorkspaceView` — вычисляемое: для `communications` определяется по `communicationSectionViewId`, иначе берётся `currentView`.
- `activeView` — вычисляемые метаданные текущего представления (заголовок, подзаголовок, иконка) из словаря `viewCopy`.
- `shellViewClass` — CSS-класс `view-{currentView}`.
- `navigateTo(viewId)` — устанавливает `currentView` и вызывает `router.push('/' + viewId)`.
- `navigateToCommunicationSection(sectionId)` — устанавливает `currentView = 'communications'`, секцию и вызывает `router.push({ name: 'communications', query: { section: sectionId } })`.
- `syncFromRoute(viewId, sectionQuery?)` — синхронизирует состояние из маршрута.
- `toggleUserMenu`, `closeUserMenu`, `toggleSidebarRail`, `toggleSidebarGroup`, `setActiveSidebarRailGroup`.

Тест `frontend/src/shared/stores/navigation.boundary.test.ts` проверяет, что навигация использует `router.push` с параметром `section` и вызывает `syncFromRoute` из оболочки.

### Боковая панель (`sidebar`)

Файл `frontend/src/shared/stores/sidebar.ts` (обрезан после 12000 символов) содержит хранилище `useSidebarStore`.

**Типы (видимая часть)**:
- `PrimaryNavId`, `CommunicationSectionId`, `CommunicationSidebarSectionId`, `SidebarPrimaryItemId`, `SidebarItemId`, `SidebarRootItemId`.
- `SidebarSettings` — настройки боковой панели, схема версии `3`: `rootItemIds`, `groups`, `hiddenItemIds`.
- `SidebarNavGroup` — группа навигации: `id`, `label`, `icon`, `itemIds`, `separatorBeforeItemIds`.
- `ResolvedSidebarItem`, `ResolvedSidebarRootEntry`.
- Статические списки `primaryWorkspaceNav` (13 элементов) и `communicationSections` (10 секций, сгруппированных в `overview`, `workflow`, `sources`).

**Логика (видимая часть)**:
- `defaultSidebarSettings()` возвращает настройки по умолчанию: группа `communications` включает секции источников (`mail`, `telegram`, `whatsapp`, `calls`, `meetings`) и `timeline`. Остальные разделы без группы.
- `resolveSidebarItem(itemId)` — возвращает метаданные элемента сайдбара (название, иконка, является ли коммуникацией).
- Хранилище управляет `sidebarSettings`, `sidebarDraft` (черновик при редактировании), предоставляет `effectiveSidebarSettings`, `sidebarRootEntries`, `sidebarHiddenNavItems`.
- Действия: `setSidebarSettings`, `updateSidebarDraft`, `resetSidebarSettingsToDefault`, `cancelSidebarSettingsEditing`, `sidebarConfigItem`, `addSidebarGroup`, `removeSidebarGroup`, `moveSidebarGroup`, `moveSidebarRootItem`, `moveSidebarItem`, `moveSidebarItemToGroup` (функция обрезана).

*Примечание:* файл обрезан, поэтому полный API может содержать дополнительные методы.

### Уведомления (`notifications`)

Файл `frontend/src/shared/stores/notifications.ts` определяет хранилище `useNotificationsStore`.

- Тип `NotificationItem`: `id`, `title`, `body?`, `icon`, `time`, `targetView?`, `targetId?`.
- Состояние: `isNotificationsDrawerOpen`, `dismissedNotificationIds` (Set), `expandedNotificationIds` (Set), `rawNotificationItems` (массив), `pendingNotificationTarget`.
- Вычисляемые: `notificationItems` — отфильтрованные (без dismissed) и отсортированные по времени (не более 12). `notificationCount` — количество элементов.
- Действия: `toggleNotificationsDrawer`, `closeNotificationsDrawer`, `dismissNotification`, `toggleNotificationExpanded`, `openNotificationTarget`, `consumePendingNotificationTarget`, `addNotification`.

В исходном состоянии `rawNotificationItems` пуст; заполнение предполагается через SSE-события (не реализовано в данном срезе).

### Статус realtime (`realtimeStatus`)

Файл `frontend/src/shared/stores/realtimeStatus.ts` предоставляет хранилище для мониторинга транспортного соединения реального времени.

**Типы**:
- `RealtimeTransportKind` — `'websocket' | 'sse' | 'long_poll'`.
- `RealtimeTransportState` — `'idle' | 'connecting' | 'connected' | 'reconnecting' | 'fallback' | 'disconnected'`.
- `RealtimeStatusTone` — `'neutral' | 'success' | 'warning' | 'danger'`.
- `RealtimeStatusUpdate` — вход для `setRealtimeStatus`: `transport`, `state`, опциональные `attempt`, `maxAttempts`, `error`.
- `RealtimeStatusSnapshot` — полный снимок статуса, включая `lastEventId`, `lastEventAt`, `lastLaggedSkipped`, `lastLaggedAt`, `updatedAt`.

**Состояние и вычисляемые**:
- `status` — реактивный `RealtimeStatusSnapshot`, начальное состояние: `transport: 'websocket'`, `state: 'idle'`.
- `isRealtimeDegraded` — истинно для `reconnecting`, `long_poll` или при наличии `lastLaggedSkipped`.
- `canTriggerReconnect` — истинно при `disconnected` или деградации.
- `realtimeStatusLabel` — текстовое описание (например, `'Realtime live'`, `'Realtime fallback'`, `'Realtime offline'`).
- `realtimeStatusTone` — тон для UI (success/warning/danger/neutral).
- `realtimeStatusDetail` — расширенный статус с ошибкой или счётчиком попыток.
- `realtimeRecoveryDetail` — диагностическая строка с курсором восстановления, пропущенными событиями, последней меткой времени.

**Методы**:
- `setRealtimeStatus(update)` — обновляет транспорт, состояние, флаги ошибок/попыток; при подключении через не-long_poll сбрасывает `lastLaggedSkipped`.
- `observeRealtimeEvent(eventId)` — сохраняет `lastEventId` и `lastEventAt`.
- `observeRealtimeLag(skipped)` — фиксирует пропущенные события.
- `resetRealtimeStatus()` — сброс к начальному состоянию.
- `setReconnectHandler(handler)` / `requestReconnect()` — регистрация и вызов внешнего обработчика переподключения.

Тесты в `frontend/src/shared/stores/realtimeStatus.test.ts` покрывают основные сценарии переключения состояний и диагностики.

### Тема (`theme`)

Файл `frontend/src/shared/stores/theme.ts` — хранилище `useThemeStore`.

**Типы** (реэкспортированы из `../../platform/theme/settings`):
- `AccentColorId`, `ShellBackgroundId`, `ThemeSettings`, `BackgroundBrightness`, `PanelBlur`, `PanelOpacity`, `SpacingDensity` (переименованы для совместимости).

**Состояние**:
- `themeSettings` — загружается из локального хранилища через `loadLocalThemeSettings()`.
- `themeDraft` — черновик во время редактирования.
- `isHydratingTheme`, `isSavingTheme` — флаги процессов загрузки/сохранения.
- `themePersistenceSource` — источник (`'local_storage'` или `'application_settings'`).
- `themePersistenceError` — текст ошибки сохранения/загрузки.

**Вычисляемые**:
- `effectiveThemeSettings` — черновик или сохранённые настройки.
- CSS-классы для оболочки: `shellBackgroundClass`, `shellBrightnessClass`, `shellAccentClass`, `shellPanelOpacityClass`, `shellPanelBlurClass`, `shellSpacingDensityClass` (формируются функциями из `../../platform/theme/settings`).
- `shellThemeClass` — агрегированная строка всех классов.
- `themePersistenceLabel` — метка режима сохранения.

**Действия**:
- `startThemeEditing()`, `updateThemeDraft(patch)`, `cancelThemeEditing()`, `resetThemeSettings()`.
- `hydrateThemeSettings()` — асинхронная загрузка сохранённой темы через `loadPersistedThemeSettings()`.
- `saveThemeSettings()` — асинхронное сохранение через `savePersistedThemeSettings()`, применяет черновик.
- `shellBackgroundLabel(id)`, `shellAccentLabel(id)` — человекочитаемые названия фонов и акцентов.

Экспортируются списки `backgroundOptions` (`shellBackgroundIds`) и `accentOptions` (`accentColorIds`).

### Редактор компоновки (`layoutEditor`)

Файл `frontend/src/shared/stores/layoutEditor.ts` (обрезан после 12000 символов). Содержит хранилище `useLayoutEditorStore`.

**Типы (видимая часть)**:
- `WidgetGridDimension` — `'columns' | 'rows'`
- `WidgetPanelSurfaceSetting` — `'panelOpacity' | 'panelBlur'`
- `ScrollMode` — `'default' | 'horizontal' | 'vertical' | 'none'`
- `ViewLayoutOverride` — переопределения для представления: `hiddenWidgetIds`, `zoneOverrides`, `orderOverrides`, `gridOverrides`, `panelSurfaceOverrides`.
- `WidgetDefinition` — описание виджета: `id`, `title`, `icon`, `viewScope`, размеры по умолчанию/минимальные, флаги `canAdd`/`removable`.
- `ResolvedWidget` — разрешённый виджет с текущими размерами, прозрачностью, блюром, зоной и порядком.
- `ResolvedLayout` — представление с списком виджетов и картой `widgetById`.
- `LayoutSettings` — настройки компоновки: `schemaVersion: 2`, `views` — запись переопределений.

**Предопределённые виджеты** (массив `defaultWidgets`, 24 элемента): домашняя страница (`home-welcome`, `home-stats`, `home-timeline`), персоны, проекты, задачи, календарь, документы, заметки, граф знаний, очередь ревью, коммуникации (универсальный inbox, mail, Telegram, WhatsApp), настройки (general, accounts, theme), AI-агенты, организации, лента, сообщения Telegram/WhatsApp.

**Логика (видимая часть)**:
- `getWidgetsForView(viewId, setting)` — возвращает `ResolvedWidget[]` для представления, исключая скрытые, применяя переопределения сетки и поверхности панели.
- Хранилище управляет `layoutSettings`, `layoutDraft`, флагами редактирования, текущим представлением `currentView`.
- Вычисляемые: `activeWidgets`, `activeWidgetById`, `visibleWidgetIds`, `addableWidgetsForCurrentView`.
- Действия: `setLayoutSettings`, `startLayoutEditing`, `cancelLayoutEditing`, `saveLayoutSettings`, `openAddWidgetDrawer`, `closeAddWidgetDrawer`, `openWidgetSettingsDrawer`, `closeWidgetSettingsDrawer`, `isWidgetVisible`, `hideWidget`, `showWidget`, `resetCurrentViewLayout`, `setWidgetGridValue`, `normalizeWidgetGridValue`, `adjustWidgetGridValue`, `handleWidgetGridInput` (файл обрезан на середине функции).

*Примечание:* полный API может содержать дополнительные методы, невидимые из-за обрезания.

## UI-компоненты

Файл `frontend/src/shared/ui/index.ts` экспортирует следующие компоненты (все — `.vue`-файлы, по умолчанию):

- `Icon`
- `Button`
- `Input`
- `Textarea`
- `Label`
- `Badge`
- `Separator`
- `Skeleton`
- `ScrollArea`
- `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, `CardFooter`
- `Tabs`, `TabTrigger`, `TabContent`
- `Switch`
- `Select`
- `Dialog`
- `Sheet`
- `Avatar`
- `Progress`
- `Toast`
- `Command`
- `DropdownMenu`, `DropdownMenuItem`, `DropdownMenuSeparator`, `DropdownMenuLabel`
- `Tooltip`
- `Popover`

Файл `frontend/src/shared/transitions/index.ts` реэкспортирует компоненты переходов:
- `FadeTransition`
- `SlideTransition`

### Компонент Dialog

Тест `frontend/src/shared/ui/Dialog.boundary.test.ts` документирует, что компонент `Dialog.vue`:
- использует `DialogRoot` с управляемым состоянием через пропс `open` и событие `@update:open` (controlled mode);
- отображает `DialogTrigger` только при наличии слота `trigger` (`v-if="$slots.trigger"`).

### Компонент Tabs

Тест `frontend/src/shared/ui/Tabs.boundary.test.ts` документирует, что компонент `Tabs.vue`:
- принимает пропсы `tabs` (тип `МакошьTab[]`) и `active` (строка);
- генерирует событие `select` с аргументом `[value: string]`;
- рендерит кнопки `TabsTrigger` через `v-for="tab in tabs"` и использует `@update:model-value` для обработки выбора.

## Интеграционные мосты

### YandexTelemost

Файл `frontend/src/shared/yandexTelemost/settingsBridge.ts` реэкспортирует тип `ProviderAccount` из `../../domains/settings/types/settings`.

### Zoom

Файл `frontend/src/shared/zoom/settingsBridge.ts` реэкспортирует:
- `settingsKeys` и `useApplicationSettingsQuery` из `../../domains/settings/queries/useSettingsQuery`;
- `useSettingsStore` из `../../domains/settings/stores/settings`;
- тип `ProviderAccount` из `../../domains/settings/types/settings`.

## Конфигурация сборки и типов

### Vite

Файл `frontend/vite.config.ts`:
- Плагин: `@vitejs/plugin-vue`.
- Алиас путей: `'@'` → `src`.
- Dev-сервер: порт `5173`.
- Сборка: выходная директория `dist`, `chunkSizeWarningLimit: 1536`.
- В rollup-опциях подавляются предупреждения о `INVALID_ANNOTATION` для `@vueuse/core`.

### Tailwind CSS

Файл `frontend/tailwind.config.ts` расширяет тему:
- **Шрифт**: `Inter`, `SF Pro Display`, системные fallback.
- **Цвета** с префиксом `hh-*`: фоны (`bg`, `bg-raised`, `surface`, `surface-deep`), текст (`text`, `text-strong`, `text-bright`, `text-soft`, `text-muted`, `text-subtle`, `text-dim`), акцентные (`accent`, `accent-strong`, `accent-soft`, `accent-contrast`), опасные (`danger`, `danger-strong`), бордеры (`border-accent-soft`, `border-accent`, `border-subtle`, `border-muted`), фокус (`focus-ring`), тонирования (`surface-tint`, `surface-panel`, `accent-tint`, `accent-control`, `danger-tint`), статусные поверхности и текст (`status-accent-surface`, `status-accent-text`, `status-warning-surface`, …, `status-neutral-text`).
- **Отступы** с префиксом `hh-*`: от `hh-1` (4px) до `hh-6` (24px).
- **Скругления**: `hh-xs`, `hh-sm`, `hh-control`, `hh-md`, `hh-lg`, `hh-xl`, `hh-pill`, `hh-round`.
- **Тени**: `hh-sidebar`, `hh-panel`, `hh-modal`.
- **Размеры**: `minWidth` для оболочки (`800px`), `minHeight` (`600px`), ширина сайдбара (`224px`/`208px`/`64px`).

### Декларация типов Vue

Файл `frontend/src/vite-env.d.ts` определяет модуль `*.vue`:
```ts
declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<object, object, unknown>
  export default component
}
```
```

## Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `frontend/src/shared/mailSync/types.ts` | Типы `MailSyncSettings`, `MailSyncSettingsUpdate`, `MailSyncStatus`, `MailSyncStatusListResponse`, `MailSyncRunResponse` и их поля. |
| `frontend/src/shared/mailSync/syncApi.ts` | Функции API-клиента, HTTP-методы, пути, кодирование параметров. |
| `frontend/src/shared/mailSync/runtimeQueries.ts` | Vue Query хуки: названия, ключи запросов, условия включения, логика инвалидации кэша. |
| `frontend/src/shared/mailSync/syncSettingsForm.ts` | Zod-схема с правилами валидации, значения по умолчанию, преобразование формы в `MailSyncSettingsUpdate`. |
| `frontend/src/shared/sanitize/emailHtml.ts` | Полный перечень: функции `renderMessageBody`, `sanitizeEmailHtml`, `normalizePlainText`, `remoteImageUrlsFromHtml`, `rewriteRemoteImageSources`; политики разрешённых тегов/атрибутов, блокируемые контейнеры, обработка удалённых изображений, экранирование. |
| `frontend/src/shared/sanitize/emailHtml.boundary.test.ts` | Подтверждение отсутствия прямого использования `fetch` и `ApiClient` в модуле санитизации. |
| `frontend/src/shared/stores/navigation.ts` | Типы навигации, состояние, действия, ключ `section` в query-параметрах, логика синхронизации из маршрута. |
| `frontend/src/shared/stores/navigation.boundary.test.ts` | Подтверждение использования `router.push` с query-параметром `section` и вызова `syncFromRoute`. |
| `frontend/src/shared/stores/notifications.ts` | Тип `NotificationItem`, состояние (drawer, dismissed, expanded, items, target), вычисляемые отфильтрованные/отсортированные уведомления, действия. |
| `frontend/src/shared/stores/realtimeStatus.ts` | Типы `RealtimeTransportKind`, `RealtimeTransportState`, `RealtimeStatusTone`, `RealtimeStatusUpdate`, `RealtimeStatusSnapshot`; вычисляемые статусы, метки, диагностическая строка восстановления; методы обновления статуса, наблюдения событий, переподключения. |
| `frontend/src/shared/stores/realtimeStatus.test.ts` | Покрытие тестами сценариев состояний. |
| `frontend/src/shared/stores/sidebar.ts` (обрезан) | Типы `SidebarSettings` (схема v3), `SidebarNavGroup`, списки `primaryWorkspaceNav` и `communicationSections`, начальная конфигурация, разрешение элементов, действия управления группами и элементами (в объёме обрезанной части). |
| `frontend/src/shared/stores/layoutEditor.ts` (обрезан) | Типы `WidgetDefinition`, `ResolvedWidget`, `LayoutSettings`, `ViewLayoutOverride`; статический массив `defaultWidgets` (24 элемента); функции `getWidgetsForView`, действия редактирования компоновки, сетки (в объёме видимой части). |
| `frontend/src/shared/stores/theme.ts` | Интеграция с `platform/theme`, состояние черновика, асинхронная загрузка/сохранение, вычисляемые CSS-классы, метки источников сохранения. |
| `frontend/src/shared/ui/index.ts` | Список экспортируемых UI-компонентов (34 позиции). |
| `frontend/src/shared/ui/Dialog.boundary.test.ts` | Подтверждение API Dialog: управляемый `open`/`update:open`, условный `DialogTrigger`. |
| `frontend/src/shared/ui/Tabs.boundary.test.ts` | Подтверждение API Tabs: пропсы `tabs`, `active`, событие `select`. |
| `frontend/src/shared/transitions/index.ts` | Экспорт `FadeTransition` и `SlideTransition`. |
| `frontend/src/shared/yandexTelemost/settingsBridge.ts` | Реэкспорт `ProviderAccount`. |
| `frontend/src/shared/zoom/settingsBridge.ts` | Реэкспорт `settingsKeys`, `useApplicationSettingsQuery`, `useSettingsStore`, `ProviderAccount`. |
| `frontend/vite.config.ts` | Конфигурация Vite: плагин, алиас, порт, параметры сборки, подавление предупреждений. |
| `frontend/tailwind.config.ts` | Расширение темы: шрифт, цвета `hh-*`, отступы, радиусы, тени, размеры. |
| `frontend/src/vite-env.d.ts` | Декларация модуля `*.vue`. |

## Исходные файлы

- [`frontend/src/shared/mailSync/runtimeQueries.ts`](../../../../frontend/src/shared/mailSync/runtimeQueries.ts)
- [`frontend/src/shared/mailSync/syncApi.ts`](../../../../frontend/src/shared/mailSync/syncApi.ts)
- [`frontend/src/shared/mailSync/syncSettingsForm.ts`](../../../../frontend/src/shared/mailSync/syncSettingsForm.ts)
- [`frontend/src/shared/mailSync/types.ts`](../../../../frontend/src/shared/mailSync/types.ts)
- [`frontend/src/shared/sanitize/emailHtml.boundary.test.ts`](../../../../frontend/src/shared/sanitize/emailHtml.boundary.test.ts)
- [`frontend/src/shared/sanitize/emailHtml.ts`](../../../../frontend/src/shared/sanitize/emailHtml.ts)
- [`frontend/src/shared/stores/layoutEditor.ts`](../../../../frontend/src/shared/stores/layoutEditor.ts)
- [`frontend/src/shared/stores/navigation.boundary.test.ts`](../../../../frontend/src/shared/stores/navigation.boundary.test.ts)
- [`frontend/src/shared/stores/navigation.ts`](../../../../frontend/src/shared/stores/navigation.ts)
- [`frontend/src/shared/stores/notifications.ts`](../../../../frontend/src/shared/stores/notifications.ts)
- [`frontend/src/shared/stores/realtimeStatus.test.ts`](../../../../frontend/src/shared/stores/realtimeStatus.test.ts)
- [`frontend/src/shared/stores/realtimeStatus.ts`](../../../../frontend/src/shared/stores/realtimeStatus.ts)
- [`frontend/src/shared/stores/sidebar.ts`](../../../../frontend/src/shared/stores/sidebar.ts)
- [`frontend/src/shared/stores/theme.ts`](../../../../frontend/src/shared/stores/theme.ts)
- [`frontend/src/shared/transitions/index.ts`](../../../../frontend/src/shared/transitions/index.ts)
- [`frontend/src/shared/ui/Dialog.boundary.test.ts`](../../../../frontend/src/shared/ui/Dialog.boundary.test.ts)
- [`frontend/src/shared/ui/Tabs.boundary.test.ts`](../../../../frontend/src/shared/ui/Tabs.boundary.test.ts)
- [`frontend/src/shared/ui/index.ts`](../../../../frontend/src/shared/ui/index.ts)
- [`frontend/src/shared/yandexTelemost/settingsBridge.ts`](../../../../frontend/src/shared/yandexTelemost/settingsBridge.ts)
- [`frontend/src/shared/zoom/settingsBridge.ts`](../../../../frontend/src/shared/zoom/settingsBridge.ts)
- [`frontend/src/vite-env.d.ts`](../../../../frontend/src/vite-env.d.ts)
- [`frontend/tailwind.config.ts`](../../../../frontend/tailwind.config.ts)
- [`frontend/vite.config.ts`](../../../../frontend/vite.config.ts)

## Кандидаты на drift

Из представленного контекста расхождений (drift) между кодом, документацией и ADR не выявлено. Часть файлов обрезана (`layoutEditor.ts`, `sidebar.ts`), что не позволяет судить о полноте их API относительно любых существующих описаний. В срезе отсутствуют другие wiki-страницы или ADR для сравнения — потенциальный drift не может быть подтверждён без дополнительного контекста.

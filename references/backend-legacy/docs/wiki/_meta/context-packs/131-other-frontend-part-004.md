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

- Chunk ID / ID чанка: `131-other-frontend-part-004`
- Group / Группа: `frontend`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/frontend.md`

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

### `frontend/src-tauri/icons/icon.png`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/icons/icon.png`
- Size bytes / Размер в байтах: `106906`
- Included characters / Включено символов: `0`
- Truncated / Обрезано: `not_applicable_binary`

_Binary file content omitted; use the path and metadata only. / Содержимое бинарного файла не включено; используй только путь и metadata._

### `frontend/src-tauri/resources/tdlib/LICENSE_1_0.txt`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/resources/tdlib/LICENSE_1_0.txt`
- Size bytes / Размер в байтах: `1338`
- Included characters / Включено символов: `1338`
- Truncated / Обрезано: `no`

```text
Boost Software License - Version 1.0 - August 17th, 2003

Permission is hereby granted, free of charge, to any person or organization
obtaining a copy of the software and accompanying documentation covered by this
license (the "Software") to use, reproduce, display, distribute, execute, and
transmit the Software, and to prepare derivative works of the Software, and to
permit third-parties to whom the Software is furnished to do so, all subject to
the following:

The copyright notices in the Software and this entire statement, including the
above license grant, this restriction and the following disclaimer, must be
included in all copies of the Software, in whole or in part, and all derivative
works of the Software, unless such copies or derivative works are solely in the
form of machine-executable object code generated by a source language processor.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE, TITLE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
COPYRIGHT HOLDERS OR ANYONE DISTRIBUTING THE SOFTWARE BE LIABLE FOR ANY DAMAGES
OR OTHER LIABILITY, WHETHER IN CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF
OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```

### `frontend/src/app/App.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/App.vue`
- Size bytes / Размер в байтах: `118`
- Included characters / Включено символов: `118`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import AppShell from './shell/AppShell.vue'
</script>

<template>
  <AppShell />
</template>
```

### `frontend/src/app/shell/AppShell.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/shell/AppShell.vue`
- Size bytes / Размер в байтах: `2261`
- Included characters / Включено символов: `2261`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { RouterView } from 'vue-router'
import { useRoute } from 'vue-router'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useThemeStore } from '../../shared/stores/theme'
import Sidebar from './Sidebar.vue'
import Topbar from './Topbar.vue'
import NotificationsDrawer from './NotificationsDrawer.vue'

const nav = useNavigationStore()
const theme = useThemeStore()
const route = useRoute()

onMounted(() => {
  void theme.hydrateThemeSettings()
})

watch(
  () => [route.name, route.query.section] as const,
  ([name, section]) => {
    if (typeof name === 'string') {
      nav.syncFromRoute(name as Parameters<typeof nav.syncFromRoute>[0], section)
    }
  },
  { immediate: true }
)
</script>

<template>
  <div
    class="viewport-guard"
    :class="[theme.shellThemeClass, nav.shellViewClass]"
  >
    <div
      class="desktop-shell"
      :class="{
        'sidebar-rail': nav.isSidebarRail
      }"
    >
      <!-- Sidebar -->
      <Sidebar />

      <!-- Workspace -->
      <div class="workspace">
        <Topbar />
        <NotificationsDrawer />
        <main class="workspace-content">
          <RouterView />
        </main>
      </div>
    </div>
  </div>
</template>

<style scoped>
.viewport-guard {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.desktop-shell {
  position: fixed;
  inset: 0;
  display: grid;
  grid-template-columns: var(--hh-shell-sidebar-width) minmax(var(--hh-shell-content-min-width), 1fr);
  gap: 16px;
  width: 100vw;
  max-width: 100vw;
  height: 100dvh;
  min-height: 0;
  overflow: hidden;
  padding: 0 var(--hh-shell-right-inset) var(--hh-shell-bottom-inset) 0;
  transition: grid-template-columns 280ms cubic-bezier(0.22, 1, 0.36, 1);
}

.desktop-shell.sidebar-rail {
  grid-template-columns: var(--hh-shell-sidebar-width-rail) minmax(var(--hh-shell-content-min-width), 1fr);
}

.workspace {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: var(--hh-shell-workspace-gap);
  height: 100%;
  min-width: 0;
  overflow: hidden;
  padding-bottom: var(--hh-shell-topbar-offset);
}

.workspace-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 0;
}
</style>
```

### `frontend/src/app/shell/LayoutEditControls.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/shell/LayoutEditControls.vue`
- Size bytes / Размер в байтах: `2058`
- Included characters / Включено символов: `2058`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useLayoutEditorStore } from '../../shared/stores/layoutEditor'

const editor = useLayoutEditorStore()

function handleAddWidget(): void {
  editor.openAddWidgetDrawer()
}

function handleCancel(): void {
  editor.cancelLayoutEditing()
}

function handleReset(): void {
  editor.resetCurrentViewLayout()
}

function handleSave(): void {
  editor.saveLayoutSettings()
}
</script>

<template>
  <div v-if="editor.isLayoutEditing" class="layout-edit-controls">
    <button class="layout-edit-btn" @click="handleAddWidget">
      <Icon icon="tabler:plus" class="layout-edit-btn-icon" />
      Add Widget
    </button>
    <button class="layout-edit-btn layout-edit-btn-secondary" @click="handleCancel">
      Cancel
    </button>
    <button class="layout-edit-btn layout-edit-btn-secondary" @click="handleReset">
      Reset
    </button>
    <button class="layout-edit-btn layout-edit-btn-primary" @click="handleSave">
      <Icon icon="tabler:device-floppy" class="layout-edit-btn-icon" />
      Save
    </button>
  </div>
</template>

<style scoped>
.layout-edit-controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1.25rem;
  background: var(--hh-panel-bg);
  border-bottom: 1px solid var(--hh-accent);
  flex-shrink: 0;
}

.layout-edit-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: 0.375rem;
  border: 1px solid var(--hh-border);
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
  cursor: pointer;
  transition: all 150ms ease;
}

.layout-edit-btn:hover {
  background: var(--hh-active-bg);
}

.layout-edit-btn-primary {
  background: var(--hh-accent);
  color: var(--hh-bg);
  border-color: var(--hh-accent);
}

.layout-edit-btn-primary:hover {
  filter: brightness(1.1);
}

.layout-edit-btn-secondary {
  border-color: transparent;
  background: transparent;
}

.layout-edit-btn-icon {
  width: 1rem;
  height: 1rem;
}
</style>
```

### `frontend/src/app/shell/NotificationsDrawer.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/shell/NotificationsDrawer.vue`
- Size bytes / Размер в байтах: `8159`
- Included characters / Включено символов: `8157`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useNotificationsStore } from '../../shared/stores/notifications'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useI18n } from '../../platform/i18n'

const notifications = useNotificationsStore()
const nav = useNavigationStore()
const { t } = useI18n()

function closeDrawer(): void {
  notifications.closeNotificationsDrawer()
}

function handleOpenTarget(notificationId: string): void {
  const notification = notifications.notificationItems.find(n => n.id === notificationId)
  if (notification) {
    notifications.openNotificationTarget(notification)
    if (notification.targetView) {
      nav.navigateTo(notification.targetView as any)
    }
  }
}

function isExpanded(notificationId: string): boolean {
  return notifications.expandedNotificationIds.has(notificationId)
}

function formatTime(date: Date): string {
  return date.toLocaleString()
}
</script>

<template>
  <div>
    <!-- Backdrop -->
    <Transition name="drawer-backdrop">
      <div
        v-if="notifications.isNotificationsDrawerOpen"
        class="drawer-backdrop"
        @click="closeDrawer"
      />
    </Transition>

    <!-- Drawer -->
    <Transition name="drawer-slide">
      <aside
        v-if="notifications.isNotificationsDrawerOpen"
        class="notifications-drawer"
      >
        <!-- Header -->
        <div class="drawer-header">
          <div class="drawer-header-left">
            <Icon icon="tabler:bell" class="drawer-header-icon" />
            <span class="drawer-title">{{ t('notifications.title') || 'Notifications' }}</span>
            <span class="drawer-count">{{ notifications.notificationCount }}</span>
          </div>
          <button class="drawer-close-btn" @click="closeDrawer">
            <Icon icon="tabler:x" class="drawer-close-icon" />
          </button>
        </div>

        <!-- Empty State -->
        <div v-if="notifications.notificationItems.length === 0" class="drawer-empty">
          <Icon icon="tabler:bell-check" class="drawer-empty-icon" />
          <p class="drawer-empty-text">{{ t('notifications.empty') || 'All caught up!' }}</p>
        </div>

        <!-- Notification List -->
        <div v-else class="drawer-list">
          <div
            v-for="notification in notifications.notificationItems"
            :key="notification.id"
            class="notification-item"
          >
            <button
              class="notification-main"
              @click="handleOpenTarget(notification.id)"
            >
              <Icon :icon="notification.icon" class="notification-icon" />
              <div class="notification-content">
                <div class="notification-header">
                  <span class="notification-title">{{ notification.title }}</span>
                  <span class="notification-time">{{ formatTime(notification.time) }}</span>
                </div>
                <div
                  v-if="notification.body"
                  class="notification-body"
                  :class="{ expanded: isExpanded(notification.id) }"
                >
                  {{ notification.body.length > 120 && !isExpanded(notification.id)
                    ? notification.body.slice(0, 120) + '…'
                    : notification.body
                  }}
                </div>
              </div>
            </button>
            <div class="notification-actions">
              <button
                v-if="notification.body && notification.body.length > 120"
                class="notification-action-btn"
                @click="notifications.toggleNotificationExpanded(notification.id)"
              >
                <Icon
                  :icon="isExpanded(notification.id) ? 'tabler:chevron-up' : 'tabler:chevron-down'"
                  class="notification-action-icon"
                />
              </button>
              <button
                class="notification-action-btn"
                @click="notifications.dismissNotification(notification.id)"
              >
                <Icon icon="tabler:x" class="notification-action-icon" />
              </button>
            </div>
          </div>
        </div>
      </aside>
    </Transition>
  </div>
</template>

<style scoped>
.drawer-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 40;
}

.notifications-drawer {
  position: fixed;
  top: 0;
  right: 0;
  width: 380px;
  height: 100vh;
  background: var(--hh-panel-bg);
  border-left: 1px solid var(--hh-border);
  z-index: 50;
  display: flex;
  flex-direction: column;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.2);
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.drawer-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.drawer-header-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent);
}

.drawer-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--hh-text-primary);
}

.drawer-count {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--hh-accent);
  background: var(--hh-active-bg);
  padding: 0.125rem 0.5rem;
  border-radius: 0.75rem;
}

.drawer-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  transition: all 150ms ease;
}

.drawer-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.drawer-close-icon {
  width: 1.25rem;
  height: 1.25rem;
}

.drawer-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 2rem;
}

.drawer-empty-icon {
  width: 3rem;
  height: 3rem;
  color: var(--hh-text-muted);
}

.drawer-empty-text {
  font-size: 0.875rem;
  color: var(--hh-text-muted);
  margin: 0;
}

.drawer-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.notification-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.75rem;
  border-radius: 0.375rem;
  transition: background 150ms ease;
}

.notification-item:hover {
  background: var(--hh-hover-bg);
}

.notification-main {
  flex: 1;
  display: flex;
  gap: 0.75rem;
  min-width: 0;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  padding: 0;
}

.notification-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--hh-accent);
  flex-shrink: 0;
  margin-top: 0.125rem;
}

.notification-content {
  min-width: 0;
}

.notification-header {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.notification-title {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary);
}

.notification-time {
  font-size: 0.6875rem;
  color: var(--hh-text-muted);
  white-space: nowrap;
}

.notification-body {
  font-size: 0.75rem;
  color: var(--hh-text-secondary);
  line-height: 1.4;
}

.notification-body.expanded {
  /* full text shown */
}

.notification-actions {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  flex-shrink: 0;
}

.notification-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 0.25rem;
  border: none;
  background: transparent;
  color: var(--hh-text-muted);
  cursor: pointer;
  transition: all 150ms ease;
}

.notification-action-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.notification-action-icon {
  width: 0.875rem;
  height: 0.875rem;
}

/* Transitions */
.drawer-backdrop-enter-active,
.drawer-backdrop-leave-active {
  transition: opacity 200ms ease;
}
.drawer-backdrop-enter-from,
.drawer-backdrop-leave-to {
  opacity: 0;
}

.drawer-slide-enter-active,
.drawer-slide-leave-active {
  transition: transform 280ms cubic-bezier(0.22, 1, 0.36, 1);
}
.drawer-slide-enter-from,
.drawer-slide-leave-to {
  transform: translateX(100%);
}
</style>
```

### `frontend/src/app/shell/Sidebar.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/shell/Sidebar.vue`
- Size bytes / Размер в байтах: `10630`
- Included characters / Включено символов: `10630`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useSidebarStore } from '../../shared/stores/sidebar'

const nav = useNavigationStore()
const sidebar = useSidebarStore()

function handleSelectItem(viewId: string): void {
  nav.navigateTo(viewId as any)
}

function handleToggleGroup(groupId: string): void {
  nav.toggleSidebarGroup(groupId)
}

function handleToggleRail(): void {
  nav.toggleSidebarRail()
}

function handleSettings(): void {
  nav.navigateTo('settings')
}

function isItemActive(itemViewId: string): boolean {
  if (nav.currentView === 'communications') {
    return itemViewId === 'communications'
  }
  return nav.currentView === itemViewId
}

function isCommunicationItemActive(sectionId?: string): boolean {
  if (!sectionId || nav.currentView !== 'communications') return false
  return nav.activeCommunicationSection === sectionId
}
</script>

<template>
  <aside
    class="sidebar"
    :class="{
      'sidebar-rail': nav.isSidebarRail
    }"
  >
    <!-- Brand -->
    <div class="sidebar-brand" @click="handleSelectItem('home')">
      <div class="sidebar-logo">
        <img class="sidebar-logo-image" src="/assets/makosh-logo-mark.png" alt="" aria-hidden="true" />
      </div>
      <div v-if="!nav.isSidebarRail" class="sidebar-brand-text">
        <span class="sidebar-brand-name">Макошь</span>
        <span class="sidebar-brand-subtitle">Memory System</span>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="sidebar-nav">
      <template v-for="entry in sidebar.sidebarRootEntries" :key="entry.rootId">
        <!-- Single Item -->
        <div v-if="entry.kind === 'item'" class="sidebar-nav-item-wrapper">
          <button
            class="sidebar-nav-item"
            :class="{ active: isItemActive(entry.item.itemId) }"
            @click="handleSelectItem(entry.item.itemId)"
          >
            <Icon :icon="entry.item.icon" class="sidebar-nav-icon" />
            <span v-if="!nav.isSidebarRail" class="sidebar-nav-label">
              {{ entry.item.label }}
            </span>
          </button>
        </div>

        <!-- Group -->
        <div v-else-if="entry.kind === 'group'" class="sidebar-nav-group-wrapper">
          <!-- Group Header -->
          <div v-if="!nav.isSidebarRail" class="sidebar-nav-group-header">
            <button
              class="sidebar-nav-group-toggle"
              :class="{ active: isItemActive(entry.group.id) }"
              @click="handleToggleGroup(entry.group.id)"
            >
              <Icon
                :icon="entry.group.icon"
                class="sidebar-nav-icon"
              />
              <span class="sidebar-nav-label">{{ entry.group.label }}</span>
              <Icon
                icon="tabler:chevron-down"
                class="sidebar-nav-chevron"
                :class="{ expanded: nav.expandedSidebarGroupIds.includes(entry.group.id) }"
              />
            </button>
          </div>

          <!-- Group Items (normal mode) -->
          <div
            v-if="!nav.isSidebarRail"
            class="sidebar-nav-subnav"
            :class="{ open: nav.expandedSidebarGroupIds.includes(entry.group.id) }"
          >
            <button
              v-for="item in entry.group.items"
              :key="item.itemId"
              class="sidebar-nav-item sidebar-nav-subitem"
              :class="{
                active: item.isCommunication
                  ? isCommunicationItemActive(item.sectionId)
                  : isItemActive(item.itemId)
              }"
              @click="item.isCommunication
                ? nav.navigateToCommunicationSection(item.sectionId!)
                : handleSelectItem(item.itemId)"
            >
              <Icon :icon="item.icon" class="sidebar-nav-icon" />
              <span class="sidebar-nav-label">{{ item.label }}</span>
            </button>
          </div>

          <!-- Rail mode: group shows as dropdown trigger -->
          <div v-else class="sidebar-rail-group">
            <button
              class="sidebar-nav-item"
              :class="{
                'rail-active': nav.activeSidebarRailGroupId === entry.group.id || isItemActive(entry.group.id)
              }"
              @click="nav.setActiveSidebarRailGroup(
                nav.activeSidebarRailGroupId === entry.group.id ? null : entry.group.id
              )"
            >
              <Icon :icon="entry.group.icon" class="sidebar-nav-icon" />
            </button>
            <div
              v-if="nav.activeSidebarRailGroupId === entry.group.id"
              class="sidebar-rail-dropdown"
            >
              <button
                v-for="item in entry.group.items"
                :key="item.itemId"
                class="sidebar-rail-dropdown-item"
                :class="{
                  active: item.isCommunication
                    ? isCommunicationItemActive(item.sectionId)
                    : isItemActive(item.itemId)
                }"
                @click="item.isCommunication
                  ? nav.navigateToCommunicationSection(item.sectionId!)
                  : handleSelectItem(item.itemId)"
              >
                <Icon :icon="item.icon" class="sidebar-nav-icon" />
                <span>{{ item.label }}</span>
              </button>
            </div>
          </div>
        </div>
      </template>
    </nav>

    <!-- Bottom section -->
    <div class="sidebar-footer">
      <button class="sidebar-nav-item" @click="handleToggleRail">
        <Icon
          :icon="nav.isSidebarRail ? 'tabler:layout-sidebar-right' : 'tabler:layout-sidebar'"
          class="sidebar-nav-icon"
        />
        <span v-if="!nav.isSidebarRail" class="sidebar-nav-label">
          {{ nav.isSidebarRail ? 'Expand' : 'Collapse' }}
        </span>
      </button>
      <button
        class="sidebar-nav-item"
        :class="{ active: isItemActive('settings') }"
        @click="handleSettings"
      >
        <Icon icon="tabler:settings" class="sidebar-nav-icon" />
        <span v-if="!nav.isSidebarRail" class="sidebar-nav-label">Settings</span>
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: rgba(5, 22, 25, var(--hh-panel-alpha));
  border-right: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sidebar);
  box-shadow: var(--hh-shadow-sidebar);
  backdrop-filter: blur(var(--hh-panel-blur));
  width: var(--hh-shell-sidebar-width);
  transition: width 280ms cubic-bezier(0.22, 1, 0.36, 1);
  overflow: hidden;
  z-index: 10;
}

.sidebar.sidebar-rail {
  width: var(--hh-shell-sidebar-width-rail);
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: var(--hh-space-section) var(--hh-space-panel);
  cursor: pointer;
  border-bottom: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.sidebar-logo {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(45, 240, 206, 0.08);
  border: 1px solid var(--hh-border-accent-soft);
  border-radius: var(--hh-radius-md);
  flex-shrink: 0;
}

.sidebar-logo-image {
  width: 1.45rem;
  height: 1.45rem;
  object-fit: contain;
}

.sidebar-brand-text {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  white-space: nowrap;
}

.sidebar-brand-name {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--hh-text-primary);
}

.sidebar-brand-subtitle {
  font-size: 0.75rem;
  color: var(--hh-text-muted);
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.sidebar-nav-item-wrapper {
  /* container for single items */
}

.sidebar-nav-group-wrapper {
  /* container for groups */
}

.sidebar-nav-group-header {
  /* group header row */
}

.sidebar-nav-group-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-primary);
  cursor: pointer;
  font-size: 0.8125rem;
  font-weight: 500;
  transition: background 150ms ease;
}

.sidebar-nav-group-toggle:hover {
  background: var(--hh-hover-bg);
}

.sidebar-nav-group-toggle.active {
  background: var(--hh-active-bg);
  color: var(--hh-accent);
}

.sidebar-nav-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  font-size: 0.8125rem;
  transition: all 150ms ease;
  text-align: left;
}

.sidebar-nav-item:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.sidebar-nav-item.active {
  background: var(--hh-active-bg);
  color: var(--hh-accent);
}

.sidebar-nav-icon {
  width: 1.25rem;
  height: 1.25rem;
  flex-shrink: 0;
}

.sidebar-nav-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-nav-chevron {
  margin-left: auto;
  width: 1rem;
  height: 1rem;
  transition: transform 200ms ease;
}

.sidebar-nav-chevron.expanded {
  transform: rotate(180deg);
}

.sidebar-nav-subnav {
  max-height: 0;
  overflow: hidden;
  transition: max-height 250ms ease;
}

.sidebar-nav-subnav.open {
  max-height: 500px;
}

.sidebar-nav-subitem {
  padding-left: 2.75rem;
  font-size: 0.75rem;
}

.sidebar-rail-group {
  position: relative;
}

.sidebar-rail-dropdown {
  position: absolute;
  left: 100%;
  top: 0;
  width: 200px;
  background: var(--hh-panel-bg);
  border: 1px solid var(--hh-border);
  border-radius: 0.5rem;
  padding: 0.25rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  margin-left: 0.25rem;
}

.sidebar-rail-dropdown-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  font-size: 0.8125rem;
  text-align: left;
  transition: all 150ms ease;
}

.sidebar-rail-dropdown-item:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.sidebar-rail-dropdown-item.active {
  color: var(--hh-accent);
}

.sidebar-nav-item.rail-active {
  background: var(--hh-active-bg);
  color: var(--hh-accent);
}

.sidebar-footer {
  border-top: 1px solid var(--hh-border);
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  flex-shrink: 0;
}
</style>
```

### `frontend/src/app/shell/Topbar.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/shell/Topbar.vue`
- Size bytes / Размер в байтах: `8488`
- Included characters / Включено символов: `8481`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@iconify/vue'
import { useNavigationStore } from '../../shared/stores/navigation'
import { useNotificationsStore } from '../../shared/stores/notifications'
import { useRealtimeStatusStore } from '../../shared/stores/realtimeStatus'
import { useI18n } from '../../platform/i18n'

const nav = useNavigationStore()
const notifications = useNotificationsStore()
const realtimeStatus = useRealtimeStatusStore()
const { t, setLocale, locale } = useI18n()

const realtimeStatusIcon = computed<string>(() => {
  if (realtimeStatus.realtimeStatusTone === 'success') return 'tabler:cloud-check'
  if (realtimeStatus.realtimeStatusTone === 'danger') return 'tabler:cloud-off'
  if (realtimeStatus.isRealtimeDegraded) return 'tabler:cloud-exclamation'
  return 'tabler:cloud-up'
})

function toggleNotifications(): void {
  notifications.toggleNotificationsDrawer()
}

function toggleMenu(): void {
  nav.toggleUserMenu()
}

function toggleLocale(): void {
  setLocale(locale.value === 'ru' ? 'en' : 'ru')
}

function exitApplication(): void {
  // In Tauri this would call window.close()
  // In browser, just a placeholder
  window.close()
}
</script>

<template>
  <header class="topbar">
    <div class="topbar-slot-shell">
      <div id="makosh-topbar-slot" class="topbar-slot" />
      <div class="topbar-slot-fallback">
        <h1 class="topbar-title">{{ nav.activeView?.title ?? 'Макошь' }}</h1>
        <p class="topbar-subtitle">{{ nav.activeView?.subtitle ?? '' }}</p>
      </div>
    </div>

    <div class="topbar-actions">
      <div
        class="topbar-realtime-status"
        :class="realtimeStatus.realtimeStatusTone"
        :title="realtimeStatus.realtimeStatusDetail"
        :aria-label="realtimeStatus.realtimeStatusDetail"
      >
        <span class="topbar-realtime-dot" />
        <Icon :icon="realtimeStatusIcon" class="topbar-realtime-icon" aria-hidden="true" />
        <span class="topbar-realtime-label">{{ realtimeStatus.realtimeStatusLabel }}</span>
      </div>

      <button
        class="topbar-action-btn"
        @click="toggleNotifications"
        title="Notifications"
        aria-label="Notifications"
      >
        <Icon icon="tabler:bell" class="topbar-action-icon" />
        <span
          v-if="notifications.notificationCount > 0"
          class="topbar-badge"
        >
          {{ notifications.notificationCount > 9 ? '9+' : notifications.notificationCount }}
        </span>
      </button>

      <div class="topbar-menu-wrapper">
        <button
          class="topbar-action-btn topbar-menu-btn"
          :class="{ active: nav.isUserMenuOpen }"
          @click="toggleMenu"
          title="Menu"
          aria-label="Menu"
        >
          <Icon icon="tabler:menu-2" class="topbar-action-icon" />
        </button>

        <div v-if="nav.isUserMenuOpen" class="topbar-dropdown" @mouseleave="nav.closeUserMenu()">
          <button class="topbar-dropdown-item" @click="toggleLocale">
            <Icon icon="tabler:language" class="topbar-dropdown-icon" />
            <span>{{ locale === 'ru' ? 'English' : 'Русский' }}</span>
          </button>

          <div class="topbar-dropdown-separator" />

          <button class="topbar-dropdown-item topbar-dropdown-exit" @click="exitApplication">
            <Icon icon="tabler:logout" class="topbar-dropdown-icon" />
            <span>{{ t('actions.exit') || 'Exit' }}</span>
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.75rem;
  height: 3.5rem;
  margin-top: var(--hh-shell-topbar-offset);
  padding: 0 0.875rem;
  background: rgba(5, 22, 25, var(--hh-panel-alpha));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: var(--hh-shadow-panel);
  backdrop-filter: blur(var(--hh-panel-blur));
  flex-shrink: 0;
}

.topbar-slot-shell {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  height: 100%;
}

.topbar-slot {
  display: flex;
  align-items: stretch;
  flex: 1;
  min-width: 0;
  height: 100%;
}

.topbar-slot:has(> *) + .topbar-slot-fallback {
  display: none;
}

.topbar-slot-fallback {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  justify-content: center;
  min-width: 0;
}

.topbar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin: 0;
  line-height: 1.2;
}

.topbar-subtitle {
  font-size: 0.75rem;
  color: var(--hh-text-muted);
  margin: 0;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.topbar-realtime-status {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  max-width: 11rem;
  height: 2.25rem;
  padding: 0 0.625rem;
  border: 1px solid var(--hh-border);
  border-radius: 0.375rem;
  background: rgba(255, 255, 255, 0.04);
  color: var(--hh-text-secondary);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.topbar-realtime-dot {
  width: 0.45rem;
  height: 0.45rem;
  border-radius: 999px;
  background: var(--hh-text-muted);
  box-shadow: 0 0 0 0.1875rem rgba(255, 255, 255, 0.05);
  flex-shrink: 0;
}

.topbar-realtime-icon {
  width: 1rem;
  height: 1rem;
  flex-shrink: 0;
}

.topbar-realtime-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.75rem;
  font-weight: 600;
}

.topbar-realtime-status.success {
  border-color: color-mix(in srgb, var(--hh-status-success, #22c55e) 36%, transparent);
  color: var(--hh-status-success-text, #16a34a);
}

.topbar-realtime-status.success .topbar-realtime-dot {
  background: var(--hh-status-success, #22c55e);
}

.topbar-realtime-status.warning {
  border-color: color-mix(in srgb, var(--hh-status-warning, #f59e0b) 36%, transparent);
  color: var(--hh-status-warning-text, #d97706);
}

.topbar-realtime-status.warning .topbar-realtime-dot {
  background: var(--hh-status-warning, #f59e0b);
}

.topbar-realtime-status.danger {
  border-color: color-mix(in srgb, var(--hh-status-danger, #ef4444) 36%, transparent);
  color: var(--hh-status-danger-text, #ef4444);
}

.topbar-realtime-status.danger .topbar-realtime-dot {
  background: var(--hh-status-danger, #ef4444);
}

.topbar-action-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 0.375rem;
  border: none;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  transition: all 150ms ease;
}

.topbar-action-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.topbar-action-btn.active {
  background: var(--hh-active-bg);
  color: var(--hh-accent);
}

.topbar-action-icon {
  width: 1.25rem;
  height: 1.25rem;
}

.topbar-badge {
  position: absolute;
  top: 0.25rem;
  right: 0.25rem;
  min-width: 1rem;
  height: 1rem;
  padding: 0 0.25rem;
  font-size: 0.625rem;
  font-weight: 700;
  line-height: 1rem;
  text-align: center;
  border-radius: 0.5rem;
  background: var(--hh-accent);
  color: var(--hh-bg);
}

.topbar-menu-wrapper {
  position: relative;
  flex-shrink: 0;
}

.topbar-dropdown {
  position: absolute;
  right: 0;
  top: 100%;
  margin-top: 0.25rem;
  width: 200px;
  background: var(--hh-panel-bg);
  border: 1px solid var(--hh-border);
  border-radius: 0.5rem;
  padding: 0.25rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
}

.topbar-dropdown-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border: none;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--hh-text-secondary);
  cursor: pointer;
  font-size: 0.8125rem;
  text-align: left;
  transition: all 150ms ease;
}

.topbar-dropdown-item:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.topbar-dropdown-item.active {
  color: var(--hh-accent);
}

.topbar-dropdown-item.topbar-dropdown-exit:hover {
  color: var(--hh-danger, #ef4444);
}

.topbar-dropdown-icon {
  width: 1.125rem;
  height: 1.125rem;
  flex-shrink: 0;
}

.topbar-dropdown-separator {
  height: 1px;
  background: var(--hh-border);
  margin: 0.25rem 0.5rem;
}

@media (max-width: 900px) {
  .topbar-realtime-status {
    width: 2.25rem;
    justify-content: center;
    padding: 0;
  }

  .topbar-realtime-label,
  .topbar-realtime-dot {
    display: none;
  }
}
</style>
```

### `frontend/src/app/views/AgentsView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/AgentsView.vue`
- Size bytes / Размер в байтах: `143`
- Included characters / Включено символов: `143`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import AgentsPage from '../../domains/agents/views/AgentsPage.vue'
</script>

<template>
  <AgentsPage />
</template>
```

### `frontend/src/app/views/CalendarView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/CalendarView.vue`
- Size bytes / Размер в байтах: `151`
- Included characters / Включено символов: `151`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import CalendarPage from '../../domains/calendar/views/CalendarPage.vue'
</script>

<template>
  <CalendarPage />
</template>
```

### `frontend/src/app/views/CommunicationsView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/CommunicationsView.vue`
- Size bytes / Размер в байтах: `175`
- Included characters / Включено символов: `175`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import CommunicationsPage from '../../domains/communications/views/CommunicationsPage.vue'
</script>

<template>
  <CommunicationsPage />
</template>
```

### `frontend/src/app/views/DocumentsView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/DocumentsView.vue`
- Size bytes / Размер в байтах: `155`
- Included characters / Включено символов: `155`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import DocumentsPage from '../../domains/documents/views/DocumentsPage.vue'
</script>

<template>
  <DocumentsPage />
</template>
```

### `frontend/src/app/views/EventTracingView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/EventTracingView.vue`
- Size bytes / Размер в байтах: `172`
- Included characters / Включено символов: `172`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import EventTraceWorkspace from '../../platform/event-tracing/EventTraceWorkspace.vue'
</script>

<template>
  <EventTraceWorkspace />
</template>
```

### `frontend/src/app/views/HomeView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/HomeView.vue`
- Size bytes / Размер в байтах: `135`
- Included characters / Включено символов: `135`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import HomePage from '../../domains/home/views/HomePage.vue'
</script>

<template>
  <HomePage />
</template>
```

### `frontend/src/app/views/KnowledgeView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/KnowledgeView.vue`
- Size bytes / Размер в байтах: `155`
- Included characters / Включено символов: `155`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import KnowledgePage from '../../domains/knowledge/views/KnowledgePage.vue'
</script>

<template>
  <KnowledgePage />
</template>
```

### `frontend/src/app/views/NotesView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/NotesView.vue`
- Size bytes / Размер в байтах: `139`
- Included characters / Включено символов: `139`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import NotesPage from '../../domains/notes/views/NotesPage.vue'
</script>

<template>
  <NotesPage />
</template>
```

### `frontend/src/app/views/OrganizationsView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/OrganizationsView.vue`
- Size bytes / Размер в байтах: `171`
- Included characters / Включено символов: `171`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import OrganizationsPage from '../../domains/organizations/views/OrganizationsPage.vue'
</script>

<template>
  <OrganizationsPage />
</template>
```

### `frontend/src/app/views/PersonsView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/PersonsView.vue`
- Size bytes / Размер в байтах: `148`
- Included characters / Включено символов: `148`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import PersonsPage from '../../domains/personas/views/PersonsPage.vue'
</script>

<template>
  <PersonsPage />
</template>
```

### `frontend/src/app/views/ProjectsView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/ProjectsView.vue`
- Size bytes / Размер в байтах: `151`
- Included characters / Включено символов: `151`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import ProjectsPage from '../../domains/projects/views/ProjectsPage.vue'
</script>

<template>
  <ProjectsPage />
</template>
```

### `frontend/src/app/views/ReviewView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/ReviewView.vue`
- Size bytes / Размер в байтах: `143`
- Included characters / Включено символов: `143`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import ReviewPage from '../../domains/review/views/ReviewPage.vue'
</script>

<template>
  <ReviewPage />
</template>
```

### `frontend/src/app/views/SettingsView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/SettingsView.vue`
- Size bytes / Размер в байтах: `151`
- Included characters / Включено символов: `151`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import SettingsPage from '../../domains/settings/views/SettingsPage.vue'
</script>

<template>
  <SettingsPage />
</template>
```

### `frontend/src/app/views/TasksView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/TasksView.vue`
- Size bytes / Размер в байтах: `139`
- Included characters / Включено символов: `139`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import TasksPage from '../../domains/tasks/views/TasksPage.vue'
</script>

<template>
  <TasksPage />
</template>
```

### `frontend/src/app/views/TimelineView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/app/views/TimelineView.vue`
- Size bytes / Размер в байтах: `151`
- Included characters / Включено символов: `151`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import TimelinePage from '../../domains/timeline/views/TimelinePage.vue'
</script>

<template>
  <TimelinePage />
</template>
```

### `frontend/src/domains/agents/components/AgentsDetail.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/components/AgentsDetail.vue`
- Size bytes / Размер в байтах: `2544`
- Included characters / Включено символов: `2544`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AgentCard } from '../types/agents'

interface Props {
	selectedAgent: AgentCard | null
}

const props = defineProps<Props>()
</script>

<template>
	<section class="panel agent-detail">
		<template v-if="selectedAgent">
			<header>
				<span class="round-icon" :class="selectedAgent.tone">
					<Icon :icon="selectedAgent.icon" width="26" height="26" />
				</span>
				<div>
					<h2>{{ selectedAgent.name }}</h2>
					<em>{{ selectedAgent.model }}</em>
				</div>
			</header>
			<div class="section-tabs">
				<button type="button" class="active">Overview</button>
				<button type="button" disabled>Run History</button>
				<button type="button" disabled>Citations</button>
				<button type="button" disabled>Settings</button>
			</div>
			<div class="agent-detail-grid">
				<p>{{ selectedAgent.summary }}. This V3 agent reads local memory projections, retrieves citations and records every run in the backend.</p>
				<div class="spark-chart"></div>
				<ul>
					<li v-for="capability in ['Ollama Runtime','pgvector Retrieval','Source Citations','Run Provenance','Review Queue']" :key="capability">
						<Icon icon="tabler:circle-check" width="16" height="16" />{{ capability }}
					</li>
				</ul>
			</div>
		</template>
		<template v-else>
			<header>
				<span class="round-icon cyan">
					<Icon icon="tabler:robot-off" width="26" height="26" />
				</span>
				<div>
					<h2>No agent selected</h2>
					<em>Backend status required</em>
				</div>
			</header>
		</template>
	</section>
</template>

<style scoped>
.agent-detail {
	margin-top: 12px;
	padding: 14px;
}

.agent-detail header {
	display: flex;
	align-items: center;
	gap: 12px;
}

.agent-detail h2 {
	color: var(--hh-color-text-bright);
	font-size: 20px;
}

.agent-detail-grid {
	display: grid;
	grid-template-columns: 1fr 300px 240px;
	gap: 22px;
	padding: 14px 8px 0;
}

.agent-detail-grid p,
.agent-detail-grid li {
	color: #c7d9d8;
	font-size: 13px;
	line-height: 1.5;
}

.agent-detail-grid ul {
	display: grid;
	gap: 12px;
	margin: 0;
	padding: 0;
	list-style: none;
}

.agent-detail-grid li {
	display: flex;
	align-items: center;
	gap: 8px;
}

.spark-chart {
	height: 150px;
	border: 1px solid rgba(111, 205, 195, 0.1);
	border-radius: var(--hh-radius-md);
	background:
		linear-gradient(160deg, transparent 42%, rgba(45, 240, 206, 0.9) 43%, transparent 44%),
		linear-gradient(rgba(45, 240, 206, 0.035) 1px, transparent 1px);
	background-size: auto, 28px 28px;
}
</style>
```

### `frontend/src/domains/agents/components/AgentsGrid.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/agents/components/AgentsGrid.vue`
- Size bytes / Размер в байтах: `1974`
- Included characters / Включено символов: `1974`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { AgentCard } from '../types/agents'

interface Props {
	agentCards: AgentCard[]
	selectedAgentIndex: number
	isAiLoading: boolean
}

interface Emits {
	(e: 'selectAgent', index: number): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
	<div class="agent-grid">
		<template v-if="isAiLoading && agentCards.length === 0">
			<div class="graph-strip-message"><span>Loading local AI agents.</span></div>
		</template>
		<template v-else-if="agentCards.length === 0">
			<div class="graph-strip-message"><span>No V3 agents returned by the backend.</span></div>
		</template>
		<template v-else>
			<button
				v-for="(agent, index) in agentCards"
				:key="agent.agentId"
				type="button"
				class="agent-card panel"
				:class="{ active: selectedAgentIndex === index }"
				@click="emit('selectAgent', index)"
			>
				<span class="round-icon" :class="agent.tone">
					<Icon :icon="agent.icon" width="22" height="22" />
				</span>
				<div>
					<strong>{{ agent.name }}</strong>
					<p>{{ agent.summary }}</p>
					<em>{{ agent.status }}</em>
				</div>
				<footer>
					<span>{{ agent.tasks }} runs</span>
					<span>{{ agent.success }} success</span>
				</footer>
			</button>
		</template>
	</div>
</template>

<style scoped>
.agent-grid {
	display: grid;
	grid-template-columns: repeat(3, minmax(0, 1fr));
	gap: 10px;
}

.agent-card {
	display: grid;
	grid-template-columns: 44px 1fr;
	gap: 12px;
	min-height: var(--hh-widget-panel);
	padding: 12px;
	text-align: left;
}

.agent-card.active {
	border-color: rgba(45, 240, 206, 0.38);
}

.agent-card footer {
	grid-column: 1 / -1;
	display: flex;
	justify-content: space-between;
	border-top: 1px solid rgba(102, 189, 180, 0.08);
	padding-top: 10px;
	font-size: 11px;
}

@media (max-width: 1360px) {
	.agent-grid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
}
</style>
```

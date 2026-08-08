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

- Chunk ID / ID чанка: `133-other-frontend-part-006`
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

### `frontend/src/domains/communications/components/CommunicationsContextRail.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsContextRail.vue`
- Size bytes / Размер в байтах: `4181`
- Included characters / Включено символов: `4181`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageDetailResponse, ProjectItem, TaskItem } from '../types/communications'
import { senderLabel, senderEmail } from '../stores/communications'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
  projects: ProjectItem[]
  tasks: TaskItem[]
}>()

const message = props.detail?.message ?? null
const sender = message ? senderLabel(message.sender) : ''
const email = message ? senderEmail(message.sender) : ''
</script>

<template>
  <div class="context-rail">
    <div v-if="!detail" class="rail-empty">
      <Icon icon="tabler:sidebar" class="rail-icon" />
      <span>Select a message</span>
    </div>
    <div v-else class="rail-content">
      <!-- Sender profile -->
      <div class="rail-section">
        <h4 class="rail-section-title">Sender</h4>
        <div class="sender-card">
          <div class="sender-avatar">{{ sender.charAt(0).toUpperCase() }}</div>
          <div class="sender-details">
            <span class="sender-name">{{ sender }}</span>
            <span class="sender-email-text">{{ email }}</span>
          </div>
        </div>
      </div>

      <!-- Summary -->
      <div class="rail-section">
        <h4 class="rail-section-title">Summary</h4>
        <p class="rail-text">
          {{ message?.ai_summary || message?.body_text?.slice(0, 200) || 'No summary' }}
        </p>
      </div>

      <!-- Projects -->
      <div class="rail-section">
        <h4 class="rail-section-title">Related Projects</h4>
        <div v-if="projects.length === 0" class="rail-empty-small">No related projects</div>
        <div v-for="p in projects" :key="p.project_id" class="rail-item">
          <Icon icon="tabler:briefcase" class="rail-item-icon" />
          <span>{{ p.name }}</span>
        </div>
      </div>

      <!-- Tasks -->
      <div class="rail-section">
        <h4 class="rail-section-title">Related Tasks</h4>
        <div v-if="tasks.length === 0" class="rail-empty-small">No related tasks</div>
        <div v-for="t in tasks" :key="t.task_id" class="rail-item">
          <Icon icon="tabler:checkbox" class="rail-item-icon" />
          <span>{{ t.title }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.context-rail {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.rail-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
}

.rail-icon {
  width: 32px;
  height: 32px;
  opacity: 0.3;
}

.rail-content {
  flex: 1;
  overflow-y: auto;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.rail-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.rail-section-title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--hh-text-secondary, #6b7280);
  margin: 0;
}

.sender-card {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.sender-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 0.875rem;
  flex-shrink: 0;
}

.sender-details {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  min-width: 0;
}

.sender-name {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.sender-email-text {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.rail-text {
  font-size: 0.75rem;
  color: var(--hh-text-primary, #1f2937);
  margin: 0;
  line-height: 1.4;
}

.rail-empty-small {
  font-size: 0.75rem;
  color: var(--hh-text-tertiary, #9ca3af);
  font-style: italic;
}

.rail-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--hh-text-primary, #1f2937);
}

.rail-item-icon {
  width: 14px;
  height: 14px;
  color: var(--hh-text-tertiary, #9ca3af);
  flex-shrink: 0;
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationsConversationList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsConversationList.vue`
- Size bytes / Размер в байтах: `8498`
- Included characters / Включено символов: `8497`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageSummary, CommunicationThreadSummary, NavigatorMode } from '../types/communications'
import { senderLabel, messageTime } from '../stores/communications'
import { useThreadMessagesPrefetch } from '../queries/communicationPrefetch'

const props = defineProps<{
  accountId: string
  messages: CommunicationMessageSummary[]
  threads: CommunicationThreadSummary[]
  selectedIndex: number
  selectedThreadId: string
  navigatorMode: NavigatorMode
  hasThreadNextPage: boolean
  isFetchingThreadNextPage: boolean
}>()

const prefetchThreadMessages = useThreadMessagesPrefetch()

const emit = defineEmits<{
  select: [index: number]
  selectThread: [thread: CommunicationThreadSummary]
  loadMoreThreads: []
  'update:navigatorMode': [mode: NavigatorMode]
}>()

// Group messages by sender for contacts mode
type ContactGroup = {
  email: string
  label: string
  messages: CommunicationMessageSummary[]
  latestTime: string | null
  unreadCount: number
}

const contactGroups = computed<ContactGroup[]>(() => {
  const groups = new Map<string, CommunicationMessageSummary[]>()
  for (const msg of props.messages) {
    const emailMatch = msg.sender.match(/<(.+?)>/)
    const email = emailMatch ? emailMatch[1] : msg.sender
    if (!groups.has(email)) groups.set(email, [])
    groups.get(email)!.push(msg)
  }
  return Array.from(groups.entries())
    .map(([email, msgs]) => {
      const latest = msgs.reduce((a, b) =>
        (b.projected_at > a.projected_at) ? b : a
      )
      return {
        email,
        label: senderLabel(msgs[0].sender),
        messages: msgs,
        latestTime: latest.projected_at ?? latest.occurred_at,
        unreadCount: msgs.filter(m => m.workflow_state === 'new').length
      }
    })
    .sort((a, b) => {
      const aTime = a.latestTime ?? ''
      const bTime = b.latestTime ?? ''
      return bTime.localeCompare(aTime)
    })
})

function getMessageIndex(msg: CommunicationMessageSummary): number {
  return props.messages.indexOf(msg)
}

function handleThreadPrefetch(thread: CommunicationThreadSummary): void {
  void prefetchThreadMessages(props.accountId, thread.subject)
}
</script>

<template>
  <div class="conversation-list">
    <!-- Navigator mode toggle -->
    <div class="nav-mode-toggle">
      <button
        class="mode-btn"
        :class="{ active: navigatorMode === 'threads' }"
        @click="emit('update:navigatorMode', 'threads')"
      >
        <Icon icon="tabler:list" /> Threads
      </button>
      <button
        class="mode-btn"
        :class="{ active: navigatorMode === 'contacts' }"
        @click="emit('update:navigatorMode', 'contacts')"
      >
        <Icon icon="tabler:users" /> Contacts
      </button>
    </div>

    <!-- Threads mode -->
    <div v-if="navigatorMode === 'threads'" class="thread-list">
      <div
        v-for="thread in threads"
        :key="thread.thread_id"
        class="thread-item"
        :class="{ selected: thread.thread_id === selectedThreadId }"
        tabindex="0"
        @mouseenter="handleThreadPrefetch(thread)"
        @focus="handleThreadPrefetch(thread)"
        @click="emit('selectThread', thread)"
      >
        <div class="thread-sender">
          {{ thread.message_count }} messages
          <span v-if="thread.participant_count > 1"> · {{ thread.participant_count }} participants</span>
        </div>
        <div class="thread-subject-row">
          <span v-if="thread.has_open_action" class="unread-dot" />
          <span class="thread-subject">{{ thread.subject }}</span>
          <Icon v-if="thread.has_attachments" icon="tabler:paperclip" class="thread-attachment-icon" />
        </div>
        <div class="thread-time">{{ messageTime(thread.last_activity_at) }}</div>
      </div>
      <button
        v-if="hasThreadNextPage"
        class="thread-load-more"
        type="button"
        :disabled="isFetchingThreadNextPage"
        @click="emit('loadMoreThreads')"
      >
        <Icon :icon="isFetchingThreadNextPage ? 'tabler:loader-2' : 'tabler:chevron-down'" />
      </button>
    </div>

    <!-- Contacts mode -->
    <div v-else class="contact-list">
      <div v-for="group in contactGroups" :key="group.email" class="contact-group">
        <div class="contact-header">
          <span class="contact-label">{{ group.label }}</span>
          <span class="contact-count">{{ group.messages.length }}</span>
        </div>
        <div
          v-for="msg in group.messages"
          :key="msg.message_id"
          class="contact-message"
          :class="{ selected: getMessageIndex(msg) === selectedIndex }"
          @click="emit('select', getMessageIndex(msg))"
        >
          <span v-if="msg.workflow_state === 'new'" class="unread-dot" />
          <span class="contact-msg-subject">{{ msg.subject }}</span>
          <span class="contact-msg-time">{{ messageTime(msg.projected_at ?? msg.occurred_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conversation-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.nav-mode-toggle {
  display: flex;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  padding: 0.25rem;
  gap: 0.25rem;
}

.mode-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.375rem 0.5rem;
  border: none;
  background: transparent;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  transition: background-color 0.1s;
}
.mode-btn:hover {
  background: var(--hh-bg-hover, #f3f4f6);
}
.mode-btn.active {
  background: var(--hh-bg-selected, #eff6ff);
  color: var(--hh-accent, #3b82f6);
  font-weight: 500;
}

.thread-list,
.contact-list {
  flex: 1;
  overflow-y: auto;
}

.thread-item {
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}
.thread-item:hover {
  background: var(--hh-bg-hover, #f3f4f6);
}
.thread-item.selected {
  background: var(--hh-bg-selected, #eff6ff);
  border-left: 3px solid var(--hh-accent, #3b82f6);
}

.thread-sender {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
}

.thread-subject-row {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.unread-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  flex-shrink: 0;
}

.thread-subject {
  flex: 1;
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.thread-time {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  text-align: right;
}

.thread-attachment-icon {
  flex: 0 0 auto;
  width: 0.875rem;
  height: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
}

.thread-load-more {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 2.5rem;
  margin: 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
}

.thread-load-more:hover:not(:disabled) {
  background: var(--hh-bg-hover, #f3f4f6);
}

.thread-load-more:disabled {
  cursor: wait;
  opacity: 0.65;
}

.contact-group {
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.contact-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-secondary, #f9fafb);
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.contact-count {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.contact-message {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem 0.375rem 1.5rem;
  cursor: pointer;
  font-size: 0.75rem;
}
.contact-message:hover {
  background: var(--hh-bg-hover, #f3f4f6);
}
.contact-message.selected {
  background: var(--hh-bg-selected, #eff6ff);
}

.contact-msg-subject {
  flex: 1;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contact-msg-time {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  white-space: nowrap;
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationsDetailPane.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsDetailPane.vue`
- Size bytes / Размер в байтах: `4035`
- Included characters / Включено символов: `4035`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import CommunicationViewer from './CommunicationViewer.vue'
import ThreadConversationView from './ThreadConversationView.vue'
import type {
  AiReplyResponse,
  CommunicationMessageDetailResponse,
  CommunicationMessageInsight,
  CommunicationThreadSummary,
  MessageContextTab,
  MessageExportFormat,
  ThreadMessage
} from '../types/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'

defineProps<{
  detail: CommunicationMessageDetailResponse | null
  insight: CommunicationMessageInsight | null
  activeTab: MessageContextTab
  selectedThread: CommunicationThreadSummary | null
  threadMessages: ThreadMessage[]
  isThreadLoading: boolean
  threadErrorMessage: string
  isThreadReplySending: boolean
}>()

const emit = defineEmits<{
  'update:activeTab': [tab: MessageContextTab]
  reply: []
  replyAll: []
  forwardMessage: []
  redirectMessage: [recipientsText: string]
  createTask: []
  createNote: []
  translate: []
  generateAiReply: [payload: { tone: string; language: string }]
  applyAiReply: [payload: AiReplyResponse]
  reviewSecurity: []
  reviewRecipients: []
  analyze: []
  markMessageRead: []
  markMessageUnread: []
  deleteFromProvider: []
  togglePin: []
  toggleImportant: []
  mute: []
  exportMessage: [format: MessageExportFormat]
  addLabel: [label: string]
  removeLabel: [label: string]
  snoozeMessage: [until: string]
  openCompose: []
  sendBilingualReply: [payload: BilingualReplyFlowResponse]
  openThreadMessage: [messageId: string]
  replyToThreadMessage: [message: ThreadMessage, bodyHtml: string, draftId: string]
  saveThreadReplyDraft: [message: ThreadMessage, bodyHtml: string, draftId: string]
  sendThreadReply: [message: ThreadMessage, bodyHtml: string, draftId: string]
}>()
</script>

<template>
  <main class="communications-detail-pane">
    <ThreadConversationView
      v-if="selectedThread"
      :thread="selectedThread"
      :messages="threadMessages"
      :is-loading="isThreadLoading"
      :error-message="threadErrorMessage"
      :is-sending-reply="isThreadReplySending"
      @open-message="emit('openThreadMessage', $event)"
      @reply-to-message="(message, bodyHtml, draftId) => emit('replyToThreadMessage', message, bodyHtml, draftId)"
      @save-reply-draft="(message, bodyHtml, draftId) => emit('saveThreadReplyDraft', message, bodyHtml, draftId)"
      @send-reply="(message, bodyHtml, draftId) => emit('sendThreadReply', message, bodyHtml, draftId)"
    />
    <CommunicationViewer
      v-else
      :detail="detail"
      :insight="insight"
      :active-tab="activeTab"
      @update:active-tab="emit('update:activeTab', $event)"
      @reply="emit('reply')"
      @reply-all="emit('replyAll')"
      @forward-message="emit('forwardMessage')"
      @redirect-message="emit('redirectMessage', $event)"
      @create-task="emit('createTask')"
      @create-note="emit('createNote')"
      @translate="emit('translate')"
      @generate-ai-reply="emit('generateAiReply', $event)"
      @apply-ai-reply="emit('applyAiReply', $event)"
      @review-security="emit('reviewSecurity')"
      @review-recipients="emit('reviewRecipients')"
      @analyze="emit('analyze')"
      @mark-message-read="emit('markMessageRead')"
      @mark-message-unread="emit('markMessageUnread')"
      @delete-from-provider="emit('deleteFromProvider')"
      @toggle-pin="emit('togglePin')"
      @toggle-important="emit('toggleImportant')"
      @mute="emit('mute')"
      @export-message="emit('exportMessage', $event)"
      @add-label="emit('addLabel', $event)"
      @remove-label="emit('removeLabel', $event)"
      @snooze-message="emit('snoozeMessage', $event)"
      @open-compose="emit('openCompose')"
      @send-bilingual-reply="emit('sendBilingualReply', $event)"
    />
  </main>
</template>

<style scoped>
.communications-detail-pane {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--hh-bg-primary, #ffffff);
  backdrop-filter: blur(var(--hh-panel-blur));
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationsListPane.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsListPane.vue`
- Size bytes / Размер в байтах: `3542`
- Included characters / Включено символов: `3542`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import CommunicationsConversationList from './CommunicationsConversationList.vue'
import CommunicationList from './CommunicationList.vue'
import type { CommunicationMessageSummary, CommunicationThreadSummary, NavigatorMode } from '../types/communications'

defineProps<{
  accountId: string
  messages: CommunicationMessageSummary[]
  threads: CommunicationThreadSummary[]
  selectedIndex: number
  selectedThreadId: string
  selectedMessageIds: string[]
  navigatorMode: NavigatorMode
  isFolderMode: boolean
  isLoading: boolean
  hasNextPage: boolean
  isFetchingNextPage: boolean
  hasThreadNextPage: boolean
  isFetchingThreadNextPage: boolean
  errorMessage: string
}>()

const emit = defineEmits<{
  select: [index: number]
  selectThread: [thread: CommunicationThreadSummary]
  toggleSelection: [messageId: string, extendRange: boolean]
  selectVisible: [messageIds: string[]]
  clearSelection: []
  loadMore: []
  loadMoreThreads: []
  'update:navigatorMode': [mode: NavigatorMode]
}>()

function forwardToggleSelection(messageId: string, extendRange: boolean) {
  emit('toggleSelection', messageId, extendRange)
}
</script>

<template>
  <nav class="communications-list-pane">
    <div v-if="errorMessage" class="pane-state error">
      <Icon icon="tabler:alert-circle" />
      <span>{{ errorMessage }}</span>
    </div>
    <div v-else-if="isLoading" class="pane-state">
      <Icon icon="tabler:loader-2" class="spin-icon" />
      <span>Loading messages...</span>
    </div>
    <div v-else-if="!isFolderMode && (navigatorMode === 'threads' || navigatorMode === 'contacts')" class="pane-content">
      <CommunicationsConversationList
        :account-id="accountId"
        :messages="messages"
        :threads="threads"
        :selected-index="selectedIndex"
        :selected-thread-id="selectedThreadId"
        :navigator-mode="navigatorMode"
        :has-thread-next-page="hasThreadNextPage"
        :is-fetching-thread-next-page="isFetchingThreadNextPage"
        @select="emit('select', $event)"
        @select-thread="emit('selectThread', $event)"
        @load-more-threads="emit('loadMoreThreads')"
        @update:navigator-mode="emit('update:navigatorMode', $event)"
      />
    </div>
    <CommunicationList
      v-else
      :messages="messages"
      :selected-index="selectedIndex"
      :selected-message-ids="selectedMessageIds"
      :is-loading="isLoading"
      :has-next-page="hasNextPage"
      :is-fetching-next-page="isFetchingNextPage"
      @select="emit('select', $event)"
      @toggle-selection="forwardToggleSelection"
      @select-visible="emit('selectVisible', $event)"
      @clear-selection="emit('clearSelection')"
      @load-more="emit('loadMore')"
    />
  </nav>
</template>

<style scoped>
.communications-list-pane {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-primary, #ffffff);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.pane-content {
  flex: 1;
  min-height: 0;
}

.pane-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  height: 100%;
  padding: 2rem;
  font-size: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
  text-align: center;
}

.pane-state.error {
  color: var(--hh-text-error, #ef4444);
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationsRailPane.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsRailPane.vue`
- Size bytes / Размер в байтах: `1118`
- Included characters / Включено символов: `1118`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import CommunicationsContextInspector from './CommunicationsContextInspector.vue'
import CommunicationsContextRail from './CommunicationsContextRail.vue'
import type { InspectorMode, CommunicationMessageDetailResponse, ProjectItem, TaskItem } from '../types/communications'

defineProps<{
  detail: CommunicationMessageDetailResponse | null
  inspectorMode: InspectorMode
  projects: ProjectItem[]
  tasks: TaskItem[]
}>()

const emit = defineEmits<{
  'update:inspectorMode': [mode: InspectorMode]
}>()
</script>

<template>
  <aside class="communications-rail-pane">
    <CommunicationsContextInspector
      v-if="detail"
      :detail="detail"
      :inspector-mode="inspectorMode"
      @update:inspector-mode="emit('update:inspectorMode', $event)"
    />
    <CommunicationsContextRail v-else :detail="detail" :projects="projects" :tasks="tasks" />
  </aside>
</template>

<style scoped>
.communications-rail-pane {
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-primary, #ffffff);
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationsTopbarSlot.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsTopbarSlot.vue`
- Size bytes / Размер в байтах: `4269`
- Included characters / Включено символов: `4269`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'

const props = defineProps<{
  searchQuery: string
  isSyncBusy: boolean
}>()

const emit = defineEmits<{
  'update:searchQuery': [query: string]
  search: []
  openAccountSetup: []
  compose: []
  syncNow: []
}>()

function updateSearchQuery(event: Event) {
  emit('update:searchQuery', (event.target as HTMLInputElement).value)
}
</script>

<template>
  <div class="communications-topbar-slot">
    <div class="communications-topbar-main">
      <h1 class="communications-topbar-title">Mail</h1>
      <label class="communications-topbar-search" aria-label="Search messages">
        <Icon icon="tabler:search" class="search-icon" />
        <input
          type="text"
          placeholder="Search messages..."
          :value="searchQuery"
          @input="updateSearchQuery"
          @keyup.enter="emit('search')"
        />
      </label>

      <div class="communications-topbar-actions">
        <button
          type="button"
          class="communications-topbar-icon-btn"
          title="Add mail account"
          aria-label="Add mail account"
          @click="emit('openAccountSetup')"
        >
          <Icon icon="tabler:mail-plus" />
        </button>
        <button
          type="button"
          class="communications-topbar-icon-btn"
          title="Compose"
          aria-label="Compose"
          @click="emit('compose')"
        >
          <Icon icon="tabler:edit" />
        </button>
        <button
          type="button"
          class="communications-topbar-icon-btn"
          :disabled="isSyncBusy"
          title="Refresh"
          aria-label="Refresh"
          @click="emit('syncNow')"
        >
          <Icon icon="tabler:refresh" :class="isSyncBusy ? 'spin-icon' : ''" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.communications-topbar-slot {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  min-width: 0;
  height: 100%;
  padding: 0;
}

.communications-topbar-main {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex: 1;
  min-width: 0;
}

.communications-topbar-title {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 1rem;
  font-weight: 760;
  line-height: 1;
  white-space: nowrap;
}

.communications-topbar-search {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  flex: 1;
  height: 2rem;
  min-width: 12rem;
  max-width: 32rem;
  padding: 0 0.625rem;
  border: 1px solid var(--hh-border-subtle, #e5e7eb);
  border-radius: var(--hh-radius-control, 0.375rem);
  background: rgba(2, 12, 16, calc(var(--hh-panel-alpha, 0.7) * 0.72));
  color: var(--hh-text-primary, #1f2937);
}

.search-icon {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  color: var(--hh-text-tertiary, #9ca3af);
}

.communications-topbar-search input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.communications-topbar-search input::placeholder {
  color: var(--hh-text-tertiary, #9ca3af);
}

.communications-topbar-actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 0.25rem;
}

.communications-topbar-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: 1px solid transparent;
  border-radius: var(--hh-radius-control, 0.375rem);
  background: transparent;
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease, border-color 150ms ease, opacity 150ms ease;
}

.communications-topbar-icon-btn:hover:not(:disabled) {
  border-color: var(--hh-border-subtle, #e5e7eb);
  background: var(--hh-hover-bg, rgba(255, 255, 255, 0.06));
  color: var(--hh-text-primary, #1f2937);
}

.communications-topbar-icon-btn:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.communications-topbar-icon-btn:disabled {
  cursor: not-allowed;
  opacity: 0.48;
}

.communications-topbar-icon-btn :deep(svg) {
  width: 1.125rem;
  height: 1.125rem;
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
```

### `frontend/src/domains/communications/components/CommunicationsWorkbench.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/CommunicationsWorkbench.vue`
- Size bytes / Размер в байтах: `590`
- Included characters / Включено символов: `590`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
defineProps<{
  isLoading: boolean
  hasError: boolean
  hasRail: boolean
}>()
</script>

<template>
  <section class="communications-workbench" :class="{ 'has-rail': hasRail, 'is-loading': isLoading, 'has-error': hasError }">
    <slot name="list" />
    <slot name="detail" />
    <slot v-if="hasRail" name="rail" />
  </section>
</template>

<style scoped>
.communications-workbench {
  flex: 1;
  display: grid;
  grid-template-columns: 280px 1fr;
  overflow: hidden;
}

.communications-workbench.has-rail {
  grid-template-columns: 240px 1fr 280px;
}
</style>
```

### `frontend/src/domains/communications/components/ComposeDrawer.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeDrawer.css`
- Size bytes / Размер в байтах: `7441`
- Included characters / Включено символов: `7441`
- Truncated / Обрезано: `no`

```text
.compose-drawer.makosh-sheet-content {
  width: min(560px, 92vw);
  max-width: min(560px, 92vw);
  max-height: 100vh;
  background: var(--hh-bg-primary, #ffffff);
  box-shadow: -4px 0 24px rgba(0, 0, 0, 0.12);
  overflow: hidden;
}

.compose-drawer.makosh-sheet-content .makosh-sheet-header {
  padding: 0.875rem 1rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
}

.compose-drawer.makosh-sheet-content .makosh-sheet-body {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  padding: 0;
}

.compose-drawer .compose-header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding-right: 2.25rem;
}

.compose-drawer .saving-indicator {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.compose-drawer .spin-icon {
  animation: compose-drawer-spin 1s linear infinite;
  width: 14px;
  height: 14px;
}

@keyframes compose-drawer-spin {
  to { transform: rotate(360deg); }
}

.compose-drawer .compose-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: 0.75rem 1rem;
  gap: 0.5rem;
}

.compose-drawer .form-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.compose-drawer .form-field label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
}

.compose-drawer .form-field input {
  padding: 0.4375rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-primary, #ffffff);
  outline: none;
  transition: border-color 0.15s;
}

.compose-drawer .form-field input:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.compose-drawer .field-error {
  font-size: 0.6875rem;
  color: var(--hh-text-error, #ef4444);
}

.compose-drawer .body-field {
  flex: 1;
}

.compose-drawer .body-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.compose-drawer .body-mode-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.125rem;
  padding: 0.125rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: var(--hh-bg-secondary, #f9fafb);
}

.compose-drawer .body-mode-toggle button {
  border: 0;
  border-radius: 0.25rem;
  padding: 0.25rem 0.5rem;
  background: transparent;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  cursor: pointer;
}

.compose-drawer .body-mode-toggle button.active {
  background: var(--hh-bg-primary, #ffffff);
  color: var(--hh-text-primary, #1f2937);
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.12);
}

.compose-drawer .body-field textarea {
  flex: 1;
  min-height: 200px;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  font-family: inherit;
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-primary, #ffffff);
  resize: vertical;
  outline: none;
  line-height: 1.6;
}

.compose-drawer .body-field textarea.html-body-editor {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.compose-drawer .delivery-options {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 10rem;
  gap: 0.75rem;
}

.compose-drawer .delivery-field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.compose-drawer .delivery-field span {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
}

.compose-drawer .delivery-field input,
.compose-drawer .delivery-field select {
  padding: 0.4375rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
  background: var(--hh-bg-primary, #ffffff);
  outline: none;
}

.compose-drawer .delivery-field input:focus,
.compose-drawer .delivery-field select:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.compose-drawer .body-field textarea:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.compose-drawer .compose-attachments {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.compose-drawer .attachment-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.compose-drawer .attachment-header span {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
}

.compose-drawer .attachment-header button,
.compose-drawer .attachment-list button {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.25rem;
  background: var(--hh-bg-primary, #ffffff);
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  font-size: 0.75rem;
  padding: 0.25rem 0.5rem;
}

.compose-drawer .attachment-input {
  display: none;
}

.compose-drawer .attachment-drop-zone {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  min-height: 2.5rem;
  border: 1px dashed var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 78%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.compose-drawer .attachment-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  list-style: none;
  margin: 0;
  padding: 0;
}

.compose-drawer .attachment-list li {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: var(--hh-bg-primary, #ffffff);
}

.compose-drawer .attachment-name {
  overflow: hidden;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.compose-drawer .attachment-meta,
.compose-drawer .attachment-warning {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.compose-drawer .attachment-warning {
  margin: 0;
}

.compose-drawer .compose-error {
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-error-light, #fef2f2);
  color: var(--hh-text-error, #ef4444);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
}

.compose-drawer .compose-status {
  padding: 0.375rem 0.75rem;
  background: var(--hh-bg-success-light, #f0fdf4);
  color: var(--hh-text-success, #16a34a);
  border-radius: 0.375rem;
  font-size: 0.8125rem;
}

.compose-drawer .compose-actions {
  display: flex;
  gap: 0.5rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--hh-border, #e5e7eb);
}

.compose-drawer .delete-btn {
  margin-left: auto;
}

.compose-drawer .send-review {
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.compose-drawer .send-review h3 {
  font-size: 1rem;
  font-weight: 600;
  margin: 0;
}

.compose-drawer .review-field {
  display: flex;
  gap: 0.5rem;
}

.compose-drawer .review-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-secondary, #6b7280);
  min-width: 60px;
  flex-shrink: 0;
}

.compose-drawer .review-value {
  font-size: 0.8125rem;
  color: var(--hh-text-primary, #1f2937);
}

.compose-drawer .review-actions {
  display: flex;
  gap: 0.5rem;
  padding-top: 0.5rem;
}
```

### `frontend/src/domains/communications/components/ComposeDrawer.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeDrawer.vue`
- Size bytes / Размер в байтах: `18653`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { ref, computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import Sheet from '../../../shared/ui/Sheet.vue'
import ComposeSignaturePicker from './ComposeSignaturePicker.vue'
import ComposeTemplatePicker from './ComposeTemplatePicker.vue'
import RichComposeEditor from './RichComposeEditor.vue'
import { useCommunicationsStore } from '../stores/communications'
import type { ComposeFormModel } from '../types/communications'
import {
  useDeleteDraftMutation,
  useSaveDraftMutation,
  useSendMailMutation
} from '../queries/useCommunicationsQuery'
import { useComposeDraftAutosave } from '../forms/composeDraftAutosave'
import { datetimeLocalToIso } from '../forms/composeDraftAutosave'
import { splitComposeRecipients, useComposeValidation } from '../forms/composeValidation'
import {
  appendHtmlSignature,
  appendPlainTextSignature,
  htmlToComposePlainText,
  plainTextToComposeHtml
} from './richComposeHtml'
import './ComposeDrawer.css'

const store = useCommunicationsStore()
const sendMailMutation = useSendMailMutation()
const saveDraftMutation = useSaveDraftMutation()
const deleteDraftMutation = useDeleteDraftMutation()

const isSaving = computed(() => saveDraftMutation.isPending.value)
const isSending = computed(() => sendMailMutation.isPending.value)
const { errors: composeValidationErrors, validateForSend } = useComposeValidation(() => store.composeForm)
const htmlEditorMode = ref<'rich' | 'source'>('rich')
const attachmentInput = ref<HTMLInputElement | null>(null)
type StagedComposeAttachment = {
  id: string
  file: File
  name: string
  size: number
  type: string
}
const stagedAttachments = ref<StagedComposeAttachment[]>([])
const hasStagedAttachments = computed(() => stagedAttachments.value.length > 0)

const draftAutosave = useComposeDraftAutosave({
  formSource: () => store.composeForm,
  saveDraft: (payload) => saveDraftMutation.mutateAsync(payload),
  onSaved: () => store.setComposeStatusMessage('Draft saved'),
  onError: (error) => {
    store.setComposeSendError(error instanceof Error ? error.message : 'Draft save failed')
  }
})

async function handleSaveDraft() {
  store.setComposeSendError('')
  await draftAutosave.saveNow()
}

function triggerAutoSave() {
  draftAutosave.schedule()
}

async function handleSend() {
  const form = store.composeForm
  if (isSending.value) return
  store.setComposeSendError('')
  if (hasStagedAttachments.value) {
    store.setComposeSendError('Attachment upload is not connected to provider send yet; remove staged attachments before sending')
    return
  }
  if (!(await validateForSend())) {
    store.setComposeSendError('Fix compose validation errors before sending')
    return
  }
  try {
    const result = await sendMailMutation.mutateAsync({
      account_id: form.accountId,
      to: splitComposeRecipients(form.toText),
      cc: splitComposeRecipients(form.ccText),
      bcc: splitComposeRecipients(form.bccText),
      subject: form.subject,
      body_text: form.body,
      body_html: form.bodyFormat === 'html' ? form.bodyHtml : null,
      in_reply_to: form.inReplyTo,
      draft_id: form.draftId,
      scheduled_send_at: datetimeLocalToIso(form.scheduledSendAt),
      undo_send_seconds: form.undoSendSeconds,
      confirmed_provider_write: true
    })
    store.setComposeStatusMessage(`Sent via ${result.transport ?? 'provider'}`)
    store.closeCompose()
  } catch (error) {
    store.setComposeSendError(error instanceof Error ? error.message : 'Send failed')
  }
}

async function handleDeleteCurrentDraft() {
  const draftId = store.composeForm.draftId?.trim()
  if (!draftId) return
  store.setComposeSendError('')
  draftAutosave.cancel()
  try {
    await deleteDraftMutation.mutateAsync(draftId)
    store.closeCompose()
  } catch (error) {
    store.setComposeSendError(error instanceof Error ? error.message : 'Delete failed')
  }
}

async function handleClose() {
  await draftAutosave.flush()
  stagedAttachments.value = []
  store.closeCompose()
}

function handleSheetOpenChange(open: boolean) {
  if (open) return
  void handleClose()
}

function updateField<K extends keyof ComposeFormModel>(key: K, value: ComposeFormModel[K]) {
  store.updateComposeForm({ [key]: value })
  if (key !== 'draftId' && key !== 'accountId') {
    triggerAutoSave()
  }
}

function setBodyFormat(format: ComposeFormModel['bodyFormat'], htmlMode: 'rich' | 'source' = 'rich') {
  if (format === 'html') {
    htmlEditorMode.value = htmlMode
  }
  updateField('bodyFormat', format)
  if (format === 'html' && store.composeForm.bodyHtml === null) {
    updateField('bodyHtml', htmlMode === 'rich'
      ? plainTextToComposeHtml(store.composeForm.body)
      : store.composeForm.body)
  }
}

function updateHtmlBody(value: string) {
  store.updateComposeForm({
    bodyHtml: value,
    body: htmlToComposePlainText(value)
  })
  triggerAutoSave()
}

function applyRenderedTemplate(payload: { subject: string; bodyHtml: string }) {
  htmlEditorMode.value = 'rich'
  store.updateComposeForm({
    subject: payload.subject,
    bodyFormat: 'html',
    bodyHtml: payload.bodyHtml,
    body: htmlToComposePlainText(payload.bodyHtml)
  })
  store.setComposeStatusMessage('Template applied')
  triggerAutoSave()
}

function applySignature(signature: string) {
  const trimmed = signature.trim()
  if (!trimmed) return

  if (store.composeForm.bodyFormat === 'html') {
    updateHtmlBody(appendHtmlSignature(store.composeForm.bodyHtml, trimmed))
  } else {
    updateField('body', appendPlainTextSignature(store.composeForm.body, trimmed))
  }
  store.setComposeStatusMessage('Signature inserted')
}

function handleAttachmentFiles(files: File[] | FileList) {
  const nextFiles = Array.from(files).filter((file) => file.size >= 0)
  if (nextFiles.length === 0) return
  const existingKeys = new Set(stagedAttachments.value.map((attachment) => attachment.id))
  const nextAttachments = nextFiles
    .map((file) => ({
      id: composeAttachmentId(file),
      file,
      name: file.name || 'Untitled attachment',
      size: file.size,
      type: file.type || 'application/octet-stream'
    }))
    .filter((attachment) => !existingKeys.has(attachment.id))
  if (nextAttachments.length === 0) {
    store.setComposeStatusMessage('Attachment already staged')
    return
  }
  stagedAttachments.value = [...stagedAttachments.value, ...nextAttachments]
  store.setComposeStatusMessage(`${nextAttachments.length} attachment${nextAttachments.length === 1 ? '' : 's'} staged locally`)
}

function handleAttachmentInput(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files) handleAttachmentFiles(input.files)
  input.value = ''
}

function handleAttachmentDrop(event: DragEvent) {
  handleAttachmentFiles(event.dataTransfer?.files ?? [])
}

function openAttachmentPicker() {
  attachmentInput.value?.click()
}

function removeStagedAttachment(id: string) {
  stagedAttachments.value = stagedAttachments.value.filter((attachment) => attachment.id !== id)
}

function composeAttachmentId(file: File): string {
  return `${file.name}:${file.size}:${file.lastModified}`
}

function formatAttachmentSize(size: number): string {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / (1024 * 1024)).toFixed(1)} MB`
}

// Send review
const isReviewOpen = ref(false)

async function openSendReview() {
  store.setComposeSendError('')
  if (hasStagedAttachments.value) {
    store.setComposeSendError('Attachment upload is not connected to provider send yet; remove staged attachments before sending')
    return
  }
  if (!(await validateForSend())) {
    store.setComposeSendError('Fix compose validation errors before sending')
    return
  }
  isReviewOpen.value = true
}

function closeSendReview() {
  isReviewOpen.value = false
}

function confirmSend() {
  isReviewOpen.value = false
  void handleSend()
}

// Mode label
const modeLabel = computed(() => {
  switch (store.composeForm.mode) {
    case 'reply': return 'Reply'
    case 'forward': return 'Forward'
    default: return 'New Message'
  }
})

const deliveryActionLabel = computed(() => {
  return store.composeForm.scheduledSendAt ? 'Schedule' : 'Send'
})
const scheduledSendReviewLabel = computed(() => {
  if (!store.composeForm.scheduledSendAt) return ''
  const timestamp = Date.parse(store.composeForm.scheduledSendAt)
  if (!Number.isFinite(timestamp)) return store.composeForm.scheduledSendAt
  return new Intl.DateTimeFormat('en-US', {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format(new Date(timestamp))
})
const undoSendReviewLabel = computed(() => {
  return store.composeForm.undoSendSeconds
    ? `${store.composeForm.undoSendSeconds} seconds`
    : 'Off'
})
</script>

<template>
  <Sheet
    :open="true"
    side="right"
    :title="modeLabel"
    content-class="compose-drawer"
    @update:open="handleSheetOpenChange"
  >
    <template #header>
      <div class="compose-header-actions">
        <span v-if="isSaving" class="saving-indicator">
          <Icon icon="tabler:loader-2" class="spin-icon" /> Saving...
        </span>
      </div>
    </template>

    <!-- Send review step -->
    <div v-if="isReviewOpen" class="send-review">
      <h3>Review before sending</h3>
      <div class="review-field">
        <span class="review-label">To:</span>
        <span class="review-value">{{ store.composeForm.toText }}</span>
      </div>
      <div v-if="store.composeForm.ccText" class="review-field">
        <span class="review-label">CC:</span>
        <span class="review-value">{{ store.composeForm.ccText }}</span>
      </div>
      <div v-if="store.composeForm.bccText" class="review-field">
        <span class="review-label">BCC:</span>
        <span class="review-value">{{ store.composeForm.bccText }}</span>
      </div>
      <div class="review-field">
        <span class="review-label">Subject:</span>
        <span class="review-value">{{ store.composeForm.subject || '(No subject)' }}</span>
      </div>
      <div v-if="scheduledSendReviewLabel" class="review-field">
        <span class="review-label">Schedule:</span>
        <span class="review-value">{{ scheduledSendReviewLabel }}</span>
      </div>
      <div class="review-field">
        <span class="review-label">Undo:</span>
        <span class="review-value">{{ undoSendReviewLabel }}</span>
      </div>
      <div class="review-actions">
        <Button variant="default" @click="confirmSend" :disabled="isSending">
          <Icon icon="tabler:send" /> {{ isSending ? 'Sending...' : deliveryActionLabel }}
        </Button>
        <Button variant="ghost" @click="closeSendReview">Edit</Button>
      </div>
    </div>

    <!-- Compose form -->
    <div v-else class="compose-form">
        <div class="form-field">
          <label>To</label>
          <input
            type="text"
            placeholder="recipient@example.com"
            :value="store.composeForm.toText"
            @input="updateField('toText', ($event.target as HTMLInputElement).value)"
          />
          <span v-if="composeValidationErrors.toText" class="field-error">
            {{ composeValidationErrors.toText }}
          </span>
        </div>
        <div class="form-field">
          <label>CC</label>
          <input
            type="text"
            placeholder="cc@example.com"
            :value="store.composeForm.ccText"
            @input="updateField('ccText', ($event.target as HTMLInputElement).value)"
          />
          <span v-if="composeValidationErrors.ccText" class="field-error">
            {{ composeValidationErrors.ccText }}
          </span>
        </div>
        <div class="form-field">
          <label>BCC</label>
          <input
            type="text"
            placeholder="bcc@example.com"
            :value="store.composeForm.bccText"
            @input="updateField('bccText', ($event.target
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/ComposeSignaturePicker.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeSignaturePicker.vue`
- Size bytes / Размер в байтах: `2893`
- Included characters / Включено символов: `2893`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import { usePersonasQuery } from '../queries/useCommunicationsQuery'

const emit = defineEmits<{
  apply: [signature: string]
}>()

const personasQuery = usePersonasQuery()
const selectedPersonaId = ref('')

const personasWithSignatures = computed(() => {
  return (personasQuery.data.value ?? []).filter((persona) => persona.signature.trim())
})
const selectedPersona = computed(() => {
  return personasWithSignatures.value.find((persona) => persona.persona_id === selectedPersonaId.value) ?? null
})
const isLoading = computed(() => personasQuery.isPending.value)
const canApplySignature = computed(() => Boolean(selectedPersona.value?.signature.trim()))

function applySignature(): void {
  const signature = selectedPersona.value?.signature.trim()
  if (!signature) return
  emit('apply', signature)
}
</script>

<template>
  <section class="compose-signature-picker" aria-label="Email signatures">
    <label>
      <span>Signature</span>
      <select
        :value="selectedPersonaId"
        :disabled="isLoading || personasWithSignatures.length === 0"
        @change="selectedPersonaId = ($event.target as HTMLSelectElement).value"
      >
        <option value="">
          {{ isLoading ? 'Loading signatures...' : 'No signature' }}
        </option>
        <option
          v-for="persona in personasWithSignatures"
          :key="persona.persona_id"
          :value="persona.persona_id"
        >
          {{ persona.display_name || persona.name }}
        </option>
      </select>
    </label>
    <Button
      variant="secondary"
      size="sm"
      :disabled="!canApplySignature"
      @click="applySignature"
    >
      <Icon icon="tabler:signature" size="16" />
      Insert
    </Button>
  </section>
</template>

<style scoped>
.compose-signature-picker {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.5rem;
  align-items: end;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
}

.compose-signature-picker label {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.25rem;
}

.compose-signature-picker span {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  font-weight: 500;
}

.compose-signature-picker select {
  min-width: 0;
  padding: 0.4375rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: var(--hh-bg-primary, #ffffff);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  outline: none;
}

.compose-signature-picker select:focus {
  border-color: var(--hh-accent, #3b82f6);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}
</style>
```

### `frontend/src/domains/communications/components/ComposeTemplatePicker.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeTemplatePicker.css`
- Size bytes / Размер в байтах: `7280`
- Included characters / Включено символов: `7280`
- Truncated / Обрезано: `no`

```text
.compose-template-picker {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
}

.template-toolbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.template-toolbar-main {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.template-toolbar-side {
  display: flex;
  flex: 1 1 18rem;
  flex-direction: column;
  gap: 0.5rem;
  min-width: 16rem;
}

.template-library-search,
.template-category-filter {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.template-library-search span,
.template-category-filter span,
.template-library-preview span,
.template-variables span,
.template-mail-merge-preview label > span,
.template-save-form label > span,
.template-recipient-mapping label > span,
.template-recipient-mapping-summary {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 500;
}

.template-library-search-row,
.template-category-filter-row,
.template-recipient-mapping-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.template-library-search-row input,
.template-variables input,
.template-save-form input,
.template-recipient-mapping select,
.template-mail-merge-preview textarea {
  width: 100%;
  min-height: 2rem;
  padding: 0.4375rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: var(--hh-bg-primary, #ffffff);
  color: var(--hh-text-primary, #1f2937);
}

.template-category-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 1.875rem;
  padding: 0.25rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.template-category-chip.active {
  border-color: var(--hh-accent, #3b82f6);
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 12%, var(--hh-bg-primary, #ffffff));
  color: var(--hh-text-primary, #111827);
}

.template-library-surface {
  display: grid;
  grid-template-columns: minmax(14rem, 1.35fr) minmax(13rem, 1fr);
  gap: 0.625rem;
}

.template-library-list {
  display: grid;
  gap: 0.375rem;
  max-height: 16rem;
  overflow: auto;
  padding-right: 0.125rem;
}

.template-library-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25rem;
  text-align: left;
  min-height: 2.125rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
  color: var(--hh-text-primary, #111827);
  padding: 0.4375rem 0.625rem;
}

.template-library-item-title {
  font-size: 0.75rem;
  color: var(--hh-text-primary, #111827);
}

.template-library-item-meta,
.template-library-preview-categories {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  flex-wrap: wrap;
}

.template-library-item-count,
.template-library-item-updated,
.template-library-item-diagnostics {
  font-size: 0.6875rem;
  color: var(--hh-text-muted, #9ca3af);
}

.template-library-item-diagnostics {
  color: var(--hh-warning, #b45309);
}

.template-library-tag {
  display: inline-flex;
  align-items: center;
  min-height: 1.25rem;
  padding: 0.125rem 0.375rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f3f4f6) 86%, transparent);
  color: var(--hh-text-muted, #9ca3af);
  font-size: 0.625rem;
}

.template-library-item.active {
  border-color: var(--hh-accent, #3b82f6);
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 8%, var(--hh-bg-primary, #ffffff));
}

.template-library-preview,
.template-recipient-mapping {
  display: grid;
  gap: 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  padding: 0.5rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 94%, transparent);
}

.template-library-preview h4,
.template-mail-merge-preview h4,
.template-recipient-mapping h4 {
  margin: 0;
  color: var(--hh-text-primary, #111827);
  font-size: 0.8125rem;
}

.template-library-preview-meta {
  color: var(--hh-text-muted, #9ca3af);
  font-size: 0.6875rem;
}

.template-library-preview label,
.template-mail-merge-preview label,
.template-save-form label,
.template-recipient-mapping label {
  display: grid;
  gap: 0.25rem;
}

.template-library-preview pre,
.template-mail-merge-preview pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  font: inherit;
  font-size: 0.75rem;
  color: var(--hh-text-primary, #111827);
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  padding: 0.5rem;
  min-height: 4.5rem;
  max-height: 5.5rem;
  overflow: auto;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f3f4f6) 70%, transparent);
}

.template-library-empty,
.template-library-empty-state {
  color: var(--hh-text-muted, #9ca3af);
  font-size: 0.75rem;
}

.template-library-empty-state {
  padding: 0.375rem 0.125rem;
}

.template-diagnostics,
.template-variables,
.template-save-form,
.template-mail-merge-preview,
.template-mail-merge-preview-result {
  display: grid;
  gap: 0.5rem;
}

.template-diagnostic {
  display: flex;
  align-items: flex-start;
  gap: 0.375rem;
  padding: 0.4375rem 0.5rem;
  border-radius: 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 94%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.template-diagnostic[data-kind='error'] {
  border-color: color-mix(in srgb, var(--hh-danger, #dc2626) 35%, var(--hh-border, #e5e7eb));
  color: var(--hh-danger, #b91c1c);
}

.template-recipient-mapping-header,
.template-mail-merge-preview-header,
.template-mail-merge-preview-row-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.template-recipient-mapping-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.5rem;
}

.template-mail-merge-preview-summary {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
}

.template-mail-merge-preview-item {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 90%, transparent);
}

.template-mail-merge-preview-item[data-ready='false'] {
  border-color: color-mix(in srgb, var(--hh-danger, #ef4444) 42%, var(--hh-border, #e5e7eb));
}

.template-save-form {
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  padding-top: 0.5rem;
  border-top: 1px solid var(--hh-border, #e5e7eb);
}

.template-save-actions {
  display: flex;
  justify-content: flex-end;
}

.template-error {
  color: var(--hh-danger, #dc2626);
  font-size: 0.75rem;
}

@media (max-width: 960px) {
  .template-library-surface,
  .template-recipient-mapping-grid,
  .template-save-form {
    grid-template-columns: 1fr;
  }
}
```

### `frontend/src/domains/communications/components/ComposeTemplatePicker.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ComposeTemplatePicker.vue`
- Size bytes / Размер в байтах: `23395`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Ref } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import TemplateRecipientMappingPanel from './TemplateRecipientMappingPanel.vue'
import TemplateSaveForm from './TemplateSaveForm.vue'
import {
  useCreateRichTemplateMutation,
  useDeleteRichTemplateMutation,
  usePreviewRichTemplateMailMergeMutation,
  useRenderRichTemplateMutation,
  useRichTemplatesQuery
} from '../queries/useCommunicationsQuery'
import {
  missingTemplateVariables,
  parseTemplateMailMergePreviewRows,
  resolveTemplateVariableValues,
  stringifyTemplateMailMergePreviewRows,
  storedTemplateDiagnosticMessages,
  templateContentDiagnostics,
  templateDiagnosticsErrorMessage,
  templateFormDefaults,
  templateFormToInput,
  templateMergeErrorMessage,
  templateVeeValidationSchema,
  type TemplateFormValues
} from '../forms/templateForm'
import {
  applyTemplateRecipientMapping,
  buildTemplateRecipientPreviewRows,
  deriveTemplateLibraryCategories,
  filterTemplateLibraryTemplates,
  formatTemplateUpdatedLabel,
  inferRecipientVariableMapping,
  orderTemplateLibraryTemplates,
  recipientPreviewSummary,
  suggestTemplateSaveName,
  templateLibraryCategoryLabel,
  templateLibraryCategoryOptions,
  type TemplateLibraryCategory,
  type TemplateRecipientVariableMapping
} from './templateLibrary'
import type { CommunicationTemplate, RichTemplateMailMergePreviewResponse } from '../types/communications'
import './ComposeTemplatePicker.css'

const props = defineProps<{
  toText: string
  ccText: string
  bccText: string
  subject: string
  body: string
  bodyHtml: string | null
}>()

const emit = defineEmits<{
  apply: [payload: { subject: string; bodyHtml: string }]
  error: [message: string]
  saved: [name: string]
  deleted: [name: string]
}>()

const templatesQuery = useRichTemplatesQuery()
const createTemplateMutation = useCreateRichTemplateMutation()
const deleteTemplateMutation = useDeleteRichTemplateMutation()
const previewMailMergeMutation = usePreviewRichTemplateMailMergeMutation()
const renderTemplateMutation = useRenderRichTemplateMutation()
const selectedTemplateId = ref('')
const variableValues = ref<Record<string, string>>({})
const isSaveOpen = ref(false)
const deleteConfirmTemplateId = ref('')
const previewRowsText = ref('')
const previewError = ref('')
const previewResult = ref<RichTemplateMailMergePreviewResponse | null>(null)
const selectedCategory = ref<'all' | TemplateLibraryCategory>('all')
const saveMode = ref<'new' | 'duplicate'>('new')
const recipientVariableMapping = ref<TemplateRecipientVariableMapping>({
  toVariable: '',
  ccVariable: '',
  bccVariable: ''
})

const templates = computed(() => templatesQuery.data.value ?? [])
const isTemplatesLoading = computed(() => templatesQuery.isPending.value)
const isSavingTemplate = computed(() => createTemplateMutation.isPending.value)
const isDeletingTemplate = computed(() => deleteTemplateMutation.isPending.value)
const isPreviewingMailMerge = computed(() => previewMailMergeMutation.isPending.value)
const isRenderingTemplate = computed(() => renderTemplateMutation.isPending.value)
const selectedTemplate = computed(() => {
  return templates.value.find((template) => template.template_id === selectedTemplateId.value) ?? null
})
const templateVariables = computed(() => selectedTemplate.value?.variables ?? [])
const selectedTemplateDiagnostics = computed(() => {
  return storedTemplateDiagnosticMessages(selectedTemplate.value)
})
const selectedTemplateBlockingDiagnostics = computed(() => {
  return selectedTemplateDiagnostics.value.filter((message) => message.kind === 'error')
})
const selectedTemplateValidationMessage = computed(() => {
  const blocking = selectedTemplateBlockingDiagnostics.value[0]
  if (!blocking) return ''
  return `${blocking.label}: ${blocking.values.join(', ')}`
})
const missingMergeVariables = computed(() => {
  return missingTemplateVariables(templateVariables.value, variableValues.value)
})
const mergeValidationMessage = computed(() => {
  return templateMergeErrorMessage(missingMergeVariables.value)
})
const hasTemplateContent = computed(() => {
  return Boolean(props.subject.trim() || props.bodyHtml?.trim() || props.body.trim())
})
const saveDiagnostics = computed(() => {
  return templateContentDiagnostics(props.subject, props.bodyHtml ?? props.body)
})
const saveValidationMessage = computed(() => {
  return templateDiagnosticsErrorMessage(saveDiagnostics.value)
})
const canApplyTemplate = computed(() => {
  return Boolean(selectedTemplate.value)
    && !selectedTemplateValidationMessage.value
    && !mergeValidationMessage.value
    && !isRenderingTemplate.value
})
const canSaveTemplate = computed(() => {
  return hasTemplateContent.value && !saveValidationMessage.value && !isSavingTemplate.value
})
const canUpdateTemplate = computed(() => {
  return Boolean(selectedTemplate.value) && hasTemplateContent.value && !saveValidationMessage.value && !isSavingTemplate.value
})
const isDeleteArmed = computed(() => {
  return Boolean(selectedTemplate.value && deleteConfirmTemplateId.value === selectedTemplate.value.template_id)
})
const canDeleteTemplate = computed(() => {
  return Boolean(selectedTemplate.value) && !isDeletingTemplate.value
})
const templateLibraryQuery: Ref<string> = ref('')
const composeRecipientSummary = computed(() => recipientPreviewSummary({
  toText: props.toText,
  ccText: props.ccText,
  bccText: props.bccText
}))

const filteredTemplates = computed(() => {
  return filterTemplateLibraryTemplates(
    templates.value,
    templateLibraryQuery.value,
    selectedCategory.value
  )
})
const orderedTemplates = computed(() => orderTemplateLibraryTemplates(filteredTemplates.value))

const selectedTemplateSubjectPreview = computed(() => selectedTemplate.value?.subject_template ?? '')
const selectedTemplateBodyPreview = computed(() => selectedTemplate.value?.body_template ?? '')
const selectedTemplateCategories = computed(() => (
  selectedTemplate.value ? deriveTemplateLibraryCategories(selectedTemplate.value) : []
))

const {
  errors: saveFormErrors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: saveFormValues
} = useForm<TemplateFormValues>({
  validationSchema: templateVeeValidationSchema,
  initialValues: templateFormDefaults()
})

const templateVariableDefaultsContext = computed(() => ({
  toText: props.toText,
  ccText: props.ccText,
  bccText: props.bccText,
  subject: props.subject,
  body: props.body
}))

watch(
  selectedTemplate,
  (template, previousTemplate) => {
    const isSameTemplate = Boolean(template && previousTemplate && template.template_id === previousTemplate.template_id)
    variableValues.value = resolveTemplateVariableValues(
      template,
      variableValues.value,
      templateVariableDefaultsContext.value,
      {
        preserveExisting: isSameTemplate
      }
    )
    if (!isSameTemplate) {
      recipientVariableMapping.value = inferRecipientVariableMapping(template?.variables ?? [])
      previewRowsText.value = stringifyTemplateMailMergePreviewRows(buildDefaultPreviewRows(template))
      previewError.value = ''
      previewResult.value = null
    }
    deleteConfirmTemplateId.value = ''
  },
  { immediate: true }
)
watch(
  orderedTemplates,
  (items) => {
    if (!items.length) {
      selectedTemplateId.value = ''
      return
    }
    if (!items.some((template) => template.template_id === selectedTemplateId.value)) {
      selectedTemplateId.value = items[0].template_id
    }
  },
  { immediate: true }
)

function updateVariable(variable: string, value: string): void {
  variableValues.value = {
    ...variableValues.value,
    [variable]: value
  }
}

function templateDiagnosticCount(template: CommunicationTemplate): number {
  return storedTemplateDiagnosticMessages(template).length
}

function templateUpdatedLabel(template: CommunicationTemplate): string {
  return formatTemplateUpdatedLabel(template.updated_at)
}

async function applyTemplate(): Promise<void> {
  const template = selectedTemplate.value
  if (!template) return
  if (selectedTemplateValidationMessage.value) {
    emit('error', selectedTemplateValidationMessage.value)
    return
  }
  if (mergeValidationMessage.value) {
    emit('error', mergeValidationMessage.value)
    return
  }

  try {
    const result = await renderTemplateMutation.mutateAsync({
      template_id: template.template_id,
      variables: variableValues.value
    })
    if (result.rendered.malformed_placeholders.length) {
      emit('error', `Fix malformed template placeholders: ${result.rendered.malformed_placeholders.join(', ')}`)
      return
    }
    if (result.rendered.unresolved_variables.length) {
      emit('error', templateMergeErrorMessage(result.rendered.unresolved_variables))
      return
    }
    emit('apply', {
      subject: result.rendered.subject,
      bodyHtml: result.rendered.body
    })
  } catch (error) {
    emit('error', error instanceof Error ? error.message : 'Template render failed')
  }
}

function openSaveTemplate(mode: 'new' | 'duplicate'): void {
  saveMode.value = mode
  isSaveOpen.value = true
  resetForm({
    values: {
      name: suggestTemplateSaveName(props.subject, selectedTemplate.value?.name ?? '', {
        duplicate: mode === 'duplicate'
      })
    }
  })
}

function closeSaveTemplate(): void {
  saveMode.value = 'new'
  isSaveOpen.value = false
  resetForm({ values: templateFormDefaults() })
}

function clearTemplateLibraryQuery(): void {
  templateLibraryQuery.value = ''
}

function buildDefaultPreviewRows(template: CommunicationTemplate | null): Array<{ row_id: string; variables: Record<string, string> }> {
  if (!template) return []
  const values = resolveTemplateVariableValues(
    template,
    variableValues.value,
    templateVariableDefaultsContext.value,
    { preserveExisting: true }
  )
  if (!Object.keys(values).length) return []
  return [{ row_id: 'row-1', variables: values }]
}

function applyRecipientMapping(): void {
  variableValues.value = applyTemplateRecipientMapping(variableValues.value, recipientVariableMapping.value, {
    toText: props.toText,
    ccText: props.ccText,
    bccText: props.bccText
  })
}

function buildPreviewRowsFromRecipients(): void {
  const rows = buildTemplateRecipientPreviewRows(
    templateVariables.value,
    recipientVariableMapping.value,
    {
      toText: props.toText,
      ccText: props.ccText,
      bccText: props.bccText
    },
    variableValues.value
  )
  if (!rows.length) {
    previewError.value = 'Map a To variable and add at least one To recipient to build preview rows'
    return
  }
  previewRowsText.value = stringifyTemplateMailMergePreviewRows(rows)
  previewError.value = ''
  previewResult.value = null
}

async function previewMailMerge(): Promise<void> {
  const template = selectedTemplate.value
  if (!template) return
  if (selectedTemplateValidationMessage.value) {
    emit('error', selectedTemplateValidationMessage.value)
    return
  }

  previewError.value = ''
  previewResult.value = null
  try {
    const rows = parseTemplateMailMergePreviewRows(previewRowsText.value)
    if (!rows.length) {
      previewError.value = 'Add at least one preview row'
      return
    }
    previewResult.value = await previewMailMergeMutation.mutateAsync({
      template_id: template.template_id,
      rows
    })
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Mail merge preview failed'
    previewError.value = message
    emit('error', message)
  }
}

const saveCurrentTemplate = handleSubmit(async (values) => {
  if (!hasTemplateContent.value) {
    emit('error', 'Add a subject or body before saving a template')
    return
  }
  if (saveValidationMessage.value) {
    emit('error', saveValidationMessage.value)
    return
  }

  try {
    const result = await createTemplateMutation.muta
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/DraftStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/DraftStrip.vue`
- Size bytes / Размер в байтах: `4317`
- Included characters / Включено символов: `4317`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CommunicationDraft } from '../types/communications'

const props = defineProps<{
  drafts: CommunicationDraft[]
  hasMore: boolean
  isLoadingMore: boolean
}>()

const emit = defineEmits<{
  openDraft: [draft: CommunicationDraft]
  deleteDraft: [draftId: string]
  loadMore: []
}>()

const draftScrollRef = ref<HTMLDivElement | null>(null)
const draftVirtualOptions = computed(() => ({
  count: props.drafts.length,
  getScrollElement: () => draftScrollRef.value,
  estimateSize: () => 46,
  overscan: 8
}))
const draftVirtualizer = useVirtualizer(draftVirtualOptions)
const virtualDraftRows = computed(() => draftVirtualizer.value.getVirtualItems())
const draftVirtualTotalSize = computed(() => draftVirtualizer.value.getTotalSize())
</script>

<template>
  <div v-if="drafts.length > 0" class="draft-strip">
    <div class="draft-strip-header">
      <Icon icon="tabler:edit" class="draft-strip-icon" />
      <span class="draft-strip-title">Drafts ({{ drafts.length }})</span>
    </div>
    <div ref="draftScrollRef" class="draft-list" :style="{ maxHeight: '12rem' }">
      <div class="draft-list-track" :style="{ height: `${draftVirtualTotalSize}px` }">
        <div
          v-for="virtualRow in virtualDraftRows"
          :key="String(virtualRow.key)"
          class="draft-item"
          :style="{
            height: `${virtualRow.size}px`,
            transform: `translateY(${virtualRow.start}px)`
          }"
        >
          <div class="draft-info" @click="emit('openDraft', drafts[virtualRow.index])">
            <span class="draft-subject">{{ drafts[virtualRow.index].subject || '(No subject)' }}</span>
            <span class="draft-recipients">{{ drafts[virtualRow.index].to_recipients?.join(', ') || 'No recipients' }}</span>
          </div>
          <Button
            variant="ghost"
            size="sm"
            class="draft-delete-btn"
            @click="emit('deleteDraft', drafts[virtualRow.index].draft_id)"
          >
            <Icon icon="tabler:x" />
          </Button>
        </div>
      </div>
    </div>
    <Button
      v-if="hasMore"
      class="draft-load-more"
      type="button"
      variant="ghost"
      size="sm"
      :disabled="isLoadingMore"
      @click="emit('loadMore')"
    >
      <Icon icon="tabler:chevron-down" />
      <span>{{ isLoadingMore ? 'Loading drafts...' : 'Load more drafts' }}</span>
    </Button>
  </div>
</template>

<style scoped>
.draft-strip {
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: var(--hh-bg-warning-light, #fffbeb);
}

.draft-strip-header {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--hh-text-warning, #d97706);
}

.draft-strip-icon {
  width: 14px;
  height: 14px;
}

.draft-strip-title {
  color: var(--hh-text-warning, #d97706);
}

.draft-list {
  position: relative;
  overflow: auto;
}

.draft-list-track {
  position: relative;
  width: 100%;
}

.draft-item {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  padding: 0.25rem 0.75rem;
  gap: 0.5rem;
  border-top: 1px solid var(--hh-border, #e5e7eb);
}

.draft-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.0625rem;
  cursor: pointer;
  min-width: 0;
}

.draft-subject {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.draft-recipients {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.draft-delete-btn {
  flex-shrink: 0;
}

.draft-load-more {
  width: 100%;
  justify-content: center;
  border-radius: 0;
  border-top: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-warning-light, #fffbeb) 78%, transparent);
  color: var(--hh-text-warning, #d97706);
  font-size: 0.75rem;
  font-weight: 600;
  gap: 0.375rem;
}

.draft-load-more:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}
</style>
```

### `frontend/src/domains/communications/components/HealthStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/HealthStrip.vue`
- Size bytes / Размер в байтах: `2522`
- Included characters / Включено символов: `2522`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { MailboxHealth } from '../types/communications'

const props = defineProps<{
  health: MailboxHealth | null
}>()

function healthToneClass(value: number, max: number): string {
  const ratio = value / Math.max(max, 1)
  if (ratio > 0.8) return 'health-item--danger'
  if (ratio > 0.5) return 'health-item--warning'
  return 'health-item--success'
}
</script>

<template>
  <div v-if="health" class="health-strip">
    <div class="health-item">
      <Icon icon="tabler:mail" class="health-icon" />
      <span class="health-value">{{ health.total_messages }}</span>
      <span class="health-label">Total</span>
    </div>
    <div class="health-item" :class="healthToneClass(health.unread, health.total_messages)">
      <Icon icon="tabler:mail-opened" class="health-icon" />
      <span class="health-value">{{ health.unread }}</span>
      <span class="health-label">Unread</span>
    </div>
    <div class="health-item" :class="healthToneClass(health.needs_action, health.total_messages)">
      <Icon icon="tabler:alert-circle" class="health-icon" />
      <span class="health-value">{{ health.needs_action }}</span>
      <span class="health-label">Action</span>
    </div>
    <div class="health-item" :class="healthToneClass(health.waiting, health.total_messages)">
      <Icon icon="tabler:clock" class="health-icon" />
      <span class="health-value">{{ health.waiting }}</span>
      <span class="health-label">Waiting</span>
    </div>
    <div class="health-item">
      <Icon icon="tabler:star" class="health-icon" />
      <span class="health-value">{{ health.important }}</span>
      <span class="health-label">Important</span>
    </div>
  </div>
</template>

<style scoped>
.health-strip {
  display: flex;
  gap: 1rem;
  padding: 0.5rem 0.75rem;
  background: var(--hh-bg-secondary, #f9fafb);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  overflow-x: auto;
}

.health-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.75rem;
  white-space: nowrap;
}

.health-item--danger {
  color: var(--hh-status-danger-text, #ef4444);
}

.health-item--warning {
  color: var(--hh-status-warning-text, #f59e0b);
}

.health-item--success {
  color: var(--hh-status-success-text, #16a34a);
}

.health-icon {
  width: 14px;
  height: 14px;
}

.health-value {
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.health-label {
  color: var(--hh-text-secondary, #6b7280);
}
</style>
```

### `frontend/src/domains/communications/components/MailCertificateStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MailCertificateStrip.vue`
- Size bytes / Размер в байтах: `10649`
- Included characters / Включено символов: `10646`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useForm } from 'vee-validate'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  certificateFormDefaults,
  certificateFormToCreateRequest,
  certificateVeeValidationSchema,
  type CertificateFormValues
} from '../forms/certificateForm'
import {
  useCreateMailCertificateMutation,
  useExpiringMailCertificatesQuery,
  useMailCertificatesQuery
} from '../queries/useCommunicationsQuery'
import {
  certificateProviderOptions,
  certificateStorageKindOptions,
  certificateTrustStatusOptions,
  certificateTypeOptions,
  type MailCertificate
} from '../types/certificates'

const isOpen = ref(false)
const statusMessage = ref('')
const errorMessage = ref('')
const certificatesQuery = useMailCertificatesQuery()
const expiringQuery = useExpiringMailCertificatesQuery(90)
const createCertificateMutation = useCreateMailCertificateMutation()

const {
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<CertificateFormValues>({
  validationSchema: certificateVeeValidationSchema,
  initialValues: certificateFormDefaults()
})

const certificates = computed(() => certificatesQuery.data.value ?? [])
const expiringCertificates = computed(() => expiringQuery.data.value ?? [])
const isLoading = computed(() => certificatesQuery.isLoading.value || expiringQuery.isLoading.value)
const isSaving = computed(() => createCertificateMutation.isPending.value)
const visibleCertificates = computed(() => certificates.value.slice(0, 3))
const visibleExpiringCertificates = computed(() => expiringCertificates.value.slice(0, 3))

const submitCertificate = handleSubmit(async (values) => {
  errorMessage.value = ''
  statusMessage.value = ''
  try {
    await createCertificateMutation.mutateAsync(certificateFormToCreateRequest(values))
    statusMessage.value = 'Certificate metadata saved'
    resetForm({ values: certificateFormDefaults() })
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : 'Certificate save failed'
  }
})

function toggleOpen(): void {
  isOpen.value = !isOpen.value
}

function updateTextField(field: keyof CertificateFormValues, event: Event): void {
  const input = event.target as HTMLInputElement | HTMLSelectElement
  setFieldValue(field, input.value)
}

function certificateLabel(certificate: MailCertificate): string {
  return `${certificate.owner_name} · ${certificate.trust_status}`
}
</script>

<template>
  <section class="mail-certificate-strip" aria-label="Mail certificates">
    <button class="certificate-toggle" type="button" :aria-expanded="isOpen" @click="toggleOpen">
      <span class="certificate-title">
        <Icon icon="tabler:certificate" />
        Certificates
      </span>
      <span class="certificate-count">
        {{ isLoading ? 'Loading...' : `${certificates.length} stored · ${expiringCertificates.length} expiring` }}
      </span>
    </button>

    <div v-if="isOpen" class="certificate-body">
      <div class="certificate-groups">
        <article class="certificate-group">
          <div class="certificate-heading">Expiring certificates</div>
          <div v-if="isLoading" class="certificate-muted">Loading certificates...</div>
          <div v-else-if="visibleExpiringCertificates.length === 0" class="certificate-muted">No expiry in 90 days</div>
          <div v-else class="certificate-list">
            <span
              v-for="certificate in visibleExpiringCertificates"
              :key="certificate.cert_id"
              class="certificate-chip warning"
            >
              {{ certificate.owner_name }} · {{ certificate.valid_until ?? 'unknown expiry' }}
            </span>
          </div>
        </article>

        <article class="certificate-group">
          <div class="certificate-heading">Stored certificates</div>
          <div v-if="isLoading" class="certificate-muted">Loading certificates...</div>
          <div v-else-if="visibleCertificates.length === 0" class="certificate-muted">No certificate metadata</div>
          <div v-else class="certificate-list">
            <span
              v-for="certificate in visibleCertificates"
              :key="certificate.cert_id"
              class="certificate-chip"
              :class="{ warning: certificate.is_revoked || certificate.trust_status !== 'trusted' }"
            >
              {{ certificateLabel(certificate) }}
            </span>
          </div>
        </article>
      </div>

      <form class="certificate-form" @submit.prevent="submitCertificate">
        <div class="certificate-form-title">Add certificate</div>
        <label>
          <span>Certificate id</span>
          <input :value="formValues.cert_id" type="text" autocomplete="off" @input="updateTextField('cert_id', $event)" />
          <small v-if="errors.cert_id">{{ errors.cert_id }}</small>
        </label>
        <label>
          <span>Owner</span>
          <input :value="formValues.owner_name" type="text" autocomplete="off" @input="updateTextField('owner_name', $event)" />
          <small v-if="errors.owner_name">{{ errors.owner_name }}</small>
        </label>
        <label>
          <span>Issuer</span>
          <input :value="formValues.issuer" type="text" autocomplete="off" @input="updateTextField('issuer', $event)" />
          <small v-if="errors.issuer">{{ errors.issuer }}</small>
        </label>
        <label>
          <span>Fingerprint SHA-256</span>
          <input :value="formValues.fingerprint_sha256" type="text" autocomplete="off" @input="updateTextField('fingerprint_sha256', $event)" />
        </label>
        <label>
          <span>Valid until</span>
          <input :value="formValues.valid_until" type="datetime-local" @input="updateTextField('valid_until', $event)" />
        </label>
        <label>
          <span>Type</span>
          <select :value="formValues.cert_type" @change="updateTextField('cert_type', $event)">
            <option v-for="option in certificateTypeOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Provider</span>
          <select :value="formValues.provider" @change="updateTextField('provider', $event)">
            <option v-for="option in certificateProviderOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Storage</span>
          <select :value="formValues.storage_kind" @change="updateTextField('storage_kind', $event)">
            <option v-for="option in certificateStorageKindOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Storage reference</span>
          <input :value="formValues.storage_ref" type="text" autocomplete="off" placeholder="keychain item, vault ref, token id" @input="updateTextField('storage_ref', $event)" />
        </label>
        <label>
          <span>Trust</span>
          <select :value="formValues.trust_status" @change="updateTextField('trust_status', $event)">
            <option v-for="option in certificateTrustStatusOptions" :key="option" :value="option">{{ option }}</option>
          </select>
        </label>
        <label>
          <span>Usage</span>
          <input :value="formValues.usage" type="text" autocomplete="off" placeholder="signing, encryption" @input="updateTextField('usage', $event)" />
        </label>

        <p v-if="statusMessage" class="certificate-status">{{ statusMessage }}</p>
        <p v-if="errorMessage" class="certificate-error">{{ errorMessage }}</p>
        <Button variant="outline" size="sm" type="submit" :loading="isSaving">
          Save metadata
        </Button>
      </form>
    </div>
  </section>
</template>

<style scoped>
.mail-certificate-strip {
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.certificate-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  border: 0;
  padding: 0.5rem 0.75rem;
  background: transparent;
  color: var(--hh-text-primary, #1f2937);
  cursor: pointer;
}

.certificate-title,
.certificate-count {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 0;
  font-size: 0.75rem;
}

.certificate-title {
  font-weight: 700;
}

.certificate-count {
  color: var(--hh-text-secondary, #6b7280);
}

.certificate-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(18rem, 0.85fr);
  gap: 0.75rem;
  padding: 0 0.75rem 0.75rem;
}

.certificate-groups,
.certificate-group,
.certificate-form {
  display: grid;
  gap: 0.5rem;
  min-width: 0;
}

.certificate-heading,
.certificate-form-title {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.certificate-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.certificate-chip,
.certificate-muted {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.certificate-chip {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  padding: 0.125rem 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
}

.certificate-chip.warning {
  color: var(--hh-text-error, #ef4444);
}

.certificate-form {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-items: end;
}

.certificate-form-title,
.certificate-status,
.certificate-error {
  grid-column: 1 / -1;
}

.certificate-form label {
  display: grid;
  gap: 0.1875rem;
  min-width: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 600;
}

.certificate-form input,
.certificate-form select {
  min-height: 1.875rem;
  border: 1px solid var(--hh-border, #d1d5db);
  border-radius: var(--hh-radius-sm, 0.375rem);
  padding: 0.25rem 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 74%, transparent);
  color: var(--hh-text-primary, #111827);
  font-size: 0.75rem;
}

.certificate-form small,
.certificate-error {
  color: var(--hh-text-error, #ef4444);
  font-size: 0.625rem;
}

.certificate-status {
  color: var(--hh-text-success, #16a34a);
  font-size: 0.6875rem;
}

@media (max-width: 900px) {
  .certificate-body,
  .certificate-form {
    grid-template-columns: 1fr;
  }
}
</style>
```

### `frontend/src/domains/communications/components/MailResourceOverviewStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MailResourceOverviewStrip.vue`
- Size bytes / Размер в байтах: `7228`
- Included characters / Включено символов: `7226`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type {
  CommunicationArchitectureBlocker,
  SenderStats,
  SubscriptionSource
} from '../types/communications'

const props = defineProps<{
  subscriptions: SubscriptionSource[]
  topSenders: SenderStats[]
  blockers: CommunicationArchitectureBlocker[]
  isLoading: boolean
  hasMoreSubscriptions: boolean
  isLoadingMoreSubscriptions: boolean
  hasMoreTopSenders: boolean
  isLoadingMoreTopSenders: boolean
}>()

const emit = defineEmits<{
  loadMoreSubscriptions: []
  loadMoreTopSenders: []
}>()

const subscriptionScrollRef = ref<HTMLDivElement | null>(null)
const subscriptionVirtualizer = useVirtualizer(computed(() => ({
  count: props.subscriptions.length,
  getScrollElement: () => subscriptionScrollRef.value,
  estimateSize: () => 28,
  overscan: 6
})))
const virtualSubscriptionRows = computed(() => subscriptionVirtualizer.value.getVirtualItems())
const subscriptionTotalSize = computed(() => subscriptionVirtualizer.value.getTotalSize())

const topSenderScrollRef = ref<HTMLDivElement | null>(null)
const topSenderVirtualizer = useVirtualizer(computed(() => ({
  count: props.topSenders.length,
  getScrollElement: () => topSenderScrollRef.value,
  estimateSize: () => 28,
  overscan: 6
})))
const virtualTopSenderRows = computed(() => topSenderVirtualizer.value.getVirtualItems())
const topSenderTotalSize = computed(() => topSenderVirtualizer.value.getTotalSize())
</script>

<template>
  <section class="mail-resource-strip" aria-label="Mailbox resources">
    <article class="resource-group">
      <div class="resource-heading">
        <Icon icon="tabler:mail-bolt" class="resource-icon" />
        <span>Newsletters</span>
      </div>
      <div v-if="isLoading && subscriptions.length === 0" class="resource-muted">Loading...</div>
      <div v-else-if="subscriptions.length === 0" class="resource-muted">No sources</div>
      <div v-else ref="subscriptionScrollRef" class="resource-virtual-list">
        <div class="resource-list-track" :style="{ height: `${subscriptionTotalSize}px` }">
          <span
            v-for="virtualRow in virtualSubscriptionRows"
            :key="String(virtualRow.key)"
            class="resource-chip virtual-resource-chip"
            :style="{
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`
            }"
          >
            {{ subscriptions[virtualRow.index]?.sender ?? '' }} · {{ subscriptions[virtualRow.index]?.message_count ?? 0 }}
          </span>
        </div>
      </div>
      <Button
        v-if="hasMoreSubscriptions"
        class="resource-load-more"
        type="button"
        variant="ghost"
        size="sm"
        :disabled="isLoadingMoreSubscriptions"
        @click="emit('loadMoreSubscriptions')"
      >
        <Icon icon="tabler:chevron-down" />
        <span>{{ isLoadingMoreSubscriptions ? 'Loading...' : 'More newsletters' }}</span>
      </Button>
    </article>

    <article class="resource-group">
      <div class="resource-heading">
        <Icon icon="tabler:user-star" class="resource-icon" />
        <span>Top senders</span>
      </div>
      <div v-if="isLoading && topSenders.length === 0" class="resource-muted">Loading...</div>
      <div v-else-if="topSenders.length === 0" class="resource-muted">No senders</div>
      <div v-else ref="topSenderScrollRef" class="resource-virtual-list">
        <div class="resource-list-track" :style="{ height: `${topSenderTotalSize}px` }">
          <span
            v-for="virtualRow in virtualTopSenderRows"
            :key="String(virtualRow.key)"
            class="resource-chip virtual-resource-chip"
            :style="{
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`
            }"
          >
            {{ topSenders[virtualRow.index]?.sender ?? '' }} · {{ topSenders[virtualRow.index]?.message_count ?? 0 }}
          </span>
        </div>
      </div>
      <Button
        v-if="hasMoreTopSenders"
        class="resource-load-more"
        type="button"
        variant="ghost"
        size="sm"
        :disabled="isLoadingMoreTopSenders"
        @click="emit('loadMoreTopSenders')"
      >
        <Icon icon="tabler:chevron-down" />
        <span>{{ isLoadingMoreTopSenders ? 'Loading...' : 'More senders' }}</span>
      </Button>
    </article>

    <article class="resource-group">
      <div class="resource-heading">
        <Icon icon="tabler:road-sign" class="resource-icon" />
        <span>Blockers</span>
      </div>
      <div v-if="isLoading" class="resource-muted">Loading...</div>
      <div v-else-if="blockers.length === 0" class="resource-muted">No blockers</div>
      <div v-else class="resource-list">
        <span v-for="blocker in blockers.slice(0, 2)" :key="`${blocker.section}-${blocker.feature}`" class="resource-chip warning">
          {{ blocker.feature }}
        </span>
      </div>
    </article>
  </section>
</template>

<style scoped>
.mail-resource-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.resource-group {
  min-width: 0;
  display: grid;
  gap: 0.375rem;
}

.resource-heading {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.resource-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--hh-accent, #2563eb);
}

.resource-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  min-width: 0;
}

.resource-virtual-list {
  position: relative;
  min-width: 0;
  min-height: 1.75rem;
  max-height: 5.25rem;
  overflow: auto;
}

.resource-list-track {
  position: relative;
  width: 100%;
}

.resource-chip,
.resource-muted {
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
}

.resource-chip {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
}

.virtual-resource-chip {
  position: absolute;
  top: 0;
  left: 0;
  display: inline-flex;
  align-items: center;
  max-width: calc(100% - 0.25rem);
}

.resource-chip.warning {
  color: var(--hh-text-error, #ef4444);
}

.resource-load-more {
  justify-content: flex-start;
  min-width: 0;
  width: fit-content;
  max-width: 100%;
  color: var(--hh-accent, #2563eb);
  font-size: 0.6875rem;
  gap: 0.25rem;
  padding-inline: 0;
}

.resource-load-more:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

@media (max-width: 900px) {
  .mail-resource-strip {
    grid-template-columns: 1fr;
  }
}
</style>
```

### `frontend/src/domains/communications/components/MessageAiReplyPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageAiReplyPanel.vue`
- Size bytes / Размер в байтах: `6694`
- Included characters / Включено символов: `6694`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { AiReplyResponse, CommunicationMessageInsight } from '../types/communications'
import { useGenerateAiReplyVariantsMutation } from '../queries/useCommunicationsQuery'

const props = defineProps<{
  messageId: string | null
  insight: CommunicationMessageInsight | null
}>()

const emit = defineEmits<{
  generateAiReply: [payload: { tone: string; language: string }]
  applyAiReply: [payload: AiReplyResponse]
}>()

const selectedAiReplyTone = ref('business')
const selectedAiReplyLanguage = ref('en')

const aiReplyToneOptions = ['formal', 'business', 'friendly', 'short', 'detailed']
const aiReplyLanguageOptions = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Russian' }
]

const aiReply = computed(() => props.insight?.aiReply ?? null)
const generateAiReplyVariantsMutation = useGenerateAiReplyVariantsMutation()
const replyVariants = ref<AiReplyResponse[]>([])
const variantsPending = computed(() => generateAiReplyVariantsMutation.isPending.value)
const variantsError = computed(() => generateAiReplyVariantsMutation.error.value?.message ?? '')

async function generateVariants() {
  if (!props.messageId) return
  try {
    const result = await generateAiReplyVariantsMutation.mutateAsync({
      messageId: props.messageId,
      languages: aiReplyLanguageOptions.map((language) => language.value),
      tones: aiReplyToneOptions
    })
    replyVariants.value = result.variants
  } catch {
    replyVariants.value = []
  }
}
</script>

<template>
  <section class="ai-reply-review">
    <div class="ai-reply-header">
      <div class="ai-reply-title">
        <Icon icon="tabler:sparkles" class="intel-icon" />
        <span class="intel-title">AI Reply Review</span>
      </div>
      <div class="ai-reply-controls">
        <label>
          <span>Tone</span>
          <select v-model="selectedAiReplyTone">
            <option v-for="tone in aiReplyToneOptions" :key="tone" :value="tone">
              {{ tone }}
            </option>
          </select>
        </label>
        <label>
          <span>Language</span>
          <select v-model="selectedAiReplyLanguage">
            <option v-for="language in aiReplyLanguageOptions" :key="language.value" :value="language.value">
              {{ language.label }}
            </option>
          </select>
        </label>
        <Button
          variant="outline"
          size="sm"
          :disabled="!messageId"
          @click="emit('generateAiReply', { tone: selectedAiReplyTone, language: selectedAiReplyLanguage })"
        >
          <Icon icon="tabler:sparkles" /> Generate
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!messageId || variantsPending"
          @click="generateVariants"
        >
          <Icon icon="tabler:versions" /> Variants
        </Button>
      </div>
    </div>
    <p v-if="variantsError" class="ai-reply-error">{{ variantsError }}</p>
    <div v-if="aiReply" class="ai-reply-card">
      <div class="ai-reply-meta">
        <span>{{ aiReply.tone || selectedAiReplyTone }}</span>
        <span>{{ aiReply.language || selectedAiReplyLanguage }}</span>
        <span v-if="aiReply.generated === false">{{ aiReply.reason || 'Not generated' }}</span>
      </div>
      <strong>{{ aiReply.subject || 'Reply draft' }}</strong>
      <p>{{ aiReply.body || aiReply.reason || 'No reply body returned.' }}</p>
      <Button
        variant="ghost"
        size="sm"
        :disabled="!aiReply.body"
        @click="emit('applyAiReply', aiReply)"
      >
        <Icon icon="tabler:pencil" /> Apply to compose
      </Button>
    </div>
    <div v-if="replyVariants.length" class="ai-reply-variants">
      <article
        v-for="(variant, index) in replyVariants"
        :key="`${variant.language}-${variant.tone}-${index}`"
        class="ai-reply-card"
      >
        <div class="ai-reply-meta">
          <span>{{ variant.tone || 'tone' }}</span>
          <span>{{ variant.language || 'language' }}</span>
        </div>
        <strong>{{ variant.subject || 'Reply variant' }}</strong>
        <p>{{ variant.body || 'No reply body returned.' }}</p>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!variant.body"
          @click="emit('applyAiReply', variant)"
        >
          <Icon icon="tabler:pencil" /> Apply
        </Button>
      </article>
    </div>
  </section>
</template>

<style scoped>
.ai-reply-review {
  display: grid;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border-info, #bae6fd);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-info, #f0f9ff) 78%, transparent);
}

.ai-reply-header,
.ai-reply-title,
.ai-reply-controls {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.ai-reply-header {
  justify-content: space-between;
}

.intel-icon {
  width: 16px;
  height: 16px;
  color: var(--hh-accent, #3b82f6);
}

.intel-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary, #1f2937);
}

.ai-reply-controls label {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.ai-reply-controls select {
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
}

.ai-reply-card {
  display: grid;
  gap: 0.375rem;
  min-width: 0;
  padding: 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
}

.ai-reply-variants {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 0.5rem;
}

.ai-reply-card strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.ai-reply-card p {
  margin: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
  line-height: 1.4;
  white-space: pre-wrap;
}

.ai-reply-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.ai-reply-meta span {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  color: var(--hh-text-secondary, #6b7280);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
  font-size: 0.6875rem;
}

.ai-reply-error {
  margin: 0;
  color: var(--hh-danger, #b91c1c);
  font-size: 0.75rem;
}
</style>
```

### `frontend/src/domains/communications/components/MessageAttachmentsTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageAttachmentsTab.vue`
- Size bytes / Размер в байтах: `19968`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { FlexRender, getCoreRowModel, useVueTable } from '@tanstack/vue-table'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationMessageDetailResponse } from '../types/communications'
import { attachmentIcon } from '../stores/communications'
import {
  useAttachmentArchiveInspectionQuery,
  useAttachmentPreviewQuery,
  useTranslateAttachmentMutation
} from '../queries/useCommunicationsQuery'
import type { AttachmentTranslationResponse } from '../types/attachments'
import {
  attachmentTableColumns,
  attachmentTableRowId,
  formatAttachmentSize,
  isInspectableArchiveAttachment,
  isPreviewableAttachment,
  isPreviewablePdfAttachment,
  isPreviewableImageAttachment,
  scanStatusClass
} from './attachmentTable'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

const attachments = computed(() => props.detail?.attachments ?? [])
const selectedArchiveAttachmentId = ref<string | null>(null)
const selectedArchiveAttachment = computed(() =>
  attachments.value.find((attachment) => attachment.attachment_id === selectedArchiveAttachmentId.value) ?? null
)
const selectedPreviewAttachmentId = ref<string | null>(null)
const selectedPreviewAttachment = computed(() =>
  attachments.value.find((attachment) => attachment.attachment_id === selectedPreviewAttachmentId.value) ?? null
)
const attachmentTranslationTarget = ref('en')
const attachmentTranslationResult = ref<AttachmentTranslationResponse | null>(null)
const attachmentTranslationError = ref('')
const translateAttachmentMutation = useTranslateAttachmentMutation()
const isAttachmentTranslationPending = computed(() => translateAttachmentMutation.isPending.value)
const {
  data: archiveInspectionData,
  error: archiveInspectionError,
  isFetching: isArchiveInspectionFetching
} = useAttachmentArchiveInspectionQuery(
  () => selectedArchiveAttachmentId.value,
  () => Boolean(selectedArchiveAttachmentId.value)
)
const archiveInspection = computed(() => archiveInspectionData.value)
const archiveInspectionErrorMessage = computed(() => {
  if (!archiveInspectionError.value) return ''
  return archiveInspectionError.value instanceof Error
    ? archiveInspectionError.value.message
    : 'Archive inspection failed'
})
const {
  data: attachmentPreviewData,
  error: attachmentPreviewError,
  isFetching: isAttachmentPreviewFetching
} = useAttachmentPreviewQuery(
  () => selectedPreviewAttachmentId.value,
  () => Boolean(selectedPreviewAttachmentId.value)
)
const attachmentPreview = computed(() => attachmentPreviewData.value)
const attachmentPreviewErrorMessage = computed(() => {
  if (!attachmentPreviewError.value) return ''
  return attachmentPreviewError.value instanceof Error
    ? attachmentPreviewError.value.message
    : 'Attachment preview failed'
})

const table = useVueTable({
  get data() {
    return attachments.value
  },
  columns: attachmentTableColumns,
  getCoreRowModel: getCoreRowModel(),
  getRowId: attachmentTableRowId
})

watch(attachments, (items) => {
  if (
    selectedArchiveAttachmentId.value
    && !items.some((attachment) => attachment.attachment_id === selectedArchiveAttachmentId.value)
  ) {
    selectedArchiveAttachmentId.value = null
  }
  if (
    selectedPreviewAttachmentId.value
    && !items.some((attachment) => attachment.attachment_id === selectedPreviewAttachmentId.value)
  ) {
    selectedPreviewAttachmentId.value = null
    attachmentTranslationResult.value = null
    attachmentTranslationError.value = ''
  }
})

function inspectArchive(attachmentId: string) {
  selectedArchiveAttachmentId.value = attachmentId
}

function showAttachmentPreview(attachmentId: string) {
  selectedPreviewAttachmentId.value = attachmentId
  attachmentTranslationResult.value = null
  attachmentTranslationError.value = ''
}

async function translateSelectedAttachment() {
  const attachmentId = selectedPreviewAttachmentId.value
  const preview = attachmentPreview.value
  if (!attachmentId || !preview?.text.trim()) return

  attachmentTranslationError.value = ''
  try {
    attachmentTranslationResult.value = await translateAttachmentMutation.mutateAsync({
      attachmentId,
      request: {
        target_language: attachmentTranslationTarget.value,
        source_text: preview.text
      }
    })
  } catch (e) {
    attachmentTranslationError.value = e instanceof Error ? e.message : 'Attachment translation failed'
  }
}
</script>

<template>
  <div class="attachments-tab">
    <div v-if="attachments.length === 0" class="no-data">No attachments</div>
    <div v-else class="attachment-table-shell">
      <table class="attachment-table">
        <thead>
          <tr v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
            <th v-for="header in headerGroup.headers" :key="header.id">
              <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in table.getRowModel().rows" :key="row.id">
            <td
              v-for="cell in row.getVisibleCells()"
              :key="cell.id"
              :class="`attachment-cell attachment-cell--${cell.column.id}`"
            >
              <div v-if="cell.column.id === 'filename'" class="att-file">
                <Icon :icon="attachmentIcon(row.original.content_type)" class="att-icon" />
                <span class="att-filename">{{ row.original.filename || 'Unnamed' }}</span>
                <div class="att-actions">
                  <button
                    v-if="isPreviewableAttachment(row.original)"
                    class="att-inspect"
                    type="button"
                    :disabled="isAttachmentPreviewFetching && selectedPreviewAttachmentId === row.original.attachment_id"
                    @click="showAttachmentPreview(row.original.attachment_id)"
                  >
                    {{
                      isPreviewableImageAttachment(row.original)
                        ? 'Preview image'
                        : isPreviewablePdfAttachment(row.original)
                          ? 'Preview PDF'
                          : 'Preview'
                    }}
                  </button>
                  <button
                    v-if="isInspectableArchiveAttachment(row.original)"
                    class="att-inspect"
                    type="button"
                    :disabled="isArchiveInspectionFetching && selectedArchiveAttachmentId === row.original.attachment_id"
                    @click="inspectArchive(row.original.attachment_id)"
                  >
                    Inspect archive
                  </button>
                </div>
              </div>
              <span v-else-if="cell.column.id === 'size'">
                {{ formatAttachmentSize(row.original.size_bytes) }}
              </span>
              <span
                v-else-if="cell.column.id === 'scan_status'"
                class="att-scan"
                :class="scanStatusClass(row.original.scan_status)"
              >
                {{ row.original.scan_status }}
              </span>
              <span v-else>{{ cell.getValue() }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <section
      v-if="selectedPreviewAttachment"
      class="attachment-preview-panel"
      aria-label="Attachment preview"
    >
      <div class="attachment-preview-header">
        <div>
          <h4>Attachment preview</h4>
          <p>{{ selectedPreviewAttachment.filename || 'Unnamed attachment' }}</p>
        </div>
        <span class="att-scan" :class="scanStatusClass(selectedPreviewAttachment.scan_status)">
          {{ selectedPreviewAttachment.scan_status }}
        </span>
      </div>
      <p v-if="isAttachmentPreviewFetching" class="attachment-preview-muted">Loading safe attachment preview...</p>
      <p v-else-if="attachmentPreviewErrorMessage" class="attachment-preview-error">
        {{ attachmentPreviewErrorMessage }}
      </p>
      <div v-else-if="attachmentPreview" class="attachment-preview-report">
        <div class="attachment-preview-stats">
          <span>{{ formatAttachmentSize(attachmentPreview.byte_count) }}</span>
          <span v-if="attachmentPreview.truncated">
            Truncated to {{ formatAttachmentSize(attachmentPreview.max_preview_bytes) }}
          </span>
          <span v-if="attachmentPreview.preview_kind === 'image'">Image preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'audio'">Audio preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'video'">Video preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'pdf'">PDF preview</span>
          <label v-if="attachmentPreview.preview_kind === 'text'" class="attachment-translation-target">
            <span>Translate</span>
            <select v-model="attachmentTranslationTarget">
              <option value="en">EN</option>
              <option value="ru">RU</option>
              <option value="es">ES</option>
            </select>
          </label>
          <button
            v-if="attachmentPreview.preview_kind === 'text'"
            class="att-inspect"
            type="button"
            :disabled="isAttachmentTranslationPending || !attachmentPreview.text.trim()"
            @click="translateSelectedAttachment"
          >
            {{ isAttachmentTranslationPending ? 'Translating' : 'Translate preview' }}
          </button>
        </div>
        <img
          v-if="attachmentPreview.preview_kind === 'image' && attachmentPreview.data_url"
          class="attachment-preview-image"
          :src="attachmentPreview.data_url"
          :alt="selectedPreviewAttachment.filename || 'Attachment image preview'"
        >
        <audio
          v-else-if="attachmentPreview.preview_kind === 'audio' && attachmentPreview.data_url"
          class="attachment-preview-media"
          controls
          preload="metadata"
          :src="attachmentPreview.data_url"
        />
        <video
          v-else-if="attachmentPreview.preview_kind === 'video' && attachmentPreview.data_url"
          class="attachment-preview-media"
          controls
          preload="metadata"
          :src="attachmentPreview.data_url"
        />
        <iframe
          v-else-if="attachmentPreview.preview_kind === 'pdf' && attachmentPreview.data_url"
          class="attachment-preview-document"
          :src="attachmentPreview.data_url"
          :title="selectedPreviewAttachment.filename || 'Attachment PDF preview'"
        />
        <pre v-else class="attachment-preview-text">{{ attachmentPreview.text }}</pre>
        <section
          v-if="attachmentTranslationResult || attachmentTranslationError"
          class="attachment-translation-panel"
          aria-label="Attachment translation"
        >
          <div class="attachment-translation-header">
            <h4>Attachment translation</h4>
            <span v-if="attachmentTranslationResult">
              {{ attachmentTranslationResult.translated ? 'Translated' : 'Fallback' }}
            </span>
          </div>
          <p v-if="attachmentTranslationError" class="attachment-preview-error">
            {{ attachmentTranslationError }}
          </p>
          <p v-else-if="attachmentTranslationResult?.text" class="attachment-translation-text">
            {{ attachmentTranslationResult.text }}
          </p>
          <p v-else class="attachment-preview-muted">
            {{ attachmentTranslationResult?.reason ?? 'Translation unavailable' }}
          </p>
        </section>
      </div>
    </section>
    <section
      v-if="selectedArchiveAttachment"
      class="archive-inspection-panel"
      aria-label="Archive inspection"
    >
      <div clas
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

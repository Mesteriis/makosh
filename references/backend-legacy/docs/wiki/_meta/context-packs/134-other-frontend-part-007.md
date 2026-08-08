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

- Chunk ID / ID чанка: `134-other-frontend-part-007`
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

### `frontend/src/domains/communications/components/MessageBodyTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageBodyTab.vue`
- Size bytes / Размер в байтах: `16443`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import {
  remoteImageUrlsFromHtml,
  renderMessageBody,
  rewriteRemoteImageSources
} from '../../../shared/sanitize/emailHtml'
import { loadFrontendConfig } from '../../../platform/config/env'
import BilingualReplyPanel from './BilingualReplyPanel.vue'
import MessageAiReplyPanel from './MessageAiReplyPanel.vue'
import MessageLocalIntelligencePanel from './MessageLocalIntelligencePanel.vue'
import MessageTrustReviewPanel from './MessageTrustReviewPanel.vue'
import type { AiReplyResponse, CommunicationMessageDetailResponse, CommunicationMessageInsight } from '../types/communications'
import type { BilingualReplyFlowResponse } from '../types/bilingualReplyFlow'
import {
  aiSummaryContractFromMetadata,
  communicationExtractionSectionsFromInsight,
  communicationKnowledgeSectionsFromSummaryContract
} from '../helpers/communicationPageModels'

const frontendConfig = loadFrontendConfig()

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
  insight: CommunicationMessageInsight | null
}>()

const emit = defineEmits<{
  analyze: []
  reply: []
  createTask: []
  createNote: []
  translate: []
  generateAiReply: [payload: { tone: string; language: string }]
  applyAiReply: [payload: AiReplyResponse]
  reviewSecurity: []
  reviewRecipients: []
  sendBilingualReply: [payload: BilingualReplyFlowResponse]
}>()

const showOriginalFrame = ref(false)
const isBilingualReplyOpen = ref(false)
const shouldLoadRemoteImages = ref(false)

const message = computed(() => props.detail?.message ?? null)
const attachments = computed(() => props.detail?.attachments ?? [])
const messageId = computed(() => message.value?.message_id ?? null)
const summaryContract = computed(() =>
  message.value ? aiSummaryContractFromMetadata(message.value.message_metadata) : null
)
const summarySections = computed(() => {
  const contract = summaryContract.value
  if (!contract) return []
  return [
    { title: 'Key points', items: contract.key_points },
    { title: 'Action items', items: contract.action_items },
    { title: 'Risks', items: contract.risks },
    { title: 'Deadlines', items: contract.deadlines }
  ].filter((section) => section.items.length > 0)
})
const extractionSections = computed(() => communicationExtractionSectionsFromInsight(props.insight))
const knowledgeSections = computed(() => communicationKnowledgeSectionsFromSummaryContract(summaryContract.value))

const bodyText = computed(() => {
  if (!message.value) return ''
  return message.value.body_text ?? ''
})

const renderedBody = computed(() =>
  renderMessageBody({
    bodyHtml: message.value?.body_html,
    bodyText: bodyText.value
  })
)

const isHtmlEmail = computed(() => renderedBody.value.kind === 'html')
const remoteImageUrls = computed(() => {
  const bodyHtml = message.value?.body_html
  return bodyHtml ? remoteImageUrlsFromHtml(bodyHtml) : []
})
const hasRemoteImages = computed(() => remoteImageUrls.value.length > 0)
const displayHtml = computed(() => {
  if (!isHtmlEmail.value) return ''
  if (!hasRemoteImages.value) return renderedBody.value.html
  return rewriteRemoteImageSources(renderedBody.value.html, (url) =>
    shouldLoadRemoteImages.value && messageId.value ? remoteImageProxyUrl(messageId.value, url) : null
  )
})

const originalSrcdoc = computed(() => {
  if (!isHtmlEmail.value) return ''
  const safeHtml = displayHtml.value
  return `<!DOCTYPE html><html><head><base target="_blank"><meta charset="utf-8"><style>
    body { font-family: Arial, Helvetica, sans-serif; color: #1f2933; background: #fff; padding: 1rem; margin: 0; line-height: 1.5; }
    img { max-width: 100%; height: auto; }
    a { color: #2563eb; }
  </style></head><body>${safeHtml}</body></html>`
})

const importanceLabel = computed(() => {
  const score = props.insight?.explain?.reasons?.[0]
  return score ?? null
})

function remoteImageProxyUrl(currentMessageId: string, imageUrl: string): string {
  const base = frontendConfig.apiBaseUrl.replace(/\/+$/, '')
  return `${base}/api/v1/communications/messages/${encodeURIComponent(currentMessageId)}/remote-image?url=${encodeURIComponent(imageUrl)}`
}
</script>

<template>
  <div class="message-body-tab">
    <!-- Render original HTML in sandboxed iframe -->
    <div v-if="isHtmlEmail" class="html-frame-container">
      <div v-if="hasRemoteImages" class="remote-image-notice">
        <div>
          <strong>Remote images blocked</strong>
          <span>{{ remoteImageUrls.length }} external image{{ remoteImageUrls.length === 1 ? '' : 's' }}</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          @click="shouldLoadRemoteImages = !shouldLoadRemoteImages"
        >
          <Icon :icon="shouldLoadRemoteImages ? 'tabler:eye-off' : 'tabler:photo-down'" />
          {{ shouldLoadRemoteImages ? 'Block' : 'Load via proxy' }}
        </Button>
      </div>
      <div v-if="showOriginalFrame" class="original-frame-wrapper">
        <iframe
          class="original-iframe"
          sandbox="allow-popups allow-popups-to-escape-sandbox"
          :srcdoc="originalSrcdoc"
          title="Message body"
        />
        <Button variant="ghost" size="sm" class="toggle-view-btn" @click="showOriginalFrame = false">
          <Icon icon="tabler:code" /> Rendered
        </Button>
      </div>
      <div v-else class="shadow-frame-wrapper">
        <div class="mail-html-body" v-html="displayHtml" />
        <Button variant="ghost" size="sm" class="toggle-view-btn" @click="showOriginalFrame = true">
          <Icon icon="tabler:file-code" /> Original HTML
        </Button>
      </div>
    </div>
    <!-- Plain text fallback -->
    <div v-else class="plain-text-body">
      <pre>{{ bodyText }}</pre>
    </div>

    <!-- Message Intelligence -->
    <div v-if="insight" class="intelligence-card">
      <div class="intel-header">
        <Icon icon="tabler:bulb" class="intel-icon" />
        <span class="intel-title">Message Intelligence</span>
      </div>
      <div v-if="insight.explain?.reasons?.length" class="intel-row">
        <span class="intel-label">Summary:</span>
        <span class="intel-value">{{ insight.explain.reasons.join(', ') }}</span>
      </div>
      <div v-if="insight.language" class="intel-row">
        <span class="intel-label">Language:</span>
        <span class="intel-value">{{ insight.language.language }} ({{ (insight.language.confidence * 100).toFixed(0) }}%)</span>
      </div>
      <div v-if="insight.signature?.has_signature" class="intel-row">
        <span class="intel-label">Signature:</span>
        <span class="intel-value">{{ insight.signature.signature_type }}</span>
      </div>
    </div>

    <MessageAiReplyPanel
      :message-id="messageId"
      :insight="insight"
      @generate-ai-reply="emit('generateAiReply', $event)"
      @apply-ai-reply="emit('applyAiReply', $event)"
    />

    <MessageLocalIntelligencePanel
      :message-id="messageId"
      :insight="insight"
    />

    <MessageTrustReviewPanel
      :message-id="messageId"
      :insight="insight"
      @review-security="emit('reviewSecurity')"
      @review-recipients="emit('reviewRecipients')"
    />

    <section v-if="summarySections.length" class="ai-summary-contract">
      <div class="ai-summary-header">
        <Icon icon="tabler:sparkles" class="intel-icon" />
        <span class="intel-title">AI Summary Review</span>
      </div>
      <div class="ai-summary-grid">
        <div
          v-for="section in summarySections"
          :key="section.title"
          class="ai-summary-section"
        >
          <h4>{{ section.title }}</h4>
          <ul>
            <li v-for="item in section.items" :key="item">{{ item }}</li>
          </ul>
        </div>
      </div>
    </section>

    <section v-if="knowledgeSections.length" class="knowledge-review">
      <div class="knowledge-review-header">
        <Icon icon="tabler:affiliate" class="intel-icon" />
        <span class="intel-title">Knowledge Review</span>
      </div>
      <div class="knowledge-review-grid">
        <article
          v-for="section in knowledgeSections"
          :key="section.kind"
          class="knowledge-review-section"
        >
          <h4>{{ section.title }}</h4>
          <div class="knowledge-review-items">
            <div
              v-for="(item, index) in section.items"
              :key="`${section.kind}-${index}-${item.title}`"
              class="knowledge-review-item"
            >
              <strong>{{ item.title }}</strong>
              <p v-if="item.evidence">{{ item.evidence }}</p>
            </div>
          </div>
        </article>
      </div>
    </section>

    <section v-if="extractionSections.length" class="extraction-review">
      <div class="extraction-review-header">
        <Icon icon="tabler:list-check" class="intel-icon" />
        <span class="intel-title">Extraction Review</span>
      </div>
      <div class="extraction-review-grid">
        <article
          v-for="section in extractionSections"
          :key="section.kind"
          class="extraction-review-section"
        >
          <h4>{{ section.title }}</h4>
          <div class="extraction-review-items">
            <div
              v-for="(item, index) in section.items"
              :key="`${section.kind}-${index}-${item.title}`"
              class="extraction-review-item"
            >
              <strong>{{ item.title }}</strong>
              <div v-if="item.meta.length" class="extraction-review-meta">
                <span v-for="meta in item.meta" :key="meta">{{ meta }}</span>
              </div>
              <p>{{ item.body }}</p>
            </div>
          </div>
        </article>
      </div>
    </section>

    <!-- Workflow action buttons -->
    <div class="workflow-actions">
      <Button variant="outline" size="sm" @click="emit('reply')">
        <Icon icon="tabler:arrow-back-up" /> Reply
      </Button>
      <Button variant="outline" size="sm" @click="emit('createTask')">
        <Icon icon="tabler:checkbox" /> Task
      </Button>
      <Button variant="outline" size="sm" @click="emit('createNote')">
        <Icon icon="tabler:notes" /> Note
      </Button>
      <Button variant="outline" size="sm" @click="emit('translate')">
        <Icon icon="tabler:language" /> Translate
      </Button>
      <Button variant="outline" size="sm" @click="emit('generateAiReply', { tone: 'business', language: 'en' })">
        <Icon icon="tabler:sparkles" /> AI Reply
      </Button>
      <Button variant="outline" size="sm" @click="emit('reviewSecurity')">
        <Icon icon="tabler:shield-search" /> Security
      </Button>
      <Button variant="outline" size="sm" @click="emit('reviewRecipients')">
        <Icon icon="tabler:users-plus" /> Smart CC
      </Button>
      <Button variant="outline" size="sm" @click="isBilingualReplyOpen = !isBilingualReplyOpen">
        <Icon icon="tabler:language-hiragana" /> Bilingual
      </Button>
      <Button variant="outline" size="sm" @click="emit('analyze')">
        <Icon icon="tabler:sparkles" /> Analyze
      </Button>
    </div>

    <BilingualReplyPanel
      v-if="isBilingualReplyOpen && messageId"
      :message-id="messageId"
      @send-bilingual-reply="emit('sendBilingualReply', $event)"
    />
  </div>
</template>

<style scoped>
.message-body-tab {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: 100%;
}

.html-frame-container {
  flex: 1;
  min-height: 300px;
  position: relative;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  overflow: hidden;
}

.original-frame-wrapper,
.shadow-frame-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
}

.original-iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.mail-html-body {
  padding: 1rem;
  min-height: 300px;
  color:
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/MessageHeadersTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageHeadersTab.vue`
- Size bytes / Размер в байтах: `2483`
- Included characters / Включено символов: `2483`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import type { CommunicationMessageDetailResponse } from '../types/communications'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

const message = props.detail?.message ?? null
</script>

<template>
  <div class="headers-tab">
    <table v-if="message" class="headers-table">
      <tbody>
        <tr>
          <td class="header-label">From</td>
          <td class="header-value">{{ message.sender }}</td>
        </tr>
        <tr>
          <td class="header-label">To</td>
          <td class="header-value">{{ message.recipients?.join(', ') || '-' }}</td>
        </tr>
        <tr>
          <td class="header-label">Subject</td>
          <td class="header-value">{{ message.subject }}</td>
        </tr>
        <tr>
          <td class="header-label">Date</td>
          <td class="header-value">{{ message.projected_at || message.occurred_at || '-' }}</td>
        </tr>
        <tr>
          <td class="header-label">Channel</td>
          <td class="header-value">{{ message.channel_kind }}</td>
        </tr>
        <tr>
          <td class="header-label">Message ID</td>
          <td class="header-value monospace">{{ message.message_id }}</td>
        </tr>
        <tr>
          <td class="header-label">Account</td>
          <td class="header-value">{{ message.account_id }}</td>
        </tr>
        <tr>
          <td class="header-label">State</td>
          <td class="header-value">{{ message.workflow_state }} / {{ message.local_state }}</td>
        </tr>
        <tr>
          <td class="header-label">Importance</td>
          <td class="header-value">{{ message.importance_score ?? 'N/A' }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else class="no-data">No message selected</div>
  </div>
</template>

<style scoped>
.headers-tab {
  padding: 1rem;
}

.headers-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8125rem;
}

.headers-table td {
  padding: 0.375rem 0.5rem;
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  vertical-align: top;
}

.header-label {
  width: 120px;
  font-weight: 600;
  color: var(--hh-text-secondary, #6b7280);
  white-space: nowrap;
}

.header-value {
  color: var(--hh-text-primary, #1f2937);
  word-break: break-all;
}

.monospace {
  font-family: monospace;
  font-size: 0.75rem;
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
```

### `frontend/src/domains/communications/components/MessageLocalIntelligencePanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageLocalIntelligencePanel.vue`
- Size bytes / Размер в байтах: `4424`
- Included characters / Включено символов: `4423`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  useDetectMessageLanguageMutation,
  useExplainMessageMutation
} from '../queries/useCommunicationsQuery'
import type {
  LanguageDetection,
  CommunicationMessageInsight,
  MessageExplainResponse
} from '../types/communications'

const props = defineProps<{
  messageId: string | null
  insight: CommunicationMessageInsight | null
}>()

const explainMutation = useExplainMessageMutation()
const languageMutation = useDetectMessageLanguageMutation()
const explainResult = ref<MessageExplainResponse | null>(null)
const languageResult = ref<LanguageDetection | null>(null)
const errorMessage = ref('')

const currentExplain = computed(() => explainResult.value ?? props.insight?.explain ?? null)
const currentLanguage = computed(() => languageResult.value ?? props.insight?.language ?? null)
const isRunning = computed(() => explainMutation.isPending.value || languageMutation.isPending.value)

watch(
  () => props.messageId,
  () => {
    explainResult.value = null
    languageResult.value = null
    errorMessage.value = ''
  }
)

async function explainMessage(): Promise<void> {
  if (!props.messageId) return
  errorMessage.value = ''
  try {
    explainResult.value = await explainMutation.mutateAsync(props.messageId)
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : 'Importance explanation failed'
  }
}

async function detectLanguage(): Promise<void> {
  if (!props.messageId) return
  errorMessage.value = ''
  try {
    languageResult.value = await languageMutation.mutateAsync(props.messageId)
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : 'Language detection failed'
  }
}
</script>

<template>
  <section class="local-intelligence-panel">
    <div class="local-intelligence-header">
      <div class="local-intelligence-title">
        <Icon icon="tabler:brain" class="intel-icon" />
        <span>Importance & Language</span>
      </div>
      <div class="local-intelligence-actions">
        <Button variant="outline" size="sm" :disabled="!messageId" :loading="explainMutation.isPending.value" @click="explainMessage">
          <Icon icon="tabler:info-circle" /> Why this matters
        </Button>
        <Button variant="outline" size="sm" :disabled="!messageId" :loading="languageMutation.isPending.value" @click="detectLanguage">
          <Icon icon="tabler:language" /> Detect language
        </Button>
      </div>
    </div>

    <div v-if="currentExplain" class="local-intelligence-card">
      <strong>Importance</strong>
      <ul>
        <li v-for="reason in currentExplain.reasons" :key="reason">{{ reason }}</li>
      </ul>
    </div>

    <div v-if="currentLanguage" class="local-intelligence-card">
      <strong>Language</strong>
      <span>{{ currentLanguage.language }} · {{ (currentLanguage.confidence * 100).toFixed(0) }}%</span>
    </div>

    <p v-if="!currentExplain && !currentLanguage && !isRunning" class="local-intelligence-empty">
      No local intelligence review has been run for this message.
    </p>
    <p v-if="errorMessage" class="local-intelligence-error">{{ errorMessage }}</p>
  </section>
</template>

<style scoped>
.local-intelligence-panel {
  display: grid;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.local-intelligence-header,
.local-intelligence-title,
.local-intelligence-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.local-intelligence-header {
  justify-content: space-between;
}

.local-intelligence-title {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
  font-weight: 700;
}

.intel-icon {
  width: 16px;
  height: 16px;
  color: var(--hh-accent, #2563eb);
}

.local-intelligence-card {
  display: grid;
  gap: 0.25rem;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
}

.local-intelligence-card ul {
  margin: 0;
  padding-left: 1rem;
}

.local-intelligence-empty {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.local-intelligence-error {
  color: var(--hh-text-error, #ef4444);
  font-size: 0.75rem;
}
</style>
```

### `frontend/src/domains/communications/components/MessageRelatedTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageRelatedTab.vue`
- Size bytes / Размер в байтах: `7338`
- Included characters / Включено символов: `7338`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CommunicationMessageDetailResponse, MessageExportFormat } from '../types/communications'
import {
  communicationMessageLabelsFromMetadata,
  communicationMessageSnoozeUntilFromMetadata
} from '../helpers/communicationPageModels'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

const emit = defineEmits<{
  togglePin: []
  toggleImportant: []
  mute: []
  replyAll: []
  forwardMessage: []
  redirectMessage: [recipientsText: string]
  exportMessage: [format: MessageExportFormat]
  addLabel: [label: string]
  removeLabel: [label: string]
  markMessageRead: []
  markMessageUnread: []
  deleteFromProvider: []
  snoozeMessage: [until: string]
}>()

const redirectRecipientsText = ref('')
const exportFormats: { format: MessageExportFormat; label: string }[] = [
  { format: 'md', label: 'Markdown' },
  { format: 'eml', label: 'EML' },
  { format: 'json', label: 'JSON' }
]

const quickLabels = ['Follow up', 'Finance', 'Legal']
const labels = computed(() =>
  props.detail ? communicationMessageLabelsFromMetadata(props.detail.message.message_metadata) : []
)
const snoozeUntil = computed(() =>
  props.detail ? communicationMessageSnoozeUntilFromMetadata(props.detail.message.message_metadata) : null
)

function snoozePreset(days: number): string {
  const date = new Date()
  date.setDate(date.getDate() + days)
  date.setHours(9, 0, 0, 0)
  return date.toISOString()
}
</script>

<template>
  <div class="related-tab">
    <div v-if="!detail" class="no-data">No message selected</div>
    <div v-else class="related-actions">
      <div class="actions-group">
        <h4 class="group-title">Read / Delete</h4>
        <Button variant="outline" size="sm" @click="emit('markMessageRead')">
          <Icon icon="tabler:mail-opened" /> Mark as read
        </Button>
        <Button variant="outline" size="sm" @click="emit('markMessageUnread')">
          <Icon icon="tabler:mail" /> Mark as unread
        </Button>
        <Button variant="outline" size="sm" @click="emit('deleteFromProvider')">
          <Icon icon="tabler:trash" /> Delete in provider
        </Button>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Message Actions</h4>
        <Button variant="outline" size="sm" @click="emit('togglePin')">
          <Icon icon="tabler:pin" /> Pin
        </Button>
        <Button variant="outline" size="sm" @click="emit('toggleImportant')">
          <Icon icon="tabler:star" /> Important
        </Button>
        <Button variant="outline" size="sm" @click="emit('mute')">
          <Icon icon="tabler:bell-off" /> Mute
        </Button>
        <Button variant="outline" size="sm" @click="emit('replyAll')">
          <Icon icon="tabler:reply-all" /> Reply All
        </Button>
        <Button variant="outline" size="sm" @click="emit('forwardMessage')">
          <Icon icon="tabler:mail-forward" /> Forward
        </Button>
        <div class="export-format-group" aria-label="Export message">
          <Button
            v-for="item in exportFormats"
            :key="item.format"
            variant="outline"
            size="sm"
            @click="emit('exportMessage', item.format)"
          >
            <Icon icon="tabler:download" /> {{ item.label }}
          </Button>
        </div>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Redirect</h4>
        <div class="redirect-row">
          <input
            v-model="redirectRecipientsText"
            class="redirect-input"
            type="text"
            placeholder="recipient@example.com, team@example.com"
          />
          <Button
            variant="outline"
            size="sm"
            :disabled="!redirectRecipientsText.trim()"
            @click="emit('redirectMessage', redirectRecipientsText)"
          >
            <Icon icon="tabler:send" /> Redirect
          </Button>
        </div>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Labels</h4>
        <div v-if="labels.length" class="label-chip-row">
          <button
            v-for="label in labels"
            :key="label"
            class="label-chip"
            type="button"
            @click="emit('removeLabel', label)"
          >
            {{ label }}
            <Icon icon="tabler:x" />
          </button>
        </div>
        <div class="label-chip-row">
          <button
            v-for="label in quickLabels"
            :key="label"
            class="label-chip add"
            type="button"
            :disabled="labels.includes(label)"
            @click="emit('addLabel', label)"
          >
            <Icon icon="tabler:tag-plus" />
            {{ label }}
          </button>
        </div>
      </div>
      <div class="actions-group">
        <h4 class="group-title">Snooze</h4>
        <p v-if="snoozeUntil" class="snooze-status">
          Snoozed until {{ new Date(snoozeUntil).toLocaleString() }}
        </p>
        <div class="export-format-group" aria-label="Snooze message">
          <Button variant="outline" size="sm" @click="emit('snoozeMessage', snoozePreset(1))">
            <Icon icon="tabler:clock" /> Tomorrow
          </Button>
          <Button variant="outline" size="sm" @click="emit('snoozeMessage', snoozePreset(7))">
            <Icon icon="tabler:calendar-time" /> Next week
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.related-tab {
  padding: 0.75rem;
}

.related-actions {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.actions-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.export-format-group {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(6.25rem, 1fr));
  gap: 0.375rem;
}

.redirect-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.375rem;
}

.redirect-input {
  min-width: 0;
  min-height: 1.875rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
}

.label-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.label-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  min-height: 1.875rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  color: var(--hh-text-primary, #1f2937);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 84%, transparent);
  font-size: 0.75rem;
}

.label-chip.add {
  color: var(--hh-accent, #2563eb);
}

.label-chip:disabled {
  cursor: default;
  opacity: 0.45;
}

.label-chip svg {
  width: 14px;
  height: 14px;
}

.snooze-status {
  margin: 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.group-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--hh-text-secondary, #6b7280);
  margin: 0;
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
```

### `frontend/src/domains/communications/components/MessageTimelineTab.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageTimelineTab.vue`
- Size bytes / Размер в байтах: `1988`
- Included characters / Включено символов: `1988`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import type { CommunicationMessageDetailResponse } from '../types/communications'

const props = defineProps<{
  detail: CommunicationMessageDetailResponse | null
}>()

interface TimelineEntry {
  label: string
  time: string | null
}

const entries = props.detail?.message
  ? [
      { label: 'Received', time: props.detail.message.occurred_at ?? props.detail.message.projected_at },
      { label: 'Projected', time: props.detail.message.projected_at },
      { label: 'State changed', time: props.detail.message.local_state_changed_at },
      { label: 'AI analyzed', time: props.detail.message.ai_summary_generated_at }
    ].filter(e => e.time != null)
  : []
</script>

<template>
  <div class="timeline-tab">
    <div v-if="entries.length === 0" class="no-data">No timeline data</div>
    <div v-else class="timeline-list">
      <div v-for="(entry, i) in entries" :key="i" class="timeline-entry">
        <div class="timeline-dot" />
        <div class="timeline-content">
          <span class="timeline-label">{{ entry.label }}</span>
          <span class="timeline-time">{{ entry.time }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timeline-tab {
  padding: 0.75rem;
}

.timeline-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  position: relative;
}

.timeline-entry {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.timeline-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--hh-accent, #3b82f6);
  margin-top: 0.375rem;
  flex-shrink: 0;
}

.timeline-content {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.timeline-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary, #1f2937);
}

.timeline-time {
  font-size: 0.6875rem;
  color: var(--hh-text-tertiary, #9ca3af);
}

.no-data {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.875rem;
  padding: 2rem;
  text-align: center;
}
</style>
```

### `frontend/src/domains/communications/components/MessageTrustReviewPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageTrustReviewPanel.vue`
- Size bytes / Размер в байтах: `4574`
- Included characters / Включено символов: `4574`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import type { CommunicationMessageInsight } from '../types/communications'

const props = defineProps<{
  messageId: string | null
  insight: CommunicationMessageInsight | null
}>()

const emit = defineEmits<{
  reviewSecurity: []
  reviewRecipients: []
}>()

const smartCc = computed(() => props.insight?.smartCc ?? null)
const authReview = computed(() => props.insight?.auth ?? null)
const signatureReview = computed(() => props.insight?.signature ?? null)
const authRisk = computed(() => authReview.value?.risk ?? null)
const authChecks = computed(() => {
  const auth = authReview.value
  if (!auth) return []
  return [
    { label: 'SPF', result: auth.auth.spf?.result ?? 'missing', passed: auth.risk.spf_pass },
    { label: 'DKIM', result: auth.auth.dkim?.result ?? 'missing', passed: auth.risk.dkim_pass },
    { label: 'DMARC', result: auth.auth.dmarc?.result ?? 'missing', passed: auth.risk.dmarc_pass }
  ]
})
</script>

<template>
  <section class="message-review-grid">
    <article class="security-review">
      <div class="review-header">
        <Icon icon="tabler:shield-check" class="intel-icon" />
        <span class="intel-title">Security Review</span>
      </div>
      <Button variant="outline" size="sm" :disabled="!messageId" @click="emit('reviewSecurity')">
        <Icon icon="tabler:shield-search" /> Check auth
      </Button>
      <div v-if="authRisk" class="security-risk" :class="{ risky: authRisk.is_spoofed }">
        <strong>{{ authRisk.risk_summary }}</strong>
        <div class="auth-chip-row">
          <span
            v-for="check in authChecks"
            :key="check.label"
            class="auth-chip"
            :class="{ passed: check.passed }"
          >
            {{ check.label }} {{ check.result }}
          </span>
        </div>
      </div>
      <div v-if="signatureReview" class="signature-review">
        <span>{{ signatureReview.has_signature ? 'Signed message' : 'No signature detected' }}</span>
        <span v-if="signatureReview.signature_type">{{ signatureReview.signature_type }}</span>
        <span v-if="signatureReview.cert_expiry_warning">{{ signatureReview.cert_expiry_warning }}</span>
      </div>
    </article>

    <article class="recipient-review">
      <div class="review-header">
        <Icon icon="tabler:user-plus" class="intel-icon" />
        <span class="intel-title">Recipient Suggestions</span>
      </div>
      <Button variant="outline" size="sm" :disabled="!messageId" @click="emit('reviewRecipients')">
        <Icon icon="tabler:users-plus" /> Smart CC
      </Button>
      <div v-if="smartCc" class="recipient-chip-row">
        <span v-if="smartCc.suggestions.length === 0" class="empty-review">No suggestions</span>
        <span v-for="suggestion in smartCc.suggestions" :key="suggestion" class="recipient-chip">
          {{ suggestion }}
        </span>
      </div>
    </article>
  </section>
</template>

<style scoped>
.message-review-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: 0.625rem;
}

.security-review,
.recipient-review {
  display: grid;
  align-content: start;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border-info, #bae6fd);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-info, #f0f9ff) 78%, transparent);
}

.review-header {
  display: flex;
  align-items: center;
  gap: 0.375rem;
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

.security-risk,
.signature-review,
.recipient-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.security-risk {
  display: grid;
}

.security-risk strong {
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.security-risk.risky strong {
  color: var(--hh-danger, #dc2626);
}

.auth-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.auth-chip,
.recipient-chip,
.empty-review {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 999px;
  color: var(--hh-text-secondary, #6b7280);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent);
  font-size: 0.6875rem;
}

.auth-chip.passed {
  color: var(--hh-success, #15803d);
}
</style>
```

### `frontend/src/domains/communications/components/OutboxStatusStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/OutboxStatusStrip.vue`
- Size bytes / Размер в байтах: `6083`
- Included characters / Включено символов: `6083`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { CommunicationOutboxItem } from '../types/communications'
import {
  outboxStatusPresentation,
  visibleOutboxStatusItems
} from './outboxStatus'

const props = defineProps<{
  items: CommunicationOutboxItem[]
  isLoading: boolean
  isLoadingMore: boolean
  hasMore: boolean
  isUndoing: boolean
  errorMessage: string
}>()

const emit = defineEmits<{
  undo: [outboxId: string]
  loadMore: []
  prefetchMore: []
}>()

const statusItems = computed(() =>
  visibleOutboxStatusItems(props.items).map((item) => ({
    item,
    presentation: outboxStatusPresentation(item)
  }))
)
</script>

<template>
  <section v-if="isLoading || errorMessage || statusItems.length || hasMore" class="outbox-status-strip" aria-label="Outbox delivery status">
    <div v-if="isLoading" class="outbox-status-skeleton" />
    <div v-else-if="errorMessage" class="outbox-status-error">
      <Icon icon="tabler:alert-circle" />
      <span>{{ errorMessage }}</span>
    </div>
    <div v-else class="outbox-status-items">
      <article
        v-for="{ item, presentation } in statusItems"
        :key="item.outbox_id"
        class="outbox-status-item"
        :class="`tone-${presentation.tone}`"
      >
        <Icon :icon="presentation.icon" class="outbox-status-icon" />
        <div class="outbox-status-copy">
          <div class="outbox-status-line">
            <span class="outbox-status-title">{{ presentation.title }}</span>
            <span class="outbox-status-subject">{{ item.subject || '(No subject)' }}</span>
          </div>
          <div class="outbox-status-detail">{{ presentation.detail }}</div>
        </div>
        <button
          v-if="presentation.canUndo"
          class="outbox-status-undo"
          type="button"
          :disabled="isUndoing"
          title="Undo send"
          @click="emit('undo', item.outbox_id)"
        >
          <Icon icon="tabler:arrow-back-up" />
        </button>
      </article>
      <button
        v-if="hasMore"
        class="outbox-status-more"
        type="button"
        :disabled="isLoadingMore"
        title="Load more delivery records"
        @mouseenter="emit('prefetchMore')"
        @focus="emit('prefetchMore')"
        @click="emit('loadMore')"
      >
        <Icon :icon="isLoadingMore ? 'tabler:loader-2' : 'tabler:chevron-right'" />
      </button>
    </div>
  </section>
</template>

<style scoped>
.outbox-status-strip {
  flex: 0 0 auto;
  padding: 0.5rem;
  border-right: 1px solid var(--hh-border, #e5e7eb);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
}

.outbox-status-items {
  display: flex;
  gap: 0.5rem;
  overflow-x: auto;
  padding-bottom: 0.125rem;
}

.outbox-status-item {
  display: grid;
  grid-template-columns: auto minmax(10rem, 1fr) auto;
  align-items: center;
  gap: 0.5rem;
  min-width: min(22rem, 100%);
  max-width: 26rem;
  padding: 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  box-shadow: 0 10px 30px color-mix(in srgb, var(--hh-shadow, #0f172a) 9%, transparent);
}

.outbox-status-icon {
  width: 1rem;
  height: 1rem;
}

.outbox-status-copy {
  min-width: 0;
}

.outbox-status-line {
  display: flex;
  gap: 0.375rem;
  min-width: 0;
  align-items: baseline;
}

.outbox-status-title {
  flex: 0 0 auto;
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--hh-text-primary, #1f2937);
}

.outbox-status-subject {
  overflow: hidden;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.outbox-status-detail {
  overflow: hidden;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.6875rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.outbox-status-undo {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 90%, transparent);
  color: var(--hh-text-primary, #1f2937);
  cursor: pointer;
}

.outbox-status-undo:hover:not(:disabled) {
  background: var(--hh-bg-hover, #f3f4f6);
}

.outbox-status-undo:disabled {
  cursor: wait;
  opacity: 0.65;
}

.outbox-status-more {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  min-width: 2.25rem;
  height: 2.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-primary, #1f2937);
  cursor: pointer;
}

.outbox-status-more:hover:not(:disabled) {
  background: var(--hh-bg-hover, #f3f4f6);
}

.outbox-status-more:disabled {
  cursor: wait;
  opacity: 0.65;
}

.outbox-status-error {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--hh-danger, #b91c1c);
  font-size: 0.75rem;
}

.outbox-status-skeleton {
  width: min(24rem, 100%);
  height: 2.75rem;
  border-radius: 8px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--hh-bg-muted, #f3f4f6) 70%, transparent),
    color-mix(in srgb, var(--hh-bg-primary, #ffffff) 72%, transparent),
    color-mix(in srgb, var(--hh-bg-muted, #f3f4f6) 70%, transparent)
  );
  background-size: 200% 100%;
  animation: outbox-status-pulse 1.4s ease-in-out infinite;
}

.tone-success .outbox-status-icon {
  color: var(--hh-success, #047857);
}

.tone-warning .outbox-status-icon {
  color: var(--hh-warning, #b45309);
}

.tone-danger .outbox-status-icon {
  color: var(--hh-danger, #b91c1c);
}

.tone-muted .outbox-status-icon {
  color: var(--hh-text-secondary, #6b7280);
}

@keyframes outbox-status-pulse {
  0% {
    background-position: 200% 0;
  }

  100% {
    background-position: -200% 0;
  }
}
</style>
```

### `frontend/src/domains/communications/components/RichComposeEditor.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/RichComposeEditor.vue`
- Size bytes / Размер в байтах: `12863`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import {
	EditorContent,
	useEditor
} from '@tiptap/vue-3'
import Icon from '../../../shared/ui/Icon.vue'
import { richComposeExtensions } from './richComposeExtensions'
import { normalizeMailComposeLinkHref, sanitizeMailComposePastedHtml, type MailComposeTextAlign } from './richComposeHtml'

const props = defineProps<{
	modelValue: string
	placeholder?: string
}>()

const emit = defineEmits<{
	'update:modelValue': [value: string]
	'attachments-dropped': [files: File[]]
	blur: []
}>()

const isFocused = ref(false)
const lastAppliedHtml = ref('')
const linkHref = ref('')

type RichComposeEditorInstance = {
	getHTML: () => string
	commands: {
		insertContent: (value: string) => unknown
		setContent: (value: string, options?: { emitUpdate?: boolean }) => unknown
	}
	chain: () => {
		focus: () => {
			toggleMark: (name: string) => { run: () => unknown }
			setNode: (name: string) => { run: () => unknown }
			toggleNode: (name: string, fallback: string, attributes?: Record<string, unknown>) => { run: () => unknown }
			updateAttributes: (name: string, attributes: Record<string, unknown>) => { run: () => unknown }
			toggleList: (name: string, itemName: string) => { run: () => unknown }
			toggleWrap: (name: string) => { run: () => unknown }
			extendMarkRange: (name: string) => {
				setMark: (mark: string, attributes: Record<string, unknown>) => { run: () => unknown }
				unsetMark: (mark: string) => { run: () => unknown }
			}
		}
	}
	isActive: (name: string, attributes?: Record<string, unknown>) => boolean
}

function normalizedHtml(value: string): string {
	return value.trim() ? value : '<p></p>'
}

function emitCurrentHtml(editor: RichComposeEditorInstance): void {
	const html = editor.getHTML()
	lastAppliedHtml.value = html
	emit('update:modelValue', html)
}

function insertSanitizedClipboardHtml(editor: RichComposeEditorInstance, html: string): boolean {
	if (!html.trim()) return false
	editor.commands.insertContent(sanitizeMailComposePastedHtml(html))
	return true
}

const editor = useEditor({
	content: normalizedHtml(props.modelValue),
	extensions: richComposeExtensions,
	editorProps: {
		attributes: {
			class: 'rich-compose-prosemirror',
			'aria-multiline': 'true',
			role: 'textbox'
		},
		handlePaste: (_view, event) => {
			const html = event.clipboardData?.getData('text/html') ?? ''
			const currentEditor = editor.value
			if (!currentEditor || !insertSanitizedClipboardHtml(currentEditor, html)) return false
			event.preventDefault()
			return true
		},
		handleDrop: (_view, event) => {
			const files = Array.from(event.dataTransfer?.files ?? [])
			if (files.length > 0) {
				emit('attachments-dropped', files)
				event.preventDefault()
				return true
			}
			const html = event.dataTransfer?.getData('text/html') ?? ''
			const currentEditor = editor.value
			if (!currentEditor || !insertSanitizedClipboardHtml(currentEditor, html)) return false
			event.preventDefault()
			return true
		}
	},
	onCreate: ({ editor }) => {
		lastAppliedHtml.value = editor.getHTML()
	},
	onUpdate: ({ editor }) => {
		emitCurrentHtml(editor)
	},
	onFocus: () => {
		isFocused.value = true
	},
	onBlur: ({ editor }) => {
		isFocused.value = false
		emitCurrentHtml(editor)
		emit('blur')
	}
})

watch(
	() => props.modelValue,
	(value) => {
		const currentEditor = editor.value
		if (!currentEditor) return
		const nextHtml = normalizedHtml(value)
		if (nextHtml === lastAppliedHtml.value) return
		if (isFocused.value) return
		if (currentEditor.getHTML() === nextHtml) return
		currentEditor.commands.setContent(nextHtml, { emitUpdate: false })
		lastAppliedHtml.value = nextHtml
	}
)

onBeforeUnmount(() => {
	editor.value?.destroy()
})

type RichComposeCommand =
	| 'alignCenter'
	| 'alignLeft'
	| 'alignRight'
	| 'blockquote'
	| 'bold'
	| 'bulletList'
	| 'heading2'
	| 'heading3'
	| 'italic'
	| 'link'
	| 'orderedList'
	| 'paragraph'
	| 'unlink'

type RichComposeActiveCommand = Exclude<RichComposeCommand, 'unlink'>

function runCommand(command: RichComposeCommand): void {
	const currentEditor = editor.value
	if (!currentEditor) return
	if (command === 'bold') {
		currentEditor.chain().focus().toggleMark('bold').run()
		return
	}
	if (command === 'italic') {
		currentEditor.chain().focus().toggleMark('italic').run()
		return
	}
	if (command === 'paragraph') {
		currentEditor.chain().focus().setNode('paragraph').run()
		return
	}
	if (command === 'heading2') {
		currentEditor.chain().focus().toggleNode('heading', 'paragraph', { level: 2 }).run()
		return
	}
	if (command === 'heading3') {
		currentEditor.chain().focus().toggleNode('heading', 'paragraph', { level: 3 }).run()
		return
	}
	if (command === 'alignLeft') {
		setActiveBlockTextAlign(currentEditor, 'left')
		return
	}
	if (command === 'alignCenter') {
		setActiveBlockTextAlign(currentEditor, 'center')
		return
	}
	if (command === 'alignRight') {
		setActiveBlockTextAlign(currentEditor, 'right')
		return
	}
	if (command === 'orderedList') {
		currentEditor.chain().focus().toggleList('orderedList', 'listItem').run()
		return
	}
	if (command === 'blockquote') {
		currentEditor.chain().focus().toggleWrap('blockquote').run()
		return
	}
	if (command === 'link') {
		const href = normalizeMailComposeLinkHref(linkHref.value)
		if (!href) return
		currentEditor.chain().focus().extendMarkRange('link').setMark('link', { href }).run()
		linkHref.value = href
		return
	}
	if (command === 'unlink') {
		currentEditor.chain().focus().extendMarkRange('link').unsetMark('link').run()
		linkHref.value = ''
		return
	}
	currentEditor.chain().focus().toggleList('bulletList', 'listItem').run()
}

function setActiveBlockTextAlign(editor: RichComposeEditorInstance, textAlign: MailComposeTextAlign): void {
	const nodeName = editor.isActive('heading') ? 'heading' : 'paragraph'
	editor.chain().focus().updateAttributes(nodeName, { textAlign }).run()
}

function isCommandActive(command: RichComposeActiveCommand): boolean {
	const currentEditor = editor.value
	if (!currentEditor) return false
	if (command === 'heading2') return currentEditor.isActive('heading', { level: 2 })
	if (command === 'heading3') return currentEditor.isActive('heading', { level: 3 })
	if (command === 'alignLeft') return isActiveTextAlign(currentEditor, 'left')
	if (command === 'alignCenter') return isActiveTextAlign(currentEditor, 'center')
	if (command === 'alignRight') return isActiveTextAlign(currentEditor, 'right')
	return currentEditor.isActive(command)
}

function isActiveTextAlign(editor: RichComposeEditorInstance, textAlign: MailComposeTextAlign): boolean {
	return editor.isActive('heading', { textAlign }) || editor.isActive('paragraph', { textAlign })
}
</script>

<template>
	<div class="rich-compose-editor">
		<div class="rich-compose-toolbar" aria-label="Rich text formatting">
			<button type="button" title="Paragraph" :class="{ active: isCommandActive('paragraph') }" @click="runCommand('paragraph')">
				<Icon icon="tabler:pilcrow" size="16" />
			</button>
			<button type="button" title="Heading" :class="{ active: isCommandActive('heading2') }" @click="runCommand('heading2')">
				<Icon icon="tabler:h-2" size="16" />
			</button>
			<button type="button" title="Subheading" :class="{ active: isCommandActive('heading3') }" @click="runCommand('heading3')">
				<Icon icon="tabler:h-3" size="16" />
			</button>
			<button type="button" title="Align left" :class="{ active: isCommandActive('alignLeft') }" @click="runCommand('alignLeft')">
				<Icon icon="tabler:align-left" size="16" />
			</button>
			<button type="button" title="Align center" :class="{ active: isCommandActive('alignCenter') }" @click="runCommand('alignCenter')">
				<Icon icon="tabler:align-center" size="16" />
			</button>
			<button type="button" title="Align right" :class="{ active: isCommandActive('alignRight') }" @click="runCommand('alignRight')">
				<Icon icon="tabler:align-right" size="16" />
			</button>
			<button type="button" title="Bold" :class="{ active: isCommandActive('bold') }" @click="runCommand('bold')">
				<Icon icon="tabler:bold" size="16" />
			</button>
			<button type="button" title="Italic" :class="{ active: isCommandActive('italic') }" @click="runCommand('italic')">
				<Icon icon="tabler:italic" size="16" />
			</button>
			<button type="button" title="Bulleted list" :class="{ active: isCommandActive('bulletList') }" @click="runCommand('bulletList')">
				<Icon icon="tabler:list" size="16" />
			</button>
			<button type="button" title="Numbered list" :class="{ active: isCommandActive('orderedList') }" @click="runCommand('orderedList')">
				<Icon icon="tabler:list-numbers" size="16" />
			</button>
			<button type="button" title="Quote" :class="{ active: isCommandActive('blockquote') }" @click="runCommand('blockquote')">
				<Icon icon="tabler:quote" size="16" />
			</button>
			<span class="rich-compose-link-tools">
				<input
					v-model="linkHref"
					type="url"
					placeholder="https://example.com"
					aria-label="Link URL"
					@keydown.enter.prevent="runCommand('link')"
				>
				<button type="button" title="Link" :class="{ active: isCommandActive('link') }" @click="runCommand('link')">
					<Icon icon="tabler:link" size="16" />
				</button>
				<button type="button" title="Unlink" @click="runCommand('unlink')">
					<Icon icon="tabler:unlink" size="16" />
				</button>
			</span>
		</div>
		<EditorContent
			v-if="editor"
			:editor="editor"
			class="rich-compose-surface"
			:data-placeholder="placeholder ?? 'Write your message...'"
		/>
	</div>
</template>

<style scoped>
.rich-compose-editor {
	display: flex;
	min-height: 240px;
	flex: 1;
	flex-direction: column;
	overflow: hidden;
	border: 1px solid var(--hh-border, #e5e7eb);
	border-radius: 0.375rem;
	background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
}

.rich-compose-editor:focus-within {
	border-color: var(--hh-accent, #3b82f6);
	box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.rich-compose-toolbar {
	display: flex;
	align-items: center;
	gap: 0.125rem;
	padding: 0.25rem;
	border-bottom: 1px solid var(--hh-border, #e5e7eb);
	background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 86%, transparent);
}

.rich-compose-toolbar button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 1.75rem;
	height: 1.75rem;
	border: 0;
	border-radius: 0.25rem;
	background: transparent;
	color: var(--hh-text-secondary, #6b7280);
	cursor: pointer;
}

.rich-compose-toolbar button:hover {
	background: var(--hh-hover-bg, rgba(148, 163, 184, 0.12));
	color: var(--hh-text-primary, #1f2937);
}

.rich-compose-toolbar button.active {
	background: var(--hh-bg-primary, #ffffff);
	color: var(--hh-text-primary, #1f2937);
	box-shadow: 0 1px 2px rgba(15, 23, 42, 0.12);
}

.rich-compose-link-tools {
	display: inline-flex;
	align-items: center;
	gap: 0.125rem;
	margin-left: 0.25rem;
	padding-left: 0.25rem;
	border-left: 1px solid var(--hh-border, #e5e7eb);
}

.rich-compose-link-tools input {
	width: clamp(9rem, 18vw, 14rem);
	height: 1.75rem;
	border: 1px solid var(--hh-border, #e5e7eb);
	border-radius: 0.25rem;
	background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
	color: var(--hh-text-primary, #1f2937);
	font-size: 0.75rem;
	outline: none;
	padding: 0 0.5rem;
}

.rich-compose-link-tools input:focus {
	border-color: var(--hh-accent, #3b82f6);
	box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.rich-compose-surface {
	display: flex;
	flex: 1;
	min-height: 200px;
	overflow-y: auto;
	position: relative;
}

.rich-compose-surface :deep(.rich-compose-prosemirror) {
	width: 100%;
	min-height: 200px;
	padding: 0.625rem 0.75rem;
	color: var(--hh-text-primary, #1f2937);
	font-size: 0.8125rem;
	line-height: 1.6;
	outline: none;
}

.rich-compose-surface:has(.rich-compose-prosemirror > p:first-child:last-child:empty)::before {
	content: attr(data-placeholder);
	position: absolute;
	top: 0.625rem;
	left
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/SavedSearchRuleGroupEditor.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/SavedSearchRuleGroupEditor.vue`
- Size bytes / Размер в байтах: `3335`
- Included characters / Включено символов: `3335`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
defineOptions({ name: 'SavedSearchRuleGroupEditor' })

import {
  createSavedSearchRuleCondition,
  createSavedSearchRuleGroup,
  type SavedSearchRuleGroup,
  type SavedSearchRuleNode
} from '../forms/savedSearchForm'
import {
  savedSearchRuleGroupDepthLabel,
  savedSearchRuleGroupSummary
} from './savedSearchRuleTreePresentation'

const props = defineProps<{
  group: SavedSearchRuleGroup
  isRoot?: boolean
  depth?: number
}>()

const emit = defineEmits<{
  removeGroup: []
}>()

function addRule() {
  props.group.children.push(createSavedSearchRuleCondition())
}

function addGroup() {
  props.group.children.push(createSavedSearchRuleGroup('all', [createSavedSearchRuleCondition()]))
}

function removeNode(nodeId: string) {
  props.group.children = props.group.children.filter((child) => child.id !== nodeId)
}

function removeNestedGroup(node: SavedSearchRuleNode) {
  if (node.kind !== 'group') return
  removeNode(node.id)
}

function nextDepth(): number {
  return (props.depth ?? 0) + 1
}
</script>

<template>
  <div class="saved-search-group-builder" :class="{ root: isRoot }">
    <div class="saved-search-group-builder-header">
      <div class="saved-search-group-builder-summary">
        <span class="saved-search-group-builder-depth">{{ savedSearchRuleGroupDepthLabel(depth ?? 0) }}</span>
        <span class="saved-search-group-builder-description">{{ savedSearchRuleGroupSummary(group) }}</span>
      </div>
      <label class="saved-search-field">
        <span>{{ isRoot ? 'Match' : 'Group match' }}</span>
        <select v-model="group.matchMode">
          <option value="all">All conditions</option>
          <option value="any">Any condition</option>
        </select>
      </label>
      <div class="saved-search-group-builder-actions">
        <button class="saved-search-rule-add" type="button" @click="addRule">+ Rule</button>
        <button class="saved-search-rule-add" type="button" @click="addGroup">+ Group</button>
        <button
          v-if="!isRoot"
          class="saved-search-rule-remove"
          type="button"
          @click="emit('removeGroup')"
        >
          Remove group
        </button>
      </div>
    </div>

    <div v-if="group.children.length" class="saved-search-rules saved-search-rules-tree">
      <template v-for="node in group.children" :key="node.id">
        <label v-if="node.kind === 'rule'" class="saved-search-rule-row">
          <select v-model="node.field">
            <option value="subject">Subject</option>
            <option value="body">Body</option>
            <option value="sender">Sender</option>
            <option value="all">All</option>
          </select>
          <select v-model="node.operator">
            <option value=":">Contains</option>
            <option value="=">Equals</option>
          </select>
          <input v-model="node.value" type="text" autocomplete="off" />
          <button class="saved-search-rule-remove" type="button" @click="removeNode(node.id)">Remove</button>
        </label>
        <SavedSearchRuleGroupEditor
          v-else
          :group="node"
          :depth="nextDepth()"
          @remove-group="removeNestedGroup(node)"
        />
      </template>
    </div>
    <div v-else class="saved-search-rule-empty">No rules yet</div>
  </div>
</template>
```

### `frontend/src/domains/communications/components/SavedSearchStrip.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/SavedSearchStrip.css`
- Size bytes / Размер в байтах: `8519`
- Included characters / Включено символов: `8519`
- Truncated / Обрезано: `no`

```text
.saved-search-strip {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 2.25rem;
  padding: 0.375rem 0.5rem;
  border-right: 1px solid var(--hh-border, #e5e7eb);
  border-bottom: 1px solid var(--hh-border, #e5e7eb);
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 82%, transparent);
  backdrop-filter: blur(var(--hh-panel-blur));
  overflow: hidden;
}

.saved-search-group {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  flex: 1 1 0;
  min-width: 12rem;
}

.saved-search-virtual-scroll {
  flex: 1 1 auto;
  min-width: 6rem;
  height: 1.625rem;
  overflow-x: auto;
  overflow-y: hidden;
}

.saved-search-virtual-track {
  position: relative;
  height: 1.625rem;
}

.saved-search-virtual-row {
  position: absolute;
  top: 0;
  left: 0;
  height: 1.625rem;
  overflow: hidden;
}

.saved-search-item {
  display: inline-flex;
  align-items: center;
  gap: 0.125rem;
  width: 11.5rem;
}

.saved-search-label {
  font-size: 0.6875rem;
  color: var(--hh-text-muted, #9ca3af);
  text-transform: uppercase;
  letter-spacing: 0;
}

.saved-search-loading-more {
  color: var(--hh-text-muted, #9ca3af);
  font-size: 0.75rem;
  white-space: nowrap;
}

.saved-search-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  height: 1.5rem;
  max-width: 12rem;
  padding: 0 0.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  white-space: nowrap;
  cursor: pointer;
}

.saved-search-name {
  overflow: hidden;
  text-overflow: ellipsis;
}

.saved-search-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.125rem;
  height: 1rem;
  padding: 0 0.25rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f3f4f6) 76%, transparent);
  color: var(--hh-text-muted, #9ca3af);
  font-size: 0.6875rem;
  line-height: 1;
}

.saved-search-chip:hover,
.saved-search-chip.active,
.saved-search-tool:hover {
  color: var(--hh-accent, #3b82f6);
  border-color: color-mix(in srgb, var(--hh-accent, #3b82f6) 42%, var(--hh-border, #e5e7eb));
}

.saved-search-tool.danger:hover {
  color: var(--hh-danger, #ef4444);
  border-color: color-mix(in srgb, var(--hh-danger, #ef4444) 42%, var(--hh-border, #e5e7eb));
}

.saved-search-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 88%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
}

.saved-search-actions {
  flex: 0 0 auto;
  min-width: auto;
  margin-left: auto;
}

.saved-search-icon {
  width: 0.875rem;
  height: 0.875rem;
  flex: 0 0 auto;
}

.saved-search-skeleton {
  width: 12rem;
  height: 1.25rem;
  border-radius: 6px;
  background: linear-gradient(
    90deg,
    var(--hh-bg-secondary, #f3f4f6),
    color-mix(in srgb, var(--hh-bg-secondary, #f3f4f6) 42%, transparent),
    var(--hh-bg-secondary, #f3f4f6)
  );
}

.saved-search-form {
  display: grid;
  gap: 0.875rem;
}

.saved-search-delete {
  display: grid;
  gap: 0.875rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
}

.saved-search-field {
  display: grid;
  gap: 0.3125rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.saved-search-field input,
.saved-search-field textarea,
.saved-search-field select {
  width: 100%;
  min-height: 2.25rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
  color: var(--hh-text-primary, #111827);
  padding: 0.5rem 0.625rem;
  font: inherit;
}

.saved-search-effective-query {
  display: flex;
  align-items: center;
  min-height: 2.25rem;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 90%, transparent);
  color: var(--hh-text-primary, #111827);
  font: 0.75rem/1.4 ui-monospace, SFMono-Regular, Menlo, monospace;
  white-space: pre-wrap;
  word-break: break-word;
}

.saved-search-field small,
.saved-search-delete small {
  color: var(--hh-danger, #ef4444);
}

.saved-search-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 0.75rem;
}

.saved-search-preset-row,
.saved-search-rule-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.saved-search-preset,
.saved-search-rule-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.saved-search-rules-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  gap: 0.5rem;
}

.saved-search-rule-add,
.saved-search-rule-remove {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 86%, transparent);
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
  min-height: 1.5rem;
  padding: 0 0.5rem;
  cursor: pointer;
}

.saved-search-rules {
  display: grid;
  gap: 0.5rem;
}

.saved-search-rules-tree {
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 90%, transparent);
}

.saved-search-group-builder {
  display: grid;
  gap: 0.75rem;
  padding: 0.75rem;
  border: 1px solid color-mix(in srgb, var(--hh-border, #e5e7eb) 88%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 94%, transparent);
}

.saved-search-group-builder.root {
  padding: 0;
  border: 0;
  background: transparent;
}

.saved-search-group-builder-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.saved-search-group-builder-summary {
  display: grid;
  gap: 0.125rem;
}

.saved-search-group-builder-depth {
  font-size: 0.6875rem;
  color: var(--hh-text-muted, #9ca3af);
  text-transform: uppercase;
}

.saved-search-group-builder-description {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
}

.saved-search-group-builder-actions {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.saved-search-rule-row {
  display: grid;
  grid-template-columns: minmax(8rem, 9rem) minmax(7rem, 7.5rem) minmax(0, 1fr) 5rem;
  align-items: center;
  gap: 0.5rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.saved-search-rule-row input,
.saved-search-rule-row select {
  width: 100%;
  min-height: 1.875rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 6px;
  background: color-mix(in srgb, var(--hh-bg-primary, #ffffff) 92%, transparent);
  color: var(--hh-text-primary, #111827);
  padding: 0 0.5rem;
  font: inherit;
}

.saved-search-rule-empty {
  font-size: 0.75rem;
  color: var(--hh-text-muted, #9ca3af);
  margin: 0;
}

.saved-search-rule-error {
  margin: 0;
  font-size: 0.75rem;
  color: var(--hh-danger, #ef4444);
}

.saved-search-preset {
  cursor: pointer;
  padding: 0 0.625rem;
}

.saved-search-rule-chip {
  gap: 0.25rem;
  padding: 0 0.5rem;
}

.saved-search-rule-chip b {
  color: var(--hh-text-muted, #9ca3af);
  font-weight: 600;
}

.saved-search-check {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.8125rem;
}

.saved-search-form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.saved-search-primary,
.saved-search-secondary,
.saved-search-danger {
  min-height: 2rem;
  border-radius: 6px;
  padding: 0 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  cursor: pointer;
}

.saved-search-primary {
  background: var(--hh-accent, #3b82f6);
  color: #fff;
}

.saved-search-secondary {
  background: transparent;
  color: var(--hh-text-secondary, #6b7280);
}

.saved-search-danger {
  background: var(--hh-danger, #ef4444);
  color: #fff;
}

.saved-search-primary:disabled,
.saved-search-danger:disabled {
  opacity: 0.65;
  cursor: progress;
}
```

### `frontend/src/domains/communications/components/SavedSearchStrip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/SavedSearchStrip.vue`
- Size bytes / Размер в байтах: `19515`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useForm } from 'vee-validate'
import { useVirtualizer } from '@tanstack/vue-virtual'
import Dialog from '../../../shared/ui/Dialog.vue'
import Icon from '../../../shared/ui/Icon.vue'
import SavedSearchRuleGroupEditor from './SavedSearchRuleGroupEditor.vue'
import {
  useCreateSavedSearchMutation,
  useDeleteSavedSearchMutation,
  useSavedSearchesQuery,
  useUpdateSavedSearchMutation
} from '../queries/useCommunicationsQuery'
import {
  composeSavedSearchRuleTreeQuery,
  createSavedSearchRuleGroup,
  flattenSavedSearchRuleTree,
  parseSavedSearchQuery,
  savedSearchDeleteDialogCopy,
  savedSearchChannelOptions,
  savedSearchFilterChips,
  savedSearchFormDefaults,
  normalizeSavedSearchBuilderState,
  savedSearchFormToInput,
  savedSearchLocalStateOptions,
  savedSearchMessageCountLabel,
  savedSearchPresetOptions,
  validateSavedSearchRuleTree,
  savedSearchWorkflowOptions,
  savedSearchVeeValidationSchema,
  type SavedSearchRuleGroup,
  type SavedSearchPresetOption,
  type SavedSearchFormValues
} from '../forms/savedSearchForm'
import { useSavedSearchCommunicationListPrefetch } from '../queries/communicationPrefetch'
import type { CommunicationSavedSearch } from '../types/savedSearches'
import type { LocalMessageState, WorkflowState } from '../types/communications'
import './SavedSearchStrip.css'

const props = defineProps<{
  accountId: string | null
  activeId: string
  currentQuery: string
  currentWorkflowState: WorkflowState | ''
  currentLocalState: LocalMessageState
  currentChannelKind: string
}>()

const emit = defineEmits<{
  select: [savedSearch: CommunicationSavedSearch]
  deleted: [savedSearch: CommunicationSavedSearch]
}>()

const {
  data: smartFolderData,
  fetchNextPage: fetchNextSmartFolderPage,
  hasNextPage: hasNextSmartFolderPage,
  isFetchingNextPage: isFetchingNextSmartFolderPage,
  isLoading: isSmartFolderLoading
} = useSavedSearchesQuery(() => true, () => props.accountId || undefined)
const {
  data: savedSearchData,
  fetchNextPage: fetchNextSavedSearchPage,
  hasNextPage: hasNextSavedSearchPage,
  isFetchingNextPage: isFetchingNextSavedSearchPage,
  isLoading: isSavedSearchLoading
} = useSavedSearchesQuery(() => false, () => props.accountId || undefined)

const smartFolders = computed(() => smartFolderData.value ?? [])
const savedSearches = computed(() => savedSearchData.value ?? [])
const isLoading = computed(() => isSmartFolderLoading.value || isSavedSearchLoading.value)
const dialogOpen = ref(false)
const editingSearch = ref<CommunicationSavedSearch | null>(null)
const deleteDialogOpen = ref(false)
const deletingSearch = ref<CommunicationSavedSearch | null>(null)
const searchRuleTree = ref<SavedSearchRuleGroup>(createSavedSearchRuleGroup('all'))
const deleteError = ref('')
const createMutation = useCreateSavedSearchMutation()
const updateMutation = useUpdateSavedSearchMutation()
const deleteMutation = useDeleteSavedSearchMutation()
const prefetchSavedSearchCommunicationList = useSavedSearchCommunicationListPrefetch(() => props.accountId)
const smartFolderVirtualScrollRef = ref<HTMLDivElement | null>(null)
const savedSearchVirtualScrollRef = ref<HTMLDivElement | null>(null)
const {
  errors,
  handleSubmit,
  resetForm,
  setFieldValue,
  values: formValues
} = useForm<SavedSearchFormValues>({
  validationSchema: savedSearchVeeValidationSchema,
  initialValues: savedSearchFormDefaults(null, false)
})
const isSaving = computed(() => createMutation.isPending.value || updateMutation.isPending.value)
const isDeleting = computed(() => deleteMutation.isPending.value)
const dialogTitle = computed(() => {
  if (editingSearch.value) return editingSearch.value.is_smart_folder ? 'Edit smart folder' : 'Edit saved search'
  return formValues.is_smart_folder ? 'New smart folder' : 'New saved search'
})
const deleteCopy = computed(() => {
  return deletingSearch.value ? savedSearchDeleteDialogCopy(deletingSearch.value) : null
})
const activeFilterChips = computed(() =>
  savedSearchFilterChips(
    {
      ...formValues,
      match_mode: searchRuleTree.value.matchMode
    },
    flattenSavedSearchRuleTree(searchRuleTree.value)
  )
)
const effectiveQueryPreview = computed(() =>
  composeSavedSearchRuleTreeQuery(formValues.query, searchRuleTree.value)
)
const ruleValidation = computed(() => validateSavedSearchRuleTree(searchRuleTree.value))
const smartFolderVirtualOptions = computed(() => ({
  count: smartFolders.value.length,
  getScrollElement: () => smartFolderVirtualScrollRef.value,
  estimateSize: () => 192,
  horizontal: true,
  overscan: 6
}))
const savedSearchVirtualOptions = computed(() => ({
  count: savedSearches.value.length,
  getScrollElement: () => savedSearchVirtualScrollRef.value,
  estimateSize: () => 192,
  horizontal: true,
  overscan: 6
}))
const smartFolderVirtualizer = useVirtualizer(smartFolderVirtualOptions)
const savedSearchVirtualizer = useVirtualizer(savedSearchVirtualOptions)
const virtualSmartFolders = computed(() => smartFolderVirtualizer.value.getVirtualItems())
const virtualSavedSearches = computed(() => savedSearchVirtualizer.value.getVirtualItems())
const smartFolderVirtualTotalSize = computed(() => smartFolderVirtualizer.value.getTotalSize())
const savedSearchVirtualTotalSize = computed(() => savedSearchVirtualizer.value.getTotalSize())

watch(dialogOpen, (open) => {
  if (!open) editingSearch.value = null
})
watch(deleteDialogOpen, (open) => {
  if (!open) deletingSearch.value = null
})

function openCreateDialog(isSmartFolder: boolean) {
  editingSearch.value = null
  const defaults = currentSearchDefaults(isSmartFolder)
  resetForm({ values: defaults })
  searchRuleTree.value = createSavedSearchRuleGroup('all')
  syncRuleTreeFromQuery(defaults.query)
  dialogOpen.value = true
}

function openEditDialog(savedSearch: CommunicationSavedSearch) {
  editingSearch.value = savedSearch
  resetForm({ values: savedSearchFormDefaults(savedSearch) })
  syncRuleTreeFromQuery(savedSearch.query)
  dialogOpen.value = true
}

function openDeleteDialog(savedSearch: CommunicationSavedSearch) {
  deletingSearch.value = savedSearch
  deleteError.value = ''
  deleteDialogOpen.value = true
}

function currentSearchDefaults(isSmartFolder: boolean): SavedSearchFormValues {
  return {
    ...savedSearchFormDefaults(null, isSmartFolder),
    query: props.currentQuery.trim(),
    workflow_state: props.currentWorkflowState || null,
    local_state: props.currentLocalState,
    channel_kind: props.currentChannelKind.trim()
  }
}

function handleSavedSearchPrefetch(savedSearch: CommunicationSavedSearch) {
  void prefetchSavedSearchCommunicationList(savedSearch)
}

function applyPreset(preset: SavedSearchPresetOption) {
  resetForm({
    values: {
      ...formValues,
      ...preset.values,
      match_mode: 'all'
    }
  })
  searchRuleTree.value = createSavedSearchRuleGroup('all')
  syncRuleTreeFromQuery(preset.values.query ?? '')
}

function syncRuleTreeFromQuery(rawQuery: string) {
  const parsed = parseSavedSearchQuery(rawQuery)
  searchRuleTree.value = parsed.tree
  updateFormQuery(parsed.plainQuery)
}

function updateFormQuery(query: string) {
  setFieldValue('query', query)
}

function normalizeQueryIntoBuilder(rawQuery: string) {
  const normalized = normalizeSavedSearchBuilderState(
    rawQuery,
    flattenSavedSearchRuleTree(searchRuleTree.value),
    searchRuleTree.value.matchMode
  )
  searchRuleTree.value = normalized.tree
  updateFormQuery(normalized.plainQuery)
}

function handleSmartFolderVirtualScroll() {
  const scrollEl = smartFolderVirtualScrollRef.value
  if (!scrollEl || !hasNextSmartFolderPage.value || isFetchingNextSmartFolderPage.value) return
  if (scrollEl.scrollLeft + scrollEl.clientWidth >= scrollEl.scrollWidth - 320) {
    void fetchNextSmartFolderPage()
  }
}

function handleSavedSearchVirtualScroll() {
  const scrollEl = savedSearchVirtualScrollRef.value
  if (!scrollEl || !hasNextSavedSearchPage.value || isFetchingNextSavedSearchPage.value) return
  if (scrollEl.scrollLeft + scrollEl.clientWidth >= scrollEl.scrollWidth - 320) {
    void fetchNextSavedSearchPage()
  }
}

const submitSavedSearch = handleSubmit(async (values) => {
  normalizeQueryIntoBuilder(values.query)
  const validation = validateSavedSearchRuleTree(searchRuleTree.value)
  if (!validation.isValid) return
  const request = savedSearchFormToInput(
    {
      ...values,
      query: composeSavedSearchRuleTreeQuery(values.query, searchRuleTree.value)
    },
    props.accountId
  )
  if (editingSearch.value) {
    await updateMutation.mutateAsync({
      savedSearchId: editingSearch.value.saved_search_id,
      request
    })
  } else {
    await createMutation.mutateAsync(request)
  }
  dialogOpen.value = false
})

async function confirmDeleteSavedSearch() {
  const savedSearch = deletingSearch.value
  if (!savedSearch) return

  deleteError.value = ''
  try {
    await deleteMutation.mutateAsync(savedSearch.saved_search_id)
    if (savedSearch.saved_search_id === props.activeId) emit('deleted', savedSearch)
    deleteDialogOpen.value = false
  } catch (e) {
    deleteError.value = e instanceof Error ? e.message : 'Saved search deletion failed'
  }
}
</script>

<template>
  <div class="saved-search-strip">
    <div v-if="isLoading" class="saved-search-skeleton" />
    <template v-else>
      <div v-if="smartFolders.length" class="saved-search-group">
        <span class="saved-search-label">Smart</span>
        <div
          ref="smartFolderVirtualScrollRef"
          class="saved-search-virtual-scroll"
          @scroll="handleSmartFolderVirtualScroll"
        >
          <div class="saved-search-virtual-track" :style="{ width: `${smartFolderVirtualTotalSize}px` }">
            <div
              v-for="virtualItem in virtualSmartFolders"
              :key="String(virtualItem.key)"
              class="saved-search-virtual-row"
              :style="{
                width: `${virtualItem.size}px`,
                transform: `translateX(${virtualItem.start}px)`
              }"
            >
              <div class="saved-search-item">
                <button
                  class="saved-search-chip"
                  :class="{ active: smartFolders[virtualItem.index].saved_search_id === activeId }"
                  type="button"
                  @mouseenter="handleSavedSearchPrefetch(smartFolders[virtualItem.index])"
                  @focus="handleSavedSearchPrefetch(smartFolders[virtualItem.index])"
                  @click="emit('select', smartFolders[virtualItem.index])"
                >
                  <Icon icon="tabler:folder-bolt" class="saved-search-icon" />
                  <span class="saved-search-name">{{ smartFolders[virtualItem.index].name }}</span>
                  <span class="saved-search-count">{{ savedSearchMessageCountLabel(smartFolders[virtualItem.index]) }}</span>
                </button>
                <button class="saved-search-tool" type="button" :title="`Edit ${smartFolders[virtualItem.index].name}`" @click="openEditDialog(smartFolders[virtualItem.index])">
                  <Icon icon="tabler:pencil" class="saved-search-icon" />
                </button>
                <button class="saved-search-tool danger" type="button" :title="`Delete ${smartFolders[virtualItem.index].name}`" @click="openDeleteDialog(smartFolders[virtualItem.index])">
                  <Icon icon="tabler:trash" class="saved-search-icon" />
                </button>
              </div>
            </div>
          </div>
        </div>
        <span v-if="isFetchingNextSmartFolderPage" class="saved-search-loading-more">Loading smart folders...</span>
      </div>
      <div v-if="savedSearches.length" class="saved-search-group">
        <span class="saved-search-label">Saved</span>
        <div
          ref="savedSearchVirtualScrollRef"
          class="saved-search-virtual-scroll"
          @scroll="handleSavedSearchVirtualScroll"
        >
          <div cl
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/TemplateRecipientMappingPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/TemplateRecipientMappingPanel.vue`
- Size bytes / Размер в байтах: `2584`
- Included characters / Включено символов: `2584`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { TemplateRecipientVariableMapping } from './templateLibrary'

const props = defineProps<{
  templateVariables: string[]
  mapping: TemplateRecipientVariableMapping
  summary: string
}>()

const emit = defineEmits<{
  'update:mapping': [mapping: TemplateRecipientVariableMapping]
  fill: []
  buildPreview: []
}>()

function updateMappingField(
  key: keyof TemplateRecipientVariableMapping,
  value: string
): void {
  emit('update:mapping', {
    ...props.mapping,
    [key]: value
  })
}
</script>

<template>
  <div class="template-recipient-mapping">
    <div class="template-recipient-mapping-header">
      <h4>Recipient mapping</h4>
      <span class="template-recipient-mapping-summary">{{ summary }}</span>
    </div>
    <div class="template-recipient-mapping-grid">
      <label>
        <span>To variable</span>
        <select :value="mapping.toVariable" @change="updateMappingField('toVariable', ($event.target as HTMLSelectElement).value)">
          <option value="">Not mapped</option>
          <option v-for="variable in templateVariables" :key="`to:${variable}`" :value="variable">
            {{ variable }}
          </option>
        </select>
      </label>
      <label>
        <span>CC variable</span>
        <select :value="mapping.ccVariable" @change="updateMappingField('ccVariable', ($event.target as HTMLSelectElement).value)">
          <option value="">Not mapped</option>
          <option v-for="variable in templateVariables" :key="`cc:${variable}`" :value="variable">
            {{ variable }}
          </option>
        </select>
      </label>
      <label>
        <span>BCC variable</span>
        <select :value="mapping.bccVariable" @change="updateMappingField('bccVariable', ($event.target as HTMLSelectElement).value)">
          <option value="">Not mapped</option>
          <option v-for="variable in templateVariables" :key="`bcc:${variable}`" :value="variable">
            {{ variable }}
          </option>
        </select>
      </label>
    </div>
    <div class="template-recipient-mapping-actions">
      <Button type="button" variant="ghost" size="sm" @click="emit('fill')">
        <Icon icon="tabler:user-share" size="16" />
        Fill mapped variables
      </Button>
      <Button type="button" variant="ghost" size="sm" @click="emit('buildPreview')">
        <Icon icon="tabler:users-group" size="16" />
        Build rows from To
      </Button>
    </div>
  </div>
</template>
```

### `frontend/src/domains/communications/components/TemplateSaveForm.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/TemplateSaveForm.vue`
- Size bytes / Размер в байтах: `1524`
- Included characters / Включено символов: `1524`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Button from '../../../shared/ui/Button.vue'
import Icon from '../../../shared/ui/Icon.vue'

defineProps<{
  name: string
  nameError: string
  validationMessage: string
  canSave: boolean
  isSaving: boolean
  saveMode: 'new' | 'duplicate'
}>()

const emit = defineEmits<{
  cancel: []
  submit: []
  updateName: [value: string]
}>()
</script>

<template>
  <form class="template-save-form" @submit.prevent="emit('submit')">
    <span class="template-library-preview-meta">
      {{ saveMode === 'duplicate' ? 'Saving a new copy from the current compose content' : 'Saving the current compose content as a reusable template' }}
    </span>
    <label>
      <span>Template name</span>
      <input
        type="text"
        :value="name"
        :aria-invalid="Boolean(nameError)"
        @input="emit('updateName', ($event.target as HTMLInputElement).value)"
      />
    </label>
    <span v-if="nameError" class="template-error">
      {{ nameError }}
    </span>
    <span v-if="validationMessage" class="template-error">
      {{ validationMessage }}
    </span>
    <div class="template-save-actions">
      <Button type="button" variant="ghost" size="sm" @click="emit('cancel')">
        Cancel
      </Button>
      <Button
        type="submit"
        variant="secondary"
        size="sm"
        :disabled="!canSave"
        :loading="isSaving"
      >
        <Icon icon="tabler:device-floppy" size="16" />
        Save current
      </Button>
    </div>
  </form>
</template>
```

### `frontend/src/domains/communications/components/ThreadAttachmentInsightPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ThreadAttachmentInsightPanel.vue`
- Size bytes / Размер в байтах: `14065`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import {
  useAttachmentArchiveInspectionQuery,
  useAttachmentPreviewQuery,
  useTranslateAttachmentMutation
} from '../queries/useCommunicationsQuery'
import type { CommunicationAttachment } from '../types/communications'
import type { AttachmentTranslationResponse } from '../types/attachments'
import {
  formatAttachmentSize,
  isInspectableArchiveAttachment,
  isPreviewableAttachment,
  isPreviewablePdfAttachment,
  isPreviewableImageAttachment,
  scanStatusClass
} from './attachmentTable'

const props = defineProps<{
  attachment: CommunicationAttachment
}>()

const panelMode = ref<'preview' | 'archive' | ''>('')
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
  () => panelMode.value === 'archive' ? props.attachment.attachment_id : null,
  () => panelMode.value === 'archive'
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
  () => panelMode.value === 'preview' ? props.attachment.attachment_id : null,
  () => panelMode.value === 'preview'
)
const attachmentPreview = computed(() => attachmentPreviewData.value)
const attachmentPreviewErrorMessage = computed(() => {
  if (!attachmentPreviewError.value) return ''
  return attachmentPreviewError.value instanceof Error
    ? attachmentPreviewError.value.message
    : 'Attachment preview failed'
})

const canPreviewAttachment = computed(() => isPreviewableAttachment(props.attachment))
const canInspectArchive = computed(() => isInspectableArchiveAttachment(props.attachment))

function openPreview(): void {
  panelMode.value = panelMode.value === 'preview' ? '' : 'preview'
  attachmentTranslationResult.value = null
  attachmentTranslationError.value = ''
}

function openArchiveInspection(): void {
  panelMode.value = panelMode.value === 'archive' ? '' : 'archive'
}

async function translateAttachmentPreview(): Promise<void> {
  const preview = attachmentPreview.value
  if (!preview?.text.trim()) return

  attachmentTranslationError.value = ''
  try {
    attachmentTranslationResult.value = await translateAttachmentMutation.mutateAsync({
      attachmentId: props.attachment.attachment_id,
      request: {
        target_language: attachmentTranslationTarget.value,
        source_text: preview.text
      }
    })
  } catch (error) {
    attachmentTranslationError.value = error instanceof Error
      ? error.message
      : 'Attachment translation failed'
  }
}
</script>

<template>
  <div
    v-if="canPreviewAttachment || canInspectArchive || panelMode"
    class="thread-attachment-insight"
  >
    <div class="thread-attachment-actions">
      <button
        v-if="canPreviewAttachment"
        class="thread-attachment-action"
        type="button"
        @click="openPreview"
      >
        {{
          panelMode === 'preview'
            ? 'Hide preview'
            : (
                isPreviewableImageAttachment(attachment)
                  ? 'Preview image'
                  : isPreviewablePdfAttachment(attachment)
                    ? 'Preview PDF'
                    : 'Preview'
              )
        }}
      </button>
      <button
        v-if="canInspectArchive"
        class="thread-attachment-action"
        type="button"
        @click="openArchiveInspection"
      >
        {{ panelMode === 'archive' ? 'Hide archive' : 'Inspect archive' }}
      </button>
    </div>

    <section
      v-if="panelMode === 'preview'"
      class="thread-attachment-panel"
      aria-label="Thread attachment preview"
    >
      <div class="thread-attachment-panel-header">
        <div>
          <h4>Attachment preview</h4>
          <p>{{ attachment.filename || 'Unnamed attachment' }}</p>
        </div>
        <span class="thread-attachment-scan" :class="scanStatusClass(attachment.scan_status)">
          {{ attachment.scan_status }}
        </span>
      </div>
      <p v-if="isAttachmentPreviewFetching" class="thread-attachment-muted">Loading safe attachment preview...</p>
      <p v-else-if="attachmentPreviewErrorMessage" class="thread-attachment-error">{{ attachmentPreviewErrorMessage }}</p>
      <div v-else-if="attachmentPreview" class="thread-attachment-report">
        <div class="thread-attachment-stats">
          <span>{{ formatAttachmentSize(attachmentPreview.byte_count) }}</span>
          <span v-if="attachmentPreview.truncated">
            Truncated to {{ formatAttachmentSize(attachmentPreview.max_preview_bytes) }}
          </span>
          <span v-if="attachmentPreview.preview_kind === 'image'">Image preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'audio'">Audio preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'video'">Video preview</span>
          <span v-else-if="attachmentPreview.preview_kind === 'pdf'">PDF preview</span>
          <label v-if="attachmentPreview.preview_kind === 'text'" class="thread-attachment-translation-target">
            <span>Translate</span>
            <select v-model="attachmentTranslationTarget">
              <option value="en">EN</option>
              <option value="ru">RU</option>
              <option value="es">ES</option>
            </select>
          </label>
          <button
            v-if="attachmentPreview.preview_kind === 'text'"
            class="thread-attachment-action"
            type="button"
            :disabled="isAttachmentTranslationPending || !attachmentPreview.text.trim()"
            @click="translateAttachmentPreview"
          >
            {{ isAttachmentTranslationPending ? 'Translating' : 'Translate preview' }}
          </button>
        </div>
        <img
          v-if="attachmentPreview.preview_kind === 'image' && attachmentPreview.data_url"
          class="thread-attachment-image"
          :src="attachmentPreview.data_url"
          :alt="attachment.filename || 'Attachment image preview'"
        >
        <audio
          v-else-if="attachmentPreview.preview_kind === 'audio' && attachmentPreview.data_url"
          class="thread-attachment-media"
          controls
          preload="metadata"
          :src="attachmentPreview.data_url"
        />
        <video
          v-else-if="attachmentPreview.preview_kind === 'video' && attachmentPreview.data_url"
          class="thread-attachment-media"
          controls
          preload="metadata"
          :src="attachmentPreview.data_url"
        />
        <iframe
          v-else-if="attachmentPreview.preview_kind === 'pdf' && attachmentPreview.data_url"
          class="thread-attachment-document"
          :src="attachmentPreview.data_url"
          :title="attachment.filename || 'Attachment PDF preview'"
        />
        <pre v-else class="thread-attachment-text">{{ attachmentPreview.text }}</pre>
        <section
          v-if="attachmentTranslationResult || attachmentTranslationError"
          class="thread-attachment-translation"
          aria-label="Thread attachment translation"
        >
          <div class="thread-attachment-panel-header compact">
            <h4>Attachment translation</h4>
            <span v-if="attachmentTranslationResult">
              {{ attachmentTranslationResult.translated ? 'Translated' : 'Fallback' }}
            </span>
          </div>
          <p v-if="attachmentTranslationError" class="thread-attachment-error">{{ attachmentTranslationError }}</p>
          <p v-else-if="attachmentTranslationResult?.text" class="thread-attachment-translation-text">
            {{ attachmentTranslationResult.text }}
          </p>
          <p v-else class="thread-attachment-muted">
            {{ attachmentTranslationResult?.reason ?? 'Translation unavailable' }}
          </p>
        </section>
      </div>
    </section>

    <section
      v-if="panelMode === 'archive'"
      class="thread-attachment-panel"
      aria-label="Thread archive inspection"
    >
      <div class="thread-attachment-panel-header">
        <div>
          <h4>Archive inspection</h4>
          <p>{{ attachment.filename || 'Unnamed archive' }}</p>
        </div>
        <span class="thread-attachment-scan" :class="scanStatusClass(attachment.scan_status)">
          {{ attachment.scan_status }}
        </span>
      </div>
      <p v-if="isArchiveInspectionFetching" class="thread-attachment-muted">Inspecting archive metadata...</p>
      <p v-else-if="archiveInspectionErrorMessage" class="thread-attachment-error">{{ archiveInspectionErrorMessage }}</p>
      <div v-else-if="archiveInspection" class="thread-attachment-report">
        <div class="thread-attachment-stats">
          <span>{{ archiveInspection.report.entry_count }} entries</span>
          <span>{{ formatAttachmentSize(archiveInspection.report.total_uncompressed_bytes) }}</span>
          <span v-if="archiveInspection.report.has_nested_archive">Nested archive</span>
        </div>
        <ul class="thread-attachment-archive-list">
          <li v-for="entry in archiveInspection.report.entries" :key="entry.normalized_path">
            <span>{{ entry.normalized_path }}</span>
            <span>{{ formatAttachmentSize(entry.uncompressed_size) }}</span>
          </li>
        </ul>
      </div>
    </section>
  </div>
</template>

<style scoped>
.thread-attachment-insight {
  display: grid;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.thread-attachment-actions,
.thread-attachment-stats,
.thread-attachment-panel-header,
.thread-attachment-archive-list li {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.thread-attachment-panel-header {
  justify-content: space-between;
}

.thread-attachment-panel-header.compact {
  justify-content: space-between;
}

.thread-attachment-action {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.25rem;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 12%, transparent);
  color: var(--hh-accent, #3b82f6);
  cursor: pointer;
  font: inherit;
  font-size: 0.6875rem;
  padding: 0.1875rem 0.375rem;
  white-space: nowrap;
}

.thread-attachment-action:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.thread-attachment-panel {
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
  padding: 0.75rem;
}

.thread-attachment-panel h4 {
  margin: 0;
  color: var(--hh-text-primary, #1f2937);
  font-size: 0.8125rem;
}

.thread-attachment-panel p,
.thread-attachment-muted,
.thread-attachment-error {
  margin: 0.25rem 0 0;
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.thread-attachment-error {
  color: var(--hh-status-danger-text, #ef4444);
}

.thread-attachment-scan {
  font-size: 0.6875rem;
  font-weight: 500;
  white-space: nowrap;
}

.thread-attachment-report {
  display: grid;
  gap: 0.625rem;
  margin-top: 0.625rem;
}

.thread-attachment-stats {
  color: var(--hh-text-secondary, #6b7280);
  font-size: 0.75rem;
}

.thread-attachment-translation-target {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
}

.thread-attachment-translation-target select {
  min-height: 1.625rem;
  border: 1px solid var(
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/ThreadConversationView.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ThreadConversationView.vue`
- Size bytes / Размер в байтах: `20234`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from '../../../shared/ui/Icon.vue'
import Button from '../../../shared/ui/Button.vue'
import ThreadInlineReplyComposer from './ThreadInlineReplyComposer.vue'
import ThreadAttachmentInsightPanel from './ThreadAttachmentInsightPanel.vue'
import { useTranslateThreadMutation } from '../queries/useCommunicationsQuery'
import type { CommunicationThreadSummary, ThreadMessage } from '../types/communications'
import type { ThreadTranslationResponse } from '../types/multilingual'
import { attachmentIcon } from '../stores/communications'
import { messageTime, senderEmail, senderLabel } from '../stores/communications'
import { formatAttachmentSize, scanStatusClass } from './attachmentTable'
import { previewThreadMessageBody, splitThreadMessageBody } from './threadMessageBody'
import {
  defaultExpandedThreadMessageIds,
  hasQuotedThreadMessages,
  summarizeThreadExpansion
} from './threadConversationPresentation'

const props = defineProps<{
  thread: CommunicationThreadSummary
  messages: ThreadMessage[]
  isLoading: boolean
  errorMessage: string
  isSendingReply: boolean
}>()

const emit = defineEmits<{
  openMessage: [messageId: string]
  replyToMessage: [message: ThreadMessage, bodyHtml: string, draftId: string]
  saveReplyDraft: [message: ThreadMessage, bodyHtml: string, draftId: string]
  sendReply: [message: ThreadMessage, bodyHtml: string, draftId: string]
}>()

const expandedMessageIds = ref<Set<string>>(new Set())
const activeReplyMessageId = ref('')
const activeReplyDraftId = ref('')
const inlineReplyHtml = ref('')
const showQuotedContent = ref(true)
const autoExpandedThreadId = ref('')
const threadTranslationTarget = ref('en')
const threadTranslationResult = ref<ThreadTranslationResponse | null>(null)
const threadTranslationError = ref('')
const translateThreadMutation = useTranslateThreadMutation()
const isTranslatingThread = computed(() => translateThreadMutation.isPending.value)
const canTranslateThread = computed(() => props.messages.length > 0 && !isTranslatingThread.value)
const expansionSummary = computed(() => summarizeThreadExpansion(props.messages, expandedMessageIds.value))
const expandedMessageCount = computed(() => expansionSummary.value.expandedCount)
const hasQuotedMessages = computed(() => hasQuotedThreadMessages(props.messages))
const canExpandAllMessages = computed(() => expansionSummary.value.canExpandAll)
const canCollapseAllMessages = computed(() => expansionSummary.value.canCollapseAll)

const translatedMessages = computed(() => {
  const items = threadTranslationResult.value?.items ?? []
  return new Map(items.map((item) => [item.message_id, item]))
})
const translatedThreadCount = computed(() =>
  threadTranslationResult.value?.items.filter((item) => item.translated).length ?? 0
)

watch(
  () => props.thread.thread_id,
  () => {
    expandedMessageIds.value = new Set()
    autoExpandedThreadId.value = ''
    cancelInlineReply()
    threadTranslationResult.value = null
    threadTranslationError.value = ''
  }
)

watch(
  () => props.messages,
  (messages) => {
    if (messages.length === 0) return
    if (autoExpandedThreadId.value === props.thread.thread_id) return
    expandedMessageIds.value = defaultExpandedThreadMessageIds(messages)
    autoExpandedThreadId.value = props.thread.thread_id
  }
)

function isMessageExpanded(messageId: string): boolean {
  return expandedMessageIds.value.has(messageId)
}

function toggleMessageExpanded(messageId: string): void {
  const next = new Set(expandedMessageIds.value)
  if (next.has(messageId)) {
    next.delete(messageId)
  } else {
    next.add(messageId)
  }
  expandedMessageIds.value = next
}

function expandAllMessages(): void {
  expandedMessageIds.value = new Set(props.messages.map((message) => message.message_id))
}

function collapseAllMessages(): void {
  expandedMessageIds.value = new Set()
}

function startInlineReply(message: ThreadMessage): void {
  activeReplyMessageId.value = message.message_id
  activeReplyDraftId.value = `draft-${Date.now()}`
  inlineReplyHtml.value = ''
}

function cancelInlineReply(): void {
  activeReplyMessageId.value = ''
  activeReplyDraftId.value = ''
  inlineReplyHtml.value = ''
}

function continueReplyInCompose(message: ThreadMessage): void {
  emit('replyToMessage', message, inlineReplyHtml.value, activeReplyDraftId.value)
  cancelInlineReply()
}

function saveInlineReplyDraft(message: ThreadMessage): void {
  if (!activeReplyDraftId.value || !inlineReplyHtml.value.trim()) return
  emit('saveReplyDraft', message, inlineReplyHtml.value, activeReplyDraftId.value)
}

function sendInlineReply(message: ThreadMessage): void {
  emit('sendReply', message, inlineReplyHtml.value, activeReplyDraftId.value)
}

async function handleTranslateThread(): Promise<void> {
  const firstMessage = props.messages[0]
  if (!firstMessage) return

  threadTranslationError.value = ''
  try {
    threadTranslationResult.value = await translateThreadMutation.mutateAsync({
      accountId: firstMessage.account_id,
      subject: firstMessage.subject,
      targetLanguage: threadTranslationTarget.value,
      limit: Math.max(props.messages.length, 1)
    })
  } catch (e) {
    threadTranslationError.value = e instanceof Error ? e.message : 'Thread translation failed'
  }
}

function translatedTextForMessage(messageId: string): string {
  const item = translatedMessages.value.get(messageId)
  if (!item) return ''
  if (item.translated && item.text) return item.text
  return item.reason ?? 'Translation unavailable'
}

function previewBody(message: ThreadMessage): string {
  return previewThreadMessageBody(message, isMessageExpanded(message.message_id))
}

function quotedBody(message: ThreadMessage): string {
  return splitThreadMessageBody(message.body_text).quotedText
}

function primaryBody(message: ThreadMessage): string {
  const segments = splitThreadMessageBody(message.body_text)
  return isMessageExpanded(message.message_id)
    ? (segments.mainText || segments.quotedText)
    : previewBody(message)
}
</script>

<template>
  <section class="thread-conversation">
    <header class="thread-header">
      <div>
        <p class="thread-kicker">Conversation</p>
        <h2>{{ thread.subject }}</h2>
      </div>
      <div class="thread-meta">
        <span>{{ thread.message_count }} messages</span>
        <span>{{ thread.participant_count }} participants</span>
        <span>{{ expandedMessageCount }} expanded</span>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!canExpandAllMessages"
          @click="expandAllMessages"
        >
          Expand all
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="!canCollapseAllMessages"
          @click="collapseAllMessages"
        >
          Collapse all
        </Button>
        <Button
          v-if="hasQuotedMessages"
          variant="ghost"
          size="sm"
          @click="showQuotedContent = !showQuotedContent"
        >
          {{ showQuotedContent ? 'Hide quoted' : 'Show quoted' }}
        </Button>
        <label class="thread-translation-target">
          <span>Translate</span>
          <select v-model="threadTranslationTarget">
            <option value="en">EN</option>
            <option value="ru">RU</option>
            <option value="es">ES</option>
          </select>
        </label>
        <Button
          variant="outline"
          size="sm"
          icon="tabler:language"
          :loading="isTranslatingThread"
          :disabled="!canTranslateThread"
          @click="handleTranslateThread"
        >
          Translate
        </Button>
      </div>
    </header>

    <section v-if="threadTranslationResult" class="thread-translation-panel">
      <div>
        <p class="thread-kicker">Thread translation review</p>
        <strong>{{ threadTranslationResult.items.length }} messages to {{ threadTranslationResult.target_language }}</strong>
      </div>
      <span>{{ translatedThreadCount }} translated</span>
    </section>
    <div v-if="threadTranslationError" class="thread-state error compact">
      <Icon icon="tabler:alert-circle" />
      <span>{{ threadTranslationError }}</span>
    </div>

    <div v-if="errorMessage" class="thread-state error">
      <Icon icon="tabler:alert-circle" />
      <span>{{ errorMessage }}</span>
    </div>
    <div v-else-if="isLoading" class="thread-state">
      <Icon icon="tabler:loader-2" class="spin-icon" />
      <span>Loading conversation...</span>
    </div>
    <div v-else-if="messages.length === 0" class="thread-state">
      <Icon icon="tabler:messages" />
      <span>No messages in this conversation</span>
    </div>
    <ol v-else class="thread-timeline">
      <li
        v-for="message in messages"
        :key="message.message_id"
        class="thread-message"
      >
        <div class="message-marker" />
        <article class="message-card">
          <header class="message-header">
            <div class="sender-block">
              <strong>{{ senderLabel(message.sender) }}</strong>
              <span>{{ senderEmail(message.sender) }}</span>
            </div>
            <div class="message-actions">
              <span class="message-time">{{ messageTime(message.projected_at ?? message.occurred_at) }}</span>
              <Button
                variant="ghost"
                size="sm"
                title="Reply"
                aria-label="Reply to message"
                @click="startInlineReply(message)"
              >
                <Icon icon="tabler:corner-up-left" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                :title="isMessageExpanded(message.message_id) ? 'Collapse' : 'Expand'"
                :aria-label="isMessageExpanded(message.message_id) ? 'Collapse message' : 'Expand message'"
                @click="toggleMessageExpanded(message.message_id)"
              >
                <Icon :icon="isMessageExpanded(message.message_id) ? 'tabler:chevron-up' : 'tabler:chevron-down'" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                title="Open message"
                aria-label="Open full message"
                @click="emit('openMessage', message.message_id)"
              >
                <Icon icon="tabler:mail-opened" />
              </Button>
            </div>
          </header>
          <p
            class="message-body"
            :class="{ collapsed: !isMessageExpanded(message.message_id) }"
          >
            {{ primaryBody(message) }}
          </p>
          <blockquote
            v-if="isMessageExpanded(message.message_id) && showQuotedContent && quotedBody(message)"
            class="message-quoted"
          >
            {{ quotedBody(message) }}
          </blockquote>
          <ul
            v-if="isMessageExpanded(message.message_id) && message.attachments.length > 0"
            class="message-attachments"
            aria-label="Thread message attachments"
          >
            <li
              v-for="attachment in message.attachments"
              :key="attachment.attachment_id"
              class="message-attachment"
            >
              <Icon :icon="attachmentIcon(attachment.content_type)" class="message-attachment-icon" />
              <div class="message-attachment-copy">
                <strong>{{ attachment.filename || 'Unnamed attachment' }}</strong>
                <span>{{ formatAttachmentSize(attachment.size_bytes) }} · {{ attachment.content_type }}</span>
              </div>
              <span class="message-attachment-scan" :class="scanStatusClass(attachment.scan_status)">
                {{ attachment.scan_status }}
              </span>
              <ThreadAttachmentInsightPanel :attachment="attachment" />
            </li>
          </ul>
          <div

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/components/ThreadInlineReplyComposer.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ThreadInlineReplyComposer.vue`
- Size bytes / Размер в байтах: `4923`
- Included characters / Включено символов: `4923`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref } from 'vue'
import Button from '../../../shared/ui/Button.vue'
import RichComposeEditor from './RichComposeEditor.vue'
import type { ThreadMessage } from '../types/communications'
import { senderEmail, senderLabel } from '../stores/communications'

const props = defineProps<{
  message: ThreadMessage
  bodyHtml: string
  isSendingReply: boolean
}>()

const emit = defineEmits<{
  'update:bodyHtml': [bodyHtml: string]
  cancel: []
  saveDraft: []
  continueInCompose: []
  send: []
}>()

const reviewingReply = ref(false)

function updateBodyHtml(bodyHtml: string): void {
  emit('update:bodyHtml', bodyHtml)
  if (!bodyHtml.trim()) {
    reviewingReply.value = false
  }
}

function openSendReview(): void {
  if (!props.bodyHtml.trim()) return
  reviewingReply.value = true
}

function closeSendReview(): void {
  reviewingReply.value = false
}

function confirmSend(): void {
  if (!props.bodyHtml.trim()) return
  reviewingReply.value = false
  emit('send')
}

function replyReviewRecipient(message: ThreadMessage): string {
  const label = senderLabel(message.sender)
  const email = senderEmail(message.sender)
  return label && label !== email ? `${label} <${email}>` : email
}

function replyReviewSubject(message: ThreadMessage): string {
  return message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`
}
</script>

<template>
  <div class="inline-reply">
    <div class="inline-reply-header">
      <span>Replying to {{ senderLabel(message.sender) }}</span>
      <button type="button" @click="emit('cancel')">Discard</button>
    </div>
    <RichComposeEditor
      :model-value="bodyHtml"
      placeholder="Write a reply..."
      @update:model-value="updateBodyHtml"
      @blur="emit('saveDraft')"
    />
    <div class="inline-reply-actions">
      <Button variant="secondary" size="sm" @click="emit('cancel')">
        Cancel
      </Button>
      <Button
        variant="secondary"
        size="sm"
        :disabled="!bodyHtml.trim()"
        @click="emit('saveDraft')"
      >
        Save Draft
      </Button>
      <Button variant="secondary" size="sm" @click="emit('continueInCompose')">
        Continue in Compose
      </Button>
      <Button
        variant="default"
        size="sm"
        :disabled="!bodyHtml.trim() || isSendingReply"
        @click="openSendReview"
      >
        Review & Send
      </Button>
    </div>
    <div v-if="reviewingReply" class="inline-send-review">
      <div class="review-title">Review reply before sending</div>
      <div class="review-grid">
        <span>To</span>
        <strong>{{ replyReviewRecipient(message) }}</strong>
        <span>Subject</span>
        <strong>{{ replyReviewSubject(message) }}</strong>
        <span>Delivery</span>
        <strong>Immediate provider send</strong>
        <span>Undo</span>
        <strong>Off</strong>
      </div>
      <div class="review-actions">
        <Button variant="default" size="sm" :disabled="isSendingReply" @click="confirmSend">
          {{ isSendingReply ? 'Sending...' : 'Send' }}
        </Button>
        <Button variant="ghost" size="sm" @click="closeSendReview">
          Edit
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inline-reply {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  margin-top: 0.875rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #e5e7eb);
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-bg-secondary, #f9fafb) 82%, transparent);
}

.inline-reply :deep(.rich-compose-editor) {
  min-height: 160px;
}

.inline-reply :deep(.rich-compose-surface) {
  min-height: 120px;
}

.inline-reply-header,
.inline-reply-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.inline-reply-header {
  font-size: 0.75rem;
  color: var(--hh-text-secondary, #6b7280);
}

.inline-reply-header button {
  border: 0;
  padding: 0;
  background: transparent;
  color: var(--hh-accent, #3b82f6);
  cursor: pointer;
  font: inherit;
}

.inline-reply-actions {
  justify-content: flex-end;
}

.inline-send-review {
  display: grid;
  gap: 0.625rem;
  padding: 0.75rem;
  border: 1px solid color-mix(in srgb, var(--hh-accent, #3b82f6) 32%, var(--hh-border, #e5e7eb));
  border-radius: 8px;
  background: color-mix(in srgb, var(--hh-accent, #3b82f6) 8%, var(--hh-bg-primary, #ffffff));
}

.review-title {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--hh-text-primary, #1f2937);
}

.review-grid {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 0.375rem 0.75rem;
  font-size: 0.75rem;
}

.review-grid span {
  color: var(--hh-text-secondary, #6b7280);
}

.review-grid strong {
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--hh-text-primary, #1f2937);
}

.review-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
```

### `frontend/src/domains/communications/providers/telegram/views/TelegramCommunicationsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/providers/telegram/views/TelegramCommunicationsPanel.vue`
- Size bytes / Размер в байтах: `12337`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import type { TelegramChat, TelegramMessage } from '../../../../../shared/communications/types/telegram'
import {
  useDeleteTelegramMessageMutation,
  useEditTelegramMessageMutation,
  usePinTelegramMessageMutation,
  useReplyTelegramMessageMutation,
  useSendTelegramMessageMutation,
  useTelegramChatsQuery,
  useTelegramMessageSearchQuery,
  useTelegramMessagesQuery,
} from '../../../queries/telegramBusinessQueries'

const { t } = useI18n()
const selectedConversationId = ref('')
const draftText = ref('')
const searchText = ref('')
const actionMessage = ref('')
const actionError = ref('')

const chatsQuery = useTelegramChatsQuery(undefined, 500)
const chats = computed(() => chatsQuery.data.value ?? [])
const selectedChat = computed<TelegramChat | null>(
  () => chats.value.find((chat) => chat.provider_chat_id === selectedConversationId.value) ?? chats.value[0] ?? null
)
const messagesQuery = useTelegramMessagesQuery(
  () => selectedChat.value?.account_id ?? null,
  () => selectedChat.value?.provider_chat_id ?? null,
  100
)
const searchQuery = useTelegramMessageSearchQuery({
  q: searchText,
  accountId: () => selectedChat.value?.account_id ?? null,
  providerChatId: () => selectedChat.value?.provider_chat_id ?? null,
  limit: 50,
})
const sendMutation = useSendTelegramMessageMutation()
const replyMutation = useReplyTelegramMessageMutation()
const editMutation = useEditTelegramMessageMutation()
const deleteMutation = useDeleteTelegramMessageMutation()
const pinMutation = usePinTelegramMessageMutation()
const messages = computed(() => messagesQuery.data.value ?? [])
const visibleMessages = computed(() =>
  searchText.value.trim()
    ? (searchQuery.data.value?.items ?? [])
    : messages.value
)
const isBusy = computed(() =>
  sendMutation.isPending.value ||
  replyMutation.isPending.value ||
  editMutation.isPending.value ||
  deleteMutation.isPending.value ||
  pinMutation.isPending.value
)

watch(
  chats,
  (items) => {
    if (!items.length) {
      selectedConversationId.value = ''
      return
    }
    if (!items.some((item) => item.provider_chat_id === selectedConversationId.value)) {
      selectedConversationId.value = items[0]?.provider_chat_id ?? ''
    }
  },
  { immediate: true }
)

function messageTime(message: TelegramMessage): string {
  const value = message.occurred_at ?? message.projected_at
  if (!value) return ''
  const date = new Date(value)
  return Number.isNaN(date.getTime())
    ? ''
    : new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)
}

function messagePreview(chat: TelegramChat): string {
  const latest = messages.value
    .filter((message) => message.provider_chat_id === chat.provider_chat_id)
    .at(-1)
  return latest?.text || chat.sync_state
}

function requireProviderChatId(message: TelegramMessage): string | null {
  if (message.provider_chat_id) return message.provider_chat_id
  actionError.value = t('Message is missing provider conversation metadata')
  return null
}

async function sendMessage() {
  const chat = selectedChat.value
  const text = draftText.value.trim()
  if (!chat || !text || isBusy.value) return
  actionMessage.value = ''
  actionError.value = ''
  try {
    const result = await sendMutation.mutateAsync({
      account_id: chat.account_id,
      provider_chat_id: chat.provider_chat_id,
      text,
    })
    draftText.value = ''
    actionMessage.value = `Telegram message ${result.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function replyToMessage(message: TelegramMessage) {
  const text = draftText.value.trim()
  if (!text || isBusy.value) return
  actionMessage.value = ''
  actionError.value = ''
  try {
    const result = await replyMutation.mutateAsync({
      message_id: message.message_id,
      text,
    })
    draftText.value = ''
    actionMessage.value = `Telegram reply ${result.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function editMessage(message: TelegramMessage) {
  const nextText = window.prompt(t('Edit message'), message.text)
  if (nextText === null || !nextText.trim() || isBusy.value) return
  const providerChatId = requireProviderChatId(message)
  if (!providerChatId) return
  try {
    await editMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: providerChatId,
      provider_message_id: message.provider_message_id,
      new_text: nextText.trim(),
    })
    actionMessage.value = t('Message edited')
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function deleteMessage(message: TelegramMessage) {
  if (isBusy.value) return
  const providerChatId = requireProviderChatId(message)
  if (!providerChatId) return
  try {
    await deleteMutation.mutateAsync({
      message_id: message.message_id,
      account_id: message.account_id,
      provider_chat_id: providerChatId,
      provider_message_id: message.provider_message_id,
      reason_class: 'deleted_by_owner',
      actor_class: 'owner',
      is_provider_delete: false,
    })
    actionMessage.value = t('Message deleted locally')
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function togglePin(message: TelegramMessage) {
  if (isBusy.value) return
  try {
    const result = await pinMutation.mutateAsync({
      message_id: message.message_id,
    })
    actionMessage.value = result.pinned ? t('Message pinned') : t('Message unpinned')
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}
</script>

<template>
  <section class="telegram-communications-panel communications-page">
    <header class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small">
          <Icon icon="tabler:brand-telegram" width="28" height="28" />
        </span>
        <div>
          <h1>{{ t('Telegram') }}</h1>
          <p>{{ t('Projected Communication conversations and messages') }}</p>
        </div>
      </div>
      <label class="provider-search">
        <Icon icon="tabler:search" width="16" height="16" />
        <input v-model="searchText" type="search" :placeholder="t('Search messages')" />
      </label>
    </header>

    <p v-if="actionMessage" class="setup-state success">{{ actionMessage }}</p>
    <p v-if="actionError" class="inline-error">{{ actionError }}</p>

    <div class="three-pane communications-grid telegram-grid">
      <section class="panel conversation-list">
        <header class="provider-panel-header">
          <h2>{{ t('Conversations') }}</h2>
          <button type="button" :disabled="chatsQuery.isFetching.value" @click="chatsQuery.refetch()">
            <Icon icon="tabler:refresh" width="16" height="16" />
          </button>
        </header>
        <div class="provider-list-scroll">
          <div v-if="chatsQuery.isLoading.value" class="empty-panel">{{ t('Loading Telegram conversations...') }}</div>
          <button
            v-for="chat in chats"
            :key="chat.provider_chat_id"
            type="button"
            class="provider-row"
            :class="{ active: selectedConversationId === chat.provider_chat_id }"
            @click="selectedConversationId = chat.provider_chat_id"
          >
            <strong>{{ chat.title }}</strong>
            <span>{{ messagePreview(chat) }}</span>
          </button>
        </div>
      </section>

      <section class="panel chat-pane">
        <header class="provider-thread-header">
          <div>
            <h2>{{ selectedChat?.title ?? t('No conversation selected') }}</h2>
            <p>{{ selectedChat?.account_id ?? '' }}</p>
          </div>
        </header>
        <div class="message-scroll">
          <article v-for="message in visibleMessages" :key="message.message_id" class="message-bubble">
            <header>
              <strong>{{ message.sender_display_name ?? message.sender }}</strong>
              <time>{{ messageTime(message) }}</time>
            </header>
            <p>{{ message.text }}</p>
            <footer>
              <button type="button" :disabled="isBusy || !draftText.trim()" @click="replyToMessage(message)">
                <Icon icon="tabler:message-reply" width="14" height="14" />{{ t('Reply') }}
              </button>
              <button type="button" :disabled="isBusy" @click="editMessage(message)">
                <Icon icon="tabler:edit" width="14" height="14" />{{ t('Edit') }}
              </button>
              <button type="button" :disabled="isBusy" @click="togglePin(message)">
                <Icon icon="tabler:pin" width="14" height="14" />{{ t('Pin') }}
              </button>
              <button type="button" :disabled="isBusy" @click="deleteMessage(message)">
                <Icon icon="tabler:trash" width="14" height="14" />{{ t('Delete') }}
              </button>
            </footer>
          </article>
          <div v-if="!visibleMessages.length" class="empty-panel">{{ t('No projected Telegram messages yet.') }}</div>
        </div>
        <form class="provider-inline-form" @submit.prevent="sendMessage">
          <input v-model="draftText" type="text" :placeholder="t('Write a message')" autocomplete="off" />
          <button type="submit" :disabled="isBusy || !selectedChat || !draftText.trim()">
            <Icon icon="tabler:send" width="16" height="16" />{{ t('Send') }}
          </button>
        </form>
      </section>
    </div>
  </section>
</template>

<style scoped>
.telegram-communications-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
.view-header,
.view-title-with-icon,
.provider-search,
.provider-panel-header,
.provider-thread-header,
.message-bubble header,
.message-bubble footer,
.provider-inline-form {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header {
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.provider-search,
.provider-inline-form {
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  padding: 0.4rem 0.6rem;
  background: var(--hh-bg-primary, #fff);
}
.provider-search input,
.provider-inline-form input {
  border: 0;
  outline: 0;
  min-width: 220px;
  background: transparent;
  color: inherit;
}
.provider-panel-header,
.provider-thread-header {
  justify-content: space-between;
  padding: 0.75rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.provider-list-scroll,
.message-scroll {
  flex: 1;
  overflow: auto;
  min-height: 0;
}
.provider-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  width: 100%;
  padding: 0.7rem 0.85rem;
  border: 0;
  border-bottom: 1px solid var(--hh-border, #eef2f6);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.provider-row.active,
.provider-row:hover {
  background: var(--hh-bg-muted, #f5f8fb);
}
.provider-row span,
.provider-thread-header p,
.message-bubble time {
  color: var(--hh-text-muted, #667085);
  font-size: 0.78rem;
}
.message-bubble {
  margin: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-primary, #fff);
}
.message-bubble header,
.message-bubble footer {
  justify-content: space-between;
}
.message-bubble footer {
  justify-content: flex-start;
}
.message-bubble button,
.provider-panel-header button,
.provider-inline-form button {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 6px;

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

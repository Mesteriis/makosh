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

- Chunk ID / ID чанка: `149-source-frontend-part-009`
- Group / Группа: `frontend`
- Role / Роль: `source`
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

### `frontend/src/domains/communications/views/useThreadReplyActions.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/useThreadReplyActions.ts`
- Size bytes / Размер в байтах: `2140`
- Included characters / Включено символов: `2140`
- Truncated / Обрезано: `no`

```typescript
import { computed } from 'vue'
import {
  useSaveDraftMutation,
  useSendMailMutation
} from '../queries/useCommunicationsQuery'
import { buildComposeDraftPayload } from '../forms/composeDraftAutosave'
import {
  composeFormToSendRequest,
  threadReplyComposeForm
} from '../helpers/communicationPageModels'
import type { ThreadMessage } from '../types/communications'
import type { useCommunicationsStore } from '../stores/communications'

type CommunicationsStore = ReturnType<typeof useCommunicationsStore>

export function useThreadReplyActions(store: CommunicationsStore) {
  const saveDraftMutation = useSaveDraftMutation()
  const sendMailMutation = useSendMailMutation()
  const isThreadReplySending = computed(() => sendMailMutation.isPending.value)

  function handleReplyToThreadMessage(message: ThreadMessage, bodyHtml: string, draftId: string) {
    store.openCompose(threadReplyComposeForm(message, store.selectedMailAccountId, draftId || `draft-${Date.now()}`, bodyHtml))
  }

  async function handleSaveThreadReplyDraft(message: ThreadMessage, bodyHtml: string, draftId: string) {
    if (!bodyHtml.trim()) return
    try {
      await saveDraftMutation.mutateAsync(buildComposeDraftPayload(
        threadReplyComposeForm(message, store.selectedMailAccountId, draftId, bodyHtml)
      ))
      store.setMailActionStatus('Draft saved')
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Save draft failed')
    }
  }

  async function handleSendThreadReply(message: ThreadMessage, bodyHtml: string, draftId: string) {
    if (!bodyHtml.trim()) return
    try {
      const form = threadReplyComposeForm(message, store.selectedMailAccountId, draftId || `draft-${Date.now()}`, bodyHtml)
      const result = await sendMailMutation.mutateAsync(composeFormToSendRequest(form))
      store.setMailActionStatus(`Sent via ${result.transport ?? 'provider'}`)
    } catch (e) {
      store.setMailActionError(e instanceof Error ? e.message : 'Send failed')
    }
  }

  return {
    handleReplyToThreadMessage,
    handleSaveThreadReplyDraft,
    handleSendThreadReply,
    isThreadReplySending
  }
}
```

### `frontend/src/domains/documents/api/documents.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/api/documents.ts`
- Size bytes / Размер в байтах: `1235`
- Included characters / Включено символов: `1235`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  DocumentProcessingRecord,
  DocumentProcessingJobsResponse,
  DocumentProcessingRetryRequest,
  DocumentProcessingRetryResponse
} from '../types/documents'

export async function fetchDocumentProcessing(documentId: string): Promise<DocumentProcessingRecord> {
  return ApiClient.instance.get<DocumentProcessingRecord>(
    `/api/v1/documents/${encodeURIComponent(documentId)}/processing`,
    'Document processing request failed'
  )
}

export async function fetchDocumentProcessingJobs(limit = 50): Promise<DocumentProcessingJobsResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<DocumentProcessingJobsResponse>(
    `/api/v1/document-processing/jobs?${params.toString()}`,
    'Document processing jobs request failed'
  )
}

export async function retryDocumentProcessingJob(
  jobId: string,
  request: DocumentProcessingRetryRequest
): Promise<DocumentProcessingRetryResponse> {
  return ApiClient.instance.post<DocumentProcessingRetryResponse>(
    `/api/v1/document-processing/jobs/${encodeURIComponent(jobId)}/retry`,
    request,
    'Document processing retry request failed'
  )
}
```

### `frontend/src/domains/documents/queries/useDocumentsQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/queries/useDocumentsQuery.ts`
- Size bytes / Размер в байтах: `304`
- Included characters / Включено символов: `304`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchDocumentProcessingJobs } from '../api/documents'

export function useDocumentProcessingJobsQuery(limit = 50) {
  return useQuery({
    queryKey: ['document-processing-jobs', limit],
    queryFn: () => fetchDocumentProcessingJobs(limit)
  })
}
```

### `frontend/src/domains/documents/stores/documents.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/stores/documents.ts`
- Size bytes / Размер в байтах: `779`
- Included characters / Включено символов: `779`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useDocumentsStore = defineStore('documents-ui', () => {
  const searchQuery = ref('')
  const activeFilter = ref<string>('all')
  const documentsError = ref('')
  const retryingJobId = ref<string | null>(null)

  function setSearchQuery(q: string) {
    searchQuery.value = q
  }

  function setActiveFilter(filter: string) {
    activeFilter.value = filter
  }

  function setDocumentsError(err: string) {
    documentsError.value = err
  }

  function setRetryingJobId(id: string | null) {
    retryingJobId.value = id
  }

  return {
    searchQuery,
    activeFilter,
    documentsError,
    retryingJobId,
    setSearchQuery,
    setActiveFilter,
    setDocumentsError,
    setRetryingJobId
  }
})
```

### `frontend/src/domains/documents/types/documents.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/types/documents.ts`
- Size bytes / Размер в байтах: `1500`
- Included characters / Включено символов: `1500`
- Truncated / Обрезано: `no`

```typescript
export type DocumentProcessingStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'skipped'

export type DocumentProcessingStep = 'extract_text' | 'ocr'

export type DocumentProcessingArtifactKind = 'extracted_text' | 'ocr_text'

export interface DocumentProcessingJob {
  job_id: string
  document_id: string
  step: DocumentProcessingStep
  status: DocumentProcessingStatus
  attempts: number
  max_attempts: number
  last_error_summary: string | null
  queued_at: string
  started_at: string | null
  finished_at: string | null
  created_at: string
  updated_at: string
}

export interface DocumentProcessingArtifact {
  artifact_id: string
  document_id: string
  job_id: string
  artifact_kind: DocumentProcessingArtifactKind
  content_sha256: string
  text_content: string | null
  storage_kind: string | null
  storage_path: string | null
  metadata: Record<string, unknown>
  created_at: string
}

export interface DocumentProcessingRecord {
  document_id: string
  jobs: DocumentProcessingJob[]
  artifacts: DocumentProcessingArtifact[]
}

export interface DocumentProcessingJobsResponse {
  items: DocumentProcessingJob[]
}

export interface DocumentProcessingRetryRequest {
  command_id: string
}

export interface DocumentProcessingRetryResponse {
  job_id: string
  status: DocumentProcessingStatus
  event_id: string
}

export interface DocDisplayItem {
  name: string
  source: string
  project: string
  type: string
  date: string
  size: string
  icon: string
  tone: string
}
```

### `frontend/src/domains/home/api/home.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/api/home.ts`
- Size bytes / Размер в байтах: `728`
- Included characters / Включено символов: `728`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type { CommunicationMessagesResponse } from '../types/api'
import type { MailboxHealth } from '../types/api'

export async function fetchCommunicationMessages(limit = 50): Promise<CommunicationMessagesResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<CommunicationMessagesResponse>(
    `/api/v1/communications/messages?${params.toString()}`,
    'Communication messages request failed'
  )
}

export async function fetchMailboxHealth(): Promise<MailboxHealth> {
  return ApiClient.instance.get<MailboxHealth>(
    '/api/v1/communications/analytics/health',
    'Health request failed'
  )
}
```

### `frontend/src/domains/home/queries/useHomeQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/queries/useHomeQuery.ts`
- Size bytes / Размер в байтах: `652`
- Included characters / Включено символов: `652`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchCommunicationMessages, fetchMailboxHealth } from '../api/home'
import type { CommunicationMessageSummary, MailboxHealth } from '../types/api'

export function useCommunicationMessagesQuery(limit = 50) {
  return useQuery<CommunicationMessageSummary[]>({
    queryKey: ['home', 'communication-messages', limit],
    queryFn: async () => {
      const res = await fetchCommunicationMessages(limit)
      return res.items
    }
  })
}

export function useMailboxHealthQuery() {
  return useQuery<MailboxHealth>({
    queryKey: ['home', 'mailbox-health'],
    queryFn: fetchMailboxHealth
  })
}
```

### `frontend/src/domains/home/types/api.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/types/api.ts`
- Size bytes / Размер в байтах: `967`
- Included characters / Включено символов: `967`
- Truncated / Обрезано: `no`

```typescript
export type CommunicationMessageSummary = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_record_id: string
  subject: string
  sender: string
  recipients: string[]
  body_text_preview: string
  occurred_at: string | null
  projected_at: string
  channel_kind: string
  conversation_id: string | null
  sender_display_name: string | null
  delivery_state: string
  message_metadata: Record<string, unknown>
  attachment_count: number
  local_state: LocalMessageState
  local_state_changed_at: string | null
}

export type LocalMessageState = 'active' | 'trash' | 'all'

export type CommunicationMessagesResponse = {
  items: CommunicationMessageSummary[]
}

export type MailboxHealth = {
  total_messages: number
  unread: number
  needs_action: number
  waiting: number
  done: number
  archived: number
  spam: number
  important: number
  with_attachments: number
  average_importance: number
  oldest_message_days: number | null
}
```

### `frontend/src/domains/home/types/home.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/types/home.ts`
- Size bytes / Размер в байтах: `630`
- Included characters / Включено символов: `630`
- Truncated / Обрезано: `no`

```typescript
export interface StatCard {
  label: string
  value: string
  delta: string
  icon: string
  tone?: string
}

export interface FeedItem {
  icon: string
  title: string
  meta: string
  time: string
  tag?: string
  tone?: string
}

export interface TaskItem {
  title: string
  assignee: string
  due: string
  priority: string
}

export interface PersonItem {
  name: string
  meta: string
  icon: string
}

export interface ProjectItem {
  name: string
  kind: string
  progress: number
  icon: string
  tone: string
}

export interface SystemStatusItem {
  label: string
  value: string
  status: 'ok' | 'warning' | 'error'
}
```

### `frontend/src/domains/knowledge/api/knowledge.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/api/knowledge.ts`
- Size bytes / Размер в байтах: `2061`
- Included characters / Включено символов: `2061`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
	GraphSummary,
	GraphNode,
	GraphNeighborhood,
	ContradictionListResponse,
	ContradictionObservation,
	ContradictionReviewRequest
} from '../types/knowledge'

export async function fetchGraphSummary(): Promise<GraphSummary> {
	return ApiClient.instance.get<GraphSummary>('/api/v1/graph/summary', 'Graph summary request failed')
}

export async function fetchGraphNodes(limit = 20): Promise<GraphNode[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
	return ApiClient.instance.get<GraphNode[]>(
		`/api/v1/graph/nodes?${params.toString()}`,
		'Graph node picker request failed'
	)
}

export async function searchGraphNodes(query: string, limit = 20): Promise<GraphNode[]> {
	const normalizedQuery = query.trim()
	if (!normalizedQuery) {
		return []
	}
	const params = new URLSearchParams({
		q: normalizedQuery,
		limit: String(Math.trunc(limit))
	})
	return ApiClient.instance.get<GraphNode[]>(
		`/api/v1/graph/search?${params.toString()}`,
		'Graph search request failed'
	)
}

export async function fetchGraphNeighborhood(nodeId: string, depth = 1): Promise<GraphNeighborhood> {
	const params = new URLSearchParams({
		node_id: nodeId,
		depth: String(depth)
	})
	return ApiClient.instance.get<GraphNeighborhood>(
		`/api/v1/graph/neighborhood?${params.toString()}`,
		'Graph neighborhood request failed'
	)
}

export async function fetchContradictions(limit = 50): Promise<ContradictionListResponse> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
	return ApiClient.instance.get<ContradictionListResponse>(
		`/api/v1/contradictions?${params.toString()}`,
		'Contradictions request failed'
	)
}

export async function reviewContradiction(
	observationId: string,
	request: ContradictionReviewRequest
): Promise<ContradictionObservation> {
	return ApiClient.instance.put<ContradictionObservation>(
		`/api/v1/contradictions/${encodeURIComponent(observationId)}/review`,
		request,
		'Contradiction review request failed'
	)
}
```

### `frontend/src/domains/knowledge/queries/useKnowledgeQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/queries/useKnowledgeQuery.ts`
- Size bytes / Размер в байтах: `536`
- Included characters / Включено символов: `536`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchGraphSummary, fetchContradictions } from '../api/knowledge'

export function useGraphSummaryQuery() {
	return useQuery({
		queryKey: ['graph-summary'],
		queryFn: async () => {
			const summary = await fetchGraphSummary()
			return summary
		}
	})
}

export function useContradictionsQuery(limit = 50) {
	return useQuery({
		queryKey: ['contradictions', limit],
		queryFn: async () => {
			const response = await fetchContradictions(limit)
			return response.items
		}
	})
}
```

### `frontend/src/domains/knowledge/stores/knowledge.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/stores/knowledge.ts`
- Size bytes / Размер в байтах: `8317`
- Included characters / Включено символов: `8316`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
	GraphSummary,
	GraphNode,
	GraphNeighborhood,
	GraphEdge,
	ContradictionObservation,
	ContradictionSeverity,
	GraphNodeKind
} from '../types/knowledge'
import {
	fetchGraphNodes,
	searchGraphNodes,
	fetchGraphNeighborhood,
	reviewContradiction
} from '../api/knowledge'

export interface GraphCanvasNode {
	node_id: string
	node_kind: GraphNodeKind
	label: string
	x: number
	y: number
	isSelected: boolean
	layoutClass: string
}

export interface GraphCanvasEdge {
	x1: number
	y1: number
	x2: number
	y2: number
	label: string
	review_state: string
}

export type GraphFilterChip = {
	kind: string
	label: string
	icon: string
	count: number
}

const RADIUS = 38

function buildRadialLayout(
	center: GraphNode,
	neighbors: GraphNode[],
	radius: number
): GraphCanvasNode[] {
	const maxNeighbors = 14
	const nodes: GraphCanvasNode[] = [
		{
			node_id: center.node_id,
			node_kind: center.node_kind,
			label: center.label,
			x: 50,
			y: 50,
			isSelected: true,
			layoutClass: 'center'
		}
	]

	const limited = neighbors.slice(0, maxNeighbors)
	const count = limited.length
	for (let i = 0; i < count; i++) {
		const angle = (2 * Math.PI * i) / count - Math.PI / 2
		nodes.push({
			node_id: limited[i].node_id,
			node_kind: limited[i].node_kind,
			label: limited[i].label,
			x: 50 + radius * Math.cos(angle),
			y: 50 + radius * Math.sin(angle),
			isSelected: false,
			layoutClass: `neighbor-${i}`
		})
	}
	return nodes
}

function buildEdges(
	centerId: string,
	edges: GraphEdge[],
	canvasNodes: GraphCanvasNode[]
): GraphCanvasEdge[] {
	const nodeMap = new Map(canvasNodes.map((n) => [n.node_id, n]))
	return edges.map((edge) => {
		const source = nodeMap.get(edge.source_node_id)
		const target = nodeMap.get(edge.target_node_id)
		return {
			x1: source?.x ?? 50,
			y1: source?.y ?? 50,
			x2: target?.x ?? 50,
			y2: target?.y ?? 50,
			label: edge.relationship_type.replace(/_/g, ' '),
			review_state: edge.review_state
		}
	})
}

export function graphNodeKindIcon(kind: string): string {
	switch (kind) {
		case 'person':
			return 'tabler:user'
		case 'email_address':
			return 'tabler:mail'
		case 'message':
			return 'tabler:message'
		case 'document':
			return 'tabler:file'
		case 'project':
			return 'tabler:folder'
		case 'organization':
			return 'tabler:building'
		case 'task':
			return 'tabler:checkbox'
		case 'event':
			return 'tabler:calendar'
		case 'decision':
			return 'tabler:scale'
		case 'obligation':
			return 'tabler:gavel'
		case 'knowledge':
			return 'tabler:brain'
		default:
			return 'tabler:circle'
	}
}

export function graphNodeKindLabel(kind: string): string {
	return kind
		.split('_')
		.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
		.join(' ')
}

export function contradictionSeverityTone(severity: ContradictionSeverity): ContradictionSeverity {
	return severity
}

export function formatContradictionClaim(observation: ContradictionObservation): string {
	return `${observation.old_claim} -> ${observation.new_claim}`
}

export function formatContradictionTime(value: string): string {
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) {
		return 'Unknown date'
	}
	return new Intl.DateTimeFormat('en', {
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	}).format(date)
}

export function formatContradictionSource(kind: string, sourceId: string): string {
	const label = kind
		.split('_')
		.map((p) => p.charAt(0).toUpperCase() + p.slice(1))
		.join(' ')
	return `${label} · ${sourceId}`
}

export const useKnowledgeStore = defineStore('knowledge', () => {
	const graphSummary = ref<GraphSummary | null>(null)
	const graphError = ref('')
	const graphSearchQuery = ref('')
	const graphSearchResults = ref<GraphNode[]>([])
	const graphNeighborhood = ref<GraphNeighborhood | null>(null)
	const selectedGraphNode = ref<GraphNode | null>(null)
	const contradictionObservations = ref<ContradictionObservation[]>([])
	const reviewingContradictionObservationId = ref<string | null>(null)

	const graphCanvasNodes = computed<GraphCanvasNode[]>(() => {
		const neighborhood = graphNeighborhood.value
		if (!neighborhood) return []
		return buildRadialLayout(neighborhood.selected_node, neighborhood.nodes, RADIUS)
	})

	const graphCanvasEdges = computed<GraphCanvasEdge[]>(() => {
		const neighborhood = graphNeighborhood.value
		if (!neighborhood) return []
		return buildEdges(neighborhood.selected_node.node_id, neighborhood.edges, graphCanvasNodes.value)
	})

	const selectedGraphProperties = computed(() => {
		const node = selectedGraphNode.value
		if (!node) return []
		return Object.entries(node.properties)
			.slice(0, 8)
			.sort(([a], [b]) => a.localeCompare(b))
			.map(([key, value]) => ({ key, value }))
	})

	const graphNeighborCounts = computed(() => {
		const neighborhood = graphNeighborhood.value
		if (!neighborhood) return []
		const counts = new Map<string, number>()
		for (const node of neighborhood.nodes) {
			counts.set(node.node_kind, (counts.get(node.node_kind) ?? 0) + 1)
		}
		return Array.from(counts.entries())
			.sort(([, a], [, b]) => b - a)
			.map(([kind, count]) => ({ kind, count }))
	})

	const graphFilterChips = computed<GraphFilterChip[]>(() => {
		const summary = graphSummary.value
		if (!summary) return []
		return summary.node_counts.map((c) => ({
			kind: c.key,
			label: graphNodeKindLabel(c.key),
			icon: graphNodeKindIcon(c.key),
			count: c.count
		}))
	})

	function setGraphSummary(summary: GraphSummary | null, error: string) {
		graphSummary.value = summary
		graphError.value = error
	}

	function setGraphSearchResults(results: GraphNode[], query: string) {
		graphSearchResults.value = results
		graphSearchQuery.value = query
	}

	async function selectGraphNode(node: GraphNode) {
		selectedGraphNode.value = node
		graphNeighborhood.value = null
		try {
			const neighborhood = await fetchGraphNeighborhood(node.node_id, 1)
			graphNeighborhood.value = neighborhood
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown graph neighborhood error'
		}
	}

	async function runGraphSearch(query: string) {
		if (!query.trim()) {
			graphSearchResults.value = []
			graphSearchQuery.value = ''
			return
		}
		try {
			const results = await searchGraphNodes(query, 20)
			graphSearchResults.value = results
			graphSearchQuery.value = query
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown graph search error'
		}
	}

	async function loadGraphNodeChoices() {
		try {
			const nodes = await fetchGraphNodes(20)
			return nodes
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown graph node picker error'
			return []
		}
	}

	function setContradictionObservations(observations: ContradictionObservation[]) {
		contradictionObservations.value = observations
	}

	async function reviewContradictionObservation(
		observation: ContradictionObservation,
		reviewState: Exclude<ContradictionObservation['review_state'], 'suggested'>,
		resolution?: string
	) {
		reviewingContradictionObservationId.value = observation.observation_id
		try {
			await reviewContradiction(observation.observation_id, {
				review_state: reviewState,
				resolution: resolution?.trim() || undefined
			})
			const idx = contradictionObservations.value.findIndex(
				(o) => o.observation_id === observation.observation_id
			)
			if (idx !== -1) {
				contradictionObservations.value[idx] = {
					...contradictionObservations.value[idx],
					review_state: reviewState
				}
			}
		} catch (error) {
			graphError.value = error instanceof Error ? error.message : 'Unknown contradiction review action error'
		} finally {
			reviewingContradictionObservationId.value = null
		}
	}

	return {
		graphSummary,
		graphError,
		graphSearchQuery,
		graphSearchResults,
		graphNeighborhood,
		selectedGraphNode,
		contradictionObservations,
		reviewingContradictionObservationId,
		graphCanvasNodes,
		graphCanvasEdges,
		selectedGraphProperties,
		graphNeighborCounts,
		graphFilterChips,
		setGraphSummary,
		setGraphSearchResults,
		selectGraphNode,
		runGraphSearch,
		loadGraphNodeChoices,
		setContradictionObservations,
		reviewContradictionObservation
	}
})
```

### `frontend/src/domains/knowledge/types/knowledge.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/types/knowledge.ts`
- Size bytes / Размер в байтах: `3079`
- Included characters / Включено символов: `3079`
- Truncated / Обрезано: `no`

```typescript
export type GraphNodeKind =
	| 'person'
	| 'email_address'
	| 'message'
	| 'document'
	| 'project'
	| 'organization'
	| 'task'
	| 'event'
	| 'decision'
	| 'obligation'
	| 'knowledge';

export type GraphRelationshipType =
	| 'person_has_email_address'
	| 'person_sent_message'
	| 'person_received_message'
	| 'email_address_sent_message'
	| 'email_address_received_message'
	| 'project_has_message'
	| 'project_has_document'
	| 'project_involves_person'
	| 'project_involves_email_address'
	| 'entity_relationship';

export type GraphReviewState =
	| 'system_accepted'
	| 'suggested'
	| 'user_confirmed'
	| 'user_rejected';

export type GraphEvidenceSourceKind =
	| 'person'
	| 'message'
	| 'document'
	| 'raw_record'
	| 'relationship'
	| 'decision'
	| 'obligation';

export interface GraphNode {
	node_id: string;
	node_kind: GraphNodeKind;
	stable_key: string;
	label: string;
	properties: Record<string, unknown>;
	created_at: string;
	updated_at: string;
}

export interface GraphEdge {
	edge_id: string;
	source_node_id: string;
	target_node_id: string;
	relationship_type: GraphRelationshipType;
	confidence: number;
	review_state: GraphReviewState;
	properties: Record<string, unknown>;
	valid_from: string | null;
	valid_to: string | null;
	created_at: string;
	updated_at: string;
}

export interface GraphCount {
	key: string;
	count: number;
}

export interface GraphSummary {
	node_counts: GraphCount[];
	edge_counts: GraphCount[];
	evidence_count: number;
	latest_projection_at: string | null;
	is_empty: boolean;
}

export interface GraphEvidenceSummary {
	edge_id: string;
	source_kind: GraphEvidenceSourceKind;
	source_id: string;
	excerpt: string | null;
	metadata: Record<string, unknown>;
}

export interface GraphNeighborhood {
	selected_node: GraphNode;
	nodes: GraphNode[];
	edges: GraphEdge[];
	evidence: GraphEvidenceSummary[];
	edge_limit: number;
	truncated: boolean;
	evidence_limit: number;
	evidence_truncated: boolean;
}

export type ContradictionSourceKind =
	| 'communication'
	| 'document'
	| 'event'
	| 'memory'
	| 'knowledge'
	| 'decision'
	| 'obligation'
	| 'task'
	| 'relationship'
	| 'raw_record';

export type ContradictionSeverity = 'low' | 'medium' | 'high' | 'critical';

export type ContradictionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected';

export interface ContradictionObservation {
	observation_id: string;
	old_source_kind: ContradictionSourceKind;
	old_source_id: string;
	new_source_kind: ContradictionSourceKind;
	new_source_id: string;
	affected_entities: unknown;
	conflict_type: string;
	old_claim: string;
	new_claim: string;
	confidence: number;
	severity: ContradictionSeverity;
	review_state: ContradictionReviewState;
	metadata: Record<string, unknown>;
	reviewed_by: string | null;
	reviewed_at: string | null;
	resolution: string | null;
	created_at: string;
	updated_at: string;
}

export interface ContradictionListResponse {
	items: ContradictionObservation[];
}

export interface ContradictionReviewRequest {
	review_state: Exclude<ContradictionReviewState, 'suggested'>;
	resolution?: string;
}
```

### `frontend/src/domains/notes/api/notes.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/api/notes.ts`
- Size bytes / Размер в байтах: `287`
- Included characters / Включено символов: `287`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type { NoteItem } from '../types/notes'

export async function fetchNotes(): Promise<{ items: NoteItem[] }> {
  return ApiClient.instance.get<{ items: NoteItem[] }>(
    '/api/v1/notes',
    'Notes request failed'
  )
}
```

### `frontend/src/domains/notes/queries/useNotesQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/queries/useNotesQuery.ts`
- Size bytes / Размер в байтах: `208`
- Included characters / Включено символов: `208`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchNotes } from '../api/notes'

export function useNotesQuery() {
  return useQuery({
    queryKey: ['notes'],
    queryFn: () => fetchNotes()
  })
}
```

### `frontend/src/domains/notes/stores/notes.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/stores/notes.ts`
- Size bytes / Размер в байтах: `979`
- Included characters / Включено символов: `979`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useNotesStore = defineStore('notes-ui', () => {
  const searchQuery = ref('')
  const activeSources = ref<string[]>([])
  const activeTags = ref<string[]>([])
  const notesError = ref('')

  function setSearchQuery(q: string) {
    searchQuery.value = q
  }

  function toggleSource(source: string) {
    const idx = activeSources.value.indexOf(source)
    if (idx >= 0) {
      activeSources.value.splice(idx, 1)
    } else {
      activeSources.value.push(source)
    }
  }

  function toggleTag(tag: string) {
    const idx = activeTags.value.indexOf(tag)
    if (idx >= 0) {
      activeTags.value.splice(idx, 1)
    } else {
      activeTags.value.push(tag)
    }
  }

  function setNotesError(err: string) {
    notesError.value = err
  }

  return {
    searchQuery,
    activeSources,
    activeTags,
    notesError,
    setSearchQuery,
    toggleSource,
    toggleTag,
    setNotesError
  }
})
```

### `frontend/src/domains/notes/types/notes.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/types/notes.ts`
- Size bytes / Размер в байтах: `122`
- Included characters / Включено символов: `122`
- Truncated / Обрезано: `no`

```typescript
export interface NoteItem {
  title: string
  body: string
  source: string
  tag: string
  time: string
  icon: string
}
```

### `frontend/src/domains/organizations/api/organizations.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/organizations/api/organizations.ts`
- Size bytes / Размер в байтах: `726`
- Included characters / Включено символов: `726`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type { Organization } from '../types/organization'

export type OrganizationListResponse = { items: Organization[] }

export async function fetchOrganizations(limit = 50): Promise<OrganizationListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<OrganizationListResponse>(
    `/api/v1/organizations?${params.toString()}`,
    'Organizations request failed'
  )
}

export async function fetchOrganization(orgId: string): Promise<Organization> {
  return ApiClient.instance.get<Organization>(
    `/api/v1/organizations/${encodeURIComponent(orgId)}`,
    'Organization request failed'
  )
}
```

### `frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`
- Size bytes / Размер в байтах: `632`
- Included characters / Включено символов: `632`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchOrganizations, fetchOrganization } from '../api/organizations'
import type { Organization } from '../types/organization'

export function useOrganizationsQuery() {
  return useQuery<Organization[]>({
    queryKey: ['organizations', 'list'],
    queryFn: async () => {
      const res = await fetchOrganizations(50)
      return res.items as Organization[]
    }
  })
}

export function useOrganizationQuery(orgId: string) {
  return useQuery<Organization>({
    queryKey: ['organizations', orgId],
    queryFn: () => fetchOrganization(orgId),
    enabled: !!orgId
  })
}
```

### `frontend/src/domains/organizations/types/organization.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/organizations/types/organization.ts`
- Size bytes / Размер в байтах: `430`
- Included characters / Включено символов: `430`
- Truncated / Обрезано: `no`

```typescript
export interface Organization {
  organization_id: string
  display_name: string
  industry?: string | null
  country?: string | null
  status?: string | null
  watchlist?: boolean | null
  health_status?: string | null
  description?: string | null
  website?: string | null
  legal_name?: string | null
  registration_number?: string | null
  vat?: string | null
  interaction_count?: number | null
  priority?: string | null
}
```

### `frontend/src/domains/personas/api/personas.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/api/personas.ts`
- Size bytes / Размер в байтах: `3401`
- Included characters / Включено символов: `3401`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type { EnrichedPerson, PersonDossier, PersonIdentityCandidate, PersonIdentity, PersonIdentityReviewState, Relationship } from '../types/persona'

export type PersonListResponse = { items: EnrichedPerson[] }
export type PersonIdentityCandidateListResponse = { items: PersonIdentityCandidate[] }
export type PersonIdentityTraceListResponse = { items: PersonIdentity[] }
export type OrganizationListResponse = { items: any[] }
export type RelationshipListResponse = { relationships: Relationship[] }

export async function fetchPersons(limit = 50): Promise<PersonListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonListResponse>(
    `/api/v1/persons?${params.toString()}`,
    'Persons request failed'
  )
}

export async function fetchPersonDossier(personId: string): Promise<PersonDossier> {
  return ApiClient.instance.get<PersonDossier>(
    `/api/v1/persons/${encodeURIComponent(personId)}/dossier`,
    'Person dossier request failed'
  )
}

export async function fetchIdentityCandidates(limit = 50): Promise<PersonIdentityCandidateListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonIdentityCandidateListResponse>(
    `/api/v1/identity-candidates?${params.toString()}`,
    'Identity candidate request failed'
  )
}

export async function reviewIdentityCandidate(
  identityCandidateId: string,
  reviewState: PersonIdentityReviewState
): Promise<void> {
  await ApiClient.instance.put(
    `/api/v1/identity-candidates/${encodeURIComponent(identityCandidateId)}/review`,
    { review_state: reviewState }
  )
}

export async function fetchIdentityTraces(limit = 50): Promise<PersonIdentityTraceListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<PersonIdentityTraceListResponse>(
    `/api/v1/identity-traces?${params.toString()}`,
    'Identity traces request failed'
  )
}

export async function assignIdentityTrace(traceId: string, personId: string): Promise<void> {
  await ApiClient.instance.post(
    `/api/v1/identity-traces/${encodeURIComponent(traceId)}/assign`,
    { person_id: personId }
  )
}

export async function fetchRelationships(limit = 50): Promise<RelationshipListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<RelationshipListResponse>(
    `/api/v1/relationships?${params.toString()}`,
    'Relationships request failed'
  )
}

export async function reviewRelationship(
  relationshipId: string,
  reviewState: string
): Promise<void> {
  await ApiClient.instance.put(
    `/api/v1/relationships/${encodeURIComponent(relationshipId)}/review`,
    { review_state: reviewState }
  )
}

export async function fetchOrganizations(limit = 50): Promise<OrganizationListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<OrganizationListResponse>(
    `/api/v1/organizations?${params.toString()}`,
    'Organizations request failed'
  )
}

export async function fetchOrganization(orgId: string): Promise<any> {
  return ApiClient.instance.get<any>(
    `/api/v1/organizations/${encodeURIComponent(orgId)}`,
    'Organization request failed'
  )
}
```

### `frontend/src/domains/personas/queries/usePersonasQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/queries/usePersonasQuery.ts`
- Size bytes / Размер в байтах: `1224`
- Included characters / Включено символов: `1224`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchPersons, fetchIdentityCandidates, fetchIdentityTraces, fetchRelationships } from '../api/personas'
import type { EnrichedPerson, PersonIdentityCandidate, PersonIdentity, Relationship } from '../types/persona'

export function usePersonsQuery() {
  return useQuery<EnrichedPerson[]>({
    queryKey: ['persons', 'list'],
    queryFn: async () => {
      const res = await fetchPersons(50)
      return res.items
    }
  })
}

export function useIdentityCandidatesQuery() {
  return useQuery<PersonIdentityCandidate[]>({
    queryKey: ['persons', 'identity-candidates'],
    queryFn: async () => {
      const res = await fetchIdentityCandidates(50)
      return res.items
    }
  })
}

export function useIdentityTracesQuery() {
  return useQuery<PersonIdentity[]>({
    queryKey: ['persons', 'identity-traces'],
    queryFn: async () => {
      const res = await fetchIdentityTraces(50)
      return res.items
    }
  })
}

export function useRelationshipsQuery() {
  return useQuery<Relationship[]>({
    queryKey: ['persons', 'relationships'],
    queryFn: async () => {
      const res = await fetchRelationships(50)
      return res.relationships
    }
  })
}
```

### `frontend/src/domains/personas/stores/personas.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/stores/personas.ts`
- Size bytes / Размер в байтах: `5104`
- Included characters / Включено символов: `5104`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { PersonDossier, PersonIdentityCandidate, PersonIdentity, Relationship } from '../types/persona'
import { reviewIdentityCandidate, assignIdentityTrace, reviewRelationship } from '../api/personas'

export function formatIdentityTraceKind(kind: string): string {
  const labels: Record<string, string> = {
    email: 'Email',
    phone: 'Phone',
    telegram: 'Telegram',
    whatsapp: 'WhatsApp',
    social: 'Social Profile',
    name: 'Name',
    organization: 'Organization'
  }
  return labels[kind] || kind
}

export function formatIdentityTraceValue(trace: PersonIdentity): string {
  return trace.value || trace.identity_type
}

export function identityTraceConfidence(trace: PersonIdentity): string {
  return `${Math.round(trace.confidence * 100)}%`
}

export function formatRelationshipType(type: string): string {
  const labels: Record<string, string> = {
    colleague: 'Colleague',
    manager: 'Manager',
    client: 'Client',
    partner: 'Partner',
    vendor: 'Vendor',
    friend: 'Friend',
    family: 'Family',
    acquaintance: 'Acquaintance',
    competitor: 'Competitor',
    other: 'Other'
  }
  return labels[type] || type
}

export function formatRelationshipScore(score: number): string {
  if (score >= 0.8) return 'Strong'
  if (score >= 0.5) return 'Moderate'
  return 'Weak'
}

export function formatRelationshipEndpoint(kind: string, id: string): string {
  return `${kind}:${id.slice(0, 8)}...`
}

export function personIdentityPairKey(leftPersonId: string, rightPersonId: string): string {
  return leftPersonId <= rightPersonId
    ? `${leftPersonId}:${rightPersonId}`
    : `${rightPersonId}:${leftPersonId}`
}

export function dossierSectionPreview(dossier: PersonDossier): string[] {
  const words = (dossier.summary || '').split(/\s+/).filter(Boolean)
  return [...new Set(words.slice(0, 10))]
}

export const usePersonasStore = defineStore('personas', () => {
  const selectedPersonIndex = ref(0)
  const loadedDossierPersonId = ref<string | null>(null)
  const personDossier = ref<PersonDossier | null>(null)
  const personDossierError = ref('')
  const isPersonDossierLoading = ref(false)
  const identityCandidatesError = ref('')
  const identityTracesError = ref('')
  const relationshipsError = ref('')
  const assigningIdentityTraceId = ref<string | null>(null)
  const reviewingRelationshipId = ref<string | null>(null)

  const identityCandidates = ref<PersonIdentityCandidate[]>([])
  const identityTraces = ref<PersonIdentity[]>([])
  const relationships = ref<Relationship[]>([])

  const suggestedIdentityCandidates = computed(() =>
    identityCandidates.value.filter((item) => item.review_state === 'suggested')
  )

  function setIdentityCandidates(items: PersonIdentityCandidate[]) {
    identityCandidates.value = items
  }

  function setIdentityTraces(items: PersonIdentity[]) {
    identityTraces.value = items
  }

  function setRelationships(items: Relationship[]) {
    relationships.value = items
  }

  function selectPerson(index: number) {
    selectedPersonIndex.value = index
  }

  function setPersonDossier(dossier: PersonDossier | null, error: string) {
    personDossier.value = dossier
    personDossierError.value = error
  }

  function setPersonDossierLoading(loading: boolean) {
    isPersonDossierLoading.value = loading
  }

  function setLoadedDossierPersonId(id: string | null) {
    loadedDossierPersonId.value = id
  }

  async function reviewCandidate(candidate: PersonIdentityCandidate, state: PersonIdentityCandidate['review_state']) {
    try {
      await reviewIdentityCandidate(candidate.candidate_id, state as any)
    } catch (e: any) {
      identityCandidatesError.value = e.message || 'Review failed'
    }
  }

  async function assignTraceToPersona(trace: PersonIdentity, personId: string) {
    assigningIdentityTraceId.value = trace.id
    try {
      await assignIdentityTrace(trace.id, personId)
    } catch (e: any) {
      identityTracesError.value = e.message || 'Assignment failed'
    }
    assigningIdentityTraceId.value = null
  }

  async function reviewRelation(relationship: Relationship, reviewState: string) {
    reviewingRelationshipId.value = relationship.relationship_id
    try {
      await reviewRelationship(relationship.relationship_id, reviewState)
    } catch (e: any) {
      relationshipsError.value = e.message || 'Review failed'
    }
    reviewingRelationshipId.value = null
  }

  return {
    selectedPersonIndex,
    personDossier,
    personDossierError,
    isPersonDossierLoading,
    identityCandidatesError,
    identityTracesError,
    relationshipsError,
    assigningIdentityTraceId,
    reviewingRelationshipId,
    suggestedIdentityCandidates,
    identityCandidates,
    identityTraces,
    relationships,
    loadedDossierPersonId,
    setIdentityCandidates,
    setIdentityTraces,
    setRelationships,
    selectPerson,
    setPersonDossier,
    setPersonDossierLoading,
    setLoadedDossierPersonId,
    reviewCandidate,
    assignTraceToPersona,
    reviewRelation
  }
})
```

### `frontend/src/domains/personas/types/persona.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/personas/types/persona.ts`
- Size bytes / Размер в байтах: `1509`
- Included characters / Включено символов: `1509`
- Truncated / Обрезано: `no`

```typescript
export type PersonaType = 'human' | 'ai_agent' | 'organization_proxy' | 'system'

export interface EnrichedPerson {
  person_id: string
  display_name: string
  email_address: string
  preferred_channel: string | null
  last_interaction_at: string | null
  linked_projects: string[] | null
}

export interface PersonDossier {
  summary: string
  source_refs: string[]
  generated_at: string
}

export interface PersonIdentityCandidate {
  candidate_id: string
  candidate_kind: string
  left_person_id: string
  right_person_id: string | null
  evidence_summary: string
  confidence: number
  review_state: string
  created_at: string
}

export type PersonIdentityReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

export interface PersonIdentity {
  id: string
  identity_type: string
  value: string
  source: string
  confidence: number
  person_id: string | null
}

export interface Relationship {
  relationship_id: string
  source_entity_id: string
  source_entity_kind: string
  target_entity_id: string
  target_entity_kind: string
  relationship_type: string
  trust_score: number
  strength_score: number
  confidence: number
  review_state: string
}

export type RelationshipReviewState = 'suggested' | 'system_accepted' | 'user_confirmed' | 'user_rejected'

export interface PersonItem {
  person_id: string
  name: string
  role: string
  company: string
  channel?: string
  status?: string
}

export interface PersonaOption {
  person_id: string
  name: string
  company: string
}
```

### `frontend/src/domains/projects/api/projects.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/api/projects.ts`
- Size bytes / Размер в байтах: `660`
- Included characters / Включено символов: `660`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type { ProjectListResponse, ProjectDetail } from '../types/project'

export async function fetchProjects(limit = 25): Promise<ProjectListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<ProjectListResponse>(
    `/api/v1/projects?${params.toString()}`,
    'Projects request failed'
  )
}

export async function fetchProjectDetail(projectId: string): Promise<ProjectDetail> {
  return ApiClient.instance.get<ProjectDetail>(
    `/api/v1/projects/${encodeURIComponent(projectId)}`,
    'Project detail request failed'
  )
}
```

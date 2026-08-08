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

- Chunk ID / ID чанка: `150-source-frontend-part-010`
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

### `frontend/src/domains/projects/queries/useProjectsQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/queries/useProjectsQuery.ts`
- Size bytes / Размер в байтах: `722`
- Included characters / Включено символов: `722`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchProjects, fetchProjectDetail } from '../api/projects'
import type { ProjectSummary, ProjectDetail } from '../types/project'

export function useProjectsQuery() {
  return useQuery<ProjectSummary[]>({
    queryKey: ['projects'],
    queryFn: async () => {
      const response = await fetchProjects(25)
      return response.items
    }
  })
}

export function useProjectQuery(projectId: string | null) {
  return useQuery<ProjectDetail | null>({
    queryKey: ['project', projectId],
    queryFn: async () => {
      if (!projectId) return null
      const detail = await fetchProjectDetail(projectId)
      return detail
    },
    enabled: !!projectId
  })
}
```

### `frontend/src/domains/projects/stores/projects.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/stores/projects.ts`
- Size bytes / Размер в байтах: `1998`
- Included characters / Включено символов: `1998`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useProjectsStore = defineStore('projects-ui', () => {
  const selectedProjectId = ref<string>('')
  const projectsError = ref<string>('')
  const isProjectsLoading = ref<boolean>(false)

  function selectProject(projectId: string) {
    selectedProjectId.value = projectId
  }

  function setError(msg: string) {
    projectsError.value = msg
  }

  function clearError() {
    projectsError.value = ''
  }

  function setLoading(val: boolean) {
    isProjectsLoading.value = val
  }

  return {
    selectedProjectId,
    projectsError,
    isProjectsLoading,
    selectProject,
    setError,
    clearError,
    setLoading
  }
})

export function projectStatusLabel(status: string): string {
  return status
    .split('_')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

export function projectTimelineIcon(itemKind: string): string {
  switch (itemKind) {
    case 'message':
      return 'tabler:mail'
    case 'document':
      return 'tabler:file-text'
    default:
      return 'tabler:circle-dot'
  }
}

export function projectDocumentIcon(documentKind: string): string {
  switch (documentKind) {
    case 'pdf':
      return 'tabler:file-type-pdf'
    case 'markdown':
      return 'tabler:file-text'
    default:
      return 'tabler:file'
  }
}

export function formatProjectDate(value: string | null): string {
  if (!value) return 'Not set'
  const date = new Date(`${value}T00:00:00`)
  if (Number.isNaN(date.getTime())) return 'Invalid date'
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', year: 'numeric' }).format(date)
}

export function formatProjectDateTime(value: string | null): string {
  if (!value) return 'No activity'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Invalid date'
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)
}
```

### `frontend/src/domains/projects/types/project.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/projects/types/project.ts`
- Size bytes / Размер в байтах: `1531`
- Included characters / Включено символов: `1531`
- Truncated / Обрезано: `no`

```typescript
export type ProjectStatus = 'planning' | 'active' | 'on_hold' | 'completed' | 'archived'

export interface ProjectRecord {
  project_id: string
  name: string
  kind: string
  status: ProjectStatus
  description: string
  owner_display_name: string
  progress_percent: number
  start_date: string | null
  target_date: string | null
  created_at: string
  updated_at: string
}

export interface ProjectStats {
  message_count: number
  document_count: number
  people_count: number
  graph_connection_count: number
  latest_activity_at: string | null
}

export interface ProjectSummary {
  project: ProjectRecord
  stats: ProjectStats
  graph_node_id: string
}

export interface ProjectTimelineItem {
  item_kind: string
  item_id: string
  title: string
  subtitle: string
  occurred_at: string
}

export interface ProjectPersonSummary {
  display_name: string
  email_address: string
  interaction_count: number
  last_interaction_at: string | null
}

export interface ProjectMessageSummary {
  message_id: string
  subject: string
  sender: string
  occurred_at: string
}

export interface ProjectDocumentSummary {
  document_id: string
  document_kind: string
  title: string
  imported_at: string
}

export interface ProjectDetail {
  project: ProjectRecord
  stats: ProjectStats
  graph_node_id: string
  timeline: ProjectTimelineItem[]
  key_people: ProjectPersonSummary[]
  recent_messages: ProjectMessageSummary[]
  documents: ProjectDocumentSummary[]
}

export interface ProjectListResponse {
  items: ProjectSummary[]
}
```

### `frontend/src/domains/review/api/items.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/review/api/items.ts`
- Size bytes / Размер в байтах: `1894`
- Included characters / Включено символов: `1894`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
	ReviewItem,
	ReviewItemsResponse,
	ReviewItemPromotionRequest
} from '../types/review'

export async function fetchReviewItems(params: { status?: string; limit?: number } = {}): Promise<ReviewItemsResponse> {
	const search = new URLSearchParams()
	if (params.status) {
		search.set('status', params.status)
	}
	if (params.limit) {
		search.set('limit', String(Math.trunc(params.limit)))
	}
	return ApiClient.instance.get<ReviewItemsResponse>(
		`/api/v1/review/items?${search.toString()}`,
		'Review items request failed'
	)
}

export async function approveReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/approve`,
		{},
		'Review item approve request failed'
	)
}

export async function dismissReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/dismiss`,
		{},
		'Review item dismiss request failed'
	)
}

export async function takeReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/take`,
		{},
		'Review item take request failed'
	)
}

export async function archiveReviewItem(reviewItemId: string): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/archive`,
		{},
		'Review item archive request failed'
	)
}

export async function promoteReviewItem(
	reviewItemId: string,
	payload: ReviewItemPromotionRequest
): Promise<ReviewItem> {
	return ApiClient.instance.post<ReviewItem>(
		`/api/v1/review/items/${encodeURIComponent(reviewItemId)}/promote`,
		payload,
		'Review item promote request failed'
	)
}
```

### `frontend/src/domains/review/api/workspace.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/review/api/workspace.ts`
- Size bytes / Размер в байтах: `2997`
- Included characters / Включено символов: `2997`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ContradictionListResponse,
  ContradictionObservation,
  Decision,
  DecisionListResponse,
  Obligation,
  ObligationListResponse,
  RelationshipListResponse,
} from '../types/review'

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

export async function fetchDecisionReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<DecisionListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50)),
  })
  return ApiClient.instance.get<DecisionListResponse>(
    `/api/v1/decisions?${query.toString()}`,
    'Decision review items request failed'
  )
}

export async function reviewDecision(
  decisionId: string,
  request: { review_state: 'user_confirmed' | 'user_rejected' }
): Promise<Decision> {
  return ApiClient.instance.put<Decision>(
    `/api/v1/decisions/${encodeURIComponent(decisionId)}/review`,
    request,
    'Decision review request failed'
  )
}

export async function fetchObligationReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<ObligationListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50)),
  })
  return ApiClient.instance.get<ObligationListResponse>(
    `/api/v1/obligations?${query.toString()}`,
    'Obligation review items request failed'
  )
}

export async function reviewObligation(
  obligationId: string,
  request: { review_state: 'user_confirmed' | 'user_rejected' }
): Promise<Obligation> {
  return ApiClient.instance.put<Obligation>(
    `/api/v1/obligations/${encodeURIComponent(obligationId)}/review`,
    request,
    'Obligation review request failed'
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
  request: { review_state: 'user_confirmed' | 'user_rejected' }
): Promise<ContradictionObservation> {
  return ApiClient.instance.put<ContradictionObservation>(
    `/api/v1/contradictions/${encodeURIComponent(observationId)}/review`,
    request,
    'Contradiction review request failed'
  )
}
```

### `frontend/src/domains/review/stores/review.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/review/stores/review.ts`
- Size bytes / Размер в байтах: `7047`
- Included characters / Включено символов: `7046`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
	Relationship,
	Decision,
	Obligation,
	ContradictionObservation,
	ReviewItem,
	ReviewWorkspaceItemAction
} from '../types/review'
import {
	fetchRelationships,
	reviewRelationship,
	fetchDecisionReviewItems,
	reviewDecision,
	fetchObligationReviewItems,
	reviewObligation,
	fetchContradictions,
	reviewContradiction
} from '../api/workspace'
import {
	fetchReviewItems,
	approveReviewItem,
	dismissReviewItem,
	archiveReviewItem,
	promoteReviewItem,
	takeReviewItem
} from '../api/items'

export const useReviewStore = defineStore('review', () => {
	const relationships = ref<Relationship[]>([])
	const decisions = ref<Decision[]>([])
	const obligations = ref<Obligation[]>([])
	const contradictions = ref<ContradictionObservation[]>([])
	const reviewItems = ref<ReviewItem[]>([])
	const error = ref('')
	const reviewingItemKey = ref<string | null>(null)

	const relationsSuggestedCount = computed(() =>
		relationships.value.filter((r) => r.review_state === 'suggested').length
	)

	const decisionsSuggestedCount = computed(() =>
		decisions.value.filter((d) => d.review_state === 'suggested').length
	)

	const obligationsSuggestedCount = computed(() =>
		obligations.value.filter((o) => o.review_state === 'suggested').length
	)

	const contradictionsSuggestedCount = computed(() =>
		contradictions.value.filter((c) => c.review_state === 'suggested').length
	)
	const reviewItemsCount = computed(() => reviewItems.value.filter((r) => r.status === 'new' || r.status === 'in_review').length)

	const totalSuggestedCount = computed(() =>
		relationsSuggestedCount.value +
		decisionsSuggestedCount.value +
		obligationsSuggestedCount.value +
		contradictionsSuggestedCount.value +
		reviewItemsCount.value
	)

	async function loadAll() {
		error.value = ''
		const errors: string[] = []

		try {
			const relRes = await fetchRelationships(50)
			relationships.value = relRes.relationships || []
		} catch (e) {
			errors.push(`Relationships: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		try {
			const decRes = await fetchDecisionReviewItems({ reviewState: 'suggested', limit: 50 })
			decisions.value = decRes.items || []
		} catch (e) {
			errors.push(`Decisions: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		try {
			const oblRes = await fetchObligationReviewItems({ reviewState: 'suggested', limit: 50 })
			obligations.value = oblRes.items || []
		} catch (e) {
			errors.push(`Obligations: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		try {
			const conRes = await fetchContradictions(50)
			contradictions.value = conRes.items || []
		} catch (e) {
			errors.push(`Contradictions: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}
		try {
			const reviewRes = await fetchReviewItems({ status: 'active', limit: 50 })
			reviewItems.value = reviewRes.items || []
		} catch (e) {
			errors.push(`Review inbox: ${e instanceof Error ? e.message : 'Unknown error'}`)
		}

		if (errors.length > 0) {
			error.value = errors.join(' · ')
		}
	}

	async function reviewItem(action: ReviewWorkspaceItemAction): Promise<string> {
		const itemKey = reviewItemKey(action)
		reviewingItemKey.value = itemKey

		try {
			switch (action.kind) {
				case 'relationship': {
					await reviewRelationship(action.item.relationship_id, action.reviewState)
					const idx = relationships.value.findIndex(
						(r: Relationship) => r.relationship_id === action.item.relationship_id
					)
					if (idx !== -1) {
						relationships.value[idx] = { ...relationships.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'decision': {
					await reviewDecision(action.item.decision_id, { review_state: action.reviewState })
					const idx = decisions.value.findIndex(
						(d: Decision) => d.decision_id === action.item.decision_id
					)
					if (idx !== -1) {
						decisions.value[idx] = { ...decisions.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'obligation': {
					await reviewObligation(action.item.obligation_id, { review_state: action.reviewState })
					const idx = obligations.value.findIndex(
						(o: Obligation) => o.obligation_id === action.item.obligation_id
					)
					if (idx !== -1) {
						obligations.value[idx] = { ...obligations.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'contradiction': {
					await reviewContradiction(action.item.observation_id, { review_state: action.reviewState })
					const idx = contradictions.value.findIndex(
						(c: ContradictionObservation) => c.observation_id === action.item.observation_id
					)
					if (idx !== -1) {
						contradictions.value[idx] = { ...contradictions.value[idx], review_state: action.reviewState }
					}
					break
				}
				case 'review_item': {
					if (action.action === 'approve') {
						const updated = await approveReviewItem(action.item.review_item_id)
						updateReviewItem(updated)
					} else {
						const updated = await dismissReviewItem(action.item.review_item_id)
						updateReviewItem(updated)
					}
					break
				}
				case 'review_item_archive': {
					const updated = await archiveReviewItem(action.item.review_item_id)
					updateReviewItem(updated)
					break
				}
				case 'review_item_take': {
					const updated = await takeReviewItem(action.item.review_item_id)
					updateReviewItem(updated)
					break
				}
				case 'review_item_promote': {
					const updated = await promoteReviewItem(action.item.review_item_id, action.promotion)
					updateReviewItem(updated)
					break
				}
			}
			return ''
		} catch (e) {
			return e instanceof Error ? e.message : 'Unknown review action error'
		} finally {
			reviewingItemKey.value = null
		}
	}

	function updateReviewItem(updated: ReviewItem) {
		const idx = reviewItems.value.findIndex((item) => item.review_item_id === updated.review_item_id)
		if (idx === -1) return
		reviewItems.value[idx] = updated
	}

	return {
		relationships,
		decisions,
		obligations,
		contradictions,
		reviewItems,
		reviewItemsCount,
		error,
		reviewingItemKey,
		relationsSuggestedCount,
		decisionsSuggestedCount,
		obligationsSuggestedCount,
		contradictionsSuggestedCount,
		totalSuggestedCount,
		loadAll,
		reviewItem
	}
})

function reviewItemKey(action: ReviewWorkspaceItemAction): string {
	switch (action.kind) {
		case 'relationship':
			return `relationship:${action.item.relationship_id}`
		case 'decision':
			return `decision:${action.item.decision_id}`
		case 'obligation':
			return `obligation:${action.item.obligation_id}`
		case 'contradiction':
			return `contradiction:${action.item.observation_id}`
		case 'review_item':
			return `review_item:${action.item.review_item_id}`
		case 'review_item_archive':
			return `review_item_archive:${action.item.review_item_id}`
		case 'review_item_take':
			return `review_item_take:${action.item.review_item_id}`
		case 'review_item_promote':
			return `review_item_promote:${action.item.review_item_id}`
	}
}
```

### `frontend/src/domains/review/types/review.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/review/types/review.ts`
- Size bytes / Размер в байтах: `3509`
- Included characters / Включено символов: `3509`
- Truncated / Обрезано: `no`

```typescript
export type RelationshipReviewState = 'suggested' | 'system_accepted' | 'user_confirmed' | 'user_rejected'
export type DecisionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type ObligationReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type ContradictionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type UserRelationshipReviewState = Extract<
	RelationshipReviewState,
	'user_confirmed' | 'user_rejected'
>

export interface Relationship {
	relationship_id: string
	source_entity_kind: string
	source_entity_id: string
	target_entity_kind: string
	target_entity_id: string
	relationship_type: string
	trust_score?: number | null
	review_state: RelationshipReviewState
}

export interface Decision {
	decision_id: string
	title: string
	decided_by_entity_kind?: string | null
	decided_by_entity_id?: string | null
	decided_at?: string | null
	review_state: DecisionReviewState
}

export interface Obligation {
	obligation_id: string
	statement: string
	obligated_entity_kind: string
	obligated_entity_id: string
	due_at?: string | null
	review_state: ObligationReviewState
}

export interface ContradictionObservation {
	observation_id: string
	old_claim: string
	new_claim: string
	severity: string
	created_at: string
	review_state: ContradictionReviewState
}

export interface RelationshipListResponse {
	relationships: Relationship[]
}

export interface DecisionListResponse {
	items: Decision[]
}

export interface ObligationListResponse {
	items: Obligation[]
}

export interface ContradictionListResponse {
	items: ContradictionObservation[]
}

export type ReviewWorkspaceItemKind = 'relationship' | 'decision' | 'obligation' | 'contradiction'

export interface ReviewItem {
	review_item_id: string
	item_kind: ReviewItemKind
	title: string
	summary: string
	status: ReviewItemStatus
	target_domain: string | null
	target_entity_kind: string | null
	target_entity_id: string | null
	confidence: number
	metadata: Record<string, unknown>
	created_at: string
	updated_at: string
}

export interface ReviewItemsResponse {
	items: ReviewItem[]
}

export interface ReviewItemPromotionRequest {
	target_domain: string
	target_entity_kind: string
	target_entity_id: string
}

export type ReviewItemStatus =
	| 'new'
	| 'in_review'
	| 'approved'
	| 'promoted'
	| 'dismissed'
	| 'archived'

export type ReviewItemKind =
	| 'new_person'
	| 'new_organization'
	| 'identity_candidate'
	| 'project_link_candidate'
	| 'contradiction_candidate'
	| 'potential_task'
	| 'potential_obligation'
	| 'potential_decision'
	| 'potential_relationship'
	| 'potential_project'
	| 'knowledge_candidate'

export type ReviewWorkspaceItemAction =
	| {
			kind: 'relationship'
			item: Relationship
			reviewState: UserRelationshipReviewState
	  }
	| {
			kind: 'decision'
			item: Decision
			reviewState: 'user_confirmed' | 'user_rejected'
	  }
	| {
			kind: 'obligation'
			item: Obligation
			reviewState: 'user_confirmed' | 'user_rejected'
	  }
	| {
			kind: 'contradiction'
			item: ContradictionObservation
			reviewState: 'user_confirmed' | 'user_rejected'
	  }
	| {
			kind: 'review_item'
			item: ReviewItem
			action: 'approve' | 'dismiss'
	  }
	| {
			kind: 'review_item_take'
			item: ReviewItem
	  }
	| {
			kind: 'review_item_archive'
			item: ReviewItem
	  }
	| {
			kind: 'review_item_promote'
			item: ReviewItem
			promotion: ReviewItemPromotionRequest
	  }

export interface ReviewWorkspaceItemActionResult {
	itemKey: string
	error: string
}
```

### `frontend/src/domains/settings/api/settings.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/api/settings.ts`
- Size bytes / Размер в байтах: `2196`
- Included characters / Включено символов: `2196`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ProviderAccountListResponse,
  CalendarAccount
} from '../types/settings'

export {
  fetchApplicationSettings,
  saveApplicationSetting,
  FRONTEND_LAYOUT_SETTING_KEY,
  FRONTEND_SIDEBAR_SETTING_KEY,
  FRONTEND_LOCALE_SETTING_KEY,
  FRONTEND_THEME_SETTING_KEY,
  FRONTEND_UI_STATE_SETTING_KEY
} from '../../../platform/settings/applicationSettingsClient'

export async function fetchProviderAccounts(): Promise<ProviderAccountListResponse> {
  return ApiClient.instance.get<ProviderAccountListResponse>(
    '/api/v1/settings/accounts',
    'Provider accounts request failed'
  )
}

export async function fetchCalendarAccounts(): Promise<{ items: CalendarAccount[] }> {
  return ApiClient.instance.get<{ items: CalendarAccount[] }>(
    '/api/v1/settings/accounts/calendar',
    'Calendar accounts request failed'
  )
}

export async function deleteMailAccount(accountId: string): Promise<{ result: boolean; error?: string }> {
  return ApiClient.instance.delete<{ result: boolean; error?: string }>(
    `/api/v1/settings/accounts/mail/${encodeURIComponent(accountId)}`,
    'Mail account delete failed'
  )
}

export async function logoutMailAccount(accountId: string): Promise<{ result: boolean; error?: string }> {
  return ApiClient.instance.post<{ result: boolean; error?: string }>(
    `/api/v1/settings/accounts/mail/${encodeURIComponent(accountId)}/logout`,
    {},
    'Mail account logout failed'
  )
}

export async function exportMailAccountSettings(
  accountId: string
): Promise<{ result?: { exported_at: string }; error?: string }> {
  return ApiClient.instance.get<{ result?: { exported_at: string }; error?: string }>(
    `/api/v1/settings/accounts/mail/${encodeURIComponent(accountId)}/export`,
    'Mail account export failed'
  )
}

export async function importMailAccountSettings(
  request: { account_id?: string; provider_kind: string; settings: Record<string, unknown> }
): Promise<{ result?: unknown; error?: string }> {
  return ApiClient.instance.post<{ result?: unknown; error?: string }>(
    '/api/v1/settings/accounts/mail/import',
    request,
    'Mail account import failed'
  )
}
```

### `frontend/src/domains/settings/api/signalHub.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/api/signalHub.test.ts`
- Size bytes / Размер в байтах: `32628`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import { resetSignalHubConnectClientForTests } from '../../../platform/connect/signalHubClient'
import {
  applySignalHubProfile,
  fetchSignalHubCapabilities,
  createSignalHubProfile,
  createSignalHubReplayRequest,
  createSignalHubPolicy,
  createSignalHubConnection,
  disableSignalHubSignals,
  disableSignalHubSource,
  emitSignalHubFixtureSignal,
  enableSignalHubSignals,
  enableSignalHubSource,
  fetchSignalHubFixtureSources,
  fetchSignalHubProfiles,
  fetchSignalHubConnections,
  fetchSignalHubHealth,
  fetchSignalHubPolicies,
  fetchSignalHubReplayRequests,
  fetchSignalHubRuntimeStates,
  muteSignalHubSignals,
  pauseSignalHubSignals,
  fetchSignalHubSources,
  removeSignalHubConnection,
  removeSignalHubProfile,
  resumeSignalHubSignals,
  restoreSignalHubSystemFixture,
  runSignalHubHealthCheck,
  unmuteSignalHubSignals,
  updateSignalHubConnection,
  updateSignalHubProfile,
  updateSignalHubRuntimeState
} from './signalHub'

describe('Signal Hub settings API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
    resetSignalHubConnectClientForTests()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    resetSignalHubConnectClientForTests()
    ApiClient.resetForTests()
  })

  it('fetches Signal Hub sources through the protected API client', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchSignalHubSources()

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/ListSources')
    expect(options.method).toBe('POST')
    expect(new Headers(options.headers).get('X-Макошь-Secret')).toBe('test-secret')
  })

  it('lists Signal Hub capabilities through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          id: 'cap-1',
          sourceCode: 'telegram',
          capability: 'runtime.replay',
          state: 'available',
          reason: 'source events can be replayed from durable Signal Hub history',
          requiresConfirmation: false,
          actionClass: 'local_write',
          updatedAt: '2026-06-23T00:00:00Z'
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchSignalHubCapabilities()

    expect(response.items).toEqual([{
      id: 'cap-1',
      source_code: 'telegram',
      connection_id: null,
      capability: 'runtime.replay',
      state: 'available',
      reason: 'source events can be replayed from durable Signal Hub history',
      requires_confirmation: false,
      action_class: 'local_write',
      updated_at: '2026-06-23T00:00:00Z'
    }])
    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/ListCapabilities'
    )
  })

  it('runs source and scoped signal control commands through ConnectRPC', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ sourceCode: 'telegram', clearedCount: 1 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ sourceCode: 'telegram', policyId: 'policy-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ policyId: 'policy-2' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ clearedCount: 2 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ policyId: 'policy-3' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ clearedCount: 2 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ policyId: 'policy-4' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ clearedCount: 3 }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' }
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await enableSignalHubSource('telegram')
    await disableSignalHubSource('telegram')
    await disableSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await enableSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await muteSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await unmuteSignalHubSignals({ scope: 'event_pattern', event_pattern: 'signal.raw.telegram.*' })
    await pauseSignalHubSignals({ scope: 'global' })
    await resumeSignalHubSignals({ scope: 'global' })

    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/EnableSource'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/DisableSource'
    )
    expect(fetchMock.mock.calls[2][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/DisableSignals'
    )
    expect(fetchMock.mock.calls[3][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/EnableSignals'
    )
    expect(fetchMock.mock.calls[4][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/MuteSignals'
    )
    expect(fetchMock.mock.calls[5][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/UnmuteSignals'
    )
    expect(fetchMock.mock.calls[6][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/PauseSignals'
    )
    expect(fetchMock.mock.calls[7][0]).toBe(
      'http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/ResumeSignals'
    )
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[2][1].body))).toMatchObject({
      scope: 'event_pattern',
      eventPattern: 'signal.raw.telegram.*'
    })
    expect(JSON.parse(decodeBody(fetchMock.mock.calls[4][1].body))).toMatchObject({
      scope: 'event_pattern',
      eventPattern: 'signal.raw.telegram.*'
    })
  })

  it('restores the system fixture through the Signal Hub recovery endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ sourcesCreated: 13, sourcesRepaired: 0, profilesCreated: 4, profilesRepaired: 0 }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const report = await restoreSignalHubSystemFixture()

    expect(report.sources_created).toBe(13)
    expect(report.profiles_created).toBe(4)
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, options] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/RestoreSystemFixture')
    expect(options.method).toBe('POST')
  })

  it('lists Signal Hub fixture sources through ConnectRPC', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({
        items: [{
          fixtureId: 'fixture_basic_message',
          sourceCode: 'fixture',
          eventType: 'signal.raw.fixture.message.observed',
          correlationId: 'fixture-basic-message',
          occurredAt: '2026-01-01T00:00:00Z',
          summary: 'Fixture message'
        }]
      }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const response = await fetchSignalHubFixtureSources()

    expect(response.items).toEqual([{
      fixture_id: 'fixture_basic_message',
      source_code: 'fixture',
      event_type: 'signal.raw.fixture.message.observed',
      correlation_id: 'fixture-basic-message',
      occurred_at: '2026-01-01T00:00:00Z',
      summary: 'Fixture message'
    }])
    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toBe('http://127.0.0.1:8080/makosh.signal_hub.v1.SignalHubService/ListFixtureSources')
  })

  it('creates, updates, applies and removes Signal Hub custom profiles through ConnectRPC', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Mute noisy sources overnight.',
            policyCount: 2,
            sourcePolicies: [
              { scope: 'source', sourceCode: 'telegram', mode: 'muted', reason: 'night mute' },
              { scope: 'source', sourceCode: 'mail', mode: 'paused', reason: 'overnight pause' }
            ],
            isSystem: false,
            isActive: false,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:00:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Updated description.',
            policyCount: 1,
            sourcePolicies: [
              { scope: 'event_pattern', eventPattern: 'signal.raw.mail.*', mode: 'muted', reason: 'mail quiet hours' }
            ],
            isSystem: false,
            isActive: false,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:10:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Updated description.',
            policyCount: 1,
            sourcePolicies: [
              { scope: 'event_pattern', eventPattern: 'signal.raw.mail.*', mode: 'muted', reason: 'mail quiet hours' }
            ],
            isSystem: false,
            isActive: true,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:10:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({
          item: {
            id: 'profile-custom-1',
            code: 'quiet_hours',
            displayName: 'Quiet Hours',
            description: 'Updated description.',
            policyCount: 1,
            sourcePolicies: [
              { scope: 'event_pattern', eventPattern: 'signal.raw.mail.*', mode: 'muted', reason: 'mail quiet hours' }
            ],
            isSystem: false,
            isActive: false,
            createdAt: '2026-06-23T00:00:00Z',
            updatedAt: '2026-06-23T00:10:00Z'
          }
        }), { status: 200, headers: { 'Content-Type': 'application/json' } })
      )
    vi.stubGlobal('fetch', fetchMock)

    c
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/settings/api/signalHub.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/api/signalHub.ts`
- Size bytes / Размер в байтах: `23520`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { getSignalHubConnectClient } from '../../../platform/connect/signalHubClient'
import type {
  SignalHubCapabilitiesResponse,
  SignalHubCapability,
  SignalHubConnectionCreateRequest,
  SignalHubConnectionResponse,
  SignalHubConnectionUpdateRequest,
  SignalHubConnectionsResponse,
  SignalHubControlRequest,
  SignalHubControlResponse,
  SignalHubCreatePolicyResponse,
  SignalHubFixtureEmission,
  SignalHubFixtureSourcesResponse,
  SignalHubFixtureRestoreReport,
  SignalHubHealthResponse,
  SignalHubHealthCheckRequest,
  SignalHubPoliciesResponse,
  SignalHubPolicyRequest,
  SignalHubProfile,
  SignalHubProfileCreateRequest,
  SignalHubProfilePolicy,
  SignalHubProfileUpdateRequest,
  SignalHubProfilesResponse,
  SignalHubReplayRequestsResponse,
  SignalHubReplayRequestCreateRequest,
  SignalHubRuntimeState,
  SignalHubRuntimeStateRequest,
  SignalHubRuntimeStatesResponse,
  SignalHubSourcesResponse
} from '../types/signalHub'

export async function fetchSignalHubSources(): Promise<SignalHubSourcesResponse> {
  const response = await getSignalHubConnectClient().listSources({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      code: item.code,
      display_name: item.displayName,
      category: item.category,
      source_kind: item.sourceKind,
      default_enabled: item.defaultEnabled,
      supports_connections: item.supportsConnections,
      supports_runtime: item.supportsRuntime,
      supports_replay: item.supportsReplay,
      supports_pause: item.supportsPause,
      supports_mute: item.supportsMute,
      capability_schema_version: item.capabilitySchemaVersion,
      created_at: item.createdAt,
      updated_at: item.updatedAt
    }))
  }
}

export async function fetchSignalHubSource(sourceCode: string): Promise<SignalHubSourcesResponse['items'][number]> {
  const response = await getSignalHubConnectClient().getSource({
    code: sourceCode
  })
  return {
    id: response.item?.id ?? '',
    code: response.item?.code ?? sourceCode,
    display_name: response.item?.displayName ?? sourceCode,
    category: response.item?.category ?? '',
    source_kind: response.item?.sourceKind ?? '',
    default_enabled: response.item?.defaultEnabled ?? false,
    supports_connections: response.item?.supportsConnections ?? false,
    supports_runtime: response.item?.supportsRuntime ?? false,
    supports_replay: response.item?.supportsReplay ?? false,
    supports_pause: response.item?.supportsPause ?? false,
    supports_mute: response.item?.supportsMute ?? false,
    capability_schema_version: response.item?.capabilitySchemaVersion ?? 1,
    created_at: response.item?.createdAt ?? '',
    updated_at: response.item?.updatedAt ?? ''
  }
}

export async function fetchSignalHubCapabilities(): Promise<SignalHubCapabilitiesResponse> {
  const response = await getSignalHubConnectClient().listCapabilities({})
  return {
    items: response.items.map(mapCapability)
  }
}

export async function fetchSignalHubFixtureSources(): Promise<SignalHubFixtureSourcesResponse> {
  const response = await getSignalHubConnectClient().listFixtureSources({})
  return {
    items: response.items.map((item) => ({
      fixture_id: item.fixtureId,
      source_code: item.sourceCode,
      event_type: item.eventType,
      correlation_id: item.correlationId ?? null,
      occurred_at: item.occurredAt,
      summary: item.summary
    }))
  }
}

export async function restoreSignalHubSystemFixture(): Promise<SignalHubFixtureRestoreReport> {
  const response = await getSignalHubConnectClient().restoreSystemFixture({})
  return {
    sources_created: response.sourcesCreated,
    sources_repaired: response.sourcesRepaired,
    profiles_created: response.profilesCreated,
    profiles_repaired: response.profilesRepaired
  }
}

export async function fetchSignalHubProfiles(): Promise<SignalHubProfilesResponse> {
  const response = await getSignalHubConnectClient().listProfiles({})
  return {
    items: response.items.map((item) => mapProfile(item, item.code))
  }
}

export async function createSignalHubProfile(
  request: SignalHubProfileCreateRequest
): Promise<SignalHubProfile> {
  const response = await getSignalHubConnectClient().createProfile({
    code: request.code,
    displayName: request.display_name,
    description: request.description,
    sourcePolicies: request.source_policies.map(protoProfilePolicy)
  })
  return mapProfile(response.item, request.code)
}

export async function updateSignalHubProfile(
  profileCode: string,
  request: SignalHubProfileUpdateRequest
): Promise<SignalHubProfile> {
  const response = await getSignalHubConnectClient().updateProfile({
    code: profileCode,
    displayName: request.display_name,
    description: request.description,
    sourcePolicies: (request.source_policies ?? []).map(protoProfilePolicy),
    updateSourcePolicies: request.source_policies !== undefined
  })
  return mapProfile(response.item, profileCode)
}

export async function removeSignalHubProfile(profileCode: string): Promise<SignalHubProfile> {
  const response = await getSignalHubConnectClient().removeProfile({
    code: profileCode
  })
  return mapProfile(response.item, profileCode)
}

export async function applySignalHubProfile(profileCode: string): Promise<SignalHubProfilesResponse['items'][number]> {
  const response = await getSignalHubConnectClient().applyProfile({
    code: profileCode
  })
  return mapProfile(response.item, profileCode)
}

export async function emitSignalHubFixtureSignal(
  fixtureId: string
): Promise<SignalHubFixtureEmission> {
  const response = await getSignalHubConnectClient().emitFixtureSignal({
    fixtureId
  })
  return {
    fixture_id: response.fixtureId,
    raw_event_id: response.rawEventId,
    event_type: response.eventType,
    source_code: response.sourceCode,
    correlation_id: response.correlationId ?? null
  }
}

export async function fetchSignalHubConnections(): Promise<SignalHubConnectionsResponse> {
  const response = await getSignalHubConnectClient().listConnections({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      source_code: item.sourceCode,
      display_name: item.displayName,
      status: item.status,
      profile: item.profile ?? null,
      settings: parseJsonObject(item.settingsJson),
      secret_ref: item.secretRef ?? null,
      connected_at: item.connectedAt ?? null,
      last_seen_at: item.lastSeenAt ?? null,
      last_signal_at: item.lastSignalAt ?? null,
      last_sync_at: item.lastSyncAt ?? null,
      created_at: item.createdAt,
      updated_at: item.updatedAt
    }))
  }
}

export async function createSignalHubConnection(
  request: SignalHubConnectionCreateRequest
): Promise<SignalHubConnectionResponse> {
  const response = await getSignalHubConnectClient().createConnection({
    sourceCode: request.source_code,
    displayName: request.display_name,
    status: request.status,
    profile: request.profile ?? undefined,
    secretRef: request.secret_ref ?? undefined,
    settingsJson: JSON.stringify(request.settings ?? {})
  })
  return {
    item: {
      id: response.item?.id ?? '',
      source_code: response.item?.sourceCode ?? request.source_code,
      display_name: response.item?.displayName ?? request.display_name,
      status: response.item?.status ?? request.status,
      profile: response.item?.profile ?? null,
      settings: parseJsonObject(response.item?.settingsJson),
      secret_ref: response.item?.secretRef ?? null,
      connected_at: response.item?.connectedAt ?? null,
      last_seen_at: response.item?.lastSeenAt ?? null,
      last_signal_at: response.item?.lastSignalAt ?? null,
      last_sync_at: response.item?.lastSyncAt ?? null,
      created_at: response.item?.createdAt ?? '',
      updated_at: response.item?.updatedAt ?? ''
    }
  }
}

export async function updateSignalHubConnection(
  connectionId: string,
  request: SignalHubConnectionUpdateRequest
): Promise<SignalHubConnectionResponse> {
  const response = await getSignalHubConnectClient().updateConnection({
    id: connectionId,
    displayName: request.display_name,
    status: request.status,
    profile: request.profile ?? undefined,
    secretRef: request.secret_ref ?? undefined,
    settingsJson: request.settings ? JSON.stringify(request.settings) : undefined
  })
  return {
    item: {
      id: response.item?.id ?? connectionId,
      source_code: response.item?.sourceCode ?? '',
      display_name: response.item?.displayName ?? '',
      status: response.item?.status ?? request.status ?? '',
      profile: response.item?.profile ?? null,
      settings: parseJsonObject(response.item?.settingsJson),
      secret_ref: response.item?.secretRef ?? null,
      connected_at: response.item?.connectedAt ?? null,
      last_seen_at: response.item?.lastSeenAt ?? null,
      last_signal_at: response.item?.lastSignalAt ?? null,
      last_sync_at: response.item?.lastSyncAt ?? null,
      created_at: response.item?.createdAt ?? '',
      updated_at: response.item?.updatedAt ?? ''
    }
  }
}

export async function removeSignalHubConnection(
  connectionId: string
): Promise<SignalHubConnectionResponse> {
  const response = await getSignalHubConnectClient().removeConnection({
    id: connectionId
  })
  return {
    item: {
      id: response.item?.id ?? connectionId,
      source_code: response.item?.sourceCode ?? '',
      display_name: response.item?.displayName ?? '',
      status: response.item?.status ?? 'removed',
      profile: response.item?.profile ?? null,
      settings: parseJsonObject(response.item?.settingsJson),
      secret_ref: response.item?.secretRef ?? null,
      connected_at: response.item?.connectedAt ?? null,
      last_seen_at: response.item?.lastSeenAt ?? null,
      last_signal_at: response.item?.lastSignalAt ?? null,
      last_sync_at: response.item?.lastSyncAt ?? null,
      created_at: response.item?.createdAt ?? '',
      updated_at: response.item?.updatedAt ?? ''
    }
  }
}

export async function fetchSignalHubHealth(): Promise<SignalHubHealthResponse> {
  const response = await getSignalHubConnectClient().listHealth({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      source_code: item.sourceCode,
      connection_id: item.connectionId ?? null,
      level: item.level,
      summary: item.summary,
      last_ok_at: item.lastOkAt ?? null,
      last_failure_at: item.lastFailureAt ?? null,
      failure_count: item.failureCount,
      consecutive_failure_count: item.consecutiveFailureCount,
      next_retry_at: item.nextRetryAt ?? null,
      evidence: parseJsonObject(item.evidenceJson),
      updated_at: item.updatedAt
    }))
  }
}

export async function runSignalHubHealthCheck(
  request: SignalHubHealthCheckRequest
): Promise<SignalHubHealthResponse['items'][number]> {
  const response = await getSignalHubConnectClient().runHealthCheck({
    sourceCode: request.source_code,
    connectionId: request.connection_id ?? undefined
  })
  return {
    id: response.item?.id ?? '',
    source_code: response.item?.sourceCode ?? request.source_code,
    connection_id: response.item?.connectionId ?? null,
    level: response.item?.level ?? 'unknown',
    summary: response.item?.summary ?? '',
    last_ok_at: response.item?.lastOkAt ?? null,
    last_failure_at: response.item?.lastFailureAt ?? null,
    failure_count: response.item?.failureCount ?? 0,
    consecutive_failure_count: response.item?.consecutiveFailureCount ?? 0,
    next_retry_at: response.item?.nextRetryAt ?? null,
    evidence: parseJsonObject(response.item?.evidenceJson),
    updated_at: response.item?.updatedAt ?? ''
  }
}

export async function fetchSignalHubRuntimeStates(): Promise<SignalHubRuntimeStatesResponse> {
  const response = await getSignalHubConnectClient().listRuntimeStates({})
  return {
    items: response.items.map((item) => ({
      id: item.id,
      source_code: item.sourceCode,
      connection_id: item.connectionId ?? null,

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/settings/components/SignalHubSettings.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/SignalHubSettings.boundary.test.ts`
- Size bytes / Размер в байтах: `1883`
- Included characters / Включено символов: `1883`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SignalHubSettings boundary', () => {
  it('uses Settings-local queries and does not import integration internals directly', () => {
    const source = [
      readFileSync(new URL('./SignalHubSettings.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./useSignalHubSettingsController.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(source).toContain("../queries/useSignalHubQuery")
    expect(source).toContain("../lib/signalHubReplay")
    expect(source).toContain("../types/signalHub")
    expect(source).not.toContain("/integrations/")
    expect(source).not.toContain("../integrations/")
    expect(source).not.toContain("../../integrations/")
    expect(source).not.toContain("ApiClient")
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })

  it('renders Signal Hub diagnostics from Settings-domain data instead of flattening them away', () => {
    const source = [
      readFileSync(new URL('./SignalHubSettings.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./SignalHubOperationsTab.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./SignalHubSourcesTab.vue', import.meta.url), 'utf8'),
      readFileSync(new URL('./useSignalHubSettingsController.ts', import.meta.url), 'utf8')
    ].join('\n')

    expect(source).toContain('formatSettingsSummary(t, connection)')
    expect(source).toContain('formatRuntimeTimeline(t, runtime)')
    expect(source).toContain('formatRuntimeError(t, runtime)')
    expect(source).toContain('formatHealthStatus(t, connections, item)')
    expect(source).toContain('formatHealthEvidence(t, item)')
    expect(source).toContain('sourceControlState(policies, source)')
    expect(source).toContain('useSignalHubCapabilitiesQuery')
    expect(source).toContain('selectedSourceCapabilities')
  })
})
```

### `frontend/src/domains/settings/components/signalHubSettingsPresentation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/signalHubSettingsPresentation.ts`
- Size bytes / Размер в байтах: `8586`
- Included characters / Включено символов: `8574`
- Truncated / Обрезано: `no`

```typescript
import type {
  SignalHubConnection,
  SignalHubCapability,
  SignalHubHealth,
  SignalHubPolicyMode,
  SignalHubPolicyScope,
  SignalHubProfilePolicy,
  SignalHubRuntimeState,
  SignalHubSource
} from '../types/signalHub'

type Translator = (value: string) => string

export function capabilityLabels(source: SignalHubSource): string[] {
  const labels: string[] = []
  if (source.supports_connections) labels.push('Connections')
  if (source.supports_runtime) labels.push('Runtime')
  if (source.supports_replay) labels.push('Replay')
  if (source.supports_pause) labels.push('Pause')
  if (source.supports_mute) labels.push('Mute')
  return labels
}

export function capabilityTone(state: string): string {
  if (state === 'available') return 'good'
  if (state === 'degraded') return 'warn'
  if (state === 'blocked' || state === 'unsupported') return 'bad'
  return 'neutral'
}

export function sourceControlState(
  policies: Array<{ scope: SignalHubPolicyScope; mode: SignalHubPolicyMode; source_code: string | null }>,
  source: SignalHubSource
): 'running' | 'muted' | 'paused' | 'disabled' | 'off' {
  const relevantPolicies = policies.filter(
    (policy) =>
      (policy.scope === 'global' || policy.scope === 'source') &&
      (policy.scope !== 'source' || policy.source_code === source.code)
  )
  if (relevantPolicies.some((policy) => policy.mode === 'disabled')) return 'disabled'
  if (relevantPolicies.some((policy) => policy.mode === 'paused')) return 'paused'
  if (relevantPolicies.some((policy) => policy.mode === 'muted')) return 'muted'
  return source.default_enabled ? 'running' : 'off'
}

export function sourceIcon(source: SignalHubSource): string {
  const icons: Record<string, string> = {
    browser: 'tabler:browser',
    calendar: 'tabler:calendar',
    filesystem: 'tabler:folder',
    fixture: 'tabler:test-pipe',
    github: 'tabler:brand-github',
    home_assistant: 'tabler:home-cog',
    ai: 'tabler:sparkles',
    mail: 'tabler:mail',
    rss: 'tabler:rss',
    system: 'tabler:settings-cog',
    telegram: 'tabler:brand-telegram',
    voice: 'tabler:microphone',
    whatsapp: 'tabler:brand-whatsapp'
  }
  return icons[source.code] ?? 'tabler:plug'
}

export function sourceIconForCode(sources: SignalHubSource[], sourceCode: string): string {
  const source = sources.find((item) => item.code === sourceCode)
  return source ? sourceIcon(source) : 'tabler:plug'
}

export function statusTone(status: string): string {
  if (status === 'connected' || status === 'ready') return 'good'
  if (status === 'paused' || status === 'degraded') return 'warn'
  if (status === 'error' || status === 'disconnected') return 'bad'
  return 'neutral'
}

export function sourceStateTone(state: string): string {
  if (state === 'running') return 'good'
  if (state === 'paused' || state === 'muted') return 'warn'
  if (state === 'disabled' || state === 'off') return 'bad'
  return 'neutral'
}

export function healthTone(level: string): string {
  if (level === 'healthy') return 'good'
  if (level === 'degraded' || level === 'warning') return 'warn'
  if (level === 'blocked' || level === 'error' || level === 'failed') return 'bad'
  return 'neutral'
}

export function runtimeTone(state: string): string {
  if (state === 'running') return 'good'
  if (state === 'paused' || state === 'muted' || state === 'stopping') return 'warn'
  if (state === 'error' || state === 'stopped') return 'bad'
  return 'neutral'
}

export function connectionLabel(
  t: Translator,
  connections: SignalHubConnection[],
  connectionId: string | null | undefined
): string {
  if (!connectionId) return t('No connection')
  const connection = connections.find((item) => item.id === connectionId)
  if (!connection) return connectionId
  return `${connection.display_name} (${connection.source_code})`
}

function summarizeObjectEntries(
  t: Translator,
  value: Record<string, unknown> | null | undefined,
  maxEntries = 3
): string[] {
  if (!value) return []
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue != null)
  return entries.slice(0, maxEntries).map(([key, entryValue]) => `${key}: ${formatSummaryValue(t, entryValue)}`)
}

function formatSummaryValue(t: Translator, value: unknown): string {
  if (typeof value === 'string') return value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  if (Array.isArray(value)) return `${value.length} ${t('items')}`
  if (value && typeof value === 'object') return t('structured')
  return t('unknown')
}

export function formatSettingsSummary(t: Translator, connection: SignalHubConnection): string {
  const entries = summarizeObjectEntries(t, connection.settings)
  if (entries.length === 0) return t('No non-secret settings')
  const totalEntries = Object.keys(connection.settings).length
  const suffix = totalEntries > entries.length ? ` +${totalEntries - entries.length} ${t('more')}` : ''
  return `${entries.join(' • ')}${suffix}`
}

export function formatConnectionTimeline(t: Translator, connection: SignalHubConnection): string {
  const checkpoints = [
    connection.connected_at ? `${t('Connected')} ${connection.connected_at}` : null,
    connection.last_seen_at ? `${t('Seen')} ${connection.last_seen_at}` : null,
    connection.last_signal_at ? `${t('Signal')} ${connection.last_signal_at}` : null,
    connection.last_sync_at ? `${t('Sync')} ${connection.last_sync_at}` : null
  ].filter((value): value is string => Boolean(value))
  return checkpoints.length > 0 ? checkpoints.join(' • ') : t('No activity recorded')
}

export function formatRuntimeTimeline(t: Translator, runtime: SignalHubRuntimeState): string {
  const checkpoints = [
    runtime.last_started_at ? `${t('Started')} ${runtime.last_started_at}` : null,
    runtime.last_stopped_at ? `${t('Stopped')} ${runtime.last_stopped_at}` : null,
    runtime.last_heartbeat_at ? `${t('Heartbeat')} ${runtime.last_heartbeat_at}` : null
  ].filter((value): value is string => Boolean(value))
  return checkpoints.length > 0 ? checkpoints.join(' • ') : t('No runtime telemetry yet')
}

export function formatRuntimeError(t: Translator, runtime: SignalHubRuntimeState): string | null {
  if (!runtime.last_error_at && !runtime.last_error_code && !runtime.last_error_message_redacted) {
    return null
  }
  return [runtime.last_error_code ?? t('error'), runtime.last_error_message_redacted, runtime.last_error_at]
    .filter((value): value is string => Boolean(value))
    .join(' • ')
}

export function formatHealthStatus(
  t: Translator,
  connections: SignalHubConnection[],
  item: SignalHubHealth
): string {
  const fragments = [
    item.connection_id ? connectionLabel(t, connections, item.connection_id) : null,
    item.last_ok_at ? `${t('Last OK')} ${item.last_ok_at}` : null,
    item.last_failure_at ? `${t('Last failure')} ${item.last_failure_at}` : null,
    item.failure_count > 0 ? `${t('Failures')} ${item.failure_count}` : null,
    item.consecutive_failure_count > 0 ? `${t('Consecutive')} ${item.consecutive_failure_count}` : null
  ].filter((value): value is string => Boolean(value))
  return fragments.length > 0 ? fragments.join(' • ') : t('No health history')
}

export function formatHealthEvidence(t: Translator, item: SignalHubHealth): string | null {
  const entries = summarizeObjectEntries(t, item.evidence)
  if (entries.length === 0) return null
  const totalEntries = Object.keys(item.evidence).length
  const suffix = totalEntries > entries.length ? ` +${totalEntries - entries.length} ${t('more')}` : ''
  return `${entries.join(' • ')}${suffix}`
}

export function policyTargetLabel(
  t: Translator,
  connections: SignalHubConnection[],
  policy: {
    scope: SignalHubPolicyScope
    source_code: string | null
    connection_id: string | null
    event_pattern: string | null
  }
): string {
  if (policy.scope === 'connection' && policy.connection_id) {
    return connectionLabel(t, connections, policy.connection_id)
  }
  return policy.event_pattern ?? policy.source_code ?? policy.scope
}

export function profilePolicyLabel(
  t: Translator,
  connections: SignalHubConnection[],
  policy: SignalHubProfilePolicy
): string {
  if (policy.scope === 'connection' && policy.connection_id) {
    return `${policy.mode} / ${connectionLabel(t, connections, policy.connection_id)}`
  }
  return `${policy.mode} / ${policy.event_pattern ?? policy.source_code ?? policy.scope}`
}

export function capabilityLabel(capability: SignalHubCapability): string {
  return `${capability.capability} / ${capability.action_class}`
}
```

### `frontend/src/domains/settings/components/useSignalHubSettingsController.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/components/useSignalHubSettingsController.ts`
- Size bytes / Размер в байтах: `19801`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { computed, ref } from 'vue'
import {
  useApplySignalHubProfileMutation,
  useCreateSignalHubProfileMutation,
  useCreateSignalHubConnectionMutation,
  useCreateSignalHubPolicyMutation,
  useRemoveSignalHubProfileMutation,
  useCreateSignalHubReplayRequestMutation,
  useDisableSignalHubMutation,
  useDisableSignalHubSourceMutation,
  useEmitSignalHubFixtureMutation,
  useEnableSignalHubMutation,
  useEnableSignalHubSourceMutation,
  useSignalHubCapabilitiesQuery,
  useMuteSignalHubMutation,
  usePauseSignalHubMutation,
  useRemoveSignalHubConnectionMutation,
  useResumeSignalHubMutation,
  useRunSignalHubHealthCheckMutation,
  useSignalHubConnectionsQuery,
  useSignalHubFixtureSourcesQuery,
  useSignalHubHealthQuery,
  useSignalHubProfilesQuery,
  useSignalHubReplayRequestsQuery,
  useSignalHubRuntimeStatesQuery,
  useRestoreSignalHubFixtureMutation,
  useSignalHubPoliciesQuery,
  useSignalHubSourcesQuery,
  useUnmuteSignalHubMutation,
  useUpdateSignalHubConnectionMutation,
  useUpdateSignalHubProfileMutation,
  useUpdateSignalHubRuntimeStateMutation
} from '../queries/useSignalHubQuery'
import {
  buildSignalHubReplayRequest,
  describeSignalHubReplayRequest,
  type SignalHubReplaySelectorMode
} from '../lib/signalHubReplay'
import type {
  SignalHubProfile,
  SignalHubProfilePolicy,
  SignalHubPolicyMode,
  SignalHubPolicyScope,
  SignalHubRuntimeState
} from '../types/signalHub'
import { sourceControlState } from './signalHubSettingsPresentation'

export type SignalHubTab =
  | 'sources'
  | 'profiles'
  | 'connections'
  | 'runtime'
  | 'policies'
  | 'health'
  | 'replay'

export function useSignalHubSettingsController() {
  const { data: sourcesData, isLoading } = useSignalHubSourcesQuery()
  const { data: capabilitiesData } = useSignalHubCapabilitiesQuery()
  const { data: connectionsData } = useSignalHubConnectionsQuery()
  const { data: fixtureSourcesData } = useSignalHubFixtureSourcesQuery()
  const { data: profilesData } = useSignalHubProfilesQuery()
  const { data: runtimeData } = useSignalHubRuntimeStatesQuery()
  const { data: healthData } = useSignalHubHealthQuery()
  const { data: replayData } = useSignalHubReplayRequestsQuery()
  const { data: policiesData } = useSignalHubPoliciesQuery()
  const restoreFixture = useRestoreSignalHubFixtureMutation()
  const emitFixture = useEmitSignalHubFixtureMutation()
  const applyProfile = useApplySignalHubProfileMutation()
  const createProfile = useCreateSignalHubProfileMutation()
  const runHealthCheck = useRunSignalHubHealthCheckMutation()
  const createConnection = useCreateSignalHubConnectionMutation()
  const createPolicy = useCreateSignalHubPolicyMutation()
  const enableSource = useEnableSignalHubSourceMutation()
  const disableSource = useDisableSignalHubSourceMutation()
  const enableSignals = useEnableSignalHubMutation()
  const disableSignals = useDisableSignalHubMutation()
  const muteSignals = useMuteSignalHubMutation()
  const unmuteSignals = useUnmuteSignalHubMutation()
  const pauseSignals = usePauseSignalHubMutation()
  const resumeSignals = useResumeSignalHubMutation()
  const createReplayRequest = useCreateSignalHubReplayRequestMutation()
  const updateConnection = useUpdateSignalHubConnectionMutation()
  const removeConnection = useRemoveSignalHubConnectionMutation()
  const updateProfile = useUpdateSignalHubProfileMutation()
  const removeProfile = useRemoveSignalHubProfileMutation()
  const updateRuntime = useUpdateSignalHubRuntimeStateMutation()

  const activeTab = ref<SignalHubTab>('sources')
  const selectedSourceCode = ref<string | null>(null)
  const selectedProfileCode = ref<string | null>(null)
  const sourceSearch = ref('')
  const sourceCategory = ref('all')
  const policyScope = ref<SignalHubPolicyScope>('event_pattern')
  const policyMode = ref<SignalHubPolicyMode>('paused')
  const policySourceCode = ref('system')
  const policyConnectionId = ref('')
  const policyEventPattern = ref('signal.raw.*')
  const policyReason = ref('owner policy')
  const connectionSourceCode = ref('telegram')
  const connectionDisplayName = ref('Primary Connection')
  const connectionProfile = ref('default')
  const profileCodeInput = ref('')
  const profileDisplayNameInput = ref('')
  const profileDescriptionInput = ref('')
  const profilePolicyScope = ref<SignalHubPolicyScope>('source')
  const profilePolicyMode = ref<SignalHubPolicyMode>('muted')
  const profilePolicySourceCode = ref('telegram')
  const profilePolicyConnectionId = ref('')
  const profilePolicyEventPattern = ref('signal.raw.*')
  const profilePolicyReason = ref('profile policy')
  const profileDraftPolicies = ref<SignalHubProfilePolicy[]>([])
  const replaySourceCode = ref('telegram')
  const replayConnectionId = ref('')
  const replayEventPattern = ref('signal.raw.telegram.*')
  const replayTargetConsumer = ref('')
  const replayTargetProjection = ref('')
  const replaySelectorMode = ref<SignalHubReplaySelectorMode>('all')
  const replayFromPosition = ref('')
  const replayToPosition = ref('')
  const replayFromTime = ref('')
  const replayToTime = ref('')
  const fixtureSignalId = ref('fixture_basic_message')

  const tabs: Array<{ id: SignalHubTab; label: string; icon: string }> = [
    { id: 'sources', label: 'Sources', icon: 'tabler:database-import' },
    { id: 'profiles', label: 'Profiles', icon: 'tabler:layout-dashboard' },
    { id: 'connections', label: 'Connections', icon: 'tabler:plug-connected' },
    { id: 'runtime', label: 'Runtime', icon: 'tabler:player-play' },
    { id: 'policies', label: 'Policies', icon: 'tabler:shield-cog' },
    { id: 'health', label: 'Health', icon: 'tabler:activity-heartbeat' },
    { id: 'replay', label: 'Replay', icon: 'tabler:player-track-next' }
  ]

  const sources = computed(() => sourcesData.value?.items ?? [])
  const policies = computed(() => policiesData.value?.items ?? [])
  const profiles = computed(() => profilesData.value?.items ?? [])
  const capabilityItems = computed(() => capabilitiesData.value?.items ?? [])
  const connections = computed(() => connectionsData.value?.items ?? [])
  const runtimeStates = computed(() => runtimeData.value?.items ?? [])
  const healthItems = computed(() => healthData.value?.items ?? [])
  const replayRequests = computed(() => replayData.value?.items ?? [])
  const fixtureSources = computed(() => fixtureSourcesData.value?.items ?? [])
  const replayTargetConsumers = computed(() =>
    Array.from(
      new Set(
        runtimeStates.value
          .map((runtime) => runtime.runtime_kind.trim())
          .filter((runtimeKind) => runtimeKind.length > 0)
      )
    ).sort()
  )
  const categories = computed(() => {
    const values = new Set(sources.value.map((source) => source.category))
    return ['all', ...Array.from(values).sort()]
  })
  const filteredSources = computed(() => {
    const search = sourceSearch.value.trim().toLowerCase()
    return sources.value.filter((source) => {
      const matchesCategory =
        sourceCategory.value === 'all' || source.category === sourceCategory.value
      const matchesSearch =
        search.length === 0 ||
        source.code.toLowerCase().includes(search) ||
        source.display_name.toLowerCase().includes(search)
      return matchesCategory && matchesSearch
    })
  })
  const connectionCapableSources = computed(() =>
    sources.value.filter((source) => source.supports_connections)
  )
  const policyScopeConnections = computed(() =>
    connections.value.filter((connection) =>
      policyScope.value === 'connection' && policySourceCode.value.trim().length > 0
        ? connection.source_code === policySourceCode.value
        : true
    )
  )
  const profileScopeConnections = computed(() =>
    connections.value.filter((connection) =>
      profilePolicyScope.value === 'connection' && profilePolicySourceCode.value.trim().length > 0
        ? connection.source_code === profilePolicySourceCode.value
        : true
    )
  )
  const replayScopedConnections = computed(() =>
    connections.value.filter((connection) =>
      replaySourceCode.value.trim().length > 0 ? connection.source_code === replaySourceCode.value : true
    )
  )
  const selectedSource = computed(() => {
    const selectedCode = selectedSourceCode.value
    if (!selectedCode) return filteredSources.value[0] ?? null
    return sources.value.find((source) => source.code === selectedCode) ?? null
  })
  const selectedProfile = computed(() => {
    const selectedCode = selectedProfileCode.value
    if (!selectedCode) return null
    return profiles.value.find((profile) => profile.code === selectedCode) ?? null
  })
  const selectedSourceCapabilities = computed(() =>
    selectedSource.value
      ? capabilityItems.value.filter(
          (capability) =>
            capability.source_code === selectedSource.value?.code && capability.connection_id === null
        )
      : []
  )
  const enabledCount = computed(
    () => sources.value.filter((source) => sourceControlState(policies.value, source) === 'running').length
  )
  const runtimeCount = computed(() => sources.value.filter((source) => source.supports_runtime).length)
  const activeRuntimeCount = computed(
    () => runtimeStates.value.filter((runtime) => runtime.state === 'running').length
  )
  const replayCount = computed(() => sources.value.filter((source) => source.supports_replay).length)
  const connectedCount = computed(
    () => connections.value.filter((connection) => connection.status === 'connected').length
  )
  const activeProfile = computed(() => profiles.value.find((profile) => profile.is_active) ?? null)
  const unhealthyCount = computed(() => healthItems.value.filter((item) => item.level !== 'healthy').length)
  const replayPendingCount = computed(
    () =>
      replayRequests.value.filter(
        (request) => request.status !== 'completed' && request.status !== 'failed'
      ).length
  )
  const isRestoringFixture = computed(() => restoreFixture.isPending.value)
  const isEmittingFixture = computed(() => emitFixture.isPending.value)
  const isApplyingProfile = computed(() => applyProfile.isPending.value)
  const isSavingProfile = computed(() => createProfile.isPending.value || updateProfile.isPending.value)
  const isRemovingProfile = computed(() => removeProfile.isPending.value)
  const isRunningHealthCheck = computed(() => runHealthCheck.isPending.value)
  const isCreatingConnection = computed(() => createConnection.isPending.value)
  const isCreatingPolicy = computed(() => createPolicy.isPending.value)
  const isUpdatingSignalControls = computed(
    () =>
      enableSource.isPending.value ||
      disableSource.isPending.value ||
      muteSignals.isPending.value ||
      unmuteSignals.isPending.value ||
      pauseSignals.isPending.value ||
      resumeSignals.isPending.value
  )
  const isCreatingReplayRequest = computed(() => createReplayRequest.isPending.value)
  const isUpdatingConnection = computed(() => updateConnection.isPending.value || removeConnection.isPending.value)
  const isUpdatingRuntime = computed(() => updateRuntime.isPending.value)

  async function handleRestoreFixture() {
    await restoreFixture.mutateAsync()
  }

  async function handleEmitFixtureSignal() {
    await emitFixture.mutateAsync(fixtureSignalId.value.trim())
  }

  async function handleApplyProfile(profileCode: string) {
    await applyProfile.mutateAsync(profileCode)
  }

  function resetProfileEditor() {
    selectedProfileCode.value = null
    profileCodeInput.value = ''
    profileDisplayNameInput.value = ''
    profileDescriptionInput.value = ''
    profileDraftPolicies.value = []
    profilePolicyScope.value = 'source'
    profilePolicyMode.value = 'muted'
    profilePolicySourceCode.value = connectionCapableSources.value[0]?.code ?? 'telegram'
    profilePolicyConnectionId.value = ''
    profilePolicyEventPattern.value = 'signal.raw.*'
    profilePolicyReason.value = 'profile policy'
  }

  function handleSelectProfile(profile: SignalHubProfile) {
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/settings/lib/signalHubReplay.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/lib/signalHubReplay.test.ts`
- Size bytes / Размер в байтах: `4381`
- Included characters / Включено символов: `4381`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  buildSignalHubReplayRequest,
  describeSignalHubReplayRequest
} from './signalHubReplay'

describe('signalHubReplay helpers', () => {
  it('builds a replay request without selectors for full pattern replay', () => {
    const request = buildSignalHubReplayRequest({
      source_code: ' telegram ',
      event_pattern: ' signal.raw.telegram.* ',
      target_consumer: ' signal_hub_raw_signal_dispatcher ',
      selector_mode: 'all'
    })

    expect(request).toMatchObject({
      source_code: 'telegram',
      event_pattern: 'signal.raw.telegram.*',
      target_consumer: 'signal_hub_raw_signal_dispatcher',
      metadata: {
        requested_from: 'settings_signal_hub',
        selector_mode: 'all'
      }
    })
    expect('from_position' in request).toBe(false)
    expect('to_position' in request).toBe(false)
    expect('from_time' in request).toBe(false)
    expect('to_time' in request).toBe(false)
  })

  it('builds a replay request with position selectors', () => {
    const request = buildSignalHubReplayRequest({
      source_code: 'telegram',
      event_pattern: 'signal.raw.telegram.*',
      selector_mode: 'position',
      from_position: '10',
      to_position: '20'
    })

    expect(request.from_position).toBe(10n)
    expect(request.to_position).toBe(20n)
    expect(request.from_time).toBeUndefined()
    expect(request.to_time).toBeUndefined()
  })

  it('builds a replay request with time selectors', () => {
    const request = buildSignalHubReplayRequest({
      source_code: 'telegram',
      event_pattern: 'signal.raw.telegram.*',
      selector_mode: 'time',
      from_time: '2026-06-23T00:00:00Z',
      to_time: '2026-06-23T01:00:00Z'
    })

    expect(request.from_position).toBeUndefined()
    expect(request.to_position).toBeUndefined()
    expect(request.from_time).toBe('2026-06-23T00:00:00Z')
    expect(request.to_time).toBe('2026-06-23T01:00:00Z')
  })

  it('describes replay selectors for the list row', () => {
    const description = describeSignalHubReplayRequest({
      id: 'replay-1',
      source_code: 'telegram',
      connection_id: null,
      event_pattern: 'signal.raw.telegram.*',
      from_position: 10n,
      to_position: 20n,
      from_time: null,
      to_time: null,
      target_consumer: 'signal_hub_raw_signal_dispatcher',
      target_projection: 'timeline_event_log',
      status: 'queued',
      requested_by: 'makosh-frontend',
      requested_at: '2026-06-23T00:00:00Z',
      started_at: null,
      completed_at: null,
      last_error_redacted: null,
      replayed_count: 0,
      metadata: {}
    })

    expect(description).toBe(
      'pos 10..20 / consumer signal_hub_raw_signal_dispatcher / projection timeline_event_log / 2026-06-23T00:00:00Z'
    )
  })

  it('builds a projection-targeted replay request with optional source filters', () => {
    const request = buildSignalHubReplayRequest({
      source_code: '',
      event_pattern: '',
      target_projection: ' communication_messages ',
      selector_mode: 'all'
    })

    expect(request.source_code).toBeNull()
    expect(request.event_pattern).toBeNull()
    expect(request.target_projection).toBe('communication_messages')
  })

  it('builds a connection-scoped replay request', () => {
    const request = buildSignalHubReplayRequest({
      source_code: ' mail ',
      connection_id: ' conn-1 ',
      event_pattern: ' signal.raw.mail.* ',
      selector_mode: 'all'
    })

    expect(request.source_code).toBe('mail')
    expect(request.connection_id).toBe('conn-1')
    expect(request.event_pattern).toBe('signal.raw.mail.*')
  })

  it('includes connection scope in replay description', () => {
    const description = describeSignalHubReplayRequest({
      id: 'replay-2',
      source_code: 'mail',
      connection_id: 'conn-1',
      event_pattern: 'signal.raw.mail.*',
      from_position: null,
      to_position: null,
      from_time: null,
      to_time: null,
      target_consumer: null,
      target_projection: null,
      status: 'queued',
      requested_by: 'makosh-frontend',
      requested_at: '2026-06-23T00:00:00Z',
      started_at: null,
      completed_at: null,
      last_error_redacted: null,
      replayed_count: 0,
      metadata: {}
    })

    expect(description).toBe('connection conn-1 / 2026-06-23T00:00:00Z')
  })
})
```

### `frontend/src/domains/settings/lib/signalHubReplay.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/lib/signalHubReplay.ts`
- Size bytes / Размер в байтах: `2759`
- Included characters / Включено символов: `2759`
- Truncated / Обрезано: `no`

```typescript
import type {
  SignalHubReplayRequest,
  SignalHubReplayRequestCreateRequest
} from '../types/signalHub'

export type SignalHubReplaySelectorMode = 'all' | 'position' | 'time'

export interface BuildSignalHubReplayRequestInput {
  source_code?: string
  connection_id?: string
  event_pattern?: string
  selector_mode: SignalHubReplaySelectorMode
  target_consumer?: string
  target_projection?: string
  from_position?: string
  to_position?: string
  from_time?: string
  to_time?: string
}

export function buildSignalHubReplayRequest(
  input: BuildSignalHubReplayRequestInput
): SignalHubReplayRequestCreateRequest {
  const request: SignalHubReplayRequestCreateRequest = {
    source_code: normalizeOptionalText(input.source_code),
    connection_id: normalizeOptionalText(input.connection_id),
    event_pattern: normalizeOptionalText(input.event_pattern),
    target_consumer: normalizeOptionalText(input.target_consumer),
    target_projection: normalizeOptionalText(input.target_projection),
    metadata: {
      requested_from: 'settings_signal_hub',
      selector_mode: input.selector_mode
    }
  }

  if (input.selector_mode === 'position') {
    request.from_position = parseOptionalBigInt(input.from_position)
    request.to_position = parseOptionalBigInt(input.to_position)
  }

  if (input.selector_mode === 'time') {
    request.from_time = normalizeOptionalText(input.from_time)
    request.to_time = normalizeOptionalText(input.to_time)
  }

  return request
}

export function describeSignalHubReplayRequest(request: SignalHubReplayRequest): string {
  const selectors: string[] = []

  if (request.connection_id) {
    selectors.push(`connection ${request.connection_id}`)
  }

  if (request.from_position !== null || request.to_position !== null) {
    selectors.push(
      `pos ${request.from_position?.toString() ?? '...'}..${request.to_position?.toString() ?? '...'}`
    )
  }

  if (request.from_time !== null || request.to_time !== null) {
    selectors.push(`time ${request.from_time ?? '...'}..${request.to_time ?? '...'}`)
  }

  if (request.target_consumer) {
    selectors.push(`consumer ${request.target_consumer}`)
  }

  if (request.target_projection) {
    selectors.push(`projection ${request.target_projection}`)
  }

  if (selectors.length === 0) {
    return request.requested_at
  }

  selectors.push(request.requested_at)
  return selectors.join(' / ')
}

function parseOptionalBigInt(value: string | undefined): bigint | null {
  const normalized = normalizeOptionalText(value)
  if (!normalized) return null
  return BigInt(normalized)
}

function normalizeOptionalText(value: string | undefined): string | null {
  const normalized = value?.trim() ?? ''
  return normalized.length > 0 ? normalized : null
}
```

### `frontend/src/domains/settings/queries/useSettingsQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/queries/useSettingsQuery.ts`
- Size bytes / Размер в байтах: `2124`
- Included characters / Включено символов: `2124`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import {
  fetchApplicationSettings,
  fetchProviderAccounts,
  fetchCalendarAccounts
} from '../api/settings'
import type { ApplicationSetting } from '../types/settings'

export const settingsKeys = {
  all: ['settings'] as const,
  application: () => [...settingsKeys.all, 'application'] as const,
  providerAccounts: () => [...settingsKeys.all, 'provider-accounts'] as const,
  calendarAccounts: () => [...settingsKeys.all, 'calendar-accounts'] as const,
  workspace: () => [...settingsKeys.all, 'workspace'] as const
}

export function useApplicationSettingsQuery() {
  return useQuery({
    queryKey: settingsKeys.application(),
    queryFn: fetchApplicationSettings
  })
}

export function useProviderAccountsQuery() {
  return useQuery({
    queryKey: settingsKeys.providerAccounts(),
    queryFn: fetchProviderAccounts
  })
}

export function useCalendarAccountsQuery() {
  return useQuery({
    queryKey: settingsKeys.calendarAccounts(),
    queryFn: fetchCalendarAccounts
  })
}

export function useSettingsWorkspaceQuery() {
  return useQuery({
    queryKey: settingsKeys.workspace(),
    queryFn: async () => {
      const [appSettings, providerAccounts, calendarAccounts] = await Promise.all([
        fetchApplicationSettings(),
        fetchProviderAccounts(),
        fetchCalendarAccounts()
      ])
      return { appSettings, providerAccounts, calendarAccounts }
    }
  })
}

/** Find a specific setting by key from a settings list. */
export function findSetting(
  settings: ApplicationSetting[] | undefined,
  key: string
): ApplicationSetting | null {
  if (!settings) return null
  return settings.find((s) => s.setting_key === key) ?? null
}

/** Group settings by category. */
export function groupSettingsByCategory(
  settings: ApplicationSetting[] | undefined
): Record<string, ApplicationSetting[]> {
  if (!settings) return {}
  const groups: Record<string, ApplicationSetting[]> = {}
  for (const setting of settings) {
    const cat = setting.category
    if (!groups[cat]) groups[cat] = []
    groups[cat].push(setting)
  }
  return groups
}
```

### `frontend/src/domains/settings/queries/useSignalHubQuery.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/queries/useSignalHubQuery.test.ts`
- Size bytes / Размер в байтах: `853`
- Included characters / Включено символов: `853`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { signalHubKeys } from './useSignalHubQuery'

describe('Signal Hub query keys', () => {
  it('keeps all Signal Hub queries under one stable namespace', () => {
    expect(signalHubKeys.all).toEqual(['signal-hub'])
    expect(signalHubKeys.sources()).toEqual(['signal-hub', 'sources'])
    expect(signalHubKeys.connections()).toEqual(['signal-hub', 'connections'])
    expect(signalHubKeys.runtimes()).toEqual(['signal-hub', 'runtimes'])
    expect(signalHubKeys.health()).toEqual(['signal-hub', 'health'])
    expect(signalHubKeys.replay()).toEqual(['signal-hub', 'replay'])
    expect(signalHubKeys.policies()).toEqual(['signal-hub', 'policies'])
    expect(signalHubKeys.profiles()).toEqual(['signal-hub', 'profiles'])
    expect(signalHubKeys.fixture()).toEqual(['signal-hub', 'fixture'])
  })
})
```

### `frontend/src/domains/settings/queries/useSignalHubQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/queries/useSignalHubQuery.ts`
- Size bytes / Размер в байтах: `8925`
- Included characters / Включено символов: `8925`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import {
  applySignalHubProfile,
  createSignalHubProfile,
  createSignalHubReplayRequest,
  createSignalHubPolicy,
  createSignalHubConnection,
  disableSignalHubSignals,
  disableSignalHubSource,
  emitSignalHubFixtureSignal,
  enableSignalHubSignals,
  enableSignalHubSource,
  fetchSignalHubCapabilities,
  fetchSignalHubFixtureSources,
  fetchSignalHubProfiles,
  fetchSignalHubConnections,
  fetchSignalHubHealth,
  fetchSignalHubPolicies,
  fetchSignalHubReplayRequests,
  muteSignalHubSignals,
  pauseSignalHubSignals,
  resumeSignalHubSignals,
  fetchSignalHubRuntimeStates,
  fetchSignalHubSources,
  removeSignalHubConnection,
  removeSignalHubProfile,
  restoreSignalHubSystemFixture,
  runSignalHubHealthCheck,
  unmuteSignalHubSignals,
  updateSignalHubConnection,
  updateSignalHubProfile,
  updateSignalHubRuntimeState
} from '../api/signalHub'

export const signalHubKeys = {
  all: ['signal-hub'] as const,
  sources: () => [...signalHubKeys.all, 'sources'] as const,
  capabilities: () => [...signalHubKeys.all, 'capabilities'] as const,
  connections: () => [...signalHubKeys.all, 'connections'] as const,
  runtimes: () => [...signalHubKeys.all, 'runtimes'] as const,
  health: () => [...signalHubKeys.all, 'health'] as const,
  replay: () => [...signalHubKeys.all, 'replay'] as const,
  policies: () => [...signalHubKeys.all, 'policies'] as const,
  profiles: () => [...signalHubKeys.all, 'profiles'] as const,
  fixture: () => [...signalHubKeys.all, 'fixture'] as const
}

export function useSignalHubSourcesQuery() {
  return useQuery({
    queryKey: signalHubKeys.sources(),
    queryFn: fetchSignalHubSources
  })
}

export function useSignalHubCapabilitiesQuery() {
  return useQuery({
    queryKey: signalHubKeys.capabilities(),
    queryFn: fetchSignalHubCapabilities
  })
}

export function useRestoreSignalHubFixtureMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: restoreSignalHubSystemFixture,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubFixtureSourcesQuery() {
  return useQuery({
    queryKey: signalHubKeys.fixture(),
    queryFn: fetchSignalHubFixtureSources
  })
}

export function useEmitSignalHubFixtureMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: emitSignalHubFixtureSignal,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubConnectionsQuery() {
  return useQuery({
    queryKey: signalHubKeys.connections(),
    queryFn: fetchSignalHubConnections
  })
}

export function useSignalHubProfilesQuery() {
  return useQuery({
    queryKey: signalHubKeys.profiles(),
    queryFn: fetchSignalHubProfiles
  })
}

export function useApplySignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: applySignalHubProfile,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useCreateSignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubProfile,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUpdateSignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      profileCode,
      request
    }: {
      profileCode: string
      request: Parameters<typeof updateSignalHubProfile>[1]
    }) => updateSignalHubProfile(profileCode, request),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useRemoveSignalHubProfileMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: removeSignalHubProfile,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useCreateSignalHubConnectionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubConnection,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUpdateSignalHubConnectionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      connectionId,
      request
    }: {
      connectionId: string
      request: Parameters<typeof updateSignalHubConnection>[1]
    }) => updateSignalHubConnection(connectionId, request),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useRemoveSignalHubConnectionMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: removeSignalHubConnection,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubHealthQuery() {
  return useQuery({
    queryKey: signalHubKeys.health(),
    queryFn: fetchSignalHubHealth
  })
}

export function useRunSignalHubHealthCheckMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: runSignalHubHealthCheck,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubRuntimeStatesQuery() {
  return useQuery({
    queryKey: signalHubKeys.runtimes(),
    queryFn: fetchSignalHubRuntimeStates
  })
}

export function useSignalHubReplayRequestsQuery() {
  return useQuery({
    queryKey: signalHubKeys.replay(),
    queryFn: fetchSignalHubReplayRequests
  })
}

export function useCreateSignalHubReplayRequestMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubReplayRequest,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useSignalHubPoliciesQuery() {
  return useQuery({
    queryKey: signalHubKeys.policies(),
    queryFn: fetchSignalHubPolicies
  })
}

export function useCreateSignalHubPolicyMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createSignalHubPolicy,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useEnableSignalHubSourceMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: enableSignalHubSource,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useDisableSignalHubSourceMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: disableSignalHubSource,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useDisableSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: disableSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useEnableSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: enableSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useMuteSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: muteSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUnmuteSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: unmuteSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function usePauseSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: pauseSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useResumeSignalHubMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: resumeSignalHubSignals,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}

export function useUpdateSignalHubRuntimeStateMutation() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: updateSignalHubRuntimeState,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: signalHubKeys.all })
    }
  })
}
```

### `frontend/src/domains/settings/stores/settings.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/stores/settings.ts`
- Size bytes / Размер в байтах: `3233`
- Included characters / Включено символов: `3231`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ApplicationSetting } from '../types/settings'
import { saveApplicationSetting } from '../api/settings'

export type SettingsSection =
  | 'appearance'
  | 'language'
  | 'application'
  | 'sidebar'
  | 'integrations'
  | 'signal-hub'
  | 'ai'

export const useSettingsStore = defineStore('settings-ui', () => {
  // --- UI state ---
  const selectedSection = ref<SettingsSection>('appearance')
  const actionMessage = ref('')
  const errorMessage = ref('')
  const savingSettingKey = ref<string | null>(null)

  // --- Drafts for application settings ---
  const settingDrafts = ref<Record<string, string>>({})

  // --- Sidebar editing state ---
  const isSidebarSettingsSaving = ref(false)
  const sidebarError = ref('')
  const newSidebarGroupLabel = ref('')

  // --- Selected integration ---
  const selectedIntegrationId = ref<string | null>(null)

  // --- Computed ---
  const hasSidebarChanges = computed(() => false) // Placeholder — will connect to sidebar store later

  // --- Actions ---
  function selectSection(section: SettingsSection) {
    selectedSection.value = section
    actionMessage.value = ''
    errorMessage.value = ''
  }

  function setActionMessage(msg: string) {
    actionMessage.value = msg
    errorMessage.value = ''
  }

  function setError(msg: string) {
    errorMessage.value = msg
    actionMessage.value = ''
  }

  function clearMessages() {
    actionMessage.value = ''
    errorMessage.value = ''
  }

  function updateSettingDraft(key: string, value: string) {
    settingDrafts.value[key] = value
  }

  async function saveSetting(setting: ApplicationSetting) {
    savingSettingKey.value = setting.setting_key
    clearMessages()
    try {
      const draftValue = settingDrafts.value[setting.setting_key]
      const valueToSave = draftValue !== undefined ? coerceValue(draftValue, setting.value_kind) : setting.value
      await saveApplicationSetting(setting.setting_key, valueToSave)
      setActionMessage(`Saved ${setting.label}`)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save setting')
    } finally {
      savingSettingKey.value = null
    }
  }

  function selectIntegration(id: string | null) {
    selectedIntegrationId.value = id
  }

  function updateNewSidebarGroupLabel(label: string) {
    newSidebarGroupLabel.value = label
  }

  return {
    selectedSection,
    actionMessage,
    errorMessage,
    savingSettingKey,
    settingDrafts,
    isSidebarSettingsSaving,
    sidebarError,
    newSidebarGroupLabel,
    selectedIntegrationId,
    hasSidebarChanges,
    selectSection,
    setActionMessage,
    setError,
    clearMessages,
    updateSettingDraft,
    saveSetting,
    selectIntegration,
    updateNewSidebarGroupLabel
  }
})

/** Coerce a draft string value to the correct type for saving. */
function coerceValue(
  draft: string,
  kind: string
): ApplicationSetting['value'] {
  switch (kind) {
    case 'boolean':
      return draft === 'true'
    case 'integer':
      return parseInt(draft, 10) || 0
    case 'json':
      try { return JSON.parse(draft) } catch { return draft }
    default:
      return draft
  }
}
```

### `frontend/src/domains/settings/types/settings.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/types/settings.ts`
- Size bytes / Размер в байтах: `1005`
- Included characters / Включено символов: `1005`
- Truncated / Обрезано: `no`

```typescript
export type {
  ApplicationSetting,
  ApplicationSettingsResponse,
  ApplicationSettingValue,
  SettingValueKind
} from '../../../platform/settings/applicationSettingsClient'

export {
  FRONTEND_LAYOUT_SETTING_KEY,
  FRONTEND_SIDEBAR_SETTING_KEY,
  FRONTEND_LOCALE_SETTING_KEY,
  FRONTEND_THEME_SETTING_KEY,
  FRONTEND_UI_STATE_SETTING_KEY
} from '../../../platform/settings/applicationSettingsClient'

export interface ProviderAccount {
  account_id: string
  provider_kind: 'gmail' | 'icloud' | 'imap' | string
  display_name: string
  external_account_id: string
  config: Record<string, unknown>
  created_at: string
  updated_at: string
  email?: string | null
  label?: string | null
  is_active?: boolean
  is_authenticated?: boolean
  last_sync_at?: string | null
}

export interface ProviderAccountListResponse {
  items: ProviderAccount[]
}

export interface CalendarAccount {
  id: string
  provider_kind: string
  email: string
  label: string
  is_active: boolean
  calendar_ids: string[]
}
```

### `frontend/src/domains/settings/types/signalHub.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/types/signalHub.ts`
- Size bytes / Размер в байтах: `6608`
- Included characters / Включено символов: `6608`
- Truncated / Обрезано: `no`

```typescript
export interface SignalHubSource {
  id: string
  code: string
  display_name: string
  category: string
  source_kind: string
  default_enabled: boolean
  supports_connections: boolean
  supports_runtime: boolean
  supports_replay: boolean
  supports_pause: boolean
  supports_mute: boolean
  capability_schema_version: number
  created_at: string
  updated_at: string
}

export interface SignalHubSourcesResponse {
  items: SignalHubSource[]
}

export interface SignalHubCapability {
  id: string
  source_code: string
  connection_id: string | null
  capability: string
  state: string
  reason: string | null
  requires_confirmation: boolean
  action_class: string
  updated_at: string
}

export interface SignalHubCapabilitiesResponse {
  items: SignalHubCapability[]
}

export interface SignalHubFixtureSource {
  fixture_id: string
  source_code: string
  event_type: string
  correlation_id: string | null
  occurred_at: string
  summary: string
}

export interface SignalHubFixtureSourcesResponse {
  items: SignalHubFixtureSource[]
}

export interface SignalHubConnection {
  id: string
  source_code: string
  display_name: string
  status: string
  profile: string | null
  settings: Record<string, unknown>
  secret_ref: string | null
  connected_at: string | null
  last_seen_at: string | null
  last_signal_at: string | null
  last_sync_at: string | null
  created_at: string
  updated_at: string
}

export interface SignalHubConnectionsResponse {
  items: SignalHubConnection[]
}

export interface SignalHubConnectionResponse {
  item: SignalHubConnection
}

export interface SignalHubConnectionCreateRequest {
  source_code: string
  display_name: string
  status: string
  profile?: string | null
  settings?: Record<string, unknown>
  secret_ref?: string | null
}

export interface SignalHubConnectionUpdateRequest {
  display_name?: string
  status?: string
  profile?: string | null
  settings?: Record<string, unknown>
  secret_ref?: string | null
}

export interface SignalHubHealth {
  id: string
  source_code: string
  connection_id: string | null
  level: string
  summary: string
  last_ok_at: string | null
  last_failure_at: string | null
  failure_count: number
  consecutive_failure_count: number
  next_retry_at: string | null
  evidence: Record<string, unknown>
  updated_at: string
}

export interface SignalHubHealthResponse {
  items: SignalHubHealth[]
}

export interface SignalHubHealthCheckRequest {
  source_code: string
  connection_id?: string | null
}

export interface SignalHubRuntimeState {
  id: string
  source_code: string
  connection_id: string | null
  runtime_kind: string
  state: string
  last_started_at: string | null
  last_stopped_at: string | null
  last_heartbeat_at: string | null
  last_error_at: string | null
  last_error_code: string | null
  last_error_message_redacted: string | null
  metadata: Record<string, unknown>
  updated_at: string
}

export interface SignalHubRuntimeStatesResponse {
  items: SignalHubRuntimeState[]
}

export interface SignalHubReplayRequest {
  id: string
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  from_position: bigint | null
  to_position: bigint | null
  from_time: string | null
  to_time: string | null
  target_consumer: string | null
  target_projection: string | null
  status: string
  requested_by: string
  requested_at: string
  started_at: string | null
  completed_at: string | null
  last_error_redacted: string | null
  replayed_count: number
  metadata: Record<string, unknown>
}

export interface SignalHubReplayRequestsResponse {
  items: SignalHubReplayRequest[]
}

export interface SignalHubReplayRequestCreateRequest {
  source_code?: string | null
  connection_id?: string | null
  event_pattern?: string | null
  from_position?: bigint | number | string | null
  to_position?: bigint | number | string | null
  from_time?: string | null
  to_time?: string | null
  target_consumer?: string | null
  target_projection?: string | null
  metadata?: Record<string, unknown>
}

export interface SignalHubFixtureRestoreReport {
  sources_created: number
  sources_repaired: number
  profiles_created: number
  profiles_repaired: number
}

export interface SignalHubFixtureEmission {
  fixture_id: string
  raw_event_id: string
  event_type: string
  source_code: string
  correlation_id: string | null
}

export interface SignalHubProfile {
  id: string
  code: string
  display_name: string
  description: string
  policy_count: number
  source_policies: SignalHubProfilePolicy[]
  is_system: boolean
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface SignalHubProfilesResponse {
  items: SignalHubProfile[]
}

export interface SignalHubProfilePolicy {
  scope: SignalHubPolicyScope
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  mode: SignalHubPolicyMode
  reason: string
}

export interface SignalHubProfileCreateRequest {
  code: string
  display_name: string
  description: string
  source_policies: SignalHubProfilePolicy[]
}

export interface SignalHubProfileUpdateRequest {
  display_name?: string
  description?: string
  source_policies?: SignalHubProfilePolicy[]
}

export type SignalHubPolicyScope =
  | 'global'
  | 'source'
  | 'connection'
  | 'event_pattern'
  | 'profile'

export type SignalHubPolicyMode =
  | 'enabled'
  | 'disabled'
  | 'muted'
  | 'paused'
  | 'replay_only'
  | 'fixture_only'

export interface SignalHubPolicy {
  scope: SignalHubPolicyScope
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  mode: SignalHubPolicyMode
  reason: string
  expires_at: string | null
}

export interface SignalHubPoliciesResponse {
  items: SignalHubPolicy[]
}

export interface SignalHubPolicyRequest {
  scope: SignalHubPolicyScope
  source_code?: string | null
  connection_id?: string | null
  event_pattern?: string | null
  mode: SignalHubPolicyMode
  reason: string
  expires_at?: string | null
}

export interface SignalHubCreatePolicyResponse {
  id: string
}

export interface SignalHubControlRequest {
  scope: SignalHubPolicyScope
  source_code?: string | null
  connection_id?: string | null
  event_pattern?: string | null
  reason?: string | null
}

export interface SignalHubControlResponse {
  source_code: string | null
  connection_id: string | null
  event_pattern: string | null
  policy_id: string | null
  cleared_count: number
}

export interface SignalHubRuntimeStateRequest {
  source_code: string
  runtime_kind: string
  state: string
  metadata?: Record<string, unknown>
}
```

### `frontend/src/domains/settings/views/SettingsPage.signalHub.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/settings/views/SettingsPage.signalHub.boundary.test.ts`
- Size bytes / Размер в байтах: `691`
- Included characters / Включено символов: `691`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SettingsPage Signal Hub boundary', () => {
  it('keeps Signal Hub under Settings navigation instead of a standalone route', () => {
    const source = readFileSync(new URL('./SettingsPage.vue', import.meta.url), 'utf8')

    expect(source).toContain("{ id: 'signal-hub', label: 'Signal Hub'")
    expect(source).toContain("<SignalHubSettings v-else-if=\"store.selectedSection === 'signal-hub'\" />")
    expect(source).toContain("{ id: 'integrations', label: 'Integrations'")
    expect(source).not.toContain("path: '/signal-hub'")
    expect(source).not.toContain("name: 'signal-hub'")
  })
})
```

### `frontend/src/domains/tasks/api/tasks.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/tasks/api/tasks.ts`
- Size bytes / Размер в байтах: `4515`
- Included characters / Включено символов: `4515`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TaskCandidateListResponse,
  TaskCandidateReviewState,
  TaskRecordsResponse,
  Task,
  Decision,
  DecisionEntityKind,
  DecisionReviewRequest,
  DecisionListResponse,
  Obligation,
  ObligationEntityKind,
  ObligationReviewRequest,
  ObligationListResponse
} from '../types/task'

// --- Task Candidates ---
export async function fetchTaskCandidates(limit = 50): Promise<TaskCandidateListResponse> {
  const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
  return ApiClient.instance.get<TaskCandidateListResponse>(
    `/api/v1/task-candidates?${params.toString()}`,
    'Task candidates request failed'
  )
}

export async function reviewTaskCandidate(
  taskCandidateId: string,
  reviewState: TaskCandidateReviewState
): Promise<void> {
  return ApiClient.instance.put(
    `/api/v1/task-candidates/${encodeURIComponent(taskCandidateId)}/review`,
    {
      command_id: `task-candidate-review-${crypto.randomUUID()}`,
      review_state: reviewState
    }
  )
}

// --- Tasks ---
export async function fetchTaskRecords(
  params: { status?: string; project_id?: string; source_type?: string; limit?: number } = {}
): Promise<TaskRecordsResponse> {
  const sp = new URLSearchParams()
  if (params.status) sp.set('status', params.status)
  if (params.project_id) sp.set('project_id', params.project_id)
  if (params.source_type) sp.set('source_type', params.source_type)
  if (params.limit) sp.set('limit', String(params.limit))
  return ApiClient.instance.get<TaskRecordsResponse>(
    `/api/v1/tasks?${sp.toString()}`,
    'Tasks request failed'
  )
}

export async function updateTask(taskId: string, body: Record<string, unknown>): Promise<Task> {
  return ApiClient.instance.put<Task>(
    `/api/v1/tasks/${encodeURIComponent(taskId)}`,
    body,
    'Update task failed'
  )
}

export async function setTaskStatus(taskId: string, status: string): Promise<{ status: string }> {
  return ApiClient.instance.post<{ status: string }>(
    `/api/v1/tasks/${encodeURIComponent(taskId)}/status`,
    { status },
    'Set status failed'
  )
}

// --- Decisions ---
export async function fetchDecisions(params: {
  entityKind: DecisionEntityKind
  entityId: string
  limit?: number
}): Promise<DecisionListResponse> {
  const query = new URLSearchParams({
    entity_kind: params.entityKind,
    entity_id: params.entityId,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<DecisionListResponse>(
    `/api/v1/decisions?${query.toString()}`,
    'Decisions request failed'
  )
}

export async function fetchDecisionReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<DecisionListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<DecisionListResponse>(
    `/api/v1/decisions?${query.toString()}`,
    'Decision review items request failed'
  )
}

export async function reviewDecision(
  decisionId: string,
  request: DecisionReviewRequest
): Promise<Decision> {
  return ApiClient.instance.put<Decision>(
    `/api/v1/decisions/${encodeURIComponent(decisionId)}/review`,
    request,
    'Decision review request failed'
  )
}

// --- Obligations ---
export async function fetchObligations(params: {
  entityKind: ObligationEntityKind
  entityId: string
  limit?: number
}): Promise<ObligationListResponse> {
  const query = new URLSearchParams({
    entity_kind: params.entityKind,
    entity_id: params.entityId,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<ObligationListResponse>(
    `/api/v1/obligations?${query.toString()}`,
    'Obligations request failed'
  )
}

export async function fetchObligationReviewItems(params: {
  reviewState: string
  limit?: number
}): Promise<ObligationListResponse> {
  const query = new URLSearchParams({
    review_state: params.reviewState,
    limit: String(Math.trunc(params.limit ?? 50))
  })
  return ApiClient.instance.get<ObligationListResponse>(
    `/api/v1/obligations?${query.toString()}`,
    'Obligation review items request failed'
  )
}

export async function reviewObligation(
  obligationId: string,
  request: ObligationReviewRequest
): Promise<Obligation> {
  return ApiClient.instance.put<Obligation>(
    `/api/v1/obligations/${encodeURIComponent(obligationId)}/review`,
    request,
    'Obligation review request failed'
  )
}
```

### `frontend/src/domains/tasks/queries/useTasksQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/tasks/queries/useTasksQuery.ts`
- Size bytes / Размер в байтах: `627`
- Included characters / Включено символов: `627`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchTaskCandidates, fetchTaskRecords } from '../api/tasks'
import type { TaskCandidate, Task } from '../types/task'

export function useTaskCandidatesQuery() {
  return useQuery<TaskCandidate[]>({
    queryKey: ['task-candidates'],
    queryFn: async () => {
      const response = await fetchTaskCandidates(50)
      return response.items
    }
  })
}

export function useTasksQuery() {
  return useQuery<Task[]>({
    queryKey: ['tasks'],
    queryFn: async () => {
      const response = await fetchTaskRecords({ limit: 50 })
      return response.items
    }
  })
}
```

### `frontend/src/domains/tasks/stores/tasks.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/tasks/stores/tasks.ts`
- Size bytes / Размер в байтах: `3690`
- Included characters / Включено символов: `3687`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type {
  Decision,
  DecisionEntityKind,
  Obligation,
  TaskCandidate
} from '../types/task'

export const useTasksStore = defineStore('tasks-ui', () => {
  const tasksError = ref<string>('')
  const contextReviewError = ref<string>('')
  const isAiTaskRefreshSubmitting = ref<boolean>(false)
  const reviewEntityKind = ref<DecisionEntityKind>('project')
  const reviewEntityId = ref<string>('')
  const reviewingContextItemId = ref<string | null>(null)
  const decisions = ref<Decision[]>([])
  const obligations = ref<Obligation[]>([])
  const isContextReviewLoading = ref<boolean>(false)

  function setError(msg: string) {
    tasksError.value = msg
  }

  function clearError() {
    tasksError.value = ''
  }

  function setReviewEntityKind(kind: DecisionEntityKind) {
    reviewEntityKind.value = kind
  }

  function setReviewEntityId(entityId: string) {
    reviewEntityId.value = entityId
  }

  function setReviewingItemId(id: string | null) {
    reviewingContextItemId.value = id
  }

  function setDecisions(items: Decision[]) {
    decisions.value = items
  }

  function setObligations(items: Obligation[]) {
    obligations.value = items
  }

  function setContextReviewLoading(val: boolean) {
    isContextReviewLoading.value = val
  }

  function setContextReviewError(msg: string) {
    contextReviewError.value = msg
  }

  return {
    tasksError,
    contextReviewError,
    isAiTaskRefreshSubmitting,
    reviewEntityKind,
    reviewEntityId,
    reviewingContextItemId,
    decisions,
    obligations,
    isContextReviewLoading,
    setError,
    clearError,
    setReviewEntityKind,
    setReviewEntityId,
    setReviewingItemId,
    setDecisions,
    setObligations,
    setContextReviewLoading,
    setContextReviewError
  }
})

// Utility functions

export function taskSourceLabel(item: TaskCandidate | { source_kind: string; source_id: string }): string {
  const kind = item.source_kind
  return `${kind.charAt(0).toUpperCase()}${kind.slice(1)} · ${item.source_id}`
}

export function taskConfidence(item: TaskCandidate): string {
  return `${Math.round(item.confidence * 100)}%`
}

export function taskCreatedTime(value: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Unknown date'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

export function formatDecisionTime(value: string | null): string {
  if (!value) return 'No decision date'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Unknown date'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

export function formatEntityKind(kind: string): string {
  return kind
    .split('_')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

export function formatDecisionEntity(kind: string | null, entityId: string | null): string {
  if (!kind || !entityId) return 'No decider'
  return `${formatEntityKind(kind)} · ${entityId}`
}

export function formatObligationDueTime(value: string | null): string {
  if (!value) return 'No due date'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Unknown date'
  return new Intl.DateTimeFormat('en', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

export function formatObligationEntity(kind: string, entityId: string): string {
  return `${formatEntityKind(kind)} · ${entityId}`
}
```

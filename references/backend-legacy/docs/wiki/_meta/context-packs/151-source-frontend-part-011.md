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

- Chunk ID / ID чанка: `151-source-frontend-part-011`
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

### `frontend/src/domains/tasks/types/task.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/tasks/types/task.ts`
- Size bytes / Размер в байтах: `3350`
- Included characters / Включено символов: `3350`
- Truncated / Обрезано: `no`

```typescript
// --- Task Candidate types ---
export type TaskCandidateReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

export interface TaskCandidate {
  task_candidate_id: string
  source_kind: 'message' | 'document'
  source_id: string
  project_id: string | null
  title: string
  due_text: string | null
  assignee_label: string | null
  confidence: number
  review_state: TaskCandidateReviewState
  evidence_excerpt: string
  generated_at: string
  reviewed_at: string | null
  updated_at: string
}

export interface TaskCandidateListResponse {
  items: TaskCandidate[]
}

// --- Task types ---
export interface Task {
  task_id: string
  task_candidate_id: string | null
  title: string
  description: string | null
  source_kind: string
  source_id: string
  source_type: string
  project_id: string | null
  status: string
  makosh_status: string
  priority_score: number | null
  risk_score: number | null
  readiness_score: number | null
  area: string | null
  why: string | null
  outcome: string | null
  due_at: string | null
  completed_at: string | null
  archived_at: string | null
  waiting_reason: string | null
  energy_type: string | null
  confidentiality: string
  tags: unknown[]
  task_metadata: Record<string, unknown>
  linked_person_id: string | null
  linked_organization_id: string | null
  created_from_event_id: string | null
  created_by_actor_id: string | null
  created_at: string
  updated_at: string
}

export interface TaskRecordsResponse {
  items: Task[]
}

// --- Decision types ---
export type DecisionEntityKind =
  | 'persona' | 'organization' | 'project' | 'communication'
  | 'document' | 'task' | 'event' | 'decision' | 'obligation' | 'knowledge'

export type DecisionReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'

export interface Decision {
  decision_id: string
  title: string
  status: string
  rationale: string
  alternatives: unknown
  decided_by_entity_kind: DecisionEntityKind | null
  decided_by_entity_id: string | null
  decided_at: string | null
  review_state: DecisionReviewState
  confidence: number
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface DecisionListResponse {
  items: Decision[]
}

export interface DecisionReviewRequest {
  review_state: Exclude<DecisionReviewState, 'suggested'>
}

// --- Obligation types ---
export type ObligationEntityKind =
  | 'persona' | 'organization' | 'project' | 'communication'
  | 'document' | 'task' | 'event' | 'decision' | 'obligation' | 'knowledge'

export type ObligationReviewState = 'suggested' | 'user_confirmed' | 'user_rejected'
export type ObligationRiskState = 'none' | 'watch' | 'at_risk' | 'breached'

export interface Obligation {
  obligation_id: string
  obligated_entity_kind: ObligationEntityKind
  obligated_entity_id: string
  beneficiary_entity_kind: ObligationEntityKind | null
  beneficiary_entity_id: string | null
  statement: string
  status: string
  review_state: ObligationReviewState
  due_at: string | null
  condition: string | null
  risk_state: ObligationRiskState
  confidence: number
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface ObligationListResponse {
  items: Obligation[]
}

export interface ObligationReviewRequest {
  review_state: Exclude<ObligationReviewState, 'suggested'>
}
```

### `frontend/src/domains/timeline/api/timeline.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/timeline/api/timeline.ts`
- Size bytes / Размер в байтах: `496`
- Included characters / Включено символов: `496`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type { TimelineMessage } from '../types/timeline'

export async function fetchCommunicationMessages(limit = 500): Promise<TimelineMessage[]> {
	const params = new URLSearchParams({ limit: String(Math.trunc(limit)) })
	const response = await ApiClient.instance.get<{ items: TimelineMessage[] }>(
		`/api/v1/communications/messages?${params.toString()}`,
		'Communication messages request failed'
	)
	return response.items ?? []
}
```

### `frontend/src/domains/timeline/queries/useTimelineQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/timeline/queries/useTimelineQuery.ts`
- Size bytes / Размер в байтах: `321`
- Included characters / Включено символов: `321`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { fetchCommunicationMessages } from '../api/timeline'

export function useTimelineMessagesQuery() {
	return useQuery({
		queryKey: ['timeline-messages'],
		queryFn: () => fetchCommunicationMessages(500),
		refetchOnMount: 'always' as const,
		staleTime: 30_000
	})
}
```

### `frontend/src/domains/timeline/stores/timeline.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/timeline/stores/timeline.ts`
- Size bytes / Размер в байтах: `1138`
- Included characters / Включено символов: `1136`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TimelineMessage, TimelineFilters } from '../types/timeline'

export const useTimelineStore = defineStore('timeline-ui', () => {
	const messages = ref<TimelineMessage[]>([])
	const error = ref('')
	const isLoading = ref(false)
	const filters = ref<TimelineFilters>({
		Messages: true,
		Documents: true,
		Tasks: true,
		Calendar: true,
		Notes: true,
		Decisions: true
	})

	const filteredMessages = computed<TimelineMessage[]>(() => {
		// Filter is a placeholder — in the Svelte original, filter state exists
		// but all items pass through. Keep the structure for AC4 compliance.
		return messages.value
	})

	function setMessages(items: TimelineMessage[]) {
		messages.value = items
	}

	function setLoading(v: boolean) {
		isLoading.value = v
	}

	function setError(msg: string) {
		error.value = msg
	}

	function toggleFilter(kind: keyof TimelineFilters) {
		filters.value[kind] = !filters.value[kind]
	}

	return {
		messages,
		error,
		isLoading,
		filters,
		filteredMessages,
		setMessages,
		setLoading,
		setError,
		toggleFilter
	}
})
```

### `frontend/src/domains/timeline/types/timeline.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/timeline/types/timeline.ts`
- Size bytes / Размер в байтах: `479`
- Included characters / Включено символов: `479`
- Truncated / Обрезано: `no`

```typescript
export interface TimelineMessage {
	message_id: string
	sender_display_name: string | null
	sender: string
	subject: string
	body_text_preview: string
	occurred_at: string | null
	projected_at: string
	channel_kind: string
}

export type TimelineFilterKind = 'Messages' | 'Documents' | 'Tasks' | 'Calendar' | 'Notes' | 'Decisions'

export interface TimelineFilters {
	Messages: boolean
	Documents: boolean
	Tasks: boolean
	Calendar: boolean
	Notes: boolean
	Decisions: boolean
}
```

### `frontend/src/gen/makosh/common/v1/common_pb.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/gen/makosh/common/v1/common_pb.ts`
- Size bytes / Размер в байтах: `1825`
- Included characters / Включено символов: `1825`
- Truncated / Обрезано: `no`

```typescript
// @generated by protoc-gen-es v2.12.0 with parameter "target=ts"
// @generated from file makosh/common/v1/common.proto (package makosh.common.v1, syntax proto3)
/* eslint disabled in generated source */

import type { GenFile, GenMessage } from "@bufbuild/protobuf/codegenv2";
import { fileDesc, messageDesc } from "@bufbuild/protobuf/codegenv2";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file makosh/common/v1/common.proto.
 */
export const file_makosh_common_v1_common: GenFile = /*@__PURE__*/
  fileDesc("Ch1oZXJtZXMvY29tbW9uL3YxL2NvbW1vbi5wcm90bxIQaGVybWVzLmNvbW1vbi52MSIsCgtQYWdlUmVxdWVzdBINCgVsaW1pdBgBIAEoDRIOCgZjdXJzb3IYAiABKAkiNQoMUGFnZVJlc3BvbnNlEhMKC25leHRfY3Vyc29yGAEgASgJEhAKCGhhc19tb3JlGAIgASgIYgZwcm90bzM");

/**
 * @generated from message makosh.common.v1.PageRequest
 */
export type PageRequest = Message<"makosh.common.v1.PageRequest"> & {
  /**
   * @generated from field: uint32 limit = 1;
   */
  limit: number;

  /**
   * @generated from field: string cursor = 2;
   */
  cursor: string;
};

/**
 * Describes the message makosh.common.v1.PageRequest.
 * Use `create(PageRequestSchema)` to create a new message.
 */
export const PageRequestSchema: GenMessage<PageRequest> = /*@__PURE__*/
  messageDesc(file_makosh_common_v1_common, 0);

/**
 * @generated from message makosh.common.v1.PageResponse
 */
export type PageResponse = Message<"makosh.common.v1.PageResponse"> & {
  /**
   * @generated from field: string next_cursor = 1;
   */
  nextCursor: string;

  /**
   * @generated from field: bool has_more = 2;
   */
  hasMore: boolean;
};

/**
 * Describes the message makosh.common.v1.PageResponse.
 * Use `create(PageResponseSchema)` to create a new message.
 */
export const PageResponseSchema: GenMessage<PageResponse> = /*@__PURE__*/
  messageDesc(file_makosh_common_v1_common, 1);
```

### `frontend/src/gen/makosh/communications/v1/communications_pb.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/gen/makosh/communications/v1/communications_pb.ts`
- Size bytes / Размер в байтах: `207436`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
// @generated by protoc-gen-es v2.12.0 with parameter "target=ts"
// @generated from file makosh/communications/v1/communications.proto (package makosh.communications.v1, syntax proto3)
/* eslint disabled in generated source */

import type { GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv2";
import { fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv2";
import type { PageRequest, PageResponse } from "../../common/v1/common_pb";
import { file_makosh_common_v1_common } from "../../common/v1/common_pb";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file makosh/communications/v1/communications.proto.
 */
export const file_makosh_communications_v1_communications: GenFile = /*@__PURE__*/
  fileDesc("Ci1oZXJtZXMvY29tbXVuaWNhdGlvbnMvdjEvY29tbXVuaWNhdGlvbnMucHJvdG8SGGhlcm1lcy5jb21tdW5pY2F0aW9ucy52MSKTBAoeQ29tbXVuaWNhdGlvbk1lc3NhZ2VBdHRhY2htZW50EhUKDWF0dGFjaG1lbnRfaWQYASABKAkSEgoKbWVzc2FnZV9pZBgCIAEoCRIVCg1yYXdfcmVjb3JkX2lkGAMgASgJEg8KB2Jsb2JfaWQYBCABKAkSHgoWcHJvdmlkZXJfYXR0YWNobWVudF9pZBgFIAEoCRIVCghmaWxlbmFtZRgGIAEoCUgAiAEBEhQKDGNvbnRlbnRfdHlwZRgHIAEoCRISCgpzaXplX2J5dGVzGAggASgDEg4KBnNoYTI1NhgJIAEoCRITCgtkaXNwb3NpdGlvbhgKIAEoCRITCgtzY2FuX3N0YXR1cxgLIAEoCRIYCgtzY2FuX2VuZ2luZRgMIAEoCUgBiAEBEhwKD3NjYW5fY2hlY2tlZF9hdBgNIAEoCUgCiAEBEhkKDHNjYW5fc3VtbWFyeRgOIAEoCUgDiAEBEhoKEnNjYW5fbWV0YWRhdGFfanNvbhgPIAEoCRIUCgxzdG9yYWdlX2tpbmQYECABKAkSFAoMc3RvcmFnZV9wYXRoGBEgASgJEhIKCmNyZWF0ZWRfYXQYEiABKAkSEgoKdXBkYXRlZF9hdBgTIAEoCUILCglfZmlsZW5hbWVCDgoMX3NjYW5fZW5naW5lQhIKEF9zY2FuX2NoZWNrZWRfYXRCDwoNX3NjYW5fc3VtbWFyeSLRBgoUQ29tbXVuaWNhdGlvbk1lc3NhZ2USEgoKbWVzc2FnZV9pZBgBIAEoCRIVCg1yYXdfcmVjb3JkX2lkGAIgASgJEhYKDm9ic2VydmF0aW9uX2lkGAMgASgJEhIKCmFjY291bnRfaWQYBCABKAkSGgoScHJvdmlkZXJfcmVjb3JkX2lkGAUgASgJEg8KB3N1YmplY3QYBiABKAkSDgoGc2VuZGVyGAcgASgJEhIKCnJlY2lwaWVudHMYCCADKAkSEQoJYm9keV90ZXh0GAkgASgJEhgKC29jY3VycmVkX2F0GAogASgJSACIAQESFAoMcHJvamVjdGVkX2F0GAsgASgJEhQKDGNoYW5uZWxfa2luZBgMIAEoCRIcCg9jb252ZXJzYXRpb25faWQYDSABKAlIAYgBARIgChNzZW5kZXJfZGlzcGxheV9uYW1lGA4gASgJSAKIAQESFgoOZGVsaXZlcnlfc3RhdGUYDyABKAkSHQoVbWVzc2FnZV9tZXRhZGF0YV9qc29uGBAgASgJEhYKDndvcmtmbG93X3N0YXRlGBEgASgJEh0KEGltcG9ydGFuY2Vfc2NvcmUYEiABKAVIA4gBARIYCgthaV9jYXRlZ29yeRgTIAEoCUgEiAEBEhcKCmFpX3N1bW1hcnkYFCABKAlIBYgBARIkChdhaV9zdW1tYXJ5X2dlbmVyYXRlZF9hdBgVIAEoCUgGiAEBEhMKC2xvY2FsX3N0YXRlGBYgASgJEiMKFmxvY2FsX3N0YXRlX2NoYW5nZWRfYXQYFyABKAlIB4gBARIfChJsb2NhbF9zdGF0ZV9yZWFzb24YGCABKAlICIgBARIYChBhdHRhY2htZW50X2NvdW50GBkgASgDQg4KDF9vY2N1cnJlZF9hdEISChBfY29udmVyc2F0aW9uX2lkQhYKFF9zZW5kZXJfZGlzcGxheV9uYW1lQhMKEV9pbXBvcnRhbmNlX3Njb3JlQg4KDF9haV9jYXRlZ29yeUINCgtfYWlfc3VtbWFyeUIaChhfYWlfc3VtbWFyeV9nZW5lcmF0ZWRfYXRCGQoXX2xvY2FsX3N0YXRlX2NoYW5nZWRfYXRCFQoTX2xvY2FsX3N0YXRlX3JlYXNvbiLqAgoTTGlzdE1lc3NhZ2VzUmVxdWVzdBIXCgphY2NvdW50X2lkGAEgASgJSACIAQESGwoOd29ya2Zsb3dfc3RhdGUYAiABKAlIAYgBARIZCgxjaGFubmVsX2tpbmQYAyABKAlIAogBARIcCg9jb252ZXJzYXRpb25faWQYBCABKAlIA4gBARISCgVxdWVyeRgFIAEoCUgEiAEBEhcKCm1hdGNoX21vZGUYBiABKAlIBYgBARIYCgtsb2NhbF9zdGF0ZRgHIAEoCUgGiAEBEhMKBmN1cnNvchgIIAEoCUgHiAEBEg0KBWxpbWl0GAkgASgNQg0KC19hY2NvdW50X2lkQhEKD193b3JrZmxvd19zdGF0ZUIPCg1fY2hhbm5lbF9raW5kQhIKEF9jb252ZXJzYXRpb25faWRCCAoGX3F1ZXJ5Qg0KC19tYXRjaF9tb2RlQg4KDF9sb2NhbF9zdGF0ZUIJCgdfY3Vyc29yIpEBChRMaXN0TWVzc2FnZXNSZXNwb25zZRI9CgVpdGVtcxgBIAMoCzIuLmhlcm1lcy5jb21tdW5pY2F0aW9ucy52MS5Db21tdW5pY2F0aW9uTWVzc2FnZRIYCgtuZXh0X2N1cnNvchgCIAEoCUgAiAEBEhAKCGhhc19tb3JlGAMgASgIQg4KDF9uZXh0X2N1cnNvciInChFHZXRNZXNzYWdlUmVxdWVzdBISCgptZXNzYWdlX2lkGAEgASgJIqEBChJHZXRNZXNzYWdlUmVzcG9uc2USPAoEaXRlbRgBIAEoCzIuLmhlcm1lcy5jb21tdW5pY2F0aW9ucy52MS5Db21tdW5pY2F0aW9uTWVzc2FnZRJNCgthdHRhY2htZW50cxgCIAMoCzI4Lmhlcm1lcy5jb21tdW5pY2F0aW9ucy52MS5Db21tdW5pY2F0aW9uTWVzc2FnZUF0dGFjaG1lbnQiUwolVHJhbnNpdGlvbk1lc3NhZ2VXb3JrZmxvd1N0YXRlUmVxdWVzdBISCgptZXNzYWdlX2lkGAEgASgJEhYKDndvcmtmbG93X3N0YXRlGAIgASgJImwKJlRyYW5zaXRpb25NZXNzYWdlV29ya2Zsb3dTdGF0ZVJlc3BvbnNlEhIKCm1lc3NhZ2VfaWQYASABKAkSFgoOd29ya2Zsb3dfc3RhdGUYAiABKAkSFgoOcHJldmlvdXNfc3RhdGUYAyABKAkiNAoeVXBkYXRlTWVzc2FnZUxvY2FsU3RhdGVSZXF1ZXN0EhIKCm1lc3NhZ2VfaWQYASABKAkifgofVXBkYXRlTWVzc2FnZUxvY2FsU3RhdGVSZXNwb25zZRISCgptZXNzYWdlX2lkGAEgASgJEhMKC2xvY2FsX3N0YXRlGAIgASgJEh0KEHByb3ZpZGVyX2RlbGV0ZWQYAyABKAhIAIgBAUITChFfcHJvdmlkZXJfZGVsZXRlZCIsChZNYXJrTWVzc2FnZVJlYWRSZXF1ZXN0EhIKCm1lc3NhZ2VfaWQYASABKAkiWgoXTWFya01lc3NhZ2VSZWFkUmVzcG9uc2USEgoKbWVzc2FnZV9pZBgBIAEoCRITCgttYXJrZWRfcmVhZBgCIAEoCBIWCg53b3JrZmxvd19zdGF0ZRgDIAEoCSI2CiBEZWxldGVNZXNzYWdlRnJvbVByb3ZpZGVyUmVxdWVzdBISCgptZXNzYWdlX2lkGAEgASgJIpEBCiFEZWxldGVNZXNzYWdlRnJvbVByb3ZpZGVyUmVzcG9uc2USEgoKbWVzc2FnZV9pZBgBIAEoCRIPCgdkZWxldGVkGAIgASgIEhMKC2xvY2FsX3N0YXRlGAMgASgJEh0KEHByb3ZpZGVyX2RlbGV0ZWQYBCABKAhIAIgBAUITChFfcHJvdmlkZXJfZGVsZXRlZCKJAQoYQnVsa01lc3NhZ2VBY3Rpb25SZXF1ZXN0Eg4KBmFjdGlvbhgBIAEoCRITCgttZXNzYWdlX2lkcxgCIAMoCRISCgVsYWJlbBgDIAEoCUgAiAEBEhkKDHNub296ZV91bnRpbBgEIAEoCUgBiAEBQggKBl9sYWJlbEIPCg1fc25vb3plX3VudGlsIoUBChlCdWxrTWVzc2FnZUFjdGlvblJlc3BvbnNlEg4KBmFjdGlvbhgBIAEoCRIXCg9yZXF1ZXN0ZWRfY291bnQYAiABKA0SFQoNbWF0Y2hlZF9jb3VudBgDIAEoDRIVCg11cGRhdGVkX2NvdW50GAQgASgNEhEKCW5vdF9mb3VuZBgFIAMoCSIqChRNZXNzYWdlVG9nZ2xlUmVxdWVzdBISCgptZXNzYWdlX2lkGAEgASgJIj4KGFRvZ2dsZU1lc3NhZ2VQaW5SZXNwb25zZRISCgptZXNzYWdlX2lkGAEgASgJEg4KBnBpbm5lZBgCIAEoCCJHCh5Ub2dnbGVNZXNzYWdlSW1wb3J0YW50UmVzcG9uc2USEgoKbWVzc2FnZV9pZBgBIAEoCRIRCglpbXBvcnRhbnQYAiABKAgiPgoZVG9nZ2xlTWVzc2FnZU11dGVSZXNwb25zZRISCgptZXNzYWdlX2lkGAEgASgJEg0KBW11dGVkGAIgASgIIjkKFFNub296ZU1lc3NhZ2VSZXF1ZXN0EhIKCm1lc3NhZ2VfaWQYASABKAkSDQoFdW50aWwYAiABKAkiKAoVU25vb3plTWVzc2FnZVJlc3BvbnNlEg8KB3Nub296ZWQYASABKAgiPgoZVXBkYXRlTWVzc2FnZUxhYmVsUmVxdWVzdBISCgptZXNzYWdlX2lkGAEgASgJEg0KBWxhYmVsGAIgASgJIioKF0FkZE1lc3NhZ2VMYWJlbFJlc3BvbnNlEg8KB2xhYmVsZWQYASABKAgiLQoaUmVtb3ZlTWVzc2FnZUxhYmVsUmVzcG9uc2USDwoHcmVtb3ZlZBgBIAEoCCI8ChlNZXNzYWdlS25vd2xlZGdlQ2FuZGlkYXRlEg0KBXRpdGxlGAEgASgJEhAKCGV2aWRlbmNlGAIgASgJIv8DChZNZXNzYWdlU3VtbWFyeUNvbnRyYWN0EhIKCmtleV9wb2ludHMYASADKAkSFAoMYWN0aW9uX2l0ZW1zGAIgAygJEg0KBXJpc2tzGAMgAygJEhEKCWRlYWRsaW5lcxgEIAMoCRJNChBldmVudF9jYW5kaWRhdGVzGAUgAygLMjMuaGVybWVzLmNvbW11bmljYXRpb25zLnYxLk1lc3NhZ2VLbm93bGVkZ2VDYW5kaWRhdGUSTwoScGVyc29uYV9jYW5kaWRhdGVzGAYgAygLMjMuaGVybWVzLmNvbW11bmljYXRpb25zLnYxLk1lc3NhZ2VLbm93bGVkZ2VDYW5kaWRhdGUSVAoXb3JnYW5pemF0aW9uX2NhbmRpZGF0ZXMYByADKAsyMy5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuTWVzc2FnZUtub3dsZWRnZUNhbmRpZGF0ZRJQChNkb2N1bWVudF9jYW5kaWRhdGVzGAggAygLMjMuaGVybWVzLmNvbW11bmljYXRpb25zLnYxLk1lc3NhZ2VLbm93bGVkZ2VDYW5kaWRhdGUSUQoUYWdyZWVtZW50X2NhbmRpZGF0ZXMYCSADKAsyMy5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuTWVzc2FnZUtub3dsZWRnZUNhbmRpZGF0ZSIrChVBbmFseXplTWVzc2FnZVJlcXVlc3QSEgoKbWVzc2FnZV9pZBgBIAEoCSLmAgoWQW5hbHl6ZU1lc3NhZ2VSZXNwb25zZRISCgptZXNzYWdlX2lkGAEgASgJEhAKCGFuYWx5emVkGAIgASgIEhUKCGNhdGVnb3J5GAMgASgJSACIAQESFAoHc3VtbWFyeRgEIAEoCUgBiAEBEkoKEHN1bW1hcnlfY29udHJhY3QYBSABKAsyMC5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuTWVzc2FnZVN1bW1hcnlDb250cmFjdBIdChBpbXBvcnRhbmNlX3Njb3JlGAYgASgFSAKIAQESFgoOd29ya2Zsb3dfc3RhdGUYByABKAkSDgoGc291cmNlGAggASgJEhcKCmNvbmZpZGVuY2UYCSABKAFIA4gBARIQCghldmlkZW5jZRgKIAMoCUILCglfY2F0ZWdvcnlCCgoIX3N1bW1hcnlCEwoRX2ltcG9ydGFuY2Vfc2NvcmVCDQoLX2NvbmZpZGVuY2UiMAoUV29ya2Zsb3dBY3Rpb25Tb3VyY2USDAoEa2luZBgBIAEoCRIKCgJpZBgCIAEoCSKrAgoTV29ya2Zsb3dBY3Rpb25JbnB1dBISCgV0aXRsZRgBIAEoCUgAiAEBEhEKBGJvZHkYAiABKAlIAYgBARISCgVlbWFpbBgDIAEoCUgCiAEBEhkKDGRpc3BsYXlfbmFtZRgEIAEoCUgDiAEBEhYKCXN0YXJ0c19hdBgFIAEoCUgEiAEBEhQKB2VuZHNfYXQYBiABKAlIBYgBARITCgZkdWVfYXQYByABKAlIBogBARIYCgtkb2N1bWVudF9pZBgIIAEoCUgHiAEBQggKBl90aXRsZUIHCgVfYm9keUIICgZfZW1haWxCDwoNX2Rpc3BsYXlfbmFtZUIMCgpfc3RhcnRzX2F0QgoKCF9lbmRzX2F0QgkKB19kdWVfYXRCDgoMX2RvY3VtZW50X2lkItgBChVXb3JrZmxvd0FjdGlvblJlcXVlc3QSEgoKY29tbWFuZF9pZBgBIAEoCRIOCgZhY3Rpb24YAiABKAkSQwoGc291cmNlGAMgASgLMi4uaGVybWVzLmNvbW11bmljYXRpb25zLnYxLldvcmtmbG93QWN0aW9uU291cmNlSACIAQESQQoFaW5wdXQYBCABKAsyLS5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuV29ya2Zsb3dBY3Rpb25JbnB1dEgBiAEBQgkKB19zb3VyY2VCCAoGX2lucHV0IjwKFFdvcmtmbG93QWN0aW9uVGFyZ2V0EgwKBGtpbmQYASABKAkSDwoCaWQYAiABKAlIAIgBAUIFCgNfaWQipAEKGFdvcmtmbG93QWN0aW9uUHJvdmVuYW5jZRIYCgtzb3VyY2Vfa2luZBgBIAEoCUgAiAEBEhYKCXNvdXJjZV9pZBgCIAEoCUgBiAEBEhcKCmNvbmZpZGVuY2UYAyABKAFIAogBARIQCghldmlkZW5jZRgEIAMoCUIOCgxfc291cmNlX2tpbmRCDAoKX3NvdXJjZV9pZEINCgtfY29uZmlkZW5jZSLmAQoWV29ya2Zsb3dBY3Rpb25SZXNwb25zZRISCgpjb21tYW5kX2lkGAEgASgJEhAKCGV2ZW50X2lkGAIgASgJEg4KBmFjdGlvbhgDIAEoCRIOCgZzdGF0dXMYBCABKAkSPgoGdGFyZ2V0GAUgASgLMi4uaGVybWVzLmNvbW11bmljYXRpb25zLnYxLldvcmtmbG93QWN0aW9uVGFyZ2V0EkYKCnByb3ZlbmFuY2UYBiABKAsyMi5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuV29ya2Zsb3dBY3Rpb25Qcm92ZW5hbmNlIisKFUV4cGxhaW5NZXNzYWdlUmVxdWVzdBISCgptZXNzYWdlX2lkGAEgASgJIikKFkV4cGxhaW5NZXNzYWdlUmVzcG9uc2USDwoHcmVhc29ucxgBIAMoCSIuChhHZXRNZXNzYWdlU21hcnRDY1JlcXVlc3QSEgoKbWVzc2FnZV9pZBgBIAEoCSIwChlHZXRNZXNzYWdlU21hcnRDY1Jlc3BvbnNlEhMKC3N1Z2dlc3Rpb25zGAEgAygJIj0KF0dldE1lc3NhZ2VFeHBvcnRSZXF1ZXN0EhIKCm1lc3NhZ2VfaWQYASABKAkSDgoGZm9ybWF0GAIgASgJIlMKGEdldE1lc3NhZ2VFeHBvcnRSZXNwb25zZRIUCgxjb250ZW50X3R5cGUYASABKAkSDwoHY29udGVudBgCIAEoCRIQCghmaWxlbmFtZRgDIAEoCSKfAQoRTWVzc2FnZUF1dGhSZXN1bHQSDgoGcmVzdWx0GAEgASgJEhMKBmRvbWFpbhgCIAEoCUgAiAEBEg8KAmlwGAMgASgJSAGIAQESFQoIc2VsZWN0b3IYBCABKAlIAogBARITCgZwb2xpY3kYBSABKAlIA4gBAUIJCgdfZG9tYWluQgUKA19pcEILCglfc2VsZWN0b3JCCQoHX3BvbGljeSKDAgoRTWVzc2FnZUF1dGhSZXBvcnQSPQoDc3BmGAEgASgLMisuaGVybWVzLmNvbW11bmljYXRpb25zLnYxLk1lc3NhZ2VBdXRoUmVzdWx0SACIAQESPgoEZGtpbRgCIAEoCzIrLmhlcm1lcy5jb21tdW5pY2F0aW9ucy52MS5NZXNzYWdlQXV0aFJlc3VsdEgBiAEBEj8KBWRtYXJjGAMgASgLMisuaGVybWVzLmNvbW11bmljYXRpb25zLnYxLk1lc3NhZ2VBdXRoUmVzdWx0SAKIAQESEwoLcmF3X2hlYWRlcnMYBCADKAlCBgoEX3NwZkIHCgVfZGtpbUIICgZfZG1hcmMisAEKFU1lc3NhZ2VBdXRoUmlza1JlcG9ydBIPCgdoYXNfc3BmGAEgASgIEhAKCHNwZl9wYXNzGAIgASgIEhAKCGhhc19ka2ltGAMgASgIEhEKCWRraW1fcGFzcxgEIAEoCBIRCgloYXNfZG1hcmMYBSABKAgSEgoKZG1hcmNfcGFzcxgGIAEoCBISCgppc19zcG9vZmVkGAcgASgIEhQKDHJpc2tfc3VtbWFyeRgIIAEoCSIrChVHZXRNZXNzYWdlQXV0aFJlcXVlc3QSEgoKbWVzc2FnZV9pZBgBIAEoCSKSAQoWR2V0TWVzc2FnZUF1dGhSZXNwb25zZRI5CgRhdXRoGAEgASgLMisuaGVybWVzLmNvbW11bmljYXRpb25zLnYxLk1lc3NhZ2VBdXRoUmVwb3J0Ej0KBHJpc2sYAiABKAsyLy5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuTWVzc2FnZUF1dGhSaXNrUmVwb3J0IjAKGkdldE1lc3NhZ2VTaWduYXR1cmVSZXF1ZXN0EhIKCm1lc3NhZ2VfaWQYASABKAki7AEKG0dldE1lc3NhZ2VTaWduYXR1cmVSZXNwb25zZRIVCg1oYXNfc2lnbmF0dXJlGAEgASgIEhsKDnNpZ25hdHVyZV90eXBlGAIgASgJSACIAQESGAoLc2lnbmVyX2luZm8YAyABKAlIAYgBARIVCghpc192YWxpZBgEIAEoCEgCiAEBEiAKE2NlcnRfZXhwaXJ5X3dhcm5pbmcYBSABKAlIA4gBAUIRCg9fc2lnbmF0dXJlX3R5cGVCDgoMX3NpZ25lcl9pbmZvQgsKCV9pc192YWxpZEIWChRfY2VydF9leHBpcnlfd2FybmluZyKGAQoOQWlSZXBseVJlcXVlc3QSEgoKbWVzc2FnZV9pZBgBIAEoCRIRCgR0b25lGAIgASgJSACIAQESFQoIbGFuZ3VhZ2UYAyABKAlIAYgBARIUCgdjb250ZXh0GAQgASgJSAKIAQFCBwoFX3RvbmVCCwoJX2xhbmd1YWdlQgoKCF9jb250ZXh0ItUBCg9BaVJlcGx5UmVzcG9uc2USFAoHc3ViamVjdBgBIAEoCUgAiAEBEhEKBGJvZHkYAiABKAlIAYgBARIRCgR0b25lGAMgASgJSAKIAQESFQoIbGFuZ3VhZ2UYBCABKAlIA4gBARIWCglnZW5lcmF0ZWQYBSABKAhIBIgBARITCgZyZWFzb24YBiABKAlIBYgBAUIKCghfc3ViamVjdEIHCgVfYm9keUIHCgVfdG9uZUILCglfbGFuZ3VhZ2VCDAoKX2dlbmVyYXRlZEIJCgdfcmVhc29uIk4KFkFpUmVwbHlWYXJpYW50c1JlcXVlc3QSEgoKbWVzc2FnZV9pZBgBIAEoCRIRCglsYW5ndWFnZXMYAiADKAkSDQoFdG9uZXMYAyADKAkiVgoXQWlSZXBseVZhcmlhbnRzUmVzcG9uc2USOwoIdmFyaWFudHMYASADKAsyKS5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuQWlSZXBseVJlc3BvbnNlIjIKEldvcmtmbG93U3RhdGVDb3VudBINCgVzdGF0ZRgBIAEoCRINCgVjb3VudBgCIAEoAyJ5CiVMaXN0TWVzc2FnZVdvcmtmbG93U3RhdGVDb3VudHNSZXF1ZXN0EhcKCmFjY291bnRfaWQYASABKAlIAIgBARIYCgtsb2NhbF9zdGF0ZRgCIAEoCUgBiAEBQg0KC19hY2NvdW50X2lkQg4KDF9sb2NhbF9zdGF0ZSJmCiZMaXN0TWVzc2FnZVdvcmtmbG93U3RhdGVDb3VudHNSZXNwb25zZRI8CgZjb3VudHMYASADKAsyLC5oZXJtZXMuY29tbXVuaWNhdGlvbnMudjEuV29ya2Zsb3dTdGF0ZUNvdW50IpIBChJTdWJzY3JpcHRpb25Tb3VyY2USDgoGc2VuZGVyGAEgASgJEhUKDW1lc3NhZ2VfY291bnQYAiABKAMSEgoKZmlyc3Rfc2VlbhgDIAEoCRIRCglsYXN0X3NlZW4YBCABKAkSFQoNaXNfbmV3c2xldHRlchgFIAEoCBIXCg9oYXNfdW5zdWJzY3JpYmUYBiABKAgicQoYTGlzdFN1YnNjcmlwdGlvbnNSZXF1ZXN0EhcKCmFjY291bnRfaWQYASABKAlIAIgBARITCgZjdXJzb3IYAiABKAlIAYgBARINCgVsaW1pdBgDIAEoDUINCgtfYWNjb3VudF9pZEIJCgdfY3Vyc29yIpQBChlMaXN0U3Vic2NyaXB0aW9uc1Jlc3BvbnNlEjsKBWl0ZW1zGAEgAygLMiwuaGVybWVzLmNvbW11bmljYXRpb25zLnYxLlN1YnNjcmlwdGlvblNvdXJjZRIYCgtuZXh0X2N1cnNvchgCIAEoCUgAiAEBEhAKCGhhc19tb3JlGAMgASgIQg4KDF9uZXh0X2N1cnNvciKPAgoNTWFpbGJveEhlYWx0aBIWCg50b3RhbF9tZXNzYWdlcxgBIAEoAxIOCgZ1bnJlYWQYAiABKAMSFAoMbmVlZHNfYWN0aW9uGAMgASgDEg8KB3dhaXRpbmcYBCABKAMSDAoEZG9uZRgFIAEoAxIQCg
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/gen/makosh/events/v1/event_envelope_pb.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/gen/makosh/events/v1/event_envelope_pb.ts`
- Size bytes / Размер в байтах: `3103`
- Included characters / Включено символов: `3103`
- Truncated / Обрезано: `no`

```typescript
// @generated by protoc-gen-es v2.12.0 with parameter "target=ts"
// @generated from file makosh/events/v1/event_envelope.proto (package makosh.events.v1, syntax proto3)
/* eslint disabled in generated source */

import type { GenFile, GenMessage } from "@bufbuild/protobuf/codegenv2";
import { fileDesc, messageDesc } from "@bufbuild/protobuf/codegenv2";
import type { Timestamp } from "@bufbuild/protobuf/wkt";
import { file_google_protobuf_struct, file_google_protobuf_timestamp } from "@bufbuild/protobuf/wkt";
import type { JsonObject, Message } from "@bufbuild/protobuf";

/**
 * Describes the file makosh/events/v1/event_envelope.proto.
 */
export const file_makosh_events_v1_event_envelope: GenFile = /*@__PURE__*/
  fileDesc("CiVoZXJtZXMvZXZlbnRzL3YxL2V2ZW50X2VudmVsb3BlLnByb3RvEhBoZXJtZXMuZXZlbnRzLnYxIq8DCg1FdmVudEVudmVsb3BlEhAKCGV2ZW50X2lkGAEgASgJEhIKCmV2ZW50X3R5cGUYAiABKAkSFgoOc2NoZW1hX3ZlcnNpb24YAyABKAUSLwoLb2NjdXJyZWRfYXQYBCABKAsyGi5nb29nbGUucHJvdG9idWYuVGltZXN0YW1wEi8KC3JlY29yZGVkX2F0GAUgASgLMhouZ29vZ2xlLnByb3RvYnVmLlRpbWVzdGFtcBInCgZzb3VyY2UYBiABKAsyFy5nb29nbGUucHJvdG9idWYuU3RydWN0EiYKBWFjdG9yGAcgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdBIoCgdzdWJqZWN0GAggASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdBIoCgdwYXlsb2FkGAkgASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdBIrCgpwcm92ZW5hbmNlGAogASgLMhcuZ29vZ2xlLnByb3RvYnVmLlN0cnVjdBIUCgxjYXVzYXRpb25faWQYCyABKAkSFgoOY29ycmVsYXRpb25faWQYDCABKAliBnByb3RvMw", [file_google_protobuf_struct, file_google_protobuf_timestamp]);

/**
 * @generated from message makosh.events.v1.EventEnvelope
 */
export type EventEnvelope = Message<"makosh.events.v1.EventEnvelope"> & {
  /**
   * @generated from field: string event_id = 1;
   */
  eventId: string;

  /**
   * @generated from field: string event_type = 2;
   */
  eventType: string;

  /**
   * @generated from field: int32 schema_version = 3;
   */
  schemaVersion: number;

  /**
   * @generated from field: google.protobuf.Timestamp occurred_at = 4;
   */
  occurredAt?: Timestamp | undefined;

  /**
   * @generated from field: google.protobuf.Timestamp recorded_at = 5;
   */
  recordedAt?: Timestamp | undefined;

  /**
   * @generated from field: google.protobuf.Struct source = 6;
   */
  source?: JsonObject | undefined;

  /**
   * @generated from field: google.protobuf.Struct actor = 7;
   */
  actor?: JsonObject | undefined;

  /**
   * @generated from field: google.protobuf.Struct subject = 8;
   */
  subject?: JsonObject | undefined;

  /**
   * @generated from field: google.protobuf.Struct payload = 9;
   */
  payload?: JsonObject | undefined;

  /**
   * @generated from field: google.protobuf.Struct provenance = 10;
   */
  provenance?: JsonObject | undefined;

  /**
   * @generated from field: string causation_id = 11;
   */
  causationId: string;

  /**
   * @generated from field: string correlation_id = 12;
   */
  correlationId: string;
};

/**
 * Describes the message makosh.events.v1.EventEnvelope.
 * Use `create(EventEnvelopeSchema)` to create a new message.
 */
export const EventEnvelopeSchema: GenMessage<EventEnvelope> = /*@__PURE__*/
  messageDesc(file_makosh_events_v1_event_envelope, 0);
```

### `frontend/src/gen/makosh/signal_hub/v1/signal_hub_pb.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/gen/makosh/signal_hub/v1/signal_hub_pb.ts`
- Size bytes / Размер в байтах: `82176`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
// @generated by protoc-gen-es v2.12.0 with parameter "target=ts"
// @generated from file makosh/signal_hub/v1/signal_hub.proto (package makosh.signal_hub.v1, syntax proto3)
/* eslint disabled in generated source */

import type { GenFile, GenMessage, GenService } from "@bufbuild/protobuf/codegenv2";
import { fileDesc, messageDesc, serviceDesc } from "@bufbuild/protobuf/codegenv2";
import type { Message } from "@bufbuild/protobuf";

/**
 * Describes the file makosh/signal_hub/v1/signal_hub.proto.
 */
export const file_makosh_signal_hub_v1_signal_hub: GenFile = /*@__PURE__*/
  fileDesc("CiVoZXJtZXMvc2lnbmFsX2h1Yi92MS9zaWduYWxfaHViLnByb3RvEhRoZXJtZXMuc2lnbmFsX2h1Yi52MSLJAgoMU2lnbmFsU291cmNlEgoKAmlkGAEgASgJEgwKBGNvZGUYAiABKAkSFAoMZGlzcGxheV9uYW1lGAMgASgJEhAKCGNhdGVnb3J5GAQgASgJEhMKC3NvdXJjZV9raW5kGAUgASgJEhcKD2RlZmF1bHRfZW5hYmxlZBgGIAEoCBIcChRzdXBwb3J0c19jb25uZWN0aW9ucxgHIAEoCBIYChBzdXBwb3J0c19ydW50aW1lGAggASgIEhcKD3N1cHBvcnRzX3JlcGxheRgJIAEoCBIWCg5zdXBwb3J0c19wYXVzZRgKIAEoCBIVCg1zdXBwb3J0c19tdXRlGAsgASgIEiEKGWNhcGFiaWxpdHlfc2NoZW1hX3ZlcnNpb24YDCABKAUSEgoKY3JlYXRlZF9hdBgNIAEoCRISCgp1cGRhdGVkX2F0GA4gASgJIhQKEkxpc3RTb3VyY2VzUmVxdWVzdCJIChNMaXN0U291cmNlc1Jlc3BvbnNlEjEKBWl0ZW1zGAEgAygLMiIuaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsU291cmNlIiAKEEdldFNvdXJjZVJlcXVlc3QSDAoEY29kZRgBIAEoCSJFChFHZXRTb3VyY2VSZXNwb25zZRIwCgRpdGVtGAEgASgLMiIuaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsU291cmNlIu0BChBTaWduYWxDYXBhYmlsaXR5EgoKAmlkGAEgASgJEhMKC3NvdXJjZV9jb2RlGAIgASgJEhoKDWNvbm5lY3Rpb25faWQYAyABKAlIAIgBARISCgpjYXBhYmlsaXR5GAQgASgJEg0KBXN0YXRlGAUgASgJEhMKBnJlYXNvbhgGIAEoCUgBiAEBEh0KFXJlcXVpcmVzX2NvbmZpcm1hdGlvbhgHIAEoCBIUCgxhY3Rpb25fY2xhc3MYCCABKAkSEgoKdXBkYXRlZF9hdBgJIAEoCUIQCg5fY29ubmVjdGlvbl9pZEIJCgdfcmVhc29uInEKF0xpc3RDYXBhYmlsaXRpZXNSZXF1ZXN0EhgKC3NvdXJjZV9jb2RlGAEgASgJSACIAQESGgoNY29ubmVjdGlvbl9pZBgCIAEoCUgBiAEBQg4KDF9zb3VyY2VfY29kZUIQCg5fY29ubmVjdGlvbl9pZCJRChhMaXN0Q2FwYWJpbGl0aWVzUmVzcG9uc2USNQoFaXRlbXMYASADKAsyJi5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxDYXBhYmlsaXR5IqgBChNTaWduYWxGaXh0dXJlU291cmNlEhIKCmZpeHR1cmVfaWQYASABKAkSEwoLc291cmNlX2NvZGUYAiABKAkSEgoKZXZlbnRfdHlwZRgDIAEoCRIbCg5jb3JyZWxhdGlvbl9pZBgEIAEoCUgAiAEBEhMKC29jY3VycmVkX2F0GAUgASgJEg8KB3N1bW1hcnkYBiABKAlCEQoPX2NvcnJlbGF0aW9uX2lkIhsKGUxpc3RGaXh0dXJlU291cmNlc1JlcXVlc3QiVgoaTGlzdEZpeHR1cmVTb3VyY2VzUmVzcG9uc2USOAoFaXRlbXMYASADKAsyKS5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxGaXh0dXJlU291cmNlIpYDChBTaWduYWxDb25uZWN0aW9uEgoKAmlkGAEgASgJEhMKC3NvdXJjZV9jb2RlGAIgASgJEhQKDGRpc3BsYXlfbmFtZRgDIAEoCRIOCgZzdGF0dXMYBCABKAkSFAoHcHJvZmlsZRgFIAEoCUgAiAEBEhcKCnNlY3JldF9yZWYYBiABKAlIAYgBARIZCgxjb25uZWN0ZWRfYXQYByABKAlIAogBARIZCgxsYXN0X3NlZW5fYXQYCCABKAlIA4gBARIbCg5sYXN0X3NpZ25hbF9hdBgJIAEoCUgEiAEBEhkKDGxhc3Rfc3luY19hdBgKIAEoCUgFiAEBEhIKCmNyZWF0ZWRfYXQYCyABKAkSEgoKdXBkYXRlZF9hdBgMIAEoCRIVCg1zZXR0aW5nc19qc29uGA0gASgJQgoKCF9wcm9maWxlQg0KC19zZWNyZXRfcmVmQg8KDV9jb25uZWN0ZWRfYXRCDwoNX2xhc3Rfc2Vlbl9hdEIRCg9fbGFzdF9zaWduYWxfYXRCDwoNX2xhc3Rfc3luY19hdCIYChZMaXN0Q29ubmVjdGlvbnNSZXF1ZXN0IlAKF0xpc3RDb25uZWN0aW9uc1Jlc3BvbnNlEjUKBWl0ZW1zGAEgAygLMiYuaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsQ29ubmVjdGlvbiK1AQoXQ3JlYXRlQ29ubmVjdGlvblJlcXVlc3QSEwoLc291cmNlX2NvZGUYASABKAkSFAoMZGlzcGxheV9uYW1lGAIgASgJEg4KBnN0YXR1cxgDIAEoCRIUCgdwcm9maWxlGAQgASgJSACIAQESFwoKc2VjcmV0X3JlZhgFIAEoCUgBiAEBEhUKDXNldHRpbmdzX2pzb24YBiABKAlCCgoIX3Byb2ZpbGVCDQoLX3NlY3JldF9yZWYiUAoYQ3JlYXRlQ29ubmVjdGlvblJlc3BvbnNlEjQKBGl0ZW0YASABKAsyJi5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxDb25uZWN0aW9uIukBChdVcGRhdGVDb25uZWN0aW9uUmVxdWVzdBIKCgJpZBgBIAEoCRIZCgxkaXNwbGF5X25hbWUYAiABKAlIAIgBARITCgZzdGF0dXMYAyABKAlIAYgBARIUCgdwcm9maWxlGAQgASgJSAKIAQESFwoKc2VjcmV0X3JlZhgFIAEoCUgDiAEBEhoKDXNldHRpbmdzX2pzb24YBiABKAlIBIgBAUIPCg1fZGlzcGxheV9uYW1lQgkKB19zdGF0dXNCCgoIX3Byb2ZpbGVCDQoLX3NlY3JldF9yZWZCEAoOX3NldHRpbmdzX2pzb24iUAoYVXBkYXRlQ29ubmVjdGlvblJlc3BvbnNlEjQKBGl0ZW0YASABKAsyJi5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxDb25uZWN0aW9uIiUKF1JlbW92ZUNvbm5lY3Rpb25SZXF1ZXN0EgoKAmlkGAEgASgJIlAKGFJlbW92ZUNvbm5lY3Rpb25SZXNwb25zZRI0CgRpdGVtGAEgASgLMiYuaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsQ29ubmVjdGlvbiLqAgoMU2lnbmFsSGVhbHRoEgoKAmlkGAEgASgJEhMKC3NvdXJjZV9jb2RlGAIgASgJEhoKDWNvbm5lY3Rpb25faWQYAyABKAlIAIgBARINCgVsZXZlbBgEIAEoCRIPCgdzdW1tYXJ5GAUgASgJEhcKCmxhc3Rfb2tfYXQYBiABKAlIAYgBARIcCg9sYXN0X2ZhaWx1cmVfYXQYByABKAlIAogBARIVCg1mYWlsdXJlX2NvdW50GAggASgFEiEKGWNvbnNlY3V0aXZlX2ZhaWx1cmVfY291bnQYCSABKAUSGgoNbmV4dF9yZXRyeV9hdBgKIAEoCUgDiAEBEhIKCnVwZGF0ZWRfYXQYCyABKAkSFQoNZXZpZGVuY2VfanNvbhgMIAEoCUIQCg5fY29ubmVjdGlvbl9pZEINCgtfbGFzdF9va19hdEISChBfbGFzdF9mYWlsdXJlX2F0QhAKDl9uZXh0X3JldHJ5X2F0IhMKEUxpc3RIZWFsdGhSZXF1ZXN0IkcKEkxpc3RIZWFsdGhSZXNwb25zZRIxCgVpdGVtcxgBIAMoCzIiLmhlcm1lcy5zaWduYWxfaHViLnYxLlNpZ25hbEhlYWx0aCJaChVSdW5IZWFsdGhDaGVja1JlcXVlc3QSEwoLc291cmNlX2NvZGUYASABKAkSGgoNY29ubmVjdGlvbl9pZBgCIAEoCUgAiAEBQhAKDl9jb25uZWN0aW9uX2lkIkoKFlJ1bkhlYWx0aENoZWNrUmVzcG9uc2USMAoEaXRlbRgBIAEoCzIiLmhlcm1lcy5zaWduYWxfaHViLnYxLlNpZ25hbEhlYWx0aCL3AwoSU2lnbmFsUnVudGltZVN0YXRlEgoKAmlkGAEgASgJEhMKC3NvdXJjZV9jb2RlGAIgASgJEhoKDWNvbm5lY3Rpb25faWQYAyABKAlIAIgBARIUCgxydW50aW1lX2tpbmQYBCABKAkSDQoFc3RhdGUYBSABKAkSFQoNbWV0YWRhdGFfanNvbhgGIAEoCRISCgp1cGRhdGVkX2F0GAcgASgJEhwKD2xhc3Rfc3RhcnRlZF9hdBgIIAEoCUgBiAEBEhwKD2xhc3Rfc3RvcHBlZF9hdBgJIAEoCUgCiAEBEh4KEWxhc3RfaGVhcnRiZWF0X2F0GAogASgJSAOIAQESGgoNbGFzdF9lcnJvcl9hdBgLIAEoCUgEiAEBEhwKD2xhc3RfZXJyb3JfY29kZRgMIAEoCUgFiAEBEigKG2xhc3RfZXJyb3JfbWVzc2FnZV9yZWRhY3RlZBgNIAEoCUgGiAEBQhAKDl9jb25uZWN0aW9uX2lkQhIKEF9sYXN0X3N0YXJ0ZWRfYXRCEgoQX2xhc3Rfc3RvcHBlZF9hdEIUChJfbGFzdF9oZWFydGJlYXRfYXRCEAoOX2xhc3RfZXJyb3JfYXRCEgoQX2xhc3RfZXJyb3JfY29kZUIeChxfbGFzdF9lcnJvcl9tZXNzYWdlX3JlZGFjdGVkIhoKGExpc3RSdW50aW1lU3RhdGVzUmVxdWVzdCJUChlMaXN0UnVudGltZVN0YXRlc1Jlc3BvbnNlEjcKBWl0ZW1zGAEgAygLMiguaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsUnVudGltZVN0YXRlImwKGVVwZGF0ZVJ1bnRpbWVTdGF0ZVJlcXVlc3QSEwoLc291cmNlX2NvZGUYASABKAkSFAoMcnVudGltZV9raW5kGAIgASgJEg0KBXN0YXRlGAMgASgJEhUKDW1ldGFkYXRhX2pzb24YBCABKAkiVAoaVXBkYXRlUnVudGltZVN0YXRlUmVzcG9uc2USNgoEaXRlbRgBIAEoCzIoLmhlcm1lcy5zaWduYWxfaHViLnYxLlNpZ25hbFJ1bnRpbWVTdGF0ZSLpAQoMU2lnbmFsUG9saWN5Eg0KBXNjb3BlGAEgASgJEhgKC3NvdXJjZV9jb2RlGAIgASgJSACIAQESGgoNY29ubmVjdGlvbl9pZBgDIAEoCUgBiAEBEhoKDWV2ZW50X3BhdHRlcm4YBCABKAlIAogBARIMCgRtb2RlGAUgASgJEg4KBnJlYXNvbhgGIAEoCRIXCgpleHBpcmVzX2F0GAcgASgJSAOIAQFCDgoMX3NvdXJjZV9jb2RlQhAKDl9jb25uZWN0aW9uX2lkQhAKDl9ldmVudF9wYXR0ZXJuQg0KC19leHBpcmVzX2F0IhUKE0xpc3RQb2xpY2llc1JlcXVlc3QiSQoUTGlzdFBvbGljaWVzUmVzcG9uc2USMQoFaXRlbXMYASADKAsyIi5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxQb2xpY3ki8AEKE0NyZWF0ZVBvbGljeVJlcXVlc3QSDQoFc2NvcGUYASABKAkSGAoLc291cmNlX2NvZGUYAiABKAlIAIgBARIaCg1jb25uZWN0aW9uX2lkGAMgASgJSAGIAQESGgoNZXZlbnRfcGF0dGVybhgEIAEoCUgCiAEBEgwKBG1vZGUYBSABKAkSDgoGcmVhc29uGAYgASgJEhcKCmV4cGlyZXNfYXQYByABKAlIA4gBAUIOCgxfc291cmNlX2NvZGVCEAoOX2Nvbm5lY3Rpb25faWRCEAoOX2V2ZW50X3BhdHRlcm5CDQoLX2V4cGlyZXNfYXQiIgoUQ3JlYXRlUG9saWN5UmVzcG9uc2USCgoCaWQYASABKAkiSgoTRW5hYmxlU291cmNlUmVxdWVzdBITCgtzb3VyY2VfY29kZRgBIAEoCRITCgZyZWFzb24YAiABKAlIAIgBAUIJCgdfcmVhc29uIkIKFEVuYWJsZVNvdXJjZVJlc3BvbnNlEhMKC3NvdXJjZV9jb2RlGAEgASgJEhUKDWNsZWFyZWRfY291bnQYAiABKA0iSwoURGlzYWJsZVNvdXJjZVJlcXVlc3QSEwoLc291cmNlX2NvZGUYASABKAkSEwoGcmVhc29uGAIgASgJSACIAQFCCQoHX3JlYXNvbiI/ChVEaXNhYmxlU291cmNlUmVzcG9uc2USEwoLc291cmNlX2NvZGUYASABKAkSEQoJcG9saWN5X2lkGAIgASgJIswBChVEaXNhYmxlU2lnbmFsc1JlcXVlc3QSDQoFc2NvcGUYASABKAkSGAoLc291cmNlX2NvZGUYAiABKAlIAIgBARIaCg1jb25uZWN0aW9uX2lkGAMgASgJSAGIAQESGgoNZXZlbnRfcGF0dGVybhgEIAEoCUgCiAEBEhMKBnJlYXNvbhgFIAEoCUgDiAEBQg4KDF9zb3VyY2VfY29kZUIQCg5fY29ubmVjdGlvbl9pZEIQCg5fZXZlbnRfcGF0dGVybkIJCgdfcmVhc29uIj4KFkRpc2FibGVTaWduYWxzUmVzcG9uc2USFgoJcG9saWN5X2lkGAEgASgJSACIAQFCDAoKX3BvbGljeV9pZCLLAQoURW5hYmxlU2lnbmFsc1JlcXVlc3QSDQoFc2NvcGUYASABKAkSGAoLc291cmNlX2NvZGUYAiABKAlIAIgBARIaCg1jb25uZWN0aW9uX2lkGAMgASgJSAGIAQESGgoNZXZlbnRfcGF0dGVybhgEIAEoCUgCiAEBEhMKBnJlYXNvbhgFIAEoCUgDiAEBQg4KDF9zb3VyY2VfY29kZUIQCg5fY29ubmVjdGlvbl9pZEIQCg5fZXZlbnRfcGF0dGVybkIJCgdfcmVhc29uIi4KFUVuYWJsZVNpZ25hbHNSZXNwb25zZRIVCg1jbGVhcmVkX2NvdW50GAEgASgNIskBChJNdXRlU2lnbmFsc1JlcXVlc3QSDQoFc2NvcGUYASABKAkSGAoLc291cmNlX2NvZGUYAiABKAlIAIgBARIaCg1jb25uZWN0aW9uX2lkGAMgASgJSAGIAQESGgoNZXZlbnRfcGF0dGVybhgEIAEoCUgCiAEBEhMKBnJlYXNvbhgFIAEoCUgDiAEBQg4KDF9zb3VyY2VfY29kZUIQCg5fY29ubmVjdGlvbl9pZEIQCg5fZXZlbnRfcGF0dGVybkIJCgdfcmVhc29uIjsKE011dGVTaWduYWxzUmVzcG9uc2USFgoJcG9saWN5X2lkGAEgASgJSACIAQFCDAoKX3BvbGljeV9pZCLLAQoUVW5tdXRlU2lnbmFsc1JlcXVlc3QSDQoFc2NvcGUYASABKAkSGAoLc291cmNlX2NvZGUYAiABKAlIAIgBARIaCg1jb25uZWN0aW9uX2lkGAMgASgJSAGIAQESGgoNZXZlbnRfcGF0dGVybhgEIAEoCUgCiAEBEhMKBnJlYXNvbhgFIAEoCUgDiAEBQg4KDF9zb3VyY2VfY29kZUIQCg5fY29ubmVjdGlvbl9pZEIQCg5fZXZlbnRfcGF0dGVybkIJCgdfcmVhc29uIi4KFVVubXV0ZVNpZ25hbHNSZXNwb25zZRIVCg1jbGVhcmVkX2NvdW50GAEgASgNIsoBChNQYXVzZVNpZ25hbHNSZXF1ZXN0Eg0KBXNjb3BlGAEgASgJEhgKC3NvdXJjZV9jb2RlGAIgASgJSACIAQESGgoNY29ubmVjdGlvbl9pZBgDIAEoCUgBiAEBEhoKDWV2ZW50X3BhdHRlcm4YBCABKAlIAogBARITCgZyZWFzb24YBSABKAlIA4gBAUIOCgxfc291cmNlX2NvZGVCEAoOX2Nvbm5lY3Rpb25faWRCEAoOX2V2ZW50X3BhdHRlcm5CCQoHX3JlYXNvbiI8ChRQYXVzZVNpZ25hbHNSZXNwb25zZRIWCglwb2xpY3lfaWQYASABKAlIAIgBAUIMCgpfcG9saWN5X2lkIssBChRSZXN1bWVTaWduYWxzUmVxdWVzdBINCgVzY29wZRgBIAEoCRIYCgtzb3VyY2VfY29kZRgCIAEoCUgAiAEBEhoKDWNvbm5lY3Rpb25faWQYAyABKAlIAYgBARIaCg1ldmVudF9wYXR0ZXJuGAQgASgJSAKIAQESEwoGcmVhc29uGAUgASgJSAOIAQFCDgoMX3NvdXJjZV9jb2RlQhAKDl9jb25uZWN0aW9uX2lkQhAKDl9ldmVudF9wYXR0ZXJuQgkKB19yZWFzb24iLgoVUmVzdW1lU2lnbmFsc1Jlc3BvbnNlEhUKDWNsZWFyZWRfY291bnQYASABKA0i/AEKDVNpZ25hbFByb2ZpbGUSCgoCaWQYASABKAkSDAoEY29kZRgCIAEoCRIUCgxkaXNwbGF5X25hbWUYAyABKAkSEwoLZGVzY3JpcHRpb24YBCABKAkSFAoMcG9saWN5X2NvdW50GAUgASgNEhEKCWlzX3N5c3RlbRgGIAEoCBIRCglpc19hY3RpdmUYByABKAgSEgoKY3JlYXRlZF9hdBgIIAEoCRISCgp1cGRhdGVkX2F0GAkgASgJEkIKD3NvdXJjZV9wb2xpY2llcxgKIAMoCzIpLmhlcm1lcy5zaWduYWxfaHViLnYxLlNpZ25hbFByb2ZpbGVQb2xpY3kiyAEKE1NpZ25hbFByb2ZpbGVQb2xpY3kSDQoFc2NvcGUYASABKAkSGAoLc291cmNlX2NvZGUYAiABKAlIAIgBARIaCg1jb25uZWN0aW9uX2lkGAMgASgJSAGIAQESGgoNZXZlbnRfcGF0dGVybhgEIAEoCUgCiAEBEgwKBG1vZGUYBSABKAkSDgoGcmVhc29uGAYgASgJQg4KDF9zb3VyY2VfY29kZUIQCg5fY29ubmVjdGlvbl9pZEIQCg5fZXZlbnRfcGF0dGVybiIVChNMaXN0UHJvZmlsZXNSZXF1ZXN0IkoKFExpc3RQcm9maWxlc1Jlc3BvbnNlEjIKBWl0ZW1zGAEgAygLMiMuaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsUHJvZmlsZSKTAQoUQ3JlYXRlUHJvZmlsZVJlcXVlc3QSDAoEY29kZRgBIAEoCRIUCgxkaXNwbGF5X25hbWUYAiABKAkSEwoLZGVzY3JpcHRpb24YAyABKAkSQgoPc291cmNlX3BvbGljaWVzGAQgAygLMikuaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsUHJvZmlsZVBvbGljeSJKChVDcmVhdGVQcm9maWxlUmVzcG9uc2USMQoEaXRlbRgBIAEoCzIjLmhlcm1lcy5zaWduYWxfaHViLnYxLlNpZ25hbFByb2ZpbGUi3gEKFFVwZGF0ZVByb2ZpbGVSZXF1ZXN0EgwKBGNvZGUYASABKAkSGQoMZGlzcGxheV9uYW1lGAIgASgJSACIAQESGAoLZGVzY3JpcHRpb24YAyABKAlIAYgBARJCCg9zb3VyY2VfcG9saWNpZXMYBCADKAsyKS5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxQcm9maWxlUG9saWN5Eh4KFnVwZGF0ZV9zb3VyY2VfcG9saWNpZXMYBSABKAhCDwoNX2Rpc3BsYXlfbmFtZUIOCgxfZGVzY3JpcHRpb24iSgoVVXBkYXRlUHJvZmlsZVJlc3BvbnNlEjEKBGl0ZW0YASABKAsyIy5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxQcm9maWxlIiQKFFJlbW92ZVByb2ZpbGVSZXF1ZXN0EgwKBGNvZGUYASABKAkiSgoVUmVtb3ZlUHJvZmlsZVJlc3BvbnNlEjEKBGl0ZW0YASABKAsyIy5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxQcm9maWxlIiMKE0FwcGx5UHJvZmlsZVJlcXVlc3QSDAoEY29kZRgBIAEoCSJJChRBcHBseVByb2ZpbGVSZXNwb25zZRIxCgRpdGVtGAEgASgLMiMuaGVybWVzLnNpZ25hbF9odWIudjEuU2lnbmFsUHJvZmlsZSKoBQoTU2lnbmFsUmVwbGF5UmVxdWVzdBIKCgJpZBgBIAEoCRIYCgtzb3VyY2VfY29kZRgCIAEoCUgAiAEBEhoKDWNvbm5lY3Rpb25faWQYAyABKAlIAYgBARIaCg1ldmVudF9wYXR0ZXJuGAQgASgJSAKIAQESGgoNZnJvbV9wb3NpdGlvbhgFIAEoA0gDiAEBEhgKC3RvX3Bvc2l0aW9uGAYgASgDSASIAQESFgoJZnJvbV90aW1lGAcgASgJSAWIAQESFAoHdG9fdGltZRgIIAEoCUgGiAEBEhwKD3RhcmdldF9jb25zdW1lchgJIAEoCUgHiAEBEh4KEXRhcmdldF9wcm9qZWN0aW9uGAogASgJSAiIAQESDgoGc3RhdHVzGAsgASgJEhQKDHJlcXVlc3RlZF9ieRgMIAEoCRIUCgxyZXF1ZXN0ZWRfYXQYDSABKAkSFwoKc3RhcnRlZF9hdBgOIAEoCUgJiAEBEhkKDGNvbXBsZXRlZF9hdBgPIAEoCUgKiAEBEiAKE2xhc3RfZXJyb3JfcmVkYWN0ZWQYECABKAlIC4gBARIWCg5yZXBsYXllZF9jb3VudBgRIAEoBRIVCg1tZXRhZGF0YV9qc29uGBIgASgJQg4KDF9zb3VyY2VfY29kZUIQCg5fY29ubmVjdGlvbl9pZEIQCg5fZXZlbnRfcGF0dGVybkIQCg5fZnJvbV9wb3NpdGlvbkIOCgxfdG9fcG9zaXRpb25CDAoKX2Zyb21fdGltZUIKCghfdG9fdGltZUISChBfdGFyZ2V0X2NvbnN1bWVyQhQKEl90YXJnZXRfcHJvamVjdGlvbkINCgtfc3RhcnRlZF9hdEIPCg1fY29tcGxldGVkX2F0QhYKFF9sYXN0X2Vycm9yX3JlZGFjdGVkIhsKGUxpc3RSZXBsYXlSZXF1ZXN0c1JlcXVlc3QiVgoaTGlzdFJlcGxheVJlcXVlc3RzUmVzcG9uc2USOAoFaXRlbXMYASADKAsyKS5oZXJtZXMuc2lnbmFsX2h1Yi52MS5TaWduYWxSZXBsYXlSZXF1ZXN0IrsDChRSZXF1ZXN0
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/mail/api/accountSetup.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/api/accountSetup.test.ts`
- Size bytes / Размер в байтах: `3179`
- Included characters / Включено символов: `3179`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
	setupImapEmailAccount,
	startGmailOAuthSetup
} from './accountSetup'

describe('account setup API', () => {
	beforeEach(() => {
		ApiClient.resetForTests()
		ApiClient.init('http://127.0.0.1:8080', 'test-secret')
	})

	afterEach(() => {
		vi.unstubAllGlobals()
		ApiClient.resetForTests()
	})

	it('starts Gmail OAuth through the protected email account setup endpoint', async () => {
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(
				JSON.stringify({
					setup_id: 'setup-1',
					authorization_url: 'https://accounts.google.com/o/oauth2/v2/auth?state=oauth-state',
					state: 'oauth-state',
					redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback'
				}),
				{ status: 200, headers: { 'Content-Type': 'application/json' } }
			)
		)
		vi.stubGlobal('fetch', fetchMock)

		const response = await startGmailOAuthSetup({
			account_id: 'mail-gmail-user-gmail-com',
			display_name: 'user@gmail.com',
			external_account_id: 'user@gmail.com',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback'
		})

		expect(response.setup_id).toBe('setup-1')
		expect(fetchMock).toHaveBeenCalledOnce()
		const [url, init] = fetchMock.mock.calls[0]
		expect(url).toBe('http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/start')
		expect(init.method).toBe('POST')
		expect(init.headers['X-Макошь-Secret']).toBe('test-secret')
		expect(JSON.parse(init.body as string)).toEqual({
			account_id: 'mail-gmail-user-gmail-com',
			display_name: 'user@gmail.com',
			external_account_id: 'user@gmail.com',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback'
		})
	})

	it('creates IMAP-backed accounts through the protected setup endpoint', async () => {
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(
				JSON.stringify({
					account_id: 'mail-imap-user-example-com',
					secret_ref: 'secret:provider-account:mail-imap-user-example-com:imap_password',
					secret_kind: 'password',
					store_kind: 'host_vault'
				}),
				{ status: 200, headers: { 'Content-Type': 'application/json' } }
			)
		)
		vi.stubGlobal('fetch', fetchMock)

		const response = await setupImapEmailAccount({
			account_id: 'mail-imap-user-example-com',
			provider_kind: 'imap',
			display_name: 'User',
			external_account_id: 'user@example.com',
			host: 'imap.example.com',
			port: 993,
			tls: true,
			mailbox: 'INBOX',
			username: 'user@example.com',
			password: 'mailbox-password',
			secret_kind: 'password'
		})

		expect(response.account_id).toBe('mail-imap-user-example-com')
		expect(fetchMock).toHaveBeenCalledOnce()
		const [url, init] = fetchMock.mock.calls[0]
		expect(url).toBe('http://127.0.0.1:8080/api/v1/integrations/mail/accounts/imap')
		expect(init.method).toBe('POST')
		expect(JSON.parse(init.body as string)).toMatchObject({
			account_id: 'mail-imap-user-example-com',
			provider_kind: 'imap',
			password: 'mailbox-password',
			secret_kind: 'password'
		})
	})
})
```

### `frontend/src/integrations/mail/api/accountSetup.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/api/accountSetup.ts`
- Size bytes / Размер в байтах: `1732`
- Included characters / Включено символов: `1732`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'

export type EmailAccountSetupSecretKind = 'app_password' | 'password' | 'oauth_token'
export type EmailAccountSetupStoreKind =
	| 'host_vault'
	| 'os_keychain'
	| 'encrypted_vault'
	| 'database_encrypted_vault'
	| 'external_vault'
	| string

export type GmailOAuthStartRequest = {
	account_id: string
	display_name: string
	external_account_id?: string
	redirect_uri: string
	app_return_url?: string
	scopes?: string[]
}

export type GmailOAuthStartResponse = {
	setup_id: string
	authorization_url: string
	state: string
	redirect_uri: string
}

export type ImapEmailAccountSetupRequest = {
	account_id: string
	provider_kind: 'icloud' | 'imap'
	display_name: string
	external_account_id: string
	host: string
	port: number
	tls: boolean
	mailbox: string
	username: string
	password: string
	secret_kind: 'app_password' | 'password'
	smtp_host?: string
	smtp_port?: number
	smtp_tls?: boolean
	smtp_starttls?: boolean
	smtp_username?: string
}

export type EmailAccountSetupResponse = {
	account_id: string
	secret_ref: string
	secret_kind: EmailAccountSetupSecretKind
	store_kind: EmailAccountSetupStoreKind
}

export async function startGmailOAuthSetup(
	request: GmailOAuthStartRequest
): Promise<GmailOAuthStartResponse> {
	return ApiClient.instance.post<GmailOAuthStartResponse>(
		'/api/v1/integrations/mail/accounts/gmail/oauth/start',
		request,
		'Gmail OAuth setup start failed'
	)
}

export async function setupImapEmailAccount(
	request: ImapEmailAccountSetupRequest
): Promise<EmailAccountSetupResponse> {
	return ApiClient.instance.post<EmailAccountSetupResponse>(
		'/api/v1/integrations/mail/accounts/imap',
		request,
		'IMAP account setup failed'
	)
}
```

### `frontend/src/integrations/mail/api/syncApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/api/syncApi.ts`
- Size bytes / Размер в байтах: `1631`
- Included characters / Включено символов: `1631`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  MailSyncStatusListResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncRunResponse
} from '../../../shared/mailSync/types'

export async function fetchMailSyncStatus(): Promise<MailSyncStatusListResponse> {
  return ApiClient.instance.get<MailSyncStatusListResponse>(
    '/api/v1/integrations/mail/accounts/sync-status',
    'Mail sync status request failed'
  )
}

export async function fetchMailSyncSettings(accountId: string): Promise<MailSyncSettings> {
  return ApiClient.instance.get<MailSyncSettings>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-settings`,
    'Mail sync settings request failed'
  )
}

export async function updateMailSyncSettings(
  accountId: string,
  settings: MailSyncSettingsUpdate
): Promise<MailSyncSettings> {
  return ApiClient.instance.put<MailSyncSettings>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-settings`,
    settings,
    'Mail sync settings update failed'
  )
}

export async function runMailSyncNow(accountId: string): Promise<MailSyncRunResponse> {
  return ApiClient.instance.post<MailSyncRunResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-now`,
    {},
    'Mail sync request failed'
  )
}

export async function runMailFullResync(accountId: string): Promise<MailSyncRunResponse> {
  return ApiClient.instance.post<MailSyncRunResponse>(
    `/api/v1/integrations/mail/accounts/${encodeURIComponent(accountId)}/sync-full-resync`,
    {},
    'Mail full resync request failed'
  )
}
```

### `frontend/src/integrations/mail/components/AccountSetupModal.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/components/AccountSetupModal.boundary.test.ts`
- Size bytes / Размер в байтах: `735`
- Included characters / Включено символов: `735`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('AccountSetupModal account setup boundary', () => {
	it('uses validated account setup helpers and real API calls instead of simulated success', () => {
		const source = readFileSync(
			new URL('./AccountSetupModal.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain("from 'vee-validate'")
		expect(source).toContain('../forms/accountSetupForm')
		expect(source).toContain('../queries/accountSetupQueries')
		expect(source).toContain('useSetupImapEmailAccountMutation')
		expect(source).toContain('useStartGmailOAuthSetupMutation')
		expect(source).toContain('mutateAsync')
		expect(source).not.toContain('setTimeout')
	})
})
```

### `frontend/src/integrations/mail/components/MailSyncSettingsStrip.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/components/MailSyncSettingsStrip.boundary.test.ts`
- Size bytes / Размер в байтах: `746`
- Included characters / Включено символов: `746`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailSyncSettingsStrip boundary', () => {
	it('renders provider sync settings controls without direct API access', () => {
		const source = readFileSync(new URL('./MailSyncSettingsStrip.vue', import.meta.url), 'utf8')

		expect(source).toContain('sync_enabled')
		expect(source).toContain('batch_size')
		expect(source).toContain('poll_interval_seconds')
		expect(source).toContain('Provider sync')
		expect(source).toContain('Save')
		expect(source).toContain('defineEmits')
		expect(source).toContain('update')
		expect(source).not.toContain('../api/')
		expect(source).not.toMatch(/\bfetch\s*\(/)
		expect(source).not.toContain('ApiClient')
	})
})
```

### `frontend/src/integrations/mail/forms/accountSetupForm.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/forms/accountSetupForm.test.ts`
- Size bytes / Размер в байтах: `3172`
- Included characters / Включено символов: `3172`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
	accountSetupDefaultAccountId,
	accountSetupFormDefaults,
	accountSetupFormSchema,
	accountSetupFormToGmailOAuthStart,
	accountSetupFormToImapRequest
} from './accountSetupForm'

describe('account setup form', () => {
	it('normalizes iCloud values into the protected IMAP setup payload', () => {
		const values = accountSetupFormSchema.parse({
			...accountSetupFormDefaults('icloud'),
			display_name: '  Personal iCloud  ',
			email: ' User@iCloud.com ',
			password: 'icloud-app-password'
		})

		expect(accountSetupFormToImapRequest(values)).toEqual({
			account_id: 'mail-icloud-user-icloud-com',
			provider_kind: 'icloud',
			display_name: 'Personal iCloud',
			external_account_id: 'user@icloud.com',
			host: 'imap.mail.me.com',
			port: 993,
			tls: true,
			mailbox: 'INBOX',
			username: 'user@icloud.com',
			password: 'icloud-app-password',
			secret_kind: 'app_password'
		})
	})

	it('normalizes generic IMAP values with SMTP settings', () => {
		const values = accountSetupFormSchema.parse({
			...accountSetupFormDefaults('imap'),
			email: 'sender@example.com',
			password: 'mailbox-password',
			imap_host: ' imap.example.com ',
			imap_port: 143,
			imap_tls: false,
			username: ' sender@example.com ',
			smtp_host: ' smtp.example.com ',
			smtp_port: 587,
			smtp_tls: false,
			smtp_starttls: true,
			smtp_username: ' sender@example.com '
		})

		expect(accountSetupFormToImapRequest(values)).toEqual({
			account_id: 'mail-imap-sender-example-com',
			provider_kind: 'imap',
			display_name: 'sender@example.com',
			external_account_id: 'sender@example.com',
			host: 'imap.example.com',
			port: 143,
			tls: false,
			mailbox: 'INBOX',
			username: 'sender@example.com',
			password: 'mailbox-password',
			secret_kind: 'password',
			smtp_host: 'smtp.example.com',
			smtp_port: 587,
			smtp_tls: false,
			smtp_starttls: true,
			smtp_username: 'sender@example.com'
		})
	})

	it('rejects provider-specific missing credential and host fields', () => {
		const result = accountSetupFormSchema.safeParse({
			...accountSetupFormDefaults('imap'),
			email: 'sender@example.com',
			password: '',
			imap_host: ''
		})

		expect(result.success).toBe(false)
		if (!result.success) {
			expect(result.error.issues.map((issue) => issue.path.join('.'))).toEqual([
				'password',
				'imap_host'
			])
		}
	})

	it('builds Gmail OAuth start requests without password fields', () => {
		const values = accountSetupFormSchema.parse({
			...accountSetupFormDefaults('gmail'),
			display_name: '',
			email: ' User@Gmail.com ',
			password: 'ignored-password'
		})

		expect(accountSetupFormToGmailOAuthStart(values, 'http://127.0.0.1:8080/')).toEqual({
			account_id: 'mail-gmail-user-gmail-com',
			display_name: 'user@gmail.com',
			external_account_id: 'user@gmail.com',
			redirect_uri: 'http://127.0.0.1:8080/api/v1/integrations/mail/accounts/gmail/oauth/callback'
		})
	})

	it('generates stable safe account ids from provider and email', () => {
		expect(accountSetupDefaultAccountId('imap', 'Team.Mail+Archive@example.org')).toBe(
			'mail-imap-team-mail-archive-example-org'
		)
	})
})
```

### `frontend/src/integrations/mail/forms/accountSetupForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/forms/accountSetupForm.ts`
- Size bytes / Размер в байтах: `4114`
- Included characters / Включено символов: `4114`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type {
	GmailOAuthStartRequest,
	ImapEmailAccountSetupRequest
} from '../api/accountSetup'

const providerKinds = ['gmail', 'icloud', 'imap'] as const

export type MailAccountSetupProvider = typeof providerKinds[number]
export type AccountSetupFormValues = z.infer<typeof accountSetupFormSchema>

const emailSchema = z
	.string()
	.trim()
	.email('Valid email address is required')
	.transform((value) => value.toLowerCase())

export const accountSetupFormSchema = z.object({
	provider_kind: z.enum(providerKinds),
	display_name: z.string().trim().max(120, 'Account name is too long'),
	email: emailSchema,
	password: z.string(),
	imap_host: z.string().trim(),
	imap_port: z.coerce.number().int().min(1, 'IMAP port is required').max(65535, 'IMAP port is invalid'),
	imap_tls: z.boolean(),
	mailbox: z.string().trim().min(1, 'Mailbox is required'),
	username: z.string().trim(),
	smtp_host: z.string().trim(),
	smtp_port: z.coerce.number().int().min(1, 'SMTP port is required').max(65535, 'SMTP port is invalid'),
	smtp_tls: z.boolean(),
	smtp_starttls: z.boolean(),
	smtp_username: z.string().trim()
}).superRefine((values, context) => {
	if (values.provider_kind === 'gmail') return

	if (values.password.trim().length === 0) {
		context.addIssue({
			code: z.ZodIssueCode.custom,
			message: values.provider_kind === 'icloud'
				? 'App password is required'
				: 'Password is required',
			path: ['password']
		})
	}
	if (values.imap_host.trim().length === 0) {
		context.addIssue({
			code: z.ZodIssueCode.custom,
			message: 'IMAP host is required',
			path: ['imap_host']
		})
	}
})

export const accountSetupVeeValidationSchema = toTypedSchema(accountSetupFormSchema)

export function accountSetupFormDefaults(
	provider: MailAccountSetupProvider
): AccountSetupFormValues {
	return {
		provider_kind: provider,
		display_name: '',
		email: '',
		password: '',
		imap_host: provider === 'icloud' ? 'imap.mail.me.com' : '',
		imap_port: 993,
		imap_tls: true,
		mailbox: 'INBOX',
		username: '',
		smtp_host: '',
		smtp_port: 587,
		smtp_tls: true,
		smtp_starttls: true,
		smtp_username: ''
	}
}

export function accountSetupDefaultAccountId(
	provider: MailAccountSetupProvider,
	email: string
): string {
	const slug = email
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '')
	return `mail-${provider}-${slug || 'account'}`
}

export function accountSetupFormToImapRequest(
	values: AccountSetupFormValues
): ImapEmailAccountSetupRequest {
	const parsed = accountSetupFormSchema.parse(values)
	if (parsed.provider_kind === 'gmail') {
		throw new Error('Gmail account setup must use OAuth')
	}

	const request: ImapEmailAccountSetupRequest = {
		account_id: accountSetupDefaultAccountId(parsed.provider_kind, parsed.email),
		provider_kind: parsed.provider_kind,
		display_name: parsed.display_name || parsed.email,
		external_account_id: parsed.email,
		host: parsed.imap_host,
		port: parsed.imap_port,
		tls: parsed.imap_tls,
		mailbox: parsed.mailbox,
		username: parsed.username || parsed.email,
		password: parsed.password,
		secret_kind: parsed.provider_kind === 'icloud' ? 'app_password' : 'password'
	}

	if (parsed.smtp_host || parsed.smtp_username) {
		request.smtp_host = parsed.smtp_host || undefined
		request.smtp_port = parsed.smtp_port
		request.smtp_tls = parsed.smtp_tls
		request.smtp_starttls = parsed.smtp_starttls
		request.smtp_username = parsed.smtp_username || undefined
	}

	return request
}

export function accountSetupFormToGmailOAuthStart(
	values: AccountSetupFormValues,
	apiBaseUrl: string
): GmailOAuthStartRequest {
	const parsed = accountSetupFormSchema.parse(values)
	return {
		account_id: accountSetupDefaultAccountId('gmail', parsed.email),
		display_name: parsed.display_name || parsed.email,
		external_account_id: parsed.email,
		redirect_uri: gmailOAuthRedirectUri(apiBaseUrl)
	}
}

function gmailOAuthRedirectUri(apiBaseUrl: string): string {
	return `${apiBaseUrl.replace(/\/+$/, '')}/api/v1/integrations/mail/accounts/gmail/oauth/callback`
}
```

### `frontend/src/integrations/mail/forms/syncSettingsForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/forms/syncSettingsForm.ts`
- Size bytes / Размер в байтах: `1341`
- Included characters / Включено символов: `1341`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from '../../../shared/mailSync/types'

export const syncSettingsFormSchema = z.object({
  sync_enabled: z.boolean(),
  batch_size: z.coerce
    .number()
    .int('Batch size must be a whole number')
    .min(1, 'Batch size must be at least 1')
    .max(500, 'Batch size must be 500 or less'),
  poll_interval_seconds: z.coerce
    .number()
    .int('Poll interval must be a whole number')
    .min(60, 'Poll interval must be at least 60 seconds')
    .max(86400, 'Poll interval must be 86400 seconds or less')
})

export type SyncSettingsFormValues = z.infer<typeof syncSettingsFormSchema>

export const syncSettingsVeeValidationSchema = toTypedSchema(syncSettingsFormSchema)

export function syncSettingsFormDefaults(settings: MailSyncSettings | null): SyncSettingsFormValues {
  return {
    sync_enabled: settings?.sync_enabled ?? true,
    batch_size: settings?.batch_size ?? 100,
    poll_interval_seconds: settings?.poll_interval_seconds ?? 300
  }
}

export function syncSettingsFormToUpdate(values: SyncSettingsFormValues): MailSyncSettingsUpdate {
  return {
    sync_enabled: values.sync_enabled,
    batch_size: values.batch_size,
    poll_interval_seconds: values.poll_interval_seconds
  }
}
```

### `frontend/src/integrations/mail/queries/accountSetupQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/queries/accountSetupQueries.ts`
- Size bytes / Размер в байтах: `688`
- Included characters / Включено символов: `688`
- Truncated / Обрезано: `no`

```typescript
import { useMutation } from '@tanstack/vue-query'
import {
  setupImapEmailAccount,
  startGmailOAuthSetup,
  type EmailAccountSetupResponse,
  type GmailOAuthStartRequest,
  type GmailOAuthStartResponse,
  type ImapEmailAccountSetupRequest
} from '../api/accountSetup'

export function useStartGmailOAuthSetupMutation() {
  return useMutation<GmailOAuthStartResponse, Error, GmailOAuthStartRequest>({
    mutationFn: async (request) => startGmailOAuthSetup(request)
  })
}

export function useSetupImapEmailAccountMutation() {
  return useMutation<EmailAccountSetupResponse, Error, ImapEmailAccountSetupRequest>({
    mutationFn: async (request) => setupImapEmailAccount(request)
  })
}
```

### `frontend/src/integrations/mail/queries/runtimeQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/mail/queries/runtimeQueries.ts`
- Size bytes / Размер в байтах: `206`
- Included characters / Включено символов: `206`
- Truncated / Обрезано: `no`

```typescript
export {
  useMailSyncSettingsQuery,
  useUpdateMailSyncSettingsMutation
} from '../../../shared/mailSync/runtimeQueries'
export { useRunMailSyncNowMutation } from '../../../shared/mailSync/runtimeQueries'
```

### `frontend/src/integrations/telegram/api/telegram.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegram.ts`
- Size bytes / Размер в байтах: `14091`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramCapabilitiesResponse,
  TelegramCallTranscriptResponse,
  TelegramCallListResponse,
  TelegramChatGroupFilterListResponse,
  TelegramChatMembersSyncResponse,
  TelegramChatActionRequest,
  TelegramChatActionResponse,
  TelegramChatLifecycleCommandResponse,
  TelegramChatFolderReassignRequest,
  TelegramChatFolderReassignResponse,
  TelegramRuntimeStatus,
  TelegramAccountListResponse,
  TelegramAccountLifecycleResponse,
  TelegramChatSyncRequest,
  TelegramChatSyncResponse,
  TelegramHistorySyncRequest,
  TelegramHistorySyncResponse,
  TelegramQrLoginStatusResponse,
  TelegramQrLoginStartRequest,
  TelegramQrLoginPasswordRequest,
  TelegramAccountSetupResponse,
  TelegramSendDryRunResponse,
  TelegramMessageIngestResponse,
  TelegramMediaDownloadRequest,
  TelegramMediaDownloadResponse,
  TelegramRuntimeRestartRequest,
  TelegramRuntimeStartRequest,
  TelegramRuntimeStopRequest,
} from '../types/telegram'

// --- Capabilities ---
export async function fetchTelegramCapabilities(): Promise<TelegramCapabilitiesResponse> {
  return ApiClient.instance.get<TelegramCapabilitiesResponse>(
    '/api/v1/integrations/telegram/capabilities',
    'Telegram capabilities request failed'
  )
}

export async function fetchTelegramAccountCapabilities(
  accountId: string
): Promise<TelegramCapabilitiesResponse> {
  return ApiClient.instance.get<TelegramCapabilitiesResponse>(
    `/api/v1/integrations/telegram/accounts/${encodeURIComponent(accountId)}/capabilities`,
    'Telegram account capabilities request failed'
  )
}

// --- Accounts ---
export async function fetchTelegramAccounts(query?: string): Promise<TelegramAccountListResponse> {
  const qs = query?.trim() ? `?${query}` : ''
  return ApiClient.instance.get<TelegramAccountListResponse>(
    `/api/v1/integrations/telegram/accounts${qs}`,
    'Telegram account list request failed'
  )
}

export async function setupTelegramAccount(request: {
  account_id: string
  provider_kind: string
  display_name: string
  external_account_id: string
  api_id?: number
  api_hash?: string
  bot_token?: string
  session_encryption_key?: string
  tdlib_data_path?: string
  qr_authorized?: boolean
  transcription_enabled: boolean
}): Promise<TelegramAccountSetupResponse> {
  return ApiClient.instance.post<TelegramAccountSetupResponse>(
    '/api/v1/integrations/telegram/accounts',
    request,
    'Telegram account setup failed'
  )
}

export async function removeTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
  return ApiClient.instance.delete<TelegramAccountLifecycleResponse>(
    `/api/v1/integrations/telegram/accounts/${encodeURIComponent(accountId)}`,
    'Telegram account remove failed'
  )
}

export async function logoutTelegramAccount(accountId: string): Promise<TelegramAccountLifecycleResponse> {
  return ApiClient.instance.post<TelegramAccountLifecycleResponse>(
    `/api/v1/integrations/telegram/accounts/${encodeURIComponent(accountId)}/logout`,
    {},
    'Telegram account logout failed'
  )
}

export async function fetchTelegramFolders(accountId?: string): Promise<TelegramChatGroupFilterListResponse> {
  const params = new URLSearchParams()
  if (accountId?.trim()) {
    params.set('account_id', accountId.trim())
  }
  const suffix = params.size ? `?${params.toString()}` : ''
  return ApiClient.instance.get<TelegramChatGroupFilterListResponse>(
    `/api/v1/integrations/telegram/conversation-folders${suffix}`,
    'Telegram folders request failed'
  )
}

export async function syncTelegramChatMembers(telegramChatId: string): Promise<TelegramChatMembersSyncResponse> {
  return ApiClient.instance.post<TelegramChatMembersSyncResponse>(
    `/api/v1/integrations/telegram/provider-sync/conversations/${encodeURIComponent(telegramChatId)}/members`,
    {},
    'Telegram chat members sync failed'
  )
}

export async function syncTelegramChats(request: TelegramChatSyncRequest): Promise<TelegramChatSyncResponse> {
  return ApiClient.instance.post<TelegramChatSyncResponse>(
    '/api/v1/integrations/telegram/provider-sync/chats',
    request,
    'Telegram chat sync failed'
  )
}

export async function pinTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/pin`,
    request,
    'Telegram chat pin failed'
  )
}

export async function unpinTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unpin`,
    request,
    'Telegram chat unpin failed'
  )
}

export async function archiveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/archive`,
    request,
    'Telegram chat archive failed'
  )
}

export async function unarchiveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unarchive`,
    request,
    'Telegram chat unarchive failed'
  )
}

export async function muteTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/mute`,
    request,
    'Telegram chat mute failed'
  )
}

export async function addTelegramChatToFolder(
  telegramChatId: string,
  providerFolderId: number,
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/folders/${providerFolderId}`,
    request,
    'Telegram chat folder add failed'
  )
}

export async function removeTelegramChatFromFolder(
  telegramChatId: string,
  providerFolderId: number,
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/folders/${providerFolderId}/remove`,
    request,
    'Telegram chat folder remove failed'
  )
}

export async function reassignTelegramChatFolders(
  telegramChatId: string,
  request: TelegramChatFolderReassignRequest
): Promise<TelegramChatFolderReassignResponse> {
  return ApiClient.instance.post<TelegramChatFolderReassignResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/folders/reassign`,
    request,
    'Telegram chat folder reassignment failed'
  )
}

export async function unmuteTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unmute`,
    request,
    'Telegram chat unmute failed'
  )
}

export async function markTelegramChatRead(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/read`,
    request,
    'Telegram chat mark read failed'
  )
}

export async function markTelegramChatUnread(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatActionResponse> {
  return ApiClient.instance.post<TelegramChatActionResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/unread`,
    request,
    'Telegram chat mark unread failed'
  )
}

export async function joinTelegramChat(
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    '/api/v1/integrations/telegram/provider-commands/conversations/join',
    request,
    'Telegram chat join failed'
  )
}

export async function leaveTelegramChat(
  telegramChatId: string,
  request: TelegramChatActionRequest
): Promise<TelegramChatLifecycleCommandResponse> {
  return ApiClient.instance.post<TelegramChatLifecycleCommandResponse>(
    `/api/v1/integrations/telegram/provider-commands/conversations/${encodeURIComponent(telegramChatId)}/leave`,
    request,
    'Telegram chat leave failed'
  )
}

export async function syncTelegramHistory(request: TelegramHistorySyncRequest): Promise<TelegramHistorySyncResponse> {
  return ApiClient.instance.post<TelegramHistorySyncResponse>(
    '/api/v1/integrations/telegram/provider-sync/history',
    request,
    'Telegram history sync failed'
  )
}

// --- Runtime ---
export async function fetchTelegramRuntimeStatus(accountId: string): Promise<TelegramRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<TelegramRuntimeStatus>(
    `/api/v1/integrations/telegram/runtime/status?${params.toString()}`,
    'Telegram runtime status request failed'
  )
}

export async function startTelegramRuntime(request: TelegramRuntimeStartRequest): Promise<TelegramRuntimeStatus> {
  return ApiClient.instance.post<TelegramRuntimeStatus>(
    '/api/v1/integrations/telegram/runtime/start',
    request,
    'Telegram runtime start failed'
  )
}

export async function stopTelegramRuntime(request: TelegramRuntimeStopRequest): Promise<TelegramRuntimeStatus> {
  return ApiClient.instance.post<TelegramRuntimeStatus>(
    '/api/v1/integrations/telegram/runtime/stop',
    request,
    'Telegram runtime stop failed'
  )
}

export async function restartTelegramRuntime(request: TelegramRuntimeRestartRequest): Promise<TelegramRuntimeStatus> {
  return ApiClient.instance.post<TelegramRuntimeStatus>(
    '/api/v1/integrations/telegram/runtime/restart',
    request,
    'Telegram runtime restart failed'
  )
}

// --- Media ---
export async function downloadTelegramMedia(
  request: TelegramMediaDownloadRequest
): Promise<TelegramMediaDownloadResponse> {
  return ApiClient.instance.post<TelegramMediaDownloadResponse>(
    '/api/v1/integrations/telegram/provider-media/download',
    request,
    'Telegram media download failed'
  )
}

export async function sendTelegramDryRun(request: {
  account_id: string
  provider_chat_id: string
  text: string
}): Promise<TelegramSendDryRunResponse> {
  return ApiClient.instance.post<TelegramSendDryRunResponse>(
    '/api/v1/policies/telegram-send/dry-run',
    request,
    'Telegram send dry-run failed'
  )
}

export async function ingestTelegramFixtureMessage(request: {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  chat_kind: string
  chat_title: string
  sender_id: string
  sender_display_name: string
  text: string
  import_batch_id: string
  occurred_at: string
  delivery_state: string
}): Promise<TelegramMessageIngestResponse> {
  return ApiClient.instance.post<TelegramMessageIngestResponse>(
    '/api/v1/integrations/telegram/fixtures/messages',
    request,
    'Telegram fixture message ingest failed'
  )
}

// --- Q
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/telegram/api/telegramAutomation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramAutomation.test.ts`
- Size bytes / Размер в байтах: `2584`
- Included characters / Включено символов: `2584`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchTelegramAutomationPolicies,
  fetchTelegramAutomationTemplates,
  runTelegramSendDryRun,
} from './telegramAutomation'

describe('telegram automation API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('loads policies, templates and send dry-run routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [{ policy_id: 'pol-1', template_id: 'tpl-1', name: 'Follow up', enabled: true, account_id: 'acc-1', allowed_chat_ids: ['chat-1'], trigger_kind: 'ai_follow_up', max_sends_per_hour: 3, quiet_hours: {}, expires_at: null, conditions: {}, created_at: '2026-06-16T12:00:00Z', updated_at: '2026-06-16T12:00:00Z' }] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [{ template_id: 'tpl-1', name: 'Follow up template', body_template: 'Hello {{name}}', required_variables: ['name'], created_at: '2026-06-16T12:00:00Z', updated_at: '2026-06-16T12:00:00Z' }] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ outbound_message_id: 'out-1', policy_id: 'pol-1', template_id: 'tpl-1', account_id: 'acc-1', provider_chat_id: 'chat-1', rendered_text: 'Hello Maria', rendered_preview_hash: 'sha256:abc', status: 'allowed', event_id: 'evt-1' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramAutomationPolicies()
    await fetchTelegramAutomationTemplates()
    await runTelegramSendDryRun({
      command_id: 'cmd-1',
      policy_id: 'pol-1',
      provider_chat_id: 'chat-1',
      variables: { name: 'Maria' },
      source_context: { source: 'telegram_workbench' },
    })

    expect(fetchMock).toHaveBeenCalledTimes(3)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/policies')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/policies/templates')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/policies/telegram-send/dry-run')
    expect(fetchMock.mock.calls[2][1].method).toBe('POST')
  })
})
```

### `frontend/src/integrations/telegram/api/telegramAutomation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramAutomation.ts`
- Size bytes / Размер в байтах: `1057`
- Included characters / Включено символов: `1057`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramAutomationPolicyListResponse,
  TelegramAutomationTemplateListResponse,
  TelegramSendDryRunRequest,
  TelegramSendDryRunResponse,
} from '../types/automation'

export async function fetchTelegramAutomationPolicies(): Promise<TelegramAutomationPolicyListResponse> {
  return ApiClient.instance.get<TelegramAutomationPolicyListResponse>(
    '/api/v1/policies',
    'Telegram automation policy request failed'
  )
}

export async function fetchTelegramAutomationTemplates(): Promise<TelegramAutomationTemplateListResponse> {
  return ApiClient.instance.get<TelegramAutomationTemplateListResponse>(
    '/api/v1/policies/templates',
    'Telegram automation template request failed'
  )
}

export async function runTelegramSendDryRun(
  request: TelegramSendDryRunRequest
): Promise<TelegramSendDryRunResponse> {
  return ApiClient.instance.post<TelegramSendDryRunResponse>(
    '/api/v1/policies/telegram-send/dry-run',
    request,
    'Telegram send dry-run failed'
  )
}
```

### `frontend/src/integrations/telegram/api/telegramDialogs.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramDialogs.test.ts`
- Size bytes / Размер в байтах: `17153`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  addTelegramChatToFolder,
  archiveTelegramChat,
  fetchTelegramCalls,
  fetchTelegramCallTranscript,
  fetchTelegramAccountCapabilities,
  fetchTelegramFolders,
  joinTelegramChat,
  leaveTelegramChat,
  logoutTelegramAccount,
  markTelegramChatRead,
  markTelegramChatUnread,
  muteTelegramChat,
  pinTelegramChat,
  reassignTelegramChatFolders,
  removeTelegramAccount,
  removeTelegramChatFromFolder,
  restartTelegramRuntime,
  setupTelegramAccount,
  stopTelegramRuntime,
  syncTelegramChatMembers,
  unarchiveTelegramChat,
  unmuteTelegramChat,
  unpinTelegramChat,
} from './telegram'

describe('telegram dialog action API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  it('posts runtime restart requests for a selected telegram account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ account_id: 'acc-1', runtime_kind: 'fixture', status: 'running', blockers: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await restartTelegramRuntime({ account_id: 'acc-1' })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/runtime/restart')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({ account_id: 'acc-1' })
  })

  it('posts runtime stop requests for a selected telegram account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ account_id: 'acc-1', runtime_kind: 'fixture', status: 'stopped', blockers: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await stopTelegramRuntime({ account_id: 'acc-1' })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/runtime/stop')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toEqual({ account_id: 'acc-1' })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('posts member sync through provider-sync route', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', synced_count: 0, items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await syncTelegramChatMembers('tgchat-1')

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-sync/conversations/tgchat-1/members')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
  })

  it('loads projection-backed telegram folders for the selected account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [{ id: 'local:all', label: 'All', source: 'local', count: 2, icon: 'tabler:message' }] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramFolders('acc-1')

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/conversation-folders?account_id=acc-1')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
  })

  it('loads account-scoped capability routes for a selected telegram account', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ version: '2.0', runtime_mode: 'fixture', capabilities: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramAccountCapabilities('acc-1')

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/accounts/acc-1/capabilities')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
  })

  it('loads projected call metadata and transcript routes', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ items: [{ call_id: 'call-1', account_id: 'acc-1', provider_chat_id: 'chat-1', status: 'ended', occurred_at: '2026-06-06T12:20:00Z' }] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ transcript: { transcript_id: 'tx-1', call_id: 'call-1', account_id: 'acc-1', provider_chat_id: 'chat-1', transcript_status: 'succeeded', stt_provider: 'fixture-stt', source_audio_ref: 'audio.wav', language_code: 'en', transcript_text: 'Follow up on the Telegram call.', segments: [], provenance: {}, created_at: '2026-06-06T12:21:00Z', updated_at: '2026-06-06T12:21:00Z' } }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramCalls('acc-1', 10)
    await fetchTelegramCallTranscript('call-1')

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/calls?limit=10&account_id=acc-1')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/calls/call-1/transcript')
    expect(fetchMock.mock.calls[0][1].method).toBe('GET')
    expect(fetchMock.mock.calls[1][1].method).toBe('GET')
  })

  it('posts account setup and lifecycle routes for telegram accounts', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ account_id: 'acc-1', runtime: 'live_blocked', provider_kind: 'telegram_user', transcription_enabled: false, credential_bindings: [] }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ account: { account_id: 'acc-1' }, stopped_runtime_actor: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ account: { account_id: 'acc-1' }, stopped_runtime_actor: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await setupTelegramAccount({
      account_id: 'acc-1',
      provider_kind: 'telegram_user',
      display_name: 'Account One',
      external_account_id: 'telegram:1',
      tdlib_data_path: '/tmp/telegram-1',
      qr_authorized: true,
      transcription_enabled: false,
    })
    await logoutTelegramAccount('acc-1')
    await removeTelegramAccount('acc-1')

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/accounts')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/telegram/accounts/acc-1/logout')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/telegram/accounts/acc-1')
    expect(fetchMock.mock.calls[0][1].method).toBe('POST')
    expect(fetchMock.mock.calls[1][1].method).toBe('POST')
    expect(fetchMock.mock.calls[2][1].method).toBe('DELETE')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body as string)).toMatchObject({
      account_id: 'acc-1',
      provider_kind: 'telegram_user',
      qr_authorized: true,
    })
  })

  it('posts pin and unpin requests for projected Telegram chats', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'pin', status: 'pinned', metadata: {} }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'unpin', status: 'unpinned', metadata: {} }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await pinTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await unpinTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })

    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-commands/conversations/tgchat-1/pin')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/telegram/provider-commands/conversations/tgchat-1/unpin')
  })

  it('posts archive/mute dialog lifecycle requests', async () => {
    const fetchMock = vi
      .fn()
      .mockImplementation(() =>
        new Response(JSON.stringify({ telegram_chat_id: 'tgchat-1', action: 'ok', status: 'ok', metadata: {} }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await archiveTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await unarchiveTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await muteTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })
    await unmuteTelegramChat('tgchat-1', { account_id: 'acc-1', provider_chat_id: 'provider-chat-1' })

    expect(fetchMock).toHaveBeenCalledTimes(4)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-commands/conversations/tgchat-1/archive')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/telegram/provider-commands/conversations/tgchat-1/unarchive')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/telegram/provider-commands/conversations/tgchat-1/mute')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/telegram/provider-commands/conversations/tgchat-1/unmute')
    for (const [, init] of fetchMock.mock.calls) {
      expect(init.method).toBe('POST')
      expect(JSON.parse(init.body as string)).toEqual({
        account_id: 'acc-1',
        provider_chat_id: 'provider-chat-1',
      })
    }
  })

  it('posts add-to-folder dialog lifecycle requests', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        new Response(JSON.stringify({
          telegram_chat_id: 'tgchat-1',
          provider_chat_id: 'provider-chat-1',
          action: 'folder_add',
          status: 'queued',
          command_id: 'cmd-folder-add',
        }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      )
    vi.stubGlobal('fetch', fetchMock)

    await addTelegramChatToFolder('tgchat-1', 7, {
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
    })

    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/provider-commands/conversations/tgchat-1/folders/7')
    expect(fetchMock.mock.calls[0][1]?.method).toBe('POST')
    expect(JSON.parse(fetchMock.mock.calls[0][1]?.body as string)).toEqual({
      account_id: 'acc-1',
      provider_chat_id: 'provider-chat-1',
    })
  })

  it('posts remove-from-folder dialog lifecycle requests', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(
        new Response(JSON.stringify({
          telegram_chat_id: 'tgchat-1',
          provider_chat_id: 'provider-chat-1',
          action: 'folder_remove',

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/telegram/api/telegramLifecycle.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramLifecycle.test.ts`
- Size bytes / Размер в байтах: `2007`
- Included characters / Включено символов: `2007`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  fetchTelegramCommands,
  retryTelegramCommand,
} from './telegramLifecycle'

describe('telegram lifecycle reference API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('fetches account command rows', async () => {
    const fetchMock = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ items: [] }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchTelegramCommands('acct-1', 25, {
      providerChatId: 'chat-42',
      providerMessageId: 'chat-42:77',
      commandKinds: ['mark_read', 'mark_unread'],
    })

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/commands?account_id=acct-1&limit=25')
    expect(fetchMock.mock.calls[0][0]).toContain('provider_chat_id=chat-42')
    expect(fetchMock.mock.calls[0][0]).toContain('provider_message_id=chat-42%3A77')
    expect(fetchMock.mock.calls[0][0]).toContain('command_kinds=mark_read%2Cmark_unread')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('sends manual retry through the provider command outbox endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ command_id: 'cmd-retry-1', status: 'retrying' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await retryTelegramCommand('cmd-retry-1')

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/telegram/commands/cmd-retry-1/retry')
    const [, init] = fetchMock.mock.calls[0]
    expect(init?.method).toBe('POST')
  })

})
```

### `frontend/src/integrations/telegram/api/telegramLifecycle.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/telegram/api/telegramLifecycle.ts`
- Size bytes / Размер в байтах: `1425`
- Included characters / Включено символов: `1425`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  TelegramCommandListResponse,
  TelegramProviderWriteCommand,
} from '../types/telegram'

export async function fetchTelegramCommands(
  accountId: string,
  limit = 50,
  options?: {
    providerChatId?: string | null
    providerMessageId?: string | null
    commandKinds?: string[]
  }
): Promise<TelegramCommandListResponse> {
  const params = new URLSearchParams({ account_id: accountId, limit: String(limit) })
  if (options?.providerChatId?.trim()) {
    params.set('provider_chat_id', options.providerChatId.trim())
  }
  if (options?.providerMessageId?.trim()) {
    params.set('provider_message_id', options.providerMessageId.trim())
  }
  const commandKinds = (options?.commandKinds ?? [])
    .map((value) => value.trim())
    .filter((value) => value.length > 0)
  if (commandKinds.length > 0) {
    params.set('command_kinds', commandKinds.join(','))
  }
  return ApiClient.instance.get<TelegramCommandListResponse>(
    `/api/v1/integrations/telegram/commands?${params.toString()}`,
    'Telegram commands request failed'
  )
}

export async function retryTelegramCommand(
  commandId: string
): Promise<TelegramProviderWriteCommand> {
  return ApiClient.instance.post<TelegramProviderWriteCommand>(
    `/api/v1/integrations/telegram/commands/${encodeURIComponent(commandId)}/retry`,
    {},
    'Telegram command retry failed'
  )
}
```

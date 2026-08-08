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

- Chunk ID / ID чанка: `157-source-frontend-part-017`
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

### `frontend/src/shared/mailSync/runtimeQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/mailSync/runtimeQueries.ts`
- Size bytes / Размер в байтах: `3048`
- Included characters / Включено символов: `3048`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  fetchMailSyncSettings,
  fetchMailSyncStatus,
  runMailFullResync,
  runMailSyncNow,
  updateMailSyncSettings
} from './syncApi'
import type {
  MailSyncRunResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncStatus
} from './types'

type NullableQueryParam<T> = T | null | undefined | (() => T | null | undefined)

export function useSyncStatusesQuery() {
  return useQuery<MailSyncStatus[]>({
    queryKey: ['communications', 'mail', 'sync-statuses'],
    queryFn: async () => {
      const res = await fetchMailSyncStatus()
      return res.items
    }
  })
}

export function useMailSyncSettingsQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailSyncSettings | null>({
    queryKey: computed(() => {
      const id = toValue(accountId)
      return id
        ? (['communications', 'mail', 'sync-settings', id] as const)
        : (['communications', 'mail', 'sync-settings', null] as const)
    }),
    queryFn: async () => {
      const id = toValue(accountId)
      if (!id) return null
      return fetchMailSyncSettings(id)
    },
    enabled: computed(() => Boolean(toValue(accountId)))
  })
}

export function useUpdateMailSyncSettingsMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailSyncSettings,
    Error,
    { accountId: string; settings: MailSyncSettingsUpdate }
  >({
    mutationFn: async ({ accountId, settings }) => updateMailSyncSettings(accountId, settings),
    onSuccess: (_settings, variables) => {
      queryClient.invalidateQueries({
        queryKey: ['communications', 'mail', 'sync-settings', variables.accountId]
      })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
    }
  })
}

export function useRunMailSyncNowMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailSyncNow(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'mailbox-health'] })
    }
  })
}

export function useRunMailFullResyncMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailFullResync(accountId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['communications-list'] })
      queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'mailbox-health'] })
    }
  })
}
```

### `frontend/src/shared/mailSync/syncApi.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/mailSync/syncApi.ts`
- Size bytes / Размер в байтах: `1605`
- Included characters / Включено символов: `1605`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../platform/api/ApiClient'
import type {
  MailSyncRunResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncStatusListResponse
} from './types'

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

### `frontend/src/shared/mailSync/syncSettingsForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/mailSync/syncSettingsForm.ts`
- Size bytes / Размер в байтах: `1318`
- Included characters / Включено символов: `1318`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type {
  MailSyncSettings,
  MailSyncSettingsUpdate
} from './types'

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

### `frontend/src/shared/mailSync/types.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/mailSync/types.ts`
- Size bytes / Размер в байтах: `1621`
- Included characters / Включено символов: `1621`
- Truncated / Обрезано: `no`

```typescript
export type MailSyncSettings = {
  account_id: string
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
  updated_at: string
}

export type MailSyncSettingsUpdate = {
  sync_enabled: boolean
  batch_size: number
  poll_interval_seconds: number
}

export type MailSyncStatus = {
  account_id: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  last_started_at: string | null
  last_completed_at: string | null
  next_run_at: string | null
  last_error_code: string | null
  last_error_message: string | null
  last_fetched_messages: number
  last_projected_messages: number
  last_upserted_persons: number
  last_upserted_organizations: number
}

export type MailSyncStatusListResponse = {
  items: MailSyncStatus[]
}

export type MailSyncRunResponse = {
  run_id: string
  account_id: string
  trigger: string
  status: string
  phase: string
  progress_mode: 'none' | 'determinate' | 'indeterminate' | string
  progress_percent: number | null
  processed_messages: number
  estimated_total_messages: number | null
  current_batch_size: number
  fetched_messages: number
  projected_messages: number
  upserted_persons: number
  upserted_organizations: number
  checkpoint_before_present: boolean
  checkpoint_after_present: boolean
  checkpoint_saved: boolean
  failure_reason: { code: string; message: string } | null
  started_at: string
  completed_at: string | null
  next_run_at: string | null
}
```

### `frontend/src/shared/sanitize/emailHtml.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/sanitize/emailHtml.boundary.test.ts`
- Size bytes / Размер в байтах: `623`
- Included characters / Включено символов: `623`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('emailHtml remote image privacy boundary', () => {
  it('exposes pure helpers for blocking and proxying remote images', () => {
    const source = readFileSync(new URL('./emailHtml.ts', import.meta.url), 'utf8')

    expect(source).toContain('remoteImageUrlsFromHtml')
    expect(source).toContain('rewriteRemoteImageSources')
    expect(source).toContain('data-makosh-remote-src')
    expect(source).toContain('isRemoteImageUrl')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/shared/sanitize/emailHtml.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/sanitize/emailHtml.ts`
- Size bytes / Размер в байтах: `8030`
- Included characters / Включено символов: `8030`
- Truncated / Обрезано: `no`

```typescript
export type RenderedMessageBody = {
	kind: 'html' | 'plain'
	html: string
}

type AttributePolicy = {
	readonly allowed: ReadonlySet<string>
	readonly urlAttributes?: ReadonlySet<string>
}

const VOID_TAGS = new Set(['br', 'img'])

const TAG_RENAMES = new Map<string, string>([
	['b', 'strong'],
	['i', 'em'],
	['font', 'span']
])

const ALLOWED_TAGS = new Set([
	'a',
	'blockquote',
	'br',
	'code',
	'div',
	'em',
	'img',
	'li',
	'ol',
	'p',
	'pre',
	's',
	'span',
	'strong',
	'table',
	'tbody',
	'td',
	'tfoot',
	'th',
	'thead',
	'tr',
	'u',
	'ul'
])

const GLOBAL_ATTRIBUTES: AttributePolicy = {
	allowed: new Set(['title'])
}

const ATTRIBUTE_POLICIES = new Map<string, AttributePolicy>([
	[
		'a',
		{
			allowed: new Set(['href', 'title']),
			urlAttributes: new Set(['href'])
		}
	],
	[
		'img',
		{
			allowed: new Set(['alt', 'height', 'src', 'title', 'width']),
			urlAttributes: new Set(['src'])
		}
	],
	['td', { allowed: new Set(['colspan', 'rowspan']) }],
	['th', { allowed: new Set(['colspan', 'rowspan']) }]
])

const BLOCKED_CONTAINER_TAGS = [
	'script',
	'style',
	'iframe',
	'object',
	'embed',
	'svg',
	'math',
	'form',
	'head',
	'noscript',
	'template'
]

const TAG_TOKEN_PATTERN = /<!--[\s\S]*?-->|<![^>]*>|<\/?[A-Za-z][^>]*>/g
const ATTRIBUTE_PATTERN = /([^\s"'<>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g
const BLOCKED_REMOTE_IMAGE_SRC = 'data:image/gif;base64,R0lGODlhAQABAAAAACw='

export function renderMessageBody(input: { bodyHtml: string | null | undefined; bodyText: string }): RenderedMessageBody {
	if (input.bodyHtml?.trim()) {
		return {
			kind: 'html',
			html: sanitizeEmailHtml(input.bodyHtml)
		}
	}

	return {
		kind: 'plain',
		html: normalizePlainText(input.bodyText)
	}
}

export function remoteImageUrlsFromHtml(html: string): string[] {
	const urls = new Set<string>()
	const sanitized = sanitizeEmailHtml(html)

	for (const match of sanitized.matchAll(/<img\b[^>]*\bsrc="([^"]*)"[^>]*>/gi)) {
		const decoded = decodeHtmlEntities(match[1] ?? '')
		if (isRemoteImageUrl(decoded)) {
			urls.add(decoded)
		}
	}

	return Array.from(urls)
}

export function rewriteRemoteImageSources(
	html: string,
	remoteSource: (url: string) => string | null
): string {
	return html.replace(/<img\b[^>]*\bsrc="([^"]*)"[^>]*>/gi, (tag, rawSrc: string) => {
		const decoded = decodeHtmlEntities(rawSrc)
		if (!isRemoteImageUrl(decoded)) {
			return tag
		}

		const replacement = remoteSource(decoded)
		if (!replacement) {
			return tag.replace(
				/\bsrc="[^"]*"/i,
				`src="${BLOCKED_REMOTE_IMAGE_SRC}" data-makosh-remote-src="${escapeAttribute(decoded)}" aria-label="Remote image blocked"`
			)
		}

		return tag.replace(/\bsrc="[^"]*"/i, `src="${escapeAttribute(replacement)}"`)
	})
}

export function sanitizeEmailHtml(html: string): string {
	const withoutBlockedContainers = stripBlockedContainers(html)
	let sanitized = ''
	let cursor = 0
	const openTags: string[] = []

	for (const match of withoutBlockedContainers.matchAll(TAG_TOKEN_PATTERN)) {
		const token = match[0]
		const index = match.index ?? 0

		sanitized += escapeHtml(withoutBlockedContainers.slice(cursor, index))
		cursor = index + token.length

		if (token.startsWith('<!--') || token.startsWith('<!')) {
			continue
		}

		const tag = parseTagToken(token)
		if (!tag) {
			continue
		}

		if (tag.closing) {
			if (VOID_TAGS.has(tag.name)) {
				continue
			}

			const openIndex = openTags.lastIndexOf(tag.name)
			if (openIndex === -1) {
				continue
			}

			for (let i = openTags.length - 1; i >= openIndex; i -= 1) {
				sanitized += `</${openTags[i]}>`
				openTags.pop()
			}
			continue
		}

		if (!ALLOWED_TAGS.has(tag.name)) {
			continue
		}

		const attributes = sanitizeAttributes(tag.name, tag.attributesSource)
		sanitized += `<${tag.name}${attributes}>`

		if (!tag.selfClosing && !VOID_TAGS.has(tag.name)) {
			openTags.push(tag.name)
		}
	}

	sanitized += escapeHtml(withoutBlockedContainers.slice(cursor))

	for (let i = openTags.length - 1; i >= 0; i -= 1) {
		sanitized += `</${openTags[i]}>`
	}

	return sanitized
}

export function normalizePlainText(text: string): string {
	return escapeHtml(text).replace(/\r\n|\r|\n/g, '<br>')
}

function stripBlockedContainers(html: string): string {
	let stripped = html

	for (const tagName of BLOCKED_CONTAINER_TAGS) {
		stripped = stripped.replace(new RegExp(`<${tagName}\\b[^>]*>[\\s\\S]*?<\\/${tagName}>`, 'gi'), '')
		stripped = stripped.replace(new RegExp(`<${tagName}\\b[^>]*\\/?>`, 'gi'), '')
		stripped = stripped.replace(new RegExp(`<\\/${tagName}>`, 'gi'), '')
	}

	stripped = stripped.replace(/<meta\b[^>]*\/?>/gi, '')
	stripped = stripped.replace(/<link\b[^>]*\/?>/gi, '')

	return stripped
}

function parseTagToken(token: string): { name: string; closing: boolean; selfClosing: boolean; attributesSource: string } | null {
	const match = token.match(/^<\s*(\/?)\s*([A-Za-z][A-Za-z0-9:-]*)([\s\S]*?)(\/?)\s*>$/)
	if (!match) return null

	const sourceName = match[2].toLowerCase()
	const name = TAG_RENAMES.get(sourceName) ?? sourceName

	return {
		name,
		closing: match[1] === '/',
		selfClosing: match[4] === '/',
		attributesSource: match[3] ?? ''
	}
}

function sanitizeAttributes(tagName: string, source: string): string {
	const policy = ATTRIBUTE_POLICIES.get(tagName) ?? GLOBAL_ATTRIBUTES
	const attributes: string[] = []

	for (const match of source.matchAll(ATTRIBUTE_PATTERN)) {
		const rawName = match[1].toLowerCase()
		if (rawName.startsWith('on') || rawName === 'style' || !policy.allowed.has(rawName)) {
			continue
		}

		const rawValue = match[2] ?? match[3] ?? match[4] ?? ''
		if (policy.urlAttributes?.has(rawName) && !isSafeUrl(tagName, rawName, rawValue)) {
			continue
		}

		if (['colspan', 'rowspan', 'height', 'width'].includes(rawName) && !isSafeIntegerAttribute(rawValue)) {
			continue
		}

		attributes.push(`${rawName}="${escapeAttribute(rawValue)}"`)
	}

	if (tagName === 'a' && attributes.some((attribute) => attribute.startsWith('href='))) {
		attributes.push('target="_blank"', 'rel="noreferrer noopener"')
	}

	return attributes.length ? ` ${attributes.join(' ')}` : ''
}

function isRemoteImageUrl(value: string): boolean {
	const normalized = value.trim().toLowerCase()
	return normalized.startsWith('http://') || normalized.startsWith('https://')
}

function isSafeUrl(tagName: string, attributeName: string, rawValue: string): boolean {
	const value = stripUnsafeUrlCharacters(decodeHtmlEntities(rawValue).trim())
	const lowerValue = value.toLowerCase()

	if (!lowerValue) {
		return false
	}

	if (attributeName === 'src' && tagName === 'img' && lowerValue.startsWith('cid:')) {
		return true
	}

	if (attributeName === 'src' && tagName === 'img') {
		return lowerValue.startsWith('https://') || lowerValue.startsWith('http://')
	}

	return lowerValue.startsWith('https://') || lowerValue.startsWith('http://') || lowerValue.startsWith('mailto:')
}

function stripUnsafeUrlCharacters(value: string): string {
	return Array.from(value)
		.filter((char) => {
			const code = char.charCodeAt(0)
			return code > 31 && code !== 127 && !/\s/.test(char)
		})
		.join('')
}

function isSafeIntegerAttribute(value: string): boolean {
	return /^\d{1,4}$/.test(value.trim())
}

function escapeHtml(value: string): string {
	return value
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
}

function escapeAttribute(value: string): string {
	return escapeHtml(value).replace(/"/g, '&quot;')
}

function decodeHtmlEntities(value: string): string {
	return value
		.replace(/&#x([0-9a-f]+);?/gi, (_, hex: string) => decodeCodePoint(Number.parseInt(hex, 16)))
		.replace(/&#([0-9]+);?/g, (_, decimal: string) => decodeCodePoint(Number.parseInt(decimal, 10)))
		.replace(/&colon;?/gi, ':')
		.replace(/&tab;?/gi, '\t')
		.replace(/&newline;?/gi, '\n')
		.replace(/&amp;?/gi, '&')
}

function decodeCodePoint(codePoint: number): string {
	if (!Number.isInteger(codePoint) || codePoint < 0 || codePoint > 0x10ffff) {
		return ''
	}

	return String.fromCodePoint(codePoint)
}
```

### `frontend/src/shared/stores/layoutEditor.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/layoutEditor.ts`
- Size bytes / Размер в байтах: `16147`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type WidgetGridDimension = 'columns' | 'rows'
export type WidgetPanelSurfaceSetting = 'panelOpacity' | 'panelBlur'
export type ScrollMode = 'default' | 'horizontal' | 'vertical' | 'none'

export type ViewLayoutOverride = {
  hiddenWidgetIds: string[]
  zoneOverrides: Record<string, unknown>
  orderOverrides: Record<string, unknown>
  gridOverrides: Record<string, { columns: number; rows: number }>
  panelSurfaceOverrides: Record<string, { panelOpacity: number; panelBlur: number }>
}

export type WidgetDefinition = {
  id: string
  title: string
  icon: string
  viewScope: string[]
  defaultColumns: number
  defaultRows: number
  minColumns: number
  minRows: number
  canAdd: boolean
  removable: boolean
}

export type ResolvedWidget = {
  id: string
  title: string
  icon: string
  columns: number
  rows: number
  minColumns: number
  minRows: number
  canAdd: boolean
  removable: boolean
  panelOpacity: number
  panelBlur: number
  zone: string
  order: number
}

export type ResolvedLayout = {
  viewId: string
  widgets: ResolvedWidget[]
  widgetById: Map<string, ResolvedWidget>
}

export type LayoutSettings = {
  schemaVersion: number
  views: Record<string, ViewLayoutOverride>
}

function defaultLayoutSettings(): LayoutSettings {
  return {
    schemaVersion: 2,
    views: {}
  }
}

const defaultWidgets: WidgetDefinition[] = [
  { id: 'home-welcome', title: 'Welcome', icon: 'tabler:home', viewScope: ['home'], defaultColumns: 6, defaultRows: 2, minColumns: 3, minRows: 1, canAdd: true, removable: true },
  { id: 'home-stats', title: 'Statistics', icon: 'tabler:chart-bar', viewScope: ['home'], defaultColumns: 6, defaultRows: 2, minColumns: 3, minRows: 1, canAdd: true, removable: true },
  { id: 'home-timeline', title: 'Recent Activity', icon: 'tabler:timeline-event', viewScope: ['home'], defaultColumns: 4, defaultRows: 3, minColumns: 2, minRows: 2, canAdd: true, removable: true },
  { id: 'persons-list', title: 'Personas', icon: 'tabler:user', viewScope: ['persons'], defaultColumns: 6, defaultRows: 4, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'persons-recent', title: 'Recent Persons', icon: 'tabler:user-plus', viewScope: ['persons'], defaultColumns: 6, defaultRows: 2, minColumns: 3, minRows: 1, canAdd: true, removable: true },
  { id: 'projects-list', title: 'Projects', icon: 'tabler:briefcase', viewScope: ['projects'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'tasks-list', title: 'Tasks', icon: 'tabler:checkbox', viewScope: ['tasks'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'calendar-month', title: 'Month View', icon: 'tabler:calendar', viewScope: ['calendar'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'calendar-upcoming', title: 'Upcoming', icon: 'tabler:calendar-event', viewScope: ['calendar'], defaultColumns: 4, defaultRows: 4, minColumns: 2, minRows: 2, canAdd: true, removable: true },
  { id: 'documents-list', title: 'Documents', icon: 'tabler:file-text', viewScope: ['documents'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'notes-list', title: 'Notes', icon: 'tabler:notes', viewScope: ['notes'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'knowledge-graph', title: 'Graph Explorer', icon: 'tabler:share', viewScope: ['knowledge'], defaultColumns: 12, defaultRows: 6, minColumns: 6, minRows: 3, canAdd: true, removable: true },
  { id: 'review-queue', title: 'Review Queue', icon: 'tabler:clipboard-check', viewScope: ['review'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'communications-unified', title: 'Unified Inbox', icon: 'tabler:messages', viewScope: ['communications'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true },
  { id: 'communications-mail', title: 'Mail', icon: 'tabler:mail', viewScope: ['communications'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'communications-telegram', title: 'Telegram', icon: 'tabler:brand-telegram', viewScope: ['communications', 'telegram'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'communications-whatsapp', title: 'WhatsApp', icon: 'tabler:brand-whatsapp', viewScope: ['communications', 'whatsapp'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'settings-general', title: 'General', icon: 'tabler:settings', viewScope: ['settings'], defaultColumns: 6, defaultRows: 3, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'settings-accounts', title: 'Accounts', icon: 'tabler:plug', viewScope: ['settings'], defaultColumns: 6, defaultRows: 3, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'settings-theme', title: 'Theme', icon: 'tabler:palette', viewScope: ['settings'], defaultColumns: 6, defaultRows: 3, minColumns: 3, minRows: 2, canAdd: true, removable: true },
  { id: 'agents-overview', title: 'AI Agents', icon: 'tabler:sparkles', viewScope: ['agents'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'organizations-list', title: 'Organizations', icon: 'tabler:building-community', viewScope: ['organizations'], defaultColumns: 8, defaultRows: 4, minColumns: 4, minRows: 2, canAdd: true, removable: true },
  { id: 'timeline-stream', title: 'Timeline', icon: 'tabler:timeline-event', viewScope: ['timeline'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true },
  { id: 'telegram-messages', title: 'Messages', icon: 'tabler:brand-telegram', viewScope: ['communications', 'telegram'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true },
  { id: 'whatsapp-messages', title: 'Messages', icon: 'tabler:brand-whatsapp', viewScope: ['communications', 'whatsapp'], defaultColumns: 8, defaultRows: 6, minColumns: 4, minRows: 3, canAdd: true, removable: true }
]

function getWidgetsForView(viewId: string, setting: LayoutSettings): ResolvedWidget[] {
  const override = setting.views[viewId]
  const hiddenIds = new Set(override?.hiddenWidgetIds ?? [])
  const gridOverrides = override?.gridOverrides ?? {}
  const panelOverrides = override?.panelSurfaceOverrides ?? {}

  const widgets: ResolvedWidget[] = []
  const viewWidgets = defaultWidgets.filter((w) => w.viewScope.includes(viewId))

  for (let i = 0; i < viewWidgets.length; i++) {
    const def = viewWidgets[i]
    if (hiddenIds.has(def.id)) continue

    const gridOverride = gridOverrides[def.id]

    widgets.push({
      id: def.id,
      title: def.title,
      icon: def.icon,
      columns: gridOverride?.columns ?? def.defaultColumns,
      rows: gridOverride?.rows ?? def.defaultRows,
      minColumns: def.minColumns,
      minRows: def.minRows,
      canAdd: def.canAdd,
      removable: def.removable,
      panelOpacity: panelOverrides[def.id]?.panelOpacity ?? 60,
      panelBlur: panelOverrides[def.id]?.panelBlur ?? 12,
      zone: 'main',
      order: i
    })
  }

  return widgets
}

export const useLayoutEditorStore = defineStore('layoutEditor', () => {
  const layoutSettings = ref<LayoutSettings>(defaultLayoutSettings())
  const layoutDraft = ref<LayoutSettings | null>(null)
  const isLayoutEditing = ref(false)
  const isWidgetDrawerOpen = ref(false)
  const selectedLayoutWidgetId = ref<string | null>(null)

  const effectiveLayoutSettings = computed<LayoutSettings>(() => {
    return layoutDraft.value ?? layoutSettings.value
  })

  const currentView = ref<string>('home')

  const activeWidgets = computed<ResolvedWidget[]>(() => {
    return getWidgetsForView(currentView.value, effectiveLayoutSettings.value)
  })

  const activeWidgetById = computed<Map<string, ResolvedWidget>>(() => {
    const map = new Map<string, ResolvedWidget>()
    for (const widget of activeWidgets.value) {
      map.set(widget.id, widget)
    }
    return map
  })

  const visibleWidgetIds = computed<Set<string>>(() => {
    return new Set(activeWidgets.value.map((w) => w.id))
  })

  const addableWidgetsForCurrentView = computed<WidgetDefinition[]>(() => {
    const visibleIds = visibleWidgetIds.value
    return defaultWidgets
      .filter((w) => w.viewScope.includes(currentView.value))
      .filter((w) => w.canAdd && !visibleIds.has(w.id))
  })

  function setLayoutSettings(settings: LayoutSettings): void {
    layoutSettings.value = settings
  }

  function startLayoutEditing(): void {
    layoutDraft.value = JSON.parse(JSON.stringify(layoutSettings.value))
    isLayoutEditing.value = true
  }

  function cancelLayoutEditing(): void {
    layoutDraft.value = null
    isLayoutEditing.value = false
    selectedLayoutWidgetId.value = null
  }

  function saveLayoutSettings(): void {
    if (layoutDraft.value) {
      layoutSettings.value = JSON.parse(JSON.stringify(layoutDraft.value))
      layoutDraft.value = null
    }
    isLayoutEditing.value = false
    selectedLayoutWidgetId.value = null
  }

  function openAddWidgetDrawer(): void {
    isWidgetDrawerOpen.value = true
  }

  function closeAddWidgetDrawer(): void {
    isWidgetDrawerOpen.value = false
  }

  function openWidgetSettingsDrawer(widgetId: string): void {
    selectedLayoutWidgetId.value = widgetId
  }

  function closeWidgetSettingsDrawer(): void {
    selectedLayoutWidgetId.value = null
  }

  function isWidgetVisible(widgetId: string): boolean {
    return visibleWidgetIds.value.has(widgetId)
  }

  function hideWidget(widgetId: string): void {
    updateCurrentViewOverride((override) => ({
      ...override,
      hiddenWidgetIds: [...override.hiddenWidgetIds, widgetId]
    }))
  }

  function showWidget(widgetId: string): void {
    updateCurrentViewOverride((override) => ({
      ...override,
      hiddenWidgetIds: override.hiddenWidgetIds.filter((id) => id !== widgetId)
    }))
  }

  function resetCurrentViewLayout(): void {
    updateCurrentViewOverride(() => ({
      hiddenWidgetIds: [],
      zoneOverrides: {},
      orderOverrides: {},
      gridOverrides: {},
      panelSurfaceOverrides: {}
    }))
  }

  function setWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, value: number): void {
    updateCurrentViewOverride((override) => ({
      ...override,
      gridOverrides: {
        ...override.gridOverrides,
        [widgetId]: {
          ...(override.gridOverrides[widgetId] ?? { columns: 6, rows: 4 }),
          [dimension]: value
        }
      }
    }))
  }

  function normalizeWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, value: number): number {
    const def = defaultWidgets.find((w) => w.id === widgetId)
    if (!def) return value
    const min = dimension === 'columns' ? def.minColumns : def.minRows
    const max = dimension === 'columns' ? 12 : 8
    return Math.max(min, Math.min(max, Math.round(value)))
  }

  function adjustWidgetGridValue(widgetId: string, dimension: WidgetGridDimension, delta: number): void {
    const current = activeWidgetById.value.get(widgetId)
    if (!current) return
    const currentVal = dimension === 'columns' ? current.columns : current.rows
    const newVal = normalizeWidgetGridValue(widgetId, dimension, currentVal + delta)
    setWidgetGridValue(widgetId, dimension, newVal)
  }

  function handleWidgetGridInput(widgetId: string, dimension: WidgetGridDimension, event: Event): void {
    const target = event.target as HTMLInputElement
    const value = parseInt(target.value, 10)
    if (isNaN(value)) return
    const normalized = normalizeWidgetGridValue(widgetId, dimension, value)
    setWidgetGridValue(wid
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/shared/stores/navigation.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/navigation.boundary.test.ts`
- Size bytes / Размер в байтах: `888`
- Included characters / Включено символов: `888`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('navigation communication source routing boundary', () => {
	it('routes provider communication sections through the communications route query', () => {
		const source = readFileSync(new URL('./navigation.ts', import.meta.url), 'utf8')
		const shellSource = readFileSync(
			new URL('../../app/shell/AppShell.vue', import.meta.url),
			'utf8'
		)

		expect(source).toContain("router.push({ name: 'communications', query: { section: sectionId } })")
		expect(source).toContain('communicationSectionFromQuery(sectionQuery)')
		expect(shellSource).toContain('route.query.section')
		expect(shellSource).toContain('nav.syncFromRoute(')
		expect(source).not.toContain('router.push(`/${routeViewId}`)')
		expect(source).not.toContain("type RouteViewId = AppViewId | 'telegram' | 'whatsapp'")
	})
})
```

### `frontend/src/shared/stores/navigation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/navigation.ts`
- Size bytes / Размер в байтах: `6372`
- Included characters / Включено символов: `6340`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'

export type PrimaryNavId =
  | 'home'
  | 'communications'
  | 'timeline'
  | 'persons'
  | 'projects'
  | 'tasks'
  | 'calendar'
  | 'documents'
  | 'notes'
  | 'knowledge'
  | 'review'
  | 'event-tracing'
  | 'agents'

export type CommunicationSectionId =
  | 'unified'
  | 'inbox'
  | 'waiting'
  | 'needs_reply'
  | 'mentions'
  | 'mail'
  | 'telegram'
  | 'whatsapp'
  | 'calls'
  | 'meetings'

export type SidebarViewId = PrimaryNavId | 'telegram' | 'whatsapp' | 'settings' | 'organizations'
export type AppViewId = PrimaryNavId | 'settings' | 'organizations'
type RouteViewId = AppViewId
type RouteSectionQuery = string | null | Array<string | null> | undefined

const communicationSectionIds: CommunicationSectionId[] = [
  'unified',
  'inbox',
  'waiting',
  'needs_reply',
  'mentions',
  'mail',
  'telegram',
  'whatsapp',
  'calls',
  'meetings'
]

function communicationSectionViewId(sectionId: CommunicationSectionId): SidebarViewId {
  if (sectionId === 'telegram' || sectionId === 'whatsapp') {
    return sectionId
  }
  return 'communications'
}

function communicationSectionFromQuery(section: RouteSectionQuery): CommunicationSectionId | null {
  const value = Array.isArray(section) ? section[0] : section
  if (!value) return null
  return communicationSectionIds.includes(value as CommunicationSectionId)
    ? value as CommunicationSectionId
    : null
}

type ViewCopy = {
  title: string
  subtitle: string
  search?: string
  icon: string
}

const viewCopy: Record<string, ViewCopy> = {
  home: { title: 'Home', subtitle: 'Dashboard', search: 'Search home…', icon: 'tabler:home' },
  communications: { title: 'Communications', subtitle: 'Unified inbox', search: 'Search communications…', icon: 'tabler:messages' },
  timeline: { title: 'Timeline', subtitle: 'Activity stream', search: 'Search timeline…', icon: 'tabler:timeline-event' },
  persons: { title: 'Persons', subtitle: 'Persona intelligence', search: 'Search persons…', icon: 'tabler:user' },
  projects: { title: 'Projects', subtitle: 'Projects and initiatives', search: 'Search projects…', icon: 'tabler:briefcase' },
  tasks: { title: 'Tasks', subtitle: 'Task management', search: 'Search tasks…', icon: 'tabler:checkbox' },
  calendar: { title: 'Calendar', subtitle: 'Schedule and events', search: 'Search calendar…', icon: 'tabler:calendar' },
  documents: { title: 'Documents', subtitle: 'Document management', search: 'Search documents…', icon: 'tabler:file-text' },
  notes: { title: 'Notes', subtitle: 'Notes and memos', search: 'Search notes…', icon: 'tabler:notes' },
  knowledge: { title: 'Knowledge Graph', subtitle: 'Graph exploration', search: 'Search knowledge graph…', icon: 'tabler:share' },
  review: { title: 'Review', subtitle: 'Review queue', search: 'Search review…', icon: 'tabler:clipboard-check' },
  'event-tracing': { title: 'Event Traces', subtitle: 'Causal event graph', search: 'Search event traces…', icon: 'tabler:route' },
  settings: { title: 'Settings', subtitle: 'Application settings', search: 'Search settings…', icon: 'tabler:settings' },
  agents: { title: 'AI Agents', subtitle: 'AI control center', search: '', icon: 'tabler:sparkles' },
  organizations: { title: 'Organizations', subtitle: 'Organizations workspace', search: 'Search organizations…', icon: 'tabler:building-community' },
  telegram: { title: 'Telegram', subtitle: 'Telegram messages', search: 'Search Telegram…', icon: 'tabler:brand-telegram' },
  whatsapp: { title: 'WhatsApp', subtitle: 'WhatsApp messages', search: 'Search WhatsApp…', icon: 'tabler:brand-whatsapp' }
}

export const useNavigationStore = defineStore('navigation', () => {
  const router = useRouter()

  const currentView = ref<AppViewId>('home')
  const activeCommunicationSection = ref<CommunicationSectionId>('unified')
  const isSidebarRail = ref(false)
  const isUserMenuOpen = ref(false)
  const expandedSidebarGroupIds = ref<string[]>(['communications'])
  const activeSidebarRailGroupId = ref<string | null>(null)

  const activeWorkspaceView = computed<SidebarViewId>(() => {
    if (currentView.value === 'communications') {
      return communicationSectionViewId(activeCommunicationSection.value)
    }
    return currentView.value as SidebarViewId
  })

  const activeView = computed<ViewCopy | null>(() => {
    return viewCopy[currentView.value] ?? null
  })

  const shellViewClass = computed<string>(() => {
    return `view-${currentView.value}`
  })

  function navigateTo(viewId: AppViewId): void {
    currentView.value = viewId
    activeSidebarRailGroupId.value = null
    router.push(`/${viewId}`)
  }

  function navigateToCommunicationSection(sectionId: CommunicationSectionId): void {
    currentView.value = 'communications'
    activeCommunicationSection.value = sectionId
    router.push({ name: 'communications', query: { section: sectionId } })
  }

  function syncFromRoute(viewId: RouteViewId, sectionQuery?: RouteSectionQuery): void {
    currentView.value = viewId
    if (viewId !== 'communications') {
      activeSidebarRailGroupId.value = null
      return
    }

    activeCommunicationSection.value = communicationSectionFromQuery(sectionQuery) ?? 'unified'
  }

  function toggleUserMenu(): void {
    isUserMenuOpen.value = !isUserMenuOpen.value
  }

  function closeUserMenu(): void {
    isUserMenuOpen.value = false
  }

  function toggleSidebarRail(): void {
    isSidebarRail.value = !isSidebarRail.value
  }

  function toggleSidebarGroup(groupId: string): void {
    const index = expandedSidebarGroupIds.value.indexOf(groupId)
    if (index >= 0) {
      expandedSidebarGroupIds.value.splice(index, 1)
    } else {
      expandedSidebarGroupIds.value.push(groupId)
    }
  }

  function setActiveSidebarRailGroup(groupId: string | null): void {
    activeSidebarRailGroupId.value = groupId
  }

  return {
    currentView,
    activeCommunicationSection,
    isSidebarRail,
    isUserMenuOpen,
    expandedSidebarGroupIds,
    activeSidebarRailGroupId,
    activeWorkspaceView,
    activeView,
    shellViewClass,
    navigateTo,
    navigateToCommunicationSection,
    syncFromRoute,
    toggleUserMenu,
    closeUserMenu,
    toggleSidebarRail,
    toggleSidebarGroup,
    setActiveSidebarRailGroup
  }
})
```

### `frontend/src/shared/stores/notifications.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/notifications.ts`
- Size bytes / Размер в байтах: `2701`
- Included characters / Включено символов: `2701`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type NotificationItem = {
  id: string
  title: string
  body?: string
  icon: string
  time: Date
  targetView?: string
  targetId?: string
}

export const useNotificationsStore = defineStore('notifications', () => {
  const isNotificationsDrawerOpen = ref(false)
  const dismissedNotificationIds = ref<Set<string>>(new Set())
  const expandedNotificationIds = ref<Set<string>>(new Set())
  const rawNotificationItems = ref<NotificationItem[]>([])
  const pendingNotificationTarget = ref<NotificationItem | null>(null)

  // In a real implementation, notification items would come from SSE events
  // For now, using an empty array as the shell-ready state

  const notificationItems = computed<NotificationItem[]>(() => {
    return rawNotificationItems.value
      .filter((item) => !dismissedNotificationIds.value.has(item.id))
      .sort((a, b) => b.time.getTime() - a.time.getTime())
      .slice(0, 12)
  })

  const notificationCount = computed<number>(() => {
    return notificationItems.value.length
  })

  function toggleNotificationsDrawer(): void {
    isNotificationsDrawerOpen.value = !isNotificationsDrawerOpen.value
  }

  function closeNotificationsDrawer(): void {
    isNotificationsDrawerOpen.value = false
  }

  function dismissNotification(notificationId: string): void {
    dismissedNotificationIds.value = new Set([...dismissedNotificationIds.value, notificationId])
  }

  function toggleNotificationExpanded(notificationId: string): void {
    const newSet = new Set(expandedNotificationIds.value)
    if (newSet.has(notificationId)) {
      newSet.delete(notificationId)
    } else {
      newSet.add(notificationId)
    }
    expandedNotificationIds.value = newSet
  }

  function openNotificationTarget(notification: NotificationItem): void {
    pendingNotificationTarget.value = notification
    closeNotificationsDrawer()
  }

  function consumePendingNotificationTarget(): NotificationItem | null {
    const target = pendingNotificationTarget.value
    pendingNotificationTarget.value = null
    return target
  }

  function addNotification(notification: NotificationItem): void {
    rawNotificationItems.value = [notification, ...rawNotificationItems.value]
  }

  return {
    isNotificationsDrawerOpen,
    dismissedNotificationIds,
    expandedNotificationIds,
    rawNotificationItems,
    pendingNotificationTarget,
    notificationItems,
    notificationCount,
    toggleNotificationsDrawer,
    closeNotificationsDrawer,
    dismissNotification,
    toggleNotificationExpanded,
    openNotificationTarget,
    consumePendingNotificationTarget,
    addNotification
  }
})
```

### `frontend/src/shared/stores/realtimeStatus.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/realtimeStatus.test.ts`
- Size bytes / Размер в байтах: `3322`
- Included characters / Включено символов: `3322`
- Truncated / Обрезано: `no`

```typescript
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useRealtimeStatusStore } from './realtimeStatus'

beforeEach(() => {
	setActivePinia(createPinia())
})

describe('realtime status store', () => {
	it('summarizes live and degraded realtime transport states for shell UI', () => {
		const store = useRealtimeStatusStore()

		expect(store.realtimeStatusLabel).toBe('Realtime starting')
		expect(store.realtimeStatusTone).toBe('neutral')
		expect(store.isRealtimeDegraded).toBe(false)

		store.setRealtimeStatus({ transport: 'sse', state: 'connected' })
		expect(store.realtimeStatusLabel).toBe('Realtime live')
		expect(store.realtimeStatusTone).toBe('success')
		expect(store.isRealtimeDegraded).toBe(false)

		store.setRealtimeStatus({ transport: 'long_poll', state: 'fallback' })
		expect(store.realtimeStatusLabel).toBe('Realtime fallback')
		expect(store.realtimeStatusTone).toBe('warning')
		expect(store.isRealtimeDegraded).toBe(true)
	})

	it('keeps sanitized error context for reconnecting transports', () => {
		const store = useRealtimeStatusStore()

		store.setRealtimeStatus({
			transport: 'sse',
			state: 'reconnecting',
			attempt: 2,
			maxAttempts: 10,
			error: 'SSE connection failed with HTTP 503'
		})

		expect(store.status.error).toBe('SSE connection failed with HTTP 503')
		expect(store.realtimeStatusDetail).toBe(
			'Realtime reconnecting: SSE connection failed with HTTP 503'
		)
		expect(store.realtimeStatusTone).toBe('warning')
	})

	it('tracks replay cursor progress for offline recovery diagnostics', () => {
		const store = useRealtimeStatusStore()

		expect(store.realtimeRecoveryDetail).toBe('Waiting for first replay cursor')

		store.observeRealtimeEvent('51')
		expect(store.status.lastEventId).toBe('51')
		expect(store.status.lastEventAt).toBeTruthy()
		expect(store.realtimeRecoveryDetail).toContain('Replay cursor 51.')

		store.setRealtimeStatus({
			transport: 'sse',
			state: 'disconnected',
			error: 'stream closed'
		})
		expect(store.realtimeRecoveryDetail).toContain(
			'Offline recovery will resume from cursor 51.'
		)
		expect(store.canTriggerReconnect).toBe(true)
	})

	it('surfaces replay-gap diagnostics when the transport reports skipped realtime events', () => {
		const store = useRealtimeStatusStore()

		store.observeRealtimeEvent('51')
		store.observeRealtimeLag(7)

		expect(store.isRealtimeDegraded).toBe(true)
		expect(store.canTriggerReconnect).toBe(true)
		expect(store.realtimeRecoveryDetail).toContain('Replay gap detected after cursor 51.')
		expect(store.realtimeRecoveryDetail).toContain('Skipped 7 events.')

		store.setRealtimeStatus({
			transport: 'websocket',
			state: 'connected'
		})
		expect(store.isRealtimeDegraded).toBe(false)
		expect(store.realtimeRecoveryDetail).toContain('Replay cursor 51.')
	})

	it('exposes a manual reconnect control only for degraded or disconnected transports', () => {
		const store = useRealtimeStatusStore()
		const reconnect = vi.fn()

		store.setReconnectHandler(reconnect)
		expect(store.canTriggerReconnect).toBe(false)

		store.requestReconnect()
		expect(reconnect).toHaveBeenCalledTimes(1)

		store.setRealtimeStatus({
			transport: 'long_poll',
			state: 'fallback'
		})
		expect(store.canTriggerReconnect).toBe(true)
	})
})
```

### `frontend/src/shared/stores/realtimeStatus.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/realtimeStatus.ts`
- Size bytes / Размер в байтах: `5692`
- Included characters / Включено символов: `5692`
- Truncated / Обрезано: `no`

```typescript
import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

export type RealtimeTransportKind = 'websocket' | 'sse' | 'long_poll'
export type RealtimeTransportState =
	| 'idle'
	| 'connecting'
	| 'connected'
	| 'reconnecting'
	| 'fallback'
	| 'disconnected'
export type RealtimeStatusTone = 'neutral' | 'success' | 'warning' | 'danger'

export type RealtimeStatusUpdate = {
	transport: RealtimeTransportKind
	state: Exclude<RealtimeTransportState, 'idle'>
	attempt?: number
	maxAttempts?: number
	error?: string
}

export type RealtimeStatusSnapshot = {
	transport: RealtimeTransportKind
	state: RealtimeTransportState
	attempt: number | null
	maxAttempts: number | null
	error: string | null
	lastEventId: string | null
	lastEventAt: string | null
	lastLaggedSkipped: number | null
	lastLaggedAt: string | null
	updatedAt: string | null
}

const initialStatus: RealtimeStatusSnapshot = {
	transport: 'websocket',
	state: 'idle',
	attempt: null,
	maxAttempts: null,
	error: null,
	lastEventId: null,
	lastEventAt: null,
	lastLaggedSkipped: null,
	lastLaggedAt: null,
	updatedAt: null
}

export const useRealtimeStatusStore = defineStore('realtimeStatus', () => {
	const status = ref<RealtimeStatusSnapshot>({ ...initialStatus })
	let reconnectHandler: (() => void) | null = null

	const isRealtimeDegraded = computed<boolean>(() => {
		return (
			status.value.state === 'reconnecting' ||
			status.value.transport === 'long_poll' ||
			status.value.lastLaggedSkipped !== null
		)
	})

	const canTriggerReconnect = computed<boolean>(() => {
		return status.value.state === 'disconnected' || isRealtimeDegraded.value
	})

	const realtimeStatusLabel = computed<string>(() => {
		if (status.value.state === 'idle') return 'Realtime starting'
		if (status.value.state === 'connecting') return 'Realtime connecting'
		if (status.value.state === 'connected') {
			return status.value.transport === 'long_poll' ? 'Realtime fallback' : 'Realtime live'
		}
		if (status.value.state === 'reconnecting') return 'Realtime reconnecting'
		if (status.value.state === 'fallback') return 'Realtime fallback'
		return 'Realtime offline'
	})

	const realtimeStatusTone = computed<RealtimeStatusTone>(() => {
		if (status.value.state === 'connected' && status.value.transport !== 'long_poll') {
			return 'success'
		}
		if (
			status.value.state === 'reconnecting' ||
			status.value.state === 'fallback' ||
			status.value.transport === 'long_poll'
		) {
			return 'warning'
		}
		if (status.value.state === 'disconnected') return 'danger'
		return 'neutral'
	})

	const realtimeStatusDetail = computed<string>(() => {
		const base = realtimeStatusLabel.value
		if (status.value.error) return `${base}: ${status.value.error}`
		if (status.value.attempt && status.value.maxAttempts) {
			return `${base}: attempt ${status.value.attempt}/${status.value.maxAttempts}`
		}
		return base
	})

	const realtimeRecoveryDetail = computed<string>(() => {
		const cursor = status.value.lastEventId
		const laggedSkipped = status.value.lastLaggedSkipped
		if (laggedSkipped !== null) {
			const cursorLabel = cursor ?? 'unknown'
			return `Replay gap detected after cursor ${cursorLabel}. Skipped ${laggedSkipped} event${laggedSkipped === 1 ? '' : 's'}. Reconnect to recover missed updates.`
		}
		if (!cursor) {
			return 'Waiting for first replay cursor'
		}

		const lastEventAt = status.value.lastEventAt ? formatRecoveryTimestamp(status.value.lastEventAt) : 'unknown'
		if (status.value.state === 'disconnected') {
			return `Offline recovery will resume from cursor ${cursor}. Last event ${lastEventAt}`
		}
		if (isRealtimeDegraded.value) {
			return `Recovery cursor ${cursor}. Last event ${lastEventAt}`
		}
		return `Replay cursor ${cursor}. Last event ${lastEventAt}`
	})

	function setRealtimeStatus(update: RealtimeStatusUpdate): void {
		status.value = {
			transport: update.transport,
			state: update.state,
			attempt: update.attempt ?? null,
			maxAttempts: update.maxAttempts ?? null,
			error: update.error?.trim() || null,
			lastEventId: status.value.lastEventId,
			lastEventAt: status.value.lastEventAt,
			lastLaggedSkipped:
				update.state === 'connected' && update.transport !== 'long_poll'
					? null
					: status.value.lastLaggedSkipped,
			lastLaggedAt:
				update.state === 'connected' && update.transport !== 'long_poll'
					? null
					: status.value.lastLaggedAt,
			updatedAt: new Date().toISOString()
		}
	}

	function observeRealtimeEvent(eventId: string): void {
		const cursor = eventId.trim()
		if (!cursor) return

		status.value = {
			...status.value,
			lastEventId: cursor,
			lastEventAt: new Date().toISOString(),
			updatedAt: new Date().toISOString()
		}
	}

	function observeRealtimeLag(skipped: number): void {
		if (!Number.isFinite(skipped) || skipped <= 0) return

		status.value = {
			...status.value,
			lastLaggedSkipped: Math.floor(skipped),
			lastLaggedAt: new Date().toISOString(),
			updatedAt: new Date().toISOString()
		}
	}

	function resetRealtimeStatus(): void {
		status.value = { ...initialStatus }
	}

	function setReconnectHandler(handler: (() => void) | null): void {
		reconnectHandler = handler
	}

	function requestReconnect(): void {
		reconnectHandler?.()
	}

	return {
		status,
		isRealtimeDegraded,
		canTriggerReconnect,
		realtimeStatusLabel,
		realtimeStatusTone,
		realtimeStatusDetail,
		realtimeRecoveryDetail,
		setRealtimeStatus,
		observeRealtimeEvent,
		observeRealtimeLag,
		resetRealtimeStatus,
		setReconnectHandler,
		requestReconnect
	}
})

function formatRecoveryTimestamp(value: string): string {
	const date = new Date(value)
	if (Number.isNaN(date.getTime())) return value
	return date.toISOString()
}
```

### `frontend/src/shared/stores/sidebar.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/sidebar.ts`
- Size bytes / Размер в байтах: `14656`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type PrimaryNavId =
  | 'home'
  | 'communications'
  | 'timeline'
  | 'persons'
  | 'projects'
  | 'tasks'
  | 'calendar'
  | 'documents'
  | 'notes'
  | 'knowledge'
  | 'review'
  | 'event-tracing'
  | 'agents'

export type CommunicationSectionId =
  | 'unified'
  | 'inbox'
  | 'waiting'
  | 'needs_reply'
  | 'mentions'
  | 'mail'
  | 'telegram'
  | 'whatsapp'
  | 'calls'
  | 'meetings'

export type CommunicationSidebarSectionId = Extract<
  CommunicationSectionId,
  'mail' | 'telegram' | 'whatsapp' | 'calls' | 'meetings'
>
export type CommunicationSidebarItemId = `communications.${CommunicationSidebarSectionId}`
export type SidebarPrimaryItemId = Exclude<PrimaryNavId, 'communications'>
export type SidebarItemId = SidebarPrimaryItemId | CommunicationSidebarItemId
export type SidebarRootItemId = SidebarPrimaryItemId | `group:${string}`

export type PrimaryWorkspaceNavItem = {
  id: PrimaryNavId
  label: string
  icon: string
}

export type CommunicationSection = {
  id: CommunicationSectionId
  label: string
  icon: string
  group: 'overview' | 'workflow' | 'sources'
}

export type SidebarNavGroup = {
  id: string
  label: string
  icon: string
  itemIds: SidebarItemId[]
  separatorBeforeItemIds: SidebarItemId[]
}

export type SidebarSettings = {
  schemaVersion: 3
  rootItemIds: SidebarRootItemId[]
  groups: SidebarNavGroup[]
  hiddenItemIds: SidebarItemId[]
}

export type ResolvedSidebarItem = {
  itemId: SidebarItemId
  label: string
  icon: string
  isCommunication: boolean
  sectionId?: CommunicationSidebarSectionId
}

export type ResolvedSidebarRootEntry =
  | {
      kind: 'item'
      rootId: SidebarPrimaryItemId
      item: ResolvedSidebarItem
    }
  | {
      kind: 'group'
      rootId: `group:${string}`
      group: SidebarNavGroup & { items: ResolvedSidebarItem[] }
    }

const SIDEBAR_SETTINGS_SCHEMA_VERSION = 3 as const

export const primaryWorkspaceNav: PrimaryWorkspaceNavItem[] = [
  { id: 'home', label: 'Home', icon: 'tabler:home' },
  { id: 'communications', label: 'Communications', icon: 'tabler:messages' },
  { id: 'timeline', label: 'Timeline', icon: 'tabler:timeline-event' },
  { id: 'persons', label: 'Persons', icon: 'tabler:user' },
  { id: 'projects', label: 'Projects', icon: 'tabler:briefcase' },
  { id: 'tasks', label: 'Tasks', icon: 'tabler:checkbox' },
  { id: 'calendar', label: 'Calendar', icon: 'tabler:calendar' },
  { id: 'documents', label: 'Documents', icon: 'tabler:file-text' },
  { id: 'notes', label: 'Notes', icon: 'tabler:notes' },
  { id: 'knowledge', label: 'Knowledge Graph', icon: 'tabler:share' },
  { id: 'review', label: 'Review', icon: 'tabler:clipboard-check' },
  { id: 'event-tracing', label: 'Event Traces', icon: 'tabler:route' },
  { id: 'agents', label: 'AI Agents', icon: 'tabler:sparkles' }
]

export const communicationSections: CommunicationSection[] = [
  { id: 'unified', label: 'Unified', icon: 'tabler:sparkles', group: 'overview' },
  { id: 'inbox', label: 'Inbox', icon: 'tabler:mail', group: 'workflow' },
  { id: 'waiting', label: 'Waiting', icon: 'tabler:clock-hour-4', group: 'workflow' },
  { id: 'needs_reply', label: 'Needs Reply', icon: 'tabler:message-reply', group: 'workflow' },
  { id: 'mentions', label: 'Mentions', icon: 'tabler:at', group: 'workflow' },
  { id: 'mail', label: 'Mail', icon: 'tabler:mail', group: 'sources' },
  { id: 'telegram', label: 'Telegram', icon: 'tabler:brand-telegram', group: 'sources' },
  { id: 'whatsapp', label: 'WhatsApp', icon: 'tabler:brand-whatsapp', group: 'sources' },
  { id: 'calls', label: 'Calls', icon: 'tabler:phone', group: 'sources' },
  { id: 'meetings', label: 'Meetings', icon: 'tabler:calendar-event', group: 'sources' }
]

const communicationSidebarSectionIds: CommunicationSidebarSectionId[] = [
  'mail', 'telegram', 'whatsapp', 'calls', 'meetings'
]

function communicationSidebarItemId(sectionId: CommunicationSidebarSectionId): CommunicationSidebarItemId {
  return `communications.${sectionId}`
}

function normalizeGroupId(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9-]+/g, '-').replace(/^-+|-+$/g, '')
}

function sidebarGroupRootId(groupId: string): `group:${string}` {
  return `group:${normalizeGroupId(groupId)}`
}

function defaultSidebarSettings(): SidebarSettings {
  const communicationSectionsInSidebar = communicationSections.filter((s) =>
    communicationSidebarSectionIds.includes(s.id as CommunicationSidebarSectionId)
  ) as Array<CommunicationSection & { id: CommunicationSidebarSectionId }>

  const defaultCommunicationsGroup: SidebarNavGroup = {
    id: 'communications',
    label: 'Communications',
    icon: 'tabler:messages',
    itemIds: [
      ...communicationSectionsInSidebar.map((s) => communicationSidebarItemId(s.id)),
      'timeline' as SidebarPrimaryItemId
    ],
    separatorBeforeItemIds: []
  }

  const communicationGroupPrimaryItemIds: SidebarPrimaryItemId[] = ['timeline']

  const defaultRootItemIds: SidebarRootItemId[] = primaryWorkspaceNav.flatMap((item) =>
    item.id === 'communications'
      ? [sidebarGroupRootId(defaultCommunicationsGroup.id)]
      : !communicationGroupPrimaryItemIds.includes(item.id as SidebarPrimaryItemId)
        ? [item.id as SidebarPrimaryItemId]
        : []
  )

  return {
    schemaVersion: SIDEBAR_SETTINGS_SCHEMA_VERSION,
    rootItemIds: defaultRootItemIds,
    groups: [defaultCommunicationsGroup],
    hiddenItemIds: []
  }
}

function resolveSidebarItem(itemId: SidebarItemId): ResolvedSidebarItem | null {
  // Check if it's a primary nav item
  const primaryItem = primaryWorkspaceNav.find((p) => p.id === itemId)
  if (primaryItem) {
    return {
      itemId: itemId as SidebarPrimaryItemId,
      label: primaryItem.label,
      icon: primaryItem.icon,
      isCommunication: false
    }
  }

  // Check if it's a communication sidebar item
  if (itemId.startsWith('communications.')) {
    const sectionId = itemId.slice('communications.'.length) as CommunicationSidebarSectionId
    const section = communicationSections.find((s) => s.id === sectionId)
    if (section && communicationSidebarSectionIds.includes(sectionId)) {
      return {
        itemId,
        label: section.label,
        icon: section.icon,
        isCommunication: true,
        sectionId
      }
    }
  }

  return null
}

export const useSidebarStore = defineStore('sidebar', () => {
  const sidebarSettings = ref<SidebarSettings>(defaultSidebarSettings())
  const sidebarDraft = ref<SidebarSettings | null>(null)

  const effectiveSidebarSettings = computed<SidebarSettings>(() => {
    return sidebarDraft.value ?? sidebarSettings.value
  })

  const sidebarRootEntries = computed<ResolvedSidebarRootEntry[]>(() => {
    const entries: ResolvedSidebarRootEntry[] = []
    const settings = effectiveSidebarSettings.value

    for (const rootId of settings.rootItemIds) {
      // Check if it's a primary item
      if (!rootId.startsWith('group:')) {
        const primaryId = rootId as SidebarPrimaryItemId
        const resolved = resolveSidebarItem(primaryId)
        if (resolved) {
          entries.push({ kind: 'item', rootId: primaryId, item: resolved })
        }
        continue
      }

      // It's a group
      const groupId = rootId.slice('group:'.length)
      const group = settings.groups.find((g) => normalizeGroupId(g.id) === groupId)
      if (!group) continue

      const items: ResolvedSidebarItem[] = []
      for (const itemId of group.itemIds) {
        if (!settings.hiddenItemIds.includes(itemId)) {
          const resolved = resolveSidebarItem(itemId)
          if (resolved) items.push(resolved)
        }
      }

      entries.push({
        kind: 'group',
        rootId: rootId as `group:${string}`,
        group: { ...group, items }
      })
    }

    return entries
  })

  const sidebarHiddenNavItems = computed<SidebarItemId[]>(() => {
    return effectiveSidebarSettings.value.hiddenItemIds
  })

  function setSidebarSettings(settings: SidebarSettings): void {
    sidebarSettings.value = settings
  }

  function updateSidebarDraft(update: (draft: SidebarSettings) => SidebarSettings): void {
    if (!sidebarDraft.value) {
      sidebarDraft.value = JSON.parse(JSON.stringify(sidebarSettings.value))
    }
    const draft = sidebarDraft.value as SidebarSettings
    sidebarDraft.value = update(draft)
  }

  function resetSidebarSettingsToDefault(): void {
    sidebarDraft.value = defaultSidebarSettings()
  }

  function cancelSidebarSettingsEditing(): void {
    sidebarDraft.value = null
  }

  function sidebarConfigItem(itemId: SidebarItemId): { id: SidebarItemId; label: string; icon: string } | null {
    const resolved = resolveSidebarItem(itemId)
    if (resolved) {
      return { id: resolved.itemId, label: resolved.label, icon: resolved.icon }
    }
    return null
  }

  function sidebarGroupIdFromLabel(label: string): string {
    return label.toLowerCase().replace(/[^a-z0-9-]+/g, '-').replace(/^-+|-+$/g, '')
  }

  function addSidebarGroup(): void {
    updateSidebarDraft((draft) => {
      const groupCount = draft.groups.filter((g) => g.id.startsWith('group-')).length + 1
      const label = `Group ${groupCount}`
      const id = `group-${groupCount}`
      return {
        ...draft,
        groups: [...draft.groups, { id, label, icon: 'tabler:folder', itemIds: [], separatorBeforeItemIds: [] }],
        rootItemIds: [...draft.rootItemIds, `group:${id}`]
      }
    })
  }

  function removeSidebarGroup(groupId: string): void {
    updateSidebarDraft((draft) => {
      const normalized = normalizeGroupId(groupId)
      return {
        ...draft,
        groups: draft.groups.filter((g) => normalizeGroupId(g.id) !== normalized),
        rootItemIds: draft.rootItemIds.filter((id) => {
          if (id.startsWith('group:')) {
            return normalizeGroupId(id.slice('group:'.length)) !== normalized
          }
          return true
        })
      }
    })
  }

  function moveSidebarGroup(groupId: string, direction: -1 | 1): void {
    updateSidebarDraft((draft) => {
      const normalized = normalizeGroupId(groupId)
      const idx = draft.groups.findIndex((g) => normalizeGroupId(g.id) === normalized)
      if (idx < 0) return draft
      const newIdx = idx + direction
      if (newIdx < 0 || newIdx >= draft.groups.length) return draft
      const groups = [...draft.groups]
      const [moved] = groups.splice(idx, 1)
      groups.splice(newIdx, 0, moved)
      return { ...draft, groups }
    })
  }

  function moveSidebarRootItem(rootId: string, direction: -1 | 1): void {
    updateSidebarDraft((draft) => {
      const idx = draft.rootItemIds.indexOf(rootId as SidebarRootItemId)
      if (idx < 0) return draft
      const newIdx = idx + direction
      if (newIdx < 0 || newIdx >= draft.rootItemIds.length) return draft
      const items = [...draft.rootItemIds]
      const [moved] = items.splice(idx, 1)
      items.splice(newIdx, 0, moved)
      return { ...draft, rootItemIds: items }
    })
  }

  function moveSidebarItem(itemId: SidebarItemId, direction: -1 | 1): void {
    updateSidebarDraft((draft) => {
      const groups = draft.groups.map((group) => {
        const idx = group.itemIds.indexOf(itemId)
        if (idx < 0) return group
        const newIdx = idx + direction
        if (newIdx < 0 || newIdx >= group.itemIds.length) return group
        const items = [...group.itemIds]
        const [moved] = items.splice(idx, 1)
        items.splice(newIdx, 0, moved)
        return { ...group, itemIds: items }
      })
      return { ...draft, groups }
    })
  }

  function moveSidebarItemToGroup(itemId: SidebarItemId, targetGroupId: string): void {
    updateSidebarDraft((draft) => {
      const normalizedTarget = normalizeGroupId(targetGroupId)
      let sourceItemRemoved = false

      const groups = draft.groups.map((group) => {
        const normalized = normalizeGroupId(group.id)
        if (normaliz
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/shared/stores/theme.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/stores/theme.ts`
- Size bytes / Размер в байтах: `5715`
- Included characters / Включено символов: `5715`
- Truncated / Обрезано: `no`

```typescript
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
	accentColorIds,
	defaultThemeSettings,
	parseThemeSettings,
	shellAccentClass as themeAccentClass,
	shellBackgroundClass as themeBackgroundClass,
	shellBackgroundIds,
	shellBrightnessClass as themeBrightnessClass,
	shellPanelBlurClass as themePanelBlurClass,
	shellPanelOpacityClass as themePanelOpacityClass,
	shellSpacingDensityClass as themeSpacingDensityClass,
	type AccentColorId,
	type ShellBackgroundId,
	type ThemeSettings
} from '../../platform/theme/settings'
import {
	loadLocalThemeSettings,
	loadPersistedThemeSettings,
	savePersistedThemeSettings,
	type ThemePersistenceSource
} from '../../platform/theme/persistence'

export type {
	AccentColorId as ShellAccentColorId,
	BackgroundBrightness,
	PanelBlur as ShellPanelBlur,
	PanelOpacity as ShellPanelOpacity,
	ShellBackgroundId,
	SpacingDensity,
	ThemeSettings as FrontendThemeSettings
} from '../../platform/theme/settings'

export const useThemeStore = defineStore('theme', () => {
	const themeSettings = ref<ThemeSettings>(loadLocalThemeSettings())
	const themeDraft = ref<ThemeSettings | null>(null)
	const isHydratingTheme = ref(false)
	const isSavingTheme = ref(false)
	const themePersistenceSource = ref<ThemePersistenceSource>('local_storage')
	const themePersistenceError = ref('')

	const effectiveThemeSettings = computed<ThemeSettings>(() => {
		return themeDraft.value ?? themeSettings.value
	})

	const shellBackgroundClass = computed<string>(() => {
		return themeBackgroundClass(effectiveThemeSettings.value)
	})

	const shellBrightnessClass = computed<string>(() => {
		return themeBrightnessClass(effectiveThemeSettings.value)
	})

	const shellAccentClass = computed<string>(() => {
		return themeAccentClass(effectiveThemeSettings.value)
	})

	const shellPanelOpacityClass = computed<string>(() => {
		return themePanelOpacityClass(effectiveThemeSettings.value)
	})

	const shellPanelBlurClass = computed<string>(() => {
		return themePanelBlurClass(effectiveThemeSettings.value)
	})

	const shellSpacingDensityClass = computed<string>(() => {
		return themeSpacingDensityClass(effectiveThemeSettings.value)
	})

		const shellThemeClass = computed<string>(() => {
			return [
				shellBackgroundClass.value,
			shellBrightnessClass.value,
			shellAccentClass.value,
			shellPanelOpacityClass.value,
			shellPanelBlurClass.value,
			shellSpacingDensityClass.value
			].join(' ')
		})

		const themePersistenceLabel = computed<string>(() => {
			if (isSavingTheme.value) return 'Saving'
			if (themePersistenceError.value) return 'Local fallback'
			return themePersistenceSource.value === 'application_settings' ? 'Auto-save' : 'Local settings'
		})

	function startThemeEditing(): void {
		themeDraft.value = { ...themeSettings.value }
	}

	function updateThemeDraft(patch: Partial<ThemeSettings>): void {
		const current = themeDraft.value ?? themeSettings.value
		themeDraft.value = parseThemeSettings({
			...current,
			...patch,
			schemaVersion: current.schemaVersion
		})
	}

	function cancelThemeEditing(): void {
		themeDraft.value = null
	}

	async function hydrateThemeSettings(): Promise<void> {
			isHydratingTheme.value = true
			themePersistenceError.value = ''
			try {
				const result = await loadPersistedThemeSettings()
				themeSettings.value = result.settings
				themePersistenceSource.value = result.source
				themePersistenceError.value = result.errorMessage
				themeDraft.value = null
			} catch (error) {
				themePersistenceError.value = error instanceof Error ? error.message : 'Failed to load theme'
		} finally {
			isHydratingTheme.value = false
		}
	}

	async function saveThemeSettings(): Promise<void> {
		isSavingTheme.value = true
			themePersistenceError.value = ''
			try {
				const next = themeDraft.value ?? themeSettings.value
				const result = await savePersistedThemeSettings(next)
				themeSettings.value = result.settings
				themePersistenceSource.value = result.source
				themePersistenceError.value = result.errorMessage
				themeDraft.value = null
			} catch (error) {
				themePersistenceError.value = error instanceof Error ? error.message : 'Failed to save theme'
		} finally {
			isSavingTheme.value = false
		}
	}

	function resetThemeSettings(): void {
		themeDraft.value = defaultThemeSettings()
	}

	function shellBackgroundLabel(id: ShellBackgroundId): string {
			const labels: Record<ShellBackgroundId, string> = {
				none: 'No background',
				'eclipse-grid': 'Dark grid',
				'data-stream': 'Data flow',
				'network-mesh': 'Digital network',
				'forest-network': 'Green network',
				'knowledge-map': 'Knowledge map',
				'forest-stream': 'Green flow',
				'dna-blueprint': 'Connection blueprint',
				'node-frame': 'Node grid',
				'rune-teal': 'Teal accent',
				'rune-gold': 'Warm accent'
			}
		return labels[id] ?? id
	}

	function shellAccentLabel(id: AccentColorId): string {
		const labels: Record<AccentColorId, string> = {
			teal: 'Teal',
			cyan: 'Cyan',
			blue: 'Blue',
			violet: 'Violet',
			amber: 'Amber',
			rose: 'Rose'
		}
		return labels[id] ?? id
	}

	return {
		themeSettings,
		themeDraft,
			isHydratingTheme,
			isSavingTheme,
			themePersistenceSource,
			themePersistenceError,
			themePersistenceLabel,
			backgroundOptions: shellBackgroundIds,
		accentOptions: accentColorIds,
		effectiveThemeSettings,
		shellBackgroundClass,
		shellBrightnessClass,
		shellAccentClass,
		shellPanelOpacityClass,
		shellPanelBlurClass,
		shellSpacingDensityClass,
		shellThemeClass,
		startThemeEditing,
		updateThemeDraft,
		cancelThemeEditing,
		hydrateThemeSettings,
		saveThemeSettings,
		resetThemeSettings,
		shellBackgroundLabel,
		shellAccentLabel
	}
})
```

### `frontend/src/shared/transitions/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/transitions/index.ts`
- Size bytes / Размер в байтах: `132`
- Included characters / Включено символов: `132`
- Truncated / Обрезано: `no`

```typescript
export { default as FadeTransition } from './FadeTransition.vue'
export { default as SlideTransition } from './SlideTransition.vue'
```

### `frontend/src/shared/ui/Dialog.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Dialog.boundary.test.ts`
- Size bytes / Размер в байтах: `497`
- Included characters / Включено символов: `497`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Dialog controlled mode compatibility', () => {
  it('renders DialogTrigger only when a trigger slot is provided', () => {
    const source = readFileSync(new URL('./Dialog.vue', import.meta.url), 'utf8')

    expect(source).toContain('<DialogTrigger v-if="$slots.trigger" as-child>')
    expect(source).toContain('<DialogRoot :open="open" @update:open="(val) => emit(\'update:open\', val)">')
  })
})
```

### `frontend/src/shared/ui/Tabs.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Tabs.boundary.test.ts`
- Size bytes / Размер в байтах: `634`
- Included characters / Включено символов: `634`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Tabs compatibility API', () => {
  it('renders trigger buttons from tabs/active props and emits select events', () => {
    const source = readFileSync(new URL('./Tabs.vue', import.meta.url), 'utf8')

    expect(source).toContain('tabs?: МакошьTab[]')
    expect(source).toContain('active?: string')
    expect(source).toContain('select: [value: string]')
    expect(source).toContain('<TabsTrigger')
    expect(source).toContain('v-for="tab in tabs"')
    expect(source).toContain('@update:model-value="handleUpdateModelValue"')
  })
})
```

### `frontend/src/shared/ui/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/index.ts`
- Size bytes / Размер в байтах: `1728`
- Included characters / Включено символов: `1728`
- Truncated / Обрезано: `no`

```typescript
export { default as Icon } from './Icon.vue'
export { default as Button } from './Button.vue'
export { default as Input } from './Input.vue'
export { default as Textarea } from './Textarea.vue'
export { default as Label } from './Label.vue'
export { default as Badge } from './Badge.vue'
export { default as Separator } from './Separator.vue'
export { default as Skeleton } from './Skeleton.vue'
export { default as ScrollArea } from './ScrollArea.vue'
export { default as Card } from './Card.vue'
export { default as CardHeader } from './CardHeader.vue'
export { default as CardTitle } from './CardTitle.vue'
export { default as CardDescription } from './CardDescription.vue'
export { default as CardContent } from './CardContent.vue'
export { default as CardFooter } from './CardFooter.vue'
export { default as Tabs } from './Tabs.vue'
export { default as TabTrigger } from './TabTrigger.vue'
export { default as TabContent } from './TabContent.vue'
export { default as Switch } from './Switch.vue'
export { default as Select } from './Select.vue'
export { default as Dialog } from './Dialog.vue'
export { default as Sheet } from './Sheet.vue'
export { default as Avatar } from './Avatar.vue'
export { default as Progress } from './Progress.vue'
export { default as Toast } from './Toast.vue'
export { default as Command } from './Command.vue'
export { default as DropdownMenu } from './DropdownMenu.vue'
export { default as DropdownMenuItem } from './DropdownMenuItem.vue'
export { default as DropdownMenuSeparator } from './DropdownMenuSeparator.vue'
export { default as DropdownMenuLabel } from './DropdownMenuLabel.vue'
export { default as Tooltip } from './Tooltip.vue'
export { default as Popover } from './Popover.vue'
```

### `frontend/src/shared/yandexTelemost/settingsBridge.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/yandexTelemost/settingsBridge.ts`
- Size bytes / Размер в байтах: `77`
- Included characters / Включено символов: `77`
- Truncated / Обрезано: `no`

```typescript
export type { ProviderAccount } from '../../domains/settings/types/settings'
```

### `frontend/src/shared/zoom/settingsBridge.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/zoom/settingsBridge.ts`
- Size bytes / Размер в байтах: `259`
- Included characters / Включено символов: `259`
- Truncated / Обрезано: `no`

```typescript
export { settingsKeys, useApplicationSettingsQuery } from '../../domains/settings/queries/useSettingsQuery'
export { useSettingsStore } from '../../domains/settings/stores/settings'
export type { ProviderAccount } from '../../domains/settings/types/settings'
```

### `frontend/src/vite-env.d.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/vite-env.d.ts`
- Size bytes / Размер в байтах: `195`
- Included characters / Включено символов: `195`
- Truncated / Обрезано: `no`

```typescript
/// <reference types="vite/client" />

declare module '*.vue' {
	import type { DefineComponent } from 'vue'
	const component: DefineComponent<object, object, unknown>
	export default component
}
```

### `frontend/tailwind.config.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/tailwind.config.ts`
- Size bytes / Размер в байтах: `2982`
- Included characters / Включено символов: `2982`
- Truncated / Обрезано: `no`

```typescript
import type { Config } from 'tailwindcss'

export default {
	content: ['./index.html', './src/**/*.{vue,ts,tsx}'],
	theme: {
		extend: {
			fontFamily: {
				sans: [
					'Inter',
					'SF Pro Display',
					'ui-sans-serif',
					'system-ui',
					'-apple-system',
					'BlinkMacSystemFont',
					'Segoe UI',
					'sans-serif'
				]
			},
			colors: {
				'hh-bg': '#02090b',
				'hh-bg-raised': '#020d10',
				'hh-surface': '#06181b',
				'hh-surface-deep': '#041215',
				'hh-text': '#eefefb',
				'hh-text-strong': '#f2fffd',
				'hh-text-bright': '#ffffff',
				'hh-text-soft': '#dcefed',
				'hh-text-muted': '#91a8a8',
				'hh-text-subtle': '#8ea4a6',
				'hh-text-dim': '#849ca0',
				'hh-accent': '#2df0ce',
				'hh-accent-strong': '#25d8bd',
				'hh-accent-soft': '#9ee8df',
				'hh-accent-contrast': '#032522',
				'hh-danger': '#ffabab',
				'hh-danger-strong': '#ef3140',
				'hh-border-accent-soft': 'rgba(45, 240, 206, 0.18)',
				'hh-border-accent': 'rgba(45, 240, 206, 0.42)',
				'hh-border-subtle': 'rgba(111, 205, 195, 0.14)',
				'hh-border-muted': 'rgba(102, 189, 180, 0.1)',
				'hh-focus-ring': 'rgba(45, 240, 206, 0.62)',
				'hh-surface-tint': 'rgba(5, 22, 25, 0.78)',
				'hh-surface-panel': 'rgba(8, 29, 33, 0.94)',
				'hh-accent-tint': 'rgba(45, 240, 206, 0.08)',
				'hh-accent-control': 'rgba(25, 154, 132, 0.2)',
				'hh-danger-tint': 'rgba(128, 32, 40, 0.26)',
				'hh-status-accent-surface': 'rgba(45, 240, 206, 0.08)',
				'hh-status-accent-text': '#2df0ce',
				'hh-status-warning-surface': 'rgba(240, 170, 70, 0.16)',
				'hh-status-warning-text': '#f4c889',
				'hh-status-info-surface': 'rgba(120, 156, 240, 0.18)',
				'hh-status-info-text': '#aec6f7',
				'hh-status-success-surface': 'rgba(45, 214, 150, 0.16)',
				'hh-status-success-text': '#7fe6b4',
				'hh-status-danger-surface': 'rgba(128, 32, 40, 0.26)',
				'hh-status-danger-text': '#ffabab',
				'hh-status-archive-surface': 'rgba(176, 132, 240, 0.18)',
				'hh-status-archive-text': '#cdb2f2',
				'hh-status-neutral-surface': 'rgba(124, 156, 156, 0.12)',
				'hh-status-neutral-text': '#91a8a8'
			},
			spacing: {
				'hh-1': '4px',
				'hh-2': '8px',
				'hh-3': '12px',
				'hh-4': '16px',
				'hh-5': '20px',
				'hh-6': '24px'
			},
			borderRadius: {
				'hh-xs': '4px',
				'hh-sm': '6px',
				'hh-control': '7px',
				'hh-md': '8px',
				'hh-lg': '14px',
				'hh-xl': '18px',
				'hh-pill': '999px',
				'hh-round': '50%'
			},
			boxShadow: {
				'hh-sidebar':
					'inset -1px 0 0 rgba(255, 255, 255, 0.03), 18px 0 48px rgba(0, 0, 0, 0.28)',
				'hh-panel': 'inset 0 1px 0 rgba(255, 255, 255, 0.035)',
				'hh-modal': '0 24px 80px rgba(0, 0, 0, 0.55)'
			},
			minWidth: {
				'hh-shell': '800px',
				'hh-shell-content': '0px',
				'hh-shell-content-compact': '0px'
			},
			minHeight: {
				'hh-shell': '600px'
			},
			width: {
				'hh-sidebar': '224px',
				'hh-sidebar-compact': '208px',
				'hh-sidebar-rail': '64px'
			}
		}
	},
	plugins: []
} satisfies Config
```

### `frontend/vite.config.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/vite.config.ts`
- Size bytes / Размер в байтах: `588`
- Included characters / Включено символов: `588`
- Truncated / Обрезано: `no`

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
	plugins: [vue()],
	resolve: {
		alias: {
			'@': resolve(__dirname, 'src')
		}
	},
	server: {
		port: 5173
	},
	build: {
		outDir: 'dist',
		chunkSizeWarningLimit: 1536,
		rollupOptions: {
			onwarn(warning, defaultHandler) {
				const isIgnoredAnnotationWarning = warning.message.includes('INVALID_ANNOTATION') &&
					warning.message.includes('@vueuse/core');
				if (!isIgnoredAnnotationWarning) {
					defaultHandler(warning);
				}
			}
		}
	}
})
```

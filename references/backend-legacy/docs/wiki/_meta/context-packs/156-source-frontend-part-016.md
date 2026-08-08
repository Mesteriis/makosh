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

- Chunk ID / ID чанка: `156-source-frontend-part-016`
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

### `frontend/src/platform/i18n/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/i18n/index.ts`
- Size bytes / Размер в байтах: `1634`
- Included characters / Включено символов: `1634`
- Truncated / Обрезано: `no`

```typescript
import { ref } from 'vue'
import type { Locale, TranslationFunction, Dictionary } from './types'
import ru from './ru.json'
import en from './en.json'

/** Reactive current locale (persisted to localStorage). */
const currentLocale = ref<Locale>(loadLocale())
const dictionaries: Record<Locale, Dictionary> = { ru, en }

function loadLocale(): Locale {
	try {
		const stored = localStorage.getItem('hh-locale')
		if (stored === 'ru' || stored === 'en') return stored
	} catch {
		// localStorage unavailable
	}
	// Default to Russian
	return 'ru'
}

function persistLocale(locale: Locale) {
	try {
		localStorage.setItem('hh-locale', locale)
	} catch {
		// ignore
	}
}

/**
 * Set the active locale and persist to localStorage.
 */
export function setLocale(locale: Locale) {
	currentLocale.value = locale
	persistLocale(locale)
}

/**
 * Vue composable: returns `t(key)` function that looks up the key
 * in the current locale dictionary.
 *
 * - If the key exists in the active dictionary, returns the translated value.
 * - If the key does not exist, returns the key itself as fallback (identity).
 * - Supports `{param}` interpolation: `t("Hello {name}", { name: "Alex" })`.
 */
export function useI18n(): {
	t: TranslationFunction
	locale: typeof currentLocale
	setLocale: typeof setLocale
} {
	const t: TranslationFunction = (key, params) => {
		const dict = dictionaries[currentLocale.value as Locale]
		let value = dict?.[key] ?? key
		if (params) {
			for (const [k, v] of Object.entries(params)) {
				value = value.replace(`{${k}}`, String(v))
			}
		}
		return value
	}

	return { t, locale: currentLocale, setLocale }
}
```

### `frontend/src/platform/i18n/types.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/i18n/types.ts`
- Size bytes / Размер в байтах: `183`
- Included characters / Включено символов: `183`
- Truncated / Обрезано: `no`

```typescript
export type Locale = 'ru' | 'en'

export type TranslationFunction = (key: string, params?: Record<string, string | number>) => string

export type Dictionary = Record<string, string>
```

### `frontend/src/platform/settings/applicationSettingsClient.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/settings/applicationSettingsClient.ts`
- Size bytes / Размер в байтах: `1417`
- Included characters / Включено символов: `1417`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../api/ApiClient'

export type SettingValueKind = 'boolean' | 'integer' | 'string' | 'json'

export type ApplicationSettingValue = boolean | number | string | Record<string, unknown> | unknown[]

export interface ApplicationSetting {
	setting_key: string
	category: string
	value_kind: SettingValueKind
	value: ApplicationSettingValue
	label: string
	description: string
	metadata: Record<string, unknown>
	is_editable: boolean
	updated_by_actor_id: string | null
	created_at: string
	updated_at: string
}

export interface ApplicationSettingsResponse {
	items: ApplicationSetting[]
}

export const FRONTEND_LAYOUT_SETTING_KEY = 'frontend.layout'
export const FRONTEND_SIDEBAR_SETTING_KEY = 'frontend.sidebar'
export const FRONTEND_LOCALE_SETTING_KEY = 'frontend.locale'
export const FRONTEND_THEME_SETTING_KEY = 'frontend.theme'
export const FRONTEND_UI_STATE_SETTING_KEY = 'frontend.ui_state'

export async function fetchApplicationSettings(): Promise<ApplicationSettingsResponse> {
	return ApiClient.instance.get<ApplicationSettingsResponse>(
		'/api/v1/settings',
		'Settings request failed'
	)
}

export async function saveApplicationSetting(
	settingKey: string,
	value: ApplicationSetting['value']
): Promise<ApplicationSetting> {
	return ApiClient.instance.put<ApplicationSetting>(
		`/api/v1/settings/${encodeURIComponent(settingKey)}`,
		{ value },
		'Setting update failed'
	)
}
```

### `frontend/src/platform/sse/SseClient.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/sse/SseClient.test.ts`
- Size bytes / Размер в байтах: `6026`
- Included characters / Включено символов: `6026`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, describe, expect, it, vi } from 'vitest'
import { SseClient } from './SseClient'

describe('SseClient', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it('reports protected SSE transport status transitions', async () => {
    vi.useFakeTimers()
    const statuses: string[] = []
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 42\nevent: event\ndata: {"ok":true}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(stream, {
        status: 200,
        headers: { 'Content-Type': 'text/event-stream' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<void>((resolve) => {
      const client = new SseClient({
        url: 'http://127.0.0.1:8080/api/events/stream',
        secret: 'test-secret',
        reconnectDelay: 60_000,
        onStatus: (status) => statuses.push(`${status.transport}:${status.state}`),
        onMessage: () => {
          client.disconnect()
          resolve()
        }
      })
      client.connect()
    })

    await received

    expect(statuses).toEqual(['sse:connecting', 'sse:connected', 'sse:disconnected'])
  })

  it('connects with the local API secret and replays from the last event id', async () => {
    vi.useFakeTimers()
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 42\nevent: event\ndata: {"ok":true}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(stream, {
        status: 200,
        headers: { 'Content-Type': 'text/event-stream' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<{ id: string; event: string; data: string }>((resolve) => {
      const client = new SseClient({
        url: 'http://127.0.0.1:8080/api/events/stream',
        secret: 'test-secret',
        lastEventId: '41',
        reconnectDelay: 60_000,
        onMessage: (event) => {
          resolve(event)
          client.disconnect()
        }
      })
      client.connect()
    })

    await expect(received).resolves.toEqual({
      id: '42',
      event: 'event',
      data: '{"ok":true}'
    })
    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:8080/api/events/stream?after_position=41')
    expect(init.headers).toMatchObject({
      Accept: 'text/event-stream',
      'X-Макошь-Secret': 'test-secret',
      'Last-Event-ID': '41'
    })
  })

  it('builds replay requests for relative stream URLs', async () => {
    vi.useFakeTimers()
    vi.stubGlobal('location', { origin: 'http://127.0.0.1:5174' })
    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(new TextEncoder().encode('id: 10\nevent: event\ndata: {}\n\n'))
        controller.close()
      }
    })
    const fetchMock = vi.fn().mockResolvedValue(new Response(stream, { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<void>((resolve) => {
      const client = new SseClient({
        url: '/api/events/stream?source=frontend',
        secret: 'test-secret',
        lastEventId: '9',
        reconnectDelay: 60_000,
        onMessage: () => {
          client.disconnect()
          resolve()
        }
      })
      client.connect()
    })

    await received
    const [url] = fetchMock.mock.calls[0]
    expect(url).toBe('http://127.0.0.1:5174/api/events/stream?source=frontend&after_position=9')
  })

  it('falls back to protected long polling after SSE reconnect attempts are exhausted', async () => {
    vi.useFakeTimers()
    const statuses: string[] = []
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response('stream unavailable', { status: 503 }))
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            items: [
              {
                position: 42,
                event: {
                  event_id: 'evt-long-poll',
                  event_type: 'mail.ai_state.changed',
                  payload: { ai_state: 'PROCESSING' }
                }
              }
            ],
            next_after_position: 42,
            has_more: false
          }),
          {
            status: 200,
            headers: { 'Content-Type': 'application/json' }
          }
        )
      )
    vi.stubGlobal('fetch', fetchMock)

    const received = new Promise<{ id: string; event: string; data: string }>((resolve) => {
      const client = new SseClient({
        url: 'http://127.0.0.1:8080/api/events/stream',
        longPollUrl: 'http://127.0.0.1:8080/api/v1/events',
        secret: 'test-secret',
        lastEventId: '41',
        maxReconnectAttempts: 0,
        longPollDelay: 60_000,
        longPollWaitSeconds: 15,
        onStatus: (status) => statuses.push(`${status.transport}:${status.state}`),
        onMessage: (event) => {
          resolve(event)
          client.disconnect()
        }
      })
      client.connect()
    })

    await expect(received).resolves.toEqual({
      id: '42',
      event: 'event',
      data: JSON.stringify({
        position: 42,
        event: {
          event_id: 'evt-long-poll',
          event_type: 'mail.ai_state.changed',
          payload: { ai_state: 'PROCESSING' }
        }
      })
    })
    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/api/v1/events?after_position=41&limit=100&wait_seconds=15'
    )
    expect(fetchMock.mock.calls[1][1].headers).toMatchObject({
      Accept: 'application/json',
      'X-Макошь-Secret': 'test-secret',
      'Last-Event-ID': '41'
    })
    expect(statuses).toContain('long_poll:fallback')
    expect(statuses).toContain('long_poll:connected')
  })
})
```

### `frontend/src/platform/sse/SseClient.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/sse/SseClient.ts`
- Size bytes / Размер в байтах: `10345`
- Included characters / Включено символов: `10345`
- Truncated / Обрезано: `no`

```typescript
export type SseEventHandler = (event: SseMessageEvent) => void
export type SseErrorHandler = (error: unknown) => void
export type SseTransport = 'sse' | 'long_poll'
export type SseConnectionState =
	| 'connecting'
	| 'connected'
	| 'reconnecting'
	| 'fallback'
	| 'disconnected'
export type SseStatusEvent = {
	transport: SseTransport
	state: SseConnectionState
	attempt?: number
	maxAttempts?: number
	error?: string
}
export type SseStatusHandler = (status: SseStatusEvent) => void

export type SseMessageEvent = {
	id: string
	event: string
	data: string
}

export interface SseClientOptions {
	url: string
	longPollUrl?: string
	secret: string
	lastEventId?: string
	onMessage?: SseEventHandler
	onError?: SseErrorHandler
	onStatus?: SseStatusHandler
	reconnectDelay?: number
	maxReconnectAttempts?: number
	longPollDelay?: number
	longPollBatchSize?: number
	longPollWaitSeconds?: number
	fetchImpl?: typeof fetch
}

type LongPollEventItem = {
	position: number | string
	event: unknown
}

type LongPollResponse = {
	items?: LongPollEventItem[]
	next_after_position?: number
	has_more?: boolean
}

/**
 * Fetch-based SSE client. Browser EventSource cannot send X-Макошь-Secret,
 * so protected local streams must be consumed through a readable fetch body.
 */
export class SseClient {
	private url: string
	private longPollUrl?: string
	private secret: string
	private lastEventId: string
	private onMessage?: SseEventHandler
	private onError?: SseErrorHandler
	private reconnectDelay: number
	private maxReconnectAttempts: number
	private longPollDelay: number
	private longPollBatchSize: number
	private longPollWaitSeconds: number
	private currentTransport: SseTransport = 'sse'
	private reconnectAttempts = 0
	private shouldReconnect = true
	private longPollRunning = false
	private abortController: AbortController | null = null
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null
	private buffer = ''
	private fetchImpl: typeof fetch
	private onStatus?: SseStatusHandler

	constructor(options: SseClientOptions) {
		this.url = options.url
		this.longPollUrl = options.longPollUrl
		this.secret = options.secret.trim()
		if (!this.secret) {
			throw new Error('X-Макошь-Secret cannot be empty')
		}
		this.lastEventId = options.lastEventId?.trim() ?? ''
		this.onMessage = options.onMessage
		this.onError = options.onError
		this.onStatus = options.onStatus
		this.reconnectDelay = options.reconnectDelay ?? 3000
		this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10
		this.longPollDelay = options.longPollDelay ?? 3000
		this.longPollBatchSize = options.longPollBatchSize ?? 100
		this.longPollWaitSeconds = options.longPollWaitSeconds ?? 15
		this.fetchImpl = options.fetchImpl ?? globalThis.fetch.bind(globalThis)
	}

	connect(): void {
		this.stopCurrentConnection()
		this.currentTransport = 'sse'
		this.shouldReconnect = true
		this.reconnectAttempts = 0
		this.reportStatus({ transport: 'sse', state: 'connecting' })
		void this.connectOnce()
	}

	disconnect(): void {
		this.shouldReconnect = false
		this.stopCurrentConnection()
		this.reportStatus({ transport: this.currentTransport, state: 'disconnected' })
	}

	private async connectOnce(): Promise<void> {
		this.abortController = new AbortController()
		try {
			const response = await this.fetchImpl(this.replayUrl(), {
				method: 'GET',
				headers: this.sseHeaders(),
				cache: 'no-store',
				signal: this.abortController.signal
			})

			if (!response.ok) {
				throw new Error(`SSE connection failed with HTTP ${response.status}`)
			}
			if (!response.body) {
				throw new Error('SSE response body is unavailable')
			}

			this.currentTransport = 'sse'
			this.reportStatus({ transport: 'sse', state: 'connected' })
			await this.readSseBody(response.body)
			if (this.shouldReconnect) {
				this.scheduleReconnect()
			}
		} catch (error) {
			if (!this.shouldReconnect) return
			this.onError?.(error)
			this.scheduleReconnect(error)
		}
	}

	private async readSseBody(body: ReadableStream<Uint8Array>): Promise<void> {
		const reader = body.getReader()
		const decoder = new TextDecoder()

		try {
			while (this.shouldReconnect) {
				const { done, value } = await reader.read()
				if (done) break
				this.buffer += decoder.decode(value, { stream: true })
				this.dispatchBufferedEvents()
			}

			this.buffer += decoder.decode()
			this.dispatchBufferedEvents(true)
		} finally {
			reader.releaseLock()
		}
	}

	private dispatchBufferedEvents(final = false): void {
		this.buffer = this.buffer.replaceAll('\r\n', '\n').replaceAll('\r', '\n')
		let boundary = this.buffer.indexOf('\n\n')
		while (boundary !== -1) {
			const block = this.buffer.slice(0, boundary)
			this.buffer = this.buffer.slice(boundary + 2)
			this.dispatchEventBlock(block)
			boundary = this.buffer.indexOf('\n\n')
		}

		if (final && this.buffer.trim()) {
			this.dispatchEventBlock(this.buffer)
			this.buffer = ''
		}
	}

	private dispatchEventBlock(block: string): void {
		let id = ''
		let event = 'message'
		const data: string[] = []

		for (const line of block.split('\n')) {
			if (!line || line.startsWith(':')) continue
			const separator = line.indexOf(':')
			const field = separator === -1 ? line : line.slice(0, separator)
			let value = separator === -1 ? '' : line.slice(separator + 1)
			if (value.startsWith(' ')) value = value.slice(1)

			if (field === 'id') {
				id = value
			} else if (field === 'event') {
				event = value || 'message'
			} else if (field === 'data') {
				data.push(value)
			}
		}

		if (id) {
			this.lastEventId = id
		}
		if (!id && event === 'message' && data.length === 0) {
			return
		}

		this.reconnectAttempts = 0
		this.onMessage?.({
			id,
			event,
			data: data.join('\n')
		})
	}

	private replayUrl(): string {
		if (!this.lastEventId) return this.url
		const parsed = new URL(this.url, globalThis.location?.origin ?? 'http://localhost')
		parsed.searchParams.set('after_position', this.lastEventId)
		return parsed.toString()
	}

	private sseHeaders(): Record<string, string> {
		const headers: Record<string, string> = {
			Accept: 'text/event-stream',
			'X-Макошь-Secret': this.secret
		}
		if (this.lastEventId) {
			headers['Last-Event-ID'] = this.lastEventId
		}
		return headers
	}

	private scheduleReconnect(error?: unknown): void {
		if (!this.shouldReconnect) return
		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			if (this.longPollUrl) {
				this.startLongPolling()
			} else {
				this.reportStatus({
					transport: 'sse',
					state: 'disconnected',
					error: error ? statusErrorMessage(error) : undefined
				})
				this.onError?.(new Error('SSE reconnect attempts exhausted'))
			}
			return
		}

		this.reconnectAttempts += 1
		this.reportStatus({
			transport: 'sse',
			state: 'reconnecting',
			attempt: this.reconnectAttempts,
			maxAttempts: this.maxReconnectAttempts,
			error: error ? statusErrorMessage(error) : undefined
		})
		const delay = this.reconnectDelay * Math.min(this.reconnectAttempts, 5)
		this.reconnectTimer = setTimeout(() => {
			void this.connectOnce()
		}, delay)
	}

	private startLongPolling(): void {
		if (this.longPollRunning || !this.longPollUrl) return
		this.currentTransport = 'long_poll'
		this.reportStatus({ transport: 'long_poll', state: 'fallback' })
		void this.longPollLoop()
	}

	private async longPollLoop(): Promise<void> {
		this.longPollRunning = true
		try {
			while (this.shouldReconnect && this.longPollUrl) {
				try {
					await this.longPollOnce()
				} catch (error) {
					if (!this.shouldReconnect) return
					this.reportStatus({
						transport: 'long_poll',
						state: 'reconnecting',
						error: statusErrorMessage(error)
					})
					this.onError?.(error)
				}

				if (this.shouldReconnect) {
					await this.wait(this.longPollDelay)
				}
			}
		} finally {
			this.longPollRunning = false
		}
	}

	private async longPollOnce(): Promise<void> {
		this.abortController = new AbortController()
		const response = await this.fetchImpl(this.longPollReplayUrl(), {
			method: 'GET',
			headers: this.longPollHeaders(),
			cache: 'no-store',
			signal: this.abortController.signal
		})

		if (!response.ok) {
			throw new Error(`Long polling failed with HTTP ${response.status}`)
		}

		const payload = (await response.json()) as LongPollResponse
		if (!Array.isArray(payload.items)) {
			throw new Error('Long polling response missing items')
		}

		this.currentTransport = 'long_poll'
		this.reportStatus({ transport: 'long_poll', state: 'connected' })
		for (const item of payload.items) {
			this.dispatchLongPollItem(item)
			if (!this.shouldReconnect) break
		}
	}

	private dispatchLongPollItem(item: LongPollEventItem): void {
		const position = Number(item.position)
		if (!Number.isFinite(position) || position < 0) {
			throw new Error('Long polling event position is invalid')
		}

		const id = String(position)
		this.lastEventId = id
		this.reconnectAttempts = 0
		this.onMessage?.({
			id,
			event: 'event',
			data: JSON.stringify(item)
		})
	}

	private longPollReplayUrl(): string {
		const parsed = new URL(
			this.longPollUrl ?? '',
			globalThis.location?.origin ?? 'http://localhost'
		)
		parsed.searchParams.set('after_position', this.lastEventId || '0')
		parsed.searchParams.set('limit', String(this.longPollBatchSize))
		parsed.searchParams.set('wait_seconds', String(this.longPollWaitSeconds))
		return parsed.toString()
	}

	private longPollHeaders(): Record<string, string> {
		const headers: Record<string, string> = {
			Accept: 'application/json',
			'X-Макошь-Secret': this.secret
		}
		if (this.lastEventId) {
			headers['Last-Event-ID'] = this.lastEventId
		}
		return headers
	}

	private wait(delay: number): Promise<void> {
		return new Promise((resolve) => {
			this.reconnectTimer = setTimeout(resolve, delay)
		})
	}

	private stopCurrentConnection(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer)
			this.reconnectTimer = null
		}
		if (this.abortController) {
			this.abortController.abort()
			this.abortController = null
		}
	}

	private reportStatus(status: SseStatusEvent): void {
		this.onStatus?.(status)
	}
}

function statusErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	if (typeof error === 'string') return error
	return 'Unknown realtime transport error'
}
```

### `frontend/src/platform/sse/WebSocketClient.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/sse/WebSocketClient.test.ts`
- Size bytes / Размер в байтах: `1735`
- Included characters / Включено символов: `1735`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, describe, expect, it, vi } from 'vitest'
import { WebSocketClient } from './WebSocketClient'

type ListenerMap = Record<string, Array<(event?: { data?: unknown }) => void>>

class FakeWebSocket {
  static instances: FakeWebSocket[] = []

  url: string
  listeners: ListenerMap = {}
  closed = false

  constructor(url: string) {
    this.url = url
    FakeWebSocket.instances.push(this)
  }

  addEventListener(type: string, listener: (event?: { data?: unknown }) => void): void {
    this.listeners[type] ??= []
    this.listeners[type].push(listener)
  }

  close(): void {
    this.closed = true
  }

  emit(type: string, data?: unknown): void {
    for (const listener of this.listeners[type] ?? []) {
      listener(data === undefined ? undefined : { data })
    }
  }
}

describe('WebSocketClient', () => {
  afterEach(() => {
    FakeWebSocket.instances = []
    vi.unstubAllGlobals()
  })

  it('forwards websocket lagged payloads as replay-gap events instead of unknown-message errors', () => {
    vi.stubGlobal('WebSocket', FakeWebSocket as unknown as typeof WebSocket)

    const onMessage = vi.fn()
    const onError = vi.fn()
    const client = new WebSocketClient({
      url: 'ws://127.0.0.1:8080/api/events/ws',
      secret: 'test-secret',
      lastEventId: '41',
      onMessage,
      onError
    })

    client.connect()

    const socket = FakeWebSocket.instances[0]
    expect(socket.url).toContain('after_position=41')
    socket.emit('message', JSON.stringify({ type: 'lagged', data: { skipped: 6 } }))

    expect(onError).not.toHaveBeenCalled()
    expect(onMessage).toHaveBeenCalledWith({
      id: '41',
      event: 'lagged',
      data: JSON.stringify({ skipped: 6 })
    })
  })
})
```

### `frontend/src/platform/sse/WebSocketClient.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/sse/WebSocketClient.ts`
- Size bytes / Размер в байтах: `7148`
- Included characters / Включено символов: `7148`
- Truncated / Обрезано: `no`

```typescript
export type WebSocketEventHandler = (event: {
	id: string
	event: string
	data: string
}) => void
export type WebSocketErrorHandler = (error: unknown) => void
export type WebSocketTransportState =
	| 'connecting'
	| 'connected'
	| 'reconnecting'
	| 'disconnected'

export type WebSocketStatusEvent = {
	transport: 'websocket'
	state: WebSocketTransportState
	attempt?: number
	maxAttempts?: number
	error?: string
}

export type WebSocketStatusHandler = (status: WebSocketStatusEvent) => void

export interface WebSocketClientOptions {
	url: string
	secret: string
	lastEventId?: string
	onMessage?: WebSocketEventHandler
	onError?: WebSocketErrorHandler
	onStatus?: WebSocketStatusHandler
	reconnectDelay?: number
	maxReconnectAttempts?: number
}

type WebSocketPayload = {
	position?: number | string
	type?: string
}

type WebSocketEnvelope = {
	type?: string
	data?: WebSocketPayload | string
}

type WebSocketLaggedPayload = {
	skipped?: number
}

/**
 * Browser WebSocket event client with replay cursor persistence and reconnect loop.
 * Uses query parameters for authentication and replay because browsers cannot set
 * custom headers in native WebSocket requests.
 */
export class WebSocketClient {
	private url: string
	private secret: string
	private lastEventId: string
	private onMessage?: WebSocketEventHandler
	private onError?: WebSocketErrorHandler
	private onStatus?: WebSocketStatusHandler
	private reconnectDelay: number
	private maxReconnectAttempts: number
	private reconnectAttempts = 0
	private shouldReconnect = true
	private socket: WebSocket | null = null
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null

	constructor(options: WebSocketClientOptions) {
		this.url = options.url
		this.secret = options.secret.trim()
		if (!this.secret) {
			throw new Error('X-Макошь-Secret cannot be empty')
		}
		this.lastEventId = options.lastEventId?.trim() ?? ''
		this.onMessage = options.onMessage
		this.onError = options.onError
		this.onStatus = options.onStatus
		this.reconnectDelay = options.reconnectDelay ?? 3000
		this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10
	}

	connect(): void {
		this.stop()
		this.shouldReconnect = true
		this.reconnectAttempts = 0
		this.connectOnce()
	}

	disconnect(): void {
		this.shouldReconnect = false
		this.stop()
		this.reportStatus({ transport: 'websocket', state: 'disconnected' })
	}

	private connectOnce(): void {
		if (typeof globalThis.WebSocket !== 'function') {
			const error = new Error('WebSocket is not supported in this runtime')
			this.onError?.(error)
			this.reportStatus({
				transport: 'websocket',
				state: 'disconnected',
				error: error.message
			})
			return
		}

		this.reportStatus({ transport: 'websocket', state: 'connecting' })
		this.socket = new WebSocket(this.replayUrl())

		this.socket.addEventListener('open', () => {
			this.reconnectAttempts = 0
			this.reportStatus({ transport: 'websocket', state: 'connected' })
		})

		this.socket.addEventListener('message', (event) => {
			this.handleMessage(event.data)
		})

		this.socket.addEventListener('error', () => {
			this.handleSocketError(new Error('WebSocket transport error'))
		})

		this.socket.addEventListener('close', () => {
			if (!this.shouldReconnect) {
				return
			}
			this.handleSocketClose()
		})
	}

	private handleMessage(rawMessage: unknown): void {
		if (typeof rawMessage !== 'string') {
			return
		}

		let envelope: WebSocketEnvelope
		try {
			envelope = JSON.parse(rawMessage) as WebSocketEnvelope
		} catch (error) {
			this.onError?.(error)
			return
		}

		if (envelope.type === 'heartbeat') {
			return
		}

		if (envelope.type === 'lagged') {
			const skipped = parseLaggedSkipped(envelope.data)
			if (skipped === null) {
				this.onError?.(new Error('WebSocket lagged payload missing skipped count'))
				return
			}

			this.onMessage?.({
				id: this.lastEventId,
				event: 'lagged',
				data: JSON.stringify({ skipped })
			})
			return
		}

		if (envelope.type !== 'event') {
			const message = `Unknown WebSocket message type ${String(envelope.type ?? 'unknown')}`
			this.onError?.(new Error(message))
			return
		}

		if (envelope.data === undefined) {
			this.onError?.(new Error('WebSocket message missing event payload'))
			return
		}

		const position = parseEnvelopePosition(envelope.data)
		if (!position) {
			this.onError?.(new Error('WebSocket message missing event position'))
			return
		}

		const data =
			typeof envelope.data === 'string' ? envelope.data : JSON.stringify(envelope.data)
		this.lastEventId = position
		this.reconnectAttempts = 0
		this.onMessage?.({
			id: this.lastEventId,
			event: 'event',
			data
		})
	}

	private handleSocketError(error: unknown): void {
		this.onError?.(error)
		if (!this.shouldReconnect) return
		this.scheduleReconnect(error)
	}

	private handleSocketClose(): void {
		this.scheduleReconnect(new Error('WebSocket connection closed'))
	}

	private scheduleReconnect(error: unknown): void {
		if (!this.shouldReconnect) {
			return
		}
		if (this.reconnectTimer) {
			return
		}

		if (this.reconnectAttempts >= this.maxReconnectAttempts) {
			this.reportStatus({
				transport: 'websocket',
				state: 'disconnected',
				error: statusErrorMessage(error)
			})
			this.onError?.(new Error('WebSocket reconnect attempts exhausted'))
			return
		}

		this.reconnectAttempts += 1
		this.reportStatus({
			transport: 'websocket',
			state: 'reconnecting',
			attempt: this.reconnectAttempts,
			maxAttempts: this.maxReconnectAttempts,
			error: statusErrorMessage(error)
		})

		const delay = this.reconnectDelay * Math.min(this.reconnectAttempts, 5)
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null
			this.connectOnce()
		}, delay)
	}

	private replayUrl(): string {
		const parsed = new URL(this.url, globalThis.location?.origin ?? 'http://localhost')
		if (this.lastEventId) {
			parsed.searchParams.set('after_position', this.lastEventId)
		}
		parsed.searchParams.set('makosh_secret', this.secret)
		return parsed.toString()
	}

	private stop(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer)
			this.reconnectTimer = null
		}
		if (this.socket) {
			this.socket.close()
			this.socket = null
		}
	}

	private reportStatus(status: WebSocketStatusEvent): void {
		this.onStatus?.(status)
	}
}

function parseEnvelopePosition(data: WebSocketPayload | string): string | null {
	if (typeof data === 'string') return null
	if (typeof data.position === 'undefined') return null

	const position = Number(data.position)
	if (!Number.isFinite(position) || position < 0) {
		return null
	}

	return String(Math.floor(position))
}

function parseLaggedSkipped(data: WebSocketPayload | string | undefined): number | null {
	if (!data || typeof data === 'string') return null

	const payload = data as WebSocketLaggedPayload
	return typeof payload.skipped === 'number' && payload.skipped > 0 ? payload.skipped : null
}

function statusErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message
	if (typeof error === 'string') return error
	return 'Unknown realtime transport error'
}
```

### `frontend/src/platform/sse/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/sse/index.ts`
- Size bytes / Размер в байтах: `432`
- Included characters / Включено символов: `432`
- Truncated / Обрезано: `no`

```typescript
export { SseClient } from './SseClient'
export type {
	SseClientOptions,
	SseConnectionState,
	SseErrorHandler,
	SseEventHandler,
	SseMessageEvent,
	SseStatusEvent,
	SseStatusHandler,
	SseTransport
} from './SseClient'
export { WebSocketClient } from './WebSocketClient'
export type {
	WebSocketClientOptions,
	WebSocketErrorHandler,
	WebSocketEventHandler,
	WebSocketStatusEvent,
	WebSocketStatusHandler
} from './WebSocketClient'
```

### `frontend/src/platform/theme/persistence.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/theme/persistence.test.ts`
- Size bytes / Размер в байтах: `1925`
- Included characters / Включено символов: `1925`
- Truncated / Обрезано: `no`

```typescript
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
	fetchApplicationSettings,
	saveApplicationSetting
} from '../settings/applicationSettingsClient'
import { defaultThemeSettings } from './settings'
import {
	loadPersistedThemeSettings,
	savePersistedThemeSettings
} from './persistence'

vi.mock('../settings/applicationSettingsClient', () => ({
	FRONTEND_THEME_SETTING_KEY: 'frontend.theme',
	fetchApplicationSettings: vi.fn(),
	saveApplicationSetting: vi.fn()
}))

const storage = new Map<string, string>()

const localStorageDouble = {
	getItem: vi.fn((key: string) => storage.get(key) ?? null),
	setItem: vi.fn((key: string, value: string) => {
		storage.set(key, value)
	})
}

describe('theme persistence', () => {
	beforeEach(() => {
		storage.clear()
		vi.clearAllMocks()
		vi.stubGlobal('localStorage', localStorageDouble)
	})

	it('reports backend save fallback instead of hiding it as autosave success', async () => {
		const settings = {
			...defaultThemeSettings(),
			accentColor: 'violet' as const
		}
		vi.mocked(saveApplicationSetting).mockRejectedValue(new Error('offline'))

		const result = await savePersistedThemeSettings(settings)

		expect(result.source).toBe('local_storage')
		expect(result.errorMessage).toContain('saved locally only')
		expect(JSON.parse(storage.get('makosh-theme-settings') ?? '{}')).toMatchObject({
			accentColor: 'violet'
		})
	})

	it('reports backend load fallback while keeping locally stored settings usable', async () => {
		storage.set(
			'makosh-theme-settings',
			JSON.stringify({
				...defaultThemeSettings(),
				accentColor: 'cyan'
			})
		)
		vi.mocked(fetchApplicationSettings).mockRejectedValue(new Error('offline'))

		const result = await loadPersistedThemeSettings()

		expect(result.source).toBe('local_storage')
		expect(result.errorMessage).toContain('backend unavailable')
		expect(result.settings.accentColor).toBe('cyan')
	})
})
```

### `frontend/src/platform/theme/persistence.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/theme/persistence.ts`
- Size bytes / Размер в байтах: `2360`
- Included characters / Включено символов: `2360`
- Truncated / Обрезано: `no`

```typescript
import {
	FRONTEND_THEME_SETTING_KEY,
	fetchApplicationSettings,
	saveApplicationSetting
} from '../settings/applicationSettingsClient'
import { defaultThemeSettings, parseThemeSettings, type ThemeSettings } from './settings'

const LOCAL_STORAGE_KEY = 'makosh-theme-settings'

export type ThemePersistenceSource = 'application_settings' | 'local_storage'

export interface PersistedThemeSettings {
	settings: ThemeSettings
	source: ThemePersistenceSource
	errorMessage: string
}

const LOAD_FALLBACK_MESSAGE = 'Theme settings backend unavailable; using local browser settings.'
const SAVE_FALLBACK_MESSAGE = 'Theme saved locally only. Application settings backend is unavailable.'

export async function loadPersistedThemeSettings(): Promise<PersistedThemeSettings> {
	try {
		const response = await fetchApplicationSettings()
		const setting = response.items.find((item) => item.setting_key === FRONTEND_THEME_SETTING_KEY)
		if (setting) {
			const parsed = parseThemeSettings(setting.value)
			saveLocalThemeSettings(parsed)
			return {
				settings: parsed,
				source: 'application_settings',
				errorMessage: ''
			}
		}
	} catch {
		return {
			settings: loadLocalThemeSettings(),
			source: 'local_storage',
			errorMessage: LOAD_FALLBACK_MESSAGE
		}
	}

	return {
		settings: loadLocalThemeSettings(),
		source: 'local_storage',
		errorMessage: ''
	}
}

export async function savePersistedThemeSettings(settings: ThemeSettings): Promise<PersistedThemeSettings> {
	try {
		const saved = await saveApplicationSetting(FRONTEND_THEME_SETTING_KEY, settings)
		const parsed = parseThemeSettings(saved.value)
		saveLocalThemeSettings(parsed)
		return {
			settings: parsed,
			source: 'application_settings',
			errorMessage: ''
		}
	} catch {
		saveLocalThemeSettings(settings)
		return {
			settings,
			source: 'local_storage',
			errorMessage: SAVE_FALLBACK_MESSAGE
		}
	}
}

export function loadLocalThemeSettings(): ThemeSettings {
	try {
		const raw = localStorage.getItem(LOCAL_STORAGE_KEY)
		return raw ? parseThemeSettings(JSON.parse(raw)) : defaultThemeSettings()
	} catch {
		return defaultThemeSettings()
	}
}

function saveLocalThemeSettings(settings: ThemeSettings): void {
	try {
		localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(settings))
	} catch {
		// localStorage may be unavailable; runtime theme still applies in memory.
	}
}
```

### `frontend/src/platform/theme/settings.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/theme/settings.test.ts`
- Size bytes / Размер в байтах: `1664`
- Included characters / Включено символов: `1664`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
	defaultThemeSettings,
	parseThemeSettings,
	shellAccentClass,
	shellBackgroundClass,
	shellBrightnessClass,
	shellPanelBlurClass,
	shellPanelOpacityClass,
	shellSpacingDensityClass
} from './settings'

describe('theme settings', () => {
	it('returns defaults for invalid values', () => {
		expect(parseThemeSettings(null)).toEqual(defaultThemeSettings())
		expect(parseThemeSettings({ schemaVersion: 99 })).toEqual(defaultThemeSettings())
	})

	it('keeps allowlisted values', () => {
		expect(
			parseThemeSettings({
				schemaVersion: 1,
				shellBackground: 'rune-teal',
				backgroundBrightness: 90,
				accentColor: 'cyan',
				panelOpacity: 50,
				panelBlur: 20,
				spacingDensity: 'compact'
			})
		).toEqual({
			schemaVersion: 1,
			shellBackground: 'rune-teal',
			backgroundBrightness: 90,
			accentColor: 'cyan',
			panelOpacity: 50,
			panelBlur: 20,
			spacingDensity: 'compact'
		})
	})

	it('returns allowlisted CSS classes', () => {
		const settings = parseThemeSettings({
			schemaVersion: 1,
			shellBackground: 'network-mesh',
			backgroundBrightness: 70,
			accentColor: 'violet',
			panelOpacity: 80,
			panelBlur: 12,
			spacingDensity: 'comfortable'
		})

		expect(shellBackgroundClass(settings)).toBe('shell-bg-network-mesh')
		expect(shellBrightnessClass(settings)).toBe('shell-bg-brightness-70')
		expect(shellAccentClass(settings)).toBe('theme-accent-violet')
		expect(shellPanelOpacityClass(settings)).toBe('panel-opacity-80')
		expect(shellPanelBlurClass(settings)).toBe('panel-blur-12')
		expect(shellSpacingDensityClass(settings)).toBe('spacing-density-comfortable')
	})
})
```

### `frontend/src/platform/theme/settings.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/theme/settings.ts`
- Size bytes / Размер в байтах: `3509`
- Included characters / Включено символов: `3509`
- Truncated / Обрезано: `no`

```typescript
export const THEME_SCHEMA_VERSION = 1

export const shellBackgroundIds = [
	'none',
	'network-mesh',
	'data-stream',
	'node-frame',
	'eclipse-grid',
	'dna-blueprint',
	'forest-network',
	'forest-stream',
	'knowledge-map',
	'rune-gold',
	'rune-teal'
] as const

export const backgroundBrightnessValues = [30, 40, 50, 60, 70, 80, 90, 100] as const
export const accentColorIds = ['teal', 'cyan', 'blue', 'violet', 'amber', 'rose'] as const
export const panelOpacityValues = [40, 50, 60, 70, 80, 90, 100] as const
export const panelBlurValues = [0, 4, 8, 12, 16, 20, 24] as const
export const spacingDensityIds = ['compact', 'normal', 'comfortable'] as const

export type ShellBackgroundId = (typeof shellBackgroundIds)[number]
export type BackgroundBrightness = (typeof backgroundBrightnessValues)[number]
export type AccentColorId = (typeof accentColorIds)[number]
export type PanelOpacity = (typeof panelOpacityValues)[number]
export type PanelBlur = (typeof panelBlurValues)[number]
export type SpacingDensity = (typeof spacingDensityIds)[number]

export type ThemeSettings = {
	schemaVersion: typeof THEME_SCHEMA_VERSION
	shellBackground: ShellBackgroundId
	backgroundBrightness: BackgroundBrightness
	accentColor: AccentColorId
	panelOpacity: PanelOpacity
	panelBlur: PanelBlur
	spacingDensity: SpacingDensity
}

export function defaultThemeSettings(): ThemeSettings {
	return {
		schemaVersion: THEME_SCHEMA_VERSION,
		shellBackground: 'network-mesh',
		backgroundBrightness: 70,
		accentColor: 'teal',
		panelOpacity: 70,
		panelBlur: 12,
		spacingDensity: 'normal'
	}
}

export function parseThemeSettings(value: unknown): ThemeSettings {
	if (!isRecord(value) || value.schemaVersion !== THEME_SCHEMA_VERSION) {
		return defaultThemeSettings()
	}

	const defaults = defaultThemeSettings()
	return {
		schemaVersion: THEME_SCHEMA_VERSION,
		shellBackground: pick(value.shellBackground, shellBackgroundIds, defaults.shellBackground),
		backgroundBrightness: pick(
			value.backgroundBrightness,
			backgroundBrightnessValues,
			defaults.backgroundBrightness
		),
		accentColor: pick(value.accentColor, accentColorIds, defaults.accentColor),
		panelOpacity: pick(value.panelOpacity, panelOpacityValues, defaults.panelOpacity),
		panelBlur: pick(value.panelBlur, panelBlurValues, defaults.panelBlur),
		spacingDensity: pick(value.spacingDensity, spacingDensityIds, defaults.spacingDensity)
	}
}

export function shellBackgroundClass(settings: ThemeSettings): string {
	return `shell-bg-${settings.shellBackground}`
}

export function shellBrightnessClass(settings: ThemeSettings): string {
	return `shell-bg-brightness-${settings.backgroundBrightness}`
}

export function shellAccentClass(settings: ThemeSettings): string {
	return `theme-accent-${settings.accentColor}`
}

export function shellPanelOpacityClass(settings: ThemeSettings): string {
	return `panel-opacity-${settings.panelOpacity}`
}

export function shellPanelBlurClass(settings: ThemeSettings): string {
	return `panel-blur-${settings.panelBlur}`
}

export function shellSpacingDensityClass(settings: ThemeSettings): string {
	return `spacing-density-${settings.spacingDensity}`
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function pick<const T extends readonly (string | number)[]>(
	value: unknown,
	allowed: T,
	fallback: T[number]
): T[number] {
	return allowed.includes(value as T[number]) ? (value as T[number]) : fallback
}
```

### `frontend/src/platform/theme/tokens.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/theme/tokens.ts`
- Size bytes / Размер в байтах: `998`
- Included characters / Включено символов: `998`
- Truncated / Обрезано: `no`

```typescript
/** Typed Макошь design token constants for use outside Tailwind classes */
export const theme = {
	font: {
		sans: [
			'Inter',
			'SF Pro Display',
			'ui-sans-serif',
			'system-ui',
			'-apple-system',
			'BlinkMacSystemFont',
			'Segoe UI',
			'sans-serif'
		].join(', ')
	},
	color: {
		bg: '#02090b',
		bgRaised: '#020d10',
		surface: '#06181b',
		surfaceDeep: '#041215',
		text: '#eefefb',
		textStrong: '#f2fffd',
		textBright: '#ffffff',
		textSoft: '#dcefed',
		textMuted: '#91a8a8',
		textSubtle: '#8ea4a6',
		textDim: '#849ca0',
		accent: '#2df0ce',
		accentStrong: '#25d8bd',
		accentSoft: '#9ee8df',
		accentContrast: '#032522',
		danger: '#ffabab',
		dangerStrong: '#ef3140'
	},
	radius: {
		xs: '4px',
		sm: '6px',
		control: '7px',
		md: '8px',
		lg: '14px',
		xl: '18px',
		pill: '999px',
		round: '50%'
	},
	space: {
		'1': '4px',
		'2': '8px',
		'3': '12px',
		'4': '16px',
		'5': '20px',
		'6': '24px'
	},
	layout: {
		row: '37px',
		gap: '10px',
		columns: 12
	}
} as const
```

### `frontend/src/shared/communications/queries/realtimePatchShared.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/queries/realtimePatchShared.ts`
- Size bytes / Размер в байтах: `9578`
- Included characters / Включено символов: `9578`
- Truncated / Обрезано: `no`

```typescript
type WorkflowState = 'new' | 'reviewed' | 'needs_action' | 'waiting' | 'done' | 'archived' | 'muted' | 'spam'
type LocalMessageState = 'active' | 'trash' | 'all'
type BulkMessageAction =
	| 'mark_read'
	| 'mark_unread'
	| 'archive'
	| 'trash'
	| 'restore'
	| 'pin'
	| 'unpin'
	| 'important'
	| 'not_important'
	| 'add_label'
	| 'remove_label'
	| 'snooze'
type CommunicationAiState = 'NEW' | 'PROCESSING' | 'PROCESSED' | 'REVIEW_REQUIRED' | 'FAILED' | 'ARCHIVED'

type CommunicationOutboxItem = {
	status: 'queued' | 'scheduled' | 'sending' | 'sent' | 'failed' | 'canceled'
}

type CacheKeyFilterField = `query${'Key'}`
type CacheKeyFilter = {
	[Field in CacheKeyFilterField]: readonly unknown[]
}

export type CommunicationFolder = {
	folder_id: string
	account_id: string | null
	name: string
	description: string | null
	color: string | null
	sort_order: number
	message_count: number
	created_at: string
	updated_at: string
}

export type FolderMessage = {
	folder_id: string
	message_id: string
	account_id: string
	subject: string
	sender: string
	occurred_at: string | null
	projected_at: string
	workflow_state: WorkflowState
	local_state: LocalMessageState
	added_at: string
	attachment_count: number
}

export type CommunicationSavedSearch = {
	saved_search_id: string
	name: string
	description: string | null
	account_id: string | null
	query: string
	workflow_state: WorkflowState | null
	local_state: LocalMessageState
	channel_kind: string | null
	is_smart_folder: boolean
	sort_order: number
	message_count: number
	created_at: string
	updated_at: string
}

export type MailRealtimePatchQueryClient = {
	getQueriesData?: <TData>(filters: CacheKeyFilter) => Array<[
		readonly unknown[],
		TData | undefined
	]>
	setQueryData?: <TData>(
		key: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export type StoredEventEnvelope = {
	event?: {
		event_type?: unknown
		payload?: unknown
	}
}

export type CommunicationMessagePatchPayload = {
	action?: unknown
	action_parameters?: unknown
	message_ids?: unknown
}

export type OutboxPatchPayload = {
	outbox_id?: unknown
	account_id?: unknown
	status?: unknown
	provider_message_id?: unknown
	last_error?: unknown
	send_attempts?: unknown
	scheduled_send_at?: unknown
	undo_deadline_at?: unknown
	sent_at?: unknown
	delivery_status?: unknown
	smtp_status?: unknown
	source_kind?: unknown
	recorded_at?: unknown
	receipt_id?: unknown
	provider_record_id?: unknown
	receipt_kind?: unknown
	read_at?: unknown
}

export type AiStatePatchPayload = {
	message_id?: unknown
	ai_state?: unknown
	review_required?: unknown
	failed?: unknown
}

export type DraftPatchPayload = {
	draft_id?: unknown
	account_id?: unknown
}

export type FolderMessagePatchPayload = {
	operation?: unknown
	folder_id?: unknown
	message_id?: unknown
	message?: unknown
}

export type SyncPatchPayload = {
	account_id?: unknown
	status?: unknown
	phase?: unknown
	progress_mode?: unknown
	progress_percent?: unknown
	processed_messages?: unknown
	estimated_total_messages?: unknown
	current_batch_size?: unknown
	fetched_messages?: unknown
	projected_messages?: unknown
	upserted_persons?: unknown
	upserted_organizations?: unknown
	error_code?: unknown
	next_run_at?: unknown
}

const AI_STATES = new Set<CommunicationAiState>([
	'NEW',
	'PROCESSING',
	'PROCESSED',
	'REVIEW_REQUIRED',
	'FAILED',
	'ARCHIVED'
])

const WORKFLOW_STATES = new Set<WorkflowState>([
	'new',
	'reviewed',
	'needs_action',
	'waiting',
	'done',
	'archived',
	'muted',
	'spam'
])

const LOCAL_MESSAGE_STATES = new Set<LocalMessageState>(['active', 'trash', 'all'])

const BULK_ACTIONS = new Set<BulkMessageAction>([
	'mark_read',
	'mark_unread',
	'archive',
	'trash',
	'restore',
	'pin',
	'unpin',
	'important',
	'not_important',
	'add_label',
	'remove_label',
	'snooze'
])

export function storedEventEnvelope(eventData: string): StoredEventEnvelope | null {
	try {
		return JSON.parse(eventData) as StoredEventEnvelope
	} catch {
		return null
	}
}

export function stringValue(value: unknown): string | null {
	return typeof value === 'string' && value.trim() ? value.trim() : null
}

export function nullableStringValue(value: unknown): string | null {
	return typeof value === 'string' && value.trim() ? value.trim() : null
}

export function numberValue(value: unknown): number | null {
	const number = Number(value)
	return Number.isFinite(number) ? number : null
}

export function nullableNumberValue(value: unknown): number | null {
	if (value === null || typeof value === 'undefined') return null
	const number = Number(value)
	return Number.isFinite(number) ? number : null
}

export function outboxStatusValue(value: unknown): CommunicationOutboxItem['status'] | null {
	const status = stringValue(value)
	if (
		status === 'queued' ||
		status === 'scheduled' ||
		status === 'sending' ||
		status === 'sent' ||
		status === 'failed' ||
		status === 'canceled'
	) {
		return status
	}
	return null
}

export function aiStateValue(value: unknown): CommunicationAiState | null {
	if (typeof value !== 'string') return null
	return AI_STATES.has(value as CommunicationAiState) ? (value as CommunicationAiState) : null
}

function workflowStateValue(value: unknown): WorkflowState | null {
	if (value === null || typeof value === 'undefined') return null
	if (typeof value !== 'string') return null
	return WORKFLOW_STATES.has(value as WorkflowState) ? (value as WorkflowState) : null
}

function localMessageStateValue(value: unknown): LocalMessageState | null {
	if (typeof value !== 'string') return null
	return LOCAL_MESSAGE_STATES.has(value as LocalMessageState) ? (value as LocalMessageState) : null
}

export function normalizeBulkAction(value: unknown): BulkMessageAction | null {
	if (typeof value !== 'string') return null
	return BULK_ACTIONS.has(value as BulkMessageAction) ? (value as BulkMessageAction) : null
}

export function normalizeMessageIds(value: unknown): string[] {
	if (!Array.isArray(value)) return []
	return value
		.filter((messageId): messageId is string => typeof messageId === 'string')
		.map((messageId) => messageId.trim())
		.filter(Boolean)
}

export function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function folderValue(value: unknown): CommunicationFolder | null {
	if (!isRecord(value)) return null
	const folderId = stringValue(value.folder_id)
	const name = stringValue(value.name)
	const sortOrder = numberValue(value.sort_order)
	const messageCount = numberValue(value.message_count)
	const createdAt = stringValue(value.created_at)
	const updatedAt = stringValue(value.updated_at)
	if (!folderId || !name || sortOrder === null || messageCount === null || !createdAt || !updatedAt) {
		return null
	}

	return {
		folder_id: folderId,
		account_id: nullableStringValue(value.account_id),
		name,
		description: nullableStringValue(value.description),
		color: nullableStringValue(value.color),
		sort_order: sortOrder,
		message_count: messageCount,
		created_at: createdAt,
		updated_at: updatedAt
	}
}

export function folderMessageValue(value: unknown): FolderMessage | null {
	if (!isRecord(value)) return null
	const folderId = stringValue(value.folder_id)
	const messageId = stringValue(value.message_id)
	const accountId = stringValue(value.account_id)
	const subject = typeof value.subject === 'string' ? value.subject : null
	const sender = typeof value.sender === 'string' ? value.sender : null
	const projectedAt = stringValue(value.projected_at)
	const workflowState = workflowStateValue(value.workflow_state)
	const localState = localMessageStateValue(value.local_state)
	const addedAt = stringValue(value.added_at)
	const attachmentCount = numberValue(value.attachment_count)
	if (
		!folderId ||
		!messageId ||
		!accountId ||
		subject === null ||
		sender === null ||
		!projectedAt ||
		!workflowState ||
		!localState ||
		!addedAt ||
		attachmentCount === null
	) {
		return null
	}

	return {
		folder_id: folderId,
		message_id: messageId,
		account_id: accountId,
		subject,
		sender,
		occurred_at: nullableStringValue(value.occurred_at),
		projected_at: projectedAt,
		workflow_state: workflowState,
		local_state: localState,
		added_at: addedAt,
		attachment_count: attachmentCount
	}
}

export function savedSearchValue(value: unknown): CommunicationSavedSearch | null {
	if (!isRecord(value)) return null
	const savedSearchId = stringValue(value.saved_search_id)
	const name = stringValue(value.name)
	const query = typeof value.query === 'string' ? value.query : null
	const localState = localMessageStateValue(value.local_state)
	const sortOrder = numberValue(value.sort_order)
	const messageCount = numberValue(value.message_count)
	const createdAt = stringValue(value.created_at)
	const updatedAt = stringValue(value.updated_at)
	if (
		!savedSearchId ||
		!name ||
		query === null ||
		!localState ||
		sortOrder === null ||
		messageCount === null ||
		typeof value.is_smart_folder !== 'boolean' ||
		!createdAt ||
		!updatedAt
	) {
		return null
	}

	return {
		saved_search_id: savedSearchId,
		name,
		description: nullableStringValue(value.description),
		account_id: nullableStringValue(value.account_id),
		query,
		workflow_state: workflowStateValue(value.workflow_state),
		local_state: localState,
		channel_kind: nullableStringValue(value.channel_kind),
		is_smart_folder: value.is_smart_folder,
		sort_order: sortOrder,
		message_count: messageCount,
		created_at: createdAt,
		updated_at: updatedAt
	}
}
```

### `frontend/src/shared/communications/types/telegram.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/types/telegram.ts`
- Size bytes / Размер в байтах: `17333`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
// --- Provider kinds ---
export type TelegramProviderKind = 'telegram_user' | 'telegram_bot'

// --- Account lifecycle ---
export type TelegramAccountLifecycleState = 'active' | 'logged_out' | 'removed' | string

export type TelegramAccount = {
  account_id: string
  provider_kind: TelegramProviderKind
  display_name: string
  external_account_id: string
  runtime: string
  lifecycle_state: TelegramAccountLifecycleState
  transcription_enabled: boolean
  tdlib_data_path: string | null
  created_at: string
  updated_at: string
}

export type TelegramAccountListResponse = {
  items: TelegramAccount[]
}

export type TelegramAccountSetupResponse = {
  account_id: string
  provider_kind: TelegramProviderKind
  runtime: string
  transcription_enabled: boolean
  credential_bindings: { secret_purpose: string; secret_ref: string; secret_kind: string; store_kind: string }[]
}

export type TelegramAccountLifecycleResponse = {
  account: TelegramAccount
  stopped_runtime_actor: boolean
}

// --- Capabilities ---
export type TelegramCapabilityState = 'available' | 'blocked' | 'degraded' | 'planned' | 'unsupported'
export type TelegramActionClass = 'read' | 'local_write' | 'provider_write' | 'destructive' | 'export' | 'secret_access' | 'automation'

export type TelegramOperationCapability = {
  operation: string
  category: string
  status: TelegramCapabilityState
  action_class: TelegramActionClass
  reason: string
  confirmation_required: boolean
  closure_gate: boolean
}

export type TelegramCapabilityAccountScope = {
  account_id: string
  provider_kind: TelegramProviderKind
  runtime_kind: string
  lifecycle_state: TelegramAccountLifecycleState
}

export type TelegramCapabilitiesResponse = {
  version: string
  runtime_mode: string
  account_scope?: TelegramCapabilityAccountScope | null
  telegram_app_credentials_configured: boolean
  tdjson_runtime_available: boolean
  qr_login_ready: boolean
  bot_runtime_available: boolean
  capabilities: TelegramOperationCapability[]
  planned_features: string[]
  unsupported_features: string[]
}

// --- Runtime ---
export type TelegramRuntimeStatus = {
  account_id: string
  provider_kind: TelegramProviderKind
  runtime_kind: string
  status: 'stopped' | 'running' | 'blocked' | 'degraded' | 'error' | string
  fixture_runtime: boolean
  tdjson_path: string | null
  tdjson_runtime_available: boolean
  tdjson_probe_error: string | null
  telegram_api_id_configured: boolean
  telegram_api_hash_configured: boolean
  telegram_app_credentials_configured: boolean
  live_send_available: boolean
  runtime_blockers: string[]
  last_error: string | null
  last_sync_scope?: string | null
  last_sync_status?: string | null
  last_synced_count?: number | null
  last_sync_has_more?: boolean | null
  last_sync_provider_chat_id?: string | null
  last_command_id?: string | null
  last_command_status?: string | null
  last_command_kind?: string | null
  last_command_provider_chat_id?: string | null
  last_command_message_id?: string | null
  last_command_telegram_chat_id?: string | null
  updated_at: string
}

// --- Chats ---
export type TelegramChatKind = 'private' | 'group' | 'channel' | 'bot'
export type TelegramChatSyncState = 'fixture' | 'syncing' | 'synced' | 'degraded' | 'error'

export type TelegramChat = {
  telegram_chat_id: string
  account_id: string
  provider_chat_id: string
  chat_kind: TelegramChatKind
  title: string
  username: string | null
  sync_state: TelegramChatSyncState
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type TelegramChatListResponse = {
  items: TelegramChat[]
}

export type TelegramChatDetailResponse = {
  item: TelegramChat
}

export type TelegramChatGroupFilterListResponse = {
  items: TelegramChatGroupFilter[]
}

export type {
  TelegramChatMember,
  TelegramChatMemberListResponse,
  TelegramChatMembersSyncResponse
} from './telegramMembers'

export type {
  TelegramChatActionRequest,
  TelegramChatActionResponse,
  TelegramChatFolderReassignRequest,
  TelegramChatFolderReassignResponse,
  TelegramChatLifecycleCommandResponse
} from './telegramChatActions'

// --- Messages ---
export type TelegramMessage = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_message_id: string
  provider_chat_id: string | null
  chat_title: string
  sender: string
  sender_display_name: string | null
  text: string
  occurred_at: string | null
  projected_at: string
  channel_kind: TelegramProviderKind
  delivery_state: string
  metadata: Record<string, unknown>
}

export type TelegramMessageListResponse = {
  items: TelegramMessage[]
}

export type TelegramMessageSearchResponse = {
  query: string
  items: TelegramMessage[]
  total: number
}
export type TelegramChatSearchResponse = {
  query: string
  items: TelegramChat[]
  total: number
}
export type TelegramMediaItem = {
  attachment_id?: string | null
  message_id: string
  provider_message_id: string
  provider_chat_id: string
  file_name: string
  kind: string
  mime_type: string | null
  size_bytes: number | null
  occurred_at: string | null
  download_state: string
  tdlib_file_id: number | null
  provider_attachment_id: string | null
  local_path: string | null
  expected_size_bytes?: number | null
  downloaded_size_bytes?: number | null
  is_downloading_active?: boolean | null
  is_downloading_completed?: boolean | null
  last_error?: string | null
}
export type TelegramMediaSearchResponse = {
  query?: string | null
  source?: 'projection' | 'provider_refresh' | string
  provider_search_attempted?: boolean
  provider_search_error?: string | null
  items: TelegramMediaItem[]
}
// --- Sync ---
export type TelegramChatSyncRequest = {
  account_id: string
  limit?: number
}

export type TelegramChatSyncResponse = {
  account_id: string
  runtime_kind: string
  status: string
  synced_count: number
  items: TelegramChat[]
}

export type TelegramHistorySyncRequest = {
  account_id: string
  provider_chat_id: string
  from_message_id?: number
  mode?: 'latest' | 'older' | 'full'
  limit?: number
}

export type TelegramHistorySyncResponse = {
  account_id: string
  provider_chat_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  next_from_message_id: number | null
  items: TelegramMessage[]
}

// --- QR login ---
export type TelegramQrLoginStatus =
  | 'waiting_qr_scan'
  | 'waiting_password'
  | 'ready'
  | 'expired'
  | 'failed'
  | 'runtime_unavailable'

export type TelegramQrLoginStatusResponse = {
  setup_id: string
  account_id: string
  status: TelegramQrLoginStatus
  qr_link: string | null
  qr_svg: string | null
  telegram_user_id: string | null
  telegram_username: string | null
  suggested_account_id: string | null
  suggested_display_name: string | null
  suggested_external_account_id: string | null
  expires_at: string | null
  poll_after_ms: number
  message: string | null
}

// --- UI types ---
export type TelegramChatFilter = 'all' | 'unread' | 'mentions' | 'pinned' | 'projects' | 'bots' | 'archived'
export type TelegramThreadTab = 'messages' | 'files' | 'links' | 'voice' | 'topics' | 'pinned' | 'timeline'
export type TelegramRailTab = 'context' | 'members' | 'about'

export type TelegramChatFilterCount = {
  filter: TelegramChatFilter
  count: number
}

export type TelegramChatGroupFilter = {
  id: string
  label: string
  source: 'local' | 'telegram'
  count: number
  icon: string
  provider_folder_id?: number | null
}

export type TelegramAttachmentHint = {
  id: string
  kind: 'document' | 'photo' | 'video' | 'audio' | 'voice' | 'sticker' | 'animation' | 'video_note' | 'file'
  fileName: string
  mimeType: string | null
  sizeBytes: number | null
  tdlibFileId: number | null
  providerAttachmentId: string
  downloadState: 'remote' | 'downloading' | 'downloaded' | 'failed' | 'unknown'
  localPath: string | null
  expectedSizeBytes?: number | null
  downloadedSizeBytes?: number | null
  isDownloadingActive?: boolean | null
  isDownloadingCompleted?: boolean | null
  lastError?: string | null
  messageId: string
  providerMessageId?: string | null
}

// --- Additional request/response types for API ---
export type TelegramRuntimeStartRequest = {
  account_id: string
}

export type TelegramRuntimeStopRequest = {
  account_id: string
}

export type TelegramRuntimeRestartRequest = {
  account_id: string
}

export type TelegramQrLoginStartRequest = {
  account_id: string
  display_name: string
  external_account_id: string
  api_id?: number
  api_hash?: string
  session_encryption_key?: string
  tdlib_data_path?: string
  transcription_enabled: boolean
}

export type TelegramQrLoginPasswordRequest = {
  password: string
}

export type TelegramMediaDownloadRequest = {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  tdlib_file_id: number
  provider_attachment_id?: string
  filename?: string
  content_type?: string
  priority?: number
}

export type TelegramMediaDownloadResponse = {
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  runtime_kind: string
  status: string
  tdlib_file_id: number
  local_path: string | null
  size_bytes: number | null
  expected_size_bytes: number | null
  downloaded_size_bytes: number | null
  is_downloading_active: boolean
  is_downloading_completed: boolean
  attachment_id: string | null
  blob_id: string | null
  scan_status: string | null
}

export type TelegramManualSendResponse = {
  message_id: string
  provider_message_id: string
  provider_chat_id: string
  status: string
}

export type TelegramSendDryRunResponse = {
  allowed: boolean
  reason: string | null
}

export type TelegramMessageIngestResponse = {
  raw_record_id: string
  message_id: string
}

export type TelegramCallListResponse = {
  items: TelegramCall[]
}

export type TelegramCall = {
  call_id: string
  account_id: string
  provider_chat_id: string
  status: string
  occurred_at: string | null
}

export type TelegramCallTranscript = {
  transcript_id: string
  call_id: string
  account_id: string
  provider_chat_id: string
  transcript_status: string
  stt_provider: string
  source_audio_ref: string | null
  language_code: string | null
  transcript_text: string
  segments: unknown
  provenance: unknown
  created_at: string
  updated_at: string
}

export type TelegramCallTranscriptResponse = {
  transcript: TelegramCallTranscript | null
}

// --- Lifecycle types (ADR-0091) ---

export type TelegramTombstoneReasonClass =
  | 'deleted_by_owner'
  | 'deleted_by_counterparty'
  | 'deleted_by_provider'
  | 'moderation_removed'
  | 'account_removed'
  | 'retention_policy'
  | 'unknown'

export type TelegramTombstoneActorClass = 'owner' | 'provider' | 'automation' | 'system' | 'unknown'

export type TelegramCommandKind =
  | 'send_text'
  | 'send_media'
  | 'edit'
  | 'delete'
  | 'restore_visibility'
  | 'mark_read'
  | 'mark_unread'
  | 'pin'
  | 'unpin'
  | 'archive'
  | 'unarchive'
  | 'mute'
  | 'unmute'
  | 'folder_add'
  | 'folder_remove'
  | 'react'
  | 'unreact'
  | 'reply'
  | 'forward'
  | 'join'
  | 'leave'
  | 'topic_create'
  | 'topic_close'
  | 'topic_reopen'
  | 'admin_action'

export type TelegramCommandStatus = 'queued' | 'executing' | 'completed' | 'failed' | 'retrying' | 'cancelled' | 'dead_letter'
export type TelegramConfirmationDecision = 'pending' | 'confirmed' | 'rejected' | 'not_required'

export type TelegramLifecycleResponse = {
  operation: string
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  status: string
  timestamp: string
  version_number: number | null
  tombstone_id: string | null
}

export type {
  TelegramDeleteRequest,
  TelegramEditRequest,
  TelegramForwardRequest,
  TelegramPinRequest,
  TelegramReplyRequest,
  TelegramRestoreVisibilityRequest,
} from './telegramLifecycleRequests'

export type TelegramMessageVersion = {
  version_id: string
  message_id: string
  account_id:
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/shared/communications/types/telegramChatActions.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/types/telegramChatActions.ts`
- Size bytes / Размер в байтах: `838`
- Included characters / Включено символов: `838`
- Truncated / Обрезано: `no`

```typescript
export type TelegramChatActionRequest = {
  account_id: string
  provider_chat_id: string
  last_read_inbox_provider_message_id?: string
}

export type TelegramChatActionResponse = {
  telegram_chat_id: string
  action: string
  status: string
  metadata: Record<string, unknown>
}

export type TelegramChatLifecycleCommandResponse = {
  telegram_chat_id: string | null
  provider_chat_id: string
  action: string
  status: string
  command_id: string
}

export type TelegramChatFolderReassignRequest = {
  account_id: string
  provider_chat_id: string
  target_provider_folder_ids: number[]
}

export type TelegramChatFolderReassignResponse = {
  telegram_chat_id: string
  provider_chat_id: string
  action: string
  status: string
  command_ids: string[]
  added_provider_folder_ids: number[]
  removed_provider_folder_ids: number[]
}
```

### `frontend/src/shared/communications/types/telegramLifecycleRequests.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/types/telegramLifecycleRequests.ts`
- Size bytes / Размер в байтах: `1462`
- Included characters / Включено символов: `1462`
- Truncated / Обрезано: `no`

```typescript
// Lifecycle write-command request types (ADR-0091).
// Split from telegram.ts to stay under the 700-line SRP limit.

type TombstoneReasonClass =
  | 'deleted_by_owner'
  | 'deleted_by_counterparty'
  | 'deleted_by_provider'
  | 'moderation_removed'
  | 'account_removed'
  | 'retention_policy'
  | 'unknown'

type TombstoneActorClass = 'owner' | 'provider' | 'automation' | 'system' | 'unknown'

export type TelegramEditRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  new_text: string
}

export type TelegramReplyRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  reply_to_provider_message_id: string
  text: string
}

export type TelegramForwardRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  from_provider_chat_id: string
  from_provider_message_id: string
}

export type TelegramDeleteRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason_class: TombstoneReasonClass
  actor_class: TombstoneActorClass
  is_provider_delete: boolean
}

export type TelegramRestoreVisibilityRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  reason: string
}

export type TelegramPinRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  is_pinned: boolean
}
```

### `frontend/src/shared/communications/types/telegramMembers.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/types/telegramMembers.ts`
- Size bytes / Размер в байтах: `652`
- Included characters / Включено символов: `652`
- Truncated / Обрезано: `no`

```typescript
export type TelegramChatMember = {
  sender_id: string
  sender_display_name: string | null
  message_count: number
  last_message_at: string | null
  source: 'tdlib' | 'bot_api' | 'message_heuristic'
  provider_member_id: string
  username: string | null
  role: string | null
  status: string | null
  is_admin: boolean
  is_owner: boolean
  permissions: Record<string, unknown>
  observed_at: string | null
}

export type TelegramChatMemberListResponse = {
  items: TelegramChatMember[]
  next_cursor: string | null
}

export type TelegramChatMembersSyncResponse = {
  telegram_chat_id: string
  synced_count: number
  items: TelegramChatMember[]
}
```

### `frontend/src/shared/communications/types/telegramRawEvidence.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/types/telegramRawEvidence.ts`
- Size bytes / Размер в байтах: `415`
- Included characters / Включено символов: `415`
- Truncated / Обрезано: `no`

```typescript
export type TelegramRawMessageRecord = {
  raw_record_id: string
  provider_kind: string
  provider_account_id: string
  provider_message_id: string
  source_uri: string | null
  occurred_at: string
  ingested_at: string
  payload: Record<string, unknown>
  headers: Record<string, string>
  provenance: Record<string, unknown>
}

export type TelegramRawMessageResponse = {
  raw_record: TelegramRawMessageRecord
}
```

### `frontend/src/shared/communications/types/telegramTopics.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/types/telegramTopics.ts`
- Size bytes / Размер в байтах: `1061`
- Included characters / Включено символов: `1059`
- Truncated / Обрезано: `no`

```typescript
// Forum topic types (P1) — split to keep telegram.ts under the 700-line SRP limit.

export type TelegramTopic = {
  topic_id: string
  telegram_chat_id: string
  account_id: string
  provider_topic_id: number
  provider_chat_id: string
  title: string
  icon_emoji: string | null
  is_pinned: boolean
  is_closed: boolean
  unread_count: number
  last_message_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type TelegramTopicListResponse = {
  telegram_chat_id: string
  items: TelegramTopic[]
}

export type TelegramTopicCreateRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  title: string
}

export type TelegramTopicCloseRequest = {
  command_id: string
  account_id: string
  provider_chat_id: string
  is_closed: boolean
}

export type TelegramTopicLifecycleResponse = {
  operation: string
  topic_id: string | null
  account_id: string
  provider_chat_id: string
  provider_topic_id: number | null
  status: string
  timestamp: string
  command_id: string
}
```

### `frontend/src/shared/communications/types/whatsapp.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/communications/types/whatsapp.ts`
- Size bytes / Размер в байтах: `13768`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
// --- Provider ---
export type WhatsappWebProviderKind = 'whatsapp_web' | 'whatsapp_business_cloud'
export type WhatsappProviderShape =
  | 'whatsapp_web_companion'
  | 'whatsapp_native_md'
  | 'whatsapp_business_cloud'

// --- Capabilities ---
export type WhatsappCapabilityStatus = {
  capability: string
  category: string
  status: 'available' | 'blocked' | string
  action_class: 'read' | 'local_write' | 'provider_write' | 'destructive' | 'secret_access' | string
  confirmation_required: boolean
  closure_gate: boolean
  reason: string
}

export type WhatsappProviderShapeStatus = {
  provider_shape: string
  status: 'available' | 'blocked' | 'degraded' | 'planned' | 'unsupported' | string
  reason: string
}

export type WhatsappCapabilityAccountScope = {
  account_id: string
  provider_kind: string
  provider_shape: string
  runtime_kind: string
  lifecycle_state: string
  live_runtime_available: boolean
  live_send_available: boolean
  media_download_available: boolean
  media_upload_available: boolean
}

export type WhatsappAccountSummary = {
  account_id: string
  provider_kind: WhatsappWebProviderKind
  provider_shape: WhatsappProviderShape | string | null
  display_name: string
  external_account_id: string
  runtime: string | null
  lifecycle_state: string | null
  created_at: string
  updated_at: string
}

export type WhatsappAccountListResponse = {
  items: WhatsappAccountSummary[]
}

export type WhatsappCapabilitiesResponse = {
  version: string
  runtime_mode: string
  provider_shapes: WhatsappProviderShapeStatus[]
  account_scope: WhatsappCapabilityAccountScope | null
  capabilities: WhatsappCapabilityStatus[]
  planned_features: string[]
  unsupported_features: string[]
}

export type WhatsAppRuntimeStatus = {
  account_id: string
  provider_kind: string
  provider_shape: string
  runtime_kind: string
  status: string
  fixture_runtime: boolean
  live_runtime_available: boolean
  live_send_available: boolean
  qr_pairing_available: boolean
  pair_code_available: boolean
  media_download_available: boolean
  media_upload_available: boolean
  session_restore_available: boolean
  session_secret_ref: string | null
  runtime_blockers: string[]
  last_error: string | null
  updated_at: string
}

export type WhatsAppRuntimeHealth = {
  account_id: string
  provider_shape: string
  runtime_kind: string
  status: string
  healthy: boolean
  checks: Record<string, unknown>
  checked_at: string
}

export type WhatsAppWebCompanionBridgeRoutes = {
  authorized_session_path: string
  runtime_event_path: string
  sync_lifecycle_path: string
  message_paths: string[]
  conversation_paths: string[]
  media_paths: string[]
}

export type WhatsAppWebCompanionCommandChannel = {
  kind: string
  claim_path: string
  failure_path: string
  completion_rule: string
}

export type WhatsAppWebCompanionExtractorContract = {
  state: string
  relay_command?: string
  relay_command_policy?: Record<string, unknown>
  initialization_script: string
  script_scope: string
  origin_guard: string
  navigation_guard: string
  relay_channel: string
  runtime_bridge_dispatch: string
  allowed_observations: string[]
  forbidden_reads: string[]
  next_gate: string
}

export type WhatsAppWebCompanionSecretPolicy = {
  session_material: string
  cookies: string
  browser_profile_secrets: string
  qr_pair_code_artifacts: string
  message_bodies: string
  media_bytes: string
  postgres_storage: string
}

export type WhatsAppWebCompanionManifest = {
  account_id: string
  provider_shape: 'whatsapp_web_companion'
  runtime_kind: 'webview_companion'
  driver_id: string
  window_label: string
  target_url: string
  opened_window: boolean
  focused_existing_window: boolean
  owner_visible: boolean
  hidden_headless_mode: string
  tauri_ipc_available_to_companion_window: boolean
  event_flow: string
  event_extractor: WhatsAppWebCompanionExtractorContract
  bridge_routes: WhatsAppWebCompanionBridgeRoutes
  command_channel: WhatsAppWebCompanionCommandChannel
  secret_policy: WhatsAppWebCompanionSecretPolicy
  remaining_blockers: string[]
}

export type WhatsAppWebCompanionRelayObservationRequest = {
  account_id: string
  event_family: string
  provider_event_id: string
  observed_at: string
  metadata?: Record<string, unknown>
}

export type WhatsAppWebCompanionRelayObservationReceipt = {
  account_id: string
  provider_shape: 'whatsapp_web_companion'
  runtime_kind: 'webview_companion'
  window_label: string
  event_family: string
  provider_event_id: string
  observed_at: string
  target_runtime_bridge_path: string
  typed_runtime_bridge_path: string
  relay_state: string
  relay_channel: string
  sanitized_metadata: Record<string, unknown>
  runtime_event_kind: string
  import_batch_id: string
  runtime_bridge_http_status: number
  event_flow: string
  completion_rule: string
}

export type WhatsAppRuntimeRemoveResponse = {
  account_id: string
  provider_kind: string
  removed: boolean
  unbound_secret_refs: string[]
  removed_at: string
}

export type WhatsAppProviderCommand = {
  command_id: string
  account_id: string
  command_kind: string
  idempotency_key: string
  provider_chat_id: string
  provider_message_id: string | null
  capability_state: string
  action_class: string
  confirmation_decision: string
  status: string
  retry_count: number
  max_retries: number
  last_error: string | null
  result_payload: Record<string, unknown>
  audit_metadata: Record<string, unknown>
  provider_state: Record<string, unknown>
  reconciliation_status: string
  next_attempt_at: string | null
  last_attempt_at: string | null
  provider_observed_at: string | null
  reconciled_at: string | null
  dead_lettered_at: string | null
  completed_at: string | null
  created_at: string
  updated_at: string
}

export type WhatsAppProviderCommandListResponse = {
  items: WhatsAppProviderCommand[]
}

export type WhatsAppChatSyncItem = {
  conversation_id: string
  account_id: string
  channel_kind: string
  provider_chat_id: string
  title: string
  chat_kind: string | null
  is_archived: boolean
  is_pinned: boolean
  is_muted: boolean
  is_unread: boolean
  unread_count: number | null
  participant_count: number | null
  community_parent_chat_id: string | null
  community_parent_title: string | null
  invite_link: string | null
  is_community_root: boolean
  is_broadcast: boolean
  is_newsletter: boolean
  avatar_metadata: Record<string, unknown>
  provider_labels: string[]
}

export type WhatsAppChatSyncResponse = {
  account_id: string
  runtime_kind: string
  status: string
  synced_count: number
  items: WhatsAppChatSyncItem[]
}

export type WhatsAppMembersSyncItem = {
  participant_id: string
  conversation_id: string
  account_id: string
  provider_chat_id: string
  provider_member_id: string
  provider_identity_id: string | null
  sender_display_name: string | null
  role: string
  status: string | null
  identity_kind: string | null
  address: string | null
  is_admin: boolean
  is_owner: boolean
  participant_metadata: Record<string, unknown>
  identity_metadata: Record<string, unknown>
}

export type WhatsAppMembersSyncResponse = {
  account_id: string
  provider_chat_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppMembersSyncItem[]
}

export type WhatsAppPresenceSyncItem = {
  identity_id: string
  account_id: string
  channel_kind: string
  provider_chat_id: string | null
  provider_identity_id: string
  identity_kind: string
  display_name: string | null
  address: string | null
  presence_state: string
  last_seen_at: string | null
  observed_at: string | null
  identity_metadata: Record<string, unknown>
}

export type WhatsAppPresenceSyncResponse = {
  account_id: string
  provider_chat_id: string | null
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppPresenceSyncItem[]
}

export type WhatsAppCallSyncItem = {
  call_id: string
  account_id: string
  provider_call_id: string
  provider_chat_id: string
  direction: string
  call_state: string
  started_at: string | null
  ended_at: string | null
  observed_at: string | null
  metadata: Record<string, unknown>
}

export type WhatsAppCallsSyncResponse = {
  account_id: string
  provider_chat_id: string | null
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppCallSyncItem[]
}

export type WhatsAppContactSyncItem = {
  identity_id: string
  account_id: string
  channel_kind: string
  provider_identity_id: string
  identity_kind: string
  display_name: string | null
  address: string | null
  push_name: string | null
  business_profile: Record<string, unknown>
  profile_photo_ref: Record<string, unknown>
  display_name_history: string[]
  identity_metadata: Record<string, unknown>
  whatsapp_trace_metadata: Record<string, unknown>
  phone_trace_metadata: Record<string, unknown>
}

export type WhatsAppContactsSyncResponse = {
  account_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppContactSyncItem[]
}

export type WhatsAppMediaSyncItem = {
  attachment_id: string
  message_id: string
  raw_record_id: string
  account_id: string
  channel_kind: WhatsappWebProviderKind | string
  provider_chat_id: string | null
  provider_message_id: string
  provider_attachment_id: string
  filename: string | null
  content_type: string
  size_bytes: number
  sha256: string
  scan_status: string
  storage_kind: string
  storage_path: string
  message_subject: string
  sender: string
  sender_display_name: string | null
  occurred_at: string | null
  created_at: string
}

export type WhatsAppMediaSyncResponse = {
  account_id: string
  provider_chat_id: string | null
  content_type: string | null
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsAppMediaSyncItem[]
}

export type WhatsAppStatusSyncResponse = {
  account_id: string
  provider_chat_id: string
  runtime_kind: string
  status: string
  synced_count: number
  has_more: boolean
  items: WhatsappWebMessage[]
}

export type WhatsAppLifecycleResponse = {
  operation: string
  message_id: string
  account_id: string
  provider_chat_id: string
  provider_message_id: string
  status: string
  timestamp: string
  version_number: number | null
  tombstone_id: string | null
}

export type WhatsAppQrLinkSession = {
  account_id: string
  provider_shape: string
  runtime_kind: string
  status: string
  setup_id: string
  qr_svg: string | null
  expires_at: string | null
  runtime_blockers: string[]
}

export type WhatsAppPairCodeSession = {
  account_id: string
  provider_shape: string
  runtime_kind: string
  status: string
  setup_id: string
  phone_number: string
  pair_code: string | null
  expires_at: string | null
  runtime_blockers: string[]
}

// --- Sessions ---
export type WhatsappWebSession = {
  session_id: string
  account_id: string
  device_name: string
  companion_runtime: 'fixture' | 'manual_webview' | 'blocked'
  link_state:
    | 'fixture'
    | 'qr_pending'
    | 'pair_code_pending'
    | 'link_required'
    | 'linked'
    | 'degraded'
    | 'revoked'
    | 'removed'
    | 'blocked'
  local_state_path: string
  last_sync_at: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export type WhatsappWebSessionListResponse = {
  items: WhatsappWebSession[]
}

// --- Messages ---
export type WhatsappWebMessage = {
  message_id: string
  raw_record_id: string
  account_id: string
  provider_message_id: string
  provider_chat_id: string | null
  chat_title: string
  sender: string
  sender_display_name: string | null
  text: string
  occurred_at: string | null
  projected_at: string
  channel_kind: WhatsappWebProviderKind
  delivery_state: string
  metadata: Record<string, unknown>
}

export type WhatsappWebMessageListResponse = {
  items: WhatsappWebMessage[]
}

export type WhatsappWebMessageSearchRespon
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/shared/composables/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/composables/index.ts`
- Size bytes / Размер в байтах: `166`
- Included characters / Включено символов: `166`
- Truncated / Обрезано: `no`

```typescript
export { useClickOutside } from './useClickOutside'
export { useKeyboard, useEscapeKey } from './useKeyboard'
export { useResizeObserver } from './useResizeObserver'
```

### `frontend/src/shared/composables/useClickOutside.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/composables/useClickOutside.ts`
- Size bytes / Размер в байтах: `664`
- Included characters / Включено символов: `664`
- Truncated / Обрезано: `no`

```typescript
import { onMounted, onUnmounted, type Ref } from 'vue'

export function useClickOutside(
  elRef: Ref<HTMLElement | null>,
  callback: () => void,
  options?: { excludeElRef?: Ref<HTMLElement | null> }
): void {
  function handleClick(event: MouseEvent): void {
    const el = elRef.value
    const excludeEl = options?.excludeElRef?.value

    if (!el) return
    if (el.contains(event.target as Node)) return
    if (excludeEl?.contains(event.target as Node)) return

    callback()
  }

  onMounted(() => {
    document.addEventListener('click', handleClick, true)
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClick, true)
  })
}
```

### `frontend/src/shared/composables/useKeyboard.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/composables/useKeyboard.ts`
- Size bytes / Размер в байтах: `936`
- Included characters / Включено символов: `936`
- Truncated / Обрезано: `no`

```typescript
import { onMounted, onUnmounted } from 'vue'

type KeyHandler = {
  key: string
  ctrl?: boolean
  meta?: boolean
  shift?: boolean
  handler: () => void
}

export function useKeyboard(handlers: KeyHandler[]): void {
  function handleKeydown(event: KeyboardEvent): void {
    for (const h of handlers) {
      const ctrl = h.ctrl ?? false
      const meta = h.meta ?? false
      const shift = h.shift ?? false

      if (
        event.key === h.key &&
        event.ctrlKey === ctrl &&
        event.metaKey === meta &&
        event.shiftKey === shift
      ) {
        event.preventDefault()
        h.handler()
        return
      }
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })
}

export function useEscapeKey(callback: () => void): void {
  useKeyboard([{ key: 'Escape', handler: callback }])
}
```

### `frontend/src/shared/composables/useResizeObserver.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/composables/useResizeObserver.ts`
- Size bytes / Размер в байтах: `724`
- Included characters / Включено символов: `724`
- Truncated / Обрезано: `no`

```typescript
import { onMounted, onUnmounted, ref, type Ref } from 'vue'

export function useResizeObserver(
  elRef: Ref<HTMLElement | null>,
  callback: (entry: ResizeObserverEntry) => void
): { width: Ref<number>; height: Ref<number> } {
  const width = ref(0)
  const height = ref(0)
  let observer: ResizeObserver | null = null

  onMounted(() => {
    const el = elRef.value
    if (!el) return

    observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        width.value = entry.contentRect.width
        height.value = entry.contentRect.height
        callback(entry)
      }
    })

    observer.observe(el)
  })

  onUnmounted(() => {
    observer?.disconnect()
  })

  return { width, height }
}
```

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

- Chunk ID / ID чанка: `145-source-frontend-part-005`
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

### `frontend/src/domains/communications/components/outboxStatus.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/outboxStatus.ts`
- Size bytes / Размер в байтах: `5955`
- Included characters / Включено символов: `5955`
- Truncated / Обрезано: `no`

```typescript
import type { CommunicationOutboxItem } from '../types/communications'

export type OutboxStatusTone = 'neutral' | 'success' | 'warning' | 'danger' | 'muted'

export type OutboxStatusPresentation = {
  title: string
  detail: string
  tone: OutboxStatusTone
  icon: string
  canUndo: boolean
  isVisible: boolean
}

type JsonObject = Record<string, unknown>

const UTC_DATE_FORMAT = new Intl.DateTimeFormat('en-US', {
  month: 'short',
  day: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  hourCycle: 'h23',
  timeZone: 'UTC'
})

export function outboxStatusPresentation(
  item: CommunicationOutboxItem,
  now: Date = new Date()
): OutboxStatusPresentation {
  const readReceipt = objectField(item.metadata, 'latest_read_receipt')
  if (readReceipt && stringField(readReceipt, 'receipt_kind') === 'read') {
    return {
      title: 'Read',
      detail: timestampDetail('Read at', stringField(readReceipt, 'read_at')),
      tone: 'success',
      icon: 'tabler:mail-check',
      canUndo: false,
      isVisible: true
    }
  }

  const deliveryStatus = objectField(item.metadata, 'delivery_status')
  const deliveryStatusValue = deliveryStatus ? stringField(deliveryStatus, 'delivery_status') : null
  if (deliveryStatusValue === 'failed') {
    const smtpStatus = stringField(deliveryStatus, 'smtp_status')
    return {
      title: 'Delivery failed',
      detail: smtpStatus ? `Provider reported SMTP ${smtpStatus}` : 'Provider reported delivery failure',
      tone: 'danger',
      icon: 'tabler:alert-triangle',
      canUndo: false,
      isVisible: true
    }
  }
  if (deliveryStatusValue === 'delayed') {
    return {
      title: 'Delivery delayed',
      detail: timestampDetail('Provider update', stringField(deliveryStatus, 'recorded_at')),
      tone: 'warning',
      icon: 'tabler:clock-exclamation',
      canUndo: false,
      isVisible: true
    }
  }
  if (deliveryStatusValue === 'delivered') {
    return {
      title: 'Delivered',
      detail: timestampDetail('Confirmed at', stringField(deliveryStatus, 'recorded_at')),
      tone: 'success',
      icon: 'tabler:circle-check',
      canUndo: false,
      isVisible: true
    }
  }

  if (canUndo(item, now)) {
    return {
      title: 'Undo available',
      detail: timestampDetail('Until', item.undo_deadline_at),
      tone: 'warning',
      icon: 'tabler:arrow-back-up',
      canUndo: true,
      isVisible: true
    }
  }

  if (item.status === 'scheduled' && item.send_attempts > 1 && item.last_error) {
    return {
      title: 'Retry scheduled',
      detail: timestampDetail('Retry at', item.scheduled_send_at),
      tone: 'warning',
      icon: 'tabler:refresh-alert',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'scheduled') {
    return {
      title: 'Scheduled',
      detail: timestampDetail('Sends at', item.scheduled_send_at),
      tone: 'neutral',
      icon: 'tabler:calendar-time',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'queued') {
    return {
      title: 'Queued',
      detail: 'Waiting for delivery',
      tone: 'neutral',
      icon: 'tabler:send-2',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'sending') {
    return {
      title: 'Sending',
      detail: 'Provider handoff in progress',
      tone: 'neutral',
      icon: 'tabler:loader-2',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'failed') {
    return {
      title: 'Send failed',
      detail: item.last_error || 'Delivery worker failed',
      tone: 'danger',
      icon: 'tabler:alert-circle',
      canUndo: false,
      isVisible: true
    }
  }

  if (item.status === 'canceled') {
    return {
      title: 'Canceled',
      detail: 'Undo send canceled delivery',
      tone: 'muted',
      icon: 'tabler:circle-off',
      canUndo: false,
      isVisible: true
    }
  }

  return {
    title: 'Sent',
    detail: item.provider_message_id ? 'Provider accepted message' : 'Sent',
    tone: 'muted',
    icon: 'tabler:mail-forward',
    canUndo: false,
    isVisible: false
  }
}

export function visibleOutboxStatusItems(
  items: CommunicationOutboxItem[],
  maxItems = 6,
  now: Date = new Date()
): CommunicationOutboxItem[] {
  return [...items]
    .filter((item) => outboxStatusPresentation(item, now).isVisible)
    .sort((a, b) => statusPriority(a) - statusPriority(b) || timestampMillis(b.updated_at) - timestampMillis(a.updated_at))
    .slice(0, maxItems)
}

function objectField(value: JsonObject, field: string): JsonObject | null {
  const nested = value[field]
  if (!nested || typeof nested !== 'object' || Array.isArray(nested)) return null
  return nested as JsonObject
}

function stringField(value: JsonObject | null, field: string): string | null {
  const nested = value?.[field]
  return typeof nested === 'string' && nested.trim() ? nested : null
}

function canUndo(item: CommunicationOutboxItem, now: Date): boolean {
  if (item.status !== 'queued' && item.status !== 'scheduled') return false
  if (!item.undo_deadline_at) return false
  return timestampMillis(item.undo_deadline_at) >= now.getTime()
}

function timestampDetail(prefix: string, value: string | null): string {
  if (!value) return prefix
  const timestamp = timestampMillis(value)
  if (!Number.isFinite(timestamp)) return prefix
  return `${prefix} ${UTC_DATE_FORMAT.format(new Date(timestamp))}`
}

function timestampMillis(value: string): number {
  const timestamp = Date.parse(value)
  return Number.isFinite(timestamp) ? timestamp : 0
}

function statusPriority(item: CommunicationOutboxItem): number {
  if (item.status === 'failed') return 0
  if (item.status === 'sending') return 1
  if (item.status === 'queued' || item.status === 'scheduled') return 2
  if (objectField(item.metadata, 'latest_read_receipt') || objectField(item.metadata, 'delivery_status')) return 3
  if (item.status === 'canceled') return 4
  return 5
}
```

### `frontend/src/domains/communications/components/richComposeExtensions.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/richComposeExtensions.ts`
- Size bytes / Размер в байтах: `4139`
- Included characters / Включено символов: `4139`
- Truncated / Обрезано: `no`

```typescript
import { Mark, Node } from '@tiptap/vue-3'
import { normalizeMailComposeLinkHref, normalizeMailComposeTextAlign } from './richComposeHtml'

const RichDocument = Node.create({
	name: 'doc',
	topNode: true,
	content: 'block+'
})

const RichParagraph = Node.create({
	name: 'paragraph',
	group: 'block',
	content: 'inline*',
	addAttributes() {
		return {
			textAlign: {
				default: null,
				parseHTML: (element) => normalizeMailComposeTextAlign(element.style.textAlign),
				renderHTML: (attributes) => getSafeTextAlignAttributes(attributes.textAlign)
			}
		}
	},
	parseHTML() {
		return [{ tag: 'p' }]
	},
	renderHTML({ HTMLAttributes }) {
		return ['p', HTMLAttributes, 0]
	}
})

const RichHeading = Node.create({
	name: 'heading',
	group: 'block',
	content: 'inline*',
	addAttributes() {
		return {
			level: {
				default: 2,
				parseHTML: (element) => {
					const level = Number(element.tagName.slice(1))
					return level === 3 ? 3 : 2
				},
				renderHTML: () => ({})
			},
			textAlign: {
				default: null,
				parseHTML: (element) => normalizeMailComposeTextAlign(element.style.textAlign),
				renderHTML: (attributes) => getSafeTextAlignAttributes(attributes.textAlign)
			}
		}
	},
	parseHTML() {
		return [
			{ tag: 'h2' },
			{ tag: 'h3' }
		]
	},
	renderHTML({ HTMLAttributes, node }) {
		const level = node.attrs.level === 3 ? 3 : 2
		return [`h${level}`, HTMLAttributes, 0]
	}
})

const RichText = Node.create({
	name: 'text',
	group: 'inline'
})

const RichBulletList = Node.create({
	name: 'bulletList',
	group: 'block',
	content: 'listItem+',
	parseHTML() {
		return [{ tag: 'ul' }]
	},
	renderHTML() {
		return ['ul', 0]
	}
})

const RichOrderedList = Node.create({
	name: 'orderedList',
	group: 'block',
	content: 'listItem+',
	parseHTML() {
		return [{ tag: 'ol' }]
	},
	renderHTML() {
		return ['ol', 0]
	}
})

const RichListItem = Node.create({
	name: 'listItem',
	content: 'paragraph block*',
	parseHTML() {
		return [{ tag: 'li' }]
	},
	renderHTML() {
		return ['li', 0]
	}
})

const RichBlockquote = Node.create({
	name: 'blockquote',
	group: 'block',
	content: 'block+',
	parseHTML() {
		return [{ tag: 'blockquote' }]
	},
	renderHTML() {
		return ['blockquote', 0]
	}
})

const RichBold = Mark.create({
	name: 'bold',
	parseHTML() {
		return [
			{ tag: 'strong' },
			{ tag: 'b' },
			{
				style: 'font-weight',
				getAttrs: (value) => {
					return /^(bold(er)?|[5-9]\d{2,})$/.test(String(value)) ? null : false
				}
			}
		]
	},
	renderHTML() {
		return ['strong', 0]
	}
})

const RichItalic = Mark.create({
	name: 'italic',
	parseHTML() {
		return [
			{ tag: 'em' },
			{ tag: 'i' },
			{ style: 'font-style=italic' }
		]
	},
	renderHTML() {
		return ['em', 0]
	}
})

function getSafeTextAlignAttributes(value: unknown): Record<string, string> {
	if (typeof value !== 'string') return {}
	const textAlign = normalizeMailComposeTextAlign(value)
	return textAlign ? { style: `text-align: ${textAlign}` } : {}
}

function getSafeLinkAttributes(value: unknown): Record<string, string> {
	if (typeof value !== 'string') return {}
	const href = normalizeMailComposeLinkHref(value)
	if (!href) return {}
	return {
		href,
		rel: 'noopener noreferrer',
		target: '_blank'
	}
}

const RichLink = Mark.create({
	name: 'link',
	inclusive: false,
	addAttributes() {
		return {
			href: {
				default: null,
				parseHTML: (element) => normalizeMailComposeLinkHref(element.getAttribute('href') ?? ''),
				renderHTML: (attributes) => getSafeLinkAttributes(attributes.href)
			}
		}
	},
	parseHTML() {
		return [
			{
				tag: 'a[href]',
				getAttrs: (node) => {
					if (!(node instanceof HTMLElement)) return false
					return normalizeMailComposeLinkHref(node.getAttribute('href') ?? '') ? null : false
				}
			}
		]
	},
	renderHTML({ HTMLAttributes }) {
		const attributes = getSafeLinkAttributes(HTMLAttributes.href)
		return Object.keys(attributes).length > 0 ? ['a', attributes, 0] : ['span', 0]
	}
})

export const richComposeExtensions = [
	RichDocument,
	RichParagraph,
	RichHeading,
	RichText,
	RichBulletList,
	RichOrderedList,
	RichListItem,
	RichBlockquote,
	RichBold,
	RichItalic,
	RichLink
]
```

### `frontend/src/domains/communications/components/richComposeHtml.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/richComposeHtml.test.ts`
- Size bytes / Размер в байтах: `3872`
- Included characters / Включено символов: `3872`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
	appendHtmlSignature,
	appendPlainTextSignature,
	htmlToComposePlainText,
	normalizeMailComposeLinkHref,
	normalizeMailComposeTextAlign,
	plainTextToComposeHtml,
	sanitizeMailComposePastedHtml
} from './richComposeHtml'

describe('rich compose HTML helpers', () => {
	it('escapes plain text and preserves paragraph breaks for rich compose mode', () => {
		expect(plainTextToComposeHtml('Hello <team>\n\nLine & two')).toBe(
			'<p>Hello &lt;team&gt;</p><p>Line &amp; two</p>'
		)
	})

	it('returns an empty paragraph for blank input so the editor remains focusable', () => {
		expect(plainTextToComposeHtml('  \n ')).toBe('<p></p>')
	})

	it('derives a plain text fallback from rich compose HTML', () => {
		expect(htmlToComposePlainText('<p>Hello <strong>team</strong></p><ul><li>One</li></ul>')).toBe(
			'Hello team\nOne'
		)
	})

	it('appends plain and HTML signatures without dropping existing body text', () => {
		expect(appendPlainTextSignature('Hello', 'Alex')).toBe('Hello\n\nAlex')
		expect(appendHtmlSignature('<p>Hello</p>', 'Alex <Lead>')).toBe(
			'<p>Hello</p><p></p><p>Alex &lt;Lead&gt;</p>'
		)
	})

	it('normalizes safe compose link hrefs and rejects active content schemes', () => {
		expect(normalizeMailComposeLinkHref('example.com/path')).toBe('https://example.com/path')
		expect(normalizeMailComposeLinkHref('https://example.com/a?b=1')).toBe(
			'https://example.com/a?b=1'
		)
		expect(normalizeMailComposeLinkHref('mailto:team@example.com')).toBe('mailto:team@example.com')
		expect(normalizeMailComposeLinkHref('javascript:alert(1)')).toBeNull()
		expect(normalizeMailComposeLinkHref('data:text/html,<script>alert(1)</script>')).toBeNull()
		expect(normalizeMailComposeLinkHref('  ')).toBeNull()
		expect(normalizeMailComposeLinkHref('https://')).toBeNull()
		expect(normalizeMailComposeLinkHref('mailto:')).toBeNull()
	})

	it('normalizes only supported mail compose text alignment values', () => {
		expect(normalizeMailComposeTextAlign(' center ')).toBe('center')
		expect(normalizeMailComposeTextAlign('RIGHT')).toBe('right')
		expect(normalizeMailComposeTextAlign('justify')).toBeNull()
		expect(normalizeMailComposeTextAlign('position:absolute')).toBeNull()
	})

	it('sanitizes pasted rich HTML to the supported mail-safe compose subset', () => {
		expect(
			sanitizeMailComposePastedHtml(
				'<p style="color:red"><strong onclick="x()">Hi</strong> <a href="javascript:alert(1)">bad</a> <a href="https://example.com/path" onclick="x()">ok</a><img src=x onerror=x></p><script>alert(1)</script>'
			)
		).toBe(
			'<p><strong>Hi</strong> bad <a href="https://example.com/path" rel="noopener noreferrer" target="_blank">ok</a></p>'
		)
	})

	it('preserves pasted office-style ordered lists in the supported rich compose schema', () => {
		expect(sanitizeMailComposePastedHtml('<ol><li>One</li><li><em>Two</em></li></ol>')).toBe(
			'<ol><li>One</li><li><em>Two</em></li></ol>'
		)
	})

	it('preserves pasted block quotes without unsafe attributes', () => {
		expect(
			sanitizeMailComposePastedHtml('<blockquote cite="https://example.com"><p>Quoted <b>text</b></p></blockquote>')
		).toBe(
			'<blockquote><p>Quoted <strong>text</strong></p></blockquote>'
		)
	})

	it('preserves mail-safe pasted headings and normalizes oversized headings', () => {
		expect(sanitizeMailComposePastedHtml('<h1 onclick="x()">Title</h1><h3>Section</h3>')).toBe(
			'<h2>Title</h2><h3>Section</h3>'
		)
	})

	it('preserves only safe pasted text alignment styles on supported block nodes', () => {
		expect(
			sanitizeMailComposePastedHtml(
				'<p style="text-align:center;color:red">Centered</p><h2 style="text-align: right">Title</h2><p style="text-align:justify">Wide</p>'
			)
		).toBe(
			'<p style="text-align: center">Centered</p><h2 style="text-align: right">Title</h2><p>Wide</p>'
		)
	})
})
```

### `frontend/src/domains/communications/components/richComposeHtml.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/richComposeHtml.ts`
- Size bytes / Размер в байтах: `6571`
- Included characters / Включено символов: `6571`
- Truncated / Обрезано: `no`

```typescript
export function plainTextToComposeHtml(text: string): string {
	const paragraphs = text
		.split(/\n{2,}/)
		.map((paragraph) => paragraph.trim())
		.filter(Boolean)

	if (paragraphs.length === 0) return '<p></p>'

	return paragraphs
		.map((paragraph) => `<p>${escapeHtml(paragraph).replace(/\n/g, '<br>')}</p>`)
		.join('')
}

export function htmlToComposePlainText(html: string): string {
	return html
		.replace(/<\/(p|div|li|h[1-6])>/gi, '\n')
		.replace(/<br\s*\/?>/gi, '\n')
		.replace(/<[^>]+>/g, '')
		.replace(/&nbsp;/g, ' ')
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&quot;/g, '"')
		.replace(/&#39;/g, "'")
		.replace(/&amp;/g, '&')
		.split('\n')
		.map((line) => line.trim())
		.filter(Boolean)
		.join('\n')
}

export function appendPlainTextSignature(body: string, signature: string): string {
	const existingBody = body.trim()
	const trimmedSignature = signature.trim()
	if (!trimmedSignature) return existingBody
	return `${existingBody}${existingBody ? '\n\n' : ''}${trimmedSignature}`
}

export function appendHtmlSignature(bodyHtml: string | null | undefined, signature: string): string {
	const existingHtml = bodyHtml?.trim() ?? ''
	const trimmedSignature = signature.trim()
	if (!trimmedSignature) return existingHtml
	return `${existingHtml}${existingHtml ? '<p></p>' : ''}<p>${escapeHtml(trimmedSignature).replace(/\n/g, '<br>')}</p>`
}

export function normalizeMailComposeLinkHref(value: string): string | null {
	const trimmedValue = value.trim()
	if (!trimmedValue || /\s/.test(trimmedValue)) return null

	const href = /^[a-z][a-z0-9+.-]*:/i.test(trimmedValue)
		? trimmedValue
		: `https://${trimmedValue}`

	try {
		const parsedHref = new URL(href)
		if (!['http:', 'https:', 'mailto:'].includes(parsedHref.protocol)) return null
		if (parsedHref.protocol === 'mailto:' && !parsedHref.pathname) return null
		if (
			(parsedHref.protocol === 'http:' || parsedHref.protocol === 'https:') &&
			(parsedHref.username || parsedHref.password)
		) {
			return null
		}
		return parsedHref.toString()
	} catch {
		return null
	}
}

export type MailComposeTextAlign = 'center' | 'left' | 'right'

export function normalizeMailComposeTextAlign(value: string | null | undefined): MailComposeTextAlign | null {
	const normalizedValue = value?.trim().toLowerCase()
	if (normalizedValue === 'center' || normalizedValue === 'left' || normalizedValue === 'right') {
		return normalizedValue
	}
	return null
}

const SKIPPED_PASTE_CONTENT_TAGS = new Set([
	'head',
	'iframe',
	'link',
	'meta',
	'object',
	'script',
	'style',
	'svg'
])

const NORMALIZED_PASTE_TAGS = new Map([
	['a', 'a'],
	['b', 'strong'],
	['blockquote', 'blockquote'],
	['br', 'br'],
	['div', 'p'],
	['em', 'em'],
	['h1', 'h2'],
	['h2', 'h2'],
	['h3', 'h3'],
	['h4', 'h3'],
	['h5', 'h3'],
	['h6', 'h3'],
	['i', 'em'],
	['li', 'li'],
	['ol', 'ol'],
	['p', 'p'],
	['strong', 'strong'],
	['ul', 'ul']
])

export function sanitizeMailComposePastedHtml(html: string): string {
	const source = html.trim()
	if (!source) return '<p></p>'

	const emittedTags: string[] = []
	let skippedContentTag: string | null = null
	let sanitized = ''

	for (const token of source.split(/(<[^>]*>)/g)) {
		if (!token) continue
		const parsedTag = parseHtmlTag(token)
		if (!parsedTag) {
			if (!skippedContentTag) sanitized += token
			continue
		}

		if (SKIPPED_PASTE_CONTENT_TAGS.has(parsedTag.name)) {
			if (!parsedTag.closing && !parsedTag.selfClosing) skippedContentTag = parsedTag.name
			if (parsedTag.closing && skippedContentTag === parsedTag.name) skippedContentTag = null
			continue
		}
		if (skippedContentTag) continue

		const normalizedTag = NORMALIZED_PASTE_TAGS.get(parsedTag.name)
		if (!normalizedTag) continue
		if (parsedTag.closing) {
			sanitized += closePastedTag(normalizedTag, emittedTags)
			continue
		}
		if (normalizedTag === 'br') {
			sanitized += '<br>'
			continue
		}
		if (normalizedTag === 'a') {
			const href = normalizeMailComposeLinkHref(extractHtmlAttribute(parsedTag.attributes, 'href') ?? '')
			if (!href) continue
			emittedTags.push(normalizedTag)
			sanitized += `<a href="${escapeHtmlAttribute(href)}" rel="noopener noreferrer" target="_blank">`
			continue
		}

		emittedTags.push(normalizedTag)
		sanitized += `<${normalizedTag}${renderPastedTextAlignAttribute(parsedTag.attributes, normalizedTag)}>`
		if (parsedTag.selfClosing) sanitized += closePastedTag(normalizedTag, emittedTags)
	}

	while (emittedTags.length > 0) {
		sanitized += `</${emittedTags.pop()}>`
	}

	return sanitized.trim() || '<p></p>'
}

function parseHtmlTag(value: string): {
	attributes: string
	closing: boolean
	name: string
	selfClosing: boolean
} | null {
	const match = value.match(/^<\s*(\/)?\s*([a-z][a-z0-9-]*)([^>]*)>/i)
	if (!match) return null
	const attributes = match[3] ?? ''
	return {
		attributes,
		closing: Boolean(match[1]),
		name: match[2].toLowerCase(),
		selfClosing: /\/\s*>$/.test(value)
	}
}

function renderPastedTextAlignAttribute(attributes: string, tag: string): string {
	if (!['h2', 'h3', 'p'].includes(tag)) return ''
	const textAlign = normalizeMailComposeTextAlign(extractCssDeclarationValue(attributes, 'text-align'))
	return textAlign ? ` style="text-align: ${textAlign}"` : ''
}

function extractCssDeclarationValue(attributes: string, propertyName: string): string | null {
	const style = extractHtmlAttribute(attributes, 'style')
	if (!style) return null
	const normalizedPropertyName = propertyName.toLowerCase()
	for (const declaration of style.split(';')) {
		const separatorIndex = declaration.indexOf(':')
		if (separatorIndex === -1) continue
		const name = declaration.slice(0, separatorIndex).trim().toLowerCase()
		if (name !== normalizedPropertyName) continue
		return declaration.slice(separatorIndex + 1).trim()
	}
	return null
}

function closePastedTag(tag: string, emittedTags: string[]): string {
	const lastTag = emittedTags.at(-1)
	if (lastTag !== tag) return ''
	emittedTags.pop()
	return `</${tag}>`
}

function extractHtmlAttribute(attributes: string, name: string): string | null {
	const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
	const match = attributes.match(new RegExp(`\\s${escapedName}\\s*=\\s*(?:"([^"]*)"|'([^']*)'|([^\\s"'>/]+))`, 'i'))
	return match?.[1] ?? match?.[2] ?? match?.[3] ?? null
}

function escapeHtmlAttribute(value: string): string {
	return escapeHtml(value)
}

function escapeHtml(value: string): string {
	return value
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#39;')
}
```

### `frontend/src/domains/communications/components/savedSearchRuleTreePresentation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/savedSearchRuleTreePresentation.test.ts`
- Size bytes / Размер в байтах: `1159`
- Included characters / Включено символов: `1156`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { createSavedSearchRuleCondition, createSavedSearchRuleGroup } from '../forms/savedSearchForm'
import {
  savedSearchRuleGroupDepthLabel,
  savedSearchRuleGroupSummary
} from './savedSearchRuleTreePresentation'

describe('saved search rule tree presentation', () => {
  it('labels root and nested groups with stable depth cues', () => {
    expect(savedSearchRuleGroupDepthLabel(0)).toBe('Root group')
    expect(savedSearchRuleGroupDepthLabel(1)).toBe('Group 2')
    expect(savedSearchRuleGroupDepthLabel(2)).toBe('Group 3')
  })

  it('summarizes group structure for the rules builder header', () => {
    expect(savedSearchRuleGroupSummary(createSavedSearchRuleGroup('all', []))).toBe(
      'All conditions · Empty'
    )

    expect(savedSearchRuleGroupSummary(createSavedSearchRuleGroup('any', [
      createSavedSearchRuleCondition({ field: 'subject', operator: ':', value: 'quarterly' }),
      createSavedSearchRuleGroup('all', [
        createSavedSearchRuleCondition({ field: 'sender', operator: ':', value: 'alex' })
      ])
    ]))).toBe('Any condition · 1 rule · 1 nested group')
  })
})
```

### `frontend/src/domains/communications/components/savedSearchRuleTreePresentation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/savedSearchRuleTreePresentation.ts`
- Size bytes / Размер в байтах: `837`
- Included characters / Включено символов: `836`
- Truncated / Обрезано: `no`

```typescript
import type { SavedSearchRuleGroup } from '../forms/savedSearchForm'

export function savedSearchRuleGroupDepthLabel(depth: number): string {
  return depth <= 0 ? 'Root group' : `Group ${depth + 1}`
}

export function savedSearchRuleGroupSummary(group: SavedSearchRuleGroup): string {
  const ruleCount = group.children.filter((child) => child.kind === 'rule').length
  const nestedGroupCount = group.children.filter((child) => child.kind === 'group').length
  const segments = [`${group.matchMode === 'all' ? 'All conditions' : 'Any condition'}`]

  if (ruleCount) segments.push(`${ruleCount} rule${ruleCount === 1 ? '' : 's'}`)
  if (nestedGroupCount) segments.push(`${nestedGroupCount} nested group${nestedGroupCount === 1 ? '' : 's'}`)
  if (!ruleCount && !nestedGroupCount) segments.push('Empty')

  return segments.join(' · ')
}
```

### `frontend/src/domains/communications/components/templateLibrary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/templateLibrary.test.ts`
- Size bytes / Размер в байтах: `3536`
- Included characters / Включено символов: `3534`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  applyTemplateRecipientMapping,
  buildTemplateRecipientPreviewRows,
  deriveTemplateLibraryCategories,
  inferRecipientVariableMapping,
  recipientPreviewSummary,
  suggestTemplateSaveName,
  templateMatchesLibraryCategory
} from './templateLibrary'

describe('template library helpers', () => {
  it('derives stable categories from template structure and diagnostics', () => {
    const template = {
      variables: ['recipient', 'project'],
      malformed_placeholders: [],
      undeclared_variables: []
    }

    expect(deriveTemplateLibraryCategories(template)).toEqual([
      'mail-merge',
      'recipient-aware'
    ])
    expect(templateMatchesLibraryCategory(template, 'mail-merge')).toBe(true)
    expect(templateMatchesLibraryCategory(template, 'static-copy')).toBe(false)
  })

  it('classifies static and broken templates into useful categories', () => {
    expect(deriveTemplateLibraryCategories({
      variables: [],
      malformed_placeholders: [],
      undeclared_variables: []
    })).toEqual(['static-copy'])

    expect(deriveTemplateLibraryCategories({
      variables: ['project'],
      malformed_placeholders: ['{{ }}'],
      undeclared_variables: ['recipient']
    })).toEqual([
      'mail-merge',
      'needs-attention'
    ])
  })

  it('infers recipient mapping and fills mapped variables from compose recipients', () => {
    const mapping = inferRecipientVariableMapping(['recipient', 'cc', 'bcc', 'project'])

    expect(mapping).toEqual({
      toVariable: 'recipient',
      ccVariable: 'cc',
      bccVariable: 'bcc'
    })

    expect(applyTemplateRecipientMapping({
      project: 'Макошь'
    }, mapping, {
      toText: 'alex@example.com, sam@example.com',
      ccText: 'ops@example.com',
      bccText: 'audit@example.com'
    })).toEqual({
      project: 'Макошь',
      recipient: 'alex@example.com, sam@example.com',
      cc: 'ops@example.com',
      bcc: 'audit@example.com'
    })
  })

  it('builds mail-merge preview rows from To recipients using the selected mapping', () => {
    expect(buildTemplateRecipientPreviewRows(
      ['recipient', 'cc', 'project'],
      {
        toVariable: 'recipient',
        ccVariable: 'cc',
        bccVariable: ''
      },
      {
        toText: 'alex@example.com, sam@example.com',
        ccText: 'ops@example.com',
        bccText: ''
      },
      {
        project: 'Макошь rollout'
      }
    )).toEqual([
      {
        row_id: 'recipient-1',
        variables: {
          recipient: 'alex@example.com',
          cc: 'ops@example.com',
          project: 'Макошь rollout'
        }
      },
      {
        row_id: 'recipient-2',
        variables: {
          recipient: 'sam@example.com',
          cc: 'ops@example.com',
          project: 'Макошь rollout'
        }
      }
    ])
  })

  it('summarizes compose recipient counts for the mapping panel', () => {
    expect(recipientPreviewSummary({
      toText: 'alex@example.com, sam@example.com',
      ccText: 'ops@example.com',
      bccText: ''
    })).toBe('2 To · 1 CC · 0 BCC')
  })

  it('suggests stable save names for new and duplicate template flows', () => {
    expect(suggestTemplateSaveName('Quarterly follow-up', '', { duplicate: false })).toBe('Quarterly follow-up')
    expect(suggestTemplateSaveName('', 'Client follow-up', { duplicate: true })).toBe('Client follow-up copy')
    expect(suggestTemplateSaveName('', 'Client follow-up', { duplicate: false })).toBe('Client follow-up')
  })
})
```

### `frontend/src/domains/communications/components/templateLibrary.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/templateLibrary.ts`
- Size bytes / Размер в байтах: `7217`
- Included characters / Включено символов: `7215`
- Truncated / Обрезано: `no`

```typescript
import { splitComposeRecipients } from '../forms/composeValidation'
import type { CommunicationTemplate } from '../types/templates'

export type TemplateLibraryCategory =
  | 'mail-merge'
  | 'recipient-aware'
  | 'static-copy'
  | 'needs-attention'

export type TemplateLibraryCategoryOption = {
  value: TemplateLibraryCategory
  label: string
}

export type TemplateRecipientVariableMapping = {
  toVariable: string
  ccVariable: string
  bccVariable: string
}

export type TemplateRecipientContext = {
  toText: string
  ccText: string
  bccText: string
}

export type TemplateMailMergePreviewDraftRow = {
  row_id: string
  variables: Record<string, string>
}

const recipientVariableAliases: Record<keyof TemplateRecipientVariableMapping, string[]> = {
  toVariable: ['recipient', 'to', 'email', 'recipient_email'],
  ccVariable: ['cc', 'cc_email'],
  bccVariable: ['bcc', 'bcc_email']
}

export const templateLibraryCategoryOptions: TemplateLibraryCategoryOption[] = [
  { value: 'mail-merge', label: 'Mail merge' },
  { value: 'recipient-aware', label: 'Recipient-aware' },
  { value: 'static-copy', label: 'Static copy' },
  { value: 'needs-attention', label: 'Needs attention' }
]

export function templateLibraryCategoryLabel(category: TemplateLibraryCategory): string {
  return templateLibraryCategoryOptions.find((option) => option.value === category)?.label ?? category
}

export function filterTemplateLibraryTemplates(
  templates: CommunicationTemplate[],
  query: string,
  category: TemplateLibraryCategory | 'all'
): CommunicationTemplate[] {
  const normalizedQuery = query.trim().toLowerCase()
  const categoryFiltered = templates.filter((template) =>
    templateMatchesLibraryCategory(template, category)
  )
  if (!normalizedQuery) return categoryFiltered

  return categoryFiltered.filter((template) => {
    const inName = template.name.toLowerCase().includes(normalizedQuery)
    const inSubject = template.subject_template.toLowerCase().includes(normalizedQuery)
    const inBody = template.body_template.toLowerCase().includes(normalizedQuery)
    const inVariables = template.variables.some((variable) => variable.toLowerCase().includes(normalizedQuery))
    return inName || inSubject || inBody || inVariables
  })
}

export function orderTemplateLibraryTemplates(templates: CommunicationTemplate[]): CommunicationTemplate[] {
  return templates
    .slice()
    .sort((left, right) => {
      const updatedComparison = right.updated_at.localeCompare(left.updated_at)
      if (updatedComparison !== 0) return updatedComparison
      return left.name.localeCompare(right.name, undefined, { sensitivity: 'base' })
    })
}

export function formatTemplateUpdatedLabel(timestamp: string): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric'
  }).format(new Date(timestamp))
}

export function suggestTemplateSaveName(
  subject: string,
  selectedTemplateName = '',
  options: { duplicate: boolean }
): string {
  const normalizedSubject = subject.trim()
  const normalizedSelectedTemplateName = selectedTemplateName.trim()

  if (options.duplicate && normalizedSelectedTemplateName) {
    return `${normalizedSelectedTemplateName} copy`
  }
  if (normalizedSubject) return normalizedSubject
  if (normalizedSelectedTemplateName) return normalizedSelectedTemplateName
  return ''
}

export function deriveTemplateLibraryCategories(
  template: Pick<CommunicationTemplate, 'variables' | 'malformed_placeholders' | 'undeclared_variables'>
): TemplateLibraryCategory[] {
  const categories: TemplateLibraryCategory[] = []

  if (template.variables.length > 0) categories.push('mail-merge')
  if (templateHasRecipientVariables(template.variables)) categories.push('recipient-aware')
  if (template.variables.length === 0) categories.push('static-copy')
  if (template.malformed_placeholders.length > 0 || template.undeclared_variables.length > 0) {
    categories.push('needs-attention')
  }

  return categories
}

export function templateMatchesLibraryCategory(
  template: Pick<CommunicationTemplate, 'variables' | 'malformed_placeholders' | 'undeclared_variables'>,
  category: TemplateLibraryCategory | 'all'
): boolean {
  if (category === 'all') return true
  return deriveTemplateLibraryCategories(template).includes(category)
}

export function inferRecipientVariableMapping(
  variables: string[]
): TemplateRecipientVariableMapping {
  return {
    toVariable: firstRecipientVariableMatch(variables, 'toVariable'),
    ccVariable: firstRecipientVariableMatch(variables, 'ccVariable'),
    bccVariable: firstRecipientVariableMatch(variables, 'bccVariable')
  }
}

export function applyTemplateRecipientMapping(
  currentValues: Record<string, string>,
  mapping: TemplateRecipientVariableMapping,
  context: TemplateRecipientContext
): Record<string, string> {
  const nextValues = { ...currentValues }
  if (mapping.toVariable) nextValues[mapping.toVariable] = context.toText
  if (mapping.ccVariable) nextValues[mapping.ccVariable] = context.ccText
  if (mapping.bccVariable) nextValues[mapping.bccVariable] = context.bccText
  return nextValues
}

export function buildTemplateRecipientPreviewRows(
  templateVariables: string[],
  mapping: TemplateRecipientVariableMapping,
  context: TemplateRecipientContext,
  currentValues: Record<string, string>
): TemplateMailMergePreviewDraftRow[] {
  const toRecipients = splitComposeRecipients(context.toText)
  if (!toRecipients.length || !mapping.toVariable) return []

  const baseValues = templateVariables.reduce<Record<string, string>>((acc, variable) => {
    acc[variable] = currentValues[variable] ?? ''
    return acc
  }, {})

  return toRecipients.map((recipient, index) => ({
    row_id: `recipient-${index + 1}`,
    variables: {
      ...baseValues,
      ...(mapping.ccVariable ? { [mapping.ccVariable]: context.ccText } : {}),
      ...(mapping.bccVariable ? { [mapping.bccVariable]: context.bccText } : {}),
      [mapping.toVariable]: recipient
    }
  }))
}

export function recipientPreviewSummary(context: TemplateRecipientContext): string {
  const toCount = splitComposeRecipients(context.toText).length
  const ccCount = splitComposeRecipients(context.ccText).length
  const bccCount = splitComposeRecipients(context.bccText).length
  return `${toCount} To · ${ccCount} CC · ${bccCount} BCC`
}

export function templateHasRecipientVariables(variables: string[]): boolean {
  return variables.some((variable) => recipientVariableKind(variable) !== null)
}

function firstRecipientVariableMatch(
  variables: string[],
  key: keyof TemplateRecipientVariableMapping
): string {
  return variables.find((variable) => {
    const normalized = variable.trim().toLowerCase()
    return recipientVariableAliases[key].includes(normalized)
  }) ?? ''
}

function recipientVariableKind(
  variable: string
): keyof TemplateRecipientVariableMapping | null {
  const normalized = variable.trim().toLowerCase()
  if (recipientVariableAliases.toVariable.includes(normalized)) return 'toVariable'
  if (recipientVariableAliases.ccVariable.includes(normalized)) return 'ccVariable'
  if (recipientVariableAliases.bccVariable.includes(normalized)) return 'bccVariable'
  return null
}
```

### `frontend/src/domains/communications/components/threadConversationPresentation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/threadConversationPresentation.test.ts`
- Size bytes / Размер в байтах: `1897`
- Included characters / Включено символов: `1897`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { ThreadMessage } from '../types/communications'
import {
  defaultExpandedThreadMessageIds,
  hasQuotedThreadMessages,
  summarizeThreadExpansion
} from './threadConversationPresentation'

function threadMessage(overrides: Partial<ThreadMessage> = {}): ThreadMessage {
  return {
    message_id: 'message-1',
    provider_record_id: 'provider-1',
    account_id: 'account-1',
    subject: 'Quarterly update',
    sender: 'alex@example.com',
    sender_display_name: 'Alex',
    body_text: 'Body',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    workflow_state: 'new',
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    delivery_state: 'received',
    attachment_count: 0,
    attachments: [],
    ...overrides
  }
}

describe('threadConversationPresentation', () => {
  it('auto-expands only the latest message in a thread by default', () => {
    const ids = defaultExpandedThreadMessageIds([
      threadMessage({ message_id: 'message-1' }),
      threadMessage({ message_id: 'message-2' })
    ])

    expect([...ids]).toEqual(['message-2'])
  })

  it('detects whether any thread message has quoted content', () => {
    expect(hasQuotedThreadMessages([
      threadMessage({ body_text: 'Body only' }),
      threadMessage({ body_text: 'Reply\n\nOn Tue, Alex wrote:\n> Prior note' })
    ])).toBe(true)

    expect(hasQuotedThreadMessages([
      threadMessage({ body_text: 'Body only' })
    ])).toBe(false)
  })

  it('summarizes expansion controls from messages and expanded ids', () => {
    expect(summarizeThreadExpansion(
      [
        threadMessage({ message_id: 'message-1' }),
        threadMessage({ message_id: 'message-2' })
      ],
      new Set(['message-2'])
    )).toEqual({
      expandedCount: 1,
      canExpandAll: true,
      canCollapseAll: true
    })
  })
})
```

### `frontend/src/domains/communications/components/threadConversationPresentation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/threadConversationPresentation.ts`
- Size bytes / Размер в байтах: `907`
- Included characters / Включено символов: `907`
- Truncated / Обрезано: `no`

```typescript
import type { ThreadMessage } from '../types/communications'
import { splitThreadMessageBody } from './threadMessageBody'

export function defaultExpandedThreadMessageIds(messages: ThreadMessage[]): Set<string> {
  const latestMessage = messages.at(-1)
  return latestMessage ? new Set([latestMessage.message_id]) : new Set()
}

export function hasQuotedThreadMessages(messages: ThreadMessage[]): boolean {
  return messages.some((message) => splitThreadMessageBody(message.body_text).quotedText.length > 0)
}

export function summarizeThreadExpansion(
  messages: ThreadMessage[],
  expandedMessageIds: ReadonlySet<string>
): {
  expandedCount: number
  canExpandAll: boolean
  canCollapseAll: boolean
} {
  return {
    expandedCount: expandedMessageIds.size,
    canExpandAll: messages.some((message) => !expandedMessageIds.has(message.message_id)),
    canCollapseAll: expandedMessageIds.size > 0
  }
}
```

### `frontend/src/domains/communications/components/threadMessageBody.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/threadMessageBody.test.ts`
- Size bytes / Размер в байтах: `1673`
- Included characters / Включено символов: `1673`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { previewThreadMessageBody, splitThreadMessageBody } from './threadMessageBody'
import type { ThreadMessage } from '../types/communications'

function threadMessage(overrides: Partial<ThreadMessage> = {}): ThreadMessage {
  return {
    message_id: 'message-1',
    provider_record_id: 'provider-1',
    account_id: 'account-1',
    subject: 'Quarterly update',
    sender: 'alex@example.com',
    sender_display_name: 'Alex',
    body_text: 'Body',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    workflow_state: 'new',
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    delivery_state: 'received',
    attachment_count: 0,
    attachments: [],
    ...overrides
  }
}

describe('threadMessageBody', () => {
  it('splits quoted reply tails from the main body', () => {
    expect(splitThreadMessageBody('Thanks for the update.\n\nOn Tue, Alex wrote:\n> Prior note')).toEqual({
      mainText: 'Thanks for the update.',
      quotedText: 'On Tue, Alex wrote:\n> Prior note'
    })
  })

  it('returns the full body as main text when no quoted segment exists', () => {
    expect(splitThreadMessageBody('Line one\nLine two')).toEqual({
      mainText: 'Line one\nLine two',
      quotedText: ''
    })
  })

  it('uses the non-quoted body for expanded preview text', () => {
    const message = threadMessage({
      body_text: 'Thanks for the update.\n\nOn Tue, Alex wrote:\n> Prior note'
    })

    expect(previewThreadMessageBody(message, true)).toBe('Thanks for the update.')
    expect(previewThreadMessageBody(message, false)).toContain('On Tue, Alex wrote:')
  })
})
```

### `frontend/src/domains/communications/components/threadMessageBody.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/threadMessageBody.ts`
- Size bytes / Размер в байтах: `1253`
- Included characters / Включено символов: `1253`
- Truncated / Обрезано: `no`

```typescript
import type { ThreadMessage } from '../types/communications'

export type ThreadMessageBodySegments = {
  mainText: string
  quotedText: string
}

export function splitThreadMessageBody(bodyText: string): ThreadMessageBodySegments {
  const normalized = bodyText.trim()
  if (!normalized) {
    return {
      mainText: '',
      quotedText: ''
    }
  }

  const lines = normalized.split('\n')
  const quotedStart = lines.findIndex((line, index) => {
    const trimmed = line.trim()
    if (trimmed.startsWith('>')) return true
    if (index === 0) return false
    return /^On .+ wrote:$/i.test(trimmed)
  })

  if (quotedStart <= 0) {
    return {
      mainText: normalized,
      quotedText: quotedStart === 0 ? normalized : ''
    }
  }

  return {
    mainText: lines.slice(0, quotedStart).join('\n').trim(),
    quotedText: lines.slice(quotedStart).join('\n').trim()
  }
}

export function previewThreadMessageBody(message: ThreadMessage, expanded: boolean): string {
  const { mainText, quotedText } = splitThreadMessageBody(message.body_text)
  const source = expanded ? (mainText || quotedText) : message.body_text
  const compact = source.trim().replace(/\s+/g, ' ')
  return compact.length > 220 ? `${compact.slice(0, 220)}...` : compact
}
```

### `frontend/src/domains/communications/components/useCommunicationFolderReorder.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/useCommunicationFolderReorder.ts`
- Size bytes / Размер в байтах: `2675`
- Included characters / Включено символов: `2675`
- Truncated / Обрезано: `no`

```typescript
import { ref, type ComputedRef } from 'vue'
import type { CommunicationFolder, CommunicationFolderUpdate } from '../types/folders'
import {
  MAIL_FOLDER_REORDER_DRAG_TYPE,
  buildCommunicationFolderReorderUpdates,
  createCommunicationFolderReorderPayload,
  hasCommunicationFolderReorderDragType,
  mailFolderReorderStatus,
  parseCommunicationFolderReorderPayload
} from './mailFolderOrdering'

type UpdateFolder = (variables: { folderId: string; request: CommunicationFolderUpdate }) => Promise<CommunicationFolder>

export function useCommunicationFolderReorder(
  folders: ComputedRef<CommunicationFolder[]>,
  updateFolder: UpdateFolder
) {
  const sourceId = ref('')
  const targetId = ref('')
  const status = ref('')
  const error = ref('')
  const isReordering = ref(false)

  function canHandleDragOver(event: DragEvent): boolean {
    return Boolean(event.dataTransfer && hasCommunicationFolderReorderDragType(event.dataTransfer.types) && !isReordering.value)
  }

  function handleDragStart(event: DragEvent, folder: CommunicationFolder) {
    if (!event.dataTransfer) return
    sourceId.value = folder.folder_id
    status.value = ''
    error.value = ''
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData(MAIL_FOLDER_REORDER_DRAG_TYPE, createCommunicationFolderReorderPayload(folder.folder_id))
  }

  function handleDragEnd() {
    sourceId.value = ''
    targetId.value = ''
  }

  async function handleDrop(event: DragEvent, folder: CommunicationFolder): Promise<boolean> {
    if (!event.dataTransfer || isReordering.value) return false
    const payload = parseCommunicationFolderReorderPayload(event.dataTransfer.getData(MAIL_FOLDER_REORDER_DRAG_TYPE))
    if (!payload) return false

    const updates = buildCommunicationFolderReorderUpdates(folders.value, payload.folder_id, folder.folder_id)
    if (updates.length === 0) return true

    targetId.value = folder.folder_id
    status.value = ''
    error.value = ''
    isReordering.value = true
    try {
      for (const update of updates) {
        await updateFolder({ folderId: update.folderId, request: { sort_order: update.sortOrder } })
      }
      status.value = mailFolderReorderStatus(folders.value, payload.folder_id, folder.folder_id)
      return true
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : 'Folder reorder failed'
      return true
    } finally {
      isReordering.value = false
      sourceId.value = ''
      targetId.value = ''
    }
  }

  return {
    canHandleDragOver,
    error,
    handleDragEnd,
    handleDragStart,
    handleDrop,
    isReordering,
    sourceId,
    status,
    targetId
  }
}
```

### `frontend/src/domains/communications/constants/sectionTabs.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/constants/sectionTabs.ts`
- Size bytes / Размер в байтах: `555`
- Included characters / Включено символов: `555`
- Truncated / Обрезано: `no`

```typescript
import type { CommunicationSectionId } from '../types/communications'

export const communicationSectionTabs: {
  id: CommunicationSectionId
  label: string
  icon: string
}[] = [
  { id: 'unified', label: 'Unified', icon: 'tabler:inbox' },
  { id: 'inbox', label: 'Inbox', icon: 'tabler:mail' },
  { id: 'needs_reply', label: 'Need Reply', icon: 'tabler:message-reply' },
  { id: 'waiting', label: 'Waiting', icon: 'tabler:clock' },
  { id: 'done', label: 'Done', icon: 'tabler:check' },
  { id: 'archived', label: 'Archived', icon: 'tabler:archive' }
]
```

### `frontend/src/domains/communications/forms/attachmentSearchForm.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/attachmentSearchForm.test.ts`
- Size bytes / Размер в байтах: `796`
- Included characters / Включено символов: `796`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  attachmentSearchFormDefaults,
  attachmentSearchFormToRequest
} from './attachmentSearchForm'

describe('attachment search form', () => {
  it('builds a bounded attachment search request from normalized form values', () => {
    expect(attachmentSearchFormToRequest({
      query: ' invoice ',
      content_type: ' application/pdf ',
      scan_status: 'not_scanned'
    }, ' account-1 ')).toEqual({
      account_id: 'account-1',
      q: 'invoice',
      content_type: 'application/pdf',
      scan_status: 'not_scanned',
      limit: 50
    })
  })

  it('keeps blank optional filters out of the request', () => {
    expect(attachmentSearchFormToRequest(attachmentSearchFormDefaults(), null)).toEqual({
      limit: 50
    })
  })
})
```

### `frontend/src/domains/communications/forms/attachmentSearchForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/attachmentSearchForm.ts`
- Size bytes / Размер в байтах: `1438`
- Included characters / Включено символов: `1438`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { AttachmentScanStatus, AttachmentSearchRequest } from '../types/attachments'

export const attachmentScanStatusOptions = [
  'not_scanned',
  'clean',
  'suspicious',
  'malicious',
  'failed'
] as const

export const attachmentSearchFormSchema = z.object({
  query: z.string().trim().max(500, 'Search query is too long'),
  content_type: z.string().trim().max(120, 'Content type is too long'),
  scan_status: z.union([z.literal(''), z.enum(attachmentScanStatusOptions)])
})

export type AttachmentSearchFormValues = z.infer<typeof attachmentSearchFormSchema>

export const attachmentSearchVeeValidationSchema = toTypedSchema(attachmentSearchFormSchema)

export function attachmentSearchFormDefaults(): AttachmentSearchFormValues {
  return {
    query: '',
    content_type: '',
    scan_status: ''
  }
}

export function attachmentSearchFormToRequest(
  values: AttachmentSearchFormValues,
  accountId: string | null
): AttachmentSearchRequest {
  const parsed = attachmentSearchFormSchema.parse(values)
  const request: AttachmentSearchRequest = { limit: 50 }
  if (accountId?.trim()) request.account_id = accountId.trim()
  if (parsed.query) request.q = parsed.query
  if (parsed.content_type) request.content_type = parsed.content_type
  if (parsed.scan_status) request.scan_status = parsed.scan_status as AttachmentScanStatus
  return request
}
```

### `frontend/src/domains/communications/forms/bilingualReplyFlowForm.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/bilingualReplyFlowForm.test.ts`
- Size bytes / Размер в байтах: `1511`
- Included characters / Включено символов: `1447`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  bilingualReplyFlowFormDefaults,
  bilingualReplyFlowFormSchema,
  bilingualReplyFlowFormToRequest,
  bilingualReplyToneOptions
} from './bilingualReplyFlowForm'

describe('bilingual reply flow form', () => {
  it('normalizes Russian reply text and the selected tone into an API request', () => {
    const values = bilingualReplyFlowFormSchema.parse({
      replyTextRu: '  Спасибо, мы проверим контракт сегодня.  ',
      tone: 'business'
    })

    expect(bilingualReplyFlowFormToRequest(values)).toEqual({
      reply_text_ru: 'Спасибо, мы проверим контракт сегодня.',
      tone: 'business'
    })
  })

  it('supports the required Макошь bilingual reply tones', () => {
    expect(bilingualReplyToneOptions).toEqual([
      'formal',
      'business',
      'friendly',
      'short',
      'detailed'
    ])
  })

  it('rejects empty replies, unsupported tones and oversized reply text', () => {
    const result = bilingualReplyFlowFormSchema.safeParse({
      replyTextRu: ' ',
      tone: 'casual'
    })

    expect(result.success).toBe(false)

    expect(() =>
      bilingualReplyFlowFormSchema.parse({
        replyTextRu: 'x'.repeat(64001),
        tone: 'formal'
      })
    ).toThrow()
  })

  it('uses a business tone and empty draft by default', () => {
    expect(bilingualReplyFlowFormDefaults()).toEqual({
      replyTextRu: '',
      tone: 'business'
    })
  })
})
```

### `frontend/src/domains/communications/forms/bilingualReplyFlowForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/bilingualReplyFlowForm.ts`
- Size bytes / Размер в байтах: `1087`
- Included characters / Включено символов: `1087`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import {
  bilingualReplyToneOptions,
  type BilingualReplyFlowRequest,
  type BilingualReplyTone
} from '../types/bilingualReplyFlow'

export { bilingualReplyToneOptions }

export const bilingualReplyFlowFormSchema = z.object({
  replyTextRu: z
    .string()
    .trim()
    .min(1, 'Russian reply is required')
    .max(64_000, 'Russian reply is too long'),
  tone: z.enum(bilingualReplyToneOptions)
})

export type BilingualReplyFlowFormValues = z.infer<typeof bilingualReplyFlowFormSchema>

export const bilingualReplyFlowVeeValidationSchema = toTypedSchema(bilingualReplyFlowFormSchema)

export function bilingualReplyFlowFormDefaults(): BilingualReplyFlowFormValues {
  return {
    replyTextRu: '',
    tone: 'business'
  }
}

export function bilingualReplyFlowFormToRequest(
  values: BilingualReplyFlowFormValues
): BilingualReplyFlowRequest {
  const parsed = bilingualReplyFlowFormSchema.parse(values)
  return {
    reply_text_ru: parsed.replyTextRu,
    tone: parsed.tone as BilingualReplyTone
  }
}
```

### `frontend/src/domains/communications/forms/certificateForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/certificateForm.ts`
- Size bytes / Размер в байтах: `2791`
- Included characters / Включено символов: `2791`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import {
  certificateProviderOptions,
  certificateStorageKindOptions,
  certificateTrustStatusOptions,
  certificateTypeOptions,
  type CertificateProvider,
  type CertificateStorageKind,
  type CertificateTrustStatus,
  type CertificateType,
  type MailCertificateCreateRequest
} from '../types/certificates'

const optionalTrimmed = z.string().trim().optional().transform((value) => value || undefined)

export const certificateFormSchema = z.object({
  cert_id: z.string().trim().min(1, 'Certificate id is required'),
  owner_name: z.string().trim().min(1, 'Owner is required'),
  issuer: z.string().trim().min(1, 'Issuer is required'),
  fingerprint_sha256: optionalTrimmed,
  valid_until: optionalTrimmed,
  cert_type: z.enum(certificateTypeOptions as [CertificateType, ...CertificateType[]]),
  provider: z.enum(certificateProviderOptions as [CertificateProvider, ...CertificateProvider[]]),
  storage_kind: z.enum(certificateStorageKindOptions as [CertificateStorageKind, ...CertificateStorageKind[]]),
  storage_ref: optionalTrimmed,
  trust_status: z.enum(certificateTrustStatusOptions as [CertificateTrustStatus, ...CertificateTrustStatus[]]),
  usage: optionalTrimmed
})

export type CertificateFormValues = z.input<typeof certificateFormSchema>

export const certificateVeeValidationSchema = toTypedSchema(certificateFormSchema)

export function certificateFormDefaults(): CertificateFormValues {
  return {
    cert_id: '',
    owner_name: '',
    issuer: '',
    fingerprint_sha256: '',
    valid_until: '',
    cert_type: 'smime',
    provider: 'other',
    storage_kind: 'encrypted_vault',
    storage_ref: '',
    trust_status: 'pending_verification',
    usage: 'signing, encryption'
  }
}

export function certificateFormToCreateRequest(values: CertificateFormValues): MailCertificateCreateRequest {
  const parsed = certificateFormSchema.parse(values)
  return {
    cert_id: parsed.cert_id,
    owner_name: parsed.owner_name,
    issuer: parsed.issuer,
    fingerprint_sha256: parsed.fingerprint_sha256 ?? null,
    valid_until: localDateTimeToIso(parsed.valid_until),
    cert_type: parsed.cert_type,
    provider: parsed.provider,
    storage_kind: parsed.storage_kind,
    storage_ref: parsed.storage_ref ?? null,
    trust_status: parsed.trust_status,
    usage: splitUsage(parsed.usage),
    metadata: {}
  }
}

function splitUsage(value: string | undefined): string[] {
  if (!value) return []
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
}

function localDateTimeToIso(value: string | undefined): string | null {
  if (!value) return null
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return null
  return date.toISOString()
}
```

### `frontend/src/domains/communications/forms/composeDraftAutosave.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/composeDraftAutosave.test.ts`
- Size bytes / Размер в байтах: `3840`
- Included characters / Включено символов: `3840`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { ComposeFormModel } from '../types/communications'
import {
	buildComposeDraftPayload,
	composeDraftHasAutosaveContent,
	useComposeDraftAutosave
} from './composeDraftAutosave'

function composeForm(overrides: Partial<ComposeFormModel> = {}): ComposeFormModel {
	return {
		mode: 'compose',
		draftId: 'draft-1',
		accountId: 'account-1',
		toText: '',
		ccText: '',
		bccText: '',
		subject: '',
		body: '',
		bodyHtml: null,
		bodyFormat: 'plain',
		scheduledSendAt: '',
		undoSendSeconds: null,
		inReplyTo: null,
		...overrides
	}
}

describe('compose draft autosave', () => {
	afterEach(() => {
		vi.useRealTimers()
	})

	it('builds an autosave draft payload from the compose form', () => {
		const payload = buildComposeDraftPayload(
			composeForm({
				mode: 'reply',
				toText: 'Alex <alex@example.com>, team@example.org',
				ccText: 'copy@example.org',
				subject: '',
				body: 'Autosaved body',
				inReplyTo: 'provider-message-1'
			})
		)

		expect(payload).toEqual({
			draft_id: 'draft-1',
			account_id: 'account-1',
			to_recipients: ['alex@example.com', 'team@example.org'],
			cc_recipients: ['copy@example.org'],
			bcc_recipients: [],
			subject: '',
			body_text: 'Autosaved body',
			body_html: null,
			in_reply_to: 'provider-message-1',
			scheduled_send_at: null,
			status: 'draft',
			metadata: { compose_mode: 'reply' }
		})
	})

	it('detects whether a draft has content worth autosaving', () => {
		expect(composeDraftHasAutosaveContent(composeForm())).toBe(false)
		expect(composeDraftHasAutosaveContent(composeForm({ toText: 'recipient@example.com' }))).toBe(true)
		expect(composeDraftHasAutosaveContent(composeForm({ body: 'Draft body' }))).toBe(true)
		expect(composeDraftHasAutosaveContent(composeForm({ bodyHtml: '<p>Draft body</p>', bodyFormat: 'html' }))).toBe(true)
	})

	it('includes body_html for HTML drafts', () => {
		const payload = buildComposeDraftPayload(
			composeForm({
				body: 'Plain fallback',
				bodyHtml: '<p>HTML body</p>',
				bodyFormat: 'html'
			})
		)

		expect(payload.body_text).toBe('Plain fallback')
		expect(payload.body_html).toBe('<p>HTML body</p>')
	})

	it('includes scheduled_send_at for scheduled drafts', () => {
		const payload = buildComposeDraftPayload(
			composeForm({
				scheduledSendAt: '2026-06-15T10:30'
			})
		)

		expect(payload.scheduled_send_at).toBe(new Date('2026-06-15T10:30').toISOString())
	})

	it('debounces autosave and persists the latest compose state', async () => {
		vi.useFakeTimers()
		let currentForm = composeForm({ body: 'First body' })
		const saveDraft = vi.fn().mockResolvedValue(undefined)
		const autosave = useComposeDraftAutosave({
			delayMs: 2000,
			formSource: () => currentForm,
			saveDraft
		})

		autosave.schedule()
		currentForm = composeForm({ body: 'Second body' })
		autosave.schedule()

		await vi.advanceTimersByTimeAsync(1999)
		expect(saveDraft).not.toHaveBeenCalled()

		await vi.advanceTimersByTimeAsync(1)
		expect(saveDraft).toHaveBeenCalledOnce()
		expect(saveDraft).toHaveBeenCalledWith({
			draft_id: 'draft-1',
			account_id: 'account-1',
			to_recipients: [],
			cc_recipients: [],
			bcc_recipients: [],
			subject: '',
			body_text: 'Second body',
			body_html: null,
			in_reply_to: null,
			scheduled_send_at: null,
			status: 'draft',
			metadata: { compose_mode: 'compose' }
		})
	})

	it('flushes a pending autosave without saving twice', async () => {
		vi.useFakeTimers()
		const saveDraft = vi.fn().mockResolvedValue(undefined)
		const autosave = useComposeDraftAutosave({
			delayMs: 2000,
			formSource: () => composeForm({ subject: 'Flush me' }),
			saveDraft
		})

		autosave.schedule()
		await autosave.flush()
		await vi.advanceTimersByTimeAsync(2000)

		expect(saveDraft).toHaveBeenCalledOnce()
	})
})
```

### `frontend/src/domains/communications/forms/composeDraftAutosave.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/composeDraftAutosave.ts`
- Size bytes / Размер в байтах: `2997`
- Included characters / Включено символов: `2997`
- Truncated / Обрезано: `no`

```typescript
import { getCurrentScope, onScopeDispose } from 'vue'
import type { ComposeFormModel } from '../types/communications'
import { splitComposeRecipients } from './composeValidation'

const DEFAULT_AUTOSAVE_DELAY_MS = 2000

export type ComposeDraftPayload = {
	draft_id: string
	account_id: string
	to_recipients: string[]
	cc_recipients: string[]
	bcc_recipients: string[]
	subject: string
	body_text: string
	body_html: string | null
	in_reply_to: string | null
	scheduled_send_at: string | null
	status: 'draft'
	metadata: {
		compose_mode: ComposeFormModel['mode']
	}
}

export type ComposeDraftAutosaveOptions = {
	delayMs?: number
	formSource: () => ComposeFormModel
	saveDraft: (payload: ComposeDraftPayload) => Promise<unknown>
	onSaved?: () => void
	onError?: (error: unknown) => void
}

export function buildComposeDraftPayload(form: ComposeFormModel): ComposeDraftPayload {
	return {
		draft_id: form.draftId,
		account_id: form.accountId,
		to_recipients: splitComposeRecipients(form.toText),
		cc_recipients: splitComposeRecipients(form.ccText),
		bcc_recipients: splitComposeRecipients(form.bccText),
		subject: form.subject,
		body_text: form.body,
		body_html: form.bodyFormat === 'html' ? form.bodyHtml : null,
		in_reply_to: form.inReplyTo,
		scheduled_send_at: datetimeLocalToIso(form.scheduledSendAt),
		status: 'draft',
		metadata: { compose_mode: form.mode }
	}
}

export function composeDraftHasAutosaveContent(form: ComposeFormModel): boolean {
	return Boolean(
		form.toText.trim() ||
			form.ccText.trim() ||
			form.bccText.trim() ||
			form.subject.trim() ||
			form.body.trim() ||
			Boolean(form.bodyHtml?.trim()) ||
			Boolean(form.scheduledSendAt.trim())
	)
}

export function datetimeLocalToIso(value: string): string | null {
	const trimmed = value.trim()
	if (!trimmed) return null
	const date = new Date(trimmed)
	return Number.isFinite(date.getTime()) ? date.toISOString() : null
}

export function useComposeDraftAutosave(options: ComposeDraftAutosaveOptions) {
	let timer: ReturnType<typeof setTimeout> | null = null
	const delayMs = options.delayMs ?? DEFAULT_AUTOSAVE_DELAY_MS

	function clearTimer(): void {
		if (!timer) return
		clearTimeout(timer)
		timer = null
	}

	function canSave(): boolean {
		const form = options.formSource()
		return Boolean(
			form.draftId.trim() &&
				form.accountId.trim() &&
				composeDraftHasAutosaveContent(form)
		)
	}

	async function saveNow(): Promise<void> {
		if (!canSave()) return

		try {
			await options.saveDraft(buildComposeDraftPayload(options.formSource()))
			options.onSaved?.()
		} catch (error) {
			options.onError?.(error)
		}
	}

	function schedule(): void {
		clearTimer()
		if (!canSave()) return
		timer = setTimeout(() => {
			void saveNow()
		}, delayMs)
	}

	async function flush(): Promise<void> {
		clearTimer()
		await saveNow()
	}

	function cancel(): void {
		clearTimer()
	}

	if (getCurrentScope()) {
		onScopeDispose(cancel)
	}

	return {
		cancel,
		flush,
		saveNow,
		schedule
	}
}
```

### `frontend/src/domains/communications/forms/composeValidation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/composeValidation.test.ts`
- Size bytes / Размер в байтах: `1971`
- Included characters / Включено символов: `1971`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  composeSendSchema,
  splitComposeRecipients,
  toComposeValidationValues
} from './composeValidation'

describe('compose validation', () => {
  it('splits comma-separated recipients and extracts angle-bracket addresses', () => {
    expect(splitComposeRecipients('Alex <alex@example.com>, team@example.org')).toEqual([
      'alex@example.com',
      'team@example.org'
    ])
  })

  it('accepts a valid send form', () => {
    const parsed = composeSendSchema.parse({
      accountId: 'account-1',
      toText: 'recipient@example.com',
      ccText: '',
      bccText: '',
      subject: 'Quarterly update',
      body: 'Hello',
      inReplyTo: null
    })

    expect(parsed.toText).toBe('recipient@example.com')
  })

  it('rejects missing account and invalid recipients', () => {
    const result = composeSendSchema.safeParse({
      accountId: '',
      toText: 'not-an-email',
      ccText: 'copy@example.com',
      bccText: '',
      subject: 'Quarterly update',
      body: 'Hello',
      inReplyTo: null
    })

    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues.map((issue) => issue.path.join('.'))).toEqual([
        'accountId',
        'toText'
      ])
    }
  })

  it('maps the store compose model into validation values', () => {
    expect(toComposeValidationValues({
      mode: 'reply',
      draftId: 'draft-1',
      accountId: 'account-1',
      toText: 'recipient@example.com',
      ccText: '',
      bccText: '',
      subject: 'Re: Update',
      body: 'Thanks',
      bodyHtml: null,
      bodyFormat: 'plain',
      scheduledSendAt: '',
      undoSendSeconds: null,
      inReplyTo: 'provider-message-1'
    })).toEqual({
      accountId: 'account-1',
      toText: 'recipient@example.com',
      ccText: '',
      bccText: '',
      subject: 'Re: Update',
      body: 'Thanks',
      inReplyTo: 'provider-message-1'
    })
  })
})
```

### `frontend/src/domains/communications/forms/composeValidation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/composeValidation.ts`
- Size bytes / Размер в байтах: `2976`
- Included characters / Включено символов: `2976`
- Truncated / Обрезано: `no`

```typescript
import { watch } from 'vue'
import { useForm } from 'vee-validate'
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { ComposeFormModel } from '../types/communications'

const MAX_EMAIL_SUBJECT_LENGTH = 998
const MAX_EMAIL_BODY_LENGTH = 1_000_000
const EMAIL_ADDRESS_PATTERN = /^[^\s@<>]+@[^\s@<>]+\.[^\s@<>]+$/

export type ComposeValidationValues = {
  accountId: string
  toText: string
  ccText: string
  bccText: string
  subject: string
  body: string
  inReplyTo: string | null
}

export function splitComposeRecipients(value: string): string[] {
  return value
    .split(',')
    .map((recipient) => normalizeRecipientAddress(recipient))
    .filter(Boolean)
}

export function toComposeValidationValues(form: ComposeFormModel): ComposeValidationValues {
  return {
    accountId: form.accountId,
    toText: form.toText,
    ccText: form.ccText,
    bccText: form.bccText,
    subject: form.subject,
    body: form.body,
    inReplyTo: form.inReplyTo
  }
}

const requiredRecipientsSchema = z
  .string()
  .trim()
  .refine((value) => splitComposeRecipients(value).length > 0, {
    message: 'At least one recipient is required'
  })
  .refine((value) => recipientsAreValid(value), {
    message: 'Recipient list contains an invalid email address'
  })

const optionalRecipientsSchema = z
  .string()
  .trim()
  .refine((value) => value === '' || recipientsAreValid(value), {
    message: 'Recipient list contains an invalid email address'
  })

export const composeSendSchema = z.object({
  accountId: z.string().trim().min(1, 'Mail account is required'),
  toText: requiredRecipientsSchema,
  ccText: optionalRecipientsSchema,
  bccText: optionalRecipientsSchema,
  subject: z.string().max(MAX_EMAIL_SUBJECT_LENGTH, 'Subject is too long'),
  body: z.string().max(MAX_EMAIL_BODY_LENGTH, 'Message body is too long'),
  inReplyTo: z.string().nullable()
})

export const composeVeeValidationSchema = toTypedSchema(composeSendSchema)

export function useComposeValidation(formSource: () => ComposeFormModel) {
  const { errors, setValues, validate } = useForm<ComposeValidationValues>({
    validationSchema: composeVeeValidationSchema,
    initialValues: toComposeValidationValues(formSource()),
    validateOnMount: false
  })

  watch(
    () => toComposeValidationValues(formSource()),
    (values) => setValues(values, false),
    { deep: true }
  )

  async function validateForSend(): Promise<boolean> {
    const result = await validate()
    return result.valid
  }

  return {
    errors,
    validateForSend
  }
}

function recipientsAreValid(value: string): boolean {
  const recipients = splitComposeRecipients(value)
  return recipients.length > 0 && recipients.every((recipient) => EMAIL_ADDRESS_PATTERN.test(recipient))
}

function normalizeRecipientAddress(value: string): string {
  const trimmed = value.trim()
  const angleAddress = trimmed.match(/<([^<>]+)>$/)
  return (angleAddress?.[1] ?? trimmed).trim()
}
```

### `frontend/src/domains/communications/forms/mailFolderForm.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/mailFolderForm.test.ts`
- Size bytes / Размер в байтах: `3669`
- Included characters / Включено символов: `3669`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
	composeCommunicationFolderName,
	mailFolderDeleteDialogCopy,
	mailFolderFormDefaults,
	mailFolderParentPathOptions,
	mailFolderFormSchema,
	mailFolderFormToInput,
	mailFolderMessageCountLabel,
	splitCommunicationFolderName,
	validateCommunicationFolderParentPath
} from './mailFolderForm'
import type { CommunicationFolder } from '../types/folders'

describe('mail folder form', () => {
	it('normalizes form values into a custom folder input payload', () => {
		const values = mailFolderFormSchema.parse({
			name: '  Client Projects  ',
			description: '  Active client mail  ',
			color: '  #3b82f6  ',
			sort_order: 12.8
		})

		expect(mailFolderFormToInput(values, 'account-1')).toEqual({
			account_id: 'account-1',
			name: 'Client Projects',
			description: 'Active client mail',
			color: '#3b82f6',
			sort_order: 12
		})
	})

	it('allows global folders and clears empty optional fields', () => {
		const values = mailFolderFormSchema.parse({
			name: 'Receipts',
			description: ' ',
			color: '',
			sort_order: 0
		})

		expect(mailFolderFormToInput(values, null)).toEqual({
			account_id: null,
			name: 'Receipts',
			description: null,
			color: null,
			sort_order: 0
		})
	})

	it('rejects empty names, long descriptions and invalid colors', () => {
		const result = mailFolderFormSchema.safeParse({
			name: ' ',
			description: 'x'.repeat(501),
			color: 'blue',
			sort_order: -1
		})

		expect(result.success).toBe(false)
		if (!result.success) {
			expect(result.error.issues.map((issue) => issue.path.join('.'))).toEqual([
				'name',
				'description',
				'color',
				'sort_order'
			])
		}
	})

	it('uses existing folder values as edit defaults', () => {
		expect(mailFolderFormDefaults(folder())).toEqual({
			name: 'Clients',
			description: 'Relationship mail',
			color: '#10b981',
			sort_order: 7
		})
	})

	it('splits and composes hierarchy-aware folder names', () => {
		expect(splitCommunicationFolderName('Projects / Client A / Q1')).toEqual({
			parentPath: 'Projects / Client A',
			leafName: 'Q1'
		})
		expect(composeCommunicationFolderName(' Projects / Client A ', ' Q1 ')).toBe('Projects / Client A / Q1')
		expect(composeCommunicationFolderName('', 'Inbox')).toBe('Inbox')
	})

	it('builds parent path suggestions and rejects self-descendant parents', () => {
		const folders = [
			folder(),
			{ ...folder(), folder_id: 'mail_folder:2', name: 'Clients / Acme' },
			{ ...folder(), folder_id: 'mail_folder:3', name: 'Finance' }
		]

		expect(mailFolderParentPathOptions(folders, folders[0])).toEqual(['Finance'])
		expect(validateCommunicationFolderParentPath('Clients', folders[0])).toBe('Folder cannot be its own parent')
		expect(validateCommunicationFolderParentPath('Clients / Acme', folders[0])).toBe(
			'Folder cannot move inside one of its child paths'
		)
		expect(validateCommunicationFolderParentPath('Finance', folders[0])).toBe('')
	})

	it('builds delete confirmation copy and compact message counts', () => {
		expect(mailFolderDeleteDialogCopy(folder())).toEqual({
			title: 'Delete folder',
			message: 'Delete the folder "Clients"? This does not delete messages.',
			confirmLabel: 'Delete'
		})
		expect(mailFolderMessageCountLabel({ message_count: -10 })).toBe('0')
		expect(mailFolderMessageCountLabel({ message_count: 42 })).toBe('42')
	})
})

function folder(): CommunicationFolder {
	return {
		folder_id: 'mail_folder:1',
		account_id: 'account-1',
		name: 'Clients',
		description: 'Relationship mail',
		color: '#10b981',
		sort_order: 7,
		message_count: 2,
		created_at: '2026-06-15T00:00:00Z',
		updated_at: '2026-06-15T00:00:00Z'
	}
}
```

### `frontend/src/domains/communications/forms/mailFolderForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/mailFolderForm.ts`
- Size bytes / Размер в байтах: `4515`
- Included characters / Включено символов: `4515`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { CommunicationFolder, CommunicationFolderInput } from '../types/folders'

const HEX_COLOR_PATTERN = /^#[0-9a-fA-F]{6}$/

export type CommunicationFolderFormValues = z.infer<typeof mailFolderFormSchema>
export type CommunicationFolderDeleteDialogCopy = {
	title: string
	message: string
	confirmLabel: string
}
export type CommunicationFolderNameParts = {
	parentPath: string
	leafName: string
}

export const mailFolderFormSchema = z.object({
	name: z.string().trim().min(1, 'Name is required').max(120, 'Name is too long'),
	description: z.string().trim().max(500, 'Description is too long'),
	color: z
		.string()
		.trim()
		.refine((value) => value === '' || HEX_COLOR_PATTERN.test(value), {
			message: 'Color must be a hex color'
		}),
	sort_order: z.coerce
		.number()
		.min(0, 'Sort order cannot be negative')
		.transform((value) => Math.trunc(value))
})

export const mailFolderVeeValidationSchema = toTypedSchema(mailFolderFormSchema)

export function mailFolderFormDefaults(folder?: CommunicationFolder | null): CommunicationFolderFormValues {
	return {
		name: folder?.name ?? '',
		description: folder?.description ?? '',
		color: folder?.color ?? '',
		sort_order: folder?.sort_order ?? 0
	}
}

export function splitCommunicationFolderName(name: string): CommunicationFolderNameParts {
	const parts = normalizeCommunicationFolderPathParts(name)
	if (parts.length === 0) {
		return {
			parentPath: '',
			leafName: ''
		}
	}

	return {
		parentPath: parts.slice(0, -1).join(' / '),
		leafName: parts[parts.length - 1] ?? ''
	}
}

export function composeCommunicationFolderName(parentPath: string, leafName: string): string {
	const parts = [
		...normalizeCommunicationFolderPathParts(parentPath),
		...normalizeCommunicationFolderPathParts(leafName)
	]
	return parts.join(' / ')
}

export function mailFolderParentPathOptions(
	folders: ReadonlyArray<Pick<CommunicationFolder, 'folder_id' | 'name'>>,
	editingFolder?: Pick<CommunicationFolder, 'folder_id' | 'name'> | null
): string[] {
	const currentPath = normalizeCommunicationFolderPath(editingFolder?.name ?? '')
	const unique = new Set<string>()
	const options: string[] = []

	for (const folder of folders) {
		const normalizedPath = normalizeCommunicationFolderPath(folder.name)
		if (!normalizedPath) continue
		if (currentPath && isSameOrDescendantPath(normalizedPath, currentPath)) continue
		if (unique.has(normalizedPath)) continue
		unique.add(normalizedPath)
		options.push(normalizedPath)
	}

	return options
}

export function validateCommunicationFolderParentPath(
	parentPath: string,
	editingFolder?: Pick<CommunicationFolder, 'name'> | null
): string {
	const normalizedParentPath = normalizeCommunicationFolderPath(parentPath)
	const currentPath = normalizeCommunicationFolderPath(editingFolder?.name ?? '')
	if (!normalizedParentPath || !currentPath) return ''
	if (normalizedParentPath === currentPath) return 'Folder cannot be its own parent'
	if (isSameOrDescendantPath(normalizedParentPath, currentPath)) {
		return 'Folder cannot move inside one of its child paths'
	}
	return ''
}

export function mailFolderFormToInput(
	values: CommunicationFolderFormValues,
	accountId: string | null
): CommunicationFolderInput {
	const parsed = mailFolderFormSchema.parse(values)
	return {
		account_id: accountId?.trim() || null,
		name: parsed.name,
		description: parsed.description || null,
		color: parsed.color || null,
		sort_order: parsed.sort_order
	}
}

export function mailFolderDeleteDialogCopy(
	folder: Pick<CommunicationFolder, 'name'>
): CommunicationFolderDeleteDialogCopy {
	return {
		title: 'Delete folder',
		message: `Delete the folder "${folder.name}"? This does not delete messages.`,
		confirmLabel: 'Delete'
	}
}

export function mailFolderMessageCountLabel(
	folder: Pick<CommunicationFolder, 'message_count'>
): string {
	return String(Math.max(0, Math.trunc(folder.message_count)))
}

function normalizeCommunicationFolderPath(value: string): string {
	return normalizeCommunicationFolderPathParts(value).join(' / ')
}

function normalizeCommunicationFolderPathParts(value: string): string[] {
	return value
		.split('/')
		.map((part) => part.trim())
		.filter(Boolean)
}

function isSameOrDescendantPath(candidatePath: string, currentPath: string): boolean {
	if (!candidatePath || !currentPath) return false
	if (candidatePath === currentPath) return true
	return candidatePath.startsWith(`${currentPath} / `)
}
```

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

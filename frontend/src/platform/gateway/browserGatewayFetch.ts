const LEGACY_SECRET_HEADER = 'x-makosh-secret'

export type BrowserGatewayFetchOptions = {
	origin?: string
	fetchImpl?: typeof fetch
}

/**
 * Same-origin fetch boundary for the future browser Gateway client.
 *
 * It deliberately knows nothing about business routes or legacy local secrets.
 * Browser authentication is supplied exclusively by the HttpOnly same-origin
 * Gateway session cookie.
 */
export class BrowserGatewayFetch {
	private readonly origin: string
	private readonly fetchImpl: typeof fetch

	constructor(options: BrowserGatewayFetchOptions = {}) {
		this.origin = exactBrowserOrigin(options.origin ?? browserOrigin())
		this.fetchImpl = options.fetchImpl ?? globalThis.fetch.bind(globalThis)
	}

	fetch(input: RequestInfo | URL, init: RequestInit = {}): Promise<Response> {
		const url = sameOriginPath(requestUrl(input), this.origin)
		const headers = requestHeaders(input, init)
		forbidLegacyCredentials(headers)
		return this.fetchImpl(url, {
			...init,
			headers,
			cache: init.cache ?? 'no-store',
			credentials: 'same-origin',
			mode: 'same-origin',
			redirect: 'error',
		})
	}
}

function requestUrl(input: RequestInfo | URL): string {
	if (typeof input === 'string') return input
	if (input instanceof URL) return input.toString()
	return input.url
}

function requestHeaders(input: RequestInfo | URL, init: RequestInit): Headers {
	const headers = new Headers(input instanceof Request ? input.headers : undefined)
	for (const [name, value] of new Headers(init.headers)) headers.set(name, value)
	return headers
}

function browserOrigin(): string {
	const origin = globalThis.location?.origin
	if (!origin) throw new Error('browser Gateway origin is unavailable')
	return origin
}

function exactBrowserOrigin(value: string): string {
	const origin = new URL(value)
	const localDevelopment = origin.protocol === 'http:' && isLocalDevelopmentHost(origin.hostname)
	if ((origin.protocol !== 'https:' && !localDevelopment)
		|| origin.username || origin.password || origin.pathname !== '/' || origin.search || origin.hash) {
		throw new Error('browser Gateway origin is invalid')
	}
	return origin.origin
}

function isLocalDevelopmentHost(hostname: string): boolean {
	if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '[::1]') return true
	const ipv4 = hostname.split('.').map(Number)
	if (ipv4.length === 4 && ipv4.every((part) => Number.isInteger(part) && part >= 0 && part <= 255)) {
		return ipv4[0] === 10
			|| (ipv4[0] === 172 && ipv4[1]! >= 16 && ipv4[1]! <= 31)
			|| (ipv4[0] === 192 && ipv4[1] === 168)
			|| (ipv4[0] === 169 && ipv4[1] === 254)
	}
	const ipv6 = hostname.replace(/^\[|\]$/g, '').toLowerCase()
	return /^f[cd][0-9a-f]{2}:/.test(ipv6) || /^fe[89ab][0-9a-f]:/.test(ipv6)
}

function sameOriginPath(path: string, origin: string): string {
	const url = new URL(path, origin)
	if (url.origin !== origin || !url.pathname.startsWith('/')) {
		throw new Error('browser Gateway request must remain same-origin')
	}
	return `${url.pathname}${url.search}`
}

function forbidLegacyCredentials(headers: Headers): void {
	if (headers.has(LEGACY_SECRET_HEADER) || headers.has('authorization')) {
		throw new Error('browser Gateway request cannot carry a legacy credential')
	}
}
